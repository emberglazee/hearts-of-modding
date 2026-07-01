#[cfg(test)]
mod tests {
    use crate::parser::ast;
    use crate::parser::parser::parse_script;
    use crate::rules::ideas::IdeaRule;
    use crate::rules::visitor::walk_script;
    use crate::rules::{ValidationContext, ValidationRule};
    use crate::scanner::province_scanner::Province;
    use crate::scanner::strategic_region_scanner::StrategicRegion;
    use crate::scope::scope::Scope;
    use dashmap::DashMap;
    use regex::Regex;
    use tower_lsp_server::ls_types::Diagnostic;

    /// Build a minimal ValidationContext with empty scanner data.
    fn empty_ctx(source: &str) -> ValidationContext<'_> {
        ValidationContext {
            uri: "test://ideas.txt",
            source,
            loc: leak_map(),
            scripted_triggers: leak_map(),
            scripted_effects: leak_map(),
            ideologies: leak_map(),
            sub_ideologies: leak_map(),
            traits: leak_map(),
            sprites: leak_map(),
            ideas: leak_map(),
            characters: leak_map(),
            provinces: Box::leak(Box::new(DashMap::<u32, Province>::new())),
            modifier_mappings: leak_map(),
            ignored_loc_regex: &[] as &[Regex],
            comments: &[] as &[(ast::ByteSpan, ast::Range)],
            sound_effects: leak_map(),
            country_tags: leak_map(),
            buildings: leak_map(),
            resources: leak_map(),
            state_categories: leak_map(),
            continents: leak_map(),
            strategic_regions: Box::leak(Box::new(DashMap::<u32, StrategicRegion>::new())),
            terrain_categories: leak_map(),
            abilities: leak_map(),
            game_path: None,
            styling_enabled: false,
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
        let (script, _) = parse_script(source);
        let ctx = empty_ctx(&script.source);

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
}
