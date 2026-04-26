use crate::parser;
use crate::ast;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug, Clone)]
pub struct Idea {
    pub name: String,
    pub category: String,
    pub path: String,
    pub range: ast::Range,
}

pub fn scan_ideas(dir_path: &Path) -> HashMap<String, Idea> {
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
                            find_ideas_in_entries(&script.entries, &path.to_string_lossy(), &mut map);
                        }
                    }
                }
            }
        }
    }
    map
}

fn find_ideas_in_entries(entries: &[ast::Entry], file_path: &str, map: &mut HashMap<String, Idea>) {
    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                if ass.key.to_lowercase() == "ideas" {
                    parse_ideas_block(ass, file_path, map);
                } else {
                    // Recurse into other blocks
                    if let ast::Value::Block(inner_entries) = &ass.value.value {
                        find_ideas_in_entries(inner_entries, file_path, map);
                    }
                }
            }
            ast::Entry::Value(val) => {
                match &val.value {
                    ast::Value::Block(inner_entries) => {
                        find_ideas_in_entries(inner_entries, file_path, map);
                    }
                    ast::Value::TaggedBlock(_, inner_entries) => {
                        find_ideas_in_entries(inner_entries, file_path, map);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn parse_ideas_block(ass: &ast::Assignment, file_path: &str, map: &mut HashMap<String, Idea>) {
    if let ast::Value::Block(categories) = &ass.value.value {
        for category_entry in categories {
            if let ast::Entry::Assignment(cat_ass) = category_entry {
                let category_name = cat_ass.key.clone();
                if let ast::Value::Block(ideas) = &cat_ass.value.value {
                    for idea_entry in ideas {
                        if let ast::Entry::Assignment(idea_ass) = idea_entry {
                            map.insert(idea_ass.key.clone(), Idea {
                                name: idea_ass.key.clone(),
                                category: category_name.clone(),
                                path: file_path.to_string(),
                                range: idea_ass.key_range.clone(),
                            });
                        }
                    }
                }
            }
        }
    }
}
