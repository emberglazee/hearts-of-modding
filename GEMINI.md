# Hearts of Modding: Project Context & Mandates

This project is a high-performance Visual Studio Code extension specifically tailored for **Hearts of Iron IV (HOI4)** modding. It features a specialized Language Server Protocol (LSP) implementation written in Rust to provide a responsive and accurate modding experience.

## Core Mandates
- **Performance First:** All parsing and workspace indexing is handled by the Rust-based LSP to ensure no UI lag even with massive mods. Scans are parallelized for optimal startup speed.
- **HOI4 Specificity:** Unlike generalized Paradox tools, this extension is strictly optimized for HOI4 Paradox Script conventions and data structures.
- **VFS Integrity:** The extension must always respect the "Vanilla + Mod" loading priority (VFS), where mod files correctly override vanilla definitions.

## Project Structure
- `client/`: TypeScript VS Code extension. Manages IDE integration, launches the server, and provides TextMate grammars.
- `server/`: Rust LSP Server.
    - `src/parser.rs`: A robust `nom`-based parser for HOI4 script. Handles complex identifiers (`daw.2.t`, `[?var]`, `array^0`), escapes, and range tracking.
    - `src/ast.rs`: Abstract Syntax Tree definition including `TaggedBlock` support for color formats.
    - `src/semantic_tokens.rs`: Logic for context-aware syntax highlighting.
    - `src/scope.rs`: Implements the `ScopeStack` and inference logic (Global > Country > State, etc.).
    - `src/hoi4_data.rs`: Static database of core game triggers, effects, scopes, and localization commands.
    - `src/*_scanner.rs`: Discovery modules for localization, ideologies, traits, sprites, ideas, scripted elements, variables, provinces, modifiers, and events.

## Key Features

### 1. Hybrid Syntax Highlighting
- **TextMate:** Provides instant basic regex-based highlighting.
- **Semantic:** The LSP provides context-aware tokens for variables, operators, and HOI4 keywords.

### 2. Specialized Discovery & Tracking Systems
The LSP automatically indexes the following across both Vanilla and Mod roots using **recursive, parallelized scanning**:
- **Localization:** Supports keys with dots (`achevents.1.t`), automatic versioning resolution (`key` -> `key:0`), and UTF-8 BOM handling.
- **Ideologies:** Tracks ideologies and sub-ideologies (types) with their parent relationships.
- **Traits:** Scans `unit_leader`, `country_leader`, and general traits.
- **Ideas:** Scans all categories of ideas defined in `common/ideas`.
- **Scripted Elements:** Discovers custom `scripted_triggers` and `scripted_effects`.
- **Graphics:** Indexes `spriteType` definitions in `.gfx` files.
- **Variables & Targets:** Workspace-wide tracking of `set_variable`, `event_target`, and `global_event_target`.
- **Modifiers:** Deep detection of custom (`common/modifiers`) and dynamic (`common/dynamic_modifiers`) modifiers, mapping engine modifiers to localization.
- **Events:** Comprehensive indexing of event definitions and their trigger-chains for graph analysis.

### 3. Navigation & Documentation
- **Hover:** Displays detailed documentation, definitions, file paths (hyperlinked), and the active **Scope Stack**. Supports sub-component hover for localization scopes like `[Root.GetName]`.
- **Go to Definition:** `Ctrl+Click` support for all discovered entities. Prioritizes logic/source definitions over localization.
- **Find All References:** `Shift+Alt+F12` workspace-wide search for identifiers.
- **Completions:** Context-aware suggestions for triggers, effects, ideologies, traits, ideas, sprites, variables, and event targets.

### 4. Advanced Tooling & Validation
- **Real-time Validation**: 
    - **Syntax**: Precise error reporting for parsing failures.
    - **Semantic**: Validates localization keys, IDs (ideologies, traits, etc.), and GFX IDs.
    - **Dynamic Data**: Reads `map/definition.csv` to validate province IDs workspace-wide.
    - **Advanced Localization**: Validates bracketed scopes (`[Root.GetName]`) and color code consistency (`§Y...§!`).
- **Cosmetic & Styling**: Enforces standard casing, indentation, and trailing whitespace with quick-fix support.
- **Color Support:** Integrated color picker and preview for various Paradox color formats.
- **Event Graphing:** Backend support for generating event trigger relationship graphs.

## Technical Nuances
- **Identifier Rules:** Identifiers can start with digits and contain `.`, `:`, `@`, `[ ]`, `?`, `^`, `$`, `/`, and `-`.
- **Proactive Refresh:** Automatically re-validates open documents after initial workspace scans complete to prevent race-condition warnings.
- **Hyperlinked Tooltips:** All file paths and sprite textures in tooltips are hyperlinked for direct navigation.
