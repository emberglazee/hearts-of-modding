#[cfg(test)]
mod tests {
    use crate::Backend;
    use crate::parser::ast::Range;
    use crate::parser::parser;
    use tower_lsp_server::ls_types::{Diagnostic, NumberOrString};

    /// Helper: parse content, run collect_assignment_space_fixes, return fixes.
    fn get_fixes(content: &str) -> Vec<(Range, String)> {
        let (script, _errors) = parser::parse_script(content);
        let mut fixes = Vec::new();
        Backend::collect_assignment_space_fixes(&script.entries, &mut fixes, content);
        fixes
    }

    fn get_assignment_diagnostics(content: &str) -> Vec<Diagnostic> {
        let (script, _errors) = parser::parse_script(content);
        let mut diags = Vec::new();
        Backend::check_assignment_spacing(&script.entries, content, &mut diags);
        diags
    }

    fn diagnostic_code(d: &Diagnostic) -> &str {
        match d.code.as_ref().expect("diagnostic without a code") {
            NumberOrString::String(s) => s,
            other => panic!("expected a string code, got {other:?}"),
        }
    }

    // ── Fix presence per value kind ──

    /// One row per value kind: an unspaced assignment must produce a fix, the
    /// correctly spaced form must produce none. These were ten near-identical
    /// tests differing only in the literal.
    #[test]
    fn assignment_spacing_fixes_by_value_kind() {
        let cases: &[(&str, &str)] = &[
            ("enable=yes\n", "enable = yes\n"),
            ("name=\"my_region\"\n", "name = \"my_region\"\n"),
            ("id=42\n", "id = 42\n"),
            ("provinces={ 1 2 3 }\n", "provinces = { 1 2 3 }\n"),
            (
                "strategic_region={\n\tid=1\n\tname=\"foo\"\n}\n",
                "strategic_region = {\n\tid = 1\n\tname = \"foo\"\n}\n",
            ),
        ];
        for (bad, good) in cases {
            assert!(
                !get_fixes(bad).is_empty(),
                "expected a fix for unspaced {bad:?}"
            );
            assert!(
                get_fixes(good).is_empty(),
                "expected no fix for correctly spaced {good:?}"
            );
        }
    }

    // ── Fix ranges and replacement text ──

    /// The fix replaces the span between key and value with `" = "`. The two
    /// rows with known geometry pin the range; the rest pin the replacement.
    #[test]
    fn assignment_spacing_fix_ranges() {
        let cases: &[(&str, u32, u32)] = &[
            ("key=value\n", 3, 4),
            ("key   =   value\n", 3, 10),
            ("key= value\n", 3, 5),
            ("key =value\n", 3, 5),
        ];
        for (input, start_col, end_col) in cases {
            let fixes = get_fixes(input);
            let (range, text) = fixes.first().expect("expected a fix");
            assert_eq!(text, " = ", "fix for {input:?} should be ' = '");
            assert_eq!(range.start_line, 0);
            assert_eq!(range.end_line, 0);
            assert_eq!(range.start_col, *start_col, "start_col for {input:?}");
            assert_eq!(range.end_col, *end_col, "end_col for {input:?}");
        }
    }

    // ── Fixes and diagnostics agree ──

    const STRATEGIC_BAD: &str = "strategic_region={\n\tid= 1\n\tname=\"STRATEGICREGION_1\"\n\tprovinces={\n\t\t1 2 3 4 5\n\t}\n\tnaval_strait={\n\t\tfrom= 1\n\t\tto= 2\n\t}\n}\n";
    const STRATEGIC_GOOD: &str = "strategic_region = {\n\tid = 1\n\tname = \"STRATEGICREGION_1\"\n\tprovinces = {\n\t\t1 2 3 4 5\n\t}\n\tnaval_strait = {\n\t\tfrom = 1\n\t\tto = 2\n\t}\n}\n";

    /// The code action and the diagnostic are two walks over the same AST; a
    /// divergence between them is the bug class this pins. Each row asserts the
    /// count in both walks and that the diagnostic ranges equal the fix ranges.
    #[test]
    fn fixes_and_diagnostics_agree() {
        let cases: &[(&str, usize)] = &[
            ("key=value\n", 1),
            ("key = value\n", 0),
            (
                "strategic_region={\n\tid= 1\n\tname =\"foo\"\n\tprovinces= { 1 2 3 }\n}\n",
                4,
            ),
            ("id= 1\nname = \"foo\"\nprovinces = { 1 2 }\n", 1),
            (STRATEGIC_BAD, 7),
            (STRATEGIC_GOOD, 0),
        ];
        for (content, expected) in cases {
            let fixes = get_fixes(content);
            let diags = get_assignment_diagnostics(content);
            assert_eq!(
                fixes.len(),
                *expected,
                "fix count for {content:?} (fixes: {fixes:?})"
            );
            assert_eq!(
                diags.len(),
                *expected,
                "diagnostic count for {content:?} (diags: {diags:?})"
            );
            for (d, (f, text)) in diags.iter().zip(fixes.iter()) {
                assert_eq!(diagnostic_code(d), "styling_assignment_space");
                assert_eq!(text, " = ");
                assert_eq!(d.range.start.line, f.start_line, "start line {content:?}");
                assert_eq!(
                    d.range.start.character, f.start_col,
                    "start col {content:?}"
                );
                assert_eq!(d.range.end.line, f.end_line, "end line {content:?}");
                assert_eq!(d.range.end.character, f.end_col, "end col {content:?}");
            }
        }
    }

    /// Styling diagnostics are informational: they are suggestions, not errors.
    #[test]
    fn assignment_spacing_diagnostic_is_information() {
        let diags = get_assignment_diagnostics("key=value\n");
        assert_eq!(diags.len(), 1);
        assert_eq!(diagnostic_code(&diags[0]), "styling_assignment_space");
        assert_eq!(
            diags[0].severity,
            Some(tower_lsp_server::ls_types::DiagnosticSeverity::INFORMATION)
        );
    }

    // ── Indentation fixes (UTF-16 columns) ──

    /// Helper: run `collect_indentation_fixes` over `content` with a parsed AST.
    fn get_indent_fixes(content: &str) -> Vec<(tower_lsp_server::ls_types::Range, String)> {
        let (script, _errors) = parser::parse_script(content);
        let mut fixes = Vec::new();
        Backend::collect_indentation_fixes(content, Some(&script), &mut fixes);
        fixes
    }

    #[test]
    fn indentation_fix_uses_utf16_columns_for_non_ascii_whitespace() {
        // `is_whitespace` includes NBSP (U+00A0), which is ONE UTF-16 unit but
        // TWO bytes. Emitting the byte length as `character` made the edit
        // range overrun the indent and swallow the first character of the line,
        // so "Fix all styling issues" deleted a character of the key.
        //
        // This line is one the parser does not turn into an entry (its
        // whitespace is ASCII-only), so it takes the spaces branch; NBSP plus
        // an ASCII space is 3 bytes but 2 UTF-16 units.
        let content = "a = b\n\u{a0} key = value\n";
        let mut fixes = Vec::new();
        Backend::collect_indentation_fixes(content, None, &mut fixes);

        let (range, text) = fixes
            .iter()
            .find(|(r, _)| r.start.line == 1)
            .expect("expected an indentation fix on the NBSP-indented line");
        assert_eq!(text, "\t");
        assert_eq!(range.start.character, 0);
        assert_eq!(
            range.end.character, 2,
            "NBSP + space is two UTF-16 units (three bytes); a byte length would \
             make the edit swallow the 'k'"
        );
    }

    /// Controls: an ASCII space indent is rewritten with an unchanged range
    /// (byte and UTF-16 lengths agree), and a correct tab indent is left alone.
    #[test]
    fn indentation_controls() {
        let fixes = get_indent_fixes("test = {\n  key = value\n}\n");
        let (range, text) = fixes
            .iter()
            .find(|(r, _)| r.start.line == 1)
            .expect("expected an indentation fix on the space-indented line");
        assert_eq!(text, "\t");
        assert_eq!((range.start.character, range.end.character), (0, 2));

        let fixes = get_indent_fixes("test = {\n\tkey = value\n}\n");
        assert!(
            fixes.iter().all(|(r, _)| r.start.line != 1),
            "a correct tab indent must not be rewritten: {fixes:?}"
        );
    }
}
