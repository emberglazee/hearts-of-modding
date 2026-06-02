use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Sprite {
    pub name: String,
    pub texture_file: String,
    pub path: InternedStr,
    pub range: ast::Range,
}

pub fn scan_sprites<F>(dir_path: &Path, filter: &F) -> HashMap<String, Sprite>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();
    crate::utils::fs_util::walk_and_parse_files(dir_path, &["gfx"], filter, |path, content| {
        let (script, parse_errors) = parser::parse_script(&content);
        find_sprites_in_entries(&script.entries, &path.to_string_lossy(), &mut map);
        for (e, range) in parse_errors {
            eprintln!(
                "Failed to parse GFX file {:?} at {}:{}: {}",
                path, range.start_line, range.start_col, e
            );
        }
    });
    map
}

pub(crate) fn find_sprites_in_entries(
    entries: &[ast::Entry],
    file_path: &str,
    map: &mut HashMap<String, Sprite>,
) {
    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                let key_lower = ass.key.to_ascii_lowercase();
                if key_lower == "spritetypes" {
                    if let ast::Value::Block(inner_entries) = &ass.value.value {
                        find_sprites_in_entries(inner_entries, file_path, map);
                    }
                } else if key_lower == "spritetype" {
                    parse_sprite_node(ass, file_path, map);
                } else {
                    // Recurse into other blocks just in case
                    if let ast::Value::Block(inner_entries) = &ass.value.value {
                        find_sprites_in_entries(inner_entries, file_path, map);
                    }
                }
            }
            ast::Entry::Value(val) => match &val.value {
                ast::Value::Block(inner_entries) => {
                    find_sprites_in_entries(inner_entries, file_path, map);
                }
                ast::Value::TaggedBlock(_, inner_entries, _) => {
                    find_sprites_in_entries(inner_entries, file_path, map);
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn parse_sprite_node(ass: &ast::Assignment, file_path: &str, map: &mut HashMap<String, Sprite>) {
    if let ast::Value::Block(details) = &ass.value.value {
        let mut name = None;
        let mut texture_file = None;

        for detail in details {
            if let ast::Entry::Assignment(d_ass) = detail {
                if d_ass.key.eq_ignore_ascii_case("name") {
                    if let ast::Value::String(s) = &d_ass.value.value {
                        name = Some(s.clone());
                    }
                } else if d_ass.key.eq_ignore_ascii_case("texturefile") {
                    if let ast::Value::String(s) = &d_ass.value.value {
                        texture_file = Some(s.clone());
                    }
                }
            }
        }

        if let (Some(n), Some(t)) = (name, texture_file) {
            map.insert(
                n.clone(),
                Sprite {
                    name: n,
                    texture_file: t,
                    path: std::sync::Arc::from(file_path),
                    range: ass.key_range.clone(),
                },
            );
        }
    }
}
