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
        count, 17,
        "Number of standard scanners in registry. \
         Update this count when adding entries to registry.rs"
    );
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

    let lookup = EntityLookup::new(&data);
    let pos = tower_lsp_server::ls_types::Position {
        line: 1,
        character: 5,
    };

    let result = lookup.entity_at("test.txt", pos);
    assert!(result.is_some(), "entity_at should find the test entity");
    let (kind, range, name) = result.unwrap();
    assert_eq!(kind, crate::data::entity_lookup::EntityKind::Achievement);
    assert_eq!(&name, "test");
    assert!(is_pos_in_range(pos, &range));
}
