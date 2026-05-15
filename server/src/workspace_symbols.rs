use tower_lsp::lsp_types::{SymbolInformation, SymbolKind, Location, Range as LspRange, Position as LspPosition, Url};
use crate::ast::Range;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

fn path_to_url(path: &str) -> Url {
    let abs_path = std::path::Path::new(path).canonicalize().unwrap_or_else(|_| {
        std::env::current_dir().unwrap_or_default().join(path)
    });
    Url::from_file_path(&abs_path).unwrap_or_else(|_| {
        Url::parse(&format!("file://{}", abs_path.to_string_lossy().replace("\\", "/"))).unwrap()
    })
}

/// Generate workspace symbols from indexed data
pub async fn generate_workspace_symbols(
    query: &str,
    events: &Arc<RwLock<HashMap<String, crate::event_scanner::Event>>>,
    ideas: &Arc<RwLock<HashMap<String, crate::idea_scanner::Idea>>>,
    traits: &Arc<RwLock<HashMap<String, crate::trait_scanner::Trait>>>,
    scripted_triggers: &Arc<RwLock<HashMap<String, crate::scripted_scanner::ScriptedEntity>>>,
    scripted_effects: &Arc<RwLock<HashMap<String, crate::scripted_scanner::ScriptedEntity>>>,
    ideologies: &Arc<RwLock<HashMap<String, crate::ideology_scanner::Ideology>>>,
    sub_ideologies: &Arc<RwLock<HashMap<String, (String, crate::ast::Range, String)>>>,
    sprites: &Arc<RwLock<HashMap<String, crate::sprite_scanner::Sprite>>>,
    characters: &Arc<RwLock<HashMap<String, crate::character_scanner::Character>>>,
    variables: &Arc<RwLock<HashMap<String, Vec<crate::variable_scanner::Variable>>>>,
    achievements: &Arc<RwLock<HashMap<String, crate::achievement_scanner::Achievement>>>,
    abilities: &Arc<RwLock<HashMap<String, crate::ability_scanner::Ability>>>,
    scripted_locs: &Arc<RwLock<HashMap<String, crate::scripted_loc_scanner::ScriptedLoc>>>,
    localization: &Arc<RwLock<HashMap<String, crate::loc_parser::LocEntry>>>,
    states: &Arc<RwLock<HashMap<u32, crate::state_scanner::State>>>,
    supply_nodes: &Arc<RwLock<Vec<crate::logistics_scanner::SupplyNode>>>,
    railways: &Arc<RwLock<Vec<crate::logistics_scanner::Railway>>>,
    map_buildings: &Arc<RwLock<Vec<crate::map_object_scanner::MapBuilding>>>,
    unitstacks: &Arc<RwLock<Vec<crate::map_object_scanner::UnitStack>>>,
    adjacencies: &Arc<RwLock<Vec<crate::adjacency_scanner::Adjacency>>>,
    adjacency_rules: &Arc<RwLock<HashMap<String, crate::adjacency_scanner::AdjacencyRule>>>,
    strategic_regions: &Arc<RwLock<HashMap<u32, crate::strategic_region_scanner::StrategicRegion>>>,
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
                    uri: path_to_url(&achievement.path),
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
                    uri: path_to_url(&event.path),
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
                    uri: path_to_url(&idea.path),
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
                    uri: path_to_url(&trait_data.path),
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
                    uri: path_to_url(&trigger.path),
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
                    uri: path_to_url(&effect.path),
                    range: range_to_lsp(&effect.range),
                },
                container_name: Some("Scripted Effect".to_string()),
            });
        }
    }

    // Search scripted locs
    let locs_lock = scripted_locs.read().await;
    for (name, loc) in locs_lock.iter() {
        if fuzzy_match(&query_lower, &name.to_lowercase()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.clone(),
                kind: SymbolKind::FUNCTION,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&loc.path),
                    range: range_to_lsp(&loc.range),
                },
                container_name: Some("Scripted Localisation".to_string()),
            });
        }
    }

    // Search states
    let states_lock = states.read().await;
    for (id, state) in states_lock.iter() {
        if fuzzy_match(&query_lower, &id.to_string()) || fuzzy_match(&query_lower, &state.name.to_lowercase()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: format!("State {}: {}", id, state.name),
                kind: SymbolKind::OBJECT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&state.path),
                    range: range_to_lsp(&state.range),
                },
                container_name: Some("State".to_string()),
            });
        }
    }

    // Search logistics
    let sn_lock = supply_nodes.read().await;
    for node in sn_lock.iter() {
        let name = format!("Supply Node in Province {}", node.province_id);
        if fuzzy_match(&query_lower, &name.to_lowercase()) || fuzzy_match(&query_lower, &node.province_id.to_string()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name,
                kind: SymbolKind::OBJECT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&node.path),
                    range: LspRange {
                        start: LspPosition { line: node.start_line, character: 0 },
                        end: LspPosition { line: node.start_line, character: 100 },
                    },
                },
                container_name: Some("Supply Node".to_string()),
            });
        }
    }

    let rw_lock = railways.read().await;
    for rw in rw_lock.iter() {
        let name = format!("Railway (Lvl {})", rw.level);
        if fuzzy_match(&query_lower, "railway") {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name,
                kind: SymbolKind::OBJECT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&rw.path),
                    range: LspRange {
                        start: LspPosition { line: rw.start_line, character: 0 },
                        end: LspPosition { line: rw.start_line, character: 100 },
                    },
                },
                container_name: Some("Railway".to_string()),
            });
        }
    }

    // Search Map Buildings
    let mb_lock = map_buildings.read().await;
    for mb in mb_lock.iter() {
        let name = format!("Building '{}' in State {}", mb.building_id, mb.state_id);
        if fuzzy_match(&query_lower, &mb.building_id.to_lowercase()) || fuzzy_match(&query_lower, &mb.state_id.to_string()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name,
                kind: SymbolKind::OBJECT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&mb.path),
                    range: LspRange {
                        start: LspPosition { line: mb.start_line, character: 0 },
                        end: LspPosition { line: mb.start_line, character: 100 },
                    },
                },
                container_name: Some("Map Building".to_string()),
            });
        }
    }

    // Search Unitstacks
    let us_lock = unitstacks.read().await;
    for us in us_lock.iter() {
        let name = format!("Unitstack {} in Province {}", us.stack_type, us.province_id);
        if fuzzy_match(&query_lower, "unitstack") || fuzzy_match(&query_lower, &us.province_id.to_string()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name,
                kind: SymbolKind::OBJECT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&us.path),
                    range: LspRange {
                        start: LspPosition { line: us.start_line, character: 0 },
                        end: LspPosition { line: us.start_line, character: 100 },
                    },
                },
                container_name: Some("Unitstack".to_string()),
            });
        }
    }

    // Search Adjacencies
    let adj_lock = adjacencies.read().await;
    for adj in adj_lock.iter() {
        let name = format!("Adjacency ({}) {} <-> {}", adj.adj_type, adj.start_prov, adj.end_prov);
        if fuzzy_match(&query_lower, "adjacency") || fuzzy_match(&query_lower, &adj.start_prov.to_string()) || fuzzy_match(&query_lower, &adj.end_prov.to_string()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name,
                kind: SymbolKind::OBJECT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&adj.path),
                    range: LspRange {
                        start: LspPosition { line: adj.start_line, character: 0 },
                        end: LspPosition { line: adj.start_line, character: 100 },
                    },
                },
                container_name: Some("Adjacency".to_string()),
            });
        }
    }

    // Search Adjacency Rules
    let rule_lock = adjacency_rules.read().await;
    for (name, rule) in rule_lock.iter() {
        if fuzzy_match(&query_lower, &name.to_lowercase()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.clone(),
                kind: SymbolKind::FUNCTION,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&rule.path),
                    range: range_to_lsp(&rule.range),
                },
                container_name: Some("Adjacency Rule".to_string()),
            });
        }
    }

    // Search Strategic Regions
    let regions_lock = strategic_regions.read().await;
    for (id, region) in regions_lock.iter() {
        if fuzzy_match(&query_lower, &id.to_string()) || fuzzy_match(&query_lower, &region.name.to_lowercase()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: format!("Strategic Region {}: {}", id, region.name),
                kind: SymbolKind::OBJECT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&region.path),
                    range: range_to_lsp(&region.range),
                },
                container_name: Some("Strategic Region".to_string()),
            });
        }
    }

    // Search localization
    // Note: Localization can be extremely large. We only return matches if they fuzzy match
    let loc_lock = localization.read().await;
    // Limit to prevent overwhelming the client
    let mut loc_count = 0;
    for (name, loc) in loc_lock.iter() {
        if fuzzy_match(&query_lower, &name.to_lowercase()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.clone(),
                kind: SymbolKind::STRING,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&loc.path),
                    range: range_to_lsp(&loc.range),
                },
                container_name: Some("Localisation".to_string()),
            });
            loc_count += 1;
            if loc_count > 1000 { // Max 1000 loc symbols to avoid performance issues
                break;
            }
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
                    uri: path_to_url(&ideology.path),
                    range: range_to_lsp(&ideology.range),
                },
                container_name: Some("Ideology".to_string()),
            });
        }
    }

    // Search sub-ideologies
    let sub_ideologies_lock = sub_ideologies.read().await;
    for (name, (parent, range, path)) in sub_ideologies_lock.iter() {
        if fuzzy_match(&query_lower, &name.to_lowercase()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.clone(),
                kind: SymbolKind::ENUM_MEMBER,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(path),
                    range: range_to_lsp(range),
                },
                container_name: Some(format!("Sub-Ideology ({})", parent)),
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
                    uri: path_to_url(&sprite.path),
                    range: range_to_lsp(&sprite.range),
                },
                container_name: Some("Sprite".to_string()),
            });
        }
    }

    // Search characters
    let characters_lock = characters.read().await;
    for (name, character) in characters_lock.iter() {
        if fuzzy_match(&query_lower, &name.to_lowercase()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.clone(),
                kind: SymbolKind::STRUCT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&character.path),
                    range: range_to_lsp(&character.range),
                },
                container_name: Some("Character".to_string()),
            });
        }
    }

    // Search abilities
    let abilities_lock = abilities.read().await;
    for (name, ability) in abilities_lock.iter() {
        if fuzzy_match(&query_lower, &name.to_lowercase()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.clone(),
                kind: SymbolKind::FUNCTION,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&ability.path),
                    range: range_to_lsp(&ability.range),
                },
                container_name: Some("Leader Ability".to_string()),
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
                        uri: path_to_url(&var.path),
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
