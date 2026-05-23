use crate::ast;
use crate::Backend;
use std::collections::HashMap;
use tower_lsp::lsp_types::{Position, Range};

impl Backend {
    pub(crate) fn collect_styling_fixes(&self, content: &str, fixes: &mut Vec<(Range, String)>) {
        for (line_idx, line) in content.lines().enumerate() {
            if line.ends_with(' ') || line.ends_with('\t') {
                let trimmed_len = line.trim_end().len();
                let start_col = crate::utf16_len(&line[..trimmed_len]);
                let end_col = crate::utf16_len(line);
                fixes.push((
                    Range {
                        start: Position {
                            line: line_idx as u32,
                            character: start_col,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: end_col,
                        },
                    },
                    "".to_string(),
                ));
            }
        }
    }

    pub(crate) fn collect_indentation_fixes(
        &self,
        content: &str,
        script_opt: Option<&ast::Script>,
        fixes: &mut Vec<(Range, String)>,
    ) {
        let mut expected_indents = HashMap::new();
        if let Some(script) = script_opt {
            Self::compute_expected_indentations(&script.entries, 0, &mut expected_indents);
        }

        for (line_idx, line) in content.lines().enumerate() {
            let leading = line
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect::<String>();
            if line.trim().is_empty() {
                continue;
            }

            if let Some(&expected_tabs) = expected_indents.get(&(line_idx as u32)) {
                let expected_str = "\t".repeat(expected_tabs);
                if leading != expected_str {
                    fixes.push((
                        Range {
                            start: Position {
                                line: line_idx as u32,
                                character: 0,
                            },
                            end: Position {
                                line: line_idx as u32,
                                character: leading.len() as u32,
                            },
                        },
                        expected_str,
                    ));
                }
            } else if leading.contains(' ') {
                let new_indent = if leading.is_empty() {
                    String::new()
                } else if leading.starts_with('\t') {
                    leading.clone()
                } else {
                    "\t".to_string()
                };

                if new_indent != leading {
                    fixes.push((
                        Range {
                            start: Position {
                                line: line_idx as u32,
                                character: 0,
                            },
                            end: Position {
                                line: line_idx as u32,
                                character: leading.len() as u32,
                            },
                        },
                        new_indent,
                    ));
                }
            }
        }
    }

    pub(crate) fn collect_assignment_space_fixes(
        &self,
        entries: &[ast::Entry],
        fixes: &mut Vec<(ast::Range, String)>,
        content: &str,
    ) {
        for entry in entries {
            match entry {
                ast::Entry::Assignment(ass) => {
                    let mut needs_fix = false;
                    if ass.key_range.end_line == ass.operator_range.start_line
                        && ass.key_range.end_line == ass.value.range.start_line
                    {
                        if ass.operator_range.start_col > ass.key_range.end_col
                            && ass.value.range.start_col > ass.operator_range.end_col
                        {
                            let space_before =
                                ass.operator_range.start_col - ass.key_range.end_col;
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
                                    format!(" {} ", op_str.trim()),
                                ));
                            }
                        }
                    }

                    match &ass.value.value {
                        ast::Value::Block(inner) => {
                            self.collect_assignment_space_fixes(inner, fixes, content)
                        }
                        ast::Value::TaggedBlock(_, inner, _) => {
                            self.collect_assignment_space_fixes(inner, fixes, content)
                        }
                        _ => {}
                    }
                }
                ast::Entry::Value(val) => match &val.value {
                    ast::Value::Block(inner) => {
                        self.collect_assignment_space_fixes(inner, fixes, content)
                    }
                    ast::Value::TaggedBlock(_, inner, _) => {
                        self.collect_assignment_space_fixes(inner, fixes, content)
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    pub(crate) fn collect_brace_newline_fixes(
        &self,
        entries: &[ast::Entry],
        fixes: &mut Vec<(ast::Range, String)>,
    ) {
        for entry in entries {
            match entry {
                ast::Entry::Assignment(ass) => match &ass.value.value {
                    ast::Value::Block(_) => {
                        if ass.value.range.start_line > ass.operator_range.end_line {
                            fixes.push((
                                ast::Range {
                                    start_line: ass.operator_range.end_line,
                                    start_col: ass.operator_range.end_col,
                                    end_line: ass.value.range.start_line,
                                    end_col: ass.value.range.start_col,
                                },
                                " ".to_string(),
                            ));
                        }
                        self.collect_brace_newline_fixes(
                            match &ass.value.value {
                                ast::Value::Block(i) => i,
                                _ => &[],
                            },
                            fixes,
                        );
                    }
                    ast::Value::TaggedBlock(tag, inner, block_range) => {
                        if block_range.start_line > ass.operator_range.end_line {
                            fixes.push((
                                ast::Range {
                                    start_line: ass.operator_range.end_line,
                                    start_col: ass.operator_range.end_col,
                                    end_line: block_range.start_line,
                                    end_col: block_range.start_col,
                                },
                                " ".to_string(),
                            ));
                        } else {
                            let tag_end_col = ass.value.range.start_col + tag.len() as u32;
                            if block_range.start_col != tag_end_col + 1 {
                                fixes.push((
                                    ast::Range {
                                        start_line: ass.value.range.start_line,
                                        start_col: tag_end_col,
                                        end_line: block_range.start_line,
                                        end_col: block_range.start_col,
                                    },
                                    " ".to_string(),
                                ));
                            }
                        }
                        self.collect_brace_newline_fixes(inner, fixes);
                    }
                    _ => {}
                },
                ast::Entry::Value(val) => match &val.value {
                    ast::Value::Block(inner) => self.collect_brace_newline_fixes(inner, fixes),
                    ast::Value::TaggedBlock(tag, inner, block_range) => {
                        if block_range.start_line > val.range.start_line {
                            fixes.push((
                                ast::Range {
                                    start_line: val.range.start_line,
                                    start_col: val.range.start_col + tag.len() as u32,
                                    end_line: block_range.start_line,
                                    end_col: block_range.start_col,
                                },
                                " ".to_string(),
                            ));
                        } else {
                            let tag_end_col = val.range.start_col + tag.len() as u32;
                            if block_range.start_col != tag_end_col + 1 {
                                fixes.push((
                                    ast::Range {
                                        start_line: val.range.start_line,
                                        start_col: tag_end_col,
                                        end_line: block_range.start_line,
                                        end_col: block_range.start_col,
                                    },
                                    " ".to_string(),
                                ));
                            }
                        }
                        self.collect_brace_newline_fixes(inner, fixes);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    pub(crate) fn collect_brace_space_fixes(
        &self,
        entries: &[ast::Entry],
        fixes: &mut Vec<(ast::Range, String)>,
        content: &str,
    ) {
        for entry in entries {
            match entry {
                ast::Entry::Assignment(ass) => {
                    Self::check_and_fix_brace(&ass.value.range, &ass.value.value, content, fixes);
                    match &ass.value.value {
                        ast::Value::Block(inner) => {
                            self.collect_brace_space_fixes(inner, fixes, content)
                        }
                        ast::Value::TaggedBlock(_, inner, _) => {
                            self.collect_brace_space_fixes(inner, fixes, content)
                        }
                        _ => {}
                    }
                }
                ast::Entry::Value(val) => {
                    Self::check_and_fix_brace(&val.range, &val.value, content, fixes);
                    match &val.value {
                        ast::Value::Block(inner) => {
                            self.collect_brace_space_fixes(inner, fixes, content)
                        }
                        ast::Value::TaggedBlock(_, inner, _) => {
                            self.collect_brace_space_fixes(inner, fixes, content)
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    pub(crate) fn check_and_fix_brace(
        range: &ast::Range,
        value: &ast::Value,
        content: &str,
        fixes: &mut Vec<(ast::Range, String)>,
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
                            let block_str = &full_str[brace_start_rel..];
                            let mut needs_fix = false;

                            if let ast::Value::TaggedBlock(tag, _, _) = value {
                                if &full_str[tag.len()..brace_start_rel] != " " {
                                    needs_fix = true;
                                }
                            }

                            if block_str.len() >= 2 {
                                let inner = &block_str[1..block_str.len() - 1];
                                if inner.trim().is_empty() {
                                    if block_str != "{}" {
                                        needs_fix = true;
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
                                let brace_end_rel =
                                    full_str.rfind('}').unwrap_or(full_str.len() - 1);
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
                                fixes.push((range.clone(), new_text));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub(crate) fn collect_casing_fixes(&self, entries: &[ast::Entry], fixes: &mut Vec<(ast::Range, String)>) {
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
            "containerWindowType",
            "origo",
            "alwaystransparent",
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
                        ast::Value::TaggedBlock(_, inner, _) => {
                            self.collect_casing_fixes(inner, fixes)
                        }
                        _ => {}
                    }
                }
                ast::Entry::Value(val) => match &val.value {
                    ast::Value::Block(inner) => self.collect_casing_fixes(inner, fixes),
                    ast::Value::TaggedBlock(_, inner, _) => self.collect_casing_fixes(inner, fixes),
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
