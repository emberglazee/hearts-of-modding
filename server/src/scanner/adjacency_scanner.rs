#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Adjacency {
    pub start_prov: u32,
    pub end_prov: u32,
    pub adj_type: String,
    pub through_prov: i32,
    pub rule_name: String,
    pub path: InternedStr,
    pub start_line: u32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AdjacencyRule {
    pub name: String,
    pub required_provinces: Vec<u32>,
    pub icon: Option<u32>,
    pub path: InternedStr,
    pub range: ast::Range,
}

#[allow(dead_code)]
pub struct AdjacencyScanResult {
    pub adjacencies: Vec<Adjacency>,
    pub rules: HashMap<String, AdjacencyRule>,
}

pub fn scan_adjacencies<F>(roots: &[PathBuf], filter: &F) -> AdjacencyScanResult
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut adjacencies = Vec::new();
    let mut rules = HashMap::new();

    for root in roots {
        let map_config = crate::utils::map_config::get_map_config(root);
        let adj_csv_path = root.join(format!("map/{}", map_config.adjacencies));
        if adj_csv_path.exists()
            && !filter(&adj_csv_path)
            && let Ok(content) = fs::read_to_string(&adj_csv_path)
        {
            for (line_idx, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                let parts: Vec<&str> = trimmed.split(';').collect();
                if parts.len() >= 9
                    && let (Ok(start_prov), Ok(end_prov)) =
                        (parts[0].parse::<u32>(), parts[1].parse::<u32>())
                {
                    let adj_type = parts[2].to_string();
                    let through_prov = parts[3].parse::<i32>().unwrap_or(-1);
                    let rule_name = parts[8].to_string();

                    adjacencies.push(Adjacency {
                        start_prov,
                        end_prov,
                        adj_type,
                        through_prov,
                        rule_name,
                        path: std::sync::Arc::from(adj_csv_path.to_string_lossy().as_ref()),
                        start_line: line_idx as u32,
                    });
                }
            }
        }

        let rules_path = root.join("map/adjacency_rules.txt");
        if rules_path.exists()
            && !filter(&rules_path)
            && let Ok(content) = fs::read_to_string(&rules_path)
        {
            let (script, _) = parser::parse_script(&content);
            for entry in script.entries {
                if let ast::Entry::Assignment(ass) = entry
                    && ass
                        .key_text(&script.source)
                        .eq_ignore_ascii_case("adjacency_rule")
                    && let ast::Value::Block(rule_entries) = &ass.value.value
                {
                    let mut name = None;
                    let mut required_provinces = Vec::new();
                    let mut icon = None;

                    for rule_entry in rule_entries {
                        if let ast::Entry::Assignment(r_ass) = rule_entry {
                            let r_key = r_ass.key_text(&script.source);
                            if r_key.eq_ignore_ascii_case("name") {
                                if let Some(s) = r_ass.value.value.as_str(&script.source) {
                                    name = Some(s.to_string());
                                }
                            } else if r_key.eq_ignore_ascii_case("required_provinces") {
                                if let ast::Value::Block(prov_entries) = &r_ass.value.value {
                                    for p_entry in prov_entries {
                                        if let ast::Entry::Value(p_val) = p_entry
                                            && let ast::Value::Number(n) = &p_val.value
                                        {
                                            required_provinces.push(*n as u32);
                                        }
                                    }
                                }
                            } else if r_key.eq_ignore_ascii_case("icon") {
                                if let ast::Value::Number(n) = &r_ass.value.value {
                                    icon = Some(*n as u32);
                                }
                            }
                        }
                    }
                    if let Some(n) = name {
                        rules.insert(
                            n.clone(),
                            AdjacencyRule {
                                name: n,
                                required_provinces,
                                icon,
                                path: std::sync::Arc::from(rules_path.to_string_lossy().as_ref()),
                                range: ass.key_range.clone(),
                            },
                        );
                    }
                }
            }
        }
    }

    AdjacencyScanResult { adjacencies, rules }
}

/// Scan a pre-determined list of adjacency files.
/// Determines the parsing strategy by filename:
/// - The adjacencies CSV file (configurable name, matched by `map_config.adjacencies`) → `Adjacency` entries
/// - `adjacency_rules.txt` → `AdjacencyRule` entries
///   Note: Because the adjacencies CSV filename is mod-configurable, this variant uses the
///   file's extension (`.csv`) and adjacency rules by fixed filename (`adjacency_rules.txt`).
pub fn scan_adjacency_files<F>(files: &[PathBuf], filter: &F) -> AdjacencyScanResult
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut adjacencies = Vec::new();
    let mut rules = HashMap::new();

    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if fname.eq_ignore_ascii_case("adjacency_rules.txt") {
            let (script, _) = parser::parse_script(&content);
            for entry in script.entries {
                if let ast::Entry::Assignment(ass) = entry
                    && ass
                        .key_text(&script.source)
                        .eq_ignore_ascii_case("adjacency_rule")
                    && let ast::Value::Block(rule_entries) = &ass.value.value
                {
                    let mut name = None;
                    let mut required_provinces = Vec::new();
                    let mut icon = None;

                    for rule_entry in rule_entries {
                        if let ast::Entry::Assignment(r_ass) = rule_entry {
                            let r_key = r_ass.key_text(&script.source);
                            if r_key.eq_ignore_ascii_case("name") {
                                if let Some(s) = r_ass.value.value.as_str(&script.source) {
                                    name = Some(s.to_string());
                                }
                            } else if r_key.eq_ignore_ascii_case("required_provinces") {
                                if let ast::Value::Block(prov_entries) = &r_ass.value.value {
                                    for p_entry in prov_entries {
                                        if let ast::Entry::Value(p_val) = p_entry
                                            && let ast::Value::Number(n) = &p_val.value
                                        {
                                            required_provinces.push(*n as u32);
                                        }
                                    }
                                }
                            } else if r_key.eq_ignore_ascii_case("icon") {
                                if let ast::Value::Number(n) = &r_ass.value.value {
                                    icon = Some(*n as u32);
                                }
                            }
                        }
                    }
                    if let Some(n) = name {
                        rules.insert(
                            n.clone(),
                            AdjacencyRule {
                                name: n,
                                required_provinces,
                                icon,
                                path: std::sync::Arc::from(path.to_string_lossy().as_ref()),
                                range: ass.key_range.clone(),
                            },
                        );
                    }
                }
            }
        } else if fname.ends_with(".csv") {
            for (line_idx, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                let parts: Vec<&str> = trimmed.split(';').collect();
                if parts.len() >= 9
                    && let (Ok(start_prov), Ok(end_prov)) =
                        (parts[0].parse::<u32>(), parts[1].parse::<u32>())
                {
                    let adj_type = parts[2].to_string();
                    let through_prov = parts[3].parse::<i32>().unwrap_or(-1);
                    let rule_name = parts[8].to_string();

                    adjacencies.push(Adjacency {
                        start_prov,
                        end_prov,
                        adj_type,
                        through_prov,
                        rule_name,
                        path: std::sync::Arc::from(path.to_string_lossy().as_ref()),
                        start_line: line_idx as u32,
                    });
                }
            }
        }
    });

    AdjacencyScanResult { adjacencies, rules }
}
