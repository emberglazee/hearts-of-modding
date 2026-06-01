use crate::achievement_scanner;
use crate::ast;
use crate::interner::InternedStr;
use crate::lsp_convert::is_pos_in_range;
use crate::scope;
use dashmap::DashMap;
use tower_lsp_server::ls_types::Position;

pub fn find_identifier_at(
    script: &ast::Script,
    pos: Position,
    scope_stack: &mut scope::ScopeStack,
    achievements: &DashMap<InternedStr, achievement_scanner::Achievement>,
) -> Option<(
    String,
    Vec<scope::Scope>,
    Option<ast::Value>,
    Option<String>,
)> {
    for entry in &script.entries {
        if let Some(res) = find_in_entry(entry, pos, scope_stack, achievements, None) {
            return Some(res);
        }
    }
    None
}

pub fn find_in_entry(
    entry: &ast::Entry,
    pos: Position,
    scope_stack: &mut scope::ScopeStack,
    achievements: &DashMap<InternedStr, achievement_scanner::Achievement>,
    context_key: Option<String>,
) -> Option<(
    String,
    Vec<scope::Scope>,
    Option<ast::Value>,
    Option<String>,
)> {
    match entry {
        ast::Entry::Assignment(ass) => {
            if is_pos_in_range(pos, &ass.key_range) {
                return Some((
                    ass.key.clone(),
                    scope_stack.iter().cloned().collect(),
                    Some(ass.value.value.clone()),
                    context_key,
                ));
            }

            let mut pushed_scope = None;
            if let ast::Value::Block(_) | ast::Value::TaggedBlock(_, _, _) = &ass.value.value {
                let s = scope::resolve_key_scope(&ass.key, achievements);

                if s != scope::Scope::Unknown || ass.key.contains(':') || ass.key.contains('.') {
                    scope_stack.push(s);
                    pushed_scope = Some(s);
                }
            }

            let mut res = find_in_value(
                &ass.value,
                pos,
                scope_stack,
                achievements,
                Some(ass.key.clone()),
            );

            if let Some((ref mut id, _, ref mut val_opt, _)) = res {
                if let ast::Value::Number(_) | ast::Value::Boolean(_) = &ass.value.value {
                    *id = ass.key.clone();
                    *val_opt = Some(ass.value.value.clone());
                }
            }

            if pushed_scope.is_some() {
                scope_stack.pop();
            }
            res
        }
        ast::Entry::Value(val) => find_in_value(val, pos, scope_stack, achievements, context_key),
        _ => None,
    }
}

pub fn find_in_value(
    val: &ast::NodeedValue,
    pos: Position,
    scope_stack: &mut scope::ScopeStack,
    achievements: &DashMap<InternedStr, achievement_scanner::Achievement>,
    context_key: Option<String>,
) -> Option<(
    String,
    Vec<scope::Scope>,
    Option<ast::Value>,
    Option<String>,
)> {
    match &val.value {
        ast::Value::String(s) => {
            if is_pos_in_range(pos, &val.range) {
                if pos.line == val.range.start_line {
                    let char_offset = pos.character.saturating_sub(val.range.start_col);
                    let is_quoted = val.range.end_col - val.range.start_col > s.len() as u32;
                    let adj_offset = if is_quoted {
                        char_offset.saturating_sub(1)
                    } else {
                        char_offset
                    } as usize;

                    let mut start_search = 0;
                    while let Some(open) = s[start_search..].find('[') {
                        let abs_open = start_search + open;
                        if let Some(close) = s[abs_open..].find(']') {
                            let abs_close = abs_open + close;
                            if adj_offset > abs_open && adj_offset <= abs_close {
                                let inner = &s[abs_open + 1..abs_close];
                                let mut current_part_start = 0;
                                for part in inner.split('.') {
                                    let part_abs_start = abs_open + 1 + current_part_start;
                                    let part_abs_end = part_abs_start + part.len();
                                    if adj_offset >= part_abs_start && adj_offset < part_abs_end {
                                        return Some((
                                            part.to_string(),
                                            scope_stack.iter().cloned().collect(),
                                            None,
                                            context_key,
                                        ));
                                    }
                                    current_part_start += part.len() + 1;
                                }
                                return Some((
                                    inner.to_string(),
                                    scope_stack.iter().cloned().collect(),
                                    None,
                                    context_key,
                                ));
                            }
                            start_search = abs_close + 1;
                        } else {
                            break;
                        }
                    }
                }
                return Some((
                    s.clone(),
                    scope_stack.iter().cloned().collect(),
                    None,
                    context_key,
                ));
            }
            None
        }
        ast::Value::Number(n) => {
            if is_pos_in_range(pos, &val.range) {
                return Some((
                    n.to_string(),
                    scope_stack.iter().cloned().collect(),
                    Some(ast::Value::Number(*n)),
                    context_key,
                ));
            }
            None
        }
        ast::Value::Boolean(b) => {
            if is_pos_in_range(pos, &val.range) {
                return Some((
                    (if *b { "yes" } else { "no" }).to_string(),
                    scope_stack.iter().cloned().collect(),
                    Some(ast::Value::Boolean(*b)),
                    context_key,
                ));
            }
            None
        }
        ast::Value::Block(entries) => {
            for entry in entries {
                if let Some(res) =
                    find_in_entry(entry, pos, scope_stack, achievements, context_key.clone())
                {
                    return Some(res);
                }
            }
            None
        }
        ast::Value::TaggedBlock(_, entries, _) => {
            for entry in entries {
                if let Some(res) =
                    find_in_entry(entry, pos, scope_stack, achievements, context_key.clone())
                {
                    return Some(res);
                }
            }
            None
        }
    }
}
