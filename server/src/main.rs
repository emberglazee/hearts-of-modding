#![allow(clippy::collapsible_if)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
mod ability_scanner;
mod achievement_scanner;
mod adjacency_scanner;
mod advanced_validation;
mod ai_area_scanner;
mod ai_strategy_plan_scanner;
mod ast;
mod building_scanner;
mod call_hierarchy;
mod character_scanner;
mod code_action_handler;
mod color_utils;
mod completion_handler;
mod config;
mod continent_scanner;
mod country_scanner;
mod csv_parser;
mod defines_parser;
mod document_symbols;
mod enhanced_color;
mod entity_lookup;
mod event_scanner;
mod formatting;
mod fs_util;
mod gfx_scanner;
mod hoi4_data;
mod hover_handler;
mod idea_scanner;
mod ideology_scanner;
mod loc_parser;
mod loc_preview;
mod logistics_scanner;
mod lsp_convert;
mod map_config;
mod map_object_scanner;
mod modifier_display;
mod modifier_format;
mod modifier_scanner;
mod music_scanner;
mod parser;
mod portrait_scanner;
mod province_scanner;
mod rename;
mod scan_orchestrator;
mod scanner_data;
mod scope;
mod scope_context;
mod scripted_loc_scanner;
mod scripted_scanner;
mod semantic_tokens;
mod sound_scanner;
mod sprite_scanner;
mod state_scanner;
mod strategic_region_scanner;
mod symbol_search;
#[cfg(test)]
mod test_loc_version;
mod trait_scanner;
mod variable_scanner;
mod workspace_symbols;

use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::color_utils::find_colors;
use crate::config::Config;
use crate::loc_preview::find_identifier_in_loc;
use crate::lsp_convert::{
    ast_range_to_lsp, ast_range_to_lsp_location, ast_related_info_to_lsp, ast_tag_to_lsp,
};
use crate::scanner_data::ScannerData;
use crate::symbol_search::find_identifier_at;

pub(crate) static TRIGGERS: Lazy<HashMap<&'static str, hoi4_data::HOI4Entity>> =
    Lazy::new(hoi4_data::get_triggers);
pub(crate) static EFFECTS: Lazy<HashMap<&'static str, hoi4_data::HOI4Entity>> =
    Lazy::new(hoi4_data::get_effects);
pub(crate) static MODIFIERS: Lazy<HashMap<&'static str, hoi4_data::HOI4Entity>> =
    Lazy::new(hoi4_data::get_modifiers);
pub(crate) static SCOPES: Lazy<Vec<&'static str>> = Lazy::new(hoi4_data::get_scopes);
pub(crate) static LOC_COMMANDS: Lazy<Vec<&'static str>> = Lazy::new(hoi4_data::get_loc_commands);

/// Convert a byte offset in a UTF-8 string to a UTF-16 code unit offset
/// This is required because LSP uses UTF-16 positions, but Rust strings are UTF-8
#[allow(dead_code)]
fn byte_offset_to_utf16(s: &str, byte_offset: usize) -> u32 {
    s[..byte_offset]
        .chars()
        .map(|c| c.len_utf16())
        .sum::<usize>() as u32
}

/// Get the UTF-16 length of a string
pub(crate) fn utf16_len(s: &str) -> u32 {
    s.chars().map(|c| c.len_utf16()).sum::<usize>() as u32
}

struct Backend {
    client: Client,
    documents: DashMap<String, String>,
    document_asts: DashMap<String, (Arc<ast::Script>, Vec<(String, ast::Range)>)>,
    scanner_data: ScannerData,
    config: Config,
    system_info: Mutex<sysinfo::System>,
}

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
            self.scan_achievements(&roots),
            self.scan_defines(&roots),
            self.scan_events(&roots),
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
        self.cache_ast(&uri, &text);
        self.validate_document(params.text_document.uri).await;
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

        // Skip semantic tokens for YAML localization files
        if uri.ends_with(".yml") {
            return Ok(None);
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

                let lookup = entity_lookup::EntityLookup::new(&self.scanner_data);
                let all_names = lookup.entity_names();

                let mut ability_names = HashSet::new();
                let mut strategy_plan_names = HashSet::new();
                let mut ai_area_names = HashSet::new();
                let mut portrait_names = HashSet::new();
                let mut character_names = HashSet::new();
                let mut ideology_types = HashSet::new();
                let mut achievement_names = HashSet::new();
                let mut scripted_trigger_names = HashSet::new();
                let mut scripted_effect_names = HashSet::new();
                let mut country_tag_names = HashSet::new();
                let mut color_code_names = HashSet::new();

                for (name, kind) in all_names {
                    match kind {
                        entity_lookup::EntityKind::Ability => {
                            ability_names.insert(name);
                        }
                        entity_lookup::EntityKind::AiStrategyPlan => {
                            strategy_plan_names.insert(name);
                        }
                        entity_lookup::EntityKind::AiArea => {
                            ai_area_names.insert(name);
                        }
                        entity_lookup::EntityKind::Portrait => {
                            portrait_names.insert(name);
                        }
                        entity_lookup::EntityKind::Character => {
                            character_names.insert(name);
                        }
                        entity_lookup::EntityKind::SubIdeology => {
                            ideology_types.insert(name);
                        }
                        entity_lookup::EntityKind::Achievement => {
                            achievement_names.insert(name);
                        }
                        entity_lookup::EntityKind::ScriptedTrigger => {
                            scripted_trigger_names.insert(name);
                        }
                        entity_lookup::EntityKind::ScriptedEffect => {
                            scripted_effect_names.insert(name);
                        }
                        entity_lookup::EntityKind::CountryTag => {
                            country_tag_names.insert(name);
                        }
                        entity_lookup::EntityKind::ColorCode => {
                            color_code_names.insert(name);
                        }
                        _ => {}
                    }
                }

                Ok(Some(semantic_tokens::get_semantic_tokens(
                    &script,
                    &keywords,
                    &ability_names,
                    &strategy_plan_names,
                    &ai_area_names,
                    &portrait_names,
                    &character_names,
                    &ideology_types,
                    &achievement_names,
                    &scripted_trigger_names,
                    &scripted_effect_names,
                    &country_tag_names,
                    &color_code_names,
                )))
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
                let achievements = self.scanner_data.achievements();
                find_identifier_at(&script, position, &mut scope_stack, &achievements)
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
            let achievements = self.scanner_data.achievements();
            if let Some((identifier, _, _, _)) =
                find_identifier_at(&script, position, &mut scope_stack, &achievements)
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
            let events = self.scanner_data.events();
            let json = serde_json::to_value(&*events).unwrap();
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
        let calls =
            call_hierarchy::get_incoming_calls(&params.item, &self.scanner_data, &self.documents)
                .await;

        Ok(Some(calls))
    }

    async fn outgoing_calls(
        &self,
        params: CallHierarchyOutgoingCallsParams,
    ) -> Result<Option<Vec<CallHierarchyOutgoingCall>>> {
        let calls =
            call_hierarchy::get_outgoing_calls(&params.item, &self.scanner_data, &self.documents)
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

        let files = self.scanner_data.workspace_files();
        let result = rename::rename_symbol(
            uri,
            position,
            new_name,
            &self.scanner_data,
            &self.documents,
            &files,
        )
        .await;

        Ok(result)
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

impl Backend {
    pub(crate) fn make_file_link(&self, path: &str) -> String {
        // Try to canonicalize for absolute path if possible
        let abs_path = std::path::Path::new(path)
            .canonicalize()
            .unwrap_or_else(|_| std::path::PathBuf::from(path));
        format!(
            "[{}]({}://{})",
            path,
            "file",
            abs_path.to_string_lossy().replace("\\", "/")
        )
    }

    /// Parse and cache the AST for a URI. Returns (Arc<Script>, parse_errors).
    fn cache_ast(&self, uri: &str, content: &str) -> (Arc<ast::Script>, Vec<(String, ast::Range)>) {
        let (script, errors) = parser::parse_script(content);
        let script = Arc::new(script);
        self.document_asts
            .insert(uri.to_string(), (script.clone(), errors.clone()));
        (script, errors)
    }

    /// Get cached AST for a URI, or parse+cache from document text if missing.
    fn ensure_ast_cached(
        &self,
        uri: &str,
    ) -> Option<(Arc<ast::Script>, Vec<(String, ast::Range)>)> {
        if let Some(cached) = self.document_asts.get(uri) {
            return Some((cached.0.clone(), cached.1.clone()));
        }
        self.documents
            .get(uri)
            .map(|content| self.cache_ast(uri, &content))
    }

    pub(crate) fn check_is_province(
        &self,
        val: &ast::NodeedValue,
        diagnostics: &mut Vec<Diagnostic>,
        provs: &HashMap<u32, province_scanner::Province>,
    ) {
        let id_opt = match &val.value {
            ast::Value::Number(n) => Some(*n as u32),
            ast::Value::String(s) => s.parse::<u32>().ok(),
            _ => None,
        };

        if let Some(id) = id_opt {
            if !provs.is_empty() && !provs.contains_key(&id) {
                diagnostics.push(Diagnostic {
                    range: ast_range_to_lsp(&val.range),
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: format!("Unknown province ID: {}", id),
                    code: Some(NumberOrString::String(
                        advanced_validation::UNKNOWN_TRIGGER.to_string(),
                    )),
                    source: Some("Hearts of Modding".to_string()),
                    ..Default::default()
                });
            }
        }
    }

    async fn find_references_in_root(
        &self,
        root: &std::path::Path,
        identifier: &str,
        locations: &mut Vec<Location>,
    ) {
        let extensions = ["txt", "yml", "gfx", "gui", "asset"];
        let filter = self.get_sync_filter();
        let files = crate::fs_util::collect_files(root, &extensions, filter, false);

        for path in &files {
            if let Ok(content) = std::fs::read_to_string(path) {
                if content.contains(identifier) {
                    for (line_idx, line) in content.lines().enumerate() {
                        let mut start_search = 0;
                        while let Some(pos) = line[start_search..].find(identifier) {
                            let actual_pos = start_search + pos;

                            let before = if actual_pos > 0 {
                                line.chars().nth(actual_pos - 1)
                            } else {
                                None
                            };
                            let after = line.chars().nth(actual_pos + identifier.len());

                            let is_word_start =
                                before.is_none_or(|c| !parser::is_identifier_char(c));
                            let is_word_end = after.is_none_or(|c| !parser::is_identifier_char(c));

                            if is_word_start && is_word_end {
                                locations.push(Location {
                                    uri: Uri::from_file_path(
                                        path.canonicalize().unwrap_or_else(|_| path.clone()),
                                    )
                                    .unwrap(),
                                    range: Range {
                                        start: Position {
                                            line: line_idx as u32,
                                            character: actual_pos as u32,
                                        },
                                        end: Position {
                                            line: line_idx as u32,
                                            character: (actual_pos + identifier.len()) as u32,
                                        },
                                    },
                                });
                            }
                            start_search = actual_pos + identifier.len();
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn get_sync_filter(
        &self,
    ) -> impl Fn(&std::path::Path) -> bool + Send + Sync + 'static {
        let ignored = self.config.ignored_files_regex();
        move |path| crate::fs_util::is_path_ignored(path, &ignored)
    }

    async fn validate_workspace(&self, root: &std::path::Path) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("Starting workspace diagnostic scan in: {:?}", root),
            )
            .await;

        let extensions = ["txt", "yml", "csv"];
        let filter = self.get_sync_filter();
        let files = crate::fs_util::collect_files(root, &extensions, filter, true);
        let mut file_count = 0;

        for path in &files {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(abs_path) = path.canonicalize() {
                    if let Some(uri) = Uri::from_file_path(abs_path) {
                        let diagnostics = self.validate_content(&uri, &content).await;
                        if !diagnostics.is_empty() {
                            self.client
                                .publish_diagnostics(uri, diagnostics, None)
                                .await;
                        }
                        file_count += 1;
                    }
                }
            }
        }
        self.client
            .log_message(
                MessageType::INFO,
                format!("Workspace scan complete. Scanned {} files.", file_count),
            )
            .await;
    }

    async fn collect_workspace_files(&self, roots: &[std::path::PathBuf]) {
        let mut all_files = HashSet::new();
        let extensions = ["txt", "yml"];

        for root in roots {
            let files =
                crate::fs_util::collect_files(root, &extensions, self.get_sync_filter(), true);
            for path in &files {
                if let Ok(abs_path) = path.canonicalize() {
                    all_files.insert(abs_path.to_string_lossy().to_string());
                }
            }
        }

        self.scanner_data.set_workspace_files(all_files);
    }

    async fn validate_document(&self, uri: Uri) {
        let content = match self.documents.get(uri.as_str()) {
            Some(c) => c.clone(),
            _ => {
                return;
            }
        };

        let diagnostics = self.validate_content(&uri, &content).await;
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    async fn validate_content(&self, uri: &Uri, content: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let styling_enabled = self.config.styling_enabled();
        let mut script_opt: Option<Arc<ast::Script>> = None;
        let map_config = crate::map_config::get_map_config(std::path::Path::new("."));

        if uri.as_str().ends_with(".yml") {
            self.validate_localization_content(uri, content, &mut diagnostics)
                .await;
        } else if uri.as_str().ends_with("/map/supply_nodes.txt")
            || uri.as_str().ends_with("\\map\\supply_nodes.txt")
        {
            self.validate_supply_nodes_content(content, &mut diagnostics)
                .await;
        } else if uri.as_str().ends_with("/map/railways.txt")
            || uri.as_str().ends_with("\\map\\railways.txt")
        {
            self.validate_railways_content(content, &mut diagnostics)
                .await;
        } else if uri.as_str().ends_with("/map/buildings.txt")
            || uri.as_str().ends_with("\\map\\buildings.txt")
        {
            self.validate_map_buildings_content(content, &mut diagnostics)
                .await;
        } else if uri.as_str().ends_with("/map/unitstacks.txt")
            || uri.as_str().ends_with("\\map\\unitstacks.txt")
        {
            self.validate_unitstacks_content(content, &mut diagnostics)
                .await;
        } else if uri.as_str().ends_with("/map/weatherpositions.txt")
            || uri.as_str().ends_with("\\map\\weatherpositions.txt")
        {
            self.validate_weather_positions_content(content, &mut diagnostics)
                .await;
        } else if uri.as_str().ends_with("adjacency_rules.txt") {
            self.validate_adjacency_rules_content(content, &mut diagnostics)
                .await;
        } else if uri.as_str().ends_with(&map_config.adjacencies) {
            self.validate_adjacencies_content(content, &mut diagnostics)
                .await;
        } else if uri.as_str().ends_with(&map_config.definitions) {
            self.validate_definition_content(content, &mut diagnostics)
                .await;
        } else if uri.as_str().contains("/common/strategic_regions/")
            && uri.as_str().ends_with(".txt")
        {
            if let Some((script, parse_errors)) = self.ensure_ast_cached(uri.as_str()) {
                for (msg, range) in &parse_errors {
                    diagnostics.push(Diagnostic {
                        range: ast_range_to_lsp(range),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: msg.clone(),
                        code: Some(NumberOrString::String(
                            advanced_validation::PARSE_ERROR.to_string(),
                        )),
                        source: Some("Hearts of Modding".to_string()),
                        ..Default::default()
                    });
                }
                self.validate_strategic_region_content(&script, &mut diagnostics)
                    .await;
                self.check_semantic(&script, &mut diagnostics, styling_enabled, uri.as_str())
                    .await;
                script_opt = Some(script);
            }
        } else if uri.as_str().ends_with(".csv") {
            // Do not parse other CSV files as clausewitz scripts
        } else {
            if let Some((script, parse_errors)) = self.ensure_ast_cached(uri.as_str()) {
                for (msg, range) in &parse_errors {
                    diagnostics.push(Diagnostic {
                        range: ast_range_to_lsp(range),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: msg.clone(),
                        code: Some(NumberOrString::String(
                            advanced_validation::PARSE_ERROR.to_string(),
                        )),
                        source: Some("Hearts of Modding".to_string()),
                        ..Default::default()
                    });
                }
                // Semantic validation
                self.check_semantic(&script, &mut diagnostics, styling_enabled, uri.as_str())
                    .await;
                script_opt = Some(script);
            }
        }

        if styling_enabled {
            let is_yaml = uri.as_str().ends_with(".yml");
            self.check_styling(
                content,
                script_opt.as_deref(),
                &mut diagnostics,
                is_yaml,
                uri.as_str(),
            );
        }

        diagnostics
    }

    async fn validate_supply_nodes_content(
        &self,
        content: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let provs = self.scanner_data.provinces();
        for (i, line) in content.lines().enumerate() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(id) = parts[1].parse::<u32>() {
                    if !provs.is_empty() && !provs.contains_key(&id) {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: i as u32,
                                    character: 0,
                                },
                                end: Position {
                                    line: i as u32,
                                    character: 100,
                                },
                            },
                            severity: Some(DiagnosticSeverity::WARNING),
                            message: format!("Unknown province ID: {}", id),
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }

    async fn validate_railways_content(&self, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        let provs = self.scanner_data.provinces();
        for (i, line) in content.lines().enumerate() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(num_provs) = parts[1].parse::<usize>() {
                    for j in 0..num_provs {
                        if parts.len() > 2 + j {
                            if let Ok(id) = parts[2 + j].parse::<u32>() {
                                if !provs.is_empty() && !provs.contains_key(&id) {
                                    diagnostics.push(Diagnostic {
                                        range: Range {
                                            start: Position {
                                                line: i as u32,
                                                character: 0,
                                            },
                                            end: Position {
                                                line: i as u32,
                                                character: 100,
                                            },
                                        },
                                        severity: Some(DiagnosticSeverity::WARNING),
                                        message: format!("Unknown province ID: {}", id),
                                        ..Default::default()
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    async fn validate_map_buildings_content(
        &self,
        content: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let states = self.scanner_data.states();
        for (i, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: i as u32,
                            character: 0,
                        },
                        end: Position {
                            line: i as u32,
                            character: line.len() as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::WARNING),
                    message:
                        "Empty line in buildings.txt is counted as an error in HOI4 error logs."
                            .to_string(),
                    ..Default::default()
                });
                continue;
            }
            let parts: Vec<&str> = line.split(';').collect();
            if parts.len() >= 7 {
                if let Ok(id) = parts[0].parse::<u32>() {
                    if !states.is_empty() && !states.contains_key(&id) {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: i as u32,
                                    character: 0,
                                },
                                end: Position {
                                    line: i as u32,
                                    character: parts[0].len() as u32,
                                },
                            },
                            severity: Some(DiagnosticSeverity::WARNING),
                            message: format!("Unknown state ID: {}", id),
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }

    async fn validate_weather_positions_content(
        &self,
        content: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let regions = self.scanner_data.strategic_regions();
        for (i, line) in content.lines().enumerate() {
            let parts: Vec<&str> = line.split(';').collect();
            if parts.len() >= 5 {
                if let Ok(id) = parts[0].parse::<u32>() {
                    if !regions.is_empty() && !regions.contains_key(&id) {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: i as u32,
                                    character: 0,
                                },
                                end: Position {
                                    line: i as u32,
                                    character: parts[0].len() as u32,
                                },
                            },
                            severity: Some(DiagnosticSeverity::WARNING),
                            message: format!("Unknown strategic region ID: {}", id),
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }

    async fn validate_unitstacks_content(&self, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        let provs = self.scanner_data.provinces();
        for (i, line) in content.lines().enumerate() {
            let parts: Vec<&str> = line.split(';').collect();
            if parts.len() >= 7 {
                if let Ok(id) = parts[0].parse::<u32>() {
                    if !provs.is_empty() && !provs.contains_key(&id) {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: i as u32,
                                    character: 0,
                                },
                                end: Position {
                                    line: i as u32,
                                    character: 100,
                                },
                            },
                            severity: Some(DiagnosticSeverity::WARNING),
                            message: format!("Unknown province ID: {}", id),
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }

    async fn validate_definition_content(&self, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.split(';').collect();
            if parts.len() < 8 {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: i as u32,
                            character: 0,
                        },
                        end: Position {
                            line: i as u32,
                            character: line.len() as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("Expected at least 8 columns, found {}", parts.len()),
                    ..Default::default()
                });
                continue;
            }

            if parts[0].parse::<u32>().is_err() {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: i as u32,
                            character: 0,
                        },
                        end: Position {
                            line: i as u32,
                            character: parts[0].len() as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: "Province ID must be an integer".to_string(),
                    ..Default::default()
                });
            }

            for j in 1..=3 {
                if parts[j].parse::<u8>().is_err() {
                    let mut start_col = 0;
                    for part in parts.iter().take(j) {
                        start_col += part.len() as u32 + 1;
                    }
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: i as u32,
                                character: start_col,
                            },
                            end: Position {
                                line: i as u32,
                                character: start_col + parts[j].len() as u32,
                            },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: "Color component must be an integer between 0 and 255".to_string(),
                        ..Default::default()
                    });
                }
            }

            let p_type = parts[4].trim();
            if p_type != "land" && p_type != "sea" && p_type != "lake" {
                let mut start_col = 0;
                for part in parts.iter().take(4) {
                    start_col += part.len() as u32 + 1;
                }
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: i as u32,
                            character: start_col,
                        },
                        end: Position {
                            line: i as u32,
                            character: start_col + parts[4].len() as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: "Province type must be 'land', 'sea', or 'lake'".to_string(),
                    ..Default::default()
                });
            }

            let coastal = parts[5].trim();
            if coastal != "true" && coastal != "false" {
                let mut start_col = 0;
                for part in parts.iter().take(5) {
                    start_col += part.len() as u32 + 1;
                }
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: i as u32,
                            character: start_col,
                        },
                        end: Position {
                            line: i as u32,
                            character: start_col + parts[5].len() as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: "Coastal status must be 'true' or 'false'".to_string(),
                    ..Default::default()
                });
            }

            if parts[7].parse::<u32>().is_err() {
                let mut start_col = 0;
                for part in parts.iter().take(7) {
                    start_col += part.len() as u32 + 1;
                }
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: i as u32,
                            character: start_col,
                        },
                        end: Position {
                            line: i as u32,
                            character: start_col + parts[7].len() as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: "Continent must be an integer".to_string(),
                    ..Default::default()
                });
            }
        }
    }

    async fn validate_adjacencies_content(&self, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        let provs = self.scanner_data.provinces();
        let rules = self.scanner_data.adjacency_rules();
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("From;To;") {
                continue;
            }
            let parts: Vec<&str> = trimmed.split(';').collect();
            if parts.len() < 9 {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: i as u32,
                            character: 0,
                        },
                        end: Position {
                            line: i as u32,
                            character: line.len() as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("Expected at least 9 columns, found {}", parts.len()),
                    ..Default::default()
                });
                continue;
            }

            if parts.len() >= 9 {
                if let Ok(id) = parts[0].parse::<u32>() {
                    if !provs.is_empty() && !provs.contains_key(&id) {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: i as u32,
                                    character: 0,
                                },
                                end: Position {
                                    line: i as u32,
                                    character: parts[0].len() as u32,
                                },
                            },
                            severity: Some(DiagnosticSeverity::WARNING),
                            message: format!("Unknown start province ID: {}", id),
                            ..Default::default()
                        });
                    }
                } else {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: i as u32,
                                character: 0,
                            },
                            end: Position {
                                line: i as u32,
                                character: parts[0].len() as u32,
                            },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: "Start province ID must be an integer".to_string(),
                        ..Default::default()
                    });
                }

                let p1_len = parts[0].len() as u32 + 1;
                if let Ok(id) = parts[1].parse::<u32>() {
                    if !provs.is_empty() && !provs.contains_key(&id) {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: i as u32,
                                    character: p1_len,
                                },
                                end: Position {
                                    line: i as u32,
                                    character: p1_len + parts[1].len() as u32,
                                },
                            },
                            severity: Some(DiagnosticSeverity::WARNING),
                            message: format!("Unknown end province ID: {}", id),
                            ..Default::default()
                        });
                    }
                } else {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: i as u32,
                                character: p1_len,
                            },
                            end: Position {
                                line: i as u32,
                                character: p1_len + parts[1].len() as u32,
                            },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: "End province ID must be an integer".to_string(),
                        ..Default::default()
                    });
                }

                let mut p3_col = 0;
                for part in parts.iter().take(3) {
                    p3_col += part.len() as u32 + 1;
                }
                if let Ok(id) = parts[3].parse::<i32>() {
                    if id > 0 && !provs.is_empty() && !provs.contains_key(&(id as u32)) {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: i as u32,
                                    character: p3_col,
                                },
                                end: Position {
                                    line: i as u32,
                                    character: p3_col + parts[3].len() as u32,
                                },
                            },
                            severity: Some(DiagnosticSeverity::WARNING),
                            message: format!("Unknown through province ID: {}", id),
                            ..Default::default()
                        });
                    }

                    if parts[2].eq_ignore_ascii_case("sea") && id <= 0 {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: i as u32,
                                    character: p3_col,
                                },
                                end: Position {
                                    line: i as u32,
                                    character: p3_col + parts[3].len() as u32,
                                },
                            },
                            severity: Some(DiagnosticSeverity::HINT),
                            message: "Sea adjacencies usually require a Through province unless they directly border.".to_string(),
                            ..Default::default()
                        });
                    }
                } else if !parts[3].trim().is_empty() {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: i as u32,
                                character: p3_col,
                            },
                            end: Position {
                                line: i as u32,
                                character: p3_col + parts[3].len() as u32,
                            },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: "Through province ID must be an integer".to_string(),
                        ..Default::default()
                    });
                }

                // Check coords
                for j in 4..=7 {
                    if !parts[j].trim().is_empty() && parts[j].parse::<i32>().is_err() {
                        let mut start_col = 0;
                        for part in parts.iter().take(j) {
                            start_col += part.len() as u32 + 1;
                        }
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: i as u32,
                                    character: start_col,
                                },
                                end: Position {
                                    line: i as u32,
                                    character: start_col + parts[j].len() as u32,
                                },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: "Coordinate must be an integer".to_string(),
                            ..Default::default()
                        });
                    }
                }

                let p8_col = {
                    let mut c = 0;
                    for part in parts.iter().take(8) {
                        c += part.len() as u32 + 1;
                    }
                    c
                };
                let rule_name = parts[8].trim();
                if !rule_name.is_empty() && !rules.is_empty() && !rules.contains_key(rule_name) {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: i as u32,
                                character: p8_col,
                            },
                            end: Position {
                                line: i as u32,
                                character: p8_col + parts[8].len() as u32,
                            },
                        },
                        severity: Some(DiagnosticSeverity::WARNING),
                        message: format!("Unknown adjacency rule: {}", rule_name),
                        ..Default::default()
                    });
                }
            }
        }
    }

    async fn validate_adjacency_rules_content(
        &self,
        content: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let provs = self.scanner_data.provinces();
        let (script, errors) = parser::parse_script(content);
        for (msg, range) in errors {
            diagnostics.push(Diagnostic {
                range: ast_range_to_lsp(&range),
                severity: Some(DiagnosticSeverity::ERROR),
                message: msg,
                ..Default::default()
            });
        }

        for entry in script.entries {
            if let ast::Entry::Assignment(ass) = entry {
                if ass.key.eq_ignore_ascii_case("adjacency_rule") {
                    if let ast::Value::Block(rule_entries) = &ass.value.value {
                        for rule_entry in rule_entries {
                            if let ast::Entry::Assignment(r_ass) = rule_entry {
                                if r_ass.key.eq_ignore_ascii_case("required_provinces") {
                                    if let ast::Value::Block(prov_entries) = &r_ass.value.value {
                                        for p_entry in prov_entries {
                                            if let ast::Entry::Value(p_val) = p_entry {
                                                if let ast::Value::Number(n) = &p_val.value {
                                                    let prov_id = *n as u32;
                                                    if !provs.is_empty()
                                                        && !provs.contains_key(&prov_id)
                                                    {
                                                        diagnostics.push(Diagnostic {
                                                            range: ast_range_to_lsp(&p_val.range),
                                                            severity: Some(
                                                                DiagnosticSeverity::WARNING,
                                                            ),
                                                            message: format!(
                                                                "Unknown province ID: {}",
                                                                prov_id
                                                            ),
                                                            ..Default::default()
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    async fn validate_strategic_region_content(
        &self,
        script: &ast::Script,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let provs = self.scanner_data.provinces();

        for entry in &script.entries {
            if let ast::Entry::Assignment(ass) = entry {
                if ass.key.eq_ignore_ascii_case("strategic_region") {
                    if let ast::Value::Block(region_entries) = &ass.value.value {
                        for region_entry in region_entries {
                            if let ast::Entry::Assignment(r_ass) = region_entry {
                                if r_ass.key.eq_ignore_ascii_case("provinces") {
                                    if let ast::Value::Block(prov_entries) = &r_ass.value.value {
                                        for prov_entry in prov_entries {
                                            if let ast::Entry::Value(val) = prov_entry {
                                                if let ast::Value::Number(id) = &val.value {
                                                    let id_u32 = *id as u32;
                                                    if !provs.is_empty()
                                                        && !provs.contains_key(&id_u32)
                                                    {
                                                        diagnostics.push(Diagnostic {
                                                            range: ast_range_to_lsp(&val.range),
                                                            severity: Some(
                                                                DiagnosticSeverity::WARNING,
                                                            ),
                                                            message: format!(
                                                                "Unknown province ID: {}",
                                                                id_u32
                                                            ),
                                                            ..Default::default()
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    } else if let ast::Value::TaggedBlock(_, prov_entries, _) =
                                        &r_ass.value.value
                                    {
                                        for prov_entry in prov_entries {
                                            if let ast::Entry::Value(val) = prov_entry {
                                                if let ast::Value::Number(id) = &val.value {
                                                    let id_u32 = *id as u32;
                                                    if !provs.is_empty()
                                                        && !provs.contains_key(&id_u32)
                                                    {
                                                        diagnostics.push(Diagnostic {
                                                            range: ast_range_to_lsp(&val.range),
                                                            severity: Some(
                                                                DiagnosticSeverity::WARNING,
                                                            ),
                                                            message: format!(
                                                                "Unknown province ID: {}",
                                                                id_u32
                                                            ),
                                                            ..Default::default()
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    async fn validate_localization_content(
        &self,
        uri: &Uri,
        content: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let path_str = uri
            .to_file_path()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let (parsed, loc_diagnostics_structural, doc_lang) =
            loc_parser::parse_loc_file(content, &path_str);
        let doc_lang_str = doc_lang.unwrap_or_else(|| "unknown".to_string());
        let event_targets = self.scanner_data.event_targets();
        let scripted_locs = self.scanner_data.scripted_locs();
        let color_codes = self.scanner_data.color_codes();
        let dups = self.scanner_data.duplicated_loc_keys();

        // Add structural diagnostics
        for d in loc_diagnostics_structural {
            diagnostics.push(Diagnostic {
                range: ast_range_to_lsp(&d.range),
                severity: Some(match d.severity {
                    ast::DiagnosticSeverity::Error => DiagnosticSeverity::ERROR,
                    ast::DiagnosticSeverity::Warning => DiagnosticSeverity::WARNING,
                    ast::DiagnosticSeverity::Information => DiagnosticSeverity::INFORMATION,
                    ast::DiagnosticSeverity::Hint => DiagnosticSeverity::HINT,
                }),
                message: d.message,
                code: d.code.map(NumberOrString::String),
                source: Some("Hearts of Modding".to_string()),
                tags: if d.tags.is_empty() {
                    None
                } else {
                    Some(d.tags.iter().map(ast_tag_to_lsp).collect())
                },
                related_information: if d.related_information.is_empty() {
                    None
                } else {
                    Some(
                        d.related_information
                            .iter()
                            .map(ast_related_info_to_lsp)
                            .collect(),
                    )
                },
                ..Default::default()
            });
        }

        for entry in parsed.values() {
            // Check for unnecessary version numbers
            if let Some(d) = loc_parser::check_unnecessary_version(entry) {
                diagnostics.push(Diagnostic {
                    range: ast_range_to_lsp(&d.range),
                    severity: Some(match d.severity {
                        ast::DiagnosticSeverity::Error => DiagnosticSeverity::ERROR,
                        ast::DiagnosticSeverity::Warning => DiagnosticSeverity::WARNING,
                        ast::DiagnosticSeverity::Information => DiagnosticSeverity::INFORMATION,
                        ast::DiagnosticSeverity::Hint => DiagnosticSeverity::HINT,
                    }),
                    message: d.message,
                    code: d.code.map(NumberOrString::String),
                    source: Some("Hearts of Modding".to_string()),
                    tags: if d.tags.is_empty() {
                        None
                    } else {
                        Some(d.tags.iter().map(ast_tag_to_lsp).collect())
                    },
                    related_information: if d.related_information.is_empty() {
                        None
                    } else {
                        Some(
                            d.related_information
                                .iter()
                                .map(ast_related_info_to_lsp)
                                .collect(),
                        )
                    },
                    ..Default::default()
                });
            }

            let color_code_set: std::collections::HashSet<String> =
                color_codes.keys().cloned().collect();
            let loc_diagnostics = loc_parser::validate_loc_string(
                entry,
                &event_targets,
                &scripted_locs,
                &color_code_set,
            );
            for d in loc_diagnostics {
                diagnostics.push(Diagnostic {
                    range: ast_range_to_lsp(&d.range),
                    severity: Some(match d.severity {
                        ast::DiagnosticSeverity::Error => DiagnosticSeverity::ERROR,
                        ast::DiagnosticSeverity::Warning => DiagnosticSeverity::WARNING,
                        ast::DiagnosticSeverity::Information => DiagnosticSeverity::INFORMATION,
                        ast::DiagnosticSeverity::Hint => DiagnosticSeverity::HINT,
                    }),
                    message: d.message,
                    code: d.code.map(NumberOrString::String),
                    source: Some("Hearts of Modding".to_string()),
                    tags: if d.tags.is_empty() {
                        None
                    } else {
                        Some(d.tags.iter().map(ast_tag_to_lsp).collect())
                    },
                    related_information: if d.related_information.is_empty() {
                        None
                    } else {
                        Some(
                            d.related_information
                                .iter()
                                .map(ast_related_info_to_lsp)
                                .collect(),
                        )
                    },
                    ..Default::default()
                });
            }

            // Check for duplicated localization keys across files
            let is_duplicated = dups.contains(&(doc_lang_str.clone(), entry.key.clone()));

            if is_duplicated {
                let loc_map = self.scanner_data.localization();
                let mut is_intentional_override = false;
                if entry.path.contains("replace") {
                    is_intentional_override = true;
                } else if let Some(existing) = loc_map.get(&entry.key) {
                    if existing.path.contains("replace") {
                        is_intentional_override = true;
                    }
                }

                if !is_intentional_override {
                    diagnostics.push(Diagnostic {
                        range: ast_range_to_lsp(&entry.range),
                        severity: Some(DiagnosticSeverity::WARNING),
                        message: format!("Duplicate localization key found: '{}'. The game will only use one of them unless one is in a 'replace' folder.", entry.key),
                        source: Some("Hearts of Modding".to_string()),
                        code: Some(NumberOrString::String("duplicate_loc_key".to_string())),
                        ..Default::default()
                    });
                }
            }
        }
    }

    pub(crate) fn compute_expected_indentations(
        entries: &[ast::Entry],
        depth: usize,
        expected: &mut HashMap<u32, usize>,
    ) {
        for entry in entries {
            let start_line = match entry {
                ast::Entry::Assignment(ass) => ass.key_range.start_line,
                ast::Entry::Value(val) => val.range.start_line,
                ast::Entry::Comment(_, r) => r.start_line,
            };

            expected.entry(start_line).or_insert(depth);

            match entry {
                ast::Entry::Assignment(ass) => match &ass.value.value {
                    ast::Value::Block(inner) => {
                        Self::compute_expected_indentations(inner, depth + 1, expected);
                        let end_line = ass.value.range.end_line;
                        if end_line != start_line {
                            expected.entry(end_line).or_insert(depth);
                        }
                    }
                    ast::Value::TaggedBlock(_, inner, _) => {
                        Self::compute_expected_indentations(inner, depth + 1, expected);
                        let end_line = ass.value.range.end_line;
                        if end_line != start_line {
                            expected.entry(end_line).or_insert(depth);
                        }
                    }
                    _ => {}
                },
                ast::Entry::Value(val) => match &val.value {
                    ast::Value::Block(inner) => {
                        Self::compute_expected_indentations(inner, depth + 1, expected);
                        let end_line = val.range.end_line;
                        if end_line != start_line {
                            expected.entry(end_line).or_insert(depth);
                        }
                    }
                    ast::Value::TaggedBlock(_, inner, _) => {
                        Self::compute_expected_indentations(inner, depth + 1, expected);
                        let end_line = val.range.end_line;
                        if end_line != start_line {
                            expected.entry(end_line).or_insert(depth);
                        }
                    }
                    _ => {}
                },
                ast::Entry::Comment(_, _) => {}
            }
        }
    }

    fn check_single_line_braces(
        entries: &[ast::Entry],
        content: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for entry in entries {
            match entry {
                ast::Entry::Assignment(ass) => {
                    Self::check_brace_spacing_for_range(
                        &ass.value.range,
                        &ass.value.value,
                        content,
                        diagnostics,
                    );
                    match &ass.value.value {
                        ast::Value::Block(inner) => {
                            Self::check_single_line_braces(inner, content, diagnostics)
                        }
                        ast::Value::TaggedBlock(_, inner, _) => {
                            Self::check_single_line_braces(inner, content, diagnostics)
                        }
                        _ => {}
                    }
                }
                ast::Entry::Value(val) => {
                    Self::check_brace_spacing_for_range(
                        &val.range,
                        &val.value,
                        content,
                        diagnostics,
                    );
                    match &val.value {
                        ast::Value::Block(inner) => {
                            Self::check_single_line_braces(inner, content, diagnostics)
                        }
                        ast::Value::TaggedBlock(_, inner, _) => {
                            Self::check_single_line_braces(inner, content, diagnostics)
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    fn check_brace_spacing_for_range(
        range: &ast::Range,
        value: &ast::Value,
        content: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match value {
            ast::Value::Block(_) | ast::Value::TaggedBlock(_, _, _)
                if range.start_line == range.end_line =>
            {
                let line_idx = range.start_line as usize;
                if let Some(line) = content.lines().nth(line_idx) {
                    let start = range.start_col as usize;
                    let end = range.end_col as usize;
                    if start < end && end <= line.len() {
                        let full_str = &line[start..end];
                        if let Some(brace_start_rel) = full_str.find('{') {
                            let mut needs_fix = false;
                            let mut message = "Single-line block should have exactly one space padding inside curly braces.";

                            // 1. Check space BEFORE { if it's a TaggedBlock
                            if let ast::Value::TaggedBlock(tag, _, _) = value {
                                if &full_str[tag.len()..brace_start_rel] != " " {
                                    needs_fix = true;
                                    message = "Single-line block should have exactly one space around curly braces.";
                                }
                            }

                            // 2. Check padding INSIDE
                            let block_str = &full_str[brace_start_rel..];
                            if block_str.len() >= 2 {
                                let inner = &block_str[1..block_str.len() - 1];
                                if inner.trim().is_empty() {
                                    if block_str != "{}" {
                                        needs_fix = true;
                                        message = "Empty single-line block should be '{}' without spaces.";
                                    }
                                } else {
                                    if !block_str.starts_with("{ ")
                                        || !block_str.ends_with(" }")
                                        || block_str.starts_with("{  ")
                                        || block_str.ends_with("  }")
                                    {
                                        needs_fix = true;
                                    }
                                }
                            }

                            if needs_fix {
                                diagnostics.push(Diagnostic {
                                    range: ast_range_to_lsp(range),
                                    severity: Some(DiagnosticSeverity::INFORMATION),
                                    code: Some(NumberOrString::String(
                                        "styling_brace_space".to_string(),
                                    )),
                                    message: message.to_string(),
                                    source: Some("Hearts of Modding".to_string()),
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn check_styling(
        &self,
        content: &str,
        script_opt: Option<&ast::Script>,
        diagnostics: &mut Vec<Diagnostic>,
        is_yaml: bool,
        uri: &str,
    ) {
        if !content.is_empty()
            && !content.ends_with('\n')
            && !content.ends_with("\r\n")
            && !uri.ends_with("map/buildings.txt")
        {
            let line_count = content.lines().count();
            let last_line = content.lines().last().unwrap_or("");
            let line_idx = if line_count > 0 {
                line_count as u32 - 1
            } else {
                0
            };
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position {
                        line: line_idx,
                        character: last_line.len() as u32,
                    },
                    end: Position {
                        line: line_idx,
                        character: last_line.len() as u32,
                    },
                },
                severity: Some(DiagnosticSeverity::INFORMATION),
                code: Some(NumberOrString::String("styling_eof_newline".to_string())),
                message: "File should end with an empty newline.".to_string(),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        }

        let mut expected_indents = HashMap::new();
        if let Some(script) = script_opt {
            Self::compute_expected_indentations(&script.entries, 0, &mut expected_indents);
            Self::check_single_line_braces(&script.entries, content, diagnostics);
        }

        for (line_idx, line) in content.lines().enumerate() {
            // Skip styling checks for CSV files as they have their own formatting rules
            if uri.ends_with(".csv") {
                continue;
            }
            // 1. Trailing whitespace
            if line.ends_with(' ') || line.ends_with('\t') {
                let trimmed_len = line.trim_end().len();
                let start_col = utf16_len(&line[..trimmed_len]);
                let end_col = utf16_len(line);
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: line_idx as u32,
                            character: start_col,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: end_col,
                        },
                    },
                    severity: Some(DiagnosticSeverity::INFORMATION),
                    code: Some(NumberOrString::String("styling_trailing".to_string())),
                    message: "Trailing whitespace detected.".to_string(),
                    source: Some("Hearts of Modding".to_string()),
                    ..Default::default()
                });
            }

            // 2. Indentation consistency
            let leading = line
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect::<String>();
            if line.trim().is_empty() {
                continue; // Skip empty lines for indentation checking
            }

            // For YAML localization files, allow flexible indentation after the first tab
            if is_yaml {
                // Skip header line (l_english:)
                let trimmed = line.trim();
                if trimmed.starts_with("l_") && trimmed.contains(':') {
                    continue;
                }

                // Skip comments
                if trimmed.starts_with('#') {
                    continue;
                }

                // For content lines, require at least one tab at the start
                if !leading.is_empty() && !leading.starts_with('\t') {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: line_idx as u32,
                                character: 0,
                            },
                            end: Position {
                                line: line_idx as u32,
                                character: leading.len() as u32,
                            },
                        },
                        severity: Some(DiagnosticSeverity::INFORMATION),
                        code: Some(NumberOrString::String("styling_indent".to_string())),
                        message: "Localization entries must start with at least one tab."
                            .to_string(),
                        source: Some("Hearts of Modding".to_string()),
                        ..Default::default()
                    });
                }
                // Allow any additional indentation after the first tab for cosmetic alignment
            } else {
                // Regular script files: strict tab-only indentation
                if let Some(&expected_tabs) = expected_indents.get(&(line_idx as u32)) {
                    let expected_str = "\t".repeat(expected_tabs);
                    if leading != expected_str {
                        let mut data = serde_json::Map::new();
                        data.insert(
                            "expected_tabs".to_string(),
                            serde_json::Value::Number(expected_tabs.into()),
                        );

                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: line_idx as u32,
                                    character: 0,
                                },
                                end: Position {
                                    line: line_idx as u32,
                                    character: leading.len() as u32,
                                },
                            },
                            severity: Some(DiagnosticSeverity::INFORMATION),
                            code: Some(NumberOrString::String("styling_indent".to_string())),
                            message: format!(
                                "Inconsistent indentation. Expected {} tab(s).",
                                expected_tabs
                            ),
                            source: Some("Hearts of Modding".to_string()),
                            data: Some(serde_json::Value::Object(data)),
                            ..Default::default()
                        });
                    }
                } else if leading.contains(' ') {
                    // Fallback if line wasn't in AST (e.g. unparsed strings or comments)
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: line_idx as u32,
                                character: 0,
                            },
                            end: Position {
                                line: line_idx as u32,
                                character: leading.len() as u32,
                            },
                        },
                        severity: Some(DiagnosticSeverity::INFORMATION),
                        code: Some(NumberOrString::String("styling_indent".to_string())),
                        message: "Spaces used in indentation. Please use tabs.".to_string(),
                        source: Some("Hearts of Modding".to_string()),
                        ..Default::default()
                    });
                }
            }
        }
    }

    async fn check_semantic(
        &self,
        script: &ast::Script,
        diagnostics: &mut Vec<Diagnostic>,
        styling_enabled: bool,
        uri: &str,
    ) {
        let loc = self.scanner_data.localization();
        let st = self.scanner_data.scripted_triggers();
        let se = self.scanner_data.scripted_effects();
        let id = self.scanner_data.ideologies();
        let sid = self.scanner_data.sub_ideologies();
        let tr = self.scanner_data.traits();
        let sp = self.scanner_data.sprites();
        let ids = self.scanner_data.ideas();
        let provs = self.scanner_data.provinces();
        let mod_maps = self.scanner_data.modifier_mappings();
        let ig_loc = self.config.ignored_loc_regex();
        let buildings = self.scanner_data.buildings();
        let defines = self.scanner_data.defines();
        let s_effects = self.scanner_data.sound_effects();
        let ct = self.scanner_data.country_tags();

        let mut comments = Vec::new();
        for entry in &script.entries {
            if let ast::Entry::Comment(c, r) = entry {
                comments.push((c.clone(), r.clone()));
            }
        }

        // Detect file type from URI for scope inference
        let initial_scope = if uri.contains("/common/abilities/") {
            scope::Scope::Character
        } else {
            scope::Scope::Global
        };
        let mut scope_stack = scope::ScopeStack::new(initial_scope);

        // Load AI area validation data
        let continents = self.scanner_data.continents();
        let strategic_regions = self.scanner_data.strategic_regions();

        // Run advanced validations
        let mut advanced_diags = Vec::new();

        // Dynamic country tag check: warn if the file is in common/country_tags/ and
        // the dynamic-to-static ratio suggests insufficient dynamic tags for civil wars.
        if uri.contains("/common/country_tags/") || uri.contains("\\common\\country_tags\\") {
            let total = ct.len();
            let dynamic_count = ct.values().filter(|t| t.dynamic).count();
            let static_count = total - dynamic_count;
            if total > 0 && dynamic_count == 0 {
                advanced_diags.push(advanced_validation::ValidationDiagnostic {
                    range: ast::Range { start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
                    severity: ast::DiagnosticSeverity::Warning,
                    message: "No dynamic country tags defined. Civil wars will fail for lack of dynamic tags, potentially causing a crash.".to_string(),
                    code: "HOM5001".to_string(),
                    fix_suggestion: None,
                    related_information: vec![],
                    tags: vec![],
                });
            } else if static_count > 10 && dynamic_count < (static_count / 10).max(3) {
                advanced_diags.push(advanced_validation::ValidationDiagnostic {
                    range: ast::Range { start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
                    severity: ast::DiagnosticSeverity::Information,
                    message: format!("Only {} dynamic tags for {} static tags. Consider adding more dynamic tags for civil wars.", dynamic_count, static_count),
                    code: "HOM5002".to_string(),
                    fix_suggestion: None,
                    related_information: vec![],
                    tags: vec![],
                });
            }
        }
        advanced_validation::validate_building_levels(
            &script.entries,
            &buildings,
            &mut advanced_diags,
        );
        advanced_validation::validate_character_skills(
            &script.entries,
            &defines,
            &mut advanced_diags,
        );
        advanced_validation::validate_victory_points(&script.entries, &mut advanced_diags);
        advanced_validation::validate_achievements(&script.entries, &loc, &mut advanced_diags);
        advanced_validation::validate_abilities(&script.entries, &loc, &mut advanced_diags);
        advanced_validation::validate_portrait_gfx(&script.entries, &sp, &mut advanced_diags);

        // Convert advanced diagnostics to LSP diagnostics
        for diag in advanced_diags {
            diagnostics.push(Diagnostic {
                range: ast_range_to_lsp(&diag.range),
                severity: Some(match diag.severity {
                    ast::DiagnosticSeverity::Error => DiagnosticSeverity::ERROR,
                    ast::DiagnosticSeverity::Warning => DiagnosticSeverity::WARNING,
                    ast::DiagnosticSeverity::Information => DiagnosticSeverity::INFORMATION,
                    ast::DiagnosticSeverity::Hint => DiagnosticSeverity::HINT,
                }),
                message: diag.message,
                code: Some(NumberOrString::String(diag.code)),
                source: Some("Hearts of Modding".to_string()),
                tags: if diag.tags.is_empty() {
                    None
                } else {
                    Some(diag.tags.iter().map(ast_tag_to_lsp).collect())
                },
                related_information: if diag.related_information.is_empty() {
                    None
                } else {
                    Some(
                        diag.related_information
                            .iter()
                            .map(ast_related_info_to_lsp)
                            .collect(),
                    )
                },
                data: diag.fix_suggestion.map(|s| serde_json::json!({ "fix": s })),
                ..Default::default()
            });
        }

        // AI area validation: verify continents and strategic regions exist when editing AI area files
        if uri.contains("/common/ai_areas/") || uri.contains("\\common\\ai_areas\\") {
            for entry in &script.entries {
                if let ast::Entry::Assignment(ass) = entry {
                    if let ast::Value::Block(inner_entries) = &ass.value.value {
                        for inner in inner_entries {
                            if let ast::Entry::Assignment(inner_ass) = inner {
                                match inner_ass.key.as_str() {
                                    "continents" => {
                                        if let ast::Value::Block(cont_entries) =
                                            &inner_ass.value.value
                                        {
                                            for ce in cont_entries {
                                                if let ast::Entry::Value(val) = ce {
                                                    if let ast::Value::String(name) = &val.value {
                                                        if !continents.contains_key(name) {
                                                            diagnostics.push(Diagnostic {
                                                                range: ast_range_to_lsp(&val.range),
                                                                severity: Some(
                                                                    DiagnosticSeverity::WARNING,
                                                                ),
                                                                message: format!(
                                                                    "Unknown continent: '{}'",
                                                                    name
                                                                ),
                                                                code: Some(NumberOrString::String(
                                                                    "HOM6001".to_string(),
                                                                )),
                                                                source: Some(
                                                                    "Hearts of Modding".to_string(),
                                                                ),
                                                                ..Default::default()
                                                            });
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    "strategic_regions" => {
                                        if let ast::Value::Block(sr_entries) =
                                            &inner_ass.value.value
                                        {
                                            for se in sr_entries {
                                                if let ast::Entry::Value(val) = se {
                                                    if let ast::Value::Number(n) = &val.value {
                                                        let id = *n as u32;
                                                        if !strategic_regions.contains_key(&id) {
                                                            diagnostics.push(Diagnostic {
                                                                range: ast_range_to_lsp(&val.range),
                                                                severity: Some(
                                                                    DiagnosticSeverity::WARNING,
                                                                ),
                                                                message: format!(
                                                                    "Unknown strategic region: {}",
                                                                    id
                                                                ),
                                                                code: Some(NumberOrString::String(
                                                                    "HOM6002".to_string(),
                                                                )),
                                                                source: Some(
                                                                    "Hearts of Modding".to_string(),
                                                                ),
                                                                ..Default::default()
                                                            });
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }

        for entry in &script.entries {
            self.check_entry_semantic(
                entry,
                diagnostics,
                &loc,
                &st,
                &se,
                &id,
                &sid,
                &tr,
                &sp,
                &ids,
                &provs,
                &mod_maps,
                &ig_loc,
                &comments,
                styling_enabled,
                &mut scope_stack,
                &s_effects,
                &ct,
            );
        }

        // Texture file path validation for .gfx and .gui files
        if uri.ends_with(".gfx") || uri.ends_with(".gui") {
            if let Ok(url) = uri.parse::<Uri>() {
                if let Some(gfx_path) = url.to_file_path() {
                    self.validate_gfx_texture_paths(
                        &script.entries,
                        diagnostics,
                        styling_enabled,
                        &gfx_path,
                    );
                }
            }
        }
    }

    fn validate_gfx_texture_paths(
        &self,
        entries: &[ast::Entry],
        diagnostics: &mut Vec<Diagnostic>,
        styling_enabled: bool,
        gfx_file_path: &std::path::Path,
    ) {
        let game_path = self.config.game_path();
        let gfx_dir = gfx_file_path.parent();

        for entry in entries {
            match entry {
                ast::Entry::Assignment(ass) => {
                    let key_lower = ass.key.to_ascii_lowercase();
                    if key_lower == "texturefile" {
                        if let ast::Value::String(val) = &ass.value.value {
                            let has_double_slash = val.contains("//");
                            let has_backslash = val.contains('\\');

                            // Styling: non-standard path separators
                            if styling_enabled && (has_double_slash || has_backslash) {
                                let suggestion = val.replace("//", "/").replace('\\', "/");
                                diagnostics.push(Diagnostic {
                                    range: ast_range_to_lsp(&ass.value.range),
                                    severity: Some(DiagnosticSeverity::INFORMATION),
                                    code: Some(NumberOrString::String(
                                        "styling_path_separator".to_string(),
                                    )),
                                    message: format!(
                                        "Use single forward slashes in texture paths. Suggestion: '{}'.",
                                        suggestion
                                    ),
                                    source: Some("Hearts of Modding".to_string()),
                                    data: Some(serde_json::to_value(suggestion).unwrap()),
                                    ..Default::default()
                                });
                            }

                            // Existence check: try resolving the texture file
                            let normalized = val.replace('\\', "/");
                            let mut found = false;

                            // Try relative to game path
                            if let Some(ref gp) = game_path {
                                let full = std::path::Path::new(gp).join(&normalized);
                                if full.exists() {
                                    found = true;
                                }
                            }

                            // Try relative to .gfx file directory
                            if !found {
                                if let Some(dir) = gfx_dir {
                                    let full = dir.join(&normalized);
                                    if full.exists() {
                                        found = true;
                                    }
                                }
                            }

                            if !found {
                                diagnostics.push(Diagnostic {
                                    range: ast_range_to_lsp(&ass.value.range),
                                    severity: Some(DiagnosticSeverity::WARNING),
                                    message: format!("Texture file not found: '{}'", val),
                                    source: Some("Hearts of Modding".to_string()),
                                    ..Default::default()
                                });
                            }
                        }
                    }

                    // Recurse into blocks
                    match &ass.value.value {
                        ast::Value::Block(entries) | ast::Value::TaggedBlock(_, entries, _) => {
                            self.validate_gfx_texture_paths(
                                entries,
                                diagnostics,
                                styling_enabled,
                                gfx_file_path,
                            );
                        }
                        _ => {}
                    }
                }
                ast::Entry::Value(val) => match &val.value {
                    ast::Value::Block(entries) | ast::Value::TaggedBlock(_, entries, _) => {
                        self.validate_gfx_texture_paths(
                            entries,
                            diagnostics,
                            styling_enabled,
                            gfx_file_path,
                        );
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn check_entry_semantic(
        &self,
        entry: &ast::Entry,
        diagnostics: &mut Vec<Diagnostic>,
        loc: &HashMap<String, loc_parser::LocEntry>,
        st: &HashMap<String, scripted_scanner::ScriptedEntity>,
        se: &HashMap<String, scripted_scanner::ScriptedEntity>,
        id: &HashMap<String, ideology_scanner::Ideology>,
        sid: &HashMap<String, (String, ast::Range, String)>,
        tr: &HashMap<String, trait_scanner::Trait>,
        sp: &HashMap<String, sprite_scanner::Sprite>,
        ids: &HashMap<String, idea_scanner::Idea>,
        provs: &HashMap<u32, province_scanner::Province>,
        mod_maps: &HashMap<String, String>,
        ig_loc: &[regex::Regex],
        comments: &[(String, ast::Range)],
        styling_enabled: bool,
        scope_stack: &mut scope::ScopeStack,
        s_effects: &HashMap<String, sound_scanner::SoundEffect>,
        ct: &HashMap<String, country_scanner::CountryTag>,
    ) {
        match entry {
            ast::Entry::Assignment(ass) => {
                let key_lower = ass.key.to_ascii_lowercase();
                let mut pushed_scope = false;

                // Structural blocks that push scope
                let mut s = scope::Scope::from_str(&ass.key);

                // Internal 'idea' definition block context
                if s == scope::Scope::Unknown {
                    let stack = scope_stack.stack();
                    if stack.contains(&scope::Scope::Idea) {
                        // We are inside 'ideas'. If depth is 2 (Global, Idea), this is a category.
                        // If depth is 3 (Global, Idea, Category), this is an idea definition.
                        if stack.len() == 2 || stack.len() == 3 {
                            s = scope::Scope::Idea;
                        }
                    }
                }

                if s != scope::Scope::Unknown || ass.key.contains(':') || ass.key.contains('.') {
                    match &ass.value.value {
                        ast::Value::Block(entries) | ast::Value::TaggedBlock(_, entries, _) => {
                            // Default picture check for ideas: If omitted, defaults to GFX_idea_[idea_name]
                            if s == scope::Scope::Idea && scope_stack.stack().len() == 3 {
                                let has_picture = entries.iter().any(|e| {
                                    if let ast::Entry::Assignment(a) = e {
                                        a.key.eq_ignore_ascii_case("picture")
                                    } else {
                                        false
                                    }
                                });
                                if !has_picture {
                                    let default_gfx = format!("GFX_idea_{}", ass.key);
                                    let exists = sp.contains_key(&default_gfx)
                                        || sp
                                            .keys()
                                            .any(|k| k.starts_with(&format!("{}_", default_gfx)));
                                    if !exists {
                                        diagnostics.push(Diagnostic {
                                            range: ast_range_to_lsp(&ass.key_range),
                                            severity: Some(DiagnosticSeverity::WARNING),
                                            message: format!("Idea '{}' is missing a 'picture' field and the default GFX '{}' was not found.", ass.key, default_gfx),
                                            source: Some("Hearts of Modding".to_string()),
                                            ..Default::default()
                                        });
                                    }
                                }
                            }

                            scope_stack.push(s);
                            pushed_scope = true;
                        }
                        _ => {}
                    }
                }

                // Casing checks for keywords
                if styling_enabled {
                    let mut needs_fix = false;
                    if ass.key_range.end_line == ass.operator_range.start_line
                        && ass.key_range.end_line == ass.value.range.start_line
                    {
                        if ass.operator_range.start_col > ass.key_range.end_col
                            && ass.value.range.start_col > ass.operator_range.end_col
                        {
                            let space_before = ass.operator_range.start_col - ass.key_range.end_col;
                            let space_after =
                                ass.value.range.start_col - ass.operator_range.end_col;
                            if space_before != 1 || space_after != 1 {
                                needs_fix = true;
                            }
                        } else {
                            needs_fix = true;
                        }
                    }

                    if needs_fix {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position { line: ass.key_range.end_line, character: ass.key_range.end_col },
                                end: Position { line: ass.value.range.start_line, character: ass.value.range.start_col },
                            },
                            severity: Some(DiagnosticSeverity::INFORMATION),
                            code: Some(NumberOrString::String("styling_assignment_space".to_string())),
                            message: "Assignment operator should be surrounded by exactly one space on each side.".to_string(),
                            source: Some("Hearts of Modding".to_string()),
                            ..Default::default()
                        });
                    }

                    // Brace newline check
                    match &ass.value.value {
                        ast::Value::Block(_)
                            if ass.value.range.start_line > ass.operator_range.end_line =>
                        {
                            diagnostics.push(Diagnostic {
                                range: Range {
                                    start: Position {
                                        line: ass.operator_range.end_line,
                                        character: ass.operator_range.end_col,
                                    },
                                    end: Position {
                                        line: ass.value.range.start_line,
                                        character: ass.value.range.start_col,
                                    },
                                },
                                severity: Some(DiagnosticSeverity::INFORMATION),
                                code: Some(NumberOrString::String(
                                    "styling_brace_newline".to_string(),
                                )),
                                message: "Curly brace should not be on a new line.".to_string(),
                                source: Some("Hearts of Modding".to_string()),
                                ..Default::default()
                            });
                        }
                        ast::Value::TaggedBlock(tag, _, block_range) => {
                            // Check if the brace part of the tagged block is on a new line
                            // Usually TaggedBlock range starts at the tag.
                            // We check if the block_range starts on a new line compared to where the tag/operator is.
                            if block_range.start_line > ass.operator_range.end_line {
                                diagnostics.push(Diagnostic {
                                    range: Range {
                                        start: Position {
                                            line: ass.operator_range.end_line,
                                            character: ass.operator_range.end_col,
                                        },
                                        end: Position {
                                            line: block_range.start_line,
                                            character: block_range.start_col,
                                        },
                                    },
                                    severity: Some(DiagnosticSeverity::INFORMATION),
                                    code: Some(NumberOrString::String(
                                        "styling_brace_newline".to_string(),
                                    )),
                                    message: "Curly brace should not be on a new line.".to_string(),
                                    source: Some("Hearts of Modding".to_string()),
                                    ..Default::default()
                                });
                            } else {
                                // Same line, check space between tag and brace
                                let tag_end_col = ass.value.range.start_col + tag.len() as u32;
                                if block_range.start_col != tag_end_col + 1 {
                                    diagnostics.push(Diagnostic {
                                        range: Range {
                                            start: Position { line: ass.value.range.start_line, character: tag_end_col },
                                            end: Position { line: block_range.start_line, character: block_range.start_col },
                                        },
                                        severity: Some(DiagnosticSeverity::INFORMATION),
                                        code: Some(NumberOrString::String("styling_brace_newline".to_string())), // Also use this code for easy fix
                                        message: "Exactly one space should separate the tag and the curly brace.".to_string(),
                                        source: Some("Hearts of Modding".to_string()),
                                        ..Default::default()
                                    });
                                }
                            }
                        }
                        _ => {}
                    }

                    let keywords = [
                        "spriteTypes",
                        "spriteType",
                        "name",
                        "texturefile",
                        "ideologies",
                        "types",
                        "ideas",
                        "country",
                        "national_focus",
                        "leader_traits",
                        "country_leader_traits",
                        "traits",
                        "orientation",
                        "buttonType",
                    ];

                    for kw in keywords {
                        if key_lower == kw && ass.key != kw {
                            let mut message = format!(
                                "Standard Paradox Script convention uses '{}' instead of '{}'.",
                                kw, ass.key
                            );
                            if kw.contains("sprite") || kw == "texturefile" {
                                message.push_str(
                                    "\nReference: https://hoi4.paradoxwikis.com/Modding#GFX",
                                );
                            } else if kw == "orientation" || kw == "buttonType" {
                                message.push_str(
                                    "\nReference: https://hoi4.paradoxwikis.com/Interface_modding",
                                );
                            }

                            diagnostics.push(Diagnostic {
                                range: ast_range_to_lsp(&ass.key_range),
                                severity: Some(DiagnosticSeverity::INFORMATION),
                                code: Some(NumberOrString::String("casing".to_string())),
                                message,
                                source: Some("Hearts of Modding".to_string()),
                                data: Some(serde_json::to_value(kw).unwrap()),
                                ..Default::default()
                            });
                            break;
                        }
                    }
                }

                // Localization checks
                if key_lower == "name"
                    || key_lower == "desc"
                    || key_lower == "text"
                    || key_lower == "title"
                {
                    if let ast::Value::String(val) = &ass.value.value {
                        let mut should_flag = true;

                        // 1. Basic heuristics (Space, numbers)
                        if val.contains(' ')
                            || val.is_empty()
                            || val.chars().all(|c| c.is_numeric())
                        {
                            should_flag = false;
                        }

                        // 2. Casing heuristic: If it starts with a capital and isn't all caps, it's likely a literal
                        if should_flag && val.chars().next().is_some_and(|c| c.is_uppercase()) {
                            let all_caps = val.chars().all(|c| !c.is_lowercase());
                            if !all_caps {
                                should_flag = false;
                            }
                        }

                        // 3. Comment suppression (# ignore)
                        if should_flag {
                            for (comment_text, range) in comments {
                                if range.start_line == ass.key_range.start_line {
                                    if comment_text.to_ascii_lowercase().contains("ignore") {
                                        should_flag = false;
                                        break;
                                    }
                                }
                            }
                        }

                        if should_flag {
                            if !loc.contains_key(val) {
                                let target = format!("{}:", val);
                                if !loc.iter().any(|(k, _)| k.starts_with(&target)) {
                                    // Final check against regex
                                    let is_regex_ignored = ig_loc.iter().any(|re| re.is_match(val));

                                    if !is_regex_ignored {
                                        diagnostics.push(Diagnostic {
                                            range: ast_range_to_lsp(&ass.value.range),
                                            severity: Some(DiagnosticSeverity::HINT), // Use HINT for lenient keys
                                            message: format!(
                                                "Missing localization key: '{}' (or literal name)",
                                                val
                                            ),
                                            code: Some(NumberOrString::String(
                                                advanced_validation::MISSING_LOCALIZATION
                                                    .to_string(),
                                            )),
                                            source: Some("Hearts of Modding".to_string()),
                                            ..Default::default()
                                        });
                                    }
                                }
                            }
                        }
                    }
                }

                // Strict ID checks (Warning level)
                // Ideology checks
                if key_lower == "ideology" || key_lower == "has_ideology" {
                    if let ast::Value::String(val) = &ass.value.value {
                        // Allow scoped references (ROOT, FROM, PREV, THIS, etc.) which resolve at runtime
                        let is_scope_ref = matches!(
                            val.to_uppercase().as_str(),
                            "ROOT"
                                | "FROM"
                                | "PREV"
                                | "THIS"
                                | "PREVPREV"
                                | "PREVPREVPREV"
                                | "PREVPREVPREVPREV"
                                | "OWNER"
                                | "CONTROLLER"
                                | "CAPITAL"
                                | "FROM.FROM"
                                | "FROM.FROM.FROM"
                        );
                        // Allow variable references (var:SCOPE@name or var:name) which resolve at runtime
                        let is_var_ref = val.starts_with("var:");
                        if !id.contains_key(val)
                            && !sid.contains_key(val)
                            && !is_scope_ref
                            && !is_var_ref
                        {
                            diagnostics.push(Diagnostic {
                                range: ast_range_to_lsp(&ass.value.range),
                                severity: Some(DiagnosticSeverity::WARNING),
                                message: format!("Unknown ideology: '{}'", val),
                                code: Some(NumberOrString::String(
                                    advanced_validation::UNKNOWN_TRIGGER.to_string(),
                                )),
                                source: Some("Hearts of Modding".to_string()),
                                ..Default::default()
                            });
                        }
                    }
                }

                // Trait checks
                if key_lower == "add_trait"
                    || key_lower == "has_trait"
                    || key_lower == "remove_trait"
                {
                    if let ast::Value::String(val) = &ass.value.value {
                        if !tr.contains_key(val) {
                            diagnostics.push(Diagnostic {
                                range: ast_range_to_lsp(&ass.value.range),
                                severity: Some(DiagnosticSeverity::WARNING),
                                message: format!("Unknown trait: '{}'", val),
                                code: Some(NumberOrString::String(
                                    advanced_validation::UNKNOWN_TRIGGER.to_string(),
                                )),
                                source: Some("Hearts of Modding".to_string()),
                                ..Default::default()
                            });
                        }
                    }
                }

                // Ability checks
                if key_lower == "has_ability"
                    || key_lower == "add_ability"
                    || key_lower == "remove_ability"
                {
                    if let ast::Value::String(val) = &ass.value.value {
                        let abilities = self.scanner_data.abilities();
                        if !abilities.contains_key(val) {
                            diagnostics.push(Diagnostic {
                                range: ast_range_to_lsp(&ass.value.range),
                                severity: Some(DiagnosticSeverity::WARNING),
                                message: format!("Unknown ability: '{}'", val),
                                code: Some(NumberOrString::String(
                                    advanced_validation::UNKNOWN_TRIGGER.to_string(),
                                )),
                                source: Some("Hearts of Modding".to_string()),
                                ..Default::default()
                            });
                        }
                    }
                }

                // Sprite/Gfx checks
                if key_lower == "sprite"
                    || key_lower == "icon"
                    || key_lower == "sprite_name"
                    || key_lower == "picture"
                {
                    if let ast::Value::String(val) = &ass.value.value {
                        let mut lookup_key = val.clone();
                        // Country idea "picture" field resolution: picture = [name] resolves to GFX_idea_[name]
                        // Only prepend if the value doesn't already carry the GFX_idea_ namespace;
                        // the engine checks for the full prefix, not just GFX_ — so GFX_skulk_economy
                        // would still get prepended to GFX_idea_GFX_skulk_economy and fail.
                        if key_lower == "picture"
                            && scope_stack.current() == scope::Scope::Idea
                            && !val.starts_with("GFX_idea_")
                        {
                            lookup_key = format!("GFX_idea_{}", val);
                        }

                        let exists = sp.contains_key(&lookup_key)
                            || (key_lower == "picture"
                                && scope_stack.current() == scope::Scope::Idea
                                && sp
                                    .keys()
                                    .any(|k| k.starts_with(&format!("{}_", lookup_key))));

                        if !exists
                            && (lookup_key.starts_with("GFX_")
                                || (key_lower == "picture"
                                    && scope_stack.current() == scope::Scope::Idea))
                        {
                            diagnostics.push(Diagnostic {
                                range: ast_range_to_lsp(&ass.value.range),
                                severity: Some(DiagnosticSeverity::WARNING),
                                message: format!(
                                    "Unknown sprite/GFX: '{}' (resolved from '{}')",
                                    lookup_key, val
                                ),
                                code: Some(NumberOrString::String(
                                    advanced_validation::UNKNOWN_TRIGGER.to_string(),
                                )),
                                source: Some("Hearts of Modding".to_string()),
                                ..Default::default()
                            });
                        }
                    }
                }

                // Sound effect checks
                if key_lower == "sound_effect" {
                    if let ast::Value::String(val) = &ass.value.value {
                        if !s_effects.contains_key(val) {
                            diagnostics.push(Diagnostic {
                                range: ast_range_to_lsp(&ass.value.range),
                                severity: Some(DiagnosticSeverity::WARNING),
                                message: format!("Unknown sound effect: '{}'", val),
                                code: Some(NumberOrString::String(
                                    advanced_validation::UNKNOWN_TRIGGER.to_string(),
                                )),
                                source: Some("Hearts of Modding".to_string()),
                                ..Default::default()
                            });
                        }
                    }
                }

                // Idea checks
                if key_lower == "add_ideas"
                    || key_lower == "has_idea"
                    || key_lower == "remove_ideas"
                {
                    if let ast::Value::String(val) = &ass.value.value {
                        if val != "all" && !ids.contains_key(val) {
                            diagnostics.push(Diagnostic {
                                range: ast_range_to_lsp(&ass.value.range),
                                severity: Some(DiagnosticSeverity::WARNING),
                                message: format!("Unknown idea: '{}'", val),
                                ..Default::default()
                            });
                        }
                    }
                }

                // Province checks
                if key_lower == "province"
                    || key_lower == "on_province"
                    || key_lower == "is_province_id"
                {
                    self.check_is_province(&ass.value, diagnostics, provs);
                }

                if key_lower == "victory_points" {
                    if let ast::Value::Block(entries) = &ass.value.value {
                        // Find the first value entry
                        for entry in entries {
                            if let ast::Entry::Value(val) = entry {
                                self.check_is_province(val, diagnostics, provs);
                                break;
                            }
                        }
                    }
                }

                // Country tag checks
                if (key_lower == "tag" && scope_stack.current() != scope::Scope::Idea)
                    || key_lower == "original_tag"
                    || key_lower == "original_tag_to_check"
                {
                    if let ast::Value::String(val) = &ass.value.value {
                        // Allow scope references (ROOT, FROM, PREV, etc.)
                        let is_scope_ref = matches!(
                            val.to_uppercase().as_str(),
                            "ROOT"
                                | "FROM"
                                | "PREV"
                                | "THIS"
                                | "PREVPREV"
                                | "PREVPREVPREV"
                                | "PREVPREVPREVPREV"
                                | "OWNER"
                                | "CONTROLLER"
                                | "CAPITAL"
                        );
                        let is_var_ref = val.starts_with("var:");
                        let b = val.as_bytes();
                        let looks_like_tag = val.len() == 3
                            && b[0].is_ascii_alphabetic()
                            && b[0].is_ascii_uppercase()
                            && b[1].is_ascii_alphanumeric()
                            && b[2].is_ascii_alphanumeric()
                            && !matches!(
                                val.as_str(),
                                "NOT" | "AND" | "TAG" | "OOB" | "LOG" | "NUM" | "RED"
                            );

                        if !is_scope_ref && !is_var_ref && looks_like_tag && !ct.contains_key(val) {
                            diagnostics.push(Diagnostic {
                                range: ast_range_to_lsp(&ass.value.range),
                                severity: Some(DiagnosticSeverity::WARNING),
                                message: format!("Unknown country tag: '{}'", val),
                                code: Some(NumberOrString::String(
                                    advanced_validation::UNKNOWN_TRIGGER.to_string(),
                                )),
                                source: Some("Hearts of Modding".to_string()),
                                ..Default::default()
                            });
                        }
                    }
                }

                // Check value recursively
                self.check_value_semantic(
                    &ass.value,
                    diagnostics,
                    loc,
                    st,
                    se,
                    id,
                    sid,
                    tr,
                    sp,
                    ids,
                    provs,
                    mod_maps,
                    ig_loc,
                    comments,
                    styling_enabled,
                    scope_stack,
                    s_effects,
                    ct,
                );

                if pushed_scope {
                    scope_stack.pop();
                }
            }
            ast::Entry::Value(val) => {
                self.check_value_semantic(
                    val,
                    diagnostics,
                    loc,
                    st,
                    se,
                    id,
                    sid,
                    tr,
                    sp,
                    ids,
                    provs,
                    mod_maps,
                    ig_loc,
                    comments,
                    styling_enabled,
                    scope_stack,
                    s_effects,
                    ct,
                );
            }
            _ => {}
        }
    }

    fn check_value_semantic(
        &self,
        val: &ast::NodeedValue,
        diagnostics: &mut Vec<Diagnostic>,
        loc: &HashMap<String, loc_parser::LocEntry>,
        st: &HashMap<String, scripted_scanner::ScriptedEntity>,
        se: &HashMap<String, scripted_scanner::ScriptedEntity>,
        id: &HashMap<String, ideology_scanner::Ideology>,
        sid: &HashMap<String, (String, ast::Range, String)>,
        tr: &HashMap<String, trait_scanner::Trait>,
        sp: &HashMap<String, sprite_scanner::Sprite>,
        ids: &HashMap<String, idea_scanner::Idea>,
        provs: &HashMap<u32, province_scanner::Province>,
        mod_maps: &HashMap<String, String>,
        ig_loc: &[regex::Regex],
        comments: &[(String, ast::Range)],
        styling_enabled: bool,
        scope_stack: &mut scope::ScopeStack,
        s_effects: &HashMap<String, sound_scanner::SoundEffect>,
        ct: &HashMap<String, country_scanner::CountryTag>,
    ) {
        match &val.value {
            ast::Value::Block(entries) => {
                self.check_duplicate_keys(entries, diagnostics, mod_maps);
                for entry in entries {
                    self.check_entry_semantic(
                        entry,
                        diagnostics,
                        loc,
                        st,
                        se,
                        id,
                        sid,
                        tr,
                        sp,
                        ids,
                        provs,
                        mod_maps,
                        ig_loc,
                        comments,
                        styling_enabled,
                        scope_stack,
                        s_effects,
                        ct,
                    );
                }
            }
            ast::Value::TaggedBlock(tag, entries, block_range) => {
                if styling_enabled {
                    if block_range.start_line > val.range.start_line {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: val.range.start_line,
                                    character: val.range.start_col + tag.len() as u32,
                                },
                                end: Position {
                                    line: block_range.start_line,
                                    character: block_range.start_col,
                                },
                            },
                            severity: Some(DiagnosticSeverity::INFORMATION),
                            code: Some(NumberOrString::String("styling_brace_newline".to_string())),
                            message: "Curly brace should not be on a new line.".to_string(),
                            source: Some("Hearts of Modding".to_string()),
                            ..Default::default()
                        });
                    } else {
                        let tag_end_col = val.range.start_col + tag.len() as u32;
                        if block_range.start_col != tag_end_col + 1 {
                            diagnostics.push(Diagnostic {
                                range: Range {
                                    start: Position {
                                        line: val.range.start_line,
                                        character: tag_end_col,
                                    },
                                    end: Position {
                                        line: block_range.start_line,
                                        character: block_range.start_col,
                                    },
                                },
                                severity: Some(DiagnosticSeverity::INFORMATION),
                                code: Some(NumberOrString::String(
                                    "styling_brace_newline".to_string(),
                                )),
                                message:
                                    "Exactly one space should separate the tag and the curly brace."
                                        .to_string(),
                                source: Some("Hearts of Modding".to_string()),
                                ..Default::default()
                            });
                        }
                    }
                }
                self.check_duplicate_keys(entries, diagnostics, mod_maps);
                for entry in entries {
                    self.check_entry_semantic(
                        entry,
                        diagnostics,
                        loc,
                        st,
                        se,
                        id,
                        sid,
                        tr,
                        sp,
                        ids,
                        provs,
                        mod_maps,
                        ig_loc,
                        comments,
                        styling_enabled,
                        scope_stack,
                        s_effects,
                        ct,
                    );
                }
            }
            _ => {}
        }
    }

    fn check_duplicate_keys(
        &self,
        entries: &[ast::Entry],
        diagnostics: &mut Vec<Diagnostic>,
        mod_maps: &HashMap<String, String>,
    ) {
        // Currently only checks keys that are in `mod_maps` (modifier names) plus a small
        // hardcoded set of common structural keys (`name`, `id`, `icon`). All other keys
        // (e.g. arbitrary custom keys from mods) are silently allowed. To extend coverage,
        // add more entries to `COMMON_KEYS` or replace the hardcoded list with a configurable
        // set of key patterns.
        const COMMON_KEYS: [&str; 3] = ["name", "id", "icon"];

        let mut seen_keys: HashMap<String, ast::Range> = HashMap::new();

        for entry in entries {
            if let ast::Entry::Assignment(ass) = entry {
                // We only care about duplicates if they are modifiers.
                // Some Paradox keys (like 'modifier = { ... }' or 'option = { ... }') are intended to be duplicates.
                // But specific engine modifiers (like 'stability_factor') should NEVER be duplicated.

                let is_modifier =
                    mod_maps.contains_key(&ass.key) || COMMON_KEYS.contains(&ass.key.as_str());

                // Exceptions: Some effects/triggers are specifically designed to be used multiple times
                let is_exception = ass.key == "modifier"
                    || ass.key == "option"
                    || ass.key == "limit"
                    || ass.key == "if"
                    || ass.key == "else"
                    || ass.key == "else_if"
                    || ass.key == "variable_name";

                if is_modifier && !is_exception {
                    if let Some(prev_range) = seen_keys.get(&ass.key) {
                        diagnostics.push(Diagnostic {
                            range: ast_range_to_lsp(prev_range),
                            severity: Some(DiagnosticSeverity::WARNING),
                            code: Some(NumberOrString::String("duplicate_key".to_string())),
                            message: format!("Duplicate modifier/key '{}' detected in the same scope. The game will ignore this value and use the last one.", ass.key),
                            source: Some("Hearts of Modding".to_string()),
                            ..Default::default()
                        });
                    }

                    let full_range = ast::Range {
                        start_line: ass.key_range.start_line,
                        start_col: ass.key_range.start_col,
                        end_line: ass.value.range.end_line,
                        end_col: ass.value.range.end_col,
                    };
                    seen_keys.insert(ass.key.clone(), full_range);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        documents: DashMap::new(),
        document_asts: DashMap::new(),
        scanner_data: ScannerData::new(),
        config: Config::new(),
        system_info: Mutex::new(sysinfo::System::new()),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
pub mod test_loc_dups;
#[cfg(test)]
pub mod test_loc_empty;
#[cfg(test)]
pub mod test_parser_skip;
