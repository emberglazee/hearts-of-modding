// Stage the natively-built hom-lsp binary (+ server assets) into client/server-bin
// using the unified `hom-lsp-<os>-<arch>[.exe]` naming. Used by `npm run package`
// for local VSIX builds; CI does its own per-target staging in the workflow.
import { existsSync, mkdirSync, copyFileSync, chmodSync, readdirSync, statSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const clientRoot = dirname(dirname(fileURLToPath(import.meta.url)));
const serverRoot = join(clientRoot, '..', 'server');
const serverBin = join(clientRoot, 'server-bin');

const osName = process.platform === 'win32' ? 'win' : process.platform === 'darwin' ? 'macos' : 'linux';
const archName = process.arch === 'x64' ? 'amd64' : process.arch === 'arm64' ? 'arm64' : process.arch;
const exe = process.platform === 'win32' ? '.exe' : '';

const src = join(serverRoot, 'target', 'release', `hom-lsp${exe}`);
const dst = join(serverBin, `hom-lsp-${osName}-${archName}${exe}`);

if (!existsSync(src)) {
    console.error(`[stage-binary] missing ${src} — run \`cargo build --release\` in server/ first`);
    process.exit(1);
}

mkdirSync(serverBin, { recursive: true });
copyFileSync(src, dst);
if (process.platform !== 'win32') chmodSync(dst, 0o755);
console.log(`[stage-binary] staged ${dst}`);

// Copy server/assets -> server-bin/assets (replace any stale copy).
const assetsSrc = join(serverRoot, 'assets');
const assetsDst = join(serverBin, 'assets');
if (existsSync(assetsSrc)) {
    mkdirSync(assetsDst, { recursive: true });
    for (const entry of readdirSync(assetsSrc)) {
        const from = join(assetsSrc, entry);
        const to = join(assetsDst, entry);
        if (statSync(from).isDirectory()) {
            mkdirSync(to, { recursive: true });
            for (const inner of readdirSync(from)) {
                copyFileSync(join(from, inner), join(to, inner));
            }
        } else {
            copyFileSync(from, to);
        }
    }
    console.log(`[stage-binary] staged assets`);
}
