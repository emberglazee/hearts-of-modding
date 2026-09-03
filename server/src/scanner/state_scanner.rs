#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct State {
    pub id: u32,
    pub name: String, // e.g. "STATE_123"
    /// Member province ids from `state = { provinces = { ... } }`.
    /// Powers the Tier-0 cross-file map checks (two-states, sea-in-state…).
    pub provinces: Vec<u32>,
    pub path: InternedStr,
    pub range: ast::Range,
}

pub fn scan_states<F>(roots: &[PathBuf], filter: &F) -> HashMap<u32, State>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut states = HashMap::new();

    for root in roots {
        crate::utils::fs_util::walk_and_parse_files(
            &root.join("history/states"),
            &["txt"],
            filter,
            |path, content| {
                let (script, _) = parser::parse_script(&content);
                extract_state(&script.entries, &script.source, path, &mut states);
            },
        );
    }

    states
}

pub fn scan_state_files<F>(files: &[PathBuf], filter: &F) -> HashMap<u32, State>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut states = HashMap::new();
    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        extract_state(&script.entries, &script.source, path, &mut states);
    });
    states
}

/// Extracts `state = { id name provinces }` docs. `pub(crate)` so the
/// incremental scanner reuses it for per-save updates (mirrors
/// `strategic_region_scanner::extract_strategic_region`).
pub(crate) fn extract_state(
    entries: &[ast::Entry],
    source: &str,
    path: &Path,
    map: &mut HashMap<u32, State>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry
            && ass.key_text(source).eq_ignore_ascii_case("state")
        {
            let mut state_id = None;
            let mut state_name = String::new();
            let mut state_provinces = Vec::new();

            if let ast::Value::Block(state_entries) = &ass.value.value {
                for state_entry in state_entries {
                    if let ast::Entry::Assignment(state_ass) = state_entry {
                        if state_ass.key_text(source).eq_ignore_ascii_case("id")
                            && let ast::Value::Number(id) = &state_ass.value.value
                        {
                            state_id = Some(*id as u32);
                        } else if state_ass.key_text(source).eq_ignore_ascii_case("name")
                            && let Some(name) = state_ass.value.value.as_str(source)
                        {
                            state_name = name.to_string();
                        } else if state_ass.key_text(source).eq_ignore_ascii_case("provinces") {
                            collect_province_ids(
                                &state_ass.value.value,
                                source,
                                &mut state_provinces,
                            );
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
                        provinces: state_provinces,
                        path: std::sync::Arc::from(path.to_string_lossy().as_ref()),
                        range: ass.key_range.clone(),
                    },
                );
            }
        }
    }
}

/// Collect bare numeric ids from a `provinces = { ... }` value into `out`.
/// Accepts plain blocks and tagged blocks; numeric strings count (the parser
/// keeps large ids as strings in some positions).
pub(crate) fn collect_province_ids(value: &ast::Value, source: &str, out: &mut Vec<u32>) {
    let entries = match value {
        ast::Value::Block(entries) => entries,
        ast::Value::TaggedBlock(_, entries, _) => entries,
        _ => return,
    };
    for entry in entries {
        if let ast::Entry::Value(val) = entry {
            match &val.value {
                ast::Value::Number(n) if *n >= 0.0 => out.push(*n as u32),
                ast::Value::String(s) => {
                    if let Ok(n) = s.resolve(source).parse::<u32>() {
                        out.push(n);
                    }
                }
                _ => {}
            }
        }
    }
}
