use crate::ability_scanner;
use crate::achievement_scanner;
use crate::adjacency_scanner;
use crate::ai_strategy_plan_scanner;
use crate::Backend;
use crate::building_scanner;
use crate::character_scanner;
use crate::defines_parser;
use crate::event_scanner;
use crate::idea_scanner;
use crate::ideology_scanner;
use crate::loc_parser;
use crate::logistics_scanner;
use crate::map_object_scanner;
use crate::modifier_scanner;
use crate::music_scanner;
use crate::province_scanner;
use crate::scripted_loc_scanner;
use crate::scripted_scanner;
use crate::sound_scanner;
use crate::sprite_scanner;
use crate::state_scanner;
use crate::strategic_region_scanner;
use crate::trait_scanner;
use crate::variable_scanner;
use std::collections::{HashMap, HashSet};
use tower_lsp::lsp_types::MessageType;

impl Backend {
    pub(crate) async fn scan_provinces(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let result = tokio::task::spawn_blocking(move || {
            province_scanner::scan_provinces(&roots_owned, &filter)
        })
        .await
        .unwrap();
        self.provinces.store(std::sync::Arc::new(result));
        let provinces = self.provinces.load();
        self.client
            .log_message(
                MessageType::INFO,
                format!("Total: Loaded {} province definitions", provinces.len()),
            )
            .await;
    }

    pub(crate) async fn scan_states(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let result =
            tokio::task::spawn_blocking(move || state_scanner::scan_states(&roots_owned, &filter))
                .await
                .unwrap();

        self.states.store(std::sync::Arc::new(result));
        let map = self.states.load();
        self.client
            .log_message(MessageType::INFO, format!("Loaded {} states", map.len()))
            .await;
    }

    pub(crate) async fn scan_logistics(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let result = tokio::task::spawn_blocking(move || {
            logistics_scanner::scan_logistics(&roots_owned, &filter)
        })
        .await
        .unwrap();

        self.supply_nodes
            .store(std::sync::Arc::new(result.supply_nodes));
        let sn = self.supply_nodes.load();

        self.railways.store(std::sync::Arc::new(result.railways));
        let rw = self.railways.load();

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

        self.map_buildings
            .store(std::sync::Arc::new(result.buildings));
        let mb = self.map_buildings.load();

        self.unitstacks
            .store(std::sync::Arc::new(result.unitstacks));
        let us = self.unitstacks.load();

        self.weather_positions
            .store(std::sync::Arc::new(result.weather_positions));
        let wp = self.weather_positions.load();

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

        self.adjacencies
            .store(std::sync::Arc::new(result.adjacencies));
        let adj = self.adjacencies.load();

        self.adjacency_rules
            .store(std::sync::Arc::new(result.rules));
        let rules = self.adjacency_rules.load();

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
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let result = tokio::task::spawn_blocking(move || {
            strategic_region_scanner::scan_strategic_regions(&roots_owned, &filter)
        })
        .await
        .unwrap();

        self.strategic_regions.store(std::sync::Arc::new(result));
        let regions = self.strategic_regions.load();

        self.client
            .log_message(
                MessageType::INFO,
                format!("Loaded {} strategic regions", regions.len()),
            )
            .await;
    }

    pub(crate) async fn scan_events(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let result =
            tokio::task::spawn_blocking(move || event_scanner::scan_events(&roots_owned, &filter))
                .await
                .unwrap();
        self.events.store(std::sync::Arc::new(result));
        let events = self.events.load();
        self.client
            .log_message(
                MessageType::INFO,
                format!("Total: Loaded {} event definitions", events.len()),
            )
            .await;
    }

    pub(crate) async fn scan_abilities(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let result = tokio::task::spawn_blocking(move || {
            ability_scanner::scan_abilities(&roots_owned, &filter)
        })
        .await
        .unwrap();
        self.abilities.store(std::sync::Arc::new(result));
        let map = self.abilities.load();
        self.client
            .log_message(
                MessageType::INFO,
                format!("Total: Loaded {} abilities", map.len()),
            )
            .await;
    }

    pub(crate) async fn scan_ai_strategy_plans(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let plans = tokio::task::spawn_blocking(move || {
            ai_strategy_plan_scanner::scan_ai_strategy_plans(&roots_owned, &filter)
        })
        .await
        .unwrap();

        self.ai_strategy_plans
            .store(std::sync::Arc::new(plans));
        let p = self.ai_strategy_plans.load();

        self.client
            .log_message(
                MessageType::INFO,
                format!("Total: Loaded {} AI strategy plans", p.len()),
            )
            .await;
    }

    pub(crate) async fn scan_music(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let result =
            tokio::task::spawn_blocking(move || music_scanner::scan_music(&roots_owned, &filter))
                .await
                .unwrap();

        self.music_assets.store(std::sync::Arc::new(result.assets));
        let assets = self.music_assets.load();

        self.music_stations
            .store(std::sync::Arc::new(result.stations));
        let stations = self.music_stations.load();

        self.songs.store(std::sync::Arc::new(result.songs));
        let songs = self.songs.load();

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

        self.sounds.store(std::sync::Arc::new(result.sounds));
        let sounds = self.sounds.load();

        self.sound_effects
            .store(std::sync::Arc::new(result.sound_effects));
        let effects = self.sound_effects.load();

        self.falloffs.store(std::sync::Arc::new(result.falloffs));
        let falloffs = self.falloffs.load();

        self.sound_categories
            .store(std::sync::Arc::new(result.categories));
        let categories = self.sound_categories.load();

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

        self.custom_modifiers
            .store(std::sync::Arc::new(result.custom_modifiers));
        let custom = self.custom_modifiers.load();

        self.modifier_mappings
            .store(std::sync::Arc::new(result.builtin_mappings));
        let mappings = self.modifier_mappings.load();

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
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let buildings = tokio::task::spawn_blocking(move || {
            building_scanner::scan_buildings(&roots_owned, &filter)
        })
        .await
        .unwrap();

        self.buildings.store(std::sync::Arc::new(buildings));
        let b = self.buildings.load();

        self.client
            .log_message(
                MessageType::INFO,
                format!("Total: Loaded {} buildings", b.len()),
            )
            .await;
    }

    pub(crate) async fn scan_achievements(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let achievements = tokio::task::spawn_blocking(move || {
            achievement_scanner::scan_achievements(&roots_owned, &filter)
        })
        .await
        .unwrap();

        self.achievements.store(std::sync::Arc::new(achievements));
        let a = self.achievements.load();

        self.client
            .log_message(
                MessageType::INFO,
                format!("Total: Loaded {} achievements", a.len()),
            )
            .await;
    }

    pub(crate) async fn scan_defines(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let defines = tokio::task::spawn_blocking(move || {
            defines_parser::scan_defines(&roots_owned, &filter)
        })
        .await
        .unwrap();

        self.defines.store(std::sync::Arc::new(defines));
        let _d = self.defines.load();

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

        self.variables.store(std::sync::Arc::new(result.variables));
        let vars = self.variables.load();

        self.event_targets
            .store(std::sync::Arc::new(result.event_targets));
        let targets = self.event_targets.load();

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

        let (all_locs, dups, logs) = tokio::task::spawn_blocking(move || {
            let mut all_locs = HashMap::new();
            let mut dups = HashSet::new();
            let mut seen_locs_by_lang: HashSet<(String, String)> = HashSet::new();
            let mut logs = Vec::new();

            for root in roots_owned {
                let loc_dir = root.join("localisation");
                if !loc_dir.exists() {
                    continue;
                }

                let mut files_to_scan = Vec::new();
                let mut dirs_to_check = vec![loc_dir.clone()];

                while let Some(current_dir) = dirs_to_check.pop() {
                    if filter(&current_dir) {
                        continue;
                    }
                    if let Ok(entries) = std::fs::read_dir(current_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_dir() {
                                dirs_to_check.push(path);
                            } else if path.extension().is_some_and(|ext| ext == "yml") {
                                if filter(&path) {
                                    continue;
                                }
                                let path_str = path.to_string_lossy().to_lowercase();
                                if path_str.contains("english") {
                                    files_to_scan.push(path);
                                }
                            }
                        }
                    }
                }

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
            (all_locs, dups, logs)
        })
        .await
        .unwrap();

        for (level, msg) in logs {
            self.client.log_message(level, msg).await;
        }

        self.duplicated_loc_keys.store(std::sync::Arc::new(dups));
        let _d_map = self.duplicated_loc_keys.load();

        self.localization.store(std::sync::Arc::new(all_locs));
        let loc = self.localization.load();
        self.client
            .log_message(
                MessageType::INFO,
                format!("Total: Loaded {} localization keys", loc.len()),
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

        self.scripted_triggers
            .store(std::sync::Arc::new(all_triggers));
        let t_map = self.scripted_triggers.load();

        self.scripted_effects
            .store(std::sync::Arc::new(all_effects));
        let e_map = self.scripted_effects.load();

        self.scripted_locs.store(std::sync::Arc::new(all_locs));
        let l_map = self.scripted_locs.load();

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

        self.ideologies.store(std::sync::Arc::new(all_results));
        let i_map = self.ideologies.load();

        self.sub_ideologies.store(std::sync::Arc::new(sub_map));
        let s_map = self.sub_ideologies.load();

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

        self.traits.store(std::sync::Arc::new(all_traits));
        let t_map = self.traits.load();

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

        self.sprites.store(std::sync::Arc::new(all_sprites));
        let s_map = self.sprites.load();

        self.client
            .log_message(
                MessageType::INFO,
                format!("Total: Loaded {} sprite definitions", s_map.len()),
            )
            .await;
    }

    pub(crate) async fn scan_characters(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let found = tokio::task::spawn_blocking(move || {
            character_scanner::scan_characters(&roots_owned, &filter)
        })
        .await
        .unwrap();

        self.characters.store(std::sync::Arc::new(found));
        let c_map = self.characters.load();

        self.client
            .log_message(
                MessageType::INFO,
                format!("Total: Loaded {} characters", c_map.len()),
            )
            .await;
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

        self.ideas.store(std::sync::Arc::new(all_ideas));
        let i_map = self.ideas.load();

        self.client
            .log_message(
                MessageType::INFO,
                format!("Total: Loaded {} ideas", i_map.len()),
            )
            .await;
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
                    let mut m = (**self.modifier_mappings.load()).clone();
                    for (k, v) in mappings {
                        m.insert(k, v);
                    }
                    self.modifier_mappings.store(std::sync::Arc::new(m));
                    let m = self.modifier_mappings.load();
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
                    let mut f = (**self.modifier_formats.load()).clone();
                    for (k, v) in formats {
                        f.insert(k, v);
                    }
                    self.modifier_formats.store(std::sync::Arc::new(f));
                    let f = self.modifier_formats.load();
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
