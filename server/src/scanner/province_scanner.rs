#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::parser::ast;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Province {
    pub id: u32,
    pub rgb: (u8, u8, u8),
    pub terrain: String,
    pub is_coastal: bool,
    pub prov_type: String, // land, sea, lake
    pub continent: u32,
    pub path: InternedStr,
    pub range: ast::Range,
}

/// Parse one definition.csv line, recording the byte range of the whole line so
/// goto-definition can jump the user straight to it.
fn insert_province(
    id: u32,
    line_idx: usize,
    line: &str,
    parts: &[&str],
    path: &std::path::Path,
    provinces: &mut HashMap<u32, Province>,
) {
    if parts.len() >= 8 {
        let r = parts[1].parse::<u8>().unwrap_or(0);
        let g = parts[2].parse::<u8>().unwrap_or(0);
        let b = parts[3].parse::<u8>().unwrap_or(0);
        // definition.csv schema: ID;R;G;B;Type;Coastal;Terrain;Continent —
        // column 5 is the province TYPE (land/sea/lake), column 7 the TERRAIN
        // category (plains/forest/ocean/...). Verified against vanilla
        // map/definition.csv (`0;0;0;0;land;false;unknown;0`).
        let prov_type = parts[4].to_string();
        let is_coastal = parts[5].eq_ignore_ascii_case("true");
        let terrain = parts[6].to_string();
        let continent = parts[7].parse::<u32>().unwrap_or(0);
        provinces.insert(
            id,
            Province {
                id,
                rgb: (r, g, b),
                terrain,
                is_coastal,
                prov_type,
                continent,
                path: InternedStr::from(path.to_string_lossy().as_ref()),
                range: ast::Range {
                    start_line: line_idx as u32,
                    start_col: 0,
                    end_line: line_idx as u32,
                    end_col: line.len() as u32,
                },
            },
        );
    } else {
        provinces.insert(
            id,
            Province {
                id,
                rgb: (0, 0, 0),
                terrain: String::new(),
                is_coastal: false,
                prov_type: String::new(),
                continent: 0,
                path: InternedStr::from(path.to_string_lossy().as_ref()),
                range: ast::Range {
                    start_line: line_idx as u32,
                    start_col: 0,
                    end_line: line_idx as u32,
                    end_col: line.len() as u32,
                },
            },
        );
    }
}

pub fn scan_provinces<F>(roots: &[PathBuf], filter: &F) -> HashMap<u32, Province>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut provinces = HashMap::new();

    for root in roots {
        let map_config = crate::utils::map_config::get_map_config(root);
        let definition_path = root.join(format!("map/{}", map_config.definitions));
        if definition_path.exists()
            && !filter(&definition_path)
            && let Ok(content) = fs::read_to_string(&definition_path)
        {
            for (line_idx, line) in content.lines().enumerate() {
                // HOI4 definition.csv format: ID;R;G;B;Terrain;IsCoastal;ProvinceType;Continent
                let parts: Vec<&str> = line.split(';').collect();
                if parts.len() >= 8
                    && let Ok(id) = parts[0].parse::<u32>()
                {
                    insert_province(id, line_idx, line, &parts, &definition_path, &mut provinces);
                } else if let Some(id_str) = parts.first() {
                    if let Ok(id) = id_str.parse::<u32>() {
                        insert_province(
                            id,
                            line_idx,
                            line,
                            &parts,
                            &definition_path,
                            &mut provinces,
                        );
                    }
                }
            }
        }
    }

    provinces
}

pub fn scan_province_files<F>(files: &[PathBuf], filter: &F) -> HashMap<u32, Province>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut provinces = HashMap::new();

    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let parsed = parse_definition_csv(&content, path);
        provinces.extend(parsed);
    });

    provinces
}

/// IDs in `0..=max(defined)` with no `definition.csv` row, lowest first.
///
/// A gap shifts every later province's properties onto its successor (the
/// engine indexes rows positionally), so offering exactly the gaps — and
/// nothing when there are none — is the only ID completion this file needs.
/// `limit` caps the list so a pathological file can't flood the client.
pub(crate) fn missing_province_ids(
    defined: &std::collections::HashSet<u32>,
    limit: usize,
) -> Vec<u32> {
    let Some(&max) = defined.iter().max() else {
        return Vec::new();
    };
    let mut missing = Vec::new();
    for id in 0..=max {
        if missing.len() >= limit {
            break;
        }
        if !defined.contains(&id) {
            missing.push(id);
        }
    }
    missing
}

/// Parse definition.csv CONTENT into provinces. Shared by the startup scan
/// (`scan_province_files`) and the incremental updater so an edit+save lands
/// in exactly the state a fresh scan would produce. Line-based by design:
/// csv rows are not HOI4 script and must never go through `parse_script`.
pub(crate) fn parse_definition_csv(
    content: &str,
    path: &std::path::Path,
) -> HashMap<u32, Province> {
    let mut provinces = HashMap::new();
    for (line_idx, line) in content.lines().enumerate() {
        let parts: Vec<&str> = line.split(';').collect();
        if parts.len() >= 8
            && let Ok(id) = parts[0].parse::<u32>()
        {
            insert_province(id, line_idx, line, &parts, path, &mut provinces);
        } else if let Some(id_str) = parts.first() {
            if let Ok(id) = id_str.parse::<u32>() {
                insert_province(id, line_idx, line, &parts, path, &mut provinces);
            }
        }
    }
    provinces
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn set(ids: &[u32]) -> HashSet<u32> {
        ids.iter().copied().collect()
    }

    #[test]
    fn test_missing_ids_reports_gaps_lowest_first() {
        assert_eq!(
            missing_province_ids(&set(&[0, 1, 3, 4, 7]), 100),
            vec![2, 5, 6]
        );
    }

    #[test]
    fn test_missing_ids_empty_when_contiguous_or_no_data() {
        assert!(missing_province_ids(&set(&[0, 1, 2]), 100).is_empty());
        assert!(missing_province_ids(&HashSet::new(), 100).is_empty());
    }

    #[test]
    fn test_missing_ids_respects_limit() {
        // Gaps 1..=100 inside {0, 101}: limit 5 yields exactly the first five.
        assert_eq!(
            missing_province_ids(&set(&[0, 101]), 5),
            vec![1, 2, 3, 4, 5]
        );
    }
}
