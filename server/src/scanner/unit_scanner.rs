#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::path::PathBuf;

/// A sub-unit type defined in common/units/*.txt
///
/// These are the building blocks of division templates: combat battalions
/// (infantry, artillery_brigade, light_armor, etc.) and support companies
/// (engineer, artillery, recon, etc.).
#[derive(Debug, Clone)]
pub struct UnitType {
    /// The unit type key (e.g. "infantry", "engineer", "artillery_brigade")
    pub name: String,
    /// Abbreviation (e.g. "INF", "ENG", "ART")
    pub abbreviation: Option<String>,
    /// Group (e.g. "infantry", "support", "mobile", "combat_support")
    pub group: Option<String>,
    /// Combat width (0 for support companies)
    pub combat_width: f64,
    /// Whether this is a support company (group == "support")
    pub is_support: bool,
    /// Type categories (e.g. "infantry", "motorized", "artillery")
    pub type_categories: Vec<String>,
    /// Modding categories (e.g. "category_front_line", "category_support_battalions")
    pub categories: Vec<String>,
    pub path: InternedStr,
    pub range: ast::Range,
}

/// Scan common/units/ from the given roots (vanilla + mods).
pub fn scan_units<F>(roots: &[PathBuf], filter: &F) -> HashMap<String, UnitType>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut map = HashMap::new();
    for root in roots {
        crate::utils::fs_util::walk_and_parse_files(
            &root.join("common/units"),
            &["txt"],
            filter,
            |path, content| {
                let (script, _) = parser::parse_script(&content);
                extract_unit_types(
                    &script.entries,
                    &script.source,
                    &path.to_string_lossy(),
                    &mut map,
                );
            },
        );
    }
    map
}

/// Scan a pre-filtered list of unit files (used by FileOverlay).
pub fn scan_unit_files<F>(files: &[PathBuf], filter: &F) -> HashMap<String, UnitType>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut map = HashMap::new();
    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        extract_unit_types(
            &script.entries,
            &script.source,
            &path.to_string_lossy(),
            &mut map,
        );
    });
    map
}

/// Extract unit type definitions from parsed AST entries.
///
/// Unit files have the structure:
/// ```hoi4
/// sub_units = {
///     infantry = {
///         abbreviation = "INF"
///         group = infantry
///         combat_width = 2
///         type = { infantry }
///         categories = { category_front_line }
///         ...
///     }
///     ...
/// }
/// ```
pub(crate) fn extract_unit_types(
    entries: &[ast::Entry],
    source: &str,
    file_path: &str,
    map: &mut HashMap<String, UnitType>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let key_lower = ass.key_text(source).to_ascii_lowercase();
            if key_lower == "sub_units" {
                if let ast::Value::Block(inner) = &ass.value.value {
                    for sub_entry in inner {
                        if let ast::Entry::Assignment(sub_ass) = sub_entry {
                            let unit_name = sub_ass.key_text(source).to_string();
                            if let ast::Value::Block(unit_entries) = &sub_ass.value.value {
                                if let Some(unit) = extract_single_unit(
                                    &unit_name,
                                    unit_entries,
                                    source,
                                    file_path,
                                    &sub_ass.key_range,
                                ) {
                                    map.insert(unit_name, unit);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Extract a single unit type definition from its inner entries.
fn extract_single_unit(
    name: &str,
    entries: &[ast::Entry],
    source: &str,
    file_path: &str,
    range: &ast::Range,
) -> Option<UnitType> {
    let mut abbreviation = None;
    let mut group = None;
    let mut combat_width = 0.0;
    let mut type_categories = Vec::new();
    let mut categories = Vec::new();

    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let key_lower = ass.key_text(source).to_ascii_lowercase();
            match key_lower.as_str() {
                "abbreviation" => {
                    if let Some(val) = ass.value.value.as_str(source) {
                        abbreviation = Some(val.to_string());
                    }
                }
                "group" => {
                    if let Some(val) = ass.value.value.as_str(source) {
                        group = Some(val.to_string());
                    }
                }
                "combat_width" => {
                    if let ast::Value::Number(val) = &ass.value.value {
                        combat_width = *val;
                    }
                }
                "type" => {
                    if let ast::Value::Block(type_entries) = &ass.value.value {
                        for te in type_entries {
                            if let ast::Entry::Assignment(t_ass) = te {
                                type_categories.push(t_ass.key_text(source).to_string());
                            }
                        }
                    }
                }
                "categories" => {
                    if let ast::Value::Block(cat_entries) = &ass.value.value {
                        for ce in cat_entries {
                            if let ast::Entry::Assignment(c_ass) = ce {
                                categories.push(c_ass.key_text(source).to_string());
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let is_support = group.as_deref() == Some("support");

    Some(UnitType {
        name: name.to_string(),
        abbreviation,
        group,
        combat_width,
        is_support,
        type_categories,
        categories,
        path: std::sync::Arc::from(file_path.to_string()),
        range: range.clone(),
    })
}
