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
            let map_config = self.map_config_for_uri(&uri);
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
        // Handle province definition files (map/definition.csv — the exact
        // filename comes from default.map). Column-aware: every data cell
        // gets only the values the engine accepts in that column, so the
        // generic trigger/effect flood below never fires in this file.
        {
            let map_config = self.map_config_for_uri(&uri);
            let is_definitions = uri
                .replace('\\', "/")
                .to_ascii_lowercase()
                .ends_with(&map_config.definitions.to_ascii_lowercase());
            if is_definitions {
                return self.definition_csv_completions(&uri, position);
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
        // Documented sub-keys of the block under the cursor, prepended to the
        // generic list below (see the `entity_parameters` block for why these
        // must be additive rather than a replacement).
        let mut param_items: Vec<CompletionItem> = Vec::new();

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

            // Value completion for `array = <builtin_array>` — e.g. inside
            // `any_of_scopes = { array = faction_members }` the RHS should
            // complete to engine-provided arrays (`faction_members`,
            // `owned_states`, `countries` …). This is additive: we return
            // early with the array list when the cursor is clearly on an
            // `array =` value, otherwise we fall through to the generic
            // trigger/effect + param list.
            //
            // Detection is line-prefix based: look for the last `=` on the
            // line up to the cursor and check whether `array` (case-insensitive)
            // appears immediately before it. That covers
            // `array = faction`, `array=faction`, `ARRAY = `, and
            // `my_block = { array = ` is handled via the enclosing chain
            // already (the value side of the `array` assignment).
            if let Some(content) = self.documents.get(&uri) {
                let lines: Vec<&str> = content.lines().collect();
                if let Some(line) = lines.get(position.line as usize) {
                    let byte_offset =
                        crate::utf16_to_byte_offset(line, position.character as usize);
                    let prefix = &line[..byte_offset.min(line.len())];
                    // Find last `=` before cursor
                    if let Some(eq_pos) = prefix.rfind('=') {
                        let before_eq = prefix[..eq_pos].trim_end();
                        // `array` is the key we care about (case-insensitive).
                        // Extract the last word before `=` to check.
                        let last_word = before_eq
                            .rsplit(|c: char| c.is_whitespace() || c == '{' || c == '}')
                            .next()
                            .unwrap_or("")
                            .trim();
                        if last_word.eq_ignore_ascii_case("array")
                            || last_word.eq_ignore_ascii_case("temp_array")
                        {
                            let current_scope_for_filter = current_scopes
                                .last()
                                .copied()
                                .unwrap_or(scope::Scope::Global)
                                .effective_scope();
                            let mut array_items = Vec::new();
                            for var in crate::data::hoi4_data::get_dynamic_variables().values() {
                                if !var.is_array {
                                    continue;
                                }
                                // Scope-filter: only offer arrays that are
                                // actually accessible from the current scope.
                                // `ScopeUsage::allows` treats Global as
                                // wildcard (any scope), which matches the
                                // engine's "array of" collections being
                                // readable from most contexts. A Country-only
                                // array like `faction_members` correctly
                                // won't appear when the stack is Global.
                                if !var.scopes.allows(&current_scope_for_filter) {
                                    continue;
                                }
                                let detail = if var.description.is_empty() {
                                    format!(
                                        "Built-in array · scope: {}",
                                        var.scopes
                                            .usage
                                            .iter()
                                            .map(|s| s.as_str())
                                            .collect::<Vec<_>>()
                                            .join(", ")
                                    )
                                } else {
                                    format!(
                                        "{} · scope: {}",
                                        var.description,
                                        var.scopes
                                            .usage
                                            .iter()
                                            .map(|s| s.as_str())
                                            .collect::<Vec<_>>()
                                            .join(", ")
                                    )
                                };
                                array_items.push(CompletionItem {
                                    label: var.name.clone(),
                                    kind: Some(CompletionItemKind::VARIABLE),
                                    detail: Some(detail),
                                    sort_text: Some(format!("0_{}", var.name)),
                                    documentation: Some(Documentation::MarkupContent(
                                        MarkupContent {
                                            kind: MarkupKind::Markdown,
                                            value: var.description.clone(),
                                        },
                                    )),
                                    ..Default::default()
                                });
                            }
                            if !array_items.is_empty() {
                                array_items.sort_by(|a, b| a.label.cmp(&b.label));
                                return Ok(Some(CompletionResponse::Array(array_items)));
                            }
                        }
                        // `var =` / `variable =` value completion for scalar
                        // builtins (e.g. `check_variable = { var = stability }`).
                        if last_word.eq_ignore_ascii_case("var")
                            || last_word.eq_ignore_ascii_case("variable")
                            || last_word.eq_ignore_ascii_case("temp_var")
                        {
                            let current_scope_for_filter = current_scopes
                                .last()
                                .copied()
                                .unwrap_or(scope::Scope::Global)
                                .effective_scope();
                            let mut var_items = Vec::new();
                            for var in crate::data::hoi4_data::get_dynamic_variables().values() {
                                // Offer scalars and arrays both for `var =`,
                                // but scalars are more common here.
                                if !var.scopes.allows(&current_scope_for_filter) {
                                    continue;
                                }
                                let kind_label = if var.is_array { "array" } else { "variable" };
                                let detail = if var.description.is_empty() {
                                    format!(
                                        "Built-in {} · scope: {}",
                                        kind_label,
                                        var.scopes
                                            .usage
                                            .iter()
                                            .map(|s| s.as_str())
                                            .collect::<Vec<_>>()
                                            .join(", ")
                                    )
                                } else {
                                    format!(
                                        "{} · scope: {}",
                                        var.description,
                                        var.scopes
                                            .usage
                                            .iter()
                                            .map(|s| s.as_str())
                                            .collect::<Vec<_>>()
                                            .join(", ")
                                    )
                                };
                                var_items.push(CompletionItem {
                                    label: var.name.clone(),
                                    kind: Some(CompletionItemKind::VARIABLE),
                                    detail: Some(detail),
                                    sort_text: Some(format!("0_{}", var.name)),
                                    documentation: Some(Documentation::MarkupContent(
                                        MarkupContent {
                                            kind: MarkupKind::Markdown,
                                            value: var.description.clone(),
                                        },
                                    )),
                                    ..Default::default()
                                });
                            }
                            if !var_items.is_empty() {
                                var_items.sort_by(|a, b| a.label.cmp(&b.label));
                                // Cap to avoid flooding: show first 100, but
                                // still a single response. The user can type
                                // to filter.
                                if var_items.len() > 100 {
                                    var_items.truncate(100);
                                }
                                return Ok(Some(CompletionResponse::Array(var_items)));
                            }
                        }
                    }
                }
            }

            // Focus search filters: inside `search_filters = { ... }` offer
            // the base-game filter set first (EnumMember, wiki notes as
            // documentation), then any mod-defined filters discovered from
            // GFX_FOCUS_FILTER_* sprites in scanner data. Additive — the
            // generic trigger/effect list still follows.
            if ctx
                .as_deref()
                .is_some_and(|k| k.eq_ignore_ascii_case("search_filters"))
            {
                let mut filter_items: Vec<CompletionItem> = Vec::new();
                for f in crate::data::focus_filters::BASE_FOCUS_FILTERS {
                    filter_items.push(CompletionItem {
                        label: (*f).to_string(),
                        kind: Some(CompletionItemKind::ENUM_MEMBER),
                        detail: Some("Focus search filter".to_string()),
                        sort_text: Some(format!("0_{f}")),
                        ..Default::default()
                    });
                }
                // Mod-defined filters: sprites named GFX_FOCUS_FILTER_*
                for entry in self.scanner_data.sprites.iter() {
                    let sprite = entry.key().as_ref();
                    if let Some(filter_name) = sprite.strip_prefix("GFX_FOCUS_FILTER_") {
                        let label = format!("FOCUS_FILTER_{filter_name}");
                        if crate::data::focus_filters::is_base_filter(&label) {
                            continue;
                        }
                        filter_items.push(CompletionItem {
                            label,
                            kind: Some(CompletionItemKind::ENUM_MEMBER),
                            detail: Some("Mod-defined focus search filter".to_string()),
                            documentation: Some(Documentation::String(format!(
                                "Defined by sprite: {}",
                                sprite
                            ))),
                            sort_text: Some(format!("0_FOCUS_FILTER_{filter_name}")),
                            ..Default::default()
                        });
                    }
                }
                return Ok(Some(CompletionResponse::Array(filter_items)));
            }

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

            // Structured block with documented parameters: surface the block's
            // own sub-keys FIRST (e.g. inside `add_timed_idea = { ... }` offer
            // idea/days/months/years at the top of the list).
            //
            // These are ADDITIVE, never a replacement. The wiki only documents
            // a block's scalar sub-keys, so the `parameters` map is always a
            // partial picture: `country_event` documents id/days/hours but not
            // title/desc/picture/option/trigger/immediate, and `if` documents
            // else/else_if/limit but none of the effects that make up its body.
            // Returning only the params would hide ~23% of the child keys that
            // appear inside these blocks in vanilla (86% for top-level event
            // definitions). A `sort_text` prefix floats them above the generic
            // trigger/effect list without suppressing anything.
            //
            // Chain walk (innermost first): take params from the first key
            // that documents any; a transparent block WITHOUT params
            // (`option`, `limit`) stops the walk — its body holds generic
            // trigger/effect content and must not inherit an outer entity's
            // params; plain instance keys (`land_doctrine_folder` under the
            // `technology_folders` param-container) let the walk continue so
            // the container's table applies to its sub-blocks.
            let chain =
                crate::scope::scope_context::find_enclosing_block_key_chain(&script, position);
            let mut inherited_params: Option<
                &'static std::collections::HashMap<String, crate::data::hoi4_data::ParameterDef>,
            > = None;
            // Chain is innermost first: take params from the first key that
            // documents any; a transparent block WITHOUT params stops the
            // walk before an outer entity's table leaks into this body.
            for key in chain.iter() {
                if let Some(params) = crate::data::hoi4_data::entity_parameters(key) {
                    if !params.is_empty() {
                        inherited_params = Some(params);
                        break;
                    }
                }
                if crate::data::hoi4_data::is_transparent_block(key) {
                    break;
                }
            }
            if let Some(params) = inherited_params {
                let mut names: Vec<&String> = params.keys().collect();
                names.sort();
                param_items.reserve(names.len());
                for name in names {
                    let pdef = &params[name];
                    let mut detail = String::new();
                    if !pdef.param_type.is_empty() {
                        detail.push_str(&format!("Type: {}", pdef.param_type));
                    }
                    if !pdef.value_type.is_empty() && pdef.value_type != pdef.param_type {
                        if !detail.is_empty() {
                            detail.push_str(" · ");
                        }
                        detail.push_str(&format!("{} reference", pdef.value_type));
                    }
                    if pdef.optional {
                        detail.push_str(" · optional");
                    }
                    param_items.push(CompletionItem {
                        label: name.clone(),
                        kind: Some(CompletionItemKind::PROPERTY),
                        detail: Some(detail),
                        // "0_" sorts ahead of the default (label-based)
                        // sort text used by the generic entries below.
                        sort_text: Some(format!("0_{name}")),
                        documentation: if pdef.description.is_empty() {
                            None
                        } else {
                            Some(Documentation::MarkupContent(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: pdef.description.clone(),
                            }))
                        },
                        ..Default::default()
                    });
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
        let total = param_items.len() + scope_items.len() + entity_items.len();
        items.reserve(total);
        // Block parameters first — they carry a "0_" sort_text so the client
        // keeps them at the top regardless of insertion order.
        items.append(&mut param_items);
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
    // Demote meta triggers whose label sorts before letters (e.g. `(building_count_trigger)`
    // U+0028 '(' and `-0.01`-style word suggestions U+002D) so bare `key = {}` with
    // empty prefix doesn't surface them as the top `Enter`-commit candidate.
    // Normal scope items keep `1_` prefix (after `0_` param-items), demoted ones get `9_`.
    let sort_text_for = |name: &str| -> String {
        let first = name.chars().next().unwrap_or('a');
        if first == '(' || first == '-' || first.is_ascii_digit() {
            format!("9_{name}")
        } else {
            format!("1_{name}")
        }
    };
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
            sort_text: Some(sort_text_for(&trigger.name)),
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
            sort_text: Some(sort_text_for(&effect.name)),
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

impl Backend {
    /// Column-aware completions for one `map/definition.csv` row
    /// (`ID;R;G;B;Type;Coastal;Terrain;Continent`).
    ///
    /// Every data cell returns a (possibly empty) list so the generic script
    /// completions never leak into this file. Comment lines and the optional
    /// `Province;…` header yield `None`; colour channels and anything past
    /// the 8th column yield an empty list — deliberately no suggestions.
    pub(crate) fn definition_csv_completions(
        &self,
        uri: &str,
        position: Position,
    ) -> Result<Option<CompletionResponse>> {
        let Some(content) = self.documents.get(uri) else {
            return Ok(None);
        };
        let Some(line) = content.lines().nth(position.line as usize) else {
            return Ok(None);
        };
        let byte_offset =
            crate::utf16_to_byte_offset(line, position.character as usize).min(line.len());
        let Some(col) = definition_csv_column(line, byte_offset) else {
            return Ok(None);
        };
        let items: Vec<CompletionItem> = match col {
            // Province ID: only the gaps in the defined sequence, if any.
            0 => {
                let defined: std::collections::HashSet<u32> = self
                    .scanner_data
                    .provinces
                    .iter()
                    .map(|e| *e.key())
                    .collect();
                crate::scanner::province_scanner::missing_province_ids(&defined, 100)
                    .into_iter()
                    .map(|id| CompletionItem {
                        label: id.to_string(),
                        kind: Some(CompletionItemKind::VALUE),
                        detail: Some("Missing province ID".to_string()),
                        sort_text: Some(format!("{id:08}")),
                        ..Default::default()
                    })
                    .collect()
            }
            // R/G/B colour channels: free numeric values, nothing to suggest.
            1..=3 => Vec::new(),
            // Province type.
            4 => ["land", "sea", "lake"]
                .into_iter()
                .map(|t| CompletionItem {
                    label: t.to_string(),
                    kind: Some(CompletionItemKind::ENUM),
                    detail: Some("Province type".to_string()),
                    ..Default::default()
                })
                .collect(),
            // Coastal status.
            5 => ["true", "false"]
                .into_iter()
                .map(|v| CompletionItem {
                    label: v.to_string(),
                    kind: Some(CompletionItemKind::ENUM),
                    detail: Some("Coastal status".to_string()),
                    ..Default::default()
                })
                .collect(),
            // Terrain: every category scanned from `common/terrain/`.
            6 => {
                let mut names: Vec<(String, bool, bool)> = self
                    .scanner_data
                    .terrain_categories
                    .iter()
                    .map(|e| {
                        let t = e.value();
                        (t.name.clone(), t.is_naval, t.is_water)
                    })
                    .collect();
                names.sort_by(|a, b| a.0.cmp(&b.0));
                names
                    .into_iter()
                    .map(|(name, is_naval, is_water)| {
                        let mut flags = Vec::new();
                        if is_naval {
                            flags.push("naval");
                        }
                        if is_water {
                            flags.push("water");
                        }
                        let detail = if flags.is_empty() {
                            "Terrain category".to_string()
                        } else {
                            format!("Terrain category ({})", flags.join(", "))
                        };
                        CompletionItem {
                            label: name,
                            kind: Some(CompletionItemKind::ENUM),
                            detail: Some(detail),
                            ..Default::default()
                        }
                    })
                    .collect()
            }
            // Continent: 0 (water) plus each scanned continent by its
            // 1-based definition order in `map/continent.txt`.
            7 => {
                let mut items = vec![CompletionItem {
                    label: "0".to_string(),
                    kind: Some(CompletionItemKind::ENUM),
                    detail: Some("No continent — sea provinces (lakes may use 0)".to_string()),
                    sort_text: Some("0000".to_string()),
                    ..Default::default()
                }];
                let mut conts: Vec<(u32, String)> = self
                    .scanner_data
                    .continents
                    .iter()
                    .map(|e| {
                        let c = e.value();
                        (c.index, c.name.clone())
                    })
                    .collect();
                conts.sort();
                for (index, name) in conts {
                    // The scanner numbers from 1; skip a 0 that would
                    // collide with the water entry above.
                    if index == 0 {
                        continue;
                    }
                    items.push(CompletionItem {
                        label: index.to_string(),
                        kind: Some(CompletionItemKind::ENUM),
                        detail: Some(format!("Continent: {name}")),
                        sort_text: Some(format!("{index:04}")),
                        ..Default::default()
                    });
                }
                items
            }
            // Anything past the 8th column is malformed input, not a value.
            _ => Vec::new(),
        };
        Ok(Some(CompletionResponse::Array(items)))
    }
}

/// Zero-based `definition.csv` cell index under `byte_offset`, or `None`
/// for comment lines and the optional header row (no completions there).
pub(crate) fn definition_csv_column(line: &str, byte_offset: usize) -> Option<usize> {
    if line.trim_start().starts_with('#') {
        return None;
    }
    if line
        .split(';')
        .next()
        .is_some_and(|first| first.trim().eq_ignore_ascii_case("province"))
    {
        return None;
    }
    Some(line[..byte_offset.min(line.len())].matches(';').count())
}

#[cfg(test)]
mod tests {
    use super::definition_csv_column;

    #[test]
    fn test_definition_csv_column_counts_cells() {
        let line = "12;34;56;78;land;false;forest;1";
        // Cursor inside the ID cell.
        assert_eq!(definition_csv_column(line, 0), Some(0));
        assert_eq!(definition_csv_column(line, 2), Some(0));
        // Just past the first ';' → R channel.
        assert_eq!(definition_csv_column(line, 3), Some(1));
        // Inside the terrain cell.
        let terrain_at = line.find("forest").unwrap();
        assert_eq!(definition_csv_column(line, terrain_at + 1), Some(6));
        // Past the end → continent cell.
        assert_eq!(definition_csv_column(line, line.len()), Some(7));
        // Past the 8th column → still a (malformed) cell index.
        assert_eq!(
            definition_csv_column("1;2;3;4;land;false;forest;1;extra", line.len() + 6),
            Some(8)
        );
    }

    #[test]
    fn test_definition_csv_column_skips_comment_and_header() {
        assert_eq!(definition_csv_column("# a comment", 2), None);
        assert_eq!(definition_csv_column("   # indented", 5), None);
        assert_eq!(
            definition_csv_column("Province;Red;Green;Blue;Type;Coastal;Terrain;Continent", 4),
            None
        );
    }
}
