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

    fn empty_characters()
    -> DashMap<InternedStr, LayeredValue<crate::scanner::character_scanner::Character>> {
        DashMap::new()
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

    fn tag_set(tags: &[&str]) -> HashSet<String> {
        tags.iter().map(|t| t.to_string()).collect()
    }

    /// RED is syntactically valid but reserved; GER is neither; NOTX is not a tag.
    /// `Scope::from_str` is not asserted here — it maps any 3-letter uppercase
    /// tag to Country by shape, so RED carries no special case there.
    #[test]
    fn red_is_valid_but_reserved() {
        assert!(
            country_scanner::is_valid_tag("RED"),
            "RED should be syntactically valid"
        );
        assert!(
            country_scanner::is_reserved_tag("RED"),
            "RED should be reserved"
        );
        assert!(country_scanner::is_valid_tag("GER"));
        assert!(!country_scanner::is_reserved_tag("GER"));
        assert!(!country_scanner::is_valid_tag("NOTX"));
    }

    /// `[RED.GetFlag]` is only flagged when the tag set is non-empty and does
    /// not contain RED: a defined RED passes, and an empty set means the scanner
    /// has not run yet, so nothing may be flagged on the strength of absence.
    #[test]
    fn red_loc_scope_follows_defined_tag_set() {
        for (tags, expect_flagged) in [
            (vec!["RED"], false),
            (vec![], false),
            (vec!["GER", "ENG"], true),
        ] {
            let e = entry("[RED.GetFlag] test");
            let diags = loc_parser::validate_loc_string(
                &e,
                &empty_targets(),
                &empty_scripted_locs(),
                &empty_color_codes(),
                &tag_set(&tags),
                &empty_characters(),
            );
            let invalid: Vec<_> = diags
                .iter()
                .filter(|d| d.code.as_deref() == Some("invalid_loc_scope"))
                .collect();
            assert_eq!(
                !invalid.is_empty(),
                expect_flagged,
                "tags {tags:?} should {} flag RED",
                if expect_flagged { "" } else { "not" }
            );
            if expect_flagged {
                assert!(invalid[0].message.contains("RED"));
            }
        }
    }

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

    fn hom4005(input: &str, uri: &str) -> Vec<tower_lsp_server::ls_types::Diagnostic> {
        run_country_tag_rule(input, uri)
            .into_iter()
            .filter(|d| {
                matches!(&d.code, Some(tower_lsp_server::ls_types::NumberOrString::String(c)) if c == "HOM4005")
            })
            .collect()
    }

    /// HOM4005 fires on a reserved tag defined in `common/country_tags` or named
    /// in a `history/countries` filename, and on nothing else — notably not on
    /// `oob = "RDM_648"`, whose value looks like a tag but is not one.
    #[test]
    fn reserved_tag_diagnostics() {
        let tags_uri = "file:///mod/common/country_tags/test.txt";
        let cases: &[(&str, &str, bool)] = &[
            (r#"RED = "countries/Redstone Mountain.txt""#, tags_uri, true),
            (r#"GER = "countries/Germany.txt""#, tags_uri, false),
            (
                "capital = 1",
                "file:///mod/history/countries/RED - Redstone Mountain.txt",
                true,
            ),
            (
                "capital = 1",
                "file:///mod/history/countries/GER - Germany.txt",
                false,
            ),
            (
                "oob = \"RDM_648\"\ncapital = 95\n",
                "file:///mod/history/countries/RDM - Redstone Mountain.txt",
                false,
            ),
        ];
        for (input, uri, expect) in cases {
            let found = hom4005(input, uri);
            assert_eq!(
                !found.is_empty(),
                *expect,
                "HOM4005 expected={expect} for {uri} with input {input:?}, got {found:?}"
            );
            if *expect {
                assert!(found[0].message.contains("RED"));
            }
        }

        // History keys that only look like tag definitions, at any case.
        for key in ["oob", "OOB", "log", "LOG", "tag", "num", "red"] {
            let found = hom4005(
                &format!("{} = \"something\"", key),
                "file:///mod/history/countries/RDM - Redstone Mountain.txt",
            );
            assert!(
                found.is_empty(),
                "history key `{key}` must not emit HOM4005, got {found:?}"
            );
        }
    }
}
