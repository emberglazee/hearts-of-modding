use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SupplyNode {
    pub level: u32,
    pub province_id: u32,
    pub path: String,
    // Note: since it's not a script, we don't have ast::Range easily, but we can store line/col
    pub start_line: u32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Railway {
    pub level: u32,
    pub provinces: Vec<u32>,
    pub path: String,
    pub start_line: u32,
}

pub struct LogisticsScanResult {
    pub supply_nodes: Vec<SupplyNode>,
    pub railways: Vec<Railway>,
}

pub fn scan_logistics<F>(roots: &[PathBuf], filter: &F) -> LogisticsScanResult
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut supply_nodes = Vec::new();
    let mut railways = Vec::new();

    for root in roots {
        let supply_nodes_path = root.join("map/supply_nodes.txt");
        if supply_nodes_path.exists() && !filter(&supply_nodes_path) {
            if let Ok(content) = fs::read_to_string(&supply_nodes_path) {
                for (line_idx, line) in content.lines().enumerate() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let (Ok(level), Ok(province_id)) =
                            (parts[0].parse::<u32>(), parts[1].parse::<u32>())
                        {
                            supply_nodes.push(SupplyNode {
                                level,
                                province_id,
                                path: supply_nodes_path.to_string_lossy().to_string(),
                                start_line: line_idx as u32,
                            });
                        }
                    }
                }
            }
        }

        let railways_path = root.join("map/railways.txt");
        if railways_path.exists() && !filter(&railways_path) {
            if let Ok(content) = fs::read_to_string(&railways_path) {
                for (line_idx, line) in content.lines().enumerate() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let (Ok(level), Ok(num_provs)) =
                            (parts[0].parse::<u32>(), parts[1].parse::<usize>())
                        {
                            if parts.len() >= 2 + num_provs {
                                let mut provs = Vec::new();
                                for i in 0..num_provs {
                                    if let Ok(prov_id) = parts[2 + i].parse::<u32>() {
                                        provs.push(prov_id);
                                    }
                                }
                                railways.push(Railway {
                                    level,
                                    provinces: provs,
                                    path: railways_path.to_string_lossy().to_string(),
                                    start_line: line_idx as u32,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    LogisticsScanResult {
        supply_nodes,
        railways,
    }
}
