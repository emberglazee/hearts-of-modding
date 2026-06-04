use crate::parser::ast;
use crate::rules::visitor::AstVisitor;
use crate::rules::{ValidationContext, ValidationRule};
use crate::utils::lsp_convert::ast_range_to_lsp;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Sub-skill keys whose gameplay bonus is capped at level 10.
const SUB_SKILLS: &[&str] = &[
    "attack_skill",
    "defense_skill",
    "planning_skill",
    "logistics_skill",
    "coordination_skill",
    "maneuvering_skill",
];

const SUB_SKILL_PRACTICAL_CAP: i32 = 10;

/// HOI4 engine hardcaps overall `skill` at 9. Values > 9 are silently set to 0 in-game.
/// This is not configurable via Lua defines — it's baked into the game binary.
const MAX_SKILL: i32 = 9;

/// Parse an i32 value from an AST assignment value (number or string).
fn parse_i32_value(ass: &ast::Assignment, source: &str) -> Option<i32> {
    match &ass.value.value {
        ast::Value::Number(n) => Some(*n as i32),
        ast::Value::String(s) => s.resolve(source).parse::<i32>().ok(),
        _ => None,
    }
}

/// Validates character skill levels,
/// and warns when sub-skills exceed the practical bonus cap of 10.
///
/// Uses the centralized AST visitor to check `skill` values inside
/// character definition blocks (create_field_marshal, etc.).
pub(crate) struct CharacterRule;

impl ValidationRule for CharacterRule {
    fn check_block(
        &self,
        _entries: &[ast::Entry],
        _ctx: &ValidationContext,
        _diags: &mut Vec<Diagnostic>,
    ) {
    }
}

/// Visitor state: a stack of character types for nested definition blocks.
///
/// For the HOI4 character structure:
/// ```hoi4
/// create_field_marshal = {
///     skill = 5
///     traits = { ... }
/// }
/// ```
/// Uses a stack so nested character definitions don't corrupt the type:
/// when we `enter` a definition we push, when we `exit` we pop.
fn detect_character_type(key: &str) -> Option<&'static str> {
    match key.to_ascii_lowercase().as_str() {
        "create_field_marshal" | "field_marshal" => Some("field_marshal"),
        "create_corps_commander" | "corps_commander" => Some("corps_commander"),
        "create_navy_leader" | "navy_leader" => Some("navy_leader"),
        "create_operative_leader" => Some("operative"),
        _ => None,
    }
}

struct CharacterVisitor {
    char_type_stack: Vec<&'static str>,
}

impl CharacterVisitor {
    fn new() -> Self {
        Self {
            char_type_stack: Vec::new(),
        }
    }

    fn current_character_type(&self) -> Option<&str> {
        self.char_type_stack.last().copied()
    }
}

impl AstVisitor for CharacterVisitor {
    fn enter_assignment(
        &mut self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &crate::scope::scope::ScopeStack,
        diags: &mut Vec<Diagnostic>,
    ) {
        // Detect character type from definition keys and push onto stack.
        if let Some(ct) = detect_character_type(ass.key_text(ctx.source)) {
            self.char_type_stack.push(ct);
        }

        if let Some(ct) = self.current_character_type() {
            let key = ass.key_text(ctx.source);

            // Validate overall skill level
            if key.eq_ignore_ascii_case("skill") {
                if let Some(skill) = parse_i32_value(ass, ctx.source) {
                    if skill < 0 {
                        diags.push(Diagnostic {
                            range: ast_range_to_lsp(&ass.value.range),
                            severity: Some(DiagnosticSeverity::WARNING),
                            message: format!(
                                "Skill level {} for {} is negative; the game will clamp it to 0 in-game",
                                skill, ct,
                            ),
                            code: Some(NumberOrString::String(
                                crate::validation::advanced_validation::CHARACTER_NEGATIVE_SKILL
                                    .to_string(),
                            )),
                            source: Some("Hearts of Modding".to_string()),
                            data: Some(serde_json::json!({
                                "fix": format!("Set to at least 1 (max {})", MAX_SKILL)
                            })),
                            ..Default::default()
                        });
                    } else if skill > MAX_SKILL {
                        diags.push(Diagnostic {
                            range: ast_range_to_lsp(&ass.value.range),
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: format!(
                                "Skill level {} exceeds maximum {} for {}",
                                skill, MAX_SKILL, ct
                            ),
                            code: Some(NumberOrString::String(
                                crate::validation::advanced_validation::CHARACTER_SKILL_EXCEEDS_MAX
                                    .to_string(),
                            )),
                            source: Some("Hearts of Modding".to_string()),
                            data: Some(serde_json::json!({
                                "fix": format!("Set to maximum skill: {}", MAX_SKILL)
                            })),
                            ..Default::default()
                        });
                    }
                }
            }

            // Warn on sub-skills exceeding practical cap or negative
            if SUB_SKILLS.contains(&key) {
                if let Some(val) = parse_i32_value(ass, ctx.source) {
                    if val < 0 {
                        diags.push(Diagnostic {
                            range: ast_range_to_lsp(&ass.value.range),
                            severity: Some(DiagnosticSeverity::WARNING),
                            message: format!(
                                "{} is {}; negative values are clamped to 1 in-game",
                                key, val,
                            ),
                            code: Some(NumberOrString::String(
                                crate::validation::advanced_validation::CHARACTER_NEGATIVE_SKILL
                                    .to_string(),
                            )),
                            source: Some("Hearts of Modding".to_string()),
                            data: Some(serde_json::json!({
                                "fix": format!("Set to at least 0 (practical cap {})", SUB_SKILL_PRACTICAL_CAP)
                            })),
                            ..Default::default()
                        });
                    } else if val > SUB_SKILL_PRACTICAL_CAP {
                        diags.push(Diagnostic {
                            range: ast_range_to_lsp(&ass.value.range),
                            severity: Some(DiagnosticSeverity::WARNING),
                            message: format!(
                                "{} is {}; the gameplay bonus caps at level 10 (higher values are accepted but confer no extra benefit)",
                                key, val,
                            ),
                            code: Some(NumberOrString::String(
                                crate::validation::advanced_validation::CHARACTER_SUBSKILL_EXCEEDS_PRACTICAL
                                    .to_string(),
                            )),
                            source: Some("Hearts of Modding".to_string()),
                            data: Some(serde_json::json!({
                                "fix": format!("Set to {}", SUB_SKILL_PRACTICAL_CAP)
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
        // Pop the stack when we leave a character definition block
        if detect_character_type(ass.key_text(_ctx.source)).is_some() {
            self.char_type_stack.pop();
        }
    }
}

impl CharacterRule {
    pub(crate) fn visitor() -> Box<dyn AstVisitor> {
        Box::new(CharacterVisitor::new())
    }
}
