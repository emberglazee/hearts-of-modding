import * as vscode from 'vscode'

/**
 * Default HOI4 colour values from the official wiki.
 * Used as fallback before the LSP provides scanned color codes.
 * The LSP scans `interface/*.gfx` for the authoritative `textcolors = { ... }`
 * definitions, which may include mod overrides.
 */
const WIKI_DEFAULT_COLORS: Record<string, string> = {
    'C': '#23CEFF', // Cyan
    'L': '#C3B091', // Lilac (dirty orange-gray)
    'B': '#0000FF', // Blue
    'G': '#009F03', // Green
    'R': '#FF3232', // Red
    'b': '#000000', // Black
    'g': '#B0B0B0', // Light gray
    'Y': '#FFBD00', // Yellow
    'H': '#FFBD00', // Header / yellow (same as Y)
    'O': '#FF7019', // Orange
    '0': '#CB00CB', // Purple (Gradient Step 0)
    '1': '#8078D3', // Lilac (Gradient Step 1)
    '2': '#5170F3', // Blue (Gradient Step 2)
    '3': '#518FDC', // Gray-blue (Gradient Step 3)
    '4': '#5ABEE7', // Light blue (Gradient Step 4)
    '5': '#3FB5C2', // Dull cyan (Gradient Step 5)
    '6': '#77CCBA', // Turquoise (Gradient Step 6)
    '7': '#99D199', // Light green (Gradient Step 7)
    '8': '#CCA333', // Orange-yellow (Gradient Step 8)
    '9': '#FCA97D', // White-orange (Gradient Step 9)
    't': '#FF4C4D'  // Vivid red (Gradient Step 10)
}

/** Regex matching HOI4 colour codes: § followed by one alphanumeric or ! */
const COLOR_CODE_RE = /§([a-zA-Z0-9!])/g

/**
 * Manages VS Code editor decorations for HOI4 localisation `.yml` files.
 *
 * Parses the document for §X colour codes and applies decorations with the
 * exact game colours to the text spans between §X and the next § or §!.
 *
 * Colours are obtained from the LSP server (which scans `interface/*.gfx`
 * for `textcolors = { ... }` definitions), falling back to hardcoded wiki
 * defaults when the LSP hasn't provided data yet.
 *
 * The § / §! markers themselves are left for the LSP semantic tokens to
 * highlight (as EnumMember) — only the rendered content gets coloured.
 */
export class LocColorDecorator {
    private disposables: vscode.Disposable[] = []
    private decorationTypes = new Map<string, vscode.TextEditorDecorationType>()

    /**
     * Active colour map: LSP-provided values merged over wiki defaults.
     * Updated via `updateColors()` when the LSP sends scanned data.
     */
    private activeColorMap: Record<string, string> = { ...WIKI_DEFAULT_COLORS }

    /** True once the LSP has provided scanned color codes. */
    private hasLspColors = false

    /**
     * Activate the decorator: listen for editor and document changes,
     * and apply decorations to the active editor immediately.
     */
    activate(): void {
        // Decorate when switching to a different editor tab
        this.disposables.push(
            vscode.window.onDidChangeActiveTextEditor(editor => {
                if (editor) this.updateDecorations(editor)
            })
        )

        // Decorate when the document content changes (typing, undo, etc.)
        this.disposables.push(
            vscode.workspace.onDidChangeTextDocument(event => {
                const editor = vscode.window.activeTextEditor
                if (editor && event.document === editor.document) {
                    this.updateDecorations(editor)
                }
            })
        )

        // Decorate the currently active editor on startup
        if (vscode.window.activeTextEditor) {
            this.updateDecorations(vscode.window.activeTextEditor)
        }
    }

    /**
     * Update the colour map with data from the LSP server.
     *
     * The LSP scans `interface/*.gfx` for `textcolors = { ... }` entries,
     * capturing both vanilla colours and mod overrides. Calling this
     * replaces the active colour map entirely with LSP data, then
     * re-applies decorations if an editor is active.
     *
     * @param colorMap  Symbol → hex colour map from the LSP
     *                  (e.g. `{ "R": "#FF3232", "G": "#009F03" }`)
     */
    updateColors(colorMap: Record<string, string>): void {
        this.activeColorMap = { ...WIKI_DEFAULT_COLORS, ...colorMap }
        this.hasLspColors = true

        // Re-apply decorations immediately with the new colour map
        const editor = vscode.window.activeTextEditor
        if (editor) {
            this.updateDecorations(editor)
        }
    }

    /**
     * Get or lazily create a decoration type for a given hex colour.
     */
    private getDecorationType(color: string): vscode.TextEditorDecorationType {
        let dt = this.decorationTypes.get(color)
        if (!dt) {
            dt = vscode.window.createTextEditorDecorationType({
                color: color,
                rangeBehavior: vscode.DecorationRangeBehavior.ClosedClosed
            })
            this.decorationTypes.set(color, dt)
        }
        return dt
    }

    /**
     * Recompute and apply colour decorations for a text editor.
     *
     * Parses the editor's document for §X colour codes and builds a map
     * of hex colour → Range[]. Any decoration types that no longer have
     * matching ranges are cleared (their ranges set to []).
     *
     * Only operates on documents with languageId === 'hoi4-localisation'.
     */
    updateDecorations(editor: vscode.TextEditor): void {
        if (editor.document.languageId !== 'hoi4-localisation') return

        const text = editor.document.getText()
        const colorRanges = new Map<string, vscode.Range[]>()

        let match: RegExpExecArray | null
        let activeColor: string | null = null
        let rangeStart: number | null = null

        COLOR_CODE_RE.lastIndex = 0

        while ((match = COLOR_CODE_RE.exec(text)) !== null) {
            const code = match[1]
            const markerEnd = match.index + 2  // offset right after the 2-byte §X

            // Emit coloured range up to this marker
            if (activeColor && rangeStart !== null && rangeStart < match.index) {
                this.addRange(colorRanges, activeColor, new vscode.Range(
                    editor.document.positionAt(rangeStart),
                    editor.document.positionAt(match.index)
                ))
            }

            if (code === '!') {
                // Reset to default text colour
                activeColor = null
                rangeStart = null
            } else if (this.activeColorMap[code]) {
                activeColor = this.activeColorMap[code]
                rangeStart = markerEnd
            } else {
                // Unknown colour code — treat as reset (game will log an error)
                activeColor = null
                rangeStart = null
            }
        }

        // Handle trailing coloured text with no closing §!
        if (activeColor && rangeStart !== null && rangeStart < text.length) {
            this.addRange(colorRanges, activeColor, new vscode.Range(
                editor.document.positionAt(rangeStart),
                editor.document.positionAt(text.length)
            ))
        }

        // Apply decorations
        const seenColours = new Set(colorRanges.keys())
        for (const [color, ranges] of colorRanges) {
            editor.setDecorations(this.getDecorationType(color), ranges)
        }
        // Clear decoration types that no longer have ranges
        for (const color of this.decorationTypes.keys()) {
            if (!seenColours.has(color)) {
                editor.setDecorations(this.getDecorationType(color), [])
            }
        }
    }

    /**
     * Helper to add a range to a colour bucket, creating the bucket if needed.
     */
    private addRange(
        map: Map<string, vscode.Range[]>,
        color: string,
        range: vscode.Range
    ): void {
        const ranges = map.get(color)
        if (ranges) {
            ranges.push(range)
        } else {
            map.set(color, [range])
        }
    }

    /**
     * Dispose all decoration types and event listeners.
     */
    dispose(): void {
        for (const dt of this.decorationTypes.values()) {
            dt.dispose()
        }
        this.decorationTypes.clear()
        for (const d of this.disposables) {
            d.dispose()
        }
        this.disposables = []
    }

    /**
     * Whether the LSP has provided colour data (as opposed to wiki defaults).
     */
    get isUsingLspColors(): boolean {
        return this.hasLspColors
    }
}
