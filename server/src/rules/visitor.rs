use crate::parser::ast;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::{Scope, ScopeCtx, ScopeStack};
use tower_lsp_server::ls_types::Diagnostic;

/// Idea-adjacent keywords that should never be treated as an idea definition.
///
/// This is the SINGLE source of truth for idea structure keys. It is consumed by:
/// 1. The scope-promotion logic below (`walk_script`) — these keys must not be
///    promoted to `Scope::Idea`.
/// 2. `rules::ideas::IdeaRule` — these keys must not be flagged as ideas
///    missing a `picture` field.
/// 3. `rules::ideas` tests, which assert membership against this same array.
///
/// It lives in `rules::visitor` (not `rules::ideas`) so the walker can use it
/// without a dependency back into `rules::ideas`.
pub(crate) const IDEA_STRUCTURE_KEYS: &[&str] = &[
    // Category structure & attributes
    "ideas",
    "hidden_ideas",
    "designer",
    "law",
    "use_list_view",
    "slot_ledgers",
    "slot",
    "character_slot",
    // Idea sub-block properties
    "picture",
    "targeted_modifier",
    "research_bonus",
    "equipment_bonus",
    "rule",
    "traits",
    "on_add",
    "on_remove",
    "cancel",
    "allowed",
    "available",
    "allowed_civil_war",
    "do_effect",
    "visible",
    "allowed_to_remove",
    "removal_cost",
    "cost",
    "level",
    "ledger",
    "hidden",
    "politics_tab",
];

/// True when `key` is one of [`IDEA_STRUCTURE_KEYS`] (case-insensitive).
pub(crate) fn is_idea_structure_key(key: &str) -> bool {
    IDEA_STRUCTURE_KEYS.contains(&key.to_ascii_lowercase().as_str())
}

/// A visitor that receives AST events during a single centralized traversal.
///
/// Implement this trait instead of `ValidationRule::check_block` when your
/// rule needs to react to individual assignments during the AST walk.
/// The `walk_script` function handles scope management and traversal so
/// each rule no longer needs its own recursive AST walk.
pub trait AstVisitor {
    /// Called before processing an assignment and its child entries.
    ///
    /// At this point, the scope has already been pushed for structural
    /// blocks (e.g. `state = { ... }` pushes `Scope::State`), so the
    /// rule sees the correct current scope.
    fn enter_assignment(
        &mut self,
        _ass: &ast::Assignment,
        _ctx: &ValidationContext,
        _scope: &ScopeStack,
        _diags: &mut Vec<Diagnostic>,
    ) {
    }

    /// Called after processing an assignment and all its child entries.
    ///
    /// The scope is still pushed at this point (it will be popped after
    /// all exit hooks return), so rules can inspect the scope one last
    /// time before leaving the block.
    fn exit_assignment(
        &mut self,
        _ass: &ast::Assignment,
        _ctx: &ValidationContext,
        _scope: &ScopeStack,
        _diags: &mut Vec<Diagnostic>,
    ) {
    }

    /// Called once after the full walk completes.
    ///
    /// Use this for cross-reference checks that need data accumulated
    /// across the entire AST (e.g. victory point province validation).
    fn after_walk(&mut self, _ctx: &ValidationContext, _diags: &mut Vec<Diagnostic>) {}
}

/// Walk the AST exactly once, calling visitor hooks and rule callbacks.
///
/// This replaces the per-rule recursive AST traversal pattern. The walk:
///
/// 1. Maintains a `ScopeStack` updated as it enters/exits structural blocks
/// 2. Calls `visitor.enter_assignment` for each visitor
/// 3. Calls `rule.check_assignment` for each ValidationRule
/// 4. Recurses into child blocks
/// 5. Calls `visitor.exit_assignment` for each visitor
/// 6. Calls `visitor.after_walk` for each visitor (post-traversal)
///
/// # Performance
///
/// All visitors AND rules share one traversal. The old pattern had rules
/// doing their own independent `check_block` recursion — with 20 rules
/// that meant 20 AST walks. Now it's exactly 1.
///
/// # For rules that still need `check_block`
///
/// `check_block` is NOT called here. The backend calls it separately on
/// the top-level entries (no recursion) for cross-entry analysis like
/// country tag ratio checks.
pub fn walk_script(
    entries: &[ast::Entry],
    visitors: &mut [Box<dyn AstVisitor>],
    rules: &[Box<dyn ValidationRule>],
    ctx: &ValidationContext,
    diags: &mut Vec<Diagnostic>,
    initial_scope: Scope,
    in_air_wings: bool,
) {
    let mut scope_stack = ScopeStack::new(initial_scope);
    walk_entries(
        entries,
        visitors,
        rules,
        ctx,
        diags,
        &mut scope_stack,
        in_air_wings,
        false,
        false,
    );

    // Post-walk hook for cross-reference checks
    for visitor in visitors.iter_mut() {
        visitor.after_walk(ctx, diags);
    }
}

fn walk_entries(
    entries: &[ast::Entry],
    visitors: &mut [Box<dyn AstVisitor>],
    rules: &[Box<dyn ValidationRule>],
    ctx: &ValidationContext,
    diags: &mut Vec<Diagnostic>,
    scope_stack: &mut ScopeStack,
    in_air_wings: bool,
    in_random_list: bool,
    in_state_targeted_decision: bool,
) {
    // Scan the current block's entries for state_target = yes to detect
    // state-targeted decisions. Inside those, FROM refers to the target state.
    let has_state_target = entries.iter().any(|e| {
        if let ast::Entry::Assignment(ass) = e {
            ass.key_text(ctx.source) == "state_target"
                && matches!(&ass.value.value, ast::Value::Boolean(true))
        } else {
            false
        }
    });
    let state_targeted = in_state_targeted_decision || has_state_target;

    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                let mut pushed_scope = false;

                let key_text = ass.key_text(ctx.source);
                let (mut s, is_transparent) = scope_stack.resolve_entry_scope(
                    key_text,
                    &ScopeCtx {
                        uri: ctx.uri,
                        // Pass the real maps directly; ScopeCtx fields are
                        // Option so no empty DashMap needs allocating here.
                        event_targets: Some(ctx.event_targets),
                        characters: Some(ctx.characters),
                        achievements: None,
                        in_random_list,
                        state_targeted,
                    },
                );

                // Idea promotion: Unknown keys with block values at depth 2-3
                // inside an Idea scope are likely idea names (e.g. `my_idea = { }`
                // inside `country = { ... }`). EXCLUDE known idea structure/sub-block
                // keywords — those are never valid idea names.
                if s == Scope::Unknown {
                    // Inspect the scope stack without allocating a Vec (nodes() is
                    // a slice; stack() materialised a copy on every unknown key).
                    let nodes = scope_stack.nodes();
                    if nodes.iter().any(|n| n.scope_type == Scope::Idea)
                        && (nodes.len() == 2 || nodes.len() == 3)
                        && !is_idea_structure_key(key_text)
                    {
                        s = Scope::Idea;
                    }
                }

                if s != Scope::Unknown || key_text.contains(':') || key_text.contains('.') {
                    match &ass.value.value {
                        ast::Value::Block(_) | ast::Value::TaggedBlock(_, _, _) => {
                            scope_stack.push_with(s, is_transparent);
                            pushed_scope = true;
                        }
                        _ => {}
                    }
                }

                // 1) Visitor hooks (for rules that migrated to AstVisitor)
                for visitor in visitors.iter_mut() {
                    visitor.enter_assignment(ass, ctx, scope_stack, diags);
                }

                // 2) check_assignment for rules that use the traditional pattern
                for rule in rules {
                    rule.check_assignment(ass, ctx, scope_stack, pushed_scope, diags);
                }

                // 3) Check for duplicate keys, then recurse into children
                match &ass.value.value {
                    ast::Value::Block(inner) | ast::Value::TaggedBlock(_, inner, _) => {
                        let key = ass.key_text(ctx.source);
                        let new_in_air_wings = in_air_wings || key == "air_wings";
                        let new_in_random_list = key == "random_list";
                        crate::backend::check_duplicate_keys(
                            inner,
                            diags,
                            ctx.modifier_mappings,
                            ctx.source,
                            ctx.range_mapper,
                            new_in_air_wings,
                            Some(key),
                        );
                        walk_entries(
                            inner,
                            visitors,
                            rules,
                            ctx,
                            diags,
                            scope_stack,
                            new_in_air_wings,
                            new_in_random_list,
                            state_targeted,
                        );
                    }
                    _ => {}
                }

                // 4) Visitor exit hooks
                for visitor in visitors.iter_mut() {
                    visitor.exit_assignment(ass, ctx, scope_stack, diags);
                }

                if pushed_scope {
                    scope_stack.pop();
                }
            }
            ast::Entry::Value(val) => match &val.value {
                ast::Value::Block(inner) | ast::Value::TaggedBlock(_, inner, _) => {
                    crate::backend::check_duplicate_keys(
                        inner,
                        diags,
                        ctx.modifier_mappings,
                        ctx.source,
                        ctx.range_mapper,
                        in_air_wings,
                        None,
                    );
                    walk_entries(
                        inner,
                        visitors,
                        rules,
                        ctx,
                        diags,
                        scope_stack,
                        in_air_wings,
                        in_random_list,
                        state_targeted,
                    );
                }
                _ => {}
            },
            ast::Entry::Comment(_, _) => {}
        }
    }
}
