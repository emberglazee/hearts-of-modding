use crate::parser;
use crate::ast;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub path: String,
    pub range: ast::Range,
}

#[derive(Debug, Clone)]
pub struct EventTarget {
    pub name: String,
    pub path: String,
    pub range: ast::Range,
    pub is_global: bool,
}

pub struct ScanResult {
    pub variables: HashMap<String, Vec<Variable>>,
    pub event_targets: HashMap<String, Vec<EventTarget>>,
}

pub fn scan_roots(roots: &[std::path::PathBuf]) -> ScanResult {
    let mut variables: HashMap<String, Vec<Variable>> = HashMap::new();
    let mut event_targets: HashMap<String, Vec<EventTarget>> = HashMap::new();

    for root in roots {
        let mut dirs_to_check = vec![root.clone()];
        while let Some(current_dir) = dirs_to_check.pop() {
            if let Ok(entries) = fs::read_dir(current_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        // Skip some obviously non-script directories for performance
                        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if name == ".git" || name == "interface" || name == "gfx" || name == "localisation" || name == "map" {
                            continue;
                        }
                        dirs_to_check.push(path);
                    } else if path.extension().map_or(false, |ext| ext == "txt") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(script) = parser::parse_script(&content) {
                                scan_entries(&script.entries, &path.to_string_lossy(), &mut variables, &mut event_targets);
                            }
                        }
                    }
                }
            }
        }
    }

    ScanResult { variables, event_targets }
}

fn scan_entries(
    entries: &[ast::Entry],
    path: &str,
    variables: &mut HashMap<String, Vec<Variable>>,
    event_targets: &mut HashMap<String, Vec<EventTarget>>,
) {
    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                match ass.key.as_str() {
                    "set_variable" | "set_temp_variable" | "set_local_variable" | 
                    "change_variable" | "multiply_variable" | "divide_variable" | "add_to_variable" | "subtract_from_variable" |
                    "clamp_variable" | "round_variable" | "clear_variable" | "has_variable" | "check_variable" => {
                        handle_variable_assignment(ass, path, variables);
                    }
                    "save_event_target_as" | "save_global_event_target_as" => {
                        handle_event_target_assignment(ass, path, event_targets);
                    }
                    _ => {
                        // Recurse into blocks
                        match &ass.value.value {
                            ast::Value::Block(inner) => scan_entries(inner, path, variables, event_targets),
                            ast::Value::TaggedBlock(_, inner) => scan_entries(inner, path, variables, event_targets),
                            _ => {}
                        }
                    }
                }
            }
            ast::Entry::Value(val) => {
                match &val.value {
                    ast::Value::Block(inner) => scan_entries(inner, path, variables, event_targets),
                    ast::Value::TaggedBlock(_, inner) => scan_entries(inner, path, variables, event_targets),
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn handle_variable_assignment(
    ass: &ast::Assignment,
    path: &str,
    variables: &mut HashMap<String, Vec<Variable>>,
) {
    match &ass.value.value {
        ast::Value::String(name) => {
            add_variable(variables, name.clone(), path, &ass.value.range);
        }
        ast::Value::Block(inner) => {
            // Find 'name = xxx' inside the block
            for entry in inner {
                if let ast::Entry::Assignment(inner_ass) = entry {
                    if inner_ass.key == "name" {
                        if let ast::Value::String(name) = &inner_ass.value.value {
                            add_variable(variables, name.clone(), path, &inner_ass.value.range);
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn handle_event_target_assignment(
    ass: &ast::Assignment,
    path: &str,
    event_targets: &mut HashMap<String, Vec<EventTarget>>,
) {
    if let ast::Value::String(name) = &ass.value.value {
        let is_global = ass.key == "save_global_event_target_as";
        add_event_target(event_targets, name.clone(), path, &ass.value.range, is_global);
    }
}

fn add_variable(variables: &mut HashMap<String, Vec<Variable>>, name: String, path: &str, range: &ast::Range) {
    let entry = Variable {
        name: name.clone(),
        path: path.to_string(),
        range: range.clone(),
    };
    variables.entry(name).or_default().push(entry);
}

fn add_event_target(event_targets: &mut HashMap<String, Vec<EventTarget>>, name: String, path: &str, range: &ast::Range, is_global: bool) {
    let entry = EventTarget {
        name: name.clone(),
        path: path.to_string(),
        range: range.clone(),
        is_global,
    };
    event_targets.entry(name).or_default().push(entry);
}
