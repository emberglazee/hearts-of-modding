# Hearts of Modding

[![made-for-VSCode](https://img.shields.io/badge/Made%20for-VSCode-1f425f.svg)](https://code.visualstudio.com/)
[![GitHub license](https://badgen.net/github/license/emberglazee/Hearts-of-Modding)](https://github.com/emberglazee/Hearts-of-Modding/blob/main/LICENSE)
![GitHub top language](https://img.shields.io/github/languages/top/emberglazee/hearts-of-modding?logo=rust&logoColor=ff8c00&label=Rust)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/emberglazee/hearts-of-modding/.github%2Fworkflows%2Fbuild.yml?logo=githubactions&logoColor=white&link=https%3A%2F%2Fgithub.com%2Femberglazee%2Fhearts-of-modding%2Factions)
![GitHub Release](https://img.shields.io/github/v/release/emberglazee/hearts-of-modding?label=release&logo=github&link=https%3A%2F%2Fgithub.com%2Femberglazee%2Fhearts-of-modding%2Freleases)

<div align="center">
<table>
<tbody>
<td align="center">
<a href="https://github.com/emberglazee/hearts-of-modding" target="_blank"><strong>GitHub</strong></a>
</td>
<td align="center">
<a href="https://github.com/emberglazee/hearts-of-modding/discussions/3" target="_blank"><strong>Discussion</strong></a>
</td>
<td align="center">
<a href="https://marketplace.visualstudio.com/items?itemName=emberglaze.hearts-of-modding" target="_blank"><strong>VSC Marketplace</strong></a>
</td>
<td align="center">
<a href="https://open-vsx.org/extension/emberglaze/hearts-of-modding" target="_blank"><strong>OVSX Registry</strong></a>
</td>
</tbody>
</table>
</div>

An experimental work-in-progress high-performance **Visual Studio Code** extension for **Hearts of Iron IV (HOI4)** modding, powered by a dedicated Language Server Protocol (LSP) server written in **Rust**.

Inspired by [CWTools](https://github.com/cwtools/cwtools) and [VModer](https://github.com/textGamex/VModer). Written with performance in mind.

---

## 🚀 Features

### ✍️ Smarter editing

- **Auto-complete** triggers, modifiers, countries, ideas, traits, abilities.

- **Hover over anything** to see what it does: hover a trigger to read its documentation, an idea to see its modifiers, a state ID to see its name, a localization to preview it.

- **Click to jump** to definitions: F12 on any event, idea, character, or focus takes you straight to where its defined.

### 🔎 Catch mistakes early

- **Errors** for typos, missing brackets, unknown entities, invalid values.

- **Warns** for too high leader skill levels, building levels exceeding their max, or non-existant victory points in a province.

- **Checks** your localization for missing and duplicate keys, unclosed color codes, and invalid scopes/commands.

### 🗺️ Map & data tools

- **Validates `definition.csv` and `adjacencies.csv`**: checks province IDs, terrain names, flags unknown provinces.

- **Hover over map files** to see province terrain, type, and coastal status.

### 🎨 Better visuals

- **Vibrant syntax highlighting** with custom light/dark VSCode themes: color-coded triggers, effects, modifiers, scope references, and localization strings.

- **In-editor color previews**: localization color codes render as their actual colors right in the editor.

## ✨ Get started

1. **Install the extension** from the [VSCode Marketplace](https://marketplace.visualstudio.com/items?itemName=emberglaze.hearts-of-modding) or the [Open VSX Registry](https://open-vsx.org/extension/emberglaze/hearts-of-modding).

2. **Open your mod folder** in VSCode.

3. **(Recommended)** Set `hoi4.gamePath` in your settings to your HOI4 installation for vanilla file references.

4. **Happy modding.**

## 🧑‍🧒 For submods

The extension automatically discovers installed mods and loads them to provide accurate completions and validation in submods as well.

## ⚙️ Might wanna tweak these

| Setting | What it does |
| ------- | ------------ |
| `hoi4.gamePath` | Path to your local HOI4 install. Enables vanilla + submod data support. |
| `hoi4.styling.enabled` | ✨ Styling suggestions ✨ (tabs vs spaces, trailing whitespace, etc). On by default. |
| `hoi4.validator.workspaceScan.enabled` | Scan all mod files for errors on startup. Off by default, slow for huge mods. |
| `hoi4.showMemoryUsage.enabled` | Shows the HoM LSP RAM usage in the status bar. |

## ⌨️ Commands

Type "Hearts of Modding" in the command palette (`Ctrl+Shift+P`) to see all commands:

- **Toggle LSP:** Start/stop the language server.

- **Set Game Path:** Point to your HOI4 installation.

- **Toggle Workspace Scan:** Scan all the files for issues.

- **Toggle Theme:** Switch between the extension themes.

## ☝️ Requirements

- **VSCode v1.82.0+**

- **Windows or Linux (`amd64`/x64) or macOS (`arm64`)**

A HOI4 install is optional but strongly recommended for the complete experience.

---

*Made with ❤️ by a [Hearts of Minecraft](https://steamcommunity.com/sharedfiles/filedetails/?id=2624254320) mod developer.*
