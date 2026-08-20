# PROJECT_STATE

## Current product

- Tauri 2 Windows desktop utility with a React/TypeScript UI and deterministic Rust diagnosis core.
- Windows-first, local-first, read-only, GUI-first; no account, telemetry, built-in AI, elevation, or automatic repair.
- Primary loop: inspect the machine and bounded project metadata, diagnose evidence-backed mismatches, explain limitations, and copy/save a sanitized report.
- Machine checks cover PATH, Python/pip/launcher inventory, Node.js/common package managers, and Git. Direct or mapped network PATH entries are not accessed and are reported as unchecked/unknown.
- Project checks parse bounded Python/Node metadata and count bounded metadata-light Python files. Project scripts and executables are never run; metadata symlinks are refused.
- GUI findings, project read notes, concise Copy for AI, and the technical report are localized for `zh-CN` and `en-US`.
- Concise and technical reports use the same sanitizer for project/home/arbitrary-drive/UNC paths, URLs, credentials, and common token formats.

## Public remote and Preview 1

- Repository: `XiaojuCH/EnvCompass`, public, default branch `main`.
- Protected `main` requires a pull request and the `Windows quality and release build` check; no bypass is configured.
- Immutable baseline `v0.1.0-preview.1` points to main commit `81525e371b60781852e3ae7d455ab03d331350f5` and has a real pre-release.
- GitHub Release assets exist for setup EXE, flat portable ZIP, MSI, and `SHA256SUMS.txt`; downloaded asset hashes matched the published checksums.
- Preview 1 CI and tag-triggered Release workflows completed successfully.

## Preview 2 hardening

- Coherent audit Issue: https://github.com/XiaojuCH/EnvCompass/issues/1
- Branch: `codex/preview-2-hardening`
- Draft PR: https://github.com/XiaojuCH/EnvCompass/pull/2
- Implementation commit: `45817c78e927e06daf44bfc88144d5124896dd1e`
- Accepted fixes: network PATH uncertainty and blocking avoidance; bounded probe reader joins; Corepack network disabled in child probes; stderr version parsing; full finding/report localization; safer bounded metadata parsing; Conda/Poetry requirement correctness; stronger report sanitization; narrow-window overflow; real synthetic mismatch fixture/screenshot; clearer MSI guidance.
- CI/Release actions use their current Node-runtime major versions (`actions/checkout@v7`, `actions/setup-node@v7`) after GitHub reported the old Node 20 action runtime as deprecated.
- No Preview 2 tag or Release exists; PR #2 is not merged.

## Validation (2026-08-20)

- `cargo fmt --all -- --check`: passed.
- `cargo test --all-targets`: 33 passed, 0 failed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `npm test`: 2 passed, 0 failed.
- `npm run build`: passed.
- `npm audit`: 0 vulnerabilities.
- `npm run tauri build`: release EXE, NSIS setup, and MSI produced.
- PR implementation CI run `32345611063`: all required steps, including Windows release bundles, passed.
- Real release-build GUI: healthy machine and synthetic runtime mismatch completed; Chinese/English UI and both report formats were checked; narrow 822 px, normal 1042 px, and wide 2560 px windows were inspected.
- Preview 1 portable: downloaded from GitHub, hash checked, flat extraction, launch, and scan passed.
- Preview 1 setup EXE: downloaded from GitHub, per-user install, Start Menu entry, first launch/scan, and uninstall passed.
- Preview 1 MSI: non-elevated install returned error 1925 and rolled back cleanly; it is now documented as an administrator-only alternative. Elevation was not attempted.

## Known issues / boundaries

- Preview binaries remain unsigned; SmartScreen/reputation friction is disclosed and users are directed to the official Release plus SHA-256 verification.
- MSI remains an advanced administrator-only alternative; setup EXE is the ordinary-user path.
- Network PATH locations are deliberately left unchecked. A network entry before a local command candidate makes command availability unknown rather than guessed.
- Runtime matching covers commands visible to the desktop process. IDE/terminal activated environments and project `.venv` interpreters can differ; EnvCompass does not execute them.
- Local GUI paths remain real by design; sharing is the sanitized boundary. Sanitization covers tested common formats but cannot guarantee every future secret syntax.
- Dependabot alert `GHSA-wrw7-89jp-8q8g` for `glib 0.18.5` remains open. `cargo tree` confirms `glib` is absent from the Windows MSVC dependency tree and present only in Tauri's Linux GTK stack; revisit if Linux becomes supported.
- Static Python import inference and deeper Conda behavior remain deliberately out of scope.

## Next priorities

1. Review Draft PR #2 and its latest required CI result; do not merge without owner approval.
2. If accepted and merged, prepare distinct `v0.1.0-preview.2` release notes/tag/assets; never mutate Preview 1.
3. Continue public dogfood before expanding the product scope.

## Git

- Base `main`: `81525e371b60781852e3ae7d455ab03d331350f5`.
- Active branch: `codex/preview-2-hardening`.
- Implementation commit: `45817c78e927e06daf44bfc88144d5124896dd1e`.
- Current state/action update: the commit containing this file.
