# Project status

Checked on 2026-10-08 against GitHub and the local checkout.
This is a dated snapshot; query GitHub again before merging or preparing a release.

## Open work

| Item | State | Remaining work |
| --- | --- | --- |
| v0.2.0 | Publication authorized; candidate verification in progress | Version, English notes, rewritten READMEs and native screenshots are prepared. Release PR CI, draft packages, checksum verification, and publication remain. See the [review record](../testing/release-v0.2.0.md). |
| Issue #100, compact sidebar agent tags | Open on GitHub; implementation merged | PR #103 merged and child issue #101 closed. Native Windows checks and documentation updates are complete locally; close the parent when the release preparation merges. See [acceptance](../testing/sidebar-status-tags.md). |

There are no open PRs in `cloudy-liu/ctty7` at this check. Issue #100 is the
only open issue. PRs through #114 are merged.
The component fork
`cloudy-liu/gpui-component` has no open PRs; its four PRs are merged and its
issue tracker is disabled.

## Completed work

| Work | Completion |
| --- | --- |
| Automatic source appearance and independent typography | PR #86 merged; issue #84 closed. PR #90 merged on 2026-10-06 with GitHub Light; issue #89 closed as completed. |
| Local terminal startup reliability | PR #92 merged on 2026-10-06 after all nine CI checks passed on `ffa76e55`. Background spawning, retry state, delayed fork commands and split sync are covered in the [verification record](../testing/local-terminal-startup.md). |
| GitHub Markdown reading and v2 custom themes | PRs #88 and #91 merged; issue #85 closed as completed on 2026-10-06. |
| Mermaid preview | PR #80 merged; issue #78 closed. Later GitHub reading fixes are included in #88 and #91. |
| File-tree icons and spacing, current search match, document controls | PRs #82, #83, #87 and #72 merged; document-control issues #68–#70 closed. |
| Compact uppercase sidebar tags | PR #103 merged; local native acceptance now covers light/dark, narrow widths, disabled text, status-only rows, and read results. |
| Markdown theme cleanup, visible dotfiles, and preview switching performance | PRs #102, #104 and #105 merged. |
| Diff width contract and GitHub reading colors | PRs #112 and #114 merged on 2026-10-07. Width regression coverage passes; new desktop acceptance and independent header/hunk/gutter color checks remain unrecorded. |
| Cursor hook and session identity | PR #113 merged on 2026-10-07. Cursor attribution and stale-session regressions pass; native Windows startup acceptance remains unrecorded. |

Markdown acceptance includes the recorded inspection of 24 native images:
macOS and Linux, light and dark, wide and narrow, at three document sections.
The [PR #91 CI run](https://github.com/cloudy-liu/ctty7/actions/runs/37419240897)
passed all nine jobs and retains both platform artifacts. Windows probes ran
at the existing 175% display scale. See the [verification record](../testing/markdown-github-theme.md).

The additional Windows DPI matrix and application 200% checks were cancelled
by the user. They are not outstanding requirements for issue #85. Exact
browser/native glyph rasterization, every language's token classification and
all Mermaid horizontal geometry remain limits of the completed acceptance.

Real SSH/SFTP resource checks and near-limit document
scrolling were not established by the later screenshot matrix. Windows source
visuals, wallpaper/transparency, footer truncation and palette focus have now
been checked. macOS/Linux native source-editor screenshots remain unverified;
these are verification gaps, not additional open GitHub tickets.

## Source and release state

- Fork `main` is at `8fcc511a`, including PRs #112, #113, and #114. Its [CI run](https://github.com/cloudy-liu/ctty7/actions/runs/37644303956) passed. The v0.2.0 preparation has separate [local verification](../testing/release-v0.2.0.md).
- All nine [PR #90 CI jobs](https://github.com/cloudy-liu/ctty7/actions/runs/37444566340) passed on `73e99d52` before its merge.
- All nine [PR #92 CI jobs](https://github.com/cloudy-liu/ctty7/actions/runs/37450774839) passed on `ffa76e55` before its merge. Local tests passed 2,985 cases with eight ignored.
- All nine [PR #93 CI jobs](https://github.com/cloudy-liu/ctty7/actions/runs/37452292509) passed before the documentation merge.
- The current component pin, `87c76bd70cd9cb988e6503503e71dd73abad03c5`, is contained in merged component PR #4.
- [v0.1.0](https://github.com/cloudy-liu/ctty7/releases/tag/v0.1.0), published on 2026-09-29, remains Latest and has 12 program assets plus `checksums.txt`. This sync checked the asset list, not downloaded checksums.
- [Changes since v0.1.0](https://github.com/cloudy-liu/ctty7/compare/v0.1.0...main), including Markdown reading, Mermaid, editor appearance and file-tree improvements, are available from source and have not been included in a newer official release.
