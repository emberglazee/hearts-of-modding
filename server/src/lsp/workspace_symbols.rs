use crate::utils::fs_util::fuzzy_match;
use crate::utils::lsp_convert::RangeMapper;
use std::collections::HashMap;

use tower_lsp_server::ls_types::{
    Location, Position as LspPosition, Range as LspRange, SymbolInformation, SymbolKind, Uri,
};

fn path_to_url(path: &str) -> Uri {
    let abs_path = std::path::Path::new(path)
        .canonicalize()
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default().join(path));
    Uri::from_file_path(&abs_path).unwrap_or_else(|| {
        format!("file://{}", abs_path.to_string_lossy().replace("\\", "/"))
            .parse::<Uri>()
            .unwrap()
    })
}

/// Generate workspace symbols from indexed data
pub async fn generate_workspace_symbols(
    query: &str,
    data: &crate::ScannerData,
) -> Vec<SymbolInformation> {
    let mut symbols = Vec::new();
    // Cache a UTF-16 RangeMapper per source file (paths are interned, so one per schema).
    let mut mapper_cache: HashMap<String, RangeMapper> = HashMap::new();
    let query_lower = query.to_ascii_lowercase();

    // Search custom modifiers
    let modifiers = &data.custom_modifiers;
    for entry in modifiers.iter() {
        let name = entry.key();
        let modifier = entry.value();
        if fuzzy_match(&query_lower, name) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::PROPERTY,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&modifier.path),
                    range: mapper_for_path(&mut mapper_cache, &modifier.path)
                        .range(&modifier.range),
                },
                container_name: Some("Modifier".to_string()),
            });
        }
    }

    // Search achievements
    let achievements = &data.achievements;
    for entry in achievements.iter() {
        let name = entry.key();
        let achievement = entry.value();
        if fuzzy_match(&query_lower, name) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::EVENT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&achievement.path),
                    range: mapper_for_path(&mut mapper_cache, &achievement.path)
                        .range(&achievement.range),
                },
                container_name: Some("Achievement".to_string()),
            });
        }
    }

    // Search events
    let events = &data.events;
    for entry in events.iter() {
        let id = entry.key();
        let event = entry.value();
        if fuzzy_match(&query_lower, id) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: id.to_string(),
                kind: SymbolKind::EVENT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&event.path),
                    range: mapper_for_path(&mut mapper_cache, &event.path).range(&event.range),
                },
                container_name: Some(format!("{:?}", event.event_type)),
            });
        }
    }

    // Search ideas
    let ideas = &data.ideas;
    for entry in ideas.iter() {
        let name = entry.key();
        let idea = entry.value();
        if fuzzy_match(&query_lower, name) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::CLASS,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&idea.path),
                    range: mapper_for_path(&mut mapper_cache, &idea.path).range(&idea.range),
                },
                container_name: Some(format!("{:?}", idea.category)),
            });
        }
    }

    // Search traits
    let traits = &data.traits;
    for entry in traits.iter() {
        let name = entry.key();
        let trait_data = entry.value();
        if fuzzy_match(&query_lower, name) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::STRUCT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&trait_data.path),
                    range: mapper_for_path(&mut mapper_cache, &trait_data.path)
                        .range(&trait_data.range),
                },
                container_name: Some(format!("{:?}", trait_data.trait_type)),
            });
        }
    }

    // Search scripted triggers
    let triggers = &data.scripted_triggers;
    for entry in triggers.iter() {
        let name = entry.key();
        let trigger = entry.value();
        if fuzzy_match(&query_lower, name) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::FUNCTION,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&trigger.path),
                    range: mapper_for_path(&mut mapper_cache, &trigger.path).range(&trigger.range),
                },
                container_name: Some("Scripted Trigger".to_string()),
            });
        }
    }

    // Search scripted effects
    let effects = &data.scripted_effects;
    for entry in effects.iter() {
        let name = entry.key();
        let effect = entry.value();
        if fuzzy_match(&query_lower, name) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::FUNCTION,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&effect.path),
                    range: mapper_for_path(&mut mapper_cache, &effect.path).range(&effect.range),
                },
                container_name: Some("Scripted Effect".to_string()),
            });
        }
    }

    // Search scripted locs
    let locs = &data.scripted_locs;
    for entry in locs.iter() {
        let name = entry.key();
        let loc = entry.value();
        if fuzzy_match(&query_lower, name) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::FUNCTION,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&loc.path),
                    range: mapper_for_path(&mut mapper_cache, &loc.path).range(&loc.range),
                },
                container_name: Some("Scripted Localisation".to_string()),
            });
        }
    }

    // Search states
    let states = &data.states;
    for entry in states.iter() {
        let id = entry.key();
        let state = entry.value();
        if fuzzy_match(&query_lower, &id.to_string()) || fuzzy_match(&query_lower, &state.name) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: format!("State {}: {}", id, state.name),
                kind: SymbolKind::OBJECT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&state.path),
                    range: mapper_for_path(&mut mapper_cache, &state.path).range(&state.range),
                },
                container_name: Some("State".to_string()),
            });
        }
    }

    // Search logistics
    let sn_lock = data.supply_nodes();
    for node in sn_lock.iter() {
        let name = format!("Supply Node in Province {}", node.province_id);
        if fuzzy_match(&query_lower, &name)
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

    let rw_lock = data.railways();
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
    let mb_lock = data.map_buildings();
    for mb in mb_lock.iter() {
        let name = format!("Building '{}' in State {}", mb.building_id, mb.state_id);
        if fuzzy_match(&query_lower, &mb.building_id)
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
    let us_lock = data.unitstacks();
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
    let wp_lock = data.weather_positions();
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
    let adj_lock = data.adjacencies();
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
    let rules = &data.adjacency_rules;
    for entry in rules.iter() {
        let name = entry.key();
        let rule = entry.value();
        if fuzzy_match(&query_lower, name) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::FUNCTION,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&rule.path),
                    range: mapper_for_path(&mut mapper_cache, &rule.path).range(&rule.range),
                },
                container_name: Some("Adjacency Rule".to_string()),
            });
        }
    }

    // Search Strategic Regions
    let regions = &data.strategic_regions;
    for entry in regions.iter() {
        let id = entry.key();
        let region = entry.value();
        if fuzzy_match(&query_lower, &id.to_string()) || fuzzy_match(&query_lower, &region.name) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: format!("Strategic Region {}: {}", id, region.name),
                kind: SymbolKind::OBJECT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&region.path),
                    range: mapper_for_path(&mut mapper_cache, &region.path).range(&region.range),
                },
                container_name: Some("Strategic Region".to_string()),
            });
        }
    }

    // Search localization
    // Note: Localization can be extremely large. We only return matches if they fuzzy match
    let loc = &data.localization;
    // Limit to prevent overwhelming the client
    let mut loc_count = 0;
    for entry in loc.iter() {
        let name = entry.key();
        let loc_entry = entry.value();
        if fuzzy_match(&query_lower, name) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::STRING,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&loc_entry.path),
                    range: mapper_for_path(&mut mapper_cache, &loc_entry.path)
                        .range(&loc_entry.range),
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
    let ideologies = &data.ideologies;
    for entry in ideologies.iter() {
        let name = entry.key();
        let ideology = entry.value();
        if fuzzy_match(&query_lower, name) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::ENUM,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&ideology.path),
                    range: mapper_for_path(&mut mapper_cache, &ideology.path)
                        .range(&ideology.range),
                },
                container_name: Some("Ideology".to_string()),
            });
        }
    }

    // Search sub-ideologies
    let sub_ideologies = &data.sub_ideologies;
    for entry in sub_ideologies.iter() {
        let name = entry.key();
        let (parent, range, path) = entry.value().resolve();
        if fuzzy_match(&query_lower, name) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::ENUM_MEMBER,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(path),
                    range: mapper_for_path(&mut mapper_cache, path).range(range),
                },
                container_name: Some(format!("Sub-Ideology ({})", parent)),
            });
        }
    }

    // Search sprites
    let sprites = &data.sprites;
    for entry in sprites.iter() {
        let name = entry.key();
        let sprite = entry.value();
        if fuzzy_match(&query_lower, name) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::CONSTANT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&sprite.path),
                    range: mapper_for_path(&mut mapper_cache, &sprite.path).range(&sprite.range),
                },
                container_name: Some("Sprite".to_string()),
            });
        }
    }

    // Search characters
    let characters = &data.characters;
    for entry in characters.iter() {
        let name = entry.key();
        let character = entry.value();
        if fuzzy_match(&query_lower, name) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::STRUCT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&character.path),
                    range: mapper_for_path(&mut mapper_cache, &character.path)
                        .range(&character.range),
                },
                container_name: Some("Character".to_string()),
            });
        }
    }

    // Search abilities
    let abilities = &data.abilities;
    for entry in abilities.iter() {
        let name = entry.key();
        let ability = entry.value();
        if fuzzy_match(&query_lower, name) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::FUNCTION,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&ability.path),
                    range: mapper_for_path(&mut mapper_cache, &ability.path).range(&ability.range),
                },
                container_name: Some("Leader Ability".to_string()),
            });
        }
    }

    // Search portraits
    let portraits = &data.portraits;
    for entry in portraits.iter() {
        let name = entry.key();
        let portrait = entry.value();
        if fuzzy_match(&query_lower, name) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::ENUM,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&portrait.path),
                    range: mapper_for_path(&mut mapper_cache, &portrait.path)
                        .range(&portrait.range),
                },
                container_name: Some("Portrait".to_string()),
            });
        }
    }

    // Search color codes
    let color_codes = &data.color_codes;
    for entry in color_codes.iter() {
        let symbol = entry.key();
        let code = entry.value();
        let display = format!(
            "§{} — RGB({}, {}, {})",
            symbol, code.rgb.0, code.rgb.1, code.rgb.2
        );
        if fuzzy_match(&query_lower, symbol) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: display,
                kind: SymbolKind::CONSTANT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&code.path),
                    range: mapper_for_path(&mut mapper_cache, &code.path).range(&code.range),
                },
                container_name: Some("Color Code".to_string()),
            });
        }
    }

    // Search variables
    let variables = &data.variables;
    for entry in variables.iter() {
        let name = entry.key();
        let var_list = entry.value();
        if fuzzy_match(&query_lower, name) {
            // Add the first occurrence
            if let Some(var) = var_list.first() {
                #[allow(deprecated)]
                symbols.push(SymbolInformation {
                    name: name.to_string(),
                    kind: SymbolKind::VARIABLE,
                    tags: None,
                    deprecated: None,
                    location: Location {
                        uri: path_to_url(&var.path),
                        range: mapper_for_path(&mut mapper_cache, &var.path).range(&var.range),
                    },
                    container_name: Some("Variable".to_string()),
                });
            }
        }
    }

    // Search sounds
    let sounds = &data.sounds;
    for entry in sounds.iter() {
        let name = entry.key();
        let sound = entry.value();
        if fuzzy_match(&query_lower, name) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::EVENT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&sound.path),
                    range: mapper_for_path(&mut mapper_cache, &sound.path).range(&sound.range),
                },
                container_name: Some("Sound".to_string()),
            });
        }
    }

    // Search sound effects
    let sound_effects = &data.sound_effects;
    for entry in sound_effects.iter() {
        let name = entry.key();
        let effect = entry.value();
        if fuzzy_match(&query_lower, name) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::EVENT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&effect.path),
                    range: mapper_for_path(&mut mapper_cache, &effect.path).range(&effect.range),
                },
                container_name: Some("Sound Effect".to_string()),
            });
        }
    }

    // Search falloffs
    let falloffs = &data.falloffs;
    for entry in falloffs.iter() {
        let name = entry.key();
        let falloff = entry.value();
        if fuzzy_match(&query_lower, name) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::EVENT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&falloff.path),
                    range: mapper_for_path(&mut mapper_cache, &falloff.path).range(&falloff.range),
                },
                container_name: Some("Sound Falloff".to_string()),
            });
        }
    }

    // Search sound categories
    let sound_categories = &data.sound_categories;
    for entry in sound_categories.iter() {
        let name = entry.key();
        let category = entry.value();
        if fuzzy_match(&query_lower, name) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::EVENT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&category.path),
                    range: mapper_for_path(&mut mapper_cache, &category.path)
                        .range(&category.range),
                },
                container_name: Some("Sound Category".to_string()),
            });
        }
    }

    // Search technologies
    let technologies = &data.technologies;
    for entry in technologies.iter() {
        let name = entry.key();
        let tech = entry.value();
        if fuzzy_match(&query_lower, name) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::OBJECT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&tech.path),
                    range: mapper_for_path(&mut mapper_cache, &tech.path).range(&tech.range),
                },
                container_name: Some("Technology".to_string()),
            });
        }
    }

    // Search technology tags (categories + folders)
    let technology_tags = &data.technology_tags;
    for entry in technology_tags.iter() {
        let name = entry.key();
        let tag = entry.value();
        if fuzzy_match(&query_lower, name) {
            use crate::scanner::technology_tags_scanner::TechnologyTagKind;
            let container = match tag.tag_kind {
                TechnologyTagKind::Category => "Technology Category",
                TechnologyTagKind::Folder => "Technology Folder",
            };
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::NAMESPACE,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&tag.path),
                    range: mapper_for_path(&mut mapper_cache, &tag.path).range(&tag.range),
                },
                container_name: Some(container.to_string()),
            });
        }
    }

    // Search unit types (common/units/*.txt)
    let unit_types = &data.unit_types;
    for entry in unit_types.iter() {
        let name = entry.key();
        let unit = entry.value();
        if fuzzy_match(&query_lower, name) {
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::CLASS,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: path_to_url(&unit.path),
                    range: mapper_for_path(&mut mapper_cache, &unit.path).range(&unit.range),
                },
                container_name: Some(format!(
                    "Unit Type ({})",
                    unit.group.as_deref().unwrap_or("unknown")
                )),
            });
        }
    }

    symbols
}

/// Get (or lazily insert) the UTF-16 `RangeMapper` for a source file path.
/// Each indexed entry stores byte-based ranges plus its source file path; we
/// rebuild the mapper on first use per file to convert to UTF-16 columns.
fn mapper_for_path<'a>(cache: &'a mut HashMap<String, RangeMapper>, path: &str) -> &'a RangeMapper {
    cache.entry(path.to_string()).or_insert_with(|| {
        let source = std::fs::read_to_string(path).unwrap_or_default();
        RangeMapper::new(&source)
    })
}
