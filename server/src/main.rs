#![allow(clippy::collapsible_if)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
mod ability_scanner;
mod achievement_scanner;
mod adjacency_scanner;
mod advanced_validation;
mod ast;
mod building_scanner;
mod call_hierarchy;
mod character_scanner;
mod csv_parser;
mod defines_parser;
mod document_symbols;
mod enhanced_color;
mod event_scanner;
mod hoi4_data;
mod idea_scanner;
mod ideology_scanner;
mod loc_parser;
mod logistics_scanner;
mod map_config;
mod map_object_scanner;
mod modifier_display;
mod modifier_scanner;
mod music_scanner;
mod parser;
mod province_scanner;
mod rename;
mod scope;
mod scripted_loc_scanner;
mod scripted_scanner;
mod semantic_tokens;
mod sound_scanner;
mod sprite_scanner;
mod state_scanner;
mod strategic_region_scanner;
#[cfg(test)]
mod test_loc_version;
mod trait_scanner;
mod variable_scanner;
mod workspace_symbols;
mod color_utils;
mod loc_preview;
mod lsp_convert;
mod modifier_format;
mod scope_context;
mod symbol_search;
mod scan_orchestrator;
mod formatting;
mod hover_handler;
mod completion_handler;
mod code_action_handler;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::color_utils::find_colors;
use crate::loc_preview::find_identifier_in_loc;
use crate::lsp_convert::{
    ast_range_to_lsp, ast_range_to_lsp_location, ast_related_info_to_lsp, ast_tag_to_lsp,
};
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
    s.chars()
        .take(byte_offset)
        .map(|c| c.len_utf16())
        .sum::<usize>() as u32
}

/// Get the UTF-16 length of a string
pub(crate) fn utf16_len(s: &str) -> u32 {
    s.chars().map(|c| c.len_utf16()).sum::<usize>() as u32
}

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: DashMap<String, String>,
    localization: Arc<arc_swap::ArcSwap<HashMap<String, loc_parser::LocEntry>>>,
    scripted_triggers: Arc<arc_swap::ArcSwap<HashMap<String, scripted_scanner::ScriptedEntity>>>,
    scripted_effects: Arc<arc_swap::ArcSwap<HashMap<String, scripted_scanner::ScriptedEntity>>>,
    ideologies: Arc<arc_swap::ArcSwap<HashMap<String, ideology_scanner::Ideology>>>,
    sub_ideologies: Arc<arc_swap::ArcSwap<HashMap<String, (String, ast::Range, String)>>>, // Sub-ideology -> (Parent Ideology, Range, Path)
    traits: Arc<arc_swap::ArcSwap<HashMap<String, trait_scanner::Trait>>>,
    sprites: Arc<arc_swap::ArcSwap<HashMap<String, sprite_scanner::Sprite>>>,
    ideas: Arc<arc_swap::ArcSwap<HashMap<String, idea_scanner::Idea>>>,
    characters: Arc<arc_swap::ArcSwap<HashMap<String, character_scanner::Character>>>,
    variables: Arc<arc_swap::ArcSwap<HashMap<String, Vec<variable_scanner::Variable>>>>,
    event_targets: Arc<arc_swap::ArcSwap<HashMap<String, Vec<variable_scanner::EventTarget>>>>,
    provinces: Arc<arc_swap::ArcSwap<HashMap<u32, province_scanner::Province>>>,
    custom_modifiers: Arc<arc_swap::ArcSwap<HashMap<String, modifier_scanner::Modifier>>>,
    modifier_mappings: Arc<arc_swap::ArcSwap<HashMap<String, String>>>,
    modifier_formats: Arc<arc_swap::ArcSwap<HashMap<String, String>>>,
    events: Arc<arc_swap::ArcSwap<HashMap<String, event_scanner::Event>>>,
    music_assets: Arc<arc_swap::ArcSwap<HashMap<String, music_scanner::MusicAsset>>>,
    music_stations: Arc<arc_swap::ArcSwap<HashMap<String, music_scanner::MusicStation>>>,
    songs: Arc<arc_swap::ArcSwap<HashMap<String, music_scanner::Song>>>,
    sounds: Arc<arc_swap::ArcSwap<HashMap<String, sound_scanner::Sound>>>,
    sound_effects: Arc<arc_swap::ArcSwap<HashMap<String, sound_scanner::SoundEffect>>>,
    falloffs: Arc<arc_swap::ArcSwap<HashMap<String, sound_scanner::Falloff>>>,
    sound_categories: Arc<arc_swap::ArcSwap<HashMap<String, sound_scanner::SoundCategory>>>,
    buildings: Arc<arc_swap::ArcSwap<HashMap<String, building_scanner::Building>>>,
    achievements: Arc<arc_swap::ArcSwap<HashMap<String, achievement_scanner::Achievement>>>,
    defines: Arc<arc_swap::ArcSwap<defines_parser::GameDefines>>,
    ignored_loc_regex: Arc<arc_swap::ArcSwap<Vec<regex::Regex>>>,
    ignored_files_regex: Arc<arc_swap::ArcSwap<Vec<regex::Regex>>>,
    workspace_scan_enabled: Arc<arc_swap::ArcSwap<bool>>,
    styling_enabled: Arc<arc_swap::ArcSwap<bool>>,
    cosmetic_loc_indent: Arc<arc_swap::ArcSwap<bool>>,
    game_path: Arc<arc_swap::ArcSwap<Option<String>>>,
    abilities: Arc<arc_swap::ArcSwap<HashMap<String, ability_scanner::Ability>>>,
    scripted_locs: Arc<arc_swap::ArcSwap<HashMap<String, scripted_loc_scanner::ScriptedLoc>>>,
    duplicated_loc_keys: Arc<arc_swap::ArcSwap<HashSet<(String, String)>>>,
    states: Arc<arc_swap::ArcSwap<HashMap<u32, state_scanner::State>>>,
    supply_nodes: Arc<arc_swap::ArcSwap<Vec<logistics_scanner::SupplyNode>>>,
    railways: Arc<arc_swap::ArcSwap<Vec<logistics_scanner::Railway>>>,
    map_buildings: Arc<arc_swap::ArcSwap<Vec<map_object_scanner::MapBuilding>>>,
    unitstacks: Arc<arc_swap::ArcSwap<Vec<map_object_scanner::UnitStack>>>,
    weather_positions: Arc<arc_swap::ArcSwap<Vec<map_object_scanner::WeatherPosition>>>,
    adjacencies: Arc<arc_swap::ArcSwap<Vec<adjacency_scanner::Adjacency>>>,
    adjacency_rules: Arc<arc_swap::ArcSwap<HashMap<String, adjacency_scanner::AdjacencyRule>>>,
    strategic_regions:
        Arc<arc_swap::ArcSwap<HashMap<u32, strategic_region_scanner::StrategicRegion>>>,
    workspace_files: Arc<arc_swap::ArcSwap<HashSet<String>>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        if let Some(options) = params.initialization_options {
            if let Some(path) = options.get("gamePath").and_then(|v| v.as_str()) {
                if !path.is_empty() {
                    self.game_path
                        .store(std::sync::Arc::new(Some(path.to_string())));
                    let _gp = self.game_path.load();
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
                self.ignored_loc_regex.store(std::sync::Arc::new(patterns));
                let _ig = self.ignored_loc_regex.load();
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
                self.ignored_files_regex
                    .store(std::sync::Arc::new(patterns));
                let _ig = self.ignored_files_regex.load();
            }
            if let Some(enabled) = options
                .get("workspaceScanEnabled")
                .and_then(|v| v.as_bool())
            {
                self.workspace_scan_enabled
                    .store(std::sync::Arc::new(enabled));
                let _ws = self.workspace_scan_enabled.load();
            }
            if let Some(enabled) = options.get("stylingEnabled").and_then(|v| v.as_bool()) {
                self.styling_enabled.store(std::sync::Arc::new(enabled));
                let _st = self.styling_enabled.load();
            }
            if let Some(enabled) = options.get("cosmeticLocIndent").and_then(|v| v.as_bool()) {
                self.cosmetic_loc_indent.store(std::sync::Arc::new(enabled));
                let _ci = self.cosmetic_loc_indent.load();
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
                    resolve_provider: Some(true),
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
                        "hoi4.getEventGraph".to_string(),
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
        let gp = self.game_path.load();
        if let Some(ref path) = **gp {
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
        );

        // Collect workspace file paths for rename operations
        // Only scan the mod workspace (first root), not the game path
        self.collect_workspace_files(&roots[..1]).await;

        // Re-validate all open documents now that we have all data
        for entry in self.documents.iter() {
            if let Ok(uri) = Url::parse(entry.key()) {
                self.validate_document(uri).await;
            }
        }

        // Workspace-wide scan
        if **self.workspace_scan_enabled.load() {
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
                        self.ignored_loc_regex.store(std::sync::Arc::new(patterns));
                        let _ig = self.ignored_loc_regex.load();
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
                        self.ignored_files_regex
                            .store(std::sync::Arc::new(patterns));
                        let _ig = self.ignored_files_regex.load();
                    }
                    if let Some(enabled) = validator
                        .get("workspaceScan")
                        .and_then(|v| v.as_object())
                        .and_then(|v| v.get("enabled"))
                        .and_then(|v| v.as_bool())
                    {
                        self.workspace_scan_enabled
                            .store(std::sync::Arc::new(enabled));
                        let _ws = self.workspace_scan_enabled.load();
                    }
                }
                if let Some(styling) = hoi4.get("styling").and_then(|v| v.as_object()) {
                    if let Some(enabled) = styling.get("enabled").and_then(|v| v.as_bool()) {
                        self.styling_enabled.store(std::sync::Arc::new(enabled));
                        let _st = self.styling_enabled.load();
                    }
                    if let Some(enabled) = styling
                        .get("cosmeticLocalizationIndentation")
                        .and_then(|v| v.as_bool())
                    {
                        self.cosmetic_loc_indent.store(std::sync::Arc::new(enabled));
                        let _ci = self.cosmetic_loc_indent.load();
                    }
                }
                // Re-validate all documents
                for entry in self.documents.iter() {
                    if let Ok(uri) = Url::parse(entry.key()) {
                        self.validate_document(uri).await;
                    }
                }
            }
        }
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.documents.insert(
            params.text_document.uri.to_string(),
            params.text_document.text,
        );
        self.validate_document(params.text_document.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.documents.insert(
            params.text_document.uri.to_string(),
            params.content_changes[0].text.clone(),
        );
        self.validate_document(params.text_document.uri).await;
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

        match self.documents.get(&uri) {
            Some(content) => {
                let (script, _) = parser::parse_script(&content);
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
                    keywords.insert(k.to_lowercase());
                }

                // Add hardcoded achievement keywords
                keywords.insert("unique_id".to_string());
                keywords.insert("possible".to_string());
                keywords.insert("happened".to_string());
                keywords.insert("ribbon".to_string());
                keywords.insert("frames".to_string());
                keywords.insert("colors".to_string());

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

                let ability_names: HashSet<String> = self
                    .abilities
                    .load()
                    .keys()
                    .map(|k| k.to_string())
                    .collect();

                Ok(Some(semantic_tokens::get_semantic_tokens(
                    &script, &keywords, &ability_names,
                )))
            }
            _ => Ok(None),
        }
    }

    async fn document_color(&self, params: DocumentColorParams) -> Result<Vec<ColorInformation>> {
        let uri = params.text_document.uri.to_string();
        if let Some(content) = self.documents.get(&uri) {
            {
                let (script, _) = parser::parse_script(&content);
                return Ok(find_colors(&script));
            }
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
        let defines = self.defines.load();
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
                let cosmetic_indent = **self.cosmetic_loc_indent.load();
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

    async fn completion_resolve(&self, params: CompletionItem) -> Result<CompletionItem> {
        Ok(params)
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
                let (script, _) = parser::parse_script(&content);
                let mut scope_stack = scope::ScopeStack::new(scope::Scope::Global);
                let achievements = self.achievements.load();
                find_identifier_at(&script, position, &mut scope_stack, &achievements)
                    .map(|(id, _, _, _)| id)
            };

            if let Some(identifier) = identifier {
                let mut sources = Vec::new();
                let mut localizations = Vec::new();

                // Check scripted elements
                let st = self.scripted_triggers.load();
                if let Some(entity) = st.get(&identifier) {
                    sources.push(ast_range_to_lsp_location(&entity.range, &entity.path));
                }

                let se = self.scripted_effects.load();
                if let Some(entity) = se.get(&identifier) {
                    sources.push(ast_range_to_lsp_location(&entity.range, &entity.path));
                }

                let sl = self.scripted_locs.load();
                if let Some(loc) = sl.get(&identifier) {
                    sources.push(ast_range_to_lsp_location(&loc.range, &loc.path));
                }

                // Check ideologies
                let id_map = self.ideologies.load();
                if let Some(ideology) = id_map.get(&identifier) {
                    sources.push(ast_range_to_lsp_location(&ideology.range, &ideology.path));
                }

                let sid_map = self.sub_ideologies.load();
                if let Some((_, range, path)) = sid_map.get(&identifier) {
                    sources.push(ast_range_to_lsp_location(range, path));
                }

                // Check traits
                let t_map = self.traits.load();
                if let Some(trait_info) = t_map.get(&identifier) {
                    sources.push(ast_range_to_lsp_location(
                        &trait_info.range,
                        &trait_info.path,
                    ));
                }

                // Check sprites
                let s_map = self.sprites.load();
                if let Some(sprite) = s_map.get(&identifier) {
                    sources.push(ast_range_to_lsp_location(&sprite.range, &sprite.path));
                }

                // Check events
                let e_map = self.events.load();
                if let Some(event) = e_map.get(&identifier) {
                    sources.push(ast_range_to_lsp_location(&event.range, &event.path));
                }

                // Check abilities
                let ability_map = self.abilities.load();
                if let Some(ability) = ability_map.get(&identifier) {
                    sources.push(ast_range_to_lsp_location(&ability.range, &ability.path));
                }

                // Check ideas
                let idea_map = self.ideas.load();
                if let Some(idea) = idea_map.get(&identifier) {
                    sources.push(ast_range_to_lsp_location(&idea.range, &idea.path));
                }

                // Check achievements
                let a_map = self.achievements.load();
                if let Some(achievement) = a_map.get(&identifier) {
                    sources.push(ast_range_to_lsp_location(
                        &achievement.range,
                        &achievement.path,
                    ));
                }

                // Check variables
                let var_map = self.variables.load();
                if let Some(vars) = var_map.get(&identifier) {
                    for var in vars {
                        sources.push(ast_range_to_lsp_location(&var.range, &var.path));
                    }
                }

                // Check event targets
                let target_map = self.event_targets.load();
                if let Some(targets) = target_map.get(&identifier) {
                    for target in targets {
                        sources.push(ast_range_to_lsp_location(&target.range, &target.path));
                    }
                }

                // Check modifiers
                let custom_mods = self.custom_modifiers.load();
                if let Some(modifier) = custom_mods.get(&identifier) {
                    sources.push(ast_range_to_lsp_location(&modifier.range, &modifier.path));
                }

                // Check music
                let m_assets = self.music_assets.load();
                if let Some(asset) = m_assets.get(&identifier) {
                    sources.push(ast_range_to_lsp_location(&asset.range, &asset.path));
                }

                let m_stations = self.music_stations.load();
                if let Some(station) = m_stations.get(&identifier) {
                    sources.push(ast_range_to_lsp_location(&station.range, &station.path));
                }

                let m_songs = self.songs.load();
                if let Some(song) = m_songs.get(&identifier) {
                    sources.push(ast_range_to_lsp_location(&song.range, &song.path));
                }

                // Check sounds
                let s_sounds = self.sounds.load();
                if let Some(sound) = s_sounds.get(&identifier) {
                    sources.push(ast_range_to_lsp_location(&sound.range, &sound.path));
                }

                let s_effects = self.sound_effects.load();
                if let Some(effect) = s_effects.get(&identifier) {
                    sources.push(ast_range_to_lsp_location(&effect.range, &effect.path));
                }

                let s_falloffs = self.falloffs.load();
                if let Some(falloff) = s_falloffs.get(&identifier) {
                    sources.push(ast_range_to_lsp_location(&falloff.range, &falloff.path));
                }

                let s_categories = self.sound_categories.load();
                if let Some(category) = s_categories.get(&identifier) {
                    sources.push(ast_range_to_lsp_location(&category.range, &category.path));
                }

                // Check adjacency rules
                let rule_lock = self.adjacency_rules.load();
                if let Some(rule) = rule_lock.get(&identifier) {
                    sources.push(ast_range_to_lsp_location(&rule.range, &rule.path));
                }

                // Check strategic regions
                let regions = self.strategic_regions.load();
                if let Ok(id) = identifier.parse::<u32>() {
                    if let Some(region) = regions.get(&id) {
                        sources.push(ast_range_to_lsp_location(&region.range, &region.path));
                    }
                }

                let mappings = self.modifier_mappings.load();
                if let Some(loc_key) = mappings.get(&identifier) {
                    let loc = self.localization.load();
                    if let Some(e) = loc.get(loc_key) {
                        localizations.push(ast_range_to_lsp_location(&e.range, &e.path));
                    }
                }

                // Check localization
                let loc = self.localization.load();
                // Try exact match
                if let Some(e) = loc.get(&identifier) {
                    localizations.push(ast_range_to_lsp_location(&e.range, &e.path));
                }
                // Try key:0 etc
                let target = format!("{}:", identifier);
                for (k, e) in loc.iter() {
                    if k.starts_with(&target) {
                        localizations.push(ast_range_to_lsp_location(&e.range, &e.path));
                    }
                }

                // Prefer sources over localizations
                let mut all_locations = sources;
                all_locations.extend(localizations);

                if !all_locations.is_empty() {
                    return Ok(Some(GotoDefinitionResponse::Array(all_locations)));
                }
            }
        }
        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;

        if let Some(content) = self.documents.get(&uri) {
            {
                let (script, _) = parser::parse_script(&content);
                let mut scope_stack = scope::ScopeStack::new(scope::Scope::Global);
                let achievements = self.achievements.load();
                if let Some((identifier, _, _, _)) =
                    find_identifier_at(&script, position, &mut scope_stack, &achievements)
                {
                    let mut locations = Vec::new();

                    // Search in all roots
                    let mut roots = vec![std::path::PathBuf::from(".")];
                    let gp = self.game_path.load();
                    if let Some(ref path) = **gp {
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
        }
        Ok(None)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        self.handle_code_action(params).await
    }

    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> Result<Option<serde_json::Value>> {
        if params.command == "hoi4.getEventGraph" {
            let events = self.events.load();
            let json = serde_json::to_value(&**events).unwrap();
            return Ok(Some(json));
        } else if params.command == "hoi4/getMemoryUsage" {
            let mut sys = sysinfo::System::new();
            sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
            if let Ok(pid) = sysinfo::get_current_pid() {
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

        match self.documents.get(uri) {
            Some(entry) => {
                let content = entry.value();

                // Parse the document
                let (script, _) = parser::parse_script(content);
                let symbols = document_symbols::generate_document_symbols(&script.entries);
                Ok(Some(DocumentSymbolResponse::Nested(symbols)))
            }
            _ => Ok(None),
        }
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let symbols = workspace_symbols::generate_workspace_symbols(
            &params.query,
            &self.events,
            &self.ideas,
            &self.traits,
            &self.scripted_triggers,
            &self.scripted_effects,
            &self.ideologies,
            &self.sub_ideologies,
            &self.sprites,
            &self.characters,
            &self.variables,
            &self.achievements,
            &self.abilities,
            &self.scripted_locs,
            &self.localization,
            &self.states,
            &self.supply_nodes,
            &self.railways,
            &self.map_buildings,
            &self.unitstacks,
            &self.weather_positions,
            &self.adjacencies,
            &self.adjacency_rules,
            &self.strategic_regions,
            &self.custom_modifiers,
            &self.sounds,
            &self.sound_effects,
            &self.falloffs,
            &self.sound_categories,
        )
        .await;

        Ok(Some(symbols))
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

        let item = call_hierarchy::prepare_call_hierarchy(
            uri,
            position,
            &self.events,
            &self.scripted_triggers,
            &self.scripted_effects,
        )
        .await;

        Ok(item.map(|i| vec![i]))
    }

    async fn incoming_calls(
        &self,
        params: CallHierarchyIncomingCallsParams,
    ) -> Result<Option<Vec<CallHierarchyIncomingCall>>> {
        let calls = call_hierarchy::get_incoming_calls(
            &params.item,
            &self.events,
            &self.scripted_triggers,
            &self.scripted_effects,
            &self.documents,
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
            &self.events,
            &self.scripted_triggers,
            &self.scripted_effects,
            &self.documents,
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

        let result = rename::prepare_rename(
            uri,
            position,
            &self.events,
            &self.scripted_triggers,
            &self.scripted_effects,
            &self.ideas,
            &self.characters,
            &self.variables,
            &self.abilities,
        )
        .await;

        Ok(result)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri.as_str();
        let position = params.text_document_position.position;
        let new_name = &params.new_name;

        let files = self.workspace_files.load();
        let result = rename::rename_symbol(
            uri,
            position,
            new_name,
            &self.events,
            &self.scripted_triggers,
            &self.scripted_effects,
            &self.ideas,
            &self.characters,
            &self.variables,
            &self.abilities,
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
        let mut dirs_to_check = vec![root.to_path_buf()];
        let extensions = ["txt", "yml", "gfx", "gui", "asset"];

        while let Some(current_dir) = dirs_to_check.pop() {
            if self.should_ignore_file(&current_dir).await {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(current_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        dirs_to_check.push(path);
                    } else if path.extension().is_some_and(|ext| {
                        extensions.contains(&ext.to_string_lossy().as_ref())
                    }) {
                        if self.should_ignore_file(&path).await {
                            continue;
                        }
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if content.contains(identifier) {
                                // Find all occurrences with word boundaries
                                for (line_idx, line) in content.lines().enumerate() {
                                    let mut start_search = 0;
                                    while let Some(pos) = line[start_search..].find(identifier) {
                                        let actual_pos = start_search + pos;

                                        // Check word boundaries
                                        let before = if actual_pos > 0 {
                                            line.chars().nth(actual_pos - 1)
                                        } else {
                                            None
                                        };
                                        let after = line.chars().nth(actual_pos + identifier.len());

                                        let is_word_start =
                                            before.is_none_or(|c| !parser::is_identifier_char(c));
                                        let is_word_end =
                                            after.is_none_or(|c| !parser::is_identifier_char(c));

                                        if is_word_start && is_word_end {
                                            locations.push(Location {
                                                uri: Url::from_file_path(
                                                    path.canonicalize()
                                                        .unwrap_or_else(|_| path.clone()),
                                                )
                                                .unwrap(),
                                                range: Range {
                                                    start: Position {
                                                        line: line_idx as u32,
                                                        character: actual_pos as u32,
                                                    },
                                                    end: Position {
                                                        line: line_idx as u32,
                                                        character: (actual_pos + identifier.len())
                                                            as u32,
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
            }
        }
    }

    pub(crate) async fn should_ignore_file(&self, path: &std::path::Path) -> bool {
        let path_str = path.to_string_lossy();
        let ignored = self.ignored_files_regex.load();
        for re in ignored.iter() {
            if re.is_match(&path_str) {
                return true;
            }
        }
        false
    }

    pub(crate) fn get_sync_filter(&self) -> impl Fn(&std::path::Path) -> bool + Send + Sync + 'static {
        let regexes = self.ignored_files_regex.clone();
        move |path: &std::path::Path| {
            let path_str = path.to_string_lossy();
            let ignored = regexes.load();
            {
                for re in ignored.iter() {
                    if re.is_match(&path_str) {
                        return true;
                    }
                }
            }
            false
        }
    }

    async fn validate_workspace(&self, root: &std::path::Path) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("Starting workspace diagnostic scan in: {:?}", root),
            )
            .await;

        let mut dirs_to_check = vec![root.to_path_buf()];
        let extensions = ["txt", "yml", "csv"];
        let mut file_count = 0;

        while let Some(current_dir) = dirs_to_check.pop() {
            if self.should_ignore_file(&current_dir).await {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(current_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        // Skip .git and potentially other internal dirs if needed,
                        // but for HOI4 mods usually everything in subdirs is relevant
                        if path.file_name().is_none_or(|n| n != ".git") {
                            dirs_to_check.push(path);
                        }
                    } else if path.extension().is_some_and(|ext| {
                        extensions.contains(&ext.to_string_lossy().as_ref())
                    }) {
                        if self.should_ignore_file(&path).await {
                            continue;
                        }
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(abs_path) = path.canonicalize() {
                                if let Ok(uri) = Url::from_file_path(abs_path) {
                                    // Only validate if not already open in editor (which would have its own sync)
                                    // actually validate_content is idempotent for our needs here
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

    /// Collect all workspace file paths for rename operations
    async fn collect_workspace_files(&self, roots: &[std::path::PathBuf]) {
        let mut all_files = HashSet::new();
        let extensions = ["txt", "yml"];
        let ignored = self.ignored_files_regex.load();

        for root in roots {
            let mut dirs_to_check = vec![root.to_path_buf()];
            while let Some(current_dir) = dirs_to_check.pop() {
                let path_str = current_dir.to_string_lossy();
                if ignored.iter().any(|re| re.is_match(&path_str)) {
                    continue;
                }
                if let Ok(entries) = std::fs::read_dir(&current_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            if path.file_name().is_none_or(|n| n != ".git") {
                                let path_str = path.to_string_lossy();
                                if !ignored.iter().any(|re| re.is_match(&path_str)) {
                                    dirs_to_check.push(path);
                                }
                            }
                        } else if path.extension().is_some_and(|ext| {
                            extensions.contains(&ext.to_string_lossy().as_ref())
                        }) {
                            if let Ok(abs_path) = path.canonicalize() {
                                all_files.insert(abs_path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }

        self.workspace_files.store(std::sync::Arc::new(all_files));
    }

    async fn validate_document(&self, uri: Url) {
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

    async fn validate_content(&self, uri: &Url, content: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let styling_enabled = **self.styling_enabled.load();
        let mut script_opt = None;
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
        } else if uri.as_str().contains("strategicregions") && uri.as_str().ends_with(".txt") {
            let (script, parse_errors) = parser::parse_script(content);
            for (msg, range) in parse_errors {
                diagnostics.push(Diagnostic {
                    range: ast_range_to_lsp(&range),
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: msg,
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
        } else if uri.as_str().ends_with(".csv") {
            // Do not parse other CSV files as clausewitz scripts
        } else {
            let (script, parse_errors) = parser::parse_script(content);
            for (msg, range) in parse_errors {
                diagnostics.push(Diagnostic {
                    range: ast_range_to_lsp(&range),
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: msg,
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

        if styling_enabled {
            let is_yaml = uri.as_str().ends_with(".yml");
            self.check_styling(
                content,
                script_opt.as_ref(),
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
        let provs = self.provinces.load();
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
        let provs = self.provinces.load();
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
        let states = self.states.load();
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
        let regions = self.strategic_regions.load();
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
        let provs = self.provinces.load();
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
        let provs = self.provinces.load();
        let rules = self.adjacency_rules.load();
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

                    if parts[2].to_lowercase() == "sea" && id <= 0 {
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
        let provs = self.provinces.load();
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
                if ass.key.to_lowercase() == "adjacency_rule" {
                    if let ast::Value::Block(rule_entries) = &ass.value.value {
                        for rule_entry in rule_entries {
                            if let ast::Entry::Assignment(r_ass) = rule_entry {
                                if r_ass.key.to_lowercase() == "required_provinces" {
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
        let provs = self.provinces.load();

        for entry in &script.entries {
            if let ast::Entry::Assignment(ass) = entry {
                if ass.key.to_lowercase() == "strategic_region" {
                    if let ast::Value::Block(region_entries) = &ass.value.value {
                        for region_entry in region_entries {
                            if let ast::Entry::Assignment(r_ass) = region_entry {
                                if r_ass.key.to_lowercase() == "provinces" {
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
        uri: &Url,
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
        let event_targets = self.event_targets.load();
        let scripted_locs = self.scripted_locs.load();
        let dups = self.duplicated_loc_keys.load();

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

            let loc_diagnostics =
                loc_parser::validate_loc_string(entry, &event_targets, &scripted_locs);
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
                let loc_map = self.localization.load();
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
            ast::Value::Block(_) | ast::Value::TaggedBlock(_, _, _) if range.start_line == range.end_line => {
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
        let loc = self.localization.load();
        let st = self.scripted_triggers.load();
        let se = self.scripted_effects.load();
        let id = self.ideologies.load();
        let sid = self.sub_ideologies.load();
        let tr = self.traits.load();
        let sp = self.sprites.load();
        let ids = self.ideas.load();
        let provs = self.provinces.load();
        let mod_maps = self.modifier_mappings.load();
        let ig_loc = self.ignored_loc_regex.load();
        let buildings = self.buildings.load();
        let defines = self.defines.load();
        let s_effects = self.sound_effects.load();

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

        // Run advanced validations
        let mut advanced_diags = Vec::new();
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
            );
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
    ) {
        match entry {
            ast::Entry::Assignment(ass) => {
                let key_lower = ass.key.to_lowercase();
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
                                        a.key.to_lowercase() == "picture"
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
                        ast::Value::Block(_) if ass.value.range.start_line > ass.operator_range.end_line => {
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
                        if key_lower == kw.to_lowercase() && ass.key != kw {
                            let mut message = format!(
                                "Standard Paradox Script convention uses '{}' instead of '{}'.",
                                kw, ass.key
                            );
                            if kw.to_lowercase().contains("sprite") || kw == "texturefile" {
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
                                    if comment_text.to_lowercase().contains("ignore") {
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
                        let abilities = self.abilities.load();
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
                        if key_lower == "picture" && scope_stack.current() == scope::Scope::Idea {
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
        let mut seen_keys: HashMap<String, ast::Range> = HashMap::new();

        for entry in entries {
            if let ast::Entry::Assignment(ass) = entry {
                // We only care about duplicates if they are modifiers.
                // Some Paradox keys (like 'modifier = { ... }' or 'option = { ... }') are intended to be duplicates.
                // But specific engine modifiers (like 'stability_factor') should NEVER be duplicated.

                let is_modifier = mod_maps.contains_key(&ass.key);

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
        localization: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        scripted_triggers: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        scripted_effects: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        ideologies: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        sub_ideologies: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        traits: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        sprites: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        ideas: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        characters: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        variables: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        event_targets: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        provinces: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        custom_modifiers: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        modifier_mappings: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        modifier_formats: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        events: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        music_assets: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        music_stations: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        songs: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        sounds: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        sound_effects: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        falloffs: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        sound_categories: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        buildings: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        achievements: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        defines: Arc::new(arc_swap::ArcSwap::from_pointee(
            defines_parser::GameDefines::new(),
        )),
        ignored_loc_regex: Arc::new(arc_swap::ArcSwap::from_pointee(Vec::new())),
        ignored_files_regex: Arc::new(arc_swap::ArcSwap::from_pointee(Vec::new())),
        workspace_scan_enabled: Arc::new(arc_swap::ArcSwap::from_pointee(false)),
        styling_enabled: Arc::new(arc_swap::ArcSwap::from_pointee(true)),
        cosmetic_loc_indent: Arc::new(arc_swap::ArcSwap::from_pointee(false)),
        game_path: Arc::new(arc_swap::ArcSwap::from_pointee(None)),
        abilities: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        scripted_locs: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        duplicated_loc_keys: Arc::new(arc_swap::ArcSwap::from_pointee(HashSet::new())),
        states: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        supply_nodes: Arc::new(arc_swap::ArcSwap::from_pointee(Vec::new())),
        railways: Arc::new(arc_swap::ArcSwap::from_pointee(Vec::new())),
        map_buildings: Arc::new(arc_swap::ArcSwap::from_pointee(Vec::new())),
        unitstacks: Arc::new(arc_swap::ArcSwap::from_pointee(Vec::new())),
        weather_positions: Arc::new(arc_swap::ArcSwap::from_pointee(Vec::new())),
        adjacencies: Arc::new(arc_swap::ArcSwap::from_pointee(Vec::new())),
        adjacency_rules: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        strategic_regions: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        workspace_files: Arc::new(arc_swap::ArcSwap::from_pointee(HashSet::new())),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}

pub mod test_loc_dups;
pub mod test_loc_empty;
pub mod test_parser_skip;
