use crate::parser::parser;
use crate::rules::ValidationRule;
use crate::rules::visitor::{AstVisitor, walk_script};
use crate::scope::scope::Scope;
use crate::test_support::TestCtx;
use crate::utils::lsp_convert::RangeMapper;
use tower_lsp_server::ls_types::{Diagnostic, NumberOrString};

/// Walk `input` from Global scope with V2ScopeRule and return diagnostics.
///
/// Global (not Country) is deliberate: it proves `activation` itself pushes
/// Country — a Country trigger inside must be silent only if the push works.
fn walk_global(input: &str) -> Vec<Diagnostic> {
    let ctx_builder = TestCtx::new().with_scope_validation(true);

    let (script, _) = parser::parse_script(input);
    let range_mapper = RangeMapper::new(&script.source);
    let ctx = ctx_builder.build_context(
        "/mod/common/decisions/test.txt",
        &script.source,
        &range_mapper,
    );

    let mut visitors: Vec<Box<dyn AstVisitor>> = Vec::new();
    let rules: Vec<Box<dyn ValidationRule>> = vec![Box::new(crate::rules::v2_scope::V2ScopeRule)];
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

fn hom004(diags: &[Diagnostic]) -> Vec<&Diagnostic> {
    diags
        .iter()
        .filter(|d| matches!(&d.code, Some(NumberOrString::String(c)) if c == "HOM004"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn country_trigger_inside_activation_is_silent() {
        // activation pushes Country: has_country_flag must not flag even
        // though the walk starts at Global scope.
        let diags = walk_global("activation = { has_country_flag = my_flag }");
        assert!(
            hom004(&diags).is_empty(),
            "Country trigger inside activation must be silent, got {:#?}",
            diags,
        );
    }

    #[test]
    fn character_trigger_inside_activation_flags() {
        // Locks the resolved scope: a Character/Unit-only trigger inside
        // activation (Country) is a mismatch.
        let diags = walk_global("activation = { has_trait = my_trait }");
        let hits = hom004(&diags);
        assert_eq!(
            hits.len(),
            1,
            "Character-only trigger inside activation must flag HOM004, got {:#?}",
            diags,
        );
        assert!(
            hits[0].message.contains("has_trait"),
            "Unexpected message: {}",
            hits[0].message,
        );
    }

    #[test]
    fn activation_key_itself_is_silent() {
        // Structural scope-pushers never generate HOM004 themselves.
        let diags = walk_global("activation = { always = yes }");
        assert!(
            hom004(&diags).is_empty(),
            "The activation key itself must not flag, got {:#?}",
            diags,
        );
    }
}
