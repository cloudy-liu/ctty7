# GitHub Markdown theme verification

The implementation follows [cloudy-liu/ctty7#85](https://github.com/cloudy-liu/ctty7/issues/85).
GitHub theme values are compared against the committed github-markdown-css 5.9.0
light and dark snapshots. Primer selection/control tokens and Octicons sources
are pinned in `assets/markdown-themes/GITHUB-NOTICE` with their MIT licenses.

## Automated checks

- Theme tests cover CSS palette and typography values, validation, reserved IDs,
  the GitHub default and fallback, and preservation of saved Paperglow choices.
- The Paperglow regression compares complete reading styles and syntax themes
  with the pre-change mapping at scales 100%, 150% and 200%, in both modes. It
  also checks the existing Blue Paper package.
- GPUI reading tests cover public pointer selection across body/code/kbd,
  copy text, theme reflow and diff foreground/background spans with legacy
  fallback colors.
- Application reading tests cover theme reload, mode changes, buffer/edit state,
  selection and restoration of the saved content anchor after typography changes.

Use `cargo test --workspace --locked` for the application and
`cargo test -p gpui-component --lib --features tree-sitter-languages` for the
component. The application pins both component crates to the same Git revision.

## Visual comparison

Use [the acceptance sample](../examples/markdown-github-theme.md) unchanged in
ctty7 and on its GitHub file page. Compare the rendered Markdown area, excluding
application chrome. Repeat Light default and Dark default at wide and narrow
reading widths, then Windows display scale 100% and 150%.

Check all six headings, h1/h2 rules, inline code and heading code, kbd keys,
five alerts, nested lists, task states, narrow/overflowing tables, a horizontal
rule, code and diff. Hover a link and switch between GitHub and Paperglow while
partially selected and scrolled. Return to the editor and verify unsaved text.

Color values and layout dimensions must match the pinned sources. Native font
rasterization and tree-sitter versus TextMate token classification are accepted
differences. Mermaid uses the documented Primer palette roles for this renderer.

The native/browser screenshot matrix and macOS/Linux visual checks have not yet
been verified. This record does not claim pixel-level visual acceptance.
