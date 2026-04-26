use crate::parser;
use crate::ast;
use std::collections::HashMap;
use std::path::Path;
use std::fs;

#[derive(Debug, Clone)]
pub struct Ideology {
    pub name: String,
    pub sub_ideologies: Vec<String>,
    pub sub_ideology_ranges: HashMap<String, ast::Range>,
    pub path: String,
    pub range: ast::Range,
}

pub fn scan_ideologies(dir_path: &Path) -> HashMap<String, Ideology> {
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
                            find_ideologies_in_entries(&script.entries, &path.to_string_lossy(), &mut map);
                        }
                    }
                }
            }
        }
    }
    map
}

fn find_ideologies_in_entries(entries: &[ast::Entry], file_path: &str, map: &mut HashMap<String, Ideology>) {
    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                let key_lower = ass.key.to_lowercase();
                if key_lower == "ideologies" {
                    if let ast::Value::Block(ideology_entries) = &ass.value.value {
                        for ideology_entry in ideology_entries {
                            if let ast::Entry::Assignment(ideology_ass) = ideology_entry {
                                let mut sub_ideologies = Vec::new();
                                let mut sub_ideology_ranges = HashMap::new();
                                if let ast::Value::Block(ideology_details) = &ideology_ass.value.value {
                                    for detail in ideology_details {
                                        if let ast::Entry::Assignment(detail_ass) = detail {
                                            if detail_ass.key.to_lowercase() == "types" {
                                                if let ast::Value::Block(type_entries) = &detail_ass.value.value {
                                                    for type_entry in type_entries {
                                                        if let ast::Entry::Assignment(type_ass) = type_entry {
                                                            sub_ideologies.push(type_ass.key.clone());
                                                            sub_ideology_ranges.insert(type_ass.key.clone(), type_ass.key_range.clone());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                map.insert(ideology_ass.key.clone(), Ideology {
                                    name: ideology_ass.key.clone(),
                                    sub_ideologies,
                                    sub_ideology_ranges,
                                    path: file_path.to_string(),
                                    range: ideology_ass.key_range.clone(),
                                });
                            }
                        }
                    }
                } else {
                    // Recurse into other blocks
                    if let ast::Value::Block(inner_entries) = &ass.value.value {
                        find_ideologies_in_entries(inner_entries, file_path, map);
                    }
                }
            }
            ast::Entry::Value(val) => {
                match &val.value {
                    ast::Value::Block(inner_entries) => {
                        find_ideologies_in_entries(inner_entries, file_path, map);
                    }
                    ast::Value::TaggedBlock(_, inner_entries) => {
                        find_ideologies_in_entries(inner_entries, file_path, map);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}