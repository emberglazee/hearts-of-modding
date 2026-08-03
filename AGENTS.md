# Hearts of Modding

VS Code extension for HOI4 modding. Two-part architecture: `client/` (TypeScript VS Code extension) + `server/` (Rust LSP server, `tower-lsp` + `tokio` + `tikv-jemallocator`, Rust 2024 edition).

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
├── main.rs               # LSP entrypoint, module decls, jemalloc, UTF-16 utils, CancellationToken
├── backend.rs             # Backend struct + AST cache + validation + formatting + FxHashMap
├── config.rs              # Config struct (ArcSwap + AtomicBool + Regex fields)
├── data/                  # Static databases & shared data
│   ├── mod.rs
│   ├── hoi4_data.rs       # Static DB of triggers/effects/scopes/modifiers/loc_commands
│   ├── scanner_data.rs    # ScannerData struct (35+ DashMap fields, incl. decision_categories_file_index, event_dep_graph)
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
│   ├── ast.rs             # AST definitions (ByteSpan-based, no owned strings)
│   ├── loc_parser.rs      # Localization .yml parser
│   ├── defines_parser.rs  # Game defines parser (common/defines/*)
│   └── csv_parser.rs      # CSV file parser
├── scanner/               # Parallelized file scanners (34 modules)
│   ├── mod.rs
│   ├── orchestrator.rs    # Orchestrates all scans + load_assets
│   ├── incremental_scanner.rs # Partial rescans for changed files + remove_path (incl. DecisionCategories, event-dep-graph cleanup)
│   ├── ability_scanner, achievement_scanner, adjacency_scanner
│   ├── ai_area_scanner, ai_strategy_plan_scanner, bop_scanner
│   ├── building_scanner, character_scanner, continent_scanner
│   ├── country_scanner, event_scanner, focus_scanner, gfx_scanner
│   ├── idea_scanner, ideology_scanner, logistics_scanner
│   ├── map_object_scanner, modifier_scanner, music_scanner
│   ├── portrait_scanner, province_scanner, resource_scanner
│   ├── scripted_loc_scanner, scripted_scanner, sound_scanner
│   ├── sprite_scanner, state_category_scanner, state_scanner
│   ├── strategic_region_scanner, terrain_scanner, trait_scanner
│   ├── unit_scanner, variable_scanner
│   └── oob_scanner
├── scope/                 # Scope inference
│   ├── mod.rs
│   ├── scope.rs           # Scope stack engine / resolve_key_scope (ScopeCtx maps are Option)
│   └── scope_context.rs   # Scope-aware hover context
├── rules/                 # Validation rules (trait-based + AstVisitor-based) — 19 modules
│   ├── mod.rs             # ValidationContext struct + ValidationRule trait
│   ├── visitor.rs         # AstVisitor trait + centralized walk_script() (single AST traversal)
│   ├── abilities.rs, achievements.rs, ai_areas.rs, buildings.rs
│   ├── characters.rs, country_metadata.rs, country_tags.rs
│   ├── gfx_textures.rs, ideas.rs, ideologies.rs
│   ├── localization.rs, oob_regiments.rs, portraits.rs, provinces.rs
│   ├── sounds.rs, sprites.rs, state_definitions.rs, terrains.rs, traits.rs
├── validation/            # Formatting & semantic validation
│   ├── mod.rs
│   ├── advanced_validation.rs  # Diagnostic code constants (HOM001–HOM5005)
│   ├── formatting.rs      # Styling fixes (collect fixes, brace checks)
│   └── modifier_format.rs # Modifier display formatting
├── utils/                 # Utility modules (11 total)
│   ├── mod.rs
│   ├── lsp_convert.rs     # RangeMapper (byte→UTF-16 col), location/position helpers
│   ├── line_index.rs      # Precomputed index: O(1) UTF-16↔byte offset lookups for a line
│   ├── color_utils.rs     # Color-related utilities
│   ├── enhanced_color.rs  # Enhanced color parsing
│   ├── fs_util.rs         # File system helpers
│   ├── loc_preview.rs     # Localization preview rendering
│   ├── map_config.rs      # Map configuration helpers
│   ├── modifier_display.rs # Modifier display formatting
│   ├── mod_registry.rs    # Paradox mod registry path detection + submod resolution
│   └── symbol_search.rs   # Symbol search utilities
└── tests/                 # 219 tests across 11 modules
    ├── mod.rs
    ├── abilities.rs, formatting.rs, ideas.rs
    ├── loc_columns.rs, loc_dups.rs, loc_empty.rs, loc_version.rs
    ├── oob_regiments.rs
    ├── parser_skip.rs, scripted_loc.rs, utf16_conversion.rs
```

**Key data flow:**

1. `main.rs` → `Backend::new()` → `config.rs` + `scanner_data.rs`
2. `scanner::orchestrator` runs 34 parallel scanners, populates `ScannerData` DashMaps (vanilla → mod → submod layers via `LayeredValue`)
3. `lsp::handler` receives LSP requests, uses debounced AST cache (`document_asts`) with per-document `CancellationToken` to cancel stale parses
4. Semantic processing uses centralized `walk_script()` from `rules/visitor.rs` — single AST traversal calls both `AstVisitor` hooks + `ValidationRule::check_assignment`, replacing per-rule recursive walks
5. `ValidationRule::check_block` now handles only top-level cross-entry analysis (no recursion)
6. `validation::formatting` collects + applies style fixes
7. `scope::scope` tracks scope stacks for context-aware validation & completions
8. `did_change_watched_files` handles external file ops via incremental scanner + `LayeredValue` removal

## Extension

- **Version:** `0.23.1` — `client/package.json` is the single source of truth; `server/Cargo.toml` is kept in sync.
- **Edition:** Rust 2024 (server/Cargo.toml).
- **Allocator:** `tikv-jemallocator` via fork at `emberglazee/jemallocator` (rev pinned in `[patch.crates-io]`; Windows MSVC path + `aarch64-pc-windows-msvc` support via the `fix-aarch64-msvc` branch — `gnu_target()` maps it to the `aarch64-w64-mingw32` alias so jemalloc's `config.sub` accepts the host). Keep the fork rev bump in sync with `client/package.json`'s version workflow. Do NOT gate jemalloc off any target without re-checking the fork's `gnu_target()` mapping first.
- **Activation:** `workspaceContains:./descriptor.mod` — root-only glob. Extension activates on detection; LSP then auto-starts unless `hoi4.lsp.enabled` is false (user gets a prompt on first open if disabled). Toggle with `Hearts of Modding: Toggle LSP` command.
- **Key settings:** `hoi4.lsp.enabled`, `hoi4.lsp.suppressDisabledPrompt`, `hoi4.gamePath`, `hoi4.modPaths`, `hoi4.modRegistryPath`, `hoi4.validator.workspaceScan.enabled`, `hoi4.styling.enabled`, `hoi4.styling.cosmeticLocalizationIndentation`, `hoi4.validator.ignoreFiles`, `hoi4.validator.ignoreLocalization`, `hoi4.showMemoryUsage.enabled`, `hoi4.themePromptDismissed`.

## Gotchas

- **String interning:** All DashMap keys use `InternedStr` (`Arc<str>`) from `data/interner.rs`. All scanner entity `path` fields use `InternedStr`. `HasPath::path()` derefs to `&str` automatically. A reverse file-path index (`retain_path!` macro) provides O(K) incremental updates instead of DashMap::retain O(N). The interner has a garbage collector for strings no longer referenced (can be triggered via `Interner::collect`).
- **UTF-16/UTF-8:** LSP uses UTF-16 code units, Rust uses UTF-8. The O(n) conversion functions (`byte_offset_to_utf16`, `utf16_to_byte_offset`) are in `main.rs`. For repeated conversions within the same line, use `utils/line_index.rs` (`LineIndex`) which precomputes the mapping for O(1) lookups — referenced in the doc comments on the main.rs functions. **For LSP emission** (diagnostics, hover, rename, symbols), use `utils/lsp_convert.rs::RangeMapper` (see below) — never pass `ast::Range` byte columns straight into `Position.character`. The legacy byte-based `ast_range_to_lsp` survives only as dead plumbing for the always-empty `related_information` path; new code must use `RangeMapper`. **For LSP inception** (comparing the client cursor to byte-based AST/loc ranges, e.g. `is_pos_in_range`), convert the UTF-16 `Position.character` to byte with `utils/lsp_convert.rs::to_byte_position(content, pos)` first — `is_pos_in_range` now requires a byte column and documents it. Sites fixed to convert at entry: `entity_at` (takes `content`), `find_identifier_at`/`scope_context` `find_*_at`, and the `.yml` loc hover. Do not shadow the global hover `position`; scripts use `find_identifier_at` which self-converts — double-converting corrupts multibyte lines.
- **Semantic tokens** override TextMate grammars. Provide highlighting for `.yml` localization files too (`lsp/semantic_tokens.rs`). `.csv` map files (definition.csv, adjacencies.csv) also get semantic tokens. Semantic tokens use triggers/effects/modifiers from `data/hoi4_data.rs` + scanner data as the single source of truth for keyword highlighting. Loc semantic token extraction uses byte-scanning (0xC2 0xA7 for §, no regex) — handles §X color codes, [...], $...$, \n, and escaped quotes.
- **TextMate grammar** (`client/syntaxes/hoi4.tmLanguage.json`) is deliberately **minimal** — only structural patterns (comments, strings, numbers, operators, punctuation, GUI keywords). All effect/trigger/modifier/block name highlighting comes from semantic tokens. Do not add keyword lists to TextMate.
- **YAML files** can be parsed by the HOI4 script parser (similar syntax). Handle indentation separately — force `script_opt = None` for YAML in bulk fixes.
- **Distribution** ships binaries as `hom-lsp-<os>-<arch>[.exe]` (e.g. `hom-lsp-linux-amd64`, `hom-lsp-win-arm64.exe`). CI builds 6 combos natively — linux (x64 `ubuntu-latest`, arm64 `ubuntu-24.04-arm`), windows (x64 `windows-latest`, arm64 `windows-11-arm`), macos (arm64 `macos-latest`) — with macOS amd64 cross-built from the arm64 macos runner (GitHub's Intel mac runners are scarce). The VSIX bundles the 3 primary combos (`hom-lsp-linux-amd64`, `hom-lsp-win-amd64.exe`, `hom-lsp-macos-arm64`) in `client/server-bin/`, and ALL 6 are published as standalone release assets alongside the VSIX. The client resolves the binary by `(platform, arch)` → bundled if present, else downloads the matching asset from the GitHub release (pinned to `v<installed version>`, falling back to `latest`) into `globalStorageUri/hom-lsp/<version>/`. `client/scripts/stage-binary.mjs` stages the native build for local `npm run package`.
- **Packaging** does NOT copy `server/assets/` into `client/server-bin/` — `hoi4_data_v2.json` is embedded into the binary at compile time (`include_str!` via `server/build.rs`, which minifies it into `OUT_DIR` first). A `server-bin/assets` copy would be dead weight in the VSIX; `client/scripts/stage-binary.mjs` removes any stale copy from older packaging runs.
- **Localization:** Escaped quotes (`\"`) must be handled to avoid truncation. Version numbers (`:0`) are cosmetic only. Newline (`\n`) and escaped double-quote highlighting is now supported.
- **Workspace-wide rename** searches both open docs AND unopened workspace files. Unopened files are read from disk and parsed second. Only mod dir (`.`), not game path.
- **Validation system:** Uses a `ValidationRule` trait with `check_assignment` / `check_block` hooks, plus a newer `AstVisitor` trait with `enter_assignment` / `exit_assignment` / `after_walk` hooks. Both share one centralized AST traversal via `rules::visitor::walk_script()`. Rules are registered in `Backend::check_semantic` and receive a `ValidationContext` with all scanner data refs. Diagnostic codes prefixed HOM (HOM001–HOM5005) defined in `validation/advanced_validation.rs`. **Rules must emit diagnostic ranges via `ctx.range(&ast::Range)`** — the `ValidationContext` carries a `RangeMapper` (`range_mapper`) so squiggle columns are UTF-16-correct; do not call `ast_range_to_lsp` from rules.
- **AST caching:** `Backend` keeps a `document_asts: DashMap<String, (Arc<ast::Script>, Vec<(String, ast::Range)>)>` — parsed ASTs are cached per URI. Each document also gets a `CancellationToken` in `document_cancellation_tokens` so that stale AST parses (from rapid editing) are cancelled. For unopened workspace files, ASTs are parsed on demand and not cached (commit `e1a7e65`). `did_change` is debounced to avoid parse storms.
- **ByteSpan AST nodes:** AST nodes (`ast.rs`) store `start..end` byte offsets instead of owned strings, reducing memory and parsing time. Actual text is resolved against the source on demand.
- **Test suite:** 323 `#[test]` functions across 11 modules (abilities, formatting, ideas, loc_columns, loc_dups, loc_empty, loc_version, oob_regiments, parser_skip, scripted_loc, utf16_conversion, plus scanner/decision/event-axis tests). Run `cargo test` from `server/`.
- **did_change_watched_files:** Dynamic file watcher registration (`**/*.{txt,yml,asset,gfx,gui,csv,lua,mod}`). External file ops (Git branch switch, file explorer rename, etc.) trigger incremental rescans or `remove_path_from_scanner_data()` — no full re-scan needed.
- **Locale decorators (VS Code):** `vscode_highlighting.rs` (client-side) provides editor decorations for localization `§X` color codes and escaped `\n`/`\"`, showing rendered colour and escaped characters directly in the editor.
- **Bracket-matching error recovery:** The parser recovers from missing brackets rather than cascading parse failures through the rest of the file.
- **Unified scope inference (one path):** Validation, hover, completion, and goto-definition all share `scope::scope::initial_scope_for_uri(uri)` (per-file-type initial scope: abilities→Character, decisions→Country, aces→Ace, ai_*→Country) plus the `ScopeStack::resolve_entry_scope(key, &ScopeCtx)` implementation. `resolve_scope_key` (the old simplified second path) was removed — do NOT reintroduce a parallel scope-resolution path; build a `ScopeCtx` (real `event_targets`/`characters`/`achievements` maps) and call `resolve_entry_scope` so the four features never diverge.
- **Double-assignment recovery (`HOM6003`):** `key = value = value` on one line (no braces) is a common slip (`custom_effect_tooltip = tooltip = LOC`), and the engine does NOT recover (Clausewitz throws `Unexpected token: =` / `Non assign trigger is not enclosed in {}`). The parser recovers the AST (last value wins) so the file doesn't cascade into generic `HOM001` parse errors, but it still emits a specific `HOM6003` ERROR via a byte-scan post-pass (`collect_double_assignment_ranges`, nesting-aware, skips strings/comments, never fires on a real block `key = { ... }`). Do not "silently" fix these — surface the ERROR (empirically verified).
- **Leading-dot numbers (`HOM6004`):** the engine REJECTS `.5` (`Malformed token: .5`, empirically verified via probe mod) — `.5` must be written `0.5`. The parser keeps `.5` as a String (so the file doesn't cascade) and emits a specific `HOM6004` ERROR via a byte-scan post-pass (`collect_leading_dot_number_ranges`; token-start dot + digit, never flags `0.5` or dotted identifiers `foo.bar`). Do NOT make `.5` a Number — Rust accepts it, the engine does not.
- **Check duplicate keys** uses `FxHashMap` (`rustc-hash`) for speed over the default SipHash-based `HashMap`.
- **Decision categories incremental path:** `common/decisions/categories/*.txt` classify as `FileCategory::DecisionCategories` (new variant). Because `decision_categories` stores `LayeredValue<()>` (no per-layer path), `retain_path!` can't attribute a category to its file — the scanner keeps a separate `decision_categories_file_index` (path → declared names) and only drops a category when no other indexed file still declares it. Categories are assumed globally unique.
- **Event dependency graph:** `EventDependencyGraph` (`scanner/event_dep_graph.rs`) keeps forward + reverse edges. `remove_events(&ids)` (used on event-file deletion) scrubs deleted events both as callers (outgoing) and callees (removed from every live callers' forward set and its reverse/dropped) — prevents stale hover "called-by" edges.
- **Decision rule scope tracking:** `DecisionsVisitor` (`rules/decisions.rs`) uses an explicit scope **stack** (Root→Category→Decision→SubBlock) with symmetric push/pop, not a single mutable `level` — the old single-state machine leaked sibling nesting and wrongly re-scoped later categories (`HOM5007` false positive on category-level `visible_when_empty`). Category-only keys (`visible_when_empty`, `picture`, `scripted_gui`, `on_map_area`, `day_of_week`) are valid directly inside a category; they're flagged only as a direct child of an individual decision. `allowed`/`visible` are category-field blocks and must not be treated as nested decisions (these are keyword-highlighted too).

## Architecture Decisions

### Module organization

Top-level modules (`data/`, `lsp/`, `parser/`, `scanner/`, `scope/`, `rules/`, `validation/`, `utils/`) with `mod.rs` re-exports. Each module has a single concern — `rules/` houses the validation rule trait + visitor + implementations, `scanner/` houses all 34 scanner modules, etc.

### ScannerData + Config context objects

**Scope:** Scanner data (35+ DashMap/ArcSwap fields from 34 scanners) lives in `ScannerData` struct (`data/scanner_data.rs`). Config fields live in `Config` struct (`config.rs`). `Backend` holds both as `scanner_data: ScannerData` and `config: Config`.

**Mutation:** `ScannerData` exposes `set_*` methods per field. `Config` uses a `config_field!` macro for consistent `ArcSwap` accessors + `set_` methods. Underlying fields are not `pub` — callers go through the methods. Config also has `AtomicBool` fields (`workspace_scan_enabled`, `styling_enabled`, `cosmetic_loc_indent`) and regex-vec fields (`ignored_loc_regex`, `ignored_files_regex`).

**Depth of grouping:** Flat struct, no sub-grouping. Both are single flat structs. If a handler emerges that only ever touches a subset, sub-grouping can be revisited.

### EntityLookup adapter

**Scope:** `EntityLookup` (`data/entity_lookup.rs`) wraps `&ScannerData` with 5 query methods: `new`, `find_definition`, `entity_at`, `entity_names`, `find_symbols`. Handlers (`goto_definition`, `prepare_rename`, `find_symbol_at_position`, `semantic_tokens_full`) no longer iterate scanner data directly. `EntityKind` is a closed enum mapping all scanner entity types — adding a new scanner means one file change.

**Not on the interface:** `hover_handler` composes with `find_definition` but keeps display logic local. `workspace_symbols` has its own display logic.

**Mutation:** None. `EntityLookup` is read-only; it borrows `ScannerData` which is mutated only during scan orchestration.

### VFS Layering with LayeredValue

`LayeredValue<T>` (`data/layered_value.rs`) replaces plain `DashMap<K, V>` for overlay-able registries. It preserves ALL layers — vanilla first, then mod, then submods — in a priority-ordered `SmallVec<[T; 1]>`. Derefs to the highest-priority layer automatically, so existing code like `building.max_level` works transparently. When a mod file is deleted, `remove_path!` only removes that file's layer, keeping lower-priority vanilla entries intact. Maps with zero layers are dead and removed by callers checking `is_empty()`. Using `SmallVec` instead of `Vec` avoids heap allocation for the common single-layer case.

### Centralized AST Visitor

`rules/visitor.rs` introduces an `AstVisitor` trait with `enter_assignment`, `exit_assignment`, and `after_walk` hooks. `walk_script()` performs a single AST traversal, calling visitor hooks + `ValidationRule::check_assignment` for every assignment. This replaces the old per-rule recursive `check_block` pattern — with 15+ rules that meant 15+ AST walks, now it's exactly 1. The walker is allocation-light: it resolves scope per assignment via `ScopeCtx` (Option maps — no empty `DashMap` per node) and inspects idea-promotion via the scope-stack slice rather than a per-key `Vec`.

### String interning pattern

`InternedStr` (`Arc<str>`) used for all DashMap keys across all scanners. The `Interner` struct provides deduplication with a fast-path for already-interned strings. A companion file-path index pattern (`retain_path!` macro) enables O(K) incremental updates when rescans discover stale entries. The interner includes a garbage collector that can be triggered to release strings no longer referenced.

### ValidationRule trait + AstVisitor

Validation is split into individual `ValidationRule` implementations in `rules/` (receive `ValidationContext` with all scanner data refs, registered in `Backend::check_semantic`). Newer rules implement `AstVisitor` instead, getting centralized traversal. Rules that migrated to `AstVisitor` have empty `check_block` stubs. Both coexist during the single `walk_script()` call: visitors get `enter_assignment`/`exit_assignment`, and rules get `check_assignment`.

### LineIndex for O(1) UTF-16 ↔ byte offset

`utils/line_index.rs` provides a `LineIndex` struct that precomputes UTF-16 code unit boundaries for each line. Both `byte_to_utf16()` and `utf16_to_byte()` are O(1) array lookups (with a binary-search edge case for multi-byte chars). This is the preferred approach over the O(n) functions in `main.rs` when doing many position conversions within the same string — used by `lsp_convert.rs` and the semantic token provider.

### RangeMapper — byte→UTF-16 at the LSP boundary

`ast::Range` stores `start_col`/`end_col` as per-line **byte** offsets (the parser works in bytes). LSP `Position.character` is **UTF-16** code units. For pure-ASCII they coincide, but any multi-byte char (§, accents, emoji) before a token shifts the byte column right of the true UTF-16 column — passing byte columns straight through (`ast_range_to_lsp`) silently misplaced diagnostics/hover/symbols on such lines.

`utils/lsp_convert.rs::RangeMapper` is the single correct converter. It is built **once per document** (`RangeMapper::new(text)`, O(n)) and answers conversions in O(1) by pairing a `LineIndex` (global byte→UTF-16) with the byte offset of each line start: a line's byte column maps to a UTF-16 column as the difference of two global positions.

- `ValidationContext` carries a `range_mapper` and exposes `ctx.range(&ast::Range) -> lsp_types::Range`. **All rule diagnostics must go through `ctx.range()`.** (Lifted the walker out of per-assignment empty-map allocations.)
- LSP handlers (rename, call_hierarchy, hover, code_action, document/workspace symbols, backend) build a `RangeMapper` per document/method from the source they already have.
- The legacy byte-based `ast_range_to_lsp` / `ast_related_info_to_lsp` remain only as plumbing for the always-empty `related_information` path — do not use them for new code.

### ScopeCtx optional maps (no empty-DashMap allocation)

`ScopeCtx`'s three scanner maps (`event_targets`, `characters`, `achievements`) are `Option<&DashMap<…>>`. Callers that lack a map pass `None` (the lookup is skipped) instead of allocating an empty `DashMap` to satisfy a non-Optional field. The central validation walker (`rules/visitor.rs`) passes its real maps and `None` for achievements; `ScopeStack::resolve_scope_key` builds a `None`-only ctx. This eliminated `DashMap::new()` per assignment in the walk loop.

### AST cancellation + debounce

The `Backend` struct includes a `document_cancellation_tokens: DashMap<String, CancellationToken>` field. On each `did_change`, any in-flight AST parse for that document is cancelled via its token and a new delayed parse is scheduled. Combined with debouncing, this prevents wasted parsing of intermediate edits — only the final state after the user stops typing is parsed.
