use crate::ast::Range;
use std::collections::HashMap;
use std::sync::Arc;
use tower_lsp::lsp_types::{
    Location, Position as LspPosition, Range as LspRange, SymbolInformation, SymbolKind, Url,
};

fn path_to_url(path: &str) -> Url {
    let abs_path = std::path::Path::new(path)
        .canonicalize()
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default().join(path));
    Url::from_file_path(&abs_path).unwrap_or_else(|_| {
        Url::parse(&format!(
            "file://{}",
            abs_path.to_string_lossy().replace("\\", "/")
        ))
        .unwrap()
    })
}

/// Generate workspace symbols from indexed data
pub async fn generate_workspace_symbols(
    query: &str,
    events: &Arc<arc_swap::ArcSwap<HashMap<String, crate::event_scanner::Event>>>,
    ideas: &Arc<arc_swap::ArcSwap<HashMap<String, crate::idea_scanner::Idea>>>,
    traits: &Arc<arc_swap::ArcSwap<HashMap<String, crate::trait_scanner::Trait>>>,
    scripted_triggers: &Arc<
        arc_swap::ArcSwap<HashMap<String, crate::scripted_scanner::ScriptedEntity>>,
    >,
    scripted_effects: &Arc<
        arc_swap::ArcSwap<HashMap<String, crate::scripted_scanner::ScriptedEntity>>,
    >,
    ideologies: &Arc<arc_swap::ArcSwap<HashMap<String, crate::ideology_scanner::Ideology>>>,
    sub_ideologies: &Arc<arc_swap::ArcSwap<HashMap<String, (String, crate::ast::Range, String)>>>,
    sprites: &Arc<arc_swap::ArcSwap<HashMap<String, crate::sprite_scanner::Sprite>>>,
    characters: &Arc<arc_swap::ArcSwap<HashMap<String, crate::character_scanner::Character>>>,
    variables: &Arc<arc_swap::ArcSwap<HashMap<String, Vec<crate::variable_scanner::Variable>>>>,
    achievements: &Arc<arc_swap::ArcSwap<HashMap<String, crate::achievement_scanner::Achievement>>>,
    abilities: &Arc<arc_swap::ArcSwap<HashMap<String, crate::ability_scanner::Ability>>>,
    scripted_locs: &Arc<
        arc_swap::ArcSwap<HashMap<String, crate::scripted_loc_scanner::ScriptedLoc>>,
    >,
    portraits: &Arc<arc_swap::ArcSwap<HashMap<String, crate::portrait_scanner::Portrait>>>,
    localization: &Arc<arc_swap::ArcSwap<HashMap<String, crate::loc_parser::LocEntry>>>,
    states: &Arc<arc_swap::ArcSwap<HashMap<u32, crate::state_scanner::State>>>,
    supply_nodes: &Arc<arc_swap::ArcSwap<Vec<crate::logistics_scanner::SupplyNode>>>,
    railways: &Arc<arc_swap::ArcSwap<Vec<crate::logistics_scanner::Railway>>>,
    map_buildings: &Arc<arc_swap::ArcSwap<Vec<crate::map_object_scanner::MapBuilding>>>,
    unitstacks: &Arc<arc_swap::ArcSwap<Vec<crate::map_object_scanner::UnitStack>>>,
    weather_positions: &Arc<arc_swap::ArcSwap<Vec<crate::map_object_scanner::WeatherPosition>>>,
    adjacencies: &Arc<arc_swap::ArcSwap<Vec<crate::adjacency_scanner::Adjacency>>>,
    adjacency_rules: &Arc<
        arc_swap::ArcSwap<HashMap<String, crate::adjacency_scanner::AdjacencyRule>>,
    >,
    strategic_regions: &Arc<
        arc_swap::ArcSwap<HashMap<u32, crate::strategic_region_scanner::StrategicRegion>>,
    >,
    custom_modifiers: &Arc<arc_swap::ArcSwap<HashMap<String, crate::modifier_scanner::Modifier>>>,
    sounds: &Arc<arc_swap::ArcSwap<HashMap<String, crate::sound_scanner::Sound>>>,
    sound_effects: &Arc<arc_swap::ArcSwap<HashMap<String, crate::sound_scanner::SoundEffect>>>,
    falloffs: &Arc<arc_swap::ArcSwap<HashMap<String, crate::sound_scanner::Falloff>>>,
    sound_categories: &Arc<arc_swap::ArcSwap<HashMap<String, crate::sound_scanner::SoundCategory>>>,
) -> Vec<SymbolInformation> {
    let mut symbols = Vec::new();
    let query_lower = query.to_lowercase();

    // Search custom modifiers
    let modifiers_lock = custom_modifiers.load();
    for (name, modifier) in modifiers_lock.iter() {
        if fuzzy_match(&query_lower, &name.to_lowercase()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.clone(),
                kind: SymbolKind::PROPERTY,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&modifier.path),
                    range: range_to_lsp(&modifier.range),
                },
                container_name: Some("Modifier".to_string()),
            });
        }
    }

    // Search achievements
    let achievements_lock = achievements.load();
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
    let events_lock = events.load();
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
    let ideas_lock = ideas.load();
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
    let traits_lock = traits.load();
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
    let triggers_lock = scripted_triggers.load();
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
    let effects_lock = scripted_effects.load();
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
    let locs_lock = scripted_locs.load();
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
    let states_lock = states.load();
    for (id, state) in states_lock.iter() {
        if fuzzy_match(&query_lower, &id.to_string())
            || fuzzy_match(&query_lower, &state.name.to_lowercase())
        {
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
    let sn_lock = supply_nodes.load();
    for node in sn_lock.iter() {
        let name = format!("Supply Node in Province {}", node.province_id);
        if fuzzy_match(&query_lower, &name.to_lowercase())
            || fuzzy_match(&query_lower, &node.province_id.to_string())
        {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name,
                kind: SymbolKind::OBJECT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&node.path),
                    range: LspRange {
                        start: LspPosition {
                            line: node.start_line,
                            character: 0,
                        },
                        end: LspPosition {
                            line: node.start_line,
                            character: 100,
                        },
                    },
                },
                container_name: Some("Supply Node".to_string()),
            });
        }
    }

    let rw_lock = railways.load();
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
                        start: LspPosition {
                            line: rw.start_line,
                            character: 0,
                        },
                        end: LspPosition {
                            line: rw.start_line,
                            character: 100,
                        },
                    },
                },
                container_name: Some("Railway".to_string()),
            });
        }
    }

    // Search Map Buildings
    let mb_lock = map_buildings.load();
    for mb in mb_lock.iter() {
        let name = format!("Building '{}' in State {}", mb.building_id, mb.state_id);
        if fuzzy_match(&query_lower, &mb.building_id.to_lowercase())
            || fuzzy_match(&query_lower, &mb.state_id.to_string())
        {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name,
                kind: SymbolKind::OBJECT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&mb.path),
                    range: LspRange {
                        start: LspPosition {
                            line: mb.start_line,
                            character: 0,
                        },
                        end: LspPosition {
                            line: mb.start_line,
                            character: 100,
                        },
                    },
                },
                container_name: Some("Map Building".to_string()),
            });
        }
    }

    // Search Unitstacks
    let us_lock = unitstacks.load();
    for us in us_lock.iter() {
        let name = format!("Unitstack {} in Province {}", us.stack_type, us.province_id);
        if fuzzy_match(&query_lower, "unitstack")
            || fuzzy_match(&query_lower, &us.province_id.to_string())
        {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name,
                kind: SymbolKind::OBJECT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&us.path),
                    range: LspRange {
                        start: LspPosition {
                            line: us.start_line,
                            character: 0,
                        },
                        end: LspPosition {
                            line: us.start_line,
                            character: 100,
                        },
                    },
                },
                container_name: Some("Unitstack".to_string()),
            });
        }
    }

    // Search Weather Positions
    let wp_lock = weather_positions.load();
    for wp in wp_lock.iter() {
        let name = format!("Weather Position in Strategic Region {}", wp.region_id);
        if fuzzy_match(&query_lower, "weather")
            || fuzzy_match(&query_lower, &wp.region_id.to_string())
        {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name,
                kind: SymbolKind::OBJECT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&wp.path),
                    range: LspRange {
                        start: LspPosition {
                            line: wp.start_line,
                            character: 0,
                        },
                        end: LspPosition {
                            line: wp.start_line,
                            character: 100,
                        },
                    },
                },
                container_name: Some("Weather Position".to_string()),
            });
        }
    }

    // Search Adjacencies
    let adj_lock = adjacencies.load();
    for adj in adj_lock.iter() {
        let name = format!(
            "Adjacency ({}) {} <-> {}",
            adj.adj_type, adj.start_prov, adj.end_prov
        );
        if fuzzy_match(&query_lower, "adjacency")
            || fuzzy_match(&query_lower, &adj.start_prov.to_string())
            || fuzzy_match(&query_lower, &adj.end_prov.to_string())
        {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name,
                kind: SymbolKind::OBJECT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&adj.path),
                    range: LspRange {
                        start: LspPosition {
                            line: adj.start_line,
                            character: 0,
                        },
                        end: LspPosition {
                            line: adj.start_line,
                            character: 100,
                        },
                    },
                },
                container_name: Some("Adjacency".to_string()),
            });
        }
    }

    // Search Adjacency Rules
    let rule_lock = adjacency_rules.load();
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
    let regions_lock = strategic_regions.load();
    for (id, region) in regions_lock.iter() {
        if fuzzy_match(&query_lower, &id.to_string())
            || fuzzy_match(&query_lower, &region.name.to_lowercase())
        {
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
    let loc_lock = localization.load();
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
            if loc_count > 1000 {
                // Max 1000 loc symbols to avoid performance issues
                break;
            }
        }
    }

    // Search ideologies
    let ideologies_lock = ideologies.load();
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
    let sub_ideologies_lock = sub_ideologies.load();
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
    let sprites_lock = sprites.load();
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
    let characters_lock = characters.load();
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
    let abilities_lock = abilities.load();
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

    // Search portraits
    let portraits_lock = portraits.load();
    for (name, portrait) in portraits_lock.iter() {
        if fuzzy_match(&query_lower, &name.to_lowercase()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.clone(),
                kind: SymbolKind::ENUM,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&portrait.path),
                    range: range_to_lsp(&portrait.range),
                },
                container_name: Some("Portrait".to_string()),
            });
        }
    }

    // Search variables
    let variables_lock = variables.load();
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

    // Search sounds
    let sounds_lock = sounds.load();
    for (name, sound) in sounds_lock.iter() {
        if fuzzy_match(&query_lower, &name.to_lowercase()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.clone(),
                kind: SymbolKind::EVENT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&sound.path),
                    range: range_to_lsp(&sound.range),
                },
                container_name: Some("Sound".to_string()),
            });
        }
    }

    // Search sound effects
    let effects_lock = sound_effects.load();
    for (name, effect) in effects_lock.iter() {
        if fuzzy_match(&query_lower, &name.to_lowercase()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.clone(),
                kind: SymbolKind::EVENT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&effect.path),
                    range: range_to_lsp(&effect.range),
                },
                container_name: Some("Sound Effect".to_string()),
            });
        }
    }

    // Search falloffs
    let falloffs_lock = falloffs.load();
    for (name, falloff) in falloffs_lock.iter() {
        if fuzzy_match(&query_lower, &name.to_lowercase()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.clone(),
                kind: SymbolKind::EVENT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&falloff.path),
                    range: range_to_lsp(&falloff.range),
                },
                container_name: Some("Sound Falloff".to_string()),
            });
        }
    }

    // Search sound categories
    let categories_lock = sound_categories.load();
    for (name, category) in categories_lock.iter() {
        if fuzzy_match(&query_lower, &name.to_lowercase()) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.clone(),
                kind: SymbolKind::EVENT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&category.path),
                    range: range_to_lsp(&category.range),
                },
                container_name: Some("Sound Category".to_string()),
            });
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
