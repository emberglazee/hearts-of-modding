use crate::parser::ast::{Entry, NodeedValue, Range, Value};
use tower_lsp_server::ls_types::{
    DocumentSymbol, Position as LspPosition, Range as LspRange, SymbolKind,
};

/// Generate document symbols for outline view
pub fn generate_document_symbols(entries: &[Entry]) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();

    for entry in entries {
        if let Some(symbol) = entry_to_symbol(entry) {
            symbols.push(symbol);
        }
    }

    symbols
}

/// Convert an AST entry to a DocumentSymbol
fn entry_to_symbol(entry: &Entry) -> Option<DocumentSymbol> {
    match entry {
        Entry::Assignment(assignment) => {
            let symbol_kind = classify_assignment(&assignment.key);
            let name = extract_symbol_name(&assignment.key, &assignment.value);
            let detail = extract_symbol_detail(&assignment.key, &assignment.value);

            // Recursively process children if it's a block
            let children = if let Value::Block(entries) = &assignment.value.value {
                let child_symbols: Vec<DocumentSymbol> =
                    entries.iter().filter_map(entry_to_symbol).collect();

                if child_symbols.is_empty() {
                    None
                } else {
                    Some(child_symbols)
                }
            } else {
                None
            };

            let range = Range {
                start_line: assignment.key_range.start_line,
                start_col: assignment.key_range.start_col,
                end_line: assignment.value.range.end_line,
                end_col: assignment.value.range.end_col,
            };

            #[allow(deprecated)]
            Some(DocumentSymbol {
                name,
                detail,
                kind: symbol_kind,
                tags: None,
                deprecated: None,
                range: range_to_lsp(&range),
                selection_range: range_to_lsp(&assignment.key_range),
                children,
            })
        }
        Entry::Comment(_, _) => None,
        Entry::Value(nodeed_value) => match &nodeed_value.value {
            Value::Block(entries) => {
                let children: Vec<DocumentSymbol> =
                    entries.iter().filter_map(entry_to_symbol).collect();
                let children = if children.is_empty() {
                    None
                } else {
                    Some(children)
                };

                let range = range_to_lsp(&nodeed_value.range);

                #[allow(deprecated)]
                Some(DocumentSymbol {
                    name: "{ ... }".to_string(),
                    detail: None,
                    kind: SymbolKind::CONSTRUCTOR,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children,
                })
            }
            Value::TaggedBlock(tag, entries, _) => {
                let children: Vec<DocumentSymbol> =
                    entries.iter().filter_map(entry_to_symbol).collect();
                let children = if children.is_empty() {
                    None
                } else {
                    Some(children)
                };

                let range = range_to_lsp(&nodeed_value.range);

                #[allow(deprecated)]
                Some(DocumentSymbol {
                    name: format!("{tag} {{ ... }}"),
                    detail: None,
                    kind: SymbolKind::CONSTRUCTOR,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children,
                })
            }
            // Leaf values (String, Number, Boolean) are too noisy for document outline
            _ => None,
        },
    }
}

/// Classify an assignment by its key
fn classify_assignment(key: &str) -> SymbolKind {
    match key {
        // Events
        "country_event" | "state_event" | "news_event" | "unit_leader_event" => SymbolKind::EVENT,

        // Ideas
        "ideas"
        | "idea"
        | "country"
        | "political_advisor"
        | "theorist"
        | "army_chief"
        | "navy_chief"
        | "air_chief"
        | "high_command"
        | "tank_manufacturer"
        | "naval_manufacturer"
        | "aircraft_manufacturer"
        | "materiel_manufacturer"
        | "industrial_concern" => SymbolKind::CLASS,

        // Focus trees
        "focus_tree" | "focus" | "shared_focus" => SymbolKind::NAMESPACE,

        // Technologies
        "technologies" | "technology" => SymbolKind::INTERFACE,

        // Characters
        "characters"
        | "create_corps_commander"
        | "create_field_marshal"
        | "create_navy_leader"
        | "create_operative" => SymbolKind::STRUCT,

        // Scripted triggers/effects
        "scripted_trigger" | "scripted_effect" => SymbolKind::FUNCTION,

        // Abilities
        "ability" => SymbolKind::METHOD,

        // Modifiers
        "modifier" | "targeted_modifier" | "equipment_bonus" | "hidden_modifier" => {
            SymbolKind::PROPERTY
        }

        // Options (in events)
        "option" => SymbolKind::ENUM_MEMBER,

        // Buildings
        "buildings" => SymbolKind::MODULE,

        // States
        "state" | "history" | "provinces" | "manpower" | "victory_points" => SymbolKind::OBJECT,

        // Identifiers
        "id" | "name" | "tag" => SymbolKind::KEY,

        // Localization keys
        "title" | "desc" | "text" | "type" | "sound_effect" => SymbolKind::STRING,

        // Numeric values
        "cost" | "duration" | "cooldown" | "skill" | "attack_skill" | "defense_skill"
        | "planning_skill" | "logistics_skill" | "maneuvering_skill" | "coordination_skill"
        | "value" => SymbolKind::NUMBER,

        // Boolean flags
        "fire_only_once" | "is_triggered_only" | "major" | "hidden" | "cancelable" => {
            SymbolKind::BOOLEAN
        }

        // Default
        _ => SymbolKind::FIELD,
    }
}

/// Extract a meaningful name for the symbol
fn extract_symbol_name(key: &str, value: &NodeedValue) -> String {
    // For events, try to extract the ID
    if key == "country_event"
        || key == "state_event"
        || key == "news_event"
        || key == "unit_leader_event"
    {
        if let Value::Block(entries) = &value.value {
            for entry in entries {
                if let Entry::Assignment(ass) = entry {
                    if ass.key == "id" {
                        if let Value::String(id) = &ass.value.value {
                            return format!("{} ({})", key, id);
                        }
                    }
                }
            }
        }
        return key.to_string();
    }

    // For focuses, try to extract the ID
    if key == "focus" || key == "shared_focus" {
        if let Value::Block(entries) = &value.value {
            for entry in entries {
                if let Entry::Assignment(ass) = entry {
                    if ass.key == "id" {
                        if let Value::String(id) = &ass.value.value {
                            return format!("{} ({})", key, id);
                        }
                    }
                }
            }
        }
        return key.to_string();
    }

    // For options, try to extract the name
    if key == "option" {
        if let Value::Block(entries) = &value.value {
            for entry in entries {
                if let Entry::Assignment(ass) = entry {
                    if ass.key == "name" {
                        if let Value::String(name) = &ass.value.value {
                            return format!("option ({})", name);
                        }
                    }
                }
            }
        }
        return "option".to_string();
    }

    key.to_string()
}

/// Extract detail information for the symbol
fn extract_symbol_detail(key: &str, value: &NodeedValue) -> Option<String> {
    // For events, extract title
    if key == "country_event"
        || key == "state_event"
        || key == "news_event"
        || key == "unit_leader_event"
    {
        if let Value::Block(entries) = &value.value {
            for entry in entries {
                if let Entry::Assignment(ass) = entry {
                    if ass.key == "title" {
                        if let Value::String(title) = &ass.value.value {
                            return Some(format!("title: {}", title));
                        }
                    }
                }
            }
        }
    }

    // For focuses, extract cost
    if key == "focus" || key == "shared_focus" {
        if let Value::Block(entries) = &value.value {
            for entry in entries {
                if let Entry::Assignment(ass) = entry {
                    if ass.key == "cost" {
                        if let Value::Number(cost) = &ass.value.value {
                            return Some(format!("cost: {}", cost));
                        }
                    }
                }
            }
        }
    }

    // For simple assignments, show the value
    match &value.value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Boolean(b) => Some(b.to_string()),
        Value::Block(entries) => {
            if entries.is_empty() {
                Some("{ }".to_string())
            } else {
                Some(format!("{{ {} items }}", entries.len()))
            }
        }
        _ => None,
    }
}

/// Convert AST Range to LSP Range
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_event_assignment() {
        assert_eq!(classify_assignment("country_event"), SymbolKind::EVENT);
        assert_eq!(classify_assignment("state_event"), SymbolKind::EVENT);
    }

    #[test]
    fn test_classify_focus_assignment() {
        assert_eq!(classify_assignment("focus"), SymbolKind::NAMESPACE);
        assert_eq!(classify_assignment("focus_tree"), SymbolKind::NAMESPACE);
    }
}
