use crate::ability_scanner;
use crate::achievement_scanner;
use crate::ai_area_scanner;
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
use crate::modifier_scanner;
use crate::music_scanner;
use crate::parser;
use crate::portrait_scanner;
use crate::scanner_data::ScannerData;
use crate::scripted_loc_scanner;
use crate::scripted_scanner;
use crate::sound_scanner;
use crate::sprite_scanner;
use crate::strategic_region_scanner;
use crate::trait_scanner;
use crate::variable_scanner;
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Determine whether a stored scanner path matches a target absolute path.
/// Stored paths may be relative (`./events/x.txt`) or absolute.
fn path_matches(stored: &str, target: &str) -> bool {
    if stored == target {
        return true;
    }
    let normalized = stored.strip_prefix("./").unwrap_or(stored);
    normalized == target || target.ends_with(normalized)
}

trait HasPath {
    fn path(&self) -> &str;
}

macro_rules! impl_has_path {
    ($ty:ty) => {
        impl HasPath for $ty {
            fn path(&self) -> &str {
                &self.path
            }
        }
    };
}

impl_has_path!(loc_parser::LocEntry);
impl_has_path!(event_scanner::Event);
impl_has_path!(scripted_scanner::ScriptedEntity);
impl_has_path!(scripted_loc_scanner::ScriptedLoc);
impl_has_path!(achievement_scanner::Achievement);
impl_has_path!(modifier_scanner::Modifier);
impl_has_path!(ideology_scanner::Ideology);
impl_has_path!(trait_scanner::Trait);
impl_has_path!(idea_scanner::Idea);
impl_has_path!(character_scanner::Character);
impl_has_path!(building_scanner::Building);
impl_has_path!(ability_scanner::Ability);
impl_has_path!(ai_strategy_plan_scanner::AiStrategyPlan);
impl_has_path!(ai_area_scanner::AiArea);
impl_has_path!(music_scanner::MusicAsset);
impl_has_path!(music_scanner::MusicStation);
impl_has_path!(music_scanner::Song);
impl_has_path!(sound_scanner::Sound);
impl_has_path!(sound_scanner::SoundEffect);
impl_has_path!(sound_scanner::Falloff);
impl_has_path!(sound_scanner::SoundCategory);
impl_has_path!(portrait_scanner::Portrait);
impl_has_path!(sprite_scanner::Sprite);
impl_has_path!(variable_scanner::Variable);
impl_has_path!(variable_scanner::EventTarget);
impl_has_path!(strategic_region_scanner::StrategicRegion);
impl_has_path!(country_scanner::CountryTag);

/// Determines which scanner categories apply to a given file path.
fn classify_file(path: &str) -> Vec<FileCategory> {
    let lower = path.to_ascii_lowercase();
    let mut cats = Vec::new();

    if lower.ends_with(".yml") && lower.contains("localisation") {
        cats.push(FileCategory::Localization);
    }

    if lower.ends_with(".txt") {
        // Order matters: more specific paths should match before "any .txt" fallbacks
        if lower.contains("/events/") || lower.contains("\\events\\") {
            cats.push(FileCategory::Events);
        }
        if lower.contains("/common/scripted_triggers/")
            || lower.contains("\\common\\scripted_triggers\\")
        {
            cats.push(FileCategory::ScriptedTriggers);
        }
        if lower.contains("/common/scripted_effects/")
            || lower.contains("\\common\\scripted_effects\\")
        {
            cats.push(FileCategory::ScriptedEffects);
        }
        if lower.contains("/common/scripted_localisation/")
            || lower.contains("\\common\\scripted_localisation\\")
        {
            cats.push(FileCategory::ScriptedLocalisation);
        }
        if lower.contains("/common/achievements/") || lower.contains("\\common\\achievements\\") {
            cats.push(FileCategory::Achievements);
        }
        if lower.contains("/common/modifiers/")
            || lower.contains("\\common\\modifiers\\")
            || lower.contains("/common/dynamic_modifiers/")
            || lower.contains("\\common\\dynamic_modifiers\\")
        {
            cats.push(FileCategory::Modifiers);
        }
        if lower.contains("/common/ideologies/") || lower.contains("\\common\\ideologies\\") {
            cats.push(FileCategory::Ideologies);
        }
        if lower.contains("/common/unit_leader/") || lower.contains("\\common\\unit_leader\\") {
            cats.push(FileCategory::UnitLeaderTraits);
        }
        if lower.contains("/common/country_leader/") || lower.contains("\\common\\country_leader\\")
        {
            cats.push(FileCategory::CountryLeaderTraits);
        }
        if lower.contains("/common/traits/") || lower.contains("\\common\\traits\\") {
            cats.push(FileCategory::Traits);
        }
        if lower.contains("/common/ideas/") || lower.contains("\\common\\ideas\\") {
            cats.push(FileCategory::Ideas);
        }
        if lower.contains("/common/characters/") || lower.contains("\\common\\characters\\") {
            cats.push(FileCategory::Characters);
        }
        if lower.contains("/common/buildings/") || lower.contains("\\common\\buildings\\") {
            cats.push(FileCategory::Buildings);
        }
        if lower.contains("/common/abilities/") || lower.contains("\\common\\abilities\\") {
            cats.push(FileCategory::Abilities);
        }
        if lower.contains("/common/ai_strategy_plans/")
            || lower.contains("\\common\\ai_strategy_plans\\")
        {
            cats.push(FileCategory::AiStrategyPlans);
        }
        if lower.contains("/common/ai_areas/") || lower.contains("\\common\\ai_areas\\") {
            cats.push(FileCategory::AiAreas);
        }
        if lower.contains("/common/defines/") || lower.contains("\\common\\defines\\") {
            cats.push(FileCategory::Defines);
        }
        if lower.contains("/common/country_tags/")
            || lower.contains("\\common\\country_tags\\")
            || lower.contains("/common/countries/")
            || lower.contains("\\common\\countries\\")
            || lower.contains("/history/countries/")
            || lower.contains("\\history\\countries\\")
        {
            cats.push(FileCategory::Countries);
        }

        // Interface .gfx sprites
        if lower.contains("/interface/") || lower.contains("\\interface\\") {
            cats.push(FileCategory::Sprites);
        }

        // Portraits
        if lower.contains("/portraits/") || lower.contains("\\portraits\\") {
            cats.push(FileCategory::Portraits);
        }

        // Strategic regions
        if lower.contains("/common/strategic_regions/")
            || lower.contains("\\common\\strategic_regions\\")
        {
            cats.push(FileCategory::StrategicRegions);
        }

        cats.push(FileCategory::Variables);
    }

    if lower.ends_with(".asset") {
        if lower.contains("/music/") || lower.contains("\\music\\") {
            cats.push(FileCategory::MusicAssets);
        }
        if lower.contains("/sound/") || lower.contains("\\sound\\") {
            cats.push(FileCategory::Sounds);
        }
    }

    if lower.ends_with(".lua")
        && (lower.contains("/common/defines/") || lower.contains("\\common\\defines\\"))
    {
        cats.push(FileCategory::Defines);
    }

    if lower.ends_with(".gfx") && (lower.contains("/interface/") || lower.contains("\\interface\\"))
    {
        cats.push(FileCategory::Sprites);
    }

    cats
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FileCategory {
    Localization,
    Events,
    ScriptedTriggers,
    ScriptedEffects,
    ScriptedLocalisation,
    Achievements,
    Modifiers,
    Ideologies,
    UnitLeaderTraits,
    CountryLeaderTraits,
    Traits,
    Ideas,
    Characters,
    Buildings,
    Abilities,
    AiStrategyPlans,
    AiAreas,
    Defines,
    Countries,
    Variables,
    MusicAssets,
    Sounds,
    Portraits,
    Sprites,
    StrategicRegions,
}

/// Update `ScannerData` with fresh entities extracted from a single saved file.
/// Parses raw content, then delegates to AST-based update helpers.
pub fn update_scanner_data_for_file(scanner_data: &ScannerData, path_str: &str, content: &str) {
    let categories = classify_file(path_str);

    for category in categories {
        match category {
            FileCategory::Localization => update_localization(scanner_data, path_str, content),
            FileCategory::Defines => update_defines(scanner_data, content),
            FileCategory::Countries => update_country_tags(scanner_data, path_str, content),
            _ => {
                let (script, _parse_errors) = parser::parse_script(content);
                update_from_ast(scanner_data, path_str, &script, category);
            }
        }
    }
}

/// Update `ScannerData` from an already-parsed AST (used by `did_change` live updates).
/// Skips categories that need raw text (localization, defines, country tags).
pub fn update_scanner_data_from_ast(
    scanner_data: &ScannerData,
    path_str: &str,
    script: &ast::Script,
) {
    let categories = classify_file(path_str);

    for category in categories {
        match category {
            FileCategory::Localization | FileCategory::Defines | FileCategory::Countries => {
                // These need raw text parsing — skip in live AST-based update
            }
            _ => update_from_ast(scanner_data, path_str, script, category),
        }
    }
}

/// Dispatch a single category to the appropriate AST-based helper.
fn update_from_ast(
    scanner_data: &ScannerData,
    path_str: &str,
    script: &ast::Script,
    category: FileCategory,
) {
    match category {
        FileCategory::Events => update_events(scanner_data, path_str, script),
        FileCategory::ScriptedTriggers => {
            update_scripted(scanner_data, "triggers", path_str, script)
        }
        FileCategory::ScriptedEffects => update_scripted(scanner_data, "effects", path_str, script),
        FileCategory::ScriptedLocalisation => update_scripted_locs(scanner_data, path_str, script),
        FileCategory::Achievements => update_achievements(scanner_data, path_str, script),
        FileCategory::Modifiers => update_modifiers(scanner_data, path_str, script),
        FileCategory::Ideologies => update_ideologies(scanner_data, path_str, script),
        FileCategory::UnitLeaderTraits => {
            update_traits(scanner_data, "Unit Leader Trait", path_str, script)
        }
        FileCategory::CountryLeaderTraits => {
            update_traits(scanner_data, "Country Leader Trait", path_str, script)
        }
        FileCategory::Traits => update_traits(scanner_data, "Trait", path_str, script),
        FileCategory::Ideas => update_ideas(scanner_data, path_str, script),
        FileCategory::Characters => update_characters(scanner_data, path_str, script),
        FileCategory::Buildings => update_buildings(scanner_data, path_str, script),
        FileCategory::Abilities => update_abilities(scanner_data, path_str, script),
        FileCategory::AiStrategyPlans => update_ai_strategy_plans(scanner_data, path_str, script),
        FileCategory::AiAreas => update_ai_areas(scanner_data, path_str, script),
        FileCategory::Variables => update_variables(scanner_data, path_str, script),
        FileCategory::MusicAssets => update_music_asset(scanner_data, path_str, script),
        FileCategory::Sounds => update_sounds(scanner_data, path_str, script),
        FileCategory::Portraits => update_portraits(scanner_data, path_str, script),
        FileCategory::Sprites => update_sprites(scanner_data, path_str, script),
        FileCategory::StrategicRegions => update_strategic_regions(scanner_data, path_str, script),
        FileCategory::Localization | FileCategory::Defines | FileCategory::Countries => {
            // Handled directly in update_scanner_data_for_file, unreachable here
        }
    }
}

// ── Per-category update helpers ──
//
// Each helper now operates directly on DashMap fields — no cloning of the
// entire registry. retain() removes old entries, insert() adds new ones,
// all without allocating a full snapshot.

fn update_localization(scanner_data: &ScannerData, path_str: &str, content: &str) {
    let (parsed, _, _) = loc_parser::parse_loc_file(content, path_str);

    scanner_data
        .localization
        .retain(|_, v| !path_matches(&v.path, path_str));
    for (key, entry) in parsed {
        scanner_data.localization.insert(key, entry);
    }

    // Rebuild duplicated_loc_keys from scratch (cheapest after a loc change)
    let mut seen: HashSet<(String, String)> = HashSet::new();
    let mut dups = HashSet::new();
    for entry in scanner_data.localization.iter() {
        let pair = (entry.value().path.clone(), entry.value().key.clone());
        if seen.contains(&pair) {
            dups.insert(pair);
        } else {
            seen.insert(pair);
        }
    }
    scanner_data.duplicated_loc_keys.clear();
    for dup in dups {
        scanner_data.duplicated_loc_keys.insert(dup);
    }
}

fn update_events(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_events = HashMap::new();
    event_scanner::find_event_definitions(&script.entries, path_str, &mut new_events);

    scanner_data
        .events
        .retain(|_, v| !path_matches(v.path(), path_str));
    for (key, event) in new_events {
        scanner_data.events.insert(key, event);
    }
}

fn update_scripted(scanner_data: &ScannerData, kind: &str, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    for entry_ast in script.entries.iter() {
        if let ast::Entry::Assignment(ass) = entry_ast {
            new_entries.insert(
                ass.key.clone(),
                scripted_scanner::ScriptedEntity {
                    name: ass.key.clone(),
                    path: path_str.to_string(),
                    range: ass.key_range.clone(),
                },
            );
        }
    }

    match kind {
        "triggers" => {
            scanner_data
                .scripted_triggers
                .retain(|_, v| !path_matches(v.path(), path_str));
            for (k, v) in new_entries {
                scanner_data.scripted_triggers.insert(k, v);
            }
        }
        "effects" => {
            scanner_data
                .scripted_effects
                .retain(|_, v| !path_matches(v.path(), path_str));
            for (k, v) in new_entries {
                scanner_data.scripted_effects.insert(k, v);
            }
        }
        _ => {}
    }
}

fn update_scripted_locs(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    scripted_loc_scanner::find_scripted_locs_in_entries(
        &script.entries,
        path_str,
        &mut new_entries,
    );

    scanner_data
        .scripted_locs
        .retain(|_, v| !path_matches(v.path(), path_str));
    for (k, v) in new_entries {
        scanner_data.scripted_locs.insert(k, v);
    }
}

fn update_achievements(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    achievement_scanner::find_achievements_in_entries(&script.entries, path_str, &mut new_entries);

    scanner_data
        .achievements
        .retain(|_, v| !path_matches(v.path(), path_str));
    for (k, v) in new_entries {
        scanner_data.achievements.insert(k, v);
    }
}

fn update_modifiers(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    for entry_ast in script.entries.iter() {
        if let ast::Entry::Assignment(ass) = entry_ast {
            new_entries.insert(
                ass.key.clone(),
                modifier_scanner::Modifier {
                    name: ass.key.clone(),
                    path: path_str.to_string(),
                    range: ass.key_range.clone(),
                },
            );
        }
    }

    scanner_data
        .custom_modifiers
        .retain(|_, v| !path_matches(v.path(), path_str));
    for (k, v) in new_entries {
        scanner_data.custom_modifiers.insert(k, v);
    }
}

fn update_ideologies(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    ideology_scanner::find_ideologies_in_entries(&script.entries, path_str, &mut new_entries);

    let mut sub_map = HashMap::new();
    for ideology in new_entries.values() {
        for (sub, range) in &ideology.sub_ideology_ranges {
            sub_map.insert(
                sub.clone(),
                (ideology.name.clone(), range.clone(), ideology.path.clone()),
            );
        }
    }

    scanner_data
        .ideologies
        .retain(|_, v| !path_matches(v.path(), path_str));
    for (k, v) in new_entries {
        scanner_data.ideologies.insert(k, v);
    }

    scanner_data
        .sub_ideologies
        .retain(|_, v| !path_matches(&v.2, path_str));
    for (k, v) in sub_map {
        scanner_data.sub_ideologies.insert(k, v);
    }
}

fn update_traits(
    scanner_data: &ScannerData,
    trait_type: &str,
    path_str: &str,
    script: &ast::Script,
) {
    let mut new_entries = HashMap::new();
    trait_scanner::find_traits_in_entries(&script.entries, path_str, trait_type, &mut new_entries);

    scanner_data
        .traits
        .retain(|_, v| !path_matches(v.path(), path_str));
    for (k, v) in new_entries {
        scanner_data.traits.insert(k, v);
    }
}

fn update_ideas(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    idea_scanner::find_ideas_in_entries(&script.entries, path_str, &mut new_entries);

    scanner_data
        .ideas
        .retain(|_, v| !path_matches(v.path(), path_str));
    for (k, v) in new_entries {
        scanner_data.ideas.insert(k, v);
    }
}

fn update_characters(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    character_scanner::find_characters_in_entries(&script.entries, path_str, &mut new_entries);

    scanner_data
        .characters
        .retain(|_, v| !path_matches(v.path(), path_str));
    for (k, v) in new_entries {
        scanner_data.characters.insert(k, v);
    }
}

fn update_buildings(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    let path = Path::new(path_str);
    building_scanner::extract_buildings(&script.entries, path, &mut new_entries);

    scanner_data
        .buildings
        .retain(|_, v| !path_matches(v.path(), path_str));
    for (k, v) in new_entries {
        scanner_data.buildings.insert(k, v);
    }
}

fn update_abilities(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    ability_scanner::find_abilities_in_entries(&script.entries, path_str, &mut new_entries);

    scanner_data
        .abilities
        .retain(|_, v| !path_matches(v.path(), path_str));
    for (k, v) in new_entries {
        scanner_data.abilities.insert(k, v);
    }
}

fn update_ai_strategy_plans(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    let path = Path::new(path_str);
    ai_strategy_plan_scanner::extract_plans(&script.entries, path, &mut new_entries);

    scanner_data
        .ai_strategy_plans
        .retain(|_, v| !path_matches(v.path(), path_str));
    for (k, v) in new_entries {
        scanner_data.ai_strategy_plans.insert(k, v);
    }
}

fn update_ai_areas(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    let path = Path::new(path_str);
    ai_area_scanner::extract_areas(&script.entries, path, &mut new_entries);

    scanner_data
        .ai_areas
        .retain(|_, v| !path_matches(v.path(), path_str));
    for (k, v) in new_entries {
        scanner_data.ai_areas.insert(k, v);
    }
}

fn update_defines(scanner_data: &ScannerData, content: &str) {
    let mut new_defines = defines_parser::GameDefines::new();
    defines_parser::parse_defines_lua(content, &mut new_defines);

    let mut defines = (*scanner_data.defines()).clone();
    defines
        .max_skill_levels
        .extend(new_defines.max_skill_levels);
    defines.defines.extend(new_defines.defines);
    scanner_data.set_defines(defines);
}

fn update_country_tags(scanner_data: &ScannerData, path_str: &str, content: &str) {
    let mut new_entries = HashMap::new();
    let lower = path_str.to_ascii_lowercase();

    if lower.contains("/common/country_tags/") || lower.contains("\\common\\country_tags\\") {
        // Line-by-line parsing: TAG = "countries/TAG - Name.txt"
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some(eq_pos) = trimmed.find('=') {
                let tag = trimmed[..eq_pos].trim();
                if !country_scanner::is_valid_tag(tag) {
                    continue;
                }
                let name = country_scanner::extract_country_name(
                    trimmed[eq_pos + 1..].trim().trim_matches('"'),
                );
                new_entries.insert(
                    tag.to_string(),
                    country_scanner::CountryTag {
                        tag: tag.to_string(),
                        name,
                        path: path_str.to_string(),
                        range: ast::Range {
                            start_line: 0,
                            start_col: 0,
                            end_line: 0,
                            end_col: 0,
                        },
                        dynamic: false,
                    },
                );
            }
        }
    } else if lower.contains("/common/countries/")
        || lower.contains("\\common\\countries\\")
        || lower.contains("/history/countries/")
        || lower.contains("\\history\\countries\\")
    {
        // Filename-based: TAG - Name.txt
        let path = Path::new(path_str);
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            if let Some(tag) = stem.split_whitespace().next() {
                if country_scanner::is_valid_tag(tag) {
                    let name = country_scanner::extract_country_name(stem);
                    new_entries.insert(
                        tag.to_string(),
                        country_scanner::CountryTag {
                            tag: tag.to_string(),
                            name,
                            path: path_str.to_string(),
                            range: ast::Range {
                                start_line: 0,
                                start_col: 0,
                                end_line: 0,
                                end_col: 0,
                            },
                            dynamic: false,
                        },
                    );
                }
            }
        }
    }

    scanner_data
        .country_tags
        .retain(|_, v| !path_matches(v.path(), path_str));
    for (k, v) in new_entries {
        scanner_data.country_tags.insert(k, v);
    }
}

fn update_variables(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_vars: HashMap<String, Vec<variable_scanner::Variable>> = HashMap::new();
    let mut new_targets: HashMap<String, Vec<variable_scanner::EventTarget>> = HashMap::new();
    variable_scanner::scan_entries(&script.entries, path_str, &mut new_vars, &mut new_targets);

    // Remove old variable entries from this path, keeping others untouched.
    // Then insert new ones — DashMap entry API lets us append to a Vec value.
    scanner_data.variables.retain(|_, vec| {
        vec.retain(|v| !path_matches(v.path(), path_str));
        !vec.is_empty()
    });
    for (k, mut v) in new_vars {
        scanner_data.variables.entry(k).or_default().append(&mut v);
    }

    scanner_data.event_targets.retain(|_, vec| {
        vec.retain(|t| !path_matches(t.path(), path_str));
        !vec.is_empty()
    });
    for (k, mut v) in new_targets {
        scanner_data
            .event_targets
            .entry(k)
            .or_default()
            .append(&mut v);
    }
}

fn update_music_asset(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let ext = Path::new(path_str)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    if ext == "asset" {
        let mut assets = HashMap::new();
        music_scanner::find_assets_in_entries(&script.entries, path_str, &mut assets);

        scanner_data
            .music_assets
            .retain(|_, v| !path_matches(v.path(), path_str));
        for (k, v) in assets {
            scanner_data.music_assets.insert(k, v);
        }
    } else if ext == "txt" {
        let mut stations = HashMap::new();
        let mut songs = HashMap::new();
        music_scanner::find_stations_and_songs_in_entries(
            &script.entries,
            path_str,
            &mut stations,
            &mut songs,
        );

        scanner_data
            .music_stations
            .retain(|_, v| !path_matches(v.path(), path_str));
        for (k, v) in stations {
            scanner_data.music_stations.insert(k, v);
        }

        scanner_data
            .songs
            .retain(|_, v| !path_matches(v.path(), path_str));
        for (k, v) in songs {
            scanner_data.songs.insert(k, v);
        }
    }
}

fn update_sounds(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut sounds = HashMap::new();
    let mut effects = HashMap::new();
    let mut falloffs = HashMap::new();
    let mut categories = HashMap::new();
    sound_scanner::find_sound_definitions(
        &script.entries,
        path_str,
        &mut sounds,
        &mut effects,
        &mut falloffs,
        &mut categories,
    );

    scanner_data
        .sounds
        .retain(|_, v| !path_matches(v.path(), path_str));
    for (k, v) in sounds {
        scanner_data.sounds.insert(k, v);
    }

    scanner_data
        .sound_effects
        .retain(|_, v| !path_matches(v.path(), path_str));
    for (k, v) in effects {
        scanner_data.sound_effects.insert(k, v);
    }

    scanner_data
        .falloffs
        .retain(|_, v| !path_matches(v.path(), path_str));
    for (k, v) in falloffs {
        scanner_data.falloffs.insert(k, v);
    }

    scanner_data
        .sound_categories
        .retain(|_, v| !path_matches(v.path(), path_str));
    for (k, v) in categories {
        scanner_data.sound_categories.insert(k, v);
    }
}

fn update_portraits(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    let path = Path::new(path_str);
    portrait_scanner::extract_portraits(&script.entries, path, &mut new_entries);

    scanner_data
        .portraits
        .retain(|_, v| !path_matches(v.path(), path_str));
    for (k, v) in new_entries {
        scanner_data.portraits.insert(k, v);
    }
}

fn update_sprites(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    sprite_scanner::find_sprites_in_entries(&script.entries, path_str, &mut new_entries);

    scanner_data
        .sprites
        .retain(|_, v| !path_matches(v.path(), path_str));
    for (k, v) in new_entries {
        scanner_data.sprites.insert(k, v);
    }
}

fn update_strategic_regions(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries: HashMap<u32, strategic_region_scanner::StrategicRegion> = HashMap::new();
    strategic_region_scanner::extract_strategic_region(
        &script.entries,
        Path::new(path_str),
        &mut new_entries,
    );

    scanner_data
        .strategic_regions
        .retain(|_, v| !path_matches(&v.path, path_str));
    for (k, v) in new_entries {
        scanner_data.strategic_regions.insert(k, v);
    }
}
