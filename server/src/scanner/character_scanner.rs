#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CharacterRole {
    pub role_type: String, // e.g. "corps_commander", "country_leader", "advisor"
    pub traits: Vec<String>,
    pub skill: Option<i32>,
    pub attack_skill: Option<i32>,
    pub defense_skill: Option<i32>,
    pub planning_skill: Option<i32>,
    pub logistics_skill: Option<i32>,
    pub maneuvering_skill: Option<i32>,
    pub coordination_skill: Option<i32>,
    pub ideology: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Character {
    pub id: String,
    pub name: Option<String>,               // The loc key
    pub gender: Option<String>,             // "male", "female", or "undefined"
    pub desc: Option<String>,               // Loc key for country_leader/scientist description
    pub portraits: HashMap<String, String>, // e.g. "army_large" -> "GFX_portrait...", "civilian_small" -> ...
    pub roles: Vec<CharacterRole>,
    pub path: InternedStr,
    pub range: ast::Range,
}

pub fn scan_characters<F>(roots: &[PathBuf], filter: &F) -> HashMap<String, Character>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut map = HashMap::new();

    for root in roots {
        crate::utils::fs_util::walk_and_parse_files(
            &root.join("common/characters"),
            &["txt"],
            filter,
            |path, content| {
                let (script, _) = parser::parse_script(&content);
                find_characters_in_entries(
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

pub fn scan_character_files<F>(files: &[PathBuf], filter: &F) -> HashMap<String, Character>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut map = HashMap::new();
    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        find_characters_in_entries(
            &script.entries,
            &script.source,
            &path.to_string_lossy(),
            &mut map,
        );
    });
    map
}

pub(crate) fn find_characters_in_entries(
    entries: &[ast::Entry],
    source: &str,
    file_path: &str,
    map: &mut HashMap<String, Character>,
) {
    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                if ass.key_text(source).eq_ignore_ascii_case("characters") {
                    if let ast::Value::Block(characters) = &ass.value.value {
                        for char_entry in characters {
                            if let ast::Entry::Assignment(char_ass) = char_entry {
                                let id = char_ass.key_text(source).to_string();
                                let mut character = Character {
                                    id: id.clone(),
                                    name: None,
                                    gender: None,
                                    desc: None,
                                    portraits: HashMap::new(),
                                    roles: Vec::new(),
                                    path: std::sync::Arc::from(file_path),
                                    range: char_ass.key_range.clone(),
                                };

                                if let ast::Value::Block(details) = &char_ass.value.value {
                                    parse_character_details(details, source, &mut character);
                                }
                                map.insert(id, character);
                            }
                        }
                    }
                } else {
                    if let ast::Value::Block(inner_entries) = &ass.value.value {
                        find_characters_in_entries(inner_entries, source, file_path, map);
                    }
                }
            }
            ast::Entry::Value(val) => {
                if let ast::Value::Block(inner_entries) = &val.value {
                    find_characters_in_entries(inner_entries, source, file_path, map);
                }
            }
            _ => {}
        }
    }
}

fn parse_character_details(entries: &[ast::Entry], source: &str, character: &mut Character) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let key = ass.key_text(source).to_ascii_lowercase();
            match key.as_str() {
                "name" => {
                    if let Some(s) = ass.value.value.as_str(source) {
                        character.name = Some(s.to_string());
                    }
                }
                "gender" => {
                    if let Some(s) = ass.value.value.as_str(source) {
                        character.gender = Some(s.to_string());
                    }
                }
                "desc" => {
                    if let Some(s) = ass.value.value.as_str(source) {
                        character.desc = Some(s.to_string());
                    }
                }
                "portraits" => {
                    if let ast::Value::Block(categories) = &ass.value.value {
                        for cat_entry in categories {
                            if let ast::Entry::Assignment(cat_ass) = cat_entry {
                                let category = cat_ass.key_text(source).to_ascii_lowercase();
                                if let ast::Value::Block(sizes) = &cat_ass.value.value {
                                    for size_entry in sizes {
                                        if let ast::Entry::Assignment(size_ass) = size_entry {
                                            let size =
                                                size_ass.key_text(source).to_ascii_lowercase();
                                            if let Some(sprite) =
                                                size_ass.value.value.as_str(source)
                                            {
                                                character.portraits.insert(
                                                    format!("{}_{}", category, size),
                                                    sprite.to_string(),
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                "instance" => {
                    // Characters in instance = { allowed = { ... } ... } blocks
                    if let ast::Value::Block(instance_entries) = &ass.value.value {
                        parse_character_details(instance_entries, source, character);
                    }
                }
                "advisor" | "country_leader" | "corps_commander" | "field_marshal"
                | "navy_leader" | "scientist" => {
                    if let ast::Value::Block(role_details) = &ass.value.value {
                        let mut role = CharacterRole {
                            role_type: key.clone(),
                            traits: Vec::new(),
                            skill: None,
                            attack_skill: None,
                            defense_skill: None,
                            planning_skill: None,
                            logistics_skill: None,
                            maneuvering_skill: None,
                            coordination_skill: None,
                            ideology: None,
                        };
                        parse_role_details(role_details, source, &mut role);
                        character.roles.push(role);
                    }
                }
                _ => {}
            }
        }
    }
}

fn parse_role_details(entries: &[ast::Entry], source: &str, role: &mut CharacterRole) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let key = ass.key_text(source).to_ascii_lowercase();
            match key.as_str() {
                "ideology" => {
                    if let Some(s) = ass.value.value.as_str(source) {
                        role.ideology = Some(s.to_string());
                    }
                }
                "traits" => {
                    if let ast::Value::Block(trait_entries) = &ass.value.value {
                        for trait_entry in trait_entries {
                            if let ast::Entry::Value(v) = trait_entry {
                                if let Some(s) = v.value.as_str(source) {
                                    role.traits.push(s.to_string());
                                }
                            }
                        }
                    } else if let ast::Value::TaggedBlock(_, trait_entries, _) = &ass.value.value {
                        for trait_entry in trait_entries {
                            if let ast::Entry::Value(v) = trait_entry {
                                if let Some(s) = v.value.as_str(source) {
                                    role.traits.push(s.to_string());
                                }
                            }
                        }
                    }
                }
                "skill" => role.skill = get_int(&ass.value, source),
                "attack_skill" => role.attack_skill = get_int(&ass.value, source),
                "defense_skill" => role.defense_skill = get_int(&ass.value, source),
                "planning_skill" => role.planning_skill = get_int(&ass.value, source),
                "logistics_skill" => role.logistics_skill = get_int(&ass.value, source),
                "maneuvering_skill" => role.maneuvering_skill = get_int(&ass.value, source),
                "coordination_skill" => role.coordination_skill = get_int(&ass.value, source),
                _ => {}
            }
        }
    }
}

fn get_int(val: &ast::NodeedValue, source: &str) -> Option<i32> {
    match &val.value {
        ast::Value::Number(n) => Some(*n as i32),
        ast::Value::String(span) => span.resolve(source).parse::<i32>().ok(),
        _ => None,
    }
}
