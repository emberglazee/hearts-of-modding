use std::collections::HashMap;
use nom::{
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{char, multispace0, none_of, digit1, space0},
    combinator::{map, opt, recognize},
    multi::many0,
    sequence::{delimited, preceded, tuple},
    IResult,
};
use nom_locate::LocatedSpan;
use crate::ast::Range;

type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug, Clone)]
pub struct LocEntry {
    pub key: String,
    pub value: String,
    pub range: Range,
    pub path: String,
}

fn to_range(span: Span) -> Range {
    let start_line = span.location_line() - 1;
    let start_col = span.get_column() as u32 - 1;
    Range {
        start_line,
        start_col,
        end_line: start_line,
        end_col: start_col + span.fragment().len() as u32,
    }
}

pub fn parse_loc_file(input: &str, path: &str) -> HashMap<String, LocEntry> {
    let mut map = HashMap::new();
    
    // Handle UTF-8 BOM if present
    let input = input.strip_prefix('\u{feff}').unwrap_or(input);

    let span = Span::new(input);
    
    // Skip the l_language: header
    // We use a more robust way to find the header: it should start with l_ and end with :
    let (remainder, _) = match preceded(multispace0::<Span, nom::error::Error<Span>>, recognize(tuple((tag("l_"), take_until(":"), tag(":")))))(span) {
        Ok(res) => res,
        Err(_) => {
            // If we can't find the header with the standard parser, try a simpler fallback
            // Some files might have weird whitespace or comments before the header
            if let Some(pos) = input.find("l_") {
                if let Some(colon_pos) = input[pos..].find(':') {
                    let remainder = Span::new(&input[pos + colon_pos + 1..]);
                    // Return a dummy span for the second element to match the type
                    (remainder, remainder)
                } else {
                    return map;
                }
            } else {
                return map;
            }
        }
    };

    let mut current = remainder;
    while !current.fragment().is_empty() {
        match parse_loc_entry(current, path) {
            Ok((remainder, entry)) => {
                map.insert(entry.key.clone(), entry);
                current = remainder;
            }
            Err(_) => {
                // If we fail to parse an entry, skip the line and try again
                let next_line = current.fragment().find('\n').map(|i| i + 1).unwrap_or(current.fragment().len());
                if next_line < current.fragment().len() {
                    current = Span::new(&current.fragment()[next_line..]);
                } else {
                    break;
                }
            }
        }
    }

    map
}

fn parse_loc_entry<'a>(input: Span<'a>, path: &'a str) -> IResult<Span<'a>, LocEntry> {
    let (input, _) = multispace0(input)?;
    let (input, key_span) = take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '.')(input)?;
    let (input, _) = char(':')(input)?;
    let (input, _) = opt(digit1)(input)?; // Optional version number
    let (input, _) = space0(input)?;
    let (input, value) = delimited(
        char('"'),
        map(many0(none_of("\"")), |v| v.into_iter().collect::<String>()),
        char('"'),
    )(input)?;
    
    Ok((input, LocEntry {
        key: key_span.fragment().to_string(),
        value,
        range: to_range(key_span),
        path: path.to_string(),
    }))
}