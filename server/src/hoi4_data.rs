use std::collections::HashMap;

pub struct HOI4Entity {
    pub name: &'static str,
    pub description: &'static str,
    pub scopes: &'static [crate::scope::Scope],
}

pub fn get_triggers() -> HashMap<&'static str, HOI4Entity> {
    let mut m = HashMap::new();
    m.insert(
        "all_country",
        HOI4Entity {
            name: "all_country",
            description: r#"Checks if all countries meet the triggers.

**Example:**
```paradox
`all_country = { … }`
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "any_country",
        HOI4Entity {
            name: "any_country",
            description: r#"Checks if any country meets the triggers.

**Example:**
```paradox
`any_country = { … }`
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("all_other_country", HOI4Entity {
        name: "all_other_country",
        description: r#"Checks if all countries other than the one where this scope is located meet the triggers.

**Example:**
```paradox
`all_other_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("any_other_country", HOI4Entity {
        name: "any_other_country",
        description: r#"Checks if any country other than the one where this scope is located meets the triggers.

**Example:**
```paradox
`any_other_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("all_country_with_original_tag", HOI4Entity {
        name: "all_country_with_original_tag",
        description: r#"Checks if all countries originating from the specified country, including the dynamic countries created for civil wars and other purposes, meet the triggers. `original_tag_to_check = TAG` is used to specify the original tag.

**Example:**
```paradox
all_country_with_original_tag = {
    original_tag_to_check = TAG  #required
    …                  #triggers to check
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("any_country_with_original_tag", HOI4Entity {
        name: "any_country_with_original_tag",
        description: r#"Checks if any country originating from the specified country, including the dynamic countries created for civil wars and other purposes, meets the triggers. `original_tag_to_check = TAG` is used to specify the original tag.

**Example:**
```paradox
any_country_with_original_tag = {
    original_tag_to_check = TAG  #required
    …                  #triggers to check
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("all_neighbor_country", HOI4Entity {
        name: "all_neighbor_country",
        description: r#"Checks if all countries that border the one where this scope is located meet the triggers.

**Example:**
```paradox
`all_neighbor_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("any_neighbor_country", HOI4Entity {
        name: "any_neighbor_country",
        description: r#"Checks if any country that borders the one where this scope is located meets the triggers.

**Example:**
```paradox
`any_neighbor_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("any_home_area_neighbor_country", HOI4Entity {
        name: "any_home_area_neighbor_country",
        description: r#"Checks if any country that borders the one where this scope is located, as well as being in its home area - meaning a direct land connection between the capitals of countries - meets the triggers.

**Example:**
```paradox
`any_home_area_neighbor_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("all_guaranteed_country", HOI4Entity {
        name: "all_guaranteed_country",
        description: r#"Checks if all countries that are guaranteed by the one where this scope is located meet the triggers.

**Example:**
```paradox
`all_guaranteed_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("any_guaranteed_country", HOI4Entity {
        name: "any_guaranteed_country",
        description: r#"Checks if any country that is guaranteed by the one where this scope is located meets the triggers.

**Example:**
```paradox
`any_guaranteed_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("all_allied_country", HOI4Entity {
        name: "all_allied_country",
        description: r#"Checks if all countries that are allied with the one where this scope is located - meaning that they are either a subject of the country, its overlord, or that they share a faction - meet the triggers. Does not include the country itself.

**Example:**
```paradox
`all_allied_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("any_allied_country", HOI4Entity {
        name: "any_allied_country",
        description: r#"Checks if any country that is allied with the one where this scope is located - meaning that they are either a subject of the country, its overlord, or that they share a faction - meets the triggers. Does not include the country itself.

**Example:**
```paradox
`any_allied_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("all_occupied_country", HOI4Entity {
        name: "all_occupied_country",
        description: r#"Checks if all countries that are occupied by the one where this scope is located - meaning that the occupied country has core states controlled by the occupier country - meet the triggers.

**Example:**
```paradox
`all_occupied_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("any_occupied_country", HOI4Entity {
        name: "any_occupied_country",
        description: r#"Checks if any country that is occupied by the one where this scope is located - meaning that the occupied country has core states controlled by the occupier country - meets the triggers.

**Example:**
```paradox
`any_occupied_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("all_enemy_country", HOI4Entity {
        name: "all_enemy_country",
        description: r#"Checks if all countries that are at war with the one where this scope is located meet the triggers.

**Example:**
```paradox
`all_enemy_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("any_enemy_country", HOI4Entity {
        name: "any_enemy_country",
        description: r#"Checks if any country that are at war with the one where this scope is located meets the triggers.

**Example:**
```paradox
`any_enemy_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("all_subject_countries", HOI4Entity {
        name: "all_subject_countries",
        description: r#"Checks if all countries that are a subject of the one where this scope is located meet the triggers. Notice the plural spelling in the scope.

**Example:**
```paradox
`all_subject_countries = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("any_subject_country", HOI4Entity {
        name: "any_subject_country",
        description: r#"Checks if any country that is a subject of the one where this scope is located meets the triggers.

**Example:**
```paradox
`any_subject_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("any_country_with_core", HOI4Entity {
        name: "any_country_with_core",
        description: r#"Checks if any country that has the current scope as a core state meets the triggers. **Does not have an equivalent for other effect/trigger scope types.**

**Example:**
```paradox
`any_country_with_core = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("all_country_of", HOI4Entity {
        name: "all_country_of",
        description: r#"Checks if all of the provided countries fulfill the specified triggers. The `target` supports script constants and `tooltip` supports bindable localization.

**Example:**
```paradox
all_country_of = {
	tooltip = my_loc # Optional bindable localization
	target = { SWE NOR FIN DEN ICE }
	has_defensive_war = yes
}

all_country_of = {
    tooltip = my_loc # Optional bindable localization
	target = constant:country_groups:nordics
	has_defensive_war = yes
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("any_country_of", HOI4Entity {
        name: "any_country_of",
        description: r#"Checks if any of the provided countries fulfills the specified triggers. The `target` supports script constants and `tooltip` supports bindable localization.

**Example:**
```paradox
any_country_of = {
	tooltip = my_loc # Optional bindable localization
	target = { SWE NOR FIN DEN ICE }
	has_defensive_war = yes
}

any_country_of = {
    tooltip = my_loc # Optional bindable localization
	target = constant:country_groups:nordics
	has_defensive_war = yes
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "all_state",
        HOI4Entity {
            name: "all_state",
            description: r#"Check if all states meet the triggers.

**Example:**
```paradox
`all_state = { … }`
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "any_state",
        HOI4Entity {
            name: "any_state",
            description: r#"Check if any state meets the triggers.

**Example:**
```paradox
`any_state = { … }`
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "any_state_in",
        HOI4Entity {
            name: "any_state_in",
            description: r#"Check if any state in the given category meets the trigger.

**Example:**
```paradox
any_state_in = {
  array = array_of_states  #required
    …                  #triggers to check
}
```

 Requires on of the following fields

```paradox
array = <array_of_states>
continent = <continent_name>
ai_area = <ai_area_name>
strategic_region = <strategic_region_number>
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("any_state_of", HOI4Entity {
        name: "any_state_of",
        description: r#"Checks if any of the provided states fulfills the specified triggers. The `target` supports script constants and `tooltip` supports bindable localization.

**Example:**
```paradox
any_state_of = {
	tooltip = my_loc # Optional bindable localization
	target = { 1 42 1992 }
	controller = {
		has_defensive_war = yes
	}
}

any_state_of = {
    tooltip = my_loc # Optional bindable localization
	target = constant:country_groups:nordics
	controller = {
		has_defensive_war = yes
	}
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("all_neighbor_state", HOI4Entity {
        name: "all_neighbor_state",
        description: r#"Check if all states that are neighbour to the one where this scope is located meet the triggers.

**Example:**
```paradox
`all_neighbor_state = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("any_neighbor_state", HOI4Entity {
        name: "any_neighbor_state",
        description: r#"Check if any state that is neighbour to the one where this scope is located meets the triggers.

**Example:**
```paradox
`any_neighbor_state = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("all_owned_state", HOI4Entity {
        name: "all_owned_state",
        description: r#"Check if all states that are owned by the country where this scope is located meet the triggers.

**Example:**
```paradox
`all_owned_state = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("any_owned_state", HOI4Entity {
        name: "any_owned_state",
        description: r#"Check if any state that is owned by the country where this scope is located meets the triggers.

**Example:**
```paradox
`any_owned_state = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("all_core_state", HOI4Entity {
        name: "all_core_state",
        description: r#"Check if any state that is cored by the country where this scope is located meets the triggers.

**Example:**
```paradox
`all_core_state = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("any_core_state", HOI4Entity {
        name: "any_core_state",
        description: r#"Check if all states that are cored by the country where this scope is located meet the triggers.

**Example:**
```paradox
`any_core_state = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("all_controlled_state", HOI4Entity {
        name: "all_controlled_state",
        description: r#"Check if all states that are controlled by the country where this scope is located meet the triggers.

**Example:**
```paradox
`all_controlled_state = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("any_controlled_state", HOI4Entity {
        name: "any_controlled_state",
        description: r#"Check if any state that is controlled by the country where this scope is located meets the triggers.

**Example:**
```paradox
`any_controlled_state = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("all_unit_leader", HOI4Entity {
        name: "all_unit_leader",
        description: r#"Checks if all unit leaders (corps commanders, field marshals, admirals) that are employed by the country where this scope is located meet the triggers.

**Example:**
```paradox
`all_unit_leader = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("any_unit_leader", HOI4Entity {
        name: "any_unit_leader",
        description: r#"Checks if any unit leader (corps commander, field marshal, admiral) that is employed by the country where this scope is located meets the triggers.

**Example:**
```paradox
`any_unit_leader = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("all_army_leader", HOI4Entity {
        name: "all_army_leader",
        description: r#"Checks if all army leaders that are employed by the country where this scope is located meet the triggers.

**Example:**
```paradox
`all_army_leader = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("any_army_leader", HOI4Entity {
        name: "any_army_leader",
        description: r#"Checks if any army leader that is employed by the country where this scope is located meets the triggers.

**Example:**
```paradox
`any_army_leader = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("all_navy_leader", HOI4Entity {
        name: "all_navy_leader",
        description: r#"Checks if all navy leaders that are employed by the country where this scope is located meet the triggers.

**Example:**
```paradox
`all_navy_leader = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("any_navy_leader", HOI4Entity {
        name: "any_navy_leader",
        description: r#"Checks if any navy leader that is employed by the country where this scope is located meets the triggers.

**Example:**
```paradox
`any_navy_leader = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("all_operative_leader", HOI4Entity {
        name: "all_operative_leader",
        description: r#"Checks if all operatives that are employed by the country where this scope is located meet the triggers.

**Example:**
```paradox
`all_operative_leader = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("any_operative_leader", HOI4Entity {
        name: "any_operative_leader",
        description: r#"Checks if any operative that is employed by the country where this scope is located meets the triggers.

**Example:**
```paradox
`any_operative_leader = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("all_character", HOI4Entity {
        name: "all_character",
        description: r#"Checks if all characters that are recruited by the country where this scope is located meet the triggers.

**Example:**
```paradox
`all_character = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("any_character", HOI4Entity {
        name: "any_character",
        description: r#"Checks if any character that is recruited by the country where this scope is located meets the triggers.

**Example:**
```paradox
`any_character = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "any_country_division",
        HOI4Entity {
            name: "any_country_division",
            description: r#"Checks if any division owned by the current country meets the triggers.

**Example:**
```paradox
`any_country_division = { … }`
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "any_state_division",
        HOI4Entity {
            name: "any_state_division",
            description: r#"Checks if any division within the current state meets the triggers.

**Example:**
```paradox
`any_state_division = { … }`
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "all_military_industrial_organization",
        HOI4Entity {
            name: "all_military_industrial_organization",
            description: r#"Checks if all MIOs within the current country meet the conditions.

**Example:**
```paradox
`all_military_industrial_organization = { … }`
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "any_military_industrial_organization",
        HOI4Entity {
            name: "any_military_industrial_organization",
            description: r#"Checks if any MIO within the current country meets the conditions.

**Example:**
```paradox
`any_military_industrial_organization = { … }`
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("all_purchase_contract", HOI4Entity {
        name: "all_purchase_contract",
        description: r#"Checks if all purchase contracts within the current country meet the conditions.

**Example:**
```paradox
`all_purchase_contract = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("any_purchase_contract", HOI4Entity {
        name: "any_purchase_contract",
        description: r#"Checks if any purchase contract within the current country meets the conditions.

**Example:**
```paradox
`any_purchase_contract = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "all_scientists",
        HOI4Entity {
            name: "all_scientists",
            description: r#"Checks if all scientists of the Country in scope matches the triggers.

**Example:**
```paradox
`all_scientistst = { … }`
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("any_scientist", HOI4Entity {
        name: "any_scientist",
        description: r#"Checks if at least one active scientist of the Country in scope matches the triggers.

**Example:**
```paradox
`any_scientist = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("all_active_scientist", HOI4Entity {
        name: "all_active_scientist",
        description: r#"Checks if all active scientists of the Country in scope matches the triggers.

**Example:**
```paradox
`all_active_scientist = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("any_active_scientist", HOI4Entity {
        name: "any_active_scientist",
        description: r#"Checks if at least one active scientist of the Country in scope matches the triggers.

**Example:**
```paradox
`any_active_scientist = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("TAG", HOI4Entity {
        name: "TAG",
        description: r#"The country defined by the tag or tag alias. Tag aliases are defined in /Hearts of Iron IV/common/country_tag_aliases, as a way to refer to a specific country (such as a side in a civil war) in addition to its actual tag. If the country with the exact tag doesn't exist, but a dynamic country originating from the specified tag does, the scope will refer to the dynamic country.

**Example:**
```paradox
`SOV = { country_event = my_event.1 }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("ROOT", HOI4Entity {
        name: "ROOT",
        description: r#"Targets the root node of the block, an inherent property of each block. Most commonly, this is the default scope: for example, ROOT within a national focus will always refer to the country doing the focus and ROOT within a event will always refer to the country getting the event. However, some blocks do distinguish between the default scope and ROOT, such as certain scripted GUI contexts or certain on actions. If a block doesn't have ROOT defined (such as on_startup in on actions), then it is impossible to use it.

**Example:**
```paradox
ENG = {
    FRA = {
        GER = {
            declare_war_on = {
                target = ROOT
                type = annex_everything
            }
        }
    }
} #GER declares war on ENG (if there is no scope before ENG)
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("THIS", HOI4Entity {
        name: "THIS",
        description: r#"Targets the current scope where it's used. For example, when used in every_state, it will refer to the state that's currently being evaluated. Primarily useful for variables (as in the example, where omitting it wouldn't work) or for built-in localisation commands, where some scope must be specified. More rarely, this may help with scope manipulation when using PREV. Since omitting it makes no difference in how the code gets interpreted, there is little to no usage outside of these cases.

**Example:**
```paradox
set_temp_variable = { target_country = THIS }
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("PREV", HOI4Entity {
        name: "PREV",
        description: r#"Targets the scope that the current scope is contained in. Can have additional applications where the assumed default scope differs from the ROOT, such as in state events or some on_actions. Can be chained indefinitely as PREV.PREV. **Commonly results in broken-looking tooltips**: what's shown to the player doesn't always correlate with reality.

See also: PREV usage.

**Example:**
```paradox
FRA = {
    random_country = {
        GER = {
            declare_war_on = {
                target = PREV
                type = annex_everything
            }
        }
    }
} #Germany declares war on random_country
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("FROM", HOI4Entity {
        name: "FROM",
        description: r#"Can be chained indefinitely as FROM.FROM. Used to target various hardcoded scopes inherent to the block, often a secondary scope in addition to ROOT. For example:

In events, this refers to the country that sent the event (i.e. if the event was fired using an effect, then it's the ROOT scope where it was fired).
 In targeted decisions or diplomacy scripted triggers, this refers to the scope that is targeted.

**Example:**
```paradox
declare_war_on = {
    target = FROM
    type = annex_everything
}

FROM = {
    load_oob = defend_ourselves
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("overlord", HOI4Entity {
        name: "overlord",
        description: r#"The overlord of the country if it is a subject. Subject to the 'invalid event target' error.

**Example:**
```paradox
`overlord = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("faction_leader", HOI4Entity {
        name: "faction_leader",
        description: r#"Faction leader of the faction the country is a part of. Subject to the 'invalid event target' error.

**Example:**
```paradox
`faction_leader = { add_to_faction = FROM }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("owner", HOI4Entity {
        name: "owner",
        description: r#"In state scope, the country that owns the state. In combatant scope, the country that owns the divisions. In character scope, the country that has recruited the character. Subject to the 'invalid event target' error when used for a state.

**Example:**
```paradox
`owner = { add_ideas = owns_this_state }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("controller", HOI4Entity {
        name: "controller",
        description: r#"The controller of the current state. Subject to the 'invalid event target' error.

**Example:**
```paradox
controller = {
    ROOT = {
        create_wargoal = {
            target = PREV
            type = take_state_focus
            generator = { 123 }
        }
    }
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("capital_scope", HOI4Entity {
        name: "capital_scope",
        description: r#"The state where the capital of the current country is located in. Subject to the 'invalid event target' error in rare cases.

**Example:**
```paradox
`capital_scope = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("AND", HOI4Entity {
        name: "AND",
        description: r#"Returns false if any sub-trigger returns false, true otherwise. Evaluation stops at the first false sub-trigger.

**Example:**
```paradox
AND = {
    original_tag = GER
    has_stability > 0.5
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("OR", HOI4Entity {
        name: "OR",
        description: r#"Returns true if any sub-trigger returns true, false otherwise. Evaluation stops at the first true sub-trigger.

**Example:**
```paradox
OR = {
    original_tag = ENG
    original_tag = USA
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("NOT", HOI4Entity {
        name: "NOT",
        description: r#"Returns false if any sub-trigger returns true, true otherwise. Evaluation stops at the first true sub-trigger.

**Example:**
```paradox
NOT = {
    has_stability > 0.5
    has_war_support > 0.5
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("count_triggers", HOI4Entity {
        name: "count_triggers",
        description: r#"Sums the results of all sub-triggers (false=0, true=1) and returns true if the sum is at least `amount`.

**Example:**
```paradox
count_triggers = {
    amount = 2
    10 = { state_population = 100000 }
    11 = { state_population = 100000 }
    12 = { state_population = 100000 }
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("if", HOI4Entity {
        name: "if",
        description: r#"If `limit` is true, the sub-triggers are evaluated like an AND-trigger. If `limit` is false, `else_if` blocks are tried in sequence and finally `else` (if present). *Otherwise true is returned*.

**Example:**
```paradox
if = {
    limit = {
        has_dlc = "Poland: United and Ready"
    }
    has_political_power > 100
}
else_if = {
    limit = {
        has_dlc = "Waking the Tiger"
    }
    has_war_support > 0.5
}
else = {
    always = no
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "hidden_trigger",
        HOI4Entity {
            name: "hidden_trigger",
            description: r#"Hides the triggers from the tooltip shown to the player.

**Example:**
```paradox
hidden_trigger = {
    country_exists = GER
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("custom_trigger_tooltip", HOI4Entity {
        name: "custom_trigger_tooltip",
        description: r#"Hides the triggers from the tooltip shown to the player and instead uses the specified localisation key.

**Example:**
```paradox
custom_trigger_tooltip = {
    tooltip = sunrise_invasion_tt		
    any_state = {
        is_owned_by = JAP
        is_on_continent = europe
        is_coastal = yes
    }
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "custom_override_tooltip",
        HOI4Entity {
            name: "custom_override_tooltip",
            description: r#"An AND trigger that has an overriden custom tooltip.

**Example:**
```paradox
custom_override_tooltip = {
    tooltip = {
      localization_key = GER_inner_circle_focus_in_progress_tt
	  CHARACTER = GER_rudolf_hess
	  FLAG_DAYS = [?GER_rally_the_industrialists_in_progress_flag:days]
    }
    not_tooltip = MY_TOOLTIP_NOT
    <triggers>
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "always",
        HOI4Entity {
            name: "always",
            description: r#"Always returns true or false. Useful for debugging.

**Example:**
```paradox
always = yes
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "has_global_flag",
        HOI4Entity {
            name: "has_global_flag",
            description: r#"Checks if the specified flag has been set.

**Example:**
```paradox
has_global_flag = my_flag
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "has_dlc",
        HOI4Entity {
            name: "has_dlc",
            description: r#"Checks if the specified DLC is enabled.

**Example:**
```paradox
has_dlc = "Waking the Tiger"
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("has_start_date", HOI4Entity {
        name: "has_start_date",
        description: r#"Checks if the specified date was the start date used for the current game.

**Example:**
```paradox
has_start_date > 1950.01.01
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "date",
        HOI4Entity {
            name: "date",
            description: r#"Checks if the specified date against the current date.

**Example:**
```paradox
date < 1950.01.01
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "difficulty",
        HOI4Entity {
            name: "difficulty",
            description: r#"checks if the specified difficulty against the current difficulty.

**Example:**
```paradox
difficulty > 0
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("has_any_custom_difficulty_setting", HOI4Entity {
        name: "has_any_custom_difficulty_setting",
        description: r#"Checks if any custom difficulty setting is changed from their default value.

**Example:**
```paradox
has_any_custom_difficulty_setting = yes
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("has_custom_difficulty_setting", HOI4Entity {
        name: "has_custom_difficulty_setting",
        description: r#"Checks if the specified custom difficulty setting is changed from the default value.

**Example:**
```paradox
has_custom_difficulty_setting = custom_diff_strong_sov
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "game_rules_allow_achievements",
        HOI4Entity {
            name: "game_rules_allow_achievements",
            description: r#"Checks if all of the active game rule options allow achievements.

**Example:**
```paradox
game_rules_allow_achievements = yes
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "country_exists",
        HOI4Entity {
            name: "country_exists",
            description: r#"Checks if the specified country currently exists in game.

**Example:**
```paradox
country_exists = GER
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "is_ironman",
        HOI4Entity {
            name: "is_ironman",
            description: r#"Checks if the current game is running in Ironman mode.

**Example:**
```paradox
is_ironman = yes
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "is_historical_focus_on",
        HOI4Entity {
            name: "is_historical_focus_on",
            description: r#"Checks if the current game is running with Historical Focuses on.

**Example:**
```paradox
is_historical_focus_on = yes
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "is_tutorial",
        HOI4Entity {
            name: "is_tutorial",
            description: r#"Checks if the current game is running in Tutorial mode.

**Example:**
```paradox
is_tutorial = yes
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "is_debug",
        HOI4Entity {
            name: "is_debug",
            description: r#"Checks if game is in debug mode (launched with -debug argument).

**Example:**
```paradox
is_debug = yes
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "threat",
        HOI4Entity {
            name: "threat",
            description: r#"Checks if World Tension is above the specified amount.

**Example:**
```paradox
threat > 0.5
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "has_game_rule",
        HOI4Entity {
            name: "has_game_rule",
            description: r#"Checks if a game rule is set to a particular option.

**Example:**
```paradox
has_game_rule = { rule = GER_can_remilitarize_rhineland option = yes }
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("has_completed_custom_achievement", HOI4Entity {
        name: "has_completed_custom_achievement",
        description: r#"Checks if the player controlling the current scope has completed the specified custom achievement.

**Example:**
```paradox
has_completed_custom_achievement = {
    mod = my_mod_unique_id
    achievement = my_achievement_token
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "career_profile_check_medal",
        HOI4Entity {
            name: "career_profile_check_medal",
            description: r#"Checks if the required medal is achieved and collected.

**Example:**
```paradox
career_profile_check_medal = {
  medal = raining_debris_medal
  ???
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "career_profile_check_ribbon",
        HOI4Entity {
            name: "career_profile_check_ribbon",
            description: r#"Checks if the required ribbon is achieved and collected.

**Example:**
```paradox
career_profile_check_ribbon = {
  ribbon = orchestra_of_boom
  tooltip = my_loc_key
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "career_profile_check_playthrough_ratio",
        HOI4Entity {
            name: "career_profile_check_playthrough_ratio",
            description: r#"Compares the ratio (first/second) of two playthrough values to a number.

**Example:**
```paradox
career_profile_check_playthrough_ratio = {
  first = enemy_casualties
  second = total_own_casualties
  ratio = 4
  compare = greater_than_or_equals
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "career_profile_check_playthrough_value",
        HOI4Entity {
            name: "career_profile_check_playthrough_value",
            description: r#"Compares a playthrough value to a number.

**Example:**
```paradox
career_profile_check_playthrough_value = {
  plan_landlocked_battleship > 1
  plan_landlocked_carrier > 0
}
```

```paradox
career_profile_check_playthrough_value = {
  var = deployed_airplanes_with_air_defense_gold
  value = 100
  compare = greater_than_or_equals
  tooltip = CAREER_PROFILE_TRIGGER_DEPLOYED_AIRPLANES_WITH_AIR_DEFENSE
  tooltip_value = 100
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "career_profile_check_points",
        HOI4Entity {
            name: "career_profile_check_points",
            description: r#"Compares a career points value to a number.

**Example:**
```paradox
career_profile_check_points = {
  value = 5000
  compare = greater_than_or_equals
  tooltip = CAREER_PROFILE_TRIGGER_MINED_SEA_REGIONS
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("career_profile_check_ratio", HOI4Entity {
        name: "career_profile_check_ratio",
        description: r#"Compares the ratio (first/second) of two career profile values to a number.

**Example:**
```paradox
Possible the same as #career_profile_check_playthrough_ratio.
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "career_profile_check_value",
        HOI4Entity {
            name: "career_profile_check_value",
            description: r#"Compares a career profile value to a number.

**Example:**
```paradox
Possible the same as #career_profile_check_playthrough_value.
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "career_profile_has_player_flag",
        HOI4Entity {
            name: "career_profile_has_player_flag",
            description: r#"Checks if the flag is set for the local player.

**Example:**
```paradox
career_profile_has_player_flag = career_profile_overrun_infantry_flag
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "has_variable",
        HOI4Entity {
            name: "has_variable",
            description: r#"Checks if the specified variable exists for the current scope.

**Example:**
```paradox
has_variable = my_var
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "check_variable",
        HOI4Entity {
            name: "check_variable",
            description: r#"Check the specified variable for the current scope.

**Example:**
```paradox
check_variable = {
    var = my_var
    value = 10
    compare = greater_than_or_equals
}
```

```paradox
check_variable = {
    my_var > 10
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("log", HOI4Entity {
        name: "log",
        description: r#"Appends an entry into the game.log and, if open, the console when evaluating the trigger.

**Example:**
```paradox
log = "Added [?temp_add] to [THIS.GetTag]'s variable [?THIS.varvalue]"
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("print_variables", HOI4Entity {
        name: "print_variables",
        description: r#"Dumps the specified variables from the current scope and optionally the global scope into a log file with the specified name.

**Example:**
```paradox
print_variables = {
    var_list = { myvar1 myvar2 }
    file = "my_dump_file"
    text = "my header"
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "exists",
        HOI4Entity {
            name: "exists",
            description: r#"Checks if the current scope exists in game.

**Example:**
```paradox
exists = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "tag",
        HOI4Entity {
            name: "tag",
            description: r#"Checks if the current scope is the specified country.

**Example:**
```paradox
tag = GER
```

```paradox
tag = var:my_country
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "original_tag",
        HOI4Entity {
            name: "original_tag",
            description: r#"Checks if the current scope originates from the specified country.

**Example:**
```paradox
original_tag = GER
```

```paradox
original_tag = var:my_country
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "is_ai",
        HOI4Entity {
            name: "is_ai",
            description: r#"Checks if the current scope is AI.

**Example:**
```paradox
is_ai = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_collaboration", HOI4Entity {
        name: "has_collaboration",
        description: r#"Checks if the current scope has a collaboration level in the target scope.

**Example:**
```paradox
has_collaboration = {
    target = GER
    value > 0.5
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_country_flag",
        HOI4Entity {
            name: "has_country_flag",
            description: r#"Checks if the current scope has the specified flag.

**Example:**
```paradox
has_country_flag = my_flag
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_cosmetic_tag",
        HOI4Entity {
            name: "has_cosmetic_tag",
            description: r#"Checks if the current scope has the specified cosmetic tag active.

**Example:**
```paradox
has_cosmetic_tag = SOV_custom
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_event_target", HOI4Entity {
        name: "has_event_target",
        description: r#"Checks if current scope or global scope has the specified event target saved.

**Example:**
```paradox
has_event_target = my_var
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_decision",
        HOI4Entity {
            name: "has_decision",
            description: r#"Checks if the current scope has the specified decision activated.

**Example:**
```paradox
has_decision = my_decision
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_dynamic_modifier", HOI4Entity {
        name: "has_dynamic_modifier",
        description: r#"Checks if the current scope has the specified dynamic modifier activated.

**Example:**
```paradox
has_dynamic_modifier = {
    modifier = my_dynamic_modifier
    scope = GER
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_active_mission",
        HOI4Entity {
            name: "has_active_mission",
            description: r#"Checks if the current scope has the specified mission active.

**Example:**
```paradox
has_active_mission = my_mission
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_country_custom_difficulty_setting", HOI4Entity {
        name: "has_country_custom_difficulty_setting",
        description: r#"Checks if the any custom difficulty setting targeting the current scope is changed from the default value.

**Example:**
```paradox
has_country_custom_difficulty_setting = yes
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_terrain",
        HOI4Entity {
            name: "has_terrain",
            description: r#"Checks if the current scope has any provinces of the specified terrain.

**Example:**
```paradox
has_terrain = urban
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "is_dynamic_country",
        HOI4Entity {
            name: "is_dynamic_country",
            description: r#"Checks if the current scope is a dynamic country.

**Example:**
```paradox
is_dynamic_country = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("num_of_supply_nodes", HOI4Entity {
        name: "num_of_supply_nodes",
        description: r#"Checks if the current scope has the specified amount of supply nodes under control.

**Example:**
```paradox
num_of_supply_nodes > 10
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_resources_in_country", HOI4Entity {
        name: "has_resources_in_country",
        description: r#"Checks if the current scope has the specified amount of the specified resource in reserve.

**Example:**
```paradox
has_resources_in_country = {
    resource = oil
    amount > 10
    extracted = yes
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_focus_tree",
        HOI4Entity {
            name: "has_focus_tree",
            description: r#"Checks if the current scope has the specified focus tree.

**Example:**
```paradox
has_focus_tree = soviet_tree
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_completed_focus",
        HOI4Entity {
            name: "has_completed_focus",
            description: r#"Checks if the current scope has the specified focus completed.

**Example:**
```paradox
has_completed_focus = my_focus
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("focus_progress", HOI4Entity {
        name: "focus_progress",
        description: r#"Checks if the specified focus has been completed the specified percent for the current scope.

**Example:**
```paradox
focus_progress = {
  focus = my_focus
  progress > 0.5
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_shine_effect_on_focus", HOI4Entity {
        name: "has_shine_effect_on_focus",
        description: r#"Check if country has shine effect on focus (either manually achieved or by being worked on).

**Example:**
```paradox
has_shine_effect_on_focus = GER_wunderwaffe
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_political_power",
        HOI4Entity {
            name: "has_political_power",
            description: r#"Checks if the current scope has the specified amount of political power.

**Example:**
```paradox
has_political_power > 100
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("political_power_daily", HOI4Entity {
        name: "political_power_daily",
        description: r#"Checks if the current scope has the specified amount of daily political power gain.

**Example:**
```paradox
political_power_daily > 1
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("political_power_growth", HOI4Entity {
        name: "political_power_growth",
        description: r#"Checks if the current scope has the specified amount of daily political power gain.

**Example:**
```paradox
political_power_growth > 1
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "command_power",
        HOI4Entity {
            name: "command_power",
            description: r#"Checks if the current scope has the specified amount of command power.

**Example:**
```paradox
command_power > 1
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("command_power_daily", HOI4Entity {
        name: "command_power_daily",
        description: r#"Checks if the current scope has the specified amount of daily command power gain.

**Example:**
```paradox
command_power_daily > 1
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_war_support",
        HOI4Entity {
            name: "has_war_support",
            description: r#"Checks if the current scope has the specified percentage of War Support.

**Example:**
```paradox
has_war_support > 0.5
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_stability",
        HOI4Entity {
            name: "has_stability",
            description: r#"Checks if the current scope has the specified percentage of Stability.

**Example:**
```paradox
has_stability > 0.5
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_government", HOI4Entity {
        name: "has_government",
        description: r#"Checks if the ruling party of the current scope meets the requirements of being either the specified ideology group or having the same ideology group as the specified country.

**Example:**
```paradox
has_government = fascism
```

```paradox
has_government = ROOT
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_elections",
        HOI4Entity {
            name: "has_elections",
            description: r#"Checks if the current scope holds elections.

**Example:**
```paradox
has_elections = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "is_staging_coup",
        HOI4Entity {
            name: "is_staging_coup",
            description: r#"Checks if the current scope is staging a coup.

**Example:**
```paradox
is_staging_coup = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "is_target_of_coup",
        HOI4Entity {
            name: "is_target_of_coup",
            description: r#"Checks if the current scope is the target of a coup.

**Example:**
```paradox
is_target_of_coup = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_civil_war",
        HOI4Entity {
            name: "has_civil_war",
            description: r#"Checks if the current scope has a civil war active.

**Example:**
```paradox
has_civil_war = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "civilwar_target",
        HOI4Entity {
            name: "civilwar_target",
            description: r#"Checks if the specified country is a target of a civil war.

**Example:**
```paradox
civilwar_target = GER
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_manpower_for_recruit_change_to", HOI4Entity {
        name: "has_manpower_for_recruit_change_to",
        description: r#"Checks if the current scope has the specified amount of manpower for changing the specified idea group.

**Example:**
```paradox
has_manpower_for_recruit_change_to = {
    value > 0.05
    group = mobilization_laws
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_rule",
        HOI4Entity {
            name: "has_rule",
            description: r#"Checks if the current scope has the specified country rule.

**Example:**
```paradox
has_rule = can_create_factions
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_casualties_war_support", HOI4Entity {
        name: "has_casualties_war_support",
        description: r#"Checks if the current scope has the specified percentage of war support from own combat casualties.

**Example:**
```paradox
has_casualties_war_support < 0
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_convoys_war_support", HOI4Entity {
        name: "has_convoys_war_support",
        description: r#"Checks if the current scope has the specified percentage of war support from own convoys sunk.

**Example:**
```paradox
has_convoys_war_support < 0
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_bombing_war_support", HOI4Entity {
        name: "has_bombing_war_support",
        description: r#"Checks if the current scope has the specified percentage of war support from own states bombed by the enemy.

**Example:**
```paradox
has_bombing_war_support < 0
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_power_balance",
        HOI4Entity {
            name: "has_power_balance",
            description: r#"Checks if the current scope has the specified balance of power active.

**Example:**
```paradox
has_power_balance = {
    id = TAG_my_bop
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_any_power_balance",
        HOI4Entity {
            name: "has_any_power_balance",
            description: r#"Checks if the current scope has any balance of power active.

**Example:**
```paradox
has_any_power_balance = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("power_balance_value", HOI4Entity {
        name: "power_balance_value",
        description: r#"Checks if the current scope has the specified value within the balance of power.

**Example:**
```paradox
power_balance_value = {
    id = TAG_my_bop
    value > 0.7
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("power_balance_daily_change", HOI4Entity {
        name: "power_balance_daily_change",
        description: r#"Checks if the current scope's balance of power changes each day by the specified value.

**Example:**
```paradox
power_balance_daily_change = {
    id = TAG_my_bop
    value < -0.01
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("power_balance_weekly_change", HOI4Entity {
        name: "power_balance_weekly_change",
        description: r#"Checks if the current scope's balance of power changes each week by the specified value.

**Example:**
```paradox
power_balance_weekly_change = {
    id = TAG_my_bop
    value < -0.01
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("is_power_balance_in_range", HOI4Entity {
        name: "is_power_balance_in_range",
        description: r#"Checks if the current scope's balance of power value lies within the specified range.

**Example:**
```paradox
is_power_balance_in_range = {
    id = TAG_my_bop
    range > TAG_my_bop_right_range
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "is_power_balance_side_active",
        HOI4Entity {
            name: "is_power_balance_side_active",
            description: r#"Checks if the specified balance of power has a side active.

**Example:**
```paradox
is_power_balance_side_active = {
    id = TAG_my_bop
    side = TAG_my_bop_right_range
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_power_balance_modifier", HOI4Entity {
        name: "has_power_balance_modifier",
        description: r#"Checks if the current scope's balance of power value activates a modifier.

**Example:**
```paradox
has_power_balance_modifier = {
    id = TAG_my_bop
    modifier = TAG_my_bop_modifier
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("num_of_military_factories", HOI4Entity {
        name: "num_of_military_factories",
        description: r#"Checks if the current scope has the specified amount of military factories.

**Example:**
```paradox
num_of_military_factories > 10
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("num_of_civilian_factories", HOI4Entity {
        name: "num_of_civilian_factories",
        description: r#"Checks if the current scope has the specified amount of civilian factories.

**Example:**
```paradox
num_of_civilian_factories > 10
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "num_of_naval_factories",
        HOI4Entity {
            name: "num_of_naval_factories",
            description: r#"Checks if the current scope has the specified amount of dockyards.

**Example:**
```paradox
num_of_naval_factories > 10
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("num_of_available_military_factories", HOI4Entity {
        name: "num_of_available_military_factories",
        description: r#"Checks if the current scope has the specified amount of available military factories.

**Example:**
```paradox
num_of_available_military_factories > 10
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("num_of_available_civilian_factories", HOI4Entity {
        name: "num_of_available_civilian_factories",
        description: r#"Checks if the current scope has the specified amount of available civilian factories.

**Example:**
```paradox
num_of_available_civilian_factories > 10
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("num_of_available_naval_factories", HOI4Entity {
        name: "num_of_available_naval_factories",
        description: r#"Checks if the current scope has the specified amount of available dockyards.

**Example:**
```paradox
num_of_available_naval_factories > 10
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("num_of_factories", HOI4Entity {
        name: "num_of_factories",
        description: r#"Checks if the current scope has the specified amount of military, civilian or dockyard factories.

**Example:**
```paradox
num_of_factories > 10
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("num_of_controlled_factories", HOI4Entity {
        name: "num_of_controlled_factories",
        description: r#"Checks if the current scope has the specified amount of military, civilian or dockyard factories under control.

**Example:**
```paradox
num_of_controlled_factories > 10
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("num_of_owned_factories", HOI4Entity {
        name: "num_of_owned_factories",
        description: r#"Checks if the current scope has the specified amount of military, civilian or dockyard factories under owned states.

**Example:**
```paradox
num_of_owned_factories > 10
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("num_of_civilian_factories_available_for_projects", HOI4Entity {
        name: "num_of_civilian_factories_available_for_projects",
        description: r#"Checks if the current scope has the specified amount of civilian factories usable for projects.

**Example:**
```paradox
num_of_civilian_factories_available_for_projects > 10
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("ic_ratio", HOI4Entity {
        name: "ic_ratio",
        description: r#"Checks if the current scope has the specified ratio of factories with the target country.

**Example:**
```paradox
ic_ratio = {
    tag = GER
    ratio > 0.5
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_damaged_buildings",
        HOI4Entity {
            name: "has_damaged_buildings",
            description: r#"Checks if the current scope has any damanged buildings in their states.

**Example:**
```paradox
has_damaged_buildings = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_built", HOI4Entity {
        name: "has_built",
        description: r#"Checks if the current scope has built the specified building the specified number of times.

**Example:**
```paradox
has_built = {
    type = arms_factory
    value > 10
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_tech",
        HOI4Entity {
            name: "has_tech",
            description: r#"Checks if the current scope has the specified technology.

**Example:**
```paradox
has_tech = my_technology
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("is_researching_technology", HOI4Entity {
        name: "is_researching_technology",
        description: r#"Checks if the current scope is currently researching the specified technology.

**Example:**
```paradox
is_researching_technology = my_tech
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("can_research", HOI4Entity {
        name: "can_research",
        description: r#"Checks if the current scope can start researching the specified technology.

**Example:**
```paradox
can_research = my_tech
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("original_research_slots", HOI4Entity {
        name: "original_research_slots",
        description: r#"Checks if the current scope had the specified amount of slots at game start.

**Example:**
```paradox
original_research_slots > 3
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "amount_research_slots",
        HOI4Entity {
            name: "amount_research_slots",
            description: r#"Checks if the current scope has the specified amount of research slots.

**Example:**
```paradox
amount_research_slots > 3
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("is_in_tech_sharing_group", HOI4Entity {
        name: "is_in_tech_sharing_group",
        description: r#"Checks if the current scope is in the specified technology sharing group.

**Example:**
```paradox
is_in_tech_sharing_group = us_research
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("num_tech_sharing_groups", HOI4Entity {
        name: "num_tech_sharing_groups",
        description: r#"Checks if the current scope is in the specified amount of technology sharing groups.

**Example:**
```paradox
num_tech_sharing_groups > 3
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_tech_bonus", HOI4Entity {
        name: "has_tech_bonus",
        description: r#"Checks if the current scope has a technology bonus in the specified category, or for the specific technology.

**Example:**
```paradox
has_tech_bonus = {
    technology = my_tech
}
```

```paradox
has_tech_bonus = {
    category = my_category
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("land_doctrine_level", HOI4Entity {
        name: "land_doctrine_level",
        description: r#"Checks if the current scope has the specified amount of land doctrine technologies.

**Example:**
```paradox
land_doctrine_level > 2
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "num_researched_technologies",
        HOI4Entity {
            name: "num_researched_technologies",
            description: r#"Checks how many technologies the target has researched.

**Example:**
```paradox
num_researched_technologies > 10
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("is_special_project_being_researched", HOI4Entity {
        name: "is_special_project_being_researched",
        description: r#"Checks if the country in scope is currently researching the special project in input.

**Example:**
```paradox
is_special_project_being_researched = sp:sp_air_radar
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "is_special_project_completed",
        HOI4Entity {
            name: "is_special_project_completed",
            description: r#"Checks if the current scope has the specified special project completed.

**Example:**
```paradox
is_special_project_completed = sp:sp_land_flamethrower_tank
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_idea",
        HOI4Entity {
            name: "has_idea",
            description: r#"Checks if the current scope has the specified idea.

**Example:**
```paradox
has_idea = my_idea
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_idea_with_trait",
        HOI4Entity {
            name: "has_idea_with_trait",
            description: r#"Checks if the current scope has any ideas with the specified trait.

**Example:**
```paradox
has_idea_with_trait = my_trait
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_allowed_idea_with_traits", HOI4Entity {
        name: "has_allowed_idea_with_traits",
        description: r#"Checks if the current scope has the specified amount of ideas with the specified trait.

**Example:**
```paradox
has_available_idea_with_traits = {
    idea = my_trait
    limit = 1
    ignore = { generic_head_of_intelligence }
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_available_idea_with_traits", HOI4Entity {
        name: "has_available_idea_with_traits",
        description: r#"Checks if the current scope has the specified amount of ideas with the specified trait.

**Example:**
```paradox
has_available_idea_with_traits = {
    idea = my_trait
    limit = 1
    ignore = { generic_head_of_intelligence }
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("amount_taken_ideas", HOI4Entity {
        name: "amount_taken_ideas",
        description: r#"Checks if the current scope has the specified amount of ideas of the specified slot type. Excludes spirits, hidden ideas, and laws.

**Example:**
```paradox
amount_taken_ideas = {
    amount > 3
    slots = {
        political_advisor
    }
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "is_major",
        HOI4Entity {
            name: "is_major",
            description: r#"Checks if the current scope is considered a Major.

**Example:**
```paradox
is_major = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("is_ally_with", HOI4Entity {
        name: "is_ally_with",
        description: r#"Checks if the current scope is an ally (Faction members or subject-master relation).

**Example:**
```paradox
is_ally_with = GER
```

```paradox
is_ally_with = var:country
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "is_spymaster",
        HOI4Entity {
            name: "is_spymaster",
            description: r#"Checks if the current scope is the spymaster of a faction.

**Example:**
```paradox
is_spymaster = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_non_aggression_pact_with", HOI4Entity {
        name: "has_non_aggression_pact_with",
        description: r#"Checks if the current scope has a non-aggression pact with the specified country.

**Example:**
```paradox
has_non_aggression_pact_with = GER
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("is_guaranteed_by", HOI4Entity {
        name: "is_guaranteed_by",
        description: r#"Checks if the current scope has been guaranteed by the specified country.

**Example:**
```paradox
is_guaranteed_by = GER
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_guaranteed",
        HOI4Entity {
            name: "has_guaranteed",
            description: r#"Checks if the current scope has guaranteed the specified country.

**Example:**
```paradox
has_guaranteed = GER
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_military_access_to", HOI4Entity {
        name: "has_military_access_to",
        description: r#"Checks if the current scope has military access to the specified country.

**Example:**
```paradox
has_military_access_to = GER
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "gives_military_access_to",
        HOI4Entity {
            name: "gives_military_access_to",
            description: r#"Checks if the current scope gives military to the specified country.

**Example:**
```paradox
gives_military_access_to = GER
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "is_neighbor_of",
        HOI4Entity {
            name: "is_neighbor_of",
            description: r#"Checks if the current scope is a neighbor of the specified country.

**Example:**
```paradox
is_neighbor_of = GER
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("is_owner_neighbor_of", HOI4Entity {
        name: "is_owner_neighbor_of",
        description: r#"Checks if the current scope is a neighbor of the specified country with their core territory only.

**Example:**
```paradox
is_owner_neighbor_of = GER
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "is_puppet_of",
        HOI4Entity {
            name: "is_puppet_of",
            description: r#"Checks if the current scope is a puppet of the specified country.

**Example:**
```paradox
is_puppet_of = GER
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "is_subject_of",
        HOI4Entity {
            name: "is_subject_of",
            description: r#"Checks if the current scope is a subject of the specified scope.

**Example:**
```paradox
is_subject_of = GER
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "is_puppet",
        HOI4Entity {
            name: "is_puppet",
            description: r#"Returns true if the current country is a puppet.

**Example:**
```paradox
is_puppet = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "is_subject",
        HOI4Entity {
            name: "is_subject",
            description: r#"Checks if the current scope is a subject.

**Example:**
```paradox
is_subject = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_subject",
        HOI4Entity {
            name: "has_subject",
            description: r#"Checks if the country has for subject the given country.

**Example:**
```paradox
has_subject = GRE
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "num_subjects",
        HOI4Entity {
            name: "num_subjects",
            description: r#"Checks if the current scope has the specified amount of subjects.

**Example:**
```paradox
num_subjects > 3
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_autonomy_state",
        HOI4Entity {
            name: "has_autonomy_state",
            description: r#"Checks if the current scope is in the specified autonomous state.

**Example:**
```paradox
has_autonomy_state = autonomy_dominion
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("compare_autonomy_state", HOI4Entity {
        name: "compare_autonomy_state",
        description: r#"Checks if the current scope's autonomy state `min_freedom_level` is less or greater than that of the specified autonomy state. The special value "autonomy_free" compares as greater than any autonomy state. If the current scope is not a subject, it is treated as greater than any autonomy state (including "autonomy_free"). With `=`, checks if the current scope is in the specified autonomous state.

**Example:**
```paradox
compare_autonomy_state > autonomy_dominion
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("compare_autonomy_progress_ratio", HOI4Entity {
        name: "compare_autonomy_progress_ratio",
        description: r#"Checks if the current scope autonomy progress is at the specified ratio. If the current scope is not a subject, the ratio is 1.

**Example:**
```paradox
compare_autonomy_progress_ratio > 0.5
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_opinion_modifier",
        HOI4Entity {
            name: "has_opinion_modifier",
            description: r#"Checks if the current scope has the specified opinion modifier.

**Example:**
```paradox
has_opinion_modifier = my_modifier
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_opinion", HOI4Entity {
        name: "has_opinion",
        description: r#"Checks if the current scope has the specified opinion of the target country.

**Example:**
```paradox
has_opinion = {
    target = GER
    value > 50
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_relation_modifier", HOI4Entity {
        name: "has_relation_modifier",
        description: r#"Checks if the current scope has the specified relation modifier with the specified country.

**Example:**
```paradox
has_relation_modifier = {
    target = GER
    modifier = my_modifier
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_legitimacy",
        HOI4Entity {
            name: "has_legitimacy",
            description: r#"Checks how much legitimacy the current government in exile has.

**Example:**
```paradox
has_legitimacy > 50
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "is_exile_host",
        HOI4Entity {
            name: "is_exile_host",
            description: r#"Checks if the current country is hosting an exile.

**Example:**
```paradox
is_exile_host = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "is_hosting_exile",
        HOI4Entity {
            name: "is_hosting_exile",
            description: r#"Checks if the current country is hosting a specific exile.

**Example:**
```paradox
is_hosting_exile = POL
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "is_government_in_exile",
        HOI4Entity {
            name: "is_government_in_exile",
            description: r#"Checks if the current country is exiled in a different country.

**Example:**
```paradox
is_government_in_exile = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "is_exiled_in",
        HOI4Entity {
            name: "is_exiled_in",
            description: r#"Checks if the current country is exiled in a specific country.

**Example:**
```paradox
is_exiled_in = POL
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("received_expeditionary_forces", HOI4Entity {
        name: "received_expeditionary_forces",
        description: r#"Checks if the current country received X units in expeditions from the specified country.

**Example:**
```paradox
received_expeditionary_forces = {
    sender = POL
    value > 10
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("can_declare_war_on", HOI4Entity {
        name: "can_declare_war_on",
        description: r#"Checks if the current scope is able to declare war on the specified country.

**Example:**
```paradox
can_declare_war_on = POL
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "foreign_manpower",
        HOI4Entity {
            name: "foreign_manpower",
            description: r#"Checks how much foreign manpower we have received for garrisoning.

**Example:**
```paradox
foreign_manpower > 10000
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "is_embargoed_by",
        HOI4Entity {
            name: "is_embargoed_by",
            description: r#"Checks if the current scope is embargoed by the specified country.

**Example:**
```paradox
is_embargoed_by = USA
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "is_embargoing",
        HOI4Entity {
            name: "is_embargoing",
            description: r#"Checks if the current scope is embargoing the specified country.

**Example:**
```paradox
is_embargoing = CUB
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "is_in_faction",
        HOI4Entity {
            name: "is_in_faction",
            description: r#"Checks if the current scope is in a faction.

**Example:**
```paradox
is_in_faction = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "is_in_faction_with",
        HOI4Entity {
            name: "is_in_faction_with",
            description: r#"Checks if the current scope is in a faction with the specified country.

**Example:**
```paradox
is_in_faction_with = GER
```

```paradox
is_in_faction_with = var:country
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "is_faction_leader",
        HOI4Entity {
            name: "is_faction_leader",
            description: r#"Checks if the current scope is the leader of a faction.

**Example:**
```paradox
is_faction_leader = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("num_faction_members", HOI4Entity {
        name: "num_faction_members",
        description: r#"Checks if the faction of the current scope has the specified amount of members.

**Example:**
```paradox
num_faction_members > 1
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_manpower_to_become_leader", HOI4Entity {
        name: "has_manpower_to_become_leader",
        description: r#"Checks if the current country exceeds the current faction leader and its subjects in deployed manpower.

**Example:**
```paradox
has_manpower_to_become_leader = yes
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_industry_to_become_leader", HOI4Entity {
        name: "has_industry_to_become_leader",
        description: r#"Checks if the current country exceeds the faction leader in number of factories.

**Example:**
```paradox
has_industry_to_become_leader = yes
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_enough_influence_for_leadership", HOI4Entity {
        name: "has_enough_influence_for_leadership",
        description: r#"Checks if the current country has enough political influence to become faction leader.

**Example:**
```paradox
has_enough_influence_for_leadership = yes
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_faction_template",
        HOI4Entity {
            name: "has_faction_template",
            description: r#"Checks if the current country is in a faction with a template.

**Example:**
```paradox
has_faction_template = faction_template_chinese_united_front
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_active_rule",
        HOI4Entity {
            name: "has_active_rule",
            description: r#"Checks if the country's faction has a specific active rule.

**Example:**
```paradox
has_active_rule = government_in_exile_allowed
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_faction_goal",
        HOI4Entity {
            name: "has_faction_goal",
            description: r#"Checks if the country's faction has an active or completed goal.

**Example:**
```paradox
has_faction_goal = faction_goal_resource_control
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_completed_faction_goal",
        HOI4Entity {
            name: "has_completed_faction_goal",
            description: r#"Checks if the country's faction has successfully completed a goal.

**Example:**
```paradox
has_completed_faction_goal = faction_goal_resource_control
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "faction_goal_fulfillment",
        HOI4Entity {
            name: "faction_goal_fulfillment",
            description: r#"Checks fulfillment of a faction goal for the current country's faction.

**Example:**
```paradox
faction_goal_fulfillment = {
    goal = faction_goal_resource_control
    value > 0.85
}
```

```paradox
faction_goal_fulfillment = {
    goal = faction_goal_resource_control
    value > 0.5
    value < 0.85
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "faction_manifest_fulfillment",
        HOI4Entity {
            name: "faction_manifest_fulfillment",
            description: r#"Checks manifest fulfillment value of current country's faction manifest.

**Example:**
```paradox
faction_manifest_fulfillment > 0.95
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "faction_upgrade_level",
        HOI4Entity {
            name: "faction_upgrade_level",
            description: r#"Checks the active faction member upgrade against the specified upgrade.

**Example:**
```paradox
faction_upgrade_level > upgrade_token
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "faction_power_projection",
        HOI4Entity {
            name: "faction_power_projection",
            description: r#"Checks power value of current country's faction projection.

**Example:**
```paradox
faction_power_projection > 100
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "faction_influence_rank",
        HOI4Entity {
            name: "faction_influence_rank",
            description: r#"Checks influence rank in the faction of the current country.

**Example:**
```paradox
faction_influence_rank < 5
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "faction_influence_ratio",
        HOI4Entity {
            name: "faction_influence_ratio",
            description: r#"Checks influence ratio of current country in the faction.

**Example:**
```paradox
faction_influence_ratio > 0.1
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "faction_influence_score",
        HOI4Entity {
            name: "faction_influence_score",
            description: r#"Checks influence value of current country in the faction.

**Example:**
```paradox
faction_influence_score > 100
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("can_assign_supportive_scientist_to_faction", HOI4Entity {
        name: "can_assign_supportive_scientist_to_faction",
        description: r#"Checks if the faction from the country in scope has a free slot for a supportive scientist for the country with the specialization type.

**Example:**
```paradox
can_assign_supportive_scientist_to_faction = specialization_land
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_faction_research_unlocked",
        HOI4Entity {
            name: "has_faction_research_unlocked",
            description: r#"Whether the faction has unlocked the research.

**Example:**
```paradox
has_faction_research_unlocked = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_faction_military_unlocked",
        HOI4Entity {
            name: "has_faction_military_unlocked",
            description: r#"Whether the faction has unlocked the military operations.

**Example:**
```paradox
has_faction_military_unlocked = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("compare_ideology_with_faction", HOI4Entity {
        name: "compare_ideology_with_faction",
        description: r#"Compares the ideology support of the country's ruling party for the ideology of the faction it wants to join.

**Example:**
```paradox
compare_ideology_with_faction = {
    value > 0.5
    leader = FROM
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_war",
        HOI4Entity {
            name: "has_war",
            description: r#"Checks if the current scope is at war.

**Example:**
```paradox
has_war = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_war_with",
        HOI4Entity {
            name: "has_war_with",
            description: r#"Checks if the current scope is at war with the specified country.

**Example:**
```paradox
has_war_with = GER
```

```paradox
has_war_with = var:country
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_offensive_war_with", HOI4Entity {
        name: "has_offensive_war_with",
        description: r#"Checks if the current scope is in an offensive war against the specified country.

**Example:**
```paradox
has_offensive_war_with = GER
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_offensive_war_without_friend",
        HOI4Entity {
            name: "has_offensive_war_without_friend",
            description: r#"Is country at offensive war without specific ally present.

**Example:**
```paradox
has_offensive_war_without_friend = GER
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_defensive_war_with", HOI4Entity {
        name: "has_defensive_war_with",
        description: r#"Checks if the current scope is in an defensive war against the specified country.

**Example:**
```paradox
has_defensive_war_with = GER
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_offensive_war",
        HOI4Entity {
            name: "has_offensive_war",
            description: r#"Checks if the current scope is in an offensive war.

**Example:**
```paradox
has_offensive_war = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_defensive_war",
        HOI4Entity {
            name: "has_defensive_war",
            description: r#"Checks if the current scope is in a defensive war.

**Example:**
```paradox
has_defensive_war = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_war_together_with",
        HOI4Entity {
            name: "has_war_together_with",
            description: r#"Checks if the current scope is in a war alongside the specified country.

**Example:**
```paradox
has_war_together_with = GER
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_war_with_major", HOI4Entity {
        name: "has_war_with_major",
        description: r#"Checks if the current scope is at war with any other country that is considered major.

**Example:**
```paradox
has_war_with_major = yes
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_war_with_wargoal_against", HOI4Entity {
        name: "has_war_with_wargoal_against",
        description: r#"Checks if the current scope is at war with the specified country with the specified wargoal being active.

**Example:**
```paradox
has_war_with_wargoal_against = {
    target = ENG
    type = independence_wargoal
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("surrender_progress", HOI4Entity {
        name: "surrender_progress",
        description: r#"Checks if the current scope has the specified amount of surrender progress.

**Example:**
```paradox
surrender_progress > 0.1
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("any_war_score", HOI4Entity {
        name: "any_war_score",
        description: r#"Highest warscore value can be approximated by interating a variable by 1 for as long as any_war_score is greater than the variable. Checking with less than appears broken as a warscore of 0 is sometimes erroneously reported.

**Example:**
```paradox
any_war_score > 10
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_capitulated",
        HOI4Entity {
            name: "has_capitulated",
            description: r#"Checks if the current scope has capitulated.

**Example:**
```paradox
has_capitulated = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "days_since_capitulated",
        HOI4Entity {
            name: "days_since_capitulated",
            description: r#"Checks the amount of days since the target last capitulated.

**Example:**
```paradox
days_since_capitulated > 10
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_border_war_with",
        HOI4Entity {
            name: "has_border_war_with",
            description: r#"Checks if the current scope has a border war with the specified country.

**Example:**
```paradox
has_border_war_with = GER
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_border_war_between",
        HOI4Entity {
            name: "has_border_war_between",
            description: r#"Checks if there is a border war between the two specified states.

**Example:**
```paradox
has_border_war_between = {
    attacker = 1
    defender = 2
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_border_war",
        HOI4Entity {
            name: "has_border_war",
            description: r#"Checks if the current scope has a border war active.

**Example:**
```paradox
has_border_war = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_added_tension_amount", HOI4Entity {
        name: "has_added_tension_amount",
        description: r#"Checks if the current scope has caused the specified amount of World Tension.

**Example:**
```paradox
has_added_tension_amount > 10
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_wargoal_against", HOI4Entity {
        name: "has_wargoal_against",
        description: r#"Checks if the current scope has any wargoal against the specified country.

**Example:**
```paradox
has_wargoal_against = GER
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("is_justifying_wargoal_against", HOI4Entity {
        name: "is_justifying_wargoal_against",
        description: r#"Checks if the current scope is justifying a wargoal against the specified country.

**Example:**
```paradox
is_justifying_wargoal_against = GER
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_annex_war_goal", HOI4Entity {
        name: "has_annex_war_goal",
        description: r#"Checks if the current scope has the Annex wargoal against the specified country.

**Example:**
```paradox
has_annex_war_goal = GER
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "any_claim",
        HOI4Entity {
            name: "any_claim",
            description: r#"Checks if the current scope has any claims on another country.

**Example:**
```paradox
any_claim = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "is_in_peace_conference",
        HOI4Entity {
            name: "is_in_peace_conference",
            description: r#"Checks if the current scope is in a peace conference.

**Example:**
```paradox
is_in_peace_conference = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "controls_province",
        HOI4Entity {
            name: "controls_province",
            description: r#"Checks if the current scope has control of the specified province.

**Example:**
```paradox
controls_province = 1239
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "longest_war_length",
        HOI4Entity {
            name: "longest_war_length",
            description: r#"Checks how long a country has been at war, in months.

**Example:**
```paradox
longest_war_length > 3
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("war_length_with", HOI4Entity {
        name: "war_length_with",
        description: r#"Checks how long a country has been at war with specific country, in months.

**Example:**
```paradox
war_length_with = {
    tag = GER
    months > 3
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_truce_with",
        HOI4Entity {
            name: "has_truce_with",
            description: r#"Checks if the country has truce with the specified country.

**Example:**
```paradox
has_truce_with = GER
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_naval_control", HOI4Entity {
        name: "has_naval_control",
        description: r#"Checks if friendly nations and country scope together has enough naval dominance to assert control in strategic region.

**Example:**
```paradox
has_naval_control = 16
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_enemy_naval_control", HOI4Entity {
        name: "has_enemy_naval_control",
        description: r#"Checks if any enemy has enough naval dominance to assert control in certain strategic region.

**Example:**
```paradox
has_enemy_naval_control = 16
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "controls_state",
        HOI4Entity {
            name: "controls_state",
            description: r#"Checks if the current scope has control of the specified state.

**Example:**
```paradox
controls_state = 39
```

```paradox
controls_state = var:state
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "owns_state",
        HOI4Entity {
            name: "owns_state",
            description: r#"Checks if the current scope owns the specified state.

**Example:**
```paradox
owns_state = 39
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("num_of_controlled_states", HOI4Entity {
        name: "num_of_controlled_states",
        description: r#"Checks if the current scope has the specified amount of controlled states.

**Example:**
```paradox
num_of_controlled_states > 5
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "num_occupied_states",
        HOI4Entity {
            name: "num_occupied_states",
            description: r#"Checks if the current scope has the specified amount of occupied states.

**Example:**
```paradox
num_occupied_states > 5
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_full_control_of_state", HOI4Entity {
        name: "has_full_control_of_state",
        description: r#"Checks if the current scope has total control (100% occupation) of the specified state.

**Example:**
```paradox
has_full_control_of_state = 39
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_resources_rights",
        HOI4Entity {
            name: "has_resources_rights",
            description: r#"Checks if there are any resource rights with the specified parameters.

**Example:**
```paradox
has_resources_rights = {
  state = 123
  resources = { oil steel }
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("core_compliance", HOI4Entity {
        name: "core_compliance",
        description: r#"Compares the average compliance of core states of the specified country within controlled states of the current scope.

**Example:**
```paradox
core_compliance = {
    occupied_country_tag = ITA
    value > 10
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("core_resistance", HOI4Entity {
        name: "core_resistance",
        description: r#"Compares the average resistance of core states of the specified country within controlled states of the current scope.

**Example:**
```paradox
core_resistance = {
    occupied_country_tag = ITA
    value > 10
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("garrison_manpower_need", HOI4Entity {
        name: "garrison_manpower_need",
        description: r#"Checks how much garrison manpower we need for resistance in controlled states.

**Example:**
```paradox
garrison_manpower_need > 10000
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_core_occupation_modifier", HOI4Entity {
        name: "has_core_occupation_modifier",
        description: r#"Checks if the current scope has an occupation modifier for resistance/compliance that applies to our occupied states of a specified country.

**Example:**
```paradox
has_core_occupation_modifier = {
  occupied_country_tag = ITA
  modifier = token
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("occupation_law", HOI4Entity {
        name: "occupation_law",
        description: r#"Checks the occupation law that's either the default or applied over a specific country.

**Example:**
```paradox
POL = {
  POL = {
    occupation_law = foreign_civilian_oversight
  }
}
```

# Checks POL's default occupation law

```paradox
HOL = {
  BEL = {
    occupation_law = foreign_civilian_oversight
  }
}
```

# Checks HOL's occupation law over BEL"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_contested_owner", HOI4Entity {
        name: "has_contested_owner",
        description: r#"Checks if a state has the specified country as a contested owner. The trigger can be used either from a country or a state scope and accepts the other as parameter.

**Example:**
```paradox
has_contested_owner = 42
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "owns_any_state_of",
        HOI4Entity {
            name: "owns_any_state_of",
            description: r#"Check if the country owns any of the states in the list.

**Example:**
```paradox
owns_any_state_of = {
  123
  246
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("is_on_same_continent_as", HOI4Entity {
        name: "is_on_same_continent_as",
        description: r#"Checks if the scope country is on the same continent as the given state. The capital state is used for given country tag.

**Example:**
```paradox
is_on_same_continent_as = 111
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_army_experience",
        HOI4Entity {
            name: "has_army_experience",
            description: r#"Checks if the current scope has the specified amount of Army experience.

**Example:**
```paradox
has_army_experience > 10
```

```paradox
has_army_experience > var:number
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_air_experience",
        HOI4Entity {
            name: "has_air_experience",
            description: r#"Checks if the current scope has the specified amount of Air experience.

**Example:**
```paradox
has_air_experience > 10
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_navy_experience",
        HOI4Entity {
            name: "has_navy_experience",
            description: r#"Checks if the current scope has the specified amount of Navy experience.

**Example:**
```paradox
has_navy_experience < 10
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_manpower",
        HOI4Entity {
            name: "has_manpower",
            description: r#"Checks if the current scope has the specified amount of manpower.

**Example:**
```paradox
has_manpower > 1000
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_army_manpower", HOI4Entity {
        name: "has_army_manpower",
        description: r#"Checks if the current scope has an army using the specified amount of manpower.

**Example:**
```paradox
has_army_manpower = {
    size > 1000
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("manpower_per_military_factory", HOI4Entity {
        name: "manpower_per_military_factory",
        description: r#"Checks if the current scope has the specified manpower times their number of military factories.

**Example:**
```paradox
manpower_per_military_factory > 1000
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("conscription_ratio", HOI4Entity {
        name: "conscription_ratio",
        description: r#"Checks if the current scope has the specified conscription ratio currently, not to be mixed up with the target conscription ratio.

**Example:**
```paradox
conscription_ratio < 0.2
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "current_conscription_amount",
        HOI4Entity {
            name: "current_conscription_amount",
            description: r#"Checks if the current scope has already conscripted that much manpower.

**Example:**
```paradox
current_conscription_amount > 2000
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("target_conscription_amount", HOI4Entity {
        name: "target_conscription_amount",
        description: r#"Checks if the current scope is targeting to conscript that much manpower.

**Example:**
```paradox
target_conscription_amount > 2000
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "num_divisions",
        HOI4Entity {
            name: "num_divisions",
            description: r#"Checks if the current scope has the specified amount of divisions.

**Example:**
```paradox
num_divisions > 5
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "num_of_nukes",
        HOI4Entity {
            name: "num_of_nukes",
            description: r#"Checks if the current scope has the specified amount of nukes.

**Example:**
```paradox
num_of_nukes > 5
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("casualties", HOI4Entity {
        name: "casualties",
        description: r#"Checks if the current scope has suffered the specified amount of casualties.

**Example:**
```paradox
casualties > 10000
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("casualties_k", HOI4Entity {
        name: "casualties_k",
        description: r#"Checks if the current scope has suffered the specified amount of casualties in thousands.

**Example:**
```paradox
casualties_k > 10
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("casualties_inflicted_by", HOI4Entity {
        name: "casualties_inflicted_by",
        description: r#"Checks if the current scope has suffered the specified amount of casualties in thousands from a specific country.

**Example:**
```paradox
casualties_inflicted_by = {
    opponent = POL
    thousands > 10
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("amount_manpower_in_deployment_queue", HOI4Entity {
        name: "amount_manpower_in_deployment_queue",
        description: r#"Checks if the current scope has the specified amount of manpower in their deployment queue.

**Example:**
```paradox
amount_manpower_in_deployment_queue > 1000
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_attache_from",
        HOI4Entity {
            name: "has_attache_from",
            description: r#"Checks if the current scope has an attache from the specified scope.

**Example:**
```paradox
has_attache_from = GER
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_attache",
        HOI4Entity {
            name: "has_attache",
            description: r#"Checks if the current scope has an attache.

**Example:**
```paradox
has_attache = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "is_lend_leasing",
        HOI4Entity {
            name: "is_lend_leasing",
            description: r#"Checks if the current scope is lend leasing to the specified scope.

**Example:**
```paradox
is_lend_leasing = GER
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_template", HOI4Entity {
        name: "has_template",
        description: r#"Checks if the current scope has a division template of the specified name.

**Example:**
```paradox
has_template = "Infantry Division"
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_template_majority_unit", HOI4Entity {
        name: "has_template_majority_unit",
        description: r#"Checks if the current scope has a division template composed mostly of the specified unit.

**Example:**
```paradox
has_template_majority_unit = infantry
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_template_containing_unit", HOI4Entity {
        name: "has_template_containing_unit",
        description: r#"Checks if the current scope has a division template contained any of the specified unit.

**Example:**
```paradox
has_template_containing_unit = light_armor
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("strength_ratio", HOI4Entity {
        name: "strength_ratio",
        description: r#"Checks if the current scope has the specified strength ratio against the specified country. The ratio is the number of fielded divisions of the current scope divided by those of `tag` (or 1 if `tag` has no divisions). The ratio gets increased by 10% if the current scope has a stronger air forces.[2]

**Example:**
```paradox
strength_ratio = {
    tag = GER
    ratio > 1
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("fighting_army_strength_ratio", HOI4Entity {
        name: "fighting_army_strength_ratio",
        description: r#"Compares the total army fighting strength between the scope country and the one set with 'tag'.

**Example:**
```paradox
fighting_army_strength_ratio = {
    tag = GER
    ratio > 0.7
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("naval_strength_ratio", HOI4Entity {
        name: "naval_strength_ratio",
        description: r#"Checks if the current scope has the specified naval strength ratio against the specified country.

**Example:**
```paradox
naval_strength_ratio = {
    tag = GER
    ratio <> 1
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("naval_strength_comparison", HOI4Entity {
        name: "naval_strength_comparison",
        description: r#"Checks if the current scope has the specified naval strength ratio against the specified country.

**Example:**
```paradox
naval_strength_comparison = {
    other = POL
    tooltip = my_loc_key_tt
    ratio > 1
    sub_unit_def_weights = {
        carrier = 1
        submarine = 2
    }
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("alliance_strength_ratio", HOI4Entity {
        name: "alliance_strength_ratio",
        description: r#"Checks if the current scope and allies has an army strength higher than the specified ratio against estimated enemy strength.

**Example:**
```paradox
alliance_strength_ratio > 0.5
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("alliance_naval_strength_ratio", HOI4Entity {
        name: "alliance_naval_strength_ratio",
        description: r#"Checks if the current scope and allies has an naval strength ratio higher than the specified ratio against estimated enemy strength.

**Example:**
```paradox
alliance_naval_strength_ratio > 0.5
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("enemies_strength_ratio", HOI4Entity {
        name: "enemies_strength_ratio",
        description: r#"Checks if the estimated enemy army strength ratio is higher than the specified ratio.

**Example:**
```paradox
enemies_strength_ratio > 0.5
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("enemies_naval_strength_ratio", HOI4Entity {
        name: "enemies_naval_strength_ratio",
        description: r#"Checks if the estimated enemy naval strength ratio is higher than the specified ratio.

**Example:**
```paradox
enemies_naval_strength_ratio > 0.5
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_army_size", HOI4Entity {
        name: "has_army_size",
        description: r#"Checks if the current scope has the specified number of divisions, or of a specified type of division.

**Example:**
```paradox
has_army_size = {
    size > 10
    type = armor
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_navy_size", HOI4Entity {
        name: "has_navy_size",
        description: r#"Checks if the current scope has the specified number of ships, or of a specified type of ship.

**Example:**
```paradox
has_navy_size = {
    size > 10
    type = capital_ship
    archetype = ship_hull_heavy
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_deployed_air_force_size", HOI4Entity {
        name: "has_deployed_air_force_size",
        description: r#"Checks if the current scope has the specified number of aircraft, or of a specified type of aircraft.

**Example:**
```paradox
has_deployed_air_force_size = {
    size > 10
    type = cas
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("divisions_in_state", HOI4Entity {
        name: "divisions_in_state",
        description: r#"Checks if the specified state contains the specified amount of divisions.

**Example:**
```paradox
divisions_in_state = {
    type = armor
    size > 10
    state = 49
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("army_manpower_in_state", HOI4Entity {
        name: "army_manpower_in_state",
        description: r#"Checks if the specified state contains the specified amount of army manpower within the state.

**Example:**
```paradox
army_manpower_in_state = {
    type = support
    amount > 10000
    state = 49
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("divisions_in_border_state", HOI4Entity {
        name: "divisions_in_border_state",
        description: r#"Checks if the border provinces between the specified state and border state contain the specified amount of divisions.

**Example:**
```paradox
divisions_in_border_state = {
    type = infantry
    size > 10
    state = 49
    border_state = var:state
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("num_divisions_in_states", HOI4Entity {
        name: "num_divisions_in_states",
        description: r#"Checks if the specified states contain enough divisions of the specified types.

**Example:**
```paradox
num_divisions_in_states = {
    count > 24
    states = { 550 559 271 }
    exclude = { irregular_infantry }
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("num_battalions_in_states", HOI4Entity {
        name: "num_battalions_in_states",
        description: r#"Checks if the specified states contain enough battalions (or sub-units) of the specified types.

**Example:**
```paradox
num_battalions_in_states = {
    count > 24
    states = { 550 559 271 }
    exclude = { irregular_infantry }
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("ships_in_state_ports", HOI4Entity {
        name: "ships_in_state_ports",
        description: r#"Checks if the specified state contains the specified amount of ships, or of ships of the specified type.

**Example:**
```paradox
ships_in_state_ports = {
    type = capital_ship
    size > 10
    state = 49
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("num_planes_stationed_in_regions", HOI4Entity {
        name: "num_planes_stationed_in_regions",
        description: r#"Checks if the current scope has the specified number of aircraft stationed within strategic regions.

**Example:**
```paradox
num_planes_stationed_in_regions = {
    value > 10
    regions = { 123 321 }
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_volunteers_amount_from", HOI4Entity {
        name: "has_volunteers_amount_from",
        description: r#"Checks if the current scope has recieved volunteers from the specified country of the specified amounts.

**Example:**
```paradox
has_volunteers_amount_from = {
    tag = GER
    count > 10
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "convoy_threat",
        HOI4Entity {
            name: "convoy_threat",
            description: r#"Checks how much the convoys are threatened.

**Example:**
```paradox
convoy_threat > 0.5
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_mined", HOI4Entity {
        name: "has_mined",
        description: r#"Checks if the current scope has X mines on the coast of the specified country.

**Example:**
```paradox
has_mined = {
    target = POL
    value > 1000
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_mines", HOI4Entity {
        name: "has_mines",
        description: r#"Checks if the current scope has at least X mines within the specified strategic region.

**Example:**
```paradox
has_mined = {
    target = POL
    amount = 1000
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "mine_threat",
        HOI4Entity {
            name: "mine_threat",
            description: r#"Checks how dangerous enemy mines are.

**Example:**
```paradox
mine_threat < 0.6
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_military_industrial_organization",
        HOI4Entity {
            name: "has_military_industrial_organization",
            description: r#"Checks if the current scope has a MIO with the specified name.

**Example:**
```paradox
has_military_industrial_organization = infantry_mio_token
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_tactic", HOI4Entity {
        name: "has_tactic",
        description: r#"Check if the given tactic is unlocked (or active by default) for the country.

**Example:**
```paradox
has_tactic = tactic_masterful_blitz
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_any_grand_doctrine", HOI4Entity {
        name: "has_any_grand_doctrine",
        description: r#"Checks if any grand doctrine in folder is currently active for the country.

**Example:**
```paradox
has_any_grand_doctrine = land
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_doctrine", HOI4Entity {
        name: "has_doctrine",
        description: r#"Checks if the given grand doctrine or subdoctrine is currently active for the country.

**Example:**
```paradox
has_doctrine = mobile_warfare # Grand doctrine
```

```paradox
has_doctrine = mobile_infantry # Subdoctrine
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_subdoctrine_in_track", HOI4Entity {
        name: "has_subdoctrine_in_track",
        description: r#"Checks if any subdoctrine is currently assigned to (any instance of) the given track.

**Example:**
```paradox
has_subdoctrine_in_track = infantry
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_completed_subdoctrine", HOI4Entity {
        name: "has_completed_subdoctrine",
        description: r#"Checks if the current country has ever completed the specified subdoctrine (even if it was later switched out).

**Example:**
```paradox
has_completed_subdoctrine = mobile_infantry
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_completed_track",
        HOI4Entity {
            name: "has_completed_track",
            description: r#"Checks if the given subdoctrine track has been completed

**Example:**
```paradox
has_completed_track = infantry
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_mastery",
        HOI4Entity {
            name: "has_mastery",
            description: r#"Checks if any track of the given type has at least X mastery.

**Example:**
```paradox
has_mastery = {
    amount = 200
    track = infantry
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_mastery_level", HOI4Entity {
        name: "has_mastery_level",
        description: r#"Checks if the country has reached the specified number of mastery levels (rewards) for the given subdoctrine.

**Example:**
```paradox
has_mastery_level = {
    amount = 2
    subdoctrine = mobile_infantry
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("stockpile_ratio", HOI4Entity {
        name: "stockpile_ratio",
        description: r#"Checks if the current scope has stockpiled the specified equipment to the specified ratio against fielded equipment of the same type.

**Example:**
```paradox
stockpile_ratio = {
    archetype = infantry_equipment
    ratio > 0.5
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_equipment", HOI4Entity {
        name: "has_equipment",
        description: r#"Checks if the current scope has the specified equipment to the specified amount.

**Example:**
```paradox
has_equipment = {
    infantry_equipment_1 > 10
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_any_license",
        HOI4Entity {
            name: "has_any_license",
            description: r#"Checks if the current scope has any licenses from other countries.

**Example:**
```paradox
has_any_license = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "is_licensing_any_to",
        HOI4Entity {
            name: "is_licensing_any_to",
            description: r#"Checks if the current scope is licensing to the specified scope.

**Example:**
```paradox
is_licensing_any_to = GER
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("is_licensing_to", HOI4Entity {
        name: "is_licensing_to",
        description: r#"Checks if the current scope is licensing the specified equipment to the specified country.

**Example:**
```paradox
is_licensing_to = {
    target = GER
    archetype = infantry_equipment
}
```

```paradox
is_licensing_to = {
    target = GER
    equipment = {
        type = light_tank_equipment
        version = 1
    }
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_license", HOI4Entity {
        name: "has_license",
        description: r#"Checks if the current scope has a license for the specified equipment from the specified country.

**Example:**
```paradox
has_license = {
    from = GER
    archetype = infantry_equipment
}
```

```paradox
has_license = {
    from = GER
    equipment = {
        type = light_tank_equipment
        version = 1
    }
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "fuel_ratio",
        HOI4Entity {
            name: "fuel_ratio",
            description: r#"Checks the fuel ratio of the country.

**Example:**
```paradox
fuel_ratio > 0.4
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_fuel",
        HOI4Entity {
            name: "has_fuel",
            description: r#"Checks the fuel amount of the country.

**Example:**
```paradox
has_fuel > 400
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_design_based_on", HOI4Entity {
        name: "has_design_based_on",
        description: r#"Checks if the country has a builtable non-obsolete design based on the specified equipment archetype.

**Example:**
```paradox
has_design_based_on = light_tank_chassis
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("estimated_intel_max_piercing", HOI4Entity {
        name: "estimated_intel_max_piercing",
        description: r#"Checks if the specified scope has the specified amount of piercing based on the current scope's intel.

**Example:**
```paradox
estimated_intel_max_piercing = {
    tag = GER
    value > 2
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("estimated_intel_max_armor", HOI4Entity {
        name: "estimated_intel_max_armor",
        description: r#"Checks if the specified scope has the specified amount of armor based on the current scope's intel.

**Example:**
```paradox
estimated_intel_max_armor = {
    tag = GER
    value > 2
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "compare_intel_with",
        HOI4Entity {
            name: "compare_intel_with",
            description: r#"Compares intel between 2 countries.

**Example:**
```paradox
compare_intel_with = {
    target = POL
    civilian_intel > 0.5
    army_intel = 0
    navy_intel < 0
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("intel_level_over", HOI4Entity {
        name: "intel_level_over",
        description: r#"Checks the intel level from the current country over a specified country.

**Example:**
```paradox
intel_level_over = {
    target = POL
    civilian_intel > 0.5
    army_intel = 0
    navy_intel < 0
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_intelligence_agency",
        HOI4Entity {
            name: "has_intelligence_agency",
            description: r#"Checks if the current scope has an intelligence agency.

**Example:**
```paradox
has_intelligence_agency = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "network_national_coverage",
        HOI4Entity {
            name: "network_national_coverage",
            description: r#"Checks network national coverage over a specific country.

**Example:**
```paradox
network_national_coverage = {
    target = POL
    value < 70
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "network_strength",
        HOI4Entity {
            name: "network_strength",
            description: r#"Checks network national coverage over a specific country.

**Example:**
```paradox
network_strength = {
    target = POL
    value < 70
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_done_agency_upgrade", HOI4Entity {
        name: "has_done_agency_upgrade",
        description: r#"Checks if the current scope has the specified agency upgrade (to its highest level).

**Example:**
```paradox
has_done_agency_upgrade = upgrade_army_department
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("agency_upgrade_number", HOI4Entity {
        name: "agency_upgrade_number",
        description: r#"Checks the number of upgrades done in the current scope's intelligence agency.

**Example:**
```paradox
agency_upgrade_number > 4
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "decryption_progress",
        HOI4Entity {
            name: "decryption_progress",
            description: r#"Checks the decryption progress towards a country.

**Example:**
```paradox
decryption_progress = {
    target = POL
    value < 0.5
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_captured_operative",
        HOI4Entity {
            name: "has_captured_operative",
            description: r#"Checks if the current scope has captured an operative.

**Example:**
```paradox
has_captured_operative = POL
```

```paradox
has_captured_operative = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_finished_collecting_for_operation", HOI4Entity {
        name: "has_finished_collecting_for_operation",
        description: r#"Checks if the current scope has finished collecting resources for an operation.

**Example:**
```paradox
has_finished_collecting_for_operation = {
    target = POL
    operation = operation_infiltrate_armed_forces_navy
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("is_preparing_operation", HOI4Entity {
        name: "is_preparing_operation",
        description: r#"Checks if the current scope is preparing an operation against the specified country.

**Example:**
```paradox
is_preparing_operation = {
    target = POL
    operation = operation_infiltrate_armed_forces_navy
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("is_running_operation", HOI4Entity {
        name: "is_running_operation",
        description: r#"Checks if the current scope is running an operation against the specified country.

**Example:**
```paradox
is_running_operation = {
    target = POL
    operation = operation_infiltrate_armed_forces_navy
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("num_finished_operations", HOI4Entity {
        name: "num_finished_operations",
        description: r#"Checks how many finished operations the current scope had against the specified country.

**Example:**
```paradox
num_finished_operations = {
    target = POL
    operation = operation_infiltrate_armed_forces_navy
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_operation_token", HOI4Entity {
        name: "has_operation_token",
        description: r#"Checks if the current scope has an operation token against an another country.

**Example:**
```paradox
has_operation_token = {
    tag = POL
    token = token_name
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("is_active_decryption_bonuses_enabled", HOI4Entity {
        name: "is_active_decryption_bonuses_enabled",
        description: r#"Checks if the current scope has any decryption bonuses towards the specified country.

**Example:**
```paradox
is_active_decryption_bonuses_enabled = POL
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "is_cryptology_department_active",
        HOI4Entity {
            name: "is_cryptology_department_active",
            description: r#"Checks if the current scope has a cryptology department active.

**Example:**
```paradox
is_cryptology_department_active = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "is_decrypting",
        HOI4Entity {
            name: "is_decrypting",
            description: r#"Checks if the current scope is decrypting a certain country.

**Example:**
```paradox
is_decrypting = POL
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "is_fully_decrypted",
        HOI4Entity {
            name: "is_fully_decrypted",
            description: r#"Checks if the current scope has fully decrypted a certain country.

**Example:**
```paradox
is_fully_decrypted = POL
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "num_fake_intel_divisions",
        HOI4Entity {
            name: "num_fake_intel_divisions",
            description: r#"Checks the amount of fake intel divisions.

**Example:**
```paradox
num_fake_intel_divisions > 10
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "num_free_operative_slots",
        HOI4Entity {
            name: "num_free_operative_slots",
            description: r#"Checks the amount of free operative slots.

**Example:**
```paradox
num_free_operative_slots > 2
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "num_operative_slots",
        HOI4Entity {
            name: "num_operative_slots",
            description: r#"Checks the amount of operative slots.

**Example:**
```paradox
num_operative_slots > 2
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "num_of_operatives",
        HOI4Entity {
            name: "num_of_operatives",
            description: r#"Checks the amount of operatives.

**Example:**
```paradox
num_of_operatives > 2
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "ai_irrationality",
        HOI4Entity {
            name: "ai_irrationality",
            description: r#"Checks if the current scope AI has the specified irrationality.

**Example:**
```paradox
ai_irrationality > 10
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("ai_liberate_desire", HOI4Entity {
        name: "ai_liberate_desire",
        description: r#"Checks if the current scope AI has the specified liberation desire towards the specified country.

**Example:**
```paradox
ai_liberate_desire = {
    target = GER
    count > 1
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "ai_has_role_division",
        HOI4Entity {
            name: "ai_has_role_division",
            description: r#"Checks if the current scope AI has a division with the specified role.

**Example:**
```paradox
ai_has_role_division = infantry
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("ai_has_role_template", HOI4Entity {
        name: "ai_has_role_template",
        description: r#"Checks if the current scope AI has a division template with the specified role.

**Example:**
```paradox
ai_has_role_template = armor
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("ai_wants_divisions", HOI4Entity {
        name: "ai_wants_divisions",
        description: r#"Checks if the current scope AI desires the specified amount of divisions.

**Example:**
```paradox
ai_wants_divisions > 10
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_template_ai_majority_unit", HOI4Entity {
        name: "has_template_ai_majority_unit",
        description: r#"Checks if the current scope AI has a division template mostly made up of the specified unit.

**Example:**
```paradox
has_template_ai_majority_unit = infantry
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("can_be_country_leader", HOI4Entity {
        name: "can_be_country_leader",
        description: r#"Checks if the specified character has a country leader role, active or not, and can utilise it in this country.

**Example:**
```paradox
can_be_country_leader = POL_character_test
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("has_character", HOI4Entity {
        name: "has_character",
        description: r#"Checks if the current scope has the specified character recruited. The character does NOT need to be in power.

**Example:**
```paradox
has_character = my_character
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_country_leader",
        HOI4Entity {
            name: "has_country_leader",
            description: r#"Checks if the current scope has the specified country leader.

**Example:**
```paradox
has_country_leader = {
    id = 10
}
```

```paradox
has_country_leader = {
	character = SPR_niceto_alcala_zamora
	ruling_only = yes
}
```

```paradox
has_country_leader = {
    name = "John Smith"
    ruling_only = yes
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_country_leader_ideology", HOI4Entity {
        name: "has_country_leader_ideology",
        description: r#"Checks if the current scope's active country leader has the specified ideology.

**Example:**
```paradox
has_country_leader_ideology = nazism
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "has_country_leader_with_trait",
        HOI4Entity {
            name: "has_country_leader_with_trait",
            description: r#"Checks if the leader of the country has a specific trait.

**Example:**
```paradox
has_country_leader_with_trait = champion_of_peace_1
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "is_female",
        HOI4Entity {
            name: "is_female",
            description: r#"Checks if the current country leader is female.

**Example:**
```paradox
is_female = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "has_unit_leader",
        HOI4Entity {
            name: "has_unit_leader",
            description: r#"Checks if the current scope has a unit leader with the specified id.

**Example:**
```paradox
has_unit_leader = 1
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("has_scientist_specialization", HOI4Entity {
        name: "has_scientist_specialization",
        description: r#"Checks if the country in scope has a scientist with a skill level of at least 1 in specialization.

**Example:**
```paradox
has_scientist_specialization = specialization_nuclear
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "pc_is_winner",
        HOI4Entity {
            name: "pc_is_winner",
            description: r#"Checks if the current scope is a winner within the peace conference.

**Example:**
```paradox
pc_is_winner = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("pc_is_on_winning_side", HOI4Entity {
        name: "pc_is_on_winning_side",
        description: r#"Checks if the current scope is on the winning side within the peace conference.

**Example:**
```paradox
pc_is_on_winning_side = yes
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "pc_is_loser",
        HOI4Entity {
            name: "pc_is_loser",
            description: r#"Checks if the current scope is a loser within the peace conference.

**Example:**
```paradox
pc_is_loser = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("pc_is_untouched_loser", HOI4Entity {
        name: "pc_is_untouched_loser",
        description: r#"Checks if the current scope is an untouched loser within the peace conference.

**Example:**
```paradox
pc_is_untouched_loser = yes
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("pc_is_on_same_side_as", HOI4Entity {
        name: "pc_is_on_same_side_as",
        description: r#"Checks if the current scope is on the same side of the peace conference as the specified country.

**Example:**
```paradox
pc_is_on_same_side_as = BHR
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("pc_is_liberated", HOI4Entity {
        name: "pc_is_liberated",
        description: r#"Checks if the current scope has been liberated within the peace conference.

**Example:**
```paradox
pc_is_liberated = yes
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("pc_is_liberated_by", HOI4Entity {
        name: "pc_is_liberated_by",
        description: r#"Checks if the current scope has been liberated within the peace conference by the specified country.

**Example:**
```paradox
pc_is_liberated_by = BHR
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("pc_is_puppeted", HOI4Entity {
        name: "pc_is_puppeted",
        description: r#"Checks if the current scope has been puppeted within the peace conference.

**Example:**
```paradox
pc_is_puppeted = yes
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("pc_is_puppeted_by", HOI4Entity {
        name: "pc_is_puppeted_by",
        description: r#"Checks if the current scope has been puppeted within the peace conference by the specified country.

**Example:**
```paradox
pc_is_puppeted_by = BHR
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("pc_is_forced_government", HOI4Entity {
        name: "pc_is_forced_government",
        description: r#"Checks if the current scope has had an enforced government change within the peace conference.

**Example:**
```paradox
pc_is_forced_government = yes
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("pc_is_forced_government_by", HOI4Entity {
        name: "pc_is_forced_government_by",
        description: r#"Checks if the current scope has had an enforced government change within the peace conference demanded by the specified country.

**Example:**
```paradox
pc_is_forced_government_by = BHR
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("pc_is_forced_government_to", HOI4Entity {
        name: "pc_is_forced_government_to",
        description: r#"Checks if the current scope has had an enforced government change to the specified ideology group.

**Example:**
```paradox
pc_is_forced_government_to = democratic
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("pc_total_score", HOI4Entity {
        name: "pc_total_score",
        description: r#"Checks if the current scope has the specified amount in total score within the peace conference.

**Example:**
```paradox
pc_total_score > 2400
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("pc_current_score", HOI4Entity {
        name: "pc_current_score",
        description: r#"Checks if the current scope has the specified amount in current score within the peace conference.

**Example:**
```paradox
pc_current_score > 100
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "state",
        HOI4Entity {
            name: "state",
            description: r#"Checks if the current scope is the specified state.

**Example:**
```paradox
state = 10
```

```paradox
state = var:state
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert("region", HOI4Entity {
        name: "region",
        description: r#"Checks if the current scope is a state in the specified strategic region.

**Example:**
```paradox
region = 10
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert("free_building_slots", HOI4Entity {
        name: "free_building_slots",
        description: r#"Checks if the current scope has available slots for the specified amount of buildings.

**Example:**
```paradox
free_building_slots = {
    building = arms_factory
    size > 10
    include_locked = yes
}
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert("non_damaged_building_level", HOI4Entity {
        name: "non_damaged_building_level",
        description: r#"Checks if the current scope has the specified amount of the specified buildings that are undamaged.

**Example:**
```paradox
non_damaged_building_level = {
    building = arms_factory
    level > 4
}
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert("any_province_building_level", HOI4Entity {
        name: "any_province_building_level",
        description: r#"Checks if the current scope has the specified provincal building at the specified amount in the specified provinces.

**Example:**
```paradox
any_province_building_level = {
    province = {
        id = 445
        id = 494
        limit_to_border = yes
    }
    building = bunker
    level < 5
}
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert(
        "has_state_flag",
        HOI4Entity {
            name: "has_state_flag",
            description: r#"Checks if the current scope has the specified flag.

**Example:**
```paradox
has_state_flag = my_flag
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "state_population",
        HOI4Entity {
            name: "state_population",
            description: r#"Checks if the current scope has the specified state population.

**Example:**
```paradox
state_population > 10000
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert("state_population_k", HOI4Entity {
        name: "state_population_k",
        description: r#"Checks if the current scope has the specified state population in thousands.

**Example:**
```paradox
state_population_k > 10
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert(
        "is_capital",
        HOI4Entity {
            name: "is_capital",
            description: r#"Checks if the current scope is a capital.

**Example:**
```paradox
is_capital = yes
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "is_controlled_by",
        HOI4Entity {
            name: "is_controlled_by",
            description: r#"Checks if the current scope is controlled by the specified country.

**Example:**
```paradox
is_controlled_by = GER
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert("is_fully_controlled_by", HOI4Entity {
        name: "is_fully_controlled_by",
        description: r#"Checks if the current scope is fully controlled by the specified country.

**Example:**
```paradox
is_fully_controlled_by = GER
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert(
        "is_owned_by",
        HOI4Entity {
            name: "is_owned_by",
            description: r#"Checks if the current scope is owned by the specified country.

**Example:**
```paradox
is_owned_by = GER
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "is_claimed_by",
        HOI4Entity {
            name: "is_claimed_by",
            description: r#"Checks if the current scope is claimed by the specified country.

**Example:**
```paradox
is_claimed_by = GER
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "is_core_of",
        HOI4Entity {
            name: "is_core_of",
            description: r#"Checks if the current scope is a core of the specified country.

**Example:**
```paradox
is_core_of = GER
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert("is_owned_and_controlled_by", HOI4Entity {
        name: "is_owned_and_controlled_by",
        description: r#"Checks if the current scope is owned and controlled by the specified country.

**Example:**
```paradox
is_owned_and_controlled_by = GER
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert(
        "is_demilitarized_zone",
        HOI4Entity {
            name: "is_demilitarized_zone",
            description: r#"Checks if the current scope is a demilitarized zone.

**Example:**
```paradox
is_demilitarized_zone = yes
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "is_border_conflict",
        HOI4Entity {
            name: "is_border_conflict",
            description: r#"Checks if the current scope is part of a border war.

**Example:**
```paradox
is_border_conflict = yes
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert("is_in_home_area", HOI4Entity {
        name: "is_in_home_area",
        description: r#"Checks if the current scope is connected to the capital state over land. The scope needs to be owned as well for the statement for it to be true.

**Example:**
```paradox
is_in_home_area = yes
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert(
        "is_coastal",
        HOI4Entity {
            name: "is_coastal",
            description: r#"Checks if the current scope is a coastal state.

**Example:**
```paradox
is_coastal = yes
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert("is_one_state_island", HOI4Entity {
        name: "is_one_state_island",
        description: r#"Checks if the current scope is a coastal state with no adjacent land states.

**Example:**
```paradox
is_one_state_island = yes
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert("is_island_state", HOI4Entity {
        name: "is_island_state",
        description: r#"Checks if the current scope is a state where every province has no land neighbour.

**Example:**
```paradox
is_island_state = yes
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert(
        "is_on_continent",
        HOI4Entity {
            name: "is_on_continent",
            description: r#"Checks if the current scope is on the specified continent.

**Example:**
```paradox
is_on_continent = europe
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "impassable",
        HOI4Entity {
            name: "impassable",
            description: r#"Checks if the current scope is impassable.

**Example:**
```paradox
impassable = yes
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "has_state_category",
        HOI4Entity {
            name: "has_state_category",
            description: r#"Checks if the current scope has the specified category.

**Example:**
```paradox
has_state_category = rural
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "state_strategic_value",
        HOI4Entity {
            name: "state_strategic_value",
            description: r#"Checks if the current scope has the specified strategic value.

**Example:**
```paradox
state_strategic_value > 10
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert("state_and_terrain_strategic_value", HOI4Entity {
        name: "state_and_terrain_strategic_value",
        description: r#"Checks if the current scope has the specified state and terrain strategic value.

**Example:**
```paradox
state_and_terrain_strategic_value > 10
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert("num_owned_neighbour_states", HOI4Entity {
        name: "num_owned_neighbour_states",
        description: r#"Checks if the current scope has the specified amount of neighbor states belonging to the specified country.

**Example:**
```paradox
num_owned_neighbour_states = {
    owner = GER
    count > 2
}
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert("distance_to", HOI4Entity {
        name: "distance_to",
        description: r#"Checks if the current scope is at the specified distance from the specified state.

**Example:**
```paradox
distance_to = {
    value > 1000
    target = 49
}
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert("ships_in_area", HOI4Entity {
        name: "ships_in_area",
        description: r#"Checks if the current scope has the specified amount of ships in the specified strategic region.

**Example:**
```paradox
ships_in_area = { area = 104 size > 14 }
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert("has_resources_amount", HOI4Entity {
        name: "has_resources_amount",
        description: r#"Checks if the current scope has the specified amount of the specified resource.

**Example:**
```paradox
has_resources_amount = {
    resource = oil
    amount > 10
    delivered = yes
}
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert("days_since_last_strategic_bombing", HOI4Entity {
        name: "days_since_last_strategic_bombing",
        description: r#"Checks how many days have passed since the last strategic bombing of the state.

**Example:**
```paradox
days_since_last_strategic_bombing < 10
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert("has_railway_connection", HOI4Entity {
        name: "has_railway_connection",
        description: r#"Returns true if the states are connected by a railway. Can also check provinces.

**Example:**
```paradox
has_railway_connection = {
	start_state = 10
	target_state = 90
}
```

```paradox
has_railway_connection = {
	start_province = 402
	target_province = 9400
}
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert("can_build_railway", HOI4Entity {
        name: "can_build_railway",
        description: r#"Returns true if a railway can be built between states. Can also check for provinces.

**Example:**
```paradox
can_build_railway = {
	start_state = 10
	target_state = 90
}
```

```paradox
can_build_railway = {
	start_province = 402
	target_province = 9400
}
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert(
        "has_railway_level",
        HOI4Entity {
            name: "has_railway_level",
            description: r#"Checks if a state contains a railway at or above the specified level.

**Example:**
```paradox
has_railway_level = {
    	state = 114
    	level = 5
}
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert("pc_does_state_stack_demilitarized", HOI4Entity {
        name: "pc_does_state_stack_demilitarized",
        description: r#"Checks if the current scope was demilitarised during a current or previously-ended peace conference.

**Example:**
```paradox
pc_does_state_stack_demilitarized = yes
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert("pc_does_state_stack_dismantled", HOI4Entity {
        name: "pc_does_state_stack_dismantled",
        description: r#"Checks if the current scope was dismantled during a current or previously-ended peace conference.

**Example:**
```paradox
pc_does_state_stack_dismantled = yes
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert("pc_is_state_claimed", HOI4Entity {
        name: "pc_is_state_claimed",
        description: r#"Checks if the current scope was claimed by any country during the peace conference.

**Example:**
```paradox
pc_is_state_claimed = yes
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert("pc_is_state_claimed_by", HOI4Entity {
        name: "pc_is_state_claimed_by",
        description: r#"Checks if the current scope was claimed by the specified country during the peace conference.  Note, that "claim" in this context, while includes, is NOT limited to outright taking: forcing government, puppeting and liberating will render that trigger true as well. If one looks specifically for states taken by victors for themselves, pc_is_state_claimed_and_taken_by should be used.

**Example:**
```paradox
pc_is_state_claimed_by = BHR
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert("pc_is_state_claimed_and_taken_by", HOI4Entity {
        name: "pc_is_state_claimed_and_taken_by",
        description: r#"Checks if the current scope was claimed with "Take State" action (i.e. annexed) by the specified country during the peace conference.

**Example:**
```paradox
pc_is_state_claimed_and_taken_by = SOV
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert("pc_is_state_outside_influence_for_winner", HOI4Entity {
        name: "pc_is_state_outside_influence_for_winner",
        description: r#"Checks if the current state is outside of the influence of the specified winner country.

**Example:**
```paradox
pc_is_state_outside_influence_for_winner = ROOT
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert("pc_turn", HOI4Entity {
        name: "pc_turn",
        description: r#"Compares the amount of turns that have passed during the peace conference with a number.

**Example:**
```paradox
pc_turn > 20
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert("can_construct_building", HOI4Entity {
        name: "can_construct_building",
        description: r#"Checks if the country (as ROOT) and state in scope can build a building in the state.

**Example:**
```paradox
`can_construct_building = bunker`
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert(
        "compliance",
        HOI4Entity {
            name: "compliance",
            description: r#"Compares the compliance value of the current scope with the given value.

**Example:**
```paradox
compliance > 50
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "compliance_speed",
        HOI4Entity {
            name: "compliance_speed",
            description: r#"Compares the compliance speed of the current scope with the given value.

**Example:**
```paradox
compliance_speed > 50
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "has_active_resistance",
        HOI4Entity {
            name: "has_active_resistance",
            description: r#"Checks if the current scope has non-zero resistance.

**Example:**
```paradox
has_active_resistance = yes
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "has_resistance",
        HOI4Entity {
            name: "has_resistance",
            description: r#"Checks if the current scope has resistance.

**Example:**
```paradox
has_resistance = yes
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "resistance",
        HOI4Entity {
            name: "resistance",
            description: r#"Compares the resistance value of the current scope with the given value.

**Example:**
```paradox
resistance > 50
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "resistance_speed",
        HOI4Entity {
            name: "resistance_speed",
            description: r#"Compares the resistance speed of the current scope with the given value.

**Example:**
```paradox
resistance_speed > 50
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert("resistance_target", HOI4Entity {
        name: "resistance_target",
        description: r#"Compares the target resistance value of the current scope with the given value.

**Example:**
```paradox
resistance_target > 50
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert("has_occupation_modifier", HOI4Entity {
        name: "has_occupation_modifier",
        description: r#"Checks if the current scope has an occupation modifier, changing resistance/compliance.

**Example:**
```paradox
has_occupation_modifier = modifier_name
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert(
        "occupied_country_tag",
        HOI4Entity {
            name: "occupied_country_tag",
            description: r#"Checks which country creates resistance.

**Example:**
```paradox
occupied_country_tag = POL
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert("is_character", HOI4Entity {
        name: "is_character",
        description: r#"Checks if the current character's token matches up with the specified one.

**Example:**
```paradox
is_character = POL_test_character
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert("is_country_leader", HOI4Entity {
        name: "is_country_leader",
        description: r#"Checks if the character in the current scope is the active country leader.

**Example:**
```paradox
is_country_leader = yes
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert("is_unit_leader", HOI4Entity {
        name: "is_unit_leader",
        description: r#"Checks if the character in the current scope has an active unit leader (Army/Navy leader) role.

**Example:**
```paradox
is_unit_leader = yes
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert("is_advisor", HOI4Entity {
        name: "is_advisor",
        description: r#"Checks if the character in the current scope has an advisor role (includes advisors/theorists/high command).

**Example:**
```paradox
is_advisor = yes
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert("is_air_chief", HOI4Entity {
        name: "is_air_chief",
        description: r#"Checks if the character in the current scope is selected as an air chief.

**Example:**
```paradox
is_air_chief = yes
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert("is_army_chief", HOI4Entity {
        name: "is_army_chief",
        description: r#"Checks if the character in the current scope is selected as an army chief.

**Example:**
```paradox
is_army_chief = yes
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert("is_army_leader", HOI4Entity {
        name: "is_army_leader",
        description: r#"Checks if the character in the current scope has an army leader (General/Field Marshal) role.

**Example:**
```paradox
is_army_leader = yes
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert("is_navy_chief", HOI4Entity {
        name: "is_navy_chief",
        description: r#"Checks if the character in the current scope is selected as a navy chief.

**Example:**
```paradox
is_navy_chief = yes
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert("is_navy_leader", HOI4Entity {
        name: "is_navy_leader",
        description: r#"Checks if the character in the current scope has an navy leader (Admiral) role.

**Example:**
```paradox
is_navy_leader = yes
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert("is_high_command", HOI4Entity {
        name: "is_high_command",
        description: r#"Checks if the character in the current scope is selected as high command.

**Example:**
```paradox
is_high_command = yes
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert(
        "is_corps_commander",
        HOI4Entity {
            name: "is_corps_commander",
            description: r#"Checks if the character in the current scope is a corps commander.

**Example:**
```paradox
is_corps_commander = yes
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "is_operative",
        HOI4Entity {
            name: "is_operative",
            description: r#"Checks if the character in the current scope is an operative.

**Example:**
```paradox
is_operative = yes
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert("is_political_advisor", HOI4Entity {
        name: "is_political_advisor",
        description: r#"Checks if the character in the current scope is selected as a political advisor.

**Example:**
```paradox
is_political_advisor = yes
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert(
        "is_theorist",
        HOI4Entity {
            name: "is_theorist",
            description: r#"Checks if the character in the current scope is selected as a theorist.

**Example:**
```paradox
is_theorist = yes
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert("is_character_slot", HOI4Entity {
        name: "is_character_slot",
        description: r#"Checks if the character in the current scope has a role within the specified character slot

**Example:**
```paradox
is_character_slot = political_advisor
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert(
        "has_air_ledger",
        HOI4Entity {
            name: "has_air_ledger",
            description: r#"Checks if the character in the current scope has an air ledger.

**Example:**
```paradox
has_air_ledger = yes
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "has_army_ledger",
        HOI4Entity {
            name: "has_army_ledger",
            description: r#"Checks if the character in the current scope has an army ledger.

**Example:**
```paradox
has_army_ledger = yes
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "has_navy_ledger",
        HOI4Entity {
            name: "has_navy_ledger",
            description: r#"Checks if the character in the current scope has an navy ledger.

**Example:**
```paradox
has_navy_ledger = yes
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "has_character_flag",
        HOI4Entity {
            name: "has_character_flag",
            description: r#"Checks if the current scope has the specified flag.

**Example:**
```paradox
has_character_flag = my_flag
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "has_trait",
        HOI4Entity {
            name: "has_trait",
            description: r#"Checks if the current scope has the specified trait.

**Example:**
```paradox
has_trait = really_good_boss
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "has_id",
        HOI4Entity {
            name: "has_id",
            description: r#"Checks if the current character has the specificed ID.

**Example:**
```paradox
has_id = 1
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "is_hired_as_advisor",
        HOI4Entity {
            name: "is_hired_as_advisor",
            description: r#"Checks if the current character is activated as an advisor in any slot.

**Example:**
```paradox
is_hired_as_advisor = yes
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert("not_already_hired_except_as", HOI4Entity {
        name: "not_already_hired_except_as",
        description: r#"Checks if the current character is not hired, with the exception of the specified slot.

**Example:**
```paradox
not_already_hired_except_as = political_advisor
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert("advisor_can_be_fired", HOI4Entity {
        name: "advisor_can_be_fired",
        description: r#"Checks if the current character's `can_be_fired` attribute is set or not within a certain slot.

**Example:**
```paradox
advisor_can_be_fired = no
```

```paradox
advisor_can_be_fired = {
    slot = political_advisor
}
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert(
        "has_advisor_role",
        HOI4Entity {
            name: "has_advisor_role",
            description: r#"Checks if the character in scope has an advisor role for the given slot.

**Example:**
```paradox
has_advisor_role = political_advisor
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert("has_ideology", HOI4Entity {
        name: "has_ideology",
        description: r#"Checks if the current character has the specificed sub-ideology assigned.

**Example:**
```paradox
has_ideology = liberalism
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert(
        "has_ideology_group",
        HOI4Entity {
            name: "has_ideology_group",
            description: r#"Checks if the current character has the specificed ideology assigned.

**Example:**
```paradox
has_ideology_group = democratic
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "has_unit_leader_flag",
        HOI4Entity {
            name: "has_unit_leader_flag",
            description: r#"Checks if the current scope has the specified flag.

**Example:**
```paradox
has_unit_leader_flag = my_flag
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "is_leading_army",
        HOI4Entity {
            name: "is_leading_army",
            description: r#"Checks if the current scope is leading a single army.

**Example:**
```paradox
is_leading_army = yes
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "is_leading_army_group",
        HOI4Entity {
            name: "is_leading_army_group",
            description: r#"Checks if the current scope is leading an army group.

**Example:**
```paradox
is_leading_army_group = yes
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert("is_leading_volunteer_group", HOI4Entity {
        name: "is_leading_volunteer_group",
        description: r#"Checks if the current scope is leading a volunteer army within the specified country.

**Example:**
```paradox
is_leading_volunteer_group = POL
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert("is_leading_volunteer_group_with_original_country", HOI4Entity {
        name: "is_leading_volunteer_group_with_original_country",
        description: r#"Checks if the current scope is leading a volunteer army within a country of the specified original tag.

**Example:**
```paradox
is_leading_volunteer_group_with_original_country = POL
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert(
        "is_field_marshal",
        HOI4Entity {
            name: "is_field_marshal",
            description: r#"Checks if the current scope is a Field Marshal.

**Example:**
```paradox
is_field_marshal = yes
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "is_assigned",
        HOI4Entity {
            name: "is_assigned",
            description: r#"Checks if the current scope is an assigned unit leader.

**Example:**
```paradox
is_assigned = yes
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "can_select_trait",
        HOI4Entity {
            name: "can_select_trait",
            description: r#"Checks if the current scope can select the specified trait.

**Example:**
```paradox
can_select_trait = offensive_doctrine
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "has_ability",
        HOI4Entity {
            name: "has_ability",
            description: r#"Checks if the current scope has the specified unit leader ability.

**Example:**
```paradox
has_ability = glider_planes
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "skill",
        HOI4Entity {
            name: "skill",
            description: r#"Checks if the current scope has a Skill above the specified amount.

**Example:**
```paradox
skill > 1
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert("skill_advantage", HOI4Entity {
        name: "skill_advantage",
        description: r#"Checks if the current scope has a Skill advantage above the specified amount in against an enemy unit leader whilst in combat.

**Example:**
```paradox
skill_advantage > 1
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert("planning_skill_level", HOI4Entity {
        name: "planning_skill_level",
        description: r#"Checks if the current scope has a Planning skill above the specified amount.

**Example:**
```paradox
planning_skill_level > 1
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert("logistics_skill_level", HOI4Entity {
        name: "logistics_skill_level",
        description: r#"Checks if the current scope has a Logistics skill above the specified amount.

**Example:**
```paradox
logistics_skill_level > 1
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert("defense_skill_level", HOI4Entity {
        name: "defense_skill_level",
        description: r#"Checks if the current scope has a Defense skill above the specified amount.

**Example:**
```paradox
defense_skill_level > 1
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert("attack_skill_level", HOI4Entity {
        name: "attack_skill_level",
        description: r#"Checks if the current scope has a Attack skill above the specified amount.

**Example:**
```paradox
attack_skill_level > 1
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert("average_stats", HOI4Entity {
        name: "average_stats",
        description: r#"Checks if the current scope has an average skill above the specified amount.

**Example:**
```paradox
average_stats > 5
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert(
        "is_border_war",
        HOI4Entity {
            name: "is_border_war",
            description: r#"Checks if the current socpe is in a border war.

**Example:**
```paradox
is_border_war = yes
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert("num_units", HOI4Entity {
        name: "num_units",
        description: r#"Checks if the current scope is commanding the specified amount of divisions.

**Example:**
```paradox
num_units > 5
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert(
        "is_exiled_leader",
        HOI4Entity {
            name: "is_exiled_leader",
            description: r#"Checks if the current scope is a general from an exiled country.

**Example:**
```paradox
is_exiled_leader = yes
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert("is_exiled_leader_from", HOI4Entity {
        name: "is_exiled_leader_from",
        description: r#"Checks if the current scope is a general from the specified exiled country.

**Example:**
```paradox
is_exiled_leader_from = POL
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert("is_leading_army_in_province", HOI4Entity {
        name: "is_leading_army_in_province",
        description: r#"Checks if the current unit leader is leading an army that has any division in a specific province

**Example:**
```paradox
is_leading_army_in_province = 1234
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert(
        "has_nationality",
        HOI4Entity {
            name: "has_nationality",
            description: r#"Checks if the current operative has the nationality.

**Example:**
```paradox
has_nationality = POL
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "is_operative_captured",
        HOI4Entity {
            name: "is_operative_captured",
            description: r#"Checks if the current scope is captured.

**Example:**
```paradox
is_operative_captured = yes
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "operative_leader_mission",
        HOI4Entity {
            name: "operative_leader_mission",
            description: r#"Checks if the current scope is on the given mission.

**Example:**
```paradox
operative_leader_mission = mission_name
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "operative_leader_operation",
        HOI4Entity {
            name: "operative_leader_operation",
            description: r#"Checks if the current scope is on the given operation.

**Example:**
```paradox
operative_leader_operation = operation_name
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert("has_scientist_level", HOI4Entity {
        name: "has_scientist_level",
        description: r#"Checks if the scientist of the character in scope matches the skill level condition for a specialization. Supports < > = operators.

**Example:**
```paradox
has_scientist_level = {
  level > 2
  specialization = specialization_nuclear
}
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert("is_active_scientist", HOI4Entity {
        name: "is_active_scientist",
        description: r#"Checks if the scientist of the character in scope is assigned to a project.

**Example:**
```paradox
is_scientist_active = yes
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert(
        "is_scientist_injured",
        HOI4Entity {
            name: "is_scientist_injured",
            description: r#"Checks if the scientist of the character in scope is injured.

**Example:**
```paradox
is_scientist_injured = yes
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "hardness",
        HOI4Entity {
            name: "hardness",
            description: r#"Checks if the current scope has the specified amount of hardness.

**Example:**
```paradox
hardness > 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "armor",
        HOI4Entity {
            name: "armor",
            description: r#"Checks if the current scope has the specified amount of armor units.

**Example:**
```paradox
armor > 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "dig_in",
        HOI4Entity {
            name: "dig_in",
            description: r#"Checks if the current scope has the specified amount of Dig In bonus.

**Example:**
```paradox
dig_in > 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "min_planning",
        HOI4Entity {
            name: "min_planning",
            description: r#"Checks if the current scope has the specified amount of planning.

**Example:**
```paradox
min_planning > 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "fastest_unit",
        HOI4Entity {
            name: "fastest_unit",
            description: r#"Checks if the current scope has a unit with the specified speed.

**Example:**
```paradox
fastest_unit > 12
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("temperature", HOI4Entity {
        name: "temperature",
        description: r#"Checks if the current scope is in a province with a temperature above the specified amount.

**Example:**
```paradox
temperature > 20
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("reserves", HOI4Entity {
        name: "reserves",
        description: r#"Checks if the current scope has the specified amount of reserves waiting.

**Example:**
```paradox
reserves > 10
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "has_combat_modifier",
        HOI4Entity {
            name: "has_combat_modifier",
            description: r#"Checks if the current scope has the specified combat modifier.

**Example:**
```paradox
has_combat_modifier = river_crossing
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "is_fighting_in_terrain",
        HOI4Entity {
            name: "is_fighting_in_terrain",
            description: r#"Checks if the current scope is fighting in the specified terrain.

**Example:**
```paradox
is_fighting_in_terrain = desert
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "is_fighting_in_weather",
        HOI4Entity {
            name: "is_fighting_in_weather",
            description: r#"Checks if the current scope is fighting in the specified weather.

**Example:**
```paradox
is_fighting_in_weather = sandstorm
```

```paradox
is_fighting_in_weather = { rain_light rain_heavy }
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "phase",
        HOI4Entity {
            name: "phase",
            description: r#"Checks if the current scope is in phase.

**Example:**
```paradox
phase = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "recon_advantage",
        HOI4Entity {
            name: "recon_advantage",
            description: r#"Checks if the current scope has x recon advantage.

**Example:**
```paradox
recon_advantage > 0
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "night",
        HOI4Entity {
            name: "night",
            description: r#"Checks if the current scope is fighting at night.

**Example:**
```paradox
night = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "frontage_full",
        HOI4Entity {
            name: "frontage_full",
            description: r#"Checks if the current scope has a full combat width.

**Example:**
```paradox
frontage_full = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "has_flanked_opponent",
        HOI4Entity {
            name: "has_flanked_opponent",
            description: r#"Checks if the current scope has flanked their opponent.

**Example:**
```paradox
has_flanked_opponent = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "has_max_planning",
        HOI4Entity {
            name: "has_max_planning",
            description: r#"Checks if the current scope has the maximum planning bonus.

**Example:**
```paradox
has_max_planning = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "has_reserves",
        HOI4Entity {
            name: "has_reserves",
            description: r#"Checks if the current scope has any reserves waiting.

**Example:**
```paradox
has_reserves = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "is_amphibious_invasion",
        HOI4Entity {
            name: "is_amphibious_invasion",
            description: r#"Checks if the current scope is performing an amphibious invasion.

**Example:**
```paradox
is_amphibious_invasion = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "is_attacker",
        HOI4Entity {
            name: "is_attacker",
            description: r#"Checks if the current scope is attacking.

**Example:**
```paradox
is_attacker = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "is_defender",
        HOI4Entity {
            name: "is_defender",
            description: r#"Checks if the current scope is defending.

**Example:**
```paradox
is_defender = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "is_winning",
        HOI4Entity {
            name: "is_winning",
            description: r#"Checks if the current scope is winning their battle.

**Example:**
```paradox
is_winning = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "is_fighting_air_units",
        HOI4Entity {
            name: "is_fighting_air_units",
            description: r#"Checks if the current scope is fighting air units.

**Example:**
```paradox
is_fighting_air_units = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("less_combat_width_than_opponent", HOI4Entity {
        name: "less_combat_width_than_opponent",
        description: r#"Checks if the current scope is fighting with less combat width than their opponent.

**Example:**
```paradox
less_combat_width_than_opponent = yes
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "has_carrier_airwings_on_mission",
        HOI4Entity {
            name: "has_carrier_airwings_on_mission",
            description: r#"Checks if the current scope has carrier airwings on a mission.

**Example:**
```paradox
has_carrier_airwings_on_mission = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "has_carrier_airwings_in_own_combat",
        HOI4Entity {
            name: "has_carrier_airwings_in_own_combat",
            description: r#"Checks if the current scope has carrier airwings in their own combat.

**Example:**
```paradox
has_carrier_airwings_in_own_combat = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("has_artillery_ratio", HOI4Entity {
        name: "has_artillery_ratio",
        description: r#"Check that ratio of atrillery battalions in the composition of a side of combating troops are over a certain level.

**Example:**
```paradox
has_artillery_ratio > 0.1
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "has_unit_type",
        HOI4Entity {
            name: "has_unit_type",
            description: r#"Check if the combatant has at least one of the provided unit types.

**Example:**
```paradox
has_unit_type = amphibious_mechanized
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("division_has_majority_template", HOI4Entity {
        name: "division_has_majority_template",
        description: r#"Checks if the current scope is majority made up of the specified battalion.

**Example:**
```paradox
division_has_majority_template = light_armor
```"#,
        scopes: &[crate::scope::Scope::Unit],
    });
    m.insert("division_has_battalion_in_template", HOI4Entity {
        name: "division_has_battalion_in_template",
        description: r#"Checks if the current scope has any battalions of the type in the template.

**Example:**
```paradox
division_has_battalion_in_template = light_armor
```"#,
        scopes: &[crate::scope::Scope::Unit],
    });
    m.insert(
        "unit_strength",
        HOI4Entity {
            name: "unit_strength",
            description: r#"Checks the current strength of the unit on the scale from 0 to 1.

**Example:**
```paradox
unit_strength < 0.3
```"#,
            scopes: &[crate::scope::Scope::Unit],
        },
    );
    m.insert(
        "unit_organization",
        HOI4Entity {
            name: "unit_organization",
            description: r#"Checks the current organisation of the unit on the scale from 0 to 1.

**Example:**
```paradox
unit_organization < 0.3
```"#,
            scopes: &[crate::scope::Scope::Unit],
        },
    );
    m.insert("is_unit_template_reserves", HOI4Entity {
        name: "is_unit_template_reserves",
        description: r#"Checks if the current division has the supply priority set to 'Reserves', i.e. the lowest priority.

**Example:**
```paradox
is_unit_template_reserves = yes
```"#,
        scopes: &[crate::scope::Scope::Unit],
    });
    m.insert("has_officer_name", HOI4Entity {
        name: "has_officer_name",
        description: r#"Checks if the current division has an officer with the provided name key.

**Example:**
```paradox
has_officer_name = FIN_nikke_parmi
```"#,
        scopes: &[crate::scope::Scope::Unit],
    });
    m.insert(
        "is_military_industrial_organization",
        HOI4Entity {
            name: "is_military_industrial_organization",
            description: r#"Checks if the currently-scoped MIO matches the input token.

**Example:**
```paradox
is_military_industrial_organization = my_mio_token
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "is_mio_visible",
        HOI4Entity {
            name: "is_mio_visible",
            description: r#"Checks if the currently-scoped MIO is visible.

**Example:**
```paradox
is_mio_visible = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "is_mio_available",
        HOI4Entity {
            name: "is_mio_available",
            description: r#"Checks if the currently-scoped MIO is visible.

**Example:**
```paradox
is_mio_available = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "is_mio_assigned_to_task",
        HOI4Entity {
            name: "is_mio_assigned_to_task",
            description: r#"Checks if the currently-scoped MIO is assigned to a task.

**Example:**
```paradox
is_mio_assigned_to_task = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "has_mio_size",
        HOI4Entity {
            name: "has_mio_size",
            description: r#"Checks the size of the MIO.

**Example:**
```paradox
has_mio_size > 3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "has_mio_trait",
        HOI4Entity {
            name: "has_mio_trait",
            description: r#"Checks whether the MIO has the target trait in its list.

**Example:**
```paradox
has_mio_trait = my_trait_token
```

```paradox
has_mio_trait = {
    token = my_trait_token
}
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("is_mio_trait_available", HOI4Entity {
        name: "is_mio_trait_available",
        description: r#"Checks whether the MIO has the target trait in its list and whether it's available.

**Example:**
```paradox
is_mio_trait_available = my_trait_token
```

```paradox
is_mio_trait_available = {
    token = my_trait_token
    check_mio_parent_completed = no
}
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("is_mio_trait_completed", HOI4Entity {
        name: "is_mio_trait_completed",
        description: r#"Checks whether the MIO has the target trait in its list and whether it's completed.

**Example:**
```paradox
is_mio_trait_completed = my_trait_token
```

```paradox
is_mio_trait_completed = {
    token = my_trait_token
}
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "has_mio_number_of_completed_traits",
        HOI4Entity {
            name: "has_mio_number_of_completed_traits",
            description: r#"Checks the amount of unlocked MIO traits.

**Example:**
```paradox
has_mio_number_of_completed_traits < 2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "has_mio_flag",
        HOI4Entity {
            name: "has_mio_flag",
            description: r#"Checks if the current scope has the specified flag.

**Example:**
```paradox
has_mio_flag = my_flag
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "has_mio_policy",
        HOI4Entity {
            name: "has_mio_policy",
            description: r#"Checks if the currently-scoped MIO has the target policy allowed.

**Example:**
```paradox
has_mio_policy = my_policy_token
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "has_mio_policy_active",
        HOI4Entity {
            name: "has_mio_policy_active",
            description: r#"Checks if the currently-scoped MIO has the target policy active.

**Example:**
```paradox
has_mio_policy_active = my_policy_token
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "has_mio_research_category",
        HOI4Entity {
            name: "has_mio_research_category",
            description: r#"Checks if the currently-scoped MIO has the target research category.

**Example:**
```paradox
has_mio_research_category = my_research_category_token
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "has_mio_equipment_type",
        HOI4Entity {
            name: "has_mio_equipment_type",
            description: r#"Checks if the currently-scoped MIO has the target equipment types.

**Example:**
```paradox
has_mio_equipment_type = my_equipment_type_token
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("contract_contains_equipment", HOI4Entity {
        name: "contract_contains_equipment",
        description: r#"Checks if the currently-scoped purchase contract contains an equipment type.

**Example:**
```paradox
contract_contains_equipment = infantry_equipment
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "deal_completion",
        HOI4Entity {
            name: "deal_completion",
            description: r#"Checks the deal completion with the target value.

**Example:**
```paradox
deal_completition > 0.6
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "seller",
        HOI4Entity {
            name: "seller",
            description: r#"Checks the seller in the current purchase contract.

**Example:**
```paradox
seller = BHR
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "buyer",
        HOI4Entity {
            name: "buyer",
            description: r#"Checks the buyer in the current purchase contract.

**Example:**
```paradox
buyer = OMA
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("has_project_flag", HOI4Entity {
        name: "has_project_flag",
        description: r#"Check if flag has been set within the special project in scope. May checks on the value or date/days since last modified date.

**Example:**
```paradox
has_project_flag = my_flag
```

```paradox
has_project_flag = {
  flag = my_flag
  value < 12
  date > 1936.3.25
  days > 365
}
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "can_ROOT_get_wargoal_on_THIS",
        HOI4Entity {
            name: "can_ROOT_get_wargoal_on_THIS",
            description: r#"Checks if ROOT can obtain a wargoal on the current scope.

**Example:**
```paradox
can_ROOT_get_wargoal_on_THIS = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "is_free_or_subject_of_root",
        HOI4Entity {
            name: "is_free_or_subject_of_root",
            description: r#"Checks if the current scope is either independent or a subject of ROOT.

**Example:**
```paradox
is_free_or_subject_of_root = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "has_same_ideology",
        HOI4Entity {
            name: "has_same_ideology",
            description: r#"Checks if the current scope has the same ideology as ROOT.

**Example:**
```paradox
has_same_ideology = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("is_enemy_ideology", HOI4Entity {
        name: "is_enemy_ideology",
        description: r#"Checks if the current scope has an ideology that is considered enemy to ROOT's.

**Example:**
```paradox
is_enemy_ideology = yes
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "has_ROOT_at_least_1_div_in_current_state_scope",
        HOI4Entity {
            name: "has_ROOT_at_least_1_div_in_current_state_scope",
            description: r#"Checks if ROOT has at least one division in the current scope.

**Example:**
```paradox
has_ROOT_at_least_1_div_in_current_state_scope = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "controls_or_subject_of",
        HOI4Entity {
            name: "controls_or_subject_of",
            description: r#"Checks if the current state is controlled by ROOT or a subject of ROOT.

**Example:**
```paradox
controls_or_subject_of = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("is_controlled_by_ROOT_or_ally", HOI4Entity {
        name: "is_controlled_by_ROOT_or_ally",
        description: r#"Checks if the current state is controlled by ROOT, a subject of ROOT, or a country in the same faction as ROOT.

**Example:**
```paradox
is_controlled_by_ROOT_or_ally = yes
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "owns_or_subject_of",
        HOI4Entity {
            name: "owns_or_subject_of",
            description: r#"Checks if the current scope is owned by ROOT or a subject of ROOT.

**Example:**
```paradox
owns_or_subject_of = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m
}

pub fn get_effects() -> HashMap<&'static str, HOI4Entity> {
    let mut m = HashMap::new();
    m.insert("every_possible_country", HOI4Entity {
        name: "every_possible_country",
        description: r#"Executes children effects on every country that meets the limit, including those that do not exist.

**Example:**
```paradox
`every_possible_country = { ... }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "every_country",
        HOI4Entity {
            name: "every_country",
            description: r#"Executes contained effects on every country that meets the limit.

**Example:**
```paradox
`every_country = { … }`
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "random_country",
        HOI4Entity {
            name: "random_country",
            description: r#"Executes contained effects on a random country that meets the limit.

**Example:**
```paradox
`random_country = { … }`
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("every_other_country", HOI4Entity {
        name: "every_other_country",
        description: r#"Executes contained effects on every country that meets the limit and is not the same country as the one this is contained in.

**Example:**
```paradox
`every_other_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("random_other_country", HOI4Entity {
        name: "random_other_country",
        description: r#"Executes contained effects on a random country that meets the limit and is not the same country as the one this is contained in.

**Example:**
```paradox
`random_other_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("every_country_with_original_tag", HOI4Entity {
        name: "every_country_with_original_tag",
        description: r#"Executes contained effects on every country that meets the limit and has the specified original tag.

**Example:**
```paradox
every_country_with_original_tag = {
    original_tag_to_check = TAG  #required
    …                  #effects to run
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("random_country_with_original_tag", HOI4Entity {
        name: "random_country_with_original_tag",
        description: r#"Executes contained effects on a random country that meets the limit and has the specified original tag.

**Example:**
```paradox
random_country_with_original_tag = {
    original_tag_to_check = TAG  #required
    …                  #effects to run
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("every_neighbor_country", HOI4Entity {
        name: "every_neighbor_country",
        description: r#"Executes contained effects on every country that meets the limit and borders the country this is contained in.

**Example:**
```paradox
`every_neighbor_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("random_neighbor_country", HOI4Entity {
        name: "random_neighbor_country",
        description: r#"Executes contained effects on a random country that meets the limit and borders the country this is contained in.

**Example:**
```paradox
`random_neighbor_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("every_occupied_country", HOI4Entity {
        name: "every_occupied_country",
        description: r#"Executes contained effects on every country that meets the limit and has any core states controlled by the country this is contained in.

**Example:**
```paradox
`every_occupied_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("random_occupied_country", HOI4Entity {
        name: "random_occupied_country",
        description: r#"Executes contained effects on a random country that meets the limit and has any core states controlled by the country this is contained in.

**Example:**
```paradox
`random_occupied_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("every_allied_country", HOI4Entity {
        name: "every_allied_country",
        description: r#"Executes children effects on every Allied Country different from the one in scope (or `random_select_amount` of random country if specified) that fulfills the `limit` trigger.

**Example:**
```paradox
`every_allied_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("random_allied_country", HOI4Entity {
        name: "random_allied_country",
        description: r#"Executes children effects on a random Allied Country different from the one in scope that fulfills the `limit` trigger.

**Example:**
```paradox
`random_allied_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("every_enemy_country", HOI4Entity {
        name: "every_enemy_country",
        description: r#"Executes contained effects on every country that meets the limit and is at war with the country this is contained in.

**Example:**
```paradox
`every_enemy_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("random_enemy_country", HOI4Entity {
        name: "random_enemy_country",
        description: r#"Executes contained effects on a random country that meets the limit and is at war with the country this is contained in.

**Example:**
```paradox
`random_enemy_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("every_subject_country", HOI4Entity {
        name: "every_subject_country",
        description: r#"Executes contained effects on every country that meets the limit and is a subject of the country this is contained in.

**Example:**
```paradox
`every_subject_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("random_subject_country", HOI4Entity {
        name: "random_subject_country",
        description: r#"Executes contained effects on a random country that meets the limit and is a subject of the country this is contained in.

**Example:**
```paradox
`random_subject_country = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("every_faction_member", HOI4Entity {
        name: "every_faction_member",
        description: r#"Executes children effects on every faction member of the country's faction in scope, if country does not have a faction it will only work on itself.

**Example:**
```paradox
`every_faction_member = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "every_state",
        HOI4Entity {
            name: "every_state",
            description: r#"Executes contained effects on every state that meets the limit.

**Example:**
```paradox
`every_state = { … }`
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "random_state",
        HOI4Entity {
            name: "random_state",
            description: r#"Executes contained effects on a random state that meets the limit.

**Example:**
```paradox
random_state = {
    prioritize = { 123 321 } #optional
    …    #effects to run
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("every_neighbor_state", HOI4Entity {
        name: "every_neighbor_state",
        description: r#"Executes contained effects on every state that meets the limit and neighbours the state this is contained in.

**Example:**
```paradox
`every_neighbor_state = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("random_neighbor_state", HOI4Entity {
        name: "random_neighbor_state",
        description: r#"Executes contained effects on a random state that meets the limit and neighbours the state this is contained in. Does not support prioritizing.

**Example:**
```paradox
`random_neighbor_state = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("every_owned_state", HOI4Entity {
        name: "every_owned_state",
        description: r#"Executes contained effects on every state that meets the limit and is owned by the country this is contained in.

**Example:**
```paradox
`every_owned_state = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("random_owned_state", HOI4Entity {
        name: "random_owned_state",
        description: r#"Executes contained effects on a random state that meets the limit and is owned by the country this is contained in.

**Example:**
```paradox
random_owned_state = {
    prioritize = { 123 321 } #optional
    …    #effects to run
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("every_core_state", HOI4Entity {
        name: "every_core_state",
        description: r#"Executes contained effects on every state that meets the limit and is a core of the country this is contained in.

**Example:**
```paradox
`every_core_state = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("random_core_state", HOI4Entity {
        name: "random_core_state",
        description: r#"Executes contained effects on a random state that meets the limit and is a core of the country this is contained in.

**Example:**
```paradox
random_core_state = {
    prioritize = { 123 321 } #optional
    …    #effects to run
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("every_controlled_state", HOI4Entity {
        name: "every_controlled_state",
        description: r#"Executes contained effects on every state that meets the limit and is controlled by the country this is contained in.

**Example:**
```paradox
`every_controlled_state = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("random_controlled_state", HOI4Entity {
        name: "random_controlled_state",
        description: r#"Executes contained effects on a random state that meets the limit and is controlled by the country this is contained in.

**Example:**
```paradox
random_controlled_state = {
    prioritize = { 123 321 } #optional
    …    #effects to run
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("random_owned_controlled_state", HOI4Entity {
        name: "random_owned_controlled_state",
        description: r#"Executes contained effects on a random state that meets the limit and is owned and controlled by the country this is contained in.

**Example:**
```paradox
random_owned_controlled_state = {
    prioritize = { 123 321 } #optional
    …    #effects to run
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("every_unit_leader", HOI4Entity {
        name: "every_unit_leader",
        description: r#"Executes contained effects on every unit leader (corps commanders, field marshals, admirals) that meets the limit and is recruited by the country this is contained in.

**Example:**
```paradox
`every_unit_leader = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("random_unit_leader", HOI4Entity {
        name: "random_unit_leader",
        description: r#"Executes contained effects on a random unit leader (corps commanders, field marshals, admirals) that meets the limit and is recruited by the country this is contained in.

**Example:**
```paradox
`random_unit_leader = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("every_army_leader", HOI4Entity {
        name: "every_army_leader",
        description: r#"Executes contained effects on every army leader that meets the limit and is recruited by the country this is contained in.

**Example:**
```paradox
`every_unit_leader = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("random_army_leader", HOI4Entity {
        name: "random_army_leader",
        description: r#"Executes contained effects on a random army leader that meets the limit and is recruited by the country this is contained in.

**Example:**
```paradox
`random_army_leader = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("global_every_army_leader", HOI4Entity {
        name: "global_every_army_leader",
        description: r#"Executes contained effects on every army leader that meets the limit. Preferable to use every_army_leader unless necessary to use global_every_army_leader.

**Example:**
```paradox
`global_every_army_leader = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("every_navy_leader", HOI4Entity {
        name: "every_navy_leader",
        description: r#"Executes contained effects on every navy leader that meets the limit and is recruited by the country this is contained in.

**Example:**
```paradox
`every_navy_leader = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("random_navy_leader", HOI4Entity {
        name: "random_navy_leader",
        description: r#"Executes contained effects on a random navy leader that meets the limit and is recruited by the country this is contained in.

**Example:**
```paradox
`random_navy_leader = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("every_operative", HOI4Entity {
        name: "every_operative",
        description: r#"Executes contained effects on every operative that meets the limit and is recruited by the country this is contained in.

**Example:**
```paradox
`every_operative = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("random_operative", HOI4Entity {
        name: "random_operative",
        description: r#"Executes contained effects on a random operative that meets the limit and is recruited by the country this is contained in.

**Example:**
```paradox
`random_operative = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("every_character", HOI4Entity {
        name: "every_character",
        description: r#"Executes contained effects on every character that meets the limit and is recruited by the country this is contained in.

**Example:**
```paradox
`every_character = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("random_character", HOI4Entity {
        name: "random_character",
        description: r#"Executes contained effects on a random character that meets the limit and is recruited by the country this is contained in.

**Example:**
```paradox
`random_character = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("every_country_division", HOI4Entity {
        name: "every_country_division",
        description: r#"Executes contained effects on every division that meets the limit and is owned by the current country.

**Example:**
```paradox
`every_country_division = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("random_country_division", HOI4Entity {
        name: "random_country_division",
        description: r#"Executes contained effects on a random division that meets the limit and is owned by the current country.

**Example:**
```paradox
`random_country_division = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("every_state_division", HOI4Entity {
        name: "every_state_division",
        description: r#"Executes contained effects on every division that meets the limit and is located within the current state.

**Example:**
```paradox
`every_state_division = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("random_state_division", HOI4Entity {
        name: "random_state_division",
        description: r#"Executes contained effects on a random division that meets the limit and is located within the current state.

**Example:**
```paradox
`random_state_division = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("every_military_industrial_organization", HOI4Entity {
        name: "every_military_industrial_organization",
        description: r#"Executes contained effects on every MIO within the current country that meets the limit.

**Example:**
```paradox
`every_military_industrial_organization = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("random_military_industrial_organization", HOI4Entity {
        name: "random_military_industrial_organization",
        description: r#"Executes contained effects on a random MIO within the current country that meets the limit.

**Example:**
```paradox
`random_military_industrial_organization = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("every_purchase_contract", HOI4Entity {
        name: "every_purchase_contract",
        description: r#"Executes contained effects on every purchase contract within the current country that meets the limit.

**Example:**
```paradox
`every_purchase_contract = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("random_purchase_contract", HOI4Entity {
        name: "random_purchase_contract",
        description: r#"Executes contained effects on a random purchase contract within the current country that meets the limit.

**Example:**
```paradox
`random_purchase_contract = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("every_scientist", HOI4Entity {
        name: "every_scientist",
        description: r#"Executes children effects on every scientist (or "random_select_amount" of random character if specified) of the country in scope, that fulfills the "limit" trigger.

**Example:**
```paradox
`every_scientist = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("random_scientist", HOI4Entity {
        name: "random_scientist",
        description: r#"Executes children effects on random scientists that fulfills the "limit" trigger.

**Example:**
```paradox
`random_scientist = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("every_active_scientist", HOI4Entity {
        name: "every_active_scientist",
        description: r#"Executes children effects on every active scientist (or "random_select_amount" of random character if specified) of the country in scope, that fulfills the "limit" trigger.title.

**Example:**
```paradox
`every_active_scientist = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("random_active_scientist", HOI4Entity {
        name: "random_active_scientist",
        description: r#"Executes children effects on random scientists that fulfills the "limit" trigger.

**Example:**
```paradox
`random_active_scientist = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("party_leader", HOI4Entity {
        name: "party_leader",
        description: r#"Executes the effects on the party leader with the specified ideology type. Must contain a `has_ideology` in the limit that refers to a specific ideology type (e.g. Despotic), not a group that contain the type (e.g. Non-Aligned). The selected character must be the leader of a party corresponding to the ideology group.

**Example:**
```paradox
party_leader = {
    limit = {
        has_ideology = liberalism
    }
    set_nationality = BHR
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("every_collection_element", HOI4Entity {
        name: "every_collection_element",
        description: r#"Applies arbitrary effects to all elements of a collection. To learn more about collections, see the documentation in /Hearts of Iron IV/common/collections.

**Example:**
```paradox
every_collection_element = {
    input = {
        input = collection_id # This can be a collection name or an inline definition of a collection
        limit = {
            # Trigger - limit effect execution to a subset of elements
        }
    }
    # Effects to be executed
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("TAG", HOI4Entity {
        name: "TAG",
        description: r#"The country defined by the tag or tag alias. Tag aliases are defined in /Hearts of Iron IV/common/country_tag_aliases, as a way to refer to a specific country (such as a side in a civil war) in addition to its actual tag. If the country with the exact tag doesn't exist, but a dynamic country originating from the specified tag does, the scope will refer to the dynamic country.

**Example:**
```paradox
`SOV = { country_event = my_event.1 }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("ROOT", HOI4Entity {
        name: "ROOT",
        description: r#"Targets the root node of the block, an inherent property of each block. Most commonly, this is the default scope: for example, ROOT within a national focus will always refer to the country doing the focus and ROOT within a event will always refer to the country getting the event. However, some blocks do distinguish between the default scope and ROOT, such as certain scripted GUI contexts or certain on actions. If a block doesn't have ROOT defined (such as on_startup in on actions), then it is impossible to use it.

**Example:**
```paradox
ENG = {
    FRA = {
        GER = {
            declare_war_on = {
                target = ROOT
                type = annex_everything
            }
        }
    }
} #GER declares war on ENG (if there is no scope before ENG)
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("THIS", HOI4Entity {
        name: "THIS",
        description: r#"Targets the current scope where it's used. For example, when used in every_state, it will refer to the state that's currently being evaluated. Primarily useful for variables (as in the example, where omitting it wouldn't work) or for built-in localisation commands, where some scope must be specified. More rarely, this may help with scope manipulation when using PREV. Since omitting it makes no difference in how the code gets interpreted, there is little to no usage outside of these cases.

**Example:**
```paradox
set_temp_variable = { target_country = THIS }
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("PREV", HOI4Entity {
        name: "PREV",
        description: r#"Targets the scope that the current scope is contained in. Can have additional applications where the assumed default scope differs from the ROOT, such as in state events or some on_actions. Can be chained indefinitely as PREV.PREV. **Commonly results in broken-looking tooltips**: what's shown to the player doesn't always correlate with reality.

See also: PREV usage.

**Example:**
```paradox
FRA = {
    random_country = {
        GER = {
            declare_war_on = {
                target = PREV
                type = annex_everything
            }
        }
    }
} #Germany declares war on random_country
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("FROM", HOI4Entity {
        name: "FROM",
        description: r#"Can be chained indefinitely as FROM.FROM. Used to target various hardcoded scopes inherent to the block, often a secondary scope in addition to ROOT. For example:

In events, this refers to the country that sent the event (i.e. if the event was fired using an effect, then it's the ROOT scope where it was fired).
 In targeted decisions or diplomacy scripted triggers, this refers to the scope that is targeted.

**Example:**
```paradox
declare_war_on = {
    target = FROM
    type = annex_everything
}

FROM = {
    load_oob = defend_ourselves
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("overlord", HOI4Entity {
        name: "overlord",
        description: r#"The overlord of the country if it is a subject. Subject to the 'invalid event target' error.

**Example:**
```paradox
`overlord = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("faction_leader", HOI4Entity {
        name: "faction_leader",
        description: r#"Faction leader of the faction the country is a part of. Subject to the 'invalid event target' error.

**Example:**
```paradox
`faction_leader = { add_to_faction = FROM }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("owner", HOI4Entity {
        name: "owner",
        description: r#"In state scope, the country that owns the state. In combatant scope, the country that owns the divisions. In character scope, the country that has recruited the character. Subject to the 'invalid event target' error when used for a state.

**Example:**
```paradox
`owner = { add_ideas = owns_this_state }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("controller", HOI4Entity {
        name: "controller",
        description: r#"The controller of the current state. Subject to the 'invalid event target' error.

**Example:**
```paradox
controller = {
    ROOT = {
        create_wargoal = {
            target = PREV
            type = take_state_focus
            generator = { 123 }
        }
    }
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("capital_scope", HOI4Entity {
        name: "capital_scope",
        description: r#"The state where the capital of the current country is located in. Subject to the 'invalid event target' error in rare cases.

**Example:**
```paradox
`capital_scope = { … }`
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("add_dynamic_modifier", HOI4Entity {
        name: "add_dynamic_modifier",
        description: r#"Adds a dynamic modifier to the specified scope (the default scope is ROOT).
It will be updated daily, unless forced to update early by force_update_dynamic_modifier effect.

**Example:**
```paradox
add_dynamic_modifier = {
    modifier = example_dynamic_modifier
    scope = GER
    days = 14
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "remove_dynamic_modifier",
        HOI4Entity {
            name: "remove_dynamic_modifier",
            description: r#"Removes a dynamic modifier from the current scope

**Example:**
```paradox
remove_dynamic_modifier = { modifier = sabotaged_ressources }
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("force_update_dynamic_modifier", HOI4Entity {
        name: "force_update_dynamic_modifier",
        description: r#"Forces an update to the effects given by variables within dynamic modifiers.

**Example:**
```paradox
force_update_dynamic_modifier = yes
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("add_state_resistance_compliance_modifier", HOI4Entity {
        name: "add_state_resistance_compliance_modifier",
        description: r#"Adds either a resistance or compliance modifier to a state. Can only use modifiers from the /Hearts of Iron IV/common/resistance_modifiers.txt/compliance_modifiers.txt that are marked as `is_dynamic = yes`

**Example:**
```paradox
add_state_resistance_compliance_modifier  = {
       modifier = dynamic_modifier_name
	   state = 738
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("remove_state_resistance_compliance_modifier", HOI4Entity {
        name: "remove_state_resistance_compliance_modifier",
        description: r#"Removes either a resistance or compliance modifier from a state. Can only use modifiers from the /Hearts of Iron IV/common/resistance_modifiers.txt/compliance_modifiers.txt that are marked as `is_dynamic = yes`

**Example:**
```paradox
remove_state_resistance_compliance_modifier  = {
       modifier = dynamic_modifier_name
	   state = 738
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "set_global_flag",
        HOI4Entity {
            name: "set_global_flag",
            description: r#"Defines a global flag.

**Example:**
```paradox
set_global_flag = my_flag
```

```paradox
set_global_flag = {
    flag = my_flag
    days = 123
    value = 1
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "play_song",
        HOI4Entity {
            name: "play_song",
            description: r#"Plays an audio track

**Example:**
```paradox
play_song = "general_peace_1"
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "clr_global_flag",
        HOI4Entity {
            name: "clr_global_flag",
            description: r#"Clears a defined global flag.

**Example:**
```paradox
clr_global_flag = my_flag
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "modify_global_flag",
        HOI4Entity {
            name: "modify_global_flag",
            description: r#"Adds an integer value to a flag.

**Example:**
```paradox
modify_global_flag = {
    flag = my_flag
    value = 3
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "custom_effect_tooltip",
        HOI4Entity {
            name: "custom_effect_tooltip",
            description: r#"Displays a localized key in the effect tooltip.

**Example:**
```paradox
custom_effect_tooltip = my_tooltip_tt
```

```paradox
custom_effect_tooltip = {
    localization_key = my_loc
    NESTEDLOC = myotherloc/string
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("custom_override_tooltip", HOI4Entity {
        name: "custom_override_tooltip",
        description: r#"Executes the provided effects but with a custom tooltip surpressing all tooltips from all other effects inside this block.

**Example:**
```paradox
custom_override_tooltip= {
    tooltip = my_tt
    not_tooltip = my_tt_NOT
    <effects>
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "effect_tooltip",
        HOI4Entity {
            name: "effect_tooltip",
            description: r#"Displays the effects in the tooltip without executing them.

**Example:**
```paradox
effect_tooltip = {
    declare_war_on = {
        target = FRA
    }
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("log", HOI4Entity {
        name: "log",
        description: r#"Displays a string in the user directory's /Hearts of Iron IV/logs/game.log file when executed, as well as showing up in the console if it is open when the logging effect was executed.

**Example:**
```paradox
log = "myVariable: [?myVariable]"
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("save_event_target_as", HOI4Entity {
        name: "save_event_target_as",
        description: r#"Saves the current scope as a key. Is cleared once execution ends (i.e. end of event).

**Example:**
```paradox
capital_scope = {
    save_event_target_as = my_state
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("save_global_event_target_as", HOI4Entity {
        name: "save_global_event_target_as",
        description: r#"Saves the current scope as a key. Persists after execution until cleared via effect.

**Example:**
```paradox
random_other_country = {
    save_global_event_target_as = my_country
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "clear_global_event_target",
        HOI4Entity {
            name: "clear_global_event_target",
            description: r#"Clears a specific global event target.

**Example:**
```paradox
clear_global_event_target = my_country
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "clear_global_event_targets",
        HOI4Entity {
            name: "clear_global_event_targets",
            description: r#"Clears all global event targets.

**Example:**
```paradox
clear_global_event_targets = yes
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "sound_effect",
        HOI4Entity {
            name: "sound_effect",
            description: r#"Plays the specified sound once.

**Example:**
```paradox
sound_effect = "boom"
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "randomize_weather",
        HOI4Entity {
            name: "randomize_weather",
            description: r#"Randomizes the weather with the specified seed.

**Example:**
```paradox
randomize_weather = 12345
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("set_province_name", HOI4Entity {
        name: "set_province_name",
        description: r#"Changes the specified province/victory point's name to the specified name.

**Example:**
```paradox
set_province_name = {
    id = 325
    name = LOC_KEY
}
```

```paradox
set_province_name = { id = 325 name = "New Name" }
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "reset_province_name",
        HOI4Entity {
            name: "reset_province_name",
            description: r#"Resets the specified province's name.

**Example:**
```paradox
reset_province_name = 325
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "damage_units",
        HOI4Entity {
            name: "damage_units",
            description: r#"Damages units in the specified area.

**Example:**
```paradox
damage_units = {
    province = 42
    state = 5
    region = 5
    limit = { has_country_flag = TAG_test }
    damage = 0.5
    org_damage = 0.5
    str_damage = 0.5
    ratio = yes
    template = "template_name"
    army = no
    navy = yes
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "create_entity",
        HOI4Entity {
            name: "create_entity",
            description: r#"Creates an entity.

**Example:**
```paradox
create_entity = {
    entity = entity_name
    id = 123
    var = var_name
    x = 42
    y = 21
    z = 3
    province = 123
    state = 42
    rotation = 1.2
    scale = 10.0
    min_zoom = 100.0
    visible = scripted_trigger_name
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "destroy_entity",
        HOI4Entity {
            name: "destroy_entity",
            description: r#"Deletes an entity

**Example:**
```paradox
destroy_entity = 123
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "set_entity_movement",
        HOI4Entity {
            name: "set_entity_movement",
            description: r#"Sets the position and rotation of an entity using two coordinates.

**Example:**
```paradox
set_entity_movement = {
    id = 123
    start = {
        x = 42
        y = 21
        z = 3
    }
    target = {
        province = 124
    }
    ratio = 0.5
    rotation = 1.2
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "set_entity_position",
        HOI4Entity {
            name: "set_entity_position",
            description: r#"Sets the position of an existing entity

**Example:**
```paradox
set_entity_position = {
  id = 123
  x = 42
  y = 21
  z = 3
  province = 123
  state = 42
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "set_entity_rotation",
        HOI4Entity {
            name: "set_entity_rotation",
            description: r#"Sets the currently-facing angle of an existing entity.

**Example:**
```paradox
set_entity_rotation = {
    id = 123
    rotation = 0.23
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "set_entity_scale",
        HOI4Entity {
            name: "set_entity_scale",
            description: r#"Sets the size of an existing entity.

**Example:**
```paradox
set_entity_scale = {
  id = 123
  scale = 5.0
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "set_entity_animation",
        HOI4Entity {
            name: "set_entity_animation",
            description: r#"Sets the animation of a specified entity.

**Example:**
```paradox
set_entity_animation = {
    id = 123
    animation = "shoot_lasers"
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "build_railway",
        HOI4Entity {
            name: "build_railway",
            description: r#"Adds a railway level between two provinces or along a predefined path.

**Example:**
```paradox
build_railway = {
    level = 1
    build_only_on_allied = yes
    controller_priority = {
        base = 1
        modifier = {
            tag = MAN
            add = 2
        }
    }
    fallback = yes
    path = { 42 10 20 30 40 84 }
    start_province = 42
    target_province = 84
}
```

```paradox
build_railway = {
    level = 1
    build_only_on_allied = yes
    controller_priority = {
        base = 1
        modifier = {
            tag = MAN
            add = 2
        }
    }
    fallback = yes
    path = { 50 10 20 30 40 100 }
    start_state = 50
    target_state = 100
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("event_option_tooltip", HOI4Entity {
        name: "event_option_tooltip",
        description: r#"Shows the tooltip usually received for hovering over an event option with the specified name.

**Example:**
```paradox
event_option_tooltip = mtg_usa_civil_war_fascists.1.a
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "create_purchase_contract",
        HOI4Entity {
            name: "create_purchase_contract",
            description: r#"Creates a purchase contract with the specified parameters.

**Example:**
```paradox
create_purchase_contract = {
    seller = ROOT
    buyer = FROM
    civilian_factories = 2
    equipment = {
        type = artillery_equipment
        amount = 300
    }
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("start_border_war", HOI4Entity {
        name: "start_border_war",
        description: r#"Starts a border war for the specified attacker and defender. The participating countries are the owners of the specified states.

**Example:**
```paradox
start_border_war = {
    change_state_after_war = no
    attacker = {
        state = 527
        num_provinces = 4
        on_win = japan_border_conflict.2
        on_lose = japan_border_conflict.3
        on_cancel = japan_border_conflict.4
        modifier = 0.1
        dig_in_factor = 0
        terrain_factor = 0
    }	
    defender = {
        state = 408
        num_provinces = 4
        on_win = japan_border_conflict.3
        on_lose = japan_border_conflict.2
        on_cancel = japan_border_conflict.4
    }
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("set_border_war_data", HOI4Entity {
        name: "set_border_war_data",
        description: r#"Sets the bonuses or penalties for the attacker and defender in an on-going border war. Used after **start_border_war**.

**Example:**
```paradox
set_border_war_data = {
    attacker = 527
    defender = 408
    defender_modifier = 0.15
    combat_width = 100
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "cancel_border_war",
        HOI4Entity {
            name: "cancel_border_war",
            description: r#"Cancels an on-going border war without a winner.

**Example:**
```paradox
cancel_border_war = {
    dont_fire_events = yes
    defender = 408
    attacker = 527
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "finalize_border_war",
        HOI4Entity {
            name: "finalize_border_war",
            description: r#"Ends an on-going border war.

**Example:**
```paradox
finalize_border_war = {
    attacker_win = yes
    attacker = 527
    defender = 408
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("set_variable", HOI4Entity {
        name: "set_variable",
        description: r#"Sets a variable's value to the specified amount, creating it if not defined.

**Example:**
```paradox
set_variable = {
    var = my_variable
    value = 100
    tooltip = set_var_to_100_tt
}
```

```paradox
set_temp_variable = { temp_var = ROOT.overlord }
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("set_variable_to_random", HOI4Entity {
        name: "set_variable_to_random",
        description: r#"Sets a variable's value to the specified amount, creating it if not defined. The result will be greater than or equal than the minimum and strictly less than the maximum.

**Example:**
```paradox
set_variable_to_random = {
    var = random_num
    max = 11
    integer = yes
}
```

```paradox
set_temp_variable_to_random = my_var
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "clear_variable",
        HOI4Entity {
            name: "clear_variable",
            description: r#"Clears the value from the memory entirely.

**Example:**
```paradox
clear_variable = my_variable
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("add_to_variable", HOI4Entity {
        name: "add_to_variable",
        description: r#"Increases a variable's value by the specified amount, creating it if not defined.

**Example:**
```paradox
add_to_variable = {
    var = my_variable
    value = 100
    tooltip = add_100_to_var_tt
}
```

```paradox
add_to_temp_variable = { temp_var = num_owned_states }
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("subtract_from_variable", HOI4Entity {
        name: "subtract_from_variable",
        description: r#"Decreases a variable's value by the specified amount, creating it if not defined.

**Example:**
```paradox
subtract_from_variable = {
    var = my_variable
    value = 100
    tooltip = sub_100_from_var_tt
}
```

```paradox
subtract_from_temp_variable = { temp_var = num_owned_states }
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "multiply_variable",
        HOI4Entity {
            name: "multiply_variable",
            description: r#"Multiplies a variable's value by the specified amount.

**Example:**
```paradox
multiply_variable = {
    var = my_variable
    value = 100
    tooltip = multiply_var_by_100_tt
}
```

```paradox
multiply_temp_variable = { temp_var = num_owned_states }
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "divide_variable",
        HOI4Entity {
            name: "divide_variable",
            description: r#"Divides a variable's value by the specified amount.

**Example:**
```paradox
divide_variable = {
    var = my_variable
    value = 100
    tooltip = divide_var_by_100_tt
}
```

```paradox
divide_temp_variable = { temp_var = num_owned_states }
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("modulo_variable", HOI4Entity {
        name: "modulo_variable",
        description: r#"Makes the variable become the remainder of Euclidean division of the variable by the specified value.

**Example:**
```paradox
modulo_variable = {
    var = my_variable
    value = 50
    tooltip = get_modulo_of_var_by_50_tt
}
```

```paradox
modulo_temp_variable = { temp_var = num_controlled_states }
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "round_variable",
        HOI4Entity {
            name: "round_variable",
            description: r#"Rounds the variable towards the closest integer value.

**Example:**
```paradox
round_variable = my_variable
```

```paradox
round_temp_variable = temp
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("clamp_variable", HOI4Entity {
        name: "clamp_variable",
        description: r#"Clamps the variable to ensure its value is between the two specified numbers, raising to the minimum if smaller or lowering to the maximum if larger.

**Example:**
```paradox
clamp_variable = {
    var = my_var
    min = 0
}
```

```paradox
clamp_temp_variable = {
    var = my_var
    min = 0
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "career_profile_set_temp_playthrough_variable",
        HOI4Entity {
            name: "career_profile_set_temp_playthrough_variable",
            description: r#"Sets a temporary variable to a value or another variable.

**Example:**
```paradox
career_profile_set_temp_playthrough_variable = {
  sum = rocket_sites_built_1936
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "career_profile_set_temp_variable",
        HOI4Entity {
            name: "career_profile_set_temp_variable",
            description: r#"Sets a temporary variable to a value or another variable.

**Example:**
```paradox
career_profile_set_temp_variable = {
  var = num_dogs
  value = num_dogs_in_career_profile
}
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("add_to_array", HOI4Entity {
        name: "add_to_array",
        description: r#"Adds an element to the array either at the specified index, defaulting to the end otherwise.

**Example:**
```paradox
add_to_array = {
    array = global.my_countries
    value = THIS.id
}
```

```paradox
add_to_temp_array = { temp_states = THIS }
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "remove_from_array",
        HOI4Entity {
            name: "remove_from_array",
            description: r#"Removes an element from the array with the specified value or index.

**Example:**
```paradox
remove_from_array = {
    array = global.my_countries
    index = 0
}
```

```paradox
remove_from_temp_array = { temp_states = THIS }
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "clear_array",
        HOI4Entity {
            name: "clear_array",
            description: r#"Clears the array, removing every element inside.

**Example:**
```paradox
clear_array = global.my_countries
```

```paradox
clear_temp_array = temp_states
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert(
        "resize_array",
        HOI4Entity {
            name: "resize_array",
            description: r#"Resizes the array, removing or adding elements in the end if necessary.

**Example:**
```paradox
resize_array = {
    array = global.countries_by_states
    value = 10
    size = global.countries^num
}
```

```paradox
resize_temp_array = { temp_states = 20 }
```"#,
            scopes: &[
                crate::scope::Scope::Global,
                crate::scope::Scope::Country,
                crate::scope::Scope::State,
                crate::scope::Scope::Character,
                crate::scope::Scope::Unit,
            ],
        },
    );
    m.insert("find_highest_in_array", HOI4Entity {
        name: "find_highest_in_array",
        description: r#"Finds the largest value in the array and assigns its value and index to a temporary variable.

**Example:**
```paradox
find_highest_in_array = {
    array = global.countries_by_states
    value = temp_largest_country
    index = temp_country_index
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert("find_lowest_in_array", HOI4Entity {
        name: "find_lowest_in_array",
        description: r#"Finds the smallest value in the array and assigns its value and index to a temporary variable.

**Example:**
```paradox
find_lowest_in_array = {
    array = global.countries_by_states
    value = temp_largest_country
    index = temp_country_index
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "set_country_flag",
        HOI4Entity {
            name: "set_country_flag",
            description: r#"Defines a country flag.

**Example:**
```paradox
set_country_flag = my_flag
```

```paradox
set_country_flag = {
    flag = my_flag
    days = 123
    value = 1
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "clr_country_flag",
        HOI4Entity {
            name: "clr_country_flag",
            description: r#"Clears a defined country flag.

**Example:**
```paradox
clr_country_flag = my_flag
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "modify_country_flag",
        HOI4Entity {
            name: "modify_country_flag",
            description: r#"Adds an integer value to a flag.

**Example:**
```paradox
modify_country_flag = {
    flag = my_flag
    value = 3
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "country_event",
        HOI4Entity {
            name: "country_event",
            description: r#"Fires the specified event for the current country.

**Example:**
```paradox
country_event = {
    id = my_event.1
    days = 10
    random_hours = 12
    random_days = 10
}
```

```paradox
country_event = my_event.1
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "news_event",
        HOI4Entity {
            name: "news_event",
            description: r#"Fires the specified news event for the current country.

**Example:**
```paradox
news_event = {
    id = my_event.1
    days = 10
    random_hours = 12
    random_days = 10
}
```

```paradox
news_event = my_event.1
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("set_cosmetic_tag", HOI4Entity {
        name: "set_cosmetic_tag",
        description: r#"Makes the current scope use the specified cosmetic tag, changing name and flag.

**Example:**
```paradox
set_cosmetic_tag = SAF_SOV_communism
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "drop_cosmetic_tag",
        HOI4Entity {
            name: "drop_cosmetic_tag",
            description: r#"Makes the current scope drop the current cosmetic tag they are using.

**Example:**
```paradox
drop_cosmetic_tag = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("set_rule", HOI4Entity {
        name: "set_rule",
        description: r#"Toggles the special game rules for the current scope. Note: each rule can only be toggled a few times before a reload is required.

**Example:**
```paradox
set_rule = {
    desc = TAG_my_rule_description
    can_create_factions = yes
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "set_party_rule",
        HOI4Entity {
            name: "set_party_rule",
            description: r#"Toggles the special game rules for the current scope's political party.

**Example:**
```paradox
set_party_rule = {
    ideology = democratic
    desc = TAG_my_rule_description
    can_create_factions = yes
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("add_relation_rule_override", HOI4Entity {
        name: "add_relation_rule_override",
        description: r#"Toggles the special game rules for the current scope in diplomacy towards the specified country only, if the trigger is met.

**Example:**
```paradox
add_relation_rule_override = {
    target = SOV
    usage_desc = TAG_my_rule_description
    trigger = my_scripted_trigger
    can_access_market = yes
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "remove_relation_rule_override",
        HOI4Entity {
            name: "remove_relation_rule_override",
            description: r#"Removes the toggle added with add_relation_rule_override.

**Example:**
```paradox
remove_relation_rule_override = {
    target = SOV
    usage_desc = TAG_my_rule_description
    can_access_market = yes
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "scoped_sound_effect",
        HOI4Entity {
            name: "scoped_sound_effect",
            description: r#"Plays the specified sound once only for the current country.

**Example:**
```paradox
scoped_sound_effect = "boom"
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "scoped_play_song",
        HOI4Entity {
            name: "scoped_play_song",
            description: r#"Plays an audio track for the specified country only.

**Example:**
```paradox
scoped_play_song = "general_peace_1"
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "goto_province",
        HOI4Entity {
            name: "goto_province",
            description: r#"Moves the camera position over the specified province.

**Example:**
```paradox
goto_province = 325
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "goto_state",
        HOI4Entity {
            name: "goto_state",
            description: r#"Moves the camera position over the specified state.

**Example:**
```paradox
goto_state = 1
```

```paradox
goto_state = var:some_state
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("change_tag_from", HOI4Entity {
        name: "change_tag_from",
        description: r#"Switches the player to the current scope from the target scope. Nothing happens if the target scope is controlled by AI.

**Example:**
```paradox
change_tag_from = ROOT
```

```paradox
change_tag_from = var:from.country
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("reserve_dynamic_country", HOI4Entity {
        name: "reserve_dynamic_country",
        description: r#"Reserves the dynamic country, making sure that it does not get recycled for civil war even if it does not exist.

**Example:**
```paradox
reserve_dynamic_country = yes
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("force_update_map_mode", HOI4Entity {
        name: "force_update_map_mode",
        description: r#"Forcefully refreshes the specified mapmode for the player, rather than waiting for a daily update.

**Example:**
```paradox
force_update_map_mode = {
    limit = {
        is_ai = no
    }
    mapmode = my_map_mode
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "add_ai_strategy",
        HOI4Entity {
            name: "add_ai_strategy",
            description: r#"Sets an AI strategy for the current scope.

**Example:**
```paradox
add_ai_strategy = {
    type = alliance
    id = GER
    value = 200
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "create_dynamic_country",
        HOI4Entity {
            name: "create_dynamic_country",
            description: r#"Creates a new dynamic country, akin to ones used in civil wars.

**Example:**
```paradox
create_dynamic_country = {
    original_tag = POL
    copy_tag = SOV
    add_political_power = 100
    transfer_state = 123
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_state_core",
        HOI4Entity {
            name: "add_state_core",
            description: r#"Adds a core for the current scope to the specified state.

**Example:**
```paradox
add_state_core = 345
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "remove_state_core",
        HOI4Entity {
            name: "remove_state_core",
            description: r#"Removes the core of the current scope from the specified state.

**Example:**
```paradox
remove_state_core = 345
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_capital",
        HOI4Entity {
            name: "set_capital",
            description: r#"Makes the specified state the current scope's capital state.

**Example:**
```paradox
set_capital = {state = 345}
```

```paradox
set_capital = {
  state = 345
  remember_old_capital = no
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_state_claim",
        HOI4Entity {
            name: "add_state_claim",
            description: r#"Adds a claim for the current scope on the specified state.

**Example:**
```paradox
add_state_claim = 345
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "remove_state_claim",
        HOI4Entity {
            name: "remove_state_claim",
            description: r#"Removes a claim of the current scope from the specified state.

**Example:**
```paradox
remove_state_claim = 345
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_state_owner",
        HOI4Entity {
            name: "set_state_owner",
            description: r#"Makes the current scope the owner of the specified state.

**Example:**
```paradox
set_state_owner = 345
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_state_controller",
        HOI4Entity {
            name: "set_state_controller",
            description: r#"Makes the current scope the controller of the specified state.

**Example:**
```paradox
set_state_controller = 345
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("add_contested_owner", HOI4Entity {
        name: "add_contested_owner",
        description: r#"Adds a contested owner to a state. The effect can be used either from a country or a state scope and accepts the other as parameter.

**Example:**
```paradox
add_contested_owner = 42
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("remove_contested_owner", HOI4Entity {
        name: "remove_contested_owner",
        description: r#"Removes a contested owner to a state. The effect can be used either from a country or a state scope and accepts the other as parameter.

**Example:**
```paradox
remove_contested_owner = 42
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "transfer_state",
        HOI4Entity {
            name: "transfer_state",
            description: r#"Makes the current scope the owner and controller of the specified state.

**Example:**
```paradox
transfer_state = 345
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_province_controller",
        HOI4Entity {
            name: "set_province_controller",
            description: r#"Changes the controller of the specified province to the current scope.

**Example:**
```paradox
set_province_controller = 2999
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_political_power",
        HOI4Entity {
            name: "add_political_power",
            description: r#"Adds the specified amount of political power to the current scope.

**Example:**
```paradox
add_political_power = 100
```

```paradox
add_political_power = var:my_var
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_political_power",
        HOI4Entity {
            name: "set_political_power",
            description: r#"Sets the specified amount of political power for the current scope.

**Example:**
```paradox
set_political_power = 100
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_stability",
        HOI4Entity {
            name: "add_stability",
            description: r#"Adds to the current stability value for the current scope.

**Example:**
```paradox
add_stability = 0.1
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_stability",
        HOI4Entity {
            name: "set_stability",
            description: r#"Sets the current stability value for the current scope.

**Example:**
```paradox
set_stability = 0.5
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_war_support",
        HOI4Entity {
            name: "add_war_support",
            description: r#"Adds to the current war support value for the current scope.

**Example:**
```paradox
add_war_support = 0.1
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_war_support",
        HOI4Entity {
            name: "set_war_support",
            description: r#"Sets the current war support value for the current scope.

**Example:**
```paradox
set_war_support = 0.5
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_command_power",
        HOI4Entity {
            name: "add_command_power",
            description: r#"Adds the specified amount of command power to the current scope.

**Example:**
```paradox
add_command_power = 100
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_manpower",
        HOI4Entity {
            name: "add_manpower",
            description: r#"Adds the specified amount of manpower to the current scope.

**Example:**
```paradox
add_manpower = 100000
```

```paradox
add_manpower = var:my_var
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "army_experience",
        HOI4Entity {
            name: "army_experience",
            description: r#"Adds the specified amount of army experience to the current scope.

**Example:**
```paradox
army_experience = 10
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "navy_experience",
        HOI4Entity {
            name: "navy_experience",
            description: r#"Adds the specified amount of navy experience to the current scope.

**Example:**
```paradox
navy_experience = 10
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "air_experience",
        HOI4Entity {
            name: "air_experience",
            description: r#"Adds the specified amount of air experience to the current scope.

**Example:**
```paradox
air_experience = 10
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("set_politics", HOI4Entity {
        name: "set_politics",
        description: r#"Sets the political status of the country, including the ruling party and elections.

**Example:**
```paradox
set_politics = {
    ruling_party = democratic
    elections_allowed = no
    last_election = "1935.12.17"
    election_frequency = 48
    long_name = TAG_party_long
    name = TAG_party
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "set_popularities",
        HOI4Entity {
            name: "set_popularities",
            description: r#"Sets the political party popularities for the current scope.

**Example:**
```paradox
set_popularities = {
	democratic = 50
	neutrality = 15
	fascism = 30
	communism = 5
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_popularity",
        HOI4Entity {
            name: "add_popularity",
            description: r#"Adjusts the popularity for the specified party in the current scope.

**Example:**
```paradox
add_popularity = {
    ideology = fascism
    popularity = -0.5
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("set_political_party", HOI4Entity {
        name: "set_political_party",
        description: r#"Sets the popularity for the specified political party in the current scope.

**Example:**
```paradox
set_political_party = {
    ideology = fascism
    popularity = 50
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "set_party_name",
        HOI4Entity {
            name: "set_party_name",
            description: r#"Changes the name of the specified political party for the current scope.

**Example:**
```paradox
set_party_name = {
    ideology = neutrality
    long_name = GER_neutrality_party_kaiserreich_long
    name = GER_neutrality_party_kaiserreich
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("hold_election", HOI4Entity {
        name: "hold_election",
        description: r#"Executes the events in the **on_new_term_election** on action for the current scope.

**Example:**
```paradox
hold_election = ROOT
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "set_power_balance",
        HOI4Entity {
            name: "set_power_balance",
            description: r#"Sets a new balance of power or edits the existing one.

**Example:**
```paradox
set_power_balance = {
    id = my_bop
    left_side = my_bop_left_side
    right_side = my_bop_right_side
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "remove_power_balance",
        HOI4Entity {
            name: "remove_power_balance",
            description: r#"Removes the balance of power in entirety.

**Example:**
```paradox
remove_power_balance = {
    id = my_bop
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_power_balance_value",
        HOI4Entity {
            name: "add_power_balance_value",
            description: r#"Pushes the balance of power towards one side.

**Example:**
```paradox
add_power_balance_value = {
    id = my_bop
    value = -0.1
    tooltip_side = my_bop_side
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_power_balance_modifier",
        HOI4Entity {
            name: "add_power_balance_modifier",
            description: r#"Applies a balance of power modifier.

**Example:**
```paradox
add_power_balance_modifier = {
    id = my_bop
    modifier = my_static_modifier
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "remove_power_balance_modifier",
        HOI4Entity {
            name: "remove_power_balance_modifier",
            description: r#"Cancels a balance of power modifier.

**Example:**
```paradox
remove_power_balance_modifier = {
    id = my_bop
    modifier = my_static_modifier
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "remove_all_power_balance_modifiers",
        HOI4Entity {
            name: "remove_all_power_balance_modifiers",
            description: r#"Cancels all balance of power modifiers.

**Example:**
```paradox
remove_all_power_balance_modifiers = {
    id = my_bop
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_power_balance_gfx",
        HOI4Entity {
            name: "set_power_balance_gfx",
            description: r#"Changes the appearance of one of the sides within the balance of power.

**Example:**
```paradox
set_power_balance_gfx = {
    id = my_bop
    side = my_bop_side
    gfx = GFX_my_bop_side_new
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_major",
        HOI4Entity {
            name: "set_major",
            description: r#"Makes the current scope a major country.

**Example:**
```paradox
set_major = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("release", HOI4Entity {
        name: "release",
        description: r#"Releases the specified non-existent country as a free nation within the current country's owned states.

**Example:**
```paradox
release = GER
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("release_on_controlled", HOI4Entity {
        name: "release_on_controlled",
        description: r#"Releases the specified non-existent country as a free nation within the current country's controlled states.

**Example:**
```paradox
release_on_controlled = GER
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("release_puppet", HOI4Entity {
        name: "release_puppet",
        description: r#"Releases the specified non-existent country as a puppet of the current scope within the current country's owned states.

**Example:**
```paradox
release_puppet = GER
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("release_puppet_on_controlled", HOI4Entity {
        name: "release_puppet_on_controlled",
        description: r#"Releases the specified non-existent country as a puppet of the current scope within the current country's controlled states.

**Example:**
```paradox
release_puppet_on_controlled = GER
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("release_autonomy", HOI4Entity {
        name: "release_autonomy",
        description: r#"Releases the specified non-existent country as a subject of the specified autonomy of the current scope within the current country's owned states.

**Example:**
```paradox
release_autonomy = {
    target = VIN
    autonomy_state = autonomy_puppet
    freedom_level = 0.5
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "give_guarantee",
        HOI4Entity {
            name: "give_guarantee",
            description: r#"The current scope guarantees the target country.

**Example:**
```paradox
give_guarantee = GER
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "give_military_access",
        HOI4Entity {
            name: "give_military_access",
            description: r#"The current scope grants military access to the target country.

**Example:**
```paradox
give_military_access = GER
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "recall_attache",
        HOI4Entity {
            name: "recall_attache",
            description: r#"Recalls the current scope's attaché from the specified country.

**Example:**
```paradox
recall_attache = GER
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("diplomatic_relation", HOI4Entity {
        name: "diplomatic_relation",
        description: r#"Used to define a diplomatic relation between the current scope and target scope country.

**Example:**
```paradox
diplomatic_relation = {
    country = SOV
    relation = guarantee
    active = no
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("add_opinion_modifier", HOI4Entity {
        name: "add_opinion_modifier",
        description: r#"The current scope gains the specified opinion modifier **towards the target scope**. Can also be used to modify trade relations by adding 'trade = yes' in the opinion <modifier> in /Hearts of Iron IV/common/opinion_modifiers/*.txt. If used with a trade opinion_modifier the behaviour is reversed, meaning that the target gains the trade opinion towards the **current scope**.

**Example:**
```paradox
add_opinion_modifier = {
    target = GER
    modifier = faction_traitor
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("remove_opinion_modifier", HOI4Entity {
        name: "remove_opinion_modifier",
        description: r#"The current scope loses the specified opinion modifier **towards the target scope**.

**Example:**
```paradox
remove_opinion_modifier = {
    target = GER
    modifier = faction_traitor
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("reverse_add_opinion_modifier", HOI4Entity {
        name: "reverse_add_opinion_modifier",
        description: r#"The target scope gains the specified opinion modifier **towards the current scope**.

**Example:**
```paradox
reverse_add_opinion_modifier = {
    target = GER
    modifier = faction_traitor
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("add_relation_modifier", HOI4Entity {
        name: "add_relation_modifier",
        description: r#"The current scope gains the specified relation modifier **towards the target scope**.

**Example:**
```paradox
add_relation_modifier = {
    target = SWE
    modifier = HUN_dynastic_ties_license
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("remove_relation_modifier", HOI4Entity {
        name: "remove_relation_modifier",
        description: r#"The current scope loses the specified relation modifier for **towards the target scope**.

**Example:**
```paradox
remove_relation_modifier = {
    target = SWE
    modifier = HUN_dynastic_ties_license
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "add_collaboration",
        HOI4Entity {
            name: "add_collaboration",
            description: r#"Adds collaboration in TAG with the scoped country.

**Example:**
```paradox
add_collaboration = {
    target = TAG
    value = 0.3
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_collaboration",
        HOI4Entity {
            name: "set_collaboration",
            description: r#"Sets the collaboration in TAG with the scoped country.

**Example:**
```paradox
set_collaboration = {
    target = TAG
    value = 0.3
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("recall_volunteers_from", HOI4Entity {
        name: "recall_volunteers_from",
        description: r#"Recalls volunteers sent to the specified country back to the current country.

**Example:**
```paradox
recall_volunteers_from = SPR
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "set_occupation_law",
        HOI4Entity {
            name: "set_occupation_law",
            description: r#"Sets the occupation law of the country.

**Example:**
```paradox
USA = {
  GER = {
    set_occupation_law = foreign_civilian_oversight
  }
}
```

# Changes USA's occupation law for GER.

```paradox
USA = {
  USA = {
    set_occupation_law = default_law
  }
}
```

# Changes the USA's default occupation law to the default."#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_occupation_law_where_available",
        HOI4Entity {
            name: "set_occupation_law_where_available",
            description: r#"Sets the occupation law of the country.

**Example:**
```paradox
USA = {
  GER = {
    set_occupation_law_where_available = foreign_civilian_oversight
  }
}
```

# Changes USA's occupation law for GER where possible.

```paradox
USA = {
  USA = {
    set_occupation_law_where_available = default_law
  }
}
```

# Changes the USA's default occupation law to the default where possible."#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "send_embargo",
        HOI4Entity {
            name: "send_embargo",
            description: r#"Embargos the target country.

**Example:**
```paradox
send_embargo = ITA
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "break_embargo",
        HOI4Entity {
            name: "break_embargo",
            description: r#"Stops embargoing the target country.

**Example:**
```paradox
break_embargo = ITA
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "give_market_access",
        HOI4Entity {
            name: "give_market_access",
            description: r#"Opens market access between the two countries.

**Example:**
```paradox
give_market_access = ITA
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("create_faction", HOI4Entity {
        name: "create_faction",
        description: r#"Creates a faction with the specified name for the current scope. The current scope and any subjects automatically join the faction.

**Example:**
```paradox
create_faction = MY_FACTION_NAME
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("create_faction_from_template", HOI4Entity {
        name: "create_faction_from_template",
        description: r#"Create a faction from a template allows for optional customization of name, icon and color.

**Example:**
```paradox
create_faction_from_template = faction_template_GER_mitteleuropa_alliance
```

```paradox
create_faction_from_template = {
   template = faction_template_defensive_democratic
   name = AUS_alpine_federation
   icon = GFX_faction_logo_generic_2
   color = { 100 100 150 }
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "add_to_faction",
        HOI4Entity {
            name: "add_to_faction",
            description: r#"Adds the country to the faction of the current scope.

**Example:**
```paradox
add_to_faction = GER
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "dismantle_faction",
        HOI4Entity {
            name: "dismantle_faction",
            description: r#"Dismantles the faction of the current scope.

**Example:**
```paradox
dismantle_faction = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "leave_faction",
        HOI4Entity {
            name: "leave_faction",
            description: r#"Removes the current scope from the faction they are part of.

**Example:**
```paradox
leave_faction = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "remove_from_faction",
        HOI4Entity {
            name: "remove_from_faction",
            description: r#"Removes the specified scope from the faction led by the current scope.

**Example:**
```paradox
remove_from_faction = GER
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_faction_name",
        HOI4Entity {
            name: "set_faction_name",
            description: r#"Changes faction names.

**Example:**
```paradox
set_faction_name = SOME_LOC_KEY
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_faction_leader",
        HOI4Entity {
            name: "set_faction_leader",
            description: r#"Sets the current country as the faction leader.

**Example:**
```paradox
set_faction_leader = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_faction_spymaster",
        HOI4Entity {
            name: "set_faction_spymaster",
            description: r#"Sets the current country as the faction spymaster.

**Example:**
```paradox
set_faction_spymaster = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_faction_rule",
        HOI4Entity {
            name: "set_faction_rule",
            description: r#"Set a rule on the country's faction.

**Example:**
```paradox
set_faction_rule = rule_id
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("set_faction_manifest", HOI4Entity {
        name: "set_faction_manifest",
        description: r#"Changes current country's faction manifest, the previous manifest is removed.

**Example:**
```paradox
set_faction_manifest = faction_manifest_id
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "add_faction_goal",
        HOI4Entity {
            name: "add_faction_goal",
            description: r#"Adds a goal to the current’s country faction.

**Example:**
```paradox
add_faction_goal = faction_goal_an_armored_fist
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "remove_faction_goal",
        HOI4Entity {
            name: "remove_faction_goal",
            description: r#"Remove a goal from the current’s country faction.

**Example:**
```paradox
remove_faction_goal = faction_goal_secure_the_oil_supply
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_faction_goal_slot",
        HOI4Entity {
            name: "add_faction_goal_slot",
            description: r#"Adds extra goal slots to the faction for a specific category.

**Example:**
```paradox
add_faction_goal_slot = {
    category  = short_term
    value = 1
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("add_faction_influence_ratio", HOI4Entity {
        name: "add_faction_influence_ratio",
        description: r#"Adds influence to the country based on the given ratio of the faction’s total influence.

**Example:**
```paradox
add_faction_influence_ratio = 0.075
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "add_faction_influence_score",
        HOI4Entity {
            name: "add_faction_influence_score",
            description: r#"Adds influence to the country in the faction.

**Example:**
```paradox
add_faction_influence_score = 5
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_faction_initiative",
        HOI4Entity {
            name: "add_faction_initiative",
            description: r#"Adds Faction Initiative points to the current country’s faction.

**Example:**
```paradox
add_faction_initiative = 1
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_faction_power_projection",
        HOI4Entity {
            name: "add_faction_power_projection",
            description: r#"Adds power projection to the faction.

**Example:**
```paradox
add_faction_power_projection = 100
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_faction_upgrade",
        HOI4Entity {
            name: "set_faction_upgrade",
            description: r#"Set either a member upgrade for the specified tag.

**Example:**
```paradox
set_faction_upgrade = token
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("set_faction_member_upgrade_min", HOI4Entity {
        name: "set_faction_member_upgrade_min",
        description: r#"Set a faction's minimal requirements for an faction member upgrade group.

**Example:**
```paradox
set_faction_member_upgrade_min = {
    upgrade = TOKEN_TO_FACTION_MEMBER_UPGRADE
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("set_faction_military_unlocked", HOI4Entity {
        name: "set_faction_military_unlocked",
        description: r#"Sets wheter the current countries faction can make changes to the faction research section.

**Example:**
```paradox
set_faction_military_unlocked = yes
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("set_faction_research_unlocked", HOI4Entity {
        name: "set_faction_research_unlocked",
        description: r#"Sets wheter the current countries faction can make changes to the faction research section.

**Example:**
```paradox
set_faction_research_unlocked = yes
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "puppet",
        HOI4Entity {
            name: "puppet",
            description: r#"Makes the specified country a subject of the current scope.

**Example:**
```paradox
puppet = GER
```

```paradox
puppet = {
    target = ITA
    end_wars = no
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "end_puppet",
        HOI4Entity {
            name: "end_puppet",
            description: r#"Removes the subject status between the target and the current scope.

**Example:**
```paradox
end_puppet = GER
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_autonomy_ratio",
        HOI4Entity {
            name: "add_autonomy_ratio",
            description: r#"Adds a freedom score ratio modifier to the current scope.

**Example:**
```paradox
add_autonomy_ratio = {
    value = 0.1
    localization = AST_adopt_westminster
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_autonomy_score",
        HOI4Entity {
            name: "add_autonomy_score",
            description: r#"Adds an exact freedom score modifier to the current scope.

**Example:**
```paradox
add_autonomy_score = {
    value = 10
    localization = EXAMPLE
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("set_autonomy", HOI4Entity {
        name: "set_autonomy",
        description: r#"Sets the autonomy level for the specified country, **including independence**.

**Example:**
```paradox
set_autonomy = {
    target = AST
    autonomous_state = autonomy_free
    end_wars = no
    end_civil_wars = no
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "add_legitimacy",
        HOI4Entity {
            name: "add_legitimacy",
            description: r#"Adds legitimacy.

**Example:**
```paradox
add_legitimacy = 10
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_legitimacy",
        HOI4Entity {
            name: "set_legitimacy",
            description: r#"Sets legitimacy.

**Example:**
```paradox
set_legitimacy = 10
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "become_exiled_in",
        HOI4Entity {
            name: "become_exiled_in",
            description: r#"Creates a government in exile.

**Example:**
```paradox
become_exiled_in = { target = <Host tag> legitimacy = <0-100> (starting legitimacy, optional) }
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "end_exile",
        HOI4Entity {
            name: "end_exile",
            description: r#"Ends a government in exile.

**Example:**
```paradox
end_exile = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_threat",
        HOI4Entity {
            name: "add_threat",
            description: r#"Adjusts the level of World Tension.

**Example:**
```paradox
add_threat = 10
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("add_named_threat", HOI4Entity {
        name: "add_named_threat",
        description: r#"Adjusts the level of World Tension and adds an entry in the World Tension tooltip.

**Example:**
```paradox
add_named_threat = {
    threat = 5
    name = GER_rhineland
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "annex_country",
        HOI4Entity {
            name: "annex_country",
            description: r#"Annex the specified country for the current scope.

**Example:**
```paradox
annex_country = {
    target = GER
    transfer_troops = yes
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("add_to_war", HOI4Entity {
        name: "add_to_war",
        description: r#"Forces the current scope to join the war of the specified ally against the specified enemy.

**Example:**
```paradox
add_to_war = {
    targeted_alliance = PREV
    enemy = HUN
    hostility_reason = asked_to_join
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("declare_war_on", HOI4Entity {
        name: "declare_war_on",
        description: r#"Makes the current scope declare war on the specified country with the specified wargoal.

**Example:**
```paradox
declare_war_on = {
    target = GER
    type = annex_everything
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "white_peace",
        HOI4Entity {
            name: "white_peace",
            description: r#"Makes the current scope white peace the specified scope.

**Example:**
```paradox
white_peace = GER
```

```paradox
white_peace = {
    tag = GER
    message = my_peace_tt
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("start_peace_conference", HOI4Entity {
        name: "start_peace_conference",
        description: r#"Makes the current scope start a peace conference with the specified scope on the other side.

**Example:**
```paradox
start_peace_conference = {
    tag = GER
    score_factor = 0.4
    message = my_peace_tt
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "set_truce",
        HOI4Entity {
            name: "set_truce",
            description: r#"Makes the current scope truce with the specified scope.

**Example:**
```paradox
set_truce = {
    target = GER
    days = 90
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "create_wargoal",
        HOI4Entity {
            name: "create_wargoal",
            description: r#"Grants the current scope a wargoal against the specified country.

**Example:**
```paradox
create_wargoal = {
    type = puppet_wargoal_focus
    target = ROOT
}
```

```paradox
create_wargoal = {
    type = take_state_focus
    target = PREV
    generator = { 123 321 }
    expire = 90
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "remove_wargoal",
        HOI4Entity {
            name: "remove_wargoal",
            description: r#"Removes wargoals from the current scope to the specified country.

**Example:**
```paradox
remove_wargoal = {
    type = all
    target = ROOT
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "start_civil_war",
        HOI4Entity {
            name: "start_civil_war",
            description: r#"Starts a civil war for the current scope with the specified parameters.

**Example:**
```paradox
start_civil_war = {
    ruling_party = communism
    # Original country's ideology changes to communism
    ideology = ROOT
    # Breakaway gets old ideology of ROOT
    size = 0.8
    capital = 282
    states = {
        282 533 536 555 529 530 528
    }
    keep_unit_leaders = {
        750 751 752
    }
    keep_political_leader = yes
    keep_political_party_members = yes
}
```

```paradox
start_civil_war = {
    ideology = democratic
    size = 0.1
    states = all
    states_filter = {
        is_on_continent = europe
        is_capital = no
    }
    set_country_flag = TAG_my_country_tag_alias_trigger
    # Sets a country flag that gets used in a country tag alias.
}
```

(See country tag aliases)

```paradox
start_civil_war = {
    ideology = neutrality
    size = 0.1
    army_ratio = 0.5
    navy_ratio = 0
    air_ratio = 1
    keep_unit_leaders_trigger = {
        has_trait = my_trait_name
    }
    keep_all_characters = yes
    PREV = {  # Original country
        TAG_airforce_leader = { # Character
            set_nationality = PREV.PREV
            # Transfers to breakaway
        }
    }
    promote_character = TAG_airforce_leader
}
```

(See usage for PREV and PREV.PREV)"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("add_civil_war_target", HOI4Entity {
        name: "add_civil_war_target",
        description: r#"Sets that the war between ROOT and TAG is a civil war, resulting in the victory being the annexation of the other side and setting world tension limits on intervention.

**Example:**
```paradox
add_civil_war_target = TAG
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("remove_civil_war_target", HOI4Entity {
        name: "remove_civil_war_target",
        description: r#"Removes the status of the war as a civil war between the pair of countries.

**Example:**
```paradox
remove_civil_war_target = TAG
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("transfer_units_fraction", HOI4Entity {
        name: "transfer_units_fraction",
        description: r#"Transfers a fraction of the military to a target, including units (either type: land, navy, or air), equipment, and unit leaders.

**Example:**
```paradox
transfer_units_fraction= {
	target = SPD
	size = 0.5
	stockpile_ratio = 0.8
	army_ratio = 0.8
	navy_ratio = 0.5
	air_ratio = 0.5
	keep_unit_leaders_trigger = {
		has_trait = trait_SPA_nationalist_sympathies
	}
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "add_nuclear_bombs",
        HOI4Entity {
            name: "add_nuclear_bombs",
            description: r#"Adds specified number of nukes to the country's stockpile

**Example:**
```paradox
add_nuclear_bombs = 100
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("launch_nuke", HOI4Entity {
        name: "launch_nuke",
        description: r#"Nukes the specified province or a province in the needed state. If a state is set rather than the specific province, first prioritises the country set in `controller`, then prioritises the countries at war with the current scope, and then countries that are neutral.

**Example:**
```paradox
launch_nuke = {
    province = 1234
}
```

```paradox
launch_nuke = {
    state = 42
    controller = GER
    use_nuke = yes
    nuke_type = nuclear_bomb
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("add_resource", HOI4Entity {
        name: "add_resource",
        description: r#"Adds the specified resource in the specified amount to the specified state.

**Example:**
```paradox
add_resource = {
    type = oil
    amount = 50
    state = 88
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("create_import", HOI4Entity {
        name: "create_import",
        description: r#"Creates an import for the current scope with the specified resource and from the specified exporter.

**Example:**
```paradox
create_import = {
    resource = steel
    amount = 100
    exporter = GER
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "give_resource_rights",
        HOI4Entity {
            name: "give_resource_rights",
            description: r#"Gives all the resources of a state to the target country

**Example:**
```paradox
give_resource_rights = { receiver = ENG state = 291 }
```

```paradox
give_resource_rights = {
    receiver = POL
    state = 321
    resources = { oil }
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "remove_resource_rights",
        HOI4Entity {
            name: "remove_resource_rights",
            description: r#"Removes given resource rights

**Example:**
```paradox
ENG = { remove_resource_rights = 477 }
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_fuel",
        HOI4Entity {
            name: "add_fuel",
            description: r#"Adds fuel to the current country.

**Example:**
```paradox
add_fuel = 400
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_fuel",
        HOI4Entity {
            name: "set_fuel",
            description: r#"Sets country's current fuel amount.

**Example:**
```paradox
set_fuel = 400
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_fuel_ratio",
        HOI4Entity {
            name: "set_fuel_ratio",
            description: r#"Set country's current fuel ratio relative to its capacity.

**Example:**
```paradox
set_fuel_ratio = 0.5
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("add_offsite_building", HOI4Entity {
        name: "add_offsite_building",
        description: r#"Adds an off-map (offmap) building for the current scope that produces its effects without being present in a state.

**Example:**
```paradox
add_offsite_building = { type = arms_factory level = 1 }
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("modify_building_resources", HOI4Entity {
        name: "modify_building_resources",
        description: r#"Modifies the resource output of the specified building for the current scope.

**Example:**
```paradox
modify_building_resources = {
    building = synthetic_refinery
    resource = oil
    amount = 1
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "damage_building",
        HOI4Entity {
            name: "damage_building",
            description: r#"Damages a building in a targeted state or province.

**Example:**
```paradox
damage_building = {
  type = infrastructure
  state = 123
  damage = 1
}
```

```paradox
damage_building = {
  tags = dam_building
  damage = 1
  repair_speed_modifier = -0.8
  province = 3488
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("load_focus_tree", HOI4Entity {
        name: "load_focus_tree",
        description: r#"Loads a new focus tree for the current scope, retaining any shared focuses if set.

**Example:**
```paradox
load_focus_tree = china_communist_focus
```

```paradox
load_focus_tree = {
  tree = british_focus
  keep_completed = yes
  copy_completed_from = ENG
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("unlock_national_focus", HOI4Entity {
        name: "unlock_national_focus",
        description: r#"Bypasses the specified focus for the current scope (marks as complete without firing `complete_effect` of the focus).

**Example:**
```paradox
unlock_national_focus = my_focus
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "complete_national_focus",
        HOI4Entity {
            name: "complete_national_focus",
            description: r#"Completes the specified focus for the current scope.

**Example:**
```paradox
complete_national_focus = my_focus
```

```paradox
complete_national_focus = {
  focus = GER_autonomous_organization_todt
  use_side_message = yes
  originator_name = GER_fritz_todt
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("uncomplete_national_focus", HOI4Entity {
        name: "uncomplete_national_focus",
        description: r#"Removes a focus from list of completed focus, and potentially all focuses requiring it as a prerequisite.
If the focus has one, the 'on_uncomplete' effect will be executed on each uncompleted focus.

**Example:**
```paradox
uncomplete_national_focus = {
  focus = GER_oppose_hitler
  uncomplete_children = yes
  refund_political_power = no
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("mark_focus_tree_layout_dirty", HOI4Entity {
        name: "mark_focus_tree_layout_dirty",
        description: r#"Refreshes the focus tree for the specified country, restarting the checks in `allow_branch` and position offsets for focuses.

**Example:**
```paradox
mark_focus_tree_layout_dirty = yes
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("activate_shine_on_focus", HOI4Entity {
        name: "activate_shine_on_focus",
        description: r#"Activates the shine effect on the focus with the given id. Focuses that are completed cannot have an activated shine effect.

**Example:**
```paradox
`activate_shine_on_focus = my_focus`
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("deactivate_shine_on_focus", HOI4Entity {
        name: "deactivate_shine_on_focus",
        description: r#"Deactivate the shine effect on the focus with the given id. The current focus cannot have it's shine effect removed.

**Example:**
```paradox
`deactivate_shine_on_focus = my_focus`
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("reduce_focus_completion_cost", HOI4Entity {
        name: "reduce_focus_completion_cost",
        description: r#"Reduce the cost needed to complete a specific focus. The cost accepts script constants. The focus can be a uniform list or a single token.

**Example:**
```paradox
reduce_focus_completion_cost = {
  focus = focus_id
  cost = 35
}
```

```paradox
reduce_focus_completion_cost = {
  focus = {focus_id_1 focus_id_2}
  cost = 35
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("activate_decision", HOI4Entity {
        name: "activate_decision",
        description: r#"Activates the specified decision for the current scope, ignoring triggers for the decision.

**Example:**
```paradox
activate_decision = my_decision
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("activate_targeted_decision", HOI4Entity {
        name: "activate_targeted_decision",
        description: r#"Activates the specified targeted decision for the specified target for the current scope.

**Example:**
```paradox
activate_targeted_decision = {
    target = GER
    decision = my_decision
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "remove_targeted_decision",
        HOI4Entity {
            name: "remove_targeted_decision",
            description: r#"Removes the specified targeted decision for the current scope.

**Example:**
```paradox
remove_targeted_decision = {
    target = FROM
    decision = my_decision
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("unlock_decision_tooltip", HOI4Entity {
        name: "unlock_decision_tooltip",
        description: r#"Displays a special tooltip for the specified decision in the effect tooltip.

**Example:**
```paradox
unlock_decision_tooltip = my_decision
```

```paradox
unlock_decision_tooltip = {
    decision = my_decision
    show_effect_tooltip = yes
    show_modifiers = yes
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("unlock_decision_category_tooltip", HOI4Entity {
        name: "unlock_decision_category_tooltip",
        description: r#"Displays a special tooltip for the specified decision category in the effect tooltip.

**Example:**
```paradox
unlock_decision_category_tooltip = my_category
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("add_days_remove", HOI4Entity {
        name: "add_days_remove",
        description: r#"Adds the number of days to the timer created by a decision's days_remove.

**Example:**
```paradox
add_days_remove  = {
    decision = decision_here
    days = 30
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "remove_decision",
        HOI4Entity {
            name: "remove_decision",
            description: r#"Removes a decision.

**Example:**
```paradox
remove_decision = GER_MEPO
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("remove_decision_on_cooldown", HOI4Entity {
        name: "remove_decision_on_cooldown",
        description: r#"If the decision is on cooldown, it gets removed, in order to reactivate or remove completely.

**Example:**
```paradox
remove_decision_on_cooldown = TAG_my_decision
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("activate_mission", HOI4Entity {
        name: "activate_mission",
        description: r#"Activates the specified mission for the current scope, ignoring any triggers for the decision.

**Example:**
```paradox
activate_mission = my_mission
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("activate_mission_tooltip", HOI4Entity {
        name: "activate_mission_tooltip",
        description: r#"Displays a special tooltip for the specified mission in the effect tooltip.

**Example:**
```paradox
activate_mission_tooltip = my_mission
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "remove_mission",
        HOI4Entity {
            name: "remove_mission",
            description: r#"Removes the specified mission for the current scope.

**Example:**
```paradox
remove_mission = my_mission
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_days_mission_timeout",
        HOI4Entity {
            name: "add_days_mission_timeout",
            description: r#"Adds the number of days to the specified mission.

**Example:**
```paradox
add_days_mission_timeout = {
    mission = my_mission
    days = 20
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("add_research_slot", HOI4Entity {
        name: "add_research_slot",
        description: r#"Adjusts the number of research slots the current scope has. Can remove slots with negatives.

**Example:**
```paradox
add_research_slot = 1
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "set_research_slots",
        HOI4Entity {
            name: "set_research_slots",
            description: r#"Sets the number of research slots the current scope has.

**Example:**
```paradox
set_research_slots = 4
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("add_tech_bonus", HOI4Entity {
        name: "add_tech_bonus",
        description: r#"Grants a research bonus to the current scope with the specified parameters.

**Example:**
```paradox
add_tech_bonus = {
    bonus = 0.5
    uses = 1
    category = radar_tech
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "set_technology",
        HOI4Entity {
            name: "set_technology",
            description: r#"Grants the specified technology to the current scope.

**Example:**
```paradox
set_technology = {
    suicide_craft = 1
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_to_tech_sharing_group",
        HOI4Entity {
            name: "add_to_tech_sharing_group",
            description: r#"Adds the current scope to the specified technology sharing group.

**Example:**
```paradox
add_to_tech_sharing_group = us_research
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "remove_from_tech_sharing_group",
        HOI4Entity {
            name: "remove_from_tech_sharing_group",
            description: r#"Removes the current scope from the specified technology sharing group.

**Example:**
```paradox
remove_from_tech_sharing_group = us_research
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "modify_tech_sharing_bonus",
        HOI4Entity {
            name: "modify_tech_sharing_bonus",
            description: r#"Modifies the specified technology sharing group.

**Example:**
```paradox
modify_tech_sharing_bonus = {
    id = us_research
    bonus = 0.5
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("inherit_technology", HOI4Entity {
        name: "inherit_technology",
        description: r#"Makes the current country's researched technologies be copied from the specified country.

**Example:**
```paradox
inherit_technology = CAN
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "mark_technology_tree_layout_dirty",
        HOI4Entity {
            name: "mark_technology_tree_layout_dirty",
            description: r#"Forces the refresh of the hidden technologies for the scoped country.

**Example:**
```paradox
mark_technology_tree_layout_dirty = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_ideas",
        HOI4Entity {
            name: "add_ideas",
            description: r#"Adds the specified ideas to the current scope.

**Example:**
```paradox
add_ideas = my_idea
```

```paradox
add_ideas = {
    my_idea_1
    my_idea_2
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("add_timed_idea", HOI4Entity {
        name: "add_timed_idea",
        description: r#"Adds the specified ideas to the current scope for the specified number of days.

**Example:**
```paradox
add_timed_idea = {
    idea = my_idea
    days = 180
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("modify_timed_idea", HOI4Entity {
        name: "modify_timed_idea",
        description: r#"Extends or shortens the duration of the timed idea by the specified amount.

**Example:**
```paradox
modify_timed_idea = {
    idea = my_idea
    days = 60
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("swap_ideas", HOI4Entity {
        name: "swap_ideas",
        description: r#"Switches two ideas with a tooltip displaying any modifier differences between them.

**Example:**
```paradox
swap_ideas = {
    remove_idea = my_idea_1
    add_idea = my_idea_2
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "remove_ideas",
        HOI4Entity {
            name: "remove_ideas",
            description: r#"Removes the specified idea from the current scope.

**Example:**
```paradox
remove_ideas = my_idea
```

```paradox
remove_ideas = {
    my_idea_1
    my_idea_2
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "remove_ideas_with_trait",
        HOI4Entity {
            name: "remove_ideas_with_trait",
            description: r#"Removes all ideas for the current scope that use the specified trait.

**Example:**
```paradox
remove_ideas_with_trait = motorized_equipment_manufacturer
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("show_ideas_tooltip", HOI4Entity {
        name: "show_ideas_tooltip",
        description: r#"Displays the specified idea in the tooltip for the current effect scope. Does not add the idea.

**Example:**
```paradox
show_ideas_tooltip = my_idea
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("load_oob", HOI4Entity {
        name: "load_oob",
        description: r#"Loads the specified order of battle for the current scope, applying the effects within. The filename with the `.txt` extension omitted is used as the effect's target.

**Example:**
```paradox
load_oob = "GER_default"
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "division_template",
        HOI4Entity {
            name: "division_template",
            description: r#"Creates and adds the specified division template to the current scope.

**Example:**
```paradox
division_template = {
    name = "Test"
    is_locked = yes
    division_cap = 3
    division_names_group = USA_INF_01
    priority = 0
    template_counter = 0
    regiments = {
        infantry = { x = 0 y = 0 }
        infantry = { x = 0 y = 1 }
        infantry = { x = 0 y = 2 }
        infantry = { x = 0 y = 3 }
    }
    support = {
        military_police = { x = 0 y = 0 }
    }
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "create_colonial_division_template",
        HOI4Entity {
            name: "create_colonial_division_template",
            description: r#"Create a colonial division template for overlord/owner.

**Example:**
```paradox
create_colonial_division_template = {
  subject = RAJ
  division_template = {
    name = "Infantry Division"
    division_names_group = RAJ_INF_01
    ...
    regiments = {
      infantry = { x = 0 y = 0 }
      infantry = { x = 0 y = 1 }
     }
  }
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("add_units_to_division_template", HOI4Entity {
        name: "add_units_to_division_template",
        description: r#"Adds the specified brigades to first available slots of specified columns to the template (if possible).

**Example:**
```paradox
add_units_to_division_template = {
    template_name = "Test"
    regiments = {
        infantry = 2
        infantry = 2
    }
    support = {
        military_police = 0
    }
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("set_division_template_lock", HOI4Entity {
        name: "set_division_template_lock",
        description: r#"Toggles the locked status on a division template for the current scope, which prevents editing or deletion.

**Example:**
```paradox
set_division_template_lock = {
    division_template = "Infantry Division"
    is_locked = yes
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "country_lock_all_division_template",
        HOI4Entity {
            name: "country_lock_all_division_template",
            description: r#"Locks all division templates for the current scope.

**Example:**
```paradox
country_lock_all_division_template = yes
```

```paradox
country_lock_all_division_template = {
  is_locked = yes
  desc = loc_key
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("set_division_force_allow_recruiting", HOI4Entity {
        name: "set_division_force_allow_recruiting",
        description: r#"Changes whether it's possible to recruit divisions of a locked template without unlocking the template.

**Example:**
```paradox
set_division_force_allow_recruiting = {
    division_template = "My locked template"
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("set_division_template_cap", HOI4Entity {
        name: "set_division_template_cap",
        description: r#"Sets the cap of a division template. The template has to be locked first.

**Example:**
```paradox
set_division_template_cap = {
	division_template = "Swiss Citizen Militia"
	division_cap = SWI_militia_division_cap
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("clear_division_template_cap", HOI4Entity {
        name: "clear_division_template_cap",
        description: r#"Clears the cap on the template, allowing it to have an unlimited amount of divisions.

**Example:**
```paradox
clear_division_template_cap = {
	division_template = "Swiss Citizen Militia"
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("delete_unit_template_and_units", HOI4Entity {
        name: "delete_unit_template_and_units",
        description: r#"Deletes the specified division template and all units using it for the current scope.

**Example:**
```paradox
delete_unit_template_and_units = {
    division_template = "Infantry Division"
    disband = yes #will refund equipment and manpower
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "delete_unit",
        HOI4Entity {
            name: "delete_unit",
            description: r#"Deletes all units that meet the filters.

**Example:**
```paradox
delete_unit = {
    state = 787
    disband = yes #will refund equipment and manpower
}
```

```paradox
delete_unit = {
    division_template = "Infantry Division"
}
```

```paradox
delete_unit = {} # Will delete all units
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "delete_units",
        HOI4Entity {
            name: "delete_units",
            description: r#"Deletes all units with a certain template.

**Example:**
```paradox
delete_units = {
    division_template = "Infantry Division"
    disband = yes
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "create_railway_gun",
        HOI4Entity {
            name: "create_railway_gun",
            description: r#"Creates a railway gun.

**Example:**
```paradox
create_railway_gun = {
    equipment = railway_gun_equipment_1
	name = TAG_new_railway_gun
	location = 12406
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "teleport_railway_guns_to_deploy_province",
        HOI4Entity {
            name: "teleport_railway_guns_to_deploy_province",
            description: r#"Teleports all railway guns to the province where they get deployed.

**Example:**
```paradox
teleport_railway_guns_to_deploy_province = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_unit_bonus",
        HOI4Entity {
            name: "add_unit_bonus",
            description: r#"Adds permanent subunit and subunit category bonuses for country.

**Example:**
```paradox
add_unit_bonus = {
  category_light_infantry = {
    soft_attack = 0.05
  }

  cavalry = {
    soft_attack = 0.05
    hard_attack = 0.05
  }
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_equipment_fraction",
        HOI4Entity {
            name: "set_equipment_fraction",
            description: r#"Reduces the overall equipment stockpile by the specified fraction.

**Example:**
```paradox
set_equipment_fraction = 0.5
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("add_equipment_to_stockpile", HOI4Entity {
        name: "add_equipment_to_stockpile",
        description: r#"Edits the equipment stockpile of the current scope, adds or removes equipment of a specified type or archetype.

**Example:**
```paradox
add_equipment_to_stockpile = {
    type = infantry_equipment
    amount = -100
    producer = GER
}
```

```paradox
add_equipment_to_stockpile = {
    type = medium_tank_chassis_1
    amount = 100
    variant_name = "Panzer III"
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("send_equipment", HOI4Entity {
        name: "send_equipment",
        description: r#"Sends the specified amount of equipment to the specified target, removing said equipment from the current scope.

**Example:**
```paradox
send_equipment = {
    equipment = infantry_equipment
    amount = 100
    target = GER
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("send_equipment_fraction", HOI4Entity {
        name: "send_equipment_fraction",
        description: r#"Sends the specified fraction of equipment to the specified target, removing said equipment from the current scope.

**Example:**
```paradox
send_equipment_fraction = {
    value = 0.3
    target = GER
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("create_production_license", HOI4Entity {
        name: "create_production_license",
        description: r#"Grants the specified country a license to produce the specified equipment from the current scope.

**Example:**
```paradox
create_production_license = {
    target = HUN
    equipment = {
        type = fighter_equipment_1
        version = 0
        new_prioritised = no
    }
    cost_factor = 0
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "add_equipment_subsidy",
        HOI4Entity {
            name: "add_equipment_subsidy",
            description: r#"Creates an equipment subsidy on the international market.

**Example:**
```paradox
add_equipment_subsidy = {
    cic = 300
    equipment_type = support_equipment
    seller_tags = { BHR }
}
```

```paradox
add_equipment_subsidy = {
    cic = 1000
    equipment_type = infantry_equipment
    seller_trigger = my_scripted_trigger
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_cic",
        HOI4Entity {
            name: "add_cic",
            description: r#"Modifies the economic capacity bank on the international market.

**Example:**
```paradox
add_cic = 300
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "create_equipment_variant",
        HOI4Entity {
            name: "create_equipment_variant",
            description: r#"Creates the specified equipment variant for the current scope.

**Example:**
```paradox
create_equipment_variant = {
    name = "Vetehinen Class"								
    type = ship_hull_submarine_1
    name_group = FIN_SS_HISTORICAL
    role_icon_index = 1
    modules = {
        fixed_ship_torpedo_slot = ship_torpedo_sub_1
        fixed_ship_engine_slot = sub_ship_engine_1
        rear_1_custom_slot = ship_mine_layer_sub
    }
}
```

```paradox
create_equipment_variant = {
    name = "He 112"
    type = fighter_equipment_0
    obsolete = yes
    upgrades = {
        plane_gun_upgrade = 1
        plane_range_upgrade = 1
    }
}
```

```paradox
create_equipment_variant = {
    name = "Light Tank Mk. IV"
    type = light_tank_chassis_1
    parent_version = 1
    modules = {
        main_armament_slot = tank_heavy_machine_gun
    }
    upgrades = {
        tank_nsb_engine_upgrade = 2
    }
    icon = "GFX_ENG_basic_light_tank_medium"
    model = ENG_MKIV_light_tank_entity
    design_team = mio:ENG_vauxhall_organization
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("add_equipment_production", HOI4Entity {
        name: "add_equipment_production",
        description: r#"Starts a production line for the specified equipment for the current scope.

**Example:**
```paradox
add_equipment_production = {
    equipment = {
        type = light_cruiser_2
    }
    requested_factories = 1
    progress = 0.95
    amount = 1
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "add_design_template_bonus",
        HOI4Entity {
            name: "add_design_template_bonus",
            description: r#"Add free bonus design discount to given types with a set of uses.

**Example:**
```paradox
add_design_template_bonus = {
  name = air_equipment
  uses = 1
  cost_factor = 0.75
  equipment = small_plane_airframe
  equipment = medium_plane_airframe
  equipment = large_plane_airframe
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("add_equipment_bonus", HOI4Entity {
        name: "add_equipment_bonus",
        description: r#"Adds the specified equipment bonuses to the country. As description the given loc key or the name of given special project will be used. Same usage as in Ideas/National spirits.

**Example:**
```paradox
add_equipment_bonus = {
  project = FROM
  bonus = {
    armor = { # Type of equipment
      armor_value = 3
      soft_attack = 3
      instant = yes
    }
    small_plane_naval_bomber_airframe = {
      air_range = 0.1
      naval_strike_attack = 0.1
    }
  }
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("set_equipment_version_number", HOI4Entity {
        name: "set_equipment_version_number",
        description: r#"Changes current version number for a given equipment type to N. The next equipment variant created from that type will have version number N+1.

**Example:**
```paradox
set_equipment_version_number = {
  type = small_plane_airframe_1
  version = 4
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("destroy_ships", HOI4Entity {
        name: "destroy_ships",
        description: r#"Destroys the specified type and amount of ships controlled by the current scope.

**Example:**
```paradox
destroy_ships = {
    type = destroyer
    count = all
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "transfer_navy",
        HOI4Entity {
            name: "transfer_navy",
            description: r#"Transfers the current scope navy to the specified country.

**Example:**
```paradox
transfer_navy = {
    target = GER
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("transfer_ship", HOI4Entity {
        name: "transfer_ship",
        description: r#"Transfers the specified type of ship from the current scope to the specified country.

**Example:**
```paradox
transfer_ship = {
    prefer_name = "HMS Achilles"
    type = light_cruiser
    target = NZL
    exclude_refitting = no
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("create_ship", HOI4Entity {
        name: "create_ship",
        description: r#"Create a ship from another country and assign it to the reserve fleet. If not set, it will be the scoped country.

**Example:**
```paradox
FRA = {
    create_ship = {
        type = ship_hull_submarine_1
        equipment_variant = "S Class"
        creator = ENG
        name = "My ship name"
    }
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "add_mines",
        HOI4Entity {
            name: "add_mines",
            description: r#"Add mines to a strategic region.

**Example:**
```paradox
add_mines = { region = 42 amount = 100 }
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_ace",
        HOI4Entity {
            name: "add_ace",
            description: r#"Adds an ace for the current scope.

**Example:**
```paradox
add_ace = {
    name = "Amelia"
    surname = "Earhart"
    callsign = "Revenant"
    type = fighter_genius
    is_female = yes
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "unlock_tactic",
        HOI4Entity {
            name: "unlock_tactic",
            description: r#"Unlocks the specified combat tactic for the country.

**Example:**
```paradox
unlock_tactic = tactic_masterful_blitz
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_doctrine_cost_reduction",
        HOI4Entity {
            name: "add_doctrine_cost_reduction",
            description: r#"Adds a limited use cost reduction for doctrines.

**Example:**
```paradox
add_doctrine_cost_reduction = {
	cost_reduction = 0.5
	uses = 2
	category = land_doctrine
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_mastery",
        HOI4Entity {
            name: "add_mastery",
            description: r#"Adds doctrine mastery.

**Example:**
```paradox
add_mastery = {
    amount = 100
    # FILTERS:
    folder = land
    grand_doctrine = mobile_warfare
    sub_doctrine = mobile_infantry
    track = infantry
    index = 1
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_daily_mastery",
        HOI4Entity {
            name: "add_daily_mastery",
            description: r#"Adds doctrine mastery daily for a certain duration.

**Example:**
```paradox
add_daily_mastery = {
    amount = 0.5
    days = 90
    name = CHI_military_affairs_commission_sea
    # FILTERS:
    folder = land
    grand_doctrine = mobile_warfare
    sub_doctrine = mobile_infantry
    track = infantry
    index = 1
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_mastery_bonus",
        HOI4Entity {
            name: "add_mastery_bonus",
            description: r#"Get a bonus to doctrine mastery gain for a certain duration.

**Example:**
```paradox
add_mastery_bonus = {
    bonus = 0.5
    days = 90
    name = CHI_military_affairs_commission_sea
    # FILTERS:
    folder = land
    grand_doctrine = mobile_warfare
    sub_doctrine = mobile_infantry
    track = infantry
    index = 1
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_grand_doctrine",
        HOI4Entity {
            name: "set_grand_doctrine",
            description: r#"Activate (unlock and assign) the specified grand doctrine.

**Example:**
```paradox
set_grand_doctrine = mobile_warfare
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_sub_doctrine",
        HOI4Entity {
            name: "set_sub_doctrine",
            description: r#"Activate (unlock and assign) the specified subdoctrine.

**Example:**
```paradox
set_sub_doctrine = mobile_infantry
```

```paradox
set_sub_doctrine = {
    sub_doctrine = mobile_infantry
    folder = land
    track = 1
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "create_intelligence_agency",
        HOI4Entity {
            name: "create_intelligence_agency",
            description: r#"Creates an Intelligence Agency.

**Example:**
```paradox
create_intelligence_agency = {
    name = "A.G.E.N.C.Y"
    icon = GFX_intelligence_agency_logo_agency
}
```

```paradox
create_intelligence_agency = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "upgrade_intelligence_agency",
        HOI4Entity {
            name: "upgrade_intelligence_agency",
            description: r#"Unlocks an Intelligence Agency Upgrade.

**Example:**
```paradox
upgrade_intelligence_agency = upgrade_form_department
```

```paradox
upgrade_intelligence_agency = <upgrade>
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_decryption",
        HOI4Entity {
            name: "add_decryption",
            description: r#"Adds decryption towards the target country

**Example:**
```paradox
add_decryption = {
    target = GER
    amount = 300
}
```

```paradox
add_decryption = {
    target = GER
    ratio = 0.5
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_intel",
        HOI4Entity {
            name: "add_intel",
            description: r#"Adds the specified amount of intel towards the specified country.

**Example:**
```paradox
add_intel = {
    target = GER
    civilian_intel = 3
    army_intel = 2
    navy_intel = 1
    airforce_intel = 2
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("add_operation_token", HOI4Entity {
        name: "add_operation_token",
        description: r#"Adds an operation token towards the country, allowing access to more intel or applying a targeted modifier.

**Example:**
```paradox
add_operation_token = {
    tag = GER
    token = token_test
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "remove_operation_token",
        HOI4Entity {
            name: "remove_operation_token",
            description: r#"Removes an operation token from the country.

**Example:**
```paradox
remove_operation_token = {
    tag = GER
    token = token_test
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "capture_operative",
        HOI4Entity {
            name: "capture_operative",
            description: r#"Captures the specified operative.

**Example:**
```paradox
capture_operative = {
    operative = PREV
    ignore_death_chance = yes
}
```

```paradox
capture_operative = PREV
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("create_operative_leader", HOI4Entity {
        name: "create_operative_leader",
        description: r#"Creates an operative for the current scope with the specified attributes.

**Example:**
```paradox
create_operative_leader = {
	name = "Jacques Duclos"
	GFX = GFX_portrait_jacques_duclos
	traits = { operative_infiltrator operative_natural_orator }
	bypass_recruitment = no
	available_to_spy_master = yes
	nationalities = { FRA POL }
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "free_operative",
        HOI4Entity {
            name: "free_operative",
            description: r#"Frees the specifies operative.

**Example:**
```paradox
free_operative = PREV
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "free_random_operative",
        HOI4Entity {
            name: "free_random_operative",
            description: r#"Frees one random captured operative or all of them.

**Example:**
```paradox
free_random_operative = {
	captured_by = POL
	all = yes
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "kill_operative",
        HOI4Entity {
            name: "kill_operative",
            description: r#"Kills the targeted operative.

**Example:**
```paradox
kill_operative = {
    operative = PREV
}
```

```paradox
kill_operative = PREV
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("turn_operative", HOI4Entity {
        name: "turn_operative",
        description: r#"Turns the targeted operative against their own country, transferring them to the current country.

**Example:**
```paradox
turn_operative = {
    operative = PREV
}
```

```paradox
turn_operative = PREV
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "steal_random_tech_bonus",
        HOI4Entity {
            name: "steal_random_tech_bonus",
            description: r#"Steals a random tech bonus from the specified country.

**Example:**
```paradox
steal_random_tech_bonus = {
    category = air_equipment
    folder = naval_folder
    ahead_reduction = 0.8
    bonus = 1.2
    base_bonus = 1.1
    dynamic = yes
    name = LOC_KEY
    target = POL
    uses = 2
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_nationality",
        HOI4Entity {
            name: "set_nationality",
            description: r#"Switches the specified character to the specified country.

**Example:**
```paradox
set_nationality = {
    target_country = TZN
    character = OMA_sultan
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("retire_character", HOI4Entity {
        name: "retire_character",
        description: r#"Retires the character, removing every role they hold and making them disappear from the game.

**Example:**
```paradox
retire_character = GER_Character_Token
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "set_character_name",
        HOI4Entity {
            name: "set_character_name",
            description: r#"Sets the new name for the target character.

**Example:**
```paradox
set_character_name = {
	character = my_character
	name = my_name
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("character_list_tooltip", HOI4Entity {
        name: "character_list_tooltip",
        description: r#"Displays a list of every character meeting the specified limitation and recruited by the current country.

**Example:**
```paradox
character_list_tooltip = {
	limit = {
        has_character_flag = SOV_targeted_for_purge_flag
    }
    random_select_amount = 4
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "add_trait",
        HOI4Entity {
            name: "add_trait",
            description: r#"Adds the specified country leader trait to the character.

**Example:**
```paradox
add_trait = {
     character = TAG_jane_smith
     slot = political_advisor
     trait = really_good_boss
}
```

```paradox
add_trait = {
     character = TAG_my_leader
     ideology = liberalism
     trait = field_of_gar
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "remove_trait",
        HOI4Entity {
            name: "remove_trait",
            description: r#"Removes the specified trait from the character.

**Example:**
```paradox
remove_trait = {
    character = TAG_jane_smith
    slot = political_advisor
    trait = really_good_boss
}
```

```paradox
remove_trait = {
     character = TAG_my_leader
     ideology = liberalism
     trait = field_of_gar
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "create_corps_commander",
        HOI4Entity {
            name: "create_corps_commander",
            description: r#"Creates a commander for the current scope with the specified attributes.

**Example:**
```paradox
create_corps_commander = {
	name = "Jean de Lattre de Tassigny"
	picture = "Portrait_France_Jean_de_Lattre_de_Tassigny.dds"
	traits = { trickster brilliant_strategist }
	skill = 4
	attack_skill = 4
	defense_skill = 2
	planning_skill = 4
	logistics_skill = 3
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("create_field_marshal", HOI4Entity {
        name: "create_field_marshal",
        description: r#"Creates a field marshal for the current scope with the specified attributes.

**Example:**
```paradox
create_field_marshal = {
	name = "Maurice Gamelin"
	portrait_path = "GFX_portrait_FRA_maurice_gamelin"
	traits = { defensive_doctrine }
	skill = 2
	attack_skill = 1
	defense_skill = 3
	planning_skill = 2
	logistics_skill = 1
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("create_navy_leader", HOI4Entity {
        name: "create_navy_leader",
        description: r#"Creates a naval leader for the current scope with the specified attributes.

**Example:**
```paradox
create_navy_leader = {
	name = "François Darlan"
	gfx = "GFX_portrait_FRA_francois_darlan"
	traits = { superior_tactician }
	skill = 3
	attack_skill = 2
	defense_skill = 4
	maneuvering_skill = 3
	coordination_skill = 2
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "remove_unit_leader",
        HOI4Entity {
            name: "remove_unit_leader",
            description: r#"Removes the specified unit leader by their legacy ID.

**Example:**
```paradox
remove_unit_leader = 70
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_corps_commander_role",
        HOI4Entity {
            name: "add_corps_commander_role",
            description: r#"Sets the specified character to also act as a corps commander.

**Example:**
```paradox
add_corps_commander_role = {
    Character = GER_Character_token
    skill = 4
    attack_skill = 2
    defense_skill = 3
    planning_skill = 3
    logistics_skill = 5
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_field_marshal_role",
        HOI4Entity {
            name: "add_field_marshal_role",
            description: r#"Sets the specified character to also act as a field marshal.

**Example:**
```paradox
add_field_marshal_role = {
  character = GER_Character_token
  skill = 4
  attack_skill = 2
  defense_skill = 3
  planning_skill = 3
  logistics_skill = 5
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_naval_commander_role",
        HOI4Entity {
            name: "add_naval_commander_role",
            description: r#"Sets the specified character to also act as an admiral.

**Example:**
```paradox
add_naval_commander_role = {
  Character = GER_Character_token
  skill = 4
  attack_skill = 2
  defense_skill = 3
  planning_skill = 3
  logistics_skill = 5
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "show_unit_leaders_tooltip",
        HOI4Entity {
            name: "show_unit_leaders_tooltip",
            description: r#"Shows the name of the specified character as a tooltip.

**Example:**
```paradox
show_unit_leaders_tooltip = TAG_my_leader
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "create_country_leader",
        HOI4Entity {
            name: "create_country_leader",
            description: r#"

**Example:**
```paradox
create_country_leader = {
	name = AFG_mohammed_zahir_shah
	desc = "POLITICS_MOHAMMED_ZAHIR_SHAH_DESC"
	picture = GFX_AFG_mohammed_zahir_shah
	expire = "1965.1.1"
	ideology = despotism
	traits = {
	}
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("add_country_leader_role", HOI4Entity {
        name: "add_country_leader_role",
        description: r#"Sets the specified character to also act as a country leader, promoting to the party leader if specified.

**Example:**
```paradox
add_country_leader_role = {
    character = GER_character_token
    promote_leader = yes
    country_leader = {
        ideology = fascism_ideology
        expire = "1965.1.1.1"
        traits = { war_industrialist }
    }
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "promote_character",
        HOI4Entity {
            name: "promote_character",
            description: r#"Promotes a character to the leader of their political party.

**Example:**
```paradox
promote_character = GER_erwin_rommel
```

```paradox
promote_character = {
    character = GER_erwin_rommel
    ideology = nazism
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "remove_country_leader_role",
        HOI4Entity {
            name: "remove_country_leader_role",
            description: r#"Removes a country leader role from a character.

**Example:**
```paradox
remove_country_leader_role = {
    character = GER_Character_Token
    ideology = socialism
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("kill_ideology_leader", HOI4Entity {
        name: "kill_ideology_leader",
        description: r#"Kills the country leader of the designated ideology for the current scope.

**Example:**
```paradox
kill_ideology_leader = communism
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("retire_ideology_leader", HOI4Entity {
        name: "retire_ideology_leader",
        description: r#"Retires and removes the country leader of the ideology party for the current scope.

**Example:**
```paradox
retire_ideology_leader = fascism
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "kill_country_leader",
        HOI4Entity {
            name: "kill_country_leader",
            description: r#"Kills the country leader for the current scope.

**Example:**
```paradox
kill_country_leader = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("retire_country_leader", HOI4Entity {
        name: "retire_country_leader",
        description: r#"Retires and removes the country leader as head of their party for the current scope.

**Example:**
```paradox
retire_country_leader = yes
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "set_country_leader_ideology",
        HOI4Entity {
            name: "set_country_leader_ideology",
            description: r#"Changes the country leader's government type for the current scope.

**Example:**
```paradox
set_country_leader_ideology = socialism
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_country_leader_description",
        HOI4Entity {
            name: "set_country_leader_description",
            description: r#"Changes the country leader's description.

**Example:**
```paradox
set_country_leader_description = {
	ideology = neutrality
	desc = LOC_KEY
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_country_leader_name",
        HOI4Entity {
            name: "set_country_leader_name",
            description: r#"Changes the country leader's name.

**Example:**
```paradox
set_country_leader_name = {
	ideology = neutrality
	name = LOC_KEY
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_country_leader_portrait",
        HOI4Entity {
            name: "set_country_leader_portrait",
            description: r#"Changes the country leader's portrait.

**Example:**
```paradox
set_country_leader_portrait = {
	ideology = neutrality
	portrait = GFX_IMAGE_NAME
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_country_leader_trait",
        HOI4Entity {
            name: "add_country_leader_trait",
            description: r#"Adds the specified trait to the current country's country leader.

**Example:**
```paradox
add_country_leader_trait = nationalist_symbol
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "remove_country_leader_trait",
        HOI4Entity {
            name: "remove_country_leader_trait",
            description: r#"Removes the specified trait from the current scope's country leader.

**Example:**
```paradox
remove_country_leader_trait = nationalist_symbol
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "swap_ruler_traits",
        HOI4Entity {
            name: "swap_ruler_traits",
            description: r#"Swaps traits.

**Example:**
```paradox
swap_ruler_traits = { remove = <trait> add = <trait> }
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "activate_advisor",
        HOI4Entity {
            name: "activate_advisor",
            description: r#"Hires an advisor, placing them into their respective slot.

**Example:**
```paradox
activate_advisor = GER_character_token_air_chief
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "deactivate_advisor",
        HOI4Entity {
            name: "deactivate_advisor",
            description: r#"Dismisses an advisor from their respective slot, leaving it empty.

**Example:**
```paradox
deactivate_advisor = GER_character_token_air_chief
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("add_advisor_role", HOI4Entity {
        name: "add_advisor_role",
        description: r#"Sets the specified character to also act as an advisor, activating if specified.

**Example:**
```paradox
add_advisor_role = {
    character = GER_Character_token
    activate = yes
    advisor = {
        slot = air_chief
        cost = 50
        idea_token = GER_character_token_air_chief
        traits = {
            air_chief_ground_support_2
        }
    }
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "remove_advisor_role",
        HOI4Entity {
            name: "remove_advisor_role",
            description: r#"Removes the specified advisor role from the character.

**Example:**
```paradox
remove_advisor_role = {
  character = "SOV_genrikh_yagoda"
  slot = political_advisor
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("set_can_be_fired_in_advisor_role", HOI4Entity {
        name: "set_can_be_fired_in_advisor_role",
        description: r#"Changes the `can_be_fired` attribute of the advisor, preventing the player from dismissing the advisor.

**Example:**
```paradox
set_can_be_fired_in_advisor_role = {
    character = BHR_important_advisor
    value = no
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "add_scientist_role",
        HOI4Entity {
            name: "add_scientist_role",
            description: r#"Adds the scientist role to a character.

**Example:**
```paradox
add_scientist_role = {
  character = my_character / var:my_char_var / PREV
  scientist = {
    desc = desc_loc_key
    traits = { scientist_trait_token ... }
    skills = { specialization_token = 2 ... }
  }
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "remove_scientist_role",
        HOI4Entity {
            name: "remove_scientist_role",
            description: r#"Remove the scientist role from a character.

**Example:**
```paradox
remove_scientist_role = {
  character = my_character / var:my_char_var / PREV
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("generate_scientist_character", HOI4Entity {
        name: "generate_scientist_character",
        description: r#"Generate a new character with a scientist role and recruit it in the country in scope.

**Example:**
```paradox
generate_scientist_character = {
  portrait = GFX_portrait
  portrait_tag_override = CHI
  gender = male
  skills = {
    specialization_token = 2
  }
  traits = { trait_token }
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("show_mio_tooltip", HOI4Entity {
        name: "show_mio_tooltip",
        description: r#"Displays a tooltip that shows the name of the MIO and its initial trait (if present).

**Example:**
```paradox
show_mio_tooltip = my_mio
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "unlock_military_industrial_organization_tooltip",
        HOI4Entity {
            name: "unlock_military_industrial_organization_tooltip",
            description: r#"Display a tooltip saying the MIO is made available (aka unlocked).

**Example:**
```paradox
unlock_military_industrial_organization_tooltip = mio:my_mio_token
```

```paradox
unlock_military_industrial_organization_tooltip = var:my_mio_var
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "unlock_mio_policy_tooltip",
        HOI4Entity {
            name: "unlock_mio_policy_tooltip",
            description: r#"Displays a tooltip that says that the policy is made available.

**Example:**
```paradox
unlock_mio_policy_tooltip = my_policy_1
```

```paradox
unlock_mio_policy_tooltip = {
    policy = my_policy_2
    show_modifiers = no
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_mio_policy_cost",
        HOI4Entity {
            name: "add_mio_policy_cost",
            description: r#"Modifies the base cost of a MIO policy.

**Example:**
```paradox
add_mio_policy_cost = {
    policy = my_policy
    value = 10
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_mio_policy_cost",
        HOI4Entity {
            name: "set_mio_policy_cost",
            description: r#"Modifies the base cost of a MIO policy.

**Example:**
```paradox
set_mio_policy_cost = {
    policy = my_policy
    value = 100
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "add_mio_policy_cooldown",
        HOI4Entity {
            name: "add_mio_policy_cooldown",
            description: r#"Modifies the base length of a MIO policy cooldown.

**Example:**
```paradox
add_mio_policy_cooldown = {
    policy = my_policy
    value = 10
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "set_mio_policy_cooldown",
        HOI4Entity {
            name: "set_mio_policy_cooldown",
            description: r#"Modifies the base length of a MIO policy cooldown.

**Example:**
```paradox
set_mio_policy_cooldown  = {
    policy = my_policy
    value = 100
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("complete_special_project", HOI4Entity {
        name: "complete_special_project",
        description: r#"Complete a special project for the country in scope. This effect will not take into account the current state of the project tree and will allow you to unlock a project even if the one before is not unlocked. Since the project is not completed within a facility, the facility state and scientist effects are NOT applied.

**Example:**
```paradox
complete_special_project = sp:sp_naval_midget_submarine
```

```paradox
complete_special_project = {
  project = sp:sp_naval_midget_submarine
  scientist = ITA_curio_bernardis
  state = my_state
  iteration_output = {
    my_reward
    my_other_reward
    my_third_reward = my_option_1
  }
  show_modifiers = no
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("add_breakthrough_points", HOI4Entity {
        name: "add_breakthrough_points",
        description: r#"Add breakthrough points to one specialization or all for a country scope.

**Example:**
```paradox
add_breakthrough_points = {
  specialization = specialization_land
  value = 3
}
```

```paradox
add_breakthrough_points = {
  specialization = all
  value = 1
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("add_breakthrough_progress", HOI4Entity {
        name: "add_breakthrough_progress",
        description: r#"Add breakthrough progress to one specialization or all for a country scope.

**Example:**
```paradox
add_breakthrough_progress = {
  specialization = specialization_land
  value = 3
}
```

```paradox
add_breakthrough_progress = {
  specialization = all
  value = sp_breakthrough_progress.medium
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "career_profile_step_missiolini",
        HOI4Entity {
            name: "career_profile_step_missiolini",
            description: r#"Step completed Mussolini missions by one for the career profile.

**Example:**
```paradox
career_profile_step_missiolini = yes
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "recruit_character",
        HOI4Entity {
            name: "recruit_character",
            description: r#"Initially assigns the specified character to the current country.

**Example:**
```paradox
recruit_character = GER_Character_token
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert(
        "generate_character",
        HOI4Entity {
            name: "generate_character",
            description: r#"Generates a character for current country.

**Example:**
```paradox
generate_character = {
    token_base = army_chief_defensive_1
    name = funny_name
    advisor = {
        slot = air_chief
        cost = 50
        idea_token = GER_character_token_air_chief
        traits = {
            air_chief_ground_support_2
        }
        allowed = {
            always = yes
        }
    }
}
```"#,
            scopes: &[crate::scope::Scope::Country],
        },
    );
    m.insert("set_oob", HOI4Entity {
        name: "set_oob",
        description: r#"Sets the order of battle to be used for the current country's divisions, overriding every other non-naval and non-air order of battle.

**Example:**
```paradox
set_oob = BHR_1936
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("set_naval_oob", HOI4Entity {
        name: "set_naval_oob",
        description: r#"Sets the order of battle to be used for the current country's divisions, overriding every other naval order of battle.

**Example:**
```paradox
set_naval_oob = BHR_1936_naval_legacy
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("set_air_oob", HOI4Entity {
        name: "set_air_oob",
        description: r#"Sets the order of battle to be used for the current country's divisions, overriding every other air order of battle.

**Example:**
```paradox
set_air_oob = ITA_1936_air_bba
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("set_keyed_oob", HOI4Entity {
        name: "set_keyed_oob",
        description: r#"Sets the order of battle to be used for the current country's divisions, overriding every other keyed order of battle that uses the same key.

**Example:**
```paradox
set_keyed_oob = {
    key = naval
    name = BHR_1936_mtg
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("get_highest_scored_country_temp", HOI4Entity {
        name: "get_highest_scored_country_temp",
        description: r#"Calculates the highest scored country that is defined in a country scorer and sets it to a variable.

**Example:**
```paradox
get_highest_scored_country_temp = {
  scorer = scorer_id
  var = var_name
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("get_sorted_scored_countries_temp", HOI4Entity {
        name: "get_sorted_scored_countries_temp",
        description: r#"Calculates & sorts all countries in a country scorer and stores them and their scores in temp arrays.

**Example:**
```paradox
get_sorted_scored_countries_temp = {
  scorer = scorer_id
  array = array_name
  scores = array_name
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("get_supply_vehicles", HOI4Entity {
        name: "get_supply_vehicles",
        description: r#"Sets a variable to the number of supply vehicles in stockpile or that are needed.

**Example:**
```paradox
get_supply_vehicles = {
  var = trucks_needed
  type = truck
  need = yes
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert("get_supply_vehicles_temp", HOI4Entity {
        name: "get_supply_vehicles_temp",
        description: r#"Sets a temp variable to the number of supply vehicles in stockpile or that are needed.

**Example:**
```paradox
get_supply_vehicles_temp = {
  var = trucks_needed
  type = truck
  need = yes
}
```"#,
        scopes: &[crate::scope::Scope::Country],
    });
    m.insert(
        "state_event",
        HOI4Entity {
            name: "state_event",
            description: r#"Fires the specified event for the current state.

**Example:**
```paradox
state_event = {
    id = my_event.1
    days = 10
    random = 50
    random_days = 10
    trigger_for = controller
}
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "set_state_flag",
        HOI4Entity {
            name: "set_state_flag",
            description: r#"Defines a state flag.

**Example:**
```paradox
set_state_flag = my_flag
```

```paradox
set_state_flag = {
    flag = my_flag
    days = 123
    value = 1
}
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "clr_state_flag",
        HOI4Entity {
            name: "clr_state_flag",
            description: r#"Clears a defined state flag.

**Example:**
```paradox
clr_state_flag = my_flag
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "modify_state_flag",
        HOI4Entity {
            name: "modify_state_flag",
            description: r#"Adds an integer value to a flag.

**Example:**
```paradox
modify_state_flag = {
    flag = my_flag
    value = 3
}
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "set_state_name",
        HOI4Entity {
            name: "set_state_name",
            description: r#"Changes the current state's name to the specified name.

**Example:**
```paradox
set_state_name = "Funland"
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "reset_state_name",
        HOI4Entity {
            name: "reset_state_name",
            description: r#"Resets any changes to the current state's name.

**Example:**
```paradox
reset_state_name = yes
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "add_claim_by",
        HOI4Entity {
            name: "add_claim_by",
            description: r#"Adds a claim for the specified country on the current scope.

**Example:**
```paradox
add_claim_by = SOV
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "remove_claim_by",
        HOI4Entity {
            name: "remove_claim_by",
            description: r#"Removes a claim by the specified country on the current scope.

**Example:**
```paradox
remove_claim_by = SOV
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "add_core_of",
        HOI4Entity {
            name: "add_core_of",
            description: r#"Adds a core for the specified country on the current scope.

**Example:**
```paradox
add_core_of = SOV
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "remove_core_of",
        HOI4Entity {
            name: "remove_core_of",
            description: r#"Removes a core for the specified country on the current scope.

**Example:**
```paradox
remove_core_of = SOV
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "set_demilitarized_zone",
        HOI4Entity {
            name: "set_demilitarized_zone",
            description: r#"Makes the current scope a demilitarized zone.

**Example:**
```paradox
set_demilitarized_zone = yes
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "set_state_category",
        HOI4Entity {
            name: "set_state_category",
            description: r#"Changes the current state category to the specified category.

**Example:**
```paradox
set_state_category = rural
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "add_state_modifier",
        HOI4Entity {
            name: "add_state_modifier",
            description: r#"Adds a modifier to the current state.

**Example:**
```paradox
add_state_modifier = {
    modifier = {
        local_resources = 2.0
    }
}
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "set_border_war",
        HOI4Entity {
            name: "set_border_war",
            description: r#"Enables Border War status for the current state.

**Example:**
```paradox
set_border_war = yes
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert("create_unit", HOI4Entity {
        name: "create_unit",
        description: r#"Adds the specified division to the current state.

**Example:**
```paradox
create_unit = {
    division = "name = \"Infantry Division\" division_template = \"Infantry Division\" start_experience_factor = 0.5"
    owner = GER
}
```

```paradox
create_unit = {
    division = "name = \"Artie\" division_template = \"Artillery Division\" start_manpower_factor = 0.3"
    owner = BHR
    count = 3
    allow_spawning_on_enemy_provs = yes
    country_score = {
        base = 3
        modifier = {
            factor = 2
            tag = OMA
        }
    }
    id = 123
}
```

```paradox
create_unit = {
  division = "name = \"Tank division\" division_template = \"Tank Division\" start_manpower_factor = 1 force_equipment_variants = { medium_tank_chassis_2 = { owner = \"USA\" amount = 100 version_name = \"M4 Sherman\" }}"
  owner = USA
  count = 1
}
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert("teleport_armies", HOI4Entity {
        name: "teleport_armies",
        description: r#"Teleports all armies in the specified state if the owner of the armies meets the condition.

**Example:**
```paradox
teleport_armies = {
    limit = {
        has_war_together_with = ROOT
    }
    to_state_array = owned_controlled_states
}
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert(
        "add_province_modifier",
        HOI4Entity {
            name: "add_province_modifier",
            description: r#"Adds a province modifier to the specified provinces in this state.

**Example:**
```paradox
add_province_modifier = {
	static_modifiers = { mod_modifier_1 mod_modifier_2 }
	province = 1234
}
```

```paradox
add_province_modifier = {

static_modifiers = { mod_modifier_1 mod_modifier_2 }
	province = {
		id = 1234
		id = 4321

```paradox
days = 7
```

}

}
```

```paradox
add_province_modifier = {

static_modifiers = { mod_modifier_1 mod_modifier_2 }
	province = {
		all_provinces = yes
		limit_to_coastal = yes
		limit_to_border = yes
		limit_to_naval_base = yes
		limit_to_victory_point = yes
	}

}
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "remove_province_modifier",
        HOI4Entity {
            name: "remove_province_modifier",
            description: r#"Removes a province modifier to the specified provinces in this state.

**Example:**
```paradox
remove_province_modifier = {
	static_modifiers = { mod_modifier_1 mod_modifier_2 }
	province = 1234
}
```

```paradox
remove_province_modifier = {

static_modifiers = { mod_modifier_1 mod_modifier_2 }
	province = {
		id = 1234
		id = 4321
	}

}
```

```paradox
remove_province_modifier = {

static_modifiers = { mod_modifier_1 mod_modifier_2 }
	province = {
		all_provinces = yes
		limit_to_coastal = yes
		limit_to_border = yes
		limit_to_naval_base = yes
		limit_to_victory_point = yes
	}

}
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "add_victory_points",
        HOI4Entity {
            name: "add_victory_points",
            description: r#"Adds victory points to a province.

**Example:**
```paradox
add_victory_points = {
	province = 1234
	value = 10
}
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "set_victory_points",
        HOI4Entity {
            name: "set_victory_points",
            description: r#"Sets the number of victory point in a province.

**Example:**
```paradox
set_victory_points = {
	province = 1234
	value = 10
}
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert("set_state_province_controller", HOI4Entity {
        name: "set_state_province_controller",
        description: r#"Changes the controller of all provinces within that state controlled by countries that meet triggers to the specified country.

**Example:**
```paradox
set_state_province_controller = {
    controller = POL
    limit = {
        OR = {
            tag = GER
            is_in_faction_with = GER
        }
    }
}
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert(
        "transfer_state_to",
        HOI4Entity {
            name: "transfer_state_to",
            description: r#"Sets owner and controller of the state to the given country

**Example:**
```paradox
transfer_state_to = JAM
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "set_state_owner_to",
        HOI4Entity {
            name: "set_state_owner_to",
            description: r#"Sets the owner of the state to the given country

**Example:**
```paradox
set_state_owner_to = JAM
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "set_state_controller_to",
        HOI4Entity {
            name: "set_state_controller_to",
            description: r#"Sets the controller of the state to the given country

**Example:**
```paradox
set_state_controller_to = ITA
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert("strategic_province_location", HOI4Entity {
        name: "strategic_province_location",
        description: r#"Add a strategic location to a province using state scope. The available strategic locations are defined in strategic_locations and are specified with a province id.

**Example:**
```paradox
strategic_province_location = {
    defensible_coastline = 10124
}
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert("strategic_state_location", HOI4Entity {
        name: "strategic_state_location",
        description: r#"Add strategic locations to a state in scope. The available strategic locations are defined in strategic_locations.

**Example:**
```paradox
strategic_state_location = {
    favorable_approach = 11932
}
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert(
        "add_extra_state_shared_building_slots",
        HOI4Entity {
            name: "add_extra_state_shared_building_slots",
            description: r#"Changes the number of shared building slots for the current state.

**Example:**
```paradox
add_extra_state_shared_building_slots = 2
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "add_building_construction",
        HOI4Entity {
            name: "add_building_construction",
            description: r#"Starts construction in the current state for the specified building.

**Example:**
```paradox
add_building_construction = {
    type = arms_factory
    level = 5
    instant_build = yes
}
```

```paradox
add_building_construction = {
    type = bunker
    level = 10
    instant_build = yes
    province = {
        all_provinces = yes
        limit_to_border = yes
        limit_to_victory_point > 1
    }
}
```

```paradox
add_building_construction = {
    type = bunker
    level = 1
    instant_build = yes
    province = 2999
}
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert("set_building_level", HOI4Entity {
        name: "set_building_level",
        description: r#"Sets the specified building to the current state (or provinces within the state).

**Example:**
```paradox
set_building_level = {
    type = infrastructure
    level = 10
    instant_build = yes
}
```

```paradox
set_building_level = {
    type = bunker
    level = 3
    province = {
        all_provinces = yes
        limit_to_border = yes
        level < 3
    }
}
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert("remove_building", HOI4Entity {
        name: "remove_building",
        description: r#"Removes the specified building in the current state. For shared buildings level determines the amount, whereas for the others it is the actual level.

**Example:**
```paradox
remove_building = {
    type = arms_factory
    level = 5
}
```

```paradox
remove_building = {
    tag = facility
    level = 1
}
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert(
        "construct_building_in_random_province",
        HOI4Entity {
            name: "construct_building_in_random_province",
            description: r#"Set building level in a random province of state scope.

**Example:**
```paradox
65 = {
    construct_building_in_random_province = {
        land_facility = 1
    }
}
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "add_compliance",
        HOI4Entity {
            name: "add_compliance",
            description: r#"Adds compliance to the specified state.

**Example:**
```paradox
add_compliance = 30
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "add_resistance",
        HOI4Entity {
            name: "add_resistance",
            description: r#"Adds resistance to the specified state.

**Example:**
```paradox
add_resistance = 30
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "add_resistance_target",
        HOI4Entity {
            name: "add_resistance_target",
            description: r#"Increases resistance target in the specified state.

**Example:**
```paradox
add_resistance_target = 30
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "cancel_resistance",
        HOI4Entity {
            name: "cancel_resistance",
            description: r#"Cancels resistance activity for the current state.

**Example:**
```paradox
cancel_resistance = yes
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert("force_disable_resistance", HOI4Entity {
        name: "force_disable_resistance",
        description: r#"Disables resistance for the scoped state when the occupier is the specified country.

**Example:**
```paradox
force_disable_resistance = GER
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert("force_enable_resistance", HOI4Entity {
        name: "force_enable_resistance",
        description: r#"Enables resistance for the scoped state when the occupier is the specified country.

**Example:**
```paradox
force_enable_resistance = GER
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert(
        "remove_resistance_target",
        HOI4Entity {
            name: "remove_resistance_target",
            description: r#"Removes a set resistance target increase in the specified state.

**Example:**
```paradox
remove_resistance_target = 30
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "set_compliance",
        HOI4Entity {
            name: "set_compliance",
            description: r#"Sets compliance in the specified state.

**Example:**
```paradox
set_compliance = 30
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "set_resistance",
        HOI4Entity {
            name: "set_resistance",
            description: r#"Sets resistance in the specified state.

**Example:**
```paradox
set_resistance = 30
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "start_resistance",
        HOI4Entity {
            name: "start_resistance",
            description: r#"Starts resistance in the specified state.

**Example:**
```paradox
start_resistance = POL
```

```paradox
start_resistance = yes
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert(
        "set_garrison_strength",
        HOI4Entity {
            name: "set_garrison_strength",
            description: r#"Sets the strength of the garrison in the specified state.

**Example:**
```paradox
set_garrison_strength = 0.5
```"#,
            scopes: &[crate::scope::Scope::State],
        },
    );
    m.insert("raid_reduce_project_progress_ratio", HOI4Entity {
        name: "raid_reduce_project_progress_ratio",
        description: r#"Reduce progress to the special project in state. Root scope is raid instance scope. The input value is a ratio of the total needed progress to complete the special project, i.e. a decimal number between 0 and 1.

**Example:**
```paradox
raid_reduce_project_progress_ratio = 0.1
```"#,
        scopes: &[crate::scope::Scope::State],
    });
    m.insert(
        "set_character_flag",
        HOI4Entity {
            name: "set_character_flag",
            description: r#"Defines a character flag.

**Example:**
```paradox
set_character_flag = my_flag
```

```paradox
set_character_flag = {
    flag = my_flag
    days = 123
    value = 1
}
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "modify_character_flag",
        HOI4Entity {
            name: "modify_character_flag",
            description: r#"Adds an integer value to a flag.

**Example:**
```paradox
modify_character_flag = {
    flag = my_flag
    value = 3
}
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "clr_character_flag",
        HOI4Entity {
            name: "clr_character_flag",
            description: r#"Clears a character flag

**Example:**
```paradox
clr_character_flag = <bool>
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "retire",
        HOI4Entity {
            name: "retire",
            description: r#"Retires the current character (removing them).

**Example:**
```paradox
retire = yes
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "set_portraits",
        HOI4Entity {
            name: "set_portraits",
            description: r#"Changes the specified portraits of a character.

**Example:**
```paradox
set_portraits = {
    character = my_character
    army = { small ="MySmallCharacterGFX" }
    civilian = { large ="MyLargeCharacterGFX" }
}
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert("add_scientist_level", HOI4Entity {
        name: "add_scientist_level",
        description: r#"Add levels to a special project specialization for a scientist character in scope.

**Example:**
```paradox
add_scientist_level = {
  level = 2
  specialization = specialization_nuclear
}
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert("injure_scientist_for_days", HOI4Entity {
        name: "injure_scientist_for_days",
        description: r#"Injure a scientist for x amount of days to a scientist character in scope.

**Example:**
```paradox
injure_scientist_for_days = 12
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert(
        "add_scientist_trait",
        HOI4Entity {
            name: "add_scientist_trait",
            description: r#"Add a trait to a scientist character in scope.

**Example:**
```paradox
add_scientist_trait = my_trait_token
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert("add_scientist_xp", HOI4Entity {
        name: "add_scientist_xp",
        description: r#"Add experience to a special project specialization for a scientist character in scope.

**Example:**
```paradox
add_scientist_xp = {
  experience = 2
  specialization = specialization_nuclear
}
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert(
        "unit_leader_event",
        HOI4Entity {
            name: "unit_leader_event",
            description: r#"Fires the specified event for the owner of the current unit leader.

**Example:**
```paradox
unit_leader_event = {
    id = my_event.1
    days = 10
    random = 50
    random_days = 10
}
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "set_unit_leader_flag",
        HOI4Entity {
            name: "set_unit_leader_flag",
            description: r#"Defines a unit leader flag.

**Example:**
```paradox
set_unit_leader_flag = my_flag
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "clr_unit_leader_flag",
        HOI4Entity {
            name: "clr_unit_leader_flag",
            description: r#"Clears a defined unit leader flag.

**Example:**
```paradox
clr_unit_leader_flag = my_flag
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "modify_unit_leader_flag",
        HOI4Entity {
            name: "modify_unit_leader_flag",
            description: r#"Adds an integer value to a flag.

**Example:**
```paradox
modify_unit_leader_flag = {
    flag = my_flag
    value = 3
}
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "promote_leader",
        HOI4Entity {
            name: "promote_leader",
            description: r#"Promotes the current unit leader to Field Marshal (if Commander).

**Example:**
```paradox
promote_leader = yes
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "demote_leader",
        HOI4Entity {
            name: "demote_leader",
            description: r#"Demotes the current unit leader to Commander (if Field Marshal).

**Example:**
```paradox
demote_leader = yes
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "add_unit_leader_trait",
        HOI4Entity {
            name: "add_unit_leader_trait",
            description: r#"Adds the specified trait to the current unit leader.

**Example:**
```paradox
add_unit_leader_trait = old_guard
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "remove_unit_leader_trait",
        HOI4Entity {
            name: "remove_unit_leader_trait",
            description: r#"Removes the specified trait from the current unit leader.

**Example:**
```paradox
remove_unit_leader_trait = old_guard
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "add_random_trait",
        HOI4Entity {
            name: "add_random_trait",
            description: r#"Adds a random trait from the list to the character.

**Example:**
```paradox
add_random_trait = { old_guard brilliant_strategist inflexible_strategist }
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert("add_timed_unit_leader_trait", HOI4Entity {
        name: "add_timed_unit_leader_trait",
        description: r#"Adds the specified trait to the current unit leader for the specified duration.

**Example:**
```paradox
add_timed_unit_leader_trait = {
    trait = wounded
    days = 90
}
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert(
        "replace_unit_leader_trait",
        HOI4Entity {
            name: "replace_unit_leader_trait",
            description: r#"Replaces the specified trait with the new trait.

**Example:**
```paradox
replace_unit_leader_trait = {
    trait = old_guard
    replace = brilliant_strategist
}
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "remove_exile_tag",
        HOI4Entity {
            name: "remove_exile_tag",
            description: r#"Removes a leaders exile tag.

**Example:**
```paradox
remove_exile_tag = yes
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert("gain_xp", HOI4Entity {
        name: "gain_xp",
        description: r#"Adds experience to the current unit leader, promoting to the next skill level if applicable.

**Example:**
```paradox
gain_xp = 5
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert(
        "remove_unit_leader_role",
        HOI4Entity {
            name: "remove_unit_leader_role",
            description: r#"Removes every unit leader role from the character

**Example:**
```paradox
remove_unit_leader_role = yes
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "swap_country_leader_traits",
        HOI4Entity {
            name: "swap_country_leader_traits",
            description: r#"Swaps traits of the current character.

**Example:**
```paradox
swap_country_leader_traits = {
    remove = nationalist_symbol
    add = anti_communist
    ideology = marxism
}
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert("supply_units", HOI4Entity {
        name: "supply_units",
        description: r#"Adds the specified amount of hours of supply to troops led by the current unit leader.

**Example:**
```paradox
supply_units = 24
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert("add_max_trait", HOI4Entity {
        name: "add_max_trait",
        description: r#"Adds the specified amount of assignable trait slots to the current unit leader.

**Example:**
```paradox
add_max_trait = 1
```"#,
        scopes: &[crate::scope::Scope::Character],
    });
    m.insert(
        "add_skill_level",
        HOI4Entity {
            name: "add_skill_level",
            description: r#"Adds skill to the current unit leader.

**Example:**
```paradox
add_skill_level = 1
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "add_logistics",
        HOI4Entity {
            name: "add_logistics",
            description: r#"Adds logistics skill to the current unit leader.

**Example:**
```paradox
add_logistics = 1
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "add_planning",
        HOI4Entity {
            name: "add_planning",
            description: r#"Adds planning skill to the current unit leader.

**Example:**
```paradox
add_planning = 1
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "add_defense",
        HOI4Entity {
            name: "add_defense",
            description: r#"Adds defense skill to the current unit leader.

**Example:**
```paradox
add_defense = 1
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "add_attack",
        HOI4Entity {
            name: "add_attack",
            description: r#"Adds attack skill to the current unit leader.

**Example:**
```paradox
add_attack = 1
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "add_coordination",
        HOI4Entity {
            name: "add_coordination",
            description: r#"Adds coordination skill to the current navy leader.

**Example:**
```paradox
add_coordination = 1
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "add_maneuver",
        HOI4Entity {
            name: "add_maneuver",
            description: r#"Adds maneuver skill to the current navy leader.

**Example:**
```paradox
add_maneuver = 1
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "add_temporary_buff_to_units",
        HOI4Entity {
            name: "add_temporary_buff_to_units",
            description: r#"Adds the specified combat buff to the current unit leader.

**Example:**
```paradox
add_temporary_buff_to_units = {
    combat_offense = 0.25
    combat_breakthrough = 0.25
    org_damage_multiplier = -1.0
    str_damage_multiplier = 0.25
    war_support_reduction_on_damage = 0.2
    cannot_retreat_while_attacking = 1.0
				
    days = 7
    tooltip = ABILITY_FORCE_ATTACK_TOOLTIP
}
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "add_nationality",
        HOI4Entity {
            name: "add_nationality",
            description: r#"Adds the nationality to the current operative.

**Example:**
```paradox
add_nationality = GER
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "force_operative_leader_into_hiding",
        HOI4Entity {
            name: "force_operative_leader_into_hiding",
            description: r#"Forces the current operative into hiding.

**Example:**
```paradox
force_operative_leader_into_hiding = yes
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "harm_operative_leader",
        HOI4Entity {
            name: "harm_operative_leader",
            description: r#"Harms the current operative.

**Example:**
```paradox
harm_operative_leader = 12
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "operative_leader_event",
        HOI4Entity {
            name: "operative_leader_event",
            description: r#"Fires the specified event for the operative.

**Example:**
```paradox
operative_leader_event = {
    id = my_event.1
	originator = POL
	recipient = GER
    days = 10
    random = 50
    random_days = 10
	set_from = ENG
	set_root = SOV
	set_from_from = FRA
}
```"#,
            scopes: &[crate::scope::Scope::Character],
        },
    );
    m.insert(
        "destroy_unit",
        HOI4Entity {
            name: "destroy_unit",
            description: r#"Destroys the currently-scoped division.

**Example:**
```paradox
destroy_unit = yes
```"#,
            scopes: &[crate::scope::Scope::Unit],
        },
    );
    m.insert(
        "add_history_entry",
        HOI4Entity {
            name: "add_history_entry",
            description: r#"Creates an entry within the command history of a division.

**Example:**
```paradox
add_history_entry = {
    key = my_history_entry
    subject = "Test entry"
    allow = no
}
```"#,
            scopes: &[crate::scope::Scope::Unit],
        },
    );
    m.insert(
        "change_division_template",
        HOI4Entity {
            name: "change_division_template",
            description: r#"Changes the template of the division to the specified one.

**Example:**
```paradox
change_division_template = {
    division_template = "New template"
}
```"#,
            scopes: &[crate::scope::Scope::Unit],
        },
    );
    m.insert(
        "add_random_valid_trait_from_unit",
        HOI4Entity {
            name: "add_random_valid_trait_from_unit",
            description: r#"Adds a random valid unit trait to a unit leader.

**Example:**
```paradox
add_random_valid_trait_from_unit = FROM
```"#,
            scopes: &[crate::scope::Scope::Unit],
        },
    );
    m.insert(
        "add_unit_medal_to_latest_entry",
        HOI4Entity {
            name: "add_unit_medal_to_latest_entry",
            description: r#"Adds the specified medal to the latest entry within the unit's history.

**Example:**
```paradox
add_unit_medal_to_latest_entry = {
    unit_medals = my_medal
}
```"#,
            scopes: &[crate::scope::Scope::Unit],
        },
    );
    m.insert(
        "add_divisional_commander_xp",
        HOI4Entity {
            name: "add_divisional_commander_xp",
            description: r#"Adds the specified amount of experience to the divisional commander.

**Example:**
```paradox
add_divisional_commander_xp = 10
```"#,
            scopes: &[crate::scope::Scope::Unit],
        },
    );
    m.insert(
        "reseed_division_commander",
        HOI4Entity {
            name: "reseed_division_commander",
            description: r#"Re-randomises the division commander using the given seed.

**Example:**
```paradox
reseed_division_commander = 760
```"#,
            scopes: &[crate::scope::Scope::Unit],
        },
    );
    m.insert(
        "promote_officer_to_general",
        HOI4Entity {
            name: "promote_officer_to_general",
            description: r#"Promote the officer of the division to a general.

**Example:**
```paradox
promote_officer_to_general = yes
```"#,
            scopes: &[crate::scope::Scope::Unit],
        },
    );
    m.insert(
        "set_unit_organization",
        HOI4Entity {
            name: "set_unit_organization",
            description: r#"Changes the organisation of the unit.

**Example:**
```paradox
set_unit_organization = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unit],
        },
    );
    m.insert(
        "add_mio_funds",
        HOI4Entity {
            name: "add_mio_funds",
            description: r#"Adds funds to the MIO.

**Example:**
```paradox
add_mio_funds = 1000
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "set_mio_funds",
        HOI4Entity {
            name: "set_mio_funds",
            description: r#"Sets the funds of a MIO to the certain level.

**Example:**
```paradox
set_mio_funds = 1000
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "add_mio_funds_gain_factor",
        HOI4Entity {
            name: "add_mio_funds_gain_factor",
            description: r#"Changes the base multiplier to MIO's funds.

**Example:**
```paradox
add_mio_funds_gain_factor = 0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "set_mio_funds_gain_factor",
        HOI4Entity {
            name: "set_mio_funds_gain_factor",
            description: r#"Changes the base multiplier to MIO's funds.

**Example:**
```paradox
set_mio_funds = 0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "add_mio_size",
        HOI4Entity {
            name: "add_mio_size",
            description: r#"Adds sizes to the MIO.

**Example:**
```paradox
add_mio_size = 2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "add_mio_size_up_requirement_factor",
        HOI4Entity {
            name: "add_mio_size_up_requirement_factor",
            description: r#"Changes the base multiplier to the requirement to size up a MIO.

**Example:**
```paradox
add_mio_size_up_requirement_factor = 0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "set_mio_size_up_requirement_factor",
        HOI4Entity {
            name: "set_mio_size_up_requirement_factor",
            description: r#"Changes the base multiplier to the requirement to size up a MIO.

**Example:**
```paradox
set_mio_size_up_requirement_factor = 0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "add_mio_task_capacity",
        HOI4Entity {
            name: "add_mio_task_capacity",
            description: r#"Changes the base maximum task capacity of the MIO.

**Example:**
```paradox
add_mio_task_capacity = 2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "set_mio_task_capacity",
        HOI4Entity {
            name: "set_mio_task_capacity",
            description: r#"Changes the base maximum task capacity of the MIO.

**Example:**
```paradox
set_mio_task_capacity = 2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "add_mio_research_bonus",
        HOI4Entity {
            name: "add_mio_research_bonus",
            description: r#"Changes the base research bonus of the MIO.

**Example:**
```paradox
add_mio_research_bonus = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "set_mio_research_bonus",
        HOI4Entity {
            name: "set_mio_research_bonus",
            description: r#"Changes the base research bonus of the MIO.

**Example:**
```paradox
set_mio_research_bonus = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "set_mio_name_key",
        HOI4Entity {
            name: "set_mio_name_key",
            description: r#"Changes the name of the MIO.

**Example:**
```paradox
set_mio_name_key = mio_new_name
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "set_mio_icon",
        HOI4Entity {
            name: "set_mio_icon",
            description: r#"Changes the MIO's icon.

**Example:**
```paradox
set_mio_icon = GFX_new_mio_icon
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "add_mio_design_team_assign_cost",
        HOI4Entity {
            name: "add_mio_design_team_assign_cost",
            description: r#"Changes the base political power cost of the MIO to assign research.

**Example:**
```paradox
add_mio_design_team_assign_cost = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "set_mio_design_team_assign_cost",
        HOI4Entity {
            name: "set_mio_design_team_assign_cost",
            description: r#"Changes the base political power cost of the MIO to assign research.

**Example:**
```paradox
set_mio_design_team_assign_cost = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("add_mio_industrial_manufacturer_assign_cost", HOI4Entity {
        name: "add_mio_industrial_manufacturer_assign_cost",
        description: r#"Changes the base political power cost of the MIO to assign production lines.

**Example:**
```paradox
add_mio_industrial_manufacturer_assign_cost = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("set_mio_industrial_manufacturer_assign_cost", HOI4Entity {
        name: "set_mio_industrial_manufacturer_assign_cost",
        description: r#"Changes the base political power cost of the MIO to assign production lines.

**Example:**
```paradox
set_mio_industrial_manufacturer_assign_cost = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("add_mio_design_team_change_cost", HOI4Entity {
        name: "add_mio_design_team_change_cost",
        description: r#"Changes the base experience cost of the MIO to assign to equipment by a percentage.

**Example:**
```paradox
add_mio_design_team_change_cost = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("set_mio_design_team_change_cost", HOI4Entity {
        name: "set_mio_design_team_change_cost",
        description: r#"Changes the base experience cost of the MIO to assign to equipment by a percentage.

**Example:**
```paradox
set_mio_design_team_change_cost = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "unlock_mio_trait_tooltip",
        HOI4Entity {
            name: "unlock_mio_trait_tooltip",
            description: r#"Displays a tooltip that says that the trait is made available.

**Example:**
```paradox
unlock_mio_trait_tooltip = my_trait_1
```

```paradox
unlock_mio_trait_tooltip = {
    trait = my_trait_2
    show_modifiers = no
}
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "complete_mio_trait",
        HOI4Entity {
            name: "complete_mio_trait",
            description: r#"Completes the specified MIO trait.

**Example:**
```paradox
complete_mio_trait = my_trait_1
```

```paradox
complete_mio_trait = {
    trait = my_trait_2
    show_modifiers = no
}
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "set_mio_flag",
        HOI4Entity {
            name: "set_mio_flag",
            description: r#"Defines a MIO flag.

**Example:**
```paradox
set_mio_flag = my_flag
```

```paradox
set_mio_flag = {
    flag = my_flag
    days = 123
    value = 1
}
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "clr_mio_flag",
        HOI4Entity {
            name: "clr_mio_flag",
            description: r#"Clears a defined MIO flag.

**Example:**
```paradox
clr_mio_flag = my_flag
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "modify_mio_flag",
        HOI4Entity {
            name: "modify_mio_flag",
            description: r#"Adds an integer value to a flag.

**Example:**
```paradox
modify_mio_flag = {
    flag = my_flag
    value = 3
}
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "cancel_purchase_contract",
        HOI4Entity {
            name: "cancel_purchase_contract",
            description: r#"Cancels the current purchase contract.

**Example:**
```paradox
cancel_purchase_contract = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "add_raid_history_entry",
        HOI4Entity {
            name: "add_raid_history_entry",
            description: r#"Add history entry to a raid.

**Example:**
```paradox
add_raid_history_entry = yes/no
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("raid_add_unit_experience", HOI4Entity {
        name: "raid_add_unit_experience",
        description: r#"Will give experience to any type of unit assigned to the raid, e.g. divisions or air wings.

**Example:**
```paradox
raid_add_unit_experience = 0.2
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("raid_damage_units", HOI4Entity {
        name: "raid_damage_units",
        description: r#"Damage is applied to ground units while damage to plane is defined as the amount of planes lost.

**Example:**
```paradox
# Apply 50% damage to units
raid_damage_units = {
	damage = 0.5
	ratio = yes
}

# Apply 10 strength loss and 20 organization loss to units
raid_damage_units = {
	org_damage = 20
	str_damage = 10
}

# Lose 40% of all planes
raid_damage_units = {
	plane_loss = 0.4
	ratio = yes
}

# Lose 5 planes
raid_damage_units = {
	plane_loss = 5
}
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "add_project_progress_ratio",
        HOI4Entity {
            name: "add_project_progress_ratio",
            description: r#"Add progress to the project's prototype phase.

**Example:**
```paradox
sp:my_project = {
  add_project_progress_ratio = 0.1
  add_project_progress_ratio = var:my_var
}
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "complete_prototype_reward_option",
        HOI4Entity {
            name: "complete_prototype_reward_option",
            description: r#"Complete a prototype reward option for the project in scope

**Example:**
```paradox
complete_prototype_reward_option = {
	prototype_reward = my_reward
	prototyp_reward_option = my_option
	show_modifiers = yes
}
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "set_project_flag",
        HOI4Entity {
            name: "set_project_flag",
            description: r#"Defines a project flag.

**Example:**
```paradox
set_project_flag = my_flag
```

```paradox
set_project_flag = {
    flag = my_flag
    days = 123
    value = 1
}
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "clr_project_flag",
        HOI4Entity {
            name: "clr_project_flag",
            description: r#"Clears a defined project flag.

**Example:**
```paradox
clr_project_flag = my_flag
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "modify_project_flag",
        HOI4Entity {
            name: "modify_project_flag",
            description: r#"Adds an integer value to a flag.

**Example:**
```paradox
modify_mproject_flag = {
    flag = my_flag
    value = 3
}
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("execute_operation_coordinated_strike", HOI4Entity {
        name: "execute_operation_coordinated_strike",
        description: r#"All prepared Port Strike and Strategic Bombing in the target region will execute multiple times without air defence being able to intercept them.

**Example:**
```paradox
execute_operation_coordinated_strike = {
    amount = 12
}
```"#,
        scopes: &[crate::scope::Scope::Global, crate::scope::Scope::Country, crate::scope::Scope::State, crate::scope::Scope::Character, crate::scope::Scope::Unit],
    });
    m.insert(
        "instantiate_collaboration_government",
        HOI4Entity {
            name: "instantiate_collaboration_government",
            description: r#"Creates a collaboration government, with the current scope as overlord.

**Example:**
```paradox
instantiate_collaboration_government = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "add_potential_special_forces_tree",
        HOI4Entity {
            name: "add_potential_special_forces_tree",
            description: r#"Adds 1 special forces branch specialism

**Example:**
```paradox
add_potential_special_forces_tree = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "upgrade_economy_law",
        HOI4Entity {
            name: "upgrade_economy_law",
            description: r#"Switches the economy law one level towards total mobilisation.

**Example:**
```paradox
upgrade_economy_law = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "gain_random_agency_upgrade",
        HOI4Entity {
            name: "gain_random_agency_upgrade",
            description: r#"Grants a random available intelligence agency upgrade.

**Example:**
```paradox
gain_random_agency_upgrade = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("add_ruling_to_dem", HOI4Entity {
        name: "add_ruling_to_dem",
        description: r#"All of the ruling party's popularity gets added to the Democratic ideology group.

**Example:**
```paradox
add_ruling_to_dem = yes
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "remove_any_country_role_from_character",
        HOI4Entity {
            name: "remove_any_country_role_from_character",
            description: r#"Removes all advisor roles from the current scope.

**Example:**
```paradox
remove_any_country_role_from_character = yes
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("increase_state_category", HOI4Entity {
        name: "increase_state_category",
        description: r#"Changes the state category to the next one that contains more building slots.

**Example:**
```paradox
increase_state_category = yes
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("lerp", HOI4Entity {
        name: "lerp",
        description: r#"Creates the `lerp_result` regular variable with     r e s u l t := a + ( b − −  a ) ⋅ ⋅  x   {\displaystyle result:=a+(b-a)\cdot x}

**Example:**
```paradox
lerp = yes
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("store_core_states_on_game_start", HOI4Entity {
        name: "store_core_states_on_game_start",
        description: r#"Stores the current core states of the current scope in an array in ROOT's scope.

**Example:**
```paradox
store_core_states_on_game_start = yes
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m
}

pub fn get_modifiers() -> HashMap<&'static str, HOI4Entity> {
    let mut m = HashMap::new();
    m.insert(
        "monthly_population",
        HOI4Entity {
            name: "monthly_population",
            description: r#"Changes the monthly population gain in states owned by the country.

**Example:**
```paradox
monthly_population = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "nuclear_production",
        HOI4Entity {
            name: "nuclear_production",
            description: r#"Enables the production of nukes.

**Example:**
```paradox
nuclear_production = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "nuclear_production_factor",
        HOI4Entity {
            name: "nuclear_production_factor",
            description: r#"Changes speed at which nukes are produced.

**Example:**
```paradox
nuclear_production_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "research_sharing_per_country_bonus",
        HOI4Entity {
            name: "research_sharing_per_country_bonus",
            description: r#"Changes the bonus in research speed per country when technology sharing.

**Example:**
```paradox
research_sharing_per_country_bonus = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("research_sharing_per_country_bonus_factor", HOI4Entity {
        name: "research_sharing_per_country_bonus_factor",
        description: r#"Changes the bonus in research speed per country when technology sharing by a percentage.

**Example:**
```paradox
research_sharing_per_country_bonus_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "research_speed_factor",
        HOI4Entity {
            name: "research_speed_factor",
            description: r#"Changes the research speed.

**Example:**
```paradox
research_speed_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("local_resources_factor", HOI4Entity {
        name: "local_resources_factor",
        description: r#"Resource extraction efficiency. Modifies the amount of available resources.

**Example:**
```paradox
local_resources_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("surrender_limit", HOI4Entity {
        name: "surrender_limit",
        description: r#"Changes the percentage of victory points the country needs to lose control of to capitulate.

**Example:**
```paradox
surrender_limit = 0.1
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "max_surrender_limit_offset",
        HOI4Entity {
            name: "max_surrender_limit_offset",
            description: r#"Controls the maximum surrender progress of a nation.

**Example:**
```paradox
max_surrender_limit_offset = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("forced_surrender_limit", HOI4Entity {
        name: "forced_surrender_limit",
        description: r#"Changes the percentage of victory points the country needs to lose control of to capitulate, bypassing the minimum or maximum.

**Example:**
```paradox
forced_surrender_limit = 0.1
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "country_resource",
        HOI4Entity {
            name: "country_resource",
            description: r#"Directly modifies the country's resource stockpile.

**Example:**
```paradox
country_resource_oil = 10
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "country_resource_cost",
        HOI4Entity {
            name: "country_resource_cost",
            description: r#"Directly modifies the country's resource stockpile.

**Example:**
```paradox
country_resource_cost_aluminium = 10
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "resource_trade_cost_bonus_per_factory",
        HOI4Entity {
            name: "resource_trade_cost_bonus_per_factory",
            description: r#"Modifies the country's cost to buy resources from others.

**Example:**
```paradox
resource_trade_cost_bonus_per_factory = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "factory_energy_consumption",
        HOI4Entity {
            name: "factory_energy_consumption",
            description: r#"Directly modifies the country's energy usage per factory

**Example:**
```paradox
factory_energy_consumption = 0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "min_export",
        HOI4Entity {
            name: "min_export",
            description: r#"Changes the amount of resources to market.

**Example:**
```paradox
min_export = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "trade_opinion_factor",
        HOI4Entity {
            name: "trade_opinion_factor",
            description: r#"Makes AI more likely to purchase resources from this country.

**Example:**
```paradox
trade_opinion_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("defensive_war_stability_factor", HOI4Entity {
        name: "defensive_war_stability_factor",
        description: r#"Changes the penalty to the stability invoked by participating in a defensive war.

**Example:**
```paradox
defensive_war_stability_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "disabled_ideas",
        HOI4Entity {
            name: "disabled_ideas",
            description: r#"Disables manually changing ideas (including ministers and laws).

**Example:**
```paradox
disabled_ideas = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("cost_factor", HOI4Entity {
        name: "cost_factor",
        description: r#"Changes the cost in political power to add an idea or character within the specified slot.

**Example:**
```paradox
political_advisor_cost_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("category_type_cost_factor", HOI4Entity {
        name: "category_type_cost_factor",
        description: r#"Changes the cost in army experience to add an idea within any of the categories with the specified type.

**Example:**
```paradox
air_spirit_category_type_cost_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("advisor_cost_factor", HOI4Entity {
        name: "advisor_cost_factor",
        description: r#"Changes the cost in political power to add an advisor assigned the specified military ledger.

**Example:**
```paradox
air_advisor_cost_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "unit_leader_as_advisor_cp_cost_factor",
        HOI4Entity {
            name: "unit_leader_as_advisor_cp_cost_factor",
            description: r#"Changes the cost in command power to turn a unit leader into an advisor.

**Example:**
```paradox
unit_leader_as_advisor_cp_cost_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("improve_relations_maintain_cost_factor", HOI4Entity {
        name: "improve_relations_maintain_cost_factor",
        description: r#"Changes the cost in political power to maintain improvement of relations.

**Example:**
```paradox
improve_relations_maintain_cost_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "female_random_country_leader_chance",
        HOI4Entity {
            name: "female_random_country_leader_chance",
            description: r#"Changes the chance for a randomly-generated country leader to be female.

**Example:**
```paradox
female_random_country_leader_chance = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("offensive_war_stability_factor", HOI4Entity {
        name: "offensive_war_stability_factor",
        description: r#"Modifies the stability penalty received from participating in an offensive war.

**Example:**
```paradox
offensive_war_stability_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "party_popularity_stability_factor",
        HOI4Entity {
            name: "party_popularity_stability_factor",
            description: r#"Modifies the stability gained by the popularity of the ruling party.

**Example:**
```paradox
party_popularity_stability_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "political_power_cost",
        HOI4Entity {
            name: "political_power_cost",
            description: r#"Daily cost in political power.

**Example:**
```paradox
political_power_cost = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "political_power_gain",
        HOI4Entity {
            name: "political_power_gain",
            description: r#"Modifies daily gain in political power.

**Example:**
```paradox
political_power_gain = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "political_power_factor",
        HOI4Entity {
            name: "political_power_factor",
            description: r#"Modifies daily gain in political power by a percentage.

**Example:**
```paradox
political_power_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "stability_factor",
        HOI4Entity {
            name: "stability_factor",
            description: r#"Modifies stability of the country.

**Example:**
```paradox
stability_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "stability_weekly",
        HOI4Entity {
            name: "stability_weekly",
            description: r#"Modifies weekly stability gain of the country.

**Example:**
```paradox
stability_weekly = 0.01
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "stability_weekly_factor",
        HOI4Entity {
            name: "stability_weekly_factor",
            description: r#"Modifies weekly stability gain of the country by a percentage.

**Example:**
```paradox
stability_weekly_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "war_stability_factor",
        HOI4Entity {
            name: "war_stability_factor",
            description: r#"Modifies the stability loss caused by being at war.

**Example:**
```paradox
war_stability_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "war_support_factor",
        HOI4Entity {
            name: "war_support_factor",
            description: r#"Modifies war support of the country.

**Example:**
```paradox
war_support_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "war_support_weekly",
        HOI4Entity {
            name: "war_support_weekly",
            description: r#"Modifies weekly war support gain of the country.

**Example:**
```paradox
war_support_weekly = 0.01
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "war_support_weekly_factor",
        HOI4Entity {
            name: "war_support_weekly_factor",
            description: r#"Modifies weekly war support gain of the country by a percentage.

**Example:**
```paradox
war_support_weekly_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("weekly_casualties_war_support", HOI4Entity {
        name: "weekly_casualties_war_support",
        description: r#"Modifies weekly war support gain of the country depending on the casualties suffered by it.

**Example:**
```paradox
weekly_casualties_war_support = 0.006
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("weekly_convoys_war_support", HOI4Entity {
        name: "weekly_convoys_war_support",
        description: r#"Modifies weekly war support gain of the country depending on the amount of its convoys that have been sunk.

**Example:**
```paradox
weekly_convoys_war_support = 0.006
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("weekly_bombing_war_support", HOI4Entity {
        name: "weekly_bombing_war_support",
        description: r#"Modifies weekly war support gain of the country depending on the enemy bombing of its states.

**Example:**
```paradox
weekly_bombing_war_support = 0.006
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "drift_defence_factor",
        HOI4Entity {
            name: "drift_defence_factor",
            description: r#"Ideology drift defense.

**Example:**
```paradox
drift_defence_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "power_balance_daily",
        HOI4Entity {
            name: "power_balance_daily",
            description: r#"Pushes the power balance by a specified amount on each day.

**Example:**
```paradox
power_balance_daily = 0.01
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "power_balance_weekly",
        HOI4Entity {
            name: "power_balance_weekly",
            description: r#"Pushes the power balance by a specified amount on each week.

**Example:**
```paradox
power_balance_weekly = 0.01
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "drift",
        HOI4Entity {
            name: "drift",
            description: r#"Daily gain of the specified ideology.

**Example:**
```paradox
communism_drift = 0.03
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("acceptance", HOI4Entity {
        name: "acceptance",
        description: r#"Likelihood of AI to accept offers from countries of the specified ideology.

**Example:**
```paradox
fascism_acceptance = 50
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("civil_war_involvement_tension", HOI4Entity {
        name: "civil_war_involvement_tension",
        description: r#"Changes the world tension amount necessary to intervene in an ally's civil war.

**Example:**
```paradox
civil_war_involvement_tension = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("enemy_declare_war_tension", HOI4Entity {
        name: "enemy_declare_war_tension",
        description: r#"Changes the world tension required for an enemy to justify a wargoal on us.

**Example:**
```paradox
enemy_declare_war_tension = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "enemy_justify_war_goal_time",
        HOI4Entity {
            name: "enemy_justify_war_goal_time",
            description: r#"Changes the time required for an enemy to justify a wargoal on us.

**Example:**
```paradox
enemy_justify_war_goal_time = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "faction_trade_opinion_factor",
        HOI4Entity {
            name: "faction_trade_opinion_factor",
            description: r#"Changes the opinion gain gained by trade between faction members.

**Example:**
```paradox
faction_trade_opinion_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "generate_wargoal_tension",
        HOI4Entity {
            name: "generate_wargoal_tension",
            description: r#"Changes the necessary tension for us to generate a wargoal.

**Example:**
```paradox
generate_wargoal_tension = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "guarantee_cost",
        HOI4Entity {
            name: "guarantee_cost",
            description: r#"Cost in political power for the country to guarantee an another country.

**Example:**
```paradox
guarantee_cost = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "guarantee_tension",
        HOI4Entity {
            name: "guarantee_tension",
            description: r#"Necessary world tension for the country to guarantee an another country.

**Example:**
```paradox
guarantee_tension = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "join_faction_tension",
        HOI4Entity {
            name: "join_faction_tension",
            description: r#"Necessary world tension for the country to join a faction.

**Example:**
```paradox
join_faction_tension = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "justify_war_goal_time",
        HOI4Entity {
            name: "justify_war_goal_time",
            description: r#"The amount of time necessary to justify a wargoal.

**Example:**
```paradox
justify_war_goal_time = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("justify_war_goal_when_in_major_war_time", HOI4Entity {
        name: "justify_war_goal_when_in_major_war_time",
        description: r#"The amount of time necessary to justify a wargoal when in a war with a major country.

**Example:**
```paradox
justify_war_goal_when_in_major_war_time = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "lend_lease_tension",
        HOI4Entity {
            name: "lend_lease_tension",
            description: r#"Necessary world tension for the country to lend-lease.

**Example:**
```paradox
lend_lease_tension = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "lend_lease_tension_with_overlord",
        HOI4Entity {
            name: "lend_lease_tension_with_overlord",
            description: r#"Necessary world tension for the country to lend-lease to its overlord.

**Example:**
```paradox
lend_lease_tension_with_overlord = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "opinion_gain_monthly",
        HOI4Entity {
            name: "opinion_gain_monthly",
            description: r#"Changes opinion gain from the 'Improve relations' diplomatic action.

**Example:**
```paradox
opinion_gain_monthly = 5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("opinion_gain_monthly_factor", HOI4Entity {
        name: "opinion_gain_monthly_factor",
        description: r#"Changes opinion gain from the 'Improve relations' diplomatic action by a percentage.

**Example:**
```paradox
opinion_gain_monthly_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("opinion_gain_monthly_same_ideology", HOI4Entity {
        name: "opinion_gain_monthly_same_ideology",
        description: r#"Changes opinion gain from the 'Improve relations' diplomatic action for countries of the same ideology.

**Example:**
```paradox
opinion_gain_monthly_same_ideology = 5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("opinion_gain_monthly_same_ideology_factor", HOI4Entity {
        name: "opinion_gain_monthly_same_ideology_factor",
        description: r#"Changes opinion gain from the 'Improve relations' diplomatic action for countries of the same ideology by a percentage.

**Example:**
```paradox
opinion_gain_monthly_same_ideology_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "request_lease_tension",
        HOI4Entity {
            name: "request_lease_tension",
            description: r#"Necessary world tension for the country to request lend-lease.

**Example:**
```paradox
request_lease_tension = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "annex_cost_factor",
        HOI4Entity {
            name: "annex_cost_factor",
            description: r#"Modifies the cost in victory points to annex states in peace deals.

**Example:**
```paradox
annex_cost_factor = 0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("puppet_cost_factor", HOI4Entity {
        name: "puppet_cost_factor",
        description: r#"Modifies the cost in victory points per state to puppet countries in peace deals.

**Example:**
```paradox
puppet_cost_factor = 0.1
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "send_volunteer_divisions_required",
        HOI4Entity {
            name: "send_volunteer_divisions_required",
            description: r#"Changes the number of divisions needed to send volunteers.

**Example:**
```paradox
send_volunteer_divisions_required = -0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("send_volunteer_factor", HOI4Entity {
        name: "send_volunteer_factor",
        description: r#"Changes the number of divisions the country can send as volunteers by a percentage.

**Example:**
```paradox
send_volunteer_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "send_volunteer_size",
        HOI4Entity {
            name: "send_volunteer_size",
            description: r#"Changes the number of divisions the country can send as volunteers.

**Example:**
```paradox
send_volunteer_size = 5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "send_volunteers_tension",
        HOI4Entity {
            name: "send_volunteers_tension",
            description: r#"Changes the world tension necessary for the country to send volunteers.

**Example:**
```paradox
send_volunteers_tension = -0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_volunteer_cap",
        HOI4Entity {
            name: "air_volunteer_cap",
            description: r#"Changes the amount of airforce you can send as volunteers.

**Example:**
```paradox
air_volunteer_cap = 100
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("embargo_threshold_factor", HOI4Entity {
        name: "embargo_threshold_factor",
        description: r#"Changes the necessary world tension level in order to be able to embargo a country.

**Example:**
```paradox
embargo_threshold_factor = 0.2
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "embargo_cost_factor",
        HOI4Entity {
            name: "embargo_cost_factor",
            description: r#"Changes the cost in political power to send an embargo.

**Example:**
```paradox
embargo_cost_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "autonomy_gain",
        HOI4Entity {
            name: "autonomy_gain",
            description: r#"Daily gain of autonomy.

**Example:**
```paradox
autonomy_gain = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "autonomy_gain_global_factor",
        HOI4Entity {
            name: "autonomy_gain_global_factor",
            description: r#"Modifies all gain of autonomy by a subject.

**Example:**
```paradox
autonomy_gain_global_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "subjects_autonomy_gain",
        HOI4Entity {
            name: "subjects_autonomy_gain",
            description: r#"Daily gain of autonomy in our subjects.

**Example:**
```paradox
subjects_autonomy_gain = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "autonomy_gain_ll_to_overlord",
        HOI4Entity {
            name: "autonomy_gain_ll_to_overlord",
            description: r#"Modifies gain of autonomy from lend-leasing to the overlord.

**Example:**
```paradox
autonomy_gain_ll_to_overlord = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("autonomy_gain_ll_to_overlord_factor", HOI4Entity {
        name: "autonomy_gain_ll_to_overlord_factor",
        description: r#"Modifies gain of autonomy from lend-leasing to the overlord by a percentage.

**Example:**
```paradox
autonomy_gain_ll_to_overlord_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "autonomy_gain_ll_to_subject",
        HOI4Entity {
            name: "autonomy_gain_ll_to_subject",
            description: r#"Modifies loss of autonomy from lend-leasing to the subject.

**Example:**
```paradox
autonomy_gain_ll_to_subject = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("autonomy_gain_ll_to_subject_factor", HOI4Entity {
        name: "autonomy_gain_ll_to_subject_factor",
        description: r#"Modifies loss of autonomy from lend-leasing to the subject by a percentage.

**Example:**
```paradox
autonomy_gain_ll_to_subject_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "autonomy_gain_trade",
        HOI4Entity {
            name: "autonomy_gain_trade",
            description: r#"Modifies gain of autonomy from the overlord trading with the subject.

**Example:**
```paradox
autonomy_gain_trade = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("autonomy_gain_trade_factor", HOI4Entity {
        name: "autonomy_gain_trade_factor",
        description: r#"Modifies gain of autonomy from the overlord trading with the subject by a percentage.

**Example:**
```paradox
autonomy_gain_trade_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "autonomy_gain_warscore",
        HOI4Entity {
            name: "autonomy_gain_warscore",
            description: r#"Modifies gain of autonomy from the subject gaining warscore.

**Example:**
```paradox
autonomy_gain_warscore = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("autonomy_gain_warscore_factor", HOI4Entity {
        name: "autonomy_gain_warscore_factor",
        description: r#"Modifies gain of autonomy from the subject gaining warscore by a percentage.

**Example:**
```paradox
autonomy_gain_warscore_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "autonomy_manpower_share",
        HOI4Entity {
            name: "autonomy_manpower_share",
            description: r#"Modifies the amount of manpower the overlord can use from the subject.

**Example:**
```paradox
autonomy_manpower_share = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "can_master_build_for_us",
        HOI4Entity {
            name: "can_master_build_for_us",
            description: r#"Makes the overlord be able to build in the subject.

**Example:**
```paradox
can_master_build_for_us = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("cic_to_overlord_factor", HOI4Entity {
        name: "cic_to_overlord_factor",
        description: r#"Modifies the amount of the subject's civilian industry that goes to the overlord.

**Example:**
```paradox
cic_to_overlord_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("mic_to_overlord_factor", HOI4Entity {
        name: "mic_to_overlord_factor",
        description: r#"Modifies the amount of the subject's military industry that goes to the overlord.

**Example:**
```paradox
mic_to_overlord_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("extra_trade_to_overlord_factor", HOI4Entity {
        name: "extra_trade_to_overlord_factor",
        description: r#"Modifies the amount of the subject's resources that the overlord can receive via trade.

**Example:**
```paradox
extra_trade_to_overlord_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "license_subject_master_purchase_cost",
        HOI4Entity {
            name: "license_subject_master_purchase_cost",
            description: r#"Modifies the cost of licensed production from the overlord.

**Example:**
```paradox
license_subject_master_purchase_cost = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("master_build_autonomy_factor", HOI4Entity {
        name: "master_build_autonomy_factor",
        description: r#"Modifies loss of autonomy from the overlord building in subject's states by a percentage.

**Example:**
```paradox
master_build_autonomy_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "master_ideology_drift",
        HOI4Entity {
            name: "master_ideology_drift",
            description: r#"Changes daily gain of the overlord's ideology in the country.

**Example:**
```paradox
master_ideology_drift = 0.03
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("overlord_trade_cost_factor", HOI4Entity {
        name: "overlord_trade_cost_factor",
        description: r#"Modifies the cost of trade between the overlord and the subject in civilian factories.

**Example:**
```paradox
overlord_trade_cost_factor = -0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "dockyard_donations",
        HOI4Entity {
            name: "dockyard_donations",
            description: r#"Amount of dockyards donated.

**Example:**
```paradox
dockyard_donations = 2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "industrial_factory_donations",
        HOI4Entity {
            name: "industrial_factory_donations",
            description: r#"Amount of civilian factories donated.

**Example:**
```paradox
industrial_factory_donations = 2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "military_factory_donations",
        HOI4Entity {
            name: "military_factory_donations",
            description: r#"Amount of military factories donated.

**Example:**
```paradox
military_factory_donations = 2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "exile_manpower_factor",
        HOI4Entity {
            name: "exile_manpower_factor",
            description: r#"Amount of manpower given to the host country.

**Example:**
```paradox
exile_manpower_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "exiled_government_weekly_manpower",
        HOI4Entity {
            name: "exiled_government_weekly_manpower",
            description: r#"Amount of weekly manpower given to the host country.

**Example:**
```paradox
exiled_government_weekly_manpower = 100
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "legitimacy_daily",
        HOI4Entity {
            name: "legitimacy_daily",
            description: r#"Changes the amount of legitimacy gained daily.

**Example:**
```paradox
legitimacy_daily = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "legitimacy_gain_factor",
        HOI4Entity {
            name: "legitimacy_gain_factor",
            description: r#"Changes the amount of legitimacy gained daily by a percentage.

**Example:**
```paradox
legitimacy_gain_factor = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "equipment_capture",
        HOI4Entity {
            name: "equipment_capture",
            description: r#"Changes the combat equipment capture ratio.

**Example:**
```paradox
equipment_capture = 0.2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "equipment_capture_factor",
        HOI4Entity {
            name: "equipment_capture_factor",
            description: r#"Modifies the combat equipment capture ratio.

**Example:**
```paradox
equipment_capture_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "equipment_conversion_speed",
        HOI4Entity {
            name: "equipment_conversion_speed",
            description: r#"Changes the speed at which equipment is converted.

**Example:**
```paradox
equipment_conversion_speed = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "equipment_upgrade_xp_cost",
        HOI4Entity {
            name: "equipment_upgrade_xp_cost",
            description: r#"Changes the experience cost to upgrade military equipment.

**Example:**
```paradox
equipment_upgrade_xp_cost = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "license_purchase_cost",
        HOI4Entity {
            name: "license_purchase_cost",
            description: r#"Changes the cost of licensed equipment by a percentage.

**Example:**
```paradox
license_purchase_cost = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "license_purchase_cost_factor",
        HOI4Entity {
            name: "license_purchase_cost_factor",
            description: r#"Changes the cost of licensed equipment by a percentage.

**Example:**
```paradox
license_purchase_cost_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("license_tech_difference_speed", HOI4Entity {
        name: "license_tech_difference_speed",
        description: r#"Changes the production penalty of licensed equipment by tech difference by a percentage.

**Example:**
```paradox
license_tech_difference_speed = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "license_production_speed",
        HOI4Entity {
            name: "license_production_speed",
            description: r#"Changes the production speed of licensed equipment by a percentage.

**Example:**
```paradox
license_production_speed = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "license__production_speed_factor",
        HOI4Entity {
            name: "license__production_speed_factor",
            description: r#"Changes the production speed of licensed equipment by a percentage.

**Example:**
```paradox
license_infantry_eq_production_speed_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "production_cost_max",
        HOI4Entity {
            name: "production_cost_max",
            description: r#"Modifies the maximum cost of the ship type.

**Example:**
```paradox
production_cost_max_ship_hull_light = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "production_factory_efficiency_gain_factor",
        HOI4Entity {
            name: "production_factory_efficiency_gain_factor",
            description: r#"Production efficiency growth.

**Example:**
```paradox
production_factory_efficiency_gain_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "production_factory_max_efficiency_factor",
        HOI4Entity {
            name: "production_factory_max_efficiency_factor",
            description: r#"Production efficiency cap.

**Example:**
```paradox
production_factory_max_efficiency_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "production_factory_start_efficiency_factor",
        HOI4Entity {
            name: "production_factory_start_efficiency_factor",
            description: r#"Production efficiency base.

**Example:**
```paradox
production_factory_start_efficiency_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "line_change_production_efficiency_factor",
        HOI4Entity {
            name: "line_change_production_efficiency_factor",
            description: r#"Production efficiency retention.

**Example:**
```paradox
line_change_production_efficiency_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "production_lack_of_resource_penalty_factor",
        HOI4Entity {
            name: "production_lack_of_resource_penalty_factor",
            description: r#"Lack of resources penalty.

**Example:**
```paradox
production_lack_of_resource_penalty_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "floating_harbor_duration",
        HOI4Entity {
            name: "floating_harbor_duration",
            description: r#"Modifies the duration of floating harbours.

**Example:**
```paradox
floating_harbor_duration = 2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "floating_harbor_range",
        HOI4Entity {
            name: "floating_harbor_range",
            description: r#"Modifies the range of floating harbours.

**Example:**
```paradox
floating_harbor_range = 2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "floating_harbor_supply",
        HOI4Entity {
            name: "floating_harbor_supply",
            description: r#"Modifies the supply of floating harbours.

**Example:**
```paradox
floating_harbor_supply = 2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "railway_gun_bombardment_factor",
        HOI4Entity {
            name: "railway_gun_bombardment_factor",
            description: r#"Modifies the bombardment of railway guns.

**Example:**
```paradox
railway_gun_bombardment_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "base_fuel_gain",
        HOI4Entity {
            name: "base_fuel_gain",
            description: r#"Changes base daily gain of fuel.

**Example:**
```paradox
base_fuel_gain = 100
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "base_fuel_gain_factor",
        HOI4Entity {
            name: "base_fuel_gain_factor",
            description: r#"Changes base daily gain of fuel by a percentage.

**Example:**
```paradox
base_fuel_gain_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "fuel_cost",
        HOI4Entity {
            name: "fuel_cost",
            description: r#"Changes hourly cost of fuel.

**Example:**
```paradox
fuel_cost = 100
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "fuel_gain",
        HOI4Entity {
            name: "fuel_gain",
            description: r#"Changes daily gain of fuel from our controlled oil.

**Example:**
```paradox
fuel_gain = 100
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "fuel_gain_factor",
        HOI4Entity {
            name: "fuel_gain_factor",
            description: r#"Changes daily gain of fuel from our controlled oil by a percentage.

**Example:**
```paradox
fuel_gain_factor = 100
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "fuel_gain_from_states",
        HOI4Entity {
            name: "fuel_gain_from_states",
            description: r#"Changes daily gain of fuel.

**Example:**
```paradox
fuel_gain_from_states = 100
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "fuel_gain_factor_from_states",
        HOI4Entity {
            name: "fuel_gain_factor_from_states",
            description: r#"Changes daily gain of fuel by a percentage.

**Example:**
```paradox
fuel_gain_factor_from_states = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "max_fuel",
        HOI4Entity {
            name: "max_fuel",
            description: r#"Changes maximum amount of fuel you can have.

**Example:**
```paradox
max_fuel = 100
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "max_fuel_factor",
        HOI4Entity {
            name: "max_fuel_factor",
            description: r#"Changes maximum amount of fuel you can have by a percentage.

**Example:**
```paradox
max_fuel_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_fuel_capacity_factor",
        HOI4Entity {
            name: "army_fuel_capacity_factor",
            description: r#"Modifies how much fuel a single unit can store before running out.

**Example:**
```paradox
army_fuel_capacity_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_fuel_consumption_factor",
        HOI4Entity {
            name: "army_fuel_consumption_factor",
            description: r#"Modifies the rate at which the army consumes fuel.

**Example:**
```paradox
army_fuel_consumption_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_fuel_consumption_factor",
        HOI4Entity {
            name: "air_fuel_consumption_factor",
            description: r#"Modifies the rate at which the airforce consumes fuel.

**Example:**
```paradox
air_fuel_consumption_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_fuel_consumption_factor",
        HOI4Entity {
            name: "navy_fuel_consumption_factor",
            description: r#"Modifies the rate at which the navy consumes fuel.

**Example:**
```paradox
navy_fuel_consumption_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "supply_factor",
        HOI4Entity {
            name: "supply_factor",
            description: r#"Modifies the total amount of supply the military has.

**Example:**
```paradox
supply_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("supply_combat_penalties_on_core_factor", HOI4Entity {
        name: "supply_combat_penalties_on_core_factor",
        description: r#"Modifies the penalty given by low supply when the army is on a core state.

**Example:**
```paradox
supply_combat_penalties_on_core_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "supply_consumption_factor",
        HOI4Entity {
            name: "supply_consumption_factor",
            description: r#"Modifies the rate at which army consumes supply.

**Example:**
```paradox
supply_consumption_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "no_supply_grace",
        HOI4Entity {
            name: "no_supply_grace",
            description: r#"Modifies the grace period for units without supply.

**Example:**
```paradox
no_supply_grace = 120
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "out_of_supply_factor",
        HOI4Entity {
            name: "out_of_supply_factor",
            description: r#"Reduces the penalty that units take when they run out of supplies.

**Example:**
```paradox
out_of_supply_factor = 0.2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "attrition",
        HOI4Entity {
            name: "attrition",
            description: r#"Modifies the army's attrition.

**Example:**
```paradox
attrition = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "unit_upkeep_attrition_factor",
        HOI4Entity {
            name: "unit_upkeep_attrition_factor",
            description: r#"Modifies the unit upkeep.

**Example:**
```paradox
unit_upkeep_attrition_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "naval_attrition",
        HOI4Entity {
            name: "naval_attrition",
            description: r#"Modifies attrition suffered by naval units.

**Example:**
```paradox
naval_attrition = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "heat_attrition",
        HOI4Entity {
            name: "heat_attrition",
            description: r#"Changes the attrition due to heat.

**Example:**
```paradox
heat_attrition = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "heat_attrition_factor",
        HOI4Entity {
            name: "heat_attrition_factor",
            description: r#"Changes the attrition due to heat by a percentage.

**Example:**
```paradox
heat_attrition_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "winter_attrition",
        HOI4Entity {
            name: "winter_attrition",
            description: r#"Changes the attrition due to winter.

**Example:**
```paradox
winter_attrition = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "winter_attrition_factor",
        HOI4Entity {
            name: "winter_attrition_factor",
            description: r#"Changes the attrition due to winter by a percentage.

**Example:**
```paradox
winter_attrition_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "extra_marine_supply_grace",
        HOI4Entity {
            name: "extra_marine_supply_grace",
            description: r#"Changes the supply grace given to marines.

**Example:**
```paradox
extra_marine_supply_grace = 96
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "extra_paratrooper_supply_grace",
        HOI4Entity {
            name: "extra_paratrooper_supply_grace",
            description: r#"Changes the supply grace given to paratroopers.

**Example:**
```paradox
extra_paratrooper_supply_grace = 96
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "special_forces_no_supply_grace",
        HOI4Entity {
            name: "special_forces_no_supply_grace",
            description: r#"Changes the supply grace period for special forces.

**Example:**
```paradox
special_forces_no_supply_grace = 120
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "special_forces_out_of_supply_factor",
        HOI4Entity {
            name: "special_forces_out_of_supply_factor",
            description: r#"Changes the penalty for special forces out of supply.

**Example:**
```paradox
special_forces_out_of_supply_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "truck_attrition",
        HOI4Entity {
            name: "truck_attrition",
            description: r#"Changes the attrition supply trucks suffer from.

**Example:**
```paradox
truck_attrition = 3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "truck_attrition_factor",
        HOI4Entity {
            name: "truck_attrition_factor",
            description: r#"Modifies the attrition supply trucks suffer from.

**Example:**
```paradox
truck_attrition_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "production_speed_buildings_factor",
        HOI4Entity {
            name: "production_speed_buildings_factor",
            description: r#"Changes the construction speed of all buildings.

**Example:**
```paradox
production_speed_buildings_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "production_speed__factor",
        HOI4Entity {
            name: "production_speed__factor",
            description: r#"Changes the construction speed of a specific building.

**Example:**
```paradox
production_speed_industrial_complex_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "production_cost__factor",
        HOI4Entity {
            name: "production_cost__factor",
            description: r#"Changes the base cost of a specific building.

**Example:**
```paradox
production_cost_industrial_complex_factor = -0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "civilian_factory_use",
        HOI4Entity {
            name: "civilian_factory_use",
            description: r#"Uses the specified amount of civilian factory as a special project.

**Example:**
```paradox
civilian_factory_use = 3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "consumer_goods_factor",
        HOI4Entity {
            name: "consumer_goods_factor",
            description: r#"Modifies the percentage of factories used for consumer goods.

**Example:**
```paradox
consumer_goods_factor = 0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "consumer_goods_expected_value",
        HOI4Entity {
            name: "consumer_goods_expected_value",
            description: r#"Sets the baseline percentage of expected consumer goods.

**Example:**
```paradox
consumer_goods_expected_value = 0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "conversion_cost_civ_to_mil_factor",
        HOI4Entity {
            name: "conversion_cost_civ_to_mil_factor",
            description: r#"Changes the cost to convert civilian factories to military factories.

**Example:**
```paradox
conversion_cost_civ_to_mil_factor = 0.4
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "conversion_cost_mil_to_civ_factor",
        HOI4Entity {
            name: "conversion_cost_mil_to_civ_factor",
            description: r#"Changes the cost to convert military factories to civilian factories.

**Example:**
```paradox
conversion_cost_mil_to_civ_factor = 0.4
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "global_building_slots",
        HOI4Entity {
            name: "global_building_slots",
            description: r#"Changes amount of building slots in our every state.

**Example:**
```paradox
global_building_slots = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "global_building_slots_factor",
        HOI4Entity {
            name: "global_building_slots_factor",
            description: r#"Changes amount of building slots in our every state by a percentage.

**Example:**
```paradox
global_building_slots_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "industrial_capacity_dockyard",
        HOI4Entity {
            name: "industrial_capacity_dockyard",
            description: r#"Dockyard output.

**Example:**
```paradox
industrial_capacity_dockyard = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "industrial_capacity_factory",
        HOI4Entity {
            name: "industrial_capacity_factory",
            description: r#"Military factory output.

**Example:**
```paradox
industrial_capacity_factory = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "industry_air_damage_factor",
        HOI4Entity {
            name: "industry_air_damage_factor",
            description: r#"Amount of damage our factories receive from air bombings.

**Example:**
```paradox
industry_air_damage_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("industry_free_repair_factor", HOI4Entity {
        name: "industry_free_repair_factor",
        description: r#"Changes the speed at which buildings repair themselves without factories assigned.

**Example:**
```paradox
industry_free_repair_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "industry_repair_factor",
        HOI4Entity {
            name: "industry_repair_factor",
            description: r#"Changes the speed at which buildings are repaired.

**Example:**
```paradox
industry_repair_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "production_oil_factor",
        HOI4Entity {
            name: "production_oil_factor",
            description: r#"Synthetic oil gain.

**Example:**
```paradox
production_oil_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "repair_speed__factor",
        HOI4Entity {
            name: "repair_speed__factor",
            description: r#"Changes the repair speed of a specific building.

**Example:**
```paradox
repair_speed_arms_factory_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "supply_node_range",
        HOI4Entity {
            name: "supply_node_range",
            description: r#"Increases the effective range of supply nodes.

**Example:**
```paradox
supply_node_range = 0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "static_anti_air_damage_factor",
        HOI4Entity {
            name: "static_anti_air_damage_factor",
            description: r#"Modifies the damage done to planes by the anti-air buildings.

**Example:**
```paradox
static_anti_air_damage_factor = 0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "static_anti_air_hit_chance_factor",
        HOI4Entity {
            name: "static_anti_air_hit_chance_factor",
            description: r#"Modifies the chance for the anti-air buildings to hit enemy planes.

**Example:**
```paradox
static_anti_air_hit_chance_factor = 0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("tech_air_damage_factor", HOI4Entity {
        name: "tech_air_damage_factor",
        description: r#"Modifies the damage done to the country's planes by enemy anti-air buildings.

**Example:**
```paradox
tech_air_damage_factor = 0.1
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "cic_construction_boost",
        HOI4Entity {
            name: "cic_construction_boost",
            description: r#"Modifies the base construction speed from civilian factories.

**Example:**
```paradox
cic_construction_boost = 0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("cic_construction_boost_factor", HOI4Entity {
        name: "cic_construction_boost_factor",
        description: r#"Modifies the modifier to the base construction speed from civilian factories.

**Example:**
```paradox
cic_construction_boost_factor = 0.1
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "land_bunker_effectiveness_factor",
        HOI4Entity {
            name: "land_bunker_effectiveness_factor",
            description: r#"Modifies the effectiveness of land forts in defence.

**Example:**
```paradox
land_bunker_effectiveness_factor = 0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "coastal_bunker_effectiveness_factor",
        HOI4Entity {
            name: "coastal_bunker_effectiveness_factor",
            description: r#"Modifies the effectiveness of coastal forts in defence.

**Example:**
```paradox
coastal_bunker_effectiveness_factor = 0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "compliance_growth_on_our_occupied_states",
        HOI4Entity {
            name: "compliance_growth_on_our_occupied_states",
            description: r#"Changes the compliance growth speed on the country's controlled states.

**Example:**
```paradox
compliance_growth_on_our_occupied_states = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "no_compliance_gain",
        HOI4Entity {
            name: "no_compliance_gain",
            description: r#"Disables the compliance gain on our controlled states.

**Example:**
```paradox
no_compliance_gain = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "required_garrison_factor",
        HOI4Entity {
            name: "required_garrison_factor",
            description: r#"Changes the required garrison in our occupied states.

**Example:**
```paradox
required_garrison_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("resistance_activity", HOI4Entity {
        name: "resistance_activity",
        description: r#"Changes the chance for resistance activity to occur on our occupied states.

**Example:**
```paradox
resistance_activity = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "resistance_damage_to_garrison_on_our_occupied_states",
        HOI4Entity {
            name: "resistance_damage_to_garrison_on_our_occupied_states",
            description: r#"Changes the resistance damage to the garrison in our occupied states.

**Example:**
```paradox
resistance_damage_to_garrison_on_our_occupied_states = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "resistance_decay_on_our_occupied_states",
        HOI4Entity {
            name: "resistance_decay_on_our_occupied_states",
            description: r#"Changes the resistance decay in our occupied states.

**Example:**
```paradox
resistance_decay_on_our_occupied_states = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "resistance_growth_on_our_occupied_states",
        HOI4Entity {
            name: "resistance_growth_on_our_occupied_states",
            description: r#"Changes the resistance growth speed in our occupied states.

**Example:**
```paradox
resistance_growth_on_our_occupied_states = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "resistance_target_on_our_occupied_states",
        HOI4Entity {
            name: "resistance_target_on_our_occupied_states",
            description: r#"Changes the resistance target in our occupied states.

**Example:**
```paradox
resistance_target_on_our_occupied_states = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "resistance_target",
        HOI4Entity {
            name: "resistance_target",
            description: r#"Changes the resistance target in foreign states occupied by us

**Example:**
```paradox
resistance_target = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "agency_upgrade_time",
        HOI4Entity {
            name: "agency_upgrade_time",
            description: r#"Changes the time it takes to upgrade the agency

**Example:**
```paradox
agency_upgrade_time = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "decryption",
        HOI4Entity {
            name: "decryption",
            description: r#"Changes the decription capability of the country.

**Example:**
```paradox
decryption = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "decryption_factor",
        HOI4Entity {
            name: "decryption_factor",
            description: r#"Changes the decription capability of the country by a percentage.

**Example:**
```paradox
decryption_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "encryption",
        HOI4Entity {
            name: "encryption",
            description: r#"Changes the encryption capability of the country.

**Example:**
```paradox
encryption = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "encryption_factor",
        HOI4Entity {
            name: "encryption_factor",
            description: r#"Changes the encryption capability of the country by a percentage.

**Example:**
```paradox
encryption_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "intel_decryption_bonus",
        HOI4Entity {
            name: "intel_decryption_bonus",
            description: r#"Adds a cipher bonus to the specified intel.

**Example:**
```paradox
civilian_intel_decryption_bonus = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "intel_factor",
        HOI4Entity {
            name: "intel_factor",
            description: r#"Modifies the intelligence you receive of the specified type.

**Example:**
```paradox
navy_intel_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("intel_to_others", HOI4Entity {
        name: "intel_to_others",
        description: r#"Changes the amount of intel other countries will receive of the specified type.

**Example:**
```paradox
civilian_intel_to_others = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "female_random_operative_chance",
        HOI4Entity {
            name: "female_random_operative_chance",
            description: r#"Changes the chance for a randomly-generated operative to be female.

**Example:**
```paradox
female_random_operative_chance = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "foreign_subversive_activites",
        HOI4Entity {
            name: "foreign_subversive_activites",
            description: r#"Changes efficiency of foreign subversive activities.

**Example:**
```paradox
foreign_subversive_activites = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "intel_network_gain",
        HOI4Entity {
            name: "intel_network_gain",
            description: r#"Changes gain of intel network strength.

**Example:**
```paradox
intel_network_gain = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "intel_network_gain_factor",
        HOI4Entity {
            name: "intel_network_gain_factor",
            description: r#"Changes gain of intel network strength by a percentage.

**Example:**
```paradox
intel_network_gain_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "subversive_activites_upkeep",
        HOI4Entity {
            name: "subversive_activites_upkeep",
            description: r#"Changes the cost of subversive activities.

**Example:**
```paradox
subversive_activites_upkeep = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "operation_cost",
        HOI4Entity {
            name: "operation_cost",
            description: r#"Changes the cost of operations.

**Example:**
```paradox
operation_cost = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "operation_outcome",
        HOI4Entity {
            name: "operation_outcome",
            description: r#"Changes the efficiency of operations.

**Example:**
```paradox
operation_outcome = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "operation_risk",
        HOI4Entity {
            name: "operation_risk",
            description: r#"Changes the risk of operations.

**Example:**
```paradox
operation_risk = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "cost",
        HOI4Entity {
            name: "cost",
            description: r#"Changes the cost of the specified operation.

**Example:**
```paradox
operation_infiltrate_cost = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "outcome",
        HOI4Entity {
            name: "outcome",
            description: r#"Changes the efficiency of the specified operation.

**Example:**
```paradox
operation_coup_government_outcome = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "risk",
        HOI4Entity {
            name: "risk",
            description: r#"Changes the risk of the specified operation.

**Example:**
```paradox
operation_make_resistance_contacts_risk = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "factor",
        HOI4Entity {
            name: "factor",
            description: r#"Modifies the effect of the specified mission.

**Example:**
```paradox
boost_ideology_mission_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("commando_trait_chance_factor", HOI4Entity {
        name: "commando_trait_chance_factor",
        description: r#"Modifies the chance for an operative to get the commando trait when hired.

**Example:**
```paradox
commando_trait_chance_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "crypto_department_enabled",
        HOI4Entity {
            name: "crypto_department_enabled",
            description: r#"Enables the crypto department.

**Example:**
```paradox
crypto_department_enabled = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "crypto_strength",
        HOI4Entity {
            name: "crypto_strength",
            description: r#"Modifies the cryptology level.

**Example:**
```paradox
crypto_strength = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "decryption_power",
        HOI4Entity {
            name: "decryption_power",
            description: r#"Modifies the decryption power.

**Example:**
```paradox
decryption_power = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "decryption_power_factor",
        HOI4Entity {
            name: "decryption_power_factor",
            description: r#"Modifies the decryption power by a percentage.

**Example:**
```paradox
decryption_power_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("defense_impact_on_blueprint_stealing", HOI4Entity {
        name: "defense_impact_on_blueprint_stealing",
        description: r#"Modifies the impact of enemy defense on the blueprint stealing operation.

**Example:**
```paradox
defense_impact_on_blueprint_stealing = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "intel_from_combat_factor",
        HOI4Entity {
            name: "intel_from_combat_factor",
            description: r#"Modifies the intelligence gained from combat.

**Example:**
```paradox
intel_from_combat_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "intel_from_operatives_factor",
        HOI4Entity {
            name: "intel_from_operatives_factor",
            description: r#"Modifies the intelligence gained from operatives and infiltrated assets.

**Example:**
```paradox
intel_from_operatives_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "intelligence_agency_defense",
        HOI4Entity {
            name: "intelligence_agency_defense",
            description: r#"Modifies the counter intelligence.

**Example:**
```paradox
intelligence_agency_defense = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "root_out_resistance_effectiveness_factor",
        HOI4Entity {
            name: "root_out_resistance_effectiveness_factor",
            description: r#"Modifies the effectiveness of rooting out resistance.

**Example:**
```paradox
root_out_resistance_effectiveness_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "own_operative_capture_chance_factor",
        HOI4Entity {
            name: "own_operative_capture_chance_factor",
            description: r#"Changes the chance for our operatives to be captured.

**Example:**
```paradox
own_operative_capture_chance_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "own_operative_detection_chance",
        HOI4Entity {
            name: "own_operative_detection_chance",
            description: r#"Changes the chance for our operatives to be detected.

**Example:**
```paradox
own_operative_detection_chance = 10
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "own_operative_detection_chance_factor",
        HOI4Entity {
            name: "own_operative_detection_chance_factor",
            description: r#"Changes the chance for our operatives to be detected by a percentage.

**Example:**
```paradox
own_operative_detection_chance_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("own_operative_forced_into_hiding_time_factor", HOI4Entity {
        name: "own_operative_forced_into_hiding_time_factor",
        description: r#"Changes the chance for our operatives to be forced into hiding by a percentage.

**Example:**
```paradox
own_operative_forced_into_hiding_time_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "own_operative_harmed_time_factor",
        HOI4Entity {
            name: "own_operative_harmed_time_factor",
            description: r#"Changes the chance for our operatives to be harmed by a percentage.

**Example:**
```paradox
own_operative_harmed_time_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "own_operative_intel_extraction_rate",
        HOI4Entity {
            name: "own_operative_intel_extraction_rate",
            description: r#"Changes the rate at which our operatives extract enemy intel.

**Example:**
```paradox
own_operative_intel_extraction_rate = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "enemy_operative_capture_chance_factor",
        HOI4Entity {
            name: "enemy_operative_capture_chance_factor",
            description: r#"Changes the chance for an enemy operative to be captured.

**Example:**
```paradox
enemy_operative_capture_chance_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "enemy_operative_detection_chance",
        HOI4Entity {
            name: "enemy_operative_detection_chance",
            description: r#"Changes the chance for an enemy operative to be detected.

**Example:**
```paradox
enemy_operative_detection_chance = 10
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("enemy_operative_detection_chance_factor", HOI4Entity {
        name: "enemy_operative_detection_chance_factor",
        description: r#"Changes the chance for an enemy operative to be detected by a percentage.

**Example:**
```paradox
enemy_operative_detection_chance_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("enemy_operative_forced_into_hiding_time_factor", HOI4Entity {
        name: "enemy_operative_forced_into_hiding_time_factor",
        description: r#"Changes the chance for an enemy operative to be forced into hiding by a percentage.

**Example:**
```paradox
enemy_operative_forced_into_hiding_time_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "enemy_operative_harmed_time_factor",
        HOI4Entity {
            name: "enemy_operative_harmed_time_factor",
            description: r#"Changes the chance for an enemy operative to be harmed by a percentage.

**Example:**
```paradox
enemy_operative_harmed_time_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "enemy_operative_intel_extraction_rate",
        HOI4Entity {
            name: "enemy_operative_intel_extraction_rate",
            description: r#"Changes the rate at which the enemy operatives extract our intel.

**Example:**
```paradox
enemy_operative_intel_extraction_rate = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "enemy_spy_negative_status_factor",
        HOI4Entity {
            name: "enemy_spy_negative_status_factor",
            description: r#"Changes the chance an enemy spy can receive a negative status.

**Example:**
```paradox
enemy_spy_negative_status_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "enemy_operative_recruitment_chance",
        HOI4Entity {
            name: "enemy_operative_recruitment_chance",
            description: r#"Modifies the chance to recruit an enemy operative.

**Example:**
```paradox
enemy_operative_recruitment_chance = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "new_operative_slot_bonus",
        HOI4Entity {
            name: "new_operative_slot_bonus",
            description: r#"Modifies the operative recruitment choices.

**Example:**
```paradox
new_operative_slot_bonus = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "occupied_operative_recruitment_chance",
        HOI4Entity {
            name: "occupied_operative_recruitment_chance",
            description: r#"Modifies the chance to get an operative from occupied territory.

**Example:**
```paradox
occupied_operative_recruitment_chance = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("operative_death_on_capture_chance", HOI4Entity {
        name: "operative_death_on_capture_chance",
        description: r#"Modifies the chance for the country's operative to die on being captured.

**Example:**
```paradox
operative_death_on_capture_chance = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "operative_slot",
        HOI4Entity {
            name: "operative_slot",
            description: r#"Modifies the amount of operative slots.

**Example:**
```paradox
operative_slot = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "ai_badass_factor",
        HOI4Entity {
            name: "ai_badass_factor",
            description: r#"AI's threat perception.

**Example:**
```paradox
ai_badass_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "ai_call_ally_desire_factor",
        HOI4Entity {
            name: "ai_call_ally_desire_factor",
            description: r#"Chance for AI to call allies.

**Example:**
```paradox
ai_call_ally_desire_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "ai_desired_divisions_factor",
        HOI4Entity {
            name: "ai_desired_divisions_factor",
            description: r#"The amount of divisions AI seeks to produce.

**Example:**
```paradox
ai_desired_divisions_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "ai_focus_aggressive_factor",
        HOI4Entity {
            name: "ai_focus_aggressive_factor",
            description: r#"AI's focus on offense.

**Example:**
```paradox
ai_focus_aggressive_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "ai_focus_defense_factor",
        HOI4Entity {
            name: "ai_focus_defense_factor",
            description: r#"AI's focus on defense.

**Example:**
```paradox
ai_focus_defense_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "ai_focus_aviation_factor",
        HOI4Entity {
            name: "ai_focus_aviation_factor",
            description: r#"AI's focus on aviation.

**Example:**
```paradox
ai_focus_aviation_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "ai_focus_military_advancements_factor",
        HOI4Entity {
            name: "ai_focus_military_advancements_factor",
            description: r#"AI's focus on advanced military technologies.

**Example:**
```paradox
ai_focus_military_advancements_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "ai_focus_military_equipment_factor",
        HOI4Entity {
            name: "ai_focus_military_equipment_factor",
            description: r#"AI's focus on advanced military equipment.

**Example:**
```paradox
ai_focus_military_equipment_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "ai_focus_naval_air_factor",
        HOI4Entity {
            name: "ai_focus_naval_air_factor",
            description: r#"AI's focus on building naval airforce.

**Example:**
```paradox
ai_focus_naval_air_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "ai_focus_naval_factor",
        HOI4Entity {
            name: "ai_focus_naval_factor",
            description: r#"AI's focus on building a navy.

**Example:**
```paradox
ai_focus_naval_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "ai_focus_peaceful_factor",
        HOI4Entity {
            name: "ai_focus_peaceful_factor",
            description: r#"AI's focus on peaceful research and policies.

**Example:**
```paradox
ai_focus_peaceful_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "ai_focus_war_production_factor",
        HOI4Entity {
            name: "ai_focus_war_production_factor",
            description: r#"AI's focus on wartime production.

**Example:**
```paradox
ai_focus_war_production_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "ai_get_ally_desire_factor",
        HOI4Entity {
            name: "ai_get_ally_desire_factor",
            description: r#"AI's desire to be in or expand a faction.

**Example:**
```paradox
ai_get_ally_desire_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "ai_join_ally_desire_factor",
        HOI4Entity {
            name: "ai_join_ally_desire_factor",
            description: r#"AI's desire to join the wars led by allies.

**Example:**
```paradox
ai_join_ally_desire_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "ai_license_acceptance",
        HOI4Entity {
            name: "ai_license_acceptance",
            description: r#"AI's chance to agree licensing equipment.

**Example:**
```paradox
ai_license_acceptance = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "command_power_gain",
        HOI4Entity {
            name: "command_power_gain",
            description: r#"Changes the daily gain of command power.

**Example:**
```paradox
command_power_gain = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "command_power_gain_mult",
        HOI4Entity {
            name: "command_power_gain_mult",
            description: r#"Changes the daily gain of command power by a percentage.

**Example:**
```paradox
command_power_gain_mult = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "conscription",
        HOI4Entity {
            name: "conscription",
            description: r#"Changes the recruitable percentage of the total population.

**Example:**
```paradox
conscription = 0.02
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "conscription_factor",
        HOI4Entity {
            name: "conscription_factor",
            description: r#"Changes the recruitable percentage of the total population by a percent.

**Example:**
```paradox
conscription_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "experience_gain_army",
        HOI4Entity {
            name: "experience_gain_army",
            description: r#"Modifies the daily gain of army experience.

**Example:**
```paradox
experience_gain_army = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "experience_gain_army_factor",
        HOI4Entity {
            name: "experience_gain_army_factor",
            description: r#"Modifies the gain of army experience by a percentage.

**Example:**
```paradox
experience_gain_army_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "experience_gain_navy",
        HOI4Entity {
            name: "experience_gain_navy",
            description: r#"Modifies the daily gain of naval experience.

**Example:**
```paradox
experience_gain_navy = 0.02
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "experience_gain_navy_factor",
        HOI4Entity {
            name: "experience_gain_navy_factor",
            description: r#"Modifies the gain of naval experience by a percentage.

**Example:**
```paradox
experience_gain_navy_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "experience_gain_air",
        HOI4Entity {
            name: "experience_gain_air",
            description: r#"Modifies the daily gain of air experience.

**Example:**
```paradox
experience_gain_air = 0.05
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "experience_gain_air_factor",
        HOI4Entity {
            name: "experience_gain_air_factor",
            description: r#"Modifies the daily gain of air experience by a percentage.

**Example:**
```paradox
experience_gain_air_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "land_equipment_upgrade_xp_cost",
        HOI4Entity {
            name: "land_equipment_upgrade_xp_cost",
            description: r#"Changes the experience cost to upgrade land army equipment.

**Example:**
```paradox
land_equipment_upgrade_xp_cost = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "land_reinforce_rate",
        HOI4Entity {
            name: "land_reinforce_rate",
            description: r#"Changes the rate at which reinforcements to divisions arrive.

**Example:**
```paradox
land_reinforce_rate = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "max_command_power",
        HOI4Entity {
            name: "max_command_power",
            description: r#"Changes maximum command power.

**Example:**
```paradox
max_command_power = 20
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "max_command_power_mult",
        HOI4Entity {
            name: "max_command_power_mult",
            description: r#"Changes maximum command power by a percentage.

**Example:**
```paradox
max_command_power_mult = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "weekly_manpower",
        HOI4Entity {
            name: "weekly_manpower",
            description: r#"Amount of manpower gained each week.

**Example:**
```paradox
weekly_manpower = 1000
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "naval_equipment_upgrade_xp_cost",
        HOI4Entity {
            name: "naval_equipment_upgrade_xp_cost",
            description: r#"Changes the naval experience cost to upgrade equipment.

**Example:**
```paradox
naval_equipment_upgrade_xp_cost = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "refit_ic_cost",
        HOI4Entity {
            name: "refit_ic_cost",
            description: r#"The IC cost to refit naval equipment.

**Example:**
```paradox
refit_ic_cost = 20
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "refit_speed",
        HOI4Entity {
            name: "refit_speed",
            description: r#"The speed at which naval equipment is refitted.

**Example:**
```paradox
refit_speed = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_equipment_upgrade_xp_cost",
        HOI4Entity {
            name: "air_equipment_upgrade_xp_cost",
            description: r#"Changes the air experience cost to upgrade equipment.

**Example:**
```paradox
air_equipment_upgrade_xp_cost = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "training_time_factor",
        HOI4Entity {
            name: "training_time_factor",
            description: r#"Modifies the training time for both army and navy.

**Example:**
```paradox
training_time_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "minimum_training_level",
        HOI4Entity {
            name: "minimum_training_level",
            description: r#"Changes training level necessary for the unit to deploy.

**Example:**
```paradox
minimum_training_level = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "max_training",
        HOI4Entity {
            name: "max_training",
            description: r#"Modifies the required experience to achieve full training.

**Example:**
```paradox
max_training = -0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "training_time_army_factor",
        HOI4Entity {
            name: "training_time_army_factor",
            description: r#"Modifies the training time for the army.

**Example:**
```paradox
training_time_army_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "special_forces_training_time_factor",
        HOI4Entity {
            name: "special_forces_training_time_factor",
            description: r#"Changes the time it takes to train special forces.

**Example:**
```paradox
special_forces_training_time_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "doctrine_cost_factor",
        HOI4Entity {
            name: "doctrine_cost_factor",
            description: r#"Changes the cost of buying a new doctrine of the specified type.

**Example:**
```paradox
land_doctrine_cost_factor = -0.05
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("mastery_gain_factor", HOI4Entity {
        name: "mastery_gain_factor",
        description: r#"Modifies the speed at which mastery in a given doctrine folder is gained.

**Example:**
```paradox
`land_mastery_gain_factor = 0.15`
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "track_mastery_gain_factor",
        HOI4Entity {
            name: "track_mastery_gain_factor",
            description: r#"Modifies the speed at which mastery of a given track is gained.

**Example:**
```paradox
`operations_track_mastery_gain_factor = 0.1`
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "choose_preferred_tactics_cost",
        HOI4Entity {
            name: "choose_preferred_tactics_cost",
            description: r#"Changes the cost to choose a preferred tactic.

**Example:**
```paradox
choose_preferred_tactics_cost = 5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "command_abilities_cost_factor",
        HOI4Entity {
            name: "command_abilities_cost_factor",
            description: r#"Changes the cost to choose a command ability.

**Example:**
```paradox
command_abilities_cost_factor = -0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "transport_capacity",
        HOI4Entity {
            name: "transport_capacity",
            description: r#"Modifies how many convoys units require to be transported over sea.

**Example:**
```paradox
transport_capacity = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("paratroopers_special_forces_contribution_factor", HOI4Entity {
        name: "paratroopers_special_forces_contribution_factor",
        description: r#"Modifies how much paratroopers contribute to the limit of special forces on a template.

**Example:**
```paradox
paratroopers_special_forces_contribution_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("marines_special_forces_contribution_factor", HOI4Entity {
        name: "marines_special_forces_contribution_factor",
        description: r#"Modifies how much marines contribute to the limit of special forces on a template.

**Example:**
```paradox
marines_special_forces_contribution_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("mountaineers_special_forces_contribution_factor", HOI4Entity {
        name: "mountaineers_special_forces_contribution_factor",
        description: r#"Modifies how much mountaineers contribute to the limit of special forces on a template.

**Example:**
```paradox
mountaineers_special_forces_contribution_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("rangers_special_forces_contribution_factor", HOI4Entity {
        name: "rangers_special_forces_contribution_factor",
        description: r#"Modifies how much rangers contribute to the limit of special forces on a template.

**Example:**
```paradox
rangers_special_forces_contribution_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("special_forces_cap_flat", HOI4Entity {
        name: "special_forces_cap_flat",
        description: r#"Modifies how many special forces sub-units can be put into a single template.

**Example:**
```paradox
special_forces_cap_flat = 10
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("additional_brigade_column_size", HOI4Entity {
        name: "additional_brigade_column_size",
        description: r#"Changes the amount of maximum unlocked slots on each brigade column in division templates.

**Example:**
```paradox
additional_brigade_column_size = 1
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("unit__design_cost_factor", HOI4Entity {
        name: "unit__design_cost_factor",
        description: r#"Modifies how much experience it costs to add a brigade of the specified type to a template.

**Example:**
```paradox
unit_artillery_brigade_design_cost_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("design_cost_factor", HOI4Entity {
        name: "design_cost_factor",
        description: r#"Modifies how much experience it costs to add upgrades or modules to a specified equipment archetype.

**Example:**
```paradox
strat_bomber_equipment_design_cost_factor = 0.3
```

```paradox
ship_hull_heavy_design_cost_factor = -0.2
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("module__design_cost_factor", HOI4Entity {
        name: "module__design_cost_factor",
        description: r#"Modifies how much experience it costs to add a module of the specified type to equipment.

**Example:**
```paradox
module_tank_torsion_bar_suspension_design_cost_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "military_industrial_organization_research_bonus",
        HOI4Entity {
            name: "military_industrial_organization_research_bonus",
            description: r#"Modifies the research bonus granted by MIOs.

**Example:**
```paradox
military_industrial_organization_research_bonus = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "military_industrial_organization_design_team_assign_cost",
        HOI4Entity {
            name: "military_industrial_organization_design_team_assign_cost",
            description: r#"Modifies the political power cost to assign a design team.

**Example:**
```paradox
military_industrial_organization_design_team_assign_cost = 30
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "military_industrial_organization_design_team_change_cost",
        HOI4Entity {
            name: "military_industrial_organization_design_team_change_cost",
            description: r#"Modifies the political power cost to change a design team.

**Example:**
```paradox
military_industrial_organization_design_team_change_cost = 20
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "military_industrial_organization_industrial_manufacturer_assign_cost",
        HOI4Entity {
            name: "military_industrial_organization_industrial_manufacturer_assign_cost",
            description: r#"Modifies the political power cost to assign an industrial manufacturer.

**Example:**
```paradox
military_industrial_organization_industrial_manufacturer_assign_cost = 10
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "military_industrial_organization_task_capacity",
        HOI4Entity {
            name: "military_industrial_organization_task_capacity",
            description: r#"Modifies the amount of tasks possible to assign to the MIO.

**Example:**
```paradox
military_industrial_organization_task_capacity = 2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "military_industrial_organization_size_up_requirement",
        HOI4Entity {
            name: "military_industrial_organization_size_up_requirement",
            description: r#"Modifies the requirement to size up a MIO.

**Example:**
```paradox
military_industrial_organization_size_up_requirement = 2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "military_industrial_organization_funds_gain",
        HOI4Entity {
            name: "military_industrial_organization_funds_gain",
            description: r#"Modifies the amount of funds gained by the MIO.

**Example:**
```paradox
military_industrial_organization_funds_gain = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "military_industrial_organization_policy_cost",
        HOI4Entity {
            name: "military_industrial_organization_policy_cost",
            description: r#"Modifies the political power cost to assign a MIO policy.

**Example:**
```paradox
military_industrial_organization_policy_cost = 20
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("military_industrial_organization_policy_cooldown", HOI4Entity {
        name: "military_industrial_organization_policy_cooldown",
        description: r#"Modifies the cooldown between how often it's possible to change policies.

**Example:**
```paradox
military_industrial_organization_policy_cooldown = 5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "female_random_army_leader_chance",
        HOI4Entity {
            name: "female_random_army_leader_chance",
            description: r#"Changes the chance for a randomly-generated army leader to be female.

**Example:**
```paradox
female_random_army_leader_chance = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "assign_army_leader_cp_cost",
        HOI4Entity {
            name: "assign_army_leader_cp_cost",
            description: r#"Modifies the cost to assign an army leader to an army.

**Example:**
```paradox
assign_army_leader_cp_cost = -5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_leader_cost_factor",
        HOI4Entity {
            name: "army_leader_cost_factor",
            description: r#"The cost in political power to recruit an unit leader for the land army.

**Example:**
```paradox
army_leader_cost_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_leader_start_level",
        HOI4Entity {
            name: "army_leader_start_level",
            description: r#"Bonus to the starting level of generic unit leaders.

**Example:**
```paradox
army_leader_start_level = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_leader_start_attack_level",
        HOI4Entity {
            name: "army_leader_start_attack_level",
            description: r#"Bonus to the starting level of attack in generic unit leaders.

**Example:**
```paradox
army_leader_start_attack_level = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_leader_start_defense_level",
        HOI4Entity {
            name: "army_leader_start_defense_level",
            description: r#"Bonus to the starting level of defense in generic unit leaders.

**Example:**
```paradox
army_leader_start_defense_level = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_leader_start_logistics_level",
        HOI4Entity {
            name: "army_leader_start_logistics_level",
            description: r#"Bonus to the starting level of logistics in generic unit leaders.

**Example:**
```paradox
army_leader_start_logistics_level = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_leader_start_planning_level",
        HOI4Entity {
            name: "army_leader_start_planning_level",
            description: r#"Bonus to the starting level of planning in generic unit leaders.

**Example:**
```paradox
army_leader_start_planning_level = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "military_leader_cost_factor",
        HOI4Entity {
            name: "military_leader_cost_factor",
            description: r#"The cost in political power to recruit an unit leader.

**Example:**
```paradox
military_leader_cost_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "female_random_admiral_chance",
        HOI4Entity {
            name: "female_random_admiral_chance",
            description: r#"Changes the chance for a randomly-generated admiral to be female.

**Example:**
```paradox
female_random_admiral_chance = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "assign_navy_leader_cp_cost",
        HOI4Entity {
            name: "assign_navy_leader_cp_cost",
            description: r#"Modifies the cost to assign a navy leader to a navy.

**Example:**
```paradox
assign_navy_leader_cp_cost = -5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_leader_cost_factor",
        HOI4Entity {
            name: "navy_leader_cost_factor",
            description: r#"The cost in political power to recruit an unit leader for the land navy.

**Example:**
```paradox
navy_leader_cost_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_leader_start_level",
        HOI4Entity {
            name: "navy_leader_start_level",
            description: r#"Bonus to the starting level of generic unit leaders.

**Example:**
```paradox
navy_leader_start_level = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_leader_start_attack_level",
        HOI4Entity {
            name: "navy_leader_start_attack_level",
            description: r#"Bonus to the starting level of attack in generic unit leaders.

**Example:**
```paradox
navy_leader_start_attack_level = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_leader_start_coordination_level",
        HOI4Entity {
            name: "navy_leader_start_coordination_level",
            description: r#"Bonus to the starting level of coordination in generic unit leaders.

**Example:**
```paradox
navy_leader_start_coordination_level = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_leader_start_defense_level",
        HOI4Entity {
            name: "navy_leader_start_defense_level",
            description: r#"Bonus to the starting level of defense in generic unit leaders.

**Example:**
```paradox
navy_leader_start_defense_level = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_leader_start_maneuvering_level",
        HOI4Entity {
            name: "navy_leader_start_maneuvering_level",
            description: r#"Bonus to the starting level of maneuvering in generic unit leaders.

**Example:**
```paradox
navy_leader_start_maneuvering_level = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("grant_medal_cost_factor", HOI4Entity {
        name: "grant_medal_cost_factor",
        description: r#"Changes the cost in command power to grant a medal to a division commander.

**Example:**
```paradox
grant_medal_cost_factor = 0.2
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("field_officer_promotion_penalty", HOI4Entity {
        name: "field_officer_promotion_penalty",
        description: r#"Changes the experience penalty applied to the divisions when a commander is promoted to a field marshal.

**Example:**
```paradox
field_officer_promotion_penalty = 0.2
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "female_divisional_commander_chance",
        HOI4Entity {
            name: "female_divisional_commander_chance",
            description: r#"Changes the chance to get a female divisional commander.

**Example:**
```paradox
female_divisional_commander_chance = 0.2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "offence",
        HOI4Entity {
            name: "offence",
            description: r#"Modifies the attack value of our military, navy, and airforce.

**Example:**
```paradox
offence = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "defence",
        HOI4Entity {
            name: "defence",
            description: r#"Modifies the defence value of our military, navy, and airforce.

**Example:**
```paradox
defence = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "preferred_weight_factor",
        HOI4Entity {
            name: "preferred_weight_factor",
            description: r#"Modifies the chance for a commander to choose the specified tactic.

**Example:**
```paradox
tactic_ambush_preferred_weight_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "acclimatization_cold_climate_gain_factor",
        HOI4Entity {
            name: "acclimatization_cold_climate_gain_factor",
            description: r#"Cold acclimatization gain factor.

**Example:**
```paradox
acclimatization_cold_climate_gain_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "acclimatization_hot_climate_gain_factor",
        HOI4Entity {
            name: "acclimatization_hot_climate_gain_factor",
            description: r#"Hot acclimatization gain factor.

**Example:**
```paradox
acclimatization_hot_climate_gain_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_superiority_bonus_in_combat",
        HOI4Entity {
            name: "air_superiority_bonus_in_combat",
            description: r#"The bonus in combat given from having air superiority.

**Example:**
```paradox
air_superiority_bonus_in_combat = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_attack_factor",
        HOI4Entity {
            name: "army_attack_factor",
            description: r#"The bonus to land army's attack.

**Example:**
```paradox
army_attack_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_core_attack_factor",
        HOI4Entity {
            name: "army_core_attack_factor",
            description: r#"The bonus to land army's attack on core territory.

**Example:**
```paradox
army_core_attack_factor = 0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_claim_attack_factor",
        HOI4Entity {
            name: "army_claim_attack_factor",
            description: r#"The bonus to land army's attack on claimed territory.

**Example:**
```paradox
army_claim_attack_factor = 0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_attack_against_major_factor",
        HOI4Entity {
            name: "army_attack_against_major_factor",
            description: r#"The bonus to land army's attack against a major country.

**Example:**
```paradox
army_attack_against_major_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_attack_against_minor_factor",
        HOI4Entity {
            name: "army_attack_against_minor_factor",
            description: r#"The bonus to land army's attack against a non-major country.

**Example:**
```paradox
army_attack_against_minor_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_attack_speed_factor",
        HOI4Entity {
            name: "army_attack_speed_factor",
            description: r#"The bonus to speed at which the land army attacks.

**Example:**
```paradox
army_attack_speed_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_breakthrough_against_major_factor",
        HOI4Entity {
            name: "army_breakthrough_against_major_factor",
            description: r#"The bonus to land army's breakthrough against a major country.

**Example:**
```paradox
army_breakthrough_against_major_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_breakthrough_against_minor_factor",
        HOI4Entity {
            name: "army_breakthrough_against_minor_factor",
            description: r#"The bonus to land army's breakthrough against a non-major country.

**Example:**
```paradox
army_breakthrough_against_minor_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_defence_factor",
        HOI4Entity {
            name: "army_defence_factor",
            description: r#"The bonus to land army's defence.

**Example:**
```paradox
army_defence_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_defence_against_major_factor",
        HOI4Entity {
            name: "army_defence_against_major_factor",
            description: r#"The bonus to land army's defence against a major country.

**Example:**
```paradox
army_defence_against_major_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_defence_against_minor_factor",
        HOI4Entity {
            name: "army_defence_against_minor_factor",
            description: r#"The bonus to land army's defence against a non-major country.

**Example:**
```paradox
army_defence_against_minor_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_core_defence_factor",
        HOI4Entity {
            name: "army_core_defence_factor",
            description: r#"The bonus to land army's defence on core territory.

**Example:**
```paradox
army_core_defence_factor = 0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_claim_defence_factor",
        HOI4Entity {
            name: "army_claim_defence_factor",
            description: r#"The bonus to land army's defence on claimed territory.

**Example:**
```paradox
army_claim_defence_factor = 0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_speed_factor",
        HOI4Entity {
            name: "army_speed_factor",
            description: r#"The bonus to land army's speed.

**Example:**
```paradox
army_speed_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_strength_factor",
        HOI4Entity {
            name: "army_strength_factor",
            description: r#"The bonus to land army's strength.

**Example:**
```paradox
army_strength_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "attack_factor",
        HOI4Entity {
            name: "attack_factor",
            description: r#"The bonus to specified unit type's attack.

**Example:**
```paradox
cavalry_attack_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "defence_factor",
        HOI4Entity {
            name: "defence_factor",
            description: r#"The bonus to the specified unit type's defence.

**Example:**
```paradox
army_artillery_defence_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "speed_factor",
        HOI4Entity {
            name: "speed_factor",
            description: r#"The bonus to specified unit type's speed.

**Example:**
```paradox
army_armor_speed_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "modifier_army_sub_unit__attack_factor",
        HOI4Entity {
            name: "modifier_army_sub_unit__attack_factor",
            description: r#"The bonus to specified unit type's attack.

**Example:**
```paradox
modifier_army_sub_unit_armored_car_attack_factor = 0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "modifier_army_sub_unit__defence_factor",
        HOI4Entity {
            name: "modifier_army_sub_unit__defence_factor",
            description: r#"The bonus to the specified unit type's defence.

**Example:**
```paradox
modifier_army_sub_unit_armored_car_defence_factor = 0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "modifier_army_sub_unit__speed_factor",
        HOI4Entity {
            name: "modifier_army_sub_unit__speed_factor",
            description: r#"The bonus to specified unit type's speed.

**Example:**
```paradox
modifier_army_sub_unit_armored_car_speed_factor = 0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_morale",
        HOI4Entity {
            name: "army_morale",
            description: r#"Modifies the division recovery rate.

**Example:**
```paradox
army_morale = 10
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_morale_factor",
        HOI4Entity {
            name: "army_morale_factor",
            description: r#"Modifies the division recovery rate by a percentage.

**Example:**
```paradox
army_morale_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_org",
        HOI4Entity {
            name: "army_org",
            description: r#"Modifies the army's organisation.

**Example:**
```paradox
army_org = 10
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_org_factor",
        HOI4Entity {
            name: "army_org_factor",
            description: r#"Modifies the army's organisation by a percentage.

**Example:**
```paradox
army_org_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_org_regain",
        HOI4Entity {
            name: "army_org_regain",
            description: r#"Modifies the army's organisation regain speed by a percentage.

**Example:**
```paradox
army_org_regain = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "breakthrough_factor",
        HOI4Entity {
            name: "breakthrough_factor",
            description: r#"Modifies the army's breakthrough.

**Example:**
```paradox
breakthrough_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "cas_damage_reduction",
        HOI4Entity {
            name: "cas_damage_reduction",
            description: r#"Reduces the damage dealt by close air support.

**Example:**
```paradox
cas_damage_reduction = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "combat_width_factor",
        HOI4Entity {
            name: "combat_width_factor",
            description: r#"Changes our own combat width.

**Example:**
```paradox
combat_width_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("coordination_bonus", HOI4Entity {
        name: "coordination_bonus",
        description: r#"Changes the bonus to coordination, that is how much damage is done to the primary target instead of being spread out.

**Example:**
```paradox
coordination_bonus = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "dig_in_speed",
        HOI4Entity {
            name: "dig_in_speed",
            description: r#"Changes entrenchment speed.

**Example:**
```paradox
dig_in_speed = 2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "dig_in_speed_factor",
        HOI4Entity {
            name: "dig_in_speed_factor",
            description: r#"Changes entrenchment speed by a percentage.

**Example:**
```paradox
dig_in_speed_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "experience_gain_army_unit",
        HOI4Entity {
            name: "experience_gain_army_unit",
            description: r#"Changes experience gain by the army divisions.

**Example:**
```paradox
experience_gain_army_unit = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "experience_gain_army_unit_factor",
        HOI4Entity {
            name: "experience_gain_army_unit_factor",
            description: r#"Changes experience gain by the army divisions by a percentage.

**Example:**
```paradox
experience_gain_army_unit_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "experience_loss_factor",
        HOI4Entity {
            name: "experience_loss_factor",
            description: r#"Changes the loss in divisions' experience in combat.

**Example:**
```paradox
experience_loss_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "initiative_factor",
        HOI4Entity {
            name: "initiative_factor",
            description: r#"Modifies the initiative.

**Example:**
```paradox
initiative_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "land_night_attack",
        HOI4Entity {
            name: "land_night_attack",
            description: r#"Changes the penalty due to attacking at night.

**Example:**
```paradox
land_night_attack = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "max_dig_in",
        HOI4Entity {
            name: "max_dig_in",
            description: r#"Changes the maximum entrenchment.

**Example:**
```paradox
max_dig_in = 20
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "max_dig_in_factor",
        HOI4Entity {
            name: "max_dig_in_factor",
            description: r#"Changes the maximum entrenchment by a percentage.

**Example:**
```paradox
max_dig_in_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "max_planning",
        HOI4Entity {
            name: "max_planning",
            description: r#"Changes the maximum planning.

**Example:**
```paradox
max_planning = 20
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "max_planning_factor",
        HOI4Entity {
            name: "max_planning_factor",
            description: r#"Changes the maximum planning by a percentage.

**Example:**
```paradox
max_planning_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "pocket_penalty",
        HOI4Entity {
            name: "pocket_penalty",
            description: r#"Reduces the penalty that troops take when they are encircled.

**Example:**
```paradox
pocket_penalty = 0.2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "recon_factor",
        HOI4Entity {
            name: "recon_factor",
            description: r#"Changes reconnaisance.

**Example:**
```paradox
recon_factor = 0.2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "recon_factor_while_entrenched",
        HOI4Entity {
            name: "recon_factor_while_entrenched",
            description: r#"Changes reconnaisance for entrenched divisions.

**Example:**
```paradox
recon_factor_while_entrenched = 0.2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "special_forces_cap",
        HOI4Entity {
            name: "special_forces_cap",
            description: r#"Changes the maximum amount of special forces by a percentage.

**Example:**
```paradox
special_forces_cap = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "special_forces_min",
        HOI4Entity {
            name: "special_forces_min",
            description: r#"Changes the minimum amount of special forces.

**Example:**
```paradox
special_forces_min = 250
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "terrain_penalty_reduction",
        HOI4Entity {
            name: "terrain_penalty_reduction",
            description: r#"Decreases the penalties given by terrain.

**Example:**
```paradox
terrain_penalty_reduction = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("org_loss_at_low_org_factor", HOI4Entity {
        name: "org_loss_at_low_org_factor",
        description: r#"Modifies the organisation loss for units when they have low organisation.

**Example:**
```paradox
org_loss_at_low_org_factor = 0.2
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "org_loss_when_moving",
        HOI4Entity {
            name: "org_loss_when_moving",
            description: r#"Modifies the organisation loss for units when they are moving.

**Example:**
```paradox
org_loss_when_moving = 0.2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "planning_speed",
        HOI4Entity {
            name: "planning_speed",
            description: r#"Modifies the planning speed.

**Example:**
```paradox
planning_speed = 0.2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "experience_gain__combat_factor",
        HOI4Entity {
            name: "experience_gain__combat_factor",
            description: r#"Modifies the experience gain in combat for the unit type.

**Example:**
```paradox
experience_gain_artillery_combat_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "experience_gain__training_factor",
        HOI4Entity {
            name: "experience_gain__training_factor",
            description: r#"Modifies the experience gain in training for the unit type.

**Example:**
```paradox
experience_gain_destroyer_training_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "naval_invasion_prep_speed",
        HOI4Entity {
            name: "naval_invasion_prep_speed",
            description: r#"Modifies the speed at which a naval invasion is prepared.

**Example:**
```paradox
naval_invasion_prep_speed = 10
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("naval_invasion_capacity", HOI4Entity {
        name: "naval_invasion_capacity",
        description: r#"Modifies the amount of divisions that can have a naval invasion plan going on at the same time.

**Example:**
```paradox
naval_invasion_capacity = 10
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "naval_invasion_penalty",
        HOI4Entity {
            name: "naval_invasion_penalty",
            description: r#"Modifies the penalty for naval invasions.

**Example:**
```paradox
naval_invasion_penalty = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("naval_invasion_planning_bonus_speed", HOI4Entity {
        name: "naval_invasion_planning_bonus_speed",
        description: r#"Modifies the speed at which the planning bonus is accumulated during a naval invasion preparation.

**Example:**
```paradox
naval_invasion_planning_bonus_speed = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "amphibious_invasion",
        HOI4Entity {
            name: "amphibious_invasion",
            description: r#"Modifies the speed of units during naval invasions.

**Example:**
```paradox
amphibious_invasion = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "amphibious_invasion_defence",
        HOI4Entity {
            name: "amphibious_invasion_defence",
            description: r#"Modifies the penalty given by naval invasions.

**Example:**
```paradox
amphibious_invasion_defence = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "invasion_preparation",
        HOI4Entity {
            name: "invasion_preparation",
            description: r#"Modifies the required preparation needed to execute a naval invasion.

**Example:**
```paradox
invasion_preparation = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "convoy_escort_efficiency",
        HOI4Entity {
            name: "convoy_escort_efficiency",
            description: r#"Modifies the efficiency of the convoy escort mission.

**Example:**
```paradox
convoy_escort_efficiency = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "convoy_raiding_efficiency_factor",
        HOI4Entity {
            name: "convoy_raiding_efficiency_factor",
            description: r#"Modifies the efficiency of the convoy raiding mission.

**Example:**
```paradox
convoy_raiding_efficiency_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "convoy_retreat_speed",
        HOI4Entity {
            name: "convoy_retreat_speed",
            description: r#"Modifies the speed of convoys retreating.

**Example:**
```paradox
convoy_retreat_speed = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("critical_receive_chance", HOI4Entity {
        name: "critical_receive_chance",
        description: r#"Changes the chance for the enemy to get a critical hit on us in naval combat.

**Example:**
```paradox
critical_receive_chance = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "experience_gain_navy_unit",
        HOI4Entity {
            name: "experience_gain_navy_unit",
            description: r#"Modifies the daily gain of experience by the ships.

**Example:**
```paradox
experience_gain_navy_unit = 0.02
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "experience_gain_navy_unit_factor",
        HOI4Entity {
            name: "experience_gain_navy_unit_factor",
            description: r#"Modifies the gain of experience by the ships by a percentage.

**Example:**
```paradox
experience_gain_navy_unit_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "mines_planting_by_fleets_factor",
        HOI4Entity {
            name: "mines_planting_by_fleets_factor",
            description: r#"Modifies the efficiency of the mine planting mission.

**Example:**
```paradox
mines_planting_by_fleets_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "mines_sweeping_by_fleets_factor",
        HOI4Entity {
            name: "mines_sweeping_by_fleets_factor",
            description: r#"Modifies the efficiency of the mine sweeping mission.

**Example:**
```paradox
mines_sweeping_by_fleets_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "naval_accidents_chance",
        HOI4Entity {
            name: "naval_accidents_chance",
            description: r#"Modifies the chance for a ship to be accidentally sunk or damaged.

**Example:**
```paradox
naval_accidents_chance = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_anti_air_attack",
        HOI4Entity {
            name: "navy_anti_air_attack",
            description: r#"Modifies the attack against enemy airplanes for the country's ships.

**Example:**
```paradox
navy_anti_air_attack = 5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("navy_anti_air_attack_factor", HOI4Entity {
        name: "navy_anti_air_attack_factor",
        description: r#"Modifies the attack against enemy airplanes for the country's ships by a percentage.

**Example:**
```paradox
navy_anti_air_attack_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("naval_coordination", HOI4Entity {
        name: "naval_coordination",
        description: r#"Modifies how quickly the fleet can gather or disperse when a target is found or when switching missions.

**Example:**
```paradox
naval_coordination = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "naval_critical_effect_factor",
        HOI4Entity {
            name: "naval_critical_effect_factor",
            description: r#"Modifies the effects of sustained critical hits on our ships.

**Example:**
```paradox
naval_critical_effect_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("naval_critical_score_chance_factor", HOI4Entity {
        name: "naval_critical_score_chance_factor",
        description: r#"Modifies the chance for us to get a critical hit on the enemy in naval combat.

**Example:**
```paradox
naval_critical_score_chance_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "naval_damage_factor",
        HOI4Entity {
            name: "naval_damage_factor",
            description: r#"Modifies the damage dealt by our ships.

**Example:**
```paradox
naval_damage_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "naval_defense_factor",
        HOI4Entity {
            name: "naval_defense_factor",
            description: r#"Modifies the damage received by our ships.

**Example:**
```paradox
naval_defense_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "naval_detection",
        HOI4Entity {
            name: "naval_detection",
            description: r#"Modifies the chance for our ships to detect submarines.

**Example:**
```paradox
naval_detection = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("naval_enemy_fleet_size_ratio_penalty_factor", HOI4Entity {
        name: "naval_enemy_fleet_size_ratio_penalty_factor",
        description: r#"Modifies the penalty the enemy receives for having a larger amount of ships than us.

**Example:**
```paradox
naval_enemy_fleet_size_ratio_penalty_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "naval_enemy_positioning_in_initial_attack",
        HOI4Entity {
            name: "naval_enemy_positioning_in_initial_attack",
            description: r#"Modifies the positioning of the enemy during the initial naval attack.

**Example:**
```paradox
naval_enemy_positioning_in_initial_attack = 3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "naval_enemy_retreat_chance",
        HOI4Entity {
            name: "naval_enemy_retreat_chance",
            description: r#"Modifies the chance for the enemy to retreat.

**Example:**
```paradox
naval_enemy_retreat_chance = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("naval_has_potf_in_combat_attack", HOI4Entity {
        name: "naval_has_potf_in_combat_attack",
        description: r#"Modifies the attack of the navy when fighting together with the pride of the fleet.

**Example:**
```paradox
naval_has_potf_in_combat_attack = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("naval_has_potf_in_combat_defense", HOI4Entity {
        name: "naval_has_potf_in_combat_defense",
        description: r#"Modifies the defense of the navy when fighting together with the pride of the fleet.

**Example:**
```paradox
naval_has_potf_in_combat_defense = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "naval_hit_chance",
        HOI4Entity {
            name: "naval_hit_chance",
            description: r#"Modifies the chance for the naval attacks to land.

**Example:**
```paradox
naval_hit_chance = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "naval_mine_hit_chance",
        HOI4Entity {
            name: "naval_mine_hit_chance",
            description: r#"Modifies the chance for a naval mine to hit.

**Example:**
```paradox
naval_mine_hit_chance = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "naval_mines_damage_factor",
        HOI4Entity {
            name: "naval_mines_damage_factor",
            description: r#"Modifies the damage naval mines deal to enemy ships.

**Example:**
```paradox
naval_mines_damage_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "naval_mines_effect_reduction",
        HOI4Entity {
            name: "naval_mines_effect_reduction",
            description: r#"Modifies the damage enemy naval mines deal.

**Example:**
```paradox
naval_mines_effect_reduction = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "naval_morale",
        HOI4Entity {
            name: "naval_morale",
            description: r#"Modifies the navy recovery rate.

**Example:**
```paradox
naval_morale = 15
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "naval_morale_factor",
        HOI4Entity {
            name: "naval_morale_factor",
            description: r#"Modifies the navy recovery rate by a percentage.

**Example:**
```paradox
naval_morale_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "naval_night_attack",
        HOI4Entity {
            name: "naval_night_attack",
            description: r#"Modifies the damage dealt by the country's ships at night.

**Example:**
```paradox
naval_night_attack = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "naval_retreat_chance",
        HOI4Entity {
            name: "naval_retreat_chance",
            description: r#"Modifies the chance for the country's ships to retreat.

**Example:**
```paradox
naval_retreat_chance = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("naval_retreat_chance_after_initial_combat", HOI4Entity {
        name: "naval_retreat_chance_after_initial_combat",
        description: r#"Modifies the chance for the country's ships to retreat after initial combat.

**Example:**
```paradox
naval_retreat_chance_after_initial_combat = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "naval_retreat_speed",
        HOI4Entity {
            name: "naval_retreat_speed",
            description: r#"Modifies the speed at which the country's ships retreat.

**Example:**
```paradox
naval_retreat_speed = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("naval_retreat_speed_after_initial_combat", HOI4Entity {
        name: "naval_retreat_speed_after_initial_combat",
        description: r#"Modifies the speed at which the country's ships to retreat after initial combat.

**Example:**
```paradox
naval_retreat_speed_after_initial_combat = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "naval_speed_factor",
        HOI4Entity {
            name: "naval_speed_factor",
            description: r#"Modifies the speed of the country's ships.

**Example:**
```paradox
naval_speed_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_org",
        HOI4Entity {
            name: "navy_org",
            description: r#"Modifies the navy's organisation.

**Example:**
```paradox
navy_org = 10
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_org_factor",
        HOI4Entity {
            name: "navy_org_factor",
            description: r#"Modifies the navy's organisation by a percentage.

**Example:**
```paradox
navy_org_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_max_range",
        HOI4Entity {
            name: "navy_max_range",
            description: r#"Modifies the navy's maximum range.

**Example:**
```paradox
navy_max_range = 10
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_max_range_factor",
        HOI4Entity {
            name: "navy_max_range_factor",
            description: r#"Modifies the navy's maximum range by a percentage.

**Example:**
```paradox
navy_max_range_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "naval_torpedo_cooldown_factor",
        HOI4Entity {
            name: "naval_torpedo_cooldown_factor",
            description: r#"Modifies the rate at which the country's ships can fire torpedos.

**Example:**
```paradox
naval_torpedo_cooldown_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "naval_torpedo_hit_chance_factor",
        HOI4Entity {
            name: "naval_torpedo_hit_chance_factor",
            description: r#"Modifies the likelihood for country's torpedos to hit enemy ships.

**Example:**
```paradox
naval_torpedo_hit_chance_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("naval_torpedo_reveal_chance_factor", HOI4Entity {
        name: "naval_torpedo_reveal_chance_factor",
        description: r#"Modifies the chance that the country's submarines reveal themselves when firing torpedos.

**Example:**
```paradox
naval_torpedo_reveal_chance_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("naval_torpedo_screen_penetration_factor", HOI4Entity {
        name: "naval_torpedo_screen_penetration_factor",
        description: r#"Modifies the rate at which the country's torpedos penalise enemy screening.

**Example:**
```paradox
naval_torpedo_screen_penetration_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "naval_torpedo_damage_reduction_factor",
        HOI4Entity {
            name: "naval_torpedo_damage_reduction_factor",
            description: r#"Modifies the damage at which enemy torpedos damage the country's ships.

**Example:**
```paradox
naval_torpedo_damage_reduction_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("naval_torpedo_enemy_critical_chance_factor", HOI4Entity {
        name: "naval_torpedo_enemy_critical_chance_factor",
        description: r#"Modifies the chance for an enemy torpedo to get a cricical hit against the country's ships.

**Example:**
```paradox
naval_torpedo_enemy_critical_chance_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("naval_light_gun_hit_chance_factor", HOI4Entity {
        name: "naval_light_gun_hit_chance_factor",
        description: r#"Modifies the chance for the country's naval light guns to hit enemy ships.

**Example:**
```paradox
naval_light_gun_hit_chance_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("naval_heavy_gun_hit_chance_factor", HOI4Entity {
        name: "naval_heavy_gun_hit_chance_factor",
        description: r#"Modifies the chance for the country's naval heavy guns to hit enemy ships.

**Example:**
```paradox
naval_heavy_gun_hit_chance_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "navy_capital_ship_attack_factor",
        HOI4Entity {
            name: "navy_capital_ship_attack_factor",
            description: r#"Modifies the attack of the country's capital ships.

**Example:**
```paradox
navy_capital_ship_attack_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_capital_ship_defence_factor",
        HOI4Entity {
            name: "navy_capital_ship_defence_factor",
            description: r#"Modifies the defence of the country's capital ships.

**Example:**
```paradox
navy_capital_ship_defence_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_screen_attack_factor",
        HOI4Entity {
            name: "navy_screen_attack_factor",
            description: r#"Modifies the attack of the country's screening ships.

**Example:**
```paradox
navy_screen_attack_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_screen_defence_factor",
        HOI4Entity {
            name: "navy_screen_defence_factor",
            description: r#"Modifies the defence of the country's screening ships.

**Example:**
```paradox
navy_screen_defence_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_submarine_attack_factor",
        HOI4Entity {
            name: "navy_submarine_attack_factor",
            description: r#"Modifies the attack of the country's submarines.

**Example:**
```paradox
navy_submarine_attack_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_submarine_defence_factor",
        HOI4Entity {
            name: "navy_submarine_defence_factor",
            description: r#"Modifies the defence of the country's submarines.

**Example:**
```paradox
navy_submarine_defence_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_submarine_detection_factor",
        HOI4Entity {
            name: "navy_submarine_detection_factor",
            description: r#"Modifies the country's detection of enemy submarines.

**Example:**
```paradox
navy_submarine_detection_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_visibility",
        HOI4Entity {
            name: "navy_visibility",
            description: r#"Modifies the visibility of the country's navy.

**Example:**
```paradox
navy_visibility = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_weather_penalty",
        HOI4Entity {
            name: "navy_weather_penalty",
            description: r#"Modifies the penalty the country's navy gets during poor weather.

**Example:**
```paradox
navy_weather_penalty = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "night_spotting_chance",
        HOI4Entity {
            name: "night_spotting_chance",
            description: r#"Modifies the chance for the country's navy to spot the enemy at night.

**Example:**
```paradox
night_spotting_chance = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "positioning",
        HOI4Entity {
            name: "positioning",
            description: r#"Modifies the positioning of the country's navy.

**Example:**
```paradox
positioning = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "repair_speed_factor",
        HOI4Entity {
            name: "repair_speed_factor",
            description: r#"Modifies the speed at which the dockyards repair the navy.

**Example:**
```paradox
repair_speed_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "screening_efficiency",
        HOI4Entity {
            name: "screening_efficiency",
            description: r#"Modifies the efficiency screen ships operate.

**Example:**
```paradox
screening_efficiency = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "screening_without_screens",
        HOI4Entity {
            name: "screening_without_screens",
            description: r#"Modifies the base screening without any screen ships assigned.

**Example:**
```paradox
screening_without_screens = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "ships_at_battle_start",
        HOI4Entity {
            name: "ships_at_battle_start",
            description: r#"Modifies the number of ships at first contact.

**Example:**
```paradox
ships_at_battle_start = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "spotting_chance",
        HOI4Entity {
            name: "spotting_chance",
            description: r#"Modifies the chance to spot enemy ships.

**Example:**
```paradox
spotting_chance = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("strike_force_movement_org_loss", HOI4Entity {
        name: "strike_force_movement_org_loss",
        description: r#"Modifies the organisation loss from movement during the strike force mission.

**Example:**
```paradox
strike_force_movement_org_loss = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "sub_retreat_speed",
        HOI4Entity {
            name: "sub_retreat_speed",
            description: r#"Modifies the retreat speed of submarines.

**Example:**
```paradox
sub_retreat_speed = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "submarine_attack",
        HOI4Entity {
            name: "submarine_attack",
            description: r#"Modifies the attack of submarines.

**Example:**
```paradox
submarine_attack = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_carrier_air_agility_factor",
        HOI4Entity {
            name: "navy_carrier_air_agility_factor",
            description: r#"Modifies the agility of airplanes executing tasks from carriers.

**Example:**
```paradox
navy_carrier_air_agility_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_carrier_air_attack_factor",
        HOI4Entity {
            name: "navy_carrier_air_attack_factor",
            description: r#"Modifies the attack of airplanes executing tasks from carriers.

**Example:**
```paradox
navy_carrier_air_attack_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_carrier_air_targetting_factor",
        HOI4Entity {
            name: "navy_carrier_air_targetting_factor",
            description: r#"Modifies the targeting of airplanes executing tasks from carriers.

**Example:**
```paradox
navy_carrier_air_targetting_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_carrier_night_penalty_reduction_factor",
        HOI4Entity {
            name: "air_carrier_night_penalty_reduction_factor",
            description: r#"Modifies the reduction of the night penalty for air carriers.

**Example:**
```paradox
air_carrier_night_penalty_reduction_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "carrier_capacity_penalty_reduction",
        HOI4Entity {
            name: "carrier_capacity_penalty_reduction",
            description: r#"Modifies the penalty given by overcrowding a carrier with planes.

**Example:**
```paradox
carrier_capacity_penalty_reduction = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "carrier_traffic",
        HOI4Entity {
            name: "carrier_traffic",
            description: r#"Modifies the traffic of carriers.

**Example:**
```paradox
carrier_traffic = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("sortie_efficiency", HOI4Entity {
        name: "sortie_efficiency",
        description: r#"Modifies the speed when refueling and rearming planes on the carrier during the battle.

**Example:**
```paradox
sortie_efficiency = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("carrier_sortie_hours_delay", HOI4Entity {
        name: "carrier_sortie_hours_delay",
        description: r#"Modifies the delay in hours for refueling and rearming planes on the carrier.

**Example:**
```paradox
carrier_sortie_hours_delay = 2
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "carrier_night_traffic",
        HOI4Entity {
            name: "carrier_night_traffic",
            description: r#"Modifies the traffic of carriers at night.

**Example:**
```paradox
carrier_night_traffic = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("fighter_sortie_efficiency", HOI4Entity {
        name: "fighter_sortie_efficiency",
        description: r#"Modifies the speed when refueling and rearming fighter planes on the carrier during the battle.

**Example:**
```paradox
fighter_sortie_efficiency = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "air_accidents_factor",
        HOI4Entity {
            name: "air_accidents_factor",
            description: r#"Modifies the chance for air accidents to happen.

**Example:**
```paradox
air_accidents_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_ace_bonuses_factor",
        HOI4Entity {
            name: "air_ace_bonuses_factor",
            description: r#"Modifies the bonuses the aces grant.

**Example:**
```paradox
air_ace_bonuses_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_ace_generation_chance_factor",
        HOI4Entity {
            name: "air_ace_generation_chance_factor",
            description: r#"Modifies the chance for aces to appear.

**Example:**
```paradox
air_ace_generation_chance_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "ace_effectiveness_factor",
        HOI4Entity {
            name: "ace_effectiveness_factor",
            description: r#"Modifies the effectiveness of aces

**Example:**
```paradox
ace_effectiveness_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_agility_factor",
        HOI4Entity {
            name: "air_agility_factor",
            description: r#"Modifies the agility of the country's airplanes.

**Example:**
```paradox
air_agility_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_attack_factor",
        HOI4Entity {
            name: "air_attack_factor",
            description: r#"Modifies the attack of the country's airplanes.

**Example:**
```paradox
air_attack_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_defence_factor",
        HOI4Entity {
            name: "air_defence_factor",
            description: r#"Modifies the defence of the country's airplanes.

**Example:**
```paradox
air_defence_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("air_interception_detect_factor", HOI4Entity {
        name: "air_interception_detect_factor",
        description: r#"Modifies the chance of detecting an enemy plane while on interception mission.

**Example:**
```paradox
air_interception_detect_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("naval_strike_targetting_factor", HOI4Entity {
        name: "naval_strike_targetting_factor",
        description: r#"Modifies the ability of planes to target their objectives when executing naval strikes.

**Example:**
```paradox
naval_strike_targetting_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "port_strike",
        HOI4Entity {
            name: "port_strike",
            description: r#"Modifies the damage done by planes on the port strike mission.

**Example:**
```paradox
port_strike = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("air_close_air_support_org_damage_factor", HOI4Entity {
        name: "air_close_air_support_org_damage_factor",
        description: r#"Modifies the damage to division organisation by planes on the close air support mission.

**Example:**
```paradox
air_close_air_support_org_damage_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "air_bombing_targetting",
        HOI4Entity {
            name: "air_bombing_targetting",
            description: r#"Modifies targetting for ground bombing.

**Example:**
```paradox
air_bombing_targetting = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_cas_efficiency",
        HOI4Entity {
            name: "air_cas_efficiency",
            description: r#"Modifies efficiency of close-air-support.

**Example:**
```paradox
air_cas_efficiency = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_cas_present_factor",
        HOI4Entity {
            name: "air_cas_present_factor",
            description: r#"Modifies impact of close-air-support in land combat.

**Example:**
```paradox
air_cas_present_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_escort_efficiency",
        HOI4Entity {
            name: "air_escort_efficiency",
            description: r#"Modifies ability of planes in dogfights.

**Example:**
```paradox
air_escort_efficiency = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("air_home_defence_factor", HOI4Entity {
        name: "air_home_defence_factor",
        description: r#"Modifies the defence of airplanes when defending states in the home region (Connected to the country's capital by land)

**Example:**
```paradox
air_home_defence_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "air_intercept_efficiency",
        HOI4Entity {
            name: "air_intercept_efficiency",
            description: r#"Modifies the efficiency of air interception.

**Example:**
```paradox
air_intercept_efficiency = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_manpower_requirement_factor",
        HOI4Entity {
            name: "air_manpower_requirement_factor",
            description: r#"Modifies the manpower required to deploy an airplane.

**Example:**
```paradox
air_manpower_requirement_factor = -0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_maximum_speed_factor",
        HOI4Entity {
            name: "air_maximum_speed_factor",
            description: r#"Modifies the maximum speed of the airforce.

**Example:**
```paradox
air_maximum_speed_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_mission_efficiency",
        HOI4Entity {
            name: "air_mission_efficiency",
            description: r#"Modifies the efficiency of airplanes in missions.

**Example:**
```paradox
air_mission_efficiency = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_mission_xp_gain_factor",
        HOI4Entity {
            name: "air_mission_xp_gain_factor",
            description: r#"Modifies the experience gain for airplanes for doing missions.

**Example:**
```paradox
air_mission_xp_gain_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("air_nav_efficiency", HOI4Entity {
        name: "air_nav_efficiency",
        description: r#"Modifies the efficiency of airplanes doing port strike and naval bombing missions.

**Example:**
```paradox
air_nav_efficiency = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "air_night_penalty",
        HOI4Entity {
            name: "air_night_penalty",
            description: r#"Modifies the penalty the airforce receives while at night.

**Example:**
```paradox
air_night_penalty = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_power_projection_factor",
        HOI4Entity {
            name: "air_power_projection_factor",
            description: r#"Modifies the power projection given out by the airplanes.

**Example:**
```paradox
air_power_projection_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_range_factor",
        HOI4Entity {
            name: "air_range_factor",
            description: r#"Modifies the range of the airplanes.

**Example:**
```paradox
air_range_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_strategic_bomber_bombing_factor",
        HOI4Entity {
            name: "air_strategic_bomber_bombing_factor",
            description: r#"Modifies the efficiency of the strategic bombing mission.

**Example:**
```paradox
air_strategic_bomber_bombing_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_strategic_bomber_night_penalty",
        HOI4Entity {
            name: "air_strategic_bomber_night_penalty",
            description: r#"Modifies the penalty for the strategic bombing mission while at night.

**Example:**
```paradox
air_strategic_bomber_night_penalty = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("air_superiority_detect_factor", HOI4Entity {
        name: "air_superiority_detect_factor",
        description: r#"Modifies the chance to detect enemy planes while on the air superiority mission. Displays as Fighter Detection.

**Example:**
```paradox
air_superiority_detect_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "air_superiority_efficiency",
        HOI4Entity {
            name: "air_superiority_efficiency",
            description: r#"Modifies the efficiency of the air superiority mission.

**Example:**
```paradox
air_superiority_efficiency = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_training_xp_gain_factor",
        HOI4Entity {
            name: "air_training_xp_gain_factor",
            description: r#"Modifies the air experience gain from training.

**Example:**
```paradox
air_training_xp_gain_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("air_untrained_pilots_penalty_factor", HOI4Entity {
        name: "air_untrained_pilots_penalty_factor",
        description: r#"Modifies the penalty given to airplanes which don't have enough experience.

**Example:**
```paradox
air_untrained_pilots_penalty_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "air_weather_penalty",
        HOI4Entity {
            name: "air_weather_penalty",
            description: r#"Modifies the penalty the airplanes receive because of weather.

**Example:**
```paradox
air_weather_penalty = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("air_wing_xp_loss_when_killed_factor", HOI4Entity {
        name: "air_wing_xp_loss_when_killed_factor",
        description: r#"Modifies the experience loss of airplanes due to airplanes being shot down.

**Example:**
```paradox
air_wing_xp_loss_when_killed_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "army_bonus_air_superiority_factor",
        HOI4Entity {
            name: "army_bonus_air_superiority_factor",
            description: r#"Modifies the bonus to land combat from air superiority.

**Example:**
```paradox
army_bonus_air_superiority_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "enemy_army_bonus_air_superiority_factor",
        HOI4Entity {
            name: "enemy_army_bonus_air_superiority_factor",
            description: r#"Modifies the effect to land combat from enemy air superiority.

**Example:**
```paradox
enemy_army_bonus_air_superiority_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("ground_attack_factor", HOI4Entity {
        name: "ground_attack_factor",
        description: r#"Modifies the bonus to airplane attack on enemy divisions by a percentage.

**Example:**
```paradox
ground_attack_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "mines_planting_by_air_factor",
        HOI4Entity {
            name: "mines_planting_by_air_factor",
            description: r#"Modifies efficiency of airplanes planting mines.

**Example:**
```paradox
mines_planting_by_air_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "mines_sweeping_by_air_factor",
        HOI4Entity {
            name: "mines_sweeping_by_air_factor",
            description: r#"Modifies efficiency of airplanes sweeping mines.

**Example:**
```paradox
mines_sweeping_by_air_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "strategic_bomb_visibility",
        HOI4Entity {
            name: "strategic_bomb_visibility",
            description: r#"Modifies the chance for the enemy to detect our strategic bombers.

**Example:**
```paradox
strategic_bomb_visibility = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "rocket_attack_factor",
        HOI4Entity {
            name: "rocket_attack_factor",
            description: r#"Modifies the attack given to rockets.

**Example:**
```paradox
rocket_attack_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "extra_trade_to_target_factor",
        HOI4Entity {
            name: "extra_trade_to_target_factor",
            description: r#"Adds extra produced resources available for trade to target country.

**Example:**
```paradox
extra_trade_to_target_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("generate_wargoal_tension_against", HOI4Entity {
        name: "generate_wargoal_tension_against",
        description: r#"Changes world tension necessary for us to justify against the target country.

**Example:**
```paradox
generate_wargoal_tension_against = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("cic_to_target_factor", HOI4Entity {
        name: "cic_to_target_factor",
        description: r#"Gives a portion of the country's civilian industry to the specified target.

**Example:**
```paradox
cic_to_target_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("mic_to_target_factor", HOI4Entity {
        name: "mic_to_target_factor",
        description: r#"Gives a portion of the country's military industry to the specified target.

**Example:**
```paradox
mic_to_target_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "trade_cost_for_target_factor",
        HOI4Entity {
            name: "trade_cost_for_target_factor",
            description: r#"The cost for the targeted country to purchase this country's resources.

**Example:**
```paradox
trade_cost_for_target_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "targeted_legitimacy_daily",
        HOI4Entity {
            name: "targeted_legitimacy_daily",
            description: r#"Changes daily gain of legitimacy of the target country.

**Example:**
```paradox
targeted_legitimacy_daily = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "attack_bonus_against",
        HOI4Entity {
            name: "attack_bonus_against",
            description: r#"Gives an attack bonus against the armies of the specified country.

**Example:**
```paradox
attack_bonus_against = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("attack_bonus_against_cores", HOI4Entity {
        name: "attack_bonus_against_cores",
        description: r#"Gives an attack bonus against the armies of the specified country on its core territory.

**Example:**
```paradox
attack_bonus_against_cores = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "breakthrough_bonus_against",
        HOI4Entity {
            name: "breakthrough_bonus_against",
            description: r#"Gives a breakthrough bonus against the armies of the specified country.

**Example:**
```paradox
breakthrough_bonus_against = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "defense_bonus_against",
        HOI4Entity {
            name: "defense_bonus_against",
            description: r#"Gives a defense bonus against the armies of the specified country.

**Example:**
```paradox
defense_bonus_against = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "army_speed_factor_for_controller",
        HOI4Entity {
            name: "army_speed_factor_for_controller",
            description: r#"Changes the division speed for the controller of the state.

**Example:**
```paradox
army_speed_factor_for_controller = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "attrition_for_controller",
        HOI4Entity {
            name: "attrition_for_controller",
            description: r#"Changes the attrition for the controller of the state.

**Example:**
```paradox
attrition_for_controller = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "equipment_capture_for_controller",
        HOI4Entity {
            name: "equipment_capture_for_controller",
            description: r#"Changes the equipment capture ratio by the state's controller.

**Example:**
```paradox
equipment_capture_for_controller = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "equipment_capture_factor_for_controller",
        HOI4Entity {
            name: "equipment_capture_factor_for_controller",
            description: r#"Modifies the equipment capture ratio by the state's controller.

**Example:**
```paradox
equipment_capture_factor_for_controller = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "enemy_army_speed_factor",
        HOI4Entity {
            name: "enemy_army_speed_factor",
            description: r#"Modifies the speed of divisions at war with the state's owner.

**Example:**
```paradox
enemy_army_speed_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "enemy_local_supplies",
        HOI4Entity {
            name: "enemy_local_supplies",
            description: r#"Modifies the supply of divisions at war with the state's owner.

**Example:**
```paradox
enemy_local_supplies = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "enemy_attrition",
        HOI4Entity {
            name: "enemy_attrition",
            description: r#"Modifies the attrition of divisions at war with the state's owner.

**Example:**
```paradox
enemy_attrition = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "enemy_truck_attrition_factor",
        HOI4Entity {
            name: "enemy_truck_attrition_factor",
            description: r#"Modifies the truck attrition of divisions at war with the state's owner.

**Example:**
```paradox
enemy_truck_attrition_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "compliance_gain",
        HOI4Entity {
            name: "compliance_gain",
            description: r#"Changes the compliance gain in the current state.

**Example:**
```paradox
compliance_gain = 0.01
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "compliance_growth",
        HOI4Entity {
            name: "compliance_growth",
            description: r#"Changes the compliance growth speed in the current state.

**Example:**
```paradox
compliance_growth = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "disable_strategic_redeployment",
        HOI4Entity {
            name: "disable_strategic_redeployment",
            description: r#"Disables strategic redeployment in the state.

**Example:**
```paradox
disable_strategic_redeployment = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "disable_strategic_redeployment_for_controller",
        HOI4Entity {
            name: "disable_strategic_redeployment_for_controller",
            description: r#"Disables strategic redeployment in the state for the controller.

**Example:**
```paradox
disable_strategic_redeployment_for_controller = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "enemy_intel_network_gain_factor_over_occupied_tag",
        HOI4Entity {
            name: "enemy_intel_network_gain_factor_over_occupied_tag",
            description: r#"Modifies enemy intel network strength gain.

**Example:**
```paradox
enemy_intel_network_gain_factor_over_occupied_tag = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "local_building_slots",
        HOI4Entity {
            name: "local_building_slots",
            description: r#"Modifies amount of building slots.

**Example:**
```paradox
local_building_slots = 2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "local_building_slots_factor",
        HOI4Entity {
            name: "local_building_slots_factor",
            description: r#"Modifies amount of building slots by a percentage.

**Example:**
```paradox
local_building_slots_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "local_factories",
        HOI4Entity {
            name: "local_factories",
            description: r#"Modifies amount of available factories in the state.

**Example:**
```paradox
local_factories = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "local_factory_energy_consumption",
        HOI4Entity {
            name: "local_factory_energy_consumption",
            description: r#"Modifies amount of energy consumed by factories in the state.

**Example:**
```paradox
local_factory_energy_consumption = 0.2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("local_factory_energy_consumption_per_infrastructure", HOI4Entity {
        name: "local_factory_energy_consumption_per_infrastructure",
        description: r#"Modifies amount of energy consumed by factories depending on the infrastructure of the state.

**Example:**
```paradox
local_factory_energy_consumption_per_infrastructure = 0.2
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "local_factory_sabotage",
        HOI4Entity {
            name: "local_factory_sabotage",
            description: r#"Modifies chance for factory sabotage.

**Example:**
```paradox
local_factory_sabotage = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "local_intel_to_enemies",
        HOI4Entity {
            name: "local_intel_to_enemies",
            description: r#"Modifies amount of intel to enemies.

**Example:**
```paradox
local_intel_to_enemies = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "local_manpower",
        HOI4Entity {
            name: "local_manpower",
            description: r#"Modifies amount of available manpower.

**Example:**
```paradox
local_manpower = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "local_non_core_manpower",
        HOI4Entity {
            name: "local_non_core_manpower",
            description: r#"Modifies amount of available non-core manpower.

**Example:**
```paradox
local_non_core_manpower = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "local_org_regain",
        HOI4Entity {
            name: "local_org_regain",
            description: r#"Modifies how much organisation is regained after combat.

**Example:**
```paradox
local_org_regain = -0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("local_resource_gain_efficiency_per_infrastructure", HOI4Entity {
        name: "local_resource_gain_efficiency_per_infrastructure",
        description: r#"Modifies amount of available resources gained depending on the infrastructure of the state.

**Example:**
```paradox
local_resource_gain_efficiency_per_infrastructure = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "local_resources",
        HOI4Entity {
            name: "local_resources",
            description: r#"Modifies amount of available resources.

**Example:**
```paradox
local_resources = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "local_supplies",
        HOI4Entity {
            name: "local_supplies",
            description: r#"Modifies amount of available supplies.

**Example:**
```paradox
local_supplies = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "local_supplies_for_controller",
        HOI4Entity {
            name: "local_supplies_for_controller",
            description: r#"Modifies amount of available supplies for the controller.

**Example:**
```paradox
local_supplies_for_controller = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "local_supply_impact_factor",
        HOI4Entity {
            name: "local_supply_impact_factor",
            description: r#"Modifies the impact that the state's local supplies have.

**Example:**
```paradox
local_supply_impact_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("local_non_core_supply_impact_factor", HOI4Entity {
        name: "local_non_core_supply_impact_factor",
        description: r#"Modifies the impact that the state's local supplies have if the state is not cored by the controller of provinces within.

**Example:**
```paradox
local_non_core_supply_impact_factor = 0.3
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "mobilization_speed",
        HOI4Entity {
            name: "mobilization_speed",
            description: r#"Modifies the mobilisation speed.

**Example:**
```paradox
mobilization_speed = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "non_core_manpower",
        HOI4Entity {
            name: "non_core_manpower",
            description: r#"Modifies the amount of recruited non-core manpower.

**Example:**
```paradox
non_core_manpower = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("max_fuel_building", HOI4Entity {
        name: "max_fuel_building",
        description: r#"Modifies the amount of fuel capacity, in thousands, given to the state controller from the building.

**Example:**
```paradox
max_fuel_building = 1500
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "recruitable_population",
        HOI4Entity {
            name: "recruitable_population",
            description: r#"Modifies the amount of recruited manpower.

**Example:**
```paradox
recruitable_population = 0.03
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "recruitable_population_factor",
        HOI4Entity {
            name: "recruitable_population_factor",
            description: r#"Modifies the amount of recruited manpower by a percentage.

**Example:**
```paradox
recruitable_population_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "resistance_damage_to_garrison",
        HOI4Entity {
            name: "resistance_damage_to_garrison",
            description: r#"Modifies the amount of resistance damage to the garrison.

**Example:**
```paradox
resistance_damage_to_garrison = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "resistance_decay",
        HOI4Entity {
            name: "resistance_decay",
            description: r#"Modifies the speed of resistance decay.

**Example:**
```paradox
resistance_decay = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "resistance_garrison_penetration_chance",
        HOI4Entity {
            name: "resistance_garrison_penetration_chance",
            description: r#"Modifies the chance for the garrison to be penetrated.

**Example:**
```paradox
resistance_garrison_penetration_chance = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "resistance_growth",
        HOI4Entity {
            name: "resistance_growth",
            description: r#"Modifies the speed of the resistance growth.

**Example:**
```paradox
resistance_growth = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "starting_compliance",
        HOI4Entity {
            name: "starting_compliance",
            description: r#"Modifies the base compliance value.

**Example:**
```paradox
starting_compliance = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "state_bunker_max_level_terrain_limit",
        HOI4Entity {
            name: "state_bunker_max_level_terrain_limit",
            description: r#"Modifies the amount of available bunker building slots

in the state.

**Example:**
```paradox
state_bunker_max_level_terrain_limit = 6
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "state_coastal_bunker_max_level_terrain_limit",
        HOI4Entity {
            name: "state_coastal_bunker_max_level_terrain_limit",
            description: r#"Modifies the amount of available coastal bunker building slots

in the state.

**Example:**
```paradox
state_coastal_bunker_max_level_terrain_limit = 6
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "state_production_speed__factor",
        HOI4Entity {
            name: "state_production_speed__factor",
            description: r#"Modifies the building speed of the specified building in the state.

**Example:**
```paradox
state_production_speed_industrial_complex_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "state_repair_speed__factor",
        HOI4Entity {
            name: "state_repair_speed__factor",
            description: r#"Modifies the repair speed of the specified building in the state.

**Example:**
```paradox
state_repair_speed_industrial_complex_factor = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "state_resource",
        HOI4Entity {
            name: "state_resource",
            description: r#"Modifies the amount of the specified resource in the state.

**Example:**
```paradox
state_resource_oil = 5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "state_resources_factor",
        HOI4Entity {
            name: "state_resources_factor",
            description: r#"Modifies the amount of resources in a state.

**Example:**
```paradox
state_resources_factor = 0.2
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "state_resource_cost",
        HOI4Entity {
            name: "state_resource_cost",
            description: r#"Modifies the amount of the specified resource in the state.

**Example:**
```paradox
state_resource_cost_rubber = 5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("temporary_state_resource", HOI4Entity {
        name: "temporary_state_resource",
        description: r#"Modifies the amount of the specified resource in the state as an added modifier after the base one.

**Example:**
```paradox
temporary_state_resource_tungsten = 5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("enemy_operative_detection_chance_over_occupied_tag", HOI4Entity {
        name: "enemy_operative_detection_chance_over_occupied_tag",
        description: r#"Offsets the chance for an enemy operative to be detected for the tag that occupies this state.

**Example:**
```paradox
enemy_operative_detection_chance_over_occupied_tag = 5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("enemy_operative_detection_chance_factor_over_occupied_tag", HOI4Entity {
        name: "enemy_operative_detection_chance_factor_over_occupied_tag",
        description: r#"Modifies the chance for an enemy operative to be detected for the tag that occupies this state.

**Example:**
```paradox
enemy_operative_detection_chance_factor_over_occupied_tag = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "cannot_use_abilities",
        HOI4Entity {
            name: "cannot_use_abilities",
            description: r#"Disables using abilities.

**Example:**
```paradox
cannot_use_abilities = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "dont_lose_dig_in_on_attack",
        HOI4Entity {
            name: "dont_lose_dig_in_on_attack",
            description: r#"Disables losing the entrechment bonus during attack.

**Example:**
```paradox
dont_lose_dig_in_on_attack = 1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("exiled_divisions_attack_factor", HOI4Entity {
        name: "exiled_divisions_attack_factor",
        description: r#"Modifies the attack of divisions led by this unit leader if they're exiled.

**Example:**
```paradox
exiled_divisions_attack_factor = 0.4
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("exiled_divisions_defense_factor", HOI4Entity {
        name: "exiled_divisions_defense_factor",
        description: r#"Modifies the defence of divisions led by this unit leader if they're exiled.

**Example:**
```paradox
exiled_divisions_defense_factor = 0.4
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("own_exiled_divisions_attack_factor", HOI4Entity {
        name: "own_exiled_divisions_attack_factor",
        description: r#"Modifies the attack of divisions led by this unit leader if they're exiled and belong to the same country.

**Example:**
```paradox
own_exiled_divisions_attack_factor = 0.4
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("own_exiled_divisions_defense_factor", HOI4Entity {
        name: "own_exiled_divisions_defense_factor",
        description: r#"Modifies the defence of divisions led by this unit leader if they're exiled and belong to the same country.

**Example:**
```paradox
own_exiled_divisions_defense_factor = 0.4
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "experience_gain_factor",
        HOI4Entity {
            name: "experience_gain_factor",
            description: r#"Modifies the experience gained by the unit leader.

**Example:**
```paradox
experience_gain_factor = 0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "fortification_collateral_chance",
        HOI4Entity {
            name: "fortification_collateral_chance",
            description: r#"Chance for combat to damage enemy forts.

**Example:**
```paradox
fortification_collateral_chance = 0.4
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "fortification_damage",
        HOI4Entity {
            name: "fortification_damage",
            description: r#"Damage enemy forts receive from combat.

**Example:**
```paradox
fortification_damage = 0.4
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("max_commander_army_size", HOI4Entity {
        name: "max_commander_army_size",
        description: r#"Modifies amount of divisions that can be led by the army leader without penalty.

**Example:**
```paradox
max_commander_army_size = 12
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert("max_army_group_size", HOI4Entity {
        name: "max_army_group_size",
        description: r#"Modifies amount of army groups that can be led by the field marshal without penalty.

**Example:**
```paradox
max_army_group_size = 1
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "paradrop_organization_factor",
        HOI4Entity {
            name: "paradrop_organization_factor",
            description: r#"The amount of organisation paratroopers will have after paradropping.

**Example:**
```paradox
paradrop_organization_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "paratrooper_aa_defense",
        HOI4Entity {
            name: "paratrooper_aa_defense",
            description: r#"The strength of anti-air against paratroopers.

**Example:**
```paradox
paratrooper_aa_defense = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "paratrooper_weight_factor",
        HOI4Entity {
            name: "paratrooper_weight_factor",
            description: r#"Paratrooper transport space factor.

**Example:**
```paradox
paratrooper_weight_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "promote_cost_factor",
        HOI4Entity {
            name: "promote_cost_factor",
            description: r#"The cost to promote the unit leader.

**Example:**
```paradox
promote_cost_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "reassignment_duration_factor",
        HOI4Entity {
            name: "reassignment_duration_factor",
            description: r#"The length of the reassignment penalty.

**Example:**
```paradox
reassignment_duration_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "river_crossing_factor",
        HOI4Entity {
            name: "river_crossing_factor",
            description: r#"The effects of the river crossing penalty.

**Example:**
```paradox
river_crossing_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "sickness_chance",
        HOI4Entity {
            name: "sickness_chance",
            description: r#"The chance for the unit leader to get sick.

**Example:**
```paradox
sickness_chance = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "skill_bonus_factor",
        HOI4Entity {
            name: "skill_bonus_factor",
            description: r#"The bonus the unit leader receives from their skillset.

**Example:**
```paradox
skill_bonus_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "trait__xp_gain_factor",
        HOI4Entity {
            name: "trait__xp_gain_factor",
            description: r#"Modifies the experience gain towards the specified trait.

**Example:**
```paradox
trait_infantry_leader_xp_gain_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert("terrain_trait_xp_gain_factor", HOI4Entity {
        name: "terrain_trait_xp_gain_factor",
        description: r#"Modifies the experience gain towards all terrain traits (With the type of either basic_terrain_trait or assignable_terrain_trait).

**Example:**
```paradox
terrain_trait_xp_gain_factor = 0.5
```"#,
        scopes: &[crate::scope::Scope::Unknown],
    });
    m.insert(
        "wounded_chance_factor",
        HOI4Entity {
            name: "wounded_chance_factor",
            description: r#"The chance for the unit leader to get wounded.

**Example:**
```paradox
wounded_chance_factor = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "shore_bombardment_bonus",
        HOI4Entity {
            name: "shore_bombardment_bonus",
            description: r#"Modifies the penalty given by the shore bombardment on divisions.

**Example:**
```paradox
shore_bombardment_bonus = 0.5
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "female_random_scientist_chance",
        HOI4Entity {
            name: "female_random_scientist_chance",
            description: r#"The chance of spawn female scientist

**Example:**
```paradox
female_random_scientist_chance = 0.05
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "scientist_breakthrough_bonus_factor",
        HOI4Entity {
            name: "scientist_breakthrough_bonus_factor",
            description: r#"Modifiers scientist breakthrough bonus for special projects

**Example:**
```paradox
scientist_breakthrough_bonus_factor = -0.25
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "scientist_research_bonus_factor",
        HOI4Entity {
            name: "scientist_research_bonus_factor",
            description: r#"Modifiers scientist research bonus for special projects

**Example:**
```paradox
scientist_research_bonus_factor = 0.15
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "scientist_xp_gain_factor",
        HOI4Entity {
            name: "scientist_xp_gain_factor",
            description: r#"Modifiers scientist gain xp

**Example:**
```paradox
scientist_xp_gain_factor = 0.02
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_accidents",
        HOI4Entity {
            name: "air_accidents",
            description: r#"Base chance for an air accident to happen.

**Example:**
```paradox
air_accidents = 0.3
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "air_detection",
        HOI4Entity {
            name: "air_detection",
            description: r#"Base chance for air detection.

**Example:**
```paradox
air_detection = -0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "naval_strike",
        HOI4Entity {
            name: "naval_strike",
            description: r#"Base efficiency for naval strikes.

**Example:**
```paradox
naval_strike = -0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_casualty_on_sink",
        HOI4Entity {
            name: "navy_casualty_on_sink",
            description: r#"Modifies the casualties when ships are sunk in this region.

**Example:**
```paradox
navy_casualty_on_sink = -0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m.insert(
        "navy_casualty_on_hit",
        HOI4Entity {
            name: "navy_casualty_on_hit",
            description: r#"Modifies the casualties when ships are damaged in this region.

**Example:**
```paradox
navy_casualty_on_hit = -0.1
```"#,
            scopes: &[crate::scope::Scope::Unknown],
        },
    );
    m
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
