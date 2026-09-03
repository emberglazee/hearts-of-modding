use crate::data::interner::InternedStr;
use crate::data::layered_value::LayeredValue;
use crate::parser::ast;
use crate::rules::visitor::AstVisitor;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::ScopeStack;
use dashmap::DashMap;
use std::collections::HashSet;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates state definition files (`history/states/*.txt`):
///
/// - `state_category = X` → warns if X is not a known state category
/// - `resources = { X = N }` → warns if X is not a known resource type
/// - `buildings = { X = N }` → warns if X is not a known building type
///
/// Assumes definitions are scanned from `common/state_category/`,
/// `common/resources/`, and `common/buildings/`.
pub(crate) struct StateDefinitionRule;

impl ValidationRule for StateDefinitionRule {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        _pushed_scope: bool,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key_lower = ass.key_text(ctx.source).to_ascii_lowercase();

        // state_category = <value> — validate value is known
        if key_lower == "state_category" {
            if let Some(value_str) = extract_string_value(&ass.value, ctx.source) {
                if !ctx.state_categories.is_empty() && !ctx.state_categories.contains_key(value_str)
                {
                    let known = format_known_list(ctx.state_categories);
                    diags.push(Diagnostic {
                        range: ctx.range(&ass.value.range),
                        severity: Some(DiagnosticSeverity::WARNING),
                        message: format!(
                            "Unknown state category '{}'{}",
                            value_str,
                            if known.is_empty() {
                                String::new()
                            } else {
                                format!(". Known: {}", known)
                            },
                        ),
                        code: Some(NumberOrString::String(
                            crate::validation::advanced_validation::UNKNOWN_STATE_CATEGORY
                                .to_string(),
                        )),
                        source: Some("Hearts of Modding".to_string()),
                        ..Default::default()
                    });
                }
            }
            return;
        }

        // resources = { <resource> = <amount> } — validate resource names
        if key_lower == "resources" {
            if let ast::Value::Block(resource_entries) = &ass.value.value {
                validate_keys_in_dashmap(
                    resource_entries,
                    ctx.resources,
                    "resource",
                    "common/resources/*.txt",
                    crate::validation::advanced_validation::UNKNOWN_RESOURCE,
                    diags,
                    ctx.source,
                    ctx.range_mapper,
                );
            }
            return;
        }

        // buildings = { <building> = <level> } — validate building names
        // Numeric keys are province IDs for province-level building placements
        // (e.g., 2671 = { naval_base = 2 }), not building type names.
        // Recurse into province-level blocks to validate their building names too.
        if key_lower == "buildings" {
            if let ast::Value::Block(building_entries) = &ass.value.value {
                for entry in building_entries {
                    if let ast::Entry::Assignment(inner_ass) = entry {
                        let key = inner_ass.key_text(ctx.source);
                        if key.bytes().all(|b| b.is_ascii_digit()) {
                            // Province-level placement: 2671 = { naval_base = 2 }
                            if let ast::Value::Block(province_entries) = &inner_ass.value.value {
                                validate_keys_in_dashmap(
                                    province_entries,
                                    ctx.buildings,
                                    "building",
                                    "common/buildings/*.txt",
                                    crate::validation::advanced_validation::UNKNOWN_BUILDING,
                                    diags,
                                    ctx.source,
                                    ctx.range_mapper,
                                );
                            }
                        } else {
                            // State-level building: infrastructure = 2
                            if !ctx.buildings.is_empty() && !ctx.buildings.contains_key(key) {
                                diags.push(Diagnostic {
                                    range: ctx.range(&inner_ass.key_range),
                                    severity: Some(DiagnosticSeverity::WARNING),
                                    message: format!(
                                        "Unknown building '{}'. buildings are defined in common/buildings/*.txt",
                                        key,
                                    ),
                                    code: Some(NumberOrString::String(
                                        crate::validation::advanced_validation::UNKNOWN_BUILDING.to_string(),
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
    }
}

/// Extract a string value from a `NodeedValue`.
/// HOI4 identifiers are parsed as `Value::String` by the parser.
fn extract_string_value<'a>(val: &'a ast::NodeedValue, source: &'a str) -> Option<&'a str> {
    val.value.as_str(source)
}

/// Check that every assignment key in `entries` exists in the DashMap.
fn validate_keys_in_dashmap<T>(
    entries: &[ast::Entry],
    map: &DashMap<InternedStr, T>,
    entity_type: &str,
    source_hint: &str,
    error_code: &str,
    diags: &mut Vec<Diagnostic>,
    source: &str,
    range: &crate::utils::lsp_convert::RangeMapper,
) {
    if map.is_empty() {
        return;
    }
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            let name = ass.key_text(source);
            if !map.contains_key(name) {
                diags.push(Diagnostic {
                    range: range.range(&ass.key_range),
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: format!(
                        "Unknown {} '{}'. {}s are defined in {}",
                        entity_type, name, entity_type, source_hint,
                    ),
                    code: Some(NumberOrString::String(error_code.to_string())),
                    source: Some("Hearts of Modding".to_string()),
                    ..Default::default()
                });
            }
        }
    }
}

/// Build a comma-separated list of known state categories.
fn format_known_list(
    map: &DashMap<InternedStr, LayeredValue<crate::scanner::state_category_scanner::StateCategory>>,
) -> String {
    let mut names: Vec<String> = map.iter().map(|e| e.key().to_string()).collect();
    names.sort();
    names.join(", ")
}

/// Cross-file map integrity checks for `history/states/*.txt` (Tier 0).
///
/// Collects the state's own id, member provinces, victory-point provinces and
/// province-keyed building placements during the walk; `after_walk`
/// cross-references once everything is collected.
///
/// Two vanilla shapes drive the design (verified in `38-Sweden.txt`):
/// `victory_points` REPEATS as separate blocks, so VP ids ACCUMULATE (assign
/// would keep only the last block); province placements are numeric-keyed
/// blocks inside `history.buildings` (`11129 = { dam = 1 }`).
///
/// URI-gated like `AiAreaRule`: outside `history/states/` the visitor is inert.
struct StateMapVisitor {
    is_state_file: bool,
    in_state: u32,
    in_buildings: u32,
    saw_state: bool,
    self_id: Option<u32>,
    state_key_range: Option<ast::Range>,
    provinces_key_range: Option<ast::Range>,
    provinces: Vec<(u32, ast::Range)>,
    vp_firsts: Vec<(u32, ast::Range)>,
    placements: Vec<ProvincePlacement>,
}

/// A `1234 = { naval_base = 1 }` entry inside `history.buildings`.
struct ProvincePlacement {
    province: u32,
    building: String,
    province_range: ast::Range,
    building_range: ast::Range,
}

impl StateMapVisitor {
    fn new(uri: &str) -> Self {
        let is_state_file = uri.contains("/history/states/") || uri.contains("\\history\\states\\");
        Self {
            is_state_file,
            in_state: 0,
            in_buildings: 0,
            saw_state: false,
            self_id: None,
            state_key_range: None,
            provinces_key_range: None,
            provinces: Vec::new(),
            vp_firsts: Vec::new(),
            placements: Vec::new(),
        }
    }
}

/// Collect bare numeric ids (with ranges) from a `{ ... }` value block.
fn collect_ids_with_ranges(value: &ast::Value, source: &str, out: &mut Vec<(u32, ast::Range)>) {
    let entries = match value {
        ast::Value::Block(entries) => entries,
        ast::Value::TaggedBlock(_, entries, _) => entries,
        _ => return,
    };
    for entry in entries {
        if let ast::Entry::Value(val) = entry {
            let id = match &val.value {
                ast::Value::Number(n) if *n >= 0.0 => Some(*n as u32),
                ast::Value::String(s) => s.resolve(source).parse::<u32>().ok(),
                _ => None,
            };
            if let Some(id) = id {
                out.push((id, val.range.clone()));
            }
        }
    }
}

fn push_state_warning(
    ctx: &ValidationContext,
    range: &ast::Range,
    code: &str,
    message: String,
    diags: &mut Vec<Diagnostic>,
) {
    diags.push(Diagnostic {
        range: ctx.range(range),
        severity: Some(DiagnosticSeverity::WARNING),
        message,
        code: Some(NumberOrString::String(code.to_string())),
        source: Some("Hearts of Modding".to_string()),
        ..Default::default()
    });
}

impl AstVisitor for StateMapVisitor {
    fn enter_assignment(
        &mut self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        _diags: &mut Vec<Diagnostic>,
    ) {
        if !self.is_state_file {
            return;
        }
        let key = ass.key_text(ctx.source);
        let is_block = matches!(
            &ass.value.value,
            ast::Value::Block(_) | ast::Value::TaggedBlock(_, _, _)
        );

        if key.eq_ignore_ascii_case("state") && is_block {
            self.in_state += 1;
            self.saw_state = true;
            if self.state_key_range.is_none() {
                self.state_key_range = Some(ass.key_range.clone());
            }
            return;
        }
        if self.in_state == 0 {
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
            if self.provinces_key_range.is_none() {
                self.provinces_key_range = Some(ass.key_range.clone());
            }
            collect_ids_with_ranges(&ass.value.value, ctx.source, &mut self.provinces);
            return;
        }
        if key.eq_ignore_ascii_case("victory_points") && is_block {
            // Pairs are (province, value) — keep the first of each pair,
            // accumulated across repeated blocks.
            let mut values: Vec<(u32, ast::Range)> = Vec::new();
            collect_ids_with_ranges(&ass.value.value, ctx.source, &mut values);
            for i in (0..values.len()).step_by(2) {
                self.vp_firsts.push(values[i].clone());
            }
            return;
        }
        if key.eq_ignore_ascii_case("buildings") && is_block {
            self.in_buildings += 1;
            return;
        }
        if self.in_buildings > 0
            && !key.is_empty()
            && key.bytes().all(|b| b.is_ascii_digit())
            && let ast::Value::Block(placement_entries) = &ass.value.value
        {
            if let Ok(prov) = key.parse::<u32>() {
                for entry in placement_entries {
                    if let ast::Entry::Assignment(inner) = entry {
                        self.placements.push(ProvincePlacement {
                            province: prov,
                            building: inner.key_text(ctx.source).to_string(),
                            province_range: ass.key_range.clone(),
                            building_range: inner.key_range.clone(),
                        });
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
        if !self.is_state_file {
            return;
        }
        let key = ass.key_text(ctx.source);
        let is_block = matches!(
            &ass.value.value,
            ast::Value::Block(_) | ast::Value::TaggedBlock(_, _, _)
        );
        if key.eq_ignore_ascii_case("state") && is_block {
            self.in_state = self.in_state.saturating_sub(1);
        } else if key.eq_ignore_ascii_case("buildings") && is_block {
            self.in_buildings = self.in_buildings.saturating_sub(1);
        }
    }

    fn after_walk(&mut self, ctx: &ValidationContext, diags: &mut Vec<Diagnostic>) {
        if !self.is_state_file || !self.saw_state {
            return;
        }
        let use_provinces = !ctx.provinces.is_empty();

        // Empty state — usually a merge leftover.
        if self.provinces.is_empty() {
            if let Some(range) = self
                .provinces_key_range
                .as_ref()
                .or(self.state_key_range.as_ref())
            {
                push_state_warning(
                    ctx,
                    range,
                    crate::validation::advanced_validation::EMPTY_STATE,
                    "State has no provinces".to_string(),
                    diags,
                );
            }
        }

        let mut members: HashSet<u32> = HashSet::new();
        let mut seen_members: HashSet<u32> = HashSet::new();
        for (prov, range) in &self.provinces {
            // Diagnose each listed id once even if repeated.
            if !seen_members.insert(*prov) {
                continue;
            }
            members.insert(*prov);
            if use_provinces {
                match ctx.provinces.get(prov) {
                    None => push_state_warning(
                        ctx,
                        range,
                        crate::validation::advanced_validation::STATE_UNKNOWN_PROVINCE,
                        format!("Province {} is not in map/definition.csv", prov),
                        diags,
                    ),
                    Some(def) => {
                        if def.prov_type == "sea" {
                            push_state_warning(
                                ctx,
                                range,
                                crate::validation::advanced_validation::SEA_PROVINCE_IN_STATE,
                                format!(
                                    "Sea province {} listed in a state (lakes are legal, sea is not)",
                                    prov
                                ),
                                diags,
                            );
                        }
                    }
                }
            }
            // Cross-file: claimed by another state file too. The current
            // file's own index entry is excluded by id; without a parseable
            // id the check is skipped rather than run against stale self data.
            if let Some(self_id) = self.self_id {
                let mut other: Option<u32> = None;
                for entry in ctx.states.iter() {
                    let id = *entry.key();
                    if id != self_id && entry.value().provinces.contains(prov) {
                        other = Some(id);
                        break;
                    }
                }
                if let Some(other_id) = other {
                    push_state_warning(
                        ctx,
                        range,
                        crate::validation::advanced_validation::PROVINCE_IN_TWO_STATES,
                        format!("Province {} is also in state {}", prov, other_id),
                        diags,
                    );
                }
            }
        }

        // Duplicate victory points.
        let mut seen_vp: HashSet<u32> = HashSet::new();
        for (prov, range) in &self.vp_firsts {
            if !seen_vp.insert(*prov) {
                push_state_warning(
                    ctx,
                    range,
                    crate::validation::advanced_validation::DUPLICATE_VICTORY_POINT,
                    format!("Duplicate victory_points entry for province {}", prov),
                    diags,
                );
            }
        }

        // Province-keyed building placements.
        for placement in &self.placements {
            if !members.contains(&placement.province) {
                push_state_warning(
                    ctx,
                    &placement.province_range,
                    crate::validation::advanced_validation::PROVINCE_BUILDING_OUTSIDE_STATE,
                    format!(
                        "Building '{}' placed on province {} which is not in this state",
                        placement.building, placement.province
                    ),
                    diags,
                );
                continue;
            }
            if !use_provinces {
                continue;
            }
            let coastal_only = ctx
                .buildings
                .get(placement.building.as_str())
                .map(|b| b.coastal_only)
                .unwrap_or(false);
            if !coastal_only {
                continue;
            }
            let inland = ctx
                .provinces
                .get(&placement.province)
                .map(|p| !p.is_coastal)
                .unwrap_or(false);
            if inland {
                push_state_warning(
                    ctx,
                    &placement.building_range,
                    crate::validation::advanced_validation::COASTAL_BUILDING_ON_INLAND,
                    format!(
                        "Building '{}' requires a coastal province (province {} is inland)",
                        placement.building, placement.province
                    ),
                    diags,
                );
            }
        }
    }
}

impl StateDefinitionRule {
    pub(crate) fn map_visitor(uri: &str) -> Box<dyn AstVisitor> {
        Box::new(StateMapVisitor::new(uri))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scope::scope::Scope;

    const STATE_URI: &str = "/mod/history/states/1-Test.txt";

    use tower_lsp_server::ls_types::NumberOrString;

    fn map_diags(ctx: &crate::test_support::TestCtx, input: &str, uri: &str) -> Vec<Diagnostic> {
        ctx.walk(
            input,
            uri,
            Scope::Global,
            vec![],
            vec![StateDefinitionRule::map_visitor(uri)],
        )
    }

    fn has_code(diags: &[Diagnostic], code: &str) -> bool {
        diags.iter().any(|d| {
            d.code == Some(NumberOrString::String(code.to_string()))
                && d.severity == Some(DiagnosticSeverity::WARNING)
        })
    }

    // ── HOM2002 sea-in-state (lakes legal) ──
    #[test]
    fn test_sea_province_in_state_fires_lake_does_not() {
        let ctx = crate::test_support::TestCtx::new().with_provinces(&[
            (5, "sea", true),
            (6, "lake", false),
            (7, "land", false),
        ]);
        let diags = map_diags(&ctx, "state = { id = 1 provinces = { 5 6 7 } }", STATE_URI);
        assert_eq!(diags.len(), 1, "only the sea province fires: {:?}", diags);
        assert!(has_code(&diags, "HOM2002"));
        assert!(diags[0].message.contains('5'));
    }

    // ── HOM2009 unknown province + empty-map guard ──
    #[test]
    fn test_unknown_province_fires_only_with_definition_data() {
        let ctx = crate::test_support::TestCtx::new().with_provinces(&[(7, "land", false)]);
        let diags = map_diags(&ctx, "state = { id = 1 provinces = { 7 999 } }", STATE_URI);
        assert!(
            has_code(&diags, "HOM2009"),
            "999 has no csv row: {:?}",
            diags
        );

        // No definition.csv loaded (fresh mod, game path unset): fail closed.
        let bare = crate::test_support::TestCtx::new();
        let diags = map_diags(&bare, "state = { id = 1 provinces = { 7 999 } }", STATE_URI);
        assert!(diags.is_empty(), "no signal without csv data: {:?}", diags);
    }

    // ── HOM2004 duplicate VP, incl. across repeated blocks ──
    #[test]
    fn test_duplicate_victory_points_across_repeated_blocks() {
        let ctx = crate::test_support::TestCtx::new().with_provinces(&[(7, "land", false)]);
        let diags = map_diags(
            &ctx,
            "state = { id = 1 provinces = { 7 } history = { \
               victory_points = { 7 5 } victory_points = { 7 3 } } }",
            STATE_URI,
        );
        assert!(has_code(&diags, "HOM2004"), "dup VP: {:?}", diags);

        let diags = map_diags(
            &ctx,
            "state = { id = 1 provinces = { 7 } history = { victory_points = { 7 5 } } }",
            STATE_URI,
        );
        assert!(
            !has_code(&diags, "HOM2004"),
            "single VP is fine: {:?}",
            diags
        );
    }

    // ── HOM2005 foreign building placement ──
    #[test]
    fn test_province_building_outside_state() {
        let ctx = crate::test_support::TestCtx::new()
            .with_provinces(&[(7, "land", false), (8, "land", false)])
            .with_buildings(&[("naval_base", Some(6), true)]);
        let diags = map_diags(
            &ctx,
            "state = { id = 1 provinces = { 7 } history = { \
               buildings = { 8 = { naval_base = 1 } } } }",
            STATE_URI,
        );
        assert!(
            has_code(&diags, "HOM2005"),
            "prov 8 not a member: {:?}",
            diags
        );
    }

    // ── HOM2007 coastal-only building ──
    #[test]
    fn test_coastal_building_on_inland_fires_coastal_does_not() {
        let ctx = crate::test_support::TestCtx::new()
            .with_provinces(&[(7, "land", false), (9, "land", true)])
            .with_buildings(&[
                ("naval_base", Some(6), true),
                ("arms_factory", Some(3), false),
            ]);
        // Inland placement fires.
        let diags = map_diags(
            &ctx,
            "state = { id = 1 provinces = { 7 } history = { \
               buildings = { 7 = { naval_base = 1 } } } }",
            STATE_URI,
        );
        assert!(
            has_code(&diags, "HOM2007"),
            "inland naval_base: {:?}",
            diags
        );
        // Coastal placement is fine.
        let diags = map_diags(
            &ctx,
            "state = { id = 1 provinces = { 9 } history = { \
               buildings = { 9 = { naval_base = 1 } } } }",
            STATE_URI,
        );
        assert!(
            !has_code(&diags, "HOM2007"),
            "coastal naval_base: {:?}",
            diags
        );
        // Non-coastal-gated building on inland is fine.
        let diags = map_diags(
            &ctx,
            "state = { id = 1 provinces = { 7 } history = { \
               buildings = { 7 = { arms_factory = 1 } } } }",
            STATE_URI,
        );
        assert!(diags.is_empty(), "arms_factory inland: {:?}", diags);
        // Unknown building: no pile-on (name check lives elsewhere).
        let diags = map_diags(
            &ctx,
            "state = { id = 1 provinces = { 7 } history = { \
               buildings = { 7 = { mystery_bldg = 1 } } } }",
            STATE_URI,
        );
        assert!(!has_code(&diags, "HOM2007"), "unknown bldg: {:?}", diags);
    }

    // ── HOM2008 empty state ──
    #[test]
    fn test_empty_state() {
        let ctx = crate::test_support::TestCtx::new();
        let diags = map_diags(&ctx, "state = { id = 1 provinces = { } }", STATE_URI);
        assert!(has_code(&diags, "HOM2008"), "empty: {:?}", diags);
        let diags = map_diags(&ctx, "state = { id = 1 provinces = { 7 } }", STATE_URI);
        assert!(!has_code(&diags, "HOM2008"), "non-empty: {:?}", diags);
    }

    // ── HOM2003 province in two states (via the real incremental path) ──
    #[test]
    fn test_province_in_two_states() {
        let ctx = crate::test_support::TestCtx::new()
            .with_provinces(&[(7, "land", false), (8, "land", false)])
            .with_file(
                "/mod/history/states/2-Other.txt",
                "state = { id = 2 provinces = { 7 } }",
            );
        // Sibling claims 7; walking state 1 (also claiming 7) must flag it.
        let diags = map_diags(
            &ctx,
            "state = { id = 1 provinces = { 7 8 } }",
            "/mod/history/states/1-Test.txt",
        );
        let two_state: Vec<_> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("HOM2003".to_string())))
            .collect();
        assert_eq!(two_state.len(), 1, "only shared prov 7: {:?}", diags);
        assert!(two_state[0].message.contains('2'));
    }

    #[test]
    fn test_no_two_states_for_disjoint_or_self_claim() {
        let ctx = crate::test_support::TestCtx::new()
            .with_provinces(&[(7, "land", false), (8, "land", false)])
            .with_file(
                "/mod/history/states/2-Other.txt",
                "state = { id = 2 provinces = { 7 } }",
            );
        // Disjoint claim — clean.
        let diags = map_diags(
            &ctx,
            "state = { id = 3 provinces = { 8 } }",
            "/mod/history/states/3-Test.txt",
        );
        assert!(!has_code(&diags, "HOM2003"), "disjoint: {:?}", diags);
        // Same content as the indexed file, own entry excluded by id — clean.
        let diags = map_diags(
            &ctx,
            "state = { id = 2 provinces = { 7 } }",
            "/mod/history/states/2-Other.txt",
        );
        assert!(!has_code(&diags, "HOM2003"), "self excluded: {:?}", diags);
    }

    // ── URI gate ──
    #[test]
    fn test_visitor_inert_outside_state_files() {
        let ctx = crate::test_support::TestCtx::new().with_provinces(&[(5, "sea", true)]);
        let diags = map_diags(
            &ctx,
            "state = { id = 1 provinces = { 5 } }",
            "/mod/common/decisions/test.txt",
        );
        assert!(diags.is_empty(), "gated off: {:?}", diags);
    }

    // ── Vanilla-shaped file stays clean ──
    #[test]
    fn test_vanilla_shaped_state_is_clean() {
        let ctx = crate::test_support::TestCtx::new()
            .with_provinces(&[(130, "land", true), (11129, "land", false)])
            .with_buildings(&[
                ("infrastructure", Some(5), false),
                ("industrial_complex", Some(1), false),
                ("air_base", Some(6), false),
                ("dam", Some(5), false),
                ("naval_base", Some(6), true),
            ]);
        let input = "state = { id = 38 manpower = 931955 \
            history = { owner = SWE \
              victory_points = { 130 1 } \
              buildings = { infrastructure = 2 11129 = { dam = 1 } 130 = { naval_base = 1 } } } \
            provinces = { 130 11129 } }";
        let diags = map_diags(&ctx, input, STATE_URI);
        assert!(
            diags.is_empty(),
            "vanilla-shaped must be clean: {:?}",
            diags
        );
    }
}
