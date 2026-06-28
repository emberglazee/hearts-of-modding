#![allow(dead_code)]

use crate::ScannerData;
use crate::data::interner::InternedStr;
use crate::for_each_standard_scanner;
use crate::parser::ast;
use crate::utils::lsp_convert::is_pos_in_range;
use std::collections::HashMap;
use tower_lsp_server::ls_types::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityKind {
    ScriptedTrigger,
    ScriptedEffect,
    ScriptedLoc,
    Ideology,
    SubIdeology,
    Trait,
    Sprite,
    Idea,
    Character,
    Event,
    Focus,
    Ability,
    Achievement,
    AiArea,
    Variable,
    EventTarget,
    CustomModifier,
    MusicAsset,
    MusicStation,
    Song,
    Sound,
    SoundEffect,
    Falloff,
    SoundCategory,
    AdjacencyRule,
    BalanceOfPower,
    EventNamespace,
    StrategicRegion,
    TerrainCategory,
    Portrait,
    Building,
    AiStrategyPlan,
    Province,
    State,
    SupplyNode,
    Railway,
    MapBuilding,
    UnitStack,
    WeatherPosition,
    Adjacency,
    Localization,
    ModifierMapping,
    CountryTag,
    ColorCode,
    Decision,
    Resource,
    StateCategory,
    OobDivisionTemplate,
    OobFleet,
    UnitType,
}

impl EntityKind {
    pub fn symbol_kind(&self) -> tower_lsp_server::ls_types::SymbolKind {
        use tower_lsp_server::ls_types::SymbolKind;
        match self {
            EntityKind::ScriptedTrigger | EntityKind::ScriptedEffect | EntityKind::ScriptedLoc => {
                SymbolKind::FUNCTION
            }
            EntityKind::Ideology | EntityKind::SubIdeology => SymbolKind::ENUM,
            EntityKind::Trait => SymbolKind::STRUCT,
            EntityKind::Sprite => SymbolKind::OBJECT,
            EntityKind::Idea => SymbolKind::CLASS,
            EntityKind::Character => SymbolKind::STRUCT,
            EntityKind::Event => SymbolKind::EVENT,
            EntityKind::Focus => SymbolKind::EVENT,
            EntityKind::Ability => SymbolKind::METHOD,
            EntityKind::Achievement => SymbolKind::EVENT,
            EntityKind::Variable => SymbolKind::VARIABLE,
            EntityKind::EventTarget => SymbolKind::VARIABLE,
            EntityKind::CustomModifier => SymbolKind::PROPERTY,
            EntityKind::MusicAsset | EntityKind::MusicStation | EntityKind::Song => {
                SymbolKind::PROPERTY
            }
            EntityKind::Sound
            | EntityKind::SoundEffect
            | EntityKind::Falloff
            | EntityKind::SoundCategory => SymbolKind::PROPERTY,
            EntityKind::AdjacencyRule => SymbolKind::FUNCTION,
            EntityKind::EventNamespace => SymbolKind::NAMESPACE,
            EntityKind::StrategicRegion => SymbolKind::OBJECT,
            EntityKind::TerrainCategory => SymbolKind::ENUM,
            EntityKind::Portrait => SymbolKind::OBJECT,
            EntityKind::Building => SymbolKind::OBJECT,
            EntityKind::AiArea => SymbolKind::CLASS,
            EntityKind::AiStrategyPlan => SymbolKind::CLASS,
            EntityKind::BalanceOfPower => SymbolKind::CLASS,
            EntityKind::Province => SymbolKind::NUMBER,
            EntityKind::State => SymbolKind::OBJECT,
            EntityKind::SupplyNode | EntityKind::Railway => SymbolKind::OBJECT,
            EntityKind::MapBuilding | EntityKind::UnitStack | EntityKind::WeatherPosition => {
                SymbolKind::OBJECT
            }
            EntityKind::Adjacency => SymbolKind::OBJECT,
            EntityKind::Localization => SymbolKind::STRING,
            EntityKind::ModifierMapping => SymbolKind::PROPERTY,
            EntityKind::CountryTag => SymbolKind::MODULE,
            EntityKind::ColorCode => SymbolKind::CONSTANT,
            EntityKind::Decision => SymbolKind::EVENT,
            EntityKind::Resource => SymbolKind::PROPERTY,
            EntityKind::StateCategory => SymbolKind::ENUM,
            EntityKind::OobDivisionTemplate => SymbolKind::STRUCT,
            EntityKind::OobFleet => SymbolKind::OBJECT,
            EntityKind::UnitType => SymbolKind::CLASS,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EntityLocation {
    pub kind: EntityKind,
    pub range: ast::Range,
    pub path: InternedStr,
}

#[derive(Debug, Clone)]
pub struct EntityHit {
    pub name: String,
    pub kind: EntityKind,
    pub container: Option<String>,
    pub location: EntityLocation,
}

pub struct EntityLookup<'a> {
    data: &'a ScannerData,
}

impl<'a> EntityLookup<'a> {
    pub fn new(data: &'a ScannerData) -> Self {
        EntityLookup { data }
    }

    pub fn find_definition(&self, key: &str) -> Vec<EntityLocation> {
        let mut results = Vec::new();

        macro_rules! try_lookup {
            ($kind:ident, $name:ident) => {
                if let Some(entity) = self.data.$name.get(key) {
                    results.push(EntityLocation {
                        kind: EntityKind::$kind,
                        range: entity.range.clone(),
                        path: entity.path.clone(),
                    });
                }
            };
        }

        // Standard scanners (generated via registry)
        macro_rules! std_lookup_get {
            ($mod:ident, $ty:ident, $kind:ident, $field:ident, $dir:expr, $ext:expr) => {
                if let Some(entity) = self.data.$field.get(key) {
                    results.push(EntityLocation {
                        kind: EntityKind::$kind,
                        range: entity.range.clone(),
                        path: entity.path.clone(),
                    });
                }
            };
        }
        for_each_standard_scanner!(std_lookup_get);

        // Special scanners (manual)
        try_lookup!(ScriptedTrigger, scripted_triggers);
        try_lookup!(ScriptedEffect, scripted_effects);
        try_lookup!(ScriptedLoc, scripted_locs);
        try_lookup!(Ideology, ideologies);

        {
            let map = &self.data.sub_ideologies;
            if let Some(entry) = map.get(key) {
                results.push(EntityLocation {
                    kind: EntityKind::SubIdeology,
                    range: entry.1.clone(),
                    path: entry.2.clone(),
                });
            }
        }

        try_lookup!(Trait, traits);
        try_lookup!(Event, events);

        {
            let map = &self.data.variables;
            if let Some(vars) = map.get(key) {
                for var in vars.iter() {
                    results.push(EntityLocation {
                        kind: EntityKind::Variable,
                        range: var.range.clone(),
                        path: var.path.clone(),
                    });
                }
            }
        }

        {
            let map = &self.data.event_targets;
            if let Some(targets) = map.get(key) {
                for target in targets.iter() {
                    results.push(EntityLocation {
                        kind: EntityKind::EventTarget,
                        range: target.range.clone(),
                        path: target.path.clone(),
                    });
                }
            }
        }

        try_lookup!(CustomModifier, custom_modifiers);
        try_lookup!(MusicAsset, music_assets);
        try_lookup!(MusicStation, music_stations);
        try_lookup!(Song, songs);
        try_lookup!(Sound, sounds);
        try_lookup!(SoundEffect, sound_effects);
        try_lookup!(Falloff, falloffs);
        try_lookup!(SoundCategory, sound_categories);
        try_lookup!(AdjacencyRule, adjacency_rules);

        if let Ok(id) = key.parse::<u32>() {
            let map = &self.data.strategic_regions;
            if let Some(region) = map.get(&id) {
                results.push(EntityLocation {
                    kind: EntityKind::StrategicRegion,
                    range: region.range.clone(),
                    path: region.path.clone(),
                });
            }
        }

        try_lookup!(CountryTag, country_tags);
        try_lookup!(OobDivisionTemplate, oob_division_templates);
        try_lookup!(OobFleet, oob_fleets);
        try_lookup!(EventNamespace, event_namespaces);

        {
            let map = &self.data.modifier_mappings;
            if let Some(loc_key) = map.get(key) {
                let loc = &self.data.localization;
                if let Some(entry) = loc.get(loc_key.as_str()) {
                    results.push(EntityLocation {
                        kind: EntityKind::ModifierMapping,
                        range: entry.range.clone(),
                        path: entry.path.clone(),
                    });
                }
            }
        }

        {
            let loc = &self.data.localization;
            if let Some(entry) = loc.get(key) {
                results.push(EntityLocation {
                    kind: EntityKind::Localization,
                    range: entry.range.clone(),
                    path: entry.path.clone(),
                });
            }
            let prefix = format!("{}:", key);
            for entry in loc.iter() {
                let k = entry.key();
                if k.starts_with(&prefix) {
                    results.push(EntityLocation {
                        kind: EntityKind::Localization,
                        range: entry.value().range.clone(),
                        path: entry.value().path.clone(),
                    });
                }
            }
        }

        results
    }

    pub fn entity_at(&self, path: &str, pos: Position) -> Option<(EntityKind, ast::Range, String)> {
        macro_rules! check_entity {
            ($kind:ident, $name:ident) => {
                for entry in self.data.$name.iter() {
                    let entity = entry.value();
                    let name = entry.key();
                    if entity.path.as_ref() == path && is_pos_in_range(pos, &entity.range) {
                        return Some((EntityKind::$kind, entity.range.clone(), name.to_string()));
                    }
                }
            };
        }

        // Standard scanners (generated)
        macro_rules! std_check_entity {
            ($mod:ident, $ty:ident, $kind:ident, $field:ident, $dir:expr, $ext:expr) => {
                for entry in self.data.$field.iter() {
                    let entity = entry.value();
                    let name = entry.key();
                    if entity.path.as_ref() == path && is_pos_in_range(pos, &entity.range) {
                        return Some((EntityKind::$kind, entity.range.clone(), name.to_string()));
                    }
                }
            };
        }
        for_each_standard_scanner!(std_check_entity);

        // Special scanners (manual)
        check_entity!(Event, events);

        {
            let map = &self.data.variables;
            for entry in map.iter() {
                let name = entry.key();
                for var in entry.value().iter() {
                    if var.path.as_ref() == path && is_pos_in_range(pos, &var.range) {
                        return Some((EntityKind::Variable, var.range.clone(), name.to_string()));
                    }
                }
            }
        }

        None
    }

    pub fn entity_names(&self) -> HashMap<String, EntityKind> {
        let mut names = HashMap::new();

        macro_rules! collect_names {
            ($kind:ident, $name:ident) => {
                for entry in self.data.$name.iter() {
                    names.insert(entry.key().to_string(), EntityKind::$kind);
                }
            };
        }

        // Standard scanners (generated)
        macro_rules! std_collect_names {
            ($mod:ident, $ty:ident, $kind:ident, $field:ident, $dir:expr, $ext:expr) => {
                for entry in self.data.$field.iter() {
                    names.insert(entry.key().to_string(), EntityKind::$kind);
                }
            };
        }
        for_each_standard_scanner!(std_collect_names);

        // Special scanners (manual)

        {
            let map = &self.data.sub_ideologies;
            for entry in map.iter() {
                names.insert(entry.key().to_string(), EntityKind::SubIdeology);
            }
        }

        collect_names!(Event, events);
        collect_names!(ScriptedTrigger, scripted_triggers);
        collect_names!(ScriptedEffect, scripted_effects);
        collect_names!(ScriptedLoc, scripted_locs);
        collect_names!(Ideology, ideologies);
        collect_names!(Trait, traits);
        collect_names!(CustomModifier, custom_modifiers);
        collect_names!(MusicAsset, music_assets);
        collect_names!(MusicStation, music_stations);
        collect_names!(Song, songs);
        collect_names!(Sound, sounds);
        collect_names!(SoundEffect, sound_effects);
        collect_names!(Falloff, falloffs);
        collect_names!(SoundCategory, sound_categories);
        collect_names!(AdjacencyRule, adjacency_rules);
        collect_names!(CountryTag, country_tags);
        collect_names!(OobDivisionTemplate, oob_division_templates);
        collect_names!(OobFleet, oob_fleets);
        collect_names!(EventNamespace, event_namespaces);

        names
    }

    pub fn find_symbols(&self, query: &str) -> Vec<EntityHit> {
        let query_lower = query.to_ascii_lowercase();
        let mut results = Vec::new();

        let fuzzy_match =
            |query: &str, target: &str| crate::utils::fs_util::fuzzy_match(query, target);

        macro_rules! push_symbols {
            ($kind:ident, $name:ident, $container:expr) => {
                for entry in self.data.$name.iter() {
                    let name = entry.key();
                    let entity = entry.value();
                    if fuzzy_match(&query_lower, name) {
                        results.push(EntityHit {
                            name: name.to_string(),
                            kind: EntityKind::$kind,
                            container: Some($container.to_string()),
                            location: EntityLocation {
                                kind: EntityKind::$kind,
                                range: entity.range.clone(),
                                path: entity.path.clone(),
                            },
                        });
                    }
                }
            };
        }

        push_symbols!(CustomModifier, custom_modifiers, "Modifier");
        push_symbols!(Achievement, achievements, "Achievement");
        push_symbols!(Focus, focuses, "National Focus");

        {
            let map = &self.data.events;
            for entry in map.iter() {
                let id = entry.key();
                let event = entry.value();
                if fuzzy_match(&query_lower, id) {
                    results.push(EntityHit {
                        name: id.to_string(),
                        kind: EntityKind::Event,
                        container: Some(event.event_type.clone()),
                        location: EntityLocation {
                            kind: EntityKind::Event,
                            range: event.range.clone(),
                            path: event.path.clone(),
                        },
                    });
                }
            }
        }

        {
            let map = &self.data.ideas;
            for entry in map.iter() {
                let name = entry.key();
                let idea = entry.value();
                if fuzzy_match(&query_lower, name) {
                    results.push(EntityHit {
                        name: name.to_string(),
                        kind: EntityKind::Idea,
                        container: Some(idea.category.clone()),
                        location: EntityLocation {
                            kind: EntityKind::Idea,
                            range: idea.range.clone(),
                            path: idea.path.clone(),
                        },
                    });
                }
            }
        }

        {
            let map = &self.data.traits;
            for entry in map.iter() {
                let name = entry.key();
                let entity = entry.value();
                if fuzzy_match(&query_lower, name) {
                    results.push(EntityHit {
                        name: name.to_string(),
                        kind: EntityKind::Trait,
                        container: Some(entity.trait_type.clone()),
                        location: EntityLocation {
                            kind: EntityKind::Trait,
                            range: entity.range.clone(),
                            path: entity.path.clone(),
                        },
                    });
                }
            }
        }

        push_symbols!(ScriptedTrigger, scripted_triggers, "Scripted Trigger");
        push_symbols!(ScriptedEffect, scripted_effects, "Scripted Effect");
        push_symbols!(ScriptedLoc, scripted_locs, "Scripted Localisation");

        {
            let map = &self.data.states;
            for entry in map.iter() {
                let id = entry.key();
                let state = entry.value();
                let display = format!("State {}: {}", id, state.name);
                if fuzzy_match(&query_lower, &id.to_string())
                    || fuzzy_match(&query_lower, &state.name)
                {
                    results.push(EntityHit {
                        name: display,
                        kind: EntityKind::State,
                        container: Some("State".to_string()),
                        location: EntityLocation {
                            kind: EntityKind::State,
                            range: state.range.clone(),
                            path: state.path.clone(),
                        },
                    });
                }
            }
        }

        {
            let map = self.data.supply_nodes();
            for node in map.iter() {
                let display = format!("Supply Node in Province {}", node.province_id);
                if fuzzy_match(&query_lower, &node.province_id.to_string()) {
                    results.push(EntityHit {
                        name: display,
                        kind: EntityKind::SupplyNode,
                        container: Some("Supply Node".to_string()),
                        location: EntityLocation {
                            kind: EntityKind::SupplyNode,
                            range: ast::Range {
                                start_line: node.start_line,
                                start_col: 0,
                                end_line: node.start_line,
                                end_col: 100,
                            },
                            path: node.path.clone(),
                        },
                    });
                }
            }
        }

        {
            let map = self.data.railways();
            for rw in map.iter() {
                if fuzzy_match(&query_lower, "railway") {
                    results.push(EntityHit {
                        name: format!("Railway (Lvl {})", rw.level),
                        kind: EntityKind::Railway,
                        container: Some("Railway".to_string()),
                        location: EntityLocation {
                            kind: EntityKind::Railway,
                            range: ast::Range {
                                start_line: rw.start_line,
                                start_col: 0,
                                end_line: rw.start_line,
                                end_col: 100,
                            },
                            path: rw.path.clone(),
                        },
                    });
                }
            }
        }

        {
            let map = self.data.map_buildings();
            for mb in map.iter() {
                let display = format!("Building '{}' in State {}", mb.building_id, mb.state_id);
                if fuzzy_match(&query_lower, &mb.building_id)
                    || fuzzy_match(&query_lower, &mb.state_id.to_string())
                {
                    results.push(EntityHit {
                        name: display,
                        kind: EntityKind::MapBuilding,
                        container: Some("Map Building".to_string()),
                        location: EntityLocation {
                            kind: EntityKind::MapBuilding,
                            range: ast::Range {
                                start_line: mb.start_line,
                                start_col: 0,
                                end_line: mb.start_line,
                                end_col: 100,
                            },
                            path: mb.path.clone(),
                        },
                    });
                }
            }
        }

        {
            let map = self.data.unitstacks();
            for us in map.iter() {
                let display = format!("Unitstack {} in Province {}", us.stack_type, us.province_id);
                if fuzzy_match(&query_lower, &us.province_id.to_string()) {
                    results.push(EntityHit {
                        name: display,
                        kind: EntityKind::UnitStack,
                        container: Some("Unitstack".to_string()),
                        location: EntityLocation {
                            kind: EntityKind::UnitStack,
                            range: ast::Range {
                                start_line: us.start_line,
                                start_col: 0,
                                end_line: us.start_line,
                                end_col: 100,
                            },
                            path: us.path.clone(),
                        },
                    });
                }
            }
        }

        {
            let map = self.data.weather_positions();
            for wp in map.iter() {
                let display = format!("Weather Position in Strategic Region {}", wp.region_id);
                if fuzzy_match(&query_lower, &wp.region_id.to_string()) {
                    results.push(EntityHit {
                        name: display,
                        kind: EntityKind::WeatherPosition,
                        container: Some("Weather Position".to_string()),
                        location: EntityLocation {
                            kind: EntityKind::WeatherPosition,
                            range: ast::Range {
                                start_line: wp.start_line,
                                start_col: 0,
                                end_line: wp.start_line,
                                end_col: 100,
                            },
                            path: wp.path.clone(),
                        },
                    });
                }
            }
        }

        {
            let map = self.data.adjacencies();
            for adj in map.iter() {
                let display = format!(
                    "Adjacency ({}) {} <-> {}",
                    adj.adj_type, adj.start_prov, adj.end_prov
                );
                if fuzzy_match(&query_lower, &adj.start_prov.to_string())
                    || fuzzy_match(&query_lower, &adj.end_prov.to_string())
                {
                    results.push(EntityHit {
                        name: display,
                        kind: EntityKind::Adjacency,
                        container: Some("Adjacency".to_string()),
                        location: EntityLocation {
                            kind: EntityKind::Adjacency,
                            range: ast::Range {
                                start_line: adj.start_line,
                                start_col: 0,
                                end_line: adj.start_line,
                                end_col: 100,
                            },
                            path: adj.path.clone(),
                        },
                    });
                }
            }
        }

        push_symbols!(AdjacencyRule, adjacency_rules, "Adjacency Rule");

        {
            let map = &self.data.strategic_regions;
            for entry in map.iter() {
                let id = entry.key();
                let region = entry.value();
                let display = format!("Strategic Region {}: {}", id, region.name);
                if fuzzy_match(&query_lower, &id.to_string())
                    || fuzzy_match(&query_lower, &region.name)
                {
                    results.push(EntityHit {
                        name: display,
                        kind: EntityKind::StrategicRegion,
                        container: Some("Strategic Region".to_string()),
                        location: EntityLocation {
                            kind: EntityKind::StrategicRegion,
                            range: region.range.clone(),
                            path: region.path.clone(),
                        },
                    });
                }
            }
        }

        {
            let map = &self.data.localization;
            let mut count = 0;
            for entry in map.iter() {
                let name = entry.key();
                let loc = entry.value();
                if fuzzy_match(&query_lower, name) {
                    results.push(EntityHit {
                        name: name.to_string(),
                        kind: EntityKind::Localization,
                        container: Some("Localisation".to_string()),
                        location: EntityLocation {
                            kind: EntityKind::Localization,
                            range: loc.range.clone(),
                            path: loc.path.clone(),
                        },
                    });
                    count += 1;
                    if count > 1000 {
                        break;
                    }
                }
            }
        }

        push_symbols!(Ideology, ideologies, "Ideology");
        push_symbols!(EventNamespace, event_namespaces, "Event Namespace");

        {
            let map = &self.data.sub_ideologies;
            for entry in map.iter() {
                let name = entry.key();
                let (parent, range, path) = entry.value().resolve();
                if fuzzy_match(&query_lower, name) {
                    results.push(EntityHit {
                        name: name.to_string(),
                        kind: EntityKind::SubIdeology,
                        container: Some(format!("Sub-Ideology ({})", parent)),
                        location: EntityLocation {
                            kind: EntityKind::SubIdeology,
                            range: range.clone(),
                            path: path.clone(),
                        },
                    });
                }
            }
        }

        push_symbols!(Sprite, sprites, "Sprite");
        push_symbols!(MusicAsset, music_assets, "Music Asset");
        push_symbols!(MusicStation, music_stations, "Music Station");
        push_symbols!(Song, songs, "Song");
        push_symbols!(Sound, sounds, "Sound");
        push_symbols!(SoundEffect, sound_effects, "Sound Effect");
        push_symbols!(Falloff, falloffs, "Falloff");
        push_symbols!(SoundCategory, sound_categories, "Sound Category");
        push_symbols!(Character, characters, "Character");
        push_symbols!(Ability, abilities, "Ability");
        push_symbols!(Portrait, portraits, "Portrait");
        push_symbols!(ColorCode, color_codes, "Color Code");
        push_symbols!(Decision, decisions, "Decision");
        push_symbols!(TerrainCategory, terrain_categories, "Terrain Category");
        push_symbols!(CountryTag, country_tags, "Country Tag");
        push_symbols!(Building, buildings, "Building");
        push_symbols!(AiStrategyPlan, ai_strategy_plans, "AI Strategy Plan");
        push_symbols!(AiArea, ai_areas, "AI Area");
        push_symbols!(BalanceOfPower, balance_of_powers, "Balance of Power");

        results
    }
}
