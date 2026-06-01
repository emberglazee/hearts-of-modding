use crate::ast;
use crate::lsp_convert::ast_range_to_lsp;
use crate::rules::{ValidationContext, ValidationRule};
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates portrait GFX references inside `portraits = { ... }` blocks.
///
/// Recursively checks that all string values starting with `GFX_` in
/// portrait blocks reference known sprites from the scanner.
pub(crate) struct PortraitRule;

impl ValidationRule for PortraitRule {
    fn check_block(
        &self,
        entries: &[ast::Entry],
        ctx: &ValidationContext,
        diags: &mut Vec<Diagnostic>,
    ) {
        validate_portrait_gfx_recursive(entries, ctx, diags);
    }
}

fn validate_portrait_gfx_recursive(
    entries: &[ast::Entry],
    ctx: &ValidationContext,
    diags: &mut Vec<Diagnostic>,
) {
    for entry in entries {
        let ast::Entry::Assignment(ass) = entry else {
            continue;
        };

        if ass.key.eq_ignore_ascii_case("portraits") {
            if let ast::Value::Block(portrait_entries) = &ass.value.value {
                validate_portrait_values(portrait_entries, ctx, diags);
            }
        }

        match &ass.value.value {
            ast::Value::Block(inner) => {
                validate_portrait_gfx_recursive(inner, ctx, diags);
            }
            ast::Value::TaggedBlock(_, inner, _) => {
                validate_portrait_gfx_recursive(inner, ctx, diags);
            }
            _ => {}
        }
    }
}

fn validate_portrait_values(
    entries: &[ast::Entry],
    ctx: &ValidationContext,
    diags: &mut Vec<Diagnostic>,
) {
    for entry in entries {
        let ast::Entry::Assignment(ass) = entry else {
            continue;
        };

        // Check if value is a string starting with GFX_
        if let ast::Value::String(s) = &ass.value.value {
            if s.starts_with("GFX_") && !ctx.sprites.contains_key(s.as_str()) {
                diags.push(Diagnostic {
                    range: ast_range_to_lsp(&ass.value.range),
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: format!(
                        "Unknown portrait sprite '{}' — not found in any .gfx sprite definition",
                        s
                    ),
                    code: Some(NumberOrString::String(
                        crate::advanced_validation::PORTRAIT_UNKNOWN_GFX.to_string(),
                    )),
                    source: Some("Hearts of Modding".to_string()),
                    ..Default::default()
                });
            }
        }

        // Recurse into nested blocks (for civilian/army/navy categories)
        if let ast::Value::Block(inner) = &ass.value.value {
            validate_portrait_values(inner, ctx, diags);
        }
    }
}
