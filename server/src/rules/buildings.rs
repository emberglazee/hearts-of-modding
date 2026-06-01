use crate::ast;
use crate::lsp_convert::ast_range_to_lsp;
use crate::rules::{ValidationContext, ValidationRule};
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates building level values against their maximum allowed levels.
///
/// Recurses through entries looking for `buildings = { ... }` blocks and
/// checks each building's level against the scanner's max_level data.
pub(crate) struct BuildingRule;

impl ValidationRule for BuildingRule {
    fn check_block(
        &self,
        entries: &[ast::Entry],
        ctx: &ValidationContext,
        diags: &mut Vec<Diagnostic>,
    ) {
        validate_buildings_recursive(entries, ctx, diags);
    }
}

fn validate_buildings_recursive(
    entries: &[ast::Entry],
    ctx: &ValidationContext,
    diags: &mut Vec<Diagnostic>,
) {
    for entry in entries {
        let ast::Entry::Assignment(ass) = entry else {
            continue;
        };

        if ass.key.to_ascii_lowercase() == "buildings" {
            if let ast::Value::Block(building_entries) = &ass.value.value {
                validate_building_block(building_entries, ctx, diags);
            }
        }

        match &ass.value.value {
            ast::Value::Block(inner) => {
                validate_buildings_recursive(inner, ctx, diags);
            }
            ast::Value::TaggedBlock(_, inner, _) => {
                validate_buildings_recursive(inner, ctx, diags);
            }
            _ => {}
        }
    }
}

fn validate_building_block(
    entries: &[ast::Entry],
    ctx: &ValidationContext,
    diags: &mut Vec<Diagnostic>,
) {
    for entry in entries {
        let ast::Entry::Assignment(ass) = entry else {
            continue;
        };
        let building_name = &ass.key;

        let level = match &ass.value.value {
            ast::Value::Number(n) => Some(*n as i32),
            ast::Value::String(s) => s.parse::<i32>().ok(),
            _ => None,
        };

        if let Some(level) = level {
            if let Some(building) = ctx.buildings.get(building_name) {
                if let Some(max_level) = building.max_level {
                    if level > max_level {
                        diags.push(Diagnostic {
                            range: ast_range_to_lsp(&ass.value.range),
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: format!(
                                "Building level {} exceeds maximum level {} for '{}'",
                                level, max_level, building_name
                            ),
                            code: Some(NumberOrString::String(
                                crate::advanced_validation::BUILDING_LEVEL_EXCEEDS_MAX
                                    .to_string(),
                            )),
                            source: Some("Hearts of Modding".to_string()),
                            data: Some(serde_json::json!({
                                "fix": format!("Set to maximum level: {}", max_level)
                            })),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        // Recurse into province-specific building blocks (keyed by numeric province ID)
        if let ast::Value::Block(nested) = &ass.value.value {
            if ass.key.parse::<i32>().is_ok() {
                validate_building_block(nested, ctx, diags);
            }
        }
    }
}
