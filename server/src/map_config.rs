use crate::ast;
use crate::parser;
use std::fs;
use std::path::Path;

pub struct MapConfig {
    pub definitions: String,
    pub adjacencies: String,
}

impl Default for MapConfig {
    fn default() -> Self {
        MapConfig {
            definitions: "definition.csv".to_string(),
            adjacencies: "adjacencies.csv".to_string(),
        }
    }
}

pub fn get_map_config(root: &Path) -> MapConfig {
    let mut config = MapConfig::default();
    let default_map_path = root.join("map/default.map");
    if default_map_path.exists() {
        if let Ok(content) = fs::read_to_string(&default_map_path) {
            let (script, _) = parser::parse_script(&content);
            for entry in script.entries {
                if let ast::Entry::Assignment(ass) = entry {
                    let key = ass.key.as_str();
                    if key.eq_ignore_ascii_case("definitions") {
                        if let ast::Value::String(s) = ass.value.value {
                            config.definitions = s;
                        }
                    } else if key.eq_ignore_ascii_case("adjacencies") {
                        if let ast::Value::String(s) = ass.value.value {
                            config.adjacencies = s;
                        }
                    }
                }
            }
        }
    }
    config
}
