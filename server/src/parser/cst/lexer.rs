//! CST Tokenizer — linear char-by-char scanner producing tokens with trivia.

use crate::parser::ast;
use crate::parser::cst::diagnostic::*;
use crate::parser::cst::token::*;

/// Returns `true` if `c` is a valid HOI4 identifier character.
fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric()
        || c == '_'
        || c == '.'
        || c == ':'
        || c == '@'
        || c == '['
        || c == ']'
        || c == '?'
        || c == '^'
        || c == '$'
        || c == '/'
        || c == '-'
        || c == '\''
        || c == '%'
        || c == '|'
        || c == '*'
}

/// Try to scan a numeric literal starting at the current byte position.
///
/// Returns `Some((text, value, end_byte, end_line, end_col))` on success.
/// On failure the cursors (`byte_pos`, `line`, `col`) are restored and
/// `None` is returned so the caller can fall back to ident scanning.
fn try_scan_number(
    input: &str,
    byte_pos: &mut usize,
    line: &mut u32,
    col: &mut u32,
) -> Option<(String, f64, usize, u32, u32)> {
    let saved_pos = *byte_pos;
    let saved_line = *line;
    let saved_col = *col;

    let rest = &input[saved_pos..];
    let c = rest.chars().next()?;

    // Check whether we could be starting a number.
    if !c.is_ascii_digit() && c != '.' && c != '-' {
        return None;
    }

    // For '.' and '-' we must peek at the following character — it must be a digit.
    if c == '.' {
        if rest.len() <= 1 {
            return None;
        }
        let next = input[saved_pos + 1..].chars().next()?;
        if !next.is_ascii_digit() {
            return None;
        }
    }
    if c == '-' {
        if rest.len() <= 1 {
            return None;
        }
        let next = input[saved_pos + 1..].chars().next()?;
        if !next.is_ascii_digit() {
            return None;
        }
    }

    // --- Commit to scanning the number ---
    let mut num_text = String::new();
    let mut has_dot = false;

    // Handle a leading '-' or '.' prefix.
    if c == '-' || c == '.' {
        num_text.push(c);
        *byte_pos += c.len_utf8();
        *col += 1;
        if c == '.' {
            has_dot = true;
        }
    }

    // Consume digits and at most one fractional dot.
    while *byte_pos < input.len() {
        let b = input.as_bytes()[*byte_pos];
        if b.is_ascii_digit() {
            num_text.push(b as char);
            *byte_pos += 1;
            *col += 1;
        } else if b == b'.' && !has_dot {
            num_text.push('.');
            *byte_pos += 1;
            *col += 1;
            has_dot = true;
        } else {
            break;
        }
    }

    // Boundary check: the next character MUST NOT be alphanumeric.
    // If it is, this is really an identifier (e.g. "42abc").
    if *byte_pos < input.len() {
        let next = input[*byte_pos..].chars().next().unwrap();
        if next.is_alphanumeric() {
            *byte_pos = saved_pos;
            *line = saved_line;
            *col = saved_col;
            return None;
        }
    }

    // Try to parse as f64.
    let val: f64 = match num_text.parse() {
        Ok(v) => v,
        Err(_) => {
            // Parse failure — fall back to ident (e.g. "-" or "." alone).
            *byte_pos = saved_pos;
            *line = saved_line;
            *col = saved_col;
            return None;
        }
    };

    let end_byte = *byte_pos;
    let end_line = *line;
    let end_col = *col;

    Some((num_text, val, end_byte, end_line, end_col))
}

/// Tokenize HOI4 script source text into a flat list of CST tokens with
/// leading trivia.  Returns `(tokens, diagnostics)`.
pub fn tokenize(input: &str) -> (Vec<CstToken>, Vec<CstDiagnostic>) {
    let mut tokens: Vec<CstToken> = Vec::new();
    let mut diagnostics: Vec<CstDiagnostic> = Vec::new();

    // Scanner state.
    let mut byte_pos: usize = 0;
    let mut line: u32 = 0;
    let mut col: u32 = 0;

    // --- BOM stripping ---
    if input.starts_with('\u{feff}') {
        byte_pos = '\u{feff}'.len_utf8();
    }

    // --- Main scanning loop ---
    loop {
        // ======== 1. Collect leading trivia ========
        let mut leading_trivia: Vec<Trivia> = Vec::new();

        'trivia: loop {
            if byte_pos >= input.len() {
                break 'trivia;
            }

            let b = input.as_bytes()[byte_pos];
            match b {
                // Space / tab — group consecutive runs into one Whitespace piece.
                b' ' | b'\t' => {
                    let tr_start_line = line;
                    let tr_start_col = col;
                    let mut text = String::new();
                    while byte_pos < input.len() {
                        let b2 = input.as_bytes()[byte_pos];
                        if b2 == b' ' || b2 == b'\t' {
                            text.push(b2 as char);
                            byte_pos += 1;
                            col += 1;
                        } else {
                            break;
                        }
                    }
                    leading_trivia.push(Trivia::new(
                        TriviaKind::Whitespace,
                        text,
                        ast::Range {
                            start_line: tr_start_line,
                            start_col: tr_start_col,
                            end_line: line,
                            end_col: col,
                        },
                    ));
                }

                // Newline (\n or \r\n).
                b'\n' => {
                    let tr_start_line = line;
                    let tr_start_col = col;
                    byte_pos += 1;
                    line += 1;
                    col = 0;
                    leading_trivia.push(Trivia::new(
                        TriviaKind::Newline,
                        "\n".to_string(),
                        ast::Range {
                            start_line: tr_start_line,
                            start_col: tr_start_col,
                            end_line: line,
                            end_col: col,
                        },
                    ));
                }

                // Carriage return — possibly part of \r\n.
                b'\r' => {
                    let tr_start_line = line;
                    let tr_start_col = col;
                    byte_pos += 1;
                    let mut text = String::from("\r");
                    if byte_pos < input.len() && input.as_bytes()[byte_pos] == b'\n' {
                        byte_pos += 1;
                        text.push('\n');
                    }
                    line += 1;
                    col = 0;
                    leading_trivia.push(Trivia::new(
                        TriviaKind::Newline,
                        text,
                        ast::Range {
                            start_line: tr_start_line,
                            start_col: tr_start_col,
                            end_line: line,
                            end_col: col,
                        },
                    ));
                }

                // Comment — from # to end of line (newline NOT consumed).
                b'#' => {
                    let tr_start_line = line;
                    let tr_start_col = col;
                    let mut text = String::new();
                    text.push('#');
                    byte_pos += 1;
                    col += 1;
                    // Consume everything up to (but not including) \n, \r, or EOF.
                    while byte_pos < input.len() {
                        let b2 = input.as_bytes()[byte_pos];
                        if b2 == b'\n' || b2 == b'\r' {
                            break;
                        }
                        text.push(b2 as char);
                        byte_pos += 1;
                        col += 1;
                    }
                    leading_trivia.push(Trivia::new(
                        TriviaKind::Comment,
                        text,
                        ast::Range {
                            start_line: tr_start_line,
                            start_col: tr_start_col,
                            end_line: line,
                            end_col: col,
                        },
                    ));
                }

                // Not trivia — stop collecting.
                _ => break 'trivia,
            }
        }

        // ======== 2. End of input — emit EOF ========
        if byte_pos >= input.len() {
            tokens.push(CstToken::new(
                TokenKind::Eof,
                String::new(),
                ast::Range {
                    start_line: line,
                    start_col: col,
                    end_line: line,
                    end_col: col,
                },
                TextRange::new(byte_pos, byte_pos),
                leading_trivia,
            ));
            break;
        }

        // ======== 3. Recognise the next token ========
        let tok_start_byte = byte_pos;
        let tok_start_line = line;
        let tok_start_col = col;

        // Peek at the current character (handles multi-byte UTF-8 safely).
        let rest = &input[byte_pos..];
        let c = rest.chars().next().unwrap();

        // ---- Try number before the main match (overlaps with ident chars '.' and '-') ----
        if c.is_ascii_digit()
            || (c == '.'
                && byte_pos + 1 < input.len()
                && input.as_bytes()[byte_pos + 1].is_ascii_digit())
            || (c == '-'
                && byte_pos + 1 < input.len()
                && input.as_bytes()[byte_pos + 1].is_ascii_digit())
        {
            if let Some((num_text, num_val, end_byte, end_line, end_col)) =
                try_scan_number(input, &mut byte_pos, &mut line, &mut col)
            {
                tokens.push(CstToken::new(
                    TokenKind::Number(num_val),
                    num_text,
                    ast::Range {
                        start_line: tok_start_line,
                        start_col: tok_start_col,
                        end_line,
                        end_col,
                    },
                    TextRange::new(tok_start_byte, end_byte),
                    leading_trivia,
                ));
                continue;
            }
            // Fallback: try_scan_number restored positions, so `c` is still valid.
        }

        // ---- Character-by-character dispatch ----
        match c {
            '{' => {
                byte_pos += 1;
                col += 1;
                tokens.push(CstToken::new(
                    TokenKind::OpenBrace,
                    "{".to_string(),
                    ast::Range {
                        start_line: tok_start_line,
                        start_col: tok_start_col,
                        end_line: line,
                        end_col: col,
                    },
                    TextRange::new(tok_start_byte, byte_pos),
                    leading_trivia,
                ));
            }
            '}' => {
                byte_pos += 1;
                col += 1;
                tokens.push(CstToken::new(
                    TokenKind::CloseBrace,
                    "}".to_string(),
                    ast::Range {
                        start_line: tok_start_line,
                        start_col: tok_start_col,
                        end_line: line,
                        end_col: col,
                    },
                    TextRange::new(tok_start_byte, byte_pos),
                    leading_trivia,
                ));
            }
            '<' => {
                byte_pos += 1;
                col += 1;
                if byte_pos < input.len() && input.as_bytes()[byte_pos] == b'=' {
                    byte_pos += 1;
                    col += 1;
                    tokens.push(CstToken::new(
                        TokenKind::OpLessOrEqual,
                        "<=".to_string(),
                        ast::Range {
                            start_line: tok_start_line,
                            start_col: tok_start_col,
                            end_line: line,
                            end_col: col,
                        },
                        TextRange::new(tok_start_byte, byte_pos),
                        leading_trivia,
                    ));
                } else {
                    tokens.push(CstToken::new(
                        TokenKind::OpLessThan,
                        "<".to_string(),
                        ast::Range {
                            start_line: tok_start_line,
                            start_col: tok_start_col,
                            end_line: line,
                            end_col: col,
                        },
                        TextRange::new(tok_start_byte, byte_pos),
                        leading_trivia,
                    ));
                }
            }
            '>' => {
                byte_pos += 1;
                col += 1;
                if byte_pos < input.len() && input.as_bytes()[byte_pos] == b'=' {
                    byte_pos += 1;
                    col += 1;
                    tokens.push(CstToken::new(
                        TokenKind::OpGreaterOrEqual,
                        ">=".to_string(),
                        ast::Range {
                            start_line: tok_start_line,
                            start_col: tok_start_col,
                            end_line: line,
                            end_col: col,
                        },
                        TextRange::new(tok_start_byte, byte_pos),
                        leading_trivia,
                    ));
                } else {
                    tokens.push(CstToken::new(
                        TokenKind::OpGreaterThan,
                        ">".to_string(),
                        ast::Range {
                            start_line: tok_start_line,
                            start_col: tok_start_col,
                            end_line: line,
                            end_col: col,
                        },
                        TextRange::new(tok_start_byte, byte_pos),
                        leading_trivia,
                    ));
                }
            }
            '!' => {
                byte_pos += 1;
                col += 1;
                if byte_pos < input.len() && input.as_bytes()[byte_pos] == b'=' {
                    byte_pos += 1;
                    col += 1;
                    tokens.push(CstToken::new(
                        TokenKind::OpNotEquals,
                        "!=".to_string(),
                        ast::Range {
                            start_line: tok_start_line,
                            start_col: tok_start_col,
                            end_line: line,
                            end_col: col,
                        },
                        TextRange::new(tok_start_byte, byte_pos),
                        leading_trivia,
                    ));
                } else {
                    // Standalone '!' is an error.
                    diagnostics.push(CstDiagnostic::error(
                        format!("Unexpected character: '!'"),
                        ast::Range {
                            start_line: tok_start_line,
                            start_col: tok_start_col,
                            end_line: line,
                            end_col: col,
                        },
                    ));
                    // No token is emitted — leading_trivia is discarded.
                }
            }
            '=' => {
                byte_pos += 1;
                col += 1;
                tokens.push(CstToken::new(
                    TokenKind::OpEquals,
                    "=".to_string(),
                    ast::Range {
                        start_line: tok_start_line,
                        start_col: tok_start_col,
                        end_line: line,
                        end_col: col,
                    },
                    TextRange::new(tok_start_byte, byte_pos),
                    leading_trivia,
                ));
            }
            '"' => {
                // ---- Quoted string ----
                let str_start_byte = byte_pos;
                // Skip opening quote.
                byte_pos += 1;
                col += 1;
                let mut string_content = String::new();
                let mut unterminated = false;

                loop {
                    if byte_pos >= input.len() {
                        unterminated = true;
                        break;
                    }
                    let b = input.as_bytes()[byte_pos];
                    if b == b'"' {
                        // Consume closing quote.
                        byte_pos += 1;
                        col += 1;
                        break;
                    } else if b == b'\\' {
                        // Escape sequence: skip the backslash and take the
                        // following character literally.
                        byte_pos += 1;
                        col += 1;
                        if byte_pos < input.len() {
                            let esc = input[byte_pos..].chars().next().unwrap();
                            string_content.push(esc);
                            byte_pos += esc.len_utf8();
                            col += 1;
                        }
                    } else {
                        let ch = input[byte_pos..].chars().next().unwrap();
                        string_content.push(ch);
                        byte_pos += ch.len_utf8();
                        col += 1;
                    }
                }

                if unterminated {
                    diagnostics.push(CstDiagnostic::error(
                        "Unterminated string literal",
                        ast::Range {
                            start_line: tok_start_line,
                            start_col: tok_start_col,
                            end_line: line,
                            end_col: col,
                        },
                    ));
                }

                let text = input[str_start_byte..byte_pos].to_string();
                tokens.push(CstToken::new(
                    TokenKind::String(string_content),
                    text,
                    ast::Range {
                        start_line: tok_start_line,
                        start_col: tok_start_col,
                        end_line: line,
                        end_col: col,
                    },
                    TextRange::new(tok_start_byte, byte_pos),
                    leading_trivia,
                ));
            }
            _ if is_identifier_char(c) => {
                let mut ident_text = String::new();
                while byte_pos < input.len() {
                    let ch = input[byte_pos..].chars().next().unwrap();
                    if is_identifier_char(ch) {
                        ident_text.push(ch);
                        byte_pos += ch.len_utf8();
                        col += 1;
                    } else {
                        break;
                    }
                }
                tokens.push(CstToken::new(
                    TokenKind::Ident(ident_text.clone()),
                    ident_text,
                    ast::Range {
                        start_line: tok_start_line,
                        start_col: tok_start_col,
                        end_line: line,
                        end_col: col,
                    },
                    TextRange::new(tok_start_byte, byte_pos),
                    leading_trivia,
                ));
            }
            _ => {
                // Unexpected character — emit diagnostic and skip it.
                let char_len = c.len_utf8();
                byte_pos += char_len;
                col += 1;
                diagnostics.push(CstDiagnostic::error(
                    format!("Unexpected character: '{}'", c),
                    ast::Range {
                        start_line: tok_start_line,
                        start_col: tok_start_col,
                        end_line: line,
                        end_col: col,
                    },
                ));
                // No token is emitted — leading_trivia is discarded.
            }
        }
    }

    (tokens, diagnostics)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    // Helper: extract just the token kinds for compact assertions.
    fn kinds(tokens: &[CstToken]) -> Vec<&TokenKind> {
        tokens.iter().map(|t| &t.kind).collect()
    }

    // Helper: collect diagnostic messages.
    fn diag_msgs(diags: &[CstDiagnostic]) -> Vec<&str> {
        diags.iter().map(|d| d.message.as_str()).collect()
    }

    // Helper: extract trivia kinds for a given token.
    fn trivia_kinds(tok: &CstToken) -> Vec<&TriviaKind> {
        tok.leading_trivia.iter().map(|t| &t.kind).collect()
    }

    #[test]
    fn test_empty_input() {
        let (tokens, diags) = tokenize("");
        assert!(diags.is_empty());
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Eof);
    }

    #[test]
    fn test_whitespace_only() {
        let (tokens, diags) = tokenize("  \n  ");
        assert!(diags.is_empty());
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Eof);
        assert_eq!(trivia_kinds(&tokens[0]), &[&TriviaKind::Whitespace, &TriviaKind::Newline, &TriviaKind::Whitespace]);
    }

    #[test]
    fn test_basic_tokens() {
        let (tokens, diags) = tokenize("{}");
        assert!(diags.is_empty());
        assert_eq!(kinds(&tokens), &[&TokenKind::OpenBrace, &TokenKind::CloseBrace, &TokenKind::Eof]);
    }

    #[test]
    fn test_operators() {
        let (tokens, diags) = tokenize("= < > <= >= !=");
        assert!(diags.is_empty());
        assert_eq!(
            kinds(&tokens),
            &[
                &TokenKind::OpEquals,
                &TokenKind::OpLessThan,
                &TokenKind::OpGreaterThan,
                &TokenKind::OpLessOrEqual,
                &TokenKind::OpGreaterOrEqual,
                &TokenKind::OpNotEquals,
                &TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_identifiers() {
        let (tokens, diags) = tokenize("hello world.test[?var] array^0");
        assert!(diags.is_empty());
        assert_eq!(
            kinds(&tokens),
            &[
                &TokenKind::Ident("hello".into()),
                &TokenKind::Ident("world.test[?var]".into()),
                &TokenKind::Ident("array^0".into()),
                &TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_pipe_in_ident() {
        let (tokens, diags) = tokenize("tech_effect|sp_main");
        assert!(diags.is_empty());
        assert_eq!(
            kinds(&tokens),
            &[
                &TokenKind::Ident("tech_effect|sp_main".into()),
                &TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_quoted_strings() {
        let (tokens, diags) = tokenize("\"hello world\"");
        assert!(diags.is_empty());
        assert_eq!(
            kinds(&tokens),
            &[
                &TokenKind::String("hello world".into()),
                &TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_escaped_quotes() {
        let (tokens, diags) = tokenize("\"say \\\"hi\\\"\"");
        assert!(diags.is_empty());
        assert_eq!(tokens.len(), 2);
        match &tokens[0].kind {
            TokenKind::String(s) => {
                assert_eq!(s, "say \"hi\"");
            }
            other => panic!("expected String, got {:?}", other),
        }
    }

    #[test]
    fn test_unterminated_string() {
        let (tokens, diags) = tokenize("\"hello");
        assert_eq!(diags.len(), 1);
        assert_eq!(diag_msgs(&diags)[0], "Unterminated string literal");
        assert_eq!(tokens.len(), 2);
        match &tokens[0].kind {
            TokenKind::String(s) => {
                assert_eq!(s, "hello");
            }
            other => panic!("expected String, got {:?}", other),
        }
        assert_eq!(tokens[1].kind, TokenKind::Eof);
    }

    #[test]
    fn test_numbers() {
        let (tokens, diags) = tokenize("42 -0.15 .5");
        assert!(diags.is_empty());
        assert_eq!(
            kinds(&tokens),
            &[
                &TokenKind::Number(42.0),
                &TokenKind::Number(-0.15),
                &TokenKind::Number(0.5),
                &TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_number_boundary() {
        let (tokens, diags) = tokenize("42abc");
        assert!(diags.is_empty());
        assert_eq!(
            kinds(&tokens),
            &[
                &TokenKind::Ident("42abc".into()),
                &TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_number_with_percent() {
        let (tokens, diags) = tokenize("0.5%");
        assert!(diags.is_empty());
        assert_eq!(
            kinds(&tokens),
            &[
                &TokenKind::Number(0.5),
                &TokenKind::Ident("%".into()),
                &TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_comments() {
        let (tokens, diags) = tokenize("# comment\nkey = val");
        assert!(diags.is_empty());
        // First token (key) should have leading trivia: Comment("# comment"), Newline
        assert!(tokens.len() >= 4);
        // Check the first content token (key) has trivia
        let first_content = &tokens[0];
        assert_eq!(first_content.kind, TokenKind::Ident("key".into()));
        assert_eq!(
            trivia_kinds(first_content),
            &[&TriviaKind::Comment, &TriviaKind::Newline]
        );
        // Verify the comment text
        assert_eq!(first_content.leading_trivia[0].text, "# comment");

        // Rest of tokens
        assert_eq!(tokens[1].kind, TokenKind::OpEquals);
        assert_eq!(tokens[2].kind, TokenKind::Ident("val".into()));
        assert_eq!(tokens[3].kind, TokenKind::Eof);
    }

    #[test]
    fn test_bom_stripping() {
        let input = format!("\u{feff}key = val");
        let (tokens, diags) = tokenize(&input);
        assert!(diags.is_empty());
        assert_eq!(
            kinds(&tokens),
            &[
                &TokenKind::Ident("key".into()),
                &TokenKind::OpEquals,
                &TokenKind::Ident("val".into()),
                &TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_yes_no_as_ident() {
        let (tokens, diags) = tokenize("yes no");
        assert!(diags.is_empty());
        assert_eq!(
            kinds(&tokens),
            &[
                &TokenKind::Ident("yes".into()),
                &TokenKind::Ident("no".into()),
                &TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_utf8_multibyte() {
        let (tokens, diags) = tokenize("§test = 1");
        assert_eq!(diags.len(), 1);
        assert!(diag_msgs(&diags)[0].contains("Unexpected character"));
        // The § diagnostic means no token for §, then test, =, 1, EOF
        assert_eq!(
            kinds(&tokens),
            &[
                &TokenKind::Ident("test".into()),
                &TokenKind::OpEquals,
                &TokenKind::Number(1.0),
                &TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_text_field_preserves_source() {
        // Verify that the `text` field of tokens holds the original source.
        let (tokens, _) = tokenize("{ = \"hello\" 42 }");
        // OpenBrace: text should be "{"
        assert_eq!(tokens[0].text, "{");
        // OpEquals: text should be "="
        assert_eq!(tokens[1].text, "=");
        // String: text should include the quotes
        assert_eq!(tokens[2].text, "\"hello\"");
        // Number: text should be "42"
        assert_eq!(tokens[3].text, "42");
        // CloseBrace: text should be "}"
        assert_eq!(tokens[4].text, "}");
    }

    #[test]
    fn test_byte_range() {
        let (tokens, _) = tokenize("ab {");
        // "ab" has byte range 0..2
        assert_eq!(tokens[0].byte_range, TextRange::new(0, 2));
        // "{" has byte range 3..4 (after the space)
        // Actually there's a space trivia before {, so the { starts at byte 3
        assert_eq!(tokens[1].byte_range, TextRange::new(3, 4));
    }

    #[test]
    fn test_comment_only() {
        let (tokens, diags) = tokenize("# just a comment");
        assert!(diags.is_empty());
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Eof);
        assert_eq!(trivia_kinds(&tokens[0]), &[&TriviaKind::Comment]);
        assert_eq!(tokens[0].leading_trivia[0].text, "# just a comment");
    }

    #[test]
    fn test_crlf_newline() {
        let (tokens, diags) = tokenize("a\r\nb");
        assert!(diags.is_empty());
        assert_eq!(kinds(&tokens), &[
            &TokenKind::Ident("a".into()),
            &TokenKind::Ident("b".into()),
            &TokenKind::Eof,
        ]);
        // The trivia between a and b should be a Newline containing \r\n
        assert_eq!(tokens[1].leading_trivia.len(), 1);
        assert_eq!(tokens[1].leading_trivia[0].kind, TriviaKind::Newline);
        assert_eq!(tokens[1].leading_trivia[0].text, "\r\n");
    }
}
