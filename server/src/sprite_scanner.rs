use crate::ast;
use crate::parser;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Sprite {
    pub name: String,
    pub texture_file: String,
    pub path: String,
    pub range: ast::Range,
}

pub fn scan_sprites<F>(dir_path: &Path, filter: &F) -> HashMap<String, Sprite>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();
    if !dir_path.exists() || filter(dir_path) {
        return map;
    }

    let mut dirs_to_check = vec![dir_path.to_path_buf()];
    while let Some(current_dir) = dirs_to_check.pop() {
        if let Ok(entries) = fs::read_dir(current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if !filter(&path) {
                        dirs_to_check.push(path);
                    }
                } else if path.extension().is_some_and(|ext| ext == "gfx") {
                    if filter(&path) {
                        continue;
                    }
                    if let Ok(content) = fs::read_to_string(&path) {
                        let (script, parse_errors) = parser::parse_script(&content);
                        find_sprites_in_entries(&script.entries, &path.to_string_lossy(), &mut map);
                        for (e, range) in parse_errors {
                            // We can't easily log to the client from here without passing it down,
                            // but we should at least handle the error gracefully.
                            eprintln!(
                                "Failed to parse GFX file {:?} at {}:{}: {}",
                                path, range.start_line, range.start_col, e
                            );
                        }
                    }
                }
            }
        }
    }
    map
}

fn find_sprites_in_entries(
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
                    path: file_path.to_string(),
                    range: ass.key_range.clone(),
                },
            );
        }
    }
}
