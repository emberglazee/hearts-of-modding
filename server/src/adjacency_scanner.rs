use crate::parser;
use crate::ast;
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Adjacency {
    pub start_prov: u32,
    pub end_prov: u32,
    pub adj_type: String,
    pub through_prov: i32,
    pub rule_name: String,
    pub path: String,
    pub start_line: u32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
#[allow(dead_code)]
pub struct AdjacencyRule {
    pub name: String,
    pub path: String,
    pub range: ast::Range,
}

#[allow(dead_code)]
pub struct AdjacencyScanResult {
    pub adjacencies: Vec<Adjacency>,
    pub rules: HashMap<String, AdjacencyRule>,
}

pub fn scan_adjacencies<F>(roots: &[PathBuf], filter: &F) -> AdjacencyScanResult 
where F: Fn(&std::path::Path) -> bool {
    let mut adjacencies = Vec::new();
    let mut rules = HashMap::new();

    for root in roots {
        let map_config = crate::map_config::get_map_config(root);
        let adj_csv_path = root.join(format!("map/{}", map_config.adjacencies));
        if adj_csv_path.exists() && !filter(&adj_csv_path) {
            if let Ok(content) = fs::read_to_string(&adj_csv_path) {
                for (line_idx, line) in content.lines().enumerate() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        continue;
                    }
                    let parts: Vec<&str> = trimmed.split(';').collect();
                    if parts.len() >= 9 {
                        if let (Ok(start_prov), Ok(end_prov)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                            let adj_type = parts[2].to_string();
                            let through_prov = parts[3].parse::<i32>().unwrap_or(-1);
                            let rule_name = parts[8].to_string();

                            adjacencies.push(Adjacency {
                                start_prov,
                                end_prov,
                                adj_type,
                                through_prov,
                                rule_name,
                                path: adj_csv_path.to_string_lossy().to_string(),
                                start_line: line_idx as u32,
                            });
                        }
                    }
                }
            }
        }

        let rules_path = root.join("map/adjacency_rules.txt");
        if rules_path.exists() && !filter(&rules_path) {
            if let Ok(content) = fs::read_to_string(&rules_path) {
                { let (script, _) = parser::parse_script(&content);
                    for entry in script.entries {
                        if let ast::Entry::Assignment(ass) = entry {
                            if ass.key.to_lowercase() == "adjacency_rule" {
                                if let ast::Value::Block(rule_entries) = &ass.value.value {
                                    let mut name = None;
                                    for rule_entry in rule_entries {
                                        if let ast::Entry::Assignment(r_ass) = rule_entry {
                                            if r_ass.key.to_lowercase() == "name" {
                                                if let ast::Value::String(s) = &r_ass.value.value {
                                                    name = Some(s.clone());
                                                }
                                            }
                                        }
                                    }
                                    if let Some(n) = name {
                                        rules.insert(n.clone(), AdjacencyRule {
                                            name: n,
                                            path: rules_path.to_string_lossy().to_string(),
                                            range: ass.key_range.clone(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    AdjacencyScanResult {
        adjacencies,
        rules,
    }
}
