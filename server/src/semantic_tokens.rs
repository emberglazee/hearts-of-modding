use crate::ast::*;
use std::collections::HashSet;
use tower_lsp_server::ls_types::{SemanticToken, SemanticTokens, SemanticTokensResult};

#[allow(dead_code)]
#[repr(u32)]
enum TokenType {
    Keyword = 0,
    Variable = 1,
    String = 2,
    Number = 3,
    Operator = 4,
    Comment = 5,
    Type = 6,
}

/// Fields whose values are always localization keys, not entity references.
/// Values under these keys skip entity-type semantic highlighting.
const LOCALIZATION_VALUE_FIELDS: [&str; 4] = ["name", "desc", "custom_description", "text"];

pub fn get_semantic_tokens(
    script: &Script,
    keywords: &HashSet<String>,
    abilities: &HashSet<String>,
    strategy_plans: &HashSet<String>,
    ai_areas: &HashSet<String>,
    portrait_names: &HashSet<String>,
    character_names: &HashSet<String>,
    ideology_types: &HashSet<String>,
    achievement_names: &HashSet<String>,
    scripted_triggers: &HashSet<String>,
    scripted_effects: &HashSet<String>,
    country_tags: &HashSet<String>,
    color_codes: &HashSet<String>,
) -> SemanticTokensResult {
    let mut tokens = Vec::new();
    for entry in &script.entries {
        push_entry_tokens(
            entry,
            &mut tokens,
            keywords,
            abilities,
            strategy_plans,
            ai_areas,
            portrait_names,
            character_names,
            ideology_types,
            achievement_names,
            scripted_triggers,
            scripted_effects,
            country_tags,
            color_codes,
            None,
        );
    }

    // Sort tokens by line and column
    tokens.sort_by(|a, b| {
        if a.line != b.line {
            a.line.cmp(&b.line)
        } else {
            a.start.cmp(&b.start)
        }
    });

    let mut lsp_tokens = Vec::new();
    let mut last_line = 0;
    let mut last_start = 0;

    for token in tokens {
        let delta_line = token.line - last_line;
        let delta_start = if delta_line == 0 {
            token.start - last_start
        } else {
            token.start
        };

        lsp_tokens.push(SemanticToken {
            delta_line,
            delta_start,
            length: token.length,
            token_type: token.token_type,
            token_modifiers_bitset: 0,
        });

        last_line = token.line;
        last_start = token.start;
    }

    SemanticTokensResult::Tokens(SemanticTokens {
        result_id: None,
        data: lsp_tokens,
    })
}

fn push_entry_tokens(
    entry: &Entry,
    tokens: &mut Vec<RawToken>,
    keywords: &HashSet<String>,
    abilities: &HashSet<String>,
    strategy_plans: &HashSet<String>,
    ai_areas: &HashSet<String>,
    portrait_names: &HashSet<String>,
    character_names: &HashSet<String>,
    ideology_types: &HashSet<String>,
    achievement_names: &HashSet<String>,
    scripted_triggers: &HashSet<String>,
    scripted_effects: &HashSet<String>,
    country_tags: &HashSet<String>,
    color_codes: &HashSet<String>,
    parent_key: Option<&str>,
) {
    match entry {
        Entry::Assignment(ass) => {
            let is_keyword = keywords.contains(&ass.key);
            let is_ability = abilities.contains(&ass.key);
            let is_strategy_plan = strategy_plans.contains(&ass.key);
            let is_ai_area = ai_areas.contains(&ass.key);
            let is_portrait = portrait_names.contains(&ass.key);
            let is_character = character_names.contains(&ass.key);
            let is_achievement = achievement_names.contains(&ass.key);
            let is_color_code = color_codes.contains(&ass.key);

            if is_keyword {
                tokens.push(RawToken {
                    line: ass.key_range.start_line,
                    start: ass.key_range.start_col,
                    length: ass.key_range.end_col - ass.key_range.start_col,
                    token_type: TokenType::Keyword as u32,
                });
            } else if is_ability
                || is_strategy_plan
                || is_ai_area
                || is_portrait
                || is_character
                || is_achievement
                || is_color_code
                || country_tags.contains(&ass.key)
                || scripted_triggers.contains(&ass.key)
                || scripted_effects.contains(&ass.key)
            {
                tokens.push(RawToken {
                    line: ass.key_range.start_line,
                    start: ass.key_range.start_col,
                    length: ass.key_range.end_col - ass.key_range.start_col,
                    token_type: TokenType::Type as u32,
                });
            }

            tokens.push(RawToken {
                line: ass.operator_range.start_line,
                start: ass.operator_range.start_col,
                length: ass.operator_range.end_col - ass.operator_range.start_col,
                token_type: TokenType::Operator as u32,
            });
            push_value_tokens(
                &ass.value,
                tokens,
                keywords,
                abilities,
                strategy_plans,
                ai_areas,
                portrait_names,
                character_names,
                ideology_types,
                achievement_names,
                scripted_triggers,
                scripted_effects,
                country_tags,
                color_codes,
                Some(&ass.key),
            );
        }
        Entry::Value(val) => {
            push_value_tokens(
                val,
                tokens,
                keywords,
                abilities,
                strategy_plans,
                ai_areas,
                portrait_names,
                character_names,
                ideology_types,
                achievement_names,
                scripted_triggers,
                scripted_effects,
                country_tags,
                color_codes,
                parent_key,
            );
        }
        Entry::Comment(_, range) => {
            tokens.push(RawToken {
                line: range.start_line,
                start: range.start_col,
                length: range.end_col - range.start_col,
                token_type: TokenType::Comment as u32,
            });
        }
    }
}

fn push_value_tokens(
    val: &NodeedValue,
    tokens: &mut Vec<RawToken>,
    keywords: &HashSet<String>,
    abilities: &HashSet<String>,
    strategy_plans: &HashSet<String>,
    ai_areas: &HashSet<String>,
    portrait_names: &HashSet<String>,
    character_names: &HashSet<String>,
    ideology_types: &HashSet<String>,
    achievement_names: &HashSet<String>,
    scripted_triggers: &HashSet<String>,
    scripted_effects: &HashSet<String>,
    country_tags: &HashSet<String>,
    color_codes: &HashSet<String>,
    parent_key: Option<&str>,
) {
    match &val.value {
        Value::String(s) => {
            let is_localization_value =
                parent_key.is_some_and(|k| LOCALIZATION_VALUE_FIELDS.contains(&k));

            if keywords.contains(s) {
                tokens.push(RawToken {
                    line: val.range.start_line,
                    start: val.range.start_col,
                    length: val.range.end_col - val.range.start_col,
                    token_type: TokenType::Keyword as u32,
                });
            } else if !is_localization_value
                && (abilities.contains(s)
                    || strategy_plans.contains(s)
                    || ai_areas.contains(s)
                    || portrait_names.contains(s)
                    || character_names.contains(s)
                    || ideology_types.contains(s)
                    || achievement_names.contains(s)
                    || color_codes.contains(s)
                    || country_tags.contains(s)
                    || scripted_triggers.contains(s)
                    || scripted_effects.contains(s))
            {
                tokens.push(RawToken {
                    line: val.range.start_line,
                    start: val.range.start_col,
                    length: val.range.end_col - val.range.start_col,
                    token_type: TokenType::Type as u32,
                });
            } else if s.starts_with("var:") || s.starts_with("temp_var:") {
                tokens.push(RawToken {
                    line: val.range.start_line,
                    start: val.range.start_col,
                    length: val.range.end_col - val.range.start_col,
                    token_type: TokenType::Variable as u32,
                });
            }
        }
        Value::Number(_) => {
            tokens.push(RawToken {
                line: val.range.start_line,
                start: val.range.start_col,
                length: val.range.end_col - val.range.start_col,
                token_type: TokenType::Number as u32,
            });
        }
        Value::Boolean(_) => {
            tokens.push(RawToken {
                line: val.range.start_line,
                start: val.range.start_col,
                length: val.range.end_col - val.range.start_col,
                token_type: TokenType::Keyword as u32,
            });
        }
        Value::Block(entries) => {
            for entry in entries {
                push_entry_tokens(
                    entry,
                    tokens,
                    keywords,
                    abilities,
                    strategy_plans,
                    ai_areas,
                    portrait_names,
                    character_names,
                    ideology_types,
                    achievement_names,
                    scripted_triggers,
                    scripted_effects,
                    country_tags,
                    color_codes,
                    parent_key,
                );
            }
        }
        Value::TaggedBlock(tag, entries, _) => {
            tokens.push(RawToken {
                line: val.range.start_line,
                start: val.range.start_col,
                length: tag.len() as u32,
                token_type: TokenType::Keyword as u32,
            });
            for entry in entries {
                push_entry_tokens(
                    entry,
                    tokens,
                    keywords,
                    abilities,
                    strategy_plans,
                    ai_areas,
                    portrait_names,
                    character_names,
                    ideology_types,
                    achievement_names,
                    scripted_triggers,
                    scripted_effects,
                    country_tags,
                    color_codes,
                    parent_key,
                );
            }
        }
    }
}

struct RawToken {
    line: u32,
    start: u32,
    length: u32,
    token_type: u32,
}
