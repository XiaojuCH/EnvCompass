# EnvCompass

## Why won't your project run again?

EnvCompass is a Windows development environment doctor. It checks project declarations against the Python, Node.js, PATH, and Git actually used on the machine, identifies evidence-backed version or resolution conflicts, and generates a sanitized report you can send to an AI or another developer.

**Inspect → Diagnose → Explain → Share**

[Download the Windows Preview](https://github.com/XiaojuCH/EnvCompass/releases) · [简体中文](./README.md)

The current Preview targets Windows x64. Scans stay on the device: EnvCompass does not upload data, modify the system, execute project scripts, require an account, or collect telemetry.

![EnvCompass home screen](./docs/assets/envcompass-home.png)

## Download and install

Ordinary users do not need to clone the repository or install Python, Node.js, or Rust first.

1. Open [GitHub Releases](https://github.com/XiaojuCH/EnvCompass/releases).
2. **For most users:** download `EnvCompass-0.1.0-preview.1-windows-x64-setup.exe`.
3. **No installation:** download `EnvCompass-0.1.0-preview.1-windows-x64-portable.zip`, extract it once, and run `EnvCompass.exe`.
4. The MSI is an advanced alternative. Use `SHA256SUMS.txt` to verify downloaded files.

> Preview builds are currently unsigned, so Windows SmartScreen may show a warning. Confirm that the file came from this repository's GitHub Release and verify its SHA-256 checksum. You do not need to disable or permanently bypass Windows security features.

The packaged application version remains `0.1.0`; the first Preview Git tag and Release are `v0.1.0-preview.1`.

## What it can do today

- **Diagnose a project:** compare declared requirements with the runtimes actually in use; this is the recommended entry point.
- **Scan this PC:** inspect the resolution and versions of PATH, Python / pip, Node.js / common package managers, and Git.
- **Explain evidence-backed issues:** report runtime mismatches, package-manager mismatches, and real command-resolution failures without guessing when evidence is incomplete.
- **Detect Python projects:** read bounded `.python-version`, `pyproject.toml`, `requirements*.txt`, `environment.yml/.yaml`, `setup.py`, `setup.cfg`, `Pipfile`, `Pipfile.lock`, `poetry.lock`, and `uv.lock` metadata.
- **Detect Node.js projects:** read `.nvmrc`, `.node-version`, `package.json`, `engines.node`, `packageManager`, and common lockfiles.
- **Detect metadata-light Python projects:** count `.py` files within strict bounds without reading or executing source, and state clearly when no runtime compatibility conclusion is possible.
- **Share a diagnosis:** produce a concise Copy for AI report or an optional technical report with the full tool inventory and PATH.
- **Chinese / English:** the GUI supports `zh-CN` and `en-US` and follows the Windows language by default.

![EnvCompass synthetic project diagnosis](./docs/assets/envcompass-project-results.png)

## Privacy and read-only boundaries

The local GUI shows real paths to the person troubleshooting. Copying or saving is a separate sharing boundary, and every report uses the same sanitizer:

- the project root becomes `%PROJECT_ROOT%`;
- user, common system, and other local/network absolute paths use placeholders;
- `.env` values, environment variable values, and source content are not read or exported;
- common API key, token, password, proxy credential, and URL patterns are filtered.

The sanitizer reduces common disclosure risks but cannot guarantee detection of every unknown credential format. Review a report before sharing it, especially before posting anything in a public issue.

EnvCompass v0.1 is a read-only diagnosis tool, not an environment manager. It does not modify PATH, the Registry, environment variables, or project files; install or uninstall runtimes; repair automatically; or execute scripts found in a scanned project.

## Current scope

The verified packaging target is Windows x64, with development and real workflow validation completed on Windows 11. Deep Conda diagnosis, CUDA, YOLO / PyTorch setup, Docker, WSL, Java, automatic repair, built-in AI, cloud sync, and telemetry are not included.

See [ROADMAP.md](./ROADMAP.md) for planned directions. Future Recipes would prefer isolated environments over changes to the system Python, PATH, or an existing project; they are not implemented today.

## Develop from source

Only contributors need Node.js, the Rust MSVC toolchain, and the Tauri prerequisites:

```powershell
npm ci
npm run tauri:dev
```

Run the full checks:

```powershell
npm test
npm run build

cd src-tauri
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
```

Build the Windows release:

```powershell
npm run tauri:build
```

## Contributing

Open an issue before substantial changes, and read [CONTRIBUTING.md](./CONTRIBUTING.md) and [SECURITY.md](./SECURITY.md). Never commit real scan data, private paths, or secrets.

EnvCompass is available under the [MIT License](./LICENSE).
