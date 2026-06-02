//! AST Lowering — convert CST to existing AST types for semantic consumers.

use crate::parser::ast;
use crate::parser::cst::diagnostic::*;
use crate::parser::cst::token::*;
use crate::parser::cst::types::*;

/// Convert a CstScript to the existing AST format.
///
/// Returns `(ast::Script, Vec<(String, ast::Range)>)` matching the current
/// `parser::parse_script` return type exactly.
pub fn lower(cst: CstScript) -> (ast::Script, Vec<(String, ast::Range)>) {
    let mut entries: Vec<ast::Entry> = Vec::new();
    let mut diagnostics: Vec<(String, ast::Range)> = Vec::new();

    // Convert all CST diagnostics to the old (message, range) format.
    for diag in &cst.diagnostics {
        diagnostics.push((diag.message.clone(), diag.range.clone()));
    }

    // Lower each CST node into an AST entry.
    for node in cst.nodes {
        if let Some(entry) = lower_node(node) {
            entries.push(entry);
        }
        // Error nodes are skipped (they contribute only to diagnostics already captured above).
    }

    (ast::Script { entries }, diagnostics)
}

/// Convert a single `CstNode` into an optional `ast::Entry`.
///
/// Returns `None` for `CstNode::Error` — those contribute only to
/// diagnostics and should not produce AST entries.
fn lower_node(node: CstNode) -> Option<ast::Entry> {
    match node {
        CstNode::Assignment(ass) => {
            let operator = operator_from_token_kind(&ass.operator.kind);
            let value = lower_value_to_nodeed(ass.value);
            Some(ast::Entry::Assignment(ast::Assignment {
                key: ass.key.text.clone(),
                key_range: ass.key.range.clone(),
                operator,
                operator_range: ass.operator.range.clone(),
                value,
            }))
        }
        CstNode::EntryValue(ev) => {
            let nodeed = lower_value_to_nodeed(*ev.value);
            Some(ast::Entry::Value(nodeed))
        }
        CstNode::EntryComment(trivia) => {
            let text = trivia
                .text
                .strip_prefix('#')
                .unwrap_or(&trivia.text)
                .to_string();
            Some(ast::Entry::Comment(text, trivia.range.clone()))
        }
        CstNode::Error(_) => None,
    }
}

/// Lower a `CstValue` into an `ast::Value` (without range wrapping).
fn lower_value(value: CstValue) -> ast::Value {
    match value {
        CstValue::Ident(token) => ast::Value::String(token.text.clone()),
        CstValue::String(token) => {
            // Use the resolved string content from TokenKind::String, which
            // has escape sequences processed.  Fall back to stripping quotes
            // from the source text for any other TokenKind.
            let content = match &token.kind {
                TokenKind::String(s) => s.clone(),
                _ => token
                    .text
                    .strip_prefix('"')
                    .and_then(|s| s.strip_suffix('"'))
                    .unwrap_or(&token.text)
                    .to_string(),
            };
            ast::Value::String(content)
        }
        CstValue::Number(token) => ast::Value::Number(token.text.parse::<f64>().unwrap()),
        CstValue::Boolean(token) => ast::Value::Boolean(token.text == "yes"),
        CstValue::Block(block) => ast::Value::Block(lower_block(&block)),
        CstValue::TaggedBlock { tag, block } => {
            let entries = lower_block(&block);
            let block_range = combined_range(
                &tag.range,
                &block
                    .close_brace
                    .range()
                    .unwrap_or_else(|| tag.range.clone()),
            );
            ast::Value::TaggedBlock(tag.text.clone(), entries, block_range)
        }
        CstValue::Error(_) => ast::Value::String(String::new()),
    }
}

/// Lower a `CstValue` into an `ast::NodeedValue` (value + range wrapping).
fn lower_value_to_nodeed(value: CstValue) -> ast::NodeedValue {
    let range = value_range(&value);
    let inner = lower_value(value);
    ast::NodeedValue {
        value: inner,
        range,
    }
}

/// Extract the `ast::Range` from a `CstValue` for use in `NodeedValue`.
fn value_range(value: &CstValue) -> ast::Range {
    match value {
        CstValue::Ident(t) | CstValue::String(t) | CstValue::Number(t) | CstValue::Boolean(t) => {
            t.range.clone()
        }
        CstValue::Block(block) => {
            let start = block.open_brace.range.clone();
            let end = block.close_brace.range().unwrap_or_else(|| start.clone());
            combined_range(&start, &end)
        }
        CstValue::TaggedBlock { tag, block } => {
            let start = tag.range.clone();
            let end = block
                .close_brace
                .range()
                .unwrap_or_else(|| tag.range.clone());
            combined_range(&start, &end)
        }
        CstValue::Error(diag) => diag.range.clone(),
    }
}

/// Combine two ranges into one spanning from the start of the first to the
/// end of the second.
fn combined_range(start: &ast::Range, end: &ast::Range) -> ast::Range {
    ast::Range {
        start_line: start.start_line,
        start_col: start.start_col,
        end_line: end.end_line,
        end_col: end.end_col,
    }
}

/// Convert a `CstBlock` into `Vec<ast::Entry>`.
fn lower_block(block: &CstBlock) -> Vec<ast::Entry> {
    let mut entries = Vec::new();
    for node in &block.entries {
        if let Some(entry) = lower_node(node.clone()) {
            entries.push(entry);
        }
    }
    entries
}

/// Map a `TokenKind` operator to the corresponding `ast::Operator`.
///
/// Falls back to `Operator::Equals` for non-operator token kinds.
fn operator_from_token_kind(kind: &TokenKind) -> ast::Operator {
    match kind {
        TokenKind::OpEquals => ast::Operator::Equals,
        TokenKind::OpLessThan => ast::Operator::LessThan,
        TokenKind::OpGreaterThan => ast::Operator::GreaterThan,
        TokenKind::OpNotEquals => ast::Operator::NotEquals,
        TokenKind::OpLessOrEqual => ast::Operator::LessOrEqual,
        TokenKind::OpGreaterOrEqual => ast::Operator::GreaterOrEqual,
        _ => ast::Operator::Equals,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::cst::lexer;
    use crate::parser::cst::parser::parse_cst;

    /// Tokenize, parse, and lower a source string into the AST.
    fn lower_str(input: &str) -> (ast::Script, Vec<(String, ast::Range)>) {
        let (tokens, _) = lexer::tokenize(input);
        let cst = parse_cst(tokens);
        lower(cst)
    }

    // ── End-to-end tests via lower_str ──────────────────────────────────

    #[test]
    fn test_empty_input() {
        let (script, errors) = lower_str("");
        assert_eq!(script.entries.len(), 0, "empty input should have 0 entries");
        assert_eq!(errors.len(), 0, "empty input should have 0 errors");
    }

    #[test]
    fn test_simple_assignment() {
        let (script, errors) = lower_str("key = val");
        assert_eq!(errors.len(), 0);
        assert_eq!(script.entries.len(), 1);
        match &script.entries[0] {
            ast::Entry::Assignment(assign) => {
                assert_eq!(assign.key, "key");
                assert!(matches!(assign.operator, ast::Operator::Equals));
                match &assign.value.value {
                    ast::Value::String(s) => assert_eq!(s, "val"),
                    other => panic!("Expected String value, got {:?}", other),
                }
            }
            other => panic!("Expected Assignment, got {:?}", other),
        }
    }

    #[test]
    fn test_block_assignment() {
        let (script, errors) = lower_str("e = { id = 1 }");
        assert_eq!(errors.len(), 0);
        assert_eq!(script.entries.len(), 1);
        match &script.entries[0] {
            ast::Entry::Assignment(assign) => {
                assert_eq!(assign.key, "e");
                match &assign.value.value {
                    ast::Value::Block(entries) => {
                        assert_eq!(entries.len(), 1);
                        match &entries[0] {
                            ast::Entry::Assignment(inner) => {
                                assert_eq!(inner.key, "id");
                                match &inner.value.value {
                                    ast::Value::Number(n) => assert_eq!(*n, 1.0),
                                    other => panic!("Expected Number, got {:?}", other),
                                }
                            }
                            other => panic!("Expected inner Assignment, got {:?}", other),
                        }
                    }
                    other => panic!("Expected Block, got {:?}", other),
                }
            }
            other => panic!("Expected Assignment, got {:?}", other),
        }
    }

    #[test]
    fn test_tagged_block() {
        let (script, errors) = lower_str("modifier = my_tag { f = 0.5 }");
        assert_eq!(errors.len(), 0);
        assert_eq!(script.entries.len(), 1);
        match &script.entries[0] {
            ast::Entry::Assignment(assign) => {
                assert_eq!(assign.key, "modifier");
                match &assign.value.value {
                    ast::Value::TaggedBlock(tag, entries, _range) => {
                        assert_eq!(tag, "my_tag");
                        assert_eq!(entries.len(), 1);
                        match &entries[0] {
                            ast::Entry::Assignment(inner) => {
                                assert_eq!(inner.key, "f");
                                match &inner.value.value {
                                    ast::Value::Number(n) => assert!((*n - 0.5).abs() < 1e-10),
                                    other => panic!("Expected Number, got {:?}", other),
                                }
                            }
                            other => panic!("Expected Assignment, got {:?}", other),
                        }
                    }
                    other => panic!("Expected TaggedBlock, got {:?}", other),
                }
            }
            other => panic!("Expected Assignment, got {:?}", other),
        }
    }

    #[test]
    fn test_boolean_values() {
        let (script, errors) = lower_str("a = yes\nb = no");
        assert_eq!(errors.len(), 0);
        assert_eq!(script.entries.len(), 2);

        match &script.entries[0] {
            ast::Entry::Assignment(a) => match &a.value.value {
                ast::Value::Boolean(b) => assert!(*b, "expected true"),
                other => panic!("Expected Boolean, got {:?}", other),
            },
            other => panic!("Expected Assignment, got {:?}", other),
        }
        match &script.entries[1] {
            ast::Entry::Assignment(b) => match &b.value.value {
                ast::Value::Boolean(v) => assert!(!*v, "expected false"),
                other => panic!("Expected Boolean, got {:?}", other),
            },
            other => panic!("Expected Assignment, got {:?}", other),
        }
    }

    #[test]
    fn test_string_value() {
        let (script, errors) = lower_str("a = \"hello\"");
        assert_eq!(errors.len(), 0);
        assert_eq!(script.entries.len(), 1);
        match &script.entries[0] {
            ast::Entry::Assignment(a) => match &a.value.value {
                ast::Value::String(s) => assert_eq!(s, "hello"),
                other => panic!("Expected String, got {:?}", other),
            },
            other => panic!("Expected Assignment, got {:?}", other),
        }
    }

    #[test]
    fn test_number_values() {
        let (script, errors) = lower_str("a = 42\nb = -0.15");
        assert_eq!(errors.len(), 0);
        assert_eq!(script.entries.len(), 2);

        match &script.entries[0] {
            ast::Entry::Assignment(a) => match &a.value.value {
                ast::Value::Number(n) => assert_eq!(*n, 42.0),
                other => panic!("Expected Number, got {:?}", other),
            },
            other => panic!("Expected Assignment, got {:?}", other),
        }
        match &script.entries[1] {
            ast::Entry::Assignment(b) => match &b.value.value {
                ast::Value::Number(n) => assert!((*n - (-0.15)).abs() < 1e-10),
                other => panic!("Expected Number, got {:?}", other),
            },
            other => panic!("Expected Assignment, got {:?}", other),
        }
    }

    #[test]
    fn test_special_idents() {
        let (script, errors) = lower_str("title = daw.2.t");
        assert_eq!(errors.len(), 0);
        assert_eq!(script.entries.len(), 1);
        match &script.entries[0] {
            ast::Entry::Assignment(a) => {
                assert_eq!(a.key, "title");
                match &a.value.value {
                    ast::Value::String(s) => assert_eq!(s, "daw.2.t"),
                    other => panic!("Expected String, got {:?}", other),
                }
            }
            other => panic!("Expected Assignment, got {:?}", other),
        }
    }

    #[test]
    fn test_missing_close_brace() {
        let (script, errors) = lower_str("e = { id = 1");
        // Should still produce AST entries
        assert!(
            script.entries.len() >= 1,
            "expected at least one entry despite missing close brace"
        );
        // Should have at least one diagnostic for the missing close brace
        assert!(
            errors.len() >= 1,
            "expected diagnostic for missing close brace"
        );
    }

    #[test]
    fn test_multiple_entries() {
        let (script, errors) = lower_str("a = 1\nb = 2\nc = 3");
        assert_eq!(errors.len(), 0);
        assert_eq!(script.entries.len(), 3);
    }

    #[test]
    fn test_bare_value() {
        let (script, errors) = lower_str("{ a = 1 }");
        assert_eq!(errors.len(), 0);
        assert_eq!(script.entries.len(), 1);
        match &script.entries[0] {
            ast::Entry::Value(nodeed) => match &nodeed.value {
                ast::Value::Block(entries) => {
                    assert_eq!(entries.len(), 1);
                    match &entries[0] {
                        ast::Entry::Assignment(inner) => {
                            assert_eq!(inner.key, "a");
                            match &inner.value.value {
                                ast::Value::Number(n) => assert_eq!(*n, 1.0),
                                other => panic!("Expected Number, got {:?}", other),
                            }
                        }
                        other => panic!("Expected Assignment, got {:?}", other),
                    }
                }
                other => panic!("Expected Block, got {:?}", other),
            },
            other => panic!("Expected Value, got {:?}", other),
        }
    }

    #[test]
    fn test_all_operators() {
        let cases = [
            ("a = 1", ast::Operator::Equals),
            ("a < 1", ast::Operator::LessThan),
            ("a > 1", ast::Operator::GreaterThan),
            ("a != 1", ast::Operator::NotEquals),
            ("a <= 1", ast::Operator::LessOrEqual),
            ("a >= 1", ast::Operator::GreaterOrEqual),
        ];
        for (input, expected_op) in &cases {
            let (script, errors) = lower_str(input);
            assert_eq!(errors.len(), 0, "input: {input}");
            assert_eq!(script.entries.len(), 1, "input: {input}");
            match &script.entries[0] {
                ast::Entry::Assignment(assign) => {
                    assert_eq!(assign.key, "a");
                    assert!(
                        std::mem::discriminant(&assign.operator)
                            == std::mem::discriminant(expected_op),
                        "input: {input}, expected operator {expected_op:?}, got {:?}",
                        assign.operator
                    );
                }
                other => panic!("Expected Assignment, got {:?}", other),
            }
        }
    }

    #[test]
    fn test_nested_blocks() {
        let (script, errors) = lower_str("a = { b = { c = 1 } }");
        assert_eq!(errors.len(), 0);
        assert_eq!(script.entries.len(), 1);
        match &script.entries[0] {
            ast::Entry::Assignment(a) => match &a.value.value {
                ast::Value::Block(outer) => {
                    assert_eq!(outer.len(), 1);
                    match &outer[0] {
                        ast::Entry::Assignment(b) => match &b.value.value {
                            ast::Value::Block(inner) => {
                                assert_eq!(inner.len(), 1);
                                match &inner[0] {
                                    ast::Entry::Assignment(c) => {
                                        assert_eq!(c.key, "c");
                                        match &c.value.value {
                                            ast::Value::Number(n) => assert_eq!(*n, 1.0),
                                            other => panic!("Expected Number, got {:?}", other),
                                        }
                                    }
                                    other => panic!("Expected Assignment, got {:?}", other),
                                }
                            }
                            other => panic!("Expected Block, got {:?}", other),
                        },
                        other => panic!("Expected Assignment, got {:?}", other),
                    }
                }
                other => panic!("Expected Block, got {:?}", other),
            },
            other => panic!("Expected Assignment, got {:?}", other),
        }
    }

    // ── Direct lower_node tests for variants not produced by the current parser ──

    #[test]
    fn test_entry_comment() {
        // The current parser stores comments as trivia on tokens, not as
        // CstNode::EntryComment.  Test the lowering path directly.
        let trivia = Trivia::new(
            TriviaKind::Comment,
            "#   a comment with extra spacing   ".to_string(),
            ast::Range {
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 1,
            },
        );
        let node = CstNode::EntryComment(trivia);
        match lower_node(node) {
            Some(ast::Entry::Comment(text, range)) => {
                assert_eq!(text, "   a comment with extra spacing   ");
                assert_eq!(range.start_line, 0);
            }
            other => panic!("Expected Comment entry, got {:?}", other),
        }
    }

    #[test]
    fn test_entry_comment_minimal() {
        let trivia = Trivia::new(
            TriviaKind::Comment,
            "#comment".to_string(),
            ast::Range {
                start_line: 1,
                start_col: 0,
                end_line: 1,
                end_col: 8,
            },
        );
        let node = CstNode::EntryComment(trivia);
        match lower_node(node) {
            Some(ast::Entry::Comment(text, range)) => {
                assert_eq!(text, "comment");
                assert_eq!(range.start_line, 1);
            }
            other => panic!("Expected Comment entry, got {:?}", other),
        }
    }

    #[test]
    fn test_error_node_skipped() {
        let diag = CstDiagnostic::error(
            "test error",
            ast::Range {
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 0,
            },
        );
        let node = CstNode::Error(diag);
        assert!(lower_node(node).is_none(), "Error node should be skipped");
    }
}
