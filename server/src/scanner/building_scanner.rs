#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Building {
    #[allow(dead_code)]
    pub name: String,
    pub max_level: Option<i32>,
    /// True when the definition gates placement behind `only_costal = yes`
    /// (vanilla spelling — the engine's own typo) or `only_coastal = yes`.
    /// Powers the Tier-0 coastal-building check (HOM2007).
    pub coastal_only: bool,
    #[allow(dead_code)]
    pub path: InternedStr,
    #[allow(dead_code)]
    pub range: ast::Range,
}

pub fn scan_buildings<F>(roots: &[PathBuf], filter: &F) -> HashMap<String, Building>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut buildings = HashMap::new();

    for root in roots {
        crate::utils::fs_util::walk_and_parse_files(
            &root.join("common/buildings"),
            &["txt"],
            filter,
            |path, content| {
                let (script, _) = parser::parse_script(&content);
                extract_buildings(&script.entries, &script.source, path, &mut buildings);
            },
        );
    }

    buildings
}

pub fn scan_building_files<F>(files: &[PathBuf], filter: &F) -> HashMap<String, Building>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut buildings = HashMap::new();

    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        extract_buildings(&script.entries, &script.source, path, &mut buildings);
    });

    buildings
}

pub(crate) fn extract_buildings(
    entries: &[ast::Entry],
    source: &str,
    path: &Path,
    map: &mut HashMap<String, Building>,
) {
    // First pass: find top-level `buildings = { ... }` block and extract from inside it
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            if ass.key_text(source).eq_ignore_ascii_case("buildings") {
                // Standard format: buildings = { infrastructure = { ... } }
                if let ast::Value::Block(inner_entries) = &ass.value.value {
                    extract_building_defs(inner_entries, source, path, map);
                }
            } else {
                // bare format: infrastructure = { ... } or single-building file
                extract_building_def(entry, source, path, map);
            }
        }
    }
}

/// Extract individual building definitions from entries that are inside the
/// `buildings = { ... }` block.
fn extract_building_defs(
    entries: &[ast::Entry],
    source: &str,
    path: &Path,
    map: &mut HashMap<String, Building>,
) {
    for entry in entries {
        extract_building_def(entry, source, path, map);
    }
}

/// Extract a single building definition from an assignment entry.
fn extract_building_def(
    entry: &ast::Entry,
    source: &str,
    path: &Path,
    map: &mut HashMap<String, Building>,
) {
    if let ast::Entry::Assignment(ass) = entry {
        let building_name = ass.key_text(source).to_string();
        let mut max_level = None;
        let mut coastal_only = false;

        // Extract max_level from building definition
        if let ast::Value::Block(building_entries) = &ass.value.value {
            for building_entry in building_entries {
                if let ast::Entry::Assignment(building_ass) = building_entry {
                    let key = building_ass.key_text(source);
                    if key.eq_ignore_ascii_case("only_costal")
                        || key.eq_ignore_ascii_case("only_coastal")
                    {
                        // `yes` parses as Boolean(true); accept a quoted/string
                        // "yes" too so mods that quote the value still match.
                        coastal_only = match &building_ass.value.value {
                            ast::Value::Boolean(b) => *b,
                            ast::Value::String(s) => s.resolve(source).eq_ignore_ascii_case("yes"),
                            ast::Value::QuotedString(s) => s.eq_ignore_ascii_case("yes"),
                            _ => false,
                        };
                    } else if key.eq_ignore_ascii_case("max_level")
                        && let ast::Value::Number(level) = &building_ass.value.value
                    {
                        max_level = Some(*level as i32);
                    } else if let Some(s) = building_ass.value.value.as_str(source) {
                        max_level = s.parse::<i32>().ok();
                    }
                }
            }
        }

        map.insert(
            building_name.clone(),
            Building {
                name: building_name,
                max_level,
                coastal_only,
                path: std::sync::Arc::from(path.to_string_lossy().as_ref()),
                range: ass.key_range.clone(),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parser;

    /// `only_costal` is the engine's own misspelling (verified in vanilla
    /// `common/buildings/00_buildings.txt`); `only_coastal` is accepted too.
    /// `yes` parses as `Boolean(true)`, not a string — match both.
    #[test]
    fn test_coastal_only_spellings_and_value_shapes() {
        let (script, _) = parser::parse_script(
            "buildings = { naval_base = { max_level = 6 only_costal = yes } \
               dockyard = { only_coastal = \"yes\" } \
               arms_factory = { max_level = 3 } \
               bunker = { only_costal = no } }",
        );
        let mut map = HashMap::new();
        extract_buildings(
            &script.entries,
            &script.source,
            Path::new("common/buildings/test.txt"),
            &mut map,
        );
        assert!(map["naval_base"].coastal_only, "vanilla spelling + boolean");
        assert!(map["dockyard"].coastal_only, "fixed spelling + string");
        assert!(!map["arms_factory"].coastal_only, "absent flag");
        assert!(!map["bunker"].coastal_only, "explicit no");
        assert_eq!(map["naval_base"].max_level, Some(6));
    }
}
