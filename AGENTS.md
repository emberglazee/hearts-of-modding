# Hearts of Modding

VS Code extension for HOI4 modding. Two-part architecture: `client/` (TypeScript VS Code extension) + `server/` (Rust LSP server, `tower-lsp` + `tokio`).

## Build & Dev

| Scope | Commands |
|-------|----------|
| Client | `cd client && npm install && npm run compile` |
| Server | `cd server && cargo build --release` |
| Both + VSIX | `cd client && npm run package` |
| Rust tests | `cd server && cargo test` |
| Rust lint | `cd server && cargo clippy` |
| Rust check | `cd server && cargo check` |

Client helpers in `package.json`: `npm run cargo:test`, `cargo:check`, `cargo:fmt` (run from `client/`).

**VS Code debugging:** Use "Launch Extension" config (`.vscode/launch.json`). Falls back to `../server/target/release/server` if `client/server-bin/` not found.

## Gotchas

- **CWT schemas removed.** `server/Config/*.cwt` and `src/schema.rs` are gone (v0.5.3). All validation is now custom Rust code in `server/src/`. Do not reference CWT.
- **UTF-16/UTF-8:** LSP uses UTF-16 code units, Rust uses UTF-8. Always convert — `§`, emoji, etc. break otherwise.
- **Semantic tokens** override TextMate grammars. Skip for `.yml` files (`main.rs:415`). Semantic tokens use triggers/effects/modifiers from `hoi4_data.rs` + scanner data as the single source of truth for keyword highlighting.
- **TextMate grammar** (`client/syntaxes/hoi4.tmLanguage.json`) is deliberately **minimal** — only structural patterns (comments, strings, numbers, operators, punctuation, GUI keywords). All effect/trigger/modifier/block name highlighting comes from semantic tokens. Do not add keyword lists to TextMate.
- **YAML files** can be parsed by the HOI4 script parser (similar syntax). Handle indentation separately — force `script_opt = None` for YAML in bulk fixes.
- **Version** is `client/package.json` (single source of truth). `server/Cargo.toml` version may lag.
- **Distribution** ships binaries for 3 platforms: Linux (`x86_64-unknown-linux-gnu`), Windows (`x86_64-pc-windows-msvc`), macOS (`aarch64-apple-darwin`). CI names them `server-linux`, `server-win.exe`, `server-macos-arm64`.
- **Packaging** copies `server/assets/` (not Config/) to `client/server-bin/`.
- **Localization:** Escaped quotes (`\"`) must be handled to avoid truncation. Version numbers (`:0`) are cosmetic only.
- **Workspace-wide rename** searches both open docs AND unopened workspace files. Unopened files are read from disk and parsed second. Only mod dir (`.`), not game path.

## Architecture

**Key server modules** (`server/src/`):
- `main.rs` (~3400 loc) — LSP core logic
- `parser.rs` — `nom`-based HOI4 script parser (handles complex identifiers: `daw.2.t`, `[?var]`, `array^0`)
- `ast.rs` — AST definitions
- `loc_parser.rs` — Localization `.yml` parser
- `hoi4_data.rs` — Static db of triggers/effects/scopes
- `semantic_tokens.rs` — Context-aware highlighting
- `scope.rs` — Scope stack inference
- `rename.rs` — Cross-file rename
- `call_hierarchy.rs` — Event relationship graphs
- `scan_orchestrator.rs` — All 22 scan methods + `load_assets`
- `formatting.rs` — Styling fixes (collect fixes, brace checks)
- `hover_handler.rs` — Hover logic (achievement/event/variable/scope context)
- `completion_handler.rs` — Completion logic for script and localization
- `code_action_handler.rs` — Code action logic (formatting, validation fixes)
- `entity_lookup.rs` — Adapter over `&ScannerData` with 4 query methods; eliminates entity-type cascades in goto-def, rename, semantic tokens
- `color_utils.rs`, `lsp_convert.rs`, `modifier_format.rs`, `loc_preview.rs`, `symbol_search.rs`, `scope_context.rs` — Utility modules extracted from main.rs

**Scanner modules** (parallelized, recursive): `event_scanner`, `ideology_scanner`, `trait_scanner`, `idea_scanner`, `sprite_scanner`, `variable_scanner`, `modifier_scanner`, `province_scanner`, `music_scanner`, `sound_scanner`, `scripted_loc_scanner`, `scripted_scanner`, `achievement_scanner`, `character_scanner`, `building_scanner`, `state_scanner`, `strategic_region_scanner`, `map_object_scanner`, `logistics_scanner`, `ability_scanner`, `adjacency_scanner`, `ai_strategy_plan_scanner`.

## Extension

- **Activation:** Opening `.txt` files in `common/events/map/history/script/`, `descriptor.mod`, `.gui`, `.gfx`, `.asset` (lang `hoi4`); `.yml` in `localisation/` (lang `hoi4-localisation`); or workspace contains `descriptor.mod`.
- **Key settings:** `hoi4.gamePath` (required for VFS), `hoi4.validator.workspaceScan.enabled` (off by default), `hoi4.styling.enabled`, `hoi4.validator.ignoreFiles`, `hoi4.validator.ignoreLocalization`.

## Architecture Decisions (unstable — may need reconsideration)

These were made during the 2026-05-26 architecture review. They are not carved in stone — if new evidence or friction emerges they should be revisited.

### ScannerData + Config context objects

**Scope:** Scanner data (32 ArcSwap fields from 22 scanners) lives in a separate `ScannerData` struct. Config fields (ignored_loc_regex, styling_enabled, cosmetic_loc_indent, workspace_scan_enabled, game_path, ignored_files_regex) live in a separate `Config` struct. `Backend` holds both as `scanner_data: ScannerData` and `config: Config`.

**Mutation:** `ScannerData` exposes individual `set_*` methods per field (e.g. `set_events(HashMap<String, Event>)`). The underlying `ArcSwap` fields are not `pub` — callers go through the methods.

**Depth of grouping:** Flat struct, no sub-grouping. Both `ScannerData` and `Config` are single flat structs. Sub-grouping (e.g. `EntityData`, `MapData`) was deferred — revisit if a handler emerges that only ever touches a subset.

### EntityLookup adapter

**Scope:** `EntityLookup` (`entity_lookup.rs`) wraps `&ScannerData` as an adapter with 4 query methods: `find_definition`, `entity_at`, `entity_names`, `find_symbols`. Handlers (`goto_definition`, `prepare_rename`, `find_symbol_at_position`, `semantic_tokens_full`) no longer iterate scanner data directly. `EntityKind` is a closed enum mapping all 22+ scanner entity types — adding scanner #23 means one file change.

**Not on the interface:** `hover_handler` composes with `find_definition` but keeps display logic (achievement/event/variable scope context) local. `workspace_symbols` not yet refactored — its per-entity display logic (containers, nested icons) is deeper than the symbol-search concern.

**Mutation:** None. `EntityLookup` is read-only; it borrows `ScannerData` which is mutated only during scan orchestration.
