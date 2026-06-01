use crate::interner::InternedStr;
use crate::ast;
use crate::parser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Event {
    pub id: String,
    pub event_type: String, // country_event, state_event, etc.
    pub path: InternedStr,
    pub range: ast::Range,
    pub triggered_events: Vec<String>, // IDs of events triggered BY this event
}

impl serde::Serialize for Event {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Event", 5)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("event_type", &self.event_type)?;
        state.serialize_field("path", &*self.path)?;
        state.serialize_field("range", &self.range)?;
        state.serialize_field("triggered_events", &self.triggered_events)?;
        state.end()
    }
}

pub fn scan_events<F>(roots: &[PathBuf], filter: &F) -> HashMap<String, Event>
where
    F: Fn(&Path) -> bool,
{
    let mut events = HashMap::new();

    for root in roots {
        crate::fs_util::walk_and_parse_files(
            &root.join("events"),
            &["txt"],
            filter,
            |path, content| {
                let (script, _) = parser::parse_script(&content);
                find_event_definitions(&script.entries, &path.to_string_lossy(), &mut events);
            },
        );
    }

    // Second pass: Find where events are triggered
    for root in roots {
        scan_for_triggers(root, &mut events, filter);
    }

    events
}

pub(crate) fn find_event_definitions(
    entries: &[ast::Entry],
    path: &str,
    events: &mut HashMap<String, Event>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let key = ass.key.as_str();
            if (key == "country_event"
                || key == "state_event"
                || key == "news_event"
                || key == "unit_leader_event")
                && let ast::Value::Block(inner) = &ass.value.value
            {
                let mut id = None;
                for inner_entry in inner {
                    if let ast::Entry::Assignment(inner_ass) = inner_entry
                        && inner_ass.key == "id"
                        && let ast::Value::String(s) = &inner_ass.value.value
                    {
                        id = Some(s.clone());
                        break;
                    }
                }

                if let Some(event_id) = id {
                    events.insert(
                        event_id.clone(),
                        Event {
                            id: event_id,
                            event_type: key.to_string(),
                            path: std::sync::Arc::from(path),
                            range: ass.key_range.clone(),
                            triggered_events: Vec::new(),
                        },
                    );
                }
            }
        }
    }
}

fn scan_for_triggers<F>(root: &Path, events: &mut HashMap<String, Event>, filter: &F)
where
    F: Fn(&Path) -> bool,
{
    for subdir in ["common", "events"] {
        crate::fs_util::walk_and_parse_files(
            &root.join(subdir),
            &["txt"],
            filter,
            |_path, content| {
                let (script, _) = parser::parse_script(&content);
                find_triggers_in_script(&script.entries, events);
            },
        );
    }
}

fn find_triggers_in_script(entries: &[ast::Entry], events: &mut HashMap<String, Event>) {
    // This is tricky because we need to know WHICH event we are currently inside
    // if we find a trigger.
    find_triggers_recursive(entries, None, events);
}

fn find_triggers_recursive(
    entries: &[ast::Entry],
    current_event_id: Option<&str>,
    events: &mut HashMap<String, Event>,
) {
    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                let key = ass.key.as_str();

                // Are we entering a new event definition?
                let next_event_id = if key == "country_event"
                    || key == "state_event"
                    || key == "news_event"
                    || key == "unit_leader_event"
                {
                    // Extract ID
                    if let ast::Value::Block(inner) = &ass.value.value {
                        inner.iter().find_map(|e| {
                            if let ast::Entry::Assignment(ia) = e
                                && ia.key == "id"
                                && let ast::Value::String(s) = &ia.value.value
                            {
                                return Some(s.as_str());
                            }
                            None
                        })
                    } else {
                        None
                    }
                } else {
                    current_event_id
                };

                // Is this an event call?
                if key == "country_event"
                    || key == "state_event"
                    || key == "news_event"
                    || key == "unit_leader_event"
                {
                    // Check if it's a call: country_event = { id = ... } OR country_event = id
                    let called_id = match &ass.value.value {
                        ast::Value::String(s) => Some(s.as_str()),
                        ast::Value::Block(inner) => inner.iter().find_map(|e| {
                            if let ast::Entry::Assignment(ia) = e
                                && ia.key == "id"
                                && let ast::Value::String(s) = &ia.value.value
                            {
                                return Some(s.as_str());
                            }
                            None
                        }),
                        _ => None,
                    };

                    if let (Some(source), Some(target)) = (current_event_id, called_id)
                        && source != target
                        && let Some(event) = events.get_mut(source)
                    {
                        if !event.triggered_events.contains(&target.to_string()) {
                            event.triggered_events.push(target.to_string());
                        }
                    }
                }

                // Recurse
                match &ass.value.value {
                    ast::Value::Block(inner) => {
                        find_triggers_recursive(inner, next_event_id, events)
                    }
                    ast::Value::TaggedBlock(_, inner, _) => {
                        find_triggers_recursive(inner, next_event_id, events)
                    }
                    _ => {}
                }
            }
            ast::Entry::Value(val) => match &val.value {
                ast::Value::Block(inner) => {
                    find_triggers_recursive(inner, current_event_id, events)
                }
                ast::Value::TaggedBlock(_, inner, _) => {
                    find_triggers_recursive(inner, current_event_id, events)
                }
                _ => {}
            },
            _ => {}
        }
    }
}
