use crate::parser::ast;

/// Byte-level range in the source text.
/// Used internally by the tokenizer and parser for efficient position tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextRange {
    pub start: usize,
    pub end: usize,
}

impl TextRange {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// A diagnostic produced during tokenizing or parsing.
#[derive(Debug, Clone)]
pub struct CstDiagnostic {
    pub message: String,
    pub range: ast::Range,
}

impl CstDiagnostic {
    pub fn error(message: impl Into<String>, range: ast::Range) -> Self {
        Self {
            message: message.into(),
            range,
        }
    }
}
