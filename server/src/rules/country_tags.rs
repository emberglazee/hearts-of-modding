use crate::parser::ast;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::ScopeStack;
use crate::utils::lsp_convert::ast_range_to_lsp;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates country tag references and dynamic/static tag ratios.
///
/// Per-entry: checks `tag`, `original_tag`, and `original_tag_to_check`
/// values against known country tags (allowing scope refs and var refs).
/// Block-level: warns if the file is in `common/country_tags/` and has
/// insufficient dynamic tags for civil war support.
pub(crate) struct CountryTagRule;

impl ValidationRule for CountryTagRule {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        scope: &ScopeStack,
        _pushed_scope: bool,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key_lower = ass.key.to_ascii_lowercase();

        // Skip 'tag' inside Idea scope — ideas use 'tag = { ... }' differently
        if key_lower == "tag" && scope.current() == crate::scope::scope::Scope::Idea {
            return;
        }

        if key_lower != "tag" && key_lower != "original_tag" && key_lower != "original_tag_to_check"
        {
            return;
        }

        let ast::Value::String(val) = &ass.value.value else {
            return;
        };

        // Allow scope references (ROOT, FROM, PREV, etc.)
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
        );
        let is_var_ref = val.starts_with("var:");

        let b = val.as_bytes();
        let looks_like_tag = val.len() == 3
            && b[0].is_ascii_alphabetic()
            && b[0].is_ascii_uppercase()
            && b[1].is_ascii_alphanumeric()
            && b[2].is_ascii_alphanumeric()
            && !matches!(
                val.as_str(),
                "NOT" | "AND" | "TAG" | "OOB" | "LOG" | "NUM" | "RED"
            );

        if !is_scope_ref
            && !is_var_ref
            && looks_like_tag
            && !ctx.country_tags.contains_key(val.as_str())
        {
            diags.push(Diagnostic {
                range: ast_range_to_lsp(&ass.value.range),
                severity: Some(DiagnosticSeverity::WARNING),
                message: format!("Unknown country tag: '{}'", val),
                code: Some(NumberOrString::String(
                    crate::validation::advanced_validation::UNKNOWN_TRIGGER.to_string(),
                )),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        }
    }

    fn check_block(
        &self,
        _entries: &[ast::Entry],
        ctx: &ValidationContext,
        diags: &mut Vec<Diagnostic>,
    ) {
        let uri = ctx.uri;
        // Only run for country_tags files
        if !uri.contains("/common/country_tags/") && !uri.contains("\\common\\country_tags\\") {
            return;
        }

        let ct = ctx.country_tags;
        let total = ct.len();
        let dynamic_count = ct.iter().filter(|t| t.value().dynamic).count();
        let static_count = total - dynamic_count;

        if total > 0 && dynamic_count == 0 {
            diags.push(Diagnostic {
                range: crate::utils::lsp_convert::ast_range_to_lsp(&ast::Range {
                    start_line: 0,
                    start_col: 0,
                    end_line: 0,
                    end_col: 0,
                }),
                severity: Some(DiagnosticSeverity::WARNING),
                message: "No dynamic country tags defined. Civil wars will fail for lack of dynamic tags, potentially causing a crash.".to_string(),
                code: Some(NumberOrString::String("HOM5001".to_string())),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        } else if static_count > 10 && dynamic_count < (static_count / 10).max(3) {
            diags.push(Diagnostic {
                range: crate::utils::lsp_convert::ast_range_to_lsp(&ast::Range {
                    start_line: 0,
                    start_col: 0,
                    end_line: 0,
                    end_col: 0,
                }),
                severity: Some(DiagnosticSeverity::INFORMATION),
                message: format!(
                    "Only {} dynamic tags for {} static tags. Consider adding more dynamic tags for civil wars.",
                    dynamic_count, static_count
                ),
                code: Some(NumberOrString::String("HOM5002".to_string())),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        }
    }
}
