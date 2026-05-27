use crate::Backend;
use crate::ability_scanner;
use crate::achievement_scanner;
use crate::adjacency_scanner;
use crate::ai_area_scanner;
use crate::ai_strategy_plan_scanner;
use crate::building_scanner;
use crate::character_scanner;
use crate::continent_scanner;
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
use std::collections::{HashMap, HashSet};
use tower_lsp_server::ls_types::MessageType;

impl Backend {
    pub(crate) async fn scan_provinces(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let result = tokio::task::spawn_blocking(move || {
            province_scanner::scan_provinces(&roots_owned, &filter)
        })
        .await
        .unwrap();
        self.scanner_data.set_provinces(result);
        let provinces = self.scanner_data.provinces();
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

        self.scanner_data.set_states(result);
        let map = self.scanner_data.states();
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

        self.scanner_data.set_adjacency_rules(result.rules);
        let rules = self.scanner_data.adjacency_rules();

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

        self.scanner_data.set_strategic_regions(result);
        let regions = self.scanner_data.strategic_regions();

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
        self.scanner_data.set_events(result);
        let events = self.scanner_data.events();
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
        self.scanner_data.set_abilities(result);
        let map = self.scanner_data.abilities();
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

        self.scanner_data.set_ai_strategy_plans(plans);
        let p = self.scanner_data.ai_strategy_plans();

        self.client
            .log_message(
                MessageType::INFO,
                format!("Total: Loaded {} AI strategy plans", p.len()),
            )
            .await;
    }

    pub(crate) async fn scan_ai_areas(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let areas = tokio::task::spawn_blocking(move || {
            ai_area_scanner::scan_ai_areas(&roots_owned, &filter)
        })
        .await
        .unwrap();

        self.scanner_data.set_ai_areas(areas);
        let a = self.scanner_data.ai_areas();

        self.client
            .log_message(
                MessageType::INFO,
                format!("Total: Loaded {} AI areas", a.len()),
            )
            .await;
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

        self.scanner_data.set_continents(result);
        let c = self.scanner_data.continents();

        self.client
            .log_message(
                MessageType::INFO,
                format!("Total: Loaded {} continent definitions", c.len()),
            )
            .await;
    }

    pub(crate) async fn scan_portraits(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let portraits = tokio::task::spawn_blocking(move || {
            portrait_scanner::scan_portraits(&roots_owned, &filter)
        })
        .await
        .unwrap();

        self.scanner_data.set_portraits(portraits);
        let p = self.scanner_data.portraits();

        self.client
            .log_message(
                MessageType::INFO,
                format!("Total: Loaded {} portrait definitions", p.len()),
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

        self.scanner_data.set_music_assets(result.assets);
        let assets = self.scanner_data.music_assets();

        self.scanner_data.set_music_stations(result.stations);
        let stations = self.scanner_data.music_stations();

        self.scanner_data.set_songs(result.songs);
        let songs = self.scanner_data.songs();

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

        self.scanner_data.set_sounds(result.sounds);
        let sounds = self.scanner_data.sounds();

        self.scanner_data.set_sound_effects(result.sound_effects);
        let effects = self.scanner_data.sound_effects();

        self.scanner_data.set_falloffs(result.falloffs);
        let falloffs = self.scanner_data.falloffs();

        self.scanner_data.set_sound_categories(result.categories);
        let categories = self.scanner_data.sound_categories();

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

        self.scanner_data
            .set_custom_modifiers(result.custom_modifiers);
        let custom = self.scanner_data.custom_modifiers();

        self.scanner_data
            .set_modifier_mappings(result.builtin_mappings);
        let mappings = self.scanner_data.modifier_mappings();

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

        self.scanner_data.set_buildings(buildings);
        let b = self.scanner_data.buildings();

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

        self.scanner_data.set_achievements(achievements);
        let a = self.scanner_data.achievements();

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

        self.scanner_data.set_variables(result.variables);
        let vars = self.scanner_data.variables();

        self.scanner_data.set_event_targets(result.event_targets);
        let targets = self.scanner_data.event_targets();

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

                let mut files_to_scan: Vec<_> =
                    crate::fs_util::collect_files(&loc_dir, &["yml"], &filter, false)
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

        self.scanner_data.set_duplicated_loc_keys(dups);
        let _d_map = self.scanner_data.duplicated_loc_keys();

        self.scanner_data.set_localization(all_locs);
        let loc = self.scanner_data.localization();
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

        self.scanner_data.set_scripted_triggers(all_triggers);
        let t_map = self.scanner_data.scripted_triggers();

        self.scanner_data.set_scripted_effects(all_effects);
        let e_map = self.scanner_data.scripted_effects();

        self.scanner_data.set_scripted_locs(all_locs);
        let l_map = self.scanner_data.scripted_locs();

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

        self.scanner_data.set_ideologies(all_results);
        let i_map = self.scanner_data.ideologies();

        self.scanner_data.set_sub_ideologies(sub_map);
        let s_map = self.scanner_data.sub_ideologies();

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

        self.scanner_data.set_traits(all_traits);
        let t_map = self.scanner_data.traits();

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

        self.scanner_data.set_sprites(all_sprites);
        let s_map = self.scanner_data.sprites();

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

        self.scanner_data.set_characters(found);
        let c_map = self.scanner_data.characters();

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

        self.scanner_data.set_ideas(all_ideas);
        let i_map = self.scanner_data.ideas();

        self.client
            .log_message(
                MessageType::INFO,
                format!("Total: Loaded {} ideas", i_map.len()),
            )
            .await;
    }

    pub(crate) async fn scan_countries(&self, roots: &[std::path::PathBuf]) {
        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();
        let result = tokio::task::spawn_blocking(move || {
            country_scanner::scan_country_tags(&roots_owned, &filter)
        })
        .await
        .unwrap();
        self.scanner_data.set_country_tags(result);
        let tags = self.scanner_data.country_tags();
        self.client
            .log_message(
                MessageType::INFO,
                format!("Loaded {} country tags", tags.len()),
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
                    let mut m = (*self.scanner_data.modifier_mappings()).clone();
                    for (k, v) in mappings {
                        m.insert(k, v);
                    }
                    self.scanner_data.set_modifier_mappings(m);
                    let m = self.scanner_data.modifier_mappings();
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
                    let mut f = (*self.scanner_data.modifier_formats()).clone();
                    for (k, v) in formats {
                        f.insert(k, v);
                    }
                    self.scanner_data.set_modifier_formats(f);
                    let f = self.scanner_data.modifier_formats();
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
