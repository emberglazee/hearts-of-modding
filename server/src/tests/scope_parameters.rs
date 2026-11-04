//! HOM004 must not fire on a key the enclosing block documents as a
//! PARAMETER.
//!
//! The engine parses a block's sub-keys against that block's own schema, so a
//! parameter is positional data, not a nested invocation — even when the same
//! word also names a trigger or effect with a narrower scope contract.
//! `promote_leader = yes` inside `add_country_leader_role = { ... }` is the
//! canonical collision: `promote_leader` is also a Character-scope effect
//! ("promotes general to field marshal"). Hearts of Minecraft ships the
//! parameter form 83 times, all inside Country-scope focus rewards and decision
//! effects, and every one of them was reported as a scope mismatch.

use crate::rules::v2_scope::V2ScopeRule;
use crate::scope::scope::initial_scope_for_uri;
use crate::test_support::TestCtx;
use tower_lsp_server::ls_types::Diagnostic;

/// Walk `input` as a decisions file (Country scope) with only HOM004 active.
fn walk_decisions(input: &str) -> Vec<Diagnostic> {
    let ctx = TestCtx::new().with_scope_validation(true);
    let uri = "/mod/common/decisions/test.txt";
    let initial = initial_scope_for_uri(uri);
    assert_eq!(
        initial,
        crate::scope::scope::Scope::Country,
        "decisions files must start in Country scope for these cases"
    );
    ctx.walk(input, uri, initial, vec![Box::new(V2ScopeRule)], vec![])
}

fn has_scope_mismatch_for(diags: &[Diagnostic], key: &str) -> bool {
    diags
        .iter()
        .any(|d| d.message.starts_with(&format!("'{}' is not valid", key)))
}

#[test]
fn promote_leader_parameter_inside_add_country_leader_role_is_not_a_mismatch() {
    let diags = walk_decisions(
        r#"cat = {
            dec = {
                complete_effect = {
                    add_country_leader_role = {
                        character = SPF_Colonial_administration
                        promote_leader = yes
                        country_leader = {
                            ideology = authoritarian_villagism
                            expire = "1965.1.1.1"
                        }
                    }
                }
            }
        }"#,
    );
    assert!(
        !has_scope_mismatch_for(&diags, "promote_leader"),
        "promote_leader is a documented parameter of add_country_leader_role — got: {:?}",
        diags.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn skill_parameter_inside_create_corps_commander_is_not_a_mismatch() {
    // Same shape, different family: `skill` is a Character/Unit trigger and a
    // documented parameter of every commander-creating effect.
    let diags = walk_decisions(
        r#"cat = {
            dec = {
                complete_effect = {
                    create_corps_commander = {
                        name = "Test"
                        skill = 4
                    }
                    create_field_marshal = {
                        name = "Test"
                        skill = 5
                    }
                }
            }
        }"#,
    );
    assert!(
        !has_scope_mismatch_for(&diags, "skill"),
        "skill is a documented parameter of the commander effects — got: {:?}",
        diags.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

/// Control: the same key OUTSIDE its documenting block is still a genuine
/// mismatch. Without this, a key-only allowlist would pass the tests above
/// while silently disabling HOM004 for every colliding parameter name.
#[test]
fn promote_leader_outside_its_parameter_block_still_fires() {
    let diags = walk_decisions(
        r#"cat = {
            dec = {
                complete_effect = {
                    every_state = {
                        limit = { always = yes }
                        promote_leader = yes
                    }
                }
            }
        }"#,
    );
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("promote_leader") && d.message.contains("State scope")),
        "promote_leader at State scope is a real mismatch — got: {:?}",
        diags.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

/// Control: a top-level effect invocation at Country scope is unaffected by the
/// parameter guard (the parent context there has no key at all).
#[test]
fn top_level_character_effect_still_fires() {
    let diags = walk_decisions(
        r#"cat = {
            dec = {
                complete_effect = {
                    promote_leader = yes
                }
            }
        }"#,
    );
    assert!(
        has_scope_mismatch_for(&diags, "promote_leader"),
        "promote_leader as a bare effect at Country scope is a real mismatch — got: {:?}",
        diags.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}
