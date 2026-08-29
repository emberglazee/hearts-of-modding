use crate::parser::ast;
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::ScopeStack;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// Validates country tag references and dynamic/static tag ratios.
///
/// Per-entry: checks `tag`, `original_tag`, and `original_tag_to_check`
/// values against known country tags (allowing scope refs and var refs).
/// Block-level: warns if the file is in `common/country_tags/` and has
/// insufficient dynamic tags for civil war support.
pub(crate) struct CountryTagRule;

impl ValidationRule for CountryTagRule {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        scope: &ScopeStack,
        _pushed_scope: bool,
        diags: &mut Vec<Diagnostic>,
    ) {
        // ── Reserved tag definition (HOM4005) ──
        // `RED = "countries/Red.txt"` etc. still loads in engine but breaks
        // map modes (wiki). Warn at the *definition* site, not at every use.
        // Covers all three definition locations:
        //   common/country_tags/*.txt  (key = tag)
        //   common/countries/*.txt     (key = tag for cosmetic/underlay files)
        //   history/countries/*.txt    (filename = tag, handled in check_block)
        let key = ass.key_text(ctx.source);
        let key_upper = key.to_ascii_uppercase();
        if crate::scanner::country_scanner::is_reserved_tag(&key_upper)
            && crate::scanner::country_scanner::is_valid_tag(&key_upper)
        {
            // Only `common/country_tags/*.txt` defines tags via assignment
            // keys (`TAG = "countries/..."`). Keys inside `history/countries/`
            // and `common/countries/` (e.g. `oob = "RDM_648"`,
            // `graphical_culture = ...`) are NOT tag definitions — flagging
            // them produced false positives (RDM file `oob` -> HOM4005 OOB).
            // Filename-based definitions (history) are handled in check_block.
            let uri_lower = ctx.uri.to_ascii_lowercase();
            let is_tag_def_file = uri_lower.contains("/common/country_tags/")
                || uri_lower.contains("\\common\\country_tags\\");
            if is_tag_def_file {
                // Every key in country_tags *is* a tag definition.
                let msg = match key_upper.as_str() {
                    "RED" => {
                        "Country tag 'RED' is reserved for custom map modes (used as `red` variable). The tag will still load but every custom map mode will render with red=0. Prefer a non-reserved tag (e.g. RMT, REM)."
                    }
                    "NOT" | "AND" => {
                        "Country tag is a reserved flow-control keyword (NOT/AND) and will break trigger parsing."
                    }
                    "TAG" => {
                        "Country tag 'TAG' collides with the `tag` trigger and will break country checks."
                    }
                    "OOB" => {
                        "Country tag 'OOB' collides with `oob` in history files and will break OOB loading."
                    }
                    "LOG" => "Country tag 'LOG' collides with the `log` effect/trigger.",
                    "NUM" => {
                        "Country tag 'NUM' collides with array `NUM` and will break the resistance system."
                    }
                    _ => "Country tag uses a reserved keyword and may break engine features.",
                };
                diags.push(Diagnostic {
                    range: ctx.range(&ass.key_range),
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: msg.to_string(),
                    code: Some(NumberOrString::String(
                        crate::validation::advanced_validation::RESERVED_COUNTRY_TAG.to_string(),
                    )),
                    source: Some("Hearts of Modding".to_string()),
                    ..Default::default()
                });
            }
        }

        let key_lower = ass.key_text(ctx.source).to_ascii_lowercase();

        // Skip 'tag' inside Idea scope — ideas use 'tag = { ... }' differently
        if key_lower == "tag" && scope.current() == crate::scope::scope::Scope::Idea {
            return;
        }

        if key_lower != "tag" && key_lower != "original_tag" && key_lower != "original_tag_to_check"
        {
            return;
        }

        let Some(val) = ass.value.value.as_str(ctx.source) else {
            return;
        };

        // Allow scope references (ROOT, FROM, PREV, etc.)
        let is_scope_ref = matches!(
            val.to_uppercase().as_str(),
            "ROOT"
                | "FROM"
                | "PREV"
                | "THIS"
                | "PREVPREV"
                | "PREVPREVPREV"
                | "PREVPREVPREVPREV"
                | "OWNER"
                | "CONTROLLER"
                | "CAPITAL"
        );
        let is_var_ref = val.starts_with("var:");

        // Any syntactically valid 3-char tag looks like a country tag.
        // Reserved tags (RED etc.) are also valid — engine loads them, so
        // `tag = RED` should warn as unknown only when RED isn't defined,
        // and get a separate HOM4005 about the reservation.
        let looks_like_tag = crate::scanner::country_scanner::is_valid_tag(val);

        if !is_scope_ref
            && !is_var_ref
            && looks_like_tag
            && !ctx.country_tags.contains_key(val)
            && !ctx.tag_aliases.contains_key(val)
        {
            diags.push(Diagnostic {
                range: ctx.range(&ass.value.range),
                severity: Some(DiagnosticSeverity::WARNING),
                message: format!("Unknown country tag: '{}'", val),
                code: Some(NumberOrString::String(
                    crate::validation::advanced_validation::UNKNOWN_TRIGGER.to_string(),
                )),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        }
    }

    fn check_block(
        &self,
        _entries: &[ast::Entry],
        ctx: &ValidationContext,
        diags: &mut Vec<Diagnostic>,
    ) {
        let uri = ctx.uri;
        let uri_lower = uri.to_ascii_lowercase();

        // ── Filename-based reserved tag (history/countries/RED - Name.txt) ──
        // The tag is in the filename, not as an assignment key, so
        // check_assignment cannot see it.
        if uri_lower.contains("/history/countries/") || uri_lower.contains("\\history\\countries\\")
        {
            // Extract filename stem from uri (handles both / and \)
            let filename = uri.rsplit('/').next().unwrap_or(uri);
            let filename = filename.rsplit('\\').next().unwrap_or(filename);
            let stem = filename.trim_end_matches(".txt").trim_end_matches(".TXT");
            if stem.len() >= 3 {
                let tag = stem[..3].to_ascii_uppercase();
                if crate::scanner::country_scanner::is_reserved_tag(&tag)
                    && crate::scanner::country_scanner::is_valid_tag(&tag)
                {
                    let msg = match tag.as_str() {
                        "RED" => {
                            "Country tag 'RED' is reserved for custom map modes (used as `red` variable). The tag will still load but every custom map mode will render with red=0. Prefer a non-reserved tag (e.g. RMT, REM)."
                        }
                        "NOT" | "AND" => {
                            "Country tag is a reserved flow-control keyword (NOT/AND) and will break trigger parsing."
                        }
                        "TAG" => {
                            "Country tag 'TAG' collides with the `tag` trigger and will break country checks."
                        }
                        "OOB" => {
                            "Country tag 'OOB' collides with `oob` in history files and will break OOB loading."
                        }
                        "LOG" => "Country tag 'LOG' collides with the `log` effect/trigger.",
                        "NUM" => {
                            "Country tag 'NUM' collides with array `NUM` and will break the resistance system."
                        }
                        _ => "Country tag uses a reserved keyword and may break engine features.",
                    };
                    diags.push(Diagnostic {
                        range: ctx.range(&ast::Range {
                            start_line: 0,
                            start_col: 0,
                            end_line: 0,
                            end_col: 3,
                        }),
                        severity: Some(DiagnosticSeverity::WARNING),
                        message: msg.to_string(),
                        code: Some(NumberOrString::String(
                            crate::validation::advanced_validation::RESERVED_COUNTRY_TAG
                                .to_string(),
                        )),
                        source: Some("Hearts of Modding".to_string()),
                        ..Default::default()
                    });
                }
            }
        }

        // Only run dynamic-count checks for country_tags files
        if !uri.contains("/common/country_tags/") && !uri.contains("\\common\\country_tags\\") {
            return;
        }

        let ct = ctx.country_tags;
        let total = ct.len();
        let dynamic_count = ct.iter().filter(|t| t.value().dynamic).count();
        let static_count = total - dynamic_count;

        if total > 0 && dynamic_count == 0 {
            diags.push(Diagnostic {
                range: ctx.range(&ast::Range {
                    start_line: 0,
                    start_col: 0,
                    end_line: 0,
                    end_col: 0,
                }),
                severity: Some(DiagnosticSeverity::WARNING),
                message: "No dynamic country tags defined. Civil wars will fail for lack of dynamic tags, potentially causing a crash.".to_string(),
                code: Some(NumberOrString::String("HOM5001".to_string())),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        } else if static_count > 10 && dynamic_count < (static_count / 10).max(3) {
            diags.push(Diagnostic {
                range: ctx.range(&ast::Range {
                    start_line: 0,
                    start_col: 0,
                    end_line: 0,
                    end_col: 0,
                }),
                severity: Some(DiagnosticSeverity::INFORMATION),
                message: format!(
                    "Only {} dynamic tags for {} static tags. Consider adding more dynamic tags for civil wars.",
                    dynamic_count, static_count
                ),
                code: Some(NumberOrString::String("HOM5002".to_string())),
                source: Some("Hearts of Modding".to_string()),
                ..Default::default()
            });
        }
    }
}
