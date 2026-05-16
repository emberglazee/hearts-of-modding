# Hearts of Modding - Agent Instructions

VS Code extension for Hearts of Iron IV modding with a Rust LSP server.

## Project Structure

Two-part architecture:
- `client/`: TypeScript VS Code extension (Node.js/npm, esbuild for packaging)
- `server/`: Rust LSP server (Cargo, `tower-lsp` + `tokio`)
- `server/Config/*.cwt`: CWTools schema files (triggers, effects, modifiers, scopes)
- `server/assets/*.json`: Modifier mappings and formats

## Build & Development Commands

### Client (TypeScript)
```bash
cd client
npm install              # Install dependencies
npm run compile          # Compile TypeScript to out/
npm run watch            # Watch mode for development
npm run lint             # Run ESLint
npm run package          # Full build: server + client + VSIX
```

### Server (Rust)
```bash
cd server
cargo build              # Debug build
cargo build --release    # Release build
cargo check              # Fast syntax/type check
cargo clippy             # Linting
cargo test               # Run tests (server/src/test_*.rs)
```

### VS Code Debugging
- Use "Launch Extension" configuration (`.vscode/launch.json`)
- Pre-launch task: `npm: compile` in `client/`
- Server binary fallback: `../server/target/release/server` or `../server/target/debug/server` if `client/server-bin/` not found

## Packaging

`npm run package` in `client/`:
1. Builds Rust server with `cargo build --release`
2. Copies binary to `client/server-bin/` (platform-specific name)
3. Copies `Config/` and `assets/` to `client/server-bin/`
4. Compiles TypeScript via `tsc`, bundles via `esbuild`
5. Packages VSIX with `vsce package`

**Distribution requires binaries for Linux, Windows (msvc), and macOS ARM64.** CI handles cross-compilation.

## CI/CD Workflows

### `.github/workflows/build.yml`
- Triggers on push/PR to `main` affecting `client/`, `server/`, or workflows
- Builds server for three targets in parallel:
  - Linux: `x86_64-unknown-linux-gnu`
  - Windows: `x86_64-pc-windows-msvc`
  - macOS: `aarch64-apple-darwin`
- Downloads binaries individually, renames to `server-linux`, `server-win.exe`, `server-macos-arm64`
- Packages VSIX, uploads as `extension-vsix` artifact

### `.github/workflows/release.yml`
- Manual trigger only (`workflow_dispatch`)
- Reads version from `client/package.json` (single source of truth)
- Same build matrix as build.yml
- Creates GitHub release with tagged VSIX

## Server Architecture

### Key Files
- `src/main.rs`: LSP implementation (~7900 lines, most logic)
- `src/parser.rs`: `nom`-based HOI4 script parser
- `src/ast.rs`: AST definitions (includes `TaggedBlock` for colors)
- `src/loc_parser.rs`: Localization `.yml` parser
- `src/scope.rs`: Scope stack inference (Global > Country > State, etc.)
- `src/hoi4_data.rs`: Static database of triggers/effects/scopes
- `src/semantic_tokens.rs`: Context-aware syntax highlighting
- `src/schema.rs`: CWTools schema loading

### Scanner Modules (parallelized, recursive)
- `event_scanner.rs`: Events and trigger chains
- `ideology_scanner.rs`: Ideologies and sub-ideologies
- `trait_scanner.rs`: Unit/country leader traits
- `idea_scanner.rs`: Ideas (all categories)
- `sprite_scanner.rs`: GFX sprite definitions
- `variable_scanner.rs`: Variables and event targets
- `modifier_scanner.rs`: Custom and dynamic modifiers
- `province_scanner.rs`: Province IDs from `map/definition.csv`
- `music_scanner.rs`: Music assets and radio stations
- `sound_scanner.rs`: Sound effects and categories
- `scripted_loc_scanner.rs`: Scripted localization
- `scripted_scanner.rs`: Scripted triggers and effects
- `achievement_scanner.rs`: Achievements
- `character_scanner.rs`: Characters
- `building_scanner.rs`: Buildings
- `state_scanner.rs`: States
- `strategic_region_scanner.rs`: Strategic regions
- `map_object_scanner.rs`: Map objects
- `logistics_scanner.rs`: Logistics
- `ability_scanner.rs`: Abilities
- `adjacency_scanner.rs`: Adjacencies

### Other Modules
- `rename.rs`: Cross-file rename support
- `call_hierarchy.rs`: Event relationship graphs
- `workspace_symbols.rs`: Global fuzzy symbol search
- `document_symbols.rs`: Per-document symbol outline
- `advanced_validation.rs`: Character skills, building levels, VP locations
- `defines_parser.rs`: Game defines parsing
- `enhanced_color.rs`: Color picker and preview
- `modifier_display.rs`: Modifier tooltip formatting
- `map_config.rs`: Map configuration handling

## Extension Activation

Activates when:
- Opening a file with `hoi4` or `hoi4-localisation` language ID
- Workspace contains `descriptor.mod`
- User runs `HOI4: Activate Extension`

Language associations:
- `hoi4`: `.txt` in `common/`, `events/`, `map/`, `history/`, `script/`; `descriptor.mod`; `.gui`, `.gfx`, `.asset`
- `hoi4-localisation`: `.yml` in `localisation/`

## Configuration Settings

Key workspace settings:
- `hoi4.enabled`: Enable/disable extension
- `hoi4.gamePath`: HOI4 installation path (required for VFS)
- `hoi4.styling.enabled`: Toggle cosmetic checks (casing, indentation, whitespace)
- `hoi4.styling.cosmeticLocalizationIndentation`: Extra tab for `_DEF`, `_ADJ` variants
- `hoi4.showMemoryUsage.enabled`: Status bar memory display
- `hoi4.validator.ignoreLocalization`: Regex patterns for missing loc keys to ignore
- `hoi4.validator.ignoreFiles`: Regex patterns for files/dirs to skip during workspace scan
- `hoi4.validator.workspaceScan.enabled`: Auto workspace-wide diagnostic scan on init (default: false)

## Commands

- `HOI4: Activate Extension` - Manually start LSP
- `HOI4: Set Game Path` - Update game path
- `HOI4: Toggle Styling Checks` - Enable/disable cosmetic checks
- `HOI4: Toggle Workspace Scan` - Enable/disable auto workspace scan
- `HOI4: Show Memory Usage` - Toggle memory status bar

## HOI4-Specific Parsing Rules

- Identifiers can start with digits and contain: `.`, `:`, `@`, `[ ]`, `?`, `^`, `$`, `/`, `-`, `'`, `%`
- Handles UTF-8 BOM in script and localization files
- Supports complex identifiers: `daw.2.t`, `[?var]`, `array^0`
- Localization keys support dots and automatic versioning (`key` → `key:0`)
- Version numbers in localization are cosmetic only (never read by HOI4 engine)

## Development Gotchas

- Server must copy `Config/` and `assets/` to `client/server-bin/` for packaging
- CI uses `x86_64-pc-windows-msvc` (not `gnu`) for Windows
- Local dev doesn't need `server-bin/` (falls back to `target/`)
- YAML files CAN be parsed by the HOI4 script parser - handle separately for indentation
- Semantic tokens override TextMate grammars - skip for `.yml` files
- LSP uses UTF-16 code units for positions, Rust strings are UTF-8 - always convert
- Multi-byte UTF-8 characters (`§`, emoji) require UTF-16 position conversion
- Localization parser must handle escaped quotes (`\"`) to avoid truncation
- Version is in `client/package.json` (server `Cargo.toml` version may lag)
