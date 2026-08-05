// Differential (parity) tests for the legacy O(n) UTF-16 conversion helpers in
// `main.rs` (`utf16_to_byte_offset`, `byte_offset_to_utf16`, `utf16_len`).
//
// The authoritative expected-value tests live in `utils/line_index.rs`
// (`LineIndex`, the preferred O(1) implementation). These tests assert the two
// implementations AGREE across every offset of several representative strings —
// including the real-world crash-scenario line — so the legacy helpers can't
// drift from the O(1) index without a test failure, without duplicating every
// hand-computed expected constant.
#[cfg(test)]
mod tests {
    use crate::utils::line_index::LineIndex;
    use crate::{byte_offset_to_utf16, utf16_len, utf16_to_byte_offset};

    /// Every string the parity sweep runs over: ASCII, one 2-byte char (§),
    /// a 4-byte surrogate pair (🔥), a mixed 2/3/4-byte string, and the exact
    /// line from the original crash report.
    const SAMPLES: &[&str] = &[
        "hello world",
        "abc§def",
        "abc§",
        "a🔥b",
        "£a€b🎉c",
        "abc§def🔥xyz£end",
        "\tstate_lore_text_container_state_name_7: \"Forbidden Mountains§\"",
    ];

    /// `utf16_to_byte_offset` (legacy O(n)) must agree with
    /// `LineIndex::utf16_to_byte` at EVERY UTF-16 offset, including offsets
    /// that land inside surrogate pairs and past the end of the string.
    #[test]
    fn parity_utf16_to_byte_across_all_offsets() {
        for s in SAMPLES {
            let idx = LineIndex::new(s);
            let max = idx.utf16_len() as usize + 2;
            for utf16_pos in 0..=max {
                assert_eq!(
                    utf16_to_byte_offset(s, utf16_pos),
                    idx.utf16_to_byte(utf16_pos),
                    "utf16_to_byte_offset({utf16_pos}) diverged from LineIndex for {s:?}"
                );
            }
        }
    }

    /// `byte_offset_to_utf16` (legacy O(n)) must agree with
    /// `LineIndex::byte_to_utf16` at every char-boundary byte offset. (Both
    /// slice with `s[..byte_offset]`, so only char boundaries are valid.)
    #[test]
    fn parity_byte_to_utf16_across_all_offsets() {
        for s in SAMPLES {
            let idx = LineIndex::new(s);
            for byte_pos in 0..=s.len() {
                if !s.is_char_boundary(byte_pos) {
                    continue; // interior of a multi-byte char — not a valid slice bound
                }
                assert_eq!(
                    byte_offset_to_utf16(s, byte_pos),
                    idx.byte_to_utf16(byte_pos),
                    "byte_offset_to_utf16({byte_pos}) diverged from LineIndex for {s:?}"
                );
            }
        }
    }

    /// `utf16_len` (legacy O(n)) must agree with `LineIndex::utf16_len`.
    #[test]
    fn parity_utf16_len() {
        for s in SAMPLES {
            assert_eq!(
                utf16_len(s),
                LineIndex::new(s).utf16_len(),
                "utf16_len diverged from LineIndex for {s:?}"
            );
        }
    }

    /// The original crash-scenario regression, asserted against the legacy
    /// helper directly: a UTF-16 cursor at 62 on this line must resolve to a
    /// valid char boundary whose prefix includes the § character.
    #[test]
    fn crash_scenario_prefix_via_legacy_helper() {
        let line = "\tstate_lore_text_container_state_name_7: \"Forbidden Mountains§\"";
        let byte_off = utf16_to_byte_offset(line, 62);
        let prefix = &line[..byte_off]; // would have panicked before the fix
        assert!(
            prefix.ends_with('§'),
            "prefix should include §, got: {prefix:?}"
        );
        assert_eq!(
            prefix,
            "\tstate_lore_text_container_state_name_7: \"Forbidden Mountains§"
        );
    }

    /// Completion-handler simulation: the legacy helper must not panic when
    /// used the way `completion_handler.rs` uses it (slice + rfind).
    #[test]
    fn completion_handler_simulation_via_legacy_helper() {
        let line = "\tstate_lore_text_container_state_name_7: \"Forbidden Mountains§\"";
        let byte_off = utf16_to_byte_offset(line, 62);
        let prefix = &line[..byte_off];
        let _bracket_pos = prefix.rfind('[');
    }
}
