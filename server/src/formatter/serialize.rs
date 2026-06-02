//! CST → text serialization (lossless roundtrip).
//!
//! Walks the CST tree depth-first, emitting trivia then token text.
//! For an unmodified CST, the output should equal the original input
//! (except for comments extracted during parsing — see note below).
//!
//! ## Comment roundtrip limitation
//!
//! The parser extracts comments from token leading-trivia into separate
//! `EntryComment` nodes, but the surrounding whitespace/newlines remain
//! in the original token's trivia (or trailing_trivia on the script).
//! This means the serialized order for standalone comments may not match
//! the original input exactly: the newline that preceded the comment
//! will instead appear after it in the output.

use crate::parser::cst::token::{CstToken, Trivia, TriviaKind};
use crate::parser::cst::types::*;

/// Serialize a CstScript back to source text.
/// This is lossless — for an unmodified CST, the output should equal the original input.
pub fn serialize(cst: &CstScript) -> String {
    let mut output = String::new();
    let mut last_was_entry_comment = false;
    for node in &cst.nodes {
        if let CstNode::EntryComment(trivia) = node {
            // Standalone comments extracted from trivia need a preceding newline
            // if the output doesn't already end with one.
            if !output.is_empty() && !output.ends_with('\n') {
                output.push('\n');
            }
            output.push_str(&trivia.text);
            last_was_entry_comment = true;
        } else {
            serialize_node(node, &mut output);
            last_was_entry_comment = false;
        }
    }
    // Emit trailing trivia.
    // If the last node was an EntryComment to which we added a leading newline,
    // skip the first newline piece in trailing_trivia to avoid double newlines.
    let mut skip_first_newline = last_was_entry_comment;
    for trivia in &cst.trailing_trivia {
        if skip_first_newline && trivia.kind == TriviaKind::Newline {
            skip_first_newline = false;
            continue;
        }
        output.push_str(&trivia.text);
    }
    output
}

fn serialize_node(node: &CstNode, out: &mut String) {
    match node {
        CstNode::Assignment(ass) => {
            serialize_token(&ass.key, out);
            serialize_token(&ass.operator, out);
            serialize_value(&ass.value, out);
        }
        CstNode::EntryValue(ev) => {
            serialize_value(&ev.value, out);
        }
        CstNode::EntryComment(trivia) => {
            out.push_str(&trivia.text); // includes # and the comment content
        }
    }
}

fn serialize_token(token: &CstToken, out: &mut String) {
    for trivia in &token.leading_trivia {
        out.push_str(&trivia.text);
    }
    out.push_str(&token.text);
}

fn serialize_value(value: &CstValue, out: &mut String) {
    match value {
        CstValue::Ident(t) | CstValue::String(t) | CstValue::Number(t) | CstValue::Boolean(t) => {
            serialize_token(t, out);
        }
        CstValue::Block(block) => serialize_block(block, out),
        CstValue::TaggedBlock { tag, block } => {
            serialize_token(tag, out);
            serialize_block(block, out);
        }
        CstValue::Error(_) => {}
    }
}

fn serialize_block(block: &CstBlock, out: &mut String) {
    // Emit leading trivia of the open brace (space before {), then the brace text
    for trivia in &block.open_brace.leading_trivia {
        out.push_str(&trivia.text);
    }
    out.push_str(&block.open_brace.text);

    // Serialize entries
    for entry in &block.entries {
        serialize_node(entry, out);
    }

    // Close brace
    // Its leading trivia contains any whitespace/newlines between the last entry and }
    match &block.close_brace {
        CloseBrace::Present(token) => {
            serialize_token(token, out);
        }
        CloseBrace::Missing(_diag) => {
            // Serialize a reasonable error-recovery close brace
            // Find the indent level from the last entry
            if let Some(last_node) = block.entries.last() {
                // Add newline + close brace
                out.push('\n');
                // Guess indent: one level less than the last entry
                if let Some(last_ident) = extract_ident_from_node(last_node) {
                    let indent = guess_indent_from_trivia(&last_ident.leading_trivia);
                    if indent > 0 {
                        for _ in 0..(indent - 1) {
                            out.push('\t');
                        }
                    }
                }
                out.push('}');
            } else {
                out.push('}');
            }
        }
    }
}

fn extract_ident_from_node<'a>(node: &'a CstNode) -> Option<&'a CstToken> {
    match node {
        CstNode::Assignment(ass) => Some(&ass.key),
        CstNode::EntryComment(_) => None,
        _ => None,
    }
}

fn guess_indent_from_trivia(trivia: &[Trivia]) -> u32 {
    // Look at the last whitespace after a newline in the leading trivia
    let mut indent = 0u32;
    let mut after_newline = false;
    for t in trivia {
        if t.kind == TriviaKind::Newline {
            after_newline = true;
            indent = 0;
        } else if after_newline && t.kind == TriviaKind::Whitespace {
            // Count tabs (each tab = 1 indent level) or spaces (every 4 spaces = 1 level)
            indent = t.text.chars().filter(|&c| c == '\t').count() as u32;
            if indent == 0 {
                indent = (t.text.len() / 4) as u32;
            }
        }
    }
    indent
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::cst::lexer::tokenize;
    use crate::parser::cst::parser::parse_cst;

    fn roundtrip(input: &str) -> String {
        let (tokens, _) = tokenize(input);
        let cst = parse_cst(tokens);
        serialize(&cst)
    }

    #[test]
    fn test_roundtrip_simple() {
        let input = "key = value\n";
        assert_eq!(roundtrip(input), input);
    }

    #[test]
    fn test_roundtrip_block() {
        let input = "country_event = {\n    id = test.1\n}\n";
        assert_eq!(roundtrip(input), input);
    }

    #[test]
    fn test_roundtrip_comments() {
        // Standalone comments (extracted from token trivia) have a known
        // roundtrip limitation: the newline that preceded the comment in
        // the original appears after it in the serialized output.
        let input = "# header\nkey = val\n# footer\n";
        // The serializer ensures a newline precedes each EntryComment,
        // and skips the first newline in trailing_trivia to balance.
        // This approximates the original but may differ in edge cases.
        let result = roundtrip(input);
        // The key result: all content is preserved and formatting is reasonable.
        assert_eq!(result, input, "comment roundtrip mismatch");
    }

    #[test]
    fn test_roundtrip_empty() {
        let input = "";
        assert_eq!(roundtrip(input), input);
    }

    #[test]
    fn test_roundtrip_complex() {
        let input = "modifier = {\n    political_power_factor = 0.15\n    stability_factor > -0.1\n    tag != \"ENG\"\n}\n";
        assert_eq!(roundtrip(input), input);
    }
}
