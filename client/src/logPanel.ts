import { WebviewView, WebviewViewProvider, WebviewViewResolveContext, CancellationToken, window, workspace, env, Uri } from 'vscode'

interface LogEntry {
    level: string
    message: string
    timestamp: number
}

const LEVEL_COLORS: Record<string, string> = {
    ERROR: '#ff4444',
    WARN: '#ffaa00',
    INFO: '#66ccff',
    DEBUG: '#88cc88',
    TRACE: '#888888'
}

const LEVEL_ORDER = ['ERROR', 'WARN', 'INFO', 'DEBUG', 'TRACE']

export class LogPanelProvider implements WebviewViewProvider {
    public static readonly viewType = 'hoi4Log'
    private _view?: WebviewView
    private _entries: LogEntry[] = []
    private _filters: Set<string> = new Set(LEVEL_ORDER)
    private _autoScroll = true

    resolveWebviewView(
        webviewView: WebviewView,
        _context: WebviewViewResolveContext,
        _token: CancellationToken
    ): void {
        this._view = webviewView
        webviewView.webview.options = { enableScripts: true }
        // Full HTML with entries inline — avoids the postMessage race that
        // happens when VS Code recreates the webview on panel switch.
        webviewView.webview.html = this._buildHtml()

        webviewView.webview.onDidReceiveMessage(msg => {
            switch (msg.command) {
                case 'toggleFilter':
                    if (this._filters.has(msg.level)) {
                        this._filters.delete(msg.level)
                    } else {
                        this._filters.add(msg.level)
                    }
                    // Full re-render on filter change (user-initiated, rare)
                    this._fullRender()
                    break
                case 'clear':
                    this._entries = []
                    this._fullRender()
                    break
                case 'toggleAutoScroll':
                    this._autoScroll = !this._autoScroll
                    this._fullRender()
                    break
                case 'copyAll':
                    this._copyAllFiltered()
                    break
                case 'exportAsFile':
                    this._exportAsFile()
                    break
            }
        })
    }

    /** Hot path: called on every server log message. Only pushes a single
     *  entry to the webview instead of rebuilding the entire log HTML. */
    public append(level: string, message: string): void {
        const entry: LogEntry = { level, message, timestamp: Date.now() }
        this._entries.push(entry)
        if (this._entries.length > 5000) {
            this._entries = this._entries.slice(-2500)
        }
        // Send to webview only if the entry passes the current filter.
        // Entries hidden by filter are still stored and shown on a
        // full re-render when the user toggles the filter.
        if (this._view && this._filters.has(level)) {
            this._postAppend(entry)
        }
    }

    // ── Incremental append (hot path) ──

    private _postAppend(entry: LogEntry): void {
        if (!this._view) return
        this._view.webview.postMessage({
            command: 'append',
            entryHtml: this._entryHtml(entry),
            scroll: this._autoScroll
        })
    }

    private _entryHtml(entry: LogEntry): string {
        const color = LEVEL_COLORS[entry.level] || '#fff'
        const d = new Date(entry.timestamp)
        const time = this._formatTime(d)
        return `<div class="entry ${entry.level.toLowerCase()}"><span class="time">${time}</span> <span class="level" style="color:${color}">[${entry.level}]</span> <span class="msg">${this._escapeHtml(entry.message)}</span></div>`
    }

    // ── Full re-render (cold path: filter toggle / clear / initial) ──

    private _fullRender(): void {
        if (!this._view) return
        this._view.webview.postMessage({
            command: 'fullRender',
            html: this._logHtml(),
            scroll: this._autoScroll,
            toolbarHtml: this._toolbarHtml()
        })
    }

    private _logHtml(): string {
        const filtered = this._entries.filter(e => this._filters.has(e.level))
        return filtered.map(e => this._entryHtml(e)).join('\n')
    }

    private _toolbarHtml(): string {
        const filterBtns = LEVEL_ORDER.map(l => {
            const active = this._filters.has(l) ? 'active' : ''
            return `<button class="filter ${active}" data-level="${l}" style="color:${LEVEL_COLORS[l]}">${l}</button>`
        }).join(' ')
        const countLabel = this._entries.length === 0
            ? '<span style="color:#666">HoM Log — waiting for entries...</span>'
            : `<span style="color:#666;font-size:11px;margin-left:auto">${this._entries.length} entries</span>`
        return `<div class="toolbar">${filterBtns} <button class="filter" data-cmd="clear">✕ Clear</button> <button class="filter ${this._autoScroll ? 'active' : ''}" data-cmd="autoscroll">⬇ Auto</button> <button class="filter" data-cmd="copyAll">📋 Copy All</button> <button class="filter" data-cmd="exportAsFile">💾 Export</button> ${countLabel}</div>`
    }

    // ── Full page HTML for initial view creation ──

    private _buildHtml(): string {
        return `<!DOCTYPE html>
<html>
<head>
<style>
    body { font-family: 'Cascadia Code', 'Fira Code', monospace; font-size: 12px; padding: 0; margin: 0; background: #1e1e1e; color: #ccc; }
    .toolbar { position: sticky; top: 0; background: #252526; padding: 6px 8px; border-bottom: 1px solid #333; display: flex; gap: 4px; flex-wrap: wrap; z-index: 10; }
    .filter { background: #333; border: 1px solid #555; color: #ccc; padding: 2px 8px; cursor: pointer; border-radius: 3px; font-size: 11px; }
    .filter.active { background: #555; border-color: #888; }
    .filter:hover { background: #444; }
    #log { padding: 4px 8px; overflow-y: auto; max-height: calc(100vh - 50px); }
    .entry { padding: 1px 0; line-height: 1.5; }
    .entry:hover { background: #2a2a2a; }
    .time { color: #666; font-size: 11px; }
    .level { font-weight: bold; }
    .msg { color: #ccc; word-break: break-all; }
    .entry.error .msg { color: #ff6666; }
    .entry.warn .msg { color: #ffcc66; }
    .entry.trace .msg { color: #888; }
</style>
</head>
<body>
<div id="app">${this._toolbarHtml()}<div id="log" data-autoscroll="${this._autoScroll}">${this._logHtml()}</div></div>
<script>
const vscode = acquireVsCodeApi();
// Auto-scroll to bottom on initial load if autoscroll is enabled
(function() {
    const logEl = document.getElementById('log');
    if (logEl && logEl.dataset.autoscroll === 'true') {
        logEl.scrollTop = logEl.scrollHeight;
    }
})();
window.addEventListener('message', (e) => {
    const msg = e.data;
    if (msg.command === 'fullRender') {
        // Full replace — used on filter toggle and clear (user-initiated, rare)
        document.getElementById('app').innerHTML = msg.toolbarHtml + '<div id="log">' + msg.html + '</div>';
        _bindFilters();
        const logEl = document.getElementById('log');
        if (logEl && msg.scroll !== false) {
            logEl.scrollTop = logEl.scrollHeight;
        }
    } else if (msg.command === 'append') {
        // Incremental append — used on every new log entry (hot path)
        const logEl = document.getElementById('log');
        if (!logEl) return;
        logEl.insertAdjacentHTML('beforeend', msg.entryHtml);
        if (msg.scroll !== false) {
            logEl.scrollTop = logEl.scrollHeight;
        }
    }
});
function _bindFilters() {
    document.querySelectorAll('.filter[data-level]').forEach((btn) => {
        btn.addEventListener('click', () => vscode.postMessage({ command: 'toggleFilter', level: btn.dataset.level }));
    });
    document.querySelectorAll('.filter[data-cmd="clear"]').forEach((btn) => {
        btn.addEventListener('click', () => vscode.postMessage({ command: 'clear' }));
    });
    document.querySelectorAll('.filter[data-cmd="autoscroll"]').forEach((btn) => {
        btn.addEventListener('click', () => vscode.postMessage({ command: 'toggleAutoScroll' }));
    });
    document.querySelectorAll('.filter[data-cmd="copyAll"]').forEach((btn) => {
        btn.addEventListener('click', () => vscode.postMessage({ command: 'copyAll' }));
    });
    document.querySelectorAll('.filter[data-cmd="exportAsFile"]').forEach((btn) => {
        btn.addEventListener('click', () => vscode.postMessage({ command: 'exportAsFile' }));
    });
}
_bindFilters();
</script>
</body>
</html>`
    }

    // ── Utility ──

    private _formatTime(d: Date): string {
        return `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}:${String(d.getSeconds()).padStart(2, '0')}.${String(d.getMilliseconds()).padStart(3, '0')}`
    }

    private _formatFilteredEntries(): string {
        const filtered = this._entries.filter(e => this._filters.has(e.level))
        return filtered.map(e => {
            return `[${this._formatTime(new Date(e.timestamp))}] [${e.level}] ${e.message}`
        }).join('\n')
    }

    private async _copyAllFiltered(): Promise<void> {
        if (!this._entries.length) return
        const text = this._formatFilteredEntries()
        await env.clipboard.writeText(text)
        window.showInformationMessage(`Copied ${this._entries.filter(e => this._filters.has(e.level)).length} log entries to clipboard`)
    }

    private async _exportAsFile(): Promise<void> {
        if (!this._entries.length) return
        const filtered = this._entries.filter(e => this._filters.has(e.level))
        const uri = await window.showSaveDialog({
            filters: { 'Log Files': ['log', 'txt'], 'All Files': ['*'] },
            defaultUri: Uri.file(`hoi4-log-${Date.now()}.log`),
            title: 'Export HoM Log'
        })
        if (!uri) return
        const text = this._formatFilteredEntries()
        await workspace.fs.writeFile(uri, new TextEncoder().encode(text))
        window.showInformationMessage(`Exported ${filtered.length} log entries to ${uri.fsPath}`)
    }

    private _escapeHtml(text: string): string {
        return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')
    }
}
