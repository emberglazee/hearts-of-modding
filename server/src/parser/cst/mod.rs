//! CST (Concrete Syntax Tree) — lossless representation of HOI4 script.
//!
//! The CST preserves all source information including whitespace, comments, and
//! formatting. It is produced by a two-phase pipeline:
//!   1. Tokenizer (`lexer::tokenize`) — chars → tokens with trivia
//!   2. Parser (`parser::parse_cst`) — tokens → CST tree
//!
//! The CST can be:
//!   - Lowered to the existing AST for semantic consumers (`lower::lower`)
//!   - Transformed + serialized for formatting (`formatter` module, future)

pub mod diagnostic;
pub mod lexer;
pub mod lower;
pub mod parser;
pub mod token;
pub mod types;

/// Parse source text into CST, then lower to existing AST.
/// Returns (ast::Script, parse_error_tuples) matching the old parser signature.
pub fn parse_and_lower(input: &str) -> (crate::parser::ast::Script, Vec<(String, crate::parser::ast::Range)>) {
    let (tokens, _) = lexer::tokenize(input);
    let cst = parser::parse_cst(tokens);
    lower::lower(cst)
}

/// Parse source text into CST only (no lowering).
pub fn parse_cst(input: &str) -> crate::parser::cst::types::CstScript {
    let (tokens, _) = lexer::tokenize(input);
    parser::parse_cst(tokens)
}

#[cfg(test)]
pub mod tests;
