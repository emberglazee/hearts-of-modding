use crate::parser::ast;
use crate::scope::scope;
use crate::utils::lsp_convert::is_pos_in_range;
use tower_lsp_server::ls_types::Position;

pub fn find_scope_context_at(
    script: &ast::Script,
    pos: Position,
    initial_scope: scope::Scope,
    sctx: &scope::ScopeCtx,
) -> (Option<String>, Vec<scope::Scope>) {
    let pos = crate::utils::lsp_convert::to_byte_position(&script.source, pos);
    let mut scope_stack = scope::ScopeStack::new(initial_scope);
    let mut context = None;
    for entry in &script.entries {
        if let Some(ctx) = find_scope_context_in_entry(entry, pos, &mut scope_stack, sctx, script) {
            context = Some(ctx);
            break;
        }
    }
    (
        context,
        scope_stack
            .nodes()
            .iter()
            .filter(|n| !n.is_transparent)
            .map(|n| n.scope_type)
            .collect(),
    )
}

fn find_scope_context_in_entry(
    entry: &ast::Entry,
    pos: Position,
    scope_stack: &mut scope::ScopeStack,
    sctx: &scope::ScopeCtx,
    script: &ast::Script,
) -> Option<String> {
    match entry {
        ast::Entry::Assignment(ass) => {
            if is_pos_in_range(pos, &ass.value.range) {
                if let ast::Value::Block(_) | ast::Value::TaggedBlock(_, _, _) = &ass.value.value {
                    let key = ass.key_text(&script.source);
                    // Unified resolution — the SAME path the validation walker
                    // uses (file-type initial scope + full ScopeCtx maps), so
                    // hover/completion never diverge from HOM004.
                    let (mut s, is_transparent) = scope_stack.resolve_entry_scope(key, sctx);
                    if s == scope::Scope::Unknown {
                        let nodes = scope_stack.nodes();
                        if nodes.iter().any(|n| n.scope_type == scope::Scope::Idea)
                            && (nodes.len() == 2 || nodes.len() == 3)
                            && !crate::rules::visitor::is_idea_structure_key(key)
                        {
                            s = scope::Scope::Idea;
                        }
                    }
                    if s != scope::Scope::Unknown || key.contains(':') || key.contains('.') {
                        scope_stack.push_with(s, is_transparent);
                    }
                }

                if let Some(inner) =
                    find_scope_context_in_value(&ass.value, pos, scope_stack, sctx, script)
                {
                    return Some(inner);
                }

                return Some(ass.key_text(&script.source).to_string());
            }
            None
        }
        ast::Entry::Value(val) => find_scope_context_in_value(val, pos, scope_stack, sctx, script),
        _ => None,
    }
}

fn find_scope_context_in_value(
    val: &ast::NodeedValue,
    pos: Position,
    scope_stack: &mut scope::ScopeStack,
    sctx: &scope::ScopeCtx,
    script: &ast::Script,
) -> Option<String> {
    match &val.value {
        ast::Value::Block(entries) => {
            for entry in entries {
                if let Some(ctx) =
                    find_scope_context_in_entry(entry, pos, scope_stack, sctx, script)
                {
                    return Some(ctx);
                }
            }
            None
        }
        ast::Value::TaggedBlock(_, entries, _) => {
            for entry in entries {
                if let Some(ctx) =
                    find_scope_context_in_entry(entry, pos, scope_stack, sctx, script)
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
    let pos = crate::utils::lsp_convert::to_byte_position(&script.source, pos);
    for entry in &script.entries {
        if let Some(ctx) = find_context_in_entry(entry, pos, &script.source) {
            return Some(ctx);
        }
    }
    None
}

/// Find the key of the innermost block whose value range contains `pos`.
///
/// Used for parent-aware completions: inside `add_timed_idea = { ... }` this
/// returns `add_timed_idea`, so the caller can offer that block's documented
/// parameters instead of the whole trigger/effect list. Nested blocks win
/// (a cursor inside `limit = { ... }` returns `limit`); plain values return
/// `None`. Client position is UTF-16, AST ranges are byte columns — convert
/// at entry like the sibling walkers.
/// Full chain of enclosing block keys, innermost first. Used by completion to
/// inherit a param-container ancestor's parameter table (`technology_folders`
/// documents `ledger`/`doctrine` for its per-folder sub-blocks). Callers that
/// only want the innermost block take element 0.
pub fn find_enclosing_block_key_chain(script: &ast::Script, pos: Position) -> Vec<String> {
    let pos = crate::utils::lsp_convert::to_byte_position(&script.source, pos);
    let mut chain = Vec::new();
    collect_enclosing_chain(&script.entries, pos, &script.source, &mut chain);
    // collect pushes outermost first; callers want innermost first.
    chain.reverse();
    chain
}

/// Pushes enclosing block keys onto `chain` (outermost pushed first). Returns
/// true when `pos` was enclosed somewhere in these entries.
fn collect_enclosing_chain(
    entries: &[ast::Entry],
    pos: Position,
    source: &str,
    chain: &mut Vec<String>,
) -> bool {
    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                if is_pos_in_range(pos, &ass.value.range) {
                    match &ass.value.value {
                        ast::Value::Block(inner) | ast::Value::TaggedBlock(_, inner, _) => {
                            chain.push(ass.key_text(source).to_string());
                            if collect_enclosing_chain(inner, pos, source, chain) {
                                return true;
                            }
                            // Cursor sits in this block but no deeper block
                            // encloses it — this key ends the chain.
                            return true;
                        }
                        _ => continue,
                    }
                }
            }
            ast::Entry::Value(val) => {
                if let ast::Value::Block(inner) | ast::Value::TaggedBlock(_, inner, _) = &val.value
                {
                    if collect_enclosing_chain(inner, pos, source, chain) {
                        return true;
                    }
                }
            }
            ast::Entry::Comment(_, _) => {}
        }
    }
    false
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parser::parse_script;
    use tower_lsp_server::ls_types::Position;

    fn pos(line: u32, character: u32) -> Position {
        Position { line, character }
    }

    #[test]
    fn test_find_enclosing_block_key() {
        let (script, _) = parse_script(
            "add_timed_idea = {\n    idea = SPE_x\n    days = 180\n}\nadd_political_power = 100\n",
        );

        // Inside the add_timed_idea block (empty line / after a child) -> the
        // block key itself. UTF-16 columns -> byte conversion handled at entry.
        assert_eq!(
            find_enclosing_block_key_chain(&script, pos(1, 4))
                .first()
                .map(String::as_str),
            Some("add_timed_idea")
        );
        // Cursor on a child key line (days = 180) -> still the parent block.
        assert_eq!(
            find_enclosing_block_key_chain(&script, pos(2, 4))
                .first()
                .map(String::as_str),
            Some("add_timed_idea")
        );
        // Top-level non-block value -> empty chain.
        assert!(find_enclosing_block_key_chain(&script, pos(4, 0)).is_empty());
    }

    #[test]
    fn test_find_enclosing_block_key_nested() {
        let (script, _) = parse_script(
            "random_other_country = {\n    limit = {\n        has_stability = 0.5\n    }\n}\n",
        );
        // Cursor inside the limit block -> limit innermost, outer key behind
        // it (innermost wins for callers taking element 0).
        assert_eq!(
            find_enclosing_block_key_chain(&script, pos(2, 8))
                .first()
                .map(String::as_str),
            Some("limit")
        );
        // Cursor in the outer block body -> the outer block key.
        assert_eq!(
            find_enclosing_block_key_chain(&script, pos(1, 4))
                .first()
                .map(String::as_str),
            Some("random_other_country")
        );
    }

    #[test]
    fn test_find_enclosing_block_key_tagged() {
        let (script, _) = parse_script("set_variable = {\n    temp:foo = 1\n}\n");
        assert_eq!(
            find_enclosing_block_key_chain(&script, pos(1, 4))
                .first()
                .map(String::as_str),
            Some("set_variable")
        );
    }

    #[test]
    fn test_find_enclosing_block_key_chain_full_order() {
        let (script, _) = parse_script(
            "technology_folders = {\n    land_doctrine_folder = {\n        doctrine = yes\n    }\n}\n",
        );
        // Cursor on `doctrine` -> full chain innermost-first: the folder
        // instance, then the param-container above it. This is what lets
        // completion inherit technology_folders' parameter table.
        let chain = find_enclosing_block_key_chain(&script, pos(2, 8));
        assert_eq!(
            chain,
            vec![
                "land_doctrine_folder".to_string(),
                "technology_folders".to_string()
            ]
        );
    }
}
