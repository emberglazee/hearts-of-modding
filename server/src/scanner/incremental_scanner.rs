use crate::data::interner::InternedStr;
use crate::data::scanner_data::ScannerData;
use crate::parser::ast;
use crate::parser::defines_parser;
use crate::parser::loc_parser;
use crate::parser::parser;
use crate::scanner::ability_scanner;
use crate::scanner::achievement_scanner;
use crate::scanner::ai_area_scanner;
use crate::scanner::ai_strategy_plan_scanner;
use crate::scanner::building_scanner;
use crate::scanner::character_scanner;
use crate::scanner::country_scanner;
use crate::scanner::event_scanner;
use crate::scanner::focus_scanner;
use crate::scanner::idea_scanner;
use crate::scanner::ideology_scanner;
use crate::scanner::modifier_scanner;
use crate::scanner::music_scanner;
use crate::scanner::portrait_scanner;
use crate::scanner::resource_scanner;
use crate::scanner::scripted_loc_scanner;
use crate::scanner::scripted_scanner;
use crate::scanner::sound_scanner;
use crate::scanner::sprite_scanner;
use crate::scanner::state_category_scanner;
use crate::scanner::strategic_region_scanner;
use crate::scanner::trait_scanner;
use crate::scanner::variable_scanner;
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

/// O(K) replacement for DashMap::retain + insert pattern using a reverse
/// file-path index. Removes old entries by looking up which keys were
/// defined in the changed file, then inserts new entries and updates the index.
///
/// Falls back to O(N) retain when the index has no entry for the path
/// (e.g., first incremental update before `rebuild_all_file_indices` runs,
/// or newly created files that weren't in the initial scan).
///
/// Two forms:
/// - 4-arg: values implement `HasPath` (uses `v.path()` for fallback retain)
/// - 5-arg: custom path accessor closure `|v| -> &str` for tuple types
macro_rules! retain_path {
    ($map:expr, $index:expr, $path:expr, $new_entries:expr $(,)?) => {{
        let p: &str = $path;
        if let Some((_, old_keys)) = $index.remove(p) {
            for key in old_keys {
                $map.remove(&key);
            }
        } else {
            let p_owned = p.to_owned();
            $map.retain(|_, v| !path_matches(v.path(), &p_owned));
        }
        let mut file_keys = Vec::with_capacity($new_entries.len());
        for (key, value) in $new_entries {
            let ik: InternedStr = std::sync::Arc::from(key.as_ref());
            $map.insert(ik.clone(), value);
            file_keys.push(ik);
        }
        $index.insert(std::sync::Arc::from(p), file_keys);
    }};
    ($map:expr, $index:expr, $path:expr, $new_entries:expr, $path_fn:expr $(,)?) => {{
        let p: &str = $path;
        if let Some((_, old_keys)) = $index.remove(p) {
            for key in old_keys {
                $map.remove(&key);
            }
        } else {
            let p_owned = p.to_owned();
            $map.retain(|_, v| !path_matches($path_fn(v), &p_owned));
        }
        let mut file_keys = Vec::with_capacity($new_entries.len());
        for (key, value) in $new_entries {
            let ik: InternedStr = std::sync::Arc::from(key.as_ref());
            $map.insert(ik.clone(), value);
            file_keys.push(ik);
        }
        $index.insert(std::sync::Arc::from(p), file_keys);
    }};
}

pub(crate) trait HasPath {
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
impl_has_path!(focus_scanner::Focus);
impl_has_path!(scripted_scanner::ScriptedEntity);
impl_has_path!(scripted_loc_scanner::ScriptedLoc);
impl_has_path!(achievement_scanner::Achievement);
impl_has_path!(modifier_scanner::Modifier);
impl_has_path!(ideology_scanner::Ideology);
impl_has_path!(trait_scanner::Trait);
impl_has_path!(idea_scanner::Idea);
impl_has_path!(character_scanner::Character);
impl_has_path!(building_scanner::Building);
impl_has_path!(resource_scanner::Resource);
impl_has_path!(state_category_scanner::StateCategory);
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

        // Music song/txt files
        if lower.contains("/music/") || lower.contains("\\music\\") {
            cats.push(FileCategory::MusicAssets);
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
// Each helper now uses retain_path! macro for O(K) removal via reverse index.

fn update_localization(scanner_data: &ScannerData, path_str: &str, content: &str) {
    let (parsed, _, _) = loc_parser::parse_loc_file(content, path_str);

    // O(K) removal via reverse index instead of O(N) retain
    retain_path!(
        scanner_data.localization,
        scanner_data.localization_file_index,
        path_str,
        parsed
    );

    // Rebuild duplicated_loc_keys from scratch (cheapest after a loc change)
    let mut seen: HashSet<(InternedStr, InternedStr)> = HashSet::new();
    let mut dups: HashSet<(InternedStr, InternedStr)> = HashSet::new();
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
    let mut new_entries = HashMap::new();
    event_scanner::find_event_definitions(&script.entries, path_str, &mut new_entries);

    retain_path!(
        scanner_data.events,
        scanner_data.events_file_index,
        path_str,
        new_entries
    );
}

fn update_scripted(scanner_data: &ScannerData, kind: &str, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    for entry_ast in script.entries.iter() {
        if let ast::Entry::Assignment(ass) = entry_ast {
            new_entries.insert(
                ass.key.clone(),
                scripted_scanner::ScriptedEntity {
                    name: ass.key.clone(),
                    path: path_str.into(),
                    range: ass.key_range.clone(),
                },
            );
        }
    }

    match kind {
        "triggers" => {
            retain_path!(
                scanner_data.scripted_triggers,
                scanner_data.scripted_triggers_file_index,
                path_str,
                new_entries
            );
        }
        "effects" => {
            retain_path!(
                scanner_data.scripted_effects,
                scanner_data.scripted_effects_file_index,
                path_str,
                new_entries
            );
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

    retain_path!(
        scanner_data.scripted_locs,
        scanner_data.scripted_locs_file_index,
        path_str,
        new_entries
    );
}

fn update_achievements(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    achievement_scanner::find_achievements_in_entries(&script.entries, path_str, &mut new_entries);

    retain_path!(
        scanner_data.achievements,
        scanner_data.achievements_file_index,
        path_str,
        new_entries
    );
}

fn update_modifiers(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    for entry_ast in script.entries.iter() {
        if let ast::Entry::Assignment(ass) = entry_ast {
            new_entries.insert(
                ass.key.clone(),
                modifier_scanner::Modifier {
                    name: ass.key.clone(),
                    path: path_str.into(),
                    range: ass.key_range.clone(),
                },
            );
        }
    }

    retain_path!(
        scanner_data.custom_modifiers,
        scanner_data.custom_modifiers_file_index,
        path_str,
        new_entries
    );
}

fn update_ideologies(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    ideology_scanner::find_ideologies_in_entries(&script.entries, path_str, &mut new_entries);

    let mut sub_map: HashMap<InternedStr, (InternedStr, ast::Range, InternedStr)> = HashMap::new();
    for ideology in new_entries.values() {
        for (sub, range) in &ideology.sub_ideology_ranges {
            sub_map.insert(
                std::sync::Arc::from(sub.as_str()),
                (
                    std::sync::Arc::from(ideology.name.as_str()),
                    range.clone(),
                    ideology.path.clone(),
                ),
            );
        }
    }

    retain_path!(
        scanner_data.ideologies,
        scanner_data.ideologies_file_index,
        path_str,
        new_entries
    );

    // sub_ideologies is a tuple (String, Range, String) — handle manually
    // since tuples can't implement HasPath or use the retain_path! macro
    if let Some((_, old_keys)) = scanner_data.sub_ideologies_file_index.remove(path_str) {
        for key in old_keys {
            scanner_data.sub_ideologies.remove(&key);
        }
    } else {
        let p_owned = path_str.to_owned();
        scanner_data
            .sub_ideologies
            .retain(|_, v| !path_matches(&v.2, &p_owned));
    }
    let mut sub_keys = Vec::with_capacity(sub_map.len());
    for (key, value) in sub_map {
        scanner_data.sub_ideologies.insert(key.clone(), value);
        sub_keys.push(key);
    }
    scanner_data
        .sub_ideologies_file_index
        .insert(std::sync::Arc::from(path_str), sub_keys);
}

fn update_traits(
    scanner_data: &ScannerData,
    trait_type: &str,
    path_str: &str,
    script: &ast::Script,
) {
    let mut new_entries = HashMap::new();
    trait_scanner::find_traits_in_entries(&script.entries, path_str, trait_type, &mut new_entries);

    retain_path!(
        scanner_data.traits,
        scanner_data.traits_file_index,
        path_str,
        new_entries
    );
}

fn update_ideas(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    idea_scanner::find_ideas_in_entries(&script.entries, path_str, &mut new_entries);

    retain_path!(
        scanner_data.ideas,
        scanner_data.ideas_file_index,
        path_str,
        new_entries
    );
}

fn update_characters(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    character_scanner::find_characters_in_entries(&script.entries, path_str, &mut new_entries);

    retain_path!(
        scanner_data.characters,
        scanner_data.characters_file_index,
        path_str,
        new_entries
    );
}

fn update_buildings(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    let path = Path::new(path_str);
    building_scanner::extract_buildings(&script.entries, path, &mut new_entries);

    retain_path!(
        scanner_data.buildings,
        scanner_data.buildings_file_index,
        path_str,
        new_entries
    );
}

fn update_abilities(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    ability_scanner::find_abilities_in_entries(&script.entries, path_str, &mut new_entries);

    retain_path!(
        scanner_data.abilities,
        scanner_data.abilities_file_index,
        path_str,
        new_entries
    );
}

fn update_ai_strategy_plans(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    let path = Path::new(path_str);
    ai_strategy_plan_scanner::extract_plans(&script.entries, path, &mut new_entries);

    retain_path!(
        scanner_data.ai_strategy_plans,
        scanner_data.ai_strategy_plans_file_index,
        path_str,
        new_entries
    );
}

fn update_ai_areas(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    let path = Path::new(path_str);
    ai_area_scanner::extract_areas(&script.entries, path, &mut new_entries);

    retain_path!(
        scanner_data.ai_areas,
        scanner_data.ai_areas_file_index,
        path_str,
        new_entries
    );
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
                        path: path_str.into(),
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
                            path: path_str.into(),
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

    retain_path!(
        scanner_data.country_tags,
        scanner_data.country_tags_file_index,
        path_str,
        new_entries
    );
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
        scanner_data
            .variables
            .entry(k.into())
            .or_default()
            .append(&mut v);
    }

    scanner_data.event_targets.retain(|_, vec| {
        vec.retain(|t| !path_matches(t.path(), path_str));
        !vec.is_empty()
    });
    for (k, mut v) in new_targets {
        scanner_data
            .event_targets
            .entry(k.into())
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

        retain_path!(
            scanner_data.music_assets,
            scanner_data.music_assets_file_index,
            path_str,
            assets
        );
    } else if ext == "txt" {
        let mut stations = HashMap::new();
        let mut songs = HashMap::new();
        music_scanner::find_stations_and_songs_in_entries(
            &script.entries,
            path_str,
            &mut stations,
            &mut songs,
        );

        retain_path!(
            scanner_data.music_stations,
            scanner_data.music_stations_file_index,
            path_str,
            stations
        );
        retain_path!(
            scanner_data.songs,
            scanner_data.songs_file_index,
            path_str,
            songs
        );
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

    retain_path!(
        scanner_data.sounds,
        scanner_data.sounds_file_index,
        path_str,
        sounds
    );
    retain_path!(
        scanner_data.sound_effects,
        scanner_data.sound_effects_file_index,
        path_str,
        effects
    );
    retain_path!(
        scanner_data.falloffs,
        scanner_data.falloffs_file_index,
        path_str,
        falloffs
    );
    retain_path!(
        scanner_data.sound_categories,
        scanner_data.sound_categories_file_index,
        path_str,
        categories
    );
}

fn update_portraits(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    let path = Path::new(path_str);
    portrait_scanner::extract_portraits(&script.entries, path, &mut new_entries);

    retain_path!(
        scanner_data.portraits,
        scanner_data.portraits_file_index,
        path_str,
        new_entries
    );
}

fn update_sprites(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries = HashMap::new();
    sprite_scanner::find_sprites_in_entries(&script.entries, path_str, &mut new_entries);

    retain_path!(
        scanner_data.sprites,
        scanner_data.sprites_file_index,
        path_str,
        new_entries
    );
}

fn update_strategic_regions(scanner_data: &ScannerData, path_str: &str, script: &ast::Script) {
    let mut new_entries: HashMap<u32, strategic_region_scanner::StrategicRegion> = HashMap::new();
    strategic_region_scanner::extract_strategic_region(
        &script.entries,
        Path::new(path_str),
        &mut new_entries,
    );

    let p: &str = path_str;
    if let Some((_, old_keys)) = scanner_data.strategic_regions_file_index.remove(p) {
        for key in old_keys {
            scanner_data.strategic_regions.remove(&key);
        }
    } else {
        let p_owned = p.to_owned();
        scanner_data
            .strategic_regions
            .retain(|_, v| !path_matches(v.path(), &p_owned));
    }
    let mut file_keys = Vec::with_capacity(new_entries.len());
    for (key, value) in new_entries {
        scanner_data.strategic_regions.insert(key, value);
        file_keys.push(key);
    }
    scanner_data
        .strategic_regions_file_index
        .insert(std::sync::Arc::from(path_str), file_keys);
}
