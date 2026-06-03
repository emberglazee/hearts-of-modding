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
                extract_terrain_categories(&script.entries, &script.source, path, &mut terrains);
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
    source: &str,
    path: &Path,
    map: &mut HashMap<String, TerrainCategory>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            if ass.key_text(source).eq_ignore_ascii_case("categories") {
                if let ast::Value::Block(cat_entries) = &ass.value.value {
                    for cat_entry in cat_entries {
                        if let ast::Entry::Assignment(cat_ass) = cat_entry {
                            let name = cat_ass.key_text(source).to_string();
                            let mut is_naval = false;
                            let mut is_water = false;

                            if let ast::Value::Block(props) = &cat_ass.value.value {
                                for prop in props {
                                    if let ast::Entry::Assignment(prop_ass) = prop {
                                        let pkey = prop_ass.key_text(source);
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

    /// Representative terrain content matching the structure of
    /// vanilla HOI4's common/terrain/00_terrain.txt.
    const MOCK_TERRAIN_CONTENT: &str = r#"categories = {
    unknown = {
        color = { 255 0 0 }
    }
    ocean = {
        naval_terrain = yes
        is_water = yes
    }
    lakes = {
        is_water = yes
    }
    forest = {
    }
    hills = {
    }
    mountain = {
    }
    plains = {
    }
    urban = {
    }
    jungle = {
    }
    marsh = {
    }
    desert = {
    }
    water_fjords = {
        naval_terrain = yes
        is_water = yes
    }
    water_shallow_sea = {
        naval_terrain = yes
        is_water = yes
    }
    water_deep_ocean = {
        naval_terrain = yes
        is_water = yes
    }
}"#;

    #[test]
    fn test_extract_terrain_categories_from_mock_data() {
        let (script, _) = parser::parse_script(MOCK_TERRAIN_CONTENT);

        let mut terrains = HashMap::new();
        let path = Path::new("common/terrain/00_terrain.txt");
        extract_terrain_categories(&script.entries, &script.source, path, &mut terrains);

        // We should find all terrain categories from the mock
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
    fn test_extract_terrain_categories_empty_script() {
        // Test with an empty script (simulating a file with no categories)
        let content = "";
        let (script, _) = parser::parse_script(content);
        let mut terrains = HashMap::new();
        let path = Path::new("common/terrain/empty.txt");
        extract_terrain_categories(&script.entries, &script.source, path, &mut terrains);
        assert!(terrains.is_empty());
    }

    #[test]
    fn test_province_terrains_match_terrain_categories() {
        // Integration test with mock data: cross-check all province terrain
        // values against the known terrain categories.
        //
        // Uses inline mock content for both the terrain file and definition.csv
        // so the test is reproducible on any machine (including CI runners).

        // 1. Load terrain categories from mock content
        let (terrain_script, _) = parser::parse_script(MOCK_TERRAIN_CONTENT);
        let mut terrains = HashMap::new();
        extract_terrain_categories(
            &terrain_script.entries,
            &terrain_script.source,
            Path::new("mock/00_terrain.txt"),
            &mut terrains,
        );
        let terrain_names: std::collections::HashSet<String> = terrains.keys().cloned().collect();

        // 2. Mock definition.csv with provinces using various terrain types
        let mock_def_csv = "\
0;0;0;0;land;false;unknown;0
1;230;81;119;lake;false;lakes;7
2;0;0;55;land;false;forest;1
3;0;0;205;land;false;forest;1
4;0;0;232;sea;true;ocean;0
5;0;2;240;sea;false;ocean;0
6;0;3;20;land;false;hills;6
7;0;3;170;land;false;mountain;1
8;0;3;225;land;false;plains;1
9;0;4;248;sea;false;water_fjords;0
10;0;6;85;land;false;forest;1
11;0;6;135;land;false;urban;1
12;0;6;190;land;false;jungle;6
13;0;8;244;sea;false;ocean;0
14;0;9;155;land;true;plains;1
15;0;9;200;land;false;marsh;1
16;0;10;232;sea;true;desert;0
17;0;12;15;land;false;water_shallow_sea;1
18;0;12;165;land;false;water_deep_ocean;1
";

        let mut unknown_terrains: Vec<(u32, String)> = Vec::new();

        for line in mock_def_csv.lines() {
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

        // Every province terrain in the mock data should match a known category
        assert!(
            unknown_terrains.is_empty(),
            "Province terrains not found in terrain categories: {:?}",
            unknown_terrains,
        );
    }
}
