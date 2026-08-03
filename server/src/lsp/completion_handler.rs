use crate::Backend;
use crate::scope::scope;
use crate::scope::scope_context::{find_context_at, find_scope_context_at};
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;

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
                    let byte_offset =
                        crate::utf16_to_byte_offset(line, position.character as usize);
                    let prefix = &line[..byte_offset];

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
                            let target_map = &self.scanner_data.event_targets;
                            for entry in target_map.iter() {
                                items.push(CompletionItem {
                                    label: entry.key().to_string(),
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
            let map_config = crate::utils::map_config::get_map_config(std::path::Path::new("."));
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
                            let rules = &self.scanner_data.adjacency_rules;
                            for entry in rules.iter() {
                                items.push(CompletionItem {
                                    label: entry.key().to_string(),
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
            if let Some((script, _)) = self.ensure_ast_cached(&uri) {
                if let Some(context_key) = find_context_at(&script, position) {
                    let key_lower = context_key.to_ascii_lowercase();
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
            if let Some((script, _)) = self.ensure_ast_cached(&uri) {
                if let Some(context_key) = find_context_at(&script, position) {
                    let mut completion_items = Vec::new();
                    let key_lower = context_key.to_ascii_lowercase();

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

        let mut current_scopes = vec![scope::Scope::Global];

        // Try to find context for HOI4 scripts
        if let Some((script, _)) = self.ensure_ast_cached(&uri) {
            // Unified with validation: per-file initial scope + full ScopeCtx maps
            // (event targets, characters, achievements) so completion never
            // diverges from HOM004 scope inference.
            let initial_scope = scope::initial_scope_for_uri(&uri);
            let sctx = scope::ScopeCtx {
                uri: &uri,
                event_targets: Some(&self.scanner_data.event_targets),
                characters: Some(&self.scanner_data.characters),
                achievements: Some(&self.scanner_data.achievements),
                in_random_list: false,
                state_targeted: false,
            };
            let (ctx, scopes) = find_scope_context_at(&script, position, initial_scope, &sctx);
            current_scopes = scopes;
            if let Some(context_key) = ctx {
                if context_key.to_ascii_lowercase().contains("color") {
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

        let mut items = Vec::new();

        let current_scope = current_scopes
            .last()
            .copied()
            .unwrap_or(scope::Scope::Global)
            .effective_scope();

        // Static triggers + effects, prebuilt per scope and cached (cheap clone).
        let scope_items = scope_trigger_effect_items(current_scope);
        // Scanner-derived entities, prebuilt after scans and cached (cheap clone).
        let entity_items = self.completion_entity_cache.load_full();
        let total = scope_items.len() + entity_items.len();
        items.reserve(total);
        items.extend(scope_items.iter().cloned());
        items.extend(entity_items.iter().cloned());

        Ok(Some(CompletionResponse::Array(items)))
    }
}

/// Prebuilt `triggers` + `effects` completion items for a given scope.
///
/// These come from the static `TRIGGERS`/`EFFECTS` data bases and depend only on
/// the current scope, so each scope's list is built once and cached (the vec is
/// Arc-cloned per completion request instead of re-iterating and re-forming every
/// CompletionItem on each keystroke).
fn scope_trigger_effect_items(current_scope: scope::Scope) -> std::sync::Arc<Vec<CompletionItem>> {
    use once_cell::sync::Lazy;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    static CACHE: Lazy<Mutex<HashMap<String, Arc<Vec<CompletionItem>>>>> =
        Lazy::new(|| Mutex::new(HashMap::new()));

    let key = current_scope.as_str().to_string();
    if let Some(cached) = CACHE.lock().unwrap().get(&key) {
        return cached.clone();
    }

    let mut items: Vec<CompletionItem> = Vec::new();
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

    let cached = Arc::new(items);
    CACHE.lock().unwrap().insert(key, cached.clone());
    cached
}
