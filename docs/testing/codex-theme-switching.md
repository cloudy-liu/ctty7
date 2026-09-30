# Codex live theme switching

Codex 0.159.2 caches the terminal colors it queries at startup. Repainting the
TUI does not refresh that cache. Its composer, submitted prompts and built-in
diff fills can therefore retain a light palette after tty7 switches to dark,
or a dark palette after switching to light.

tty7 resolves the known RGB fills while painting an identified Codex pane.
It derives prompt fills from registered terminal themes and resolves built-in
diff fills for the active background. Text contrast is checked on these fills,
including dim placeholders and explicit syntax colors. The terminal grid stays
unchanged, so repeated switches and scrollback use the original output.

## Automated checks

```sh
cargo test --locked -p tty7 --bin tty7-app codex_
```

`codex_cached_surfaces_follow_live_theme_round_trips` replays the reported
pane's SGR colors, then applies dark, light and dark themes to the same grid.
It checks composer/history backgrounds, added/deleted diff lines and gutters,
dim text, syntax text and the original grid contents. It uses the production
theme application and per-agent paint-color resolution.

The other adapter tests cover custom backgrounds, switches within dark mode,
unrecognized RGB colors, indexed fills, other applications, inverse cells and
concealed text.

## Manual check

1. Start Codex in a light terminal theme. Keep the session running.
2. Submit a prompt and display a diff containing additions and deletions.
3. Switch tty7 to One Dark Pro. Check the composer, submitted prompt, both diff
   fills and gutters, placeholder, syntax text and cursor.
4. Switch back to light, then dark again. Scroll to older prompts and diffs.
5. Repeat with Codex started in dark mode and inside a host application.
6. Check another application's RGB output in a separate pane.

## Scope

This is an RGB compatibility adapter for the known Codex palette, not a refresh
of Codex's internal cache. Unknown/custom Codex fills and indexed backgrounds
stay literal. A source terminal theme must still be registered to recognize
its derived prompt fills; black and white fallbacks are always recognized.
Exact matching RGB fills in other content within a Codex pane also match the
adapter. Future Codex palette changes need new captured fixtures.
