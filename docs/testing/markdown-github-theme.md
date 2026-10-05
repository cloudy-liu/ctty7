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

### Live website audit, 2026-10-05

Final Windows tests passed without local dependency overrides: application
workspace 2,930 passed / 6 ignored; component workspace 296 passed / 2 ignored.
The application pins component revision `31660d5d2446d6abf8ee27a157a62836c0d91d84`.

Chrome DevTools MCP captured the actual GitHub Markdown file preview in an
isolated signed-out context at 1280px and 600px widths. Light and Dark default
computed styles are committed in `tests/fixtures/github-live-2026-10-05`.
The repository README article was checked separately for layout agreement.

This found and corrected three differences from the third-party CSS example:

- Body links are underlined by default and remain underlined on hover.
- The content cap is 1012px with 32px surrounding padding at both widths.
  The native card includes padding and therefore has a 1076px total cap.
- Heading code inherits heading size with 0.2em horizontal and zero vertical
  padding, independently of body inline-code padding.

The new live-layout regression failed on the old 890px effective content cap.
Per-heading code metrics have a native layout test covering reflow and
preservation of selected text. These tests do not replace screenshot comparison.

The selection test also exposed a native bounds bug: an unpositioned observer
recorded heading and block bounds after their text, shifting anchors down by
the content height. Markdown observers now explicitly cover their parent's
top-left corner. A regression checks anchor/block origins before and after
heading-code reflow; it failed on the old bounds.

At the time of that audit, the remaining gaps were native pixel rasterization and spacing,
heading hover permalink controls, native task checkbox appearance, exact
syntax token boundaries and Mermaid renderer output. Chrome CDP inspects
webpages; it does not capture ctty7's native GPUI window. No native automation
tool is currently exposed in this session. Full website parity remains open.

Use [the acceptance sample](../examples/markdown-github-theme.md) unchanged in
ctty7 and on its GitHub file page. Compare the rendered Markdown area, excluding
application chrome. Repeat Light default and Dark default at wide and narrow
reading widths, then Windows display scale 100% and 150%.

Check all six headings, h1/h2 rules, inline code and heading code, kbd keys,
five alerts, nested lists, task states, narrow/overflowing tables, a horizontal
rule, code and diff. Hover a link and switch between GitHub and a v2 custom theme while
partially selected and scrolled. Return to the editor and verify unsaved text.

Color values and layout dimensions must match the recorded website observations.
Native font rasterization, tree-sitter versus TextMate token classification and
Mermaid output remain parity gaps until verified or explicitly accepted by the user.

The complete native/browser screenshot matrix and macOS/Linux visual checks
remain unverified. The later Windows run below does not claim pixel-level parity.

The original audit used GitHub's white / `#0d1117` reading backgrounds.
On 2026-10-05 the user approved a revised acceptance target: the builtin's
outer and body surfaces follow the active application background, without a
new setting; other GitHub element colors and typography stay independent.
Custom v2 packages retain their configured background and paper colors.

The old macOS CI failure in `closing_the_pool_drops_queued_work` was reproduced
by allowing a second worker to run before close. Its corrected test occupies
all workers first, then verifies that closing drops the queued closure. The
production pool is unchanged. The corrected test passed 30 standalone
repetitions; removing queue clearing made it fail, confirming that it still
detects the behavior it protects.

## Native bugfix verification, 2026-10-05

The application uses the pushed Git component revision
`02d0885016cda0703a88c9ce8ad0b3a5aca81af6`, without a local path override.
`cargo test --workspace --locked -j 1` passed: 2,949 tests plus 13 CLI end-to-end
cases, 7 ignored, no failures. Git for Windows `bin` and `usr/bin` were on PATH
for the existing shell integration tests. The component workspace passed 299
tests with 2 documentation tests ignored; example targets compiled. Windows
component tests used `RUST_MIN_STACK=8388608`. Formatting and the application
host-boundary check passed.

Native computer-use testing covered Windows at 175% display scale with application
font size 16 and editor font size 14. Light and Harbor Dark reading surfaces,
wide and narrow reading panes, the repository README and the unchanged acceptance
sample were exercised. Heading inline code no longer overlaps its preceding text;
body code chips and long table cells stay on one line, and table scrolling reaches
the last column. Heading hover links, gray readonly tasks, five alert titles and
icons, Rust and diff highlighting, and full image-failure labels were checked.
Workspace child chevrons are one 14-logical-pixel step below the root.

Mermaid labels now appear in native SVG output. Light/default and dark node
palettes use a transparent outer canvas. Copying a diagram was checked by pasting
its original fence content into the source buffer and undoing that temporary
edit without saving. Expand, zoom in/out, horizontal pan, reset, close and focus
recovery were exercised. Native testing found an initially invisible dialog;
adding the component dialog layer to the application root fixed it. Source mode
was checked with Atom One Light and Atom One Dark after theme changes.

The heading regression runs through deferred layout and fails with the old
ambient font-size lookup. Application tests also cover opening an anchor before
the initial background parse completes, selection, unsaved edits, undo, theme
reflow, and reading-position restoration. The reported original README stack
overflow was not reproduced; the Windows GUI now reserves 8 MiB rather than the
linker's 1 MiB default, with a linked-PE regression checking the reserve.

The manual GPUI headless opening profile measured installation / response frame /
content drawn, respectively: README 25 / 30 / 406 ms; acceptance sample
12 / 18 / 662 ms; Chinese README 7 / 10 / 209 ms. Initial parsing and Mermaid work
run in the background, and source grammar loading waits for source mode. Thirty
warm switches took 3.018 s, about 101 ms per switch. These measurements establish
shorter initial UI blocking, not instant complete rendering or an overall warm
switch speedup; they exclude native GPU presentation. The former synchronous
profile stopped at the first draw, so its timing is not a comparable complete-page
measurement.

Windows 100%/150% display scale, application zoom 200%, and macOS/Linux native
visual checks remain open. Native font rasterization, all syntax token boundaries
and pure-Rust Mermaid geometry have not been established as identical to GitHub.
The user-approved application background is an intentional exception to website
background parity; the PR remains a draft pending the outstanding matrix.
