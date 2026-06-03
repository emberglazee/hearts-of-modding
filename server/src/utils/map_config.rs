use crate::parser::ast;
use crate::parser::parser;
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
                    let key = ass.key_text(&content);
                    if key.eq_ignore_ascii_case("definitions") {
                        if let Some(s) = ass.value.value.as_str(&content) {
                            config.definitions = s.to_string();
                        }
                    } else if key.eq_ignore_ascii_case("adjacencies") {
                        if let Some(s) = ass.value.value.as_str(&content) {
                            config.adjacencies = s.to_string();
                        }
                    }
                }
            }
        }
    }
    config
}
