#[cfg(test)]
mod tests {
    use crate::loc_parser;

    #[test]
    fn test_loc_comment_dups_snippet() {
        let content = "l_english:
#  ##Leaders##
#   PLC_polski_crafter: \"Polski Crafter\"
#   PLC_pcrafter_leader_desc: \"§LPolski Crafter, who was the greatest admin of Crazy Adventure, has successfully led a march from our homeland. He was the one player who knew from the beginning that we cannot win a fight with the Zombies. After the famous Long March he became a leader of the newborn state, the Red Rose Union.§!\"
#    PLC_pcrafter_general_desc: \"$PLC_pcrafter_leader_desc$\"
#    PLC_pcrafter_political_desc: \"$PLC_pcrafter_leader_desc$\"
#    PLC_pcrafter_chief_desc: \"$PLC_pcrafter_leader_desc$\"
#   PLC_skarabii: \"Skarabii\"
";
        let (parsed, _) = loc_parser::parse_loc_file(content, "test.yml");
        for (k, _v) in parsed.iter() {
            println!("Parsed key: {}", k);
        }
        assert!(!parsed.contains_key("PLC_pcrafter_general_desc"));
        assert_eq!(parsed.len(), 0);
    }
}
