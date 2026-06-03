use crate::parser::ast;
use crate::rules::visitor::AstVisitor;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::ScopeStack;
use crate::utils::lsp_convert::ast_range_to_lsp;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates AI area definitions: checks that continents and strategic
/// regions referenced in `/common/ai_areas/` files actually exist in
/// the scanner data.
///
/// Uses the centralized AST visitor — no per-rule recursion.
pub(crate) struct AiAreaRule;

impl ValidationRule for AiAreaRule {
    fn check_block(
        &self,
        _entries: &[ast::Entry],
        _ctx: &ValidationContext,
        _diags: &mut Vec<Diagnostic>,
    ) {
    }
}

/// Visitor state: tracks whether the current file is an AI areas file,
/// and whether we're inside a top-level area definition block.
///
/// AI areas files look like:
/// ```hoi4
/// my_area = {
///     continents = { europe asia }
///     strategic_regions = { 1 2 3 }
/// }
/// ```
///
/// The walker visits each assignment. When we see `continents = { ... }`
/// or `strategic_regions = { ... }`, we inspect their block entries
/// inline (since the values inside are bare Values, not Assignments).
struct AiAreaVisitor {
    /// Whether we've confirmed this is an AI areas file.
    /// Set on the first entry, remains true for the full walk.
    is_ai_areas_file: bool,
}

impl AiAreaVisitor {
    fn new(uri: &str) -> Self {
        // Fast path: check URI once at construction time
        let is_ai_areas_file =
            uri.contains("/common/ai_areas/") || uri.contains("\\common\\ai_areas\\");
        Self { is_ai_areas_file }
    }
}

impl AstVisitor for AiAreaVisitor {
    fn enter_assignment(
        &mut self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        diags: &mut Vec<Diagnostic>,
    ) {
        if !self.is_ai_areas_file {
            return;
        }

        match ass.key.as_str() {
            "continents" => {
                if let ast::Value::Block(cont_entries) = &ass.value.value {
                    for ce in cont_entries {
                        if let ast::Entry::Value(val) = ce {
                            if let ast::Value::String(name) = &val.value {
                                if !ctx.continents.contains_key(name.as_str()) {
                                    diags.push(Diagnostic {
                                        range: ast_range_to_lsp(&val.range),
                                        severity: Some(DiagnosticSeverity::WARNING),
                                        message: format!("Unknown continent: '{}'", name),
                                        code: Some(NumberOrString::String("HOM6001".to_string())),
                                        source: Some("Hearts of Modding".to_string()),
                                        ..Default::default()
                                    });
                                }
                            }
                        }
                    }
                }
            }
            "strategic_regions" => {
                if let ast::Value::Block(sr_entries) = &ass.value.value {
                    for se in sr_entries {
                        if let ast::Entry::Value(val) = se {
                            if let ast::Value::Number(n) = &val.value {
                                let id = *n as u32;
                                if !ctx.strategic_regions.contains_key(&id) {
                                    diags.push(Diagnostic {
                                        range: ast_range_to_lsp(&val.range),
                                        severity: Some(DiagnosticSeverity::WARNING),
                                        message: format!("Unknown strategic region: {}", id),
                                        code: Some(NumberOrString::String("HOM6002".to_string())),
                                        source: Some("Hearts of Modding".to_string()),
                                        ..Default::default()
                                    });
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

impl AiAreaRule {
    pub(crate) fn visitor(uri: &str) -> Box<dyn AstVisitor> {
        Box::new(AiAreaVisitor::new(uri))
    }
}
