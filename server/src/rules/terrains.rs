use crate::parser::ast;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::ScopeStack;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates terrain type usage across HOI4 mod files.
///
/// Per-entry checks:
/// - `naval_terrain = X` in strategic region definitions → warns if X is not a
///   known terrain category with `naval_terrain = yes`
///
/// NOTE: there is deliberately NO cross-check of definition.csv terrain cells
/// here. That condition is covered once, with exact cell ranges, by
/// `Backend::check_province_terrain_csv` on the csv itself. A duplicate here
/// would have no AST node to point at (provinces live in another file) and
/// would stack one range-less diag per province on every terrain file.
pub(crate) struct TerrainRule;

impl ValidationRule for TerrainRule {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        _pushed_scope: bool,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key_lower = ass.key_text(ctx.source).to_ascii_lowercase();

        // Validate naval_terrain = <value> in strategic region definitions
        if key_lower == "naval_terrain" {
            if let Some(value_str) = extract_string_value(&ass.value, ctx.source) {
                if !ctx.terrain_categories.is_empty()
                    && !ctx.terrain_categories.iter().any(|entry| {
                        let terrain = entry.value();
                        terrain.name == value_str && terrain.is_naval
                    })
                {
                    let known = format_naval_terrains(ctx);
                    diags.push(Diagnostic {
                        range: ctx.range(&ass.value.range),
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
}

/// Extract a string value from a `NodeedValue`.
fn extract_string_value<'a>(val: &'a ast::NodeedValue, source: &'a str) -> Option<&'a str> {
    val.value.as_str(source)
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
