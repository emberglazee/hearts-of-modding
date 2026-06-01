#![allow(clippy::collapsible_if)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
mod ability_scanner;
mod achievement_scanner;
mod adjacency_scanner;
mod advanced_validation;
mod ai_area_scanner;
mod ai_strategy_plan_scanner;
mod ast;
mod backend;
mod building_scanner;
mod call_hierarchy;
mod character_scanner;
mod code_action_handler;
mod color_utils;
mod completion_handler;
mod config;
mod continent_scanner;
mod country_scanner;
mod csv_parser;
mod defines_parser;
mod document_symbols;
mod enhanced_color;
mod entity_lookup;
mod event_scanner;
mod focus_scanner;
mod formatting;
mod fs_util;
mod gfx_scanner;
mod hoi4_data;
mod hover_handler;
mod idea_scanner;
mod ideology_scanner;
mod incremental_scanner;
mod interner;
mod loc_parser;
mod loc_preview;
mod logistics_scanner;
mod lsp_convert;
mod lsp_handler;
mod map_config;
mod map_object_scanner;
mod modifier_display;
mod modifier_format;
mod modifier_scanner;
mod music_scanner;
mod parser;
mod portrait_scanner;
mod province_scanner;
mod rename;
mod rules;
mod scan_orchestrator;
mod scanner_data;
mod scope;
mod scope_context;
mod scripted_loc_scanner;
mod scripted_scanner;
mod semantic_tokens;
mod sound_scanner;
mod sprite_scanner;
mod state_scanner;
mod strategic_region_scanner;
mod symbol_search;
#[cfg(test)]
mod test_loc_version;
mod trait_scanner;
mod variable_scanner;
mod workspace_symbols;
pub(crate) use crate::backend::Backend;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

use dashmap::DashMap;
use tower_lsp_server::{LspService, Server};

use crate::config::Config;
use crate::scanner_data::ScannerData;

pub(crate) static TRIGGERS: Lazy<HashMap<String, hoi4_data::HOI4Entity>> =
    Lazy::new(hoi4_data::get_triggers);
pub(crate) static EFFECTS: Lazy<HashMap<String, hoi4_data::HOI4Entity>> =
    Lazy::new(hoi4_data::get_effects);
pub(crate) static MODIFIERS: Lazy<HashMap<String, hoi4_data::HOI4Entity>> =
    Lazy::new(hoi4_data::get_modifiers);
pub(crate) static SCOPES: Lazy<Vec<&'static str>> = Lazy::new(hoi4_data::get_scopes);
pub(crate) static LOC_COMMANDS: Lazy<Vec<&'static str>> = Lazy::new(hoi4_data::get_loc_commands);

/// Convert a byte offset in a UTF-8 string to a UTF-16 code unit offset
/// This is required because LSP uses UTF-16 positions, but Rust strings are UTF-8
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

/// Convert a UTF-16 code unit offset to a UTF-8 byte offset
/// This is required because LSP uses UTF-16 positions, but Rust strings are UTF-8
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
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        documents: DashMap::new(),
        document_asts: DashMap::new(),
        scanner_data: ScannerData::new(),
        config: Config::new(),
        system_info: Mutex::new(sysinfo::System::new()),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
pub mod test_loc_columns;
#[cfg(test)]
pub mod test_loc_dups;
#[cfg(test)]
pub mod test_loc_empty;
#[cfg(test)]
pub mod test_parser_skip;
#[cfg(test)]
pub mod test_utf16_conversion;
