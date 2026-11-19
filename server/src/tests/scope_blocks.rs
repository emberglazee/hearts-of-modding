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

/// A country-leader trait id may be the same string as a character id — the
/// engine keeps the two namespaces apart. Resolving the trait key through the
/// character index pushed Character onto the trait body, so every country
/// modifier inside it was reported as out-of-scope.
#[test]
fn trait_id_colliding_with_a_character_is_still_a_property_bag() {
    let ctx = TestCtx::new().with_scope_validation(true).with_file(
        "/mod/common/characters/other.txt",
        "characters = { SPF_Colonial_administration = { name = SPF_Colonial_administration } }",
    );
    let uri = "/mod/common/country_leader/00_traits.txt";
    let diags = ctx.walk(
        r#"leader_traits = {
            SPF_Colonial_administration = {
                stability_factor = -0.15
                offence = -0.25
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
        "a trait id may collide with a character id, got {bad:#?}"
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

/// `keep_unit_leaders_trigger` is evaluated once per unit leader, not on the
/// country running `start_civil_war`. A unit leader IS a character, so its body
/// is Character scope: vanilla writes `has_trait`, `has_character_flag`,
/// `is_character` and `has_unit_leader_flag` there, and only `has_trait` would
/// also be legal in a division (Unit) scope.
#[test]
fn keep_unit_leaders_trigger_runs_in_character_scope() {
    assert_no_mismatch(
        DECISIONS_URI,
        r#"TEST_category = {
            TEST_decision = {
                allowed = { always = yes }
                complete_effect = {
                    start_civil_war = {
                        ruling_party = communism
                        size = 0.3
                        keep_unit_leaders_trigger = {
                            has_trait = hidden_sympathies
                            has_unit_leader_flag = is_loyal
                        }
                    }
                }
            }
        }"#,
    );
}

/// The negative control for the rule above: a decision's own `allowed` block is
/// still Country scope, so a character trigger there stays an error. Without
/// this, `has_trait` could have been blanket-allowed instead of scoped.
#[test]
fn has_trait_is_still_rejected_in_a_decision_condition() {
    let bad = mismatched(&walk(
        DECISIONS_URI,
        r#"TEST_category = {
            TEST_decision = {
                allowed = { has_trait = hidden_sympathies }
            }
        }"#,
    ));
    assert_eq!(
        bad.len(),
        1,
        "a decision's allowed block is Country scope, got {bad:#?}"
    );
}

/// A leader trait's `allowed` block gates the leader who would receive it, so
/// `has_trait` is legal there even though `allowed` pushes Country elsewhere.
#[test]
fn a_leader_trait_gate_runs_against_the_leader() {
    assert_no_mismatch(
        "/mod/common/unit_leader/00_traits.txt",
        r#"leader_traits = {
            substance_addict = {
                type = all
                trait_type = status_trait
                allowed = { has_trait = substance_abuser }
            }
        }"#,
    );
}

/// The three per-target filter blocks carry an explicit pushed scope in the
/// data; the resolver reads it through `lookup_pushes_scope`.
#[test]
fn per_target_filter_blocks_push_their_scope() {
    use crate::data::hoi4_data::lookup_pushes_scope;

    assert_eq!(
        lookup_pushes_scope("keep_unit_leaders_trigger"),
        Some(Scope::Character),
        "the block is evaluated once per unit leader, and a leader is a character"
    );
    assert_eq!(
        lookup_pushes_scope("keep_scientists_trigger"),
        Some(Scope::Character),
        "the block is evaluated once per scientist"
    );
    assert_eq!(
        lookup_pushes_scope("states_filter"),
        Some(Scope::State),
        "the block is evaluated once per state"
    );
}

/// `PREV` inside a `limit` is the scope the iterator was called from, not the
/// scope being iterated. `every_state = { limit = { PREV = { tag = X } } }`
/// therefore checks the country that called `every_state`; the transparent
/// `limit` frame must not shift PREV onto the state.
#[test]
fn prev_inside_a_limit_is_the_calling_scope() {
    assert_no_mismatch(
        DECISIONS_URI,
        r#"TEST_category = {
            TEST_decision = {
                remove_effect = {
                    random_country = {
                        limit = { always = yes }
                        every_state = {
                            limit = {
                                is_core_of = prev
                                OR = { PREV = { tag = ASP } }
                            }
                        }
                    }
                }
            }
        }"#,
    );
}

/// The other half of the rule: PREV inside a state iterator's limit is a
/// country, so a state-only trigger there is a real error. Without this, the
/// fix above could have been "make PREV Country everywhere".
#[test]
fn prev_inside_a_limit_is_not_the_state_being_iterated() {
    let bad = mismatched(&walk(
        DECISIONS_URI,
        r#"TEST_category = {
            TEST_decision = {
                remove_effect = {
                    random_country = {
                        every_state = {
                            limit = { PREV = { is_capital = yes } }
                        }
                    }
                }
            }
        }"#,
    ));
    assert_eq!(
        bad.len(),
        1,
        "is_capital is State-only and PREV is the country, got {bad:#?}"
    );
}

/// A technology's per-sub-unit blocks run in Unit scope.
///
/// `mechanised_infantry = { motorized = { hardness = 1.0 } }` is vanilla
/// (`common/technologies/infantry.txt:2347`): the block is keyed by a sub-unit
/// and the entries inside are that sub-unit's stats. Validated in Country scope
/// `hardness` (Unit-only) was reported, on 5 vanilla lines and every mod copy.
#[test]
fn technology_subunit_block_runs_in_unit_scope() {
    const UNITS: &str = r#"sub_units = {
    motorized = {
        abbreviation = "MOT"
        group = mobile
        combat_width = 2
    }
}"#;
    const TECH: &str = r#"technologies = {
    mechanised_infantry = {
        enable_subunits = {
            motorized
        }
        motorized = {
            hardness = 1.0
        }
    }
}"#;
    const URI: &str = "/mod/common/technologies/00_test.txt";
    let ctx = TestCtx::new()
        .with_scope_validation(true)
        .with_file("/mod/common/units/00_units.txt", UNITS);
    let diags = ctx.walk(
        TECH,
        URI,
        initial_scope_for_uri(URI),
        vec![Box::new(V2ScopeRule)],
        vec![],
    );
    let bad = mismatched(&diags);
    assert!(
        bad.is_empty(),
        "expected no scope diagnostics, got {bad:#?}"
    );
}

/// The sub-unit rule is keyed on the block, not on the file: a unit stat written
/// straight into a technology body is still Country scope and still reported.
#[test]
fn unit_stat_outside_a_subunit_block_still_reports() {
    const TECH: &str = r#"technologies = {
    mechanised_infantry = {
        hardness = 1.0
    }
}"#;
    const URI: &str = "/mod/common/technologies/00_test.txt";
    let ctx = TestCtx::new()
        .with_scope_validation(true)
        .with_unit_types(&["motorized"]);
    let diags = ctx.walk(
        TECH,
        URI,
        initial_scope_for_uri(URI),
        vec![Box::new(V2ScopeRule)],
        vec![],
    );
    let bad = mismatched(&diags);
    assert!(
        bad.iter().any(|m| m.contains("hardness")),
        "expected 'hardness' to be reported in Country scope, got {bad:#?}"
    );
}

/// A dynamic modifier's `enable` / `remove_trigger` are polymorphic: the
/// modifier runs in whatever scope it was added to, and the definition does not
/// say which. Vanilla has both shapes, so neither may be reported.
#[test]
fn dynamic_modifier_conditions_are_polymorphic() {
    // Country-shaped: bare country triggers, `ROOT` for the attached scope.
    const COUNTRY_DM: &str = r#"sabotaged_resources = {
    enable = {
        has_civil_war = yes
        ROOT = {
            has_resistance = no
        }
    }
    remove_trigger = {
        ROOT = {
            has_resistance = yes
        }
    }
    icon = GFX_modifiers_sabotaged_resource
}"#;
    // State-shaped: bare state triggers, no ROOT wrapper at all
    // (SEA_dynamic_modifiers.txt:1316).
    const STATE_DM: &str = r#"PRC_guerilla_cells_active_1 = {
    enable = { always = yes }
    remove_trigger = {
        OR = {
            controller = {
                NOT = { has_war_with = PRC }
            }
            AND = {
                NOT = { is_core_of = controller }
                compliance > 60
            }
        }
    }
    resistance_decay = -0.1
}"#;
    const URI: &str = "/mod/common/dynamic_modifiers/00_dm.txt";
    for src in [COUNTRY_DM, STATE_DM] {
        let ctx = TestCtx::new().with_scope_validation(true);
        let diags = ctx.walk(
            src,
            URI,
            initial_scope_for_uri(URI),
            vec![Box::new(V2ScopeRule)],
            vec![],
        );
        let bad = mismatched(&diags);
        assert!(
            bad.is_empty(),
            "expected no scope diagnostics, got {bad:#?}"
        );
    }
}

/// The dynamic-modifier rule is gated on the file: a state-only trigger written
/// in an idea's country block is still reported.
#[test]
fn state_trigger_in_an_idea_still_reports() {
    const IDEA: &str = r#"ideas = {
    country = {
        my_idea = {
            allowed = { always = yes }
            modifier = { stability_factor = 0.1 }
            has_resistance = no
        }
    }
}"#;
    const URI: &str = "/mod/common/ideas/00_ideas.txt";
    let ctx = TestCtx::new().with_scope_validation(true);
    let diags = ctx.walk(
        IDEA,
        URI,
        initial_scope_for_uri(URI),
        vec![Box::new(V2ScopeRule)],
        vec![],
    );
    let bad = mismatched(&diags);
    assert!(
        bad.iter().any(|m| m.contains("has_resistance")),
        "expected 'has_resistance' to be reported outside dynamic modifiers, got {bad:#?}"
    );
}
