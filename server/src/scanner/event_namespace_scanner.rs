#![allow(dead_code)]
use crate::data::interner::InternedStr;
use crate::parser::ast;
use crate::parser::parser;
use std::collections::HashMap;

/// A namespace declared via `add_namespace = my_namespace` in event files.
///
/// Namespaces must be declared before events that use them. The game assigns
/// an internal numeric ID to each namespace (starting at 10, incrementing by 1,
/// multiplying by 100000), which is combined with the event's numeric ID
/// (e.g., my_event.123) to produce the internal event ID.
#[derive(Debug, Clone)]
pub struct EventNamespace {
    pub name: String,
    pub path: InternedStr,
    pub range: ast::Range,
}

/// Parsed components of an event ID in the form `namespace.integer_id`.
#[derive(Debug, Clone)]
pub struct ParsedEventId<'a> {
    /// The namespace portion (e.g., "my_event" from "my_event.123")
    pub namespace: &'a str,
    /// The raw integer portion as a string (e.g., "123" from "my_event.123")
    pub numeric_raw: &'a str,
    /// Whether the numeric portion is a valid integer
    pub is_valid_integer: bool,
    /// The parsed integer value, if valid
    pub numeric_value: Option<u64>,
}

/// Parse an event ID string into its namespace and numeric components.
///
/// HOI4 event IDs follow the format: `<namespace>.<integer_id>`
/// e.g., `my_event.123`, `news.0`, `focus_events.100001`
///
/// Returns `None` if the ID doesn't contain a dot separator.
pub fn parse_event_id(id: &str) -> Option<ParsedEventId<'_>> {
    let dot_pos = id.rfind('.')?;

    let namespace = &id[..dot_pos];
    let numeric_part = &id[dot_pos + 1..];

    if namespace.is_empty() || numeric_part.is_empty() {
        return None;
    }

    let (numeric_value, is_valid_integer) = match numeric_part.parse::<u64>() {
        Ok(n) => (Some(n), true),
        Err(_) => (None, false),
    };

    Some(ParsedEventId {
        namespace,
        numeric_raw: numeric_part,
        is_valid_integer,
        numeric_value,
    })
}

/// Extract `add_namespace` declarations from parsed script content.
///
/// Returns a map of namespace name → EventNamespace for all `add_namespace = <name>`
/// declarations found in the entries. Namespaces can appear anywhere in a file,
/// but must be outside any event block (top-level entries).
pub(crate) fn find_namespaces_in_entries(
    entries: &[ast::Entry],
    source: &str,
    file_path: &str,
    map: &mut HashMap<String, EventNamespace>,
) {
    for entry in entries {
        // Namespaces are always top-level assignments (never inside blocks)
        if let ast::Entry::Assignment(ass) = entry {
            let key = ass.key_text(source);
            if key == "add_namespace" {
                if let Some(name) = ass.value.value.as_str(source) {
                    // Keep the first declaration's path — subsequent ones are duplicates.
                    // The first declaration's internal ID is what the game uses.
                    // Store namespace name lowercased for case-insensitive lookups.
                    map.entry(name.to_ascii_lowercase())
                        .or_insert(EventNamespace {
                            name: name.to_string(),
                            path: std::sync::Arc::from(file_path),
                            range: ass.value.range.clone(),
                        });
                }
            }
        }
    }
}

/// Scan event files for `add_namespace` declarations, returning a namespace map.
pub fn scan_event_namespaces<F>(
    files: &[std::path::PathBuf],
    filter: &F,
) -> HashMap<String, EventNamespace>
where
    F: Fn(&std::path::Path) -> bool,
{
    let mut namespaces = HashMap::new();

    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let (script, _) = parser::parse_script(&content);
        find_namespaces_in_entries(
            &script.entries,
            &script.source,
            &path.to_string_lossy(),
            &mut namespaces,
        );
    });

    namespaces
}

/// Extract `add_namespace` declarations from a single file (for incremental updates).
pub fn find_namespaces_in_file(
    content: &str,
    file_path: &str,
    map: &mut HashMap<String, EventNamespace>,
) {
    let (script, _) = parser::parse_script(content);
    find_namespaces_in_entries(&script.entries, &script.source, file_path, map);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_event_id_basic() {
        let parsed = parse_event_id("my_event.123").unwrap();
        assert_eq!(parsed.namespace, "my_event");
        assert_eq!(parsed.numeric_raw, "123");
        assert!(parsed.is_valid_integer);
        assert_eq!(parsed.numeric_value, Some(123));
    }

    #[test]
    fn test_parse_event_id_with_subnamespace() {
        let parsed = parse_event_id("my_event.subtopic.1").unwrap();
        assert_eq!(parsed.namespace, "my_event.subtopic");
        assert_eq!(parsed.numeric_raw, "1");
        assert!(parsed.is_valid_integer);
        assert_eq!(parsed.numeric_value, Some(1));
    }

    #[test]
    fn test_parse_event_id_non_integer() {
        let parsed = parse_event_id("my_event.abc").unwrap();
        assert_eq!(parsed.namespace, "my_event");
        assert_eq!(parsed.numeric_raw, "abc");
        assert!(!parsed.is_valid_integer);
        assert_eq!(parsed.numeric_value, None);
    }

    #[test]
    fn test_parse_event_id_no_dot() {
        assert!(parse_event_id("my_event").is_none());
    }

    #[test]
    fn test_parse_event_id_empty_id() {
        assert!(parse_event_id(".123").is_none());
        assert!(parse_event_id("namespace.").is_none());
    }

    #[test]
    fn test_find_namespaces() {
        let content = r#"
add_namespace = my_event
add_namespace = my_hidden_event

country_event = {
    id = my_event.1
}
"#;
        let (script, _) = parser::parse_script(content);
        let mut map = HashMap::new();
        find_namespaces_in_entries(&script.entries, &script.source, "test.txt", &mut map);
        assert_eq!(map.len(), 2);
        assert!(map.contains_key("my_event"));
        assert!(map.contains_key("my_hidden_event"));
    }
}
