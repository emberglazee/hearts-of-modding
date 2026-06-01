use crate::interner::InternedStr;
use crate::ast;
use crate::parser;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Sound {
    pub name: String,
    pub file: String,
    pub path: InternedStr,
    pub range: ast::Range,
}

#[derive(Debug, Clone)]
pub struct SoundEffect {
    pub name: String,
    pub sounds: Vec<String>,
    pub path: InternedStr,
    pub range: ast::Range,
}

#[derive(Debug, Clone)]
pub struct Falloff {
    pub name: String,
    pub path: InternedStr,
    pub range: ast::Range,
}

#[derive(Debug, Clone)]
pub struct SoundCategory {
    pub name: String,
    pub soundeffects: Vec<String>,
    pub path: InternedStr,
    pub range: ast::Range,
}

pub struct SoundScanResult {
    pub sounds: HashMap<String, Sound>,
    pub sound_effects: HashMap<String, SoundEffect>,
    pub falloffs: HashMap<String, Falloff>,
    pub categories: HashMap<String, SoundCategory>,
}

fn process_sound_file(
    path: &std::path::Path,
    content: String,
    sounds: &mut HashMap<String, Sound>,
    sound_effects: &mut HashMap<String, SoundEffect>,
    falloffs: &mut HashMap<String, Falloff>,
    categories: &mut HashMap<String, SoundCategory>,
) {
    let (script, _) = parser::parse_script(&content);
    find_sound_definitions(
        &script.entries,
        &path.to_string_lossy(),
        sounds,
        sound_effects,
        falloffs,
        categories,
    );
}

fn scan_sound_dir<F>(
    dir_path: &std::path::Path,
    filter: &F,
    sounds: &mut HashMap<String, Sound>,
    sound_effects: &mut HashMap<String, SoundEffect>,
    falloffs: &mut HashMap<String, Falloff>,
    categories: &mut HashMap<String, SoundCategory>,
) where
    F: Fn(&std::path::Path) -> bool,
{
    crate::fs_util::walk_and_parse_files(dir_path, &["asset"], filter, |path, content| {
        process_sound_file(path, content, sounds, sound_effects, falloffs, categories);
    });
}

pub fn scan_sounds<F>(roots: &[PathBuf], filter: &F) -> SoundScanResult
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut sounds = HashMap::new();
    let mut sound_effects = HashMap::new();
    let mut falloffs = HashMap::new();
    let mut categories = HashMap::new();

    for root in roots {
        let sound_dir = root.join("sound");
        if sound_dir.exists() && !filter(&sound_dir) {
            scan_sound_dir(
                &sound_dir,
                filter,
                &mut sounds,
                &mut sound_effects,
                &mut falloffs,
                &mut categories,
            );
        }

        // Also scan integrated_dlc/*/sound/ and dlc/*/sound/ directories for vanilla sound effects
        for dlc_root_name in ["integrated_dlc", "dlc"] {
            let dlc_dir = root.join(dlc_root_name);
            if dlc_dir.exists() && !filter(&dlc_dir) {
                if let Ok(dlc_entries) = std::fs::read_dir(&dlc_dir) {
                    for dlc_entry in dlc_entries.flatten() {
                        let dlc_path = dlc_entry.path();
                        if dlc_path.is_dir() {
                            let dlc_sound_dir = dlc_path.join("sound");
                            if dlc_sound_dir.exists() && !filter(&dlc_sound_dir) {
                                scan_sound_dir(
                                    &dlc_sound_dir,
                                    filter,
                                    &mut sounds,
                                    &mut sound_effects,
                                    &mut falloffs,
                                    &mut categories,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    SoundScanResult {
        sounds,
        sound_effects,
        falloffs,
        categories,
    }
}

pub(crate) fn find_sound_definitions(
    entries: &[ast::Entry],
    file_path: &str,
    sounds: &mut HashMap<String, Sound>,
    sound_effects: &mut HashMap<String, SoundEffect>,
    falloffs: &mut HashMap<String, Falloff>,
    categories: &mut HashMap<String, SoundCategory>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let key = ass.key.as_str();
            if key.eq_ignore_ascii_case("sound") {
                if let ast::Value::Block(details) = &ass.value.value {
                    let mut name = None;
                    let mut file = None;
                    for detail in details {
                        if let ast::Entry::Assignment(d_ass) = detail {
                            let d_key = d_ass.key.as_str();
                            if d_key.eq_ignore_ascii_case("name") {
                                if let ast::Value::String(s) = &d_ass.value.value {
                                    name = Some(s.clone());
                                }
                            } else if d_key.eq_ignore_ascii_case("file") {
                                if let ast::Value::String(s) = &d_ass.value.value {
                                    file = Some(s.clone());
                                }
                            }
                        }
                    }
                    if let (Some(n), Some(f)) = (name, file) {
                        sounds.insert(
                            n.clone(),
                            Sound {
                                name: n,
                                file: f,
                                path: std::sync::Arc::from(file_path),
                                range: ass.key_range.clone(),
                            },
                        );
                    }
                }
            } else if key.eq_ignore_ascii_case("soundeffect") {
                if let ast::Value::Block(details) = &ass.value.value {
                    let mut name = None;
                    let mut sounds_list = Vec::new();
                    for detail in details {
                        if let ast::Entry::Assignment(d_ass) = detail {
                            let d_key = d_ass.key.as_str();
                            if d_key.eq_ignore_ascii_case("name") {
                                if let ast::Value::String(s) = &d_ass.value.value {
                                    name = Some(s.clone());
                                }
                            } else if d_key.eq_ignore_ascii_case("sounds") {
                                if let ast::Value::Block(s_details) = &d_ass.value.value {
                                    for s_detail in s_details {
                                        if let ast::Entry::Assignment(s_ass) = s_detail {
                                            let s_key = s_ass.key.as_str();
                                            if s_key.eq_ignore_ascii_case("sound") {
                                                if let ast::Value::String(s_name) =
                                                    &s_ass.value.value
                                                {
                                                    sounds_list.push(s_name.clone());
                                                }
                                            } else if s_key.eq_ignore_ascii_case("weighted_sound") {
                                                if let ast::Value::Block(w_details) =
                                                    &s_ass.value.value
                                                {
                                                    for w_detail in w_details {
                                                        if let ast::Entry::Assignment(w_ass) =
                                                            w_detail
                                                        {
                                                            if w_ass
                                                                .key
                                                                .eq_ignore_ascii_case("sound")
                                                            {
                                                                if let ast::Value::String(s_name) =
                                                                    &w_ass.value.value
                                                                {
                                                                    sounds_list
                                                                        .push(s_name.clone());
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
                        }
                    }
                    if let Some(n) = name {
                        sound_effects.insert(
                            n.clone(),
                            SoundEffect {
                                name: n,
                                sounds: sounds_list,
                                path: std::sync::Arc::from(file_path),
                                range: ass.key_range.clone(),
                            },
                        );
                    }
                }
            } else if key.eq_ignore_ascii_case("falloff") {
                if let ast::Value::Block(details) = &ass.value.value {
                    let mut name = None;
                    for detail in details {
                        if let ast::Entry::Assignment(d_ass) = detail {
                            if d_ass.key.eq_ignore_ascii_case("name") {
                                if let ast::Value::String(s) = &d_ass.value.value {
                                    name = Some(s.clone());
                                }
                            }
                        }
                    }
                    if let Some(n) = name {
                        falloffs.insert(
                            n.clone(),
                            Falloff {
                                name: n,
                                path: std::sync::Arc::from(file_path),
                                range: ass.key_range.clone(),
                            },
                        );
                    }
                }
            } else if key.eq_ignore_ascii_case("category") {
                if let ast::Value::Block(details) = &ass.value.value {
                    let mut name = None;
                    let mut effects = Vec::new();
                    for detail in details {
                        if let ast::Entry::Assignment(d_ass) = detail {
                            let d_key = d_ass.key.as_str();
                            if d_key.eq_ignore_ascii_case("name") {
                                if let ast::Value::String(s) = &d_ass.value.value {
                                    name = Some(s.clone());
                                }
                            } else if d_key.eq_ignore_ascii_case("soundeffects") {
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
                        }
                    }
                    if let Some(n) = name {
                        categories.insert(
                            n.clone(),
                            SoundCategory {
                                name: n,
                                soundeffects: effects,
                                path: std::sync::Arc::from(file_path),
                                range: ass.key_range.clone(),
                            },
                        );
                    }
                }
            }
        }
    }
}
