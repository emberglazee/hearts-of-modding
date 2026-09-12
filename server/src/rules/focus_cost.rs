//! Focus `cost` (completion time in weeks) validation.
//!
//! `cost = N` inside `focus` / `shared_focus` / `joint_focus` sets how long
//! the focus takes (1 = one week by default, decimals round down to a whole
//! day). The engine integer-overflows beyond `131762457.66935` weeks
//! (~922,337,203 days), so anything above it is an ERROR, not a longer focus.
//!
//! Scope guard: only `Scope::NationalFocus` is checked. Nested effect blocks
//! (`completion_reward`, `select_effect`, …) push `Scope::Country`, and
//! decision/idea `cost` keys live in other scopes — exact `cost` match plus
//! the scope check keeps those silent (false negatives over false positives).

use crate::parser::ast;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::{Scope, ScopeStack};
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Engine maximum for focus `cost`, in weeks.
pub(crate) const MAX_FOCUS_COST_WEEKS: f64 = 131762457.66935;

/// Whole-day equivalent of the max (`floor(weeks * 7)`), for the message.
const MAX_FOCUS_COST_DAYS: &str = "922,337,203";

pub(crate) struct FocusCostRule;

impl ValidationRule for FocusCostRule {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        scope: &ScopeStack,
        _pushed_scope: bool,
        diags: &mut Vec<Diagnostic>,
    ) {
        if !ass.key_text(ctx.source).eq_ignore_ascii_case("cost") {
            return;
        }
        if scope.current() != Scope::NationalFocus {
            return;
        }
        let ast::Value::Number(n) = &ass.value.value else {
            return;
        };
        if *n <= MAX_FOCUS_COST_WEEKS {
            return;
        }

        diags.push(Diagnostic {
            range: ctx.range(&ass.value.range),
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String(
                crate::validation::advanced_validation::FOCUS_COST_EXCEEDS_MAX.to_string(),
            )),
            message: format!(
                "Focus cost {} weeks exceeds the engine maximum of {} weeks (~{} days) — beyond this the value integer-overflows.",
                n, MAX_FOCUS_COST_WEEKS, MAX_FOCUS_COST_DAYS
            ),
            source: Some("Hearts of Modding".to_string()),
            ..Default::default()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::parser::parser;
    use crate::rules::visitor::{AstVisitor, walk_script};
    use crate::scope::scope::Scope as TestScope;
    use crate::utils::lsp_convert::RangeMapper;

    fn run(source: &str) -> Vec<Diagnostic> {
        let (script, _) = parser::parse_script(source);
        let range_mapper = RangeMapper::new(&script.source);
        let test_ctx = crate::test_support::TestCtx::new();
        let ctx = test_ctx.build_context(
            "/common/national_focus/a.txt",
            &script.source,
            &range_mapper,
        );

        let rules: Vec<Box<dyn ValidationRule>> = vec![Box::new(FocusCostRule)];
        let mut visitors: Vec<Box<dyn AstVisitor>> = Vec::new();
        let mut diags = Vec::new();
        walk_script(
            &script.entries,
            &mut visitors,
            &rules,
            &ctx,
            &mut diags,
            TestScope::Global,
            false,
        );
        diags
    }

    fn h5011(diags: &[Diagnostic]) -> usize {
        diags
            .iter()
            .filter(|d| {
                d.code
                    == Some(NumberOrString::String(
                        crate::validation::advanced_validation::FOCUS_COST_EXCEEDS_MAX.to_string(),
                    ))
            })
            .count()
    }

    #[test]
    fn test_normal_cost_silent() {
        let src = "focus = { id = TST_a cost = 10 }";
        assert_eq!(h5011(&run(src)), 0);
    }

    #[test]
    fn test_max_cost_silent() {
        let src = "focus = { id = TST_a cost = 131762457.66935 }";
        assert_eq!(h5011(&run(src)), 0);
    }

    #[test]
    fn test_overflow_cost_fires() {
        let src = "focus = { id = TST_a cost = 131762457.66936 }";
        let diags = run(src);
        assert_eq!(h5011(&diags), 1);
        assert_eq!(diags[0].severity, Some(DiagnosticSeverity::ERROR));
        assert!(diags[0].message.contains("131762457.66935"));
        assert!(diags[0].message.contains("922,337,203"));
    }

    #[test]
    fn test_absurd_cost_fires() {
        let src = "focus = { id = TST_a cost = 999999999999 }";
        let diags = run(src);
        assert_eq!(h5011(&diags), 1);
        assert!(diags[0].message.contains("131762457.66935"));
    }

    #[test]
    fn test_shared_and_joint_focus_checked() {
        assert_eq!(
            h5011(&run("shared_focus = { id = TST_s cost = 200000000 }")),
            1
        );
        assert_eq!(
            h5011(&run("joint_focus = { id = TST_j cost = 200000000 }")),
            1
        );
    }

    #[test]
    fn test_nested_cost_not_flagged() {
        // completion_reward pushes Country scope — not a focus duration.
        let src =
            "focus = { id = TST_a cost = 10 completion_reward = { add_political_power = 50 } }";
        assert_eq!(h5011(&run(src)), 0);
    }

    #[test]
    fn test_decision_cost_not_flagged() {
        // Same key, different scope — must stay silent.
        let src = "hom_test_cat = { my_decision = { cost = 999999999999 complete_effect = { add_political_power = 50 } } }";
        assert_eq!(h5011(&run(src)), 0);
    }

    #[test]
    fn test_non_numeric_cost_ignored() {
        let src = "focus = { id = TST_a cost = { foo = bar } }";
        assert_eq!(h5011(&run(src)), 0);
    }
}
