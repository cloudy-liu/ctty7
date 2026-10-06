# Project status

Checked on 2026-10-06 at 18:10 Asia/Shanghai against GitHub and the local checkout.
This is a dated snapshot; query GitHub again before merging or preparing a release.

## Open work

| Item | State | Remaining work |
| --- | --- | --- |
| [ctty7 PR #92](https://github.com/cloudy-liu/ctty7/pull/92), local startup reliability | Open, awaiting latest-head CI | Local workspace tests passed 2,985 cases with eight ignored; merge after cross-platform checks pass. |
| Next official release | Not published | Select the release scope, prepare its version and English notes, then build and verify the draft assets before publication. Follow the [release runbook](release.md). |

There are no open issues in `cloudy-liu/ctty7`. This documentation sync has its
own PR in addition to the startup fix. The component fork
`cloudy-liu/gpui-component` has no open PRs; its four PRs are merged and its
issue tracker is disabled.

## Completed work

| Work | Completion |
| --- | --- |
| Automatic source appearance and independent typography | PR #86 merged; issue #84 closed. PR #90 merged on 2026-10-06 with GitHub Light; issue #89 closed as completed. |
| GitHub Markdown reading and v2 custom themes | PRs #88 and #91 merged; issue #85 closed as completed on 2026-10-06. |
| Mermaid preview | PR #80 merged; issue #78 closed. Later GitHub reading fixes are included in #88 and #91. |
| File-tree icons and spacing, current search match, document controls | PRs #82, #83, #87 and #72 merged; document-control issues #68–#70 closed. |

Markdown acceptance includes the recorded inspection of 24 native images:
macOS and Linux, light and dark, wide and narrow, at three document sections.
The [PR #91 CI run](https://github.com/cloudy-liu/ctty7/actions/runs/37419240897)
passed all nine jobs and retains both platform artifacts. Windows probes ran
at the existing 175% display scale. See the [verification record](../testing/markdown-github-theme.md).

The additional Windows DPI matrix and application 200% checks were cancelled
by the user. They are not outstanding requirements for issue #85. Exact
browser/native glyph rasterization, every language's token classification and
all Mermaid horizontal geometry remain limits of the completed acceptance.

The older Paperglow spec and its unchecked historical checklist do not define
the current backlog. Real SSH/SFTP resource checks and near-limit document
scrolling were not established by the later screenshot matrix. Windows source
visuals, wallpaper/transparency, footer truncation and palette focus have now
been checked. macOS/Linux native source-editor screenshots remain unverified;
these are verification gaps, not additional open GitHub tickets.

## Source and release state

- This documentation branch started at `a2ae6646` and includes the accepted GitHub Light merge `f3c88eeb`; check GitHub for subsequent merges.
- All nine [PR #90 CI jobs](https://github.com/cloudy-liu/ctty7/actions/runs/37444566340) passed on `73e99d52` before its merge.
- The GitHub Light worktree contains the editor branch and the separate local startup fix. Generated `scripts/__pycache__/` remains untracked and is excluded from the commits.
- The current component pin, `87c76bd70cd9cb988e6503503e71dd73abad03c5`, is contained in merged component PR #4.
- [v0.1.0](https://github.com/cloudy-liu/ctty7/releases/tag/v0.1.0), published on 2026-09-29, remains Latest and has 12 program assets plus `checksums.txt`. This sync checked the asset list, not downloaded checksums.
- [Changes since v0.1.0](https://github.com/cloudy-liu/ctty7/compare/v0.1.0...main), including Markdown reading, Mermaid, editor appearance and file-tree improvements, are available from source and have not been included in a newer official release.
