#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Idea {
    pub name: String,
    pub category: String,
    #[allow(dead_code)]
    pub picture: Option<String>,
    pub path: InternedStr,
    pub range: ast::Range,
}

pub fn scan_ideas<F>(dir_path: &Path, filter: &F) -> HashMap<String, Idea>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();
    crate::utils::fs_util::walk_and_parse_files(dir_path, &["txt"], filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        find_ideas_in_entries(
            &script.entries,
            &script.source,
            &path.to_string_lossy(),
            &mut map,
        );
    });
    map
}

pub fn scan_idea_files<F>(files: &[PathBuf], filter: &F) -> HashMap<String, Idea>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();
    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        find_ideas_in_entries(
            &script.entries,
            &script.source,
            &path.to_string_lossy(),
            &mut map,
        );
    });
    map
}

pub(crate) fn find_ideas_in_entries(
    entries: &[ast::Entry],
    source: &str,
    file_path: &str,
    map: &mut HashMap<String, Idea>,
) {
    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                if ass.key_text(source).eq_ignore_ascii_case("ideas") {
                    parse_ideas_block(ass, source, file_path, map);
                } else {
                    // Recurse into other blocks
                    if let ast::Value::Block(inner_entries) = &ass.value.value {
                        find_ideas_in_entries(inner_entries, source, file_path, map);
                    }
                }
            }
            ast::Entry::Value(val) => match &val.value {
                ast::Value::Block(inner_entries) => {
                    find_ideas_in_entries(inner_entries, source, file_path, map);
                }
                ast::Value::TaggedBlock(_, inner_entries, _) => {
                    find_ideas_in_entries(inner_entries, source, file_path, map);
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn parse_ideas_block(
    ass: &ast::Assignment,
    source: &str,
    file_path: &str,
    map: &mut HashMap<String, Idea>,
) {
    if let ast::Value::Block(categories) = &ass.value.value {
        for category_entry in categories {
            if let ast::Entry::Assignment(cat_ass) = category_entry {
                let category_name = cat_ass.key_text(source).to_string();
                if let ast::Value::Block(ideas) = &cat_ass.value.value {
                    for idea_entry in ideas {
                        if let ast::Entry::Assignment(idea_ass) = idea_entry {
                            let mut picture = None;
                            if let ast::Value::Block(details) = &idea_ass.value.value {
                                for detail in details {
                                    if let ast::Entry::Assignment(d_ass) = detail
                                        && d_ass.key_text(source).eq_ignore_ascii_case("picture")
                                        && let Some(s) = d_ass.value.value.as_str(source)
                                    {
                                        picture = Some(s.to_string());
                                    }
                                }
                            }

                            let name = idea_ass.key_text(source).to_string();
                            map.insert(
                                name.clone(),
                                Idea {
                                    name,
                                    category: category_name.clone(),
                                    picture,
                                    path: std::sync::Arc::from(file_path),
                                    range: idea_ass.key_range.clone(),
                                },
                            );
                        }
                    }
                }
            }
        }
    }
}
