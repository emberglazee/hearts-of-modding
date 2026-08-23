#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A technology defined in `common/technologies/*.txt`.
///
/// Each technology is a block inside `technologies = { tech_name = { ... } }`.
/// Technologies include researchable techs, doctrines, and hidden techs that
/// unlock equipment/subunits/buildings.
#[derive(Debug, Clone)]
pub struct Technology {
    pub name: String,
    pub path: InternedStr,
    pub range: ast::Range,
    pub start_year: Option<i32>,
    pub research_cost: Option<f64>,
    pub categories: Vec<String>,
    pub folder: Option<String>,
    pub leads_to_tech: Vec<String>,
    pub xor: Vec<String>,
    pub dependencies: Vec<String>,
    pub enable_subunits: Vec<String>,
    pub enable_equipments: Vec<String>,
    pub enable_equipment_modules: Vec<String>,
    pub enable_building: Option<(String, i32)>,
    pub sub_technologies: Vec<String>,
}

/// Collect bare identifiers from a block of `Value` entries.
/// HOI4 uses whitespace-separated lists for categories, enable_*,
/// sub_technologies, and xor — these are `Entry::Value(NodeedValue)`
/// not `Entry::Assignment`.
fn collect_bare_identifiers(entries: &[ast::Entry], source: &str) -> Vec<String> {
    let mut result = Vec::new();
    for entry in entries {
        match entry {
            // Bare identifier: `infantry_equipment_0`
            ast::Entry::Value(nv) => {
                if let ast::Value::String(span) = &nv.value {
                    result.push(span.resolve(source).to_string());
                }
            }
            // Assignments inside these blocks are malformed content and are
            // skipped. There is no `key = yes` form here to tolerate: bare
            // yes/no parse as Value::Boolean (never a String), and vanilla's
            // only key=value shapes are DLC-gate sub-blocks (`limit = {
            // has_dlc = ... }`), which are nested blocks, not scalars.
            // Pushing either side of a scalar assignment would record an
            // identifier that doesn't exist in the file as written.
            _ => {}
        }
    }
    result
}

/// Extract technologies from a parsed script AST.
/// Used by both initial scan AND incremental update.
pub(crate) fn find_technologies_in_entries(
    entries: &[ast::Entry],
    source: &str,
    file_path: &str,
    map: &mut HashMap<String, Technology>,
) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            if ass.key_text(source).eq_ignore_ascii_case("technologies") {
                if let ast::Value::Block(block_entries) = &ass.value.value {
                    for tech_entry in block_entries {
                        if let ast::Entry::Assignment(tech_ass) = tech_entry {
                            let tech_name = tech_ass.key_text(source).to_string();

                            // Skip macro defines like `@1918 = 0` — they have scalar
                            // values (Number), not Block. Only process tech blocks.
                            if !matches!(&tech_ass.value.value, ast::Value::Block(_)) {
                                continue;
                            }

                            let mut tech = Technology {
                                name: tech_name.clone(),
                                path: file_path.into(),
                                range: tech_ass.key_range.clone(),
                                start_year: None,
                                research_cost: None,
                                categories: Vec::new(),
                                folder: None,
                                leads_to_tech: Vec::new(),
                                xor: Vec::new(),
                                dependencies: Vec::new(),
                                enable_subunits: Vec::new(),
                                enable_equipments: Vec::new(),
                                enable_equipment_modules: Vec::new(),
                                enable_building: None,
                                sub_technologies: Vec::new(),
                            };

                            if let ast::Value::Block(tech_entries) = &tech_ass.value.value {
                                for field_entry in tech_entries {
                                    if let ast::Entry::Assignment(field_ass) = field_entry {
                                        let key = field_ass.key_text(source);
                                        match key {
                                            "start_year" => match &field_ass.value.value {
                                                ast::Value::Number(n) => {
                                                    tech.start_year = Some(*n as i32);
                                                }
                                                ast::Value::String(span) => {
                                                    if let Ok(n) =
                                                        span.resolve(source).parse::<f64>()
                                                    {
                                                        tech.start_year = Some(n as i32);
                                                    }
                                                }
                                                _ => {}
                                            },
                                            "research_cost" => match &field_ass.value.value {
                                                ast::Value::Number(n) => {
                                                    tech.research_cost = Some(*n);
                                                }
                                                ast::Value::String(span) => {
                                                    if let Ok(n) =
                                                        span.resolve(source).parse::<f64>()
                                                    {
                                                        tech.research_cost = Some(n);
                                                    }
                                                }
                                                _ => {}
                                            },
                                            "categories" => {
                                                if let ast::Value::Block(cat_entries) =
                                                    &field_ass.value.value
                                                {
                                                    tech.categories = collect_bare_identifiers(
                                                        cat_entries,
                                                        source,
                                                    );
                                                }
                                            }
                                            "folder" => {
                                                if let ast::Value::Block(folder_entries) =
                                                    &field_ass.value.value
                                                {
                                                    for folder_entry in folder_entries {
                                                        if let ast::Entry::Assignment(folder_ass) =
                                                            folder_entry
                                                        {
                                                            if folder_ass
                                                                .key_text(source)
                                                                .eq_ignore_ascii_case("name")
                                                            {
                                                                if let Some(s) = folder_ass
                                                                    .value
                                                                    .value
                                                                    .as_str(source)
                                                                {
                                                                    tech.folder =
                                                                        Some(s.to_string());
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            "path" => {
                                                // path = { leads_to_tech = tech_name
                                                // research_cost_coeff = 1 }
                                                if let ast::Value::Block(path_entries) =
                                                    &field_ass.value.value
                                                {
                                                    for path_entry in path_entries {
                                                        if let ast::Entry::Assignment(path_ass) =
                                                            path_entry
                                                        {
                                                            if path_ass
                                                                .key_text(source)
                                                                .eq_ignore_ascii_case(
                                                                    "leads_to_tech",
                                                                )
                                                            {
                                                                if let Some(s) = path_ass
                                                                    .value
                                                                    .value
                                                                    .as_str(source)
                                                                {
                                                                    tech.leads_to_tech
                                                                        .push(s.to_string());
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            "xor" => {
                                                if let ast::Value::Block(xor_entries) =
                                                    &field_ass.value.value
                                                {
                                                    tech.xor = collect_bare_identifiers(
                                                        xor_entries,
                                                        source,
                                                    );
                                                }
                                            }
                                            "dependencies" => {
                                                if let ast::Value::Block(dep_entries) =
                                                    &field_ass.value.value
                                                {
                                                    for dep_entry in dep_entries {
                                                        if let ast::Entry::Assignment(dep_ass) =
                                                            dep_entry
                                                        {
                                                            // dependencies = { tech_name = 1 }
                                                            // Only collect if value is 1
                                                            // (enabled dependency)
                                                            match &dep_ass.value.value {
                                                                ast::Value::Number(n)
                                                                    if *n == 1.0 =>
                                                                {
                                                                    tech.dependencies.push(
                                                                        dep_ass
                                                                            .key_text(source)
                                                                            .to_string(),
                                                                    );
                                                                }
                                                                _ => {}
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            "enable_subunits" => {
                                                if let ast::Value::Block(sub_entries) =
                                                    &field_ass.value.value
                                                {
                                                    tech.enable_subunits = collect_bare_identifiers(
                                                        sub_entries,
                                                        source,
                                                    );
                                                }
                                            }
                                            "enable_equipments" => {
                                                if let ast::Value::Block(equip_entries) =
                                                    &field_ass.value.value
                                                {
                                                    tech.enable_equipments =
                                                        collect_bare_identifiers(
                                                            equip_entries,
                                                            source,
                                                        );
                                                }
                                            }
                                            "enable_equipment_modules" => {
                                                if let ast::Value::Block(module_entries) =
                                                    &field_ass.value.value
                                                {
                                                    tech.enable_equipment_modules =
                                                        collect_bare_identifiers(
                                                            module_entries,
                                                            source,
                                                        );
                                                }
                                            }
                                            "enable_building" => {
                                                if let ast::Value::Block(building_entries) =
                                                    &field_ass.value.value
                                                {
                                                    let mut building_name = None;
                                                    let mut level = None;
                                                    for building_entry in building_entries {
                                                        if let ast::Entry::Assignment(
                                                            building_ass,
                                                        ) = building_entry
                                                        {
                                                            match building_ass.key_text(source) {
                                                                "building" => {
                                                                    if let Some(s) = building_ass
                                                                        .value
                                                                        .value
                                                                        .as_str(source)
                                                                    {
                                                                        building_name =
                                                                            Some(s.to_string());
                                                                    }
                                                                }
                                                                "level" => {
                                                                    match &building_ass.value.value
                                                                    {
                                                                        ast::Value::Number(n) => {
                                                                            level = Some(*n as i32);
                                                                        }
                                                                        ast::Value::String(
                                                                            span,
                                                                        ) => {
                                                                            if let Ok(n) = span
                                                                                .resolve(source)
                                                                                .parse::<f64>()
                                                                            {
                                                                                level =
                                                                                    Some(n as i32);
                                                                            }
                                                                        }
                                                                        _ => {}
                                                                    }
                                                                }
                                                                _ => {}
                                                            }
                                                        }
                                                    }
                                                    if let (Some(name), Some(lvl)) =
                                                        (building_name, level)
                                                    {
                                                        tech.enable_building = Some((name, lvl));
                                                    }
                                                }
                                            }
                                            "sub_technologies" => {
                                                if let ast::Value::Block(subtech_entries) =
                                                    &field_ass.value.value
                                                {
                                                    tech.sub_technologies =
                                                        collect_bare_identifiers(
                                                            subtech_entries,
                                                            source,
                                                        );
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }

                            map.insert(tech_name, tech);
                        }
                    }
                }
            }
        }
    }
}

/// Scan a list of winning technology files and extract all technologies.
/// Called by the orchestrator (overlay path) — the registry macro expects
/// this exact name (`technology_scanner` → `scan_technology_files`).
pub fn scan_technology_files<F>(files: &[PathBuf], filter: &F) -> HashMap<String, Technology>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();

    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        find_technologies_in_entries(
            &script.entries,
            &script.source,
            &path.to_string_lossy(),
            &mut map,
        );
    });

    map
}
