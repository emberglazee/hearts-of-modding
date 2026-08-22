#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// What kind of technology tag this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TechnologyTagKind {
    /// From `technology_categories = { A B C }`
    Category,
    /// From `technology_folders = { name = { ledger = army } }`
    Folder,
}

/// A technology tag — either a category or a folder — defined in
/// `common/technology_tags/*.txt`.
///
/// Categories are bare identifiers in `technology_categories = { A B C }`.
/// Folders are key=value blocks in `technology_folders = { name = { ledger = army } }`.
///
/// Struct-with-discriminator rather than an enum so the standard
/// infrastructure (`HasPath` field access, `try_lookup!`, `rebuild_index!`)
/// works unchanged.
#[derive(Debug, Clone)]
pub struct TechnologyTag {
    pub name: String,
    pub path: InternedStr,
    pub range: ast::Range,
    pub tag_kind: TechnologyTagKind,
    /// Folders only: `ledger = army|navy|civilian`
    pub ledger: Option<String>,
    /// Folders only: whether this folder holds doctrines
    pub doctrine: bool,
}

/// Extract technology tags from a parsed script AST.
/// Used by both initial scan AND incremental update.
pub(crate) fn find_tags_in_entries(
    entries: &[ast::Entry],
    source: &str,
    file_path: &str,
    map: &mut HashMap<String, TechnologyTag>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let key = ass.key_text(source);

            // technology_categories = { cat1 cat2 cat3 }
            if key.eq_ignore_ascii_case("technology_categories") {
                if let ast::Value::Block(cat_entries) = &ass.value.value {
                    for cat_entry in cat_entries {
                        // Bare identifier: `light_air`
                        if let ast::Entry::Value(nv) = cat_entry {
                            if let ast::Value::String(span) = &nv.value {
                                let name = span.resolve(source).to_string();
                                map.insert(
                                    name.clone(),
                                    TechnologyTag {
                                        name,
                                        path: file_path.into(),
                                        range: nv.range.clone(),
                                        tag_kind: TechnologyTagKind::Category,
                                        ledger: None,
                                        doctrine: false,
                                    },
                                );
                            }
                        }
                    }
                }
            }

            // technology_folders = { folder_name = { ledger = army } }
            if key.eq_ignore_ascii_case("technology_folders") {
                if let ast::Value::Block(folder_entries) = &ass.value.value {
                    for folder_entry in folder_entries {
                        if let ast::Entry::Assignment(folder_ass) = folder_entry {
                            let folder_name = folder_ass.key_text(source).to_string();
                            let mut ledger = None;
                            let mut doctrine = false;

                            if let ast::Value::Block(props) = &folder_ass.value.value {
                                for prop in props {
                                    if let ast::Entry::Assignment(prop_ass) = prop {
                                        let prop_key = prop_ass.key_text(source);
                                        if prop_key.eq_ignore_ascii_case("ledger") {
                                            if let Some(s) = prop_ass.value.value.as_str(source) {
                                                ledger = Some(s.to_string());
                                            }
                                        }
                                        if prop_key.eq_ignore_ascii_case("doctrine") {
                                            doctrine = match &prop_ass.value.value {
                                                ast::Value::String(span) => {
                                                    span.resolve(source).eq_ignore_ascii_case("yes")
                                                }
                                                ast::Value::Boolean(b) => *b,
                                                _ => false,
                                            };
                                        }
                                    }
                                }
                            }

                            map.insert(
                                folder_name.clone(),
                                TechnologyTag {
                                    name: folder_name,
                                    path: file_path.into(),
                                    range: folder_ass.key_range.clone(),
                                    tag_kind: TechnologyTagKind::Folder,
                                    ledger,
                                    doctrine,
                                },
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Scan a list of winning technology tag files and extract all tags.
pub fn scan_technology_tag_files<F>(files: &[PathBuf], filter: &F) -> HashMap<String, TechnologyTag>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();

    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        find_tags_in_entries(
            &script.entries,
            &script.source,
            &path.to_string_lossy(),
            &mut map,
        );
    });

    map
}
