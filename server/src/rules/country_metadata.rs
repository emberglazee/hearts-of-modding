use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::ScopeStack;
use crate::utils::lsp_convert::ast_range_to_lsp;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates GFX sprite references in `country_metadata/` files.
///
/// Checks that `playthrough_stats_background` and `career_profile_background`
/// values (e.g. `GFX_playthrough_stats_bg_ENG`) reference known sprites
/// from the `.gfx` sprite definitions.
pub(crate) struct CountryMetadataRule;

impl ValidationRule for CountryMetadataRule {
    fn check_assignment(
        &self,
        ass: &crate::parser::ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        _pushed_scope: bool,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key_lower = ass.key_text(ctx.source).to_ascii_lowercase();
        if key_lower != "playthrough_stats_background" && key_lower != "career_profile_background" {
            return;
        }

        let Some(val) = ass.value.value.as_str(ctx.source) else {
            return;
        };

        if !val.starts_with("GFX_") {
            return;
        }

        if !ctx.sprites.contains_key(val) {
            diags.push(Diagnostic {
                range: ast_range_to_lsp(&ass.value.range),
                severity: Some(DiagnosticSeverity::WARNING),
                message: format!(
                    "Unknown country metadata sprite '{}' — not found in any .gfx sprite definition",
                    val
                ),
                code: Some(NumberOrString::String(
                    crate::validation::advanced_validation::UNKNOWN_COUNTRY_METADATA_GFX
                        .to_string(),
                )),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        }
    }
}
