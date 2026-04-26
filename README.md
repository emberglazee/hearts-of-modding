# Hearts of Modding

**Hearts of Modding** is a heavily work in progress, high-performance Visual Studio Code extension specifically tailored for **Hearts of Iron IV (HOI4)** modders. Powered by a specialized Language Server Protocol (LSP) written in Rust, it provides a responsive, accurate, and deeply integrated modding experience that goes beyond standard Paradox scripting tools.

Inspired by the extensions [CWTools](https://github.com/cwtools/cwtools) and [VModer](https://github.com/textGamex/VModer).

## Key Features

### 🚀 High-Performance Intelligence
- **Rust-Powered LSP:** Instantaneous response times even in massive mods like Millennium Dawn or Kaiserreich.
- **Parallelized Workspace Scanning:** Rapidly indexes your entire mod and the vanilla game files on startup.
- **Virtual File System (VFS):** Correctly respects mod loading priority, ensuring your mod's overrides are always accurately represented.

### 🔍 Specialized Discovery & Tracking
- **Workspace-Wide Variables:** Track `set_variable` and `event_target` usage across your entire project.
- **Event Graphing:** Visualize the complex trigger relationships between events to map out your mod's narrative flow.
- **Deep Modifier Detection:** Automatically indexes custom and dynamic modifiers, linking them directly to their source definitions.
- **Sprite & GFX Indexing:** Hover over sprite names to see their texture paths and jump directly to the `.gfx` or texture file.

### 🛡️ Advanced Validation
- **Real-Time Semantic Checking:** Catch unknown ideology, trait, idea, or GFX references as you type.
- **Dynamic Data Validation:** Reads `map/definition.csv` to ensure every province ID you use actually exists.
- **Localization Scopes:** Deep parsing and validation of complex localization chains like `[Root.GetTag]` or `[THIS.GetName]`.
- **Paradox Styling Rules:** Optional checks for standard Paradox casing conventions, indentation (tabs vs. spaces), and trailing whitespace.

### 🛠️ Developer Productivity
- **Hyperlinked Tooltips:** All file and texture paths in hovers are clickable, allowing you to navigate your project at light speed.
- **Smart Completion:** Context-aware suggestions for triggers, effects, scopes, and localization commands.
- **Go to Definition:** Jump from a script directly to a trait definition, an event, or a localization key.
- **Color Support:** Integrated color picker and previews for `rgb`, `hsv`, and Paradox ideology color formats.

## Getting Started

1. **Install the extension** from the VS Code Marketplace.
2. **Set your Game Path:** Open VS Code settings (`Ctrl+,`) and search for `hoi4.gamePath`. Set this to your HOI4 installation directory (e.g., `C:\Program Files (x86)\Steam\steamapps\common\Hearts of Iron IV`).
3. **Open your Mod:** Simply open your mod folder in VS Code, and the server will start automatically.

## Commands

- `HOI4: Activate Extension`: Manually starts the LSP server if it hasn't already.
- `HOI4: Set Game Path`: Quickly update your game path for VFS merging.
- `HOI4: Toggle Styling Checks`: Enable or disable cosmetic casing and whitespace diagnostics.

## Requirements

- VS Code version 1.75.0 or higher.
- (Optional) Latest version of Hearts of Iron IV for full VFS features.

---

*Made with ❤️ by a Hearts of Minecraft mod developer*
