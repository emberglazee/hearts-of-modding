use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Continent {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub path: InternedStr,
    #[allow(dead_code)]
    pub range: ast::Range,
}

pub fn scan_continents(root: &Path) -> HashMap<String, Continent> {
    let mut map = HashMap::new();
    let path = root.join("map/continent.txt");
    if !path.exists() {
        return map;
    }
    if let Ok(content) = fs::read_to_string(&path) {
        let (script, _) = parser::parse_script(&content);
        for entry in &script.entries {
            if let ast::Entry::Assignment(ass) = entry {
                if ass.key == "continents" {
                    if let ast::Value::Block(inner) = &ass.value.value {
                        for inner_entry in inner.iter() {
                            if let ast::Entry::Value(val) = inner_entry {
                                if let ast::Value::String(name) = &val.value {
                                    map.insert(
                                        name.clone(),
                                        Continent {
                                            name: name.clone(),
                                            path: std::sync::Arc::from(
                                                path.to_string_lossy().as_ref(),
                                            ),
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
    map
}
