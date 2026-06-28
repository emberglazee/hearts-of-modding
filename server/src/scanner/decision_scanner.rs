#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A decision defined inside a `common/decisions/*.txt` category block.
#[derive(Debug, Clone)]
pub struct Decision {
    /// The decision's key (e.g. `my_decision_1`)
    pub key: String,
    /// The category this decision belongs to (e.g. `my_decision_category`)
    pub category: String,
    pub path: InternedStr,
    pub range: ast::Range,
}

pub fn scan_decisions<F>(roots: &[PathBuf], filter: &F) -> HashMap<String, Decision>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();

    for root in roots {
        crate::utils::fs_util::walk_and_parse_files(
            &root.join("common/decisions"),
            &["txt"],
            filter,
            |path, content| {
                let (script, _) = parser::parse_script(&content);
                find_decisions_in_entries(
                    &script.entries,
                    &script.source,
                    &path.to_string_lossy(),
                    &mut map,
                );
            },
        );
    }

    map
}

pub fn scan_decision_files<F>(files: &[PathBuf], filter: &F) -> HashMap<String, Decision>
where
    F: Fn(&Path) -> bool,
{
    let mut map = HashMap::new();

    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        find_decisions_in_entries(
            &script.entries,
            &script.source,
            &path.to_string_lossy(),
            &mut map,
        );
    });

    map
}

/// Extract decision IDs from category blocks in decision files.
///
/// Structure:
/// ```txt
/// my_category = {
///     icon = some_icon
///     my_decision = { ... }
///     another_decision = { ... }
/// }
/// ```
///
/// Each entry inside a category block whose value is a Block is treated
/// as a decision definition. Category properties (simple values) are ignored.
pub(crate) fn find_decisions_in_entries(
    entries: &[ast::Entry],
    source: &str,
    file_path: &str,
    map: &mut HashMap<String, Decision>,
) {
    for entry in entries {
        match entry {
            ast::Entry::Assignment(ass) => {
                // Top-level assignment: could be a category block
                if let ast::Value::Block(inner) = &ass.value.value {
                    let category_key = ass.key_text(source);
                    // Skip obvious non-decision top-level blocks
                    let cat_lower = category_key.to_ascii_lowercase();
                    if cat_lower == "country_event"
                        || cat_lower == "state_event"
                        || cat_lower == "news_event"
                        || cat_lower == "unit_leader_event"
                        || cat_lower == "operative_leader_event"
                        || cat_lower == "focus"
                        || cat_lower == "idea"
                    {
                        // These aren't decision categories — recurse into sub-blocks
                        // in case decisions are nested inside something else
                        find_decisions_in_entries(inner, source, file_path, map);
                        continue;
                    }

                    for inner_entry in inner {
                        if let ast::Entry::Assignment(inner_ass) = inner_entry {
                            if let ast::Value::Block(_) = &inner_ass.value.value {
                                let decision_key = inner_ass.key_text(source);
                                map.insert(
                                    decision_key.to_string(),
                                    Decision {
                                        key: decision_key.to_string(),
                                        category: category_key.to_string(),
                                        path: std::sync::Arc::from(file_path),
                                        range: inner_ass.key_range.clone(),
                                    },
                                );
                            }
                        }
                    }
                } else {
                    // Recurse into non-block assignments to handle nested structures
                    // (e.g. shared_focus = TAG_name)
                    if let ast::Value::Block(inner_entries) = &ass.value.value {
                        find_decisions_in_entries(inner_entries, source, file_path, map);
                    }
                }
            }
            ast::Entry::Value(val) => match &val.value {
                ast::Value::Block(inner_entries) | ast::Value::TaggedBlock(_, inner_entries, _) => {
                    find_decisions_in_entries(inner_entries, source, file_path, map);
                }
                _ => {}
            },
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_decisions_basic() {
        let input = r#"political_actions = {
    my_decision = {
        icon = test_icon
        cost = 50
        complete_effect = {
            add_political_power = 50
        }
    }
    another_decision = {
        fire_only_once = yes
    }
}"#;
        let (script, _) = parser::parse_script(input);
        let mut map = HashMap::new();
        find_decisions_in_entries(&script.entries, &script.source, "test.txt", &mut map);

        assert_eq!(map.len(), 2);
        assert!(map.contains_key("my_decision"));
        assert!(map.contains_key("another_decision"));

        let d1 = map.get("my_decision").unwrap();
        assert_eq!(d1.category, "political_actions");
        assert_eq!(d1.key, "my_decision");

        let d2 = map.get("another_decision").unwrap();
        assert_eq!(d2.category, "political_actions");
    }

    #[test]
    fn test_find_decisions_multiple_categories() {
        let input = r#"eco_decisions = {
    build_factories = {
        cost = 100
    }
    improve_infrastructure = {
        cost = 50
    }
}
war_decisions = {
    declare_rally = {
        fire_only_once = yes
    }
}"#;
        let (script, _) = parser::parse_script(input);
        let mut map = HashMap::new();
        find_decisions_in_entries(&script.entries, &script.source, "test.txt", &mut map);

        assert_eq!(map.len(), 3);

        let d = map.get("build_factories").unwrap();
        assert_eq!(d.category, "eco_decisions");

        let d = map.get("declare_rally").unwrap();
        assert_eq!(d.category, "war_decisions");
    }

    #[test]
    fn test_find_decisions_empty_category() {
        let input = r#"empty_cat = {
    icon = my_icon
}"#;
        let (script, _) = parser::parse_script(input);
        let mut map = HashMap::new();
        find_decisions_in_entries(&script.entries, &script.source, "test.txt", &mut map);

        assert_eq!(
            map.len(),
            0,
            "No decisions should be found in an empty category"
        );
    }

    #[test]
    fn test_find_decisions_skips_event_blocks() {
        // Decision files can sometimes be in the same directory as event files
        // (unlikely but the scanner should be robust)
        let input = r#"my_cat = {
    test_decision = {
        icon = x
    }
}
country_event = {
    id = test.1
    title = "test"
}
"#;
        let (script, _) = parser::parse_script(input);
        let mut map = HashMap::new();
        find_decisions_in_entries(&script.entries, &script.source, "test.txt", &mut map);

        // Should only find the decision, not treat country_event as a category
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("test_decision"));
    }

    #[test]
    fn test_scan_decision_files_empty() {
        let map = scan_decision_files(&[], &|_| true);
        assert!(map.is_empty());
    }
}
