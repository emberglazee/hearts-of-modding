use std::sync::Arc;

use tokio_util::sync::CancellationToken;
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
                        if let Ok(re) =
                            regex::Regex::new(&crate::utils::fs_util::escape_filename_chars(s))
                        {
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
                        if let Ok(re) =
                            regex::Regex::new(&crate::utils::fs_util::escape_filename_chars(s))
                        {
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
                            range: Some(true),
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
                                    SemanticTokenType::from("escapeCharacter"),
                                    SemanticTokenType::from("parameter"),
                                    SemanticTokenType::from("boolean"),
                                    SemanticTokenType::from("metaScope"),
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
                        "hoi4/getColorCodes".to_string(),
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

        // Log current configuration
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Config: styling={}, workspace_scan={}, log_level={}, cosmetic_loc_indent={}, loc_ignore={:?}, files_ignore={:?}",
                    if self.config.styling_enabled() { "on" } else { "off" },
                    if self.config.workspace_scan_enabled() { "on" } else { "off" },
                    self.config.log_level().prefix(),
                    if self.config.cosmetic_loc_indent() { "on" } else { "off" },
                    self.config.ignored_loc_regex().iter().map(|r| r.as_str()).collect::<Vec<_>>(),
                    self.config.ignored_files_regex().iter().map(|r| r.as_str()).collect::<Vec<_>>(),
                ),
            )
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

        // Parse replace_path directives from the workspace descriptor.mod.
        // Total-conversion mods declare replace_path to wipe entire subdirectories
        // from lower-priority layers (vanilla game path and dependency mods).
        let mut replace_paths: Vec<String> = Vec::new();

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
                    // and replace_path directives.
                    let descriptor_path = std::path::Path::new("descriptor.mod");
                    if descriptor_path.exists() {
                        match std::fs::read_to_string(descriptor_path) {
                            Ok(content) => {
                                // Parse replace_path declarations first — these must be
                                // available when the overlay is built, regardless of
                                // whether any dependencies were found.
                                replace_paths =
                                    crate::utils::mod_registry::parse_replace_paths(&content);
                                if !replace_paths.is_empty() {
                                    self.client
                                        .log_message(
                                            MessageType::INFO,
                                            format!(
                                                "Workspace mod declares replace_path: {:?}",
                                                replace_paths
                                            ),
                                        )
                                        .await;
                                }

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
        //
        // But if the workspace IS the HOI4 game installation directory, skip
        // pushing "." to avoid double-scanning every file — the game path root
        // (step 1) already covers this directory. Without this check, the same
        // files would be walked twice through the overlay, and every entity
        // would get an identical duplicate layer, causing false-positive
        // diagnostics ("everything is a duplicate of everything else").
        let same_as_game = gp
            .as_ref()
            .and_then(|gp_path| {
                let ws = std::path::Path::new(".").canonicalize().ok();
                let gp = std::path::Path::new(gp_path).canonicalize().ok();
                ws.zip(gp).map(|(w, g)| w == g)
            })
            .unwrap_or(false);

        if same_as_game {
            self.client
                .log_message(
                    MessageType::INFO,
                    "Workspace root is the configured HOI4 game installation path — \
                     skipping duplicate workspace root to avoid double-scanning.",
                )
                .await;
        } else {
            roots.push(std::path::PathBuf::from("."));
        }

        // Store roots for texture file path resolution and other validation
        *self.workspace_roots.lock().unwrap() = roots.clone();

        // Build file-level overlay for path-priority-based scanning.
        // Script files (events, ideas, focuses, etc.) use file-path-level override:
        // a mod file at the same relative path completely replaces the vanilla file.
        // Localization and defines still use key-level merge (LayeredValue).
        let overlay = crate::scanner::file_overlay::FileOverlay::build_script_only(
            &roots,
            &["txt", "yml", "asset", "gfx", "gui", "csv", "lua"],
            self.get_sync_filter(),
            &replace_paths,
        );

        let scan_start = std::time::Instant::now();
        tokio::join!(
            self.scan_localization(&roots), // merges by key — keeps root-based scanning
            self.load_assets(),             // no roots needed
            self.scan_scripted(&overlay),
            self.scan_ideologies(&overlay),
            self.scan_traits(&overlay),
            self.scan_sprites(&overlay),
            self.scan_ideas(&overlay),
            self.scan_characters(&overlay),
            self.scan_variables(&overlay),
            self.scan_provinces(&overlay),
            self.scan_states(&overlay),
            self.scan_logistics(&overlay),
            self.scan_map_objects(&overlay),
            self.scan_adjacencies(&overlay),
            self.scan_strategic_regions(&overlay),
            self.scan_terrains(&overlay),
            self.scan_modifiers(&overlay),
            self.scan_buildings(&overlay),
            self.scan_resources(&overlay),
            self.scan_state_categories(&overlay),
            self.scan_achievements(&overlay),
            self.scan_balance_of_powers(&overlay),
            self.scan_defines(&roots),
            self.scan_events(&overlay),
            self.scan_focuses(&overlay),
            self.scan_music(&overlay),
            self.scan_sounds(&overlay),
            self.scan_abilities(&overlay),
            self.scan_ai_strategy_plans(&overlay),
            self.scan_ai_areas(&overlay),
            self.scan_continents(&overlay),
            self.scan_portraits(&overlay),
            self.scan_countries(&overlay),
            self.scan_gfx(&overlay),
            self.scan_oobs(&overlay),
            self.scan_units(&overlay),
        );
        let scan_elapsed = scan_start.elapsed();
        self.client
            .log_message(
                MessageType::INFO,
                format!("All scanners completed in {:.1?}", scan_elapsed),
            )
            .await;

        // Rebuild reverse file-path indices so incremental updates are O(K) not O(N)
        self.scanner_data.rebuild_all_file_indices();
        // Update entity token context so semantic tokens reflect the freshly scanned entities
        self.update_entity_token_context();

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
                        // If the user just enabled the workspace scan, trigger it now
                        if enabled {
                            self.validate_workspace(std::path::Path::new(".")).await;
                        }
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
                if let Some(level_str) = hoi4.get("logLevel").and_then(|v| v.as_str()) {
                    let level = match level_str {
                        "error" => crate::log_level::LogLevel::Error,
                        "warn" => crate::log_level::LogLevel::Warn,
                        "info" => crate::log_level::LogLevel::Info,
                        "debug" => crate::log_level::LogLevel::Debug,
                        "trace" => crate::log_level::LogLevel::Trace,
                        _ => crate::log_level::LogLevel::Info,
                    };
                    self.config.set_log_level(level);
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
            (
                Arc::new(ast::Script {
                    source: Arc::from(""),
                    entries: vec![],
                }),
                vec![],
            )
        });

        self.document_asts.insert(uri, result);
        self.validate_document(params.text_document.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.as_str().to_string();
        let text = params.content_changes[0].text.clone();
        self.documents.insert(uri.clone(), text.clone());

        // Cancel any previous in-flight parse for this document.
        // Rapid keystrokes cancel each other, preventing CPU spiking.
        if let Some(token) = self.document_cancellation_tokens.get(&uri) {
            token.cancel();
        }

        // Create a fresh cancellation token for this parse attempt.
        let cancellation_token = CancellationToken::new();
        self.document_cancellation_tokens
            .insert(uri.clone(), cancellation_token.clone());

        // Debounce: sleep for 80ms, but wake immediately if cancelled.
        // Unlike the old version-query approach, select! ensures we don't
        // accumulate sleeping futures — cancelled ones return instantly.
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_millis(80)) => {},
            _ = cancellation_token.cancelled() => {
                return;
            }
        }

        // Gate 1: if cancelled during the sleep window, bail.
        if cancellation_token.is_cancelled() {
            return;
        }

        // Re-read the latest text (may have been updated multiple times during sleep)
        let text = match self.documents.get(&uri) {
            Some(t) => t.clone(),
            None => return,
        };

        // Parse on a blocking thread (CPU-bound work, off the event loop).
        let result = tokio::task::spawn_blocking(move || {
            let (script, errors) = parser::parse_script(&text);
            (Arc::new(script), errors)
        })
        .await
        .unwrap_or_else(|e| {
            eprintln!("[hoi4] Parse task panicked: {e}");
            (
                Arc::new(ast::Script {
                    source: Arc::from(""),
                    entries: vec![],
                }),
                vec![],
            )
        });

        // Gate 2: if cancelled during spawn_blocking, discard.
        if cancellation_token.is_cancelled() {
            return;
        }

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
            (
                Arc::new(ast::Script {
                    source: Arc::from(""),
                    entries: vec![],
                }),
                vec![],
            )
        });

        self.document_asts.insert(uri_str, result);
        self.validate_document(uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.as_str().to_string();
        // Cancel any in-flight parse for this document
        if let Some(token) = self.document_cancellation_tokens.get(&uri) {
            token.cancel();
        }
        self.document_cancellation_tokens.remove(&uri);
        self.documents.remove(&uri);
        self.document_asts.remove(&uri);
        // Run interner GC: the dropped AST may free interned strings
        // that were only referenced by this document's parse tree.
        self.scanner_data.interner.gc();
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        use std::sync::Arc;

        // Collect affected re-validation prefixes across ALL change events
        // so we only re-validate documents that could actually be affected.
        let mut all_affected: Vec<&'static str> = Vec::new();
        let mut has_wildcard = false;

        for event in params.changes {
            let uri = &event.uri;
            let path_str = match uri.to_file_path() {
                Some(p) => p.to_string_lossy().to_string(),
                None => continue,
            };

            // Accumulate dependency prefixes (skip if already wildcard)
            if !has_wildcard {
                let prefixes =
                    crate::scanner::incremental_scanner::dependency_affected_prefixes(&path_str);
                if prefixes.contains(&"/") {
                    has_wildcard = true;
                    all_affected.clear();
                } else {
                    all_affected.extend(prefixes);
                }
            }

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

        // Re-validate only the open documents whose paths intersect
        // with the accumulated affected prefixes.
        for entry in self.documents.iter() {
            let doc_uri_str = entry.key();
            if has_wildcard || all_affected.iter().any(|p| doc_uri_str.contains(p)) {
                if let Ok(uri) = doc_uri_str.parse::<Uri>() {
                    self.validate_document(uri).await;
                }
            }
        }
        // Update entity token context since scanner data may have changed,
        // so semantic tokens reflect the updated entity names on the next request.
        self.update_entity_token_context();
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

        // Script files — use a fresh AST that matches the current document
        // content, not a cached one that might be stale during the did_change
        // debounce window.
        match self.get_or_parse_ast(&uri).await {
            Some((script, _)) => {
                let ctx = self.build_semantic_token_context();

                Ok(Some(semantic_tokens::get_semantic_tokens(&script, &ctx)))
            }
            _ => Ok(None),
        }
    }

    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Result<Option<SemanticTokensRangeResult>> {
        let uri = params.text_document.uri.to_string();
        let range = params.range;

        // YAML localization files
        if uri.ends_with(".yml") {
            return if let Some(content) = self.documents.get(&uri) {
                let result = semantic_tokens::loc_semantic_tokens(&content);
                Ok(Some(filter_semantic_tokens_by_range(result, &range)))
            } else {
                Ok(None)
            };
        }

        // CSV map files
        if uri.ends_with(".csv") {
            return if let Some(content) = self.documents.get(&uri) {
                match semantic_tokens::csv_semantic_tokens(&content) {
                    Some(result) => Ok(Some(filter_semantic_tokens_by_range(result, &range))),
                    None => Ok(None),
                }
            } else {
                Ok(None)
            };
        }

        // Script files — use range-aware AST walking with a fresh AST
        match self.get_or_parse_ast(&uri).await {
            Some((script, _)) => {
                let ctx = self.build_semantic_token_context();
                let result = semantic_tokens::get_semantic_tokens_range(&script, &ctx, &range);
                match result {
                    SemanticTokensResult::Tokens(t) => {
                        Ok(Some(SemanticTokensRangeResult::Tokens(t)))
                    }
                    _ => Ok(None),
                }
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
        } else if params.command == "hoi4/getColorCodes" {
            let color_codes: std::collections::HashMap<String, String> = self
                .scanner_data
                .color_codes
                .iter()
                .map(|entry| {
                    let cc = entry.value().resolve();
                    let hex = format!("#{:02X}{:02X}{:02X}", cc.rgb.0, cc.rgb.1, cc.rgb.2);
                    (cc.symbol.clone(), hex)
                })
                .collect();
            let json = serde_json::to_value(&color_codes).unwrap();
            return Ok(Some(json));
        }
        Ok(None)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri.as_str();

        if let Some((script, _)) = self.ensure_ast_cached(uri) {
            let symbols =
                document_symbols::generate_document_symbols(&script.entries, &script.source);
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

/// Post-filter a delta-encoded SemanticTokensResult by LSP line range.
/// Decodes the delta-encoded tokens, filters to only those within `range`,
/// and re-encodes with fresh delta positions.
fn filter_semantic_tokens_by_range(
    result: SemanticTokensResult,
    range: &Range,
) -> SemanticTokensRangeResult {
    match result {
        SemanticTokensResult::Tokens(tokens) => {
            // Decode delta-encoded tokens to absolute positions
            let mut absolute: Vec<(u32, u32, u32, u32)> = Vec::new();
            let mut last_line = 0u32;
            let mut last_start = 0u32;
            for st in &tokens.data {
                let line = last_line + st.delta_line;
                let col = if st.delta_line == 0 {
                    last_start + st.delta_start
                } else {
                    st.delta_start
                };
                absolute.push((line, col, st.length, st.token_type));
                last_line = line;
                last_start = col;
            }

            // Filter by line range
            absolute.retain(|(line, _, _, _)| *line >= range.start.line && *line <= range.end.line);

            // Re-delta-encode the filtered tokens
            let mut data = Vec::with_capacity(absolute.len());
            last_line = 0;
            last_start = 0;
            for (line, col, len, ttype) in absolute {
                let delta_line = line - last_line;
                let delta_col = if delta_line == 0 {
                    col - last_start
                } else {
                    col
                };
                data.push(SemanticToken {
                    delta_line,
                    delta_start: delta_col,
                    length: len,
                    token_type: ttype,
                    token_modifiers_bitset: 0,
                });
                last_line = line;
                last_start = col;
            }

            SemanticTokensRangeResult::Tokens(SemanticTokens {
                result_id: None,
                data,
            })
        }
        SemanticTokensResult::Partial(_) => SemanticTokensRangeResult::Tokens(SemanticTokens {
            result_id: None,
            data: vec![],
        }),
    }
}
