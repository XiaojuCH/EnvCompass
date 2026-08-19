# EnvCompass

## Why won't your project run again?

Check Python, Node.js, PATH, Git, and project runtime requirements for conflicts in one click.

**Find the issue → understand why → send a sanitized report to an AI or another developer.**

[English](./README.en.md) | [简体中文](./README.md)

EnvCompass is a Windows-first, local-first, read-only development environment diagnosis tool. It does not install, modify, or repair anything. It only tells you what does not match, why, and what evidence supports that conclusion.

## What it can do today

- Scan this PC: check the actual resolution order and versions of PATH, Python, Node.js, and Git.
- Select a project folder: read only the minimal runtime metadata from `.python-version`, `pyproject.toml`, `.nvmrc`, `.node-version`, and `package.json`. It does not recurse into source code and does not execute project scripts.
- Explain problems: compare project declarations with the runtime currently in use and show readable findings.
- Copy for AI: generate a standalone Markdown diagnosis report with user home paths and common secret/token/credential patterns redacted by default.
- Save report: write the sanitized Markdown report locally.

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

- User home directories are replaced with `%USERPROFILE%`
- `.env` values are never read or exported
- Environment variable values are not exported
- Common API key, token, password, and proxy credential patterns are filtered again

Do not commit real scan reports to the repository.

## Tests

```powershell
cd src-tauri
cargo test
```

## License

TBD.

