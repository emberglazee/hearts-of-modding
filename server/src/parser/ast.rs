use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Byte offset range into a source document.
/// Use `.resolve(source)` to get the referenced substring.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ByteSpan {
    pub start: usize,
    pub end: usize,
}

impl ByteSpan {
    /// Resolve this span against the source document text.
    /// Panics if the offsets are out of bounds (should never happen if
    /// the span was produced by the parser against the same source).
    #[inline]
    pub fn resolve<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start..self.end]
    }

    /// Returns the byte length of this span.
    #[inline]
    pub fn len(&self) -> usize {
        self.end - self.start
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeedValue {
    pub value: Value,
    pub range: Range,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    /// An unquoted identifier / plain value — text can be resolved from
    /// the source document via `ByteSpan::resolve(source)`.
    String(ByteSpan),
    /// A quoted string literal with escape sequences already resolved.
    /// These are rare compared to identifiers (only event titles, descriptions, etc.).
    QuotedString(String),
    Number(f64),
    Boolean(bool),
    Block(Vec<Entry>),
    /// A tagged block like `name = { ... }` with tag name as `ByteSpan`.
    TaggedBlock(ByteSpan, Vec<Entry>, Range),
}

impl Value {
    /// Get the text content of a string-like value (String or QuotedString).
    /// Returns `None` for non-string variants.
    #[inline]
    pub fn as_str<'a>(&'a self, source: &'a str) -> Option<&'a str> {
        match self {
            Value::String(span) => Some(span.resolve(source)),
            Value::QuotedString(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operator {
    Equals,         // =
    LessThan,       // <
    GreaterThan,    // >
    NotEquals,      // !=
    LessOrEqual,    // <=
    GreaterOrEqual, // >=
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    /// Byte offsets for the key text in the source document.
    pub key: ByteSpan,
    /// Line/column range for the key (used for LSP positions).
    pub key_range: Range,
    pub operator: Operator,
    pub operator_range: Range,
    pub value: NodeedValue,
}

impl Assignment {
    /// Resolve the assignment key from the source document.
    /// Named `key_text` to avoid ambiguity with the `key` field.
    #[inline]
    pub fn key_text<'a>(&self, source: &'a str) -> &'a str {
        self.key.resolve(source)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Entry {
    Assignment(Assignment),
    Value(NodeedValue),
    /// A comment: `ByteSpan` covers the text after `#`, `Range` is
    /// the line/col position.
    Comment(ByteSpan, Range),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    /// The source document text that all ByteSpan offsets reference.
    pub source: Arc<str>,
    pub entries: Vec<Entry>,
    /// True when at least one block in this script was implicitly closed
    /// at EOF (the Clausewitz engine accepts this).
    pub closed_by_eof: bool,
}

impl Default for Script {
    fn default() -> Self {
        Self {
            source: Arc::from(""),
            entries: Vec::new(),
            closed_by_eof: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiagnosticTag {
    Unnecessary,
    Deprecated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticRelatedInformation {
    pub location: Location,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}
