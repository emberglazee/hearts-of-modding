use crate::ast;
use crate::parser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Building {
    #[allow(dead_code)]
    pub name: String,
    pub max_level: Option<i32>,
    #[allow(dead_code)]
    pub path: String,
    #[allow(dead_code)]
    pub range: ast::Range,
}

pub fn scan_buildings<F>(roots: &[PathBuf], filter: &F) -> HashMap<String, Building>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut buildings = HashMap::new();

    for root in roots {
        crate::fs_util::walk_and_parse_files(
            &root.join("common/buildings"),
            &["txt"],
            filter,
            |path, content| {
                let (script, _) = parser::parse_script(&content);
                extract_buildings(&script.entries, path, &mut buildings);
            },
        );
    }

    buildings
}

pub(crate) fn extract_buildings(entries: &[ast::Entry], path: &Path, map: &mut HashMap<String, Building>) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let building_name = ass.key.clone();
            let mut max_level = None;

            // Extract max_level from building definition
            if let ast::Value::Block(building_entries) = &ass.value.value {
                for building_entry in building_entries {
                    if let ast::Entry::Assignment(building_ass) = building_entry {
                        if building_ass.key.eq_ignore_ascii_case("max_level")
                            && let ast::Value::Number(level) = &building_ass.value.value
                        {
                            max_level = Some(*level as i32);
                        } else if let ast::Value::String(s) = &building_ass.value.value {
                            max_level = s.parse::<i32>().ok();
                        }
                    }
                }
            }

            map.insert(
                building_name.clone(),
                Building {
                    name: building_name,
                    max_level,
                    path: path.to_string_lossy().to_string(),
                    range: ass.key_range.clone(),
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_extract_buildings() {
        // Test would require mock AST data
    }
}
