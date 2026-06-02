#[cfg(test)]
mod tests {
    use crate::{byte_offset_to_utf16, utf16_len, utf16_to_byte_offset};

    // ── utf16_to_byte_offset ──────────────────────────────────────────────

    #[test]
    fn test_utf16_to_byte_offset_ascii_only() {
        let s = "hello world";
        // ASCII-only means UTF-16 offset == byte offset
        assert_eq!(utf16_to_byte_offset(s, 0), 0);
        assert_eq!(utf16_to_byte_offset(s, 5), 5);
        assert_eq!(utf16_to_byte_offset(s, 11), 11);
    }

    #[test]
    fn test_utf16_to_byte_offset_before_paragraph() {
        // § (U+00A7) is 2 UTF-8 bytes = 1 UTF-16 code unit.
        // Cursor BEFORE § (UTF-16 offset = position of §):
        //   prefix should NOT include §
        let s = "abc§def";
        // UTF-16 indices:  a=0, b=1, c=2, §=3, d=4, e=5, f=6
        // UTF-8  bytes:   a=0, b=1, c=2, §=3..4, d=5, e=6, f=7
        // utf16_offset=3  → byte_offset should be 3 (start of §)
        assert_eq!(utf16_to_byte_offset(s, 3), 3);
        let prefix = &s[..utf16_to_byte_offset(s, 3)];
        assert_eq!(prefix, "abc");
    }

    #[test]
    fn test_utf16_to_byte_offset_after_paragraph() {
        // Cursor AFTER § (UTF-16 offset = position of 'd'):
        //   prefix SHOULD include §
        let s = "abc§def";
        // utf16_offset=4 (d)  → byte_offset should be 5 (after §)
        assert_eq!(utf16_to_byte_offset(s, 4), 5);
        let prefix = &s[..utf16_to_byte_offset(s, 4)];
        assert_eq!(prefix, "abc§");
    }

    #[test]
    fn test_utf16_to_byte_offset_cursor_at_end() {
        let s = "abc§";
        // utf16_offset=4  → byte_offset = 5 (past §)
        assert_eq!(utf16_to_byte_offset(s, 4), 5);
        let prefix = &s[..utf16_to_byte_offset(s, 4)];
        assert_eq!(prefix, "abc§");
    }

    #[test]
    fn test_utf16_to_byte_offset_empty_string() {
        assert_eq!(utf16_to_byte_offset("", 0), 0);
        assert_eq!(utf16_to_byte_offset("", 5), 0);
    }

    #[test]
    fn test_utf16_to_byte_offset_offset_zero() {
        let s = "abc§def";
        assert_eq!(utf16_to_byte_offset(s, 0), 0);
    }

    #[test]
    fn test_utf16_to_byte_offset_beyond_end() {
        let s = "abc";
        // offset beyond string → clamped to total bytes
        assert_eq!(utf16_to_byte_offset(s, 10), 3);
    }

    #[test]
    fn test_utf16_to_byte_offset_emoji() {
        // 🔥 (U+1F525) = 4 UTF-8 bytes = 2 UTF-16 surrogate code units
        let s = "a🔥b";
        // UTF-16 indices: a=0, 🔥=1-2 (surrogate pair), b=3
        // UTF-8  bytes:  a=0, 🔥=1..4, b=5
        //
        // utf16_offset=1 (start of 🔥-surrogate):
        assert_eq!(utf16_to_byte_offset(s, 1), 1);
        let prefix = &s[..utf16_to_byte_offset(s, 1)];
        assert_eq!(prefix, "a");

        // utf16_offset=3 (b, after 🔥):
        assert_eq!(utf16_to_byte_offset(s, 3), 5);
        let prefix = &s[..utf16_to_byte_offset(s, 3)];
        assert_eq!(prefix, "a🔥");

        // utf16_offset=2 (second surrogate unit of 🔥, in the middle):
        assert_eq!(utf16_to_byte_offset(s, 2), 1);
        let prefix = &s[..utf16_to_byte_offset(s, 2)];
        assert_eq!(prefix, "a");
    }

    #[test]
    fn test_utf16_to_byte_offset_mixed_multibyte() {
        // Various multi-byte characters
        // £ (U+00A3) = 2 UTF-8 bytes, 1 UTF-16
        // € (U+20AC) = 3 UTF-8 bytes, 1 UTF-16
        // 🎉 (U+1F389) = 4 UTF-8 bytes, 2 UTF-16
        let s = "£a€b🎉c";
        // UTF-16 indices: £=0, a=1, €=2, b=3, 🎉=4-5, c=6
        // UTF-8  bytes:  £=0..1, a=2, €=3..5, b=6, 🎉=7..10, c=11

        assert_eq!(utf16_to_byte_offset(s, 0), 0); // before £
        assert_eq!(utf16_to_byte_offset(s, 1), 2); // after £, before a
        assert_eq!(utf16_to_byte_offset(s, 2), 3); // before €
        assert_eq!(utf16_to_byte_offset(s, 3), 6); // after €, before b
        assert_eq!(utf16_to_byte_offset(s, 4), 7); // start of 🎉
        assert_eq!(utf16_to_byte_offset(s, 6), 11); // after 🎉, before c
    }

    // ── Crash scenario reproduction ──────────────────────────────────────

    #[test]
    fn test_utf16_conversion_does_not_panic_on_paragraph_crash_scenario() {
        // Exact line from the crash report
        let line = "\tstate_lore_text_container_state_name_7: \"Forbidden Mountains§\"";
        // The crash happened at position.character = 62 (UTF-16 offset of closing quote)
        let byte_off = utf16_to_byte_offset(line, 62);
        // Should produce a valid char boundary (not split § at bytes 61..63)
        let prefix = &line[..byte_off]; // would have panicked here before fix
        // Prefix should include everything including § (cursor was after §)
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
        // Test EVERY position around a multi-byte character to ensure
        // no position causes a panic when used as a slice bound.
        let line = "\tstate_lore_text_container_state_name_7: \"Forbidden Mountains§\"";
        for utf16_pos in 0..=65 {
            let byte_off = utf16_to_byte_offset(line, utf16_pos);
            // This must not panic
            let _prefix = &line[..byte_off];
            // And byte_off must be a valid char boundary
            assert!(
                line.is_char_boundary(byte_off),
                "byte_off {byte_off} (from utf16_pos {utf16_pos}) is not a char boundary"
            );
        }
    }

    #[test]
    fn test_utf16_conversion_simulates_completion_handler() {
        // The completion handler does:
        //   let byte_offset = crate::utf16_to_byte_offset(line, position.character as usize);
        //   let prefix = &line[..byte_offset];
        //   if let Some(bracket_start) = prefix.rfind('[') { ... }
        let line = "\tstate_lore_text_container_state_name_7: \"Forbidden Mountains§\"";
        let byte_off = utf16_to_byte_offset(line, 62);
        let prefix = &line[..byte_off];
        // Should not panic on rfind
        let _bracket_pos = prefix.rfind('[');
    }

    // ── byte_offset_to_utf16 (round-trip) ────────────────────────────────

    #[test]
    fn test_byte_offset_to_utf16_ascii() {
        let s = "hello";
        assert_eq!(byte_offset_to_utf16(s, 0), 0);
        assert_eq!(byte_offset_to_utf16(s, 3), 3);
        assert_eq!(byte_offset_to_utf16(s, 5), 5);
    }

    #[test]
    fn test_byte_offset_to_utf16_paragraph() {
        let s = "abc§def";
        // byte_offset 3 = start of § → 3 UTF-16 cu (a,b,c)
        assert_eq!(byte_offset_to_utf16(s, 3), 3);
        // byte_offset 5 = after § → 4 UTF-16 cu (a,b,c,§)
        assert_eq!(byte_offset_to_utf16(s, 5), 4);
    }

    #[test]
    fn test_byte_offset_to_utf16_emoji() {
        let s = "a🔥b";
        // byte_offset 1 = after 'a' → 1 UTF-16 cu
        assert_eq!(byte_offset_to_utf16(s, 1), 1);
        // byte_offset 5 = after 🔥 → 3 UTF-16 cu (a + 🔥-surrogate-pair)
        assert_eq!(byte_offset_to_utf16(s, 5), 3);
    }

    #[test]
    fn test_surrogate_pair_interior_offset_snaps_to_char_start() {
        // 🔥 (U+1F525) = 4 UTF-8 bytes, 2 UTF-16 code units
        let s = "a🔥b";
        // UTF-16 offsets: a=0, 🔥-lead=1, 🔥-trail=2, b=3
        // The interior offset 2 (trail surrogate) is not a valid Rust char boundary.
        // utf16_to_byte_offset should snap to the start of 🔥 (byte 1).
        let byte_off = utf16_to_byte_offset(s, 2);
        assert_eq!(byte_off, 1, "should snap to start of surrogate pair");
        let prefix = &s[..byte_off];
        assert_eq!(prefix, "a");

        // Offset 1 (lead surrogate) also snaps to char start
        let byte_off = utf16_to_byte_offset(s, 1);
        assert_eq!(byte_off, 1);
    }

    #[test]
    fn test_round_trip_utf16_to_byte_to_utf16() {
        let s = "abc§def🔥xyz£end";
        // Only test at UTF-16 offsets that align with Rust char boundaries.
        // Surrogate pairs (🔥=2cu, 1char) have an invalid interior offset
        // that cannot round-trip through byte offsets.
        let mut utf16_pos = 0;
        for c in s.chars() {
            assert_eq!(
                byte_offset_to_utf16(s, utf16_to_byte_offset(s, utf16_pos)) as usize,
                utf16_pos,
                "round-trip failed at char '{c}' (utf16_pos={utf16_pos})"
            );
            utf16_pos += c.len_utf16();
        }
        // Also test at end
        assert_eq!(
            byte_offset_to_utf16(s, utf16_to_byte_offset(s, utf16_pos)) as usize,
            utf16_pos
        );
    }

    #[test]
    fn test_round_trip_byte_to_utf16_to_byte() {
        let s = "abc§def🔥xyz£end";
        let len = s.len();
        // Test at each char boundary
        for byte_pos in 0..=len {
            if !s.is_char_boundary(byte_pos) {
                continue;
            }
            let utf16_pos = byte_offset_to_utf16(s, byte_pos) as usize;
            let back = utf16_to_byte_offset(s, utf16_pos);
            assert_eq!(
                byte_pos, back,
                "round-trip failed at byte_pos={byte_pos}: utf16_pos={utf16_pos} returned {back}"
            );
        }
    }

    // ── utf16_len ────────────────────────────────────────────────────────

    #[test]
    fn test_utf16_len_ascii() {
        assert_eq!(utf16_len("hello"), 5);
    }

    #[test]
    fn test_utf16_len_paragraph() {
        assert_eq!(utf16_len("abc§"), 4);
        assert_eq!(utf16_len("§§"), 2);
    }

    #[test]
    fn test_utf16_len_emoji() {
        // 🔥 is surrogate pair = 2 UTF-16 units
        assert_eq!(utf16_len("🔥"), 2);
        assert_eq!(utf16_len("a🔥b"), 4);
    }

    #[test]
    fn test_utf16_len_mixed() {
        assert_eq!(utf16_len("£a€b🎉c"), 7);
        // £=1, a=1, €=1, b=1, 🎉=2, c=1 = 7
    }
}
