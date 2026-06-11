/// Integration test for dependency resolution + replace_path + game path.
///
/// Uses the real test mods created at:
///   ~/.local/share/Paradox Interactive/Hearts of Iron IV/mod/test_base/
///   ~/.local/share/Paradox Interactive/Hearts of Iron IV/mod/test_submod/
///
/// The submod depends on the base mod and declares:
///   replace_path = "common/national_focus"
///
/// Expected HOI4 behavior:
///   - Submod's common/national_focus/sub_focus.txt  ✓ (highest priority)
///   - Base's  common/national_focus/base_focus.txt  ✗ (replaced away)
///   - Game's  common/national_focus/vanilla_focus.txt ✗ (replaced away)
///   - Submod's common/ideas/sub_idea.txt             ✓ (not replaced)
///   - Base's  common/ideas/base_idea.txt             ✓ (not replaced)
#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use crate::scanner::file_overlay::FileOverlay;
    use crate::utils::mod_registry;

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_dir() -> PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "hom_dep_resolve_test_{}_{}",
            std::process::id(),
            id
        ))
    }

    fn create_file(path: &std::path::Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, "").unwrap();
    }

    /// Helper: create a no-op filter that passes everything.
    fn noop_filter() -> impl Fn(&std::path::Path) -> bool {
        |_: &std::path::Path| false
    }

    /// Test the full root stack: mock game path → base mod → workspace,
    /// with dependency resolution and replace_path applied.
    #[test]
    fn test_dependency_resolution_and_replace_path_with_game_path() {
        // ── 1. Create mock game path with a focus and an idea ───────────
        let tmp = temp_dir();
        let game = tmp.join("game");
        create_file(&game.join("common/national_focus/vanilla_focus.txt"));
        create_file(&game.join("common/ideas/vanilla_idea.txt"));

        // ── 2. Create a base mod (dependency) ──────────────────────────
        let base_mod = tmp.join("base_mod");
        create_file(&base_mod.join("common/national_focus/base_focus.txt"));
        create_file(&base_mod.join("common/ideas/base_idea.txt"));

        // ── 3. Create a workspace (submod) with replace_path ────────────
        let workspace = tmp.join("workspace");
        create_file(&workspace.join("common/national_focus/sub_focus.txt"));
        create_file(&workspace.join("common/ideas/sub_idea.txt"));
        create_file(&workspace.join("descriptor.mod"));

        // Write descriptor.mod with replace_path + dependencies
        let descriptor = r#"name = "Test Submod"
dependencies = { "Test Base Mod" }
replace_path = "common/national_focus"
supported_version = "1.18.*""#;
        fs::write(workspace.join("descriptor.mod"), descriptor).unwrap();

        // ── 4. Set up a mock mod registry ──────────────────────────────
        let registry = tmp.join("registry");
        fs::create_dir_all(&registry).unwrap();
        // Registry .mod file for the base mod (using absolute path)
        let base_mod_reg = format!(
            r#"name = "Test Base Mod"
path = "{}"
supported_version = "1.18.*""#,
            base_mod.to_string_lossy()
        );
        fs::write(registry.join("test_base.mod"), &base_mod_reg).unwrap();

        // ── 5. Parse dependencies and replace_path from the workspace ───
        let content = fs::read_to_string(workspace.join("descriptor.mod")).unwrap();
        let dep_names = mod_registry::parse_dependencies(&content);
        let replace_paths = mod_registry::parse_replace_paths(&content);

        assert!(
            dep_names.contains(&"Test Base Mod".to_string()),
            "Workspace should declare dependency, got: {:?}",
            dep_names
        );
        assert_eq!(
            replace_paths,
            vec!["common/national_focus"],
            "Workspace should declare replace_path"
        );

        // ── 6. Resolve the dependency ──────────────────────────────────
        let resolved = mod_registry::resolve_dependency_paths(&registry, &dep_names);
        assert!(!resolved.is_empty(), "Should resolve dependency");
        let resolved_base = &resolved[0];
        assert_eq!(resolved_base, &base_mod, "Should resolve to base_mod path");

        // ── 7. Build the full root stack ───────────────────────────────
        // Order: [game (lowest), base_mod, workspace (highest)]
        let roots = vec![game.clone(), resolved_base.clone(), workspace.clone()];

        let overlay =
            FileOverlay::build_script_only(&roots, &["txt"], noop_filter(), &replace_paths);

        let entries = overlay.all_entries();

        // ── 8. Verify national_focus (replaced) ────────────────────────
        // All files from replaced dirs in non-highest-priority roots are gone.
        assert!(
            !entries.contains_key("common/national_focus/vanilla_focus.txt"),
            "Game focus should be GONE (replaced)"
        );
        assert!(
            !entries.contains_key("common/national_focus/base_focus.txt"),
            "Base mod focus should be GONE (replaced)"
        );
        assert!(
            entries.contains_key("common/national_focus/sub_focus.txt"),
            "Workspace focus should be PRESENT"
        );

        // The workspace root is NOT affected by replace_path.
        let focus_winner = overlay.winning_files_in("common/national_focus");
        assert_eq!(
            focus_winner.len(),
            1,
            "Only workspace focus should survive in the replaced dir"
        );
        let winner_path = &focus_winner[0];
        assert!(
            winner_path.to_string_lossy().contains("workspace"),
            "Winner should be from workspace, got: {:?}",
            winner_path
        );

        // ── 9. Verify ideas (NOT replaced) ─────────────────────────────
        // All layers contribute because ideas are not in replace_path.
        assert!(
            entries.contains_key("common/ideas/vanilla_idea.txt"),
            "Vanilla idea should be PRESENT"
        );
        assert!(
            entries.contains_key("common/ideas/base_idea.txt"),
            "Base idea should be PRESENT"
        );
        assert!(
            entries.contains_key("common/ideas/sub_idea.txt"),
            "Submod idea should be PRESENT"
        );

        let idea_files = overlay.winning_files_in("common/ideas");
        assert_eq!(idea_files.len(), 3, "All 3 idea files should be present");

        // The workspace (highest priority) wins for conflicting file paths.
        // But since each file has a unique name in the non-replaced dir,
        // all files are present under their own keys. The overlay only
        // deduplicates by exact relative path — different filenames coexist.
        // Let's verify the conflict case: if the workspace had the same
        // relative path as a base mod file, the workspace would win.
        // But since our test uses unique filenames, all three survive.

        fs::remove_dir_all(&tmp).ok();
    }
}
