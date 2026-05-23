use crate::ast;
use crate::parser;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AiStrategyPlan {
    #[allow(dead_code)]
    pub name: String,
    pub has_ai_national_focuses: bool,
    pub has_research: bool,
    pub has_ideas: bool,
    pub has_traits: bool,
    pub has_ai_strategy: bool,
    pub has_focus_factors: bool,
    pub has_weight: bool,
    #[allow(dead_code)]
    pub path: String,
    #[allow(dead_code)]
    pub range: ast::Range,
}

pub fn scan_ai_strategy_plans<F>(roots: &[PathBuf], filter: &F) -> HashMap<String, AiStrategyPlan>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut plans = HashMap::new();

    for root in roots {
        let dir = root.join("common/ai_strategy_plans");
        if dir.exists() {
            let found = scan_directory(&dir, filter);
            plans.extend(found);
        }
    }

    plans
}

fn scan_directory<F>(dir_path: &Path, filter: &F) -> HashMap<String, AiStrategyPlan>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut map = HashMap::new();
    let mut dirs_to_check = vec![dir_path.to_path_buf()];

    while let Some(current_dir) = dirs_to_check.pop() {
        if filter(&current_dir) {
            continue;
        }
        if let Ok(entries) = fs::read_dir(current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if !filter(&path) {
                        dirs_to_check.push(path);
                    }
                } else if path.extension().is_some_and(|ext| ext == "txt") {
                    if filter(&path) {
                        continue;
                    }
                    if let Ok(content) = fs::read_to_string(&path) {
                        let (script, _) = parser::parse_script(&content);
                        extract_plans(&script.entries, &path, &mut map);
                    }
                }
            }
        }
    }

    map
}

fn extract_plans(entries: &[ast::Entry], path: &Path, map: &mut HashMap<String, AiStrategyPlan>) {
    for entry in entries {
        if let ast::Entry::Assignment(ass) = entry {
            if let ast::Value::Block(inner_entries) = &ass.value.value {
                let mut has_ai_national_focuses = false;
                let mut has_research = false;
                let mut has_ideas = false;
                let mut has_traits = false;
                let mut has_ai_strategy = false;
                let mut has_focus_factors = false;
                let mut has_weight = false;

                for inner in inner_entries {
                    if let ast::Entry::Assignment(inner_ass) = inner {
                        match inner_ass.key.to_lowercase().as_str() {
                            "ai_national_focuses" => has_ai_national_focuses = true,
                            "research" => has_research = true,
                            "ideas" => has_ideas = true,
                            "traits" => has_traits = true,
                            "ai_strategy" => has_ai_strategy = true,
                            "focus_factors" => has_focus_factors = true,
                            "weight" => has_weight = true,
                            _ => {}
                        }
                    }
                }

                map.insert(
                    ass.key.clone(),
                    AiStrategyPlan {
                        name: ass.key.clone(),
                        has_ai_national_focuses,
                        has_research,
                        has_ideas,
                        has_traits,
                        has_ai_strategy,
                        has_focus_factors,
                        has_weight,
                        path: path.to_string_lossy().to_string(),
                        range: ass.key_range.clone(),
                    },
                );
            }
        }
    }
}
