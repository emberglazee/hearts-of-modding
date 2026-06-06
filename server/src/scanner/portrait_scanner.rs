#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
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
    pub path: InternedStr,
    #[allow(dead_code)]
    pub range: ast::Range,
}

pub fn scan_portraits<F>(roots: &[PathBuf], filter: &F) -> HashMap<String, Portrait>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut portraits = HashMap::new();

    for root in roots {
        crate::utils::fs_util::walk_and_parse_files(
            &root.join("portraits"),
            &["txt"],
            filter,
            |path, content| {
                let (script, _) = parser::parse_script(&content);
                extract_portraits(&script.entries, &script.source, path, &mut portraits);
            },
        );
    }

    portraits
}

pub fn scan_portrait_files<F>(files: &[PathBuf], filter: &F) -> HashMap<String, Portrait>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut portraits = HashMap::new();
    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        extract_portraits(&script.entries, &script.source, path, &mut portraits);
    });
    portraits
}

pub(crate) fn extract_portraits(
    entries: &[ast::Entry],
    source: &str,
    path: &Path,
    map: &mut HashMap<String, Portrait>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            if let ast::Value::Block(inner_entries) = &ass.value.value {
                let is_continent = ass.key_text(source).eq_ignore_ascii_case("continent");

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
                        let inner_key = inner_ass.key_text(source);
                        if inner_key.eq_ignore_ascii_case("name") && is_continent {
                            if let Some(s) = inner_ass.value.value.as_str(source) {
                                continent_name = Some(s.to_string());
                            }
                        } else if inner_key.eq_ignore_ascii_case("male") {
                            has_male = true;
                        } else if inner_key.eq_ignore_ascii_case("female") {
                            has_female = true;
                        } else if inner_key.eq_ignore_ascii_case("army") {
                            has_army = true;
                        } else if inner_key.eq_ignore_ascii_case("navy") {
                            has_navy = true;
                        } else if inner_key.eq_ignore_ascii_case("operative") {
                            has_operative = true;
                        } else if inner_key.eq_ignore_ascii_case("scientist") {
                            has_scientist = true;
                        } else if inner_key.eq_ignore_ascii_case("political") {
                            has_political = true;
                            if let ast::Value::Block(pol_entries) = &inner_ass.value.value {
                                for pol_entry in pol_entries {
                                    if let ast::Entry::Assignment(pol_ass) = pol_entry {
                                        let ideo = pol_ass.key_text(source).to_string();
                                        if !ideologies.contains(&ideo) {
                                            ideologies.push(ideo);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Collect GFX references from string values in the block
                collect_gfx_references(inner_entries, source, &mut gfx_entries);

                let portrait_name = if is_continent {
                    continent_name
                        .clone()
                        .unwrap_or_else(|| ass.key_text(source).to_string())
                } else {
                    ass.key_text(source).to_string()
                };

                let block_type = if is_continent {
                    PortraitBlockType::Continent
                } else if ass.key_text(source).eq_ignore_ascii_case("default") {
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
                        path: std::sync::Arc::from(path.to_string_lossy().as_ref()),
                        range: ass.key_range.clone(),
                    },
                );
            }
        }
    }
}

fn collect_gfx_references(entries: &[ast::Entry], source: &str, gfx_list: &mut Vec<String>) {
    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                if let Some(s) = ass.value.value.as_str(source) {
                    if s.starts_with("GFX_") && !gfx_list.iter().any(|x| x.as_str() == s) {
                        gfx_list.push(s.to_string());
                    }
                }
                if let ast::Value::Block(inner) = &ass.value.value {
                    collect_gfx_references(inner, source, gfx_list);
                }
                if let ast::Value::TaggedBlock(_, inner, _) = &ass.value.value {
                    collect_gfx_references(inner, source, gfx_list);
                }
            }
            ast::Entry::Value(val) => {
                if let Some(s) = val.value.as_str(source) {
                    if s.starts_with("GFX_") && !gfx_list.iter().any(|x| x.as_str() == s) {
                        gfx_list.push(s.to_string());
                    }
                }
                if let ast::Value::Block(inner) = &val.value {
                    collect_gfx_references(inner, source, gfx_list);
                }
                if let ast::Value::TaggedBlock(_, inner, _) = &val.value {
                    collect_gfx_references(inner, source, gfx_list);
                }
            }
            _ => {}
        }
    }
}
