use crate::ast::{Entry, Range, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::lsp_types::{
    Position as LspPosition, PrepareRenameResponse, Range as LspRange, TextEdit, Url, WorkspaceEdit,
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
}

/// Prepare rename - check if the symbol at the position can be renamed
pub async fn prepare_rename(
    uri: &str,
    position: LspPosition,
    events: &Arc<RwLock<HashMap<String, crate::event_scanner::Event>>>,
    scripted_triggers: &Arc<RwLock<HashMap<String, crate::scripted_scanner::ScriptedEntity>>>,
    scripted_effects: &Arc<RwLock<HashMap<String, crate::scripted_scanner::ScriptedEntity>>>,
    ideas: &Arc<RwLock<HashMap<String, crate::idea_scanner::Idea>>>,
    characters: &Arc<RwLock<HashMap<String, crate::character_scanner::Character>>>,
    variables: &Arc<RwLock<HashMap<String, Vec<crate::variable_scanner::Variable>>>>,
    abilities: &Arc<RwLock<HashMap<String, crate::ability_scanner::Ability>>>,
) -> Option<PrepareRenameResponse> {
    let path = uri.trim_start_matches("file://");

    // Check if position is on an event
    let events_lock = events.read().await;
    for (_id, event) in events_lock.iter() {
        if event.path == path && position_in_range(&position, &event.range) {
            return Some(PrepareRenameResponse::Range(range_to_lsp(&event.range)));
        }
    }
    drop(events_lock);

    // Check if position is on a scripted trigger
    let triggers_lock = scripted_triggers.read().await;
    for (_name, trigger) in triggers_lock.iter() {
        if trigger.path == path && position_in_range(&position, &trigger.range) {
            return Some(PrepareRenameResponse::Range(range_to_lsp(&trigger.range)));
        }
    }
    drop(triggers_lock);

    // Check if position is on a scripted effect
    let effects_lock = scripted_effects.read().await;
    for (_name, effect) in effects_lock.iter() {
        if effect.path == path && position_in_range(&position, &effect.range) {
            return Some(PrepareRenameResponse::Range(range_to_lsp(&effect.range)));
        }
    }
    drop(effects_lock);

    // Check if position is on an idea
    let ideas_lock = ideas.read().await;
    for (_name, idea) in ideas_lock.iter() {
        if idea.path == path && position_in_range(&position, &idea.range) {
            return Some(PrepareRenameResponse::Range(range_to_lsp(&idea.range)));
        }
    }
    drop(ideas_lock);

    // Check if position is on a character
    let characters_lock = characters.read().await;
    for (_name, character) in characters_lock.iter() {
        if character.path == path && position_in_range(&position, &character.range) {
            return Some(PrepareRenameResponse::Range(range_to_lsp(&character.range)));
        }
    }
    drop(characters_lock);

    // Check if position is on an ability
    let abilities_lock = abilities.read().await;
    for (_name, ability) in abilities_lock.iter() {
        if ability.path == path && position_in_range(&position, &ability.range) {
            return Some(PrepareRenameResponse::Range(range_to_lsp(&ability.range)));
        }
    }
    drop(abilities_lock);

    // Check if position is on a variable
    let variables_lock = variables.read().await;
    for (_name, var_list) in variables_lock.iter() {
        for var in var_list {
            if var.path == path && position_in_range(&position, &var.range) {
                return Some(PrepareRenameResponse::Range(range_to_lsp(&var.range)));
            }
        }
    }

    None
}

/// Perform rename - find all references and create workspace edit
pub async fn rename_symbol(
    uri: &str,
    position: LspPosition,
    new_name: &str,
    events: &Arc<RwLock<HashMap<String, crate::event_scanner::Event>>>,
    scripted_triggers: &Arc<RwLock<HashMap<String, crate::scripted_scanner::ScriptedEntity>>>,
    scripted_effects: &Arc<RwLock<HashMap<String, crate::scripted_scanner::ScriptedEntity>>>,
    ideas: &Arc<RwLock<HashMap<String, crate::idea_scanner::Idea>>>,
    characters: &Arc<RwLock<HashMap<String, crate::character_scanner::Character>>>,
    variables: &Arc<RwLock<HashMap<String, Vec<crate::variable_scanner::Variable>>>>,
    abilities: &Arc<RwLock<HashMap<String, crate::ability_scanner::Ability>>>,
    documents: &dashmap::DashMap<String, String>,
) -> Option<WorkspaceEdit> {
    let path = uri.trim_start_matches("file://");

    // Find what symbol we're renaming
    let symbol = find_symbol_at_position(
        path,
        &position,
        events,
        scripted_triggers,
        scripted_effects,
        ideas,
        characters,
        variables,
        abilities,
    )
    .await?;

    // Find all references to this symbol
    let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();

    match symbol {
        RenameableSymbol::Event(old_name) => {
            find_event_references(&old_name, new_name, documents, &mut changes);
        }
        RenameableSymbol::ScriptedTrigger(old_name) => {
            find_scripted_trigger_references(&old_name, new_name, documents, &mut changes);
        }
        RenameableSymbol::ScriptedEffect(old_name) => {
            find_scripted_effect_references(&old_name, new_name, documents, &mut changes);
        }
        RenameableSymbol::Idea(old_name) => {
            find_idea_references(&old_name, new_name, documents, &mut changes);
        }
        RenameableSymbol::Character(old_name) => {
            find_character_references(&old_name, new_name, documents, &mut changes);
        }
        RenameableSymbol::Ability(old_name) => {
            find_ability_references(&old_name, new_name, documents, &mut changes);
        }
        RenameableSymbol::Variable(old_name) => {
            find_variable_references(&old_name, new_name, documents, &mut changes);
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
    events: &Arc<RwLock<HashMap<String, crate::event_scanner::Event>>>,
    scripted_triggers: &Arc<RwLock<HashMap<String, crate::scripted_scanner::ScriptedEntity>>>,
    scripted_effects: &Arc<RwLock<HashMap<String, crate::scripted_scanner::ScriptedEntity>>>,
    ideas: &Arc<RwLock<HashMap<String, crate::idea_scanner::Idea>>>,
    characters: &Arc<RwLock<HashMap<String, crate::character_scanner::Character>>>,
    variables: &Arc<RwLock<HashMap<String, Vec<crate::variable_scanner::Variable>>>>,
    abilities: &Arc<RwLock<HashMap<String, crate::ability_scanner::Ability>>>,
) -> Option<RenameableSymbol> {
    // Check events
    let events_lock = events.read().await;
    for (id, event) in events_lock.iter() {
        if event.path == path && position_in_range(position, &event.range) {
            return Some(RenameableSymbol::Event(id.clone()));
        }
    }
    drop(events_lock);

    // Check scripted triggers
    let triggers_lock = scripted_triggers.read().await;
    for (name, trigger) in triggers_lock.iter() {
        if trigger.path == path && position_in_range(position, &trigger.range) {
            return Some(RenameableSymbol::ScriptedTrigger(name.clone()));
        }
    }
    drop(triggers_lock);

    // Check scripted effects
    let effects_lock = scripted_effects.read().await;
    for (name, effect) in effects_lock.iter() {
        if effect.path == path && position_in_range(position, &effect.range) {
            return Some(RenameableSymbol::ScriptedEffect(name.clone()));
        }
    }
    drop(effects_lock);

    // Check ideas
    let ideas_lock = ideas.read().await;
    for (name, idea) in ideas_lock.iter() {
        if idea.path == path && position_in_range(position, &idea.range) {
            return Some(RenameableSymbol::Idea(name.clone()));
        }
    }
    drop(ideas_lock);

    // Check characters
    let characters_lock = characters.read().await;
    for (name, character) in characters_lock.iter() {
        if character.path == path && position_in_range(position, &character.range) {
            return Some(RenameableSymbol::Character(name.clone()));
        }
    }
    drop(characters_lock);

    // Check abilities
    let abilities_lock = abilities.read().await;
    for (id, ability) in abilities_lock.iter() {
        if ability.path == path && position_in_range(position, &ability.range) {
            return Some(RenameableSymbol::Ability(id.clone()));
        }
    }
    drop(abilities_lock);

    // Check variables
    let variables_lock = variables.read().await;
    for (name, var_list) in variables_lock.iter() {
        for var in var_list {
            if var.path == path && position_in_range(position, &var.range) {
                return Some(RenameableSymbol::Variable(name.clone()));
            }
        }
    }
    None
}

/// Find all references to an event and create text edits
fn find_event_references(
    old_name: &str,
    new_name: &str,
    documents: &dashmap::DashMap<String, String>,
    changes: &mut HashMap<Url, Vec<TextEdit>>,
) {
    for entry in documents.iter() {
        let uri_str = entry.key();
        let content = entry.value();

        // Parse the document
        {
            let (script, _) = crate::parser::parse_script(content);
            let mut edits = Vec::new();
            find_event_references_in_entries(&script.entries, old_name, new_name, &mut edits);

            if !edits.is_empty() {
                if let Ok(url) = Url::parse(uri_str) {
                    changes.insert(url, edits);
                }
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
        match entry {
            Entry::Assignment(ass) => {
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
            _ => {}
        }
    }
}

/// Find all references to a scripted trigger
fn find_scripted_trigger_references(
    old_name: &str,
    new_name: &str,
    documents: &dashmap::DashMap<String, String>,
    changes: &mut HashMap<Url, Vec<TextEdit>>,
) {
    for entry in documents.iter() {
        let uri_str = entry.key();
        let content = entry.value();

        {
            let (script, _) = crate::parser::parse_script(content);
            let mut edits = Vec::new();
            find_scripted_references_in_entries(
                &script.entries,
                old_name,
                new_name,
                &mut edits,
                true,
            );

            if !edits.is_empty() {
                if let Ok(url) = Url::parse(uri_str) {
                    changes.insert(url, edits);
                }
            }
        }
    }
}

/// Find all references to a scripted effect
fn find_scripted_effect_references(
    old_name: &str,
    new_name: &str,
    documents: &dashmap::DashMap<String, String>,
    changes: &mut HashMap<Url, Vec<TextEdit>>,
) {
    for entry in documents.iter() {
        let uri_str = entry.key();
        let content = entry.value();

        {
            let (script, _) = crate::parser::parse_script(content);
            let mut edits = Vec::new();
            find_scripted_references_in_entries(
                &script.entries,
                old_name,
                new_name,
                &mut edits,
                false,
            );

            if !edits.is_empty() {
                if let Ok(url) = Url::parse(uri_str) {
                    changes.insert(url, edits);
                }
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
    is_trigger: bool,
) {
    for entry in entries {
        match entry {
            Entry::Assignment(ass) => {
                // Check for definition
                if (is_trigger && ass.key == old_name) || (!is_trigger && ass.key == old_name) {
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
                    find_scripted_references_in_entries(
                        children, old_name, new_name, edits, is_trigger,
                    );
                }
            }
            _ => {}
        }
    }
}

/// Find all references to an idea
fn find_idea_references(
    old_name: &str,
    new_name: &str,
    documents: &dashmap::DashMap<String, String>,
    changes: &mut HashMap<Url, Vec<TextEdit>>,
) {
    for entry in documents.iter() {
        let uri_str = entry.key();
        let content = entry.value();

        {
            let (script, _) = crate::parser::parse_script(content);
            let mut edits = Vec::new();
            find_idea_references_in_entries(&script.entries, old_name, new_name, &mut edits);

            if !edits.is_empty() {
                if let Ok(url) = Url::parse(uri_str) {
                    changes.insert(url, edits);
                }
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
        match entry {
            Entry::Assignment(ass) => {
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
            _ => {}
        }
    }
}

/// Find all references to a character
fn find_character_references(
    old_name: &str,
    new_name: &str,
    documents: &dashmap::DashMap<String, String>,
    changes: &mut HashMap<Url, Vec<TextEdit>>,
) {
    for entry in documents.iter() {
        let uri_str = entry.key();
        let content = entry.value();

        {
            let (script, _) = crate::parser::parse_script(content);
            let mut edits = Vec::new();
            find_character_references_in_entries(&script.entries, old_name, new_name, &mut edits);

            if !edits.is_empty() {
                if let Ok(url) = Url::parse(uri_str) {
                    changes.insert(url, edits);
                }
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
        match entry {
            Entry::Assignment(ass) => {
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
            _ => {}
        }
    }
}

/// Find all references to a variable
fn find_variable_references(
    old_name: &str,
    new_name: &str,
    documents: &dashmap::DashMap<String, String>,
    changes: &mut HashMap<Url, Vec<TextEdit>>,
) {
    for entry in documents.iter() {
        let uri_str = entry.key();
        let content = entry.value();

        {
            let (script, _) = crate::parser::parse_script(content);
            let mut edits = Vec::new();
            find_variable_references_in_entries(&script.entries, old_name, new_name, &mut edits);

            if !edits.is_empty() {
                if let Ok(url) = Url::parse(uri_str) {
                    changes.insert(url, edits);
                }
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
        match entry {
            Entry::Assignment(ass) => {
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
            _ => {}
        }
    }
}

/// Find all references to an ability
fn find_ability_references(
    old_name: &str,
    new_name: &str,
    documents: &dashmap::DashMap<String, String>,
    changes: &mut HashMap<Url, Vec<TextEdit>>,
) {
    for entry in documents.iter() {
        let uri_str = entry.key();
        let content = entry.value();

        {
            let (script, _) = crate::parser::parse_script(content);
            let mut edits = Vec::new();
            find_ability_references_in_entries(&script.entries, old_name, new_name, &mut edits);

            if !edits.is_empty() {
                if let Ok(url) = Url::parse(uri_str) {
                    changes.insert(url, edits);
                }
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
        match entry {
            Entry::Assignment(ass) => {
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
            _ => {}
        }
    }
}

/// Helper functions
fn position_in_range(position: &LspPosition, range: &Range) -> bool {
    let line = position.line;
    let character = position.character;

    (line > range.start_line || (line == range.start_line && character >= range.start_col))
        && (line < range.end_line || (line == range.end_line && character <= range.end_col))
}

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
