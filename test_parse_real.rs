use std::collections::HashMap;

fn main() {
    let content = r#"defined_text = {
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
    let script = server::parser::parse_script(content).unwrap();
    let mut map = HashMap::new();
    server::scripted_loc_scanner::find_scripted_locs_in_entries(&script.entries, "test", &mut map);
    println!("{:?}", map);
}
