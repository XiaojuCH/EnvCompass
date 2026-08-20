# Contributing to EnvCompass

感谢你帮助改进 EnvCompass。`PRODUCT.md` 是产品边界的最高真源；当前核心循环是 **检查 → 诊断 → 解释 → 分享**。

## 开始之前

- 较大的功能或行为变化请先创建 Issue，说明用户问题与证据。
- 修复应尽量小而明确；不要把 EnvCompass 扩成安装器、环境管理器或自动修复工具。
- 扫描目录是不可信输入。不得执行其中的脚本、命令或 executable。
- Finding 必须有真实 Evidence；无法判断时应明确保留不确定性。

## 本地检查

```powershell
npm ci
npm test
npm run build

cd src-tauri
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
```

涉及桌面工作流时，还应执行 `npm run tauri:build` 并实际启动 release EXE。

## 隐私与 fixtures

- 只使用 synthetic identity、路径、token 和项目 fixture。
- 不提交真实扫描报告、用户名、Home 路径、私有项目内容或 credential。
- 新的报告输出必须通过统一 sanitizer，并添加相应隐私测试。

## Pull Request

请说明：解决的问题、实现边界、验证方式与尚未覆盖的限制。提交信息使用英文。

---

English contributions are welcome. Please open an issue before substantial changes, keep diagnosis evidence-based and read-only, use synthetic fixtures only, and include relevant tests and validation notes in the pull request.
