//! `state_target` accepts more than `yes`, and every form makes `FROM` the
//! target STATE inside a decision body.
//!
//! Vanilla writes all of these:
//!
//! ```text
//! state_target = yes                     (209 sites)
//! state_target = any_controlled_state    (34)
//! state_target = any_owned_state         (20)
//! state_target = europe | asia | africa  (15 across the continent names)
//! ```
//!
//! The walker used to recognise only the boolean, so a decision using the
//! scoped form left `FROM` resolving to Country and every State trigger inside
//! `FROM = { ... }` was reported as a scope mismatch
//! (Hearts of Minecraft `BWC_decisions.txt`: `compliance`, `resistance`,
//! `is_claimed_by`).
//!
//! The control tests matter as much as the fix: with no `state_target` the
//! same body is a genuine mismatch, so the flag is what clears it and a
//! blanket suppression cannot pass.

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

fn mismatch_keys(diags: &[Diagnostic]) -> Vec<String> {
    diags
        .iter()
        .filter(|d| d.message.contains("is not valid in"))
        .map(|d| d.message.split('\'').nth(1).unwrap_or("").to_string())
        .collect()
}

/// The body the engine evaluates with FROM = the target state.
const STATE_BODY: &str = r#"
        available = {
            FROM = {
                is_fully_controlled_by = BWC
                compliance > 50
                resistance < 5
            }
        }
        visible = {
            FROM = {
                is_claimed_by = BWC
            }
        }"#;

fn decision_with_target(state_target: Option<&str>) -> String {
    let line = match state_target {
        Some(v) => format!("\t\tstate_target = {v}\n"),
        None => String::new(),
    };
    format!(
        r#"bwc_cat = {{
    core_bwc = {{
{line}        days_remove = 50
{STATE_BODY}
    }}
}}"#
    )
}

#[test]
fn scoped_state_target_makes_from_the_target_state() {
    // `any_controlled_state` is a state iterator, not a boolean — vanilla
    // writes it 34 times.
    let diags = walk_decisions(&decision_with_target(Some("any_controlled_state")));
    let keys = mismatch_keys(&diags);
    assert!(
        keys.is_empty(),
        "FROM is the target state; none of these should be mismatches: {:?}",
        keys
    );
}

#[test]
fn owned_state_and_continent_forms_also_mark_the_decision() {
    for value in ["any_owned_state", "europe", "asia"] {
        let diags = walk_decisions(&decision_with_target(Some(value)));
        let keys = mismatch_keys(&diags);
        assert!(
            keys.is_empty(),
            "state_target = {value} must mark FROM as the target state, got: {:?}",
            keys
        );
    }
}

#[test]
fn boolean_state_target_still_works() {
    let diags = walk_decisions(&decision_with_target(Some("yes")));
    let keys = mismatch_keys(&diags);
    assert!(keys.is_empty(), "state_target = yes regressed: {:?}", keys);
}

/// Control: no `state_target` means the engine does not target a state, so
/// these State triggers inside `FROM` are real mismatches.
#[test]
fn without_state_target_the_same_body_is_a_mismatch() {
    let diags = walk_decisions(&decision_with_target(None));
    let keys = mismatch_keys(&diags);
    assert!(
        keys.contains(&"compliance".to_string()) && keys.contains(&"is_claimed_by".to_string()),
        "control must still report state triggers when the decision is not state-targeted: {:?}",
        keys
    );
}
