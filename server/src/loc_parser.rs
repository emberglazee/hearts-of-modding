use crate::ast::{DiagnosticSeverity, Range};
use nom::{
    IResult, Parser,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{char, digit1, multispace0, space0},
    combinator::opt,
};
use nom_locate::LocatedSpan;
use std::collections::HashMap;

type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug, Clone)]
pub struct LocEntry {
    pub key: String,
    pub value: String,
    pub range: Range,
    pub path: String,
    pub value_start_col: u32,
    pub version: Option<String>,
    pub version_range: Option<Range>,
}

#[derive(Debug, Clone)]
pub struct LocDiagnostic {
    pub range: Range,
    pub message: String,
    pub severity: DiagnosticSeverity,
    pub code: Option<String>,
    pub related_information: Vec<crate::ast::DiagnosticRelatedInformation>,
    pub tags: Vec<crate::ast::DiagnosticTag>,
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

pub fn validate_unescaped_quotes_in_file(input: &str) -> Vec<LocDiagnostic> {
    let mut diagnostics = Vec::new();
    let content_without_bom = input.strip_prefix('\u{feff}').unwrap_or(input);

    for (line_idx, line) in content_without_bom.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Find the start of the value (after first quote) and end (before last quote, excluding comments)
        let comment_start = line.find('#').unwrap_or(line.len());
        let line_no_comment = &line[..comment_start];

        let mut unescaped_quote_indices = Vec::new();
        let chars: Vec<char> = line_no_comment.chars().collect();
        for i in 0..chars.len() {
            if chars[i] == '"' {
                let is_escaped = i > 0 && chars[i - 1] == '\\';
                if !is_escaped {
                    unescaped_quote_indices.push(i);
                }
            }
        }

        // In HOI4 loc, we expect exactly 2 unescaped quotes (start and end)
        // If there are more, the ones in the middle are errors.
        if unescaped_quote_indices.len() > 2 {
            let _first_quote = unescaped_quote_indices[0];
            let _last_quote = *unescaped_quote_indices.last().unwrap();

            for &idx in &unescaped_quote_indices[1..unescaped_quote_indices.len() - 1] {
                let range = Range {
                    start_line: line_idx as u32,
                    start_col: idx as u32,
                    end_line: line_idx as u32,
                    end_col: idx as u32 + 1,
                };
                diagnostics.push(LocDiagnostic {
                    range,
                    message: "Internal unescaped double quote. Use \\\" instead to prevent parsing errors.".to_string(),
                    severity: DiagnosticSeverity::Warning,
                    code: Some("unescaped_quote".to_string()),
                    related_information: Vec::new(),
                    tags: Vec::new(),
                });
            }
        }
    }
    diagnostics
}

pub fn validate_loc_string(
    entry: &LocEntry,
    event_targets: &HashMap<String, Vec<crate::variable_scanner::EventTarget>>,
    scripted_locs: &HashMap<String, crate::scripted_loc_scanner::ScriptedLoc>,
) -> Vec<LocDiagnostic> {
    let mut diagnostics = Vec::new();

    let re_scope = regex::Regex::new(r"\[([^\]]+)\]").unwrap();
    let re_color = regex::Regex::new(r"§([a-zA-Z0-9!])").unwrap();
    let re_nested = regex::Regex::new(r"\$([^\$]+)\$").unwrap();

    let loc_commands = [
        "GetName",
        "GetNameDef",
        "GetNameDefCap",
        "GetAdjective",
        "GetAdjectiveCap",
        "GetTag",
        "GetRulingIdeology",
        "GetRulingIdeologyNoun",
        "GetPartyName",
        "GetPartySupport",
        "GetLeaderName",
        "GetLeaderNameDef",
        "GetPlayerName",
        "GetCapitalName",
        "GetLastElection",
        "GetRulingParty",
        "GetRulingPartyLong",
        "GetCommunistParty",
        "GetDemocraticParty",
        "GetFascistParty",
        "GetNeutralParty",
        "GetCommunistLeader",
        "GetDemocraticLeader",
        "GetFascistLeader",
        "GetNeutralLeader",
        "GetPowerBalanceName",
        "GetPowerBalanceModDesc",
        "GetRightSideName",
        "GetLeftSideName",
        "GetActiveSideName",
        "GetTrendingSideName",
        "GetActiveRangeName",
        "GetActiveRangeModDesc",
        "GetActiveRangeRuleDesc",
        "GetActiveRangeActivationEffect",
        "GetActiveRangeDeactivationEffect",
        "GetChangeRateDesc",
        "GetBopTrendTextIcon",
        "GetSheHe",
        "GetSheHeCap",
        "GetHerHim",
        "GetHerHimCap",
        "GetHerHis",
        "GetHerHisCap",
        "GetHersHis",
        "GetHersHisCap",
        "GetHerselfHimself",
        "GetHerselfHimselfCap",
        "GetIdeology",
        "GetIdeologyGroup",
        "GetRank",
        "GetCodeName",
        "GetCallsign",
        "GetSurname",
        "GetFullName",
        "GetWing",
        "GetWingShort",
        "GetAceType",
        "GetMissionRegion",
        "GetTokenKey",
        "GetTokenLocalizedKey",
        "GetDateString",
        "GetDateStringShortMonth",
        "GetDateStringNoHour",
        "GetDateStringNoHourLong",
        "GetManpower",
        "GetFactionName",
        "GetAgency",
        "GetNameWithFlag",
        "GetFlag",
        "GetDate",
        "GetTime",
        "GetYear",
        "GetMonth",
        "GetDay",
        "GetID",
        "GetCapitalVictoryPointName",
        "GetOldName",
        "GetOldNameDef",
        "GetOldNameDefCap",
        "GetOldAdjective",
        "GetOldAdjectiveCap",
        "GetNonIdeologyName",
        "GetNonIdeologyNameDef",
        "GetNonIdeologyNameDefCap",
        "GetNonIdeologyAdjective",
        "GetNonIdeologyAdjectiveCap",
        "GetLeader",
        "GetDateText",
    ];
    let scopes = [
        "ROOT",
        "FROM",
        "PREV",
        "THIS",
        "COUNTRY",
        "STATE",
        "UNIT",
        "CHARACTER",
        "GLOBAL",
        "Owner",
        "Controller",
        "Capital",
        "Leader",
        // Contextual Objects (Patch 1.15+)
        "Ace",
        "Building",
        "IndustrialOrg",
        "Operation",
        "Province",
        "PurchaseContract",
        "SpecialProject",
        "Terrain",
        "UnitLeader",
    ];
    let formatters = [
        "character_name",
        "country_culture",
        "idea_name",
        "advisor_desc",
        "tech_effect",
        "idea_desc",
        "building_state_modifier",
    ];

    // 1. Validate Scopes [Root.GetTag], Variables [?var], Formatters [idea_name|idea_id], etc.
    for cap in re_scope.captures_iter(&entry.value) {
        let full_match = cap.get(0).unwrap();
        let mut inner = cap.get(1).unwrap().as_str();
        let start_pos = full_match.start();

        // Handle ternary conditions: [(OBJECT ? TRUE_CASE : FALSE_CASE)]
        if inner.starts_with('(') && inner.ends_with(')') {
            inner = &inner[1..inner.len() - 1];
            // Split at the ternary ? but handle optional chaining style OBJECT?.PROPERTY
            if let Some(q_pos) = inner.find('?') {
                let obj = inner[..q_pos].trim();
                let remainder = &inner[q_pos + 1..].trim();

                if let Some(c_pos) = remainder.find(':') {
                    // Valid conditional, handle the "." efficiency shortcut
                    let true_case = &remainder[..c_pos].trim();
                    if true_case.starts_with('.') {
                        // Efficiency shortcut used: OBJECT?.PROPERTY
                        inner = obj;
                    } else {
                        inner = true_case;
                    }
                } else {
                    inner = obj;
                }
            }
        }

        // Handle variables [?var|formatting]
        if inner.starts_with('?') {
            let var_inner = &inner[1..];
            if let Some(pipe_pos) = var_inner.find('|') {
                let formatting = &var_inner[pipe_pos + 1..];
                // Validate formatting codes: *, ^, =, 0..9, %, %%, +, -, or color chars
                for c in formatting.chars() {
                    if !c.is_ascii_digit()
                        && !"*^=%+-.".contains(c)
                        && !c.is_ascii_alphabetic()
                        && c != '!'
                    {
                        let range = Range {
                            start_line: entry.range.start_line,
                            start_col: entry.value_start_col
                                + start_pos as u32
                                + 2
                                + pipe_pos as u32
                                + formatting.find(c).unwrap_or(0) as u32,
                            end_line: entry.range.start_line,
                            end_col: entry.value_start_col
                                + start_pos as u32
                                + 2
                                + pipe_pos as u32
                                + formatting.find(c).unwrap_or(0) as u32
                                + 1,
                        };
                        diagnostics.push(LocDiagnostic {
                            range,
                            message: format!("Invalid variable formatting code: '{}'", c),
                            severity: DiagnosticSeverity::Warning,
                            code: Some("invalid_var_format".to_string()),
                            related_information: Vec::new(),
                            tags: Vec::new(),
                        });
                    }
                }
            }
            continue;
        }

        // Handle localization formatters [formatter|token]
        if let Some(pipe_pos) = inner.find('|') {
            let formatter = &inner[..pipe_pos];
            if !formatters.contains(&formatter) {
                let range = Range {
                    start_line: entry.range.start_line,
                    start_col: entry.value_start_col + start_pos as u32 + 1,
                    end_line: entry.range.start_line,
                    end_col: entry.value_start_col + start_pos as u32 + 1 + formatter.len() as u32,
                };
                diagnostics.push(LocDiagnostic {
                    range,
                    message: format!("Unknown localization formatter: '{}'", formatter),
                    severity: DiagnosticSeverity::Warning,
                    code: Some("unknown_loc_formatter".to_string()),
                    related_information: Vec::new(),
                    tags: Vec::new(),
                });
            }
            continue;
        }

        let parts: Vec<&str> = inner.split('.').collect();
        let mut current_part_start = start_pos + 1; // +1 for [

        for (i, part) in parts.iter().enumerate() {
            let is_last = i == parts.len() - 1;
            let mut valid = false;
            let part_upper = part.to_uppercase();

            if is_last {
                if loc_commands
                    .iter()
                    .any(|&c| c.to_lowercase() == part.to_lowercase())
                    || scopes.contains(&part_upper.as_str())
                    || event_targets.contains_key(*part)
                    || scripted_locs.contains_key(*part)
                    || part.chars().all(|c| c.is_ascii_digit())
                {
                    // Allow numbers as scopes (state IDs)
                    valid = true;
                }
            } else {
                if scopes.contains(&part_upper.as_str())
                    || event_targets.contains_key(*part)
                    || scripted_locs.contains_key(*part)
                    || part.chars().all(|c| c.is_ascii_digit())
                {
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
                    message: format!(
                        "Potential invalid localization scope or command: '{}'",
                        part
                    ),
                    severity: DiagnosticSeverity::Warning,
                    code: Some("invalid_loc_scope".to_string()),
                    related_information: Vec::new(),
                    tags: Vec::new(),
                });
            }
            current_part_start += part.len() + 1; // +1 for .
        }
    }

    // 2. Validate Nested Keys $key$
    for cap in re_nested.captures_iter(&entry.value) {
        let inner = cap.get(1).unwrap().as_str();

        if inner.is_empty() {
            // Probably a literal $$
            continue;
        }

        // Handle variable formatting inside $key|formatting$
        if let Some(pipe_pos) = inner.find('|') {
            let formatting = &inner[pipe_pos + 1..];
            for c in formatting.chars() {
                if !c.is_ascii_digit()
                    && !"*^=%+-.".contains(c)
                    && !c.is_ascii_alphabetic()
                    && c != '!'
                {
                    let range = Range {
                        start_line: entry.range.start_line,
                        start_col: entry.value_start_col
                            + cap.get(0).unwrap().start() as u32
                            + 1
                            + pipe_pos as u32
                            + 1
                            + formatting.find(c).unwrap_or(0) as u32,
                        end_line: entry.range.start_line,
                        end_col: entry.value_start_col
                            + cap.get(0).unwrap().start() as u32
                            + 1
                            + pipe_pos as u32
                            + 1
                            + formatting.find(c).unwrap_or(0) as u32
                            + 1,
                    };
                    diagnostics.push(LocDiagnostic {
                        range,
                        message: format!("Invalid variable formatting code: '{}'", c),
                        severity: DiagnosticSeverity::Warning,
                        code: Some("invalid_var_format".to_string()),
                        related_information: Vec::new(),
                        tags: Vec::new(),
                    });
                }
            }
        }
    }

    // 3. Validate Text Icons £icon_name|frame
    let _re_icon = regex::Regex::new(r"£([a-zA-Z0-9_]+)(?:\|[0-9]+)?").unwrap();
    // (We could validate icon existence if we had sprite data here, but for now just ensure syntax is OK)

    // 3. Validate Color Codes §Y...§!
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
                    message: "Dangling color reset symbol '§!' without an opening color code."
                        .to_string(),
                    severity: DiagnosticSeverity::Information,
                    code: Some("dangling_color_reset".to_string()),
                    related_information: Vec::new(),
                    tags: Vec::new(),
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
            message: format!(
                "Unclosed color code '§{}'. Expected a matching '§!' reset.",
                code
            ),
            severity: DiagnosticSeverity::Information,
            code: Some("unclosed_color_code".to_string()),
            related_information: Vec::new(),
            tags: Vec::new(),
        });
    }

    diagnostics
}

pub fn check_unnecessary_version(
    entry: &LocEntry,
) -> Option<LocDiagnostic> {
    // Only check if this entry has a version number
    if let (Some(version), Some(version_range)) = (&entry.version, &entry.version_range) {
        return Some(LocDiagnostic {
            range: version_range.clone(),
            message: format!(
                "Version number '{}' is unnecessary. Modders generally don't need version numbers as they are only used for Paradox's internal translation tracking and are ignored by the game engine.",
                version
            ),
            severity: DiagnosticSeverity::Hint,
            code: Some("unnecessary_version".to_string()),
            related_information: Vec::new(),
            tags: vec![crate::ast::DiagnosticTag::Unnecessary],
        });
    }

    None
}

pub fn validate_loc_file_structure(input: &str) -> Vec<LocDiagnostic> {
    let mut diagnostics = Vec::new();

    // Check for l_language header
    let valid_languages = [
        "l_english",
        "l_braz_por",
        "l_french",
        "l_german",
        "l_polish",
        "l_russian",
        "l_spanish",
        "l_japanese",
        "l_simp_chinese",
        "l_korean",
    ];
    let content_without_bom = input.strip_prefix('\u{feff}').unwrap_or(input);

    let mut header_found = false;
    let mut has_content = false;
    for (line_idx, line) in content_without_bom.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        has_content = true;

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
                    message: format!(
                        "Unknown Paradox language header: '{}'. Valid languages are: {}",
                        lang,
                        valid_languages.join(", ")
                    ),
                    severity: DiagnosticSeverity::Warning,
                    code: Some("unknown_language".to_string()),
                    related_information: Vec::new(),
                    tags: Vec::new(),
                });
                header_found = true; // Still found a header, just a weird one
            }
            break;
        }
    }

    if !header_found && has_content {
        diagnostics.push(LocDiagnostic {
            range: Range { start_line: 0, start_col: 0, end_line: 0, end_col: 10 },
            message: "Missing language header (e.g., 'l_english:'). Localization files must start with a supported language tag.".to_string(),
            severity: DiagnosticSeverity::Error,
            code: Some("missing_language_header".to_string()),
            related_information: Vec::new(),
            tags: Vec::new(),
        });
    }

    diagnostics
}

pub fn parse_loc_file(input: &str, path: &str) -> (HashMap<String, LocEntry>, Vec<LocDiagnostic>) {
    let mut map = HashMap::new();
    let mut diagnostics = validate_loc_file_structure(input);
    diagnostics.extend(validate_unescaped_quotes_in_file(input));

    let input_clean = input.strip_prefix('\u{feff}').unwrap_or(input);
    let span = Span::new(input_clean);

    let mut current = span;
    let mut header_found = false;

    // Fast forward to the header while preserving the nom_locate line/column tracking
    while !current.fragment().is_empty() {
        if current.fragment().starts_with("l_") && current.fragment().contains(':') {
            if let Ok((rem, _)) =
                take_until::<&str, Span, nom::error::Error<Span>>(":").parse(current)
            {
                if let Ok((rem2, _)) = tag::<&str, Span, nom::error::Error<Span>>(":")(rem) {
                    current = rem2;
                    header_found = true;
                    break;
                }
            }
        }

        match nom::bytes::complete::take::<usize, Span, nom::error::Error<Span>>(1usize)
            .parse(current)
        {
            Ok((rem, _)) => {
                current = rem;
            }
            _ => {
                break;
            }
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
                match nom::character::complete::not_line_ending::<Span, nom::error::Error<Span>>.parse(current) {
                    Ok((rem, _)) => {
                        match nom::character::complete::line_ending::<Span, nom::error::Error<Span>>.parse(rem) {
                            Ok((rem2, _)) => current = rem2,
                            Err(_) => break,
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        }
    }

    (map, diagnostics)
}

fn parse_loc_entry<'a>(input: Span<'a>, path: &'a str) -> IResult<Span<'a>, LocEntry> {
    let (input, _) = multispace0(input)?;
    let (input, key_span) =
        take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '.' || c == '-')
            .parse(input)?;
    let (input, _) = char(':').parse(input)?;
    let (input, version_span) = opt(digit1).parse(input)?;
    let (input, _) = space0(input)?;

    let (input, _) = char('"').parse(input)?;
    let start_val = input;

    // Parse string content, handling escaped quotes
    let mut value = String::new();
    let mut current = input;

    loop {
        // Try to find a quote
        match take_until::<&str, Span, nom::error::Error<Span>>("\"").parse(current) {
            Ok((after_quote, before_quote)) => {
                value.push_str(before_quote.fragment());

                // Check if the quote is escaped
                if value.ends_with('\\') {
                    // It's an escaped quote, include it and continue
                    value.push('"');
                    current = after_quote;
                    // Skip the quote character
                    match char::<Span, nom::error::Error<Span>>('"').parse(current) {
                        Ok((after, _)) => {
                            current = after;
                        }
                        _ => {
                            break;
                        }
                    }
                } else {
                    // It's the closing quote
                    current = after_quote;
                    break;
                }
            }
            _ => {
                // No quote found, this is an error
                return Err(nom::Err::Error(nom::error::Error::new(
                    current,
                    nom::error::ErrorKind::Char,
                )));
            }
        }
    }

    let (input, _) = char('"').parse(current)?;

    let (version, version_range) = if let Some(v_span) = version_span {
        (Some(v_span.fragment().to_string()), Some(to_range(v_span)))
    } else {
        (None, None)
    };

    Ok((
        input,
        LocEntry {
            key: key_span.fragment().to_string(),
            value,
            range: to_range(key_span),
            path: path.to_string(),
            value_start_col: start_val.get_column() as u32 - 1,
            version,
            version_range,
        },
    ))
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

            output.push_str(&format!(
                "{}{}:{}: {}\n",
                current_indent, key, version, value
            ));
        } else {
            // Not a valid entry line, keep as is but trimmed
            output.push_str(&current_indent);
            output.push_str(trimmed);
            output.push('\n');
        }
    }

    output
}
