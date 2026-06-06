#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BoPSide {
    pub id: String,
    pub icon: Option<String>,
    pub ranges: Vec<BoPRange>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BoPRange {
    pub id: String,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BalanceOfPower {
    pub id: String,
    pub initial_value: Option<f64>,
    pub left_side: Option<String>,
    pub right_side: Option<String>,
    pub decision_category: Option<String>,
    pub sides: Vec<BoPSide>,
    pub ranges: Vec<BoPRange>,
    pub path: InternedStr,
    pub range: ast::Range,
}

pub fn scan_balance_of_powers<F>(roots: &[PathBuf], filter: &F) -> HashMap<String, BalanceOfPower>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut map = HashMap::new();

    for root in roots {
        crate::utils::fs_util::walk_and_parse_files(
            &root.join("common/bop"),
            &["txt"],
            filter,
            |path, content| {
                let (script, _) = parser::parse_script(&content);
                extract_balance_of_powers(
                    &script.entries,
                    &script.source,
                    &path.to_string_lossy(),
                    &mut map,
                );
            },
        );
    }

    map
}

pub fn scan_balance_of_power_files<F>(
    files: &[PathBuf],
    filter: &F,
) -> HashMap<String, BalanceOfPower>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut map = HashMap::new();

    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        extract_balance_of_powers(
            &script.entries,
            &script.source,
            &path.to_string_lossy(),
            &mut map,
        );
    });

    map
}

pub(crate) fn extract_balance_of_powers(
    entries: &[ast::Entry],
    source: &str,
    file_path: &str,
    map: &mut HashMap<String, BalanceOfPower>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let bop_id = ass.key_text(source).to_string();

            if let ast::Value::Block(bop_entries) = &ass.value.value {
                let mut initial_value = None;
                let mut left_side = None;
                let mut right_side = None;
                let mut decision_category = None;
                let mut sides = Vec::new();
                let mut ranges = Vec::new();

                for bop_entry in bop_entries {
                    if let ast::Entry::Assignment(bop_ass) = bop_entry {
                        let bop_key = bop_ass.key_text(source);
                        match bop_key.to_ascii_lowercase().as_str() {
                            "initial_value" => {
                                if let ast::Value::Number(val) = &bop_ass.value.value {
                                    initial_value = Some(*val);
                                }
                            }
                            "left_side" => {
                                if let Some(val) = bop_ass.value.value.as_str(source) {
                                    left_side = Some(val.to_string());
                                }
                            }
                            "right_side" => {
                                if let Some(val) = bop_ass.value.value.as_str(source) {
                                    right_side = Some(val.to_string());
                                }
                            }
                            "decision_category" => {
                                if let Some(val) = bop_ass.value.value.as_str(source) {
                                    decision_category = Some(val.to_string());
                                }
                            }
                            "range" => {
                                if let ast::Value::Block(range_entries) = &bop_ass.value.value {
                                    if let Some(range) = parse_range_block(range_entries, source) {
                                        ranges.push(range);
                                    }
                                }
                            }
                            "side" => {
                                if let ast::Value::Block(side_entries) = &bop_ass.value.value {
                                    if let Some(side) = parse_side_block(side_entries, source) {
                                        sides.push(side);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }

                map.insert(
                    bop_id.clone(),
                    BalanceOfPower {
                        id: bop_id,
                        initial_value,
                        left_side,
                        right_side,
                        decision_category,
                        sides,
                        ranges,
                        path: std::sync::Arc::from(file_path.to_string()),
                        range: ass.key_range.clone(),
                    },
                );
            }
        }
    }
}

fn parse_side_block(entries: &[ast::Entry], source: &str) -> Option<BoPSide> {
    let mut id = None;
    let mut icon = None;
    let mut ranges = Vec::new();

    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            match ass.key_text(source).to_ascii_lowercase().as_str() {
                "id" => {
                    if let Some(val) = ass.value.value.as_str(source) {
                        id = Some(val.to_string());
                    }
                }
                "icon" => {
                    if let Some(val) = ass.value.value.as_str(source) {
                        icon = Some(val.to_string());
                    }
                }
                "range" => {
                    if let ast::Value::Block(range_entries) = &ass.value.value {
                        if let Some(range) = parse_range_block(range_entries, source) {
                            ranges.push(range);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    id.map(|side_id| BoPSide {
        id: side_id,
        icon,
        ranges,
    })
}

fn parse_range_block(entries: &[ast::Entry], source: &str) -> Option<BoPRange> {
    let mut id = None;
    let mut min = None;
    let mut max = None;

    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            match ass.key_text(source).to_ascii_lowercase().as_str() {
                "id" => {
                    if let Some(val) = ass.value.value.as_str(source) {
                        id = Some(val.to_string());
                    }
                }
                "min" => {
                    if let ast::Value::Number(val) = &ass.value.value {
                        min = Some(*val);
                    }
                }
                "max" => {
                    if let ast::Value::Number(val) = &ass.value.value {
                        max = Some(*val);
                    }
                }
                _ => {}
            }
        }
    }

    if let (Some(range_id), Some(range_min), Some(range_max)) = (id, min, max) {
        Some(BoPRange {
            id: range_id,
            min: range_min,
            max: range_max,
        })
    } else {
        None
    }
}
