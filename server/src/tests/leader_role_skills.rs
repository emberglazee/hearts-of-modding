//! HOM1005: a leader-role block given a stat from a different role.
//!
//! The engine reads a role block's stats positionally against that role's own
//! schema. Writing `planning_skill` on a naval commander therefore does not
//! error at load: the stat is dropped and the leader arrives with the default
//! value, which is invisible in game. The rule fires only for a key that is a
//! documented leader stat somewhere (so typos and unrelated parameters stay out
//! of scope), inside a block whose parameter table has the leader-role shape.

use crate::rules::leader_role_skills::LeaderRoleSkillRule;
use crate::scope::scope::initial_scope_for_uri;
use crate::test_support::TestCtx;
use tower_lsp_server::ls_types::{Diagnostic, NumberOrString};

const URI: &str = "/mod/common/national_focus/zz_focus.txt";

fn walk(input: &str) -> Vec<Diagnostic> {
    let ctx = TestCtx::new();
    let initial = initial_scope_for_uri(URI);
    ctx.walk(
        input,
        URI,
        initial,
        vec![Box::new(LeaderRoleSkillRule)],
        vec![],
    )
}

fn wrong_skill(diags: &[Diagnostic]) -> Vec<&Diagnostic> {
    diags
        .iter()
        .filter(|d| matches!(&d.code, Some(NumberOrString::String(code)) if code == "HOM1005"))
        .collect()
}

/// A national focus whose reward carries the given role block body.
fn focus_with(block: &str, body: &str) -> String {
    format!(
        "focus_tree = {{\n\tid = zz_tree\n\tfocus = {{\n\t\tid = zz_focus\n\t\tcompletion_reward = {{\n\t\t\t{block} = {{\n{body}\n\t\t\t}}\n\t\t}}\n\t}}\n}}\n"
    )
}

#[test]
fn army_stat_on_a_naval_commander_is_reported() {
    let diags = walk(&focus_with(
        "add_naval_commander_role",
        "\t\t\t\tcharacter = GER_admiral\n\t\t\t\tskill = 4\n\t\t\t\tattack_skill = 3\n\t\t\t\tdefense_skill = 3\n\t\t\t\tplanning_skill = 2\n\t\t\t\tcoordination_skill = 4",
    ));
    let found = wrong_skill(&diags);
    assert_eq!(found.len(), 1, "one misplaced stat: {:?}", diags);
    let msg = &found[0].message;
    assert!(
        msg.contains("'planning_skill' is not a parameter of add_naval_commander_role"),
        "{msg}"
    );
    assert!(
        msg.contains("It belongs to corps_commander and field_marshal."),
        "the message must name the roles that do take it, once each: {msg}"
    );
    assert!(
        msg.contains("maneuvering_skill") && msg.contains("coordination_skill"),
        "the message must list this role's own stats: {msg}"
    );
    assert!(
        !msg.contains("planning_skill, ") && !msg.contains(", planning_skill"),
        "the rejected stat must not appear in the accepted list: {msg}"
    );
}

#[test]
fn the_roles_own_stats_pass() {
    let diags = walk(&focus_with(
        "add_naval_commander_role",
        "\t\t\t\tcharacter = GER_admiral\n\t\t\t\tskill = 4\n\t\t\t\tattack_skill = 3\n\t\t\t\tdefense_skill = 3\n\t\t\t\tmaneuvering_skill = 3\n\t\t\t\tcoordination_skill = 4",
    ));
    assert!(wrong_skill(&diags).is_empty(), "{:?}", diags);
}

#[test]
fn army_roles_keep_their_own_stats() {
    let diags = walk(&focus_with(
        "add_corps_commander_role",
        "\t\t\t\tcharacter = GER_general\n\t\t\t\tskill = 4\n\t\t\t\tplanning_skill = 2\n\t\t\t\tlogistics_skill = 1",
    ));
    assert!(wrong_skill(&diags).is_empty(), "{:?}", diags);
}

#[test]
fn naval_stats_on_an_army_role_are_reported_in_the_other_direction() {
    let diags = walk(&focus_with(
        "add_field_marshal_role",
        "\t\t\t\tcharacter = GER_general\n\t\t\t\tskill = 5\n\t\t\t\tcoordination_skill = 3",
    ));
    let found = wrong_skill(&diags);
    assert_eq!(found.len(), 1, "{:?}", diags);
    assert!(
        found[0].message.contains("add_field_marshal_role"),
        "{:?}",
        found[0]
    );
}

#[test]
fn the_create_forms_are_covered_too() {
    let diags = walk(&focus_with(
        "create_navy_leader",
        "\t\t\t\tname = \"Test Admiral\"\n\t\t\t\tskill = 4\n\t\t\t\tlogistics_skill = 2",
    ));
    assert_eq!(wrong_skill(&diags).len(), 1, "{:?}", diags);
}

#[test]
fn a_misspelled_stat_is_out_of_scope_for_this_rule() {
    // `planing_skill` is not a leader stat anywhere, so HOM1005 stays quiet.
    // (A general unknown-parameter check would be a different rule.)
    let diags = walk(&focus_with(
        "add_naval_commander_role",
        "\t\t\t\tcharacter = GER_admiral\n\t\t\t\tplaning_skill = 2",
    ));
    assert!(wrong_skill(&diags).is_empty(), "{:?}", diags);
}

#[test]
fn non_skill_keys_are_left_alone() {
    // `gfx` is not in the create_* table but vanilla writes it; a rule that
    // flagged every undeclared key would have to be measured against the whole
    // corpus first. This one only owns the skill vocabulary.
    let diags = walk(&focus_with(
        "create_corps_commander",
        "\t\t\t\tname = \"Test General\"\n\t\t\t\tgfx = \"GFX_portrait_test\"\n\t\t\t\tskill = 3",
    ));
    assert!(wrong_skill(&diags).is_empty(), "{:?}", diags);
}

#[test]
fn a_character_definition_role_block_is_covered_too() {
    // `common/characters` writes the same stat block as a definition, not as an
    // effect: `navy_leader = { ... }` inside the character.
    let ctx = TestCtx::new();
    let uri = "/mod/common/characters/zz_test.txt";
    let initial = initial_scope_for_uri(uri);
    let diags = ctx.walk(
        "characters = {\n\tZZ_admiral = {\n\t\tname = \"ZZ Admiral\"\n\t\tnavy_leader = {\n\t\t\tskill = 4\n\t\t\tplanning_skill = 3\n\t\t}\n\t}\n}\n",
        uri,
        initial,
        vec![Box::new(LeaderRoleSkillRule)],
        vec![],
    );
    let found = wrong_skill(&diags);
    assert_eq!(found.len(), 1, "{:?}", diags);
    assert!(
        found[0].message.contains("add_naval_commander_role")
            || found[0].message.contains("navy_leader"),
        "{:?}",
        found[0]
    );
}

#[test]
fn a_character_definition_with_the_right_stats_is_quiet() {
    let ctx = TestCtx::new();
    let uri = "/mod/common/characters/zz_test.txt";
    let initial = initial_scope_for_uri(uri);
    let diags = ctx.walk(
        "characters = {\n\tZZ_admiral = {\n\t\tname = \"ZZ Admiral\"\n\t\tnavy_leader = {\n\t\t\tskill = 4\n\t\t\tmaneuvering_skill = 3\n\t\t\tcoordination_skill = 4\n\t\t\ttraits = { spotter }\n\t\t\tvisible = { has_completed_focus = ZZ_test }\n\t\t}\n\t}\n}\n",
        uri,
        initial,
        vec![Box::new(LeaderRoleSkillRule)],
        vec![],
    );
    assert!(wrong_skill(&diags).is_empty(), "{:?}", diags);
}

#[test]
fn key_case_does_not_matter() {
    let diags = walk(&focus_with(
        "add_naval_commander_role",
        "\t\t\t\tCharacter = GER_admiral\n\t\t\t\tPLANNING_SKILL = 2",
    ));
    assert_eq!(wrong_skill(&diags).len(), 1, "{:?}", diags);
}
