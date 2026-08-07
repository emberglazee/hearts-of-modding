#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Continent {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub path: InternedStr,
    #[allow(dead_code)]
    pub range: ast::Range,
}

/// Extract continent names from an already-parsed `map/continent.txt` AST.
///
/// Shared by the full scan, the winning-files scan, and the incremental
/// updater so all three agree on what counts as a continent.
pub(crate) fn extract_continents(
    entries: &[ast::Entry],
    source: &str,
    path: &Path,
    map: &mut HashMap<String, Continent>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            if ass.key_text(source) == "continents" {
                if let ast::Value::Block(inner) = &ass.value.value {
                    for inner_entry in inner.iter() {
                        if let ast::Entry::Value(val) = inner_entry {
                            if let Some(name) = val.value.as_str(source) {
                                map.insert(
                                    name.to_string(),
                                    Continent {
                                        name: name.to_string(),
                                        path: std::sync::Arc::from(path.to_string_lossy().as_ref()),
                                        range: ast::Range {
                                            start_line: val.range.start_line,
                                            start_col: val.range.start_col,
                                            end_line: val.range.end_line,
                                            end_col: val.range.end_col,
                                        },
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

pub fn scan_continents(root: &Path) -> HashMap<String, Continent> {
    let mut map = HashMap::new();
    let path = root.join("map/continent.txt");
    if !path.exists() {
        return map;
    }
    if let Ok(content) = fs::read_to_string(&path) {
        let (script, _) = parser::parse_script(&content);
        extract_continents(&script.entries, &script.source, &path, &mut map);
    }
    map
}

pub fn scan_continent_files(files: &[PathBuf]) -> HashMap<String, Continent> {
    let mut map = HashMap::new();
    for path in files {
        let filename = path.to_string_lossy();
        if filename.ends_with("continent.txt") {
            if let Ok(content) = std::fs::read_to_string(path) {
                let (script, _) = parser::parse_script(&content);
                extract_continents(&script.entries, &script.source, path, &mut map);
            }
        }
    }
    map
}
