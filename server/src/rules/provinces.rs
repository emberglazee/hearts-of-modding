use std::collections::HashSet;

use crate::parser::ast;
use crate::rules::visitor::AstVisitor;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::ScopeStack;
use crate::utils::lsp_convert::ast_range_to_lsp;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates province references.
///
/// Per-entry: checks `province`, `on_province`, `is_province_id`, and
/// `victory_points` values against known province IDs.
///
/// Block-level (via AstVisitor): validates victory points reference
/// provinces in the state — accumulates data during the walk and
/// cross-references in `after_walk`.
pub(crate) struct ProvinceRule;

impl ValidationRule for ProvinceRule {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        _pushed_scope: bool,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key_lower = ass.key_text(ctx.source).to_ascii_lowercase();

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
        _entries: &[ast::Entry],
        _ctx: &ValidationContext,
        _diags: &mut Vec<Diagnostic>,
    ) {
    }
}

/// Simple province ID existence check.
fn check_is_province(val: &ast::NodeedValue, ctx: &ValidationContext, diags: &mut Vec<Diagnostic>) {
    let id_opt = match &val.value {
        ast::Value::Number(n) => Some(*n as u32),
        ast::Value::String(s) => s.resolve(ctx.source).parse::<u32>().ok(),
        _ => None,
    };

    if let Some(id) = id_opt {
        if !ctx.provinces.is_empty() && !ctx.provinces.contains_key(&id) {
            diags.push(Diagnostic {
                range: ast_range_to_lsp(&val.range),
                severity: Some(DiagnosticSeverity::WARNING),
                message: format!("Unknown province ID: {}", id),
                code: Some(NumberOrString::String(
                    crate::validation::advanced_validation::UNKNOWN_TRIGGER.to_string(),
                )),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        }
    }
}

/// Visitor state for victory point cross-reference.
///
/// For a state definition like:
/// ```hoi4
/// state = {
///     provinces = { 1 2 3 }
///     victory_points = { 1 10 3 5 }
/// }
/// ```
/// The visitor accumulates the province set and VP pairs during
/// `enter_assignment`. `after_walk` cross-references once both
/// data sets are collected.
struct ProvinceVpVisitor {
    /// Set of province IDs from `provinces = { ... }`
    state_provinces: Option<HashSet<i32>>,
    /// Victory point province IDs with their source ranges
    victory_points: Option<Vec<(i32, ast::Range)>>,
}

impl ProvinceVpVisitor {
    fn new() -> Self {
        Self {
            state_provinces: None,
            victory_points: None,
        }
    }
}

impl AstVisitor for ProvinceVpVisitor {
    fn enter_assignment(
        &mut self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        _diags: &mut Vec<Diagnostic>,
    ) {
        let key_lower = ass.key_text(ctx.source).to_ascii_lowercase();

        // Collect province IDs from `provinces = { ... }`
        if key_lower == "provinces" {
            if let ast::Value::Block(entries) = &ass.value.value {
                let mut provs = HashSet::new();
                for entry in entries {
                    if let ast::Entry::Value(val) = entry {
                        match &val.value {
                            ast::Value::Number(n) => {
                                provs.insert(*n as i32);
                            }
                            ast::Value::String(s) => {
                                if let Ok(n) = s.resolve(ctx.source).parse::<i32>() {
                                    provs.insert(n);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                self.state_provinces = Some(provs);
            }
            return;
        }

        // Collect victory points from `victory_points = { ... }`
        if key_lower == "victory_points" {
            if let ast::Value::Block(entries) = &ass.value.value {
                let mut values: Vec<(i32, ast::Range)> = Vec::new();
                for entry in entries {
                    if let ast::Entry::Value(val) = entry {
                        let num = match &val.value {
                            ast::Value::Number(n) => Some(*n as i32),
                            ast::Value::String(s) => s.resolve(ctx.source).parse::<i32>().ok(),
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
                self.victory_points = Some(vps);
            }
        }
    }

    fn after_walk(&mut self, _ctx: &ValidationContext, diags: &mut Vec<Diagnostic>) {
        // Cross-reference: check that VP provinces exist in the state's province list
        if let (Some(provs), Some(vps)) =
            (self.state_provinces.as_ref(), self.victory_points.as_ref())
        {
            for (vp_province, range) in vps {
                if !provs.contains(vp_province) {
                    diags.push(Diagnostic {
                        range: ast_range_to_lsp(range),
                        severity: Some(DiagnosticSeverity::HINT),
                        message: format!(
                            "Victory point province {} is not in the state's province list",
                            vp_province
                        ),
                        code: Some(NumberOrString::String(
                            crate::validation::advanced_validation::VICTORY_POINT_PROVINCE_NOT_IN_STATE
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
}

impl ProvinceRule {
    pub(crate) fn vp_visitor() -> Box<dyn AstVisitor> {
        Box::new(ProvinceVpVisitor::new())
    }
}
