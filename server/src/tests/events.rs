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
        "file:///events/zzz_events.txt", // loads last (ASCII)
        &[("before_ns", "/events/aaa_events.txt")], // loads first
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

#[test]
fn test_event_case_insensitive_namespace() {
    // Case mismatch between add_namespace and event ID → should still work
    // (Clausewitz engine is case-insensitive for identifiers)
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
