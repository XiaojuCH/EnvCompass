# EnvCompass 产品规格

> **为什么你的项目又跑不起来了？**

EnvCompass 是一个 **Windows-first、本地优先、只读的开发环境诊断工具**。

它帮助用户检查 Python、Node.js、PATH、Git 与项目运行时要求之间的冲突，解释问题产生的原因，并生成一份经过脱敏的诊断报告，让用户可以直接发送给 AI、GitHub Issue、同学、老师或其他开发者。

---

# 1. 产品一句话

**找到问题 → 看懂原因 → 把报告发给能帮你的人。**

英文长期 tagline：

> **Find why your project won't run.**

我们希望未来用户能够自然地说：

> **“你先用 EnvCompass 扫一下，把报告发我。”**

这句话是 EnvCompass 最核心的产品目标。

---

# 2. 第一目标用户

EnvCompass v0.1 首先服务：

* 中国大陆 Windows 用户；
* 计算机专业学生；
* 初学和中级开发者；
* 使用 AI Coding 工具的开发者；
* 经常遇到“我这里能跑，你那里为什么不能跑”的人；
* 帮别人远程排查环境问题的人。

第一版重点优化中文用户体验。

但项目必须从一开始保持国际参与能力：

* `README.md`：中文首页；
* `README.en.md`：完整英文首页；
* 代码 identifier：英文；
* Git commit：英文；
* GUI：`zh-CN` 与 `en-US`；
* 默认 GUI 语言跟随 Windows 系统语言；
* 中文 Windows 默认中文。

中文优先不等于中文限定。

---

# 3. 核心工作流

EnvCompass 的核心不是“管理环境”。

它的工作流是：

**Inspect → Diagnose → Explain → Share**

即：

**检查 → 诊断 → 解释 → 分享**

第一版不得把产品变成：

* 环境管理器；
* Python/Node 安装器；
* 包管理器；
* 系统优化器；
* AI Chat；
* IDE；
* 万能开发工具箱。

---

# 4. 用户故事

## 4.1 扫描我的电脑

用户双击 EnvCompass。

无需 PowerShell。

无需安装 Python、Node、Rust 或任何开发工具链。

首页提供：

* **扫描这台电脑**
* **选择项目文件夹**

点击“扫描这台电脑”后，EnvCompass 检查当前 Windows 开发环境。

可能发现：

### Python

* 安装了多个 Python；
* `python` 指向 Python 3.13；
* `pip` 却属于 Python 3.11；
* `py` launcher 与 PATH 中 Python 不一致；
* 当前虚拟环境与解释器来源异常；
* WindowsApps alias 抢在真正 Python 前面。

### Node.js

* Node 与 npm 路径异常；
* pnpm/yarn/bun 已安装但不可正确解析；
* 多份 Node 安装产生 shadowing；
* package manager 配置与实际工具不一致。

### PATH

* 完全重复的 PATH；
* 规范化后重复的 PATH；
* 已不存在的目录；
* 支持工具存在多个解析结果；
* WindowsApps 等路径遮蔽真正 runtime。

### Git

* Git 是否可用；
* Git 版本；
* 实际解析到的 Git executable。

---

# 5. 项目扫描

用户可以选择一个项目文件夹。

EnvCompass 不仅列出机器环境，还要把：

**项目声明的要求**

和：

**电脑当前真正使用的环境**

进行比较。

---

## Python 项目

第一版至少识别：

* `.python-version`
* `pyproject.toml`
* `requires-python`

例如：

项目：

```text
.python-version
3.11
```

机器：

```text
python → Python 3.13
```

结果应该是：

> **项目要求 Python 3.11，但当前终端正在使用 Python 3.13。**
>
> 这可能导致依赖安装或运行失败。

而不是：

> Found Python 3.13.

---

## Node 项目

第一版至少识别：

* `.nvmrc`
* `.node-version`
* `package.json`
* `engines.node`
* `packageManager`
* 常见 lockfile

例如：

```json
"engines": {
  "node": ">=20 <23"
}
```

但系统实际：

```text
Node 24
```

应该产生明确 Finding。

---

# 6. EnvCompass 与 envinfo / Repomix 的区别

EnvCompass 不应只是 inventory 工具。

不是：

> “你的电脑有 Python 3.13、Node 24、Git 2.x。”

而是：

> **“项目要求 Python 3.11，但当前解释器是 3.13，而且 pip 还绑定到了另一套 Python。”**

核心区别：

```text
Inventory
    ↓
Diagnosis
    ↓
Explanation
    ↓
Share
```

EnvCompass 也不负责把整个代码库打包给 LLM。

它只收集与开发环境诊断有关的最小必要证据。

---

# 7. Finding 契约

每一个 Problem / Warning 都必须有真实证据。

一个 Finding 在概念上至少应包含：

* `id`
* `severity`
* `title`
* `summary`
* `evidence`
* `recommendation`
* `limitations`（必要时）

建议 severity：

* `problem`
* `warning`
* `info`

---

## 好 Finding

> Python 与 pip 指向不同的 Python 安装。
>
> Python:
> `%USERPROFILE%\...\Python313\python.exe`
>
> pip:
> `%USERPROFILE%\...\Python311\...`

---

## 坏 Finding

> 检测到三个 Python，因此系统有问题。

安装多个 runtime 本身不是错误。

必须证明它真正造成：

* resolution 冲突；
* runtime mismatch；
* 项目要求不满足；
* package manager mismatch；
* 其他明确异常。

---

# 8. 不确定性

必须区分：

* 正常；
* 未安装；
* 查询失败；
* 超时；
* 权限不足；
* 不支持；
* 无法判断。

不能把：

> 没检测到

自动解释成：

> 不存在

也不能因为 probe 失败就断言系统有问题。

---

# 9. 第一版诊断范围

v0.1 优先：

1. Windows / architecture
2. PATH
3. Python
4. Node.js
5. Git
6. Project runtime declarations
7. Share Diagnosis

这些形成第一条完整产品链。

---

# 10. 明确延期

第一版不要因为时间充足就顺手加入：

* CUDA；
* NVIDIA；
* Java；
* Android SDK；
* Rust；
* WSL；
* Docker；
* Visual Studio / MSVC；
* 深度 Conda 分析；
* 自动安装 runtime；
* 自动修复 PATH；
* 自动修改项目；
* MCP server；
* plugin marketplace；
* 云同步；
* 用户账户；
* telemetry；
* 内置 LLM；
* OpenAI / DeepSeek / Claude API Key；
* AI Chat。

它们可以未来增加。

但不能阻碍第一版产品成立。

---

# 11. Read-only 原则

EnvCompass v0.1 是只读诊断工具。

不得：

* 修改 PATH；
* 修改 Registry；
* 创建/删除环境变量；
* 安装软件；
* 卸载软件；
* 修改 Python；
* 修改 Node；
* 自动创建虚拟环境；
* kill process；
* 修改项目文件；
* 执行项目脚本；
* 自动执行修复命令；
* 请求管理员权限。

可以给出推荐命令。

但默认只负责告诉用户：

> 哪里可能有问题，以及为什么。

---

# 12. 外部命令安全

如果需要执行：

```text
python --version
node --version
git --version
```

等本地命令：

必须：

* 明确 allowlist；
* 参数结构化传递；
* 尽量不经过 shell；
* 有 timeout；
* stdout/stderr 有大小限制；
* non-zero exit 不导致整个扫描崩溃。

永远不要执行从项目文件中读取出来的命令。

特别禁止执行：

* `package.json` scripts；
* `.bat`；
* `.cmd`；
* `.ps1`；
* shell script；
* 用户项目中任意 executable。

项目目录是不可信输入。

---

# 13. 隐私原则

诊断结果可能包含：

* Windows 用户名；
* 用户目录；
* 项目目录；
* 内网 URL；
* proxy；
* token；
* API key；
* credential；
* 私有 repository 信息。

因此：

**本地显示**

和：

**导出报告**

属于两个不同 trust surface。

---

## 导出报告必须经过统一 Sanitizer

至少：

* 用户 Home 替换为 `%USERPROFILE%`；
* 不输出环境变量 value；
* 不读取 `.env` value；
* 不导出 proxy credential；
* 不递归读取源码；
* 不导出无关目录内容；
* 对常见 secret/token 模式再次 redaction；
* Markdown 和 JSON 必须走同一套 sanitization contract。

必须有 synthetic privacy tests。

测试数据禁止包含真实用户名、真实 token、真实用户项目路径。

---

# 14. Share Diagnosis

这是核心产品功能，不是附属功能。

结果页必须有：

**复制给 AI**

未来可以有：

**复制为 GitHub Issue**

第一版至少支持：

* Copy for AI
* Save report

---

## 中文报告示例

```markdown
# EnvCompass 开发环境诊断报告

## 系统

Windows 11 x64

## 项目要求

Python 3.11

## 当前环境

Python 3.13
pip → Python 3.11

## 发现的问题

### Python 与 pip 指向不同的 Python 安装

...

## 隐私处理

- 用户目录已替换为 `%USERPROFILE%`
- 环境变量值未包含在报告中
- 已执行敏感信息过滤

请根据以上证据帮助我排查环境问题。
优先提供最小修改方案，不要在没有必要的情况下建议重装整个开发环境。

Generated by EnvCompass
```

英文 GUI 应生成英文版本。

---

# 15. GUI

第一版 GUI 是产品主体。

CLI 未来可以提供给高级用户，但不能替代 GUI。

---

## 首页

推荐：

```text
为什么你的项目又跑不起来了？

EnvCompass 检查你的开发环境，
找出 runtime、PATH 与项目要求之间的冲突。

[ 扫描这台电脑 ]

[ 选择项目文件夹 ]

本地运行 · 只读 · 无遥测
```

不应该先显示：

* Collector；
* Adapter；
* Probe registry；
* Runtime graph；
* JSON；
* schema。

---

## Scanning

展示真实步骤：

```text
正在检查 PATH…
正在检查 Python…
正在检查 Node.js…
正在检查 Git…
正在读取项目要求…
```

单个 probe 失败不得导致整个扫描失败。

GUI 必须保持响应。

---

## Results

顶部：

```text
发现 2 个问题 · 1 个警告 · 7 项检查正常
```

分组：

* 项目
* Python
* Node.js
* PATH
* Git
* 系统

Finding 首屏必须回答：

1. 哪里有问题？
2. 为什么？
3. 有什么证据？
4. 下一步可以做什么？

Technical details 可以折叠。

---

# 16. GUI 视觉原则

EnvCompass 是桌面 utility。

不要做成 AI SaaS Dashboard。

避免：

* 巨型 sidebar；
* 五颜六色 gradient；
* 假图表；
* health score；
* AI 聊天框；
* 一屏十几个 card；
* 没意义的动画。

追求：

* 简洁；
* 清晰；
* 信息层级明确；
* Windows desktop utility 感；
* 强可读性；
* 结果一眼能看懂。

用户应在扫描结束后 **10 秒内理解最重要的问题**。

---

# 17. 国际化

GUI 第一版就必须留出 i18n 结构。

至少：

```text
zh-CN
en-US
```

不要把所有 UI 文案散落 hard-code 在组件里。

默认：

* 中文 Windows → zh-CN；
* 其他系统语言 → en-US；
* 用户可以手工切换语言。

第一晚不要求复杂 locale framework。

但不能让未来国际化必须重构整个 UI。

---

# 18. 技术方向

首选：

* Tauri 2
* React
* TypeScript
* Rust diagnosis core

但产品目标优先于技术偏好。

如果实际环境表明某个具体实现存在严重阻塞，可以做最小合理调整并在 `PROJECT_STATE.md` 记录原因。

Diagnosis core 不应依赖 React。

最终结构应允许未来：

```text
Rust diagnosis core
       │
 ┌─────┴─────┐
GUI         CLI
```

但 v0.1 不需要复杂 framework。

---

# 19. 第一晚成功标准

第一晚不是比代码量。

以下条件比 test 数量重要：

1. 有真正 Windows GUI；
2. 双击 release build 可以启动；
3. 不需要 PowerShell；
4. Scan this PC 有真实扫描结果；
5. 至少 Python / Node / PATH / Git 有真实 evidence；
6. 至少一个 mismatch 能从 backend 一路显示到 GUI；
7. 可以选择项目；
8. 能比较项目要求和当前 runtime；
9. Copy for AI 能产生脱敏 Markdown；
10. 不需要网络/API key；
11. 产品仍是只读；
12. core tests 通过。

如果有：

> 300 tests + plugin framework

但没有：

> 能双击的 GUI

则第一晚判定失败。

---

# 20. 长期判断标准

每一个未来 feature 都问：

> **它是否让“你先用 EnvCompass 扫一下，把报告发我”这句话更有价值？**

如果答案是否定的，它大概率不是当前最高优先级。
