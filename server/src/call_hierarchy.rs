use crate::ast::{Entry, Range, Value};
use std::collections::HashMap;
use tower_lsp_server::ls_types::{
    CallHierarchyIncomingCall, CallHierarchyItem, CallHierarchyOutgoingCall,
    Position as LspPosition, Range as LspRange, SymbolKind, Uri,
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
pub async fn prepare_call_hierarchy(
    uri: &str,
    position: LspPosition,
    data: &crate::ScannerData,
) -> Option<CallHierarchyItem> {
    let path = uri.trim_start_matches("file://");

    // Check if position is on an event
    let events_lock = data.events();
    for (id, event) in events_lock.iter() {
        if event.path == path && position_in_range(&position, &event.range) {
            return Some(CallHierarchyItem {
                name: id.clone(),
                kind: SymbolKind::EVENT,
                tags: None,
                detail: Some(format!("{:?}", event.event_type)),
                uri: path_to_url(&event.path),
                range: range_to_lsp(&event.range),
                selection_range: range_to_lsp(&event.range),
                data: None,
            });
        }
    }
    drop(events_lock);

    // Check if position is on a scripted trigger
    let triggers_lock = data.scripted_triggers();
    for (name, trigger) in triggers_lock.iter() {
        if trigger.path == path && position_in_range(&position, &trigger.range) {
            return Some(CallHierarchyItem {
                name: name.clone(),
                kind: SymbolKind::FUNCTION,
                tags: None,
                detail: Some("Scripted Trigger".to_string()),
                uri: path_to_url(&trigger.path),
                range: range_to_lsp(&trigger.range),
                selection_range: range_to_lsp(&trigger.range),
                data: None,
            });
        }
    }
    drop(triggers_lock);

    // Check if position is on a scripted effect
    let effects_lock = data.scripted_effects();
    for (name, effect) in effects_lock.iter() {
        if effect.path == path && position_in_range(&position, &effect.range) {
            return Some(CallHierarchyItem {
                name: name.clone(),
                kind: SymbolKind::FUNCTION,
                tags: None,
                detail: Some("Scripted Effect".to_string()),
                uri: path_to_url(&effect.path),
                range: range_to_lsp(&effect.range),
                selection_range: range_to_lsp(&effect.range),
                data: None,
            });
        }
    }

    None
}

/// Get incoming calls (who calls this symbol)
pub async fn get_incoming_calls(
    item: &CallHierarchyItem,
    data: &crate::ScannerData,
    documents: &dashmap::DashMap<String, String>,
) -> Vec<CallHierarchyIncomingCall> {
    let mut incoming = Vec::new();
    let target_name = &item.name;

    // Search for references in all documents
    for entry in documents.iter() {
        let uri = entry.key();
        let content = entry.value();

        // Parse the document
        {
            let (script, _) = crate::parser::parse_script(content);
            let references = find_references_in_entries(&script.entries, target_name);

            if !references.is_empty() {
                // Try to find the containing symbol
                if let Some(container) = find_container_symbol(uri, &references[0], data).await {
                    incoming.push(CallHierarchyIncomingCall {
                        from: container,
                        from_ranges: references.iter().map(range_to_lsp).collect(),
                    });
                }
            }
        }
    }

    incoming
}

/// Get outgoing calls (what this symbol calls)
pub async fn get_outgoing_calls(
    item: &CallHierarchyItem,
    data: &crate::ScannerData,
    documents: &dashmap::DashMap<String, String>,
) -> Vec<CallHierarchyOutgoingCall> {
    let mut outgoing = Vec::new();

    // Get the document content
    if let Some(entry) = documents.get(item.uri.as_str()) {
        let content = entry.value();

        // Parse the document
        {
            let (script, _) = crate::parser::parse_script(content);
            // Find all calls within this symbol's range
            let calls = find_calls_in_range(&script.entries, &lsp_to_range(&item.range));

            for (call_name, call_ranges) in calls {
                // Try to find the target symbol
                if let Some(target) = find_symbol_by_name(&call_name, data).await {
                    outgoing.push(CallHierarchyOutgoingCall {
                        to: target,
                        from_ranges: call_ranges.iter().map(range_to_lsp).collect(),
                    });
                }
            }
        }
    }

    outgoing
}

/// Find references to a symbol in AST entries
fn find_references_in_entries(entries: &[Entry], target: &str) -> Vec<Range> {
    let mut references = Vec::new();

    for entry in entries {
        find_references_recursive(entry, target, &mut references);
    }

    references
}

fn find_references_recursive(entry: &Entry, target: &str, references: &mut Vec<Range>) {
    if let Entry::Assignment(ass) = entry {
        // Check for event triggers: country_event = { id = target }
        if ass.key == "country_event" || ass.key == "state_event" || ass.key == "news_event" {
            if let Value::Block(children) = &ass.value.value {
                for child in children {
                    if let Entry::Assignment(child_ass) = child {
                        if child_ass.key == "id" {
                            if let Value::String(id) = &child_ass.value.value {
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
        if let Value::String(s) = &ass.value.value {
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
                find_references_recursive(child, target, references);
            }
        }
    }
}

/// Find all calls within a specific range
fn find_calls_in_range(entries: &[Entry], range: &Range) -> HashMap<String, Vec<Range>> {
    let mut calls = HashMap::new();

    for entry in entries {
        find_calls_recursive(entry, range, &mut calls);
    }

    calls
}

fn find_calls_recursive(
    entry: &Entry,
    target_range: &Range,
    calls: &mut HashMap<String, Vec<Range>>,
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
        if ass.key == "country_event" || ass.key == "state_event" || ass.key == "news_event" {
            if let Value::Block(children) = &ass.value.value {
                for child in children {
                    if let Entry::Assignment(child_ass) = child {
                        if child_ass.key == "id" {
                            if let Value::String(id) = &child_ass.value.value {
                                calls.entry(id.clone()).or_default().push(range.clone());
                            }
                        }
                    }
                }
            }
        }

        // Check for scripted trigger/effect calls
        if let Value::String(s) = &ass.value.value {
            calls.entry(s.clone()).or_default().push(range.clone());
        }

        // Recurse into blocks
        if let Value::Block(children) = &ass.value.value {
            for child in children {
                find_calls_recursive(child, target_range, calls);
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
    let path = uri.trim_start_matches("file://");

    // Check events
    let events_lock = data.events();
    for (id, event) in events_lock.iter() {
        if event.path == path && range_contains(&event.range, range) {
            return Some(CallHierarchyItem {
                name: id.clone(),
                kind: SymbolKind::EVENT,
                tags: None,
                detail: Some(format!("{:?}", event.event_type)),
                uri: path_to_url(&event.path),
                range: range_to_lsp(&event.range),
                selection_range: range_to_lsp(&event.range),
                data: None,
            });
        }
    }
    drop(events_lock);

    // Check scripted triggers
    let triggers_lock = data.scripted_triggers();
    for (name, trigger) in triggers_lock.iter() {
        if trigger.path == path && range_contains(&trigger.range, range) {
            return Some(CallHierarchyItem {
                name: name.clone(),
                kind: SymbolKind::FUNCTION,
                tags: None,
                detail: Some("Scripted Trigger".to_string()),
                uri: path_to_url(&trigger.path),
                range: range_to_lsp(&trigger.range),
                selection_range: range_to_lsp(&trigger.range),
                data: None,
            });
        }
    }
    drop(triggers_lock);

    // Check scripted effects
    let effects_lock = data.scripted_effects();
    for (name, effect) in effects_lock.iter() {
        if effect.path == path && range_contains(&effect.range, range) {
            return Some(CallHierarchyItem {
                name: name.clone(),
                kind: SymbolKind::FUNCTION,
                tags: None,
                detail: Some("Scripted Effect".to_string()),
                uri: path_to_url(&effect.path),
                range: range_to_lsp(&effect.range),
                selection_range: range_to_lsp(&effect.range),
                data: None,
            });
        }
    }

    None
}

/// Find a symbol by name
async fn find_symbol_by_name(name: &str, data: &crate::ScannerData) -> Option<CallHierarchyItem> {
    // Check events
    let events_lock = data.events();
    if let Some(event) = events_lock.get(name) {
        return Some(CallHierarchyItem {
            name: name.to_string(),
            kind: SymbolKind::EVENT,
            tags: None,
            detail: Some(format!("{:?}", event.event_type)),
            uri: path_to_url(&event.path),
            range: range_to_lsp(&event.range),
            selection_range: range_to_lsp(&event.range),
            data: None,
        });
    }
    drop(events_lock);

    // Check scripted triggers
    let triggers_lock = data.scripted_triggers();
    if let Some(trigger) = triggers_lock.get(name) {
        return Some(CallHierarchyItem {
            name: name.to_string(),
            kind: SymbolKind::FUNCTION,
            tags: None,
            detail: Some("Scripted Trigger".to_string()),
            uri: path_to_url(&trigger.path),
            range: range_to_lsp(&trigger.range),
            selection_range: range_to_lsp(&trigger.range),
            data: None,
        });
    }
    drop(triggers_lock);

    // Check scripted effects
    let effects_lock = data.scripted_effects();
    if let Some(effect) = effects_lock.get(name) {
        return Some(CallHierarchyItem {
            name: name.to_string(),
            kind: SymbolKind::FUNCTION,
            tags: None,
            detail: Some("Scripted Effect".to_string()),
            uri: path_to_url(&effect.path),
            range: range_to_lsp(&effect.range),
            selection_range: range_to_lsp(&effect.range),
            data: None,
        });
    }

    None
}

/// Helper functions
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

fn range_to_lsp(range: &Range) -> LspRange {
    LspRange {
        start: LspPosition {
            line: range.start_line,
            character: range.start_col,
        },
        end: LspPosition {
            line: range.end_line,
            character: range.end_col,
        },
    }
}

fn lsp_to_range(range: &LspRange) -> Range {
    Range {
        start_line: range.start.line,
        start_col: range.start.character,
        end_line: range.end.line,
        end_col: range.end.character,
    }
}
