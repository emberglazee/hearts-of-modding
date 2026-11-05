//! Strategic region files contain weather property blocks (`weather = { period
//! = { temperature = { ... } ... } }`) whose keys are not triggers or effects.
//! They must not be checked against the enclosing Strategic Region scope.

use crate::rules::v2_scope::V2ScopeRule;
use crate::scope::scope::initial_scope_for_uri;
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
fn strategic_region_weather_keys_do_not_flag() {
    let input = r#"
strategic_region = {
    id = 1
    provinces = { 1 2 }
    weather = {
        period = {
            between = { 0.0 30.0 }
            temperature = { -7.0 3.0 }
            no_phenomenon = 0.500
            rain_light = 0.600
            rain_heavy = 0.700
            snow = 0.000
            blizzard = 0.000
            arctic_water = 0.000
            mud = 1.000
            sandstorm = 0.000
            min_snow_level = 0.000
        }
    }
}
"#;
    let uri = "/mod/map/strategicregions/1-Test.txt";
    let initial = initial_scope_for_uri(uri);
    let ctx = TestCtx::new().with_scope_validation(true);
    let diags = ctx.walk(input, uri, initial, vec![Box::new(V2ScopeRule)], vec![]);
    let keys = mismatch_keys(&diags);
    assert!(
        keys.is_empty(),
        "strategic region weather keys must not flag as HOM004, got: {keys:?}"
    );
}
