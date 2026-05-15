use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Province {
    pub id: u32,
    pub rgb: (u8, u8, u8),
    pub terrain: String,
    pub is_coastal: bool,
    pub prov_type: String, // land, sea, lake
    pub continent: u32,
}

pub fn scan_provinces<F>(roots: &[PathBuf], filter: &F) -> HashMap<u32, Province>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut provinces = HashMap::new();

    for root in roots {
        let map_config = crate::map_config::get_map_config(root);
        let definition_path = root.join(format!("map/{}", map_config.definitions));
        if definition_path.exists() && !filter(&definition_path) {
            if let Ok(content) = fs::read_to_string(&definition_path) {
                for line in content.lines() {
                    // HOI4 definition.csv format: ID;R;G;B;Terrain;IsCoastal;ProvinceType;Continent
                    let parts: Vec<&str> = line.split(';').collect();
                    if parts.len() >= 8 {
                        if let Ok(id) = parts[0].parse::<u32>() {
                            let r = parts[1].parse::<u8>().unwrap_or(0);
                            let g = parts[2].parse::<u8>().unwrap_or(0);
                            let b = parts[3].parse::<u8>().unwrap_or(0);
                            let terrain = parts[4].to_string();
                            let is_coastal = parts[5].to_lowercase() == "true";
                            let prov_type = parts[6].to_string();
                            let continent = parts[7].parse::<u32>().unwrap_or(0);

                            provinces.insert(
                                id,
                                Province {
                                    id,
                                    rgb: (r, g, b),
                                    terrain,
                                    is_coastal,
                                    prov_type,
                                    continent,
                                },
                            );
                        }
                    } else if let Some(id_str) = parts.first() {
                        // Fallback if missing fields
                        if let Ok(id) = id_str.parse::<u32>() {
                            provinces.insert(
                                id,
                                Province {
                                    id,
                                    rgb: (0, 0, 0),
                                    terrain: String::new(),
                                    is_coastal: false,
                                    prov_type: String::new(),
                                    continent: 0,
                                },
                            );
                        }
                    }
                }
            }
        }
    }

    provinces
}
