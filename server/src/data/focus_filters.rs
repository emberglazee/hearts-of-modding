//! Focus search-filter knowledge (base-game set + lookup helpers).
//!
//! Engine facts (wiki `national-focus-modding.md` + vanilla corpus verified):
//! - `search_filters = { FOCUS_FILTER_X FOCUS_FILTER_Y }` inside focus /
//!   shared_focus blocks; values are bare whitespace-separated identifiers.
//! - A filter is not defined in any file: it is created dynamically per
//!   focus tree, using sprite `GFX_<FILTER_NAME>` and loc key `<FILTER_NAME>`.
//! - Unknown filter names do NOT error in the game log — the filter simply
//!   never renders in the focus-tree search menu (vanilla itself ships three
//!   typo'd filters that silently no-op: INDUSTRIAL, BALANCE_OF_POWERS,
//!   MISSIOLINI). Validation severity is therefore WARNING, not ERROR.

/// The base-game filter set from the wiki's "every base game focus filter"
/// table, plus DLC-country filters that appear throughout vanilla script.
pub(crate) const BASE_FOCUS_FILTERS: &[&str] = &[
    "FOCUS_FILTER_POLITICAL",
    "FOCUS_FILTER_RESEARCH",
    "FOCUS_FILTER_INDUSTRY",
    "FOCUS_FILTER_STABILITY",
    "FOCUS_FILTER_WAR_SUPPORT",
    "FOCUS_FILTER_MANPOWER",
    "FOCUS_FILTER_ANNEXATION",
    "FOCUS_FILTER_HISTORICAL",
    "FOCUS_FILTER_INTERNATIONAL_TRADE",
    "FOCUS_FILTER_ARMY_XP",
    "FOCUS_FILTER_NAVY_XP",
    "FOCUS_FILTER_AIR_XP",
    "FOCUS_FILTER_TFV_AUTONOMY",
    "FOCUS_FILTER_POLITICAL_CHARACTER",
    "FOCUS_FILTER_MILITARY_CHARACTER",
    "FOCUS_FILTER_INTERNAL_AFFAIRS",
    "FOCUS_FILTER_FRA_POLITICAL_VIOLENCE",
    "FOCUS_FILTER_PROPAGANDA",
    "FOCUS_FILTER_FRA_OCCUPATION_COST",
    "FOCUS_FILTER_CHI_INFLATION",
    "FOCUS_FILTER_BALANCE_OF_POWER",
    "FOCUS_FILTER_SWI_MILITARY_READINESS",
    "FOCUS_FILTER_USA_CONGRESS",
    "FOCUS_FILTER_MEX_CHURCH_AUTHORITY",
    "FOCUS_FILTER_MEX_CAUDILLO_REBELLION",
    "FOCUS_FILTER_SPA_CIVIL_WAR",
    "FOCUS_FILTER_SPA_CARLIST_UPRISING",
    // DLC-country filters used across vanilla national_focus files
    "FOCUS_FILTER_TUR_KURDISTAN",
    "FOCUS_FILTER_TUR_KEMALISM",
    "FOCUS_FILTER_TUR_TRADITIONALISM",
    "FOCUS_FILTER_GRE_DEBT_TO_IFC",
    "FOCUS_FILTER_SOV_POLITICAL_PARANOIA",
    "FOCUS_FILTER_ITA_MISSIOLINI",
    "FOCUS_FILTER_INNER_CIRCLE",
];

/// True when `name` is a base-game focus filter (case-insensitive — HOI4
/// script keys are case-insensitive and mods occasionally write
/// `focus_filter_political`).
pub(crate) fn is_base_filter(name: &str) -> bool {
    BASE_FOCUS_FILTERS
        .iter()
        .any(|f| f.eq_ignore_ascii_case(name))
}

/// True when the value has the shape of a filter reference. Anything else
/// inside `search_filters` is left unvalidated (false negatives over false
/// positives — a mod could technically reference an engine-internal name).
pub(crate) fn looks_like_filter(name: &str) -> bool {
    name.len() > "FOCUS_FILTER_".len()
        && name
            .get(.."FOCUS_FILTER_".len())
            .is_some_and(|p| p.eq_ignore_ascii_case("FOCUS_FILTER_"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_lookup_is_case_insensitive() {
        assert!(is_base_filter("FOCUS_FILTER_POLITICAL"));
        assert!(is_base_filter("focus_filter_political"));
        assert!(!is_base_filter("FOCUS_FILTER_MY_MOD"));
    }

    #[test]
    fn filter_shape_detection() {
        assert!(looks_like_filter("FOCUS_FILTER_POLITICAL"));
        assert!(looks_like_filter("FOCUS_FILTER_MY_MOD"));
        assert!(!looks_like_filter("FOCUS_FILTER_"));
        assert!(!looks_like_filter("political_power"));
    }
}
