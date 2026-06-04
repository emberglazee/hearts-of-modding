use std::collections::HashMap;
use std::path::PathBuf;

/// Represents game defines loaded from common/defines/*.lua
#[derive(Debug, Clone, Default)]
pub struct GameDefines {
    /// Numeric defines keyed by their Lua name (e.g. "NCouncil.POLITICAL_ADVISOR_MAX_LEVEL")
    pub defines: HashMap<String, f64>,
}

/// Scan defines files from game/mod directories
pub fn scan_defines<F>(roots: &[PathBuf], filter: &F) -> GameDefines
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut defines = GameDefines::default();

    for root in roots {
        crate::utils::fs_util::walk_and_parse_files(
            &root.join("common/defines"),
            &["lua"],
            filter,
            |_path, content| {
                parse_defines_lua(&content, &mut defines);
            },
        );
    }

    defines
}

/// Simple Lua parser for defines files
/// This is a basic implementation that handles common patterns in HOI4 defines
pub(crate) fn parse_defines_lua(content: &str, defines: &mut GameDefines) {
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Skip comments and empty lines
        if line.starts_with("--") || line.is_empty() {
            i += 1;
            continue;
        }

        // Look for general numeric defines
        if let Some((key, value)) = parse_define_assignment(line) {
            defines.defines.insert(key, value);
        }

        i += 1;
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
}
