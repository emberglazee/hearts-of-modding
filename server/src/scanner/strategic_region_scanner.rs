#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StrategicRegion {
    pub id: u32,
    pub name: String,
    pub provinces: Vec<u32>,
    pub weather: Option<String>,
    pub naval_terrain: Option<String>,
    pub path: InternedStr,
    pub range: ast::Range,
}

pub fn scan_strategic_regions<F>(roots: &[PathBuf], filter: &F) -> HashMap<u32, StrategicRegion>
where
    F: Fn(&Path) -> bool,
{
    let mut regions = HashMap::new();

    for root in roots {
        crate::utils::fs_util::walk_and_parse_files(
            &root.join("map/strategicregions"),
            &["txt"],
            filter,
            |path, content| {
                let (script, _) = parser::parse_script(&content);
                extract_strategic_region(&script.entries, &script.source, path, &mut regions);
            },
        );
    }

    regions
}

pub fn scan_strategic_region_files<F>(
    files: &[PathBuf],
    filter: &F,
) -> HashMap<u32, StrategicRegion>
where
    F: Fn(&Path) -> bool,
{
    let mut regions = HashMap::new();
    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        extract_strategic_region(&script.entries, &script.source, &path, &mut regions);
    });
    regions
}

pub(crate) fn extract_strategic_region(
    entries: &[ast::Entry],
    source: &str,
    path: &Path,
    map: &mut HashMap<u32, StrategicRegion>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            if ass
                .key_text(source)
                .eq_ignore_ascii_case("strategic_region")
            {
                let mut region_id = None;
                let mut region_name = String::new();
                let mut provinces = Vec::new();
                let mut weather = None;
                let mut naval_terrain = None;

                if let ast::Value::Block(region_entries) = &ass.value.value {
                    for region_entry in region_entries {
                        if let ast::Entry::Assignment(r_ass) = region_entry {
                            let r_key = r_ass.key_text(source);
                            if r_key.eq_ignore_ascii_case("id") {
                                if let ast::Value::Number(id) = &r_ass.value.value {
                                    region_id = Some(*id as u32);
                                }
                            } else if r_key.eq_ignore_ascii_case("name") {
                                if let Some(name) = r_ass.value.value.as_str(source) {
                                    region_name = name.to_string();
                                }
                            } else if r_key.eq_ignore_ascii_case("provinces") {
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
                            } else if r_key.eq_ignore_ascii_case("weather") {
                                weather = Some("Defined".to_string());
                            } else if r_key.eq_ignore_ascii_case("naval_terrain") {
                                if let Some(s) = r_ass.value.value.as_str(source) {
                                    naval_terrain = Some(s.to_string());
                                }
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
                            path: std::sync::Arc::from(path.to_string_lossy().as_ref()),
                            range: ass.key_range.clone(),
                        },
                    );
                }
            }
        }
    }
}
