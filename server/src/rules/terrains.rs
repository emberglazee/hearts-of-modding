use crate::parser::ast;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::ScopeStack;
use crate::utils::lsp_convert::ast_range_to_lsp;
use std::collections::HashSet;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates terrain type usage across HOI4 mod files.
///
/// Per-entry checks:
/// - `naval_terrain = X` in strategic region definitions → warns if X is not a
///   known terrain category with `naval_terrain = yes`
///
/// Block-level checks:
/// - When editing `common/terrain/*.txt`, cross-references all province terrains
///   from `definition.csv` against known terrain categories — flags provinces
///   using undefined terrain names so the modder sees them in the terrain editor.
pub(crate) struct TerrainRule;

impl ValidationRule for TerrainRule {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key_lower = ass.key.to_ascii_lowercase();

        // Validate naval_terrain = <value> in strategic region definitions
        if key_lower == "naval_terrain" {
            if let Some(value_str) = extract_string_value(&ass.value) {
                if !ctx.terrain_categories.is_empty()
                    && !ctx.terrain_categories.iter().any(|entry| {
                        let terrain = entry.value();
                        terrain.name == value_str && terrain.is_naval
                    })
                {
                    let known = format_naval_terrains(ctx);
                    diags.push(Diagnostic {
                        range: ast_range_to_lsp(&ass.value.range),
                        severity: Some(DiagnosticSeverity::WARNING),
                        message: format!(
                            "Unknown naval terrain '{}'{}. Naval terrains must be defined in common/terrain/*.txt with naval_terrain = yes",
                            value_str,
                            if known.is_empty() {
                                String::new()
                            } else {
                                format!(". Known: {}", known)
                            },
                        ),
                        code: Some(NumberOrString::String(
                            crate::validation::advanced_validation::UNKNOWN_NAVAL_TERRAIN
                                .to_string(),
                        )),
                        source: Some("Hearts of Modding".to_string()),
                        ..Default::default()
                    });
                }
            }
        }
    }

    fn check_block(
        &self,
        _entries: &[ast::Entry],
        ctx: &ValidationContext,
        diags: &mut Vec<Diagnostic>,
    ) {
        // When editing a terrain definition file, cross-validate all province
        // terrain values from definition.csv against known terrain categories.
        if !ctx.uri.contains("/common/terrain/") {
            return;
        }

        let terrain_names: HashSet<String> = ctx
            .terrain_categories
            .iter()
            .map(|entry| entry.key().to_string())
            .collect();

        if terrain_names.is_empty() || ctx.provinces.is_empty() {
            return;
        }

        for entry in ctx.provinces.iter() {
            let province = entry.value();
            // Note: the province scanner stores terrain from definition.csv's
            // column 6 in `.prov_type` (see scanner/province_scanner.rs).
            let prov_terrain = province.prov_type.trim().to_lowercase();
            if !prov_terrain.is_empty() && !terrain_names.contains(&prov_terrain) {
                diags.push(Diagnostic {
                    range: crate::utils::lsp_convert::ast_range_to_lsp(
                        &crate::parser::ast::Range {
                            start_line: 0,
                            start_col: 0,
                            end_line: 0,
                            end_col: 0,
                        },
                    ),
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: format!(
                        "Province {} uses unknown terrain '{}'. Terrains are defined in common/terrain/*.txt",
                        province.id,
                        prov_terrain,
                    ),
                    code: Some(NumberOrString::String(
                        crate::validation::advanced_validation::UNKNOWN_PROVINCE_TERRAIN
                            .to_string(),
                    )),
                    source: Some("Hearts of Modding".to_string()),
                    ..Default::default()
                });
            }
        }
    }
}

/// Extract a string value from a `NodeedValue`.
fn extract_string_value(val: &ast::NodeedValue) -> Option<String> {
    if let ast::Value::String(s) = &val.value {
        Some(s.clone())
    } else {
        None
    }
}

/// Build a comma-separated list of known naval terrain categories.
fn format_naval_terrains(ctx: &ValidationContext) -> String {
    let mut names: Vec<String> = ctx
        .terrain_categories
        .iter()
        .filter(|entry| entry.value().is_naval)
        .map(|entry| entry.key().to_string())
        .collect();
    names.sort();
    names.join(", ")
}
