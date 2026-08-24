#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::data::layered_value::LayeredValue;
use crate::scanner::technology_scanner::Technology;
use dashmap::DashMap;
use std::collections::HashSet;

/// A concurrent, directed dependency graph of technology tree relationships.
///
/// Maintains both **forward** edges (tech → leads_to_tech) and **reverse** edges
/// (leads_to_tech -> techs that point at it), enabling O(1) lookups for:
///
/// - "Which technologies does `X` lead to?"      → [`callees_of`]
/// - "Which technologies lead to `X`?"           → [`callers_of`]
/// - "Is tech `X` orphaned (no tech points at it)?" → [`is_orphaned`]
///
/// # Initial scan
///
/// After the full technology scan populates `ScannerData.technologies`, call
/// [`rebuild_from_technologies_db`] once to build the graph from scratch.
///
/// # Incremental update (file edit)
///
/// When a single technology file is modified:
///
/// 1. **Before** `retain_path!`, collect the old tech IDs for that path
///    from the companion `_file_index` (`technologies_file_index`).
/// 2. Call [`remove_callers`] with those IDs to strip outgoing edges.
/// 3. Re-parse the file and populate `leads_to_tech` on the new structs.
/// 4. **After** `retain_path!`, call [`add_edge`] for each new leads_to_tech.
pub(crate) struct TechDependencyGraph {
    /// caller_id -> set of callee_ids (techs that this tech leads to)
    forward: DashMap<String, HashSet<String>>,
    /// callee_id -> set of caller_ids (techs that lead to this callee)
    reverse: DashMap<String, HashSet<String>>,
}

impl TechDependencyGraph {
    pub(crate) fn new() -> Self {
        Self {
            forward: DashMap::new(),
            reverse: DashMap::new(),
        }
    }

    /// Add a single directed edge: `caller` leads-to `callee`.
    ///
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

    /// Remove ALL outgoing edges for a single caller tech.
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

    /// Remove ALL outgoing edges for a batch of caller techs.
    pub(crate) fn remove_callers(&self, callers: &[String]) {
        for caller in callers {
            self.remove_caller(caller);
        }
    }

    /// Remove a set of techs from the graph entirely — as callers AND as
    /// callees (i.e. a technology file was deleted).
    pub(crate) fn remove_techs(&self, ids: &[String]) {
        for id in ids {
            self.remove_caller(id);
            for mut entry in self.forward.iter_mut() {
                entry.value_mut().remove(id.as_str());
            }
            self.reverse.remove(id.as_str());
        }
    }

    /// Technologies directly led-to by `tech_id`.
    pub(crate) fn callees_of(&self, tech_id: &str) -> Vec<String> {
        self.forward
            .get(tech_id)
            .map(|e| e.value().iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Technologies that directly lead to `tech_id`.
    pub(crate) fn callers_of(&self, tech_id: &str) -> Vec<String> {
        self.reverse
            .get(tech_id)
            .map(|e| e.value().iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Number of technologies that directly lead to `tech_id`.
    pub(crate) fn caller_count(&self, tech_id: &str) -> usize {
        self.reverse.get(tech_id).map_or(0, |e| e.value().len())
    }

    /// Whether `tech_id` has zero incoming edges (no tech leads to it).
    pub(crate) fn is_orphaned(&self, tech_id: &str) -> bool {
        self.reverse
            .get(tech_id)
            .is_none_or(|e| e.value().is_empty())
    }

    /// Clear all edges from the graph.
    pub(crate) fn clear(&self) {
        self.forward.clear();
        self.reverse.clear();
    }

    /// Rebuild the entire graph from the current `technologies` DashMap.
    ///
    /// Call this once after the initial full technology scan completes.
    pub(crate) fn rebuild_from_technologies_db(
        &self,
        technologies: &DashMap<InternedStr, LayeredValue<Technology>>,
    ) {
        self.clear();
        for entry in technologies.iter() {
            let tech = entry.value().resolve();
            for callee in &tech.leads_to_tech {
                // Skip self-edges, matching update_technologies: a tech cannot
                // be its own prerequisite, and keeping the two paths identical
                // means hover content doesn't depend on how the graph got
                // built (startup scan vs file edit).
                if callee != &tech.name {
                    self.add_edge(&tech.name, callee);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast;
    use std::sync::Arc;

    fn make_tech(name: &str, leads_to: &[&str]) -> Technology {
        Technology {
            name: name.to_string(),
            path: Arc::from("common/technologies/test.txt"),
            range: ast::Range {
                start_line: 1,
                start_col: 0,
                end_line: 1,
                end_col: name.len() as u32,
            },
            start_year: None,
            research_cost: None,
            categories: Vec::new(),
            folder: None,
            leads_to_tech: leads_to.iter().map(|s| s.to_string()).collect(),
            xor: Vec::new(),
            dependencies: Vec::new(),
            enable_subunits: Vec::new(),
            enable_equipments: Vec::new(),
            enable_equipment_modules: Vec::new(),
            enable_building: None,
            sub_technologies: Vec::new(),
        }
    }

    fn make_layered(tech: Technology) -> LayeredValue<Technology> {
        LayeredValue::new(tech)
    }

    #[test]
    fn test_add_and_query_edge() {
        let graph = TechDependencyGraph::new();
        graph.add_edge("tech_a", "tech_b");
        graph.add_edge("tech_a", "tech_c");
        graph.add_edge("tech_b", "tech_c");

        let a_callees = graph.callees_of("tech_a");
        assert_eq!(a_callees.len(), 2);
        assert!(a_callees.contains(&"tech_b".to_string()));
        assert!(a_callees.contains(&"tech_c".to_string()));

        let c_callers = graph.callers_of("tech_c");
        assert_eq!(c_callers.len(), 2);
        assert!(c_callers.contains(&"tech_a".to_string()));
        assert!(c_callers.contains(&"tech_b".to_string()));

        assert_eq!(graph.caller_count("tech_c"), 2);
        assert_eq!(graph.caller_count("tech_z"), 0);
    }

    #[test]
    fn test_orphan_detection() {
        let graph = TechDependencyGraph::new();
        graph.add_edge("tech_a", "tech_b");
        graph.add_edge("tech_b", "tech_c");

        // tech_a has no incoming edges → orphaned
        assert!(graph.is_orphaned("tech_a"));
        // tech_b is led-to by tech_a → not orphaned
        assert!(!graph.is_orphaned("tech_b"));
        // tech_c is led-to by tech_b → not orphaned
        assert!(!graph.is_orphaned("tech_c"));
        // tech_z doesn't exist → considered orphaned
        assert!(graph.is_orphaned("tech_z"));
    }

    #[test]
    fn test_remove_caller() {
        let graph = TechDependencyGraph::new();
        graph.add_edge("tech_a", "tech_b");
        graph.add_edge("tech_b", "tech_c");
        graph.add_edge("tech_a", "tech_c");

        graph.remove_caller("tech_a");
        assert!(graph.callees_of("tech_a").is_empty());
        assert_eq!(graph.caller_count("tech_c"), 1);
        assert!(graph.callers_of("tech_c").contains(&"tech_b".to_string()));
        assert!(graph.is_orphaned("tech_b"));
    }

    #[test]
    fn test_remove_callers_batch() {
        let graph = TechDependencyGraph::new();
        graph.add_edge("tech_a", "tech_c");
        graph.add_edge("tech_b", "tech_c");

        graph.remove_callers(&["tech_a".to_string(), "tech_b".to_string()]);
        assert!(graph.callees_of("tech_a").is_empty());
        assert!(graph.callees_of("tech_b").is_empty());
        assert_eq!(graph.caller_count("tech_c"), 0);
    }

    #[test]
    fn test_remove_techs() {
        let graph = TechDependencyGraph::new();
        graph.add_edge("tech_a", "tech_b");
        graph.add_edge("tech_b", "tech_c");

        graph.remove_techs(&["tech_b".to_string()]);
        // tech_b's outgoing edges gone
        assert!(graph.callees_of("tech_b").is_empty());
        // tech_b removed as callee of tech_a
        assert!(!graph.callees_of("tech_a").contains(&"tech_b".to_string()));
        // tech_b's reverse index gone
        assert!(graph.callers_of("tech_b").is_empty());
    }

    #[test]
    fn test_rebuild_from_technologies_db() {
        let db: DashMap<InternedStr, LayeredValue<Technology>> = DashMap::new();
        db.insert(
            Arc::from("tech_a"),
            make_layered(make_tech("tech_a", &["tech_b", "tech_c"])),
        );
        db.insert(
            Arc::from("tech_b"),
            make_layered(make_tech("tech_b", &["tech_c"])),
        );

        let graph = TechDependencyGraph::new();
        graph.rebuild_from_technologies_db(&db);

        assert_eq!(graph.callees_of("tech_a").len(), 2);
        assert!(graph.callees_of("tech_a").contains(&"tech_b".to_string()));
        assert!(graph.callees_of("tech_a").contains(&"tech_c".to_string()));
        assert_eq!(graph.caller_count("tech_c"), 2);
    }

    /// Self-referential edges (`leads_to_tech` pointing at the tech itself)
    /// are skipped by the incremental update path (update_technologies) and
    /// must be skipped by the full rebuild too — otherwise hover content
    /// differs depending on whether the last write was a startup scan or an
    /// edit. A tech cannot meaningfully be its own prerequisite.
    #[test]
    fn test_rebuild_skips_self_edges() {
        let db: DashMap<InternedStr, LayeredValue<Technology>> = DashMap::new();
        db.insert(
            Arc::from("loop_tech"),
            make_layered(make_tech("loop_tech", &["loop_tech", "other_tech"])),
        );
        db.insert(
            Arc::from("other_tech"),
            make_layered(make_tech("other_tech", &[])),
        );

        let graph = TechDependencyGraph::new();
        graph.rebuild_from_technologies_db(&db);

        assert_eq!(
            graph.callees_of("loop_tech"),
            vec!["other_tech".to_string()],
            "self-edge must be dropped at rebuild, matching incremental updates"
        );
        assert_eq!(graph.caller_count("loop_tech"), 0);
    }
}
