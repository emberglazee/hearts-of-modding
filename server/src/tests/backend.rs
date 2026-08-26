//! Tests for `backend.rs` internals: compute-pool scheduling, CSV terrain
//! checks, and dynamic ideology keyword generation. Extracted from inline
//! `#[cfg(test)] mod tests` so the 3.7k-line source file stays readable.

use crate::backend::{
    IDEOLOGY_MODIFIER_SUFFIXES, build_dynamic_ideology_keywords, check_province_terrain_csv,
    run_in_compute_pool,
};
use crate::data::scanner_data::ScannerData;
use crate::parser::ast;
use std::collections::{HashMap, HashSet};
use tower_lsp_server::ls_types::NumberOrString;

#[tokio::test(flavor = "current_thread")]
async fn compute_pool_work_does_not_block_async_runtime() {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .expect("test compute pool");
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let work = run_in_compute_pool(&pool, move || {
        release_rx.recv().expect("release compute task");
    });
    tokio::pin!(work);

    tokio::select! {
        _ = tokio::time::sleep(std::time::Duration::from_millis(20)) => {}
        result = work.as_mut() => panic!("compute task unexpectedly completed: {result:?}"),
    }

    release_tx.send(()).expect("release compute task");
    tokio::time::timeout(std::time::Duration::from_secs(1), work.as_mut())
        .await
        .expect("compute task timed out")
        .expect("compute task failed");
}

#[test]
fn test_check_province_terrain_csv_invalid() {
    let mut names = HashSet::new();
    names.insert("ocean".to_string());
    names.insert("forest".to_string());
    names.insert("plains".to_string());

    // Valid terrain should produce no diagnostic
    let result = check_province_terrain_csv("ocean", &names, 0, 0, 5);
    assert!(
        result.is_none(),
        "Valid terrain 'ocean' should not produce a diagnostic"
    );

    let result = check_province_terrain_csv("forest", &names, 1, 20, 6);
    assert!(
        result.is_none(),
        "Valid terrain 'forest' should not produce a diagnostic"
    );

    // Invalid terrain should be caught
    let result = check_province_terrain_csv("oceann", &names, 2, 15, 6);
    assert!(
        result.is_some(),
        "Invalid terrain 'oceann' SHOULD produce a diagnostic"
    );
    let diag = result.unwrap();
    assert_eq!(diag.range.start.line, 2);
    assert_eq!(diag.range.start.character, 15);
    assert!(diag.message.contains("oceann"));
    assert_eq!(
        diag.code,
        Some(NumberOrString::String(
            crate::validation::advanced_validation::UNKNOWN_PROVINCE_TERRAIN.to_string()
        ))
    );

    // Empty terrain should be ignored (no diagnostic)
    let result = check_province_terrain_csv("", &names, 3, 0, 0);
    assert!(
        result.is_none(),
        "Empty terrain should not produce a diagnostic"
    );

    // Whitespace-only terrain should be ignored
    let result = check_province_terrain_csv("  ", &names, 4, 0, 2);
    assert!(
        result.is_none(),
        "Whitespace terrain should not produce a diagnostic"
    );
}

#[test]
fn test_build_dynamic_ideology_keywords_vanilla() {
    let sd = ScannerData::new();
    // Insert vanilla ideologies
    sd.ideologies.insert(
        std::sync::Arc::from("democratic"),
        crate::data::layered_value::LayeredValue::new(crate::scanner::ideology_scanner::Ideology {
            name: "democratic".to_string(),
            sub_ideologies: vec![],
            sub_ideology_ranges: HashMap::new(),
            path: std::sync::Arc::from("test"),
            range: ast::Range {
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 0,
            },
        }),
    );
    sd.ideologies.insert(
        std::sync::Arc::from("communism"),
        crate::data::layered_value::LayeredValue::new(crate::scanner::ideology_scanner::Ideology {
            name: "communism".to_string(),
            sub_ideologies: vec![],
            sub_ideology_ranges: HashMap::new(),
            path: std::sync::Arc::from("test"),
            range: ast::Range {
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 0,
            },
        }),
    );

    let keywords = build_dynamic_ideology_keywords(&sd);
    // 2 ideologies × 5 suffixes = 10 keywords
    assert_eq!(keywords.len(), 10);

    // Check all expected suffixes for democratic
    for suffix in IDEOLOGY_MODIFIER_SUFFIXES {
        assert!(
            keywords.contains(&format!("democratic_{}", suffix)),
            "Missing democratic_{}",
            suffix
        );
    }
    // Check all expected suffixes for communism
    for suffix in IDEOLOGY_MODIFIER_SUFFIXES {
        assert!(
            keywords.contains(&format!("communism_{}", suffix)),
            "Missing communism_{}",
            suffix
        );
    }
}

#[test]
fn test_build_dynamic_ideology_keywords_custom_mod() {
    let sd = ScannerData::new();
    // Custom ideology names from mods
    for name in &["anarchist", "monarchism", "absolutist"] {
        sd.ideologies.insert(
            std::sync::Arc::from(*name),
            crate::data::layered_value::LayeredValue::new(
                crate::scanner::ideology_scanner::Ideology {
                    name: name.to_string(),
                    sub_ideologies: vec![],
                    sub_ideology_ranges: HashMap::new(),
                    path: std::sync::Arc::from("test"),
                    range: ast::Range {
                        start_line: 0,
                        start_col: 0,
                        end_line: 0,
                        end_col: 0,
                    },
                },
            ),
        );
    }

    let keywords = build_dynamic_ideology_keywords(&sd);
    // 3 ideologies × 5 suffixes = 15 keywords
    assert_eq!(keywords.len(), 15);

    // Verify a custom ideology gets the drift suffix
    assert!(
        keywords.contains(&"anarchist_drift".to_string()),
        "anarchist_drift should be generated"
    );
    assert!(
        keywords.contains(&"monarchism_acceptance".to_string()),
        "monarchism_acceptance should be generated"
    );
    assert!(
        keywords.contains(&"absolutist_support".to_string()),
        "absolutist_support should be generated"
    );
}

#[test]
fn test_build_dynamic_ideology_keywords_empty() {
    let sd = ScannerData::new();
    // No ideologies → no keywords
    let keywords = build_dynamic_ideology_keywords(&sd);
    assert!(keywords.is_empty());
}

#[test]
fn test_ideology_modifier_suffixes_complete() {
    // Verify the suffix set matches what vanilla actually uses.
    // These are the 5 modifier suffix patterns that combine with ideology
    // names in HOI4 script files (verified against vanilla game files).
    assert_eq!(IDEOLOGY_MODIFIER_SUFFIXES.len(), 5);
    assert!(IDEOLOGY_MODIFIER_SUFFIXES.contains(&"drift"));
    assert!(IDEOLOGY_MODIFIER_SUFFIXES.contains(&"acceptance"));
    assert!(IDEOLOGY_MODIFIER_SUFFIXES.contains(&"popularity"));
    assert!(IDEOLOGY_MODIFIER_SUFFIXES.contains(&"influence"));
    assert!(IDEOLOGY_MODIFIER_SUFFIXES.contains(&"support"));
}
