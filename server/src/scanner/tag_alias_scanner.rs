#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct TagAlias {
    pub name: String,
    pub path: InternedStr,
    pub range: ast::Range,
}

pub fn scan_tag_aliases<F>(roots: &[PathBuf], filter: &F) -> HashMap<String, TagAlias>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();
    for root in roots {
        let dir = root.join("common/country_tag_aliases");
        if !dir.exists() {
            continue;
        }
        crate::utils::fs_util::walk_and_parse_files(&dir, &["txt"], filter, |path, content| {
            let (script, _) = parser::parse_script(&content);
            find_tag_aliases_in_entries(
                &script.entries,
                &script.source,
                &path.to_string_lossy(),
                &mut map,
            );
        });
    }
    map
}

pub fn scan_tag_alias_files<F>(files: &[PathBuf], filter: &F) -> HashMap<String, TagAlias>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();
    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        find_tag_aliases_in_entries(
            &script.entries,
            &script.source,
            &path.to_string_lossy(),
            &mut map,
        );
    });
    map
}

pub(crate) fn find_tag_aliases_in_entries(
    entries: &[ast::Entry],
    source: &str,
    file_path: &str,
    map: &mut HashMap<String, TagAlias>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let name = ass.key_text(source).to_string();
            // Only capture entries that look like tag aliases (3-letter uppercase with block)
            if name.len() == 3
                && name.as_bytes()[0].is_ascii_uppercase()
                && matches!(&ass.value.value, ast::Value::Block(_))
            {
                map.insert(
                    name.clone(),
                    TagAlias {
                        name,
                        path: std::sync::Arc::from(file_path),
                        range: ass.key_range.clone(),
                    },
                );
            }
        }
    }
}
