use crate::data::interner::InternedStr;
use crate::data::layered_value::LayeredValue;
use crate::scanner::achievement_scanner::Achievement;
use crate::scanner::character_scanner::Character;
use crate::scanner::variable_scanner::EventTarget;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scope {
    Global,
    Country,
    State,
    Unit,
    Character,
    MusicStation,
    MusicTrack,
    Achievement,
    Ribbon,
    Idea,
    /// The `hidden_ideas` category inside `ideas = { }`. Works like
    /// `country` as a category keyword, except ideas defined within it
    /// don't show up in the spirit container and don't need a `picture`.
    HiddenIdeaCategory,
    /// A focus tree or continuous focus palette container
    FocusTree,
    /// A national focus, shared focus, or joint focus definition
    NationalFocus,
    /// Strategic region (air zone / naval region)
    StrategicRegion,
    /// Ace pilot modifier definitions (common/aces/*.txt)
    /// Represents the file-level scope for ace modifier definitions.
    Ace,
    /// File-level scope for `common/technology_tags/*.txt`. Top level holds
    /// only the `technology_categories` / `technology_folders` declaration
    /// blocks — country triggers/effects don't belong here, so this scope
    /// (unlike Global) filters them out of completion.
    TechnologyTags,
    /// File-level scope for `common/technologies/*.txt` (technologies AND
    /// doctrines). Children of `technologies = { ... }` are tech definitions;
    /// regular trigger/effect bodies only appear inside nested evaluation
    /// blocks like `ai_will_do`.
    Technologies,
    /// `technology_categories = { ... }` container (common/technology_tags/*.txt).
    /// Contents are bare category identifiers — no trigger/effect evaluation
    /// happens inside.
    TechnologyCategories,
    /// `technology_folders = { ... }` container (common/technology_tags/*.txt).
    /// Children are dynamically-named folder blocks (`available`/`ledger`/
    /// `doctrine`); the folder's `available` block is evaluated per-country,
    /// so the container is effectively country-scoped.
    TechnologyFolders,
    /// A modifier-application target block (e.g. `unit_modifiers = { }`).
    /// The engine reads these as a flat bag of modifiers and routes them
    /// per-key — not a trigger/effect evaluation scope. V2ScopeRule skips
    /// scope checks inside these blocks.
    ModifierBag,
    /// Structural container for `common/on_actions/*.txt` (`on_actions = { }`). Its children are individual `on_*` blocks, each with its own runtime
    /// scope (Country / State / Character / Unit / Global).
    OnActions,
    /// File-level scope for `common/scripted_effects/*.txt`. Top level holds
    /// `my_effect = { ... }` definitions whose bodies are evaluated in the
    /// caller's scope. This scope skips HOM004 validation at the top level
    /// (like `ModifierBag`) because scripted effects are polymorphic — the
    /// engine runs them in whatever scope they're called from (Country/State/
    /// Character). Explicit iterators (`every_state`, `every_country`, etc.)
    /// still push their respective scopes correctly from here.
    ScriptedEffect,
    /// File-level scope for `common/scripted_triggers/*.txt`. Same semantics
    /// as `ScriptedEffect` — top-level trigger definitions inherit the caller's
    /// scope at use-site.
    ScriptedTrigger,
    Unknown,
}

impl Scope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Scope::Global => "Global",
            Scope::Country => "Country",
            Scope::State => "State",
            Scope::Unit => "Unit",
            Scope::Character => "Character",
            Scope::MusicStation => "Music Station",
            Scope::MusicTrack => "Music Track",
            Scope::Achievement => "Achievement",
            Scope::Ribbon => "Ribbon",
            Scope::Idea => "Idea",
            Scope::HiddenIdeaCategory => "Hidden Idea Category",
            Scope::FocusTree => "Focus Tree",
            Scope::NationalFocus => "National Focus",
            Scope::StrategicRegion => "Strategic Region",
            Scope::Ace => "Ace",
            Scope::TechnologyTags => "Technology Tags",
            Scope::Technologies => "Technologies",
            Scope::TechnologyCategories => "Technology Categories",
            Scope::TechnologyFolders => "Technology Folders",
            Scope::ModifierBag => "Modifier Bag",
            Scope::OnActions => "On Actions",
            Scope::ScriptedEffect => "Scripted Effect",
            Scope::ScriptedTrigger => "Scripted Trigger",
            Scope::Unknown => "Unknown",
        }
    }

    /// Return the scope variant used for behavioral filtering (completions,
    /// validation). Focus-related scopes are semantically country-scoped,
    /// so they map to `Country` for filtering purposes.
    pub fn effective_scope(&self) -> Scope {
        match self {
            Scope::FocusTree | Scope::NationalFocus | Scope::Technologies => Scope::Country,
            Scope::OnActions => Scope::Global,
            Scope::ScriptedEffect | Scope::ScriptedTrigger => Scope::Country,
            Scope::Unknown => Scope::Global,
            other => *other,
        }
    }

    /// Map an `on_*` action name to its runtime scope.
    ///
    /// Data collected from wiki + vanilla `common/on_actions/*.txt` + `_documentation.md`:
    /// - `on_startup` is explicitly `none` (Global) — no country context.
    /// - State-default: border wars and naval invasions (THIS = invaded state).
    /// - Unit: `on_add_history` (ROOT = unit).
    /// - Character: unit-leader and operative actions (ROOT/THIS = character/operative).
    /// - Everything else is Country-scoped (ROOT = country, FROM = other country).
    ///
    ///   Unknown `on_*` fall back to Country; prefix `on_daily_`, `on_weekly_`,
    ///   `on_monthly_` are dynamic country-specific variants (Country).
    pub fn on_action_scope(key: &str) -> Option<Scope> {
        let lower = key.to_ascii_lowercase();
        // Dynamic per-country pulses
        if lower.starts_with("on_daily_")
            || lower.starts_with("on_weekly_")
            || lower.starts_with("on_monthly_")
        {
            return Some(Scope::Country);
        }
        match lower.as_str() {
            // Global — explicitly "none"
            "on_startup" => Some(Scope::Global),
            // State-default
            "on_border_war_lost" => Some(Scope::State),
            "on_naval_invasion" => Some(Scope::State),
            "on_paradrop" => Some(Scope::State),
            "on_units_paradropped_in_state" => Some(Scope::State),
            // Unit
            "on_add_history" => Some(Scope::Unit),
            // Character — unit leaders / operatives
            "on_unit_leader_created"
            | "on_army_leader_daily"
            | "on_army_leader_won_combat"
            | "on_army_leader_lost_combat"
            | "on_unit_leader_level_up"
            | "on_army_leader_promoted"
            | "on_unit_leader_promote_from_ranks_veteran"
            | "on_unit_leader_promote_from_ranks_green"
            | "on_deployed_leader_defeated"
            // Operative — THIS is the operative (Character)
            | "on_operative_created"
            | "on_operative_death"
            | "on_operative_recruited"
            | "on_operative_captured"
            | "on_operative_on_mission_spotted"
            | "on_operative_detected_during_operation"
            | "on_operation_completed" => Some(Scope::Character),
            // Country fallback for every other known on_action
            s if s.starts_with("on_") => Some(Scope::Country),
            _ => None,
        }
    }

    pub fn from_str(s: &str) -> Self {
        let s_lower = s.to_ascii_lowercase();
        match s_lower.as_str() {
            "music_station" => Scope::MusicStation,
            "music" => Scope::MusicTrack,
            "state" => Scope::State,
            "strategic_region" => Scope::StrategicRegion,
            "ideas" => Scope::Idea,
            "hidden_ideas" => Scope::HiddenIdeaCategory,
            // Focus-specific scopes — structural containers for focus definitions.
            // Mapped before Country so focus keywords aren't swallowed by the
            // Country wildcard match.
            "focus_tree" | "continuous_focus_palette" => Scope::FocusTree,
            "focus" | "shared_focus" | "joint_focus" => Scope::NationalFocus,
            "on_actions" => Scope::OnActions,
            // Technology tag containers — same reasoning as focus scopes:
            // structural, matched before the Country wildcard list.
            "technology_categories" => Scope::TechnologyCategories,
            "technology_folders" => Scope::TechnologyFolders,
            // Scripted effect/trigger containers — matched before Country
            // wildcard so they're recognized as structural scopes.
            "scripted_effect" | "scripted_effects" => Scope::ScriptedEffect,
            "scripted_trigger" | "scripted_triggers" => Scope::ScriptedTrigger,
            "country"
            | "ger"
            | "eng"
            | "fra"
            | "ita"
            | "jap"
            | "sov"
            | "usa"
            | "completion_reward"
            | "completion_reward_joint_originator"
            | "completion_reward_joint_member"
            | "select_effect"
            | "bypass_effect"
            | "cancel_effect"
            | "complete_tooltip"
            | "ai_will_do"
            | "available"
            | "available_if_capitulated"
            | "bypass"
            | "bypass_if_unavailable"
            | "allow_branch"
            | "will_lead_to_war_with"
            | "historical_ai"
            | "joint_trigger"
            | "supports_ai_strategy"
            | "cancel_if_invalid"
            | "continue_if_invalid"
            | "allowed"
            | "enable"
            | "daily_cost"
            | "on_start"
            | "immediate"
            | "option"
            | "after"
            | "country_event"
            | "news_event"
            | "on_action"
            | "modifier"
            | "trigger"
            | "limit"
            | "chance"
            | "any_country"
            | "every_country"
            | "random_country"
            | "any_neighbor_country"
            | "any_allied_country"
            | "any_war_adversary"
            | "any_war_ally"
            | "any_guaranteed_country"
            | "possible"
            | "happened" => Scope::Country,
            "any_state"
            | "every_state"
            | "random_state"
            | "all_state"
            | "any_neighbor_state"
            | "any_home_state"
            | "any_owned_state"
            | "all_owned_state"
            | "any_controlled_state"
            | "all_controlled_state"
            | "any_core_state"
            | "all_core_state" => Scope::State,
            "unit" | "any_unit" | "every_unit" | "random_unit" => Scope::Unit,
            "character"
            | "any_character"
            | "every_character"
            | "random_character"
            | "any_unit_leader"
            | "any_army_leader"
            | "any_navy_leader"
            | "any_operative_leader"
            | "all_operative_leader"
            | "every_operative_leader"
            | "random_operative_leader" => Scope::Character,
            _ => {
                // Any syntactically valid 3-char tag is a Country scope.
                // Reserved tags (NOT/AND/TAG/…) are *also* valid tags — the
                // engine loads them, they just break map modes etc., so we
                // treat them as Country and warn separately (HOM4005).
                if crate::scanner::country_scanner::is_valid_tag(s) {
                    Scope::Country
                } else {
                    Scope::Unknown
                }
            }
        }
    }
}

/// A node on the scope stack with transparency metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScopeNode {
    pub scope_type: Scope,
    pub is_transparent: bool,
}

impl ScopeNode {
    pub fn new(scope_type: Scope) -> Self {
        Self {
            scope_type,
            is_transparent: false,
        }
    }
}

/// Resolve a key to its semantic scope, with achievement/ribbon overrides.
/// Use this instead of `Scope::from_str` directly when achievements data is available.
/// This ensures the achievement-override logic lives in one place.
pub fn resolve_key_scope(
    key: &str,
    achievements: Option<&DashMap<InternedStr, LayeredValue<Achievement>>>,
) -> Scope {
    if let Some(achievements) = achievements {
        if let Some(achievement) = achievements.get(key) {
            if achievement.is_ribbon {
                return Scope::Ribbon;
            } else {
                return Scope::Achievement;
            }
        }
    }
    Scope::from_str(key)
}

/// External context for unified scope resolution via [`ScopeStack::resolve_entry_scope`].
///
/// Pass whatever data is available at the call site. Maps that aren't available
/// are `None` — the corresponding lookups are simply skipped.
pub struct ScopeCtx<'a> {
    pub uri: &'a str,
    /// Optional scanner maps so callers that lack them (hover/symbol paths)
    /// don't have to allocate empty DashMaps just to satisfy a non-Optional
    /// span. `None` simply means "no data for this lookup".
    pub event_targets: Option<&'a DashMap<InternedStr, Vec<EventTarget>>>,
    pub characters: Option<&'a DashMap<InternedStr, LayeredValue<Character>>>,
    pub achievements: Option<&'a DashMap<InternedStr, LayeredValue<Achievement>>>,
    pub in_random_list: bool,
    pub state_targeted: bool,
}

/// Detect the initial scope for a script file based on its path — the SAME
/// single source of truth used by the validation walker, hover, and completion
/// so they never diverge on file-type scope inference.
pub fn initial_scope_for_uri(uri: &str) -> Scope {
    if uri.contains("/common/abilities/") {
        Scope::Character
    } else if uri.contains("/common/decisions/") {
        Scope::Country
    } else if uri.contains("/common/aces/") {
        Scope::Ace
    } else if uri.contains("/common/units/") {
        // Unit type definitions (common/units/*.txt) — top level is
        // `sub_units = { ... }` with unit type blocks. No trigger/effect
        // evaluation at the top level, so Global is appropriate.
        // This ensures files are classified consistently with other definition files.
        Scope::Global
    } else if uri.contains("/common/technology_tags/") {
        // Top level of technology-tags files holds only the
        // technology_categories / technology_folders declaration blocks.
        // A dedicated scope (not Global) keeps country triggers/effects out
        // of top-level completion and flags stray ones via HOM004.
        Scope::TechnologyTags
    } else if uri.contains("/common/technologies/") {
        // Technology/doctrine definition files. Children of `technologies`
        // are tech definitions; trigger/effect bodies only appear inside
        // nested evaluation blocks (ai_will_do, available, on_research_complete).
        Scope::Technologies
    } else if uri.contains("/common/ai_faction_theaters/")
        || uri.contains("/common/ai_focuses/")
        || uri.contains("/common/ai_navy/taskforce/")
        || uri.contains("/common/ai_equipment/")
        || uri.contains("/common/ai_strategy/")
        || uri.contains("/common/ai_strategy_plans/")
        || uri.contains("/common/ai_templates/")
    {
        // Mapped to Country to avoid false positives (no scanners yet).
        Scope::Country
    } else if uri.contains("/common/on_actions/") {
        // `common/on_actions/` only ever holds `on_actions = { on_* = { ... } }`.
        // Starting from `OnActions` instead of `Global` removes the misleading
        // `Global > On Actions` prefix the editor was showing.
        Scope::OnActions
    } else if uri.contains("/common/scripted_effects/") {
        // Scripted effects are polymorphic — the top-level `my_effect = { }`
        // body runs in the caller's scope. This structural scope skips HOM004
        // at the top level (like ModifierBag) but still lets explicit
        // iterators (every_state, every_country, etc.) push correct scopes.
        Scope::ScriptedEffect
    } else if uri.contains("/common/scripted_triggers/") {
        // Same semantics as scripted_effects — polymorphic triggers.
        Scope::ScriptedTrigger
    } else {
        Scope::Global
    }
}

pub struct ScopeStack {
    nodes: Vec<ScopeNode>,
}

impl ScopeStack {
    pub fn new(initial: Scope) -> Self {
        Self {
            nodes: vec![ScopeNode::new(initial)],
        }
    }

    #[allow(dead_code)]
    pub fn push(&mut self, scope: Scope) {
        self.nodes.push(ScopeNode::new(scope));
    }

    /// Push a scope with explicit transparency flag.
    pub fn push_with(&mut self, scope: Scope, is_transparent: bool) {
        self.nodes.push(ScopeNode {
            scope_type: scope,
            is_transparent,
        });
    }

    pub fn pop(&mut self) -> Option<Scope> {
        self.nodes.pop().map(|n| n.scope_type)
    }

    pub fn current(&self) -> Scope {
        self.nodes.last().map_or(Scope::Global, |n| n.scope_type)
    }

    /// Get the current node for transparency checking
    #[allow(dead_code)]
    pub fn current_node(&self) -> Option<&ScopeNode> {
        self.nodes.last()
    }

    /// Returns scopes as a Vec<Scope> for backward compatibility.
    pub fn stack(&self) -> Vec<Scope> {
        self.nodes.iter().map(|n| n.scope_type).collect()
    }

    /// Iterate over Scope values (not ScopeNode).
    #[allow(dead_code)]
    pub fn iter(&self) -> impl Iterator<Item = &Scope> {
        self.nodes.iter().map(|n| &n.scope_type)
    }

    /// Get all scopes as a slice of nodes
    #[allow(dead_code)]
    pub fn nodes(&self) -> &[ScopeNode] {
        &self.nodes
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Resolve ROOT/THIS/PREV pointers, using transparency-aware semantics.
    /// - ROOT: First non-transparent scope in the stack
    /// - THIS: Current (top) scope
    /// - PREV: Previous non-transparent scope before the current one
    pub fn resolve_pointer(&self, pointer: &str) -> Scope {
        match pointer.to_ascii_uppercase().as_str() {
            "ROOT" => self
                .nodes
                .iter()
                .find(|n| !n.is_transparent)
                .map_or(Scope::Global, |n| n.scope_type),
            "THIS" => self.current(),
            "PREV" => self
                .nodes
                .iter()
                .rev()
                .skip(1)
                .find(|n| !n.is_transparent)
                .map_or(Scope::Global, |n| n.scope_type),
            _ => Scope::Unknown,
        }
    }

    /// Resolve a dot-notation scope chain like `ROOT.owner.capital.controller`.
    ///
    /// Walk the chain using V2 chain target data:
    /// 1. Split on `.`
    /// 2. First segment: resolve via pointer or from_str
    /// 3. Subsequent segments: look up chain_target(current, segment) in V2 data
    /// 4. Returns `(final_scope, is_known)`
    #[allow(dead_code)]
    pub fn resolve_chain(&self, key: &str) -> (Scope, bool) {
        let parts: Vec<&str> = key.split('.').collect();
        if parts.is_empty() {
            return (Scope::Unknown, false);
        }

        // Resolve the first segment
        let first = parts[0];
        let (mut current, mut known) = match first.to_ascii_uppercase().as_str() {
            "ROOT" => (self.resolve_pointer("ROOT"), true),
            "THIS" => (self.resolve_pointer("THIS"), true),
            "PREV" => (self.resolve_pointer("PREV"), true),
            "FROM" => (Scope::Country, true),
            _ => {
                let s = Scope::from_str(first);
                let known = s != Scope::Unknown;
                (s, known)
            }
        };

        // Walk subsequent segments via chain targets
        for segment in &parts[1..] {
            if let Some(target) = crate::data::hoi4_data::lookup_chain_target(&current, segment) {
                current = target.scope;
            } else {
                // Unknown link — try from_str as fallback
                let s = Scope::from_str(segment);
                if s != Scope::Unknown {
                    current = s;
                } else if segment.len() == 3
                    && segment.is_ascii()
                    && segment.as_bytes()[0].is_ascii_alphabetic()
                    && segment.as_bytes()[0].is_ascii_uppercase()
                {
                    // Country tag
                    current = Scope::Country;
                } else {
                    known = false;
                    break;
                }
            }
        }

        (current, known)
    }

    /// Resolve a meta-scope reference dynamically based on the current
    /// scope stack context.
    ///
    /// HOI4 defines several meta-scopes that refer to contextual scopes
    /// rather than named ones:
    ///
    /// | Keyword | Resolves to |
    /// |---------|------------|
    /// | `THIS`  | Current scope (top of stack) |
    /// | `ROOT`  | First non-transparent scope pushed (the entry point of the block) |
    /// | `PREV`  | Previous non-transparent scope (one level up) |
    /// | `PREVPREV` | Two non-transparent scopes up |
    /// | `FROM`  | Event/action source scope — typically `Country` |
    /// | `FROM.FROM` | Chained FROM — typically `Country` |
    ///
    /// Returns `None` when the key is not a meta-scope, so callers can
    /// fall back to [`Scope::from_str`] or [`resolve_key_scope`].
    pub fn resolve_meta_scope(&self, key: &str) -> Option<Scope> {
        let upper = key.to_ascii_uppercase();
        match upper.as_str() {
            // THIS = current scope (top of stack).
            // Always succeeds because the stack is never empty
            // (it always has at least Global).
            "THIS" => Some(self.current()),

            // ROOT = the first non-transparent scope pushed after Global.
            // In HOI4 this is usually Country (events, focuses, decisions),
            // but can be State (state events) or Character (character events).
            "ROOT" => Some(self.resolve_pointer("ROOT")),

            // PREV = previous non-transparent scope (one above current).
            // PREVPREV, PREVPREVPREV, etc. = N non-transparent scopes up.
            // We handle any string made of consecutive "PREV" parts.
            "PREV" | "PREVPREV" | "PREVPREVPREV" | "PREVPREVPREVPREV" => {
                let depth = upper.matches("PREV").count();
                let mut count = 0;
                for node in self.nodes.iter().rev().skip(1) {
                    if !node.is_transparent {
                        count += 1;
                        if count == depth {
                            return Some(node.scope_type);
                        }
                    }
                }
                Some(Scope::Unknown)
            }

            // FROM = source scope in events / targeted effects.
            // Cannot be determined statically without tracking which
            // event/effect fired this block. Default to Country since
            // most senders are countries.
            "FROM" => Some(Scope::Country),

            // Chained FROM references: FROM.FROM, FROM.FROM.FROM.
            // Also default to Country.
            _ if upper.starts_with("FROM.") => {
                let count = upper.matches("FROM").count();
                if count > 3 || count == 0 {
                    Some(Scope::Unknown)
                } else {
                    Some(Scope::Country)
                }
            }

            _ => None,
        }
    }

    /// Unified scope resolution for any block key.
    ///
    /// Returns `(scope, is_transparent)` — the scope to push and whether
    /// it should be transparent (passes THIS/ROOT/PREV through).
    ///
    /// Resolution order:
    /// 1. Transparent block (AND, OR, NOT, limit, if) → inherit current scope
    /// 2. V2 pushes_scope → explicit scope from trigger/effect data
    /// 3. Event target → scope saved by `save_event_target`
    /// 4. Chain target from current scope (e.g. State → owner → Country)
    /// 5. Ace file `effect` block → Country scope (ace modifiers are Country-scoped)
    /// 6. Modifier application blocks (`modifiers`, `*_modifiers`) → ModifierBag
    /// 7. Legacy fallback:
    ///    a. Meta-scope (THIS/ROOT/PREV/FROM)
    ///    b. Achievement-aware resolution via `resolve_key_scope`
    ///    c. Character token lookup
    ///    d. Numeric state IDs (outside `random_list`)
    ///    e. State-targeted FROM override
    ///
    /// NOTE: Idea promotion (Unknown keys at depth 2-3 inside Idea scope)
    /// is NOT handled here — callers that need it (the validation walker)
    /// apply it after this call using `is_idea_structure_key` from the
    /// rules module.
    pub fn resolve_entry_scope(&self, key: &str, ctx: &ScopeCtx) -> (Scope, bool) {
        // 1. Transparent block (AND, OR, NOT, limit, if)
        if crate::data::hoi4_data::is_transparent_block(key)
            || matches!(key.to_ascii_uppercase().as_str(), "AND" | "OR" | "NOT")
        {
            return (self.current(), true);
        }

        // 1b. On-actions: `on_actions` wrapper and per-action `on_*` blocks
        // (only in common/on_actions/*.txt). This runs before V2/chain lookups
        // so each on_action gets its documented runtime scope (Country / State /
        // Character / Unit / Global) instead of falling back to Global/Unknown.
        //
        // Guard: only direct children of the `on_actions` container push an
        // on_action scope. Without this, any nested `on_` key (e.g. inside an
        // effect block) would be mis-scoped.
        if ctx.uri.contains("/common/on_actions/") {
            if key.eq_ignore_ascii_case("on_actions") {
                // File's initial scope already IS `OnActions` (see
                // `initial_scope_for_uri`). The literal `on_actions = { }`
                // wrapper would otherwise push a duplicate `OnActions` on top
                // of it (`On Actions > On Actions`). Make it a no-op so the
                // stack stays `On Actions > <on_*>` instead of
                // `Global > On Actions > <on_*>` / `On Actions > On Actions`.
                if self.current() == Scope::OnActions {
                    return (Scope::Unknown, false);
                }
                return (Scope::OnActions, false);
            }
            if self.current() == Scope::OnActions {
                if let Some(s) = Scope::on_action_scope(key) {
                    return (s, false);
                }
            }
        }

        // 2. V2: Known trigger/effect with explicit scope push
        if let Some(pushed) = crate::data::hoi4_data::lookup_pushes_scope(key) {
            return (pushed, false);
        }

        // 3. V2: Saved event target
        if let Some(event_targets) = ctx.event_targets {
            let lower_key = key.to_ascii_lowercase();
            let result = event_targets
                .get(&*lower_key)
                .or_else(|| event_targets.get(key));
            if let Some(scope) = result
                .and_then(|targets| targets.value().first().map(|t| t.scope))
                .filter(|s| *s != Scope::Unknown)
            {
                return (scope, false);
            }
        }

        // 4. Chain target from current scope (e.g. State -> owner -> Country)
        if let Some(chain_target) =
            crate::data::hoi4_data::lookup_chain_target(&self.current(), key)
        {
            return (chain_target.scope, false);
        }

        // 5. Ace file: `effect` block contains Country-scope modifiers
        if key == "effect" && ctx.uri.contains("/common/aces/") {
            return (Scope::Country, false);
        }

        // 6. Modifier application blocks (unit_modifiers, modifiers, *_modifiers)
        if key == "modifiers" || key.ends_with("_modifiers") {
            return (Scope::ModifierBag, true);
        }

        // 7. Legacy fallback
        let mut s = self
            .resolve_meta_scope(key)
            .unwrap_or_else(|| resolve_key_scope(key, ctx.achievements));

        // 7c. Known character tokens -> Character scope
        if s == Scope::Unknown {
            if let Some(characters) = ctx.characters {
                if characters.contains_key(key) {
                    s = Scope::Character;
                }
            }
        }

        // 7d. Numeric keys (state IDs) -> State scope (outside random_list)
        if s == Scope::Unknown
            && !ctx.in_random_list
            && !key.is_empty()
            && key.as_bytes().iter().all(|b| b.is_ascii_digit())
        {
            s = Scope::State;
        }

        // 7e. State-targeted decisions: FROM -> State
        if s == Scope::Country && ctx.state_targeted && key.eq_ignore_ascii_case("FROM") {
            s = Scope::State;
        }

        (s, false)
    }
}
