#![allow(clippy::collapsible_if)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::module_inception)]

use tikv_jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
mod data;
mod lsp;
mod parser;
mod scanner;
mod scope;
mod utils;
mod validation;

mod backend;
mod config;
mod log_level;
mod rules;

pub(crate) use crate::backend::Backend;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use arc_swap::ArcSwap;
use dashmap::DashMap;
use tower_lsp_server::{LspService, Server};

use crate::config::Config;
use crate::data::scanner_data::ScannerData;

pub(crate) static TRIGGERS: Lazy<&'static HashMap<String, data::hoi4_data::HOI4Entity>> =
    Lazy::new(data::hoi4_data::get_triggers);
pub(crate) static EFFECTS: Lazy<&'static HashMap<String, data::hoi4_data::HOI4Entity>> =
    Lazy::new(data::hoi4_data::get_effects);
pub(crate) static MODIFIERS: Lazy<&'static HashMap<String, data::hoi4_data::HOI4Entity>> =
    Lazy::new(data::hoi4_data::get_modifiers);
pub(crate) static SCOPES: Lazy<Vec<&'static str>> = Lazy::new(data::hoi4_data::get_scopes);
pub(crate) static LOC_COMMANDS: Lazy<Vec<&'static str>> =
    Lazy::new(data::hoi4_data::get_loc_commands);

/// Convert a byte offset in a UTF-8 string to a UTF-16 code unit offset.
///
/// This is required because LSP uses UTF-16 positions, but Rust strings are UTF-8.
///
/// **Performance note:** This is O(n) per call. If you need to convert many offsets
/// within the same string, use [`crate::utils::line_index::LineIndex`] instead,
/// which precomputes the mapping for O(1) lookups.
#[allow(dead_code)]
pub(crate) fn byte_offset_to_utf16(s: &str, byte_offset: usize) -> u32 {
    s[..byte_offset]
        .chars()
        .map(|c| c.len_utf16())
        .sum::<usize>() as u32
}

/// Get the UTF-16 length of a string
pub(crate) fn utf16_len(s: &str) -> u32 {
    s.chars().map(|c| c.len_utf16()).sum::<usize>() as u32
}

/// Convert a UTF-16 code unit offset to a UTF-8 byte offset.
///
/// This is required because LSP uses UTF-16 positions, but Rust strings are UTF-8.
///
/// **Performance note:** This is O(n) per call. If you need to convert many offsets
/// within the same string, use [`crate::utils::line_index::LineIndex`] instead,
/// which precomputes the mapping for O(1) lookups.
pub(crate) fn utf16_to_byte_offset(s: &str, utf16_offset: usize) -> usize {
    let mut byte_offset = 0;
    let mut utf16_so_far = 0;
    for c in s.chars() {
        let cu = c.len_utf16();
        if utf16_so_far + cu > utf16_offset {
            break;
        }
        utf16_so_far += cu;
        byte_offset += c.len_utf8();
    }
    byte_offset
}
#[tokio::main]
async fn main() {
    // CLI validation mode: cargo run --release -- <file_path>
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && !args[1].starts_with("--") {
        cli_validate(&args[1]).await;
        return;
    }

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| {
        let static_keywords = Arc::new(backend::build_static_semantic_keywords());
        Backend {
            client,
            documents: DashMap::new(),
            document_asts: DashMap::new(),
            document_cancellation_tokens: DashMap::new(),
            scanner_data: ScannerData::new(),
            config: Config::new(),
            system_info: Mutex::new(sysinfo::System::new()),
            workspace_roots: Mutex::new(Vec::new()),
            static_token_keywords: static_keywords,
            entity_token_context: ArcSwap::from_pointee(HashMap::new()),
        }
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod tests;

/// CLI validation mode: parse and validate a file, print diagnostics.
async fn cli_validate(path: &str) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading {}: {}", path, e);
            return;
        }
    };

    let (script, parse_errors) = crate::parser::parser::parse_script(&content);
    let source = &script.source;

    // Determine initial scope from file path
    let initial_scope = if path.contains("/common/abilities/") {
        crate::scope::scope::Scope::Character
    } else if path.contains("/common/decisions/") {
        crate::scope::scope::Scope::Country
    } else if path.contains("/common/aces/") {
        crate::scope::scope::Scope::Ace
    } else if path.contains("/common/ai_faction_theaters/")
        || path.contains("/common/ai_focuses/")
    {
        crate::scope::scope::Scope::Country
    } else {
        crate::scope::scope::Scope::Global
    };

    // Build typed empty DashMaps for ValidationContext
    use crate::data::interner::InternedStr;
    use crate::data::layered_value::LayeredValue;
    use dashmap::DashMap;

    let loc: DashMap<InternedStr, LayeredValue<crate::parser::loc_parser::LocEntry>> =
        DashMap::new();
    let st: DashMap<InternedStr, LayeredValue<crate::scanner::scripted_scanner::ScriptedEntity>> =
        DashMap::new();
    let se: DashMap<InternedStr, LayeredValue<crate::scanner::scripted_scanner::ScriptedEntity>> =
        DashMap::new();
    let ideologies: DashMap<InternedStr, LayeredValue<crate::scanner::ideology_scanner::Ideology>> =
        DashMap::new();
    let sub_ideologies: DashMap<
        InternedStr,
        LayeredValue<(InternedStr, crate::parser::ast::Range, InternedStr)>,
    > = DashMap::new();
    let traits: DashMap<InternedStr, LayeredValue<crate::scanner::trait_scanner::Trait>> =
        DashMap::new();
    let sprites: DashMap<InternedStr, LayeredValue<crate::scanner::sprite_scanner::Sprite>> =
        DashMap::new();
    let ideas: DashMap<InternedStr, LayeredValue<crate::scanner::idea_scanner::Idea>> =
        DashMap::new();
    let characters: DashMap<
        InternedStr,
        LayeredValue<crate::scanner::character_scanner::Character>,
    > = DashMap::new();
    let provinces: DashMap<u32, crate::scanner::province_scanner::Province> = DashMap::new();
    let modifier_mappings: DashMap<InternedStr, String> = DashMap::new();
    let sound_effects: DashMap<
        InternedStr,
        LayeredValue<crate::scanner::sound_scanner::SoundEffect>,
    > = DashMap::new();
    let country_tags: DashMap<
        InternedStr,
        LayeredValue<crate::scanner::country_scanner::CountryTag>,
    > = DashMap::new();
    let buildings: DashMap<InternedStr, LayeredValue<crate::scanner::building_scanner::Building>> =
        DashMap::new();
    let resources: DashMap<InternedStr, LayeredValue<crate::scanner::resource_scanner::Resource>> =
        DashMap::new();
    let state_categories: DashMap<
        InternedStr,
        LayeredValue<crate::scanner::state_category_scanner::StateCategory>,
    > = DashMap::new();
    let continents: DashMap<
        InternedStr,
        LayeredValue<crate::scanner::continent_scanner::Continent>,
    > = DashMap::new();
    let strategic_regions: DashMap<u32, crate::scanner::strategic_region_scanner::StrategicRegion> =
        DashMap::new();
    let terrain_categories: DashMap<
        InternedStr,
        LayeredValue<crate::scanner::terrain_scanner::TerrainCategory>,
    > = DashMap::new();
    let abilities: DashMap<InternedStr, LayeredValue<crate::scanner::ability_scanner::Ability>> =
        DashMap::new();
    let ace_modifiers: DashMap<
        InternedStr,
        LayeredValue<crate::scanner::ace_scanner::AceModifier>,
    > = DashMap::new();
    let event_targets: DashMap<InternedStr, Vec<crate::scanner::variable_scanner::EventTarget>> =
        DashMap::new();
    let event_namespaces: DashMap<
        InternedStr,
        LayeredValue<crate::scanner::event_namespace_scanner::EventNamespace>,
    > = DashMap::new();
    let events: DashMap<InternedStr, LayeredValue<crate::scanner::event_scanner::Event>> =
        DashMap::new();
    let decisions: DashMap<InternedStr, LayeredValue<crate::scanner::decision_scanner::Decision>> =
        DashMap::new();
    let decision_categories: DashMap<InternedStr, LayeredValue<()>> = DashMap::new();
    let unit_types: DashMap<InternedStr, LayeredValue<crate::scanner::unit_scanner::UnitType>> =
        DashMap::new();

    let ctx = crate::rules::ValidationContext {
        uri: path,
        source,
        loc: &loc,
        scripted_triggers: &st,
        scripted_effects: &se,
        ideologies: &ideologies,
        sub_ideologies: &sub_ideologies,
        traits: &traits,
        sprites: &sprites,
        ideas: &ideas,
        characters: &characters,
        provinces: &provinces,
        modifier_mappings: &modifier_mappings,
        ignored_loc_regex: &[],
        comments: &[],
        sound_effects: &sound_effects,
        country_tags: &country_tags,
        buildings: &buildings,
        resources: &resources,
        state_categories: &state_categories,
        continents: &continents,
        strategic_regions: &strategic_regions,
        terrain_categories: &terrain_categories,
        abilities: &abilities,
        ace_modifiers: &ace_modifiers,
        game_path: None,
        styling_enabled: false,
        workspace_roots: &[],
        unit_types: &unit_types,
        event_targets: &event_targets,
        event_namespaces: &event_namespaces,
        events: &events,
        decisions: &decisions,
        decision_categories: &decision_categories,
    };

    let mut visitors: Vec<Box<dyn crate::rules::visitor::AstVisitor>> = Vec::new();
    let rules: Vec<Box<dyn crate::rules::ValidationRule>> =
        vec![Box::new(crate::rules::v2_scope::V2ScopeRule)];

    let mut diags = Vec::new();
    crate::rules::visitor::walk_script(
        &script.entries,
        &mut visitors,
        &rules,
        &ctx,
        &mut diags,
        initial_scope,
        false,
    );

    // Print diagnostics in a compact format
    if diags.is_empty() {
        println!("No diagnostics for {}", path);
    } else {
        for d in &diags {
            let range = &d.range;
            println!(
                "{}:{}:{}: {} [{}]",
                path,
                range.start.line + 1,
                range.start.character + 1,
                d.message,
                d.code
                    .as_ref()
                    .map(|c| match c {
                        tower_lsp_server::ls_types::NumberOrString::String(s) => s.as_str(),
                        tower_lsp_server::ls_types::NumberOrString::Number(_) => "?",
                    })
                    .unwrap_or("?"),
            );
        }
    }

    // Also show parse errors
    for (msg, range) in &parse_errors {
        println!("PARSE {}:{}: {}", path, range.start_line + 1, msg,);
    }
}
