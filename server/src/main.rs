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
mod rules;

pub(crate) use crate::backend::Backend;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

use dashmap::DashMap;
use tower_lsp_server::{LspService, Server};

use crate::config::Config;
use crate::data::scanner_data::ScannerData;

pub(crate) static TRIGGERS: Lazy<HashMap<String, data::hoi4_data::HOI4Entity>> =
    Lazy::new(data::hoi4_data::get_triggers);
pub(crate) static EFFECTS: Lazy<HashMap<String, data::hoi4_data::HOI4Entity>> =
    Lazy::new(data::hoi4_data::get_effects);
pub(crate) static MODIFIERS: Lazy<HashMap<String, data::hoi4_data::HOI4Entity>> =
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
pub mod tests;
