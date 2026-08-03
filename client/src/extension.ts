import * as path from 'path'
import * as fs from 'fs'
import { workspace, ExtensionContext, window, OutputChannel, commands, StatusBarAlignment, ConfigurationTarget, StatusBarItem } from 'vscode'

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

async function downloadHomLspBinary(
    context: ExtensionContext,
    asset: string
): Promise<string | null> {
    const version: string = (context.extension?.packageJSON?.version as string) ?? '0.0.0'
    const dir = path.join(context.globalStorageUri.fsPath, 'hom-lsp', version)
    const dst = path.join(dir, asset)

    if (fs.existsSync(dst)) {
        logInfo(`Using cached hom-lsp binary at: ${dst}`)
        return dst
    }

    // Pin to the release matching the installed extension version; fall back to
    // the latest release if that tag doesn't exist yet.
    const urls = [
        `https://github.com/${HOM_REPO}/releases/download/v${version}/${asset}`,
        `https://github.com/${HOM_REPO}/releases/latest/download/${asset}`
    ]

    for (const url of urls) {
        try {
            logInfo(`No bundled binary for this platform; downloading ${asset} from ${url}...`)
            const res = await fetch(url)
            if (!res.ok) {
                logWarn(`Download failed (HTTP ${res.status}) from ${url}`)
                continue
            }
            const buf = Buffer.from(await res.arrayBuffer())
            fs.mkdirSync(dir, { recursive: true })
            fs.writeFileSync(dst, new Uint8Array(buf))
            if (process.platform !== 'win32') {
                fs.chmodSync(dst, 0o755)
            }
            logInfo(`Downloaded ${asset} (${buf.length} bytes) to ${dst}`)
            return dst
        } catch (err) {
            logWarn(`Download error from ${url}: ${err}`)
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
            await startServer(context, statusBarItem)
            window.showInformationMessage('Hearts of Modding LSP started!')
        }
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

async function startServer(context: ExtensionContext, statusBarItem: StatusBarItem) {
    if (client && client.isRunning()) {
        return
    }

    // Open the HoM Log panel in the bottom panel instead of the output channel
    // Wrap in try-catch because the view container may not be registered yet
    // during early activation — an unhandled rejection here destabilizes the
    // extension host, cascading into session crashes and message queue backlogs.
    try {
        commands.executeCommand('workbench.view.extension.hoi4-log')
    } catch {
        // View container not ready — messages still go to outputChannel
    }
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
            // Notify the server about file changes to '.txt files contained in the workspace
            fileEvents: [
                workspace.createFileSystemWatcher('**/*.txt'),
                workspace.createFileSystemWatcher('**/*.csv')
            ]
        },
        outputChannel: outputChannel,
        initializationOptions: {
            gamePath: workspace.getConfiguration('hoi4').get('gamePath'),
            dependencyModPaths: workspace.getConfiguration('hoi4').get('modPaths'),
            modRegistryPath: workspace.getConfiguration('hoi4').get('modRegistryPath'),
            ignoreLocalization: workspace.getConfiguration('hoi4.validator').get('ignoreLocalization'),
            ignoreFiles: workspace.getConfiguration('hoi4.validator').get('ignoreFiles'),
            workspaceScanEnabled: workspace.getConfiguration('hoi4.validator.workspaceScan').get('enabled'),
            stylingEnabled: workspace.getConfiguration('hoi4.styling').get('enabled'),
            cosmeticLocIndent: workspace.getConfiguration('hoi4.styling').get('cosmeticLocalizationIndentation')
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
    client.onNotification('window/logMessage', (params: { type?: number, message?: string }) => {
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
    })

    // ── Command: Show the HoM Log panel ──
    context.subscriptions.push(commands.registerCommand('hearts-of-modding.showLog', () => {
        try {
            commands.executeCommand('workbench.view.extension.hoi4-log')
        } catch {
            // Silently ignore — fallback to the output channel
        }
    }))

    // ── Scanned color codes pushed by the LSP after each scan ──
    // The old startup one-shot `hoi4/getColorCodes` request raced the ~12s
    // workspace scan and almost always got an empty map, leaving the
    // decorator on wiki defaults forever. The server now pushes the map
    // after the scan completes.
    client.onNotification('hoi4/colorCodes', (colorMap: Record<string, string>) => {
        if (colorMap && Object.keys(colorMap).length > 0) {
            locColorDecorator.updateColors(colorMap)
            outputChannel.appendLine(`HoM color decorator: loaded ${Object.keys(colorMap).length} color codes from LSP`)
        }
    })

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
