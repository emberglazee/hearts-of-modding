use crate::parser::ast;
use crate::rules::visitor::AstVisitor;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scanner::event_namespace_scanner;
use crate::scope::scope::ScopeStack;
use crate::utils::lsp_convert::ast_range_to_lsp;
use std::str::FromStr;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Uri};

/// State tracked for a single event option definition during validation.
struct EventOptionDef {
    /// Range of the `option` key (for diagnostic positioning).
    key_range: ast::Range,
    /// Whether this option has a `name` field.
    has_name: bool,
    /// Whether this option has an `ai_chance` block.
    has_ai_chance: bool,
}

/// AstVisitor that validates event option blocks (`option = { ... }`) inside
/// event definitions (`country_event`, `state_event`, etc.).
///
/// Checks performed:
/// - HOM3013 (EVENT_MISSING_OPTION_NAME): Warns when an option has no `name` field.
/// - HOM3017 (EVENT_OPTION_MISSING_AI_CHANCE): Information when an option has no
///   `ai_chance` block (AI may not choose optimally).
struct EventOptionVisitor {
    /// Depth of event definition nesting (>0 means inside an event definition).
    /// Event definitions cannot nest, but event-type keys also serve as effects
    /// inside options (e.g. `country_event = { id = ... days = ... }`).
    /// We increment for any `country_event = { ... }` with a block value and
    /// decrement on exit, so nested effects are tracked correctly.
    event_depth: u32,
    /// Whether the current event has `major = yes` set.
    _event_is_major: bool,
    /// Stack of option definitions currently being walked.
    option_stack: Vec<EventOptionDef>,
}

impl EventOptionVisitor {
    fn new() -> Self {
        Self {
            event_depth: 0,
            _event_is_major: false,
            option_stack: Vec::new(),
        }
    }

    /// Returns `true` if `key` is an event definition type.
    fn is_event_type(key: &str) -> bool {
        matches!(
            key,
            "country_event"
                | "state_event"
                | "news_event"
                | "unit_leader_event"
                | "operative_leader_event"
        )
    }

    fn in_option(&self) -> bool {
        !self.option_stack.is_empty()
    }

    fn validate_option(
        &self,
        state: &EventOptionDef,
        _ctx: &ValidationContext,
        diags: &mut Vec<Diagnostic>,
    ) {
        // HOM3013: Option without a name field
        if !state.has_name {
            diags.push(Diagnostic {
                range: ast_range_to_lsp(&state.key_range),
                severity: Some(DiagnosticSeverity::WARNING),
                message: "Event option is missing a 'name' field. Players will not see \
                          a descriptive label for this option."
                    .to_string(),
                code: Some(NumberOrString::String(
                    crate::validation::advanced_validation::EVENT_MISSING_OPTION_NAME.to_string(),
                )),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        }

        // HOM3017: Option without ai_chance (AI guidance)
        if !state.has_ai_chance {
            diags.push(Diagnostic {
                range: ast_range_to_lsp(&state.key_range),
                severity: Some(DiagnosticSeverity::INFORMATION),
                message: "Event option is missing an 'ai_chance' block. \
                          The AI may not choose this option optimally."
                    .to_string(),
                code: Some(NumberOrString::String(
                    crate::validation::advanced_validation::EVENT_OPTION_MISSING_AI_CHANCE
                        .to_string(),
                )),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        }
    }
}

impl AstVisitor for EventOptionVisitor {
    fn enter_assignment(
        &mut self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        _diags: &mut Vec<Diagnostic>,
    ) {
        let key = ass.key_text(ctx.source);

        // ── Detect event definition entry ──────────────────────────
        if Self::is_event_type(key) && matches!(&ass.value.value, ast::Value::Block(_)) {
            self.event_depth += 1;
            return;
        }

        if self.event_depth == 0 {
            return;
        }

        // ── Track `major` flag at event level (not inside options) ──
        if !self.in_option() && key.eq_ignore_ascii_case("major") {
            self._event_is_major = ass.value.value.as_str(ctx.source) == Some("yes");
            return;
        }

        // ── Detect option definition entry (only at event level) ────
        if !self.in_option()
            && key.eq_ignore_ascii_case("option")
            && matches!(&ass.value.value, ast::Value::Block(_))
        {
            self.option_stack.push(EventOptionDef {
                key_range: ass.key_range.clone(),
                has_name: false,
                has_ai_chance: false,
            });
            return;
        }

        // ── Inside an option: track properties ──────────────────────
        if self.in_option() {
            if let Some(state) = self.option_stack.last_mut() {
                match key.to_ascii_lowercase().as_str() {
                    "name" => {
                        state.has_name = true;
                    }
                    "ai_chance" if matches!(&ass.value.value, ast::Value::Block(_)) => {
                        state.has_ai_chance = true;
                    }
                    _ => {}
                }
            }
        }
    }

    fn exit_assignment(
        &mut self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key = ass.key_text(ctx.source);

        // ── Exiting option block ─────────────────────────────────────
        if self.in_option()
            && key.eq_ignore_ascii_case("option")
            && matches!(&ass.value.value, ast::Value::Block(_))
        {
            if let Some(state) = self.option_stack.pop() {
                self.validate_option(&state, ctx, diags);
            }
            return;
        }

        // ── Exiting event definition ─────────────────────────────────
        if self.event_depth > 0
            && Self::is_event_type(key)
            && matches!(&ass.value.value, ast::Value::Block(_))
        {
            self.event_depth -= 1;
            if self.event_depth == 0 {
                self._event_is_major = false;
            }
        }
    }
}

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
                    // Compare as canonicalized filesystem paths to handle
                    // both the URI-vs-path format mismatch (file:///foo vs /foo)
                    // and symlinked paths pointing to the same physical file.
                    let other_path = &*entry.value().resolve().path;
                    let same_file = match Uri::from_str(ctx.uri) {
                        Ok(uri) => match uri.to_file_path() {
                            Some(path) => {
                                let current = path.into_owned().canonicalize().ok();
                                let stored = std::path::Path::new(other_path).canonicalize().ok();
                                current.zip(stored).map(|(c, o)| c == o).unwrap_or(false)
                            }
                            None => false,
                        },
                        Err(_) => false,
                    };
                    if !same_file {
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

impl EventValidationRule {
    pub(crate) fn visitor() -> Box<dyn AstVisitor> {
        Box::new(EventOptionVisitor::new())
    }
}
