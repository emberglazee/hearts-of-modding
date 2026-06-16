use crate::data::interner::{InternedStr, Interner};
use crate::data::layered_value::LayeredValue;
use crate::for_each_standard_scanner;
use crate::parser::ast;
use crate::parser::defines_parser;
use crate::parser::loc_parser;
use crate::scanner::ability_scanner;
use crate::scanner::achievement_scanner;
use crate::scanner::adjacency_scanner;
use crate::scanner::ai_area_scanner;
use crate::scanner::ai_strategy_plan_scanner;
use crate::scanner::bop_scanner;
use crate::scanner::building_scanner;
use crate::scanner::character_scanner;
use crate::scanner::continent_scanner;
use crate::scanner::country_scanner;
use crate::scanner::event_namespace_scanner;
use crate::scanner::event_scanner;
use crate::scanner::focus_scanner;
use crate::scanner::gfx_scanner;
use crate::scanner::idea_scanner;
use crate::scanner::ideology_scanner;
use crate::scanner::incremental_scanner::HasPath;
use crate::scanner::logistics_scanner;
use crate::scanner::map_object_scanner;
use crate::scanner::modifier_scanner;
use crate::scanner::music_scanner;
use crate::scanner::oob_scanner;
use crate::scanner::portrait_scanner;
use crate::scanner::province_scanner;
use crate::scanner::resource_scanner;
use crate::scanner::scripted_loc_scanner;
use crate::scanner::scripted_scanner;
use crate::scanner::sound_scanner;
use crate::scanner::sprite_scanner;
use crate::scanner::state_category_scanner;
use crate::scanner::state_scanner;
use crate::scanner::strategic_region_scanner;
use crate::scanner::terrain_scanner;
use crate::scanner::trait_scanner;
use crate::scanner::unit_scanner;
use crate::scanner::variable_scanner;
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
///
/// **VFS layering:** Overlay-able registries (those where a mod file can override
/// a vanilla file of the same relative path) use `DashMap<InternedStr, LayeredValue<V>>`
/// instead of `DashMap<InternedStr, V>`. `LayeredValue` preserves ALL layers,
/// sorted by increasing priority: entry[0] = vanilla, entry[last] = active mod.
/// Always use `.resolve()` to get the active entry. When a mod file is deleted,
/// `remove_path!` only removes that file's layer, keeping lower-priority layers.
pub(crate) struct ScannerData {
    pub interner: Interner,

    // ── DashMap registries (VFS-layered: vanilla → mod) ──
    // These use LayeredValue to preserve lower-priority layers when
    // a higher-priority file is deleted.
    pub unit_types: DashMap<InternedStr, LayeredValue<unit_scanner::UnitType>>,
    pub localization: DashMap<InternedStr, LayeredValue<loc_parser::LocEntry>>,
    pub scripted_triggers: DashMap<InternedStr, LayeredValue<scripted_scanner::ScriptedEntity>>,
    pub scripted_effects: DashMap<InternedStr, LayeredValue<scripted_scanner::ScriptedEntity>>,
    pub ideologies: DashMap<InternedStr, LayeredValue<ideology_scanner::Ideology>>,
    pub sub_ideologies: DashMap<InternedStr, LayeredValue<(InternedStr, ast::Range, InternedStr)>>,
    pub traits: DashMap<InternedStr, LayeredValue<trait_scanner::Trait>>,
    pub sprites: DashMap<InternedStr, LayeredValue<sprite_scanner::Sprite>>,
    pub ideas: DashMap<InternedStr, LayeredValue<idea_scanner::Idea>>,
    pub characters: DashMap<InternedStr, LayeredValue<character_scanner::Character>>,
    pub variables: DashMap<InternedStr, Vec<variable_scanner::Variable>>,
    pub event_targets: DashMap<InternedStr, Vec<variable_scanner::EventTarget>>,
    pub provinces: DashMap<u32, province_scanner::Province>,
    pub custom_modifiers: DashMap<InternedStr, LayeredValue<modifier_scanner::Modifier>>,
    pub modifier_mappings: DashMap<InternedStr, String>,
    pub modifier_formats: DashMap<InternedStr, String>,
    pub events: DashMap<InternedStr, LayeredValue<event_scanner::Event>>,
    pub focuses: DashMap<InternedStr, LayeredValue<focus_scanner::Focus>>,
    pub music_assets: DashMap<InternedStr, LayeredValue<music_scanner::MusicAsset>>,
    pub music_stations: DashMap<InternedStr, LayeredValue<music_scanner::MusicStation>>,
    pub songs: DashMap<InternedStr, LayeredValue<music_scanner::Song>>,
    pub sounds: DashMap<InternedStr, LayeredValue<sound_scanner::Sound>>,
    pub sound_effects: DashMap<InternedStr, LayeredValue<sound_scanner::SoundEffect>>,
    pub falloffs: DashMap<InternedStr, LayeredValue<sound_scanner::Falloff>>,
    pub sound_categories: DashMap<InternedStr, LayeredValue<sound_scanner::SoundCategory>>,
    pub buildings: DashMap<InternedStr, LayeredValue<building_scanner::Building>>,
    pub resources: DashMap<InternedStr, LayeredValue<resource_scanner::Resource>>,
    pub state_categories: DashMap<InternedStr, LayeredValue<state_category_scanner::StateCategory>>,
    pub achievements: DashMap<InternedStr, LayeredValue<achievement_scanner::Achievement>>,
    pub abilities: DashMap<InternedStr, LayeredValue<ability_scanner::Ability>>,
    pub ai_strategy_plans:
        DashMap<InternedStr, LayeredValue<ai_strategy_plan_scanner::AiStrategyPlan>>,
    pub ai_areas: DashMap<InternedStr, LayeredValue<ai_area_scanner::AiArea>>,
    pub continents: DashMap<InternedStr, LayeredValue<continent_scanner::Continent>>,
    pub portraits: DashMap<InternedStr, LayeredValue<portrait_scanner::Portrait>>,
    pub scripted_locs: DashMap<InternedStr, LayeredValue<scripted_loc_scanner::ScriptedLoc>>,
    pub adjacency_rules: DashMap<InternedStr, LayeredValue<adjacency_scanner::AdjacencyRule>>,
    pub strategic_regions: DashMap<u32, strategic_region_scanner::StrategicRegion>,
    pub terrain_categories: DashMap<InternedStr, LayeredValue<terrain_scanner::TerrainCategory>>,
    pub balance_of_powers: DashMap<InternedStr, LayeredValue<bop_scanner::BalanceOfPower>>,
    pub color_codes: DashMap<InternedStr, LayeredValue<gfx_scanner::ColorCode>>,
    pub country_tags: DashMap<InternedStr, LayeredValue<country_scanner::CountryTag>>,
    pub states: DashMap<u32, state_scanner::State>,
    pub oob_division_templates:
        DashMap<InternedStr, LayeredValue<oob_scanner::OobDivisionTemplate>>,
    pub oob_fleets: DashMap<InternedStr, LayeredValue<oob_scanner::OobFleet>>,
    pub event_namespaces:
        DashMap<InternedStr, LayeredValue<event_namespace_scanner::EventNamespace>>,
    pub event_namespaces_file_index: DashMap<InternedStr, Vec<InternedStr>>,

    // ── Reverse file-path indices for O(K) incremental updates ──
    // Maps file path -> Vec of keys defined in that file.
    // Populated once after initial scan, used by retain_path! in incremental_scanner
    // to avoid O(N) DashMap::retain on every keystroke.
    pub localization_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub unit_types_file_index: DashMap<InternedStr, Vec<InternedStr>>,
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
    pub resources_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub state_categories_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub achievements_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub abilities_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub ai_strategy_plans_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub ai_areas_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub portraits_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub scripted_locs_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub country_tags_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub strategic_regions_file_index: DashMap<InternedStr, Vec<u32>>,
    pub terrain_categories_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub balance_of_powers_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub oob_division_templates_file_index: DashMap<InternedStr, Vec<InternedStr>>,
    pub oob_fleets_file_index: DashMap<InternedStr, Vec<InternedStr>>,

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
            unit_types: DashMap::new(),
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
            resources: DashMap::new(),
            state_categories: DashMap::new(),
            achievements: DashMap::new(),
            abilities: DashMap::new(),
            ai_strategy_plans: DashMap::new(),
            ai_areas: DashMap::new(),
            continents: DashMap::new(),
            portraits: DashMap::new(),
            scripted_locs: DashMap::new(),
            adjacency_rules: DashMap::new(),
            strategic_regions: DashMap::new(),
            terrain_categories: DashMap::new(),
            balance_of_powers: DashMap::new(),
            color_codes: DashMap::new(),
            country_tags: DashMap::new(),
            states: DashMap::new(),
            oob_division_templates: DashMap::new(),
            oob_fleets: DashMap::new(),
            event_namespaces: DashMap::new(),
            event_namespaces_file_index: DashMap::new(),
            localization_file_index: DashMap::new(),
            unit_types_file_index: DashMap::new(),
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
            resources_file_index: DashMap::new(),
            state_categories_file_index: DashMap::new(),
            achievements_file_index: DashMap::new(),
            abilities_file_index: DashMap::new(),
            ai_strategy_plans_file_index: DashMap::new(),
            ai_areas_file_index: DashMap::new(),
            portraits_file_index: DashMap::new(),
            scripted_locs_file_index: DashMap::new(),
            country_tags_file_index: DashMap::new(),
            strategic_regions_file_index: DashMap::new(),
            terrain_categories_file_index: DashMap::new(),
            balance_of_powers_file_index: DashMap::new(),
            oob_division_templates_file_index: DashMap::new(),
            oob_fleets_file_index: DashMap::new(),
            duplicated_loc_keys: DashSet::new(),
            game_loc_keys: DashSet::new(),
            workspace_files: DashSet::new(),
            defines_field: Arc::new(ArcSwap::from_pointee(defines_parser::GameDefines::default())),
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
                    // LayeredValue — take the resolved entry for path tracking
                    let path = self.interner.intern(entry.value().resolve().path());
                    $index.entry(path).or_default().push(entry.key().clone());
                }
            };
        }

        macro_rules! rebuild_index_flat {
            ($map:expr, $index:expr) => {
                $index.clear();
                for entry in $map.iter() {
                    let path = self.interner.intern(entry.value().path());
                    $index.entry(path).or_default().push(entry.key().clone());
                }
            };
        }

        // ── Standard scanners (generated) ──
        macro_rules! std_rebuild {
            ($mod:ident, $ty:ident, $kind:ident, $field:ident, $dir:expr, $ext:expr) => {
                paste::paste! {
                    rebuild_index!(self.[<$field>], self.[<$field _file_index>]);
                }
            };
        }
        for_each_standard_scanner!(std_rebuild);

        // ── Special scanners (manual) ──
        rebuild_index!(self.localization, self.localization_file_index);
        rebuild_index!(self.unit_types, self.unit_types_file_index);
        rebuild_index!(self.scripted_triggers, self.scripted_triggers_file_index);
        rebuild_index!(self.scripted_effects, self.scripted_effects_file_index);
        rebuild_index!(self.ideologies, self.ideologies_file_index);
        // sub_ideologies is a tuple (InternedStr, Range, InternedStr) — path is v.2
        self.sub_ideologies_file_index.clear();
        for entry in self.sub_ideologies.iter() {
            let path = entry.value().resolve().2.clone();
            self.sub_ideologies_file_index
                .entry(path)
                .or_default()
                .push(entry.key().clone());
        }
        rebuild_index!(self.traits, self.traits_file_index);
        rebuild_index!(self.scripted_locs, self.scripted_locs_file_index);
        rebuild_index!(self.custom_modifiers, self.custom_modifiers_file_index);
        rebuild_index!(self.events, self.events_file_index);
        rebuild_index!(self.music_assets, self.music_assets_file_index);
        rebuild_index!(self.music_stations, self.music_stations_file_index);
        rebuild_index!(self.songs, self.songs_file_index);
        rebuild_index!(self.sounds, self.sounds_file_index);
        rebuild_index!(self.sound_effects, self.sound_effects_file_index);
        rebuild_index!(self.falloffs, self.falloffs_file_index);
        rebuild_index!(self.sound_categories, self.sound_categories_file_index);
        rebuild_index!(self.country_tags, self.country_tags_file_index);
        rebuild_index_flat!(self.strategic_regions, self.strategic_regions_file_index);
        rebuild_index!(
            self.oob_division_templates,
            self.oob_division_templates_file_index
        );
        rebuild_index!(self.oob_fleets, self.oob_fleets_file_index);
        rebuild_index!(self.event_namespaces, self.event_namespaces_file_index);
    }
}
