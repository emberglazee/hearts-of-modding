use crate::parser::cst::diagnostic::CstDiagnostic;
use crate::parser::cst::token::{CstToken, Trivia};

/// A complete HOI4 script CST.
#[derive(Debug, Clone)]
pub struct CstScript {
    /// Top-level nodes (entries at the script root).
    pub nodes: Vec<CstNode>,
    /// Diagnostics produced during parsing.
    pub diagnostics: Vec<CstDiagnostic>,
    /// Trailing trivia after the last token (EOF whitespace).
    pub trailing_trivia: Vec<Trivia>,
}

/// Represents the state of a closing brace — present or missing (error recovery).
#[derive(Debug, Clone)]
pub enum CloseBrace {
    Present(CstToken),
    Missing(CstDiagnostic),
}

impl CloseBrace {
    pub fn range(&self) -> Option<crate::parser::ast::Range> {
        match self {
            CloseBrace::Present(t) => Some(t.range.clone()),
            CloseBrace::Missing(d) => Some(d.range.clone()),
        }
    }
}

/// A top-level node in the CST — an assignment, bare value, comment, or error.
#[derive(Debug, Clone)]
pub enum CstNode {
    /// `key = value`
    Assignment(CstAssignment),
    /// A bare value with no key (e.g. a bare block or bare identifier).
    EntryValue(CstEntryValue),
    /// A standalone comment line.
    EntryComment(Trivia),
    /// A parse error recovered at the entry level.
    Error(CstDiagnostic),
}

/// `key = value`
#[derive(Debug, Clone)]
pub struct CstAssignment {
    pub key: CstToken,
    /// The operator token's leading_trivia includes whitespace between key and operator.
    pub operator: CstToken,
    pub value: CstValue,
}

/// A bare value entry (no key, no operator).
#[derive(Debug, Clone)]
pub struct CstEntryValue {
    pub value: Box<CstValue>,
}

impl CstEntryValue {
    pub fn new(value: CstValue) -> Self {
        Self {
            value: Box::new(value),
        }
    }
}

/// A value in the CST.
#[derive(Debug, Clone)]
pub enum CstValue {
    /// Unquoted identifier (may be a string-like value).
    Ident(Box<CstToken>),
    /// Quoted string "..." (with escapes resolved in the token text).
    String(Box<CstToken>),
    /// Numeric literal.
    Number(Box<CstToken>),
    /// Boolean literal (yes / no).
    Boolean(Box<CstToken>),
    /// Block: `{ ... }`
    Block(CstBlock),
    /// Tagged block: `identifier { ... }`
    TaggedBlock {
        tag: Box<CstToken>,
        block: Box<CstBlock>,
    },
    /// Placeholder for a missing/erroneous value.
    Error(CstDiagnostic),
}

/// A block: `{ entries }`  (may have missing close brace).
#[derive(Debug, Clone)]
pub struct CstBlock {
    pub open_brace: CstToken,
    pub entries: Vec<CstNode>,
    pub close_brace: CloseBrace,
}

impl CstBlock {
    pub fn new(open_brace: CstToken) -> Self {
        Self {
            open_brace,
            entries: Vec::new(),
            close_brace: CloseBrace::Missing(CstDiagnostic::error(
                "Unclosed block",
                crate::parser::ast::Range {
                    start_line: 0,
                    start_col: 0,
                    end_line: 0,
                    end_col: 0,
                },
            )),
        }
    }
}
