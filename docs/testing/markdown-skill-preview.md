# Skill Markdown preview verification

Verified on Windows on 2026-10-08 with the production Markdown reading view
and the native visual fixture, at 175% system DPI and 100% application scale.
The reference is [show-me's SKILL.md](https://github.com/humanlayer/skills/blob/main/plugins/show-me/skills/show-me/SKILL.md).

## Reproduction

On the unmodified preview, the opening YAML delimiter became a horizontal rule
and the closing delimiter turned the metadata into a setext heading. GitHub
instead shows the same `name`, `description`, and boolean field as table rows.
This was Markdown interpretation, not corrupted character encoding.

The downloaded LF document rendered its sequence diagram. Converting the same
document to CRLF reproduced the unrendered Mermaid code block. The shared fence
matcher did not accept the carriage return after `mermaid`, so the diagram
never reached the renderer.

## Native evidence

The light 1000-by-800 viewport now shows metadata rows and the sequence diagram.
The release build also passed the dark 700-by-800 viewport check. Diagram copy
matched the original source exactly, including all seven CRLF line endings.
Long metadata cells remain accessible through the native table's horizontal
scrolling in the narrow viewport.

![Metadata before and after](images/markdown-frontmatter-before-after.webp)

![CRLF Mermaid before and after](images/markdown-mermaid-crlf-before-after.webp)

These are cropped native captures with comparison labels added. Document text
and diagram pixels are unchanged.

## Automated checks

- Both symptom-specific regression tests failed before the fix and passed after it.
- Metadata tests cover source order, LF/CRLF, BOM, folding, multiline and nested values, escaping, invalid YAML, and incomplete delimiters.
- GPUI tests cover metadata with Mermaid enabled or disabled, unsaved edits, removal of metadata, source preservation, and theme/selection stability with CRLF diagrams.
- Invalid Mermaid syntax still falls back to a code block.
- `cargo test --workspace --locked`: 2990 passed, 8 ignored, 0 failed.
- `cargo fmt --all -- --check` and `git diff --check`: passed.
- `cargo build -p tty7 --bin tty7-app --features markdown-visual-tests --release --locked`: passed.

## Scope

Metadata uses the existing native table layout, not GitHub's browser wrapping.
Nested values display as literal YAML rather than nested browser tables.
The standalone visual fixture omits the application's dialog layer, so this
pass does not revalidate the expanded diagram viewer, zoom, or pan controls.
The eight ignored workspace tests were not run.
