//! The cursor walkers (hover, go-to-definition, references) must resolve scope
//! with the same block context as validation.
//!
//! They used to build a `ScopeCtx` with `in_random_list: false,
//! state_targeted: false`, so inside `random_list = { 10 = { … } }` the weight
//! `10` hovered as a state id, and `FROM` inside a `state_target` decision
//! resolved to Country.
//!
//! The control cases matter as much as the fixes: the same numeric key outside
//! a weight block must still be State, and `FROM` without `state_target` must
//! stay Country, so a blanket suppression cannot pass.

use crate::scope::scope::{self, Scope, ScopeCtx, ScopeStack};
use crate::utils::symbol_search::find_identifier_at;
use tower_lsp_server::ls_types::Position;

fn position_of(content: &str, needle: &str) -> Position {
    let offset = content.find(needle).expect("needle present in fixture");
    let before = &content[..offset];
    let line = before.matches('\n').count() as u32;
    let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    Position {
        line,
        character: (offset - line_start) as u32,
    }
}

fn identifier_scopes(content: &str, uri: &str, needle: &str) -> (String, Vec<Scope>) {
    let (script, _) = crate::parser::parser::parse_script(content);
    let mut stack = ScopeStack::new(scope::initial_scope_for_uri(uri));
    let sctx = ScopeCtx::new(uri, None, None, None, None);
    let pos = position_of(content, needle);
    let (identifier, scopes, _, _) =
        find_identifier_at(&script, pos, &mut stack, &sctx).expect("identifier at cursor");
    (identifier, scopes)
}

/// The completion path walks the AST with its own function
/// (`find_scope_context_at`); it carried the same two false flags and needs the
/// same context.
#[test]
fn completion_scope_flags_match_the_cursor_walker() {
    let content =
        "country_event = {\n\trandom_list = {\n\t\t10 = { add_stability = 0.1 }\n\t}\n}\n";
    let uri = "/mod/common/events/test.txt";
    let (script, _) = crate::parser::parser::parse_script(content);
    let (_, scopes) = crate::scope::scope_context::find_scope_context_at(
        &script,
        position_of(content, "add_stability"),
        scope::initial_scope_for_uri(uri),
        &ScopeCtx::new(uri, None, None, None, None),
    );
    assert!(
        !scopes.contains(&Scope::State),
        "completion must not treat a random_list weight as a state, got {scopes:?}"
    );

    let content = "decisions = {\n\tbwc_cat = {\n\t\tcore_bwc = {\n\t\t\tstate_target = yes\n\t\t\tavailable = {\n\t\t\t\tFROM = { compliance > 50 }\n\t\t\t}\n\t\t}\n\t}\n}\n";
    let uri = "/mod/common/decisions/test.txt";
    let (script, _) = crate::parser::parser::parse_script(content);
    let (_, scopes) = crate::scope::scope_context::find_scope_context_at(
        &script,
        position_of(content, "compliance"),
        scope::initial_scope_for_uri(uri),
        &ScopeCtx::new(uri, None, None, None, None),
    );
    assert!(
        scopes.contains(&Scope::State),
        "completion must resolve FROM in a state_target decision as State, got {scopes:?}"
    );
}

#[test]
fn random_list_weight_is_not_a_state() {
    let content =
        "country_event = {\n\trandom_list = {\n\t\t10 = { add_stability = 0.1 }\n\t}\n}\n";
    let (identifier, scopes) =
        identifier_scopes(content, "/mod/common/events/test.txt", "add_stability");
    assert_eq!(identifier, "add_stability");
    assert!(
        !scopes.contains(&Scope::State),
        "a random_list weight must not push State, got {scopes:?}"
    );
}

/// Control: outside a weight block a bare numeric key is a state id.
#[test]
fn numeric_key_outside_random_list_is_still_a_state() {
    let content =
        "country_event = {\n\tsome_effect = {\n\t\t123 = { add_stability = 0.1 }\n\t}\n}\n";
    let (identifier, scopes) =
        identifier_scopes(content, "/mod/common/events/test.txt", "add_stability");
    assert_eq!(identifier, "add_stability");
    assert!(
        scopes.contains(&Scope::State),
        "a bare numeric key outside random_list is a state, got {scopes:?}"
    );
}

#[test]
fn from_in_state_targeted_decision_resolves_to_the_target_state() {
    for target in ["yes", "any_controlled_state"] {
        let content = format!(
            "decisions = {{\n\tbwc_cat = {{\n\t\tcore_bwc = {{\n\t\t\tstate_target = {target}\n\t\t\tavailable = {{\n\t\t\t\tFROM = {{ compliance > 50 }}\n\t\t\t}}\n\t\t}}\n\t}}\n}}\n"
        );
        let (identifier, scopes) =
            identifier_scopes(&content, "/mod/common/decisions/test.txt", "compliance");
        assert_eq!(identifier, "compliance");
        assert!(
            scopes.contains(&Scope::State),
            "FROM inside a state_target = {target} decision must be State, got {scopes:?}"
        );
    }
}

/// Control: without `state_target`, FROM is the country the decision is
/// evaluated for, which is what hover showed before.
#[test]
fn from_without_state_target_stays_a_country() {
    let content = "decisions = {\n\tbwc_cat = {\n\t\tcore_bwc = {\n\t\t\tavailable = {\n\t\t\t\tFROM = { has_war = yes }\n\t\t\t}\n\t\t}\n\t}\n}\n";
    let (identifier, scopes) =
        identifier_scopes(content, "/mod/common/decisions/test.txt", "has_war");
    assert_eq!(identifier, "has_war");
    assert!(
        !scopes.contains(&Scope::State),
        "FROM without state_target must stay Country, got {scopes:?}"
    );
}
