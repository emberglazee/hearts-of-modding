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
