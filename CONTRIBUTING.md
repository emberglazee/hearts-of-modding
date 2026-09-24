# ⌨️ Contributing to Hearts of Modding

Anything from simple usage and review, to bug reporting and testing, to direct code contribution, is considered contribution, and is welcome.

- No general issue reporting guidelines yet, but try to describe and provide as much information as possible.

- Prefer branch prefixing (`feat/`, `fix/`, `chore/`) for own convenience. Not required.

- Same for commit message and PR title prefixes (`feat:`, `fix:`, `chore:`, `docs:`, `refactor:`, `test:`, `perf:`).

- Linting (`rustfmt` + `clippy`, `eslint`) ***is*** required by CI.
  - `npm run check-all-package` handles everything from linting and checks/tests to building and packaging.

- Prefer one focused change per PR (batching multiple into one is acceptable if changes are adjacent), CI must be green before merging.

- Commit amending is acceptable, but *please* push only with `--force-with-lease` in those cases and try to warn beforehand.

### 🖥️ Also for LLM agents:

- Fact check against verified HOI4 modding sources like https://hoi4.paradoxwikis.com/Modding, https://hoi4doc.dev, local vanilla HOI4 files (if present), HOI4 modding focused skills, etc.

- Do extensive regression testing.

- Write short and focused changelog entries and commit messages (only if decided to write them; can be left up to the user).

## 🛜 Get started locally

System dependencies (or atleast the rough idea of what to install):

- Rust (`rustup`, stable) + `rustfmt` + `clippy`, >= v1.95.0 (forced by `sysinfo`, would be >= v1.85.0 otherwise)
- Node.js + `npm`, >= v20.19.0
- C compiler + `make` + POSIX `sh`, for `jemalloc`
- `git`, to clone the repository (`jemalloc` fork is fetched by `cargo`)

### Debian / Ubuntu

```bash
sudo apt install -y build-essential curl git # gcc, make, binutils, libc headers

# MUST be rustup; distro rustc (1.75/1.85) is below the 1.95 floor as of writing this
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y # default profile includes rustfmt + clippy
source "$HOME/.cargo/env"

# Node ≥ 20.19 — Ubuntu's nodejs is 18.x, too old for eslint 10 / vsce 3
curl -fsSL https://deb.nodesource.com/setup_24.x | sudo -E bash - && sudo apt install -y nodejs
# or via nvm:
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.8/install.sh | bash
nvm install 24
```

### Arch

```bash
sudo pacman -S --needed base-devel git nodejs npm # base-devel = gcc/make/binutils/pkgconf
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

### Windows (via `winget`)

> by far the most messy, rough idea from CI equivalent (preships VS 2022 + MSYS2 + Git-for-Windows), not tested/verified yet;

> ill try to sort it out once i get around to it on my windows 11 dualboot

```pwsh
winget install Rustlang.Rustup # default host: x86_64-pc-windows-msvc
# rustc's `-pc-windows-msvc` requires `cl.exe`/`link.exe` + Windows SDK (also used by `cc` to compile jemalloc)
winget install Microsoft.VisualStudio.2022.BuildTools --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
winget install MSYS2.MSYS2 # `jemalloc-sys` hardcodes the tool names, especially `"mingw32-make"`
winget install OpenJS.NodeJS.LTS # Node 24 LTS, ≥ 20.19
```

In MSYS2:

```bash
pacman -S --needed make mingw-w64-x86_64-make # provides sh + mingw32-make
```

> more chaos:

- Run `cargo` from MSYS2's MINGW64 shell or add `C:\msys64\usr\bin` and `C:\msys64\mingw64\bin` to PATH, and keep MSVC dirs *ahead* of them (MSYS2's `usr\bin` also carries a unix `link.exe`)
- `server/.cargo/config.toml` pins `+crt-static` for `windows-msvc`, so `hom-lsp.exe` carries no VCRUNTIME (CI asserts this)

### macOS

```bash
xcode-select --install # clang, make, sh; nothing else needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
brew install node # or get nvm and `nvm install 24`
```

Apple silicon builds `aarch64` natively, while Intel builds `x86_64` (CI cross-builds `macos-amd64` from the arm64 runner)

## 📦 Build and package

(Arch) Budget ~2 GB for `server/target/release` and ~200 MB for `node_modules`

```bash
git clone https://github.com/emberglazee/Hearts-of-Modding
cd Hearts-of-Modding/client

### all linting, checks, compilation and packaging:
# eslint . --fix -> cargo fmt -> cargo check ->
# cargo clippy --all-targets -- -D warnings ->
# cargo test -> cargo build --release ->
# node stage-binary.mjs -> npm install ->
# tsc -> node esbuild.js -> vsce package
npm run check-all-package
```

### ❗ Local VSIX caveats

- Only contains your host's binary.
- No README/CHANGELOG inside.
