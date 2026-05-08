# Changelog

All notable changes to the **Hearts of Modding** extension will be documented in this file.

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
