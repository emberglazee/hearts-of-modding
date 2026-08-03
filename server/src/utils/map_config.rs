use crate::parser::ast;
use crate::parser::parser;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

#[derive(Clone)]
pub struct MapConfig {
    pub definitions: String,
    pub adjacencies: String,
}

impl Default for MapConfig {
    fn default() -> Self {
        MapConfig {
            definitions: "definition.csv".to_string(),
            adjacencies: "adjacencies.csv".to_string(),
        }
    }
}

/// Cached parse of `map/default.map` per workspace root, invalidated by file mtime.
///
/// Historically `get_map_config` did a `read_to_string` + full `parse_script` on
/// EVERY call; hover and adjacency completion call it on the hot path, so parsing
/// per request was significant. `map/default.map` is workspace-static, so we cache
/// the parsed config keyed by root and only reparse when the file's modification
/// time changes (which also covers edits/replacements via the file watcher).
type CachedEntry = (MapConfig, Option<SystemTime>);

static CACHE: Lazy<Mutex<HashMap<PathBuf, CachedEntry>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn get_map_config(root: &Path) -> MapConfig {
    // Canonicalize so `"."` (server CWD) and absolute spellings of the same
    // root share one cache entry, and so per-URI lookups that resolve the
    // containing root can match against the stored key reliably.
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let default_map_path = root.join("map/default.map");
    // Cheap: one stat call. Absent file → None (we still cache that so we don't
    // re-stat-and-miss the absent case every request — we only re-check existence
    // cheaply below; if a default.map appears later its mtime differs).
    let current_mtime = default_map_path.metadata().and_then(|m| m.modified()).ok();
    let key = root.clone();

    let cached = {
        let cache = CACHE.lock().unwrap();
        match cache.get(&key) {
            Some((cfg, cached_mtime)) if *cached_mtime == current_mtime => Some(cfg.clone()),
            _ => None,
        }
    };
    if let Some(cfg) = cached {
        return cfg;
    }

    let mut config = MapConfig::default();
    if default_map_path.exists() {
        if let Ok(content) = fs::read_to_string(&default_map_path) {
            let (script, _) = parser::parse_script(&content);
            for entry in script.entries {
                if let ast::Entry::Assignment(ass) = entry {
                    let key = ass.key_text(&content);
                    if key.eq_ignore_ascii_case("definitions") {
                        if let Some(s) = ass.value.value.as_str(&content) {
                            config.definitions = s.to_string();
                        }
                    } else if key.eq_ignore_ascii_case("adjacencies") {
                        if let Some(s) = ass.value.value.as_str(&content) {
                            config.adjacencies = s.to_string();
                        }
                    }
                }
            }
        }
    }

    let mut cache = CACHE.lock().unwrap();
    cache.insert(key, (config.clone(), current_mtime));
    config
}

/// Find the workspace root whose canonical path prefixes `uri`, preferring the
/// LONGEST (most specific) match so `/mod` doesn't claim `/mod2/...`. Both the
/// URI and the roots are compared in forward-slash form (Windows-safe).
/// Returns `None` when no root contains the URI.
pub(crate) fn matching_root<'a>(roots: &'a [PathBuf], uri: &str) -> Option<&'a PathBuf> {
    let uri_norm = uri.replace('\\', "/");

    let mut best: Option<(usize, &'a PathBuf)> = None;
    for root in roots.iter() {
        let root_str = root.to_string_lossy().replace('\\', "/");
        // Path-boundary match: `/mod/` prefixes the file path (never `/mod2/`).
        let boundary = format!("{}/", root_str);
        if uri_norm.contains(&boundary) || uri_norm.ends_with(&root_str) {
            if best.is_none_or(|(len, _)| root_str.len() > len) {
                best = Some((root_str.len(), root));
            }
        }
    }

    best.map(|(_, root)| root)
}

// ---------------------------------------------------------------------------
// SECTION - Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_root() -> PathBuf {
        let d = std::env::temp_dir().join(format!("hom_mapcfg_{}", std::process::id()));
        let _ = fs::create_dir_all(d.join("map"));
        d
    }

    #[test]
    fn parses_default_map_and_reparses_on_change() {
        let root = temp_root();
        let map = root.join("map/default.map");
        fs::write(
            &map,
            "definitions = \"vars/definition.csv\"\nadjacencies = \"vars/adjacencies.csv\"\n",
        )
        .unwrap();

        let c1 = get_map_config(&root);
        assert_eq!(c1.definitions, "vars/definition.csv");
        assert_eq!(c1.adjacencies, "vars/adjacencies.csv");

        // Second call (hot path) returns the cached value unchanged.
        assert_eq!(get_map_config(&root).definitions, c1.definitions);

        // After the file changes, the mtime differs and we reparse.
        std::thread::sleep(std::time::Duration::from_millis(20));
        fs::write(&map, "definitions = \"other.csv\"\n").unwrap();
        assert_eq!(get_map_config(&root).definitions, "other.csv");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn missing_default_map_uses_defaults() {
        let c = get_map_config(&PathBuf::from("/definitely/not/a/real/mod"));
        assert_eq!(c.definitions, "definition.csv");
        assert_eq!(c.adjacencies, "adjacencies.csv");
    }

    /// The longest (most specific) root must win when roots nest, so a
    /// sub-mod folder inside another root resolves to the sub-mod's config.
    #[test]
    fn test_matching_root_prefers_longest() {
        let roots = vec![
            PathBuf::from("/home/embi/mod"),
            PathBuf::from("/home/embi/mod/submod"),
        ];
        let uri = "file:///home/embi/mod/submod/map/adjacencies.csv";
        assert_eq!(
            matching_root(&roots, uri),
            Some(&roots[1]),
            "nested sub-mod root must win over the parent root"
        );

        let outer = "file:///home/embi/mod/map/adjacencies.csv";
        assert_eq!(
            matching_root(&roots, outer),
            Some(&roots[0]),
            "parent root must win when the file is outside the sub-mod"
        );
    }

    /// Path-boundary matching: `/mod` must not claim files under `/mod2/`.
    #[test]
    fn test_matching_root_path_boundary() {
        let roots = vec![PathBuf::from("/home/embi/mod")];
        assert_eq!(
            matching_root(&roots, "file:///home/embi/mod2/map/adjacencies.csv"),
            None,
            "`/mod` must not match `/mod2/...`"
        );
        assert_eq!(
            matching_root(&roots, "file:///home/embi/mod/map/adjacencies.csv"),
            Some(&roots[0])
        );
    }

    /// Windows-style backslash URIs and roots normalize to forward slashes.
    #[test]
    fn test_matching_root_windows_paths() {
        let roots = vec![PathBuf::from(r"C:\Users\embi\mod")];
        assert_eq!(
            matching_root(&roots, r"file:///C:/Users/embi/mod/map/adjacencies.csv"),
            Some(&roots[0]),
            "backslash root must match a forward-slash URI"
        );
    }

    /// No containing root → None (caller falls back to first root / legacy ".").
    #[test]
    fn test_matching_root_no_match() {
        let roots = vec![PathBuf::from("/home/embi/mod")];
        assert_eq!(
            matching_root(&roots, "file:///tmp/somewhere/else/map/adjacencies.csv"),
            None
        );
        assert_eq!(matching_root(&[], "file:///home/embi/mod/x.csv"), None);
    }
}

// ---------------------------------------------------------------------------
// !SECTION
// ---------------------------------------------------------------------------
