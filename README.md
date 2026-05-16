# Hearts of Modding

[![made-for-VSCode](https://img.shields.io/badge/Made%20for-VSCode-1f425f.svg)](https://code.visualstudio.com/) [![GitHub license](https://badgen.net/github/license/Naereen/Strapdown.js)](https://github.com/Naereen/StrapDown.js/blob/master/LICENSE) [![made-with-rust](https://img.shields.io/badge/Made%20with-Rust-1f425f.svg)](https://www.rust-lang.org/)

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/emberglazee/hearts-of-modding/.github%2Fworkflows%2Fbuild.yml?style=flat-square&link=https%3A%2F%2Fgithub.com%2Femberglazee%2Fhearts-of-modding%2Factions%2Fworkflows%2Fbuild.yml) ![GitHub Release](https://img.shields.io/github/v/release/emberglazee/hearts-of-modding?style=flat-square&label=release&link=https%3A%2F%2Fgithub.com%2Femberglazee%2Fhearts-of-modding%2Freleases)

**Hearts of Modding** is a heavily work in progress, high-performance Visual Studio Code extension specifically tailored for **Hearts of Iron IV (HOI4)** modders. Powered by a specialized Language Server Protocol (LSP) written in Rust, it provides a responsive, accurate, and deeply integrated modding experience that goes beyond standard Paradox scripting tools.

Inspired by the extensions [CWTools](https://github.com/cwtools/cwtools) and [VModer](https://github.com/textGamex/VModer).

## Key Features

### 🚀 High-Performance Intelligence
- **Rust-Powered LSP:** Quick response times even in massive mods like Millennium Dawn or Kaiserreich.
- **Parallelized Workspace Scanning:** Rapidly indexes your entire mod and the vanilla game files on startup.
- **Virtual File System (VFS):** Correctly respects mod loading priority, ensuring your mod's overrides are always accurately represented.

### 🔍 Specialized Discovery & Tracking
- **Achievement & Ribbon Indexing:** Comprehensive support for achievements and ribbons, featuring specialized tooltips (🏆/🎀) and direct definition linking.
- **Workspace-Wide Symbols:** Global fuzzy search (`Ctrl+T`) for all indexed symbols (Events, Ideas, Achievements, Sprites, etc.).
- **Call Hierarchy:** Visualize incoming and outgoing relationships for events and scripted entities to understand your mod's flow.
- **Deep Modifier Detection:** Automatically indexes custom and dynamic modifiers, linking them directly to their source definitions.
- **Virtual File System (VFS):** Correctly respects mod loading priority, ensuring your mod's overrides are always accurately represented.

### 🛡️ Advanced Validation
- **Deep Schema Validation (CWT):** Powered by an engine that supports the full CWTools specification, ensuring triggers, effects, and cardinality are correct.
- **Logical Integrity Checks:** Validates character skills, building levels, and victory point locations against game definitions and defines.
- **Proactive Workspace Scan:** Optionally scan your entire mod directory for errors upon initialization—no need to open every file manually.
- **Localization Infrastructure:** Deep parsing and validation of over 80 localization commands and complex chains like `[Root.GetTag]`.
- **Paradox Styling Rules:** Optional checks for standard Paradox casing conventions, indentation (tabs vs. spaces), and trailing whitespace.

### 🛠️ Developer Productivity
- **Safe LSP Rename:** Rename Events, Scripted Triggers, Ideas, and Variables across your entire project with confidence.
- **Diagnostic Enhancements:** Detailed error reporting with unique `HOM` codes, related information links, and "Unnecessary" tags for redundant code.
- **Hyperlinked Tooltips:** All file and texture paths in hovers are clickable, allowing you to navigate your project at light speed.
- **Smart Completion:** Context-aware suggestions for triggers, effects, scopes, and localization commands.
- **Color Support:** Integrated color picker and previews for `rgb`, `hsv`, and Paradox ideology color formats.

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
- (Optional) Latest version of Hearts of Iron IV for full VFS features.

---

*Made with ❤️ by a [Hearts of Minecraft](https://steamcommunity.com/sharedfiles/filedetails/?id=2624254320) mod developer.*
