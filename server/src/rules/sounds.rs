use crate::ast;
use crate::lsp_convert::ast_range_to_lsp;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::ScopeStack;
use tower_lsp_server::ls_types::{
    Diagnostic, DiagnosticSeverity, NumberOrString,
};

/// Validates `sound_effect` references against known sound effects.
pub(crate) struct SoundRule;

impl ValidationRule for SoundRule {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        diags: &mut Vec<Diagnostic>,
    ) {
        if ass.key.to_ascii_lowercase() != "sound_effect" {
            return;
        }

        let ast::Value::String(val) = &ass.value.value else {
            return;
        };

        if !ctx.sound_effects.contains_key(val) {
            diags.push(Diagnostic {
                range: ast_range_to_lsp(&ass.value.range),
                severity: Some(DiagnosticSeverity::WARNING),
                message: format!("Unknown sound effect: '{}'", val),
                code: Some(NumberOrString::String(
                    crate::advanced_validation::UNKNOWN_TRIGGER.to_string(),
                )),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        }
    }
}
