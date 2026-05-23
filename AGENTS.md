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
- **Semantic tokens** override TextMate grammars. Skip for `.yml` files (`main.rs:268`).
- **YAML files** can be parsed by the HOI4 script parser (similar syntax). Handle indentation separately — force `script_opt = None` for YAML in bulk fixes.
- **Version** is `client/package.json` (single source of truth). `server/Cargo.toml` version may lag.
- **Distribution** ships binaries for 3 platforms: Linux (`x86_64-unknown-linux-gnu`), Windows (`x86_64-pc-windows-msvc`), macOS (`aarch64-apple-darwin`). CI names them `server-linux`, `server-win.exe`, `server-macos-arm64`.
- **Packaging** copies `server/assets/` (not Config/) to `client/server-bin/`.
- **Localization:** Escaped quotes (`\"`) must be handled to avoid truncation. Version numbers (`:0`) are cosmetic only.
- **Workspace-wide rename** searches both open docs AND unopened workspace files. Unopened files are read from disk and parsed second. Only mod dir (`.`), not game path.

## Architecture

**Key server modules** (`server/src/`):
- `main.rs` (~7900 loc) — LSP core logic
- `parser.rs` — `nom`-based HOI4 script parser (handles complex identifiers: `daw.2.t`, `[?var]`, `array^0`)
- `ast.rs` — AST definitions
- `loc_parser.rs` — Localization `.yml` parser
- `hoi4_data.rs` — Static db of triggers/effects/scopes
- `semantic_tokens.rs` — Context-aware highlighting
- `scope.rs` — Scope stack inference
- `rename.rs` — Cross-file rename
- `call_hierarchy.rs` — Event relationship graphs

**Scanner modules** (parallelized, recursive): `event_scanner`, `ideology_scanner`, `trait_scanner`, `idea_scanner`, `sprite_scanner`, `variable_scanner`, `modifier_scanner`, `province_scanner`, `music_scanner`, `sound_scanner`, `scripted_loc_scanner`, `scripted_scanner`, `achievement_scanner`, `character_scanner`, `building_scanner`, `state_scanner`, `strategic_region_scanner`, `map_object_scanner`, `logistics_scanner`, `ability_scanner`, `adjacency_scanner`.

## Extension

- **Activation:** Opening `.txt` files in `common/events/map/history/script/`, `descriptor.mod`, `.gui`, `.gfx`, `.asset` (lang `hoi4`); `.yml` in `localisation/` (lang `hoi4-localisation`); or workspace contains `descriptor.mod`.
- **Key settings:** `hoi4.gamePath` (required for VFS), `hoi4.validator.workspaceScan.enabled` (off by default), `hoi4.styling.enabled`, `hoi4.validator.ignoreFiles`, `hoi4.validator.ignoreLocalization`.
