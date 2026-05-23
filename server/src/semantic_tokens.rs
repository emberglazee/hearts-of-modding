use crate::ast::*;
use std::collections::HashSet;
use tower_lsp::lsp_types::{SemanticToken, SemanticTokens, SemanticTokensResult};

pub fn get_semantic_tokens(
    script: &Script,
    keywords: &HashSet<String>,
    abilities: &HashSet<String>,
    strategy_plans: &HashSet<String>,
    portrait_names: &HashSet<String>,
    character_names: &HashSet<String>,
    ideology_types: &HashSet<String>,
) -> SemanticTokensResult {
    let mut tokens = Vec::new();
    for entry in &script.entries {
        push_entry_tokens(entry, &mut tokens, keywords, abilities, strategy_plans, portrait_names, character_names, ideology_types);
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
    portrait_names: &HashSet<String>,
    character_names: &HashSet<String>,
    ideology_types: &HashSet<String>,
) {
    match entry {
        Entry::Assignment(ass) => {
            let is_keyword = keywords.contains(&ass.key);
            let is_ability = abilities.contains(&ass.key);
            let is_strategy_plan = strategy_plans.contains(&ass.key);
            let is_portrait = portrait_names.contains(&ass.key);
            let is_character = character_names.contains(&ass.key);

            if is_keyword {
                tokens.push(RawToken {
                    line: ass.key_range.start_line,
                    start: ass.key_range.start_col,
                    length: ass.key_range.end_col - ass.key_range.start_col,
                    token_type: 0,
                });
            } else if is_ability || is_strategy_plan || is_portrait || is_character {
                tokens.push(RawToken {
                    line: ass.key_range.start_line,
                    start: ass.key_range.start_col,
                    length: ass.key_range.end_col - ass.key_range.start_col,
                    token_type: 6,
                });
            }

            tokens.push(RawToken {
                line: ass.operator_range.start_line,
                start: ass.operator_range.start_col,
                length: ass.operator_range.end_col - ass.operator_range.start_col,
                token_type: 4,
            });
            push_value_tokens(&ass.value, tokens, keywords, abilities, strategy_plans, portrait_names, character_names, ideology_types);
        }
        Entry::Value(val) => {
            push_value_tokens(val, tokens, keywords, abilities, strategy_plans, portrait_names, character_names, ideology_types);
        }
        Entry::Comment(_, range) => {
            tokens.push(RawToken {
                line: range.start_line,
                start: range.start_col,
                length: range.end_col - range.start_col,
                token_type: 5,
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
    portrait_names: &HashSet<String>,
    character_names: &HashSet<String>,
    ideology_types: &HashSet<String>,
) {
    match &val.value {
        Value::String(s) => {
            if keywords.contains(s) {
                tokens.push(RawToken {
                    line: val.range.start_line,
                    start: val.range.start_col,
                    length: val.range.end_col - val.range.start_col,
                    token_type: 0,
                });
            } else if abilities.contains(s) || strategy_plans.contains(s) || portrait_names.contains(s)
                || character_names.contains(s) || ideology_types.contains(s)
            {
                tokens.push(RawToken {
                    line: val.range.start_line,
                    start: val.range.start_col,
                    length: val.range.end_col - val.range.start_col,
                    token_type: 6,
                });
            } else if s.starts_with("var:") || s.starts_with("temp_var:") {
                tokens.push(RawToken {
                    line: val.range.start_line,
                    start: val.range.start_col,
                    length: val.range.end_col - val.range.start_col,
                    token_type: 1,
                });
            }
        }
        Value::Number(_) => {
            tokens.push(RawToken {
                line: val.range.start_line,
                start: val.range.start_col,
                length: val.range.end_col - val.range.start_col,
                token_type: 3,
            });
        }
        Value::Boolean(_) => {
            tokens.push(RawToken {
                line: val.range.start_line,
                start: val.range.start_col,
                length: val.range.end_col - val.range.start_col,
                token_type: 0,
            });
        }
        Value::Block(entries) => {
            for entry in entries {
                push_entry_tokens(entry, tokens, keywords, abilities, strategy_plans, portrait_names, character_names, ideology_types);
            }
        }
        Value::TaggedBlock(tag, entries, _) => {
            tokens.push(RawToken {
                line: val.range.start_line,
                start: val.range.start_col,
                length: tag.len() as u32,
                token_type: 0,
            });
            for entry in entries {
                push_entry_tokens(entry, tokens, keywords, abilities, strategy_plans, portrait_names, character_names, ideology_types);
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
