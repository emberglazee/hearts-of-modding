#![allow(dead_code)]

use crate::ScannerData;
use crate::data::interner::InternedStr;
use crate::for_each_standard_scanner;
use crate::parser::ast;
use crate::scanner::incremental_scanner::{index_key, path_matches};
use crate::utils::lsp_convert::{is_pos_in_range, to_byte_position};
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
    AceModifier,
    Achievement,
    AiArea,
    Variable,
    Array,
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
    DecisionCategory,
    Resource,
    StateCategory,
    OobDivisionTemplate,
    OobFleet,
    UnitType,
    Technology,
    TechnologyTag,
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
            EntityKind::AceModifier => SymbolKind::CLASS,
            EntityKind::Achievement => SymbolKind::EVENT,
            EntityKind::Variable => SymbolKind::VARIABLE,
            EntityKind::Array => SymbolKind::VARIABLE,
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
            EntityKind::DecisionCategory => SymbolKind::ENUM,
            EntityKind::Resource => SymbolKind::PROPERTY,
            EntityKind::StateCategory => SymbolKind::ENUM,
            EntityKind::OobDivisionTemplate => SymbolKind::STRUCT,
            EntityKind::OobFleet => SymbolKind::OBJECT,
            EntityKind::UnitType => SymbolKind::CLASS,
            EntityKind::Technology => SymbolKind::OBJECT,
            EntityKind::TechnologyTag => SymbolKind::NAMESPACE,
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
            let map = &self.data.arrays;
            if let Some(arrs) = map.get(key) {
                for arr in arrs.iter() {
                    results.push(EntityLocation {
                        kind: EntityKind::Array,
                        range: arr.range.clone(),
                        path: arr.path.clone(),
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
        try_lookup!(TechnologyTag, technology_tags);

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
            // Exact match only. (A `{key}:`-prefixed scan used to live here;
            // stored loc keys are the text before the first `:` — e.g. `foo`
            // from `foo:0 "..."` — so they can never contain `:` and that scan
            // was O(N) dead work even gated behind the exact-loc hit.)
            if let Some(entry) = loc.get(key) {
                results.push(EntityLocation {
                    kind: EntityKind::Localization,
                    range: entry.range.clone(),
                    path: entry.path.clone(),
                });
            }
        }

        results
    }

    /// Resolve a **numeric** reference to a u32-keyed entity (state, strategic
    /// region, province) to its definition location.
    ///
    /// `find_definition` looks up string-keyed entities and can never resolve
    /// `state = 422` — for a `key = <number>` assignment the identifier
    /// returned by `find_identifier_at` is the *key text* (`"state"`), not the
    /// number, and the maps below are keyed by `u32`. This method takes the
    /// parsed number plus the surrounding identifier/context key and applies
    /// the same guards the hover uses (a bare number in an unrelated key must
    /// not jump to a state of the same id).
    pub fn find_numeric_definition(
        &self,
        id: u32,
        identifier: &str,
        context_key: Option<&str>,
    ) -> Vec<EntityLocation> {
        let mut results = Vec::new();
        let ident_lower = identifier.to_ascii_lowercase();
        let ctx_lower = context_key.map(|s| s.to_ascii_lowercase());
        // Mirrors `hover_handler`'s guards for the state/region sections.
        let is_state_key = ident_lower.contains("state")
            || ident_lower.contains("capital")
            || (ident_lower == "id" && ctx_lower.as_deref() == Some("state"))
            || ident_lower == "add_core_of"
            || ident_lower == "add_claim_by"
            || (identifier.parse::<u32>().is_ok()
                && ctx_lower.as_ref().is_some_and(|ck| ck.contains("state")));
        let is_region_key = ident_lower.contains("strategic_region")
            || (ident_lower == "id" && ctx_lower.as_deref() == Some("strategic_region"))
            || (identifier.parse::<u32>().is_ok()
                && ctx_lower
                    .as_ref()
                    .is_some_and(|ck| ck.contains("strategic_region")));
        let is_province_key = ident_lower.contains("province")
            || ident_lower == "victory_points"
            || (identifier.parse::<u32>().is_ok()
                && ctx_lower
                    .as_ref()
                    .is_some_and(|ck| ck.contains("province") || ck == "victory_points"));

        if is_state_key {
            if let Some(state) = self.data.states.get(&id) {
                results.push(EntityLocation {
                    kind: EntityKind::State,
                    range: state.range.clone(),
                    path: state.path.clone(),
                });
            }
        }
        if is_region_key {
            if let Some(region) = self.data.strategic_regions.get(&id) {
                results.push(EntityLocation {
                    kind: EntityKind::StrategicRegion,
                    range: region.range.clone(),
                    path: region.path.clone(),
                });
            }
        }
        if is_province_key {
            if let Some(province) = self.data.provinces.get(&id) {
                results.push(EntityLocation {
                    kind: EntityKind::Province,
                    range: province.range.clone(),
                    path: province.path.clone(),
                });
            }
        }
        results
    }

    pub fn entity_at(
        &self,
        path: &str,
        content: &str,
        pos: Position,
    ) -> Option<(EntityKind, ast::Range, String)> {
        // The client position is in UTF-16; AST ranges are byte columns. Convert
        // the cursor to byte so `is_pos_in_range` matches multi-byte lines too.
        let pos = to_byte_position(content, pos);
        macro_rules! check_entity {
            ($kind:ident, $name:ident) => {
                // Reverse per-path index (path -> declared names) — O(entities-at-path).
                // The index is keyed by forward-slash-normalized paths, so the
                // lookup normalizes too — on Windows, `path` arrives from
                // `Uri::to_file_path()` (forward slashes) while the index was
                // built from raw PathBuf strings (backslashes).
                paste::paste! {
                    let index = &self.data.[<$name _file_index>];
                    if let Some(names) = index.get(&index_key(path)) {
                        for name in names.value() {
                            if let Some(entity) = self.data.$name.get(&**name) {
                                let e = entity.value();
                                if is_pos_in_range(pos, &e.range) {
                                    return Some((
                                        EntityKind::$kind,
                                        e.range.clone(),
                                        name.to_string(),
                                    ));
                                }
                            }
                        }
                    }
                }
            };
        }

        // Standard scanners (generated)
        macro_rules! std_check_entity {
            ($mod:ident, $ty:ident, $kind:ident, $field:ident, $dir:expr, $ext:expr) => {
                // Reverse per-path index (`<field>_file_index`: path → declared
                // names) so we only iterate the entities declared in THIS file,
                // instead of scanning every entity in the map. The index is
                // keyed by forward-slash-normalized paths; the lookup
                // normalizes `path` to match (see `index_key`).
                paste::paste! {
                    let index = &self.data.[<$field _file_index>];
                    if let Some(names) = index.get(&index_key(path)) {
                        for name in names.value() {
                            if let Some(entity) = self.data.$field.get(&**name) {
                                let e = entity.value();
                                if is_pos_in_range(pos, &e.range) {
                                    return Some((
                                        EntityKind::$kind,
                                        e.range.clone(),
                                        name.to_string(),
                                    ));
                                }
                            }
                        }
                    }
                }
            };
        }
        for_each_standard_scanner!(std_check_entity);

        // Special scanners (manual)
        check_entity!(Event, events);
        check_entity!(TechnologyTag, technology_tags);

        {
            // Reverse per-path index (path → variable names) — the last
            // O(workspace)-per-call entity type; everything else uses an index.
            // The variables map is Vec-valued, so check each declaration's path
            // + range after the index narrows to this file's names.
            let index = &self.data.variables_file_index;
            if let Some(names) = index.get(&index_key(path)) {
                for name in names.value() {
                    if let Some(vars) = self.data.variables.get(&**name) {
                        for var in vars.iter() {
                            // Normalized compare: var.path is a raw stored path
                            // (backslashes on Windows), `path` comes from
                            // to_file_path (forward slashes).
                            if path_matches(var.path.as_ref(), path)
                                && is_pos_in_range(pos, &var.range)
                            {
                                return Some((
                                    EntityKind::Variable,
                                    var.range.clone(),
                                    name.to_string(),
                                ));
                            }
                        }
                    }
                }
            }
        }

        {
            let index = &self.data.arrays_file_index;
            if let Some(names) = index.get(&index_key(path)) {
                for name in names.value() {
                    if let Some(arrs) = self.data.arrays.get(&**name) {
                        for arr in arrs.iter() {
                            if path_matches(arr.path.as_ref(), path)
                                && is_pos_in_range(pos, &arr.range)
                            {
                                return Some((
                                    EntityKind::Array,
                                    arr.range.clone(),
                                    name.to_string(),
                                ));
                            }
                        }
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

        // Decision categories — derived from decisions DashMap
        {
            let map = &self.data.decisions;
            for entry in map.iter() {
                let cat = entry.value().resolve().category.clone();
                names.insert(cat, EntityKind::DecisionCategory);
            }
        }

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
        // Tag aliases come from common/country_tag_aliases/*.txt and are
        // functionally country tags (they resolve to a country at runtime).
        // Without them, semantic tokens classify ASP/IMR/etc. as plain
        // strings/localization tokens and skip the country-tag token type.
        collect_names!(CountryTag, tag_aliases);
        collect_names!(OobDivisionTemplate, oob_division_templates);
        collect_names!(OobFleet, oob_fleets);
        collect_names!(EventNamespace, event_namespaces);
        collect_names!(TechnologyTag, technology_tags);

        // Variables and arrays — user-defined names from `set_variable` /
        // `add_to_array`. They resolve as `Variable`/`Array` token types so
        // `var = my_var` and `array = my_array` highlight distinct from
        // plain strings. Builtins are added in `Backend::update_entity_token_context`
        // so they also appear here without needing a scan.
        for entry in self.data.variables.iter() {
            names.insert(entry.key().to_string(), EntityKind::Variable);
        }
        for entry in self.data.arrays.iter() {
            names.insert(entry.key().to_string(), EntityKind::Array);
        }

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
        push_symbols!(Technology, technologies, "Technology");
        push_symbols!(TechnologyTag, technology_tags, "Technology Tag");

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
        push_symbols!(AceModifier, ace_modifiers, "Ace Modifier");
        push_symbols!(Portrait, portraits, "Portrait");
        push_symbols!(ColorCode, color_codes, "Color Code");
        push_symbols!(Decision, decisions, "Decision");
        push_symbols!(TerrainCategory, terrain_categories, "Terrain Category");
        push_symbols!(CountryTag, country_tags, "Country Tag");
        push_symbols!(Building, buildings, "Building");
        push_symbols!(AiStrategyPlan, ai_strategy_plans, "AI Strategy Plan");
        push_symbols!(AiArea, ai_areas, "AI Area");
        push_symbols!(BalanceOfPower, balance_of_powers, "Balance of Power");
        push_symbols!(UnitType, unit_types, "Unit Type");

        results
    }
}

// ---------------------------------------------------------------------------
// SECTION - Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::interner::InternedStr;
    use crate::parser::ast;
    use crate::scanner::province_scanner::Province;
    use crate::scanner::state_scanner::State;
    use crate::scanner::strategic_region_scanner::StrategicRegion;

    fn range(line: u32) -> ast::Range {
        ast::Range {
            start_line: line,
            start_col: 0,
            end_line: line,
            end_col: 5,
        }
    }

    fn lookup_with_state() -> (EntityLookup<'static>, InternedStr) {
        let data = Box::leak(Box::new(ScannerData::new()));
        let path: InternedStr = InternedStr::from("history/states/422.txt");
        data.states.insert(
            422,
            State {
                id: 422,
                name: "STATE_422".to_string(),
                provinces: vec![10928, 10435],
                path: path.clone(),
                range: range(0),
            },
        );
        data.strategic_regions.insert(
            12,
            StrategicRegion {
                id: 12,
                name: "STRATEGICREGION_12".to_string(),
                provinces: vec![422],
                weather: None,
                naval_terrain: None,
                path: InternedStr::from("map/strategicregions/12.txt"),
                range: range(0),
            },
        );
        data.provinces.insert(
            422,
            Province {
                id: 422,
                rgb: (255, 0, 0),
                terrain: "plains".to_string(),
                is_coastal: false,
                prov_type: "land".to_string(),
                continent: 1,
                path: InternedStr::from("map/definition.csv"),
                range: ast::Range {
                    start_line: 422,
                    start_col: 0,
                    end_line: 422,
                    end_col: 30,
                },
            },
        );
        (EntityLookup { data }, path)
    }

    /// `state = 422` resolves to the state definition.
    #[test]
    fn test_numeric_state_resolves() {
        let (lookup, state_path) = lookup_with_state();
        let locations = lookup.find_numeric_definition(422, "state", Some("state"));
        assert_eq!(
            locations.len(),
            1,
            "state=422 should resolve to exactly one location"
        );
        assert_eq!(locations[0].kind, EntityKind::State);
        assert_eq!(locations[0].path, state_path);
    }

    /// A bare number inside a state-context block (e.g. `any_state_of = { 422 }`)
    /// resolves via the number-parse guard.
    #[test]
    fn test_bare_number_in_state_context_resolves() {
        let (lookup, _) = lookup_with_state();
        let locations = lookup.find_numeric_definition(422, "422", Some("any_state_of"));
        assert_eq!(
            locations.len(),
            1,
            "bare 422 in state context should resolve"
        );
        assert_eq!(locations[0].kind, EntityKind::State);
    }

    /// `strategic_region = 12` resolves to the region definition.
    #[test]
    fn test_numeric_strategic_region_resolves() {
        let (lookup, _) = lookup_with_state();
        let locations =
            lookup.find_numeric_definition(12, "strategic_region", Some("strategic_region"));
        assert_eq!(locations.len(), 1, "strategic_region=12 should resolve");
        assert_eq!(locations[0].kind, EntityKind::StrategicRegion);
    }

    /// `province = 422` resolves to the province's line in definition.csv.
    #[test]
    fn test_numeric_province_resolves() {
        let (lookup, _) = lookup_with_state();
        let locations = lookup.find_numeric_definition(422, "province", Some("province"));
        assert_eq!(locations.len(), 1, "province=422 should resolve");
        assert_eq!(locations[0].kind, EntityKind::Province);
        assert_eq!(locations[0].path, InternedStr::from("map/definition.csv"));
        assert_eq!(
            locations[0].range.start_line, 422,
            "range must point at the definition.csv line"
        );
    }

    /// `victory_points = { 422 3 }`-style bare number in a province-ish block
    /// also resolves via the number-parse guard.
    #[test]
    fn test_victory_points_province_resolves() {
        let (lookup, _) = lookup_with_state();
        let locations = lookup.find_numeric_definition(422, "422", Some("victory_points"));
        assert_eq!(
            locations.len(),
            1,
            "bare 422 in victory_points context should resolve"
        );
        assert_eq!(locations[0].kind, EntityKind::Province);
    }

    /// A bare number in a *non*-state key must NOT jump to a state with the
    /// same id — guards must keep unrelated numeric keys out.
    #[test]
    fn test_unrelated_numeric_key_does_not_resolve() {
        let (lookup, _) = lookup_with_state();
        // `factor = 422` — the id exists as a state, but the key is unrelated.
        let locations = lookup.find_numeric_definition(422, "factor", Some("factor"));
        assert!(
            locations.is_empty(),
            "a bare number under an unrelated key must not resolve to a state"
        );
    }

    /// `capital = 422` (a state reference) resolves; `add_core_of` too.
    #[test]
    fn test_capital_and_core_keys_resolve() {
        let (lookup, _) = lookup_with_state();
        assert_eq!(
            lookup
                .find_numeric_definition(422, "capital", Some("capital"))
                .len(),
            1,
            "capital=422 should resolve to a state"
        );
        assert_eq!(
            lookup
                .find_numeric_definition(422, "add_core_of", Some("add_core_of"))
                .len(),
            1,
            "add_core_of=422 should resolve to a state"
        );
    }

    /// End-to-end: replicate the goto_definition resolution for the real
    /// snippet shape (`highlight_states_trigger = { state = 422 }` inside a
    /// decision). `find_identifier_at` must report "state" with the number in
    /// `assigned_value`, and the string lookup must stay empty so the numeric
    /// fallback fires.
    #[test]
    fn test_goto_path_state_nested_in_decision() {
        use crate::scope::scope::ScopeStack;
        use crate::utils::symbol_search::find_identifier_at;
        use tower_lsp_server::ls_types::Position;

        let content = "X = {\n\thighlight_states_trigger = { state = 422 }\n}\n";
        let (script, _) = crate::parser::parser::parse_script(content);
        let uri = "/common/decisions/fke_decisions.txt".to_string();

        let (lookup, _) = lookup_with_state();
        let data = lookup.data;

        let mut scope_stack = ScopeStack::new(crate::scope::scope::initial_scope_for_uri(&uri));
        let sctx = crate::scope::scope::ScopeCtx {
            uri: &uri,
            event_targets: Some(&data.event_targets),
            characters: Some(&data.characters),
            achievements: Some(&data.achievements),
            in_random_list: false,
            state_targeted: false,
        };
        let line = 1u32;
        let col = content.lines().nth(1).unwrap().find("422").unwrap() as u32;
        let res = find_identifier_at(
            &script,
            Position {
                line,
                character: col,
            },
            &mut scope_stack,
            &sctx,
        );
        let (identifier, _, assigned_value, context_key) = res.expect("identifier at cursor");
        assert_eq!(identifier, "state", "identifier must be the key text");
        let mut locations = lookup.find_definition(&identifier);
        assert!(
            locations.is_empty(),
            "string lookup must stay empty for 'state' so the numeric fallback fires"
        );
        if let Some(ast::Value::Number(n)) = &assigned_value {
            locations.extend(lookup.find_numeric_definition(
                *n as u32,
                &identifier,
                context_key.as_deref(),
            ));
        }
        assert_eq!(locations.len(), 1, "goto must resolve state 422");
        assert_eq!(locations[0].kind, EntityKind::State);
    }
}

// ---------------------------------------------------------------------------
// !SECTION
// ---------------------------------------------------------------------------
