use crate::parser::ast;
use crate::rules::visitor::AstVisitor;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::ScopeStack;
use crate::utils::lsp_convert::ast_range_to_lsp;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates ability references and ability definition completeness.
///
/// Uses the centralized AST visitor:
/// - Entry-level: catches `has_ability`, `add_ability`, `remove_ability` values
/// - Block-level: checks `ability = { ... }` definitions for required fields
pub(crate) struct AbilityRule;

impl ValidationRule for AbilityRule {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        _pushed_scope: bool,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key_lower = ass.key_text(ctx.source).to_ascii_lowercase();
        if key_lower != "has_ability" && key_lower != "add_ability" && key_lower != "remove_ability"
        {
            return;
        }

        let Some(val) = ass.value.value.as_str(ctx.source) else {
            return;
        };

        if !ctx.abilities.contains_key(val) {
            diags.push(Diagnostic {
                range: ast_range_to_lsp(&ass.value.range),
                severity: Some(DiagnosticSeverity::WARNING),
                message: format!("Unknown ability: '{}'", val),
                code: Some(NumberOrString::String(
                    crate::validation::advanced_validation::UNKNOWN_TRIGGER.to_string(),
                )),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
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

/// Track the state of a single ability definition.
struct AbilityDefState {
    name: String,
    key_range: ast::Range,
    has_name: bool,
    has_desc: bool,
    has_cost: bool,
    has_duration: bool,
    has_type: bool,
    has_ai_will_do: bool,
}

/// Visitor state: tracks `ability = { ... }` blocks and their definitions.
struct AbilityVisitor {
    /// True when inside `ability = { ... }` container, tracking definitions
    inside_ability_container: bool,
    /// Block nesting depth inside the current definition:
    ///   0 = at definition level (ready to accept a new ability def)
    ///   1 = inside a definition body (tracking properties)
    ///  >1 = inside nested property blocks (e.g. allowed, modifier, ai_will_do, etc.)
    block_depth: u32,
    /// Stack of ability definitions being tracked
    ability_stack: Vec<AbilityDefState>,
}

impl AbilityVisitor {
    fn new() -> Self {
        Self {
            inside_ability_container: false,
            block_depth: 0,
            ability_stack: Vec::new(),
        }
    }
}

impl AstVisitor for AbilityVisitor {
    fn enter_assignment(
        &mut self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        diags: &mut Vec<Diagnostic>,
    ) {
        // Track entry into `ability = { ... }` container
        if ass.key_text(ctx.source).eq_ignore_ascii_case("ability") {
            if matches!(&ass.value.value, ast::Value::Block(_)) {
                self.inside_ability_container = true;
            }
            return; // 'ability' itself is not a definition or property
        }

        if self.inside_ability_container {
            if matches!(
                &ass.value.value,
                ast::Value::Block(_) | ast::Value::TaggedBlock(..)
            ) {
                if self.block_depth == 0 {
                    // Direct child of `ability = {}` → this IS an ability definition
                    self.ability_stack.push(AbilityDefState {
                        name: ass.key_text(ctx.source).to_string(),
                        key_range: ass.key_range.clone(),
                        has_name: false,
                        has_desc: false,
                        has_cost: false,
                        has_duration: false,
                        has_type: false,
                        has_ai_will_do: false,
                    });
                    self.block_depth = 1;
                } else {
                    // Nested block inside a definition → could be a tracked property
                    if ass.key_text(ctx.source).eq_ignore_ascii_case("ai_will_do") {
                        if let Some(state) = self.ability_stack.last_mut() {
                            state.has_ai_will_do = true;
                        }
                    }
                    self.block_depth += 1;
                }
                return;
            }

            // Otherwise, this is a scalar property of the current ability
            if let Some(state) = self.ability_stack.last_mut() {
                let p_key = ass.key_text(ctx.source);
                if p_key.eq_ignore_ascii_case("name") {
                    state.has_name = true;
                    if let Some(s) = ass.value.value.as_str(ctx.source) {
                        if !ctx.loc.contains_key(s) {
                            diags.push(Diagnostic {
                                range: ast_range_to_lsp(&ass.value.range),
                                severity: Some(DiagnosticSeverity::WARNING),
                                message: format!(
                                    "Ability '{}' is missing localization key: '{}'",
                                    state.name, s
                                ),
                                code: Some(NumberOrString::String(
                                    crate::validation::advanced_validation::ABILITY_MISSING_LOCALIZATION
                                        .to_string(),
                                )),
                                source: Some("Hearts of Modding".to_string()),
                                ..Default::default()
                            });
                        }
                    }
                } else if p_key.eq_ignore_ascii_case("desc") {
                    state.has_desc = true;
                    if let Some(s) = ass.value.value.as_str(ctx.source) {
                        if !ctx.loc.contains_key(s) {
                            diags.push(Diagnostic {
                                range: ast_range_to_lsp(&ass.value.range),
                                severity: Some(DiagnosticSeverity::WARNING),
                                message: format!(
                                    "Ability '{}' is missing localization key: '{}'",
                                    state.name, s
                                ),
                                code: Some(NumberOrString::String(
                                    crate::validation::advanced_validation::ABILITY_MISSING_LOCALIZATION
                                        .to_string(),
                                )),
                                source: Some("Hearts of Modding".to_string()),
                                ..Default::default()
                            });
                        }
                    }
                } else if p_key.eq_ignore_ascii_case("cost") {
                    state.has_cost = true;
                } else if p_key.eq_ignore_ascii_case("duration") {
                    state.has_duration = true;
                } else if p_key.eq_ignore_ascii_case("type") {
                    state.has_type = true;
                } else if p_key.eq_ignore_ascii_case("ai_will_do") {
                    state.has_ai_will_do = true;
                }
            }
        }
    }

    fn exit_assignment(
        &mut self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        diags: &mut Vec<Diagnostic>,
    ) {
        // Track exit from `ability = { ... }` container
        if ass.key_text(ctx.source).eq_ignore_ascii_case("ability") {
            if matches!(&ass.value.value, ast::Value::Block(_)) {
                self.inside_ability_container = false;
            }
            return;
        }

        if self.inside_ability_container {
            if matches!(
                &ass.value.value,
                ast::Value::Block(_) | ast::Value::TaggedBlock(..)
            ) {
                if self.block_depth == 1 {
                    // Exiting a definition block → pop, validate, reset depth
                    if let Some(state) = self.ability_stack.pop() {
                        if !state.has_name {
                            diags.push(Diagnostic {
                                range: ast_range_to_lsp(&state.key_range),
                                severity: Some(DiagnosticSeverity::WARNING),
                                message: format!(
                                    "Ability '{}' is missing required 'name' field",
                                    state.name
                                ),
                                code: Some(NumberOrString::String(
                                    crate::validation::advanced_validation::ABILITY_MISSING_REQUIRED_FIELD
                                        .to_string(),
                                )),
                                source: Some("Hearts of Modding".to_string()),
                                ..Default::default()
                            });
                        }
                        if !state.has_desc {
                            diags.push(Diagnostic {
                                range: ast_range_to_lsp(&state.key_range),
                                severity: Some(DiagnosticSeverity::WARNING),
                                message: format!(
                                    "Ability '{}' is missing required 'desc' field",
                                    state.name
                                ),
                                code: Some(NumberOrString::String(
                                    crate::validation::advanced_validation::ABILITY_MISSING_REQUIRED_FIELD
                                        .to_string(),
                                )),
                                source: Some("Hearts of Modding".to_string()),
                                ..Default::default()
                            });
                        }
                        if !state.has_cost {
                            diags.push(Diagnostic {
                                range: ast_range_to_lsp(&state.key_range),
                                severity: Some(DiagnosticSeverity::WARNING),
                                message: format!(
                                    "Ability '{}' is missing required 'cost' field",
                                    state.name
                                ),
                                code: Some(NumberOrString::String(
                                    crate::validation::advanced_validation::ABILITY_MISSING_REQUIRED_FIELD
                                        .to_string(),
                                )),
                                source: Some("Hearts of Modding".to_string()),
                                ..Default::default()
                            });
                        }
                        if !state.has_duration {
                            diags.push(Diagnostic {
                                range: ast_range_to_lsp(&state.key_range),
                                severity: Some(DiagnosticSeverity::INFORMATION),
                                message: format!(
                                    "Ability '{}' is missing 'duration' field (ability will use indefinite duration)",
                                    state.name
                                ),
                                code: Some(NumberOrString::String(
                                    crate::validation::advanced_validation::ABILITY_MISSING_REQUIRED_FIELD
                                        .to_string(),
                                )),
                                source: Some("Hearts of Modding".to_string()),
                                ..Default::default()
                            });
                        }
                        if !state.has_type {
                            diags.push(Diagnostic {
                                range: ast_range_to_lsp(&state.key_range),
                                severity: Some(DiagnosticSeverity::INFORMATION),
                                message: format!(
                                    "Ability '{}' is missing 'type' field (defaults may apply)",
                                    state.name
                                ),
                                code: Some(NumberOrString::String(
                                    crate::validation::advanced_validation::ABILITY_MISSING_REQUIRED_FIELD
                                        .to_string(),
                                )),
                                source: Some("Hearts of Modding".to_string()),
                                ..Default::default()
                            });
                        }
                        if !state.has_ai_will_do {
                            diags.push(Diagnostic {
                                range: ast_range_to_lsp(&state.key_range),
                                severity: Some(DiagnosticSeverity::INFORMATION),
                                message: format!(
                                    "Ability '{}' is missing 'ai_will_do' block (AI will never use this ability)",
                                    state.name
                                ),
                                code: Some(NumberOrString::String(
                                    crate::validation::advanced_validation::ABILITY_MISSING_AI_LOGIC
                                        .to_string(),
                                )),
                                source: Some("Hearts of Modding".to_string()),
                                ..Default::default()
                            });
                        }
                    }
                    self.block_depth = 0;
                } else if self.block_depth > 1 {
                    self.block_depth -= 1;
                }
            }
        }
    }
}

impl AbilityRule {
    pub(crate) fn visitor() -> Box<dyn AstVisitor> {
        Box::new(AbilityVisitor::new())
    }
}
