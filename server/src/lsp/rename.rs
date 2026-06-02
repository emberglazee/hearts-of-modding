use crate::data::interner::InternedStr;
use crate::parser::ast::{Entry, Range, Value};
use dashmap::DashSet;
use std::collections::HashMap;
use std::sync::Arc;
use tower_lsp_server::ls_types::{
    Position as LspPosition, PrepareRenameResponse, Range as LspRange, TextEdit, Uri, WorkspaceEdit,
};

/// Symbol type that can be renamed
#[derive(Debug, Clone, PartialEq)]
pub enum RenameableSymbol {
    Event(String),
    ScriptedTrigger(String),
    ScriptedEffect(String),
    Idea(String),
    Character(String),
    Variable(String),
    Ability(String),
    ColorCode(String),
}

/// Prepare rename - check if the symbol at the position can be renamed
pub async fn prepare_rename(
    uri: &str,
    position: LspPosition,
    data: &crate::ScannerData,
) -> Option<PrepareRenameResponse> {
    let path = uri.trim_start_matches("file://");
    let lookup = crate::data::entity_lookup::EntityLookup::new(data);
    if let Some((_, range, _)) = lookup.entity_at(path, position) {
        return Some(PrepareRenameResponse::Range(range_to_lsp(&range)));
    }
    None
}

/// Perform rename - find all references and create workspace edit
pub async fn rename_symbol(
    uri: &str,
    position: LspPosition,
    new_name: &str,
    data: &crate::ScannerData,
    documents: &dashmap::DashMap<String, String>,
    document_asts: &dashmap::DashMap<
        String,
        (
            Arc<crate::parser::ast::Script>,
            Vec<(String, crate::parser::ast::Range)>,
        ),
    >,
    workspace_files: &DashSet<InternedStr>,
) -> Option<WorkspaceEdit> {
    let path = uri.trim_start_matches("file://");

    // Find what symbol we're renaming
    let symbol = find_symbol_at_position(path, &position, data).await?;

    // Find all references to this symbol
    let mut changes: HashMap<Uri, Vec<TextEdit>> = HashMap::new();

    match symbol {
        RenameableSymbol::Event(old_name) => {
            find_event_references(
                &old_name,
                new_name,
                document_asts,
                workspace_files,
                &mut changes,
            );
        }
        RenameableSymbol::ScriptedTrigger(old_name) => {
            find_scripted_trigger_references(
                &old_name,
                new_name,
                document_asts,
                workspace_files,
                &mut changes,
            );
        }
        RenameableSymbol::ScriptedEffect(old_name) => {
            find_scripted_effect_references(
                &old_name,
                new_name,
                document_asts,
                workspace_files,
                &mut changes,
            );
        }
        RenameableSymbol::Idea(old_name) => {
            find_idea_references(
                &old_name,
                new_name,
                document_asts,
                workspace_files,
                &mut changes,
            );
        }
        RenameableSymbol::Character(old_name) => {
            find_character_references(
                &old_name,
                new_name,
                document_asts,
                workspace_files,
                &mut changes,
            );
        }
        RenameableSymbol::Ability(old_name) => {
            find_ability_references(
                &old_name,
                new_name,
                document_asts,
                workspace_files,
                &mut changes,
            );
        }
        RenameableSymbol::Variable(old_name) => {
            find_variable_references(
                &old_name,
                new_name,
                document_asts,
                workspace_files,
                &mut changes,
            );
        }
        RenameableSymbol::ColorCode(old_name) => {
            find_color_code_references(
                &old_name,
                new_name,
                documents,
                workspace_files,
                &mut changes,
            );
        }
    }

    if changes.is_empty() {
        None
    } else {
        Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        })
    }
}

/// Find what symbol is at the given position
async fn find_symbol_at_position(
    path: &str,
    position: &LspPosition,
    data: &crate::ScannerData,
) -> Option<RenameableSymbol> {
    let lookup = crate::data::entity_lookup::EntityLookup::new(data);
    if let Some((kind, _, name)) = lookup.entity_at(path, *position) {
        return Some(match kind {
            crate::data::entity_lookup::EntityKind::Event => RenameableSymbol::Event(name),
            crate::data::entity_lookup::EntityKind::ScriptedTrigger => {
                RenameableSymbol::ScriptedTrigger(name)
            }
            crate::data::entity_lookup::EntityKind::ScriptedEffect => {
                RenameableSymbol::ScriptedEffect(name)
            }
            crate::data::entity_lookup::EntityKind::Idea => RenameableSymbol::Idea(name),
            crate::data::entity_lookup::EntityKind::Character => RenameableSymbol::Character(name),
            crate::data::entity_lookup::EntityKind::Variable => RenameableSymbol::Variable(name),
            crate::data::entity_lookup::EntityKind::Ability => RenameableSymbol::Ability(name),
            crate::data::entity_lookup::EntityKind::ColorCode => RenameableSymbol::ColorCode(name),
            _ => return None,
        });
    }
    None
}

/// Find all references to an event and create text edits
fn find_event_references(
    old_name: &str,
    new_name: &str,
    document_asts: &dashmap::DashMap<
        String,
        (
            Arc<crate::parser::ast::Script>,
            Vec<(String, crate::parser::ast::Range)>,
        ),
    >,
    workspace_files: &DashSet<InternedStr>,
    changes: &mut HashMap<Uri, Vec<TextEdit>>,
) {
    for entry in document_asts.iter() {
        let uri_str = entry.key();
        let (script, _) = entry.value();

        let mut edits = Vec::new();
        find_event_references_in_entries(&script.entries, old_name, new_name, &mut edits);

        if !edits.is_empty() {
            if let Ok(url) = uri_str.parse::<Uri>() {
                changes.insert(url, edits);
            }
        }
    }

    // Process unopened workspace files
    for entry in workspace_files.iter() {
        let file_path: &str = &entry;
        let Some(url) = Uri::from_file_path(std::path::Path::new(file_path)) else {
            continue;
        };
        let uri_str = url.as_str().to_string();
        if document_asts.contains_key(&uri_str) {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(file_path) {
            let (script, _) = crate::parser::parser::parse_script(&content);
            let mut edits = Vec::new();
            find_event_references_in_entries(&script.entries, old_name, new_name, &mut edits);
            if !edits.is_empty() {
                changes.insert(url, edits);
            }
        }
    }
}

/// Find event references in AST entries
fn find_event_references_in_entries(
    entries: &[Entry],
    old_name: &str,
    new_name: &str,
    edits: &mut Vec<TextEdit>,
) {
    for entry in entries {
        if let Entry::Assignment(ass) = entry {
            // Check for event triggers: country_event = { id = old_name }
            if ass.key == "country_event"
                || ass.key == "state_event"
                || ass.key == "news_event"
                || ass.key == "unit_leader_event"
            {
                if let Value::Block(children) = &ass.value.value {
                    for child in children {
                        if let Entry::Assignment(child_ass) = child {
                            if child_ass.key == "id" {
                                if let Value::String(id) = &child_ass.value.value {
                                    if id == old_name {
                                        edits.push(TextEdit {
                                            range: range_to_lsp(&child_ass.value.range),
                                            new_text: format!("\"{}\"", new_name),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Recurse into blocks
            if let Value::Block(children) = &ass.value.value {
                find_event_references_in_entries(children, old_name, new_name, edits);
            }
        }
    }
}

/// Find all references to a scripted trigger
fn find_scripted_trigger_references(
    old_name: &str,
    new_name: &str,
    document_asts: &dashmap::DashMap<
        String,
        (
            Arc<crate::parser::ast::Script>,
            Vec<(String, crate::parser::ast::Range)>,
        ),
    >,
    workspace_files: &DashSet<InternedStr>,
    changes: &mut HashMap<Uri, Vec<TextEdit>>,
) {
    for entry in document_asts.iter() {
        let uri_str = entry.key();
        let (script, _) = entry.value();

        let mut edits = Vec::new();
        find_scripted_references_in_entries(&script.entries, old_name, new_name, &mut edits);

        if !edits.is_empty() {
            if let Ok(url) = uri_str.parse::<Uri>() {
                changes.insert(url, edits);
            }
        }
    }

    // Process unopened workspace files
    for entry in workspace_files.iter() {
        let file_path: &str = &entry;
        let Some(url) = Uri::from_file_path(std::path::Path::new(file_path)) else {
            continue;
        };
        let uri_str = url.as_str().to_string();
        if document_asts.contains_key(&uri_str) {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(file_path) {
            let (script, _) = crate::parser::parser::parse_script(&content);
            let mut edits = Vec::new();
            find_scripted_references_in_entries(&script.entries, old_name, new_name, &mut edits);
            if !edits.is_empty() {
                changes.insert(url, edits);
            }
        }
    }
}

/// Find all references to a scripted effect
fn find_scripted_effect_references(
    old_name: &str,
    new_name: &str,
    document_asts: &dashmap::DashMap<
        String,
        (
            Arc<crate::parser::ast::Script>,
            Vec<(String, crate::parser::ast::Range)>,
        ),
    >,
    workspace_files: &DashSet<InternedStr>,
    changes: &mut HashMap<Uri, Vec<TextEdit>>,
) {
    for entry in document_asts.iter() {
        let uri_str = entry.key();
        let (script, _) = entry.value();

        let mut edits = Vec::new();
        find_scripted_references_in_entries(&script.entries, old_name, new_name, &mut edits);

        if !edits.is_empty() {
            if let Ok(url) = uri_str.parse::<Uri>() {
                changes.insert(url, edits);
            }
        }
    }

    // Process unopened workspace files
    for entry in workspace_files.iter() {
        let file_path: &str = &entry;
        let Some(url) = Uri::from_file_path(std::path::Path::new(file_path)) else {
            continue;
        };
        let uri_str = url.as_str().to_string();
        if document_asts.contains_key(&uri_str) {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(file_path) {
            let (script, _) = crate::parser::parser::parse_script(&content);
            let mut edits = Vec::new();
            find_scripted_references_in_entries(&script.entries, old_name, new_name, &mut edits);
            if !edits.is_empty() {
                changes.insert(url, edits);
            }
        }
    }
}

/// Find scripted trigger/effect references in AST entries
fn find_scripted_references_in_entries(
    entries: &[Entry],
    old_name: &str,
    new_name: &str,
    edits: &mut Vec<TextEdit>,
) {
    for entry in entries {
        if let Entry::Assignment(ass) = entry {
            // Check for definition
            if ass.key == old_name {
                edits.push(TextEdit {
                    range: range_to_lsp(&ass.key_range),
                    new_text: new_name.to_string(),
                });
            }

            // Check for usage: old_name = yes
            if ass.key == old_name {
                edits.push(TextEdit {
                    range: range_to_lsp(&ass.key_range),
                    new_text: new_name.to_string(),
                });
            }

            // Recurse into blocks
            if let Value::Block(children) = &ass.value.value {
                find_scripted_references_in_entries(children, old_name, new_name, edits);
            }
        }
    }
}

/// Find all references to an idea
fn find_idea_references(
    old_name: &str,
    new_name: &str,
    document_asts: &dashmap::DashMap<
        String,
        (
            Arc<crate::parser::ast::Script>,
            Vec<(String, crate::parser::ast::Range)>,
        ),
    >,
    workspace_files: &DashSet<InternedStr>,
    changes: &mut HashMap<Uri, Vec<TextEdit>>,
) {
    for entry in document_asts.iter() {
        let uri_str = entry.key();
        let (script, _) = entry.value();

        let mut edits = Vec::new();
        find_idea_references_in_entries(&script.entries, old_name, new_name, &mut edits);

        if !edits.is_empty() {
            if let Ok(url) = uri_str.parse::<Uri>() {
                changes.insert(url, edits);
            }
        }
    }

    // Process unopened workspace files
    for entry in workspace_files.iter() {
        let file_path: &str = &entry;
        let Some(url) = Uri::from_file_path(std::path::Path::new(file_path)) else {
            continue;
        };
        let uri_str = url.as_str().to_string();
        if document_asts.contains_key(&uri_str) {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(file_path) {
            let (script, _) = crate::parser::parser::parse_script(&content);
            let mut edits = Vec::new();
            find_idea_references_in_entries(&script.entries, old_name, new_name, &mut edits);
            if !edits.is_empty() {
                changes.insert(url, edits);
            }
        }
    }
}

/// Find idea references in AST entries
fn find_idea_references_in_entries(
    entries: &[Entry],
    old_name: &str,
    new_name: &str,
    edits: &mut Vec<TextEdit>,
) {
    for entry in entries {
        if let Entry::Assignment(ass) = entry {
            // Check for idea definition or usage
            if ass.key == old_name {
                edits.push(TextEdit {
                    range: range_to_lsp(&ass.key_range),
                    new_text: new_name.to_string(),
                });
            }

            // Check for add_ideas/remove_ideas
            if ass.key == "add_ideas"
                || ass.key == "remove_ideas"
                || ass.key == "add_timed_idea"
                || ass.key == "swap_ideas"
            {
                if let Value::String(idea_name) = &ass.value.value {
                    if idea_name == old_name {
                        edits.push(TextEdit {
                            range: range_to_lsp(&ass.value.range),
                            new_text: format!("\"{}\"", new_name),
                        });
                    }
                }
            }

            // Recurse into blocks
            if let Value::Block(children) = &ass.value.value {
                find_idea_references_in_entries(children, old_name, new_name, edits);
            }
        }
    }
}

/// Find all references to a character
fn find_character_references(
    old_name: &str,
    new_name: &str,
    document_asts: &dashmap::DashMap<
        String,
        (
            Arc<crate::parser::ast::Script>,
            Vec<(String, crate::parser::ast::Range)>,
        ),
    >,
    workspace_files: &DashSet<InternedStr>,
    changes: &mut HashMap<Uri, Vec<TextEdit>>,
) {
    for entry in document_asts.iter() {
        let uri_str = entry.key();
        let (script, _) = entry.value();

        let mut edits = Vec::new();
        find_character_references_in_entries(&script.entries, old_name, new_name, &mut edits);

        if !edits.is_empty() {
            if let Ok(url) = uri_str.parse::<Uri>() {
                changes.insert(url, edits);
            }
        }
    }

    // Process unopened workspace files
    for entry in workspace_files.iter() {
        let file_path: &str = &entry;
        let Some(url) = Uri::from_file_path(std::path::Path::new(file_path)) else {
            continue;
        };
        let uri_str = url.as_str().to_string();
        if document_asts.contains_key(&uri_str) {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(file_path) {
            let (script, _) = crate::parser::parser::parse_script(&content);
            let mut edits = Vec::new();
            find_character_references_in_entries(&script.entries, old_name, new_name, &mut edits);
            if !edits.is_empty() {
                changes.insert(url, edits);
            }
        }
    }
}

/// Find character references in AST entries
fn find_character_references_in_entries(
    entries: &[Entry],
    old_name: &str,
    new_name: &str,
    edits: &mut Vec<TextEdit>,
) {
    for entry in entries {
        if let Entry::Assignment(ass) = entry {
            // Character definition
            if ass.key == old_name {
                edits.push(TextEdit {
                    range: range_to_lsp(&ass.key_range),
                    new_text: new_name.to_string(),
                });
            }

            // Character usage (recruit_character, etc)
            if ass.key == "recruit_character"
                || ass.key == "has_character"
                || ass.key == "promote_character"
                || ass.key == "retire_character"
            {
                if let Value::String(char_name) = &ass.value.value {
                    if char_name == old_name {
                        edits.push(TextEdit {
                            range: range_to_lsp(&ass.value.range),
                            new_text: new_name.to_string(),
                        });
                    }
                }
            }

            // character = X block usage
            if ass.key == "character" {
                if let Value::String(char_name) = &ass.value.value {
                    if char_name == old_name {
                        edits.push(TextEdit {
                            range: range_to_lsp(&ass.value.range),
                            new_text: new_name.to_string(),
                        });
                    }
                }
            }

            // Recurse into blocks
            if let Value::Block(children) = &ass.value.value {
                find_character_references_in_entries(children, old_name, new_name, edits);
            }
        }
    }
}

/// Find all references to a variable
fn find_variable_references(
    old_name: &str,
    new_name: &str,
    document_asts: &dashmap::DashMap<
        String,
        (
            Arc<crate::parser::ast::Script>,
            Vec<(String, crate::parser::ast::Range)>,
        ),
    >,
    workspace_files: &DashSet<InternedStr>,
    changes: &mut HashMap<Uri, Vec<TextEdit>>,
) {
    for entry in document_asts.iter() {
        let uri_str = entry.key();
        let (script, _) = entry.value();

        let mut edits = Vec::new();
        find_variable_references_in_entries(&script.entries, old_name, new_name, &mut edits);

        if !edits.is_empty() {
            if let Ok(url) = uri_str.parse::<Uri>() {
                changes.insert(url, edits);
            }
        }
    }

    // Process unopened workspace files
    for entry in workspace_files.iter() {
        let file_path: &str = &entry;
        let Some(url) = Uri::from_file_path(std::path::Path::new(file_path)) else {
            continue;
        };
        let uri_str = url.as_str().to_string();
        if document_asts.contains_key(&uri_str) {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(file_path) {
            let (script, _) = crate::parser::parser::parse_script(&content);
            let mut edits = Vec::new();
            find_variable_references_in_entries(&script.entries, old_name, new_name, &mut edits);
            if !edits.is_empty() {
                changes.insert(url, edits);
            }
        }
    }
}

/// Find variable references in AST entries
fn find_variable_references_in_entries(
    entries: &[Entry],
    old_name: &str,
    new_name: &str,
    edits: &mut Vec<TextEdit>,
) {
    for entry in entries {
        if let Entry::Assignment(ass) = entry {
            // Check for set_variable, check_variable, etc.
            if ass.key == "set_variable"
                || ass.key == "check_variable"
                || ass.key == "add_to_variable"
                || ass.key == "subtract_from_variable"
                || ass.key == "multiply_variable"
                || ass.key == "divide_variable"
                || ass.key == "modulo_variable"
                || ass.key == "clamp_variable"
            {
                if let Value::Block(children) = &ass.value.value {
                    for child in children {
                        if let Entry::Assignment(child_ass) = child {
                            if child_ass.key == "var" || child_ass.key == "variable" {
                                if let Value::String(var_name) = &child_ass.value.value {
                                    if var_name == old_name {
                                        edits.push(TextEdit {
                                            range: range_to_lsp(&child_ass.value.range),
                                            new_text: format!("\"{}\"", new_name),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Recurse into blocks
            if let Value::Block(children) = &ass.value.value {
                find_variable_references_in_entries(children, old_name, new_name, edits);
            }
        }
    }
}

/// Find all references to an ability
fn find_ability_references(
    old_name: &str,
    new_name: &str,
    document_asts: &dashmap::DashMap<
        String,
        (
            Arc<crate::parser::ast::Script>,
            Vec<(String, crate::parser::ast::Range)>,
        ),
    >,
    workspace_files: &DashSet<InternedStr>,
    changes: &mut HashMap<Uri, Vec<TextEdit>>,
) {
    for entry in document_asts.iter() {
        let uri_str = entry.key();
        let (script, _) = entry.value();

        let mut edits = Vec::new();
        find_ability_references_in_entries(&script.entries, old_name, new_name, &mut edits);

        if !edits.is_empty() {
            if let Ok(url) = uri_str.parse::<Uri>() {
                changes.insert(url, edits);
            }
        }
    }

    // Process unopened workspace files
    for entry in workspace_files.iter() {
        let file_path: &str = &entry;
        let Some(url) = Uri::from_file_path(std::path::Path::new(file_path)) else {
            continue;
        };
        let uri_str = url.as_str().to_string();
        if document_asts.contains_key(&uri_str) {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(file_path) {
            let (script, _) = crate::parser::parser::parse_script(&content);
            let mut edits = Vec::new();
            find_ability_references_in_entries(&script.entries, old_name, new_name, &mut edits);
            if !edits.is_empty() {
                changes.insert(url, edits);
            }
        }
    }
}

/// Find ability references in AST entries
fn find_ability_references_in_entries(
    entries: &[Entry],
    old_name: &str,
    new_name: &str,
    edits: &mut Vec<TextEdit>,
) {
    for entry in entries {
        if let Entry::Assignment(ass) = entry {
            if ass.key == old_name {
                edits.push(TextEdit {
                    range: range_to_lsp(&ass.key_range),
                    new_text: new_name.to_string(),
                });
            }

            if let Value::Block(children) = &ass.value.value {
                find_ability_references_in_entries(children, old_name, new_name, edits);
            } else if let Value::String(s) = &ass.value.value {
                if s == old_name
                    && (ass.key == "has_ability"
                        || ass.key == "add_ability"
                        || ass.key == "remove_ability")
                {
                    edits.push(TextEdit {
                        range: range_to_lsp(&ass.value.range),
                        new_text: new_name.to_string(),
                    });
                }
            }
        }
    }
}

/// Find all references to a color code in loc files and gfx files
fn find_color_code_references(
    old_name: &str,
    new_name: &str,
    documents: &dashmap::DashMap<String, String>,
    workspace_files: &DashSet<InternedStr>,
    changes: &mut HashMap<Uri, Vec<TextEdit>>,
) {
    // Only allow single-character color codes
    if old_name.len() != 1 || new_name.len() != 1 {
        return;
    }

    // Search in open documents
    for entry in documents.iter() {
        let uri_str = entry.key();
        let content = entry.value();
        let mut edits = Vec::new();

        if uri_str.ends_with(".yml") {
            // In loc files, replace §old with §new
            for (line_idx, line) in content.lines().enumerate() {
                let old_pattern = format!("§{}", old_name);
                let new_pattern = format!("§{}", new_name);
                let mut search_start = 0;
                while let Some(pos) = line[search_start..].find(&old_pattern) {
                    let abs_pos = search_start + pos;
                    edits.push(TextEdit {
                        range: LspRange {
                            start: LspPosition {
                                line: line_idx as u32,
                                character: abs_pos as u32,
                            },
                            end: LspPosition {
                                line: line_idx as u32,
                                character: (abs_pos + old_pattern.len()) as u32,
                            },
                        },
                        new_text: new_pattern.clone(),
                    });
                    search_start = abs_pos + 1;
                }
            }
        }

        if !edits.is_empty() {
            if let Ok(url) = uri_str.parse::<Uri>() {
                changes.insert(url, edits);
            }
        }
    }

    // Process unopened workspace files
    for entry in workspace_files.iter() {
        let file_path: &str = &entry;
        let Some(url) = Uri::from_file_path(std::path::Path::new(file_path)) else {
            continue;
        };
        let uri_str = url.as_str().to_string();
        if documents.contains_key(&uri_str) {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(file_path) {
            let mut edits = Vec::new();
            for (line_idx, line) in content.lines().enumerate() {
                let old_pattern = format!("§{}", old_name);
                let new_pattern = format!("§{}", new_name);
                let mut search_start = 0;
                while let Some(pos) = line[search_start..].find(&old_pattern) {
                    let abs_pos = search_start + pos;
                    edits.push(TextEdit {
                        range: LspRange {
                            start: LspPosition {
                                line: line_idx as u32,
                                character: abs_pos as u32,
                            },
                            end: LspPosition {
                                line: line_idx as u32,
                                character: (abs_pos + old_pattern.len()) as u32,
                            },
                        },
                        new_text: new_pattern.clone(),
                    });
                    search_start = abs_pos + 1;
                }
            }
            if !edits.is_empty() {
                changes.insert(url, edits);
            }
        }
    }
}

/// Helper functions
fn range_to_lsp(range: &Range) -> LspRange {
    LspRange {
        start: LspPosition {
            line: range.start_line,
            character: range.start_col,
        },
        end: LspPosition {
            line: range.end_line,
            character: range.end_col,
        },
    }
}
