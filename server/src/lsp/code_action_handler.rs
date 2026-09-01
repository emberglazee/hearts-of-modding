use crate::Backend;
use crate::parser::loc_parser;
use crate::utils::lsp_convert::RangeMapper;
use std::collections::HashMap;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;

impl Backend {
    pub(crate) async fn handle_code_action(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<CodeActionResponse>> {
        let mut actions = Vec::new();
        let mut has_casing_diagnostic = false;
        let mut has_trailing_whitespace_diagnostic = false;
        let mut has_mixed_indentation_diagnostic = false;
        let mut has_assignment_space_diagnostic = false;
        let mut has_brace_space_diagnostic = false;
        let mut has_unnecessary_version_diagnostic = false;
        let mut has_unescaped_quote_diagnostic = false;

        let mut has_eof_newline_diagnostic = false;
        let mut has_path_separator_diagnostic = false;
        let mut has_unit_type_casing_diagnostic = false;

        for diagnostic in &params.context.diagnostics {
            if let Some(target_casing) = diagnostic.data.as_ref().and_then(|v| v.as_str()) {
                let is_casing_fix = match &diagnostic.code {
                    Some(NumberOrString::String(s)) => s == "casing",
                    _ => {
                        diagnostic.message.contains("Standard Paradox Script")
                            || diagnostic.message.contains("Standard casing")
                    }
                };

                if is_casing_fix {
                    has_casing_diagnostic = true;
                    let mut changes = HashMap::new();
                    changes.insert(
                        params.text_document.uri.clone(),
                        vec![TextEdit {
                            range: diagnostic.range,
                            new_text: target_casing.to_string(),
                        }],
                    );

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
                } else if let Some(NumberOrString::String(code)) = &diagnostic.code {
                    if code == "styling_path_separator" {
                        has_path_separator_diagnostic = true;
                        let mut changes = HashMap::new();
                        changes.insert(
                            params.text_document.uri.clone(),
                            vec![TextEdit {
                                range: diagnostic.range,
                                new_text: format!("\"{}\"", target_casing),
                            }],
                        );

                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: format!("Use single forward slashes: '{}'", target_casing),
                            kind: Some(CodeActionKind::QUICKFIX),
                            edit: Some(WorkspaceEdit {
                                changes: Some(changes),
                                ..Default::default()
                            }),
                            diagnostics: Some(vec![diagnostic.clone()]),
                            is_preferred: Some(true),
                            ..Default::default()
                        }));
                    } else if code == "HOM3007" {
                        has_unit_type_casing_diagnostic = true;
                        let mut changes = HashMap::new();
                        changes.insert(
                            params.text_document.uri.clone(),
                            vec![TextEdit {
                                range: diagnostic.range,
                                new_text: target_casing.to_string(),
                            }],
                        );

                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: format!("Change to canonical casing: '{}'", target_casing),
                            kind: Some(CodeActionKind::QUICKFIX),
                            edit: Some(WorkspaceEdit {
                                changes: Some(changes),
                                ..Default::default()
                            }),
                            diagnostics: Some(vec![diagnostic.clone()]),
                            is_preferred: Some(true),
                            ..Default::default()
                        }));
                    } else if code == "HOM4004" {
                        // Idea reference differs in case from its definition —
                        // change it to the canonical casing stored in `data`.
                        let mut changes = HashMap::new();
                        changes.insert(
                            params.text_document.uri.clone(),
                            vec![TextEdit {
                                range: diagnostic.range,
                                new_text: target_casing.to_string(),
                            }],
                        );

                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: format!("Change idea reference casing to: '{}'", target_casing),
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
            } else {
                // Check other styling codes
                if let Some(NumberOrString::String(code)) = &diagnostic.code {
                    if code == "styling_trailing" {
                        has_trailing_whitespace_diagnostic = true;
                        let mut changes = HashMap::new();
                        changes.insert(
                            params.text_document.uri.clone(),
                            vec![TextEdit {
                                range: diagnostic.range,
                                new_text: "".to_string(),
                            }],
                        );

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
                        has_eof_newline_diagnostic = true;
                        let mut changes = HashMap::new();
                        changes.insert(
                            params.text_document.uri.clone(),
                            vec![TextEdit {
                                range: diagnostic.range,
                                new_text: "\n".to_string(),
                            }],
                        );

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
                        if let Some(content) =
                            self.documents.get(&params.text_document.uri.to_string())
                        {
                            let line_idx = diagnostic.range.start.line as usize;
                            if let Some(line) = content.lines().nth(line_idx) {
                                // `diagnostic.range` comes from the CLIENT, so its
                                // columns are UTF-16 code units. Slicing `line` with
                                // them directly panics on any multi-byte character
                                // ("byte index N is not a char boundary"), and the
                                // `end <= line.len()` guard does not help: line.len()
                                // is a BYTE length, which is >= the UTF-16 length, so
                                // it lets bad indices through. Convert first —
                                // utf16_to_byte always lands on a char boundary.
                                let index = crate::utils::line_index::LineIndex::new(line);
                                let start =
                                    index.utf16_to_byte(diagnostic.range.start.character as usize);
                                let end =
                                    index.utf16_to_byte(diagnostic.range.end.character as usize);
                                if start <= end && end <= line.len() {
                                    let op_str = &line[start..end];
                                    let mut changes = HashMap::new();
                                    changes.insert(
                                        params.text_document.uri.clone(),
                                        vec![TextEdit {
                                            range: diagnostic.range,
                                            new_text: format!(" {} ", op_str.trim()),
                                        }],
                                    );

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
                        if let Some(content) =
                            self.documents.get(&params.text_document.uri.to_string())
                        {
                            let line_idx = diagnostic.range.start.line as usize;
                            if let Some(line) = content.lines().nth(line_idx) {
                                // Client-supplied UTF-16 columns — convert before
                                // slicing (see the assignment-space branch above).
                                let index = crate::utils::line_index::LineIndex::new(line);
                                let start =
                                    index.utf16_to_byte(diagnostic.range.start.character as usize);
                                let end =
                                    index.utf16_to_byte(diagnostic.range.end.character as usize);
                                if start < end && end <= line.len() {
                                    let full_str = &line[start..end];
                                    if let Some(brace_start_rel) = full_str.find('{') {
                                        // `rfind('}').unwrap_or(len - 1)` produced
                                        // `brace_end_rel == brace_start_rel` when the
                                        // slice held no closing brace (e.g. the range
                                        // ends at the `{`), making the slice below
                                        // `[start+1 .. start]` — a "begin > end"
                                        // panic. There is nothing to fix without a
                                        // closing brace, so bail instead of guessing.
                                        let Some(brace_end_rel) = full_str.rfind('}') else {
                                            continue;
                                        };
                                        if brace_end_rel <= brace_start_rel {
                                            continue;
                                        }
                                        let inner = &full_str[brace_start_rel + 1..brace_end_rel];

                                        let before_brace = full_str[..brace_start_rel].trim();

                                        let new_text = if inner.trim().is_empty() {
                                            if !before_brace.is_empty() {
                                                format!("{} {{}}", before_brace)
                                            } else {
                                                "{}".to_string()
                                            }
                                        } else {
                                            if !before_brace.is_empty() {
                                                format!("{} {{ {} }}", before_brace, inner.trim())
                                            } else {
                                                format!("{{ {} }}", inner.trim())
                                            }
                                        };

                                        let mut changes = HashMap::new();
                                        changes.insert(
                                            params.text_document.uri.clone(),
                                            vec![TextEdit {
                                                range: diagnostic.range,
                                                new_text,
                                            }],
                                        );

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
                        changes.insert(
                            params.text_document.uri.clone(),
                            vec![TextEdit {
                                range: diagnostic.range,
                                new_text: " ".to_string(),
                            }],
                        );

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
                        changes.insert(
                            params.text_document.uri.clone(),
                            vec![TextEdit {
                                range: diagnostic.range,
                                new_text: "".to_string(),
                            }],
                        );

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
                    } else if code == "unnecessary_version" {
                        has_unnecessary_version_diagnostic = true;
                        let mut changes = HashMap::new();
                        changes.insert(
                            params.text_document.uri.clone(),
                            vec![TextEdit {
                                range: diagnostic.range,
                                new_text: "".to_string(),
                            }],
                        );

                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Remove unnecessary version number".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            edit: Some(WorkspaceEdit {
                                changes: Some(changes),
                                ..Default::default()
                            }),
                            diagnostics: Some(vec![diagnostic.clone()]),
                            is_preferred: Some(true),
                            ..Default::default()
                        }));
                    } else if code == "unescaped_quote" {
                        has_unescaped_quote_diagnostic = true;
                        let mut changes = HashMap::new();
                        changes.insert(
                            params.text_document.uri.clone(),
                            vec![TextEdit {
                                range: diagnostic.range,
                                new_text: "\\\"".to_string(),
                            }],
                        );

                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Escape double quote".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            edit: Some(WorkspaceEdit {
                                changes: Some(changes),
                                ..Default::default()
                            }),
                            diagnostics: Some(vec![diagnostic.clone()]),
                            is_preferred: Some(true),
                            ..Default::default()
                        }));
                    } else if code == "language_case_mismatch" {
                        if let Some(content) =
                            self.documents.get(&params.text_document.uri.to_string())
                        {
                            let line_idx = diagnostic.range.start.line as usize;
                            if let Some(line) = content.lines().nth(line_idx) {
                                let index = crate::utils::line_index::LineIndex::new(line);
                                let start =
                                    index.utf16_to_byte(diagnostic.range.start.character as usize);
                                let end =
                                    index.utf16_to_byte(diagnostic.range.end.character as usize);
                                if start <= end && end <= line.len() {
                                    let original = &line[start..end];
                                    let fixed = original.to_ascii_lowercase();
                                    let mut changes = HashMap::new();
                                    changes.insert(
                                        params.text_document.uri.clone(),
                                        vec![TextEdit {
                                            range: diagnostic.range,
                                            new_text: fixed.clone(),
                                        }],
                                    );
                                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                        title: format!(
                                            "Fix language header case: '{}' → '{}'",
                                            original, fixed
                                        ),
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
                    } else if code == "styling_indent" {
                        has_mixed_indentation_diagnostic = true;
                        if let Some(content) =
                            self.documents.get(&params.text_document.uri.to_string())
                        {
                            let line_idx = diagnostic.range.start.line as usize;
                            if let Some(line) = content.lines().nth(line_idx) {
                                let leading = line
                                    .chars()
                                    .take_while(|c| c.is_whitespace())
                                    .collect::<String>();

                                let new_indent = if let Some(expected_tabs) = diagnostic
                                    .data
                                    .as_ref()
                                    .and_then(|v| v.get("expected_tabs"))
                                    .and_then(|v| v.as_u64())
                                {
                                    "\t".repeat(expected_tabs as usize)
                                } else {
                                    // For YAML files or other cases without expected_tabs
                                    if leading.is_empty() {
                                        String::new()
                                    } else if leading.starts_with('\t') {
                                        // Already has tabs, keep them
                                        leading.clone()
                                    } else {
                                        // Has spaces, convert to at least one tab
                                        // For YAML: any amount of leading spaces should become one tab
                                        "\t".to_string()
                                    }
                                };

                                let mut changes = HashMap::new();
                                changes.insert(
                                    params.text_document.uri.clone(),
                                    vec![TextEdit {
                                        range: diagnostic.range,
                                        new_text: new_indent,
                                    }],
                                );

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

        // Fetch a fresh AST for "Fix all" bulk operations below.
        // Uses get_or_parse_ast to ensure the AST matches current document content,
        // preventing stale-position edits when the user clicks the lightbulb
        // between a did_change update and the debounced AST re-parse.
        let fresh_uri = params.text_document.uri.as_str().to_owned();
        let fresh_ast = self.get_or_parse_ast(&fresh_uri).await;

        // Add "Fix all" if any casing diagnostic is present
        if has_casing_diagnostic {
            if let Some((ref script, _)) = fresh_ast {
                let mapper = RangeMapper::new(&script.source);
                let mut all_fixes = Vec::new();
                self.collect_casing_fixes(&script.entries, &mut all_fixes, &script.source);

                if !all_fixes.is_empty() {
                    let mut changes = HashMap::new();
                    let edits: Vec<TextEdit> = all_fixes
                        .into_iter()
                        .map(|(range, text)| TextEdit {
                            range: mapper.range(&range),
                            new_text: text,
                        })
                        .collect();

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

        // Add "Fix all path separators" if any such diagnostic is present
        if has_path_separator_diagnostic {
            if let Some((ref script, _)) = fresh_ast {
                let mapper = RangeMapper::new(&script.source);
                let mut all_fixes = Vec::new();
                self.collect_path_separator_fixes(&script.entries, &mut all_fixes, &script.source);

                if !all_fixes.is_empty() {
                    let mut changes = HashMap::new();
                    let edits: Vec<TextEdit> = all_fixes
                        .into_iter()
                        .map(|(range, text)| TextEdit {
                            range: mapper.range(&range),
                            new_text: text,
                        })
                        .collect();

                    changes.insert(params.text_document.uri.clone(), edits);

                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                        title: "Fix all path separators in this file".to_string(),
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

        // Add "Remove all trailing whitespace" if any such diagnostic is present
        if has_trailing_whitespace_diagnostic {
            if let Some(content) = self.documents.get(&params.text_document.uri.to_string()) {
                let mut all_fixes = Vec::new();
                self.collect_styling_fixes(&content, &mut all_fixes);

                if !all_fixes.is_empty() {
                    let mut changes = HashMap::new();
                    let edits: Vec<TextEdit> = all_fixes
                        .into_iter()
                        .map(|(range, text)| TextEdit {
                            range,
                            new_text: text,
                        })
                        .collect();

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
                let is_yaml = params.text_document.uri.as_str().ends_with(".yml");
                let script_opt = if is_yaml {
                    None
                } else {
                    fresh_ast.as_ref().map(|(s, _)| s.clone())
                };

                let mut all_fixes = Vec::new();
                self.collect_indentation_fixes(&content, script_opt.as_deref(), &mut all_fixes);

                if !all_fixes.is_empty() {
                    let mut changes = HashMap::new();
                    let edits: Vec<TextEdit> = all_fixes
                        .into_iter()
                        .map(|(range, text)| TextEdit {
                            range,
                            new_text: text,
                        })
                        .collect();

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
                if let Some((ref script, _)) = fresh_ast {
                    let mapper = RangeMapper::new(&content);
                    let mut all_fixes = Vec::new();
                    Self::collect_assignment_space_fixes(&script.entries, &mut all_fixes, &content);

                    if !all_fixes.is_empty() {
                        let mut changes = HashMap::new();
                        let edits: Vec<TextEdit> = all_fixes
                            .into_iter()
                            .map(|(range, text)| TextEdit {
                                range: mapper.range(&range),
                                new_text: text,
                            })
                            .collect();

                        changes.insert(params.text_document.uri.clone(), edits);

                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Surround all assignment operators with spaces in this file"
                                .to_string(),
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
                if let Some((ref script, _)) = fresh_ast {
                    let mapper = RangeMapper::new(&content);
                    let mut all_fixes = Vec::new();
                    self.collect_brace_space_fixes(&script.entries, &mut all_fixes, &content);
                    self.collect_brace_newline_fixes(&script.entries, &mut all_fixes);

                    if !all_fixes.is_empty() {
                        let mut changes = HashMap::new();
                        let edits: Vec<TextEdit> = all_fixes
                            .into_iter()
                            .map(|(range, text)| TextEdit {
                                range: mapper.range(&range),
                                new_text: text,
                            })
                            .collect();

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

        // Add "Remove all unnecessary version numbers" if any such diagnostic is present
        if has_unnecessary_version_diagnostic {
            if let Some(content) = self.documents.get(&params.text_document.uri.to_string()) {
                let mapper = RangeMapper::new(&content);
                let path_str = params
                    .text_document
                    .uri
                    .to_file_path()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let (parsed, _, _) = loc_parser::parse_loc_file(&content, &path_str);
                let mut all_fixes = Vec::new();

                for entry in parsed.values() {
                    if let Some(d) = loc_parser::check_unnecessary_version(entry) {
                        all_fixes.push((d.range, "".to_string()));
                    }
                }

                if !all_fixes.is_empty() {
                    let mut changes = HashMap::new();
                    let edits: Vec<TextEdit> = all_fixes
                        .into_iter()
                        .map(|(range, text)| TextEdit {
                            range: mapper.range(&range),
                            new_text: text,
                        })
                        .collect();

                    changes.insert(params.text_document.uri.clone(), edits);

                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                        title: "Remove all unnecessary version numbers in this file".to_string(),
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

        // Add "Escape all unescaped double quotes" if any such diagnostic is present
        if has_unescaped_quote_diagnostic {
            if let Some(content) = self.documents.get(&params.text_document.uri.to_string()) {
                let mapper = RangeMapper::new(&content);
                let diagnostics = loc_parser::validate_unescaped_quotes_in_file(&content);

                if !diagnostics.is_empty() {
                    let mut changes = HashMap::new();
                    let edits: Vec<TextEdit> = diagnostics
                        .into_iter()
                        .map(|d| TextEdit {
                            range: mapper.range(&d.range),
                            new_text: "\\\"".to_string(),
                        })
                        .collect();

                    changes.insert(params.text_document.uri.clone(), edits);

                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                        title: "Escape all unescaped double quotes in this file".to_string(),
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

        let has_any_styling_diagnostic = has_casing_diagnostic
            || has_trailing_whitespace_diagnostic
            || has_mixed_indentation_diagnostic
            || has_assignment_space_diagnostic
            || has_brace_space_diagnostic
            || has_unnecessary_version_diagnostic
            || has_unescaped_quote_diagnostic
            || has_eof_newline_diagnostic
            || has_path_separator_diagnostic;

        if has_any_styling_diagnostic {
            if let Some(content) = self.documents.get(&params.text_document.uri.to_string()) {
                if let Some((ref script, _)) = fresh_ast {
                    let mapper = RangeMapper::new(&content);
                    let mut all_changes = Vec::new();
                    let is_yaml = fresh_uri.ends_with(".yml");
                    let lines: Vec<&str> = content.lines().collect();

                    // Add EOF newline fix if needed — check the file content
                    // directly, NOT the `has_eof_newline_diagnostic` flag (which
                    // only reflects diagnostics near the cursor). Consistent with
                    // every other fix type collected in this bulk action.
                    if !content.is_empty()
                        && !content.ends_with('\n')
                        && !content.ends_with("\r\n")
                        && !fresh_uri.ends_with("map/buildings.txt")
                    {
                        let line_count = lines.len();
                        let last_line = lines.last().copied().unwrap_or("");
                        let line_idx = if line_count > 0 {
                            line_count as u32 - 1
                        } else {
                            0
                        };
                        all_changes.push(TextEdit {
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
                            new_text: "\n".to_string(),
                        });
                    }

                    // ── Always collect ALL fix types across the whole file ──
                    // Unlike individual "Fix all X" actions (which scan the whole file
                    // for their type), this combined action does them all in one go.
                    // The individual has_*_diagnostic flags are NOT used here because
                    // they only reflect diagnostics near the cursor — we want to fix
                    // all styling issues in the entire file regardless.
                    let mut casing_fixes = Vec::new();
                    self.collect_casing_fixes(&script.entries, &mut casing_fixes, &script.source);
                    for (range, text) in casing_fixes {
                        all_changes.push(TextEdit {
                            range: mapper.range(&range),
                            new_text: text,
                        });
                    }

                    let mut tw_fixes = Vec::new();
                    self.collect_styling_fixes(&content, &mut tw_fixes);
                    for (range, text) in tw_fixes {
                        all_changes.push(TextEdit {
                            range,
                            new_text: text,
                        });
                    }

                    let mut indent_fixes = Vec::new();
                    let script_opt = if is_yaml { None } else { Some(&**script) };
                    self.collect_indentation_fixes(&content, script_opt, &mut indent_fixes);
                    for (range, text) in indent_fixes {
                        all_changes.push(TextEdit {
                            range,
                            new_text: text,
                        });
                    }

                    let mut assign_fixes = Vec::new();
                    Self::collect_assignment_space_fixes(
                        &script.entries,
                        &mut assign_fixes,
                        &content,
                    );
                    for (range, text) in assign_fixes {
                        all_changes.push(TextEdit {
                            range: mapper.range(&range),
                            new_text: text,
                        });
                    }

                    let mut brace_fixes = Vec::new();
                    self.collect_brace_space_fixes(&script.entries, &mut brace_fixes, &content);
                    self.collect_brace_newline_fixes(&script.entries, &mut brace_fixes);
                    for (range, text) in brace_fixes {
                        all_changes.push(TextEdit {
                            range: mapper.range(&range),
                            new_text: text,
                        });
                    }

                    let path_str = params
                        .text_document
                        .uri
                        .to_file_path()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let (parsed, _, _) = loc_parser::parse_loc_file(&content, &path_str);
                    for entry in parsed.values() {
                        if let Some(d) = loc_parser::check_unnecessary_version(entry) {
                            all_changes.push(TextEdit {
                                range: mapper.range(&d.range),
                                new_text: "".to_string(),
                            });
                        }
                    }

                    let mut path_sep_fixes = Vec::new();
                    self.collect_path_separator_fixes(
                        &script.entries,
                        &mut path_sep_fixes,
                        &script.source,
                    );
                    for (range, text) in path_sep_fixes {
                        all_changes.push(TextEdit {
                            range: mapper.range(&range),
                            new_text: text,
                        });
                    }

                    let quote_diagnostics = loc_parser::validate_unescaped_quotes_in_file(&content);
                    for d in quote_diagnostics {
                        all_changes.push(TextEdit {
                            range: mapper.range(&d.range),
                            new_text: "\"".to_string(),
                        });
                    }

                    if !all_changes.is_empty() {
                        let mut changes = HashMap::new();
                        changes.insert(params.text_document.uri.clone(), all_changes);

                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Fix all styling issues in this file".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            edit: Some(WorkspaceEdit {
                                changes: Some(changes),
                                ..Default::default()
                            }),
                            is_preferred: Some(true),
                            ..Default::default()
                        }));
                    }
                }
            }
        }

        // Add "Fix all unit type casing" actions if any HOM3007 diagnostic is present.
        // Generates two kinds of bulk fixes:
        //   1. Per canonical type: "Fix all 'infantry' → 'infantry' in this file"
        //   2. All types: "Fix all unit type casing in this file"
        if has_unit_type_casing_diagnostic {
            // Collect unique canonical types from diagnostics
            let mut seen_types: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            for diagnostic in &params.context.diagnostics {
                if let Some(NumberOrString::String(code)) = &diagnostic.code {
                    if code == "HOM3007" {
                        if let Some(canonical) = diagnostic.data.as_ref().and_then(|v| v.as_str()) {
                            seen_types.insert(canonical.to_string());
                        }
                    }
                }
            }

            if let Some((ref script, _)) = fresh_ast {
                let mapper = RangeMapper::new(&script.source);
                // ── Per-type bulk actions ─────────────────────────────
                for canonical in &seen_types {
                    let mut fixes = Vec::new();
                    self.collect_unit_type_casing_fixes(
                        &script.entries,
                        &mut fixes,
                        &script.source,
                        Some(canonical.as_str()),
                    );

                    if !fixes.is_empty() {
                        let mut changes = std::collections::HashMap::new();
                        let edits: Vec<TextEdit> = fixes
                            .into_iter()
                            .map(|(range, text)| TextEdit {
                                range: mapper.range(&range),
                                new_text: text,
                            })
                            .collect();

                        changes.insert(params.text_document.uri.clone(), edits);

                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: format!("Fix all '{}' in this file", canonical),
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

                // ── All-types bulk action ────────────────────────────
                let mut all_fixes = Vec::new();
                self.collect_unit_type_casing_fixes(
                    &script.entries,
                    &mut all_fixes,
                    &script.source,
                    None,
                );

                if !all_fixes.is_empty() {
                    let mut changes = std::collections::HashMap::new();
                    let edits: Vec<TextEdit> = all_fixes
                        .into_iter()
                        .map(|(range, text)| TextEdit {
                            range: mapper.range(&range),
                            new_text: text,
                        })
                        .collect();

                    changes.insert(params.text_document.uri.clone(), edits);

                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                        title: "Fix all unit type casing in this file".to_string(),
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

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }
}

// ---------------------------------------------------------------------------
// SECTION - Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::utils::line_index::LineIndex;

    /// REGRESSION: the styling code actions sliced the document line with the
    /// UTF-16 columns that arrived in `diagnostic.range` from the client.
    ///
    /// The old guard was `end <= line.len()`, comparing a UTF-16 column against
    /// a BYTE length. For a line containing multi-byte characters the byte
    /// length is LARGER than the UTF-16 length, so the guard happily admitted
    /// an index that then sliced through the middle of a character and
    /// panicked, killing the codeAction request.
    ///
    /// This reproduces the shape of that bug and proves the conversion used by
    /// both call sites always yields a char boundary.
    #[test]
    fn test_utf16_columns_must_be_converted_before_slicing() {
        // 'é' is 2 bytes / 1 UTF-16 unit, so byte len > utf16 len from here on.
        let line = "\tdesc = \"café\" trigger = { tag = GER }";
        let index = LineIndex::new(line);

        // The old code did `&line[start..end]` with raw UTF-16 columns.
        // Walk every column and show the naive index is unsafe while the
        // converted one is always valid.
        let mut naive_would_panic = 0;
        for utf16_col in 0..=index.utf16_len() as usize {
            // Naive path (what the bug did).
            if utf16_col <= line.len() && !line.is_char_boundary(utf16_col) {
                naive_would_panic += 1;
            }
            // Fixed path.
            let byte = index.utf16_to_byte(utf16_col);
            assert!(
                line.is_char_boundary(byte),
                "utf16_to_byte({utf16_col}) = {byte} is not a char boundary"
            );
            let _ = &line[..byte]; // must not panic
        }
        assert!(
            naive_would_panic > 0,
            "this line should expose the raw-column bug; test is not proving anything"
        );
    }

    /// The brace-spacing action used `rfind('}').unwrap_or(len - 1)`. When the
    /// slice contained no closing brace that yielded
    /// `brace_end_rel == brace_start_rel`, and the following
    /// `full_str[brace_start_rel + 1..brace_end_rel]` was a "begin > end"
    /// panic. The fix bails out instead.
    #[test]
    fn test_brace_slice_without_closing_brace_is_skipped() {
        for full_str in ["tag {", "{", "foo = {"] {
            let brace_start_rel = full_str.find('{').expect("test inputs contain '{'");

            // Old behaviour: reproduce the computation and assert it was broken.
            let old_end = full_str.rfind('}').unwrap_or(full_str.len() - 1);
            assert!(
                brace_start_rel + 1 > old_end,
                "{full_str:?} should have produced an inverted range under the old code"
            );

            // New behaviour: no closing brace -> skip, never slice.
            assert!(
                full_str.rfind('}').is_none(),
                "{full_str:?} has no closing brace, so the action must bail"
            );
        }

        // Sanity: a well-formed single-line block still yields a valid inner slice.
        let ok = "tag { a = 1 }";
        let bs = ok.find('{').unwrap();
        let be = ok.rfind('}').unwrap();
        assert!(be > bs);
        assert_eq!(&ok[bs + 1..be], " a = 1 ");
    }
}

// ---------------------------------------------------------------------------
// !SECTION
// ---------------------------------------------------------------------------
