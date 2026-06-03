use crate::parser::ast;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::{Scope, ScopeStack};
use crate::utils::lsp_convert::ast_range_to_lsp;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity};

/// Known idea structure keywords that should not have picture validation.
/// These are the top-level `ideas = { ... }` wrapper and built-in category
/// names. `country` is handled implicitly because it resolves to
/// `Scope::Country`, so the picture check doesn't fire for it.
fn is_idea_structure_key(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        "ideas" | "hidden_ideas" | "designer" | "law"
    )
}

/// Validates idea references and default picture coverage.
///
/// Per-entry: checks `add_ideas`, `has_idea`, `remove_ideas` values.
/// Block-level: checks idea definition blocks for a `picture` field,
/// falling back to `GFX_idea_<name>` auto-resolution.
pub(crate) struct IdeaRule;

impl ValidationRule for IdeaRule {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        scope: &ScopeStack,
        pushed_scope: bool,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key_lower = ass.key.to_ascii_lowercase();

        // Idea existence checks (add_ideas, has_idea, remove_ideas)
        if key_lower == "add_ideas" || key_lower == "has_idea" || key_lower == "remove_ideas" {
            if let ast::Value::String(val) = &ass.value.value {
                if val != "all" && !ctx.ideas.contains_key(val.as_str()) {
                    diags.push(Diagnostic {
                        range: ast_range_to_lsp(&ass.value.range),
                        severity: Some(DiagnosticSeverity::WARNING),
                        message: format!("Unknown idea: '{}'", val),
                        source: Some("Hearts of Modding".to_string()),
                        ..Default::default()
                    });
                }
            }
            return;
        }

        // Default picture check for idea definitions
        // Fires only when the assignment caused a scope push (structural
        // entry) AND the current scope is Idea (actual idea definition,
        // not a sub-block like modifier/on_add), excluding structural
        // keywords like `ideas`, `hidden_ideas`, `designer`, `law`.
        if pushed_scope && scope.current() == Scope::Idea && !is_idea_structure_key(&ass.key) {
            if let ast::Value::Block(entries) | ast::Value::TaggedBlock(_, entries, _) =
                &ass.value.value
            {
                let has_picture = entries.iter().any(|e| {
                    if let ast::Entry::Assignment(a) = e {
                        a.key.eq_ignore_ascii_case("picture")
                    } else {
                        false
                    }
                });

                if !has_picture {
                    let default_gfx = format!("GFX_idea_{}", ass.key);
                    let exists = ctx.sprites.contains_key(default_gfx.as_str())
                        || ctx
                            .sprites
                            .iter()
                            .any(|e| e.key().starts_with(&format!("{}_", default_gfx)));
                    if !exists {
                        diags.push(Diagnostic {
                            range: ast_range_to_lsp(&ass.key_range),
                            severity: Some(DiagnosticSeverity::WARNING),
                            message: format!(
                                "Idea '{}' is missing a 'picture' field and the default GFX '{}' was not found.",
                                ass.key, default_gfx
                            ),
                            source: Some("Hearts of Modding".to_string()),
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }
}
