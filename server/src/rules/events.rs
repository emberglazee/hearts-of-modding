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

/// State tracked for the event itself.
struct EventDef {
    key_range: ast::Range,
    has_title: bool,
    has_desc: bool,
    has_picture: bool,
    is_hidden: bool,
    has_mtth: bool,
    has_is_triggered_only: bool,
    /// Whether this block contains at least one `option = { ... }`.
    /// Event definitions always have options; `country_event = { ... }`
    /// used as an effect never does. Only validate definition usage.
    has_option: bool,
    /// Loc key extracted from `title = "..."` or `title = key` (not block form).
    title_key: Option<String>,
    /// Loc key extracted from `desc = "..."` or `desc = key` (not block form).
    desc_key: Option<String>,
    /// Sprite name extracted from `picture = GFX_...` (not quoted or block form).
    picture_sprite: Option<String>,
}

/// AstVisitor that validates event structure and option blocks.
///
/// Checks performed at the event level:
/// - HOM3016 (EVENT_MISSING_TITLE): Warns when a non-hidden event lacks both `title` and `desc`.
/// - HOM3018 (EVENT_MISSING_TITLE_LOC): Warns when the `title` localization key is missing.
/// - HOM3019 (EVENT_MISSING_DESC_LOC): Warns when the `desc` localization key is missing.
/// - HOM3020 (EVENT_PICTURE_SPRITE_NOT_FOUND): Warns when `picture` references an unknown sprite.
///
/// Checks performed at the option level (existing):
/// - HOM3013 (EVENT_MISSING_OPTION_NAME): Warns when an option has no `name` field.
/// - HOM3017 (EVENT_OPTION_MISSING_AI_CHANCE): Information when an option has no
///   `ai_chance` block.
struct EventVisitor {
    /// Depth of event definition nesting (>0 means inside an event definition).
    event_depth: u32,
    /// Stack of events being tracked (supports nested effects).
    event_stack: Vec<EventDef>,
    /// Stack of option definitions currently being walked.
    option_stack: Vec<EventOptionDef>,
}

impl EventVisitor {
    fn new() -> Self {
        Self {
            event_depth: 0,
            event_stack: Vec::new(),
            option_stack: Vec::new(),
        }
    }

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

    fn validate_event(
        &self,
        state: &EventDef,
        ctx: &ValidationContext,
        diags: &mut Vec<Diagnostic>,
    ) {
        // Only validate blocks that are actual event definitions (contain at least one option).
        // `country_event = { ... }` used as an effect inside options doesn't have options
        // and would produce false-positive "missing title" diagnostics.
        if !state.has_option {
            return;
        }

        // HOM3016: non-hidden event without title AND desc
        if !state.is_hidden && !state.has_title && !state.has_desc {
            diags.push(Diagnostic {
                range: ast_range_to_lsp(&state.key_range),
                severity: Some(DiagnosticSeverity::WARNING),
                message: "Event is missing both 'title' and 'desc'. A non-hidden event \
                          requires at least one of them to display anything to the player."
                    .to_string(),
                code: Some(NumberOrString::String(
                    crate::validation::advanced_validation::EVENT_MISSING_TITLE.to_string(),
                )),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        }

        // HOM3018: title loc key missing from localization
        if let Some(ref key) = state.title_key {
            if !ctx.loc.contains_key(key.as_str()) {
                let prefix = format!("{}:", key);
                if !ctx.loc.iter().any(|e| e.key().starts_with(&prefix)) {
                    diags.push(Diagnostic {
                        range: ast_range_to_lsp(&state.key_range),
                        severity: Some(DiagnosticSeverity::WARNING),
                        message: format!(
                            "Event title localization key '{}' not found in any localization file.",
                            key,
                        ),
                        code: Some(NumberOrString::String(
                            crate::validation::advanced_validation::EVENT_MISSING_TITLE_LOC
                                .to_string(),
                        )),
                        source: Some("Hearts of Modding".to_string()),
                        ..Default::default()
                    });
                }
            }
        }

        // HOM3019: desc loc key missing from localization
        if let Some(ref key) = state.desc_key {
            if !ctx.loc.contains_key(key.as_str()) {
                let prefix = format!("{}:", key);
                if !ctx.loc.iter().any(|e| e.key().starts_with(&prefix)) {
                    diags.push(Diagnostic {
                        range: ast_range_to_lsp(&state.key_range),
                        severity: Some(DiagnosticSeverity::WARNING),
                        message: format!(
                            "Event description localization key '{}' not found in any localization file.",
                            key,
                        ),
                        code: Some(NumberOrString::String(
                            crate::validation::advanced_validation::EVENT_MISSING_DESC_LOC
                                .to_string(),
                        )),
                        source: Some("Hearts of Modding".to_string()),
                        ..Default::default()
                    });
                }
            }
        }

        // HOM3020: picture sprite not found
        if let Some(ref sprite) = state.picture_sprite {
            if sprite.starts_with("GFX_") && !ctx.sprites.contains_key(sprite.as_str()) {
                diags.push(Diagnostic {
                    range: ast_range_to_lsp(&state.key_range),
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: format!(
                        "Event picture sprite '{}' not found. Define it in an interface/*.gfx file.",
                        sprite,
                    ),
                    code: Some(NumberOrString::String(
                        crate::validation::advanced_validation::EVENT_PICTURE_SPRITE_NOT_FOUND
                            .to_string(),
                    )),
                    source: Some("Hearts of Modding".to_string()),
                    ..Default::default()
                });
            }
        }
    }
}

impl AstVisitor for EventVisitor {
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
            self.event_stack.push(EventDef {
                key_range: ass.key_range.clone(),
                has_title: false,
                has_desc: false,
                has_picture: false,
                is_hidden: false,
                has_mtth: false,
                has_is_triggered_only: false,
                has_option: false,
                title_key: None,
                desc_key: None,
                picture_sprite: None,
            });
            self.event_depth += 1;
            return;
        }

        if self.event_depth == 0 {
            return;
        }

        // ── Track event-level properties (not inside options) ──────
        if !self.in_option() {
            if let Some(state) = self.event_stack.last_mut() {
                match key.to_ascii_lowercase().as_str() {
                    "title" => {
                        state.has_title = true;
                        if let Some(s) = ass.value.value.as_str(ctx.source) {
                            // Skip block-form title (multiple conditional texts).
                            // Simple identifiers and quoted strings are both valid loc keys.
                            if !matches!(
                                &ass.value.value,
                                ast::Value::Block(_) | ast::Value::TaggedBlock(..)
                            ) {
                                state.title_key = Some(s.to_string());
                            }
                        }
                    }
                    "desc" => {
                        state.has_desc = true;
                        if let Some(s) = ass.value.value.as_str(ctx.source) {
                            if !matches!(
                                &ass.value.value,
                                ast::Value::Block(_) | ast::Value::TaggedBlock(..)
                            ) {
                                state.desc_key = Some(s.to_string());
                            }
                        }
                    }
                    "picture" => {
                        state.has_picture = true;
                        if let Some(s) = ass.value.value.as_str(ctx.source) {
                            // Only check GFX_ prefixed sprites — quoted strings
                            // may be scripted localisation.
                            if s.starts_with("GFX_") {
                                state.picture_sprite = Some(s.to_string());
                            }
                        }
                    }
                    "hidden" => {
                        state.is_hidden = ass.value.value.as_str(ctx.source) == Some("yes");
                    }
                    _ => {}
                }
                // Track MTTH and is_triggered_only (non-block form)
                if key == "is_triggered_only" {
                    state.has_is_triggered_only = ass.value.value.as_str(ctx.source) == Some("yes");
                } else if key == "mean_time_to_happen" {
                    if matches!(&ass.value.value, ast::Value::Block(_)) {
                        state.has_mtth = true;
                    }
                }
            }
            // Don't return — fall through to option detection below.
        }

        // ── Detect option definition entry (only at event level) ────
        if !self.in_option()
            && key.eq_ignore_ascii_case("option")
            && matches!(&ass.value.value, ast::Value::Block(_))
        {
            // Mark the current event as having options (signals it's a definition,
            // not an effect used inside another event).
            if let Some(state) = self.event_stack.last_mut() {
                state.has_option = true;
            }
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
                // Validate the outer-most event when its depth returns to 0.
                // Nested event-effect blocks (e.g. country_event = { ... }
                // inside an option) are also events, but we only validate
                // at the top level to avoid double-reporting.
            }
            // Always validate when popping, regardless of nesting
            if let Some(state) = self.event_stack.pop() {
                self.validate_event(&state, ctx, diags);
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
                severity: Some(DiagnosticSeverity::ERROR),
                message: format!(
                    "Event ID '{}' uses namespace '{}' which has not been declared. \
                     The event will not be registered by the game (log error: \
                     'Malformed token: {}'). \
                     Add 'add_namespace = {}' before any events using this namespace.",
                    id, namespace_str, id, namespace_str
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
        Box::new(EventVisitor::new())
    }
}
