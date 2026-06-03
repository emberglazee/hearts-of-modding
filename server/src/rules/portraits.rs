use crate::parser::ast;
use crate::rules::visitor::AstVisitor;
use crate::rules::{ValidationContext, ValidationRule};
use crate::utils::lsp_convert::ast_range_to_lsp;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates portrait GFX references inside `portraits = { ... }` blocks.
///
/// Uses the centralized AST visitor to check that all string values
/// starting with `GFX_` in portrait blocks reference known sprites.
pub(crate) struct PortraitRule;

impl ValidationRule for PortraitRule {
    fn check_block(
        &self,
        _entries: &[ast::Entry],
        _ctx: &ValidationContext,
        _diags: &mut Vec<Diagnostic>,
    ) {
    }
}

/// Visitor state: tracks depth inside `portraits = { ... }` blocks.
///
/// For the HOI4 portrait structure:
/// ```hoi4
/// portraits = {
///     civilian = {
///         western = {
///             post_apocalyptic = {
///                 small = "GFX_portrait_eng_small"
///             }
///         }
///     }
/// }
/// ```
/// The depth counter increments when we see `portraits = { Block }`
/// and decrements on exit. All nested category blocks are automatically
/// inside the portraits block.
struct PortraitVisitor {
    in_portraits: u32,
}

impl PortraitVisitor {
    fn new() -> Self {
        Self { in_portraits: 0 }
    }
}

impl AstVisitor for PortraitVisitor {
    fn enter_assignment(
        &mut self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &crate::scope::scope::ScopeStack,
        diags: &mut Vec<Diagnostic>,
    ) {
        // Track entry into `portraits = { ... }` blocks
        if ass.key_text(ctx.source).eq_ignore_ascii_case("portraits") {
            if matches!(&ass.value.value, ast::Value::Block(_)) {
                self.in_portraits += 1;
            }
            return;
        }

        if self.in_portraits == 0 {
            return;
        }

        // Inside a portraits block: check string values for GFX_ references
        if let Some(s) = ass.value.value.as_str(ctx.source) {
            if s.starts_with("GFX_") && !ctx.sprites.contains_key(s) {
                diags.push(Diagnostic {
                    range: ast_range_to_lsp(&ass.value.range),
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: format!(
                        "Unknown portrait sprite '{}' — not found in any .gfx sprite definition",
                        s
                    ),
                    code: Some(NumberOrString::String(
                        crate::validation::advanced_validation::PORTRAIT_UNKNOWN_GFX.to_string(),
                    )),
                    source: Some("Hearts of Modding".to_string()),
                    ..Default::default()
                });
            }
        }
    }

    fn exit_assignment(
        &mut self,
        ass: &ast::Assignment,
        _ctx: &ValidationContext,
        _scope: &crate::scope::scope::ScopeStack,
        _diags: &mut Vec<Diagnostic>,
    ) {
        if ass.key_text(_ctx.source).eq_ignore_ascii_case("portraits") {
            if matches!(&ass.value.value, ast::Value::Block(_)) {
                self.in_portraits = self.in_portraits.saturating_sub(1);
            }
        }
    }
}

impl PortraitRule {
    pub(crate) fn visitor() -> Box<dyn AstVisitor> {
        Box::new(PortraitVisitor::new())
    }
}
