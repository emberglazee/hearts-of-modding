use crate::ast;
use crate::lsp_convert::ast_range_to_lsp;
use crate::rules::{ValidationContext, ValidationRule};
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates character skill levels against game-defined maxima.
///
/// Recurses through entries looking for character definitions
/// (`create_field_marshal`, `create_corps_commander`, etc.) and
/// checks `skill` values against the `GameDefines` max_skill table.
pub(crate) struct CharacterRule;

impl ValidationRule for CharacterRule {
    fn check_block(
        &self,
        entries: &[ast::Entry],
        ctx: &ValidationContext,
        diags: &mut Vec<Diagnostic>,
    ) {
        validate_character_skills_recursive(entries, ctx, diags, None);
    }
}

fn validate_character_skills_recursive(
    entries: &[ast::Entry],
    ctx: &ValidationContext,
    diags: &mut Vec<Diagnostic>,
    current_character_type: Option<&str>,
) {
    for entry in entries {
        let ast::Entry::Assignment(ass) = entry else {
            continue;
        };
        let key_lower = ass.key.to_ascii_lowercase();

        // Detect character type
        let char_type = match key_lower.as_str() {
            "create_field_marshal" | "field_marshal" => Some("field_marshal"),
            "create_corps_commander" | "corps_commander" => Some("corps_commander"),
            "create_navy_leader" | "navy_leader" => Some("navy_leader"),
            "create_operative_leader" => Some("operative"),
            _ => current_character_type,
        };

        // Check skill field
        if key_lower == "skill" {
            if let Some(ct) = char_type {
                let skill = match &ass.value.value {
                    ast::Value::Number(n) => Some(*n as i32),
                    ast::Value::String(s) => s.parse::<i32>().ok(),
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
                                crate::advanced_validation::CHARACTER_SKILL_EXCEEDS_MAX
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

        // Recurse into nested blocks — pass char_type as a string copy to
        // satisfy the borrow checker (char_type borrows from 'key_lower'
        // which is local to this iteration).
        let char_type_owned = char_type.map(|s| s.to_string());
        match &ass.value.value {
            ast::Value::Block(inner) => {
                validate_character_skills_recursive(inner, ctx, diags, char_type_owned.as_deref());
            }
            ast::Value::TaggedBlock(_, inner, _) => {
                validate_character_skills_recursive(inner, ctx, diags, char_type_owned.as_deref());
            }
            _ => {}
        }
    }
}
