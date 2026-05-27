use crate::ast;
use crate::parser;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Vanilla ability names used as fallback when no ability files are found in workspace
pub const VANILLA_ABILITY_NAMES: &[&str] = &[
    "force_attack",
    "last_stand",
    "staff_office_plan",
    "siege_artillery",
    "glider_planes",
    "faster_naval_invasion_planning",
    "probing_attack",
    "makeshift_bridges",
    "extra_suplies",
    "requisition_winter_gear",
];

#[derive(Debug, Clone)]
pub struct Ability {
    pub key: String,
    pub name_loc: Option<String>,
    pub desc_loc: Option<String>,
    pub cost: Option<f64>,
    pub duration: Option<i32>,
    pub sound_effect: Option<String>,
    pub type_name: Option<String>,
    pub cancelable: Option<bool>,
    pub cooldown: Option<i32>,
    pub icon: Option<String>,
    pub has_allowed: bool,
    pub has_one_time_effect: bool,
    pub has_unit_modifiers: bool,
    pub has_ai_will_do: bool,
    pub path: String,
    pub range: ast::Range,
}

pub fn scan_abilities<F>(roots: &[std::path::PathBuf], filter: &F) -> HashMap<String, Ability>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();

    for root in roots {
        let dir_path = root.join("common").join("abilities");
        if !dir_path.exists() || filter(&dir_path) {
            continue;
        }

        let mut dirs_to_check = vec![dir_path.to_path_buf()];
        while let Some(current_dir) = dirs_to_check.pop() {
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
                                find_abilities_in_entries(
                                    &script.entries,
                                    &path.to_string_lossy(),
                                    &mut map,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback: if no abilities were scanned, seed with vanilla names for completions
    if map.is_empty() {
        for name in VANILLA_ABILITY_NAMES {
            map.insert(
                name.to_string(),
                Ability {
                    key: name.to_string(),
                    name_loc: None,
                    desc_loc: None,
                    cost: None,
                    duration: None,
                    sound_effect: None,
                    type_name: Some("army_leader".to_string()),
                    cancelable: None,
                    cooldown: None,
                    icon: None,
                    has_allowed: false,
                    has_one_time_effect: false,
                    has_unit_modifiers: false,
                    has_ai_will_do: false,
                    path: String::new(),
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

    map
}

fn find_abilities_in_entries(
    entries: &[ast::Entry],
    file_path: &str,
    map: &mut HashMap<String, Ability>,
) {
    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                if ass.key.eq_ignore_ascii_case("ability") {
                    if let ast::Value::Block(ability_entries) = &ass.value.value {
                        for ability_entry in ability_entries {
                            if let ast::Entry::Assignment(a_ass) = ability_entry {
                                let mut name_loc = None;
                                let mut desc_loc = None;
                                let mut cost = None;
                                let mut duration = None;
                                let mut sound_effect = None;
                                let mut type_name = None;
                                let mut cancelable = None;
                                let mut cooldown = None;
                                let mut icon = None;
                                let mut has_allowed = false;
                                let mut has_one_time_effect = false;
                                let mut has_unit_modifiers = false;
                                let mut has_ai_will_do = false;

                                if let ast::Value::Block(props) = &a_ass.value.value {
                                    for prop in props {
                                        if let ast::Entry::Assignment(p_ass) = prop {
                                            let p_key = p_ass.key.as_str();
                                            if p_key.eq_ignore_ascii_case("name") {
                                                if let ast::Value::String(s) = &p_ass.value.value {
                                                    name_loc = Some(s.clone());
                                                }
                                            } else if p_key.eq_ignore_ascii_case("desc") {
                                                if let ast::Value::String(s) = &p_ass.value.value {
                                                    desc_loc = Some(s.clone());
                                                }
                                            } else if p_key.eq_ignore_ascii_case("sound_effect") {
                                                if let ast::Value::String(s) = &p_ass.value.value {
                                                    sound_effect = Some(s.clone());
                                                }
                                            } else if p_key.eq_ignore_ascii_case("type") {
                                                if let ast::Value::String(s) = &p_ass.value.value {
                                                    type_name = Some(s.clone());
                                                }
                                            } else if p_key.eq_ignore_ascii_case("cost") {
                                                match &p_ass.value.value {
                                                    ast::Value::Number(n) => cost = Some(*n),
                                                    ast::Value::String(s) => {
                                                        if let Ok(n) = s.parse() {
                                                            cost = Some(n);
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            } else if p_key.eq_ignore_ascii_case("duration") {
                                                match &p_ass.value.value {
                                                    ast::Value::Number(n) => {
                                                        duration = Some(*n as i32)
                                                    }
                                                    ast::Value::String(s) => {
                                                        if let Ok(n) = s.parse() {
                                                            duration = Some(n);
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            } else if p_key.eq_ignore_ascii_case("cancelable") {
                                                cancelable = match &p_ass.value.value {
                                                    ast::Value::String(s) => Some(s == "yes"),
                                                    _ => None,
                                                };
                                            } else if p_key.eq_ignore_ascii_case("cooldown") {
                                                match &p_ass.value.value {
                                                    ast::Value::Number(n) => {
                                                        cooldown = Some(*n as i32)
                                                    }
                                                    ast::Value::String(s) => {
                                                        if let Ok(n) = s.parse() {
                                                            cooldown = Some(n);
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            } else if p_key.eq_ignore_ascii_case("icon") {
                                                if let ast::Value::String(s) = &p_ass.value.value {
                                                    icon = Some(s.clone());
                                                }
                                            } else if p_key.eq_ignore_ascii_case("allowed") {
                                                has_allowed = true;
                                            } else if p_key.eq_ignore_ascii_case("one_time_effect")
                                            {
                                                has_one_time_effect = true;
                                            } else if p_key.eq_ignore_ascii_case("unit_modifiers") {
                                                has_unit_modifiers = true;
                                            } else if p_key.eq_ignore_ascii_case("ai_will_do") {
                                                has_ai_will_do = true;
                                            }
                                        }
                                    }
                                }

                                map.insert(
                                    a_ass.key.clone(),
                                    Ability {
                                        key: a_ass.key.clone(),
                                        name_loc,
                                        desc_loc,
                                        cost,
                                        duration,
                                        sound_effect,
                                        type_name,
                                        cancelable,
                                        cooldown,
                                        icon,
                                        has_allowed,
                                        has_one_time_effect,
                                        has_unit_modifiers,
                                        has_ai_will_do,
                                        path: file_path.to_string(),
                                        range: a_ass.key_range.clone(),
                                    },
                                );
                            }
                        }
                    }
                } else {
                    // Recurse into other blocks
                    if let ast::Value::Block(inner_entries) = &ass.value.value {
                        find_abilities_in_entries(inner_entries, file_path, map);
                    }
                }
            }
            ast::Entry::Value(val) => match &val.value {
                ast::Value::Block(inner_entries) | ast::Value::TaggedBlock(_, inner_entries, _) => {
                    find_abilities_in_entries(inner_entries, file_path, map);
                }
                _ => {}
            },
            _ => {}
        }
    }
}
