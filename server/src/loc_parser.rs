use std::collections::HashMap;
use nom::{
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{char, multispace0, none_of, digit1, space0},
    combinator::opt,
    multi::many0,
    IResult,
};
use nom_locate::LocatedSpan;
use crate::ast::{Range, DiagnosticSeverity};

type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug, Clone)]
pub struct LocEntry {
    pub key: String,
    pub value: String,
    pub range: Range,
    pub path: String,
    pub value_start_col: u32,
}

#[derive(Debug, Clone)]
pub struct LocDiagnostic {
    pub range: Range,
    pub message: String,
    pub severity: DiagnosticSeverity,
    pub code: Option<String>,
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

pub fn validate_loc_string(entry: &LocEntry, event_targets: &HashMap<String, Vec<crate::variable_scanner::EventTarget>>) -> Vec<LocDiagnostic> {
    let mut diagnostics = Vec::new();
    
    let re_scope = regex::Regex::new(r"\[([^\]]+)\]").unwrap();
    let re_color = regex::Regex::new(r"§([a-zA-Z0-9!])").unwrap();

    let loc_commands = ["GetTag", "GetName", "GetAdjective", "GetRulingIdeology", "GetRulingIdeologyNoun", "GetPartyName", "GetLeaderName", "GetLeaderNameDef", "GetPlayerName", "GetCapitalName", "GetYear", "GetMonth", "GetDay", "GetFlag", "GetDate", "GetTime"];
    let scopes = ["ROOT", "FROM", "PREV", "THIS", "COUNTRY", "STATE", "UNIT", "CHARACTER", "GLOBAL"];

    // 1. Validate Scopes [Root.GetTag]
    for cap in re_scope.captures_iter(&entry.value) {
        let full_match = cap.get(0).unwrap();
        let inner = cap.get(1).unwrap().as_str();
        let start_pos = full_match.start();
        
        let parts: Vec<&str> = inner.split('.').collect();
        let mut current_part_start = start_pos + 1; // +1 for [

        for (i, part) in parts.iter().enumerate() {
            let is_last = i == parts.len() - 1;
            let mut valid = false;
            let part_upper = part.to_uppercase();

            if is_last {
                if loc_commands.iter().any(|&c| c.to_lowercase() == part.to_lowercase()) ||
                   scopes.contains(&part_upper.as_str()) ||
                   event_targets.contains_key(*part) {
                    valid = true;
                }
            } else {
                if scopes.contains(&part_upper.as_str()) ||
                   event_targets.contains_key(*part) {
                    valid = true;
                }
            }

            if !valid {
                let range = Range {
                    start_line: entry.range.start_line,
                    start_col: entry.value_start_col + current_part_start as u32,
                    end_line: entry.range.start_line,
                    end_col: entry.value_start_col + current_part_start as u32 + part.len() as u32,
                };
                diagnostics.push(LocDiagnostic {
                    range,
                    message: format!("Potential invalid localization scope or command: '{}'", part),
                    severity: DiagnosticSeverity::Warning,
                    code: Some("invalid_loc_scope".to_string()),
                });
            }
            current_part_start += part.len() + 1; // +1 for .
        }
    }

    // 2. Validate Color Codes §Y...§!
    let mut open_colors = Vec::new();
    for cap in re_color.captures_iter(&entry.value) {
        let m = cap.get(0).unwrap();
        let code = cap.get(1).unwrap().as_str();
        let pos = m.start();

        if code == "!" {
            if open_colors.is_empty() {
                let range = Range {
                    start_line: entry.range.start_line,
                    start_col: entry.value_start_col + pos as u32,
                    end_line: entry.range.start_line,
                    end_col: entry.value_start_col + pos as u32 + 2,
                };
                diagnostics.push(LocDiagnostic {
                    range,
                    message: "Dangling color reset symbol '§!' without an opening color code.".to_string(),
                    severity: DiagnosticSeverity::Information,
                    code: Some("dangling_color_reset".to_string()),
                });
            } else {
                open_colors.pop();
            }
        } else {
            open_colors.push((code.to_string(), pos));
        }
    }

    for (code, pos) in open_colors {
        let range = Range {
            start_line: entry.range.start_line,
            start_col: entry.value_start_col + pos as u32,
            end_line: entry.range.start_line,
            end_col: entry.value_start_col + pos as u32 + 2,
        };
        diagnostics.push(LocDiagnostic {
            range,
            message: format!("Unclosed color code '§{}'. Expected a matching '§!' reset.", code),
            severity: DiagnosticSeverity::Information,
            code: Some("unclosed_color_code".to_string()),
        });
    }

    diagnostics
}

pub fn validate_loc_file_structure(input: &str) -> Vec<LocDiagnostic> {
    let mut diagnostics = Vec::new();
    
    // Check for l_language header
    let valid_languages = ["l_english", "l_braz_por", "l_french", "l_german", "l_polish", "l_russian", "l_spanish"];
    let content_without_bom = input.strip_prefix('\u{feff}').unwrap_or(input);
    
    let mut header_found = false;
    for (line_idx, line) in content_without_bom.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.contains(':') {
            let parts: Vec<&str> = trimmed.split(':').collect();
            let lang = parts[0].trim();
            if valid_languages.contains(&lang) {
                header_found = true;
            } else if lang.starts_with("l_") {
                diagnostics.push(LocDiagnostic {
                    range: Range { 
                        start_line: line_idx as u32, 
                        start_col: line.find(lang).unwrap_or(0) as u32,
                        end_line: line_idx as u32,
                        end_col: (line.find(lang).unwrap_or(0) + lang.len()) as u32,
                    },
                    message: format!("Unknown Paradox language header: '{}'. Valid languages are: {}", lang, valid_languages.join(", ")),
                    severity: DiagnosticSeverity::Warning,
                    code: Some("unknown_language".to_string()),
                });
                header_found = true; // Still found a header, just a weird one
            }
            break;
        }
    }

    if !header_found {
        diagnostics.push(LocDiagnostic {
            range: Range { start_line: 0, start_col: 0, end_line: 0, end_col: 10 },
            message: "Missing language header (e.g., 'l_english:'). Localization files must start with a supported language tag.".to_string(),
            severity: DiagnosticSeverity::Error,
            code: Some("missing_language_header".to_string()),
        });
    }

    diagnostics
}

pub fn parse_loc_file(input: &str, path: &str) -> (HashMap<String, LocEntry>, Vec<LocDiagnostic>) {
    let mut map = HashMap::new();
    let diagnostics = validate_loc_file_structure(input);
    
    let input_clean = input.strip_prefix('\u{feff}').unwrap_or(input);
    let span = Span::new(input_clean);
    
    let mut current = span;
    let mut header_found = false;

    // Fast forward to the header while preserving the nom_locate line/column tracking
    while !current.fragment().is_empty() {
        if current.fragment().starts_with("l_") && current.fragment().contains(':') {
            if let Ok((rem, _)) = take_until::<&str, Span, nom::error::Error<Span>>(":")(current) {
                if let Ok((rem2, _)) = tag::<&str, Span, nom::error::Error<Span>>(":")(rem) {
                    current = rem2;
                    header_found = true;
                    break;
                }
            }
        }
        
        if let Ok((rem, _)) = nom::bytes::complete::take::<usize, Span, nom::error::Error<Span>>(1usize)(current) {
            current = rem;
        } else {
            break;
        }
    }

    if !header_found {
        return (map, diagnostics);
    }

    while !current.fragment().is_empty() {
        match parse_loc_entry(current, path) {
            Ok((remainder, entry)) => {
                map.insert(entry.key.clone(), entry);
                current = remainder;
            }
            Err(_) => {
                let next_line = current.fragment().find('\n').map(|i| i + 1).unwrap_or(current.fragment().len());
                if let Ok((rem, _)) = nom::bytes::complete::take::<usize, Span, nom::error::Error<Span>>(next_line)(current) {
                    current = rem;
                } else {
                    break;
                }
            }
        }
    }

    (map, diagnostics)
}

fn parse_loc_entry<'a>(input: Span<'a>, path: &'a str) -> IResult<Span<'a>, LocEntry> {
    let (input, _) = multispace0(input)?;
    let (input, key_span) = take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '.' || c == '-')(input)?;
    let (input, _) = char(':')(input)?;
    let (input, _) = opt(digit1)(input)?;
    let (input, _) = space0(input)?;
    
    let (input, _) = char('"')(input)?;
    let start_val = input;
    let (input, value_chars) = many0(none_of("\""))(input)?;
    let (input, _) = char('"')(input)?;
    
    let value = value_chars.into_iter().collect::<String>();
    
    Ok((input, LocEntry {
        key: key_span.fragment().to_string(),
        value,
        range: to_range(key_span),
        path: path.to_string(),
        value_start_col: start_val.get_column() as u32 - 1,
    }))
}

pub fn format_loc_file(input: &str, cosmetic_indent: bool) -> String {
    let mut output = String::new();
    
    // Ensure UTF-8 BOM
    if !input.starts_with('\u{feff}') {
        output.push('\u{feff}');
    }

    let mut lines = input.lines();
    
    // Find and format the header
    let mut header_found = false;
    for line in lines.by_ref() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            output.push_str(line);
            output.push('\n');
            continue;
        }
        if trimmed.starts_with('#') {
            output.push_str(line);
            output.push('\n');
            continue;
        }
        if trimmed.starts_with("l_") && trimmed.contains(':') {
            output.push_str(trimmed);
            output.push('\n');
            header_found = true;
            break;
        }
    }

    if !header_found {
        // If no header found, we can't safely format as HOI4 loc
        return input.to_string();
    }

    let mut last_base_key = String::new();
    let variants = ["_DEF", "_ADJ", "_desc", "_party", "_party_long", "_ADJ_DEF"];

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            output.push_str(line);
            output.push('\n');
            continue;
        }

        let mut current_indent = "\t".to_string();

        if trimmed.starts_with('#') {
            output.push_str(&current_indent);
            output.push_str(trimmed);
            output.push('\n');
            continue;
        }

        // Parse entry manually for formatting
        if let Some(colon_pos) = trimmed.find(':') {
            let key = trimmed[..colon_pos].trim();
            
            if cosmetic_indent {
                let mut is_variant = false;
                for var in variants {
                    if key.ends_with(var) {
                        let base = &key[..key.len() - var.len()];
                        if base == last_base_key {
                            is_variant = true;
                            break;
                        }
                    }
                }

                if is_variant {
                    current_indent.push('\t');
                } else {
                    last_base_key = key.to_string();
                }
            }

            let remainder = &trimmed[colon_pos + 1..];
            
            let (version, remainder) = if let Some(space_pos) = remainder.find(' ') {
                let v = remainder[..space_pos].trim();
                (if v.is_empty() { "0" } else { v }, &remainder[space_pos..])
            } else if let Some(quote_pos) = remainder.find('"') {
                let v = remainder[..quote_pos].trim();
                (if v.is_empty() { "0" } else { v }, &remainder[quote_pos..])
            } else {
                ("0", remainder)
            };

            let value = remainder.trim();
            
            output.push_str(&format!("{}{}:{}: {}\n", current_indent, key, version, value));
        } else {
            // Not a valid entry line, keep as is but trimmed
            output.push_str(&current_indent);
            output.push_str(trimmed);
            output.push('\n');
        }
    }

    output
}
