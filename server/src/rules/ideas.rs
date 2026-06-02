use crate::parser::ast;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::{Scope, ScopeStack};
use crate::utils::lsp_convert::ast_range_to_lsp;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity};

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
        // This runs when we see an assignment whose key is the idea name
        // (Scope::Idea, depth 3 = inside ideas > category > idea)
        if scope.stack().len() == 3 && scope.current() == Scope::Idea {
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
