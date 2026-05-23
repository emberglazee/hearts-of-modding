use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use crate::parser;
use crate::scope;
use crate::scope_context::{find_context_at, find_scope_context_at};
use crate::Backend;

impl Backend {
    pub(crate) async fn handle_completion(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;

        // Handle localization files
        if uri.ends_with(".yml") {
            if let Some(content) = self.documents.get(&uri) {
                let lines: Vec<&str> = content.lines().collect();
                if let Some(line) = lines.get(position.line as usize) {
                    let prefix = &line[..position.character as usize];

                    // Check if we are inside a bracketed scope [Root.GetTag]
                    if let Some(bracket_start) = prefix.rfind('[') {
                        if prefix.rfind(']').is_none_or(|i| i < bracket_start) {
                            let _inner_prefix = &prefix[bracket_start + 1..];
                            let mut items = Vec::new();

                            // Provide scopes, commands, and event targets
                            for scope in crate::SCOPES.iter() {
                                items.push(CompletionItem {
                                    label: scope.to_string(),
                                    kind: Some(CompletionItemKind::CLASS),
                                    detail: Some("Paradox Scope".to_string()),
                                    ..Default::default()
                                });
                            }
                            for command in crate::LOC_COMMANDS.iter() {
                                items.push(CompletionItem {
                                    label: command.to_string(),
                                    kind: Some(CompletionItemKind::FUNCTION),
                                    detail: Some("Localization Command".to_string()),
                                    ..Default::default()
                                });
                            }
                            let target_map = self.event_targets.load();
                            for target_name in target_map.keys() {
                                items.push(CompletionItem {
                                    label: target_name.clone(),
                                    kind: Some(CompletionItemKind::VARIABLE),
                                    detail: Some("Event Target".to_string()),
                                    ..Default::default()
                                });
                            }

                            return Ok(Some(CompletionResponse::Array(items)));
                        }
                    }
                }
            }
            return Ok(None);
        }

        // Handle adjacency files
        {
            let map_config = crate::map_config::get_map_config(std::path::Path::new("."));
            if uri.ends_with(&map_config.adjacencies) {
                if let Some(content) = self.documents.get(&uri) {
                    if let Some(line) = content.lines().nth(position.line as usize) {
                        let parts: Vec<&str> = line.split(';').collect();
                        let mut current_col = 0;
                        let mut hovered_index = None;
                        for (i, part) in parts.iter().enumerate() {
                            let end_col = current_col + part.len() as u32;
                            if position.character >= current_col && position.character <= end_col {
                                hovered_index = Some(i);
                                break;
                            }
                            current_col = end_col + 1;
                        }

                        if let Some(8) = hovered_index {
                            let mut items = Vec::new();
                            let rules = self.adjacency_rules.load();
                            for rule_name in rules.keys() {
                                items.push(CompletionItem {
                                    label: rule_name.clone(),
                                    kind: Some(CompletionItemKind::ENUM),
                                    detail: Some("Adjacency Rule".to_string()),
                                    ..Default::default()
                                });
                            }
                            return Ok(Some(CompletionResponse::Array(items)));
                        }
                    }
                }
                return Ok(None);
            }
        }
        // Handle adjacency rules file
        if uri.ends_with("adjacency_rules.txt") {
            if let Some(content) = self.documents.get(&uri) {
                let (script, _) = parser::parse_script(&content);
                if let Some(context_key) = find_context_at(&script, position) {
                    let key_lower = context_key.to_lowercase();
                    let mut items = Vec::new();
                    if key_lower == "adjacency_rule" {
                        for f in [
                            "name",
                            "required_provinces",
                            "is_disabled",
                            "icon",
                            "contested",
                            "friend",
                            "enemy",
                            "neutral",
                        ] {
                            items.push(CompletionItem {
                                label: f.to_string(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                ..Default::default()
                            });
                        }
                    } else if ["contested", "friend", "enemy", "neutral"]
                        .contains(&key_lower.as_str())
                    {
                        for f in ["army", "navy", "submarine", "trade"] {
                            items.push(CompletionItem {
                                label: f.to_string(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                ..Default::default()
                            });
                        }
                    }
                    if !items.is_empty() {
                        return Ok(Some(CompletionResponse::Array(items)));
                    }
                } else {
                    return Ok(Some(CompletionResponse::Array(vec![CompletionItem {
                        label: "adjacency_rule".to_string(),
                        kind: Some(CompletionItemKind::CLASS),
                        ..Default::default()
                    }])));
                }
            }
        }

        // Handle music/sound files
        let is_asset_file = uri.ends_with(".asset");
        let is_music_file = is_asset_file || uri.contains("/music/");
        let is_sound_file = is_asset_file || uri.contains("/sound/");

        if is_music_file || is_sound_file {
            if let Some(content) = self.documents.get(&uri) {
                {
                    let (script, _) = parser::parse_script(&content);
                    if let Some(context_key) = find_context_at(&script, position) {
                        let mut completion_items = Vec::new();
                        let key_lower = context_key.to_lowercase();

                        if key_lower == "music" {
                            if uri.ends_with(".asset") {
                                completion_items.push(CompletionItem {
                                    label: "name".to_string(),
                                    kind: Some(CompletionItemKind::PROPERTY),
                                    detail: Some("Track ID".to_string()),
                                    ..Default::default()
                                });
                                completion_items.push(CompletionItem {
                                    label: "file".to_string(),
                                    kind: Some(CompletionItemKind::PROPERTY),
                                    detail: Some("OGG Filename".to_string()),
                                    ..Default::default()
                                });
                                completion_items.push(CompletionItem {
                                    label: "volume".to_string(),
                                    kind: Some(CompletionItemKind::PROPERTY),
                                    detail: Some("Volume Multiplier".to_string()),
                                    ..Default::default()
                                });
                            } else {
                                completion_items.push(CompletionItem {
                                    label: "song".to_string(),
                                    kind: Some(CompletionItemKind::PROPERTY),
                                    detail: Some("Song ID".to_string()),
                                    ..Default::default()
                                });
                                completion_items.push(CompletionItem {
                                    label: "chance".to_string(),
                                    kind: Some(CompletionItemKind::PROPERTY),
                                    detail: Some("Weighting logic".to_string()),
                                    ..Default::default()
                                });
                            }
                        } else if key_lower == "sound" {
                            completion_items.push(CompletionItem {
                                label: "name".to_string(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                ..Default::default()
                            });
                            completion_items.push(CompletionItem {
                                label: "file".to_string(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                ..Default::default()
                            });
                            completion_items.push(CompletionItem {
                                label: "always_load".to_string(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                ..Default::default()
                            });
                            completion_items.push(CompletionItem {
                                label: "volume".to_string(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                ..Default::default()
                            });
                        } else if key_lower == "soundeffect" {
                            completion_items.push(CompletionItem {
                                label: "name".to_string(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                ..Default::default()
                            });
                            completion_items.push(CompletionItem {
                                label: "falloff".to_string(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                ..Default::default()
                            });
                            completion_items.push(CompletionItem {
                                label: "sounds".to_string(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                ..Default::default()
                            });
                            completion_items.push(CompletionItem {
                                label: "loop".to_string(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                ..Default::default()
                            });
                            completion_items.push(CompletionItem {
                                label: "is3d".to_string(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                ..Default::default()
                            });
                            completion_items.push(CompletionItem {
                                label: "volume".to_string(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                ..Default::default()
                            });
                        } else if key_lower == "falloff" {
                            completion_items.push(CompletionItem {
                                label: "name".to_string(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                ..Default::default()
                            });
                            completion_items.push(CompletionItem {
                                label: "min_distance".to_string(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                ..Default::default()
                            });
                            completion_items.push(CompletionItem {
                                label: "max_distance".to_string(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                ..Default::default()
                            });
                            completion_items.push(CompletionItem {
                                label: "height_scale".to_string(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                ..Default::default()
                            });
                        } else if key_lower == "category" {
                            completion_items.push(CompletionItem {
                                label: "name".to_string(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                ..Default::default()
                            });
                            completion_items.push(CompletionItem {
                                label: "soundeffects".to_string(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                ..Default::default()
                            });
                            completion_items.push(CompletionItem {
                                label: "compressor".to_string(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                ..Default::default()
                            });
                        } else if key_lower == "chance" || key_lower == "modifier" {
                            completion_items.push(CompletionItem {
                                label: "factor".to_string(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                ..Default::default()
                            });
                            completion_items.push(CompletionItem {
                                label: "add".to_string(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                ..Default::default()
                            });
                            completion_items.push(CompletionItem {
                                label: "base".to_string(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                ..Default::default()
                            });
                            if key_lower == "chance" {
                                completion_items.push(CompletionItem {
                                    label: "modifier".to_string(),
                                    kind: Some(CompletionItemKind::CLASS),
                                    ..Default::default()
                                });
                            }
                        }

                        if !completion_items.is_empty() {
                            return Ok(Some(CompletionResponse::Array(completion_items)));
                        }
                    } else {
                        // Top level
                        let mut top_items = Vec::new();
                        if is_music_file {
                            top_items.push(CompletionItem {
                                label: "music".to_string(),
                                kind: Some(CompletionItemKind::CLASS),
                                ..Default::default()
                            });
                            if !uri.ends_with(".asset") {
                                top_items.push(CompletionItem {
                                    label: "music_station".to_string(),
                                    kind: Some(CompletionItemKind::PROPERTY),
                                    ..Default::default()
                                });
                            }
                        }
                        if is_sound_file {
                            top_items.push(CompletionItem {
                                label: "sound".to_string(),
                                kind: Some(CompletionItemKind::CLASS),
                                ..Default::default()
                            });
                            top_items.push(CompletionItem {
                                label: "soundeffect".to_string(),
                                kind: Some(CompletionItemKind::CLASS),
                                ..Default::default()
                            });
                            top_items.push(CompletionItem {
                                label: "falloff".to_string(),
                                kind: Some(CompletionItemKind::CLASS),
                                ..Default::default()
                            });
                            top_items.push(CompletionItem {
                                label: "category".to_string(),
                                kind: Some(CompletionItemKind::CLASS),
                                ..Default::default()
                            });
                        }
                        return Ok(Some(CompletionResponse::Array(top_items)));
                    }
                }
            }
        }

        let mut current_scopes = vec![scope::Scope::Global];

        // Try to find context for HOI4 scripts
        if let Some(content) = self.documents.get(&uri) {
            {
                let (script, _) = parser::parse_script(&content);
                let achievements = self.achievements.load();
                let (ctx, scopes) = find_scope_context_at(&script, position, &achievements);
                current_scopes = scopes;
                if let Some(context_key) = ctx {
                    if context_key.to_lowercase().contains("color") {
                        let color_items = vec![
                            CompletionItem {
                                label: "rgb".to_string(),
                                kind: Some(CompletionItemKind::KEYWORD),
                                detail: Some("RGB Color Format".to_string()),
                                ..Default::default()
                            },
                            CompletionItem {
                                label: "hsv".to_string(),
                                kind: Some(CompletionItemKind::KEYWORD),
                                detail: Some("HSV Color Format".to_string()),
                                ..Default::default()
                            },
                        ];
                        return Ok(Some(CompletionResponse::Array(color_items)));
                    }
                }
            }
        }

        let mut items = Vec::new();

        let current_scope = *current_scopes.last().unwrap_or(&scope::Scope::Global);

        for trigger in crate::TRIGGERS.values() {
            if !trigger.scopes.contains(&scope::Scope::Unknown)
                && !trigger.scopes.contains(&current_scope)
                && !trigger.scopes.contains(&scope::Scope::Global)
            {
                continue;
            }
            items.push(CompletionItem {
                label: trigger.name.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Trigger".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: trigger.description.to_string(),
                })),
                ..Default::default()
            });
        }

        for effect in crate::EFFECTS.values() {
            if !effect.scopes.contains(&scope::Scope::Unknown)
                && !effect.scopes.contains(&current_scope)
                && !effect.scopes.contains(&scope::Scope::Global)
            {
                continue;
            }
            items.push(CompletionItem {
                label: effect.name.to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Effect".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: effect.description.to_string(),
                })),
                ..Default::default()
            });
        }

        let st = self.scripted_triggers.load();
        for trigger in st.values() {
            items.push(CompletionItem {
                label: trigger.name.clone(),
                kind: Some(CompletionItemKind::EVENT),
                detail: Some("Scripted Trigger".to_string()),
                documentation: Some(Documentation::String(format!(
                    "Defined in: {}",
                    trigger.path
                ))),
                ..Default::default()
            });
        }

        let se = self.scripted_effects.load();
        for effect in se.values() {
            items.push(CompletionItem {
                label: effect.name.clone(),
                kind: Some(CompletionItemKind::EVENT),
                detail: Some("Scripted Effect".to_string()),
                documentation: Some(Documentation::String(format!(
                    "Defined in: {}",
                    effect.path
                ))),
                ..Default::default()
            });
        }

        let ids = self.ideologies.load();
        for ideology in ids.values() {
            items.push(CompletionItem {
                label: ideology.name.clone(),
                kind: Some(CompletionItemKind::ENUM),
                detail: Some("Ideology".to_string()),
                documentation: Some(Documentation::String(format!(
                    "Defined in: {}",
                    ideology.path
                ))),
                ..Default::default()
            });
        }

        let sids = self.sub_ideologies.load();
        for (sid, (parent, _, _)) in sids.iter() {
            items.push(CompletionItem {
                label: sid.clone(),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                detail: Some(format!("Sub-Ideology (Parent: {})", parent)),
                ..Default::default()
            });
        }

        let traits = self.traits.load();
        for trait_info in traits.values() {
            items.push(CompletionItem {
                label: trait_info.name.clone(),
                kind: Some(CompletionItemKind::INTERFACE),
                detail: Some(trait_info.trait_type.clone()),
                documentation: Some(Documentation::String(format!(
                    "Defined in: {}",
                    trait_info.path
                ))),
                ..Default::default()
            });
        }

        let s_map = self.sprites.load();
        for sprite in s_map.values() {
            items.push(CompletionItem {
                label: sprite.name.clone(),
                kind: Some(CompletionItemKind::FILE),
                detail: Some("Sprite/GFX".to_string()),
                documentation: Some(Documentation::String(format!(
                    "Defined in: {}",
                    sprite.path
                ))),
                ..Default::default()
            });
        }

        let id_map = self.ideas.load();
        for idea in id_map.values() {
            items.push(CompletionItem {
                label: idea.name.clone(),
                kind: Some(CompletionItemKind::CONSTANT),
                detail: Some(format!("Idea ({})", idea.category)),
                documentation: Some(Documentation::String(format!(
                    "Defined in: {}",
                    idea.category
                ))),
                ..Default::default()
            });
        }

        let ability_map = self.abilities.load();
        for ability in ability_map.values() {
            items.push(CompletionItem {
                label: ability.key.clone(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Leader Ability".to_string()),
                ..Default::default()
            });
        }

        let a_map = self.achievements.load();
        for achievement in a_map.values() {
            items.push(CompletionItem {
                label: achievement.name.clone(),
                kind: Some(CompletionItemKind::EVENT),
                detail: Some("Achievement".to_string()),
                documentation: Some(Documentation::String(format!(
                    "Defined in: {}",
                    achievement.path
                ))),
                ..Default::default()
            });
        }

        let ap_map = self.ai_strategy_plans.load();
        for plan in ap_map.values() {
            items.push(CompletionItem {
                label: plan.name.clone(),
                kind: Some(CompletionItemKind::FOLDER),
                detail: Some("AI Strategy Plan".to_string()),
                documentation: Some(Documentation::String(format!(
                    "Defined in: {}",
                    plan.path
                ))),
                ..Default::default()
            });
        }

        let var_map = self.variables.load();
        for var_name in var_map.keys() {
            items.push(CompletionItem {
                label: var_name.clone(),
                kind: Some(CompletionItemKind::VARIABLE),
                detail: Some("Variable".to_string()),
                ..Default::default()
            });
        }

        let target_map = self.event_targets.load();
        for target_name in target_map.keys() {
            items.push(CompletionItem {
                label: target_name.clone(),
                kind: Some(CompletionItemKind::STRUCT),
                detail: Some("Event Target".to_string()),
                ..Default::default()
            });
        }

        let m_assets = self.music_assets.load();
        for asset in m_assets.values() {
            items.push(CompletionItem {
                label: asset.name.clone(),
                kind: Some(CompletionItemKind::FILE),
                detail: Some("Music Asset".to_string()),
                documentation: Some(Documentation::String(format!("File: {}", asset.file))),
                ..Default::default()
            });
        }

        let m_stations = self.music_stations.load();
        for station in m_stations.values() {
            items.push(CompletionItem {
                label: station.name.clone(),
                kind: Some(CompletionItemKind::FOLDER),
                detail: Some("Music Station".to_string()),
                ..Default::default()
            });
        }

        let m_songs = self.songs.load();
        for song in m_songs.values() {
            items.push(CompletionItem {
                label: song.name.clone(),
                kind: Some(CompletionItemKind::FILE),
                detail: Some("Song".to_string()),
                ..Default::default()
            });
        }

        let s_sounds = self.sounds.load();
        for sound in s_sounds.values() {
            items.push(CompletionItem {
                label: sound.name.clone(),
                kind: Some(CompletionItemKind::FILE),
                detail: Some("Sound".to_string()),
                documentation: Some(Documentation::String(format!("File: {}", sound.file))),
                ..Default::default()
            });
        }

        let s_effects = self.sound_effects.load();
        for effect in s_effects.values() {
            items.push(CompletionItem {
                label: effect.name.clone(),
                kind: Some(CompletionItemKind::EVENT),
                detail: Some("Sound Effect".to_string()),
                ..Default::default()
            });
        }

        let s_falloffs = self.falloffs.load();
        for falloff in s_falloffs.values() {
            items.push(CompletionItem {
                label: falloff.name.clone(),
                kind: Some(CompletionItemKind::UNIT),
                detail: Some("Sound Falloff".to_string()),
                ..Default::default()
            });
        }

        let s_categories = self.sound_categories.load();
        for category in s_categories.values() {
            items.push(CompletionItem {
                label: category.name.clone(),
                kind: Some(CompletionItemKind::FOLDER),
                detail: Some("Sound Category".to_string()),
                ..Default::default()
            });
        }

        Ok(Some(CompletionResponse::Array(items)))
    }
}
