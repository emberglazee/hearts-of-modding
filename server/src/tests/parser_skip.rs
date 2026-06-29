#[cfg(test)]
mod tests {
    use crate::parser::parser;

    #[test]
    fn test_parser_recovery_multibyte() {
        let input = "§test = yes";
        let (script, errors) = parser::parse_script(input);
        println!("Errors: {:?}", errors);
        println!("Script entries: {:?}", script.entries);
    }

    #[test]
    fn test_parse_minimal_characters_block() {
        // Minimal reproduction of the vanilla ARG.txt structure:
        // a `characters = { ... }` block containing nested character
        // assignments, including a comma-separated traits list that
        // previously caused cascading parse failures.
        let input = "characters = {\n\
            \tARG_leader = {\n\
                \t\tname = \"Test Leader\"\n\
                \t\tcorps_commander = {\n\
                    \t\t\ttraits = { trait_cautious, inflexible_strategist }\n\
                    \t\t\tskill = 2\n\
                    \t\t}\n\
                \t}\n\
            \tARG_general = {\n\
                \t\tname = \"Test General\"\n\
                \t}\n\
            }\n";
        let (script, errors) = parser::parse_script(input);
        assert!(errors.is_empty(), "Got parsing errors: {:?}", errors);
        assert_eq!(
            script.entries.len(),
            1,
            "Should parse as single top-level assignment"
        );
    }
}
