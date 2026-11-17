use crate::data::interner::InternedStr;
use crate::data::layered_value::LayeredValue;
use crate::parser::ast;
use crate::scanner::achievement_scanner::Achievement;
use crate::scanner::character_scanner::Character;
use crate::scanner::variable_scanner::{EventTarget, Variable};
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
    /// Combatant scope used by `common/combat_tactics.txt`: a tactic body and its
    /// `trigger` evaluate in a combined Country+Unit context (vanilla documents
    /// `trigger = { # combined scope for both country and combatant }`), so both
    /// country and unit triggers/effects are valid here. HOM004 accepts either.
    Combatant,
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
    /// A modifier-application target block (e.g. `unit_modifiers = { }`), an
    /// idea `rule = { }` block, or any other flat bag of modifier keys.
    /// The engine reads these as a flat bag of modifiers and routes them
    /// per-key — not a trigger/effect evaluation scope. V2ScopeRule skips
    /// scope checks inside these blocks.
    ModifierBag,
    /// A trigger block inside a character definition (`allowed`, `available`,
    /// `visible` under `common/characters/` or `generate_character`). The engine
    /// evaluates these with the character as THIS, yet vanilla freely mixes
    /// country triggers (`original_tag`, `has_completed_focus`) with character
    /// triggers (`has_character_flag`, `is_country_leader`) in the same block,
    /// so neither scope describes it. `V2ScopeRule` skips scope checks here —
    /// a false negative is cheaper than flagging valid vanilla content.
    CharacterCondition,
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
    /// File-level scope for `common/scripted_localisation/*.txt`.
    ///
    /// A `defined_text = { name/text = { trigger = { ... } localization_key = } }`
    /// body is evaluated in whatever scope the displayed string is used in —
    /// the same polymorphism as `ScriptedEffect`, only for localisation. No
    /// single runtime scope describes it, so HOM004 is skipped at the
    /// structural level while explicit iterators still push their own scopes.
    ScriptedLocalisation,
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
            Scope::Combatant => "Combatant",
            Scope::Ace => "Ace",
            Scope::TechnologyTags => "Technology Tags",
            Scope::Technologies => "Technologies",
            Scope::TechnologyCategories => "Technology Categories",
            Scope::TechnologyFolders => "Technology Folders",
            Scope::ModifierBag => "Modifier Bag",
            Scope::CharacterCondition => "Character Condition",
            Scope::OnActions => "On Actions",
            Scope::ScriptedEffect => "Scripted Effect",
            Scope::ScriptedTrigger => "Scripted Trigger",
            Scope::ScriptedLocalisation => "Scripted Localisation",
            Scope::Unknown => "Unknown",
        }
    }

    /// Return the scope variant used for behavioral filtering (completions,
    /// validation). Focus-related scopes are semantically country-scoped,
    /// so they map to `Country` for filtering purposes.
    pub fn effective_scope(&self) -> Scope {
        match self {
            Scope::FocusTree | Scope::NationalFocus | Scope::Technologies => Scope::Country,
            Scope::Combatant => Scope::Country,
            Scope::OnActions => Scope::Global,
            Scope::ScriptedEffect | Scope::ScriptedTrigger | Scope::ScriptedLocalisation => {
                Scope::Country
            }
            // Character condition blocks are character-context trigger blocks;
            // `Character` gives the most useful completion filtering (character
            // triggers dominate them).
            Scope::CharacterCondition => Scope::Character,
            Scope::Unknown => Scope::Global,
            other => *other,
        }
    }

    /// Specificity rank for completion filtering. Higher means more specific,
    /// so when the scope stack holds several non-transparent scopes we pick
    /// the narrowest one and water down completions to the user context.
    /// Tiers: 5 narrow runtime scopes, 4 entity-defining scopes,
    /// 3 State, 2 Country, 1 Global and file-level markers,
    /// 0 Unknown and ModifierBag (no useful filtering semantics).
    pub fn scope_tier_rank(&self) -> u8 {
        match self {
            Scope::Unit
            | Scope::Character
            | Scope::Ace
            | Scope::StrategicRegion
            | Scope::Combatant => 5,
            Scope::Idea
            | Scope::MusicTrack
            | Scope::MusicStation
            | Scope::Achievement
            | Scope::HiddenIdeaCategory
            | Scope::Ribbon => 4,
            Scope::State => 3,
            Scope::Country => 2,
            Scope::Global
            | Scope::Technologies
            | Scope::TechnologyTags
            | Scope::TechnologyCategories
            | Scope::TechnologyFolders
            | Scope::OnActions
            | Scope::ScriptedEffect
            | Scope::ScriptedTrigger
            | Scope::ScriptedLocalisation
            | Scope::FocusTree
            | Scope::NationalFocus => 1,
            Scope::Unknown | Scope::ModifierBag | Scope::CharacterCondition => 0,
        }
    }

    /// Pick the most specific scope from the stack for completion filtering.
    /// Transparent scopes (ModifierBag, Unknown) are skipped; an empty or
    /// fully transparent stack falls back to Global.
    pub fn pick_inferred_scope(scopes: &[Scope]) -> Scope {
        let mut best_scope = Scope::Global;
        let mut best_rank = 0u8;
        let mut found = false;
        for scope in scopes.iter().copied() {
            if scope == Scope::ModifierBag || scope == Scope::Unknown {
                continue;
            }
            let rank = scope.scope_tier_rank();
            if !found || rank > best_rank {
                found = true;
                best_rank = rank;
                best_scope = scope;
            }
        }
        best_scope
    }

    /// The runtime scope a definition-body scope evaluates against.
    ///
    /// Entity-definition scopes exist so the editor can tell an idea body from
    /// a country body, but the engine evaluates them on a country: a `ROOT`
    /// pointer inside `common/ideas` or a focus tree means the country, not the
    /// definition container. Used by [`ScopeStack::resolve_pointer`] so `ROOT`
    /// does not resolve to `Idea`/`Focus Tree`.
    pub fn runtime_root_scope(&self) -> Scope {
        match self {
            Scope::Idea
            | Scope::HiddenIdeaCategory
            | Scope::NationalFocus
            | Scope::FocusTree
            | Scope::Technologies
            | Scope::ScriptedLocalisation => Scope::Country,
            other => *other,
        }
    }

    /// True for scopes that only ever sit at the bottom of a stack: the file's
    /// initial marker, not a scope its content runs in.
    ///
    /// [`ScopeStack::resolve_pointer`] skips one of these at index 0 so `ROOT`
    /// lands on the first *runtime* scope the content executes in. `OnActions`
    /// is the case that matters: the file starts at `OnActions`, each `on_*`
    /// hook pushes its own runtime scope, and `root = { ... }` inside the hook
    /// must resolve to the hook's scope — not to the marker, whose effective
    /// scope is Global (vanilla `07_nsb_on_actions.txt` does exactly this in
    /// 763 places).
    ///
    /// Deliberately narrow: scopes that carry a meaningful runtime root of
    /// their own (`Idea`, `NationalFocus`, `Technologies` → Country; `Ace` →
    /// the ace) are NOT markers, so `ROOT` keeps resolving through them.
    pub fn is_file_level_marker(&self) -> bool {
        matches!(
            self,
            Scope::Global
                | Scope::Unknown
                | Scope::OnActions
                | Scope::ModifierBag
                | Scope::CharacterCondition
                | Scope::TechnologyTags
                | Scope::TechnologyCategories
                | Scope::TechnologyFolders
        )
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
            // Character definitions. `characters` is the file-level container in
            // `common/characters/*.txt`; `generate_character` is the effect-block
            // form used by `history/general/*.txt`. Both bodies are evaluated
            // with ROOT/THIS = the character — role blocks (`advisor`,
            // `corps_commander`, `country_leader`, ...) inherit this scope and
            // must NOT be listed here: `country_leader = { ... }` also appears
            // inside effects and decisions, where it is not a character body.
            "characters" | "generate_character" => Scope::Character,
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
            | "activation"
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
            | "allowed_to_remove"
            | "allowed_civil_war"
            | "cancel"
            | "do_effect"
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
#[derive(Clone, Copy)]
pub struct ScopeCtx<'a> {
    pub uri: &'a str,
    /// Optional scanner maps so callers that lack them (hover/symbol paths)
    /// don't have to allocate empty DashMaps just to satisfy a non-Optional
    /// span. `None` simply means "no data for this lookup".
    pub event_targets: Option<&'a DashMap<InternedStr, Vec<EventTarget>>>,
    pub characters: Option<&'a DashMap<InternedStr, LayeredValue<Character>>>,
    pub achievements: Option<&'a DashMap<InternedStr, LayeredValue<Achievement>>>,
    /// Tracked variables (`set_variable` etc. via `variable_scanner`) with
    /// statically inferred value scopes. Powers `var:name = { }` /
    /// `temp_var:name = { }` scope blocks. `None` simply means "no data for
    /// this lookup" (unknown vars keep resolving `Unknown`, as before).
    pub variables: Option<&'a DashMap<InternedStr, Vec<Variable>>>,
    /// True when the keys of the block being resolved are `random_list` /
    /// `random_events` weights. A numeric weight is not a state id.
    pub in_random_list: bool,
    /// True inside a decision that carries `state_target`, where `FROM` is the
    /// target state rather than the country.
    pub state_targeted: bool,
}

impl<'a> ScopeCtx<'a> {
    /// Build a ctx from a file's scanned maps. The two context flags start
    /// false, which is correct at file root; walkers that descend blocks call
    /// [`child_flags`] and pass the result through [`with_flags`].
    pub fn new(
        uri: &'a str,
        event_targets: Option<&'a DashMap<InternedStr, Vec<EventTarget>>>,
        characters: Option<&'a DashMap<InternedStr, LayeredValue<Character>>>,
        achievements: Option<&'a DashMap<InternedStr, LayeredValue<Achievement>>>,
        variables: Option<&'a DashMap<InternedStr, Vec<Variable>>>,
    ) -> Self {
        ScopeCtx {
            uri,
            event_targets,
            characters,
            achievements,
            variables,
            in_random_list: false,
            state_targeted: false,
        }
    }

    #[must_use]
    pub fn with_flags(self, in_random_list: bool, state_targeted: bool) -> Self {
        ScopeCtx {
            in_random_list,
            state_targeted,
            ..self
        }
    }
}

/// Context flags for the entries INSIDE a block whose key is `key`.
///
/// `random_list` and `random_events` both use numeric weights as keys, which
/// must not resolve as state ids. `state_target` marks a decision whose `FROM`
/// is the target state; vanilla writes it as `yes`, a state iterator or a
/// continent name, so any value other than `no` counts.
///
/// The random flag describes this block only (mirroring the validation walker,
/// which resets it per block), while the state-target flag inherits down the
/// tree.
pub fn child_flags(
    key: &str,
    entries: &[ast::Entry],
    source: &str,
    parent_state_targeted: bool,
) -> (bool, bool) {
    (
        is_weight_block_key(key),
        parent_state_targeted || block_targets_state(entries, source),
    )
}

/// `random_list` and `random_events` keys hold weights, not state ids.
pub fn is_weight_block_key(key: &str) -> bool {
    key.eq_ignore_ascii_case("random_list") || key.eq_ignore_ascii_case("random_events")
}

/// Whether a block declares `state_target`.
///
/// The value is not always a boolean. Vanilla writes `state_target = yes`
/// (209 sites), a state iterator (`any_controlled_state` 34, `any_owned_state`
/// 20) and continent names (`europe`, `asia`, `africa`, …). Every form means
/// the decision targets a state, so `FROM` inside its body is that state.
/// Recognising only the boolean left the other forms resolving `FROM` to
/// Country.
pub fn block_targets_state(entries: &[ast::Entry], source: &str) -> bool {
    entries.iter().any(|entry| match entry {
        ast::Entry::Assignment(ass) => {
            ass.key_text(source).eq_ignore_ascii_case("state_target")
                && !matches!(&ass.value.value, ast::Value::Boolean(false))
        }
        _ => false,
    })
}

/// Detect the initial scope for a script file based on its path — the SAME
/// single source of truth used by the validation walker, hover, and completion
/// so they never diverge on file-type scope inference.
pub fn initial_scope_for_uri(uri: &str) -> Scope {
    if uri.contains("/common/abilities/") {
        Scope::Character
    } else if uri.contains("/common/characters/") {
        // Character definition files: `characters = { token = { ... } }`.
        // The file starts in Country scope so that ROOT resolves to the country
        // (in a character file THIS is the character, ROOT is its country — see
        // vanilla `on_add = { IF = { limit = { ROOT = { has_government = ... }`
        // }). The `characters` wrapper then pushes Character, so every token
        // body evaluates with THIS = character.
        Scope::Country
    } else if uri.contains("/history/countries/") {
        // History files are evaluated in the country's scope: `capital`, `oob`,
        // `set_technology`, `add_ideas`, `set_politics`, … at top level.
        Scope::Country
    } else if uri.contains("/common/decisions/") {
        Scope::Country
    } else if uri.contains("/common/combat_tactics/") || uri.ends_with("/common/combat_tactics.txt")
    {
        // Tactic bodies evaluate in a combined Country+Unit context (the engine
        // provides both the country and the combatant). Modelled as Combatant so
        // triggers valid in either scope are accepted.
        Scope::Combatant
    } else if uri.contains("/common/profile_pictures/")
        || uri.contains("/common/profile_backgrounds/")
    {
        // Data-only definition files (cosmetic/profile assets). No trigger/effect
        // evaluation; treat bodies as modifier/property bags to avoid false HOM004.
        Scope::ModifierBag
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
    } else if uri.contains("/history/units/") {
        // Order-of-battle files. Top level is country script:
        // `division_template = { ... }`, `add_equipment_production = { ... }`,
        // plus bare `has_government` / `original_tag` conditions. Starting at
        // Global turned 6.8k valid OOB lines into HOM004.
        Scope::Country
    } else if uri.contains("/common/scripted_localisation/") {
        // `defined_text` bodies are polymorphic — see `ScriptedLocalisation`.
        Scope::ScriptedLocalisation
    } else if uri.contains("/common/autonomous_states/")
        || uri.contains("/common/bop/")
        || uri.contains("/common/factions/")
        || uri.contains("/common/doctrines/")
        || uri.contains("/common/military_industrial_organization/")
        || uri.contains("/common/special_projects/")
        || uri.contains("/common/raids/")
        || uri.contains("/common/operations/")
        || uri.contains("/common/intelligence_agencies/")
        || uri.contains("/common/technology_sharing/")
        || uri.contains("/common/collections/")
        || uri.contains("/common/focus_inlay_windows/")
        || uri.contains("/common/operation_phases/")
        || uri.contains("/common/wargoals/")
        || uri.contains("/common/resistance_activity/")
    {
        // Country-evaluated definition families: their bodies hold country
        // triggers (`available`, `allowed`, `visible`, `enable`, `limit`,
        // `original_tag`) inside transparent blocks, so the definition block
        // itself is what has to carry the scope. Falling through to Global made
        // every one of them a HOM004 false positive.
        Scope::Country
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
            // ROOT = the runtime scope the definition body is evaluated against.
            // The bottom node is the file's initial marker, which is `Global`
            // for definition files (`ideas`, `events`, `common/characters`, ...)
            // — it is not a scope the content runs in, so it is skipped. Entity
            // definition scopes then map to the country they evaluate on
            // (`Idea`, `Focus Tree`, `Technologies`), so `ROOT = { ... }` inside
            // an idea checks country triggers, as vanilla expects.
            "ROOT" => self
                .nodes
                .iter()
                .enumerate()
                .find(|(idx, n)| {
                    !n.is_transparent && !(*idx == 0 && n.scope_type.is_file_level_marker())
                })
                .map_or(Scope::Global, |(_, n)| n.scope_type.runtime_root_scope()),
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
    ///    3b. `var:` / `temp_var:` scope block → inferred variable value scope
    /// 4. Chain target from current scope (e.g. State → owner → Country)
    /// 5. Ace file `effect` block → Country scope (ace modifiers are Country-scoped)
    /// 6. Modifier application blocks (`modifier`, `modifiers`, `*_modifier(s)`,
    ///    `research_bonus`, `equipment_bonus`) → ModifierBag
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

        // 1a. Character definition blocks. Inside a character body the engine
        // evaluates `allowed` / `available` / `visible` with the character as
        // THIS, but vanilla mixes country triggers (`original_tag`,
        // `has_completed_focus`) with character triggers (`has_character_flag`,
        // `is_country_leader`) in the same block — including `visible`, which
        // the V2 data otherwise pushes to Country as a technology-folder block.
        // Neither scope is right, so validation is skipped for the subtree.
        if self.current() == Scope::Character {
            let lower = key.to_ascii_lowercase();
            if matches!(lower.as_str(), "allowed" | "available" | "visible") {
                // `common/abilities/*.txt` is the one place where `allowed` is a
                // pure Character trigger block (`OWNER = { ... }` inside it
                // resolves as Character) — keep validating it there.
                if ctx.uri.contains("/common/abilities/") {
                    return (Scope::Character, false);
                }
                return (Scope::CharacterCondition, true);
            }
            // `on_add` / `on_remove` on an advisor or leader body run with THIS =
            // the character (vanilla sets character flags there); `ROOT` inside
            // them resolves to the owning country.
            if matches!(lower.as_str(), "on_add" | "on_remove") {
                return (Scope::Character, false);
            }
        }

        // 1a'. Hook blocks evaluated on the owning country everywhere else —
        // idea and decision `on_add` / `on_remove` apply to the country.
        if matches!(key.to_ascii_lowercase().as_str(), "on_add" | "on_remove") {
            return (Scope::Country, false);
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

        let lk = key.to_ascii_lowercase();

        // 1c. Event definition blocks (`events/*.txt`). The file's initial
        // scope stays Global — correct, because `namespace = ...` and
        // `add_namespace = ...` are file-level declarations. Each event
        // definition then runs in its own scope: country events on a country,
        // state events on a state, leader events on the leader. Without this
        // the whole body (`trigger`, `immediate`, `option`) validated at Global
        // and every country trigger inside it was a false positive.
        //
        // Gated on `/events/` because `country_event = { id = ... }` is ALSO a
        // fire-an-event effect used all over script.
        if ctx.uri.contains("/events/") {
            match lk.as_str() {
                "country_event" | "news_event" => return (Scope::Country, false),
                "state_event" => return (Scope::State, false),
                // Leader/operative events are SHOWN to the leader's owner:
                // vanilla puts country effects (`activate_mission`,
                // `puppet`, `add_political_power`) straight in their options,
                // with the leader as FROM. The body is country scope.
                "unit_leader_event" | "operative_leader_event" => return (Scope::Country, false),
                _ => {}
            }
        }

        // 1d. Modifier-container / data definition blocks whose bodies are flat
        // bags of properties or modifiers, not trigger/effect evaluation scopes.
        // `leader_traits`, dynamic/static modifiers, terrain categories, state
        // categories, building definitions, intelligence agency upgrades and
        // character-generation data all hold keys that collide with triggers
        // (e.g. `hardness`, `naval_mine_hit_chance`, `local_building_slots`,
        // `attack_skill_level`, `civilian_factory_use`). Treating their top-level
        // definition blocks as ModifierBag skips HOM004 while nested evaluation
        // blocks with their own structural scope are still pushed by later arms.
        if lk == "leader_traits" || lk == "unit_leader_traits" {
            return (Scope::ModifierBag, true);
        }
        if self.current() == Scope::Global
            && (ctx.uri.contains("/common/dynamic_modifiers/")
                || ctx.uri.contains("/common/modifiers/")
                || ctx.uri.contains("/common/terrain/")
                || ctx.uri.contains("/common/state_category/")
                || ctx.uri.contains("/common/buildings/")
                || ctx.uri.contains("/common/intelligence_agency_upgrades/")
                || ctx.uri.contains("/common/generation/")
                || ctx.uri.contains("/common/country_leader/")
                || ctx.uri.contains("/common/unit_leader/")
                || ctx.uri.contains("/common/opinion_modifiers/"))
        {
            return (Scope::ModifierBag, true);
        }

        // 1e. Structural blocks whose body scope the engine defines but the
        // data tables do not push.
        match lk.as_str() {
            // Unit-leader trait gate: `unit_trigger = { division_has_majority_template = ... }`
            // is a division trigger block, so `owner = { ... }` inside it chains
            // to Country the normal way.
            "unit_trigger" => return (Scope::Unit, false),
            // A trait's XP-gain gate is a DIVISION trigger block: vanilla
            // gates it on `temperature`, `has_combat_modifier`,
            // `is_fighting_in_terrain`, `fastest_unit`. (The `gain_xp = 10`
            // effect is a scalar and never reaches this arm.)
            "gain_xp" if ctx.uri.contains("/common/unit_leader/") => return (Scope::Unit, false),
            // Special-project scientist blocks evaluate on the scientist.
            "scientist_effects" | "scientist_trigger" => return (Scope::Character, false),
            // Operation target selection runs on the candidate state.
            "selection_target_state" => return (Scope::State, false),
            // Balance-of-power activation hooks run for the country holding it.
            "on_activate" | "on_deactivate" => return (Scope::Country, false),
            _ => {}
        }

        // 1f. Raid outcome blocks. `success = { base = 0.75 experience = {
        // weight = 0.25 } resistance = { ... } }` is a probability/weight bag,
        // not a trigger body — its entries are weights per outcome axis. The
        // effect blocks nested inside it do evaluate, on the victim/raider
        // country respectively.
        if ctx.uri.contains("/common/raids/") {
            match lk.as_str() {
                "success" | "failure" => return (Scope::ModifierBag, true),
                "victim_effects" | "raider_effects" => return (Scope::Country, false),
                _ => {}
            }
        }

        // 1f'. Resistance activities (`common/resistance_activity/`): the
        // activity is evaluated per state, so `available`, `weight`, `effect`
        // and `state_modifier` run in State scope (vanilla checks
        // `non_damaged_building_level` in `available`, applies state damage in
        // `effect`).
        if ctx.uri.contains("/common/resistance_activity/") {
            match lk.as_str() {
                "available" | "weight" | "effect" | "state_modifier" => {
                    return (Scope::State, false)
                }
                _ => {}
            }
        }

        // 1f''. Special projects: `facility_state_effects` runs in the state
        // hosting the facility (vanilla does `set_building_level` there), while
        // `country_effects` / `enable_equipment_modules` run on the owning
        // country.
        if ctx.uri.contains("/common/special_projects/") {
            match lk.as_str() {
                "facility_state_effects" => return (Scope::State, false),
                "country_effects" | "enable_equipment_modules" => {
                    return (Scope::Country, false)
                }
                _ => {}
            }
        }

        // 1g. Collection definitions (`common/collections/`). `limit` and
        // `operators` run once per ELEMENT of the collection, and the element
        // scope is decided by the collection's `input` (`game:all_states` →
        // State, `game:all_countries` → Country, `game:all_controlled_states`
        // → State, …). That is not statically resolvable from the block alone,
        // so no scope is claimed here rather than guessing Country and
        // flagging every state trigger.
        if ctx.uri.contains("/common/collections/")
            && matches!(lk.as_str(), "limit" | "operators" | "input")
        {
            return (Scope::ModifierBag, true);
        }

        // 1h. Combat tactics (`common/combat_tactics.txt`). The file starts in
        // Combatant scope (combined Country+Unit). Tactic definition bodies and
        // their `trigger`/`active` evaluation stay in that combined scope so
        // both country and unit conditions are valid; modifier/stat leaves like
        // `attacker`, `defender`, `base` are numeric/property bags and are not
        // scope-checked as triggers when used as assignments, but as blocks they
        // can hold modifier math — treat `base` as a modifier bag.
        if ctx.uri.contains("/common/combat_tactics") {
            if lk == "base" {
                return (Scope::ModifierBag, true);
            }
            if lk == "phases" {
                return (Scope::ModifierBag, true);
            }
        }

        // 1i. Unit and equipment definitions (`common/units/`). Stat/property
        // bodies: `equipments` children are equipment archetypes/variants, and
        // `sub_units` children are unit type definitions — both are flat bags of
        // stats/modifiers, not trigger bodies. Treat their definition blocks as
        // ModifierBag. The `sub_units` / `equipments` wrappers themselves are
        // transparent containers (inherit file scope).
        if ctx.uri.contains("/common/units/") {
            if matches!(lk.as_str(), "equipments" | "sub_units") {
                return (self.current(), true);
            }
            // Equipment definitions live under `equipments = { <def> = { ... } }`.
            // Sub-unit definitions live under `sub_units = { <def> = { ... } }`.
            // At those positions the current scope is the file's Global (wrapper
            // was transparent); treat definition bodies as property bags.
            if self.current() == Scope::Global {
                return (Scope::ModifierBag, true);
            }
        }

        // 1j. State-targeted decisions: `target_trigger` / `target_root_trigger`
        // evaluate their conditions against the target state, while the rest of
        // the decision body (`available`, `visible`, `complete_effect`, etc.)
        // evaluates on the country. Only when the enclosing decision declares
        // `state_target` (ctx.state_targeted).
        if ctx.uri.contains("/common/decisions/") && ctx.state_targeted {
            match lk.as_str() {
                // `highlight_state_targets` holds state ids; the trigger blocks around
                // it are NOT state-scoped. Vanilla writes `target_trigger` with ROOT =
                // the acting country and FROM = the candidate state, so country
                // triggers (`owns_state = FROM`, `has_completed_focus`,
                // `original_tag`) are legal there and pushing State makes them lie.
                // Same for `target_root_trigger`, which runs on the country outright.
                "highlight_state_targets" => return (Scope::State, false),
                "targeted_modifier" => return (Scope::ModifierBag, true),
                _ => {}
            }
        }

        // 1k. AI strategy / focus / plan data: `states` is a numeric state-id
        // list; `research`, `ai_national_focuses`, `ideas`, `traits` and
        // `equipment_variant` blocks are property/weight bags (e.g. `armor` in
        // `research = { armor = 25.0 }` is a tech-category weight, not the
        // Unit-scope trigger). `area` in ai_strategy entries names a strategic
        // area, colliding with the State-scope trigger of the same name — do
        // not push a scope for it.
        if ctx.uri.contains("/common/ai_strategy/")
            || ctx.uri.contains("/common/ai_strategy_plans/")
            || ctx.uri.contains("/common/ai_focuses/")
        {
            match lk.as_str() {
                "states" => return (self.current(), true),
                "research" | "ai_national_focuses" | "ideas" | "traits"
                | "equipment_variant" | "allowed" => {
                    // 'allowed' here is a simple condition bag in some plan files,
                    // but keeping it as transparent/current is fine; treat heavy
                    // property bags as ModifierBag to silence trigger-name collisions.
                }
                _ => {}
            }
            if matches!(
                lk.as_str(),
                "research" | "ai_national_focuses" | "ideas" | "traits" | "equipment_variant"
            ) {
                return (Scope::ModifierBag, true);
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

        // 3b. `var:` / `temp_var:` scope block → inferred variable value scope.
        //
        // The engine runs `var:my_var = { ... }` in whatever scope object the
        // variable currently holds. Statically we only know what the scanner
        // inferred from `set_variable` RHS values (TAG-anchored → Country;
        // anything else → Unknown). A known non-Unknown scope is pushed;
        // unknown vars fall through to the legacy path below (→ Unknown),
        // preserving the old safe behaviour (HOM004 skips Unknown scopes).
        //
        // The remainder after the prefix is the variable name as tracked by
        // the scanner, including scope-qualified forms (`var:ETH.host_nation`
        // ↔ `set_variable = { ETH.host_nation = ROOT }`). `temp_var:` is
        // checked first. Keys may be non-ASCII, so the prefix is matched on
        // bytes — see `strip_var_prefix`.
        let var_name = crate::data::hoi4_data::strip_var_prefix(key);
        if let Some(var_name) = var_name {
            // Case-insensitive prefix, case-preserving name: `VAR:X` and
            // `var:X` address the same variable. The scanner stores raw-case
            // keys, so try the raw name first, then its lowercase form (which
            // matches when the definition was written lowercase).
            if !var_name.is_empty() {
                if let Some(variables) = ctx.variables {
                    let scope = variables
                        .get(var_name)
                        .or_else(|| variables.get(&*var_name.to_ascii_lowercase()))
                        .and_then(|defs| {
                            defs.value().iter().find_map(|d| {
                                if d.scope != Scope::Unknown {
                                    Some(d.scope)
                                } else {
                                    None
                                }
                            })
                        });
                    if let Some(s) = scope {
                        return (s, false);
                    }
                }
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

        // 5b. Strategic region weather blocks are property bags: temperature,
        // no_phenomenon, rain_light, ... are not triggers/effects and must not
        // be checked against the enclosing Strategic Region scope.
        let lower_key = key.to_ascii_lowercase();
        if (lower_key == "weather" || lower_key == "period")
            && ctx.uri.contains("/map/strategicregions/")
        {
            return (Scope::ModifierBag, true);
        }

        // 6. Modifier application blocks → ModifierBag (validation skipped).
        // Covers the plural/plural-suffixed forms (`modifiers`, `unit_modifiers`),
        // the singular containers used by ideas and characters — `modifier`,
        // `targeted_modifier`, `research_bonus`, `equipment_bonus` — and the
        // idea `rule` block, whose entries are declarative permissions
        // (`can_guarantee_other_ideologies`) rather than scoped triggers.
        // Their entries are modifier keys and equipment/research categories, not
        // triggers: validating `armor = 0.15` as a trigger against the enclosing
        // Idea scope is a category error (modifiers have no runtime scope).
        if lower_key == "modifiers"
            || lower_key == "modifier"
            || lower_key.ends_with("_modifiers")
            || lower_key.ends_with("_modifier")
            || lower_key == "research_bonus"
            || lower_key == "equipment_bonus"
            || lower_key == "rule"
        {
            return (Scope::ModifierBag, true);
        }

        // 7. Legacy fallback
        let mut s = self
            .resolve_meta_scope(key)
            .unwrap_or_else(|| resolve_key_scope(key, ctx.achievements));

        // 7a'. Peace conference actions evaluate FROM.FROM.FROM as the target
        // state (FROM=negotiator, FROM.FROM=loser, FROM.FROM.FROM=state).
        if ctx.uri.contains("/common/peace_conference/")
            && key.eq_ignore_ascii_case("FROM.FROM.FROM")
        {
            s = Scope::State;
        }

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
            // Data-only definition files and unit/equipment definition trees:
            // numeric entries are IDs/slots/levels, not state scopes.
            if ctx.uri.contains("/common/profile_pictures/")
                || ctx.uri.contains("/common/profile_backgrounds/")
                || ctx.uri.contains("/common/units/")
            {
                s = Scope::ModifierBag;
            } else {
                s = Scope::State;
            }
        }

        // 7e. State-targeted decisions: FROM -> State
        if s == Scope::Country && ctx.state_targeted && key.eq_ignore_ascii_case("FROM") {
            s = Scope::State;
        }

        (s, false)
    }
}

#[cfg(test)]
mod tests {
    use super::Scope;

    #[test]
    fn test_pick_inferred_scope_prefers_narrowest() {
        // Character outranks Country and State.
        assert_eq!(
            Scope::pick_inferred_scope(&[Scope::Country, Scope::State, Scope::Character]),
            Scope::Character
        );
        // State outranks Country.
        assert_eq!(
            Scope::pick_inferred_scope(&[Scope::Country, Scope::State]),
            Scope::State
        );
        // Transparent scopes are skipped.
        assert_eq!(
            Scope::pick_inferred_scope(&[Scope::Country, Scope::ModifierBag, Scope::Unknown]),
            Scope::Country
        );
        // Empty or fully transparent stacks fall back to Global.
        assert_eq!(Scope::pick_inferred_scope(&[]), Scope::Global);
        assert_eq!(
            Scope::pick_inferred_scope(&[Scope::ModifierBag, Scope::Unknown]),
            Scope::Global
        );
    }
}
