import * as path from 'path'
import * as fs from 'fs'
import { createHash } from 'crypto'
import { workspace, ExtensionContext, window, OutputChannel, commands, StatusBarAlignment, ConfigurationTarget, StatusBarItem, FileSystemWatcher } from 'vscode'

// The extension host runs on Node >=18 where `fetch` is a global, but the
// project's @types/node (18.15) predates the global fetch typings — declare the
// minimal surface we use so `tsc` stays green without pulling in DOM libs.
declare function fetch(input: string, init?: object): Promise<{
    ok: boolean
    status: number
    arrayBuffer(): Promise<ArrayBuffer>
}>

import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node'

import { LocColorDecorator } from './locColorDecorator'
import { LogPanelProvider } from './logPanel'

let client: LanguageClient
let outputChannel: OutputChannel
let logPanelProvider: LogPanelProvider
let memoryInterval: NodeJS.Timeout | undefined
let locColorDecorator: LocColorDecorator

// Owned by the extension, NOT by the language client. `synchronize.fileEvents`
// routes through FileSystemWatcherFeature.registerRaw(), which stores only the
// onDidCreate/onDidChange/onDidDelete *subscriptions* — never the watcher
// objects themselves (vscode-languageclient 9.0.1, fileSystemWatcher.js:51-57;
// contrast the dynamic register() path at :29-50, which pushes the watcher it
// created at :47). `client.stop()` reaches that feature's clear() via
// cleanUp() (client.js:997-1011) and disposes the subscriptions, so a watcher
// created per-start was orphaned on every toggle-off. Created once here and
// reused; hookFileEvents runs inside each start() (client.js:888), so every
// client attaches fresh listeners to the same watchers.
let fileEventWatchers: FileSystemWatcher[] | undefined

// Serializes every LSP lifecycle transition. `isRunning()` is
// `$state === Running` only (client.js:639), and the ClientState enum
// (client.js:164-172) has distinct `Starting`/`Stopping` states — so a guard
// built on it is blind during both transitions.
let lifecycle: Promise<unknown> = Promise.resolve()

/// Run `op` after every previously queued lifecycle transition has settled.
///
/// Without this, a second toggle arriving mid-transition saw `isRunning()`
/// false, started a SECOND hom-lsp process, and overwrote the module-level
/// `client` — orphaning the first server with its whole workspace scan
/// resident and no remaining handle to stop it.
///
/// The `.catch` on the chain is load-bearing: a rejected transition must not
/// poison `lifecycle`, or one failed start wedges every later toggle.
function serialize<T>(op: () => Promise<T>): Promise<T> {
    const next = lifecycle.then(op, op)
    lifecycle = next.catch(() => undefined)
    return next
}

function formatBytes(bytes: number): string {
    if (!Number.isFinite(bytes) || bytes <= 0) {
        return '0 Bytes'
    }
    const k = 1024
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    const size = (bytes / Math.pow(k, i)).toFixed(2)
    return `${size} ${sizes[i]}`
}

// ── hom-lsp binary resolution ────────────────────────────────────────────────
// The server binary ships as `hom-lsp-<os>-<arch>[.exe]`, bundled in the VSIX
// for the three "primary" combos (linux-amd64, win-amd64, macos-arm64) and
// published as standalone release assets for ALL combos. Any other platform/arch
// downloads its binary from the matching GitHub release (pinned to the installed
// extension version, falling back to latest) into the extension's global storage.
const HOM_REPO = 'emberglazee/Hearts-of-Modding'

function homLspAssetName(platform: string, arch: string): string {
    const os = platform === 'win32' ? 'win' : platform === 'darwin' ? 'macos' : 'linux'
    const a = arch === 'x64' ? 'amd64' : arch === 'arm64' ? 'arm64' : arch
    return `hom-lsp-${os}-${a}${platform === 'win32' ? '.exe' : ''}`
}

function logInfo(msg: string): void {
    logPanelProvider.append('INFO', msg)
    outputChannel.appendLine(msg)
}

function logWarn(msg: string): void {
    logPanelProvider.append('WARN', msg)
    outputChannel.appendLine(msg)
}

/// Network budgets for resolving the server binary.
///
/// `SHA256SUMS` is a few hundred bytes; a release binary is ~8-9 MB, so 120s
/// tolerates a link as slow as ~75 KB/s (roughly 0.6 Mbit/s) before aborting a
/// download that would have succeeded. One blanket cap would either false-abort
/// on a slow link or leave the tiny checksum request hanging for minutes.
///
/// `DOWNLOAD_BUDGET_MS` bounds the WHOLE resolution: per-request caps multiply
/// across base URLs (2 bases x 2 requests each), so without an overall deadline
/// a black-holed network still stalls activate() for far longer than any single
/// cap suggests.
const SHA_FETCH_TIMEOUT_MS = 15_000
const BINARY_FETCH_TIMEOUT_MS = 120_000
const DOWNLOAD_BUDGET_MS = 150_000

/// An aborted fetch rejects with a DOMException named `AbortError`. Logging
/// that as a generic failure is what makes a stalled network indistinguishable
/// from a 404 in a bug report.
function isAbortError(err: unknown): boolean {
    return typeof err === 'object' && err !== null
        && (err as { name?: string }).name === 'AbortError'
}

/// `fetch` with a deadline covering the response BODY, not just the headers.
///
/// `fetch` resolves as soon as headers arrive, so a timer cleared at that point
/// leaves `arrayBuffer()` free to hang forever on a transfer that stalls
/// mid-body — the realistic failure mode. Keeping one signal live across both
/// awaits means the body stream is aborted too. The timer is always cleared:
/// a pending two-minute timer would otherwise keep a handle on the event loop
/// after a fast response.
///
/// `AbortController` rather than `AbortSignal.timeout()` because each request
/// is clamped to whatever remains of the overall budget, which the static
/// helper cannot express.
async function fetchWithDeadline(
    url: string,
    timeoutMs: number
): Promise<{ ok: true, buf: Buffer } | { ok: false, status: number }> {
    const ac = new AbortController()
    const timer = setTimeout(() => ac.abort(), timeoutMs)
    try {
        const res = await fetch(url, { signal: ac.signal })
        if (!res.ok) return { ok: false, status: res.status }
        return { ok: true, buf: Buffer.from(await res.arrayBuffer()) }
    } finally {
        clearTimeout(timer)
    }
}

/// Why a checksum lookup needs three outcomes rather than `string | undefined`:
/// "this release predates checksums" and "the checksum request failed" are
/// both absences, but only the first may proceed to an unverified install.
/// Collapsing them lets one flaky request silently downgrade a verified
/// download path to an unverified one.
type ChecksumLookup =
    | { kind: 'found', digest: string }
    | { kind: 'absent' }
    | { kind: 'unavailable' }

/// Fetch `SHA256SUMS` for a release and return the digest for `asset`.
///
/// `absent` covers a 404 or a file with no line for this asset — releases
/// published before checksums existed, which install unverified rather than
/// breaking. `unavailable` is a network failure or timeout, which must NOT be
/// treated as permission to skip verification.
async function fetchExpectedSha256(
    baseUrl: string,
    asset: string,
    timeoutMs: number
): Promise<ChecksumLookup> {
    let res: { ok: true, buf: Buffer } | { ok: false, status: number }
    try {
        res = await fetchWithDeadline(`${baseUrl}/SHA256SUMS`, timeoutMs)
    } catch (err) {
        logWarn(
            `Could not fetch SHA256SUMS from ${baseUrl}: ` +
            (isAbortError(err) ? `timed out after ${timeoutMs}ms` : `${err}`)
        )
        return { kind: 'unavailable' }
    }
    if (!res.ok) {
        // Only a 404 means "no checksums for this release". A 5xx is a failure
        // to answer, not an answer.
        return res.status === 404 ? { kind: 'absent' } : { kind: 'unavailable' }
    }
    const text = res.buf.toString('utf8')
    for (const line of text.split('\n')) {
        // `sha256sum` format: "<64 hex>  <filename>"
        const m = line.trim().match(/^([a-fA-F0-9]{64})\s+\*?(.+)$/)
        if (m && m[2].trim() === asset) return { kind: 'found', digest: m[1].toLowerCase() }
    }
    return { kind: 'absent' }
}

/// Delete cached binaries for versions other than the one in use, so global
/// storage does not accumulate a ~10 MB binary per extension update.
function pruneOldBinaryCaches(rootDir: string, keepVersion: string): void {
    try {
        if (!fs.existsSync(rootDir)) return
        for (const entry of fs.readdirSync(rootDir)) {
            if (entry === keepVersion) continue
            fs.rmSync(path.join(rootDir, entry), { recursive: true, force: true })
        }
    } catch {
        // Best-effort cleanup — never block startup on it.
    }
}

async function downloadHomLspBinary(
    context: ExtensionContext,
    asset: string
): Promise<string | null> {
    const version: string = (context.extension?.packageJSON?.version as string) ?? '0.0.0'
    const root = path.join(context.globalStorageUri.fsPath, 'hom-lsp')
    const dir = path.join(root, version)
    const dst = path.join(dir, asset)

    if (fs.existsSync(dst)) {
        logInfo(`Using cached hom-lsp binary at: ${dst}`)
        pruneOldBinaryCaches(root, version)
        return dst
    }

    // One deadline for the whole resolution, not just per request — see the
    // note on DOWNLOAD_BUDGET_MS.
    const deadline = Date.now() + DOWNLOAD_BUDGET_MS
    const remaining = () => deadline - Date.now()

    // Pin to the release matching the installed extension version; fall back to
    // the latest release if that tag doesn't exist yet.
    const bases = [
        `https://github.com/${HOM_REPO}/releases/download/v${version}`,
        `https://github.com/${HOM_REPO}/releases/latest/download`
    ]

    for (const base of bases) {
        if (remaining() <= 0) {
            logWarn(`Gave up resolving ${asset}: exceeded the ${DOWNLOAD_BUDGET_MS}ms download budget.`)
            break
        }
        const url = `${base}/${asset}`
        // Write to a temp file and rename only after the bytes check out, so an
        // interrupted or corrupt download can never be served from cache later.
        const tmp = `${dst}.part`
        try {
            logInfo(`No bundled binary for this platform; downloading ${asset} from ${url}...`)
            const res = await fetchWithDeadline(url, Math.min(BINARY_FETCH_TIMEOUT_MS, remaining()))
            if (!res.ok) {
                logWarn(`Download failed (HTTP ${res.status}) from ${url}`)
                continue
            }
            const buf = res.buf

            if (buf.length === 0) {
                logWarn(`Downloaded ${asset} was empty — discarding.`)
                continue
            }

            // The checksum MUST come from the same base as the binary: comparing
            // a pinned-version digest against a `latest` binary is meaningless
            // whichever way it lands.
            const lookup = await fetchExpectedSha256(base, asset, Math.min(SHA_FETCH_TIMEOUT_MS, Math.max(remaining(), 0)))
            if (lookup.kind === 'found') {
                const actual = createHash('sha256').update(new Uint8Array(buf)).digest('hex')
                if (actual !== lookup.digest) {
                    // A proxy or captive portal returning an HTML error page with
                    // HTTP 200 lands here, as does a truncated transfer.
                    //
                    // Terminal, NOT `continue`. Each base is verified against its
                    // own SHA256SUMS, so falling through to `releases/latest`
                    // would fetch a DIFFERENT release, verify it successfully
                    // against its own checksum, and silently install a server
                    // that does not match this extension. A failed integrity
                    // check must never be answered by installing something else.
                    logWarn(
                        `Checksum mismatch for ${asset} — expected ${lookup.digest}, got ${actual}. ` +
                        'Refusing to install, and not trying another release.'
                    )
                    return null
                }
                logInfo(`Verified ${asset} against SHA256SUMS.`)
            } else if (lookup.kind === 'absent') {
                // Releases published before SHA256SUMS existed — documented
                // non-fatal contract, so older installs keep working.
                logWarn(`No SHA256SUMS entry for ${asset}; skipping integrity check.`)
            } else {
                // Couldn't reach the checksum: distinct from "there isn't one".
                // Installing unverified here would let one flaky request
                // downgrade a verified path, so try the next base instead.
                logWarn(`Could not verify ${asset} (checksum unavailable) — not installing from ${base}.`)
                continue
            }

            fs.mkdirSync(dir, { recursive: true })
            fs.writeFileSync(tmp, new Uint8Array(buf))
            fs.renameSync(tmp, dst)
            if (process.platform !== 'win32') {
                fs.chmodSync(dst, 0o755)
            }
            logInfo(`Downloaded ${asset} (${buf.length} bytes) to ${dst}`)
            pruneOldBinaryCaches(root, version)
            return dst
        } catch (err) {
            logWarn(
                `Download error from ${url}: ` +
                (isAbortError(err) ? 'timed out' : `${err}`)
            )
        } finally {
            // Never leave a partial file behind for a later run to trip over.
            try {
                if (fs.existsSync(tmp)) fs.rmSync(tmp, { force: true })
            } catch {
                /* ignore */
            }
        }
    }
    logWarn(`Could not download hom-lsp binary '${asset}' from any release URL.`)
    return null
}

export async function activate(context: ExtensionContext) {
    outputChannel = window.createOutputChannel('Hearts of Modding')
    console.log('Hearts of Modding extension: activate called')

    const statusBarItem = window.createStatusBarItem(StatusBarAlignment.Right, 100)
    context.subscriptions.push(statusBarItem)

    // ── Register the HoM Log panel provider ──
    logPanelProvider = new LogPanelProvider()
    context.subscriptions.push(
        window.registerWebviewViewProvider(LogPanelProvider.viewType, logPanelProvider)
    )

    // ── Initialise localisation color decorator ──
    locColorDecorator = new LocColorDecorator()
    locColorDecorator.activate()
    context.subscriptions.push(locColorDecorator)

    // ── File watchers fed to the LSP via `synchronize.fileEvents` ──
    // Created here rather than in startServer() so a toggle cycle reuses them
    // (see the note on `fileEventWatchers`). Registered on context.subscriptions
    // so VS Code owns teardown — deactivate() must NOT dispose them again.
    // Must be assigned before the first startServer() call below.
    fileEventWatchers = [
        workspace.createFileSystemWatcher('**/*.txt'),
        workspace.createFileSystemWatcher('**/*.csv')
    ]
    context.subscriptions.push(...fileEventWatchers)

    // ── Command: Show the HoM Log panel ──
    // Registered in activate(), NOT in startServer(): startServer runs again on
    // every LSP toggle, and re-registering an existing command id throws. That
    // throw happened AFTER client.start() had already succeeded, so the LSP came
    // up but everything below the registration was skipped — the
    // `hoi4/colorCodes` listener never attached (loc colours stuck on wiki
    // defaults) and the RAM status-bar poll never started, with no error shown.
    // The panel is a webview owned by the extension, so it works whether or not
    // the server is running.
    context.subscriptions.push(commands.registerCommand('hearts-of-modding.showLog', () => {
        // executeCommand returns a Thenable and rejects asynchronously — it
        // never throws synchronously, so a try/catch here would catch nothing.
        void commands.executeCommand('workbench.view.extension.hoi4-log').then(
            undefined,
            () => { /* view unavailable — the output channel still has the logs */ }
        )
    }))

    context.subscriptions.push(commands.registerCommand('hearts-of-modding.showMemoryUsage', async () => {
        const config = workspace.getConfiguration('hoi4.showMemoryUsage')
        const currentState = config.get('enabled')
        await config.update('enabled', !currentState, true)
        window.showInformationMessage(`Memory Usage Display: ${!currentState ? 'Enabled' : 'Disabled'}`)
    }))

    context.subscriptions.push(commands.registerCommand('hearts-of-modding.toggleTheme', async () => {
        const workbenchConfig = workspace.getConfiguration('workbench')
        const currentTheme = workbenchConfig.inspect<string>('colorTheme')
        const current = currentTheme?.workspaceValue || currentTheme?.globalValue || 'Default Dark+'

        // Friendly display label → registered workbench.colorTheme id.
        // These MUST match the theme "name"/"label" contributed in package.json
        // (themes/hoi4-*-color-theme.json). Passing the short name to
        // workbench.update('colorTheme', ...) would silently set an unknown theme.
        const THEME_OPTIONS = [
            { label: 'HoM Dark', themeId: 'Hearts of Modding Dark' },
            { label: 'HoM Light', themeId: 'Hearts of Modding Light' },
            { label: 'Reset to Global Theme', themeId: undefined }
        ] as const

        const pick = await window.showQuickPick(
            THEME_OPTIONS,
            { placeHolder: `Current: ${current}` }
        )

        if (!pick) return

        if (pick.themeId === undefined) {
            await workbenchConfig.update('colorTheme', undefined, ConfigurationTarget.Workspace)
            window.showInformationMessage('✓ Theme reset to your global preference!')
        } else {
            await workbenchConfig.update('colorTheme', pick.themeId, ConfigurationTarget.Workspace)
            window.showInformationMessage(`✓ Switched to ${pick.themeId}!`)
        }
    }))

    context.subscriptions.push(commands.registerCommand('hearts-of-modding.toggleWorkspaceScan', async () => {
        const config = workspace.getConfiguration('hoi4.validator.workspaceScan')
        const currentState = config.get('enabled')
        await config.update('enabled', !currentState, ConfigurationTarget.Workspace)
        window.showInformationMessage(`Workspace Diagnostic Scan: ${!currentState ? 'Enabled (Re-indexing...)' : 'Disabled'}`)
    }))

    context.subscriptions.push(commands.registerCommand('hearts-of-modding.toggleLsp', async () => {
        // The start-vs-stop decision is read INSIDE the critical section. Read
        // outside it, two quick toggles both observe the same pre-transition
        // state and issue two stops (or two starts) — serializing the actions
        // alone would not prevent that.
        await serialize(async () => {
            if (client && client.isRunning()) {
                if (memoryInterval) {
                    clearInterval(memoryInterval)
                    memoryInterval = undefined
                }
                await client.stop()
                await workspace.getConfiguration('hoi4.lsp').update('enabled', false, ConfigurationTarget.Workspace)
                outputChannel.appendLine('Hearts of Modding LSP stopped.')
                window.showInformationMessage('Hearts of Modding LSP stopped. Toggle again to restart.')
            } else {
                await workspace.getConfiguration('hoi4.lsp').update('enabled', true, ConfigurationTarget.Workspace)
                // startServerInner, not startServer: we already hold the lock,
                // and the wrapper would await a chain containing this callback.
                await startServerInner(context, statusBarItem)
                window.showInformationMessage('Hearts of Modding LSP started!')
            }
        })
    }))

    context.subscriptions.push(commands.registerCommand('hearts-of-modding.setGamePath', async () => {
        const options = {
            canSelectMany: false,
            openLabel: 'Select HOI4 Installation Folder',
            canSelectFiles: false,
            canSelectFolders: true
        }

        const fileUri = await window.showOpenDialog(options)
        if (fileUri && fileUri[0]) {
            const folderPath = fileUri[0].fsPath
            await workspace.getConfiguration('hoi4').update('gamePath', folderPath, true)
            window.showInformationMessage(`HOI4 Game Path set to: ${folderPath}`)
        }
    }))

    context.subscriptions.push(commands.registerCommand('hearts-of-modding.toggleStyling', async () => {
        const config = workspace.getConfiguration('hoi4.styling')
        const currentState = config.get('enabled')
        await config.update('enabled', !currentState, true)
        window.showInformationMessage(`HOI4 Styling Checks: ${!currentState ? 'Enabled' : 'Disabled'}`)
    }))

    // ── LSP auto-start (or prompt if disabled) ──
    const lspConfig = workspace.getConfiguration('hoi4.lsp')
    const lspEnabled = lspConfig.get<boolean>('enabled', true)

    if (lspEnabled) {
        await promptForTheme()
        await startServer(context, statusBarItem)
    } else {
        const suppressed = lspConfig.get<boolean>('suppressDisabledPrompt', false)
        if (!suppressed) {
            const result = await window.showInformationMessage(
                'Hearts of Modding LSP is disabled for this workspace. Language features will not be available.',
                'Enable', 'Stop reminding'
            )
            if (result === 'Enable') {
                await lspConfig.update('enabled', true, ConfigurationTarget.Workspace)
                await promptForTheme()
                await startServer(context, statusBarItem)
            } else if (result === 'Stop reminding') {
                await lspConfig.update('suppressDisabledPrompt', true, ConfigurationTarget.Workspace)
            }
        }
    }

    context.subscriptions.push(workspace.onDidChangeConfiguration(e => {
        if (!client || !client.isRunning()) {
            return
        }
        if (e.affectsConfiguration('hoi4.gamePath')) {
            window.showInformationMessage('HOI4 Game Path changed. Reload window to re-index vanilla files.', 'Reload').then(selection => {
                if (selection === 'Reload') {
                    commands.executeCommand('workbench.action.reloadWindow')
                }
            })
        }
        if (e.affectsConfiguration('hoi4.modPaths')) {
            window.showInformationMessage('HOI4 dependency mod paths changed. Reload window to re-index.', 'Reload').then(selection => {
                if (selection === 'Reload') {
                    commands.executeCommand('workbench.action.reloadWindow')
                }
            })
        }
        if (e.affectsConfiguration('hoi4.modRegistryPath')) {
            // The mod registry path is consumed at server initialize time and
            // feeds dependency-mod resolution during the scan — a live
            // didChangeConfiguration notification can't re-run it, so prompt
            // for a reload like gamePath/modPaths do.
            window.showInformationMessage('HOI4 mod registry path changed. Reload window to re-index dependency mods.', 'Reload').then(selection => {
                if (selection === 'Reload') {
                    commands.executeCommand('workbench.action.reloadWindow')
                }
            })
        }
        if (e.affectsConfiguration('hoi4.validator.ignoreLocalization')) {
            const newValue = workspace.getConfiguration('hoi4.validator').get('ignoreLocalization')
            client.sendNotification('workspace/didChangeConfiguration', {
                settings: {
                    hoi4: {
                        validator: {
                            ignoreLocalization: newValue
                        }
                    }
                }
            })
        }
        if (e.affectsConfiguration('hoi4.validator.ignoreFiles')) {
            const newValue = workspace.getConfiguration('hoi4.validator').get('ignoreFiles')
            client.sendNotification('workspace/didChangeConfiguration', {
                settings: {
                    hoi4: {
                        validator: {
                            ignoreFiles: newValue
                        }
                    }
                }
            })
        }
        if (e.affectsConfiguration('hoi4.validator.workspaceScan.enabled')) {
            const newValue = workspace.getConfiguration('hoi4.validator.workspaceScan').get('enabled')
            client.sendNotification('workspace/didChangeConfiguration', {
                settings: {
                    hoi4: {
                        validator: {
                            workspaceScan: {
                                enabled: newValue
                            }
                        }
                    }
                }
            })
        }
        if (e.affectsConfiguration('hoi4.validator.scopeValidationEnabled')) {
            const newValue = workspace.getConfiguration('hoi4.validator').get('scopeValidationEnabled')
            client.sendNotification('workspace/didChangeConfiguration', {
                settings: {
                    hoi4: {
                        validator: {
                            scopeValidationEnabled: newValue
                        }
                    }
                }
            })
        }
        if (e.affectsConfiguration('hoi4.styling.enabled')) {
            const newValue = workspace.getConfiguration('hoi4.styling').get('enabled')
            client.sendNotification('workspace/didChangeConfiguration', {
                settings: {
                    hoi4: {
                        styling: {
                            enabled: newValue
                        }
                    }
                }
            })
        }
        if (e.affectsConfiguration('hoi4.styling.cosmeticLocalizationIndentation')) {
            const newValue = workspace.getConfiguration('hoi4.styling').get('cosmeticLocalizationIndentation')
            client.sendNotification('workspace/didChangeConfiguration', {
                settings: {
                    hoi4: {
                        styling: {
                            cosmeticLocalizationIndentation: newValue
                        }
                    }
                }
            })
        }
        if (e.affectsConfiguration('hoi4.logLevel')) {
            // Sits directly under `hoi4`, not under `hoi4.validator` like the
            // arms above — that's the shape the server's handler reads.
            const newValue = workspace.getConfiguration('hoi4').get('logLevel')
            client.sendNotification('workspace/didChangeConfiguration', {
                settings: {
                    hoi4: {
                        logLevel: newValue
                    }
                }
            })
        }
    }))
}

async function promptForTheme(): Promise<void> {
    const hoi4Config = workspace.getConfiguration('hoi4')
    const dismissed = hoi4Config.get<boolean>('themePromptDismissed')
    if (dismissed) return

    const workbenchConfig = workspace.getConfiguration('workbench')
    const currentTheme = workbenchConfig.get<string>('colorTheme')
    if (currentTheme === 'Hearts of Modding Dark' || currentTheme === 'Hearts of Modding Light') return

    const choice = await window.showInformationMessage(
        'This workspace supports Hearts of Modding themes! Would you like to use one? (Your global theme stays unchanged.)',
        'Hearts of Modding Dark', 'Hearts of Modding Light', 'Not Now'
    )

    if (choice === 'Hearts of Modding Dark') {
        await workbenchConfig.update('colorTheme', 'Hearts of Modding Dark', ConfigurationTarget.Workspace)
        window.showInformationMessage('✓ HoM Dark theme applied to this workspace!')
    } else if (choice === 'Hearts of Modding Light') {
        await workbenchConfig.update('colorTheme', 'Hearts of Modding Light', ConfigurationTarget.Workspace)
        window.showInformationMessage('✓ HoM Light theme applied to this workspace!')
    } else if (choice === 'Not Now') {
        await hoi4Config.update('themePromptDismissed', true, ConfigurationTarget.Workspace)
    }
}

/// Serialized entry point. Every caller outside an existing critical section
/// must use this rather than `startServerInner`.
async function startServer(context: ExtensionContext, statusBarItem: StatusBarItem) {
    return serialize(() => startServerInner(context, statusBarItem))
}

/// The actual start sequence. MUST NOT be called except while holding the
/// lifecycle lock — calling `startServer` from in here would await a chain
/// containing this function and deadlock.
async function startServerInner(context: ExtensionContext, statusBarItem: StatusBarItem) {
    if (client && client.isRunning()) {
        return
    }

    // Reveal the HoM Log panel. `executeCommand` returns a Thenable and rejects
    // asynchronously, so the try/catch that used to sit here caught nothing —
    // the view container being unregistered during early activation produced an
    // unhandled rejection regardless. Attach a rejection handler instead, and
    // don't await: focusing a view must not delay server start.
    void commands.executeCommand('workbench.view.extension.hoi4-log').then(
        undefined,
        () => { /* view container not ready — messages still go to outputChannel */ }
    )
    logPanelProvider.append('INFO', 'Hearts of Modding extension is now starting...')
    outputChannel.appendLine('Hearts of Modding extension is now starting...')

    // Resolve the hom-lsp binary for this platform/arch: bundled in the VSIX →
    // downloaded from the matching release → local build (dev fallbacks).
    const asset = homLspAssetName(process.platform, process.arch)
    let serverModule = context.asAbsolutePath(
        path.join('server-bin', asset)
    )

    if (!fs.existsSync(serverModule)) {
        logInfo(`Server binary not bundled for this platform (${asset}); checking release...`)
        const fetched = await downloadHomLspBinary(context, asset)
        if (fetched) {
            serverModule = fetched
        }
    }

    if (!fs.existsSync(serverModule)) {
        logInfo('Server binary not found (bundled/downloaded), falling back to local build...')
        // Fallback for development if not packaged
        const localSuffix = process.platform === 'win32' ? '.exe' : ''
        serverModule = context.asAbsolutePath(
            path.join('..', 'server', 'target', 'release', `hom-lsp${localSuffix}`)
        )
    }

    if (!fs.existsSync(serverModule)) {
        logInfo('Release binary not found, falling back to debug build...')
        const localSuffix = process.platform === 'win32' ? '.exe' : ''
        serverModule = context.asAbsolutePath(
            path.join('..', 'server', 'target', 'debug', `hom-lsp${localSuffix}`)
        )
    }

    if (!fs.existsSync(serverModule)) {
        logPanelProvider.append('ERROR', 'CRITICAL: No server binary found! Language features will not be available.')
        outputChannel.appendLine('CRITICAL: No server binary found! Language features will not be available.')
    } else {
        logInfo(`Using server binary at: ${serverModule}`)
    }

    // If the extension is launched in debug mode then the debug server options are used
    // Otherwise the run options are used
    const serverOptions: ServerOptions = {
        run: { command: serverModule, transport: TransportKind.stdio },
        debug: { command: serverModule, transport: TransportKind.stdio }
    }

    // Options to control the language client
    const clientOptions: LanguageClientOptions = {
        // Register the server for HOI4 and HOI4 Localisation documents
        documentSelector: [
            { scheme: 'file', language: 'hoi4' },
            { scheme: 'file', language: 'hoi4-localisation' },
            { scheme: 'file', language: 'hoi4-csv' }
        ],
        synchronize: {
            // Notify the server about .txt/.csv changes in the workspace.
            // These watchers are created once in activate() and reused across
            // clients — the language client hooks fresh listeners onto them on
            // every start() and disposes only those listeners on stop().
            fileEvents: fileEventWatchers ?? []
        },
        outputChannel: outputChannel,
        initializationOptions: {
            gamePath: workspace.getConfiguration('hoi4').get('gamePath'),
            dependencyModPaths: workspace.getConfiguration('hoi4').get('modPaths'),
            modRegistryPath: workspace.getConfiguration('hoi4').get('modRegistryPath'),
            ignoreLocalization: workspace.getConfiguration('hoi4.validator').get('ignoreLocalization'),
            ignoreFiles: workspace.getConfiguration('hoi4.validator').get('ignoreFiles'),
            workspaceScanEnabled: workspace.getConfiguration('hoi4.validator.workspaceScan').get('enabled'),
            scopeValidationEnabled: workspace.getConfiguration('hoi4.validator').get('scopeValidationEnabled'),
            stylingEnabled: workspace.getConfiguration('hoi4.styling').get('enabled'),
            cosmeticLocIndent: workspace.getConfiguration('hoi4.styling').get('cosmeticLocalizationIndentation'),
            logLevel: workspace.getConfiguration('hoi4').get('logLevel')
        }
    }

    // Create the language client and start the client.
    client = new LanguageClient(
        'heartsOfModding',
        'Hearts of Modding Language Server',
        serverOptions,
        clientOptions
    )

    // Start the client. This will also launch the server
    await client.start()

    // ── Intercept server log messages for the HoM Log panel ──
    // Captures window/logMessage notifications from the server and
    // maps the LSP MessageType to log panel severity levels.
    context.subscriptions.push(client.onNotification('window/logMessage', (params: { type?: number, message?: string }) => {
        if (!params.message) return

        // Map LSP MessageType to our log levels:
        //   1 = Error, 2 = Warning, 3 = Info, 4 = Log
        const typeMap: Record<number, string> = { 1: 'ERROR', 2: 'WARN', 3: 'INFO', 4: 'INFO' }
        const typeLevel = params.type !== undefined ? typeMap[params.type] : undefined

        // Also check for legacy [LEVEL] text prefix (older server builds)
        const levelMatch = params.message.match(/^\[(ERROR|WARN|INFO|DEBUG|TRACE)\]\s*/)
        const level = typeLevel || (levelMatch ? levelMatch[1] : 'INFO')
        const body = levelMatch ? params.message.slice(levelMatch[0].length) : params.message

        logPanelProvider.append(level, body)
    }))

    // ── Scanned color codes pushed by the LSP after each scan ──
    // The old startup one-shot `hoi4/getColorCodes` request raced the ~12s
    // workspace scan and almost always got an empty map, leaving the
    // decorator on wiki defaults forever. The server now pushes the map
    // after the scan completes.
    context.subscriptions.push(client.onNotification('hoi4/colorCodes', (colorMap: Record<string, string>) => {
        if (colorMap && Object.keys(colorMap).length > 0) {
            locColorDecorator.updateColors(colorMap)
            outputChannel.appendLine(`HoM color decorator: loaded ${Object.keys(colorMap).length} color codes from LSP`)
        }
    }))

    const updateMemoryUsage = async () => {
        const enabled = workspace.getConfiguration('hoi4.showMemoryUsage').get('enabled')
        if (enabled) {
            try {
                const usage: { memoryUsedBytes?: number, pendingTasks?: number } | undefined = await client.sendRequest('workspace/executeCommand', {
                    command: 'hoi4/getMemoryUsage',
                    arguments: []
                }) as { memoryUsedBytes?: number, pendingTasks?: number } | undefined
                if (usage && usage.memoryUsedBytes) {
                    const icon = usage.pendingTasks && usage.pendingTasks > 0 ? '$(sync~spin)' : '$(pulse)'
                    statusBarItem.text = `${icon} HoM RAM: ${formatBytes(usage.memoryUsedBytes)}`
                    statusBarItem.tooltip = usage.pendingTasks && usage.pendingTasks > 0
                        ? 'Hearts of Modding Server Memory Usage (processing...)'
                        : 'Hearts of Modding Server Memory Usage'
                    statusBarItem.show()
                } else {
                    statusBarItem.hide()
                }
            } catch {
                statusBarItem.hide()
            }
        } else {
            statusBarItem.hide()
        }
    }

    // Initial update and interval
    updateMemoryUsage()
    if (memoryInterval) {
        clearInterval(memoryInterval)
    }
    memoryInterval = setInterval(updateMemoryUsage, 2000)

}

export function deactivate(): Thenable<void> | undefined {
    if (memoryInterval) {
        clearInterval(memoryInterval)
    }
    if (!client) {
        return undefined
    }
    return client.stop()
}
