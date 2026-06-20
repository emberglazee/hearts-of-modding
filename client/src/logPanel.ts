import { WebviewView, WebviewViewProvider, WebviewViewResolveContext, CancellationToken } from 'vscode'

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
        webviewView.webview.html = this._getHtml()

        webviewView.webview.onDidReceiveMessage(msg => {
            switch (msg.command) {
                case 'toggleFilter':
                    if (this._filters.has(msg.level)) {
                        this._filters.delete(msg.level)
                    } else {
                        this._filters.add(msg.level)
                    }
                    this._render()
                    break
                case 'clear':
                    this._entries = []
                    this._render()
                    break
                case 'toggleAutoScroll':
                    this._autoScroll = !this._autoScroll
                    this._render()
                    break
            }
        })

        this._render()
    }

    public append(level: string, message: string): void {
        this._entries.push({ level, message, timestamp: Date.now() })
        if (this._entries.length > 5000) {
            this._entries = this._entries.slice(-2500)
        }
        this._render()
    }

    private _render(): void {
        if (!this._view) return

        const filtered = this._entries.filter(e => this._filters.has(e.level))
        const html = filtered.map(e => {
            const color = LEVEL_COLORS[e.level] || '#fff'
            const d = new Date(e.timestamp)
            const time = `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}:${String(d.getSeconds()).padStart(2, '0')}.${String(d.getMilliseconds()).padStart(3, '0')}`
            return `<div class="entry ${e.level.toLowerCase()}"><span class="time">${time}</span> <span class="level" style="color:${color}">[${e.level}]</span> <span class="msg">${this._escapeHtml(e.message)}</span></div>`
        }).join('\n')

        const filterBtns = LEVEL_ORDER.map(l => {
            const active = this._filters.has(l) ? 'active' : ''
            return `<button class="filter ${active}" data-level="${l}" style="color:${LEVEL_COLORS[l]}">${l}</button>`
        }).join(' ')

        this._view.webview.postMessage({
            command: 'render',
            html: `<div class="toolbar">${filterBtns} <button class="filter" data-cmd="clear">✕ Clear</button> <button class="filter ${this._autoScroll ? 'active' : ''}" data-cmd="autoscroll">⬇ Auto</button> <span style="color:#666;font-size:11px;margin-left:auto">${this._entries.length} entries</span></div><div id="log">${html}</div>`,
            scroll: this._autoScroll
        })
    }

    private _escapeHtml(text: string): string {
        return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')
    }

    private _getHtml(): string {
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
<div id="app"><div class="toolbar"><span style="color:#666">HoM Log — waiting for entries...</span></div><div id="log"></div></div>
<script>
const vscode = acquireVsCodeApi();
window.addEventListener('message', (e) => {
    const msg = e.data;
    if (msg.command === 'render') {
        document.getElementById('app').innerHTML = msg.html;
        document.querySelectorAll('.filter[data-level]').forEach((btn) => {
            btn.addEventListener('click', () => vscode.postMessage({ command: 'toggleFilter', level: btn.dataset.level }));
        });
        document.querySelectorAll('.filter[data-cmd="clear"]').forEach((btn) => {
            btn.addEventListener('click', () => vscode.postMessage({ command: 'clear' }));
        });
        document.querySelectorAll('.filter[data-cmd="autoscroll"]').forEach((btn) => {
            btn.addEventListener('click', () => vscode.postMessage({ command: 'toggleAutoScroll' }));
        });
        const logEl = document.getElementById('log');
        if (logEl && msg.scroll !== false) {
            logEl.scrollTop = logEl.scrollHeight;
        }
    }
});
</script>
</body>
</html>`
    }
}
