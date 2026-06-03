use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
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
    pub path: InternedStr,
    pub range: ast::Range,
}

pub fn scan_abilities<F>(roots: &[std::path::PathBuf], filter: &F) -> HashMap<String, Ability>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();

    for root in roots {
        crate::utils::fs_util::walk_and_parse_files(
            &root.join("common/abilities"),
            &["txt"],
            filter,
            |path, content| {
                let (script, _) = parser::parse_script(&content);
                find_abilities_in_entries(
                    &script.entries,
                    &script.source,
                    &path.to_string_lossy(),
                    &mut map,
                );
            },
        );
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
                    path: std::sync::Arc::from(""),
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

pub(crate) fn find_abilities_in_entries(
    entries: &[ast::Entry],
    source: &str,
    file_path: &str,
    map: &mut HashMap<String, Ability>,
) {
    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                if ass.key_text(source).eq_ignore_ascii_case("ability") {
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
                                            let p_key = p_ass.key_text(source);
                                            if p_key.eq_ignore_ascii_case("name") {
                                                if let Some(s) = p_ass.value.value.as_str(source) {
                                                    name_loc = Some(s.to_string());
                                                }
                                            } else if p_key.eq_ignore_ascii_case("desc") {
                                                if let Some(s) = p_ass.value.value.as_str(source) {
                                                    desc_loc = Some(s.to_string());
                                                }
                                            } else if p_key.eq_ignore_ascii_case("sound_effect") {
                                                if let Some(s) = p_ass.value.value.as_str(source) {
                                                    sound_effect = Some(s.to_string());
                                                }
                                            } else if p_key.eq_ignore_ascii_case("type") {
                                                if let Some(s) = p_ass.value.value.as_str(source) {
                                                    type_name = Some(s.to_string());
                                                }
                                            } else if p_key.eq_ignore_ascii_case("cost") {
                                                match &p_ass.value.value {
                                                    ast::Value::Number(n) => cost = Some(*n),
                                                    ast::Value::String(span) => {
                                                        if let Ok(n) = span.resolve(source).parse()
                                                        {
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
                                                    ast::Value::String(span) => {
                                                        if let Ok(n) = span.resolve(source).parse()
                                                        {
                                                            duration = Some(n);
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            } else if p_key.eq_ignore_ascii_case("cancelable") {
                                                cancelable = match &p_ass.value.value {
                                                    ast::Value::String(span) => {
                                                        Some(span.resolve(source) == "yes")
                                                    }
                                                    _ => None,
                                                };
                                            } else if p_key.eq_ignore_ascii_case("cooldown") {
                                                match &p_ass.value.value {
                                                    ast::Value::Number(n) => {
                                                        cooldown = Some(*n as i32)
                                                    }
                                                    ast::Value::String(span) => {
                                                        if let Ok(n) = span.resolve(source).parse()
                                                        {
                                                            cooldown = Some(n);
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            } else if p_key.eq_ignore_ascii_case("icon") {
                                                if let Some(s) = p_ass.value.value.as_str(source) {
                                                    icon = Some(s.to_string());
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

                                let key = a_ass.key_text(source).to_string();
                                map.insert(
                                    key.clone(),
                                    Ability {
                                        key,
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
                                        path: std::sync::Arc::from(file_path),
                                        range: a_ass.key_range.clone(),
                                    },
                                );
                            }
                        }
                    }
                } else {
                    // Recurse into other blocks
                    if let ast::Value::Block(inner_entries) = &ass.value.value {
                        find_abilities_in_entries(inner_entries, source, file_path, map);
                    }
                }
            }
            ast::Entry::Value(val) => match &val.value {
                ast::Value::Block(inner_entries) | ast::Value::TaggedBlock(_, inner_entries, _) => {
                    find_abilities_in_entries(inner_entries, source, file_path, map);
                }
                _ => {}
            },
            _ => {}
        }
    }
}
