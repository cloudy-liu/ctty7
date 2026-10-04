# Editor search highlights

The current search result must be distinguishable from other matches, including
when the application's selection or accent color is gray. The reported case
showed `5/16` while every `icon` match had the same `#434750` background.

The code editor supplies independent search colors through
`gpui-component::input::CodeEditorStyle`. Ordinary matches use a neutral fill;
the current match uses the active application accent. Both fills reuse the
terminal's theme-dependent contrast targets and are opaque, so painting the
current match over an ordinary match does not change its final color.
The application's text selection, caret and syntax palette are preserved.

The component and asset packages are pinned together to
`cloudy-liu/gpui-component@833b0e084a12a3a0561d1e844b307e120cb37123`.
This existing component revision supplies the instance color interface.

## Automated regression coverage

```powershell
cargo test --locked --bin tty7-app editor_search_highlights -- --nocapture
cargo check --locked
cargo test --locked
cargo fmt --all -- --check
```

The regression tests fail with the previous saturation-only colors. They check
the reported dark gray selection with colored, gray and background-colored
accents, plus every built-in application theme. They require at least 1.2:1
luminance contrast between the two match fills, at least 1.89:1 between the
current match and the editor background, and at least 2.95:1 between the
theme's body text and each fill. These are regression bounds, not a claim that
all syntax token colors meet a text accessibility standard.

## Manual verification

Open a source file containing several occurrences of `icon`, then press
`Ctrl+F` on Windows or Linux, or `Cmd+F` on macOS. Search for `icon` and move
forward and backward through the results using Enter, Shift+Enter and the
search panel arrows. Check that one match uses the accent fill, its position
follows the counter, and the remaining matches use the neutral fill.

Repeat in a light theme and a dark theme, switch themes with the search open,
and select some source text to check its usual selection color. Automated
color tests do not replace this native window inspection, especially with
custom image or gradient backgrounds.
