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
use crate::focus_scanner;
use crate::gfx_scanner;
use crate::idea_scanner;
use crate::ideology_scanner;
use crate::incremental_scanner::HasPath;
use crate::interner::{InternedStr, Interner};
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
/// All string keys and path fields are interned (`Arc<str>`) to reduce memory
/// duplication and make clones cheap (atomic increment).
pub(crate) struct ScannerData {
    pub interner: Interner,

    // ── DashMap registries (concurrent, lock-free incremental updates) ──
    pub localization: DashMap<InternedStr, loc_parser::LocEntry>,
    pub scripted_triggers: DashMap<InternedStr, scripted_scanner::ScriptedEntity>,
    pub scripted_effects: DashMap<InternedStr, scripted_scanner::ScriptedEntity>,
    pub ideologies: DashMap<InternedStr, ideology_scanner::Ideology>,
    pub sub_ideologies: DashMap<InternedStr, (InternedStr, ast::Range, InternedStr)>,
    pub traits: DashMap<InternedStr, trait_scanner::Trait>,
    pub sprites: DashMap<InternedStr, sprite_scanner::Sprite>,
    pub ideas: DashMap<InternedStr, idea_scanner::Idea>,
    pub characters: DashMap<InternedStr, character_scanner::Character>,
    pub variables: DashMap<InternedStr, Vec<variable_scanner::Variable>>,
    pub event_targets: DashMap<InternedStr, Vec<variable_scanner::EventTarget>>,
    pub provinces: DashMap<u32, province_scanner::Province>,
    pub custom_modifiers: DashMap<InternedStr, modifier_scanner::Modifier>,
    pub modifier_mappings: DashMap<InternedStr, String>,
    pub modifier_formats: DashMap<InternedStr, String>,
    pub events: DashMap<InternedStr, event_scanner::Event>,
    pub focuses: DashMap<InternedStr, focus_scanner::Focus>,
    pub music_assets: DashMap<InternedStr, music_scanner::MusicAsset>,
    pub music_stations: DashMap<InternedStr, music_scanner::MusicStation>,
    pub songs: DashMap<InternedStr, music_scanner::Song>,
    pub sounds: DashMap<InternedStr, sound_scanner::Sound>,
    pub sound_effects: DashMap<InternedStr, sound_scanner::SoundEffect>,
    pub falloffs: DashMap<InternedStr, sound_scanner::Falloff>,
    pub sound_categories: DashMap<InternedStr, sound_scanner::SoundCategory>,
    pub buildings: DashMap<InternedStr, building_scanner::Building>,
    pub achievements: DashMap<InternedStr, achievement_scanner::Achievement>,
    pub abilities: DashMap<InternedStr, ability_scanner::Ability>,
    pub ai_strategy_plans: DashMap<InternedStr, ai_strategy_plan_scanner::AiStrategyPlan>,
    pub ai_areas: DashMap<InternedStr, ai_area_scanner::AiArea>,
    pub continents: DashMap<InternedStr, continent_scanner::Continent>,
    pub portraits: DashMap<InternedStr, portrait_scanner::Portrait>,
    pub scripted_locs: DashMap<InternedStr, scripted_loc_scanner::ScriptedLoc>,
    pub adjacency_rules: DashMap<InternedStr, adjacency_scanner::AdjacencyRule>,
    pub strategic_regions: DashMap<u32, strategic_region_scanner::StrategicRegion>,
    pub color_codes: DashMap<InternedStr, gfx_scanner::ColorCode>,
    pub country_tags: DashMap<InternedStr, country_scanner::CountryTag>,
    pub states: DashMap<u32, state_scanner::State>,

    // ── Reverse file-path indices for O(K) incremental updates ──
    // Maps file path -> Vec of keys defined in that file.
    // Populated once after initial scan, used by retain_path! in incremental_scanner
    // to avoid O(N) DashMap::retain on every keystroke.
    pub localization_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub scripted_triggers_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub scripted_effects_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub ideologies_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub sub_ideologies_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub traits_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub sprites_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub ideas_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub characters_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub custom_modifiers_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub events_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub focuses_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub music_assets_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub music_stations_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub songs_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub sounds_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub sound_effects_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub falloffs_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub sound_categories_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub buildings_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub achievements_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub abilities_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub ai_strategy_plans_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub ai_areas_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub portraits_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub scripted_locs_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub country_tags_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub strategic_regions_file_index: DashMap<InternedStr, Vec<u32>>,

    // ── DashSet registries ──
    pub duplicated_loc_keys: DashSet<(InternedStr, InternedStr)>,
    pub game_loc_keys: DashSet<(InternedStr, InternedStr)>,
    pub workspace_files: DashSet<InternedStr>,

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
            interner: Interner::new(),
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
            focuses: DashMap::new(),
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
            localization_file_index: DashMap::new(),
            scripted_triggers_file_index: DashMap::new(),
            scripted_effects_file_index: DashMap::new(),
            ideologies_file_index: DashMap::new(),
            sub_ideologies_file_index: DashMap::new(),
            traits_file_index: DashMap::new(),
            sprites_file_index: DashMap::new(),
            ideas_file_index: DashMap::new(),
            characters_file_index: DashMap::new(),
            custom_modifiers_file_index: DashMap::new(),
            events_file_index: DashMap::new(),
            focuses_file_index: DashMap::new(),
            music_assets_file_index: DashMap::new(),
            music_stations_file_index: DashMap::new(),
            songs_file_index: DashMap::new(),
            sounds_file_index: DashMap::new(),
            sound_effects_file_index: DashMap::new(),
            falloffs_file_index: DashMap::new(),
            sound_categories_file_index: DashMap::new(),
            buildings_file_index: DashMap::new(),
            achievements_file_index: DashMap::new(),
            abilities_file_index: DashMap::new(),
            ai_strategy_plans_file_index: DashMap::new(),
            ai_areas_file_index: DashMap::new(),
            portraits_file_index: DashMap::new(),
            scripted_locs_file_index: DashMap::new(),
            country_tags_file_index: DashMap::new(),
            strategic_regions_file_index: DashMap::new(),
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

    /// Rebuild all reverse file-path indices from current DashMap contents.
    ///
    /// Call this once after the full initial scan completes.
    /// Without indices, the first incremental update per file path falls back
    /// to O(N) retain — this populates them so all subsequent edits are O(K).
    #[allow(clippy::too_many_lines)]
    pub fn rebuild_all_file_indices(&self) {
        macro_rules! rebuild_index {
            ($map:expr, $index:expr) => {
                $index.clear();
                for entry in $map.iter() {
                    let path = self.interner.intern(entry.value().path());
                    $index
                        .entry(path)
                        .or_default()
                        .push(entry.key().clone());
                }
            };
        }

        rebuild_index!(self.localization, self.localization_file_index);
        rebuild_index!(self.scripted_triggers, self.scripted_triggers_file_index);
        rebuild_index!(self.scripted_effects, self.scripted_effects_file_index);
        rebuild_index!(self.ideologies, self.ideologies_file_index);
        // sub_ideologies is a tuple (InternedStr, Range, InternedStr) — path is v.2
        self.sub_ideologies_file_index.clear();
        for entry in self.sub_ideologies.iter() {
            let path = entry.value().2.clone();
            self.sub_ideologies_file_index
                .entry(path)
                .or_default()
                .push(entry.key().clone());
        }
        rebuild_index!(self.traits, self.traits_file_index);
        rebuild_index!(self.sprites, self.sprites_file_index);
        rebuild_index!(self.ideas, self.ideas_file_index);
        rebuild_index!(self.characters, self.characters_file_index);
        rebuild_index!(self.custom_modifiers, self.custom_modifiers_file_index);
        rebuild_index!(self.events, self.events_file_index);
        rebuild_index!(self.focuses, self.focuses_file_index);
        rebuild_index!(self.music_assets, self.music_assets_file_index);
        rebuild_index!(self.music_stations, self.music_stations_file_index);
        rebuild_index!(self.songs, self.songs_file_index);
        rebuild_index!(self.sounds, self.sounds_file_index);
        rebuild_index!(self.sound_effects, self.sound_effects_file_index);
        rebuild_index!(self.falloffs, self.falloffs_file_index);
        rebuild_index!(self.sound_categories, self.sound_categories_file_index);
        rebuild_index!(self.buildings, self.buildings_file_index);
        rebuild_index!(self.achievements, self.achievements_file_index);
        rebuild_index!(self.abilities, self.abilities_file_index);
        rebuild_index!(self.ai_strategy_plans, self.ai_strategy_plans_file_index);
        rebuild_index!(self.ai_areas, self.ai_areas_file_index);
        rebuild_index!(self.portraits, self.portraits_file_index);
        rebuild_index!(self.scripted_locs, self.scripted_locs_file_index);
        rebuild_index!(self.country_tags, self.country_tags_file_index);
        rebuild_index!(self.strategic_regions, self.strategic_regions_file_index);
    }
}
