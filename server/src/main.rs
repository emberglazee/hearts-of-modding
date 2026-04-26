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
mod event_scanner;

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
    events: Arc<RwLock<HashMap<String, event_scanner::Event>>>,
    ignored_loc_regex: Arc<RwLock<Vec<regex::Regex>>>,
    styling_enabled: Arc<RwLock<bool>>,
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
                    trigger_characters: Some(vec!["=".to_string(), "{".to_string()]),
                    ..Default::default()
                }),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["hoi4.getEventGraph".to_string()],
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
            self.scan_scripted(&roots),
            self.scan_ideologies(&roots),
            self.scan_traits(&roots),
            self.scan_sprites(&roots),
            self.scan_ideas(&roots),
            self.scan_variables(&roots),
            self.scan_provinces(&roots),
            self.scan_modifiers(&roots),
            self.scan_events(&roots),
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
        let r = (params.color.red * 255.0) as u32;
        let g = (params.color.green * 255.0) as u32;
        let b = (params.color.blue * 255.0) as u32;

        let new_text = format!("{{ {} {} {} }}", r, g, b);

        Ok(vec![ColorPresentation {
            label: new_text.clone(),
            text_edit: Some(TextEdit {
                range: params.range,
                new_text,
            }),
            additional_text_edits: None,
        }])
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let position = params.text_document_position_params.position;

        if let Some(content) = self.documents.get(&uri) {
            if let Ok(script) = parser::parse_script(&content) {
                let mut scope_stack = scope::ScopeStack::new(scope::Scope::Global);
                if let Some((identifier, final_scopes)) = find_identifier_at(&script, position, &mut scope_stack) {
                    let mut hover_text = String::new();
                    
                    fn push_section(full_text: &mut String, section: &str) {
                        if !full_text.is_empty() && !full_text.ends_with("---\n\n") {
                            full_text.push_str("\n\n---\n\n");
                        }
                        full_text.push_str(section);
                    }

                    // Show scope stack
                    let mut scope_text = String::from("### Scope Stack\n");
                    for (i, s) in final_scopes.iter().enumerate() {
                        if i > 0 { scope_text.push_str(" > "); }
                        scope_text.push_str(s.as_str());
                    }
                    push_section(&mut hover_text, &scope_text);

                    // Check triggers/effects
                    if let Some(entity) = TRIGGERS.get(identifier.as_str()).or_else(|| EFFECTS.get(identifier.as_str())) {
                        push_section(&mut hover_text, entity.description);
                    } else if SCOPES.contains(&identifier.to_uppercase().as_str()) {
                         push_section(&mut hover_text, &format!("### Scope: {}\n\nStandard Paradox scope.", identifier.to_uppercase()));
                    } else if LOC_COMMANDS.contains(&identifier.as_str()) {
                         push_section(&mut hover_text, &format!("### Localization Command: {}\n\nStandard localization command.", identifier));
                    } else {
                        // Check localization
                        let loc = self.localization.read().await;
                        // Try exact match first, then try keys starting with ID:
                        let entry = loc.get(&identifier).or_else(|| {
                            // Find any key that starts with "identifier:"
                            let target = format!("{}:", identifier);
                            loc.iter().find(|(k, _)| k.starts_with(&target)).map(|(_, e)| e)
                        });

                        if let Some(e) = entry {
                            push_section(&mut hover_text, &format!("**Localization:**\n\n{}", e.value));
                        } else {
                            // Check scripted triggers
                            let st = self.scripted_triggers.read().await;
                            if let Some(entity) = st.get(&identifier) {
                                push_section(&mut hover_text, &format!("**Scripted Trigger**\n\nDefined in: {}", self.make_file_link(&entity.path)));
                            } else {
                                // Check scripted effects
                                let se = self.scripted_effects.read().await;
                                if let Some(entity) = se.get(&identifier) {
                                    push_section(&mut hover_text, &format!("**Scripted Effect**\n\nDefined in: {}", self.make_file_link(&entity.path)));
                                }
                            }
                        }
                    }

                    // Check ideologies
                    let id_map = self.ideologies.read().await;
                    if let Some(ideology) = id_map.get(&identifier) {
                        push_section(&mut hover_text, &format!("### Ideology: {}\n\nDefined in: {}\n\nSub-ideologies: {}", 
                            ideology.name, self.make_file_link(&ideology.path), ideology.sub_ideologies.join(", ")));
                    }

                    // Check sub-ideologies
                    let sid_map = self.sub_ideologies.read().await;
                    if let Some((parent, _, path)) = sid_map.get(&identifier) {
                        push_section(&mut hover_text, &format!("### Sub-Ideology: {}\n\nParent Ideology: `{}`\n\nDefined in: {}", 
                            identifier, parent, self.make_file_link(path)));
                    }

                    // Check traits
                    let t_map = self.traits.read().await;
                    if let Some(trait_info) = t_map.get(&identifier) {
                        push_section(&mut hover_text, &format!("### Trait: {}\n\nType: `{}`\n\nDefined in: {}", 
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

                        push_section(&mut hover_text, &format!("### Sprite: {}\n\nTexture: {}\n\nDefined in: {}", 
                            sprite.name, texture_link, self.make_file_link(&sprite.path)));
                    }

                    // Check events
                    let e_map = self.events.read().await;
                    if let Some(event) = e_map.get(&identifier) {
                        push_section(&mut hover_text, &format!("### Event: {}\n\nType: `{}`\n\nDefined in: {}\n\nTriggers: {}", 
                            event.id, event.event_type, self.make_file_link(&event.path), 
                            if event.triggered_events.is_empty() { "None".to_string() } else { event.triggered_events.join(", ") }));
                    }

                    // Check ideas
                    let idea_map = self.ideas.read().await;
                    if let Some(idea) = idea_map.get(&identifier) {
                        push_section(&mut hover_text, &format!("### Idea: {}\n\nCategory: `{}`\n\nDefined in: {}",
                            idea.name, idea.category, self.make_file_link(&idea.path)));
                    }

                    // Check modifiers
                    let custom_mods = self.custom_modifiers.read().await;
                    if let Some(modifier) = custom_mods.get(&identifier) {
                        push_section(&mut hover_text, &format!("### Custom Modifier: {}\n\nDefined in: {}",
                            identifier, self.make_file_link(&modifier.path)));
                    }
                    let mappings = self.modifier_mappings.read().await;
                    if let Some(loc_key) = mappings.get(&identifier) {
                        push_section(&mut hover_text, &format!("### Engine Modifier: {}\n\nMaps to localization: `{}`",
                            identifier, loc_key));
                    }

                    // Check variables
                    let var_map = self.variables.read().await;
                    if let Some(vars) = var_map.get(&identifier) {
                        let paths: Vec<String> = vars.iter().map(|v| self.make_file_link(&v.path)).collect();
                        push_section(&mut hover_text, &format!("### Variable: {}\n\nUsed/Defined in:\n- {}", 
                            identifier, paths.join("\n- ")));
                    }

                    // Check event targets
                    let target_map = self.event_targets.read().await;
                    if let Some(targets) = target_map.get(&identifier) {
                        let paths: Vec<String> = targets.iter().map(|t| format!("{} ({})", self.make_file_link(&t.path), if t.is_global { "Global" } else { "Local" })).collect();
                        push_section(&mut hover_text, &format!("### Event Target: {}\n\nSaved in:\n- {}", 
                            identifier, paths.join("\n- ")));
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
        
        // Try to find context
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
            if let Ok(script) = parser::parse_script(&content) {
                let mut scope_stack = scope::ScopeStack::new(scope::Scope::Global);
                if let Some((identifier, _)) = find_identifier_at(&script, position, &mut scope_stack) {
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
                if let Some((identifier, _)) = find_identifier_at(&script, position, &mut scope_stack) {
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
                    } else if code == "styling_indent" {
                        if let Some(content) = self.documents.get(&params.text_document.uri.to_string()) {
                            let line_idx = diagnostic.range.start.line as usize;
                            if let Some(line) = content.lines().nth(line_idx) {
                                let leading = line.chars().take_while(|c| c.is_whitespace()).collect::<String>();
                                // Convert all spaces to tabs in the leading whitespace
                                let new_indent = leading.replace("    ", "\t").replace("  ", "\t"); // Simple heuristic
                                
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

    async fn scan_modifiers(&self, roots: &[std::path::PathBuf]) {
        let result = modifier_scanner::scan_modifiers(roots);
        
        let mut custom = self.custom_modifiers.write().await;
        *custom = result.custom_modifiers;

        let mut mappings = self.modifier_mappings.write().await;
        *mappings = result.builtin_mappings;

        self.client.log_message(MessageType::INFO, format!("Total: Loaded {} custom modifiers and {} builtin mappings", custom.len(), mappings.len())).await;
    }

    async fn scan_variables(&self, roots: &[std::path::PathBuf]) {
        let result = variable_scanner::scan_roots(roots);
        
        let mut vars = self.variables.write().await;
        *vars = result.variables;

        let mut targets = self.event_targets.write().await;
        *targets = result.event_targets;

        self.client.log_message(MessageType::INFO, format!("Total: Loaded {} variables and {} event targets", vars.len(), targets.len())).await;
    }

    fn collect_casing_fixes(&self, entries: &[ast::Entry], fixes: &mut Vec<(ast::Range, String)>) {
        let keywords = [
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
                        ast::Value::TaggedBlock(_, inner) => self.collect_casing_fixes(inner, fixes),
                        _ => {}
                    }
                }
                ast::Entry::Value(val) => {
                    match &val.value {
                        ast::Value::Block(inner) => self.collect_casing_fixes(inner, fixes),
                        ast::Value::TaggedBlock(_, inner) => self.collect_casing_fixes(inner, fixes),
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
                            files_to_scan.push(path);
                        }
                    }
                }
            }

            self.client.log_message(MessageType::INFO, format!("Found {} .yml files in {:?}", files_to_scan.len(), loc_dir)).await;

            for path in files_to_scan {
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        let path_str = path.to_string_lossy().to_string();
                        let parsed = loc_parser::parse_loc_file(&content, &path_str);
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

        if uri.as_str().ends_with(".yml") {
            self.validate_localization_content(&content, &mut diagnostics).await;
        } else {
            match parser::parse_script(&content) {
                Ok(script) => {
                    // Semantic validation
                    self.check_semantic(&script, &mut diagnostics, styling_enabled).await;
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
            self.check_styling(&content, &mut diagnostics);
        }

        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }

    async fn validate_localization_content(&self, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        let parsed = loc_parser::parse_loc_file(content, "");
        let event_targets = self.event_targets.read().await;

        for entry in parsed.values() {
            // 1. Check scopes [Root.GetName]
            let re_scope = regex::Regex::new(r"\[([^\]]+)\]").unwrap();
            for cap in re_scope.captures_iter(&entry.value) {
                let inner = cap.get(1).unwrap().as_str();
                let parts: Vec<&str> = inner.split('.').collect();
                for (i, part) in parts.iter().enumerate() {
                    let is_last = i == parts.len() - 1;
                    let part_upper = part.to_uppercase();
                    let mut valid = false;
                    
                    if is_last {
                        if LOC_COMMANDS.contains(&part.to_string().as_str()) || 
                           SCOPES.contains(&part_upper.as_str()) ||
                           event_targets.contains_key(*part) {
                            valid = true;
                        }
                    } else {
                        if SCOPES.contains(&part_upper.as_str()) ||
                           event_targets.contains_key(*part) {
                            valid = true;
                        }
                    }

                    if !valid {
                        diagnostics.push(Diagnostic {
                            range: ast_range_to_lsp(&entry.range),
                            severity: Some(DiagnosticSeverity::WARNING),
                            message: format!("Potential invalid localization scope or command: '{}' in '{}'", part, inner),
                            ..Default::default()
                        });
                    }
                }
            }

            // 2. Check color codes §Y...§!
            let re_color = regex::Regex::new(r"§([a-zA-Z!])").unwrap();
            let mut open_colors = 0;
            for cap in re_color.captures_iter(&entry.value) {
                let code = cap.get(1).unwrap().as_str();
                if code == "!" {
                    if open_colors > 0 {
                        open_colors -= 1;
                    } else {
                        diagnostics.push(Diagnostic {
                            range: ast_range_to_lsp(&entry.range),
                            severity: Some(DiagnosticSeverity::INFORMATION),
                            message: "Found color reset (§!) without matching color start.".to_string(),
                            ..Default::default()
                        });
                    }
                } else {
                    open_colors += 1;
                }
            }
            if open_colors > 0 {
                diagnostics.push(Diagnostic {
                    range: ast_range_to_lsp(&entry.range),
                    severity: Some(DiagnosticSeverity::INFORMATION),
                    message: format!("Unclosed color code(s) in localization: {}", entry.key),
                    ..Default::default()
                });
            }
        }
    }

    fn check_styling(&self, content: &str, diagnostics: &mut Vec<Diagnostic>) {
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

            // 2. Mixed indentation
            let leading = line.chars().take_while(|c| c.is_whitespace()).collect::<String>();
            if leading.contains(' ') && leading.contains('\t') {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: line_idx as u32, character: 0 },
                        end: Position { line: line_idx as u32, character: leading.len() as u32 },
                    },
                    severity: Some(DiagnosticSeverity::INFORMATION),
                    code: Some(NumberOrString::String("styling_indent".to_string())),
                    message: "Mixed tabs and spaces in indentation.".to_string(),
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

        let mut comments = Vec::new();
        for entry in &script.entries {
            if let ast::Entry::Comment(c, r) = entry {
                comments.push((c.clone(), r.clone()));
            }
        }

        for entry in &script.entries {
            self.check_entry_semantic(entry, diagnostics, &loc, &st, &se, &id, &sid, &tr, &sp, &ids, &provs, &comments, styling_enabled);
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
        comments: &[ (String, ast::Range) ],
        styling_enabled: bool,
    ) {
        match entry {
            ast::Entry::Assignment(ass) => {
                let key_lower = ass.key.to_lowercase();

                // Casing checks for keywords
                if styling_enabled {
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
                                    let ig = futures::executor::block_on(self.ignored_loc_regex.read());
                                    let is_regex_ignored = ig.iter().any(|re| re.is_match(val));
                                    
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
                self.check_value_semantic(&ass.value, diagnostics, loc, st, se, id, sid, tr, sp, ids, provs, comments, styling_enabled);
            }
            ast::Entry::Value(val) => {
                self.check_value_semantic(val, diagnostics, loc, st, se, id, sid, tr, sp, ids, provs, comments, styling_enabled);
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
        comments: &[ (String, ast::Range) ],
        styling_enabled: bool,
    ) {
        match &val.value {
            ast::Value::Block(entries) => {
                for entry in entries {
                    self.check_entry_semantic(entry, diagnostics, loc, st, se, id, sid, tr, sp, ids, provs, comments, styling_enabled);
                }
            }
            ast::Value::TaggedBlock(_, entries) => {
                for entry in entries {
                    self.check_entry_semantic(entry, diagnostics, loc, st, se, id, sid, tr, sp, ids, provs, comments, styling_enabled);
                }
            }
            _ => {}
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
        events: Arc::new(RwLock::new(HashMap::new())),
        ignored_loc_regex: Arc::new(RwLock::new(Vec::new())),
        styling_enabled: Arc::new(RwLock::new(true)),
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
        if ass.key.to_lowercase().contains("color") {
            find_colors_in_value(&ass.value, colors);
        } else {
            // Recurse into blocks even if key doesn't match
            find_colors_in_value(&ass.value, colors);
        }
    } else if let ast::Entry::Value(val) = entry {
        find_colors_in_value(val, colors);
    }
}

fn find_colors_in_value(val: &ast::NodeedValue, colors: &mut Vec<ColorInformation>) {
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

            if nums.len() == 3 {
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
        ast::Value::TaggedBlock(tag, entries) => {
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

fn find_identifier_at(script: &ast::Script, pos: Position, scope_stack: &mut scope::ScopeStack) -> Option<(String, Vec<scope::Scope>)> {
    for entry in &script.entries {
        if let Some(res) = find_in_entry(entry, pos, scope_stack) {
            return Some(res);
        }
    }
    None
}

fn find_in_entry(entry: &ast::Entry, pos: Position, scope_stack: &mut scope::ScopeStack) -> Option<(String, Vec<scope::Scope>)> {
    match entry {
        ast::Entry::Assignment(ass) => {
            if is_pos_in_range(pos, &ass.key_range) {
                return Some((ass.key.clone(), scope_stack.iter().cloned().collect()));
            }
            
            // Push scope if it's a block
            let mut pushed = false;
            if let ast::Value::Block(_) = &ass.value.value {
                scope_stack.push(scope::Scope::from_str(&ass.key));
                pushed = true;
            } else if let ast::Value::TaggedBlock(_, _) = &ass.value.value {
                scope_stack.push(scope::Scope::from_str(&ass.key));
                pushed = true;
            }

            let res = find_in_value(&ass.value, pos, scope_stack);
            
            if pushed {
                scope_stack.pop();
            }
            res
        }
        ast::Entry::Value(val) => find_in_value(val, pos, scope_stack),
        _ => None,
    }
}

fn find_in_value(val: &ast::NodeedValue, pos: Position, scope_stack: &mut scope::ScopeStack) -> Option<(String, Vec<scope::Scope>)> {
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
                                        return Some((part.to_string(), scope_stack.iter().cloned().collect()));
                                    }
                                    current_part_start += part.len() + 1;
                                }
                                return Some((inner.to_string(), scope_stack.iter().cloned().collect()));
                            }
                            start_search = abs_close + 1;
                        } else { break; }
                    }
                }
                return Some((s.clone(), scope_stack.iter().cloned().collect()));
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
        ast::Value::TaggedBlock(_, entries) => {
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
        ast::Value::TaggedBlock(_, entries) => {
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
