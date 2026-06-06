use std::path::{Path, PathBuf};

/// Recursively walk a directory, collecting files whose extension matches
/// one of `extensions` and for which `ignore_filter` returns `false`.
/// If `skip_git` is `true`, `.git` directories are not descended into.
pub fn collect_files<F>(
    root: &Path,
    extensions: &[&str],
    ignore_filter: F,
    skip_git: bool,
) -> Vec<PathBuf>
where
    F: Fn(&Path) -> bool,
{
    let mut matching_files = Vec::new();
    let mut dirs_to_check = vec![root.to_path_buf()];

    while let Some(current_dir) = dirs_to_check.pop() {
        if ignore_filter(&current_dir) {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if !skip_git || path.file_name().is_none_or(|n| n != ".git") {
                        dirs_to_check.push(path);
                    }
                } else if let Some(ext) = path.extension() {
                    if extensions.contains(&ext.to_string_lossy().as_ref()) {
                        if !ignore_filter(&path) {
                            matching_files.push(path);
                        }
                    }
                }
            }
        }
    }

    matching_files
}

/// Recursively walk a directory, calling `processor` for each file whose extension
/// matches one in `extensions` and for which `ignore_filter` returns `false`.
/// Directories for which `ignore_filter` returns `true` are also skipped entirely.
/// This is the consolidated traversal helper that scanners should use instead of
/// duplicating the `fs::read_dir` + stack pattern.
pub fn walk_and_parse_files<F, P>(
    dir_path: &Path,
    extensions: &[&str],
    ignore_filter: &F,
    mut processor: P,
) where
    F: Fn(&Path) -> bool,
    P: FnMut(&Path, String),
{
    if !dir_path.exists() || ignore_filter(dir_path) {
        return;
    }

    let mut dirs_to_check = vec![dir_path.to_path_buf()];
    while let Some(current_dir) = dirs_to_check.pop() {
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if !ignore_filter(&path) {
                        dirs_to_check.push(path);
                    }
                } else if let Some(ext) = path.extension() {
                    if extensions.contains(&ext.to_string_lossy().as_ref()) {
                        if ignore_filter(&path) {
                            continue;
                        }
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            processor(&path, content);
                        }
                    }
                }
            }
        }
    }
}

/// Parse a pre-determined list of winning files (from FileOverlay) without
/// walking a directory. Each file is read and passed to the processor only
/// if it passes the ignore filter.
///
/// This is the file-level-overlay counterpart to `walk_and_parse_files` —
/// instead of discovering files by walking a directory, the caller provides
/// the exact list of files to process (the ones that "won" the overlay).
pub fn parse_winning_files<F, P>(files: &[PathBuf], ignore_filter: &F, mut processor: P)
where
    F: Fn(&Path) -> bool,
    P: FnMut(&Path, String),
{
    for path in files {
        if ignore_filter(path) {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(path) {
            processor(path, content);
        }
    }
}

/// Escape only the regex metacharacters that commonly appear literally in filenames:
/// `(`, `)`, `[`, `]`, `{`, `}`. Unlike `regex::escape()`, this leaves `.*+?^$|\\`
/// untouched so existing regex patterns like `./directory/*` or `.*\.txt$` keep working.
pub fn escape_filename_chars(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '(' | ')' | '[' | ']' | '{' | '}' => {
                result.push('\\');
                result.push(c);
            }
            _ => result.push(c),
        }
    }
    result
}

/// Check whether a path matches any of the provided ignore regex patterns.
pub fn is_path_ignored(path: &Path, ignored: &[regex::Regex]) -> bool {
    let path_str = path.to_string_lossy();
    ignored.iter().any(|re| re.is_match(&path_str))
}

/// Fuzzy match for symbol search.
/// Returns true if `query` is empty, is a substring of `target`,
/// or all characters in `query` appear in order in `target` (case-insensitive).
pub fn fuzzy_match(query_lowercase: &str, target: &str) -> bool {
    if query_lowercase.is_empty() {
        return true;
    }

    // 1. Case-insensitive substring check without allocating target_lower
    if target.len() >= query_lowercase.len() {
        let found = target
            .as_bytes()
            .windows(query_lowercase.len())
            .any(|window| window.eq_ignore_ascii_case(query_lowercase.as_bytes()));
        if found {
            return true;
        }
    }

    // 2. Case-insensitive char-by-char subsequence check without allocating
    let mut query_chars = query_lowercase.chars();
    let mut current_query_char = query_chars.next();

    for target_char in target.chars() {
        if let Some(qc) = current_query_char {
            if target_char.to_ascii_lowercase() == qc {
                current_query_char = query_chars.next();
            }
        } else {
            return true;
        }
    }

    current_query_char.is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match() {
        assert!(fuzzy_match("", "anything"));
        assert!(fuzzy_match("test", "test"));
        assert!(fuzzy_match("test", "my_test_event"));
        assert!(fuzzy_match("mte", "my_test_event"));
        assert!(!fuzzy_match("xyz", "my_test_event"));
    }

    #[test]
    fn test_fuzzy_match_case_insensitive() {
        assert!(fuzzy_match("test", "TEST"));
        assert!(fuzzy_match("test", "MyTestEvent"));
    }

    #[test]
    fn test_is_path_ignored_with_parentheses() {
        // Patterns come from config, pre-processed via escape_filename_chars()
        // e.g., user writes "event (copy)" → compiled as "event \(copy\)"
        let escaped_pattern = regex::Regex::new(r"event \(copy\)").unwrap();
        let escaped_patterns = vec![escaped_pattern];

        // Path with literal parentheses should be ignored
        let path_with_parens = std::path::Path::new("/mod/events/event (copy).txt");
        assert!(
            is_path_ignored(path_with_parens, &escaped_patterns),
            "escaped pattern should match path with literal parentheses"
        );

        // Path without parentheses should NOT be matched
        let path_without_parens = std::path::Path::new("/mod/events/event copy.txt");
        assert!(
            !is_path_ignored(path_without_parens, &escaped_patterns),
            "escaped pattern should NOT match path without parentheses"
        );

        // Verify old behavior (unescaped) would FAIL with parens
        let unescaped_pattern = regex::Regex::new(r"event (copy)").unwrap();
        let unescaped_patterns = vec![unescaped_pattern];
        assert!(
            !is_path_ignored(path_with_parens, &unescaped_patterns),
            "unescaped regex treats ( ) as group metacharacters, missing literal parens"
        );
        // But unescaped matches path without parens
        assert!(
            is_path_ignored(path_without_parens, &unescaped_patterns),
            "unescaped regex matches path without literal parens"
        );
    }

    #[test]
    fn test_escape_filename_chars() {
        // Parens/brackets/braces get escaped
        assert_eq!(escape_filename_chars("event (copy)"), r"event \(copy\)");
        assert_eq!(escape_filename_chars("[debug] v2"), r"\[debug\] v2");
        assert_eq!(escape_filename_chars("{test}"), r"\{test\}");

        // Common regex patterns are LEFT UNTOUCHED
        assert_eq!(escape_filename_chars("./directory/*"), "./directory/*");
        assert_eq!(escape_filename_chars(r".*\.txt$"), r".*\.txt$");
        assert_eq!(escape_filename_chars("^events/test_.*"), "^events/test_.*");
        assert_eq!(escape_filename_chars("debug_v\\d+"), "debug_v\\d+");

        // Empty string stays empty
        assert_eq!(escape_filename_chars(""), "");

        // No special chars stays unchanged
        assert_eq!(escape_filename_chars("simple_path"), "simple_path");
    }

    #[test]
    fn test_is_path_ignored_with_globs_unchanged() {
        // Verify patterns like "./directory/*" still work as regex
        let pattern = regex::Regex::new(r"./directory/*").unwrap();
        let patterns = vec![pattern];
        // This matches the literal path portion (the * matches zero or more /)
        assert!(is_path_ignored(
            std::path::Path::new("./directory/"),
            &patterns
        ));
    }
}
