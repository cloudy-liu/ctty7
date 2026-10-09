<div align="center">

<p align="center" style="line-height: 24px;">
  <img src="assets/logo.svg" alt="ctty7" width="120" height="120" style="margin: 0;" />
  <br />
  <br />
  <img src="https://readme-typing-svg.demolab.com?font=JetBrains+Mono&weight=700&size=48&duration=3000&pause=1000&color=FF5FA2&center=true&vCenter=true&width=435&lines=ctty7" alt="ctty7" style="margin: 0;" />
</p>

cloudy's customized tty7: the terminal that outlives its window, tuned for coding agents.

[Download](https://github.com/cloudy-liu/ctty7/releases/latest) · [Releases](https://github.com/cloudy-liu/ctty7/releases) · [Documentation](docs/index.mdx) · [简体中文](README.zh-CN.md)

[![Release](https://img.shields.io/github/v/release/cloudy-liu/ctty7)](https://github.com/cloudy-liu/ctty7/releases/latest)
[![CI](https://github.com/cloudy-liu/ctty7/actions/workflows/ci.yml/badge.svg)](https://github.com/cloudy-liu/ctty7/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

</div>

<img src="assets/screenshots/agents-light-dark.webp" alt="ctty7 in light and dark appearance with Claude Code in the terminal and coding-agent sessions in the sidebar" width="1280" />

*Claude Code and Codex at their prompts, in light and dark, from native Windows captures. [Capture details](assets/screenshots/README.md).*

## What this is

ctty7 is cloudy's customized version of [tty7](https://github.com/l0ng-ai/tty7), a terminal workbench written in Rust. In tty7 a background server owns your shells, not the window: quit the app and your builds, agents, and sessions keep running; after a reboot your layout, screens, and supported agent conversations come back. That persistence comes with editor-grade input, per-pane agent awareness, and a native SSH and WSL stack, all pure Rust and GPU-rendered; upstream benchmarks put it at roughly twice the throughput of Alacritty, Ghostty, or Kitty on a large `cat`.

ctty7 keeps all of that and focuses on what daily use demanded: Windows shells, coding agents, and reading and editing in the same window. Every change and its upstream status is tracked in [fork history](docs/maintenance/fork-history.md), and fixes that fit upstream are proposed back to it.

## What ctty7 changes

**Windows shells as first-class citizens.** Stock CMD and Cmder/Clink get prompt boundaries, working-directory reporting, completion, ghost suggestions, and prompt editing. Up/Down keeps the shell's native history, and fuzzy search reads PSReadLine and Clink history. Windows paths select as a single range, and modified Enter reaches programs that request ConPTY win32-input-mode instead of submitting the line. [Shell integration](docs/reference/shell-integration.mdx)

**See which agent needs you.** Sidebar rows carry compact idle, run, input, and done capsules, so one glance says who is working, who is blocked on you, and whose result is unread. Known conversations survive restarts and daemon replacement: Claude Code and Codex resume their exact session, and when an identity cannot be recovered, restore leaves a usable shell instead of a session picker. Agent detection walks the Windows process tree, so prompt helpers and MCP child processes are not mistaken for the foreground agent, and every avatar follows the theme. [Agent status](docs/agents/status.mdx) · [Sessions](docs/agents/sessions.mdx)

**Read and edit in the same window.** Markdown opens in a GitHub-styled preview with Mermaid diagrams and installable v2 themes, sharing one buffer with the source editor. The editor follows the app appearance with GitHub Light or Atom One Dark colors and its own typography. File trees and editor headers use Symbols icons, documents fill and restore, and diffs switch between split and unified layouts. [Markdown themes](docs/customization/markdown-themes.mdx)

![GitHub Light source editor in ctty7](assets/screenshots/source-light.webp)

**Remote work without ceremony.** WSL resolves the account's login shell, and Windows packages bundle the Linux server needed to bootstrap it. An `ssh` started inside a pane is detected, so the sidebar follows the remote context instead of the local one. [Remote workspaces](docs/remote/workspaces.mdx) · [SSH](docs/remote/ssh.mdx)

**A release line of its own.** ctty7 runs its own versioned releases with in-app update checks that do not depend on the GitHub REST API, and defaults the terminal bell to off. [Updates](docs/reference/updates.mdx)

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

## Configure and build

Most options are in Settings. Configuration lives at `%APPDATA%\tty7\config.json` on Windows and `~/.config/tty7/config.json` on macOS/Linux; override the directory with `TTY7_CONFIG_DIR` or `--config-dir`. [Configuration reference](docs/reference/configuration.mdx) · [Themes](docs/customization/themes.mdx)

Use stable Rust. Windows needs MSVC C++ build tools and the Windows SDK; macOS needs Xcode command line tools; Linux needs the [documented system libraries](docs/getting-started/installation.mdx#building-from-source).

```sh
git clone https://github.com/cloudy-liu/ctty7.git
cd ctty7
cargo build --locked
cargo dev
```

`cargo dev` uses the isolated `.tty7-dev` configuration directory. Before submitting changes, run `cargo fmt --all -- --check` and `cargo test --locked --workspace`. On Windows, the full test suite also needs Git for Windows' `usr/bin` utilities on the test process's PATH.

Issues and pull requests belong in [cloudy-liu/ctty7](https://github.com/cloudy-liu/ctty7); PRs target its `main` branch. See the [project status](docs/maintenance/status.md), [release runbook](docs/maintenance/release.md), and [fork history](docs/maintenance/fork-history.md).

## Credits and license

ctty7 is maintained from [l0ng-ai/tty7](https://github.com/l0ng-ai/tty7), with original copyright and attribution retained. Licensed under [Apache-2.0](LICENSE).

File and folder icons come from Miguel Solorio's [Symbols](https://github.com/miguelsolorio/vscode-symbols) theme under MIT. Bundled themes and other assets retain their notices in `assets/`.
