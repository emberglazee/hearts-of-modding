use std::path::PathBuf;
use std::fs;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MapBuilding {
    pub state_id: u32,
    pub building_id: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub rotation: f64,
    pub sea_province: u32,
    pub path: String,
    pub start_line: u32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UnitStack {
    pub province_id: u32,
    pub stack_type: u32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub rotation: f64,
    pub offset: f64,
    pub path: String,
    pub start_line: u32,
}

pub struct MapObjectScanResult {
    pub buildings: Vec<MapBuilding>,
    pub unitstacks: Vec<UnitStack>,
}

pub fn scan_map_objects<F>(roots: &[PathBuf], filter: &F) -> MapObjectScanResult 
where F: Fn(&std::path::Path) -> bool {
    let mut buildings = Vec::new();
    let mut unitstacks = Vec::new();

    for root in roots {
        let buildings_path = root.join("map/buildings.txt");
        if buildings_path.exists() && !filter(&buildings_path) {
            if let Ok(content) = fs::read_to_string(&buildings_path) {
                for (line_idx, line) in content.lines().enumerate() {
                    let parts: Vec<&str> = line.split(';').collect();
                    if parts.len() >= 7 {
                        if let Ok(state_id) = parts[0].parse::<u32>() {
                            let building_id = parts[1].to_string();
                            let x = parts[2].parse::<f64>().unwrap_or(0.0);
                            let y = parts[3].parse::<f64>().unwrap_or(0.0);
                            let z = parts[4].parse::<f64>().unwrap_or(0.0);
                            let rotation = parts[5].parse::<f64>().unwrap_or(0.0);
                            let sea_province = parts[6].parse::<u32>().unwrap_or(0);
                            
                            buildings.push(MapBuilding {
                                state_id,
                                building_id,
                                x,
                                y,
                                z,
                                rotation,
                                sea_province,
                                path: buildings_path.to_string_lossy().to_string(),
                                start_line: line_idx as u32,
                            });
                        }
                    }
                }
            }
        }

        let unitstacks_path = root.join("map/unitstacks.txt");
        if unitstacks_path.exists() && !filter(&unitstacks_path) {
            if let Ok(content) = fs::read_to_string(&unitstacks_path) {
                for (line_idx, line) in content.lines().enumerate() {
                    let parts: Vec<&str> = line.split(';').collect();
                    if parts.len() >= 7 {
                        if let Ok(province_id) = parts[0].parse::<u32>() {
                            let stack_type = parts[1].parse::<u32>().unwrap_or(0);
                            let x = parts[2].parse::<f64>().unwrap_or(0.0);
                            let y = parts[3].parse::<f64>().unwrap_or(0.0);
                            let z = parts[4].parse::<f64>().unwrap_or(0.0);
                            let rotation = parts[5].parse::<f64>().unwrap_or(0.0);
                            let offset = parts[6].parse::<f64>().unwrap_or(0.0);

                            unitstacks.push(UnitStack {
                                province_id,
                                stack_type,
                                x,
                                y,
                                z,
                                rotation,
                                offset,
                                path: unitstacks_path.to_string_lossy().to_string(),
                                start_line: line_idx as u32,
                            });
                        }
                    }
                }
            }
        }
    }

    MapObjectScanResult {
        buildings,
        unitstacks,
    }
}
