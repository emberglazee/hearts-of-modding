use crate::parser::ast;
use crate::rules::visitor::AstVisitor;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::ScopeStack;
use std::collections::HashSet;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates strategic region definitions (`map/strategicregions/*.txt`).
///
/// Tier 0 currently covers one cross-file check: a province claimed by two
/// region files (HOM2006). Per-file `naval_terrain` checks already exist
/// elsewhere (HOM5004); region coverage gaps (a province in NO region) have
/// no per-file trigger and stay out of scope until workspace-level
/// diagnostics exist.
///
/// URI-gated like `AiAreaRule`: outside `map/strategicregions/` the visitor
/// is inert.
pub(crate) struct StrategicRegionRule;

impl ValidationRule for StrategicRegionRule {
    fn check_block(
        &self,
        _entries: &[ast::Entry],
        _ctx: &ValidationContext,
        _diags: &mut Vec<Diagnostic>,
    ) {
    }
}

/// Collects the current file's region id + member provinces during the walk;
/// `after_walk` reports members that sibling region files also claim.
struct RegionMapVisitor {
    is_region_file: bool,
    in_region: u32,
    saw_region: bool,
    self_id: Option<u32>,
    members: Vec<(u32, ast::Range)>,
}

impl RegionMapVisitor {
    fn new(uri: &str) -> Self {
        let is_region_file =
            uri.contains("/map/strategicregions/") || uri.contains("\\map\\strategicregions\\");
        Self {
            is_region_file,
            in_region: 0,
            saw_region: false,
            self_id: None,
            members: Vec::new(),
        }
    }
}

impl AstVisitor for RegionMapVisitor {
    fn enter_assignment(
        &mut self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        _diags: &mut Vec<Diagnostic>,
    ) {
        if !self.is_region_file {
            return;
        }
        let key = ass.key_text(ctx.source);
        let is_block = matches!(
            &ass.value.value,
            ast::Value::Block(_) | ast::Value::TaggedBlock(_, _, _)
        );

        if key.eq_ignore_ascii_case("strategic_region") && is_block {
            self.in_region += 1;
            self.saw_region = true;
            return;
        }
        if self.in_region == 0 {
            return;
        }
        if key.eq_ignore_ascii_case("id") && self.self_id.is_none() {
            if let ast::Value::Number(n) = &ass.value.value
                && *n >= 0.0
            {
                self.self_id = Some(*n as u32);
            }
            return;
        }
        if key.eq_ignore_ascii_case("provinces") && is_block {
            let entries = match &ass.value.value {
                ast::Value::Block(entries) => entries,
                ast::Value::TaggedBlock(_, entries, _) => entries,
                _ => return,
            };
            for entry in entries {
                if let ast::Entry::Value(val) = entry {
                    let id = match &val.value {
                        ast::Value::Number(n) if *n >= 0.0 => Some(*n as u32),
                        ast::Value::String(s) => s.resolve(ctx.source).parse::<u32>().ok(),
                        _ => None,
                    };
                    if let Some(id) = id {
                        self.members.push((id, val.range.clone()));
                    }
                }
            }
        }
    }

    fn exit_assignment(
        &mut self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        _diags: &mut Vec<Diagnostic>,
    ) {
        if !self.is_region_file {
            return;
        }
        if ass
            .key_text(ctx.source)
            .eq_ignore_ascii_case("strategic_region")
            && matches!(
                &ass.value.value,
                ast::Value::Block(_) | ast::Value::TaggedBlock(_, _, _)
            )
        {
            self.in_region = self.in_region.saturating_sub(1);
        }
    }

    fn after_walk(&mut self, ctx: &ValidationContext, diags: &mut Vec<Diagnostic>) {
        if !self.is_region_file || !self.saw_region {
            return;
        }
        // Without a parseable id the check is skipped rather than run
        // against the file's own stale index entry.
        let Some(self_id) = self.self_id else {
            return;
        };
        let mut seen: HashSet<u32> = HashSet::new();
        for (prov, range) in &self.members {
            if !seen.insert(*prov) {
                continue;
            }
            let mut other: Option<u32> = None;
            for entry in ctx.strategic_regions.iter() {
                let id = *entry.key();
                if id != self_id && entry.value().provinces.contains(prov) {
                    other = Some(id);
                    break;
                }
            }
            if let Some(other_id) = other {
                diags.push(Diagnostic {
                    range: ctx.range(range),
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: format!("Province {} is also in strategic region {}", prov, other_id),
                    code: Some(NumberOrString::String(
                        crate::validation::advanced_validation::PROVINCE_IN_TWO_STRATEGIC_REGIONS
                            .to_string(),
                    )),
                    source: Some("Hearts of Modding".to_string()),
                    ..Default::default()
                });
            }
        }
    }
}

impl StrategicRegionRule {
    pub(crate) fn visitor(uri: &str) -> Box<dyn AstVisitor> {
        Box::new(RegionMapVisitor::new(uri))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scope::scope::Scope;
    use tower_lsp_server::ls_types::NumberOrString;

    const REGION_URI: &str = "/mod/map/strategicregions/01_test.txt";

    fn region_diags(ctx: &crate::test_support::TestCtx, input: &str, uri: &str) -> Vec<Diagnostic> {
        ctx.walk(
            input,
            uri,
            Scope::Global,
            vec![],
            vec![StrategicRegionRule::visitor(uri)],
        )
    }

    fn has_code(diags: &[Diagnostic], code: &str) -> bool {
        diags.iter().any(|d| {
            d.code == Some(NumberOrString::String(code.to_string()))
                && d.severity == Some(DiagnosticSeverity::WARNING)
        })
    }

    #[test]
    fn test_province_in_two_regions() {
        let ctx = crate::test_support::TestCtx::new().with_file(
            "/mod/map/strategicregions/02_other.txt",
            "strategic_region = { id = 2 provinces = { 7 } }",
        );
        let diags = region_diags(
            &ctx,
            "strategic_region = { id = 1 provinces = { 7 8 } }",
            REGION_URI,
        );
        let hits: Vec<_> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM2006".to_string())))
            .collect();
        assert_eq!(hits.len(), 1, "only shared prov 7: {:?}", diags);
        assert!(hits[0].message.contains('2'));
    }

    #[test]
    fn test_disjoint_and_self_claims_clean() {
        let ctx = crate::test_support::TestCtx::new().with_file(
            "/mod/map/strategicregions/02_other.txt",
            "strategic_region = { id = 2 provinces = { 7 } }",
        );
        let diags = region_diags(
            &ctx,
            "strategic_region = { id = 3 provinces = { 8 } }",
            "/mod/map/strategicregions/03_test.txt",
        );
        assert!(!has_code(&diags, "HOM2006"), "disjoint: {:?}", diags);
        let diags = region_diags(
            &ctx,
            "strategic_region = { id = 2 provinces = { 7 } }",
            "/mod/map/strategicregions/02_other.txt",
        );
        assert!(!has_code(&diags, "HOM2006"), "self excluded: {:?}", diags);
    }

    #[test]
    fn test_visitor_inert_outside_region_files() {
        let ctx = crate::test_support::TestCtx::new().with_file(
            "/mod/map/strategicregions/02_other.txt",
            "strategic_region = { id = 2 provinces = { 7 } }",
        );
        let diags = region_diags(
            &ctx,
            "strategic_region = { id = 1 provinces = { 7 } }",
            "/mod/history/states/1-Test.txt",
        );
        assert!(diags.is_empty(), "gated off: {:?}", diags);
    }
}
