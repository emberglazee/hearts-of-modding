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

/// Remove entries from a `HashMap<String, T>` where the entry's path matches.
fn remove_by_path<T>(map: &mut HashMap<String, T>, file_path: &str)
where
    T: HasPath,
{
    map.retain(|_, v| !path_matches(v.path(), file_path));
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

fn update_localization(scanner_data: &ScannerData, path_str: &str, content: &str) {
    let (parsed, _, _) = loc_parser::parse_loc_file(content, path_str);

    let mut loc = (*scanner_data.localization()).clone();
    loc.retain(|_, v| !path_matches(&v.path, path_str));
    for (key, entry) in parsed {
        loc.insert(key, entry);
    }
    scanner_data.set_localization(loc);

    // Rebuild duplicated_loc_keys from scratch (cheapest after a loc change)
    let all_loc = scanner_data.localization();
    let mut seen: HashSet<(String, String)> = HashSet::new();
    let mut dups = HashSet::new();
    for (_key, entry) in all_loc.iter() {
        let pair = (entry.path.clone(), entry.key.clone());
        if seen.contains(&pair) {
            dups.insert((entry.path.clone(), entry.key.clone()));
        } else {
            seen.insert(pair);
        }
    }
    scanner_data.set_duplicated_loc_keys(dups);
}

fn update_events(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_events = HashMap::new();
    event_scanner::find_event_definitions(&script.entries, path_str, &mut new_events);

    let mut events = (*scanner_data.events()).clone();
    remove_by_path(&mut events, path_str);
    for (key, event) in new_events {
        events.insert(key, event);
    }
    scanner_data.set_events(events);
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
            let mut map = (*scanner_data.scripted_triggers()).clone();
            remove_by_path(&mut map, path_str);
            for (k, v) in new_entries {
                map.insert(k, v);
            }
            scanner_data.set_scripted_triggers(map);
        }
        "effects" => {
            let mut map = (*scanner_data.scripted_effects()).clone();
            remove_by_path(&mut map, path_str);
            for (k, v) in new_entries {
                map.insert(k, v);
            }
            scanner_data.set_scripted_effects(map);
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

    let mut map = (*scanner_data.scripted_locs()).clone();
    remove_by_path(&mut map, path_str);
    for (k, v) in new_entries {
        map.insert(k, v);
    }
    scanner_data.set_scripted_locs(map);
}

fn update_achievements(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    achievement_scanner::find_achievements_in_entries(&script.entries, path_str, &mut new_entries);

    let mut map = (*scanner_data.achievements()).clone();
    remove_by_path(&mut map, path_str);
    for (k, v) in new_entries {
        map.insert(k, v);
    }
    scanner_data.set_achievements(map);
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

    let mut map = (*scanner_data.custom_modifiers()).clone();
    remove_by_path(&mut map, path_str);
    for (k, v) in new_entries {
        map.insert(k, v);
    }
    scanner_data.set_custom_modifiers(map);
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

    let mut map = (*scanner_data.ideologies()).clone();
    remove_by_path(&mut map, path_str);
    for (k, v) in new_entries {
        map.insert(k, v);
    }
    scanner_data.set_ideologies(map);

    let mut s_map = (*scanner_data.sub_ideologies()).clone();
    s_map.retain(|_, v| !path_matches(&v.2, path_str));
    for (k, v) in sub_map {
        s_map.insert(k, v);
    }
    scanner_data.set_sub_ideologies(s_map);
}

fn update_traits(
    scanner_data: &ScannerData,
    trait_type: &str,
    path_str: &str,
    script: &ast::Script,
) {
    let mut new_entries = HashMap::new();
    trait_scanner::find_traits_in_entries(&script.entries, path_str, trait_type, &mut new_entries);

    let mut map = (*scanner_data.traits()).clone();
    remove_by_path(&mut map, path_str);
    for (k, v) in new_entries {
        map.insert(k, v);
    }
    scanner_data.set_traits(map);
}

fn update_ideas(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    idea_scanner::find_ideas_in_entries(&script.entries, path_str, &mut new_entries);

    let mut map = (*scanner_data.ideas()).clone();
    remove_by_path(&mut map, path_str);
    for (k, v) in new_entries {
        map.insert(k, v);
    }
    scanner_data.set_ideas(map);
}

fn update_characters(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    character_scanner::find_characters_in_entries(&script.entries, path_str, &mut new_entries);

    let mut map = (*scanner_data.characters()).clone();
    remove_by_path(&mut map, path_str);
    for (k, v) in new_entries {
        map.insert(k, v);
    }
    scanner_data.set_characters(map);
}

fn update_buildings(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    let path = Path::new(path_str);
    building_scanner::extract_buildings(&script.entries, path, &mut new_entries);

    let mut map = (*scanner_data.buildings()).clone();
    remove_by_path(&mut map, path_str);
    for (k, v) in new_entries {
        map.insert(k, v);
    }
    scanner_data.set_buildings(map);
}

fn update_abilities(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    ability_scanner::find_abilities_in_entries(&script.entries, path_str, &mut new_entries);

    let mut map = (*scanner_data.abilities()).clone();
    remove_by_path(&mut map, path_str);
    for (k, v) in new_entries {
        map.insert(k, v);
    }
    scanner_data.set_abilities(map);
}

fn update_ai_strategy_plans(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    let path = Path::new(path_str);
    ai_strategy_plan_scanner::extract_plans(&script.entries, path, &mut new_entries);

    let mut map = (*scanner_data.ai_strategy_plans()).clone();
    remove_by_path(&mut map, path_str);
    for (k, v) in new_entries {
        map.insert(k, v);
    }
    scanner_data.set_ai_strategy_plans(map);
}

fn update_ai_areas(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    let path = Path::new(path_str);
    ai_area_scanner::extract_areas(&script.entries, path, &mut new_entries);

    let mut map = (*scanner_data.ai_areas()).clone();
    remove_by_path(&mut map, path_str);
    for (k, v) in new_entries {
        map.insert(k, v);
    }
    scanner_data.set_ai_areas(map);
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

    let mut map = (*scanner_data.country_tags()).clone();
    remove_by_path(&mut map, path_str);
    for (k, v) in new_entries {
        map.insert(k, v);
    }
    scanner_data.set_country_tags(map);
}

fn update_variables(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_vars: HashMap<String, Vec<variable_scanner::Variable>> = HashMap::new();
    let mut new_targets: HashMap<String, Vec<variable_scanner::EventTarget>> = HashMap::new();
    variable_scanner::scan_entries(&script.entries, path_str, &mut new_vars, &mut new_targets);

    let mut vars = (*scanner_data.variables()).clone();
    remove_by_path_for_vec(&mut vars, path_str);
    for (k, mut v) in new_vars {
        vars.entry(k).or_default().append(&mut v);
    }
    scanner_data.set_variables(vars);

    let mut targets = (*scanner_data.event_targets()).clone();
    remove_by_path_for_vec(&mut targets, path_str);
    for (k, mut v) in new_targets {
        targets.entry(k).or_default().append(&mut v);
    }
    scanner_data.set_event_targets(targets);
}

fn remove_by_path_for_vec<T>(map: &mut HashMap<String, Vec<T>>, file_path: &str)
where
    T: HasPath,
{
    map.retain(|_, vec| {
        vec.retain(|v| !path_matches(v.path(), file_path));
        !vec.is_empty()
    });
}

fn update_music_asset(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let ext = Path::new(path_str)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    if ext == "asset" {
        let mut assets = HashMap::new();
        music_scanner::find_assets_in_entries(&script.entries, path_str, &mut assets);

        let mut map = (*scanner_data.music_assets()).clone();
        remove_by_path(&mut map, path_str);
        for (k, v) in assets {
            map.insert(k, v);
        }
        scanner_data.set_music_assets(map);
    } else if ext == "txt" {
        let mut stations = HashMap::new();
        let mut songs = HashMap::new();
        music_scanner::find_stations_and_songs_in_entries(
            &script.entries,
            path_str,
            &mut stations,
            &mut songs,
        );

        let mut s_map = (*scanner_data.music_stations()).clone();
        remove_by_path(&mut s_map, path_str);
        for (k, v) in stations {
            s_map.insert(k, v);
        }
        scanner_data.set_music_stations(s_map);

        let mut song_map = (*scanner_data.songs()).clone();
        remove_by_path(&mut song_map, path_str);
        for (k, v) in songs {
            song_map.insert(k, v);
        }
        scanner_data.set_songs(song_map);
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

    let mut s_map = (*scanner_data.sounds()).clone();
    remove_by_path(&mut s_map, path_str);
    for (k, v) in sounds {
        s_map.insert(k, v);
    }
    scanner_data.set_sounds(s_map);

    let mut e_map = (*scanner_data.sound_effects()).clone();
    remove_by_path(&mut e_map, path_str);
    for (k, v) in effects {
        e_map.insert(k, v);
    }
    scanner_data.set_sound_effects(e_map);

    let mut f_map = (*scanner_data.falloffs()).clone();
    remove_by_path(&mut f_map, path_str);
    for (k, v) in falloffs {
        f_map.insert(k, v);
    }
    scanner_data.set_falloffs(f_map);

    let mut c_map = (*scanner_data.sound_categories()).clone();
    remove_by_path(&mut c_map, path_str);
    for (k, v) in categories {
        c_map.insert(k, v);
    }
    scanner_data.set_sound_categories(c_map);
}

fn update_portraits(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    let path = Path::new(path_str);
    portrait_scanner::extract_portraits(&script.entries, path, &mut new_entries);

    let mut map = (*scanner_data.portraits()).clone();
    remove_by_path(&mut map, path_str);
    for (k, v) in new_entries {
        map.insert(k, v);
    }
    scanner_data.set_portraits(map);
}

fn update_sprites(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    sprite_scanner::find_sprites_in_entries(&script.entries, path_str, &mut new_entries);

    let mut map = (*scanner_data.sprites()).clone();
    remove_by_path(&mut map, path_str);
    for (k, v) in new_entries {
        map.insert(k, v);
    }
    scanner_data.set_sprites(map);
}

fn update_strategic_regions(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries: HashMap<u32, strategic_region_scanner::StrategicRegion> = HashMap::new();
    strategic_region_scanner::extract_strategic_region(
        &script.entries,
        Path::new(path_str),
        &mut new_entries,
    );

    let mut map = (*scanner_data.strategic_regions()).clone();
    // StrategicRegion is keyed by u32 id, not String — iterate and retain
    map.retain(|_, v| !path_matches(&v.path, path_str));
    for (k, v) in new_entries {
        map.insert(k, v);
    }
    scanner_data.set_strategic_regions(map);
}
