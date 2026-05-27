use crate::ast;
use crate::parser;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ScriptedEntity {
    pub name: String,
    pub path: String,
    pub range: ast::Range,
}

pub fn scan_directory<F>(dir_path: &Path, filter: &F) -> HashMap<String, ScriptedEntity>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();
    crate::fs_util::walk_and_parse_files(dir_path, &["txt"], filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        for entry_ast in script.entries {
            if let ast::Entry::Assignment(ass) = entry_ast {
                map.insert(
                    ass.key.clone(),
                    ScriptedEntity {
                        name: ass.key.clone(),
                        path: path.to_string_lossy().to_string(),
                        range: ass.key_range,
                    },
                );
            }
        }
    });
    map
}
