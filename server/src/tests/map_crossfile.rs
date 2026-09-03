//! Tier-0 map cross-file integration test (HOM2002–HOM2009).
//!
//! Unlike the rule unit tests (fully synthetic seeds), this loads a tiny
//! vendored `definition.csv` + buildings file from disk through the REAL
//! `scan_province_files` / `scan_building_files` startup path, then seeds
//! states/regions through the real incremental path (`with_file` — the same
//! fn production calls on did_save). Fixtures live in-repo under
//! `src/tests/fixtures/` and are addressed via `CARGO_MANIFEST_DIR`, so this
//! runs identically on every CI platform with no game install required.

use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

use crate::scope::scope::Scope;

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/tests/fixtures/map_crossfile")
        .join(name)
}

/// Context seeded from the on-disk fixtures via the real startup scanners
/// (CSV line parser + building script parser included — a parse regression
/// in either file shape fails here, not just in production).
fn ctx_from_fixtures() -> crate::test_support::TestCtx {
    let no_filter = |_: &std::path::Path| false;
    let provinces = crate::scanner::province_scanner::scan_province_files(
        &[fixture("definition.csv")],
        &no_filter,
    );
    assert_eq!(
        provinces.len(),
        5,
        "fixture csv must yield 5 provinces, got {}",
        provinces.len()
    );
    let buildings = crate::scanner::building_scanner::scan_building_files(
        &[fixture("buildings.txt")],
        &no_filter,
    );
    assert!(
        buildings.contains_key("dockyard") && buildings.contains_key("arms_factory"),
        "fixture buildings must parse: {:?}",
        buildings.keys().collect::<Vec<_>>()
    );
    // The vanilla `only_costal` misspelling must light up `coastal_only`.
    assert!(
        buildings["dockyard"].coastal_only,
        "only_costal = yes must set coastal_only"
    );
    assert!(
        !buildings["arms_factory"].coastal_only,
        "plain building must not be coastal_only"
    );

    let mut ctx = crate::test_support::TestCtx::new();
    for (id, p) in &provinces {
        ctx.data().provinces.insert(*id, p.clone());
    }
    for (name, b) in &buildings {
        ctx.data().buildings.insert(
            crate::data::interner::InternedStr::from(name.as_str()),
            crate::data::layered_value::LayeredValue::new(b.clone()),
        );
    }
    ctx
}

fn state_diags(ctx: &crate::test_support::TestCtx, input: &str, uri: &str) -> Vec<Diagnostic> {
    ctx.walk(
        input,
        uri,
        Scope::Global,
        vec![],
        vec![crate::rules::state_definitions::StateDefinitionRule::map_visitor(uri)],
    )
}

fn region_diags(ctx: &crate::test_support::TestCtx, input: &str, uri: &str) -> Vec<Diagnostic> {
    ctx.walk(
        input,
        uri,
        Scope::Global,
        vec![],
        vec![crate::rules::strategic_regions::StrategicRegionRule::visitor(uri)],
    )
}

fn has_code(diags: &[Diagnostic], code: &str) -> bool {
    diags.iter().any(|d| {
        d.code == Some(NumberOrString::String(code.to_string()))
            && d.severity == Some(DiagnosticSeverity::WARNING)
    })
}

const STATE_URI: &str = "/mod/history/states/10-Clean.txt";
const REGION_URI: &str = "/mod/map/strategicregions/01_test.txt";

/// Vanilla-shaped content over fixture provinces: coastal members, VP on a
/// member, buildings on members, coastal-only building on a coast.
const CLEAN_STATE: &str = "state = { id = 10 provinces = { 1 2 } history = { \
    victory_points = { 1 5 } \
    buildings = { 1 = { arms_factory = 1 } 2 = { dockyard = 1 } } } }";

#[test]
fn clean_state_and_region_are_silent() {
    let ctx = ctx_from_fixtures().with_file(
        "/mod/map/strategicregions/01_test.txt",
        "strategic_region = { id = 1 provinces = { 1 } }",
    );
    let diags = state_diags(&ctx, CLEAN_STATE, STATE_URI);
    assert!(diags.is_empty(), "clean state must be silent: {:?}", diags);
    let diags = region_diags(
        &ctx,
        "strategic_region = { id = 1 provinces = { 1 } }",
        REGION_URI,
    );
    assert!(diags.is_empty(), "clean region must be silent: {:?}", diags);
}

/// One bad state exercising every state-side Tier-0 code at once:
/// sea member (4, lake 5 exempt), double-claimed member (2, also in state
/// 21), duplicate VP, arms_factory on non-member province 1 (HOM2005),
/// coastal-only dockyard on inland *member* 3 (HOM2007), unknown 9999.
/// (A foreign placement reports only HOM2005 by design — one problem, one
/// diag — so HOM2007 needs a member province.)
const BAD_STATE: &str = "state = { id = 20 provinces = { 2 3 4 5 9999 } history = { \
    victory_points = { 2 5 } victory_points = { 2 3 } \
    buildings = { 1 = { arms_factory = 1 } 3 = { dockyard = 1 } } } }";

#[test]
fn each_state_side_code_fires() {
    let ctx = ctx_from_fixtures().with_file(
        "/mod/history/states/21-Other.txt",
        "state = { id = 21 provinces = { 2 } }",
    );
    let diags = state_diags(&ctx, BAD_STATE, "/mod/history/states/20-Bad.txt");
    for code in [
        "HOM2002", "HOM2003", "HOM2004", "HOM2005", "HOM2007", "HOM2009",
    ] {
        assert!(has_code(&diags, code), "{code} must fire: {:?}", diags);
    }
    // Lake 5 is a legal member — must not fire HOM2002.
    assert!(
        !diags.iter().any(|d| d.message.contains('5')
            && d.code == Some(NumberOrString::String("HOM2002".to_string()))),
        "lake 5 must stay silent: {:?}",
        diags
    );

    let diags = state_diags(
        &ctx,
        "state = { id = 22 provinces = { } }",
        "/mod/history/states/22-Empty.txt",
    );
    assert!(has_code(&diags, "HOM2008"), "empty state: {:?}", diags);
}

#[test]
fn region_double_claim_fires() {
    let ctx = ctx_from_fixtures().with_file(
        "/mod/map/strategicregions/02_other.txt",
        "strategic_region = { id = 2 provinces = { 1 } }",
    );
    let diags = region_diags(
        &ctx,
        "strategic_region = { id = 1 provinces = { 1 2 } }",
        REGION_URI,
    );
    let hits: Vec<_> = diags
        .iter()
        .filter(|d| d.code == Some(NumberOrString::String("HOM2006".to_string())))
        .collect();
    assert_eq!(hits.len(), 1, "only shared prov 1: {:?}", diags);
}
