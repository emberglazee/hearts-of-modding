/// Check if a character can be part of a HOI4 script identifier.
pub fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric()
        || c == '_'
        || c == '.'
        || c == ':'
        || c == '@'
        || c == '['
        || c == ']'
        || c == '?'
        || c == '^'
        || c == '$'
        || c == '/'
        || c == '-'
        || c == '\''
        || c == '%'
        || c == '|'
        || c == '*'
}

/// Parse HOI4 script using the CST pipeline, then lower to AST.
/// This is the main entry point for all consumers.
pub fn parse_script(input: &str) -> (crate::parser::ast::Script, Vec<(String, crate::parser::ast::Range)>) {
    crate::parser::cst::parse_and_lower(input)
}

#[cfg(test)]
mod tests {
    use crate::parser::ast::*;

    #[test]
    fn test_parse_basic() {
        let input = r#"
        # This is a test HOI4 script
        country_event = {
            id = test.1
            is_triggered_only = yes
            trigger = {
                tag = GER
                has_war = no
            }
        }
        "#;
        let result = crate::parser::parser::parse_script(input);
        assert!(result.1.is_empty());
        let script = result.0;
        assert_eq!(script.entries.len(), 2); // Comment and Assignment
    }

    #[test]
    fn test_parse_complex() {
        let input = r#"
        modifier = {
            political_power_factor = 0.15
            stability_factor > -0.1
            tag != "ENG"
            [?my_var] = 10
            array^0 = 1
        }
        "#;
        let result = crate::parser::parser::parse_script(input);
        assert!(result.1.is_empty());
        let script = result.0;
        assert_eq!(script.entries.len(), 1);
    }

    #[test]
    fn test_parse_quoted_escapes() {
        let input = r#"title = "Event \"The Great War\" Begins""#;
        let result = crate::parser::parser::parse_script(input);
        assert!(result.1.is_empty());
    }

    #[test]
    fn test_parse_dots_in_key() {
        let input = r#"title = daw.2.t"#;
        let result = crate::parser::parser::parse_script(input);
        assert!(result.1.is_empty());
        let script = result.0;
        if let Entry::Assignment(ass) = &script.entries[0] {
            if let Value::String(s) = &ass.value.value {
                assert_eq!(s, "daw.2.t");
            } else {
                panic!("Value should be a string/identifier");
            }
        }
    }

    #[test]
    fn test_parse_pipe_in_value() {
        let input = r#"custom_effect_tooltip = tech_effect|sp_naval_support_ships_pick_a"#;
        let result = crate::parser::parser::parse_script(input);
        assert!(result.1.is_empty());
    }
}
