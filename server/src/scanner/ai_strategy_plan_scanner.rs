#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
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
    pub path: InternedStr,
    #[allow(dead_code)]
    pub range: ast::Range,
}

pub fn scan_ai_strategy_plans<F>(roots: &[PathBuf], filter: &F) -> HashMap<String, AiStrategyPlan>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut plans = HashMap::new();

    for root in roots {
        crate::utils::fs_util::walk_and_parse_files(
            &root.join("common/ai_strategy_plans"),
            &["txt"],
            filter,
            |path, content| {
                let (script, _) = parser::parse_script(&content);
                extract_plans(&script.entries, &script.source, path, &mut plans);
            },
        );
    }

    plans
}

pub fn scan_ai_strategy_plan_files<F>(
    files: &[PathBuf],
    filter: &F,
) -> HashMap<String, AiStrategyPlan>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut plans = HashMap::new();

    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        extract_plans(&script.entries, &script.source, path, &mut plans);
    });

    plans
}

pub(crate) fn extract_plans(
    entries: &[ast::Entry],
    source: &str,
    path: &Path,
    map: &mut HashMap<String, AiStrategyPlan>,
) {
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
                        let inner_key = inner_ass.key_text(source);
                        if inner_key.eq_ignore_ascii_case("ai_national_focuses") {
                            has_ai_national_focuses = true;
                        } else if inner_key.eq_ignore_ascii_case("research") {
                            has_research = true;
                        } else if inner_key.eq_ignore_ascii_case("ideas") {
                            has_ideas = true;
                        } else if inner_key.eq_ignore_ascii_case("traits") {
                            has_traits = true;
                        } else if inner_key.eq_ignore_ascii_case("ai_strategy") {
                            has_ai_strategy = true;
                        } else if inner_key.eq_ignore_ascii_case("focus_factors") {
                            has_focus_factors = true;
                        } else if inner_key.eq_ignore_ascii_case("weight") {
                            has_weight = true;
                        }
                    }
                }

                let name = ass.key_text(source).to_string();
                map.insert(
                    name.clone(),
                    AiStrategyPlan {
                        name,
                        has_ai_national_focuses,
                        has_research,
                        has_ideas,
                        has_traits,
                        has_ai_strategy,
                        has_focus_factors,
                        has_weight,
                        path: std::sync::Arc::from(path.to_string_lossy().as_ref()),
                        range: ass.key_range.clone(),
                    },
                );
            }
        }
    }
}
