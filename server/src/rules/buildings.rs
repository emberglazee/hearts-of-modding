use crate::parser::ast;
use crate::rules::visitor::AstVisitor;
use crate::rules::{ValidationContext, ValidationRule};
use crate::utils::lsp_convert::ast_range_to_lsp;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates building level values against their maximum allowed levels.
///
/// Uses the centralized AST visitor to react to `buildings = { ... }`
/// blocks and check each building's level against the scanner's max_level
/// data — no more recursive traversal.
pub(crate) struct BuildingRule;

impl ValidationRule for BuildingRule {
    fn check_block(
        &self,
        _entries: &[ast::Entry],
        _ctx: &ValidationContext,
        _diags: &mut Vec<Diagnostic>,
    ) {
    }
}

/// Visitor state: tracks depth inside `buildings = { ... }` blocks.
///
/// For the HOI4 building structure:
/// ```hoi4
/// buildings = {
///     infrastructure = 10
///     2671 = {              # province-level block
///         naval_base = 5
///     }
/// }
/// ```
/// The depth counter increments when we see `buildings = { Block }`
/// and decrements on exit. Nested province blocks (like `2671 = { }`)
/// are automatically inside the buildings block because depth > 0,
/// so their building-name keys are checked too.
struct BuildingVisitor {
    in_buildings: u32,
}

impl BuildingVisitor {
    fn new() -> Self {
        Self { in_buildings: 0 }
    }
}

impl AstVisitor for BuildingVisitor {
    fn enter_assignment(
        &mut self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &crate::scope::scope::ScopeStack,
        diags: &mut Vec<Diagnostic>,
    ) {
        // Track entry into `buildings = { ... }` blocks
        if ass.key_text(ctx.source).eq_ignore_ascii_case("buildings") {
            if matches!(&ass.value.value, ast::Value::Block(_)) {
                self.in_buildings += 1;
            }
            return; // 'buildings' itself is not a building name to check
        }

        if self.in_buildings == 0 {
            return;
        }

        // We're inside a `buildings = { ... }` block.
        // Check if this key is a known building name with a numeric level.
        let level = match &ass.value.value {
            ast::Value::Number(n) => Some(*n as i32),
            ast::Value::String(s) => s.resolve(ctx.source).parse::<i32>().ok(),
            _ => None,
        };

        if let Some(level) = level {
            if let Some(building) = ctx.buildings.get(ass.key_text(ctx.source)) {
                if let Some(max_level) = building.max_level {
                    if level > max_level {
                        diags.push(Diagnostic {
                            range: ast_range_to_lsp(&ass.value.range),
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: format!(
                                "Building level {} exceeds maximum level {} for '{}'",
                                level,
                                max_level,
                                ass.key_text(ctx.source)
                            ),
                            code: Some(NumberOrString::String(
                                crate::validation::advanced_validation::BUILDING_LEVEL_EXCEEDS_MAX
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
    }

    fn exit_assignment(
        &mut self,
        ass: &ast::Assignment,
        _ctx: &ValidationContext,
        _scope: &crate::scope::scope::ScopeStack,
        _diags: &mut Vec<Diagnostic>,
    ) {
        if ass.key_text(_ctx.source).eq_ignore_ascii_case("buildings") {
            if matches!(&ass.value.value, ast::Value::Block(_)) {
                self.in_buildings = self.in_buildings.saturating_sub(1);
            }
        }
    }
}

impl BuildingRule {
    pub(crate) fn visitor() -> Box<dyn AstVisitor> {
        Box::new(BuildingVisitor::new())
    }
}
