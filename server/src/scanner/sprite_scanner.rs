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
        find_sprites_in_entries(
            &script.entries,
            &script.source,
            &path.to_string_lossy(),
            &mut map,
        );
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
    source: &str,
    file_path: &str,
    map: &mut HashMap<String, Sprite>,
) {
    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                let key_lower = ass.key_text(source).to_ascii_lowercase();
                // Match sprite type container (spriteTypes = { ... })
                if key_lower == "spritetypes" {
                    if let ast::Value::Block(inner_entries) = &ass.value.value {
                        find_sprites_in_entries(inner_entries, source, file_path, map);
                    }
                }
                // Match any key ending with "spritetype" — handles spriteType,
                // frameAnimatedSpriteType, corneredTileSpriteType, etc.
                else if key_lower.ends_with("spritetype") {
                    parse_sprite_node(ass, source, file_path, map);
                } else {
                    // Recurse into other blocks just in case
                    if let ast::Value::Block(inner_entries) = &ass.value.value {
                        find_sprites_in_entries(inner_entries, source, file_path, map);
                    }
                }
            }
            ast::Entry::Value(val) => match &val.value {
                ast::Value::Block(inner_entries) => {
                    find_sprites_in_entries(inner_entries, source, file_path, map);
                }
                ast::Value::TaggedBlock(_, inner_entries, _) => {
                    find_sprites_in_entries(inner_entries, source, file_path, map);
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn parse_sprite_node(
    ass: &ast::Assignment,
    source: &str,
    file_path: &str,
    map: &mut HashMap<String, Sprite>,
) {
    if let ast::Value::Block(details) = &ass.value.value {
        let mut name = None;
        let mut texture_file = None;

        for detail in details {
            if let ast::Entry::Assignment(d_ass) = detail {
                if d_ass.key_text(source).eq_ignore_ascii_case("name") {
                    if let Some(s) = d_ass.value.value.as_str(source) {
                        name = Some(s.to_string());
                    }
                } else if d_ass.key_text(source).eq_ignore_ascii_case("texturefile") {
                    if let Some(s) = d_ass.value.value.as_str(source) {
                        texture_file = Some(s.to_string());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parser;

    fn scan_gfx_content(content: &str) -> HashMap<String, Sprite> {
        let (script, _errors) = parser::parse_script(content);
        let mut map = HashMap::new();
        find_sprites_in_entries(&script.entries, &script.source, "test.gfx", &mut map);
        map
    }

    #[test]
    fn test_sprite_type() {
        let map = scan_gfx_content(
            "spriteTypes = {\n\
            \tspriteType = {\n\
            \t\tname = \"GFX_test_sprite\"\n\
            \t\ttexturefile = \"gfx/test.dds\"\n\
            \t}\n\
            }",
        );
        assert!(map.contains_key("GFX_test_sprite"));
        assert_eq!(map["GFX_test_sprite"].texture_file, "gfx/test.dds");
    }

    #[test]
    fn test_frame_animated_sprite_type() {
        let map = scan_gfx_content(
            "spriteTypes = {\n\
            \tframeAnimatedSpriteType = {\n\
            \t\tname = \"GFX_SK_skulk\"\n\
            \t\ttexturefile = \"gfx/leaders/SK/SK_skulk_mind.dds\"\n\
            \t\tnoOfFrames = 20\n\
            \t\tanimation_rate_fps = 5\n\
            \t}\n\
            }",
        );
        assert!(
            map.contains_key("GFX_SK_skulk"),
            "frameAnimatedSpriteType should be detected"
        );
        assert_eq!(
            map["GFX_SK_skulk"].texture_file,
            "gfx/leaders/SK/SK_skulk_mind.dds"
        );
    }

    /// corneredTileSpriteType is a tiled-background sprite variant that still
    /// uses the same name + texturefile structure as spriteType.
    #[test]
    fn test_cornered_tile_sprite_type() {
        let map = scan_gfx_content(
            "spriteTypes = {\n\
            \tcorneredTileSpriteType = {\n\
            \t\tname = \"GFX_tiled_sprite\"\n\
            \t\ttexturefile = \"gfx/tiles/tiled.dds\"\n\
            \t}\n\
            }",
        );
        assert!(
            map.contains_key("GFX_tiled_sprite"),
            "corneredTileSpriteType should still be detected as a valid sprite"
        );
    }

    #[test]
    fn test_multiple_sprite_types() {
        let map = scan_gfx_content(
            "spriteTypes = {\n\
            \tspriteType = {\n\
            \t\tname = \"GFX_normal\"\n\
            \t\ttexturefile = \"gfx/normal.dds\"\n\
            \t}\n\
            \tframeAnimatedSpriteType = {\n\
            \t\tname = \"GFX_animated\"\n\
            \t\ttexturefile = \"gfx/animated.dds\"\n\
            \t}\n\
            \tcorneredTileSpriteType = {\n\
            \t\tname = \"GFX_tiled\"\n\
            \t\ttexturefile = \"gfx/tiled.dds\"\n\
            \t}\n\
            }",
        );
        assert!(map.contains_key("GFX_normal"));
        assert!(map.contains_key("GFX_animated"));
        assert!(map.contains_key("GFX_tiled"));
    }

    #[test]
    fn test_sprite_without_texturefile_not_added() {
        let map = scan_gfx_content(
            "spriteTypes = {\n\
            \tspriteType = {\n\
            \t\tname = \"GFX_incomplete\"\n\
            \t}\n\
            }",
        );
        assert!(!map.contains_key("GFX_incomplete"));
    }
}
