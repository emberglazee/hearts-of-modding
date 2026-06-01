use crate::ast;
use crate::lsp_convert::ast_range_to_lsp;
use crate::rules::{ValidationContext, ValidationRule};
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates achievement and ribbon definitions for localization coverage.
///
/// Checks that `custom_achievement` and `custom_ribbon` entries have
/// corresponding `<name>_NAME` and `<name>_DESC` localization keys.
pub(crate) struct AchievementRule;

impl ValidationRule for AchievementRule {
    fn check_block(
        &self,
        entries: &[ast::Entry],
        ctx: &ValidationContext,
        diags: &mut Vec<Diagnostic>,
    ) {
        for entry in entries {
            let ast::Entry::Assignment(ass) = entry else {
                continue;
            };
            let key_lower = ass.key.to_ascii_lowercase();
            if key_lower == "custom_achievement" || key_lower == "custom_ribbon" {
                let name_key = format!("{}_NAME", ass.key);
                let desc_key = format!("{}_DESC", ass.key);

                if !ctx.loc.contains_key(&name_key) {
                    diags.push(Diagnostic {
                        range: ast_range_to_lsp(&ass.key_range),
                        severity: Some(DiagnosticSeverity::WARNING),
                        message: format!(
                            "Achievement '{}' is missing localization key: '{}'",
                            ass.key, name_key
                        ),
                        code: Some(NumberOrString::String(
                            crate::advanced_validation::ACHIEVEMENT_MISSING_LOCALIZATION
                                .to_string(),
                        )),
                        source: Some("Hearts of Modding".to_string()),
                        ..Default::default()
                    });
                }
                if !ctx.loc.contains_key(&desc_key) {
                    diags.push(Diagnostic {
                        range: ast_range_to_lsp(&ass.key_range),
                        severity: Some(DiagnosticSeverity::WARNING),
                        message: format!(
                            "Achievement '{}' is missing localization key: '{}'",
                            ass.key, desc_key
                        ),
                        code: Some(NumberOrString::String(
                            crate::advanced_validation::ACHIEVEMENT_MISSING_LOCALIZATION
                                .to_string(),
                        )),
                        source: Some("Hearts of Modding".to_string()),
                        ..Default::default()
                    });
                }
            }
        }
    }
}
