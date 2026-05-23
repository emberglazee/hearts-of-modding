# 🧾 Changelog

All changes to the **Hearts of Modding** extension will be documented in this file.

## [v0.7.0] - 2026-05-23

### Added

- Added `alwaystransparent` to the casing styling check.

### Changed

- Numerous LSP performance optimizations.

- Proper Workspace-wide rename symbol support, including for files that aren't opened in the editor.

- Extensive modernization throughout the LSP codebase and adopting Rust 2024 idioms by addressing `cargo clippy` errors and warnings.

### Fixed

- Properly parse localization keys with dashes (`-`) in them (TextMate syntax highlighting).

- Proper float number parsing fallback to a number type.

## [v0.6.1] - 2026-05-22

### Changed

- Optimized the localization parser to fix a performance bottleneck.

- Added `containerWindowType` and `origo` to the keyword casing styling check.

- Added the trailing newline check in the "Fix all styling issues" bulk code action.

- Updated Rust LSP and extension dependencies.

## [v0.6.0] - 2026-05-17

### Added

- **Enhanced Adjacency Support:**
  - Improved `adjacency_rules.txt` scanning to extract `required_provinces` and `icon` data for use in tooltips and validation.
  - Added context-aware hover tooltips for `adjacencies.csv` that resolve province IDs (Start, End, Through) into their terrain/type and display full rule details for rule references.
  - Added completions for adjacency rule names in `adjacencies.csv` and for all `adjacency_rule` fields and sub-blocks in `adjacency_rules.txt`.
  - Implemented specialized validation for `adjacency_rules.txt` to verify that all province IDs in `required_provinces` exist in the workspace.
  - Added a sea-adjacency validation hint for `adjacencies.csv` entries missing a Through province.
  - Expanded TextMate grammar to highlight `adjacency_rule` and its associated fields (`required_provinces`, `icon`, `contested`, etc.).
- **Integrated CSV Support:**
  - Implemented a specialized CSV formatter (`csv_parser.rs`) for map definition and adjacency files. It provides semantic alignment by padding columns with spaces to match the maximum width of each column across the entire file.
  - Added support for the standard LSP `documentFormatting` command for `.csv` files.
  - Exempted `.csv` files from standard Paradox script styling checks (indentation, trailing whitespace) to allow for custom tabular formatting.
- **Context-Aware Hover Refinements:**
  - Improved the LSP's identifier resolution to track the "context key" (parent assignment or block name) during AST traversal.
  - Restricted Province, State, and Strategic Region tooltips for integers to only appear when the surrounding context is relevant (e.g., inside `provinces = { ... }` or assigned to `owns_state`). This significantly reduces false-positive tooltips when hovering over generic numbers like quantities or years.
  - Enabled hover and navigation support for `Number` and `Boolean` values in the AST, allowing tooltips to appear when hovering directly over values, not just their keys.
- **Bug Fixes:**
  - Fixed a false-positive where specialized map data validation and hover tooltips were incorrectly applied to regular script files named `buildings.txt` (e.g., `common/buildings/00_buildings.txt`). These are now strictly restricted to files in the `map/` directory.
  - Fixed a diagnostic bug where `remove_ideas = all` was flagged as an "Unknown idea". The `all` keyword is now recognized as valid within this context.
- **Map Validation Improvements:**
  - Added specific validation for `map/buildings.txt` to warn about empty lines, which are counted as errors by the HOI4 engine.
  - Added a styling exception for `map/buildings.txt` to suppress the end-of-file newline diagnostic, preventing forced empty lines at the end of the file.

- Added an option to bulk fix all styling issues in a file.

## [v0.5.4] - 2026-05-16

### Added

- **Idea Picture Resolution & Validation:**
  - Implemented engine-accurate resolution for the `picture` field in country ideas. The extension now strictly prepends the `GFX_idea_` prefix to the `picture` attribute value, matching HOI4's internal sprite lookup.
  - Added support for graphical culture fallbacks. Validation and hover resolution now intelligently check for culture-specific variants (e.g., `_middle_eastern_2d`) when the base sprite is not found.
  - Added default picture resolution: if the `picture` field is omitted, the extension automatically validates against the implied `GFX_idea_[videa_name]` sprite.
  - Introduced a specialized internal `Idea` scope to correctly handle these context-sensitive resolution and validation rules within idea definition blocks.
- **UI Keyword Casing Diagnostic:** Expanded the keyword casing check to include `orientation` and `buttonType`. The extension now flags non-standard casing (e.g., `Orientation` or `buttontype`) in `.gui` and script files, providing links to interface modding documentation and a bulk fix to standardize casing across the entire file.
- **Major TextMate Grammar Expansion:** Significantly expanded the base TextMate syntax highlighting using official engine documentation and vanilla file analysis to provide richer, high-fidelity colors without relying on the LSP.
  - Categorized highlighting for hundreds of Effects, Triggers, and Modifiers extracted directly from the HOI4 documentation.
  - Added specialized highlighting for scopes (`ROOT`, `FROM`, etc.), 3-letter country tags, and event IDs.
  - Improved variable highlighting for `[v?var]` and scoped variables (e.g., `var:tag@name`).
  - Added full support for GUI files (`.gui`), including component types (`containerWindowType`, `buttonType`, etc.) and common UI properties (`font`, `clipping`, `shortcut`).
  - Improved numeric highlighting to support double-percentage values (e.g., `100%%`) commonly used in UI layouts.
  - Added highlighting for punctuation (braces/brackets) and hundreds of additional top-level blocks and effects.
  - Localization grammars now correctly highlight numeric color codes and bindable `$VAR$` variables.
- **Enhanced Leader Ability Support:** Expanded semantic highlighting for leader abilities to include key fields like `cost`, `duration`, `one_time_effect`, and `unit_modifiers`, ensuring better visual hierarchy in ability definition files.
- **Refined Hybrid Syntax Highlighting:** Improved the balance between TextMate and Semantic highlighting. Semantic tokens are now more conservative, only applying to known HOI4 keywords, operators, and variables. This ensures that TextMate's specialized colors for identifiers, country tags, and scopes are preserved and no longer overwritten by generic LSP tokens.

### Fixed

- **Scoped References in Ideology Fields:** `ideology = ROOT` (and other scoped references like `FROM`, `PREV`, `THIS`, `OWNER`, `CONTROLLER`, `CAPITAL`) are now recognized as valid runtime scope references instead of being flagged as unknown ideologies. This pattern is used extensively in vanilla HOI4 and workshop mods for `start_civil_war`, `add_popularity`, and similar effects.

## [v0.5.3] - 2026-05-16

### Added

- **Sound Effects Support:**
  - Added sound effects, sounds, falloffs, and sound categories to Workspace Symbols (`Ctrl+T`), enabling direct navigation to sound asset definitions.
  - Added validation for `sound_effect` references in ability files, warning when referencing undefined sound effects.

### Fixed

- **VFS-Aware Sound Effect Validation:** Sound effect scanner now scans `integrated_dlc/` and `dlc/` directories from the HOI4 game path, eliminating false-positive "Unknown sound effect" warnings for vanilla sound effects defined in DLC files.

### Removed

- Removed .cwt (CWTools) schema parsing and validation to be replaced with an alternative in a future version.

## [v0.5.2] - 2026-05-16

### Changed

- Updated VSCode extension dependencies, cleaned up unused dependencies.

- Bundle VSCode extension with `esbuild` for better performance.

## [v0.5.1] - 2026-05-16

### Added

- Added static modifiers to workspace symbols.
- **Cross-File Localization Duplicate Detection:** Added a warning diagnostic for localization keys defined in multiple files across the workspace.
- **VFS-Aware Overrides:** Duplicate localization warnings intelligently respect the `replace/` folder priority, suppressing warnings when an override is intentional.

### Changed

- Updated the LSP server Rust edition to 2024, reformatted the project code, updated some dependencies.
- **Streamlined Localization Versions:** Updated validation to always flag localization version numbers (e.g., `:0`) as unnecessary. Research confirms these are purely for internal Paradox translation tracking and ignored by the game engine.

### Fixed

- **Commented Localization Parsing:** Fixed an issue where the parser incorrectly interpreted commented-out localization keys as valid entries when they were preceded by specific multi-byte characters, resolving false positive duplicate key warnings.
- **Localization Version False Positives:** Fixed a bug where version numbers were incorrectly flagged due to improper file path tracking during duplicate checks.

## [v0.5.0] - 2026-05-15

### Added

- **Leader Abilities Support:**
  - Added full support for parsing and visualizing leader abilities from `common/abilities/`.
  - Tooltips display name, description, cost, duration, and associated sound effects.
  - Hover, completion, goto definition, rename, and workspace symbols are now supported for leader abilities.
- **Scripted Localisation Support:**
  - Implemented `scripted_loc_scanner` to index scripted localization from `common/scripted_localisation/`.
  - Fixes false-positive `invalid_loc_scope` warnings in localization files when referencing valid scripted localizations.
  - Scripted localizations are now fully integrated with Hover previews, Goto Definition (`F12`), and Workspace Symbols (`Ctrl+T`).
- **State Cross-Referencing & Intelligence:**
  - Implemented a fast `state_scanner` to parse all state definitions from `history/states/`.
  - Hovering over state IDs in triggers/effects (e.g. `owns_state = 123` or `any_state_of = { 123 }`) will dynamically look up and display the corresponding state's ID, its localized in-game name, and its definition source file.
  - States are now fully searchable through Workspace Symbols (`Ctrl+T`) by both their ID (e.g. `123`) and their localized name (e.g. `Texas`).
- **Workspace Symbols Update:**
  - Added support for sub-ideologies in Workspace Symbols (`Ctrl+T`), which now displays the sub-ideology and its parent ideology context.
  - Added support for localization keys in Workspace Symbols. The search will fuzzy-match against all parsed localization string keys, displaying them as `Localisation` entries. Results are capped at 1,000 matches to ensure IDE responsiveness while searching across potentially huge localization databases.
- **Map & Logistics Intelligence:**
  - **Custom Map Configurations:** Added support for parsing `map/default.map`. The extension dynamically reads the `definitions` and `adjacencies` assignments so that renamed mapping files (e.g. custom `definition.csv` names) receive full validation, highlighting, and tooltip support. Textmate rules have been updated to recognize `.map` files as native script.
  - **Deep CSV Validation:** Integrated `definition.csv` and `adjacencies.csv` directly into the document validation pipeline. The extension performs rigorous structural checks (e.g., ensuring exact column counts, validating bounds for RGB values, verifying province types and coastal booleans, checking coordinates).
  - **Column-Snapping Tooltips:** Hovering over map `.csv`, `unitstacks.txt`, or `buildings.txt` definitions now identifies the exact column your cursor is under and displays specific, contextual metadata (e.g., resolving a Province ID column into its terrain and coastal status, or highlighting exactly what coordinate a specific integer maps to).
  - **Logistics Scanner:** Added full support for `supply_nodes.txt` and `railways.txt`, exposing them to Workspace Symbols and validating their Province IDs.
  - **Map Objects Scanner:** Parses `buildings.txt`, `unitstacks.txt`, and `weatherpositions.txt`. This enables "Jump to Definition" for map buildings/objects, validates State and Province assignments, and provides contextual hover metadata for weather positions (Strategic Region lookup).
  - **Adjacencies & Rules:** Implemented parsing for `adjacencies.csv` and `adjacency_rules.txt`. This enables cross-referencing of straits and impassable borders, and validating province connectivity rules. Descriptive header lines in `adjacencies.csv` are now correctly handled and skipped during validation.
  - **Strategic Regions:** Added `strategic_region_scanner` to process `map/strategicregions/*.txt`.
  - **Enhanced Validations:** Direct document validation now supports `.csv` and `.txt` map definition files, highlighting invalid province/state references inline without needing script AST parsing.

### Fixed

- **Parser & Styling Robustness:**
  - Fixed a critical off-by-one error in `quoted_string` range calculations. This resolves a bug where the styling engine would incorrectly insert extra spaces before the assignment operator when using quoted keys (e.g., in bookmark files).
  - Fixed an issue where strategic region files bypassed general semantic and styling checks. They now receive full validation, including assignment operator spacing and brace placement rules.
  - Fixed a validation bug in `adjacencies.csv` where the column header was incorrectly flagged as an invalid ID.

### Notes

Because this version also scans the map files expect a major jump in extension memory usage in total conversion mod code bases. You can track Rust LSP server memory usage in the status bar with the `HOI4: Show Memory Usage` command.

## [v0.4.0] - 2026-05-12

### Added

- **Character Modding Support:**
  - **Comprehensive Character Scanner:** Recursively indexes character definitions from `common/characters/`, parsing traits, skills, roles, ideologies, and portraits.
  - **Rich Hover Visualizations:** Hovering over character IDs now displays their localized name, defined roles (advisor, country leader, unit leaders), their specific stats (attack, defense, planning, logistics, etc.), and their associated traits.
  - **Portrait Previews:** Sprite (`GFX_...`) or direct file paths (`"gfx/leaders/..."`) for character portraits are dynamically resolved and linked in the tooltip for quick viewing.
  - **IDE Integration:** Added character entities to Workspace Symbols (`Ctrl+T`), enabling direct navigation to definitions.
  - **Rename Refactoring:** Full support for renaming character identifiers across all script usages (e.g., `recruit_character`, `has_character`).
  - **Syntax Highlighting:** Added semantic highlighting coverage for character-specific keywords (e.g., `characters`, `portraits`, `field_marshal`, `corps_commander`).
- **Comprehensive Trigger, Effect, and Modifier Support:**
  - Added built-in support for over 550 individual HOI4 triggers, over 530 effects, and over 550 modifiers based on the official documentation.
  - Tooltips, autocompletion, and syntax highlighting are now fully operational for virtually all game triggers, effects, and modifiers.
- **Smart Localization Fixes:**
  - **Unescaped Quote Detection:** Added a context-aware diagnostic that identifies unescaped double quotes *inside* localization values while ignoring valid delimiters and comments.
  - **Quick Fixes:** Added both individual "Escape double quote" and bulk "Escape all unescaped double quotes in this file" code actions.
- **Advanced Localization Support (Patch 1.15+):**
  - **Contextual Objects:** Support for 1.15 objects like `IndustrialOrg`, `SpecialProject`, `PurchaseContract`, and `Ace`.
  - **Ternary Logic:** Full validation and preview support for conditional localization: `[v(Object.Property ? TRUE_KEY : FALSE_KEY)]`.
  - **Localization Formatters:** Support for `<formatter>|<token>` syntax (e.g., `tech_effect|id`).
  - **Bindable Localization:** Support for `$VAR$` style bindable variables within localization strings.
  - **New Formatting Codes:** Added support for `^` (SI units) and `%%` (literal percentage sign) in variable blocks.

### Changed

- **Backend Upgrades:**
  - Updated the Rust toolchain target to 1.95.0.
  - Bumped core dependencies including `tokio` (v1.52.3), `nom` (v8.0.0), `dashmap` (v6.1.0), and `sysinfo` (v0.39.1) for improved stability and performance.
- **Improved Localization Parser:**
  - Added support for hyphens (`-`) in localization keys to match native game behavior.
  - Recognized over 10 additional localization commands (e.g., `GetDateText`, `GetBopTrendTextIcon`).
  - Improved UTF-8 BOM handling to prevent rare parsing offsets.
- **VFS Priority:** Refined the workspace scanner to ensure files in `/replace/` subdirectories correctly override keys from standard folders.

### Fixed

- **Workspace Symbols Path Resolution:** Fixed an issue where `Url::from_file_path` incorrectly generated invalid paths like `//./` for workspace symbols and call hierarchies on relative paths. Paths are now correctly resolved to their absolute form before parsing.

## [v0.3.0] - 2026-05-10

### Added

- **Diagnostic Infrastructure Overhaul:**
  - **Unique Diagnostic Codes:** Introduced a standardized coding system with the `HOM` prefix (e.g., `HOM001` for Parse Errors, `HOM1002` for Building Levels). This enables easier filtering and future suppression rules.
  - **Diagnostic Related Information:** Implemented support for linking diagnostics to other parts of the codebase (e.g., linking a validation error to a specific building definition).
  - **New Diagnostic Tags:** Redundant localization version numbers are now tagged as `Unnecessary`, allowing editors to render them with faded/greyed-out styling.
  - **Unified Metadata:** All diagnostics now consistently report "Hearts of Modding" as their source for better attribution in the Problems view.
- **Workspace-Wide Awareness:**
  - **Automatic Workspace Scan:** The server can now perform a full recursive scan of all `.txt` and `.yml` files in the mod directory upon initialization.
  - **Toggleable Scanning:** This feature is **off by default** and can be enabled via `hoi4.validator.workspaceScan.enabled` or the new **HOI4: Toggle Workspace Scan** command.
  - **Custom Ignore Patterns:** Added `hoi4.validator.ignoreFiles` configuration. Supports regex patterns to completely exclude files or directories from diagnostics and intelligence features (References, Rename).
  - **Proactive Reporting:** You can now see all errors and warnings across your entire mod without needing to open every file manually.
  - **Mod Scope Isolation:** The full scan is strictly limited to the mod workspace, ensuring you aren't flooded with diagnostics from base game files.

### Changed

- **Improved Parser Robustness:**
  - Added support for `|` (pipe) and `*` (asterisk) characters in identifiers, fixing parsing errors in complex effect tooltips and SI-unit localization formatters.
  - Refactored internal diagnostic handling to a modular `validate_content` pipeline.
- **Internal Optimization:**
  - Cleaned up unused imports and suppressed dead code warnings across the server for a more stable build.
  - Improved workspace symbol search (`Ctrl+T`) by making fuzzy matching case-insensitive.

## [v0.2.2] - 2026-05-10

### Added

- MacOS ARM64 support.

### Changed

- Move README.md and CHANGELOG.md files from root into `client/` during extension compilation/packaging, otherwise the Marketplace is oblivious to them.

## [v0.2.1] - 2026-05-10

### Changed

- Bumped [vfast-uri](https://github.com/fastify/fast-uri) from v3.1.0 to v3.1.2 ([vsecurity update](https://github.com/fastify/fast-uri/releases/tag/v3.1.2)) | [vPR #1](https://github.com/emberglazee/hearts-of-modding/pull/1)

## [v0.2.0] - 2026-05-10

### Added

- **First-Class Achievement Support:**
  - Implemented `achievement_scanner` to index achievements and ribbons from `common/achievements/`.
  - Added specialized tooltips for achievements featuring custom headers (🏆/🎀), localized name/description previews, and direct definition links.
  - Added validation for missing `_NAME` and `_DESC` localization keys.
- **Workspace Intelligence & Navigation:**
  - **LSP Rename:** Full support for renaming Events, Scripted Triggers, Scripted Effects, Ideas, and Variables across the entire mod.
  - **Call Hierarchy:** Added support for visualizing incoming and outgoing relationships for events and scripted entities.
  - **Workspace Symbols:** Added global fuzzy search (`Ctrl+T`) for all indexed HOI4 symbols (Events, Ideas, Traits, Achievements, Sprites, etc.).
  - **Document Symbols:** Added comprehensive outline view support for script files, categorizing logic into meaningful sections (Events, Focuses, Characters, etc.).
- **Advanced Validation Engine:**
  - Added `advanced_validation` module for complex logical checks.
  - **Deep Schema Validation (CWT Support):** Rewrote the schema engine to support the full CWTools specification, including cardinality (e.g., `## cardinality = 1..1`), severity levels, and recursive blocks.
  - **Enum Validation:** Now parses `shared_enums.cwt` to validate values against thousands of game-defined constants (DLCs, tech bonuses, etc.).
  - **Custom Mod Schemas:** Added support for project-level schemas; `.cwt` files in `.cwtools/` or `Config/` are automatically merged into the validation engine.
  - **Building Levels:** Validates that building levels in state history do not exceed their `max_level` defined in `common/buildings/`.
  - **Character Skills:** Validates character skill levels against limits defined in `common/defines/*.lua`.
  - **Victory Points:** Validates that victory point provinces are correctly located within their assigned state.
- **Rich Tooltips & Documentation:**
  - **Modifier Display:** Implemented a sophisticated modifier formatting service that converts snake_case to Title Case, handles percentage formatting, and uses ✓/✗ indicators for beneficial/detrimental effects.
  - **Enhanced Scopes:** Tooltips now show a "Scope Stack" to help track nested logic; added specialized headers for Achievement and Music contexts.
- **Localization Improvements:**
  - Expanded recognized localization commands from 15 to over 80 (e.g., `GetNameDefCap`, `GetPartySupport`).
  - Added support for localization version numbers in the parser.
  - Added detection and "Hint" diagnostics for unnecessary version numbers in localization files.
- **Asset Visualization:**
  - **Enhanced Color Picker:** Added support for RGB and HSV color formats with integrated color picker/presentation support.

### Changed

- **Improved Syntax Highlighting:**
  - Updated semantic token engine to recognize thousands of game triggers, effects, and links as keywords.
  - Achievement keywords (`possible`, `happened`, `ribbon`) now receive proper semantic highlighting.
- **Refined Parser:**
  - Improved identifier boundary checks and support for special characters.
  - Enhanced robustness when handling large blocks and complex assignments.

### Fixed

- Fixed color picker appearing for arbitrary sets of three numbers that weren't intended as colors.
- Fixed scope detection for state history files (properly identifying the `state` scope).
- Removed redundant achievement hover blocks that caused duplicate tooltips.
- Resolved various unused import and dead code warnings across the server.

## [v0.1.0] - 2024-05-09

### Added

- **Core Features:**
  - Initial implementation of CWT schema parsing for triggers and effects.
- **New Asset Scanners:**
  - Implemented Music scanner for `music/*.asset` and `music/*.txt` files. Tracks music assets, radio stations, and song assignments.
  - Implemented Sound scanner for `sound/*.asset` files. Tracks sounds, sound effects, falloffs, and categories.
- **Advanced Localization Support:**
  - Added first-class support for `.yml` localization files with full syntax highlighting and validation.
  - Added validation for bracketed scopes (e.g., `[vRoot.GetName]`) within localization strings.
  - Added validation for Paradox color codes (`§Y...§!`) and support for numeric color codes (`§5`).
  - Implemented "Cosmetic Localization Indentation" heuristic to improve visual hierarchy of localization variants (e.g., `_DEF`, `_ADJ`).
- **Refined Activation Logic:**
  - Added `hoi4.enabled` workspace setting to control extension activation per workspace.
  - Extension now prompts to enable itself when a `descriptor.mod` file is detected in a new workspace.
  - Language associations for `.txt` and `.yml` are now scoped to standard HOI4 directories (e.g., `common/`, `events/`, `localisation/`) to avoid hijacking unrelated files.
- **UI & Tooling Improvements:**
  - Added memory usage display for the language server in the status bar (toggleable via `hoi4.showMemoryUsage.enabled`).
  - Added emoji-based categorization in tooltips (🎵 for music, 📜 for rules, 🌐 for localization) to improve readability.
  - Hyperlinked file paths and textures in tooltips for direct navigation.
- **Styling & Formatting:**
  - Added code action to remove all trailing whitespaces in the file.
  - Added "Convert all mixed indentation to tabs in this file" code action.
- **Backend Enhancements (LSP Server):**
  - Updated parser to support identifiers starting with digits and containing special characters like `'` and `%`.
  - Improved robustness by handling UTF-8 BOM in script and localization files.
  - Expanded `Value::TaggedBlock` (e.g., `rgb { ... }`) to track internal ranges for better diagnostic precision.
  - Integrated `sysinfo` for real-time memory monitoring.

### Fixed

- Fixed not being able to parse bookmark files due to quoted identifiers for country tags.
- Fixed language server not reporting its memory usage correctly to the client.
- Fixed broad `.yml` file associations incorrectly applying to GitHub workflow files.
- Improved identifier boundary checks in the parser to prevent false positives for keywords like `yes`/`no` when part of a longer string.
- Fixed styling code action "Convert indentation to tabs" not doing anything.

## [v0.0.1] - 2026-04-26

Initial release.
