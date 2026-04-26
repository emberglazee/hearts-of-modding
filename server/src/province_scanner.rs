use std::collections::HashSet;
use std::path::PathBuf;
use std::fs;

pub fn scan_provinces(roots: &[PathBuf]) -> HashSet<u32> {
    let mut provinces = HashSet::new();

    for root in roots {
        let definition_path = root.join("map/definition.csv");
        if definition_path.exists() {
            if let Ok(content) = fs::read_to_string(&definition_path) {
                for line in content.lines() {
                    // HOI4 definition.csv format: ID;R;G;B;Terrain;IsCoastal;ProvinceType;Continent
                    let parts: Vec<&str> = line.split(';').collect();
                    if let Some(id_str) = parts.first() {
                        if let Ok(id) = id_str.parse::<u32>() {
                            provinces.insert(id);
                        }
                    }
                }
            }
        }
    }

    provinces
}
