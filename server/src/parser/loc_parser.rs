use crate::data::interner::InternedStr;
use crate::data::layered_value::LayeredValue;
use crate::parser::ast::{DiagnosticSeverity, Range};
use crate::utils::line_index::LineIndex;
use dashmap::DashMap;
use nom::{
    IResult, Parser,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{char, digit1, multispace0, space0},
    combinator::opt,
};
use nom_locate::LocatedSpan;
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug, Clone)]
pub struct LocEntry {
    pub key: InternedStr,
    pub value: String,
    pub range: Range,
    pub path: InternedStr,
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
    pub related_information: Vec<crate::parser::ast::DiagnosticRelatedInformation>,
    pub tags: Vec<crate::parser::ast::DiagnosticTag>,
}

fn to_range(span: Span) -> Range {
    let start_line = span.location_line() - 1;
    let start_col = span.get_column() as u32 - 1;
    Range {
        start_line,
        start_col,
        end_line: start_line,
        end_col: start_col + span.fragment().chars().count() as u32,
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
        let mut prev_char = '\0';
        for (i, c) in line_no_comment.char_indices() {
            if c == '"' && prev_char != '\\' {
                unescaped_quote_indices.push(i);
            }
            prev_char = c;
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

static RE_SCOPE: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"\[([^\]]+)\]").unwrap());
static RE_COLOR: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"§([a-zA-Z0-9!])").unwrap());
static RE_NESTED: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"\$([^\$]+)\$").unwrap());

pub fn validate_loc_string(
    entry: &LocEntry,
    event_targets: &DashMap<InternedStr, Vec<crate::scanner::variable_scanner::EventTarget>>,
    scripted_locs: &DashMap<
        InternedStr,
        LayeredValue<crate::scanner::scripted_loc_scanner::ScriptedLoc>,
    >,
    color_codes: &HashSet<String>,
    country_tags: &HashSet<String>,
) -> Vec<LocDiagnostic> {
    let mut diagnostics = Vec::new();

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

    // Precompute a byte→UTF-16 index for this entry's value so that the
    // many conversions below are O(1) each instead of O(N).
    let line_idx = LineIndex::new(&entry.value);

    // 1. Validate Scopes [Root.GetTag], Variables [?var], Formatters [idea_name|idea_id], etc.
    for cap in RE_SCOPE.captures_iter(&entry.value) {
        let full_match = cap.get(0).unwrap();
        let mut inner = cap.get(1).unwrap().as_str();
        let start_pos = full_match.start();

        // Warn about backslash before bracket — HOI4 does not recognize \[ as an escape
        if start_pos > 0 {
            let preceding_char = entry.value[..start_pos].chars().last();
            if preceding_char == Some('\\') {
                let range = Range {
                    start_line: entry.range.start_line,
                    start_col: entry.value_start_col + line_idx.byte_to_utf16(start_pos),
                    end_line: entry.range.start_line,
                    end_col: entry.value_start_col + line_idx.byte_to_utf16(start_pos) + 1,
                };
                diagnostics.push(LocDiagnostic {
                    range,
                    message: "Backslash-escaped square bracket (\\) is not valid in HOI4. The game will treat \\ as an illegal break character and still attempt to parse [..]. Remove the backslash.".to_string(),
                    severity: DiagnosticSeverity::Warning,
                    code: Some("escaped_bracket".to_string()),
                    related_information: Vec::new(),
                    tags: Vec::new(),
                });
            }
        }

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
        if let Some(var_inner) = inner.strip_prefix('?') {
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
                                + line_idx.byte_to_utf16(start_pos)
                                + 2
                                + pipe_pos as u32
                                + formatting.find(c).unwrap_or(0) as u32,
                            end_line: entry.range.start_line,
                            end_col: entry.value_start_col
                                + line_idx.byte_to_utf16(start_pos)
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
                    start_col: entry.value_start_col + line_idx.byte_to_utf16(start_pos) + 1,
                    end_line: entry.range.start_line,
                    end_col: entry.value_start_col
                        + line_idx.byte_to_utf16(start_pos)
                        + 1
                        + formatter.len() as u32,
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
        let mut all_parts_invalid = true;

        for part in &parts {
            let mut valid = false;
            let part_upper = part.to_uppercase();

            // Check if this part is a known loc_command (valid in any position
            // — some commands like GetLeader, GetCapital act as scope transitions)
            let is_loc_command = loc_commands.iter().any(|&c| c.eq_ignore_ascii_case(part));

            // Check if this part is a country tag like GER, USA, SOV
            let is_country_tag = crate::scanner::country_scanner::is_valid_tag(&part_upper)
                && (country_tags.is_empty() || country_tags.contains(&part_upper));

            // Check if this part is a known scope keyword, event target, scripted loc, or numeric (state ID)
            let is_scope_or_target = scopes.contains(&part_upper.as_str())
                || event_targets.contains_key(*part)
                || scripted_locs.contains_key(*part)
                || part.chars().all(|c| c.is_ascii_digit());

            if is_loc_command || is_scope_or_target || is_country_tag {
                valid = true;
            }

            if !valid {
                let range = Range {
                    start_line: entry.range.start_line,
                    start_col: entry.value_start_col + line_idx.byte_to_utf16(current_part_start),
                    end_line: entry.range.start_line,
                    end_col: entry.value_start_col
                        + line_idx.byte_to_utf16(current_part_start)
                        + part.len() as u32,
                };
                diagnostics.push(LocDiagnostic {
                    range,
                    message: format!(
                        "Unrecognized '{}' scope or command in square brackets. If you intended literal text, note that HOI4 does not support backslash-escaping — use a different approach (e.g., concatenation or rewriting).",
                        part,
                    ),
                    severity: DiagnosticSeverity::Information,
                    code: Some("invalid_loc_scope".to_string()),
                    related_information: Vec::new(),
                    tags: Vec::new(),
                });
            } else {
                all_parts_invalid = false;
            }
            current_part_start += part.len() + 1; // +1 for .
        }

        if all_parts_invalid {
            diagnostics.push(LocDiagnostic {
                range: Range {
                    start_line: entry.range.start_line,
                    start_col: entry.value_start_col + line_idx.byte_to_utf16(start_pos),
                    end_line: entry.range.start_line,
                    end_col: entry.value_start_col + line_idx.byte_to_utf16(start_pos) + 2 + inner.len() as u32,
                },
                message: "Entirely unrecognized bracket content. Backslash-escaping is not valid in HOI4; remove the brackets or rewrite the text.".to_string(),
                severity: DiagnosticSeverity::Hint,
                code: Some("unescaped_bracket".to_string()),
                related_information: Vec::new(),
                tags: Vec::new(),
            });
        }
    }

    // 2. Validate Nested Keys $key$
    for cap in RE_NESTED.captures_iter(&entry.value) {
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
                            + line_idx.byte_to_utf16(cap.get(0).unwrap().start())
                            + 1
                            + pipe_pos as u32
                            + 1
                            + formatting.find(c).unwrap_or(0) as u32,
                        end_line: entry.range.start_line,
                        end_col: entry.value_start_col
                            + line_idx.byte_to_utf16(cap.get(0).unwrap().start())
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

    // 3. Validate Color Codes §Y...§!
    // NOTE: HOI4 color codes are flat/replacement-based, not nested.
    // Each new color code replaces the previous one, and §! resets to default.
    // A color code is only "unclosed" if it's the last one before end-of-string
    // with no §! after it.
    let mut open_color: Option<(String, usize)> = None;
    for cap in RE_COLOR.captures_iter(&entry.value) {
        let m = cap.get(0).unwrap();
        let code = cap.get(1).unwrap().as_str();
        let pos = m.start();

        if code == "!" {
            if open_color.is_none() {
                let range = Range {
                    start_line: entry.range.start_line,
                    start_col: entry.value_start_col + line_idx.byte_to_utf16(pos),
                    end_line: entry.range.start_line,
                    end_col: entry.value_start_col + line_idx.byte_to_utf16(pos) + 2,
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
                open_color = None;
            }
        } else {
            if !color_codes.contains(code) {
                let range = Range {
                    start_line: entry.range.start_line,
                    start_col: entry.value_start_col + line_idx.byte_to_utf16(pos),
                    end_line: entry.range.start_line,
                    end_col: entry.value_start_col + line_idx.byte_to_utf16(pos) + 2,
                };
                diagnostics.push(LocDiagnostic {
                    range,
                    message: format!(
                        "Unknown color code '§{}'. This will cause a 'Could not find coloring for character' error in-game.",
                        code
                    ),
                    severity: DiagnosticSeverity::Warning,
                    code: Some("unknown_color_code".to_string()),
                    related_information: Vec::new(),
                    tags: Vec::new(),
                });
            }
            // Any new color code replaces the previous one — they don't nest
            open_color = Some((code.to_string(), pos));
        }
    }

    if let Some((code, pos)) = open_color {
        let range = Range {
            start_line: entry.range.start_line,
            start_col: entry.value_start_col + line_idx.byte_to_utf16(pos),
            end_line: entry.range.start_line,
            end_col: entry.value_start_col + line_idx.byte_to_utf16(pos) + 2,
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

pub fn check_unnecessary_version(entry: &LocEntry) -> Option<LocDiagnostic> {
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
            tags: vec![crate::parser::ast::DiagnosticTag::Unnecessary],
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

pub fn parse_loc_file(
    input: &str,
    path: &str,
) -> (
    HashMap<InternedStr, LocEntry>,
    Vec<LocDiagnostic>,
    Option<String>,
) {
    let mut map: HashMap<InternedStr, LocEntry> = HashMap::new();
    let mut diagnostics = validate_loc_file_structure(input);
    diagnostics.extend(validate_unescaped_quotes_in_file(input));

    let input_clean = input.strip_prefix('\u{feff}').unwrap_or(input);
    let span = Span::new(input_clean);

    let mut current = span;
    let mut header_found = false;
    let mut language = None;

    // Fast forward to the header while preserving the nom_locate line/column tracking
    while !current.fragment().is_empty() {
        let frag = current.fragment();
        let line_end = frag.find('\n').unwrap_or(frag.len());
        let line = &frag[..line_end];
        let trimmed = line.trim();

        if trimmed.starts_with("l_") && trimmed.contains(':') {
            if let Ok((rem, lang)) =
                take_until::<&str, Span, nom::error::Error<Span>>(":").parse(current)
            {
                if let Ok((rem2, _)) = tag::<&str, Span, nom::error::Error<Span>>(":")(rem) {
                    current = rem2;
                    header_found = true;
                    language = Some(lang.fragment().trim().to_string());
                    break;
                }
            }
        }

        let advance = if line_end < frag.len() {
            line_end + 1
        } else {
            frag.len()
        };
        if advance == 0 {
            break;
        }

        match nom::bytes::complete::take::<usize, Span, nom::error::Error<Span>>(advance)
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
        return (map, diagnostics, language);
    }

    while !current.fragment().is_empty() {
        match parse_loc_entry(current, path) {
            Ok((remainder, entry)) => {
                map.insert(entry.key.clone(), entry);
                current = remainder;
            }
            Err(_) => {
                match nom::character::complete::not_line_ending::<Span, nom::error::Error<Span>>
                    .parse(current)
                {
                    Ok((rem, _)) => {
                        match nom::character::complete::line_ending::<Span, nom::error::Error<Span>>
                            .parse(rem)
                        {
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

    (map, diagnostics, language)
}

fn parse_loc_entry<'a>(input: Span<'a>, path: &'a str) -> IResult<Span<'a>, LocEntry> {
    let (input, _) = multispace0(input)?;
    let (input, key_span) =
        take_while1(|c: char| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-')
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
            key: {
                let key_s: &str = key_span.fragment();
                std::sync::Arc::from(key_s)
            },
            value,
            range: to_range(key_span),
            path: std::sync::Arc::from(path),
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
                    if let Some(base) = key.strip_suffix(var) {
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
