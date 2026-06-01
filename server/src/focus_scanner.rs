use crate::ast;
use crate::parser;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Focus {
    pub path: String,
    pub range: ast::Range,
}

pub fn scan_focuses<F>(roots: &[std::path::PathBuf], filter: &F) -> HashMap<String, Focus>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();

    for root in roots {
        crate::fs_util::walk_and_parse_files(
            &root.join("common/national_focus"),
            &["txt"],
            filter,
            |path, content| {
                let (script, _) = parser::parse_script(&content);
                find_focuses_in_entries(&script.entries, &path.to_string_lossy(), &mut map);
            },
        );
    }

    map
}

/// Find focus IDs inside `focus = { ... }` and `shared_focus = TAG_name` blocks.
pub(crate) fn find_focuses_in_entries(
    entries: &[ast::Entry],
    file_path: &str,
    map: &mut HashMap<String, Focus>,
) {
    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                let key = ass.key.as_str();

                // Case 1: focus = { id = TAG_focus_name ... }
                if key == "focus" {
                    if let ast::Value::Block(inner_entries) = &ass.value.value {
                        if let Some(id) = find_id_in_block(inner_entries) {
                            map.insert(
                                id,
                                Focus {
                                    path: file_path.to_string(),
                                    range: ass.key_range.clone(),
                                },
                            );
                        }
                    }
                }
                // Case 2: shared_focus = TAG_focus_name (value is a string)
                else if key == "shared_focus" {
                    if let ast::Value::String(focus_id) = &ass.value.value {
                        map.insert(
                            focus_id.clone(),
                            Focus {
                                path: file_path.to_string(),
                                range: ass.value.range.clone(),
                            },
                        );
                    }
                }

                // Recurse into sub-blocks (e.g. focus_tree = { ... })
                if let ast::Value::Block(inner_entries) = &ass.value.value {
                    find_focuses_in_entries(inner_entries, file_path, map);
                }
            }
            ast::Entry::Value(val) => match &val.value {
                ast::Value::Block(inner_entries) | ast::Value::TaggedBlock(_, inner_entries, _) => {
                    find_focuses_in_entries(inner_entries, file_path, map);
                }
                _ => {}
            },
            _ => {}
        }
    }
}

/// Find the `id` value inside a focus block's entries.
fn find_id_in_block(entries: &[ast::Entry]) -> Option<String> {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            if ass.key == "id" {
                if let ast::Value::String(s) = &ass.value.value {
                    return Some(s.clone());
                }
            }
        }
    }
    None
}
