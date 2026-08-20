# PROJECT_STATE

## Current product

- Tauri 2 Windows desktop app with React + TypeScript UI and a Rust diagnosis core
- `zh-CN` / `en-US`, Windows language detection, GUI-first workflow
- Local-first, read-only, no account, no telemetry, no built-in AI
- Primary flow: choose a project, compare declarations with the machine, explain evidence, copy/save a sanitized report
- Machine checks: PATH, Python / pip, Python launcher inventory, Node.js / common package managers, Git
- Project checks: bounded Python and Node.js metadata plus metadata-light `.py` file counting; source is not read or executed
- Concise Copy for AI and full technical Markdown share the same sanitizer

## Public repository readiness

- Canonical Chinese `README.md` and complete English `README.en.md` are stranger-first and recommend GitHub Releases before source builds.
- MIT license, concise contributing/security guidance, a bounded roadmap, and two minimal GitHub Issue forms are present.
- Real screenshots from the final release app are committed under `docs/assets`; they use the synthetic fixture and exclude usernames, absolute paths, and private project data.
- The former Tauri template icon was replaced with the repository-owned `app-icon.svg` and generated Windows bundle icons.
- Package/Cargo/Tauri metadata points to the planned `XiaojuCH/EnvCompass` repository.

## CI and release

- `.github/workflows/ci.yml`: Windows `cargo fmt`, locked Rust tests, strict clippy, frontend tests/build, and real Tauri release build; `contents: read` only.
- `.github/workflows/release.yml`: tag-triggered fresh validation/build, normalized packaging, SHA-256 generation, and GitHub Release creation; only this workflow has `contents: write`.
- Application/Tauri/Cargo version remains `0.1.0`; first Preview tag is planned as `v0.1.0-preview.1`.
- `scripts/package-release.ps1` produces exactly:
  - `EnvCompass-0.1.0-preview.1-windows-x64-setup.exe`
  - `EnvCompass-0.1.0-preview.1-windows-x64-portable.zip`
  - `EnvCompass-0.1.0-preview.1-windows-x64.msi`
  - `SHA256SUMS.txt`
- Portable ZIP opens directly to `EnvCompass.exe`, `QUICKSTART.txt`, and `LICENSE`; it is not nested.
- A matching `.github/release-notes/<tag>.md` is required before the release workflow publishes a tag.

## Validation (2026-08-20)

- `cargo fmt --all -- --check`: passed
- `cargo test --locked`: 24 passed
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: passed
- `npm test`: 2 passed
- `npm run build`: passed
- `npm run tauri:build`: release EXE, NSIS setup, and MSI produced with the new icon/metadata
- Packaging dry-run after the final build: exact names, flat portable contents, and three SHA-256 entries verified
- Workflow and Issue YAML parsed successfully; release tag/title/assets/flags were locally simulated against the installed GitHub CLI
- Final release EXE launched successfully; machine scan and synthetic metadata-light project scan both completed with 0 problems, 0 warnings, and 2 cleanup suggestions

## Known issues / boundaries

- Preview binaries are unsigned; SmartScreen/reputation behavior remains a public-download limitation and is disclosed in README/Release Notes.
- MSI output remains `en-US`; this is an advanced alternate and does not block Preview.
- Static Python import inference is deferred until a bounded reliable parser is selected.
- `setup.py` support extracts only a literal bounded `python_requires`; it never executes the file.
- Local GUI paths are real by design; copy/save is the sanitized sharing boundary.
- Sanitization covers tested common path/credential patterns but cannot guarantee every unknown secret format.
- No GIF/demo is committed; create one later only with a synthetic project and a privacy-reviewed capture.

## Public launch still requires owner action

- Create the empty GitHub repository and push `main`.
- Review GitHub repository/Actions/security settings and let the first CI run pass.
- Push `v0.1.0-preview.1` only after reviewing the release notes and unsigned-build disclosure.
- No remote, push, tag, public repository, or GitHub Release has been created locally.

## Next priorities

1. Bootstrap `XiaojuCH/EnvCompass`, confirm CI, and enable the appropriate GitHub security settings.
2. Publish Preview 1 from the reviewed tag and verify the actual public download/checksum path on a second Windows machine.
3. Recruit public dogfood users before starting Conda/Recipes feature work.

## Git

- Branch: `main`
- Launch Prep base: `1354de2`
- Launch Prep commit: the commit containing this state file (`chore: prepare EnvCompass for public preview`)
- No remote and no tags
