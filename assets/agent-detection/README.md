# Agent screen-detection manifests

These TOML files come from [herdr](https://github.com/herdrdev/herdr)
(`src/detect/manifests/`, commit `2563803dca97c040beaf3dc3acdcb5a3221b4238`,
version `2026.10.01.1`), licensed under the Apache License 2.0. `codex.toml`
adds tty7's `ready_prompt` rule for the visible
input prompt and shortcuts footer; preserve this extension when updating
the upstream rules. The other manifests are copied unchanged.
`copilot.toml` is herdr's `github-copilot.toml`, renamed after
tty7's slug.

Each file lists rules that classify a coding agent's pane as `working`,
`blocked` or `idle`, based on what the agent's TUI is showing at the bottom of
the screen. `src/terminal/screen_status.rs` evaluates them the way herdr's
`src/detect/manifest.rs` does.
