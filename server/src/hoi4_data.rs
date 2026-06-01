use serde::Deserialize;
use std::collections::HashMap;

use once_cell::sync::Lazy;

/// A game-defined trigger, effect, or modifier with its metadata.
#[derive(Clone, Deserialize)]
pub struct HOI4Entity {
    pub name: String,
    pub description: String,
    pub scopes: Vec<crate::scope::Scope>,
}

#[derive(Deserialize)]
struct AllData {
    triggers: HashMap<String, HOI4Entity>,
    effects: HashMap<String, HOI4Entity>,
    modifiers: HashMap<String, HOI4Entity>,
}

static DATA: Lazy<AllData> = Lazy::new(|| {
    serde_json::from_str(include_str!("../assets/hoi4_data.json"))
        .expect("Failed to parse hoi4_data.json — file is malformed or missing")
});

pub fn get_triggers() -> HashMap<String, HOI4Entity> {
    DATA.triggers.clone()
}

pub fn get_effects() -> HashMap<String, HOI4Entity> {
    DATA.effects.clone()
}

pub fn get_modifiers() -> HashMap<String, HOI4Entity> {
    DATA.modifiers.clone()
}

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
        "GetTrendingSideName",
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
