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
mod cli_analyze;
mod config;
mod log_level;
mod rules;

pub(crate) use crate::backend::Backend;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::PathBuf;

pub(crate) use crate::data::scanner_data::ScannerData;

use tower_lsp_server::{LspService, Server};

pub(crate) static TRIGGERS: Lazy<&'static HashMap<String, data::hoi4_data::HOI4Entity>> =
    Lazy::new(data::hoi4_data::get_triggers);
pub(crate) static EFFECTS: Lazy<&'static HashMap<String, data::hoi4_data::HOI4Entity>> =
    Lazy::new(data::hoi4_data::get_effects);
pub(crate) static MODIFIERS: Lazy<&'static HashMap<String, data::hoi4_data::HOI4Entity>> =
    Lazy::new(data::hoi4_data::get_modifiers);
pub(crate) static DEFINITIONS: Lazy<&'static HashMap<String, data::hoi4_data::BlockSchema>> =
    Lazy::new(data::hoi4_data::get_definitions);
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
    // Static analysis: `hom-lsp analyze <file>` (the bare `<file>` form is the
    // same thing). No transport, no client, one process.
    let args: Vec<String> = std::env::args().collect();
    let mut cli_path: Option<PathBuf> = None;
    let mut cli_game: Option<PathBuf> = None;
    let mut cli_discover_game = true;
    let mut cli_workspaces: Vec<PathBuf> = Vec::new();
    let mut cli_scope_validation = true;
    let mut cli_styling = false;
    let mut cli_tokens = false;
    let mut cli_json = false;
    let mut args = args[1..].iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "analyze" => {}
            "--game" => cli_game = args.next().map(PathBuf::from),
            "--no-game" => cli_discover_game = false,
            "--workspace" => {
                if let Some(w) = args.next() {
                    cli_workspaces.push(PathBuf::from(w));
                }
            }
            "--scope-validation" => cli_scope_validation = true,
            "--no-scope-validation" => cli_scope_validation = false,
            "--styling" => cli_styling = true,
            "--tokens" => cli_tokens = true,
            "--json" => cli_json = true,
            other if other.starts_with("--") => {} // --stdio and friends: server mode
            other => cli_path = Some(PathBuf::from(other)),
        }
    }
    if let Some(path) = cli_path {
        let code = cli_analyze::run(cli_analyze::Options {
            path,
            game: cli_game,
            discover_game: cli_discover_game,
            workspaces: cli_workspaces,
            scope_validation: cli_scope_validation,
            styling: cli_styling,
            tokens: cli_tokens,
            json: cli_json,
            timeout: std::time::Duration::from_secs(600),
        })
        .await;
        std::process::exit(code);
    }

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(backend::Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod tests;

/// Test-only shared builder for `ValidationContext`. Declared at crate root
/// (not inside `tests`) so rule modules' own `#[cfg(test)]` blocks can use
/// the same builder as integration-style test files.
#[cfg(test)]
pub(crate) mod test_support;
