use crate::data::interner::InternedStr;
use crate::data::layered_value::LayeredValue;
use crate::parser::ast;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::ScopeStack;
use crate::utils::lsp_convert::ast_range_to_lsp;
use dashmap::DashMap;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates state definition files (`history/states/*.txt`):
///
/// - `state_category = X` → warns if X is not a known state category
/// - `resources = { X = N }` → warns if X is not a known resource type
/// - `buildings = { X = N }` → warns if X is not a known building type
///
/// Assumes definitions are scanned from `common/state_category/`,
/// `common/resources/`, and `common/buildings/`.
pub(crate) struct StateDefinitionRule;

impl ValidationRule for StateDefinitionRule {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key_lower = ass.key.to_ascii_lowercase();

        // state_category = <value> — validate value is known
        if key_lower == "state_category" {
            if let Some(value_str) = extract_string_value(&ass.value) {
                if !ctx.state_categories.is_empty()
                    && !ctx.state_categories.contains_key(value_str.as_str())
                {
                    let known = format_known_list(ctx.state_categories);
                    diags.push(Diagnostic {
                        range: ast_range_to_lsp(&ass.value.range),
                        severity: Some(DiagnosticSeverity::WARNING),
                        message: format!(
                            "Unknown state category '{}'{}",
                            value_str,
                            if known.is_empty() {
                                String::new()
                            } else {
                                format!(". Known: {}", known)
                            },
                        ),
                        code: Some(NumberOrString::String(
                            crate::validation::advanced_validation::UNKNOWN_STATE_CATEGORY
                                .to_string(),
                        )),
                        source: Some("Hearts of Modding".to_string()),
                        ..Default::default()
                    });
                }
            }
            return;
        }

        // resources = { <resource> = <amount> } — validate resource names
        if key_lower == "resources" {
            if let ast::Value::Block(resource_entries) = &ass.value.value {
                validate_keys_in_dashmap(
                    resource_entries,
                    ctx.resources,
                    "resource",
                    "common/resources/*.txt",
                    crate::validation::advanced_validation::UNKNOWN_RESOURCE,
                    diags,
                );
            }
            return;
        }

        // buildings = { <building> = <level> } — validate building names
        // Numeric keys are province IDs for province-level building placements
        // (e.g., 2671 = { naval_base = 2 }), not building type names.
        // Recurse into province-level blocks to validate their building names too.
        if key_lower == "buildings" {
            if let ast::Value::Block(building_entries) = &ass.value.value {
                for entry in building_entries {
                    if let ast::Entry::Assignment(ass) = entry {
                        let key = ass.key.as_str();
                        if key.bytes().all(|b| b.is_ascii_digit()) {
                            // Province-level placement: 2671 = { naval_base = 2 }
                            if let ast::Value::Block(province_entries) = &ass.value.value {
                                validate_keys_in_dashmap(
                                    province_entries,
                                    ctx.buildings,
                                    "building",
                                    "common/buildings/*.txt",
                                    crate::validation::advanced_validation::UNKNOWN_BUILDING,
                                    diags,
                                );
                            }
                        } else {
                            // State-level building: infrastructure = 2
                            if !ctx.buildings.is_empty() && !ctx.buildings.contains_key(key) {
                                diags.push(Diagnostic {
                                    range: ast_range_to_lsp(&ass.key_range),
                                    severity: Some(DiagnosticSeverity::WARNING),
                                    message: format!(
                                        "Unknown building '{}'. buildings are defined in common/buildings/*.txt",
                                        key,
                                    ),
                                    code: Some(NumberOrString::String(
                                        crate::validation::advanced_validation::UNKNOWN_BUILDING.to_string(),
                                    )),
                                    source: Some("Hearts of Modding".to_string()),
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Extract a string value from a `NodeedValue`.
/// HOI4 identifiers are parsed as `Value::String` by the parser.
fn extract_string_value(val: &ast::NodeedValue) -> Option<String> {
    if let ast::Value::String(s) = &val.value {
        Some(s.clone())
    } else {
        None
    }
}

/// Check that every assignment key in `entries` exists in the DashMap.
fn validate_keys_in_dashmap<T>(
    entries: &[ast::Entry],
    map: &DashMap<InternedStr, T>,
    entity_type: &str,
    source_hint: &str,
    error_code: &str,
    diags: &mut Vec<Diagnostic>,
) {
    if map.is_empty() {
        return;
    }
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let name = ass.key.as_str();
            if !map.contains_key(name) {
                diags.push(Diagnostic {
                    range: ast_range_to_lsp(&ass.key_range),
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: format!(
                        "Unknown {} '{}'. {}s are defined in {}",
                        entity_type, name, entity_type, source_hint,
                    ),
                    code: Some(NumberOrString::String(error_code.to_string())),
                    source: Some("Hearts of Modding".to_string()),
                    ..Default::default()
                });
            }
        }
    }
}

/// Build a comma-separated list of known state categories.
fn format_known_list(
    map: &DashMap<InternedStr, LayeredValue<crate::scanner::state_category_scanner::StateCategory>>,
) -> String {
    let mut names: Vec<String> = map.iter().map(|e| e.key().to_string()).collect();
    names.sort();
    names.join(", ")
}
