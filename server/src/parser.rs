use crate::ast::*;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while1, take_while},
    character::complete::{char, multispace0, none_of, anychar, one_of},
    combinator::{map, opt, recognize, peek, eof},
    multi::many0,
    sequence::{delimited, preceded, tuple},
    IResult,
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
    c.is_alphanumeric() || c == '_' || c == '.' || c == ':' || c == '@' || c == '[' || c == ']' || c == '?' || c == '^' || c == '$' || c == '/' || c == '-'
}

fn identifier(input: Span) -> IResult<Span, (String, crate::ast::Range)> {
    map(take_while1(is_identifier_char), |s: Span| {
        (s.fragment().to_string(), to_range(s))
    })(input)
}

fn quoted_string(input: Span) -> IResult<Span, (String, crate::ast::Range)> {
    let (input, start) = recognize(char('"'))(input)?;
    let mut s = String::new();
    let mut current = input;
    loop {
        let (next, c) = anychar(current)?;
        if c == '"' {
            let range = crate::ast::Range {
                start_line: start.location_line() - 1,
                start_col: start.get_column() as u32 - 1,
                end_line: next.location_line() - 1,
                end_col: next.get_column() as u32,
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
    let (input, s) = recognize(tuple((
        opt(char('-')),
        alt((
            recognize(tuple((take_while1(|c: char| c.is_ascii_digit()), opt(tuple((char('.'), take_while(|c: char| c.is_ascii_digit()))))))),
            recognize(tuple((char('.'), take_while1(|c: char| c.is_ascii_digit())))),
        ))
    )))(input)?;
    
    // Ensure we're not immediately followed by more identifier chars (like in "1.0.1")
    let (input, _) = peek(alt((
        recognize(eof),
        recognize(multispace0::<Span, nom::error::Error<Span>>), // Should be at least one space if not eof or special char
        recognize(one_of(" \t\r\n#{}<>=!")),
    )))(input)?;
    
    // BUT, we must also ensure the NEXT char is definitely not an identifier char if it was a space
    let next_char = input.fragment().chars().next();
    if let Some(c) = next_char {
        if is_identifier_char(c) && !c.is_ascii_whitespace() && c != '#' && c != '{' && c != '}' {
             return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
        }
    }

    Ok((input, (s.fragment().parse().unwrap(), to_range(s))))
}

fn boolean(input: Span) -> IResult<Span, (bool, crate::ast::Range)> {
    let (input, s) = alt((tag("yes"), tag("no")))(input)?;
    // Ensure boundary
    let (input, _) = peek(alt((
        recognize(eof),
        recognize(one_of(" \t\r\n#{}<>=!")),
    )))(input)?;
    
    Ok((input, (*s.fragment() == "yes", to_range(s))))
}

fn comment(input: Span) -> IResult<Span, (String, crate::ast::Range)> {
    let (input, s) = recognize(preceded(char('#'), take_until("\n")))(input)?;
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
    ))(input)
}

fn parse_tagged_block(input: Span) -> IResult<Span, (String, Vec<Entry>, crate::ast::Range)> {
    let (input, (tag, tag_range)) = identifier(input)?;
    let (input, (entries, block_range)) = preceded(multispace0, parse_block)(input)?;
    
    let range = crate::ast::Range {
        start_line: tag_range.start_line,
        start_col: tag_range.start_col,
        end_line: block_range.end_line,
        end_col: block_range.end_col,
    };
    
    Ok((input, (tag, entries, range)))
}

fn parse_value(input: Span) -> IResult<Span, NodeedValue> {
    alt((
        map(quoted_string, |(s, r)| NodeedValue { value: Value::String(s), range: r }),
        map(boolean, |(b, r)| NodeedValue { value: Value::Boolean(b), range: r }),
        map(parse_tagged_block, |(tag, entries, r)| NodeedValue { value: Value::TaggedBlock(tag, entries), range: r }),
        // Try identifier FIRST because it can contain dots and start with numbers (like 1.0.1)
        map(identifier, |(s, r)| NodeedValue { value: Value::String(s), range: r }),
        map(number, |(n, r)| NodeedValue { value: Value::Number(n), range: r }),
        map(parse_block, |(v, r)| NodeedValue { value: Value::Block(v), range: r }),
    ))(input)
}

fn parse_block(input: Span) -> IResult<Span, (Vec<Entry>, crate::ast::Range)> {
    let (input, start) = preceded(multispace0, recognize(char('{')))(input)?;
    let (input, entries) = many0(preceded(multispace0, parse_entry))(input)?;
    let (input, end) = preceded(multispace0, recognize(char('}')))(input)?;
    
    let range = crate::ast::Range {
        start_line: start.location_line() - 1,
        start_col: start.get_column() as u32 - 1,
        end_line: end.location_line() - 1,
        end_col: end.get_column() as u32,
    };
    Ok((input, (entries, range)))
}

fn parse_assignment(input: Span) -> IResult<Span, Assignment> {
    let (input, (key, key_range)) = alt((identifier, quoted_string))(input)?;
    let (input, (op, op_range)) = preceded(multispace0, operator)(input)?;
    let (input, val) = preceded(multispace0, parse_value)(input)?;
    
    Ok((input, Assignment {
        key,
        key_range,
        operator: op,
        operator_range: op_range,
        value: val,
    }))
}

fn parse_entry(input: Span) -> IResult<Span, Entry> {
    alt((
        map(comment, |(s, r)| Entry::Comment(s, r)),
        map(parse_assignment, Entry::Assignment),
        map(parse_value, Entry::Value),
    ))(input)
}

pub fn parse_script(input: &str) -> Result<Script, (String, crate::ast::Range)> {
    let span = Span::new(input);
    match many0(preceded(multispace0, parse_entry))(span) {
        Ok((remainder, entries)) => {
            let (remainder, _) = multispace0::<Span, nom::error::Error<Span>>(remainder).unwrap();
            if remainder.fragment().is_empty() {
                Ok(Script { entries })
            } else {
                let range = to_range(remainder);
                Err((format!("Parsing error near: {}", remainder.fragment().chars().take(20).collect::<String>()), range))
            }
        }
        Err(e) => {
            // This is a bit simplified, ideally we'd extract the span from the error
            Err((format!("Critical parsing error: {:?}", e), crate::ast::Range {
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 0,
            }))
        },
    }
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
        assert!(result.is_ok());
        let script = result.unwrap();
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
        assert!(result.is_ok());
        let script = result.unwrap();
        assert_eq!(script.entries.len(), 1);
    }
    
    #[test]
    fn test_parse_quoted_escapes() {
        let input = r#"title = "Event \"The Great War\" Begins""#;
        let result = parse_script(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_dots_in_key() {
        let input = r#"title = daw.2.t"#;
        let result = parse_script(input);
        assert!(result.is_ok());
        let script = result.unwrap();
        if let Entry::Assignment(ass) = &script.entries[0] {
            if let Value::String(s) = &ass.value.value {
                assert_eq!(s, "daw.2.t");
            } else {
                panic!("Value should be a string/identifier");
            }
        }
    }
}