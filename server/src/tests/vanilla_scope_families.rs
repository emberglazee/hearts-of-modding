//! Initial-scope and structural-scope rules for the vanilla definition families.
//!
//! The 2026-11-15 vanilla corpus sweep (8,347 files, 697k diagnostics) measured
//! 557,260 HOM004 scope mismatches, of which 96 % sat in two
//! `common/scripted_localisation/` files and the rest clustered in a handful of
//! file families that resolved to `Global` when they are not global at all.
//!
//! Each test here pins one measured family against a minimal fixture shaped
//! like the vanilla content that produced the diagnostic. Fixtures are not
//! invented shapes: every key below was taken from the vanilla file named in
//! the test.
//!
//! The negative controls matter as much as the positives — a fix that silences
//! a real mismatch is a regression, not a fix.

use crate::rules::v2_scope::V2ScopeRule;
use crate::scope::scope::{Scope, initial_scope_for_uri};
use crate::test_support::TestCtx;
use tower_lsp_server::ls_types::Diagnostic;

const OOB_URI: &str = "/mod/history/units/ZZZ_1936.txt";
const SCRIPTED_LOC_URI: &str = "/mod/common/scripted_localisation/zz_probe.txt";
const DYNAMIC_MODIFIER_URI: &str = "/mod/common/dynamic_modifiers/zz_probe.txt";
const COUNTRY_LEADER_TRAITS_URI: &str = "/mod/common/country_leader/zz_traits.txt";
const UNIT_LEADER_TRAITS_URI: &str = "/mod/common/unit_leader/zz_traits.txt";
const EVENTS_URI: &str = "/mod/events/zz_events.txt";
const RAIDS_URI: &str = "/mod/common/raids/zz_raids.txt";
const DOCTRINES_URI: &str = "/mod/common/doctrines/tracks/zz_tracks.txt";
const SPECIAL_PROJECTS_URI: &str = "/mod/common/special_projects/projects/zz_projects.txt";
const OPERATIONS_URI: &str = "/mod/common/operations/zz_operations.txt";
const ON_ACTIONS_URI: &str = "/mod/common/on_actions/zz_on_actions.txt";

fn walk(uri: &str, input: &str) -> Vec<Diagnostic> {
    let ctx = TestCtx::new().with_scope_validation(true);
    let initial = initial_scope_for_uri(uri);
    ctx.walk(input, uri, initial, vec![Box::new(V2ScopeRule)], vec![])
}

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

// ---------------------------------------------------------------------------
// Initial scope of the definition families
// ---------------------------------------------------------------------------

/// `history/units/*.txt` is order-of-battle script: country-level keys at the
/// top level. Vanilla `history/units/SOV_1939.txt` has 6,844 such lines.
#[test]
fn order_of_battle_starts_in_country() {
    assert_eq!(initial_scope_for_uri(OOB_URI), Scope::Country);
    assert_no_mismatch(
        OOB_URI,
        r#"
division_template = {
    name = "ZZ Test Division"
    regiments = { infantry = { x = 0 y = 0 } }
}
add_equipment_production = {
    equipment = { type = infantry_equipment amount = 100 }
    requested_factory = arms_factory
}
original_tag = GER
has_government = fascism
"#,
    );
}

/// `common/scripted_localisation/` bodies are evaluated in whatever scope
/// displays the string — polymorphic, like `scripted_effects`. Widening the
/// file to Country would silence real mismatches by claiming knowledge the
/// file cannot have, so the file gets its own structural scope instead.
#[test]
fn scripted_localisation_is_polymorphic_not_country() {
    assert_eq!(
        initial_scope_for_uri(SCRIPTED_LOC_URI),
        Scope::ScriptedLocalisation
    );
    assert_no_mismatch(
        SCRIPTED_LOC_URI,
        r#"
defined_text = {
    name = GetZZLeader
    text = {
        trigger = { has_government = democratic }
        localization_key = ZZ_democratic
    }
    text = {
        localization_key = ZZ_other
    }
}
"#,
    );
}

/// A dynamic modifier body holds modifier keys directly — `stability_factor`
/// has no `modifier = { }` wrapper — so it is a flat bag, not a trigger body.
#[test]
fn dynamic_modifier_body_is_a_modifier_bag() {
    assert_no_mismatch(
        DYNAMIC_MODIFIER_URI,
        r#"
ZZ_test_dm = {
    icon = GFX_idea_test
    enable = { has_government = fascism }
    stability_factor = 0.1
    war_support_factor = 0.05
}
"#,
    );
}

/// Country-leader trait bodies are modifier bags too: `experience_gain_navy`
/// and `compliance_growth` are modifiers, and `planning_gain_factor` collides
/// with no trigger at all.
#[test]
fn country_leader_trait_body_is_a_modifier_bag() {
    assert_no_mismatch(
        COUNTRY_LEADER_TRAITS_URI,
        r#"
leader_traits = {
    zz_trait = {
        random = no
        experience_gain_navy = 0.1
        compliance_growth = 0.05
    }
}
"#,
    );
}

// ---------------------------------------------------------------------------
// Structural blocks inside those families
// ---------------------------------------------------------------------------

/// A unit-leader trait's `gain_xp` is a DIVISION trigger gate — vanilla gates
/// it on `temperature` and `has_combat_modifier`. The `gain_xp = 10` effect is
/// a scalar and never reaches the block arm.
#[test]
fn unit_leader_gain_xp_is_a_division_trigger() {
    assert_no_mismatch(
        UNIT_LEADER_TRAITS_URI,
        r#"
leader_traits = {
    commando = {
        type = corps_commander
        gain_xp = {
            OR = {
                temperature < -15
                has_combat_modifier = paradrop
            }
        }
    }
}
"#,
    );
}

/// `unit_trigger` runs on the division, so `owner = { ... }` chains to Country.
#[test]
fn unit_leader_unit_trigger_runs_on_the_division() {
    assert_no_mismatch(
        UNIT_LEADER_TRAITS_URI,
        r#"
leader_traits = {
    zz_trait = {
        unit_trigger = {
            OR = {
                is_fighting_in_terrain = desert
                owner = { has_idea = zz_idea }
            }
        }
    }
}
"#,
    );
}

/// Event definitions push their own scope. The file stays Global (that is where
/// `add_namespace` lives), but `country_event` bodies are country script and
/// `unit_leader_event` bodies run on the owner country with the leader in FROM
/// — vanilla `events/BBA_Italy.txt` puts `activate_mission` straight in options.
#[test]
fn event_definitions_push_their_runtime_scope() {
    assert_no_mismatch(
        EVENTS_URI,
        r#"
add_namespace = zz

country_event = {
    id = zz.1
    title = zz.1.t
    trigger = { has_government = democratic }
    option = { name = zz.1.a add_political_power = 50 }
}

unit_leader_event = {
    id = zz.2
    title = zz.2.t
    option = { name = zz.2.a activate_mission = { type = infiltration } }
}

state_event = {
    id = zz.3
    title = zz.3.t
    trigger = { is_coastal = yes }
    option = { name = zz.3.a }
}
"#,
    );
}

/// Raid `success`/`failure` are probability bags; `victim_effects` runs on the
/// victim country.
#[test]
fn raid_outcome_blocks_are_weight_bags() {
    assert_no_mismatch(
        RAIDS_URI,
        r#"
zz_raid = {
    success = {
        base = 0.75
        resistance = { weight = 0.25 start_weight = -0.2 }
        victim_effects = { add_stability = -0.1 }
    }
}
"#,
    );
}

/// A doctrine track file's depth-1 block keys NAME a track (`armor = { ... }`),
/// and those names collide with trigger names — `armor` is a Unit trigger.
#[test]
fn doctrine_track_definition_keys_are_not_usages() {
    assert_no_mismatch(
        DOCTRINES_URI,
        r#"
armor = {
    name = zz_track
    background = GFX_doctrine_armor
    modifier = { max_organisation = 5 }
}
operations = {
    name = zz_operations
    background = GFX_doctrine_operations
}
"#,
    );
}

/// Special-project scientist blocks evaluate on the scientist.
#[test]
fn special_project_scientist_blocks_run_on_the_scientist() {
    assert_no_mismatch(
        SPECIAL_PROJECTS_URI,
        r#"
zz_project = {
    scientist_effects = {
        if = {
            limit = { has_scientist_level > 2 }
            add_scientist_xp = { amount = 100 }
        }
    }
}
"#,
    );
}

/// Operation target selection runs on the candidate state.
#[test]
fn operation_target_selection_runs_on_the_state() {
    assert_no_mismatch(
        OPERATIONS_URI,
        r#"
zz_operation = {
    selection_target_state = { has_resistance = yes }
}
"#,
    );
}

/// `ROOT` inside an `on_*` hook resolves to the hook's runtime scope, not to
/// the file marker. Vanilla `07_nsb_on_actions.txt` does this 763 times.
#[test]
fn root_inside_on_action_hook_resolves_to_the_hook_scope() {
    assert_no_mismatch(
        ON_ACTIONS_URI,
        r#"
on_unit_leader_created = {
    root = { add_unit_leader_trait = zz_trait }
}
"#,
    );
}

// ---------------------------------------------------------------------------
// Negative controls — these MUST still be reported
// ---------------------------------------------------------------------------

/// A country trigger inside a state event is a real mismatch.
#[test]
fn country_trigger_in_state_event_is_still_reported() {
    let bad = mismatched(&walk(
        EVENTS_URI,
        r#"
state_event = {
    id = zz.9
    title = zz.9.t
    trigger = { has_government = democratic }
    option = { name = zz.9.a }
}
"#,
    ));
    assert!(
        bad.iter().any(|m| m.contains("has_government")),
        "expected has_government to be rejected in a state event, got {bad:#?}"
    );
}

/// A state trigger at country level in an order-of-battle file is a real
/// mismatch — the Country initial scope must not become a blanket silence.
#[test]
fn state_trigger_in_order_of_battle_is_still_reported() {
    let bad = mismatched(&walk(
        OOB_URI,
        r#"
division_template = { name = "ZZ" }
is_coastal = yes
"#,
    ));
    assert!(
        bad.iter().any(|m| m.contains("is_coastal")),
        "expected is_coastal to be rejected at country scope, got {bad:#?}"
    );
}

// ---------------------------------------------------------------------------
// Effect bodies that are parameter bags, not script
// ---------------------------------------------------------------------------

const NATIONAL_FOCUS_URI: &str = "/mod/common/national_focus/zz_focus.txt";

/// `add_*_role` bodies carry role stats, not trigger script. Vanilla
/// `common/national_focus/greece.txt:89` writes
/// `add_corps_commander_role = { character = GRE_x skill = 1 attack_skill = 1 … }`
/// inside a focus `completion_reward`, which runs in Country scope; validating
/// the body there reported `skill` (a Character/Unit key) 48 times across
/// vanilla national focuses.
#[test]
fn leader_role_effect_bodies_accept_their_documented_parameters() {
    let input = "focus = {\n\tid = zz_role\n\tcompletion_reward = {\n\t\tadd_corps_commander_role = {\n\t\t\tcharacter = ZZZ_leader\n\t\t\tskill = 2\n\t\t\tattack_skill = 1\n\t\t\tdefense_skill = 1\n\t\t\tplanning_skill = 1\n\t\t\tlogistics_skill = 1\n\t\t}\n\t\tadd_field_marshal_role = {\n\t\t\tcharacter = ZZZ_leader\n\t\t\tskill = 3\n\t\t}\n\t\tadd_naval_commander_role = {\n\t\t\tCharacter = ZZZ_leader\n\t\t\tskill = 1\n\t\t}\n\t\tadd_country_leader_role = {\n\t\t\tcharacter = ZZZ_leader\n\t\t\tcountry_leader = {\n\t\t\t\tideology = moderatism\n\t\t\t\texpire = \"1965.1.1.1\"\n\t\t\t\ttraits = { ZZZ_trait }\n\t\t\t}\n\t\t}\n\t}\n}\n";
    assert_no_mismatch(NATIONAL_FOCUS_URI, input);
}

/// Negative control: the same key outside a role body is still a real mismatch,
/// so the parameter declaration cannot swallow a genuine country-scope `skill`.
#[test]
fn bare_role_stat_outside_a_role_body_is_still_flagged() {
    let input = "focus = {\n\tid = zz_role\n\tcompletion_reward = {\n\t\tskill = 2\n\t}\n}\n";
    let bad = mismatched(&walk(NATIONAL_FOCUS_URI, input));
    assert!(
        !bad.is_empty(),
        "a bare `skill` in a country reward must still be reported"
    );
}

/// The role body is a normal Country block with a parameter schema, not a
/// blanket bag: anything the schema does not declare is still validated, so a
/// state-scoped effect nested in a role body is reported. This is the coverage a
/// body-wide `ModifierBag` would have traded away.
#[test]
fn undeclared_key_inside_a_role_body_is_still_validated() {
    let input = "focus = {\n\tid = zz_role\n\tcompletion_reward = {\n\t\tadd_corps_commander_role = {\n\t\t\tcharacter = ZZZ_leader\n\t\t\tskill = 2\n\t\t\tadd_extra_state_shared_building_slots = 1\n\t\t}\n\t}\n}\n";
    let bad = mismatched(&walk(NATIONAL_FOCUS_URI, input));
    assert!(
        !bad.is_empty(),
        "a state-scoped effect inside a role body must still be reported"
    );
}

/// `complete_tooltip` is display-only. Vanilla `common/national_focus/generic.txt:639`
/// lists a state-scoped effect there because the tooltip renders it, not because
/// the engine runs it in the focus's Country scope.
#[test]
fn complete_tooltip_bodies_are_display_only() {
    let input = "focus = {\n\tid = zz_tip\n\tcomplete_tooltip = {\n\t\tnavy_experience = 25\n\t\tadd_extra_state_shared_building_slots = 3\n\t\tadd_building_construction = {\n\t\t\ttype = dockyard\n\t\t\tlevel = 1\n\t\t\tinstant_build = yes\n\t\t}\n\t}\n\tcompletion_reward = {\n\t\tadd_political_power = 10\n\t}\n}\n";
    assert_no_mismatch(NATIONAL_FOCUS_URI, input);
}

/// Negative control: the same effect in `completion_reward`, which the engine
/// does execute in Country scope, is still reported.
#[test]
fn state_effect_in_completion_reward_is_still_flagged() {
    let input = "focus = {\n\tid = zz_tip\n\tcompletion_reward = {\n\t\tadd_extra_state_shared_building_slots = 3\n\t}\n}\n";
    let bad = mismatched(&walk(NATIONAL_FOCUS_URI, input));
    assert!(
        !bad.is_empty(),
        "a state-scoped effect in an executed country reward must still be reported"
    );
}
