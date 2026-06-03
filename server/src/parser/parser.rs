use std::sync::Arc;

use crate::parser::ast::{self, ByteSpan, Range, Value};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take, take_while, take_while1},
    character::complete::{anychar, char, multispace0},
    combinator::{map, opt, recognize},
    multi::many0,
    sequence::preceded,
};
use nom_locate::LocatedSpan;

type Span<'a> = LocatedSpan<&'a str>;

fn to_byte_span(span: Span) -> ByteSpan {
    ByteSpan {
        start: span.location_offset(),
        end: span.location_offset() + span.len(),
    }
}

fn to_range(span: Span) -> Range {
    let start_line = span.location_line() - 1;
    let start_col = span.get_column() as u32 - 1;
    let fragment = span.fragment();

    let mut end_line = start_line;
    let mut last_line_len = 0;
    for c in fragment.chars() {
        if c == '\n' {
            end_line += 1;
            last_line_len = 0;
        } else {
            last_line_len += 1;
        }
    }

    let end_col = if end_line == start_line {
        start_col + fragment.chars().count() as u32
    } else {
        last_line_len
    };

    Range {
        start_line,
        start_col,
        end_line,
        end_col,
    }
}

pub fn is_identifier_char(c: char) -> bool {
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

/// Match an identifier and return (ByteSpan, Range).
fn ident(input: Span) -> IResult<Span, (ByteSpan, Range)> {
    map(take_while1(is_identifier_char), |s: Span| {
        (to_byte_span(s), to_range(s))
    })
    .parse(input)
}

/// Parse a quoted string and return (resolved_content, ByteSpan_of_full_match, Range).
/// The ByteSpan covers the entire match including the surrounding quotes.
fn quoted_string(input: Span) -> IResult<Span, (String, ByteSpan, Range)> {
    let (input, start) = recognize(char('\"')).parse(input)?;
    let mut s = String::new();
    let mut current = input;
    loop {
        let (next, c) = anychar(current)?;
        if c == '\"' {
            let range = Range {
                start_line: start.location_line() - 1,
                start_col: start.get_column() as u32 - 1,
                end_line: next.location_line() - 1,
                end_col: next.get_column() as u32 - 1,
            };
            let full_span = ByteSpan {
                start: start.location_offset(),
                end: next.location_offset(),
            };
            return Ok((next, (s, full_span, range)));
        } else if c == '\\' {
            let (next2, escaped) = anychar(next)?;
            s.push(escaped);
            current = next2;
        } else {
            s.push(c);
            current = next;
        }
    }
}

fn number(input: Span) -> IResult<Span, (f64, Range)> {
    let (input, s) = recognize((
        opt(alt((char('-'), char('+')))),
        alt((
            recognize((
                take_while1(|c: char| c.is_ascii_digit()),
                opt((char('.'), take_while(|c: char| c.is_ascii_digit()))),
            )),
            recognize((char('.'), take_while1(|c: char| c.is_ascii_digit()))),
        )),
    ))
    .parse(input)?;

    let next_char = input.fragment().chars().next();
    if let Some(c) = next_char {
        if c.is_alphanumeric() && c != '%' {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
    }

    Ok((input, (s.fragment().parse().unwrap(), to_range(s))))
}

fn boolean(input: Span) -> IResult<Span, (bool, Range)> {
    let (input, s) = alt((tag("yes"), tag("no"))).parse(input)?;
    let next_char = input.fragment().chars().next();
    if let Some(c) = next_char {
        if c.is_alphanumeric() {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
    }

    Ok((input, (*s.fragment() == "yes", to_range(s))))
}

/// Parse a comment after `#`. Returns (ByteSpan_for_text_after_hash, Range).
fn comment(input: Span) -> IResult<Span, (ByteSpan, Range)> {
    let (input, s) =
        recognize(preceded(char('#'), take_while(|c| c != '\n' && c != '\r'))).parse(input)?;
    let span = to_byte_span(s);
    let range = to_range(s);
    // Skip the '#' prefix
    let span = ByteSpan {
        start: span.start + 1,
        end: span.end,
    };
    Ok((input, (span, range)))
}

fn operator(input: Span) -> IResult<Span, (ast::Operator, Range)> {
    alt((
        map(tag("<="), |s| (ast::Operator::LessOrEqual, to_range(s))),
        map(tag(">="), |s| (ast::Operator::GreaterOrEqual, to_range(s))),
        map(tag("!="), |s| (ast::Operator::NotEquals, to_range(s))),
        map(tag("="), |s| (ast::Operator::Equals, to_range(s))),
        map(tag("<"), |s| (ast::Operator::LessThan, to_range(s))),
        map(tag(">"), |s| (ast::Operator::GreaterThan, to_range(s))),
    ))
    .parse(input)
}

fn parse_tagged_block(input: Span) -> IResult<Span, (ByteSpan, Vec<ast::Entry>, Range, Range)> {
    let (input, (tag_span, tag_range)) = ident(input)?;
    let (input, (entries, block_range)) = preceded(multispace0, parse_block).parse(input)?;

    let range = Range {
        start_line: tag_range.start_line,
        start_col: tag_range.start_col,
        end_line: block_range.end_line,
        end_col: block_range.end_col,
    };

    Ok((input, (tag_span, entries, block_range, range)))
}

/// Parse an identifier value with numeric fallback.
/// This is a standalone function (not a combinator) because we need
/// access to the raw span fragment for number checking.
fn parse_identifier_value(input: Span) -> IResult<Span, ast::NodeedValue> {
    let (input, raw) = recognize(take_while1(is_identifier_char)).parse(input)?;
    let range = to_range(raw);
    let byte_span = to_byte_span(raw);
    let text = raw.fragment();

    // Check if it's a finite float (e.g. "15.0", "42")
    if let Ok(n) = text.parse::<f64>() {
        if n.is_finite() {
            return Ok((
                input,
                ast::NodeedValue {
                    value: Value::Number(n),
                    range,
                },
            ));
        }
    }

    Ok((
        input,
        ast::NodeedValue {
            value: Value::String(byte_span),
            range,
        },
    ))
}

fn parse_value(input: Span) -> IResult<Span, ast::NodeedValue> {
    alt((
        map(quoted_string, |(s, _, r)| ast::NodeedValue {
            value: Value::QuotedString(s),
            range: r,
        }),
        map(boolean, |(b, r)| ast::NodeedValue {
            value: Value::Boolean(b),
            range: r,
        }),
        map(parse_tagged_block, |(tag_span, entries, br, r)| {
            ast::NodeedValue {
                value: Value::TaggedBlock(tag_span, entries, br),
                range: r,
            }
        }),
        parse_identifier_value,
        map(number, |(n, r)| ast::NodeedValue {
            value: Value::Number(n),
            range: r,
        }),
        map(parse_block, |(v, r)| ast::NodeedValue {
            value: Value::Block(v),
            range: r,
        }),
    ))
    .parse(input)
}

fn parse_block(input: Span) -> IResult<Span, (Vec<ast::Entry>, Range)> {
    let (input, start) = preceded(multispace0, recognize(char('{'))).parse(input)?;
    let (input, entries) = many0(preceded(multispace0, parse_entry)).parse(input)?;
    let (input, end) = preceded(multispace0, recognize(char('}'))).parse(input)?;

    let range = Range {
        start_line: start.location_line() - 1,
        start_col: start.get_column() as u32 - 1,
        end_line: end.location_line() - 1,
        end_col: end.get_column() as u32,
    };
    Ok((input, (entries, range)))
}

fn parse_assignment(input: Span) -> IResult<Span, ast::Assignment> {
    // Keys are identifiers in practice; support quoted keys for robustness.
    let (input, (key_span, key_range)) =
        alt((ident, map(quoted_string, |(_, bs, r)| (bs, r)))).parse(input)?;
    let (input, (op, op_range)) = preceded(multispace0, operator).parse(input)?;
    let (input, val) = preceded(multispace0, parse_value).parse(input)?;

    Ok((
        input,
        ast::Assignment {
            key: key_span,
            key_range,
            operator: op,
            operator_range: op_range,
            value: val,
        },
    ))
}

fn parse_entry(input: Span) -> IResult<Span, ast::Entry> {
    alt((
        map(comment, |(bs, r)| ast::Entry::Comment(bs, r)),
        map(parse_assignment, ast::Entry::Assignment),
        map(parse_value, ast::Entry::Value),
    ))
    .parse(input)
}

fn to_error_range(span: Span) -> Range {
    let start_line = span.location_line() - 1;
    let start_col = span.get_column() as u32 - 1;
    let fragment = span.fragment();

    let mut end_col = start_col;
    for (i, c) in fragment.chars().enumerate() {
        if c == '\n' || c == '\r' || i >= 20 {
            break;
        }
        end_col += 1;
    }

    if end_col == start_col {
        end_col += 1;
    }

    Range {
        start_line,
        start_col,
        end_line: start_line,
        end_col,
    }
}

pub fn parse_script(input: &str) -> (ast::Script, Vec<(String, Range)>) {
    // Strip BOM if present — all ByteSpan offsets are relative to the cleaned text,
    // and Script.source will contain the cleaned text.
    let input_clean: Arc<str> = Arc::from(input.strip_prefix('\u{feff}').unwrap_or(input));
    let source_ref: &str = &input_clean;
    let mut span = Span::new(source_ref);
    let mut entries = Vec::new();
    let mut errors = Vec::new();

    loop {
        // Skip leading whitespace
        if let Ok((remainder, _)) = multispace0::<Span, nom::error::Error<Span>>(span) {
            span = remainder;
        }

        if span.fragment().is_empty() {
            break;
        }

        match parse_entry(span) {
            Ok((remainder, entry)) => {
                entries.push(entry);
                span = remainder;
            }
            Err(_) => {
                let range = to_error_range(span);
                let mut snippet = span
                    .fragment()
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if snippet.len() > 30 {
                    snippet = snippet.chars().take(30).collect::<String>();
                    snippet.push_str("...");
                }
                if snippet.is_empty() {
                    snippet = span.fragment().chars().take(10).collect::<String>();
                }

                errors.push((format!("Parsing error near: '{}'", snippet), range));

                // Recovery: skip to the next line and try again.
                // CRITICAL: use `take` to consume bytes through nom, preserving
                // LocatedSpan's offset tracking.  Span::new(subslice) would reset
                // location_offset() to 0, making future ByteSpan offsets relative
                // to the subslice instead of the original source — causing
                // &source[start..end] to land inside multi-byte characters.
                if let Some(pos) = span.fragment().find('\n') {
                    if let Ok((remaining, _)) = take::<_, _, nom::error::Error<Span>>(pos + 1)(span)
                    {
                        span = remaining;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
    }

    (
        ast::Script {
            source: input_clean,
            entries,
        },
        errors,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let input = r#"
        # This is a test HOI4 script
        country_event = {
            id = test.1
            is_triggered_only = yes
            trigger = {
                tag = GER
                has_war = no
            }
        }
        "#;
        let (script, errors) = parse_script(input);
        assert!(errors.is_empty());
        assert_eq!(script.entries.len(), 2); // Comment and Assignment
    }

    #[test]
    fn test_parse_complex() {
        let input = r#"
        modifier = {
            political_power_factor = 0.15
            stability_factor > -0.1
            tag != "ENG"
            [?my_var] = 10
            array^0 = 1
        }
        "#;
        let (script, errors) = parse_script(input);
        assert!(errors.is_empty());
        assert_eq!(script.entries.len(), 1);
    }

    #[test]
    fn test_parse_quoted_escapes() {
        let input = r#"title = "Event \"The Great War\" Begins""#;
        let (_script, errors) = parse_script(input);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_parse_dots_in_key() {
        let input = r#"title = daw.2.t"#;
        let (script, errors) = parse_script(input);
        assert!(errors.is_empty());
        assert_eq!(script.entries.len(), 1);
        if let ast::Entry::Assignment(ass) = &script.entries[0] {
            let val = ass.value.value.as_str(&script.source);
            assert_eq!(val, Some("daw.2.t"));
        } else {
            panic!("Value should be a string/identifier");
        }
    }

    #[test]
    fn test_parse_pipe_in_value() {
        let input = r#"custom_effect_tooltip = tech_effect|sp_naval_support_ships_pick_a"#;
        let (_script, errors) = parse_script(input);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_byte_span_resolve() {
        let input = "key = hello_world";
        let (script, _errors) = parse_script(input);
        assert!(_errors.is_empty());
        if let ast::Entry::Assignment(ass) = &script.entries[0] {
            assert_eq!(ass.key_text(&script.source), "key");
            if let ast::Value::String(span) = &ass.value.value {
                assert_eq!(span.resolve(&script.source), "hello_world");
            } else {
                panic!("Expected Value::String");
            }
        }
    }

    #[test]
    fn test_quoted_string_value() {
        let input = r#"name = "hello world""#;
        let (script, errors) = parse_script(input);
        assert!(errors.is_empty());
        if let ast::Entry::Assignment(ass) = &script.entries[0] {
            if let ast::Value::QuotedString(s) = &ass.value.value {
                assert_eq!(s, "hello world");
            } else {
                panic!("Expected Value::QuotedString");
            }
        }
    }

    #[test]
    fn test_byte_span_comment() {
        let input = "# comment text\nkey = val";
        let (script, errors) = parse_script(input);
        assert!(errors.is_empty());
        if let ast::Entry::Comment(bs, _) = &script.entries[0] {
            assert_eq!(bs.resolve(&script.source), " comment text");
        }
    }

    #[test]
    fn test_number_vs_string() {
        let input = "num = 15.0\nstr = test_val";
        let (script, errors) = parse_script(input);
        assert!(errors.is_empty());
        // First: number
        if let ast::Entry::Assignment(ass) = &script.entries[0] {
            assert!(matches!(ass.value.value, ast::Value::Number(_)));
        }
        // Second: string
        if let ast::Entry::Assignment(ass) = &script.entries[1] {
            assert!(matches!(ass.value.value, ast::Value::String(_)));
            assert_eq!(ass.value.value.as_str(&script.source), Some("test_val"));
        }
    }

    #[test]
    fn test_tagged_block_tag() {
        let input = "my_tag { inner = val }";
        let (script, errors) = parse_script(input);
        assert!(errors.is_empty());
        if let ast::Entry::Value(nv) = &script.entries[0] {
            if let ast::Value::TaggedBlock(tag, _, _) = &nv.value {
                assert_eq!(tag.resolve(&script.source), "my_tag");
            } else {
                panic!("Expected TaggedBlock, got {:?}", nv.value);
            }
        } else {
            panic!("Expected Entry::Value with TaggedBlock");
        }
    }

    #[test]
    fn test_entry_key_helper() {
        let input = "some_key = 42";
        let (script, errors) = parse_script(input);
        assert!(errors.is_empty());
        if let ast::Entry::Assignment(ass) = &script.entries[0] {
            assert_eq!(ass.key_text(&script.source), "some_key");
        } else {
            panic!("Expected Assignment entry");
        }
    }

    #[test]
    fn test_bom_stripping() {
        let input = "\u{feff}key = val";
        let (script, errors) = parse_script(input);
        assert!(errors.is_empty());
        if let ast::Entry::Assignment(ass) = &script.entries[0] {
            assert_eq!(ass.key_text(&script.source), "key");
        } else {
            panic!("Expected Assignment entry");
        }
        // Source should NOT contain BOM
        assert!(!script.source.contains('\u{feff}'));
    }

    #[test]
    fn test_multibyte_byte_span_offsets() {
        // Test with non-ASCII characters spanning multiple bytes
        // Using accented chars which ARE valid identifier chars (alphanumeric + special)
        let input = "äöü_ñ = value\nkey = café\n# naïve comment\ncafé = 42\nname = \"café ｡\"";
        let (script, errors) = parse_script(input);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);

        // Entry 0: "äöü_ñ = value"
        if let ast::Entry::Assignment(ass) = &script.entries[0] {
            let key = ass.key_text(&script.source);
            assert_eq!(key, "äöü_ñ", "Got key: {:?}", key);
            let val = ass.value.value.as_str(&script.source);
            assert_eq!(val, Some("value"));
        } else {
            panic!("Entry 0 should be Assignment");
        }

        // Entry 1: "key = café" — café as identifier value
        if let ast::Entry::Assignment(ass) = &script.entries[1] {
            let key = ass.key_text(&script.source);
            assert_eq!(key, "key");
            let val = ass.value.value.as_str(&script.source);
            assert_eq!(val, Some("café"), "Value should be café, got {:?}", val);
        } else {
            panic!("Entry 1 should be Assignment");
        }

        // Entry 2: comment "# naïve comment"
        if let ast::Entry::Comment(bs, _) = &script.entries[2] {
            let comment = bs.resolve(&script.source);
            assert_eq!(
                comment, " naïve comment",
                "Comment text mismatch: {:?}",
                comment
            );
        } else {
            panic!("Entry 2 should be Comment");
        }

        // Entry 3: "café = 42"
        if let ast::Entry::Assignment(ass) = &script.entries[3] {
            let key = ass.key_text(&script.source);
            assert_eq!(key, "café");
        } else {
            panic!("Entry 3 should be Assignment");
        }

        // Entry 4: 'name = "café ｡"' — quoted string with special chars
        if let ast::Entry::Assignment(ass) = &script.entries[4] {
            let key = ass.key_text(&script.source);
            assert_eq!(key, "name");
            assert!(matches!(ass.value.value, ast::Value::QuotedString(_)));
        } else {
            panic!("Entry 4 should be Assignment");
        }
    }

    #[test]
    fn test_multibyte_bom_stripping() {
        // BOM + multi-byte characters: ensure offsets align after BOM stripping
        let input = "\u{feff}äöü_ñ = value\nkey = café";
        let (script, errors) = parse_script(input);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert!(
            !script.source.contains('\u{feff}'),
            "BOM should be stripped from source"
        );

        if let ast::Entry::Assignment(ass) = &script.entries[0] {
            let key = ass.key_text(&script.source);
            assert_eq!(key, "äöü_ñ", "First key after BOM, got {:?}", key);
        } else {
            panic!("Entry 0 should be Assignment");
        }
    }

    #[test]
    fn test_error_recovery_multibyte_offsets() {
        // Regression test: error recovery after `Span::new(subslice)` would reset
        // location_offset to 0, making subsequent ByteSpan offsets relative to the
        // subslice instead of the full source.  With multi-byte characters in the
        // file, this causes &source[bad_start..] to land inside a multi-byte char.
        //
        // The line "~~~~" causes a parse error (~ is not a valid identifier char),
        // triggering error recovery.  The following lines have multi-byte chars.
        let input = "äöü = first\n~~~~\ncafé = third\n# naïve note\nkey = last";
        let (script, errors) = parse_script(input);
        // We expect some errors from the bad line, but the rest should parse
        assert!(!errors.is_empty(), "Should have parse errors from bad line");

        // Entry 0: "äöü = first" — parsed BEFORE recovery, always correct
        if let ast::Entry::Assignment(ass) = &script.entries[0] {
            assert_eq!(ass.key_text(&script.source), "äöü");
            assert_eq!(ass.value.value.as_str(&script.source), Some("first"));
        } else {
            panic!("Entry 0 should be Assignment");
        }

        // After recovery, remaining entries should still resolve correctly.
        // Find "key = last"
        let last_entry = script.entries.iter().find(
            |e| matches!(e, ast::Entry::Assignment(ass) if ass.key_text(&script.source) == "key"),
        );
        assert!(last_entry.is_some(), "Should have found 'key = last'");
        if let Some(ast::Entry::Assignment(ass)) = last_entry {
            assert_eq!(ass.value.value.as_str(&script.source), Some("last"));
        }

        // Comment "# naïve note" should also resolve correctly
        let comment_entry = script.entries.iter().find(|e| {
            matches!(e, ast::Entry::Comment(bs, _) if bs.resolve(&script.source) == " naïve note")
        });
        assert!(comment_entry.is_some(), "Should have found comment");

        // Entry "café = third" should resolve correctly
        let cafe_entry = script.entries.iter().find(
            |e| matches!(e, ast::Entry::Assignment(ass) if ass.key_text(&script.source) == "café"),
        );
        assert!(cafe_entry.is_some(), "Should have found 'café = third'");
        if let Some(ast::Entry::Assignment(ass)) = cafe_entry {
            assert_eq!(ass.value.value.as_str(&script.source), Some("third"));
        }
    }
}
