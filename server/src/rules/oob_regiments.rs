use crate::data::interner::InternedStr;
use crate::data::layered_value::LayeredValue;
use crate::parser::ast;
use crate::rules::ValidationContext;
use crate::rules::visitor::AstVisitor;
use crate::scanner::unit_scanner::UnitType;
use crate::scope::scope::ScopeStack;
use crate::utils::lsp_convert::ast_range_to_lsp;
use dashmap::DashMap;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates that:
///
/// 1. `regiments = { ... }` and `support = { ... }` entries in division
///    templates reference valid unit types defined in `common/units/*.txt`.
///
/// 2. `division_template = "..."` references in `division = { ... }` blocks
///    (inside `units = { ... }`) refer to a template defined in the same file.
///
/// Follows the AstVisitor pattern with `after_walk` for cross-reference checks.
pub(crate) struct OobRegimentVisitor {
    /// True when currently inside a `regiments = { ... }` or `support = { ... }` block
    in_slot: bool,

    // ── Template cross-reference state ──────────────────────────────
    /// Template names collected from `division_template = { name = "..." }`
    template_names: Vec<String>,
    /// Template references: (referenced_name, key_range) from
    /// `division_template = "..."` inside `unit_division` context
    template_refs: Vec<(String, ast::Range)>,

    /// True inside a `division_template = { ... }` block (template definition)
    in_template_def: bool,
    /// True inside a `units = { ... }` block
    in_units_block: bool,
    /// True inside a `division = { ... }` block inside `units`
    in_unit_division: bool,
}

impl OobRegimentVisitor {
    pub fn new() -> Self {
        Self {
            in_slot: false,
            template_names: Vec::new(),
            template_refs: Vec::new(),
            in_template_def: false,
            in_units_block: false,
            in_unit_division: false,
        }
    }

    pub fn visitor() -> Box<dyn AstVisitor> {
        Box::new(Self::new())
    }
}

/// Find the canonical (as-defined) casing for a unit type key.
/// Returns `None` if no unit type matches case-insensitively.
fn find_canonical_unit_type(
    unit_types: &DashMap<InternedStr, LayeredValue<UnitType>>,
    key: &str,
) -> Option<String> {
    let key_lower = key.to_ascii_lowercase();
    for entry in unit_types.iter() {
        if entry.key().to_ascii_lowercase() == key_lower {
            return Some(entry.key().to_string());
        }
    }
    None
}

impl AstVisitor for OobRegimentVisitor {
    fn enter_assignment(
        &mut self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key_lower = ass.key_text(ctx.source).to_ascii_lowercase();

        // ── Track structural context ────────────────────────────────

        // Handle structural keys with early return.
        // IMPORTANT: "division_template" can be either a Block (definition)
        // or a QuotedString (reference inside units > division).
        // Both must be handled here before falling through to regiment checks.
        match key_lower.as_str() {
            "division_template" => {
                if matches!(
                    &ass.value.value,
                    ast::Value::Block(_) | ast::Value::TaggedBlock(..)
                ) {
                    // Template definition: `division_template = { ... }`
                    self.in_template_def = true;
                } else if self.in_unit_division {
                    // Template reference: `division_template = "Name"` inside
                    // a `division = { ... }` block within `units = { ... }`
                    if let Some(ref_name) = ass.value.value.as_str(ctx.source) {
                        self.template_refs
                            .push((ref_name.to_string(), ass.key_range.clone()));
                    }
                }
                return;
            }
            "units" => {
                if matches!(
                    &ass.value.value,
                    ast::Value::Block(_) | ast::Value::TaggedBlock(..)
                ) {
                    self.in_units_block = true;
                }
                return;
            }
            "division" => {
                if self.in_units_block
                    && matches!(
                        &ass.value.value,
                        ast::Value::Block(_) | ast::Value::TaggedBlock(..)
                    )
                {
                    self.in_unit_division = true;
                }
                return;
            }
            _ => {}
        }

        // ── Template definition: collect `name = "..."` ─────────────
        if self.in_template_def && key_lower == "name" {
            if let Some(name) = ass.value.value.as_str(ctx.source) {
                // QuotedString resolves the content without quotes
                self.template_names.push(name.to_string());
            }
            return;
        }

        // ── Regiment/support unit type validation ───────────────────
        if matches!(key_lower.as_str(), "regiments" | "support") {
            if matches!(
                &ass.value.value,
                ast::Value::Block(_) | ast::Value::TaggedBlock(..)
            ) {
                self.in_slot = true;
            }
            return;
        }

        // When inside a slot block, each child assignment with a Block value
        // is a unit type reference (e.g. infantry = { x = 0 y = 0 }).
        // Scalar entries like x = 0 and y = 0 are coordinate properties, not unit types.
        if self.in_slot {
            if matches!(
                &ass.value.value,
                ast::Value::Block(_) | ast::Value::TaggedBlock(..)
            ) {
                let unit_key = ass.key_text(ctx.source);
                // HOI4 Clausewitz engine is case-insensitive for identifiers,
                // so we offer 3 tiers of diagnostics:
                //
                //   Tier 1 (exact match) → key exists as-is → no diagnostic
                //   Tier 2 (casing differs) → key matches but with wrong casing → HINT
                //   Tier 3 (no match) → completely unknown unit type → WARNING
                if ctx.unit_types.contains_key(unit_key) {
                    // Tier 1: exact match → all good
                } else if let Some(canonical) = find_canonical_unit_type(ctx.unit_types, unit_key) {
                    // Tier 2: exists with different casing
                    diags.push(Diagnostic {
                        range: ast_range_to_lsp(&ass.key_range),
                        severity: Some(DiagnosticSeverity::HINT),
                        message: format!(
                            "Unit type '{}' should probably be '{}' (the game accepts \
                             both, but consistent casing is good practice)",
                            unit_key, canonical
                        ),
                        code: Some(NumberOrString::String(
                            crate::validation::advanced_validation::UNIT_TYPE_CASE_MISMATCH
                                .to_string(),
                        )),
                        source: Some("Hearts of Modding".to_string()),
                        data: Some(serde_json::Value::String(canonical.clone())),
                        ..Default::default()
                    });
                } else {
                    // Tier 3: completely unknown
                    diags.push(Diagnostic {
                        range: ast_range_to_lsp(&ass.key_range),
                        severity: Some(DiagnosticSeverity::WARNING),
                        message: format!(
                            "Unknown unit type: '{}' — not defined in common/units/*.txt",
                            unit_key
                        ),
                        code: Some(NumberOrString::String(
                            crate::validation::advanced_validation::UNKNOWN_UNIT_TYPE.to_string(),
                        )),
                        source: Some("Hearts of Modding".to_string()),
                        ..Default::default()
                    });
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
        let key_lower = ass.key_text(ctx.source).to_ascii_lowercase();

        match key_lower.as_str() {
            "division_template" => {
                self.in_template_def = false;
            }
            "units" => {
                self.in_units_block = false;
            }
            "division" if self.in_unit_division => {
                self.in_unit_division = false;
            }
            "regiments" | "support" => {
                self.in_slot = false;
            }
            _ => {}
        }
    }

    fn after_walk(&mut self, _ctx: &ValidationContext, diags: &mut Vec<Diagnostic>) {
        // Cross-reference: check that every referenced template name
        // has a matching `division_template = { name = "..." }` definition.
        for (ref_name, key_range) in &self.template_refs {
            let exists = self.template_names.iter().any(|n| n == ref_name);
            if !exists {
                diags.push(Diagnostic {
                    range: ast_range_to_lsp(key_range),
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: format!(
                        "Unknown division template: '{}' — no template with that name is defined in this file",
                        ref_name
                    ),
                    code: Some(NumberOrString::String(
                        crate::validation::advanced_validation::UNKNOWN_DIVISION_TEMPLATE
                            .to_string(),
                    )),
                    source: Some("Hearts of Modding".to_string()),
                    ..Default::default()
                });
            }
        }
    }
}
