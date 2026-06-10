use crate::parser::ast;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scanner::event_namespace_scanner;
use crate::scope::scope::ScopeStack;
use crate::utils::lsp_convert::ast_range_to_lsp;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates event definitions for correct namespace usage and ID format.
///
/// Checks:
/// - Event IDs that use a namespace not declared via `add_namespace` → HOM3008
/// - Non-integer event IDs (e.g. `my_event.abc`) → HOM3009
/// - Numeric event ID >= 100000 → HOM3010
/// - Duplicate event IDs → HOM3011
/// - Duplicate `add_namespace` declarations → HOM3012
///
/// The `add_namespace` diagnostic (HOM3012) is produced at the block level
/// (cross-file), while the others are per-assignment.
pub(crate) struct EventValidationRule;

impl ValidationRule for EventValidationRule {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        _pushed_scope: bool,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key = ass.key_text(ctx.source);

        // ── Check for add_namespace duplication ─────────────────
        if key == "add_namespace" {
            if let Some(name) = ass.value.value.as_str(ctx.source) {
                // The scanner data has the namespace if it was declared.
                // If we're looking at a file that declares it, and it was
                // ALSO declared in another file, that's a duplicate.
                let ns_entry = ctx.event_namespaces.get(name);
                if let Some(entry) = ns_entry {
                    // The entry exists — check if the file paths differ.
                    // If the same file re-declares it, that's also a duplicate.
                    let this_path = ctx.uri;
                    let other_path = &*entry.value().resolve().path;
                    if other_path != this_path {
                        diags.push(Diagnostic {
                            range: ast_range_to_lsp(&ass.value.range),
                            severity: Some(DiagnosticSeverity::WARNING),
                            message: format!(
                                "Duplicate event namespace '{}' (also declared in {})",
                                name, other_path
                            ),
                            code: Some(NumberOrString::String(
                                crate::validation::advanced_validation::DUPLICATE_EVENT_NAMESPACE
                                    .to_string(),
                            )),
                            source: Some("Hearts of Modding".to_string()),
                            ..Default::default()
                        });
                    }
                }
            }
            return;
        }

        // ── Check event IDs ──────────────────────────────────────
        if key != "country_event"
            && key != "state_event"
            && key != "news_event"
            && key != "unit_leader_event"
            && key != "operative_leader_event"
        {
            return;
        }

        // Extract the `id = ...` from the event block, or the string value itself
        let id_str = match &ass.value.value {
            ast::Value::String(span) => Some(span.resolve(ctx.source)),
            ast::Value::Block(entries) => entries.iter().find_map(|e| {
                if let ast::Entry::Assignment(inner_ass) = e {
                    if inner_ass.key_text(ctx.source) == "id" {
                        inner_ass.value.value.as_str(ctx.source)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }),
            _ => None,
        };

        let Some(id) = id_str else {
            return;
        };

        // Parse the event ID
        let Some(parsed) = event_namespace_scanner::parse_event_id(id) else {
            // Cannot parse at all — the parser may have already caught this.
            return;
        };

        // Check 1: Non-integer event ID
        if !parsed.is_valid_integer {
            diags.push(Diagnostic {
                range: ast_range_to_lsp(&ass.value.range),
                severity: Some(DiagnosticSeverity::WARNING),
                message: format!(
                    "Event ID '{}' has non-integer suffix '{}'. Event IDs must be in the format \
                     <namespace>.<integer> (e.g. 'my_event.123'). Non-integer IDs cause duplicate \
                     internal event IDs (all become ID 0).",
                    id, parsed.numeric_raw
                ),
                code: Some(NumberOrString::String(
                    crate::validation::advanced_validation::NON_INTEGER_EVENT_ID.to_string(),
                )),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
            return; // Can't check further without a valid integer
        }

        // Check 2: Event ID too large (>= 100000)
        if let Some(n) = parsed.numeric_value {
            if n >= 100_000 {
                diags.push(Diagnostic {
                    range: ast_range_to_lsp(&ass.value.range),
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: format!(
                        "Event ID '{}' uses numeric ID {}, which is >= 100000. This encroaches \
                         on other namespace's internal ID range and may cause duplicate ID conflicts.",
                        id, n
                    ),
                    code: Some(NumberOrString::String(
                        crate::validation::advanced_validation::EVENT_ID_TOO_LARGE.to_string(),
                    )),
                    source: Some("Hearts of Modding".to_string()),
                    ..Default::default()
                });
            }
        }

        // Check 3: Missing namespace declaration
        let namespace_str = parsed.namespace;
        // Allow event IDs without a namespace part (e.g. just "12345" — legacy IDs)
        // Only flag if there IS a namespace part and it's NOT declared.
        if !namespace_str.is_empty()
            && !ctx.event_namespaces.contains_key(namespace_str)
            && namespace_str.chars().any(|c| !c.is_ascii_digit())
        {
            diags.push(Diagnostic {
                range: ast_range_to_lsp(&ass.value.range),
                severity: Some(DiagnosticSeverity::WARNING),
                message: format!(
                    "Event ID '{}' uses namespace '{}' which has not been declared. \
                     Add 'add_namespace = {}' before any events using this namespace.",
                    id, namespace_str, namespace_str
                ),
                code: Some(NumberOrString::String(
                    crate::validation::advanced_validation::MISSING_EVENT_NAMESPACE.to_string(),
                )),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        }
    }
}
