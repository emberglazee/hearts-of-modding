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

    fn empty_characters()
    -> DashMap<InternedStr, LayeredValue<crate::scanner::character_scanner::Character>> {
        DashMap::new()
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

    /// Runs a loc range (byte columns) through RangeMapper over the synthetic
    /// line (value_start_col leading spaces + value) and returns (start,end) as
    /// UTF-16 characters — the same conversion the backend applies on emission,
    /// so these assert the visible editor columns.
    fn to_utf16(e: &LocEntry, r: &crate::parser::ast::Range) -> (u32, u32) {
        let line_n = " ".repeat(e.value_start_col as usize) + &e.value;
        let mapper = crate::utils::lsp_convert::RangeMapper::new(&line_n);
        let l = mapper.range(r);
        (l.start.character, l.end.character)
    }

    fn validate(value: &str) -> Vec<crate::parser::loc_parser::LocDiagnostic> {
        let e = entry(value, 2, 0);
        loc_parser::validate_loc_string(
            &e,
            &empty_targets(),
            &empty_scripted_locs(),
            &empty_color_codes(),
            &empty_country_tags(),
            &empty_characters(),
        )
    }

    fn columns_of(value: &str, code: &str) -> Vec<u32> {
        let e = entry(value, 2, 0);
        validate(value)
            .iter()
            .filter(|d| d.code.as_deref() == Some(code))
            .map(|d| to_utf16(&e, &d.range).0)
            .collect()
    }

    // ─── bracket / scope column tests ───────────────────────────────────

    #[test]
    fn test_unescaped_bracket_column_after_section_symbols() {
        let cols = columns_of("§Rtext[InvalidScope]more", "unescaped_bracket");
        assert_eq!(
            cols.first().copied(),
            Some(8),
            "unescaped bracket [ at byte 7 → UTF-16 col 8 after § prefix"
        );

        let cols = columns_of("§Rtext[InvalidScope]more", "invalid_loc_scope");
        assert_eq!(
            cols.first().copied(),
            Some(9),
            "invalid scope at byte 8 → UTF-16 col 9"
        );
    }

    #[test]
    fn test_bracket_column_ascii_only() {
        // No multi-byte chars — byte and UTF-16 should be identical.
        // "Foobar" is 6 chars, not a 3-letter country tag — still invalid in lax mode.
        // The diagnostic anchors on the scope name, which starts at byte 5 of the
        // value; with value_start_col=2 and pure ASCII the byte and UTF-16 columns
        // are the same number, which is the property under test.
        let cols = columns_of("text[Foobar]more", "invalid_loc_scope");
        assert_eq!(
            cols.first().copied(),
            Some(7),
            "ASCII-only scope name at byte 5 of the value → column 7"
        );
    }

    // ─── variable formatting inside bracket / nested $key$ ──────────────

    /// `[?var|FORMAT@]` and `$key|FORMAT@$` reach the same formatting validator
    /// with the same `@`; both must report the same column after a `§` prefix.
    #[test]
    fn test_var_formatting_column_with_section_symbols() {
        for value in ["§Rtext[?var|FORMAT@]more", "§Rtext$key|FORMAT@$more"] {
            let cols = columns_of(value, "invalid_var_format");
            assert_eq!(
                cols.first().copied(),
                Some(19),
                "'@' column should account for the § prefix in {value}"
            );
        }
    }

    // ─── escaped bracket column test ────────────────────────────────────

    #[test]
    fn test_escaped_bracket_column_with_section_symbols() {
        // \\[ with a §-prefixed value — the backslash before [ is flagged
        let cols = columns_of("§Rtext\\[GetTag]", "escaped_bracket");
        assert_eq!(
            cols.first().copied(),
            Some(9),
            "escaped bracket at byte 8 → UTF-16 col 9 (backslash at char 6)"
        );
    }

    // ─── loc formatter unknown column test ──────────────────────────────

    #[test]
    fn test_unknown_loc_formatter_column_with_section_symbols() {
        // [FakeFormatter|token] where FakeFormatter is not in the known formatters list
        let cols = columns_of("§Rtext[FakeFormatter|token]", "unknown_loc_formatter");
        assert_eq!(
            cols.first().copied(),
            Some(9),
            "loc formatter column should account for § bytes: byte 7 → UTF-16 6, start_col=2+6+1=9"
        );
    }

    // ─── parser property that justifies the removed fallback scans ──────

    /// Loc keys are split on the first `:` and trimmed, so they can never
    /// contain `:`. This is the parser property that makes `{key}:`-prefixed
    /// fallback scans (hover, goto-definition, HOM3018/3019 suppression)
    /// provably dead — they were removed as O(N) dead work. A regression here
    /// (keys gaining a colon) would silently resurrect them.
    #[test]
    fn test_loc_keys_never_contain_colon() {
        let content = "l_english:\n\
            foo:0 \"Value\"\n\
            foo.bar:5 \"Nested\"\n\
            baz_qux: \"No version\"\n";
        let (parsed, _, _) = loc_parser::parse_loc_file(content, "test.yml");
        let keys: Vec<String> = parsed.keys().map(|k| k.to_string()).collect();
        assert!(
            keys.iter().all(|k| !k.contains(':')),
            "loc keys must be colon-free, got: {keys:?}"
        );
        assert!(parsed.contains_key("foo"));
        assert!(parsed.contains_key("foo.bar"));
        assert!(parsed.contains_key("baz_qux"));
    }
}
