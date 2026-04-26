use std::collections::HashMap;

pub struct HOI4Entity {
    pub name: &'static str,
    pub description: &'static str,
}

pub fn get_triggers() -> HashMap<&'static str, HOI4Entity> {
    let mut m = HashMap::new();
    m.insert("tag", HOI4Entity {
        name: "tag",
        description: "Checks if the current country has the specified tag.\n\nExample: tag = GER",
    });
    m.insert("has_war", HOI4Entity {
        name: "has_war",
        description: "Checks if the current country is at war.",
    });
    m.insert("is_ai", HOI4Entity {
        name: "is_ai",
        description: "Checks if the current country is controlled by the AI.",
    });
    m.insert("always", HOI4Entity {
        name: "always",
        description: "Always returns true or false.\n\nExample: always = yes",
    });
    m.insert("has_dlc", HOI4Entity {
        name: "has_dlc",
        description: "Checks if the specified DLC is active.",
    });
    m.insert("difficulty", HOI4Entity {
        name: "difficulty",
        description: "Checks the game difficulty level.",
    });
    m.insert("is_historical_focus_on", HOI4Entity {
        name: "is_historical_focus_on",
        description: "Checks if historical AI focuses are enabled.",
    });
    m
}

pub fn get_effects() -> HashMap<&'static str, HOI4Entity> {
    let mut m = HashMap::new();
    m.insert("add_political_power", HOI4Entity {
        name: "add_political_power",
        description: "Adds or removes political power from the current country.\n\nExample: add_political_power = 100",
    });
    m.insert("add_stability", HOI4Entity {
        name: "add_stability",
        description: "Adds or removes stability from the current country.",
    });
    m.insert("add_war_support", HOI4Entity {
        name: "add_war_support",
        description: "Adds or removes war support from the current country.",
    });
    m.insert("set_country_flag", HOI4Entity {
        name: "set_country_flag",
        description: "Sets a country flag for the current country.",
    });
    m.insert("clr_country_flag", HOI4Entity {
        name: "clr_country_flag",
        description: "Clears a country flag from the current country.",
    });
    m.insert("add_manpower", HOI4Entity {
        name: "add_manpower",
        description: "Adds or removes manpower from the current country.",
    });
    m
}

pub fn get_scopes() -> Vec<&'static str> {
    vec![
        "ROOT", "PREV", "THIS", "FROM", "FROM.FROM", "FROM.FROM.FROM", "FROM.FROM.FROM.FROM",
        "GER", "ENG", "FRA", "ITA", "JAP", "SOV", "USA",
    ]
}

pub fn get_loc_commands() -> Vec<&'static str> {
    vec![
        "GetName", "GetNameDef", "GetAdjective", "GetAdjectiveCap", "GetTag",
        "GetRulingIdeology", "GetRulingIdeologyNoun", "GetPartyName",
        "GetLeaderName", "GetLeaderNameDef", "GetPlayerName",
        "GetCapitalName", "GetYear", "GetMonth", "GetDay",
        "GetFlag", "GetDate", "GetTime",
    ]
}