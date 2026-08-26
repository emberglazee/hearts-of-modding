use crate::data::interner::InternedStr;
use crate::data::layered_value::LayeredValue;
use crate::parser::ast;
use crate::rules::ValidationRule;
use crate::rules::events::EventValidationRule;
use crate::rules::visitor::AstVisitor;
use crate::scanner::event_scanner::Event;
use crate::scope::scope::Scope;
use crate::test_support::TestCtx;
use dashmap::DashMap;
use std::sync::Arc;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

// ---------------------------------------------------------------------------
// Test runner
// ---------------------------------------------------------------------------

/// Run the EventValidationRule (with visitor) over a script and return all
/// diagnostics.
///
/// One runner covers the three historical variants (plain, pre-populated
/// events map, game_path): seed via `TestCtx` builders instead of three
/// hand-built `ValidationContext` literals.
fn run_event_visitor_ctx(
    input: &str,
    uri: &str,
    declared_namespaces: &[(&str, &str)],
    events: Option<&DashMap<InternedStr, LayeredValue<Event>>>,
    game_path: Option<&str>,
) -> Vec<Diagnostic> {
    let mut ctx_builder = TestCtx::new().with_event_namespaces(declared_namespaces);
    if let Some(gp) = game_path {
        ctx_builder = ctx_builder.with_game_path(Some(gp));
    }
    if let Some(events) = events {
        for entry in events.iter() {
            ctx_builder
                .data()
                .events
                .insert(entry.key().clone(), entry.value().clone());
        }
    }

    let rules: Vec<Box<dyn ValidationRule>> = vec![Box::new(EventValidationRule)];
    let visitors: Vec<Box<dyn AstVisitor>> = vec![EventValidationRule::visitor()];
    ctx_builder.walk(input, uri, Scope::Country, rules, visitors)
}

fn run_event_visitor(
    input: &str,
    uri: &str,
    declared_namespaces: &[(&str, &str)],
) -> Vec<Diagnostic> {
    run_event_visitor_ctx(input, uri, declared_namespaces, None, None)
}

/// Like [`run_event_visitor`] but with a pre-populated events DashMap (for
/// testing broken-reference and cross-file trigger validation).
fn run_event_visitor_with_events(
    input: &str,
    uri: &str,
    declared_namespaces: &[(&str, &str)],
    events: &DashMap<InternedStr, LayeredValue<Event>>,
) -> Vec<Diagnostic> {
    run_event_visitor_ctx(input, uri, declared_namespaces, Some(events), None)
}

fn run_event_visitor_with_game_path(
    input: &str,
    uri: &str,
    declared_namespaces: &[(&str, &str)],
    game_path: Option<&str>,
) -> Vec<Diagnostic> {
    run_event_visitor_ctx(input, uri, declared_namespaces, None, game_path)
}

/// Helper to add default namespaces for a list of files.
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
// Non-events directory: country_event used as effect, not definition
// ---------------------------------------------------------------------------

#[test]
fn test_event_namespace_in_decisions_file_skips_ordering() {
    let input = r#"
country_event = { id = dec_ns.1 }
"#;
    let diags = run_event_visitor(
        input,
        "file:///common/decisions/test_decisions.txt",
        &[("dec_ns", "/events/some_events.txt")],
    );
    let ns_diags = namespace_diags(&diags);
    assert!(
        ns_diags.is_empty(),
        "Decisions file should not get ordering-based HOM3008 when namespace exists in events"
    );
}

#[test]
fn test_event_namespace_in_decisions_file_still_errors_if_missing() {
    let input = r#"
country_event = { id = missing_ns.1 }
"#;
    let diags = run_event_visitor(input, "file:///common/decisions/test_decisions.txt", &[]);
    let ns_diags = namespace_diags(&diags);
    assert_eq!(
        ns_diags.len(),
        1,
        "Missing namespace in decisions file should still produce HOM3008"
    );
    assert!(ns_diags[0].message.contains("Malformed token"));
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

// ---------------------------------------------------------------------------
// Broken event reference (HOM3022) tests
// ---------------------------------------------------------------------------

fn broken_ref_diags(diags: &[Diagnostic]) -> Vec<&Diagnostic> {
    diags
        .iter()
        .filter(|d| matches!(&d.code, Some(NumberOrString::String(c)) if c == "HOM3022"))
        .collect()
}

#[test]
fn test_broken_event_reference_block_form_emits_diagnostic() {
    let events: DashMap<InternedStr, LayeredValue<Event>> = DashMap::new();
    events.insert(
        Arc::from("existing.1"),
        LayeredValue::new(Event {
            id: "existing.1".to_string(),
            event_type: "country_event".to_string(),
            path: Arc::from("events/existing.txt"),
            range: ast::Range {
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 0,
            },
            triggered_events: vec![],
        }),
    );

    let input = r#"
add_namespace = test_ns
country_event = {
    id = test_ns.1
    option = {
        name = test.1.a
        country_event = { id = missing.1 }
    }
}
"#;
    let diags = run_event_visitor_with_events(
        input,
        "file:///events/test.txt",
        &[("test_ns", "/events/test.txt")],
        &events,
    );
    let refs = broken_ref_diags(&diags);
    assert_eq!(refs.len(), 1, "Missing event target should produce HOM3022");
    assert!(
        refs[0].message.contains("missing.1"),
        "Message: {}",
        refs[0].message
    );
}

#[test]
fn test_broken_event_reference_block_form_to_existing_event_no_diagnostic() {
    let events: DashMap<InternedStr, LayeredValue<Event>> = DashMap::new();
    events.insert(
        Arc::from("existing.1"),
        LayeredValue::new(Event {
            id: "existing.1".to_string(),
            event_type: "country_event".to_string(),
            path: Arc::from("events/existing.txt"),
            range: ast::Range {
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 0,
            },
            triggered_events: vec![],
        }),
    );

    let input = r#"
add_namespace = test_ns
country_event = {
    id = test_ns.1
    option = {
        name = test.1.a
        country_event = { id = existing.1 }
    }
}
"#;
    let diags = run_event_visitor_with_events(
        input,
        "file:///events/test.txt",
        &[("test_ns", "/events/test.txt")],
        &events,
    );
    let refs = broken_ref_diags(&diags);
    assert!(
        refs.is_empty(),
        "Existing event target should not produce HOM3022"
    );
}

#[test]
fn test_broken_event_reference_string_form_emits_diagnostic() {
    let events: DashMap<InternedStr, LayeredValue<Event>> = DashMap::new();
    events.insert(
        Arc::from("existing.1"),
        LayeredValue::new(Event {
            id: "existing.1".to_string(),
            event_type: "country_event".to_string(),
            path: Arc::from("events/existing.txt"),
            range: ast::Range {
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 0,
            },
            triggered_events: vec![],
        }),
    );

    let input = r#"
add_namespace = test_ns
country_event = {
    id = test_ns.1
    option = {
        name = test.1.a
        country_event = "missing.1"
    }
}
"#;
    let diags = run_event_visitor_with_events(
        input,
        "file:///events/test.txt",
        &[("test_ns", "/events/test.txt")],
        &events,
    );
    let refs = broken_ref_diags(&diags);
    assert_eq!(
        refs.len(),
        1,
        "Missing event target in string form should produce HOM3022"
    );
    assert!(
        refs[0].message.contains("missing.1"),
        "Message: {}",
        refs[0].message
    );
}

#[test]
fn test_no_broken_reference_for_event_definition() {
    let events: DashMap<InternedStr, LayeredValue<Event>> = DashMap::new();
    let input = r#"
add_namespace = test_ns
country_event = { id = test_ns.1 }
"#;
    let diags = run_event_visitor_with_events(
        input,
        "file:///events/test.txt",
        &[("test_ns", "/events/test.txt")],
        &events,
    );
    let refs = broken_ref_diags(&diags);
    assert!(
        refs.is_empty(),
        "Top-level event definition should not produce HOM3022"
    );
}

#[test]
/// Regression test: event CALLs inside options should NOT trigger namespace
/// ordering checks (HOM3008). Reproduces the vanilla AAT_Finland.txt pattern
/// where a file sorted first in ASCII calls an event whose namespace is
/// declared in a file sorted later.
fn test_event_call_across_ascii_order_no_hom3008() {
    let events: DashMap<InternedStr, LayeredValue<Event>> = DashMap::new();
    events.insert(
        Arc::from("other_ns.1"),
        LayeredValue::new(Event {
            id: "other_ns.1".to_string(),
            event_type: "country_event".to_string(),
            path: Arc::from("events/ZZZ_test.txt"),
            range: ast::Range {
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 0,
            },
            triggered_events: vec![],
        }),
    );

    let input = r#"
add_namespace = call_ns
country_event = {
    id = call_ns.1
    is_triggered_only = yes
    option = {
        name = call_ns.1.a
        country_event = { id = other_ns.1 }
    }
}
"#;
    let diags = run_event_visitor_with_events(
        input,
        "file:///events/AAA_test.txt",
        // other_ns declared in ZZZ_test.txt — sorts AFTER AAA_test.txt.
        // If namespace ordering fired on the call, this would produce HOM3008.
        &[
            ("call_ns", "/events/AAA_test.txt"),
            ("other_ns", "/events/ZZZ_test.txt"),
        ],
        &events,
    );
    let ns_diags = namespace_diags(&diags);
    assert!(
        ns_diags.is_empty(),
        "Event call across ASCII order should NOT produce HOM3008: got {}",
        ns_diags.len()
    );
}

#[test]
/// Reproduces the exact AAT_Finland.txt pattern: news_event call inside
/// hidden_effect inside an option, where the namespace is declared in a
/// file that sorts after the current file in ASCII order.
fn test_news_event_call_across_ascii_order_no_hom3008() {
    let events: DashMap<InternedStr, LayeredValue<Event>> = DashMap::new();
    events.insert(
        Arc::from("aat_news.1"),
        LayeredValue::new(Event {
            id: "aat_news.1".to_string(),
            event_type: "news_event".to_string(),
            path: Arc::from("events/AAT_NewsEvents.txt"),
            range: ast::Range {
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 0,
            },
            triggered_events: vec![],
        }),
    );

    // Match the AAT_Finland.txt structure:
    // country_event = { id = AAT_finland_continuation_war.11
    //     option = {
    //         hidden_effect = {
    //             news_event = { id = aat_news.121 hours = 24 }
    //         }
    //     }
    // }
    let input = r#"
add_namespace = aat_finland
country_event = {
    id = aat_finland.1
    is_triggered_only = yes
    option = {
        name = aat_finland.1.a
        hidden_effect = {
            news_event = { id = aat_news.1 }
        }
    }
}
"#;
    // aat_news namespace is declared in AAT_NewsEvents.txt which sorts AFTER
    // AAT_Finland.txt — but this is a CALL, not a definition.
    let diags = run_event_visitor_with_events(
        input,
        "file:///events/AAT_Finland.txt",
        &[
            ("aat_finland", "/events/AAT_Finland.txt"),
            ("aat_news", "/events/AAT_NewsEvents.txt"),
        ],
        &events,
    );
    let ns_diags = namespace_diags(&diags);
    assert!(
        ns_diags.is_empty(),
        "news_event call across ASCII order should NOT produce HOM3008: got {}",
        ns_diags.len()
    );
}

/// Filter diagnostics to only HOM3016 (missing title/desc).
fn missing_title_diags(diags: &[Diagnostic]) -> Vec<&Diagnostic> {
    diags
        .iter()
        .filter(|d| {
            matches!(&d.code, Some(NumberOrString::String(c)) if c == crate::validation::advanced_validation::EVENT_MISSING_TITLE)
        })
        .collect()
}

#[test]
/// Regression: country_event = { id = X } in a decision file is a CALL,
/// not an event definition. HOM3016 should NOT fire for missing title/desc.
fn test_event_call_in_decision_file_no_hom3016() {
    let input = r#"
my_decision = {
    picture = GFX_decision
    fire_only_once = yes
    available = { always = yes }
    effect = {
        country_event = { id = test.1 }
    }
}
"#;
    let diags = run_event_visitor(input, "file:///common/decisions/test.txt", &[]);
    let title_diags = missing_title_diags(&diags);
    assert!(
        title_diags.is_empty(),
        "country_event in decision file should NOT produce HOM3016 (missing title/desc): got {}",
        title_diags.len()
    );
}

/// Two `country_event` blocks with the SAME id in one file must flag HOM3011
/// (duplicate event ID) — HOI4 loads only one event per ID.
#[test]
fn test_duplicate_event_id_hom3011() {
    let input = r#"
country_event = {
    id = test_dup.1
    title = "Some Title"
    desc = "Some desc"
    option = { name = OK ai_chance = { base = 1 } }
}
country_event = {
    id = test_dup.1
    title = "A Title"
    desc = "A desc"
    option = { name = OK ai_chance = { base = 1 } }
}
"#;
    let diags = run_event_visitor(input, "file:///common/events/test.txt", &[]);
    let dup: Vec<_> = diags
        .iter()
        .filter(|d| {
            matches!(
                d.code.as_ref(),
                Some(NumberOrString::String(s))
                    if s == crate::validation::advanced_validation::DUPLICATE_EVENT_ID
            )
        })
        .collect();
    assert_eq!(
        dup.len(),
        1,
        "expected exactly one HOM3011 (duplicate event ID); got {}: {:?}",
        dup.len(),
        diags
    );
    assert!(dup[0].message.contains("test_dup.1"));
}

// ---------------------------------------------------------------------------
// Dep-graph scrub on file DELETION — index-key spelling invariant
// ---------------------------------------------------------------------------

/// Deleting an event file must scrub its edges from `event_dep_graph`. The
/// delete arm reads the ID list from `events_file_index` — that read MUST go
/// through `index_key()` (the only sanctioned index-key builder, see
/// windows-path-normalization-2026-08-05) so any path spelling matches the
/// normalized keys the macros write. Feeding a backslash-spelled path proves
/// the invariant: `update_scanner_data_for_file` handles it (its own read is
/// keyed), so the deletion that follows must too.
///
/// Regression guard for the raw `.get(path_str)` miss in the
/// FileCategory::Events arm of `remove_path_from_scanner_data`.
#[test]
fn test_delete_scrubs_dep_graph_with_backslash_path_spelling() {
    use crate::ScannerData;
    use crate::scanner::incremental_scanner::{
        remove_path_from_scanner_data, update_scanner_data_for_file,
    };

    let data = ScannerData::new();

    // Insert with the forward-slash spelling (what the LSP handlers feed).
    let insert_path = "/mod/common/events/test_events.txt";
    // Dotted event IDs (`prefix.number`) carry everything the scanner needs;
    // no add_namespace declaration is required for edge collection.
    update_scanner_data_for_file(
        &data,
        insert_path,
        r#"country_event = {
	id = source_event.1
	immediate = { country_event = { id = target_event.1 } }
}
country_event = { id = target_event.1 }
"#,
    );
    assert_eq!(data.event_dep_graph.caller_count("target_event.1"), 1);

    // Delete via the same file under the OTHER separator convention. The
    // macros normalize both sides, so removal itself works either way — but
    // the stale-edge scrub silently no-ops when the pre-read misses.
    let delete_path = "\\mod\\common\\events\\test_events.txt";
    remove_path_from_scanner_data(&data, delete_path);

    assert!(
        data.events.is_empty(),
        "events map must be empty after deletion"
    );
    assert_eq!(
        data.event_dep_graph.callers_of("target_event.1"),
        Vec::<String>::new(),
        "dep graph must be scrubbed on deletion regardless of path spelling"
    );
}

// ---------------------------------------------------------------------------
// HOM3017: AI-invisible option triggers (is_ai = no / scripted triggers)
// ---------------------------------------------------------------------------

/// Wiki ground truth (event-modding.md): an option whose `trigger` is false
/// when the event fires "will not appear" — the AI cannot pick it — and a
/// missing `ai_chance` defaults to weight 1 of the proportional distribution.
/// So when every ai_chance-less option provably excludes the AI, the single
/// remaining visible option gets 100% and no diagnostic is warranted.
///
/// Direct `is_ai = no` in the option trigger → suppressed.
#[test]
fn hom3017_suppressed_for_direct_is_ai_no_trigger() {
    let input = r#"
country_event = {
    id = test.1
    title = TEST_1_T
    desc = TEST_1_D
    is_triggered_only = yes
    option = {
        name = TEST_1_A
        ai_chance = { factor = 100 }
    }
    option = {
        name = HoM.debug
        trigger = { is_ai = no }
    }
}
"#;
    let diags = run_event_visitor(input, "file:///events/aaa_test.txt", &[]);
    assert!(
        ai_chance_diags(&diags).is_empty(),
        "AI-invisible debug option must not require ai_chance: {:?}",
        diags
    );
}

/// `NOT = { is_ai = yes }` is the same statement → suppressed.
#[test]
fn hom3017_suppressed_for_not_is_ai_yes() {
    let input = r#"
country_event = {
    id = test.2
    title = TEST_2_T
    desc = TEST_2_D
    is_triggered_only = yes
    option = {
        name = TEST_2_A
        ai_chance = { factor = 100 }
    }
    option = {
        name = HoM.debug
        trigger = { NOT = { is_ai = yes } }
    }
}
"#;
    let diags = run_event_visitor(input, "file:///events/aaa_test.txt", &[]);
    assert!(ai_chance_diags(&diags).is_empty(), "{:?}", diags);
}

/// A scripted trigger whose body proves AI-invisibility (HoM's `dbug_mode`
/// pattern: AND of is_debug + is_ai = no) resolves through the scanner map
/// and suppresses the diagnostic.
#[test]
fn hom3017_suppressed_via_scripted_trigger_proof() {
    // Seed a real scripted trigger through the incremental-scanner path.
    let scripted = r#"
dbug_mode = {
	AND = {
		is_debug = yes
		is_ai = no
	}
}
harmless_check = {
	has_war = yes
}
"#;
    let ctx_builder = TestCtx::new()
        .with_file("/mod/common/scripted_triggers/hom.txt", scripted)
        .with_event_namespaces(&[("test", "/events/aaa_test.txt")]);

    let input = r#"
country_event = {
    id = test.3
    title = TEST_3_T
    desc = TEST_3_D
    is_triggered_only = yes
    option = {
        name = TEST_3_A
        ai_chance = { factor = 100 }
    }
    option = {
        name = HoM.debug
        trigger = { dbug_mode = yes }
    }
}
"#;
    let rules: Vec<Box<dyn ValidationRule>> = vec![Box::new(EventValidationRule)];
    let visitors: Vec<Box<dyn AstVisitor>> = vec![EventValidationRule::visitor()];
    let diags = ctx_builder.walk(
        input,
        "file:///events/aaa_test.txt",
        Scope::Country,
        rules,
        visitors,
    );
    assert!(
        ai_chance_diags(&diags).is_empty(),
        "scripted-trigger proof (dbug_mode) must suppress HOM3017: {:?}",
        diags
    );

    // Control: a scripted trigger WITHOUT an AI-invisibility proof must NOT
    // suppress — the diagnostic still fires.
    let input_control = r#"
country_event = {
    id = test.4
    title = TEST_4_T
    desc = TEST_4_D
    is_triggered_only = yes
    option = {
        name = TEST_4_A
        ai_chance = { factor = 100 }
    }
    option = {
        name = HoM.debug
        trigger = { harmless_check = yes }
    }
}
"#;
    let rules: Vec<Box<dyn ValidationRule>> = vec![Box::new(EventValidationRule)];
    let visitors: Vec<Box<dyn AstVisitor>> = vec![EventValidationRule::visitor()];
    let diags = ctx_builder.walk(
        input_control,
        "file:///events/aaa_test.txt",
        Scope::Country,
        rules,
        visitors,
    );
    assert_eq!(
        ai_chance_diags(&diags).len(),
        1,
        "non-proving scripted trigger must still flag HOM3017: {:?}",
        diags
    );
}

/// OR blocks never prove invisibility (one true arm keeps the option visible
/// to the AI) — the diagnostic stays.
#[test]
fn hom3017_not_suppressed_by_or_containing_is_ai_no() {
    let input = r#"
country_event = {
    id = test.5
    title = TEST_5_T
    desc = TEST_5_D
    is_triggered_only = yes
    option = {
        name = TEST_5_A
        ai_chance = { factor = 100 }
    }
    option = {
        name = TEST_5_B
        trigger = { OR = { is_ai = no has_country_flag = maybe } }
    }
}
"#;
    let diags = run_event_visitor(input, "file:///events/aaa_test.txt", &[]);
    assert_eq!(
        ai_chance_diags(&diags).len(),
        1,
        "OR arm proving AI-invisibility is NOT a proof for the whole OR: {:?}",
        diags
    );
}

/// Two AI-VISIBLE options both missing ai_chance: the weighted-choice
/// condition HOM3017 exists for — flagged.
#[test]
fn hom3017_still_flags_when_two_visible_options_lack_ai_chance() {
    let input = r#"
country_event = {
    id = test.6
    title = TEST_6_T
    desc = TEST_6_D
    is_triggered_only = yes
    option = {
        name = TEST_6_A
    }
    option = {
        name = TEST_6_B
        trigger = { always = yes }
    }
}
"#;
    let diags = run_event_visitor(input, "file:///events/aaa_test.txt", &[]);
    assert_eq!(
        ai_chance_diags(&diags).len(),
        1,
        "two visible options with no weights is exactly what HOM3017 exists for: {:?}",
        diags
    );
}

/// One debug option + one visible option WITH ai_chance + one visible option
/// WITHOUT: two visible options, one missing → still flagged (the missing one
/// dilutes the weighted choice).
#[test]
fn hom3017_flags_visible_missing_among_two_visible() {
    let input = r#"
country_event = {
    id = test.7
    title = TEST_7_T
    desc = TEST_7_D
    is_triggered_only = yes
    option = {
        name = TEST_7_A
        ai_chance = { factor = 100 }
    }
    option = {
        name = TEST_7_B
        trigger = { always = yes }
    }
    option = {
        name = HoM.debug
        trigger = { is_ai = no }
    }
}
"#;
    let diags = run_event_visitor(input, "file:///events/aaa_test.txt", &[]);
    assert_eq!(ai_chance_diags(&diags).len(), 1, "{:?}", diags);
}

/// EXACT reproduction of Hearts-Of-Minecraft IMP_Events.txt imp.1 + HoM's real
/// dbug_mode scripted trigger, seeded through the same scan path production
/// uses (with_file -> update_scripted).
#[test]
fn hom3017_real_world_imp1_dbug_mode() {
    let scripted = "dbug_mode = {\n\tAND = {\n\t\tis_debug = yes\n\t\tis_ai = no\n\t}\n}\n";
    let event_file = "country_event = {\t#The Imperian Situation\n\tid = imp.1\n\ttitle = imp.1.t\n\tdesc = imp.1.d\n\tpicture = GFX_long_live_the_empire\n\tis_triggered_only = yes\n\timmediate = { log = \"[GetDateText]: [Root.GetName]: event imp.1\" }\n\toption = {\n\t\tname = imp.1.a\n\t\tcustom_effect_tooltip = IMP_warn_tt\n\t\tadd_political_power = 120\n\t}\n\toption = {\n\t\tname = HoM.debug\n\t\ttrigger = { dbug_mode = yes }\n\t}\n}\n";

    let ctx_builder = TestCtx::new()
        .with_file(
            "/mod/common/scripted_triggers/HoM_scripted_triggers.txt",
            scripted,
        )
        .with_event_namespaces(&[("imp", "/events/IMP_Events.txt")]);

    // Sanity: the scanner must have computed the proof for dbug_mode.
    let entity = ctx_builder
        .scanner_data()
        .scripted_triggers
        .get("dbug_mode");
    assert!(
        entity
            .map(|e| e.resolve().guarantees_ai_invisible)
            .unwrap_or(false),
        "dbug_mode must be precomputed as AI-invisible"
    );

    let rules: Vec<Box<dyn ValidationRule>> = vec![Box::new(EventValidationRule)];
    let visitors: Vec<Box<dyn AstVisitor>> = vec![EventValidationRule::visitor()];
    let diags = ctx_builder.walk(
        event_file,
        "file:///events/IMP_Events.txt",
        Scope::Country,
        rules,
        visitors,
    );
    assert!(
        ai_chance_diags(&diags).is_empty(),
        "live repro: HOM3017 must be suppressed for dbug_mode debug option: {:?}",
        ai_chance_diags(&diags)
    );
}

/// Bisect: same repro but WITHOUT the trailing comment on the event line.
#[test]
fn hom3017_bisect_no_comment() {
    let scripted = "dbug_mode = {\n\tAND = {\n\t\tis_debug = yes\n\t\tis_ai = no\n\t}\n}\n";
    let event_file = "country_event = {\n\tid = imp.1\n\ttitle = imp.1.t\n\tdesc = imp.1.d\n\tpicture = GFX_long_live_the_empire\n\tis_triggered_only = yes\n\toption = {\n\t\tname = imp.1.a\n\t}\n\toption = {\n\t\tname = HoM.debug\n\t\ttrigger = { dbug_mode = yes }\n\t}\n}\n";
    let ctx_builder = TestCtx::new().with_file("/mod/common/scripted_triggers/hom.txt", scripted);
    let rules: Vec<Box<dyn ValidationRule>> = vec![Box::new(EventValidationRule)];
    let visitors: Vec<Box<dyn AstVisitor>> = vec![EventValidationRule::visitor()];
    let diags = ctx_builder.walk(
        event_file,
        "file:///events/x.txt",
        Scope::Country,
        rules,
        visitors,
    );
    assert!(
        ai_chance_diags(&diags).is_empty(),
        "{:?}",
        ai_chance_diags(&diags)
    );
}

/// Bisect 2: strip further — direct is_ai = no with the same surrounding
/// structure as the repro (immediate, picture, two options).
#[test]
fn hom3017_bisect_direct_is_ai_no_full_structure() {
    let event_file = "country_event = {\n\tid = imp.1\n\ttitle = imp.1.t\n\tdesc = imp.1.d\n\tpicture = GFX_long_live_the_empire\n\tis_triggered_only = yes\n\timmediate = { log = \"[GetDateText]: [Root.GetName]: event imp.1\" }\n\toption = {\n\t\tname = imp.1.a\n\t}\n\toption = {\n\t\tname = HoM.debug\n\t\ttrigger = { is_ai = no }\n\t}\n}\n";
    let ctx_builder = TestCtx::new();
    let rules: Vec<Box<dyn ValidationRule>> = vec![Box::new(EventValidationRule)];
    let visitors: Vec<Box<dyn AstVisitor>> = vec![EventValidationRule::visitor()];
    let diags = ctx_builder.walk(
        event_file,
        "file:///events/x.txt",
        Scope::Country,
        rules,
        visitors,
    );
    assert!(
        ai_chance_diags(&diags).is_empty(),
        "{:?}",
        ai_chance_diags(&diags)
    );
}

/// Bisect 3: minimal repro of the earlier passing test but with the FULL
/// option set (name first, then trigger) and tab indentation.
#[test]
fn hom3017_bisect_tabs_name_then_trigger() {
    let event_file = "country_event = {\n\tid = t.1\n\ttitle = T\n\tdesc = D\n\tis_triggered_only = yes\n\toption = {\n\t\tname = A\n\t\tai_chance = { factor = 100 }\n\t}\n\toption = {\n\t\tname = HoM.debug\n\t\ttrigger = { dbug_mode = yes }\n\t}\n}\n";
    let scripted = "dbug_mode = {\n\tAND = {\n\t\tis_debug = yes\n\t\tis_ai = no\n\t}\n}\n";
    let ctx_builder = TestCtx::new().with_file("/mod/common/scripted_triggers/hom.txt", scripted);
    let rules: Vec<Box<dyn ValidationRule>> = vec![Box::new(EventValidationRule)];
    let visitors: Vec<Box<dyn AstVisitor>> = vec![EventValidationRule::visitor()];
    let diags = ctx_builder.walk(
        event_file,
        "file:///events/x.txt",
        Scope::Country,
        rules,
        visitors,
    );
    assert!(
        ai_chance_diags(&diags).is_empty(),
        "{:?}",
        ai_chance_diags(&diags)
    );
}

#[test]
fn bis4_with_picture_only() {
    let event_file = "country_event = {\n\tid = imp.1\n\ttitle = imp.1.t\n\tdesc = imp.1.d\n\tpicture = GFX_long_live_the_empire\n\tis_triggered_only = yes\n\toption = {\n\t\tname = A\n\t\tai_chance = { factor = 100 }\n\t}\n\toption = {\n\t\tname = B\n\t\ttrigger = { is_ai = no }\n\t}\n}\n";
    let ctx_builder = TestCtx::new();
    let rules: Vec<Box<dyn ValidationRule>> = vec![Box::new(EventValidationRule)];
    let visitors: Vec<Box<dyn AstVisitor>> = vec![EventValidationRule::visitor()];
    let diags = ctx_builder.walk(
        event_file,
        "file:///events/x.txt",
        Scope::Country,
        rules,
        visitors,
    );
    assert!(
        ai_chance_diags(&diags).is_empty(),
        "picture-only: {:?}",
        ai_chance_diags(&diags)
    );
}

#[test]
fn bis5_with_immediate_only() {
    let event_file = "country_event = {\n\tid = imp.1\n\ttitle = imp.1.t\n\tdesc = imp.1.d\n\tis_triggered_only = yes\n\timmediate = { log = \"[GetDateText]: [Root.GetName]: event imp.1\" }\n\toption = {\n\t\tname = A\n\t\tai_chance = { factor = 100 }\n\t}\n\toption = {\n\t\tname = B\n\t\ttrigger = { is_ai = no }\n\t}\n}\n";
    let ctx_builder = TestCtx::new();
    let rules: Vec<Box<dyn ValidationRule>> = vec![Box::new(EventValidationRule)];
    let visitors: Vec<Box<dyn AstVisitor>> = vec![EventValidationRule::visitor()];
    let diags = ctx_builder.walk(
        event_file,
        "file:///events/x.txt",
        Scope::Country,
        rules,
        visitors,
    );
    assert!(
        ai_chance_diags(&diags).is_empty(),
        "immediate-only: {:?}",
        ai_chance_diags(&diags)
    );
}

#[test]
fn bis6_direct_no_ai_chance_on_a() {
    // The failing bisect had option A WITHOUT ai_chance too — but that should
    // still suppress for option B... unless BOTH missing means the count is 1
    // because A (visible, missing) counts. THAT'S EXPECTED BEHAVIOR!
    let event_file = "country_event = {\n\tid = imp.1\n\ttitle = imp.1.t\n\tdesc = imp.1.d\n\tpicture = GFX_long_live_the_empire\n\tis_triggered_only = yes\n\timmediate = { log = \"x\" }\n\toption = {\n\t\tname = imp.1.a\n\t}\n\toption = {\n\t\tname = HoM.debug\n\t\ttrigger = { is_ai = no }\n\t}\n}\n";
    let ctx_builder = TestCtx::new();
    let rules: Vec<Box<dyn ValidationRule>> = vec![Box::new(EventValidationRule)];
    let visitors: Vec<Box<dyn AstVisitor>> = vec![EventValidationRule::visitor()];
    let diags = ctx_builder.walk(
        event_file,
        "file:///events/x.txt",
        Scope::Country,
        rules,
        visitors,
    );
    // Option A is the ONLY AI-visible option: its pick is forced (weight 1 of
    // total 1), so ai_chance blocks are irrelevant — suppressed.
    assert!(
        ai_chance_diags(&diags).is_empty(),
        "forced choice (single visible option) must suppress: {:?}",
        ai_chance_diags(&diags)
    );
}
