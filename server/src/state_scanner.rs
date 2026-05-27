use crate::ast;
use crate::parser;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct State {
    pub id: u32,
    pub name: String, // e.g. "STATE_123"
    pub path: String,
    pub range: ast::Range,
}

pub fn scan_states<F>(roots: &[PathBuf], filter: &F) -> HashMap<u32, State>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut states = HashMap::new();

    for root in roots {
        let dir = root.join("history/states");
        if dir.exists() {
            let found = scan_directory(&dir, filter);
            states.extend(found);
        }
    }

    states
}

fn scan_directory<F>(dir_path: &Path, filter: &F) -> HashMap<u32, State>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut map = HashMap::new();
    let mut dirs_to_check = vec![dir_path.to_path_buf()];

    while let Some(current_dir) = dirs_to_check.pop() {
        if filter(&current_dir) {
            continue;
        }
        if let Ok(entries) = fs::read_dir(current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if !filter(&path) {
                        dirs_to_check.push(path);
                    }
                } else if path.extension().is_some_and(|ext| ext == "txt") {
                    if filter(&path) {
                        continue;
                    }
                    if let Ok(content) = fs::read_to_string(&path) {
                        {
                            let (script, _) = parser::parse_script(&content);
                            extract_state(&script.entries, &path, &mut map);
                        }
                    }
                }
            }
        }
    }
    map
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
                        path: path.to_string_lossy().to_string(),
                        range: ass.key_range.clone(),
                    },
                );
            }
        }
    }
}
