# Project status

Checked on 2026-10-08 against GitHub and the local checkout.
This is a dated snapshot; query GitHub again before merging or preparing a release.

## Open work

The v0.2.0 release preparation is complete. The release is published and Latest;
PRs #115 and #116 are merged, and compact sidebar parent issue #100 is closed.
See the [release verification](../testing/release-v0.2.0.md) and
[sidebar acceptance](../testing/sidebar-status-tags.md).

There are no open product issues at this check. Release documentation records
the verified publication separately from the frozen release tag.
The component fork
`cloudy-liu/gpui-component` has no open PRs; its four PRs are merged and its
issue tracker is disabled.

## Completed work

| Work | Completion |
| --- | --- |
| v0.2.0 publication | Published on 2026-10-08 and set Latest after the final source CI, platform builds, all 12 program assets, checksums, and public endpoints passed verification. |
| Automatic source appearance and independent typography | PR #86 merged; issue #84 closed. PR #90 merged on 2026-10-06 with GitHub Light; issue #89 closed as completed. |
| Local terminal startup reliability | PR #92 merged on 2026-10-06 after all nine CI checks passed on `ffa76e55`. Background spawning, retry state, delayed fork commands and split sync are covered in the [verification record](../testing/local-terminal-startup.md). |
| GitHub Markdown reading and v2 custom themes | PRs #88 and #91 merged; issue #85 closed as completed on 2026-10-06. |
| Mermaid preview | PR #80 merged; issue #78 closed. Later GitHub reading fixes are included in #88 and #91. |
| File-tree icons and spacing, current search match, document controls | PRs #82, #83, #87 and #72 merged; document-control issues #68–#70 closed. |
| Compact uppercase sidebar tags | PR #103 merged; #100 closed by #115. Native Windows acceptance covers light/dark, narrow widths, disabled text, status-only rows, and read results. |
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

- Published tag `v0.2.0` points to `f9416a52`, including release preparation #115 and the final bilingual README #116. All nine jobs in its [source CI](https://github.com/cloudy-liu/ctty7/actions/runs/37706632693) and all ten [Release jobs](https://github.com/cloudy-liu/ctty7/actions/runs/37706707010) passed. See [verification](../testing/release-v0.2.0.md).
- All nine [PR #90 CI jobs](https://github.com/cloudy-liu/ctty7/actions/runs/37444566340) passed on `73e99d52` before its merge.
- All nine [PR #92 CI jobs](https://github.com/cloudy-liu/ctty7/actions/runs/37450774839) passed on `ffa76e55` before its merge. Local tests passed 2,985 cases with eight ignored.
- All nine [PR #93 CI jobs](https://github.com/cloudy-liu/ctty7/actions/runs/37452292509) passed before the documentation merge.
- The current component pin, `87c76bd70cd9cb988e6503503e71dd73abad03c5`, is contained in merged component PR #4.
- [v0.2.0](https://github.com/cloudy-liu/ctty7/releases/tag/v0.2.0), published on 2026-10-08, is Latest and has 12 program assets plus `checksums.txt`. Downloaded bytes and package layouts were verified before publication; public URLs were checked afterwards.
- [v0.1.0](https://github.com/cloudy-liu/ctty7/releases/tag/v0.1.0), published on 2026-09-29, remains the previous release baseline.
- [v0.1.0...v0.2.0](https://github.com/cloudy-liu/ctty7/compare/v0.1.0...v0.2.0) includes Markdown reading, Mermaid, editor appearance, file-tree improvements, diff width/colors, and stable Cursor hook identity. [English notes](../releases/v0.2.0.md) retain the remaining manual acceptance limits.
