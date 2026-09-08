use crate::parser::ast;
use crate::rules::visitor::AstVisitor;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scanner::country_scanner::is_valid_tag;
use crate::scope::scope::ScopeStack;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates ideology and sub-ideology references.
///
/// Checks keys like `ideology` and `has_ideology` against known ideologies
/// and sub-ideologies from the scanner, allowing scope references (ROOT,
/// FROM, etc.) and variable references (var:...) to pass through.
pub(crate) struct IdeologyRule;

impl ValidationRule for IdeologyRule {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        _pushed_scope: bool,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key_lower = ass.key_text(ctx.source).to_ascii_lowercase();
        if key_lower != "ideology" && key_lower != "has_ideology" {
            return;
        }

        let Some(val) = ass.value.value.as_str(ctx.source) else {
            return;
        };

        if !ctx.ideologies.contains_key(val)
            && !ctx.sub_ideologies.contains_key(val)
            && !is_dynamic_ideology_ref(val)
        {
            diags.push(Diagnostic {
                range: ctx.range(&ass.value.range),
                severity: Some(DiagnosticSeverity::WARNING),
                message: format!("Unknown ideology: '{}'", val),
                code: Some(NumberOrString::String(
                    crate::validation::advanced_validation::UNKNOWN_TRIGGER.to_string(),
                )),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        }
    }
}

/// True when `val` is a dynamic ideology reference rather than a literal
/// ideology name: a scope reference (ROOT, FROM, ...), a variable reference
/// (`var:...`), or a country tag (`ideology = GER` means "that country's
/// current ideology").
pub(crate) fn is_dynamic_ideology_ref(val: &str) -> bool {
    // Allow scope references (ROOT, FROM, PREV, THIS, etc.)
    let is_scope_ref = matches!(
        val.to_uppercase().as_str(),
        "ROOT"
            | "FROM"
            | "PREV"
            | "THIS"
            | "PREVPREV"
            | "PREVPREVPREV"
            | "PREVPREVPREVPREV"
            | "OWNER"
            | "CONTROLLER"
            | "CAPITAL"
            | "FROM.FROM"
            | "FROM.FROM.FROM"
    );
    // Allow variable references (var:SCOPE@name or var:name)
    let is_var_ref = val.starts_with("var:");
    // Allow 3-letter country tags as scope references for add_popularity etc.:
    //   ideology = GER  → use Germany's current ideology
    let is_country_tag = is_valid_tag(val);

    is_scope_ref || is_var_ref || is_country_tag
}

/// True when `val` names an ideology group (parent ideology), matched
/// case-insensitively — engine identifiers are case-insensitive.
fn is_ideology_group(ctx: &ValidationContext, val: &str) -> bool {
    ctx.ideologies.contains_key(val)
        || ctx
            .ideologies
            .iter()
            .any(|e| e.key().eq_ignore_ascii_case(val))
}

/// When `val` names a sub-ideology, returns its parent group name.
/// Case-insensitive, mirroring [`is_ideology_group`].
fn sub_ideology_parent(ctx: &ValidationContext, val: &str) -> Option<String> {
    ctx.sub_ideologies
        .get(val)
        .map(|lv| lv.resolve().0.to_string())
        .or_else(|| {
            ctx.sub_ideologies.iter().find_map(|e| {
                if e.key().eq_ignore_ascii_case(val) {
                    Some(e.value().resolve().0.to_string())
                } else {
                    None
                }
            })
        })
}

/// Pushes a HOM3023 diagnostic for `token` in a group-only `slot`.
fn push_group_diagnostic(
    ctx: &ValidationContext,
    diags: &mut Vec<Diagnostic>,
    range: &ast::Range,
    token: &str,
    slot: &str,
) {
    let (message, data) = match sub_ideology_parent(ctx, token) {
        Some(parent) => (
            format!(
                "'{}' is a sub-ideology of '{}' — {} only accepts ideology groups",
                token, parent, slot
            ),
            Some(serde_json::Value::String(parent)),
        ),
        None => (
            format!(
                "Unknown ideology group: '{}' — {} only accepts ideology groups",
                token, slot
            ),
            None,
        ),
    };
    diags.push(Diagnostic {
        range: ctx.range(range),
        severity: Some(DiagnosticSeverity::WARNING),
        message,
        code: Some(NumberOrString::String(
            crate::validation::advanced_validation::INVALID_RULING_PARTY.to_string(),
        )),
        source: Some("Hearts of Modding".to_string()),
        data,
        ..Default::default()
    });
}

/// Validates slots that only accept ideology *groups* (parent ideologies),
/// never sub-ideologies:
///
/// - `ruling_party` inside `set_politics = { ... }`
/// - child keys of `set_popularities = { ... }` (`democratic = 50`)
/// - `ideology` inside `add_popularity = { ... }`
/// - `has_government = ...` and `has_ideology_group = ...` values (anywhere;
///   the wiki pins both to ideology groups explicitly)
///
/// Vanilla evidence (2345 files): every literal in these slots names a parent
/// group; sub-ideologies never appear. Deliberately NOT covered:
/// `start_civil_war.ideology` accepts sub-ideologies (23× `fascism_ideology`
/// etc.), `start_civil_war.ruling_party` is the revolt leader tag, and the
/// generic `ideology`/`has_ideology` keys stay lenient (the latter is
/// sub-only per wiki, but tightening it is a separate change).
pub(crate) struct IdeologyVisitor {
    /// Nesting depths inside the tracked container blocks.
    set_politics_depth: u32,
    set_popularities_depth: u32,
    add_popularity_depth: u32,
}

impl IdeologyVisitor {
    pub fn new() -> Self {
        Self {
            set_politics_depth: 0,
            set_popularities_depth: 0,
            add_popularity_depth: 0,
        }
    }

    pub fn visitor() -> Box<dyn AstVisitor> {
        Box::new(Self::new())
    }

    /// Validates a scalar group-only value (dynamics pass through).
    fn check_group_value(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        slot: &str,
        diags: &mut Vec<Diagnostic>,
    ) {
        let Some(val) = ass.value.value.as_str(ctx.source) else {
            return;
        };
        if is_dynamic_ideology_ref(val) || is_ideology_group(ctx, val) {
            return;
        }
        push_group_diagnostic(ctx, diags, &ass.value.range, val, slot);
    }
}

impl AstVisitor for IdeologyVisitor {
    fn enter_assignment(
        &mut self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key = ass.key_text(ctx.source);
        let is_block = matches!(
            &ass.value.value,
            ast::Value::Block(_) | ast::Value::TaggedBlock(..)
        );

        // ── Container tracking ──
        if is_block {
            if key.eq_ignore_ascii_case("set_politics") {
                self.set_politics_depth += 1;
                return;
            }
            if key.eq_ignore_ascii_case("set_popularities") {
                self.set_popularities_depth += 1;
                return;
            }
            if key.eq_ignore_ascii_case("add_popularity") {
                self.add_popularity_depth += 1;
                return;
            }
        }

        // ── set_popularities child keys (`democratic = 50`) ──
        // Per schema these children are int/variable scalars, never nested
        // blocks, so any assignment seen at depth > 0 is a popularity entry.
        if self.set_popularities_depth > 0 {
            if !is_ideology_group(ctx, key) {
                push_group_diagnostic(ctx, diags, &ass.key_range, key, "set_popularities");
            }
            return;
        }

        // ── Scalar value slots ──
        if key.eq_ignore_ascii_case("ruling_party") && self.set_politics_depth > 0 {
            self.check_group_value(ass, ctx, "set_politics.ruling_party", diags);
        } else if key.eq_ignore_ascii_case("ideology") && self.add_popularity_depth > 0 {
            self.check_group_value(ass, ctx, "add_popularity.ideology", diags);
        } else if key.eq_ignore_ascii_case("has_government") {
            self.check_group_value(ass, ctx, "has_government", diags);
        } else if key.eq_ignore_ascii_case("has_ideology_group") {
            self.check_group_value(ass, ctx, "has_ideology_group", diags);
        }
    }

    fn exit_assignment(
        &mut self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        _diags: &mut Vec<Diagnostic>,
    ) {
        if !matches!(
            &ass.value.value,
            ast::Value::Block(_) | ast::Value::TaggedBlock(..)
        ) {
            return;
        }
        let key = ass.key_text(ctx.source);
        if key.eq_ignore_ascii_case("set_politics") {
            self.set_politics_depth = self.set_politics_depth.saturating_sub(1);
        } else if key.eq_ignore_ascii_case("set_popularities") {
            self.set_popularities_depth = self.set_popularities_depth.saturating_sub(1);
        } else if key.eq_ignore_ascii_case("add_popularity") {
            self.add_popularity_depth = self.add_popularity_depth.saturating_sub(1);
        }
    }
}
