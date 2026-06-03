use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TerrainCategory {
    pub name: String,
    /// Whether this terrain has `naval_terrain = yes` (usable in strategic regions)
    pub is_naval: bool,
    /// Whether this terrain has `is_water = yes`
    pub is_water: bool,
    /// File path where this terrain is defined
    pub path: InternedStr,
    /// Range of the terrain key in the source file (for goto-definition)
    pub range: ast::Range,
}

pub fn scan_terrains<F>(roots: &[PathBuf], filter: &F) -> HashMap<String, TerrainCategory>
where
    F: Fn(&Path) -> bool,
{
    let mut terrains = HashMap::new();

    for root in roots {
        crate::utils::fs_util::walk_and_parse_files(
            &root.join("common/terrain"),
            &["txt"],
            filter,
            |path, content| {
                let (script, _) = parser::parse_script(&content);
                extract_terrain_categories(&script.entries, path, &mut terrains);
            },
        );
    }

    terrains
}

/// Extract terrain category definitions from `categories = { ... }` blocks.
///
/// HOI4 `common/terrain/*.txt` files have this structure:
/// ```hoi4
/// categories = {
///     ocean = {
///         naval_terrain = yes
///         color = { 40 83 176 }
///         movement_cost = 1.0
///         is_water = yes
///         ...
///     }
///     forest = {
///         color = { 89 199 85 }
///         ...
///     }
/// }
/// ```
pub(crate) fn extract_terrain_categories(
    entries: &[ast::Entry],
    path: &Path,
    map: &mut HashMap<String, TerrainCategory>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            if ass.key.eq_ignore_ascii_case("categories") {
                if let ast::Value::Block(cat_entries) = &ass.value.value {
                    for cat_entry in cat_entries {
                        if let ast::Entry::Assignment(cat_ass) = cat_entry {
                            let name = cat_ass.key.clone();
                            let mut is_naval = false;
                            let mut is_water = false;

                            if let ast::Value::Block(props) = &cat_ass.value.value {
                                for prop in props {
                                    if let ast::Entry::Assignment(prop_ass) = prop {
                                        let pkey = prop_ass.key.as_str();
                                        if pkey.eq_ignore_ascii_case("naval_terrain") {
                                            if let ast::Value::Boolean(b) = &prop_ass.value.value {
                                                is_naval = *b;
                                            }
                                        } else if pkey.eq_ignore_ascii_case("is_water") {
                                            if let ast::Value::Boolean(b) = &prop_ass.value.value {
                                                is_water = *b;
                                            }
                                        }
                                    }
                                }
                            }

                            map.insert(
                                name.clone(),
                                TerrainCategory {
                                    name,
                                    is_naval,
                                    is_water,
                                    path: std::sync::Arc::from(path.to_string_lossy().as_ref()),
                                    range: cat_ass.key_range.clone(),
                                },
                            );
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_terrain_categories_from_vanilla() {
        // Load the vanilla game's terrain file
        let path_str = "/home/embi/.steam/steam/steamapps/common/Hearts of Iron IV/common/terrain/00_terrain.txt";
        let content =
            std::fs::read_to_string(path_str).expect("Failed to read vanilla terrain file");
        let (script, _) = parser::parse_script(&content);

        let mut terrains = HashMap::new();
        let path = Path::new(path_str);
        extract_terrain_categories(&script.entries, path, &mut terrains);

        // We should find all terrain categories from vanilla
        assert!(
            !terrains.is_empty(),
            "Should have parsed at least some terrain categories"
        );

        // Check known land terrains exist
        assert!(terrains.contains_key("unknown"), "Missing unknown terrain");
        assert!(terrains.contains_key("ocean"), "Missing ocean terrain");
        assert!(terrains.contains_key("forest"), "Missing forest terrain");
        assert!(terrains.contains_key("hills"), "Missing hills terrain");
        assert!(
            terrains.contains_key("mountain"),
            "Missing mountain terrain"
        );
        assert!(terrains.contains_key("plains"), "Missing plains terrain");
        assert!(terrains.contains_key("urban"), "Missing urban terrain");
        assert!(terrains.contains_key("jungle"), "Missing jungle terrain");
        assert!(terrains.contains_key("marsh"), "Missing marsh terrain");
        assert!(terrains.contains_key("desert"), "Missing desert terrain");
        assert!(terrains.contains_key("lakes"), "Missing lakes terrain");

        // Check naval terrain flags
        assert!(
            terrains.get("ocean").unwrap().is_naval,
            "ocean should be naval"
        );
        assert!(
            terrains.get("lakes").unwrap().is_water,
            "lakes should be water"
        );
        assert!(
            !terrains.get("lakes").unwrap().is_naval,
            "lakes should NOT be naval"
        );
        assert!(
            !terrains.get("forest").unwrap().is_naval,
            "forest should NOT be naval"
        );

        // Check water flags
        assert!(
            terrains.get("ocean").unwrap().is_water,
            "ocean should be water"
        );
        assert!(
            terrains.get("lakes").unwrap().is_water,
            "lakes should be water"
        );
        assert!(
            !terrains.get("plains").unwrap().is_water,
            "plains should NOT be water"
        );

        // Check specific naval terrain categories
        assert!(
            terrains.contains_key("water_fjords"),
            "Missing water_fjords"
        );
        assert!(
            terrains.contains_key("water_shallow_sea"),
            "Missing water_shallow_sea"
        );
        assert!(
            terrains.contains_key("water_deep_ocean"),
            "Missing water_deep_ocean"
        );

        // Verify naval terrains
        assert!(
            terrains.get("water_fjords").unwrap().is_naval,
            "water_fjords should be naval"
        );
        assert!(
            terrains.get("water_shallow_sea").unwrap().is_naval,
            "water_shallow_sea should be naval"
        );
        assert!(
            terrains.get("water_deep_ocean").unwrap().is_naval,
            "water_deep_ocean should be naval"
        );
    }

    #[test]
    fn test_extract_terrain_categories_unknown_file() {
        // Test with a file that doesn't exist
        let path_str = "/tmp/nonexistent_terrain.txt";
        let content = std::fs::read_to_string(path_str);
        if let Ok(content) = content {
            let (script, _) = parser::parse_script(&content);
            let mut terrains = HashMap::new();
            let path = Path::new(path_str);
            extract_terrain_categories(&script.entries, path, &mut terrains);
            assert!(terrains.is_empty());
        }
    }

    #[test]
    fn test_province_terrains_match_terrain_categories() {
        // Integration test: cross-check all province terrain values from
        // the vanilla definition.csv against the known terrain categories
        // parsed from common/terrain/00_terrain.txt.
        let game_root = PathBuf::from("/home/embi/.steam/steam/steamapps/common/Hearts of Iron IV");

        // 1. Load terrain categories
        let terrain_path = game_root.join("common/terrain/00_terrain.txt");
        let terrain_content =
            std::fs::read_to_string(&terrain_path).expect("Failed to read vanilla terrain file");
        let (terrain_script, _) = parser::parse_script(&terrain_content);
        let mut terrains = HashMap::new();
        extract_terrain_categories(&terrain_script.entries, &terrain_path, &mut terrains);
        let terrain_names: std::collections::HashSet<String> = terrains.keys().cloned().collect();

        // 2. Load definition.csv provinces manually
        let def_path = game_root.join("map/definition.csv");
        let def_content =
            std::fs::read_to_string(&def_path).expect("Failed to read definition.csv");
        let mut unknown_terrains: Vec<(u32, String)> = Vec::new();

        for line in def_content.lines() {
            let parts: Vec<&str> = line.split(';').collect();
            if parts.len() >= 8 {
                if let Ok(id) = parts[0].parse::<u32>() {
                    // parts[6] = terrain name (see province_scanner.rs comment)
                    let province_terrain = parts[6].trim().to_lowercase();
                    if !province_terrain.is_empty() && !terrain_names.contains(&province_terrain) {
                        unknown_terrains.push((id, province_terrain));
                    }
                }
            }
        }

        // Every province terrain in the vanilla game should match a known category
        assert!(
            unknown_terrains.is_empty(),
            "Province terrains not found in terrain categories: {:?}",
            unknown_terrains,
        );
    }
}
