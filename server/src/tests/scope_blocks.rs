//! HOM004 scope-block semantics.
//!
//! Three rules are pinned here, because getting any of them wrong produces
//! false positives on valid vanilla content (the review found 5,120 of them
//! across `common/ideas` and `common/characters`):
//!
//! 1. A block that names an evaluation scope pushes it. `allowed` is the
//!    canonical case: it is rendered as a structural keyword but the engine
//!    evaluates it on the owning country, so it must NOT sit in
//!    `transparent_block_types` (which bypasses `Scope::from_str`).
//! 2. A block that only wraps triggers inherits the enclosing scope
//!    (`AND`/`NOT`/`OR`, `limit`, `if`, `random_list`, `trigger`, ...).
//! 3. `common/characters/*.txt` is a character-definition file: the file starts
//!    in Country (so `ROOT` is the country), `characters` pushes Character, and
//!    the blocks inside a character body are handled per-block — `allowed` /
//!    `available` / `visible` are skipped (vanilla mixes country and character
//!    triggers in them), `on_add` / `on_remove` run on the character,
//!    `do_effect` runs on the country.
//!
//! Fixtures are minimal mod files; the URIs decide the initial scope, exactly
//! like production.

use crate::rules::v2_scope::V2ScopeRule;
use crate::scope::scope::{Scope, initial_scope_for_uri};
use crate::test_support::TestCtx;
use tower_lsp_server::ls_types::Diagnostic;

const IDEAS_URI: &str = "/mod/common/ideas/test_ideas.txt";
const CHARACTERS_URI: &str = "/mod/common/characters/test_characters.txt";
const DECISIONS_URI: &str = "/mod/common/decisions/test_decisions.txt";

fn walk(uri: &str, input: &str) -> Vec<Diagnostic> {
    let ctx = TestCtx::new().with_scope_validation(true);
    let initial = initial_scope_for_uri(uri);
    ctx.walk(input, uri, initial, vec![Box::new(V2ScopeRule)], vec![])
}

/// Keys the scope rule rejected, for readable assertion failures.
fn mismatched(diags: &[Diagnostic]) -> Vec<String> {
    diags
        .iter()
        .filter(|d| d.message.contains("is not valid in"))
        .map(|d| d.message.clone())
        .collect()
}

fn assert_no_mismatch(uri: &str, input: &str) {
    let bad = mismatched(&walk(uri, input));
    assert!(
        bad.is_empty(),
        "expected no scope diagnostics for {uri}, got {bad:#?}"
    );
}

/// Country triggers inside `allowed` / `allowed_civil_war` / `allowed_to_remove`
/// are valid — this is the review's headline false positive.
#[test]
fn idea_allowed_blocks_push_country() {
    assert_no_mismatch(
        IDEAS_URI,
        r#"ideas = {
            country = {
                test_idea = {
                    allowed = {
                        original_tag = AFG
                        has_government = fascism
                    }
                    allowed_civil_war = {
                        has_government = communism
                    }
                    allowed_to_remove = {
                        NOT = { has_government = democratic }
                    }
                }
            }
        }"#,
    );
}

/// Idea hooks that run on the country: `on_add` / `on_remove` / `do_effect` /
/// `cancel`. `cancel` and `do_effect` were never mapped; `on_add` / `on_remove`
/// only apply to the country outside a character body.
#[test]
fn idea_hooks_are_country_scoped() {
    assert_no_mismatch(
        IDEAS_URI,
        r#"ideas = {
            country = {
                test_idea = {
                    allowed = { always = yes }
                    on_add = {
                        add_ideas = some_idea
                    }
                    on_remove = {
                        remove_ideas = some_idea
                    }
                    cancel = {
                        OR = {
                            has_war_with = ENG
                            ROOT = { exists = no }
                        }
                    }
                    do_effect = {
                        NOT = { has_government = communism }
                    }
                }
            }
        }"#,
    );
}

/// Modifier containers are flat bags of modifier keys, not trigger blocks.
/// `targeted_modifier` and friends must not validate `tag` / `armor` against
/// the enclosing Idea scope.
#[test]
fn idea_modifier_bags_skip_scope_checks() {
    assert_no_mismatch(
        IDEAS_URI,
        r#"ideas = {
            country = {
                test_idea = {
                    modifier = {
                        stability_factor = 0.1
                    }
                    targeted_modifier = {
                        tag = SOV
                        attack_bonus_against = 0.2
                    }
                    research_bonus = {
                        armor = 0.15
                    }
                    equipment_bonus = {
                        armor = { build_cost_ic = -0.1 }
                    }
                }
            }
        }"#,
    );
}

/// The full character shape from vanilla: role blocks inherit Character, so
/// `skill` is valid; `allowed` / `available` / `visible` mix country and
/// character triggers and are skipped; `on_add` sets character flags; `do_effect`
/// runs on the country; `ROOT` inside a character body is the country.
#[test]
fn character_definition_file_scopes_are_real() {
    assert_eq!(
        initial_scope_for_uri(CHARACTERS_URI),
        Scope::Country,
        "a character file starts in Country so ROOT resolves to the country"
    );

    assert_no_mismatch(
        CHARACTERS_URI,
        r#"characters = {
            TEST_person = {
                name = TEST_person
                portraits = {
                    civilian = { large = GFX_portrait_TEST }
                }
                advisor = {
                    slot = high_command
                    idea_token = TEST_person
                    allowed = {
                        original_tag = AFG
                    }
                    available = {
                        is_country_leader = no
                    }
                    visible = {
                        has_character_flag = TEST_ready
                    }
                    on_add = {
                        set_character_flag = TEST_hired
                        hidden_effect = {
                            ROOT = { add_ideas = TEST_idea }
                        }
                    }
                    on_remove = {
                        clr_character_flag = TEST_hired
                    }
                    traits = { army_chief_defensive_2 }
                    cost = 100
                    ai_will_do = { factor = 1.0 }
                }
                corps_commander = {
                    skill = 4
                    attack_skill = 5
                    defense_skill = 2
                    planning_skill = 3
                    logistics_skill = 3
                    traits = { infantry_officer }
                }
                country_leader = {
                    ideology = despotism
                    expire = "1965.1.1.1"
                    id = -1
                    do_effect = {
                        NOT = { has_government = communism }
                    }
                }
            }
        }"#,
    );
}

/// Positive control: the rule must still reject a Country trigger inside a
/// state-scoped iterator, so the fix cannot have disabled HOM004 wholesale.
#[test]
fn state_scoped_body_still_rejects_country_triggers() {
    let diags = walk(
        DECISIONS_URI,
        r#"test_cat = {
            test_dec = {
                complete_effect = {
                    every_state = {
                        has_government = democratic
                    }
                }
            }
        }"#,
    );
    let bad = mismatched(&diags);
    assert!(
        bad.iter().any(|m| m.contains("'has_government'")),
        "expected has_government to be rejected in State scope, got {bad:#?}"
    );
}

/// Positive control: `limit` inherits the enclosing State scope, so a
/// State-scoped trigger inside it stays valid.
#[test]
fn limit_inherits_enclosing_scope() {
    assert_no_mismatch(
        DECISIONS_URI,
        r#"test_cat = {
            test_dec = {
                complete_effect = {
                    every_state = {
                        limit = { is_coastal = yes }
                        add_building_construction = {
                            type = infrastructure
                            level = 1
                        }
                    }
                }
            }
        }"#,
    );
}

/// A known character token used as a scope key (`TOKEN = { ... }`) switches to
/// Character scope, so character triggers inside it are valid. This only works
/// with scanner data — the CLI's `--scope-validation` mode has none, which is
/// why its remaining corpus hits are all this shape.
#[test]
fn character_token_block_switches_to_character_scope() {
    let ctx = TestCtx::new().with_scope_validation(true).with_file(
        "/mod/common/characters/other.txt",
        "characters = { OTHER_person = { name = OTHER_person } }",
    );
    let uri = "/mod/events/test_event.txt";
    let diags = ctx.walk(
        r#"country_event = {
            immediate = {
                OTHER_person = { set_character_flag = test_flag }
            }
        }"#,
        uri,
        initial_scope_for_uri(uri),
        vec![Box::new(V2ScopeRule)],
        vec![],
    );
    let bad = mismatched(&diags);
    assert!(
        bad.is_empty(),
        "a seeded character token must switch to Character scope, got {bad:#?}"
    );
}

/// `allowed` is a rendered keyword AND a scope-pushing block: the two data
/// lists must stay ordered that way, or the transparency short-circuit will
/// swallow the push again.
#[test]
fn keyword_blocks_are_a_superset_of_transparent_blocks() {
    use crate::data::hoi4_data::{get_keyword_block_types, is_transparent_block};

    assert!(
        is_transparent_block("trigger"),
        "trigger inherits the enclosing scope (state-scoped on_action bodies)"
    );
    assert!(
        !is_transparent_block("allowed"),
        "allowed pushes Country — listing it as transparent is the reviewed bug"
    );

    let keywords = get_keyword_block_types();
    for name in ["AND", "NOT", "OR", "allowed", "limit", "trigger"] {
        assert!(
            keywords.iter().any(|k| k.eq_ignore_ascii_case(name)),
            "{name} must render as a structural keyword"
        );
    }
}
