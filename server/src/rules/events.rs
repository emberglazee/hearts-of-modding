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
    /// Total option blocks in this event.
    option_count: u32,
    /// Number of options missing an `ai_chance` block.
    options_missing_ai_chance: u32,
    /// Key range of the last option missing `ai_chance` (for diagnostic positioning).
    last_missing_ai_chance_range: Option<ast::Range>,
    /// Range of the `title` assignment key (for HOM3018 positioning).
    title_range: Option<ast::Range>,
    /// Range of the `desc` assignment key (for HOM3019 positioning).
    desc_range: Option<ast::Range>,
    /// Range of the `picture` assignment key (for HOM3020 positioning).
    picture_range: Option<ast::Range>,
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
                    let d = state.title_range.as_ref().unwrap_or(&state.key_range);
                    diags.push(Diagnostic {
                        range: ast_range_to_lsp(d),
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
                    let d = state.desc_range.as_ref().unwrap_or(&state.key_range);
                    diags.push(Diagnostic {
                        range: ast_range_to_lsp(d),
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
                let d = state.picture_range.as_ref().unwrap_or(&state.key_range);
                diags.push(Diagnostic {
                    range: ast_range_to_lsp(d),
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

        // HOM3017: ai_chance check — only meaningful when there are multiple options
        if state.option_count > 1 && state.options_missing_ai_chance > 0 {
            let diag_range = state
                .last_missing_ai_chance_range
                .as_ref()
                .map(ast_range_to_lsp)
                .unwrap_or_else(|| ast_range_to_lsp(&state.key_range));
            diags.push(Diagnostic {
                range: diag_range,
                severity: Some(DiagnosticSeverity::INFORMATION),
                message: format!(
                    "{} of {} option(s) are missing an 'ai_chance' block. \
                     The AI may not choose optimally without explicit weights.",
                    state.options_missing_ai_chance, state.option_count,
                ),
                code: Some(NumberOrString::String(
                    crate::validation::advanced_validation::EVENT_OPTION_MISSING_AI_CHANCE
                        .to_string(),
                )),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
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

        // Files outside `events/` use `country_event = { ... }` as an *effect*,
        // not a definition. The namespace just needs to exist somewhere in any
        // events file — ordering doesn't apply because events are fully loaded
        // before decisions/focuses/etc. are executed.
        let is_in_events_dir = ctx.uri.contains("/events/");
        if !is_in_events_dir {
            // For non-events files, check if the namespace exists anywhere
            let available = ctx.event_namespaces.get(namespace_str).is_some()
                || ctx.event_namespaces.get(ns_lower.as_str()).is_some();
            if !available {
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
                        crate::validation::advanced_validation::MISSING_EVENT_NAMESPACE
                            .to_string(),
                    )),
                    source: Some("Hearts of Modding".to_string()),
                    ..Default::default()
                });
            }
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
                //
                // Normalize paths for cross-platform comparison: lowercased, forward slashes,
                // stripped leading / (since URI paths may include / on Unix while stored
                // paths with drive letters like C:/... start without one).
                let norm = |p: &std::path::Path| -> String {
                    p.to_string_lossy()
                        .to_lowercase()
                        .replace('\\', "/")
                        .trim_start_matches('/')
                        .to_string()
                };
                let gp_norm = ctx.game_path.as_ref().map(|gp| {
                    gp.to_lowercase()
                        .replace('\\', "/")
                        .trim_start_matches('/')
                        .to_string()
                });
                let decl_norm = norm(decl_path);
                let is_declaring_under_game = gp_norm
                    .as_ref()
                    .is_some_and(|gp| decl_norm.starts_with(gp.as_str()));
                let is_current_under_game = current_path.as_ref().is_some_and(|cp| {
                    gp_norm
                        .as_ref()
                        .is_some_and(|gp| norm(cp).starts_with(gp.as_str()))
                });
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
                option_count: 0,
                options_missing_ai_chance: 0,
                last_missing_ai_chance_range: None,
                title_range: None,
                desc_range: None,
                picture_range: None,
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
                        state.title_range = Some(ass.key_range.clone());
                        // Only unquoted identifiers are loc key references.
                        // Quoted strings like title = "Literal Text" are inline
                        // text displayed directly by the game — not loc keys.
                        if let ast::Value::String(span) = &ass.value.value {
                            state.title_key = Some(span.resolve(ctx.source).to_string());
                        }
                    }
                    "desc" => {
                        state.has_desc = true;
                        state.desc_range = Some(ass.key_range.clone());
                        if let ast::Value::String(span) = &ass.value.value {
                            state.desc_key = Some(span.resolve(ctx.source).to_string());
                        }
                    }
                    "picture" => {
                        state.has_picture = true;
                        state.picture_range = Some(ass.key_range.clone());
                        if let Some(s) = ass.value.value.as_str(ctx.source) {
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
                state.option_count += 1;
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
                // Track missing ai_chance on the event for summary reporting
                if !state.has_ai_chance {
                    if let Some(event) = self.event_stack.last_mut() {
                        event.options_missing_ai_chance += 1;
                        event.last_missing_ai_chance_range = Some(state.key_range.clone());
                    }
                }
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
                                    (None, None) => {
                                        // Both files are virtual (e.g., in tests or not on
                                        // disk). Normalize separators, case, and leading
                                        // slashes for a cross-platform string comparison.
                                        let p = path
                                            .to_string_lossy()
                                            .to_lowercase()
                                            .replace('\\', "/")
                                            .trim_start_matches('/')
                                            .to_string();
                                        let d = decl_path
                                            .to_string_lossy()
                                            .to_lowercase()
                                            .replace('\\', "/")
                                            .trim_start_matches('/')
                                            .to_string();
                                        p == d
                                    }
                                    _ => false,
                                }
                            }
                            None => {
                                // URI can't be resolved (e.g., no drive letter on
                                // Windows). Extract filename from URI string directly
                                // since Path::new may not handle "file:///..." properly.
                                let uri_fn = ctx
                                    .uri
                                    .rsplit('/')
                                    .next()
                                    .and_then(|s| if s.is_empty() { None } else { Some(s) });
                                let stored_fn = std::path::Path::new(other_path)
                                    .file_name()
                                    .and_then(|n| n.to_str());
                                uri_fn
                                    .zip(stored_fn)
                                    .is_some_and(|(a, b)| a.eq_ignore_ascii_case(b))
                            }
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
