use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use arc_swap::ArcSwap;
use rustc_hash::FxHashMap;

use dashmap::DashMap;
use tokio_util::sync::CancellationToken;
use tower_lsp_server::Client;
use tower_lsp_server::ls_types::*;

use crate::config::Config;
use crate::data::entity_lookup;
use crate::data::interner::InternedStr;
use crate::data::scanner_data::ScannerData;
use crate::lsp::semantic_tokens;
use crate::parser::ast;
use crate::parser::loc_parser;
use crate::parser::parser;
use crate::rules;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope;
use crate::utf16_len;
use crate::utils::lsp_convert::{ast_range_to_lsp, ast_related_info_to_lsp, ast_tag_to_lsp};
use crate::validation::advanced_validation;
use crate::{EFFECTS, MODIFIERS, SCOPES, TRIGGERS};

pub(crate) struct Backend {
    pub(crate) client: Client,
    pub(crate) documents: DashMap<String, String>,
    pub(crate) document_asts: DashMap<String, (Arc<ast::Script>, Vec<(String, ast::Range)>)>,
    pub(crate) document_cancellation_tokens: DashMap<String, CancellationToken>,
    pub(crate) scanner_data: ScannerData,
    pub(crate) config: Config,
    pub(crate) system_info: Mutex<sysinfo::System>,
    pub(crate) workspace_roots: Mutex<Vec<std::path::PathBuf>>,
    /// Static token keywords — computed once from TRIGGERS, EFFECTS, MODIFIERS, SCOPES,
    /// and the hardcoded keyword list. Never changes at runtime, so stored as Arc for
    /// cheap cloning on every semantic token request.
    pub(crate) static_token_keywords: Arc<HashSet<String>>,
    /// Entity names from scanner data — changes when the workspace is rescanned.
    /// Stored in ArcSwap so semantic tokens rebuilds are cheap (Arc::clone + HashMap::clone
    /// of the inner value) without iterating all scanner DashMaps on every keystroke.
    pub(crate) entity_token_context: ArcSwap<HashMap<String, entity_lookup::EntityKind>>,
}

impl Backend {
    /// Send a log message to the client, filtered by the current log level.
    /// Messages below the configured level are silently dropped.
    pub(crate) async fn log_msg(&self, level: crate::log_level::LogLevel, msg: impl Into<String>) {
        if level > self.config.log_level() {
            return;
        }
        self.client
            .log_message(
                tower_lsp_server::ls_types::MessageType::INFO,
                format!("[{}] {}", level.prefix(), msg.into()),
            )
            .await;
    }

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
    pub(crate) fn cache_ast(
        &self,
        uri: &str,
        content: &str,
    ) -> (Arc<ast::Script>, Vec<(String, ast::Range)>) {
        let (script, errors) = parser::parse_script(content);
        let script = Arc::new(script);
        self.document_asts
            .insert(uri.to_string(), (script.clone(), errors.clone()));
        (script, errors)
    }

    /// Get cached AST for a URI, or parse+cache from document text if missing.
    pub(crate) fn ensure_ast_cached(
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

    /// Get an AST that is guaranteed to match the current document content.
    ///
    /// If the cached AST is stale (document was edited after the last parse),
    /// re-parses immediately on a blocking thread and updates the cache.
    /// This prevents semantic token highlighting from lagging behind the editor
    /// during the `did_change` debounce window (where `self.documents` has been
    /// updated but `self.document_asts` hasn't been re-parsed yet).
    pub(crate) async fn get_or_parse_ast(
        &self,
        uri: &str,
    ) -> Option<(Arc<ast::Script>, Vec<(String, ast::Range)>)> {
        // Fast path: check if cached AST is fresh by comparing source text
        // with current document content (BOM-stripped, since the parser strips it).
        if let Some(cached) = self.document_asts.get(uri) {
            let is_fresh = self.documents.get(uri).is_some_and(|content| {
                let content_clean = content.strip_prefix('\u{feff}').unwrap_or(&content);
                &*cached.0.source == content_clean
            });
            if is_fresh {
                return Some((cached.0.clone(), cached.1.clone()));
            }
        }

        // Stale or no cache — re-parse from current document content.
        let content = self.documents.get(uri)?.clone();
        let (script, errors) = tokio::task::spawn_blocking(move || parser::parse_script(&content))
            .await
            .ok()?;
        let script = Arc::new(script);
        let result = (script.clone(), errors.clone());
        self.document_asts.insert(uri.to_string(), (script, errors));
        Some(result)
    }

    pub(crate) async fn find_references_in_root(
        &self,
        root: &std::path::Path,
        identifier: &str,
        locations: &mut Vec<Location>,
    ) {
        let extensions = ["txt", "yml", "gfx", "gui", "asset"];
        let filter = self.get_sync_filter();
        let files = crate::utils::fs_util::collect_files(root, &extensions, filter, false);

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
        move |path| crate::utils::fs_util::is_path_ignored(path, &ignored)
    }

    pub(crate) async fn validate_workspace(&self, root: &std::path::Path) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("Starting workspace diagnostic scan in: {:?}", root),
            )
            .await;

        let extensions = ["txt", "yml", "csv"];
        let filter = self.get_sync_filter();
        let files = crate::utils::fs_util::collect_files(root, &extensions, filter, true);
        let total = files.len();
        let mut file_count = 0;
        let scan_start = std::time::Instant::now();
        let mut last_log = scan_start;

        self.client
            .log_message(
                MessageType::INFO,
                format!("Workspace scan collecting {} files...", total),
            )
            .await;

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

                        // Log progress every 100 files or every 5 seconds
                        if file_count % 100 == 0
                            || last_log.elapsed() > std::time::Duration::from_secs(5)
                        {
                            let pct = (file_count as f64 / total as f64 * 100.0) as u32;
                            self.client
                                .log_message(
                                    MessageType::INFO,
                                    format!(
                                        "Workspace scan: {}/{} files ({}%) — {}",
                                        file_count,
                                        total,
                                        pct,
                                        path.file_name()
                                            .map(|n| n.to_string_lossy())
                                            .unwrap_or_default(),
                                    ),
                                )
                                .await;
                            last_log = std::time::Instant::now();
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

    pub(crate) async fn collect_workspace_files(&self, roots: &[std::path::PathBuf]) {
        let mut all_files = HashSet::new();
        let extensions = ["txt", "yml"];

        for root in roots {
            let files = crate::utils::fs_util::collect_files(
                root,
                &extensions,
                self.get_sync_filter(),
                true,
            );
            for path in &files {
                if let Ok(abs_path) = path.canonicalize() {
                    all_files.insert(abs_path.to_string_lossy().to_string());
                }
            }
        }

        self.scanner_data.workspace_files.clear();
        for f in all_files {
            self.scanner_data.workspace_files.insert(f.into());
        }
    }

    pub(crate) async fn validate_document(&self, uri: Uri) {
        let content = match self.documents.get(uri.as_str()) {
            Some(c) => c.clone(),
            _ => {
                return;
            }
        };

        let start = std::time::Instant::now();
        let diagnostics = self.validate_content(&uri, &content).await;
        let elapsed = start.elapsed();

        // Log slow validations (>500ms) at DEBUG level so users can
        // diagnose which files are slow without cluttering default INFO.
        if elapsed > std::time::Duration::from_millis(500) {
            let path = uri
                .to_file_path()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            let diag_types: std::collections::HashSet<&str> = diagnostics
                .iter()
                .filter_map(|d| {
                    d.code
                        .as_ref()
                        .and_then(|c| match c {
                            tower_lsp_server::ls_types::NumberOrString::String(s) => {
                                Some(s.as_str())
                            }
                            _ => None,
                        })
                        .or_else(|| {
                            if d.message.contains("styling")
                                || d.message.contains("trailing")
                                || d.message.contains("indent")
                                || d.message.contains("brace")
                            {
                                Some("styling")
                            } else {
                                None
                            }
                        })
                })
                .collect();
            self.log_msg(
                crate::log_level::LogLevel::Debug,
                format!(
                    "Validated {} in {:.1?} — {} diags ({:?})",
                    path,
                    elapsed,
                    diagnostics.len(),
                    diag_types,
                ),
            )
            .await;
        }

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    async fn validate_content(&self, uri: &Uri, content: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Skip known-ignored files: internal Paradox data, gfx/fonts credits, etc.
        if let Some(path) = uri.to_file_path() {
            if crate::utils::fs_util::is_known_ignored_file(&path) {
                return diagnostics;
            }
        }

        // Check for .txt files in events/ subdirectories — the game does not load them
        if uri.as_str().ends_with(".txt") {
            let uri_str = uri.as_str();
            if let Some(events_pos) = uri_str.find("/events/") {
                let after_events = &uri_str[events_pos + 8..]; // skip past "/events/"
                if after_events.contains('/') {
                    diagnostics.push(tower_lsp_server::ls_types::Diagnostic {
                        range: tower_lsp_server::ls_types::Range {
                            start: tower_lsp_server::ls_types::Position {
                                line: 0,
                                character: 0,
                            },
                            end: tower_lsp_server::ls_types::Position {
                                line: 0,
                                character: 0,
                            },
                        },
                        severity: Some(tower_lsp_server::ls_types::DiagnosticSeverity::ERROR),
                        message: "Files in events/ subdirectories are not loaded by the game. \
                                  Move this file directly into events/ for it to be recognised."
                            .to_string(),
                        code: Some(tower_lsp_server::ls_types::NumberOrString::String(
                            crate::validation::advanced_validation::EVENTS_SUBDIRECTORY_FILE
                                .to_string(),
                        )),
                        source: Some("Hearts of Modding".to_string()),
                        ..Default::default()
                    });
                    return diagnostics;
                }
            }
        }

        let styling_enabled = self.config.styling_enabled();
        let mut script_opt: Option<Arc<ast::Script>> = None;
        let map_config = crate::utils::map_config::get_map_config(std::path::Path::new("."));

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
            // Only cache the AST if the document is actively open in the editor.
            // Workspace-scanned files are parsed but NOT cached — they'd leak RAM
            // since VS Code never sends did_close for them.
            let (script, parse_errors) = if self.documents.contains_key(uri.as_str()) {
                self.ensure_ast_cached(uri.as_str())
                    .unwrap_or_else(|| self.cache_ast(uri.as_str(), content))
            } else {
                let (s, e) = parser::parse_script(content);
                (Arc::new(s), e)
            };
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
        } else if uri.as_str().ends_with(".csv") {
            // Do not parse other CSV files as clausewitz scripts
        } else {
            // Only cache the AST if the document is actively open in the editor.
            // Workspace-scanned files are parsed but NOT cached — they'd leak RAM
            // since VS Code never sends did_close for them.
            let (script, parse_errors) = if self.documents.contains_key(uri.as_str()) {
                self.ensure_ast_cached(uri.as_str())
                    .unwrap_or_else(|| self.cache_ast(uri.as_str(), content))
            } else {
                let (s, e) = parser::parse_script(content);
                (Arc::new(s), e)
            };
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
        let provs = &self.scanner_data.provinces;
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
        let provs = &self.scanner_data.provinces;
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
        let states = &self.scanner_data.states;
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
        let regions = &self.scanner_data.strategic_regions;
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
        let provs = &self.scanner_data.provinces;
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
        // Build known terrain category names from scanned data, falling back to
        // vanilla HOI4 defaults if no mod/game terrain files were scanned.
        // This ensures province terrain validation works even without a game path.
        let terrain_names: HashSet<String> = {
            let mut names: HashSet<String> = self
                .scanner_data
                .terrain_categories
                .iter()
                .map(|entry| entry.key().to_string())
                .collect();
            if names.is_empty() {
                // Fallback: vanilla HOI4 terrain categories (stable across versions)
                names.insert("unknown".to_string());
                names.insert("ocean".to_string());
                names.insert("forest".to_string());
                names.insert("hills".to_string());
                names.insert("mountain".to_string());
                names.insert("plains".to_string());
                names.insert("urban".to_string());
                names.insert("jungle".to_string());
                names.insert("marsh".to_string());
                names.insert("desert".to_string());
                names.insert("lakes".to_string());
                names.insert("water_fjords".to_string());
                names.insert("water_shallow_sea".to_string());
                names.insert("water_deep_ocean".to_string());
            }
            names
        };

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

            // Validate terrain column (parts[6]) against known terrain categories
            if let Some(diag) = check_province_terrain_csv(
                parts[6],
                &terrain_names,
                i as u32,
                {
                    let mut col = 0;
                    for part in parts.iter().take(6) {
                        col += part.len() as u32 + 1;
                    }
                    col
                },
                parts[6].len() as u32,
            ) {
                diagnostics.push(diag);
            }
        }
    }

    async fn validate_adjacencies_content(&self, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        let provs = &self.scanner_data.provinces;
        let rules = &self.scanner_data.adjacency_rules;
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
        let provs = &self.scanner_data.provinces;
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
                if ass.key_text(content).eq_ignore_ascii_case("adjacency_rule") {
                    if let ast::Value::Block(rule_entries) = &ass.value.value {
                        for rule_entry in rule_entries {
                            if let ast::Entry::Assignment(r_ass) = rule_entry {
                                if r_ass
                                    .key_text(content)
                                    .eq_ignore_ascii_case("required_provinces")
                                {
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
        let provs = &self.scanner_data.provinces;

        for entry in &script.entries {
            if let ast::Entry::Assignment(ass) = entry {
                if ass
                    .key_text(&script.source)
                    .eq_ignore_ascii_case("strategic_region")
                {
                    if let ast::Value::Block(region_entries) = &ass.value.value {
                        for region_entry in region_entries {
                            if let ast::Entry::Assignment(r_ass) = region_entry {
                                if r_ass
                                    .key_text(&script.source)
                                    .eq_ignore_ascii_case("provinces")
                                {
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
        let event_targets = &self.scanner_data.event_targets;
        let scripted_locs = &self.scanner_data.scripted_locs;
        let color_codes = &self.scanner_data.color_codes;
        let dups = &self.scanner_data.duplicated_loc_keys;
        let game_loc_keys = &self.scanner_data.game_loc_keys;

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

        // Pre-build these sets once before the entry loop instead of rebuilding
        // them on every iteration (1280 × 610 = 780k+ DashMap lookups per file).
        let color_code_set: std::collections::HashSet<String> =
            color_codes.iter().map(|e| e.key().to_string()).collect();
        let country_tag_set: std::collections::HashSet<String> = self
            .scanner_data
            .country_tags
            .iter()
            .map(|e| e.key().to_string())
            .collect();

        for (entry_idx, entry) in parsed.values().enumerate() {
            // Yield to the async executor periodically so that large loc files
            // (e.g. countries_l_english.yml with 5688 entries) don't block LSP
            // request handling during workspace scans.
            if entry_idx > 0 && entry_idx % 500 == 0 {
                tokio::task::yield_now().await;
            }

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

            let loc_diagnostics = loc_parser::validate_loc_string(
                entry,
                event_targets,
                scripted_locs,
                &color_code_set,
                &country_tag_set,
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
            let is_duplicated = dups.contains(&(doc_lang_str.clone().into(), entry.key.clone()));

            if is_duplicated {
                // If the key exists in the vanilla game's localization files,
                // it's an intentional override — no warning needed.
                let is_game_override =
                    game_loc_keys.contains(&(doc_lang_str.clone().into(), entry.key.clone()));

                if !is_game_override {
                    let loc_map = &self.scanner_data.localization;
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
    }

    /// Compatibility wrapper — delegates to the combined `check_styling_ast`.
    ///
    /// Kept for tests in `tests/formatting.rs` that call this function directly.
    /// The combined walk emits both assignment and brace spacing diagnostics,
    /// matching the superset of what the old standalone function produced.
    #[allow(dead_code)]
    pub(crate) fn check_assignment_spacing(
        entries: &[ast::Entry],
        content: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let lines: Vec<&str> = content.lines().collect();
        Self::check_styling_ast(entries, &lines, diagnostics, &mut HashMap::new(), 0);
    }

    /// Compute expected indentation depth for each line in the AST.
    ///
    /// Used by the "Fix all indentation" code action in `validation/formatting.rs`.
    /// Standalone function (not combined with other styling checks) because it's
    /// triggered only on explicit user action, not on every keystroke.
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

    /// Combined AST walk for styling checks (per-keystroke path).
    ///
    /// Performs a single recursive traversal of the AST to:
    /// 1. Compute expected indentation depths
    /// 2. Check single-line brace spacing
    /// 3. Check assignment operator spacing
    ///
    /// Uses a pre-cached lines slice for O(1) line lookups instead of
    /// `content.lines().nth()` O(n) scans.
    fn check_styling_ast(
        entries: &[ast::Entry],
        lines: &[&str],
        diagnostics: &mut Vec<Diagnostic>,
        expected_indents: &mut HashMap<u32, usize>,
        depth: usize,
    ) {
        for entry in entries {
            let start_line = match entry {
                ast::Entry::Assignment(ass) => ass.key_range.start_line,
                ast::Entry::Value(val) => val.range.start_line,
                ast::Entry::Comment(_, r) => r.start_line,
            };
            expected_indents.entry(start_line).or_insert(depth);

            match entry {
                ast::Entry::Assignment(ass) => {
                    // ── Assignment spacing check ──
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
                        let line_idx = ass.key_range.end_line as usize;
                        if let Some(line) = lines.get(line_idx) {
                            let start = ass.key_range.end_col as usize;
                            let end = ass.value.range.start_col as usize;
                            if start <= end && end <= line.len() {
                                diagnostics.push(Diagnostic {
                                    range: Range {
                                        start: Position {
                                            line: ass.key_range.end_line,
                                            character: start as u32,
                                        },
                                        end: Position {
                                            line: ass.value.range.start_line,
                                            character: end as u32,
                                        },
                                    },
                                    severity: Some(DiagnosticSeverity::INFORMATION),
                                    code: Some(NumberOrString::String(
                                        "styling_assignment_space".to_string(),
                                    )),
                                    message: "Assignment operator should be surrounded by exactly one space on each side (e.g., 'key = value')."
                                        .to_string(),
                                    source: Some("Hearts of Modding".to_string()),
                                    ..Default::default()
                                });
                            }
                        }
                    }

                    // ── Single-line brace spacing check ──
                    Self::check_brace_spacing_for_range_slice(
                        &ass.value.range,
                        &ass.value.value,
                        lines,
                        diagnostics,
                    );

                    // ── Recurse into child blocks ──
                    match &ass.value.value {
                        ast::Value::Block(inner) => {
                            Self::check_styling_ast(
                                inner,
                                lines,
                                diagnostics,
                                expected_indents,
                                depth + 1,
                            );
                            let end_line = ass.value.range.end_line;
                            if end_line != start_line {
                                expected_indents.entry(end_line).or_insert(depth);
                            }
                        }
                        ast::Value::TaggedBlock(_, inner, _) => {
                            Self::check_styling_ast(
                                inner,
                                lines,
                                diagnostics,
                                expected_indents,
                                depth + 1,
                            );
                            let end_line = ass.value.range.end_line;
                            if end_line != start_line {
                                expected_indents.entry(end_line).or_insert(depth);
                            }
                        }
                        _ => {}
                    }
                }
                ast::Entry::Value(val) => {
                    // ── Single-line brace spacing check ──
                    Self::check_brace_spacing_for_range_slice(
                        &val.range,
                        &val.value,
                        lines,
                        diagnostics,
                    );

                    // ── Recurse into child blocks ──
                    match &val.value {
                        ast::Value::Block(inner) => {
                            Self::check_styling_ast(
                                inner,
                                lines,
                                diagnostics,
                                expected_indents,
                                depth + 1,
                            );
                            let end_line = val.range.end_line;
                            if end_line != start_line {
                                expected_indents.entry(end_line).or_insert(depth);
                            }
                        }
                        ast::Value::TaggedBlock(_, inner, _) => {
                            Self::check_styling_ast(
                                inner,
                                lines,
                                diagnostics,
                                expected_indents,
                                depth + 1,
                            );
                            let end_line = val.range.end_line;
                            if end_line != start_line {
                                expected_indents.entry(end_line).or_insert(depth);
                            }
                        }
                        _ => {}
                    }
                }
                ast::Entry::Comment(_, _) => {}
            }
        }
    }

    /// Check single-line brace spacing, using a pre-cached lines slice.
    ///
    /// Accepts `&[&str]` instead of `&str` to avoid O(n) `content.lines().nth()` calls.
    /// Logic is identical to the old `check_brace_spacing_for_range` which accepted
    /// `content: &str` and scanned from the start on every call.
    fn check_brace_spacing_for_range_slice(
        range: &ast::Range,
        value: &ast::Value,
        lines: &[&str],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match value {
            ast::Value::Block(_) | ast::Value::TaggedBlock(_, _, _)
                if range.start_line == range.end_line =>
            {
                let line_idx = range.start_line as usize;
                if let Some(line) = lines.get(line_idx) {
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
        // Pre-cache lines for O(1) index lookups — avoids content.lines().nth() O(n) scans
        // that were the main performance bottleneck in styling checks on large files.
        let lines: Vec<&str> = content.lines().collect();

        if !content.is_empty()
            && !content.ends_with('\n')
            && !content.ends_with("\r\n")
            && !uri.ends_with("map/buildings.txt")
        {
            let line_count = lines.len();
            let last_line = lines.last().copied().unwrap_or("");
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

        // Combined AST walk: indentations, single-line braces, assignment spacing.
        // Replaces 3 separate recursive walks (compute_expected_indentations,
        // check_single_line_braces, check_assignment_spacing) with a single traversal.
        // Also passes the pre-cached lines slice so helpers get O(1) line lookups.
        let mut expected_indents = HashMap::new();
        if let Some(script) = script_opt {
            Self::check_styling_ast(
                &script.entries,
                &lines,
                diagnostics,
                &mut expected_indents,
                0,
            );
        }

        for (line_idx, line) in lines.iter().enumerate() {
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
        let loc = &self.scanner_data.localization;
        let st = &self.scanner_data.scripted_triggers;
        let se = &self.scanner_data.scripted_effects;
        let id = &self.scanner_data.ideologies;
        let sid = &self.scanner_data.sub_ideologies;
        let tr = &self.scanner_data.traits;
        let sp = &self.scanner_data.sprites;
        let ids = &self.scanner_data.ideas;
        let provs = &self.scanner_data.provinces;
        let mod_maps = &self.scanner_data.modifier_mappings;
        let ig_loc = self.config.ignored_loc_regex();
        let buildings = &self.scanner_data.buildings;
        let resources = &self.scanner_data.resources;
        let state_categories = &self.scanner_data.state_categories;
        let s_effects = &self.scanner_data.sound_effects;
        let ct = &self.scanner_data.country_tags;

        let mut comments = Vec::new();
        for entry in &script.entries {
            if let ast::Entry::Comment(c, r) = entry {
                comments.push((*c, r.clone()));
            }
        }

        // Detect file type from URI for scope inference
        let initial_scope = if uri.contains("/common/abilities/") {
            scope::Scope::Character
        } else {
            scope::Scope::Global
        };

        let game_path = self.config.game_path();

        // Lock workspace roots for texture path resolution
        let workspace_roots = self.workspace_roots.lock().unwrap();

        // Build validation context
        let ctx = ValidationContext {
            uri,
            source: &script.source,
            loc,
            scripted_triggers: st,
            scripted_effects: se,
            ideologies: id,
            sub_ideologies: sid,
            traits: tr,
            sprites: sp,
            ideas: ids,
            provinces: provs,
            modifier_mappings: mod_maps,
            ignored_loc_regex: &ig_loc,
            comments: &comments,
            sound_effects: s_effects,
            country_tags: ct,
            buildings,
            resources,
            state_categories,
            continents: &self.scanner_data.continents,
            strategic_regions: &self.scanner_data.strategic_regions,
            terrain_categories: &self.scanner_data.terrain_categories,
            abilities: &self.scanner_data.abilities,
            game_path,
            styling_enabled,
            workspace_roots: &workspace_roots,
            unit_types: &self.scanner_data.unit_types,
            event_namespaces: &self.scanner_data.event_namespaces,
            events: &self.scanner_data.events,
            decisions: &self.scanner_data.decisions,
            decision_categories: &self.scanner_data.decision_categories,
        };

        // ── AST visitors (single traversal, replaces per-rule recursion) ──
        let mut visitors: Vec<Box<dyn rules::visitor::AstVisitor>> = vec![
            rules::buildings::BuildingRule::visitor(),
            rules::portraits::PortraitRule::visitor(),
            rules::characters::CharacterRule::visitor(),
            rules::abilities::AbilityRule::visitor(),
            rules::ai_areas::AiAreaRule::visitor(uri),
            rules::provinces::ProvinceRule::vp_visitor(),
            rules::oob_regiments::OobRegimentVisitor::visitor(),
            rules::events::EventValidationRule::visitor(),
            rules::decisions::DecisionsRule::visitor(),
        ];

        // Rules that still use check_assignment / check_block
        let rules: Vec<Box<dyn ValidationRule>> = vec![
            Box::new(rules::achievements::AchievementRule),
            Box::new(rules::abilities::AbilityRule),
            Box::new(rules::country_tags::CountryTagRule),
            Box::new(rules::country_metadata::CountryMetadataRule),
            Box::new(rules::ideologies::IdeologyRule),
            Box::new(rules::ideas::IdeaRule),
            Box::new(rules::localization::LocalizationRule),
            Box::new(rules::provinces::ProvinceRule),
            Box::new(rules::sounds::SoundRule),
            Box::new(rules::sprites::SpriteRule),
            Box::new(rules::state_definitions::StateDefinitionRule),
            Box::new(rules::terrains::TerrainRule),
            Box::new(rules::traits::TraitRule),
            Box::new(rules::events::EventValidationRule),
            Box::new(rules::decisions::DecisionsRule),
        ];

        // Block-level rules: top-level entries only, NO recursion.
        // Rules that did their own recursion (BuildingRule, PortraitRule, etc.)
        // have been converted to AstVisitor and have empty check_block.
        for rule in &rules {
            rule.check_block(&script.entries, &ctx, diagnostics);
        }

        // ── One AST traversal ──
        // This single walk replaces the old per-rule recursive pattern:
        // check_block (with internal recursion) × N rules
        // + check_entry_semantic (with inline recursion)
        // + check_value_semantic (with recursive entry calls)
        //
        // Now: 1 traversal, N visitor hooks + M check_assignment calls.
        rules::visitor::walk_script(
            &script.entries,
            &mut visitors,
            &rules,
            &ctx,
            diagnostics,
            initial_scope,
            false,
        );

        // Texture file path validation for .gfx and .gui files
        if uri.ends_with(".gfx") || uri.ends_with(".gui") {
            if let Ok(url) = uri.parse::<Uri>() {
                if let Some(gfx_path) = url.to_file_path() {
                    let gfx_rule = rules::gfx_textures::GfxTextureRule::new(&gfx_path);
                    gfx_rule.validate(&script.entries, &ctx, diagnostics);
                }
            }
        }
    }

    /// Build the keyword + entity context used for semantic token resolution.
    ///
    /// Uses pre-computed static keywords (computed once at startup) merged with
    /// the current scanner entity map (updated on rescans via
    /// [`update_entity_token_context()`]).
    ///
    /// This used to rebuild the entire keyword set from scratch on every semantic
    /// token request, iterating all DashMaps — now it's just two cheap clones.
    pub(crate) fn build_semantic_token_context(&self) -> semantic_tokens::SemanticTokenContext {
        let keywords = (*self.static_token_keywords).clone();
        let entity_names = self.entity_token_context.load_full();
        semantic_tokens::SemanticTokenContext::new(keywords, (*entity_names).clone())
    }

    /// Refresh the entity name map from current scanner data.
    ///
    /// Call this after a full initial scan ([`crate::lsp::handler::Backend::initialized`])
    /// or after any rescan triggered by [`crate::lsp::handler::Backend::did_change_watched_files`].
    /// Without this, semantic tokens would continue using a stale entity context
    /// (entities added or removed by rescans wouldn't be highlighted).
    pub(crate) fn update_entity_token_context(&self) {
        let lookup = entity_lookup::EntityLookup::new(&self.scanner_data);
        let names = lookup.entity_names();
        self.entity_token_context.store(Arc::new(names));
    }
}

/// Pre-compute the static keyword set used for semantic token resolution.
///
/// This includes: all built-in triggers, effects, modifiers, scopes,
/// and the extensive hardcoded keyword list for character definitions,
/// abilities, focuses, events, OOB, terrain, ideologies, etc.
///
/// This function is called exactly once at startup (see `main.rs`). The
/// result is stored in `Backend::static_token_keywords` as `Arc<HashSet>`
/// so each semantic token rebuild is just a cheap `Arc::clone`.
pub(crate) fn build_static_semantic_keywords() -> HashSet<String> {
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

    // Structural block keywords (scope filter, conditional, option blocks)
    keywords.insert("limit".to_string());
    keywords.insert("else".to_string());
    keywords.insert("else_if".to_string());
    keywords.insert("option".to_string());
    keywords.insert("trigger".to_string());

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
    keywords.insert("hidden_effect".to_string());
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
    keywords.insert("color_ui".to_string());
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
    keywords.insert("country_event".to_string());
    keywords.insert("state_event".to_string());
    keywords.insert("news_event".to_string());
    keywords.insert("unit_leader_event".to_string());
    keywords.insert("operative_leader_event".to_string());
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

    // Bookmark definition keywords (common/bookmarks/*.txt)
    keywords.insert("bookmarks".to_string());
    keywords.insert("bookmark".to_string());
    keywords.insert("default_country".to_string());
    keywords.insert("effect".to_string());
    keywords.insert("minor".to_string());
    keywords.insert("ideas".to_string());
    keywords.insert("focuses".to_string());

    // Game rule keywords (common/game_rules/*.txt)
    keywords.insert("group".to_string());
    keywords.insert("required_dlc".to_string());
    keywords.insert("exclude_dlc".to_string());
    keywords.insert("allow_achievements".to_string());

    // Difficulty setting keywords (common/difficulty_settings/*.txt)
    keywords.insert("difficulty_settings".to_string());
    keywords.insert("difficulty_setting".to_string());
    keywords.insert("countries".to_string());
    keywords.insert("multiplier".to_string());

    // OOB (Order of Battle) keywords (history/units/*.txt)
    keywords.insert("division_template".to_string());
    keywords.insert("units".to_string());
    keywords.insert("air_wings".to_string());
    keywords.insert("amount".to_string());
    keywords.insert("instant_effect".to_string());
    keywords.insert("regiments".to_string());
    keywords.insert("support".to_string());
    keywords.insert("division_names_group".to_string());
    keywords.insert("is_locked".to_string());
    keywords.insert("force_allow_recruiting".to_string());
    keywords.insert("division_cap".to_string());
    keywords.insert("template_counter".to_string());
    keywords.insert("override_model".to_string());
    keywords.insert("division_name".to_string());
    keywords.insert("is_name_ordered".to_string());
    keywords.insert("name_order".to_string());
    keywords.insert("start_experience_factor".to_string());
    keywords.insert("start_equipment_factor".to_string());
    keywords.insert("start_manpower_factor".to_string());
    keywords.insert("force_equipment_variants".to_string());
    keywords.insert("officer".to_string());
    keywords.insert("division".to_string());
    keywords.insert("location".to_string());
    keywords.insert("fleet".to_string());
    keywords.insert("naval_base".to_string());
    keywords.insert("task_force".to_string());
    keywords.insert("pride_of_the_fleet".to_string());
    keywords.insert("ship".to_string());
    keywords.insert("definition".to_string());
    keywords.insert("add_equipment_production".to_string());
    keywords.insert("requested_factories".to_string());
    keywords.insert("efficiency".to_string());
    keywords.insert("version_name".to_string());
    keywords.insert("creator".to_string());

    // lowk lazy to categorize
    keywords.insert("popularity".to_string());
    keywords.insert("ruling_party".to_string());
    keywords.insert("elections_allowed".to_string());
    keywords.insert("size".to_string());
    keywords.insert("transfer_troops".to_string());
    keywords.insert("autonomy_state".to_string());

    keywords
}

/// Check for duplicate modifier keys within a block of entries.
/// This was previously a method on `Backend`, now a free function called
/// by the centralized AST walker for every block entry.
pub(crate) fn check_duplicate_keys<'a>(
    entries: &[ast::Entry],
    diagnostics: &mut Vec<Diagnostic>,
    mod_maps: &DashMap<InternedStr, String>,
    source: &'a str,
    in_air_wings: bool,
    parent_key: Option<&'a str>,
) {
    // Currently only checks keys that are in `mod_maps` (modifier names) plus a small
    // hardcoded set of common structural keys (`name`, `id`, `icon`). All other keys
    // (e.g. arbitrary custom keys from mods) are silently allowed. To extend coverage,
    // add more entries to `COMMON_KEYS` or replace the hardcoded list with a configurable
    // set of key patterns.
    const COMMON_KEYS: [&str; 3] = ["name", "id", "icon"];

    let mut seen_keys: FxHashMap<&'a str, ast::Range> = FxHashMap::default();

    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            // We only care about duplicates if they are modifiers.
            // Some Paradox keys (like 'modifier = { ... }' or 'option = { ... }') are intended to be duplicates.
            // But specific engine modifiers (like 'stability_factor') should NEVER be duplicated.

            let key: &'a str = ass.key_text(source);
            let is_modifier = mod_maps.contains_key(key) || COMMON_KEYS.contains(&key);

            // Exceptions: Some effects/triggers are specifically designed to be used multiple times
            // In air_wings province blocks, 'name' keys are used to label each preceding
            // equipment type block, so duplicates are valid.
            // 'icon' is used structurally in army_icons.txt (and similar files) where
            // each `icon = { ... }` block is a separate entry keyed by list position.
            // Inside `province` blocks, `id` values are list items (provinces to apply
            // a modifier to), not genuine duplicates.
            let is_exception = key == "modifier"
                || key == "option"
                || key == "limit"
                || key == "if"
                || key == "else"
                || key == "else_if"
                || key == "variable_name"
                || key == "icon"
                || (in_air_wings && key == "name")
                || (parent_key == Some("province") && key == "id");

            if is_modifier && !is_exception {
                if let Some(prev_range) = seen_keys.get(key) {
                    diagnostics.push(Diagnostic {
                            range: ast_range_to_lsp(prev_range),
                            severity: Some(DiagnosticSeverity::WARNING),
                            code: Some(NumberOrString::String("duplicate_key".to_string())),
                            message: format!("Duplicate modifier/key '{}' detected in the same scope. The game will ignore this value and use the last one.", key),
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
                seen_keys.insert(key, full_range);
            }
        }
    }
}

/// Check a single terrain value from definition.csv against known terrain
/// categories. Returns `Some(Diagnostic)` if the terrain is unknown.
pub(crate) fn check_province_terrain_csv(
    terrain_value: &str,
    terrain_names: &HashSet<String>,
    line: u32,
    col_start: u32,
    col_len: u32,
) -> Option<Diagnostic> {
    let lower = terrain_value.trim().to_lowercase();
    if !lower.is_empty() && !terrain_names.contains(&lower) {
        Some(Diagnostic {
            range: Range {
                start: Position {
                    line,
                    character: col_start,
                },
                end: Position {
                    line,
                    character: col_start + col_len,
                },
            },
            severity: Some(DiagnosticSeverity::WARNING),
            message: format!(
                "Unknown terrain '{}'. Terrains are defined in common/terrain/*.txt",
                lower,
            ),
            code: Some(NumberOrString::String(
                crate::validation::advanced_validation::UNKNOWN_PROVINCE_TERRAIN.to_string(),
            )),
            source: Some("Hearts of Modding".to_string()),
            ..Default::default()
        })
    } else {
        None
    }
}

/// Collect all unit type casing mismatches in the AST for bulk-fix code actions.
///
/// Walks the AST looking for `regiments = { ... }` and `support = { ... }` blocks,
/// then checks each child entry's key against the known `unit_types` DashMap.
/// When a key exists case-insensitively but with different casing, the fix
/// (key range + canonical text) is collected.
///
/// When `specific_type` is `Some(key)`, only fixes for that specific canonical
/// unit type are collected (for per-unit-type bulk fixes). When `None`, all
/// mismatches are collected.
impl Backend {
    pub(crate) fn collect_unit_type_casing_fixes(
        &self,
        entries: &[ast::Entry],
        fixes: &mut Vec<(ast::Range, String)>,
        source: &str,
        specific_type: Option<&str>,
    ) {
        for entry in entries {
            match entry {
                ast::Entry::Assignment(ass) => {
                    let key_lower = ass.key_text(source).to_ascii_lowercase();

                    // Track when we're inside regiments/support blocks
                    let in_slot = matches!(key_lower.as_str(), "regiments" | "support")
                        && matches!(
                            &ass.value.value,
                            ast::Value::Block(_) | ast::Value::TaggedBlock(..)
                        );

                    // When inside a slot block, check each child with a Block
                    // value as a unit type reference
                    if in_slot {
                        if let ast::Value::Block(inner) = &ass.value.value {
                            for child in inner {
                                if let ast::Entry::Assignment(child_ass) = child {
                                    if matches!(
                                        &child_ass.value.value,
                                        ast::Value::Block(_) | ast::Value::TaggedBlock(..)
                                    ) {
                                        let child_key = child_ass.key_text(source);

                                        // Tier 1: exact match → skip
                                        if self.scanner_data.unit_types.contains_key(child_key) {
                                            continue;
                                        }

                                        // Tier 2: case-insensitive match → fix
                                        if let Some(canonical) =
                                            crate::scanner::unit_scanner::find_canonical_unit_type(
                                                &self.scanner_data.unit_types,
                                                child_key,
                                            )
                                        {
                                            let matches_specific = specific_type
                                                .map(|s| s == canonical.as_str())
                                                .unwrap_or(true);
                                            if matches_specific {
                                                fixes
                                                    .push((child_ass.key_range.clone(), canonical));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Recurse into child blocks
                    match &ass.value.value {
                        ast::Value::Block(inner) => {
                            self.collect_unit_type_casing_fixes(
                                inner,
                                fixes,
                                source,
                                specific_type,
                            );
                        }
                        ast::Value::TaggedBlock(_, inner, _) => {
                            self.collect_unit_type_casing_fixes(
                                inner,
                                fixes,
                                source,
                                specific_type,
                            );
                        }
                        _ => {}
                    }
                }
                ast::Entry::Value(val) => match &val.value {
                    ast::Value::Block(inner) => {
                        self.collect_unit_type_casing_fixes(inner, fixes, source, specific_type);
                    }
                    ast::Value::TaggedBlock(_, inner, _) => {
                        self.collect_unit_type_casing_fixes(inner, fixes, source, specific_type);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_province_terrain_csv_invalid() {
        let mut names = HashSet::new();
        names.insert("ocean".to_string());
        names.insert("forest".to_string());
        names.insert("plains".to_string());

        // Valid terrain should produce no diagnostic
        let result = check_province_terrain_csv("ocean", &names, 0, 0, 5);
        assert!(
            result.is_none(),
            "Valid terrain 'ocean' should not produce a diagnostic"
        );

        let result = check_province_terrain_csv("forest", &names, 1, 20, 6);
        assert!(
            result.is_none(),
            "Valid terrain 'forest' should not produce a diagnostic"
        );

        // Invalid terrain should be caught
        let result = check_province_terrain_csv("oceann", &names, 2, 15, 6);
        assert!(
            result.is_some(),
            "Invalid terrain 'oceann' SHOULD produce a diagnostic"
        );
        let diag = result.unwrap();
        assert_eq!(diag.range.start.line, 2);
        assert_eq!(diag.range.start.character, 15);
        assert!(diag.message.contains("oceann"));
        assert_eq!(
            diag.code,
            Some(NumberOrString::String(
                crate::validation::advanced_validation::UNKNOWN_PROVINCE_TERRAIN.to_string()
            ))
        );

        // Empty terrain should be ignored (no diagnostic)
        let result = check_province_terrain_csv("", &names, 3, 0, 0);
        assert!(
            result.is_none(),
            "Empty terrain should not produce a diagnostic"
        );

        // Whitespace-only terrain should be ignored
        let result = check_province_terrain_csv("  ", &names, 4, 0, 2);
        assert!(
            result.is_none(),
            "Whitespace terrain should not produce a diagnostic"
        );
    }
}
