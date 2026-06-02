use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use dashmap::DashMap;
use tower_lsp_server::Client;
use tower_lsp_server::ls_types::*;

use crate::advanced_validation;
use crate::ast;
use crate::config::Config;
use crate::interner::InternedStr;
use crate::loc_parser;
use crate::lsp_convert::{ast_range_to_lsp, ast_related_info_to_lsp, ast_tag_to_lsp};
use crate::parser;
use crate::rules;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scanner_data::ScannerData;
use crate::scope;
use crate::utf16_len;

pub(crate) struct Backend {
    pub(crate) client: Client,
    pub(crate) documents: DashMap<String, String>,
    pub(crate) document_asts: DashMap<String, (Arc<ast::Script>, Vec<(String, ast::Range)>)>,
    pub(crate) scanner_data: ScannerData,
    pub(crate) config: Config,
    pub(crate) system_info: Mutex<sysinfo::System>,
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

    pub(crate) async fn find_references_in_root(
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

    pub(crate) async fn validate_workspace(&self, root: &std::path::Path) {
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

    pub(crate) async fn collect_workspace_files(&self, roots: &[std::path::PathBuf]) {
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
        let provs = &self.scanner_data.provinces;

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
                color_codes.iter().map(|e| e.key().to_string()).collect();
            let country_tag_set: std::collections::HashSet<String> = self
                .scanner_data
                .country_tags
                .iter()
                .map(|e| e.key().to_string())
                .collect();
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
        let defines = self.scanner_data.defines();
        let s_effects = &self.scanner_data.sound_effects;
        let ct = &self.scanner_data.country_tags;

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

        let game_path = self.config.game_path();

        // Build validation context
        let ctx = ValidationContext {
            uri,
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
            defines: &defines,
            continents: &self.scanner_data.continents,
            strategic_regions: &self.scanner_data.strategic_regions,
            abilities: &self.scanner_data.abilities,
            game_path,
            styling_enabled,
        };

        // Collect all rules
        let rules: Vec<Box<dyn ValidationRule>> = vec![
            Box::new(rules::achievements::AchievementRule),
            Box::new(rules::abilities::AbilityRule),
            Box::new(rules::ai_areas::AiAreaRule),
            Box::new(rules::buildings::BuildingRule),
            Box::new(rules::characters::CharacterRule),
            Box::new(rules::country_tags::CountryTagRule),
            Box::new(rules::ideologies::IdeologyRule),
            Box::new(rules::ideas::IdeaRule),
            Box::new(rules::localization::LocalizationRule),
            Box::new(rules::portraits::PortraitRule),
            Box::new(rules::provinces::ProvinceRule),
            Box::new(rules::sounds::SoundRule),
            Box::new(rules::sprites::SpriteRule),
            Box::new(rules::traits::TraitRule),
        ];

        // Run block-level rules (check_block is called for each)
        for rule in &rules {
            rule.check_block(&script.entries, &ctx, diagnostics);
        }

        // Traverse entries with assignment rules
        let mut scope_stack = scope::ScopeStack::new(initial_scope);
        for entry in &script.entries {
            self.check_entry_semantic(entry, diagnostics, &ctx, &rules, &mut scope_stack);
        }

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

    fn check_entry_semantic(
        &self,
        entry: &ast::Entry,
        diagnostics: &mut Vec<Diagnostic>,
        ctx: &ValidationContext,
        rules: &[Box<dyn ValidationRule>],
        scope_stack: &mut scope::ScopeStack,
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
                        ast::Value::Block(_entries) | ast::Value::TaggedBlock(_, _entries, _) => {
                            scope_stack.push(s);
                            pushed_scope = true;
                        }
                        _ => {}
                    }
                }

                // Run all assignment-level rules (scope is pushed so rules see it)
                for rule in rules {
                    rule.check_assignment(ass, ctx, scope_stack, diagnostics);
                }

                // ── Styling checks (remain in backend.rs — not rule-extracted) ──

                // Casing checks for keywords
                if ctx.styling_enabled {
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
                                let tag_end_col = ass.value.range.start_col + tag.len() as u32;
                                if block_range.start_col != tag_end_col + 1 {
                                    diagnostics.push(Diagnostic {
                                        range: Range {
                                            start: Position { line: ass.value.range.start_line, character: tag_end_col },
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

                // Check value recursively
                self.check_value_semantic(&ass.value, diagnostics, ctx, rules, scope_stack);

                if pushed_scope {
                    scope_stack.pop();
                }
            }
            ast::Entry::Value(val) => {
                self.check_value_semantic(val, diagnostics, ctx, rules, scope_stack);
            }
            _ => {}
        }
    }

    fn check_value_semantic(
        &self,
        val: &ast::NodeedValue,
        diagnostics: &mut Vec<Diagnostic>,
        ctx: &ValidationContext,
        rules: &[Box<dyn ValidationRule>],
        scope_stack: &mut scope::ScopeStack,
    ) {
        match &val.value {
            ast::Value::Block(entries) => {
                self.check_duplicate_keys(entries, diagnostics, ctx.modifier_mappings);
                for entry in entries {
                    self.check_entry_semantic(entry, diagnostics, ctx, rules, scope_stack);
                }
            }
            ast::Value::TaggedBlock(tag, entries, block_range) => {
                if ctx.styling_enabled {
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
                self.check_duplicate_keys(entries, diagnostics, ctx.modifier_mappings);
                for entry in entries {
                    self.check_entry_semantic(entry, diagnostics, ctx, rules, scope_stack);
                }
            }
            _ => {}
        }
    }

    fn check_duplicate_keys(
        &self,
        entries: &[ast::Entry],
        diagnostics: &mut Vec<Diagnostic>,
        mod_maps: &DashMap<InternedStr, String>,
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

                let is_modifier = mod_maps.contains_key(ass.key.as_str())
                    || COMMON_KEYS.contains(&ass.key.as_str());

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
