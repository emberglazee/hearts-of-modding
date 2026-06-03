#[cfg(test)]
mod tests {
    use crate::data::interner::InternedStr;
    use crate::data::layered_value::LayeredValue;
    use crate::parser::ast::Range;
    use crate::parser::loc_parser::{self, LocEntry};
    use dashmap::DashMap;
    use std::collections::HashSet;

    /// Empty containers for validate_loc_string parameters
    fn empty_targets() -> DashMap<InternedStr, Vec<crate::scanner::variable_scanner::EventTarget>> {
        DashMap::new()
    }
    fn empty_scripted_locs()
    -> DashMap<InternedStr, LayeredValue<crate::scanner::scripted_loc_scanner::ScriptedLoc>> {
        DashMap::new()
    }
    fn empty_color_codes() -> HashSet<String> {
        HashSet::new()
    }
    fn empty_country_tags() -> HashSet<String> {
        HashSet::new()
    }

    /// Helper: build a minimal LocEntry with a given value and value_start_col
    fn entry(value: &str, value_start_col: u32, start_line: u32) -> LocEntry {
        LocEntry {
            key: InternedStr::from("test"),
            value: value.to_string(),
            range: Range {
                start_line,
                start_col: 0,
                end_line: start_line,
                end_col: value_start_col + value.len() as u32,
            },
            path: InternedStr::from("test.yml"),
            value_start_col,
            version: None,
            version_range: None,
        }
    }

    // ─── color code column tests ────────────────────────────────────────

    #[test]
    fn test_dangling_color_reset_column_with_section_symbols() {
        let e = entry("§8Is home to a §gNovice§! exploitation effort.§!", 11, 0);
        let diags = loc_parser::validate_loc_string(
            &e,
            &empty_targets(),
            &empty_scripted_locs(),
            &empty_color_codes(),
            &empty_country_tags(),
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
        let e = entry("plain text§! trailing", 5, 0);
        let diags = loc_parser::validate_loc_string(
            &e,
            &empty_targets(),
            &empty_scripted_locs(),
            &empty_color_codes(),
            &empty_country_tags(),
        );

        let dangling: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_deref() == Some("dangling_color_reset"))
            .collect();
        assert_eq!(dangling.len(), 1);

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
            &empty_country_tags(),
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

        assert_eq!(
            unclosed[0].range.start_col, 15,
            "unclosed §R at byte=14 should be at UTF-16 col=15 with 3 § prefix"
        );
        assert_eq!(unclosed[0].range.end_col, 17);
    }

    // ─── bracket / scope column tests ───────────────────────────────────

    #[test]
    fn test_unescaped_bracket_column_after_section_symbols() {
        let e = entry("§Rtext[InvalidScope]more", 2, 0);
        let diags = loc_parser::validate_loc_string(
            &e,
            &empty_targets(),
            &empty_scripted_locs(),
            &empty_color_codes(),
            &empty_country_tags(),
        );

        let unescaped: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_deref() == Some("unescaped_bracket"))
            .collect();
        assert!(
            !unescaped.is_empty(),
            "all-parts-invalid diagnostic expected"
        );
        assert_eq!(
            unescaped[0].range.start_col, 8,
            "unescaped bracket [ at byte 7 → UTF-16 col 8 after § prefix"
        );

        let invalid: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_deref() == Some("invalid_loc_scope"))
            .collect();
        assert!(!invalid.is_empty());

        assert_eq!(
            invalid[0].range.start_col, 9,
            "invalid scope at byte 8 → UTF-16 col 9"
        );
    }

    #[test]
    fn test_bracket_column_ascii_only() {
        // No multi-byte chars — byte and UTF-16 should be identical
        // "Foobar" is 6 chars, not a 3-letter country tag — still invalid in lax mode
        let e = entry("text[Foobar]more", 5, 0);
        let diags = loc_parser::validate_loc_string(
            &e,
            &empty_targets(),
            &empty_scripted_locs(),
            &empty_color_codes(),
            &empty_country_tags(),
        );

        let invalid: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_deref() == Some("invalid_loc_scope"))
            .collect();

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
            &empty_country_tags(),
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
            &empty_country_tags(),
        );

        let var_fmts: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_deref() == Some("invalid_var_format"))
            .collect();
        assert!(
            !var_fmts.is_empty(),
            "expected invalid_var_format for '@' in $key$ formatting"
        );

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
            &empty_country_tags(),
        );

        let escaped: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_deref() == Some("escaped_bracket"))
            .collect();
        assert!(
            !escaped.is_empty(),
            "expected escaped_bracket diagnostic for backslash before ["
        );

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
            &empty_country_tags(),
        );

        let unknown_fmt: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_deref() == Some("unknown_loc_formatter"))
            .collect();
        assert!(
            !unknown_fmt.is_empty(),
            "expected unknown_loc_formatter diagnostic"
        );

        assert_eq!(
            unknown_fmt[0].range.start_col, 9,
            "loc formatter column should account for § bytes: byte 7 → UTF-16 6, start_col=2+6+1=9"
        );
    }
}
