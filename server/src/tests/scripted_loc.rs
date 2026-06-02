#[cfg(test)]
mod tests {
    use crate::scanner::scripted_loc_scanner::find_scripted_locs_in_entries;
    use std::collections::HashMap;

    #[test]
    fn test_find_scripted_locs() {
        let content = r#"
defined_text = {
	name = DBUG_show_lar_decisions
	text = {
		trigger = {
			NOT = { has_dlc = "La Resistance" }
		}
		localization_key = DBUG_show_lar_di_decisions
	}
	text = {
		trigger = { has_dlc = "La Resistance" }
		localization_key = DBUG_show_lar_en_decisions
	}
}
        "#;
        let (script, _errors) = crate::parser::parser::parse_script(content);
        let mut map = HashMap::new();
        find_scripted_locs_in_entries(&script.entries, "test", &mut map);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("DBUG_show_lar_decisions"));
    }
}
