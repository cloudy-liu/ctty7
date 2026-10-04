# Adaptive Atom One source editor appearance

Published spec: [cloudy-liu/ctty7#84](https://github.com/cloudy-liu/ctty7/issues/84).
Implementation PR: [cloudy-liu/ctty7#86](https://github.com/cloudy-liu/ctty7/pull/86).

## Problem Statement

Users opening source files from the sidebar need consistent, readable code in
both light and dark application appearances. Separate Atom One Dark and One
Dark Pro editor choices make similar colors confusing, while fixed editor modes
can disagree with the application. Users also need to adjust source typography
without losing it when changing application themes.

## Solution

Use exactly two complete source editor palettes: Atom One Dark for every resolved
dark application appearance, and Atom One Light for every resolved light
appearance. The One Dark Pro application preset uses Atom One Dark in the editor.
The editor owns its opaque background, syntax and control colors. Application
themes determine which variant is used, rather than supplying the editor's
background. Show the active palette in Appearance without a separate theme picker.
Keep source font, font size and line height as independent saved preferences.

## User Stories

1. As a code reader, I want every dark application theme to use Atom One Dark in the source editor, so that code colors remain familiar.
2. As a light-theme user, I want every light application theme to use Atom One Light, so that code remains readable.
3. As a One Dark Pro application user, I want Atom One Dark in the source editor, so that I do not have to choose between similar dark editor palettes.
4. As a settings user, I want editor colors to adapt automatically, so that I do not configure a second theme preference.
5. As a settings user, I want to see the current editor palette, so that the automatic behavior is clear.
6. As a code reader, I want the dark editor background to be #282C34 and the light editor background to be #FAFAFA, so that syntax colors retain their intended contrast.
7. As a user switching between dark application themes, I want the editor background and syntax colors to stay consistent, so that only the surrounding appearance changes.
8. As a custom-theme user, I want the application's resolved light or dark appearance to determine the editor variant, so that a theme name does not control the choice.
9. As a system-theme user, I want the editor to follow the application's system appearance changes, so that automatic appearance stays consistent.
10. As a user who disables system following, I want the editor to follow my selected application appearance, so that the system cannot change it independently.
11. As a user previewing an application theme, I want the editor to reflect the preview, so that I can assess the complete appearance.
12. As a user canceling a theme preview, I want the editor to restore its previous variant, so that cancellation is reversible.
13. As a user with wallpaper, gradients or transparency, I want an opaque editor background, so that the surrounding window cannot change code contrast.
14. As a source editor user, I want the gutter and empty editor area to use the same background as the source, so that the editor forms one complete reading surface.
15. As an existing user, I want old editor_theme values to load safely and stop overriding automatic appearance, so that upgrading needs no manual configuration edit.
16. As a user editing configuration directly, I want obsolete or malformed editor_theme values to leave other settings intact, so that compatibility does not reset my preferences.
17. As a user saving settings, I want the retired editor_theme key omitted from new configuration writes, so that the configuration reflects the supported behavior.
18. As a user with several files or windows, I want all source editors to follow the resolved application appearance, so that files do not retain stale palettes.
19. As a user opening local or remote files, I want the same source appearance, so that file location does not change readability.
20. As a user with unsaved edits, I want theme changes to preserve text and dirty state, so that appearance cannot lose or save my work.
21. As an editor user, I want selection, cursor, scroll and undo history to survive appearance changes, so that I can continue editing.
22. As a user of docked and filled document columns, I want both layouts to use the same editor palette and complete background, so that changing layout does not change colors.
23. As a user selecting or searching code, I want readable selections, caret and search markers in both variants, so that navigation stays visible.
24. As a Rust reader, I want bindings, functions, types, strings, comments and module paths to retain deliberate color relationships, so that syntax structure is easy to identify.
25. As a reader of other supported languages or embedded code, I want the existing Atom One mappings retained, so that normalization does not reduce highlighting coverage.
26. As a reader of unknown or incomplete source, I want readable foreground text and normal editing, so that unsupported syntax does not break the editor.
27. As a Markdown author, I want source mode to use Atom One and reading mode to retain its own theme, so that each view keeps its appropriate appearance.
28. As a terminal, file-tree or diff user, I want those views to retain their application appearance behavior, so that source styling remains local.
29. As a user of ordinary input fields, I want source editor styling kept out of them, so that search and settings remain consistent.
30. As a settings user, I want source font, font size and line height saved independently, so that changing colors does not reset typography.
31. As a typography user, I want the platform default, terminal font or an installed font family, so that I can choose source text independently.
32. As a settings user, I want direct numeric input, steppers and reset controls for source typography, so that changes are convenient and predictable.
33. As a keyboard user, I want font-size shortcuts to adjust focused source text and otherwise retain terminal behavior, so that focus determines the target.
34. As an existing user, I want 13 px source text and the platform monospace font by default, so that removing a color preference does not change text layout.
35. As a multilingual user, I want translated explanations of automatic editor colors and independent typography, so that the settings describe the actual behavior.
36. As a maintainer, I want only the two pinned Atom palettes, licenses and generation rules, so that no obsolete One Dark Pro editor pipeline remains.
37. As a maintainer, I want tests of actual editor behavior and compatibility, so that verification detects incorrect colors or lost edits rather than private implementation changes.

## Implementation Decisions

1. The source appearance module resolves a complete Atom One editor style solely from the application's effective light or dark mode. Do not match application preset names or introduce another system appearance listener.
2. Remove the separate editor color picker and fixed-mode actions. Retain a read-only current-palette explanation in Appearance, together with independent typography controls.
3. Retire the editor_theme configuration field. Existing JSON keys, including auto, atom_one_dark, atom_one_light, one_dark_pro, unknown strings and malformed values, are ignored through normal unknown-field compatibility. New writes omit the key and preserve other valid settings.
4. Atom One Dark owns background #282C34; Atom One Light owns background #FAFAFA. Apply each variant's complete syntax, gutter, selection, caret, current-line and existing auxiliary colors as one instance style. Source and gutter backgrounds are opaque, including empty space.
5. Keep the existing search contrast calculation from cloudy-liu/ctty7#87. Ordinary matches use the editor contrast budget, and the current match retains the application accent with its existing contrast fallback.
6. Remove the separate One Dark Pro editor assets, generation rules, fixtures and labels. Keep the One Dark Pro application preset and its application/terminal behavior.
7. Retain pinned Atom sources, attribution, exact capture overrides and private language queries. Tree-sitter remains the syntax engine; it does not implement a general TextMate or semantic-token runtime.
8. Preserve editor state and file identity when updating styles. Theme changes do not reopen files, replace input state or create network requests.
9. Keep editor_font_family, editor_font_size and editor_line_height independent of palette data. Font size supports 8 to 72 px with a 13 px default; line height supports 1 to 3 times the text size with a 1.5 default.
10. Typography numeric controls submit on Enter or blur, retain in-progress input during unrelated refreshes, and safely handle malformed or bounded values. Focused source font-size shortcuts do not change terminal text size.
11. Use the existing fixed component revision and instance-style contract. Terminal, ordinary inputs, diffs and Markdown reading views retain their existing styling and language registrations.

## Testing Decisions

1. Prefer existing GPUI application-window tests as the primary boundary: open source, change application themes or previews, render the window, and observe colors, typography and editing state. Reuse existing source/Markdown editing and preview-cancellation tests.
2. Use existing configuration JSON read/write tests to verify retired editor_theme values are ignored, omitted on save, and do not discard typography or unrelated preferences. No new production test interface is needed.
3. Cover One Dark Pro, Dracula, Nord, other built-ins and resolved custom light/dark appearances. Preview and cancellation must restore the correct variant. Test disabled system following through the existing application appearance contract.
4. Assert source background, gutter and representative syntax/control colors against independent authored Atom values. Use the existing TextMate reference corpus for supported-language and embedded-code regressions, and retain compilation/isolation checks for all bundled queries.
5. Use the existing real-window settings tests to verify the color picker is absent, the active-palette explanation is visible, and typography inputs still submit, persist and reset correctly.
6. Verify unsaved text, dirty state, selection, cursor, scroll and undo survive light/dark changes. Retain existing document-layout and consumer-isolation checks.
7. Run targeted configuration, settings and source-editor tests during implementation, locked workspace tests once after the final change, formatting, whitespace, host-boundary checks and a Windows build. Record what actually ran.
8. Native application screenshots and other-platform visual checks are separate from GPUI headless tests. Record unperformed visual acceptance explicitly; do not count HTML demonstrations or earlier screenshots as native acceptance.

## Out of Scope

- Separate One Dark Pro, Dracula, Nord or other application-specific editor palettes.
- Fixed light/dark editor preferences, custom editor theme import and per-file palettes.
- Replacing the editor, adding a language server, semantic tokens, bracket coloring or new language support.
- Changing application theme choices, terminal palettes, Markdown reading themes or window materials.
- General layout, indentation, file-saving or unrelated editing behavior changes.
- Exact reproduction of every TextMate rule or Cursor screenshot.

## Further Notes

This revision supersedes the earlier automatic/fixed editor preference contract.
It incorporates the locally completed independent typography work, while removing
the separate One Dark Pro editor work before it is published. Application themes
choose the editor's appearance variant; the selected Atom palette owns the editor
background and other source colors. No user migration action is required.

Atom One Dark and Atom One Light are paired MIT-licensed themes from akamud.
Their pinned source data and licenses remain bundled for offline generation.
The implementation and PR target remain cloudy-liu/ctty7 with ready-for-agent
triage. Verification details and remaining limits belong in the accompanying
verification record and PR, rather than claims about unperformed checks.
