#[cfg(test)]
mod red_tests {
    use crate::data::interner::InternedStr;
    use crate::data::layered_value::LayeredValue;
    use crate::parser::ast::Range;
    use crate::parser::loc_parser::{self, LocEntry};
    use crate::scanner::country_scanner;
    use dashmap::DashMap;
    use std::collections::HashSet;

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

    fn entry(value: &str) -> LocEntry {
        LocEntry {
            key: InternedStr::from("test"),
            value: value.to_string(),
            range: Range {
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: value.len() as u32,
            },
            path: InternedStr::from("test.yml"),
            value_start_col: 0,
            version: None,
            version_range: None,
        }
    }

    #[test]
    fn red_is_valid_but_reserved() {
        assert!(
            country_scanner::is_valid_tag("RED"),
            "RED should be syntactically valid after fix"
        );
        assert!(
            country_scanner::is_reserved_tag("RED"),
            "RED should be reserved"
        );
        assert!(country_scanner::is_valid_tag("GER"));
        assert!(!country_scanner::is_reserved_tag("GER"));
        assert!(!country_scanner::is_valid_tag("NOTX"));
        assert!(!country_scanner::is_reserved_tag("GER"));
    }

    #[test]
    fn red_loc_recognized_when_defined() {
        let e = entry("[RED.GetFlag] test");
        let mut tags = HashSet::new();
        tags.insert("RED".to_string());
        let diags = loc_parser::validate_loc_string(
            &e,
            &empty_targets(),
            &empty_scripted_locs(),
            &empty_color_codes(),
            &tags,
        );
        let invalid: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_deref() == Some("invalid_loc_scope"))
            .collect();
        assert!(
            invalid.is_empty(),
            "RED with defined tag should NOT be flagged, got: {:?}",
            invalid
        );
    }

    #[test]
    fn red_loc_with_empty_set_passes() {
        // Empty set = scanner not yet run, should not flag reserved tags
        let e = entry("[RED.GetFlag] test");
        let tags = HashSet::new();
        let diags = loc_parser::validate_loc_string(
            &e,
            &empty_targets(),
            &empty_scripted_locs(),
            &empty_color_codes(),
            &tags,
        );
        let invalid: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_deref() == Some("invalid_loc_scope"))
            .collect();
        assert!(
            invalid.is_empty(),
            "RED with empty set (fallback) should not be flagged, got {:?}",
            invalid
        );
    }

    #[test]
    fn red_loc_flagged_when_not_defined_but_set_nonempty() {
        let e = entry("[RED.GetFlag] test");
        let mut tags = HashSet::new();
        tags.insert("GER".to_string());
        tags.insert("ENG".to_string());
        let diags = loc_parser::validate_loc_string(
            &e,
            &empty_targets(),
            &empty_scripted_locs(),
            &empty_color_codes(),
            &tags,
        );
        let invalid: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_deref() == Some("invalid_loc_scope"))
            .collect();
        // RED is valid tag but not in set, so should be flagged when set is non-empty and doesn't contain RED
        assert!(
            !invalid.is_empty(),
            "RED not in set should be flagged when set non-empty, got none"
        );
        assert!(invalid[0].message.contains("RED"));
    }

    #[test]
    fn scope_red_is_country() {
        let s = crate::scope::scope::Scope::from_str("RED");
        assert_eq!(
            s,
            crate::scope::scope::Scope::Country,
            "RED should be Country scope after fix"
        );
        let s2 = crate::scope::scope::Scope::from_str("GER");
        assert_eq!(s2, crate::scope::scope::Scope::Country);
    }

    // ── HOM4005 reserved tag diagnostics ───────────────────────────────
    fn run_country_tag_rule(input: &str, uri: &str) -> Vec<tower_lsp_server::ls_types::Diagnostic> {
        use crate::parser::parser;
        use crate::rules::country_tags::CountryTagRule;
        use crate::rules::{
            ValidationRule,
            visitor::{AstVisitor, walk_script},
        };
        use crate::scope::scope::Scope;
        use crate::test_support::TestCtx;
        use crate::utils::lsp_convert::RangeMapper;
        let mut visitors: Vec<Box<dyn AstVisitor>> = vec![];
        let rules: Vec<Box<dyn ValidationRule>> = vec![Box::new(CountryTagRule)];
        let mut diags = Vec::new();
        let (script, _) = parser::parse_script(input);
        let mapper = RangeMapper::new(&script.source);
        let binding = TestCtx::new();
        let ctx = binding.build_context(uri, &script.source, &mapper);
        walk_script(
            &script.entries,
            &mut visitors,
            &rules,
            &ctx,
            &mut diags,
            Scope::Global,
            false,
        );
        // Also run check_block (handles history filename case)
        for r in &rules {
            r.check_block(&script.entries, &ctx, &mut diags);
        }
        diags
    }

    #[test]
    fn reserved_tag_in_country_tags_file_emits_hom4005() {
        let input = r#"RED = "countries/Redstone Mountain.txt""#;
        let uri = "file:///mod/common/country_tags/test.txt";
        let diags = run_country_tag_rule(input, uri);
        let hom4005: Vec<_> = diags.iter().filter(|d| matches!(&d.code, Some(tower_lsp_server::ls_types::NumberOrString::String(c)) if c == "HOM4005")).collect();
        assert!(
            !hom4005.is_empty(),
            "RED definition in country_tags should emit HOM4005, got {:?}",
            diags
        );
        assert!(hom4005[0].message.contains("RED"));
    }

    #[test]
    fn non_reserved_tag_no_hom4005() {
        let input = r#"GER = "countries/Germany.txt""#;
        let uri = "file:///mod/common/country_tags/test.txt";
        let diags = run_country_tag_rule(input, uri);
        let hom4005: Vec<_> = diags.iter().filter(|d| matches!(&d.code, Some(tower_lsp_server::ls_types::NumberOrString::String(c)) if c == "HOM4005")).collect();
        assert!(
            hom4005.is_empty(),
            "GER should not emit HOM4005, got {:?}",
            hom4005
        );
    }

    #[test]
    fn reserved_tag_in_history_filename_emits_hom4005() {
        let input = r#"capital = 1"#;
        let uri = "file:///mod/history/countries/RED - Redstone Mountain.txt";
        let diags = run_country_tag_rule(input, uri);
        let hom4005: Vec<_> = diags.iter().filter(|d| matches!(&d.code, Some(tower_lsp_server::ls_types::NumberOrString::String(c)) if c == "HOM4005")).collect();
        assert!(
            !hom4005.is_empty(),
            "history filename RED should emit HOM4005, got {:?}",
            diags
        );
    }

    #[test]
    fn non_reserved_history_filename_no_hom4005() {
        let input = r#"capital = 1"#;
        let uri = "file:///mod/history/countries/GER - Germany.txt";
        let diags = run_country_tag_rule(input, uri);
        let hom4005: Vec<_> = diags.iter().filter(|d| matches!(&d.code, Some(tower_lsp_server::ls_types::NumberOrString::String(c)) if c == "HOM4005")).collect();
        assert!(
            hom4005.is_empty(),
            "GER history should not emit HOM4005, got {:?}",
            hom4005
        );
    }

    #[test]
    fn oob_key_in_history_does_not_emit_hom4005_regression() {
        // Regression: `oob = "RDM_648"` in history/countries/RDM - ...txt
        // was flagged as HOM4005 OOB because the check incorrectly treated
        // any `oob` assignment inside history as a tag definition.
        let input = "oob = \"RDM_648\"\ncapital = 95\n";
        let uri = "file:///mod/history/countries/RDM - Redstone Mountain.txt";
        let diags = run_country_tag_rule(input, uri);
        let hom4005: Vec<_> = diags
            .iter()
            .filter(|d| matches!(&d.code, Some(tower_lsp_server::ls_types::NumberOrString::String(c)) if c == "HOM4005"))
            .collect();
        assert!(
            hom4005.is_empty(),
            "history `oob =` must NOT emit HOM4005 (RDM regression), got {:?}",
            hom4005
        );
    }

    #[test]
    fn history_keys_like_oob_not_flagged_even_with_lowercase() {
        for key in ["oob", "OOB", "log", "LOG", "tag", "num", "red"] {
            let input = format!("{} = \"something\"", key);
            let uri = "file:///mod/history/countries/RDM - Redstone Mountain.txt";
            let diags = run_country_tag_rule(&input, uri);
            let hom4005: Vec<_> = diags
                .iter()
                .filter(|d| matches!(&d.code, Some(tower_lsp_server::ls_types::NumberOrString::String(c)) if c == "HOM4005"))
                .collect();
            assert!(
                hom4005.is_empty(),
                "history key `{}` must not emit HOM4005, got {:?}",
                key,
                hom4005
            );
        }
    }
}
