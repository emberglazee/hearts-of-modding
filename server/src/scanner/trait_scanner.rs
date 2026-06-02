use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Trait {
    pub name: String,
    pub trait_type: String, // e.g., "Leader Trait", "Country Leader Trait"
    pub path: InternedStr,
    pub range: ast::Range,
}

pub fn scan_traits<F>(dir_path: &Path, trait_type: &str, filter: &F) -> HashMap<String, Trait>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();
    crate::utils::fs_util::walk_and_parse_files(dir_path, &["txt"], filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        find_traits_in_entries(
            &script.entries,
            &path.to_string_lossy(),
            trait_type,
            &mut map,
        );
    });
    map
}

pub(crate) fn find_traits_in_entries(
    entries: &[ast::Entry],
    file_path: &str,
    trait_type: &str,
    map: &mut HashMap<String, Trait>,
) {
    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                let key_lower = ass.key.to_ascii_lowercase();
                if key_lower == "leader_traits"
                    || key_lower == "country_leader_traits"
                    || key_lower == "traits"
                {
                    if let ast::Value::Block(trait_entries) = &ass.value.value {
                        for trait_entry in trait_entries {
                            if let ast::Entry::Assignment(t_ass) = trait_entry {
                                map.insert(
                                    t_ass.key.clone(),
                                    Trait {
                                        name: t_ass.key.clone(),
                                        trait_type: trait_type.to_string(),
                                        path: std::sync::Arc::from(file_path),
                                        range: t_ass.key_range.clone(),
                                    },
                                );
                            }
                        }
                    }
                } else {
                    // Recurse into other blocks
                    if let ast::Value::Block(inner_entries) = &ass.value.value {
                        find_traits_in_entries(inner_entries, file_path, trait_type, map);
                    }
                }
            }
            ast::Entry::Value(val) => match &val.value {
                ast::Value::Block(inner_entries) => {
                    find_traits_in_entries(inner_entries, file_path, trait_type, map);
                }
                ast::Value::TaggedBlock(_, inner_entries, _) => {
                    find_traits_in_entries(inner_entries, file_path, trait_type, map);
                }
                _ => {}
            },
            _ => {}
        }
    }
}
