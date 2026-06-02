use crate::parser::ast;
use crate::rules::{ValidationContext, ValidationRule};
use crate::utils::lsp_convert::ast_range_to_lsp;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates AI area definitions: checks that continents and strategic
/// regions referenced in `/common/ai_areas/` files actually exist in
/// the scanner data.
pub(crate) struct AiAreaRule;

impl ValidationRule for AiAreaRule {
    fn check_block(
        &self,
        entries: &[ast::Entry],
        ctx: &ValidationContext,
        diags: &mut Vec<Diagnostic>,
    ) {
        let uri = ctx.uri;
        if !uri.contains("/common/ai_areas/") && !uri.contains("\\common\\ai_areas\\") {
            return;
        }

        for entry in entries {
            let ast::Entry::Assignment(ass) = entry else {
                continue;
            };
            let ast::Value::Block(inner_entries) = &ass.value.value else {
                continue;
            };

            for inner in inner_entries {
                let ast::Entry::Assignment(inner_ass) = inner else {
                    continue;
                };

                match inner_ass.key.as_str() {
                    "continents" => {
                        if let ast::Value::Block(cont_entries) = &inner_ass.value.value {
                            for ce in cont_entries {
                                if let ast::Entry::Value(val) = ce {
                                    if let ast::Value::String(name) = &val.value {
                                        if !ctx.continents.contains_key(name.as_str()) {
                                            diags.push(Diagnostic {
                                                range: ast_range_to_lsp(&val.range),
                                                severity: Some(DiagnosticSeverity::WARNING),
                                                message: format!("Unknown continent: '{}'", name),
                                                code: Some(NumberOrString::String(
                                                    "HOM6001".to_string(),
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
                    "strategic_regions" => {
                        if let ast::Value::Block(sr_entries) = &inner_ass.value.value {
                            for se in sr_entries {
                                if let ast::Entry::Value(val) = se {
                                    if let ast::Value::Number(n) = &val.value {
                                        let id = *n as u32;
                                        if !ctx.strategic_regions.contains_key(&id) {
                                            diags.push(Diagnostic {
                                                range: ast_range_to_lsp(&val.range),
                                                severity: Some(DiagnosticSeverity::WARNING),
                                                message: format!(
                                                    "Unknown strategic region: {}",
                                                    id
                                                ),
                                                code: Some(NumberOrString::String(
                                                    "HOM6002".to_string(),
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
                    _ => {}
                }
            }
        }
    }
}
