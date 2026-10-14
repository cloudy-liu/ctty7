# Markdown navigation and performance

The acceptance fixture is `tests/fixtures/markdown/large-navigation.md`: 101,027
UTF-8 bytes, 1,873 lines, six generated Mermaid diagrams, tables, code blocks,
and a final `DOCUMENT_END_MARKER`. It contains real Markdown and Mermaid source.

## Required behavior

- Clicking an inline diagram opens a larger canvas. Dragging an inline diagram
  must not accidentally open it.
- Ordinary wheel input pans an expanded diagram without changing its zoom.
  Modifier-wheel input zooms around the pointer. Shift-wheel supports horizontal
  movement with a standard mouse. Unzoomed inline diagrams let the document scroll.
- Fit/reset, zoom buttons, keyboard navigation, and Escape remain usable.
- The wide diagram's `WIDE_END_24`, tall diagram's `TALL_END_36`, and long sequence's
  final message must be reachable after enlarging the diagram.
- A diagram update invalidates its containing reading view rather than forcing
  every view in the window to rerender. Expanded dragging must visibly repaint.
- Long Markdown uses the component's existing virtual list. Offscreen headings
  are addressable before being painted. Home, End, page navigation, selection,
  source editing, theme changes, and document resizing continue to work.
- Every performance comparison uses the same fixture, viewport, DPI, build
  optimization, warmup, and event sequence. Both event handling and visible
  transform/scroll changes are checked so an unchanged cached frame cannot pass.

## Evidence

Run the ignored `markdown_large_document_frame_profile` benchmark for repeatable
GPUI test-platform CPU measurements, and the optional `profile_output` mode of
the native `--markdown-visual` fixture for Windows font and image measurements.
Use three fresh native processes per revision, with 100 measured frames after
10 warmup frames for each interaction. Record median, P95, maximum, full SVG
generation readiness, binary hashes, and the exact build configuration.

These measure CPU event dispatch and GPUI layout/prepaint/paint. They exclude
presentation and GPU execution; they must not be reported as displayed FPS.
Validate scrolling, dragging, zooming and distant markers with Computer Use as
well as the automated checks. Keep original before/after screenshots on the PR.

Cursor's native diagram interaction is the product reference: its support team
describes clicking a diagram to open an expanded view with zoom and pan at
<https://forum.cursor.com/t/improve-diagramming/159547>. Its native Markdown
Preview is separate from the classic VS Code preview and its extensions:
<https://forum.cursor.com/t/improve-markdown-preview-mermaid-rendering/172321/5>.
The exact keyboard and wheel bindings above are ctty7's explicit behavior.
