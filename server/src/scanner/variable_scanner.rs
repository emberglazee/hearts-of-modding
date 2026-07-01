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
    pub event_targets: HashMap<String, Vec<EventTarget>>,
}

pub fn scan_roots<F>(roots: &[std::path::PathBuf], filter: &F) -> ScanResult
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut variables: HashMap<String, Vec<Variable>> = HashMap::new();
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
        event_targets,
    }
}

pub fn scan_variable_files<F>(files: &[PathBuf], filter: &F) -> ScanResult
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut variables: HashMap<String, Vec<Variable>> = HashMap::new();
    let mut event_targets: HashMap<String, Vec<EventTarget>> = HashMap::new();

    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        scan_entries(
            &script.entries,
            &script.source,
            &path.to_string_lossy(),
            &mut variables,
            &mut event_targets,
        );
    });

    ScanResult {
        variables,
        event_targets,
    }
}

pub(crate) fn scan_entries(
    entries: &[ast::Entry],
    source: &str,
    path: &str,
    variables: &mut HashMap<String, Vec<Variable>>,
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
                    _ => {
                        // Recurse into blocks
                        match &ass.value.value {
                            ast::Value::Block(inner) => {
                                scan_entries(inner, source, path, variables, event_targets)
                            }
                            ast::Value::TaggedBlock(_, inner, _) => {
                                scan_entries(inner, source, path, variables, event_targets)
                            }
                            _ => {}
                        }
                    }
                }
            }
            ast::Entry::Value(val) => match &val.value {
                ast::Value::Block(inner) => {
                    scan_entries(inner, source, path, variables, event_targets)
                }
                ast::Value::TaggedBlock(_, inner, _) => {
                    scan_entries(inner, source, path, variables, event_targets)
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
            add_variable(variables, name, path, &ass.value.range);
        }
        ast::Value::Block(inner) => {
            let mut found_var = false;
            for entry in inner {
                if let ast::Entry::Assignment(inner_ass) = entry {
                    let key = inner_ass.key_text(source);
                    // Long form: var = xxx, variable = xxx, name = xxx, temp_var = xxx
                    if key == "var" || key == "variable" || key == "name" || key == "temp_var" {
                        if let Some(name) = inner_ass.value.value.as_str(source) {
                            add_variable(variables, name.to_string(), path, &inner_ass.value.range);
                            found_var = true;
                        }
                    }
                }
            }
            // Shorthand form: no explicit var/temp_var found, treat single-entry key as variable name
            if !found_var && inner.len() == 1 {
                if let ast::Entry::Assignment(inner_ass) = &inner[0] {
                    let var_name = inner_ass.key_text(source).to_string();
                    add_variable(variables, var_name, path, &inner_ass.key_range);
                }
            }
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
) {
    let entry = Variable {
        name: name.clone(),
        path: std::sync::Arc::from(path),
        range: range.clone(),
    };
    variables.entry(name).or_default().push(entry);
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
