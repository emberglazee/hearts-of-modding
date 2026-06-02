use crate::ast;
use crate::interner::InternedStr;
use crate::parser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct State {
    pub id: u32,
    pub name: String, // e.g. "STATE_123"
    pub path: InternedStr,
    pub range: ast::Range,
}

pub fn scan_states<F>(roots: &[PathBuf], filter: &F) -> HashMap<u32, State>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut states = HashMap::new();

    for root in roots {
        crate::fs_util::walk_and_parse_files(
            &root.join("history/states"),
            &["txt"],
            filter,
            |path, content| {
                let (script, _) = parser::parse_script(&content);
                extract_state(&script.entries, path, &mut states);
            },
        );
    }

    states
}

fn extract_state(entries: &[ast::Entry], path: &Path, map: &mut HashMap<u32, State>) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry
            && ass.key.eq_ignore_ascii_case("state")
        {
            let mut state_id = None;
            let mut state_name = String::new();

            if let ast::Value::Block(state_entries) = &ass.value.value {
                for state_entry in state_entries {
                    if let ast::Entry::Assignment(state_ass) = state_entry {
                        if state_ass.key.eq_ignore_ascii_case("id")
                            && let ast::Value::Number(id) = &state_ass.value.value
                        {
                            state_id = Some(*id as u32);
                        } else if state_ass.key.eq_ignore_ascii_case("name")
                            && let ast::Value::String(name) = &state_ass.value.value
                        {
                            state_name = name.clone();
                        }
                    }
                }
            }

            if let Some(id) = state_id {
                map.insert(
                    id,
                    State {
                        id,
                        name: state_name,
                        path: std::sync::Arc::from(path.to_string_lossy().as_ref()),
                        range: ass.key_range.clone(),
                    },
                );
            }
        }
    }
}
