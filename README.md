<div align="center">

<p align="center" style="line-height: 24px;">
  <img src="assets/logo.svg" alt="ctty7" width="120" height="120" style="margin: 0;" />
  <br />
  <br />
  <img src="https://readme-typing-svg.demolab.com?font=JetBrains+Mono&weight=700&size=48&duration=3000&pause=1000&color=FF5FA2&center=true&vCenter=true&width=435&lines=ctty7" alt="ctty7" style="margin: 0;" />
</p>

A native terminal workbench for coding agents, project files, and persistent sessions.

[Download](https://github.com/cloudy-liu/ctty7/releases/latest) · [Release notes](docs/releases/v0.2.0.md) · [Documentation](docs/index.mdx) · [简体中文](README.zh-CN.md)

[![Release](https://img.shields.io/github/v/release/cloudy-liu/ctty7)](https://github.com/cloudy-liu/ctty7/releases/latest)
[![CI](https://github.com/cloudy-liu/ctty7/actions/workflows/ci.yml/badge.svg)](https://github.com/cloudy-liu/ctty7/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

</div>

<img src="assets/screenshots/agents-light-dark.webp" alt="ctty7 in light and dark appearance with Claude Code in the terminal and coding-agent sessions in the sidebar" width="1280" />

*Light and dark captures from the Windows v0.2.0 source build. Claude Code and Codex are open at their prompts; other sidebar states use sample hook events. [Screenshot details](assets/screenshots/README.md).*

ctty7 is a Rust application rendered with GPUI. The background daemon keeps terminals running when you close a window. In the same workspace, you can monitor coding agents, browse files, edit source, read Markdown, and review Git changes. Local shells, WSL, and SSH workspaces share the interface.

## Install

Get a package from [GitHub Releases](https://github.com/cloudy-liu/ctty7/releases/latest).

| Platform | Package | Start |
| --- | --- | --- |
| Windows x64 | `ctty7-*-windows-x86_64-setup.exe` | Run the installer; launch ctty7 from the Start menu. |
| Windows x64, portable | `ctty7-*-windows-x86_64.zip` | Extract and run `tty7-app.exe`. |
| macOS, Apple Silicon | `ctty7-*-macos-arm64.dmg` | Drag ctty7 into Applications. |
| macOS, Intel | `ctty7-*-macos-x86_64.dmg` | Drag ctty7 into Applications. |
| Linux x64 | `ctty7-*-linux-x86_64.AppImage` | Make it executable and run it. |
| Linux x64, archive | `ctty7-*-linux-x86_64.tar.gz` | Extract and run `tty7-app`. |

Each release includes `checksums.txt`. Windows packages are unsigned; macOS packages use ad hoc signing and are not notarized. Older 26.x installations need a [one-time manual migration](docs/maintenance/migration.md) to the 0.x line. Updates within the 0.x line use **Settings → About → Check now**.

The command remains `tty7`; executables and configuration directories keep their existing names.

## Find your way around

The header has controls to create a tab and toggle the side panels. Below it, the workspace selector switches between saved workspaces. The sidebar groups sessions by repository and shows each tab's name, agent avatar, branch, and Git diff counts. Group names can be changed from their context menu.

**Files** on the right opens the project tree, including dotfiles. The neighboring controls open workspace information and source control. Files, folders, editor headers, and changed-file lists use Symbols icons.

Opening a document adds its own header with the filename, fill/restore control, and close button. Fill gives the document the central workspace while keeping both side panels available. Restore returns to the previous split width. Markdown's **Edit / Preview** switch is in the footer. Diff headers also have a button for split or unified layout.

| Action | Windows / Linux | macOS |
| --- | --- | --- |
| New tab | `Ctrl+Shift+T` | `Cmd+T` |
| Split right | `Ctrl+Shift+D` | `Cmd+D` |
| Command palette | `Ctrl+Shift+P` | `Cmd+P` |
| Toggle session sidebar | `Ctrl+Shift+B` | `Cmd+B` |
| Toggle right panel | `Ctrl+Shift+J` | `Cmd+J` |
| Settings | `Ctrl+,` | `Cmd+,` |

[All shortcuts](docs/reference/keyboard-shortcuts.mdx) can be customized in Settings.

## See which agent needs you

The avatar identifies the agent in the focused pane. A small symbol indicates its state, and the sidebar adds a compact tag beside the branch or working directory.

| Tag | Meaning |
| --- | --- |
| `IDLE` | Waiting for your next prompt, including a finished result you have read. |
| `RUN` | Working on a turn. |
| `INPUT` | Waiting for your reply or permission. |
| `DONE` | Finished this turn, with an unread result. |
| No tag | No known state, or an ordinary shell. An agent with unknown status keeps its grey avatar symbol. |

Tags keep their English labels in every interface language. Hide them in **Settings → Window & Tabs → Show agent status text in sidebar** if you prefer symbols alone.

Status comes from agent hooks and screen detection. Install supported hooks in **Settings → Agents**. The CLI commands `tty7 agents` and `tty7 wait` use hook reports; they do not include the GUI's screen detection.

When Herdr hosts nested agents in a pane, the tab keeps Herdr's purple sheep avatar. A separate ctty7 split shows its own agent when focused. Session restoration retains known conversation identities and resumes supported agents when possible; otherwise, it leaves a usable shell.

[Status and notifications](docs/agents/status.mdx) · [Session restoration](docs/agents/sessions.mdx)

## Read Markdown and edit source

Markdown files open in preview. GitHub is the built-in reading theme, with light and dark appearances, tables, task lists, alerts, links, images, and Mermaid diagrams. Diagram colors follow the reading palette. Switching to source and back uses the same buffer, including unsaved edits.

Install v2 YAML themes to change Markdown typography and colors. Valid edits reload automatically; an unavailable theme falls back to GitHub without discarding your saved selection. See [Markdown themes](docs/customization/markdown-themes.mdx) and [Mermaid support and limits](docs/mermaid-support.md).

Source colors follow the app's appearance: **GitHub Light** in light mode and **Atom One Dark** in dark mode. Source font, size, and line height can be configured separately from the terminal. Search gives the current match a stronger highlight.

![GitHub Light source editor in ctty7](assets/screenshots/source-light.webp)

Try the [workspace walkthrough](docs/examples/workbench-tour.md) in the app. Markdown supports a documented HTML subset; it is not a full browser renderer.

## Windows and remote workflows

The fork's Windows work includes CMD and Cmder/Clink prompt boundaries, directory reporting, completion, ghost suggestions, prompt editing, and native history. Existing custom launch arguments such as `cmd.exe /K init.bat` remain supported. Search reads PSReadLine and Clink history, and Windows paths can be selected as a single range.

Modified Enter input works with programs that request ConPTY win32-input-mode. Function keys pass through to the terminal with their modifiers; bare F11 retains the fullscreen binding. Live theme changes keep terminal text readable, and the bell is off by default.

Windows packages include the Linux server used to bootstrap WSL. WSL resolves the account's login shell. Managed SSH workspaces include file browsing and transfers, while in-pane SSH detection keeps the tab's context aligned with the remote shell.

[Shell integration](docs/reference/shell-integration.mdx) · [Remote workspaces](docs/remote/workspaces.mdx) · [SSH](docs/remote/ssh.mdx)

## Configure and build

Most options are in Settings. Configuration lives at `%APPDATA%\tty7\config.json` on Windows and `~/.config/tty7/config.json` on macOS/Linux. Override the directory with `TTY7_CONFIG_DIR` or `--config-dir`.

[Configuration reference](docs/reference/configuration.mdx) · [Themes](docs/customization/themes.mdx) · [Updates](docs/reference/updates.mdx)

Use stable Rust. Windows needs MSVC C++ build tools and the Windows SDK; macOS needs Xcode command line tools. Linux needs the [documented system libraries](docs/getting-started/installation.mdx#building-from-source).

```sh
git clone https://github.com/cloudy-liu/ctty7.git
cd ctty7
cargo build --locked
cargo dev
```

`cargo dev` uses the isolated `.tty7-dev` configuration directory. Before submitting changes, run `cargo fmt --all -- --check` and `cargo test --locked --workspace`. On Windows, the full test suite also needs Git for Windows' `usr/bin` utilities on the test process's PATH.

Issues and pull requests belong in [cloudy-liu/ctty7](https://github.com/cloudy-liu/ctty7); PRs target its `main` branch. See the [project status](docs/maintenance/status.md), [release runbook](docs/maintenance/release.md), and [historical fork changes](docs/maintenance/fork-history.md).

## Credits and license

ctty7 is maintained from [l0ng-ai/tty7](https://github.com/l0ng-ai/tty7), with fork-specific Windows, agent, editing, and release changes. Original copyright and attribution are retained. Licensed under [Apache-2.0](LICENSE).

File and folder icons come from Miguel Solorio's [Symbols](https://github.com/miguelsolorio/vscode-symbols) theme under MIT. Bundled themes and other assets retain their notices in `assets/`.
