use crate::data::interner::InternedStr;
use crate::parser::ast::{Entry, Value};
use crate::utils::lsp_convert::RangeMapper;
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
    let parsed_uri = uri.parse::<Uri>().ok()?;
    let path = parsed_uri.to_file_path()?;
    let path_str = path.to_string_lossy();
    let raw_content = std::fs::read_to_string(&path).ok()?;
    let mapper = RangeMapper::new(&raw_content);
    let lookup = crate::data::entity_lookup::EntityLookup::new(data);
    if let Some((_, range, _)) = lookup.entity_at(&path_str, &raw_content, position) {
        return Some(PrepareRenameResponse::Range(mapper.range(&range)));
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
    let parsed_uri = uri.parse::<Uri>().ok()?;
    let path = parsed_uri.to_file_path()?;
    let path_str = path.to_string_lossy();

    // Find what symbol we're renaming
    let content = match documents.get(uri).map(|s| s.clone()) {
        Some(c) => Some(c),
        None => std::fs::read_to_string(&path).ok(),
    };
    let symbol = match content.as_deref() {
        Some(c) => find_symbol_at_position(&path_str, &position, data, c).await?,
        None => return None,
    };

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
    content: &str,
) -> Option<RenameableSymbol> {
    let lookup = crate::data::entity_lookup::EntityLookup::new(data);
    if let Some((kind, _, name)) = lookup.entity_at(path, content, *position) {
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
        find_event_references_in_entries(
            &script.entries,
            old_name,
            new_name,
            &mut edits,
            &script.source,
        );

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
            find_event_references_in_entries(
                &script.entries,
                old_name,
                new_name,
                &mut edits,
                &script.source,
            );
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
    source: &str,
) {
    let mapper = RangeMapper::new(source);
    for entry in entries {
        if let Entry::Assignment(ass) = entry {
            // Check for event triggers: country_event = { id = old_name }
            if ass.key_text(source) == "country_event"
                || ass.key_text(source) == "state_event"
                || ass.key_text(source) == "news_event"
                || ass.key_text(source) == "unit_leader_event"
            {
                if let Value::Block(children) = &ass.value.value {
                    for child in children {
                        if let Entry::Assignment(child_ass) = child {
                            if child_ass.key_text(source) == "id" {
                                if let Some(id) = child_ass.value.value.as_str(source) {
                                    if id == old_name {
                                        edits.push(TextEdit {
                                            range: mapper.range(&child_ass.value.range),
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
                find_event_references_in_entries(children, old_name, new_name, edits, source);
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
        find_scripted_references_in_entries(
            &script.entries,
            old_name,
            new_name,
            &mut edits,
            &script.source,
        );

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
            find_scripted_references_in_entries(
                &script.entries,
                old_name,
                new_name,
                &mut edits,
                &script.source,
            );
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
        find_scripted_references_in_entries(
            &script.entries,
            old_name,
            new_name,
            &mut edits,
            &script.source,
        );

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
            find_scripted_references_in_entries(
                &script.entries,
                old_name,
                new_name,
                &mut edits,
                &script.source,
            );
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
    source: &str,
) {
    let mapper = RangeMapper::new(source);
    for entry in entries {
        if let Entry::Assignment(ass) = entry {
            // Key matches the scripted trigger/effect name — this is both the
            // definition site and any usage (definition: old_name = { .. },
            // usage: old_name = yes). Emit exactly one edit per occurrence.
            if ass.key_text(source) == old_name {
                edits.push(TextEdit {
                    range: mapper.range(&ass.key_range),
                    new_text: new_name.to_string(),
                });
            }

            // Recurse into blocks
            if let Value::Block(children) = &ass.value.value {
                find_scripted_references_in_entries(children, old_name, new_name, edits, source);
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
        find_idea_references_in_entries(
            &script.entries,
            old_name,
            new_name,
            &mut edits,
            &script.source,
        );

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
            find_idea_references_in_entries(
                &script.entries,
                old_name,
                new_name,
                &mut edits,
                &script.source,
            );
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
    source: &str,
) {
    let mapper = RangeMapper::new(source);
    for entry in entries {
        if let Entry::Assignment(ass) = entry {
            // Check for idea definition or usage
            if ass.key_text(source) == old_name {
                edits.push(TextEdit {
                    range: mapper.range(&ass.key_range),
                    new_text: new_name.to_string(),
                });
            }

            // Check for add_ideas/remove_ideas
            if ass.key_text(source) == "add_ideas"
                || ass.key_text(source) == "remove_ideas"
                || ass.key_text(source) == "add_timed_idea"
                || ass.key_text(source) == "swap_ideas"
            {
                if let Some(idea_name) = ass.value.value.as_str(source) {
                    if idea_name == old_name {
                        edits.push(TextEdit {
                            range: mapper.range(&ass.value.range),
                            new_text: format!("\"{}\"", new_name),
                        });
                    }
                }
            }

            // Recurse into blocks
            if let Value::Block(children) = &ass.value.value {
                find_idea_references_in_entries(children, old_name, new_name, edits, source);
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
        find_character_references_in_entries(
            &script.entries,
            old_name,
            new_name,
            &mut edits,
            &script.source,
        );

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
            find_character_references_in_entries(
                &script.entries,
                old_name,
                new_name,
                &mut edits,
                &script.source,
            );
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
    source: &str,
) {
    let mapper = RangeMapper::new(source);
    for entry in entries {
        if let Entry::Assignment(ass) = entry {
            // Character definition
            if ass.key_text(source) == old_name {
                edits.push(TextEdit {
                    range: mapper.range(&ass.key_range),
                    new_text: new_name.to_string(),
                });
            }

            // Character usage (recruit_character, etc)
            if ass.key_text(source) == "recruit_character"
                || ass.key_text(source) == "has_character"
                || ass.key_text(source) == "promote_character"
                || ass.key_text(source) == "retire_character"
            {
                if let Some(char_name) = ass.value.value.as_str(source) {
                    if char_name == old_name {
                        edits.push(TextEdit {
                            range: mapper.range(&ass.value.range),
                            new_text: new_name.to_string(),
                        });
                    }
                }
            }

            // character = X block usage
            if ass.key_text(source) == "character" {
                if let Some(char_name) = ass.value.value.as_str(source) {
                    if char_name == old_name {
                        edits.push(TextEdit {
                            range: mapper.range(&ass.value.range),
                            new_text: new_name.to_string(),
                        });
                    }
                }
            }

            // Recurse into blocks
            if let Value::Block(children) = &ass.value.value {
                find_character_references_in_entries(children, old_name, new_name, edits, source);
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
        find_variable_references_in_entries(
            &script.entries,
            old_name,
            new_name,
            &mut edits,
            &script.source,
        );

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
            find_variable_references_in_entries(
                &script.entries,
                old_name,
                new_name,
                &mut edits,
                &script.source,
            );
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
    source: &str,
) {
    let mapper = RangeMapper::new(source);
    for entry in entries {
        if let Entry::Assignment(ass) = entry {
            // Check for variable operations (set_variable, add_to_variable, etc.)
            let var_key = ass.key_text(source);
            let is_var_op = matches!(
                var_key,
                "set_variable"
                    | "set_temp_variable"
                    | "set_variable_to_random"
                    | "set_temp_variable_to_random"
                    | "check_variable"
                    | "add_to_variable"
                    | "add_to_temp_variable"
                    | "subtract_from_variable"
                    | "subtract_from_temp_variable"
                    | "multiply_variable"
                    | "multiply_temp_variable"
                    | "divide_variable"
                    | "divide_temp_variable"
                    | "modulo_variable"
                    | "modulo_temp_variable"
                    | "clamp_variable"
                    | "clamp_temp_variable"
                    | "round_variable"
                    | "round_temp_variable"
                    | "clear_variable"
                    | "has_variable"
            );
            if is_var_op {
                match &ass.value.value {
                    Value::Block(children) => {
                        let mut found = false;
                        for child in children {
                            if let Entry::Assignment(child_ass) = child {
                                let child_key = child_ass.key_text(source);
                                if child_key == "var"
                                    || child_key == "variable"
                                    || child_key == "name"
                                    || child_key == "temp_var"
                                {
                                    if let Some(var_name) = child_ass.value.value.as_str(source) {
                                        if var_name == old_name {
                                            edits.push(TextEdit {
                                                range: mapper.range(&child_ass.value.range),
                                                new_text: format!("\"{}\"", new_name),
                                            });
                                            found = true;
                                        }
                                    }
                                }
                            }
                        }
                        // Shorthand form: single entry with no explicit var/temp_var, key IS variable name
                        if !found && children.len() == 1 {
                            if let Entry::Assignment(child_ass) = &children[0] {
                                if child_ass.key_text(source) == old_name {
                                    edits.push(TextEdit {
                                        range: mapper.range(&child_ass.key_range),
                                        new_text: format!("\"{}\"", new_name),
                                    });
                                }
                            }
                        }
                    }
                    _ => {
                        // String form: clear_variable = my_var, set_variable_to_random = my_var, etc.
                        if let Some(var_name) = ass.value.value.as_str(source) {
                            if var_name == old_name {
                                edits.push(TextEdit {
                                    range: mapper.range(&ass.value.range),
                                    new_text: format!("\"{}\"", new_name),
                                });
                            }
                        }
                    }
                }
            }

            // Recurse into blocks
            if let Value::Block(children) = &ass.value.value {
                find_variable_references_in_entries(children, old_name, new_name, edits, source);
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
        find_ability_references_in_entries(
            &script.entries,
            old_name,
            new_name,
            &mut edits,
            &script.source,
        );

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
            find_ability_references_in_entries(
                &script.entries,
                old_name,
                new_name,
                &mut edits,
                &script.source,
            );
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
    source: &str,
) {
    let mapper = RangeMapper::new(source);
    for entry in entries {
        if let Entry::Assignment(ass) = entry {
            if ass.key_text(source) == old_name {
                edits.push(TextEdit {
                    range: mapper.range(&ass.key_range),
                    new_text: new_name.to_string(),
                });
            }

            if let Value::Block(children) = &ass.value.value {
                find_ability_references_in_entries(children, old_name, new_name, edits, source);
            } else if let Some(s) = ass.value.value.as_str(source) {
                if s == old_name
                    && (ass.key_text(source) == "has_ability"
                        || ass.key_text(source) == "add_ability"
                        || ass.key_text(source) == "remove_ability")
                {
                    edits.push(TextEdit {
                        range: mapper.range(&ass.value.range),
                        new_text: new_name.to_string(),
                    });
                }
            }
        }
    }
}

/// Collect the color-code edits for a single loc file's content.
///
/// Three things this must get right, all of which the two inlined copies of
/// this loop previously got wrong:
///
/// 1. **Advance by the whole match.** `§` is 2 bytes (`0xC2 0xA7`), so a match
///    always starts on a lead byte and `abs_pos + 1` lands *inside* the
///    character — the next `line[search_start..]` slice then panics with
///    "byte index N is not a char boundary". Advancing by `old_pattern.len()`
///    keeps the cursor on a boundary (and skips the match we just handled).
/// 2. **Emit UTF-16 columns.** `str::find` returns a BYTE offset but
///    `Position.character` is UTF-16 code units. Since `§` is itself
///    multi-byte, every code on a line drifts the columns further right,
///    corrupting the file when the edits are applied.
/// 3. **Use the UTF-16 length of the pattern.** `"§R".len()` is 3 bytes but
///    only 2 UTF-16 units, so the end column was one unit too wide and ate
///    the following character.
fn collect_color_code_edits(content: &str, old_name: &str, new_name: &str) -> Vec<TextEdit> {
    let old_pattern = format!("§{old_name}");
    let new_pattern = format!("§{new_name}");
    let pattern_utf16_len = old_pattern.chars().map(char::len_utf16).sum::<usize>() as u32;
    let mut edits = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        if !line.contains(&old_pattern) {
            continue;
        }
        // One index per line; byte->UTF-16 lookups are then O(1).
        let index = crate::utils::line_index::LineIndex::new(line);
        let mut search_start = 0;
        while let Some(pos) = line[search_start..].find(&old_pattern) {
            let abs_pos = search_start + pos;
            let start_utf16 = index.byte_to_utf16(abs_pos);
            edits.push(TextEdit {
                range: LspRange {
                    start: LspPosition {
                        line: line_idx as u32,
                        character: start_utf16,
                    },
                    end: LspPosition {
                        line: line_idx as u32,
                        character: start_utf16 + pattern_utf16_len,
                    },
                },
                new_text: new_pattern.clone(),
            });
            // Skip past the whole match — stays on a char boundary and avoids
            // rescanning overlapping matches.
            search_start = abs_pos + old_pattern.len();
        }
    }
    edits
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
    if old_name.chars().count() != 1 || new_name.chars().count() != 1 {
        return;
    }

    // Search in open documents
    for entry in documents.iter() {
        let uri_str = entry.key();
        let content = entry.value();

        if !uri_str.ends_with(".yml") {
            continue;
        }
        // In loc files, replace §old with §new
        let edits = collect_color_code_edits(content, old_name, new_name);

        if !edits.is_empty() {
            if let Ok(url) = uri_str.parse::<Uri>() {
                changes.insert(url, edits);
            }
        }
    }

    // Process unopened workspace files
    for entry in workspace_files.iter() {
        let file_path: &str = &entry;
        if !file_path.ends_with(".yml") {
            continue;
        }
        let Some(url) = Uri::from_file_path(std::path::Path::new(file_path)) else {
            continue;
        };
        let uri_str = url.as_str().to_string();
        if documents.contains_key(&uri_str) {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(file_path) {
            let edits = collect_color_code_edits(&content, old_name, new_name);
            if !edits.is_empty() {
                changes.insert(url, edits);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parser::parse_script;

    /// Regression test: renaming a scripted trigger/effect must produce exactly
    /// ONE edit per occurrence. Previously the `find_scripted_references_in_entries`
    /// walker had two byte-identical guards pushing the same TextEdit for both the
    /// definition and usage paths, doubling every edit (VS Code rejects or
    /// double-applies the overlapping edits).
    #[test]
    fn test_scripted_rename_emits_one_edit_per_occurrence() {
        // Two occurrences: a definition block and a usage. A definition and a
        // usage on separate keys would each match once.
        let source = "my_trigger = {\n    some_effect = yes\n}\nmy_trigger = yes\n";
        let (script, errors) = parse_script(source);
        assert!(errors.is_empty(), "parse errors: {errors:?}");

        let mut edits = Vec::new();
        find_scripted_references_in_entries(
            &script.entries,
            "my_trigger",
            "renamed_trigger",
            &mut edits,
            &script.source,
        );

        // Two occurrences → exactly two edits, not four (the pre-fix doubling).
        assert_eq!(edits.len(), 2, "expected one edit per occurrence");
        for edit in &edits {
            assert_eq!(edit.new_text, "renamed_trigger");
        }
    }

    /// Regression: renaming a color code used to PANIC.
    ///
    /// `search_start = abs_pos + 1` landed on the continuation byte of `§`
    /// (0xC2 0xA7), so the next `line[search_start..]` slice sliced mid-char:
    /// "byte index 8 is not a char boundary". Any loc line with two color
    /// codes killed the whole textDocument/rename request.
    #[test]
    fn test_color_code_rename_no_panic_multiple_codes() {
        let content = " KEY: \"\u{a7}Rred \u{a7}Ggreen \u{a7}Rmore\"\n";
        let edits = collect_color_code_edits(content, "R", "B");
        assert_eq!(edits.len(), 2, "both \u{a7}R occurrences found: {edits:?}");
        for e in &edits {
            assert_eq!(e.new_text, "\u{a7}B");
        }
    }

    /// Columns must be UTF-16, not byte offsets. `\u{a7}` is 2 bytes but 1 UTF-16
    /// unit, so byte offsets drift right by one per preceding code.
    #[test]
    fn test_color_code_rename_emits_utf16_columns() {
        //            0    1234   5 6789...      (UTF-16 columns)
        let content = "KEY: \"\u{a7}Rred \u{a7}Ggreen\"\n";
        let edits = collect_color_code_edits(content, "G", "Y");
        assert_eq!(edits.len(), 1);
        let r = edits[0].range;
        // Count UTF-16 units before the "\u{a7}G" ourselves.
        let line = content.lines().next().unwrap();
        let byte_pos = line.find("\u{a7}G").unwrap();
        let expected: u32 = line[..byte_pos].chars().map(char::len_utf16).sum::<usize>() as u32;
        assert_eq!(r.start.character, expected, "start must be a UTF-16 column");
        // "\u{a7}G" is 2 UTF-16 units (not the 3 bytes it occupies).
        assert_eq!(r.end.character, expected + 2, "width must be UTF-16 length");
        assert_eq!(r.start.line, 0);
    }

    /// A 4-byte astral char before the code shifts bytes by 4 but UTF-16 by 2.
    #[test]
    fn test_color_code_rename_columns_with_astral_char() {
        let content = "KEY: \"\u{1f396} \u{a7}Rred\"\n";
        let edits = collect_color_code_edits(content, "R", "B");
        assert_eq!(edits.len(), 1);
        let line = content.lines().next().unwrap();
        let byte_pos = line.find("\u{a7}R").unwrap();
        let expected: u32 = line[..byte_pos].chars().map(char::len_utf16).sum::<usize>() as u32;
        assert_eq!(edits[0].range.start.character, expected);
        assert_ne!(
            edits[0].range.start.character, byte_pos as u32,
            "byte offset and UTF-16 column must differ here (proves conversion)"
        );
    }

    /// Adjacent codes (`\u{a7}R\u{a7}G`) must not produce overlapping edits — advancing by
    /// the full match length also prevents rescanning.
    #[test]
    fn test_color_code_rename_adjacent_codes_no_overlap() {
        let content = "KEY: \"\u{a7}R\u{a7}R\u{a7}Rx\"\n";
        let edits = collect_color_code_edits(content, "R", "B");
        assert_eq!(edits.len(), 3);
        for w in edits.windows(2) {
            assert!(
                w[0].range.end.character <= w[1].range.start.character,
                "edits must not overlap: {:?} then {:?}",
                w[0].range,
                w[1].range
            );
        }
    }
}
