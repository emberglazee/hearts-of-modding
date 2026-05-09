use crate::ast;
use crate::building_scanner;
use crate::defines_parser;
use std::collections::{HashMap, HashSet};

/// Diagnostic codes for advanced validation
pub const BUILDING_LEVEL_EXCEEDS_MAX: &str = "HOM1002";
pub const CHARACTER_SKILL_EXCEEDS_MAX: &str = "HOM1004";
pub const VICTORY_POINT_PROVINCE_NOT_IN_STATE: &str = "HOM2001";
pub const ACHIEVEMENT_MISSING_LOCALIZATION: &str = "HOM3001";
pub const SCHEMA_VALIDATION_ERROR: &str = "HOM4001";

#[derive(Debug, Clone)]
pub struct ValidationDiagnostic {
    pub range: ast::Range,
    pub severity: ast::DiagnosticSeverity,
    pub message: String,
    pub code: String,
    pub fix_suggestion: Option<String>,
}

/// Validate entries against a schema rule
pub fn validate_against_rule(
    entries: &[ast::Entry],
    rule: &crate::schema::Rule,
    schema: &crate::schema::Schema,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    let mut counts: HashMap<String, u32> = HashMap::new();

    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                let key = &ass.key;
                *counts.entry(key.clone()).or_insert(0) += 1;

                // Find matching child rule
                if let Some(child_rule) = rule.children.iter().find(|r| r.key == *key || r.key == "alias_name[trigger]" || r.key == "alias_name[effect]") {
                    validate_value_against_rule(&ass.value, child_rule, schema, diagnostics);
                } else {
                    // Try to find in global triggers/effects if it's an alias
                    let global_rule = schema.triggers.get(key).or_else(|| schema.effects.get(key));
                    if let Some(gr) = global_rule {
                        validate_value_against_rule(&ass.value, gr, schema, diagnostics);
                    }
                }
            }
            _ => {}
        }
    }

    // Check cardinality
    for child_rule in &rule.children {
        let count = *counts.get(&child_rule.key).unwrap_or(&0);
        if count < child_rule.cardinality.min {
            diagnostics.push(ValidationDiagnostic {
                range: rule_range_placeholder(), 
                severity: child_rule.severity.clone().unwrap_or(ast::DiagnosticSeverity::Error),
                message: format!("Missing required field: '{}' (expected at least {})", child_rule.key, child_rule.cardinality.min),
                code: SCHEMA_VALIDATION_ERROR.to_string(),
                fix_suggestion: None,
            });
        }
        if let Some(max) = child_rule.cardinality.max {
            if count > max {
                diagnostics.push(ValidationDiagnostic {
                    range: rule_range_placeholder(),
                    severity: child_rule.severity.clone().unwrap_or(ast::DiagnosticSeverity::Error),
                    message: format!("Too many occurrences of field: '{}' (expected at most {})", child_rule.key, max),
                    code: SCHEMA_VALIDATION_ERROR.to_string(),
                    fix_suggestion: None,
                });
            }
        }
    }
}

pub fn validate_value_against_rule(
    node: &ast::NodeedValue,
    rule: &crate::schema::Rule,
    schema: &crate::schema::Schema,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    match &rule.value_type {
        crate::schema::ValueType::Enum(name) => {
            if let ast::Value::String(s) = &node.value {
                if let Some(values) = schema.enums.get(name) {
                    if !values.contains(s) {
                        diagnostics.push(ValidationDiagnostic {
                            range: node.range.clone(),
                            severity: rule.severity.clone().unwrap_or(ast::DiagnosticSeverity::Error),
                            message: format!("Invalid value '{}' for enum '{}'. Expected one of: {:?}", s, name, values),
                            code: SCHEMA_VALIDATION_ERROR.to_string(),
                            fix_suggestion: None,
                        });
                    }
                }
            }
        }
        crate::schema::ValueType::Type(name) => {
            if let ast::Value::String(s) = &node.value {
                if s.is_empty() {
                    diagnostics.push(ValidationDiagnostic {
                        range: node.range.clone(),
                        severity: rule.severity.clone().unwrap_or(ast::DiagnosticSeverity::Error),
                        message: format!("Empty reference for type '{}'", name),
                        code: SCHEMA_VALIDATION_ERROR.to_string(),
                        fix_suggestion: None,
                    });
                }
            }
        }
        crate::schema::ValueType::Bool => {
            if !matches!(node.value, ast::Value::Boolean(_)) {
                if let ast::Value::String(s) = &node.value {
                    if s != "yes" && s != "no" {
                        diagnostics.push(ValidationDiagnostic {
                            range: node.range.clone(),
                            severity: rule.severity.clone().unwrap_or(ast::DiagnosticSeverity::Error),
                            message: "Expected boolean (yes/no)".to_string(),
                            code: SCHEMA_VALIDATION_ERROR.to_string(),
                            fix_suggestion: None,
                        });
                    }
                } else {
                    diagnostics.push(ValidationDiagnostic {
                        range: node.range.clone(),
                        severity: rule.severity.clone().unwrap_or(ast::DiagnosticSeverity::Error),
                        message: "Expected boolean (yes/no)".to_string(),
                        code: SCHEMA_VALIDATION_ERROR.to_string(),
                        fix_suggestion: None,
                    });
                }
            }
        }
        crate::schema::ValueType::Int | crate::schema::ValueType::Float => {
            if !matches!(node.value, ast::Value::Number(_)) {
                if let ast::Value::String(s) = &node.value {
                    if s.parse::<f64>().is_err() {
                        diagnostics.push(ValidationDiagnostic {
                            range: node.range.clone(),
                            severity: rule.severity.clone().unwrap_or(ast::DiagnosticSeverity::Error),
                            message: "Expected a number".to_string(),
                            code: SCHEMA_VALIDATION_ERROR.to_string(),
                            fix_suggestion: None,
                        });
                    }
                } else {
                    diagnostics.push(ValidationDiagnostic {
                        range: node.range.clone(),
                        severity: rule.severity.clone().unwrap_or(ast::DiagnosticSeverity::Error),
                        message: "Expected a number".to_string(),
                        code: SCHEMA_VALIDATION_ERROR.to_string(),
                        fix_suggestion: None,
                    });
                }
            }
        }
        crate::schema::ValueType::Block => {
            if let ast::Value::Block(entries) = &node.value {
                validate_against_rule(entries, rule, schema, diagnostics);
            } else {
                diagnostics.push(ValidationDiagnostic {
                    range: node.range.clone(),
                    severity: rule.severity.clone().unwrap_or(ast::DiagnosticSeverity::Error),
                    message: "Expected a block { ... }".to_string(),
                    code: SCHEMA_VALIDATION_ERROR.to_string(),
                    fix_suggestion: None,
                });
            }
        }
        crate::schema::ValueType::Alias(kind) => {
            if let ast::Value::Block(entries) = &node.value {
                for entry in entries {
                    if let ast::Entry::Assignment(ass) = entry {
                        let sub_rule = if kind == "trigger" { schema.triggers.get(&ass.key) }
                                       else if kind == "effect" { schema.effects.get(&ass.key) }
                                       else { None };
                        if let Some(sr) = sub_rule {
                            validate_value_against_rule(&ass.value, sr, schema, diagnostics);
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn rule_range_placeholder() -> ast::Range {
    ast::Range { start_line: 0, start_col: 0, end_line: 0, end_col: 0 }
}

/// Validate achievements
pub fn validate_achievements(
    entries: &[ast::Entry],
    localization: &HashMap<String, crate::loc_parser::LocEntry>,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let key_lower = ass.key.to_lowercase();
            if key_lower == "custom_achievement" || key_lower == "custom_ribbon" {
                let name_key = format!("{}_NAME", ass.key);
                let desc_key = format!("{}_DESC", ass.key);

                if !localization.contains_key(&name_key) {
                    diagnostics.push(ValidationDiagnostic {
                        range: ass.key_range.clone(),
                        severity: ast::DiagnosticSeverity::Warning,
                        message: format!("Achievement '{}' is missing localization key: '{}'", ass.key, name_key),
                        code: ACHIEVEMENT_MISSING_LOCALIZATION.to_string(),
                        fix_suggestion: None,
                    });
                }
                if !localization.contains_key(&desc_key) {
                    diagnostics.push(ValidationDiagnostic {
                        range: ass.key_range.clone(),
                        severity: ast::DiagnosticSeverity::Warning,
                        message: format!("Achievement '{}' is missing localization key: '{}'", ass.key, desc_key),
                        code: ACHIEVEMENT_MISSING_LOCALIZATION.to_string(),
                        fix_suggestion: None,
                    });
                }
            }
        }
    }
}

/// Validate building levels in state history files
pub fn validate_building_levels(
    entries: &[ast::Entry],
    buildings: &HashMap<String, building_scanner::Building>,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    validate_buildings_recursive(entries, buildings, diagnostics);
}

fn validate_buildings_recursive(
    entries: &[ast::Entry],
    buildings: &HashMap<String, building_scanner::Building>,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let key_lower = ass.key.to_lowercase();
            
            // Check if we're in a buildings block
            if key_lower == "buildings" {
                if let ast::Value::Block(building_entries) = &ass.value.value {
                    validate_building_block(building_entries, buildings, diagnostics);
                }
            }
            
            // Recurse into nested blocks
            match &ass.value.value {
                ast::Value::Block(inner) => {
                    validate_buildings_recursive(inner, buildings, diagnostics);
                }
                ast::Value::TaggedBlock(_, inner, _) => {
                    validate_buildings_recursive(inner, buildings, diagnostics);
                }
                _ => {}
            }
        }
    }
}

fn validate_building_block(
    entries: &[ast::Entry],
    buildings: &HashMap<String, building_scanner::Building>,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let building_name = &ass.key;
            
            // Get the level value
            let level = match &ass.value.value {
                ast::Value::Number(n) => Some(*n as i32),
                ast::Value::String(s) => s.parse::<i32>().ok(),
                _ => None,
            };
            
            if let Some(level) = level {
                // Check if building exists and has max_level
                if let Some(building) = buildings.get(building_name) {
                    if let Some(max_level) = building.max_level {
                        if level > max_level {
                            diagnostics.push(ValidationDiagnostic {
                                range: ass.value.range.clone(),
                                severity: ast::DiagnosticSeverity::Error,
                                message: format!(
                                    "Building level {} exceeds maximum level {} for '{}'",
                                    level, max_level, building_name
                                ),
                                code: BUILDING_LEVEL_EXCEEDS_MAX.to_string(),
                                fix_suggestion: Some(format!("Set to maximum level: {}", max_level)),
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Validate character skill levels
pub fn validate_character_skills(
    entries: &[ast::Entry],
    defines: &defines_parser::GameDefines,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    validate_character_skills_recursive(entries, defines, diagnostics, None);
}

fn validate_character_skills_recursive(
    entries: &[ast::Entry],
    defines: &defines_parser::GameDefines,
    diagnostics: &mut Vec<ValidationDiagnostic>,
    current_character_type: Option<&str>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let key_lower = ass.key.to_lowercase();
            
            // Detect character type
            let mut char_type = current_character_type;
            if key_lower == "create_field_marshal" {
                char_type = Some("field_marshal");
            } else if key_lower == "create_corps_commander" {
                char_type = Some("corps_commander");
            } else if key_lower == "create_navy_leader" {
                char_type = Some("navy_leader");
            } else if key_lower == "create_operative_leader" {
                char_type = Some("operative");
            }
            
            // Check skill field
            if key_lower == "skill" {
                if let Some(ct) = char_type {
                    let skill = match &ass.value.value {
                        ast::Value::Number(n) => Some(*n as i32),
                        ast::Value::String(s) => s.parse::<i32>().ok(),
                        _ => None,
                    };
                    
                    if let Some(skill) = skill {
                        let max_skill = defines.get_max_skill(ct);
                        if skill > max_skill {
                            diagnostics.push(ValidationDiagnostic {
                                range: ass.value.range.clone(),
                                severity: ast::DiagnosticSeverity::Error,
                                message: format!(
                                    "Skill level {} exceeds maximum {} for {}",
                                    skill, max_skill, ct
                                ),
                                code: CHARACTER_SKILL_EXCEEDS_MAX.to_string(),
                                fix_suggestion: Some(format!("Set to maximum skill: {}", max_skill)),
                            });
                        }
                    }
                }
            }
            
            // Recurse into nested blocks
            match &ass.value.value {
                ast::Value::Block(inner) => {
                    validate_character_skills_recursive(inner, defines, diagnostics, char_type);
                }
                ast::Value::TaggedBlock(_, inner, _) => {
                    validate_character_skills_recursive(inner, defines, diagnostics, char_type);
                }
                _ => {}
            }
        }
    }
}

/// Validate victory points reference valid provinces in the state
pub fn validate_victory_points(
    entries: &[ast::Entry],
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    validate_victory_points_recursive(entries, diagnostics, &mut None, &mut None);
}

fn validate_victory_points_recursive(
    entries: &[ast::Entry],
    diagnostics: &mut Vec<ValidationDiagnostic>,
    state_provinces: &mut Option<HashSet<u32>>,
    victory_points: &mut Option<Vec<(u32, ast::Range)>>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let key_lower = ass.key.to_lowercase();
            
            // Collect provinces in state
            if key_lower == "provinces" {
                if let ast::Value::Block(province_entries) = &ass.value.value {
                    let mut provs = HashSet::new();
                    for prov_entry in province_entries {
                        if let ast::Entry::Value(val) = prov_entry {
                            if let ast::Value::Number(n) = &val.value {
                                provs.insert(*n as u32);
                            } else if let ast::Value::String(s) = &val.value {
                                if let Ok(n) = s.parse::<u32>() {
                                    provs.insert(n);
                                }
                            }
                        }
                    }
                    *state_provinces = Some(provs);
                }
            }
            
            // Collect victory points
            // Format: victory_points = { province_id vp_value province_id vp_value ... }
            if key_lower == "victory_points" {
                if let ast::Value::Block(vp_entries) = &ass.value.value {
                    let mut vps = Vec::new();
                    let mut values: Vec<(u32, ast::Range)> = Vec::new();
                    
                    // First, collect all numeric values
                    for vp_entry in vp_entries {
                        if let ast::Entry::Value(val) = vp_entry {
                            let num = match &val.value {
                                ast::Value::Number(n) => Some(*n as u32),
                                ast::Value::String(s) => s.parse::<u32>().ok(),
                                _ => None,
                            };
                            
                            if let Some(n) = num {
                                values.push((n, val.range.clone()));
                            }
                        }
                    }
                    
                    // Now parse pairs: (province_id, vp_value)
                    // We only care about the province_id (first of each pair)
                    for i in (0..values.len()).step_by(2) {
                        if i < values.len() {
                            vps.push(values[i].clone());
                        }
                    }
                    
                    *victory_points = Some(vps);
                }
            }
            
            // Recurse into nested blocks
            match &ass.value.value {
                ast::Value::Block(inner) => {
                    validate_victory_points_recursive(inner, diagnostics, state_provinces, victory_points);
                }
                ast::Value::TaggedBlock(_, inner, _) => {
                    validate_victory_points_recursive(inner, diagnostics, state_provinces, victory_points);
                }
                _ => {}
            }
        }
    }
    
    // After processing all entries, validate victory points against provinces
    if let (Some(provs), Some(vps)) = (state_provinces, victory_points) {
        for (vp_province, range) in vps {
            if !provs.contains(vp_province) {
                diagnostics.push(ValidationDiagnostic {
                    range: range.clone(),
                    severity: ast::DiagnosticSeverity::Hint,
                    message: format!(
                        "Victory point province {} is not in the state's province list",
                        vp_province
                    ),
                    code: VICTORY_POINT_PROVINCE_NOT_IN_STATE.to_string(),
                    fix_suggestion: Some("Remove this victory point or add the province to the state".to_string()),
                });
            }
        }
    }
}
