use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scanner::variable_scanner::{Array, EventTarget, Variable};
use crate::scope::scope::{Scope, ScopeStack};
use dashmap::DashMap;
use std::cell::RefCell;
use std::collections::HashMap;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Variable / array reference kinds for diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VarRefKind {
    /// `set_variable`, `change_variable` (definition)
    Definition,
    /// `check_variable`, `has_variable` (read/comparison)
    Read,
    /// `add_to_variable`, `subtract_from_variable`, etc. (mutation)
    Mutation,
    /// flag ops — reserved, never classified (see classify)
    #[allow(dead_code)]
    Flag,
    /// `add_to_array` — array creation (definition, regular scope)
    ArrayDef,
    /// `add_to_temp_array` — array creation (temp, chain-local)
    ArrayTempDef,
    /// `is_in_array`, `for_each_scope_loop`, `remove_from_array`, `clear_array`, etc. (array read/mutation)
    Array,
    /// `set_temp_variable`, `add_to_temp_variable` (temp - chain local)
    Temp,
}

/// Variable / array reference info for validation
#[derive(Debug, Clone)]
struct VarRef {
    name: String,
    kind: VarRefKind,
    is_temp: bool,
    is_global: bool,
    range: ast::Range,
    scope: Scope,
}

/// Variable + array validation rule (HOM9xxx range)
///
/// Validates:
/// - Variable definitions track their scope
/// - Variable reads/mutations resolve to a definition in the same or accessible scope
/// - Temp variables only valid within current effect chain
/// - Global variables (`global.name`) work everywhere
/// - Array definitions (`add_to_array` / `add_to_temp_array`) create the array;
///   all other array ops (`is_in_array`, `for_each_*`, `remove_from_*`, `clear_*`,
///   `resize_*`, `find_*`, `any_of_scopes` …) validate against a prior
///   `add_to_array` in the same or accessible scope. Temp arrays are chain-local.
///   An unset array reads as empty (like an unset variable reads as 0 / flag
///   as false) — diagnostic is WARNING for typo-catching, never ERROR.
pub(crate) struct VariableRule {
    /// Variable definitions found in this file, keyed by name -> (scope, is_temp, is_global, range)
    file_vars: HashMap<String, (Scope, bool, bool, ast::Range)>,
    /// Array definitions found in this file, keyed by name -> (scope, is_temp, is_global, range)
    file_arrays: HashMap<String, (Scope, bool, bool, ast::Range)>,
    /// Cross-file variable definitions from scanner data (DashMap from backend)
    workspace_vars: DashMap<InternedStr, Vec<Variable>>,
    /// Cross-file array definitions from scanner data
    workspace_arrays: DashMap<InternedStr, Vec<Array>>,
    /// Event targets for scope resolution
    #[allow(dead_code)] // scaffolding for upcoming event-target variable validation
    event_targets: DashMap<InternedStr, Vec<EventTarget>>,
}

impl VariableRule {
    pub(crate) fn new(
        workspace_vars: &DashMap<InternedStr, Vec<Variable>>,
        workspace_arrays: &DashMap<InternedStr, Vec<Array>>,
        event_targets: &DashMap<InternedStr, Vec<EventTarget>>,
    ) -> Self {
        Self {
            file_vars: HashMap::new(),
            file_arrays: HashMap::new(),
            workspace_vars: workspace_vars.clone(),
            workspace_arrays: workspace_arrays.clone(),
            event_targets: event_targets.clone(),
        }
    }

    /// Check if a key is a variable- or array-related effect/trigger.
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
            // Array creations (definitions)
            "add_to_array" => Some(VarRefKind::ArrayDef),
            "add_to_temp_array" => Some(VarRefKind::ArrayTempDef),
            // Array reads / mutations — all need a prior add_to_array
            "remove_from_array"
            | "clear_array"
            | "resize_array"
            | "is_in_array"
            | "find_highest_in_array"
            | "find_lowest_in_array"
            | "for_each_scope_loop"
            | "for_each_loop"
            | "random_scope_in_array"
            | "any_of"
            | "any_of_scopes"
            | "all_of"
            | "all_of_scopes" => Some(VarRefKind::Array),
            // Temp array mutations (still validated — need a prior add_to_temp_array)
            "remove_from_temp_array" | "clear_temp_array" | "resize_temp_array" => {
                Some(VarRefKind::Array)
            }
            _ => None,
        }
    }

    /// Extract variable / array name from an assignment value
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
                            || key == "temp_array"
                        {
                            if let Some(name) = inner_ass.value.value.as_str(source) {
                                return Some(name.to_string());
                            }
                        }
                    }
                }
                // Shorthand form: single entry key is the variable/array name
                // e.g. set_variable = { my_var = 5 } or is_in_array = { my_array = value }
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

    /// Check if an array is defined in the current or an accessible scope.
    ///
    /// `add_to_array` / `add_to_temp_array` are the creators — an array is
    /// assumed empty by default (wiki: "if not already created"), so a read
    /// like `is_in_array` on a non-existent array just returns false.
    /// Validation is WARNING typo-catching, same as variables.
    fn is_array_defined(&self, name: &str, current_scope: Scope, is_global: bool) -> bool {
        if is_global {
            return true;
        }

        // File-local arrays (both regular and temp defs) — scope-aware.
        // Temp array defs are stored with is_temp flag but for reads we accept
        // either: a temp array read may be satisfied by a temp def in the same
        // file chain, and a regular read by a regular def. Checking both is
        // permissive (false-negative over false-positive) while the precise
        // temp-vs-regular read distinction is not encoded in the trigger key
        // (is_in_array reads both).
        if let Some((def_scope, _temp, _, _)) = self.file_arrays.get(name) {
            if *def_scope == current_scope || self.is_accessible_scope(*def_scope, current_scope) {
                return true;
            }
        }

        // Workspace arrays — only regular arrays (add_to_array) are in ScannerData;
        // add_to_temp_array writes are chain-local but we still store them for
        // cross-file typo-catching permissiveness. If any entry exists, consider defined.
        if let Some(entry) = self.workspace_arrays.get(name) {
            if !entry.value().is_empty() {
                return true;
            }
        }

        // Also check global-prefixed name in workspace (scanner stores stripped? No, stores raw.
        // We already stripped global. for is_global check, but workspace keys are raw names
        // like "TIR_global_campaign_holders" not "global.X", so this is not needed.)

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

    /// Get array definition scope (used for scaffolding; now backed by file_arrays)
    #[allow(dead_code)]
    fn get_array_scope(&self, name: &str) -> Option<Scope> {
        self.file_arrays
            .get(name)
            .map(|(s, _, _, _)| *s)
            .or_else(|| {
                if name.starts_with("global.") {
                    Some(Scope::Global)
                } else {
                    None
                }
            })
    }

    /// Validate a variable / array reference.
    ///
    /// Severity is WARNING, not ERROR: reading an unset variable/array is legal —
    /// the engine silently returns 0 / empty. The diagnostic exists to catch typos
    /// (`my_vra` vs `my_var`), the most common silent-failure mode.
    /// Flags are excluded from validation entirely (see [`Self::classify_var_key`]).
    fn validate_var_ref(
        &self,
        ref_: &VarRef,
        diags: &mut Vec<Diagnostic>,
        ctx: &ValidationContext,
    ) {
        let is_defined = match ref_.kind {
            VarRefKind::Array => self.is_array_defined(&ref_.name, ref_.scope, ref_.is_global),
            _ => self.is_var_defined(&ref_.name, ref_.scope, ref_.is_temp, ref_.is_global),
        };

        if !is_defined {
            let (kind_str, empty_msg) = match ref_.kind {
                VarRefKind::Definition => {
                    ("definition", " — unset variables read as 0; possible typo.")
                }
                VarRefKind::Read => ("read", " — unset variables read as 0; possible typo."),
                VarRefKind::Mutation => {
                    ("mutation", " — unset variables read as 0; possible typo.")
                }
                VarRefKind::Flag => ("flag", ""),
                VarRefKind::Array => ("array", " — arrays are empty by default; possible typo."),
                VarRefKind::ArrayDef => ("array", " — arrays are empty by default; possible typo."),
                VarRefKind::ArrayTempDef => (
                    "temp array",
                    " — arrays are empty by default; possible typo.",
                ),
                VarRefKind::Temp => ("temp", " — temp variables are chain-local; possible typo."),
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
                msg.push_str(empty_msg);
            }

            diags.push(Diagnostic {
                range: ctx.range(&ref_.range),
                severity: Some(DiagnosticSeverity::WARNING),
                message: msg,
                code: Some(NumberOrString::String("HOM9001".to_string())), // Unresolved variable/array
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        }
    }

    /// Process a variable / array assignment (definition or reference)
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

        // Handle global prefix — `global.name` is accessible everywhere.
        // Works for both variables (global.var) and arrays (global.array).
        let (is_global, clean_name) = if var_name.starts_with("global.") {
            (true, var_name.strip_prefix("global.").unwrap().to_string())
        } else {
            (false, var_name)
        };

        let is_temp = matches!(kind, VarRefKind::Temp | VarRefKind::ArrayTempDef);

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
                // Track variable definition
                if !is_temp || is_global {
                    self.file_vars.insert(
                        clean_name,
                        (current_scope, is_temp, is_global, ass.value.range.clone()),
                    );
                }
            }
            VarRefKind::ArrayDef | VarRefKind::ArrayTempDef => {
                // Track array definition (`add_to_array` / `add_to_temp_array` creates the array)
                // Temp arrays are still tracked file-locally; they do not cross chains.
                self.file_arrays.insert(
                    clean_name,
                    (current_scope, is_temp, is_global, ass.value.range.clone()),
                );
            }
            VarRefKind::Read | VarRefKind::Mutation | VarRefKind::Flag | VarRefKind::Array => {
                // Validate reference (variable or array read/mutation)
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
        // We need mutable access to file_vars/file_arrays for definitions, so this is a limitation
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
        workspace_arrays: &DashMap<InternedStr, Vec<Array>>,
        event_targets: &DashMap<InternedStr, Vec<EventTarget>>,
    ) -> Self {
        Self {
            rule: RefCell::new(VariableRule::new(
                workspace_vars,
                workspace_arrays,
                event_targets,
            )),
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
