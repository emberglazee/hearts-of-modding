//! Tests for unit file (`common/units/*.txt`) integration:
//! initial scope mapping and EntityLookup resolution of UnitType entities.

use crate::data::entity_lookup::{EntityKind, EntityLookup};
use crate::data::interner::InternedStr;
use crate::data::layered_value::LayeredValue;
use crate::data::scanner_data::ScannerData;
use crate::scope::scope::{Scope, initial_scope_for_uri};

fn dummy_range() -> crate::parser::ast::Range {
    crate::parser::ast::Range {
        start_line: 0,
        start_col: 0,
        end_line: 0,
        end_col: 0,
    }
}

/// `/common/units/` files get Global as initial scope: the top level of a
/// unit file is the `sub_units = { ... }` declaration wrapper — pure
/// definition structure with no trigger/effect evaluation at that level.
/// (Sub-scope promotion for per-unit blocks, if ever needed, belongs in the
/// walker, mirroring how ideas promote via IDEA_STRUCTURE_KEYS.)
#[test]
fn test_initial_scope_for_unit_files_is_global() {
    assert_eq!(
        initial_scope_for_uri("/mod/common/units/infantry.txt"),
        Scope::Global
    );
    assert_eq!(
        initial_scope_for_uri("/mod/common/units/ships/BB.txt"),
        Scope::Global
    );
}

/// UnitType entities resolve through EntityLookup::find_definition —
/// the Go-to-definition path. Mirrors the Achievement integration test.
#[test]
fn test_entity_lookup_finds_unit_type() {
    let data = Box::leak(Box::new(ScannerData::new()));
    data.unit_types.insert(
        InternedStr::from("infantry"),
        LayeredValue::new(crate::scanner::unit_scanner::UnitType {
            name: "infantry".to_string(),
            abbreviation: Some("INF".to_string()),
            group: Some("infantry".to_string()),
            combat_width: 2.0,
            is_support: false,
            type_categories: vec!["infantry".to_string()],
            categories: vec!["category_front_line".to_string()],
            path: InternedStr::from("/mod/common/units/infantry.txt"),
            range: dummy_range(),
        }),
    );

    let lookup = EntityLookup::new(data);
    let defs = lookup.find_definition("infantry");
    assert_eq!(
        defs.len(),
        1,
        "EntityLookup should find unit type 'infantry'"
    );
    assert_eq!(defs[0].kind, EntityKind::UnitType);
    assert_eq!(
        defs[0].path.to_string(),
        "/mod/common/units/infantry.txt",
        "definition should point at the declaring unit file"
    );

    // entity_names must include it (completion/semantic-token path)
    let names = lookup.entity_names();
    assert_eq!(
        names.get("infantry"),
        Some(&EntityKind::UnitType),
        "entity_names should map 'infantry' to EntityKind::UnitType"
    );
}
