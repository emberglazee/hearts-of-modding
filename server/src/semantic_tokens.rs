use crate::ast::*;
use tower_lsp::lsp_types::{SemanticToken, SemanticTokens, SemanticTokensResult};

pub fn get_semantic_tokens(script: &Script) -> SemanticTokensResult {
    let mut tokens = Vec::new();
    for entry in &script.entries {
        push_entry_tokens(entry, &mut tokens);
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

struct RawToken {
    line: u32,
    start: u32,
    length: u32,
    token_type: u32,
}

fn push_entry_tokens(entry: &Entry, tokens: &mut Vec<RawToken>) {
    match entry {
        Entry::Assignment(ass) => {
            tokens.push(RawToken {
                line: ass.key_range.start_line,
                start: ass.key_range.start_col,
                length: ass.key_range.end_col - ass.key_range.start_col,
                token_type: 1, // VARIABLE
            });
            tokens.push(RawToken {
                line: ass.operator_range.start_line,
                start: ass.operator_range.start_col,
                length: ass.operator_range.end_col - ass.operator_range.start_col,
                token_type: 4, // OPERATOR
            });
            push_value_tokens(&ass.value, tokens);
        }
        Entry::Value(val) => {
            push_value_tokens(val, tokens);
        }
        Entry::Comment(_, range) => {
            tokens.push(RawToken {
                line: range.start_line,
                start: range.start_col,
                length: range.end_col - range.start_col,
                token_type: 5, // COMMENT
            });
        }
    }
}

fn push_value_tokens(val: &NodeedValue, tokens: &mut Vec<RawToken>) {
    match &val.value {
        Value::String(_) => {
            tokens.push(RawToken {
                line: val.range.start_line,
                start: val.range.start_col,
                length: val.range.end_col - val.range.start_col,
                token_type: 2, // STRING
            });
        }
        Value::Number(_) => {
            tokens.push(RawToken {
                line: val.range.start_line,
                start: val.range.start_col,
                length: val.range.end_col - val.range.start_col,
                token_type: 3, // NUMBER
            });
        }
        Value::Boolean(_) => {
            tokens.push(RawToken {
                line: val.range.start_line,
                start: val.range.start_col,
                length: val.range.end_col - val.range.start_col,
                token_type: 0, // KEYWORD
            });
        }
        Value::Block(entries) => {
            for entry in entries {
                push_entry_tokens(entry, tokens);
            }
        }
        Value::TaggedBlock(tag, entries) => {
            // Highlighting the tag (rgb/hsv) as a keyword
            tokens.push(RawToken {
                line: val.range.start_line,
                start: val.range.start_col,
                length: tag.len() as u32,
                token_type: 0, // KEYWORD
            });
            for entry in entries {
                push_entry_tokens(entry, tokens);
            }
        }
    }
}