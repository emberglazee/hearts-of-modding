use crate::parser::ast;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::{Scope, ScopeStack};
use tower_lsp_server::ls_types::Diagnostic;

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
) {
    let mut scope_stack = ScopeStack::new(initial_scope);
    walk_entries(entries, visitors, rules, ctx, diags, &mut scope_stack);

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
) {
    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                let mut pushed_scope = false;

                // Structural blocks that push scope (mirrors Backend::check_entry_semantic)
                let mut s = Scope::from_str(ass.key_text(ctx.source));

                // Internal 'idea' definition block context
                if s == Scope::Unknown {
                    let stack = scope_stack.stack();
                    if stack.contains(&Scope::Idea) {
                        if stack.len() == 2 || stack.len() == 3 {
                            s = Scope::Idea;
                        }
                    }
                }

                if s != Scope::Unknown
                    || ass.key_text(ctx.source).contains(':')
                    || ass.key_text(ctx.source).contains('.')
                {
                    match &ass.value.value {
                        ast::Value::Block(_) | ast::Value::TaggedBlock(_, _, _) => {
                            scope_stack.push(s);
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
                        crate::backend::check_duplicate_keys(
                            inner,
                            diags,
                            ctx.modifier_mappings,
                            ctx.source,
                        );
                        walk_entries(inner, visitors, rules, ctx, diags, scope_stack);
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
                    );
                    walk_entries(inner, visitors, rules, ctx, diags, scope_stack);
                }
                _ => {}
            },
            ast::Entry::Comment(_, _) => {}
        }
    }
}
