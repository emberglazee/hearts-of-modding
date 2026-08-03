// Stage the natively-built hom-lsp binary into client/server-bin using the
// unified `hom-lsp-<os>-<arch>[.exe]` naming. Used by `npm run package` for
// local VSIX builds; CI does its own per-target staging in the workflow.
// NOTE: server/assets is NOT copied — hoi4_data_v2.json is embedded into the
// binary at compile time via include_str! (see server/build.rs), so a
// server-bin/assets copy would be dead weight in the VSIX.
import { existsSync, mkdirSync, copyFileSync, chmodSync, rmSync } from 'fs';
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

// Remove any stale server-bin/assets copy from older packaging runs — it is
// dead weight in the VSIX (assets are compiled into the binary, not read from
// disk at runtime).
const assetsDst = join(serverBin, 'assets');
rmSync(assetsDst, { recursive: true, force: true });
if (existsSync(join(serverRoot, 'assets'))) {
    console.log('[stage-binary] removed stale server-bin/assets (data is embedded in the binary)');
}
