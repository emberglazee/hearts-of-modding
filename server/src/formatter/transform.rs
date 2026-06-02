//! In-place CST transformations for formatting.
//!
//! Applies indentation, spacing, brace style, and trailing whitespace fixes
//! directly to the CST tree. No tree recreation needed.

use crate::parser::ast;
use crate::parser::cst::token::{CstToken, Trivia, TriviaKind};
use crate::parser::cst::types::*;

/// Apply all formatting transformations to a CST in place.
pub fn format(cst: &mut CstScript) {
    fix_indentation(cst);
    fix_assignment_spacing(cst);
    fix_brace_style(cst);
    fix_trailing_whitespace(cst);
}

// ---------------------------------------------------------------------------
// fix_indentation
// ---------------------------------------------------------------------------

/// Walk all tokens in the CST tree. Track brace depth.
/// For every Newline trivia followed by Whitespace trivia, replace the
/// whitespace with "\t" * depth.
pub(crate) fn fix_indentation(cst: &mut CstScript) {
    let mut depth = 0u32;
    for node in &mut cst.nodes {
        fix_node_indentation(node, &mut depth);
    }
    // Also fix trailing trivia
    fix_trivia_indentation(&mut cst.trailing_trivia, 0);
}

fn fix_node_indentation(node: &mut CstNode, depth: &mut u32) {
    match node {
        CstNode::Assignment(ass) => {
            fix_token_indentation(&mut ass.key, *depth);
            fix_token_indentation(&mut ass.operator, *depth);
            fix_value_indentation(&mut ass.value, depth);
        }
        CstNode::EntryValue(ev) => {
            fix_value_indentation(&mut ev.value, depth);
        }
        CstNode::EntryComment(trivia) => {
            fix_single_trivia_indentation(trivia, *depth);
        }
        CstNode::Error(_) => {}
    }
}

fn fix_value_indentation(value: &mut CstValue, depth: &mut u32) {
    match value {
        CstValue::Ident(t) | CstValue::String(t) | CstValue::Number(t) | CstValue::Boolean(t) => {
            fix_token_indentation(t, *depth);
        }
        CstValue::Block(block) => {
            fix_token_indentation(&mut block.open_brace, *depth);
            *depth += 1;
            for entry in &mut block.entries {
                fix_node_indentation(entry, depth);
            }
            *depth -= 1;
            match &mut block.close_brace {
                CloseBrace::Present(token) => {
                    fix_token_indentation(token, *depth);
                }
                CloseBrace::Missing(_) => {}
            }
        }
        CstValue::TaggedBlock { tag, block } => {
            fix_token_indentation(tag, *depth);
            fix_token_indentation(&mut block.open_brace, *depth);
            *depth += 1;
            for entry in &mut block.entries {
                fix_node_indentation(entry, depth);
            }
            *depth -= 1;
            match &mut block.close_brace {
                CloseBrace::Present(token) => {
                    fix_token_indentation(token, *depth);
                }
                CloseBrace::Missing(_) => {}
            }
        }
        CstValue::Error(_) => {}
    }
}

fn fix_token_indentation(token: &mut CstToken, depth: u32) {
    let trivia = &mut token.leading_trivia;
    let mut new_trivia: Vec<Trivia> = Vec::with_capacity(trivia.len());
    let mut after_newline = false;
    for t in trivia.drain(..) {
        if t.kind == TriviaKind::Newline {
            after_newline = true;
            new_trivia.push(t);
        } else if after_newline && t.kind == TriviaKind::Whitespace {
            // Replace with correct tabs
            let expected = "\t".repeat(depth as usize);
            new_trivia.push(Trivia::new(
                TriviaKind::Whitespace,
                expected,
                placeholder_range(),
            ));
            after_newline = false;
        } else {
            after_newline = false;
            new_trivia.push(t);
        }
    }
    *trivia = new_trivia;
}

fn fix_trivia_indentation(trivia: &mut Vec<Trivia>, depth: u32) {
    let mut new_trivia: Vec<Trivia> = Vec::with_capacity(trivia.len());
    let mut after_newline = false;
    for t in trivia.drain(..) {
        if t.kind == TriviaKind::Newline {
            after_newline = true;
            new_trivia.push(t);
        } else if after_newline && t.kind == TriviaKind::Whitespace {
            let expected = "\t".repeat(depth as usize);
            new_trivia.push(Trivia::new(
                TriviaKind::Whitespace,
                expected,
                placeholder_range(),
            ));
            after_newline = false;
        } else {
            after_newline = false;
            new_trivia.push(t);
        }
    }
    *trivia = new_trivia;
}

fn fix_single_trivia_indentation(trivia: &mut Trivia, _depth: u32) {
    // EntryComment is a single Trivia piece (a Comment). Its indentation
    // is determined by its position in the token/trivia stream.  The comment
    // itself is just text — we don't modify it here.  Indentation for comments
    // is handled by the token they're attached to.
    let _ = trivia;
}

// ---------------------------------------------------------------------------
// fix_assignment_spacing
// ---------------------------------------------------------------------------

/// For each assignment, ensure the operator has exactly one space before and
/// after it.
pub(crate) fn fix_assignment_spacing(cst: &mut CstScript) {
    for node in &mut cst.nodes {
        fix_node_assignment_spacing(node);
    }
}

fn fix_node_assignment_spacing(node: &mut CstNode) {
    match node {
        CstNode::Assignment(ass) => {
            // Fix space before operator: the operator's leading_trivia should end with one space
            let op_trivia = &mut ass.operator.leading_trivia;
            ensure_trailing_space(op_trivia);

            // Fix space after operator: the value's first token's leading_trivia
            // should start with one space
            fix_value_leading_spacing(&mut ass.value);

            // Recurse into value
            fix_value_assignment_spacing(&mut ass.value);
        }
        CstNode::EntryValue(ev) => {
            fix_value_assignment_spacing(&mut ev.value);
        }
        CstNode::EntryComment(_) | CstNode::Error(_) => {}
    }
}

fn fix_value_assignment_spacing(value: &mut CstValue) {
    match value {
        CstValue::Block(block) => {
            for entry in &mut block.entries {
                fix_node_assignment_spacing(entry);
            }
        }
        CstValue::TaggedBlock { tag: _, block } => {
            for entry in &mut block.entries {
                fix_node_assignment_spacing(entry);
            }
        }
        _ => {}
    }
}

/// Ensure the last piece of leading trivia is a single space.
fn ensure_trailing_space(trivia: &mut Vec<Trivia>) {
    if let Some(last) = trivia.last_mut() {
        if last.kind == TriviaKind::Whitespace {
            // Normalize to single space
            last.text = " ".to_string();
        } else {
            trivia.push(Trivia::new(
                TriviaKind::Whitespace,
                " ".to_string(),
                placeholder_range(),
            ));
        }
    } else {
        trivia.push(Trivia::new(
            TriviaKind::Whitespace,
            " ".to_string(),
            placeholder_range(),
        ));
    }
}

/// Fix spacing at the start of a value's first token's leading trivia.
fn fix_value_leading_spacing(value: &mut CstValue) {
    let token = first_token_mut(value);
    if let Some(token) = token {
        let trivia = &mut token.leading_trivia;
        // Ensure first piece of trivia is a single space
        if trivia.is_empty() {
            trivia.push(Trivia::new(
                TriviaKind::Whitespace,
                " ".to_string(),
                placeholder_range(),
            ));
        } else if trivia[0].kind == TriviaKind::Whitespace {
            trivia[0].text = " ".to_string();
        } else {
            trivia.insert(
                0,
                Trivia::new(
                    TriviaKind::Whitespace,
                    " ".to_string(),
                    placeholder_range(),
                ),
            );
        }
    }
}

fn first_token_mut(value: &mut CstValue) -> Option<&mut CstToken> {
    match value {
        CstValue::Ident(t)
        | CstValue::String(t)
        | CstValue::Number(t)
        | CstValue::Boolean(t) => Some(t.as_mut()),
        CstValue::Block(b) => Some(&mut b.open_brace),
        CstValue::TaggedBlock { tag, .. } => Some(tag.as_mut()),
        CstValue::Error(_) => None,
    }
}

// ---------------------------------------------------------------------------
// fix_brace_style
// ---------------------------------------------------------------------------

/// Ensure `{` is on the same line as the key/value (no newline before `{`).
pub(crate) fn fix_brace_style(cst: &mut CstScript) {
    for node in &mut cst.nodes {
        fix_node_brace_style(node);
    }
}

fn fix_node_brace_style(node: &mut CstNode) {
    match node {
        CstNode::Assignment(ass) => {
            fix_value_brace_style(&mut ass.value);
        }
        CstNode::EntryValue(ev) => {
            fix_value_brace_style(&mut ev.value);
        }
        _ => {}
    }
}

fn fix_value_brace_style(value: &mut CstValue) {
    match value {
        CstValue::Block(block) => {
            fix_block_brace_style(block);
            // Recurse
            for entry in &mut block.entries {
                fix_node_brace_style(entry);
            }
        }
        CstValue::TaggedBlock { block, .. } => {
            fix_block_brace_style(block);
            // Recurse
            for entry in &mut block.entries {
                fix_node_brace_style(entry);
            }
        }
        _ => {}
    }
}

fn fix_block_brace_style(block: &mut CstBlock) {
    // Remove newlines from open_brace's leading_trivia to keep { on same line
    block.open_brace.leading_trivia.retain(|t| t.kind != TriviaKind::Newline);
    // Ensure at least a space before {
    if block.open_brace.leading_trivia.is_empty() {
        block.open_brace.leading_trivia.push(Trivia::new(
            TriviaKind::Whitespace,
            " ".to_string(),
            placeholder_range(),
        ));
    } else if let Some(last) = block.open_brace.leading_trivia.last_mut() {
        if last.kind == TriviaKind::Whitespace {
            last.text = " ".to_string();
        }
    }
}

// ---------------------------------------------------------------------------
// fix_trailing_whitespace
// ---------------------------------------------------------------------------

/// Remove trailing whitespace from each line (any Whitespace immediately
/// followed by Newline).
pub(crate) fn fix_trailing_whitespace(cst: &mut CstScript) {
    fn process_trivia(trivia: &mut Vec<Trivia>) {
        let mut i = 0;
        while i + 1 < trivia.len() {
            if trivia[i].kind == TriviaKind::Whitespace
                && trivia[i + 1].kind == TriviaKind::Newline
            {
                trivia.remove(i);
                // Don't increment i — the newline at position i may follow
                // another whitespace piece.
            } else {
                i += 1;
            }
        }
    }

    fn process_token(token: &mut CstToken) {
        process_trivia(&mut token.leading_trivia);
    }

    fn process_node(node: &mut CstNode) {
        match node {
            CstNode::Assignment(ass) => {
                process_token(&mut ass.key);
                process_token(&mut ass.operator);
                process_value(&mut ass.value);
            }
            CstNode::EntryValue(ev) => {
                process_value(&mut ev.value);
            }
            CstNode::EntryComment(_) | CstNode::Error(_) => {}
        }
    }

    fn process_value(value: &mut CstValue) {
        match value {
            CstValue::Ident(t)
            | CstValue::String(t)
            | CstValue::Number(t)
            | CstValue::Boolean(t) => {
                process_token(t);
            }
            CstValue::Block(block) => {
                process_token(&mut block.open_brace);
                for entry in &mut block.entries {
                    process_node(entry);
                }
                match &mut block.close_brace {
                    CloseBrace::Present(token) => {
                        process_token(token);
                    }
                    CloseBrace::Missing(_) => {}
                }
            }
            CstValue::TaggedBlock { block, .. } => {
                process_token(&mut block.open_brace);
                for entry in &mut block.entries {
                    process_node(entry);
                }
                match &mut block.close_brace {
                    CloseBrace::Present(token) => {
                        process_token(token);
                    }
                    CloseBrace::Missing(_) => {}
                }
            }
            CstValue::Error(_) => {}
        }
    }

    for node in &mut cst.nodes {
        process_node(node);
    }
    process_trivia(&mut cst.trailing_trivia);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn placeholder_range() -> ast::Range {
    ast::Range {
        start_line: 0,
        start_col: 0,
        end_line: 0,
        end_col: 0,
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::cst::lexer::tokenize;
    use crate::parser::cst::parser::parse_cst;
    use super::*;

    fn format_str(input: &str) -> String {
        let (tokens, _) = tokenize(input);
        let mut cst = parse_cst(tokens);
        format(&mut cst);
        crate::formatter::serialize::serialize(&cst)
    }

    #[test]
    fn test_basic_indent() {
        // Input with spaces → output with tabs
        let input = "a = {\n    b = 1\n}\n";
        let expected = "a = {\n\tb = 1\n}\n";
        assert_eq!(format_str(input), expected);
    }

    #[test]
    fn test_assignment_spacing() {
        let input = "key  =  value\n";
        let expected = "key = value\n";
        assert_eq!(format_str(input), expected);
    }

    #[test]
    fn test_brace_on_same_line() {
        // Open brace should be on same line as key
        let input = "key =\n{\n}\n";
        let expected = "key = {\n}\n";
        assert_eq!(format_str(input), expected);
    }

    #[test]
    fn test_trailing_whitespace() {
        let input = "key = value  \n";
        let expected = "key = value\n";
        assert_eq!(format_str(input), expected);
    }

    #[test]
    fn test_full_format_pipeline() {
        let input = "a   =  {  \n    b  =   2\n    c   >   3  \n}\n";
        let expected = "a = {\n\tb = 2\n\tc > 3\n}\n";
        assert_eq!(format_str(input), expected);
    }

    #[test]
    fn test_nested_indent() {
        let input = "outer = {\n    inner = {\n        x = 1\n    }\n}\n";
        let expected = "outer = {\n\tinner = {\n\t\tx = 1\n\t}\n}\n";
        assert_eq!(format_str(input), expected);
    }

    #[test]
    fn test_mixed_operators_spacing() {
        let input = "a  <  1\nb  <=  2\nc  >  3\nd  >=  4\ne  !=  5\n";
        let expected = "a < 1\nb <= 2\nc > 3\nd >= 4\ne != 5\n";
        assert_eq!(format_str(input), expected);
    }

    #[test]
    fn test_trailing_whitespace_multiple_lines() {
        let input = "a = 1  \nb = 2  \nc = 3  \n";
        let expected = "a = 1\nb = 2\nc = 3\n";
        assert_eq!(format_str(input), expected);
    }
}
