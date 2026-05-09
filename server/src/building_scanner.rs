use crate::parser;
use crate::ast;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug, Clone)]
pub struct Building {
    pub name: String,
    pub max_level: Option<i32>,
    pub path: String,
    pub range: ast::Range,
}

pub fn scan_buildings(roots: &[PathBuf]) -> HashMap<String, Building> {
    let mut buildings = HashMap::new();
    
    for root in roots {
        let dir = root.join("common/buildings");
        if dir.exists() {
            let found = scan_directory(&dir);
            buildings.extend(found);
        }
    }
    
    buildings
}

fn scan_directory(dir_path: &Path) -> HashMap<String, Building> {
    let mut map = HashMap::new();
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
                            extract_buildings(&script.entries, &path, &mut map);
                        }
                    }
                }
            }
        }
    }
    map
}

fn extract_buildings(entries: &[ast::Entry], path: &Path, map: &mut HashMap<String, Building>) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let building_name = ass.key.clone();
            let mut max_level = None;
            
            // Extract max_level from building definition
            if let ast::Value::Block(building_entries) = &ass.value.value {
                for building_entry in building_entries {
                    if let ast::Entry::Assignment(building_ass) = building_entry {
                        if building_ass.key.to_lowercase() == "max_level" {
                            if let ast::Value::Number(level) = &building_ass.value.value {
                                max_level = Some(*level as i32);
                            } else if let ast::Value::String(s) = &building_ass.value.value {
                                max_level = s.parse::<i32>().ok();
                            }
                        }
                    }
                }
            }
            
            map.insert(building_name.clone(), Building {
                name: building_name,
                max_level,
                path: path.to_string_lossy().to_string(),
                range: ass.key_range.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_buildings() {
        // Test would require mock AST data
    }
}
