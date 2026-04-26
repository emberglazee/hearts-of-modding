use crate::parser;
use crate::ast;
use std::collections::HashMap;
use std::path::Path;
use std::fs;

#[derive(Debug, Clone)]
pub struct ScriptedEntity {
    pub name: String,
    pub path: String,
    pub range: ast::Range,
}

pub fn scan_directory(dir_path: &Path) -> HashMap<String, ScriptedEntity> {
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
                            for entry_ast in script.entries {
                                if let ast::Entry::Assignment(ass) = entry_ast {
                                    map.insert(ass.key.clone(), ScriptedEntity {
                                        name: ass.key.clone(),
                                        path: path.to_string_lossy().to_string(),
                                        range: ass.key_range,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    map
}