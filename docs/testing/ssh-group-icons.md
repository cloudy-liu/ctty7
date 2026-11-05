# SSH group icon settings

SSH host groups use the same arrow and folder settings as local project
groups, including when collapsed. The regression test renders both groups
with all four arrow/folder combinations in expanded and collapsed states.

```text
cargo test --locked --offline --bin tty7-app ui::tab_sidebar::tests -- --test-threads=1
cargo fmt --all -- --check
git diff --check
```

Before the fix, `ssh_group_icons_follow_the_same_settings_as_project_groups`
failed on the SSH header with `collapsed=false, arrows=false, folders=true`:
the arrow was present when it should have been absent.

## Native reproduction

Build with `cargo build --locked --features markdown-visual-tests --bin tty7-app`,
then launch `tty7-app --config-dir <isolated-config> --ssh-tab-visual options.json`.
Use the same options for the baseline and fixed builds:

```json
{
  "sidebar": true,
  "output": "capture.png",
  "hold_seconds": 1200,
  "sidebar_group_arrows": false,
  "sidebar_group_folders": true
}
```

Inspect the `prod-web` and `dev-box` group headers. The baseline retains
arrows; the fixed build shows open folders. This fixture uses production
GPUI rendering and deterministic daemon messages without a live SSH connection.

## Known limitations

Visual acceptance is **unverified for PR evidence**. LCU displayed the native
before/after windows during local verification, but its screenshot export
returned `EPERM: operation not permitted` for both the intended repository
image path and a temporary output path. No durable before/after screenshots
are committed or linked. Keep the PR in draft until matching native captures
are archived and linked, or the limitation is explicitly accepted.
