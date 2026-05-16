# Hearts of Modding: Project Context & Mandates

This project is a high-performance Visual Studio Code extension specifically tailored for **Hearts of Iron IV (HOI4)** modding. It features a specialized Language Server Protocol (LSP) implementation written in Rust to provide a responsive and accurate modding experience.

## Core Mandates
- **Performance First:** All parsing and workspace indexing is handled by the Rust-based LSP to ensure no UI lag even with massive mods. Scans are parallelized for optimal startup speed.
- **HOI4 Specificity:** Unlike generalized Paradox tools, this extension is strictly optimized for HOI4 Paradox Script conventions and data structures.
- **VFS Integrity:** The extension must always respect the "Vanilla + Mod" loading priority (VFS), where mod files correctly override vanilla definitions.

## Project Structure
- `client/`: TypeScript VS Code extension (Node.js/npm, esbuild for packaging). Manages IDE integration, launches the server, and provides TextMate grammars.
- `server/`: Rust LSP Server (Cargo, `tower-lsp` + `tokio`).
    - `src/main.rs`: LSP implementation (~7900 lines, core logic).
    - `src/parser.rs`: A robust `nom`-based parser for HOI4 script. Handles complex identifiers (`daw.2.t`, `[?var]`, `array^0`), escapes, and range tracking.
    - `src/ast.rs`: Abstract Syntax Tree definition including `TaggedBlock` support for color formats.
    - `src/loc_parser.rs`: Localization `.yml` parser.
    - `src/semantic_tokens.rs`: Logic for context-aware syntax highlighting.
    - `src/scope.rs`: Implements the `ScopeStack` and inference logic (Global > Country > State, etc.).
    - `src/hoi4_data.rs`: Static database of core game triggers, effects, scopes, and localization commands.
    - `src/*_scanner.rs`: Discovery modules for localization, ideologies, traits, sprites, ideas, scripted elements, variables, provinces, modifiers, events, music, characters, buildings, states, and more.
    - `src/schema.rs`: CWTools schema loading (`Config/*.cwt`).
- `server/Config/`: CWTools schema files (triggers, effects, modifiers, scopes).
- `server/assets/`: Modifier mappings and formats (`.json`).

## Key Features

### 1. Hybrid Syntax Highlighting
- **TextMate:** Provides instant basic regex-based highlighting.
- **Semantic:** The LSP provides context-aware tokens for variables, operators, and HOI4 keywords.

### 2. Specialized Discovery & Tracking Systems
The LSP automatically indexes the following across both Vanilla and Mod roots using **recursive, parallelized scanning**:
- **Localization:** First-class support for `.yml` files. Supports keys with dots (`achevents.1.t`), automatic versioning resolution (`key` -> `key:0`), and UTF-8 BOM handling.
- **Cosmetic Localization Indentation:** Optional heuristic formatting for `.yml` files that indents variants (e.g., `_DEF`, `_ADJ`, `_desc`) with an additional tab to improve visual hierarchy.
- **Ideologies:** Tracks ideologies and sub-ideologies (types) with their parent relationships.
- **Traits:** Scans `unit_leader`, `country_leader`, and general traits.
- **Ideas:** Scans all categories of ideas defined in `common/ideas`.
- **Scripted Elements:** Discovers custom `scripted_triggers` and `scripted_effects`.
- **Graphics:** Indexes `spriteType` definitions in `.gfx` files.
- **Variables & Targets:** Workspace-wide tracking of `set_variable`, `event_target`, and `global_event_target`.
- **Modifiers:** Deep detection of custom (`common/modifiers`) and dynamic (`common/dynamic_modifiers`) modifiers, mapping engine modifiers to localization.
- **Events:** Comprehensive indexing of event definitions and their trigger-chains for graph analysis.
- **Music:** Scans `music/*.asset` and `music/*.txt`. Tracks music assets, radio stations, and song assignments with specialized scope handling for nested script triggers.

### 3. Navigation & Documentation
- **Hover:** Displays detailed documentation, definitions, file paths (hyperlinked), and the active **Scope Stack**. Supports sub-component hover for localization scopes like `[Root.GetName]`.
- **Visual Hierarchy:** Tooltip headers use descriptive emojis (e.g., 🎵 for music, 📜 for rules, 🌐 for localization) to improve readability and entity categorization.
- **Go to Definition:** `Ctrl+Click` support for all discovered entities. Prioritizes logic/source definitions over localization. Supports jumping between music assignments and asset definitions.
- **Find All References:** `Shift+Alt+F12` workspace-wide search for identifiers.
- **Completions:** Context-aware suggestions for triggers, effects, ideologies, traits, ideas, sprites, variables, event targets, and music content. Includes specialized completion for music file keys (`name`, `file`, `song`, `chance`).

### 4. Advanced Tooling & Validation
- **Real-time Validation**: 
    - **Syntax**: Precise error reporting for parsing failures.
    - **Semantic**: Validates localization keys, IDs (ideologies, traits, etc.), and GFX IDs.
    - **Dynamic Data**: Reads `map/definition.csv` to validate province IDs workspace-wide.
    - **Advanced Localization**: First-class support for `.yml` formatting, syntax highlighting, and validation. Includes bracketed scopes (`[Root.GetName]`), color code consistency (`§Y...§!`), and robust handling of numeric/symbolic color codes (e.g., `§5`, `§!`).
- **Cosmetic & Styling**: Enforces standard casing, indentation, and trailing whitespace with quick-fix support. Includes specialized cosmetic indentation for localization variants.
- **Color Support:** Integrated color picker and preview for various Paradox color formats.
- **Event Graphing:** Backend support for generating event trigger relationship graphs.

## Technical Nuances & Parsing Rules
- **Identifiers:** Can start with digits and contain `.`, `:`, `@`, `[ ]`, `?`, `^`, `$`, `/`, `-`, `'`, `%`. Supports complex forms: `daw.2.t`, `[?var]`, `array^0`.
- **Localization:** 
    - Keys support dots and automatic versioning resolution (`key` -> `key:0`).
    - Version numbers (e.g., `:0`) are cosmetic only and never read by the engine.
    - Handles UTF-8 BOM in both script and `.yml` files.
    - Localization parser must handle escaped quotes (`\"`) to avoid truncation.
- **Proactive Refresh:** Automatically re-validates open documents after initial workspace scans complete to prevent race-condition warnings.
- **Hyperlinked Tooltips:** All file paths and sprite textures in tooltips are hyperlinked for direct navigation.
- **Positioning:** LSP uses UTF-16 code units; Rust uses UTF-8. Conversion is mandatory, especially for multi-byte characters like `§` or emojis.

## Build & Development

### Commands

#### Client (TypeScript)
```bash
cd client
npm install              # Install dependencies
npm run compile          # Compile TypeScript to out/
npm run watch            # Watch mode for development
npm run lint             # Run ESLint
npm run package          # Full build: server + client + VSIX
```

#### Server (Rust)
```bash
cd server
cargo build              # Debug build
cargo build --release    # Release build
cargo check              # Fast syntax/type check
cargo clippy             # Linting
cargo test               # Run tests (server/src/test_*.rs)
```

### VS Code Debugging
- Use "Launch Extension" configuration (`.vscode/launch.json`).
- Pre-launch task: `npm: compile` in `client/`.
- Server binary fallback: `../server/target/release/server` or `../server/target/debug/server` if `client/server-bin/` not found.

### Packaging (`npm run package`)
1. Builds Rust server with `cargo build --release`.
2. Copies binary to `client/server-bin/` (platform-specific name).
3. Copies `Config/` and `assets/` to `client/server-bin/`.
4. Compiles TypeScript via `tsc`, bundles via `esbuild`.
5. Packages VSIX with `vsce package`.

**Distribution requires binaries for Linux, Windows (msvc), and macOS ARM64.** CI handles cross-compilation. CI uses `x86_pc-windows-msvc` (not `gnu`) for Windows distribution.

## CI/CD Workflows

### `.github/workflows/build.yml`
- Triggers on push/PR to `main` affecting `client/`, `server/`, or workflows.
- Builds server for three targets in parallel: Linux (`x86_64-unknown-linux-gnu`), Windows (`x86_64-pc-windows-msvc`), and macOS (`aarch64-apple-darwin`).
- Packages VSIX and uploads as `extension-vsix` artifact.

### `.github/workflows/release.yml`
- Manual trigger only (`workflow_dispatch`).
- Reads version from `client/package.json` (single source of truth).
- Creates GitHub release with tagged VSIX.

## Configuration & Extension Info

### Extension Activation
Activates when:
- Opening a file with `hoi4` or `hoi4-localisation` language ID.
- Workspace contains `descriptor.mod`.
- User runs `HOI4: Activate Extension`.

**Language Associations (`hoi4`):** `.txt` (common, events, map, history, script), `descriptor.mod`, `.gui`, `.gfx`, `.asset`.
**Language Associations (`hoi4-localisation`):** `.yml` (localization).

### Workspace Settings
- `hoi4.enabled`: Enable/disable extension.
- `hoi4.gamePath`: HOI4 installation path (required for VFS).
- `hoi4.styling.enabled`: Toggle cosmetic checks (casing, indentation, whitespace).
- `hoi4.styling.cosmeticLocalizationIndentation`: Extra tab for `_DEF`, `_ADJ` variants.
- `hoi4.showMemoryUsage.enabled`: Status bar memory display.
- `hoi4.validator.ignoreLocalization`: Regex patterns for missing loc keys to ignore.
- `hoi4.validator.ignoreFiles`: Regex patterns for files/dirs to skip during workspace scan.
- `hoi4.validator.workspaceScan.enabled`: Auto workspace-wide diagnostic scan on init.

### VS Code Commands
- `Hearts of Modding: Activate Extension` - Manually start LSP.
- `Hearts of Modding: Set Game Path` - Update game path.
- `Hearts of Modding: Toggle Styling Checks` - Enable/disable cosmetic checks.
- `Hearts of Modding: Toggle Workspace Scan` - Enable/disable auto workspace scan.
- `Hearts of Modding: Show Memory Usage` - Toggle memory status bar.

## Development Gotchas
- **Packaging:** Server must copy `Config/` and `assets/` to `client/server-bin/` for the VSIX to work.
- **Binary Naming:** Distribution requires specific names: `server-linux`, `server-win.exe`, `server-macos-arm64`.
- **YAML Handling:** YAML files can be parsed by the HOI4 script parser, but should be handled separately for proper indentation rules.
- **Semantic Tokens:** These override TextMate grammars; skip for `.yml` files to avoid highlighting conflicts.
- **UTF Conversion:** Always convert between Rust (UTF-8) and LSP (UTF-16) offsets.
- **Version Source:** `client/package.json` is the single source of truth for the version.

## Configuration & Schemas
- **CWTools Compatibility:** The server uses `.cwt` files in `server/Config/` (e.g., `triggers.cwt`, `effects.cwt`, `links.cwt`) to load schema definitions.
- **Assets:** Additional JSON files (`modifier_mappings.json`, `modifier_formats.json`) in `server/assets/` are used to map engine modifiers to localization keys.
