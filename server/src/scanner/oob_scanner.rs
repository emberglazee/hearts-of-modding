#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::path::PathBuf;

/// A division template defined in an OOB file.
#[derive(Debug, Clone)]
pub struct OobDivisionTemplate {
    /// The display name of the template (from `name = "..."`)
    pub name: String,
    /// Regiments in this template (sub-unit types like infantry, artillery_brigade, etc.)
    pub regiments: Vec<String>,
    /// Support companies in this template
    pub support: Vec<String>,
    pub path: InternedStr,
    pub range: ast::Range,
}

/// A division placement in an OOB file.
#[derive(Debug, Clone)]
pub struct OobDivision {
    pub name: String,
    pub template: String,
    pub location: u32,
    pub path: InternedStr,
    pub range: ast::Range,
}

/// A fleet defined in an OOB file.
#[derive(Debug, Clone)]
pub struct OobFleet {
    pub name: String,
    pub naval_base: Option<u32>,
    pub path: InternedStr,
    pub range: ast::Range,
}

/// A ship within a task force.
#[derive(Debug, Clone)]
pub struct OobShip {
    pub name: String,
    pub definition: String,
    pub path: InternedStr,
    pub range: ast::Range,
}

pub struct OobScanResult {
    pub division_templates: HashMap<String, OobDivisionTemplate>,
    pub divisions: HashMap<String, OobDivision>,
    pub fleets: HashMap<String, OobFleet>,
}

pub fn scan_oobs<F>(roots: &[PathBuf], filter: &F) -> OobScanResult
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut result = OobScanResult {
        division_templates: HashMap::new(),
        divisions: HashMap::new(),
        fleets: HashMap::new(),
    };

    for root in roots {
        crate::utils::fs_util::walk_and_parse_files(
            &root.join("history/units"),
            &["txt"],
            filter,
            |path, content| {
                let (script, _) = parser::parse_script(&content);
                extract_oob_entities(
                    &script.entries,
                    &script.source,
                    &path.to_string_lossy(),
                    &mut result,
                );
            },
        );
    }

    result
}

pub fn scan_oob_files<F>(files: &[PathBuf], filter: &F) -> OobScanResult
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut result = OobScanResult {
        division_templates: HashMap::new(),
        divisions: HashMap::new(),
        fleets: HashMap::new(),
    };

    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        extract_oob_entities(
            &script.entries,
            &script.source,
            &path.to_string_lossy(),
            &mut result,
        );
    });

    result
}

pub(crate) fn extract_oob_entities(
    entries: &[ast::Entry],
    source: &str,
    file_path: &str,
    result: &mut OobScanResult,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let key_lower = ass.key_text(source).to_ascii_lowercase();

            match key_lower.as_str() {
                "division_template" => {
                    if let ast::Value::Block(inner) = &ass.value.value {
                        extract_division_template(
                            inner,
                            source,
                            file_path,
                            &mut result.division_templates,
                            ass.key_range.clone(),
                        );
                    }
                }
                "units" => {
                    if let ast::Value::Block(inner) = &ass.value.value {
                        extract_units(inner, source, file_path, result);
                    }
                }
                "air_wings" => {
                    // Air wings are state-keyed blocks — we extract them for completeness
                    // but don't store by a global key since state IDs aren't unique names
                }
                "instant_effect" => {
                    // Effect blocks are not scanned for entities
                }
                _ => {}
            }
        }
    }
}

fn extract_division_template(
    entries: &[ast::Entry],
    source: &str,
    file_path: &str,
    map: &mut HashMap<String, OobDivisionTemplate>,
    range: ast::Range,
) {
    // A division_template block can be a direct block:
    //   division_template = { name = "..." ... }
    // or a named block with inner entries.
    // The parser handles it as a Value::Block containing entries.

    let mut name = String::new();
    let mut regiments = Vec::new();
    let mut support = Vec::new();

    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let key_lower = ass.key_text(source).to_ascii_lowercase();
            match key_lower.as_str() {
                "name" => {
                    if let Some(val) = ass.value.value.as_str(source) {
                        name = val.to_string();
                    }
                }
                "regiments" => {
                    if let ast::Value::Block(reg_entries) = &ass.value.value {
                        for reg_entry in reg_entries {
                            if let ast::Entry::Assignment(reg_ass) = reg_entry {
                                let reg_type = reg_ass.key_text(source).to_string();
                                regiments.push(reg_type);
                            }
                        }
                    }
                }
                "support" => {
                    if let ast::Value::Block(sup_entries) = &ass.value.value {
                        for sup_entry in sup_entries {
                            if let ast::Entry::Assignment(sup_ass) = sup_entry {
                                let sup_type = sup_ass.key_text(source).to_string();
                                support.push(sup_type);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if !name.is_empty() {
        map.insert(
            name.clone(),
            OobDivisionTemplate {
                name,
                regiments,
                support,
                path: std::sync::Arc::from(file_path.to_string()),
                range,
            },
        );
    }
}

fn extract_units(
    entries: &[ast::Entry],
    source: &str,
    file_path: &str,
    result: &mut OobScanResult,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let key_lower = ass.key_text(source).to_ascii_lowercase();

            match key_lower.as_str() {
                "division" => {
                    if let ast::Value::Block(div_entries) = &ass.value.value {
                        extract_division(div_entries, source, file_path, &mut result.divisions);
                    }
                }
                "fleet" => {
                    if let ast::Value::Block(fleet_entries) = &ass.value.value {
                        extract_fleet(
                            fleet_entries,
                            source,
                            file_path,
                            &mut result.fleets,
                            ass.key_range.clone(),
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

fn extract_division(
    entries: &[ast::Entry],
    source: &str,
    file_path: &str,
    map: &mut HashMap<String, OobDivision>,
) {
    let mut name = String::new();
    let mut template = String::new();
    let mut location = 0u32;

    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let key_lower = ass.key_text(source).to_ascii_lowercase();
            match key_lower.as_str() {
                "name" => {
                    if let Some(val) = ass.value.value.as_str(source) {
                        name = val.to_string();
                    }
                }
                "division_name" => {
                    // division_name block contains is_name_ordered and name_order
                    // We don't extract ordered names into the map key
                }
                "division_template" => {
                    if let Some(val) = ass.value.value.as_str(source) {
                        template = val.to_string();
                    }
                }
                "location" => {
                    if let ast::Value::Number(val) = &ass.value.value {
                        location = *val as u32;
                    }
                }
                _ => {}
            }
        }
    }

    if !name.is_empty() {
        map.insert(
            name.clone(),
            OobDivision {
                name,
                template,
                location,
                path: std::sync::Arc::from(file_path.to_string()),
                range: ast::Range {
                    start_line: 0,
                    start_col: 0,
                    end_line: 0,
                    end_col: 0,
                },
            },
        );
    }
}

fn extract_fleet(
    entries: &[ast::Entry],
    source: &str,
    file_path: &str,
    map: &mut HashMap<String, OobFleet>,
    range: ast::Range,
) {
    let mut name = String::new();
    let mut naval_base = None;

    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let key_lower = ass.key_text(source).to_ascii_lowercase();
            match key_lower.as_str() {
                "name" => {
                    if let Some(val) = ass.value.value.as_str(source) {
                        name = val.to_string();
                    }
                }
                "naval_base" => {
                    if let ast::Value::Number(val) = &ass.value.value {
                        naval_base = Some(*val as u32);
                    }
                }
                "task_force" => {
                    // Task forces have their own name and location, contain ships
                    // We extract the fleet-level info but don't currently index task forces separately
                }
                _ => {}
            }
        }
    }

    if !name.is_empty() {
        map.insert(
            name.clone(),
            OobFleet {
                name,
                naval_base,
                path: std::sync::Arc::from(file_path.to_string()),
                range,
            },
        );
    }
}
