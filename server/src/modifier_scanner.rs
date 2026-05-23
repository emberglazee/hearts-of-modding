use crate::ast;
use crate::parser;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Modifier {
    #[allow(dead_code)]
    pub name: String,
    pub path: String,
    pub range: ast::Range,
}

pub struct ModifierResult {
    pub custom_modifiers: HashMap<String, Modifier>,
    pub builtin_mappings: HashMap<String, String>,
}

pub fn scan_modifiers<F>(roots: &[PathBuf], filter: &F) -> ModifierResult
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut custom_modifiers = HashMap::new();

    for root in roots {
        let dir = root.join("common/modifiers");
        if dir.exists() {
            let found = scan_directory(&dir, filter);
            custom_modifiers.extend(found);
        }
        let dynamic_dir = root.join("common/dynamic_modifiers");
        if dynamic_dir.exists() {
            let found = scan_directory(&dynamic_dir, filter);
            custom_modifiers.extend(found);
        }
    }

    ModifierResult {
        custom_modifiers,
        builtin_mappings: get_builtin_mappings(),
    }
}

fn scan_directory<F>(dir_path: &Path, filter: &F) -> HashMap<String, Modifier>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut map = HashMap::new();
    let mut dirs_to_check = vec![dir_path.to_path_buf()];

    while let Some(current_dir) = dirs_to_check.pop() {
        if filter(&current_dir) {
            continue;
        }
        if let Ok(entries) = fs::read_dir(current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if !filter(&path) {
                        dirs_to_check.push(path);
                    }
                } else if path.extension().is_some_and(|ext| ext == "txt") {
                    if filter(&path) {
                        continue;
                    }
                    if let Ok(content) = fs::read_to_string(&path) {
                        {
                            let (script, _) = parser::parse_script(&content);
                            for entry_ast in script.entries {
                                if let ast::Entry::Assignment(ass) = entry_ast {
                                    map.insert(
                                        ass.key.clone(),
                                        Modifier {
                                            name: ass.key.clone(),
                                            path: path.to_string_lossy().to_string(),
                                            range: ass.key_range,
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    map
}

fn get_builtin_mappings() -> HashMap<String, String> {
    let mut m = HashMap::new();
    // Common HOI4 engine modifiers mapping to their localization keys
    let mappings = [
        ("breakthrough_factor", "MODIFIER_BREAKTHROUGH"),
        ("shore_bombardment_bonus", "MODIFIER_SHORE_BOMBARDMENT"),
        ("monthly_population", "MODIFIER_GLOBAL_MONTHLY_POPULATION"),
        ("conscription", "MODIFIER_CONSCRIPTION_FACTOR"),
        ("refit_ic_cost", "MODIFIER_INDUSTRIAL_REFIT_IC_COST_FACTOR"),
        ("experience_gain_factor", "MODIFIER_XP_GAIN_FACTOR"),
        (
            "resistance_damage_to_garrison",
            "MODIFIER_RESISTANCE_DAMAGE_TO_GARRISONS",
        ),
        ("compliance_gain", "MODIFIER_COMPLIANCE_GAIN_ADD"),
        ("army_strength_factor", "MODIFIER_ARMY_STRENGTH"),
        ("navy_visibility", "MODIFIER_NAVAL_VISIBILITY_FACTOR"),
        ("experience_gain_air_factor", "experience_gain_air"),
        ("political_power_factor", "MODIFIER_POLITICAL_POWER_FACTOR"),
        ("stability_factor", "MODIFIER_STABILITY_FACTOR"),
        ("war_support_factor", "MODIFIER_WAR_SUPPORT_FACTOR"),
        ("war_stability_factor", "MODIFIER_WAR_STABILITY_FACTOR"),
        ("army_morale_factor", "MODIFIER_ARMY_MORALE_FACTOR"),
        (
            "industrial_capacity_factory",
            "MODIFIER_INDUSTRIAL_CAPACITY_FACTOR",
        ),
        ("consumer_goods_factor", "MODIFIER_CONSUMER_GOODS_FACTOR"),
        ("local_resources_factor", "MODIFIER_LOCAL_RESOURCES_FACTOR"),
        ("non_core_manpower", "MODIFIER_NON_CORE_MANPOWER"),
        ("research_speed_factor", "MODIFIER_RESEARCH_SPEED_FACTOR"),
        (
            "production_speed_buildings_factor",
            "MODIFIER_PRODUCTION_SPEED_BUILDINGS_FACTOR",
        ),
    ];

    for (k, v) in mappings {
        m.insert(k.to_string(), v.to_string());
    }
    m
}
