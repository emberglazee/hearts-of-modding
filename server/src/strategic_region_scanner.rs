use crate::ast;
use crate::parser;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StrategicRegion {
    pub id: u32,
    pub name: String,
    pub provinces: Vec<u32>,
    pub weather: Option<String>,
    pub naval_terrain: Option<String>,
    pub path: String,
    pub range: ast::Range,
}

pub fn scan_strategic_regions<F>(roots: &[PathBuf], filter: &F) -> HashMap<u32, StrategicRegion>
where
    F: Fn(&Path) -> bool,
{
    let mut regions = HashMap::new();

    for root in roots {
        let dir = root.join("map/strategicregions");
        if dir.exists() {
            let found = scan_directory(&dir, filter);
            regions.extend(found);
        }
    }

    regions
}

fn scan_directory<F>(dir_path: &Path, filter: &F) -> HashMap<u32, StrategicRegion>
where
    F: Fn(&Path) -> bool,
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
                        {
                            let (script, _) = parser::parse_script(&content);
                            extract_strategic_region(&script.entries, &path, &mut map);
                        }
                    }
                }
            }
        }
    }
    map
}

fn extract_strategic_region(
    entries: &[ast::Entry],
    path: &Path,
    map: &mut HashMap<u32, StrategicRegion>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            if ass.key.to_lowercase() == "strategic_region" {
                let mut region_id = None;
                let mut region_name = String::new();
                let mut provinces = Vec::new();
                let mut weather = None;
                let mut naval_terrain = None;

                if let ast::Value::Block(region_entries) = &ass.value.value {
                    for region_entry in region_entries {
                        if let ast::Entry::Assignment(r_ass) = region_entry {
                            match r_ass.key.to_lowercase().as_str() {
                                "id" => {
                                    if let ast::Value::Number(id) = &r_ass.value.value {
                                        region_id = Some(*id as u32);
                                    }
                                }
                                "name" => {
                                    if let ast::Value::String(name) = &r_ass.value.value {
                                        region_name = name.clone();
                                    }
                                }
                                "provinces" => {
                                    if let ast::Value::Block(prov_entries) = &r_ass.value.value {
                                        for prov_entry in prov_entries {
                                            if let ast::Entry::Value(val) = prov_entry {
                                                if let ast::Value::Number(p_id) = &val.value {
                                                    provinces.push(*p_id as u32);
                                                }
                                            }
                                        }
                                    } else if let ast::Value::TaggedBlock(_, prov_entries, _) =
                                        &r_ass.value.value
                                    {
                                        for prov_entry in prov_entries {
                                            if let ast::Entry::Value(val) = prov_entry {
                                                if let ast::Value::Number(p_id) = &val.value {
                                                    provinces.push(*p_id as u32);
                                                }
                                            }
                                        }
                                    }
                                }
                                "weather" => {
                                    weather = Some("Defined".to_string());
                                }
                                "naval_terrain" => {
                                    if let ast::Value::String(s) = &r_ass.value.value {
                                        naval_terrain = Some(s.clone());
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }

                if let Some(id) = region_id {
                    map.insert(
                        id,
                        StrategicRegion {
                            id,
                            name: region_name,
                            provinces,
                            weather,
                            naval_terrain,
                            path: path.to_string_lossy().to_string(),
                            range: ass.key_range.clone(),
                        },
                    );
                }
            }
        }
    }
}
