use crate::parser;
use crate::ast;
use std::collections::HashMap;
use std::path::Path;
use std::fs;

#[derive(Debug, Clone)]
pub struct Trait {
    pub name: String,
    pub trait_type: String, // e.g., "Leader Trait", "Country Leader Trait"
    pub path: String,
    pub range: ast::Range,
}

pub fn scan_traits(dir_path: &Path, trait_type: &str) -> HashMap<String, Trait> {
    let mut map = HashMap::new();
    if !dir_path.exists() {
        return map;
    }

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
                            find_traits_in_entries(&script.entries, &path.to_string_lossy(), trait_type, &mut map);
                        }
                    }
                }
            }
        }
    }
    map
}

fn find_traits_in_entries(entries: &[ast::Entry], file_path: &str, trait_type: &str, map: &mut HashMap<String, Trait>) {
    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                let key_lower = ass.key.to_lowercase();
                if key_lower == "leader_traits" || key_lower == "country_leader_traits" || key_lower == "traits" {
                    if let ast::Value::Block(trait_entries) = &ass.value.value {
                        for trait_entry in trait_entries {
                            if let ast::Entry::Assignment(t_ass) = trait_entry {
                                map.insert(t_ass.key.clone(), Trait {
                                    name: t_ass.key.clone(),
                                    trait_type: trait_type.to_string(),
                                    path: file_path.to_string(),
                                    range: t_ass.key_range.clone(),
                                });
                            }
                        }
                    }
                } else {
                    // Recurse into other blocks
                    if let ast::Value::Block(inner_entries) = &ass.value.value {
                        find_traits_in_entries(inner_entries, file_path, trait_type, map);
                    }
                }
            }
            ast::Entry::Value(val) => {
                match &val.value {
                    ast::Value::Block(inner_entries) => {
                        find_traits_in_entries(inner_entries, file_path, trait_type, map);
                    }
                    ast::Value::TaggedBlock(_, inner_entries) => {
                        find_traits_in_entries(inner_entries, file_path, trait_type, map);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}