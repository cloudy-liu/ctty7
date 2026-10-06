# Source editor palettes

The bundled palettes adapt these MIT-licensed themes:

- [Atom One Dark 2.3.0](https://github.com/akamud/vscode-theme-onedark/tree/a8be970644982221f9b61fb1c4b3da74b4beab79), license in `LICENSE-dark`.
- [GitHub default light website tokens](https://github.com/primer/primitives), captured on 2026-10-06, license in `LICENSE-light`.

The complete Atom One Dark JSON is retained under `upstream/`, including its
74 UI colors and 211 ordered TextMate rules. `GitHubLight.json` records resolved
tokens from GitHub's published light stylesheet, its URL, capture date and raw
stylesheet checksum. Regenerate the editor palettes from the repository root with:

```powershell
python scripts/sync-editor-themes.py
```

The script checks a SHA-256 fingerprint of the parsed source before generating
colors. With the bundled sources present, regeneration needs no network.
Dark mappings select explicit TextMate scopes, preserving the last matching rule;
upstream rule names are not unique and must not be used as identifiers.
Language-qualified Tree-sitter captures preserve overrides and embedded code.
Both palettes use the same capture vocabulary. Exact subroles precede generic
capture fallbacks.

GitHub Light uses website CodeMirror tokens for its white background, dark
foreground and caret, gray line numbers, active line and blue selection. Syntax
uses red keywords, purple functions, dark blue strings, blue constants and
brown variable/type roles. Ordinary Python and JavaScript identifiers use the
foreground. Tags, regular expressions, links and Markdown roles use the
corresponding website prettylights tokens where CodeMirror has no specific key.
These are Tree-sitter role adaptations, not a browser editor runtime.
Comments and documentation comments use the website code-display token
`prettylights-syntax-comment` (`#59636E`) to match GitHub's gray comments.
The captured CodeMirror comment token is `#1F2328`, but is not used for comments.
Functions retain the CodeMirror purple entity token; GitHub's code viewer uses
a different purple entity token.

The palettes retain the authored editor backgrounds, text, line numbers,
selections, cursor and syntax colors. GitHub Light uses the website border token
for invisible characters and borders, and the foreground for active line numbers.
Auxiliary diagnostic colors use Atom syntax roles in dark mode and GitHub's
danger, attention, accent, success and muted tokens in light mode. Their backgrounds
use a 15% tint. These are tty7 adaptations, not additional upstream theme keys.
Fold controls and scrollbars use the editor's muted text, selection, background,
and border colors. These controls use the source palette rather than application colors.

The JSON retains the upstream search colors as source data. At runtime, search
fills use the resolved editor background and foreground with tty7's contrast
targets. Ordinary matches are neutral; the current match uses the application
accent with a contrast fallback. Both fills are opaque and keep the search
visibility fix from cloudy-liu/ctty7#87 in both editor appearances.

Editor-only query additions distinguish language-specific roles. Every bundled
language receives a private registry identifier, including embedded JavaScript,
JSX/TSX, Rust macros and canonical-language Markdown fences. The Markdown reader
keeps its existing registrations. Complete base queries are composed for TSX
and C++, and incompatible Kotlin patterns are removed. Local-binding captures
used for reference tracking do not participate in visual capture precedence.

The [reference corpus](tests/README.md) compares actual highlighter ranges
against authored dark TextMate colors and light website token values in 21 languages. All 38 bundled
language queries are checked for successful compilation and isolation.
CMake, C#, GraphQL, Protocol Buffers and Swift currently ship no highlight
queries in the component; they retain readable foreground text, as does plain
text. Query compilation does not imply exhaustive color coverage.

Tree-sitter classification cannot reproduce every nested TextMate selector.
The mappings cover the tested syntax roles, rather than implementing a general
TextMate theme runtime. Language-server semantic tokens and Cursor's bracket
pair coloring remain outside this feature. Unknown Markdown fence labels use
readable foreground text; canonical labels such as `javascript` and `rust`
receive the editor-specific injected colors.

The themes are embedded at compile time. No theme download occurs at runtime.

The application's resolved light/dark appearance selects GitHub Light or
Atom One Dark for every source editor. The One Dark Pro application preset
uses Atom One Dark, just like other dark presets. Each source palette owns the
complete opaque editor and gutter background, syntax, selection and caret.
There are no independent fixed-mode editor choices. The retired `editor_theme`
configuration key is ignored on load and omitted on save.
Font family, font size and line height remain independent user preferences;
their instance-local styles do not change the component's global mono font.
