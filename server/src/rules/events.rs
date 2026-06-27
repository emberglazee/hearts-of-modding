use crate::parser::ast;
use crate::rules::visitor::AstVisitor;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scanner::event_namespace_scanner;
use crate::scope::scope::ScopeStack;
use crate::utils::lsp_convert::ast_range_to_lsp;
use std::collections::HashSet;
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

/// AstVisitor that validates event structure, option blocks, and namespace IDs.
///
/// The visitor tracks `add_namespace` declarations as they appear, enabling
/// same-file positional ordering checks. Cross-file ordering is verified by
/// comparing filenames against the ASCII sort order HOI4 uses to load files.
struct EventVisitor {
    /// Depth of event definition nesting (>0 means inside an event definition).
    event_depth: u32,
    /// Stack of events being tracked (supports nested effects).
    event_stack: Vec<EventDef>,
    /// Stack of option definitions currently being walked.
    option_stack: Vec<EventOptionDef>,
    /// Namespace declarations seen so far in the current file walk (lowercased).
    /// Populated by `add_namespace = X` entries in document order.
    seen_namespaces: HashSet<String>,
}

impl EventVisitor {
    fn new() -> Self {
        Self {
            event_depth: 0,
            event_stack: Vec::new(),
            option_stack: Vec::new(),
            seen_namespaces: HashSet::new(),
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

    /// Check an event ID for namespace validity with same-file and cross-file ordering.
    fn check_event_id(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        id: &str,
        diags: &mut Vec<Diagnostic>,
    ) {
        let Some(parsed) = event_namespace_scanner::parse_event_id(id) else {
            return;
        };

        // HOM3009: non-integer event ID suffix
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
            return;
        }

        // HOM3010: event ID too large (>= 100000)
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

        // HOM3008: namespace availability check with ordering awareness
        let namespace_str = parsed.namespace;
        // Allow event IDs without a namespace part (e.g. just "12345" — legacy IDs)
        if namespace_str.is_empty() || namespace_str.chars().all(|c| c.is_ascii_digit()) {
            return;
        }

        let ns_lower = namespace_str.to_ascii_lowercase();

        // Case 1: Same-file, declared before this event → OK
        if self.seen_namespaces.contains(&ns_lower) {
            return;
        }

        // Resolve current file path from URI for cross-file ordering
        let current_path: Option<std::path::PathBuf> = match Uri::from_str(ctx.uri) {
            Ok(uri) => uri.to_file_path().map(|p| p.into_owned()),
            Err(_) => None,
        };
        let current_filename = current_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_lowercase());

        // Look up the namespace in the global map (try exact match, then lowercase)
        let global_entry = ctx.event_namespaces.get(namespace_str);
        let global_entry = if global_entry.is_some() {
            global_entry
        } else {
            ctx.event_namespaces.get(ns_lower.as_str())
        };

        match global_entry {
            Some(entry) => {
                // Namespace exists somewhere — check ordering
                let declaring_path = &*entry.value().resolve().path;
                let decl_path = std::path::Path::new(declaring_path);
                let decl_filename = decl_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_lowercase());

                // Vanilla/DLC files always load BEFORE mod files, regardless of
                // individual filenames. If the namespace is declared in a game-path
                // file and the current file is from the workspace (mod), it's available.
                let is_declaring_under_game = ctx
                    .game_path
                    .as_ref()
                    .is_some_and(|gp| decl_path.starts_with(gp));
                let is_current_under_game = current_path
                    .as_ref()
                    .is_some_and(|cp| ctx.game_path.as_ref().is_some_and(|gp| cp.starts_with(gp)));
                if is_declaring_under_game && !is_current_under_game {
                    // Vanilla/DLC files always load BEFORE mod files — namespace available
                } else if !is_declaring_under_game && is_current_under_game {
                    // Mod files load AFTER vanilla — a mod namespace is NOT available
                    // to a vanilla file. Fall through to the filename-ordering check,
                    // but we know the result will always be "not available". Since the
                    // filename comparison might incorrectly think otherwise (e.g.,
                    // aaa_mod.txt sorts before zzz_vanilla.txt), emit directly.
                    let decl_file_label = decl_filename.as_deref().unwrap_or("other file");
                    diags.push(Diagnostic {
                        range: ast_range_to_lsp(&ass.value.range),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: format!(
                            "Event ID '{}' uses namespace '{}' declared in mod file '{}', \
                             but this file is from the base game. Vanilla/DLC files load \
                             BEFORE mod files, so this namespace is not available here. \
                             Use a namespace already declared in the base game instead.",
                            id, namespace_str, decl_file_label
                        ),
                        code: Some(NumberOrString::String(
                            crate::validation::advanced_validation::MISSING_EVENT_NAMESPACE
                                .to_string(),
                        )),
                        source: Some("Hearts of Modding".to_string()),
                        ..Default::default()
                    });
                } else {
                    match (&current_filename, decl_filename) {
                        (Some(cur), Some(decl)) if decl.as_str() == cur.as_str() => {
                            // Same file → declared LATER → reorder needed
                            diags.push(Diagnostic {
                                range: ast_range_to_lsp(&ass.value.range),
                                severity: Some(DiagnosticSeverity::ERROR),
                                message: format!(
                                    "Event ID '{}' uses namespace '{}' which is declared LATER \
                                     in this file. Move 'add_namespace = {}' BEFORE this event \
                                     definition. The game registers namespaces sequentially as \
                                     it reads the file.",
                                    id, namespace_str, namespace_str
                                ),
                                code: Some(NumberOrString::String(
                                    crate::validation::advanced_validation::MISSING_EVENT_NAMESPACE
                                        .to_string(),
                                )),
                                source: Some("Hearts of Modding".to_string()),
                                ..Default::default()
                            });
                        }
                        (Some(cur), Some(decl)) if decl.as_str() > cur.as_str() => {
                            // Other file loads AFTER → namespace unavailable at this point
                            diags.push(Diagnostic {
                                range: ast_range_to_lsp(&ass.value.range),
                                severity: Some(DiagnosticSeverity::ERROR),
                                message: format!(
                                    "Event ID '{}' uses namespace '{}' which is declared in '{}'. \
                                     That file loads AFTER this one (ASCII filename order), so the \
                                     namespace is not yet registered. Either move the 'add_namespace' \
                                     declaration to a file that loads before this one, or add a \
                                     declaration here before the event.",
                                    id, namespace_str, decl
                                ),
                                code: Some(NumberOrString::String(
                                    crate::validation::advanced_validation::MISSING_EVENT_NAMESPACE
                                        .to_string(),
                                )),
                                source: Some("Hearts of Modding".to_string()),
                                ..Default::default()
                            });
                        }
                        _ => {
                            // Same-file/cross-file available, or can't determine ordering
                            // (decl < cur means declaring file loads first → available, no diagnostic)
                            // If ordering is indeterminate, be conservative: don't flag
                        }
                    }
                }
            }
            None => {
                // Namespace not declared anywhere → genuinely missing
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

    /// Extract `id = ...` from an event block and run namespace checks.
    fn check_event_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        diags: &mut Vec<Diagnostic>,
    ) {
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

        if let Some(id) = id_str {
            self.check_event_id(ass, ctx, id, diags);
        }
    }
}

impl AstVisitor for EventVisitor {
    fn enter_assignment(
        &mut self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key = ass.key_text(ctx.source);

        // ── Track add_namespace declarations (document order) ─────
        if key == "add_namespace" {
            if let Some(name) = ass.value.value.as_str(ctx.source) {
                self.seen_namespaces.insert(name.to_ascii_lowercase());
            }
            // Don't return — HOM3012 (duplicate namespace) still fires via check_assignment
        }

        // ── Detect event definition entry ──────────────────────────
        if Self::is_event_type(key) && matches!(&ass.value.value, ast::Value::Block(_)) {
            // Check namespace ID BEFORE pushing to stack (ordering check uses seen_namespaces)
            self.check_event_assignment(ass, ctx, diags);

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
            // Always validate when popping
            if let Some(state) = self.event_stack.pop() {
                self.validate_event(&state, ctx, diags);
            }
        }
    }
}

/// Validates event definitions for correct structure.
///
/// Checks that remain at the block level (not per-assignment):
/// - Duplicate `add_namespace` declarations → HOM3012
///
/// Per-assignment checks (HOM3008, HOM3009, HOM3010) are handled by
/// `EventVisitor` which has access to walking-order state for
/// same-file namespace ordering validation.
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
                                let path = path.into_owned();
                                let current = path.canonicalize().ok();
                                let decl_path = std::path::Path::new(other_path);
                                let stored = decl_path.canonicalize().ok();
                                match (current, stored) {
                                    (Some(c), Some(s)) => c == s,
                                    (None, None) => path == decl_path,
                                    _ => false,
                                }
                            }
                            None => false,
                        },
                        Err(_) => false,
                    };
                    if !same_file {
                        diags.push(Diagnostic {
                            range: ast_range_to_lsp(&ass.value.range),
                            severity: Some(DiagnosticSeverity::INFORMATION),
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
        }
    }
}

impl EventValidationRule {
    pub(crate) fn visitor() -> Box<dyn AstVisitor> {
        Box::new(EventVisitor::new())
    }
}
