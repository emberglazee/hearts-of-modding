//! Tests for the province scanner (definition.csv parsing).
//!
//! The CSV schema is ID;R;G;B;Type(land/sea/lake);Coastal;Terrain;Continent —
//! these tests pin the column mapping so hover/validation read real values.

use crate::scanner::province_scanner::scan_province_files;
use std::path::PathBuf;

fn no_filter(_: &std::path::Path) -> bool {
    false
}

/// Column 5 is the province TYPE (land/sea/lake), column 7 is the TERRAIN
/// category (plains/forest/ocean/...). A regression here inverts every hover
/// tooltip ("Terrain: land", "Type: plains") and feeds the wrong field to
/// terrain cross-validation.
#[test]
fn test_definition_csv_columns_map_to_correct_fields() {
    // Vanilla-shaped line: id=2, land province, coastal=false, forest terrain.
    let dir = std::env::temp_dir().join(format!("hom_prov_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("definition.csv");
    std::fs::write(
        &file,
        "0;0;0;0;land;false;unknown;0\n2;0;0;55;land;false;forest;1\n4;0;0;232;sea;true;ocean;0\n",
    )
    .unwrap();

    let provinces = scan_province_files(&[PathBuf::from(&file)], &no_filter);

    let p2 = provinces.get(&2).expect("province 2 parsed");
    assert_eq!(p2.prov_type, "land", "column 5 is the province type");
    assert_eq!(p2.terrain, "forest", "column 7 is the terrain category");
    assert!(!p2.is_coastal);
    assert_eq!(p2.continent, 1);

    let p4 = provinces.get(&4).expect("province 4 parsed");
    assert_eq!(p4.prov_type, "sea");
    assert_eq!(p4.terrain, "ocean");
    assert!(p4.is_coastal);

    let _ = std::fs::remove_dir_all(&dir);
}
