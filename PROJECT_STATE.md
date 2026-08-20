# PROJECT_STATE

## Current product

- Tauri 2 Windows desktop utility with a React/TypeScript UI and deterministic Rust diagnosis core.
- Windows-first, local-first, read-only, GUI-first; no account, telemetry, built-in AI, elevation, or automatic repair.
- Primary loop: inspect the machine and bounded project metadata, diagnose evidence-backed mismatches, explain limitations, and copy/save a sanitized report.
- Machine checks cover PATH, Python/pip/launcher inventory, Node.js/common package managers, and Git. Direct or mapped network PATH entries are not accessed and are reported as unchecked/unknown.
- Project checks parse bounded Python/Node metadata and count bounded metadata-light Python files. Project scripts and executables are never run; metadata symlinks are refused.
- GUI findings, project read notes, concise Copy for AI, and the technical report are localized for `zh-CN` and `en-US`.
- Concise and technical reports use the same sanitizer for project/home/arbitrary-drive/UNC paths, URLs, credentials, and common token formats.
- The GUI now uses the Compass Atelier visual system: a calibration-compass emblem, instrument-panel linework, restrained navy/ivory/teal/red tokens, and first-class light/dark themes. This is presentation-only and does not expand diagnosis scope.

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
- `v0.1.0-preview.2` is published as a GitHub pre-release from main commit `b0f15b98db618804fed02daca17bc6411d264416`.

## Compass Atelier visual identity

- Tracking issue: #4. Draft PR: #5. Working branch: `codex/compass-atelier`, based on `b0f15b98db618804fed02daca17bc6411d264416`.
- Results now follow: overall diagnosis → primary evidence-backed deviation → Current/Required comparison → direct evidence → next step → project/environment context → share → technical details.
- Home, scanning, healthy, runtime mismatch, metadata-light, and unknown-project states were exercised in the real desktop app. No mascot, new diagnostic domain, AI feature, telemetry, repair, or environment mutation was added.
- Documentation screenshots were refreshed from the real Windows application at `docs/assets/envcompass-home.png` and `docs/assets/envcompass-project-results.png`.

## Validation (2026-08-20)

- `cargo fmt --all -- --check`: passed.
- `cargo test --all-targets`: 33 passed, 0 failed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `npm test`: 4 passed, 0 failed.
- `npm run build`: passed.
- `npm audit`: 0 vulnerabilities.
- `npm run tauri build`: release EXE, NSIS setup, and MSI produced.
- PR #2 required CI run `32360211432`: all required steps, including Windows release bundles, passed before merge.
- Compass Atelier release-build GUI: release EXE launched and completed a healthy machine scan. Debug desktop workflows covered healthy machine, runtime mismatch, metadata-light, and unknown-project states; light/dark and Chinese/English were checked.
- Responsive browser checks covered 820, 1040, 1440, and 2560 CSS-pixel widths with no horizontal overflow. Keyboard focus and the reduced-motion CSS fallback were inspected.
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

1. Complete review and CI for the Compass Atelier Draft PR; do not merge until the visual direction is accepted.
2. Continue public dogfood before any scoped feature expansion.
3. Keep mascot exploration, deeper Conda behavior, CUDA/Java/WSL/Docker, AI, telemetry, and repair out of this visual-identity PR.

## Git

- Current `main` baseline: `b0f15b98db618804fed02daca17bc6411d264416`.
- Preview 2 hardening squash commit: `73622c77f05d7e7d0d3297b4940bc5b4bd6d179c`.
- Current working branch: `codex/compass-atelier`; issue #4; Draft PR #5; latest handoff state is the commit containing this file.
