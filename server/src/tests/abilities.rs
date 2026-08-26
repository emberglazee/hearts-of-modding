use crate::parser::parser;
use crate::rules::ValidationRule;
use crate::rules::abilities::AbilityRule;
use crate::rules::visitor::{AstVisitor, walk_script};
use crate::scope::scope::Scope;
use crate::test_support::TestCtx;
use crate::utils::lsp_convert::RangeMapper;
use tower_lsp_server::ls_types::{Diagnostic, NumberOrString};

// ---------------------------------------------------------------------------
// SECTION - Test runner
// ---------------------------------------------------------------------------

/// Run the ability rule visitor over a script and return all diagnostics.
fn run_ability_visitor(input: &str, uri: &str) -> Vec<Diagnostic> {
    let ctx_builder = TestCtx::new();
    let mut visitors: Vec<Box<dyn AstVisitor>> = vec![AbilityRule::visitor()];
    let rules: Vec<Box<dyn ValidationRule>> = vec![Box::new(AbilityRule)];
    let mut diags = Vec::new();

    let (script, _) = parser::parse_script(input);
    let range_mapper = RangeMapper::new(&script.source);
    let ctx = ctx_builder.build_context(uri, &script.source, &range_mapper);
    walk_script(
        &script.entries,
        &mut visitors,
        &rules,
        &ctx,
        &mut diags,
        Scope::Character,
        false,
    );

    diags
}

/// Filter diagnostics to only HOM3003 (required field) or HOM3004 (missing ai_will_do).
fn ability_field_diags(diags: &[Diagnostic]) -> Vec<&Diagnostic> {
    diags
        .iter()
        .filter(|d| {
            matches!(&d.code, Some(NumberOrString::String(c)) if c == "HOM3003" || c == "HOM3004")
        })
        .collect()
}

// ---------------------------------------------------------------------------
// !SECTION
// SECTION - Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── Positive: complete ability with deeply nested blocks ──────────────

    /// A full ability (name, desc, type, cost, duration, ai_will_do)
    /// with deeply nested property blocks (allowed, modifier, FROM,
    /// check_variable, set_temp_variable, one_time_effect, ...).
    /// The old depth-blind visitor flagged every inner block as a new
    /// "ability definition".  This test proves we now produce ZERO
    /// HOM3003 / HOM3004 false positives.
    const COMPLETE_ABILITY: &str = r#"ability = {
    force_attack = {
        name = "ABILITY_FORCE_ATTACK"
        desc = "ABILITY_FORCE_ATTACK_DESC"

        sound_effect = command_power_ability_offensive

        type = army_leader

        allowed = {
            is_leading_army_group = no
            is_border_war = no

            NOT = {
                OWNER = {
                    tag = TUL
                }
            }
        }

        ai_will_do = {
            factor = -1
            modifier = {
                FROM = {
                    has_war_support > 0.1
                    command_power > 1.5
                }
                check_variable = { num_units_offensive_combats > 6 }

                set_temp_variable = { temp = avg_offensive_combat_status }
                check_variable = { temp > 0.45 }
                check_variable = { ai_random > temp }

                add = 2
            }
        }

        cost = 0.22
        duration = 168

        one_time_effect = {
            add_temporary_buff_to_units = {
                combat_offense = 0.2
                combat_breakthrough = 0.25
                org_damage_multiplier = -1.0
                str_damage_multiplier = 0.6
                war_support_reduction_on_damage = 0.2
                cannot_retreat_while_attacking = 1.0

                days = 7
                tooltip = ABILITY_FORCE_ATTACK_TOOLTIP
            }
        }
    }
}"#;

    const TEST_URI: &str = "test:///common/abilities/test.txt";

    #[test]
    fn complete_ability_produces_no_false_positives() {
        let diags = run_ability_visitor(COMPLETE_ABILITY, TEST_URI);
        let field_diags = ability_field_diags(&diags);

        assert!(
            field_diags.is_empty(),
            "Expected zero HOM3003/HOM3004 diagnostics for a complete ability, got {}:\n{:#?}",
            field_diags.len(),
            field_diags,
        );
    }

    // ── Multiple abilities in one file ────────────────────────────────────

    const TWO_ABILITIES: &str = r#"ability = {
    last_stand = {
        name = "ABILITY_LAST_STAND"
        desc = "ABILITY_LAST_STAND_DESC"

        type = army_leader

        allowed = {
            is_leading_army_group = no
            is_border_war = no
        }

        cost = 0.22
        duration = 168

        one_time_effect = {
            add_temporary_buff_to_units = {
                combat_defense = 0.2
                combat_entrenchment = 0.25
                org_damage_multiplier = -1.0
                str_damage_multiplier = 0.6
                war_support_reduction_on_damage = 0.2
                cannot_retreat_while_defending = 1.0

                days = 7
                tooltip = ABILITY_LAST_STAND_TOOLTIP
            }
        }

        ai_will_do = {
            factor = -1
            modifier = {
                FROM = {
                    has_war_support > 0.4
                }

                check_variable = { num_units_defensive_combats > 6 }
                set_temp_variable = { temp = avg_defensive_combat_status }
                check_variable = { temp < 0.40 }
                check_variable = { ai_random > temp }
                add = 2
            }
        }
    }

    staff_office_plan = {
        name = "ABILITY_STAFF_OFFICE_PLAN"
        desc = "ABILITY_STAFF_OFFICE_PLAN_DESC"

        type = army_leader

        allowed = {
            is_border_war = no
        }

        cost = 0.12
        duration = 168

        unit_modifiers = {
            planning_speed = 4.0
        }

        ai_will_do = {
            factor = -1
            modifier = {
                FROM = { command_power > 2.0 }
                check_variable = { num_units > 6 }
                check_variable = { unit_ratio_ready_for_plan > 0.55 }
                check_variable = { avg_unit_planning_ratio < 0.4 }
                add = 2
            }
        }
    }
}"#;

    #[test]
    fn multiple_complete_abilities_no_false_positives() {
        let diags = run_ability_visitor(TWO_ABILITIES, TEST_URI);
        let field_diags = ability_field_diags(&diags);

        assert!(
            field_diags.is_empty(),
            "Expected zero HOM3003/HOM3004 for two complete abilities, got {}:\n{:#?}",
            field_diags.len(),
            field_diags,
        );
    }

    // ── Ability with unit_modifiers containing nested sub-blocks ──────────

    const ABILITY_WITH_UNIT_MODIFIER_SUBBLOCKS: &str = r#"ability = {
    siege_artillery = {
        name = "ABILITY_SIEGE_ARTILLERY"
        desc = "ABILITY_SIEGE_ARTILLERY_DESC"

        type = army_leader

        allowed = {
            has_ability = siege_artillery
            is_border_war = no
        }

        unit_modifiers = {
            fort = {
                attack = 0.2
            }
            fortification_damage = 2.0
            fortification_collateral_chance = 2.0
        }
        cost = 0.1
        duration = 168

        ai_will_do = {
            factor = -1
            modifier = {
                FROM = { command_power > 1.5 }

                set_temp_variable = { temp = num_units_offensive_combats }
                check_variable = { temp > 6 }
                set_temp_variable = { temp2 = num_units_offensive_combats_against@fort }
                divide_temp_variable = { temp2 = temp }
                check_variable = { temp2 > 0.5 }
                add = 2
            }
        }
    }
}"#;

    #[test]
    fn ability_with_unit_modifier_sub_blocks_no_false_positives() {
        let diags = run_ability_visitor(ABILITY_WITH_UNIT_MODIFIER_SUBBLOCKS, TEST_URI);
        let field_diags = ability_field_diags(&diags);

        assert!(
            field_diags.is_empty(),
            "Expected zero HOM3003/HOM3004 for siege_artillery with unit_modifiers/fort/divide_temp_variable sub-blocks, got {}:\n{:#?}",
            field_diags.len(),
            field_diags,
        );
    }

    // ── True positive: incomplete ability SHOULD produce diagnostics ──────

    const INCOMPLETE_ABILITY: &str = r#"ability = {
    orphan_ability = {
        # missing name, desc, cost, duration, type, ai_will_do
        allowed = { is_border_war = no }
    }
}"#;

    #[test]
    fn incomplete_ability_still_reported() {
        let diags = run_ability_visitor(INCOMPLETE_ABILITY, TEST_URI);
        let field_diags = ability_field_diags(&diags);

        assert!(
            !field_diags.is_empty(),
            "Incomplete ability should produce HOM3003/HOM3004 diagnostics"
        );

        let names: Vec<&str> = field_diags
            .iter()
            .filter_map(|d| d.message.contains("'name'").then_some("name"))
            .collect();
        assert!(!names.is_empty(), "Should flag missing 'name'");

        let descs: Vec<&str> = field_diags
            .iter()
            .filter_map(|d| d.message.contains("'desc'").then_some("desc"))
            .collect();
        assert!(!descs.is_empty(), "Should flag missing 'desc'");

        let costs: Vec<&str> = field_diags
            .iter()
            .filter_map(|d| d.message.contains("'cost'").then_some("cost"))
            .collect();
        assert!(!costs.is_empty(), "Should flag missing 'cost'");

        let ai: Vec<&str> = field_diags
            .iter()
            .filter_map(|d| d.message.contains("ai_will_do").then_some("ai_will_do"))
            .collect();
        assert!(!ai.is_empty(), "Should flag missing 'ai_will_do'");
    }

    // ── Mixed: one complete, one incomplete ability ───────────────────────

    const MIXED_ABILITIES: &str = r#"ability = {
    good_ability = {
        name = "GOOD"
        desc = "GOOD_DESC"
        type = army_leader
        cost = 0.1
        duration = 168
        ai_will_do = { factor = -1 }
    }
    bad_ability = {
        # missing everything except ai_will_do
        ai_will_do = { factor = -1 }
    }
}"#;

    #[test]
    fn mixed_abilities_only_flag_incomplete() {
        let diags = run_ability_visitor(MIXED_ABILITIES, TEST_URI);
        let field_diags = ability_field_diags(&diags);

        // bad_ability should trigger name, desc, cost, duration, type
        // good_ability should NOT trigger anything
        // So field_diags should have exactly 5 entries
        assert_eq!(
            field_diags.len(),
            5,
            "Expected 5 HOM3003 diagnostics (name, desc, cost, duration, type for bad_ability), got {}:\n{:#?}",
            field_diags.len(),
            field_diags,
        );

        // All should reference 'bad_ability'
        for d in &field_diags {
            assert!(
                d.message.contains("bad_ability"),
                "All diagnostics should reference 'bad_ability', got: {:?}",
                d.message,
            );
        }

        // No HOM3004 should appear (bad_ability has ai_will_do)
        let ai_missing: Vec<&&Diagnostic> = field_diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM3004".into())))
            .collect();
        assert!(
            ai_missing.is_empty(),
            "bad_ability HAS ai_will_do, so HOM3004 should NOT fire"
        );
    }

    // ── Empty ability container produces no diagnostics ────────────────────

    #[test]
    fn empty_ability_block_no_diagnostics() {
        let diags = run_ability_visitor("ability = { }", TEST_URI);
        let field_diags = ability_field_diags(&diags);
        assert!(
            field_diags.is_empty(),
            "Empty ability block should produce no diagnostics"
        );
    }

    // ── Regression: HOM004 scope check suppressed inside unit_modifiers ────

    /// An ability with unit_modifiers containing Country-scoped modifiers.
    /// The engine accepts any modifier inside unit_modifiers and routes them
    /// per-key — HOM004 must not fire here.
    const ABILITY_WITH_SCOPE_SENSITIVE_MODIFIERS: &str = r#"ability = {
    quinine = {
        type = army_leader
        name = "ABILITY_QUININE"
        desc = "ABILITY_QUININE_DESC"

        allowed = {
            is_border_war = no
            OWNER = { has_country_flag = has_quinine }
        }

        unit_modifiers = {
            no_supply_grace = 240
            attrition = -0.12
            sickness_chance = -1
            heat_attrition_factor = -0.35
        }

        one_time_effect = { supply_units = 240 }
        cost = 0.15
        duration = 240
        cooldown = 504
    }
}"#;

    /// Run the full validation suite (AbilityRule + V2ScopeRule) over ability
    /// input and return only HOM004 diagnostics.
    fn run_v2scope_test(input: &str, uri: &str) -> Vec<Diagnostic> {
        let ctx_builder = TestCtx::new();
        let mut visitors: Vec<Box<dyn AstVisitor>> = vec![AbilityRule::visitor()];
        let rules: Vec<Box<dyn ValidationRule>> = vec![
            Box::new(AbilityRule),
            Box::new(crate::rules::v2_scope::V2ScopeRule),
        ];
        let mut diags = Vec::new();

        let (script, _) = parser::parse_script(input);
        let range_mapper = RangeMapper::new(&script.source);
        let ctx = ctx_builder.build_context(uri, &script.source, &range_mapper);
        walk_script(
            &script.entries,
            &mut visitors,
            &rules,
            &ctx,
            &mut diags,
            Scope::Character,
            false,
        );

        diags
    }

    #[test]
    fn unit_modifiers_suppresses_hom004_for_country_scoped_modifiers() {
        let diags = run_v2scope_test(ABILITY_WITH_SCOPE_SENSITIVE_MODIFIERS, TEST_URI);

        // Filter to HOM004 diagnostics only
        let hom004: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| matches!(&d.code, Some(NumberOrString::String(c)) if c == "HOM004"))
            .collect();

        assert!(
            hom004.is_empty(),
            "Expected zero HOM004 diagnostics inside unit_modifiers with mixed-scope modifiers, got {}:\n{:#?}",
            hom004.len(),
            hom004,
        );
    }
}

// ---------------------------------------------------------------------------
// !SECTION
// ---------------------------------------------------------------------------
