use crate::ast;
use crate::parser;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ScriptedEntity {
    pub name: String,
    pub path: String,
    pub range: ast::Range,
}

pub fn scan_directory<F>(dir_path: &Path, filter: &F) -> HashMap<String, ScriptedEntity>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();
    if !dir_path.exists() || filter(dir_path) {
        return map;
    }

    let mut dirs_to_check = vec![dir_path.to_path_buf()];
    while let Some(current_dir) = dirs_to_check.pop() {
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
                            for entry_ast in script.entries {
                                if let ast::Entry::Assignment(ass) = entry_ast {
                                    map.insert(
                                        ass.key.clone(),
                                        ScriptedEntity {
                                            name: ass.key.clone(),
                                            path: path.to_string_lossy().to_string(),
                                            range: ass.key_range,
                                        },
                                    );
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
