<div align="center">

<p align="center" style="line-height: 24px;">
  <img src="assets/logo.svg" alt="ctty7" width="120" height="120" style="margin: 0;" />
  <br />
  <br />
  <img src="https://readme-typing-svg.demolab.com?font=JetBrains+Mono&weight=700&size=48&duration=3000&pause=1000&color=FF5FA2&center=true&vCenter=true&width=435&lines=ctty7" alt="ctty7" style="margin: 0;" />
</p>

把编码 Agent、项目文件和持久终端会话放在同一个原生工作台里。

[下载](https://github.com/cloudy-liu/ctty7/releases/latest) · [发布说明](docs/releases/v0.2.0.md) · [文档](docs/index.mdx) · [English](README.md)

[![Release](https://img.shields.io/github/v/release/cloudy-liu/ctty7)](https://github.com/cloudy-liu/ctty7/releases/latest)
[![CI](https://github.com/cloudy-liu/ctty7/actions/workflows/ci.yml/badge.svg)](https://github.com/cloudy-liu/ctty7/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

</div>

<img src="assets/screenshots/agents-light-dark.webp" alt="ctty7 浅色与深色实际界面：中央 Claude Code 终端和左侧编码 Agent 会话列表" width="1280" />

*浅色与深色截图来自 Windows 上的 v0.2.0 源码构建。Claude Code 和 Codex 停留在实际输入提示符，其他侧栏状态由样例 hook 事件驱动。[截图说明](assets/screenshots/README.md)。*

ctty7 使用 Rust 和 GPUI 构建。关闭窗口后，后台 daemon 继续保留终端会话。同一个工作区里可以查看 Agent 状态、浏览文件、编辑源码、阅读 Markdown 和检查 Git diff。本地 shell、WSL 和 SSH 工作区共用这套界面。

## 安装

从 [GitHub Releases](https://github.com/cloudy-liu/ctty7/releases/latest) 选择对应的安装包。

| 平台 | 文件 | 启动方式 |
| --- | --- | --- |
| Windows x64 | `ctty7-*-windows-x86_64-setup.exe` | 运行安装程序，从开始菜单启动 ctty7。 |
| Windows x64 便携版 | `ctty7-*-windows-x86_64.zip` | 解压后运行 `tty7-app.exe`。 |
| macOS Apple Silicon | `ctty7-*-macos-arm64.dmg` | 将 ctty7 拖入 Applications。 |
| macOS Intel | `ctty7-*-macos-x86_64.dmg` | 将 ctty7 拖入 Applications。 |
| Linux x64 | `ctty7-*-linux-x86_64.AppImage` | 添加执行权限后运行。 |
| Linux x64 压缩包 | `ctty7-*-linux-x86_64.tar.gz` | 解压后运行 `tty7-app`。 |

每次发布都附带 `checksums.txt`。Windows 安装包未签名；macOS 使用 ad hoc 签名，尚未公证。旧 26.x 版本需要[手动迁移一次](docs/maintenance/migration.md)，之后即可沿 0.x 版本线更新。检查更新入口是 **设置 → 关于 → 立即检查**。

CLI 命令仍叫 `tty7`，可执行文件和配置目录也保留原有名称。

## 认识当前界面

顶部 Header 提供新建标签和左右侧栏开关。下方的工作区选择器用来切换已保存的工作区。左侧栏按仓库分组，列出会话名称、Agent 头像、Git 分支和增删行数；分组标题可以通过右键菜单重命名。

右侧 **Files** 打开项目文件树，默认显示点文件。旁边的按钮可以切换工作区信息和版本控制。文件树、编辑器标题及变更列表都使用 Symbols 文件与文件夹图标。

打开文档后，会出现独立的文档 Header，显示文件名、填满/还原按钮和关闭按钮。填满只占据中间工作区，左右侧栏仍可使用；还原则回到先前的分栏宽度。Markdown 的 **编辑 / 预览** 切换位于底部状态栏。Diff Header 另有按钮切换并排和统一布局。

| 操作 | Windows / Linux | macOS |
| --- | --- | --- |
| 新建标签 | `Ctrl+Shift+T` | `Cmd+T` |
| 向右分屏 | `Ctrl+Shift+D` | `Cmd+D` |
| 命令面板 | `Ctrl+Shift+P` | `Cmd+P` |
| 开关会话侧栏 | `Ctrl+Shift+B` | `Cmd+B` |
| 开关右侧面板 | `Ctrl+Shift+J` | `Cmd+J` |
| 设置 | `Ctrl+,` | `Cmd+,` |

[完整快捷键列表](docs/reference/keyboard-shortcuts.mdx)可以在设置中自定义。

## 看清哪个 Agent 需要你

头像标识当前聚焦面板中的 Agent，小符号表示状态。侧栏在分支或工作目录前增加紧凑的状态标签。

| 标签 | 含义 |
| --- | --- |
| `IDLE` | 等待下一条指令，也包括你已经读过的完成结果。 |
| `RUN` | 正在执行任务。 |
| `INPUT` | 等待回复或权限确认。 |
| `DONE` | 当前任务已完成，结果尚未读过。 |
| 不显示标签 | 状态未知，或只是普通 shell。状态未知的 Agent 仍保留灰色头像状态符号。 |

这些标签在所有界面语言中保持英文大写。只想保留头像符号时，可以关闭 **设置 → 窗口与标签 → 在侧栏显示 Agent 状态文字**。

状态来自 Agent hooks 和屏幕检测。支持的 hooks 可在 **设置 → Agents** 中安装。CLI 的 `tty7 agents` 与 `tty7 wait` 读取 hook 报告，不包含 GUI 的屏幕检测结果。

如果 Herdr 在面板中托管了其他 Agent，外层标签保留 Herdr 的紫色羊头像。另一个独立 ctty7 分屏聚焦后，显示那个分屏自己的 Agent。恢复会话时会保留已知对话身份，并在支持的情况下继续原对话；无法恢复时留下可用的 shell。

[状态与通知](docs/agents/status.mdx) · [会话恢复](docs/agents/sessions.mdx)

## 阅读 Markdown，编辑源码

Markdown 文件默认进入预览。内置 GitHub 阅读主题支持深浅模式、表格、任务列表、提示块、链接、图片和 Mermaid 图表。图表配色随阅读主题变化。源码和预览共用同一缓冲区，未保存的修改也能直接预览。

可以安装 v2 YAML 主题调整 Markdown 字体、排版和颜色。有效修改会自动重载；主题不可用时回退到 GitHub，同时保留原先的主题选择。详见 [Markdown 阅读主题](docs/customization/markdown-themes.mdx)与 [Mermaid 支持范围](docs/mermaid-support.md)。

源码配色跟随应用深浅模式：浅色使用 **GitHub Light**，深色使用 **Atom One Dark**。源码字体、字号和行高独立于终端设置；搜索时，当前匹配项使用更明显的高亮。

![ctty7 中的 GitHub Light 源码编辑器](assets/screenshots/source-light.webp)

可以在应用中打开[工作区导览](docs/examples/workbench-tour.md)试用这些功能。Markdown 支持文档列出的 HTML 子集，不等同于完整浏览器。

## Windows 与远程工作流

本分支补充了 CMD、Cmder/Clink 的提示符边界、目录上报、补全、灰色建议、行内编辑和原生历史行为。`cmd.exe /K init.bat` 等自定义启动参数会按原样保留。搜索可以读取 PSReadLine 和 Clink 历史，Windows 路径可以作为完整范围选中。

对于请求 ConPTY win32-input-mode 的程序，支持带修饰键的 Enter。功能键及其修饰组合可以传给终端，单独 F11 仍用于全屏。切换深浅主题时会处理终端文字与背景的可读性，响铃默认关闭。

Windows 安装包内含 WSL 启动所需的 Linux server，WSL 使用账户配置的登录 shell。托管 SSH 工作区支持文件浏览与传输；从面板内启动 SSH 时，也会识别远程上下文并更新标签信息。

[Shell 集成](docs/reference/shell-integration.mdx) · [远程工作区](docs/remote/workspaces.mdx) · [SSH](docs/remote/ssh.mdx)

## 配置与构建

大多数选项可以在设置中修改。Windows 配置位于 `%APPDATA%\tty7\config.json`，macOS/Linux 位于 `~/.config/tty7/config.json`。使用 `TTY7_CONFIG_DIR` 环境变量或 `--config-dir` 参数可以指定独立目录。

[配置参考](docs/reference/configuration.mdx) · [界面主题](docs/customization/themes.mdx) · [更新机制](docs/reference/updates.mdx)

使用 stable Rust。Windows 需要 MSVC C++ 构建工具和 Windows SDK；macOS 需要 Xcode command line tools；Linux 需要[对应的系统依赖](docs/getting-started/installation.mdx#building-from-source)。

```sh
git clone https://github.com/cloudy-liu/ctty7.git
cd ctty7
cargo build --locked
cargo dev
```

`cargo dev` 使用隔离的 `.tty7-dev` 配置目录。提交前运行 `cargo fmt --all -- --check` 和 `cargo test --locked --workspace`。Windows 完整测试还需要把 Git for Windows 的 `usr/bin` 工具目录加入测试进程的 PATH。

Issue 和 PR 提交到 [cloudy-liu/ctty7](https://github.com/cloudy-liu/ctty7)，PR 目标分支为该仓库的 `main`。维护信息见[项目状态](docs/maintenance/status.md)、[发布流程](docs/maintenance/release.md)和[历史客制化记录](docs/maintenance/fork-history.md)。

## 来源与许可

ctty7 基于 [l0ng-ai/tty7](https://github.com/l0ng-ai/tty7) 维护，在 Windows、Agent、文档编辑和发布流程上持续客制化。保留原始版权与署名，采用 [Apache-2.0](LICENSE) 许可。

文件和文件夹图标来自 Miguel Solorio 的 [Symbols](https://github.com/miguelsolorio/vscode-symbols)，采用 MIT 许可。其他内置主题与素材的许可说明保留在 `assets/` 中。
