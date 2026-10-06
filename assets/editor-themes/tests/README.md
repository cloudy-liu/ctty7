# TextMate color reference

`textmate-colors.json` contains 46 small source samples and 440 selected token
color expectations: 21 languages, each evaluated with Atom One Dark and Light
2.3.0. The expected colors were produced independently from the bundled
upstream theme files using `vscode-textmate` 9.2.0 and `vscode-oniguruma` 1.7.0
with the installed Cursor language grammars. TOML uses Even Better TOML 0.21.2.
The samples are local test code; they contain no user document contents.

The languages are Python, JavaScript, TypeScript, JSON, Rust, CSS, Java, C,
C++, Go, Ruby, Bash, HTML, YAML, Markdown, Diff, SQL, Lua, Make, TOML and TSX.
Ranges identify selected tokens, including declarations, references, builtins,
properties, operators, numbers, strings, comments and markup where relevant.
This is a representative reference corpus, not an exhaustive grammar snapshot.

`editor_theme_matches_upstream_textmate_color_samples` runs the production
Tree-sitter highlighter and compares the resolved color of every non-whitespace
byte within those ranges against the reference. Separate regressions cover
embedded HTML/JavaScript, JSX, TSX, Rust Markdown fences and the Python import,
variable and magic-variable cases reported in the screenshots.

Other tests compare 11 editor/control colors for each of the two palettes directly with the
upstream JSON, compile all 38 private language queries, and verify that shared
language registrations remain unchanged. No TextMate or Node runtime is
required to build or run tty7 or these tests.

```powershell
cargo test --locked --features updater -j 2 --bin tty7-app editor_theme_
```
