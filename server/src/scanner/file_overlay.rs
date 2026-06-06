#![allow(dead_code)]
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A file-level VFS overlay that resolves which file "wins" for each
/// relative path when multiple roots provide the same file.
///
/// In HOI4, script files (events, ideas, focuses, traits, etc.) are
/// overridden by **file path**, not by entity key. If a mod contains
/// `common/ideas/usa.txt`, the vanilla `usa.txt` is completely ignored
/// — any ideas that were in the vanilla file but omitted in the mod
/// file cease to exist in-game.
///
/// This struct builds the overlay by iterating roots in priority order
/// (index 0 = lowest priority / vanilla, last index = highest priority /
/// workspace root). For each relative path, only the highest-priority
/// occurrence is kept as the "winning" file.
///
/// # Exceptions
/// Localization (`localisation/`) and defines (`common/defines/`) DO
/// merge by key in HOI4 and are **not** resolved through this overlay.
/// Their existing `LayeredValue` logic is correct as-is.
#[derive(Debug, Clone)]
pub struct FileOverlay {
    /// Relative path (e.g., `common/ideas/usa.txt`) → winning absolute path
    entries: HashMap<String, PathBuf>,
}

impl FileOverlay {
    /// Build a file overlay from roots in priority order.
    ///
    /// `roots` are iterated in order; index 0 is the lowest priority
    /// (vanilla game path) and the last index is the highest priority
    /// (workspace root). Later occurrences of the same relative path
    /// overwrite earlier ones — only the highest-priority file survives.
    ///
    /// `extensions` controls which file extensions to include in the overlay.
    /// Pass all script extensions: `&["txt", "yml", "asset", "gfx", "gui", "csv"]`.
    ///
    /// `filter` is the ignore filter (files/dirs matching ignore regexes
    /// are skipped).
    pub fn build<F>(roots: &[PathBuf], extensions: &[&str], filter: F) -> Self
    where
        F: Fn(&Path) -> bool,
    {
        let mut entries: HashMap<String, PathBuf> = HashMap::new();

        for root in roots.iter() {
            if filter(root) {
                continue;
            }
            let canonical_root = root.canonicalize().unwrap_or_else(|_| root.clone());
            Self::walk_and_collect(&canonical_root, extensions, &filter, &mut entries);
        }

        FileOverlay { entries }
    }

    /// Build a file overlay but skip localisation/ and common/defines/ paths.
    /// Those are resolved by key, not by file path.
    pub fn build_script_only<F>(roots: &[PathBuf], extensions: &[&str], filter: F) -> Self
    where
        F: Fn(&Path) -> bool,
    {
        let filter_with_exceptions = |path: &Path| {
            // Skip the merge-by-key directories — they are NOT resolved
            // through the file-level overlay.
            let path_str = path.to_string_lossy().to_ascii_lowercase();
            if path_str.contains("localisation")
                || path_str.contains("localisation")
                || path_str.contains("common/defines")
                || path_str.contains("common\\defines")
            {
                return true;
            }
            filter(path)
        };

        Self::build(roots, extensions, filter_with_exceptions)
    }

    /// Get the winning absolute file paths for files under a relative prefix.
    ///
    /// Example: `winning_files_in("common/ideas")` returns all winning
    /// `.txt` files under `common/ideas/` across all roots.
    pub fn winning_files_in(&self, prefix: &str) -> Vec<PathBuf> {
        // Normalize prefix to use forward slashes
        let prefix_norm = prefix.trim_end_matches('/');
        let prefix_check = format!("{}/", prefix_norm);

        self.entries
            .iter()
            .filter(|(rel_path, _)| {
                rel_path.as_str() == prefix_norm
                    || rel_path.starts_with(&prefix_check)
                    || rel_path.starts_with(prefix_norm)
            })
            .map(|(_, abs_path)| abs_path.clone())
            .collect()
    }

    /// Get all winning files as a reference to the internal map.
    pub fn all_entries(&self) -> &HashMap<String, PathBuf> {
        &self.entries
    }

    /// Number of winning files in the overlay.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Walk a single root directory, collecting files matched by extensions
    /// and inserting them into the entries map (overwriting any existing
    /// entries from lower-priority roots).
    fn walk_and_collect<F>(
        root: &Path,
        extensions: &[&str],
        filter: &F,
        entries: &mut HashMap<String, PathBuf>,
    ) where
        F: Fn(&Path) -> bool,
    {
        let mut dirs = vec![root.to_path_buf()];
        while let Some(current_dir) = dirs.pop() {
            if filter(&current_dir) {
                continue;
            }
            if let Ok(read_dir) = std::fs::read_dir(&current_dir) {
                for entry in read_dir.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if !filter(&path) {
                            dirs.push(path);
                        }
                    } else if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy();
                        if extensions.contains(&ext_str.as_ref()) {
                            if filter(&path) {
                                continue;
                            }
                            // Compute the relative path from the root
                            if let Ok(rel_path) = path.strip_prefix(root) {
                                let rel_str = rel_path.to_string_lossy().to_string();
                                // Later (higher priority) roots overwrite lower ones
                                entries.insert(rel_str, path);
                            }
                        }
                    }
                }
            }
        }
    }
}
