//! Shared test-support builder for validation-rule tests.
//!
//! One place constructs a [`ValidationContext`], so adding a field to that
//! struct is a one-line change here instead of an edit across every test
//! file. Prefer [`TestCtx`] over hand-building `ValidationContext { .. }` in
//! new tests.
//!
//! Two usage levels:
//!
//! - **Rule-unit level** (most tests): build a `TestCtx`, parse the source,
//!   and run selected rules/visitors over it with [`TestCtx::walk`] — the
//!   same `walk_script` call production validation makes. Seed scanner data
//!   either through the real incremental-scanner path ([`TestCtx::with_file`],
//!   preferred: exercises index_key normalization and retain_path for free)
//!   or via typed accessors like [`TestCtx::with_sprites`] when a test
//!   deliberately isolates the rule from its scanner.
//! - **Direct-context level**: [`TestCtx::build_context`] hands out the
//!   `ValidationContext` itself for rules with bespoke entry points
//!   (`check_block`, `GfxTextureRule::validate`, …).
//!
//! Fixture convention: keep script fixtures as inline `r#"..."#` consts next
//! to their assertions; prefix them `VANILLA_*` / `MOD_*`.

use crate::data::interner::InternedStr;
use crate::data::layered_value::LayeredValue;
use crate::parser::ast;
use crate::parser::parser::{self};
use crate::rules::visitor::{AstVisitor, walk_script};
use crate::rules::{ValidationContext, ValidationRule};
use crate::scope::scope::Scope;
use crate::utils::lsp_convert::RangeMapper;

/// Builder for a test [`ValidationContext`] plus the scanner data behind it.
///
/// All maps live in one `ScannerData` (like production) instead of loose
/// per-test locals, so seeding goes through real field names and the struct
/// never needs per-test re-listing.
pub(crate) struct TestCtx {
    data: crate::data::scanner_data::ScannerData,
    game_path: Option<String>,
    styling_enabled: bool,
    scope_validation_enabled: bool,
    workspace_roots: Option<Vec<std::path::PathBuf>>,
}

/// Borrowing twin of [`TestCtx`] for callers that already hold a
/// `&ScannerData` (rule modules' tests). Same context-building surface.
pub(crate) struct TestCtxRef<'a> {
    data: &'a crate::data::scanner_data::ScannerData,
    game_path: Option<String>,
    styling_enabled: bool,
    scope_validation_enabled: bool,
    workspace_roots: Option<Vec<std::path::PathBuf>>,
}

impl Default for TestCtx {
    fn default() -> Self {
        Self::new()
    }
}

impl TestCtx {
    pub(crate) fn new() -> Self {
        Self {
            data: crate::data::scanner_data::ScannerData::new(),
            game_path: None,
            styling_enabled: false,
            scope_validation_enabled: false,
            workspace_roots: None,
        }
    }

    /// Adopt an externally-built `ScannerData`. Prefer [`TestCtx::wrap_ref`]
    /// when the caller already holds a reference.
    #[allow(dead_code)]
    pub(crate) fn wrap(data: crate::data::scanner_data::ScannerData) -> Self {
        Self {
            data,
            game_path: None,
            styling_enabled: false,
            scope_validation_enabled: false,
            workspace_roots: None,
        }
    }

    /// Borrow an existing `ScannerData` for context construction. The
    /// resulting TestCtx borrows for `'a`; `build_context` outputs must not
    /// outlive `data`.
    pub(crate) fn wrap_ref<'a>(data: &'a crate::data::scanner_data::ScannerData) -> TestCtxRef<'a> {
        TestCtxRef {
            data,
            game_path: None,
            styling_enabled: false,
            scope_validation_enabled: false,
            workspace_roots: None,
        }
    }

    /// Seed scanner data by running the REAL incremental update path on a
    /// virtual file. This is what production does on did_save, so tests using
    /// it also cover index-key normalization and retain_path bookkeeping.
    /// Preferred seeding method for new tests.
    #[allow(dead_code)]
    pub(crate) fn with_file(self, path: &str, content: &str) -> Self {
        crate::scanner::incremental_scanner::update_scanner_data_for_file(
            &self.data, path, content,
        );
        self
    }

    /// Insert a localization key (LayeredValue boilerplate is loud inline).
    pub(crate) fn with_loc_key(self, key: &str) -> Self {
        self.data.localization.insert(
            InternedStr::from(key),
            LayeredValue::new(crate::parser::loc_parser::LocEntry {
                key: InternedStr::from(key),
                value: String::new(),
                range: ast::Range {
                    start_line: 0,
                    start_col: 0,
                    end_line: 0,
                    end_col: 0,
                },
                path: InternedStr::from("test.yml"),
                value_start_col: 0,
                version: None,
                version_range: None,
            }),
        );
        self
    }

    /// Pre-populate event namespaces as `(name, filepath)` pairs.
    pub(crate) fn with_event_namespaces(self, namespaces: &[(&str, &str)]) -> Self {
        for (name, filepath) in namespaces {
            self.data.event_namespaces.insert(
                InternedStr::from(*name),
                LayeredValue::new(crate::scanner::event_namespace_scanner::EventNamespace {
                    name: (*name).to_string(),
                    path: InternedStr::from(*filepath),
                    range: ast::Range {
                        start_line: 0,
                        start_col: 0,
                        end_line: 0,
                        end_col: 0,
                    },
                }),
            );
        }
        self
    }

    /// Pre-populate unit types by name (OOB regiment checks).
    pub(crate) fn with_unit_types(self, unit_types: &[&str]) -> Self {
        for ut in unit_types {
            self.data.unit_types.insert(
                InternedStr::from(*ut),
                LayeredValue::new(crate::scanner::unit_scanner::UnitType {
                    name: (*ut).to_string(),
                    abbreviation: Some(String::new()),
                    group: Some(String::new()),
                    combat_width: 0.0,
                    is_support: false,
                    type_categories: Vec::new(),
                    categories: Vec::new(),
                    path: InternedStr::from("test"),
                    range: ast::Range {
                        start_line: 0,
                        start_col: 0,
                        end_line: 0,
                        end_col: 0,
                    },
                }),
            );
        }
        self
    }

    /// Pre-populate ideas by name (IdeaRule existence checks).
    pub(crate) fn with_ideas(self, idea_names: &[&str]) -> Self {
        for name in idea_names {
            self.data.ideas.insert(
                InternedStr::from(*name),
                LayeredValue::new(crate::scanner::idea_scanner::Idea {
                    name: (*name).to_string(),
                    category: "country".to_string(),
                    picture: None,
                    path: InternedStr::from("test.txt"),
                    range: ast::Range {
                        start_line: 0,
                        start_col: 0,
                        end_line: 0,
                        end_col: 0,
                    },
                }),
            );
        }
        self
    }

    /// Pre-populate sprites by name (focus search-filter GFX checks).
    pub(crate) fn with_sprites(self, sprite_names: &[&str]) -> Self {
        for s in sprite_names {
            self.data.sprites.insert(
                InternedStr::from(*s),
                LayeredValue::new(crate::scanner::sprite_scanner::Sprite {
                    name: (*s).to_string(),
                    texture_file: String::new(),
                    path: InternedStr::from("interface/t.gfx"),
                    range: ast::Range {
                        start_line: 0,
                        start_col: 0,
                        end_line: 0,
                        end_col: 0,
                    },
                }),
            );
        }
        self
    }

    pub(crate) fn with_game_path(mut self, game_path: Option<&str>) -> Self {
        self.game_path = game_path.map(|s| s.to_string());
        self
    }

    pub(crate) fn with_styling(mut self, enabled: bool) -> Self {
        self.styling_enabled = enabled;
        self
    }

    #[allow(dead_code)]
    pub(crate) fn with_scope_validation(mut self, enabled: bool) -> Self {
        self.scope_validation_enabled = enabled;
        self
    }

    /// Workspace roots surfaced to rules (GfxTextureRule path resolution).
    pub(crate) fn with_workspace_roots(mut self, roots: Vec<std::path::PathBuf>) -> Self {
        self.workspace_roots = Some(roots);
        self
    }

    /// Mutable access to the underlying scanner data for seeds that don't
    /// warrant a named builder method yet. If two tests need the same seed,
    /// promote it to a `with_*` method.
    pub(crate) fn data(&mut self) -> &mut crate::data::scanner_data::ScannerData {
        &mut self.data
    }

    /// Shared read access (e.g. asserting against seeded entities).
    #[allow(dead_code)]
    pub(crate) fn scanner_data(&self) -> &crate::data::scanner_data::ScannerData {
        &self.data
    }

    /// Build the borrowed `ValidationContext` for `source`.
    ///
    /// Lifetimes: the returned context borrows `self`'s scanner data and the
    /// caller's `source`; keep both alive while running rules.
    pub(crate) fn build_context<'a>(
        &'a self,
        uri: &'a str,
        source: &'a str,
        range_mapper: &'a RangeMapper,
    ) -> ValidationContext<'a> {
        let d = &self.data;
        ValidationContext {
            uri,
            source,
            range_mapper,
            loc: &d.localization,
            scripted_triggers: &d.scripted_triggers,
            scripted_effects: &d.scripted_effects,
            ideologies: &d.ideologies,
            sub_ideologies: &d.sub_ideologies,
            traits: &d.traits,
            sprites: &d.sprites,
            ideas: &d.ideas,
            characters: &d.characters,
            provinces: &d.provinces,
            modifier_mappings: &d.modifier_mappings,
            ignored_loc_regex: &[],
            comments: &[],
            sound_effects: &d.sound_effects,
            country_tags: &d.country_tags,
            tag_aliases: &d.tag_aliases,
            buildings: &d.buildings,
            resources: &d.resources,
            state_categories: &d.state_categories,
            continents: &d.continents,
            strategic_regions: &d.strategic_regions,
            terrain_categories: &d.terrain_categories,
            abilities: &d.abilities,
            ace_modifiers: &d.ace_modifiers,
            game_path: self.game_path.clone(),
            styling_enabled: self.styling_enabled,
            scope_validation_enabled: self.scope_validation_enabled,
            workspace_roots: self.workspace_roots.as_deref().unwrap_or(&[]),
            unit_types: &d.unit_types,
            event_targets: &d.event_targets,
            event_namespaces: &d.event_namespaces,
            events: &d.events,
            decisions: &d.decisions,
            decision_categories: &d.decision_categories,
        }
    }

    /// Parse `input` and run the given rules + visitors over it starting at
    /// `initial_scope`. Returns all diagnostics produced.
    pub(crate) fn walk(
        &self,
        input: &str,
        uri: &str,
        initial_scope: Scope,
        rules: Vec<Box<dyn ValidationRule>>,
        visitors: Vec<Box<dyn AstVisitor>>,
    ) -> Vec<tower_lsp_server::ls_types::Diagnostic> {
        let (script, _) = parser::parse_script(input);
        let range_mapper = RangeMapper::new(&script.source);
        let ctx = self.build_context(uri, &script.source, &range_mapper);
        let rule_refs: Vec<Box<dyn ValidationRule>> = rules;
        let mut visitor_list: Vec<Box<dyn AstVisitor>> = visitors;
        let mut diags = Vec::new();
        walk_script(
            &script.entries,
            &mut visitor_list,
            &rule_refs,
            &ctx,
            &mut diags,
            initial_scope,
            false,
        );
        diags
    }
}

impl<'a> TestCtxRef<'a> {
    /// Build the borrowed `ValidationContext` for `source` from an existing
    /// `&ScannerData`. See [`TestCtx::build_context`] for lifetime notes.
    pub(crate) fn build_context(
        &'a self,
        uri: &'a str,
        source: &'a str,
        range_mapper: &'a RangeMapper,
    ) -> ValidationContext<'a> {
        // Identical field mapping to TestCtx::build_context — both read the
        // same ScannerData shape, only ownership differs.
        let d = self.data;
        ValidationContext {
            uri,
            source,
            range_mapper,
            loc: &d.localization,
            scripted_triggers: &d.scripted_triggers,
            scripted_effects: &d.scripted_effects,
            ideologies: &d.ideologies,
            sub_ideologies: &d.sub_ideologies,
            traits: &d.traits,
            sprites: &d.sprites,
            ideas: &d.ideas,
            characters: &d.characters,
            provinces: &d.provinces,
            modifier_mappings: &d.modifier_mappings,
            ignored_loc_regex: &[],
            comments: &[],
            sound_effects: &d.sound_effects,
            country_tags: &d.country_tags,
            tag_aliases: &d.tag_aliases,
            buildings: &d.buildings,
            resources: &d.resources,
            state_categories: &d.state_categories,
            continents: &d.continents,
            strategic_regions: &d.strategic_regions,
            terrain_categories: &d.terrain_categories,
            abilities: &d.abilities,
            ace_modifiers: &d.ace_modifiers,
            game_path: self.game_path.clone(),
            styling_enabled: self.styling_enabled,
            scope_validation_enabled: self.scope_validation_enabled,
            workspace_roots: self.workspace_roots.as_deref().unwrap_or(&[]),
            unit_types: &d.unit_types,
            event_targets: &d.event_targets,
            event_namespaces: &d.event_namespaces,
            events: &d.events,
            decisions: &d.decisions,
            decision_categories: &d.decision_categories,
        }
    }
}
