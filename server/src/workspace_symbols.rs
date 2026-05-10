use tower_lsp::lsp_types::{SymbolInformation, SymbolKind, Location, Range as LspRange, Position as LspPosition, Url};
use crate::ast::Range;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Generate workspace symbols from indexed data
pub async fn generate_workspace_symbols(
    query: &str,
    events: &Arc<RwLock<HashMap<String, crate::event_scanner::Event>>>,
    ideas: &Arc<RwLock<HashMap<String, crate::idea_scanner::Idea>>>,
    traits: &Arc<RwLock<HashMap<String, crate::trait_scanner::Trait>>>,
    scripted_triggers: &Arc<RwLock<HashMap<String, crate::scripted_scanner::ScriptedEntity>>>,
    scripted_effects: &Arc<RwLock<HashMap<String, crate::scripted_scanner::ScriptedEntity>>>,
    ideologies: &Arc<RwLock<HashMap<String, crate::ideology_scanner::Ideology>>>,
    sprites: &Arc<RwLock<HashMap<String, crate::sprite_scanner::Sprite>>>,
    variables: &Arc<RwLock<HashMap<String, Vec<crate::variable_scanner::Variable>>>>,
    achievements: &Arc<RwLock<HashMap<String, crate::achievement_scanner::Achievement>>>,
) -> Vec<SymbolInformation> {
    let mut symbols = Vec::new();
    let query_lower = query.to_lowercase();

    // Search achievements
    let achievements_lock = achievements.read().await;
    for (name, achievement) in achievements_lock.iter() {
        if fuzzy_match(&query_lower, &name.to_lowercase()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.clone(),
                kind: SymbolKind::EVENT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: Url::from_file_path(&achievement.path).unwrap_or_else(|_| {
                        Url::parse(&format!("file://{}", achievement.path)).unwrap()
                    }),
                    range: range_to_lsp(&achievement.range),
                },
                container_name: Some("Achievement".to_string()),
            });
        }
    }

    // Search events
    let events_lock = events.read().await;
    for (id, event) in events_lock.iter() {
        if fuzzy_match(&query_lower, &id.to_lowercase()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: id.clone(),
                kind: SymbolKind::EVENT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: Url::from_file_path(&event.path).unwrap_or_else(|_| {
                        Url::parse(&format!("file://{}", event.path)).unwrap()
                    }),
                    range: range_to_lsp(&event.range),
                },
                container_name: Some(format!("{:?}", event.event_type)),
            });
        }
    }

    // Search ideas
    let ideas_lock = ideas.read().await;
    for (name, idea) in ideas_lock.iter() {
        if fuzzy_match(&query_lower, &name.to_lowercase()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.clone(),
                kind: SymbolKind::CLASS,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: Url::from_file_path(&idea.path).unwrap_or_else(|_| {
                        Url::parse(&format!("file://{}", idea.path)).unwrap()
                    }),
                    range: range_to_lsp(&idea.range),
                },
                container_name: Some(format!("{:?}", idea.category)),
            });
        }
    }

    // Search traits
    let traits_lock = traits.read().await;
    for (name, trait_data) in traits_lock.iter() {
        if fuzzy_match(&query_lower, &name.to_lowercase()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.clone(),
                kind: SymbolKind::STRUCT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: Url::from_file_path(&trait_data.path).unwrap_or_else(|_| {
                        Url::parse(&format!("file://{}", trait_data.path)).unwrap()
                    }),
                    range: range_to_lsp(&trait_data.range),
                },
                container_name: Some(format!("{:?}", trait_data.trait_type)),
            });
        }
    }

    // Search scripted triggers
    let triggers_lock = scripted_triggers.read().await;
    for (name, trigger) in triggers_lock.iter() {
        if fuzzy_match(&query_lower, &name.to_lowercase()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.clone(),
                kind: SymbolKind::FUNCTION,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: Url::from_file_path(&trigger.path).unwrap_or_else(|_| {
                        Url::parse(&format!("file://{}", trigger.path)).unwrap()
                    }),
                    range: range_to_lsp(&trigger.range),
                },
                container_name: Some("Scripted Trigger".to_string()),
            });
        }
    }

    // Search scripted effects
    let effects_lock = scripted_effects.read().await;
    for (name, effect) in effects_lock.iter() {
        if fuzzy_match(&query_lower, &name.to_lowercase()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.clone(),
                kind: SymbolKind::FUNCTION,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: Url::from_file_path(&effect.path).unwrap_or_else(|_| {
                        Url::parse(&format!("file://{}", effect.path)).unwrap()
                    }),
                    range: range_to_lsp(&effect.range),
                },
                container_name: Some("Scripted Effect".to_string()),
            });
        }
    }

    // Search ideologies
    let ideologies_lock = ideologies.read().await;
    for (name, ideology) in ideologies_lock.iter() {
        if fuzzy_match(&query_lower, &name.to_lowercase()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.clone(),
                kind: SymbolKind::ENUM,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: Url::from_file_path(&ideology.path).unwrap_or_else(|_| {
                        Url::parse(&format!("file://{}", ideology.path)).unwrap()
                    }),
                    range: range_to_lsp(&ideology.range),
                },
                container_name: Some("Ideology".to_string()),
            });
        }
    }

    // Search sprites
    let sprites_lock = sprites.read().await;
    for (name, sprite) in sprites_lock.iter() {
        if fuzzy_match(&query_lower, &name.to_lowercase()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.clone(),
                kind: SymbolKind::CONSTANT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: Url::from_file_path(&sprite.path).unwrap_or_else(|_| {
                        Url::parse(&format!("file://{}", sprite.path)).unwrap()
                    }),
                    range: range_to_lsp(&sprite.range),
                },
                container_name: Some("Sprite".to_string()),
            });
        }
    }

    // Search variables
    let variables_lock = variables.read().await;
    for (name, var_list) in variables_lock.iter() {
        if fuzzy_match(&query_lower, &name.to_lowercase()) {
            // Add the first occurrence
            if let Some(var) = var_list.first() {
                #[allow(deprecated)]
                symbols.push(SymbolInformation {
                    name: name.clone(),
                    kind: SymbolKind::VARIABLE,
                    tags: None,
                    deprecated: None,
                    location: Location {
                        uri: Url::from_file_path(&var.path).unwrap_or_else(|_| {
                            Url::parse(&format!("file://{}", var.path)).unwrap()
                        }),
                        range: range_to_lsp(&var.range),
                    },
                    container_name: Some("Variable".to_string()),
                });
            }
        }
    }

    symbols
}

/// Fuzzy match for symbol search
fn fuzzy_match(query: &str, target: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let query_lower = query.to_lowercase();
    let target_lower = target.to_lowercase();

    // Exact substring match
    if target_lower.contains(&query_lower) {
        return true;
    }

    // Fuzzy match: all characters in query appear in order in target
    let mut target_chars = target_lower.chars();
    for query_char in query_lower.chars() {
        if !target_chars.any(|c| c == query_char) {
            return false;
        }
    }

    true
}

/// Convert AST Range to LSP Range
fn range_to_lsp(range: &Range) -> LspRange {
    LspRange {
        start: LspPosition {
            line: range.start_line,
            character: range.start_col,
        },
        end: LspPosition {
            line: range.end_line,
            character: range.end_col,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match() {
        assert!(fuzzy_match("", "anything"));
        assert!(fuzzy_match("test", "test"));
        assert!(fuzzy_match("test", "my_test_event"));
        assert!(fuzzy_match("mte", "my_test_event"));
        assert!(!fuzzy_match("xyz", "my_test_event"));
    }

    #[test]
    fn test_fuzzy_match_case_insensitive() {
        assert!(fuzzy_match("test", "TEST"));
        assert!(fuzzy_match("test", "MyTestEvent"));
    }
}
