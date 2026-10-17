# Project status

Checked on 2026-10-18 against GitHub and the local checkout.
This is a dated snapshot; query GitHub again before merging or preparing a release.

## Open work

The v0.3.0 release is published and Latest. PRs #128 through #132,
agent notification PR #138, and release preparation #139 are merged. It adds
cross-platform agent sounds, independent notification policies, Copy Name,
and fixes for paste, Cursor identity, Codex status, Mermaid navigation, and
daemon record cleanup. See the [release verification](../testing/release-v0.3.0.md).

There are no open product issues at this check. Release documentation records
the verified publication separately from the frozen release tag.
The component fork `cloudy-liu/gpui-component` now has a `ctty7` stable
integration branch. The application pins its exact commit; see
[gpui-component maintenance](gpui-component.md).

## Completed work

| Work | Completion |
| --- | --- |
| v0.3.0 publication | Published on 2026-10-11 and set Latest after all nine frozen source CI jobs, all ten Release jobs, downloaded assets and checksums, package contents, embedded audio, and public endpoints passed verification. |
| v0.2.2 publication | Published on 2026-10-10 and set Latest after all nine frozen source CI jobs, all ten Release jobs, downloaded assets and checksums, package contents, and public endpoints passed verification. |
| SSH tab identity and update recovery | PRs #122, #124, and #125 merged. Native badge fixtures and regression coverage are recorded in the [SSH tab](../testing/ssh-tab-identity.md), [certificate trust](../testing/update-system-trust.md), and [update recovery](../testing/update-recovery.md) records. |
| v0.2.1 publication | Published on 2026-10-09 and set Latest after source CI, all platform builds, every downloaded asset and checksum, package contents, and public endpoints passed verification. |
| Skill Markdown preview and Mermaid controls | PRs #118 and #119 merged on 2026-10-09. Windows native acceptance and regression coverage are recorded in the [Skill preview](../testing/markdown-skill-preview.md) and [Mermaid controls](../testing/markdown-mermaid-controls.md) verification records. |
| v0.2.0 publication | Published on 2026-10-08 and set Latest after the final source CI, platform builds, all 12 program assets, checksums, and public endpoints passed verification. |
| Automatic source appearance and independent typography | PR #86 merged; issue #84 closed. PR #90 merged on 2026-10-06 with GitHub Light; issue #89 closed as completed. |
| Local terminal startup reliability | PR #92 merged on 2026-10-06 after all nine CI checks passed on `ffa76e55`. Background spawning, retry state, delayed fork commands and split sync are covered in the [verification record](../testing/local-terminal-startup.md). |
| GitHub Markdown reading and v2 custom themes | PRs #88 and #91 merged; issue #85 closed as completed on 2026-10-06. |
| Mermaid preview | PR #80 merged; issue #78 closed. Later GitHub reading fixes are included in #88 and #91. |
| File-tree icons and spacing, current search match, document controls | PRs #82, #83, #87 and #72 merged; document-control issues #68–#70 closed. |
| Compact sidebar status tags | PR #103 merged; #100 closed by #115. PR #123 changes tags to lowercase with deeper colors. Native Windows acceptance covers light/dark, narrow widths, status-only rows, branch elision, and change counts. |
| Markdown theme cleanup, visible dotfiles, and preview switching performance | PRs #102, #104 and #105 merged. |
| Diff width contract and GitHub reading colors | PRs #112 and #114 merged on 2026-10-07. Width regression coverage passes; new desktop acceptance and independent header/hunk/gutter color checks remain unrecorded. |
| Cursor hook and session identity | PR #113 merged on 2026-10-07; PR #129 stabilizes legacy hook attribution. Native Windows startup, two live turns, and Git-tool activity passed the recorded flicker acceptance. |

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

- Published tag `v0.3.0` points to `3fc65a9d`, including PRs #128 through #132, #138, and release preparation #139. All nine [source CI jobs](https://github.com/cloudy-liu/ctty7/actions/runs/38155517374) and all ten [Release jobs](https://github.com/cloudy-liu/ctty7/actions/runs/38155524755) passed. See [verification](../testing/release-v0.3.0.md).
- [v0.3.0](https://github.com/cloudy-liu/ctty7/releases/tag/v0.3.0) is Latest and has 12 program assets plus `checksums.txt`; downloaded bytes, package layouts, versions, embedded notification audio, and public URLs were verified.
- [v0.2.2...v0.3.0](https://github.com/cloudy-liu/ctty7/compare/v0.2.2...v0.3.0) includes agent sounds, Copy Name, paste, identity, status, diagram, and daemon fixes. [English notes](../releases/v0.3.0.md) record remaining manual acceptance limits.

- Published tag `v0.2.2` points to `cafd9ae5`, including PRs #122 through #125 and release preparation #126. All nine [source CI jobs](https://github.com/cloudy-liu/ctty7/actions/runs/38008120429) and all ten [Release jobs](https://github.com/cloudy-liu/ctty7/actions/runs/38008127254) passed. See [verification](../testing/release-v0.2.2.md).
- Published tag `v0.2.1` points to `16d52c92`, including merged fixes #118 and #119 and release preparation #120. All nine [source CI jobs](https://github.com/cloudy-liu/ctty7/actions/runs/37864073395) and all ten [Release jobs](https://github.com/cloudy-liu/ctty7/actions/runs/37864082345) passed. See [verification](../testing/release-v0.2.1.md).
- Published tag `v0.2.0` points to `f9416a52`, including release preparation #115 and the final bilingual README #116. All nine jobs in its [source CI](https://github.com/cloudy-liu/ctty7/actions/runs/37706632693) and all ten [Release jobs](https://github.com/cloudy-liu/ctty7/actions/runs/37706707010) passed. See [verification](../testing/release-v0.2.0.md).
- All nine [PR #90 CI jobs](https://github.com/cloudy-liu/ctty7/actions/runs/37444566340) passed on `73e99d52` before its merge.
- All nine [PR #92 CI jobs](https://github.com/cloudy-liu/ctty7/actions/runs/37450774839) passed on `ffa76e55` before its merge. Local tests passed 2,985 cases with eight ignored.
- All nine [PR #93 CI jobs](https://github.com/cloudy-liu/ctty7/actions/runs/37452292509) passed before the documentation merge.
- The current component pin, `9572b7fab9c0fcf73e34c6118f8a9f10f083f928`, is the first pushed commit on the fork's `ctty7` integration branch. It combines the existing component integration line with the Markdown table, virtualization, and aligned-selection fixes; ctty7's locked metadata, dependency tree, build, and workspace tests pass. The component workspace's 309 component tests and macro test pass on Windows with `RUST_MIN_STACK=8388608`; without it, the queued-update test overflows the default test-thread stack.
- [v0.2.2](https://github.com/cloudy-liu/ctty7/releases/tag/v0.2.2), published on 2026-10-10, is the previous release baseline and retains 12 program assets plus `checksums.txt`. Downloaded bytes, package layouts, versions, and public URLs were verified.
- [v0.2.1](https://github.com/cloudy-liu/ctty7/releases/tag/v0.2.1), published on 2026-10-09, remains available with its original assets and checksums.
- [v0.2.0](https://github.com/cloudy-liu/ctty7/releases/tag/v0.2.0), published on 2026-10-08, remains available with its original assets and checksums.
- [v0.1.0](https://github.com/cloudy-liu/ctty7/releases/tag/v0.1.0), published on 2026-09-29, remains available as an older release.
- [v0.2.1...v0.2.2](https://github.com/cloudy-liu/ctty7/compare/v0.2.1...v0.2.2) includes SSH badges, sidebar tag styling, certificate trust, update cancellation, and recovery preservation. [English notes](../releases/v0.2.2.md) identify remaining manual acceptance limits.
- [v0.2.0...v0.2.1](https://github.com/cloudy-liu/ctty7/compare/v0.2.0...v0.2.1) includes Skill metadata tables, CRLF diagrams, Copy sizing, and Mermaid controls. [English notes](../releases/v0.2.1.md) identify the remaining manual acceptance limits.
- [v0.1.0...v0.2.0](https://github.com/cloudy-liu/ctty7/compare/v0.1.0...v0.2.0) includes Markdown reading, Mermaid, editor appearance, file-tree improvements, diff width/colors, and stable Cursor hook identity. [English notes](../releases/v0.2.0.md) retain the remaining manual acceptance limits.
