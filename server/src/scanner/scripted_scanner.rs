#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ScriptedEntity {
    pub name: String,
    pub path: InternedStr,
    pub range: ast::Range,
    /// Precomputed at scan time: does this trigger's body PROVE it can never
    /// be true for an AI country (contains `is_ai = no` in a conjunctive
    /// position)? See [`ScriptedTriggerAnalysis`]. Stored here so consumers
    /// (HOM3017 event-option validation) don't re-read files from disk.
    pub guarantees_ai_invisible: bool,
}

/// Static analysis result for a scripted trigger's body.
///
/// `guarantees_ai_invisible` is true when the trigger body PROVES that the
/// trigger can never be true for an AI country — e.g. it contains
/// `is_ai = no`, or an `AND`/top-level conjunction containing `is_ai = no`.
/// Used by event validation (HOM3017): an event option whose `trigger`
/// provably excludes the AI needs no `ai_chance` block, because the only
/// AI-visible option always gets 100% of the proportional weight.
#[derive(Debug, Clone, Default)]
pub struct ScriptedTriggerAnalysis {
    pub guarantees_ai_invisible: bool,
}

/// Analyze a scripted-trigger body for properties relevant to validation.
///
/// Recognized shapes for "AI can never see this":
/// - top-level `is_ai = no`
/// - `AND = { ... }` / unbraced AND-list at any depth of pure trigger-block
///   nesting (`AND`, `OR` are transparent here: inside an OR, one branch
///   proving invisibility does NOT prove the whole trigger does, so OR arms
///   are not descended for this proof)
fn analyze_trigger_body(entries: &[ast::Entry], source: &str) -> ScriptedTriggerAnalysis {
    let mut analysis = ScriptedTriggerAnalysis::default();

    fn has_ai_no(entries: &[ast::Entry], source: &str) -> bool {
        for entry in entries {
            if let ast::Entry::Assignment(ass) = entry {
                let key = ass.key_text(source);
                match key.to_ascii_lowercase().as_str() {
                    // Direct: is_ai = no
                    "is_ai" => {
                        // `no` parses as Boolean(false); accept both forms.
                        let val: Option<String> = match &ass.value.value {
                            ast::Value::Boolean(b) => {
                                Some(if *b { "yes" } else { "no" }.to_string())
                            }
                            _ => ass
                                .value
                                .value
                                .as_str(source)
                                .map(|s| s.to_ascii_lowercase()),
                        };
                        if val.as_deref() == Some("no") {
                            return true;
                        }
                    }
                    // Conjunction: every arm must hold; if ANY arm proves
                    // AI-invisibility the conjunction does too. Descend into
                    // the AND body looking for is_ai = no anywhere in it
                    // (arms themselves may be nested blocks).
                    "and" | "limit" if matches!(&ass.value.value, ast::Value::Block(_)) => {
                        if let ast::Value::Block(arms) = &ass.value.value {
                            if has_ai_no(arms, source) {
                                return true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        false
    }

    analysis.guarantees_ai_invisible = has_ai_no(entries, source);
    analysis
}

/// Public predicate: does this scripted-trigger BODY prove that the trigger
/// can never be true for an AI country? See [`ScriptedTriggerAnalysis`].
pub fn body_guarantees_ai_invisible(entries: &[ast::Entry], source: &str) -> bool {
    analyze_trigger_body(entries, source).guarantees_ai_invisible
}

pub fn scan_directory<F>(dir_path: &Path, filter: &F) -> HashMap<String, ScriptedEntity>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();
    crate::utils::fs_util::walk_and_parse_files(dir_path, &["txt"], filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        collect_entities(&script, path, &mut map);
    });
    map
}

pub fn scan_scripted_files<F>(files: &[PathBuf], filter: &F) -> HashMap<String, ScriptedEntity>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();
    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        collect_entities(&script, path, &mut map);
    });
    map
}

fn collect_entities(script: &ast::Script, path: &Path, map: &mut HashMap<String, ScriptedEntity>) {
    for entry_ast in &script.entries {
        if let ast::Entry::Assignment(ass) = entry_ast {
            let name = ass.key_text(&script.source).to_string();
            let guarantees_ai_invisible = match &ass.value.value {
                ast::Value::Block(body) => {
                    analyze_trigger_body(body, &script.source).guarantees_ai_invisible
                }
                _ => false,
            };
            map.insert(
                name.clone(),
                ScriptedEntity {
                    name,
                    path: std::sync::Arc::from(path.to_string_lossy().as_ref()),
                    // Full block span — see event_scanner.rs: call
                    // hierarchy's range-overlap walk needs the whole body.
                    range: ast::Range {
                        start_line: ass.key_range.start_line,
                        start_col: ass.key_range.start_col,
                        end_line: ass.value.range.end_line,
                        end_col: ass.value.range.end_col,
                    },
                    guarantees_ai_invisible,
                },
            );
        }
    }
}
