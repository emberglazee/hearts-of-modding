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

    #[test]
    fn test_format_never_emits_a_bom_character() {
        // A BOM belongs to the file's ENCODING, not to the document text: VS
        // Code strips it from the synced text and restores it on save. The
        // formatter used to push U+FEFF into the buffer, so saving wrote that
        // character *in addition* to the file's own BOM — the "2+ BOMs" state
        // HOM6005 reports, produced by our own Format Document.
        use crate::data::hoi4_data::{UTF8_BOM, has_exactly_one_bom};

        let formatted = format_loc_file(&format!("{HEADER} FOO:0 \"bar\"\n"), false);
        assert!(
            !formatted.contains('\u{feff}'),
            "formatter emitted a BOM character into the document text:\n{formatted:?}"
        );

        // Simulate the editor's save: the file's own encoding BOM + the buffer.
        let mut saved = UTF8_BOM.to_vec();
        saved.extend_from_slice(formatted.as_bytes());
        assert!(
            has_exactly_one_bom(&saved),
            "saved file would carry more than one BOM"
        );
    }

    #[test]
    fn test_format_accepts_bom_prefixed_input() {
        // A client that does hand us the BOM must not defeat the header scan:
        // `trim()` does not remove U+FEFF, so `header_found` stayed false and
        // formatting returned the input unchanged.
        let input = format!("\u{feff}{HEADER} FOO:0 \"bar\"\n");
        let formatted = format_loc_file(&input, false);
        assert!(
            formatted.contains("\tFOO:0 \"bar\"\n"),
            "BOM-prefixed input was not formatted:\n{formatted:?}"
        );
        assert!(
            !formatted.contains('\u{feff}'),
            "BOM-prefixed input kept the character:\n{formatted:?}"
        );
    }
}
