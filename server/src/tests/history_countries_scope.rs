//! `history/countries/*.txt` is evaluated in Country scope, so the top-level
//! effects (`capital`, `oob`, `set_technology`, `add_ideas`, `set_politics`,
//! …) do not flag as `HOM004`.
//!
//! Regression: before the branch was added, these files started in Global
//! scope and every top-level country effect reported "is not valid in Global
//! scope".

use crate::rules::v2_scope::V2ScopeRule;
use crate::scope::scope::{Scope, initial_scope_for_uri};
use crate::test_support::TestCtx;
use tower_lsp_server::ls_types::Diagnostic;

fn mismatch_keys(diags: &[Diagnostic]) -> Vec<String> {
    diags
        .iter()
        .filter(|d| d.message.contains("is not valid in"))
        .map(|d| d.message.split('\'').nth(1).unwrap_or("").to_string())
        .collect()
}

#[test]
fn history_countries_top_level_is_country_scope() {
    let input = r#"
capital = 1
set_country_flag = foo
oob = "TEST"
set_technology = { infantry_weapons = 1 }
set_politics = { ruling_party = democratic }
set_popularities = { democratic = 100 }
add_ideas = some_idea
set_research_slots = 3
set_stability = 0.5
set_war_support = 0.5
recruit_character = SOME_CHAR
"#;
    let uri = "/mod/history/countries/GER - Germany.txt";
    let initial = initial_scope_for_uri(uri);
    assert_eq!(initial, Scope::Country);

    let ctx = TestCtx::new().with_scope_validation(true);
    let diags = ctx.walk(input, uri, initial, vec![Box::new(V2ScopeRule)], vec![]);
    let keys = mismatch_keys(&diags);
    assert!(
        keys.is_empty(),
        "history/countries top-level effects must resolve as Country, got: {keys:?}"
    );
}
