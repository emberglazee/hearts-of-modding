use crate::scope::scope::Scope;
use serde::Deserialize;
use std::collections::HashMap;

use once_cell::sync::Lazy;

/// The usage restriction context for scopes
#[derive(Debug, Clone, Deserialize)]
pub struct ScopeUsage {
    pub usage: Vec<Scope>,
    #[allow(dead_code)]
    #[serde(default)]
    pub usage_restriction: String,
}

impl ScopeUsage {
    /// Check if this entity can be used in the given scope
    pub fn allows(&self, scope: &Scope) -> bool {
        self.usage.contains(scope) || self.usage.contains(&Scope::Global)
    }

    /// Check if the usage list contains a specific scope
    pub fn contains(&self, scope: &Scope) -> bool {
        self.usage.contains(scope)
    }
}

/// A parameter definition for a structured block (e.g. the `idea = X`,
/// `days = 180` sub-keys of `add_timed_idea`). Documented per-entity in the
/// JSON so sub-keys are not global keywords: a `days` key inside
/// `add_timed_idea` is this block's parameter, not a generic modifier.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ParameterDef {
    /// Base value kind: `string`, `int`, `float`, `bool`, ... (from `<type>`
    /// placeholders in the wiki docs). Loose, not an enum — docs vary.
    #[serde(rename = "type")]
    pub param_type: String,
    /// Cross-reference kind when the value points at a scanner entity
    /// (`idea`, `country`, `equipment`, `character`, `trait`, `state`, ...).
    /// Empty string for plain values. Powers goto-definition / value
    /// completion / hover cross-links for reference-typed parameters.
    #[serde(default)]
    pub value_type: String,
    #[serde(default)]
    pub description: String,
    /// Best-effort from docs ("optional" / "mandatory" wording); default
    /// false = not known to be optional. Display hint only.
    #[serde(default)]
    pub optional: bool,
    /// Key may legitimately appear multiple times in the block (e.g.
    /// `tooltip`, `custom_effect_tooltip`, `add_idea`).
    #[serde(default)]
    pub repeated: bool,
}

/// How a block behaves on the scope stack
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
pub enum StackBehaviour {
    /// Pushes a new scope onto the stack (e.g., every_state, any_country)
    #[serde(rename = "push")]
    Push,
    /// Keeps the current scope (e.g., has_stability, add_manpower)
    #[serde(rename = "passthrough")]
    #[default]
    Passthrough,
    /// Transparent block that passes parent scope through (e.g., AND, OR, limit, if)
    #[serde(rename = "transparent")]
    Transparent,
}

/// The type of a trigger/effect/modifier block
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum BlockType {
    #[serde(rename = "trigger_scope")]
    TriggerScope,
    #[serde(rename = "value_trigger")]
    ValueTrigger,
    #[serde(rename = "effect_scope")]
    EffectScope,
    #[serde(rename = "value_effect")]
    ValueEffect,
    #[serde(rename = "modifier")]
    Modifier,
    #[serde(rename = "flow_control")]
    FlowControl,
    #[serde(rename = "dual_scope")]
    DualScope,
    #[serde(rename = "idea_property")]
    IdeaProperty,
    #[serde(rename = "array_scope")]
    ArrayScope,
    #[serde(rename = "scripted_trigger")]
    ScriptedTrigger,
    #[serde(rename = "scripted_effect")]
    ScriptedEffect,
}

fn default_block_type() -> BlockType {
    BlockType::ValueEffect
}

/// Custom deserializer for `scopes` that handles both V1 (list of Scope) and V2 (ScopeUsage) formats.
fn deserialize_scopes_v1_v2<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<ScopeUsage, D::Error> {
    #[derive(Deserialize)]
    struct ScopeUsageHelper {
        usage: Vec<Scope>,
        #[serde(default)]
        usage_restriction: String,
    }

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ScopesFormat {
        V1(Vec<Scope>),
        V2(ScopeUsageHelper),
    }

    match ScopesFormat::deserialize(d)? {
        ScopesFormat::V1(scopes) => Ok(ScopeUsage {
            usage: scopes,
            usage_restriction: String::new(),
        }),
        ScopesFormat::V2(helper) => Ok(ScopeUsage {
            usage: helper.usage,
            usage_restriction: helper.usage_restriction,
        }),
    }
}

/// A game-defined trigger, effect, or modifier with its metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct HOI4Entity {
    pub name: String,
    #[allow(dead_code)]
    #[serde(default)]
    pub description: String,
    /// Backward-compat: deserializes from either Vec<Scope> (V1) or ScopeUsage (V2)
    #[serde(deserialize_with = "deserialize_scopes_v1_v2")]
    pub scopes: ScopeUsage,
    #[serde(default)]
    pub pushes_scope: Option<Scope>,
    #[allow(dead_code)]
    #[serde(default)]
    pub parameters: HashMap<String, ParameterDef>,
    #[allow(dead_code)]
    #[serde(default)]
    pub examples: Vec<String>,
    #[allow(dead_code)]
    #[serde(default)]
    pub stack_behaviour: StackBehaviour,
    #[allow(dead_code)]
    #[serde(default = "default_block_type")]
    pub block_type: BlockType,
    #[allow(dead_code)]
    #[serde(default)]
    pub version_added: String,
    #[allow(dead_code)]
    #[serde(default)]
    pub notes: String,
    #[allow(dead_code)]
    #[serde(default)]
    pub vanilla_usage_count: u32,
    #[allow(dead_code)]
    #[serde(default)]
    pub deprecated: bool,
    /// True when this block's documented `parameters` describe the DIRECT
    /// sub-blocks nested inside it rather than scalar sub-keys of itself
    /// (e.g. `technology_folders` documents `ledger`/`doctrine` for the
    /// per-folder blocks it contains). Walkers thread this block key down as
    /// a parameter anchor so `ledger` inside `industry_folder = { ... }`
    /// resolves against `technology_folders`' parameter table; any deeper
    /// block (e.g. a folder's `available`) consumes the anchor.
    #[serde(default)]
    pub param_container: bool,
}

/// Scope chain target (for dot-notation resolution like ROOT.owner.capital)
#[derive(Debug, Clone, Deserialize)]
pub struct ChainTarget {
    pub scope: Scope,
    #[allow(dead_code)]
    #[serde(default)]
    pub restriction: String,
}

/// Information about a scope type and its chain targets
#[derive(Debug, Clone, Deserialize)]
pub struct ScopeInfo {
    #[allow(dead_code)]
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub chain_targets: HashMap<String, ChainTarget>,
}

/// A built-in dynamic variable (engine-provided, never needs `set_variable`).
///
/// Sourced from `documentation/dynamic_variables_documentation.md` via
/// `server/scripts/parse_dynamic_variables.py` → `dynamic_variables` in
/// `hoi4_data_v2.json`. `is_array` is true when the doc description
/// contains "array" (e.g. `faction_members`, `allies`, `owned_states`);
/// scalars like `stability` are `is_array: false`.
#[derive(Debug, Clone, Deserialize)]
pub struct DynamicVariable {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    #[serde(default)]
    pub description: String,
    #[allow(dead_code)]
    #[serde(deserialize_with = "deserialize_scopes_v1_v2")]
    pub scopes: ScopeUsage,
    #[allow(dead_code)]
    #[serde(default)]
    pub is_array: bool,
}

/// All data loaded from the V2 JSON file
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct AllDataV2 {
    pub version: u32,
    pub triggers: HashMap<String, HOI4Entity>,
    pub effects: HashMap<String, HOI4Entity>,
    pub modifiers: HashMap<String, HOI4Entity>,
    #[serde(default)]
    pub scopes: HashMap<String, ScopeInfo>,
    #[serde(default)]
    pub transparent_block_types: Vec<String>,
    #[serde(default)]
    pub dynamic_variables: HashMap<String, DynamicVariable>,
}

static DATA: Lazy<AllDataV2> = Lazy::new(|| {
    // build.rs minifies assets/hoi4_data_v2.json into OUT_DIR at compile
    // time; embed the minified copy so shipped binaries don't carry the
    // pretty-print whitespace (~144KB). Content is identical after the
    // serde_json round-trip.
    let bytes = include_str!(concat!(env!("OUT_DIR"), "/hoi4_data_v2.min.json"));
    serde_json::from_str(bytes)
        .expect("Failed to parse hoi4_data_v2.json — file is malformed or missing")
});

/// Get a reference to the static triggers map
pub fn get_triggers() -> &'static HashMap<String, HOI4Entity> {
    &DATA.triggers
}

/// Get a reference to the static effects map
pub fn get_effects() -> &'static HashMap<String, HOI4Entity> {
    &DATA.effects
}

/// Get a reference to the static modifiers map
pub fn get_modifiers() -> &'static HashMap<String, HOI4Entity> {
    &DATA.modifiers
}

/// Get scope info (chain targets, descriptions)
#[allow(dead_code)]
pub fn get_scope_info() -> &'static HashMap<String, ScopeInfo> {
    &DATA.scopes
}

/// Get transparent block type names
pub fn get_transparent_block_types() -> &'static [String] {
    &DATA.transparent_block_types
}

/// Look up what scope a keyword pushes (if it has a pushes_scope)
pub fn lookup_pushes_scope(key: &str) -> Option<Scope> {
    // DB keys are lowercase in JSON; HOI4 files can write them in any case, so
    // fall back to a case-insensitive lookup (mirrors lookup_chain_target). The
    // exact hit short-circuits, so the lowercase fallback only allocates when
    // the raw key isn't found.
    // Check triggers first
    if let Some(entity) = DATA
        .triggers
        .get(key)
        .or_else(|| DATA.triggers.get(&key.to_ascii_lowercase()))
    {
        return entity.pushes_scope;
    }
    // Then effects
    if let Some(entity) = DATA
        .effects
        .get(key)
        .or_else(|| DATA.effects.get(&key.to_ascii_lowercase()))
    {
        return entity.pushes_scope;
    }
    None
}

/// Check if a keyword is a known trigger, effect, or modifier
#[allow(dead_code)]
pub fn is_known_entity(key: &str) -> bool {
    DATA.triggers.contains_key(key)
        || DATA.effects.contains_key(key)
        || DATA.modifiers.contains_key(key)
}

/// Look up a scope chain target — returns ChainTarget scoped to the static DATA
pub fn lookup_chain_target(from_scope: &Scope, target_name: &str) -> Option<&'static ChainTarget> {
    let scope_str = from_scope.as_str();
    let info = DATA.scopes.get(scope_str)?;
    // Chain target keys are lowercase in JSON; HOI4 files can use uppercase (e.g. OWNER)
    let lower = target_name.to_ascii_lowercase();
    info.chain_targets.get(&lower)
}

/// Look up the entity (trigger, effect, or modifier) by key, case-insensitively.
///
/// DB keys are lowercase in the JSON; HOI4 files can write them in any case.
/// The exact hit short-circuits so the lowercase fallback only allocates when
/// the raw key isn't found. Checks triggers first, then effects, then
/// modifiers (mirrors [`lookup_pushes_scope`]).
pub fn lookup_entity(key: &str) -> Option<&'static HOI4Entity> {
    if let Some(entity) = DATA
        .triggers
        .get(key)
        .or_else(|| DATA.triggers.get(&key.to_ascii_lowercase()))
    {
        return Some(entity);
    }
    if let Some(entity) = DATA
        .effects
        .get(key)
        .or_else(|| DATA.effects.get(&key.to_ascii_lowercase()))
    {
        return Some(entity);
    }
    if let Some(entity) = DATA
        .modifiers
        .get(key)
        .or_else(|| DATA.modifiers.get(&key.to_ascii_lowercase()))
    {
        return Some(entity);
    }
    None
}

/// Look up a documented parameter (sub-key) of a structured entity block.
///
/// `entity_key` is the block's key (e.g. `add_timed_idea`), `param` the
/// sub-key inside it (e.g. `days`). Returns the parameter's definition when
/// the entity documents it, `None` otherwise. Both lookups are
/// case-insensitive.
pub fn lookup_parameter(entity_key: &str, param: &str) -> Option<&'static ParameterDef> {
    let entity = lookup_entity(entity_key)?;
    if let Some(p) = entity.parameters.get(param) {
        return Some(p);
    }
    entity.parameters.get(&param.to_ascii_lowercase())
}

/// Iterate the documented parameters of an entity block, if any.
pub fn entity_parameters(entity_key: &str) -> Option<&'static HashMap<String, ParameterDef>> {
    lookup_entity(entity_key).map(|e| &e.parameters)
}

/// True when `entity_key` documents its DIRECT sub-blocks in `parameters`
/// (see [`HOI4Entity::param_container`]). Only such blocks anchor parameter
/// resolution for the blocks nested inside them.
pub fn is_param_container(entity_key: &str) -> bool {
    lookup_entity(entity_key).is_some_and(|e| e.param_container)
}

/// Resolve a key against the immediate parent first, then a threaded
/// param-container anchor. Shared by semantic tokens, hover, and completion
/// so a folder's `ledger`/`doctrine` classify identically everywhere.
pub fn lookup_parameter_with_anchor<'a>(
    parent: Option<&'a str>,
    anchor: Option<&'a str>,
    key: &str,
) -> Option<(&'a str, &'static ParameterDef)> {
    if let Some(p) = parent {
        if let Some(def) = lookup_parameter(p, key) {
            return Some((p, def));
        }
    }
    let a = anchor?;
    let def = lookup_parameter(a, key)?;
    Some((a, def))
}

/// Check if a keyword is a transparent block type
pub fn is_transparent_block(key: &str) -> bool {
    DATA.transparent_block_types
        .iter()
        .any(|t| t.eq_ignore_ascii_case(key))
}

/// Get all built-in dynamic variables (383 entries from the official docs).
///
/// Keys are lowercase base names (e.g. `faction_members`). Use the
/// helpers below rather than hashing directly — they strip scope prefixes
/// (`ROOT.faction_members` → `faction_members`) and are case-insensitive.
#[allow(dead_code)]
pub fn get_dynamic_variables() -> &'static HashMap<String, DynamicVariable> {
    &DATA.dynamic_variables
}

/// Lookup a dynamic variable by name, stripping a scope prefix if present
/// (`ROOT.faction_members` → `faction_members`) and case-insensitively.
pub fn lookup_dynamic_variable(name: &str) -> Option<&'static DynamicVariable> {
    let base = name.rsplit('.').next().unwrap_or(name);
    // DB keys are lowercase; fast path exact then fallback lowercase
    if let Some(v) = DATA.dynamic_variables.get(base) {
        return Some(v);
    }
    DATA.dynamic_variables.get(&base.to_ascii_lowercase())
}

/// True when `name` names a built-in *array* dynamic variable
/// (e.g. `faction_members`, `allies`, `owned_states`, `countries`).
pub fn is_builtin_array(name: &str) -> bool {
    lookup_dynamic_variable(name).is_some_and(|v| v.is_array)
}

/// True when `name` names any built-in dynamic variable (scalar or array).
/// Built-in scalars (e.g. `stability`) are always defined by the engine and
/// should not be flagged as undefined by `HOM9001` variable checks.
pub fn is_builtin_variable(name: &str) -> bool {
    lookup_dynamic_variable(name).is_some()
}

/// UTF-8 BOM as raw bytes.
pub const UTF8_BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];

/// True when `bytes` starts with EXACTLY ONE UTF-8 BOM — the only correct
/// form for HOI4 localization files (all 2073 vanilla files carry precisely
/// one). Zero BOMs means the game may not load the file's strings; two or
/// more put stray U+FEFF characters after the header position that corrupt
/// line 1. Must be checked on RAW bytes: once decoded to a String, a leading
/// BOM is stripped by parsers/editors and extra ones are zero-width, so the
/// problem is invisible in any text form.
pub fn has_exactly_one_bom(bytes: &[u8]) -> bool {
    bytes.starts_with(&UTF8_BOM) && !bytes[3..].starts_with(&UTF8_BOM)
}

/// Get the V2 data version
#[allow(dead_code)]
pub fn get_version() -> u32 {
    DATA.version
}

/// Legacy scope list — used for hover completion of scope keywords
pub fn get_scopes() -> Vec<&'static str> {
    vec![
        "ROOT",
        "PREV",
        "THIS",
        "FROM",
        "FROM.FROM",
        "FROM.FROM.FROM",
        "FROM.FROM.FROM.FROM",
        "GER",
        "ENG",
        "FRA",
        "ITA",
        "JAP",
        "SOV",
        "USA",
    ]
}

/// Legacy loc commands list
pub fn get_loc_commands() -> Vec<&'static str> {
    vec![
        "GetName",
        "GetNameDef",
        "GetNameDefCap",
        "GetAdjective",
        "GetAdjectiveCap",
        "GetTag",
        "GetRulingIdeology",
        "GetRulingIdeologyNoun",
        "GetPartyName",
        "GetPartySupport",
        "GetLeaderName",
        "GetLeaderNameDef",
        "GetPlayerName",
        "GetCapitalName",
        "GetLastElection",
        "GetRulingParty",
        "GetRulingPartyLong",
        "GetCommunistParty",
        "GetDemocraticParty",
        "GetFascistParty",
        "GetNeutralParty",
        "GetCommunistLeader",
        "GetDemocraticLeader",
        "GetFascistLeader",
        "GetNeutralLeader",
        "GetPowerBalanceName",
        "GetPowerBalanceModDesc",
        "GetRightSideName",
        "GetLeftSideName",
        "GetActiveSideName",
        "GetActiveRangeName",
        "GetActiveRangeModDesc",
        "GetActiveRangeRuleDesc",
        "GetActiveRangeActivationEffect",
        "GetActiveRangeDeactivationEffect",
        "GetChangeRateDesc",
        "GetBopTrendTextIcon",
        "GetSheHe",
        "GetSheHeCap",
        "GetHerHim",
        "GetHerHimCap",
        "GetHerHis",
        "GetHerHisCap",
        "GetHersHis",
        "GetHersHisCap",
        "GetHerselfHimself",
        "GetHerselfHimselfCap",
        "GetIdeology",
        "GetIdeologyGroup",
        "GetRank",
        "GetCodeName",
        "GetCallsign",
        "GetSurname",
        "GetFullName",
        "GetWing",
        "GetWingShort",
        "GetAceType",
        "GetMissionRegion",
        "GetTokenKey",
        "GetTokenLocalizedKey",
        "GetDateString",
        "GetDateStringShortMonth",
        "GetDateStringNoHour",
        "GetDateStringNoHourLong",
        "GetManpower",
        "GetFactionName",
        "GetAgency",
        "GetNameWithFlag",
        "GetFlag",
        "GetDate",
        "GetTime",
        "GetYear",
        "GetMonth",
        "GetDay",
        "GetID",
        "GetCapitalVictoryPointName",
        "GetOldName",
        "GetOldNameDef",
        "GetOldNameDefCap",
        "GetOldAdjective",
        "GetOldAdjectiveCap",
        "GetNonIdeologyName",
        "GetNonIdeologyNameDef",
        "GetNonIdeologyNameDefCap",
        "GetNonIdeologyAdjective",
        "GetNonIdeologyAdjectiveCap",
        "GetLeader",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v2_json_loads() {
        assert!(!get_triggers().is_empty(), "Triggers should not be empty");
        assert!(!get_effects().is_empty(), "Effects should not be empty");
        assert!(!get_modifiers().is_empty(), "Modifiers should not be empty");
        assert!(get_version() >= 2, "Version should be 2 or higher");
    }

    #[test]
    fn test_allows_scope() {
        let usage = ScopeUsage {
            usage: vec![Scope::Country],
            usage_restriction: String::new(),
        };
        assert!(usage.allows(&Scope::Country));
        assert!(!usage.allows(&Scope::State));
    }

    #[test]
    fn test_contains_scope() {
        let usage = ScopeUsage {
            usage: vec![Scope::Country, Scope::State],
            usage_restriction: String::new(),
        };
        assert!(usage.contains(&Scope::Country));
        assert!(!usage.contains(&Scope::Unit));
    }

    #[test]
    fn test_v1_v2_deser() {
        // V1 format (list of scopes)
        let v1_json = r#"{"name": "test", "scopes": ["Country", "State"]}"#;
        let entity: HOI4Entity = serde_json::from_str(v1_json).unwrap();
        assert_eq!(entity.scopes.usage.len(), 2);
        assert!(entity.scopes.allows(&Scope::Country));
        assert!(!entity.scopes.allows(&Scope::Unit));

        // V2 format (struct with usage)
        let v2_json = r#"{"name": "test", "scopes": {"usage": ["Country", "Character"], "usage_restriction": "test"}}"#;
        let entity: HOI4Entity = serde_json::from_str(v2_json).unwrap();
        assert_eq!(entity.scopes.usage.len(), 2);
        assert!(entity.scopes.allows(&Scope::Country));
        assert_eq!(entity.scopes.usage_restriction, "test");
    }

    #[test]
    fn test_lookup_pushes_scope() {
        // any_country is a scope-pusher: push Country scope
        let result = lookup_pushes_scope("any_country");
        assert_eq!(
            result,
            Some(Scope::Country),
            "any_country should push Country scope"
        );

        // every_state pushes State scope
        let result = lookup_pushes_scope("every_state");
        assert_eq!(
            result,
            Some(Scope::State),
            "every_state should push State scope"
        );

        // every_country pushes Country scope (iteration target)
        let result = lookup_pushes_scope("every_country");
        assert_eq!(
            result,
            Some(Scope::Country),
            "every_country should push Country scope"
        );

        // Unknown entity has no pushes_scope
        let result = lookup_pushes_scope("nonexistent_trigger_xyz");
        assert!(result.is_none());

        // Case-insensitive: DB keys are lowercase, files may write any case
        // (mirrors lookup_chain_target). Uppercase must resolve identically.
        assert_eq!(
            lookup_pushes_scope("every_country"),
            lookup_pushes_scope("EVERY_COUNTRY"),
            "lookup_pushes_scope must be case-insensitive"
        );
        assert_eq!(lookup_pushes_scope("ALL_CORE_STATE"), Some(Scope::State));
    }

    #[test]
    fn test_is_known_entity() {
        assert!(is_known_entity("has_government"));
        assert!(is_known_entity("add_ideas"));
        assert!(!is_known_entity("definitely_not_a_real_trigger_xyz123"));
    }

    #[test]
    fn test_lookup_entity_and_parameters() {
        // Entity lookup is case-insensitive and covers all three families.
        assert!(lookup_entity("add_ideas").is_some());
        assert!(lookup_entity("ADD_IDEAS").is_some(), "case-insensitive");
        assert!(lookup_entity("has_government").is_some());
        assert!(lookup_entity("army_attack_factor").is_some(), "modifiers");
        assert!(lookup_entity("not_a_real_entity_xyz").is_none());

        // add_timed_idea documents idea/days/months/years (populated from the
        // wiki docs by server/scripts/parse_wiki_parameters.py).
        let entity = lookup_entity("add_timed_idea").expect("add_timed_idea in DB");
        let params = &entity.parameters;
        assert!(
            params.contains_key("idea") && params.contains_key("days"),
            "add_timed_idea should document idea + days, got: {:?}",
            params.keys().collect::<Vec<_>>()
        );

        // lookup_parameter: case-insensitive on both the entity and the param.
        let days = lookup_parameter("add_timed_idea", "days").expect("days param");
        assert!(
            !days.description.is_empty(),
            "days should have a description"
        );
        assert!(lookup_parameter("ADD_TIMED_IDEA", "DAYS").is_some());
        // Unknown param on a documented entity -> None.
        assert!(lookup_parameter("add_timed_idea", "bogus_key").is_none());
        // Unknown entity -> None.
        assert!(lookup_parameter("not_a_real_entity_xyz", "days").is_none());

        // entity_parameters returns the map for documented entities, an
        // (empty) map for undocumented ones — consumers check is_empty().
        assert!(entity_parameters("add_timed_idea").is_some());
        assert!(
            entity_parameters("add_timed_idea").is_some_and(|p| !p.is_empty()),
            "documented entity should have a non-empty parameters map"
        );
        assert!(
            entity_parameters("add_political_power").is_some_and(|p| p.is_empty()),
            "undocumented entity keeps an empty parameters map"
        );
    }

    /// The `parameters` map is a PARTIAL picture of what a block accepts — the
    /// wiki documents scalar sub-keys but not the nested blocks or the
    /// arbitrary effects/triggers that make up a block's body. Consumers must
    /// therefore treat it as additive (highlight/rank these keys) and never as
    /// an allow-list (offer only these keys / flag anything else).
    ///
    /// These assertions encode that contract against real data so a future
    /// "just return the params" refactor fails loudly here.
    #[test]
    fn test_parameters_are_partial_not_exhaustive() {
        // country_event documents the invocation form (id/days/hours) but NOT
        // the definition form's keys, which are far more common in practice.
        let ce = entity_parameters("country_event").expect("country_event documented");
        assert!(ce.contains_key("id"), "invocation form is documented");
        for definition_key in ["title", "desc", "picture", "option", "is_triggered_only"] {
            assert!(
                !ce.contains_key(definition_key),
                "`{definition_key}` is a real country_event key but is NOT in the \
                 parameters map — proof the map is partial and must stay additive"
            );
        }

        // `if` documents else/else_if/limit; its body holds arbitrary effects.
        let if_params = entity_parameters("if").expect("if documented");
        assert!(if_params.contains_key("limit"));
        for effect in ["set_country_flag", "country_event", "add_political_power"] {
            assert!(
                !if_params.contains_key(effect),
                "`{effect}` is legal inside `if = {{}}` but undocumented"
            );
        }
    }

    /// Guards the generator's validity filter: a parameter must never be named
    /// after its own block (`is_puppet = {{ is_puppet = ... }}` is nonsense),
    /// and no type may be pure punctuation noise (`""`, `???`).
    #[test]
    fn test_no_malformed_parameters_in_data() {
        let mut problems: Vec<String> = Vec::new();
        for (family, map) in [
            ("triggers", &DATA.triggers),
            ("effects", &DATA.effects),
            ("modifiers", &DATA.modifiers),
        ] {
            for (key, entity) in map {
                for (pname, pdef) in &entity.parameters {
                    if pname == key {
                        problems.push(format!("{family}:{key}.{pname} is self-referential"));
                    }
                    if !pname
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                    {
                        problems.push(format!("{family}:{key}.{pname} is not an identifier"));
                    }
                    let t = pdef.param_type.trim();
                    if !t.is_empty() && !t.chars().any(|c| c.is_ascii_alphanumeric()) {
                        problems.push(format!("{family}:{key}.{pname} has noise type {t:?}"));
                    }
                }
            }
        }
        assert!(
            problems.is_empty(),
            "malformed parameters in hoi4_data_v2.json (re-run \
             server/scripts/parse_wiki_parameters.py):\n{}",
            problems.join("\n")
        );
    }

    #[test]
    fn test_parameter_def_deser() {
        // Full shape written by the generator script.
        let json = r#"{
            "type": "int",
            "value_type": "number",
            "description": "The number of days to add the idea for.",
            "optional": false,
            "repeated": false
        }"#;
        let p: ParameterDef = serde_json::from_str(json).unwrap();
        assert_eq!(p.param_type, "int");
        assert_eq!(p.value_type, "number");
        assert!(!p.optional);
        assert!(!p.repeated);

        // Minimal shape (older entries / hand-written) — new fields default.
        let minimal: ParameterDef = serde_json::from_str(r#"{"type": "string"}"#).unwrap();
        assert_eq!(minimal.value_type, "");
        assert_eq!(minimal.description, "");
        assert!(!minimal.optional);
        assert!(!minimal.repeated);
    }

    #[test]
    fn test_get_version() {
        assert!(get_version() >= 2);
    }
}
