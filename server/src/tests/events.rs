use crate::parser::parser;
use crate::rules::events::EventValidationRule;
use crate::rules::visitor::{AstVisitor, walk_script};
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::Scope;
use dashmap::DashMap;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

// ---------------------------------------------------------------------------
// Test runner
// ---------------------------------------------------------------------------

/// Run the EventValidationRule (with visitor) over a script and return all diagnostics.
fn run_event_visitor(
    input: &str,
    uri: &str,
    declared_namespaces: &[(/*name*/ &str, /*filepath*/ &str)],
) -> Vec<Diagnostic> {
    let (script, _) = parser::parse_script(input);

    let loc = DashMap::new();
    let st = DashMap::new();
    let se = DashMap::new();
    let ideologies = DashMap::new();
    let sub_ideologies = DashMap::new();
    let traits = DashMap::new();
    let sprites = DashMap::new();
    let ideas = DashMap::new();
    let provinces = DashMap::new();
    let modifier_mappings = DashMap::new();
    let sound_effects = DashMap::new();
    let country_tags = DashMap::new();
    let buildings = DashMap::new();
    let resources = DashMap::new();
    let state_categories = DashMap::new();
    let continents = DashMap::new();
    let strategic_regions = DashMap::new();
    let terrain_categories = DashMap::new();
    let abilities = DashMap::new();
    let event_namespaces: DashMap<
        crate::data::interner::InternedStr,
        crate::data::layered_value::LayeredValue<
            crate::scanner::event_namespace_scanner::EventNamespace,
        >,
    > = DashMap::new();

    for (name, filepath) in declared_namespaces {
        event_namespaces.insert(
            std::sync::Arc::<str>::from(*name),
            crate::data::layered_value::LayeredValue::new(
                crate::scanner::event_namespace_scanner::EventNamespace {
                    name: name.to_string(),
                    path: std::sync::Arc::<str>::from(*filepath),
                    range: crate::parser::ast::Range {
                        start_line: 0,
                        start_col: 0,
                        end_line: 0,
                        end_col: 0,
                    },
                },
            ),
        );
    }

    let ctx = ValidationContext {
        uri,
        source: &script.source,
        loc: &loc,
        scripted_triggers: &st,
        scripted_effects: &se,
        ideologies: &ideologies,
        sub_ideologies: &sub_ideologies,
        traits: &traits,
        sprites: &sprites,
        ideas: &ideas,
        provinces: &provinces,
        modifier_mappings: &modifier_mappings,
        ignored_loc_regex: &[],
        comments: &[],
        sound_effects: &sound_effects,
        country_tags: &country_tags,
        buildings: &buildings,
        resources: &resources,
        state_categories: &state_categories,
        continents: &continents,
        strategic_regions: &strategic_regions,
        terrain_categories: &terrain_categories,
        abilities: &abilities,
        game_path: None,
        styling_enabled: false,
        workspace_roots: &[],
        unit_types: &DashMap::new(),
        event_namespaces: &event_namespaces,
    };

    let rules: Vec<Box<dyn ValidationRule>> = vec![Box::new(EventValidationRule)];
    let mut visitors: Vec<Box<dyn AstVisitor>> = vec![EventValidationRule::visitor()];
    let mut diags = Vec::new();

    walk_script(
        &script.entries,
        &mut visitors,
        &rules,
        &ctx,
        &mut diags,
        Scope::Country,
        false,
    );

    diags
}

/// Like run_event_visitor but with a configurable game_path.
fn run_event_visitor_with_game_path(
    input: &str,
    uri: &str,
    declared_namespaces: &[(&str, &str)],
    game_path: Option<&str>,
) -> Vec<Diagnostic> {
    let (script, _) = parser::parse_script(input);

    let loc = DashMap::new();
    let st = DashMap::new();
    let se = DashMap::new();
    let ideologies = DashMap::new();
    let sub_ideologies = DashMap::new();
    let traits = DashMap::new();
    let sprites = DashMap::new();
    let ideas = DashMap::new();
    let provinces = DashMap::new();
    let modifier_mappings = DashMap::new();
    let sound_effects = DashMap::new();
    let country_tags = DashMap::new();
    let buildings = DashMap::new();
    let resources = DashMap::new();
    let state_categories = DashMap::new();
    let continents = DashMap::new();
    let strategic_regions = DashMap::new();
    let terrain_categories = DashMap::new();
    let abilities = DashMap::new();
    let event_namespaces: DashMap<
        crate::data::interner::InternedStr,
        crate::data::layered_value::LayeredValue<
            crate::scanner::event_namespace_scanner::EventNamespace,
        >,
    > = DashMap::new();

    for (name, filepath) in declared_namespaces {
        event_namespaces.insert(
            std::sync::Arc::<str>::from(*name),
            crate::data::layered_value::LayeredValue::new(
                crate::scanner::event_namespace_scanner::EventNamespace {
                    name: name.to_string(),
                    path: std::sync::Arc::<str>::from(*filepath),
                    range: crate::parser::ast::Range {
                        start_line: 0,
                        start_col: 0,
                        end_line: 0,
                        end_col: 0,
                    },
                },
            ),
        );
    }

    let ctx = ValidationContext {
        uri,
        source: &script.source,
        loc: &loc,
        scripted_triggers: &st,
        scripted_effects: &se,
        ideologies: &ideologies,
        sub_ideologies: &sub_ideologies,
        traits: &traits,
        sprites: &sprites,
        ideas: &ideas,
        provinces: &provinces,
        modifier_mappings: &modifier_mappings,
        ignored_loc_regex: &[],
        comments: &[],
        sound_effects: &sound_effects,
        country_tags: &country_tags,
        buildings: &buildings,
        resources: &resources,
        state_categories: &state_categories,
        continents: &continents,
        strategic_regions: &strategic_regions,
        terrain_categories: &terrain_categories,
        abilities: &abilities,
        game_path: game_path.map(|s| s.to_string()),
        styling_enabled: false,
        workspace_roots: &[],
        unit_types: &DashMap::new(),
        event_namespaces: &event_namespaces,
    };

    let rules: Vec<Box<dyn ValidationRule>> = vec![Box::new(EventValidationRule)];
    let mut visitors: Vec<Box<dyn AstVisitor>> = vec![EventValidationRule::visitor()];
    let mut diags = Vec::new();

    walk_script(
        &script.entries,
        &mut visitors,
        &rules,
        &ctx,
        &mut diags,
        Scope::Country,
        false,
    );

    diags
}

/// Filter diagnostics to HOM3008 (missing event namespace).
fn namespace_diags(diags: &[Diagnostic]) -> Vec<&Diagnostic> {
    diags
        .iter()
        .filter(|d| matches!(&d.code, Some(NumberOrString::String(c)) if c == "HOM3008"))
        .collect()
}

/// Filter diagnostics to HOM3012 (duplicate event namespace).
fn duplicate_diags(diags: &[Diagnostic]) -> Vec<&Diagnostic> {
    diags
        .iter()
        .filter(|d| matches!(&d.code, Some(NumberOrString::String(c)) if c == "HOM3012"))
        .collect()
}

/// Filter diagnostics to HOM3017 (option missing ai_chance).
fn ai_chance_diags(diags: &[Diagnostic]) -> Vec<&Diagnostic> {
    diags
        .iter()
        .filter(|d| matches!(&d.code, Some(NumberOrString::String(c)) if c == "HOM3017"))
        .collect()
}

// ---------------------------------------------------------------------------
// Same-file ordering tests
// ---------------------------------------------------------------------------

#[test]
fn test_event_declared_namespace_same_file_before() {
    let input = r#"
add_namespace = ns_test
country_event = {
    id = ns_test.1
    hidden = yes
    is_triggered_only = yes
}
"#;
    let diags = run_event_visitor(
        input,
        "file:///events/aaa_test.txt",
        &[("ns_test", "/events/aaa_test.txt")],
    );
    let ns_diags = namespace_diags(&diags);
    assert!(
        ns_diags.is_empty(),
        "Declared namespace before event should produce no HOM3008"
    );
}

#[test]
fn test_event_namespace_declared_later_in_same_file() {
    let input = r#"
country_event = {
    id = late_ns.1
    hidden = yes
    is_triggered_only = yes
}
add_namespace = late_ns
"#;
    let diags = run_event_visitor(
        input,
        "file:///events/test.txt",
        &[("late_ns", "/events/test.txt")],
    );
    let ns_diags = namespace_diags(&diags);
    assert_eq!(
        ns_diags.len(),
        1,
        "Namespace declared later should produce HOM3008"
    );
    assert_eq!(ns_diags[0].severity, Some(DiagnosticSeverity::ERROR));
    assert!(
        ns_diags[0].message.contains("LATER"),
        "Should mention 'LATER'"
    );
}

// ---------------------------------------------------------------------------
// Undeclared / missing namespace tests
// ---------------------------------------------------------------------------

#[test]
fn test_event_missing_namespace_undeclared() {
    let input = r#"
country_event = {
    id = undef_ns.1
    hidden = yes
    is_triggered_only = yes
}
"#;
    let diags = run_event_visitor(input, "file:///events/test.txt", &[]);
    let ns_diags = namespace_diags(&diags);
    assert_eq!(
        ns_diags.len(),
        1,
        "Undeclared namespace should produce HOM3008"
    );
    assert_eq!(ns_diags[0].severity, Some(DiagnosticSeverity::ERROR));
    assert!(ns_diags[0].message.contains("Malformed token"));
}

// ---------------------------------------------------------------------------
// Cross-file ordering tests
// ---------------------------------------------------------------------------

#[test]
fn test_event_namespace_in_file_that_loads_after() {
    let input = r#"
country_event = {
    id = after_ns.1
    hidden = yes
    is_triggered_only = yes
}
"#;
    let diags = run_event_visitor(
        input,
        "file:///events/aaa_events.txt",
        &[("after_ns", "/events/zzz_events.txt")],
    );
    let ns_diags = namespace_diags(&diags);
    assert_eq!(
        ns_diags.len(),
        1,
        "Namespace in file that loads after should produce HOM3008"
    );
    assert!(ns_diags[0].message.contains("loads AFTER this one"));
}

#[test]
fn test_event_namespace_in_file_that_loads_before() {
    let input = r#"
country_event = {
    id = before_ns.1
    hidden = yes
    is_triggered_only = yes
}
"#;
    let diags = run_event_visitor(
        input,
        "file:///events/zzz_events.txt",
        &[("before_ns", "/events/aaa_events.txt")],
    );
    let ns_diags = namespace_diags(&diags);
    assert!(
        ns_diags.is_empty(),
        "Namespace in file that loads before should produce no HOM3008"
    );
}

// ---------------------------------------------------------------------------
// Cross-layer (game path vs workspace) ordering tests
// ---------------------------------------------------------------------------

#[test]
fn test_event_namespace_from_game_path_available_to_workspace() {
    let input = r#"
country_event = {
    id = vanilla_ns.1
    hidden = yes
    is_triggered_only = yes
}
"#;
    let diags = run_event_visitor_with_game_path(
        input,
        "file:///workspace/events/aaa_mod.txt",
        &[(
            "vanilla_ns",
            "C:/game/Hearts of Iron IV/events/zzz_vanilla.txt",
        )],
        Some("C:/game/Hearts of Iron IV"),
    );
    let ns_diags = namespace_diags(&diags);
    assert!(
        ns_diags.is_empty(),
        "Vanilla namespace should be available to mod regardless of filename"
    );
}

#[test]
fn test_event_namespace_from_workspace_not_available_to_vanilla() {
    let input = r#"
country_event = {
    id = mod_ns.1
    hidden = yes
    is_triggered_only = yes
}
"#;
    let diags = run_event_visitor_with_game_path(
        input,
        "file:///C:/game/Hearts%20of%20Iron%20IV/events/zzz_vanilla.txt",
        &[("mod_ns", "C:/workspace/events/aaa_mod.txt")],
        Some("C:/game/Hearts of Iron IV"),
    );
    let ns_diags = namespace_diags(&diags);
    assert_eq!(
        ns_diags.len(),
        1,
        "Mod namespace should NOT be available to vanilla files"
    );
    assert!(
        ns_diags[0].message.contains("base game"),
        "Should mention base game"
    );
}

// ---------------------------------------------------------------------------
// Case-insensitivity test
// ---------------------------------------------------------------------------

#[test]
fn test_event_case_insensitive_namespace() {
    let input = r#"
add_namespace = My_Test_Case
country_event = {
    id = my_test_case.1
    hidden = yes
    is_triggered_only = yes
}
"#;
    let diags = run_event_visitor(
        input,
        "file:///events/test.txt",
        &[("My_Test_Case", "/events/test.txt")],
    );
    let ns_diags = namespace_diags(&diags);
    assert!(
        ns_diags.is_empty(),
        "Case-insensitive namespace should produce no HOM3008"
    );
}

// ---------------------------------------------------------------------------
// Numeric legacy ID test
// ---------------------------------------------------------------------------

#[test]
fn test_event_numeric_legacy_id_no_namespace() {
    let input = r#"
country_event = {
    id = 90001
    hidden = yes
    is_triggered_only = yes
}
"#;
    let diags = run_event_visitor(input, "file:///events/test.txt", &[]);
    let ns_diags = namespace_diags(&diags);
    assert!(
        ns_diags.is_empty(),
        "Numeric legacy ID should produce no HOM3008"
    );
}

// ---------------------------------------------------------------------------
// Mixed scenarios test
// ---------------------------------------------------------------------------

#[test]
fn test_event_mixed_namespaces_in_one_file() {
    let input = r#"
add_namespace = ns_test

country_event = { id = ns_test.1 hidden = yes is_triggered_only = yes }
country_event = { id = bad_ns.1 hidden = yes is_triggered_only = yes }
country_event = { id = 99999 hidden = yes is_triggered_only = yes }
country_event = { id = late_ns.1 hidden = yes is_triggered_only = yes }
add_namespace = late_ns
"#;
    let diags = run_event_visitor(
        input,
        "file:///events/test.txt",
        &[
            ("ns_test", "/events/test.txt"),
            ("late_ns", "/events/test.txt"),
        ],
    );
    let ns_diags = namespace_diags(&diags);
    assert_eq!(
        ns_diags.len(),
        2,
        "bad_ns and late_ns should produce HOM3008"
    );
    for d in &ns_diags {
        assert_eq!(d.severity, Some(DiagnosticSeverity::ERROR));
    }
    let msgs: Vec<&str> = ns_diags.iter().map(|d| d.message.as_str()).collect();
    assert!(msgs.iter().any(|m| m.contains("LATER")));
    assert!(msgs.iter().any(|m| m.contains("Malformed token")));
}

// ---------------------------------------------------------------------------
// Duplicate namespace (HOM3012) tests
// ---------------------------------------------------------------------------

#[test]
fn test_duplicate_namespace_cross_file_gets_diagnostic() {
    let input = r#"
add_namespace = dup_ns
country_event = { id = dup_ns.1 hidden = yes is_triggered_only = yes }
"#;
    let diags = run_event_visitor(
        input,
        "file:///events/zzz_events.txt",
        &[("dup_ns", "/events/aaa_events.txt")],
    );
    let dup_diags = duplicate_diags(&diags);
    assert_eq!(
        dup_diags.len(),
        1,
        "Cross-file duplicate should produce HOM3012"
    );
    assert_eq!(dup_diags[0].severity, Some(DiagnosticSeverity::INFORMATION));
}

#[test]
fn test_duplicate_namespace_same_file_no_diagnostic() {
    let input = r#"
add_namespace = same_file_ns
add_namespace = same_file_ns
country_event = { id = same_file_ns.1 hidden = yes is_triggered_only = yes }
"#;
    let diags = run_event_visitor(
        input,
        "file:///events/test.txt",
        &[("same_file_ns", "/events/test.txt")],
    );
    let dup_diags = duplicate_diags(&diags);
    assert!(
        dup_diags.is_empty(),
        "Same-file duplicate should not produce HOM3012"
    );
}

// ---------------------------------------------------------------------------
// ai_chance (HOM3017) tests
// ---------------------------------------------------------------------------

#[test]
fn test_ai_chance_skipped_for_single_option() {
    let input = r#"
add_namespace = test_ns
country_event = {
    id = test_ns.1 hidden = yes is_triggered_only = yes
    option = { name = test.1.a }
}
"#;
    let diags = run_event_visitor(
        input,
        "file:///events/test.txt",
        &[("test_ns", "/events/test.txt")],
    );
    let ai = ai_chance_diags(&diags);
    assert!(ai.is_empty(), "Single option should not produce HOM3017");
}

#[test]
fn test_ai_chance_fires_for_two_options_with_one_missing() {
    let input = r#"
add_namespace = test_ns
country_event = {
    id = test_ns.1 hidden = yes is_triggered_only = yes
    option = { name = test.1.a }
    option = { name = test.1.b ai_chance = { base = 50 } }
}
"#;
    let diags = run_event_visitor(
        input,
        "file:///events/test.txt",
        &[("test_ns", "/events/test.txt")],
    );
    let ai = ai_chance_diags(&diags);
    assert_eq!(
        ai.len(),
        1,
        "2 options with 1 missing ai_chance should produce 1 HOM3017"
    );
    assert!(
        ai[0].message.contains("1 of 2"),
        "Message: {}",
        ai[0].message
    );
}

// ---------------------------------------------------------------------------
// Events subdirectory detection (HOM3021 path pattern) tests
// ---------------------------------------------------------------------------

fn is_events_subdirectory_path(uri: &str) -> bool {
    if !uri.ends_with(".txt") {
        return false;
    }
    if let Some(events_pos) = uri.find("/events/") {
        let after_events = &uri[events_pos + 8..];
        return after_events.contains('/');
    }
    false
}

#[test]
fn test_events_subdirectory_detected() {
    assert!(is_events_subdirectory_path(
        "file:///workspace/events/subdir/my_event.txt"
    ));
    assert!(is_events_subdirectory_path(
        "file:///workspace/events/nested/deep/path.txt"
    ));
    assert!(is_events_subdirectory_path(
        "file:///C:/mod/events/subdir/event.txt"
    ));
}

#[test]
fn test_events_root_no_diagnostic() {
    assert!(!is_events_subdirectory_path(
        "file:///workspace/events/my_event.txt"
    ));
    assert!(!is_events_subdirectory_path(
        "file:///workspace/events/test.txt"
    ));
}

#[test]
fn test_non_events_path_no_diagnostic() {
    assert!(!is_events_subdirectory_path(
        "file:///workspace/common/ideas/test.txt"
    ));
    assert!(!is_events_subdirectory_path(
        "file:///workspace/localisation/test.yml"
    ));
}
