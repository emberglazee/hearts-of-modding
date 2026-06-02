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

    /// Empty range at a given position (for placeholders).
    pub fn empty(pos: usize) -> Self {
        Self { start: pos, end: pos }
    }
}

/// Severity of a parser diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// A diagnostic produced during tokenizing or parsing.
#[derive(Debug, Clone)]
pub struct CstDiagnostic {
    pub message: String,
    pub range: ast::Range,
    pub severity: Severity,
    pub fix: Option<String>,
}

impl CstDiagnostic {
    pub fn error(message: impl Into<String>, range: ast::Range) -> Self {
        Self {
            message: message.into(),
            range,
            severity: Severity::Error,
            fix: None,
        }
    }

    pub fn warning(message: impl Into<String>, range: ast::Range) -> Self {
        Self {
            message: message.into(),
            range,
            severity: Severity::Warning,
            fix: None,
        }
    }

    pub fn with_fix(mut self, fix: impl Into<String>) -> Self {
        self.fix = Some(fix.into());
        self
    }
}
