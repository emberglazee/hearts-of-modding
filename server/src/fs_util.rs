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
}
