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

/// Run the EventValidationRule over a script and return all diagnostics.
/// `declared_namespaces` controls which namespaces are pre-declared in scanner data.
fn run_event_rule(input: &str, uri: &str, declared_namespaces: &[&str]) -> Vec<Diagnostic> {
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

    for ns in declared_namespaces {
        event_namespaces.insert(
            std::sync::Arc::<str>::from(*ns),
            crate::data::layered_value::LayeredValue::new(
                crate::scanner::event_namespace_scanner::EventNamespace {
                    name: ns.to_string(),
                    path: std::sync::Arc::<str>::from("test.txt"),
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
    let mut visitors: Vec<Box<dyn AstVisitor>> = Vec::new();
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
fn test_event_declared_namespace() {
    let input = r#"
add_namespace = ns_test
country_event = {
    id = ns_test.1
    hidden = yes
    is_triggered_only = yes
}
"#;
    let diags = run_event_rule(input, "file:///test.txt", &["ns_test"]);
    let ns_diags = namespace_diags(&diags);
    assert!(
        ns_diags.is_empty(),
        "Declared namespace should produce no HOM3008, got {}: {:?}",
        ns_diags.len(),
        ns_diags.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn test_event_missing_namespace() {
    // NOTE: namespace names starting with "no" (like "no_ns") trigger a parser
    // quirk where bare "no" is treated as boolean. Use a namespace that doesn't
    // conflict with HOI4's boolean keywords.
    let input = r#"
country_event = {
    id = undef_ns.1
    hidden = yes
    is_triggered_only = yes
}
"#;
    let diags = run_event_rule(input, "file:///test.txt", &[]);
    let ns_diags = namespace_diags(&diags);
    assert_eq!(
        ns_diags.len(),
        1,
        "Missing namespace should produce exactly one HOM3008"
    );

    // Verify severity is ERROR (empirically confirmed: event is completely unregistered)
    assert_eq!(
        ns_diags[0].severity,
        Some(DiagnosticSeverity::ERROR),
        "HOM3008 must be ERROR severity (event cannot fire without namespace)"
    );

    // Verify message mentions "Malformed token" (links to game.log)
    assert!(
        ns_diags[0].message.contains("Malformed token"),
        "HOM3008 message should mention 'Malformed token': {}",
        ns_diags[0].message
    );
}

#[test]
fn test_event_numeric_legacy_id() {
    let input = r#"
country_event = {
    id = 90001
    hidden = yes
    is_triggered_only = yes
}
"#;
    let diags = run_event_rule(input, "file:///test.txt", &[]);
    let ns_diags = namespace_diags(&diags);
    assert!(
        ns_diags.is_empty(),
        "Numeric legacy ID should produce no HOM3008, got {}",
        ns_diags.len()
    );
}

#[test]
fn test_event_mixed_namespaces_in_one_file() {
    // Test all three patterns in a single file
    let input = r#"
add_namespace = ns_test

# This one has a namespace
country_event = {
    id = ns_test.1
    hidden = yes
    is_triggered_only = yes
}

# This one is missing its namespace declaration
country_event = {
    id = bad_ns.1
    hidden = yes
    is_triggered_only = yes
}

# This one uses a numeric legacy ID
country_event = {
    id = 99999
    hidden = yes
    is_triggered_only = yes
}
"#;
    let diags = run_event_rule(input, "file:///test.txt", &["ns_test"]);
    let ns_diags = namespace_diags(&diags);
    assert_eq!(
        ns_diags.len(),
        1,
        "Only the bad_ns event should produce HOM3008"
    );
    assert!(
        ns_diags[0].message.contains("bad_ns"),
        "HOM3008 should reference the broken namespace: {}",
        ns_diags[0].message
    );
    assert_eq!(
        ns_diags[0].severity,
        Some(DiagnosticSeverity::ERROR),
        "HOM3008 must be ERROR severity"
    );
}
