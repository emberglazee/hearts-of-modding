//! Focus search-filter validation.
//!
//! `search_filters = { FOCUS_FILTER_X ... }` inside focus / shared_focus
//! blocks. A filter is not defined in any file — it is created dynamically
//! per focus tree from sprite `GFX_<FILTER_NAME>` + loc key `<FILTER_NAME>`.
//! Unknown filter names never appear in the game log; the filter just fails
//! to render in the focus-tree search menu (vanilla ships three typo'd
//! filters that silently no-op). Severity is therefore WARNING.
//!
//! Scope guard: only values prefixed `FOCUS_FILTER_` are validated. Anything
//! else inside the block is left alone (false negatives over false
//! positives). A non-base filter is accepted when a matching `GFX_<name>`
//! sprite exists anywhere in scanner data — that is exactly how mods define
//! their own filters.

use crate::parser::ast;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::ScopeStack;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

pub(crate) struct FocusSearchFilterRule;

impl ValidationRule for FocusSearchFilterRule {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        _pushed_scope: bool,
        diags: &mut Vec<Diagnostic>,
    ) {
        if !ass
            .key_text(ctx.source)
            .eq_ignore_ascii_case("search_filters")
        {
            return;
        }
        let ast::Value::Block(inner) = &ass.value.value else {
            return;
        };

        for entry in inner {
            let ast::Entry::Value(nv) = entry else {
                continue;
            };
            let ast::Value::String(span) = &nv.value else {
                continue;
            };
            let name = span.resolve(ctx.source);

            if !crate::data::focus_filters::looks_like_filter(name) {
                continue;
            }
            if crate::data::focus_filters::is_base_filter(name) {
                continue;
            }
            // Mod-defined filter: valid iff its GFX_<name> sprite exists.
            let sprite_name = format!("GFX_{}", name);
            if ctx.sprites.contains_key(sprite_name.as_str()) {
                continue;
            }

            diags.push(Diagnostic {
                range: ctx.range(&nv.range),
                severity: Some(DiagnosticSeverity::WARNING),
                code: Some(NumberOrString::String(
                    crate::validation::advanced_validation::UNKNOWN_FOCUS_SEARCH_FILTER.to_string(),
                )),
                message: format!(
                    "Unknown focus search filter '{}'. Base-game filters are FOCUS_FILTER_POLITICAL, FOCUS_FILTER_INDUSTRY, etc. A custom filter needs a 'GFX_{}' sprite and a localization key with this exact name, or it will not appear in the focus tree search.",
                    name,
                    // Sprite is GFX_ + the FULL filter name (wiki:
                    // "uses the sprite the same as its name but with GFX_
                    // inserted in the beginning").
                    name
                ),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::interner::InternedStr;
    use crate::data::layered_value::LayeredValue;
    use crate::parser::loc_parser::LocEntry;
    use crate::parser::parser;
    use crate::rules::visitor::{AstVisitor, walk_script};
    use crate::scanner::sprite_scanner::Sprite;
    use crate::utils::lsp_convert::RangeMapper;
    use dashmap::DashMap;

    fn run(source: &str, sprites: &[&str]) -> Vec<Diagnostic> {
        let (script, _) = parser::parse_script(source);
        let range_mapper = RangeMapper::new(&script.source);
        let loc: DashMap<InternedStr, LayeredValue<LocEntry>> = DashMap::new();
        loc.insert(
            InternedStr::from("SOME_KEY"),
            LayeredValue::new(LocEntry {
                key: InternedStr::from("SOME_KEY"),
                value: String::new(),
                range: ast::Range {
                    start_line: 0,
                    start_col: 0,
                    end_line: 0,
                    end_col: 0,
                },
                path: InternedStr::from("t.yml"),
                value_start_col: 0,
                version: None,
                version_range: None,
            }),
        );
        let sprites_map: DashMap<InternedStr, LayeredValue<Sprite>> = DashMap::new();
        for s in sprites {
            sprites_map.insert(
                InternedStr::from(*s),
                LayeredValue::new(Sprite {
                    name: (*s).to_string(),
                    texture_file: String::new(),
                    path: InternedStr::from("interface/t.gfx"),
                    range: ast::Range {
                        start_line: 0,
                        start_col: 0,
                        end_line: 0,
                        end_col: 0,
                    },
                }),
            );
        }

        let ctx = ValidationContext {
            uri: "/common/national_focus/a.txt",
            source: &script.source,
            range_mapper: &range_mapper,
            loc: &loc,
            scripted_triggers: &DashMap::new(),
            scripted_effects: &DashMap::new(),
            ideologies: &DashMap::new(),
            sub_ideologies: &DashMap::new(),
            traits: &DashMap::new(),
            sprites: &sprites_map,
            ideas: &DashMap::new(),
            characters: &DashMap::new(),
            provinces: &DashMap::new(),
            modifier_mappings: &DashMap::new(),
            ignored_loc_regex: &[],
            comments: &[],
            sound_effects: &DashMap::new(),
            country_tags: &DashMap::new(),
            tag_aliases: &DashMap::new(),
            buildings: &DashMap::new(),
            resources: &DashMap::new(),
            state_categories: &DashMap::new(),
            continents: &DashMap::new(),
            strategic_regions: &DashMap::new(),
            terrain_categories: &DashMap::new(),
            abilities: &DashMap::new(),
            ace_modifiers: &DashMap::new(),
            game_path: None,
            styling_enabled: false,
            scope_validation_enabled: false,
            workspace_roots: &[],
            unit_types: &DashMap::new(),
            event_targets: &DashMap::new(),
            event_namespaces: &DashMap::new(),
            events: &DashMap::new(),
            decisions: &DashMap::new(),
            decision_categories: &DashMap::new(),
        };
        let rules: Vec<Box<dyn ValidationRule>> = vec![Box::new(FocusSearchFilterRule)];
        let mut visitors: Vec<Box<dyn AstVisitor>> = Vec::new();
        let mut diags = Vec::new();
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

    fn h5010(diags: &[Diagnostic]) -> usize {
        diags
            .iter()
            .filter(|d| {
                d.code
                    == Some(NumberOrString::String(
                        crate::validation::advanced_validation::UNKNOWN_FOCUS_SEARCH_FILTER
                            .to_string(),
                    ))
            })
            .count()
    }

    #[test]
    fn base_filters_are_clean() {
        let src = r#"focus = {
	id = TAG_f
	search_filters = { FOCUS_FILTER_POLITICAL FOCUS_FILTER_INDUSTRY }
}"#;
        assert_eq!(h5010(&run(src, &[])), 0);
    }

    #[test]
    fn unknown_filter_without_sprite_warns() {
        let src = r#"focus = {
	id = TAG_f
	search_filters = { FOCUS_FILTER_MY_CUSTOM }
}"#;
        let diags = run(src, &[]);
        assert_eq!(h5010(&diags), 1);
        assert!(diags[0].message.contains("GFX_FOCUS_FILTER_MY_CUSTOM"));
    }

    #[test]
    fn unknown_filter_with_sprite_is_clean() {
        let src = r#"focus = {
	id = TAG_f
	search_filters = { FOCUS_FILTER_MY_CUSTOM }
}"#;
        assert_eq!(h5010(&run(src, &["GFX_FOCUS_FILTER_MY_CUSTOM"])), 0);
    }

    #[test]
    fn mixed_known_and_unknown_reports_only_unknown() {
        let src = r#"focus = {
	id = TAG_f
	search_filters = { FOCUS_FILTER_POLITICAL FOCUS_FILTER_TYPO_X }
}"#;
        assert_eq!(h5010(&run(src, &[])), 1);
    }

    #[test]
    fn case_insensitive_base_match() {
        let src = r#"focus = {
	id = TAG_f
	search_filters = { focus_filter_political }
}"#;
        assert_eq!(h5010(&run(src, &[])), 0);
    }

    /// Non-FOCUS_FILTER_ tokens are left alone entirely (FN over FP).
    #[test]
    fn non_filter_tokens_ignored() {
        let src = r#"focus = {
	id = TAG_f
	search_filters = { political_power some_random_word }
}"#;
        assert_eq!(h5010(&run(src, &[])), 0);
    }

    /// The rule must also fire inside shared_focus blocks (which store a
    /// different AST shape — the block is under a Value entry).
    #[test]
    fn fires_inside_shared_focus_trees() {
        let src = r#"focus_tree = {
	shared_focus = {
		id = TAG_shared
		search_filters = { FOCUS_FILTER_NOPE }
	}
}"#;
        assert_eq!(h5010(&run(src, &[])), 1);
    }
}
