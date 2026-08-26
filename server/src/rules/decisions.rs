use crate::data::interner::InternedStr;
use crate::data::layered_value::LayeredValue;
use crate::parser::ast;
use crate::rules::visitor::AstVisitor;
use crate::rules::{ValidationContext, ValidationRule};
#[cfg(test)]
use crate::scanner::decision_scanner::Decision;
use crate::scope::scope::ScopeStack;
#[cfg(test)]
use crate::utils::lsp_convert::RangeMapper;
use dashmap::DashMap;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Query scanner data to collect all known decision category names.
/// Only categories declared in `categories/*.txt` files count as "known".
fn known_categories(cats: &DashMap<InternedStr, LayeredValue<()>>) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for entry in cats.iter() {
        result.push(entry.key().to_string());
    }
    result
}

/// Check whether a file path is inside `common/decisions/` (but NOT
/// `common/decisions/categories/`).
fn is_decisions_file(uri: &str) -> bool {
    let lower = uri.to_ascii_lowercase();
    lower.contains("/common/decisions/") && !lower.contains("/common/decisions/categories/")
}

/// Keys that are only valid in category blocks, not in individual decisions.
const CATEGORY_ONLY_KEYS: [&str; 5] = [
    "picture",
    "visible_when_empty",
    "on_map_area",
    "scripted_gui",
    "day_of_week",
];

/// Block keys that are legitimate *category fields* (not decisions) when they
/// appear directly under a category. Vanilla decision categories use these as
/// condition containers (e.g. `allowed = { tag = { ... } }`), so they must not
/// be interpreted as a nested decision (which would falsely trigger the
/// end-of-decision HOM5008/5009 checks).
const CATEGORY_FIELD_BLOCKS: [&str; 2] = ["allowed", "visible"];

/// # HOM5006 — Undeclared decision category
/// # HOM5007 — Category-only key used inside a decision (game logs "Unexpected token" ERROR)
/// # HOM5008 — Decision missing complete_effect
/// # HOM5009 — Decision has both cost and custom_cost_trigger
pub(crate) struct DecisionsRule;

fn is_non_decision_block(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    lower == "country_event"
        || lower == "state_event"
        || lower == "news_event"
        || lower == "unit_leader_event"
        || lower == "operative_leader_event"
        || lower == "focus"
        || lower == "idea"
}

impl ValidationRule for DecisionsRule {
    fn check_block(
        &self,
        entries: &[ast::Entry],
        ctx: &ValidationContext,
        diags: &mut Vec<Diagnostic>,
    ) {
        if !is_decisions_file(ctx.uri) {
            return;
        }
        let cats = known_categories(ctx.decision_categories);
        for entry in entries {
            let ast::Entry::Assignment(ass) = entry else {
                continue;
            };
            let ast::Value::Block(inner) = &ass.value.value else {
                continue;
            };
            let category_key = ass.key_text(ctx.source);
            if is_non_decision_block(category_key) {
                continue;
            }
            let has_decision_children = inner.iter().any(|inner_entry| {
                matches!(inner_entry,
                    ast::Entry::Assignment(inner_ass) if matches!(&inner_ass.value.value, ast::Value::Block(_))
                )
            });
            if has_decision_children && !cats.contains(&category_key.to_string()) {
                diags.push(Diagnostic {
                    range: ctx.range(&ass.key_range),
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!(
                        "Decision category '{}' is not declared anywhere. \
                         Decisions under this category will not appear in-game.",
                        category_key
                    ),
                    code: Some(NumberOrString::String(
                        crate::validation::advanced_validation::UNDECLARED_DECISION_CATEGORY
                            .to_string(),
                    )),
                    source: Some("Hearts of Modding".to_string()),
                    ..Default::default()
                });
            }
        }
    }
}

impl DecisionsRule {
    pub(crate) fn visitor() -> Box<dyn AstVisitor> {
        Box::new(DecisionsVisitor::new())
    }
}

#[derive(Clone, Copy, PartialEq)]
enum BlockScopeKind {
    Root,
    Category,
    Decision,
    SubBlock,
}

/// One scope entry on the visitor's stack. `decision_key` is the name of the
/// nearest enclosing *decision* block (propagated down into sub-blocks), or
/// `None` when the scope is a category or the file root.
///
/// Using an explicit stack (push on enter, symmetric pop on exit) fixes the
/// previous single-`level` state machine, which corrupted the current depth
/// whenever a decision contained nested sub-blocks — e.g. `cancel_trigger -> { NOT = { } }`
/// or `timeout_effect -> { HBC = { ... } }`. After such a decision, the stale
/// `level`/`decision_key` wrongly re-scoped the *next* sibling category as a
/// decision, firing HOM5007 on its category-level keys.
struct BlockScope {
    kind: BlockScopeKind,
    decision_key: Option<String>,
}

struct DecisionsVisitor {
    scopes: Vec<BlockScope>,
    has_complete_effect: bool,
    /// Numeric value of `cost = N` (None when absent or non-numeric).
    cost_value: Option<f64>,
    has_custom_cost: bool,
    has_timeout_effect: bool,
    has_remove_effect: bool,
    has_modifier: bool,
}

impl DecisionsVisitor {
    fn new() -> Self {
        Self {
            scopes: vec![BlockScope {
                kind: BlockScopeKind::Root,
                decision_key: None,
            }],
            has_complete_effect: false,
            cost_value: None,
            has_custom_cost: false,
            has_timeout_effect: false,
            has_remove_effect: false,
            has_modifier: false,
        }
    }

    fn top(&self) -> &BlockScope {
        self.scopes
            .last()
            .expect("decision scope stack never empty")
    }

    /// True when the current key sits *directly* inside a decision block — the
    /// only position where a category-only key has no effect (HOM5007).
    fn directly_in_decision(&self) -> bool {
        self.top().kind == BlockScopeKind::Decision
    }

    /// True when the current key is inside a decision block at any depth.
    fn inside_decision(&self) -> bool {
        self.scopes.iter().any(|s| s.decision_key.is_some())
    }

    fn reset_decision_flags(&mut self) {
        self.has_complete_effect = false;
        self.cost_value = None;
        self.has_custom_cost = false;
        self.has_timeout_effect = false;
        self.has_remove_effect = false;
        self.has_modifier = false;
    }
}

impl AstVisitor for DecisionsVisitor {
    fn enter_assignment(
        &mut self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        diags: &mut Vec<Diagnostic>,
    ) {
        if !is_decisions_file(ctx.uri) {
            return;
        }

        let key = ass.key_text(ctx.source);

        // HOM5007: a category-only key (picture, visible_when_empty, ...) has no
        // effect ONLY as a direct child of a decision block. It is valid inside a
        // category, and harmless deeper inside sub-blocks (e.g. `picture` inside
        // `create_country_leader = { ... }`).
        if self.directly_in_decision() && CATEGORY_ONLY_KEYS.contains(&key) {
            diags.push(Diagnostic {
                range: ctx.range(&ass.key_range),
                severity: Some(DiagnosticSeverity::ERROR),
                message: format!(
                    "'{}' is a category-only key and has no effect inside a decision block.",
                    key
                ),
                code: Some(NumberOrString::String(
                    crate::validation::advanced_validation::CATEGORY_KEY_IN_DECISION.to_string(),
                )),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        }

        // Accumulate decision flags at any depth within a decision block.
        if self.inside_decision() {
            match key {
                "complete_effect" => self.has_complete_effect = true,
                "cost" => {
                    // Record the numeric value so HOM5009 can distinguish a
                    // real (non-zero) cost from the canonical
                    // `cost = 0` + custom_cost display pattern (71 of 75
                    // vanilla dual-cost decisions use cost = 0).
                    if let ast::Value::Number(n) = &ass.value.value {
                        self.cost_value = Some(*n);
                    }
                }
                "custom_cost_trigger" => self.has_custom_cost = true,
                "timeout_effect" => self.has_timeout_effect = true,
                "remove_effect" => self.has_remove_effect = true,
                _ => {}
            }
        }
        // `modifier` / `targeted_modifier` are only a decision-level payload
        // when they sit directly under the decision (timer modifier applied for
        // the days_remove / days_mission_timeout duration). Nested inside an
        // effect block (`add_opinion_modifier = { modifier = { ... } }`) they
        // are effect parameters and must not count as a decision payload.
        if self.directly_in_decision() && (key == "modifier" || key == "targeted_modifier") {
            self.has_modifier = true;
        }

        // Empty-block / non-decision top-level types: don't open a scope.
        let ast::Value::Block(_) = &ass.value.value else {
            return;
        };
        if is_non_decision_block(key) {
            return;
        }

        let kind = match self.top().kind {
            BlockScopeKind::Root => BlockScopeKind::Category,
            // `allowed` and `visible` are legitimate category-field blocks (not
            // decisions) when they appear directly under a category — vanilla
            // decision categories use both heavily. Treat them as a container
            // (SubBlock) so they don't get flagged as "missing complete_effect".
            BlockScopeKind::Category if CATEGORY_FIELD_BLOCKS.contains(&key) => {
                BlockScopeKind::SubBlock
            }
            BlockScopeKind::Category => BlockScopeKind::Decision,
            BlockScopeKind::Decision | BlockScopeKind::SubBlock => BlockScopeKind::SubBlock,
        };
        let decision_key = match kind {
            BlockScopeKind::Decision => Some(key.to_string()),
            // Sub-blocks inherit the nearest enclosing decision's key, so the
            // "inside a decision" context propagates down through any nesting.
            // Category fields (allowed/visible) inherit `None`, keeping them
            // out of the decision path entirely.
            BlockScopeKind::SubBlock => self.top().decision_key.clone(),
            _ => None,
        };
        self.scopes.push(BlockScope { kind, decision_key });
    }

    fn exit_assignment(
        &mut self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        diags: &mut Vec<Diagnostic>,
    ) {
        if !is_decisions_file(ctx.uri) {
            return;
        }
        let key = ass.key_text(ctx.source);
        let ast::Value::Block(inner) = &ass.value.value else {
            return;
        };
        if is_non_decision_block(key) {
            return;
        }

        let Some(scope) = self.scopes.pop() else {
            return;
        };

        // End-of-decision checks fire when leaving the decision scope itself.
        if scope.kind == BlockScopeKind::Decision && scope.decision_key.as_deref() == Some(key) {
            // HOM5008: Missing complete_effect.
            //
            // `complete_effect` is only required for a decision whose *entire*
            // payload is the click reward. Several other payload shapes ship in
            // vanilla WITHOUT complete_effect:
            //   * `remove_effect` — fires on take/removal. Vanilla ships these
            //     standalone (`PHI_create_faction` = cost + remove_effect, no
            //     days_remove), and combined with `days_remove` timers
            //     (AFG_nationalize_oil, ...).
            //   * Missions — `timeout_effect` fires when the mission timer
            //     expires (`available = { hidden_trigger = { always = no } }`
            //     makes them unclickable). Vanilla: 280 missions (CHL nacista
            //     chain, ARG coup, ...).
            //   * `modifier` / `targeted_modifier` — applied for the
            //     `days_remove` duration (ENG_the_mosley_plan, ...).
            // Flagging any of these is a false positive — the message claims
            // "does nothing when selected", but the decision does its work via
            // remove/timeout/modifier. Only a decision with NONE of the four
            // effect-bearing blocks genuinely does nothing when selected.
            let is_real_decision = inner.iter().any(|e| {
                matches!(e, ast::Entry::Assignment(a) if !CATEGORY_ONLY_KEYS.contains(&a.key_text(ctx.source)))
            });
            let has_payload = self.has_complete_effect
                || self.has_remove_effect
                || self.has_timeout_effect
                || self.has_modifier;
            if is_real_decision && !has_payload {
                diags.push(Diagnostic {
                    range: ctx.range(&ass.key_range),
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: format!(
                        "Decision '{}' has no effect of any kind (complete_effect / remove_effect / timeout_effect / modifier) — it does nothing when selected.",
                        key
                    ),
                    code: Some(NumberOrString::String(
                        crate::validation::advanced_validation::DECISION_MISSING_COMPLETE_EFFECT
                            .to_string(),
                    )),
                    source: Some("Hearts of Modding".to_string()),
                    ..Default::default()
                });
            }
            // HOM5009: A real cost AND custom_cost_trigger.
            //
            // Empirically (scanned all 4125 vanilla decisions) `cost` and
            // `custom_cost_trigger` are NOT mutually exclusive — 75 vanilla
            // decisions use both, and 71 of those use `cost = 0` (a custom cost
            // is a display-only mechanism; the real resource is deducted inside
            // complete_effect). Pairing them is only worth flagging when there
            // is an actual non-zero `cost` value — the rare (4/75) genuinely
            // ambiguous case.
            let real_cost = self.cost_value.is_some_and(|v| v > 0.0);
            if real_cost && self.has_custom_cost {
                diags.push(Diagnostic {
                    range: ctx.range(&ass.key_range),
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: format!(
                        "Decision '{}' has a non-zero 'cost' and 'custom_cost_trigger'. \
                         A custom cost is display-only — nothing is deducted — so 'cost' may \
                         be redundant or misleading; deduct the real resource inside complete_effect.",
                        key
                    ),
                    code: Some(NumberOrString::String(
                        crate::validation::advanced_validation::DECISION_DUAL_COST.to_string(),
                    )),
                    source: Some("Hearts of Modding".to_string()),
                    ..Default::default()
                });
            }
            self.reset_decision_flags();
        }
    }
}

// ---------------------------------------------------------------------------
// SECTION - Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::interner::InternedStr;
    use crate::data::layered_value::LayeredValue;
    use crate::data::scanner_data::ScannerData;
    use crate::parser::parser;
    use crate::rules::visitor::walk_script;
    use crate::scope::scope::Scope;

    fn dummy_range() -> ast::Range {
        ast::Range {
            start_line: 0,
            start_col: 0,
            end_line: 0,
            end_col: 0,
        }
    }

    fn check_block_diags(source: &str, uri: &str, scanner_data: &ScannerData) -> Vec<Diagnostic> {
        let (script, _) = parser::parse_script(source);
        let rule = DecisionsRule;
        let test_ctx = crate::test_support::TestCtx::wrap_ref(scanner_data);
        let range_mapper = RangeMapper::new(&script.source);
        let ctx = test_ctx.build_context(uri, &script.source, &range_mapper);
        let mut diags = Vec::new();
        rule.check_block(&script.entries, &ctx, &mut diags);
        diags
    }

    fn visitor_diags(source: &str, uri: &str, scanner_data: &ScannerData) -> Vec<Diagnostic> {
        let (script, _) = parser::parse_script(source);
        let test_ctx = crate::test_support::TestCtx::wrap_ref(scanner_data);
        let range_mapper = RangeMapper::new(&script.source);
        let ctx = test_ctx.build_context(uri, &script.source, &range_mapper);
        let mut diags = Vec::new();
        let mut visitors: Vec<Box<dyn AstVisitor>> = vec![DecisionsRule::visitor()];
        let rules: Vec<Box<dyn ValidationRule>> = vec![];
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

    // ── HOM5006 ──
    #[test]
    fn test_declared_category_no_diag() {
        let data = ScannerData::new();
        let key: InternedStr = InternedStr::from("test_hom_decision");
        data.decisions.insert(
            key,
            LayeredValue::new(Decision {
                key: "my_decision".to_string(),
                category: "hom_test_valid_category".to_string(),
                path: InternedStr::from("/common/decisions/categories/test.txt"),
                range: dummy_range(),
            }),
        );
        data.decision_categories.insert(
            InternedStr::from("hom_test_valid_category"),
            LayeredValue::new(()),
        );
        let source = r#"hom_test_valid_category = { my_decision = { icon = generic_research complete_effect = { add_political_power = 50 } } }"#;
        let diags = check_block_diags(source, "/common/decisions/test.txt", &data);
        assert!(diags.is_empty(), "Expected no HOM5006, got: {:?}", diags);
    }

    #[test]
    fn test_inline_category_also_unddeclared() {
        let data = ScannerData::new();
        // Category is only known via inline decisions file, not categories/ dir
        let key: InternedStr = InternedStr::from("test_hom_decision");
        data.decisions.insert(
            key,
            LayeredValue::new(Decision {
                key: "my_decision".to_string(),
                category: "hom_test_inline_cat".to_string(),
                path: InternedStr::from("/common/decisions/test.txt"),
                range: dummy_range(),
            }),
        );
        let source = r#"hom_test_inline_cat = { my_decision = { icon = generic_research complete_effect = { add_political_power = 50 } } }"#;
        let diags = check_block_diags(source, "/common/decisions/test.txt", &data);
        assert_eq!(diags.len(), 1, "Inline category should be HOM5006 too");
    }

    #[test]
    fn test_undeclared_category_diag() {
        let data = ScannerData::new();
        let source = r#"hom_undefined_cat = { orphan_decision = { icon = generic_research complete_effect = { add_political_power = 50 } } }"#;
        let diags = check_block_diags(source, "/common/decisions/test.txt", &data);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("hom_undefined_cat"));
        assert_eq!(diags[0].severity, Some(DiagnosticSeverity::ERROR));
    }

    #[test]
    fn test_no_diag_outside_decisions_dir() {
        let data = ScannerData::new();
        let source = r#"hom_undefined_cat = { orphan_decision = { icon = generic_research complete_effect = { add_political_power = 50 } } }"#;
        let diags = check_block_diags(source, "/events/test.txt", &data);
        assert!(diags.is_empty());
    }

    // ── HOM5007 ──
    #[test]
    fn test_category_key_in_decision() {
        let data = ScannerData::new();
        let key: InternedStr = InternedStr::from("test_hom_decision");
        data.decisions.insert(
            key,
            LayeredValue::new(Decision {
                key: "my_decision".to_string(),
                category: "hom_test_cat".to_string(),
                path: InternedStr::from("test.txt"),
                range: dummy_range(),
            }),
        );
        let source = r#"hom_test_cat = { misplaced = { picture = GFX_decision_cat_picture visible_when_empty = yes scripted_gui = my_gui icon = generic_research complete_effect = { add_political_power = 50 } } }"#;
        let diags = visitor_diags(source, "/common/decisions/test.txt", &data);
        let cat_key_diags: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM5007".to_string())))
            .collect();
        assert_eq!(cat_key_diags.len(), 3, "Expected 3 HOM5007");
    }

    #[test]
    fn test_category_key_inside_category_not_flagged() {
        let data = ScannerData::new();
        let key: InternedStr = InternedStr::from("test_hom_decision");
        data.decisions.insert(
            key,
            LayeredValue::new(Decision {
                key: "my_decision".to_string(),
                category: "hom_test_cat".to_string(),
                path: InternedStr::from("test.txt"),
                range: dummy_range(),
            }),
        );
        let source = r#"hom_test_cat = { picture = GFX_some_pic visible_when_empty = yes }"#;
        let diags = visitor_diags(source, "/common/decisions/test.txt", &data);
        let cat_key_diags: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM5007".to_string())))
            .collect();
        assert!(cat_key_diags.is_empty());
    }

    #[test]
    /// Reproduces the user report: a top-level category whose children are BOTH
    /// category-field blocks (allowed/visible) AND a real decision, with a
    /// `visible_when_empty` on the category in between. That category-level
    /// `visible_when_empty` must NOT be flagged HOM5007.
    fn test_category_visible_when_empty_with_real_decision_children() {
        let data = ScannerData::new();
        let source = r#"hom_expand_cat = {
            allowed = { original_tag = SPE }
            visible = { has_completed_focus = SPE_expand_the_spp }
            visible_when_empty = yes
            SPE_invite_country_to_spp = {
                icon = GFX_decision_icon_default
                complete_effect = { add_political_power = 50 }
            }
        }"#;
        let diags = visitor_diags(source, "/common/decisions/test.txt", &data);
        let h7: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM5007".to_string())))
            .collect();
        assert!(
            h7.is_empty(),
            "category-level visible_when_empty wrongly flagged as HOM5007: {h7:?}"
        );
    }

    #[test]
    /// picture inside a sub-block (create_country_leader etc.) should NOT fire HOM5007.
    fn test_picture_inside_sub_block_not_flagged() {
        let data = ScannerData::new();
        let key: InternedStr = InternedStr::from("test_hom_decision");
        data.decisions.insert(
            key,
            LayeredValue::new(Decision {
                key: "my_decision".to_string(),
                category: "hom_test_cat".to_string(),
                path: InternedStr::from("test.txt"),
                range: dummy_range(),
            }),
        );
        let source = r#"hom_test_cat = { my_decision = { icon = generic_research cost = 200 complete_effect = { every_country = { limit = { tag = prev } create_country_leader = { name = "Test" picture = "gfx/leaders/test.dds" ideology = neutrality } } } } }"#;
        let diags = visitor_diags(source, "/common/decisions/test.txt", &data);
        let cat_key_diags: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM5007".to_string())))
            .collect();
        assert!(
            cat_key_diags.is_empty(),
            "picture in sub-block should not be HOM5007: {:?}",
            cat_key_diags
        );
    }

    // ── HOM5008 ──
    #[test]
    fn test_decision_missing_complete_effect() {
        let data = ScannerData::new();
        let key: InternedStr = InternedStr::from("test_hom_decision");
        data.decisions.insert(
            key,
            LayeredValue::new(Decision {
                key: "no_effect".to_string(),
                category: "hom_test_cat".to_string(),
                path: InternedStr::from("test.txt"),
                range: dummy_range(),
            }),
        );
        let source = r#"hom_test_cat = { no_effect = { icon = generic_research allowed = { always = yes } visible = { always = yes } } }"#;
        let diags = visitor_diags(source, "/common/decisions/test.txt", &data);
        let missing_diags: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM5008".to_string())))
            .collect();
        assert_eq!(missing_diags.len(), 1, "Expected HOM5008");
    }

    #[test]
    fn test_decision_with_complete_effect_no_diag() {
        let data = ScannerData::new();
        let key: InternedStr = InternedStr::from("test_hom_decision");
        data.decisions.insert(
            key,
            LayeredValue::new(Decision {
                key: "has_effect".to_string(),
                category: "hom_test_cat".to_string(),
                path: InternedStr::from("test.txt"),
                range: dummy_range(),
            }),
        );
        let source = r#"hom_test_cat = { has_effect = { icon = generic_research complete_effect = { add_political_power = 50 } } }"#;
        let diags = visitor_diags(source, "/common/decisions/test.txt", &data);
        let missing_diags: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM5008".to_string())))
            .collect();
        assert!(missing_diags.is_empty());
    }

    /// Regression test: a mission (days_mission_timeout + timeout_effect, no
    /// complete_effect) is legitimate — its payload runs on timer expiry. This
    /// matches the vanilla pattern (`CHL_nacistas_gathering_support_mission`,
    /// `ARG_military_coup_attempt`, etc., 280 shipped missions). HOM5008 must
    /// NOT fire for it.
    #[test]
    fn test_mission_with_timeout_effect_no_missing_complete_diag() {
        let data = ScannerData::new();
        let key: InternedStr = InternedStr::from("test_hom_decision");
        data.decisions.insert(
            key,
            LayeredValue::new(Decision {
                key: "gold_monthly_tick".to_string(),
                category: "hom_test_cat".to_string(),
                path: InternedStr::from("test.txt"),
                range: dummy_range(),
            }),
        );
        let source = r#"hom_test_cat = { gold_monthly_tick = { allowed = { always = yes } activation = { always = yes } available = { hidden_trigger = { always = no } } fire_only_once = yes days_mission_timeout = 30 is_good = yes timeout_effect = { hidden_effect = { activate_mission = gold_monthly_tick } } } }"#;
        let diags = visitor_diags(source, "/common/decisions/test.txt", &data);
        let missing_diags: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM5008".to_string())))
            .collect();
        assert!(
            missing_diags.is_empty(),
            "Expected no HOM5008 for a mission with timeout_effect, got: {:?}",
            missing_diags
        );
    }

    /// A decision with a timeout but NO timeout_effect and NO complete_effect
    /// genuinely does nothing on selection — HOM5008 should still fire.
    #[test]
    fn test_decision_neither_complete_nor_timeout_effect_still_flagged() {
        let data = ScannerData::new();
        let key: InternedStr = InternedStr::from("test_hom_decision");
        data.decisions.insert(
            key,
            LayeredValue::new(Decision {
                key: "does_nothing".to_string(),
                category: "hom_test_cat".to_string(),
                path: InternedStr::from("test.txt"),
                range: dummy_range(),
            }),
        );
        let source = r#"hom_test_cat = { does_nothing = { icon = generic_research so_trigger = { always = yes } } }"#;
        let diags = visitor_diags(source, "/common/decisions/test.txt", &data);
        let missing_diags: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM5008".to_string())))
            .collect();
        assert_eq!(
            missing_diags.len(),
            1,
            "Expected HOM5008 for a decision with no effect of any kind"
        );
    }

    /// Regression test: a timed decision (days_remove + remove_effect, no
    /// complete_effect) is legitimate — vanilla AFG_nationalize_oil pattern.
    /// Its payload runs when the timer ends. HOM5008 must NOT fire.
    #[test]
    fn test_timed_decision_with_remove_effect_no_missing_complete_diag() {
        let data = ScannerData::new();
        let key: InternedStr = InternedStr::from("test_hom_decision");
        data.decisions.insert(
            key,
            LayeredValue::new(Decision {
                key: "nationalize_oil".to_string(),
                category: "hom_test_cat".to_string(),
                path: InternedStr::from("test.txt"),
                range: dummy_range(),
            }),
        );
        let source = r#"hom_test_cat = { nationalize_oil = { icon = generic_oil available = { has_completed_focus = AFG_75_year_oil_concessions } days_remove = 14 modifier = { political_power_factor = -0.1 } remove_effect = { country_event = { id = AFG_industrial_events.13 } } } }"#;
        let diags = visitor_diags(source, "/common/decisions/test.txt", &data);
        let missing_diags: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM5008".to_string())))
            .collect();
        assert!(
            missing_diags.is_empty(),
            "Expected no HOM5008 for a timed decision with remove_effect, got: {:?}",
            missing_diags
        );
    }

    /// Regression test (hotfix): `remove_effect` ALONE (no days_remove, no
    /// complete_effect) is a legitimate payload — it fires on take/removal.
    /// Vanilla `PHI_create_faction` (cost + remove_effect only). The user's
    /// `FKE_supply_warning` / `FKE_reform_eastern_kingdom_decision` are this
    /// shape. HOM5008 must NOT fire.
    #[test]
    fn test_standalone_remove_effect_no_missing_complete_diag() {
        let data = ScannerData::new();
        let key: InternedStr = InternedStr::from("test_hom_decision");
        data.decisions.insert(
            key,
            LayeredValue::new(Decision {
                key: "supply_warning".to_string(),
                category: "hom_test_cat".to_string(),
                path: InternedStr::from("test.txt"),
                range: dummy_range(),
            }),
        );
        let source = r#"hom_test_cat = { supply_warning = { icon = GFX_decision_generic_scorched_earth fire_only_once = yes available = { controls_state = 422 } remove_effect = { add_war_support = 0.05 } ai_will_do = { base = 10 } } }"#;
        let diags = visitor_diags(source, "/common/decisions/test.txt", &data);
        let missing_diags: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM5008".to_string())))
            .collect();
        assert!(
            missing_diags.is_empty(),
            "Expected no HOM5008 for a decision with standalone remove_effect, got: {:?}",
            missing_diags
        );
    }

    /// Regression test: a modifier-only timer decision (days_remove + modifier,
    /// no complete_effect and no remove_effect) is legitimate — vanilla
    /// ENG_the_mosley_plan pattern (38 such decisions). The timer grants the
    /// modifier for days_remove days. HOM5008 must NOT fire.
    #[test]
    fn test_timed_decision_with_modifier_only_no_missing_complete_diag() {
        let data = ScannerData::new();
        let key: InternedStr = InternedStr::from("test_hom_decision");
        data.decisions.insert(
            key,
            LayeredValue::new(Decision {
                key: "mosley_plan".to_string(),
                category: "hom_test_cat".to_string(),
                path: InternedStr::from("test.txt"),
                range: dummy_range(),
            }),
        );
        let source = r#"hom_test_cat = { mosley_plan = { available = { always = yes } days_remove = 365 modifier = { political_power_factor = -0.05 } } }"#;
        let diags = visitor_diags(source, "/common/decisions/test.txt", &data);
        let missing_diags: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM5008".to_string())))
            .collect();
        assert!(
            missing_diags.is_empty(),
            "Expected no HOM5008 for a modifier-only timer decision, got: {:?}",
            missing_diags
        );
    }

    /// A nested `modifier` inside a sub-block (e.g. ai_will_do weight modifier)
    /// must NOT count as a decision payload: this decision has no complete_effect,
    /// no timer, no remove — it still gets HOM5008 even though `modifier` appears.
    #[test]
    fn test_nested_modifier_in_subblock_not_counted_as_payload() {
        let data = ScannerData::new();
        let key: InternedStr = InternedStr::from("test_hom_decision");
        data.decisions.insert(
            key,
            LayeredValue::new(Decision {
                key: "fake_payload".to_string(),
                category: "hom_test_cat".to_string(),
                path: InternedStr::from("test.txt"),
                range: dummy_range(),
            }),
        );
        let source = r#"hom_test_cat = { fake_payload = { icon = generic_research visible = { always = yes } ai_will_do = { base = 1 modifier = { factor = 0 } } } }"#;
        let diags = visitor_diags(source, "/common/decisions/test.txt", &data);
        let missing_diags: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM5008".to_string())))
            .collect();
        assert_eq!(
            missing_diags.len(),
            1,
            "Expected HOM5008 for a decision with no payload"
        );
    }

    #[test]
    fn test_empty_category_stub_no_effect_diag() {
        let data = ScannerData::new();
        let key: InternedStr = InternedStr::from("test_hom_decision");
        data.decisions.insert(
            key,
            LayeredValue::new(Decision {
                key: "some_decision".to_string(),
                category: "hom_test_cat".to_string(),
                path: InternedStr::from("test.txt"),
                range: dummy_range(),
            }),
        );
        let source = r#"hom_test_cat = { icon = generic_research visible_when_empty = yes }"#;
        let diags = visitor_diags(source, "/common/decisions/test.txt", &data);
        let missing_diags: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM5008".to_string())))
            .collect();
        assert!(missing_diags.is_empty());
    }

    // ── HOM5009 ──
    #[test]
    fn test_dual_cost_diag() {
        let data = ScannerData::new();
        let key: InternedStr = InternedStr::from("test_hom_decision");
        data.decisions.insert(
            key,
            LayeredValue::new(Decision {
                key: "dual_cost".to_string(),
                category: "hom_test_cat".to_string(),
                path: InternedStr::from("test.txt"),
                range: dummy_range(),
            }),
        );
        let source = r#"hom_test_cat = { dual_cost = { icon = generic_research cost = 50 custom_cost_trigger = { has_command_power > 14 } custom_cost_text = decision_cost_CP_15 complete_effect = { add_political_power = 50 } } }"#;
        let diags = visitor_diags(source, "/common/decisions/test.txt", &data);
        let dual_diags: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM5009".to_string())))
            .collect();
        assert_eq!(dual_diags.len(), 1, "Expected HOM5009");
    }

    #[test]
    fn test_only_cost_no_diag() {
        let data = ScannerData::new();
        let key: InternedStr = InternedStr::from("test_hom_decision");
        data.decisions.insert(
            key,
            LayeredValue::new(Decision {
                key: "single_cost".to_string(),
                category: "hom_test_cat".to_string(),
                path: InternedStr::from("test.txt"),
                range: dummy_range(),
            }),
        );
        let source = r#"hom_test_cat = { single_cost = { icon = generic_research cost = 50 complete_effect = { add_political_power = 50 } } }"#;
        let diags = visitor_diags(source, "/common/decisions/test.txt", &data);
        let dual_diags: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM5009".to_string())))
            .collect();
        assert!(dual_diags.is_empty());
    }

    /// Regression test (2026-08): `cost = 0` + `custom_cost_trigger` is the
    /// canonical vanilla dual-cost pattern (71 of 75 such decisions use
    /// `cost = 0`; the custom cost is display-only). HOM5009 must NOT fire —
    /// it only fires on an actual non-zero cost AND custom_cost_trigger.
    #[test]
    fn test_zero_cost_with_custom_cost_trigger_no_diag() {
        let data = ScannerData::new();
        let key: InternedStr = InternedStr::from("test_hom_decision");
        data.decisions.insert(
            key,
            LayeredValue::new(Decision {
                key: "zero_cost_custom".to_string(),
                category: "hom_test_cat".to_string(),
                path: InternedStr::from("test.txt"),
                range: dummy_range(),
            }),
        );
        // Mirrors AUS_arm_border_guards_shape but with cost = 0 (canonical).
        let source = r#"hom_test_cat = { zero_cost_custom = { icon = generic_research cost = 0 custom_cost_trigger = { has_equipment = { infantry_equipment > 499 } } custom_cost_text = decision_cost_ie_500 complete_effect = { add_war_support = 0.05 } } }"#;
        let diags = visitor_diags(source, "/common/decisions/test.txt", &data);
        let dual_diags: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM5009".to_string())))
            .collect();
        assert!(
            dual_diags.is_empty(),
            "cost = 0 + custom_cost_trigger must not fire HOM5009, got: {:?}",
            dual_diags
        );
    }

    /// Regression test: decisions with sub-blocks (available, visible, ai_will_do,
    /// highlight_states) BEFORE complete_effect should not produce a false positive
    /// HOM5008. The flag-tracking guard must not rely on `level == BlockLevel::Decision`
    /// because the level flips to SubBlock after nested sub-block entry/exit.
    #[test]
    fn test_complete_effect_after_subblocks_no_false_positive() {
        let data = ScannerData::new();
        let key: InternedStr = InternedStr::from("test_hom_decision");
        data.decisions.insert(
            key,
            LayeredValue::new(Decision {
                key: "my_decision".to_string(),
                category: "hom_test_cat".to_string(),
                path: InternedStr::from("test.txt"),
                range: dummy_range(),
            }),
        );
        let source = r#"hom_test_cat = { my_decision = { icon = generic_research available = { always = yes } visible = { always = yes } ai_will_do = { base = 50 } cost = 10 highlight_states = { highlight_states_trigger = { state = 62 } } complete_effect = { add_political_power = 50 } } }"#;
        let diags = visitor_diags(source, "/common/decisions/test.txt", &data);
        let missing_diags: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM5008".to_string())))
            .collect();
        assert!(
            missing_diags.is_empty(),
            "Expected no HOM5008 for decision with complete_effect after sub-blocks, got: {:?}",
            missing_diags
        );
    }

    /// Regression test: decisions with sub-blocks before `cost` and `custom_cost_trigger`
    /// should still fire HOM5009 (dual cost). Same level flip-flop issue.
    #[test]
    fn test_dual_cost_after_subblocks_still_detected() {
        let data = ScannerData::new();
        let key: InternedStr = InternedStr::from("test_hom_decision");
        data.decisions.insert(
            key,
            LayeredValue::new(Decision {
                key: "dual_cost".to_string(),
                category: "hom_test_cat".to_string(),
                path: InternedStr::from("test.txt"),
                range: dummy_range(),
            }),
        );
        let source = r#"hom_test_cat = { dual_cost = { icon = generic_research available = { always = yes } cost = 50 visible = { always = yes } custom_cost_trigger = { has_command_power > 14 } complete_effect = { add_political_power = 50 } } }"#;
        let diags = visitor_diags(source, "/common/decisions/test.txt", &data);
        let dual_diags: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM5009".to_string())))
            .collect();
        assert_eq!(
            dual_diags.len(),
            1,
            "Expected HOM5009 for dual cost decision with sub-blocks before cost"
        );
    }

    /// Regression: a category-level `visible_when_empty` must NOT be flagged
    /// HOM5007 even when an *earlier sibling* decision contained deeply nested
    /// sub-blocks (e.g. `cancel_trigger -> NOT`, `timeout_effect -> TAG`). The
    /// old single-`level` scope state machine leaked that nesting into later
    /// sibling blocks and wrongly re-scoped this category as a decision.
    #[test]
    fn test_visible_when_empty_not_flagged_after_nested_subblocks_sibling() {
        let data = ScannerData::new();
        let source = r#"SPE_prior_category = {
            inv_decision = {
                cancel_trigger = { NOT = { country_exists = HBC } }
                timeout_effect = { HBC = { country_event = hbc.46 } }
            }
        }
        SPE_expand_spp_category = {
            visible_when_empty = yes
            SPE_invite_country_to_spp = {
                icon = GFX_decision_icon_default
                complete_effect = { add_political_power = 50 }
            }
        }"#;
        let diags = visitor_diags(source, "/common/decisions/test.txt", &data);
        let h7: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM5007".to_string())))
            .collect();
        assert!(
            h7.is_empty(),
            "category-level visible_when_empty wrongly flagged as HOM5007: {h7:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// !SECTION
// ---------------------------------------------------------------------------
