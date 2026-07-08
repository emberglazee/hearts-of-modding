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
        let raw_scope = scope.current();
        let current_scope = raw_scope.effective_scope();

        // Skip empty keys and structural keywords
        if key.is_empty() || key.starts_with('#') {
            return;
        }

        // Can't validate if the scope is Unknown (unresolved event target,
        // mio: reference, unknown scope keyword, etc.)
        if raw_scope == Scope::Unknown {
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
            // Skip keys with pushes_scope — these are structural scope-pushers
            // (e.g. controller, any_country, all_state) not real triggers.
            // The V2ScopeRule should only validate actual trigger/effect usage.
            if entity.pushes_scope.is_some() {
                return;
            }

            // Skip keys that are recognized by Scope::from_str as structural
            // keywords (e.g. state, unit, character, country). These keywords
            // have their own scope semantics and shouldn't be double-checked
            // as triggers — they can appear as both scope-pushers and value
            // parameters in various contexts.
            if Scope::from_str(key) != Scope::Unknown {
                return;
            }

            // NOTE: is_fully_controlled_by is documented as State-scoped but
            // is commonly used inside for_each_scope_loop = { array = *_states }
            // which iterates state arrays at runtime. The LSP can't statically
            // know the array element type, so it can't push State scope.
            // Allowing Country scope avoids false positives in this pattern.
            if key == "is_fully_controlled_by" && current_scope == Scope::Country {
                return;
            }

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

            // ModifierBag scope (unit_modifiers etc.) skips strict scope checks.
            // The engine reads these blocks as a flat modifier bag and routes
            // entries per-key — asking "what scope is it" is a category error.
            if current_scope == Scope::ModifierBag {
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
