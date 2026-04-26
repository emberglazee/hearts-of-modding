use crate::parser;
use crate::ast;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Event {
    pub id: String,
    pub event_type: String, // country_event, state_event, etc.
    pub path: String,
    pub range: ast::Range,
    pub triggered_events: Vec<String>, // IDs of events triggered BY this event
}

pub fn scan_events(roots: &[PathBuf]) -> HashMap<String, Event> {
    let mut events = HashMap::new();
    
    for root in roots {
        let events_dir = root.join("events");
        if events_dir.exists() {
            scan_directory(&events_dir, &mut events);
        }
        
        // Events can also be in common/on_actions or other places, 
        // but for graphing we primarily care about the definitions in 'events/'
        // and where they are called from.
        // For now, let's focus on 'events/' folder for definitions.
    }

    // Second pass: Find where events are triggered
    // We'll search in roots again for common patterns
    for root in roots {
        scan_for_triggers(root, &mut events);
    }

    events
}

fn scan_directory(dir_path: &Path, events: &mut HashMap<String, Event>) {
    let mut dirs_to_check = vec![dir_path.to_path_buf()];
    
    while let Some(current_dir) = dirs_to_check.pop() {
        if let Ok(entries) = fs::read_dir(current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    dirs_to_check.push(path);
                } else if path.extension().map_or(false, |ext| ext == "txt") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(script) = parser::parse_script(&content) {
                            find_event_definitions(&script.entries, &path.to_string_lossy(), events);
                        }
                    }
                }
            }
        }
    }
}

fn find_event_definitions(entries: &[ast::Entry], path: &str, events: &mut HashMap<String, Event>) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let key = ass.key.as_str();
            if key == "country_event" || key == "state_event" || key == "news_event" || key == "unit_leader_event" {
                if let ast::Value::Block(inner) = &ass.value.value {
                    let mut id = None;
                    for inner_entry in inner {
                        if let ast::Entry::Assignment(inner_ass) = inner_entry {
                            if inner_ass.key == "id" {
                                if let ast::Value::String(s) = &inner_ass.value.value {
                                    id = Some(s.clone());
                                    break;
                                }
                            }
                        }
                    }
                    
                    if let Some(event_id) = id {
                        events.insert(event_id.clone(), Event {
                            id: event_id,
                            event_type: key.to_string(),
                            path: path.to_string(),
                            range: ass.key_range.clone(),
                            triggered_events: Vec::new(),
                        });
                    }
                }
            }
        }
    }
}

fn scan_for_triggers(root: &Path, events: &mut HashMap<String, Event>) {
    // We need to look through common/, events/, and potentially others
    let subdirs = ["common", "events"];
    for subdir in subdirs {
        let dir_path = root.join(subdir);
        if !dir_path.exists() { continue; }
        
        let mut dirs_to_check = vec![dir_path];
        while let Some(current_dir) = dirs_to_check.pop() {
            if let Ok(entries) = fs::read_dir(current_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        dirs_to_check.push(path);
                    } else if path.extension().map_or(false, |ext| ext == "txt") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(script) = parser::parse_script(&content) {
                                find_triggers_in_script(&script.entries, events);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn find_triggers_in_script(entries: &[ast::Entry], events: &mut HashMap<String, Event>) {
    // This is tricky because we need to know WHICH event we are currently inside
    // if we find a trigger.
    find_triggers_recursive(entries, None, events);
}

fn find_triggers_recursive(entries: &[ast::Entry], current_event_id: Option<&str>, events: &mut HashMap<String, Event>) {
    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                let key = ass.key.as_str();
                
                // Are we entering a new event definition?
                let next_event_id = if key == "country_event" || key == "state_event" || key == "news_event" || key == "unit_leader_event" {
                    // Extract ID
                    if let ast::Value::Block(inner) = &ass.value.value {
                        inner.iter().find_map(|e| {
                            if let ast::Entry::Assignment(ia) = e {
                                if ia.key == "id" {
                                    if let ast::Value::String(s) = &ia.value.value {
                                        return Some(s.as_str());
                                    }
                                }
                            }
                            None
                        })
                    } else { None }
                } else {
                    current_event_id
                };

                // Is this an event call?
                if key == "country_event" || key == "state_event" || key == "news_event" || key == "unit_leader_event" {
                    // Check if it's a call: country_event = { id = ... } OR country_event = id
                    let called_id = match &ass.value.value {
                        ast::Value::String(s) => Some(s.as_str()),
                        ast::Value::Block(inner) => {
                            inner.iter().find_map(|e| {
                                if let ast::Entry::Assignment(ia) = e {
                                    if ia.key == "id" {
                                        if let ast::Value::String(s) = &ia.value.value {
                                            return Some(s.as_str());
                                        }
                                    }
                                }
                                None
                            })
                        }
                        _ => None,
                    };

                    if let (Some(source), Some(target)) = (current_event_id, called_id) {
                        if source != target {
                            if let Some(event) = events.get_mut(source) {
                                if !event.triggered_events.contains(&target.to_string()) {
                                    event.triggered_events.push(target.to_string());
                                }
                            }
                        }
                    }
                }

                // Recurse
                match &ass.value.value {
                    ast::Value::Block(inner) => find_triggers_recursive(inner, next_event_id, events),
                    ast::Value::TaggedBlock(_, inner) => find_triggers_recursive(inner, next_event_id, events),
                    _ => {}
                }
            }
            ast::Entry::Value(val) => {
                match &val.value {
                    ast::Value::Block(inner) => find_triggers_recursive(inner, current_event_id, events),
                    ast::Value::TaggedBlock(_, inner) => find_triggers_recursive(inner, current_event_id, events),
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
