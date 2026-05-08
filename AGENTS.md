# Hearts of Modding - Agent Instructions

VS Code extension for Hearts of Iron IV modding with a Rust LSP server.

## Project Structure

Two-part architecture:
- `client/`: TypeScript VS Code extension (Node.js/npm)
- `server/`: Rust LSP server (Cargo)

## Build & Development Commands

### Client (TypeScript)
```bash
cd client
npm install              # Install dependencies
npm run compile          # Compile TypeScript to out/
npm run watch            # Watch mode for development
npm run lint             # Run ESLint
npm run package          # Build VSIX (includes server compilation)
```

### Server (Rust)
```bash
cd server
cargo build              # Debug build
cargo build --release    # Release build
cargo check              # Fast syntax/type check
cargo clippy             # Linting
cargo test               # Run tests
```

### VS Code Debugging
- Use "Launch Extension" configuration in `.vscode/launch.json`
- Runs `npm: compile` pre-launch task automatically
- Extension loads from `client/` directory
- Server binary fallback: looks for `../server/target/release/server` or `../server/target/debug/server` if `client/server-bin/` binaries not found

## Packaging for Distribution

The `npm run package` script in `client/` performs the full build:
1. Builds Rust server with `cargo build --release`
2. Copies binaries to `client/server-bin/` (platform-specific: `server-linux`, `server-win.exe`)
3. Compiles TypeScript client
4. Packages VSIX with `vsce package`

**Important:** Server binaries must be built for both Linux and Windows for distribution. CI handles cross-compilation.

## CI/CD Workflows

### `.github/workflows/build.yml`
- Triggers on push/PR to main affecting `client/`, `server/`, or workflow files
- Builds server binaries for Linux and Windows in parallel
- Packages extension with both binaries
- Uploads `hearts-of-modding.vsix` artifact

### `.github/workflows/release.yml`
- Manual trigger only (`workflow_dispatch`)
- Reads version from `client/package.json`
- Cross-compiles server for Linux (`x86_64-unknown-linux-gnu`) and Windows (`x86_64-pc-windows-gnu`)
- Creates GitHub release with tagged VSIX

## Server Architecture

### Key Files
- `src/main.rs`: LSP implementation (179KB, contains most logic)
- `src/parser.rs`: `nom`-based HOI4 script parser
- `src/ast.rs`: AST definitions
- `src/scope.rs`: Scope stack inference (Global > Country > State, etc.)
- `src/hoi4_data.rs`: Static database of triggers/effects/scopes
- `src/*_scanner.rs`: Discovery modules for game entities

### Scanner Modules
Each scanner indexes specific HOI4 data types:
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
- `loc_parser.rs`: Localization `.yml` parsing

### Configuration & Assets
- `server/Config/*.cwt`: CWTools schema files (triggers, effects, modifiers, scopes, links)
- `server/assets/*.json`: Modifier mappings and formats for localization

## HOI4-Specific Parsing Rules

- Identifiers can start with digits and contain: `.`, `:`, `@`, `[ ]`, `?`, `^`, `$`, `/`, `-`, `'`, `%`
- Handles UTF-8 BOM in script and localization files
- Supports complex identifiers: `daw.2.t`, `[?var]`, `array^0`
- Localization keys support dots and automatic versioning (`key` → `key:0`)

## Extension Activation

Extension activates when:
- Opening a file with `hoi4` or `hoi4-localisation` language ID
- Workspace contains `descriptor.mod` file
- User manually runs `HOI4: Activate Extension` command

Language associations are scoped to HOI4 directories:
- `.txt` files in: `common/`, `events/`, `map/`, `history/`, `script/`, or `descriptor.mod`
- `.yml` files in: `localisation/`

## Configuration Settings

Key workspace settings:
- `hoi4.enabled`: Enable/disable extension per workspace
- `hoi4.gamePath`: Path to HOI4 installation (required for VFS)
- `hoi4.styling.enabled`: Toggle cosmetic checks (casing, indentation, whitespace)
- `hoi4.styling.cosmeticLocalizationIndentation`: Indent localization variants (`_DEF`, `_ADJ`) with extra tab
- `hoi4.showMemoryUsage.enabled`: Show LSP memory usage in status bar

## Development Gotchas

- Server must copy `Config/` and `assets/` directories to `client/server-bin/` for packaging
- CI uses `find` commands to locate and rename binaries after artifact download
- Release workflow uses `x86_64-pc-windows-gnu` target (MinGW) for Windows cross-compilation
- Local development doesn't require pre-built binaries in `server-bin/` (client falls back to `target/` directory)
- Version is single source of truth in `client/package.json`

## Testing

No test suite currently implemented. Manual testing via "Launch Extension" debug configuration.
