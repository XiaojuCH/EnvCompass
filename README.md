# EnvCompass

## 为什么你的项目又跑不起来了？

一键检查 Python、Node.js、PATH、Git 和项目运行时要求之间的冲突。

**找到问题 → 看懂原因 → 把脱敏报告直接发给 AI 或别人。**

[English](./README.en.md) | 简体中文

EnvCompass 是一个 Windows-first、本地优先、只读的开发环境诊断工具。它不安装、不修改、不修复环境，只负责告诉你：哪里不匹配、为什么，以及有什么证据。

## 当前能做什么

- 扫描这台电脑：检查 PATH、Python、Node.js、Git 的实际解析位置和版本。
- 选择项目文件夹：只读取 `.python-version`、`pyproject.toml`、`.nvmrc`、`.node-version`、`package.json` 中与运行时版本有关的最小 metadata，不递归读取源码，不执行项目脚本。
- 解释问题：把项目声明与当前 runtime 做比较，给出可读的 Finding。
- 复制给 AI：生成独立的 Markdown 诊断报告，默认脱敏用户目录和常见 secret/token/credential。
- 保存报告：把脱敏 Markdown 保存到本地。

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

- 用户目录替换为 `%USERPROFILE%`
- 不读取或输出 `.env` 值
- 不输出环境变量 value
- 对常见 API key、token、password、proxy credential 模式再次过滤

请勿把真实扫描报告提交到仓库。

## 测试

```powershell
cd src-tauri
cargo test
```

## License

待定。

