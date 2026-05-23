use crate::ast;
use crate::parser;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum PortraitBlockType {
    Default,
    Continent,
    Tag,
}

#[derive(Debug, Clone)]
pub struct Portrait {
    #[allow(dead_code)]
    pub name: String,
    pub block_type: PortraitBlockType,
    pub continent_name: Option<String>,
    pub has_male: bool,
    pub has_female: bool,
    pub has_army: bool,
    pub has_navy: bool,
    pub has_political: bool,
    pub has_operative: bool,
    pub has_scientist: bool,
    pub ideologies: Vec<String>,
    pub gfx_entries: Vec<String>,
    #[allow(dead_code)]
    pub path: String,
    #[allow(dead_code)]
    pub range: ast::Range,
}

pub fn scan_portraits<F>(roots: &[PathBuf], filter: &F) -> HashMap<String, Portrait>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut portraits = HashMap::new();

    for root in roots {
        let dir = root.join("portraits");
        if dir.exists() {
            let found = scan_directory(&dir, filter);
            portraits.extend(found);
        }
    }

    portraits
}

fn scan_directory<F>(dir_path: &Path, filter: &F) -> HashMap<String, Portrait>
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
                        extract_portraits(&script.entries, &path, &mut map);
                    }
                }
            }
        }
    }

    map
}

fn extract_portraits(entries: &[ast::Entry], path: &Path, map: &mut HashMap<String, Portrait>) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            if let ast::Value::Block(inner_entries) = &ass.value.value {
                let key_lower = ass.key.to_lowercase();
                let is_continent = key_lower == "continent";

                let mut continent_name = None;
                let mut has_male = false;
                let mut has_female = false;
                let mut has_army = false;
                let mut has_navy = false;
                let mut has_political = false;
                let mut has_operative = false;
                let mut has_scientist = false;
                let mut ideologies = Vec::new();
                let mut gfx_entries = Vec::new();

                for inner in inner_entries {
                    if let ast::Entry::Assignment(inner_ass) = inner {
                        match inner_ass.key.to_lowercase().as_str() {
                            "name" if is_continent => {
                                if let ast::Value::String(s) = &inner_ass.value.value {
                                    continent_name = Some(s.clone());
                                }
                            }
                            "male" => has_male = true,
                            "female" => has_female = true,
                            "army" => has_army = true,
                            "navy" => has_navy = true,
                            "operative" => has_operative = true,
                            "scientist" => has_scientist = true,
                            "political" => {
                                has_political = true;
                                if let ast::Value::Block(pol_entries) = &inner_ass.value.value {
                                    for pol_entry in pol_entries {
                                        if let ast::Entry::Assignment(pol_ass) = pol_entry {
                                            let ideo = pol_ass.key.clone();
                                            if !ideologies.contains(&ideo) {
                                                ideologies.push(ideo);
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }

                // Collect GFX references from string values in the block
                collect_gfx_references(inner_entries, &mut gfx_entries);

                let portrait_name = if is_continent {
                    continent_name.clone().unwrap_or_else(|| ass.key.clone())
                } else {
                    ass.key.clone()
                };

                let block_type = if is_continent {
                    PortraitBlockType::Continent
                } else if key_lower == "default" {
                    PortraitBlockType::Default
                } else {
                    PortraitBlockType::Tag
                };

                map.insert(
                    portrait_name.clone(),
                    Portrait {
                        name: portrait_name,
                        block_type,
                        continent_name,
                        has_male,
                        has_female,
                        has_army,
                        has_navy,
                        has_political,
                        has_operative,
                        has_scientist,
                        ideologies,
                        gfx_entries,
                        path: path.to_string_lossy().to_string(),
                        range: ass.key_range.clone(),
                    },
                );
            }
        }
    }
}

fn collect_gfx_references(entries: &[ast::Entry], gfx_list: &mut Vec<String>) {
    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                if let ast::Value::String(s) = &ass.value.value {
                    if s.starts_with("GFX_") && !gfx_list.contains(s) {
                        gfx_list.push(s.clone());
                    }
                }
                if let ast::Value::Block(inner) = &ass.value.value {
                    collect_gfx_references(inner, gfx_list);
                }
                if let ast::Value::TaggedBlock(_, inner, _) = &ass.value.value {
                    collect_gfx_references(inner, gfx_list);
                }
            }
            ast::Entry::Value(val) => {
                if let ast::Value::String(s) = &val.value {
                    if s.starts_with("GFX_") && !gfx_list.contains(s) {
                        gfx_list.push(s.clone());
                    }
                }
                if let ast::Value::Block(inner) = &val.value {
                    collect_gfx_references(inner, gfx_list);
                }
                if let ast::Value::TaggedBlock(_, inner, _) = &val.value {
                    collect_gfx_references(inner, gfx_list);
                }
            }
            _ => {}
        }
    }
}
