use crate::parser;
use crate::ast;
use std::collections::HashMap;
use std::path::Path;
use std::fs;

#[derive(Debug, Clone)]
pub struct Ability {
    pub key: String,
    pub name_loc: Option<String>,
    pub desc_loc: Option<String>,
    pub cost: Option<f64>,
    pub duration: Option<i32>,
    pub sound_effect: Option<String>,
    pub type_name: Option<String>,
    pub path: String,
    pub range: ast::Range,
}

pub fn scan_abilities<F>(roots: &[std::path::PathBuf], filter: &F) -> HashMap<String, Ability> 
where F: Fn(&Path) -> bool {
    let mut map = HashMap::new();

    for root in roots {
        let dir_path = root.join("common").join("abilities");
        if !dir_path.exists() || filter(&dir_path) {
            continue;
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
                            { let (script, _) = parser::parse_script(&content);
                                find_abilities_in_entries(&script.entries, &path.to_string_lossy(), &mut map);
                            }
                        }
                    }
                }
            }
        }
    }
    map
}

fn find_abilities_in_entries(entries: &[ast::Entry], file_path: &str, map: &mut HashMap<String, Ability>) {
    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                let key_lower = ass.key.to_lowercase();
                if key_lower == "ability" {
                    if let ast::Value::Block(ability_entries) = &ass.value.value {
                        for ability_entry in ability_entries {
                            if let ast::Entry::Assignment(a_ass) = ability_entry {
                                let mut name_loc = None;
                                let mut desc_loc = None;
                                let mut cost = None;
                                let mut duration = None;
                                let mut sound_effect = None;
                                let mut type_name = None;

                                if let ast::Value::Block(props) = &a_ass.value.value {
                                    for prop in props {
                                        if let ast::Entry::Assignment(p_ass) = prop {
                                            match p_ass.key.to_lowercase().as_str() {
                                                "name" => {
                                                    if let ast::Value::String(s) = &p_ass.value.value {
                                                        name_loc = Some(s.clone());
                                                    }
                                                }
                                                "desc" => {
                                                    if let ast::Value::String(s) = &p_ass.value.value {
                                                        desc_loc = Some(s.clone());
                                                    }
                                                }
                                                "sound_effect" => {
                                                    if let ast::Value::String(s) = &p_ass.value.value {
                                                        sound_effect = Some(s.clone());
                                                    }
                                                }
                                                "type" => {
                                                    if let ast::Value::String(s) = &p_ass.value.value {
                                                        type_name = Some(s.clone());
                                                    }
                                                }
                                                "cost" => {
                                                    match &p_ass.value.value {
                                                        ast::Value::Number(n) => cost = Some(*n),
                                                        ast::Value::String(s) => if let Ok(n) = s.parse() { cost = Some(n); }
                                                        _ => {}
                                                    }
                                                }
                                                "duration" => {
                                                    match &p_ass.value.value {
                                                        ast::Value::Number(n) => duration = Some(*n as i32),
                                                        ast::Value::String(s) => if let Ok(n) = s.parse() { duration = Some(n); }
                                                        _ => {}
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }

                                map.insert(a_ass.key.clone(), Ability {
                                    key: a_ass.key.clone(),
                                    name_loc,
                                    desc_loc,
                                    cost,
                                    duration,
                                    sound_effect,
                                    type_name,
                                    path: file_path.to_string(),
                                    range: a_ass.key_range.clone(),
                                });
                            }
                        }
                    }
                } else {
                    // Recurse into other blocks
                    if let ast::Value::Block(inner_entries) = &ass.value.value {
                        find_abilities_in_entries(inner_entries, file_path, map);
                    }
                }
            }
            ast::Entry::Value(val) => {
                match &val.value {
                    ast::Value::Block(inner_entries) | ast::Value::TaggedBlock(_, inner_entries, _) => {
                        find_abilities_in_entries(inner_entries, file_path, map);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
