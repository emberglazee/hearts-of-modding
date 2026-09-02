use crate::parser::parser::parse_script;
use crate::scope::scope::{Scope, ScopeCtx, ScopeStack, initial_scope_for_uri};
use crate::utils::symbol_search::find_identifier_at;
use dashmap::DashMap;
use tower_lsp_server::ls_types::Position;

fn scopes_at(content: &str, uri: &str, line: u32, ch: u32) -> Vec<Scope> {
    let (script, _) = parse_script(content);
    let initial = initial_scope_for_uri(uri);
    let mut stack = ScopeStack::new(initial);
    let event_targets = DashMap::new();
    let characters = DashMap::new();
    let achievements = DashMap::new();
    let sctx = ScopeCtx {
        uri,
        event_targets: Some(&event_targets),
        characters: Some(&characters),
        achievements: Some(&achievements),
        in_random_list: false,
        state_targeted: false,
    };
    let pos = Position {
        line,
        character: ch,
    };
    if let Some((id, scopes, _, _)) = find_identifier_at(&script, pos, &mut stack, &sctx) {
        println!(
            "found id='{}' scopes={:?} -> {}",
            id,
            scopes,
            scopes
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(" > ")
        );
        scopes
    } else {
        panic!("no id at {}:{}", line, ch);
    }
}

#[test]
fn hover_scripted_effect_transparent_filter() {
    let uri = "/mod/common/scripted_effects/test.txt";
    // any_other_country is a value_trigger (no pushes_scope) so its body stays ScriptedEffect.
    // Use any_country (Global -> Country) to test real scope push.
    let content = "some_scripted_effect = {\n    if = {\n        limit = {\n            any_other_country = {\n                add_political_power = 100\n            }\n        }\n    }\n}\n";
    // hover over any_other_country key (line 3, char ~12) — transparent if/limit filtered
    let scopes = scopes_at(content, uri, 3, 12);
    assert_eq!(
        scopes,
        vec![Scope::ScriptedEffect],
        "hover over any_other_country should be single ScriptedEffect, got {:?}",
        scopes
    );

    // inside any_other_country block stays ScriptedEffect (no push)
    let scopes2 = scopes_at(content, uri, 4, 16);
    assert_eq!(
        scopes2,
        vec![Scope::ScriptedEffect],
        "inside any_other_country (non-pushing) stays ScriptedEffect {:?}",
        scopes2
    );

    // pushing case: any_country pushes Country
    let content_push = "some_scripted_effect = {\n    if = {\n        limit = {\n            any_country = {\n                add_political_power = 100\n            }\n        }\n    }\n}\n";
    let scopes3 = scopes_at(content_push, uri, 3, 12);
    assert_eq!(
        scopes3,
        vec![Scope::ScriptedEffect],
        "hover over any_country (before push) should be single ScriptedEffect {:?}",
        scopes3
    );
    let scopes4 = scopes_at(content_push, uri, 4, 16);
    assert_eq!(
        scopes4,
        vec![Scope::ScriptedEffect, Scope::Country],
        "inside any_country should be ScriptedEffect > Country {:?}",
        scopes4
    );
}

#[test]
fn hover_decision_transparent_filter() {
    let uri = "/mod/common/decisions/test.txt";
    let content = "my_decision = {\n    if = {\n        limit = {\n            any_other_country = {\n                add_political_power = 100\n            }\n        }\n    }\n}\n";
    let scopes = scopes_at(content, uri, 3, 12);
    assert_eq!(
        scopes,
        vec![Scope::Country],
        "decision any_other_country should be single Country {:?}",
        scopes
    );
    let scopes2 = scopes_at(content, uri, 4, 16);
    assert_eq!(
        scopes2,
        vec![Scope::Country],
        "inside non-pushing stays Country {:?}",
        scopes2
    );

    let content_push = "my_decision = {\n    if = {\n        limit = {\n            any_country = {\n                add_political_power = 100\n            }\n        }\n    }\n}\n";
    let scopes3 = scopes_at(content_push, uri, 3, 12);
    assert_eq!(
        scopes3,
        vec![Scope::Country],
        "decision any_country before push single Country {:?}",
        scopes3
    );
    let scopes4 = scopes_at(content_push, uri, 4, 16);
    assert_eq!(
        scopes4,
        vec![Scope::Country, Scope::Country],
        "inside any_country should be Country > Country {:?}",
        scopes4
    );
}
