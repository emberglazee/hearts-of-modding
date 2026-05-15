#[cfg(test)]
mod tests {
    use crate::parser;

    #[test]
    fn test_parser_recovery_multibyte() {
        let input = "§test = yes";
        let (script, errors) = parser::parse_script(input);
        println!("Errors: {:?}", errors);
        println!("Script entries: {:?}", script.entries);
    }
}
