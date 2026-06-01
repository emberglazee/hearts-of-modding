use crate::interner::InternedStr;
use crate::ast;
use crate::parser;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MusicAsset {
    pub name: String,
    pub file: String,
    pub path: InternedStr,
    pub range: ast::Range,
}

#[derive(Debug, Clone)]
pub struct MusicStation {
    pub name: String,
    pub path: InternedStr,
    pub range: ast::Range,
}

#[derive(Debug, Clone)]
pub struct Song {
    pub name: String,
    pub path: InternedStr,
    pub range: ast::Range,
}

pub struct MusicScanResult {
    pub assets: HashMap<String, MusicAsset>,
    pub stations: HashMap<String, MusicStation>,
    pub songs: HashMap<String, Song>,
}

pub fn scan_music<F>(roots: &[std::path::PathBuf], filter: &F) -> MusicScanResult
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut assets = HashMap::new();
    let mut stations = HashMap::new();
    let mut songs = HashMap::new();

    for root in roots {
        crate::fs_util::walk_and_parse_files(
            &root.join("music"),
            &["asset", "txt"],
            filter,
            |path, content| {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext == "asset" {
                    let (script, _) = parser::parse_script(&content);
                    find_assets_in_entries(&script.entries, &path.to_string_lossy(), &mut assets);
                } else if ext == "txt" {
                    let (script, _) = parser::parse_script(&content);
                    find_stations_and_songs_in_entries(
                        &script.entries,
                        &path.to_string_lossy(),
                        &mut stations,
                        &mut songs,
                    );
                }
            },
        );
    }

    MusicScanResult {
        assets,
        stations,
        songs,
    }
}

pub(crate) fn find_assets_in_entries(
    entries: &[ast::Entry],
    file_path: &str,
    map: &mut HashMap<String, MusicAsset>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            if ass.key.eq_ignore_ascii_case("music") {
                if let ast::Value::Block(details) = &ass.value.value {
                    let mut name = None;
                    let mut file = None;

                    for detail in details {
                        if let ast::Entry::Assignment(d_ass) = detail {
                            let d_key = d_ass.key.to_ascii_lowercase();
                            if d_key == "name" {
                                if let ast::Value::String(s) = &d_ass.value.value {
                                    name = Some(s.clone());
                                }
                            } else if d_key == "file" {
                                if let ast::Value::String(s) = &d_ass.value.value {
                                    file = Some(s.clone());
                                }
                            }
                        }
                    }

                    if let (Some(n), Some(f)) = (name, file) {
                        map.insert(
                            n.clone(),
                            MusicAsset {
                                name: n,
                                file: f,
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

pub(crate) fn find_stations_and_songs_in_entries(
    entries: &[ast::Entry],
    file_path: &str,
    stations: &mut HashMap<String, MusicStation>,
    songs: &mut HashMap<String, Song>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let key_lower = ass.key.to_ascii_lowercase();
            if key_lower == "music_station" {
                if let ast::Value::String(name) = &ass.value.value {
                    stations.insert(
                        name.clone(),
                        MusicStation {
                            name: name.clone(),
                            path: std::sync::Arc::from(file_path),
                            range: ass.key_range.clone(),
                        },
                    );
                }
            } else if key_lower == "music" {
                if let ast::Value::Block(details) = &ass.value.value {
                    for detail in details {
                        if let ast::Entry::Assignment(d_ass) = detail {
                            if d_ass.key.eq_ignore_ascii_case("song") {
                                if let ast::Value::String(name) = &d_ass.value.value {
                                    songs.insert(
                                        name.clone(),
                                        Song {
                                            name: name.clone(),
                                            path: std::sync::Arc::from(file_path),
                                            range: d_ass.key_range.clone(),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_scan_music() {
        let music_dir = PathBuf::from("music_test_dir");
        let _ = fs::remove_dir_all(&music_dir);
        fs::create_dir_all(music_dir.join("music")).unwrap();

        let asset_content = r#"
            music = {
                name = test_song
                file = "test.ogg"
            }
        "#;
        fs::write(music_dir.join("music/test.asset"), asset_content).unwrap();

        let txt_content = r#"
            music_station = "test_station"
            music = {
                song = test_song
            }
        "#;
        fs::write(music_dir.join("music/test.txt"), txt_content).unwrap();

        let result = scan_music(std::slice::from_ref(&music_dir), &|_| false);

        assert!(result.assets.contains_key("test_song"));
        assert_eq!(result.assets.get("test_song").unwrap().file, "test.ogg");

        assert!(result.stations.contains_key("test_station"));
        assert!(result.songs.contains_key("test_song"));

        let _ = fs::remove_dir_all(&music_dir);
    }

    #[test]
    fn test_scan_music_hom_example() {
        let music_dir = PathBuf::from("music_hom_test");
        let _ = fs::remove_dir_all(&music_dir);
        fs::create_dir_all(music_dir.join("music")).unwrap();

        let asset_content = r#"
            ### Hearts of Minecraft Soundtrack ###
            music = {
                name = Makai_Symphony-Izanagi_izanami
                file = "Makai_Symphony-Izanagi_Izanami.ogg"
                volume = 0.65
            }
        "#;
        fs::write(music_dir.join("music/_HoM_soundtrack.asset"), asset_content).unwrap();

        let txt_content = r#"
            ### Hearts of Minecraft Soundtrack ###
            music_station = "base_music"
            music = {
                song = Makai_Symphony-Izanagi_izanami
                chance = {
                    factor = 1
                    modifier = {
                        original_tag = ENC
                        factor = 3
                    }
                }
            }
        "#;
        fs::write(music_dir.join("music/HoM_songs.txt"), txt_content).unwrap();

        let result = scan_music(std::slice::from_ref(&music_dir), &|_| false);

        assert!(result.assets.contains_key("Makai_Symphony-Izanagi_izanami"));
        assert_eq!(
            result
                .assets
                .get("Makai_Symphony-Izanagi_izanami")
                .unwrap()
                .file,
            "Makai_Symphony-Izanagi_Izanami.ogg"
        );

        assert!(result.stations.contains_key("base_music"));
        assert!(result.songs.contains_key("Makai_Symphony-Izanagi_izanami"));

        let _ = fs::remove_dir_all(&music_dir);
    }
}
