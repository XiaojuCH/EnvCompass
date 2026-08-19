# PROJECT_STATE

## Current architecture

- Tauri 2 desktop shell (Windows-first)
- React + TypeScript frontend, zh-CN / en-US
- Rust diagnosis core in `src-tauri/src`
  - `probe.rs`: read-only, timeout-bounded external command queries
  - `path.rs`: PATH duplicate/missing/shadowing checks
  - `project.rs`: bounded metadata parsing for Python/Node project requirements
  - `versions.rs`: PEP 440 and Node semver comparison
  - `sanitizer.rs`: shared redaction for share/save
  - `report.rs`: Markdown report generation

## Implemented capabilities

- Scan this PC: PATH, Python, Node.js, Git
- Select project folder: `.python-version`, `pyproject.toml requires-python`, `.nvmrc`, `.node-version`, `package.json engines.node` and `packageManager`, common lockfiles
- Findings: Python/pip mismatch, project runtime mismatch, package-manager mismatch, PATH duplicates/missing entries/WindowsApps shadowing
- Copy for AI and Save report with sanitized Markdown
- Basic i18n boundary

## Real validation

- `cargo test`: 13 tests passed
- `npm run build`: TypeScript + Vite production build passed
- `npm run tauri:dev`: app launched, process stayed responsive with window title `EnvCompass`
- GUI screenshot captured for inspection (not yet visually verified by the agent because image viewing is unavailable in this run)

## Known issues

- Ports 1338-1437 are excluded by Windows on this machine; dev server uses port 1520.
- Frontend copy uses `navigator.clipboard`, which may be unavailable in some WebView contexts; fallback is manual copy.
- `py -0p` parsing labels may be imprecise for nonstandard Python launcher labels.

## Privacy gaps

- Local GUI still displays real paths by design; only share/save are sanitized.
- The sanitizer covers common secret patterns but is not a guarantee against every credential format.

## Next priorities

1. Perform a full end-to-end GUI dogfood: scan PC, scan a synthetic project, copy report, inspect Markdown.
2. Produce and launch the real Windows release build.
3. Tighten GUI UX and copy/save error states.

