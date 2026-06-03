use crate::parser::ast;
use crate::rules::visitor::AstVisitor;
use crate::rules::{ValidationContext, ValidationRule};
use crate::utils::lsp_convert::ast_range_to_lsp;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates character skill levels against game-defined maxima.
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

        // Validate skill level if we're inside a character definition
        if ass.key_text(ctx.source).eq_ignore_ascii_case("skill") {
            if let Some(ct) = self.current_character_type() {
                let skill = match &ass.value.value {
                    ast::Value::Number(n) => Some(*n as i32),
                    ast::Value::String(s) => s.resolve(ctx.source).parse::<i32>().ok(),
                    _ => None,
                };

                if let Some(skill) = skill {
                    let max_skill = ctx.defines.get_max_skill(ct);
                    if skill > max_skill {
                        diags.push(Diagnostic {
                            range: ast_range_to_lsp(&ass.value.range),
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: format!(
                                "Skill level {} exceeds maximum {} for {}",
                                skill, max_skill, ct
                            ),
                            code: Some(NumberOrString::String(
                                crate::validation::advanced_validation::CHARACTER_SKILL_EXCEEDS_MAX
                                    .to_string(),
                            )),
                            source: Some("Hearts of Modding".to_string()),
                            data: Some(serde_json::json!({
                                "fix": format!("Set to maximum skill: {}", max_skill)
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
