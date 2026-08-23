//! Tests for the technology scanner + technology tags scanner.
//!
//! Extraction is exercised through `parser::parse_script` on realistic
//! vanilla-format script text (mirrors `common/technologies/infantry.txt`
//! and `common/technology_tags/00_technology.txt`), not hand-built ASTs —
//! that way a parser change that breaks extraction fails here too.

use crate::parser::parser;
use crate::scanner::tech_dep_graph::TechDependencyGraph;
use crate::scanner::technology_scanner::{Technology, find_technologies_in_entries};
use crate::scanner::technology_tags_scanner::{TechnologyTagKind, find_tags_in_entries};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;

/// Realistic vanilla infantry tech (subset of common/technologies/infantry.txt).
const VANILLA_TECH: &str = r#"
technologies = {

	@1918 = 0
	@1936 = 2

	infantry_weapons = {

		enable_equipments = {
			infantry_equipment_0
		}

		enable_equipment_modules = {
			tank_heavy_machine_gun
			secondary_turret_hmg
		}

		enable_subunits = {
			infantry
		}

		path = {
			leads_to_tech = infantry_weapons1
			research_cost_coeff = 1
		}

		research_cost = 1.5
		start_year = 1918
		folder = {
			name = infantry_folder
			position = { x = 0 y = -1 }
		}

		categories = {
			infantry_weapons
		}

		ai_will_do = {
			factor = 1
		}
	}

	infantry_weapons1 = {
		enable_equipments = {
			infantry_equipment_1
		}
		path = {
			leads_to_tech = infantry_weapons2
		}
		research_cost = 1.5
		start_year = 1936
		folder = {
			name = infantry_folder
			position = { x = 0 y = @1936 }
		}
		categories = {
			infantry_weapons
		}
	}

	xor_tech_a = {
		xor = {
			xor_tech_b
		}
		dependencies = {
			rocket_artillery = 1
		}
		enable_building = {
			building = industrial_complex
			level = 20
		}
		sub_technologies = {
			motorized_rocket_unit
		}
		start_year = 1939
		research_cost = 2
	}
}
"#;

fn scan_technologies(source: &str) -> HashMap<String, Technology> {
    let (script, _) = parser::parse_script(source);
    let mut map = HashMap::new();
    find_technologies_in_entries(
        &script.entries,
        &script.source,
        "common/technologies/test.txt",
        &mut map,
    );
    map
}

#[test]
fn test_extracts_basic_fields() {
    let techs = scan_technologies(VANILLA_TECH);

    // The @1918/@1936 macro defines must NOT be extracted as technologies.
    assert_eq!(techs.len(), 3, "macro defines must be skipped");
    assert!(techs.contains_key("infantry_weapons"));
    assert!(techs.contains_key("infantry_weapons1"));
    assert!(techs.contains_key("xor_tech_a"));

    let iw = &techs["infantry_weapons"];
    assert_eq!(iw.start_year, Some(1918));
    assert_eq!(iw.research_cost, Some(1.5));
    assert_eq!(iw.folder.as_deref(), Some("infantry_folder"));
    assert_eq!(
        iw.categories,
        vec!["infantry_weapons".to_string()],
        "categories are bare identifiers in a block"
    );
    assert_eq!(
        iw.leads_to_tech,
        vec!["infantry_weapons1".to_string()],
        "path.leads_to_tech collected"
    );
    assert_eq!(
        iw.enable_equipments,
        vec!["infantry_equipment_0".to_string()]
    );
    assert_eq!(
        iw.enable_equipment_modules,
        vec![
            "tank_heavy_machine_gun".to_string(),
            "secondary_turret_hmg".to_string()
        ]
    );
    assert_eq!(iw.enable_subunits, vec!["infantry".to_string()]);
    assert_eq!(iw.path.as_ref(), "common/technologies/test.txt");
}

#[test]
fn test_extracts_xor_dependencies_building_subtechs() {
    let techs = scan_technologies(VANILLA_TECH);
    let xta = &techs["xor_tech_a"];

    assert_eq!(xta.xor, vec!["xor_tech_b".to_string()]);
    assert_eq!(
        xta.dependencies,
        vec!["rocket_artillery".to_string()],
        "only dependencies with value = 1 are collected"
    );
    assert_eq!(
        xta.enable_building,
        Some(("industrial_complex".to_string(), 20))
    );
    assert_eq!(
        xta.sub_technologies,
        vec!["motorized_rocket_unit".to_string()]
    );
    // research_cost written as integer literal parses as f64
    assert_eq!(xta.research_cost, Some(2.0));
}

#[test]
fn test_position_macro_reference_does_not_break_folder() {
    let techs = scan_technologies(VANILLA_TECH);
    let iw1 = &techs["infantry_weapons1"];
    // folder position uses `y = @1936` — folder name still extracts fine.
    assert_eq!(iw1.folder.as_deref(), Some("infantry_folder"));
}

/// Realistic vanilla technology_tags file.
const VANILLA_TAGS: &str = r#"
technology_categories = {
	light_air
	medium_air
	armor
	infantry_weapons
	cat_mechanized_equipment
}

technology_folders = {
	infantry_folder = {
		ledger = army
	}

	land_doctrine_folder = {
		ledger = army
		doctrine = yes
	}

	naval_folder = {
		ledger = navy
	}
}
"#;

fn scan_tags(
    source: &str,
) -> HashMap<String, crate::scanner::technology_tags_scanner::TechnologyTag> {
    let (script, _) = parser::parse_script(source);
    let mut map = HashMap::new();
    find_tags_in_entries(
        &script.entries,
        &script.source,
        "common/technology_tags/test.txt",
        &mut map,
    );
    map
}

#[test]
fn test_extracts_categories_as_bare_identifiers() {
    let tags = scan_tags(VANILLA_TAGS);

    for cat in ["light_air", "medium_air", "armor", "infantry_weapons"] {
        let tag = tags
            .get(cat)
            .unwrap_or_else(|| panic!("category {} missing", cat));
        assert_eq!(tag.tag_kind, TechnologyTagKind::Category);
        assert_eq!(tag.name, cat);
        assert!(!tag.doctrine);
        assert!(tag.ledger.is_none());
    }
    assert_eq!(tags.len(), 8, "5 categories + 3 folders");
}

#[test]
fn test_extracts_folders_with_ledger_and_doctrine() {
    let tags = scan_tags(VANILLA_TAGS);

    let inf = &tags["infantry_folder"];
    assert_eq!(inf.tag_kind, TechnologyTagKind::Folder);
    assert_eq!(inf.ledger.as_deref(), Some("army"));
    assert!(!inf.doctrine);

    let doc = &tags["land_doctrine_folder"];
    assert_eq!(doc.ledger.as_deref(), Some("army"));
    assert!(doc.doctrine, "doctrine = yes must be captured");

    let nav = &tags["naval_folder"];
    assert_eq!(nav.ledger.as_deref(), Some("navy"));
}

#[test]
fn test_incremental_update_replaces_file_entries() {
    use crate::ScannerData;
    use crate::scanner::incremental_scanner::{
        remove_path_from_scanner_data, update_scanner_data_for_file,
    };

    let data = ScannerData::new();
    let path = "/mod/common/technologies/my_techs.txt";

    update_scanner_data_for_file(&data, path, VANILLA_TECH);
    assert_eq!(data.technologies.len(), 3);
    assert!(data.technologies.contains_key("infantry_weapons"));

    // File index populated for O(K) incremental updates
    let idx_keys = data
        .technologies_file_index
        .get(&crate::scanner::incremental_scanner::index_key(path))
        .map(|v| v.value().len())
        .unwrap_or(0);
    assert_eq!(idx_keys, 3, "file index lists all techs in this file");

    // Re-save with one tech removed → old entries replaced, not duplicated
    update_scanner_data_for_file(
        &data,
        path,
        "technologies = { infantry_weapons = { start_year = 1918 } }",
    );
    assert_eq!(data.technologies.len(), 1);
    assert!(data.technologies.contains_key("infantry_weapons"));
    assert!(!data.technologies.contains_key("xor_tech_a"));

    // Delete the file → all its techs removed, dep graph scrubbed
    remove_path_from_scanner_data(&data, path);
    assert!(data.technologies.is_empty());
}

#[test]
fn test_incremental_update_maintains_dep_graph() {
    use crate::ScannerData;
    use crate::scanner::incremental_scanner::update_scanner_data_for_file;

    let data = ScannerData::new();
    let path = "/mod/common/technologies/tree.txt";

    update_scanner_data_for_file(&data, path, VANILLA_TECH);

    // infantry_weapons -> infantry_weapons1 -> infantry_weapons2 edge exists
    assert!(
        data.tech_dep_graph
            .callees_of("infantry_weapons")
            .contains(&"infantry_weapons1".to_string())
    );
    assert_eq!(
        data.tech_dep_graph.caller_count("infantry_weapons1"),
        1,
        "one caller: infantry_weapons"
    );
    // xor_tech_a has no incoming edges
    assert!(data.tech_dep_graph.is_orphaned("xor_tech_a"));
    assert!(!data.tech_dep_graph.is_orphaned("infantry_weapons1"));

    // Rewrite the file so infantry_weapons no longer leads anywhere —
    // the stale edge must be scrubbed incrementally (no full rebuild).
    update_scanner_data_for_file(
        &data,
        path,
        "technologies = { infantry_weapons = { start_year = 1918 } }",
    );
    assert!(
        data.tech_dep_graph
            .callees_of("infantry_weapons")
            .is_empty()
    );
    assert!(
        data.tech_dep_graph.is_orphaned("infantry_weapons1"),
        "stale edge removed; infantry_weapons1 now has no callers"
    );
}

#[test]
fn test_dep_graph_rebuild_from_db() {
    let db: DashMap<Arc<str>, crate::data::layered_value::LayeredValue<Technology>> =
        DashMap::new();

    let mk = |name: &str, leads: &[&str]| Technology {
        name: name.to_string(),
        path: Arc::from("common/technologies/t.txt"),
        range: crate::parser::ast::Range {
            start_line: 1,
            start_col: 0,
            end_line: 1,
            end_col: 10,
        },
        start_year: None,
        research_cost: None,
        categories: Vec::new(),
        folder: None,
        leads_to_tech: leads.iter().map(|s| s.to_string()).collect(),
        xor: Vec::new(),
        dependencies: Vec::new(),
        enable_subunits: Vec::new(),
        enable_equipments: Vec::new(),
        enable_equipment_modules: Vec::new(),
        enable_building: None,
        sub_technologies: Vec::new(),
    };

    db.insert(
        Arc::from("a"),
        crate::data::layered_value::LayeredValue::new(mk("a", &["b", "c"])),
    );
    db.insert(
        Arc::from("b"),
        crate::data::layered_value::LayeredValue::new(mk("b", &["c"])),
    );

    let graph = TechDependencyGraph::new();
    graph.rebuild_from_technologies_db(&db);

    assert_eq!(graph.callees_of("a").len(), 2);
    assert_eq!(graph.caller_count("c"), 2);
    assert!(graph.is_orphaned("a"));
    assert!(!graph.is_orphaned("b"));
}

#[test]
fn test_entity_lookup_finds_technology() {
    use crate::ScannerData;
    use crate::data::entity_lookup::{EntityKind, EntityLookup};

    let data = ScannerData::new();
    let path = "/mod/common/technologies/test.txt";
    crate::scanner::incremental_scanner::update_scanner_data_for_file(&data, path, VANILLA_TECH);

    let lookup = EntityLookup::new(&data);

    // find_definition by key
    let defs = lookup.find_definition("infantry_weapons");
    assert!(!defs.is_empty(), "goto-definition resolves tech key");
    assert!(defs.iter().any(|d| d.kind == EntityKind::Technology));

    // entity_names classifies the kind
    let names = lookup.entity_names();
    assert_eq!(names.get("infantry_weapons"), Some(&EntityKind::Technology));

    // entity_at position inside the tech block — locate the header line
    // programmatically rather than hardcoding it.
    let content = VANILLA_TECH;
    let byte_off = content
        .find("\tinfantry_weapons = {")
        .expect("VANILLA_TECH must contain infantry_weapons block");
    let prefix = &content[..byte_off];
    let line = prefix.matches('\n').count() as u32; // 0-based line index
    let at = lookup.entity_at(
        path,
        content,
        tower_lsp_server::ls_types::Position { line, character: 4 },
    );
    assert!(
        at.is_some(),
        "entity_at should resolve cursor inside tech block"
    );
    let (kind, _, name) = at.unwrap();
    assert_eq!(kind, EntityKind::Technology);
    assert_eq!(name, "infantry_weapons");
}

#[test]
fn test_entity_lookup_finds_technology_tag() {
    use crate::ScannerData;
    use crate::data::entity_lookup::{EntityKind, EntityLookup};

    let data = ScannerData::new();
    let path = "/mod/common/technology_tags/test.txt";
    crate::scanner::incremental_scanner::update_scanner_data_for_file(&data, path, VANILLA_TAGS);

    let lookup = EntityLookup::new(&data);

    let defs = lookup.find_definition("infantry_folder");
    assert!(!defs.is_empty(), "goto-definition resolves folder key");
    assert!(defs.iter().any(|d| d.kind == EntityKind::TechnologyTag));

    let names = lookup.entity_names();
    assert_eq!(
        names.get("light_air"),
        Some(&EntityKind::TechnologyTag),
        "categories and folders both land in entity_names"
    );
}

#[test]
fn test_scope_resolution_for_technology_tag_containers() {
    use crate::scope::scope::{Scope, ScopeCtx, ScopeStack};

    // The two container keys resolve to their structural scopes (not the
    // Country wildcard list).
    assert_eq!(
        Scope::from_str("technology_categories"),
        Scope::TechnologyCategories
    );
    assert_eq!(
        Scope::from_str("technology_folders"),
        Scope::TechnologyFolders
    );

    // Both are structural containers: effective_scope passes through
    // unchanged (like Ace), so a stray trigger at container level is
    // flagged rather than silently allowed as Country.
    assert_eq!(
        Scope::TechnologyCategories.effective_scope(),
        Scope::TechnologyCategories
    );
    assert_eq!(
        Scope::TechnologyFolders.effective_scope(),
        Scope::TechnologyFolders
    );

    // A folder's `available` trigger block resolves to Country scope inside
    // the stack — the engine evaluates it per-country.
    let sctx = ScopeCtx {
        uri: "/mod/common/technology_tags/test.txt",
        event_targets: None,
        characters: None,
        achievements: None,
        in_random_list: false,
        state_targeted: false,
    };
    let mut stack = ScopeStack::new(Scope::Global);
    let (s, _) = stack.resolve_entry_scope("technology_folders", &sctx);
    assert_eq!(s, Scope::TechnologyFolders);
    stack.push(s);
    let (s2, _) = stack.resolve_entry_scope("available", &sctx);
    assert_eq!(
        s2.effective_scope(),
        Scope::Country,
        "folder `available` block is country-scoped for validation"
    );
}

#[test]
fn test_initial_scope_for_technology_files() {
    use crate::scope::scope::{Scope, initial_scope_for_uri};

    // File-level scopes replace the misleading Global default.
    assert_eq!(
        initial_scope_for_uri("/mod/common/technology_tags/00_technology.txt"),
        Scope::TechnologyTags
    );
    assert_eq!(
        initial_scope_for_uri("/mod/common/technologies/infantry.txt"),
        Scope::Technologies
    );
    // Doctrine files share common/technologies/.
    assert_eq!(
        initial_scope_for_uri("/vanilla/common/technologies/naval_doctrine.txt"),
        Scope::Technologies
    );
    // Unrelated files keep Global.
    assert_eq!(
        initial_scope_for_uri("/mod/common/events/test.txt"),
        Scope::Global
    );

    // Technologies file scope is Country-effective: tech bodies are
    // evaluated per-country (modifiers grant to the researcher;
    // ai_will_do/available/on_research_complete run in country scope),
    // so the 49 modifier keys used directly in vanilla tech blocks keep
    // validating as Country instead of false-positive HOM004.
    assert_eq!(
        Scope::Technologies.effective_scope(),
        Scope::Country,
        "tech bodies are country-evaluated — NationalFocus precedent"
    );
}

#[test]
fn test_data_json_documents_technology_properties() {
    use crate::data::hoi4_data::{lookup_entity, lookup_parameter, lookup_pushes_scope};
    use crate::scope::scope::Scope;

    // The unlock blocks Embi flagged as undocumented.
    for key in [
        "enable_equipments",
        "enable_subunits",
        "enable_equipment_modules",
        "enable_building",
    ] {
        let e = lookup_entity(key).unwrap_or_else(|| panic!("{key} missing from data JSON"));
        assert_eq!(e.name, key);
    }

    // enable_building documents its structured sub-keys (building/level).
    assert!(lookup_parameter("enable_building", "building").is_some());
    assert!(lookup_parameter("enable_building", "level").is_some());

    // Tree structure + cost/display properties.
    for key in [
        "path",
        "folder",
        "categories",
        "dependencies",
        "XOR",
        "sub_technologies",
        "sub_tech_index",
        "start_year",
        "research_cost",
        "show_equipment_icon",
        "force_use_small_tech_layout",
        "show_effect_as_desc",
        "desc",
        "doctrine_name",
        "xp_research_type",
        "xp_boost_cost",
        "xp_research_bonus",
        "xp_unlock_cost",
        "doctrine",
        "is_special_project_tech",
        "special_project_specialization",
        "ai_research_weights",
        "on_research_complete_limit",
    ] {
        assert!(lookup_entity(key).is_some(), "{key} missing from data JSON");
    }

    // path's structured params resolve (leads_to_tech drives hover).
    let leads = lookup_parameter("path", "leads_to_tech").expect("path.leads_to_tech documented");
    assert_eq!(leads.value_type, "Technology");

    // Gate blocks push Country so their bodies validate as country triggers.
    for key in [
        "allow",
        "allow_branch",
        "available",
        "visible",
        "on_research_complete_limit",
    ] {
        assert_eq!(
            lookup_pushes_scope(key),
            Some(Scope::Country),
            "{key} must push Country"
        );
    }

    // Multi-home keys carry Global so units/focuses/operations files don't
    // flag them under HOM004.
    for key in ["path", "folder", "categories", "desc"] {
        let e = lookup_entity(key).expect(key);
        assert!(
            e.scopes.usage.contains(&Scope::Global),
            "{key} must include Global usage"
        );
    }
}

#[test]
fn test_technology_keywords_seeded_from_data() {
    // build_static_semantic_keywords seeds from EFFECTS keys — every newly
    // documented tech property highlights without a hand-added keyword line.
    use crate::backend::build_static_semantic_keywords;
    let kw = build_static_semantic_keywords();
    for key in [
        "enable_equipments",
        "enable_subunits",
        "enable_equipment_modules",
        "enable_building",
        "path",
        "start_year",
        "research_cost",
        "show_equipment_icon",
        "force_use_small_tech_layout",
        "show_effect_as_desc",
        "xp_research_type",
        "special_project_specialization",
        "ai_research_weights",
        "allow_branch",
        "allow",
        "visible",
    ] {
        assert!(kw.contains(key), "keyword set missing {key}");
    }
    // Structured sub-keys (leads_to_tech etc.) are NOT global keywords —
    // they're context-aware properties resolved against their parent block,
    // so they only highlight inside `path = { ... }`.
    use crate::data::hoi4_data::{lookup_parameter, lookup_parameter_with_anchor};
    assert!(
        !kw.contains("leads_to_tech"),
        "sub-keys must stay context-aware, not global keywords"
    );
    assert!(lookup_parameter("path", "leads_to_tech").is_some());
    assert!(!kw.contains("research_cost_coeff"));
    assert!(lookup_parameter("path", "research_cost_coeff").is_some());
    // Folder instance params still come from the container table (not
    // entities) — ledger/doctrine stay Property inside folder blocks because
    // lookup_parameter_with_anchor checks parent params BEFORE entity status.
    assert!(lookup_parameter("technology_folders", "ledger").is_some());
    let (owner, _) = lookup_parameter_with_anchor(
        Some("land_doctrine_folder"),
        Some("technology_folders"),
        "doctrine",
    )
    .expect("doctrine resolves via anchor");
    assert_eq!(owner, "technology_folders");
}

#[test]
fn test_data_json_documents_technology_tag_blocks() {
    use crate::data::hoi4_data::{lookup_entity, lookup_parameter};

    // Both blocks are documented entities → they land in the static keyword
    // set via build_static_semantic_keywords() and highlight automatically.
    let cats = lookup_entity("technology_categories").expect("technology_categories in data JSON");
    assert_eq!(cats.name, "technology_categories");

    let folders = lookup_entity("technology_folders").expect("technology_folders in data JSON");
    assert_eq!(folders.name, "technology_folders");

    // Folder params resolve for hover/completion/semantic tokens.
    let ledger = lookup_parameter("technology_folders", "ledger")
        .expect("ledger is a documented folder param");
    assert!(ledger.description.contains("ledger"));
    assert!(lookup_parameter("technology_folders", "doctrine").is_some());
    assert!(lookup_parameter("technology_folders", "available").is_some());
    // Case-insensitive like every other entity lookup.
    assert!(lookup_parameter("TECHNOLOGY_FOLDERS", "Ledger").is_some());
}

// ---------------------------------------------------------------------------
// Dep-graph scrub on file DELETION — index-key spelling invariant
// ---------------------------------------------------------------------------

/// Deleting a technology file must scrub its edges from `tech_dep_graph`.
/// The delete arm reads the ID list from `technologies_file_index` — that
/// read MUST go through `index_key()` (the only sanctioned index-key builder,
/// see windows-path-normalization-2026-08-05) so any path spelling matches
/// the normalized keys the macros write. The update path already keys its own
/// read correctly; this pins the delete side to the same invariant.
///
/// Regression guard for the raw `.get(path_str)` miss in the
/// FileCategory::Technologies arm of `remove_path_from_scanner_data`.
#[test]
fn test_delete_scrubs_tech_dep_graph_with_backslash_path_spelling() {
    use crate::ScannerData;
    use crate::scanner::incremental_scanner::{
        remove_path_from_scanner_data, update_scanner_data_for_file,
    };

    let data = ScannerData::new();

    // Insert with the forward-slash spelling (what the LSP handlers feed).
    let insert_path = "/mod/common/technologies/tree.txt";
    update_scanner_data_for_file(&data, insert_path, VANILLA_TECH);
    assert_eq!(
        data.tech_dep_graph.caller_count("infantry_weapons1"),
        1,
        "edge infantry_weapons -> infantry_weapons1 exists before deletion"
    );

    // Delete via the same file under the OTHER separator convention. The
    // macros normalize both sides, so removal itself works either way — but
    // the stale-edge scrub silently no-ops when the pre-read misses.
    let delete_path = "\\mod\\common\\technologies\\tree.txt";
    remove_path_from_scanner_data(&data, delete_path);

    assert!(
        data.technologies.is_empty(),
        "technologies map must be empty after deletion"
    );
    assert_eq!(
        data.tech_dep_graph.callers_of("infantry_weapons1"),
        Vec::<String>::new(),
        "dep graph must be scrubbed on deletion regardless of path spelling"
    );
}
