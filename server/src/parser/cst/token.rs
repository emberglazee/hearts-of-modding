use crate::parser::ast;
use crate::parser::cst::diagnostic::TextRange;

/// Classification of a single token in HOI4 script.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Structural
    OpenBrace,  // {
    CloseBrace, // }

    // Operators
    OpEquals,         // =
    OpLessThan,       // <
    OpGreaterThan,    // >
    OpNotEquals,      // !=
    OpLessOrEqual,    // <=
    OpGreaterOrEqual, // >=

    // Literals
    Ident(String),  // unquoted identifier
    String(String), // "quoted string"
    Number(f64),    // numeric literal
    Boolean(bool),  // yes / no

    // Special
    Eof,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::OpenBrace => write!(f, "'{{'"),
            TokenKind::CloseBrace => write!(f, "'}}'"),
            TokenKind::OpEquals => write!(f, "'='"),
            TokenKind::OpLessThan => write!(f, "'<'"),
            TokenKind::OpGreaterThan => write!(f, "'>'"),
            TokenKind::OpNotEquals => write!(f, "'!='"),
            TokenKind::OpLessOrEqual => write!(f, "'<='"),
            TokenKind::OpGreaterOrEqual => write!(f, "'>='"),
            TokenKind::Ident(s) => write!(f, "identifier '{}'", s),
            TokenKind::String(s) => write!(f, "string \"{}\"", s),
            TokenKind::Number(n) => write!(f, "number {}", n),
            TokenKind::Boolean(b) => write!(f, "boolean {}", b),
            TokenKind::Eof => write!(f, "end of file"),
        }
    }
}

/// Kind of trivia (non-semantic text between tokens).
#[derive(Debug, Clone, PartialEq)]
pub enum TriviaKind {
    /// Inline whitespace (spaces, tabs) — no newline.
    Whitespace,
    /// Newline character(s) — \n or \r\n.
    Newline,
    /// Comment line — from # to end of line.
    Comment,
}

/// A piece of trivia — whitespace, newline, or comment attached to a token.
#[derive(Debug, Clone)]
pub struct Trivia {
    pub kind: TriviaKind,
    pub text: String,
    pub range: ast::Range,
}

impl Trivia {
    pub fn new(kind: TriviaKind, text: String, range: ast::Range) -> Self {
        Self { kind, text, range }
    }
}

/// A complete token with its leading trivia and position information.
#[derive(Debug, Clone)]
pub struct CstToken {
    pub kind: TokenKind,
    /// The exact source text of this token.
    pub text: String,
    /// Line/column range for LSP compatibility.
    pub range: ast::Range,
    /// Byte-level range for efficient position tracking.
    pub byte_range: TextRange,
    /// All trivia that appears before this token (whitespace, newlines, comments).
    pub leading_trivia: Vec<Trivia>,
}

impl CstToken {
    pub fn new(
        kind: TokenKind,
        text: String,
        range: ast::Range,
        byte_range: TextRange,
        leading_trivia: Vec<Trivia>,
    ) -> Self {
        Self {
            kind,
            text,
            range,
            byte_range,
            leading_trivia,
        }
    }
}
