use crate::Backend;
use crate::formatter::serialize::serialize;
use crate::formatter::transform;
use crate::parser::ast;
use crate::parser::cst::parse_cst;
use tower_lsp_server::ls_types::{Position, Range};

impl Backend {
    /// Format a document using the full CST pipeline.
    /// Returns the formatted text.
    pub(crate) fn format_document(&self, content: &str) -> String {
        let mut cst = parse_cst(content);
        transform::format(&mut cst);
        serialize(&cst)
    }

    /// Find trailing whitespace lines and produce LSP edits to remove it.
    pub(crate) fn collect_styling_fixes(&self, content: &str, fixes: &mut Vec<(Range, String)>) {
        let mut cst = parse_cst(content);
        transform::fix_trailing_whitespace(&mut cst);
        let formatted = serialize(&cst);
        compute_lsp_edits(content, &formatted, fixes);
    }

    /// Fix indentation to use tabs at the correct depth.
    /// `_script_opt` is ignored (kept for backward compatibility).
    pub(crate) fn collect_indentation_fixes(
        &self,
        content: &str,
        _script_opt: Option<&ast::Script>,
        fixes: &mut Vec<(Range, String)>,
    ) {
        let mut cst = parse_cst(content);
        transform::fix_indentation(&mut cst);
        let formatted = serialize(&cst);
        compute_lsp_edits(content, &formatted, fixes);
    }

    /// Ensure single space around assignment operators (= < > <= >= !=).
    pub(crate) fn collect_assignment_space_fixes(
        &self,
        content: &str,
        fixes: &mut Vec<(ast::Range, String)>,
    ) {
        let mut cst = parse_cst(content);
        transform::fix_assignment_spacing(&mut cst);
        let formatted = serialize(&cst);
        compute_ast_edits(content, &formatted, fixes);
    }

    /// Move open braces to the same line as the assignment (no newline before {).
    pub(crate) fn collect_brace_newline_fixes(
        &self,
        content: &str,
        fixes: &mut Vec<(ast::Range, String)>,
    ) {
        let mut cst = parse_cst(content);
        transform::fix_brace_style(&mut cst);
        let formatted = serialize(&cst);
        compute_ast_edits(content, &formatted, fixes);
    }

    /// Fix spacing inside braces (also applies brace-style and assignment spacing).
    pub(crate) fn collect_brace_space_fixes(
        &self,
        content: &str,
        fixes: &mut Vec<(ast::Range, String)>,
    ) {
        let mut cst = parse_cst(content);
        transform::fix_brace_style(&mut cst);
        transform::fix_assignment_spacing(&mut cst);
        let formatted = serialize(&cst);
        compute_ast_edits(content, &formatted, fixes);
    }

    /// Fix key casing for known HOI4 keywords.
    pub(crate) fn collect_casing_fixes(
        &self,
        content: &str,
        fixes: &mut Vec<(ast::Range, String)>,
    ) {
        let mut cst = parse_cst(content);
        apply_casing_fixes_cst(&mut cst);
        let formatted = serialize(&cst);
        compute_ast_edits(content, &formatted, fixes);
    }

    /// Normalize path separators in texturefile values to forward slashes.
    pub(crate) fn collect_path_separator_fixes(
        &self,
        content: &str,
        fixes: &mut Vec<(ast::Range, String)>,
    ) {
        let mut cst = parse_cst(content);
        apply_path_sep_fixes_cst(&mut cst);
        let formatted = serialize(&cst);
        compute_ast_edits(content, &formatted, fixes);
    }
}

// ---------------------------------------------------------------------------
// Edit computation helpers
// ---------------------------------------------------------------------------

/// Compute per-line LSP edits by comparing original and formatted texts.
fn compute_lsp_edits(original: &str, formatted: &str, fixes: &mut Vec<(Range, String)>) {
    if original == formatted {
        return;
    }

    let orig_lines: Vec<&str> = original.lines().collect();
    let fmt_lines: Vec<&str> = formatted.lines().collect();

    if orig_lines.len() == fmt_lines.len() {
        // Same number of lines — per-line edits
        for (i, (orig, fmt)) in orig_lines.iter().zip(fmt_lines.iter()).enumerate() {
            if orig != fmt {
                let eol = crate::utf16_len(orig);
                fixes.push((
                    Range {
                        start: Position {
                            line: i as u32,
                            character: 0,
                        },
                        end: Position {
                            line: i as u32,
                            character: eol,
                        },
                    },
                    fmt.to_string(),
                ));
            }
        }
    } else {
        // Different line count — find the diff block and replace it
        let mut first_diff = 0;
        while first_diff < orig_lines.len().min(fmt_lines.len())
            && orig_lines[first_diff] == fmt_lines[first_diff]
        {
            first_diff += 1;
        }

        let mut orig_end = orig_lines.len();
        let mut fmt_end = fmt_lines.len();
        while orig_end > first_diff
            && fmt_end > first_diff
            && orig_lines[orig_end - 1] == fmt_lines[fmt_end - 1]
        {
            orig_end -= 1;
            fmt_end -= 1;
        }

        let replacement = fmt_lines[first_diff..fmt_end].join("\n");
        // Preserve trailing newline if original had one
        let replacement = if original.ends_with('\n') && !replacement.ends_with('\n') {
            replacement + "\n"
        } else {
            replacement
        };

        let end_line = if orig_end == 0 {
            0
        } else {
            (orig_end - 1) as u32
        };
        let end_col = if orig_end > 0 && orig_end <= orig_lines.len() {
            crate::utf16_len(orig_lines[orig_end - 1])
        } else {
            0
        };

        fixes.push((
            Range {
                start: Position {
                    line: first_diff as u32,
                    character: 0,
                },
                end: Position {
                    line: end_line,
                    character: end_col,
                },
            },
            replacement,
        ));
    }
}

/// Compute per-line AST edits by comparing original and formatted texts.
fn compute_ast_edits(original: &str, formatted: &str, fixes: &mut Vec<(ast::Range, String)>) {
    if original == formatted {
        return;
    }

    let orig_lines: Vec<&str> = original.lines().collect();
    let fmt_lines: Vec<&str> = formatted.lines().collect();

    if orig_lines.len() == fmt_lines.len() {
        // Same number of lines — per-line edits
        for (i, (orig, fmt)) in orig_lines.iter().zip(fmt_lines.iter()).enumerate() {
            if orig != fmt {
                let eol = crate::utf16_len(orig);
                fixes.push((
                    ast::Range {
                        start_line: i as u32,
                        start_col: 0,
                        end_line: i as u32,
                        end_col: eol,
                    },
                    fmt.to_string(),
                ));
            }
        }
    } else {
        // Different line count — find the diff block and replace it
        let mut first_diff = 0;
        while first_diff < orig_lines.len().min(fmt_lines.len())
            && orig_lines[first_diff] == fmt_lines[first_diff]
        {
            first_diff += 1;
        }

        let mut orig_end = orig_lines.len();
        let mut fmt_end = fmt_lines.len();
        while orig_end > first_diff
            && fmt_end > first_diff
            && orig_lines[orig_end - 1] == fmt_lines[fmt_end - 1]
        {
            orig_end -= 1;
            fmt_end -= 1;
        }

        let replacement = fmt_lines[first_diff..fmt_end].join("\n");
        // Preserve trailing newline if original had one
        let replacement = if original.ends_with('\n') && !replacement.ends_with('\n') {
            replacement + "\n"
        } else {
            replacement
        };

        let end_line = if orig_end == 0 {
            0
        } else {
            (orig_end - 1) as u32
        };
        let end_col = if orig_end > 0 && orig_end <= orig_lines.len() {
            crate::utf16_len(orig_lines[orig_end - 1])
        } else {
            0
        };

        fixes.push((
            ast::Range {
                start_line: first_diff as u32,
                start_col: 0,
                end_line,
                end_col,
            },
            replacement,
        ));
    }
}

// ---------------------------------------------------------------------------
// CST-based casing fixer
// ---------------------------------------------------------------------------

/// Apply casing fixes directly to a CST by checking key names against known
/// HOI4 keywords and correcting the case.
fn apply_casing_fixes_cst(cst: &mut crate::parser::cst::types::CstScript) {
    use crate::parser::cst::types::*;

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

    fn walk_nodes(nodes: &mut [CstNode], keywords: &[&str]) {
        for node in nodes {
            match node {
                CstNode::Assignment(ass) => {
                    for kw in keywords {
                        if ass.key.text.eq_ignore_ascii_case(kw) && ass.key.text != *kw {
                            ass.key.text = kw.to_string();
                            break;
                        }
                    }
                    walk_value(&mut ass.value, keywords);
                }
                CstNode::EntryValue(ev) => {
                    walk_value(&mut ev.value, keywords);
                }
                _ => {}
            }
        }
    }

    fn walk_value(value: &mut CstValue, keywords: &[&str]) {
        match value {
            CstValue::Block(block) => {
                walk_nodes(&mut block.entries, keywords);
            }
            CstValue::TaggedBlock { block, .. } => {
                walk_nodes(&mut block.entries, keywords);
            }
            _ => {}
        }
    }

    walk_nodes(&mut cst.nodes, &keywords);
}

// ---------------------------------------------------------------------------
// CST-based path separator fixer
// ---------------------------------------------------------------------------

/// Normalize path separators in texturefile values (and similar) to forward
/// slashes by modifying CST token text in place.
fn apply_path_sep_fixes_cst(cst: &mut crate::parser::cst::types::CstScript) {
    use crate::parser::cst::types::*;

    fn walk_nodes(nodes: &mut [CstNode]) {
        for node in nodes {
            match node {
                CstNode::Assignment(ass) => {
                    if ass.key.text.eq_ignore_ascii_case("texturefile") {
                        if let CstValue::String(tok) = &mut ass.value {
                            if tok.text.contains("//") || tok.text.contains('\\') {
                                tok.text = tok.text.replace("//", "/").replace('\\', "/");
                            }
                        }
                    }
                    walk_value(&mut ass.value);
                }
                CstNode::EntryValue(ev) => {
                    walk_value(&mut ev.value);
                }
                _ => {}
            }
        }
    }

    fn walk_value(value: &mut CstValue) {
        match value {
            CstValue::Block(block) => {
                walk_nodes(&mut block.entries);
            }
            CstValue::TaggedBlock { block, .. } => {
                walk_nodes(&mut block.entries);
            }
            _ => {}
        }
    }

    walk_nodes(&mut cst.nodes);
}
