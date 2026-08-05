use crate::parser::ast;
use crate::utils::line_index::LineIndex;
use std::path::{Path, PathBuf};
use tower_lsp_server::ls_types::{
    DiagnosticRelatedInformation, DiagnosticTag, Location, Position, Range, Uri,
};

/// Converts AST byte-column ranges into LSP UTF-16-column ranges.
///
/// `ast::Range` stores `start_col`/`end_col` as per-line **byte** offsets,
/// while the LSP `Position.character` is UTF-16 code units. For pure-ASCII
/// they coincide, but any multi-byte character (§, accents, emoji) before the
/// token shifts the byte column right of the true UTF-16 column. This mapper is
/// built once per document (O(n)) and then answers conversions in O(1): it pairs
/// a global byte→UTF-16 index (`LineIndex`) with the byte offset of each line
/// start, so a line's byte column maps to a UTF-16 column by a difference of two
/// global positions.
#[derive(Clone)]
pub(crate) struct RangeMapper {
    index: LineIndex,
    line_starts: Vec<u32>,
    byte_len: usize,
}

impl RangeMapper {
    pub(crate) fn new(text: &str) -> Self {
        let mut line_starts = Vec::new();
        line_starts.push(0u32);
        let mut offset = 0usize;
        for b in text.bytes() {
            offset += 1;
            if b == b'\n' {
                line_starts.push(offset as u32);
            }
        }
        Self {
            index: LineIndex::new(text),
            line_starts,
            byte_len: text.len(),
        }
    }

    /// Number of UTF-16 units on `line` ahead of the byte column `byte_col`.
    fn line_col_to_utf16(&self, line: u32, byte_col: u32) -> u32 {
        let Some(&line_start) = self.line_starts.get(line as usize) else {
            return byte_col;
        };
        let line_start_utf16 = self.index.byte_to_utf16(line_start as usize);
        // Clamp so an end-col at/just-past end of the document can't overrun
        // the table (byte_to_utf16 panics past text.len()).
        let global = (line_start as usize + byte_col as usize).min(self.byte_len);
        let col_utf16 = self.index.byte_to_utf16(global);
        col_utf16.saturating_sub(line_start_utf16)
    }

    /// Convert an AST byte range to an LSP UTF-16 range for the mapped document.
    pub(crate) fn range(&self, r: &ast::Range) -> Range {
        Range {
            start: Position {
                line: r.start_line,
                character: self.line_col_to_utf16(r.start_line, r.start_col),
            },
            end: Position {
                line: r.end_line,
                character: self.line_col_to_utf16(r.end_line, r.end_col),
            },
        }
    }
}

pub fn ast_tag_to_lsp(tag: &ast::DiagnosticTag) -> DiagnosticTag {
    match tag {
        ast::DiagnosticTag::Unnecessary => DiagnosticTag::UNNECESSARY,
        ast::DiagnosticTag::Deprecated => DiagnosticTag::DEPRECATED,
    }
}

/// Byte-based related-information converter.
///
/// NOTE: this path is only exercised on `diagnostic.related_information`, which
/// is never populated in practice (the parser produces it as an empty vec and
/// no rule fills it). It is kept byte-based purely to satisfy the plumbing;
/// do NOT use it for new code — use `RangeMapper` instead.
pub fn ast_related_info_to_lsp(
    info: &ast::DiagnosticRelatedInformation,
) -> DiagnosticRelatedInformation {
    DiagnosticRelatedInformation {
        location: Location {
            uri: info
                .location
                .uri
                .parse::<Uri>()
                .unwrap_or_else(|_| Uri::from_file_path(&info.location.uri).unwrap()),
            range: ast_range_to_lsp(&info.location.range),
        },
        message: info.message.clone(),
    }
}

/// Byte-based range converter (legacy plumbing only, see `ast_related_info_to_lsp`).
/// All new code must use `RangeMapper` so columns are correct for multi-byte text.
pub fn ast_range_to_lsp(range: &ast::Range) -> Range {
    Range {
        start: Position {
            line: range.start_line,
            character: range.start_col,
        },
        end: Position {
            line: range.end_line,
            character: range.end_col,
        },
    }
}

pub fn ast_range_to_lsp_location(range: &ast::Range, path: &str, mapper: &RangeMapper) -> Location {
    Location {
        uri: Uri::from_file_path(
            Path::new(path)
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from(path)),
        )
        .unwrap(),
        range: mapper.range(range),
    }
}

/// Check if an LSP Position falls within an AST Range (inclusive).
///
/// NOTE: `pos.character` MUST be expressed in **byte** columns on its line (the
/// same unit `ast::Range` uses). LSP sends UTF-16 characters, so callers that
/// receive a client position must convert it first via [`to_byte_position`];
/// otherwise multi-byte chars (§, accents) before the cursor mis-match the range.
pub(crate) fn is_pos_in_range(pos: Position, range: &ast::Range) -> bool {
    if pos.line < range.start_line || pos.line > range.end_line {
        return false;
    }
    if pos.line == range.start_line && pos.character < range.start_col {
        return false;
    }
    if pos.line == range.end_line && pos.character > range.end_col {
        return false;
    }
    true
}

/// Convert an LSP position (UTF-16 `character`) into a *byte*-column position on
/// the same line, so it can be compared against byte-based `ast::Range` columns
/// (see [`is_pos_in_range`]). This is the mirror step of `RangeMapper` — it bites
/// at the inception side (client cursor → byte) where the source is available.
pub(crate) fn to_byte_position(content: &str, pos: Position) -> Position {
    let byte_ch = match content.lines().nth(pos.line as usize) {
        Some(line) => {
            let li = LineIndex::new(line);
            let idx = pos.character.min(li.utf16_len());
            li.utf16_to_byte(idx as usize) as u32
        }
        None => pos.character, // line out of range — keep (won't match anyway)
    };
    Position {
        line: pos.line,
        character: byte_ch,
    }
}

// ---------------------------------------------------------------------------
// SECTION - Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast;

    /// A column on a line with multi-byte chars maps byte cols to correct UTF-16 cols.
    #[test]
    fn test_map_byte_col_to_utf16_line0() {
        // "a§b": a=1B, §=2B, b=1B. Byte-col 4 = char position 3 in UTF-16.
        let mapper = RangeMapper::new("a§b");
        let r = ast::Range {
            start_line: 0,
            start_col: 0,
            end_line: 0,
            end_col: 4,
        };
        let l = mapper.range(&r);
        assert_eq!(l.end.character, 3, "'a§b' byte col 4 -> utf16 col 3");
    }

    /// Multi-byte chars earlier on a later line shift the byte column from the UTF-16 column.
    #[test]
    fn test_map_byte_col_to_utf16_later_line() {
        // second line "née café": n(1) é(2) e(1) space(1) -> 5 bytes = 4 utf16.
        let text = "a\nnée café tail";
        let mapper = RangeMapper::new(text);
        let r = ast::Range {
            start_line: 1,
            start_col: 5,
            end_line: 1,
            end_col: 5,
        };
        let l = mapper.range(&r);
        assert_eq!(
            l.start.character, 4,
            "byte col 5 after 'née ' -> utf16 col 4"
        );
    }

    /// Pure-ASCII lines are unchanged (byte == utf16) — no regression.
    #[test]
    fn test_ascii_unchanged() {
        let mapper = RangeMapper::new("key = 10");
        let r = ast::Range {
            start_line: 0,
            start_col: 0,
            end_line: 0,
            end_col: 8,
        };
        assert_eq!(mapper.range(&r).end.character, 8);
    }

    /// Inception side: a client UTF-16 position on a multi-byte line must map to
    /// the correct BYTE column so `is_pos_in_range`/slicing match AST ranges.
    #[test]
    fn test_to_byte_position_multibyte() {
        // line 0 "plain", line 1 "a§b" (a=1B, §=2B, b=1B).
        let content = "plain\na§b\n";
        // UTF-16 character 2 on line 1 = 'b' (0:a, 1:§, 2:b).
        let pos = Position {
            line: 1,
            character: 2,
        };
        let bp = to_byte_position(content, pos);
        // byte column of 'b' = b'a'(1) + b'§'(2) = index 3.
        assert_eq!(bp.line, 1);
        assert_eq!(bp.character, 3, "UTF-16 col 2 -> byte col 3 on 'a§b'");

        // ASCII line: unchanged.
        let pos2 = Position {
            line: 0,
            character: 3,
        };
        assert_eq!(to_byte_position(content, pos2).character, 3);
    }
}

// ---------------------------------------------------------------------------
// !SECTION
// ---------------------------------------------------------------------------
