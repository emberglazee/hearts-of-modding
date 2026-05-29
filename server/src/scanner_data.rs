use crate::ability_scanner;
use crate::achievement_scanner;
use crate::adjacency_scanner;
use crate::ai_area_scanner;
use crate::ai_strategy_plan_scanner;
use crate::ast;
use crate::building_scanner;
use crate::character_scanner;
use crate::continent_scanner;
use crate::country_scanner;
use crate::defines_parser;
use crate::event_scanner;
use crate::gfx_scanner;
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
use dashmap::{DashMap, DashSet};
use std::sync::Arc;

macro_rules! scanner_vec_field {
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

/// A memory-efficient, concurrent scanner registry.
///
/// HashMap-like registries use `DashMap` for lock-free concurrent reads and writes.
/// HashSets use `DashSet`. Vec-like registries and defines use `ArcSwap` (written
/// once during initial scan, never mutated incrementally).
///
/// Writing to a DashMap (incremental scanner): retain + insert — no cloning.
/// Reading: direct access — no snapshot overhead.
pub(crate) struct ScannerData {
    // ── DashMap registries (concurrent, lock-free incremental updates) ──
    pub localization: DashMap<String, loc_parser::LocEntry>,
    pub scripted_triggers: DashMap<String, scripted_scanner::ScriptedEntity>,
    pub scripted_effects: DashMap<String, scripted_scanner::ScriptedEntity>,
    pub ideologies: DashMap<String, ideology_scanner::Ideology>,
    pub sub_ideologies: DashMap<String, (String, ast::Range, String)>,
    pub traits: DashMap<String, trait_scanner::Trait>,
    pub sprites: DashMap<String, sprite_scanner::Sprite>,
    pub ideas: DashMap<String, idea_scanner::Idea>,
    pub characters: DashMap<String, character_scanner::Character>,
    pub variables: DashMap<String, Vec<variable_scanner::Variable>>,
    pub event_targets: DashMap<String, Vec<variable_scanner::EventTarget>>,
    pub provinces: DashMap<u32, province_scanner::Province>,
    pub custom_modifiers: DashMap<String, modifier_scanner::Modifier>,
    pub modifier_mappings: DashMap<String, String>,
    pub modifier_formats: DashMap<String, String>,
    pub events: DashMap<String, event_scanner::Event>,
    pub music_assets: DashMap<String, music_scanner::MusicAsset>,
    pub music_stations: DashMap<String, music_scanner::MusicStation>,
    pub songs: DashMap<String, music_scanner::Song>,
    pub sounds: DashMap<String, sound_scanner::Sound>,
    pub sound_effects: DashMap<String, sound_scanner::SoundEffect>,
    pub falloffs: DashMap<String, sound_scanner::Falloff>,
    pub sound_categories: DashMap<String, sound_scanner::SoundCategory>,
    pub buildings: DashMap<String, building_scanner::Building>,
    pub achievements: DashMap<String, achievement_scanner::Achievement>,
    pub abilities: DashMap<String, ability_scanner::Ability>,
    pub ai_strategy_plans: DashMap<String, ai_strategy_plan_scanner::AiStrategyPlan>,
    pub ai_areas: DashMap<String, ai_area_scanner::AiArea>,
    pub continents: DashMap<String, continent_scanner::Continent>,
    pub portraits: DashMap<String, portrait_scanner::Portrait>,
    pub scripted_locs: DashMap<String, scripted_loc_scanner::ScriptedLoc>,
    pub adjacency_rules: DashMap<String, adjacency_scanner::AdjacencyRule>,
    pub strategic_regions: DashMap<u32, strategic_region_scanner::StrategicRegion>,
    pub color_codes: DashMap<String, gfx_scanner::ColorCode>,
    pub country_tags: DashMap<String, country_scanner::CountryTag>,
    pub states: DashMap<u32, state_scanner::State>,

    // ── DashSet registries ──
    pub duplicated_loc_keys: DashSet<(String, String)>,
    pub game_loc_keys: DashSet<(String, String)>,
    pub workspace_files: DashSet<String>,

    // ── ArcSwap registries (written once at startup, rarely mutated) ──
    defines_field: Arc<ArcSwap<defines_parser::GameDefines>>,
    supply_nodes_field: Arc<ArcSwap<Vec<logistics_scanner::SupplyNode>>>,
    railways_field: Arc<ArcSwap<Vec<logistics_scanner::Railway>>>,
    map_buildings_field: Arc<ArcSwap<Vec<map_object_scanner::MapBuilding>>>,
    unitstacks_field: Arc<ArcSwap<Vec<map_object_scanner::UnitStack>>>,
    weather_positions_field: Arc<ArcSwap<Vec<map_object_scanner::WeatherPosition>>>,
    adjacencies_field: Arc<ArcSwap<Vec<adjacency_scanner::Adjacency>>>,
}

impl ScannerData {
    pub fn new() -> Self {
        ScannerData {
            localization: DashMap::new(),
            scripted_triggers: DashMap::new(),
            scripted_effects: DashMap::new(),
            ideologies: DashMap::new(),
            sub_ideologies: DashMap::new(),
            traits: DashMap::new(),
            sprites: DashMap::new(),
            ideas: DashMap::new(),
            characters: DashMap::new(),
            variables: DashMap::new(),
            event_targets: DashMap::new(),
            provinces: DashMap::new(),
            custom_modifiers: DashMap::new(),
            modifier_mappings: DashMap::new(),
            modifier_formats: DashMap::new(),
            events: DashMap::new(),
            music_assets: DashMap::new(),
            music_stations: DashMap::new(),
            songs: DashMap::new(),
            sounds: DashMap::new(),
            sound_effects: DashMap::new(),
            falloffs: DashMap::new(),
            sound_categories: DashMap::new(),
            buildings: DashMap::new(),
            achievements: DashMap::new(),
            abilities: DashMap::new(),
            ai_strategy_plans: DashMap::new(),
            ai_areas: DashMap::new(),
            continents: DashMap::new(),
            portraits: DashMap::new(),
            scripted_locs: DashMap::new(),
            adjacency_rules: DashMap::new(),
            strategic_regions: DashMap::new(),
            color_codes: DashMap::new(),
            country_tags: DashMap::new(),
            states: DashMap::new(),
            duplicated_loc_keys: DashSet::new(),
            game_loc_keys: DashSet::new(),
            workspace_files: DashSet::new(),
            defines_field: Arc::new(ArcSwap::from_pointee(defines_parser::GameDefines::new())),
            supply_nodes_field: Arc::new(ArcSwap::from_pointee(Vec::new())),
            railways_field: Arc::new(ArcSwap::from_pointee(Vec::new())),
            map_buildings_field: Arc::new(ArcSwap::from_pointee(Vec::new())),
            unitstacks_field: Arc::new(ArcSwap::from_pointee(Vec::new())),
            weather_positions_field: Arc::new(ArcSwap::from_pointee(Vec::new())),
            adjacencies_field: Arc::new(ArcSwap::from_pointee(Vec::new())),
        }
    }

    scanner_vec_field!(defines, defines_parser::GameDefines);
    scanner_vec_field!(supply_nodes, Vec<logistics_scanner::SupplyNode>);
    scanner_vec_field!(railways, Vec<logistics_scanner::Railway>);
    scanner_vec_field!(map_buildings, Vec<map_object_scanner::MapBuilding>);
    scanner_vec_field!(unitstacks, Vec<map_object_scanner::UnitStack>);
    scanner_vec_field!(weather_positions, Vec<map_object_scanner::WeatherPosition>);
    scanner_vec_field!(adjacencies, Vec<adjacency_scanner::Adjacency>);
}
