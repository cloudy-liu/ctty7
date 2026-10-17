# Markdown table selection integration

The reader layout and aligned selection changes originally pinned sibling
component commits. Selecting either pin would omit the other change.

The application now pins `cloudy-liu/gpui-component` commit
`e7711b27a74afaca714b3e52652e9c386195b6c8`, directly based on the reader layout
commit `a9f601a1a632c753ae279a9c25f629d59908d87c`. It includes adaptive table
wrapping, external scrollbar ownership and aligned selection geometry. The
manifest and all three component lockfile sources use the same Git revision.

## Regression coverage

`markdown_table_text_can_be_selected_with_the_mouse` drags across the visible
number `118.3344` in right-aligned and centered table cells, in both directions.
Selection and clipboard contents must equal the complete number. Pointer
positions follow the table bounds and exclude the block's bottom gap, so the
regression also works with adaptive table layout. The drag stays clear of the
old left-aligned character bounds.

After merging the fork's current main, the Windows command
`cargo test --offline --locked --bin tty7-app ui::markdown_preview::tests -- --test-threads=1`
passes 20 tests with zero failures. One native-font test remains ignored because
the Windows test font collection does not include Noto Sans SC. The suite also
checks docked/filled reader scrollbar dragging, reaching the document end,
selection restoration, resources, Mermaid rendering and theme behavior.

Formatting, diff whitespace and the HostOps boundary check pass. This update
uses the committed Git dependency, without local path patches. Native desktop
mouse interaction was not manually repeated; pointer events and the copy
shortcut run through the GPUI application harness. Cross-platform results are
reported separately by CI.
