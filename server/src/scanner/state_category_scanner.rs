use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct StateCategory {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub local_building_slots: Option<i32>,
    #[allow(dead_code)]
    pub path: InternedStr,
    #[allow(dead_code)]
    pub range: ast::Range,
}

pub fn scan_state_categories<F>(roots: &[PathBuf], filter: &F) -> HashMap<String, StateCategory>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut categories = HashMap::new();

    for root in roots {
        crate::utils::fs_util::walk_and_parse_files(
            &root.join("common/state_category"),
            &["txt"],
            filter,
            |path, content| {
                let (script, _) = parser::parse_script(&content);
                extract_categories(&script.entries, &script.source, path, &mut categories);
            },
        );
    }

    categories
}

pub(crate) fn extract_categories(
    entries: &[ast::Entry],
    source: &str,
    path: &Path,
    map: &mut HashMap<String, StateCategory>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            if ass
                .key_text(source)
                .eq_ignore_ascii_case("state_categories")
            {
                if let ast::Value::Block(category_entries) = &ass.value.value {
                    for category_entry in category_entries {
                        if let ast::Entry::Assignment(cat_def) = category_entry {
                            let mut local_building_slots = None;

                            if let ast::Value::Block(def_entries) = &cat_def.value.value {
                                for def_entry in def_entries {
                                    if let ast::Entry::Assignment(def_ass) = def_entry {
                                        if def_ass
                                            .key_text(source)
                                            .eq_ignore_ascii_case("local_building_slots")
                                        {
                                            if let ast::Value::Number(n) = &def_ass.value.value {
                                                local_building_slots = Some(*n as i32);
                                            }
                                        }
                                    }
                                }
                            }

                            map.insert(
                                cat_def.key_text(source).to_string(),
                                StateCategory {
                                    name: cat_def.key_text(source).to_string(),
                                    local_building_slots,
                                    path: std::sync::Arc::from(path.to_string_lossy().as_ref()),
                                    range: cat_def.key_range.clone(),
                                },
                            );
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_categories() {
        let content = r#"state_categories = {
    wasteland = {
        color = { 0 255 0 }
        local_building_slots = 0
    }
    rural = {
        color = { 0 128 0 }
        local_building_slots = 5
    }
}"#;
        let (script, _) = parser::parse_script(content);
        let mut map = HashMap::new();
        extract_categories(
            &script.entries,
            &script.source,
            std::path::Path::new("test.txt"),
            &mut map,
        );

        assert_eq!(map.len(), 2);
        assert!(map.contains_key("wasteland"));
        assert!(map.contains_key("rural"));
        assert_eq!(map.get("wasteland").unwrap().local_building_slots, Some(0));
        assert_eq!(map.get("rural").unwrap().local_building_slots, Some(5));
    }
}
