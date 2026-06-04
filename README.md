# Hearts of Modding

[![made-for-VSCode](https://img.shields.io/badge/Made%20for-VSCode-1f425f.svg)](https://code.visualstudio.com/)
[![GitHub license](https://badgen.net/github/license/emberglazee/Hearts-of-Modding)](https://github.com/emberglazee/Hearts-of-Modding/blob/main/LICENSE)
![GitHub top language](https://img.shields.io/github/languages/top/emberglazee/hearts-of-modding?logo=rust&logoColor=ff8c00&label=Rust)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/emberglazee/hearts-of-modding/.github%2Fworkflows%2Fbuild.yml?style=flat-square&logo=githubactions&logoColor=white&link=https%3A%2F%2Fgithub.com%2Femberglazee%2Fhearts-of-modding%2Factions)
![GitHub Release](https://img.shields.io/github/v/release/emberglazee/hearts-of-modding?style=flat-square&label=release&logo=github&link=https%3A%2F%2Fgithub.com%2Femberglazee%2Fhearts-of-modding%2Freleases)

A work-in-progress high-performance **Visual Studio Code** extension for **Hearts of Iron IV (HOI4)** modding, powered by a dedicated Language Server Protocol (LSP) server written in **Rust**.

Inspired by [CWTools](https://github.com/cwtools/cwtools) and [VModer](https://github.com/textGamex/VModer). Built for speed, accuracy, and deep HOI4 modding insight — from total conversions like Millennium Dawn and Kaiserreich down to focused submods.

---

## 🚀 Features at a Glance

### ⚡ Performance & Architecture

- **Rust-Powered LSP**: The language server is a native Rust binary — no JVM, no Node.js overhead, no garbage collection pauses. Sub-millisecond response times on keystroke for any file size.
- **Parallelized Workspace Scanning**: 32+ concurrent scanners index events, ideas, focuses, characters, map data, abilities, states, technologies, and more on startup. Uses thread-safe `DashMap` collections and `ArcSwap` for lock-free config access.
- **Incremental Scanner**: After the initial full scan, file saves trigger a single-file re-parse instead of re-traversing the workspace. Memory-efficient AST caching eliminates redundant parses across language features.
- **jemalloc Allocator**: Switched from the system allocator to a patched `jemalloc` fork, achieving ~40% memory reduction (from 1.03 GB to 608 MB on large mods like Millenium Dawn).
- **String Interning**: All scanner entity keys use deduplicated `Arc<str>` to minimize memory pressure on large workspaces.

### 📚 Virtual File System (VFS) & Submod Support

The extension builds a **layered virtual filesystem** that mirrors HOI4's mod loading order:

```
Vanilla game files  ←  Dependency mods  ←  Your workspace
(least priority)                              (wins always)
```

- **Auto-Discovery**: If `hoi4.gamePath` is set, the extension automatically parses your workspace's `descriptor.mod` for `dependencies = { ... }`, resolves each dependency name to its directory via the Paradox mod registry, and stacks them in the VFS.
- **OS-Aware Registry Detection**: Default registry paths for Linux (`~/.local/share/Paradox Interactive/Hearts of Iron IV/mod`), Windows (`%USERPROFILE%/Documents/Paradox Interactive/Hearts of Iron IV/mod`), and macOS (`~/Documents/Paradox Interactive/Hearts of Iron IV/mod`) are detected automatically. Override via `hoi4.modRegistryPath`.
- **Manual Overrides**: Explicit paths added to `hoi4.modPaths` are stacked after auto-discovered dependencies (higher priority). Use this for dependencies not declared in `descriptor.mod` or to override auto-resolution.
- **Layered Scanning**: Each VFS layer is scanned independently and results are merged at query time. Files in your workspace override dependency mods, which override vanilla — accurate to HOI4 engine behaviour.
- **VFS-Aware Diagnostics**: Validation rules — duplicate localization keys, sound effect definitions, texture references — correctly account for VFS layering and `replace/` directory semantics.

### 🎨 Editing Experience

- **Hybrid Syntax Highlighting**: A carefully tuned blend of TextMate grammar (structural patterns, operators, comments) and **semantic tokens** (context-aware highlighting for triggers, effects, modifiers, entity names, scopes). Semantic tokens use the full scanner database as their source of truth, so custom entities (achievements, characters, ideas, abilities) are highlighted on par with built-in keywords.
- **Custom VS Code Themes**: Dedicated **Hearts of Modding Dark** and **Light** themes with expanded colour ranges for semantic tokens — localization keys, CSV columns, GFX entities, and HOI4-specific syntax elements get distinct, meaningful colours.
- **Smart Completions**: Context-aware suggestions for triggers, effects, scopes, abilities, adjacency rules, sound effects, localization commands, and more — drawing from both built-in databases and your workspace's scanned entities.
- **Rich Hover Tooltips**: Hover over any identifier to see localized names, stats, resolved GFX sprite previews, formatted modifier/effect summaries, scope context, and full documentation. All file and texture paths in tooltips are clickable.
  - **Character Portraits**: Resolve `GFX_` sprite references and direct file paths for characters, with linked previews.
  - **State Cross-Referencing**: Hover over state IDs in triggers/effects (`owns_state = 123`) to see the state's localized name, ID, and definition source.
  - **Ability Cards**: Formatted displays of cooldown, cost, duration, type, and modifier/effect summaries.
  - **Province Lookups**: Column-snapping tooltips in map files resolve province IDs to terrain, type, and coastal status.
  - **Scope Context**: Emoji-categorized scope stack headers show where you are in the document hierarchy.
- **Go to Definition (F12)**: Jump to entity definitions across your entire workspace — events, ideas, characters, achievements, abilities, traits, technologies, states, provinces, strategic regions, scripted triggers/effects, sound effects, and more.
- **Safe Rename (F2)**: Cross-file rename for events, scripted triggers/effects, ideas, characters, abilities, and variables. Searches both open documents and unopened workspace files.
- **Call Hierarchy**: Visualize incoming and outgoing relationships for events and scripted entities — understand your mod's flow at a glance.
- **Workspace Symbols (Ctrl+T)**: Global fuzzy search across all indexed symbols: events, ideas, achievements, characters, states, abilities, traits, sprites, sound effects, scripted localisation, localization keys, modifiers, sub-ideologies, technologies, and more.

### 🔍 Scanners & Intelligence

| Scanner | What it indexes |
|---------|----------------|
| **Event Scanner** | Events, event chains, on_actions |
| **National Focus Scanner** | Focus trees, shared focuses, bypass conditions |
| **Idea Scanner** | Country ideas, design companies, sub-ideologies |
| **Ideology Scanner** | Ideology and sub-ideology definitions |
| **Character Scanner** | Characters, roles (advisor/country leader/unit leader), traits, skills, portraits |
| **Achievement Scanner** | Achievements and ribbons, including custom achievements |
| **Ability Scanner** | Leader abilities — cost, duration, cooldown, modifiers, effects |
| **Modifier Scanner** | Custom and dynamic modifiers |
| **Variable Scanner** | Script variables for rename support |
| **Technology Scanner** | Technologies and doctrines |
| **State Scanner** | State definitions, buildings, resources, state categories, victory points |
| **Province Scanner** | Province definitions from `definition.csv` |
| **Strategic Region Scanner** | Strategic regions and naval terrain |
| **Terrain Scanner** | Terrain definitions and category resolution |
| **Country Scanner** | Country tags and metadata |
| **Building Scanner** | Building definitions and level validation |
| **Character Traits Scanner** | Traits used in character definitions |
| **AI Area Scanner** | AI area definitions |
| **AI Strategy Plan Scanner** | Strategy plans and weights |
| **Continent Scanner** | Continent definitions |
| **Map Object Scanner** | `buildings.txt`, `unitstacks.txt`, `weatherpositions.txt` |
| **Logistics Scanner** | `supply_nodes.txt`, `railways.txt` |
| **Adjacency Scanner** | `adjacencies.csv`, `adjacency_rules.txt` |
| **Music Scanner** | Music definitions and tracks |
| **Sound Scanner** | Sound effects, sounds, falloffs, categories |
| **Portrait Scanner** | Portrait definitions and GFX mappings |
| **GFX Scanner** | Sprite definitions, texture references |
| **Sprite Scanner** | Entity sprite definitions |
| **Balance of Power Scanner** | `common/bop/*.txt` definitions |
| **Scripted Loc Scanner** | Scripted localisation keys |
| **Scripted Scanner** | Scripted triggers, scripted effects |
| **Achievement Scanner** | Achievement and ribbon definitions |
| **Country Metadata Scanner** | Country metadata GFX references |
| **State Category Scanner** | State category definitions |
| **Resource Scanner** | Resource definitions |

… and more. All scanners populate a shared `ScannerData` store that powers highlighting, completion, validation, and navigation.

### 🛡️ Validation

- **400+ Diagnostic Codes**: All diagnostics use the `HOM` prefix (`HOM001`–`HOM4002`) with unique codes for easy filtering and suppression.
- **Logical Integrity Checks**: Validates character skills, building levels, victory points, adjacency rules, province connectivity, ability definitions, and more — against actual game mechanic constraints.
- **Map Data Validation**: Deep structural checks on `definition.csv` (column counts, RGB bounds, province types, coastal booleans, coordinates) and `adjacencies.csv` (province references, sea adjacency hints).
- **Texture & GFX Validation**: Verifies that texture paths in `.gfx` and `.gui` files point to existing `.dds`/`.png` files across the entire VFS. Validates spriteType definitions against actual usage.
- **Localization Validation**: 80+ localization commands, ternary logic (1.15+), contextual objects, bindable `$VAR$` variables, escaped quote detection, duplicate key detection (VFS-aware), unclosed colour code detection.
- **Sound Effect Validation**: Cross-file reference checking for sound effects in ability files, with VFS-aware scanning of DLC sound files.
- **Ability Validation**: Warns on missing required fields (`cost`, `duration`, `type`), missing `ai_will_do` block, and unknown ability references.
- **Country Tag Validation**: Validates country tags in localization commands and script files.
- **Paradox Styling Rules**: Optional checks for standard Paradox casing conventions (UI keywords, effects, triggers), indentation (tabs vs. spaces), brace placement, trailing whitespace, end-of-file newline, and unescaped double quotes in localization.
- **Bulk Fix Code Actions**: Fix all styling issues (casing, whitespace, indentation, unescaped quotes) in a file with a single command. Includes `Format Document` support for `.csv` files (semantic column alignment).
- **Proactive Workspace Scan**: Optional (`hoi4.validator.workspaceScan.enabled`) full recursive scan of all `.txt` and `.yml` files in the workspace on startup. Custom ignore patterns supported via `hoi4.validator.ignoreFiles`.

### 🧰 Developer Tooling

- **CSV Formatter**: Semantic column-alignment formatter for `definition.csv` and `adjacencies.csv` — pads columns to their maximum width across the file. Included in standard `Format Document` support.
- **Colour Support**: Integrated colour picker and previews for `rgb`, `hsv`, and Paradox ideology colour formats.
- **Memory Usage Monitor**: Real-time Rust LSP server memory usage in the VS Code status bar. Toggle via command or `hoi4.showMemoryUsage.enabled`.
- **File Watcher**: Dynamic registration of file system watchers for `**/*.{txt,yml,asset,gfx,gui,csv,lua,mod}`. File creations, changes, and deletions trigger appropriate rescans without restarting the server.
- **AI Area & Adjacency Completions**: Context-aware completions for adjacency rule names, province IDs, and common blocks in map files.

---

## 📦 Getting Started

1. **Install** the extension from the [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=emberglaze.hearts-of-modding).
2. **Set your Game Path** (`hoi4.gamePath`): Open VS Code settings (`Ctrl+,`), search for `hoi4.gamePath`, and point it to your HOI4 installation (e.g. `C:\Program Files (x86)\Steam\steamapps\common\Hearts of Iron IV` or `~/.steam/steam/steamapps/common/Hearts of Iron IV`). This is **required** for VFS features, vanilla game data, and submod auto-discovery.
3. **Open your Mod**: Open your mod folder in VS Code. If `descriptor.mod` exists at the workspace root, the extension activates and the LSP starts automatically — no prompt, no config needed. The status bar shows HoM RAM usage once ready.
4. **(Optional) Configure Submod Discovery**: If your mod has `dependencies = { ... }` in its `descriptor.mod`, the extension will automatically resolve and stack them. Override auto-discovery with `hoi4.modRegistryPath` and add extra dependency paths with `hoi4.modPaths`.

---

## ⚙️ Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `hoi4.lsp.enabled` | `true` | Start the language server automatically on workspace open. If disabled, prompts to re-enable. |
| `hoi4.lsp.suppressDisabledPrompt` | `false` | Suppress the LSP disabled prompt on workspace open. |
| `hoi4.gamePath` | `""` | Absolute path to your HOI4 installation. Required for VFS and submod support. |
| `hoi4.modRegistryPath` | `""` | Override the auto-detected Paradox mod registry folder. |
| `hoi4.modPaths` | `[]` | Extra dependency mod paths (absolute). Resolved after auto-discovered dependencies, before workspace. |
| `hoi4.styling.enabled` | `true` | Enable cosmetic styling checks (casing, indentation, whitespace). |
| `hoi4.styling.cosmeticLocalizationIndentation` | `false` | Indent variant localization keys with an extra tab. |
| `hoi4.validator.workspaceScan.enabled` | `false` | Enable automatic workspace-wide diagnostic scan on startup. |
| `hoi4.validator.ignoreFiles` | `[]` | Regex patterns for files/directories to exclude from diagnostics. |
| `hoi4.validator.ignoreLocalization` | `[]` | Regex patterns for localization keys to ignore if missing. |
| `hoi4.showMemoryUsage.enabled` | `true` | Show LSP memory usage in the status bar. |
| `hoi4.themePromptDismissed` | `false` | Whether the HoM theme switch prompt has been dismissed (workspace scope). |

---

## 🎮 Commands

All commands are prefixed with **Hearts of Modding:** in the command palette.

| Command | Description |
|---------|-------------|
| `Toggle LSP` | Start or stop the language server. |
| `Set Game Path` | Quickly update the HOI4 game path for VFS merging. |
| `Toggle Styling Checks` | Enable or disable cosmetic casing/whitespace diagnostics. |
| `Toggle Workspace Scan` | Enable or disable automatic workspace-wide diagnostic scanning. |
| `Show Memory Usage` | Toggle real-time LSP memory usage in the status bar. |
| `Toggle Workspace Theme` | Switch between HoM Dark, HoM Light, or your global theme for this workspace. |

---

## 📋 Requirements

- **VS Code** 1.82.0 or higher.
- **Hearts of Iron IV** (optional, but strongly recommended) — required for full VFS features, vanilla game data, and submod auto-discovery.
- **Platforms**: Linux (`x86_64`), Windows (`x86_64`), macOS (`aarch64`).

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────┐
│             VS Code Extension               │
│         (TypeScript + esbuild)              │
│         ┌───────────────────────┐           │
│         │  Language Client      │           │
│         │  (vscode-language-    │           │
│         │   client)             │           │
│         └──────────┬────────────┘           │
│                    │ LSP                     │
│                    ▼                         │
│         ┌───────────────────────┐           │
│         │  Rust LSP Server      │           │
│         │  (tokio + tower-lsp-  │           │
│         │   server + jemalloc)  │           │
│         └───────────────────────┘           │
└─────────────────────────────────────────────┘
```

The extension has two components:

- **`client/`** — TypeScript VS Code extension. Handles activation, configuration, theme management, and manages the LSP client lifecycle. Bundled with `esbuild` for distribution.
- **`server/`** — Rust LSP server (`tower-lsp-server` + `tokio`). Contains all HOI4-specific intelligence: 32+ file scanners, `nom`-based parsers (script, localization, CSV, defines), scope inference engine, validation rules, formatting, and the VFS layer.

For developers and contributors, see [`AGENTS.md`](AGENTS.md) for the full architecture reference including module layout, data flow, and contribution notes.

---

## 🩹 Troubleshooting

- **"Texture file not found" on sprites that exist**: The extension checks texture paths across the entire VFS stack. If a sprite is missing, verify the texture actually exists in the base game, a dependency mod, or your workspace.
- **Extension not activating**: Ensure your workspace contains a `descriptor.mod` file or manually trigger with `Hearts of Modding: Activate Extension`.
- **High memory usage**: Large total conversion mods may require significant memory. Monitor via the status bar. jemalloc keeps it ~40% lower than the system allocator.
- **Submod dependencies not resolving**: Verify `hoi4.gamePath` is set, and check that the dependency mod name in `descriptor.mod` matches a `.mod` file's `name=` field in the mod registry.

---

*Made with ❤️ by a [Hearts of Minecraft](https://steamcommunity.com/sharedfiles/filedetails/?id=2624254320) mod developer.*
