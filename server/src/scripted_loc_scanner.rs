use crate::parser;
use crate::ast::{self, Entry, Value};
use std::collections::HashMap;
use std::path::Path;
use std::fs;

#[derive(Debug, Clone)]
pub struct ScriptedLoc {
    pub name: String,
    pub path: String,
    pub range: ast::Range,
}

pub fn scan_directory<F>(dir_path: &Path, filter: &F) -> HashMap<String, ScriptedLoc> 
where F: Fn(&Path) -> bool {
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
                } else if path.extension().map_or(false, |ext| ext == "txt") {
                    if filter(&path) {
                        continue;
                    }
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(script) = parser::parse_script(&content) {
                            find_scripted_locs_in_entries(&script.entries, &path.to_string_lossy(), &mut map);
                        }
                    }
                }
            }
        }
    }
    map
}

fn find_scripted_locs_in_entries(entries: &[Entry], file_path: &str, map: &mut HashMap<String, ScriptedLoc>) {
    for entry in entries {
        if let Entry::Assignment(ass) = entry {
            if ass.key == "defined_text" {
                if let Value::Block(children) = &ass.value.value {
                    for child in children {
                        if let Entry::Assignment(child_ass) = child {
                            if child_ass.key == "name" {
                                if let Value::String(name) = &child_ass.value.value {
                                    map.insert(name.clone(), ScriptedLoc {
                                        name: name.clone(),
                                        path: file_path.to_string(),
                                        range: child_ass.value.range.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_scripted_locs() {
        let content = r#"
defined_text = {
	name = DBUG_show_lar_decisions
	text = {
		trigger = {
			NOT = { has_dlc = "La Resistance" }
		}
		localization_key = DBUG_show_lar_di_decisions
	}
	text = {
		trigger = { has_dlc = "La Resistance" }
		localization_key = DBUG_show_lar_en_decisions
	}
}
        "#;
        let script = crate::parser::parse_script(content).unwrap();
        let mut map = HashMap::new();
        find_scripted_locs_in_entries(&script.entries, "test", &mut map);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("DBUG_show_lar_decisions"));
    }
}