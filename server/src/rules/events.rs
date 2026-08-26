use crate::parser::ast;
use crate::rules::visitor::AstVisitor;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scanner::event_namespace_scanner;
use crate::scope::scope::ScopeStack;
use std::collections::{HashMap, HashSet};
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
    /// Whether the option's `trigger` PROVABLY hides it from the AI:
    /// - direct `is_ai = no`
    /// - `NOT = { is_ai = yes }`
    /// - a scripted trigger that statically guarantees AI-invisibility
    ///   (e.g. Hearts of Minecraft's `dbug_mode`)
    ///
    /// Wiki (event-modding.md): an option whose trigger is false when the
    /// event fires "will not appear" — the AI cannot pick it, so it holds no
    /// share of the ai_chance weight distribution.
    provably_ai_invisible: bool,
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
    /// Whether this block is an event call (nested inside another event),
    /// not a top-level definition. Calls skip namespace ordering and
    /// structural validation.
    is_call: bool,
    /// Whether this block contains at least one `option = { ... }`.
    /// Event definitions always have options; `country_event = { ... }`
    /// used as an effect never does. Only validate definition usage.
    has_option: bool,
    /// Total option blocks in this event.
    option_count: u32,
    /// Number of options missing an `ai_chance` block.
    options_missing_ai_chance: u32,
    /// Number of options that are VISIBLE to the AI (not provably hidden by
    /// their `trigger`). The AI's choice is made proportionally over only
    /// these; if exactly one is visible, its pick is forced and no
    /// `ai_chance` weights matter.
    ai_visible_option_count: u32,
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
    /// Event id (`id = <name>`) — used for duplicate-event-ID detection.
    id: Option<String>,
    /// Range of the `id` value (for pinpointing a duplicate).
    id_range: Option<ast::Range>,
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
    /// Event IDs already defined in the current file (lowercased → first range).
    /// Used to flag duplicate event IDs (HOM3011), which HOI4 silently drops.
    seen_event_ids: HashMap<String, ast::Range>,
}

impl EventVisitor {
    fn new() -> Self {
        Self {
            event_depth: 0,
            event_stack: Vec::new(),
            option_stack: Vec::new(),
            seen_namespaces: HashSet::new(),
            seen_event_ids: HashMap::new(),
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
                range: _ctx.range(&state.key_range),
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
        // Only validate blocks that are actual event definitions.
        // Calls (nested event-type blocks) have is_call=true and are
        // never structural event definitions.
        if state.is_call {
            return;
        }

        // HOM3016: non-hidden event without title AND desc
        if !state.is_hidden && !state.has_title && !state.has_desc {
            diags.push(Diagnostic {
                range: ctx.range(&state.key_range),
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
        // (A `{key}:`-prefixed suppression scan used to live here; stored loc
        // keys are the text before the first `:` — e.g. `foo` from `foo:0 "..."`
        // — so they can never contain `:` and the scan never suppressed
        // anything, just cost O(N) per missing key.)
        if !ctx.loc.is_empty() {
            if let Some(ref key) = state.title_key {
                if !ctx.loc.contains_key(key.as_str()) {
                    let d = state.title_range.as_ref().unwrap_or(&state.key_range);
                    diags.push(Diagnostic {
                        range: ctx.range(d),
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

            // HOM3019: desc loc key missing from localization
            if let Some(ref key) = state.desc_key {
                if !ctx.loc.contains_key(key.as_str()) {
                    let d = state.desc_range.as_ref().unwrap_or(&state.key_range);
                    diags.push(Diagnostic {
                        range: ctx.range(d),
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
        } // end loc_ready guard

        // HOM3020: picture sprite not found
        if let Some(ref sprite) = state.picture_sprite {
            if sprite.starts_with("GFX_") && !ctx.sprites.contains_key(sprite.as_str()) {
                let d = state.picture_range.as_ref().unwrap_or(&state.key_range);
                diags.push(Diagnostic {
                    range: ctx.range(d),
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

        // HOM3017: ai_chance check — only fires when the AI's choice is
        // actually a weighted decision among MULTIPLE visible options.
        //
        // Wiki ground truth (event-modding.md):
        // - an option whose `trigger` is false when the event fires "will not
        //   appear" — provably AI-invisible options hold no share of the
        //   weight distribution;
        // - a missing `ai_chance` defaults to weight 1, and weights are
        //   proportional ("The probability of each option is its weight
        //   divided by the sum of all option weights").
        //
        // Consequence: when only ONE option is visible to the AI, its pick is
        // FORCED (weight 1 of total 1) regardless of any ai_chance blocks —
        // identical to ai_chance = { factor = 100 }. The common debug-option
        // pattern (`trigger = { dbug_mode = yes }`, dbug_mode requiring
        // is_ai = no) leaves exactly one AI-visible option, so HOM3017 is
        // suppressed for the whole event even though that visible option has
        // no explicit weights.
        let forced_choice = state.ai_visible_option_count <= 1;
        if state.option_count > 1 && !forced_choice && state.options_missing_ai_chance > 0 {
            let diag_range = state
                .last_missing_ai_chance_range
                .as_ref()
                .map(|r| ctx.range(r))
                .unwrap_or_else(|| ctx.range(&state.key_range));
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
                range: ctx.range(&ass.value.range),
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
                    range: ctx.range(&ass.value.range),
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
                    range: ctx.range(&ass.value.range),
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
                        range: ctx.range(&ass.value.range),
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
                                range: ctx.range(&ass.value.range),
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
                                range: ctx.range(&ass.value.range),
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
                    range: ctx.range(&ass.value.range),
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

        // ── Inside an option: event type assignments are CALLS, not definitions ──
        if self.in_option() && Self::is_event_type(key) {
            // Extract target event ID and check if it exists in the workspace.
            let target_id = match &ass.value.value {
                ast::Value::String(span) => Some(span.resolve(ctx.source)),
                ast::Value::QuotedString(s) => Some(s.as_str()),
                ast::Value::Block(entries) => entries.iter().find_map(|e| {
                    if let ast::Entry::Assignment(inner_ass) = e
                        && inner_ass.key_text(ctx.source) == "id"
                    {
                        inner_ass.value.value.as_str(ctx.source)
                    } else {
                        None
                    }
                }),
                _ => None,
            };
            if let Some(id) = target_id
                && !ctx.events.contains_key(id)
            {
                diags.push(Diagnostic {
                    range: ctx.range(&ass.key_range),
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: format!(
                        "Event reference '{}' does not match any defined event in the workspace.",
                        id,
                    ),
                    code: Some(NumberOrString::String(
                        crate::validation::advanced_validation::BROKEN_EVENT_REFERENCE.to_string(),
                    )),
                    source: Some("Hearts of Modding".to_string()),
                    ..Default::default()
                });
            }
            // Fall through — still push to event_stack to maintain
            // push/pop symmetry with exit_assignment. The EventDef is
            // marked as is_call=true so namespace ordering and structural
            // validation are skipped.
        }

        // ── Detect event definition entry ──────────────────────────
        // Push ALL event-type blocks (both defs and calls) to event_stack
        // to keep push/pop symmetry with exit_assignment. Calls are marked
        // with is_call=true so namespace ordering and validation are skipped.
        if Self::is_event_type(key) && matches!(&ass.value.value, ast::Value::Block(_)) {
            // Only check namespace ordering for TOP-LEVEL event definitions.
            // Nested blocks are CALLS — they fire at runtime when all files
            // are already loaded.
            if self.event_depth == 0 {
                self.check_event_assignment(ass, ctx, diags);
            }

            self.event_stack.push(EventDef {
                key_range: ass.key_range.clone(),
                has_title: false,
                has_desc: false,
                has_picture: false,
                is_hidden: false,
                has_mtth: false,
                has_is_triggered_only: false,
                is_call: self.event_depth > 0 || !ctx.uri.contains("/events/"),
                has_option: false,
                option_count: 0,
                options_missing_ai_chance: 0,
                ai_visible_option_count: 0,
                last_missing_ai_chance_range: None,
                title_range: None,
                desc_range: None,
                picture_range: None,
                title_key: None,
                desc_key: None,
                picture_sprite: None,
                id: None,
                id_range: None,
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
                        state.is_hidden = match &ass.value.value {
                            ast::Value::Boolean(b) => *b,
                            _ => ass.value.value.as_str(ctx.source) == Some("yes"),
                        };
                    }
                    "id" => {
                        if let Some(s) = ass.value.value.as_str(ctx.source) {
                            state.id = Some(s.to_string());
                            state.id_range = Some(ass.value.range.clone());
                        }
                    }
                    _ => {}
                }
                // Track MTTH and is_triggered_only (non-block form)
                if key == "is_triggered_only" {
                    state.has_is_triggered_only = match &ass.value.value {
                        ast::Value::Boolean(b) => *b,
                        _ => ass.value.value.as_str(ctx.source) == Some("yes"),
                    };
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
                provably_ai_invisible: Self::evaluate_option_trigger_ai_visibility(&ass.value, ctx),
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
                // Track missing ai_chance on the event for summary reporting.
                // Options whose trigger provably hides them from the AI hold
                // no share of the weight distribution (wiki: invisible options
                // "will not appear"; default ai_chance weight is 1, so the
                // remaining visible option gets 100%) — they never need an
                // ai_chance block and don't count toward HOM3017.
                if let Some(event) = self.event_stack.last_mut() {
                    if !state.provably_ai_invisible {
                        event.ai_visible_option_count += 1;
                    }
                    if !state.has_ai_chance && !state.provably_ai_invisible {
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

                // HOM3011: duplicate event ID within this file. HOI4 loads only
                // one event per ID, so the second identical definition silently
                // never fires — a definite conflict (not a candidate).
                if !state.is_call {
                    if let Some(id) = &state.id {
                        let key = id.to_ascii_lowercase();
                        if let Some(_first) =
                            self.seen_event_ids.insert(key, state.key_range.clone())
                        {
                            let range = state
                                .id_range
                                .clone()
                                .unwrap_or_else(|| state.key_range.clone());
                            diags.push(Diagnostic {
                                range: ctx.range(&range),
                                severity: Some(DiagnosticSeverity::ERROR),
                                message: format!(
                                    "Duplicate event ID '{}' defined more than once in this \
                                     file. HOI4 loads only one event per ID; the other \
                                     definition will never fire. Rename one of them.",
                                    id,
                                ),
                                code: Some(NumberOrString::String(
                                    crate::validation::advanced_validation::DUPLICATE_EVENT_ID
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
                                let uri_fn = ctx.uri.rsplit('/').next().filter(|&s| !s.is_empty());
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
                            range: ctx.range(&ass.value.range),
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

impl EventVisitor {
    /// Evaluate an option block's `trigger` (if any) for provable AI
    /// invisibility. No trigger at all → visible to the AI (returns false).
    fn evaluate_option_trigger_ai_visibility(
        value: &ast::NodeedValue,
        ctx: &ValidationContext,
    ) -> bool {
        let ast::Value::Block(entries) = &value.value else {
            return false;
        };
        // Find the option's trigger block, if present.
        for entry in entries {
            if let ast::Entry::Assignment(ass) = entry {
                if ass.key_text(ctx.source).eq_ignore_ascii_case("trigger") {
                    if let ast::Value::Block(trigger_body) = &ass.value.value {
                        return Self::option_provably_ai_invisible(trigger_body, ctx.source, ctx);
                    }
                    return false;
                }
            }
        }
        false
    }

    /// Statically decide whether an option `trigger` block PROVES the option
    /// is invisible to the AI. Conservative: only returns true on certainty.
    ///
    /// Recognized proofs:
    /// - `is_ai = no` anywhere in a conjunctive position (top level, inside
    ///   `AND`, inside `limit`)
    /// - `NOT = { is_ai = yes }`
    /// - a scripted trigger reference whose body statically proves the same
    ///   (resolved against scanned scripted_triggers)
    ///
    /// NOT proven (returns false → diagnostic may still fire):
    /// - `is_ai = yes` or negations of other triggers
    /// - `OR` blocks — one arm proving AI-invisibility does not make the OR
    ///   false for the AI
    /// - scripted triggers that don't statically prove invisibility
    fn option_provably_ai_invisible(
        entries: &[ast::Entry],
        source: &str,
        ctx: &ValidationContext,
    ) -> bool {
        fn has_proof(entries: &[ast::Entry], source: &str, ctx: &ValidationContext) -> bool {
            for entry in entries {
                if let ast::Entry::Assignment(ass) = entry {
                    let key = ass.key_text(source).to_ascii_lowercase();
                    match key.as_str() {
                        "is_ai" => {
                            // `yes`/`no` parse as Boolean; accept both forms.
                            let val: Option<String> = match &ass.value.value {
                                ast::Value::Boolean(b) => {
                                    Some(if *b { "yes" } else { "no" }.to_string())
                                }
                                _ => ass
                                    .value
                                    .value
                                    .as_str(source)
                                    .map(|s| s.to_ascii_lowercase()),
                            };
                            if val.as_deref() == Some("no") {
                                return true;
                            }
                        }
                        // Conjunction: if ANY arm proves it, the AND does.
                        "and" | "limit" if matches!(&ass.value.value, ast::Value::Block(_)) => {
                            if let ast::Value::Block(arms) = &ass.value.value {
                                if has_proof(arms, source, ctx) {
                                    return true;
                                }
                            }
                        }
                        // Negation: NOT = { is_ai = yes } proves it.
                        // (A NOT arm inside an AND is still conjunctive.)
                        "not" if matches!(&ass.value.value, ast::Value::Block(_)) => {
                            if let ast::Value::Block(arms) = &ass.value.value {
                                for arm in arms {
                                    if let ast::Entry::Assignment(a) = arm {
                                        let arm_yes = match &a.value.value {
                                            ast::Value::Boolean(b) => *b,
                                            _ => a
                                                .value
                                                .value
                                                .as_str(source)
                                                .map(|s| s.eq_ignore_ascii_case("yes"))
                                                .unwrap_or(false),
                                        };
                                        if a.key_text(source).eq_ignore_ascii_case("is_ai")
                                            && arm_yes
                                        {
                                            return true;
                                        }
                                    }
                                }
                            }
                        }
                        // Scripted trigger reference: use the flag the
                        // scanner precomputed from the body at scan time.
                        _ => {
                            let name = ass.key_text(ctx.source);
                            let lower = name.to_ascii_lowercase();
                            let proves = ctx
                                .scripted_triggers
                                .get(lower.as_str())
                                .or_else(|| ctx.scripted_triggers.get(name))
                                .map(|e| e.resolve().guarantees_ai_invisible)
                                .unwrap_or(false);
                            if proves {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        }

        has_proof(entries, source, ctx)
    }
}

impl EventValidationRule {
    pub(crate) fn visitor() -> Box<dyn AstVisitor> {
        Box::new(EventVisitor::new())
    }
}
