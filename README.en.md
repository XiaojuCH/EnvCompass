# EnvCompass

## Why won't your project run again?

Check Python, Node.js, PATH, Git, and project runtime requirements for conflicts in one click.

**Find the issue → understand why → send a sanitized report to an AI or another developer.**

[English](./README.en.md) | [简体中文](./README.md)

EnvCompass is a Windows-first, local-first, read-only development environment diagnosis tool. It does not install, modify, or repair anything. It only tells you what does not match, why, and what evidence supports that conclusion.

## What it can do today

- Scan this PC: check the actual resolution order and versions of PATH, Python, Node.js, and Git.
- Diagnose a project: compare project requirements with the runtimes actually used on this PC; this is the primary GUI action.
- Detect Python projects from `.python-version`, `pyproject.toml`, `requirements*.txt`, `environment.yml/.yaml`, `setup.py`, `setup.cfg`, `Pipfile`, `Pipfile.lock`, `poetry.lock`, and `uv.lock`.
- Detect metadata-light projects: count `.py` files within strict depth/file bounds without reading or executing source; when no version is declared, report that compatibility cannot be determined reliably.
- Detect Node.js projects from `.nvmrc`, `.node-version`, `package.json`, `engines.node`, `packageManager`, and common lockfiles.
- Explain problems: compare project declarations with the runtime currently in use and show readable findings.
- Copy for AI: generate a concise, high-signal Markdown report with conclusions, relevant runtimes, and direct evidence.
- Technical report: optionally include the full tool inventory and PATH under the same sanitizer as the concise report.

EnvCompass does not infer dependencies from source code today. Detecting `.py` files is not import analysis, and no inference is presented as a declared requirement.

## How to run

An ordinary Windows user does not need PowerShell, Python, Node, or Rust installed to run the packaged app.

### Development mode

You need Node.js, Rust (MSVC toolchain), and the Tauri prerequisites:

```powershell
npm install
npm run tauri:dev
```

### Release build

```powershell
npm run tauri:build
```

Windows release artifacts are placed in `src-tauri\target\release`.

## Product boundaries

v0.1 is a read-only diagnosis tool, not an environment manager. It never modifies PATH, the Registry, environment variables, or project files, and it never executes scripts found in a scanned project.

Currently out of scope: CUDA, Java, Android, Docker, WSL, Visual Studio/MSVC diagnosis, automatic repair, built-in AI, cloud sync, accounts, and telemetry.

## Privacy

Every Markdown report produced by “Copy for AI” and “Save report” goes through the same sanitizer:

- The project root is replaced with `%PROJECT_ROOT%`
- User and common system directories use environment placeholders
- Other local/network absolute paths do not retain private directory names
- `.env` values are never read or exported
- Environment variable values are not exported
- Common API key, token, password, proxy credential, and URL patterns are filtered again

The local GUI shows real paths for the user doing the diagnosis; copy/save is the sanitized sharing boundary. The sanitizer reduces common disclosure risks but cannot guarantee detection of every unknown credential format, so a quick review before sharing is still recommended.

Do not commit real scan reports to the repository.

## Tests

```powershell
cd src-tauri
cargo test
cargo clippy --all-targets --all-features -- -D warnings

cd ..
npm test
npm run build
```

## License

TBD.
