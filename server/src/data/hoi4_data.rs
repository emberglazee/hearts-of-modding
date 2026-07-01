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

/// A parameter definition for autocomplete
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ParameterDef {
    #[serde(rename = "type")]
    pub param_type: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub optional: bool,
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
}

static DATA: Lazy<AllDataV2> = Lazy::new(|| {
    let bytes = include_str!("../../assets/hoi4_data_v2.json");
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
    // Check triggers first
    if let Some(entity) = DATA.triggers.get(key) {
        return entity.pushes_scope;
    }
    // Then effects
    if let Some(entity) = DATA.effects.get(key) {
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

/// Check if a keyword is a transparent block type
pub fn is_transparent_block(key: &str) -> bool {
    DATA.transparent_block_types
        .iter()
        .any(|t| t.eq_ignore_ascii_case(key))
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
        // any_country is a known trigger in V2 data that should push Country scope
        let result = lookup_pushes_scope("any_country");
        assert!(
            result.is_some(),
            "any_country should have a pushes_scope in V2 data"
        );
        assert_eq!(result, Some(Scope::Country));
    }

    #[test]
    fn test_is_known_entity() {
        assert!(is_known_entity("has_government"));
        assert!(is_known_entity("add_ideas"));
        assert!(!is_known_entity("definitely_not_a_real_trigger_xyz123"));
    }

    #[test]
    fn test_get_version() {
        assert!(get_version() >= 2);
    }
}
