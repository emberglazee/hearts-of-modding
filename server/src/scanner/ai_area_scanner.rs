use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
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
    pub path: InternedStr,
    #[allow(dead_code)]
    pub range: ast::Range,
}

pub fn scan_ai_areas<F>(roots: &[PathBuf], filter: &F) -> HashMap<String, AiArea>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut areas = HashMap::new();

    for root in roots {
        crate::utils::fs_util::walk_and_parse_files(
            &root.join("common/ai_areas"),
            &["txt"],
            filter,
            |path, content| {
                let (script, _) = parser::parse_script(&content);
                extract_areas(&script.entries, path, &mut areas);
            },
        );
    }

    areas
}

pub(crate) fn extract_areas(
    entries: &[ast::Entry],
    path: &Path,
    map: &mut HashMap<String, AiArea>,
) {
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
                        path: std::sync::Arc::from(path.to_string_lossy().as_ref()),
                        range: ass.key_range.clone(),
                    },
                );
            }
        }
    }
}
