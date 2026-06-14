use crate::parser::ast;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::ScopeStack;
use crate::utils::lsp_convert::ast_range_to_lsp;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Checks that string values assigned to `name`, `desc`, `text`,
/// and `title` keys have corresponding localization entries.
///
/// Uses heuristics to avoid flagging literals (space-containing strings,
/// capitalized non-all-caps strings, pure numbers) and respects the
/// `# ignore` comment suppression and `ignored_loc_regex` config.
pub(crate) struct LocalizationRule;

impl ValidationRule for LocalizationRule {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        _pushed_scope: bool,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key_lower = ass.key_text(ctx.source).to_ascii_lowercase();
        if key_lower != "name" && key_lower != "desc" && key_lower != "text" && key_lower != "title"
        {
            return;
        }

        let Some(val) = ass.value.value.as_str(ctx.source) else {
            return;
        };

        let mut should_flag = true;

        // 1. Basic heuristics: space, empty, all-numeric → literal
        if val.contains(' ') || val.is_empty() || val.chars().all(|c| c.is_numeric()) {
            should_flag = false;
        }

        // 2. Casing heuristic: starts with uppercase but isn't all-caps → likely literal
        if should_flag && val.chars().next().is_some_and(|c| c.is_uppercase()) {
            let all_caps = val.chars().all(|c| !c.is_lowercase());
            if !all_caps {
                should_flag = false;
            }
        }

        // 3. Comment suppression (# ignore on same line)
        if should_flag {
            for (comment_text, range) in ctx.comments {
                if range.start_line == ass.key_range.start_line {
                    if comment_text
                        .resolve(ctx.source)
                        .to_ascii_lowercase()
                        .contains("ignore")
                    {
                        should_flag = false;
                        break;
                    }
                }
            }
        }

        if should_flag {
            if !ctx.loc.contains_key(val) {
                // Double-check: the key might be stored with a version suffix like ":0"
                // Instead of iterating the entire 162k-entry DashMap (which is O(N) per
                // missing key), check a few common version numbers directly.
                let version_suffixed = (0..=5).any(|v| {
                    let target = format!("{}:{}", val, v);
                    ctx.loc.contains_key(target.as_str())
                });
                if !version_suffixed {
                    // Final check against regex
                    let is_regex_ignored = ctx.ignored_loc_regex.iter().any(|re| re.is_match(val));

                    if !is_regex_ignored {
                        diags.push(Diagnostic {
                            range: ast_range_to_lsp(&ass.value.range),
                            severity: Some(DiagnosticSeverity::HINT),
                            message: format!(
                                "Missing localization key: '{}' (or literal name)",
                                val
                            ),
                            code: Some(NumberOrString::String(
                                crate::validation::advanced_validation::MISSING_LOCALIZATION
                                    .to_string(),
                            )),
                            source: Some("Hearts of Modding".to_string()),
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }
}
