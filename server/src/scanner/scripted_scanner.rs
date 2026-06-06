#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ScriptedEntity {
    pub name: String,
    pub path: InternedStr,
    pub range: ast::Range,
}

pub fn scan_directory<F>(dir_path: &Path, filter: &F) -> HashMap<String, ScriptedEntity>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();
    crate::utils::fs_util::walk_and_parse_files(dir_path, &["txt"], filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        for entry_ast in script.entries {
            if let ast::Entry::Assignment(ass) = entry_ast {
                let name = ass.key_text(&script.source).to_string();
                map.insert(
                    name.clone(),
                    ScriptedEntity {
                        name,
                        path: std::sync::Arc::from(path.to_string_lossy().as_ref()),
                        range: ass.key_range,
                    },
                );
            }
        }
    });
    map
}

pub fn scan_scripted_files<F>(files: &[PathBuf], filter: &F) -> HashMap<String, ScriptedEntity>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();
    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        for entry_ast in script.entries {
            if let ast::Entry::Assignment(ass) = entry_ast {
                let name = ass.key_text(&script.source).to_string();
                map.insert(
                    name.clone(),
                    ScriptedEntity {
                        name,
                        path: std::sync::Arc::from(path.to_string_lossy().as_ref()),
                        range: ass.key_range,
                    },
                );
            }
        }
    });
    map
}
