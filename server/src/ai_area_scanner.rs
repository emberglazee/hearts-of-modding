use crate::ast;
use crate::parser;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AiArea {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub continents: Vec<String>,
    #[allow(dead_code)]
    pub strategic_regions: Vec<u32>,
    #[allow(dead_code)]
    pub path: String,
    #[allow(dead_code)]
    pub range: ast::Range,
}

pub fn scan_ai_areas<F>(roots: &[PathBuf], filter: &F) -> HashMap<String, AiArea>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut areas = HashMap::new();

    for root in roots {
        let dir = root.join("common/ai_areas");
        if dir.exists() {
            let found = scan_directory(&dir, filter);
            areas.extend(found);
        }
    }

    areas
}

fn scan_directory<F>(dir_path: &Path, filter: &F) -> HashMap<String, AiArea>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut map = HashMap::new();
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
                } else if path.extension().is_some_and(|ext| ext == "txt") {
                    if filter(&path) {
                        continue;
                    }
                    if let Ok(content) = fs::read_to_string(&path) {
                        let (script, _) = parser::parse_script(&content);
                        extract_areas(&script.entries, &path, &mut map);
                    }
                }
            }
        }
    }

    map
}

fn extract_areas(entries: &[ast::Entry], path: &Path, map: &mut HashMap<String, AiArea>) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            if let ast::Value::Block(inner_entries) = &ass.value.value {
                let mut continents = Vec::new();
                let mut strategic_regions = Vec::new();

                for inner in inner_entries {
                    if let ast::Entry::Assignment(inner_ass) = inner {
                        match inner_ass.key.as_str() {
                            "continents" => {
                                if let ast::Value::Block(cont_entries) = &inner_ass.value.value {
                                    for ce in cont_entries {
                                        if let ast::Entry::Value(val) = ce {
                                            if let ast::Value::String(name) = &val.value {
                                                continents.push(name.clone());
                                            }
                                        }
                                    }
                                }
                            }
                            "strategic_regions" => {
                                if let ast::Value::Block(sr_entries) = &inner_ass.value.value {
                                    for se in sr_entries {
                                        if let ast::Entry::Value(val) = se {
                                            if let ast::Value::Number(n) = &val.value {
                                                strategic_regions.push(*n as u32);
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }

                map.insert(
                    ass.key.clone(),
                    AiArea {
                        name: ass.key.clone(),
                        continents,
                        strategic_regions,
                        path: path.to_string_lossy().to_string(),
                        range: ass.key_range.clone(),
                    },
                );
            }
        }
    }
}
