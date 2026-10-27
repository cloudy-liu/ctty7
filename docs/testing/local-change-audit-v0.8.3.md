# Preserved local changes for v0.8.3

Checked on 2026-10-27 against the latest published release v0.8.2, the full
range to the release candidate, all seven registered worktrees and all six
preserved stashes. Source changes for this candidate are confined to accepted
fork PRs; release preparation adds version metadata and documentation.

| Preserved source | Finding and disposition |
| --- | --- |
| `release/v0.8.3` worktree | Created from fork main at `635eddd9` after #209 merged; contains only the release version, English notes and this audit/status update. |
| `feat/git-history-avatars` at `eb046777` | Git History author photos, two-line rows and independent complete commit documents are included through cloudy-liu/ctty7#209. The source branch, local verification binaries and original screenshots remain preserved. |
| `fix/editor-horizontal-scrollbar` at `41895513` | The accepted source scrollbar change is included through squash-merged cloudy-liu/ctty7#210 at `98aa012e`; the original branch remains available. |
| `fix/unsupported-preview-hints` at `98418a61` | The accepted unsupported-preview behavior is included through squash-merged cloudy-liu/ctty7#211 at `b03d58ea`. The additional uncommitted custom-name hover change in `src/ui/tab_sidebar.rs` is separate unfinished work and is excluded from this candidate. |
| `fix/background-agent-attention` worktree | Uncommitted Windows shortcut-backed taskbar overlay changes and captures remain in their original worktree. They are unfinished independent work and are excluded from this candidate. |
| `ctty7-preview-checkbox` worktree | Clean preserved checkout of the earlier unsupported-preview implementation; accepted behavior is already included through #211. |
| `.scratch/history-current-baseline` worktree | Its modified `src/document_visual.rs` supplies only the opt-in native measurement fixture. It is not production behavior and is excluded from the release source. |
| Stashes `9c1dc9ea`, `93f5a1ea`, `2bbc44c5`, `846600ab`, `74055c25` and `c73bcbb4` | Hashes and descriptions match the earlier audits. Their included, superseded or intentionally archived dispositions remain as recorded in the [v0.8.2 audit](local-change-audit-v0.8.2.md) and its references. No stash is applied, dropped or rewritten. |

The complete v0.8.2 range includes the GitHub-style Diff context controls in
#206, source scrollbars in #210, optional unsupported-preview hints in #211 and
Git History author rows and commit documents in #209. No inherited capability
or patch-equivalent synchronization is counted as another feature.

PR #209's first Intel macOS CI attempt passed compilation, tests and native
preview validation, then failed when GitHub's artifact service timed out.
The next attempt failed the existing
`fill_and_restore_preserve_a_partial_markdown_selection` test at its initial
selection assertion. The identical failure is present on main at `98aa012e`;
this PR does not modify `src/ui/markdown_preview.rs`. Logs from both attempts
remain under the original CI run and in the History worktree's
`.scratch/pr209-ci/`. The third attempt passed all ten checks on `eb046777`,
including both native
macOS evidence uploads. The squash merge at `635eddd9` has the identical tree.
See [the complete source CI](https://github.com/cloudy-liu/ctty7/actions/runs/40452423904/attempts/3).

See [the English release notes](../releases/v0.8.3.md) and the linked feature
acceptance records. Source CI, platform packaging and downloaded asset/checksum
verification must finish before the draft release is published.
