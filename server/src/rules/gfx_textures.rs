use crate::parser::ast;
use crate::rules::ValidationContext;
use crate::utils::lsp_convert::ast_range_to_lsp;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity};

/// Validates texture file paths in `.gfx` and `.gui` files.
///
/// Checks that `texturefile = "path"` values point to existing files
/// (relative to the game path or the .gfx file's directory) and flags
/// non-standard path separators (`//`, `\`).
///
/// Unlike other rules, this is not registered in the generic rule list
/// because it requires the `.gfx` file's own path for resolution.
pub(crate) struct GfxTextureRule<'a> {
    gfx_file_path: &'a std::path::Path,
}

impl<'a> GfxTextureRule<'a> {
    pub(crate) fn new(gfx_file_path: &'a std::path::Path) -> Self {
        Self { gfx_file_path }
    }

    pub(crate) fn validate(
        &self,
        entries: &[ast::Entry],
        ctx: &ValidationContext,
        diags: &mut Vec<Diagnostic>,
    ) {
        let game_path = &ctx.game_path;
        let gfx_dir = self.gfx_file_path.parent();

        for entry in entries {
            match entry {
                ast::Entry::Assignment(ass) => {
                    if ass.key_text(ctx.source).eq_ignore_ascii_case("texturefile") {
                        if let Some(val) = ass.value.value.as_str(ctx.source) {
                            let has_double_slash = val.contains("//");
                            let has_backslash = val.contains('\\');

                            // Styling: non-standard path separators
                            if ctx.styling_enabled && (has_double_slash || has_backslash) {
                                let suggestion = val.replace("//", "/").replace('\\', "/");
                                diags.push(Diagnostic {
                                    range: ast_range_to_lsp(&ass.value.range),
                                    severity: Some(DiagnosticSeverity::INFORMATION),
                                    code: Some(tower_lsp_server::ls_types::NumberOrString::String(
                                        "styling_path_separator".to_string(),
                                    )),
                                    message: format!(
                                        "Use single forward slashes in texture paths. Suggestion: '{}'.",
                                        suggestion
                                    ),
                                    source: Some("Hearts of Modding".to_string()),
                                    data: Some(serde_json::to_value(suggestion).unwrap()),
                                    ..Default::default()
                                });
                            }

                            // Existence check
                            let normalized = val.replace('\\', "/");
                            let mut found = false;

                            // Try relative to game path
                            if let Some(ref gp) = *game_path {
                                let full = std::path::Path::new(gp).join(&normalized);
                                if full.exists() {
                                    found = true;
                                }
                            }

                            // Try relative to each workspace/mod root
                            if !found {
                                for root in ctx.workspace_roots {
                                    let full = root.join(&normalized);
                                    if full.exists() {
                                        found = true;
                                        break;
                                    }
                                }
                            }

                            // Try relative to .gfx file directory
                            if !found {
                                if let Some(dir) = gfx_dir {
                                    let full = dir.join(&normalized);
                                    if full.exists() {
                                        found = true;
                                    }
                                }
                            }

                            if !found {
                                diags.push(Diagnostic {
                                    range: ast_range_to_lsp(&ass.value.range),
                                    severity: Some(DiagnosticSeverity::WARNING),
                                    message: format!("Texture file not found: '{}'", val),
                                    source: Some("Hearts of Modding".to_string()),
                                    ..Default::default()
                                });
                            }
                        }
                    }

                    // Recurse into blocks
                    match &ass.value.value {
                        ast::Value::Block(entries) | ast::Value::TaggedBlock(_, entries, _) => {
                            self.validate(entries, ctx, diags);
                        }
                        _ => {}
                    }
                }
                ast::Entry::Value(val) => match &val.value {
                    ast::Value::Block(entries) | ast::Value::TaggedBlock(_, entries, _) => {
                        self.validate(entries, ctx, diags);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parser;
    use dashmap::DashMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    /// Parse HOI4 script content and run GfxTextureRule validation with
    /// the given workspace root, game path, and file path.
    /// All DashMaps are empty — GfxTextureRule only uses source + game_path
    /// + styling_enabled + workspace_roots from the context.
    fn validate_texture_file(
        gfx_content: &str,
        gfx_file_path: &Path,
        workspace_roots: &[PathBuf],
        game_path: Option<&str>,
        styling_enabled: bool,
    ) -> Vec<Diagnostic> {
        let (script, _) = parser::parse_script(gfx_content);
        let game_path_owned = game_path.map(|s| s.to_string());

        let ctx = ValidationContext {
            uri: "test.gfx",
            source: &script.source,
            loc: &DashMap::new(),
            scripted_triggers: &DashMap::new(),
            scripted_effects: &DashMap::new(),
            ideologies: &DashMap::new(),
            sub_ideologies: &DashMap::new(),
            traits: &DashMap::new(),
            sprites: &DashMap::new(),
            ideas: &DashMap::new(),
            characters: &DashMap::new(),
            provinces: &DashMap::new(),
            modifier_mappings: &DashMap::new(),
            ignored_loc_regex: &[],
            comments: &[],
            sound_effects: &DashMap::new(),
            country_tags: &DashMap::new(),
            buildings: &DashMap::new(),
            resources: &DashMap::new(),
            state_categories: &DashMap::new(),
            continents: &DashMap::new(),
            strategic_regions: &DashMap::new(),
            terrain_categories: &DashMap::new(),
            abilities: &DashMap::new(),
            game_path: game_path_owned,
            styling_enabled,
            workspace_roots,
            unit_types: &DashMap::new(),
            event_targets: &DashMap::new(),
            event_namespaces: &DashMap::new(),
            events: &DashMap::new(),
            decisions: &DashMap::new(),
            decision_categories: &DashMap::new(),
        };

        let rule = GfxTextureRule::new(gfx_file_path);
        let mut diags = Vec::new();
        rule.validate(&script.entries, &ctx, &mut diags);
        diags
    }

    // ── Workspace root resolution tests ───────────────────────────────

    #[test]
    fn test_texture_found_in_workspace_root() {
        let tmp = PathBuf::from("target/gfx_texture_test_ws_root");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("gfx/interface")).unwrap();

        let texture_path = tmp.join("gfx/interface/test_texture.dds");
        fs::write(&texture_path, b"fake dds").unwrap();

        // .gfx file is in interface/ (typical HOI4 layout)
        let gfx_file = tmp.join("interface/some_file.gfx");

        let content = r#"spriteTypes = {
            spriteType = {
                name = "GFX_test"
                texturefile = "gfx/interface/test_texture.dds"
            }
        }"#;

        let diags =
            validate_texture_file(content, &gfx_file, std::slice::from_ref(&tmp), None, false);
        assert!(
            diags.is_empty(),
            "Texture in workspace root should be found, got: {:?}",
            diags
        );

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_texture_not_found_emits_diagnostic() {
        let tmp = PathBuf::from("target/gfx_texture_test_not_found");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let gfx_file = tmp.join("interface/some_file.gfx");
        let content = r#"spriteTypes = {
            spriteType = {
                name = "GFX_missing"
                texturefile = "gfx/interface/nonexistent.dds"
            }
        }"#;

        let diags =
            validate_texture_file(content, &gfx_file, std::slice::from_ref(&tmp), None, false);
        assert_eq!(
            diags.len(),
            1,
            "Should emit one diagnostic for missing texture"
        );
        assert!(
            diags[0].message.contains("Texture file not found"),
            "Message should indicate file not found"
        );

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_texture_found_via_gfx_file_directory_fallback() {
        // Texture placed relative to .gfx file's directory (not workspace root).
        // .gfx file:  {root}/some_subdir/some_file.gfx
        // texture:    {root}/some_subdir/gfx/interface/test.dds
        // → resolved by joining gfx parent dir with the texturefile path
        let tmp = PathBuf::from("target/gfx_texture_test_gfxdir_fallback");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("gfx/interface")).unwrap();

        let subdir = tmp.join("some_subdir");
        fs::create_dir_all(subdir.join("gfx/interface")).unwrap();

        let texture_path = subdir.join("gfx/interface/test.dds");
        fs::write(&texture_path, b"fake dds").unwrap();

        let gfx_file = subdir.join("some_file.gfx");
        let content = r#"spriteTypes = {
            frameAnimatedSpriteType = {
                name = "GFX_test"
                texturefile = "gfx/interface/test.dds"
            }
        }"#;

        let diags =
            validate_texture_file(content, &gfx_file, std::slice::from_ref(&tmp), None, false);
        assert!(
            diags.is_empty(),
            "Texture found via .gfx file directory fallback should report no diagnostic"
        );

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_texture_found_in_game_path() {
        let tmp = PathBuf::from("target/gfx_texture_test_game_path");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("gfx/interface")).unwrap();

        let texture_path = tmp.join("gfx/interface/vanilla_texture.dds");
        fs::write(&texture_path, b"fake dds").unwrap();

        // .gfx file is outside game path — only game path has the texture
        let gfx_file = PathBuf::from("/tmp/gfx_texture_test_game_path/file.gfx");
        let content = r#"spriteTypes = {
            spriteType = {
                name = "GFX_vanilla"
                texturefile = "gfx/interface/vanilla_texture.dds"
            }
        }"#;

        let diags =
            validate_texture_file(content, &gfx_file, &[], Some(&tmp.to_string_lossy()), false);
        assert!(diags.is_empty(), "Texture in game path should be found");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_workspace_root_overrides_gfx_dir() {
        // Texture exists at workspace root level, NOT near the .gfx file.
        // Only the workspace root check should find it.
        let tmp = PathBuf::from("target/gfx_texture_test_ws_override");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("gfx/interface")).unwrap();

        let texture_path = tmp.join("gfx/interface/ws_texture.dds");
        fs::write(&texture_path, b"fake dds").unwrap();

        // .gfx file is in a deeply nested subdirectory — no texture there
        let gfx_file = tmp.join("interface/subdir/deep/file.gfx");
        fs::create_dir_all(gfx_file.parent().unwrap()).unwrap();

        let content = r#"spriteTypes = {
            spriteType = {
                name = "GFX_ws_test"
                texturefile = "gfx/interface/ws_texture.dds"
            }
        }"#;

        let diags =
            validate_texture_file(content, &gfx_file, std::slice::from_ref(&tmp), None, false);
        assert!(
            diags.is_empty(),
            "Texture found via workspace root should resolve regardless of .gfx file depth"
        );

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_styling_diagnostic_for_backslash() {
        let tmp = PathBuf::from("target/gfx_texture_test_styling");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("gfx/interface")).unwrap();

        let texture_path = tmp.join("gfx/interface/styled.dds");
        fs::write(&texture_path, b"fake dds").unwrap();

        let gfx_file = tmp.join("interface/file.gfx");
        // Use double backslashes in source — parser interprets \\ as
        // an escape for literal \, so the parsed value becomes
        // "gfx\interface\styled.dds" and the rule catches the backslash.
        let content = r#"spriteTypes = {
            spriteType = {
                name = "GFX_styled"
                texturefile = "gfx\\interface\\styled.dds"
            }
        }"#;

        let diags =
            validate_texture_file(content, &gfx_file, std::slice::from_ref(&tmp), None, true);
        // Should have a styling diagnostic (backslash fix suggestion) AND
        // no "not found" diagnostic — backslashes are normalized before lookup
        let styling_diags: Vec<_> = diags.iter().filter(|d| {
            matches!(&d.code, Some(tower_lsp_server::ls_types::NumberOrString::String(s)) if s == "styling_path_separator")
        }).collect();
        let not_found_diags: Vec<_> = diags
            .iter()
            .filter(|d| d.message.contains("not found"))
            .collect();

        assert_eq!(
            styling_diags.len(),
            1,
            "Should emit styling diagnostic for backslashes"
        );
        assert!(
            not_found_diags.is_empty(),
            "Should NOT report texture not found — backslashes normalized during resolution"
        );

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_empty_workspace_roots_no_crash() {
        // With empty workspace_roots and no game_path, validation should
        // still fall through to the .gfx file directory check without crashing.
        let tmp = PathBuf::from("target/gfx_texture_test_empty_roots");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let gfx_file = tmp.join("some_file.gfx");
        let content = r#"spriteTypes = {
            spriteType = {
                name = "GFX_test"
                texturefile = "missing.dds"
            }
        }"#;

        let diags = validate_texture_file(content, &gfx_file, &[], None, false);
        assert_eq!(diags.len(), 1, "Missing file should still be reported");
        assert!(diags[0].message.contains("not found"));

        let _ = fs::remove_dir_all(&tmp);
    }
}
