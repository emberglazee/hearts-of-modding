use crate::parser;
use crate::ast;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug, Clone)]
pub struct Achievement {
    pub name: String,
    pub is_ribbon: bool,
    pub path: String,
    pub range: ast::Range,
}

pub fn scan_achievements<F>(roots: &[PathBuf], filter: &F) -> HashMap<String, Achievement> 
where F: Fn(&std::path::Path) -> bool {
    let mut map = HashMap::new();
    for root in roots {
        let achievements_dir = root.join("common").join("achievements");
        if achievements_dir.exists() {
            scan_dir(&achievements_dir, &mut map, filter);
        }
    }
    map
}

fn scan_dir<F>(dir_path: &Path, map: &mut HashMap<String, Achievement>, filter: &F) 
where F: Fn(&std::path::Path) -> bool {
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
                } else if path.extension().map_or(false, |ext| ext == "txt") {
                    if filter(&path) {
                        continue;
                    }
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(script) = parser::parse_script(&content) {
                            find_achievements_in_entries(&script.entries, &path.to_string_lossy(), map);
                        }
                    }
                }
            }
        }
    }
}

fn find_achievements_in_entries(entries: &[ast::Entry], file_path: &str, map: &mut HashMap<String, Achievement>) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let key_lower = ass.key.to_lowercase();
            
            // Skip unique_id assignment
            if key_lower == "unique_id" {
                continue;
            }

            if let ast::Value::Block(inner_entries) = &ass.value.value {
                // Determine if it's an achievement or ribbon
                // In HOI4, achievements are top-level blocks that aren't unique_id.
                // A ribbon usually contains a 'ribbon = { ... }' block inside.
                let mut is_ribbon = false;
                let mut is_achievement = false;

                for inner in inner_entries {
                    if let ast::Entry::Assignment(inner_ass) = inner {
                        let inner_key = inner_ass.key.to_lowercase();
                        if inner_key == "ribbon" {
                            is_ribbon = true;
                        }
                        if inner_key == "possible" || inner_key == "happened" {
                            is_achievement = true;
                        }
                    }
                }

                if is_achievement || is_ribbon {
                    map.insert(ass.key.clone(), Achievement {
                        name: ass.key.clone(),
                        is_ribbon,
                        path: file_path.to_string(),
                        range: ass.key_range.clone(),
                    });
                }
            }
        }
    }
}
