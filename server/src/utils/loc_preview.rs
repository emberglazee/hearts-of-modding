use crate::data::interner::InternedStr;
use crate::parser::loc_parser;
use base64::{Engine as _, engine::general_purpose};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use tower_lsp_server::ls_types::Position;

fn vanilla_color_hex(code: &str) -> &str {
    match code {
        "C" => "#23CEFF",
        "L" => "#C3B091",
        "W" | "T" => "#FFFFFF",
        "B" => "#0000FF",
        "G" => "#009F03",
        "R" => "#FF3232",
        "b" => "#000000",
        "g" => "#B0B0B0",
        "Y" | "H" => "#FFBD00",
        "O" => "#FF7019",
        "0" => "#CB00CB",
        "1" => "#8078D3",
        "2" => "#5170F3",
        "3" => "#518FDC",
        "4" => "#5ABEE7",
        "5" => "#3FB5C2",
        "6" => "#77CCBA",
        "7" => "#99D199",
        "8" => "#CCA333",
        "9" => "#FCA97D",
        "t" => "#FF4C4D",
        _ => "#FFFFFF",
    }
}

static RE_KEY: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"\$([^\$]+)\$").unwrap());
static RE_FLAG: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"@([a-zA-Z0-9]{3})").unwrap());
static RE_ICON: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"£([a-zA-Z0-9_]+)(?:\|[0-9]+)?").unwrap());
static RE_SCOPE: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"\[([^\]]+)\]").unwrap());
static RE_COLOR: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"§([a-zA-Z0-9!])").unwrap());

pub fn resolve_loc(
    input: &str,
    localization: &DashMap<InternedStr, loc_parser::LocEntry>,
    depth: u32,
) -> String {
    if depth > 10 {
        return input.to_string();
    }
    let mut last_end = 0;
    let mut result = String::new();

    for cap in RE_KEY.captures_iter(input) {
        let m = cap.get(0).unwrap();
        let key = cap.get(1).unwrap().as_str();

        result.push_str(&input[last_end..m.start()]);
        if let Some(entry) = localization.get(key) {
            result.push_str(&resolve_loc(&entry.value, localization, depth + 1));
        } else {
            result.push_str(m.as_str());
        }
        last_end = m.end();
    }
    result.push_str(&input[last_end..]);
    result
}

/// Build a hex color map: symbol "Y" -> "#FFBD00" from ColorCode data
pub fn build_color_map(data: &crate::ScannerData) -> DashMap<InternedStr, String> {
    let codes = &data.color_codes;
    let map = DashMap::new();
    for entry in codes.iter() {
        let sym = entry.key();
        let cc = entry.value();
        map.insert(
            sym.clone(),
            format!("#{:02X}{:02X}{:02X}", cc.rgb.0, cc.rgb.1, cc.rgb.2),
        );
    }
    map
}

pub fn paradox_to_markdown(
    input: &str,
    localization: Option<&DashMap<InternedStr, loc_parser::LocEntry>>,
    color_map: Option<&DashMap<InternedStr, String>>,
) -> String {
    fn split_leading_punctuation(s: &str) -> (&str, &str) {
        let punct_end = s
            .chars()
            .take_while(|c| c.is_ascii_punctuation() || c.is_whitespace())
            .map(|c| c.len_utf8())
            .sum::<usize>();

        if punct_end > 0 {
            (&s[..punct_end], &s[punct_end..])
        } else {
            ("", s)
        }
    }

    let mut resolved = if let Some(loc) = localization {
        resolve_loc(input, loc, 0)
    } else {
        input.to_string()
    };

    resolved = resolved
        .replace("\\n", "\n")
        .replace("\\r\\n", "\n")
        .replace("\\\"", "\"")
        .replace("$$", "$");

    resolved = RE_FLAG.replace_all(&resolved, "**[Flag: $1]**").to_string();

    resolved = RE_ICON.replace_all(&resolved, "**[Icon: $1]**").to_string();

    let mut scope_result = String::new();
    let mut last_scope_end = 0;

    for cap in RE_SCOPE.captures_iter(&resolved) {
        let m = cap.get(0).unwrap();
        scope_result.push_str(&resolved[last_scope_end..m.start()]);
        let inner = cap.get(1).unwrap().as_str();

        if inner.contains('?') && inner.contains(':') {
            scope_result.push_str(&format!("**[Condition: {}]**", inner));
        } else if let Some(var_inner) = inner.strip_prefix('?') {
            if let Some(pipe_pos) = var_inner.find('|') {
                scope_result.push_str(&format!("**[Variable: {}]**", &var_inner[..pipe_pos]));
            } else {
                scope_result.push_str(&format!("**[Variable: {}]**", var_inner));
            }
        } else if inner.find('|').is_some() {
            scope_result.push_str(&format!("**[Format: {}]**", inner));
        } else if inner.contains('.') || inner.chars().any(|c| c.is_uppercase()) {
            scope_result.push_str(&format!("**[Scope: {}]**", inner));
        } else {
            scope_result.push_str(&format!("**[{}]**", inner));
        }

        last_scope_end = m.end();
    }

    scope_result.push_str(&resolved[last_scope_end..]);
    resolved = scope_result;

    let mut last_end = 0;

    let mut segments: Vec<(String, String)> = Vec::new();
    let mut current_color = "#FFFFFF".to_string();

    for cap in RE_COLOR.captures_iter(&resolved) {
        let m = cap.get(0).unwrap();
        let code = cap.get(1).unwrap().as_str();

        let text_segment = &resolved[last_end..m.start()];

        let (leading_punct, rest) = split_leading_punctuation(text_segment);

        if !leading_punct.is_empty() {
            segments.push((leading_punct.to_string(), current_color.clone()));
        }

        if !rest.is_empty() {
            segments.push((rest.to_string(), current_color.clone()));
        }

        current_color = if code == "!" {
            "#FFFFFF".to_string()
        } else if let Some(map) = color_map {
            map.get(code).map(|s| s.value().clone()).unwrap_or_else(|| {
                // Fallback to hardcoded known colors
                vanilla_color_hex(code).to_string()
            })
        } else {
            vanilla_color_hex(code).to_string()
        };
        last_end = m.end();
    }

    let last_segment = &resolved[last_end..];
    if !last_segment.is_empty() {
        segments.push((last_segment.to_string(), current_color.clone()));
    }

    if !segments.is_empty() {
        let font_size = 12;
        let char_width = 7.2;
        let max_width = 600;
        let line_height = 16;
        let chars_per_line = (max_width as f64 / char_width).floor() as usize;

        let mut lines: Vec<Vec<(String, String)>> = Vec::new();
        let mut current_line: Vec<(String, String)> = Vec::new();
        let mut current_line_chars = 0;

        for (text, color) in segments {
            let parts: Vec<&str> = text.split('\n').collect();
            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    lines.push(current_line);
                    current_line = Vec::new();
                    current_line_chars = 0;
                }

                let words: Vec<&str> = part.split(' ').collect();
                for (word_idx, word) in words.iter().enumerate() {
                    let word_len = word.chars().count();
                    let has_space = word_idx > 0;

                    if has_space {
                        if current_line_chars + 1 + word_len > chars_per_line
                            && !current_line.is_empty()
                        {
                            lines.push(current_line);
                            current_line = Vec::new();
                            current_line.push((word.to_string(), color.clone()));
                            current_line_chars = word_len;
                        } else {
                            if !current_line.is_empty() {
                                current_line.push((" ".to_string(), color.clone()));
                                current_line_chars += 1;
                            }
                            current_line.push((word.to_string(), color.clone()));
                            current_line_chars += word_len;
                        }
                    } else {
                        if current_line_chars + word_len > chars_per_line
                            && !current_line.is_empty()
                        {
                            lines.push(current_line);
                            current_line = Vec::new();
                            current_line.push((word.to_string(), color.clone()));
                            current_line_chars = word_len;
                        } else {
                            current_line.push((word.to_string(), color.clone()));
                            current_line_chars += word_len;
                        }
                    }
                }
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        let svg_height = lines.len() * line_height + 4;
        let mut svg_content = String::new();

        for (line_idx, line_segments) in lines.iter().enumerate() {
            let y_pos = (line_idx + 1) * line_height;
            svg_content.push_str(&format!(r#"<text x="2" y="{}" font-family="monospace" font-size="{}" font-weight="bold" xml:space="preserve">"#, y_pos, font_size));

            for (text, color) in line_segments {
                let escaped_text = text
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
                    .replace('"', "&quot;")
                    .replace('\'', "&apos;");
                svg_content.push_str(&format!(
                    r#"<tspan fill="{}">{}</tspan>"#,
                    color, escaped_text
                ));
            }

            svg_content.push_str("</text>");
        }

        let svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">{}</svg>"#,
            max_width, svg_height, max_width, svg_height, svg_content
        );

        let b64 = general_purpose::STANDARD.encode(svg);
        return format!("![preview](data:image/svg+xml;base64,{})", b64);
    }

    String::new()
}

pub fn find_identifier_in_loc(content: &str, pos: Position) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let line = lines.get(pos.line as usize)?;
    let char_offset = pos.character as usize;

    for cap in RE_SCOPE.captures_iter(line) {
        let m = cap.get(0).unwrap();

        if char_offset >= m.start() && char_offset < m.end() {
            let inner = cap.get(1).unwrap().as_str();
            let relative_offset = char_offset - m.start() - 1;
            let parts: Vec<&str> = inner.split('.').collect();
            let mut current_pos = 0;
            for part in parts {
                if relative_offset >= current_pos && relative_offset < current_pos + part.len() {
                    return Some(part.to_string());
                }
                current_pos += part.len() + 1;
            }
        }
    }

    for cap in RE_KEY.captures_iter(line) {
        let m = cap.get(0).unwrap();
        if char_offset >= m.start() && char_offset < m.end() {
            return Some(cap.get(1).unwrap().as_str().to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::interner::InternedStr;
    use crate::parser::ast::Range;
    use crate::parser::loc_parser::LocEntry;
    use std::sync::Arc;

    #[test]
    fn test_resolve_loc() {
        let loc: DashMap<InternedStr, LocEntry> = DashMap::new();
        loc.insert(
            Arc::from("KEY1"),
            LocEntry {
                key: Arc::from("KEY1"),
                value: "Value 1".to_string(),
                range: Range {
                    start_line: 0,
                    start_col: 0,
                    end_line: 0,
                    end_col: 0,
                },
                path: Arc::from(""),
                value_start_col: 0,
                version: None,
                version_range: None,
            },
        );
        loc.insert(
            Arc::from("KEY2"),
            LocEntry {
                key: Arc::from("KEY2"),
                value: "Contains $KEY1$".to_string(),
                range: Range {
                    start_line: 0,
                    start_col: 0,
                    end_line: 0,
                    end_col: 0,
                },
                path: Arc::from(""),
                value_start_col: 0,
                version: None,
                version_range: None,
            },
        );

        assert_eq!(resolve_loc("Hello $KEY1$", &loc, 0), "Hello Value 1");
        assert_eq!(
            resolve_loc("Hello $KEY2$", &loc, 0),
            "Hello Contains Value 1"
        );
        assert_eq!(resolve_loc("Hello $UNKNOWN$", &loc, 0), "Hello $UNKNOWN$");
    }

    #[test]
    fn test_paradox_to_markdown_newlines() {
        use base64::Engine as _;
        let loc: DashMap<InternedStr, LocEntry> = DashMap::new();
        let input = "Line 1\\nLine 2";
        let output = paradox_to_markdown(input, Some(&loc), None);
        let decoded = String::from_utf8(
            base64::engine::general_purpose::STANDARD
                .decode(
                    output
                        .split("base64,")
                        .nth(1)
                        .unwrap()
                        .split(')')
                        .next()
                        .unwrap(),
                )
                .unwrap(),
        )
        .unwrap();
        assert_eq!(decoded.matches("<text ").count(), 2);
        assert!(decoded.contains("Line"));
        assert!(decoded.contains("1"));
        assert!(decoded.contains("Line"));
        assert!(decoded.contains("2"));
    }

    #[test]
    fn test_paradox_to_markdown_real_newlines() {
        use base64::Engine as _;
        let input = "Line 1\nLine 2";
        let output = paradox_to_markdown(input, None, None);
        let decoded = String::from_utf8(
            base64::engine::general_purpose::STANDARD
                .decode(
                    output
                        .split("base64,")
                        .nth(1)
                        .unwrap()
                        .split(')')
                        .next()
                        .unwrap(),
                )
                .unwrap(),
        )
        .unwrap();
        assert_eq!(decoded.matches("<text ").count(), 2);
        assert!(decoded.contains("Line"));
        assert!(decoded.contains("1"));
        assert!(decoded.contains("Line"));
        assert!(decoded.contains("2"));
    }

    #[test]
    fn test_paradox_to_markdown_escaped_quotes() {
        use base64::Engine as _;
        let input = "Hello \\\"World\\\"";
        let output = paradox_to_markdown(input, None, None);
        let decoded = String::from_utf8(
            base64::engine::general_purpose::STANDARD
                .decode(
                    output
                        .split("base64,")
                        .nth(1)
                        .unwrap()
                        .split(')')
                        .next()
                        .unwrap(),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(decoded.contains("&quot;World&quot;"));
    }

    #[test]
    fn test_paradox_to_markdown_no_extra_space() {
        use base64::Engine as _;
        let input = "§Rfoo§Gbar";
        let output = paradox_to_markdown(input, None, None);
        let decoded = String::from_utf8(
            base64::engine::general_purpose::STANDARD
                .decode(
                    output
                        .split("base64,")
                        .nth(1)
                        .unwrap()
                        .split(')')
                        .next()
                        .unwrap(),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(decoded.contains("foo</tspan><tspan"));
        assert!(decoded.contains(">bar</tspan>"));
        assert!(!decoded.contains("> <"));
    }
}
