use crate::ast;
use crate::parser;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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
    pub path: String,
    pub range: ast::Range,
}

pub fn scan_characters<F>(roots: &[PathBuf], filter: &F) -> HashMap<String, Character>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();

    for root in roots {
        let char_dir = root.join("common/characters");
        if char_dir.exists() {
            scan_directory(&char_dir, &mut map, filter);
        }
    }

    map
}

fn scan_directory<F>(dir_path: &Path, map: &mut HashMap<String, Character>, filter: &F)
where
    F: Fn(&Path) -> bool,
{
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
                            find_characters_in_entries(
                                &script.entries,
                                &path.to_string_lossy(),
                                map,
                            );
                        }
                    }
                }
            }
        }
    }
}

fn find_characters_in_entries(
    entries: &[ast::Entry],
    file_path: &str,
    map: &mut HashMap<String, Character>,
) {
    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                if ass.key.eq_ignore_ascii_case("characters") {
                    if let ast::Value::Block(characters) = &ass.value.value {
                        for char_entry in characters {
                            if let ast::Entry::Assignment(char_ass) = char_entry {
                                let mut character = Character {
                                    id: char_ass.key.clone(),
                                    name: None,
                                    gender: None,
                                    desc: None,
                                    portraits: HashMap::new(),
                                    roles: Vec::new(),
                                    path: file_path.to_string(),
                                    range: char_ass.key_range.clone(),
                                };

                                if let ast::Value::Block(details) = &char_ass.value.value {
                                    parse_character_details(details, &mut character);
                                }
                                map.insert(character.id.clone(), character);
                            }
                        }
                    }
                } else {
                    if let ast::Value::Block(inner_entries) = &ass.value.value {
                        find_characters_in_entries(inner_entries, file_path, map);
                    }
                }
            }
            ast::Entry::Value(val) => {
                if let ast::Value::Block(inner_entries) = &val.value {
                    find_characters_in_entries(inner_entries, file_path, map);
                }
            }
            _ => {}
        }
    }
}

fn parse_character_details(entries: &[ast::Entry], character: &mut Character) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let key = ass.key.to_ascii_lowercase();
            match key.as_str() {
                "name" => {
                    if let ast::Value::String(s) = &ass.value.value {
                        character.name = Some(s.clone());
                    }
                }
                "gender" => {
                    if let ast::Value::String(s) = &ass.value.value {
                        character.gender = Some(s.clone());
                    }
                }
                "desc" => {
                    if let ast::Value::String(s) = &ass.value.value {
                        character.desc = Some(s.clone());
                    }
                }
                "portraits" => {
                    if let ast::Value::Block(categories) = &ass.value.value {
                        for cat_entry in categories {
                            if let ast::Entry::Assignment(cat_ass) = cat_entry {
                                let category = cat_ass.key.to_ascii_lowercase();
                                if let ast::Value::Block(sizes) = &cat_ass.value.value {
                                    for size_entry in sizes {
                                        if let ast::Entry::Assignment(size_ass) = size_entry {
                                            let size = size_ass.key.to_ascii_lowercase();
                                            if let ast::Value::String(sprite) =
                                                &size_ass.value.value
                                            {
                                                character.portraits.insert(
                                                    format!("{}_{}", category, size),
                                                    sprite.clone(),
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
                        parse_character_details(instance_entries, character);
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
                        parse_role_details(role_details, &mut role);
                        character.roles.push(role);
                    }
                }
                _ => {}
            }
        }
    }
}

fn parse_role_details(entries: &[ast::Entry], role: &mut CharacterRole) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let key = ass.key.to_ascii_lowercase();
            match key.as_str() {
                "ideology" => {
                    if let ast::Value::String(s) = &ass.value.value {
                        role.ideology = Some(s.clone());
                    }
                }
                "traits" => {
                    if let ast::Value::Block(trait_entries) = &ass.value.value {
                        for trait_entry in trait_entries {
                            if let ast::Entry::Value(v) = trait_entry {
                                if let ast::Value::String(s) = &v.value {
                                    role.traits.push(s.clone());
                                }
                            }
                        }
                    } else if let ast::Value::TaggedBlock(_, trait_entries, _) = &ass.value.value {
                        for trait_entry in trait_entries {
                            if let ast::Entry::Value(v) = trait_entry {
                                if let ast::Value::String(s) = &v.value {
                                    role.traits.push(s.clone());
                                }
                            }
                        }
                    }
                }
                "skill" => role.skill = get_int(&ass.value),
                "attack_skill" => role.attack_skill = get_int(&ass.value),
                "defense_skill" => role.defense_skill = get_int(&ass.value),
                "planning_skill" => role.planning_skill = get_int(&ass.value),
                "logistics_skill" => role.logistics_skill = get_int(&ass.value),
                "maneuvering_skill" => role.maneuvering_skill = get_int(&ass.value),
                "coordination_skill" => role.coordination_skill = get_int(&ass.value),
                _ => {}
            }
        }
    }
}

fn get_int(val: &ast::NodeedValue) -> Option<i32> {
    match &val.value {
        ast::Value::Number(n) => Some(*n as i32),
        ast::Value::String(s) => s.parse::<i32>().ok(),
        _ => None,
    }
}
