# Atom One editor palettes

The bundled palettes adapt the MIT-licensed Atom One themes by Mahmoud Ali:

- [Atom One Dark 2.3.0](https://github.com/akamud/vscode-theme-onedark/tree/a8be970644982221f9b61fb1c4b3da74b4beab79), license in `LICENSE-dark`.
- [Atom One Light 2.3.0](https://github.com/akamud/vscode-theme-onelight/tree/5866e900db932d580e978a58db42f65cde07998b), license in `LICENSE-light`.

The JSON files map TextMate token roles to the component's Tree-sitter roles.
They retain the authored editor backgrounds, text, line numbers, selections,
cursor, search match, whitespace, and syntax colors. Auxiliary diagnostic colors
use the palette's red, gold, blue, green, and comment roles. Their backgrounds
use a 15% tint. These are tty7 adaptations, not additional upstream theme keys.
Fold controls and scrollbars use the editor's muted text, selection, background,
and border colors. Their colors remain local when app and editor modes differ.

Rust uses the upstream Rust-specific cyan type override. Editor-only query
additions distinguish variables from module paths and map escapes to the
supported `string.escape` role. They are registered under a separate language
identifier, including Rust macro injections, so the Markdown reader keeps its
existing queries. Function and
parameter classification is syntactic; it does not reproduce language-server
semantic tokens or Cursor's bracket-pair coloring.

The themes are embedded at compile time. No theme download occurs at runtime.
