# GitHub Markdown theme verification

The implementation follows [cloudy-liu/ctty7#85](https://github.com/cloudy-liu/ctty7/issues/85).
GitHub theme values are compared against the committed github-markdown-css 5.9.0
light and dark snapshots. Primer selection/control tokens and Octicons sources
are pinned in `assets/markdown-themes/GITHUB-NOTICE` with their MIT licenses.

## Automated checks

- Theme tests cover CSS palette and typography values, v2 validation, reserved
  IDs, the single GitHub builtin and fallback, and v1 upgrade errors without
  modifying the original files. A repaired v2 package restores its saved ID.
- A minimal v2 package produces GitHub's complete reading style and syntax
  colors at scales 100%, 150% and 200%, in both modes. Partial overrides,
  array replacement and explicit null values have separate coverage.
- Configuration tests migrate only the saved Paperglow selection, retain other
  settings, avoid rewriting custom IDs, and recover from a migration write
  failure while using GitHub in memory.
- GPUI reading tests cover public pointer selection across body/code/kbd,
  copy text, theme reflow and diff foreground/background spans with legacy
  fallback colors.
- Application reading tests cover theme reload, mode changes, buffer/edit state,
  selection and restoration of the saved content anchor after typography changes.

Use `cargo test --workspace --locked` for the application and
`cargo test -p gpui-component --lib --features tree-sitter-languages` for the
component. The application pins both component crates to the same Git revision.

## GitHub-only v2 verification

On Windows, 2026-10-04, `cargo test --workspace --locked -j 1` passed with
2,929 tests passed, 6 ignored and no failures, without local dependency
overrides. All 34 Markdown-targeted tests passed. `cargo check --locked
--bin tty7-app`, workspace formatting and the host-boundary check passed.

This run includes the migration success/failure/custom-ID tests and the
corrected pool-close test. The shared target directory's local core package
cache was cleared before final verification to avoid reusing another
worktree's configuration defaults.

## Previous automated verification

On Windows, 2026-10-04, an independent application checkout passed
`cargo test --workspace --locked -j 1` using only the pushed Git component
revision `b3e1c4b5343f0eb27c31d21f7266b5af56ee049b`, with no local path override:
2,923 tests passed, 6 were ignored, and none failed. This includes the search
highlight tests from current fork main.

The component passed `cargo test --all -j 1`: 295 tests passed, 2 documentation
tests were ignored, and none failed; example targets also compiled. Application
workspace formatting and the host-boundary check passed. All 12 Rust files
changed for Markdown in the component passed their formatting check.

## Visual comparison

Use [the acceptance sample](../examples/markdown-github-theme.md) unchanged in
ctty7 and on its GitHub file page. Compare the rendered Markdown area, excluding
application chrome. Repeat Light default and Dark default at wide and narrow
reading widths, then Windows display scale 100% and 150%.

Check all six headings, h1/h2 rules, inline code and heading code, kbd keys,
five alerts, nested lists, task states, narrow/overflowing tables, a horizontal
rule, code and diff. Hover a link and switch between GitHub and a v2 custom theme while
partially selected and scrolled. Return to the editor and verify unsaved text.

Color values and layout dimensions must match the pinned sources. Native font
rasterization and tree-sitter versus TextMate token classification are accepted
differences. Mermaid uses the documented Primer palette roles for this renderer.

The native/browser screenshot matrix and macOS/Linux visual checks have not yet
been verified. This record does not claim pixel-level visual acceptance.

The reading background follows the selected Markdown palette, not the
application background. GitHub stays white in light mode and `#0d1117` in dark
mode even if an application theme uses other colors. No application palette
blending is part of the visual acceptance target.

The old macOS CI failure in `closing_the_pool_drops_queued_work` was reproduced
by allowing a second worker to run before close. Its corrected test occupies
all workers first, then verifies that closing drops the queued closure. The
production pool is unchanged. The corrected test passed 30 standalone
repetitions; removing queue clearing made it fail, confirming that it still
detects the behavior it protects.
