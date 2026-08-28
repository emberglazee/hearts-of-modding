use crate::data::entity_lookup::EntityLookup;
use crate::data::interner::InternedStr;
use crate::data::layered_value::LayeredValue;
use crate::data::scanner_data::ScannerData;
use crate::parser::ast;

/// Helper to construct a dummy Range for testing.
fn dummy_range() -> ast::Range {
    ast::Range {
        start_line: 0,
        start_col: 0,
        end_line: 0,
        end_col: 0,
    }
}

/// Verifies that standard scanner DashMaps (generated via registry)
/// are accessible and that EntityLookup can find entities in them.
/// This tests the macro-generated field names match the dispatch.
#[test]
fn test_standard_scanner_entitylookup_integration() {
    let data = ScannerData::new();

    // Insert into a standard scanner DashMap (Achievement)
    data.achievements.insert(
        InternedStr::from("test_achievement"),
        LayeredValue::new(crate::scanner::achievement_scanner::Achievement {
            name: "test_achievement".to_string(),
            is_ribbon: false,
            path: InternedStr::from("test/path.txt"),
            range: dummy_range(),
        }),
    );

    // EntityLookup should find it via find_definition
    let lookup = EntityLookup::new(&data);
    let defs = lookup.find_definition("test_achievement");
    assert_eq!(defs.len(), 1, "EntityLookup should find test_achievement");
    assert_eq!(
        defs[0].kind,
        crate::data::entity_lookup::EntityKind::Achievement,
    );

    // entity_names should include it
    let names = lookup.entity_names();
    assert!(names.contains_key("test_achievement"));
    assert_eq!(
        names.get("test_achievement").unwrap(),
        &crate::data::entity_lookup::EntityKind::Achievement,
    );
}

/// Verify that standard scanner DashMaps are properly initialized in
/// ScannerData::new() and accept insertions.
#[test]
fn test_standard_scanner_maps_insertable() {
    let data = ScannerData::new();

    // Achievement
    {
        let key: InternedStr = InternedStr::from("ach_1");
        data.achievements.insert(
            key.clone(),
            LayeredValue::new(crate::scanner::achievement_scanner::Achievement {
                name: "ach_1".to_string(),
                is_ribbon: false,
                path: InternedStr::from(""),
                range: dummy_range(),
            }),
        );
        assert!(data.achievements.contains_key(&key));
    }

    // Building — has `name`, `max_level`
    {
        let key: InternedStr = InternedStr::from("bld_1");
        data.buildings.insert(
            key.clone(),
            LayeredValue::new(crate::scanner::building_scanner::Building {
                name: "bld_1".to_string(),
                max_level: Some(5),
                path: InternedStr::from(""),
                range: dummy_range(),
            }),
        );
        assert!(data.buildings.contains_key(&key));
    }

    // Ability — more complex struct
    {
        let key: InternedStr = InternedStr::from("ab_1");
        data.abilities.insert(
            key.clone(),
            LayeredValue::new(crate::scanner::ability_scanner::Ability {
                key: "ab_1".to_string(),
                name_loc: None,
                desc_loc: None,
                cost: None,
                duration: None,
                sound_effect: None,
                type_name: None,
                cancelable: None,
                cooldown: None,
                icon: None,
                has_allowed: false,
                has_one_time_effect: false,
                has_unit_modifiers: false,
                has_ai_will_do: false,
                path: InternedStr::from(""),
                range: dummy_range(),
            }),
        );
        assert!(data.abilities.contains_key(&key));
    }

    // UnitType — common/units/*.txt (sub_units)
    {
        let key: InternedStr = InternedStr::from("infantry");
        data.unit_types.insert(
            key.clone(),
            LayeredValue::new(crate::scanner::unit_scanner::UnitType {
                name: "infantry".to_string(),
                abbreviation: Some("INF".to_string()),
                group: Some("infantry".to_string()),
                combat_width: 2.0,
                is_support: false,
                type_categories: vec!["infantry".to_string()],
                categories: vec!["category_front_line".to_string()],
                path: InternedStr::from(""),
                range: dummy_range(),
            }),
        );
        assert!(data.unit_types.contains_key(&key));
    }
}

/// Verify for_each_standard_scanner! has the expected number of entries.
/// Update this count when adding a new scanner to registry.rs.
#[test]
fn test_standard_scanner_count() {
    let mut count = 0usize;

    macro_rules! counter {
        ($mod:ident, $ty:ident, $kind:ident, $field:ident, $dir:expr, $ext:expr) => {
            count += 1;
        };
    }

    crate::for_each_standard_scanner!(counter);

    assert_eq!(
        count, 19,
        "Number of standard scanners in registry. \
         Update this count when adding entries to registry.rs"
    );
}

/// Verify the registry's scanner directories match the engine's real paths.
/// Regression guard: balance-of-power files live at `common/bop` (there is NO
/// `common/balance_of_power` dir) and portrait pools at top-level `portraits/`
/// (there is NO `gfx/portraits` dir). The orchestrator feeds these dir strings
/// to the FileOverlay, so a wrong dir silently empties the whole DashMap.
#[test]
fn test_standard_scanner_dir_paths() {
    macro_rules! dir_check {
        ($mod:ident, $ty:ident, $kind:ident, $field:ident, $dir:expr, $ext:expr) => {
            match stringify!($mod) {
                "bop_scanner" => assert_eq!(
                    $dir, "common/bop",
                    "balance of power files live at common/bop, not common/balance_of_power"
                ),
                "portrait_scanner" => assert_eq!(
                    $dir, "portraits",
                    "portrait pools live at top-level portraits/, not gfx/portraits"
                ),
                _ => {}
            }
        };
    }
    crate::for_each_standard_scanner!(dir_check);
}

/// End-to-end: the FileOverlay must index `common/bop/*.txt` and `portraits/*.txt`
/// under their engine paths, and the scanners must extract from the winning files.
/// Guards the orchestrator's `winning_files_in()` prefixes — the old wrong
/// prefixes (`common/balance_of_power`, `gfx/portraits`) must stay empty.
#[test]
fn test_bop_and_portraits_overlay_paths() {
    use crate::scanner::file_overlay::FileOverlay;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp = std::env::temp_dir().join(format!("hom_path_test_{}_{}", std::process::id(), id));
    let root = tmp.join("mod");
    fs::create_dir_all(root.join("common/bop")).unwrap();
    fs::create_dir_all(root.join("portraits")).unwrap();

    fs::write(
        root.join("common/bop/test_power.txt"),
        "test_power = {\n\
         \tinitial_value = 50\n\
         \tleft_side = test_left\n\
         \tright_side = test_right\n\
         \trange = { id = r0 min = -1 max = 1 }\n\
         }\n",
    )
    .unwrap();
    fs::write(
        root.join("portraits/test_pool.txt"),
        "ENG = {\n\tarmy = { }\n}\n",
    )
    .unwrap();

    let roots = vec![root.clone()];
    let filter = |_: &std::path::Path| false;
    let overlay = FileOverlay::build_script_only(&roots, &["txt"], filter, &[]);

    // Engine paths must be indexed; the old wrong prefixes must stay empty.
    let bop_files = overlay.winning_files_in("common/bop");
    assert_eq!(bop_files.len(), 1, "overlay must index common/bop/*.txt");
    assert!(
        overlay
            .winning_files_in("common/balance_of_power")
            .is_empty(),
        "common/balance_of_power is not an engine path"
    );

    let portrait_files = overlay.winning_files_in("portraits");
    assert_eq!(
        portrait_files.len(),
        1,
        "overlay must index portraits/*.txt"
    );
    assert!(
        overlay.winning_files_in("gfx/portraits").is_empty(),
        "gfx/portraits is not an engine path"
    );

    // Extraction from the winning files must actually produce entities.
    let bops = crate::scanner::bop_scanner::scan_balance_of_power_files(&bop_files, &filter);
    assert!(
        bops.contains_key("test_power"),
        "bop extractor should parse the file"
    );
    assert_eq!(bops["test_power"].initial_value, Some(50.0));

    let portraits = crate::scanner::portrait_scanner::scan_portrait_files(&portrait_files, &filter);
    assert!(
        portraits.contains_key("ENG"),
        "portrait extractor should parse the pool file"
    );

    let _ = fs::remove_dir_all(&tmp);
}

/// find_definition resolves localization via exact key match only (the
/// `{key}:`-prefixed scan was provably dead: stored loc keys never contain
/// `:`). Guards the loc branch against resurrecting an O(N) scan.
#[test]
fn test_find_definition_localization_exact_match() {
    let data = ScannerData::new();
    data.localization.insert(
        InternedStr::from("test_loc"),
        LayeredValue::new(crate::parser::loc_parser::LocEntry {
            key: InternedStr::from("test_loc"),
            value: "Value".to_string(),
            range: dummy_range(),
            path: InternedStr::from("test.yml"),
            value_start_col: 0,
            version: None,
            version_range: None,
        }),
    );

    let lookup = EntityLookup::new(&data);
    let defs = lookup.find_definition("test_loc");
    assert_eq!(
        defs.len(),
        1,
        "exact loc key must resolve to exactly one location"
    );
    assert_eq!(
        defs[0].kind,
        crate::data::entity_lookup::EntityKind::Localization,
    );

    // Non-loc key → zero results without any scan.
    assert!(lookup.find_definition("not_a_loc_key").is_empty());
}

/// entity_at must resolve variables through the reverse per-path index — the
/// last entity type without one. The old code iterated EVERY workspace
/// variable per entity_at call (prepare_rename, find_symbol_at_position).
#[test]
fn test_entity_at_variables_via_file_index() {
    let data = ScannerData::new();
    data.variables.insert(
        InternedStr::from("my_var"),
        vec![crate::scanner::variable_scanner::Variable {
            name: "my_var".to_string(),
            path: InternedStr::from("test.txt"),
            range: ast::Range {
                start_line: 1,
                start_col: 1,
                end_line: 1,
                end_col: 10,
            },
        }],
    );
    data.variables_file_index.insert(
        InternedStr::from("test.txt"),
        vec![InternedStr::from("my_var")],
    );

    let lookup = EntityLookup::new(&data);
    let pos = tower_lsp_server::ls_types::Position {
        line: 1,
        character: 5,
    };
    let result = lookup.entity_at("test.txt", "\n0123456789\n", pos);
    assert!(result.is_some(), "variable under the index must resolve");
    let (kind, _, name_out) = result.unwrap();
    assert_eq!(kind, crate::data::entity_lookup::EntityKind::Variable);
    assert_eq!(name_out, "my_var");

    // A path with no index entry resolves nothing (no workspace-wide scan).
    assert!(
        lookup
            .entity_at("other.txt", "\n0123456789\n", pos)
            .is_none()
    );
}

/// rebuild_all_file_indices must include variables (Vec-valued — rebuilt
/// manually, not via rebuild_index!), so incremental O(K) lookups work right
/// after the full scan.
#[test]
fn test_rebuild_all_file_indices_includes_variables() {
    let data = ScannerData::new();
    data.variables.insert(
        InternedStr::from("my_var"),
        vec![crate::scanner::variable_scanner::Variable {
            name: "my_var".to_string(),
            path: InternedStr::from("test.txt"),
            range: ast::Range {
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 5,
            },
        }],
    );

    data.rebuild_all_file_indices();
    let names = data.variables_file_index.get("test.txt").unwrap();
    assert_eq!(names.value().len(), 1);
    assert_eq!(names.value()[0].as_ref(), "my_var");
}

/// Verify EntityLookup::entity_at works with standard scanners.
#[test]
fn test_standard_scanner_entity_at() {
    use crate::utils::lsp_convert::is_pos_in_range;

    let data = ScannerData::new();

    // Insert an achievement at a known position
    data.achievements.insert(
        InternedStr::from("test"),
        LayeredValue::new(crate::scanner::achievement_scanner::Achievement {
            name: "test".to_string(),
            is_ribbon: false,
            path: InternedStr::from("test.txt"),
            range: ast::Range {
                start_line: 1,
                start_col: 1,
                end_line: 1,
                end_col: 10,
            },
        }),
    );
    // The reverse per-path index `entity_at` now relies on (path -> names).
    data.achievements_file_index.insert(
        InternedStr::from("test.txt"),
        vec![InternedStr::from("test")],
    );

    let lookup = EntityLookup::new(&data);
    let pos = tower_lsp_server::ls_types::Position {
        line: 1,
        character: 5,
    };

    let result = lookup.entity_at("test.txt", "\n0123456789\n", pos);
    assert!(result.is_some(), "entity_at should find the test entity");
    let (kind, range, name) = result.unwrap();
    assert_eq!(kind, crate::data::entity_lookup::EntityKind::Achievement);
    assert_eq!(&name, "test");
    assert!(is_pos_in_range(pos, &range));
}

/// REGRESSION: four registries were populated by the full scan but had NO
/// incremental path at all — no `classify_file` arm, no `update_*`, no
/// `remove_path` arm. Editing `common/country_tag_aliases/`,
/// `common/resources/`, `common/state_category/` or `map/continent.txt` was
/// silently ignored until the whole workspace was re-scanned, and deleting
/// such a file left its entries behind forever.
///
/// `tag_aliases` is the worst of the four: `rules/country_tags.rs` consults it
/// live, so a stale entry means a wrong diagnostic on the user's screen.
///
/// This walks the full lifecycle (add -> edit -> delete) for each registry.
#[test]
fn test_incremental_updates_for_late_wired_registries() {
    use crate::parser::parser::parse_script;
    use crate::scanner::incremental_scanner::{
        remove_path_from_scanner_data, update_scanner_data_from_ast,
    };

    struct Case {
        name: &'static str,
        path: &'static str,
        initial: &'static str,
        edited: &'static str,
        first_key: &'static str,
        second_key: &'static str,
        count: fn(&ScannerData) -> usize,
        has: fn(&ScannerData, &str) -> bool,
    }

    let cases = [
        Case {
            name: "tag_aliases",
            path: "/mod/common/country_tag_aliases/00_aliases.txt",
            initial: "AAA = { original_tag = GER }\n",
            edited: "BBB = { original_tag = ITA }\n",
            first_key: "AAA",
            second_key: "BBB",
            count: |d| d.tag_aliases.len(),
            has: |d, k| d.tag_aliases.contains_key(k),
        },
        Case {
            name: "resources",
            path: "/mod/common/resources/00_resources.txt",
            initial: "resources = {\n    unobtainium = { icon_frame = 1 }\n}\n",
            edited: "resources = {\n    handwavium = { icon_frame = 2 }\n}\n",
            first_key: "unobtainium",
            second_key: "handwavium",
            count: |d| d.resources.len(),
            has: |d, k| d.resources.contains_key(k),
        },
        Case {
            name: "state_categories",
            path: "/mod/common/state_category/00_state_categories.txt",
            initial: "state_categories = {\n    tiny_hamlet = { local_building_slots = 1 }\n}\n",
            edited: "state_categories = {\n    huge_sprawl = { local_building_slots = 9 }\n}\n",
            first_key: "tiny_hamlet",
            second_key: "huge_sprawl",
            count: |d| d.state_categories.len(),
            has: |d, k| d.state_categories.contains_key(k),
        },
        Case {
            name: "continents",
            path: "/mod/map/continent.txt",
            initial: "continents = {\n    atlantis\n}\n",
            edited: "continents = {\n    lemuria\n}\n",
            first_key: "atlantis",
            second_key: "lemuria",
            count: |d| d.continents.len(),
            has: |d, k| d.continents.contains_key(k),
        },
    ];

    for c in cases {
        let data = ScannerData::new();

        // ── add ──
        let (script, _) = parse_script(c.initial);
        update_scanner_data_from_ast(&data, c.path, &script);
        assert!(
            (c.has)(&data, c.first_key),
            "{}: `{}` should be registered after an incremental add",
            c.name,
            c.first_key
        );

        // ── edit: the old key must go, the new one must appear ──
        let (script2, _) = parse_script(c.edited);
        update_scanner_data_from_ast(&data, c.path, &script2);
        assert!(
            (c.has)(&data, c.second_key),
            "{}: `{}` missing after edit",
            c.name,
            c.second_key
        );
        assert!(
            !(c.has)(&data, c.first_key),
            "{}: stale `{}` survived the edit",
            c.name,
            c.first_key
        );
        assert_eq!(
            (c.count)(&data),
            1,
            "{}: exactly one entry after edit",
            c.name
        );

        // ── delete ──
        remove_path_from_scanner_data(&data, c.path);
        assert_eq!(
            (c.count)(&data),
            0,
            "{}: entries survived deletion of their file",
            c.name
        );
    }
}
