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
/// Additionally, total-conversion mods declare `replace_path` in their
/// `descriptor.mod` to completely wipe out entire subdirectories from
/// lower-priority layers (vanilla game path and dependency mods). This
/// struct respects those directives: when building, if a root is not
/// the highest-priority (workspace) root, any files under a replaced
/// path are skipped entirely.
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
    /// `replace_paths` are directory prefixes from the active mod's
    /// `descriptor.mod` that should completely replace the corresponding
    /// directories in lower-priority roots. When set, files from non-highest-
    /// priority roots whose relative path starts with a replace_path
    /// prefix are skipped entirely.
    ///
    /// `extensions` controls which file extensions to include in the overlay.
    /// Pass all script extensions: `&["txt", "yml", "asset", "gfx", "gui", "csv"]`.
    ///
    /// `filter` is the ignore filter (files/dirs matching ignore regexes
    /// are skipped).
    pub fn build<F>(
        roots: &[PathBuf],
        extensions: &[&str],
        filter: F,
        replace_paths: &[String],
    ) -> Self
    where
        F: Fn(&Path) -> bool,
    {
        let mut entries: HashMap<String, PathBuf> = HashMap::new();
        let highest_priority_idx = roots.len().saturating_sub(1);

        for (idx, root) in roots.iter().enumerate() {
            if filter(root) {
                continue;
            }
            let canonical_root = root.canonicalize().unwrap_or_else(|_| root.clone());
            let is_highest = idx == highest_priority_idx;
            Self::walk_and_collect(
                &canonical_root,
                extensions,
                &filter,
                &mut entries,
                replace_paths,
                is_highest,
            );
        }

        FileOverlay { entries }
    }

    /// Build a file overlay but skip localisation/ and common/defines/ paths.
    /// Those are resolved by key, not by file path.
    pub fn build_script_only<F>(
        roots: &[PathBuf],
        extensions: &[&str],
        filter: F,
        replace_paths: &[String],
    ) -> Self
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
                // Skip gfx/fonts/ — .txt files here are font credits, not script
                || path_str.contains("/gfx/fonts/")
                || path_str.contains("\\gfx\\fonts\\")
            {
                return true;
            }
            filter(path)
        };

        Self::build(roots, extensions, filter_with_exceptions, replace_paths)
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
    ///
    /// When `is_highest_priority` is false, directories whose relative path
    /// matches one of the `replace_paths` are skipped entirely — simulating
    /// the HOI4 engine's behaviour of wiping those directories from lower
    /// layers.
    fn walk_and_collect<F>(
        root: &Path,
        extensions: &[&str],
        filter: &F,
        entries: &mut HashMap<String, PathBuf>,
        replace_paths: &[String],
        is_highest_priority: bool,
    ) where
        F: Fn(&Path) -> bool,
    {
        let mut dirs = vec![root.to_path_buf()];
        while let Some(current_dir) = dirs.pop() {
            if filter(&current_dir) {
                continue;
            }

            // Compute the relative directory path from the root.
            // For the root itself, rel_dir is empty (the replaced-path check
            // will never match an empty string against a replace_path like
            // "common/national_focus").
            let rel_dir = if current_dir == root {
                String::new()
            } else if let Ok(rel) = current_dir.strip_prefix(root) {
                rel.to_string_lossy().replace('\\', "/")
            } else {
                String::new()
            };

            // For non-highest-priority roots: skip entire directories
            // whose relative path falls under a replace_path prefix.
            if !is_highest_priority
                && !replace_paths.is_empty()
                && Self::is_replaced_path(&rel_dir, replace_paths)
            {
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
                                // Normalize to forward slashes for cross-platform consistency:
                                // `winning_files_in` and all scanner path checks use `/` as
                                // the separator. On Windows, `strip_prefix` yields paths with
                                // backslashes (`\`) which would never match those checks.
                                let rel_str = rel_path.to_string_lossy().replace('\\', "/");
                                // Later (higher priority) roots overwrite lower ones
                                entries.insert(rel_str, path);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Check whether `rel_dir` (a relative directory path from a root, using
    /// forward slashes) falls under any of the `replace_paths`.
    ///
    /// A directory matches if its relative path equals a replace_path exactly,
    /// or if it sits under one (i.e. `rel_dir = "common/national_focus/subdir"`
    /// matches `replace_path = "common/national_focus"`).
    ///
    /// NOTE: ancestor directories (e.g. `"common"` when `replace_path =
    /// "common/national_focus"`) do NOT match — we need to descend into them
    /// to reach the replaced subtree.
    fn is_replaced_path(rel_dir: &str, replace_paths: &[String]) -> bool {
        // The root directory itself is never replaced.
        if rel_dir.is_empty() {
            return false;
        }
        replace_paths.iter().any(|rp| {
            // Exact match: this directory IS a replaced path
            rel_dir == rp.as_str()
            // Subdirectory match: we're inside a replaced path tree
            || rel_dir.starts_with(&format!("{}/", rp))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use std::sync::atomic::{AtomicU64, Ordering};
    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_dir() -> PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "hom_file_overlay_test_{}_{}",
            std::process::id(),
            id
        ))
    }

    fn create_file(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, "").unwrap();
    }

    /// Dummy filter that passes everything.
    fn pass_all(_: &Path) -> bool {
        false
    }

    #[test]
    fn test_basic_overlay() {
        let root = temp_dir();
        let root_a = root.join("a");
        let root_b = root.join("b");

        create_file(&root_a.join("common/ideas/usa.txt"));
        create_file(&root_b.join("common/ideas/usa.txt"));

        let roots = vec![root_a.clone(), root_b.clone()];
        let overlay = FileOverlay::build(&roots, &["txt"], pass_all, &[]);

        // root_b (higher priority) wins
        let entries = overlay.all_entries();
        assert_eq!(entries.len(), 1);
        let winner = entries.get("common/ideas/usa.txt").unwrap();
        assert!(winner.to_string_lossy().contains("/b/"));

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_replace_path_skips_vanilla_dir() {
        let root = temp_dir();
        let vanilla = root.join("vanilla");
        let mod_root = root.join("mod");

        // Vanilla has files under a path that the mod replaces
        create_file(&vanilla.join("common/national_focus/ger.txt"));
        create_file(&vanilla.join("common/national_focus/eng.txt"));
        // Vanilla also has files in a non-replaced path
        create_file(&vanilla.join("common/ideas/usa.txt"));

        // Mod has only one focus file (different name)
        create_file(&mod_root.join("common/national_focus/kaiserreich.txt"));
        // Mod also has an ideas file
        create_file(&mod_root.join("common/ideas/usa.txt"));

        let roots = vec![vanilla.clone(), mod_root.clone()];
        let replace_paths = vec!["common/national_focus".to_string()];
        let overlay = FileOverlay::build(&roots, &["txt"], pass_all, &replace_paths);

        let entries = overlay.all_entries();

        // Vanilla national_focus files should be gone (replaced)
        assert!(
            !entries.contains_key("common/national_focus/ger.txt"),
            "vanilla ger.txt should be replaced"
        );
        assert!(
            !entries.contains_key("common/national_focus/eng.txt"),
            "vanilla eng.txt should be replaced"
        );

        // Mod's national_focus file should be present
        assert!(
            entries.contains_key("common/national_focus/kaiserreich.txt"),
            "mod focus file should be present"
        );

        // Ideas are not replaced — both vanilla and mod files should be present
        // (mod wins the conflict)
        assert!(
            entries.contains_key("common/ideas/usa.txt"),
            "ideas file should be present"
        );
        let ideas_winner = entries.get("common/ideas/usa.txt").unwrap();
        assert!(
            ideas_winner.to_string_lossy().contains("/mod/"),
            "ideas should be won by mod root"
        );

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_replace_path_multiple_dirs() {
        let root = temp_dir();
        let vanilla = root.join("vanilla");
        let mod_root = root.join("mod");

        create_file(&vanilla.join("common/national_focus/ger.txt"));
        create_file(&vanilla.join("common/ideas/default.txt"));
        create_file(&vanilla.join("common/decisions/war.txt"));
        create_file(&vanilla.join("common/ai_areas/normal.txt")); // not replaced

        create_file(&mod_root.join("common/national_focus/total_overhaul.txt"));
        create_file(&mod_root.join("common/ideas/new_ideas.txt"));
        create_file(&mod_root.join("common/decisions/new_decisions.txt"));
        create_file(&mod_root.join("common/ai_areas/normal.txt"));

        let roots = vec![vanilla.clone(), mod_root.clone()];
        let replace_paths = vec![
            "common/national_focus".to_string(),
            "common/ideas".to_string(),
            "common/decisions".to_string(),
        ];
        let overlay = FileOverlay::build(&roots, &["txt"], pass_all, &replace_paths);

        let entries = overlay.all_entries();

        // Replaced dirs: all vanilla files should be gone
        assert!(!entries.contains_key("common/national_focus/ger.txt"));
        assert!(!entries.contains_key("common/ideas/default.txt"));
        assert!(!entries.contains_key("common/decisions/war.txt"));

        // Mod files in replaced dirs should be present
        assert!(entries.contains_key("common/national_focus/total_overhaul.txt"));
        assert!(entries.contains_key("common/ideas/new_ideas.txt"));
        assert!(entries.contains_key("common/decisions/new_decisions.txt"));

        // Non-replaced dir: both layers present, mod wins
        assert!(entries.contains_key("common/ai_areas/normal.txt"));
        let winner = entries.get("common/ai_areas/normal.txt").unwrap();
        assert!(winner.to_string_lossy().contains("/mod/"));

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_replace_path_no_vanilla_files_in_replaced_dir() {
        // Edge case: mod replaces a directory where vanilla has NO files.
        // Should still work fine.
        let root = temp_dir();
        let vanilla = root.join("vanilla");
        let mod_root = root.join("mod");

        create_file(&vanilla.join("common/ideas/usa.txt"));
        create_file(&mod_root.join("common/national_focus/new_focus.txt"));

        let roots = vec![vanilla.clone(), mod_root.clone()];
        let replace_paths = vec!["common/national_focus".to_string()];
        let overlay = FileOverlay::build(&roots, &["txt"], pass_all, &replace_paths);

        let entries = overlay.all_entries();
        assert!(entries.contains_key("common/national_focus/new_focus.txt"));
        assert!(entries.contains_key("common/ideas/usa.txt"));

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_replace_path_no_workspace_effect() {
        // Replace paths should NOT affect the workspace (highest-priority) root.
        let root = temp_dir();
        let vanilla = root.join("vanilla");
        let workspace = root.join("workspace");

        create_file(&vanilla.join("common/national_focus/ger.txt"));
        // Workspace has a file in the replaced dir
        create_file(&workspace.join("common/national_focus/workspace_focus.txt"));
        // Workspace also has a file in a non-replaced dir
        create_file(&workspace.join("common/ideas/workspace_idea.txt"));

        let roots = vec![vanilla.clone(), workspace.clone()];
        let replace_paths = vec!["common/national_focus".to_string()];
        let overlay = FileOverlay::build(&roots, &["txt"], pass_all, &replace_paths);

        let entries = overlay.all_entries();

        // Vanilla focus file should be gone
        assert!(!entries.contains_key("common/national_focus/ger.txt"));

        // Workspace focus file should be present (workspace is not affected by replace_path)
        assert!(entries.contains_key("common/national_focus/workspace_focus.txt"));

        // Workspace idea file too
        assert!(entries.contains_key("common/ideas/workspace_idea.txt"));

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_build_script_only_skips_localisation_and_defines() {
        let root = temp_dir();
        let vanilla = root.join("vanilla");
        let mod_root = root.join("mod");

        create_file(&vanilla.join("common/national_focus/ger.txt"));
        create_file(&vanilla.join("localisation/english.yml"));
        create_file(&vanilla.join("common/defines/00_defines.txt"));
        create_file(&mod_root.join("common/national_focus/new_focus.txt"));
        create_file(&mod_root.join("localisation/english.yml"));
        create_file(&mod_root.join("common/defines/00_defines.txt"));

        let roots = vec![vanilla.clone(), mod_root.clone()];
        let replace_paths = vec!["common/national_focus".to_string()];
        let overlay =
            FileOverlay::build_script_only(&roots, &["txt", "yml"], pass_all, &replace_paths);

        let entries = overlay.all_entries();

        // national_focus files should follow replace_path rules
        assert!(!entries.contains_key("common/national_focus/ger.txt"));
        assert!(entries.contains_key("common/national_focus/new_focus.txt"));

        // localisation/ and common/defines/ should be skipped entirely
        // (they're resolved by key-level merge, not file-level overlay)
        assert!(!entries.contains_key("localisation/english.yml"));
        assert!(!entries.contains_key("common/defines/00_defines.txt"));

        fs::remove_dir_all(&root).ok();
    }

    /// Verify that the overlay normalises relative paths to forward slashes.
    /// On Windows, `Path::strip_prefix` yields `\`-separated paths. The
    /// overlay must convert those to `/` so that `winning_files_in` prefix
    /// checks (which use `/`) actually match the stored keys.
    #[cfg(windows)]
    #[test]
    fn test_windows_path_normalization() {
        let root = temp_dir();
        let vanilla = root.join("vanilla");
        let mod_root = root.join("mod");

        // Path::join uses `\` on Windows — this is the exact scenario that
        // was broken: file paths stored with backslashes never matched the
        // forward-slash prefix checks in `winning_files_in`.
        create_file(&vanilla.join("common/ideas/usa.txt"));
        create_file(&vanilla.join("common/national_focus/ger.txt"));
        create_file(&mod_root.join("common/ideas/usa.txt"));

        let roots = vec![vanilla.clone(), mod_root.clone()];
        let overlay = FileOverlay::build(&roots, &["txt"], pass_all, &[]);

        let entries = overlay.all_entries();

        // Every entry key MUST use forward slashes — no backslashes anywhere
        assert!(
            entries.keys().all(|k| !k.contains('\\')),
            "All overlay entry keys must use forward slashes, got: {:?}",
            entries.keys().collect::<Vec<_>>()
        );

        // `winning_files_in` must find the mod's usa.txt (higher priority wins)
        let ideas_files = overlay.winning_files_in("common/ideas");
        assert_eq!(ideas_files.len(), 1, "Should find 1 file under common/ideas");
        let winner = &ideas_files[0];
        assert!(
            winner.to_string_lossy().contains("mod"),
            "Mod file should win, got: {:?}",
            winner
        );

        // Focus file from vanilla must be discoverable (not replaced)
        let focus_files = overlay.winning_files_in("common/national_focus");
        assert_eq!(focus_files.len(), 1, "Should find 1 file under common/national_focus");
        let focus_winner = &focus_files[0];
        assert!(
            focus_winner.to_string_lossy().contains("vanilla"),
            "Vanilla file should be found, got: {:?}",
            focus_winner
        );

        fs::remove_dir_all(&root).ok();
    }
}
