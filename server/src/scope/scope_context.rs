use crate::data::interner::InternedStr;
use crate::data::layered_value::LayeredValue;
use crate::parser::ast;
use crate::scanner::achievement_scanner;
use crate::scope::scope;
use crate::utils::lsp_convert::is_pos_in_range;
use dashmap::DashMap;
use tower_lsp_server::ls_types::Position;

pub fn find_scope_context_at(
    script: &ast::Script,
    pos: Position,
    achievements: &DashMap<InternedStr, LayeredValue<achievement_scanner::Achievement>>,
) -> (Option<String>, Vec<scope::Scope>) {
    let mut scope_stack = scope::ScopeStack::new(scope::Scope::Global);
    let mut context = None;
    for entry in &script.entries {
        if let Some(ctx) =
            find_scope_context_in_entry(entry, pos, &mut scope_stack, achievements, script)
        {
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
    achievements: &DashMap<InternedStr, LayeredValue<achievement_scanner::Achievement>>,
    script: &ast::Script,
) -> Option<String> {
    match entry {
        ast::Entry::Assignment(ass) => {
            if is_pos_in_range(pos, &ass.value.range) {
                if let ast::Value::Block(_) | ast::Value::TaggedBlock(_, _, _) = &ass.value.value {
                    let key = ass.key_text(&script.source);
                    // Try dynamic meta-scope resolution for THIS/ROOT/PREV/FROM
                    // before falling back to static resolution.
                    let s = scope_stack.resolve_scope_key(key, achievements);
                    scope_stack.push(s);
                }

                if let Some(inner) =
                    find_scope_context_in_value(&ass.value, pos, scope_stack, achievements, script)
                {
                    return Some(inner);
                }

                return Some(ass.key_text(&script.source).to_string());
            }
            None
        }
        ast::Entry::Value(val) => {
            find_scope_context_in_value(val, pos, scope_stack, achievements, script)
        }
        _ => None,
    }
}

fn find_scope_context_in_value(
    val: &ast::NodeedValue,
    pos: Position,
    scope_stack: &mut scope::ScopeStack,
    achievements: &DashMap<InternedStr, LayeredValue<achievement_scanner::Achievement>>,
    script: &ast::Script,
) -> Option<String> {
    match &val.value {
        ast::Value::Block(entries) => {
            for entry in entries {
                if let Some(ctx) =
                    find_scope_context_in_entry(entry, pos, scope_stack, achievements, script)
                {
                    return Some(ctx);
                }
            }
            None
        }
        ast::Value::TaggedBlock(_, entries, _) => {
            for entry in entries {
                if let Some(ctx) =
                    find_scope_context_in_entry(entry, pos, scope_stack, achievements, script)
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
        if let Some(ctx) = find_context_in_entry(entry, pos, &script.source) {
            return Some(ctx);
        }
    }
    None
}

fn find_context_in_entry(entry: &ast::Entry, pos: Position, source: &str) -> Option<String> {
    match entry {
        ast::Entry::Assignment(ass) => {
            if is_pos_in_range(pos, &ass.value.range) {
                if let Some(inner) = find_context_in_value(&ass.value, pos, source) {
                    return Some(inner);
                }
                return Some(ass.key_text(source).to_string());
            }
            None
        }
        ast::Entry::Value(val) => find_context_in_value(val, pos, source),
        _ => None,
    }
}

fn find_context_in_value(val: &ast::NodeedValue, pos: Position, source: &str) -> Option<String> {
    match &val.value {
        ast::Value::Block(entries) => {
            for entry in entries {
                if let Some(ctx) = find_context_in_entry(entry, pos, source) {
                    return Some(ctx);
                }
            }
            None
        }
        ast::Value::TaggedBlock(_, entries, _) => {
            for entry in entries {
                if let Some(ctx) = find_context_in_entry(entry, pos, source) {
                    return Some(ctx);
                }
            }
            None
        }
        _ => None,
    }
}
