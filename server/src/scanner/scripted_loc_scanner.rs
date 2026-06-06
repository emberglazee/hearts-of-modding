#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::parser::ast::{self, Entry, Value};
use crate::parser::parser;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ScriptedLoc {
    pub name: String,
    pub path: InternedStr,
    pub range: ast::Range,
}

pub fn scan_directory<F>(dir_path: &Path, filter: &F) -> HashMap<String, ScriptedLoc>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();
    crate::utils::fs_util::walk_and_parse_files(dir_path, &["txt"], filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        find_scripted_locs_in_entries(
            &script.entries,
            &script.source,
            &path.to_string_lossy(),
            &mut map,
        );
    });
    map
}

pub fn scan_scripted_loc_files<F>(
    files: &[std::path::PathBuf],
    filter: &F,
) -> HashMap<String, ScriptedLoc>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();
    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        find_scripted_locs_in_entries(
            &script.entries,
            &script.source,
            &path.to_string_lossy(),
            &mut map,
        );
    });
    map
}

pub(crate) fn find_scripted_locs_in_entries(
    entries: &[Entry],
    source: &str,
    file_path: &str,
    map: &mut HashMap<String, ScriptedLoc>,
) {
    for entry in entries {
        if let Entry::Assignment(ass) = entry
            && ass.key_text(source) == "defined_text"
            && let Value::Block(children) = &ass.value.value
        {
            for child in children {
                if let Entry::Assignment(child_ass) = child
                    && child_ass.key_text(source) == "name"
                    && let Some(name) = child_ass.value.value.as_str(source)
                {
                    map.insert(
                        name.to_string(),
                        ScriptedLoc {
                            name: name.to_string(),
                            path: std::sync::Arc::from(file_path),
                            range: child_ass.value.range.clone(),
                        },
                    );
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_scripted_locs() {
        let content = r#"
defined_text = {
	name = DBUG_show_lar_decisions
	text = {
		trigger = {
			NOT = { has_dlc = "La Resistance" }
		}
		localization_key = DBUG_show_lar_di_decisions
	}
	text = {
		trigger = { has_dlc = "La Resistance" }
		localization_key = DBUG_show_lar_en_decisions
	}
}
        "#;
        let script = crate::parser::parser::parse_script(content).0;
        let mut map = HashMap::new();
        find_scripted_locs_in_entries(&script.entries, &script.source, "test", &mut map);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("DBUG_show_lar_decisions"));
    }
}
