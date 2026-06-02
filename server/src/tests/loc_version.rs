#[cfg(test)]
mod tests {
    use crate::data::interner::InternedStr;
    use crate::parser::ast::Range;
    use crate::parser::loc_parser::{self, LocEntry};

    #[test]
    fn test_check_unnecessary_version() {
        let entry = LocEntry {
            key: InternedStr::from("test_key"),
            value: "test value".to_string(),
            range: Range {
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 10,
            },
            path: InternedStr::from("file_a.yml"),
            value_start_col: 15,
            version: Some("0".to_string()),
            version_range: Some(Range {
                start_line: 0,
                start_col: 9,
                end_line: 0,
                end_col: 10,
            }),
        };

        // Case 1: Version present
        let diagnostic = loc_parser::check_unnecessary_version(&entry);
        assert!(diagnostic.is_some());
        assert!(diagnostic.unwrap().message.contains("unnecessary"));

        // Case 2: No version present
        let mut entry_no_version = entry.clone();
        entry_no_version.version = None;
        entry_no_version.version_range = None;
        let diagnostic = loc_parser::check_unnecessary_version(&entry_no_version);
        assert!(diagnostic.is_none());
    }
}
