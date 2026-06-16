#![allow(dead_code)]
use crate::data::interner::InternedStr;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::parser::ast;

/// HOI4 country tags: 3 chars, first char uppercase alphabetic, rest uppercase
/// alphanumeric. Reserved words (NOT, AND, TAG, OOB, LOG, NUM, RED) and
/// entirely-numeric tags are not valid.
/// Examples: GER, D01, SOV, B42, USA
const RESERVED_TAGS: [&str; 7] = ["NOT", "AND", "TAG", "OOB", "LOG", "NUM", "RED"];

pub(crate) fn is_valid_tag(s: &str) -> bool {
    if s.len() != 3 {
        return false;
    }
    let bytes = s.as_bytes();
    bytes[0].is_ascii_alphabetic()
        && bytes[0].is_ascii_uppercase()
        && bytes[1].is_ascii_alphanumeric()
        && bytes[2].is_ascii_alphanumeric()
        && !RESERVED_TAGS.contains(&s)
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CountryTag {
    pub tag: String,
    pub name: String,
    pub path: InternedStr,
    pub range: ast::Range,
    pub dynamic: bool,
}

pub fn scan_country_tags<F>(roots: &[PathBuf], filter: &F) -> HashMap<String, CountryTag>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut tags = HashMap::new();

    for root in roots {
        // Source 1: common/country_tags/ files (TAG = "countries/TAG - Name.txt")
        let ct_dir = root.join("common/country_tags");
        if ct_dir.exists() {
            let dir = fs::read_dir(&ct_dir);
            if let Ok(dir) = dir {
                for entry in dir.flatten() {
                    let file_path = entry.path();
                    if file_path.extension().is_none_or(|e| e != "txt") {
                        continue;
                    }
                    if filter(&file_path) {
                        continue;
                    }
                    if let Ok(content) = fs::read_to_string(&file_path) {
                        let mut dynamic_mode = false;
                        for (line_idx, line) in content.lines().enumerate() {
                            let line = line.trim();
                            if line.is_empty() || line.starts_with('#') {
                                continue;
                            }
                            // Check for dynamic_tags = yes
                            if line.to_ascii_lowercase().starts_with("dynamic_tags") {
                                dynamic_mode = line.to_ascii_lowercase().contains("= yes");
                                continue;
                            }
                            // Format: TAG = "countries/TAG - Name.txt"
                            if let Some(eq_pos) = line.find('=') {
                                let tag = line[..eq_pos].trim().to_uppercase();
                                if is_valid_tag(&tag) {
                                    let rest = line[eq_pos + 1..].trim().trim_matches('"');
                                    let name = extract_country_name(rest);
                                    let source_path = file_path.to_string_lossy().to_string();
                                    let line_no = line_idx as u32;
                                    tags.entry(tag.clone()).or_insert_with(|| CountryTag {
                                        tag,
                                        name,
                                        path: source_path.clone().into(),
                                        range: ast::Range {
                                            start_line: line_no,
                                            start_col: 0,
                                            end_line: line_no,
                                            end_col: 0,
                                        },
                                        dynamic: dynamic_mode,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // Source 2: common/countries/ directory listing
        // Files are named like "Afghanistan.txt" — no tag in filename.
        // Tags are extracted from file content via inline parsing.
        let countries_dir = root.join("common/countries");
        if countries_dir.exists() {
            let dir = fs::read_dir(&countries_dir);
            if let Ok(dir) = dir {
                for entry in dir.flatten() {
                    let file_path = entry.path();
                    if file_path.extension().is_none_or(|e| e != "txt") {
                        continue;
                    }
                    if filter(&file_path) {
                        continue;
                    }
                    if let Ok(content) = fs::read_to_string(&file_path) {
                        let path_str = file_path.to_string_lossy().to_string();
                        let (script, _) = crate::parser::parser::parse_script(&content);
                        let source = &script.source;
                        for entry in &script.entries {
                            if let ast::Entry::Assignment(ass) = entry {
                                let tag = ass.key_text(source).to_string();
                                tags.entry(tag.clone()).or_insert_with(|| CountryTag {
                                    tag,
                                    name: String::new(),
                                    path: path_str.clone().into(),
                                    range: ass.key_range.clone(),
                                    dynamic: false,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Source 3: history/countries/ directory listing
        // Filename format: First 3 chars = tag, rest is arbitrary (wiki convention).
        let hist_dir = root.join("history/countries");
        if hist_dir.exists() {
            let dir = fs::read_dir(&hist_dir);
            if let Ok(dir) = dir {
                for entry in dir.flatten() {
                    let file_path = entry.path();
                    if file_path.extension().is_none_or(|e| e != "txt") {
                        continue;
                    }
                    if filter(&file_path) {
                        continue;
                    }
                    if let Some(stem) = file_path.file_stem() {
                        let stem = stem.to_string_lossy();
                        if stem.len() >= 3 {
                            let tag = stem[..3].to_uppercase();
                            if is_valid_tag(&tag) {
                                let name = extract_country_name_from_filename(&stem);
                                let source_path = file_path.to_string_lossy().to_string();
                                tags.entry(tag.clone()).or_insert_with(|| CountryTag {
                                    tag,
                                    name,
                                    path: source_path.clone().into(),
                                    range: ast::Range {
                                        start_line: 0,
                                        start_col: 0,
                                        end_line: 0,
                                        end_col: 0,
                                    },
                                    dynamic: false,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    tags
}

/// Scan a pre-determined list of country tag files.
/// Determines parsing strategy by the file's path:
/// - Files under "common/country_tags" are parsed as `TAG = "path"` line format.
/// - Files under "common/countries" — filenames like "Afghanistan.txt" have no tag;
///   the content is parsed as HOI4 script and top-level assignment keys become tags.
/// - Files under "history/countries" — first 3 chars of filename are the tag.
pub fn scan_country_tag_files<F>(files: &[PathBuf], filter: &F) -> HashMap<String, CountryTag>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut tags = HashMap::new();

    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let path_str = path.to_string_lossy();
        let path_lower = path_str.to_ascii_lowercase().replace('\\', "/");

        if path_lower.contains("common/country_tags") {
            // Source 1: TAG = "countries/TAG - Name.txt" format
            let mut dynamic_mode = false;
            for (line_idx, line) in content.lines().enumerate() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if line.to_ascii_lowercase().starts_with("dynamic_tags") {
                    dynamic_mode = line.to_ascii_lowercase().contains("= yes");
                    continue;
                }
                if let Some(eq_pos) = line.find('=') {
                    let tag = line[..eq_pos].trim().to_uppercase();
                    if is_valid_tag(&tag) {
                        let rest = line[eq_pos + 1..].trim().trim_matches('"');
                        let name = extract_country_name(rest);
                        let line_no = line_idx as u32;
                        tags.entry(tag.clone()).or_insert_with(|| CountryTag {
                            tag,
                            name,
                            path: path_str.to_string().into(),
                            range: ast::Range {
                                start_line: line_no,
                                start_col: 0,
                                end_line: line_no,
                                end_col: 0,
                            },
                            dynamic: dynamic_mode,
                        });
                    }
                }
            }
        } else if path_lower.contains("common/countries")
            || path_lower.contains("history/countries")
        {
            // Source 2/3: parse content for inline tag definitions.
            // For common/countries/ files (e.g. "cosmetic.txt", "Afghanistan.txt"),
            // the tag is only in the content (CYC_OSTLAND = { ... }, AFG = { ... }).
            // For history/countries/, first try filename extraction (first 3 chars = tag),
            // then fall back to content parsing.
            if path_lower.contains("history/countries") {
                if let Some(stem) = path.file_stem() {
                    let stem = stem.to_string_lossy();
                    if stem.len() >= 3 {
                        let tag = stem[..3].to_uppercase();
                        if is_valid_tag(&tag) {
                            let name = extract_country_name_from_filename(&stem);
                            tags.entry(tag.clone()).or_insert_with(|| CountryTag {
                                tag,
                                name,
                                path: path_str.to_string().into(),
                                range: ast::Range {
                                    start_line: 0,
                                    start_col: 0,
                                    end_line: 0,
                                    end_col: 0,
                                },
                                dynamic: false,
                            });
                            return;
                        }
                    }
                }
            }

            // Fallback: parse content for inline tag definitions.
            // Handles common/countries/*.txt files (name-only or cosmetic)
            // and history/countries/*.txt files with non-standard filenames.
            let (script, _) = crate::parser::parser::parse_script(&content);
            let source = &script.source;
            for entry in &script.entries {
                if let ast::Entry::Assignment(ass) = entry {
                    let tag = ass.key_text(source).to_string();
                    tags.entry(tag.clone()).or_insert_with(|| CountryTag {
                        tag,
                        name: String::new(),
                        path: path_str.to_string().into(),
                        range: ass.key_range.clone(),
                        dynamic: false,
                    });
                }
            }
        }
    });

    tags
}

/// Extract country name from a path like "countries/GER - Germany.txt" or "GER - Germany.txt"
pub(crate) fn extract_country_name(s: &str) -> String {
    // Find the filename portion (after last /)
    let file = if let Some(slash_pos) = s.rfind('/') {
        &s[slash_pos + 1..]
    } else {
        s
    };
    if let Some(dot_pos) = file.rfind('.') {
        let stem = &file[..dot_pos];
        if stem.len() >= 4
            && stem.as_bytes()[3] == b' '
            && stem.len() > 5
            && stem.as_bytes()[4] == b'-'
        {
            return stem[5..].trim().to_string();
        }
        if stem.len() > 3 {
            return stem[3..]
                .trim_matches(|c: char| c == '-' || c == '_' || c == ' ')
                .to_string();
        }
    }
    String::new()
}

/// Extract readable country name from a history/countries filename stem.
/// Handles "TAG - Name", "TAG-Name", "TAG_Name", and bare "TAG".
fn extract_country_name_from_filename(stem: &str) -> String {
    if stem.len() > 4 && stem.as_bytes()[3] == b' ' && stem.as_bytes()[4] == b'-' {
        // "TAG - Name" format
        stem[5..].trim().to_string()
    } else if stem.len() > 4 && stem.as_bytes()[3] == b'-' {
        // "TAG-Name" format
        stem[4..].trim().to_string()
    } else if stem.len() > 3 {
        // "TAG_Name" or similar
        stem[3..]
            .trim_matches(|c: char| c == '-' || c == '_' || c == ' ')
            .to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::Path;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn temp_dir() -> std::path::PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("hom_country_test_{}_{}", std::process::id(), id))
    }

    #[test]
    fn test_extract_inline_tags_from_cosmetic_file() {
        let dir = temp_dir();
        std::fs::create_dir_all(dir.join("common/countries")).unwrap();
        let file_path = dir.join("common/countries/cosmetic.txt");
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"# Grey / White Colors\nCYC_WHITE = {\n\tcolor = rgb { 230 230 230 }\n\tcolor_ui = rgb { 255 255 255 }\n}\nCYC_OSTLAND = {\n\tcolor = rgb { 86 78 112 }\n\tcolor_ui = rgb { 86 78 112 }\n}\n").unwrap();
        drop(file);

        let noop_filter = |_: &Path| false;
        let tags = scan_country_tag_files(&[file_path], &noop_filter);

        assert!(
            tags.contains_key("CYC_WHITE"),
            "Should extract CYC_WHITE from inline content"
        );
        assert!(
            tags.contains_key("CYC_OSTLAND"),
            "Should extract CYC_OSTLAND from inline content"
        );
        assert_eq!(tags.len(), 2);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_extract_tag_from_history_countries_file() {
        // history/countries/ files: first 3 chars of filename = tag
        let dir = temp_dir();
        std::fs::create_dir_all(dir.join("history/countries")).unwrap();
        let file_path = dir.join("history/countries/GER - Germany.txt");
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"set_capital = 0\n").unwrap();
        drop(file);

        let noop_filter = |_: &Path| false;
        let tags = scan_country_tag_files(&[file_path], &noop_filter);

        assert!(
            tags.contains_key("GER"),
            "Should extract GER from history/countries/ filename"
        );
        assert_eq!(tags["GER"].name, "Germany");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_extract_tag_from_nameonly_countries_file() {
        // common/countries/ files like "Afghanistan.txt" — tag from content
        let dir = temp_dir();
        std::fs::create_dir_all(dir.join("common/countries")).unwrap();
        let file_path = dir.join("common/countries/Afghanistan.txt");
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"AFG = {\n\tcolor = rgb { 0 140 0 }\n}\n")
            .unwrap();
        drop(file);

        let noop_filter = |_: &Path| false;
        let tags = scan_country_tag_files(&[file_path], &noop_filter);

        assert!(
            tags.contains_key("AFG"),
            "Should extract AFG from inline content"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_history_file_with_compact_separator() {
        // "TAG-Name" and "TAG_Name" formats also valid per wiki
        let dir = temp_dir();
        std::fs::create_dir_all(dir.join("history/countries")).unwrap();
        let file_path = dir.join("history/countries/SCO-Bahrain.txt");
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"color = rgb { 0 0 0 }\n").unwrap();
        drop(file);

        let noop_filter = |_: &Path| false;
        let tags = scan_country_tag_files(&[file_path], &noop_filter);

        assert!(
            tags.contains_key("SCO"),
            "Should extract SCO from history/countries/ compact filename"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_extract_country_name_from_filename_helpers() {
        assert_eq!(
            extract_country_name_from_filename("GER - Germany"),
            "Germany"
        );
        assert_eq!(extract_country_name_from_filename("SCO-Bahrain"), "Bahrain");
        assert_eq!(extract_country_name_from_filename("SCO_Bahrain"), "Bahrain");
        assert_eq!(extract_country_name_from_filename("SCO"), "");
        assert_eq!(
            extract_country_name_from_filename("ABK - Abkhazia"),
            "Abkhazia"
        );
    }

    #[test]
    fn test_extract_country_name_from_path() {
        assert_eq!(
            extract_country_name("countries/GER - Germany.txt"),
            "Germany"
        );
        assert_eq!(extract_country_name("countries/SCO-Bahrain.txt"), "Bahrain");
        assert_eq!(extract_country_name("countries/SCO_Bahrain.txt"), "Bahrain");
        assert_eq!(extract_country_name(""), "");
        assert_eq!(extract_country_name("GER - Germany.txt"), "Germany");
    }
}
