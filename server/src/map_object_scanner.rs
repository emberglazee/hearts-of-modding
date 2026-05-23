use std::fs;
use std::path::PathBuf;

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

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WeatherPosition {
    pub region_id: u32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub size: String,
    pub path: String,
    pub start_line: u32,
}

pub struct MapObjectScanResult {
    pub buildings: Vec<MapBuilding>,
    pub unitstacks: Vec<UnitStack>,
    pub weather_positions: Vec<WeatherPosition>,
}

pub fn scan_map_objects<F>(roots: &[PathBuf], filter: &F) -> MapObjectScanResult
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut buildings = Vec::new();
    let mut unitstacks = Vec::new();
    let mut weather_positions = Vec::new();

    for root in roots {
        let buildings_path = root.join("map/buildings.txt");
        if buildings_path.exists() && !filter(&buildings_path) && let Ok(content) = fs::read_to_string(&buildings_path) {
            for (line_idx, line) in content.lines().enumerate() {
                let parts: Vec<&str> = line.split(';').collect();
                if parts.len() >= 7 && let Ok(state_id) = parts[0].parse::<u32>() {
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

        let unitstacks_path = root.join("map/unitstacks.txt");
        if unitstacks_path.exists() && !filter(&unitstacks_path) && let Ok(content) = fs::read_to_string(&unitstacks_path) {
            for (line_idx, line) in content.lines().enumerate() {
                let parts: Vec<&str> = line.split(';').collect();
                if parts.len() >= 7 && let Ok(province_id) = parts[0].parse::<u32>() {
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

        let weather_path = root.join("map/weatherpositions.txt");
        if weather_path.exists() && !filter(&weather_path) && let Ok(content) = fs::read_to_string(&weather_path) {
            for (line_idx, line) in content.lines().enumerate() {
                let parts: Vec<&str> = line.split(';').collect();
                if parts.len() >= 5 && let Ok(region_id) = parts[0].parse::<u32>() {
                    let x = parts[1].parse::<f64>().unwrap_or(0.0);
                    let y = parts[2].parse::<f64>().unwrap_or(0.0);
                    let z = parts[3].parse::<f64>().unwrap_or(0.0);
                    let size = parts[4].to_string();

                    weather_positions.push(WeatherPosition {
                        region_id,
                        x,
                        y,
                        z,
                        size,
                        path: weather_path.to_string_lossy().to_string(),
                        start_line: line_idx as u32,
                    });
                }
            }
        }
    }

    MapObjectScanResult {
        buildings,
        unitstacks,
        weather_positions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_scan_map_objects() {
        let temp_dir = PathBuf::from("temp_map_test");
        let map_dir = temp_dir.join("map");
        fs::create_dir_all(&map_dir).unwrap();

        let buildings_content =
            "1;industrial_complex;100.0;10.0;200.0;0.5;123\n2;dockyard;300.0;0.0;400.0;1.0;456";
        fs::write(map_dir.join("buildings.txt"), buildings_content).unwrap();

        let unitstacks_content = "1000;1;150.0;5.0;250.0;0.0;0.0\n1001;2;350.0;0.0;450.0;1.5;1.0";
        fs::write(map_dir.join("unitstacks.txt"), unitstacks_content).unwrap();

        let weather_content = "1;10.0;20.0;30.0;large\n2;40.0;50.0;60.0;small";
        fs::write(map_dir.join("weatherpositions.txt"), weather_content).unwrap();

        let result = scan_map_objects(std::slice::from_ref(&temp_dir), &|_| false);

        assert_eq!(result.buildings.len(), 2);
        assert_eq!(result.buildings[0].state_id, 1);
        assert_eq!(result.buildings[0].building_id, "industrial_complex");
        assert_eq!(result.buildings[1].sea_province, 456);

        assert_eq!(result.unitstacks.len(), 2);
        assert_eq!(result.unitstacks[0].province_id, 1000);
        assert_eq!(result.unitstacks[1].stack_type, 2);

        assert_eq!(result.weather_positions.len(), 2);
        assert_eq!(result.weather_positions[0].region_id, 1);
        assert_eq!(result.weather_positions[0].size, "large");
        assert_eq!(result.weather_positions[1].region_id, 2);
        assert_eq!(result.weather_positions[1].size, "small");

        fs::remove_dir_all(temp_dir).unwrap();
    }
}
