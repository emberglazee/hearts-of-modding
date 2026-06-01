use crate::ast;
use crate::lsp_convert::ast_range_to_lsp;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::ScopeStack;
use tower_lsp_server::ls_types::{
    Diagnostic, DiagnosticSeverity, NumberOrString,
};

/// Validates trait references in `add_trait`, `has_trait`, and `remove_trait` assignments.
pub(crate) struct TraitRule;

impl ValidationRule for TraitRule {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key_lower = ass.key.to_ascii_lowercase();
        if key_lower != "add_trait" && key_lower != "has_trait" && key_lower != "remove_trait" {
            return;
        }

        let ast::Value::String(val) = &ass.value.value else {
            return;
        };

        if !ctx.traits.contains_key(val.as_str()) {
            diags.push(Diagnostic {
                range: ast_range_to_lsp(&ass.value.range),
                severity: Some(DiagnosticSeverity::WARNING),
                message: format!("Unknown trait: '{}'", val),
                code: Some(NumberOrString::String(
                    crate::advanced_validation::UNKNOWN_TRIGGER.to_string(),
                )),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        }
    }
}
