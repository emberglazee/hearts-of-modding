use crate::parser::ast;

/// Diagnostic codes for advanced validation
pub const PARSE_ERROR: &str = "HOM001";
pub const UNKNOWN_TRIGGER: &str = "HOM002";
#[allow(dead_code)]
pub const UNKNOWN_EFFECT: &str = "HOM003";
#[allow(dead_code)]
pub const SCOPE_MISMATCH: &str = "HOM004";
pub const MISSING_LOCALIZATION: &str = "HOM005";

pub const BUILDING_LEVEL_EXCEEDS_MAX: &str = "HOM1002";
pub const CHARACTER_SKILL_EXCEEDS_MAX: &str = "HOM1004";
pub const CHARACTER_SUBSKILL_EXCEEDS_PRACTICAL: &str = "HOM1006";
pub const CHARACTER_NEGATIVE_SKILL: &str = "HOM1007";
pub const VICTORY_POINT_PROVINCE_NOT_IN_STATE: &str = "HOM2001";
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
#[allow(dead_code)]
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

pub const PORTRAIT_UNKNOWN_GFX: &str = "HOM4001";
pub const UNKNOWN_COUNTRY_METADATA_GFX: &str = "HOM4002";
pub const UNKNOWN_STATE_CATEGORY: &str = "HOM5001";
pub const UNKNOWN_RESOURCE: &str = "HOM5002";
pub const UNKNOWN_BUILDING: &str = "HOM5003";
pub const UNKNOWN_NAVAL_TERRAIN: &str = "HOM5004";
pub const UNKNOWN_PROVINCE_TERRAIN: &str = "HOM5005";

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
