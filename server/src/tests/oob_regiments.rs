use crate::parser::parser;
use crate::rules::ValidationRule;
use crate::rules::oob_regiments::OobRegimentVisitor;
use crate::rules::visitor::{AstVisitor, walk_script};
use crate::scope::scope::Scope;
use crate::test_support::TestCtx;
use crate::utils::lsp_convert::RangeMapper;
use tower_lsp_server::ls_types::{Diagnostic, NumberOrString};

// ---------------------------------------------------------------------------
// Test runner
// ---------------------------------------------------------------------------

/// Run the OobRegimentVisitor over a script and return all diagnostics.
fn run_oob_visitor(input: &str, unit_types: &[&str]) -> Vec<Diagnostic> {
    let ctx_builder = TestCtx::new().with_unit_types(unit_types);

    let (script, _) = parser::parse_script(input);
    let range_mapper = RangeMapper::new(&script.source);
    let ctx = ctx_builder.build_context(
        "test:///history/units/test_oob.txt",
        &script.source,
        &range_mapper,
    );

    let mut visitors: Vec<Box<dyn AstVisitor>> = vec![OobRegimentVisitor::visitor()];
    let rules: Vec<Box<dyn ValidationRule>> = Vec::new();
    let mut diags = Vec::new();

    walk_script(
        &script.entries,
        &mut visitors,
        &rules,
        &ctx,
        &mut diags,
        Scope::Global,
        false,
    );

    diags
}

/// Filter diagnostics to only HOM3006 (unknown division template).
fn template_diags(diags: &[Diagnostic]) -> Vec<&Diagnostic> {
    diags
        .iter()
        .filter(|d| matches!(&d.code, Some(NumberOrString::String(c)) if c == "HOM3006"))
        .collect()
}

/// Filter diagnostics to only HOM3005 (unknown unit type).
fn unit_type_diags(diags: &[Diagnostic]) -> Vec<&Diagnostic> {
    diags
        .iter()
        .filter(|d| matches!(&d.code, Some(NumberOrString::String(c)) if c == "HOM3005"))
        .collect()
}

/// Filter diagnostics to only HOM3007 (unit type case mismatch).
fn case_mismatch_diags(diags: &[Diagnostic]) -> Vec<&Diagnostic> {
    diags
        .iter()
        .filter(|d| matches!(&d.code, Some(NumberOrString::String(c)) if c == "HOM3007"))
        .collect()
}

// ---------------------------------------------------------------------------
// Tests: division template cross-reference
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── Positive: template defined and referenced ─────────────────────

    const OOB_WITH_VALID_TEMPLATE: &str = r#"division_template = {
	name = "Infantry Division"
	regiments = {
		infantry = { x = 0 y = 0 }
		infantry = { x = 0 y = 1 }
	}
	support = {
		engineer = { x = 0 y = 0 }
	}
}
units = {
	division = {
		name = "1st Infantry Division"
		location = 6488
		division_template = "Infantry Division"
	}
}"#;

    #[test]
    fn valid_template_reference_no_diagnostic() {
        let diags = run_oob_visitor(OOB_WITH_VALID_TEMPLATE, &["infantry", "engineer"]);
        let tdiags = template_diags(&diags);

        assert!(
            tdiags.is_empty(),
            "Expected zero HOM3006 for a valid template reference, got {}:\n{:#?}",
            tdiags.len(),
            tdiags,
        );
    }

    // ── Negative: template referenced but not defined ─────────────────

    const OOB_WITH_INVALID_TEMPLATE: &str = r#"division_template = {
	name = "Infantry Division"
	regiments = {
		infantry = { x = 0 y = 0 }
	}
}
units = {
	division = {
		name = "1st Cavalry Division"
		location = 6488
		division_template = "Cavalry Division"
	}
}"#;

    #[test]
    fn unknown_template_reference_emits_diagnostic() {
        let diags = run_oob_visitor(OOB_WITH_INVALID_TEMPLATE, &["infantry"]);
        let tdiags = template_diags(&diags);

        assert_eq!(
            tdiags.len(),
            1,
            "Expected 1 HOM3006 for unknown template ref, got {}:\n{:#?}",
            tdiags.len(),
            tdiags,
        );

        assert!(
            tdiags[0].message.contains("Cavalry Division"),
            "Message should reference the missing template name, got: {:?}",
            tdiags[0].message,
        );
    }

    // ── Multiple templates, some valid some not ───────────────────────

    const OOB_MIXED_TEMPLATES: &str = r#"division_template = {
	name = "Infantry Division"
	regiments = {
		infantry = { x = 0 y = 0 }
	}
}
division_template = {
	name = "Armored Division"
	regiments = {
		light_armor = { x = 0 y = 0 }
	}
}
units = {
	division = {
		division_template = "Infantry Division"
		location = 1
	}
	division = {
		division_template = "Armored Division"
		location = 2
	}
	division = {
		division_template = "Mountain Division"
		location = 3
	}
	division = {
		division_template = "Infantry Division"
		location = 4
	}
}"#;

    #[test]
    fn mixed_template_refs_only_flag_unknown() {
        let diags = run_oob_visitor(OOB_MIXED_TEMPLATES, &["infantry", "light_armor"]);
        let tdiags = template_diags(&diags);

        assert_eq!(
            tdiags.len(),
            1,
            "Expected 1 HOM3006 out of 4 refs (Infantry √, Armored √, Mountain ✗, Infantry √), got {}:\n{:#?}",
            tdiags.len(),
            tdiags,
        );

        assert!(
            tdiags[0].message.contains("Mountain Division"),
            "Should flag only the missing 'Mountain Division', got: {:?}",
            tdiags[0].message,
        );
    }

    // ── No diagnostic when no units block at all ──────────────────────

    #[test]
    fn only_template_defs_no_diagnostics() {
        let diags = run_oob_visitor(
            r#"division_template = {
	name = "Infantry Division"
	regiments = { infantry = { x = 0 y = 0 } }
}"#,
            &["infantry"],
        );
        let tdiags = template_diags(&diags);
        assert!(
            tdiags.is_empty(),
            "No units block → no references → no HOM3006"
        );
    }

    // ── Regiment validation still works alongside template checks ─────

    const OOB_WITH_UNKNOWN_REGIMENT: &str = r#"division_template = {
	name = "Test Division"
	regiments = {
		infantry = { x = 0 y = 0 }
		unicorn_brigade = { x = 1 y = 0 }
	}
}
units = {
	division = {
		division_template = "Test Division"
		location = 1
	}
}"#;

    #[test]
    fn regiment_validation_still_fires_alongside_template_check() {
        let diags = run_oob_visitor(OOB_WITH_UNKNOWN_REGIMENT, &["infantry"]);
        let udiags = unit_type_diags(&diags);

        assert_eq!(
            udiags.len(),
            1,
            "Expected 1 HOM3005 for 'unicorn_brigade', got {}:\n{:#?}",
            udiags.len(),
            udiags,
        );
        assert!(
            udiags[0].message.contains("unicorn_brigade"),
            "Should flag unknown unit type 'unicorn_brigade', got: {:?}",
            udiags[0].message,
        );

        // Also verify template ref is valid
        let tdiags = template_diags(&diags);
        assert!(
            tdiags.is_empty(),
            "Template ref to 'Test Division' should be valid, got HOM3006"
        );
    }

    // ── Division_template reference outside units block ────────────────
    // A division_template = "..." key-value at file level (not in a
    // template def block, not in a unit context) should NOT produce
    // a template reference diagnostic (it's probably something else).

    const OOB_STRANGE_DIVISION_TEMPLATE: &str = r#"division_template = {
	name = "Infantry Division"
	regiments = { infantry = { x = 0 y = 0 } }
}
division_template = "something weird" # This is a parser error, not a template ref
units = {
	division = {
		division_template = "Infantry Division"
		location = 1
	}
}"#;

    #[test]
    fn bare_division_template_scalar_not_treated_as_ref() {
        let diags = run_oob_visitor(OOB_STRANGE_DIVISION_TEMPLATE, &["infantry"]);
        let tdiags = template_diags(&diags);

        // The bare division_template = "something weird" is not inside
        // units > division, so it should NOT be collected as a ref.
        // The valid ref should pass. Zero HOM3006 expected.
        assert!(
            tdiags.is_empty(),
            "Bare division_template scalar should not be treated as a ref, got HOM3006:\n{:#?}",
            tdiags,
        );
    }

    // ── Case-mismatched unit types get a HINT instead of WARNING ────

    const OOB_WITH_MIXED_CASE_REGIMENT: &str = r#"division_template = {
	name = "Test Division"
	regiments = {
		Infantry = { x = 0 y = 0 }
		ENGINEER = { x = 0 y = 1 }
		artillery_brigade = { x = 1 y = 0 }
		unicorn_brigade = { x = 1 y = 1 }
	}
}
units = {
	division = {
		division_template = "Test Division"
		location = 1
	}
}"#;

    #[test]
    fn case_mismatched_unit_type_gets_hint_not_warning() {
        let diags = run_oob_visitor(
            OOB_WITH_MIXED_CASE_REGIMENT,
            &["infantry", "engineer", "artillery_brigade"],
        );
        let case_diags = case_mismatch_diags(&diags);
        let unknown_diags = unit_type_diags(&diags);

        // "Infantry" → should match "infantry" → HOM3007 hint
        // "ENGINEER" → should match "engineer" → HOM3007 hint
        // "artillery_brigade" → exact match → no diagnostic
        // "unicorn_brigade" → completely unknown → HOM3005 warning

        assert_eq!(
            case_diags.len(),
            2,
            "Expected 2 HOM3007 hints (Infantry, ENGINEER), got {}:\n{:#?}",
            case_diags.len(),
            case_diags,
        );

        assert!(
            case_diags[0].message.contains("Infantry"),
            "First hint should mention 'Infantry', got: {:?}",
            case_diags[0].message,
        );
        assert!(
            case_diags[0].message.contains("infantry"),
            "First hint should suggest lowercase 'infantry', got: {:?}",
            case_diags[0].message,
        );

        assert!(
            case_diags[1].message.contains("ENGINEER"),
            "Second hint should mention 'ENGINEER', got: {:?}",
            case_diags[1].message,
        );
        assert!(
            case_diags[1].message.contains("engineer"),
            "Second hint should suggest lowercase 'engineer', got: {:?}",
            case_diags[1].message,
        );

        assert_eq!(
            unknown_diags.len(),
            1,
            "Expected 1 HOM3005 warning for 'unicorn_brigade', got {}:\n{:#?}",
            unknown_diags.len(),
            unknown_diags,
        );

        // Confirm severity is HINT, not WARNING
        assert_eq!(
            case_diags[0].severity,
            Some(tower_lsp_server::ls_types::DiagnosticSeverity::HINT),
            "Case-mismatch should be HINT severity, got: {:?}",
            case_diags[0].severity,
        );
    }
}
