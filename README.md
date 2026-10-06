<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/logo.svg">
  <source media="(prefers-color-scheme: light)" srcset="assets/logo.svg">
  <img src="assets/logo.svg" alt="ctty7" height="120" />
</picture>

<h1>
  <img src="https://readme-typing-svg.demolab.com?font=JetBrains+Mono&weight=700&size=48&duration=3000&pause=1000&color=FF5FA2&center=true&vCenter=true&width=435&lines=ctty7" alt="ctty7" />
</h1>

<h3>A modern terminal workbench for AI-powered development</h3>

<p>
  <strong>Persistent sessions</strong> · <strong>Windows shell mastery</strong> · <strong>Coding agent awareness</strong> · <strong>Remote workspaces</strong>
</p>

<p>
  <a href="https://github.com/cloudy-liu/ctty7/releases/latest">
    <img src="https://img.shields.io/github/v/release/cloudy-liu/ctty7?style=for-the-badge&logo=github&color=FF5FA2&logoColor=white" alt="Download" />
  </a>
  <a href="https://github.com/cloudy-liu/ctty7/actions/workflows/ci.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/cloudy-liu/ctty7/ci.yml?style=for-the-badge&logo=githubactions&logoColor=white" alt="CI Status" />
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/license-Apache--2.0-3FDD8C?style=for-the-badge" alt="License" />
  </a>
</p>

<p>
  <a href="README.md">English</a> · <a href="README.zh-CN.md">简体中文</a>
</p>

</div>

<br/>

<div align="center">
  <img src="assets/hero.webp" alt="ctty7 in action" width="100%" style="border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);" />
</div>

<br/>

## 🚀 Why ctty7

ctty7 is built for developers who work with AI coding agents and need reliable terminal infrastructure. Whether you're running Claude Code, Codex, or Herdr-managed workflows, ctty7 keeps your sessions organized, your agent states visible, and your workspaces persistent across machines.

### ✨ What makes it different

<table>
<tr>
<td width="50%">

**🪟 First-class Windows shell support**

CMD and Cmder/Clink with prompt boundaries, working directory tracking, completion, ghost suggestions, and prompt editing. Native Up/Down history behavior preserved; fuzzy search reads PSReadLine and Clink history.

</td>
<td width="50%">

**🤖 Coding agent awareness**

Tabs and sidebar show agent states (working, blocked, done, idle) using hooks and screen detection. Herdr host recognition keeps the sheep avatar visible even when nested agents launch.

</td>
</tr>
<tr>
<td width="50%">

**💾 Persistent workspaces**

Background daemon owns your shells. Close the window, your sessions keep running. Reopen them across local terminals, WSL, and SSH with layouts intact.

</td>
<td width="50%">

**⚡ Built for productivity**

Split panes, renamable sidebar groups, theme-aware avatars, Git status views, configurable shortcuts, selectable prompts, and silent bell by default.

</td>
</tr>
</table>

## 📦 Install

<div align="center">

**[⬇️ Download the latest release](https://github.com/cloudy-liu/ctty7/releases/latest)**

</div>

| Platform | Package | Installation |
|----------|---------|-------------|
| **Windows** 🪟 | `ctty7-*-windows-x86_64-setup.exe` | Run installer, launch from Start menu |
| Windows portable | `ctty7-*-windows-x86_64.zip` | Extract and run `tty7-app.exe` |
| **macOS** 🍎 | `ctty7-*-macos-arm64.dmg` (Apple Silicon)<br>`ctty7-*-macos-x86_64.dmg` (Intel) | Drag to Applications |
| **Linux** 🐧 | `ctty7-*-linux-x86_64.AppImage` | Make executable and run |
| Linux archive | `ctty7-*-linux-x86_64.tar.gz` | Extract and run `tty7-app` |

<details>
<summary>📝 Installation notes</summary>

- Each release includes `checksums.txt` for verification
- Windows builds are unsigned; your OS may require manual confirmation
- macOS uses ad-hoc signing unless release credentials are configured
- **Migrating from older versions?** See the [migration guide](docs/maintenance/migration.md)

</details>

## 🎯 Quick start

1. **Launch ctty7** and create a local terminal, or connect to WSL/SSH
2. **Choose your shell** in Settings. Existing CMD/Cmder arguments like `cmd.exe /K init.bat` remain supported
3. **Start your coding agent** in a pane. Install hooks in **Settings → Agents** to track status and notifications
4. **Check for updates** in **Settings → About → Check now**. Updates are downloaded, verified, and applied explicitly

> 💡 The CLI command remains `tty7` (including `tty7 agents` and `tty7 wait`). No `ctty7` alias is installed.

## 🎨 Features in depth

<details>
<summary><strong>🪟 Windows shell integration</strong></summary>

ctty7 treats Windows shells as first-class citizens:

- **Prompt editing** with Ctrl+A/E, word navigation, and inline editing
- **Working directory tracking** for completion and status display
- **Native history** with Up/Down preserved; fuzzy search reads PSReadLine/Clink
- **Path selection** as single ranges (click-drag Windows paths without breaks)
- **ConPTY win32-input-mode** support for programs that request modified Enter events

</details>

<details>
<summary><strong>🤖 Agent state tracking</strong></summary>

Know what your agents are doing at a glance:

- **Visual states** in tabs and sidebar: working, blocked, done, idle
- **Hook integration** for accurate status reporting (`tty7 agents`, `tty7 wait`)
- **Herdr recognition** — the sheep avatar stays visible when Herdr spawns nested agents
- **Session restoration** — agent identities survive daemon replacement, resuming conversations when possible

</details>

<details>
<summary><strong>🌐 Remote workspaces</strong></summary>

Work seamlessly across machines:

- **Native SSH and WSL support** with in-pane detection and login shell resolution
- **No separate downloads** — Windows packages include the Linux server for WSL bootstrap
- **Consistent experience** across local and remote workspaces

</details>

<details>
<summary><strong>📝 Markdown preview</strong></summary>

Read and edit Markdown without leaving the terminal:

- **Built-in preview** with switchable source editing
- **Reading themes** — GitHub is the default; install your own v2 theme (light/dark aware)
- **Mermaid diagrams** — render flowcharts, sequence diagrams, class diagrams, and more inline
- **Theme integration** — diagrams automatically match your Markdown theme colors

Available in source builds from `main`; the published v0.1.0 does not include
Markdown preview. See [project status](docs/maintenance/status.md).

</details>

## ⚙️ Configuration

Most settings are available in the GUI. Configuration files:

- **Windows**: `%APPDATA%\tty7\config.json`
- **macOS/Linux**: `~/.config/tty7/config.json`

Override with `TTY7_CONFIG_DIR` environment variable or `--config-dir` flag.

### 📚 Documentation

<table>
<tr>
<td width="50%">

**Getting Started**
- [Installation and builds](docs/getting-started/installation.mdx)
- [Configuration reference](docs/reference/configuration.mdx)
- [Updates](docs/reference/updates.mdx)

</td>
<td width="50%">

**Advanced**
- [Agent status and notifications](docs/agents/status.mdx)
- [Agent sessions](docs/agents/sessions.mdx)
- [Remote workspaces](docs/remote/workspaces.mdx)

</td>
</tr>
</table>

## 🛠️ Development

### Prerequisites

- Stable Rust toolchain
- **Windows**: MSVC C++ build tools, Windows SDK
- **Linux**: X11/Wayland/font dev libraries ([details](docs/getting-started/installation.mdx#building-from-source))
- **macOS**: Xcode command line tools

### Build from source

```sh
git clone https://github.com/cloudy-liu/ctty7.git
cd ctty7
cargo build --locked
cargo dev  # Uses .tty7-dev config instead of your main config
```

Before submitting changes:

```sh
cargo fmt --check
cargo test --locked --workspace
```

### Project structure

Rust crates, binaries, protocol identities, and data paths retain `tty7` naming for compatibility and to simplify future selective patch imports. Pull requests target this repository.

**Maintenance docs:**
- [Project status and remaining work](docs/maintenance/status.md)
- [Maintenance history](docs/maintenance/fork-history.md)
- [Release runbook](docs/maintenance/release.md)

## 🤝 Contributing

Contributions are welcome! Please:

1. Check [existing issues](https://github.com/cloudy-liu/ctty7/issues) before opening new ones
2. Run tests and formatting checks before submitting PRs
3. Target the `main` branch for all pull requests

## 📄 License

ctty7 is licensed under [Apache-2.0](LICENSE).

## 🙏 Credits

ctty7 originated from [tty7](https://github.com/l0ng-ai/tty7) and has evolved independently with extensive modifications to Windows integration, agent behavior, and workflow features. Original copyright and attribution are retained.

File and folder icons are the [Symbols](https://github.com/miguelsolorio/vscode-symbols) icon theme by Miguel Solorio (MIT).

---

<div align="center">

<img src="https://img.shields.io/github/stars/cloudy-liu/ctty7?style=social" alt="GitHub stars" />
<img src="https://img.shields.io/github/forks/cloudy-liu/ctty7?style=social" alt="GitHub forks" />

<br/><br/>

**[⬇️ Download ctty7](https://github.com/cloudy-liu/ctty7/releases/latest)** · **[🐛 Report Issue](https://github.com/cloudy-liu/ctty7/issues)** · **[📚 Documentation](docs/)**

<br/>

Made with ❤️ for AI-powered development

</div>
