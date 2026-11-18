use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::path::PathBuf;

/// An equipment definition from `common/units/equipment/*.txt`.
///
/// Equipment files hold one `equipments = { ... }` block, and each child is
/// either an archetype (`is_archetype = yes`) or a variant that points at one
/// with `archetype = <name>`. Variants also chain to their predecessor through
/// `parent = <name>`, which is the upgrade line the research tree walks.
///
/// `type` is the unit category the equipment belongs to (`armor`, `infantry`,
/// `fighter`, `train`, `convoy`, ...). It is a separate namespace from the
/// archetype names: `equipment_bonus` blocks accept either, so both are indexed.
#[derive(Debug, Clone)]
/// Scanner-populated equipment definition. Every field has a reader.
pub struct Equipment {
    /// Definition key (`infantry_equipment`, `light_tank_chassis_1`, ...)
    pub name: String,
    /// `is_archetype = yes`
    pub is_archetype: bool,
    /// `archetype = <name>`: the archetype a variant belongs to
    pub archetype: Option<String>,
    /// `parent = <name>`: the previous variant in the upgrade chain
    pub parent: Option<String>,
    /// `type = <value>`: unit category (`armor`, `infantry`, `fighter`, ...)
    pub group_type: Option<String>,
    /// `year = <value>`
    pub year: Option<u32>,
    pub path: InternedStr,
    pub range: ast::Range,
}

/// Scan a pre-filtered list of equipment files (used by FileOverlay).
pub fn scan_equipment_files<F>(files: &[PathBuf], filter: &F) -> HashMap<String, Equipment>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut map = HashMap::new();
    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        find_equipments_in_entries(
            &script.entries,
            &script.source,
            &path.to_string_lossy(),
            &mut map,
        );
    });
    map
}

/// Extract equipment definitions from parsed AST entries.
///
/// ```hoi4
/// equipments = {
///     infantry_equipment = {
///         year = 1936
///         is_archetype = yes
///         type = infantry
///         build_cost_ic = 0.43
///     }
///     infantry_equipment_1 = {
///         year = 1939
///         archetype = infantry_equipment
///         parent = infantry_equipment
///     }
/// }
/// ```
///
/// `equipment_modules = { ... }`, `duplicate_archetypes = { ... }` and
/// `search_filters = { ... }` live in the same files and are deliberately not
/// read: they are a different namespace and are not equipment definitions.
pub(crate) fn find_equipments_in_entries(
    entries: &[ast::Entry],
    source: &str,
    file_path: &str,
    map: &mut HashMap<String, Equipment>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            if !ass.key_text(source).eq_ignore_ascii_case("equipments") {
                continue;
            }
            let ast::Value::Block(inner) = &ass.value.value else {
                continue;
            };
            for sub_entry in inner {
                if let ast::Entry::Assignment(sub_ass) = sub_entry {
                    let name = sub_ass.key_text(source).to_string();
                    let ast::Value::Block(body) = &sub_ass.value.value else {
                        continue;
                    };
                    map.insert(
                        name.clone(),
                        extract_single_equipment(
                            &name,
                            body,
                            source,
                            file_path,
                            &sub_ass.key_range,
                        ),
                    );
                }
            }
        }
    }
}

/// Read a yes/no field. The parser turns `yes`/`no` into `Value::Boolean`,
/// but a quoted `"yes"` still arrives as a string, so accept both.
fn truthy(value: &ast::Value, source: &str) -> Option<bool> {
    match value {
        ast::Value::Boolean(b) => Some(*b),
        other => other.as_str(source).map(|s| s.eq_ignore_ascii_case("yes")),
    }
}

fn extract_single_equipment(
    name: &str,
    entries: &[ast::Entry],
    source: &str,
    file_path: &str,
    range: &ast::Range,
) -> Equipment {
    let mut is_archetype = false;
    let mut archetype = None;
    let mut parent = None;
    let mut group_type = None;
    let mut year = None;

    for entry in entries {
        let ast::Entry::Assignment(ass) = entry else {
            continue;
        };
        match ass.key_text(source).to_ascii_lowercase().as_str() {
            "is_archetype" => {
                is_archetype = truthy(&ass.value.value, source).unwrap_or(false);
            }
            "archetype" => {
                archetype = ass.value.value.as_str(source).map(str::to_string);
            }
            "parent" => {
                parent = ass.value.value.as_str(source).map(str::to_string);
            }
            "type" => {
                group_type = ass.value.value.as_str(source).map(str::to_string);
            }
            "year" => {
                if let ast::Value::Number(n) = &ass.value.value {
                    year = Some(*n as u32);
                }
            }
            _ => {}
        }
    }

    Equipment {
        name: name.to_string(),
        is_archetype,
        archetype,
        parent,
        group_type,
        year,
        path: std::sync::Arc::from(file_path.to_string()),
        range: range.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> HashMap<String, Equipment> {
        let (script, _) = parser::parse_script(src);
        let mut map = HashMap::new();
        find_equipments_in_entries(&script.entries, &script.source, "/mod/x.txt", &mut map);
        map
    }

    #[test]
    fn reads_archetypes_and_variants() {
        let map = parse(
            r#"equipments = {
                infantry_equipment = {
                    year = 1936
                    is_archetype = yes
                    type = infantry
                    build_cost_ic = 0.43
                }
                infantry_equipment_1 = {
                    year = 1939
                    archetype = infantry_equipment
                    parent = infantry_equipment
                    build_cost_ic = 0.5
                }
            }"#,
        );
        assert_eq!(map.len(), 2);
        let arch = map.get("infantry_equipment").unwrap();
        assert!(arch.is_archetype);
        assert_eq!(arch.group_type.as_deref(), Some("infantry"));
        assert_eq!(arch.year, Some(1936));
        let variant = map.get("infantry_equipment_1").unwrap();
        assert!(!variant.is_archetype);
        assert_eq!(variant.archetype.as_deref(), Some("infantry_equipment"));
        assert_eq!(variant.parent.as_deref(), Some("infantry_equipment"));
    }

    #[test]
    fn ignores_sibling_blocks_in_the_same_file() {
        let map = parse(
            r#"equipment_modules = {
                tank_gun = { category = gun }
            }
            duplicate_archetypes = {
                tank_destroyer = { light_tank_destroyer_chassis medium_tank_destroyer_chassis }
            }
            equipments = {
                convoy = { is_archetype = yes type = convoy }
            }"#,
        );
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("convoy"));
    }
}
