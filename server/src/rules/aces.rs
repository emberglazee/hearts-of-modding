use crate::parser::ast;
use crate::rules::ValidationContext;
use crate::rules::visitor::AstVisitor;
use crate::scope::scope::ScopeStack;
use crate::utils::lsp_convert::ast_range_to_lsp;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates `add_ace = { type = X }` references against known ace modifiers.
///
/// Tracks nesting via enter/exit assignment so `type` values inside
/// `add_ace = { }` blocks are checked against the scanned ace modifier names.
pub(crate) struct AceVisitor {
    in_add_ace: bool,
}

impl AceVisitor {
    pub fn visitor() -> Box<dyn AstVisitor> {
        Box::new(Self { in_add_ace: false })
    }
}

impl AstVisitor for AceVisitor {
    fn enter_assignment(
        &mut self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key_lower = ass.key_text(ctx.source).to_ascii_lowercase();

        if key_lower == "add_ace" {
            if matches!(
                &ass.value.value,
                ast::Value::Block(_) | ast::Value::TaggedBlock(..)
            ) {
                self.in_add_ace = true;
            }
            return;
        }

        if self.in_add_ace && key_lower == "type" {
            if let Some(type_val) = ass.value.value.as_str(ctx.source) {
                if !ctx.ace_modifiers.contains_key(type_val) {
                    diags.push(Diagnostic {
                        range: ast_range_to_lsp(&ass.key_range),
                        severity: Some(DiagnosticSeverity::WARNING),
                        message: format!(
                            "'{}' is not a known ace modifier tag. Must match a tag defined in common/aces/*.txt",
                            type_val
                        ),
                        source: Some("Hearts of Modding".to_string()),
                        code: Some(NumberOrString::String("HOM3020".to_string())),
                        ..Default::default()
                    });
                }
            }
        }
    }

    fn exit_assignment(
        &mut self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        _diags: &mut Vec<Diagnostic>,
    ) {
        let key_lower = ass.key_text(ctx.source).to_ascii_lowercase();
        if key_lower == "add_ace" {
            self.in_add_ace = false;
        }
    }
}
