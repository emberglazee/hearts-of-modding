use std::path::{Path, PathBuf};

/// Determine the default Paradox mod registry path for the current OS.
///
/// # Platform paths
///
/// | OS      | Path |
/// |---------|------|
/// | Linux   | `~/.local/share/Paradox Interactive/Hearts of Iron IV/mod` |
/// | Windows | `%USERPROFILE%/Documents/Paradox Interactive/Hearts of Iron IV/mod` |
/// | macOS   | `~/Documents/Paradox Interactive/Hearts of Iron IV/mod` |
///
/// Returns `None` when the platform's home-directory variable is unset
/// or the directory doesn't exist (e.g. portable Steam installs).
pub(crate) fn default_mod_registry_path() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let path = Path::new(&home).join(".local/share/Paradox Interactive/Hearts of Iron IV/mod");
            if path.is_dir() {
                return Some(path);
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(profile) = std::env::var_os("USERPROFILE") {
            let path = Path::new(&profile).join("Documents/Paradox Interactive/Hearts of Iron IV/mod");
            if path.is_dir() {
                return Some(path);
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let path = Path::new(&home).join("Documents/Paradox Interactive/Hearts of Iron IV/mod");
            if path.is_dir() {
                return Some(path);
            }
        }
    }
    None
}

/// Extract the `name` field from a `.mod` descriptor file.
fn extract_mod_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("name=").or_else(|| trimmed.strip_prefix("name = ")) {
            let value = value.trim().trim_matches('"').trim().to_string();
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

/// Extract the `path` field from a `.mod` descriptor file and resolve it to
/// an absolute path relative to `registry_dir`.
fn extract_mod_path(content: &str, registry_dir: &Path) -> Option<PathBuf> {
    for line in content.lines() {
        let trimmed = line.trim();
        // Look for "path = " or "path=" (case-insensitive key)
        if let Some(equals_pos) = trimmed.find('=') {
            let key = trimmed[..equals_pos].trim();
            if key.eq_ignore_ascii_case("path") {
                let raw = trimmed[equals_pos + 1..].trim().trim_matches('"').trim();
                if !raw.is_empty() {
                    let p = PathBuf::from(raw);
                    if p.is_absolute() {
                        return Some(p);
                    } else {
                        // Relative to the registry directory
                        return Some(registry_dir.join(&p));
                    }
                }
            }
        }
    }
    None
}

/// Extract the `dependencies` block from a `descriptor.mod` content string.
///
/// Format: `dependencies = { "Mod Name 1" "Mod Name 2" }`
pub(crate) fn parse_dependencies(content: &str) -> Vec<String> {
    let mut deps = Vec::new();

    // Find "dependencies" or "dependencies=" or "dependencies ="
    let content_lower = content.to_lowercase();
    let deps_keyword = "dependencies";
    let mut search_start = 0;

    while let Some(keyword_pos) = content_lower[search_start..].find(deps_keyword) {
        let abs_pos = search_start + keyword_pos;
        let after_keyword = &content[abs_pos + deps_keyword.len()..].trim_start();

        // Must be followed by '=' (possibly with whitespace)
        if let Some(eq_rest) = after_keyword.strip_prefix('=') {
            let after_eq = eq_rest.trim_start();

            // Must be followed by '{'
            if let Some(block) = after_eq.strip_prefix('{') {
                // Find matching closing brace
                let mut depth: i32 = 1;
                let mut close_pos = 0;
                for (i, c) in block.char_indices() {
                    if c == '{' {
                        depth += 1;
                    } else if c == '}' {
                        depth -= 1;
                        if depth == 0 {
                            close_pos = i;
                            break;
                        }
                    }
                }
                if depth == 0 {
                    let inner = &block[..close_pos];
                    // Extract quoted strings
                    for part in inner.split('"').skip(1).step_by(2) {
                        let dep = part.trim().to_string();
                        if !dep.is_empty() {
                            deps.push(dep);
                        }
                    }
                }
                break; // Found a valid dependencies block, stop searching
            }
        }

        search_start = abs_pos + deps_keyword.len();
    }

    deps
}

/// Look up dependency mod names in the registy directory and return their
/// resolved absolute paths. Only returns paths that actually exist.
pub(crate) fn resolve_dependency_paths(
    registry_path: &Path,
    dep_names: &[String],
) -> Vec<PathBuf> {
    // Short-circuit if there's nothing to resolve
    if dep_names.is_empty() {
        return Vec::new();
    }

    // Read all .mod files in the registry, collecting name→path mappings
    let mut mod_index: Vec<(String, PathBuf)> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(registry_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            // Only process .mod files (not directories)
            if path.is_dir() || path.extension().and_then(|e| e.to_str()) != Some("mod") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Some(mod_name) = extract_mod_name(&content) {
                    if let Some(mod_path) = extract_mod_path(&content, registry_path) {
                        mod_index.push((mod_name, mod_path));
                    }
                }
            }
        }
    }

    // Resolve each dependency name to a path (preserving order)
    let mut resolved = Vec::new();
    for dep_name in dep_names {
        if let Some((_, path)) = mod_index.iter().find(|(name, _)| name == dep_name) {
            if path.exists() {
                resolved.push(path.clone());
            }
        }
    }

    resolved
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dependencies_empty() {
        let content = r#"name = "My Mod"
supported_version = "1.15.*""#;
        assert!(parse_dependencies(content).is_empty());
    }

    #[test]
    fn test_parse_dependencies_single() {
        let content = r#"name = "My Mod"
dependencies = { "Base Mod" }
supported_version = "1.15.*""#;
        assert_eq!(parse_dependencies(content), vec!["Base Mod"]);
    }

    #[test]
    fn test_parse_dependencies_multiple() {
        let content = r#"name = "My Mod"
dependencies = { "Base Mod" "Graphical Map Overhaul" }
supported_version = "1.15.*""#;
        let deps = parse_dependencies(content);
        assert_eq!(deps.len(), 2);
        assert_eq!(deps[0], "Base Mod");
        assert_eq!(deps[1], "Graphical Map Overhaul");
    }

    #[test]
    fn test_parse_dependencies_no_space() {
        // Some mods omit spaces around braces
        let content = r#"dependencies={"Dep1" "Dep2"}"#;
        let deps = parse_dependencies(content);
        assert_eq!(deps.len(), 2);
        assert_eq!(deps[0], "Dep1");
    }

    #[test]
    fn test_extract_mod_name() {
        let content = r#"name = "Test Mod"
path = "mod/testmod"
supported_version = "1.14.*""#;
        assert_eq!(extract_mod_name(content), Some("Test Mod".to_string()));
    }

    #[test]
    fn test_extract_mod_name_no_space() {
        let content = r#"name="Test Mod""#;
        assert_eq!(extract_mod_name(content), Some("Test Mod".to_string()));
    }

    #[test]
    fn test_extract_mod_path_relative() {
        let content = r#"path = "mod/testmod""#;
        let registry = Path::new("/home/user/.local/share/Paradox Interactive/Hearts of Iron IV/mod");
        let result = extract_mod_path(content, registry);
        assert_eq!(
            result,
            Some(
                Path::new("/home/user/.local/share/Paradox Interactive/Hearts of Iron IV/mod")
                    .join("mod/testmod")
            )
        );
    }

    #[test]
    fn test_extract_mod_path_absolute() {
        let content = r#"path = "/absolute/path/to/mod""#;
        let registry = Path::new("/some/registry");
        let result = extract_mod_path(content, registry);
        assert_eq!(result, Some(Path::new("/absolute/path/to/mod").to_path_buf()));
    }

    #[test]
    fn test_extract_mod_path_no_path() {
        let content = r#"name = "Test Mod"
supported_version = "1.15.*""#;
        assert!(extract_mod_path(content, Path::new("/tmp")).is_none());
    }

    #[test]
    fn test_resolve_dependency_paths_no_registry() {
        // Registry path that doesn't exist should yield empty results
        let result =
            resolve_dependency_paths(Path::new("/nonexistent/registry"), &["Something".to_string()]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_dependencies_with_nested_braces_in_comment() {
        // Comments with braces should not confuse the parser
        let content = r#"name = "My Mod"
# some { comment with } braces
dependencies = { "Dep A" }
supported_version = "1.15.*""#;
        let deps = parse_dependencies(content);
        assert_eq!(deps, vec!["Dep A"]);
    }
}
