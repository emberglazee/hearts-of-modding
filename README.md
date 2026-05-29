# Hearts of Modding

[![made-for-VSCode](https://img.shields.io/badge/Made%20for-VSCode-1f425f.svg)](https://code.visualstudio.com/) [![GitHub license](https://badgen.net/github/license/Naereen/Strapdown.js)](https://github.com/Naereen/StrapDown.js/blob/master/LICENSE) ![GitHub top language](https://img.shields.io/github/languages/top/emberglazee/hearts-of-modding?style=flat-square&logo=rust&logoColor=ff8c00&label=Rust)

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/emberglazee/hearts-of-modding/.github%2Fworkflows%2Fbuild.yml?style=flat-square&link=https%3A%2F%2Fgithub.com%2Femberglazee%2Fhearts-of-modding%2Factions%2Fworkflows%2Fbuild.yml) ![GitHub Release](https://img.shields.io/github/v/release/emberglazee/hearts-of-modding?style=flat-square&label=release&link=https%3A%2F%2Fgithub.com%2Femberglazee%2Fhearts-of-modding%2Freleases)

**Hearts of Modding** is a heavily work in progress, high-performance **Visual Studio Code** extension specifically tailored for **Hearts of Iron IV (HOI4)** modders. Powered by a specialized Language Server Protocol (LSP) written in Rust, it provides a responsive, accurate, and deeply integrated modding experience that goes beyond standard Paradox scripting tools.

Inspired by the extensions [CWTools](https://github.com/cwtools/cwtools) and [VModer](https://github.com/textGamex/VModer).

## Key Features

### 🚀 High-Performance Intelligence
- **Rust-Powered LSP:** Quick response times even in massive mods like Millennium Dawn or Kaiserreich.
- **Parallelized Workspace Scanning:** Rapidly indexes your entire mod and the vanilla game files on startup.
- **Virtual File System (VFS):** Correctly respects mod loading priority, ensuring your mod's overrides are always accurately represented.

### 🔍 Specialized Discovery & Tracking
- **Achievement & Ribbon Indexing:** Comprehensive support for achievements and ribbons, featuring specialized tooltips and direct definition linking.
- **Workspace-Wide Symbols:** Global fuzzy search (`Ctrl+T`) for all indexed symbols (Events, Ideas, Achievements, Characters, States, Abilities, Traits, Sprites, Sound Effects, Scripted Localisation, etc.).
- **Call Hierarchy:** Visualize incoming and outgoing relationships for events and scripted entities to understand your mod's flow.
- **Deep Modifier Detection:** Automatically indexes custom and dynamic modifiers, linking them directly to their source definitions.
- **Character Modding Support:** Deep scanner for characters - parses roles, traits, skills, ideologies, and portraits. Rich hovers show localized names, stats, and resolved GFX sprite previews.
- **Leader Ability Intelligence:** Full parsing and validation for `common/abilities/` - enriched hover cards with cooldown, cancelable, icon, block presence, and formatted modifier/effect summaries. Validates missing required fields and missing `ai_will_do` blocks.
- **State Cross-Referencing:** Hover over state IDs in triggers/effects (`owns_state = 123`) to see the state's localized name, ID, and definition source. States are searchable via Workspace Symbols.
- **Map & Logistics Intelligence:** Parses `map/default.map`, `definition.csv`, `adjacencies.csv`, `strategicregions/`, `supply_nodes.txt`, `railways.txt`, and weather positions. Column-snapping tooltips resolve province IDs, terrain, and coastal status on hover.

- **AI Strategy Plans:** Parse, validate, and syntax-highlight `common/ai_strategy_plans/*.txt`.
- **Scripted Localisation:** Indexes `common/scripted_localisation/`, fixing false-positive scope warnings and enabling hover previews, Goto Definition, and Workspace Symbols.

### 🛡️ Advanced Validation
- **Logical Integrity Checks:** Validates character skills, building levels, victory points, adjacency rules, province connectivity, and ability definitions against game defines.
- **Map Data Validation:** Deep structural checks on `definition.csv` and `adjacencies.csv` (column counts, RGB bounds, province types, coastal booleans, coordinates). Warns about empty lines in `map/buildings.txt`.
- **Proactive Workspace Scan:** Optionally scan your entire mod directory for errors upon initialization.
- **Localization Infrastructure:** Deep parsing and validation of 80+ localization commands, ternary logic, contextual objects (1.15+), bindable `$VAR$` variables, and complex chains like `[Root.GetTag]`. Detects duplicate localization keys across files with VFS-aware `replace/` priority.
- **Cross-File Diagnostics:** Detects duplicate localization keys across the workspace, duplicately-defined sound effects, and unknown ability/sound effect references.
- **Paradox Styling Rules:** Optional checks for standard Paradox casing conventions, indentation (tabs vs. spaces), trailing whitespace, UI keyword casing, and end-of-file newline.

### 🛠️ Developer Productivity
- **Safe LSP Rename:** Rename Events, Scripted Triggers, Ideas, Characters, Abilities, and Variables across your entire project.
- **Diagnostic Enhancements:** Detailed error reporting with unique `HOM` codes, related information links, and "Unnecessary" tags for redundant code (e.g., localization version numbers).
- **Hyperlinked Tooltips:** All file and texture paths in hovers are clickable, allowing you to navigate your project at light speed. Scope stacks and emoji-categorized headers provide rich context.
- **Smart Completion:** Context-aware suggestions for triggers, effects, scopes, abilities, adjacency rules, sound effects, and localization commands.
- **Color Support:** Integrated color picker and previews for `rgb`, `hsv`, and Paradox ideology color formats.
- **CSV Formatting:** Semantic column-alignment formatter for `definition.csv` and `adjacencies.csv`.
- **Bulk Code Actions:** Fix all styling issues (casing, whitespace, indentation, unescaped quotes) in a file with a single command.

## Getting Started

1. **Install the extension** from the VS Code Marketplace.
2. **Set your Game Path:** Open VS Code settings (`Ctrl+,`) and search for `hoi4.gamePath`. Set this to your HOI4 installation directory (e.g., `C:\Program Files (x86)\Steam\steamapps\common\Hearts of Iron IV`).
3. **Open your Mod:** Simply open your mod folder in VS Code, and the server will start automatically.

## Commands

- `Hearts of Modding: Activate Extension`: Manually starts the LSP server if it hasn't already.
- `Hearts of Modding: Set Game Path`: Quickly update your game path for VFS merging.
- `Hearts of Modding: Toggle Styling Checks`: Enable or disable cosmetic casing and whitespace diagnostics.
- `Hearts of Modding: Toggle Workspace Scan`: Enable or disable automatic workspace-wide diagnostic scanning.
- `Hearts of Modding: Show Memory Usage`: Toggle the real-time memory usage display in the status bar.

## Requirements

- VS Code version 1.75.0 or higher.
- (Optional) Hearts of Iron IV installation for full VFS and vanilla game data features.

---

*Made with ❤️ by a [Hearts of Minecraft](https://steamcommunity.com/sharedfiles/filedetails/?id=2624254320) mod developer.*
