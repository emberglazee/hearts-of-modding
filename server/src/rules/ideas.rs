use crate::parser::ast;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::{Scope, ScopeStack};
use crate::utils::lsp_convert::ast_range_to_lsp;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity};

/// Keywords that should never have picture validation in an idea context.
/// Covers category structure keys, category attributes, and idea sub-block
/// properties. Keys that resolve to a non-Idea scope (e.g. `modifier`,
/// `allowed`, `available` → Scope::Country) are handled implicitly — they
/// won't be at `scope.current() == Scope::Idea` when the check fires.
/// This list catches the rest that would otherwise hit Scope::Unknown and
/// trip the picture validation.
fn is_idea_structure_key(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        // Category structure & attributes
        "ideas"
        | "hidden_ideas"
        | "designer"
        | "law"
        | "use_list_view"
        | "slot_ledgers"
        | "slot"
        | "character_slot"
        // Idea sub-block properties
        | "picture"
        | "targeted_modifier"
        | "research_bonus"
        | "equipment_bonus"
        | "rule"
        | "traits"
        | "on_add"
        | "on_remove"
        | "cancel"
        | "allowed_civil_war"
        | "do_effect"
        | "visible"
        | "allowed_to_remove"
        | "removal_cost"
        | "level"
        | "ledger"
        | "hidden"
        | "politics_tab"
    )
}

/// Check if an assignment is an idea category container rather than an actual idea.
/// Category containers (e.g. `economy = { law = yes skulk_economy = { ... } }`)
/// have children that are sub-idea definitions — block-valued assignments whose
/// keys are both scope-unknown AND not recognised idea sub-block keywords.
/// Actual ideas have only sub-blocks (modifier, on_add, cancel, etc.) which
/// either resolve to a known scope or are in the structure key list.
fn is_idea_category_block(ass: &ast::Assignment, ctx: &ValidationContext) -> bool {
    match &ass.value.value {
        ast::Value::Block(entries) | ast::Value::TaggedBlock(_, entries, _) => {
            entries.iter().any(|e| {
                if let ast::Entry::Assignment(a) = e {
                    if matches!(
                        &a.value.value,
                        ast::Value::Block(_) | ast::Value::TaggedBlock(_, _, _)
                    ) {
                        let key = a.key_text(ctx.source);
                        Scope::from_str(key) == Scope::Unknown && !is_idea_structure_key(key)
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
        }
        _ => false,
    }
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
        let key_lower = ass.key_text(ctx.source).to_ascii_lowercase();

        // Idea existence checks (add_ideas, has_idea, remove_ideas)
        if key_lower == "add_ideas" || key_lower == "has_idea" || key_lower == "remove_ideas" {
            if let Some(val) = ass.value.value.as_str(ctx.source) {
                if val != "all" && !ctx.ideas.contains_key(val) {
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
        if pushed_scope
            && scope.current() == Scope::Idea
            && !is_idea_structure_key(ass.key_text(ctx.source))
            && !is_idea_category_block(ass, ctx)
        {
            // Skip picture check for ideas within `hidden_ideas` — they
            // are never displayed in the UI so a picture is unnecessary.
            if scope.stack().contains(&Scope::HiddenIdeaCategory) {
                return;
            }
            if let ast::Value::Block(entries) | ast::Value::TaggedBlock(_, entries, _) =
                &ass.value.value
            {
                let has_picture = entries.iter().any(|e| {
                    if let ast::Entry::Assignment(a) = e {
                        a.key_text(ctx.source).eq_ignore_ascii_case("picture")
                    } else {
                        false
                    }
                });

                if !has_picture {
                    let default_gfx = format!("GFX_idea_{}", ass.key_text(ctx.source));
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
                                ass.key_text(ctx.source), default_gfx
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

#[cfg(test)]
mod tests {
    use super::is_idea_structure_key;

    #[test]
    fn test_idea_structure_keys_are_excluded() {
        let excluded = [
            "ideas",
            "hidden_ideas",
            "designer",
            "law",
            "use_list_view",
            "slot_ledgers",
            "slot",
            "character_slot",
            "picture",
            "targeted_modifier",
            "research_bonus",
            "equipment_bonus",
            "rule",
            "traits",
            "on_add",
            "on_remove",
            "cancel",
            "allowed_civil_war",
            "do_effect",
            "visible",
            "allowed_to_remove",
            "removal_cost",
            "level",
            "ledger",
            "hidden",
            "politics_tab",
        ];
        for key in &excluded {
            assert!(
                is_idea_structure_key(key),
                "'{}' should be recognised as a structure key",
                key,
            );
        }
    }

    #[test]
    fn test_idea_names_are_not_excluded() {
        let names = [
            "my_idea",
            "red_political_purge",
            "red_corrupt_guilds_4",
            "generic_foreign_capital",
            "idea_123",
            "ZZZ_custom_idea",
            "china_designer",
        ];
        for name in &names {
            assert!(
                !is_idea_structure_key(name),
                "'{}' should NOT be a structure key",
                name,
            );
        }
    }

    #[test]
    fn test_idea_structure_key_case_insensitive() {
        assert!(is_idea_structure_key("ON_ADD"));
        assert!(is_idea_structure_key("On_Add"));
        assert!(is_idea_structure_key("on_ADD"));
        assert!(is_idea_structure_key("CANCEL"));
        assert!(is_idea_structure_key("On_Remove"));
        assert!(is_idea_structure_key("DO_EFFECT"));
        assert!(is_idea_structure_key("ALLOWED_CIVIL_WAR"));
    }

    #[test]
    fn test_picture_excluded() {
        // picture = value (not block), but still shouldn't trigger
        assert!(is_idea_structure_key("picture"));
    }
}
