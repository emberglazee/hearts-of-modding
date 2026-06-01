use crate::ast;
use crate::lsp_convert::ast_range_to_lsp;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::{Scope, ScopeStack};
use tower_lsp_server::ls_types::{
    Diagnostic, DiagnosticSeverity, NumberOrString,
};

/// Validates sprite/GFX references in `sprite`, `icon`, `sprite_name`,
/// and `picture` assignments.
///
/// Country idea `picture` resolution: `picture = name` resolves to
/// `GFX_idea_name` — the rule handles the default sprite lookup.
pub(crate) struct SpriteRule;

impl ValidationRule for SpriteRule {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        scope: &ScopeStack,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key_lower = ass.key.to_ascii_lowercase();
        if key_lower != "sprite"
            && key_lower != "icon"
            && key_lower != "sprite_name"
            && key_lower != "picture"
        {
            return;
        }

        let ast::Value::String(val) = &ass.value.value else {
            return;
        };

        let mut lookup_key = val.clone();

        // Country idea "picture" field resolution
        if key_lower == "picture" && scope.current() == Scope::Idea && !val.starts_with("GFX_idea_")
        {
            lookup_key = format!("GFX_idea_{}", val);
        }

        let exists = ctx.sprites.contains_key(&lookup_key)
            || (key_lower == "picture"
                && scope.current() == Scope::Idea
                && ctx
                    .sprites
                    .iter()
                    .any(|e| e.key().starts_with(&format!("{}_", lookup_key))));

        if !exists
            && (lookup_key.starts_with("GFX_")
                || (key_lower == "picture" && scope.current() == Scope::Idea))
        {
            diags.push(Diagnostic {
                range: ast_range_to_lsp(&ass.value.range),
                severity: Some(DiagnosticSeverity::WARNING),
                message: format!(
                    "Unknown sprite/GFX: '{}' (resolved from '{}')",
                    lookup_key, val
                ),
                code: Some(NumberOrString::String(
                    crate::advanced_validation::UNKNOWN_TRIGGER.to_string(),
                )),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        }
    }
}
