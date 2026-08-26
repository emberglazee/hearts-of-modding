use crate::parser::ast;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::ScopeStack;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Checks that string values assigned to `name`, `desc`, `text`,
/// and `title` keys have corresponding localization entries.
///
/// Uses heuristics to avoid flagging literals (space-containing strings,
/// capitalized non-all-caps strings, pure numbers) and respects the
/// `# ignore` comment suppression and `ignored_loc_regex` config.
///
/// Files under `common/scripted_localisation/` are skipped entirely: there,
/// `name = X` is the *identifier* of a `defined_text` block (used to refer to
/// it as `[X]` in loc strings) and `text` is always a block — the actual loc
/// references live in `localization_key` inside those blocks. Flagging the
/// identifier as "missing localization key" is a false positive.
pub(crate) struct LocalizationRule;

impl ValidationRule for LocalizationRule {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        _pushed_scope: bool,
        diags: &mut Vec<Diagnostic>,
    ) {
        let uri_lower = ctx.uri.to_ascii_lowercase();
        let uri_norm = crate::scanner::incremental_scanner::normalize_path_str(&uri_lower);
        if uri_norm.contains("/common/scripted_localisation/") {
            return;
        }

        let key_lower = ass.key_text(ctx.source).to_ascii_lowercase();
        if key_lower != "name" && key_lower != "desc" && key_lower != "text" && key_lower != "title"
        {
            return;
        }

        let Some(val) = ass.value.value.as_str(ctx.source) else {
            return;
        };

        let mut should_flag = true;

        // 1. Basic heuristics: space, empty, all-numeric → literal
        if val.contains(' ') || val.is_empty() || val.chars().all(|c| c.is_numeric()) {
            should_flag = false;
        }

        // 2. Casing heuristic: starts with uppercase but isn't all-caps →
        //    likely a literal word ("John", "Margaret"). NEVER applied to
        //    key-shaped values: TAG_snake_case and TAG.1.title identifiers
        //    routinely start with the uppercase country tag while containing
        //    lowercase, so suppressing them silenced HOM005 for the majority
        //    of standard mod localization references (measured: 21.4k of
        //    22.5k suppressed bare values across vanilla are underscore- or
        //    dot-shaped keys, vs 1.1k literal person/place names).
        if should_flag
            && !val.contains('_')
            && !val.contains('.')
            && val.chars().next().is_some_and(|c| c.is_uppercase())
        {
            let all_caps = val.chars().all(|c| !c.is_lowercase());
            if !all_caps {
                should_flag = false;
            }
        }

        // 3. Comment suppression (# ignore on same line)
        if should_flag {
            for (comment_text, range) in ctx.comments {
                if range.start_line == ass.key_range.start_line {
                    if comment_text
                        .resolve(ctx.source)
                        .to_ascii_lowercase()
                        .contains("ignore")
                    {
                        should_flag = false;
                        break;
                    }
                }
            }
        }

        if should_flag {
            // Skip if localization hasn't been scanned yet
            if !ctx.loc.is_empty() && !ctx.loc.contains_key(val) {
                // Double-check: the key might be stored with a version suffix like ":0"
                // Instead of iterating the entire 162k-entry DashMap (which is O(N) per
                // missing key), check a few common version numbers directly.
                let version_suffixed = (0..=5).any(|v| {
                    let target = format!("{}:{}", val, v);
                    ctx.loc.contains_key(target.as_str())
                });
                if !version_suffixed {
                    // Final check against regex
                    let is_regex_ignored = ctx.ignored_loc_regex.iter().any(|re| re.is_match(val));

                    if !is_regex_ignored {
                        diags.push(Diagnostic {
                            range: ctx.range(&ass.value.range),
                            severity: Some(DiagnosticSeverity::HINT),
                            message: format!(
                                "Missing localization key: '{}' (or literal name)",
                                val
                            ),
                            code: Some(NumberOrString::String(
                                crate::validation::advanced_validation::MISSING_LOCALIZATION
                                    .to_string(),
                            )),
                            source: Some("Hearts of Modding".to_string()),
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::parser::parser;
    use crate::rules::visitor::{AstVisitor, walk_script};
    use crate::utils::lsp_convert::RangeMapper;

    /// Run `LocalizationRule` over a single source + URI with a loc map that
    /// contains the given known keys (so the `!loc.is_empty()` guard passes
    /// and the missing-key check actually runs). All other maps are empty.
    fn run_with_loc(source: &str, uri: &str, known_keys: &[&str]) -> Vec<Diagnostic> {
        let (script, _) = parser::parse_script(source);
        let range_mapper = RangeMapper::new(&script.source);
        let mut builder = crate::test_support::TestCtx::new();
        for k in known_keys {
            builder = builder.with_loc_key(k);
        }
        let ctx = builder.build_context(uri, &script.source, &range_mapper);
        let rule = LocalizationRule;
        let rules: Vec<Box<dyn ValidationRule>> = vec![Box::new(rule)];
        let mut diags = Vec::new();
        let mut visitors: Vec<Box<dyn AstVisitor>> = Vec::new();
        walk_script(
            &script.entries,
            &mut visitors,
            &rules,
            &ctx,
            &mut diags,
            crate::scope::scope::Scope::Global,
            false,
        );
        diags
    }

    /// A `name = X` inside a `defined_text` block in a scripted_localisation
    /// file is an *identifier* (referenced as `[X]` in loc strings), not a
    /// loc-key needing its own entry. HOM005 must NOT fire (regression:
    /// `scripted_formed_great_yuon` false positive).
    #[test]
    fn test_scripted_localisation_name_not_flagged() {
        let source = r#"defined_text = {
	name = scripted_formed_great_yuon
	text = {
		trigger = {
			has_cosmetic_tag = GHD_cosmetic
		}
		localization_key = GHD_cosmetic_formed_name
	}
}"#;
        // The identifier has no loc entry — a working vanilla-pattern setup.
        let diags = run_with_loc(
            source,
            "/common/scripted_localisation/hom_generic_scripted_localisation.txt",
            &["SOME_OTHER_KEY"],
        );
        let h005: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM005".to_string())))
            .collect();
        assert!(
            h005.is_empty(),
            "scripted_localisation name must not be flagged as missing loc key, got: {:?}",
            h005
        );
    }

    /// A `name = X` in a NON-scripted_localisation file still fires HOM005.
    #[test]
    fn test_regular_name_still_flagged() {
        let source = r#"country = {
	name = MY_COUNTRY_PLACEHOLDER_KEY
}"#;
        let diags = run_with_loc(source, "/common/countries/a.txt", &["SOME_OTHER_KEY"]);
        let h005: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM005".to_string())))
            .collect();
        assert_eq!(
            h005.len(),
            1,
            "Expected HOM005 for a real (missing) loc key"
        );
    }

    /// Regression (external review): the casing heuristic suppressed
    /// TAG_snake_case keys because the country tag is uppercase while the
    /// rest is lowercase. A missing `TAG_four_year_plan` must fire HOM005.
    #[test]
    fn test_tag_snake_case_key_not_suppressed_by_casing_heuristic() {
        let source = r#"focus = {
	id = TAG_four_year_plan
	name = TAG_four_year_plan
}"#;
        let diags = run_with_loc(source, "/common/national_focus/a.txt", &["SOME_OTHER_KEY"]);
        let h005: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM005".to_string())))
            .collect();
        assert_eq!(
            h005.len(),
            1,
            "underscore-keyed values start with an uppercase tag but are loc keys, not literals: {:?}",
            h005
        );
    }

    /// Same for dotted event-title conventions: `title = TAG.8.title` starts
    /// uppercase (the tag) and contains lowercase — it is a key, not prose.
    #[test]
    fn test_dotted_event_title_key_not_suppressed_by_casing_heuristic() {
        let source = r#"country_event = {
	id = tag.8
	title = TAG.8.title
	desc = TAG.8.desc
}"#;
        let diags = run_with_loc(source, "/events/a.txt", &["SOME_OTHER_KEY"]);
        let h005: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM005".to_string())))
            .collect();
        assert_eq!(h005.len(), 2, "both dotted keys are missing: {:?}", h005);
    }

    /// The literal-word suppression must SURVIVE for underscore-free words:
    /// vanilla carries ~1.1k of them (`add_ace = { name = Margaret }`,
    /// common/names data). No underscore/dot → still treated as literal.
    #[test]
    fn test_literal_words_still_suppressed() {
        let source = r#"add_ace = {
	name = Margaret
	surname = Gee
}
character = {
	name = John
}"#;
        let diags = run_with_loc(source, "/common/characters/a.txt", &["SOME_OTHER_KEY"]);
        let h005: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM005".to_string())))
            .collect();
        assert!(
            h005.is_empty(),
            "literal person names without underscores stay suppressed, got: {:?}",
            h005
        );
    }
}
