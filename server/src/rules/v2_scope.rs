use crate::parser::ast;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::{Scope, ScopeStack};
use crate::utils::lsp_convert::ast_range_to_lsp;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates triggers, effects, and modifiers against V2 data.
///
/// This rule is the V2-powered replacement for hardcoded validation.
/// It implements HOM004 (scope mismatch).
///
/// HOM004: Scope mismatch — entity used outside its allowed scopes
pub(crate) struct V2ScopeRule;

impl V2ScopeRule {
    /// Check if a key is a known event target
    fn is_event_target(key: &str, ctx: &ValidationContext) -> bool {
        let lower = key.to_ascii_lowercase();
        ctx.event_targets.get(&*lower).is_some() || ctx.event_targets.get(key).is_some()
    }
}

impl ValidationRule for V2ScopeRule {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        scope: &ScopeStack,
        _pushed_scope: bool,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key = ass.key_text(ctx.source);
        let current_scope = scope.current();

        // Skip empty keys and structural keywords
        if key.is_empty() || key.starts_with('#') {
            return;
        }

        // Skip idea structure keys (cost, level, picture, etc.) when in Idea scope
        // — these are idea properties, not trigger/effect/modifier usages
        if current_scope == Scope::Idea && crate::rules::visitor::is_idea_structure_key(key) {
            return;
        }

        // Skip known event targets — they define scope transitions
        // dynamically and shouldn't be flagged as scope mismatches
        if Self::is_event_target(key, ctx) {
            return;
        }

        // Check V2 trigger/effect/modifier data
        let entity = crate::TRIGGERS
            .get(key)
            .or_else(|| crate::EFFECTS.get(key))
            .or_else(|| crate::MODIFIERS.get(key));

        if let Some(entity) = entity {
            // Scope mismatch check (HOM004)
            if entity.scopes.allows(&current_scope) {
                return;
            }

            // Check if scope is Unknown (we can't validate)
            if current_scope == Scope::Unknown {
                return;
            }

            // Scope mismatch — entity not allowed in this scope
            let scope_names: Vec<&str> = entity.scopes.usage.iter().map(|s| s.as_str()).collect();

            // Don't emit for Global-flagged entities — they're always valid
            if entity.scopes.usage.contains(&Scope::Global) {
                return;
            }

            diags.push(Diagnostic {
                range: ast_range_to_lsp(&ass.key_range),
                severity: Some(DiagnosticSeverity::WARNING),
                message: format!(
                    "'{}' is not valid in {} scope. Expected scopes: {}",
                    key,
                    current_scope.as_str(),
                    scope_names.join(", "),
                ),
                code: Some(NumberOrString::String(
                    crate::validation::advanced_validation::SCOPE_MISMATCH.to_string(),
                )),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        }
    }
}
