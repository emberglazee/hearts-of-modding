import * as vscode from 'vscode'

/**
 * Client-side BOM validation for HOI4 localization files.
 *
 * WHY CLIENT-SIDE: the editor text buffer cannot answer BOM questions —
 * VS Code strips a leading UTF-8 BOM out of `document.getText()` (it lives in
 * the file's encoding, not the buffer), so neither "missing" nor "extra" BOMs
 * are visible in editor-synced text. Raw bytes via
 * `vscode.workspace.fs.readFile` are ground truth.
 *
 * RULE: a localization file must start with EXACTLY ONE UTF-8 BOM
 * (`EF BB BF`). All 2073 vanilla loc files carry exactly one; zero means the
 * game may not load the file's strings; two or more (LLM-generated files
 * routinely double/triple it) put stray U+FEFF characters on line 1 that
 * corrupt the language header. The server flags the same condition from its
 * workspace scan as HOM6005 for unopened files; this module gives live,
 * per-save coverage for files actually open in the editor.
 */

function countLeadingBoms(bytes: Uint8Array): number {
    let n = 0
    while (
        bytes.length >= (n + 1) * 3
        && bytes[n * 3] === 0xef
        && bytes[n * 3 + 1] === 0xbb
        && bytes[n * 3 + 2] === 0xbf
    ) {
        n++
    }
    return n
}

export class BomChecker {
    private diagnosticCollection = vscode.languages.createDiagnosticCollection('hoi4-bom')
    private readonly disposables: vscode.Disposable[] = []
    /** Debounce timer for change-event re-checks. */
    private timer: NodeJS.Timeout | undefined
    /**
     * URIs the server has published diagnostics for at least once. Once the
     * server has spoken about a file, this checker STOPS reporting that file:
     * both sides would otherwise stack an identical HOM6005 in the Problems
     * panel (separate collections merge visually). The server's coverage is
     * authoritative once live — its set is seeded by the workspace scan and
     * re-probed on every save, so from that point on it is never behind this
     * checker. Before first contact (startup window, LSP off), this checker
     * is the only voice.
     */
    private serverCovered = new Set<string>()

    /** Called by extension.ts on every `textDocument/publishDiagnostics`. */
    noteServerPublished(uri: vscode.Uri): void {
        this.serverCovered.add(uri.toString())
        // Server spoke → our copy for this file is redundant by agreement.
        this.diagnosticCollection.delete(uri)
    }

    hasServerSpoken(uri: vscode.Uri): boolean {
        return this.serverCovered.has(uri.toString())
    }

    activate(): void {
        this.disposables.push(this.diagnosticCollection)

        // New file opened → check immediately.
        this.disposables.push(
            vscode.workspace.onDidOpenTextDocument(doc => void this.checkDocument(doc))
        )

        // Saved → the moment disk bytes can change. Prefer the save payload;
        // fall back to reading disk.
        this.disposables.push(
            vscode.workspace.onDidSaveTextDocument(doc => void this.checkDocument(doc))
        )

        // Edited → debounce; only re-check on saves/open unless content
        // changes could have introduced an issue visible in-buffer (a second
        // U+FEFF typed at position 0 IS visible to getText, unlike the first).
        this.disposables.push(
            vscode.workspace.onDidChangeTextDocument(event => {
                const doc = event.document
                if (!this.isLocFile(doc)) return
                if (this.timer) clearTimeout(this.timer)
                this.timer = setTimeout(() => void this.checkDocument(doc), 500)
            })
        )

        // Check whatever is already open when the extension activates.
        for (const doc of vscode.workspace.textDocuments) {
            void this.checkDocument(doc)
        }
    }

    private isLocFile(doc: vscode.TextDocument): boolean {
        return doc.languageId === 'hoi4-localisation' && doc.uri.scheme === 'file'
    }

    private async checkDocument(doc: vscode.TextDocument): Promise<void> {
        if (!this.isLocFile(doc)) return
        // Agreement protocol: once the server has published diagnostics for
        // this URI, it owns BOM reporting for it — stay silent to avoid two
        // stacked copies of HOM6005 in the Problems panel.
        if (this.hasServerSpoken(doc.uri)) return

        let boms: number | undefined
        try {
            // RAW DISK BYTES — the only reliable source for BOM state. The
            // buffer strips the first BOM; extra ones decode invisibly.
            const bytes = await vscode.workspace.fs.readFile(doc.uri)
            boms = countLeadingBoms(bytes)
        } catch {
            // Unsaved new file (untitled:) or unreadable — skip silently;
            // the save path will check once bytes exist on disk.
            return
        }

        if (boms === 1) {
            this.diagnosticCollection.delete(doc.uri)
            return
        }

        const detail =
            boms === 0
                ? 'has NO UTF-8 BOM — the game may fail to load these strings'
                : `has ${boms} UTF-8 BOMs — stray invisible characters corrupt the language header`

        const diagnostic = new vscode.Diagnostic(
            new vscode.Range(0, 0, 0, 1),
            `Localization file must start with EXACTLY ONE UTF-8 BOM: this file ${detail}. `
                + 'Re-save with encoding "UTF-8 with BOM" (status bar / "Change File Encoding") '
                + 'after removing any extra BOM bytes.',
            vscode.DiagnosticSeverity.Error
        )
        diagnostic.source = 'Hearts of Modding'
        diagnostic.code = 'HOM6005'
        this.diagnosticCollection.set(doc.uri, [diagnostic])
    }

    dispose(): void {
        if (this.timer) clearTimeout(this.timer)
        for (const d of this.disposables) d.dispose()
    }
}
