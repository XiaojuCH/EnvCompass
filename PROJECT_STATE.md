# PROJECT_STATE

## Current architecture

- Tauri 2 desktop shell (Windows-first)
- React + TypeScript frontend, `zh-CN` / `en-US`
- Rust diagnosis core in `src-tauri/src`
  - `probe.rs`: first-PATH-candidate, read-only external command probes with timeout and concurrently drained bounded output
  - `path.rs`: PATH cleanup suggestions plus evidence-based WindowsApps failure diagnosis
  - `project.rs`: bounded metadata/static project-shape parsing for Python and Node.js
  - `versions.rs`: PEP 440 and Node semver comparison
  - `sanitizer.rs`: shared project/home/arbitrary-path, URL, token, and credential redaction
  - `report.rs`: concise Copy for AI and sanitized technical Markdown
- Tauri `scan-progress` events drive truthful GUI stage state

## Implemented capabilities

- Primary GUI path: choose a project and compare its declarations with the machine environment
- Machine scan: PATH, Python/pip, Node.js/package managers, Git
- Python project markers: `.python-version`, `pyproject.toml`, requirements variants, `environment.yml/.yaml`, `setup.py`, `setup.cfg`, `Pipfile`, `Pipfile.lock`, `poetry.lock`, `uv.lock`
- Node project markers: `.nvmrc`, `.node-version`, `package.json`, `engines.node`, `packageManager`, common lockfiles
- Metadata-light project detection: bounded `.py` file count with excluded data/weights/build/venv directories; source is not read or executed
- Missing runtime declarations produce an explicit “cannot determine reliably” info finding
- Severity taxonomy: problem, warning, cleanup suggestion, diagnostic info
- Copy for AI is concise; technical report contains full inventory/PATH under the same sanitizer

## Real validation (2026-08-20)

- `cargo test`: 24 passed
- strict clippy: passed
- `npm test`: 2 passed
- `npm run build`: passed
- `npm run tauri:build`: release EXE, MSI, and NSIS bundles produced
- Final release EXE launched successfully
- Final release machine scan: WindowsApps Store Python resolved to Python 3.12.10; 0 problems, 0 warnings, 2 cleanup suggestions
- Final release synthetic metadata-light scan: detected Python, `environment.yml`, `requirements.txt`, and 2 `.py` files; showed missing version declaration as unknown, not healthy/failure
- Final Copy for AI checks: no username, raw drive path, UNC path, project name, source content, WindowsApps false positive, or duplicate Evidence section

## Release artifacts

- `src-tauri\target\release\envcompass.exe`
- `src-tauri\target\release\bundle\msi\EnvCompass_0.1.0_x64_en-US.msi`
- `src-tauri\target\release\bundle\nsis\EnvCompass_0.1.0_x64-setup.exe`

Screenshots were saved outside the repository:

- `%USERPROFILE%\Pictures\Screenshots\EnvCompass-home-20260820.png`
- `%USERPROFILE%\Pictures\Screenshots\EnvCompass-project-results-20260820.png`

## Known issues / boundaries

- Static import inference is deferred: no Rust-side Python AST parser has been selected, so imports are not guessed or exported.
- `setup.py` support extracts only a bounded literal `python_requires`; it never executes the file and intentionally ignores dynamic expressions.
- Local GUI paths are real by design; copy/save reports are the sanitized sharing boundary.
- Sanitization covers tested common path/credential patterns but cannot guarantee every unknown secret format; users should still review before sharing.
- `py -0p` labels may be imprecise for nonstandard Python launcher labels.
- Release binaries are not signed and public distribution/reputation behavior has not been validated.

## Future direction (not v0.1)

- Guided Setup / Recipes may be researched for YOLO, PyTorch/CUDA, OpenCV, and data science.
- It must remain separate from read-only diagnosis, require explicit confirmation, and prefer a new isolated environment over modifying system Python/PATH or an existing project.

## Next priorities

1. Broaden realistic synthetic project fixtures and dependency-declaration diagnostics without executing source.
2. Evaluate a bounded Rust-side Python AST parser before implementing optional import inference.
3. Prepare signed preview distribution and recruit public dogfood users after owner approval.

## Git

- Branch: `master`
- Baseline before this round: `6010cdb`
- No remote, push, tag, release, or public repository was created.
