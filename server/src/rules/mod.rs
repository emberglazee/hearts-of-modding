use crate::data::interner::InternedStr;
use crate::data::layered_value::LayeredValue;
use crate::parser::ast;
use crate::parser::defines_parser;
use crate::scanner::building_scanner;
use crate::scope::scope;
use dashmap::DashMap;
use regex::Regex;
use tower_lsp_server::ls_types::Diagnostic;

pub(crate) mod abilities;
pub(crate) mod achievements;
pub(crate) mod ai_areas;
pub(crate) mod buildings;
pub(crate) mod characters;
pub(crate) mod country_metadata;
pub(crate) mod country_tags;
pub(crate) mod gfx_textures;
pub(crate) mod ideas;
pub(crate) mod ideologies;
pub(crate) mod localization;
pub(crate) mod portraits;
pub(crate) mod provinces;
pub(crate) mod sounds;
pub(crate) mod sprites;
pub(crate) mod state_definitions;
pub(crate) mod terrains;
pub(crate) mod traits;
pub(crate) mod visitor;

/// Context passed to validation rules during semantic checking.
///
/// Holds all scanner data and config references that rules may need,
/// eliminating the 17-parameter pass-through that `check_entry_semantic`
/// previously required.
pub(crate) struct ValidationContext<'a> {
    pub(crate) uri: &'a str,
    pub(crate) loc: &'a DashMap<InternedStr, LayeredValue<crate::parser::loc_parser::LocEntry>>,
    /// Scripted triggers - available for rule use (not yet used by any rule)
    #[allow(dead_code)]
    pub(crate) scripted_triggers:
        &'a DashMap<InternedStr, LayeredValue<crate::scanner::scripted_scanner::ScriptedEntity>>,
    /// Scripted effects - available for rule use (not yet used by any rule)
    #[allow(dead_code)]
    pub(crate) scripted_effects:
        &'a DashMap<InternedStr, LayeredValue<crate::scanner::scripted_scanner::ScriptedEntity>>,
    pub(crate) ideologies:
        &'a DashMap<InternedStr, LayeredValue<crate::scanner::ideology_scanner::Ideology>>,
    pub(crate) sub_ideologies:
        &'a DashMap<InternedStr, LayeredValue<(InternedStr, ast::Range, InternedStr)>>,
    pub(crate) traits: &'a DashMap<InternedStr, LayeredValue<crate::scanner::trait_scanner::Trait>>,
    pub(crate) sprites:
        &'a DashMap<InternedStr, LayeredValue<crate::scanner::sprite_scanner::Sprite>>,
    pub(crate) ideas: &'a DashMap<InternedStr, LayeredValue<crate::scanner::idea_scanner::Idea>>,
    pub(crate) provinces: &'a DashMap<u32, crate::scanner::province_scanner::Province>,
    pub(crate) modifier_mappings: &'a DashMap<InternedStr, String>,
    pub(crate) ignored_loc_regex: &'a [Regex],
    pub(crate) comments: &'a [(String, ast::Range)],
    pub(crate) sound_effects:
        &'a DashMap<InternedStr, LayeredValue<crate::scanner::sound_scanner::SoundEffect>>,
    pub(crate) country_tags:
        &'a DashMap<InternedStr, LayeredValue<crate::scanner::country_scanner::CountryTag>>,
    pub(crate) buildings: &'a DashMap<InternedStr, LayeredValue<building_scanner::Building>>,
    pub(crate) resources:
        &'a DashMap<InternedStr, LayeredValue<crate::scanner::resource_scanner::Resource>>,
    pub(crate) state_categories: &'a DashMap<
        InternedStr,
        LayeredValue<crate::scanner::state_category_scanner::StateCategory>,
    >,
    pub(crate) defines: &'a defines_parser::GameDefines,
    pub(crate) continents:
        &'a DashMap<InternedStr, LayeredValue<crate::scanner::continent_scanner::Continent>>,
    pub(crate) strategic_regions:
        &'a DashMap<u32, crate::scanner::strategic_region_scanner::StrategicRegion>,
    pub(crate) terrain_categories:
        &'a DashMap<InternedStr, LayeredValue<crate::scanner::terrain_scanner::TerrainCategory>>,
    pub(crate) abilities:
        &'a DashMap<InternedStr, LayeredValue<crate::scanner::ability_scanner::Ability>>,
    pub(crate) game_path: Option<String>,
    pub(crate) styling_enabled: bool,
}

/// A validation rule for HOI4 script semantics.
///
/// Rules are registered in [`Backend::check_semantic`] and invoked during
/// AST traversal. Two lifecycle hooks:
///
/// * `check_assignment` — called for every `Assignment` entry during the
///   tree walk. Receives the current scope so rules can be scope-aware.
/// * `check_block` — called once at the start with all file-level entries.
///   Use for cross-entry analysis (e.g. country tag ratios, AI area refs).
///
/// Both methods have default empty implementations so rules only override
/// what they need.
pub(crate) trait ValidationRule {
    /// Called for every `Assignment` during AST traversal.
    fn check_assignment(
        &self,
        _ass: &ast::Assignment,
        _ctx: &ValidationContext,
        _scope: &scope::ScopeStack,
        _diags: &mut Vec<Diagnostic>,
    ) {
    }

    /// Called once at the start with all file-level entries.
    fn check_block(
        &self,
        _entries: &[ast::Entry],
        _ctx: &ValidationContext,
        _diags: &mut Vec<Diagnostic>,
    ) {
    }
}
