use crate::ast;
use crate::lsp_convert::ast_range_to_lsp;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::ScopeStack;
use tower_lsp_server::ls_types::{
    Diagnostic, DiagnosticSeverity, NumberOrString,
};

/// Validates province references.
///
/// Per-entry: checks `province`, `on_province`, `is_province_id`, and
/// `victory_points` values against known province IDs.
/// Block-level: validates victory points reference provinces in the state.
pub(crate) struct ProvinceRule;

impl ValidationRule for ProvinceRule {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key_lower = ass.key.to_ascii_lowercase();

        // Province existence checks
        if key_lower == "province" || key_lower == "on_province" || key_lower == "is_province_id" {
            check_is_province(&ass.value, ctx, diags);
            return;
        }

        if key_lower == "victory_points" {
            if let ast::Value::Block(entries) = &ass.value.value {
                for entry in entries {
                    if let ast::Entry::Value(val) = entry {
                        check_is_province(val, ctx, diags);
                        break;
                    }
                }
            }
        }
    }

    fn check_block(
        &self,
        entries: &[ast::Entry],
        _ctx: &ValidationContext,
        diags: &mut Vec<Diagnostic>,
    ) {
        validate_victory_points_reference(entries, diags);
    }
}

fn check_is_province(
    val: &ast::NodeedValue,
    ctx: &ValidationContext,
    diags: &mut Vec<Diagnostic>,
) {
    let id_opt = match &val.value {
        ast::Value::Number(n) => Some(*n as u32),
        ast::Value::String(s) => s.parse::<u32>().ok(),
        _ => None,
    };

    if let Some(id) = id_opt {
        if !ctx.provinces.is_empty() && !ctx.provinces.contains_key(&id) {
            diags.push(Diagnostic {
                range: ast_range_to_lsp(&val.range),
                severity: Some(DiagnosticSeverity::WARNING),
                message: format!("Unknown province ID: {}", id),
                code: Some(NumberOrString::String(
                    crate::advanced_validation::UNKNOWN_TRIGGER.to_string(),
                )),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        }
    }
}

/// Check that `victory_points = { ... }` province IDs exist in the
/// state's `provinces = { ... }` list.
fn validate_victory_points_reference(
    entries: &[ast::Entry],
    diags: &mut Vec<Diagnostic>,
) {
    let mut state_provinces: Option<std::collections::HashSet<i32>> = None;
    let mut victory_points: Option<Vec<(i32, ast::Range)>> = None;
    collect_vp_data(entries, &mut state_provinces, &mut victory_points);

    if let (Some(provs), Some(vps)) = (state_provinces, victory_points) {
        for (vp_province, range) in &vps {
            if !provs.contains(vp_province) {
                diags.push(Diagnostic {
                    range: ast_range_to_lsp(range),
                    severity: Some(DiagnosticSeverity::HINT),
                    message: format!(
                        "Victory point province {} is not in the state's province list",
                        vp_province
                    ),
                    code: Some(NumberOrString::String(
                        crate::advanced_validation::VICTORY_POINT_PROVINCE_NOT_IN_STATE
                            .to_string(),
                    )),
                    source: Some("Hearts of Modding".to_string()),
                    data: Some(serde_json::json!({
                        "fix": "Remove this victory point or add the province to the state"
                    })),
                    ..Default::default()
                });
            }
        }
    }
}

fn collect_vp_data(
    entries: &[ast::Entry],
    state_provinces: &mut Option<std::collections::HashSet<i32>>,
    victory_points: &mut Option<Vec<(i32, ast::Range)>>,
) {
    for entry in entries {
        let ast::Entry::Assignment(ass) = entry else {
            continue;
        };
        let key_lower = ass.key.to_ascii_lowercase();

        if key_lower == "provinces" {
            if let ast::Value::Block(inner) = &ass.value.value {
                let mut provs = std::collections::HashSet::new();
                for prov_entry in inner {
                    if let ast::Entry::Value(val) = prov_entry {
                        if let ast::Value::Number(n) = &val.value {
                            provs.insert(*n as i32);
                        } else if let ast::Value::String(s) = &val.value {
                            if let Ok(n) = s.parse::<i32>() {
                                provs.insert(n);
                            }
                        }
                    }
                }
                *state_provinces = Some(provs);
            }
        } else if key_lower == "victory_points" {
            if let ast::Value::Block(inner) = &ass.value.value {
                let mut values: Vec<(i32, ast::Range)> = Vec::new();
                for vp_entry in inner {
                    if let ast::Entry::Value(val) = vp_entry {
                        let num = match &val.value {
                            ast::Value::Number(n) => Some(*n as i32),
                            ast::Value::String(s) => s.parse::<i32>().ok(),
                            _ => None,
                        };
                        if let Some(n) = num {
                            values.push((n, val.range.clone()));
                        }
                    }
                }
                // Parse pairs: (province_id, vp_value) — take first of each pair
                let mut vps = Vec::new();
                for i in (0..values.len()).step_by(2) {
                    if i < values.len() {
                        vps.push(values[i].clone());
                    }
                }
                *victory_points = Some(vps);
            }
        }

        // Recurse
        match &ass.value.value {
            ast::Value::Block(inner) | ast::Value::TaggedBlock(_, inner, _) => {
                collect_vp_data(inner, state_provinces, victory_points);
            }
            _ => {}
        }
    }
}
