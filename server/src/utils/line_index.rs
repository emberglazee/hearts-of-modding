/// A precomputed index mapping UTF-8 byte offsets ↔ UTF-16 code unit offsets.
///
/// Building this index is `O(n)` for `n` bytes. After building, both
/// `byte_to_utf16()` and `utf16_to_byte()` are `O(1)` — a single array lookup
/// (with a binary-search edge case on one direction for multi-byte chars).
///
/// LSP uses UTF-16 positions, while Rust strings are UTF-8. For pure ASCII these
/// are identical, but any non-ASCII character (§ = 2 bytes = 1 code unit,
/// emoji = 4 bytes = 2 code units) creates a divergence. Without precomputation,
/// every conversion walks the string from the start — leading to O(N²) in loops.
pub(crate) struct LineIndex {
    /// `utf16_offsets[byte_offset]` = cumulative UTF-16 code units at this
    /// byte position.  Length = `text.len() + 1`; the final entry is a
    /// sentinel holding the total UTF-16 length.
    utf16_offsets: Vec<u32>,
}

impl LineIndex {
    /// Build a new `LineIndex` for `text` in O(n) time.
    ///
    /// The returned index is valid as long as `text` is not mutated.
    pub fn new(text: &str) -> Self {
        let mut offsets = Vec::with_capacity(text.len() + 1);
        let mut utf16_offset: u32 = 0;
        for c in text.chars() {
            let cu = c.len_utf16() as u32;
            let nbytes = c.len_utf8();
            for _ in 0..nbytes {
                offsets.push(utf16_offset);
            }
            utf16_offset += cu;
        }
        offsets.push(utf16_offset); // sentinel for text.len()
        Self {
            utf16_offsets: offsets,
        }
    }

    /// Convert a byte offset to a UTF-16 code unit column — O(1).
    ///
    /// # Panics
    /// Panics if `byte_offset > text.len()`.
    #[inline]
    pub fn byte_to_utf16(&self, byte_offset: usize) -> u32 {
        self.utf16_offsets[byte_offset]
    }

    /// Convert a UTF-16 code unit offset to a byte offset — O(log n) worst-case
    /// (due to `partition_point` snapping past surrogate-pair interiors), but
    /// usually O(1) for exact char-boundary positions.
    ///
    /// If `utf16_offset` falls inside a surrogate-pair character (🔥 = 2 code
    /// units), it snaps to the byte offset at the *start* of that character.
    /// If `utf16_offset` exceeds the total UTF-16 length, returns `text.len()`.
    #[allow(dead_code)]
    pub fn utf16_to_byte(&self, utf16_offset: usize) -> usize {
        let target = utf16_offset as u32;

        // Clamp to the end of the string when past the total UTF-16 length.
        let total = self.utf16_len();
        if target >= total {
            return self.utf16_offsets.len() - 1;
        }

        // First byte where cumulative UTF-16 >= target.
        let idx = self
            .utf16_offsets
            .partition_point(|&offset| offset < target);

        if self.utf16_offsets[idx] == target {
            // Exact char boundary — done.
            idx
        } else {
            // Fell *inside* a multi-byte character (e.g. interior of a
            // surrogate pair).  Snap backward to the character start.
            let char_off = self.utf16_offsets[idx - 1];
            self.utf16_offsets
                .partition_point(|&offset| offset < char_off)
        }
    }

    /// Total UTF-16 length of the indexed text — O(1).
    #[allow(dead_code)]
    #[inline]
    pub fn utf16_len(&self) -> u32 {
        *self.utf16_offsets.last().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── byte_to_utf16 ─────────────────────────────────────────────────

    #[test]
    fn test_byte_to_utf16_ascii() {
        let idx = LineIndex::new("hello");
        assert_eq!(idx.byte_to_utf16(0), 0);
        assert_eq!(idx.byte_to_utf16(3), 3);
        assert_eq!(idx.byte_to_utf16(5), 5);
    }

    #[test]
    fn test_byte_to_utf16_paragraph() {
        let idx = LineIndex::new("abc§def");
        // byte 3 = start of § → 3 UTF-16 cu (a,b,c)
        assert_eq!(idx.byte_to_utf16(3), 3);
        // byte 5 = after § → 4 UTF-16 cu (a,b,c,§)
        assert_eq!(idx.byte_to_utf16(5), 4);
    }

    #[test]
    fn test_byte_to_utf16_emoji() {
        let idx = LineIndex::new("a🔥b");
        // byte 1 = after 'a' → 1 UTF-16 cu
        assert_eq!(idx.byte_to_utf16(1), 1);
        // byte 5 = after 🔥 → 3 UTF-16 cu (a + 🔥-surrogate-pair)
        assert_eq!(idx.byte_to_utf16(5), 3);
    }

    // ── utf16_to_byte ─────────────────────────────────────────────────

    #[test]
    fn test_utf16_to_byte_ascii() {
        let idx = LineIndex::new("hello world");
        assert_eq!(idx.utf16_to_byte(0), 0);
        assert_eq!(idx.utf16_to_byte(5), 5);
        assert_eq!(idx.utf16_to_byte(11), 11);
    }

    #[test]
    fn test_utf16_to_byte_before_paragraph() {
        let idx = LineIndex::new("abc§def");
        assert_eq!(idx.utf16_to_byte(3), 3);
    }

    #[test]
    fn test_utf16_to_byte_after_paragraph() {
        let idx = LineIndex::new("abc§def");
        assert_eq!(idx.utf16_to_byte(4), 5);
    }

    #[test]
    fn test_utf16_to_byte_cursor_at_end() {
        let idx = LineIndex::new("abc§");
        assert_eq!(idx.utf16_to_byte(4), 5);
    }

    #[test]
    fn test_utf16_to_byte_empty_string() {
        let idx = LineIndex::new("");
        assert_eq!(idx.utf16_to_byte(0), 0);
        assert_eq!(idx.utf16_to_byte(5), 0);
    }

    #[test]
    fn test_utf16_to_byte_beyond_end() {
        let idx = LineIndex::new("abc");
        assert_eq!(idx.utf16_to_byte(10), 3);
    }

    #[test]
    fn test_utf16_to_byte_emoji() {
        let idx = LineIndex::new("a🔥b");

        // utf16_offset=1 (start of 🔥-surrogate):
        assert_eq!(idx.utf16_to_byte(1), 1);

        // utf16_offset=3 (b, after 🔥):
        assert_eq!(idx.utf16_to_byte(3), 5);

        // utf16_offset=2 (second surrogate unit of 🔥, in the middle):
        assert_eq!(idx.utf16_to_byte(2), 1);
    }

    #[test]
    fn test_utf16_to_byte_mixed_multibyte() {
        // £ (U+00A3) = 2 UTF-8 bytes, 1 UTF-16
        // € (U+20AC) = 3 UTF-8 bytes, 1 UTF-16
        // 🎉 (U+1F389) = 4 UTF-8 bytes, 2 UTF-16
        let idx = LineIndex::new("£a€b🎉c");

        assert_eq!(idx.utf16_to_byte(0), 0); // before £
        assert_eq!(idx.utf16_to_byte(1), 2); // after £, before a
        assert_eq!(idx.utf16_to_byte(2), 3); // before €
        assert_eq!(idx.utf16_to_byte(3), 6); // after €, before b
        assert_eq!(idx.utf16_to_byte(4), 7); // start of 🎉
        assert_eq!(idx.utf16_to_byte(6), 11); // after 🎉, before c
    }

    // ── Round-trip ────────────────────────────────────────────────────

    #[test]
    fn test_round_trip_utf16_to_byte_to_utf16() {
        let s = "abc§def🔥xyz£end";
        let idx = LineIndex::new(s);

        let mut utf16_pos = 0;
        for c in s.chars() {
            let byte_off = idx.utf16_to_byte(utf16_pos);
            assert_eq!(
                idx.byte_to_utf16(byte_off) as usize,
                utf16_pos,
                "round-trip failed at char '{c}' (utf16_pos={utf16_pos})"
            );
            utf16_pos += c.len_utf16();
        }
        // Also test at end
        assert_eq!(
            idx.byte_to_utf16(idx.utf16_to_byte(utf16_pos)) as usize,
            utf16_pos
        );
    }

    #[test]
    fn test_round_trip_byte_to_utf16_to_byte() {
        let s = "abc§def🔥xyz£end";
        let idx = LineIndex::new(s);

        for byte_pos in 0..=s.len() {
            if !s.is_char_boundary(byte_pos) {
                continue;
            }
            let utf16_pos = idx.byte_to_utf16(byte_pos) as usize;
            let back = idx.utf16_to_byte(utf16_pos);
            assert_eq!(
                byte_pos, back,
                "round-trip failed at byte_pos={byte_pos}: utf16_pos={utf16_pos} returned {back}"
            );
        }
    }

    // ── utf16_len ─────────────────────────────────────────────────────

    #[test]
    fn test_utf16_len_ascii() {
        assert_eq!(LineIndex::new("hello").utf16_len(), 5);
    }

    #[test]
    fn test_utf16_len_paragraph() {
        assert_eq!(LineIndex::new("abc§").utf16_len(), 4);
        assert_eq!(LineIndex::new("§§").utf16_len(), 2);
    }

    #[test]
    fn test_utf16_len_emoji() {
        assert_eq!(LineIndex::new("🔥").utf16_len(), 2);
        assert_eq!(LineIndex::new("a🔥b").utf16_len(), 4);
    }

    #[test]
    fn test_utf16_len_mixed() {
        assert_eq!(LineIndex::new("£a€b🎉c").utf16_len(), 7);
    }

    // ── Crash scenario reproduction ───────────────────────────────────

    #[test]
    fn test_utf16_conversion_does_not_panic_on_paragraph_crash_scenario() {
        let line = "\tstate_lore_text_container_state_name_7: \"Forbidden Mountains§\"";
        let idx = LineIndex::new(line);
        let byte_off = idx.utf16_to_byte(62);
        let prefix = &line[..byte_off];
        assert!(
            prefix.ends_with('§'),
            "prefix should include §, got: {prefix:?}"
        );
        assert_eq!(
            prefix,
            "\tstate_lore_text_container_state_name_7: \"Forbidden Mountains§"
        );
    }

    #[test]
    fn test_utf16_conversion_at_each_position_around_paragraph() {
        let line = "\tstate_lore_text_container_state_name_7: \"Forbidden Mountains§\"";
        let idx = LineIndex::new(line);
        for utf16_pos in 0..=65 {
            let byte_off = idx.utf16_to_byte(utf16_pos);
            let _prefix = &line[..byte_off];
            assert!(
                line.is_char_boundary(byte_off),
                "byte_off {byte_off} (from utf16_pos {utf16_pos}) is not a char boundary"
            );
        }
    }

    #[test]
    fn test_utf16_conversion_simulates_completion_handler() {
        let line = "\tstate_lore_text_container_state_name_7: \"Forbidden Mountains§\"";
        let idx = LineIndex::new(line);
        let byte_off = idx.utf16_to_byte(62);
        let prefix = &line[..byte_off];
        let _bracket_pos = prefix.rfind('[');
    }
}
