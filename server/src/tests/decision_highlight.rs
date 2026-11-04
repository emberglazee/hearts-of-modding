//! `highlight_states_trigger` is a per-state filter: the engine evaluates its
//! body once for every state on the map, so State triggers there are valid even
//! though no iterator entered a state scope.
//!
//! Vanilla uses it 101 times (and Hearts of Minecraft 387), mostly as bare
//! `state = <id>` lines, but `is_owned_by` / `is_core_of` / `has_state_flag` /
//! `is_in_array` and country-local scripted triggers appear too. Without the
//! push, a State-only trigger such as `is_capital` was reported as a
//! Country-scope mismatch.

use crate::rules::v2_scope::V2ScopeRule;
use crate::scope::scope::{Scope, initial_scope_for_uri};
use crate::test_support::TestCtx;
use tower_lsp_server::ls_types::Diagnostic;

fn walk_decisions(input: &str) -> Vec<Diagnostic> {
    let ctx = TestCtx::new().with_scope_validation(true);
    let uri = "/mod/common/decisions/test.txt";
    let initial = initial_scope_for_uri(uri);
    assert_eq!(initial, Scope::Country);
    ctx.walk(input, uri, initial, vec![Box::new(V2ScopeRule)], vec![])
}

fn has_mismatch_for(diags: &[Diagnostic], key: &str) -> bool {
    diags
        .iter()
        .any(|d| d.message.starts_with(&format!("'{}' is not valid", key)))
}

#[test]
fn highlight_states_trigger_body_is_state_scoped() {
    let diags = walk_decisions(
        r#"cat = {
    dec = {
        highlight_states = {
            highlight_states_trigger = {
                is_capital = yes
                state_population > 100
            }
        }
    }
}"#,
    );
    assert!(
        !has_mismatch_for(&diags, "is_capital") && !has_mismatch_for(&diags, "state_population"),
        "the per-state filter body must validate in State scope, got: {:?}",
        diags.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

/// Control: the same triggers directly in a decision body are still a real
/// Country-scope mismatch, so the push is scoped to the filter block.
#[test]
fn the_same_triggers_outside_the_filter_still_fire() {
    let diags = walk_decisions(
        r#"cat = {
    dec = {
        available = {
            is_capital = yes
        }
    }
}"#,
    );
    assert!(
        has_mismatch_for(&diags, "is_capital"),
        "is_capital at Country scope is a real mismatch, got: {:?}",
        diags.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}
