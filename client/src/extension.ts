import * as path from 'path';
import * as fs from 'fs';
import { workspace, ExtensionContext, window, OutputChannel, commands } from 'vscode';

import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;
let outputChannel: OutputChannel;

export function activate(context: ExtensionContext) {
    outputChannel = window.createOutputChannel('Hearts of Modding');
    outputChannel.show(true); // Force the output channel to be visible and focused
    outputChannel.appendLine('Hearts of Modding extension is now active!');
    console.log('Hearts of Modding extension: activate called');

    context.subscriptions.push(commands.registerCommand('hearts-of-modding.activate', () => {
        outputChannel.show();
        window.showInformationMessage('Hearts of Modding is already active!');
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
        // Register the server for HOI4 documents
        documentSelector: [{ scheme: 'file', language: 'hoi4' }],
        synchronize: {
            // Notify the server about file changes to '.txt files contained in the workspace
            fileEvents: workspace.createFileSystemWatcher('**/*.txt')
        },
        outputChannel: outputChannel,
        initializationOptions: {
            gamePath: workspace.getConfiguration('hoi4').get('gamePath'),
            ignoreLocalization: workspace.getConfiguration('hoi4.validator').get('ignoreLocalization'),
            stylingEnabled: workspace.getConfiguration('hoi4.styling').get('enabled')
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
    client.start();

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
    }));
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}