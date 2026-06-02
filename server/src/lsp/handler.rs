use std::collections::HashSet;
use std::sync::Arc;

use tower_lsp_server::LanguageServer;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;

use crate::backend::Backend;
use crate::data::entity_lookup;
use crate::lsp::call_hierarchy;
use crate::lsp::document_symbols;
use crate::lsp::rename;
use crate::lsp::semantic_tokens;
use crate::lsp::workspace_symbols;
use crate::parser::csv_parser;
use crate::parser::loc_parser;
use crate::parser::parser;
use crate::scope::scope;
use crate::utils::color_utils::find_colors;
use crate::utils::enhanced_color;
use crate::utils::loc_preview::find_identifier_in_loc;
use crate::utils::lsp_convert::ast_range_to_lsp_location;
use crate::utils::symbol_search::find_identifier_at;

use crate::{EFFECTS, MODIFIERS, SCOPES, TRIGGERS};

impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        if let Some(options) = params.initialization_options {
            if let Some(path) = options.get("gamePath").and_then(|v| v.as_str()) {
                if !path.is_empty() {
                    self.config.set_game_path(Some(path.to_string()));
                    let _gp = self.config.game_path();
                }
            }
            if let Some(ignore_list) = options.get("ignoreLocalization").and_then(|v| v.as_array())
            {
                let mut patterns = Vec::new();
                for val in ignore_list {
                    if let Some(s) = val.as_str() {
                        if let Ok(re) = regex::Regex::new(s) {
                            patterns.push(re);
                        }
                    }
                }
                self.config.set_ignored_loc_regex(patterns);
                let _ig = self.config.ignored_loc_regex();
            }
            if let Some(ignore_list) = options.get("ignoreFiles").and_then(|v| v.as_array()) {
                let mut patterns = Vec::new();
                for val in ignore_list {
                    if let Some(s) = val.as_str() {
                        if let Ok(re) = regex::Regex::new(s) {
                            patterns.push(re);
                        }
                    }
                }
                self.config.set_ignored_files_regex(patterns);
                let _ig = self.config.ignored_files_regex();
            }
            if let Some(enabled) = options
                .get("workspaceScanEnabled")
                .and_then(|v| v.as_bool())
            {
                self.config.set_workspace_scan_enabled(enabled);
                let _ws = self.config.workspace_scan_enabled();
            }
            if let Some(enabled) = options.get("stylingEnabled").and_then(|v| v.as_bool()) {
                self.config.set_styling_enabled(enabled);
                let _st = self.config.styling_enabled();
            }
            if let Some(enabled) = options.get("cosmeticLocIndent").and_then(|v| v.as_bool()) {
                self.config.set_cosmetic_loc_indent(enabled);
                let _ci = self.config.cosmetic_loc_indent();
            }
        }
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            range: Some(false),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::KEYWORD,
                                    SemanticTokenType::VARIABLE,
                                    SemanticTokenType::STRING,
                                    SemanticTokenType::NUMBER,
                                    SemanticTokenType::OPERATOR,
                                    SemanticTokenType::COMMENT,
                                    SemanticTokenType::TYPE,
                                    SemanticTokenType::EVENT,
                                    SemanticTokenType::FUNCTION,
                                    SemanticTokenType::ENUM,
                                    SemanticTokenType::ENUM_MEMBER,
                                    SemanticTokenType::STRUCT,
                                    SemanticTokenType::CLASS,
                                    SemanticTokenType::PROPERTY,
                                ],
                                token_modifiers: vec![],
                            },
                            ..Default::default()
                        },
                    ),
                ),
                color_provider: Some(ColorProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: None,
                    trigger_characters: Some(vec![
                        "=".to_string(),
                        "{".to_string(),
                        "[".to_string(),
                        ".".to_string(),
                    ]),
                    ..Default::default()
                }),
                document_formatting_provider: Some(OneOf::Left(true)),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![
                        "hoi4/getEventGraph".to_string(),
                        "hoi4/getMemoryUsage".to_string(),
                    ],
                    ..Default::default()
                }),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                call_hierarchy_provider: Some(CallHierarchyServerCapability::Simple(true)),
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                })),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;

        let mut roots = vec![std::path::PathBuf::from(".")];
        let gp = self.config.game_path();
        if let Some(ref path) = gp {
            roots.insert(0, std::path::PathBuf::from(path));
            self.client
                .log_message(MessageType::INFO, format!("Using HOI4 game path: {}", path))
                .await;
        }

        tokio::join!(
            self.scan_localization(&roots),
            self.load_assets(),
            self.scan_scripted(&roots),
            self.scan_ideologies(&roots),
            self.scan_traits(&roots),
            self.scan_sprites(&roots),
            self.scan_ideas(&roots),
            self.scan_characters(&roots),
            self.scan_variables(&roots),
            self.scan_provinces(&roots),
            self.scan_states(&roots),
            self.scan_logistics(&roots),
            self.scan_map_objects(&roots),
            self.scan_adjacencies(&roots),
            self.scan_strategic_regions(&roots),
            self.scan_modifiers(&roots),
            self.scan_buildings(&roots),
            self.scan_resources(&roots),
            self.scan_state_categories(&roots),
            self.scan_achievements(&roots),
            self.scan_defines(&roots),
            self.scan_events(&roots),
            self.scan_focuses(&roots),
            self.scan_music(&roots),
            self.scan_sounds(&roots),
            self.scan_abilities(&roots),
            self.scan_ai_strategy_plans(&roots),
            self.scan_ai_areas(&roots),
            self.scan_continents(&roots),
            self.scan_portraits(&roots),
            self.scan_countries(&roots),
            self.scan_gfx(&roots),
        );

        // Rebuild reverse file-path indices so incremental updates are O(K) not O(N)
        self.scanner_data.rebuild_all_file_indices();

        // Collect workspace file paths for rename operations
        // Only scan the mod workspace (first root), not the game path
        self.collect_workspace_files(&roots[..1]).await;

        // Re-validate all open documents now that we have all data
        for entry in self.documents.iter() {
            if let Ok(uri) = entry.key().parse::<Uri>() {
                self.validate_document(uri).await;
            }
        }

        // Workspace-wide scan
        if self.config.workspace_scan_enabled() {
            self.validate_workspace(std::path::Path::new(".")).await;
        }
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        if let Some(settings) = params.settings.as_object() {
            if let Some(hoi4) = settings.get("hoi4").and_then(|v| v.as_object()) {
                if let Some(validator) = hoi4.get("validator").and_then(|v| v.as_object()) {
                    if let Some(ignore_list) = validator
                        .get("ignoreLocalization")
                        .and_then(|v| v.as_array())
                    {
                        let mut patterns = Vec::new();
                        for val in ignore_list {
                            if let Some(s) = val.as_str() {
                                if let Ok(re) = regex::Regex::new(s) {
                                    patterns.push(re);
                                }
                            }
                        }
                        self.config.set_ignored_loc_regex(patterns);
                        let _ig = self.config.ignored_loc_regex();
                    }
                    if let Some(ignore_list) =
                        validator.get("ignoreFiles").and_then(|v| v.as_array())
                    {
                        let mut patterns = Vec::new();
                        for val in ignore_list {
                            if let Some(s) = val.as_str() {
                                if let Ok(re) = regex::Regex::new(s) {
                                    patterns.push(re);
                                }
                            }
                        }
                        self.config.set_ignored_files_regex(patterns);
                        let _ig = self.config.ignored_files_regex();
                    }
                    if let Some(enabled) = validator
                        .get("workspaceScan")
                        .and_then(|v| v.as_object())
                        .and_then(|v| v.get("enabled"))
                        .and_then(|v| v.as_bool())
                    {
                        self.config.set_workspace_scan_enabled(enabled);
                        let _ws = self.config.workspace_scan_enabled();
                    }
                }
                if let Some(styling) = hoi4.get("styling").and_then(|v| v.as_object()) {
                    if let Some(enabled) = styling.get("enabled").and_then(|v| v.as_bool()) {
                        self.config.set_styling_enabled(enabled);
                        let _st = self.config.styling_enabled();
                    }
                    if let Some(enabled) = styling
                        .get("cosmeticLocalizationIndentation")
                        .and_then(|v| v.as_bool())
                    {
                        self.config.set_cosmetic_loc_indent(enabled);
                        let _ci = self.config.cosmetic_loc_indent();
                    }
                }
                // Re-validate all documents
                for entry in self.documents.iter() {
                    if let Ok(uri) = entry.key().parse::<Uri>() {
                        self.validate_document(uri).await;
                    }
                }
            }
        }
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.as_str().to_string();
        let text = params.text_document.text;
        self.documents.insert(uri.clone(), text.clone());
        self.cache_ast(&uri, &text);
        self.validate_document(params.text_document.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.as_str().to_string();
        let text = params.content_changes[0].text.clone();
        self.documents.insert(uri.clone(), text.clone());
        let (script, _) = self.cache_ast(&uri, &text);

        // Live-update scanner data from cached AST (no re-parse needed)
        if let Some(file_path) = params.text_document.uri.to_file_path() {
            let path_str = file_path.to_string_lossy().to_string();
            crate::scanner::incremental_scanner::update_scanner_data_from_ast(
                &self.scanner_data,
                &path_str,
                &script,
            );
        }

        self.validate_document(params.text_document.uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        let uri_str = uri.as_str().to_string();

        // Use the text from the save notification if available, otherwise from documents cache
        let content = if let Some(ref text) = params.text {
            text.clone()
        } else if let Some(cached) = self.documents.get(&uri_str) {
            cached.clone()
        } else {
            return;
        };

        // Convert URI to a file path string for the incremental scanner
        if let Some(file_path) = uri.to_file_path() {
            let path_str = file_path.to_string_lossy().to_string();
            crate::scanner::incremental_scanner::update_scanner_data_for_file(
                &self.scanner_data,
                &path_str,
                &content,
            );
        }

        self.cache_ast(&uri_str, &content);
        self.validate_document(uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.as_str().to_string();
        self.documents.remove(&uri);
        self.document_asts.remove(&uri);
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri.to_string();

        // YAML localization files get their own line-based semantic tokens
        if uri.ends_with(".yml") {
            return if let Some(content) = self.documents.get(&uri) {
                Ok(Some(semantic_tokens::loc_semantic_tokens(&content)))
            } else {
                Ok(None)
            };
        }

        // CSV map files (definition.csv, adjacencies.csv) get positional tokens
        if uri.ends_with(".csv") {
            return if let Some(content) = self.documents.get(&uri) {
                Ok(semantic_tokens::csv_semantic_tokens(&content))
            } else {
                Ok(None)
            };
        }

        match self.ensure_ast_cached(&uri) {
            Some((script, _)) => {
                let mut keywords = HashSet::new();

                for k in TRIGGERS.keys() {
                    keywords.insert(k.to_string());
                }
                for k in EFFECTS.keys() {
                    keywords.insert(k.to_string());
                }
                for k in MODIFIERS.keys() {
                    keywords.insert(k.to_string());
                }
                for k in SCOPES.iter() {
                    keywords.insert(k.to_string());
                    keywords.insert(k.to_ascii_lowercase());
                }

                // Add hardcoded achievement keywords
                keywords.insert("unique_id".to_string());
                keywords.insert("possible".to_string());
                keywords.insert("happened".to_string());
                keywords.insert("ribbon".to_string());
                keywords.insert("frames".to_string());
                keywords.insert("colors".to_string());
                keywords.insert("custom_achievement".to_string());
                keywords.insert("custom_ribbon".to_string());
                keywords.insert("key".to_string());

                // Character keywords
                keywords.insert("characters".to_string());
                keywords.insert("advisor".to_string());
                keywords.insert("country_leader".to_string());
                keywords.insert("corps_commander".to_string());
                keywords.insert("field_marshal".to_string());
                keywords.insert("navy_leader".to_string());
                keywords.insert("scientist".to_string());
                keywords.insert("portraits".to_string());
                keywords.insert("traits".to_string());
                keywords.insert("skill".to_string());
                keywords.insert("gender".to_string());
                keywords.insert("instance".to_string());
                keywords.insert("idea_token".to_string());
                keywords.insert("legacy_id".to_string());
                keywords.insert("expire".to_string());
                keywords.insert("recruit_character".to_string());
                keywords.insert("ideology".to_string());

                // Custom advancement field keywords
                keywords.insert("achievement".to_string());

                // Ability keywords
                keywords.insert("ability".to_string());
                keywords.insert("name".to_string());
                keywords.insert("desc".to_string());
                keywords.insert("type".to_string());
                keywords.insert("cost".to_string());
                keywords.insert("duration".to_string());
                keywords.insert("cooldown".to_string());
                keywords.insert("icon".to_string());
                keywords.insert("sound_effect".to_string());
                keywords.insert("cancelable".to_string());
                keywords.insert("allowed".to_string());
                keywords.insert("one_time_effect".to_string());
                keywords.insert("unit_modifiers".to_string());
                keywords.insert("ai_will_do".to_string());
                keywords.insert("has_ability".to_string());
                keywords.insert("add_ability".to_string());
                keywords.insert("remove_ability".to_string());

                // AI strategy plan keywords
                keywords.insert("enable".to_string());
                keywords.insert("abort".to_string());
                keywords.insert("ai_national_focuses".to_string());
                keywords.insert("focus_factors".to_string());
                keywords.insert("research".to_string());
                keywords.insert("weight".to_string());
                keywords.insert("planned_production".to_string());
                keywords.insert("technologies".to_string());

                // AI area keywords
                keywords.insert("continents".to_string());
                keywords.insert("strategic_regions".to_string());

                // Music keywords
                keywords.insert("music".to_string());
                keywords.insert("music_station".to_string());
                keywords.insert("song".to_string());
                keywords.insert("chance".to_string());
                keywords.insert("base".to_string());
                keywords.insert("factor".to_string());
                keywords.insert("add".to_string());
                keywords.insert("modifier".to_string());
                keywords.insert("volume".to_string());
                keywords.insert("file".to_string());

                // Ideology definition keywords
                keywords.insert("types".to_string());
                keywords.insert("dynamic_faction_names".to_string());
                keywords.insert("rules".to_string());
                keywords.insert("can_host_government_in_exile".to_string());
                keywords.insert("war_impact_on_world_tension".to_string());
                keywords.insert("faction_impact_on_world_tension".to_string());
                keywords.insert("can_be_boosted".to_string());
                keywords.insert("can_collaborate".to_string());
                keywords.insert("modifiers".to_string());
                keywords.insert("faction_modifiers".to_string());
                keywords.insert("can_create_collaboration_government".to_string());
                keywords.insert("can_declare_war_on_same_ideology".to_string());
                keywords.insert("can_force_government".to_string());
                keywords.insert("can_send_volunteers".to_string());
                keywords.insert("can_puppet".to_string());
                keywords.insert("can_lower_tension".to_string());
                keywords.insert("can_only_justify_war_on_threat_country".to_string());
                keywords.insert("can_guarantee_other_ideologies".to_string());
                keywords.insert("take_states_cost_factor".to_string());

                // Idea definition keywords (common/ideas/*.txt)
                keywords.insert("ideas".to_string());
                keywords.insert("idea_categories".to_string());
                // Known idea category names (game-defined, not user types)
                keywords.insert("country".to_string());
                keywords.insert("slot_ledgers".to_string());
                keywords.insert("slot".to_string());
                keywords.insert("character_slot".to_string());
                keywords.insert("designer".to_string());
                keywords.insert("use_list_view".to_string());
                keywords.insert("law".to_string());
                keywords.insert("picture".to_string());
                keywords.insert("targeted_modifier".to_string());
                keywords.insert("research_bonus".to_string());
                keywords.insert("equipment_bonus".to_string());
                keywords.insert("rule".to_string());
                keywords.insert("on_add".to_string());
                keywords.insert("on_remove".to_string());
                keywords.insert("cancel".to_string());
                keywords.insert("allowed_civil_war".to_string());
                keywords.insert("do_effect".to_string());
                keywords.insert("allowed_to_remove".to_string());
                keywords.insert("visible".to_string());
                keywords.insert("available".to_string());
                keywords.insert("removal_cost".to_string());
                keywords.insert("level".to_string());
                keywords.insert("ledger".to_string());
                keywords.insert("hidden".to_string());
                keywords.insert("politics_tab".to_string());

                // National focus tree structure keywords
                keywords.insert("focus_tree".to_string());
                keywords.insert("focus".to_string());
                keywords.insert("shared_focus".to_string());
                keywords.insert("joint_focus".to_string());
                keywords.insert("continuous_focus_palette".to_string());
                keywords.insert("continuous_focus_position".to_string());
                keywords.insert("initial_show_position".to_string());
                keywords.insert("shortcut".to_string());
                keywords.insert("inlay_window".to_string());
                keywords.insert("style".to_string());
                keywords.insert("search_filter_prios".to_string());

                // National focus property keywords
                keywords.insert("prerequisite".to_string());
                keywords.insert("mutually_exclusive".to_string());
                keywords.insert("bypass".to_string());
                keywords.insert("bypass_if_unavailable".to_string());
                keywords.insert("enable_automatic_bypass".to_string());
                keywords.insert("allow_branch".to_string());
                keywords.insert("available_if_capitulated".to_string());
                keywords.insert("cancel_if_invalid".to_string());
                keywords.insert("continue_if_invalid".to_string());
                keywords.insert("historical_ai".to_string());
                keywords.insert("completion_reward".to_string());
                keywords.insert("complete_tooltip".to_string());
                keywords.insert("select_effect".to_string());
                keywords.insert("bypass_effect".to_string());
                keywords.insert("search_filters".to_string());
                keywords.insert("text_icon".to_string());
                keywords.insert("will_lead_to_war_with".to_string());
                keywords.insert("dynamic".to_string());
                keywords.insert("offset".to_string());
                keywords.insert("relative_position_id".to_string());
                keywords.insert("id".to_string());
                keywords.insert("cost".to_string());
                keywords.insert("icon".to_string());
                keywords.insert("default".to_string());
                keywords.insert("reset_on_civilwar".to_string());
                keywords.insert("target".to_string());
                keywords.insert("scroll_wheel_factor".to_string());

                // Continuous focus keywords
                keywords.insert("daily_cost".to_string());
                keywords.insert("supports_ai_strategy".to_string());
                keywords.insert("cancel_effect".to_string());

                // Joint focus keywords
                keywords.insert("joint_trigger".to_string());
                keywords.insert("completion_reward_joint_originator".to_string());
                keywords.insert("completion_reward_joint_member".to_string());

                // Focus inlay window keywords
                keywords.insert("window_name".to_string());
                keywords.insert("internal".to_string());
                keywords.insert("scripted_buttons".to_string());
                keywords.insert("scripted_images".to_string());
                keywords.insert("click_effect".to_string());

                // Style definition keywords
                keywords.insert("unavailable".to_string());
                keywords.insert("current".to_string());

                // AI strategy plan keywords
                keywords.insert("ai_strategy".to_string());

                // State definition keywords (history/states/*.txt)
                keywords.insert("state".to_string());
                keywords.insert("id".to_string());
                keywords.insert("manpower".to_string());
                keywords.insert("state_category".to_string());
                keywords.insert("impassable".to_string());
                keywords.insert("resources".to_string());
                keywords.insert("local_supplies".to_string());
                keywords.insert("buildings_max_level_factor".to_string());
                keywords.insert("history".to_string());
                keywords.insert("provinces".to_string());

                // State history sub-keywords
                keywords.insert("owner".to_string());
                keywords.insert("controller".to_string());
                keywords.insert("victory_points".to_string());
                keywords.insert("buildings".to_string());
                keywords.insert("add_core_of".to_string());
                keywords.insert("add_claim_by".to_string());
                keywords.insert("set_state_name".to_string());

                // Strategic region definition keywords (map/strategicregions/*.txt)
                keywords.insert("strategic_region".to_string());
                keywords.insert("weather".to_string());
                keywords.insert("period".to_string());
                keywords.insert("between".to_string());
                keywords.insert("temperature".to_string());
                keywords.insert("no_phenomenon".to_string());
                keywords.insert("rain_light".to_string());
                keywords.insert("rain_heavy".to_string());
                keywords.insert("snow".to_string());
                keywords.insert("blizzard".to_string());
                keywords.insert("mud".to_string());
                keywords.insert("sandstorm".to_string());
                keywords.insert("min_snow_level".to_string());
                keywords.insert("naval_terrain".to_string());

                // Resource types (inside resources = { })
                // Dynamically scanned from common/resources/*.txt — flows through entity_names pipeline
                // Building types (inside buildings = { } in state history)
                // Dynamically scanned from common/buildings/*.txt — flows through entity_names pipeline
                // Known state category names (game-defined, highlights values in state_category = X)
                // Dynamically scanned from common/state_category/*.txt — flows through entity_names pipeline

                let lookup = entity_lookup::EntityLookup::new(&self.scanner_data);
                let all_names = lookup.entity_names();

                let ctx = semantic_tokens::SemanticTokenContext::new(keywords, all_names);

                Ok(Some(semantic_tokens::get_semantic_tokens(&script, &ctx)))
            }
            _ => Ok(None),
        }
    }

    async fn document_color(&self, params: DocumentColorParams) -> Result<Vec<ColorInformation>> {
        let uri = params.text_document.uri.to_string();
        if let Some((script, _)) = self.ensure_ast_cached(&uri) {
            return Ok(find_colors(&script));
        }
        Ok(vec![])
    }

    async fn color_presentation(
        &self,
        params: ColorPresentationParams,
    ) -> Result<Vec<ColorPresentation>> {
        // Determine if this is a color_ui field by checking the document context
        let uri = params.text_document.uri.to_string();
        let is_ui = match self.documents.get(&uri) {
            Some(content) => {
                // Simple heuristic: check if "color_ui" appears near the color range
                // This is a basic implementation - could be improved with AST analysis
                content.contains("color_ui")
            }
            _ => false,
        };

        // Get color modifiers from defines
        let defines = self.scanner_data.defines();
        let modifiers = enhanced_color::ColorModifiers::from_defines(&defines);

        // Generate presentations for both RGB and HSV formats
        Ok(enhanced_color::generate_color_presentations(
            &params.color,
            params.range,
            is_ui,
            &modifiers,
        ))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri.to_string();
        if let Some(content) = self.documents.get(&uri) {
            if uri.ends_with(".yml") {
                let cosmetic_indent = self.config.cosmetic_loc_indent();
                let formatted = loc_parser::format_loc_file(&content, cosmetic_indent);
                let full_range = Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: content.lines().count() as u32,
                        character: content.lines().last().unwrap_or("").len() as u32,
                    },
                };
                return Ok(Some(vec![TextEdit {
                    range: full_range,
                    new_text: formatted,
                }]));
            } else if uri.ends_with(".csv") {
                let formatted = csv_parser::format_csv(&content, ';');
                let full_range = Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: content.lines().count() as u32,
                        character: content.lines().last().unwrap_or("").len() as u32,
                    },
                };
                return Ok(Some(vec![TextEdit {
                    range: full_range,
                    new_text: formatted,
                }]));
            }
        }
        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        self.handle_hover(params).await
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.handle_completion(params).await
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        if let Some(content) = self.documents.get(&uri) {
            let identifier = if uri.ends_with(".yml") {
                find_identifier_in_loc(&content, position)
            } else {
                let (script, _) = self.ensure_ast_cached(&uri).unwrap_or_else(|| {
                    let (s, e) = parser::parse_script(&content);
                    (Arc::new(s), e)
                });
                let mut scope_stack = scope::ScopeStack::new(scope::Scope::Global);
                let achievements = &self.scanner_data.achievements;
                find_identifier_at(&script, position, &mut scope_stack, achievements)
                    .map(|(id, _, _, _)| id)
            };

            if let Some(identifier) = identifier {
                let lookup = entity_lookup::EntityLookup::new(&self.scanner_data);
                let locations = lookup.find_definition(&identifier);
                if !locations.is_empty() {
                    let lsp_locations: Vec<Location> = locations
                        .iter()
                        .map(|loc| ast_range_to_lsp_location(&loc.range, &loc.path))
                        .collect();
                    return Ok(Some(GotoDefinitionResponse::Array(lsp_locations)));
                }
            }
        }
        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;

        if let Some((script, _)) = self.ensure_ast_cached(&uri) {
            let mut scope_stack = scope::ScopeStack::new(scope::Scope::Global);
            let achievements = &self.scanner_data.achievements;
            if let Some((identifier, _, _, _)) =
                find_identifier_at(&script, position, &mut scope_stack, achievements)
            {
                let mut locations = Vec::new();

                // Search in all roots
                let mut roots = vec![std::path::PathBuf::from(".")];
                let gp = self.config.game_path();
                if let Some(ref path) = gp {
                    roots.push(std::path::PathBuf::from(path));
                }

                for root in roots {
                    self.find_references_in_root(&root, &identifier, &mut locations)
                        .await;
                }

                if !locations.is_empty() {
                    return Ok(Some(locations));
                }
            }
        }
        Ok(None)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        self.handle_code_action(params).await
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<LSPAny>> {
        if params.command == "hoi4/getEventGraph" {
            let events_map: std::collections::HashMap<
                String,
                crate::scanner::event_scanner::Event,
            > = self
                .scanner_data
                .events
                .iter()
                .map(|e| (e.key().to_string(), e.value().clone()))
                .collect();
            let json = serde_json::to_value(&events_map).unwrap();
            return Ok(Some(json));
        } else if params.command == "hoi4/getMemoryUsage" {
            let mut sys = self.system_info.lock().unwrap();
            if let Ok(pid) = sysinfo::get_current_pid() {
                sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
                if let Some(process) = sys.process(pid) {
                    let memory = process.memory();
                    let json = serde_json::json!({
                        "memoryUsedBytes": memory
                    });
                    return Ok(Some(json));
                }
            }
            return Ok(None);
        }
        Ok(None)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri.as_str();

        if let Some((script, _)) = self.ensure_ast_cached(uri) {
            let symbols = document_symbols::generate_document_symbols(&script.entries);
            return Ok(Some(DocumentSymbolResponse::Nested(symbols)));
        }
        Ok(None)
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<WorkspaceSymbolResponse>> {
        let symbols =
            workspace_symbols::generate_workspace_symbols(&params.query, &self.scanner_data).await;

        Ok(Some(WorkspaceSymbolResponse::Flat(symbols)))
    }

    async fn prepare_call_hierarchy(
        &self,
        params: CallHierarchyPrepareParams,
    ) -> Result<Option<Vec<CallHierarchyItem>>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .as_str();
        let position = params.text_document_position_params.position;

        let item = call_hierarchy::prepare_call_hierarchy(uri, position, &self.scanner_data).await;

        Ok(item.map(|i| vec![i]))
    }

    async fn incoming_calls(
        &self,
        params: CallHierarchyIncomingCallsParams,
    ) -> Result<Option<Vec<CallHierarchyIncomingCall>>> {
        let calls = call_hierarchy::get_incoming_calls(
            &params.item,
            &self.scanner_data,
            &self.document_asts,
        )
        .await;

        Ok(Some(calls))
    }

    async fn outgoing_calls(
        &self,
        params: CallHierarchyOutgoingCallsParams,
    ) -> Result<Option<Vec<CallHierarchyOutgoingCall>>> {
        let calls = call_hierarchy::get_outgoing_calls(
            &params.item,
            &self.scanner_data,
            &self.document_asts,
        )
        .await;

        Ok(Some(calls))
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let uri = params.text_document.uri.as_str();
        let position = params.position;

        let result = rename::prepare_rename(uri, position, &self.scanner_data).await;

        Ok(result)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri.as_str();
        let position = params.text_document_position.position;
        let new_name = &params.new_name;

        let files = &self.scanner_data.workspace_files;
        let result = rename::rename_symbol(
            uri,
            position,
            new_name,
            &self.scanner_data,
            &self.documents,
            &self.document_asts,
            files,
        )
        .await;

        Ok(result)
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}
