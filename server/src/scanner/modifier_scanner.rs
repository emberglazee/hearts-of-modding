use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Modifier {
    #[allow(dead_code)]
    pub name: String,
    pub path: InternedStr,
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
        crate::utils::fs_util::walk_and_parse_files(
            &root.join("common/modifiers"),
            &["txt"],
            filter,
            |path, content| {
                let (script, _) = parser::parse_script(&content);
                for entry_ast in script.entries {
                    if let ast::Entry::Assignment(ass) = entry_ast {
                        let name = ass.key_text(&script.source).to_string();
                        custom_modifiers.insert(
                            name.clone(),
                            Modifier {
                                name,
                                path: std::sync::Arc::from(path.to_string_lossy().as_ref()),
                                range: ass.key_range,
                            },
                        );
                    }
                }
            },
        );
        crate::utils::fs_util::walk_and_parse_files(
            &root.join("common/dynamic_modifiers"),
            &["txt"],
            filter,
            |path, content| {
                let (script, _) = parser::parse_script(&content);
                for entry_ast in script.entries {
                    if let ast::Entry::Assignment(ass) = entry_ast {
                        let name = ass.key_text(&script.source).to_string();
                        custom_modifiers.insert(
                            name.clone(),
                            Modifier {
                                name,
                                path: std::sync::Arc::from(path.to_string_lossy().as_ref()),
                                range: ass.key_range,
                            },
                        );
                    }
                }
            },
        );
    }

    ModifierResult {
        custom_modifiers,
        builtin_mappings: get_builtin_mappings(),
    }
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
