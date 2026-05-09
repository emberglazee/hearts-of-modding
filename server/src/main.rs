mod ast;
mod parser;
mod semantic_tokens;
mod hoi4_data;
mod loc_parser;
mod scripted_scanner;
mod scope;
mod ideology_scanner;
mod trait_scanner;
mod sprite_scanner;
mod idea_scanner;
mod variable_scanner;
mod province_scanner;
mod modifier_scanner;
mod modifier_display;
mod event_scanner;
mod music_scanner;
mod sound_scanner;
mod schema;
mod building_scanner;
mod defines_parser;
mod advanced_validation;
mod enhanced_color;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

static TRIGGERS: Lazy<HashMap<&'static str, hoi4_data::HOI4Entity>> = Lazy::new(hoi4_data::get_triggers);
static EFFECTS: Lazy<HashMap<&'static str, hoi4_data::HOI4Entity>> = Lazy::new(hoi4_data::get_effects);
static SCOPES: Lazy<Vec<&'static str>> = Lazy::new(hoi4_data::get_scopes);
static LOC_COMMANDS: Lazy<Vec<&'static str>> = Lazy::new(hoi4_data::get_loc_commands);

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: DashMap<String, String>,
    localization: Arc<RwLock<HashMap<String, loc_parser::LocEntry>>>,
    scripted_triggers: Arc<RwLock<HashMap<String, scripted_scanner::ScriptedEntity>>>,
    scripted_effects: Arc<RwLock<HashMap<String, scripted_scanner::ScriptedEntity>>>,
    ideologies: Arc<RwLock<HashMap<String, ideology_scanner::Ideology>>>,
    sub_ideologies: Arc<RwLock<HashMap<String, (String, ast::Range, String)>>>, // Sub-ideology -> (Parent Ideology, Range, Path)
    traits: Arc<RwLock<HashMap<String, trait_scanner::Trait>>>,
    sprites: Arc<RwLock<HashMap<String, sprite_scanner::Sprite>>>,
    ideas: Arc<RwLock<HashMap<String, idea_scanner::Idea>>>,
    variables: Arc<RwLock<HashMap<String, Vec<variable_scanner::Variable>>>>,
    event_targets: Arc<RwLock<HashMap<String, Vec<variable_scanner::EventTarget>>>>,
    provinces: Arc<RwLock<HashSet<u32>>>,
    custom_modifiers: Arc<RwLock<HashMap<String, modifier_scanner::Modifier>>>,
    modifier_mappings: Arc<RwLock<HashMap<String, String>>>,
    modifier_formats: Arc<RwLock<HashMap<String, String>>>,
    events: Arc<RwLock<HashMap<String, event_scanner::Event>>>,
    music_assets: Arc<RwLock<HashMap<String, music_scanner::MusicAsset>>>,
    music_stations: Arc<RwLock<HashMap<String, music_scanner::MusicStation>>>,
    songs: Arc<RwLock<HashMap<String, music_scanner::Song>>>,
    sounds: Arc<RwLock<HashMap<String, sound_scanner::Sound>>>,
    sound_effects: Arc<RwLock<HashMap<String, sound_scanner::SoundEffect>>>,
    falloffs: Arc<RwLock<HashMap<String, sound_scanner::Falloff>>>,
    sound_categories: Arc<RwLock<HashMap<String, sound_scanner::SoundCategory>>>,
    buildings: Arc<RwLock<HashMap<String, building_scanner::Building>>>,
    defines: Arc<RwLock<defines_parser::GameDefines>>,
    ignored_loc_regex: Arc<RwLock<Vec<regex::Regex>>>,
    schema: Arc<RwLock<schema::Schema>>,
    styling_enabled: Arc<RwLock<bool>>,
    cosmetic_loc_indent: Arc<RwLock<bool>>,
    game_path: Arc<RwLock<Option<String>>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        if let Some(options) = params.initialization_options {
            if let Some(path) = options.get("gamePath").and_then(|v| v.as_str()) {
                if !path.is_empty() {
                    let mut gp = self.game_path.write().await;
                    *gp = Some(path.to_string());
                }
            }
            if let Some(ignore_list) = options.get("ignoreLocalization").and_then(|v| v.as_array()) {
                let mut patterns = Vec::new();
                for val in ignore_list {
                    if let Some(s) = val.as_str() {
                        if let Ok(re) = regex::Regex::new(s) {
                            patterns.push(re);
                        }
                    }
                }
                let mut ig = self.ignored_loc_regex.write().await;
                *ig = patterns;
            }
            if let Some(enabled) = options.get("stylingEnabled").and_then(|v| v.as_bool()) {
                let mut st = self.styling_enabled.write().await;
                *st = enabled;
            }
            if let Some(enabled) = options.get("cosmeticLocIndent").and_then(|v| v.as_bool()) {
                let mut ci = self.cosmetic_loc_indent.write().await;
                *ci = enabled;
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
        let gp = self.game_path.read().await;
        if let Some(ref path) = *gp {
            roots.insert(0, std::path::PathBuf::from(path));
            self.client.log_message(MessageType::INFO, format!("Using HOI4 game path: {}", path)).await;
        }

        tokio::join!(
            self.load_localization(&roots),
            self.load_schema(),
            self.scan_scripted(&roots),
            self.scan_ideologies(&roots),
            self.scan_traits(&roots),
            self.scan_sprites(&roots),
            self.scan_ideas(&roots),
            self.scan_variables(&roots),
            self.scan_provinces(&roots),
            self.scan_modifiers(&roots),
            self.scan_buildings(&roots),
            self.scan_defines(&roots),
            self.scan_events(&roots),
            self.scan_music(&roots),
            self.scan_sounds(&roots),
        );

        // Re-validate all open documents now that we have all data
        for entry in self.documents.iter() {
            if let Ok(uri) = Url::parse(entry.key()) {
                self.validate_document(uri).await;
            }
        }
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        if let Some(settings) = params.settings.as_object() {
            if let Some(hoi4) = settings.get("hoi4").and_then(|v| v.as_object()) {
                if let Some(validator) = hoi4.get("validator").and_then(|v| v.as_object()) {
                    if let Some(ignore_list) = validator.get("ignoreLocalization").and_then(|v| v.as_array()) {
                        let mut patterns = Vec::new();
                        for val in ignore_list {
                            if let Some(s) = val.as_str() {
                                if let Ok(re) = regex::Regex::new(s) {
                                    patterns.push(re);
                                }
                            }
                        }
                        let mut ig = self.ignored_loc_regex.write().await;
                        *ig = patterns;
                    }
                }
                if let Some(styling) = hoi4.get("styling").and_then(|v| v.as_object()) {
                    if let Some(enabled) = styling.get("enabled").and_then(|v| v.as_bool()) {
                        let mut st = self.styling_enabled.write().await;
                        *st = enabled;
                    }
                    if let Some(enabled) = styling.get("cosmeticLocalizationIndentation").and_then(|v| v.as_bool()) {
                        let mut ci = self.cosmetic_loc_indent.write().await;
                        *ci = enabled;
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
        self.documents
            .insert(params.text_document.uri.to_string(), params.text_document.text);
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
        if let Some(content) = self.documents.get(&uri) {
            match parser::parse_script(&content) {
                Ok(script) => Ok(Some(semantic_tokens::get_semantic_tokens(&script))),
                Err(_) => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    async fn document_color(&self, params: DocumentColorParams) -> Result<Vec<ColorInformation>> {
        let uri = params.text_document.uri.to_string();
        if let Some(content) = self.documents.get(&uri) {
            if let Ok(script) = parser::parse_script(&content) {
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
        let is_ui = if let Some(content) = self.documents.get(&uri) {
            // Simple heuristic: check if "color_ui" appears near the color range
            // This is a basic implementation - could be improved with AST analysis
            content.contains("color_ui")
        } else {
            false
        };

        // Get color modifiers from defines
        let defines = self.defines.read().await;
        let modifiers = enhanced_color::ColorModifiers::from_defines(&defines);

        // Generate presentations for both RGB and HSV formats
        Ok(enhanced_color::generate_color_presentations(
            &params.color,
            params.range,
            is_ui,
            &modifiers,
        ))
    }

    async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri.to_string();
        if let Some(content) = self.documents.get(&uri) {
            if uri.ends_with(".yml") {
                let cosmetic_indent = *self.cosmetic_loc_indent.read().await;
                let formatted = loc_parser::format_loc_file(&content, cosmetic_indent);
                let full_range = Range {
                    start: Position { line: 0, character: 0 },
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
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let position = params.text_document_position_params.position;

        if let Some(content) = self.documents.get(&uri) {
            if uri.ends_with(".yml") {
                let (locs, _) = loc_parser::parse_loc_file(&content, &uri);
                let global_loc = self.localization.read().await;
                for entry in locs.values() {
                    // Check key
                    if position.line == entry.range.start_line && position.character >= entry.range.start_col && position.character <= entry.range.end_col {
                        let mut hover_text = format!("### 🌐 Localization: {}\n\n", entry.key);
                        hover_text.push_str(&format!("**Raw:** `{}`\n\n", entry.value));
                        hover_text.push_str("**Preview:**\n\n");
                        hover_text.push_str(&paradox_to_markdown(&entry.value, Some(&global_loc)));

                        return Ok(Some(Hover {
                            contents: HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: hover_text,
                            }),
                            range: Some(ast_range_to_lsp(&entry.range)),
                        }));
                    }
                     // Check value
                    if position.line == entry.range.start_line && position.character >= entry.value_start_col && position.character <= entry.value_start_col + entry.value.len() as u32 {
                        let mut hover_text = format!("### 👁️ Localization Preview\n\n");
                        hover_text.push_str(&paradox_to_markdown(&entry.value, Some(&global_loc)));

                        return Ok(Some(Hover {
                            contents: HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: hover_text,
                            }),
                            range: Some(Range {
                                start: Position { line: entry.range.start_line, character: entry.value_start_col },
                                end: Position { line: entry.range.start_line, character: entry.value_start_col + entry.value.len() as u32 },
                            }),
                        }));
                    }
                }
                return Ok(None);
            }

            if let Ok(script) = parser::parse_script(&content) {
                let mut scope_stack = scope::ScopeStack::new(scope::Scope::Global);
                if let Some((identifier, final_scopes, assigned_value)) = find_identifier_at(&script, position, &mut scope_stack) {
                    let mut hover_text = String::new();

                    fn push_section(full_text: &mut String, section: &str) {
                        if !full_text.is_empty() && !full_text.ends_with("---\n\n") {
                            full_text.push_str("\n\n---\n\n");
                        }
                        full_text.push_str(section);
                    }

                    // Show scope stack
                    let is_music = final_scopes.iter().any(|s| *s == scope::Scope::MusicTrack || *s == scope::Scope::MusicStation);
                    let mut scope_text = String::from(if is_music { "### 🎵 Music Definition Stack\n" } else { "### 🔍 Scope Stack\n" });
                    for (i, s) in final_scopes.iter().enumerate() {
                        if i > 0 { scope_text.push_str(" > "); }
                        scope_text.push_str(s.as_str());
                    }
                    push_section(&mut hover_text, &scope_text);

                    // Check triggers/effects/links
                    let schema = self.schema.read().await;
                    if let Some(rule) = schema.triggers.get(&identifier).or_else(|| schema.effects.get(&identifier)).or_else(|| schema.links.get(&identifier)) {
                        let mut text = format!("### 📜 Rule: {}\n", identifier);
                        if let Some(desc) = &rule.description {
                            text.push_str(&format!("\n{}\n", desc));
                        }
                        text.push_str(&format!("\nExpected Value: `{:?}`", rule.value_type));
                        push_section(&mut hover_text, &text);
                    } else if let Some(entity) = TRIGGERS.get(identifier.as_str()) {
                        push_section(&mut hover_text, &format!("### 🔍 Trigger: {}\n\n{}", entity.name, entity.description));
                    } else if let Some(entity) = EFFECTS.get(identifier.as_str()) {
                        push_section(&mut hover_text, &format!("### ⚡ Effect: {}\n\n{}", entity.name, entity.description));
                    } else if SCOPES.contains(&identifier.to_uppercase().as_str()) {
                        push_section(&mut hover_text, &format!("### 🎯 Scope: {}\n\nStandard Paradox scope.", identifier.to_uppercase()));
                    } else if LOC_COMMANDS.contains(&identifier.as_str()) {
                        push_section(&mut hover_text, &format!("### 🛠️ Localization Command: {}\n\nStandard localization command.", identifier));
                    }

                    // Check localization
                    let loc = self.localization.read().await;
                    // Try exact match first, then try keys starting with ID:
                    let entry = loc.get(&identifier).or_else(|| {
                        // Find any key that starts with "identifier:"
                        let target = format!("{}:", identifier);
                        loc.iter().find(|(k, _)| k.starts_with(&target)).map(|(_, e)| e)
                    });

                    if let Some(e) = entry {
                        let mut text = format!("### 🌐 Localization: {}\n\n", e.key);
                        text.push_str(&format!("**Raw:** `{}`\n\n", e.value));
                        text.push_str("**Preview:**\n\n");
                        text.push_str(&paradox_to_markdown(&e.value, Some(&loc)));
                        push_section(&mut hover_text, &text);
                    } else {
                        // Check scripted triggers
                        let st = self.scripted_triggers.read().await;
                        if let Some(entity) = st.get(&identifier) {
                            push_section(&mut hover_text, &format!("### 📜 Scripted Trigger: {}\n\nDefined in: {}", identifier, self.make_file_link(&entity.path)));
                        } else {
                            // Check scripted effects
                            let se = self.scripted_effects.read().await;
                            if let Some(entity) = se.get(&identifier) {
                                push_section(&mut hover_text, &format!("### 🛠️ Scripted Effect: {}\n\nDefined in: {}", identifier, self.make_file_link(&entity.path)));
                            }
                        }
                    }

                    // Check ideologies
                    let id_map = self.ideologies.read().await;
                    if let Some(ideology) = id_map.get(&identifier) {
                        push_section(&mut hover_text, &format!("### 🗳️ Ideology: {}\n\nDefined in: {}\n\nSub-ideologies: {}", 
                            ideology.name, self.make_file_link(&ideology.path), ideology.sub_ideologies.join(", ")));
                    }

                    // Check sub-ideologies
                    let sid_map = self.sub_ideologies.read().await;
                    if let Some((parent, _, path)) = sid_map.get(&identifier) {
                        push_section(&mut hover_text, &format!("### 🗳️ Sub-Ideology: {}\n\nParent Ideology: `{}`\n\nDefined in: {}", 
                            identifier, parent, self.make_file_link(path)));
                    }

                    // Check traits
                    let t_map = self.traits.read().await;
                    if let Some(trait_info) = t_map.get(&identifier) {
                        push_section(&mut hover_text, &format!("### 🎖️ Trait: {}\n\nType: `{}`\n\nDefined in: {}", 
                            trait_info.name, trait_info.trait_type, self.make_file_link(&trait_info.path)));
                    }

                    // Check sprites
                    let s_map = self.sprites.read().await;
                    if let Some(sprite) = s_map.get(&identifier) {
                        let mut texture_link = sprite.texture_file.clone();
                        // Attempt to resolve texture path relative to root
                        let gfx_path = std::path::Path::new(&sprite.path);
                        let mut root = gfx_path.parent();
                        while let Some(r) = root {
                            if r.join("interface").exists() || r.join("common").exists() {
                                let full_texture = r.join(&sprite.texture_file);
                                if full_texture.exists() {
                                    texture_link = self.make_file_link(&full_texture.to_string_lossy());
                                    break;
                                }
                            }
                            root = r.parent();
                        }

                        push_section(&mut hover_text, &format!("### 🖼️ Sprite: {}\n\nTexture: {}\n\nDefined in: {}", 
                            sprite.name, texture_link, self.make_file_link(&sprite.path)));
                    }

                    // Check events
                    let e_map = self.events.read().await;
                    if let Some(event) = e_map.get(&identifier) {
                        push_section(&mut hover_text, &format!("### 📅 Event: {}\n\nType: `{}`\n\nDefined in: {}\n\nTriggers: {}", 
                            event.id, event.event_type, self.make_file_link(&event.path), 
                            if event.triggered_events.is_empty() { "None".to_string() } else { event.triggered_events.join(", ") }));
                    }

                    // Check ideas
                    let idea_map = self.ideas.read().await;
                    if let Some(idea) = idea_map.get(&identifier) {
                        push_section(&mut hover_text, &format!("### 💡 Idea: {}\n\nCategory: `{}`\n\nDefined in: {}",
                            idea.name, idea.category, self.make_file_link(&idea.path)));
                    }

                    // Check for modifier blocks (modifier = { ... } or modifiers = { ... })
                    let identifier_lower = identifier.to_lowercase();
                    if (identifier_lower == "modifier" || identifier_lower == "modifiers") && matches!(assigned_value, Some(ast::Value::Block(_))) {
                        let mappings = self.modifier_mappings.read().await;
                        let formats = self.modifier_formats.read().await;
                        let loc = self.localization.read().await;

                        let display_service = modifier_display::ModifierDisplayService::new(
                            mappings.clone(),
                            formats.clone(),
                            loc.clone(),
                        );

                        if let Some(value) = &assigned_value {
                            let blocks = display_service.extract_modifier_blocks(value);
                            if !blocks.is_empty() {
                                let formatted = display_service.format_all_blocks(&blocks);
                                push_section(&mut hover_text, &format!("### 📊 Modifier Block\n\n{}", formatted));
                            }
                        }
                    }

                    // Check modifiers
                    let custom_mods = self.custom_modifiers.read().await;
                    if let Some(modifier) = custom_mods.get(&identifier) {
                        push_section(&mut hover_text, &format!("### 🔧 Custom Modifier: {}\n\nDefined in: {}",
                            identifier, self.make_file_link(&modifier.path)));
                    }
                    let mappings = self.modifier_mappings.read().await;
                    if let Some(loc_key) = mappings.get(&identifier) {
                        let loc = self.localization.read().await;
                        let loc_text = if let Some(e) = loc.get(loc_key) {
                            paradox_to_markdown(&e.value, Some(&loc))
                        } else {
                            loc_key.clone()
                        };

                        let formats = self.modifier_formats.read().await;
                        let format_info = formats.get(loc_key);

                        let parsed_val = match assigned_value {
                            Some(ast::Value::Number(val)) => Some(val),
                            Some(ast::Value::String(s)) => s.parse::<f64>().ok(),
                            _ => None
                        };

                        if let Some(val) = parsed_val {
                            let formatted_val = format_modifier_value(&identifier, val, format_info);
                            push_section(&mut hover_text, &format!("### 📈 {}\n\n{}", loc_text, formatted_val));
                        } else {
                            push_section(&mut hover_text, &format!("### 📉 {}\n\nEngine Modifier: `{}`", loc_text, identifier));
                        }
                    }

                    // Check variables
                    let var_map = self.variables.read().await;
                    if let Some(vars) = var_map.get(&identifier) {
                        let paths: Vec<String> = vars.iter().map(|v| self.make_file_link(&v.path)).collect();
                        push_section(&mut hover_text, &format!("### 🔢 Variable: {}\n\nUsed/Defined in:\n- {}", 
                            identifier, paths.join("\n- ")));
                    }

                    // Check event targets
                    let target_map = self.event_targets.read().await;
                    if let Some(targets) = target_map.get(&identifier) {
                        let paths: Vec<String> = targets.iter().map(|t| format!("{} ({})", self.make_file_link(&t.path), if t.is_global { "Global" } else { "Local" })).collect();
                        push_section(&mut hover_text, &format!("### 🎯 Event Target: {}\n\nSaved in:\n- {}", 
                            identifier, paths.join("\n- ")));
                    }

                    // Check music
                    let m_assets = self.music_assets.read().await;
                    if let Some(asset) = m_assets.get(&identifier) {
                        push_section(&mut hover_text, &format!("### 🎵 Music Asset: {}\n\nFile: `{}`\n\nDefined in: {}", 
                            asset.name, asset.file, self.make_file_link(&asset.path)));
                    }

                    let m_stations = self.music_stations.read().await;
                    if let Some(station) = m_stations.get(&identifier) {
                        push_section(&mut hover_text, &format!("### 📻 Music Station: {}\n\nDefined in: {}", 
                            station.name, self.make_file_link(&station.path)));
                    }

                    let m_songs = self.songs.read().await;
                    if let Some(song) = m_songs.get(&identifier) {
                        push_section(&mut hover_text, &format!("### 🎶 Song: {}\n\nDefined in: {}", 
                            song.name, self.make_file_link(&song.path)));
                    }

                    // Check sounds
                    let s_sounds = self.sounds.read().await;
                    if let Some(sound) = s_sounds.get(&identifier) {
                        let mut file_link = sound.file.clone();
                        // Try to resolve file link
                        let asset_path = std::path::Path::new(&sound.path);
                        if let Some(root) = asset_path.parent().and_then(|p| p.parent()) {
                            let full_sound_path = root.join("sound").join(&sound.file);
                            if full_sound_path.exists() {
                                file_link = self.make_file_link(&full_sound_path.to_string_lossy());
                            }
                        }

                        push_section(&mut hover_text, &format!("### 🔊 Sound: {}\n\nFile: {}\n\nDefined in: {}", 
                            sound.name, file_link, self.make_file_link(&sound.path)));
                    }

                    let s_effects = self.sound_effects.read().await;
                    if let Some(effect) = s_effects.get(&identifier) {
                        push_section(&mut hover_text, &format!("### 🔉 Sound Effect: {}\n\nSounds: `{}`\n\nDefined in: {}", 
                            effect.name, effect.sounds.join(", "), self.make_file_link(&effect.path)));
                    }

                    let s_falloffs = self.falloffs.read().await;
                    if let Some(falloff) = s_falloffs.get(&identifier) {
                        push_section(&mut hover_text, &format!("### 📉 Sound Falloff: {}\n\nDefined in: {}", 
                            falloff.name, self.make_file_link(&falloff.path)));
                    }

                    let s_categories = self.sound_categories.read().await;
                    if let Some(category) = s_categories.get(&identifier) {
                        push_section(&mut hover_text, &format!("### 📂 Sound Category: {}\n\nEffects: `{}`\n\nDefined in: {}", 
                            category.name, category.soundeffects.join(", "), self.make_file_link(&category.path)));
                    }

                    if !hover_text.is_empty() {
                        return Ok(Some(Hover {
                            contents: HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: hover_text,
                            }),
                            range: None,
                        }));
                    }
                }
            }
        }
        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
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
                        if prefix.rfind(']').map_or(true, |i| i < bracket_start) {
                            let _inner_prefix = &prefix[bracket_start + 1..];
                            let mut items = Vec::new();

                            // Provide scopes, commands, and event targets
                            for scope in SCOPES.iter() {
                                items.push(CompletionItem {
                                    label: scope.to_string(),
                                    kind: Some(CompletionItemKind::CLASS),
                                    detail: Some("Paradox Scope".to_string()),
                                    ..Default::default()
                                });
                            }
                            for command in LOC_COMMANDS.iter() {
                                items.push(CompletionItem {
                                    label: command.to_string(),
                                    kind: Some(CompletionItemKind::FUNCTION),
                                    detail: Some("Localization Command".to_string()),
                                    ..Default::default()
                                });
                            }
                            let target_map = self.event_targets.read().await;
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

        // Handle music/sound files
        let is_asset_file = uri.ends_with(".asset");
        let is_music_file = is_asset_file || uri.contains("/music/");
        let is_sound_file = is_asset_file || uri.contains("/sound/");

        if is_music_file || is_sound_file {
            if let Some(content) = self.documents.get(&uri) {
                if let Ok(script) = parser::parse_script(&content) {
                    if let Some(context_key) = find_context_at(&script, position) {
                        let mut completion_items = Vec::new();
                        let key_lower = context_key.to_lowercase();

                        if key_lower == "music" {
                            if uri.ends_with(".asset") {
                                completion_items.push(CompletionItem { label: "name".to_string(), kind: Some(CompletionItemKind::PROPERTY), detail: Some("Track ID".to_string()), ..Default::default() });
                                completion_items.push(CompletionItem { label: "file".to_string(), kind: Some(CompletionItemKind::PROPERTY), detail: Some("OGG Filename".to_string()), ..Default::default() });
                                completion_items.push(CompletionItem { label: "volume".to_string(), kind: Some(CompletionItemKind::PROPERTY), detail: Some("Volume Multiplier".to_string()), ..Default::default() });
                            } else {
                                completion_items.push(CompletionItem { label: "song".to_string(), kind: Some(CompletionItemKind::PROPERTY), detail: Some("Song ID".to_string()), ..Default::default() });
                                completion_items.push(CompletionItem { label: "chance".to_string(), kind: Some(CompletionItemKind::PROPERTY), detail: Some("Weighting logic".to_string()), ..Default::default() });
                            }
                        } else if key_lower == "sound" {
                            completion_items.push(CompletionItem { label: "name".to_string(), kind: Some(CompletionItemKind::PROPERTY), ..Default::default() });
                            completion_items.push(CompletionItem { label: "file".to_string(), kind: Some(CompletionItemKind::PROPERTY), ..Default::default() });
                            completion_items.push(CompletionItem { label: "always_load".to_string(), kind: Some(CompletionItemKind::PROPERTY), ..Default::default() });
                            completion_items.push(CompletionItem { label: "volume".to_string(), kind: Some(CompletionItemKind::PROPERTY), ..Default::default() });
                        } else if key_lower == "soundeffect" {
                            completion_items.push(CompletionItem { label: "name".to_string(), kind: Some(CompletionItemKind::PROPERTY), ..Default::default() });
                            completion_items.push(CompletionItem { label: "falloff".to_string(), kind: Some(CompletionItemKind::PROPERTY), ..Default::default() });
                            completion_items.push(CompletionItem { label: "sounds".to_string(), kind: Some(CompletionItemKind::PROPERTY), ..Default::default() });
                            completion_items.push(CompletionItem { label: "loop".to_string(), kind: Some(CompletionItemKind::PROPERTY), ..Default::default() });
                            completion_items.push(CompletionItem { label: "is3d".to_string(), kind: Some(CompletionItemKind::PROPERTY), ..Default::default() });
                            completion_items.push(CompletionItem { label: "volume".to_string(), kind: Some(CompletionItemKind::PROPERTY), ..Default::default() });
                        } else if key_lower == "falloff" {
                            completion_items.push(CompletionItem { label: "name".to_string(), kind: Some(CompletionItemKind::PROPERTY), ..Default::default() });
                            completion_items.push(CompletionItem { label: "min_distance".to_string(), kind: Some(CompletionItemKind::PROPERTY), ..Default::default() });
                            completion_items.push(CompletionItem { label: "max_distance".to_string(), kind: Some(CompletionItemKind::PROPERTY), ..Default::default() });
                            completion_items.push(CompletionItem { label: "height_scale".to_string(), kind: Some(CompletionItemKind::PROPERTY), ..Default::default() });
                        } else if key_lower == "category" {
                            completion_items.push(CompletionItem { label: "name".to_string(), kind: Some(CompletionItemKind::PROPERTY), ..Default::default() });
                            completion_items.push(CompletionItem { label: "soundeffects".to_string(), kind: Some(CompletionItemKind::PROPERTY), ..Default::default() });
                            completion_items.push(CompletionItem { label: "compressor".to_string(), kind: Some(CompletionItemKind::PROPERTY), ..Default::default() });
                        } else if key_lower == "chance" || key_lower == "modifier" {
                            completion_items.push(CompletionItem { label: "factor".to_string(), kind: Some(CompletionItemKind::PROPERTY), ..Default::default() });
                            completion_items.push(CompletionItem { label: "add".to_string(), kind: Some(CompletionItemKind::PROPERTY), ..Default::default() });
                            completion_items.push(CompletionItem { label: "base".to_string(), kind: Some(CompletionItemKind::PROPERTY), ..Default::default() });
                            if key_lower == "chance" {
                                completion_items.push(CompletionItem { label: "modifier".to_string(), kind: Some(CompletionItemKind::CLASS), ..Default::default() });
                            }
                        }

                        if !completion_items.is_empty() {
                            return Ok(Some(CompletionResponse::Array(completion_items)));
                        }
                    } else {
                        // Top level
                        let mut top_items = Vec::new();
                        if is_music_file {
                            top_items.push(CompletionItem { label: "music".to_string(), kind: Some(CompletionItemKind::CLASS), ..Default::default() });
                            if !uri.ends_with(".asset") {
                                top_items.push(CompletionItem { label: "music_station".to_string(), kind: Some(CompletionItemKind::PROPERTY), ..Default::default() });
                            }
                        }
                        if is_sound_file {
                            top_items.push(CompletionItem { label: "sound".to_string(), kind: Some(CompletionItemKind::CLASS), ..Default::default() });
                            top_items.push(CompletionItem { label: "soundeffect".to_string(), kind: Some(CompletionItemKind::CLASS), ..Default::default() });
                            top_items.push(CompletionItem { label: "falloff".to_string(), kind: Some(CompletionItemKind::CLASS), ..Default::default() });
                            top_items.push(CompletionItem { label: "category".to_string(), kind: Some(CompletionItemKind::CLASS), ..Default::default() });
                        }
                        return Ok(Some(CompletionResponse::Array(top_items)));
                    }
                }
            }
        }

        // Try to find context for HOI4 scripts
        if let Some(content) = self.documents.get(&uri) {
            if let Ok(script) = parser::parse_script(&content) {
                if let Some(context_key) = find_context_at(&script, position) {
                    if context_key.to_lowercase().contains("color") {
                        let mut color_items = Vec::new();
                        color_items.push(CompletionItem {
                            label: "rgb".to_string(),
                            kind: Some(CompletionItemKind::KEYWORD),
                            detail: Some("RGB Color Format".to_string()),
                            ..Default::default()
                        });
                        color_items.push(CompletionItem {
                            label: "hsv".to_string(),
                            kind: Some(CompletionItemKind::KEYWORD),
                            detail: Some("HSV Color Format".to_string()),
                            ..Default::default()
                        });
                        return Ok(Some(CompletionResponse::Array(color_items)));
                    }
                }
            }
        }

        let mut items = Vec::new();

        let schema = self.schema.read().await;
        for trigger in schema.triggers.values() {
            items.push(CompletionItem {
                label: trigger.key.clone(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Trigger (Schema)".to_string()),
                documentation: trigger.description.as_ref().map(|d| Documentation::String(d.clone())),
                ..Default::default()
            });
        }
        for effect in schema.effects.values() {
            items.push(CompletionItem {
                label: effect.key.clone(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Effect (Schema)".to_string()),
                documentation: effect.description.as_ref().map(|d| Documentation::String(d.clone())),
                ..Default::default()
            });
        }
        for link in schema.links.values() {
            items.push(CompletionItem {
                label: link.key.clone(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some(format!("Link / Scope (Push: {:?})", link.push_scope)),
                documentation: link.description.as_ref().map(|d| Documentation::String(d.clone())),
                ..Default::default()
            });
        }

        for trigger in TRIGGERS.values() {
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

        for effect in EFFECTS.values() {
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

        let st = self.scripted_triggers.read().await;
        for trigger in st.values() {
            items.push(CompletionItem {
                label: trigger.name.clone(),
                kind: Some(CompletionItemKind::EVENT),
                detail: Some("Scripted Trigger".to_string()),
                documentation: Some(Documentation::String(format!("Defined in: {}", trigger.path))),
                ..Default::default()
            });
        }

        let se = self.scripted_effects.read().await;
        for effect in se.values() {
            items.push(CompletionItem {
                label: effect.name.clone(),
                kind: Some(CompletionItemKind::EVENT),
                detail: Some("Scripted Effect".to_string()),
                documentation: Some(Documentation::String(format!("Defined in: {}", effect.path))),
                ..Default::default()
            });
        }

        let ids = self.ideologies.read().await;
        for ideology in ids.values() {
            items.push(CompletionItem {
                label: ideology.name.clone(),
                kind: Some(CompletionItemKind::ENUM),
                detail: Some("Ideology".to_string()),
                documentation: Some(Documentation::String(format!("Defined in: {}", ideology.path))),
                ..Default::default()
            });
        }

        let sids = self.sub_ideologies.read().await;
        for (sid, (parent, _, _)) in sids.iter() {
            items.push(CompletionItem {
                label: sid.clone(),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                detail: Some(format!("Sub-Ideology (Parent: {})", parent)),
                ..Default::default()
            });
        }

        let traits = self.traits.read().await;
        for trait_info in traits.values() {
            items.push(CompletionItem {
                label: trait_info.name.clone(),
                kind: Some(CompletionItemKind::INTERFACE),
                detail: Some(trait_info.trait_type.clone()),
                documentation: Some(Documentation::String(format!("Defined in: {}", trait_info.path))),
                ..Default::default()
            });
        }

        let s_map = self.sprites.read().await;
        for sprite in s_map.values() {
            items.push(CompletionItem {
                label: sprite.name.clone(),
                kind: Some(CompletionItemKind::FILE),
                detail: Some("Sprite/GFX".to_string()),
                documentation: Some(Documentation::String(format!("Defined in: {}", sprite.path))),
                ..Default::default()
            });
        }

        let id_map = self.ideas.read().await;
        for idea in id_map.values() {
            items.push(CompletionItem {
                label: idea.name.clone(),
                kind: Some(CompletionItemKind::CONSTANT),
                detail: Some(format!("Idea ({})", idea.category)),
                documentation: Some(Documentation::String(format!("Defined in: {}", idea.category))),
                ..Default::default()
            });
        }

        let var_map = self.variables.read().await;
        for var_name in var_map.keys() {
            items.push(CompletionItem {
                label: var_name.clone(),
                kind: Some(CompletionItemKind::VARIABLE),
                detail: Some("Variable".to_string()),
                ..Default::default()
            });
        }

        let target_map = self.event_targets.read().await;
        for target_name in target_map.keys() {
            items.push(CompletionItem {
                label: target_name.clone(),
                kind: Some(CompletionItemKind::STRUCT),
                detail: Some("Event Target".to_string()),
                ..Default::default()
            });
        }

        let m_assets = self.music_assets.read().await;
        for asset in m_assets.values() {
            items.push(CompletionItem {
                label: asset.name.clone(),
                kind: Some(CompletionItemKind::FILE),
                detail: Some("Music Asset".to_string()),
                documentation: Some(Documentation::String(format!("File: {}", asset.file))),
                ..Default::default()
            });
        }

        let m_stations = self.music_stations.read().await;
        for station in m_stations.values() {
            items.push(CompletionItem {
                label: station.name.clone(),
                kind: Some(CompletionItemKind::FOLDER),
                detail: Some("Music Station".to_string()),
                ..Default::default()
            });
        }

        let m_songs = self.songs.read().await;
        for song in m_songs.values() {
            items.push(CompletionItem {
                label: song.name.clone(),
                kind: Some(CompletionItemKind::FILE),
                detail: Some("Song".to_string()),
                ..Default::default()
            });
        }

        let s_sounds = self.sounds.read().await;
        for sound in s_sounds.values() {
            items.push(CompletionItem {
                label: sound.name.clone(),
                kind: Some(CompletionItemKind::FILE),
                detail: Some("Sound".to_string()),
                documentation: Some(Documentation::String(format!("File: {}", sound.file))),
                ..Default::default()
            });
        }

        let s_effects = self.sound_effects.read().await;
        for effect in s_effects.values() {
            items.push(CompletionItem {
                label: effect.name.clone(),
                kind: Some(CompletionItemKind::EVENT),
                detail: Some("Sound Effect".to_string()),
                ..Default::default()
            });
        }

        let s_falloffs = self.falloffs.read().await;
        for falloff in s_falloffs.values() {
            items.push(CompletionItem {
                label: falloff.name.clone(),
                kind: Some(CompletionItemKind::UNIT),
                detail: Some("Sound Falloff".to_string()),
                ..Default::default()
            });
        }

        let s_categories = self.sound_categories.read().await;
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

    async fn completion_resolve(&self, params: CompletionItem) -> Result<CompletionItem> {
        Ok(params)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let position = params.text_document_position_params.position;

        if let Some(content) = self.documents.get(&uri) {
            let identifier = if uri.ends_with(".yml") {
                find_identifier_in_loc(&content, position)
            } else if let Ok(script) = parser::parse_script(&content) {
                let mut scope_stack = scope::ScopeStack::new(scope::Scope::Global);
                find_identifier_at(&script, position, &mut scope_stack).map(|(id, _, _)| id)
            } else {
                None
            };

            if let Some(identifier) = identifier {
                let mut sources = Vec::new();
                let mut localizations = Vec::new();

                // 1. Check scripted elements
                let st = self.scripted_triggers.read().await;
                if let Some(entity) = st.get(&identifier) {
                    sources.push(ast_range_to_lsp_location(&entity.range, &entity.path));
                }

                let se = self.scripted_effects.read().await;
                if let Some(entity) = se.get(&identifier) {
                    sources.push(ast_range_to_lsp_location(&entity.range, &entity.path));
                }

                // 2. Check ideologies
                    let id_map = self.ideologies.read().await;
                    if let Some(ideology) = id_map.get(&identifier) {
                        sources.push(ast_range_to_lsp_location(&ideology.range, &ideology.path));
                    }

                    let sid_map = self.sub_ideologies.read().await;
                    if let Some((_, range, path)) = sid_map.get(&identifier) {
                        sources.push(ast_range_to_lsp_location(range, path));
                    }

                    // 3. Check traits
                    let t_map = self.traits.read().await;
                    if let Some(trait_info) = t_map.get(&identifier) {
                        sources.push(ast_range_to_lsp_location(&trait_info.range, &trait_info.path));
                    }

                    // 4. Check sprites
                    let s_map = self.sprites.read().await;
                    if let Some(sprite) = s_map.get(&identifier) {
                        sources.push(ast_range_to_lsp_location(&sprite.range, &sprite.path));
                    }

                    // 5. Check events
                    let e_map = self.events.read().await;
                    if let Some(event) = e_map.get(&identifier) {
                        sources.push(ast_range_to_lsp_location(&event.range, &event.path));
                    }

                    // 6. Check ideas
                    let idea_map = self.ideas.read().await;
                    if let Some(idea) = idea_map.get(&identifier) {
                        sources.push(ast_range_to_lsp_location(&idea.range, &idea.path));
                    }

                    // 6. Check variables
                    let var_map = self.variables.read().await;
                    if let Some(vars) = var_map.get(&identifier) {
                        for var in vars {
                            sources.push(ast_range_to_lsp_location(&var.range, &var.path));
                        }
                    }

                    // 7. Check event targets
                    let target_map = self.event_targets.read().await;
                    if let Some(targets) = target_map.get(&identifier) {
                        for target in targets {
                            sources.push(ast_range_to_lsp_location(&target.range, &target.path));
                        }
                    }

                    // 8. Check modifiers
                    let custom_mods = self.custom_modifiers.read().await;
                    if let Some(modifier) = custom_mods.get(&identifier) {
                        sources.push(ast_range_to_lsp_location(&modifier.range, &modifier.path));
                    }

                    // 9. Check music
                    let m_assets = self.music_assets.read().await;
                    if let Some(asset) = m_assets.get(&identifier) {
                        sources.push(ast_range_to_lsp_location(&asset.range, &asset.path));
                    }

                    let m_stations = self.music_stations.read().await;
                    if let Some(station) = m_stations.get(&identifier) {
                        sources.push(ast_range_to_lsp_location(&station.range, &station.path));
                    }

                    let m_songs = self.songs.read().await;
                    if let Some(song) = m_songs.get(&identifier) {
                        sources.push(ast_range_to_lsp_location(&song.range, &song.path));
                    }

                    // 10. Check sounds
                    let s_sounds = self.sounds.read().await;
                    if let Some(sound) = s_sounds.get(&identifier) {
                        sources.push(ast_range_to_lsp_location(&sound.range, &sound.path));
                    }

                    let s_effects = self.sound_effects.read().await;
                    if let Some(effect) = s_effects.get(&identifier) {
                        sources.push(ast_range_to_lsp_location(&effect.range, &effect.path));
                    }

                    let s_falloffs = self.falloffs.read().await;
                    if let Some(falloff) = s_falloffs.get(&identifier) {
                        sources.push(ast_range_to_lsp_location(&falloff.range, &falloff.path));
                    }

                    let s_categories = self.sound_categories.read().await;
                    if let Some(category) = s_categories.get(&identifier) {
                        sources.push(ast_range_to_lsp_location(&category.range, &category.path));
                    }

                    let mappings = self.modifier_mappings.read().await;
                    if let Some(loc_key) = mappings.get(&identifier) {
                        let loc = self.localization.read().await;
                        if let Some(e) = loc.get(loc_key) {
                            localizations.push(ast_range_to_lsp_location(&e.range, &e.path));
                        }
                    }

                    // 9. Check localization
                    let loc = self.localization.read().await;
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

    async fn references(
        &self,
        params: ReferenceParams,
    ) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;

        if let Some(content) = self.documents.get(&uri) {
            if let Ok(script) = parser::parse_script(&content) {
                let mut scope_stack = scope::ScopeStack::new(scope::Scope::Global);
                if let Some((identifier, _, _)) = find_identifier_at(&script, position, &mut scope_stack) {
                    let mut locations = Vec::new();

                    // Search in all roots
                    let mut roots = vec![std::path::PathBuf::from(".")];
                    let gp = self.game_path.read().await;
                    if let Some(ref path) = *gp {
                        roots.push(std::path::PathBuf::from(path));
                    }

                    for root in roots {
                        self.find_references_in_root(&root, &identifier, &mut locations).await;
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
        let mut actions = Vec::new();
        let mut has_casing_diagnostic = false;
        let mut has_trailing_whitespace_diagnostic = false;
        let mut has_mixed_indentation_diagnostic = false;
        let mut has_assignment_space_diagnostic = false;
        let mut has_brace_space_diagnostic = false;

        for diagnostic in &params.context.diagnostics {
            if let Some(target_casing) = diagnostic.data.as_ref().and_then(|v| v.as_str()) {
                let is_casing_fix = match &diagnostic.code {
                    Some(NumberOrString::String(s)) => s == "casing",
                    _ => diagnostic.message.contains("Standard Paradox Script") || diagnostic.message.contains("Standard casing")
                };

                if is_casing_fix {
                    has_casing_diagnostic = true;
                    let mut changes = HashMap::new();
                    changes.insert(params.text_document.uri.clone(), vec![TextEdit {
                        range: diagnostic.range,
                        new_text: target_casing.to_string(),
                    }]);

                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                        title: format!("Change to standard casing: '{}'", target_casing),
                        kind: Some(CodeActionKind::QUICKFIX),
                        edit: Some(WorkspaceEdit {
                            changes: Some(changes),
                            ..Default::default()
                        }),
                        diagnostics: Some(vec![diagnostic.clone()]),
                        is_preferred: Some(true),
                        ..Default::default()
                    }));
                }
            } else {
                // Check other styling codes
                if let Some(NumberOrString::String(code)) = &diagnostic.code {
                    if code == "styling_trailing" {
                        has_trailing_whitespace_diagnostic = true;
                        let mut changes = HashMap::new();
                        changes.insert(params.text_document.uri.clone(), vec![TextEdit {
                            range: diagnostic.range,
                            new_text: "".to_string(),
                        }]);

                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Remove trailing whitespace".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            edit: Some(WorkspaceEdit {
                                changes: Some(changes),
                                ..Default::default()
                            }),
                            diagnostics: Some(vec![diagnostic.clone()]),
                            is_preferred: Some(true),
                            ..Default::default()
                        }));
                    } else if code == "styling_eof_newline" {
                        let mut changes = HashMap::new();
                        changes.insert(params.text_document.uri.clone(), vec![TextEdit {
                            range: diagnostic.range,
                            new_text: "\n".to_string(),
                        }]);

                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Add empty newline at end of file".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            edit: Some(WorkspaceEdit {
                                changes: Some(changes),
                                ..Default::default()
                            }),
                            diagnostics: Some(vec![diagnostic.clone()]),
                            is_preferred: Some(true),
                            ..Default::default()
                        }));
                    } else if code == "styling_assignment_space" {
                        has_assignment_space_diagnostic = true;
                        if let Some(content) = self.documents.get(&params.text_document.uri.to_string()) {
                            let line_idx = diagnostic.range.start.line as usize;
                            if let Some(line) = content.lines().nth(line_idx) {
                                let start = diagnostic.range.start.character as usize;
                                let end = diagnostic.range.end.character as usize;
                                if start <= end && end <= line.len() {
                                    let op_str = &line[start..end];
                                    let mut changes = HashMap::new();
                                    changes.insert(params.text_document.uri.clone(), vec![TextEdit {
                                        range: diagnostic.range,
                                        new_text: format!(" {} ", op_str.trim()),
                                    }]);

                                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                        title: "Surround with spaces".to_string(),
                                        kind: Some(CodeActionKind::QUICKFIX),
                                        edit: Some(WorkspaceEdit {
                                            changes: Some(changes),
                                            ..Default::default()
                                        }),
                                        diagnostics: Some(vec![diagnostic.clone()]),
                                        is_preferred: Some(true),
                                        ..Default::default()
                                    }));
                                }
                            }
                        }
                    } else if code == "styling_brace_space" {
                        has_brace_space_diagnostic = true;
                        if let Some(content) = self.documents.get(&params.text_document.uri.to_string()) {
                            let line_idx = diagnostic.range.start.line as usize;
                            if let Some(line) = content.lines().nth(line_idx) {
                                let start = diagnostic.range.start.character as usize;
                                let end = diagnostic.range.end.character as usize;
                                if start < end && end <= line.len() {
                                    let full_str = &line[start..end];
                                    if let Some(brace_start_rel) = full_str.find('{') {
                                        let brace_end_rel = full_str.rfind('}').unwrap_or(full_str.len() - 1);
                                        let inner = &full_str[brace_start_rel+1..brace_end_rel];

                                        let before_brace = full_str[..brace_start_rel].trim();

                                        let new_text = if inner.trim().is_empty() {
                                            if !before_brace.is_empty() { format!("{} {{}}", before_brace) } else { "{}".to_string() }
                                        } else {
                                            if !before_brace.is_empty() { format!("{} {{ {} }}", before_brace, inner.trim()) } else { format!("{{ {} }}", inner.trim()) }
                                        };

                                        let mut changes = HashMap::new();
                                        changes.insert(params.text_document.uri.clone(), vec![TextEdit {
                                            range: diagnostic.range,
                                            new_text,
                                        }]);

                                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                            title: "Fix curly brace spacing".to_string(),
                                            kind: Some(CodeActionKind::QUICKFIX),
                                            edit: Some(WorkspaceEdit {
                                                changes: Some(changes),
                                                ..Default::default()
                                            }),
                                            diagnostics: Some(vec![diagnostic.clone()]),
                                            is_preferred: Some(true),
                                            ..Default::default()
                                        }));
                                    }
                                }
                            }
                        }
                    } else if code == "styling_brace_newline" {
                        has_brace_space_diagnostic = true;
                        let mut changes = HashMap::new();
                        changes.insert(params.text_document.uri.clone(), vec![TextEdit {
                            range: diagnostic.range,
                            new_text: " ".to_string(),
                        }]);

                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Move curly brace to same line".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            edit: Some(WorkspaceEdit {
                                changes: Some(changes),
                                ..Default::default()
                            }),
                            diagnostics: Some(vec![diagnostic.clone()]),
                            is_preferred: Some(true),
                            ..Default::default()
                        }));
                    } else if code == "duplicate_key" {
                        let mut changes = HashMap::new();
                        changes.insert(params.text_document.uri.clone(), vec![TextEdit {
                            range: diagnostic.range,
                            new_text: "".to_string(),
                        }]);

                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Remove this duplicate modifier".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            edit: Some(WorkspaceEdit {
                                changes: Some(changes),
                                ..Default::default()
                            }),
                            diagnostics: Some(vec![diagnostic.clone()]),
                            is_preferred: Some(true),
                            ..Default::default()
                        }));
                    } else if code == "styling_indent" {
                        has_mixed_indentation_diagnostic = true;
                        if let Some(content) = self.documents.get(&params.text_document.uri.to_string()) {
                            let line_idx = diagnostic.range.start.line as usize;
                            if let Some(line) = content.lines().nth(line_idx) {
                                let leading = line.chars().take_while(|c| c.is_whitespace()).collect::<String>();

                                let new_indent = if let Some(expected_tabs) = diagnostic.data.as_ref().and_then(|v| v.get("expected_tabs")).and_then(|v| v.as_u64()) {
                                    "\t".repeat(expected_tabs as usize)
                                } else {
                                    let mut n = leading.replace("    ", "\t").replace("  ", "\t").replace(" ", "");
                                    if n.is_empty() && !leading.is_empty() {
                                        n = "\t".to_string();
                                    }
                                    n
                                };

                                let mut changes = HashMap::new();
                                changes.insert(params.text_document.uri.clone(), vec![TextEdit {
                                    range: diagnostic.range,
                                    new_text: new_indent,
                                }]);

                                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                    title: "Convert indentation to tabs".to_string(),
                                    kind: Some(CodeActionKind::QUICKFIX),
                                    edit: Some(WorkspaceEdit {
                                        changes: Some(changes),
                                        ..Default::default()
                                    }),
                                    diagnostics: Some(vec![diagnostic.clone()]),
                                    is_preferred: Some(true),
                                    ..Default::default()
                                }));
                            }
                        }
                    }
                }
            }
        }

        // Add "Fix all" if any casing diagnostic is present
        if has_casing_diagnostic {
            if let Some(content) = self.documents.get(&params.text_document.uri.to_string()) {
                if let Ok(script) = parser::parse_script(&content) {
                    let mut all_fixes = Vec::new();
                    self.collect_casing_fixes(&script.entries, &mut all_fixes);

                    if !all_fixes.is_empty() {
                        let mut changes = HashMap::new();
                        let edits: Vec<TextEdit> = all_fixes.into_iter().map(|(range, text)| TextEdit {
                            range: ast_range_to_lsp(&range),
                            new_text: text,
                        }).collect();

                        changes.insert(params.text_document.uri.clone(), edits);

                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Fix all casing convention issues in this file".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            edit: Some(WorkspaceEdit {
                                changes: Some(changes),
                                ..Default::default()
                            }),
                            is_preferred: Some(false),
                            ..Default::default()
                        }));
                    }
                }
            }
        }

        // Add "Remove all trailing whitespace" if any such diagnostic is present
        if has_trailing_whitespace_diagnostic {
            if let Some(content) = self.documents.get(&params.text_document.uri.to_string()) {
                let mut all_fixes = Vec::new();
                self.collect_styling_fixes(&content, &mut all_fixes);

                if !all_fixes.is_empty() {
                    let mut changes = HashMap::new();
                    let edits: Vec<TextEdit> = all_fixes.into_iter().map(|(range, text)| TextEdit {
                        range,
                        new_text: text,
                    }).collect();

                    changes.insert(params.text_document.uri.clone(), edits);

                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                        title: "Remove all trailing whitespaces in this file".to_string(),
                        kind: Some(CodeActionKind::QUICKFIX),
                        edit: Some(WorkspaceEdit {
                            changes: Some(changes),
                            ..Default::default()
                        }),
                        is_preferred: Some(false),
                        ..Default::default()
                    }));
                }
            }
        }

        // Add "Convert all mixed indentation to tabs" if any such diagnostic is present
        if has_mixed_indentation_diagnostic {
            if let Some(content) = self.documents.get(&params.text_document.uri.to_string()) {
                let parsed = parser::parse_script(&content);
                let script_opt = parsed.as_ref().ok();

                let mut all_fixes = Vec::new();
                self.collect_indentation_fixes(&content, script_opt, &mut all_fixes);

                if !all_fixes.is_empty() {
                    let mut changes = HashMap::new();
                    let edits: Vec<TextEdit> = all_fixes.into_iter().map(|(range, text)| TextEdit {
                        range,
                        new_text: text,
                    }).collect();

                    changes.insert(params.text_document.uri.clone(), edits);

                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                        title: "Convert all mixed indentation to tabs in this file".to_string(),
                        kind: Some(CodeActionKind::QUICKFIX),
                        edit: Some(WorkspaceEdit {
                            changes: Some(changes),
                            ..Default::default()
                        }),
                        is_preferred: Some(false),
                        ..Default::default()
                    }));
                }
            }
        }

        // Add "Surround all assignment operators with spaces" if any such diagnostic is present
        if has_assignment_space_diagnostic {
            if let Some(content) = self.documents.get(&params.text_document.uri.to_string()) {
                if let Ok(script) = parser::parse_script(&content) {
                    let mut all_fixes = Vec::new();
                    self.collect_assignment_space_fixes(&script.entries, &mut all_fixes, &content);

                    if !all_fixes.is_empty() {
                        let mut changes = HashMap::new();
                        let edits: Vec<TextEdit> = all_fixes.into_iter().map(|(range, text)| TextEdit {
                            range: ast_range_to_lsp(&range),
                            new_text: text,
                        }).collect();

                        changes.insert(params.text_document.uri.clone(), edits);

                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Surround all assignment operators with spaces in this file".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            edit: Some(WorkspaceEdit {
                                changes: Some(changes),
                                ..Default::default()
                            }),
                            is_preferred: Some(false),
                            ..Default::default()
                        }));
                    }
                }
            }
        }

        // Add "Fix curly brace spacing" if any such diagnostic is present
        if has_brace_space_diagnostic {
            if let Some(content) = self.documents.get(&params.text_document.uri.to_string()) {
                if let Ok(script) = parser::parse_script(&content) {
                    let mut all_fixes = Vec::new();
                    self.collect_brace_space_fixes(&script.entries, &mut all_fixes, &content);
                    self.collect_brace_newline_fixes(&script.entries, &mut all_fixes);

                    if !all_fixes.is_empty() {
                        let mut changes = HashMap::new();
                        let edits: Vec<TextEdit> = all_fixes.into_iter().map(|(range, text)| TextEdit {
                            range: ast_range_to_lsp(&range),
                            new_text: text,
                        }).collect();

                        changes.insert(params.text_document.uri.clone(), edits);

                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Fix all curly brace issues in this file".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            edit: Some(WorkspaceEdit {
                                changes: Some(changes),
                                ..Default::default()
                            }),
                            is_preferred: Some(false),
                            ..Default::default()
                        }));
                    }
                }
            }
        }

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<serde_json::Value>> {
        if params.command == "hoi4.getEventGraph" {
            let events = self.events.read().await;
            let json = serde_json::to_value(&*events).unwrap();
            return Ok(Some(json));
        } else if params.command == "hoi4/getMemoryUsage" {
            let mut sys = sysinfo::System::new();
            sys.refresh_processes();
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

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

impl Backend {
    fn make_file_link(&self, path: &str) -> String {
        // Try to canonicalize for absolute path if possible
        let abs_path = std::path::Path::new(path).canonicalize()
            .unwrap_or_else(|_| std::path::PathBuf::from(path));
        format!("[{}]({}://{})", path, "file", abs_path.to_string_lossy().replace("\\", "/"))
    }

    fn check_is_province(&self, val: &ast::NodeedValue, diagnostics: &mut Vec<Diagnostic>, provs: &HashSet<u32>) {
        let id_opt = match &val.value {
            ast::Value::Number(n) => Some(*n as u32),
            ast::Value::String(s) => s.parse::<u32>().ok(),
            _ => None,
        };

        if let Some(id) = id_opt {
            if !provs.is_empty() && !provs.contains(&id) {
                diagnostics.push(Diagnostic {
                    range: ast_range_to_lsp(&val.range),
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: format!("Unknown province ID: {}", id),
                    ..Default::default()
                });
            }
        }
    }

    async fn scan_provinces(&self, roots: &[std::path::PathBuf]) {
        let result = province_scanner::scan_provinces(roots);
        let mut provinces = self.provinces.write().await;
        *provinces = result;
        self.client.log_message(MessageType::INFO, format!("Total: Loaded {} province definitions", provinces.len())).await;
    }

    async fn scan_events(&self, roots: &[std::path::PathBuf]) {
        let result = event_scanner::scan_events(roots);
        let mut events = self.events.write().await;
        *events = result;
        self.client.log_message(MessageType::INFO, format!("Total: Loaded {} event definitions", events.len())).await;
    }

    async fn scan_music(&self, roots: &[std::path::PathBuf]) {
        let result = music_scanner::scan_music(roots);

        let mut assets = self.music_assets.write().await;
        *assets = result.assets;

        let mut stations = self.music_stations.write().await;
        *stations = result.stations;

        let mut songs = self.songs.write().await;
        *songs = result.songs;

        self.client.log_message(MessageType::INFO, format!("Total: Loaded {} music assets, {} stations, and {} songs", assets.len(), stations.len(), songs.len())).await;
    }

    async fn scan_sounds(&self, roots: &[std::path::PathBuf]) {
        let result = sound_scanner::scan_sounds(roots);

        let mut sounds = self.sounds.write().await;
        *sounds = result.sounds;

        let mut effects = self.sound_effects.write().await;
        *effects = result.sound_effects;

        let mut falloffs = self.falloffs.write().await;
        *falloffs = result.falloffs;

        let mut categories = self.sound_categories.write().await;
        *categories = result.categories;

        self.client.log_message(MessageType::INFO, format!("Total: Loaded {} sounds, {} sound effects, {} falloffs, and {} categories", sounds.len(), effects.len(), falloffs.len(), categories.len())).await;
    }

    async fn scan_modifiers(&self, roots: &[std::path::PathBuf]) {
        let result = modifier_scanner::scan_modifiers(roots);

        let mut custom = self.custom_modifiers.write().await;
        *custom = result.custom_modifiers;

        let mut mappings = self.modifier_mappings.write().await;
        *mappings = result.builtin_mappings;

        self.client.log_message(MessageType::INFO, format!("Total: Loaded {} custom modifiers and {} builtin mappings", custom.len(), mappings.len())).await;
    }

    async fn scan_buildings(&self, roots: &[std::path::PathBuf]) {
        let buildings = building_scanner::scan_buildings(roots);

        let mut b = self.buildings.write().await;
        *b = buildings;

        self.client.log_message(MessageType::INFO, format!("Total: Loaded {} buildings", b.len())).await;
    }

    async fn scan_defines(&self, roots: &[std::path::PathBuf]) {
        let defines = defines_parser::scan_defines(roots);

        let mut d = self.defines.write().await;
        *d = defines;

        self.client.log_message(MessageType::INFO, "Loaded game defines").await;
    }

    async fn scan_variables(&self, roots: &[std::path::PathBuf]) {
        let result = variable_scanner::scan_roots(roots);

        let mut vars = self.variables.write().await;
        *vars = result.variables;

        let mut targets = self.event_targets.write().await;
        *targets = result.event_targets;

        self.client.log_message(MessageType::INFO, format!("Total: Loaded {} variables and {} event targets", vars.len(), targets.len())).await;
    }

    fn collect_styling_fixes(&self, content: &str, fixes: &mut Vec<(Range, String)>) {
        for (line_idx, line) in content.lines().enumerate() {
            if line.ends_with(' ') || line.ends_with('\t') {
                let start_col = line.trim_end().len() as u32;
                fixes.push((
                    Range {
                        start: Position { line: line_idx as u32, character: start_col },
                        end: Position { line: line_idx as u32, character: line.len() as u32 },
                    },
                    "".to_string(),
                ));
            }
        }
    }

    fn collect_indentation_fixes(&self, content: &str, script_opt: Option<&ast::Script>, fixes: &mut Vec<(Range, String)>) {
        let mut expected_indents = HashMap::new();
        if let Some(script) = script_opt {
            Self::compute_expected_indentations(&script.entries, 0, &mut expected_indents);
        }

        for (line_idx, line) in content.lines().enumerate() {
            let leading = line.chars().take_while(|c| c.is_whitespace()).collect::<String>();
            if line.trim().is_empty() { continue; }

            if let Some(&expected_tabs) = expected_indents.get(&(line_idx as u32)) {
                let expected_str = "\t".repeat(expected_tabs);
                if leading != expected_str {
                    fixes.push((
                        Range {
                            start: Position { line: line_idx as u32, character: 0 },
                            end: Position { line: line_idx as u32, character: leading.len() as u32 },
                        },
                        expected_str,
                    ));
                }
            } else if leading.contains(' ') {
                let mut new_indent = leading.replace("    ", "\t").replace("  ", "\t").replace(" ", "");
                if new_indent.is_empty() && !leading.is_empty() {
                    new_indent = "\t".to_string();
                }
                if new_indent != leading {
                    fixes.push((
                        Range {
                            start: Position { line: line_idx as u32, character: 0 },
                            end: Position { line: line_idx as u32, character: leading.len() as u32 },
                        },
                        new_indent,
                    ));
                }
            }
        }
    }

    fn collect_assignment_space_fixes(&self, entries: &[ast::Entry], fixes: &mut Vec<(ast::Range, String)>, content: &str) {
        for entry in entries {
            match entry {
                ast::Entry::Assignment(ass) => {
                    let mut needs_fix = false;
                    if ass.key_range.end_line == ass.operator_range.start_line && ass.key_range.end_line == ass.value.range.start_line {
                        if ass.operator_range.start_col > ass.key_range.end_col && ass.value.range.start_col > ass.operator_range.end_col {
                            let space_before = ass.operator_range.start_col - ass.key_range.end_col;
                            let space_after = ass.value.range.start_col - ass.operator_range.end_col;
                            if space_before != 1 || space_after != 1 {
                                needs_fix = true;
                            }
                        } else {
                            needs_fix = true;
                        }
                    }

                    if needs_fix {
                        let line_idx = ass.key_range.end_line as usize;
                        if let Some(line) = content.lines().nth(line_idx) {
                            let start = ass.key_range.end_col as usize;
                            let end = ass.value.range.start_col as usize;
                            if start <= end && end <= line.len() {
                                let op_str = &line[start..end];
                                fixes.push((
                                    ast::Range {
                                        start_line: ass.key_range.end_line,
                                        start_col: ass.key_range.end_col,
                                        end_line: ass.value.range.start_line,
                                        end_col: ass.value.range.start_col,
                                    },
                                    format!(" {} ", op_str.trim())
                                ));
                            }
                        }
                    }

                    match &ass.value.value {
                        ast::Value::Block(inner) => self.collect_assignment_space_fixes(inner, fixes, content),
                        ast::Value::TaggedBlock(_, inner, _) => self.collect_assignment_space_fixes(inner, fixes, content),
                        _ => {}
                    }
                }
                ast::Entry::Value(val) => {
                    match &val.value {
                        ast::Value::Block(inner) => self.collect_assignment_space_fixes(inner, fixes, content),
                        ast::Value::TaggedBlock(_, inner, _) => self.collect_assignment_space_fixes(inner, fixes, content),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    fn collect_brace_newline_fixes(&self, entries: &[ast::Entry], fixes: &mut Vec<(ast::Range, String)>) {
        for entry in entries {
            match entry {
                ast::Entry::Assignment(ass) => {
                    match &ass.value.value {
                        ast::Value::Block(_) => {
                            if ass.value.range.start_line > ass.operator_range.end_line {
                                fixes.push((ast::Range {
                                    start_line: ass.operator_range.end_line,
                                    start_col: ass.operator_range.end_col,
                                    end_line: ass.value.range.start_line,
                                    end_col: ass.value.range.start_col,
                                }, " ".to_string()));
                            }
                            self.collect_brace_newline_fixes(match &ass.value.value { ast::Value::Block(i) => i, _ => &[] }, fixes);
                        }
                        ast::Value::TaggedBlock(tag, inner, block_range) => {
                            if block_range.start_line > ass.operator_range.end_line {
                                fixes.push((ast::Range {
                                    start_line: ass.operator_range.end_line,
                                    start_col: ass.operator_range.end_col,
                                    end_line: block_range.start_line,
                                    end_col: block_range.start_col,
                                }, " ".to_string()));
                            } else {
                                let tag_end_col = ass.value.range.start_col + tag.len() as u32;
                                if block_range.start_col != tag_end_col + 1 {
                                    fixes.push((ast::Range {
                                        start_line: ass.value.range.start_line,
                                        start_col: tag_end_col,
                                        end_line: block_range.start_line,
                                        end_col: block_range.start_col,
                                    }, " ".to_string()));
                                }
                            }
                            self.collect_brace_newline_fixes(inner, fixes);
                        }
                        _ => {}
                    }
                }
                ast::Entry::Value(val) => {
                    match &val.value {
                        ast::Value::Block(inner) => self.collect_brace_newline_fixes(inner, fixes),
                        ast::Value::TaggedBlock(tag, inner, block_range) => {
                            if block_range.start_line > val.range.start_line {
                                fixes.push((ast::Range {
                                    start_line: val.range.start_line,
                                    start_col: val.range.start_col + tag.len() as u32,
                                    end_line: block_range.start_line,
                                    end_col: block_range.start_col,
                                }, " ".to_string()));
                            } else {
                                let tag_end_col = val.range.start_col + tag.len() as u32;
                                if block_range.start_col != tag_end_col + 1 {
                                    fixes.push((ast::Range {
                                        start_line: val.range.start_line,
                                        start_col: tag_end_col,
                                        end_line: block_range.start_line,
                                        end_col: block_range.start_col,
                                    }, " ".to_string()));
                                }
                            }
                            self.collect_brace_newline_fixes(inner, fixes);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    fn collect_brace_space_fixes(&self, entries: &[ast::Entry], fixes: &mut Vec<(ast::Range, String)>, content: &str) {
        for entry in entries {
            match entry {
                ast::Entry::Assignment(ass) => {
                    Self::check_and_fix_brace(&ass.value.range, &ass.value.value, content, fixes);
                    match &ass.value.value {
                        ast::Value::Block(inner) => self.collect_brace_space_fixes(inner, fixes, content),
                        ast::Value::TaggedBlock(_, inner, _) => self.collect_brace_space_fixes(inner, fixes, content),
                        _ => {}
                    }
                }
                ast::Entry::Value(val) => {
                    Self::check_and_fix_brace(&val.range, &val.value, content, fixes);
                    match &val.value {
                        ast::Value::Block(inner) => self.collect_brace_space_fixes(inner, fixes, content),
                        ast::Value::TaggedBlock(_, inner, _) => self.collect_brace_space_fixes(inner, fixes, content),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    fn check_and_fix_brace(range: &ast::Range, value: &ast::Value, content: &str, fixes: &mut Vec<(ast::Range, String)>) {
        match value {
            ast::Value::Block(_) | ast::Value::TaggedBlock(_, _, _) => {
                if range.start_line == range.end_line {
                    let line_idx = range.start_line as usize;
                    if let Some(line) = content.lines().nth(line_idx) {
                        let start = range.start_col as usize;
                        let end = range.end_col as usize;
                        if start < end && end <= line.len() {
                            let full_str = &line[start..end];
                            if let Some(brace_start_rel) = full_str.find('{') {
                                let block_str = &full_str[brace_start_rel..];
                                let mut needs_fix = false;

                                // 1. Check space BEFORE { if it's a TaggedBlock
                                if let ast::Value::TaggedBlock(tag, _, _) = value {
                                    if &full_str[tag.len()..brace_start_rel] != " " {
                                        needs_fix = true;
                                    }
                                }

                                // 2. Check padding INSIDE
                                if block_str.len() >= 2 {
                                    let inner = &block_str[1..block_str.len()-1];
                                    if inner.trim().is_empty() {
                                        if block_str != "{}" { needs_fix = true; }
                                    } else {
                                        if !block_str.starts_with("{ ") || !block_str.ends_with(" }") || block_str.starts_with("{  ") || block_str.ends_with("  }") {
                                            needs_fix = true;
                                        }
                                    }
                                }

                                if needs_fix {
                                    let brace_end_rel = full_str.rfind('}').unwrap_or(full_str.len() - 1);
                                    let inner = &full_str[brace_start_rel + 1 .. brace_end_rel];

                                    let before_brace = full_str[..brace_start_rel].trim();

                                    let new_text = if inner.trim().is_empty() {
                                        if !before_brace.is_empty() { format!("{} {{}}", before_brace) } else { "{}".to_string() }
                                    } else {
                                        if !before_brace.is_empty() { format!("{} {{ {} }}", before_brace, inner.trim()) } else { format!("{{ {} }}", inner.trim()) }
                                    };
                                    fixes.push((range.clone(), new_text));
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn collect_casing_fixes(&self, entries: &[ast::Entry], fixes: &mut Vec<(ast::Range, String)>) {        let keywords = [
            "spriteTypes", "spriteType", "name", "texturefile", 
            "ideologies", "types", "ideas", "country", "national_focus",
            "leader_traits", "country_leader_traits", "traits"
        ];

        for entry in entries {
            match entry {
                ast::Entry::Assignment(ass) => {
                    let key_lower = ass.key.to_lowercase();
                    for kw in keywords {
                        if key_lower == kw.to_lowercase() && ass.key != kw {
                            fixes.push((ass.key_range.clone(), kw.to_string()));
                            break;
                        }
                    }

                    match &ass.value.value {
                        ast::Value::Block(inner) => self.collect_casing_fixes(inner, fixes),
                        ast::Value::TaggedBlock(_, inner, _) => self.collect_casing_fixes(inner, fixes),
                        _ => {}
                    }
                }
                ast::Entry::Value(val) => {
                    match &val.value {
                        ast::Value::Block(inner) => self.collect_casing_fixes(inner, fixes),
                        ast::Value::TaggedBlock(_, inner, _) => self.collect_casing_fixes(inner, fixes),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    async fn load_localization(&self, roots: &[std::path::PathBuf]) {
        let mut all_locs = HashMap::new();

        self.client.log_message(MessageType::INFO, format!("Starting localization scan in {} roots", roots.len())).await;

        for root in roots {
            let loc_dir = root.join("localisation");
            self.client.log_message(MessageType::INFO, format!("Checking for localization in: {:?}", loc_dir)).await;

            if !loc_dir.exists() {
                self.client.log_message(MessageType::INFO, format!("Directory does not exist: {:?}", loc_dir)).await;
                continue;
            }

            let mut files_to_scan = Vec::new();
            let mut dirs_to_check = vec![loc_dir.clone()];

            while let Some(current_dir) = dirs_to_check.pop() {
                if let Ok(entries) = std::fs::read_dir(current_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            dirs_to_check.push(path);
                        } else if path.extension().map_or(false, |ext| ext == "yml") {
                            // By default, prefer english localization. Ignore other languages to prevent overwriting keys
                            // with translated versions (e.g. Chinese or Russian)
                            let path_str = path.to_string_lossy().to_lowercase();
                            if path_str.contains("english") {
                                files_to_scan.push(path);
                            }
                        }
                    }
                }
            }

            self.client.log_message(MessageType::INFO, format!("Found {} english .yml files in {:?}", files_to_scan.len(), loc_dir)).await;

            for path in files_to_scan {
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        let path_str = path.to_string_lossy().to_string();
                        let (parsed, _) = loc_parser::parse_loc_file(&content, &path_str);
                        if parsed.is_empty() {
                            self.client.log_message(MessageType::LOG, format!("Warning: No keys found in localization file: {:?}", path)).await;
                        } else {
                            self.client.log_message(MessageType::LOG, format!("Loaded {} keys from {:?}", parsed.len(), path)).await;
                        }
                        all_locs.extend(parsed);
                    }
                    Err(e) => {
                        self.client.log_message(MessageType::ERROR, format!("Failed to read localization file {:?}: {}", path, e)).await;
                    }
                }
            }
        }
        let mut loc = self.localization.write().await;
        *loc = all_locs;
        self.client.log_message(MessageType::INFO, format!("Total: Loaded {} localization keys", loc.len())).await;
    }

    async fn scan_scripted(&self, roots: &[std::path::PathBuf]) {
        let mut all_triggers = HashMap::new();
        let mut all_effects = HashMap::new();

        for root in roots {
            let triggers_dir = root.join("common/scripted_triggers");
            let effects_dir = root.join("common/scripted_effects");

            if triggers_dir.exists() {
                let found = scripted_scanner::scan_directory(&triggers_dir);
                self.client.log_message(MessageType::LOG, format!("Loaded {} scripted triggers from {:?}", found.len(), triggers_dir)).await;
                all_triggers.extend(found);
            }
            if effects_dir.exists() {
                let found = scripted_scanner::scan_directory(&effects_dir);
                self.client.log_message(MessageType::LOG, format!("Loaded {} scripted effects from {:?}", found.len(), effects_dir)).await;
                all_effects.extend(found);
            }
        }

        let mut t_map = self.scripted_triggers.write().await;
        *t_map = all_triggers;

        let mut e_map = self.scripted_effects.write().await;
        *e_map = all_effects;

        self.client.log_message(MessageType::INFO, format!("Total: Loaded {} scripted triggers and {} scripted effects", t_map.len(), e_map.len())).await;
    }

    async fn scan_ideologies(&self, roots: &[std::path::PathBuf]) {
        let mut all_results = HashMap::new();
        let mut sub_map = HashMap::new();

        for root in roots {
            let dir = root.join("common/ideologies");
            if dir.exists() {
                let results = ideology_scanner::scan_ideologies(&dir);
                let mut sub_count = 0;
                for ideology in results.values() {
                    for (sub, range) in &ideology.sub_ideology_ranges {
                        sub_count += 1;
                        sub_map.insert(sub.clone(), (ideology.name.clone(), range.clone(), ideology.path.clone()));
                    }
                }
                self.client.log_message(MessageType::LOG, format!("Loaded {} ideologies and {} sub-ideologies from {:?}", results.len(), sub_count, dir)).await;
                all_results.extend(results);
            }
        }

        let mut i_map = self.ideologies.write().await;
        *i_map = all_results;

        let mut s_map = self.sub_ideologies.write().await;
        *s_map = sub_map;

        self.client.log_message(MessageType::INFO, format!("Total: Loaded {} ideologies and {} sub-ideologies", i_map.len(), s_map.len())).await;
    }

    async fn scan_traits(&self, roots: &[std::path::PathBuf]) {
        let mut all_traits = HashMap::new();

        for root in roots {
            let unit_leader_dir = root.join("common/unit_leader");
            if unit_leader_dir.exists() {
                let found = trait_scanner::scan_traits(&unit_leader_dir, "Unit Leader Trait");
                self.client.log_message(MessageType::LOG, format!("Loaded {} unit leader traits from {:?}", found.len(), unit_leader_dir)).await;
                all_traits.extend(found);
            }

            let country_leader_dir = root.join("common/country_leader");
            if country_leader_dir.exists() {
                let found = trait_scanner::scan_traits(&country_leader_dir, "Country Leader Trait");
                self.client.log_message(MessageType::LOG, format!("Loaded {} country leader traits from {:?}", found.len(), country_leader_dir)).await;
                all_traits.extend(found);
            }

            let trait_dir = root.join("common/traits");
            if trait_dir.exists() {
                let found = trait_scanner::scan_traits(&trait_dir, "Trait");
                self.client.log_message(MessageType::LOG, format!("Loaded {} general traits from {:?}", found.len(), trait_dir)).await;
                all_traits.extend(found);
            }
        }

        let mut t_map = self.traits.write().await;
        *t_map = all_traits;

        self.client.log_message(MessageType::INFO, format!("Total: Loaded {} traits", t_map.len())).await;
    }

    async fn scan_sprites(&self, roots: &[std::path::PathBuf]) {
        let mut all_sprites = HashMap::new();

        for root in roots {
            let interface_dir = root.join("interface");
            if !interface_dir.exists() {
                self.client.log_message(MessageType::LOG, format!("Directory does not exist: {:?}", interface_dir)).await;
                continue;
            }
            let found = sprite_scanner::scan_sprites(&interface_dir);
            self.client.log_message(MessageType::LOG, format!("Loaded {} sprite definitions from {:?}", found.len(), interface_dir)).await;
            all_sprites.extend(found);
        }

        let mut s_map = self.sprites.write().await;
        *s_map = all_sprites;

        self.client.log_message(MessageType::INFO, format!("Total: Loaded {} sprite definitions", s_map.len())).await;
    }

    async fn scan_ideas(&self, roots: &[std::path::PathBuf]) {
        let mut all_ideas = HashMap::new();

        for root in roots {
            let ideas_dir = root.join("common/ideas");
            if ideas_dir.exists() {
                let found = idea_scanner::scan_ideas(&ideas_dir);
                self.client.log_message(MessageType::LOG, format!("Loaded {} ideas from {:?}", found.len(), ideas_dir)).await;
                all_ideas.extend(found);
            }
        }

        let mut i_map = self.ideas.write().await;
        *i_map = all_ideas;

        self.client.log_message(MessageType::INFO, format!("Total: Loaded {} ideas", i_map.len())).await;
    }

    async fn load_schema(&self) {
        let mut schema = schema::Schema::new();

        // Resolve paths relative to executable (production) or CWD (development)
        let exe_path = std::env::current_exe().unwrap_or_default();
        let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));

        let possible_roots = vec![
            std::path::PathBuf::from("."),
            exe_dir.to_path_buf(),
            exe_dir.join(".."), // Handle bin/server case
        ];

        let mut triggers_path = None;
        let mut effects_path = None;
        let mut links_path = None;

        for root in &possible_roots {
            let t = root.join("Config/triggers.cwt");
            let e = root.join("Config/effects.cwt");
            let l = root.join("Config/links.cwt");
            if t.exists() { triggers_path = Some(t); }
            if e.exists() { effects_path = Some(e); }
            if l.exists() { links_path = Some(l); }
        }

        if let Some(path) = triggers_path {
            if let Ok(content) = std::fs::read_to_string(&path) {
                schema.parse_cwt(&content, true);
            }
        }

        if let Some(path) = effects_path {
            if let Ok(content) = std::fs::read_to_string(&path) {
                schema.parse_cwt(&content, false);
            }
        }

        if let Some(path) = links_path {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(parsed) = parser::parse_script(&content) {
                    schema.parse_links(&parsed);
                }
            }
        }

        self.client.log_message(MessageType::INFO, format!("Schema loaded: {} triggers, {} effects, {} links", 
            schema.triggers.len(), schema.effects.len(), schema.links.len())).await;

        let mut s = self.schema.write().await;
        *s = schema;

        // Resolve assets
        let mut mapping_path = None;
        let mut formats_path = None;

        for root in &possible_roots {
            let m = root.join("assets/modifier_mappings.json");
            let f = root.join("assets/modifier_formats.json");
            if m.exists() { mapping_path = Some(m); }
            if f.exists() { formats_path = Some(f); }
        }

        if let Some(path) = mapping_path {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(mappings) = serde_json::from_str::<HashMap<String, String>>(&content) {
                    let mut m = self.modifier_mappings.write().await;
                    for (k, v) in mappings {
                        m.insert(k, v);
                    }
                    self.client.log_message(MessageType::INFO, format!("Loaded {} modifier mappings from assets", m.len())).await;
                }
            }
        }

        if let Some(path) = formats_path {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(formats) = serde_json::from_str::<HashMap<String, String>>(&content) {
                    let mut f = self.modifier_formats.write().await;
                    for (k, v) in formats {
                        f.insert(k, v);
                    }
                    self.client.log_message(MessageType::INFO, format!("Loaded {} modifier formats from assets", f.len())).await;
                }
            }
        }
    }

    async fn find_references_in_root(&self, root: &std::path::Path, identifier: &str, locations: &mut Vec<Location>) {
        let mut dirs_to_check = vec![root.to_path_buf()];
        let extensions = ["txt", "yml", "gfx", "gui", "asset"];

        while let Some(current_dir) = dirs_to_check.pop() {
            if let Ok(entries) = std::fs::read_dir(current_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        dirs_to_check.push(path);
                    } else if path.extension().map_or(false, |ext| extensions.contains(&ext.to_string_lossy().as_ref())) {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if content.contains(identifier) {
                                // Find all occurrences with word boundaries
                                for (line_idx, line) in content.lines().enumerate() {
                                    let mut start_search = 0;
                                    while let Some(pos) = line[start_search..].find(identifier) {
                                        let actual_pos = start_search + pos;

                                        // Check word boundaries
                                        let before = if actual_pos > 0 { line.chars().nth(actual_pos - 1) } else { None };
                                        let after = line.chars().nth(actual_pos + identifier.len());

                                        let is_word_start = before.map_or(true, |c| !parser::is_identifier_char(c));
                                        let is_word_end = after.map_or(true, |c| !parser::is_identifier_char(c));

                                        if is_word_start && is_word_end {
                                            locations.push(Location {
                                                uri: Url::from_file_path(path.canonicalize().unwrap_or_else(|_| path.clone())).unwrap(),
                                                range: Range {
                                                    start: Position { line: line_idx as u32, character: actual_pos as u32 },
                                                    end: Position { line: line_idx as u32, character: (actual_pos + identifier.len()) as u32 },
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

    async fn validate_document(&self, uri: Url) {
        let content = self.documents.get(uri.as_str()).unwrap();
        let mut diagnostics = Vec::new();

        let styling_enabled = *self.styling_enabled.read().await;
        let mut script_opt = None;

        if uri.as_str().ends_with(".yml") {
            self.validate_localization_content(&content, &mut diagnostics).await;
        } else {
            match parser::parse_script(&content) {
                Ok(script) => {
                    // Semantic validation
                    self.check_semantic(&script, &mut diagnostics, styling_enabled).await;
                    script_opt = Some(script);
                }
                Err((msg, range)) => {
                    diagnostics.push(Diagnostic {
                        range: ast_range_to_lsp(&range),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: msg,
                        ..Default::default()
                    });
                }
            };
        }

        if styling_enabled {
            self.check_styling(&content, script_opt.as_ref(), &mut diagnostics);
        }

        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }

    async fn validate_localization_content(&self, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        let (parsed, loc_diagnostics_structural) = loc_parser::parse_loc_file(content, "");
        let event_targets = self.event_targets.read().await;

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
                ..Default::default()
            });
        }

        for entry in parsed.values() {
            let loc_diagnostics = loc_parser::validate_loc_string(entry, &event_targets);
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
                    ..Default::default()
                });
            }
        }
    }

    fn compute_expected_indentations(entries: &[ast::Entry], depth: usize, expected: &mut HashMap<u32, usize>) {
        for entry in entries {
            let start_line = match entry {
                ast::Entry::Assignment(ass) => ass.key_range.start_line,
                ast::Entry::Value(val) => val.range.start_line,
                ast::Entry::Comment(_, r) => r.start_line,
            };

            expected.entry(start_line).or_insert(depth);

            match entry {
                ast::Entry::Assignment(ass) => {
                    match &ass.value.value {
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
                    }
                }
                ast::Entry::Value(val) => {
                    match &val.value {
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
                    }
                }
                ast::Entry::Comment(_, _) => {}
            }
        }
    }

    fn check_single_line_braces(entries: &[ast::Entry], content: &str, diagnostics: &mut Vec<Diagnostic>) {
        for entry in entries {
            match entry {
                ast::Entry::Assignment(ass) => {
                    Self::check_brace_spacing_for_range(&ass.value.range, &ass.value.value, content, diagnostics);
                    match &ass.value.value {
                        ast::Value::Block(inner) => Self::check_single_line_braces(inner, content, diagnostics),
                        ast::Value::TaggedBlock(_, inner, _) => Self::check_single_line_braces(inner, content, diagnostics),
                        _ => {}
                    }
                }
                ast::Entry::Value(val) => {
                    Self::check_brace_spacing_for_range(&val.range, &val.value, content, diagnostics);
                    match &val.value {
                        ast::Value::Block(inner) => Self::check_single_line_braces(inner, content, diagnostics),
                        ast::Value::TaggedBlock(_, inner, _) => Self::check_single_line_braces(inner, content, diagnostics),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    fn check_brace_spacing_for_range(range: &ast::Range, value: &ast::Value, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        match value {
            ast::Value::Block(_) | ast::Value::TaggedBlock(_, _, _) => {
                if range.start_line == range.end_line {
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
                                    let inner = &block_str[1..block_str.len()-1];
                                    if inner.trim().is_empty() {
                                        if block_str != "{}" {
                                            needs_fix = true;
                                            message = "Empty single-line block should be '{}' without spaces.";
                                        }
                                    } else {
                                        if !block_str.starts_with("{ ") || !block_str.ends_with(" }") || block_str.starts_with("{  ") || block_str.ends_with("  }") {
                                            needs_fix = true;
                                        }
                                    }
                                }

                                if needs_fix {
                                    diagnostics.push(Diagnostic {
                                        range: ast_range_to_lsp(range),
                                        severity: Some(DiagnosticSeverity::INFORMATION),
                                        code: Some(NumberOrString::String("styling_brace_space".to_string())),
                                        message: message.to_string(),
                                        source: Some("Hearts of Modding".to_string()),
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

    fn check_styling(&self, content: &str, script_opt: Option<&ast::Script>, diagnostics: &mut Vec<Diagnostic>) {
        if !content.is_empty() && !content.ends_with('\n') && !content.ends_with("\r\n") {
            let line_count = content.lines().count();
            let last_line = content.lines().last().unwrap_or("");
            let line_idx = if line_count > 0 { line_count as u32 - 1 } else { 0 };
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position { line: line_idx, character: last_line.len() as u32 },
                    end: Position { line: line_idx, character: last_line.len() as u32 },
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
            // 1. Trailing whitespace
            if line.ends_with(' ') || line.ends_with('\t') {
                let start_col = line.trim_end().len() as u32;
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: line_idx as u32, character: start_col },
                        end: Position { line: line_idx as u32, character: line.len() as u32 },
                    },
                    severity: Some(DiagnosticSeverity::INFORMATION),
                    code: Some(NumberOrString::String("styling_trailing".to_string())),
                    message: "Trailing whitespace detected.".to_string(),
                    source: Some("Hearts of Modding".to_string()),
                    ..Default::default()
                });
            }

            // 2. Indentation consistency
            let leading = line.chars().take_while(|c| c.is_whitespace()).collect::<String>();
            if line.trim().is_empty() {
                continue; // Skip empty lines for indentation checking
            }

            if let Some(&expected_tabs) = expected_indents.get(&(line_idx as u32)) {
                let expected_str = "\t".repeat(expected_tabs);
                if leading != expected_str {
                    let mut data = serde_json::Map::new();
                    data.insert("expected_tabs".to_string(), serde_json::Value::Number(expected_tabs.into()));

                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: line_idx as u32, character: 0 },
                            end: Position { line: line_idx as u32, character: leading.len() as u32 },
                        },
                        severity: Some(DiagnosticSeverity::INFORMATION),
                        code: Some(NumberOrString::String("styling_indent".to_string())),
                        message: format!("Inconsistent indentation. Expected {} tab(s).", expected_tabs),
                        source: Some("Hearts of Modding".to_string()),
                        data: Some(serde_json::Value::Object(data)),
                        ..Default::default()
                    });
                }
            } else if leading.contains(' ') {
                // Fallback if line wasn't in AST (e.g. unparsed strings or comments)
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: line_idx as u32, character: 0 },
                        end: Position { line: line_idx as u32, character: leading.len() as u32 },
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

    async fn check_semantic(&self, script: &ast::Script, diagnostics: &mut Vec<Diagnostic>, styling_enabled: bool) {
        let loc = self.localization.read().await;
        let st = self.scripted_triggers.read().await;
        let se = self.scripted_effects.read().await;
        let id = self.ideologies.read().await;
        let sid = self.sub_ideologies.read().await;
        let tr = self.traits.read().await;
        let sp = self.sprites.read().await;
        let ids = self.ideas.read().await;
        let provs = self.provinces.read().await;
        let schema = self.schema.read().await;
        let mod_maps = self.modifier_mappings.read().await;
        let ig_loc = self.ignored_loc_regex.read().await;
        let buildings = self.buildings.read().await;
        let defines = self.defines.read().await;

        let mut comments = Vec::new();
        for entry in &script.entries {
            if let ast::Entry::Comment(c, r) = entry {
                comments.push((c.clone(), r.clone()));
            }
        }

        let mut scope_stack = scope::ScopeStack::new(scope::Scope::Global);

        // Run advanced validations
        let mut advanced_diags = Vec::new();
        advanced_validation::validate_building_levels(&script.entries, &buildings, &mut advanced_diags);
        advanced_validation::validate_character_skills(&script.entries, &defines, &mut advanced_diags);
        advanced_validation::validate_victory_points(&script.entries, &mut advanced_diags);

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
                ..Default::default()
            });
        }

        for entry in &script.entries {
            self.check_entry_semantic(entry, diagnostics, &loc, &st, &se, &id, &sid, &tr, &sp, &ids, &provs, &schema, &mod_maps, &ig_loc, &comments, styling_enabled, &mut scope_stack);
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
        provs: &HashSet<u32>,
        schema: &schema::Schema,
        mod_maps: &HashMap<String, String>,
        ig_loc: &[regex::Regex],
        comments: &[ (String, ast::Range) ],
        styling_enabled: bool,
        scope_stack: &mut scope::ScopeStack,
    ) {
        match entry {
            ast::Entry::Assignment(ass) => {
                let key_lower = ass.key.to_lowercase();
                let mut pushed_scope = false;

                // Schema validation for triggers, effects, and links
                if let Some(rule) = schema.triggers.get(&ass.key).or_else(|| schema.effects.get(&ass.key)).or_else(|| schema.links.get(&ass.key)) {
                    if let Some(push) = &rule.push_scope {
                        scope_stack.push(scope::Scope::from_str(push));
                        pushed_scope = true;
                    }

                    if !rule.scopes.is_empty() {
                        let current_scope = scope_stack.current();
                        let current_str = current_scope.as_str().to_lowercase();
                        let mut valid = false;
                        for s in &rule.scopes {
                            if s.to_lowercase() == "any" || s.to_lowercase() == "all" || s.to_lowercase() == current_str || current_scope == scope::Scope::Unknown || current_scope == scope::Scope::Global {
                                valid = true;
                                break;
                            }
                        }
                        if !valid {
                            diagnostics.push(Diagnostic {
                                range: ast_range_to_lsp(&ass.key_range),
                                severity: Some(DiagnosticSeverity::WARNING),
                                message: format!("Invalid scope. '{}' is not supported in {:?} scope. Supported scopes: {:?}", ass.key, current_scope, rule.scopes),
                                ..Default::default()
                            });
                        }
                    }

                    // Type checking
                    match rule.value_type {
                        schema::ValueType::Bool => {
                            match &ass.value.value {
                                ast::Value::Boolean(_) => {},
                                ast::Value::String(s) if s == "yes" || s == "no" => {},
                                _ => {
                                    diagnostics.push(Diagnostic {
                                        range: ast_range_to_lsp(&ass.value.range),
                                        severity: Some(DiagnosticSeverity::ERROR),
                                        message: format!("Expected boolean (yes/no) for '{}'", ass.key),
                                        ..Default::default()
                                    });
                                }
                            }
                        },
                        schema::ValueType::Int => {
                            if let ast::Value::Number(_) = &ass.value.value {
                            } else if let ast::Value::String(s) = &ass.value.value {
                                if s.parse::<i64>().is_err() {
                                    diagnostics.push(Diagnostic {
                                        range: ast_range_to_lsp(&ass.value.range),
                                        severity: Some(DiagnosticSeverity::ERROR),
                                        message: format!("Expected integer for '{}'", ass.key),
                                        ..Default::default()
                                    });
                                }
                            } else {
                                diagnostics.push(Diagnostic {
                                    range: ast_range_to_lsp(&ass.value.range),
                                    severity: Some(DiagnosticSeverity::ERROR),
                                    message: format!("Expected integer for '{}'", ass.key),
                                    ..Default::default()
                                });
                            }
                        },
                        _ => {}
                    }
                } else {
                    // Structural blocks that push scope but aren't in the schema
                    let s = scope::Scope::from_str(&ass.key);
                    if s != scope::Scope::Unknown || ass.key.contains(':') || ass.key.contains('.') {
                        match &ass.value.value {
                            ast::Value::Block(_) | ast::Value::TaggedBlock(_, _, _) => {
                                scope_stack.push(s);
                                pushed_scope = true;
                            }
                            _ => {}
                        }
                    }
                }

                // Casing checks for keywords
                if styling_enabled {
                    let mut needs_fix = false;
                    if ass.key_range.end_line == ass.operator_range.start_line && ass.key_range.end_line == ass.value.range.start_line {
                        if ass.operator_range.start_col > ass.key_range.end_col && ass.value.range.start_col > ass.operator_range.end_col {
                            let space_before = ass.operator_range.start_col - ass.key_range.end_col;
                            let space_after = ass.value.range.start_col - ass.operator_range.end_col;
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
                        ast::Value::Block(_) => {
                            if ass.value.range.start_line > ass.operator_range.end_line {
                                diagnostics.push(Diagnostic {
                                    range: Range {
                                        start: Position { line: ass.operator_range.end_line, character: ass.operator_range.end_col },
                                        end: Position { line: ass.value.range.start_line, character: ass.value.range.start_col },
                                    },
                                    severity: Some(DiagnosticSeverity::INFORMATION),
                                    code: Some(NumberOrString::String("styling_brace_newline".to_string())),
                                    message: "Curly brace should not be on a new line.".to_string(),
                                    source: Some("Hearts of Modding".to_string()),
                                    ..Default::default()
                                });
                            }
                        }
                        ast::Value::TaggedBlock(tag, _, block_range) => {
                            // Check if the brace part of the tagged block is on a new line
                            // Usually TaggedBlock range starts at the tag.
                            // We check if the block_range starts on a new line compared to where the tag/operator is.
                            if block_range.start_line > ass.operator_range.end_line {
                                diagnostics.push(Diagnostic {
                                    range: Range {
                                        start: Position { line: ass.operator_range.end_line, character: ass.operator_range.end_col },
                                        end: Position { line: block_range.start_line, character: block_range.start_col },
                                    },
                                    severity: Some(DiagnosticSeverity::INFORMATION),
                                    code: Some(NumberOrString::String("styling_brace_newline".to_string())),
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
                        "spriteTypes", "spriteType", "name", "texturefile", 
                        "ideologies", "types", "ideas", "country", "national_focus",
                        "leader_traits", "country_leader_traits", "traits"
                    ];

                    for kw in keywords {
                        if key_lower == kw.to_lowercase() && ass.key != kw {
                            let mut message = format!("Standard Paradox Script convention uses '{}' instead of '{}'.", kw, ass.key);
                            if kw.to_lowercase().contains("sprite") || kw == "texturefile" {
                                message.push_str("\nReference: https://hoi4.paradoxwikis.com/Modding#GFX");
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
                if key_lower == "name" || key_lower == "desc" || key_lower == "text" || key_lower == "title" {
                    if let ast::Value::String(val) = &ass.value.value {
                        let mut should_flag = true;

                        // 1. Basic heuristics (Space, numbers)
                        if val.contains(' ') || val.is_empty() || val.chars().all(|c| c.is_numeric()) {
                            should_flag = false;
                        }

                        // 2. Casing heuristic: If it starts with a capital and isn't all caps, it's likely a literal
                        if should_flag && val.chars().next().map_or(false, |c| c.is_uppercase()) {
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
                                            message: format!("Missing localization key: '{}' (or literal name)", val),
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
                        if !id.contains_key(val) && !sid.contains_key(val) {
                            diagnostics.push(Diagnostic {
                                range: ast_range_to_lsp(&ass.value.range),
                                severity: Some(DiagnosticSeverity::WARNING),
                                message: format!("Unknown ideology: '{}'", val),
                                ..Default::default()
                            });
                        }
                    }
                }

                // Trait checks
                if key_lower == "add_trait" || key_lower == "has_trait" || key_lower == "remove_trait" {
                    if let ast::Value::String(val) = &ass.value.value {
                        if !tr.contains_key(val) {
                            diagnostics.push(Diagnostic {
                                range: ast_range_to_lsp(&ass.value.range),
                                severity: Some(DiagnosticSeverity::WARNING),
                                message: format!("Unknown trait: '{}'", val),
                                ..Default::default()
                            });
                        }
                    }
                }

                // Sprite/Gfx checks
                if key_lower == "sprite" || key_lower == "icon" || key_lower == "sprite_name" || key_lower == "picture" {
                    if let ast::Value::String(val) = &ass.value.value {
                        if !sp.contains_key(val) && val.starts_with("GFX_") {
                            diagnostics.push(Diagnostic {
                                range: ast_range_to_lsp(&ass.value.range),
                                severity: Some(DiagnosticSeverity::WARNING),
                                message: format!("Unknown sprite/GFX: '{}'", val),
                                ..Default::default()
                            });
                        }
                    }
                }

                // Idea checks
                if key_lower == "add_ideas" || key_lower == "has_idea" || key_lower == "remove_ideas" {
                    if let ast::Value::String(val) = &ass.value.value {
                        if !ids.contains_key(val) {
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
                if key_lower == "province" || key_lower == "on_province" || key_lower == "is_province_id" {
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
                self.check_value_semantic(&ass.value, diagnostics, loc, st, se, id, sid, tr, sp, ids, provs, schema, mod_maps, ig_loc, comments, styling_enabled, scope_stack);

                if pushed_scope {
                    scope_stack.pop();
                }
            }
            ast::Entry::Value(val) => {
                self.check_value_semantic(val, diagnostics, loc, st, se, id, sid, tr, sp, ids, provs, schema, mod_maps, ig_loc, comments, styling_enabled, scope_stack);
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
        provs: &HashSet<u32>,
        schema: &schema::Schema,
        mod_maps: &HashMap<String, String>,
        ig_loc: &[regex::Regex],
        comments: &[ (String, ast::Range) ],
        styling_enabled: bool,
        scope_stack: &mut scope::ScopeStack,
    ) {
        match &val.value {
            ast::Value::Block(entries) => {
                self.check_duplicate_keys(entries, diagnostics, schema, mod_maps);
                for entry in entries {
                    self.check_entry_semantic(entry, diagnostics, loc, st, se, id, sid, tr, sp, ids, provs, schema, mod_maps, ig_loc, comments, styling_enabled, scope_stack);
                }
            }
            ast::Value::TaggedBlock(tag, entries, block_range) => {
                if styling_enabled {
                    if block_range.start_line > val.range.start_line {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position { line: val.range.start_line, character: val.range.start_col + tag.len() as u32 },
                                end: Position { line: block_range.start_line, character: block_range.start_col },
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
                                    start: Position { line: val.range.start_line, character: tag_end_col },
                                    end: Position { line: block_range.start_line, character: block_range.start_col },
                                },
                                severity: Some(DiagnosticSeverity::INFORMATION),
                                code: Some(NumberOrString::String("styling_brace_newline".to_string())),
                                message: "Exactly one space should separate the tag and the curly brace.".to_string(),
                                source: Some("Hearts of Modding".to_string()),
                                ..Default::default()
                            });
                        }
                    }
                }
                self.check_duplicate_keys(entries, diagnostics, schema, mod_maps);
                for entry in entries {
                    self.check_entry_semantic(entry, diagnostics, loc, st, se, id, sid, tr, sp, ids, provs, schema, mod_maps, ig_loc, comments, styling_enabled, scope_stack);
                }
            }
            _ => {}
        }
    }

    fn check_duplicate_keys(&self, entries: &[ast::Entry], diagnostics: &mut Vec<Diagnostic>, schema: &schema::Schema, mod_maps: &HashMap<String, String>) {
        let mut seen_keys: HashMap<String, ast::Range> = HashMap::new();

        for entry in entries {
            if let ast::Entry::Assignment(ass) = entry {
                // We only care about duplicates if they are modifiers. 
                // Some Paradox keys (like 'modifier = { ... }' or 'option = { ... }') are intended to be duplicates.
                // But specific engine modifiers (like 'stability_factor') should NEVER be duplicated.

                let is_modifier = mod_maps.contains_key(&ass.key) ||
                                 schema.triggers.contains_key(&ass.key) ||
                                 schema.effects.contains_key(&ass.key);

                // Exceptions: Some effects/triggers are specifically designed to be used multiple times
                let is_exception = ass.key == "modifier" || ass.key == "option" || ass.key == "limit" || ass.key == "if" || ass.key == "else" || ass.key == "else_if" || ass.key == "variable_name";

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
        localization: Arc::new(RwLock::new(HashMap::new())),
        scripted_triggers: Arc::new(RwLock::new(HashMap::new())),
        scripted_effects: Arc::new(RwLock::new(HashMap::new())),
        ideologies: Arc::new(RwLock::new(HashMap::new())),
        sub_ideologies: Arc::new(RwLock::new(HashMap::new())),
        traits: Arc::new(RwLock::new(HashMap::new())),
        sprites: Arc::new(RwLock::new(HashMap::new())),
        ideas: Arc::new(RwLock::new(HashMap::new())),
        variables: Arc::new(RwLock::new(HashMap::new())),
        event_targets: Arc::new(RwLock::new(HashMap::new())),
        provinces: Arc::new(RwLock::new(HashSet::new())),
        custom_modifiers: Arc::new(RwLock::new(HashMap::new())),
        modifier_mappings: Arc::new(RwLock::new(HashMap::new())),
        modifier_formats: Arc::new(RwLock::new(HashMap::new())),
        events: Arc::new(RwLock::new(HashMap::new())),
        music_assets: Arc::new(RwLock::new(HashMap::new())),
        music_stations: Arc::new(RwLock::new(HashMap::new())),
        songs: Arc::new(RwLock::new(HashMap::new())),
        sounds: Arc::new(RwLock::new(HashMap::new())),
        sound_effects: Arc::new(RwLock::new(HashMap::new())),
        falloffs: Arc::new(RwLock::new(HashMap::new())),
        sound_categories: Arc::new(RwLock::new(HashMap::new())),
        buildings: Arc::new(RwLock::new(HashMap::new())),
        defines: Arc::new(RwLock::new(defines_parser::GameDefines::new())),
        ignored_loc_regex: Arc::new(RwLock::new(Vec::new())),
        schema: Arc::new(RwLock::new(schema::Schema::new())),
        styling_enabled: Arc::new(RwLock::new(true)),
        cosmetic_loc_indent: Arc::new(RwLock::new(false)),
        game_path: Arc::new(RwLock::new(None)),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}

fn find_colors(script: &ast::Script) -> Vec<ColorInformation> {
    let mut colors = Vec::new();
    for entry in &script.entries {
        find_colors_in_entry(entry, &mut colors);
    }
    colors
}

fn find_colors_in_entry(entry: &ast::Entry, colors: &mut Vec<ColorInformation>) {
    if let ast::Entry::Assignment(ass) = entry {
        let key_lower = ass.key.to_lowercase();
        if key_lower.contains("color") {
            find_colors_in_value(&ass.value, colors, true);
        } else {
            // Recurse into blocks even if key doesn't match, but don't treat as color context
            find_colors_in_value(&ass.value, colors, false);
        }
    } else if let ast::Entry::Value(val) = entry {
        find_colors_in_value(val, colors, false);
    }
}

fn find_colors_in_value(val: &ast::NodeedValue, colors: &mut Vec<ColorInformation>, is_color_context: bool) {
    match &val.value {
        ast::Value::Block(entries) => {
            let nums: Vec<f64> = entries.iter().filter_map(|e| {
                if let ast::Entry::Value(v) = e {
                    match &v.value {
                        ast::Value::Number(n) => Some(*n),
                        ast::Value::String(s) => s.parse::<f64>().ok(),
                        _ => None
                    }
                } else {
                    None
                }
            }).collect();

            // Only treat as color if we're in a color context (key contains "color")
            if nums.len() == 3 && is_color_context {
                // Determine if it's 0-1 or 0-255
                // Most HOI4 color blocks are 0-255, but some might be 0-1
                // If any value is > 1.0, it must be 0-255
                let is_255 = nums.iter().any(|&n| n > 1.0);

                let (r, g, b) = if is_255 {
                    ((nums[0] / 255.0) as f32, (nums[1] / 255.0) as f32, (nums[2] / 255.0) as f32)
                } else {
                    (nums[0] as f32, nums[1] as f32, nums[2] as f32)
                };

                colors.push(ColorInformation {
                    range: ast_range_to_lsp(&val.range),
                    color: Color { red: r, green: g, blue: b, alpha: 1.0 },
                });
            } else {
                for e in entries {
                    find_colors_in_entry(e, colors);
                }
            }
        }
        ast::Value::TaggedBlock(tag, entries, _) => {
            let nums: Vec<f64> = entries.iter().filter_map(|e| {
                if let ast::Entry::Value(v) = e {
                    match &v.value {
                        ast::Value::Number(n) => Some(*n),
                        ast::Value::String(s) => s.parse::<f64>().ok(),
                        _ => None
                    }
                } else {
                    None
                }
            }).collect();

            if nums.len() == 3 {
                let tag_lower = tag.to_lowercase();
                // Tagged blocks (rgb/hsv) are always colors regardless of context
                if tag_lower == "rgb" {
                    let r = (nums[0] / 255.0) as f32;
                    let g = (nums[1] / 255.0) as f32;
                    let b = (nums[2] / 255.0) as f32;
                    colors.push(ColorInformation {
                        range: ast_range_to_lsp(&val.range),
                        color: Color { red: r, green: g, blue: b, alpha: 1.0 },
                    });
                } else if tag_lower == "hsv" {
                    // Convert HSV to RGB
                    let (r, g, b) = hsv_to_rgb(nums[0], nums[1], nums[2]);
                    colors.push(ColorInformation {
                        range: ast_range_to_lsp(&val.range),
                        color: Color { red: r as f32, green: g as f32, blue: b as f32, alpha: 1.0 },
                    });
                }
            } else {
                for e in entries {
                    find_colors_in_entry(e, colors);
                }
            }
        }
        _ => {}
    }
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    let angle = h * 360.0;
    let c = v * s;
    let x = c * (1.0 - ((angle / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r_prime, g_prime, b_prime) = if angle < 60.0 {
        (c, x, 0.0)
    } else if angle < 120.0 {
        (x, c, 0.0)
    } else if angle < 180.0 {
        (0.0, c, x)
    } else if angle < 240.0 {
        (0.0, x, c)
    } else if angle < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (r_prime + m, g_prime + m, b_prime + m)
}

#[allow(dead_code)]
fn rgb_to_hsv(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let h = if delta < 1e-6 {
        0.0
    } else if (max - r).abs() < 1e-6 {
        60.0 * (((g - b) / delta) % 6.0)
    } else if (max - g).abs() < 1e-6 {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };
    let s = if max < 1e-6 { 0.0 } else { delta / max };
    let v = max;

    (h / 360.0, s, v)
}

fn format_modifier_value(key: &str, val: f64, format_str: Option<&String>) -> String {
    let mut is_percentage = key.ends_with("factor");
    let mut display_digits = 1;
    let mut is_double_percent = false;

    if let Some(fmt) = format_str {
        if fmt.contains("%%") {
            is_double_percent = true;
            is_percentage = false;
        } else if fmt.contains('%') {
            is_percentage = true;
        } else {
            is_percentage = false;
        }

        for c in fmt.chars().rev() {
            if c.is_ascii_digit() {
                display_digits = c.to_digit(10).unwrap() as usize;
                break;
            }
        }
    }

    let mut actual_val = val;
    if is_percentage && !is_double_percent {
        actual_val *= 100.0;
    }

    let sign = if actual_val >= 0.0 { "+" } else { "" };

    // In Rust, format!("{:.0}", 1.0) is "1", but we want to mimic C# '1' format which means 1 decimal place.
    // If format string has `0`, it's 0 decimal places. If it has `1`, it's 1.
    // However, if the value is an exact integer, C# often drops the .0
    let mut formatted_num = format!("{}{:.*}", sign, display_digits, actual_val);

    if is_percentage || is_double_percent {
        formatted_num.push('%');
    }

    formatted_num
}
fn resolve_loc(input: &str, localization: &HashMap<String, loc_parser::LocEntry>, depth: u32) -> String {
    if depth > 10 { return input.to_string(); }
    let re_key = regex::Regex::new(r"\$([^\$]+)\$").unwrap();
    let mut last_end = 0;
    let mut result = String::new();

    for cap in re_key.captures_iter(input) {
        let m = cap.get(0).unwrap();
        let key = cap.get(1).unwrap().as_str();

        result.push_str(&input[last_end..m.start()]);
        if let Some(entry) = localization.get(key) {
            result.push_str(&resolve_loc(&entry.value, localization, depth + 1));
        } else {
            result.push_str(m.as_str());
        }
        last_end = m.end();
    }
    result.push_str(&input[last_end..]);
    result
}

fn paradox_to_markdown(input: &str, localization: Option<&HashMap<String, loc_parser::LocEntry>>) -> String {
    let mut resolved = if let Some(loc) = localization {
        resolve_loc(input, loc, 0)
    } else {
        input.to_string()
    };

    // Handle literal \n
    resolved = resolved.replace("\\n", "\n").replace("\\r\\n", "\n");

    let mut result = String::new();
    let re_color = regex::Regex::new(r"§([a-zA-Z0-9!])").unwrap();
    let mut last_end = 0;

    let mut segments = Vec::new();
    let mut current_color = "#FFFFFF"; // Default white

    for cap in re_color.captures_iter(&resolved) {
        let m = cap.get(0).unwrap();
        let code = cap.get(1).unwrap().as_str();

        let text_segment = &resolved[last_end..m.start()];
        if !text_segment.is_empty() {
            segments.push((text_segment.to_string(), current_color));
        }

        current_color = match code {
            "!" => "#FFFFFF",
            "C" => "#23CEFF", // Cyan
            "L" => "#C3B091", // Lilac
            "W" => "#FFFFFF", // White
            "B" => "#0000FF", // Blue
            "G" => "#009F03", // Green
            "R" => "#FF3232", // Red
            "b" => "#000000", // Black
            "g" => "#B0B0B0", // Light Gray
            "Y" | "H" => "#FFBD00", // Yellow / Header
            "T" => "#FFFFFF", // Title (White)
            "O" => "#FF7019", // Orange
            "0" => "#CB00CB", // Gradient 0 (Purple)
            "1" => "#8078D3", // Gradient 1 (Lilac)
            "2" => "#5170F3", // Gradient 2 (Blue)
            "3" => "#518FDC", // Gradient 3 (Gray-Blue)
            "4" => "#5ABEE7", // Gradient 4 (Light Blue)
            "5" => "#3FB5C2", // Gradient 5 (Dull Cyan)
            "6" => "#77CCBA", // Gradient 6 (Turquoise)
            "7" => "#99D199", // Gradient 7 (Light Green)
            "8" => "#CCA333", // Gradient 8 (Orange-Yellow)
            "9" => "#FCA97D", // Gradient 9 (White-Orange)
            "t" => "#FF4C4D", // Gradient 10 (Vivid Red)
            "M" => "#FF60FF", // Magenta (fallback)
            "p" => "#FF80FF", // Pink (fallback)
            _ => "#FFFFFF",
        };
        last_end = m.end();
    }

    let last_segment = &resolved[last_end..];
    if !last_segment.is_empty() {
        segments.push((last_segment.to_string(), current_color));
    }

    for (text, color) in segments {
        if text == "\n" {
            result.push_str("  \n");
            continue;
        }

        let lines: Vec<&str> = text.split('\n').collect();
        for (i, line) in lines.iter().enumerate() {
            if i > 0 { result.push_str("  \n"); }
            if line.is_empty() { continue; }

            let escaped_line = line.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&apos;").replace(' ', "&#160;");
            // Estimate width for monospace font
            let width = (line.len() as f64 * 8.5).ceil() as usize;
            let svg = format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="18"><text x="0" y="14" fill="{}" font-family="monospace" font-size="14" font-weight="bold" xml:space="preserve">{}</text></svg>"#,
                width, color, escaped_line
            );

            use base64::{Engine as _, engine::general_purpose};
            let b64 = general_purpose::STANDARD.encode(svg);
            // CRITICAL: No space between ] and ( for Markdown images
            result.push_str(&format!("![{}](data:image/svg+xml;base64,{})", line, b64));
        }
    }


    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loc_parser::LocEntry;
    use crate::ast::Range;

    #[test]
    fn test_resolve_loc() {
        let mut loc = HashMap::new();
        loc.insert("KEY1".to_string(), LocEntry {
            key: "KEY1".to_string(),
            value: "Value 1".to_string(),
            range: Range { start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            path: "".to_string(),
            value_start_col: 0,
        });
        loc.insert("KEY2".to_string(), LocEntry {
            key: "KEY2".to_string(),
            value: "Contains $KEY1$".to_string(),
            range: Range { start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            path: "".to_string(),
            value_start_col: 0,
        });

        assert_eq!(resolve_loc("Hello $KEY1$", &loc, 0), "Hello Value 1");
        assert_eq!(resolve_loc("Hello $KEY2$", &loc, 0), "Hello Contains Value 1");
        assert_eq!(resolve_loc("Hello $UNKNOWN$", &loc, 0), "Hello $UNKNOWN$");
    }

    #[test]
    fn test_paradox_to_markdown_newlines() {
        use base64::Engine as _;
        let loc = HashMap::new();
        // Test literal \n
        let input = "Line 1\\nLine 2";
        let output = paradox_to_markdown(input, Some(&loc));
        // It should contain two SVG segments (one for each line)
        // SVG contains &#160; for spaces
        let decoded = String::from_utf8(base64::engine::general_purpose::STANDARD.decode(output.split("base64,").nth(1).unwrap().split(')').next().unwrap()).unwrap()).unwrap();
        assert!(decoded.contains("Line&#160;1"));
        assert!(output.contains("  \n"));
    }
}

fn find_identifier_in_loc(content: &str, pos: Position) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let line = lines.get(pos.line as usize)?;
    let char_offset = pos.character as usize;

    // Check for bracketed scope [Root.GetTag]
    let re_scope = regex::Regex::new(r"\[([^\]]+)\]").unwrap();
    for cap in re_scope.captures_iter(line) {
        let m = cap.get(0).unwrap();
        if char_offset >= m.start() && char_offset < m.end() {
            let inner = cap.get(1).unwrap().as_str();
            let relative_offset = char_offset - m.start() - 1; // -1 for [
            let parts: Vec<&str> = inner.split('.').collect();
            let mut current_pos = 0;
            for part in parts {
                if relative_offset >= current_pos && relative_offset < current_pos + part.len() {
                    return Some(part.to_string());
                }
                current_pos += part.len() + 1; // +1 for .
            }
        }
    }

    // Check for variables $VAR$
    let re_var = regex::Regex::new(r"\$([^\$]+)\$").unwrap();
    for cap in re_var.captures_iter(line) {
        let m = cap.get(0).unwrap();
        if char_offset >= m.start() && char_offset < m.end() {
            return Some(cap.get(1).unwrap().as_str().to_string());
        }
    }

    None
}

fn ast_range_to_lsp(range: &ast::Range) -> Range {
    Range {
        start: Position { line: range.start_line, character: range.start_col },
        end: Position { line: range.end_line, character: range.end_col },
    }
}

fn ast_range_to_lsp_location(range: &ast::Range, path: &str) -> Location {
    Location {
        uri: Url::from_file_path(std::path::Path::new(path).canonicalize().unwrap_or_else(|_| std::path::PathBuf::from(path))).unwrap(),
        range: ast_range_to_lsp(range),
    }
}

fn find_identifier_at(script: &ast::Script, pos: Position, scope_stack: &mut scope::ScopeStack) -> Option<(String, Vec<scope::Scope>, Option<ast::Value>)> {
    for entry in &script.entries {
        if let Some(res) = find_in_entry(entry, pos, scope_stack) {
            return Some(res);
        }
    }
    None
}

fn find_in_entry(entry: &ast::Entry, pos: Position, scope_stack: &mut scope::ScopeStack) -> Option<(String, Vec<scope::Scope>, Option<ast::Value>)> {
    match entry {
        ast::Entry::Assignment(ass) => {
            if is_pos_in_range(pos, &ass.key_range) {
                return Some((ass.key.clone(), scope_stack.iter().cloned().collect(), Some(ass.value.value.clone())));
            }

            // Push scope if it's a block
            let mut pushed = false;
            if let ast::Value::Block(_) | ast::Value::TaggedBlock(_, _, _) = &ass.value.value {
                let s = scope::Scope::from_str(&ass.key);
                if s != scope::Scope::Unknown || ass.key.contains(':') || ass.key.contains('.') {
                    scope_stack.push(s);
                    pushed = true;
                }
            }

            let mut res = find_in_value(&ass.value, pos, scope_stack);

            if let Some((ref mut id, _, ref mut val_opt)) = res {
                if let ast::Value::Number(_) | ast::Value::Boolean(_) = &ass.value.value {
                    *id = ass.key.clone();
                    *val_opt = Some(ass.value.value.clone());
                }
            }

            if pushed {
                scope_stack.pop();
            }
            res
        }
        ast::Entry::Value(val) => find_in_value(val, pos, scope_stack),
        _ => None,
    }
}

fn find_in_value(val: &ast::NodeedValue, pos: Position, scope_stack: &mut scope::ScopeStack) -> Option<(String, Vec<scope::Scope>, Option<ast::Value>)> {
    match &val.value {
        ast::Value::String(s) => {
            if is_pos_in_range(pos, &val.range) {
                if pos.line == val.range.start_line {
                    let char_offset = pos.character.saturating_sub(val.range.start_col);
                    // Heuristic: quoted strings have quotes at start/end
                    let is_quoted = val.range.end_col - val.range.start_col > s.len() as u32;
                    let adj_offset = if is_quoted { char_offset.saturating_sub(1) } else { char_offset } as usize;

                    // Handle localization scopes [Scope.Command]
                    let mut start_search = 0;
                    while let Some(open) = s[start_search..].find('[') {
                        let abs_open = start_search + open;
                        if let Some(close) = s[abs_open..].find(']') {
                            let abs_close = abs_open + close;
                            if adj_offset > abs_open && adj_offset <= abs_close {
                                let inner = &s[abs_open + 1 .. abs_close];
                                let mut current_part_start = 0;
                                for part in inner.split('.') {
                                    let part_abs_start = abs_open + 1 + current_part_start;
                                    let part_abs_end = part_abs_start + part.len();
                                    if adj_offset >= part_abs_start && adj_offset < part_abs_end {
                                        return Some((part.to_string(), scope_stack.iter().cloned().collect(), None));
                                    }
                                    current_part_start += part.len() + 1;
                                }
                                return Some((inner.to_string(), scope_stack.iter().cloned().collect(), None));
                            }
                            start_search = abs_close + 1;
                        } else { break; }
                    }
                }
                return Some((s.clone(), scope_stack.iter().cloned().collect(), None));
            }
            None
        }
        ast::Value::Block(entries) => {
            for entry in entries {
                if let Some(res) = find_in_entry(entry, pos, scope_stack) {
                    return Some(res);
                }
            }
            None
        }
        ast::Value::TaggedBlock(_, entries, _) => {
            for entry in entries {
                if let Some(res) = find_in_entry(entry, pos, scope_stack) {
                    return Some(res);
                }
            }
            None
        }
        _ => None,
    }
}

fn is_pos_in_range(pos: Position, range: &ast::Range) -> bool {
    if pos.line < range.start_line || pos.line > range.end_line {
        return false;
    }
    if pos.line == range.start_line && pos.character < range.start_col {
        return false;
    }
    if pos.line == range.end_line && pos.character > range.end_col {
        return false;
    }
    true
}

fn find_context_at(script: &ast::Script, pos: Position) -> Option<String> {
    for entry in &script.entries {
        if let Some(ctx) = find_context_in_entry(entry, pos) {
            return Some(ctx);
        }
    }
    None
}

fn find_context_in_entry(entry: &ast::Entry, pos: Position) -> Option<String> {
    match entry {
        ast::Entry::Assignment(ass) => {
            if is_pos_in_range(pos, &ass.value.range) {
                if let Some(inner) = find_context_in_value(&ass.value, pos) {
                    return Some(inner);
                }
                return Some(ass.key.clone());
            }
            None
        }
        ast::Entry::Value(val) => find_context_in_value(val, pos),
        _ => None,
    }
}

fn find_context_in_value(val: &ast::NodeedValue, pos: Position) -> Option<String> {
    match &val.value {
        ast::Value::Block(entries) => {
            for entry in entries {
                if let Some(ctx) = find_context_in_entry(entry, pos) {
                    return Some(ctx);
                }
            }
            None
        }
        ast::Value::TaggedBlock(_, entries, _) => {
            for entry in entries {
                if let Some(ctx) = find_context_in_entry(entry, pos) {
                    return Some(ctx);
                }
            }
            None
        }
        _ => None,
    }
}
