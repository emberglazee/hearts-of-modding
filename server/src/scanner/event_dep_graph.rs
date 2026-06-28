#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::data::layered_value::LayeredValue;
use crate::scanner::event_scanner::Event;
use dashmap::DashMap;
use std::collections::HashSet;

/// A concurrent, directed dependency graph of event trigger relationships.
///
/// Maintains both **forward** edges (caller → callees) and **reverse** edges
/// (callee → callers), enabling O(1) lookups for:
///
/// - "Which events does `X` trigger/call?"             → [`callees_of`]
/// - "Which events call `X`?"                           → [`callers_of`]
/// - "Is event `X` orphaned (no caller references)?"    → [`is_orphaned`]
///
/// # Initial scan
///
/// After the full event scan populates `ScannerData.events`, call
/// [`rebuild_from_events_db`] once to build the graph from scratch.
///
/// # Incremental update (file edit)
///
/// When a single event file is modified (via `did_change_watched_files`):
///
/// 1. **Before** `retain_path!`, collect the old event IDs for that path
///    from the companion `_file_index` (`events_file_index`).
/// 2. Call [`remove_callers`] with those IDs to strip outgoing edges
///    (incoming edges from unchanged files are left intact).
/// 3. Re-parse the file and call `find_triggers_in_script` to populate
///    `triggered_events` on the new `Event` structs.
/// 4. **After** `retain_path!`, call [`add_edge`] for each new trigger.
///
/// This is O(K) in the number of events in the edited file — no full
/// workspace rescan required.
pub(crate) struct EventDependencyGraph {
    /// caller_id -> set of callee_ids (events that caller directly triggers)
    forward: DashMap<String, HashSet<String>>,
    /// callee_id -> set of caller_ids (events that reference this callee)
    reverse: DashMap<String, HashSet<String>>,
}

impl EventDependencyGraph {
    pub(crate) fn new() -> Self {
        Self {
            forward: DashMap::new(),
            reverse: DashMap::new(),
        }
    }

    /// Add a single directed edge: `caller` triggers/refers-to `callee`.
    ///
    /// Updates both the forward map and the reverse index atomically.
    /// Duplicate edges are idempotent (the inner `HashSet` dedupes).
    pub(crate) fn add_edge(&self, caller: &str, callee: &str) {
        self.forward
            .entry(caller.to_string())
            .or_default()
            .insert(callee.to_string());
        self.reverse
            .entry(callee.to_string())
            .or_default()
            .insert(caller.to_string());
    }

    /// Remove ALL outgoing edges for a single caller event.
    ///
    /// Also removes this caller from the reverse index of its former
    /// callees. Incoming edges (from other callers to this event) are
    /// **not** affected — only outgoing edges are stripped.
    pub(crate) fn remove_caller(&self, caller: &str) {
        if let Some((_, callees)) = self.forward.remove(caller) {
            for callee in &callees {
                if let Some(mut callers) = self.reverse.get_mut(callee) {
                    callers.remove(caller);
                    if callers.is_empty() {
                        drop(callers);
                        self.reverse.remove(callee);
                    }
                }
            }
        }
    }

    /// Remove ALL outgoing edges for a batch of caller events.
    ///
    /// A convenience wrapper around [`remove_caller`] — use this when
    /// updating a file that defines multiple events.
    pub(crate) fn remove_callers(&self, callers: &[String]) {
        for caller in callers {
            self.remove_caller(caller);
        }
    }

    /// Events directly triggered/called by `event_id`.
    ///
    /// Returns an empty `Vec` if `event_id` has no outgoing edges
    /// or is not in the graph.
    pub(crate) fn callees_of(&self, event_id: &str) -> Vec<String> {
        self.forward
            .get(event_id)
            .map(|e| e.value().iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Events that directly trigger/call `event_id`.
    ///
    /// Returns an empty `Vec` if no other event references `event_id`.
    pub(crate) fn callers_of(&self, event_id: &str) -> Vec<String> {
        self.reverse
            .get(event_id)
            .map(|e| e.value().iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Number of events that directly call `event_id`.
    pub(crate) fn caller_count(&self, event_id: &str) -> usize {
        self.reverse.get(event_id).map_or(0, |e| e.value().len())
    }

    /// Whether `event_id` has zero incoming edges (no other event calls it).
    ///
    /// An orphaned event may still be legitimate if it's triggered by
    /// engine mechanics (on_startup, national_focus, decisions, etc.),
    /// but it's a strong signal that something may be wrong.
    pub(crate) fn is_orphaned(&self, event_id: &str) -> bool {
        self.reverse
            .get(event_id)
            .is_none_or(|e| e.value().is_empty())
    }

    /// Total number of distinct caller events in the graph.
    pub(crate) fn caller_count_total(&self) -> usize {
        self.forward.len()
    }

    /// Total number of distinct callee events in the graph.
    pub(crate) fn callee_count_total(&self) -> usize {
        // Only count callees that are the target of at least one edge
        let mut count = 0;
        for entry in self.reverse.iter() {
            if !entry.value().is_empty() {
                count += 1;
            }
        }
        count
    }

    /// Clear all edges from the graph.
    pub(crate) fn clear(&self) {
        self.forward.clear();
        self.reverse.clear();
    }

    /// Rebuild the entire graph from the current `events` DashMap.
    ///
    /// Call this once after the initial full event scan completes.
    ///
    /// For incremental updates (file edits), use [`remove_caller`] +
    /// [`add_edge`] instead to avoid rebuilding the whole graph.
    pub(crate) fn rebuild_from_events_db(
        &self,
        events: &DashMap<InternedStr, LayeredValue<Event>>,
    ) {
        self.clear();
        for entry in events.iter() {
            let event = entry.value().resolve();
            let caller = &event.id;
            for callee in &event.triggered_events {
                self.add_edge(caller, callee);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast;
    use std::sync::Arc;

    fn make_event(id: &str, triggered: &[&str]) -> Event {
        Event {
            id: id.to_string(),
            event_type: "country_event".to_string(),
            path: Arc::from("events/test.txt"),
            range: ast::Range {
                start_line: 1,
                start_col: 0,
                end_line: 1,
                end_col: 15,
            },
            triggered_events: triggered.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn make_layered(event: Event) -> LayeredValue<Event> {
        LayeredValue::new(event)
    }

    #[test]
    fn test_add_and_query_edge() {
        let graph = EventDependencyGraph::new();
        graph.add_edge("A.1", "B.1");
        graph.add_edge("A.1", "C.1");
        graph.add_edge("B.1", "C.1");

        let a_callees = graph.callees_of("A.1");
        assert_eq!(a_callees.len(), 2);
        assert!(a_callees.contains(&"B.1".to_string()));
        assert!(a_callees.contains(&"C.1".to_string()));

        let c_callers = graph.callers_of("C.1");
        assert_eq!(c_callers.len(), 2);
        assert!(c_callers.contains(&"A.1".to_string()));
        assert!(c_callers.contains(&"B.1".to_string()));

        assert_eq!(graph.caller_count("C.1"), 2);
        assert_eq!(graph.caller_count("Z.1"), 0);
    }

    #[test]
    fn test_orphan_detection() {
        let graph = EventDependencyGraph::new();
        graph.add_edge("A.1", "B.1");
        graph.add_edge("B.1", "C.1");

        // A.1 has no incoming edges → orphaned
        assert!(graph.is_orphaned("A.1"));
        // B.1 is called by A.1 → not orphaned
        assert!(!graph.is_orphaned("B.1"));
        // C.1 is called by B.1 → not orphaned
        assert!(!graph.is_orphaned("C.1"));
        // Z.1 doesn't exist → considered orphaned (no reverse entry)
        assert!(graph.is_orphaned("Z.1"));
    }

    #[test]
    fn test_remove_caller() {
        let graph = EventDependencyGraph::new();
        graph.add_edge("A.1", "B.1");
        graph.add_edge("B.1", "C.1");
        graph.add_edge("A.1", "C.1");

        // Remove A.1's outgoing edges
        graph.remove_caller("A.1");
        assert!(graph.callees_of("A.1").is_empty());
        // C.1 should still be called by B.1
        assert_eq!(graph.caller_count("C.1"), 1);
        assert!(graph.callers_of("C.1").contains(&"B.1".to_string()));
        // B.1 should now have no callers → orphaned
        assert!(graph.is_orphaned("B.1"));
    }

    #[test]
    fn test_remove_callers_batch() {
        let graph = EventDependencyGraph::new();
        graph.add_edge("A.1", "B.1");
        graph.add_edge("A.2", "C.1");
        graph.add_edge("A.3", "B.1");

        graph.remove_callers(&["A.1".to_string(), "A.2".to_string()]);
        assert!(graph.callees_of("A.1").is_empty());
        assert!(graph.callees_of("A.2").is_empty());
        // B.1 should still be called by A.3
        assert_eq!(graph.caller_count("B.1"), 1);
    }

    #[test]
    fn test_rebuild_from_events_db() {
        let events: DashMap<InternedStr, LayeredValue<Event>> = DashMap::new();
        events.insert(
            Arc::from("A.1"),
            make_layered(make_event("A.1", &["B.1", "C.1"])),
        );
        events.insert(Arc::from("B.1"), make_layered(make_event("B.1", &["C.1"])));
        events.insert(Arc::from("C.1"), make_layered(make_event("C.1", &[])));

        let graph = EventDependencyGraph::new();
        graph.rebuild_from_events_db(&events);

        assert_eq!(graph.callees_of("A.1").len(), 2);
        assert!(graph.callees_of("A.1").contains(&"B.1".to_string()));
        assert_eq!(graph.callees_of("B.1").len(), 1);
        assert!(graph.is_orphaned("A.1")); // no incoming edges
        assert!(!graph.is_orphaned("B.1")); // called by A.1
    }

    #[test]
    fn test_clear() {
        let graph = EventDependencyGraph::new();
        graph.add_edge("A.1", "B.1");
        graph.clear();
        assert!(graph.callees_of("A.1").is_empty());
        assert!(graph.callers_of("B.1").is_empty());
        assert_eq!(graph.caller_count_total(), 0);
    }

    #[test]
    fn test_idempotent_duplicate_edge() {
        let graph = EventDependencyGraph::new();
        graph.add_edge("A.1", "B.1");
        graph.add_edge("A.1", "B.1"); // same edge again
        assert_eq!(graph.callees_of("A.1").len(), 1);
        assert_eq!(graph.callers_of("B.1").len(), 1);
    }
}
