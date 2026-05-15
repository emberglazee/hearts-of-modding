use crate::parser;
use crate::ast;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Sound {
    pub name: String,
    pub file: String,
    pub path: String,
    pub range: ast::Range,
}

#[derive(Debug, Clone)]
pub struct SoundEffect {
    pub name: String,
    pub sounds: Vec<String>,
    pub path: String,
    pub range: ast::Range,
}

#[derive(Debug, Clone)]
pub struct Falloff {
    pub name: String,
    pub path: String,
    pub range: ast::Range,
}

#[derive(Debug, Clone)]
pub struct SoundCategory {
    pub name: String,
    pub soundeffects: Vec<String>,
    pub path: String,
    pub range: ast::Range,
}

pub struct SoundScanResult {
    pub sounds: HashMap<String, Sound>,
    pub sound_effects: HashMap<String, SoundEffect>,
    pub falloffs: HashMap<String, Falloff>,
    pub categories: HashMap<String, SoundCategory>,
}

pub fn scan_sounds<F>(roots: &[PathBuf], filter: &F) -> SoundScanResult 
where F: Fn(&std::path::Path) -> bool {
    let mut sounds = HashMap::new();
    let mut sound_effects = HashMap::new();
    let mut falloffs = HashMap::new();
    let mut categories = HashMap::new();

    for root in roots {
        let sound_dir = root.join("sound");
        if !sound_dir.exists() || filter(&sound_dir) {
            continue;
        }

        let mut dirs_to_check = vec![sound_dir];
        while let Some(current_dir) = dirs_to_check.pop() {
            if let Ok(entries) = fs::read_dir(current_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if !filter(&path) {
                            dirs_to_check.push(path);
                        }
                    } else {
                        if filter(&path) {
                            continue;
                        }
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                        if ext == "asset" {
                            if let Ok(content) = fs::read_to_string(&path) {
                                { let (script, _) = parser::parse_script(&content);
                                    find_sound_definitions(&script.entries, &path.to_string_lossy(), &mut sounds, &mut sound_effects, &mut falloffs, &mut categories);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    SoundScanResult { sounds, sound_effects, falloffs, categories }
}

fn find_sound_definitions(
    entries: &[ast::Entry], 
    file_path: &str, 
    sounds: &mut HashMap<String, Sound>,
    sound_effects: &mut HashMap<String, SoundEffect>,
    falloffs: &mut HashMap<String, Falloff>,
    categories: &mut HashMap<String, SoundCategory>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let key_lower = ass.key.to_lowercase();
            match key_lower.as_str() {
                "sound" => {
                    if let ast::Value::Block(details) = &ass.value.value {
                        let mut name = None;
                        let mut file = None;
                        for detail in details {
                            if let ast::Entry::Assignment(d_ass) = detail {
                                match d_ass.key.to_lowercase().as_str() {
                                    "name" => {
                                        if let ast::Value::String(s) = &d_ass.value.value {
                                            name = Some(s.clone());
                                        }
                                    }
                                    "file" => {
                                        if let ast::Value::String(s) = &d_ass.value.value {
                                            file = Some(s.clone());
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        if let (Some(n), Some(f)) = (name, file) {
                            sounds.insert(n.clone(), Sound {
                                name: n,
                                file: f,
                                path: file_path.to_string(),
                                range: ass.key_range.clone(),
                            });
                        }
                    }
                }
                "soundeffect" => {
                    if let ast::Value::Block(details) = &ass.value.value {
                        let mut name = None;
                        let mut sounds_list = Vec::new();
                        for detail in details {
                            if let ast::Entry::Assignment(d_ass) = detail {
                                match d_ass.key.to_lowercase().as_str() {
                                    "name" => {
                                        if let ast::Value::String(s) = &d_ass.value.value {
                                            name = Some(s.clone());
                                        }
                                    }
                                    "sounds" => {
                                        if let ast::Value::Block(s_details) = &d_ass.value.value {
                                            for s_detail in s_details {
                                                if let ast::Entry::Assignment(s_ass) = s_detail {
                                                    let s_key = s_ass.key.to_lowercase();
                                                    if s_key == "sound" {
                                                        if let ast::Value::String(s_name) = &s_ass.value.value {
                                                            sounds_list.push(s_name.clone());
                                                        }
                                                    } else if s_key == "weighted_sound" {
                                                        if let ast::Value::Block(w_details) = &s_ass.value.value {
                                                            for w_detail in w_details {
                                                                if let ast::Entry::Assignment(w_ass) = w_detail {
                                                                    if w_ass.key.to_lowercase() == "sound" {
                                                                        if let ast::Value::String(s_name) = &w_ass.value.value {
                                                                            sounds_list.push(s_name.clone());
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        if let Some(n) = name {
                            sound_effects.insert(n.clone(), SoundEffect {
                                name: n,
                                sounds: sounds_list,
                                path: file_path.to_string(),
                                range: ass.key_range.clone(),
                            });
                        }
                    }
                }
                "falloff" => {
                    if let ast::Value::Block(details) = &ass.value.value {
                        let mut name = None;
                        for detail in details {
                            if let ast::Entry::Assignment(d_ass) = detail {
                                if d_ass.key.to_lowercase() == "name" {
                                    if let ast::Value::String(s) = &d_ass.value.value {
                                        name = Some(s.clone());
                                    }
                                }
                            }
                        }
                        if let Some(n) = name {
                            falloffs.insert(n.clone(), Falloff {
                                name: n,
                                path: file_path.to_string(),
                                range: ass.key_range.clone(),
                            });
                        }
                    }
                }
                "category" => {
                    if let ast::Value::Block(details) = &ass.value.value {
                        let mut name = None;
                        let mut effects = Vec::new();
                        for detail in details {
                            if let ast::Entry::Assignment(d_ass) = detail {
                                match d_ass.key.to_lowercase().as_str() {
                                    "name" => {
                                        if let ast::Value::String(s) = &d_ass.value.value {
                                            name = Some(s.clone());
                                        }
                                    }
                                    "soundeffects" => {
                                        if let ast::Value::Block(e_details) = &d_ass.value.value {
                                            for e_detail in e_details {
                                                if let ast::Entry::Value(v) = e_detail {
                                                    if let ast::Value::String(s) = &v.value {
                                                        effects.push(s.clone());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        if let Some(n) = name {
                            categories.insert(n.clone(), SoundCategory {
                                name: n,
                                soundeffects: effects,
                                path: file_path.to_string(),
                                range: ass.key_range.clone(),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
