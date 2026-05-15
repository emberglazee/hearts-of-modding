use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Represents game defines loaded from common/defines/*.lua
#[derive(Debug, Clone)]
pub struct GameDefines {
    /// Max skill levels for different character types
    pub max_skill_levels: HashMap<String, i32>,
    /// Other numeric defines
    pub defines: HashMap<String, f64>,
}

impl GameDefines {
    pub fn new() -> Self {
        Self {
            max_skill_levels: HashMap::new(),
            defines: HashMap::new(),
        }
    }

    /// Get max skill level for a character type (default to 5 if not found)
    pub fn get_max_skill(&self, character_type: &str) -> i32 {
        self.max_skill_levels
            .get(character_type)
            .copied()
            .unwrap_or(5)
    }
}

impl Default for GameDefines {
    fn default() -> Self {
        Self::new()
    }
}

/// Scan defines files from game/mod directories
pub fn scan_defines<F>(roots: &[PathBuf], filter: &F) -> GameDefines
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut defines = GameDefines::new();

    // Set default max skill levels (HOI4 defaults)
    defines
        .max_skill_levels
        .insert("field_marshal".to_string(), 5);
    defines
        .max_skill_levels
        .insert("corps_commander".to_string(), 5);
    defines
        .max_skill_levels
        .insert("navy_leader".to_string(), 5);
    defines.max_skill_levels.insert("operative".to_string(), 3);

    for root in roots {
        let dir = root.join("common/defines");
        if dir.exists() {
            scan_defines_directory(&dir, &mut defines, filter);
        }
    }

    defines
}

fn scan_defines_directory<F>(dir_path: &Path, defines: &mut GameDefines, filter: &F)
where
    F: Fn(&std::path::Path) -> bool,
{
    if filter(dir_path) {
        return;
    }
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_defines_directory(&path, defines, filter);
            } else if path.extension().map_or(false, |ext| ext == "lua") {
                if filter(&path) {
                    continue;
                }
                if let Ok(content) = fs::read_to_string(&path) {
                    parse_defines_lua(&content, defines);
                }
            }
        }
    }
}

/// Simple Lua parser for defines files
/// This is a basic implementation that handles common patterns in HOI4 defines
fn parse_defines_lua(content: &str, defines: &mut GameDefines) {
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Skip comments and empty lines
        if line.starts_with("--") || line.is_empty() {
            i += 1;
            continue;
        }

        // Look for MAX_SKILL_LEVEL patterns
        if line.contains("MAX_SKILL_LEVEL") || line.contains("MAX_SKILL") {
            if let Some(value) = extract_number_from_line(line) {
                // Try to determine character type from context
                let context = if i > 0 { lines[i - 1] } else { "" };

                if context.contains("FIELD_MARSHAL") || line.contains("FIELD_MARSHAL") {
                    defines
                        .max_skill_levels
                        .insert("field_marshal".to_string(), value);
                } else if context.contains("CORPS_COMMANDER") || line.contains("CORPS_COMMANDER") {
                    defines
                        .max_skill_levels
                        .insert("corps_commander".to_string(), value);
                } else if context.contains("NAVY") || line.contains("NAVY") {
                    defines
                        .max_skill_levels
                        .insert("navy_leader".to_string(), value);
                } else if context.contains("OPERATIVE") || line.contains("OPERATIVE") {
                    defines
                        .max_skill_levels
                        .insert("operative".to_string(), value);
                }
            }
        }

        // Look for general numeric defines
        if let Some((key, value)) = parse_define_assignment(line) {
            defines.defines.insert(key, value);
        }

        i += 1;
    }
}

/// Extract a number from a line like "MAX_SKILL_LEVEL = 5" or "MAX_SKILL_LEVEL = 5,"
fn extract_number_from_line(line: &str) -> Option<i32> {
    // Find the = sign
    if let Some(eq_pos) = line.find('=') {
        let value_part = &line[eq_pos + 1..];
        // Remove trailing comma, whitespace, and comments
        let value_str = value_part
            .split("--")
            .next()
            .unwrap_or(value_part)
            .trim()
            .trim_end_matches(',')
            .trim();

        value_str.parse::<i32>().ok()
    } else {
        None
    }
}

/// Parse a define assignment like "SOME_VALUE = 123.45"
fn parse_define_assignment(line: &str) -> Option<(String, f64)> {
    if let Some(eq_pos) = line.find('=') {
        let key = line[..eq_pos].trim();
        let value_part = &line[eq_pos + 1..];

        // Skip if it's a table assignment
        if value_part.trim().starts_with('{') {
            return None;
        }

        let value_str = value_part
            .split("--")
            .next()
            .unwrap_or(value_part)
            .trim()
            .trim_end_matches(',')
            .trim();

        if let Ok(value) = value_str.parse::<f64>() {
            return Some((key.to_string(), value));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_number_from_line() {
        assert_eq!(extract_number_from_line("MAX_SKILL_LEVEL = 5"), Some(5));
        assert_eq!(extract_number_from_line("MAX_SKILL_LEVEL = 5,"), Some(5));
        assert_eq!(
            extract_number_from_line("MAX_SKILL_LEVEL = 5, -- comment"),
            Some(5)
        );
        assert_eq!(
            extract_number_from_line("    MAX_SKILL_LEVEL = 10    "),
            Some(10)
        );
    }

    #[test]
    fn test_parse_define_assignment() {
        assert_eq!(
            parse_define_assignment("SOME_VALUE = 123.45"),
            Some(("SOME_VALUE".to_string(), 123.45))
        );
        assert_eq!(
            parse_define_assignment("SOME_VALUE = 123.45,"),
            Some(("SOME_VALUE".to_string(), 123.45))
        );
        assert_eq!(
            parse_define_assignment("SOME_VALUE = 123.45, -- comment"),
            Some(("SOME_VALUE".to_string(), 123.45))
        );
        assert_eq!(parse_define_assignment("SOME_TABLE = {"), None);
    }

    #[test]
    fn test_default_max_skills() {
        let defines = GameDefines::new();
        assert_eq!(defines.get_max_skill("field_marshal"), 5);
        assert_eq!(defines.get_max_skill("unknown_type"), 5);
    }
}
