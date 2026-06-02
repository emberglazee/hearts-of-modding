//! CST Parser — recursive descent from token stream with error recovery.

use crate::parser::ast;
use crate::parser::cst::diagnostic::*;
use crate::parser::cst::token::*;
use crate::parser::cst::types::*;

/// Parse a token stream into a CST script.
///
/// The input `tokens` should be the output of `lexer::tokenize()`.
/// The last token must always be `TokenKind::Eof`.
pub fn parse_cst(tokens: Vec<CstToken>) -> CstScript {
    CstParser::new(tokens).parse()
}

/// Internal recursive-descent parser with error recovery.
struct CstParser {
    tokens: Vec<CstToken>,
    pos: usize,
    diagnostics: Vec<CstDiagnostic>,
}

impl CstParser {
    fn new(tokens: Vec<CstToken>) -> Self {
        Self {
            tokens,
            pos: 0,
            diagnostics: Vec::new(),
        }
    }

    fn parse(mut self) -> CstScript {
        self.parse_script()
    }

    // ── Core parse functions ──────────────────────────────────────────

    /// Parse the top-level script.
    fn parse_script(&mut self) -> CstScript {
        let mut nodes: Vec<CstNode> = Vec::new();

        loop {
            // Drain standalone comments from the current token first (even if EOF)
            if let Some(comment) = self.drain_comment() {
                nodes.push(comment);
                continue;
            }

            if self.is_at_end() || self.is_eof(self.peek()) {
                break;
            }

            // At the script level, a CloseBrace has no enclosing block to
            // propagate to — consume it and emit an error.
            if self.is_close_brace(self.peek()) {
                let token = self.advance().expect("CloseBrace token");
                self.emit_error("Unexpected '}'", token.range.clone());
                continue;
            }

            match self.parse_entry() {
                Some(node) => nodes.push(node),
                None => self.skip_to_entry_boundary(),
            }
        }

        // Extract trailing trivia from the EOF token AFTER draining comments
        let trailing_trivia = self
            .tokens
            .last()
            .map(|eof| eof.leading_trivia.clone())
            .unwrap_or_default();

        CstScript {
            nodes,
            diagnostics: std::mem::take(&mut self.diagnostics),
            trailing_trivia,
        }
    }

    /// Parse a single entry (assignment, bare value, or error).
    ///
    /// Returns `None` if the current token does not start an entry
    /// (e.g. `CloseBrace` or `Eof`), letting the caller handle recovery.
    fn parse_entry(&mut self) -> Option<CstNode> {
        // Always drain comments from the current token's leading trivia first.
        // This handles both standalone comment lines and inline comments after
        // values (e.g. `key = val # comment\nnext = 1`).
        if let Some(comment_node) = self.drain_comment() {
            return Some(comment_node);
        }

        let kind = self.peek_kind()?.clone();

        match kind {
            TokenKind::OpenBrace => {
                let value = self.parse_value();
                Some(CstNode::EntryValue(CstEntryValue::new(value)))
            }

            TokenKind::Ident(_) => {
                let ident = self.advance().expect("Ident token");
                match self.peek_kind() {
                    // Operator follows → this is a key=value assignment
                    Some(TokenKind::OpEquals)
                    | Some(TokenKind::OpLessThan)
                    | Some(TokenKind::OpGreaterThan)
                    | Some(TokenKind::OpNotEquals)
                    | Some(TokenKind::OpLessOrEqual)
                    | Some(TokenKind::OpGreaterOrEqual) => {
                        Some(CstNode::Assignment(self.parse_assignment(ident)))
                    }
                    // OpenBrace follows → this is a TaggedBlock bare value
                    Some(TokenKind::OpenBrace) => {
                        let block = self.parse_block();
                        Some(CstNode::EntryValue(CstEntryValue::new(
                            CstValue::TaggedBlock {
                                tag: Box::new(ident),
                                block: Box::new(block),
                            },
                        )))
                    }
                    // Otherwise the ident is a bare value on its own
                    _ => Some(CstNode::EntryValue(CstEntryValue::new(classify_ident(
                        ident,
                    )))),
                }
            }

            TokenKind::String(_) => {
                let token = self.advance().expect("String token");
                Some(CstNode::EntryValue(CstEntryValue::new(CstValue::String(
                    Box::new(token),
                ))))
            }

            TokenKind::Number(_) => {
                let token = self.advance().expect("Number token");
                match self.peek_kind() {
                    // Operator follows → this is a key=value assignment
                    Some(TokenKind::OpEquals)
                    | Some(TokenKind::OpLessThan)
                    | Some(TokenKind::OpGreaterThan)
                    | Some(TokenKind::OpNotEquals)
                    | Some(TokenKind::OpLessOrEqual)
                    | Some(TokenKind::OpGreaterOrEqual) => {
                        Some(CstNode::Assignment(self.parse_assignment(token)))
                    }
                    // OpenBrace follows → tagged block
                    Some(TokenKind::OpenBrace) => {
                        let block = self.parse_block();
                        Some(CstNode::EntryValue(CstEntryValue::new(
                            CstValue::TaggedBlock {
                                tag: Box::new(token),
                                block: Box::new(block),
                            },
                        )))
                    }
                    // Otherwise it's a bare number value
                    _ => Some(CstNode::EntryValue(CstEntryValue::new(CstValue::Number(
                        Box::new(token),
                    )))),
                }
            }

            // These tokens belong to the enclosing block; signal the caller
            // to stop parsing entries.
            TokenKind::CloseBrace => None,
            TokenKind::Eof => None,

            // Anything else is unexpected at the entry level.
            _ => {
                let token = self.peek().expect("current token");
                let range = token.range.clone();
                self.emit_error(format!("Unexpected token: {}", token.kind), range);
                None
            }
        }
    }

    /// Parse `= value` after the key has already been consumed.
    fn parse_assignment(&mut self, key: CstToken) -> CstAssignment {
        let operator = self.advance().expect("operator token (already peeked)");
        let value = self.parse_value();

        if let CstValue::Error(_) = &value {
            self.emit_error("Expected value after operator", key.range.clone());
        }

        CstAssignment {
            key,
            operator,
            value,
        }
    }

    /// Parse any value — block, tagged block, literal, or identifier.
    fn parse_value(&mut self) -> CstValue {
        let kind = match self.peek_kind() {
            Some(k) => k.clone(),
            None => {
                return CstValue::Error(CstDiagnostic::error(
                    "Unexpected end of file",
                    ast::Range {
                        start_line: 0,
                        start_col: 0,
                        end_line: 0,
                        end_col: 0,
                    },
                ));
            }
        };

        match kind {
            TokenKind::OpenBrace => CstValue::Block(self.parse_block()),

            TokenKind::Ident(_) => {
                let ident = self.advance().expect("Ident token");
                match self.peek_kind() {
                    Some(TokenKind::OpenBrace) => CstValue::TaggedBlock {
                        tag: Box::new(ident),
                        block: Box::new(self.parse_block()),
                    },
                    _ => classify_ident(ident),
                }
            }

            TokenKind::String(_) => {
                let token = self.advance().expect("String token");
                CstValue::String(Box::new(token))
            }

            TokenKind::Number(_) => {
                let token = self.advance().expect("Number token");
                CstValue::Number(Box::new(token))
            }

            TokenKind::CloseBrace => {
                let token = self.advance().expect("CloseBrace token");
                self.emit_error("Unexpected '}'", token.range.clone());
                CstValue::Error(CstDiagnostic::error("Unexpected '}'", token.range))
            }

            TokenKind::Eof => {
                self.emit_error(
                    "Unexpected end of file",
                    ast::Range {
                        start_line: 0,
                        start_col: 0,
                        end_line: 0,
                        end_col: 0,
                    },
                );
                CstValue::Error(CstDiagnostic::error(
                    "Unexpected end of file",
                    ast::Range {
                        start_line: 0,
                        start_col: 0,
                        end_line: 0,
                        end_col: 0,
                    },
                ))
            }

            // Operators without a preceding key are invalid values.
            _ => {
                let (kind_str, range) = {
                    let token = self.peek().expect("current token");
                    (token.kind.to_string(), token.range.clone())
                };
                self.emit_error(
                    format!("Unexpected operator {} without key", kind_str),
                    range.clone(),
                );
                CstValue::Error(CstDiagnostic::error(
                    format!("Unexpected operator {} without key", kind_str),
                    range,
                ))
            }
        }
    }

    /// Parse `{ entries }` with optional missing-close-brace recovery.
    fn parse_block(&mut self) -> CstBlock {
        let open = self.advance().expect("OpenBrace token");
        let open_range = open.range.clone();

        let mut entries: Vec<CstNode> = Vec::new();

        loop {
            if self.is_at_end() || self.is_eof(self.peek()) {
                break;
            }
            if self.is_close_brace(self.peek()) {
                break;
            }
            match self.parse_entry() {
                Some(node) => entries.push(node),
                None => {
                    // CloseBrace or Eof — stop processing entries.
                    break;
                }
            }
        }

        let close_brace = if let Some(token) = self.peek() {
            if matches!(token.kind, TokenKind::CloseBrace) {
                CloseBrace::Present(self.advance().expect("CloseBrace token"))
            } else {
                // Non-CloseBrace token (probably EOF) — missing close brace.
                let diag = CstDiagnostic::error(
                    format!(
                        "Expected '}}' to close block opened at line:{}",
                        open_range.start_line,
                    ),
                    open_range.clone(),
                );
                self.diagnostics.push(diag.clone());
                CloseBrace::Missing(diag)
            }
        } else {
            let diag = CstDiagnostic::error(
                format!(
                    "Expected '}}' to close block opened at line:{}",
                    open_range.start_line,
                ),
                open_range.clone(),
            );
            self.diagnostics.push(diag.clone());
            CloseBrace::Missing(diag)
        };

        CstBlock {
            open_brace: open,
            entries,
            close_brace,
        }
    }

    // ── Token helpers ─────────────────────────────────────────────────

    /// Peek at the current token without consuming it.
    fn peek(&self) -> Option<&CstToken> {
        self.tokens.get(self.pos)
    }

    /// Peek at the kind of the current token.
    fn peek_kind(&self) -> Option<&TokenKind> {
        self.peek().map(|t| &t.kind)
    }

    /// Consume and return the current token, advancing the position.
    fn advance(&mut self) -> Option<CstToken> {
        if self.pos < self.tokens.len() {
            let token = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(token)
        } else {
            None
        }
    }

    /// Try to consume a token of the given kind; emit an error if it doesn't match.
    /// Always advances past the token if present.
    #[allow(dead_code)]
    fn expect(&mut self, kind: &TokenKind, msg: &str) -> Option<CstToken> {
        match self.peek_kind() {
            Some(k) if k == kind => self.advance(),
            Some(_) => {
                let token = self.peek().expect("current token");
                let range = token.range.clone();
                self.emit_error(msg.to_string(), range);
                None
            }
            None => {
                self.emit_error(
                    msg.to_string(),
                    ast::Range {
                        start_line: 0,
                        start_col: 0,
                        end_line: 0,
                        end_col: 0,
                    },
                );
                None
            }
        }
    }

    /// Returns `true` if we're at (or past) the EOF token.
    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len().saturating_sub(1)
    }

    /// Returns `true` if the token is EOF.
    fn is_eof(&self, token: Option<&CstToken>) -> bool {
        matches!(token, Some(t) if matches!(t.kind, TokenKind::Eof))
    }

    /// Returns `true` if the token is a close brace.
    fn is_close_brace(&self, token: Option<&CstToken>) -> bool {
        matches!(token, Some(t) if matches!(t.kind, TokenKind::CloseBrace))
    }

    // ── Error recovery ────────────────────────────────────────────────

    /// Skip tokens until we find one that can start a new entry.
    ///
    /// Tokens that can start an entry: Ident, OpenBrace, String, Number.
    /// CloseBrace is NOT skipped (it propagates up to the enclosing block).
    /// Eof is also NOT consumed.
    fn skip_to_entry_boundary(&mut self) {
        while let Some(token) = self.peek() {
            if self.is_at_end() {
                break;
            }
            match &token.kind {
                TokenKind::Ident(_)
                | TokenKind::OpenBrace
                | TokenKind::String(_)
                | TokenKind::Number(_) => break,
                TokenKind::CloseBrace => break, // let it propagate
                TokenKind::Eof => break,
                _ => {
                    let range = token.range.clone();
                    self.emit_error(format!("Skipping unexpected token: {}", token.kind), range);
                    self.advance();
                }
            }
        }
    }

    /// Record a diagnostic (error).
    fn emit_error(&mut self, msg: impl Into<String>, range: ast::Range) {
        self.diagnostics.push(CstDiagnostic::error(msg, range));
    }

    /// Drain the first `Comment` trivia from the current token's leading_trivia
    /// and return it as an `EntryComment` node.
    ///
    /// All comments — whether standalone (on their own line) or inline (after
    /// a value on the same line) — are extracted as entries. This matches the
    /// old parser's behavior which treats every `#` line as a comment entry.
    fn drain_comment(&mut self) -> Option<CstNode> {
        if self.pos >= self.tokens.len() {
            return None;
        }
        let token = &mut self.tokens[self.pos];
        let idx = token
            .leading_trivia
            .iter()
            .position(|t| t.kind == TriviaKind::Comment)?;
        let comment = token.leading_trivia.remove(idx);
        Some(CstNode::EntryComment(comment))
    }
}

/// Classify an identifier token into the appropriate `CstValue` variant.
///
/// - `"yes"` / `"no"` → Boolean
/// - Valid finite float → Number
/// - Everything else → Ident
fn classify_ident(token: CstToken) -> CstValue {
    let text = &token.text;
    if text == "yes" {
        return CstValue::Boolean(Box::new(token));
    }
    if text == "no" {
        return CstValue::Boolean(Box::new(token));
    }
    if let Ok(n) = text.parse::<f64>() {
        if n.is_finite() {
            return CstValue::Number(Box::new(token));
        }
    }
    CstValue::Ident(Box::new(token))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::cst::lexer;

    fn parse(input: &str) -> CstScript {
        let (tokens, _) = lexer::tokenize(input);
        parse_cst(tokens)
    }

    #[test]
    fn empty_input() {
        let script = parse("");
        assert_eq!(script.nodes.len(), 0, "empty input should have 0 nodes");
        assert_eq!(
            script.diagnostics.len(),
            0,
            "empty input should have 0 diagnostics"
        );
    }

    #[test]
    fn simple_assignment() {
        let script = parse("key = val");
        assert_eq!(script.nodes.len(), 1);
        match &script.nodes[0] {
            CstNode::Assignment(assign) => {
                assert_eq!(assign.key.text, "key");
                assert!(matches!(assign.operator.kind, TokenKind::OpEquals));
                match &assign.value {
                    CstValue::Ident(v) => assert_eq!(v.text, "val"),
                    _ => panic!("Expected Ident value"),
                }
            }
            other => panic!("Expected Assignment, got {:?}", other),
        }
        assert_eq!(script.diagnostics.len(), 0, "no diagnostics expected");
    }

    #[test]
    fn block_assignment() {
        let script = parse("country_event = { id = test.1 }");
        assert_eq!(script.nodes.len(), 1);
        match &script.nodes[0] {
            CstNode::Assignment(assign) => {
                assert_eq!(assign.key.text, "country_event");
                match &assign.value {
                    CstValue::Block(block) => {
                        assert_eq!(block.entries.len(), 1);
                        match &block.entries[0] {
                            CstNode::Assignment(inner) => {
                                assert_eq!(inner.key.text, "id");
                                match &inner.value {
                                    CstValue::Ident(v) => {
                                        assert_eq!(v.text, "test.1");
                                    }
                                    _ => panic!("Expected Ident value"),
                                }
                            }
                            other => panic!("Expected inner Assignment, got {:?}", other),
                        }
                    }
                    other => panic!("Expected Block value, got {:?}", other),
                }
            }
            other => panic!("Expected Assignment, got {:?}", other),
        }
        assert_eq!(script.diagnostics.len(), 0, "no diagnostics expected");
    }

    #[test]
    fn tagged_block() {
        let script = parse("modifier = my_tag { factor = 0.5 }");
        assert_eq!(script.nodes.len(), 1);
        match &script.nodes[0] {
            CstNode::Assignment(assign) => {
                assert_eq!(assign.key.text, "modifier");
                match &assign.value {
                    CstValue::TaggedBlock { tag, block } => {
                        assert_eq!(tag.text, "my_tag");
                        assert_eq!(block.entries.len(), 1);
                        match &block.entries[0] {
                            CstNode::Assignment(inner) => {
                                assert_eq!(inner.key.text, "factor");
                                match &inner.value {
                                    CstValue::Number(n) => assert_eq!(n.text, "0.5"),
                                    _ => panic!("Expected Number value"),
                                }
                            }
                            other => panic!("Expected inner Assignment, got {:?}", other),
                        }
                    }
                    other => panic!("Expected TaggedBlock, got {:?}", other),
                }
            }
            other => panic!("Expected Assignment, got {:?}", other),
        }
        assert_eq!(script.diagnostics.len(), 0, "no diagnostics expected");
    }

    #[test]
    fn bare_value() {
        let script = parse("{ key = val }");
        assert_eq!(script.nodes.len(), 1);
        match &script.nodes[0] {
            CstNode::EntryValue(entry) => match &*entry.value {
                CstValue::Block(block) => {
                    assert_eq!(block.entries.len(), 1);
                    match &block.entries[0] {
                        CstNode::Assignment(inner) => {
                            assert_eq!(inner.key.text, "key");
                        }
                        other => panic!("Expected Assignment, got {:?}", other),
                    }
                }
                other => panic!("Expected Block, got {:?}", other),
            },
            other => panic!("Expected EntryValue, got {:?}", other),
        }
        assert_eq!(script.diagnostics.len(), 0, "no diagnostics expected");
    }

    #[test]
    fn bare_ident() {
        let script = parse("yes");
        assert_eq!(script.nodes.len(), 1);
        match &script.nodes[0] {
            CstNode::EntryValue(entry) => match &*entry.value {
                CstValue::Boolean(b) => assert_eq!(b.text, "yes"),
                other => panic!("Expected Boolean, got {:?}", other),
            },
            other => panic!("Expected EntryValue, got {:?}", other),
        }
        assert_eq!(script.diagnostics.len(), 0, "no diagnostics expected");
    }

    #[test]
    fn multiple_entries() {
        let script = parse("a = 1\nb = 2");
        assert_eq!(script.nodes.len(), 2);
        match &script.nodes[0] {
            CstNode::Assignment(assign) => assert_eq!(assign.key.text, "a"),
            other => panic!("Expected Assignment, got {:?}", other),
        }
        match &script.nodes[1] {
            CstNode::Assignment(assign) => assert_eq!(assign.key.text, "b"),
            other => panic!("Expected Assignment, got {:?}", other),
        }
        assert_eq!(script.diagnostics.len(), 0, "no diagnostics expected");
    }

    #[test]
    fn number_fallback() {
        let script = parse("42");
        assert_eq!(script.nodes.len(), 1);
        match &script.nodes[0] {
            CstNode::EntryValue(entry) => match &*entry.value {
                CstValue::Number(n) => {
                    assert_eq!(n.text, "42");
                }
                other => panic!("Expected Number, got {:?}", other),
            },
            other => panic!("Expected EntryValue, got {:?}", other),
        }
        assert_eq!(script.diagnostics.len(), 0, "no diagnostics expected");
    }

    #[test]
    fn boolean_ident_assignment() {
        let script = parse("key = yes");
        assert_eq!(script.nodes.len(), 1);
        match &script.nodes[0] {
            CstNode::Assignment(assign) => {
                assert_eq!(assign.key.text, "key");
                match &assign.value {
                    CstValue::Boolean(b) => assert_eq!(b.text, "yes"),
                    other => panic!("Expected Boolean, got {:?}", other),
                }
            }
            other => panic!("Expected Assignment, got {:?}", other),
        }
        assert_eq!(script.diagnostics.len(), 0, "no diagnostics expected");
    }

    #[test]
    fn string_value() {
        let script = parse("key = \"hello\"");
        assert_eq!(script.nodes.len(), 1);
        match &script.nodes[0] {
            CstNode::Assignment(assign) => {
                assert_eq!(assign.key.text, "key");
                match &assign.value {
                    CstValue::String(s) => assert_eq!(s.text, "\"hello\""),
                    other => panic!("Expected String, got {:?}", other),
                }
            }
            other => panic!("Expected Assignment, got {:?}", other),
        }
        // The tokenizer preserves the quotes in the text; the string value
        // inside the quotes is the token kind's inner String.
        assert_eq!(script.diagnostics.len(), 0, "no diagnostics expected");
    }

    #[test]
    fn missing_close_brace() {
        let script = parse("x = { y = 1");
        assert_eq!(script.nodes.len(), 1);
        match &script.nodes[0] {
            CstNode::Assignment(assign) => {
                assert_eq!(assign.key.text, "x");
                match &assign.value {
                    CstValue::Block(block) => {
                        assert_eq!(block.entries.len(), 1);
                        match &block.close_brace {
                            CloseBrace::Missing(diag) => {
                                assert!(
                                    diag.message.contains("Expected '}'"),
                                    "Diagnostic message: {}",
                                    diag.message
                                );
                            }
                            other => panic!("Expected Missing close brace, got {:?}", other),
                        }
                    }
                    other => panic!("Expected Block, got {:?}", other),
                }
            }
            other => panic!("Expected Assignment, got {:?}", other),
        }
        // Should have at least one diagnostic about the missing brace
        assert!(
            script.diagnostics.len() >= 1,
            "expected diagnostic about missing close brace"
        );
    }

    #[test]
    fn extra_close_brace() {
        let script = parse("x = { } }");
        assert_eq!(script.nodes.len(), 1);
        match &script.nodes[0] {
            CstNode::Assignment(assign) => {
                assert_eq!(assign.key.text, "x");
                match &assign.value {
                    CstValue::Block(block) => {
                        assert!(matches!(block.close_brace, CloseBrace::Present(_)));
                    }
                    other => panic!("Expected Block, got {:?}", other),
                }
            }
            other => panic!("Expected Assignment, got {:?}", other),
        }
        // The extra `}` should produce a diagnostic
        assert!(
            script.diagnostics.len() >= 1,
            "expected diagnostic about extra closing brace"
        );
    }

    #[test]
    fn dots_in_key() {
        let script = parse("title = daw.2.t");
        assert_eq!(script.nodes.len(), 1);
        match &script.nodes[0] {
            CstNode::Assignment(assign) => {
                assert_eq!(assign.key.text, "title");
                match &assign.value {
                    CstValue::Ident(v) => assert_eq!(v.text, "daw.2.t"),
                    other => panic!("Expected Ident, got {:?}", other),
                }
            }
            other => panic!("Expected Assignment, got {:?}", other),
        }
        assert_eq!(script.diagnostics.len(), 0);
    }

    #[test]
    fn pipe_in_value() {
        let script = parse("custom_effect_tooltip = tech_effect|sp_main");
        assert_eq!(script.nodes.len(), 1);
        match &script.nodes[0] {
            CstNode::Assignment(assign) => {
                assert_eq!(assign.key.text, "custom_effect_tooltip");
                match &assign.value {
                    CstValue::Ident(v) => assert_eq!(v.text, "tech_effect|sp_main"),
                    other => panic!("Expected Ident, got {:?}", other),
                }
            }
            other => panic!("Expected Assignment, got {:?}", other),
        }
        assert_eq!(script.diagnostics.len(), 0);
    }

    #[test]
    fn special_idents() {
        let script = parse("[?my_var] = 10");
        assert_eq!(script.nodes.len(), 1);
        match &script.nodes[0] {
            CstNode::Assignment(assign) => {
                assert_eq!(assign.key.text, "[?my_var]");
                match &assign.value {
                    CstValue::Number(n) => assert_eq!(n.text, "10"),
                    other => panic!("Expected Number, got {:?}", other),
                }
            }
            other => panic!("Expected Assignment, got {:?}", other),
        }
        assert_eq!(script.diagnostics.len(), 0);
    }

    #[test]
    fn nested_blocks() {
        let script = parse("a = { b = { c = 1 } }");
        assert_eq!(script.nodes.len(), 1);
        match &script.nodes[0] {
            CstNode::Assignment(outer) => {
                assert_eq!(outer.key.text, "a");
                match &outer.value {
                    CstValue::Block(outer_block) => {
                        assert_eq!(outer_block.entries.len(), 1);
                        match &outer_block.entries[0] {
                            CstNode::Assignment(mid) => {
                                assert_eq!(mid.key.text, "b");
                                match &mid.value {
                                    CstValue::Block(inner_block) => {
                                        assert_eq!(inner_block.entries.len(), 1);
                                        match &inner_block.entries[0] {
                                            CstNode::Assignment(inner) => {
                                                assert_eq!(inner.key.text, "c");
                                                match &inner.value {
                                                    CstValue::Number(n) => {
                                                        assert_eq!(n.text, "1");
                                                    }
                                                    other => {
                                                        panic!("Expected Number, got {:?}", other)
                                                    }
                                                }
                                            }
                                            other => panic!("Expected Assignment, got {:?}", other),
                                        }
                                    }
                                    other => panic!("Expected Block, got {:?}", other),
                                }
                            }
                            other => panic!("Expected Assignment, got {:?}", other),
                        }
                    }
                    other => panic!("Expected Block, got {:?}", other),
                }
            }
            other => panic!("Expected Assignment, got {:?}", other),
        }
        assert_eq!(script.diagnostics.len(), 0);
    }

    #[test]
    fn string_value_text() {
        // Verify that the token text for strings includes quotes
        let script = parse("key = \"hello world\"");
        match &script.nodes[0] {
            CstNode::Assignment(assign) => match &assign.value {
                CstValue::String(s) => {
                    // The tokenizer preserves quotes
                    assert_eq!(s.text, "\"hello world\"");
                }
                other => panic!("Expected String, got {:?}", other),
            },
            other => panic!("Expected Assignment, got {:?}", other),
        }
    }

    #[test]
    fn negative_number() {
        let script = parse("x = -42");
        assert_eq!(script.nodes.len(), 1);
        match &script.nodes[0] {
            CstNode::Assignment(assign) => {
                assert_eq!(assign.key.text, "x");
                match &assign.value {
                    CstValue::Number(n) => assert_eq!(n.text, "-42"),
                    other => panic!("Expected Number, got {:?}", other),
                }
            }
            other => panic!("Expected Assignment, got {:?}", other),
        }
    }

    #[test]
    fn all_operators() {
        let inputs = [
            ("a = 1", TokenKind::OpEquals, "1"),
            ("a < 1", TokenKind::OpLessThan, "1"),
            ("a > 1", TokenKind::OpGreaterThan, "1"),
            ("a != 1", TokenKind::OpNotEquals, "1"),
            ("a <= 1", TokenKind::OpLessOrEqual, "1"),
            ("a >= 1", TokenKind::OpGreaterOrEqual, "1"),
        ];
        for (input, expected_op, expected_val) in &inputs {
            let script = parse(input);
            assert_eq!(script.nodes.len(), 1, "input: {}", input);
            match &script.nodes[0] {
                CstNode::Assignment(assign) => {
                    assert_eq!(assign.key.text, "a", "input: {}", input);
                    assert!(
                        std::mem::discriminant(&assign.operator.kind)
                            == std::mem::discriminant(expected_op),
                        "input: {}, expected operator {:?}",
                        input,
                        expected_op
                    );
                    match &assign.value {
                        CstValue::Number(n) => assert_eq!(n.text, *expected_val),
                        other => panic!("Expected Number, got {:?}", other),
                    }
                }
                other => panic!("Expected Assignment, got {:?}", other),
            }
        }
    }

    #[test]
    fn tagged_block_bare() {
        // A bare tagged block at the top level
        let script = parse("my_tag { }");
        assert_eq!(script.nodes.len(), 1);
        match &script.nodes[0] {
            CstNode::EntryValue(entry) => match &*entry.value {
                CstValue::TaggedBlock { tag, block } => {
                    assert_eq!(tag.text, "my_tag");
                    assert_eq!(block.entries.len(), 0);
                }
                other => panic!("Expected TaggedBlock, got {:?}", other),
            },
            other => panic!("Expected EntryValue, got {:?}", other),
        }
    }

    #[test]
    fn bare_no_ident() {
        let script = parse("no");
        match &script.nodes[0] {
            CstNode::EntryValue(entry) => match &*entry.value {
                CstValue::Boolean(b) => assert_eq!(b.text, "no"),
                other => panic!("Expected Boolean, got {:?}", other),
            },
            other => panic!("Expected EntryValue, got {:?}", other),
        }
        assert_eq!(script.diagnostics.len(), 0);
    }

    #[test]
    fn boolean_number_string_literals() {
        let script = parse("yes = yes\nno = no\ntrue_val = true");
        assert_eq!(script.nodes.len(), 3);

        // yes = yes → key=Boolean(true), value=Boolean(true)
        match &script.nodes[0] {
            CstNode::Assignment(assign) => {
                assert_eq!(assign.key.text, "yes");
                assert!(matches!(assign.value, CstValue::Boolean(_)));
            }
            other => panic!("Expected Assignment, got {:?}", other),
        }
        // no = no → key=Boolean(false)? Actually "no" is parsed as a bare ident
        // and classified as Boolean. But the key is just an Ident token? No,
        // the key token is preserved as-is — it's a CstToken with text "no".
        // The operator and value are separate.
        match &script.nodes[0] {
            CstNode::Assignment(assign) => {
                // The `yes` key token is just an Ident token — we don't classify keys
                assert_eq!(assign.key.text, "yes");
            }
            _ => {}
        }
    }

    #[test]
    fn empty_block() {
        let script = parse("x = { }");
        assert_eq!(script.nodes.len(), 1);
        match &script.nodes[0] {
            CstNode::Assignment(assign) => match &assign.value {
                CstValue::Block(block) => {
                    assert_eq!(block.entries.len(), 0);
                    assert!(matches!(block.close_brace, CloseBrace::Present(_)));
                }
                other => panic!("Expected Block, got {:?}", other),
            },
            other => panic!("Expected Assignment, got {:?}", other),
        }
        assert_eq!(script.diagnostics.len(), 0);
    }

    #[test]
    fn eof_after_open_brace_recovery() {
        let script = parse("x = {");
        // Should recover with a missing close brace diagnostic
        assert_eq!(script.nodes.len(), 1);
        match &script.nodes[0] {
            CstNode::Assignment(assign) => match &assign.value {
                CstValue::Block(block) => {
                    assert!(matches!(block.close_brace, CloseBrace::Missing(_)));
                }
                other => panic!("Expected Block, got {:?}", other),
            },
            other => panic!("Expected Assignment, got {:?}", other),
        }
        assert!(
            script.diagnostics.len() >= 1,
            "expected diagnostic for missing close brace"
        );
    }

    #[test]
    fn multiple_blocks() {
        let script = parse("a = { 1 }\nb = { 2 }");
        assert_eq!(script.nodes.len(), 2);
        for node in &script.nodes {
            match node {
                CstNode::Assignment(assign) => match &assign.value {
                    CstValue::Block(b) => {
                        assert_eq!(b.entries.len(), 1);
                        match &b.entries[0] {
                            CstNode::EntryValue(entry) => match &*entry.value {
                                CstValue::Number(n) => {
                                    assert!(
                                        n.text == "1" || n.text == "2",
                                        "unexpected number: {}",
                                        n.text
                                    );
                                }
                                other => panic!("Expected Number, got {:?}", other),
                            },
                            other => panic!("Expected EntryValue, got {:?}", other),
                        }
                    }
                    other => panic!("Expected Block, got {:?}", other),
                },
                other => panic!("Expected Assignment, got {:?}", other),
            }
        }
    }

    #[test]
    fn numeric_key_assignment() {
        let script = parse("588 = { naval_base = 3 }");
        assert_eq!(script.nodes.len(), 1, "should have 1 node");
        assert_eq!(script.diagnostics.len(), 0, "no diagnostics expected");
        match &script.nodes[0] {
            CstNode::Assignment(assign) => {
                assert_eq!(assign.key.text, "588", "numeric key text should be '588'");
                assert!(matches!(assign.value, CstValue::Block(_)), "value should be a block");
            }
            other => panic!("Expected Assignment with numeric key, got {:?}", other),
        }
    }

    #[test]
    fn state_history_numeric_keys() {
        let input = r#"state = {
	id = 11
	name = "STATE_11"
	manpower = 3031124
	state_category = large_city
	provinces = {
		588 1646 1978
	}
	resources = {
		wood = 2
		coal = 1
	}
	history = {
		owner = SPE
		add_core_of = SPE
		victory_points = { 5259 20 }
		victory_points = { 1978 10 }
		buildings = {
			infrastructure = 3
			arms_factory = 1
			588 = { naval_base = 3 }
		}
	}
}
"#;
        let script = parse(input);
        assert_eq!(
            script.diagnostics.len(), 0,
            "state history should parse with 0 diagnostics, got {}",
            script.diagnostics.iter().map(|d| format!("{}", d.message)).collect::<Vec<_>>().join(", ")
        );
        assert_eq!(script.nodes.len(), 1, "should have 1 top-level node");
    }
}
