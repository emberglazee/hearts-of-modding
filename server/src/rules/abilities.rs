use crate::ast;
use crate::lsp_convert::ast_range_to_lsp;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::ScopeStack;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates ability references and ability definition completeness.
///
/// Per-entry checks: `has_ability`, `add_ability`, `remove_ability` keys.
/// Block-level checks: `ability = { ... }` definitions for required fields
/// (name, desc, cost, duration, type, ai_will_do) and localization coverage.
pub(crate) struct AbilityRule;

impl ValidationRule for AbilityRule {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key_lower = ass.key.to_ascii_lowercase();
        if key_lower != "has_ability" && key_lower != "add_ability" && key_lower != "remove_ability"
        {
            return;
        }

        let ast::Value::String(val) = &ass.value.value else {
            return;
        };

        if !ctx.abilities.contains_key(val.as_str()) {
            diags.push(Diagnostic {
                range: ast_range_to_lsp(&ass.value.range),
                severity: Some(DiagnosticSeverity::WARNING),
                message: format!("Unknown ability: '{}'", val),
                code: Some(NumberOrString::String(
                    crate::advanced_validation::UNKNOWN_TRIGGER.to_string(),
                )),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        }
    }

    fn check_block(
        &self,
        entries: &[ast::Entry],
        ctx: &ValidationContext,
        diags: &mut Vec<Diagnostic>,
    ) {
        validate_ability_definitions(entries, ctx, diags);
    }
}

/// Recursively check `ability = { ... }` definition blocks for required fields
/// and localization keys.
fn validate_ability_definitions(
    entries: &[ast::Entry],
    ctx: &ValidationContext,
    diags: &mut Vec<Diagnostic>,
) {
    for entry in entries {
        let ast::Entry::Assignment(ass) = entry else {
            continue;
        };

        if ass.key.eq_ignore_ascii_case("ability") {
            if let ast::Value::Block(ability_entries) = &ass.value.value {
                for ability_entry in ability_entries {
                    let ast::Entry::Assignment(a_ass) = ability_entry else {
                        continue;
                    };
                    let ast::Value::Block(props) = &a_ass.value.value else {
                        continue;
                    };
                    check_ability_properties(a_ass, props, ctx, diags);
                }
            }
        }

        // Recurse into nested blocks
        match &ass.value.value {
            ast::Value::Block(inner) | ast::Value::TaggedBlock(_, inner, _) => {
                validate_ability_definitions(inner, ctx, diags);
            }
            _ => {}
        }
    }
}

fn check_ability_properties(
    a_ass: &ast::Assignment,
    props: &[ast::Entry],
    ctx: &ValidationContext,
    diags: &mut Vec<Diagnostic>,
) {
    let mut has_name = false;
    let mut has_desc = false;
    let mut has_cost = false;
    let mut has_duration = false;
    let mut has_type = false;
    let mut has_ai_will_do = false;

    for prop in props {
        let ast::Entry::Assignment(p_ass) = prop else {
            continue;
        };
        let p_key = p_ass.key.as_str();
        if p_key.eq_ignore_ascii_case("name") {
            has_name = true;
            if let ast::Value::String(s) = &p_ass.value.value {
                if !ctx.loc.contains_key(s.as_str()) {
                    diags.push(Diagnostic {
                        range: ast_range_to_lsp(&p_ass.value.range),
                        severity: Some(DiagnosticSeverity::WARNING),
                        message: format!(
                            "Ability '{}' is missing localization key: '{}'",
                            a_ass.key, s
                        ),
                        code: Some(NumberOrString::String(
                            crate::advanced_validation::ABILITY_MISSING_LOCALIZATION.to_string(),
                        )),
                        source: Some("Hearts of Modding".to_string()),
                        ..Default::default()
                    });
                }
            }
        } else if p_key.eq_ignore_ascii_case("desc") {
            has_desc = true;
            if let ast::Value::String(s) = &p_ass.value.value {
                if !ctx.loc.contains_key(s.as_str()) {
                    diags.push(Diagnostic {
                        range: ast_range_to_lsp(&p_ass.value.range),
                        severity: Some(DiagnosticSeverity::WARNING),
                        message: format!(
                            "Ability '{}' is missing localization key: '{}'",
                            a_ass.key, s
                        ),
                        code: Some(NumberOrString::String(
                            crate::advanced_validation::ABILITY_MISSING_LOCALIZATION.to_string(),
                        )),
                        source: Some("Hearts of Modding".to_string()),
                        ..Default::default()
                    });
                }
            }
        } else if p_key.eq_ignore_ascii_case("cost") {
            has_cost = true;
        } else if p_key.eq_ignore_ascii_case("duration") {
            has_duration = true;
        } else if p_key.eq_ignore_ascii_case("type") {
            has_type = true;
        } else if p_key.eq_ignore_ascii_case("ai_will_do") {
            has_ai_will_do = true;
        }
    }

    if !has_name {
        diags.push(Diagnostic {
            range: ast_range_to_lsp(&a_ass.key_range),
            severity: Some(DiagnosticSeverity::WARNING),
            message: format!("Ability '{}' is missing required 'name' field", a_ass.key),
            code: Some(NumberOrString::String(
                crate::advanced_validation::ABILITY_MISSING_REQUIRED_FIELD.to_string(),
            )),
            source: Some("Hearts of Modding".to_string()),
            ..Default::default()
        });
    }
    if !has_desc {
        diags.push(Diagnostic {
            range: ast_range_to_lsp(&a_ass.key_range),
            severity: Some(DiagnosticSeverity::WARNING),
            message: format!("Ability '{}' is missing required 'desc' field", a_ass.key),
            code: Some(NumberOrString::String(
                crate::advanced_validation::ABILITY_MISSING_REQUIRED_FIELD.to_string(),
            )),
            source: Some("Hearts of Modding".to_string()),
            ..Default::default()
        });
    }
    if !has_cost {
        diags.push(Diagnostic {
            range: ast_range_to_lsp(&a_ass.key_range),
            severity: Some(DiagnosticSeverity::WARNING),
            message: format!("Ability '{}' is missing required 'cost' field", a_ass.key),
            code: Some(NumberOrString::String(
                crate::advanced_validation::ABILITY_MISSING_REQUIRED_FIELD.to_string(),
            )),
            source: Some("Hearts of Modding".to_string()),
            ..Default::default()
        });
    }
    if !has_duration {
        diags.push(Diagnostic {
            range: ast_range_to_lsp(&a_ass.key_range),
            severity: Some(DiagnosticSeverity::INFORMATION),
            message: format!(
                "Ability '{}' is missing 'duration' field (ability will use indefinite duration)",
                a_ass.key
            ),
            code: Some(NumberOrString::String(
                crate::advanced_validation::ABILITY_MISSING_REQUIRED_FIELD.to_string(),
            )),
            source: Some("Hearts of Modding".to_string()),
            ..Default::default()
        });
    }
    if !has_type {
        diags.push(Diagnostic {
            range: ast_range_to_lsp(&a_ass.key_range),
            severity: Some(DiagnosticSeverity::INFORMATION),
            message: format!(
                "Ability '{}' is missing 'type' field (defaults may apply)",
                a_ass.key
            ),
            code: Some(NumberOrString::String(
                crate::advanced_validation::ABILITY_MISSING_REQUIRED_FIELD.to_string(),
            )),
            source: Some("Hearts of Modding".to_string()),
            ..Default::default()
        });
    }
    if !has_ai_will_do {
        diags.push(Diagnostic {
            range: ast_range_to_lsp(&a_ass.key_range),
            severity: Some(DiagnosticSeverity::INFORMATION),
            message: format!(
                "Ability '{}' is missing 'ai_will_do' block (AI will never use this ability)",
                a_ass.key
            ),
            code: Some(NumberOrString::String(
                crate::advanced_validation::ABILITY_MISSING_AI_LOGIC.to_string(),
            )),
            source: Some("Hearts of Modding".to_string()),
            ..Default::default()
        });
    }
}
