use crate::Backend;
use crate::data::interner::InternedStr;
use crate::data::layered_value::LayeredValue;
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
use crate::scanner::event_scanner;
use crate::scanner::focus_scanner;
use crate::scanner::gfx_scanner;
use crate::scanner::idea_scanner;
use crate::scanner::ideology_scanner;
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
use std::collections::{HashMap, HashSet};
use tower_lsp_server::ls_types::MessageType;

/// File-overlay variant: builds a flat list of winning files from the
/// pre-computed FileOverlay, then calls the scanner function once with
/// only those files. Results are inserted as single-layer (no merging)
/// because the overlay already resolved which files win.
///
/// `$prefix` is the relative directory prefix for this scanner
/// (e.g., "common/ideas"). `$extensions` filters file extensions.
/// `$scanner_fn` must be `fn(&[PathBuf], &Filter) -> HashMap<String, T>`.
macro_rules! scan_dashmap_overlay {
    ($self:ident, $overlay:expr, $prefix:expr, $scanner_fn:expr, $field:ident, $extensions:expr, $msg:literal) => {{
        let filter = $self.get_sync_filter();
        let all_files: Vec<std::path::PathBuf> = $overlay.winning_files_in($prefix);
        let filtered: Vec<std::path::PathBuf> = all_files
            .into_iter()
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| $extensions.contains(&e))
            })
            .collect();
        let result = tokio::task::spawn_blocking(move || $scanner_fn(&filtered, &filter))
            .await
            .unwrap();
        $self.scanner_data.$field.clear();
        for (k, v) in result {
            $self
                .scanner_data
                .$field
                .insert(k.into(), crate::data::layered_value::LayeredValue::new(v));
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
    // ═══════════════════════════════════════════════════════════════════
    // File-level overlay scanners — only winning files are parsed
    // ═══════════════════════════════════════════════════════════════════

    pub(crate) async fn scan_provinces(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        let filter = self.get_sync_filter();
        let all_files: Vec<std::path::PathBuf> = overlay.winning_files_in("map");
        let filtered: Vec<std::path::PathBuf> = all_files
            .into_iter()
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e == "csv")
                    .unwrap_or(false)
            })
            .collect();
        let result = tokio::task::spawn_blocking(move || {
            province_scanner::scan_province_files(&filtered, &filter)
        })
        .await
        .unwrap();
        self.scanner_data.provinces.clear();
        for (k, v) in result {
            self.scanner_data.provinces.insert(k, v);
        }
        let count = self.scanner_data.provinces.len();
        self.client
            .log_message(
                MessageType::INFO,
                format!("Total: Loaded {} province definitions", count),
            )
            .await;
    }

    pub(crate) async fn scan_states(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        let filter = self.get_sync_filter();
        let all_files: Vec<std::path::PathBuf> = overlay.winning_files_in("history/states");
        let filtered: Vec<std::path::PathBuf> = all_files
            .into_iter()
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e == "txt")
                    .unwrap_or(false)
            })
            .collect();
        let result = tokio::task::spawn_blocking(move || {
            state_scanner::scan_state_files(&filtered, &filter)
        })
        .await
        .unwrap();
        self.scanner_data.states.clear();
        for (k, v) in result {
            self.scanner_data.states.insert(k, v);
        }
        let count = self.scanner_data.states.len();
        self.client
            .log_message(MessageType::INFO, format!("Loaded {} states", count))
            .await;
    }

    pub(crate) async fn scan_logistics(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        let filter = self.get_sync_filter();
        let files: Vec<std::path::PathBuf> = overlay.winning_files_in("map");
        let result = tokio::task::spawn_blocking(move || {
            logistics_scanner::scan_logistics_files(&files, &filter)
        })
        .await
        .unwrap();
        self.scanner_data.set_supply_nodes(result.supply_nodes);
        self.scanner_data.set_railways(result.railways);
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Loaded {} supply nodes, {} railways",
                    self.scanner_data.supply_nodes().len(),
                    self.scanner_data.railways().len()
                ),
            )
            .await;
    }

    pub(crate) async fn scan_map_objects(
        &self,
        overlay: &crate::scanner::file_overlay::FileOverlay,
    ) {
        let filter = self.get_sync_filter();
        let files: Vec<std::path::PathBuf> = overlay.winning_files_in("map");
        let result = tokio::task::spawn_blocking(move || {
            map_object_scanner::scan_map_object_files(&files, &filter)
        })
        .await
        .unwrap();
        self.scanner_data.set_map_buildings(result.buildings);
        self.scanner_data.set_unitstacks(result.unitstacks);
        self.scanner_data
            .set_weather_positions(result.weather_positions);
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Loaded {} map buildings, {} unit stacks, {} weather positions",
                    self.scanner_data.map_buildings().len(),
                    self.scanner_data.unitstacks().len(),
                    self.scanner_data.weather_positions().len()
                ),
            )
            .await;
    }

    pub(crate) async fn scan_adjacencies(
        &self,
        overlay: &crate::scanner::file_overlay::FileOverlay,
    ) {
        let filter = self.get_sync_filter();
        let files: Vec<std::path::PathBuf> = overlay.winning_files_in("map");
        let result = tokio::task::spawn_blocking(move || {
            adjacency_scanner::scan_adjacency_files(&files, &filter)
        })
        .await
        .unwrap();
        self.scanner_data.set_adjacencies(result.adjacencies);
        self.scanner_data.adjacency_rules.clear();
        for (k, v) in result.rules {
            self.scanner_data
                .adjacency_rules
                .insert(k.into(), LayeredValue::new(v));
        }
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Loaded {} adjacencies, {} adjacency rules",
                    self.scanner_data.adjacencies().len(),
                    self.scanner_data.adjacency_rules.len()
                ),
            )
            .await;
    }

    pub(crate) async fn scan_strategic_regions(
        &self,
        overlay: &crate::scanner::file_overlay::FileOverlay,
    ) {
        let filter = self.get_sync_filter();
        let all_files: Vec<std::path::PathBuf> = overlay.winning_files_in("map/strategicregions");
        let filtered: Vec<std::path::PathBuf> = all_files
            .into_iter()
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e == "txt")
                    .unwrap_or(false)
            })
            .collect();
        let result = tokio::task::spawn_blocking(move || {
            strategic_region_scanner::scan_strategic_region_files(&filtered, &filter)
        })
        .await
        .unwrap();
        self.scanner_data.strategic_regions.clear();
        for (k, v) in result {
            self.scanner_data.strategic_regions.insert(k, v);
        }
        let count = self.scanner_data.strategic_regions.len();
        self.client
            .log_message(
                MessageType::INFO,
                format!("Loaded {} strategic regions", count),
            )
            .await;
    }

    pub(crate) async fn scan_terrains(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        scan_dashmap_overlay!(
            self,
            overlay,
            "common/terrain",
            terrain_scanner::scan_terrain_files,
            terrain_categories,
            &["txt"],
            "Loaded {} terrain categories"
        );
    }

    pub(crate) async fn scan_events(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        let filter = self.get_sync_filter();
        let mut files: Vec<std::path::PathBuf> = overlay.winning_files_in("events");
        files.retain(|p| {
            p.extension().is_some_and(|ext| ext == "txt")
                && p.parent()
                    .and_then(|par| par.file_name())
                    .is_some_and(|n| n == "events")
        });
        let file_count = files.len();
        let start = std::time::Instant::now();
        let result =
            tokio::task::spawn_blocking(move || event_scanner::scan_event_files(&files, &filter))
                .await
                .unwrap();
        let elapsed = start.elapsed();
        self.scanner_data.events.clear();
        for (k, v) in result {
            self.scanner_data
                .events
                .insert(k.into(), crate::data::layered_value::LayeredValue::new(v));
        }
        let count = self.scanner_data.events.len();

        // ── Rebuild event dependency graph ──────────────────────────
        self.scanner_data
            .event_dep_graph
            .rebuild_from_events_db(&self.scanner_data.events);

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Loaded {} events from {} files in {:.1?}",
                    count, file_count, elapsed,
                ),
            )
            .await;

        // ── Scan event namespaces from event files ───────────────────
        let ns_filter = self.get_sync_filter();
        let ns_files: Vec<std::path::PathBuf> = overlay
            .winning_files_in("events")
            .into_iter()
            .filter(|p| {
                p.extension().is_some_and(|ext| ext == "txt")
                    && p.parent()
                        .and_then(|par| par.file_name())
                        .is_some_and(|n| n == "events")
            })
            .collect();
        let ns_result = tokio::task::spawn_blocking(move || {
            crate::scanner::event_namespace_scanner::scan_event_namespaces(&ns_files, &ns_filter)
        })
        .await
        .unwrap();
        self.scanner_data.event_namespaces.clear();
        for (k, v) in ns_result {
            self.scanner_data
                .event_namespaces
                .insert(k.into(), crate::data::layered_value::LayeredValue::new(v));
        }

        let count = self.scanner_data.events.len();
        self.client
            .log_message(
                MessageType::INFO,
                format!("Total: Loaded {} event definitions", count),
            )
            .await;
    }

    pub(crate) async fn scan_focuses(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        scan_dashmap_overlay!(
            self,
            overlay,
            "common/national_focus",
            focus_scanner::scan_focus_files,
            focuses,
            &["txt"],
            "Total: Loaded {} national focus definitions"
        );
    }

    pub(crate) async fn scan_abilities(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        scan_dashmap_overlay!(
            self,
            overlay,
            "common/abilities",
            ability_scanner::scan_ability_files,
            abilities,
            &["txt"],
            "Total: Loaded {} abilities"
        );
    }

    pub(crate) async fn scan_ai_strategy_plans(
        &self,
        overlay: &crate::scanner::file_overlay::FileOverlay,
    ) {
        scan_dashmap_overlay!(
            self,
            overlay,
            "common/ai_strategy_plans",
            ai_strategy_plan_scanner::scan_ai_strategy_plan_files,
            ai_strategy_plans,
            &["txt"],
            "Total: Loaded {} AI strategy plans"
        );
    }

    pub(crate) async fn scan_ai_areas(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        scan_dashmap_overlay!(
            self,
            overlay,
            "common/ai_areas",
            ai_area_scanner::scan_ai_area_files,
            ai_areas,
            &["txt"],
            "Total: Loaded {} AI areas"
        );
    }

    pub(crate) async fn scan_continents(
        &self,
        overlay: &crate::scanner::file_overlay::FileOverlay,
    ) {
        let files: Vec<std::path::PathBuf> = overlay
            .winning_files_in("map")
            .into_iter()
            .filter(|p| p.to_string_lossy().ends_with("continent.txt"))
            .collect();
        let result =
            tokio::task::spawn_blocking(move || continent_scanner::scan_continent_files(&files))
                .await
                .unwrap();
        self.scanner_data.continents.clear();
        for (k, v) in result {
            self.scanner_data
                .continents
                .insert(k.into(), LayeredValue::new(v));
        }
        let c = &self.scanner_data.continents;
        self.client
            .log_message(
                MessageType::INFO,
                format!("Total: Loaded {} continent definitions", c.len()),
            )
            .await;
    }

    pub(crate) async fn scan_portraits(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        scan_dashmap_overlay!(
            self,
            overlay,
            "gfx/portraits",
            portrait_scanner::scan_portrait_files,
            portraits,
            &["txt"],
            "Total: Loaded {} portrait definitions"
        );
    }

    pub(crate) async fn scan_music(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        let filter = self.get_sync_filter();
        let files: Vec<std::path::PathBuf> = overlay.winning_files_in("music");
        let result =
            tokio::task::spawn_blocking(move || music_scanner::scan_music_files(&files, &filter))
                .await
                .unwrap();
        self.scanner_data.music_assets.clear();
        self.scanner_data.music_stations.clear();
        self.scanner_data.songs.clear();
        for (k, v) in result.assets {
            self.scanner_data
                .music_assets
                .insert(k.into(), LayeredValue::new(v));
        }
        for (k, v) in result.stations {
            self.scanner_data
                .music_stations
                .insert(k.into(), LayeredValue::new(v));
        }
        for (k, v) in result.songs {
            self.scanner_data
                .songs
                .insert(k.into(), LayeredValue::new(v));
        }
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Total: Loaded {} music assets, {} stations, and {} songs",
                    self.scanner_data.music_assets.len(),
                    self.scanner_data.music_stations.len(),
                    self.scanner_data.songs.len()
                ),
            )
            .await;
    }

    pub(crate) async fn scan_sounds(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        let filter = self.get_sync_filter();
        let mut files: Vec<std::path::PathBuf> = overlay.winning_files_in("sound");
        // Also include DLC and integrated DLC sound directories
        for dlc_prefix in ["dlc", "integrated_dlc"] {
            files.extend(
                overlay
                    .winning_files_in(dlc_prefix)
                    .into_iter()
                    .filter(|p| {
                        let s = p.to_string_lossy().replace('\\', "/");
                        s.contains("/sound/")
                    }),
            );
        }

        // Only .asset files contain sound definitions
        files.retain(|p| p.extension().is_some_and(|ext| ext == "asset"));
        let result =
            tokio::task::spawn_blocking(move || sound_scanner::scan_sound_files(&files, &filter))
                .await
                .unwrap();
        self.scanner_data.sounds.clear();
        self.scanner_data.sound_effects.clear();
        self.scanner_data.falloffs.clear();
        self.scanner_data.sound_categories.clear();
        for (k, v) in result.sounds {
            self.scanner_data
                .sounds
                .insert(k.into(), LayeredValue::new(v));
        }
        for (k, v) in result.sound_effects {
            self.scanner_data
                .sound_effects
                .insert(k.into(), LayeredValue::new(v));
        }
        for (k, v) in result.falloffs {
            self.scanner_data
                .falloffs
                .insert(k.into(), LayeredValue::new(v));
        }
        for (k, v) in result.categories {
            self.scanner_data
                .sound_categories
                .insert(k.into(), LayeredValue::new(v));
        }
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Total: Loaded {} sounds, {} sound effects, {} falloffs, and {} categories",
                    self.scanner_data.sounds.len(),
                    self.scanner_data.sound_effects.len(),
                    self.scanner_data.falloffs.len(),
                    self.scanner_data.sound_categories.len()
                ),
            )
            .await;
    }

    pub(crate) async fn scan_modifiers(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        let filter = self.get_sync_filter();
        let mut files: Vec<std::path::PathBuf> = Vec::new();
        files.extend(overlay.winning_files_in("common/modifiers"));
        files.extend(overlay.winning_files_in("common/dynamic_modifiers"));
        // Only .txt files contain modifier definitions
        files.retain(|p| p.extension().is_some_and(|ext| ext == "txt"));
        let result = tokio::task::spawn_blocking(move || {
            modifier_scanner::scan_modifier_files(&files, &filter)
        })
        .await
        .unwrap();
        self.scanner_data.custom_modifiers.clear();
        self.scanner_data.modifier_mappings.clear();
        for (k, v) in result.custom_modifiers {
            self.scanner_data
                .custom_modifiers
                .insert(k.into(), LayeredValue::new(v));
        }
        for (k, v) in result.builtin_mappings {
            self.scanner_data.modifier_mappings.insert(k.into(), v);
        }
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Total: Loaded {} custom modifiers and {} builtin mappings",
                    self.scanner_data.custom_modifiers.len(),
                    self.scanner_data.modifier_mappings.len()
                ),
            )
            .await;
    }

    pub(crate) async fn scan_buildings(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        scan_dashmap_overlay!(
            self,
            overlay,
            "common/buildings",
            building_scanner::scan_building_files,
            buildings,
            &["txt"],
            "Total: Loaded {} buildings"
        );
    }

    pub(crate) async fn scan_decisions(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        scan_dashmap_overlay!(
            self,
            overlay,
            "common/decisions",
            crate::scanner::decision_scanner::scan_decision_files,
            decisions,
            &["txt"],
            "Total: Loaded {} decisions"
        );

        // Also scan categories/*.txt for declared category names.
        // These are separate from decision definitions and don't get
        // recorded in the decisions DashMap.
        {
            let cat_files: Vec<std::path::PathBuf> = overlay
                .winning_files_in("common/decisions/categories")
                .into_iter()
                .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("txt"))
                .collect();
            let filter = self.get_sync_filter();
            let cats = tokio::task::spawn_blocking(move || {
                crate::scanner::decision_scanner::scan_category_declarations_files(
                    &cat_files, &filter,
                )
            })
            .await
            .unwrap();
            self.scanner_data.decision_categories.clear();
            for cat in cats {
                self.scanner_data.decision_categories.insert(
                    cat.into(),
                    crate::data::layered_value::LayeredValue::new(()),
                );
            }
            self.client
                .log_message(
                    MessageType::INFO,
                    format!(
                        "Total: Loaded {} decision categories",
                        self.scanner_data.decision_categories.len()
                    ),
                )
                .await;
        }
    }

    pub(crate) async fn scan_resources(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        scan_dashmap_overlay!(
            self,
            overlay,
            "common/resources",
            resource_scanner::scan_resource_files,
            resources,
            &["txt"],
            "Total: Loaded {} resources"
        );
    }

    pub(crate) async fn scan_state_categories(
        &self,
        overlay: &crate::scanner::file_overlay::FileOverlay,
    ) {
        scan_dashmap_overlay!(
            self,
            overlay,
            "common/state_category",
            state_category_scanner::scan_state_category_files,
            state_categories,
            &["txt"],
            "Total: Loaded {} state categories"
        );
    }

    pub(crate) async fn scan_achievements(
        &self,
        overlay: &crate::scanner::file_overlay::FileOverlay,
    ) {
        scan_dashmap_overlay!(
            self,
            overlay,
            "common/achievements",
            achievement_scanner::scan_achievement_files,
            achievements,
            &["txt"],
            "Total: Loaded {} achievements"
        );
    }

    pub(crate) async fn scan_balance_of_powers(
        &self,
        overlay: &crate::scanner::file_overlay::FileOverlay,
    ) {
        scan_dashmap_overlay!(
            self,
            overlay,
            "common/balance_of_power",
            bop_scanner::scan_balance_of_power_files,
            balance_of_powers,
            &["txt"],
            "Total: Loaded {} balance of power definitions"
        );
    }

    pub(crate) async fn scan_defines(&self, roots: &[std::path::PathBuf]) {
        // Defines merge by key — keep the old per-root approach
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

    pub(crate) async fn scan_variables(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        let filter = self.get_sync_filter();
        let files: Vec<std::path::PathBuf> = overlay
            .winning_files_in("")
            .into_iter()
            .filter(|p| {
                // Only process .txt files (skip .yml, .gui, .asset, .csv, .lua, .mod)
                p.extension().is_some_and(|ext| ext == "txt")
            })
            .filter(|p| {
                // Skip non-script directories (mirrors old scan_roots logic)
                let s = p.to_string_lossy().replace('\\', "/");
                !s.contains("/map/")
                    && !s.contains("/interface/")
                    && !s.contains("/gfx/")
                    && !s.contains("/localisation/")
            })
            .collect();
        let result = tokio::task::spawn_blocking(move || {
            variable_scanner::scan_variable_files(&files, &filter)
        })
        .await
        .unwrap();

        self.scanner_data.variables.clear();
        for (k, v) in result.variables {
            self.scanner_data.variables.insert(k.into(), v);
        }

        self.scanner_data.event_targets.clear();
        for (k, v) in result.event_targets {
            self.scanner_data.event_targets.insert(k.into(), v);
        }

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Total: Loaded {} variables and {} event targets",
                    self.scanner_data.variables.len(),
                    self.scanner_data.event_targets.len()
                ),
            )
            .await;
    }

    pub(crate) async fn scan_localization(&self, roots: &[std::path::PathBuf]) {
        // Localization merges by key — keep the old per-root approach
        self.client
            .log_message(
                MessageType::INFO,
                format!("Starting localization scan in {} roots", roots.len()),
            )
            .await;

        let filter = self.get_sync_filter();
        let roots_owned = roots.to_vec();

        let (per_root_locs, dups, game_keys, logs) = tokio::task::spawn_blocking(move || {
            let mut per_root_locs: Vec<HashMap<InternedStr, loc_parser::LocEntry>> = Vec::new();
            let mut dups = HashSet::new();
            let mut game_keys = HashSet::new();
            let mut seen_locs_by_lang: HashSet<(String, InternedStr)> = HashSet::new();
            let mut logs = Vec::new();
            let has_game_root = roots_owned.len() > 1;

            for (root_idx, root) in roots_owned.iter().enumerate() {
                let is_game_root = has_game_root && root_idx == 0;
                let loc_dir = root.join("localisation");
                if !loc_dir.exists() {
                    per_root_locs.push(HashMap::new());
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

                let mut root_locs = HashMap::new();
                let mut root_file_count = 0u32;
                let mut root_key_count: usize = 0;
                for path in files_to_scan {
                    match std::fs::read_to_string(&path) {
                        Ok(content) => {
                            let path_str = path.to_string_lossy().to_string();
                            let (parsed, _, doc_lang) =
                                loc_parser::parse_loc_file(&content, &path_str);
                            let lang_str = doc_lang.unwrap_or_else(|| "unknown".to_string());

                            root_file_count += 1;
                            root_key_count += parsed.len();

                            if parsed.is_empty() {
                                logs.push((
                                    MessageType::WARNING,
                                    format!("No keys found in localization file: {:?}", path),
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
                                root_locs.insert(key, entry);
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
                if root_file_count > 0 {
                    let root_label = if is_game_root {
                        "game path"
                    } else {
                        "workspace root"
                    };
                    logs.push((
                        MessageType::INFO,
                        format!(
                            "Loaded {} keys from {} files ({})",
                            root_key_count, root_file_count, root_label,
                        ),
                    ));
                }
                per_root_locs.push(root_locs);
            }
            (per_root_locs, dups, game_keys, logs)
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

        self.scanner_data.game_loc_keys.clear();
        for x in game_keys {
            self.scanner_data.game_loc_keys.insert((x.0.into(), x.1));
        }
        let game_loc_count = self.scanner_data.game_loc_keys.len();

        self.scanner_data.localization.clear();
        for root_locs in per_root_locs {
            for (k, v) in root_locs {
                let ik: InternedStr = k.clone();
                match self.scanner_data.localization.get_mut(&ik) {
                    Some(mut existing) => existing.push(v),
                    None => {
                        self.scanner_data
                            .localization
                            .insert(k, LayeredValue::new(v));
                    }
                }
            }
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

    pub(crate) async fn scan_scripted(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        let filter = self.get_sync_filter();
        let trig_files: Vec<std::path::PathBuf> =
            overlay.winning_files_in("common/scripted_triggers");
        let eff_files: Vec<std::path::PathBuf> =
            overlay.winning_files_in("common/scripted_effects");
        let loc_files: Vec<std::path::PathBuf> =
            overlay.winning_files_in("common/scripted_localisation");
        let result = tokio::task::spawn_blocking(move || {
            (
                scripted_scanner::scan_scripted_files(&trig_files, &filter),
                scripted_scanner::scan_scripted_files(&eff_files, &filter),
                scripted_loc_scanner::scan_scripted_loc_files(&loc_files, &filter),
            )
        })
        .await
        .unwrap();

        self.scanner_data.scripted_triggers.clear();
        self.scanner_data.scripted_effects.clear();
        self.scanner_data.scripted_locs.clear();

        for (k, v) in result.0 {
            self.scanner_data
                .scripted_triggers
                .insert(k.into(), LayeredValue::new(v));
        }
        for (k, v) in result.1 {
            self.scanner_data
                .scripted_effects
                .insert(k.into(), LayeredValue::new(v));
        }
        for (k, v) in result.2 {
            self.scanner_data
                .scripted_locs
                .insert(k.into(), LayeredValue::new(v));
        }

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Total: Loaded {} scripted triggers, {} scripted effects, {} scripted locs",
                    self.scanner_data.scripted_triggers.len(),
                    self.scanner_data.scripted_effects.len(),
                    self.scanner_data.scripted_locs.len()
                ),
            )
            .await;
    }

    pub(crate) async fn scan_ideologies(
        &self,
        overlay: &crate::scanner::file_overlay::FileOverlay,
    ) {
        let filter = self.get_sync_filter();
        let files: Vec<std::path::PathBuf> = overlay.winning_files_in("common/ideologies");
        let result = tokio::task::spawn_blocking(move || {
            let ideologies = ideology_scanner::scan_ideology_files(&files, &filter);
            let mut sub_map: HashMap<String, (String, ast::Range, String)> = HashMap::new();
            for ideology in ideologies.values() {
                for (sub, range) in &ideology.sub_ideology_ranges {
                    sub_map.insert(
                        sub.clone(),
                        (
                            ideology.name.clone(),
                            range.clone(),
                            ideology.path.to_string(),
                        ),
                    );
                }
            }
            (ideologies, sub_map)
        })
        .await
        .unwrap();

        self.scanner_data.ideologies.clear();
        self.scanner_data.sub_ideologies.clear();

        for (k, v) in result.0 {
            self.scanner_data
                .ideologies
                .insert(k.into(), LayeredValue::new(v));
        }
        for (k, v) in result.1 {
            let wrapped = (InternedStr::from(v.0), v.1, InternedStr::from(v.2));
            self.scanner_data
                .sub_ideologies
                .insert(k.into(), LayeredValue::new(wrapped));
        }

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Total: Loaded {} ideologies and {} sub-ideologies",
                    self.scanner_data.ideologies.len(),
                    self.scanner_data.sub_ideologies.len()
                ),
            )
            .await;
    }

    pub(crate) async fn scan_traits(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        let filter = self.get_sync_filter();
        let ul_files: Vec<std::path::PathBuf> = overlay.winning_files_in("common/unit_leader");
        let cl_files: Vec<std::path::PathBuf> = overlay.winning_files_in("common/country_leader");
        let tr_files: Vec<std::path::PathBuf> = overlay.winning_files_in("common/traits");
        let result = tokio::task::spawn_blocking(move || {
            let mut map = HashMap::new();
            for (files, trait_type) in [
                (&ul_files as &[std::path::PathBuf], "Unit Leader Trait"),
                (&cl_files, "Country Leader Trait"),
                (&tr_files, "Trait"),
            ] {
                map.extend(trait_scanner::scan_trait_files(files, trait_type, &filter));
            }
            map
        })
        .await
        .unwrap();

        self.scanner_data.traits.clear();
        for (k, v) in result {
            self.scanner_data
                .traits
                .insert(k.into(), LayeredValue::new(v));
        }

        self.client
            .log_message(
                MessageType::INFO,
                format!("Total: Loaded {} traits", self.scanner_data.traits.len()),
            )
            .await;
    }

    pub(crate) async fn scan_sprites(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        scan_dashmap_overlay!(
            self,
            overlay,
            "interface",
            sprite_scanner::scan_sprite_files,
            sprites,
            &["gfx", "gui"],
            "Total: Loaded {} sprite definitions"
        );
    }

    pub(crate) async fn scan_characters(
        &self,
        overlay: &crate::scanner::file_overlay::FileOverlay,
    ) {
        scan_dashmap_overlay!(
            self,
            overlay,
            "common/characters",
            character_scanner::scan_character_files,
            characters,
            &["txt"],
            "Total: Loaded {} characters"
        );
    }

    pub(crate) async fn scan_ideas(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        scan_dashmap_overlay!(
            self,
            overlay,
            "common/ideas",
            idea_scanner::scan_idea_files,
            ideas,
            &["txt"],
            "Total: Loaded {} ideas"
        );
    }

    pub(crate) async fn scan_gfx(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        scan_dashmap_overlay!(
            self,
            overlay,
            "interface",
            gfx_scanner::scan_color_code_files,
            color_codes,
            &["gfx"],
            "Total: Loaded {} color codes from interface/*.gfx"
        );
    }

    pub(crate) async fn scan_countries(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        let filter = self.get_sync_filter();
        let mut files: Vec<std::path::PathBuf> = Vec::new();
        files.extend(overlay.winning_files_in("common/country_tags"));
        files.extend(overlay.winning_files_in("common/countries"));
        files.extend(overlay.winning_files_in("history/countries"));
        // Only .txt files contain country tag definitions
        files.retain(|p| p.extension().is_some_and(|ext| ext == "txt"));
        let result = tokio::task::spawn_blocking(move || {
            country_scanner::scan_country_tag_files(&files, &filter)
        })
        .await
        .unwrap();
        self.scanner_data.country_tags.clear();
        for (k, v) in result {
            self.scanner_data
                .country_tags
                .insert(k.into(), crate::data::layered_value::LayeredValue::new(v));
        }
        let count = self.scanner_data.country_tags.len();
        self.client
            .log_message(MessageType::INFO, format!("Loaded {} country tags", count))
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

    pub(crate) async fn scan_oobs(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        let filter = self.get_sync_filter();
        let files: Vec<std::path::PathBuf> = overlay
            .winning_files_in("history/units")
            .into_iter()
            .filter(|p| p.extension().is_some_and(|ext| ext == "txt"))
            .collect();
        let result =
            tokio::task::spawn_blocking(move || oob_scanner::scan_oob_files(&files, &filter))
                .await
                .unwrap();
        self.scanner_data.oob_division_templates.clear();
        self.scanner_data.oob_fleets.clear();
        for (k, v) in result.division_templates {
            self.scanner_data
                .oob_division_templates
                .insert(k.into(), crate::data::layered_value::LayeredValue::new(v));
        }
        for (k, v) in result.fleets {
            self.scanner_data
                .oob_fleets
                .insert(k.into(), crate::data::layered_value::LayeredValue::new(v));
        }
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Total: Loaded {} OOB division templates and {} fleets",
                    self.scanner_data.oob_division_templates.len(),
                    self.scanner_data.oob_fleets.len()
                ),
            )
            .await;
    }

    pub(crate) async fn scan_units(&self, overlay: &crate::scanner::file_overlay::FileOverlay) {
        let filter = self.get_sync_filter();
        let files: Vec<std::path::PathBuf> = overlay
            .winning_files_in("common/units")
            .into_iter()
            .filter(|p| p.extension().is_some_and(|ext| ext == "txt"))
            .collect();
        let result =
            tokio::task::spawn_blocking(move || unit_scanner::scan_unit_files(&files, &filter))
                .await
                .unwrap();
        self.scanner_data.unit_types.clear();
        for (k, v) in result {
            self.scanner_data
                .unit_types
                .insert(k.into(), crate::data::layered_value::LayeredValue::new(v));
        }
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Total: Loaded {} unit types from common/units",
                    self.scanner_data.unit_types.len()
                ),
            )
            .await;
    }
}
