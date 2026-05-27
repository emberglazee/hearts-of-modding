use crate::achievement_scanner;
use crate::ast;
use crate::scope;
use crate::symbol_search::is_pos_in_range;
use std::collections::HashMap;
use tower_lsp_server::ls_types::Position;

pub fn find_scope_context_at(
    script: &ast::Script,
    pos: Position,
    achievements: &HashMap<String, achievement_scanner::Achievement>,
) -> (Option<String>, Vec<scope::Scope>) {
    let mut scope_stack = scope::ScopeStack::new(scope::Scope::Global);
    let mut context = None;
    for entry in &script.entries {
        if let Some(ctx) = find_scope_context_in_entry(entry, pos, &mut scope_stack, achievements) {
            context = Some(ctx);
            break;
        }
    }
    (context, scope_stack.iter().cloned().collect())
}

fn find_scope_context_in_entry(
    entry: &ast::Entry,
    pos: Position,
    scope_stack: &mut scope::ScopeStack,
    achievements: &HashMap<String, achievement_scanner::Achievement>,
) -> Option<String> {
    match entry {
        ast::Entry::Assignment(ass) => {
            if is_pos_in_range(pos, &ass.value.range) {
                if let ast::Value::Block(_) | ast::Value::TaggedBlock(_, _, _) = &ass.value.value {
                    let s = if let Some(achievement) = achievements.get(&ass.key) {
                        if achievement.is_ribbon {
                            scope::Scope::Ribbon
                        } else {
                            scope::Scope::Achievement
                        }
                    } else {
                        scope::Scope::from_str(&ass.key)
                    };
                    scope_stack.push(s);
                }

                if let Some(inner) =
                    find_scope_context_in_value(&ass.value, pos, scope_stack, achievements)
                {
                    return Some(inner);
                }

                return Some(ass.key.clone());
            }
            None
        }
        ast::Entry::Value(val) => find_scope_context_in_value(val, pos, scope_stack, achievements),
        _ => None,
    }
}

fn find_scope_context_in_value(
    val: &ast::NodeedValue,
    pos: Position,
    scope_stack: &mut scope::ScopeStack,
    achievements: &HashMap<String, achievement_scanner::Achievement>,
) -> Option<String> {
    match &val.value {
        ast::Value::Block(entries) => {
            for entry in entries {
                if let Some(ctx) =
                    find_scope_context_in_entry(entry, pos, scope_stack, achievements)
                {
                    return Some(ctx);
                }
            }
            None
        }
        ast::Value::TaggedBlock(_, entries, _) => {
            for entry in entries {
                if let Some(ctx) =
                    find_scope_context_in_entry(entry, pos, scope_stack, achievements)
                {
                    return Some(ctx);
                }
            }
            None
        }
        _ => None,
    }
}

pub fn find_context_at(script: &ast::Script, pos: Position) -> Option<String> {
    for entry in &script.entries {
        if let Some(ctx) = find_context_in_entry(entry, pos) {
            return Some(ctx);
        }
    }
    None
}

fn find_context_in_entry(entry: &ast::Entry, pos: Position) -> Option<String> {
    match entry {
        ast::Entry::Assignment(ass) => {
            if is_pos_in_range(pos, &ass.value.range) {
                if let Some(inner) = find_context_in_value(&ass.value, pos) {
                    return Some(inner);
                }
                return Some(ass.key.clone());
            }
            None
        }
        ast::Entry::Value(val) => find_context_in_value(val, pos),
        _ => None,
    }
}

fn find_context_in_value(val: &ast::NodeedValue, pos: Position) -> Option<String> {
    match &val.value {
        ast::Value::Block(entries) => {
            for entry in entries {
                if let Some(ctx) = find_context_in_entry(entry, pos) {
                    return Some(ctx);
                }
            }
            None
        }
        ast::Value::TaggedBlock(_, entries, _) => {
            for entry in entries {
                if let Some(ctx) = find_context_in_entry(entry, pos) {
                    return Some(ctx);
                }
            }
            None
        }
        _ => None,
    }
}
