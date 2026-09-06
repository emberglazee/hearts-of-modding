#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use crate::scope::scope::Scope;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
#[derive(Debug, Clone)]
pub struct Variable {
    #[allow(dead_code)]
    pub name: String,
    pub path: InternedStr,
    pub range: ast::Range,
    /// Statically inferred scope of the stored value (`Scope::Unknown` when
    /// the value is numeric, contextual (THIS/PREV/ROOT), or otherwise not a
    /// statically known scope reference). Used to resolve `var:name = { }`
    /// scope blocks. Only TAG-anchored values (`HAB`, `HAB.id`) infer
    /// `Country` — everything else stays `Unknown` (false negatives over
    /// false positives).
    pub scope: Scope,
}

#[derive(Debug, Clone)]
pub struct Array {
    #[allow(dead_code)]
    pub name: String,
    pub path: InternedStr,
    pub range: ast::Range,
    pub is_temp: bool,
}

#[derive(Debug, Clone)]
pub struct EventTarget {
    #[allow(dead_code)]
    pub name: String,
    pub path: InternedStr,
    pub range: ast::Range,
    pub is_global: bool,
    pub scope: Scope,
}

pub struct ScanResult {
    pub variables: HashMap<String, Vec<Variable>>,
    pub arrays: HashMap<String, Vec<Array>>,
    pub event_targets: HashMap<String, Vec<EventTarget>>,
}

pub fn scan_roots<F>(roots: &[std::path::PathBuf], filter: &F) -> ScanResult
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut variables: HashMap<String, Vec<Variable>> = HashMap::new();
    let mut arrays: HashMap<String, Vec<Array>> = HashMap::new();
    let mut event_targets: HashMap<String, Vec<EventTarget>> = HashMap::new();

    for root in roots {
        let mut dirs_to_check = vec![root.clone()];
        while let Some(current_dir) = dirs_to_check.pop() {
            if filter(&current_dir) {
                continue;
            }
            if let Ok(entries) = fs::read_dir(current_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if filter(&path) {
                            continue;
                        }
                        // Skip some obviously non-script directories for performance
                        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if name == ".git"
                            || name == "interface"
                            || name == "gfx"
                            || name == "localisation"
                            || name == "map"
                        {
                            continue;
                        }
                        dirs_to_check.push(path);
                    } else if path.extension().is_some_and(|ext| ext == "txt") {
                        if filter(&path) {
                            continue;
                        }
                        if let Ok(content) = fs::read_to_string(&path) {
                            {
                                let (script, _) = parser::parse_script(&content);
                                scan_entries(
                                    &script.entries,
                                    &script.source,
                                    &path.to_string_lossy(),
                                    &mut variables,
                                    &mut arrays,
                                    &mut event_targets,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    ScanResult {
        variables,
        arrays,
        event_targets,
    }
}

pub fn scan_variable_files<F>(files: &[PathBuf], filter: &F) -> ScanResult
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut variables: HashMap<String, Vec<Variable>> = HashMap::new();
    let mut arrays: HashMap<String, Vec<Array>> = HashMap::new();
    let mut event_targets: HashMap<String, Vec<EventTarget>> = HashMap::new();

    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        scan_entries(
            &script.entries,
            &script.source,
            &path.to_string_lossy(),
            &mut variables,
            &mut arrays,
            &mut event_targets,
        );
    });

    ScanResult {
        variables,
        arrays,
        event_targets,
    }
}

pub(crate) fn scan_entries(
    entries: &[ast::Entry],
    source: &str,
    path: &str,
    variables: &mut HashMap<String, Vec<Variable>>,
    arrays: &mut HashMap<String, Vec<Array>>,
    event_targets: &mut HashMap<String, Vec<EventTarget>>,
) {
    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                match ass.key_text(source) {
                    "set_variable"
                    | "set_temp_variable"
                    | "set_local_variable"
                    | "change_variable"
                    | "multiply_variable"
                    | "multiply_temp_variable"
                    | "divide_variable"
                    | "divide_temp_variable"
                    | "add_to_variable"
                    | "add_to_temp_variable"
                    | "subtract_from_variable"
                    | "subtract_from_temp_variable"
                    | "clamp_variable"
                    | "clamp_temp_variable"
                    | "round_variable"
                    | "round_temp_variable"
                    | "modulo_variable"
                    | "modulo_temp_variable"
                    | "clear_variable"
                    | "has_variable"
                    | "check_variable"
                    | "set_variable_to_random"
                    | "set_temp_variable_to_random" => {
                        handle_variable_assignment(ass, source, path, variables);
                    }
                    "save_event_target_as" | "save_global_event_target_as" => {
                        handle_event_target_assignment(ass, source, path, event_targets);
                    }
                    "add_to_array" | "add_to_temp_array" => {
                        let is_temp = ass
                            .key_text(source)
                            .eq_ignore_ascii_case("add_to_temp_array");
                        handle_array_assignment(ass, source, path, arrays, is_temp);
                    }
                    _ => {
                        // Recurse into blocks
                        match &ass.value.value {
                            ast::Value::Block(inner) => {
                                scan_entries(inner, source, path, variables, arrays, event_targets)
                            }
                            ast::Value::TaggedBlock(_, inner, _) => {
                                scan_entries(inner, source, path, variables, arrays, event_targets)
                            }
                            _ => {}
                        }
                    }
                }
            }
            ast::Entry::Value(val) => match &val.value {
                ast::Value::Block(inner) => {
                    scan_entries(inner, source, path, variables, arrays, event_targets)
                }
                ast::Value::TaggedBlock(_, inner, _) => {
                    scan_entries(inner, source, path, variables, arrays, event_targets)
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn handle_variable_assignment(
    ass: &ast::Assignment,
    source: &str,
    path: &str,
    variables: &mut HashMap<String, Vec<Variable>>,
) {
    match &ass.value.value {
        ast::Value::String(name_span) => {
            let name = name_span.resolve(source).to_string();
            add_variable(variables, name, path, &ass.value.range, Scope::Unknown);
        }
        ast::Value::Block(inner) => {
            let mut found_var = false;
            let mut value_text: Option<String> = None;
            // Collect (name, value) pairs. A block normally holds one
            // variable op; handle each `var=`-style entry so multi-entry
            // blocks don't silently drop definitions. `value` may come
            // before `var`, so insert only after the full scan.
            let mut pending: Vec<(String, ast::Range)> = Vec::new();
            for entry in inner {
                if let ast::Entry::Assignment(inner_ass) = entry {
                    let key = inner_ass.key_text(source);
                    if key == "var" || key == "variable" || key == "name" || key == "temp_var" {
                        if let Some(name) = inner_ass.value.value.as_str(source) {
                            pending.push((name.to_string(), inner_ass.value.range.clone()));
                            found_var = true;
                        }
                    } else if key.eq_ignore_ascii_case("value") {
                        if value_text.is_none() {
                            if let Some(v) = inner_ass.value.value.as_str(source) {
                                value_text = Some(v.to_string());
                            }
                        }
                    }
                }
            }
            for (name, range) in pending {
                let scope = value_text
                    .as_deref()
                    .map(infer_var_scope_from_value_text)
                    .unwrap_or(Scope::Unknown);
                add_variable(variables, name, path, &range, scope);
            }
            // Shorthand form: no explicit var/temp_var found, treat single-entry key as variable name
            if !found_var && inner.len() == 1 {
                if let ast::Entry::Assignment(inner_ass) = &inner[0] {
                    let var_name = inner_ass.key_text(source).to_string();
                    let scope = inner_ass
                        .value
                        .value
                        .as_str(source)
                        .map(infer_var_scope_from_value_text)
                        .unwrap_or(Scope::Unknown);
                    add_variable(variables, var_name, path, &inner_ass.key_range, scope);
                }
            }
        }
        _ => {}
    }
}

/// Infer the scope of a variable's stored value from its raw RHS text.
///
/// Only TAG-anchored references are certain: `HAB`, `VN6`, `HAB.id`,
/// `DEN.supporting_nation`-style values whose head segment is a
/// syntactically valid country tag resolve to `Country` (mirrors
/// `Scope::from_str`'s tag handling). Everything else — numerics,
/// contextual pointers (THIS/PREV/ROOT/FROM), game variables,
/// `token:` literals — is `Unknown`. Callers treat `Unknown` as
/// "can't resolve" and keep the current safe behaviour (skip HOM004).
fn infer_var_scope_from_value_text(val: &str) -> Scope {
    let head = val.split(['.', ':']).next().unwrap_or("").trim();
    // Strip a `var:`/`temp_var:` value wrapper (`set_variable = { x = var:y }`
    // stores whatever `y` holds — which we can't see here).
    let head_upper = head.to_ascii_uppercase();
    if head_upper.starts_with("VAR:") || head_upper.starts_with("TEMP_VAR:") {
        return Scope::Unknown;
    }
    if crate::scanner::country_scanner::is_valid_tag(head) {
        Scope::Country
    } else {
        Scope::Unknown
    }
}

fn handle_array_assignment(
    ass: &ast::Assignment,
    source: &str,
    path: &str,
    arrays: &mut HashMap<String, Vec<Array>>,
    is_temp: bool,
) {
    match &ass.value.value {
        ast::Value::String(name_span) => {
            let name = name_span.resolve(source).to_string();
            add_array(arrays, name, path, &ass.value.range, is_temp);
        }
        ast::Value::Block(inner) => {
            let mut found = false;
            for entry in inner {
                if let ast::Entry::Assignment(inner_ass) = entry {
                    let key = inner_ass.key_text(source);
                    if key == "array" {
                        if let Some(name) = inner_ass.value.value.as_str(source) {
                            add_array(
                                arrays,
                                name.to_string(),
                                path,
                                &inner_ass.value.range,
                                is_temp,
                            );
                            found = true;
                        }
                    }
                }
            }
            // Shorthand form: add_to_array = { my_array = value } or { array_name = 5 }
            if !found && inner.len() == 1 {
                if let ast::Entry::Assignment(inner_ass) = &inner[0] {
                    let arr_name = inner_ass.key_text(source).to_string();
                    // Guard: ignore when the sole key is "array" but value wasn't a string (e.g. array = { complex })
                    // In that case we already tried and failed; don't treat "array" as name.
                    if !arr_name.eq_ignore_ascii_case("array") {
                        add_array(arrays, arr_name, path, &inner_ass.key_range, is_temp);
                    }
                }
            }
            // Also handle short-hand with two entries? e.g. add_to_array = { array = X value = Y } already handled above.
            // If block has `array = X` found, we already added. Nothing else.
        }
        _ => {}
    }
}

fn handle_event_target_assignment(
    ass: &ast::Assignment,
    source: &str,
    path: &str,
    event_targets: &mut HashMap<String, Vec<EventTarget>>,
) {
    if let Some(name) = ass.value.value.as_str(source) {
        let is_global = ass.key_text(source) == "save_global_event_target_as";
        add_event_target(
            event_targets,
            name.to_string(),
            path,
            &ass.value.range,
            is_global,
        );
    }
}

fn add_variable(
    variables: &mut HashMap<String, Vec<Variable>>,
    name: String,
    path: &str,
    range: &ast::Range,
    scope: Scope,
) {
    let entry = Variable {
        name: name.clone(),
        path: std::sync::Arc::from(path),
        range: range.clone(),
        scope,
    };
    variables.entry(name).or_default().push(entry);
}

fn add_array(
    arrays: &mut HashMap<String, Vec<Array>>,
    name: String,
    path: &str,
    range: &ast::Range,
    is_temp: bool,
) {
    let entry = Array {
        name: name.clone(),
        path: std::sync::Arc::from(path),
        range: range.clone(),
        is_temp,
    };
    arrays.entry(name).or_default().push(entry);
}

fn add_event_target(
    event_targets: &mut HashMap<String, Vec<EventTarget>>,
    name: String,
    path: &str,
    range: &ast::Range,
    is_global: bool,
) {
    let entry = EventTarget {
        name: name.clone(),
        path: std::sync::Arc::from(path),
        range: range.clone(),
        is_global,
        scope: Scope::Unknown,
    };
    event_targets.entry(name).or_default().push(entry);
}
