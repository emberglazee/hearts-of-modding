use crate::parser::ast;

/// Diagnostic codes for advanced validation
pub const PARSE_ERROR: &str = "HOM001";
pub const UNKNOWN_TRIGGER: &str = "HOM002";
#[allow(dead_code)]
pub const UNKNOWN_EFFECT: &str = "HOM003";
pub const SCOPE_MISMATCH: &str = "HOM004";
pub const MISSING_LOCALIZATION: &str = "HOM005";

pub const BUILDING_LEVEL_EXCEEDS_MAX: &str = "HOM1002";
pub const CHARACTER_SKILL_EXCEEDS_MAX: &str = "HOM1004";
pub const CHARACTER_SUBSKILL_EXCEEDS_PRACTICAL: &str = "HOM1006";
pub const CHARACTER_NEGATIVE_SKILL: &str = "HOM1007";
pub const VICTORY_POINT_PROVINCE_NOT_IN_STATE: &str = "HOM2001";
// ── Map/state cross-validation (HOM2002–HOM2009) ──
// Text-side port of the ANKA map editor's integrity checks that need no
// bitmap access (ideas clean-roomed, no code copied — ANKA ships no LICENSE
// file). All WARNING: none of these has yet been pinned against error.log,
// and the LSP's standing rule is false negatives over false positives.
/// Sea province listed in a state's `provinces`. Lakes are legal members
/// (118 in vanilla, e.g. IJsselmeer in Friesland) — only `sea` fires.
pub const SEA_PROVINCE_IN_STATE: &str = "HOM2002";
/// One province claimed by two `history/states` files (engine assigns it to
/// one; the other state's content silently breaks).
pub const PROVINCE_IN_TWO_STATES: &str = "HOM2003";
/// Same province listed twice in one state's `victory_points`.
pub const DUPLICATE_VICTORY_POINT: &str = "HOM2004";
/// Province-keyed building (`1234 = { naval_base = 1 }`) on a province that
/// is not a member of the state being validated.
pub const PROVINCE_BUILDING_OUTSIDE_STATE: &str = "HOM2005";
/// One province claimed by two `map/strategicregions` files.
pub const PROVINCE_IN_TWO_STRATEGIC_REGIONS: &str = "HOM2006";
/// `only_costal`-gated building (vanilla spelling, `only_coastal` accepted
/// too) placed on a non-coastal province.
pub const COASTAL_BUILDING_ON_INLAND: &str = "HOM2007";
/// State with an empty `provinces` list (usually a merge leftover).
pub const EMPTY_STATE: &str = "HOM2008";
/// State member province with no row in `map/definition.csv`.
pub const STATE_UNKNOWN_PROVINCE: &str = "HOM2009";
pub const ACHIEVEMENT_MISSING_LOCALIZATION: &str = "HOM3001";
pub const ABILITY_MISSING_LOCALIZATION: &str = "HOM3002";
pub const ABILITY_MISSING_REQUIRED_FIELD: &str = "HOM3003";
pub const ABILITY_MISSING_AI_LOGIC: &str = "HOM3004";
pub const UNKNOWN_UNIT_TYPE: &str = "HOM3005";
pub const UNIT_TYPE_CASE_MISMATCH: &str = "HOM3007";
pub const UNKNOWN_DIVISION_TEMPLATE: &str = "HOM3006";
pub const MISSING_EVENT_NAMESPACE: &str = "HOM3008";
pub const NON_INTEGER_EVENT_ID: &str = "HOM3009";
pub const EVENT_ID_TOO_LARGE: &str = "HOM3010";
pub const DUPLICATE_EVENT_ID: &str = "HOM3011";
pub const DUPLICATE_EVENT_NAMESPACE: &str = "HOM3012";

// ── Event option & structure validation (HOM3013–HOM3020) ──
pub const EVENT_MISSING_OPTION_NAME: &str = "HOM3013";
#[allow(dead_code)]
pub const TRIGGERED_ONLY_WITH_MTTH: &str = "HOM3014";
#[allow(dead_code)]
pub const NEWS_MAJOR_FIRE_ONCE: &str = "HOM3015";
pub const EVENT_MISSING_TITLE: &str = "HOM3016";
pub const EVENT_OPTION_MISSING_AI_CHANCE: &str = "HOM3017";
pub const EVENT_MISSING_TITLE_LOC: &str = "HOM3018";
pub const EVENT_MISSING_DESC_LOC: &str = "HOM3019";
pub const EVENT_PICTURE_SPRITE_NOT_FOUND: &str = "HOM3020";
pub const EVENTS_SUBDIRECTORY_FILE: &str = "HOM3021";
pub const BROKEN_EVENT_REFERENCE: &str = "HOM3022";

pub const PORTRAIT_UNKNOWN_GFX: &str = "HOM4001";
pub const UNKNOWN_COUNTRY_METADATA_GFX: &str = "HOM4002";
pub const IDEA_PICTURE_NOT_FOUND: &str = "HOM4003";
pub const IDEA_CASE_MISMATCH: &str = "HOM4004";
/// Country tag uses a reserved engine keyword (NOT/AND/TAG/OOB/LOG/NUM/RED).
/// Per wiki the engine still loads the tag but custom map modes break
/// (RED always 0), so severity is WARNING not ERROR. Appears at the
/// definition site in `common/country_tags`/`common/countries`/`history/countries`.
pub const RESERVED_COUNTRY_TAG: &str = "HOM4005";
pub const UNKNOWN_STATE_CATEGORY: &str = "HOM5001";
pub const UNKNOWN_RESOURCE: &str = "HOM5002";
pub const UNKNOWN_BUILDING: &str = "HOM5003";
pub const UNKNOWN_NAVAL_TERRAIN: &str = "HOM5004";
pub const UNKNOWN_PROVINCE_TERRAIN: &str = "HOM5005";

// ── Decision validation codes (HOM5006–HOM5009) ──
pub const UNDECLARED_DECISION_CATEGORY: &str = "HOM5006";
pub const CATEGORY_KEY_IN_DECISION: &str = "HOM5007";
pub const DECISION_MISSING_COMPLETE_EFFECT: &str = "HOM5008";
pub const DECISION_DUAL_COST: &str = "HOM5009";

// ── Focus validation (HOM5010–…) ──
/// Unknown focus search filter (`search_filters = { FOCUS_FILTER_TYPO }`).
/// The engine never errors on these — the filter silently fails to render in
/// the focus-tree search menu — so severity is WARNING. A non-base filter is
/// valid when a `GFX_<name>` sprite exists (that is how mods define filters).
pub const UNKNOWN_FOCUS_SEARCH_FILTER: &str = "HOM5010";

// ── Syntax validation (HOM6000–HOM6004) ──
/// Block implicitly closed at end-of-file (Clausewitz engine accepts this)
pub const IMPLICIT_EOF_CLOSE: &str = "HOM6000";

/// Extra closing brace `}` that doesn't match any open block — the engine
/// silently discards it, but it's worth flagging as INFO for cleanliness.
pub const STRAY_BRACE: &str = "HOM6001";

/// Section sign `§` in an unquoted script value — the engine either silently
/// corrupts the value to 0 or errors out depending on position. Check the
/// actual file content and consider replacing or quoting it.
/// Added because this was encountered in v1.18.3 `ship_hull_carrier.txt:708`.
pub const SECTION_SIGN_IN_VALUE: &str = "HOM6002";

/// Double assignment on one line (`key = value = value`) — a modder slip the
/// engine does NOT recover from (Clausewitz throws "Unexpected token: =").
/// We recover the AST (last value wins) so the file doesn't cascade, but still
/// surface a specific, descriptive ERROR so the modder knows to fix it.
pub const DOUBLE_ASSIGNMENT: &str = "HOM6003";

/// Leading-dot number (`.5`) — the engine REJECTS these (empirically:
/// `Malformed token: .5`). The parser keeps `.5` as a String (no cascade) but
/// surfaces a specific ERROR telling the modder to write `0.5`.
pub const MALFORMED_LEADING_DOT_NUMBER: &str = "HOM6004";

/// Localization file whose on-disk bytes do not start with EXACTLY ONE UTF-8
/// BOM — either none (the game may fail to load the file's strings) or two or
/// more (LLM-generated files routinely double/triple it; stray U+FEFFs corrupt
/// the language header). ERROR (not a warning): the failure is silent, the
/// cause invisible in editors (VS Code strips/hides BOM bytes), new modders
/// hit it constantly, and the fix is simply re-saving with encoding
/// "UTF-8 with BOM".
pub const LOC_BOM_ISSUE: &str = "HOM6005";

#[derive(Debug, Clone)]
/// Kept for public API compatibility; no longer directly constructed by validation rules.
#[allow(dead_code)]
pub struct ValidationDiagnostic {
    pub range: ast::Range,
    pub severity: ast::DiagnosticSeverity,
    pub message: String,
    pub code: String,
    #[allow(dead_code)]
    pub fix_suggestion: Option<String>,
    pub related_information: Vec<ast::DiagnosticRelatedInformation>,
    pub tags: Vec<ast::DiagnosticTag>,
}
