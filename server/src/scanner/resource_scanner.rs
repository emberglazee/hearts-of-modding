use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Resource {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub icon_frame: i32,
    #[allow(dead_code)]
    pub path: InternedStr,
    #[allow(dead_code)]
    pub range: ast::Range,
}

pub fn scan_resources<F>(roots: &[PathBuf], filter: &F) -> HashMap<String, Resource>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut resources = HashMap::new();

    for root in roots {
        crate::utils::fs_util::walk_and_parse_files(
            &root.join("common/resources"),
            &["txt"],
            filter,
            |path, content| {
                let (script, _) = parser::parse_script(&content);
                extract_resources(&script.entries, path, &mut resources);
            },
        );
    }

    resources
}

pub(crate) fn extract_resources(
    entries: &[ast::Entry],
    path: &Path,
    map: &mut HashMap<String, Resource>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            if ass.key.eq_ignore_ascii_case("resources") {
                if let ast::Value::Block(resource_entries) = &ass.value.value {
                    for resource_entry in resource_entries {
                        if let ast::Entry::Assignment(resource_def) = resource_entry {
                            let resource_name = resource_def.key.clone();
                            let mut icon_frame = 0;

                            if let ast::Value::Block(def_entries) = &resource_def.value.value {
                                for def_entry in def_entries {
                                    if let ast::Entry::Assignment(def_ass) = def_entry {
                                        if def_ass.key.eq_ignore_ascii_case("icon_frame") {
                                            if let ast::Value::Number(n) = &def_ass.value.value {
                                                icon_frame = *n as i32;
                                            }
                                        }
                                    }
                                }
                            }

                            map.insert(
                                resource_name,
                                Resource {
                                    name: resource_def.key.clone(),
                                    icon_frame,
                                    path: std::sync::Arc::from(path.to_string_lossy().as_ref()),
                                    range: resource_def.key_range.clone(),
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
    fn test_extract_resources() {
        let content = r#"resources = {
    oil = {
        icon_frame = 1
        cic = 0.125
        convoys = 0.1
    }
    steel = {
        icon_frame = 2
        cic = 0.125
        convoys = 0.1
    }
}"#;
        let (script, _) = parser::parse_script(content);
        let mut map = HashMap::new();
        extract_resources(&script.entries, std::path::Path::new("test.txt"), &mut map);

        assert_eq!(map.len(), 2);
        assert!(map.contains_key("oil"));
        assert!(map.contains_key("steel"));
        assert_eq!(map.get("oil").unwrap().icon_frame, 1);
        assert_eq!(map.get("steel").unwrap().icon_frame, 2);
    }
}
