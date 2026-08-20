# EnvCompass

## 为什么你的项目又跑不起来了？

一键检查 Python、Node.js、PATH、Git 和项目运行时要求之间的冲突。

**找到问题 → 看懂原因 → 把脱敏报告直接发给 AI 或别人。**

[English](./README.en.md) | 简体中文

EnvCompass 是一个 Windows-first、本地优先、只读的开发环境诊断工具。它不安装、不修改、不修复环境，只负责告诉你：哪里不匹配、为什么，以及有什么证据。

## 当前能做什么

- 扫描这台电脑：检查 PATH、Python、Node.js、Git 的实际解析位置和版本。
- 选择项目并诊断：对照项目要求与本机实际 runtime；这是 GUI 的主入口。
- 识别 Python 项目：支持 `.python-version`、`pyproject.toml`、`requirements*.txt`、`environment.yml/.yaml`、`setup.py`、`setup.cfg`、`Pipfile`、`Pipfile.lock`、`poetry.lock`、`uv.lock`。
- 识别 metadata-light 项目：在有边界的目录深度和文件数量内统计 `.py` 文件，但不读取或执行源码；缺少版本声明时明确显示“无法可靠判断”。
- 识别 Node.js 项目：支持 `.nvmrc`、`.node-version`、`package.json`、`engines.node`、`packageManager` 与常见 lockfile。
- 解释问题：把项目声明与当前 runtime 做比较，给出可读的 Finding。
- 复制给 AI：生成简明、高信号的 Markdown，只保留结论、相关 runtime 与直接证据。
- 技术报告：按需包含完整工具清单和 PATH；简明/技术报告使用同一套 sanitizer。

EnvCompass 当前不会从源码推断依赖。识别 `.py` 文件不等于读取 import，也不会把推断冒充项目声明。

## 怎么运行

普通 Windows 用户不需要 PowerShell，也不需要安装 Python、Node 或 Rust。

### 开发模式

需要 Node.js、Rust（MSVC toolchain）和 Tauri 依赖：

```powershell
npm install
npm run tauri:dev
```

### 构建 release

```powershell
npm run tauri:build
```

Windows release 产物位于 `src-tauri\target\release`。

## 产品边界

v0.1 是只读诊断工具，不是环境管理器。它不会修改 PATH、Registry、环境变量或项目文件，也不会执行扫描项目中的脚本。

暂不包含：CUDA、Java、Android、Docker、WSL、Visual Studio/MSVC 诊断、自动修复、内置 AI、云同步、账户、遥测。

## 隐私

所有“复制给 AI”和“保存报告”的 Markdown 都经过统一 sanitizer：

- 项目根目录替换为 `%PROJECT_ROOT%`
- 用户目录和常见系统目录替换为环境变量占位符
- 其他本地/网络绝对路径不保留私有目录名
- 不读取或输出 `.env` 值
- 不输出环境变量 value
- 对常见 API key、token、password、proxy credential 和 URL 模式再次过滤

本地 GUI 会显示真实路径，便于用户本人排查；只有复制/保存属于默认脱敏的分享边界。Sanitizer 能降低常见泄露风险，但不是对所有未知凭据格式的绝对保证，分享前仍建议快速检查。

请勿把真实扫描报告提交到仓库。

## 测试

```powershell
cd src-tauri
cargo test
cargo clippy --all-targets --all-features -- -D warnings

cd ..
npm test
npm run build
```

## License

待定。
