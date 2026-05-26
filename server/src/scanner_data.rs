use crate::ability_scanner;
use crate::achievement_scanner;
use crate::adjacency_scanner;
use crate::ai_strategy_plan_scanner;
use crate::ast;
use crate::building_scanner;
use crate::character_scanner;
use crate::country_scanner;
use crate::defines_parser;
use crate::event_scanner;
use crate::idea_scanner;
use crate::ideology_scanner;
use crate::loc_parser;
use crate::logistics_scanner;
use crate::map_object_scanner;
use crate::modifier_scanner;
use crate::music_scanner;
use crate::portrait_scanner;
use crate::province_scanner;
use crate::scripted_loc_scanner;
use crate::scripted_scanner;
use crate::sound_scanner;
use crate::sprite_scanner;
use crate::state_scanner;
use crate::strategic_region_scanner;
use crate::trait_scanner;
use crate::variable_scanner;
use arc_swap::ArcSwap;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

macro_rules! scanner_field {
    ($name:ident, $ty:ty) => {
        paste::paste! {
            pub fn $name(&self) -> std::sync::Arc<$ty> {
                self.[<$name _field>].load_full()
            }
            pub fn [<set_ $name>](&self, value: $ty) {
                self.[<$name _field>].store(std::sync::Arc::new(value));
            }
        }
    };
}

pub(crate) struct ScannerData {
    localization_field: Arc<ArcSwap<HashMap<String, loc_parser::LocEntry>>>,
    scripted_triggers_field: Arc<ArcSwap<HashMap<String, scripted_scanner::ScriptedEntity>>>,
    scripted_effects_field: Arc<ArcSwap<HashMap<String, scripted_scanner::ScriptedEntity>>>,
    ideologies_field: Arc<ArcSwap<HashMap<String, ideology_scanner::Ideology>>>,
    sub_ideologies_field: Arc<ArcSwap<HashMap<String, (String, ast::Range, String)>>>,
    traits_field: Arc<ArcSwap<HashMap<String, trait_scanner::Trait>>>,
    sprites_field: Arc<ArcSwap<HashMap<String, sprite_scanner::Sprite>>>,
    ideas_field: Arc<ArcSwap<HashMap<String, idea_scanner::Idea>>>,
    characters_field: Arc<ArcSwap<HashMap<String, character_scanner::Character>>>,
    variables_field: Arc<ArcSwap<HashMap<String, Vec<variable_scanner::Variable>>>>,
    event_targets_field: Arc<ArcSwap<HashMap<String, Vec<variable_scanner::EventTarget>>>>,
    provinces_field: Arc<ArcSwap<HashMap<u32, province_scanner::Province>>>,
    custom_modifiers_field: Arc<ArcSwap<HashMap<String, modifier_scanner::Modifier>>>,
    modifier_mappings_field: Arc<ArcSwap<HashMap<String, String>>>,
    modifier_formats_field: Arc<ArcSwap<HashMap<String, String>>>,
    events_field: Arc<ArcSwap<HashMap<String, event_scanner::Event>>>,
    music_assets_field: Arc<ArcSwap<HashMap<String, music_scanner::MusicAsset>>>,
    music_stations_field: Arc<ArcSwap<HashMap<String, music_scanner::MusicStation>>>,
    songs_field: Arc<ArcSwap<HashMap<String, music_scanner::Song>>>,
    sounds_field: Arc<ArcSwap<HashMap<String, sound_scanner::Sound>>>,
    sound_effects_field: Arc<ArcSwap<HashMap<String, sound_scanner::SoundEffect>>>,
    falloffs_field: Arc<ArcSwap<HashMap<String, sound_scanner::Falloff>>>,
    sound_categories_field: Arc<ArcSwap<HashMap<String, sound_scanner::SoundCategory>>>,
    buildings_field: Arc<ArcSwap<HashMap<String, building_scanner::Building>>>,
    achievements_field: Arc<ArcSwap<HashMap<String, achievement_scanner::Achievement>>>,
    defines_field: Arc<ArcSwap<defines_parser::GameDefines>>,
    abilities_field: Arc<ArcSwap<HashMap<String, ability_scanner::Ability>>>,
    ai_strategy_plans_field: Arc<ArcSwap<HashMap<String, ai_strategy_plan_scanner::AiStrategyPlan>>>,
    portraits_field: Arc<ArcSwap<HashMap<String, portrait_scanner::Portrait>>>,
    scripted_locs_field: Arc<ArcSwap<HashMap<String, scripted_loc_scanner::ScriptedLoc>>>,
    duplicated_loc_keys_field: Arc<ArcSwap<HashSet<(String, String)>>>,
    states_field: Arc<ArcSwap<HashMap<u32, state_scanner::State>>>,
    supply_nodes_field: Arc<ArcSwap<Vec<logistics_scanner::SupplyNode>>>,
    railways_field: Arc<ArcSwap<Vec<logistics_scanner::Railway>>>,
    map_buildings_field: Arc<ArcSwap<Vec<map_object_scanner::MapBuilding>>>,
    unitstacks_field: Arc<ArcSwap<Vec<map_object_scanner::UnitStack>>>,
    weather_positions_field: Arc<ArcSwap<Vec<map_object_scanner::WeatherPosition>>>,
    adjacencies_field: Arc<ArcSwap<Vec<adjacency_scanner::Adjacency>>>,
    adjacency_rules_field: Arc<ArcSwap<HashMap<String, adjacency_scanner::AdjacencyRule>>>,
    strategic_regions_field: Arc<ArcSwap<HashMap<u32, strategic_region_scanner::StrategicRegion>>>,
    country_tags_field: Arc<ArcSwap<HashMap<String, country_scanner::CountryTag>>>,
    workspace_files_field: Arc<ArcSwap<HashSet<String>>>,
}

impl ScannerData {
    pub fn new() -> Self {
        ScannerData {
            localization_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            scripted_triggers_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            scripted_effects_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            ideologies_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            sub_ideologies_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            traits_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            sprites_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            ideas_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            characters_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            variables_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            event_targets_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            provinces_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            custom_modifiers_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            modifier_mappings_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            modifier_formats_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            events_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            music_assets_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            music_stations_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            songs_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            sounds_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            sound_effects_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            falloffs_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            sound_categories_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            buildings_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            achievements_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            defines_field: Arc::new(ArcSwap::from_pointee(defines_parser::GameDefines::new())),
            abilities_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            ai_strategy_plans_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            portraits_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            scripted_locs_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            duplicated_loc_keys_field: Arc::new(ArcSwap::from_pointee(HashSet::new())),
            states_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            supply_nodes_field: Arc::new(ArcSwap::from_pointee(Vec::new())),
            railways_field: Arc::new(ArcSwap::from_pointee(Vec::new())),
            map_buildings_field: Arc::new(ArcSwap::from_pointee(Vec::new())),
            unitstacks_field: Arc::new(ArcSwap::from_pointee(Vec::new())),
            weather_positions_field: Arc::new(ArcSwap::from_pointee(Vec::new())),
            adjacencies_field: Arc::new(ArcSwap::from_pointee(Vec::new())),
            adjacency_rules_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            strategic_regions_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            country_tags_field: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            workspace_files_field: Arc::new(ArcSwap::from_pointee(HashSet::new())),
        }
    }

    scanner_field!(localization, HashMap<String, loc_parser::LocEntry>);
    scanner_field!(scripted_triggers, HashMap<String, scripted_scanner::ScriptedEntity>);
    scanner_field!(scripted_effects, HashMap<String, scripted_scanner::ScriptedEntity>);
    scanner_field!(ideologies, HashMap<String, ideology_scanner::Ideology>);
    scanner_field!(sub_ideologies, HashMap<String, (String, ast::Range, String)>);
    scanner_field!(traits, HashMap<String, trait_scanner::Trait>);
    scanner_field!(sprites, HashMap<String, sprite_scanner::Sprite>);
    scanner_field!(ideas, HashMap<String, idea_scanner::Idea>);
    scanner_field!(characters, HashMap<String, character_scanner::Character>);
    scanner_field!(variables, HashMap<String, Vec<variable_scanner::Variable>>);
    scanner_field!(event_targets, HashMap<String, Vec<variable_scanner::EventTarget>>);
    scanner_field!(provinces, HashMap<u32, province_scanner::Province>);
    scanner_field!(custom_modifiers, HashMap<String, modifier_scanner::Modifier>);
    scanner_field!(modifier_mappings, HashMap<String, String>);
    scanner_field!(modifier_formats, HashMap<String, String>);
    scanner_field!(events, HashMap<String, event_scanner::Event>);
    scanner_field!(music_assets, HashMap<String, music_scanner::MusicAsset>);
    scanner_field!(music_stations, HashMap<String, music_scanner::MusicStation>);
    scanner_field!(songs, HashMap<String, music_scanner::Song>);
    scanner_field!(sounds, HashMap<String, sound_scanner::Sound>);
    scanner_field!(sound_effects, HashMap<String, sound_scanner::SoundEffect>);
    scanner_field!(falloffs, HashMap<String, sound_scanner::Falloff>);
    scanner_field!(sound_categories, HashMap<String, sound_scanner::SoundCategory>);
    scanner_field!(buildings, HashMap<String, building_scanner::Building>);
    scanner_field!(achievements, HashMap<String, achievement_scanner::Achievement>);
    scanner_field!(defines, defines_parser::GameDefines);
    scanner_field!(abilities, HashMap<String, ability_scanner::Ability>);
    scanner_field!(ai_strategy_plans, HashMap<String, ai_strategy_plan_scanner::AiStrategyPlan>);
    scanner_field!(portraits, HashMap<String, portrait_scanner::Portrait>);
    scanner_field!(scripted_locs, HashMap<String, scripted_loc_scanner::ScriptedLoc>);
    scanner_field!(duplicated_loc_keys, HashSet<(String, String)>);
    scanner_field!(states, HashMap<u32, state_scanner::State>);
    scanner_field!(supply_nodes, Vec<logistics_scanner::SupplyNode>);
    scanner_field!(railways, Vec<logistics_scanner::Railway>);
    scanner_field!(map_buildings, Vec<map_object_scanner::MapBuilding>);
    scanner_field!(unitstacks, Vec<map_object_scanner::UnitStack>);
    scanner_field!(weather_positions, Vec<map_object_scanner::WeatherPosition>);
    scanner_field!(adjacencies, Vec<adjacency_scanner::Adjacency>);
    scanner_field!(adjacency_rules, HashMap<String, adjacency_scanner::AdjacencyRule>);
    scanner_field!(strategic_regions, HashMap<u32, strategic_region_scanner::StrategicRegion>);
    scanner_field!(country_tags, HashMap<String, country_scanner::CountryTag>);
    scanner_field!(workspace_files, HashSet<String>);
}
