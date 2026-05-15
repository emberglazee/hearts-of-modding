use crate::ast::*;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{anychar, char, multispace0},
    combinator::{map, opt, recognize},
    multi::many0,
    sequence::preceded,
};
use nom_locate::LocatedSpan;

type Span<'a> = LocatedSpan<&'a str>;

fn to_range(span: Span) -> crate::ast::Range {
    let start_line = span.location_line() - 1;
    let start_col = span.get_column() as u32 - 1;
    let fragment = span.fragment();

    // Calculate end line and column by counting newlines in fragment
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
        start_col + fragment.len() as u32
    } else {
        last_line_len
    };

    crate::ast::Range {
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

fn identifier(input: Span) -> IResult<Span, (String, crate::ast::Range)> {
    map(take_while1(is_identifier_char), |s: Span| {
        (s.fragment().to_string(), to_range(s))
    })
    .parse(input)
}

fn quoted_string(input: Span) -> IResult<Span, (String, crate::ast::Range)> {
    let (input, start) = recognize(char('"')).parse(input)?;
    let mut s = String::new();
    let mut current = input;
    loop {
        let (next, c) = anychar(current)?;
        if c == '"' {
            let range = crate::ast::Range {
                start_line: start.location_line() - 1,
                start_col: start.get_column() as u32 - 1,
                end_line: next.location_line() - 1,
                end_col: next.get_column() as u32 - 1,
            };
            return Ok((next, (s, range)));
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

fn number(input: Span) -> IResult<Span, (f64, crate::ast::Range)> {
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

    // Boundary check: next char should not be an alphanumeric char unless it's a known identifier char that is NOT part of a number
    // Actually, in Paradox script, numbers are often followed by %, so we should allow that.
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

fn boolean(input: Span) -> IResult<Span, (bool, crate::ast::Range)> {
    let (input, s) = alt((tag("yes"), tag("no"))).parse(input)?;
    // Ensure boundary
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

fn comment(input: Span) -> IResult<Span, (String, crate::ast::Range)> {
    let (input, s) =
        recognize(preceded(char('#'), take_while(|c| c != '\n' && c != '\r'))).parse(input)?;
    Ok((input, (s.fragment()[1..].to_string(), to_range(s))))
}

fn operator(input: Span) -> IResult<Span, (Operator, crate::ast::Range)> {
    alt((
        map(tag("<="), |s| (Operator::LessOrEqual, to_range(s))),
        map(tag(">="), |s| (Operator::GreaterOrEqual, to_range(s))),
        map(tag("!="), |s| (Operator::NotEquals, to_range(s))),
        map(tag("="), |s| (Operator::Equals, to_range(s))),
        map(tag("<"), |s| (Operator::LessThan, to_range(s))),
        map(tag(">"), |s| (Operator::GreaterThan, to_range(s))),
    ))
    .parse(input)
}

fn parse_tagged_block(
    input: Span,
) -> IResult<Span, (String, Vec<Entry>, crate::ast::Range, crate::ast::Range)> {
    let (input, (tag, tag_range)) = identifier(input)?;
    let (input, (entries, block_range)) = preceded(multispace0, parse_block).parse(input)?;

    let range = crate::ast::Range {
        start_line: tag_range.start_line,
        start_col: tag_range.start_col,
        end_line: block_range.end_line,
        end_col: block_range.end_col,
    };

    Ok((input, (tag, entries, block_range, range)))
}

fn parse_value(input: Span) -> IResult<Span, NodeedValue> {
    alt((
        map(quoted_string, |(s, r)| NodeedValue {
            value: Value::String(s),
            range: r,
        }),
        map(boolean, |(b, r)| NodeedValue {
            value: Value::Boolean(b),
            range: r,
        }),
        map(parse_tagged_block, |(tag, entries, br, r)| NodeedValue {
            value: Value::TaggedBlock(tag, entries, br),
            range: r,
        }),
        // Try identifier FIRST because it can contain dots and start with numbers (like 1.0.1)
        map(identifier, |(s, r)| NodeedValue {
            value: Value::String(s),
            range: r,
        }),
        map(number, |(n, r)| NodeedValue {
            value: Value::Number(n),
            range: r,
        }),
        map(parse_block, |(v, r)| NodeedValue {
            value: Value::Block(v),
            range: r,
        }),
    ))
    .parse(input)
}

fn parse_block(input: Span) -> IResult<Span, (Vec<Entry>, crate::ast::Range)> {
    let (input, start) = preceded(multispace0, recognize(char('{'))).parse(input)?;
    let (input, entries) = many0(preceded(multispace0, parse_entry)).parse(input)?;
    let (input, end) = preceded(multispace0, recognize(char('}'))).parse(input)?;

    let range = crate::ast::Range {
        start_line: start.location_line() - 1,
        start_col: start.get_column() as u32 - 1,
        end_line: end.location_line() - 1,
        end_col: end.get_column() as u32,
    };
    Ok((input, (entries, range)))
}

fn parse_assignment(input: Span) -> IResult<Span, Assignment> {
    let (input, (key, key_range)) = alt((identifier, quoted_string)).parse(input)?;
    let (input, (op, op_range)) = preceded(multispace0, operator).parse(input)?;
    let (input, val) = preceded(multispace0, parse_value).parse(input)?;

    Ok((
        input,
        Assignment {
            key,
            key_range,
            operator: op,
            operator_range: op_range,
            value: val,
        },
    ))
}

fn parse_entry(input: Span) -> IResult<Span, Entry> {
    alt((
        map(comment, |(s, r)| Entry::Comment(s, r)),
        map(parse_assignment, Entry::Assignment),
        map(parse_value, Entry::Value),
    ))
    .parse(input)
}

fn to_error_range(span: Span) -> crate::ast::Range {
    let start_line = span.location_line() - 1;
    let start_col = span.get_column() as u32 - 1;
    let fragment = span.fragment();

    // Find the end of the current line
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

    crate::ast::Range {
        start_line,
        start_col,
        end_line: start_line,
        end_col,
    }
}

pub fn parse_script(input: &str) -> (Script, Vec<(String, crate::ast::Range)>) {
    let input_clean = input.strip_prefix('\u{feff}').unwrap_or(input);
    let mut span = Span::new(input_clean);
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

                // Recovery: skip one character and try again
                let (next, _) = nom::bytes::complete::take::<usize, Span, nom::error::Error<Span>>(
                    1usize,
                )(span)
                .unwrap_or((span.clone(), span));
                span = next;
            }
        }
    }

    (Script { entries }, errors)
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
        let result = parse_script(input);
        assert!(result.1.is_empty());
        let script = result.0;
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
        let result = parse_script(input);
        assert!(result.1.is_empty());
        let script = result.0;
        assert_eq!(script.entries.len(), 1);
    }

    #[test]
    fn test_parse_quoted_escapes() {
        let input = r#"title = "Event \"The Great War\" Begins""#;
        let result = parse_script(input);
        assert!(result.1.is_empty());
    }

    #[test]
    fn test_parse_dots_in_key() {
        let input = r#"title = daw.2.t"#;
        let result = parse_script(input);
        assert!(result.1.is_empty());
        let script = result.0;
        if let Entry::Assignment(ass) = &script.entries[0] {
            if let Value::String(s) = &ass.value.value {
                assert_eq!(s, "daw.2.t");
            } else {
                panic!("Value should be a string/identifier");
            }
        }
    }

    #[test]
    fn test_parse_pipe_in_value() {
        let input = r#"custom_effect_tooltip = tech_effect|sp_naval_support_ships_pick_a"#;
        let result = parse_script(input);
        assert!(result.1.is_empty());
    }
}
