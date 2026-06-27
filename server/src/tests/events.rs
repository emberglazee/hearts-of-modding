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

/// Filter diagnostics to only HOM3008 (missing event namespace).
fn namespace_diags(diags: &[Diagnostic]) -> Vec<&Diagnostic> {
    diags
        .iter()
        .filter(|d| matches!(&d.code, Some(NumberOrString::String(c)) if c == "HOM3008"))
        .collect()
}

/// Filter diagnostics to only HOM3012 (duplicate event namespace).
fn duplicate_diags(diags: &[Diagnostic]) -> Vec<&Diagnostic> {
    diags
        .iter()
        .filter(|d| matches!(&d.code, Some(NumberOrString::String(c)) if c == "HOM3012"))
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_event_declared_namespace_same_file_before() {
    // Namespace declared BEFORE event in same file → no diagnostic
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
        "Declared namespace before event should produce no HOM3008, got {}: {:?}",
        ns_diags.len(),
        ns_diags.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn test_event_missing_namespace_undeclared() {
    // Namespace never declared anywhere → ERROR
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
        "Undeclared namespace should produce exactly one HOM3008"
    );
    assert_eq!(
        ns_diags[0].severity,
        Some(DiagnosticSeverity::ERROR),
        "HOM3008 must be ERROR severity (event cannot fire without namespace)"
    );
    assert!(
        ns_diags[0].message.contains("Malformed token"),
        "HOM3008 message should mention 'Malformed token': {}",
        ns_diags[0].message
    );
}

#[test]
fn test_event_namespace_declared_later_in_same_file() {
    // Namespace declared AFTER event in same file → ERROR with reorder message
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
        "Namespace declared later in same file should produce HOM3008"
    );
    assert_eq!(
        ns_diags[0].severity,
        Some(DiagnosticSeverity::ERROR),
        "HOM3008 must be ERROR severity"
    );
    assert!(
        ns_diags[0].message.contains("LATER"),
        "Message should mention 'LATER': {}",
        ns_diags[0].message
    );
}

#[test]
fn test_event_namespace_in_file_that_loads_after() {
    // Namespace declared in a file that loads AFTER the current file → ERROR
    let input = r#"
country_event = {
    id = after_ns.1
    hidden = yes
    is_triggered_only = yes
}
"#;
    let diags = run_event_visitor(
        input,
        "file:///events/aaa_events.txt", // loads first (ASCII)
        &[("after_ns", "/events/zzz_events.txt")], // loads after
    );
    let ns_diags = namespace_diags(&diags);
    assert_eq!(
        ns_diags.len(),
        1,
        "Namespace in file that loads after should produce HOM3008"
    );
    assert!(
        ns_diags[0].message.contains("loads AFTER this one"),
        "Message should mention 'loads AFTER this one': {}",
        ns_diags[0].message
    );
}

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
        "Case-insensitive namespace should produce no HOM3008, got {}",
        ns_diags.len()
    );
}

#[test]
fn test_event_namespace_in_file_that_loads_before() {
    // Namespace declared in a file that loads BEFORE → available → no diagnostic
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
        "Namespace in file that loads before should produce no HOM3008, got {}",
        ns_diags.len()
    );
}

#[test]
fn test_event_numeric_legacy_id_no_namespace() {
    // Pure numeric legacy ID → no namespace check needed
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
        "Numeric legacy ID should produce no HOM3008, got {}",
        ns_diags.len()
    );
}

// ---------------------------------------------------------------------------
// Cross-layer ordering: namespace in game path, event in workspace
// ---------------------------------------------------------------------------

/// Like run_event_visitor but allows setting a game_path for cross-layer tests.
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

#[test]
fn test_event_namespace_from_game_path_available_to_workspace() {
    // Namespace declared in a game-path file (zzz_vanilla.txt, last alpha)
    // Event in a workspace file (aaa_mod.txt, first alpha)
    // Vanilla loads before mod regardless of filename → available
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
            "/game/Hearts of Iron IV/events/zzz_vanilla.txt",
        )],
        Some("/game/Hearts of Iron IV"),
    );
    let ns_diags = namespace_diags(&diags);
    assert!(
        ns_diags.is_empty(),
        "Vanilla namespace should be available to mod regardless of filename, got {} HOM3008",
        ns_diags.len()
    );
}

#[test]
fn test_event_namespace_from_workspace_not_available_to_vanilla() {
    // Namespace declared in a workspace file (aaa_mod.txt)
    // but validated file is a game-path file (zzz_vanilla.txt, last alpha)
    // A mod namespace is NOT available to vanilla files (vanilla loads first)
    let input = r#"
country_event = {
    id = mod_ns.1
    hidden = yes
    is_triggered_only = yes
}
"#;
    let diags = run_event_visitor_with_game_path(
        input,
        "file:///game/Hearts%20of%20Iron%20IV/events/zzz_vanilla.txt",
        &[("mod_ns", "/workspace/events/aaa_mod.txt")],
        Some("/game/Hearts of Iron IV"),
    );
    let ns_diags = namespace_diags(&diags);
    assert_eq!(
        ns_diags.len(),
        1,
        "Mod namespace should NOT be available to vanilla files"
    );
    assert!(
        ns_diags[0].message.contains("base game"),
        "Should mention base game: {}",
        ns_diags[0].message
    );
}

// ---------------------------------------------------------------------------
// Duplicate namespace (HOM3012) tests
// ---------------------------------------------------------------------------

#[test]
fn test_duplicate_namespace_cross_file_gets_diagnostic() {
    // Two files declare the same namespace → both should get HOM3012
    // Simulate validating File B where namespace 'dup_ns' is already in File A.
    // This test shows that the second file's duplicate is detected.
    let input = r#"
add_namespace = dup_ns
country_event = {
    id = dup_ns.1
    hidden = yes
    is_triggered_only = yes
}
"#;
    // File A (aaa_events.txt) already declared dup_ns in scanner data
    let diags = run_event_visitor(
        input,
        "file:///events/zzz_events.txt",         // File B
        &[("dup_ns", "/events/aaa_events.txt")], // stored from File A
    );
    let dup_diags = duplicate_diags(&diags);
    assert_eq!(
        dup_diags.len(),
        1,
        "Cross-file duplicate namespace should produce HOM3012"
    );
    assert_eq!(
        dup_diags[0].severity,
        Some(DiagnosticSeverity::INFORMATION),
        "HOM3012 should be INFORMATION severity"
    );
}

#[test]
fn test_duplicate_namespace_same_file_no_diagnostic() {
    // Same namespace declared twice in the same file → no HOM3012
    // (the check compares paths and skips if they refer to the same file)
    let input = r#"
add_namespace = same_file_ns
add_namespace = same_file_ns
country_event = {
    id = same_file_ns.1
    hidden = yes
    is_triggered_only = yes
}
"#;
    let diags = run_event_visitor(
        input,
        "file:///events/test.txt",
        &[("same_file_ns", "/events/test.txt")],
    );
    let dup_diags = duplicate_diags(&diags);
    assert!(
        dup_diags.is_empty(),
        "Same-file duplicate namespace should not produce HOM3012, got {}",
        dup_diags.len()
    );
    // Also verify the event itself is not flagged (namespace IS seen)
    let ns_diags = namespace_diags(&diags);
    assert!(
        ns_diags.is_empty(),
        "Event with same-file duplicate namespace should not produce HOM3008"
    );
}

// ---------------------------------------------------------------------------
// Events subdirectory detection (HOM3021 path pattern)
// ---------------------------------------------------------------------------

/// Check whether a URI string would be flagged as an events/ subdirectory file.
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
    assert!(
        is_events_subdirectory_path("file:///workspace/events/subdir/my_event.txt"),
        "File in events/subdir/ should be flagged"
    );
    assert!(
        is_events_subdirectory_path("file:///workspace/events/nested/deep/path.txt"),
        "File in events/nested/deep/ should be flagged"
    );
    assert!(
        is_events_subdirectory_path("file:///C:/mod/events/subdir/event.txt"),
        "Windows URI in events/subdir/ should be flagged"
    );
}

#[test]
fn test_events_root_no_diagnostic() {
    assert!(
        !is_events_subdirectory_path("file:///workspace/events/my_event.txt"),
        "File directly in events/ should NOT be flagged"
    );
    assert!(
        !is_events_subdirectory_path("file:///workspace/events/test.txt"),
        "File directly in events/ should NOT be flagged"
    );
}

#[test]
fn test_non_events_path_no_diagnostic() {
    assert!(
        !is_events_subdirectory_path("file:///workspace/common/ideas/test.txt"),
        "File in common/ideas/ should NOT be flagged"
    );
    assert!(
        !is_events_subdirectory_path("file:///workspace/localisation/test.yml"),
        "Non-txt file should NOT be flagged"
    );
}

#[test]
fn test_event_mixed_namespaces_in_one_file() {
    // Test all patterns in a single file with ordering
    let input = r#"
add_namespace = ns_test

# This one has a namespace declared before it → OK
country_event = {
    id = ns_test.1
    hidden = yes
    is_triggered_only = yes
}

# This one uses an undeclared namespace → ERROR
country_event = {
    id = bad_ns.1
    hidden = yes
    is_triggered_only = yes
}

# This one uses a numeric legacy ID → OK
country_event = {
    id = 99999
    hidden = yes
    is_triggered_only = yes
}

# This one's namespace is declared AFTER it in same file → ERROR (reorder)
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
        &[
            ("ns_test", "/events/test.txt"),
            ("late_ns", "/events/test.txt"),
        ],
    );
    let ns_diags = namespace_diags(&diags);
    assert_eq!(
        ns_diags.len(),
        2,
        "bad_ns and late_ns should produce HOM3008; ns_test and 99999 should not"
    );

    // Verify both get ERROR severity
    for d in &ns_diags {
        assert_eq!(
            d.severity,
            Some(DiagnosticSeverity::ERROR),
            "All HOM3008 must be ERROR severity: {}",
            d.message
        );
    }

    // One should mention "LATER", one should mention undeclared
    let msgs: Vec<&str> = ns_diags.iter().map(|d| d.message.as_str()).collect();
    let has_later = msgs.iter().any(|m| m.contains("LATER"));
    let has_undeclared = msgs.iter().any(|m| m.contains("Malformed token"));
    assert!(
        has_later,
        "Should have a 'LATER in this file' message: {:?}",
        msgs
    );
    assert!(
        has_undeclared,
        "Should have a 'Malformed token' message for truly undeclared: {:?}",
        msgs
    );
}
