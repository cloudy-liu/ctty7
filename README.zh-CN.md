<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/logo.svg">
  <source media="(prefers-color-scheme: light)" srcset="assets/logo.svg">
  <img src="assets/logo.svg" alt="ctty7" height="120" />
</picture>

<h1>
  <img src="https://readme-typing-svg.demolab.com?font=JetBrains+Mono&weight=700&size=48&duration=3000&pause=1000&color=FF5FA2&center=true&vCenter=true&width=435&lines=ctty7" alt="ctty7" />
</h1>

<h3>面向 AI 编程的现代终端工作台</h3>

<p>
  <strong>持久会话</strong> · <strong>Windows shell 深度支持</strong> · <strong>编程 Agent 感知</strong> · <strong>远程工作区</strong>
</p>

<p>
  <a href="https://github.com/cloudy-liu/ctty7/releases/latest">
    <img src="https://img.shields.io/github/v/release/cloudy-liu/ctty7?style=for-the-badge&logo=github&color=FF5FA2&logoColor=white" alt="下载" />
  </a>
  <a href="https://github.com/cloudy-liu/ctty7/actions/workflows/ci.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/cloudy-liu/ctty7/ci.yml?style=for-the-badge&logo=githubactions&logoColor=white" alt="CI 状态" />
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/license-Apache--2.0-3FDD8C?style=for-the-badge" alt="许可证" />
  </a>
</p>

<p>
  <a href="README.md">English</a> · <a href="README.zh-CN.md">简体中文</a>
</p>

</div>

<br/>

<div align="center">
  <img src="assets/hero.webp" alt="ctty7 实际运行" width="100%" style="border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);" />
</div>

<br/>

## 🚀 为什么选择 ctty7

ctty7 专为需要可靠终端基础设施、与 AI 编程 Agent 协作的开发者打造。无论你使用 Claude Code、Codex 还是 Herdr 管理的工作流，ctty7 都能让你的会话井然有序、Agent 状态一目了然、工作区跨机器持久化。

### ✨ 核心特色

<table>
<tr>
<td width="50%">

**🪟 Windows shell 一等公民待遇**

CMD 和 Cmder/Clink 完整支持：提示符边界识别、工作目录跟踪、补全、输入建议、提示符编辑。保留原生上下键历史行为；模糊搜索读取 PSReadLine 和 Clink 历史。

</td>
<td width="50%">

**🤖 编程 Agent 状态感知**

标签和侧栏通过 hooks 与屏幕检测显示 Agent 状态（工作中、等待确认、完成、空闲）。Herdr 宿主识别，即使嵌套 Agent 启动时也保持羊头像可见。

</td>
</tr>
<tr>
<td width="50%">

**💾 持久化工作区**

后台守护进程持有你的 shell。关闭窗口，会话继续运行。重新打开本地终端、WSL 和 SSH 工作区时布局完整保留。

</td>
<td width="50%">

**⚡ 生产力优先**

分屏面板、可重命名侧栏分组、主题感知头像、Git 状态视图、可配置快捷键、可选择提示符文本、默认静音响铃。

</td>
</tr>
</table>

## 📦 安装

<div align="center">

**[⬇️ 下载最新版本](https://github.com/cloudy-liu/ctty7/releases/latest)**

</div>

| 平台 | 安装包 | 安装方式 |
|------|--------|----------|
| **Windows** 🪟 | `ctty7-*-windows-x86_64-setup.exe` | 运行安装器，从开始菜单启动 |
| Windows 便携版 | `ctty7-*-windows-x86_64.zip` | 解压后运行 `tty7-app.exe` |
| **macOS** 🍎 | `ctty7-*-macos-arm64.dmg` (Apple Silicon)<br>`ctty7-*-macos-x86_64.dmg` (Intel) | 拖入 Applications |
| **Linux** 🐧 | `ctty7-*-linux-x86_64.AppImage` | 添加执行权限后运行 |
| Linux 压缩包 | `ctty7-*-linux-x86_64.tar.gz` | 解压后运行 `tty7-app` |

<details>
<summary>📝 安装说明</summary>

- 每个发布版本都包含 `checksums.txt` 用于校验
- Windows 构建未签名；操作系统可能要求手动确认
- macOS 在未配置发布凭据时使用临时签名
- **从旧版本迁移？** 查看[迁移指南](docs/maintenance/migration.md)

</details>

## 🎯 快速开始

1. **启动 ctty7**，创建本地终端或连接到 WSL/SSH
2. **在设置中选择 shell**。CMD/Cmder 现有参数如 `cmd.exe /K init.bat` 继续可用
3. **在面板中启动编程 Agent**。在 **设置 → Agents** 安装 hooks 以跟踪状态和通知
4. **在设置 → 关于 → 检查更新** 中检查更新。更新会先下载、校验，然后明确应用

> 💡 命令行仍然使用 `tty7`（包括 `tty7 agents` 和 `tty7 wait`）。不会安装 `ctty7` 别名。

## 🎨 功能详解

<details>
<summary><strong>🪟 Windows shell 集成</strong></summary>

ctty7 将 Windows shell 视为一等公民：

- **提示符编辑** 支持 Ctrl+A/E、单词导航、行内编辑
- **工作目录跟踪** 用于补全和状态显示
- **原生历史** 上下键行为保留；模糊搜索读取 PSReadLine/Clink
- **路径整段选择** 点击拖拽 Windows 路径无断裂
- **ConPTY win32-input-mode 支持** 为请求修改 Enter 事件的程序提供支持

</details>

<details>
<summary><strong>🤖 Agent 状态跟踪</strong></summary>

一眼掌握 Agent 在做什么：

- **可视化状态** 标签和侧栏显示：工作中、等待确认、完成、空闲
- **Hook 集成** 准确的状态报告（`tty7 agents`、`tty7 wait`）
- **Herdr 识别** — Herdr 生成嵌套 Agent 时羊头像保持可见
- **会话恢复** — Agent 身份在守护进程替换后保留，尽可能恢复会话

</details>

<details>
<summary><strong>🌐 远程工作区</strong></summary>

跨机器无缝工作：

- **原生 SSH 和 WSL 支持** 面板内检测和登录 shell 解析
- **无需额外下载** — Windows 安装包包含用于 WSL 引导的 Linux 服务器
- **一致体验** 本地和远程工作区体验统一

</details>

<details>
<summary><strong>📝 Markdown 预览</strong></summary>

无需离开终端即可阅读和编辑 Markdown：

- **内置预览** 可切换源码编辑
- **自定义主题** — 安装自己的主题或使用 Paperglow（自动适配深浅模式）
- _（此功能在当前版本尚未发布）_

</details>

## ⚙️ 配置

大部分设置在 GUI 中可用。配置文件位置：

- **Windows**: `%APPDATA%\tty7\config.json`
- **macOS/Linux**: `~/.config/tty7/config.json`

可使用 `TTY7_CONFIG_DIR` 环境变量或 `--config-dir` 参数覆盖。

### 📚 文档

<table>
<tr>
<td width="50%">

**入门指南**
- [安装与构建](docs/getting-started/installation.mdx)
- [配置参考](docs/reference/configuration.mdx)
- [更新说明](docs/reference/updates.mdx)

</td>
<td width="50%">

**高级功能**
- [Agent 状态和通知](docs/agents/status.mdx)
- [Agent 会话](docs/agents/sessions.mdx)
- [远程工作区](docs/remote/workspaces.mdx)

</td>
</tr>
</table>

## 🛠️ 开发

### 前置要求

- 稳定版 Rust 工具链
- **Windows**: MSVC C++ 构建工具、Windows SDK
- **Linux**: X11/Wayland/字体开发库（[详情](docs/getting-started/installation.mdx#building-from-source)）
- **macOS**: Xcode 命令行工具

### 从源码构建

```sh
git clone https://github.com/cloudy-liu/ctty7.git
cd ctty7
cargo build --locked
cargo dev  # 使用 .tty7-dev 配置而非主配置
```

提交前检查：

```sh
cargo fmt --check
cargo test --locked --workspace
```

### 项目结构

Rust crate、二进制文件、协议标识和数据路径保留 `tty7` 命名以保持兼容性，并简化未来选择性 patch 导入。Pull request 提交到本仓库。

**维护文档：**
- [维护历史](docs/maintenance/fork-history.md)
- [发布操作清单](docs/maintenance/release.md)

## 🤝 贡献

欢迎贡献！请：

1. 提交新 issue 前检查[现有 issues](https://github.com/cloudy-liu/ctty7/issues)
2. 提交 PR 前运行测试和格式检查
3. 所有 pull request 以 `main` 分支为目标

## 📄 许可证

ctty7 使用 [Apache-2.0](LICENSE) 许可证。

## 🙏 致谢

ctty7 源自 [tty7](https://github.com/l0ng-ai/tty7) 并独立演进，在 Windows 集成、Agent 行为、工作流特性等方面进行了大量修改。保留原作者版权和署名。

---

<div align="center">

<img src="https://img.shields.io/github/stars/cloudy-liu/ctty7?style=social" alt="GitHub stars" />
<img src="https://img.shields.io/github/forks/cloudy-liu/ctty7?style=social" alt="GitHub forks" />

<br/><br/>

**[⬇️ 下载 ctty7](https://github.com/cloudy-liu/ctty7/releases/latest)** · **[🐛 报告问题](https://github.com/cloudy-liu/ctty7/issues)** · **[📚 文档](docs/)**

<br/>

用 ❤️ 为 AI 编程打造

</div>
