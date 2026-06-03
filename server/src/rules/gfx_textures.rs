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
