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

## Mermaid background follow-up

A focused output regression checks the final SVG root background for flowchart,
sequence and grouped flowchart diagrams in both light/dark modes, with builtin
and custom palettes: 12 combinations. Builtin canvases are transparent; custom
canvases match their configured background. Restoring the renderer's former
default output setting makes this test fail with a white builtin canvas, so the
check detects the reported background class rather than only successful rendering.
Native computer-use inspection of the latest executable confirmed transparent
canvases for inline and expanded diagrams in Harbor Dark and Light; the inline
canvas also followed Rosé Pine Dawn's warm reading background without a white
rectangle. Node fills
remain Mermaid's own gray/dark and lavender/light colors, as required by the
approved preservation of element colors. This follow-up did not reproduce a new
white-canvas defect; it does not establish xAI/Grok styling equivalence.

## Remote README images and animation support

GPUI now receives the application HTTP client at GUI startup. Remote Markdown
images previously used GPUI's unconfigured client and showed their alt text.
The client shares the updater's proxy normalization and user-agent policy.
The proxy selection applies at launch; changing it requires a GUI restart.

A real loopback HTTP fixture exercises the GPUI image loader, rather than a
substitute decoder. It fetches a static SVG and an animated GIF, checks the
SVG dimensions and preserves both GIF frames. A second check verifies the
configured bare proxy is normalized for GPUI. Omitting client installation
makes the HTTP image test fail before the fixture can serve an image.

README's `readme-typing-svg.demolab.com` title uses a looping SMIL `<animate>`
to create its `textPath` geometry. The animation adapter now samples that
geometry and renders actual playback frames with resvg. Remote HTTP images
and files from the owning Host use the same validation and preparation path.
XML root parsing accepts comments, declarations and a BOM before local SVGs.
The component's visible image has its own stable ID within the indexed image
wrapper, retaining GPUI's GIF/WebP/SVG frame state and repaint scheduling.

Supported SVG animation is deliberately bounded: parent-targeted `<animate>`
with a zero-start indefinite or self-restarting loop, linear/discrete values,
numeric or color attributes, and matching non-arc path commands. Tracks must
share a duration of 0.1 to 12 seconds. Playback samples at 20 frames/second,
with at most 240 frames and 8 million total frame pixels. Encoded images are
limited to 8 MiB; remote requests time out after 15 seconds. The document's
64-image and 32 MiB cache limits include generated SVG frame bytes.

CSS/JavaScript animation, animateTransform/animateMotion, finite or event
timelines, additive animation, arc morphing and different track durations are
not implemented. Unsupported SMIL returns an image error with alt text and
a tooltip. Animated SVGs load base64 TTF/OTF `@font-face` data, including CSS
family aliases, into a document-private font database before rasterization.
Embedded families override installed faces without changing other documents.
Font data is limited to 2 MiB total and 16 faces per SVG. WOFF/WOFF2 fonts,
external font URLs, nested images and other external resources are disabled.
Other text falls back to system fonts. These limits do not establish browser
pixel parity or the remaining platform/DPI
matrix. Static SVG and the existing GIF/WebP decoders retain their paths.

Regression checks cover changing rendered SVG pixels, interpolation and
allocation limits, local XML prologs, the shared HTTP/Host preparation path,
HTTP failure/size handling, and actual visible image frame progression in an
active GPUI test window. Removing the image ID makes the playback regression
fail; the XML-prolog test and typing-image pixel test failed before repair.
An embedded-font pixel regression also failed with system fallback and now
matches a reference rendered with the explicit JetBrains Mono Bold subset.
Font tests cover quoted/unquoted CSS URLs, family aliases, document scope,
external-source rejection and malformed/oversized data.

Native Windows computer-use verification on 2026-10-05 used the rebuilt
`tty7-app-animation-check.exe` and the file-panel reading view. The remote
title and its comment-prefixed local copy changed between full `ctty7`,
partial `ctt`, and empty typing frames; the two-frame GIF changed from red
to green. Switching to source mode and back retained working playback.
The actual README showed the animated title and loaded release/build/license
badges. Screenshots and a reusable probe are retained in the local
`.humanlayer/tasks/theme-spec-native-verification` directory.
The user then reported that the title's font still differed from GitHub.
The captured SVG embeds JetBrains Mono Bold, which the original animation
adapter ignored. Native re-verification initially stopped with physical
Escape. After the user resumed the task on 2026-10-06, Computer Use opened
the actual README from the file tree in the rebuilt isolated PR executable
`tty7-app-font-check.exe`. The title displayed the supplied bold pink font,
and changed from full text to an empty animation phase after switching to
source mode and back. Badges and the README screenshot loaded normally.
Evidence is saved as `readme-font-fixed.png`,
`readme-font-fixed-after-mode-switch.png` and
`readme-font-fixed-empty-frame.png` in the same local task directory.
These observations verify the reported font fallback repair, not complete
browser pixel parity.

## Responsive README image and Chinese font follow-up

On 2026-10-06 the Chinese README's `width="100%"` hero left roughly 430
logical pixels before the next heading in the native window. The same image
on GitHub measured 847 by 574.72 pixels, with 53.71 pixels to the next heading.
The image is intrinsically 1459 by 990 pixels. GPUI filled `height:auto` with
the intrinsic height while painting the percentage-width image at a smaller
size, leaving the unused image layout height below the visible screenshot.

The component now resolves percentage width and intrinsic aspect ratio on
the existing image wrapper, and fits the visible playback child into that
area. Explicit-height, loading/error and other image paths retain their
behavior. A public TextView regression uses the README HTML structure and
checks both lower and upper heading bounds for 100% and 50% images at 400
and 600 pixel content widths. This catches both the original excess height
and an image collapsed to zero height. The original image path failed the
upper bound; an intermediate nested wrapper failed the lower bound.

GitHub's Chinese paragraph computes to 16px/400, with h2 at 24px/600 and
strong text at 600. Those numeric weights already matched ctty7. Native
DirectWrite shaping instead selected Microsoft YaHei for Chinese because
the theme placed it before Noto Sans SC. In Windows Chrome, a raster check
of the same Chinese phrase at 400 and 600 produced different pixels for
Microsoft YaHei and the actual GitHub CSS stack; the Noto Sans SC CSS family
matched the latter. Loading a local Noto Sans SC face also confirmed its
availability and the ordinary-weight raster match.

The GitHub theme now prefers Noto Sans SC in its native CJK additions, with
PingFang SC and Microsoft YaHei retained as alternatives. The Latin stack
and 400/600 weights are unchanged. The opt-in native Windows regression
`native_github_cjk_uses_noto_at_body_and_heading_weights` shapes actual CJK
runs through the builtin stack at both weights and requires Noto Sans SC
to be installed. It failed before the reorder and passes after it:

```powershell
cargo test -p tty7 --bin tty7-app --locked native_github_cjk_uses_noto_at_body_and_heading_weights -j 1 -- --ignored
```

These measurements explain the reported heavier Chinese appearance; they
do not establish identical GPUI/Chrome rasterization across font installs,
platforms or DPI settings. No font is downloaded or bundled by this change.

Final native inspection used `tty7-app-readme-layout-check.exe`, built from
the updated component pin and theme with an isolated test configuration.
The Chinese README's loaded hero ended near y650 and the next heading
started near y694, approximately 44 logical pixels apart instead of the
former 430. The source/reading round trip preserved this layout and reading
position. The supplied pink typing title and remote badges loaded normally.
Evidence: `readme-gap-cjk-fixed.png` and `readme-gap-after-mode-switch.png`
in `.humanlayer/tasks/theme-spec-native-verification`.
Restoring the split view also retained an approximately 44px gap and correct
image proportions; the pink typing title was observed in partial frames.

The updated component passed 301 workspace tests with 2 doc tests ignored.
Combined main passed 2,960 tests plus 13 CLI end-to-end cases, with 8 ignored;
the new native font test also passed when run explicitly. An initial run
missed Git's `usr/bin` in PATH and four child-process tests could not find
`cat`/`sleep`; the complete rerun with those test dependencies passed.

The component workspace passed 301 tests, with 2 documentation tests ignored.
Combined local main passed 2,960 tests plus 13 CLI end-to-end cases, with
8 ignored. The isolated PR branch passed 2,947 tests plus the same 13 CLI
end-to-end cases, with 8 ignored. The native CJK test passed explicitly on
Windows, and the rebuilt combined application supplied the final screenshots.
Formatting of touched component files, application formatting and the
host-boundary checks passed. The PR remains a draft for the outstanding
platform/DPI matrix and complete browser parity.
