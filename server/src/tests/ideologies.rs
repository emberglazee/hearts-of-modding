use crate::rules::ideologies::{IdeologyRule, IdeologyVisitor};
use crate::scope::scope::Scope;
use tower_lsp_server::ls_types::{Diagnostic, NumberOrString};

const IDEOLOGY_SEED: &str = r#"ideologies = {
	democratic = {
		types = {
			liberalism = {
			}
			conservatism = {
			}
		}
	}
	communism = {
		types = {
			marxism = {
			}
			stalinism = {
			}
		}
	}
	fascism = {
		types = {
			nazism = {
			}
		}
	}
	neutrality = {
		types = {
			despotism = {
			}
		}
	}
}"#;

/// Run the ruling-party visitor (plus the untouched `IdeologyRule`, to lock
/// its behaviour) over `input` with the seed ideologies above.
fn run_visitor(input: &str) -> Vec<Diagnostic> {
    let ctx = crate::test_support::TestCtx::new()
        .with_file("/mod/common/ideologies/00_test.txt", IDEOLOGY_SEED);
    ctx.walk(
        input,
        "/mod/common/national_focus/test.txt",
        Scope::Country,
        vec![Box::new(IdeologyRule)],
        vec![IdeologyVisitor::visitor()],
    )
}

fn ruling_party_diags(diags: &[Diagnostic]) -> Vec<&Diagnostic> {
    diags
        .iter()
        .filter(|d| matches!(&d.code, Some(NumberOrString::String(c)) if c == "HOM3023"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_group_is_silent() {
        for group in ["democratic", "communism"] {
            let diags = run_visitor(&format!("set_politics = {{ ruling_party = {} }}", group));
            assert!(
                ruling_party_diags(&diags).is_empty(),
                "Expected zero HOM3023 for parent group '{}', got {:#?}",
                group,
                diags,
            );
        }
    }

    #[test]
    fn sub_ideology_is_flagged_with_parent() {
        let diags = run_visitor("set_politics = { ruling_party = stalinism }");
        let hits = ruling_party_diags(&diags);
        assert_eq!(
            hits.len(),
            1,
            "Expected one HOM3023 for sub-ideology, got {:#?}",
            diags,
        );
        assert!(
            hits[0].message.contains("sub-ideology") && hits[0].message.contains("communism"),
            "Message should name the sub-ideology and its parent group, got: {}",
            hits[0].message,
        );
        assert_eq!(
            hits[0].data,
            Some(serde_json::Value::String("communism".to_string())),
            "Sub-ideology diag must carry the parent group in `data` for the quick-fix, got: {:?}",
            hits[0].data,
        );
    }

    #[test]
    fn unknown_token_is_flagged() {
        let diags = run_visitor("set_politics = { ruling_party = blorgism }");
        let hits = ruling_party_diags(&diags);
        assert_eq!(
            hits.len(),
            1,
            "Expected one HOM3023 for unknown token, got {:#?}",
            diags,
        );
        assert!(
            hits[0].message.contains("Unknown ideology group"),
            "Unexpected message: {}",
            hits[0].message,
        );
        assert!(
            hits[0].data.is_none(),
            "Unknown-token diag must carry no `data` (no quick-fix target), got: {:?}",
            hits[0].data,
        );
    }

    #[test]
    fn dynamic_refs_are_silent() {
        for val in ["ROOT", "FROM", "var:my_ideology", "GER"] {
            let diags = run_visitor(&format!("set_politics = {{ ruling_party = {} }}", val));
            assert!(
                ruling_party_diags(&diags).is_empty(),
                "Expected zero HOM3023 for dynamic ref '{}', got {:#?}",
                val,
                diags,
            );
        }
    }

    #[test]
    fn ruling_party_outside_set_politics_is_ignored() {
        // start_civil_war's ruling_party is the revolt leader tag, not an ideology.
        let diags = run_visitor("start_civil_war = { ideology = communism ruling_party = SIA }");
        assert!(
            ruling_party_diags(&diags).is_empty(),
            "Expected zero HOM3023 outside set_politics, got {:#?}",
            diags,
        );
    }

    #[test]
    fn matching_is_case_insensitive() {
        let diags = run_visitor("set_politics = { ruling_party = Democratic }");
        assert!(
            ruling_party_diags(&diags).is_empty(),
            "Parent group with different casing should pass, got {:#?}",
            diags,
        );
        let diags = run_visitor("set_politics = { ruling_party = Stalinism }");
        assert_eq!(
            ruling_party_diags(&diags).len(),
            1,
            "Sub-ideology with different casing should still flag, got {:#?}",
            diags,
        );
    }

    #[test]
    fn plain_ideology_key_still_accepts_subs() {
        // Locks IdeologyRule behaviour: sub-ideologies remain valid for
        // ideology/has_ideology keys — only the group-only slots are strict.
        let diags = run_visitor("ideology = stalinism");
        let unknown: Vec<_> = diags
            .iter()
            .filter(|d| matches!(&d.code, Some(NumberOrString::String(c)) if c == "HOM002"))
            .collect();
        assert!(
            unknown.is_empty(),
            "ideology = <sub> must stay silent, got {:#?}",
            diags,
        );
    }

    #[test]
    fn set_popularities_keys_are_group_only() {
        let diags = run_visitor("set_popularities = { democratic = 50 neutrality = 50 }");
        assert!(
            ruling_party_diags(&diags).is_empty(),
            "Group keys must pass, got {:#?}",
            diags,
        );
        let diags = run_visitor("set_popularities = { stalinism = 50 }");
        let hits = ruling_party_diags(&diags);
        assert_eq!(hits.len(), 1, "Sub key must flag, got {:#?}", diags);
        assert!(
            hits[0].message.contains("set_popularities"),
            "Unexpected message: {}",
            hits[0].message,
        );
    }

    #[test]
    fn add_popularity_ideology_is_group_only() {
        let diags = run_visitor("add_popularity = { ideology = fascism popularity = 0.1 }");
        assert!(
            ruling_party_diags(&diags).is_empty(),
            "Group must pass, got {:#?}",
            diags,
        );
        let diags = run_visitor("add_popularity = { ideology = ROOT popularity = 0.1 }");
        assert!(
            ruling_party_diags(&diags).is_empty(),
            "Dynamic ref must pass, got {:#?}",
            diags,
        );
        let diags = run_visitor("add_popularity = { ideology = liberalism popularity = 0.1 }");
        assert_eq!(
            ruling_party_diags(&diags).len(),
            1,
            "Sub-ideology must flag, got {:#?}",
            diags,
        );
    }

    #[test]
    fn government_triggers_are_group_only() {
        for trig in ["has_government", "has_ideology_group"] {
            let diags = run_visitor(&format!("{} = democratic", trig));
            assert!(
                ruling_party_diags(&diags).is_empty(),
                "{} with group must pass, got {:#?}",
                trig,
                diags,
            );
            let diags = run_visitor(&format!("{} = GER", trig));
            assert!(
                ruling_party_diags(&diags).is_empty(),
                "{} with tag must pass, got {:#?}",
                trig,
                diags,
            );
            let diags = run_visitor(&format!("{} = marxism", trig));
            assert_eq!(
                ruling_party_diags(&diags).len(),
                1,
                "{} with sub-ideology must flag, got {:#?}",
                trig,
                diags,
            );
        }
    }

    #[test]
    fn civil_war_ideology_still_accepts_subs() {
        // start_civil_war.ideology takes sub-ideologies in vanilla — out of scope.
        let diags = run_visitor("start_civil_war = { ideology = stalinism }");
        assert!(
            ruling_party_diags(&diags).is_empty(),
            "start_civil_war subs must stay silent, got {:#?}",
            diags,
        );
    }
}
