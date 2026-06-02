use crate::parser::ast;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::ScopeStack;
use crate::utils::lsp_convert::ast_range_to_lsp;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates ideology and sub-ideology references.
///
/// Checks keys like `ideology` and `has_ideology` against known ideologies
/// and sub-ideologies from the scanner, allowing scope references (ROOT,
/// FROM, etc.) and variable references (var:...) to pass through.
pub(crate) struct IdeologyRule;

impl ValidationRule for IdeologyRule {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key_lower = ass.key.to_ascii_lowercase();
        if key_lower != "ideology" && key_lower != "has_ideology" {
            return;
        }

        let ast::Value::String(val) = &ass.value.value else {
            return;
        };

        // Allow scoped references (ROOT, FROM, PREV, THIS, etc.)
        let is_scope_ref = matches!(
            val.to_uppercase().as_str(),
            "ROOT"
                | "FROM"
                | "PREV"
                | "THIS"
                | "PREVPREV"
                | "PREVPREVPREV"
                | "PREVPREVPREVPREV"
                | "OWNER"
                | "CONTROLLER"
                | "CAPITAL"
                | "FROM.FROM"
                | "FROM.FROM.FROM"
        );
        // Allow variable references (var:SCOPE@name or var:name)
        let is_var_ref = val.starts_with("var:");

        if !ctx.ideologies.contains_key(val.as_str())
            && !ctx.sub_ideologies.contains_key(val.as_str())
            && !is_scope_ref
            && !is_var_ref
        {
            diags.push(Diagnostic {
                range: ast_range_to_lsp(&ass.value.range),
                severity: Some(DiagnosticSeverity::WARNING),
                message: format!("Unknown ideology: '{}'", val),
                code: Some(NumberOrString::String(
                    crate::validation::advanced_validation::UNKNOWN_TRIGGER.to_string(),
                )),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        }
    }
}
