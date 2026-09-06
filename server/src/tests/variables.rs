use crate::rules::v2_scope::V2ScopeRule;
use crate::rules::variables::VariableRuleState;
use crate::scope::scope::initial_scope_for_uri;
use crate::test_support::TestCtx;

fn walk_variables(input: &str, uri: &str) -> Vec<tower_lsp_server::ls_types::Diagnostic> {
    let ctx = TestCtx::new().with_scope_validation(true);
    let initial = initial_scope_for_uri(uri);
    ctx.walk(
        input,
        uri,
        initial,
        vec![
            Box::new(V2ScopeRule),
            Box::new(VariableRuleState::new(
                &Default::default(),
                &Default::default(),
                &Default::default(),
            )),
        ],
        vec![],
    )
}

#[test]
fn test_set_variable_then_check_variable_valid() {
    // Basic variable definition followed by read should be valid
    let diags = walk_variables(
        r#"
        set_variable = my_var
        check_variable = my_var
        "#,
        "/mod/common/scripted_effects/test.txt",
    );
    // No HOM9001 for my_var since it was defined
    assert!(
        !diags.iter().any(|d| d
            .code
            .as_ref()
            .map(|c| format!("{:?}", c))
            .unwrap_or_default()
            .contains("HOM9001")),
        "check_variable after set_variable should not flag, got: {:?}",
        diags
    );
}

#[test]
fn test_check_variable_without_set_variable_flags() {
    // Reading a variable that was never defined should flag
    let diags = walk_variables(
        r#"
        check_variable = undefined_var
        "#,
        "/mod/common/scripted_effects/test.txt",
    );
    assert!(
        diags.iter().any(|d| d
            .code
            .as_ref()
            .map(|c| format!("{:?}", c))
            .unwrap_or_default()
            .contains("HOM9001")),
        "check_variable without prior set_variable should flag HOM9001, got: {:?}",
        diags
    );
}

#[test]
fn test_has_variable_without_set_variable_flags() {
    let diags = walk_variables(
        r#"
        has_variable = undefined_var
        "#,
        "/mod/common/scripted_effects/test.txt",
    );
    assert!(
        diags.iter().any(|d| d
            .code
            .as_ref()
            .map(|c| format!("{:?}", c))
            .unwrap_or_default()
            .contains("HOM9001")),
        "has_variable without prior set_variable should flag HOM9001, got: {:?}",
        diags
    );
}

#[test]
fn test_add_to_variable_without_set_variable_flags() {
    let diags = walk_variables(
        r#"
        add_to_variable = undefined_var
        "#,
        "/mod/common/scripted_effects/test.txt",
    );
    assert!(
        diags.iter().any(|d| d
            .code
            .as_ref()
            .map(|c| format!("{:?}", c))
            .unwrap_or_default()
            .contains("HOM9001")),
        "add_to_variable without prior set_variable should flag HOM9001, got: {:?}",
        diags
    );
}

#[test]
fn test_temp_variable_chain_local() {
    // Temp variables should only be valid within same chain
    // set_temp_variable then add_to_temp_variable in same block = valid
    let diags = walk_variables(
        r#"
        set_temp_variable = my_temp
        add_to_temp_variable = my_temp
        "#,
        "/mod/common/scripted_effects/test.txt",
    );
    // TODO: Full chain tracking needs scope stack awareness; for MVP this may flag
    // We'll verify it doesn't crash
    let _ = diags;
}

#[test]
fn test_global_variable_prefix_always_valid() {
    // global.vars should be accessible everywhere without prior definition
    let diags = walk_variables(
        r#"
        check_variable = global.my_global
        add_to_variable = global.other_global
        "#,
        "/mod/common/scripted_effects/test.txt",
    );
    // Global variables should not flag as undefined
    let hom9001: Vec<_> = diags
        .iter()
        .filter(|d| {
            d.code
                .as_ref()
                .map(|c| format!("{:?}", c))
                .unwrap_or_default()
                .contains("HOM9001")
        })
        .collect();
    // Filter out only the global ones - we expect no HOM9001 for global.*
    assert!(
        hom9001.is_empty(),
        "global.* variables should not flag HOM9001, got: {:?}",
        diags
    );
}

#[test]
fn test_flag_operations() {
    // Flags implicitly exist — no definition-tracking for them at all.
    // set/has/clr in any order, defined or not, produces no diagnostics.
    let diags = walk_variables(
        r#"
        set_country_flag = my_flag
        has_country_flag = my_flag
        clr_country_flag = my_flag
        set_global_flag = g_flag
        "#,
        "/mod/common/decisions/test.txt",
    );
    let hom9001: Vec<_> = diags
        .iter()
        .filter(|d| {
            d.code
                .as_ref()
                .map(|c| format!("{:?}", c))
                .unwrap_or_default()
                .contains("HOM9001")
        })
        .collect();
    assert!(
        hom9001.is_empty(),
        "flag operations must never flag HOM9001, got: {:?}",
        diags
    );
}

#[test]
fn test_flag_read_without_set_no_diagnostic() {
    // Engine semantics: unset flags read as false. The one-shot-guard idiom
    // (`NOT = { has_global_flag = x }` in a trigger, `set_global_flag = x` in
    // the same event's immediate) is valid and core. Reading a flag that is
    // never set anywhere is legal — it just stays false.
    let diags = walk_variables(
        r#"
        has_country_flag = undefined_flag
        has_global_flag = never_set_flag
        clr_state_flag = some_flag
        "#,
        "/mod/common/decisions/test.txt",
    );
    assert!(
        !diags.iter().any(|d| d
            .code
            .as_ref()
            .map(|c| format!("{:?}", c))
            .unwrap_or_default()
            .contains("HOM9001")),
        "flag reads must never flag HOM9001 (flags implicitly exist), got: {:?}",
        diags
    );
}

#[test]
fn test_array_operations() {
    // add_to_array defines the array; is_in_array after should not flag
    let diags = walk_variables(
        r#"
        add_to_array = { array = my_array value = 1 }
        is_in_array = { array = my_array value = 1 }
        "#,
        "/mod/common/scripted_effects/test.txt",
    );
    assert!(
        !diags.iter().any(|d| d
            .code
            .as_ref()
            .map(|c| format!("{:?}", c))
            .unwrap_or_default()
            .contains("HOM9001")),
        "is_in_array after add_to_array should not flag, got: {:?}",
        diags
    );
}

#[test]
fn test_array_shorthand_def_then_read() {
    // add_to_array shorthand creates array, subsequent clear/remove/for_each should be valid
    let diags = walk_variables(
        r#"
        add_to_array = { array = my_arr value = 1 }
        clear_array = my_arr
        remove_from_array = { array = my_arr index = 0 }
        for_each_scope_loop = { array = my_arr }
        "#,
        "/mod/common/scripted_effects/test.txt",
    );
    assert!(
        !diags.iter().any(|d| d
            .code
            .as_ref()
            .map(|c| format!("{:?}", c))
            .unwrap_or_default()
            .contains("HOM9001")),
        "array mutations after def should not flag, got: {:?}",
        diags
    );
}

#[test]
fn test_array_read_without_def_flags() {
    let diags = walk_variables(
        r#"
        is_in_array = { array = undefined_arr value = 1 }
        for_each_loop = { array = undefined_arr }
        any_of_scopes = { array = undefined_arr }
        "#,
        "/mod/common/scripted_effects/test.txt",
    );
    let count = diags
        .iter()
        .filter(|d| {
            d.code
                .as_ref()
                .map(|c| format!("{:?}", c))
                .unwrap_or_default()
                .contains("HOM9001")
        })
        .count();
    assert_eq!(
        count, 3,
        "all three array reads without prior add should flag, got: {:?}",
        diags
    );
}

#[test]
fn test_array_temp_chain_local() {
    // add_to_temp_array defines temp array chain-local; clear_temp_array and is_in_array in same chain should be valid
    let diags = walk_variables(
        r#"
        add_to_temp_array = { array = temp_arr value = 1 }
        clear_temp_array = temp_arr
        is_in_array = { array = temp_arr value = 1 }
        "#,
        "/mod/common/scripted_effects/test.txt",
    );
    assert!(
        !diags.iter().any(|d| d
            .code
            .as_ref()
            .map(|c| format!("{:?}", c))
            .unwrap_or_default()
            .contains("HOM9001")),
        "temp array ops after add_to_temp should not flag, got: {:?}",
        diags
    );
}

#[test]
fn test_array_block_form_and_shorthand() {
    // Block form with array = X and shorthand with single key
    let diags = walk_variables(
        r#"
        add_to_array = { array = block_arr value = 1 }
        is_in_array = { block_arr = 1 }
        add_to_array = { shorthand_arr = 42 }
        is_in_array = { array = shorthand_arr value = 1 }
        "#,
        "/mod/common/scripted_effects/test.txt",
    );
    assert!(
        !diags.iter().any(|d| d
            .code
            .as_ref()
            .map(|c| format!("{:?}", c))
            .unwrap_or_default()
            .contains("HOM9001")),
        "array block/shorthand forms should be tracked, got: {:?}",
        diags
    );
}

#[test]
fn test_block_form_variable_definition() {
    // set_variable = { var = my_var }
    let diags = walk_variables(
        r#"
        set_variable = { var = my_block_var }
        check_variable = my_block_var
        "#,
        "/mod/common/scripted_effects/test.txt",
    );
    let hom9001: Vec<_> = diags
        .iter()
        .filter(|d| {
            d.code
                .as_ref()
                .map(|c| format!("{:?}", c))
                .unwrap_or_default()
                .contains("HOM9001")
        })
        .collect();
    assert!(
        hom9001.is_empty(),
        "block form set_variable with var= should work, got: {:?}",
        diags
    );
}

#[test]
fn test_shorthand_variable_definition() {
    // set_variable = { my_var = 5 }
    let diags = walk_variables(
        r#"
        set_variable = { my_shorthand = 5 }
        check_variable = my_shorthand
        "#,
        "/mod/common/scripted_effects/test.txt",
    );
    let hom9001: Vec<_> = diags
        .iter()
        .filter(|d| {
            d.code
                .as_ref()
                .map(|c| format!("{:?}", c))
                .unwrap_or_default()
                .contains("HOM9001")
        })
        .collect();
    assert!(
        hom9001.is_empty(),
        "shorthand form set_variable should work, got: {:?}",
        diags
    );
}

#[test]
fn test_variable_name_keyword_in_block() {
    // set_variable = { name = my_named_var }
    let diags = walk_variables(
        r#"
        set_variable = { name = my_named_var }
        check_variable = my_named_var
        "#,
        "/mod/common/scripted_effects/test.txt",
    );
    let hom9001: Vec<_> = diags
        .iter()
        .filter(|d| {
            d.code
                .as_ref()
                .map(|c| format!("{:?}", c))
                .unwrap_or_default()
                .contains("HOM9001")
        })
        .collect();
    assert!(
        hom9001.is_empty(),
        "block form with name= should work, got: {:?}",
        diags
    );
}

#[test]
fn test_variable_scope_country_vs_state() {
    // Variables defined at Country scope should be accessible at State scope
    // This tests scope accessibility logic
    let diags = walk_variables(
        r#"
        state = {
            set_variable = my_state_var
        }
        "#,
        "/mod/common/decisions/test.txt",
    );
    // Just verify it doesn't crash; scope accessibility needs full scope stack context
    let _ = diags;
}

#[test]
fn test_array_cross_file_via_workspace() {
    // Seed a file with add_to_array, then is_in_array in another file should resolve via workspace
    let ctx = crate::test_support::TestCtx::new()
        .with_file(
            "/mod/common/decisions/seed.txt",
            "add_to_array = { array = TIR_global_campaign_holders value = PREV }",
        )
        .with_scope_validation(true);
    let initial = crate::scope::scope::initial_scope_for_uri("/mod/common/decisions/other.txt");
    let diags = ctx.walk(
        r#"
        if = {
            limit = { NOT = { is_in_array = { array = TIR_global_campaign_holders value = PREV } } }
            add_to_array = { array = TIR_global_campaign_holders value = PREV }
        }
        clear_array = TIR_global_campaign_holders
        for_each_scope_loop = { array = TIR_global_campaign_holders }
        "#,
        "/mod/common/decisions/other.txt",
        initial,
        vec![
            Box::new(crate::rules::v2_scope::V2ScopeRule),
            Box::new(crate::rules::variables::VariableRuleState::new(
                &ctx.scanner_data().variables,
                &ctx.scanner_data().arrays,
                &ctx.scanner_data().event_targets,
            )),
        ],
        vec![],
    );
    let hom9001: Vec<_> = diags
        .iter()
        .filter(|d| {
            d.code
                .as_ref()
                .map(|c| format!("{:?}", c))
                .unwrap_or_default()
                .contains("HOM9001")
        })
        .collect();
    assert!(
        hom9001.is_empty(),
        "TIR guard pattern with workspace-populated array should not flag, got: {:?}",
        diags
    );
}

#[test]
fn test_array_typo_still_flags_even_with_workspace() {
    // Same workspace but typo name should still flag
    let ctx = crate::test_support::TestCtx::new()
        .with_file(
            "/mod/common/decisions/seed.txt",
            "add_to_array = { array = TIR_global_campaign_holders value = PREV }",
        )
        .with_scope_validation(true);
    let initial = crate::scope::scope::initial_scope_for_uri("/mod/common/decisions/other.txt");
    let diags = ctx.walk(
        r#"
        is_in_array = { array = TIR_global_campiagn_holders value = PREV }
        "#,
        "/mod/common/decisions/other.txt",
        initial,
        vec![
            Box::new(crate::rules::v2_scope::V2ScopeRule),
            Box::new(crate::rules::variables::VariableRuleState::new(
                &ctx.scanner_data().variables,
                &ctx.scanner_data().arrays,
                &ctx.scanner_data().event_targets,
            )),
        ],
        vec![],
    );
    assert!(
        diags.iter().any(|d| d
            .code
            .as_ref()
            .map(|c| format!("{:?}", c))
            .unwrap_or_default()
            .contains("HOM9001")),
        "typo array name should flag HOM9001, got: {:?}",
        diags
    );
    // Message should mention arrays are empty by default, not variables read as 0
    let msg = diags
        .iter()
        .find(|d| {
            d.code
                .as_ref()
                .map(|c| format!("{:?}", c))
                .unwrap_or_default()
                .contains("HOM9001")
        })
        .unwrap()
        .message
        .clone();
    assert!(
        msg.contains("empty by default"),
        "array diagnostic should mention empty by default, got: {}",
        msg
    );
}

#[test]
fn test_array_any_of_scopes_and_find() {
    let ctx = crate::test_support::TestCtx::new()
        .with_file(
            "/mod/common/decisions/seed.txt",
            "add_to_array = { array = my_arr value = 1 }",
        )
        .with_scope_validation(true);
    let initial = crate::scope::scope::initial_scope_for_uri("/mod/common/decisions/other.txt");
    let diags = ctx.walk(
        r#"
        any_of_scopes = { array = my_arr }
        find_highest_in_array = { array = my_arr value = temp_val }
        is_in_array = { my_arr = 1 }
        "#,
        "/mod/common/decisions/other.txt",
        initial,
        vec![
            Box::new(crate::rules::v2_scope::V2ScopeRule),
            Box::new(crate::rules::variables::VariableRuleState::new(
                &ctx.scanner_data().variables,
                &ctx.scanner_data().arrays,
                &ctx.scanner_data().event_targets,
            )),
        ],
        vec![],
    );
    assert!(
        diags
            .iter()
            .filter(|d| {
                d.code
                    .as_ref()
                    .map(|c| format!("{:?}", c))
                    .unwrap_or_default()
                    .contains("HOM9001")
            })
            .count()
            == 0,
        "array reads via any_of_scopes/find_highest/shorthand should resolve, got: {:?}",
        diags
    );
}

#[test]
fn test_faction_members_builtin_not_flagged() {
    let diags = walk_variables(
        r#"
        has_large_ally_not_pick_closed_economy = {
            any_of_scopes = {
                array = faction_members
                NOT = { tag = PREV }
                num_of_military_factories > 30
            }
        }
        "#,
        "/mod/common/scripted_triggers/00_scripted_triggers.txt",
    );
    let hom9001: Vec<_> = diags
        .iter()
        .filter(|d| format!("{:?}", d.code).contains("HOM9001"))
        .collect();
    assert!(
        hom9001.is_empty(),
        "faction_members builtin should not flag HOM9001, got: {:?}",
        diags
    );
}
#[test]
fn test_faction_members_scoped_form() {
    let diags = walk_variables(
        r#"
        any_of_scopes = { array = ROOT.faction_members value = 1 }
        is_in_array = { array = THIS.allies value = 1 }
        for_each_scope_loop = { array = neighbors }
        for_each_loop = { array = owned_states }
        "#,
        "/mod/common/scripted_effects/test.txt",
    );
    let hom9001: Vec<_> = diags
        .iter()
        .filter(|d| format!("{:?}", d.code).contains("HOM9001"))
        .collect();
    assert!(
        hom9001.is_empty(),
        "builtin arrays with scope prefix should not flag, got: {:?}",
        diags
    );
}
#[test]
fn test_typo_still_flags() {
    let diags = walk_variables(
        r#"
        is_in_array = { array = faction_memebers value = 1 }
        "#,
        "/mod/common/scripted_effects/test.txt",
    );
    assert!(
        diags
            .iter()
            .any(|d| format!("{:?}", d.code).contains("HOM9001")),
        "typo should still flag"
    );
}

// ── `var:` scope blocks ──
//
// `var:UNO_target = { ... }` runs in whatever scope the variable holds.
// The scanner infers Country for TAG-anchored values (`HAB.id`); anything
// ambiguous stays Unknown so HOM004 keeps skipping (false negatives over
// false positives). `compliance` is a State-only trigger with no Country
// exception, so it flags HOM004 if and only if the block resolved Country.

#[test]
fn test_var_scope_block_resolves_country_from_tag_valued_definition() {
    // Real-world shape: Hearts-Of-Minecraft BOH_spf_decisions.txt
    // (`set_variable = { UNO_target = HAB.id }` + `var:UNO_target` blocks).
    let ctx = TestCtx::new().with_scope_validation(true).with_file(
        "/mod/common/decisions/BOH_spf_decisions.txt",
        "SPF_Diana_UN = {\n\tSPF_focus_on_appon = {\n\t\tcomplete_effect = {\n\t\t\tset_variable = { UNO_target = HAB.id }\n\t\t}\n\t}\n}\n",
    );
    let uri = "/mod/common/decisions/BOH_spf_decisions.txt";
    let diags = ctx.walk(
        "SPF_Invite_to_faction = {\n\tavailable = {\n\t\tvar:UNO_target = {\n\t\t\tcompliance > 50\n\t\t}\n\t}\n}\n",
        uri,
        initial_scope_for_uri(uri),
        vec![Box::new(V2ScopeRule)],
        vec![],
    );
    assert!(
        diags
            .iter()
            .any(|d| format!("{:?}", d.code).contains("HOM004")),
        "state-only 'compliance' inside var:UNO_target (Country) should flag HOM004, got: {:?}",
        diags
    );
}

#[test]
fn test_var_scope_block_unknown_variable_skips_scope_validation() {
    let ctx = TestCtx::new().with_scope_validation(true);
    let uri = "/mod/common/decisions/test.txt";
    let diags = ctx.walk(
        "SPF_Invite_to_faction = {\n\tavailable = {\n\t\tvar:UNO_missing = {\n\t\t\tcompliance > 50\n\t\t}\n\t}\n}\n",
        uri,
        initial_scope_for_uri(uri),
        vec![Box::new(V2ScopeRule)],
        vec![],
    );
    assert!(
        !diags
            .iter()
            .any(|d| format!("{:?}", d.code).contains("HOM004")),
        "unknown var must stay Unknown (skip HOM004), got: {:?}",
        diags
    );
}

#[test]
fn test_var_scope_block_numeric_variable_skips_scope_validation() {
    // A plain counter is not a scope — `var:my_count` must not resolve.
    let ctx = TestCtx::new().with_scope_validation(true).with_file(
        "/mod/common/decisions/test.txt",
        "SPF_counter = {\n\tcomplete_effect = {\n\t\tset_variable = { my_count = 5 }\n\t}\n}\n",
    );
    let uri = "/mod/common/decisions/test.txt";
    let diags = ctx.walk(
        "SPF_use = {\n\tavailable = {\n\t\tvar:my_count = {\n\t\t\tcompliance > 50\n\t\t}\n\t}\n}\n",
        uri,
        initial_scope_for_uri(uri),
        vec![Box::new(V2ScopeRule)],
        vec![],
    );
    assert!(
        !diags
            .iter()
            .any(|d| format!("{:?}", d.code).contains("HOM004")),
        "numeric var must stay Unknown (skip HOM004), got: {:?}",
        diags
    );
}

#[test]
fn test_variable_scanner_infers_country_only_for_tag_values() {
    use crate::scanner::variable_scanner;
    use std::collections::HashMap;
    let content = "SPF = {\n\tset_variable = { UNO_target = HAB.id }\n\tset_variable = { my_count = 5 }\n\tset_variable = { var = long_form value = GER }\n\tset_variable = { var = ctx_var value = ROOT }\n}\n";
    let (script, _) = crate::parser::parser::parse_script(content);
    let mut vars: HashMap<String, Vec<variable_scanner::Variable>> = HashMap::new();
    variable_scanner::scan_entries(
        &script.entries,
        &script.source,
        "test.txt",
        &mut vars,
        &mut HashMap::new(),
        &mut HashMap::new(),
    );
    use crate::scope::scope::Scope;
    assert_eq!(vars["UNO_target"][0].scope, Scope::Country);
    assert_eq!(vars["my_count"][0].scope, Scope::Unknown);
    assert_eq!(vars["long_form"][0].scope, Scope::Country);
    assert_eq!(vars["ctx_var"][0].scope, Scope::Unknown);
}

#[test]
fn test_resolve_var_scope_block_from_tracked_variables() {
    use crate::data::interner::InternedStr;
    use crate::parser::ast;
    use crate::scanner::variable_scanner::Variable;
    use crate::scope::scope::{Scope, ScopeCtx, ScopeStack};
    use dashmap::DashMap;
    let vars: DashMap<InternedStr, Vec<Variable>> = DashMap::new();
    vars.insert(
        InternedStr::from("UNO_target"),
        vec![Variable {
            name: "UNO_target".to_string(),
            path: InternedStr::from("test.txt"),
            range: ast::Range {
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 5,
            },
            scope: Scope::Country,
        }],
    );
    let sctx = ScopeCtx {
        uri: "/mod/common/decisions/test.txt",
        event_targets: None,
        characters: None,
        achievements: None,
        variables: Some(&vars),
        in_random_list: false,
        state_targeted: false,
    };
    let stack = ScopeStack::new(Scope::Country);
    let (s, _) = stack.resolve_entry_scope("var:UNO_target", &sctx);
    assert_eq!(s, Scope::Country);
    // Case-insensitive prefix, temp_var: alias, and unknown names.
    let (s_upper, _) = stack.resolve_entry_scope("VAR:UNO_target", &sctx);
    assert_eq!(s_upper, Scope::Country);
    let (s_temp, _) = stack.resolve_entry_scope("temp_var:UNO_target", &sctx);
    assert_eq!(s_temp, Scope::Country);
    let (s_missing, _) = stack.resolve_entry_scope("var:UNO_missing", &sctx);
    assert_eq!(s_missing, Scope::Unknown);
    let (s_empty, _) = stack.resolve_entry_scope("var:", &sctx);
    assert_eq!(s_empty, Scope::Unknown);
}
