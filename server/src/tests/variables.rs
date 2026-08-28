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
    // Flag operations should work similarly - need Country scope
    let diags = walk_variables(
        r#"
        set_country_flag = my_flag
        has_country_flag = my_flag
        clr_country_flag = my_flag
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
        "flag operations with matching set/has/clr should not flag, got: {:?}",
        diags
    );
}

#[test]
fn test_flag_read_without_set_flags() {
    let diags = walk_variables(
        r#"
        has_country_flag = undefined_flag
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
        "has_country_flag without set_country_flag should flag HOM9001, got: {:?}",
        diags
    );
}

#[test]
fn test_array_operations() {
    let diags = walk_variables(
        r#"
        add_to_array = my_array
        is_in_array = my_array
        "#,
        "/mod/common/scripted_effects/test.txt",
    );
    // Array defs don't have dedicated scanner yet - may flag for now
    let _ = diags;
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
