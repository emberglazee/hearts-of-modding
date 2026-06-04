# Hearts of Modding

VS Code extension for HOI4 modding. Two-part architecture: `client/` (TypeScript VS Code extension) + `server/` (Rust LSP server, `tower-lsp` + `tokio` + `tikv-jemallocator`).

## Reference Docs (`hoi4-wiki/`)

When editing the extension's code (parser, scopes, triggers, effects, semantic tokens, validation, etc.), consult the `hoi4-wiki/` directory first. It contains **Paradox Wiki-format** HOI4 modding reference pages scraped from the official wiki, organized by category:

| Category | Contents |
|----------|----------|
| `scripting/` | Event modding, national focus modding, decision modding, idea/ideology, unit, equipment, technology, doctrine, division, character, building, MIO, country creation, cosmetic tags, balance of power, autonomy/state, achievements, AI modding, AI focuses, faction, bookmark, resources, scripted GUI |
| `documentation/` | Reference: triggers, effects, scopes, modifiers, defines, localisation, on-actions, data structures, ideology modding |
| `graphical/` | Interface modding, graphical assets, entity modding, particle/posteffect/font modding |
| `cosmetic/` | Portrait modding, namelist modding, music/sound modding |
| `map/` | Map modding, strategic region modding, state modding |
| `other/` | Mod structure, mods, nudger, troubleshooting, console commands |

These pages use Paradox Wiki markup (`{{version|1.12}}`, `{{path|events/}}`, `{{Main|Scopes}}`, `<pre>` blocks, `{|` wiki tables) but are otherwise plain markdown. They are the canonical reference for how HOI4 mod files are structured — the parser, scope inference, trigger/effect databases, and validator logic all relate directly to what's documented here. Read these files whenever you need to understand the underlying game mechanics that the extension operates on.

Categories roughly map to extension concerns: `documentation/triggers.md`, `documentation/effects.md`, `documentation/scopes.md` wire directly to `data/hoi4_data.rs` and `scope/scope.rs`; `documentation/localisation.md` to `parser/loc_parser.rs`; `scripting/event-modding.md`, `decision-modding.md`, `national-focus-modding.md` etc. inform scanner logic and semantic token behaviour.

## Build & Dev

| Scope | Commands |
|-------|----------|
| Client | `cd client && npm install && npm run compile` |
| Server | `cd server && cargo build --release` |
| Both + VSIX | `cd client && npm run package` |
| Rust tests | `cd server && cargo test` |
| Rust lint | `cd server && cargo clippy` |
| Rust check | `cd server && cargo check` |
| Rust format | `cd server && cargo fmt` |

Client helpers in `package.json`: `npm run cargo:test`, `cargo:check`, `cargo:fmt` (run from `client/`).

**VS Code debugging:** Use "Launch Extension" config (`.vscode/launch.json`). Falls back to `../server/target/release/server` if `client/server-bin/` not found.

## Architecture

**Server module layout** (`server/src/`):

```
server/src/
├── main.rs               # LSP entrypoint, module decls, jemalloc, UTF-16 utils
├── backend.rs             # Backend struct + AST cache + validation + formatting
├── config.rs              # Config struct (ArcSwap + AtomicBool fields)
├── data/                  # Static databases & shared data
│   ├── mod.rs
│   ├── hoi4_data.rs       # Static DB of triggers/effects/scopes/modifiers/loc_commands
│   ├── scanner_data.rs    # ScannerData struct (35+ DashMap fields via ArcSwap)
│   ├── entity_lookup.rs   # Adapter over &ScannerData — find_definition, entity_at, etc.
│   ├── interner.rs        # String interning (InternedStr = Arc<str>) for DashMap keys
│   └── layered_value.rs   # VFS layering: LayeredValue<T> preserves vanilla→mod→submod layers
├── lsp/                   # LSP protocol handlers
│   ├── mod.rs
│   ├── handler.rs         # impl LanguageServer for Backend — all LSP protocol handlers
│   ├── semantic_tokens.rs # Context-aware highlighting (script, .yml loc, .csv map files)
│   ├── hover_handler.rs   # Hover docs (achievement/event/variable/scope context)
│   ├── completion_handler.rs  # Completion logic for script and localization
│   ├── code_action_handler.rs # Code actions (formatting, validation fixes)
│   ├── rename.rs          # Cross-file rename
│   ├── call_hierarchy.rs  # Event relationship graphs
│   ├── document_symbols.rs # Document symbol provider
│   └── workspace_symbols.rs # Workspace symbol search
├── parser/                # HOI4 script parsers
│   ├── mod.rs
│   ├── parser.rs          # nom-based HOI4 script parser (complex identifiers)
│   ├── ast.rs             # AST definitions
│   ├── loc_parser.rs      # Localization .yml parser
│   ├── defines_parser.rs  # Game defines parser (common/defines/*)
│   └── csv_parser.rs      # CSV file parser
├── scanner/               # Parallelized file scanners
│   ├── mod.rs
│   ├── orchestrator.rs    # Orchestrates all scans + load_assets
│   ├── incremental_scanner.rs # Partial rescans for changed files
│   ├── bop_scanner.rs, event_scanner.rs, focus_scanner.rs, idea_scanner.rs
│   ├── ideology_scanner.rs, trait_scanner.rs, modifier_scanner.rs
│   ├── variable_scanner.rs, character_scanner.rs, building_scanner.rs
│   ├── country_scanner.rs, province_scanner.rs, state_scanner.rs
│   ├── strategic_region_scanner.rs, state_category_scanner.rs
│   ├── continent_scanner.rs, resource_scanner.rs, achievement_scanner.rs
│   ├── ability_scanner.rs, logistics_scanner.rs, adjacency_scanner.rs
│   ├── ai_strategy_plan_scanner.rs, ai_area_scanner.rs, music_scanner.rs
│   ├── sound_scanner.rs, sprite_scanner.rs, portrait_scanner.rs
│   ├── gfx_scanner.rs, map_object_scanner.rs, scripted_loc_scanner.rs
│   ├── scripted_scanner.rs, terrain_scanner.rs
│   └── (35 scanners total as of v0.15.1)
├── scope/                 # Scope inference
│   ├── mod.rs
│   ├── scope.rs           # Scope stack engine / resolve_key_scope
│   └── scope_context.rs   # Scope-aware hover context
├── rules/                 # Validation rules (trait-based + AstVisitor-based)
│   ├── mod.rs             # ValidationContext struct + ValidationRule trait
│   ├── visitor.rs         # AstVisitor trait + centralized walk_script() (single AST traversal)
│   ├── abilities.rs, achievements.rs, ai_areas.rs, buildings.rs
│   ├── characters.rs, country_metadata.rs, country_tags.rs
│   ├── gfx_textures.rs, ideas.rs, ideologies.rs
│   ├── localization.rs, portraits.rs, provinces.rs
│   ├── sounds.rs, sprites.rs, state_definitions.rs, terrains.rs, traits.rs
├── validation/            # Formatting & semantic validation
│   ├── mod.rs
│   ├── advanced_validation.rs  # Diagnostic code constants (HOM001–HOM4002)
│   ├── formatting.rs      # Styling fixes (collect fixes, brace checks)
│   └── modifier_format.rs # Modifier display formatting
├── utils/                 # Utility modules
│   ├── mod.rs
│   ├── lsp_convert.rs     # UTF-16 ↔ UTF-8 position conversion
│   ├── color_utils.rs     # Color-related utilities
│   ├── enhanced_color.rs  # Enhanced color parsing
│   ├── fs_util.rs         # File system helpers
│   ├── loc_preview.rs     # Localization preview rendering
│   ├── map_config.rs      # Map configuration helpers
│   ├── modifier_display.rs # Modifier display formatting
│   ├── mod_registry.rs    # Paradox mod registry path detection + submod resolution
│   └── symbol_search.rs   # Symbol search utilities
└── tests/                 # Integration tests
    ├── mod.rs
    ├── loc_columns.rs, loc_dups.rs, loc_empty.rs, loc_version.rs
    ├── parser_skip.rs, scripted_loc.rs, utf16_conversion.rs
```

**Key data flow:**

1. `main.rs` → `Backend::new()` → `config.rs` + `scanner_data.rs`
2. `scanner::orchestrator` runs 35 parallel scanners, populates `ScannerData` DashMaps (vanilla → mod → submod layers via `LayeredValue`)
3. `lsp::handler` receives LSP requests, uses AST cache (`document_asts`) for fast re-parsing
4. Semantic processing uses centralized `walk_script()` from `rules/visitor.rs` — single AST traversal calls both `AstVisitor` hooks + `ValidationRule::check_assignment`, replacing per-rule recursive walks
5. `ValidationRule::check_block` now handles only top-level cross-entry analysis (no recursion)
6. `validation::formatting` collects + applies style fixes
7. `scope::scope` tracks scope stacks for context-aware validation & completions
8. `did_change_watched_files` handles external file ops via incremental scanner + `LayeredValue` removal

## Extension

- **Version:** `0.15.1` — `client/package.json` is the single source of truth; `server/Cargo.toml` is kept in sync.
- **Allocator:** `tikv-jemallocator` (see fork at `emberglazee/jemallocator` fix-windows-msvc-spaces for Windows CI compat).
- **Activation:** `workspaceContains:./descriptor.mod` — root-only glob. LSP auto-starts unless `hoi4.lsp.enabled` is false. Toggle with `Hearts of Modding: Toggle LSP` command.
- **Key settings:** `hoi4.lsp.enabled`, `hoi4.lsp.suppressDisabledPrompt`, `hoi4.gamePath`, `hoi4.modPaths`, `hoi4.modRegistryPath`, `hoi4.validator.workspaceScan.enabled`, `hoi4.styling.enabled`, `hoi4.styling.cosmeticLocalizationIndentation`, `hoi4.validator.ignoreFiles`, `hoi4.validator.ignoreLocalization`, `hoi4.showMemoryUsage.enabled`, `hoi4.themePromptDismissed`.

## Gotchas

- **String interning:** All DashMap keys use `InternedStr` (`Arc<str>`) from `data/interner.rs`. All scanner entity `path` fields use `InternedStr`. `HasPath::path()` derefs to `&str` automatically. A reverse file-path index (`retain_path!` macro) provides O(K) incremental updates instead of DashMap::retain O(N).
- **UTF-16/UTF-8:** LSP uses UTF-16 code units, Rust uses UTF-8. Always convert (`byte_offset_to_utf16`, `utf16_to_byte_offset` in `main.rs`) — `§`, emoji, etc. break otherwise.
- **Semantic tokens** override TextMate grammars. Provide highlighting for `.yml` localization files too (`lsp/semantic_tokens.rs`). `.csv` map files (definition.csv, adjacencies.csv) also get semantic tokens. Semantic tokens use triggers/effects/modifiers from `data/hoi4_data.rs` + scanner data as the single source of truth for keyword highlighting.
- **TextMate grammar** (`client/syntaxes/hoi4.tmLanguage.json`) is deliberately **minimal** — only structural patterns (comments, strings, numbers, operators, punctuation, GUI keywords). All effect/trigger/modifier/block name highlighting comes from semantic tokens. Do not add keyword lists to TextMate.
- **YAML files** can be parsed by the HOI4 script parser (similar syntax). Handle indentation separately — force `script_opt = None` for YAML in bulk fixes.
- **Distribution** ships binaries for 3 platforms: Linux (`x86_64-unknown-linux-gnu`), Windows (`x86_64-pc-windows-msvc`), macOS (`aarch64-apple-darwin`). CI names them `server-linux`, `server-win.exe`, `server-macos-arm64`. CI runs across all 3 targets (Linux stable/nightly, Win MSVC, macOS ARM).
- **Packaging** copies `server/assets/` (not Config/) to `client/server-bin/`.
- **Localization:** Escaped quotes (`\"`) must be handled to avoid truncation. Version numbers (`:0`) are cosmetic only.
- **Workspace-wide rename** searches both open docs AND unopened workspace files. Unopened files are read from disk and parsed second. Only mod dir (`.`), not game path.
- **Validation system:** Uses a `ValidationRule` trait with `check_assignment` / `check_block` hooks, plus a newer `AstVisitor` trait with `enter_assignment` / `exit_assignment` / `after_walk` hooks. Both share one centralized AST traversal via `rules::visitor::walk_script()`. Rules are registered in `Backend::check_semantic` and receive a `ValidationContext` with all scanner data refs. Diagnostic codes prefixed HOM (HOM001–HOM4002) defined in `validation/advanced_validation.rs`.
- **AST caching:** `Backend` keeps a `document_asts: DashMap<String, (Arc<ast::Script>, Vec<(String, ast::Range)>)>` — parsed ASTs are cached per URI to avoid re-parsing on every semantic token / hover / completion. Cleared on `did_close`.
- **Test suite:** 72+ tests across 7 test modules. Run `cargo test` from `server/`.
- **did_change_watched_files:** Dynamic file watcher registration (`**/*.{txt,yml,asset,gfx,gui,csv,lua,mod}`). External file ops (Git branch switch, file explorer rename, etc.) trigger incremental rescans or `remove_path_from_scanner_data()` — no full re-scan needed.

## Architecture Decisions (stable as of v0.15.0)

### Module organization

Top-level modules (`data/`, `lsp/`, `parser/`, `scanner/`, `scope/`, `rules/`, `validation/`, `utils/`) with `mod.rs` re-exports. Each module has a single concern — `rules/` houses the validation rule trait + visitor + implementations, `scanner/` houses all 35 scanners, etc.

### ScannerData + Config context objects

**Scope:** Scanner data (35+ DashMap/ArcSwap fields from 35 scanners) lives in `ScannerData` struct (`data/scanner_data.rs`). Config fields live in `Config` struct (`config.rs`). `Backend` holds both as `scanner_data: ScannerData` and `config: Config`.

**Mutation:** `ScannerData` exposes `set_*` methods per field. `Config` uses a `config_field!` macro for consistent `ArcSwap` accessors + `set_` methods. Underlying fields are not `pub` — callers go through the methods.

**Depth of grouping:** Flat struct, no sub-grouping. Both are single flat structs. If a handler emerges that only ever touches a subset, sub-grouping can be revisited.

### EntityLookup adapter

**Scope:** `EntityLookup` (`data/entity_lookup.rs`) wraps `&ScannerData` with 5 query methods: `new`, `find_definition`, `entity_at`, `entity_names`, `find_symbols`. Handlers (`goto_definition`, `prepare_rename`, `find_symbol_at_position`, `semantic_tokens_full`) no longer iterate scanner data directly. `EntityKind` is a closed enum mapping all scanner entity types — adding a new scanner means one file change.

**Not on the interface:** `hover_handler` composes with `find_definition` but keeps display logic local. `workspace_symbols` has its own display logic.

**Mutation:** None. `EntityLookup` is read-only; it borrows `ScannerData` which is mutated only during scan orchestration.

### VFS Layering with LayeredValue

`LayeredValue<T>` (`data/layered_value.rs`) replaces plain `DashMap<K, V>` for overlay-able registries. It preserves ALL layers — vanilla first, then mod, then submods — in a priority-ordered Vec. Derefs to the highest-priority layer automatically, so existing code like `building.max_level` works transparently. When a mod file is deleted, `remove_path!` only removes that file's layer, keeping lower-priority vanilla entries intact. Maps with zero layers are dead and removed by callers checking `is_empty()`.

### Centralized AST Visitor

`rules/visitor.rs` introduces an `AstVisitor` trait with `enter_assignment`, `exit_assignment`, and `after_walk` hooks. `walk_script()` performs a single AST traversal, calling visitor hooks + `ValidationRule::check_assignment` for every assignment. This replaces the old per-rule recursive `check_block` pattern — with 15+ rules that meant 15+ AST walks, now it's exactly 1.

### String interning pattern

`InternedStr` (`Arc<str>`) used for all DashMap keys across all scanners. The `Interner` struct provides deduplication. A companion file-path index pattern (`retain_path!` macro) enables O(K) incremental updates when rescans discover stale entries.

### ValidationRule trait + AstVisitor

Validation is split into individual `ValidationRule` implementations in `rules/` (receive `ValidationContext` with all scanner data refs, registered in `Backend::check_semantic`). Newer rules implement `AstVisitor` instead, getting centralized traversal. Rules that migrated to `AstVisitor` have empty `check_block` stubs. Both coexist during the single `walk_script()` call: visitors get `enter_assignment`/`exit_assignment`, and rules get `check_assignment`.
