use crate::ast::*;
use crate::entity_lookup::EntityKind;
use std::collections::{HashMap, HashSet};
use tower_lsp_server::ls_types::{SemanticToken, SemanticTokens, SemanticTokensResult};

/// Indices into the LSP semantic token legend registered in lsp_handler.rs.
/// Must match the order of token_types in the legend vec.
#[repr(u32)]
enum TokenType {
    Keyword = 0,
    Variable = 1,
    String = 2,
    Number = 3,
    Operator = 4,
    Comment = 5,
    Type = 6,
    Event = 7,
    Function = 8,
    Enum = 9,
    EnumMember = 10,
    Struct = 11,
    Class = 12,
    Property = 13,
}

/// Fields whose values are always localization keys, not entity references.
/// Values under these keys skip entity-type semantic highlighting.
const LOCALIZATION_VALUE_FIELDS: [&str; 4] = ["name", "desc", "custom_description", "text"];

/// Context struct that replaces the 18-parameter threading pattern.
/// Carries all data needed to resolve token types for a document.
pub struct SemanticTokenContext {
    pub keywords: HashSet<String>,
    pub entity_names: HashMap<String, EntityKind>,
}

impl SemanticTokenContext {
    pub fn new(keywords: HashSet<String>, entity_names: HashMap<String, EntityKind>) -> Self {
        SemanticTokenContext {
            keywords,
            entity_names,
        }
    }
}

/// Map an entity kind to its semantic token type index.
/// Each entity kind gets a distinct type so the theme can color them differently.
fn entity_kind_to_token_type(kind: EntityKind) -> u32 {
    match kind {
        // Callable/behavioural constructs → Function
        EntityKind::ScriptedTrigger
        | EntityKind::ScriptedEffect
        | EntityKind::ScriptedLoc
        | EntityKind::Ability
        | EntityKind::AdjacencyRule => TokenType::Function as u32,

        // Named categories → Enum
        EntityKind::Ideology | EntityKind::SoundCategory => TokenType::Enum as u32,

        // Members of named categories → EnumMember
        EntityKind::SubIdeology | EntityKind::ColorCode => TokenType::EnumMember as u32,

        // Data structures → Struct
        EntityKind::Trait | EntityKind::Character | EntityKind::Building => {
            TokenType::Struct as u32
        }

        // Named concept definitions → Class
        EntityKind::Idea | EntityKind::AiArea | EntityKind::AiStrategyPlan => {
            TokenType::Class as u32
        }

        // Narrative / event-like → Event
        EntityKind::Event | EntityKind::Focus | EntityKind::Achievement => TokenType::Event as u32,

        // Asset references → Property
        EntityKind::Sprite
        | EntityKind::MusicAsset
        | EntityKind::MusicStation
        | EntityKind::Song
        | EntityKind::Sound
        | EntityKind::SoundEffect
        | EntityKind::Falloff
        | EntityKind::Portrait
        | EntityKind::CustomModifier
        | EntityKind::ModifierMapping => TokenType::Property as u32,

        // Identifiers → Type
        EntityKind::CountryTag
        | EntityKind::StrategicRegion
        | EntityKind::State
        | EntityKind::Province => TokenType::Type as u32,

        // Variables
        EntityKind::Variable | EntityKind::EventTarget => TokenType::Variable as u32,

        // Localization entries → String
        EntityKind::Localization => TokenType::String as u32,

        // Fallback for any unexpected kind
        _ => TokenType::Type as u32,
    }
}

pub fn get_semantic_tokens(script: &Script, ctx: &SemanticTokenContext) -> SemanticTokensResult {
    let mut tokens = Vec::new();
    for entry in &script.entries {
        push_entry_tokens(entry, &mut tokens, ctx, None);
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
    ctx: &SemanticTokenContext,
    parent_key: Option<&str>,
) {
    match entry {
        Entry::Assignment(ass) => {
            let is_keyword = ctx.keywords.contains(&ass.key);

            if is_keyword {
                tokens.push(RawToken {
                    line: ass.key_range.start_line,
                    start: ass.key_range.start_col,
                    length: ass.key_range.end_col - ass.key_range.start_col,
                    token_type: TokenType::Keyword as u32,
                });
            } else if let Some(kind) = ctx.entity_names.get(&ass.key) {
                tokens.push(RawToken {
                    line: ass.key_range.start_line,
                    start: ass.key_range.start_col,
                    length: ass.key_range.end_col - ass.key_range.start_col,
                    token_type: entity_kind_to_token_type(*kind),
                });
            } else {
                // Contextual checks based on parent key
                let is_idea_category =
                    parent_key.is_some_and(|p| p == "ideas" || p == "idea_categories");

                if is_idea_category {
                    tokens.push(RawToken {
                        line: ass.key_range.start_line,
                        start: ass.key_range.start_col,
                        length: ass.key_range.end_col - ass.key_range.start_col,
                        token_type: TokenType::Type as u32,
                    });
                }
            }

            // Always emit operator token
            tokens.push(RawToken {
                line: ass.operator_range.start_line,
                start: ass.operator_range.start_col,
                length: ass.operator_range.end_col - ass.operator_range.start_col,
                token_type: TokenType::Operator as u32,
            });

            push_value_tokens(&ass.value, tokens, ctx, Some(&ass.key));
        }
        Entry::Value(val) => {
            push_value_tokens(val, tokens, ctx, parent_key);
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
    ctx: &SemanticTokenContext,
    parent_key: Option<&str>,
) {
    match &val.value {
        Value::String(s) => {
            let is_localization_value =
                parent_key.is_some_and(|k| LOCALIZATION_VALUE_FIELDS.contains(&k));

            if ctx.keywords.contains(s) {
                tokens.push(RawToken {
                    line: val.range.start_line,
                    start: val.range.start_col,
                    length: val.range.end_col - val.range.start_col,
                    token_type: TokenType::Keyword as u32,
                });
            } else if !is_localization_value {
                if let Some(kind) = ctx.entity_names.get(s) {
                    tokens.push(RawToken {
                        line: val.range.start_line,
                        start: val.range.start_col,
                        length: val.range.end_col - val.range.start_col,
                        token_type: entity_kind_to_token_type(*kind),
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
                push_entry_tokens(entry, tokens, ctx, parent_key);
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
                push_entry_tokens(entry, tokens, ctx, parent_key);
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
