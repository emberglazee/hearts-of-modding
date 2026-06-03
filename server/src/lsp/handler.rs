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
use crate::parser::ast;
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
            if let Some(dep_paths) = options.get("dependencyModPaths").and_then(|v| v.as_array()) {
                let mut paths = Vec::new();
                for val in dep_paths {
                    if let Some(s) = val.as_str() {
                        if !s.is_empty() {
                            paths.push(s.to_string());
                        }
                    }
                }
                self.config.set_dependency_mod_paths(paths);
                let _dp = self.config.dependency_mod_paths();
            }
            if let Some(path) = options.get("modRegistryPath").and_then(|v| v.as_str()) {
                if !path.is_empty() {
                    self.config.set_mod_registry_path(Some(path.to_string()));
                    let _mrp = self.config.mod_registry_path();
                }
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

        // Build the VFS root stack: game path (lowest priority) → dependency
        // mods → workspace (highest priority). The scan_dashmap_layered! macro
        // iterates roots in order, pushing each root's entries into LayeredValue
        // so that later roots override earlier ones.
        let mut roots: Vec<std::path::PathBuf> = Vec::new();

        // 1. Game path (vanilla files, lowest priority)
        let gp = self.config.game_path();
        let game_path_configured = gp.is_some();
        if let Some(ref path) = gp {
            roots.push(std::path::PathBuf::from(path));
            self.client
                .log_message(MessageType::INFO, format!("Using HOI4 game path: {}", path))
                .await;
        }

        // 2. Auto-discover dependency mods via the Paradox mod registry.
        //
        //    Only engages when the user has configured `hoi4.gamePath` — no
        //    game path means no vanilla HOI4 reference, so dependency mods
        //    wouldn't be meaningful either.
        //
        //    Registry path resolution order:
        //      a) User-configured `hoi4.modRegistryPath` setting
        //      b) OS default (`~/.local/share/...`, `~/Documents/...`)
        //      c) None → no auto-discovery
        //
        //    If the registry exists we parse `descriptor.mod` from the workspace
        //    for `dependencies = { ... }`, then resolve each dependency name
        //    to a mod directory via `.mod` files in the registry.
        //
        // 3. Explicit `hoi4.modPaths` paths — these are inserted AFTER the
        //    auto-discovered ones, so they take higher priority (but the
        //    workspace root always wins).
        // -----------------------------------------------------------------

        if !game_path_configured {
            self.client
                .log_message(
                    MessageType::LOG,
                    "Game path not configured — skipping auto-discovery of dependency mods",
                )
                .await;
        }

        if game_path_configured {
            let registry_path = self.config.mod_registry_path().or_else(|| {
                crate::utils::mod_registry::default_mod_registry_path()
                    .map(|p| p.to_string_lossy().to_string())
            });

            match registry_path {
                Some(ref reg) => {
                    let reg_path = std::path::Path::new(reg);
                    self.client
                        .log_message(
                            MessageType::INFO,
                            format!("Using Paradox mod registry: {}", reg),
                        )
                        .await;

                    // Read the workspace descriptor.mod to find declared dependencies
                    let descriptor_path = std::path::Path::new("descriptor.mod");
                    if descriptor_path.exists() {
                        match std::fs::read_to_string(descriptor_path) {
                            Ok(content) => {
                                let dep_names =
                                    crate::utils::mod_registry::parse_dependencies(&content);
                                if !dep_names.is_empty() {
                                    let resolved =
                                        crate::utils::mod_registry::resolve_dependency_paths(
                                            reg_path, &dep_names,
                                        );
                                    for resolved_path in &resolved {
                                        roots.push(resolved_path.clone());
                                        self.client
                                            .log_message(
                                                MessageType::INFO,
                                                format!(
                                                    "Resolved dependency mod: {}",
                                                    resolved_path.display()
                                                ),
                                            )
                                            .await;
                                    }
                                    self.client
                                        .log_message(
                                            MessageType::INFO,
                                            format!(
                                                "Resolved {}/{} dependency mods from workspace descriptor.mod",
                                                resolved.len(),
                                                dep_names.len(),
                                            ),
                                        )
                                        .await;
                                } else {
                                    self.client
                                        .log_message(
                                            MessageType::INFO,
                                            "No dependencies found in workspace descriptor.mod",
                                        )
                                        .await;
                                }
                            }
                            Err(e) => {
                                self.client
                                    .log_message(
                                        MessageType::WARNING,
                                        format!("Failed to read descriptor.mod: {}", e),
                                    )
                                    .await;
                            }
                        }
                    } else {
                        self.client
                            .log_message(
                                MessageType::LOG,
                                "No descriptor.mod found in workspace — skipping dependency resolution",
                            )
                            .await;
                    }
                }
                None => {
                    // Game path is configured but we couldn't find a mod registry
                    self.client
                        .show_message(
                            MessageType::WARNING,
                            "Hearts of Modding: could not find the Paradox Interactive mod registry folder. \
                             Dependency mods from descriptor.mod won't be resolved. \
                             Set hoi4.modRegistryPath in settings to point to the correct location, \
                             or add paths manually via hoi4.modPaths."
                        )
                        .await;
                    self.client
                        .log_message(
                            MessageType::WARNING,
                            "Mod registry not found. Set hoi4.modRegistryPath or add paths via hoi4.modPaths.",
                        )
                        .await;
                }
            }
        }

        // 4. Explicit dependency mod paths (higher priority than auto-discovered)
        let dep_paths = self.config.dependency_mod_paths();
        for dep_path in dep_paths {
            let path_buf = std::path::PathBuf::from(&dep_path);
            if path_buf.exists() {
                roots.push(path_buf);
                self.client
                    .log_message(
                        MessageType::INFO,
                        format!("Using explicit dependency mod path: {}", dep_path),
                    )
                    .await;
            } else {
                self.client
                    .log_message(
                        MessageType::WARNING,
                        format!("Explicit dependency mod path does not exist: {}", dep_path),
                    )
                    .await;
            }
        }

        // 5. Workspace root (active mod, highest priority)
        roots.push(std::path::PathBuf::from("."));

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
            self.scan_terrains(&roots),
            self.scan_modifiers(&roots),
            self.scan_buildings(&roots),
            self.scan_resources(&roots),
            self.scan_state_categories(&roots),
            self.scan_achievements(&roots),
            self.scan_balance_of_powers(&roots),
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
        // Use the workspace root (last element) — not the game path
        if let Some(workspace_root) = roots.last() {
            self.collect_workspace_files(std::slice::from_ref(workspace_root))
                .await;
        }

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

        // Register file watchers so did_change_watched_files fires for
        // external file operations (Git branch switch, rename/delete via
        // VS Code file explorer, etc.)
        let watchers = vec![FileSystemWatcher {
            glob_pattern: GlobPattern::String("**/*.{txt,yml,asset,gfx,gui,csv,lua,mod}".into()),
            kind: Some(WatchKind::Create | WatchKind::Change | WatchKind::Delete),
        }];
        let registration = Registration {
            id: "hoi4-watched-files".to_string(),
            method: "workspace/didChangeWatchedFiles".to_string(),
            register_options: Some(
                serde_json::to_value(DidChangeWatchedFilesRegistrationOptions { watchers })
                    .unwrap_or_default(),
            ),
        };
        if let Err(e) = self.client.register_capability(vec![registration]).await {
            self.client
                .log_message(
                    MessageType::WARNING,
                    format!("Failed to register file watchers: {}", e),
                )
                .await;
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

        // Parse on a blocking thread to avoid blocking the LSP event loop.
        let result = tokio::task::spawn_blocking(move || {
            let (script, errors) = parser::parse_script(&text);
            (Arc::new(script), errors)
        })
        .await
        .unwrap_or_else(|e| {
            eprintln!("[hoi4] Parse task panicked: {e}");
            (Arc::new(ast::Script { entries: vec![] }), vec![])
        });

        self.document_asts.insert(uri, result);
        self.validate_document(params.text_document.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.as_str().to_string();
        let text = params.content_changes[0].text.clone();
        self.documents.insert(uri.clone(), text.clone());

        // Parse on a blocking thread to avoid blocking the LSP event loop.
        let result = tokio::task::spawn_blocking(move || {
            let (script, errors) = parser::parse_script(&text);
            (Arc::new(script), errors)
        })
        .await
        .unwrap_or_else(|e| {
            eprintln!("[hoi4] Parse task panicked: {e}");
            (Arc::new(ast::Script { entries: vec![] }), vec![])
        });

        let (script, errors) = result;
        let script_for_scanner = script.clone();
        self.document_asts.insert(uri.clone(), (script, errors));

        // Live-update scanner data from cached AST (no re-parse needed)
        if let Some(file_path) = params.text_document.uri.to_file_path() {
            let path_str = file_path.to_string_lossy().to_string();
            crate::scanner::incremental_scanner::update_scanner_data_from_ast(
                &self.scanner_data,
                &path_str,
                &script_for_scanner,
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

        // Update scanner data for changed file (fast DashMap ops, safe on event loop)
        if let Some(file_path) = uri.to_file_path() {
            let path_str = file_path.to_string_lossy().to_string();
            crate::scanner::incremental_scanner::update_scanner_data_for_file(
                &self.scanner_data,
                &path_str,
                &content,
            );
        }

        // Parse on a blocking thread to avoid blocking the LSP event loop.
        let result = tokio::task::spawn_blocking(move || {
            let (script, errors) = parser::parse_script(&content);
            (Arc::new(script), errors)
        })
        .await
        .unwrap_or_else(|e| {
            eprintln!("[hoi4] Parse task panicked: {e}");
            (Arc::new(ast::Script { entries: vec![] }), vec![])
        });

        self.document_asts.insert(uri_str, result);
        self.validate_document(uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.as_str().to_string();
        self.documents.remove(&uri);
        self.document_asts.remove(&uri);
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        use std::sync::Arc;

        for event in params.changes {
            let uri = &event.uri;
            let path_str = match uri.to_file_path() {
                Some(p) => p.to_string_lossy().to_string(),
                None => continue,
            };

            match event.typ {
                FileChangeType::CREATED | FileChangeType::CHANGED => {
                    // Try to read the file from disk and route through
                    // the incremental scanner to add/refresh entities.
                    match tokio::fs::read_to_string(&path_str).await {
                        Ok(content) => {
                            crate::scanner::incremental_scanner::update_scanner_data_for_file(
                                &self.scanner_data,
                                &path_str,
                                &content,
                            );
                            // Track in workspace_files for rename operations
                            self.scanner_data
                                .workspace_files
                                .insert(Arc::from(path_str.as_str()));
                        }
                        Err(_) => {
                            // File disappeared between the event and our
                            // attempt to read it — treat as deletion.
                            crate::scanner::incremental_scanner::remove_path_from_scanner_data(
                                &self.scanner_data,
                                &path_str,
                            );
                            self.scanner_data.workspace_files.remove(path_str.as_str());
                        }
                    }
                }
                FileChangeType::DELETED => {
                    crate::scanner::incremental_scanner::remove_path_from_scanner_data(
                        &self.scanner_data,
                        &path_str,
                    );
                    self.scanner_data.workspace_files.remove(path_str.as_str());
                }
                _ => {}
            }
        }
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
                // Unit leader skill properties
                keywords.insert("attack_skill".to_string());
                keywords.insert("defense_skill".to_string());
                keywords.insert("planning_skill".to_string());
                keywords.insert("logistics_skill".to_string());
                keywords.insert("maneuvering_skill".to_string());
                keywords.insert("coordination_skill".to_string());
                // Scientist specialization block
                keywords.insert("skills".to_string());
                // Advisor-only property
                keywords.insert("can_be_fired".to_string());

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
                keywords.insert("arctic_water".to_string());
                keywords.insert("min_snow_level".to_string());
                keywords.insert("naval_terrain".to_string());

                // Terrain definition keywords (common/terrain/*.txt)
                keywords.insert("categories".to_string());
                keywords.insert("color".to_string());
                keywords.insert("terrain".to_string());
                keywords.insert("movement_cost".to_string());
                keywords.insert("is_water".to_string());
                keywords.insert("sound_type".to_string());
                keywords.insert("minimum_seazone_dominance".to_string());
                keywords.insert("combat_width".to_string());
                keywords.insert("combat_support_width".to_string());
                keywords.insert("ai_terrain_importance_factor".to_string());
                keywords.insert("match_value".to_string());
                keywords.insert("buildings_max_level".to_string());
                keywords.insert("supply_flow_penalty_factor".to_string());
                keywords.insert("truck_attrition_factor".to_string());
                keywords.insert("navy_fuel_consumption_factor".to_string());
                keywords.insert("units".to_string());
                keywords.insert("battle_cruiser".to_string());
                keywords.insert("battleship".to_string());
                keywords.insert("heavy_cruiser".to_string());
                keywords.insert("carrier".to_string());
                keywords.insert("destroyer".to_string());
                keywords.insert("light_cruiser".to_string());
                keywords.insert("submarine".to_string());
                keywords.insert("texture".to_string());
                keywords.insert("spawn_city".to_string());
                keywords.insert("perm_snow".to_string());

                // Balance of power definition keywords (common/bop/*.txt)
                keywords.insert("initial_value".to_string());
                keywords.insert("left_side".to_string());
                keywords.insert("right_side".to_string());
                keywords.insert("decision_category".to_string());
                keywords.insert("side".to_string());
                keywords.insert("range".to_string());
                keywords.insert("min".to_string());
                keywords.insert("max".to_string());
                keywords.insert("on_activate".to_string());
                keywords.insert("on_deactivate".to_string());

                // Event definition keywords (events/*.txt) structural — not in triggers/effects data
                keywords.insert("add_namespace".to_string());
                keywords.insert("mean_time_to_happen".to_string());
                keywords.insert("fire_only_once".to_string());
                keywords.insert("is_triggered_only".to_string());
                keywords.insert("major".to_string());
                keywords.insert("show_major".to_string());
                keywords.insert("fire_for_sender".to_string());
                keywords.insert("minor_flavor".to_string());
                keywords.insert("timeout_days".to_string());
                keywords.insert("immediate".to_string());
                keywords.insert("after".to_string());
                keywords.insert("option".to_string());
                keywords.insert("original_recipient_only".to_string());
                keywords.insert("ai_chance".to_string());
                keywords.insert("title".to_string());
                keywords.insert("text".to_string());
                keywords.insert("tooltip".to_string());
                keywords.insert("trigger".to_string());

                // Event time / delay keywords (MTTH + event firing effect)
                keywords.insert("days".to_string());
                keywords.insert("months".to_string());
                keywords.insert("years".to_string());
                keywords.insert("hours".to_string());
                keywords.insert("random_hours".to_string());
                keywords.insert("random_days".to_string());
                keywords.insert("random".to_string());

                // Event-type-specific effect sub-keys
                keywords.insert("trigger_for".to_string());
                keywords.insert("occupied".to_string());
                keywords.insert("originator".to_string());
                keywords.insert("recipient".to_string());
                keywords.insert("set_root".to_string());
                keywords.insert("set_from".to_string());
                keywords.insert("set_from_from".to_string());

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
                .map(|e| (e.key().to_string(), e.value().resolve().clone()))
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
