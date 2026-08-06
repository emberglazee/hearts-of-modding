#[cfg(test)]
mod tests {
    use crate::data::interner::InternedStr;
    use crate::data::layered_value::LayeredValue;
    use crate::parser::ast;
    use crate::parser::parser::parse_script;
    use crate::rules::ideas::IdeaRule;
    use crate::rules::visitor::walk_script;
    use crate::rules::{ValidationContext, ValidationRule};
    use crate::scanner::idea_scanner::Idea;
    use crate::scanner::province_scanner::Province;
    use crate::scanner::strategic_region_scanner::StrategicRegion;
    use crate::scope::scope::Scope;
    use crate::utils::lsp_convert::RangeMapper;
    use dashmap::DashMap;
    use regex::Regex;
    use tower_lsp_server::ls_types::{Diagnostic, NumberOrString};

    /// Build a minimal ValidationContext with empty scanner data.
    fn empty_ctx_with_ideas<'a>(source: &'a str, idea_names: &[&str]) -> ValidationContext<'a> {
        // Leak the mapper so the returned context (which borrows it) is valid;
        // matches the existing leak_map() pattern used for the other fields.
        let range_mapper: &'static RangeMapper = Box::leak(Box::new(RangeMapper::new(source)));
        let ideas: &'static DashMap<InternedStr, LayeredValue<Idea>> =
            Box::leak(Box::new(DashMap::new()));
        for name in idea_names {
            ideas.insert(
                InternedStr::from(*name),
                LayeredValue::new(Idea {
                    name: (*name).to_string(),
                    category: "country".to_string(),
                    picture: None,
                    path: InternedStr::from("test.txt"),
                    range: ast::Range {
                        start_line: 0,
                        start_col: 0,
                        end_line: 0,
                        end_col: 0,
                    },
                }),
            );
        }
        ValidationContext {
            uri: "test://ideas.txt",
            source,
            range_mapper,
            loc: leak_map(),
            scripted_triggers: leak_map(),
            scripted_effects: leak_map(),
            ideologies: leak_map(),
            sub_ideologies: leak_map(),
            traits: leak_map(),
            sprites: leak_map(),
            ideas,
            characters: leak_map(),
            provinces: Box::leak(Box::new(DashMap::<u32, Province>::new())),
            modifier_mappings: leak_map(),
            ignored_loc_regex: &[] as &[Regex],
            comments: &[] as &[(ast::ByteSpan, ast::Range)],
            sound_effects: leak_map(),
            country_tags: leak_map(),
            tag_aliases: leak_map(),
            buildings: leak_map(),
            resources: leak_map(),
            state_categories: leak_map(),
            continents: leak_map(),
            strategic_regions: Box::leak(Box::new(DashMap::<u32, StrategicRegion>::new())),
            terrain_categories: leak_map(),
            abilities: leak_map(),
            ace_modifiers: leak_map(),
            game_path: None,
            styling_enabled: false,
            scope_validation_enabled: false,
            workspace_roots: &[] as &[std::path::PathBuf],
            unit_types: leak_map(),
            event_targets: leak_map(),
            event_namespaces: leak_map(),
            events: leak_map(),
            decisions: leak_map(),
            decision_categories: leak_map(),
        }
    }

    fn leak_map<K: Eq + std::hash::Hash, V>() -> &'static DashMap<K, V> {
        Box::leak(Box::new(DashMap::new()))
    }

    /// Run only IdeaRule against the parsed script, returning diagnostics.
    fn run_idea_rules(source: &str) -> Vec<Diagnostic> {
        run_idea_rules_with_ideas(source, &[])
    }

    /// Like [`run_idea_rules`] but pre-populates the ideas map with `idea_names`.
    fn run_idea_rules_with_ideas(source: &str, idea_names: &[&str]) -> Vec<Diagnostic> {
        let (script, _) = parse_script(source);
        let ctx = empty_ctx_with_ideas(&script.source, idea_names);

        let rule: Box<dyn ValidationRule> = Box::new(IdeaRule);
        let rules: [Box<dyn ValidationRule>; 1] = [rule];
        let mut visitors: Vec<Box<dyn crate::rules::visitor::AstVisitor>> = vec![];
        let mut diags = vec![];

        walk_script(
            &script.entries,
            &mut visitors,
            &rules,
            &ctx,
            &mut diags,
            Scope::Global,
            false,
        );

        diags
    }

    // ── Sub-block keywords should NOT trigger missing-picture warnings ──

    #[test]
    fn test_on_add_does_not_trigger_picture_warning() {
        let diags = run_idea_rules(
            r#"ideas = {
                country = {
                    test_idea = {
                        picture = some_pic
                        on_add = {
                            add_stability = 0.1
                        }
                    }
                }
            }"#,
        );
        let subblock: Vec<_> = diags
            .iter()
            .filter(|d| d.message.contains("on_add"))
            .collect();
        assert!(
            subblock.is_empty(),
            "on_add triggered false picture warning: {:?}",
            subblock,
        );
    }

    #[test]
    fn test_multiple_subblocks_no_false_warnings() {
        let diags = run_idea_rules(
            r#"ideas = {
                country = {
                    multi_idea = {
                        picture = my_pic
                        cancel = {
                            NOT = { has_idea = multi_idea }
                        }
                        allowed_civil_war = { always = yes }
                        do_effect = { has_government = democratic }
                        visible = { always = yes }
                        on_add = { add_stability = 0.1 }
                        on_remove = { add_stability = -0.1 }
                    }
                }
            }"#,
        );
        let forbidden = [
            "on_add",
            "cancel",
            "allowed_civil_war",
            "do_effect",
            "visible",
            "on_remove",
            "modifier",
        ];
        for kw in &forbidden {
            let hits: Vec<_> = diags.iter().filter(|d| d.message.contains(kw)).collect();
            assert!(
                hits.is_empty(),
                "'{}' triggered false picture warning(s): {:?}",
                kw,
                hits,
            );
        }
    }

    #[test]
    fn test_orphaned_subblock_at_category_level_no_warning() {
        // Even if someone writes `on_add` at the `country = {}` level
        // (wrong level for a real mod), the picture check should not fire.
        let diags = run_idea_rules(
            r#"ideas = {
                country = {
                    some_idea = { picture = x modifier = { } }
                    on_add = {
                        add_stability = 0.1
                    }
                }
            }"#,
        );
        let subblock: Vec<_> = diags
            .iter()
            .filter(|d| d.message.contains("on_add"))
            .collect();
        assert!(
            subblock.is_empty(),
            "Orphaned on_add triggered picture warning: {:?}",
            subblock,
        );
    }

    #[test]
    fn test_allowed_subblock_does_not_trigger_picture_warning() {
        // Regression: `allowed = { ... }` and `available = { ... }` inside an
        // idea definition (irresistible for country-restrictive national
        // spirits) are transparent sub-blocks, not nested ideas — they must
        // never be flagged as ideas missing a picture. Mirrors the real
        // Hearts-Of-Minecraft `common/ideas/ENC.txt` `enc_fractured_1` shape.
        let diags = run_idea_rules(
            r#"ideas = {
                country = {
                    enc_fractured_1 = {
                        picture = IMP_rump_state
                        allowed = {
                            tag = ENC
                        }
                        allowed_civil_war = { always = yes }
                        modifier = {
                            political_power_gain = -0.4
                        }
                    }
                    enc_propaganda_1 = {
                        picture = FRA_scw_intervention_republicans_focus
                        available = { tag = ENC }
                        modifier = { stability_weekly = 0.005 }
                    }
                }
            }"#,
        );
        let forbidden = ["allowed", "available"];
        for kw in &forbidden {
            let hits: Vec<_> = diags.iter().filter(|d| d.message.contains(kw)).collect();
            assert!(
                hits.is_empty(),
                "'{}' triggered false picture warning(s): {:?}",
                kw,
                hits,
            );
        }
    }

    // ── Real ideas without pictures SHOULD get warnings ──

    #[test]
    fn test_idea_without_picture_gets_warning() {
        let diags = run_idea_rules(
            r#"ideas = {
                country = {
                    my_unpictured_idea = {
                        modifier = {
                            stability_factor = 0.1
                        }
                    }
                }
            }"#,
        );
        assert_eq!(
            diags.len(),
            1,
            "Expected 1 missing-picture warning, got {}: {:?}",
            diags.len(),
            diags,
        );
        assert!(diags[0].message.contains("my_unpictured_idea"));
        assert!(diags[0].message.contains("GFX_idea_my_unpictured_idea"));
        assert_eq!(
            diags[0].code,
            Some(NumberOrString::String("HOM4003".to_string())),
            "picture warning must carry the HOM4003 code"
        );
    }

    /// Regression: `NOT` inside an idea sub-block (cancel/available/...) is a
    /// transparent logical block that passes the parent Idea scope through —
    /// it must NOT be flagged as an idea missing a picture. Mirrors the real
    /// Hearts-Of-Minecraft `common/ideas/SPE.txt` `SPE_military_advisors`
    /// shape (cancel -> NOT -> is_in_faction_with).
    #[test]
    fn test_not_in_subblock_does_not_trigger_picture_warning() {
        let diags = run_idea_rules(
            r#"ideas = {
                country = {
                    SPE_military_advisors = {
                        picture = SPE_burden_of_hegemony
                        allowed_civil_war = {
                            always = yes
                        }
                        cancel = {
                            NOT = {
                                is_in_faction_with = SPE
                            }
                        }
                        removal_cost = -1
                        modifier = {
                            experience_gain_army_factor = 0.1
                        }
                    }
                }
            }"#,
        );
        let forbidden = ["NOT", "is_in_faction_with", "cancel"];
        for kw in &forbidden {
            let hits: Vec<_> = diags.iter().filter(|d| d.message.contains(kw)).collect();
            assert!(
                hits.is_empty(),
                "'{}' triggered false picture warning(s): {:?}",
                kw,
                hits,
            );
        }
    }

    #[test]
    fn test_idea_with_subblocks_no_picture_gets_only_idea_warning() {
        // When the idea itself has no picture, ONLY the idea name should
        // get a warning — NOT the sub-block keywords.
        let diags = run_idea_rules(
            r#"ideas = {
                country = {
                    no_pic_idea = {
                        modifier = { stability_factor = 0.1 }
                        on_add = { add_stability = 0.1 }
                        cancel = { always = no }
                    }
                }
            }"#,
        );
        let subblock: Vec<_> = diags
            .iter()
            .filter(|d| {
                d.message.contains("on_add")
                    || d.message.contains("cancel")
                    || d.message.contains("modifier")
            })
            .collect();
        assert!(
            subblock.is_empty(),
            "Sub-block keywords triggered false warnings: {:?}",
            subblock,
        );
        assert_eq!(
            diags.len(),
            1,
            "Expected exactly 1 warning (for no_pic_idea), got {}: {:?}",
            diags.len(),
            diags,
        );
    }

    #[test]
    fn test_idea_with_picture_no_warning() {
        let diags = run_idea_rules(
            r#"ideas = {
                country = {
                    my_idea = {
                        picture = my_pic
                        modifier = { stability_factor = 0.1 }
                    }
                }
            }"#,
        );
        assert!(
            diags.is_empty(),
            "Expected no diagnostics for idea with picture, got: {:?}",
            diags,
        );
    }

    // ── Idea category names should NOT trigger picture warnings ──

    #[test]
    fn test_law_category_no_picture_warning() {
        // Category names like `economy` and `trade_laws` are containers
        // for actual ideas — they don't need a `picture` field.
        let diags = run_idea_rules(
            r#"ideas = {
                economy = {
                    law = yes
                    use_list_view = yes
                    skulk_economy = {
                        picture = GFX_idea_skulk_economy
                        modifier = { stability_factor = 0.1 }
                    }
                    subsistence_economy = {
                        picture = GFX_idea_subsistence_economy
                        modifier = { stability_factor = -0.1 }
                    }
                }
            }"#,
        );
        let category_hits: Vec<_> = diags
            .iter()
            .filter(|d| d.message.contains("economy"))
            .collect();
        assert!(
            category_hits.is_empty(),
            "Category name 'economy' triggered false picture warning(s): {:?}",
            category_hits,
        );
    }

    #[test]
    fn test_designer_category_no_picture_warning() {
        let diags = run_idea_rules(
            r#"ideas = {
                my_designers = {
                    designer = yes
                    tank_designer = {
                        picture = GFX_idea_tank_designer
                        modifier = { research_bonus = { armor = 0.1 } }
                    }
                }
            }"#,
        );
        let category_hits: Vec<_> = diags
            .iter()
            .filter(|d| d.message.contains("my_designers"))
            .collect();
        assert!(
            category_hits.is_empty(),
            "Category name 'my_designers' triggered false picture warning(s): {:?}",
            category_hits,
        );
    }

    #[test]
    fn test_idea_in_category_missing_picture_still_warns() {
        // The picture check should still fire on actual ideas inside a category
        // that lack a `picture` field.
        let diags = run_idea_rules(
            r#"ideas = {
                economy = {
                    law = yes
                    unpictured_idea = {
                        modifier = { stability_factor = 0.1 }
                    }
                }
            }"#,
        );
        assert_eq!(
            diags.len(),
            1,
            "Expected 1 warning for unpictured idea inside category, got {}: {:?}",
            diags.len(),
            diags,
        );
        assert!(diags[0].message.contains("unpictured_idea"));
        assert!(diags[0].message.contains("GFX_idea_unpictured_idea"));
    }

    #[test]
    fn test_multiple_categories_one_without_pics() {
        // Two categories: one with pictured ideas, one with unpictured.
        // Only the unpictured idea in the second category should warn.
        let diags = run_idea_rules(
            r#"ideas = {
                laws_a = {
                    law = yes
                    good_idea = {
                        picture = x
                        modifier = { }
                    }
                    bad_idea = { modifier = { } }
                }
                laws_b = {
                    law = yes
                    fine_idea = {
                        picture = y
                        modifier = { }
                    }
                }
            }"#,
        );
        let bad: Vec<_> = diags
            .iter()
            .filter(|d| d.message.contains("bad_idea"))
            .collect();
        assert_eq!(
            bad.len(),
            1,
            "Expected exactly 1 warning for bad_idea, got {:?}",
            bad,
        );
        let category_hits: Vec<_> = diags
            .iter()
            .filter(|d| d.message.contains("laws_a") || d.message.contains("laws_b"))
            .collect();
        assert!(
            category_hits.is_empty(),
            "Category names triggered false warnings: {:?}",
            category_hits,
        );
    }

    // ── Case-insensitive idea references ───────────────────────────────

    /// `add_ideas = SPE_pale_unit_idea` referencing an idea defined as
    /// `SPE_PALE_unit_idea` (case mismatch) works in the engine (verified
    /// empirically via probe mod) — it must NOT fire "Unknown idea". Instead
    /// it gets a HINT about the casing inconsistency (HOM4004).
    #[test]
    fn test_case_mismatched_idea_reference_hint() {
        let diags = run_idea_rules_with_ideas(
            r#"focus = {
                completion_reward = {
                    add_ideas = SPE_pale_unit_idea
                }
            }"#,
            &["SPE_PALE_unit_idea"],
        );
        let unknown: Vec<_> = diags
            .iter()
            .filter(|d| d.message.contains("Unknown idea"))
            .collect();
        assert!(
            unknown.is_empty(),
            "case-mismatched idea reference must not be flagged as unknown, got: {:?}",
            unknown,
        );
        let hints: Vec<_> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM4004".to_string())))
            .collect();
        assert_eq!(
            hints.len(),
            1,
            "Expected 1 HOM4004 case-mismatch hint, got {:?}",
            diags,
        );
        assert_eq!(
            hints[0].severity,
            Some(tower_lsp_server::ls_types::DiagnosticSeverity::HINT)
        );
        // Data must carry the canonical casing for the code-action fix.
        assert_eq!(
            hints[0].data.as_ref().and_then(|v| v.as_str()),
            Some("SPE_PALE_unit_idea"),
            "HOM4004 diagnostic must store the canonical casing in data"
        );
    }

    /// An exact-case idea reference is silent (canonical — no hint, no warning).
    #[test]
    fn test_exact_idea_reference_silent() {
        let diags = run_idea_rules_with_ideas(
            r#"focus = {
                completion_reward = {
                    add_ideas = SPE_PALE_unit_idea
                }
            }"#,
            &["SPE_PALE_unit_idea"],
        );
        assert!(
            diags.is_empty(),
            "exact-case idea reference must be silent, got: {:?}",
            diags,
        );
    }

    /// A genuinely unknown idea (no matching definition in ANY case) still
    /// fires "Unknown idea".
    #[test]
    fn test_truly_unknown_idea_still_flagged() {
        let diags = run_idea_rules_with_ideas(
            r#"focus = {
                completion_reward = {
                    add_ideas = TOTALLY_MISSING_IDEA
                }
            }"#,
            &["SPE_PALE_unit_idea"],
        );
        let unknown: Vec<_> = diags
            .iter()
            .filter(|d| d.message.contains("Unknown idea"))
            .collect();
        assert_eq!(
            unknown.len(),
            1,
            "Expected 1 Unknown idea warning, got {:?}",
            unknown,
        );
    }
}
