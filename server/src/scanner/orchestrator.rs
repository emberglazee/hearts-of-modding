use crate::Backend;
use crate::data::interner::InternedStr;
use crate::parser::defines_parser;
use crate::parser::loc_parser;
use crate::scanner::ability_scanner;
use crate::scanner::achievement_scanner;
use crate::scanner::adjacency_scanner;
use crate::scanner::ai_area_scanner;
use crate::scanner::ai_strategy_plan_scanner;
use crate::scanner::building_scanner;
use crate::scanner::character_scanner;
use crate::scanner::continent_scanner;
use crate::scanner::country_scanner;
use crate::scanner::event_scanner;
use crate::scanner::focus_scanner;
use crate::scanner::gfx_scanner;
use crate::scanner::idea_scanner;
use crate::scanner::ideology_scanner;
use crate::scanner::logistics_scanner;
use crate::scanner::map_object_scanner;
use crate::scanner::modifier_scanner;
use crate::scanner::music_scanner;
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
use crate::scanner::trait_scanner;
use crate::scanner::variable_scanner;
use std::collections::{HashMap, HashSet};
use tower_lsp_server::ls_types::MessageType;

/// Spawn a blocking scan that returns `HashMap<K, V>`, then clear and
/// re-insert results into the named DashMap field on `ScannerData`.
macro_rules! scan_dashmap {
    ($self:ident, $roots:expr, $scanner_fn:expr, $field:ident, $msg:literal) => {{
        let filter = $self.get_sync_filter();
        let roots_owned = $roots.to_vec();
        let result = tokio::task::spawn_blocking(move || $scanner_fn(&roots_owned, &filter))
            .await
            .unwrap();
        $self.scanner_data.$field.clear();
        for (k, v) in result {
            $self.scanner_data.$field.insert(k.into(), v);
        }
        let count = $self.scanner_data.$field.len();
        $self
            .client
            .log_message(MessageType::INFO, format!($msg, count))
            .await;
    }};
}

// ── Scan methods ────────────────────────────────────────────────────

impl Backend {
    pub(crate) async fn scan_provinces(&self, roots: &[std::path::PathBuf]) {
        scan_dashmap!(
            self,
            roots,
            province_scanner::scan_provinces,
            provinces,
            "Total: Loaded {} province definitions"
        );
    }

    pub(crate) async fn scan_states(&self, roots: &[std::path::PathBuf]) {
        scan_dashmap!(
            self,
            roots,
            state_scanner::scan_states,
            states,
            "Loaded {} states"
        );
    }

    pub(crate) async fn scan_logistics(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let result = tokio::task::spawn_blocking(move || {
            logistics_scanner::scan_logistics(&roots_owned, &filter)
        })
        .await
        .unwrap();

        self.scanner_data.set_supply_nodes(result.supply_nodes);
        let sn = self.scanner_data.supply_nodes();

        self.scanner_data.set_railways(result.railways);
        let rw = self.scanner_data.railways();

        self.client
            .log_message(
                MessageType::INFO,
                format!("Loaded {} supply nodes, {} railways", sn.len(), rw.len()),
            )
            .await;
    }

    pub(crate) async fn scan_map_objects(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let result = tokio::task::spawn_blocking(move || {
            map_object_scanner::scan_map_objects(&roots_owned, &filter)
        })
        .await
        .unwrap();

        self.scanner_data.set_map_buildings(result.buildings);
        let mb = self.scanner_data.map_buildings();

        self.scanner_data.set_unitstacks(result.unitstacks);
        let us = self.scanner_data.unitstacks();

        self.scanner_data
            .set_weather_positions(result.weather_positions);
        let wp = self.scanner_data.weather_positions();

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Loaded {} map buildings, {} unit stacks, {} weather positions",
                    mb.len(),
                    us.len(),
                    wp.len()
                ),
            )
            .await;
    }

    pub(crate) async fn scan_adjacencies(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let result = tokio::task::spawn_blocking(move || {
            adjacency_scanner::scan_adjacencies(&roots_owned, &filter)
        })
        .await
        .unwrap();

        self.scanner_data.set_adjacencies(result.adjacencies);
        let adj = self.scanner_data.adjacencies();

        self.scanner_data.adjacency_rules.clear();
        for (k, v) in result.rules {
            self.scanner_data.adjacency_rules.insert(k.into(), v);
        }
        let rules = &self.scanner_data.adjacency_rules;

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Loaded {} adjacencies, {} adjacency rules",
                    adj.len(),
                    rules.len()
                ),
            )
            .await;
    }

    pub(crate) async fn scan_strategic_regions(&self, roots: &[std::path::PathBuf]) {
        scan_dashmap!(
            self,
            roots,
            strategic_region_scanner::scan_strategic_regions,
            strategic_regions,
            "Loaded {} strategic regions"
        );
    }

    pub(crate) async fn scan_events(&self, roots: &[std::path::PathBuf]) {
        scan_dashmap!(
            self,
            roots,
            event_scanner::scan_events,
            events,
            "Total: Loaded {} event definitions"
        );
    }

    pub(crate) async fn scan_focuses(&self, roots: &[std::path::PathBuf]) {
        scan_dashmap!(
            self,
            roots,
            focus_scanner::scan_focuses,
            focuses,
            "Total: Loaded {} national focus definitions"
        );
    }

    pub(crate) async fn scan_abilities(&self, roots: &[std::path::PathBuf]) {
        scan_dashmap!(
            self,
            roots,
            ability_scanner::scan_abilities,
            abilities,
            "Total: Loaded {} abilities"
        );
    }

    pub(crate) async fn scan_ai_strategy_plans(&self, roots: &[std::path::PathBuf]) {
        scan_dashmap!(
            self,
            roots,
            ai_strategy_plan_scanner::scan_ai_strategy_plans,
            ai_strategy_plans,
            "Total: Loaded {} AI strategy plans"
        );
    }

    pub(crate) async fn scan_ai_areas(&self, roots: &[std::path::PathBuf]) {
        scan_dashmap!(
            self,
            roots,
            ai_area_scanner::scan_ai_areas,
            ai_areas,
            "Total: Loaded {} AI areas"
        );
    }

    pub(crate) async fn scan_continents(&self, roots: &[std::path::PathBuf]) {
        let roots_owned = roots.to_vec();
        let result = tokio::task::spawn_blocking(move || {
            let mut all_continents = std::collections::HashMap::new();
            for root in &roots_owned {
                all_continents.extend(continent_scanner::scan_continents(root));
            }
            all_continents
        })
        .await
        .unwrap();

        self.scanner_data.continents.clear();
        for (k, v) in result {
            self.scanner_data.continents.insert(k.into(), v);
        }
        let c = &self.scanner_data.continents;

        self.client
            .log_message(
                MessageType::INFO,
                format!("Total: Loaded {} continent definitions", c.len()),
            )
            .await;
    }

    pub(crate) async fn scan_portraits(&self, roots: &[std::path::PathBuf]) {
        scan_dashmap!(
            self,
            roots,
            portrait_scanner::scan_portraits,
            portraits,
            "Total: Loaded {} portrait definitions"
        );
    }

    pub(crate) async fn scan_music(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let result =
            tokio::task::spawn_blocking(move || music_scanner::scan_music(&roots_owned, &filter))
                .await
                .unwrap();

        self.scanner_data.music_assets.clear();
        for (k, v) in result.assets {
            self.scanner_data.music_assets.insert(k.into(), v);
        }
        let assets = &self.scanner_data.music_assets;

        self.scanner_data.music_stations.clear();
        for (k, v) in result.stations {
            self.scanner_data.music_stations.insert(k.into(), v);
        }
        let stations = &self.scanner_data.music_stations;

        self.scanner_data.songs.clear();
        for (k, v) in result.songs {
            self.scanner_data.songs.insert(k.into(), v);
        }
        let songs = &self.scanner_data.songs;

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Total: Loaded {} music assets, {} stations, and {} songs",
                    assets.len(),
                    stations.len(),
                    songs.len()
                ),
            )
            .await;
    }

    pub(crate) async fn scan_sounds(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let result =
            tokio::task::spawn_blocking(move || sound_scanner::scan_sounds(&roots_owned, &filter))
                .await
                .unwrap();

        self.scanner_data.sounds.clear();
        for (k, v) in result.sounds {
            self.scanner_data.sounds.insert(k.into(), v);
        }
        let sounds = &self.scanner_data.sounds;

        self.scanner_data.sound_effects.clear();
        for (k, v) in result.sound_effects {
            self.scanner_data.sound_effects.insert(k.into(), v);
        }
        let effects = &self.scanner_data.sound_effects;

        self.scanner_data.falloffs.clear();
        for (k, v) in result.falloffs {
            self.scanner_data.falloffs.insert(k.into(), v);
        }
        let falloffs = &self.scanner_data.falloffs;

        self.scanner_data.sound_categories.clear();
        for (k, v) in result.categories {
            self.scanner_data.sound_categories.insert(k.into(), v);
        }
        let categories = &self.scanner_data.sound_categories;

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Total: Loaded {} sounds, {} sound effects, {} falloffs, and {} categories",
                    sounds.len(),
                    effects.len(),
                    falloffs.len(),
                    categories.len()
                ),
            )
            .await;
    }

    pub(crate) async fn scan_modifiers(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let result = tokio::task::spawn_blocking(move || {
            modifier_scanner::scan_modifiers(&roots_owned, &filter)
        })
        .await
        .unwrap();

        self.scanner_data.custom_modifiers.clear();
        for (k, v) in result.custom_modifiers {
            self.scanner_data.custom_modifiers.insert(k.into(), v);
        }
        let custom = &self.scanner_data.custom_modifiers;

        self.scanner_data.modifier_mappings.clear();
        for (k, v) in result.builtin_mappings {
            self.scanner_data.modifier_mappings.insert(k.into(), v);
        }
        let mappings = &self.scanner_data.modifier_mappings;

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Total: Loaded {} custom modifiers and {} builtin mappings",
                    custom.len(),
                    mappings.len()
                ),
            )
            .await;
    }

    pub(crate) async fn scan_buildings(&self, roots: &[std::path::PathBuf]) {
        scan_dashmap!(
            self,
            roots,
            building_scanner::scan_buildings,
            buildings,
            "Total: Loaded {} buildings"
        );
    }

    pub(crate) async fn scan_resources(&self, roots: &[std::path::PathBuf]) {
        scan_dashmap!(
            self,
            roots,
            resource_scanner::scan_resources,
            resources,
            "Total: Loaded {} resources"
        );
    }

    pub(crate) async fn scan_state_categories(&self, roots: &[std::path::PathBuf]) {
        scan_dashmap!(
            self,
            roots,
            state_category_scanner::scan_state_categories,
            state_categories,
            "Total: Loaded {} state categories"
        );
    }

    pub(crate) async fn scan_achievements(&self, roots: &[std::path::PathBuf]) {
        scan_dashmap!(
            self,
            roots,
            achievement_scanner::scan_achievements,
            achievements,
            "Total: Loaded {} achievements"
        );
    }

    pub(crate) async fn scan_defines(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let defines = tokio::task::spawn_blocking(move || {
            defines_parser::scan_defines(&roots_owned, &filter)
        })
        .await
        .unwrap();

        self.scanner_data.set_defines(defines);
        let _d = self.scanner_data.defines();

        self.client
            .log_message(MessageType::INFO, "Loaded game defines")
            .await;
    }

    pub(crate) async fn scan_variables(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let result = tokio::task::spawn_blocking(move || {
            variable_scanner::scan_roots(&roots_owned, &filter)
        })
        .await
        .unwrap();

        self.scanner_data.variables.clear();
        for (k, v) in result.variables {
            self.scanner_data.variables.insert(k.into(), v);
        }
        let vars = &self.scanner_data.variables;

        self.scanner_data.event_targets.clear();
        for (k, v) in result.event_targets {
            self.scanner_data.event_targets.insert(k.into(), v);
        }
        let targets = &self.scanner_data.event_targets;

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Total: Loaded {} variables and {} event targets",
                    vars.len(),
                    targets.len()
                ),
            )
            .await;
    }

    pub(crate) async fn scan_localization(&self, roots: &[std::path::PathBuf]) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("Starting localization scan in {} roots", roots.len()),
            )
            .await;

        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();

        let (all_locs, dups, game_keys, logs) = tokio::task::spawn_blocking(move || {
            let mut all_locs = HashMap::new();
            let mut dups = HashSet::new();
            let mut game_keys = HashSet::new();
            let mut seen_locs_by_lang: HashSet<(String, InternedStr)> = HashSet::new();
            let mut logs = Vec::new();
            let has_game_root = roots_owned.len() > 1;

            for (root_idx, root) in roots_owned.iter().enumerate() {
                let is_game_root = has_game_root && root_idx == 0;
                let loc_dir = root.join("localisation");
                if !loc_dir.exists() {
                    continue;
                }

                let mut files_to_scan: Vec<_> =
                    crate::utils::fs_util::collect_files(&loc_dir, &["yml"], &filter, false)
                        .into_iter()
                        .filter(|p| p.to_string_lossy().to_ascii_lowercase().contains("english"))
                        .collect();

                files_to_scan.sort_by(|a, b| {
                    let a_is_replace = a.to_string_lossy().contains("replace");
                    let b_is_replace = b.to_string_lossy().contains("replace");
                    match (a_is_replace, b_is_replace) {
                        (true, false) => std::cmp::Ordering::Greater,
                        (false, true) => std::cmp::Ordering::Less,
                        _ => a.cmp(b),
                    }
                });

                for path in files_to_scan {
                    match std::fs::read_to_string(&path) {
                        Ok(content) => {
                            let path_str = path.to_string_lossy().to_string();
                            let (parsed, _, doc_lang) =
                                loc_parser::parse_loc_file(&content, &path_str);
                            let lang_str = doc_lang.unwrap_or_else(|| "unknown".to_string());

                            if parsed.is_empty() {
                                logs.push((
                                    MessageType::LOG,
                                    format!(
                                        "Warning: No keys found in localization file: {:?}",
                                        path
                                    ),
                                ));
                            } else {
                                logs.push((
                                    MessageType::LOG,
                                    format!("Loaded {} keys from {:?}", parsed.len(), path),
                                ));
                            }

                            for (key, entry) in parsed {
                                let lang_key_pair = (lang_str.clone(), key.clone());
                                if is_game_root {
                                    game_keys.insert(lang_key_pair.clone());
                                }
                                if seen_locs_by_lang.contains(&lang_key_pair) {
                                    dups.insert(lang_key_pair.clone());
                                } else {
                                    seen_locs_by_lang.insert(lang_key_pair);
                                }
                                all_locs.insert(key, entry);
                            }
                        }
                        Err(e) => {
                            logs.push((
                                MessageType::ERROR,
                                format!("Failed to read localization file {:?}: {}", path, e),
                            ));
                        }
                    }
                }
            }
            (all_locs, dups, game_keys, logs)
        })
        .await
        .unwrap();

        for (level, msg) in logs {
            self.client.log_message(level, msg).await;
        }

        self.scanner_data.duplicated_loc_keys.clear();
        for x in dups {
            self.scanner_data
                .duplicated_loc_keys
                .insert((x.0.into(), x.1));
        }
        let _d_map = &self.scanner_data.duplicated_loc_keys;

        self.scanner_data.game_loc_keys.clear();
        for x in game_keys {
            self.scanner_data.game_loc_keys.insert((x.0.into(), x.1));
        }
        let game_loc_count = self.scanner_data.game_loc_keys.len();

        self.scanner_data.localization.clear();
        for (k, v) in all_locs {
            self.scanner_data.localization.insert(k, v);
        }
        let loc = &self.scanner_data.localization;
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Total: Loaded {} localization keys ({} from game path)",
                    loc.len(),
                    game_loc_count
                ),
            )
            .await;
    }

    pub(crate) async fn scan_scripted(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();

        let (all_triggers, all_effects, all_locs) = tokio::task::spawn_blocking(move || {
            let mut all_triggers = HashMap::new();
            let mut all_effects = HashMap::new();
            let mut all_locs = HashMap::new();

            for root in &roots_owned {
                let triggers_dir = root.join("common/scripted_triggers");
                let effects_dir = root.join("common/scripted_effects");
                let locs_dir = root.join("common/scripted_localisation");

                if triggers_dir.exists() {
                    let found = scripted_scanner::scan_directory(&triggers_dir, &filter);
                    all_triggers.extend(found);
                }
                if effects_dir.exists() {
                    let found = scripted_scanner::scan_directory(&effects_dir, &filter);
                    all_effects.extend(found);
                }
                if locs_dir.exists() {
                    let found = scripted_loc_scanner::scan_directory(&locs_dir, &filter);
                    all_locs.extend(found);
                }
            }
            (all_triggers, all_effects, all_locs)
        })
        .await
        .unwrap();

        self.scanner_data.scripted_triggers.clear();
        for (k, v) in all_triggers {
            self.scanner_data.scripted_triggers.insert(k.into(), v);
        }
        let t_map = &self.scanner_data.scripted_triggers;

        self.scanner_data.scripted_effects.clear();
        for (k, v) in all_effects {
            self.scanner_data.scripted_effects.insert(k.into(), v);
        }
        let e_map = &self.scanner_data.scripted_effects;

        self.scanner_data.scripted_locs.clear();
        for (k, v) in all_locs {
            self.scanner_data.scripted_locs.insert(k.into(), v);
        }
        let l_map = &self.scanner_data.scripted_locs;

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Total: Loaded {} scripted triggers, {} scripted effects, {} scripted locs",
                    t_map.len(),
                    e_map.len(),
                    l_map.len()
                ),
            )
            .await;
    }

    pub(crate) async fn scan_ideologies(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();

        let (all_results, sub_map) = tokio::task::spawn_blocking(move || {
            let mut all_results = HashMap::new();
            let mut sub_map = HashMap::new();

            for root in &roots_owned {
                let dir = root.join("common/ideologies");
                if dir.exists() {
                    let results = ideology_scanner::scan_ideologies(&dir, &filter);
                    for ideology in results.values() {
                        for (sub, range) in &ideology.sub_ideology_ranges {
                            sub_map.insert(
                                sub.clone(),
                                (ideology.name.clone(), range.clone(), ideology.path.clone()),
                            );
                        }
                    }
                    all_results.extend(results);
                }
            }
            (all_results, sub_map)
        })
        .await
        .unwrap();

        self.scanner_data.ideologies.clear();
        for (k, v) in all_results {
            self.scanner_data.ideologies.insert(k.into(), v);
        }
        let i_map = &self.scanner_data.ideologies;

        self.scanner_data.sub_ideologies.clear();
        for (k, v) in sub_map {
            self.scanner_data
                .sub_ideologies
                .insert(k.into(), (v.0.into(), v.1, v.2));
        }
        let s_map = &self.scanner_data.sub_ideologies;

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Total: Loaded {} ideologies and {} sub-ideologies",
                    i_map.len(),
                    s_map.len()
                ),
            )
            .await;
    }

    pub(crate) async fn scan_traits(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();

        let all_traits = tokio::task::spawn_blocking(move || {
            let mut all_traits = HashMap::new();
            for root in &roots_owned {
                let unit_leader_dir = root.join("common/unit_leader");
                if unit_leader_dir.exists() {
                    let found =
                        trait_scanner::scan_traits(&unit_leader_dir, "Unit Leader Trait", &filter);
                    all_traits.extend(found);
                }

                let country_leader_dir = root.join("common/country_leader");
                if country_leader_dir.exists() {
                    let found = trait_scanner::scan_traits(
                        &country_leader_dir,
                        "Country Leader Trait",
                        &filter,
                    );
                    all_traits.extend(found);
                }

                let trait_dir = root.join("common/traits");
                if trait_dir.exists() {
                    let found = trait_scanner::scan_traits(&trait_dir, "Trait", &filter);
                    all_traits.extend(found);
                }
            }
            all_traits
        })
        .await
        .unwrap();

        self.scanner_data.traits.clear();
        for (k, v) in all_traits {
            self.scanner_data.traits.insert(k.into(), v);
        }
        let t_map = &self.scanner_data.traits;

        self.client
            .log_message(
                MessageType::INFO,
                format!("Total: Loaded {} traits", t_map.len()),
            )
            .await;
    }

    pub(crate) async fn scan_sprites(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();

        let all_sprites = tokio::task::spawn_blocking(move || {
            let mut all_sprites = HashMap::new();
            for root in &roots_owned {
                let interface_dir = root.join("interface");
                if interface_dir.exists() {
                    let found = sprite_scanner::scan_sprites(&interface_dir, &filter);
                    all_sprites.extend(found);
                }
            }
            all_sprites
        })
        .await
        .unwrap();

        self.scanner_data.sprites.clear();
        for (k, v) in all_sprites {
            self.scanner_data.sprites.insert(k.into(), v);
        }
        let s_map = &self.scanner_data.sprites;

        self.client
            .log_message(
                MessageType::INFO,
                format!("Total: Loaded {} sprite definitions", s_map.len()),
            )
            .await;
    }

    pub(crate) async fn scan_characters(&self, roots: &[std::path::PathBuf]) {
        scan_dashmap!(
            self,
            roots,
            character_scanner::scan_characters,
            characters,
            "Total: Loaded {} characters"
        );
    }

    pub(crate) async fn scan_ideas(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();

        let all_ideas = tokio::task::spawn_blocking(move || {
            let mut all_ideas = HashMap::new();
            for root in &roots_owned {
                let ideas_dir = root.join("common/ideas");
                if ideas_dir.exists() {
                    let found = idea_scanner::scan_ideas(&ideas_dir, &filter);
                    all_ideas.extend(found);
                }
            }
            all_ideas
        })
        .await
        .unwrap();

        self.scanner_data.ideas.clear();
        for (k, v) in all_ideas {
            self.scanner_data.ideas.insert(k.into(), v);
        }
        let i_map = &self.scanner_data.ideas;

        self.client
            .log_message(
                MessageType::INFO,
                format!("Total: Loaded {} ideas", i_map.len()),
            )
            .await;
    }

    pub(crate) async fn scan_gfx(&self, roots: &[std::path::PathBuf]) {
        scan_dashmap!(
            self,
            roots,
            gfx_scanner::scan_color_codes,
            color_codes,
            "Total: Loaded {} color codes from interface/*.gfx"
        );
    }

    pub(crate) async fn scan_countries(&self, roots: &[std::path::PathBuf]) {
        scan_dashmap!(
            self,
            roots,
            country_scanner::scan_country_tags,
            country_tags,
            "Loaded {} country tags"
        );
    }

    pub(crate) async fn load_assets(&self) {
        let exe_path = std::env::current_exe().unwrap_or_default();
        let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));

        let possible_roots = vec![
            std::path::PathBuf::from("."),
            exe_dir.to_path_buf(),
            exe_dir.join(".."),
        ];

        let mut mapping_path = None;
        let mut formats_path = None;

        for root in &possible_roots {
            let m = root.join("assets/modifier_mappings.json");
            let f = root.join("assets/modifier_formats.json");
            if m.exists() {
                mapping_path = Some(m);
            }
            if f.exists() {
                formats_path = Some(f);
            }
        }

        if let Some(path) = mapping_path {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(mappings) = serde_json::from_str::<HashMap<String, String>>(&content) {
                    for (k, v) in mappings {
                        self.scanner_data.modifier_mappings.insert(k.into(), v);
                    }
                    let m = &self.scanner_data.modifier_mappings;
                    self.client
                        .log_message(
                            MessageType::INFO,
                            format!("Loaded {} modifier mappings from assets", m.len()),
                        )
                        .await;
                }
            }
        }

        if let Some(path) = formats_path {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(formats) = serde_json::from_str::<HashMap<String, String>>(&content) {
                    for (k, v) in formats {
                        self.scanner_data.modifier_formats.insert(k.into(), v);
                    }
                    let f = &self.scanner_data.modifier_formats;
                    self.client
                        .log_message(
                            MessageType::INFO,
                            format!("Loaded {} modifier formats from assets", f.len()),
                        )
                        .await;
                }
            }
        }
    }
}
