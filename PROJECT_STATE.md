# PROJECT_STATE

## Current product

- Tauri 2 Windows desktop utility with a React/TypeScript UI and deterministic Rust diagnosis core.
- Windows-first, local-first, read-only, GUI-first; no account, telemetry, built-in AI, elevation, or automatic repair.
- Primary loop: inspect the machine and bounded project metadata, diagnose evidence-backed mismatches, explain limitations, and copy/save a sanitized report.
- Machine checks cover PATH, Python/pip/launcher inventory, Node.js/common package managers, and Git. Direct or mapped network PATH entries are not accessed and are reported as unchecked/unknown.
- Project checks parse bounded Python/Node metadata and count bounded metadata-light Python files. Project scripts and executables are never run; metadata symlinks are refused.
- GUI findings, project read notes, concise Copy for AI, and the technical report are localized for `zh-CN` and `en-US`.
- Concise and technical reports use the same sanitizer for project/home/arbitrary-drive/UNC paths, URLs, credentials, and common token formats.
- The formal UI branch `codex/devdeck-light` applies the owner-approved DevDeck Light shell with P2 Cool Scientific tokens. The result view adds a Finding navigator, primary finding hierarchy, Current vs Required comparison, configuration/terminal evidence blocks, next-step callout, share bar, technical details, and compact status bar. This is presentation-only and does not expand diagnosis scope.

## Public remote and Preview 1

- Repository: `XiaojuCH/EnvCompass`, public, default branch `main`.
- Protected `main` requires a pull request and the `Windows quality and release build` check; no bypass is configured.
- Immutable baseline `v0.1.0-preview.1` points to main commit `81525e371b60781852e3ae7d455ab03d331350f5` and has a real pre-release.
- GitHub Release assets exist for setup EXE, flat portable ZIP, MSI, and `SHA256SUMS.txt`; downloaded asset hashes matched the published checksums.
- Preview 1 CI and tag-triggered Release workflows completed successfully.

## Preview 2 hardening

- Coherent audit Issue #1 was closed automatically when PR #2 was merged.
- PR #2 was squash-merged as `73622c77f05d7e7d0d3297b4940bc5b4bd6d179c`.
- Accepted fixes: network PATH uncertainty and blocking avoidance; bounded probe reader joins; Corepack network disabled in child probes; stderr version parsing; full finding/report localization; safer bounded metadata parsing; Conda/Poetry requirement correctness; stronger report sanitization; narrow-window overflow; real synthetic mismatch fixture/screenshot; clearer MSI guidance.
- CI/Release actions use their current Node-runtime major versions (`actions/checkout@v7`, `actions/setup-node@v7`) after GitHub reported the old Node 20 action runtime as deprecated.
- Preview 2 release notes are prepared at `.github/release-notes/v0.1.0-preview.2.md`; the release workflow will resolve this exact path from tag `v0.1.0-preview.2`.
- README download guidance is version-independent: select the newest Preview marked Pre-release, then choose the stable setup/portable/MSI filename suffix and checksum file from that Release.
- No Preview 2 tag or Release exists; publishing `v0.1.0-preview.2` is the next release step.

## Validation (2026-08-22)

- `cargo fmt --all -- --check`: passed.
- `cargo test --all-targets`: 33 passed, 0 failed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `npm test`: 2 passed, 0 failed.
- `npm run build`: passed.
- DevDeck Light branch: `npm test` 3 passed, `npm run build` passed, `cargo fmt --all -- --check` passed, `cargo test --all-targets` 33 passed, `cargo clippy --all-targets --all-features -- -D warnings` passed, `npm audit --audit-level=high` 0 vulnerabilities, and `npm run tauri:build` produced release EXE, NSIS setup, and MSI bundles.
- `npm audit`: 0 vulnerabilities.
- `npm run tauri build`: release EXE, NSIS setup, and MSI produced.
- PR #2 required CI run `32360211432`: all required steps, including Windows release bundles, passed before merge.
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

1. Push `codex/devdeck-light`, open the requested Draft PR for Issue #6, and run Windows CI.
2. Capture and replace `docs/assets/envcompass-project-results.png` with a real DevDeck Light runtime-mismatch Tauri screenshot before PR review; do not reuse Design Lab or superseded Compass Atelier imagery.
3. Continue public dogfood before any scoped feature expansion.

## Git

- Current `main` baseline: `73622c77f05d7e7d0d3297b4940bc5b4bd6d179c`.
- Preview 2 hardening squash commit: `73622c77f05d7e7d0d3297b4940bc5b4bd6d179c`.
- Current working branch: `codex/devdeck-light`; Issue #6; Draft PR not yet opened; latest handoff state is the commit containing this file.
