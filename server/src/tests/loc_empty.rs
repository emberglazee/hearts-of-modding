#[cfg(test)]
mod tests {
    use crate::parser::loc_parser;

    #[test]
    fn test_missing_language_header_suppression() {
        let empty_content = "";
        let commented_content = "# Just a comment\n\n# Another comment\n";
        let valid_content = "l_english:\n test_key:0 \"Test\"";
        let missing_header_content = "test_key:0 \"Test\"";

        assert!(loc_parser::validate_loc_file_structure(empty_content).is_empty());
        assert!(loc_parser::validate_loc_file_structure(commented_content).is_empty());
        assert!(loc_parser::validate_loc_file_structure(valid_content).is_empty());

        let diags = loc_parser::validate_loc_file_structure(missing_header_content);
        assert!(!diags.is_empty());
        assert_eq!(diags[0].code.as_deref(), Some("missing_language_header"));
    }
}
