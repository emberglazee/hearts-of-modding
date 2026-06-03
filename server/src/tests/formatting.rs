#[cfg(test)]
mod tests {
    use crate::Backend;
    use crate::parser::parser;

    /// Helper: parse content, run collect_assignment_space_fixes, return fixes.
    fn get_fixes(content: &str) -> Vec<(crate::parser::ast::Range, String)> {
        let (script, _errors) = parser::parse_script(content);
        let mut fixes = Vec::new();
        Backend::collect_assignment_space_fixes(&script.entries, &mut fixes, content);
        fixes
    }

    // ── Basic patterns ──

    #[test]
    fn test_no_spaces_around_operator() {
        let fixes = get_fixes("key=value\n");
        assert!(
            !fixes.is_empty(),
            "Expected a fix for 'key=value' (no spaces)"
        );
        let (_range, text) = &fixes[0];
        assert_eq!(text, " = ", "Should produce ' = ' replacement");
        assert_eq!(_range.start_line, 0);
        assert_eq!(_range.start_col, 3);
        assert_eq!(_range.end_line, 0);
        assert_eq!(_range.end_col, 4);
    }

    #[test]
    fn test_too_many_spaces() {
        let fixes = get_fixes("key   =   value\n");
        assert!(!fixes.is_empty(), "Expected a fix for 'key   =   value'");
        let (_range, text) = &fixes[0];
        assert_eq!(text, " = ", "Should collapse to ' = '");
        // "key   =   value"
        //  0123456789...
        assert_eq!(_range.start_col, 3, "start_col should be after 'key'");
        assert_eq!(_range.end_col, 10, "end_col should be before 'value'");
    }

    #[test]
    fn test_correct_spacing_no_fix() {
        let fixes = get_fixes("key = value\n");
        assert!(
            fixes.is_empty(),
            "Should NOT produce a fix for 'key = value'"
        );
    }

    #[test]
    fn test_no_space_before_operator() {
        let fixes = get_fixes("key= value\n");
        assert!(!fixes.is_empty(), "Expected a fix for 'key= value'");
        let (_range, text) = &fixes[0];
        assert_eq!(text, " = ", "Should produce ' = ' replacement");
    }

    #[test]
    fn test_no_space_after_operator() {
        let fixes = get_fixes("key =value\n");
        assert!(!fixes.is_empty(), "Expected a fix for 'key =value'");
    }

    // ── Bool values ──

    #[test]
    fn test_bool_no_spaces() {
        let fixes = get_fixes("enable=yes\n");
        assert!(!fixes.is_empty(), "Expected a fix for bool assignment");
    }

    #[test]
    fn test_bool_correct_spacing() {
        let fixes = get_fixes("enable = yes\n");
        assert!(
            fixes.is_empty(),
            "Should NOT produce a fix for correct bool spacing"
        );
    }

    // ── String values ──

    #[test]
    fn test_string_no_spaces() {
        let fixes = get_fixes("name=\"my_region\"\n");
        assert!(
            !fixes.is_empty(),
            "Expected a fix for 'name=\"...\"' no spaces"
        );
    }

    #[test]
    fn test_string_correct_spacing() {
        let fixes = get_fixes("name = \"my_region\"\n");
        assert!(
            fixes.is_empty(),
            "Should NOT produce a fix for correct string spacing"
        );
    }

    // ── Number values ──

    #[test]
    fn test_number_no_spaces() {
        let fixes = get_fixes("id=42\n");
        assert!(!fixes.is_empty(), "Expected a fix for 'id=42'");
    }

    #[test]
    fn test_number_correct_spacing() {
        let fixes = get_fixes("id = 42\n");
        assert!(fixes.is_empty(), "Should NOT produce a fix for 'id = 42'");
    }

    // ── Single-line inline blocks ──

    #[test]
    fn test_inline_block_no_spaces() {
        let fixes = get_fixes("provinces={ 1 2 3 }\n");
        assert!(!fixes.is_empty(), "Expected a fix for 'provinces={{...}}'");
    }

    #[test]
    fn test_inline_block_correct_spacing() {
        let fixes = get_fixes("provinces = { 1 2 3 }\n");
        assert!(
            fixes.is_empty(),
            "Should NOT produce a fix for correct spacing"
        );
    }

    // ── Multi-line blocks ──

    #[test]
    fn test_multiline_block_no_spaces() {
        let content = "strategic_region={\n\tid=1\n\tname=\"foo\"\n}\n";
        let fixes = get_fixes(&content);
        // Should find at least the `strategic_region={...}` issue
        assert!(
            !fixes.is_empty(),
            "Expected fixes for multi-line with no spaces"
        );
    }

    #[test]
    fn test_multiline_block_correct_spacing() {
        let content = "strategic_region = {\n\tid = 1\n\tname = \"foo\"\n}\n";
        let fixes = get_fixes(&content);
        assert!(
            fixes.is_empty(),
            "Should NOT produce fixes for correctly spaced multi-line"
        );
    }

    // ── Block-level assignment spacing ──

    #[test]
    fn test_block_assignment_no_spaces() {
        let content = "strategic_region={\n\tid= 1\n\tname =\"foo\"\n\tprovinces= { 1 2 3 }\n}\n";
        let fixes = get_fixes(&content);
        // `strategic_region={` should be a fix (no space before `{`)
        // `id= 1` should be a fix (no space before `=`)
        // `name ="foo"` should be a fix (no space after `=`)
        // `provinces= { 1 2 3 }` should also be a fix (no space before `=`)
        assert_eq!(
            fixes.len(),
            4,
            "Expected 4 fixes: str_reg, id, name, provinces"
        );
    }

    // ── Mixed patterns: some correct, some wrong ──

    #[test]
    fn test_mixed_spacing() {
        let content = "id= 1\nname = \"foo\"\nprovinces = { 1 2 }\n";
        let fixes = get_fixes(&content);
        assert_eq!(fixes.len(), 1, "Only 'id= 1' should need a fix");
        assert_eq!(fixes[0].1, " = ", "Fix should normalize to ' = '");
    }

    // ── Strategic region style (realistic) ──

    #[test]
    fn test_strategic_region_style_no_spaces() {
        let content = "strategic_region={\n".to_owned()
            + "\tid= 1\n"
            + "\tname=\"STRATEGICREGION_1\"\n"
            + "\tprovinces={\n"
            + "\t\t1 2 3 4 5\n"
            + "\t}\n"
            + "\tnaval_strait={\n"
            + "\t\tfrom= 1\n"
            + "\t\tto= 2\n"
            + "\t}\n"
            + "}\n";
        let fixes = get_fixes(&content);
        // `strategic_region={`, `id= `, `name="..."`, `provinces={`, `naval_strait={`, `from= `, `to= `
        assert_eq!(fixes.len(), 7, "Expected 7 fixes for this strategic region");
        for (_, text) in &fixes {
            assert_eq!(text, " = ", "Every fix should produce ' = '");
        }
    }

    #[test]
    fn test_strategic_region_correct_spacing() {
        let content = "strategic_region = {\n".to_owned()
            + "\tid = 1\n"
            + "\tname = \"STRATEGICREGION_1\"\n"
            + "\tprovinces = {\n"
            + "\t\t1 2 3 4 5\n"
            + "\t}\n"
            + "\tnaval_strait = {\n"
            + "\t\tfrom = 1\n"
            + "\t\tto = 2\n"
            + "\t}\n"
            + "}\n";
        let fixes = get_fixes(&content);
        assert!(
            fixes.is_empty(),
            "Expected no fixes for correctly spaced strategic region"
        );
    }

    // ── Diagnostic emission (via check_assignment_spacing) ──
    // These verify that the styling diagnostics are actually emitted,
    // which was the root cause of the bug.

    fn get_assignment_diagnostics(content: &str) -> Vec<tower_lsp_server::ls_types::Diagnostic> {
        let (script, _errors) = parser::parse_script(content);
        let mut diags = Vec::new();
        Backend::check_assignment_spacing(&script.entries, content, &mut diags);
        diags
    }

    #[test]
    fn test_diagnostic_emitted_for_no_spaces() {
        let diags = get_assignment_diagnostics("key=value\n");
        assert_eq!(diags.len(), 1, "Should emit diagnostic for 'key=value'");
        let code = diags[0].code.as_ref().unwrap();
        match code {
            tower_lsp_server::ls_types::NumberOrString::String(s) => {
                assert_eq!(s, "styling_assignment_space");
            }
            _ => panic!("Expected string code"),
        }
        assert_eq!(
            diags[0].severity,
            Some(tower_lsp_server::ls_types::DiagnosticSeverity::INFORMATION)
        );
    }

    #[test]
    fn test_no_diagnostic_for_correct_spacing() {
        let diags = get_assignment_diagnostics("key = value\n");
        assert_eq!(
            diags.len(),
            0,
            "Should NOT emit diagnostic for 'key = value'"
        );
    }

    #[test]
    fn test_diagnostic_strategic_region() {
        let content = "strategic_region={\n".to_owned()
            + "\tid= 1\n"
            + "\tname=\"STRATEGICREGION_1\"\n"
            + "\tprovinces={\n"
            + "\t\t1 2 3 4 5\n"
            + "\t}\n"
            + "\tnaval_strait={\n"
            + "\t\tfrom= 1\n"
            + "\t\tto= 2\n"
            + "\t}\n"
            + "}\n";
        let diags = get_assignment_diagnostics(&content);
        // Same 7 as the fix test
        assert_eq!(
            diags.len(),
            7,
            "Expected 7 diagnostics for strategic region with no spaces"
        );
        for d in &diags {
            let code = d.code.as_ref().unwrap();
            match code {
                tower_lsp_server::ls_types::NumberOrString::String(s) => {
                    assert_eq!(s, "styling_assignment_space");
                }
                _ => panic!("Expected string code"),
            }
        }
    }

    #[test]
    fn test_no_diagnostic_for_correct_strategic_region() {
        let content = "strategic_region = {\n".to_owned()
            + "\tid = 1\n"
            + "\tname = \"STRATEGICREGION_1\"\n"
            + "\tprovinces = {\n"
            + "\t\t1 2 3 4 5\n"
            + "\t}\n"
            + "\tnaval_strait = {\n"
            + "\t\tfrom = 1\n"
            + "\t\tto = 2\n"
            + "\t}\n"
            + "}\n";
        let diags = get_assignment_diagnostics(&content);
        assert_eq!(
            diags.len(),
            0,
            "Expected 0 diagnostics for correctly spaced strategic region"
        );
    }

    #[test]
    fn test_diagnostic_range_matches_fix_range() {
        // Both the diagnostic and the fix should cover the same text span
        let content = "key=value\n";
        let (script, _errors) = parser::parse_script(content);
        let mut diags = Vec::new();
        Backend::check_assignment_spacing(&script.entries, content, &mut diags);
        let mut fixes = Vec::new();
        Backend::collect_assignment_space_fixes(&script.entries, &mut fixes, content);

        assert_eq!(diags.len(), 1);
        assert_eq!(fixes.len(), 1);

        let d_range = &diags[0].range;
        let (f_range, _) = &fixes[0];

        assert_eq!(d_range.start.line, f_range.start_line);
        assert_eq!(d_range.start.character, f_range.start_col);
        assert_eq!(d_range.end.line, f_range.end_line);
        assert_eq!(d_range.end.character, f_range.end_col);
    }
}
