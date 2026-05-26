use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::ast;

/// HOI4 country tags: 3 chars, first char uppercase alphabetic, rest uppercase
/// alphanumeric. Reserved words (NOT, AND, TAG, OOB, LOG, NUM, RED) and
/// entirely-numeric tags are not valid.
/// Examples: GER, D01, SOV, B42, USA
const RESERVED_TAGS: [&str; 7] = ["NOT", "AND", "TAG", "OOB", "LOG", "NUM", "RED"];

fn is_valid_tag(s: &str) -> bool {
    if s.len() != 3 {
        return false;
    }
    let bytes = s.as_bytes();
    bytes[0].is_ascii_alphabetic()
        && bytes[0].is_ascii_uppercase()
        && bytes[1].is_ascii_alphanumeric()
        && bytes[2].is_ascii_alphanumeric()
        && !RESERVED_TAGS.contains(&s)
        && !(bytes[1].is_ascii_digit() && bytes[2].is_ascii_digit()) // not entirely numeric (already covered by first char check but explicit)
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CountryTag {
    pub tag: String,
    pub name: String,
    pub path: String,
    pub range: ast::Range,
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
                        for (line_idx, line) in content.lines().enumerate() {
                            let line = line.trim();
                            if line.is_empty() || line.starts_with('#') {
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
                                        path: source_path.clone(),
                                        range: ast::Range {
                                            start_line: line_no,
                                            start_col: 0,
                                            end_line: line_no,
                                            end_col: 0,
                                        },
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // Source 2: common/countries/ directory listing (TAG - Name.txt)
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
                    if let Some(stem) = file_path.file_stem() {
                        let stem = stem.to_string_lossy();
                        if stem.len() >= 4 && stem.as_bytes()[3] == b'-' {
                            let tag = stem[..3].to_uppercase();
                            if is_valid_tag(&tag) {
                                let name = stem[4..].trim().to_string();
                                let source_path = file_path.to_string_lossy().to_string();
                                tags.entry(tag.clone()).or_insert_with(|| CountryTag {
                                    tag,
                                    name,
                                    path: source_path.clone(),
                                    range: ast::Range {
                                        start_line: 0,
                                        start_col: 0,
                                        end_line: 0,
                                        end_col: 0,
                                    },
                                });
                            }
                        }
                    }
                }
            }
        }

        // Source 3: history/countries/ directory listing (TAG - Name.txt)
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
                        if stem.len() >= 4 && stem.as_bytes()[3] == b'-' {
                            let tag = stem[..3].to_uppercase();
                            if is_valid_tag(&tag) {
                                let name = stem[4..].trim().to_string();
                                let source_path = file_path.to_string_lossy().to_string();
                                tags.entry(tag.clone()).or_insert_with(|| CountryTag {
                                    tag,
                                    name,
                                    path: source_path.clone(),
                                    range: ast::Range {
                                        start_line: 0,
                                        start_col: 0,
                                        end_line: 0,
                                        end_col: 0,
                                    },
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

fn extract_country_name(s: &str) -> String {
    // Try to extract name from paths like "countries/GER - Germany.txt" or "countries/GER-Germany.txt"
    if let Some(slash_pos) = s.rfind('/') {
        let file = &s[slash_pos + 1..];
        if let Some(dot_pos) = file.rfind('.') {
            let stem = &file[..dot_pos];
            if stem.len() >= 4 && stem.as_bytes()[3] == b'-' {
                return stem[4..].trim().to_string();
            }
            if stem.len() > 3 {
                // Try without the tag prefix separator
                return stem[3..].trim_matches('-').trim().to_string();
            }
        }
    }
    String::new()
}
