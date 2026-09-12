use crate::data::interner::InternedStr;
use crate::data::layered_value::LayeredValue;
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
/// Matches the placeholder left by flag extraction (`<PUA-E000><index><PUA-E001>`).
/// PUA codepoints survive the icon/scope/colour passes untouched, so extraction
/// must happen BEFORE them and expansion AFTER (the old code expanded `@TAG` to
/// `**[Flag: TAG]**` up front, which `RE_SCOPE` then re-matched into the
/// `****[Scope: Flag: TAG]****` hover bug).
static RE_FLAG_TOKEN: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new("\u{e000}(\\d+)\u{e001}").unwrap());
static RE_ICON: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"£([a-zA-Z0-9_]+)(?:\|[0-9]+)?").unwrap());
static RE_SCOPE: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"\[([^\]]+)\]").unwrap());
static RE_COLOR: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"§([a-zA-Z0-9!])").unwrap());

pub fn resolve_loc(
    input: &str,
    localization: &DashMap<InternedStr, LayeredValue<loc_parser::LocEntry>>,
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

/// A decoded country flag ready for inline SVG embedding. `width_px` /
/// `height_px` are the decoded bitmap dims; the display size is derived from
/// them preserving aspect (see `FLAG_DISPLAY_H`).
#[derive(Clone)]
pub struct FlagImage {
    pub width_px: u32,
    pub height_px: u32,
    pub data_uri: String,
}

/// One laid-out inline element of the preview: colored text or a flag image.
enum PreviewRun {
    Text { text: String, color: String },
    Flag { tag: String, image: FlagImage },
}

/// Process-wide decoded-flag cache keyed by absolute file path. Flags almost
/// never change mid-session, so no invalidation: editing a `.tga` needs an LSP
/// restart to show in hovers (same staleness class as the release binary).
/// Misses are NOT cached — the `fs::read` existence check is cheap and a later
/// edit may fix an undecodable file.
pub static FLAG_CACHE: Lazy<DashMap<String, FlagImage>> = Lazy::new(DashMap::new);

/// Display height of inline flags in the preview SVG (px). Width follows the
/// bitmap aspect; `FLAG_THUMB_H` is its 2x source for retina crispness.
pub const FLAG_DISPLAY_H: f64 = 12.0;
const FLAG_THUMB_H: u32 = 24;

/// Resolve `@TAG` to its engine flag (`gfx/flags/TAG.tga` — always the default
/// flag, never ideology variants; `hoi4-wiki/documentation/localisation.md`
/// §"Country's flags"). Mod roots shadow the game path (FileOverlay
/// semantics, cf. `rules/gfx_textures.rs` texture lookup). Returns `None` when
/// no file exists or it fails to decode — callers fall back to `[Flag: TAG]`.
pub fn resolve_flag_image(
    tag: &str,
    roots: &[std::path::PathBuf],
    game_path: Option<&str>,
    cache: &DashMap<String, FlagImage>,
) -> Option<FlagImage> {
    // The engine is case-insensitive, the Linux FS is not: try verbatim, then
    // canonical uppercase (mod files are uppercase by convention).
    let upper = tag.to_uppercase();
    let spellings: [&str; 2] = [tag, &upper];
    for dir in roots.iter().map(Some).chain(std::iter::once(
        game_path.map(std::path::PathBuf::from).as_ref(),
    )) {
        let Some(dir) = dir else { continue };
        for spelling in spellings {
            // `spellings` may hold the same &str twice; the cache makes the
            // duplicate lookup free.
            let candidate = dir.join("gfx/flags").join(format!("{spelling}.tga"));
            let key = candidate.to_string_lossy().into_owned();
            if let Some(hit) = cache.get(&key) {
                return Some(hit.clone());
            }
            if let Ok(bytes) = std::fs::read(&candidate) {
                if let Some((width_px, height_px, data_uri)) = decode_flag(&bytes) {
                    let flag = FlagImage {
                        width_px,
                        height_px,
                        data_uri,
                    };
                    cache.insert(key, flag.clone());
                    return Some(flag);
                }
            }
        }
    }
    None
}

/// Decode a flag `.tga` to a height-`FLAG_THUMB_H` PNG data URI. Tall custom
/// banners (Hearts of Minecraft flags are 82x164) survive intact — aspect is
/// preserved end to end, `FLAG_DISPLAY_H` only scales at render time.
fn decode_flag(bytes: &[u8]) -> Option<(u32, u32, String)> {
    let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Tga).ok()?;
    let rgba = img.to_rgba8();
    if rgba.width() == 0 || rgba.height() == 0 {
        return None;
    }
    let thumb = {
        // Fit within FLAG_THUMB_H preserving aspect. (Not `u32::MAX` width:
        // `thumbnail` allocates the exact output dims, so a huge bound is a
        // ~400GB alloc + pixel loop, not a fit-within.)
        let scale = FLAG_THUMB_H as f64 / rgba.height() as f64;
        let tw = ((rgba.width() as f64 * scale).ceil() as u32).max(1);
        image::imageops::thumbnail(&rgba, tw, FLAG_THUMB_H)
    };
    let (tw, th) = (thumb.width(), thumb.height());
    let mut png = Vec::new();
    {
        use image::ImageEncoder;
        image::codecs::png::PngEncoder::new(&mut png)
            .write_image(thumb.as_raw(), tw, th, image::ExtendedColorType::Rgba8)
            .ok()?;
    }
    Some((
        tw,
        th,
        format!(
            "data:image/png;base64,{}",
            general_purpose::STANDARD.encode(&png)
        ),
    ))
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Plain variant without flag resolution: `@TAG` renders as `[Flag: TAG]` text.
pub fn paradox_to_markdown(
    input: &str,
    localization: Option<&DashMap<InternedStr, LayeredValue<loc_parser::LocEntry>>>,
    color_map: Option<&DashMap<InternedStr, String>>,
) -> String {
    paradox_to_markdown_with_flags(input, localization, color_map, &|_| None)
}

/// Flush a buffered run of consecutive text as ONE multi-tspan `<text>`
/// element, advancing `x` past it. Keeps the pre-flag SVG shape (one `<text>`
/// per line for flag-free strings) while letting `<image>` sit inline.
fn flush_text_group(
    pending: &mut Option<(f64, String, usize)>,
    out: &mut String,
    x: &mut f64,
    y_pos: usize,
    font_size: i32,
    char_width: f64,
) {
    if let Some((gx, tspans, gw)) = pending.take() {
        out.push_str(&format!(r#"<text x="{gx:.1}" y="{y_pos}" font-family="monospace" font-size="{font_size}" font-weight="bold" xml:space="preserve">{tspans}</text>"#));
        *x += gw as f64 * char_width;
    }
}

/// Render Paradox localisation to a hover-preview SVG (base64 markdown image),
/// with `@TAG` country flags embedded inline. `flags` resolves a tag to its
/// decoded image; returning `None` falls back to `[Flag: TAG]` text.
pub fn paradox_to_markdown_with_flags(
    input: &str,
    localization: Option<&DashMap<InternedStr, LayeredValue<loc_parser::LocEntry>>>,
    color_map: Option<&DashMap<InternedStr, String>>,
    flags: &dyn Fn(&str) -> Option<FlagImage>,
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

    // Extract `@TAG` flags to PUA placeholders BEFORE the icon/scope passes:
    // expanding inline here would let RE_SCOPE re-match the brackets (the old
    // `****[Scope: Flag: TAG]****` bug). Expansion happens after the scope
    // pass, at layout time, where the resolver embeds the real flag image.
    let mut flag_tags: Vec<String> = Vec::new();
    resolved = RE_FLAG
        .replace_all(&resolved, |caps: &regex::Captures| {
            flag_tags.push(caps[1].to_string());
            format!("\u{e000}{}\u{e001}", flag_tags.len() - 1)
        })
        .to_string();

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

        // Display width of a flag in whole chars (ceil): a 1:2 banner at
        // FLAG_DISPLAY_H is 6px wide + 2px padding -> 2 chars.
        let flag_char_width = |image: &FlagImage| -> usize {
            let disp_w = FLAG_DISPLAY_H * image.width_px as f64 / image.height_px.max(1) as f64;
            (((disp_w + 2.0) / char_width).ceil() as usize).max(1)
        };
        let run_width = |run: &PreviewRun| -> usize {
            match run {
                PreviewRun::Text { text, .. } => text.chars().count(),
                PreviewRun::Flag { image, .. } => flag_char_width(image),
            }
        };
        // Split a word on flag placeholders, resolving each tag. Unresolvable
        // tags degrade to `[Flag: TAG]` text — never an error, and never a
        // second scope pass (false negatives over false positives).
        let split_word = |word: &str, color: &str| -> Vec<PreviewRun> {
            let mut pieces = Vec::new();
            let mut last = 0;
            for cap in RE_FLAG_TOKEN.captures_iter(word) {
                let m = cap.get(0).unwrap();
                if m.start() > last {
                    pieces.push(PreviewRun::Text {
                        text: word[last..m.start()].to_string(),
                        color: color.to_string(),
                    });
                }
                let tag = cap
                    .get(1)
                    .and_then(|idx| idx.as_str().parse::<usize>().ok())
                    .and_then(|idx| flag_tags.get(idx));
                match tag {
                    Some(tag) => match flags(tag) {
                        Some(image) => pieces.push(PreviewRun::Flag {
                            tag: tag.clone(),
                            image,
                        }),
                        None => pieces.push(PreviewRun::Text {
                            text: format!("[Flag: {tag}]"),
                            color: color.to_string(),
                        }),
                    },
                    None => pieces.push(PreviewRun::Text {
                        text: m.as_str().to_string(),
                        color: color.to_string(),
                    }),
                }
                last = m.end();
            }
            if last < word.len() {
                pieces.push(PreviewRun::Text {
                    text: word[last..].to_string(),
                    color: color.to_string(),
                });
            }
            pieces
        };

        let mut lines: Vec<Vec<PreviewRun>> = Vec::new();
        let mut current_line: Vec<PreviewRun> = Vec::new();
        let mut current_line_chars = 0;

        for (text, color) in segments {
            let parts: Vec<&str> = text.split('\n').collect();
            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    lines.push(std::mem::take(&mut current_line));
                    current_line_chars = 0;
                }

                let words: Vec<&str> = part.split(' ').collect();
                for (word_idx, word) in words.iter().enumerate() {
                    let runs = split_word(word, &color);
                    let word_len: usize = runs.iter().map(&run_width).sum();
                    let has_space = word_idx > 0;

                    if has_space {
                        if current_line_chars + 1 + word_len > chars_per_line
                            && !current_line.is_empty()
                        {
                            lines.push(std::mem::take(&mut current_line));
                            current_line = runs;
                            current_line_chars = word_len;
                        } else {
                            if !current_line.is_empty() {
                                current_line.push(PreviewRun::Text {
                                    text: " ".to_string(),
                                    color: color.clone(),
                                });
                                current_line_chars += 1;
                            }
                            current_line_chars += word_len;
                            current_line.extend(runs);
                        }
                    } else if current_line_chars + word_len > chars_per_line
                        && !current_line.is_empty()
                    {
                        lines.push(std::mem::take(&mut current_line));
                        current_line = runs;
                        current_line_chars = word_len;
                    } else {
                        current_line_chars += word_len;
                        current_line.extend(runs);
                    }
                }
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        let svg_height = lines.len() * line_height + 4;
        let mut svg_content = String::new();

        // Runs carry explicit x positions: text advances by char count, flags
        // by display width + padding. Consecutive text runs flush as ONE
        // <text> element (multi-tspan, as before) so a flag <image> can sit
        // inline between two text groups.
        for (line_idx, line_runs) in lines.iter().enumerate() {
            let y_pos = (line_idx + 1) * line_height;
            let mut x = 2.0f64;
            // (group x-origin, collected tspans, group width in chars)
            let mut pending: Option<(f64, String, usize)> = None;
            for run in line_runs {
                match run {
                    PreviewRun::Text { text, color } => {
                        if text.is_empty() {
                            continue;
                        }
                        let gx = x;
                        let entry = pending.get_or_insert_with(|| (gx, String::new(), 0));
                        entry.1.push_str(&format!(
                            r#"<tspan fill="{color}">{}</tspan>"#,
                            escape_xml(text)
                        ));
                        entry.2 += text.chars().count();
                    }
                    PreviewRun::Flag { tag, image } => {
                        flush_text_group(
                            &mut pending,
                            &mut svg_content,
                            &mut x,
                            y_pos,
                            font_size,
                            char_width,
                        );
                        let disp_w =
                            FLAG_DISPLAY_H * image.width_px as f64 / image.height_px.max(1) as f64;
                        svg_content.push_str(&format!(
                            r#"<image x="{x:.1}" y="{}" width="{disp_w:.1}" height="{FLAG_DISPLAY_H}" href="{}"><title>{}</title></image>"#,
                            y_pos as f64 - FLAG_DISPLAY_H,
                            image.data_uri,
                            escape_xml(tag)
                        ));
                        x += disp_w + 2.0;
                    }
                }
            }
            flush_text_group(
                &mut pending,
                &mut svg_content,
                &mut x,
                y_pos,
                font_size,
                char_width,
            );
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

/// Find the identifier inside a `[...]` scope or `$key$` reference on the
/// cursor's line of a `.yml` localization file.
///
/// Inception contract: the client's `pos.character` is UTF-16 code units but
/// the regex capture offsets (`m.start()`/`m.end()`) are byte columns on the
/// line. Convert once at this public entry (the caller's `content` is the
/// document the position refers to) so multi-byte chars (§, accents) before
/// the reference don't mis-resolve the identifier.
pub fn find_identifier_in_loc(content: &str, pos: Position) -> Option<String> {
    let pos = crate::utils::lsp_convert::to_byte_position(content, pos);
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
        let loc: DashMap<InternedStr, LayeredValue<LocEntry>> = DashMap::new();
        loc.insert(
            Arc::from("KEY1"),
            LayeredValue::new(LocEntry {
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
            }),
        );
        loc.insert(
            Arc::from("KEY2"),
            LayeredValue::new(LocEntry {
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
            }),
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
        let loc: DashMap<InternedStr, LayeredValue<LocEntry>> = DashMap::new();
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

    /// Regression: the cursor comes in as UTF-16 code units but the regex
    /// capture offsets are byte columns. On a line with § markers before a
    /// `[Root.GetName]` reference, a raw UTF-16 cursor on 'G' lands inside
    /// 'Root' (relative offset math off by the byte/UTF-16 delta) and resolves
    /// the WRONG identifier. The byte-converted position must resolve to
    /// 'GetName'.
    #[test]
    fn test_find_identifier_in_loc_utf16_to_byte_scope() {
        use tower_lsp_server::ls_types::Position;

        // § = 1 UTF-16 unit but 2 bytes; the two § markers before the scope
        // reference shift byte vs UTF-16 columns by 2.
        let line = "foo:0 \"§Rtext§![Root.GetName]\"";
        // 'G' of GetName: byte col 23, UTF-16 col 21.
        let pos = Position {
            line: 0,
            character: 21,
        };
        assert_eq!(
            find_identifier_in_loc(line, pos).as_deref(),
            Some("GetName"),
            "cursor on 'G' must resolve to 'GetName', not 'Root' \
             (raw UTF-16 col 21 would compute relative offset 3, inside 'Root')"
        );
        // Control: cursor on 'R' of Root — pure-ASCII region, byte == UTF-16.
        let pos_root = Position {
            line: 0,
            character: 16,
        };
        assert_eq!(
            find_identifier_in_loc(line, pos_root).as_deref(),
            Some("Root")
        );
    }

    /// Helper: unwrap the `![preview](data:image/svg+xml;base64,...)` wrapper.
    fn decode_preview_svg(output: &str) -> String {
        use base64::Engine as _;
        let b64 = output
            .split("base64,")
            .nth(1)
            .unwrap()
            .split(')')
            .next()
            .unwrap();
        String::from_utf8(
            base64::engine::general_purpose::STANDARD
                .decode(b64)
                .unwrap(),
        )
        .unwrap()
    }

    /// Regression: `@TAG` must not be re-matched by the scope pass. The old
    /// code expanded to `**[Flag: TAG]**` up front, which `RE_SCOPE` rewrote
    /// into `****[Scope: Flag: TAG]****` in hovers.
    #[test]
    fn test_flag_no_scope_wrap() {
        let loc: DashMap<InternedStr, LayeredValue<LocEntry>> = DashMap::new();
        let output = paradox_to_markdown(
            "Gives §R-50% §Ydefense§W for §5@KFS Foo§W.",
            Some(&loc),
            None,
        );
        let decoded = decode_preview_svg(&output);
        assert!(
            decoded.contains("[Flag: KFS]"),
            "fallback text kept: {decoded}"
        );
        assert!(
            !decoded.contains("Scope:"),
            "flag must not be scope-wrapped: {decoded}"
        );
    }

    /// A resolved flag embeds as an inline `<image>` with a `<title>` tag.
    #[test]
    fn test_flag_inline_image() {
        let loc: DashMap<InternedStr, LayeredValue<LocEntry>> = DashMap::new();
        let stub = |tag: &str| {
            assert_eq!(tag, "KFS");
            Some(FlagImage {
                width_px: 82,
                height_px: 164,
                data_uri: "data:image/png;base64,AAAA".to_string(),
            })
        };
        let output = paradox_to_markdown_with_flags("for §5@KFS Foo§W.", Some(&loc), None, &stub);
        let decoded = decode_preview_svg(&output);
        assert!(decoded.contains("<image"), "inline image: {decoded}");
        assert!(decoded.contains("AAAA"), "data uri embedded: {decoded}");
        assert!(
            decoded.contains("<title>KFS</title>"),
            "tag title: {decoded}"
        );
        assert!(!decoded.contains("[Flag:"), "no fallback text: {decoded}");
    }

    /// End to end: a real `.tga` under `gfx/flags/` resolves, decodes, and
    /// downscales to thumbnail height; unknown tags miss without caching.
    #[test]
    fn test_resolve_flag_image_tga() {
        use image::ImageEncoder;
        let root = std::env::temp_dir().join(format!("hom_flag_{}", std::process::id()));
        let dir = root.join("gfx/flags");
        std::fs::create_dir_all(&dir).unwrap();
        // 4x2 solid-red RGBA written as TGA via the image crate itself.
        let pixels = [255u8, 0, 0, 255].repeat(8);
        let path = dir.join("KFS.tga");
        let f = std::fs::File::create(&path).unwrap();
        image::codecs::tga::TgaEncoder::new(f)
            .write_image(&pixels, 4, 2, image::ExtendedColorType::Rgba8)
            .unwrap();

        let cache: DashMap<String, FlagImage> = DashMap::new();
        let roots = vec![root.clone()];
        let hit = resolve_flag_image("KFS", &roots, None, &cache).expect("flag resolves");
        assert_eq!(hit.height_px, 24, "thumbnail height");
        assert_eq!(hit.width_px, 48, "aspect preserved (4x2 -> 48x24)");
        assert!(
            hit.data_uri.starts_with("data:image/png;base64,"),
            "png data uri"
        );
        // Lowercase still hits (engine is case-insensitive, FS is not).
        assert!(resolve_flag_image("kfs", &roots, None, &cache).is_some());
        // Unknown tag misses, and the miss is not cached.
        assert!(resolve_flag_image("ZZZ", &roots, None, &cache).is_none());
        assert!(cache.iter().all(|e| !e.key().ends_with("ZZZ.tga")));
        // Second lookup serves from cache even after the file is deleted.
        std::fs::remove_file(&path).unwrap();
        assert!(resolve_flag_image("KFS", &roots, None, &cache).is_some());
        std::fs::remove_dir_all(&root).unwrap();
    }

    /// Same contract for `$key$` references: the membership test
    /// `char_offset < m.end()` is byte-based, so a UTF-16 cursor near the end
    /// of the key (where the byte/UTF-16 delta pushes it past the closing `$`
    /// in byte space) must NOT resolve the key.
    #[test]
    fn test_find_identifier_in_loc_utf16_to_byte_key() {
        use tower_lsp_server::ls_types::Position;

        let line = "foo:0 \"§Rtext§! $TAG_NAME$\"";
        // Cursor on 'N' inside the key: byte col 23, UTF-16 col 21.
        let pos = Position {
            line: 0,
            character: 21,
        };
        assert_eq!(
            find_identifier_in_loc(line, pos).as_deref(),
            Some("TAG_NAME")
        );

        // Cursor at end of line: UTF-16 col 26. In raw UTF-16 units that value
        // (26) still lies inside the byte window [18, 28) of the capture, so
        // the unconverted code resolves TAG_NAME even though the cursor is
        // past the closing `$`. The byte-converted position (28) is past
        // m.end() (exclusive) → None, which is the byte-consistent answer.
        let pos_delim = Position {
            line: 0,
            character: 26,
        };
        assert_eq!(
            find_identifier_in_loc(line, pos_delim),
            None,
            "cursor past the closing $ must not resolve the key"
        );
    }
}
