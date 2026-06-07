use crate::data::entity_lookup::EntityKind;
// Note: LineIndex is not used here — per-line byte→UTF-16 conversion
// uses a zero-allocation char walk instead (see byte_to_col closure).
use std::collections::{HashMap, HashSet};
use tower_lsp_server::ls_types::{SemanticToken, SemanticTokens, SemanticTokensResult};

/// Indices into the LSP semantic token legend registered in lsp_handler.rs.
/// Must match the order of token_types in the legend vec.
#[repr(u32)]
enum TokenType {
    Keyword = 0,
    Variable = 1,
    String = 2,
    Number = 3,
    Operator = 4,
    Comment = 5,
    Type = 6,
    Event = 7,
    Function = 8,
    Enum = 9,
    EnumMember = 10,
    Struct = 11,
    Class = 12,
    Property = 13,
    EscapeCharacter = 14,
    Parameter = 15,
}

/// Fields whose values are always localization keys, not entity references.
/// Values under these keys skip entity-type semantic highlighting.
const LOCALIZATION_VALUE_FIELDS: [&str; 4] = ["name", "desc", "custom_description", "text"];

/// Context struct that replaces the 18-parameter threading pattern.
/// Carries all data needed to resolve token types for a document.
pub struct SemanticTokenContext {
    pub keywords: HashSet<String>,
    pub entity_names: HashMap<String, EntityKind>,
}

impl SemanticTokenContext {
    pub fn new(keywords: HashSet<String>, entity_names: HashMap<String, EntityKind>) -> Self {
        SemanticTokenContext {
            keywords,
            entity_names,
        }
    }
}

/// Map an entity kind to its semantic token type index.
/// Each entity kind gets a distinct type so the theme can color them differently.
fn entity_kind_to_token_type(kind: EntityKind) -> u32 {
    match kind {
        // Callable/behavioural constructs → Function
        EntityKind::ScriptedTrigger
        | EntityKind::ScriptedEffect
        | EntityKind::ScriptedLoc
        | EntityKind::Ability
        | EntityKind::AdjacencyRule => TokenType::Function as u32,

        // Named categories → Enum
        EntityKind::Ideology | EntityKind::SoundCategory | EntityKind::StateCategory => {
            TokenType::Enum as u32
        }

        // Members of named categories → EnumMember
        EntityKind::SubIdeology | EntityKind::ColorCode | EntityKind::Resource => {
            TokenType::EnumMember as u32
        }

        // Data structures → Struct
        EntityKind::Trait | EntityKind::Character | EntityKind::Building => {
            TokenType::Struct as u32
        }

        // Named concept definitions → Class
        EntityKind::Idea | EntityKind::AiArea | EntityKind::AiStrategyPlan => {
            TokenType::Class as u32
        }

        // Narrative / event-like → Event
        EntityKind::Event | EntityKind::Focus | EntityKind::Achievement => TokenType::Event as u32,

        // Asset references → Property
        EntityKind::Sprite
        | EntityKind::MusicAsset
        | EntityKind::MusicStation
        | EntityKind::Song
        | EntityKind::Sound
        | EntityKind::SoundEffect
        | EntityKind::Falloff
        | EntityKind::Portrait
        | EntityKind::CustomModifier
        | EntityKind::ModifierMapping => TokenType::Property as u32,

        // Identifiers → Type
        EntityKind::CountryTag
        | EntityKind::StrategicRegion
        | EntityKind::State
        | EntityKind::Province => TokenType::Type as u32,

        // Variables
        EntityKind::Variable | EntityKind::EventTarget => TokenType::Variable as u32,

        // Localization entries → String
        EntityKind::Localization => TokenType::String as u32,

        // Unit type names (infantry, engineer, etc.) → Type (teal/green)
        EntityKind::OobDivisionTemplate | EntityKind::OobFleet | EntityKind::UnitType => {
            TokenType::Type as u32
        }

        // Fallback for any unexpected kind
        _ => TokenType::Type as u32,
    }
}

pub fn get_semantic_tokens(script: &Script, ctx: &SemanticTokenContext) -> SemanticTokensResult {
    let mut tokens = Vec::new();
    for entry in &script.entries {
        push_entry_tokens(entry, &mut tokens, ctx, &script.source, None);
    }

    tokens_to_lsp(tokens)
}

/// Check whether an AST line/col range overlaps with an LSP range (line-only check).
/// Uses line comparison only — the client is typically requesting viewport-sized ranges
/// and columns would add complexity for negligible extra filtering.
fn ast_range_overlaps_lsp(
    ast_range: &Range,
    lsp_range: &tower_lsp_server::ls_types::Range,
) -> bool {
    !(ast_range.end_line < lsp_range.start.line || ast_range.start_line > lsp_range.end.line)
}

/// Get semantic tokens only for top-level entries that intersect with the requested range.
/// Non-overlapping entries are skipped entirely, avoiding unnecessary AST traversal.
/// This is the key optimization when VS Code requests tokens for the visible viewport
/// in a large file (e.g., 20,000-line events.txt).
pub fn get_semantic_tokens_range(
    script: &Script,
    ctx: &SemanticTokenContext,
    range: &tower_lsp_server::ls_types::Range,
) -> SemanticTokensResult {
    let mut tokens = Vec::new();
    for entry in &script.entries {
        // Determine the full range of the entry (key + value tree)
        let entry_range = match entry {
            Entry::Assignment(ass) => &ass.value.range,
            Entry::Value(val) => &val.range,
            Entry::Comment(_, cr) => cr,
        };

        // Skip entries entirely outside the requested range
        if !ast_range_overlaps_lsp(entry_range, range) {
            continue;
        }

        push_entry_tokens(entry, &mut tokens, ctx, &script.source, None);
    }

    tokens_to_lsp(tokens)
}

/// Convert a sorted Vec<RawToken> into the LSP delta-encoded format.
fn tokens_to_lsp(tokens: Vec<RawToken>) -> SemanticTokensResult {
    if tokens.is_empty() {
        return SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: vec![],
        });
    }

    let mut sorted = tokens;
    sorted.sort_by(|a, b| {
        if a.line != b.line {
            a.line.cmp(&b.line)
        } else {
            a.start.cmp(&b.start)
        }
    });

    let mut lsp_tokens = Vec::with_capacity(sorted.len());
    let mut last_line = 0;
    let mut last_start = 0;

    for token in sorted {
        let delta_line = token.line - last_line;
        let delta_start = if delta_line == 0 {
            token.start - last_start
        } else {
            token.start
        };

        lsp_tokens.push(SemanticToken {
            delta_line,
            delta_start,
            length: token.length,
            token_type: token.token_type,
            token_modifiers_bitset: 0,
        });

        last_line = token.line;
        last_start = token.start;
    }

    SemanticTokensResult::Tokens(SemanticTokens {
        result_id: None,
        data: lsp_tokens,
    })
}

// ── Locality patterns for .yml localization file highlighting ──

// All sub-string pattern scanning uses manual byte iteration below (10-50x faster than regex).

/// Valid language header prefixes in HOI4 localization files.
const LOC_LANGUAGE_PREFIXES: [&str; 10] = [
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

/// Produce semantic tokens for a .yml HOI4 localization file.
///
/// Highlights:
/// - Language header (`l_english:`) → Keyword + Operator
/// - Comments → Comment
/// - Localization keys → Type
/// - Version numbers → Number
/// - Colons and delimiters → Operator
/// - Quote characters → String
/// - Plain text values → String
/// - Color codes (§G, §R, §! etc.) → Operator (rendered by decorations)
/// - Scope references ([ROOT.GetName]) → Function
/// - Nested key refs ($key$) → Variable
/// - Escape sequences (\n, \") → EscapeCharacter
pub fn loc_semantic_tokens(content: &str) -> SemanticTokensResult {
    let mut tokens = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        let line_u32 = line_idx as u32;
        let line_len = line.len() as u32;

        if line.is_empty() {
            continue;
        }

        let trimmed_start = line.trim_start();
        let trim_col = (line.len() - trimmed_start.len()) as u32;

        // ── Comment lines ──
        if trimmed_start.starts_with('#') {
            let hash_col = line.find('#').unwrap() as u32;
            tokens.push(RawToken {
                line: line_u32,
                start: hash_col,
                length: line_len - hash_col,
                token_type: TokenType::Comment as u32,
            });
            continue;
        }

        // ── Language header (e.g. "l_english:") ──
        if LOC_LANGUAGE_PREFIXES
            .iter()
            .any(|p| trimmed_start.starts_with(p))
            && trimmed_start.contains(':')
        {
            let colon_col = line.find(':').unwrap() as u32;
            tokens.push(RawToken {
                line: line_u32,
                start: trim_col,
                length: colon_col - trim_col,
                token_type: TokenType::Keyword as u32,
            });
            tokens.push(RawToken {
                line: line_u32,
                start: colon_col,
                length: 1,
                token_type: TokenType::Operator as u32,
            });
            continue;
        }

        // ── Localization entry: <key>:<version> "<value>" ──
        // Find the first colon to split key from the rest
        if let Some(first_colon) = line.find(':') {
            let first_colon_u = first_colon as u32;

            // Key token (from first non-whitespace to colon)
            let key_start = line.find(|c: char| !c.is_whitespace()).unwrap_or(0) as u32;
            if first_colon_u > key_start {
                tokens.push(RawToken {
                    line: line_u32,
                    start: key_start,
                    length: first_colon_u - key_start,
                    token_type: TokenType::Type as u32,
                });
            }

            // First colon operator
            tokens.push(RawToken {
                line: line_u32,
                start: first_colon_u,
                length: 1,
                token_type: TokenType::Operator as u32,
            });

            // Parse version (digits after first colon)
            let after_colon = &line[first_colon + 1..];
            let digit_end = after_colon
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(after_colon.len());

            if digit_end > 0 {
                tokens.push(RawToken {
                    line: line_u32,
                    start: first_colon_u + 1,
                    length: digit_end as u32,
                    token_type: TokenType::Number as u32,
                });
            }

            // Find the opening quote of the value
            let after_digits = &after_colon[digit_end..];
            let quote_offset = after_digits.find('"');
            let second_colon_pos = after_digits.find(':');

            // Handle optional second colon before the value (some editors add it)
            // e.g. "KEY:0: \"value\"" instead of "KEY:0 \"value\""
            let value_colon_offset = if let (Some(sc), Some(qo)) = (second_colon_pos, quote_offset)
            {
                if sc < qo {
                    // There's a colon before the quote, emit it as operator
                    let colon_abs = first_colon_u + 1u32 + digit_end as u32 + sc as u32;
                    tokens.push(RawToken {
                        line: line_u32,
                        start: colon_abs,
                        length: 1,
                        token_type: TokenType::Operator as u32,
                    });
                    Some(sc)
                } else {
                    None
                }
            } else {
                None
            };

            // Determine where the quoted value starts
            let search_start = if let Some(sc) = value_colon_offset {
                digit_end + sc + 1
            } else {
                digit_end
            };

            let remaining = &after_colon[search_start..];
            let quote_start = remaining.find('"');

            if let Some(qs) = quote_start {
                let abs_quote_col = first_colon_u + 1u32 + search_start as u32 + qs as u32;

                // Opening quote
                tokens.push(RawToken {
                    line: line_u32,
                    start: abs_quote_col,
                    length: 1,
                    token_type: TokenType::String as u32,
                });

                // Find closing quote (not escaped by backslash)
                // Track BYTE offset, not char offset — multibyte chars like §
                // would otherwise cause off-by-one errors.
                let value_start_byte = abs_quote_col as usize + 1;
                let rest_after_open = &line[value_start_byte..];
                let mut closing_rel = None;
                let mut byte_offset = 0;
                for c in rest_after_open.chars() {
                    if c == '"' {
                        // Check if escaped
                        if byte_offset > 0 && rest_after_open[..byte_offset].ends_with('\\') {
                            byte_offset += c.len_utf8();
                            continue;
                        }
                        closing_rel = Some(byte_offset);
                        break;
                    }
                    byte_offset += c.len_utf8();
                }

                if let Some(cq) = closing_rel {
                    let value_end_byte = value_start_byte + cq;

                    // Emit sub-tokens for the value content between quotes.
                    // All positions in the value region MUST be converted from
                    // byte offsets to UTF-16 code unit columns, because LSP
                    // uses UTF-16 (§ is 2 bytes but 1 code unit, etc.).
                    let value_content = &line[value_start_byte..value_end_byte];

                    // Convert a byte offset within value_content to a
                    // UTF-16 column in the full line via a zero-allocation
                    // char walk.  Lines in localisation files are short, and
                    // this avoids allocating a Vec<u32> per line (10,000 Vecs
                    // in a 10K-line file on every keystroke).
                    let byte_to_col = |rel_byte: usize| -> u32 {
                        line[..(value_start_byte + rel_byte)]
                            .chars()
                            .map(|c| c.len_utf16() as u32)
                            .sum()
                    };

                    // Collect all interesting ranges (color codes, scopes, nested keys)
                    // sorted by start position
                    let mut ranges: Vec<(usize, usize, u32)> = Vec::new();

                    // Color codes: § followed by [a-zA-Z0-9!] → scan for § (0xC2 0xA7 in UTF-8)
                    {
                        let bytes = value_content.as_bytes();
                        let mut i = 0;
                        while i + 2 < bytes.len() {
                            if bytes[i] == 0xC2 && bytes[i + 1] == 0xA7 {
                                let flag = bytes[i + 2];
                                if flag.is_ascii_alphanumeric() || flag == b'!' {
                                    ranges.push((i, i + 3, TokenType::Operator as u32));
                                }
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                    }

                    // Scope references [...] → scan for '[' and find matching ']'
                    {
                        let mut search_start = 0;
                        while let Some(open) = value_content[search_start..].find('[') {
                            let open_abs = search_start + open;
                            if let Some(close_rel) = value_content[open_abs + 1..].find(']') {
                                let close_abs = open_abs + 1 + close_rel;
                                ranges.push((open_abs, open_abs + 1, TokenType::Operator as u32));
                                if close_abs > open_abs + 1 {
                                    ranges.push((
                                        open_abs + 1,
                                        close_abs,
                                        TokenType::Function as u32,
                                    ));
                                }
                                ranges.push((close_abs, close_abs + 1, TokenType::Operator as u32));
                                search_start = close_abs + 1;
                            } else {
                                search_start = open_abs + 1;
                            }
                        }
                    }

                    // Nested key refs $...$ → scan for '$' and find closing '$'
                    {
                        let mut search_start = 0;
                        while let Some(dollar) = value_content[search_start..].find('$') {
                            let open_abs = search_start + dollar;
                            if let Some(close_rel) = value_content[open_abs + 1..].find('$') {
                                let close_abs = open_abs + 1 + close_rel;
                                ranges.push((open_abs, open_abs + 1, TokenType::Operator as u32));
                                if close_abs > open_abs + 1 {
                                    ranges.push((
                                        open_abs + 1,
                                        close_abs,
                                        TokenType::Variable as u32,
                                    ));
                                }
                                ranges.push((close_abs, close_abs + 1, TokenType::Operator as u32));
                                search_start = close_abs + 1;
                            } else {
                                search_start = open_abs + 1;
                            }
                        }
                    }

                    // Escape sequences (\n, \") → scan for backslash
                    {
                        let bytes = value_content.as_bytes();
                        let mut i = 0;
                        while i + 1 < bytes.len() {
                            if bytes[i] == b'\\' {
                                let next = bytes[i + 1];
                                if next == b'n' || next == b'"' {
                                    ranges.push((i, i + 2, TokenType::EscapeCharacter as u32));
                                    i += 2;
                                    continue;
                                }
                            }
                            i += 1;
                        }
                    }

                    // Sort by start position
                    ranges.sort_by_key(|r| r.0);

                    // Fill gaps between ranges with String tokens
                    let mut pos = 0;
                    for (start, end, ttype) in &ranges {
                        if *start > pos {
                            tokens.push(RawToken {
                                line: line_u32,
                                start: byte_to_col(pos),
                                length: byte_to_col(*start) - byte_to_col(pos),
                                token_type: TokenType::String as u32,
                            });
                        }
                        tokens.push(RawToken {
                            line: line_u32,
                            start: byte_to_col(*start),
                            length: byte_to_col(*end) - byte_to_col(*start),
                            token_type: *ttype,
                        });
                        pos = *end;
                    }

                    // Trailing plain text after last special range
                    if pos < value_content.len() {
                        tokens.push(RawToken {
                            line: line_u32,
                            start: byte_to_col(pos),
                            length: byte_to_col(value_content.len()) - byte_to_col(pos),
                            token_type: TokenType::String as u32,
                        });
                    }

                    // Closing quote (byte_to_col handles the UTF-16 conversion)
                    tokens.push(RawToken {
                        line: line_u32,
                        start: byte_to_col(value_content.len()),
                        length: 1,
                        token_type: TokenType::String as u32,
                    });
                } else {
                    // No closing quote found — highlight as String from open to end
                    let remaining_len = line_len.saturating_sub(abs_quote_col);
                    tokens.push(RawToken {
                        line: line_u32,
                        start: abs_quote_col,
                        length: remaining_len,
                        token_type: TokenType::String as u32,
                    });
                }
            }
        }
    }

    tokens_to_lsp(tokens)
}

// ── AST-based token generation (for .txt script files) ──

fn push_entry_tokens(
    entry: &Entry,
    tokens: &mut Vec<RawToken>,
    ctx: &SemanticTokenContext,
    source: &str,
    parent_key: Option<&str>,
) {
    match entry {
        Entry::Assignment(ass) => {
            let key_text = ass.key_text(source);
            let is_keyword = ctx.keywords.contains(key_text);

            if is_keyword {
                tokens.push(RawToken {
                    line: ass.key_range.start_line,
                    start: ass.key_range.start_col,
                    length: ass.key_range.end_col - ass.key_range.start_col,
                    token_type: TokenType::Keyword as u32,
                });
            } else if let Some(kind) = ctx.entity_names.get(key_text) {
                tokens.push(RawToken {
                    line: ass.key_range.start_line,
                    start: ass.key_range.start_col,
                    length: ass.key_range.end_col - ass.key_range.start_col,
                    token_type: entity_kind_to_token_type(*kind),
                });
            } else {
                // Contextual checks based on parent key
                let is_idea_category =
                    parent_key.is_some_and(|p| p == "ideas" || p == "idea_categories");

                let is_portrait_category = parent_key.is_some_and(|p| p == "portraits")
                    && matches!(key_text, "civilian" | "army" | "navy");

                let is_portrait_size =
                    matches!(parent_key, Some("civilian") | Some("army") | Some("navy"))
                        && matches!(key_text, "large" | "small");

                if is_idea_category {
                    tokens.push(RawToken {
                        line: ass.key_range.start_line,
                        start: ass.key_range.start_col,
                        length: ass.key_range.end_col - ass.key_range.start_col,
                        token_type: TokenType::Type as u32,
                    });
                } else if is_portrait_category || is_portrait_size {
                    tokens.push(RawToken {
                        line: ass.key_range.start_line,
                        start: ass.key_range.start_col,
                        length: ass.key_range.end_col - ass.key_range.start_col,
                        token_type: TokenType::Keyword as u32,
                    });
                } else if matches!(key_text, "x" | "y") {
                    // Coordinate/position parameters used in division templates,
                    // .gui, .gfx, and other placement contexts
                    tokens.push(RawToken {
                        line: ass.key_range.start_line,
                        start: ass.key_range.start_col,
                        length: ass.key_range.end_col - ass.key_range.start_col,
                        token_type: TokenType::Parameter as u32,
                    });
                }
            }

            // Always emit operator token
            tokens.push(RawToken {
                line: ass.operator_range.start_line,
                start: ass.operator_range.start_col,
                length: ass.operator_range.end_col - ass.operator_range.start_col,
                token_type: TokenType::Operator as u32,
            });

            push_value_tokens(&ass.value, tokens, ctx, source, Some(key_text));
        }
        Entry::Value(val) => {
            push_value_tokens(val, tokens, ctx, source, parent_key);
        }
        Entry::Comment(_, range) => {
            tokens.push(RawToken {
                line: range.start_line,
                start: range.start_col,
                length: range.end_col - range.start_col,
                token_type: TokenType::Comment as u32,
            });
        }
    }
}

fn push_value_tokens(
    val: &NodeedValue,
    tokens: &mut Vec<RawToken>,
    ctx: &SemanticTokenContext,
    source: &str,
    parent_key: Option<&str>,
) {
    match &val.value {
        Value::String(span) => {
            let s = span.resolve(source);
            let is_localization_value =
                parent_key.is_some_and(|k| LOCALIZATION_VALUE_FIELDS.contains(&k));

            if ctx.keywords.contains(s) {
                tokens.push(RawToken {
                    line: val.range.start_line,
                    start: val.range.start_col,
                    length: val.range.end_col - val.range.start_col,
                    token_type: TokenType::Keyword as u32,
                });
            } else if !is_localization_value {
                if let Some(kind) = ctx.entity_names.get(s) {
                    tokens.push(RawToken {
                        line: val.range.start_line,
                        start: val.range.start_col,
                        length: val.range.end_col - val.range.start_col,
                        token_type: entity_kind_to_token_type(*kind),
                    });
                } else if s.starts_with("var:") || s.starts_with("temp_var:") {
                    tokens.push(RawToken {
                        line: val.range.start_line,
                        start: val.range.start_col,
                        length: val.range.end_col - val.range.start_col,
                        token_type: TokenType::Variable as u32,
                    });
                }
            }
        }
        Value::Number(_) => {
            tokens.push(RawToken {
                line: val.range.start_line,
                start: val.range.start_col,
                length: val.range.end_col - val.range.start_col,
                token_type: TokenType::Number as u32,
            });
        }
        Value::Boolean(_) => {
            tokens.push(RawToken {
                line: val.range.start_line,
                start: val.range.start_col,
                length: val.range.end_col - val.range.start_col,
                token_type: TokenType::Keyword as u32,
            });
        }
        Value::Block(entries) => {
            for entry in entries {
                push_entry_tokens(entry, tokens, ctx, source, parent_key);
            }
        }
        Value::TaggedBlock(tag, entries, _) => {
            tokens.push(RawToken {
                line: val.range.start_line,
                start: val.range.start_col,
                length: tag.len() as u32,
                token_type: TokenType::Keyword as u32,
            });
            for entry in entries {
                push_entry_tokens(entry, tokens, ctx, source, parent_key);
            }
        }
        Value::QuotedString(_) => {
            // Quoted string literals get String token type
            tokens.push(RawToken {
                line: val.range.start_line,
                start: val.range.start_col,
                length: val.range.end_col - val.range.start_col,
                token_type: TokenType::String as u32,
            });
        }
    }
}

// ── CSV file highlighting (definition.csv, adjacencies.csv) ──

/// Token type for a column in definition.csv (8‑column format).
/// Columns: Province ID;R;G;B;Type(land/sea/lake);Coastal(true/false);Terrain;Continent
fn definition_csv_col(col_idx: u32) -> u32 {
    match col_idx {
        0 | 1 | 2 | 3 | 7 => TokenType::Number as u32, // ID, R, G, B, continent
        4 | 5 => TokenType::Keyword as u32,            // type, coastal status
        6 => TokenType::Type as u32,                   // terrain
        _ => TokenType::String as u32,
    }
}

/// Token type for a column in adjacencies.csv (10‑column format).
/// Columns: From;To;Type;Through;start_x;start_y;stop_x;stop_y;adjacency_rule_name;comment
fn adjacency_csv_col(col_idx: u32) -> u32 {
    match col_idx {
        0 | 1 => TokenType::Number as u32, // start/end province IDs
        2 => TokenType::Keyword as u32,    // adjacency type
        3..=7 => TokenType::Number as u32, // through, x/y positions
        8 => TokenType::String as u32,     // adjacency rule name
        _ => TokenType::String as u32,
    }
}

/// Produce semantic tokens for a `.csv` HOI4 map file.
///
/// Two formats are recognised by column count:
/// - **8 columns** → `definition.csv` (province definitions)
/// - **10 columns** → `adjacencies.csv` (adjacency relations)
///
/// Comment lines (`# …`) are highlighted as Comment. Headers (first line
/// starting with `Province` / `From`) are fully Keyword. Data columns
/// are coloured positionally: numbers, keywords (type/coastal), and
/// terrain names (Type).
pub fn csv_semantic_tokens(content: &str) -> Option<SemanticTokensResult> {
    let mut tokens = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        let line_u32 = line_idx as u32;
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Comment lines starting with `#`
        if trimmed.starts_with('#') {
            if let Some(hash_pos) = line.find('#') {
                tokens.push(RawToken {
                    line: line_u32,
                    start: hash_pos as u32,
                    length: (line.len() - hash_pos) as u32,
                    token_type: TokenType::Comment as u32,
                });
            }
            continue;
        }

        // Detect header from the first column of the raw (untrimmed) line
        let is_header = {
            let first_semi = line.find(';');
            let first_col = match first_semi {
                Some(pos) => line[..pos].trim(),
                None => line.trim(),
            };
            let lowered = first_col.to_lowercase();
            lowered == "province" || lowered == "from"
        };

        // Count semicolons IN THE ORIGINAL LINE to determine column count.
        // This is the only reliable way to detect format (8-col vs 10-col)
        // because whitespace or leading/trailing semicolons affect trimming
        // but not the logical column structure.
        let semicolon_count = line.matches(';').count();

        // Minimum 1 semicolon (= 2 columns) to be a data line
        if semicolon_count == 0 {
            continue;
        }

        let is_adjacencies = semicolon_count >= 9; // 10 columns = 9 semicolons

        // Scan for semicolons in the *original* line to compute correct
        // column byte-offsets, so whitespace padding is handled correctly.
        let mut col_start = 0usize;
        let mut col_idx = 0u32;

        for (byte_pos, ch) in line.char_indices() {
            if ch != ';' {
                continue;
            }
            // Emit a token for the column that ends at this semicolon
            if col_start < byte_pos {
                if let Some(col_content) = line.get(col_start..byte_pos) {
                    let trimmed_val = col_content.trim();
                    if !trimmed_val.is_empty() {
                        let indent = col_content.len() - col_content.trim_start().len();
                        let content_start = col_start + indent;

                        let token_type = if is_header {
                            TokenType::Keyword as u32
                        } else if is_adjacencies {
                            adjacency_csv_col(col_idx)
                        } else {
                            definition_csv_col(col_idx)
                        };

                        tokens.push(RawToken {
                            line: line_u32,
                            start: content_start as u32,
                            length: trimmed_val.len() as u32,
                            token_type,
                        });
                    }
                }
            }
            col_start = byte_pos + 1;
            col_idx += 1;
        }

        // Last column (after the final semicolon)
        if col_start < line.len() {
            if let Some(col_content) = line.get(col_start..) {
                let trimmed_val = col_content.trim();
                if !trimmed_val.is_empty() {
                    let indent = col_content.len() - col_content.trim_start().len();
                    let content_start = col_start + indent;

                    let token_type = if is_header {
                        TokenType::Keyword as u32
                    } else if is_adjacencies {
                        adjacency_csv_col(col_idx)
                    } else {
                        definition_csv_col(col_idx)
                    };

                    tokens.push(RawToken {
                        line: line_u32,
                        start: content_start as u32,
                        length: trimmed_val.len() as u32,
                        token_type,
                    });
                }
            }
        }
    }

    // No tokens → caller returns None so TextMate grammar isn't overridden
    if tokens.is_empty() {
        return None;
    }

    Some(tokens_to_lsp(tokens))
}

struct RawToken {
    line: u32,
    start: u32,
    length: u32,
    token_type: u32,
}

// ── Re-exports for backwards compatibility ──

/// The Script type referenced in get_semantic_tokens signature.
use crate::parser::ast::*;

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: call loc_semantic_tokens and collect (line, start, length, token_type_str)
    fn collect_loc_tokens(content: &str) -> Vec<(u32, u32, u32, String)> {
        let result = loc_semantic_tokens(content);
        match result {
            SemanticTokensResult::Tokens(t) => {
                // Decode delta encoding back to absolute positions
                let legend = [
                    "keyword",
                    "variable",
                    "string",
                    "number",
                    "operator",
                    "comment",
                    "type",
                    "event",
                    "function",
                    "enum",
                    "enum_member",
                    "struct",
                    "class",
                    "property",
                ];
                let mut absolute: Vec<(u32, u32, u32, u32)> = Vec::new();
                let mut last_line = 0u32;
                let mut last_start = 0u32;
                for st in &t.data {
                    let line = last_line + st.delta_line;
                    let start = if st.delta_line == 0 {
                        last_start + st.delta_start
                    } else {
                        st.delta_start
                    };
                    absolute.push((line, start, st.length, st.token_type));
                    last_line = line;
                    last_start = start;
                }
                absolute
                    .into_iter()
                    .map(|(l, s, len, tt)| {
                        let name = legend
                            .get(tt as usize)
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| format!("unknown({})", tt));
                        (l, s, len, name)
                    })
                    .collect()
            }
            _ => panic!("expected Tokens result"),
        }
    }

    #[test]
    fn test_loc_basic_entry() {
        let content = "l_english:\n KEY:0 \"Hello world\"\n";
        let tokens = collect_loc_tokens(content);
        // l_english: → keyword + operator
        assert_eq!(
            tokens[0],
            (0, 0, 9, "keyword".to_string()),
            "header keyword"
        );
        assert_eq!(tokens[1], (0, 9, 1, "operator".to_string()), "header colon");
        // KEY:0 "Hello world" → type + operator + number + string + string + string
        assert_eq!(tokens[2], (1, 1, 3, "type".to_string()), "key");
        assert_eq!(tokens[3], (1, 4, 1, "operator".to_string()), "key colon");
        assert_eq!(tokens[4], (1, 5, 1, "number".to_string()), "version");
        assert_eq!(tokens[5], (1, 7, 1, "string".to_string()), "open quote");
        assert_eq!(tokens[6], (1, 8, 11, "string".to_string()), "value");
        assert_eq!(tokens[7], (1, 19, 1, "string".to_string()), "close quote");
    }

    #[test]
    fn test_loc_color_codes() {
        let content = "l_english:\n KEY:0 \"Hello §Gworld§!\"\n";
        let tokens = collect_loc_tokens(content);
        // Check that §G and §! are operator (color codes changed from enum_member)
        let operators: Vec<&(u32, u32, u32, String)> =
            tokens.iter().filter(|t| t.3 == "operator").collect();
        // line: " KEY:0 \"Hello §Gworld§!\""
        // pos:   KEY:0 \"Hello §Gworld§!\"
        // 0: space, 1:K, 2:E, 3:Y, 4::, 5:0, 6:space, 7:\", 8:H, 9:e, 10:l, 11:l, 12:o, 13:space, 14:§, 15:G, 16:w, 17:o, 18:r, 19:l, 20:d, 21:§, 22:!, 23:\"
        let color_ops: Vec<&&(u32, u32, u32, String)> =
            operators.iter().filter(|t| t.2 == 2).collect();
        assert_eq!(
            color_ops.len(),
            2,
            "§G and §! are operator tokens of length 2"
        );
        assert!(
            operators.iter().any(|t| t.1 == 14 && t.2 == 2),
            "§G at col 14"
        );
        assert!(
            operators.iter().any(|t| t.1 == 21 && t.2 == 2),
            "§! at col 21"
        );
    }

    #[test]
    fn test_loc_scope_ref() {
        let content = "l_english:\n KEY:0 \"Hello [ROOT.GetName]\"\n";
        let tokens = collect_loc_tokens(content);
        // [ROOT.GetName] should produce: operator([) + function(content) + operator(])
        // line: " KEY:0 \"Hello [ROOT.GetName]\""
        // pos: 0:_,1:K,2:E,3:Y,4::,5:0,6:_,7:\",8:H,9:e,10:l,11:l,12:o,13:_,14:[,15:R...
        let operators: Vec<&(u32, u32, u32, String)> =
            tokens.iter().filter(|t| t.3 == "operator").collect();
        // Should have: header colon, key colon, [, ]
        assert!(
            operators.iter().any(|t| t.2 == 1 && t.1 == 14),
            "opening bracket at col 14"
        );
        let functions: Vec<&(u32, u32, u32, String)> =
            tokens.iter().filter(|t| t.3 == "function").collect();
        assert_eq!(functions.len(), 1, "scope content is function");
        assert_eq!(functions[0].1, 15); // "ROOT.GetName" starts at col 15
        assert_eq!(functions[0].2, 12); // "ROOT.GetName" length
    }

    #[test]
    fn test_loc_nested_key() {
        let content = "l_english:\n KEY:0 \"Hello $OTHER$\"\n";
        let tokens = collect_loc_tokens(content);
        // $OTHER$ should produce: operator($) + variable(OTHER) + operator($)
        // line: " KEY:0 \"Hello $OTHER$\""
        // pos: 7:\",8:H,9:e,10:l,11:l,12:o,13:_,14:$,15:O,16:T,17:H,18:E,19:R,20:$,21:\"
        let variables: Vec<&(u32, u32, u32, String)> =
            tokens.iter().filter(|t| t.3 == "variable").collect();
        assert_eq!(variables.len(), 1, "nested key is variable");
        assert_eq!(variables[0].1, 15); // OTHER starts at col 15
        assert_eq!(variables[0].2, 5); // "OTHER" length
    }

    #[test]
    fn test_loc_comment() {
        let content = "l_english:\n# This is a comment\n";
        let tokens = collect_loc_tokens(content);
        let comments: Vec<&(u32, u32, u32, String)> =
            tokens.iter().filter(|t| t.3 == "comment").collect();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].0, 1); // line 1
        assert_eq!(comments[0].1, 0); // starts at col 0
    }

    #[test]
    fn test_loc_closing_quote_is_highlighted() {
        // Regression test: closing quote must have its own String token
        let content = "l_english:\n KEY:0 \"Hello\"\n";
        let tokens = collect_loc_tokens(content);
        // Tokens on line 1 should be: type, operator, number, string(open), string(value), string(close)
        let line1_tokens: Vec<&(u32, u32, u32, String)> =
            tokens.iter().filter(|t| t.0 == 1).collect();
        // Last token on line 1 should be the closing quote
        let last = line1_tokens.last().unwrap();
        assert_eq!(last.3, "string", "closing quote should be string");
        assert_eq!(last.2, 1, "closing quote should be length 1");
        // line: " KEY:0 \"Hello\""
        // pos: 7:\",8:H,9:e,10:l,11:l,12:o,13:\"
        assert_eq!(last.1, 13, "closing quote at col 13");
    }

    #[test]
    fn test_loc_mixed_content() {
        let content = "l_english:\n MY_KEY:0 \"§GHello§! from [ROOT.GetName] with $NESTED$\"\n";
        let tokens = collect_loc_tokens(content);
        let operators: Vec<&(u32, u32, u32, String)> =
            tokens.iter().filter(|t| t.3 == "operator").collect();
        let functions: Vec<&(u32, u32, u32, String)> =
            tokens.iter().filter(|t| t.3 == "function").collect();
        let variables: Vec<&(u32, u32, u32, String)> =
            tokens.iter().filter(|t| t.3 == "variable").collect();
        let color_ops: Vec<&&(u32, u32, u32, String)> = operators
            .iter()
            .filter(|t| t.2 == 2 && (t.1 == 11 || t.1 == 18))
            .collect();
        assert_eq!(color_ops.len(), 2, "§G and §! as operator");
        assert_eq!(functions.len(), 1, "one scope ref");
        assert_eq!(variables.len(), 1, "one nested key");
    }

    #[test]
    fn test_loc_no_version() {
        // Format: KEY: "value" (no version number)
        let content = "l_english:\n KEY: \"Hello\"\n";
        let tokens = collect_loc_tokens(content);
        // " KEY: \"Hello\""
        // pos: 1:K,2:E,3:Y,4::,5:_,6:\",...
        let line1_tokens: Vec<&(u32, u32, u32, String)> =
            tokens.iter().filter(|t| t.0 == 1).collect();
        // type, operator, string(open), string(value), string(close)
        assert_eq!(line1_tokens.len(), 5);
        assert_eq!(line1_tokens[1].3, "operator"); // :
        assert_eq!(line1_tokens[2].3, "string"); // opening " at col 6
        assert_eq!(line1_tokens[2].1, 6); // opening " at col 6
    }

    // ── CSV helper ──

    /// Helper: call csv_semantic_tokens and decode delta encoding into
    /// (line, start, length, token_type_name) tuples.
    fn collect_csv_tokens(content: &str) -> Vec<(u32, u32, u32, String)> {
        let result = csv_semantic_tokens(content).unwrap_or_else(|| {
            panic!("csv_semantic_tokens returned None for test content");
        });
        let legend = [
            "keyword",
            "variable",
            "string",
            "number",
            "operator",
            "comment",
            "type",
            "event",
            "function",
            "enum",
            "enum_member",
            "struct",
            "class",
            "property",
        ];
        match result {
            SemanticTokensResult::Tokens(t) => {
                let mut absolute: Vec<(u32, u32, u32, u32)> = Vec::new();
                let mut last_line = 0u32;
                let mut last_start = 0u32;
                for st in &t.data {
                    let line = last_line + st.delta_line;
                    let start = if st.delta_line == 0 {
                        last_start + st.delta_start
                    } else {
                        st.delta_start
                    };
                    absolute.push((line, start, st.length, st.token_type));
                    last_line = line;
                    last_start = start;
                }
                absolute
                    .into_iter()
                    .map(|(l, s, len, tt)| {
                        let name = legend
                            .get(tt as usize)
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| format!("unknown({})", tt));
                        (l, s, len, name)
                    })
                    .collect()
            }
            _ => panic!("expected Tokens result"),
        }
    }

    #[test]
    fn test_csv_def_basic_entry() {
        // definition.csv: Province ID;R;G;B;Type;Coastal;Terrain;Continent
        let content = "7;212;179;179;sea;true;ocean;0\n";
        let tokens = collect_csv_tokens(content);
        assert_eq!(tokens.len(), 8, "all 8 columns should produce tokens");
        // Province ID: col 0
        assert_eq!(tokens[0].0, 0);
        assert_eq!(tokens[0].1, 0);
        assert_eq!(tokens[0].2, 1);
        assert_eq!(tokens[0].3, "number");
        // R value: col 1
        assert_eq!(tokens[1].1, 2);
        assert_eq!(tokens[1].2, 3);
        assert_eq!(tokens[1].3, "number");
        // B value: col 3
        assert_eq!(tokens[3].1, 10);
        assert_eq!(tokens[3].2, 3);
        assert_eq!(tokens[3].3, "number");
        // Type 'sea': col 4 → keyword
        assert_eq!(tokens[4].1, 14);
        assert_eq!(tokens[4].2, 3);
        assert_eq!(tokens[4].3, "keyword");
        // Coastal 'true': col 5 → keyword
        assert_eq!(tokens[5].1, 18);
        assert_eq!(tokens[5].2, 4);
        assert_eq!(tokens[5].3, "keyword");
        // Terrain 'ocean': col 6 → type
        assert_eq!(tokens[6].1, 23);
        assert_eq!(tokens[6].2, 5);
        assert_eq!(tokens[6].3, "type");
        // Continent '0': col 7 → number
        assert_eq!(tokens[7].1, 29);
        assert_eq!(tokens[7].2, 1);
        assert_eq!(tokens[7].3, "number");
    }

    #[test]
    fn test_csv_def_comment() {
        let content = "# This is a province definition\n";
        let tokens = collect_csv_tokens(content);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].0, 0); // line 0
        assert_eq!(tokens[0].1, 0); // starts at col 0
        assert_eq!(tokens[0].3, "comment");
    }

    #[test]
    fn test_csv_def_header() {
        let content = "Province;R;G;B;Terrain;Coastal;Continent;ID\n";
        let tokens = collect_csv_tokens(content);
        // All columns are keyword (header)
        assert_eq!(tokens.len(), 8, "all 8 header columns keyword");
        for (i, tok) in tokens.iter().enumerate() {
            assert_eq!(
                tok.3, "keyword",
                "header col {} should be keyword, got {:?}",
                i, tok
            );
        }
    }

    #[test]
    fn test_csv_adj_basic_entry() {
        // adjacencies.csv: From;To;Type;Through;start_x;start_y;stop_x;stop_y;adjacency_rule_name;comment
        let content = "6891;3838;sea;5579;-1;-1;-1;-1;;Sardinia-Corsica\n";
        let tokens = collect_csv_tokens(content);
        // Expect 9 tokens (two empty columns aren't emitted: after ;; and last is text)
        // Columns: 6891(0), 3838(1), sea(2), 5579(3), -1(4), -1(5), -1(6), -1(7), (8 empty - skipped), Sardinia-Corsica(9)
        // Number: cols 0,1,3,4,5,6,7 → 7 number tokens
        assert!(
            tokens.len() >= 7,
            "at least 7 non-empty columns → {}",
            tokens.len()
        );
        // Check types
        let numbers: Vec<&(u32, u32, u32, String)> =
            tokens.iter().filter(|t| t.3 == "number").collect();
        let keywords: Vec<&(u32, u32, u32, String)> =
            tokens.iter().filter(|t| t.3 == "keyword").collect();
        let strings: Vec<&(u32, u32, u32, String)> =
            tokens.iter().filter(|t| t.3 == "string").collect();
        assert_eq!(numbers.len(), 7, "7 number columns");
        assert_eq!(keywords.len(), 1, "adjacency type 'sea'");
        assert_eq!(strings.len(), 1, "comment text");
        // Verify adjacency type
        assert_eq!(keywords[0].3, "keyword");
        assert_eq!(keywords[0].2, 3); // "sea" length 3
    }

    #[test]
    fn test_csv_adj_comment() {
        let content = "# Comment line for adjacencies\n";
        let tokens = collect_csv_tokens(content);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].3, "comment");
    }

    #[test]
    fn test_csv_whitespace_padding() {
        // definition.csv with extra whitespace around columns
        let content = "  7  ;  212  ;  179  ;  179  ;  sea  ;  true  ;  ocean  ;  0  \n";
        let tokens = collect_csv_tokens(content);
        assert_eq!(tokens.len(), 8, "all 8 padded columns tokenised");
        // Province ID: starts after padding
        assert_eq!(tokens[0].1, 2, "padded ID starts at col 2");
        assert_eq!(tokens[0].2, 1, "padded ID length 1");
        // Terrain
        assert_eq!(tokens[6].3, "type", "terrain is type");
    }

    #[test]
    fn test_csv_empty_lines_skipped() {
        let content = "7;212;179;179;sea;true;ocean;0\n\n\n114;40;15;15;land;false;plains;1\n";
        let tokens = collect_csv_tokens(content);
        assert_eq!(tokens.len(), 16, "2 data lines × 8 columns each");
    }

    #[test]
    fn test_csv_adj_header_detection() {
        let content = "From;To;Type;Through;start_x;start_y;stop_x;stop_y;adjacency_rule_name\n";
        let tokens = collect_csv_tokens(content);
        assert_eq!(tokens.len(), 9, "9 header columns");
        for tok in &tokens {
            assert_eq!(tok.3, "keyword", "header should be keyword");
        }
    }
}
