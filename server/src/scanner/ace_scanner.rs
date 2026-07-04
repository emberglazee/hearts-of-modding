#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AceModifier {
    pub name: String,
    pub path: InternedStr,
    pub range: ast::Range,
}

pub fn scan_ace_files<F>(files: &[PathBuf], filter: &F) -> HashMap<String, AceModifier>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();

    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        find_aces_in_entries(
            &script.entries,
            &script.source,
            &path.to_string_lossy(),
            &mut map,
        );
    });

    map
}

pub(crate) fn find_aces_in_entries(
    entries: &[ast::Entry],
    source: &str,
    file_path: &str,
    map: &mut HashMap<String, AceModifier>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            if ass.key_text(source).eq_ignore_ascii_case("modifiers") {
                if let ast::Value::Block(inner) = &ass.value.value {
                    for inner_entry in inner {
                        if let ast::Entry::Assignment(ace_ass) = inner_entry {
                            if let ast::Value::Block(_) = &ace_ass.value.value {
                                let name = ace_ass.key_text(source).to_string();
                                map.insert(
                                    name.clone(),
                                    AceModifier {
                                        name,
                                        path: std::sync::Arc::from(file_path),
                                        range: ace_ass.key_range.clone(),
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
