import * as path from 'path';
import * as fs from 'fs';
import { workspace, ExtensionContext, window, OutputChannel, commands, StatusBarAlignment, ConfigurationTarget } from 'vscode';

import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;
let outputChannel: OutputChannel;
let memoryInterval: NodeJS.Timeout | undefined;

function formatBytes(bytes: number): string {
    if (!Number.isFinite(bytes) || bytes <= 0) {
        return "0 Bytes";
    }
    const k = 1024;
    const sizes = ["Bytes", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    const size = (bytes / Math.pow(k, i)).toFixed(2);
    return `${size} ${sizes[i]}`;
}

export async function activate(context: ExtensionContext) {
    outputChannel = window.createOutputChannel('Hearts of Modding');
    console.log('Hearts of Modding extension: activate called');

    const statusBarItem = window.createStatusBarItem(StatusBarAlignment.Right, 100);
    context.subscriptions.push(statusBarItem);

    context.subscriptions.push(commands.registerCommand('hearts-of-modding.showMemoryUsage', async () => {
        const config = workspace.getConfiguration('hoi4.showMemoryUsage');
        const currentState = config.get('enabled');
        await config.update('enabled', !currentState, true);
        window.showInformationMessage(`Memory Usage Display: ${!currentState ? 'Enabled' : 'Disabled'}`);
    }));

    context.subscriptions.push(commands.registerCommand('hearts-of-modding.activate', async () => {
        if (client && client.isRunning()) {
            outputChannel.show();
            window.showInformationMessage('Hearts of Modding is already active!');
            return;
        }
        await workspace.getConfiguration('hoi4').update('enabled', true, ConfigurationTarget.Workspace);
        await startServer(context, statusBarItem);
    }));

    context.subscriptions.push(commands.registerCommand('hearts-of-modding.setGamePath', async () => {
        const options = {
            canSelectMany: false,
            openLabel: 'Select HOI4 Installation Folder',
            canSelectFiles: false,
            canSelectFolders: true
        };

        const fileUri = await window.showOpenDialog(options);
        if (fileUri && fileUri[0]) {
            const folderPath = fileUri[0].fsPath;
            await workspace.getConfiguration('hoi4').update('gamePath', folderPath, true);
            window.showInformationMessage(`HOI4 Game Path set to: ${folderPath}`);
        }
    }));

    context.subscriptions.push(commands.registerCommand('hearts-of-modding.toggleStyling', async () => {
        const config = workspace.getConfiguration('hoi4.styling');
        const currentState = config.get('enabled');
        await config.update('enabled', !currentState, true);
        window.showInformationMessage(`HOI4 Styling Checks: ${!currentState ? 'Enabled' : 'Disabled'}`);
    }));

    const config = workspace.getConfiguration('hoi4');
    let enabled = config.get<boolean | null>('enabled');

    if (enabled === false) {
        outputChannel.appendLine('Hearts of Modding is disabled for this workspace.');
        return;
    }

    if (enabled === undefined || enabled === null) {
        const descriptorFiles = await workspace.findFiles('descriptor.mod', null, 1);
        if (descriptorFiles.length > 0) {
            const result = await window.showInformationMessage(
                'This workspace looks like a Hearts of Iron IV mod. Enable Hearts of Modding features?',
                'Yes', 'No'
            );
            if (result === 'Yes') {
                await config.update('enabled', true, ConfigurationTarget.Workspace);
                enabled = true;
            } else if (result === 'No') {
                await config.update('enabled', false, ConfigurationTarget.Workspace);
                outputChannel.appendLine('Hearts of Modding was declined for this workspace.');
                return;
            } else {
                // User dismissed the message
                return;
            }
        } else {
            // Not a mod, and not explicitly enabled
            return;
        }
    }

    if (enabled === true) {
        await startServer(context, statusBarItem);
    }
}

async function startServer(context: ExtensionContext, statusBarItem: any) {
    if (client && client.isRunning()) {
        return;
    }

    outputChannel.show(true); 
    outputChannel.appendLine('Hearts of Modding extension is now starting...');

    // The server is implemented in Rust
    let osSuffix = process.platform === 'win32' ? '-win.exe' : '-linux';
    let serverModule = context.asAbsolutePath(
        path.join('server-bin', `server${osSuffix}`)
    );
    
    if (!fs.existsSync(serverModule)) {
        outputChannel.appendLine(`Server binary not found in server-bin (${serverModule}), falling back to local build...`);
        // Fallback for development if not packaged
        let localSuffix = process.platform === 'win32' ? '.exe' : '';
        serverModule = context.asAbsolutePath(
            path.join('..', 'server', 'target', 'release', `server${localSuffix}`)
        );
    }
    
    if (!fs.existsSync(serverModule)) {
        outputChannel.appendLine('Release binary not found, falling back to debug build...');
        let localSuffix = process.platform === 'win32' ? '.exe' : '';
        serverModule = context.asAbsolutePath(
            path.join('..', 'server', 'target', 'debug', `server${localSuffix}`)
        );
    }

    if (!fs.existsSync(serverModule)) {
        outputChannel.appendLine('CRITICAL: No server binary found! Language features will not be available.');
    } else {
        outputChannel.appendLine(`Using server binary at: ${serverModule}`);
    }
    
    // If the extension is launched in debug mode then the debug server options are used
    // Otherwise the run options are used
    let serverOptions: ServerOptions = {
        run: { command: serverModule, transport: TransportKind.stdio },
        debug: { command: serverModule, transport: TransportKind.stdio }
    };

    // Options to control the language client
    let clientOptions: LanguageClientOptions = {
        // Register the server for HOI4 and HOI4 Localisation documents
        documentSelector: [
            { scheme: 'file', language: 'hoi4' },
            { scheme: 'file', language: 'hoi4-localisation' }
        ],
        synchronize: {
            // Notify the server about file changes to '.txt files contained in the workspace
            fileEvents: workspace.createFileSystemWatcher('**/*.txt')
        },
        outputChannel: outputChannel,
        initializationOptions: {
            gamePath: workspace.getConfiguration('hoi4').get('gamePath'),
            ignoreLocalization: workspace.getConfiguration('hoi4.validator').get('ignoreLocalization'),
            stylingEnabled: workspace.getConfiguration('hoi4.styling').get('enabled'),
            cosmeticLocIndent: workspace.getConfiguration('hoi4.styling').get('cosmeticLocalizationIndentation')
        }
    };

    // Create the language client and start the client.
    client = new LanguageClient(
        'heartsOfModding',
        'Hearts of Modding Language Server',
        serverOptions,
        clientOptions
    );

    // Start the client. This will also launch the server
    await client.start();
    
    const updateMemoryUsage = async () => {
        const enabled = workspace.getConfiguration('hoi4.showMemoryUsage').get('enabled');
        if (enabled) {
            try {
                const usage: any = await client.sendRequest('workspace/executeCommand', {
                    command: 'hoi4/getMemoryUsage',
                    arguments: []
                });
                if (usage && usage.memoryUsedBytes) {
                    statusBarItem.text = `$(pulse) HoM RAM: ${formatBytes(usage.memoryUsedBytes)}`;
                    statusBarItem.tooltip = "Hearts of Modding Server Memory Usage";
                    statusBarItem.show();
                } else {
                    statusBarItem.hide();
                }
            } catch (e) {
                statusBarItem.hide();
            }
        } else {
            statusBarItem.hide();
        }
    };

    // Initial update and interval
    updateMemoryUsage();
    if (memoryInterval) {
        clearInterval(memoryInterval);
    }
    memoryInterval = setInterval(updateMemoryUsage, 2000);

    // Listen for configuration changes
    context.subscriptions.push(workspace.onDidChangeConfiguration(e => {
        if (e.affectsConfiguration('hoi4.gamePath')) {
            window.showInformationMessage('HOI4 Game Path changed. Reload window to re-index vanilla files.', 'Reload').then(selection => {
                if (selection === 'Reload') {
                    commands.executeCommand('workbench.action.reloadWindow');
                }
            });
        }
        if (e.affectsConfiguration('hoi4.validator.ignoreLocalization')) {
            const newValue = workspace.getConfiguration('hoi4.validator').get('ignoreLocalization');
            client.sendNotification('workspace/didChangeConfiguration', {
                settings: {
                    hoi4: {
                        validator: {
                            ignoreLocalization: newValue
                        }
                    }
                }
            });
        }
        if (e.affectsConfiguration('hoi4.styling.enabled')) {
            const newValue = workspace.getConfiguration('hoi4.styling').get('enabled');
            client.sendNotification('workspace/didChangeConfiguration', {
                settings: {
                    hoi4: {
                        styling: {
                            enabled: newValue
                        }
                    }
                }
            });
        }
        if (e.affectsConfiguration('hoi4.styling.cosmeticLocalizationIndentation')) {
            const newValue = workspace.getConfiguration('hoi4.styling').get('cosmeticLocalizationIndentation');
            client.sendNotification('workspace/didChangeConfiguration', {
                settings: {
                    hoi4: {
                        styling: {
                            cosmeticLocalizationIndentation: newValue
                        }
                    }
                }
            });
        }
    }));
}

export function deactivate(): Thenable<void> | undefined {
    if (memoryInterval) {
        clearInterval(memoryInterval);
    }
    if (!client) {
        return undefined;
    }
    return client.stop();
}