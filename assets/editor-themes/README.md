# Atom One editor palettes

The bundled palettes adapt the MIT-licensed Atom One themes by Mahmoud Ali:

- [Atom One Dark 2.3.0](https://github.com/akamud/vscode-theme-onedark/tree/a8be970644982221f9b61fb1c4b3da74b4beab79), license in `LICENSE-dark`.
- [Atom One Light 2.3.0](https://github.com/akamud/vscode-theme-onelight/tree/5866e900db932d580e978a58db42f65cde07998b), license in `LICENSE-light`.

The complete upstream JSON files are retained under `upstream/`, including
each theme's 74 UI colors and 211 ordered TextMate rules. Regenerate the
editor palettes from the repository root with:

```powershell
python scripts/sync-editor-themes.py
```

The script checks a SHA-256 fingerprint of the parsed source before generating
colors. With the bundled sources present, regeneration needs no network.
Mappings select explicit TextMate scopes, preserving the last matching rule;
upstream rule names are not unique and must not be used as identifiers.
Language-qualified Tree-sitter captures preserve overrides such as ordinary
Python/JavaScript identifiers using the foreground, JSON literals using cyan,
and Rust types using cyan. Exact subroles precede generic capture fallbacks.

The palettes retain the authored editor backgrounds, text, line numbers,
selections, cursor, whitespace, and syntax colors. Auxiliary diagnostic colors
use the palette's red, gold, blue, green, and comment roles. Their backgrounds
use a 15% tint. These are tty7 adaptations, not additional upstream theme keys.
Fold controls and scrollbars use the editor's muted text, selection, background,
and border colors. Their colors remain local when app and editor modes differ.

The JSON retains the upstream search colors as source data. At runtime, search
fills use the resolved editor background and foreground with tty7's contrast
targets. Ordinary matches are neutral; the current match uses the application
accent with a contrast fallback. Both fills are opaque and keep the search
visibility fix from cloudy-liu/ctty7#87 when app and editor appearances differ.

Editor-only query additions distinguish language-specific roles. Every bundled
language receives a private registry identifier, including embedded JavaScript,
JSX/TSX, Rust macros and canonical-language Markdown fences. The Markdown reader
keeps its existing registrations. Complete base queries are composed for TSX
and C++, and incompatible Kotlin patterns are removed. Local-binding captures
used for reference tracking do not participate in visual capture precedence.

The [reference corpus](tests/README.md) compares actual highlighter ranges
against TextMate colors in 21 languages, in both appearances. All 38 bundled
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
