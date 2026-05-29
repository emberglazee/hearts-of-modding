#[cfg(test)]
mod tests {
    use crate::ast::Range;
    use crate::loc_parser::{self, LocEntry};
    use dashmap::DashMap;
    use std::collections::HashSet;

    /// Empty containers for validate_loc_string parameters
    fn empty_targets() -> DashMap<String, Vec<crate::variable_scanner::EventTarget>> {
        DashMap::new()
    }
    fn empty_scripted_locs() -> DashMap<String, crate::scripted_loc_scanner::ScriptedLoc> {
        DashMap::new()
    }
    fn empty_color_codes() -> HashSet<String> {
        HashSet::new()
    }

    /// Helper: build a minimal LocEntry with a given value and value_start_col
    fn entry(value: &str, value_start_col: u32, start_line: u32) -> LocEntry {
        LocEntry {
            key: "test".to_string(),
            value: value.to_string(),
            range: Range {
                start_line,
                start_col: 0,
                end_line: start_line,
                end_col: value_start_col + value.len() as u32,
            },
            path: "test.yml".to_string(),
            value_start_col,
            version: None,
            version_range: None,
        }
    }

    // ─── color code column tests ────────────────────────────────────────

    #[test]
    fn test_dangling_color_reset_column_with_section_symbols() {
        // This is the exact pattern from the reported bug:
        //   §8...§g...§!...§!   — the second §! is dangling
        // Value has 3 § characters before the dangling §!
        let e = entry("§8Is home to a §gNovice§! exploitation effort.§!", 11, 0);
        let diags = loc_parser::validate_loc_string(
            &e,
            &empty_targets(),
            &empty_scripted_locs(),
            &empty_color_codes(),
        );

        // Find the dangling color reset diagnostic
        let dangling: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_deref() == Some("dangling_color_reset"))
            .collect();
        assert_eq!(
            dangling.len(),
            1,
            "expected exactly one dangling_color_reset diagnostic"
        );

        // Byte offset of last §! in value = 49. With value_start_col = 11 (all ASCII
        // prefix), the correct UTF-16 start_col is 11 + 46 = 57.
        //   Byte offset 49 = 3 § chars (6 bytes) + 43 ASCII bytes = 49
        //   UTF-16 offset = 3 § chars (3 units) + 43 ASCII chars (43 units) = 46
        // Wrong (byte-based) would give 11 + 49 = 60.
        assert_eq!(
            dangling[0].range.start_col, 57,
            "dangling §! should be at UTF-16 column 57, not byte-based 60"
        );
        assert_eq!(
            dangling[0].range.end_col, 59,
            "dangling §! end should be at UTF-16 column 59 (§! = 2 units)"
        );
    }

    #[test]
    fn test_dangling_color_reset_column_no_section_symbols() {
        // No § characters before the dangling reset — byte and UTF-16 should match
        let e = entry("plain text§! trailing", 5, 0);
        let diags = loc_parser::validate_loc_string(
            &e,
            &empty_targets(),
            &empty_scripted_locs(),
            &empty_color_codes(),
        );

        let dangling: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_deref() == Some("dangling_color_reset"))
            .collect();
        assert_eq!(dangling.len(), 1);

        // Byte offset of §! = 10. All ASCII, so UTF-16 offset = 10.
        // start_col = 5 + 10 = 15
        assert_eq!(dangling[0].range.start_col, 15);
        assert_eq!(dangling[0].range.end_col, 17);
    }

    #[test]
    fn test_unclosed_color_code_column_with_section_symbols() {
        // §8 opens, §g replaces it, §R opens at end and is never closed.
        // Value: §8Some§gText§R  (3 § chars before the unclosed §R)
        let e = entry("§8Some§gText§R", 3, 0);
        let diags = loc_parser::validate_loc_string(
            &e,
            &empty_targets(),
            &empty_scripted_locs(),
            &empty_color_codes(),
        );

        let unclosed: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_deref() == Some("unclosed_color_code"))
            .collect();
        assert_eq!(
            unclosed.len(),
            1,
            "expected exactly one unclosed_color_code diagnostic"
        );

        // value bytes: §(0-1),8(2),S(3),o(4),m(5),e(6),§(7-8),g(9),T(10),e(11),x(12),t(13),§(14-15),R(16)
        // pos = byte offset of last § = 14
        // chars before last §: §8Some§gText = 12 chars = 12 UTF-16 units
        // start_col = value_start_col(3) + 12 = 15
        assert_eq!(
            unclosed[0].range.start_col, 15,
            "unclosed §R at byte=14 should be at UTF-16 col=15 with 3 § prefix"
        );
        // §R is 2 chars = 2 UTF-16 units
        assert_eq!(unclosed[0].range.end_col, 17);
    }

    // ─── bracket / scope column tests ───────────────────────────────────

    #[test]
    fn test_unescaped_bracket_column_after_section_symbols() {
        // value: "§Rtext[InvalidScope]more"
        // §Rtext = 6 chars (1§ + 5 ASCII), then [ = char 6
        // bracket [ starts at byte 7, which is 6 UTF-16 code units in
        let e = entry("§Rtext[InvalidScope]more", 2, 0);
        let diags = loc_parser::validate_loc_string(
            &e,
            &empty_targets(),
            &empty_scripted_locs(),
            &empty_color_codes(),
        );

        // "[InvalidScope]" is unrecognized — all parts are invalid.
        // The unescaped_bracket diagnostic covers from [ (inclusive).
        // start_col = value_start_col + byte_offset_to_utf16(value, byte_of_[)
        //           = 2 + byte_offset_to_utf16("§Rtext[InvalidScope]more", 7)
        // s[..7] = bytes 0..6 = chars "§Rtext" = 6 UTF-16 units
        // start_col = 2 + 6 = 8
        let unescaped: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_deref() == Some("unescaped_bracket"))
            .collect();
        assert!(!unescaped.is_empty(), "all-parts-invalid diagnostic expected");
        assert_eq!(
            unescaped[0].range.start_col, 8,
            "unescaped bracket [ at byte 7 → UTF-16 col 8 after § prefix"
        );

        // Also check the first "invalid_loc_scope" for "InvalidScope"
        let invalid: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_deref() == Some("invalid_loc_scope"))
            .collect();
        assert!(!invalid.is_empty());
        // "InvalidScope" starts at byte 8 (after "[") = 7 chars from value start
        // UTF-16 offset = 7 chars = 7. start_col = 2 + 7 = 9.
        assert_eq!(
            invalid[0].range.start_col, 9,
            "invalid scope at byte 8 → UTF-16 col 9"
        );
    }

    #[test]
    fn test_bracket_column_ascii_only() {
        // No multi-byte chars — byte and UTF-16 should be identical
        let e = entry("text[Foo]more", 5, 0);
        let diags = loc_parser::validate_loc_string(
            &e,
            &empty_targets(),
            &empty_scripted_locs(),
            &empty_color_codes(),
        );

        let invalid: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_deref() == Some("invalid_loc_scope"))
            .collect();
        // "[Foo]" matches RE_SCOPE; start_pos = 4 (byte offset of [)
        // byte_offset_to_utf16("text[Foo]more", 4) = 4 (all ASCII = same)
        // start_col = 5 + 4 + 1 = 10 (the "F" of Foo)
        assert!(!invalid.is_empty());
        assert_eq!(
            invalid[0].range.start_col, 10,
            "ASCII-only bracket should have matching byte/UTF-16 column"
        );
    }

    // ─── variable formatting inside bracket column tests ────────────────

    #[test]
    fn test_var_formatting_column_with_section_symbols() {
        // [?var|FORMAT@] where @ is not a valid formatting character
        // Preceded by §R to test byte-vs-UTF-16 offset
        let e = entry("§Rtext[?var|FORMAT@]more", 2, 0);
        let diags = loc_parser::validate_loc_string(
            &e,
            &empty_targets(),
            &empty_scripted_locs(),
            &empty_color_codes(),
        );

        // Should get invalid_var_format diagnostics for '@'
        let var_fmts: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_deref() == Some("invalid_var_format"))
            .collect();
        assert!(
            !var_fmts.is_empty(),
            "expected invalid_var_format for '@' in formatting"
        );

        // The scope match starts at '[' at byte 7
        // "var|FORMAT@" in inner (after [?var|FORMAT@])
        // pipe_pos = 3 (position of | in "var|FORMAT@")
        // formatting = "FORMAT@"  (starts at pipe_pos + 1 = 4 in var_inner)
        // "@" is at formatting.find('@') = 6 in formatting
        // start_col = value_start_col + byte_offset_to_utf16(value, 7) + 2 + 3 + 6
        // byte_offset_to_utf16("§Rtext[?var|FORMAT@]more", 7) = 6 chars = 6
        // start_col = 2 + 6 + 2 + 3 + 6 = 19
        assert_eq!(
            var_fmts[0].range.start_col, 19,
            "var formatting '@' column should account for § prefix"
        );
    }

    // ─── nested $key$ formatting column tests ───────────────────────────

    #[test]
    fn test_nested_var_formatting_column_with_section_symbols() {
        // $key|FORMAT@$ with §R prefix — @ is invalid formatting
        let e = entry("§Rtext$key|FORMAT@$more", 2, 0);
        let diags = loc_parser::validate_loc_string(
            &e,
            &empty_targets(),
            &empty_scripted_locs(),
            &empty_color_codes(),
        );

        let var_fmts: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_deref() == Some("invalid_var_format"))
            .collect();
        assert!(
            !var_fmts.is_empty(),
            "expected invalid_var_format for '@' in $key$ formatting"
        );

        // $key|FORMAT@$ match starts at byte 7 (after §Rtext)
        // cap.get(0).unwrap().start() = 7
        // byte_offset_to_utf16(value, 7) = chars "§Rtext" = 6 UTF-16 units... wait
        // No, s[..7] = bytes 0-6 = chars: §,R,t,e,x,t = 6 chars = 6 UTF-16 units
        // Hmm but that's the $ sign. Let me recount.
        // 
        // value = "§Rtext$key|FORMAT@$more"
        // §(0-1), R(2), t(3), e(4), x(5), t(6), $(7)
        // s[..7] = bytes 0-6 = chars "§Rtext" = 6 chars = 6 UTF-16
        // So byte_offset_to_utf16(value, 7) = 6
        //
        // But wait, for the invalid_var_format diagnostic:
        // start_col = value_start_col + byte_offset_to_utf16(value, cap_start) + 1 + pipe_pos + 1 + formatting.find(c)
        // = 2 + 6 + 1 + 3 + 1 + 6 = 19
        assert_eq!(
            var_fmts[0].range.start_col, 19,
            "nested $key$ formatting column should account for § prefix"
        );
    }

    // ─── escaped bracket column test ────────────────────────────────────

    #[test]
    fn test_escaped_bracket_column_with_section_symbols() {
        // \\[ with a §-prefixed value — the backslash before [ is flagged
        let e = entry("§Rtext\\[GetTag]", 2, 0);
        let diags = loc_parser::validate_loc_string(
            &e,
            &empty_targets(),
            &empty_scripted_locs(),
            &empty_color_codes(),
        );

        let escaped: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_deref() == Some("escaped_bracket"))
            .collect();
        assert!(
            !escaped.is_empty(),
            "expected escaped_bracket diagnostic for backslash before ["
        );

        // The backslash is at byte 7 in value (after §Rtext = 6 bytes for 6 chars)
        // Wait: §(0-1), R(2), t(3), e(4), x(5), t(6), \(7), [(8)
        // The backslash is at byte 7.
        // start_pos = 7 (byte offset of backslash from the regex match start_pos)
        // Actually let me think about what RE_SCOPE returns.
        // RE_SCOPE = r"\[([^\]]+)\]"
        // For "§Rtext\[GetTag]", it matches "[GetTag]" starting at byte 8
        // start_pos = 8
        // But the escaped_bracket check looks at entry.value[..start_pos].chars().last()
        // to check if the preceding char is a backslash. start_pos = 8 here.
        // s[..8] = bytes 0-7 = chars "§Rtext\" = 7 chars
        // The last char is '\'. So preceding_char = Some('\\').
        //
        // The diagnostic start_col = value_start_col + byte_offset_to_utf16(value, start_pos)
        // byte_offset_to_utf16("§Rtext\\[GetTag]", 8):
        //   s[..8] = bytes 0-7 = chars "§Rtext\" = 7 chars = 7 UTF-16
        // start_col = 2 + 7 = 9
        assert_eq!(
            escaped[0].range.start_col, 9,
            "escaped bracket at byte 8 → UTF-16 col 9 (backslash at char 6)"
        );
    }

    // ─── loc formatter unknown column test ──────────────────────────────

    #[test]
    fn test_unknown_loc_formatter_column_with_section_symbols() {
        // [FakeFormatter|token] where FakeFormatter is not in the known formatters list
        let e = entry("§Rtext[FakeFormatter|token]", 2, 0);
        let diags = loc_parser::validate_loc_string(
            &e,
            &empty_targets(),
            &empty_scripted_locs(),
            &empty_color_codes(),
        );

        let unknown_fmt: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_deref() == Some("unknown_loc_formatter"))
            .collect();
        assert!(
            !unknown_fmt.is_empty(),
            "expected unknown_loc_formatter diagnostic"
        );

        // Scope match starts at '[' at byte 7
        // start_pos = 7
        // byte_offset_to_utf16(value, 7) = chars in s[..7] = "§Rtext" = 6 chars = 6
        // start_col = value_start_col + byte_offset_to_utf16(value, 7) + 1
        // = 2 + 6 + 1 = 9  (the 'F' of FakeFormatter)
        assert_eq!(
            unknown_fmt[0].range.start_col, 9,
            "loc formatter column should account for § bytes: byte 7 → UTF-16 6, start_col=2+6+1=9"
        );
    }
}
