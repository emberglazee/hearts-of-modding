#[cfg(test)]
mod tests {
    use crate::data::hoi4_data::{UTF8_BOM, has_exactly_one_bom};
    use crate::parser::loc_parser;

    #[test]
    fn test_missing_language_header_suppression() {
        let empty_content = "";
        let commented_content = "# Just a comment\n\n# Another comment\n";
        let valid_content = "l_english:\n test_key:0 \"Test\"";
        let missing_header_content = "test_key:0 \"Test\"";

        assert!(loc_parser::validate_loc_file_structure(empty_content).is_empty());
        assert!(loc_parser::validate_loc_file_structure(commented_content).is_empty());
        assert!(loc_parser::validate_loc_file_structure(valid_content).is_empty());

        let diags = loc_parser::validate_loc_file_structure(missing_header_content);
        assert!(!diags.is_empty());
        assert_eq!(diags[0].code.as_deref(), Some("missing_language_header"));
    }

    #[test]
    fn test_language_header_case_sensitivity() {
        use crate::parser::ast::DiagnosticSeverity;
        // Exact lowercase → clean
        assert!(loc_parser::validate_loc_file_structure("l_english:\n K:0 \"hi\"\n").is_empty());
        assert!(loc_parser::validate_loc_file_structure("l_polish:\n K:0 \"hi\"\n").is_empty());
        assert!(
            loc_parser::validate_loc_file_structure("l_simp_chinese:\n K:0 \"hi\"\n").is_empty()
        );

        // Wrong case → language_case_mismatch Error (probe_loc_case 2026-09-01: silently discarded in-game)
        for bad in [
            "l_English:\n K:0 \"hi\"\n",
            "l_Polish:\n K:0 \"hi\"\n",
            "L_english:\n K:0 \"hi\"\n",
            "L_ENGLISH:\n K:0 \"hi\"\n",
            "l_SIMP_CHINESE:\n K:0 \"hi\"\n",
        ] {
            let diags = loc_parser::validate_loc_file_structure(bad);
            assert_eq!(
                diags.len(),
                1,
                "bad header {bad:?} should produce exactly one diagnostic"
            );
            assert_eq!(
                diags[0].code.as_deref(),
                Some("language_case_mismatch"),
                "bad header {bad:?}"
            );
            assert_eq!(
                diags[0].severity,
                DiagnosticSeverity::Error,
                "case mismatch must be Error"
            );
            assert!(
                diags[0].message.contains("should be"),
                "message should suggest correct lowercase"
            );
        }

        // Truly unknown → unknown_language Warning (not case mismatch)
        let diags = loc_parser::validate_loc_file_structure("l_elvish:\n K:0 \"hi\"\n");
        assert_eq!(diags[0].code.as_deref(), Some("unknown_language"));
        assert_eq!(diags[0].severity, DiagnosticSeverity::Warning);
    }

    /// HOM6005 classifier: a localization file must start with EXACTLY ONE
    /// UTF-8 BOM. Vanilla corpus ground truth: 2073/2073 files carry exactly
    /// one; zero and 2+ are both broken (the latter is what LLM-generated
    /// files produce when they prepend a BOM to an already-BOM'd file).
    #[test]
    fn test_has_exactly_one_bom() {
        let bom = UTF8_BOM;
        let header = b"l_english:\n";

        // Exactly one BOM → healthy, regardless of what follows.
        assert!(has_exactly_one_bom(&bom));
        assert!(has_exactly_one_bom(
            &[bom.as_slice(), header.as_slice()].concat()
        ));

        // No BOM at all → flagged.
        assert!(!has_exactly_one_bom(header));
        assert!(!has_exactly_one_bom(&[]));

        // Double / triple BOM → flagged (the LLM failure mode).
        let double = [bom.as_slice(), bom.as_slice(), header.as_slice()].concat();
        let triple = [
            bom.as_slice(),
            bom.as_slice(),
            bom.as_slice(),
            header.as_slice(),
        ]
        .concat();
        assert!(!has_exactly_one_bom(&double), "double BOM must be flagged");
        assert!(!has_exactly_one_bom(&triple), "triple BOM must be flagged");

        // Truncated garbage: lone EF BB or empty — no false pass.
        assert!(!has_exactly_one_bom(&[0xEF]));
        assert!(!has_exactly_one_bom(&[0xEF, 0xBB]));

        // A BOM-shaped byte sequence NOT at the start is not a leading BOM:
        // content starting with the raw bytes followed by BOM again is still
        // "no leading BOM".
        assert!(!has_exactly_one_bom(
            &[header.as_slice(), bom.as_slice()].concat()
        ));
    }
}
