use crate::parser::ast::{Entry, Range, Value};
use crate::scanner::incremental_scanner::index_key;
use crate::utils::lsp_convert::RangeMapper;
use std::collections::HashMap;
use std::sync::Arc;
use tower_lsp_server::ls_types::{
    CallHierarchyIncomingCall, CallHierarchyItem, CallHierarchyOutgoingCall,
    Position as LspPosition, SymbolKind, Uri,
};

fn path_to_url(path: &str) -> Uri {
    let abs_path = std::path::Path::new(path)
        .canonicalize()
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default().join(path));
    Uri::from_file_path(&abs_path).unwrap_or_else(|| {
        format!("file://{}", abs_path.to_string_lossy().replace("\\", "/"))
            .parse::<Uri>()
            .unwrap()
    })
}

/// Call hierarchy information for a symbol
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CallInfo {
    pub name: String,
    pub kind: CallKind,
    pub path: String,
    pub range: Range,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum CallKind {
    Event,
    ScriptedTrigger,
    ScriptedEffect,
}

/// Prepare call hierarchy item at the given position
///
/// `content` is the open document's text (used to convert the client's UTF-16
/// position to byte columns for hit-testing against scanner-data ranges);
/// `None` falls back to reading the file from disk (unopened files), matching
/// the rename handler's pattern.
pub async fn prepare_call_hierarchy(
    uri: &str,
    position: LspPosition,
    data: &crate::ScannerData,
    content: Option<&str>,
) -> Option<CallHierarchyItem> {
    let parsed_uri = uri.parse::<Uri>().ok()?;
    let path = parsed_uri.to_file_path()?;
    let path = path.to_string_lossy();

    // Inception contract: the client position is UTF-16 code units but entity
    // ranges are byte columns. Convert once at this public entry — the source
    // we convert against must match what produced the ranges (the open
    // document, which the incremental scanner keeps in sync).
    let source = match content {
        Some(c) => c.to_owned(),
        None => std::fs::read_to_string(&*path).unwrap_or_default(),
    };
    let pos = crate::utils::lsp_convert::to_byte_position(&source, position);
    let mapper = RangeMapper::new(&source);
    let path_ref: &str = &path;

    // Check if position is on an event — only entities declared in THIS file
    // (via the reverse per-path index), not a scan of every event in the
    // workspace.
    if let Some(names) = data.events_file_index.get(&index_key(path_ref)) {
        for name in names.value() {
            if let Some(event) = data.events.get(&**name) {
                let event = event.value();
                if position_in_range(&pos, &event.range) {
                    return Some(CallHierarchyItem {
                        name: name.to_string(),
                        kind: SymbolKind::EVENT,
                        tags: None,
                        detail: Some(format!("{:?}", event.event_type)),
                        uri: path_to_url(&event.path),
                        range: mapper.range(&event.range),
                        selection_range: mapper.range(&event.range),
                        data: None,
                    });
                }
            }
        }
    }

    // Check if position is on a scripted trigger
    if let Some(names) = data.scripted_triggers_file_index.get(&index_key(path_ref)) {
        for name in names.value() {
            if let Some(trigger) = data.scripted_triggers.get(&**name) {
                let trigger = trigger.value();
                if position_in_range(&pos, &trigger.range) {
                    return Some(CallHierarchyItem {
                        name: name.to_string(),
                        kind: SymbolKind::FUNCTION,
                        tags: None,
                        detail: Some("Scripted Trigger".to_string()),
                        uri: path_to_url(&trigger.path),
                        range: mapper.range(&trigger.range),
                        selection_range: mapper.range(&trigger.range),
                        data: None,
                    });
                }
            }
        }
    }

    // Check if position is on a scripted effect
    if let Some(names) = data.scripted_effects_file_index.get(&index_key(path_ref)) {
        for name in names.value() {
            if let Some(effect) = data.scripted_effects.get(&**name) {
                let effect = effect.value();
                if position_in_range(&pos, &effect.range) {
                    return Some(CallHierarchyItem {
                        name: name.to_string(),
                        kind: SymbolKind::FUNCTION,
                        tags: None,
                        detail: Some("Scripted Effect".to_string()),
                        uri: path_to_url(&effect.path),
                        range: mapper.range(&effect.range),
                        selection_range: mapper.range(&effect.range),
                        data: None,
                    });
                }
            }
        }
    }

    None
}

/// Get incoming calls (who calls this symbol)
pub async fn get_incoming_calls(
    item: &CallHierarchyItem,
    data: &crate::ScannerData,
    document_asts: &dashmap::DashMap<
        String,
        (
            Arc<crate::parser::ast::Script>,
            Vec<(String, crate::parser::ast::Range)>,
        ),
    >,
) -> Vec<CallHierarchyIncomingCall> {
    let mut incoming = Vec::new();
    let target_name = &item.name;

    // Search for references in all documents
    for entry in document_asts.iter() {
        let uri = entry.key();
        let (script, _) = entry.value();
        let mapper = RangeMapper::new(&script.source);

        let references = find_references_in_entries(&script.entries, target_name, &script.source);

        if !references.is_empty() {
            // Try to find the containing symbol
            if let Some(container) = find_container_symbol(uri, &references[0], data).await {
                incoming.push(CallHierarchyIncomingCall {
                    from: container,
                    from_ranges: references.iter().map(|r| mapper.range(r)).collect(),
                });
            }
        }
    }

    incoming
}

/// Get outgoing calls (what this symbol calls)
pub async fn get_outgoing_calls(
    item: &CallHierarchyItem,
    data: &crate::ScannerData,
    document_asts: &dashmap::DashMap<
        String,
        (
            Arc<crate::parser::ast::Script>,
            Vec<(String, crate::parser::ast::Range)>,
        ),
    >,
) -> Vec<CallHierarchyOutgoingCall> {
    let mut outgoing = Vec::new();

    // The item's LSP range is UTF-16 (as emitted to the client); round-tripping
    // it through lsp_to_range and comparing against byte-based AST ranges would
    // mis-overlap on multi-byte lines. Resolve the symbol's BYTE range from
    // scanner data instead — it is the same range the client's item was built
    // from, just pre-UTF-16-conversion.
    let symbol_range = if let Some(event) = data.events.get(item.name.as_str()) {
        Some(event.range.clone())
    } else if let Some(trigger) = data.scripted_triggers.get(item.name.as_str()) {
        Some(trigger.range.clone())
    } else if let Some(effect) = data.scripted_effects.get(item.name.as_str()) {
        Some(effect.range.clone())
    } else {
        None
    };

    // Get the document content
    if let Some(entry) = document_asts.get(item.uri.as_str()) {
        let (script, _) = &*entry;
        let mapper = RangeMapper::new(&script.source);

        // Find all calls within this symbol's range
        if let Some(symbol_range) = symbol_range {
            let calls = find_calls_in_range(&script.entries, &symbol_range, &script.source);

            for (call_name, call_ranges) in calls {
                // Try to find the target symbol
                if let Some(target) = find_symbol_by_name(&call_name, data).await {
                    outgoing.push(CallHierarchyOutgoingCall {
                        to: target,
                        from_ranges: call_ranges.iter().map(|r| mapper.range(r)).collect(),
                    });
                }
            }
        }
    }

    outgoing
}

/// Find references to a symbol in AST entries
fn find_references_in_entries(entries: &[Entry], target: &str, source: &str) -> Vec<Range> {
    let mut references = Vec::new();

    for entry in entries {
        find_references_recursive(entry, target, &mut references, source);
    }

    references
}

fn find_references_recursive(
    entry: &Entry,
    target: &str,
    references: &mut Vec<Range>,
    source: &str,
) {
    if let Entry::Assignment(ass) = entry {
        // Check for event triggers: country_event = { id = target }
        if ass.key_text(source) == "country_event"
            || ass.key_text(source) == "state_event"
            || ass.key_text(source) == "news_event"
        {
            if let Value::Block(children) = &ass.value.value {
                for child in children {
                    if let Entry::Assignment(child_ass) = child {
                        if child_ass.key_text(source) == "id" {
                            if let Some(id) = child_ass.value.value.as_str(source) {
                                if id == target {
                                    let range = Range {
                                        start_line: ass.key_range.start_line,
                                        start_col: ass.key_range.start_col,
                                        end_line: ass.value.range.end_line,
                                        end_col: ass.value.range.end_col,
                                    };
                                    references.push(range);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check for scripted trigger/effect calls
        if let Some(s) = ass.value.value.as_str(source) {
            if s == target {
                let range = Range {
                    start_line: ass.key_range.start_line,
                    start_col: ass.key_range.start_col,
                    end_line: ass.value.range.end_line,
                    end_col: ass.value.range.end_col,
                };
                references.push(range);
            }
        }

        // Recurse into blocks
        if let Value::Block(children) = &ass.value.value {
            for child in children {
                find_references_recursive(child, target, references, source);
            }
        }
    }
}

/// Find all calls within a specific range
fn find_calls_in_range(
    entries: &[Entry],
    range: &Range,
    source: &str,
) -> HashMap<String, Vec<Range>> {
    let mut calls = HashMap::new();

    for entry in entries {
        find_calls_recursive(entry, range, &mut calls, source);
    }

    calls
}

fn find_calls_recursive(
    entry: &Entry,
    target_range: &Range,
    calls: &mut HashMap<String, Vec<Range>>,
    source: &str,
) {
    if let Entry::Assignment(ass) = entry {
        let range = Range {
            start_line: ass.key_range.start_line,
            start_col: ass.key_range.start_col,
            end_line: ass.value.range.end_line,
            end_col: ass.value.range.end_col,
        };

        if !range_overlaps(&range, target_range) {
            return;
        }

        // Check for event triggers
        if ass.key_text(source) == "country_event"
            || ass.key_text(source) == "state_event"
            || ass.key_text(source) == "news_event"
        {
            if let Value::Block(children) = &ass.value.value {
                for child in children {
                    if let Entry::Assignment(child_ass) = child {
                        if child_ass.key_text(source) == "id" {
                            if let Some(id) = child_ass.value.value.as_str(source) {
                                calls.entry(id.to_string()).or_default().push(range.clone());
                            }
                        }
                    }
                }
            }
        }

        // Check for scripted trigger/effect calls
        if let Some(s) = ass.value.value.as_str(source) {
            calls.entry(s.to_string()).or_default().push(range.clone());
        }

        // Recurse into blocks
        if let Value::Block(children) = &ass.value.value {
            for child in children {
                find_calls_recursive(child, target_range, calls, source);
            }
        }
    }
}

/// Find the container symbol for a given position
async fn find_container_symbol(
    uri: &str,
    range: &Range,
    data: &crate::ScannerData,
) -> Option<CallHierarchyItem> {
    let parsed_uri = uri.parse::<Uri>().ok()?;
    let path = parsed_uri.to_file_path()?;
    let path = path.to_string_lossy();

    // Check events — only entities declared in THIS file (reverse per-path
    // index), not a scan of every event in the workspace.
    if let Some(names) = data.events_file_index.get(&index_key(path.as_ref())) {
        for name in names.value() {
            if let Some(event) = data.events.get(&**name) {
                let event = event.value();
                if range_contains(&event.range, range) {
                    let content = std::fs::read_to_string(&*event.path).unwrap_or_default();
                    let mapper = RangeMapper::new(&content);
                    return Some(CallHierarchyItem {
                        name: name.to_string(),
                        kind: SymbolKind::EVENT,
                        tags: None,
                        detail: Some(format!("{:?}", event.event_type)),
                        uri: path_to_url(&event.path),
                        range: mapper.range(&event.range),
                        selection_range: mapper.range(&event.range),
                        data: None,
                    });
                }
            }
        }
    }

    // Check scripted triggers
    if let Some(names) = data
        .scripted_triggers_file_index
        .get(&index_key(path.as_ref()))
    {
        for name in names.value() {
            if let Some(trigger) = data.scripted_triggers.get(&**name) {
                let trigger = trigger.value();
                if range_contains(&trigger.range, range) {
                    let content = std::fs::read_to_string(&*trigger.path).unwrap_or_default();
                    let mapper = RangeMapper::new(&content);
                    return Some(CallHierarchyItem {
                        name: name.to_string(),
                        kind: SymbolKind::FUNCTION,
                        tags: None,
                        detail: Some("Scripted Trigger".to_string()),
                        uri: path_to_url(&trigger.path),
                        range: mapper.range(&trigger.range),
                        selection_range: mapper.range(&trigger.range),
                        data: None,
                    });
                }
            }
        }
    }

    // Check scripted effects
    if let Some(names) = data
        .scripted_effects_file_index
        .get(&index_key(path.as_ref()))
    {
        for name in names.value() {
            if let Some(effect) = data.scripted_effects.get(&**name) {
                let effect = effect.value();
                if range_contains(&effect.range, range) {
                    let content = std::fs::read_to_string(&*effect.path).unwrap_or_default();
                    let mapper = RangeMapper::new(&content);
                    return Some(CallHierarchyItem {
                        name: name.to_string(),
                        kind: SymbolKind::FUNCTION,
                        tags: None,
                        detail: Some("Scripted Effect".to_string()),
                        uri: path_to_url(&effect.path),
                        range: mapper.range(&effect.range),
                        selection_range: mapper.range(&effect.range),
                        data: None,
                    });
                }
            }
        }
    }

    None
}

/// Find a symbol by name
async fn find_symbol_by_name(name: &str, data: &crate::ScannerData) -> Option<CallHierarchyItem> {
    // Check events
    let events = &data.events;
    if let Some(event) = events.get(name) {
        let content = std::fs::read_to_string(&*event.path).unwrap_or_default();
        let mapper = RangeMapper::new(&content);
        return Some(CallHierarchyItem {
            name: name.to_string(),
            kind: SymbolKind::EVENT,
            tags: None,
            detail: Some(format!("{:?}", event.event_type)),
            uri: path_to_url(&event.path),
            range: mapper.range(&event.range),
            selection_range: mapper.range(&event.range),
            data: None,
        });
    }

    // Check scripted triggers
    let triggers = &data.scripted_triggers;
    if let Some(trigger) = triggers.get(name) {
        let content = std::fs::read_to_string(&*trigger.path).unwrap_or_default();
        let mapper = RangeMapper::new(&content);
        return Some(CallHierarchyItem {
            name: name.to_string(),
            kind: SymbolKind::FUNCTION,
            tags: None,
            detail: Some("Scripted Trigger".to_string()),
            uri: path_to_url(&trigger.path),
            range: mapper.range(&trigger.range),
            selection_range: mapper.range(&trigger.range),
            data: None,
        });
    }

    // Check scripted effects
    let effects = &data.scripted_effects;
    if let Some(effect) = effects.get(name) {
        let content = std::fs::read_to_string(&*effect.path).unwrap_or_default();
        let mapper = RangeMapper::new(&content);
        return Some(CallHierarchyItem {
            name: name.to_string(),
            kind: SymbolKind::FUNCTION,
            tags: None,
            detail: Some("Scripted Effect".to_string()),
            uri: path_to_url(&effect.path),
            range: mapper.range(&effect.range),
            selection_range: mapper.range(&effect.range),
            data: None,
        });
    }

    None
}

/// Helper functions
/// NOTE: `position.character` must be a BYTE column (the same unit `ast::Range`
/// uses). LSP sends UTF-16 — convert at the public entry (see `prepare_call_hierarchy`).
fn position_in_range(position: &LspPosition, range: &Range) -> bool {
    let line = position.line;
    let character = position.character;

    (line > range.start_line || (line == range.start_line && character >= range.start_col))
        && (line < range.end_line || (line == range.end_line && character <= range.end_col))
}

fn range_contains(outer: &Range, inner: &Range) -> bool {
    (outer.start_line < inner.start_line
        || (outer.start_line == inner.start_line && outer.start_col <= inner.start_col))
        && (outer.end_line > inner.end_line
            || (outer.end_line == inner.end_line && outer.end_col >= inner.end_col))
}

fn range_overlaps(a: &Range, b: &Range) -> bool {
    !(a.end_line < b.start_line
        || (a.end_line == b.start_line && a.end_col < b.start_col)
        || b.end_line < a.start_line
        || (b.end_line == a.start_line && b.end_col < a.start_col))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::interner::InternedStr;
    use crate::data::layered_value::LayeredValue;
    use crate::data::scanner_data::ScannerData;
    use crate::scanner::event_scanner::Event;

    fn rng(sl: u32, sc: u32, el: u32, ec: u32) -> Range {
        Range {
            start_line: sl,
            start_col: sc,
            end_line: el,
            end_col: ec,
        }
    }

    fn insert_event(data: &ScannerData, path: &str, id: &str, range: Range) {
        data.events.insert(
            InternedStr::from(id),
            LayeredValue::new(Event {
                id: id.to_string(),
                event_type: "country_event".to_string(),
                path: InternedStr::from(path),
                range,
                triggered_events: vec![],
            }),
        );
        data.events_file_index
            .entry(InternedStr::from(path))
            .or_default()
            .push(InternedStr::from(id));
    }

    /// Regression for the UTF-16 inception bug: the client's position is UTF-16
    /// code units but scanner-data ranges are byte columns. A UTF-8 BOM at the
    /// file start shifts the first event's key range by 3 bytes while the
    /// client sees the BOM as 1 UTF-16 unit. A raw UTF-16 position of char 1
    /// (on the 'c' of `country_event`) is rejected by the byte range
    /// (1 < start_col 3); the byte-converted position (3) must match.
    #[tokio::test]
    async fn prepare_call_hierarchy_converts_utf16_to_byte() {
        let data = ScannerData::new();
        let tmp = std::env::temp_dir().join(format!("hom_callh_{}", std::process::id()));
        let file = tmp.join("test_events.txt");
        std::fs::create_dir_all(&tmp).unwrap();
        let content = "\u{feff}country_event = {\n    id = test_event\n}\n";
        std::fs::write(&file, content).unwrap();

        let uri = tower_lsp_server::ls_types::Uri::from_file_path(&file).unwrap();
        let uri_str = uri.as_str().to_string();
        // Derive the index key the SAME way `prepare_call_hierarchy` does
        // (`Uri::to_file_path`). On Windows the fork's Uri normalizes the
        // path (capitalized drive letter, forward slashes), so the raw
        // `PathBuf` string must not be used as the key — key and lookup must
        // come from the identical derivation or they never match.
        let path_str = uri.to_file_path().unwrap().to_string_lossy().to_string();

        // Scanner-side range: key starts at byte col 3 (after the 3-byte BOM),
        // block ends on line 2 at byte col 1.
        insert_event(&data, &path_str, "test_event", rng(0, 3, 2, 1));

        // Demonstrate the raw UTF-16 position fails the byte-range hit-test.
        let raw = tower_lsp_server::ls_types::Position {
            line: 0,
            character: 1,
        };
        assert!(
            !position_in_range(&raw, &rng(0, 3, 2, 1)),
            "raw UTF-16 char 1 must NOT match a range starting at byte col 3"
        );

        let item = prepare_call_hierarchy(&uri_str, raw, &data, Some(content)).await;
        assert!(
            item.is_some(),
            "converted position should initiate call hierarchy"
        );
        let item = item.unwrap();
        assert_eq!(item.name, "test_event");
        assert_eq!(item.kind, SymbolKind::EVENT);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// The same conversion must apply to scripted triggers/effects (same
    /// byte-range hit-test, different index/map).
    #[tokio::test]
    async fn prepare_call_hierarchy_scripted_trigger() {
        let data = ScannerData::new();
        let tmp = std::env::temp_dir().join(format!("hom_callh_st_{}", std::process::id()));
        let file = tmp.join("common/scripted_triggers/test_triggers.txt");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        let content = "\u{feff}my_trigger = {\n    hidden = yes\n}\n";
        std::fs::write(&file, content).unwrap();

        let uri = tower_lsp_server::ls_types::Uri::from_file_path(&file).unwrap();
        // Same derivation as `prepare_call_hierarchy` (see the other test):
        // the index key must match `Uri::to_file_path()` on every platform.
        let path_str = uri.to_file_path().unwrap().to_string_lossy().to_string();

        data.scripted_triggers.insert(
            InternedStr::from("my_trigger"),
            LayeredValue::new(crate::scanner::scripted_scanner::ScriptedEntity {
                name: "my_trigger".to_string(),
                path: InternedStr::from(path_str.clone()),
                range: rng(0, 3, 2, 1),
            }),
        );
        data.scripted_triggers_file_index.insert(
            InternedStr::from(path_str.clone()),
            vec![InternedStr::from("my_trigger")],
        );

        let position = tower_lsp_server::ls_types::Position {
            line: 0,
            character: 1,
        };
        let item = prepare_call_hierarchy(uri.as_str(), position, &data, Some(content)).await;
        assert!(
            item.is_some(),
            "converted position should find the scripted trigger"
        );
        assert_eq!(item.unwrap().name, "my_trigger");

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
