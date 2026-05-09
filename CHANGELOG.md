# Changelog

All notable changes to the **Hearts of Modding** extension will be documented in this file.

## [0.2.0] - 2026-05-10

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

## [0.1.0] - 2024-05-09

### Added

- **Core Features:**
  - Initial implementation of CWT schema parsing for triggers and effects.
- **New Asset Scanners:**
  - Implemented Music scanner for `music/*.asset` and `music/*.txt` files. Tracks music assets, radio stations, and song assignments.
  - Implemented Sound scanner for `sound/*.asset` files. Tracks sounds, sound effects, falloffs, and categories.
- **Advanced Localization Support:**
  - Added first-class support for `.yml` localization files with full syntax highlighting and validation.
  - Added validation for bracketed scopes (e.g., `[Root.GetName]`) within localization strings.
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

## [0.0.1] - 2026-04-26

Initial release.
