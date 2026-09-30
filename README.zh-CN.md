<div align="center">
<img src="assets/app-icon.svg" alt="ctty7" width="88" height="88" />

# ctty7

面向本地开发、远程工作和 AI 编程的终端工作台。

[English](README.md) · [简体中文](README.zh-CN.md) · [下载](https://github.com/cloudy-liu/ctty7/releases/latest)

[![CI](https://github.com/cloudy-liu/ctty7/actions/workflows/ci.yml/badge.svg)](https://github.com/cloudy-liu/ctty7/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/cloudy-liu/ctty7)](https://github.com/cloudy-liu/ctty7/releases/latest)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

</div>

ctty7 将持久终端会话、本地与远程工作区、Git 操作和编程 Agent 状态放在一起，
重点改进 Windows CMD/Cmder 支持、Herdr 识别和 Agent 会话恢复。
项目由 [l0ng-ai/tty7](https://github.com/l0ng-ai/tty7) fork 并修改，独立维护，与上游无隶属关系。

<img src="assets/hero.webp" alt="终端工作区和编程 Agent 会话" width="900" />

## 主要功能

- **管理多个工作区。** 本地终端、WSL 和 SSH 工作区共用标签、分屏和侧栏；分组可以重命名。
  后台服务持有 shell，关闭窗口后进程继续运行。
- **Windows CMD / Cmder 支持。** 通过提示符边界和工作目录报告提供补全、输入建议及提示符编辑。
  保留 CMD、Clink、PowerShell 等 shell 原生的上下键历史行为，模糊搜索可读取 PSReadLine 和
  Clink 历史。Windows 路径支持整段选择；对于请求 ConPTY win32-input-mode 的程序，
  Shift+Enter 等组合键会按键盘事件传递。
- **Herdr 与嵌套 Agent 识别。** Herdr 作为面板内的宿主应用运行时，即使内部启动 Codex 或
  Claude Code，标签仍显示 Herdr 羊头像。分屏标签的头像跟随当前焦点面板，后台标签沿用最后的焦点。
- **Agent 状态一眼可见。** 标签和侧栏结合 hooks 与 Herdr 式屏幕识别，显示工作中、等待确认、
  完成和空闲等状态。支持情况因 Agent 而异；GUI 的屏幕识别是对 hooks 的补充，
  `tty7 agents` 和 `tty7 wait` 仍使用 hooks 报告的状态。
- **恢复对应的会话。** 后台服务被替换后，重新打开原先的工作区并保留 Agent 会话身份。
  对支持的 Agent，已知会话 ID 时恢复对应对话；无法确定时退回可用 shell。
  系统重启后恢复的是布局和可恢复的对话，不是让运行中的进程跨重启存活。
- **WSL 与 SSH 工作。** 支持远程工作区、Windows 面板内 SSH 识别和 WSL 登录 shell 解析。
  Windows 安装包附带用于启动 WSL 的 Linux server，无需另行下载。
- **日常终端操作。** 内置 Git 状态与差异查看、主题、字体、快捷键、可选择提示符文字和适配主题的
  Agent 头像，默认关闭响铃。

- **直接阅读 Markdown。** 默认打开预览，可切换源码编辑，并安装
  [自定义阅读主题](docs/customization/markdown-themes.mdx)。Paperglow 跟随应用深浅模式，此功能尚未发布。

## 下载与安装

从 [cloudy-liu/ctty7 Releases](https://github.com/cloudy-liu/ctty7/releases/latest)
下载最新正式版。项目只有一个 Release 发布渠道。

| 平台 | 文件 | 启动方式 |
|---|---|---|
| Windows x86_64 | `ctty7-<版本>-windows-x86_64-setup.exe` | 运行安装器，从开始菜单打开 ctty7 |
| Windows 便携版 | `ctty7-<版本>-windows-x86_64.zip` | 解压后运行 `tty7-app.exe` |
| macOS Apple silicon / Intel | `ctty7-<版本>-macos-arm64.dmg` / `…-x86_64.dmg` | 拖入 Applications |
| Linux x86_64 | `ctty7-<版本>-linux-x86_64.AppImage` | 添加执行权限后运行 |
| Linux 压缩包 | `ctty7-<版本>-linux-x86_64.tar.gz` | 解压后运行 `tty7-app` |

为兼容现有安装，macOS 内部应用包目录仍叫 `tty7.app`。Windows 构建未签名；macOS 在未配置
发布签名凭据时使用临时签名，系统可能要求手动确认。每次发布提供 `checksums.txt`；macOS ZIP
供应用更新使用，`tty7-server-*` 是远程工作区内部组件。

**从旧 fork 迁移：** 首次需要手动安装 ctty7 v0.1.0。旧版 26.x 更新器会将 0.1.0 视为降级。
保留原配置和数据，操作步骤见[迁移指南](docs/maintenance/migration.md)。

## 快速上手

1. 启动 ctty7，创建本地终端，或选择 WSL / SSH 工作区。
2. 在设置中选择 shell。CMD/Cmder 原有启动参数继续可用，例如 `cmd.exe /K init.bat`。
3. 在面板中启动编程 Agent。在 **设置 → Agents** 为需要状态和通知的 Agent 安装 hooks。
4. 在 **设置 → 关于 → 检查更新** 获取正式版。安装包会先下载并校验，应用更新需要明确操作。

命令仍然叫 `tty7`，包括 `tty7 agents` 和 `tty7 wait`，不会安装 `ctty7` 命令别名。

## 配置与文档

大部分选项可以在设置中调整。Windows 配置仍在 `%APPDATA%\tty7\config.json`，
macOS/Linux 仍在 `~/.config/tty7/config.json`。可以通过 `TTY7_CONFIG_DIR` 或
`--config-dir` 指定独立配置目录。

- [配置参考](docs/reference/configuration.mdx)
- [安装与构建](docs/getting-started/installation.mdx)
- [Agent 状态和通知](docs/agents/status.mdx)
- [Agent 会话](docs/agents/sessions.mdx)
- [远程工作区](docs/remote/workspaces.mdx)
- [更新说明](docs/reference/updates.mdx)

参考文档中命令和内部组件继续使用 `tty7` 名称。
使用问题请提交到[本仓库 Issues](https://github.com/cloudy-liu/ctty7/issues)。

## 开发与贡献

需要稳定版 Rust 和对应平台的原生构建工具。Windows 需要 MSVC C++ 构建工具与 Windows SDK；
Linux 所需 X11、Wayland 和字体开发库见[构建说明](docs/getting-started/installation.mdx#building-from-source)。

```sh
git clone https://github.com/cloudy-liu/ctty7.git
cd ctty7
cargo build --locked
cargo dev
```

`cargo dev` 使用 `.tty7-dev`，不使用日常配置。提交前运行 `cargo fmt --check` 和
`cargo test --locked --workspace`。

保留 Rust crate、二进制、协议标识和数据路径中的 `tty7` 命名，是为了兼容已有配置和脚本，
也便于以后按需引入 patch。PR 提交到本 fork。
旧差异记录见[维护历史](docs/maintenance/fork-history.md)，发布流程见[发布操作清单](docs/maintenance/release.md)。

## 来源与许可证

ctty7 是 tty7 的修改发行版，使用 [Apache-2.0](LICENSE) 许可证，保留原作者版权和署名。
本发行版修改了 Windows shell 集成、Agent 与 Herdr 行为、品牌、文档以及发布和更新流程。
Agent 文档保留 Herdr 名称及检测规则来源说明。
