use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scanner::variable_scanner::{EventTarget, Variable};
use crate::scope::scope::{Scope, ScopeStack};
use dashmap::DashMap;
use std::cell::RefCell;
use std::collections::HashMap;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Variable reference kinds for diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VarRefKind {
    /// `set_variable`, `set_temp_variable`, `change_variable`, etc. (definition)
    Definition,
    /// `check_variable`, `has_variable` (read/comparison)
    Read,
    /// `add_to_variable`, `subtract_from_variable`, `multiply_variable`, etc. (mutation)
    Mutation,
    /// `set_global_flag`, `has_country_flag`, etc. (flag operations)
    // Scaffolding: flag ops currently classify as Definition/Read; this
    // dedicated kind is reserved for finer-grained flag diagnostics.
    #[allow(dead_code)]
    Flag,
    /// `add_to_array`, `is_in_array`, `for_each_scope_loop`, etc. (array operations)
    Array,
    /// `set_temp_variable`, `add_to_temp_variable` (temp - chain local)
    Temp,
}

/// Variable reference info for validation
#[derive(Debug, Clone)]
struct VarRef {
    name: String,
    kind: VarRefKind,
    is_temp: bool,
    is_global: bool,
    range: ast::Range,
    scope: Scope,
}

/// Variable validation rule (HOM9xxx range)
///
/// Validates:
/// - Variable definitions track their scope
/// - Variable reads/mutations resolve to a definition in the same or accessible scope
/// - Temp variables only valid within current effect chain
/// - Global variables (`global.name`) work everywhere
/// - Array operations validate array definitions
pub(crate) struct VariableRule {
    /// Variable definitions found in this file, keyed by name -> (scope, is_temp, is_global, range)
    file_vars: HashMap<String, (Scope, bool, bool, ast::Range)>,
    /// Cross-file variable definitions from scanner data (DashMap from backend)
    workspace_vars: DashMap<InternedStr, Vec<Variable>>,
    /// Event targets for scope resolution
    #[allow(dead_code)] // scaffolding for upcoming event-target variable validation
    event_targets: DashMap<InternedStr, Vec<EventTarget>>,
}

impl VariableRule {
    pub(crate) fn new(
        workspace_vars: &DashMap<InternedStr, Vec<Variable>>,
        event_targets: &DashMap<InternedStr, Vec<EventTarget>>,
    ) -> Self {
        Self {
            file_vars: HashMap::new(),
            workspace_vars: workspace_vars.clone(),
            event_targets: event_targets.clone(),
        }
    }

    /// Check if a key is a variable-related effect/trigger.
    ///
    /// Flags are intentionally NOT classified: HOI4 flags have no "definition"
    /// concept — every flag name implicitly exists, an unset flag reads as
    /// false, and set-after-read (one-shot guards, e.g. `NOT = { has_global_flag = x }`
    /// followed later by `set_global_flag = x`) is a core engine idiom.
    /// Validating flag reads as unresolved would be a guaranteed false positive.
    fn classify_var_key(key: &str) -> Option<VarRefKind> {
        let lower = key.to_ascii_lowercase();
        match lower.as_str() {
            // Definitions (setters/creators)
            "set_variable" | "change_variable" => Some(VarRefKind::Definition),
            "set_temp_variable" | "set_local_variable" => Some(VarRefKind::Temp),
            // Reads (checkers/queries)
            // NOTE: unset variables read as 0 — the engine recovers gracefully,
            // so unresolved reads validate at WARNING (typo-catching), never ERROR.
            "check_variable" | "has_variable" => Some(VarRefKind::Read),
            // Mutations (non-temp)
            "add_to_variable"
            | "subtract_from_variable"
            | "multiply_variable"
            | "divide_variable"
            | "modulo_variable"
            | "clamp_variable"
            | "round_variable"
            | "randomize_variable"
            | "set_variable_to_random"
            | "clear_variable" => Some(VarRefKind::Mutation),
            // Temp mutations
            "add_to_temp_variable"
            | "subtract_from_temp_variable"
            | "multiply_temp_variable"
            | "divide_temp_variable"
            | "modulo_temp_variable"
            | "clamp_temp_variable"
            | "round_temp_variable"
            | "randomize_temp_variable"
            | "set_temp_variable_to_random"
            | "clear_temp_variable" => Some(VarRefKind::Temp),
            // Arrays
            "add_to_array"
            | "remove_from_array"
            | "clear_array"
            | "resize_array"
            | "is_in_array"
            | "find_highest_in_array"
            | "find_lowest_in_array"
            | "for_each_scope_loop"
            | "for_each_loop"
            | "random_scope_in_array" => Some(VarRefKind::Array),
            // Temp arrays
            "add_to_temp_array"
            | "remove_from_temp_array"
            | "clear_temp_array"
            | "resize_temp_array" => Some(VarRefKind::Array),
            _ => None,
        }
    }

    /// Extract variable name from an assignment value
    fn extract_var_name(ass: &ast::Assignment, source: &str) -> Option<String> {
        match &ass.value.value {
            ast::Value::String(name_span) => Some(name_span.resolve(source).to_string()),
            ast::Value::Block(inner) => {
                // Long form: var = xxx, variable = xxx, name = xxx, temp_var = xxx, array = xxx
                for entry in inner {
                    if let ast::Entry::Assignment(inner_ass) = entry {
                        let key = inner_ass.key_text(source).to_ascii_lowercase();
                        if key == "var"
                            || key == "variable"
                            || key == "name"
                            || key == "temp_var"
                            || key == "array"
                        {
                            if let Some(name) = inner_ass.value.value.as_str(source) {
                                return Some(name.to_string());
                            }
                        }
                    }
                }
                // Shorthand form: single entry key is the variable name
                if inner.len() == 1 {
                    if let ast::Entry::Assignment(inner_ass) = &inner[0] {
                        return Some(inner_ass.key_text(source).to_string());
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Check if a variable is defined in the current or an accessible scope
    fn is_var_defined(
        &self,
        name: &str,
        current_scope: Scope,
        is_temp: bool,
        is_global: bool,
    ) -> bool {
        if is_global {
            return true; // global. vars are always accessible
        }

        if is_temp {
            // Temp variables are chain-local — only valid if defined in this chain
            // For MVP, we track temp defs in file_vars; real chain tracking needs scope stack state
            return self
                .file_vars
                .get(name)
                .map(|(_, temp, _, _)| *temp)
                .unwrap_or(false);
        }

        // Check file-local definitions first
        if let Some((def_scope, temp, _, _)) = self.file_vars.get(name) {
            if *temp {
                return false; // temp vars don't cross chains
            }
            // Same scope or parent scope (Country can see Global, State can see Country via owner, etc.)
            if *def_scope == current_scope || self.is_accessible_scope(*def_scope, current_scope) {
                return true;
            }
        }

        // Check workspace definitions (from scanner data)
        if let Some(entry) = self.workspace_vars.get(name) {
            // We don't have scope info in Variable struct yet — for now, assume workspace vars are accessible
            // TODO: Enhance Variable with scope info from scanner
            if !entry.value().is_empty() {
                return true;
            }
        }

        false
    }

    /// Check if def_scope is accessible from current_scope
    /// HOI4 scope hierarchy: Global < Country < State/Character/Unit
    /// Character/Unit can access Country via ROOT
    /// State can access Country via OWNER/CONTROLLER
    fn is_accessible_scope(&self, def_scope: Scope, current_scope: Scope) -> bool {
        match (def_scope, current_scope) {
            // Global is accessible from everywhere
            (Scope::Global, _) => true,
            // Same scope
            (a, b) if a == b => true,
            // Country -> State/Character/Unit (via ROOT/OWNER)
            (Scope::Country, Scope::State) => true,
            (Scope::Country, Scope::Character) => true,
            (Scope::Country, Scope::Unit) => true,
            // State -> Character (character in state) — rare but possible
            (Scope::State, Scope::Character) => true,
            _ => false,
        }
    }

    /// Get array definition scope
    #[allow(dead_code)] // scaffolding for upcoming array validation
    fn get_array_scope(&self, name: &str) -> Option<Scope> {
        // Arrays don't have a dedicated scanner yet; check file_vars
        self.file_vars
            .get(name)
            .map(|(s, _, _, _)| *s)
            // TODO: enhance scanner to index arrays with scope
            .or_else(|| {
                // Fallback: check if it looks like a global array
                if name.starts_with("global.") {
                    Some(Scope::Global)
                } else {
                    None
                }
            })
    }

    /// Validate a variable reference.
    ///
    /// Severity is WARNING, not ERROR: reading an unset variable is legal —
    /// the engine silently returns 0. The diagnostic exists to catch typos
    /// (`my_vra` vs `my_var`), the most common silent-failure mode for
    /// variables. Flags are excluded from validation entirely (see
    /// [`Self::classify_var_key`]): an unset flag reads as false, which is
    /// the standard one-shot-guard idiom.
    fn validate_var_ref(
        &self,
        ref_: &VarRef,
        diags: &mut Vec<Diagnostic>,
        ctx: &ValidationContext,
    ) {
        let is_defined = self.is_var_defined(&ref_.name, ref_.scope, ref_.is_temp, ref_.is_global);

        if !is_defined {
            let kind_str = match ref_.kind {
                VarRefKind::Definition => "definition",
                VarRefKind::Read => "read",
                VarRefKind::Mutation => "mutation",
                VarRefKind::Flag => "flag",
                VarRefKind::Array => "array",
                VarRefKind::Temp => "temp",
            };

            let scope_str = ref_.scope.as_str();

            // Build helpful message
            let mut msg = format!(
                "{} '{}' not defined in accessible scope (current: {})",
                kind_str, ref_.name, scope_str
            );

            if ref_.is_global {
                msg = format!("global {} '{}' not defined", kind_str, ref_.name);
            } else if ref_.is_temp {
                msg = format!(
                    "temp {} '{}' not defined in current effect chain",
                    kind_str, ref_.name
                );
            } else {
                // Engine reality: unset variables read as 0, so this is a
                // typo warning, not a breakage.
                msg.push_str(" — unset variables read as 0; possible typo.");
            }

            diags.push(Diagnostic {
                range: ctx.range(&ref_.range),
                severity: Some(DiagnosticSeverity::WARNING),
                message: msg,
                code: Some(NumberOrString::String("HOM9001".to_string())), // Unresolved variable
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        }
    }

    /// Process a variable assignment (definition or reference)
    fn process_assignment(
        &mut self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        scope_stack: &ScopeStack,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key = ass.key_text(ctx.source);
        let current_scope = scope_stack.current();

        let Some(kind) = Self::classify_var_key(key) else {
            return;
        };

        let var_name = match Self::extract_var_name(ass, ctx.source) {
            Some(n) => n,
            None => return,
        };

        // Handle global prefix
        let (is_global, clean_name) = if var_name.starts_with("global.") {
            (true, var_name.strip_prefix("global.").unwrap().to_string())
        } else {
            (false, var_name)
        };

        let is_temp = matches!(kind, VarRefKind::Temp);

        let var_ref = VarRef {
            name: clean_name.clone(),
            kind,
            is_temp,
            is_global,
            range: ass.value.range.clone(),
            scope: current_scope,
        };

        match kind {
            VarRefKind::Definition | VarRefKind::Temp => {
                // Track definition
                if !is_temp || is_global {
                    self.file_vars.insert(
                        clean_name,
                        (current_scope, is_temp, is_global, ass.value.range.clone()),
                    );
                }
                // For temp variables, we still validate they're not redefined weirdly
                // but don't error on definition
            }
            VarRefKind::Read | VarRefKind::Mutation | VarRefKind::Flag | VarRefKind::Array => {
                // Validate reference
                self.validate_var_ref(&var_ref, diags, ctx);
            }
        }
    }
}

impl ValidationRule for VariableRule {
    fn check_assignment(
        &self,
        _ass: &ast::Assignment,
        _ctx: &ValidationContext,
        _scope: &ScopeStack,
        _pushed_scope: bool,
        _diags: &mut Vec<Diagnostic>,
    ) {
        // We need mutable access to file_vars for definitions, so this is a limitation
        // of the ValidationRule trait. We'll handle this by making the check_mutable.
        // Actually, we can't mutate self in check_assignment.
        // Solution: use interior mutability or split into two phases.
        // For now, we'll use a workaround with a cell.
    }
}

/// Interior mutable wrapper for VariableRule state
pub(crate) struct VariableRuleState {
    rule: RefCell<VariableRule>,
}

impl VariableRuleState {
    pub(crate) fn new(
        workspace_vars: &DashMap<InternedStr, Vec<Variable>>,
        event_targets: &DashMap<InternedStr, Vec<EventTarget>>,
    ) -> Self {
        Self {
            rule: RefCell::new(VariableRule::new(workspace_vars, event_targets)),
        }
    }
}

impl ValidationRule for VariableRuleState {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        scope: &ScopeStack,
        _pushed_scope: bool,
        diags: &mut Vec<Diagnostic>,
    ) {
        self.rule
            .borrow_mut()
            .process_assignment(ass, ctx, scope, diags);
    }
}
