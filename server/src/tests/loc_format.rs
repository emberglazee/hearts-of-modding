#[cfg(test)]
mod tests {
    use crate::parser::loc_parser::{format_loc_file, parse_loc_file};

    const HEADER: &str = "l_english:\n";

    #[test]
    fn test_format_emits_single_colon() {
        let input = format!("{HEADER} FOO:0 \"bar\"\n FOO_desc:1 \"baz\"\n");
        let formatted = format_loc_file(&input, false);
        assert!(
            formatted.contains("\tFOO:0 \"bar\"\n"),
            "expected single-colon entry, got:\n{formatted}"
        );
        assert!(
            formatted.contains("\tFOO_desc:1 \"baz\"\n"),
            "expected single-colon entry, got:\n{formatted}"
        );
        assert!(
            !formatted.contains(":0:"),
            "double colon must not appear, got:\n{formatted}"
        );
    }

    #[test]
    fn test_format_roundtrip_preserves_entries() {
        let input = format!("{HEADER} FOO:0 \"bar\"\n FOO_desc:1 \"baz\"\n");
        let formatted = format_loc_file(&input, false);
        let (map, _, _) = parse_loc_file(&formatted, "test.yml");
        assert_eq!(map.len(), 2, "round-trip dropped entries:\n{formatted}");
        assert!(map.values().any(|e| &*e.key == "FOO"));
        assert!(map.values().any(|e| &*e.key == "FOO_desc"));
    }

    #[test]
    fn test_format_heals_double_colon_input() {
        // Files corrupted by the old formatter must self-heal on next format.
        let input = format!("{HEADER} FOO:0: \"bar\"\n");
        let formatted = format_loc_file(&input, false);
        assert!(
            formatted.contains("\tFOO:0 \"bar\"\n"),
            "expected healed single-colon entry, got:\n{formatted}"
        );
        let (map, _, _) = parse_loc_file(&formatted, "test.yml");
        assert_eq!(
            map.len(),
            1,
            "healed file still drops entries:\n{formatted}"
        );
    }
}
