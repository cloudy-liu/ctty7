"""Regenerate the bundled Atom One palettes from the pinned MIT sources.

Run from the repository root. TextMate scopes are selected explicitly: rule
names are not identifiers and can be duplicated in the upstream theme.
The editor-only queries namespace captures so injected languages keep their
own overrides instead of inheriting the containing document's colors.
"""

import json
import hashlib
from pathlib import Path
from urllib.request import urlopen

DEST = Path(__file__).resolve().parent.parent / "assets" / "editor-themes"
SOURCES = {
    "dark": ("onedark", "a8be970644982221f9b61fb1c4b3da74b4beab79", "OneDark"),
    "light": ("onelight", "5866e900db932d580e978a58db42f65cde07998b", "OneLight"),
}
FINGERPRINTS = {
    "dark": "53dbe439275f8b382dd9aabc1cc72a5447ed71b8aa47ab370b5dde50e795df7b",
    "light": "deb60ce3c0c7d084d96205b70c166a0449695f63f12afcc84de37cade650a4cf",
}

# These are Tree-sitter roles, paired with authored TextMate scopes.
ROLES = {
    "keyword": "keyword", "operator": "keyword.operator",
    "function": "entity.name.function", "function.builtin": "support.function",
    "function.method.builtin": "support.function", "constructor": "entity.name.function",
    "type": "entity.name.type", "type.builtin": "support.type",
    "enum": "entity.name.type", "variant": "entity.name.type",
    "variable": "variable", "variable.special": "variable",
    "variable.builtin": "variable", "variable.member": "variable",
    "variable.parameter": "variable.parameter", "property": "variable",
    "constant": "constant", "number": "constant.numeric", "boolean": "constant",
    "string": "string", "string.escape": "constant.character.escape",
    "escape": "constant.character.escape", "string.regex": "string.regexp",
    "string.special.regex": "string.regexp", "string.special": "string",
    "string.special.symbol": "constant.other.symbol", "comment": "comment",
    "comment.doc": "comment", "tag": "entity.name.tag", "tag.doctype": "keyword",
    "attribute": "entity.other.attribute-name", "preproc": "keyword", "label": "constant",
    "link_text": "entity.name.function", "link_uri": "string.other.link",
    "title": "entity.name.section", "text.literal": "string", "text.code.span": "string",
    "embedded": "variable.interpolation", "emphasis": "markup.italic",
    "emphasis.strong": "markup.bold", "primary": "$foreground",
    "punctuation": "$foreground", "namespace": "$foreground", "module": "$foreground",
    # Capture vocabulary used by the bundled Scala, SQL, Make and Zig queries.
    "conditional": "keyword", "exception": "keyword", "include": "keyword",
    "repeat": "keyword", "storageclass": "storage", "float": "constant.numeric",
    "character": "string", "parameter": "variable.parameter", "field": "variable",
    "method": "entity.name.function", "import": "keyword", "cImport": "keyword",
    "delimiter": "$foreground", "text.uri": "string.other.link",
    "text.reference": "entity.name.function", "none": "$foreground",
}
OVERRIDES = {
    "python": {
        "variable": "$foreground", "constant": "source.python constant.other",
        "constant.builtin": "source.python constant", "property": "$foreground",
        "constructor": "$foreground", "variable.parameter": "source.python variable.parameter",
        "variable.builtin": "support.variable.magic.python", "type.parameter": "meta.function.parameters.python",
    },
    "javascript": {
        "variable": "variable.other.readwrite.js", "variable.parameter": "variable.parameter",
        "operator": "source.js keyword.operator", "keyword.operator.word": "keyword",
        "punctuation.key_value": "punctuation.separator.key-value.js",
        "variable.constant": "variable.other.constant.js", "variable.builtin": "source.js support.variable",
        "string.special": "string.regexp", "keyword.default": "keyword.control.default.js",
    },
    "typescript": {
        "variable": "variable.other.readwrite.ts", "variable.parameter": "variable.parameter",
        "operator": "source.ts keyword.operator", "keyword.operator.word": "keyword",
        "punctuation.key_value": "punctuation.separator.key-value.ts",
        "variable.constant": "variable.other.constant.ts", "variable.builtin": "source.ts support.variable",
        "string.special": "string.regexp", "keyword.default": "keyword.control.default.ts",
    },
    "json": {"boolean": "constant.language.json", "constant.builtin": "constant.language.json"},
    "rust": {
        "type": "entity.name.type.rust", "type.builtin": "storage.type.core.rust",
        "variable.parameter": "variable", "label": "entity.name.lifetime.rust",
        "attribute": "meta.attribute.rust",
    },
    "css": {
        "property": "support.type.property-name", "type": "keyword.other.unit",
        "string.special": "constant.other.color", "attribute.id": "entity.other.attribute-name.id",
        "constant.color": "support.constant", "constant.builtin": "support.constant.property-value.css",
    },
    "java": {"type.builtin": "storage.type.primitive", "namespace": "source.java storage.modifier.import", "variable.argument": "$foreground"},
    "c": {"variable": "$foreground", "property": "$foreground", "type.builtin": "storage.type.primitive", "operator": "source.c keyword.operator"},
    "cpp": {"variable": "$foreground", "property": "$foreground", "type.builtin": "storage.type.primitive", "operator": "source.cpp keyword.operator"},
    "ruby": {"constructor": "entity.name.type", "variable": "$foreground", "variable.definition": "variable"},
    "go": {"namespace": "entity.name.type"},
    "toml": {"type": "support.type.property-name", "property": "support.type.property-name", "operator": "$foreground"},
    "make": {"string": "$foreground", "operator": "$foreground", "constant": "variable", "constant.macro": "entity.name.function"},
    "markdown": {"title": "entity.name.section.markdown", "title.setext": "markup.heading.setext", "punctuation.heading": "punctuation.definition.heading.markdown", "text.literal": "markup.raw.block.markdown"},
    "markdown_inline": {"link_uri": "markup.underline.link.markdown"},
    "elixir": {
        "constant": "source.elixir constant.language", "constant.builtin": "source.elixir constant.language",
        "number": "source.elixir constant.numeric", "operator": "source.elixir keyword.operator",
        "module": "entity.name.type", "variable.definition": "source.elixir variable.definition",
    },
    "sql": {"variable": "$foreground", "field": "$foreground", "parameter": "$foreground", "type": "$foreground"},
    "diff": {"constant": "$foreground", "attribute": "$foreground", "keyword": "$foreground",
             "string": "$foreground", "variable.builtin": "$foreground",
             "markup.inserted": "markup.inserted", "markup.deleted": "markup.deleted"},
}
OVERRIDES["tsx"] = OVERRIDES["typescript"]


def scopes(rule):
    value = rule.get("scope", [])
    return [s.strip() for s in value.split(",")] if isinstance(value, str) else value


def theme_style(data, scope):
    if scope == "$foreground":
        return {"color": data["colors"]["editor.foreground"]}
    matches = [r["settings"] for r in data["tokenColors"] if scope in scopes(r)]
    assert matches, scope
    value = matches[-1]
    style = {"color": value["foreground"]}
    if value.get("fontStyle") == "italic":
        style["font_style"] = "italic"
    if value.get("fontStyle") == "bold":
        style["font_weight"] = 700
    return style


def generate(mode, data):
    colors = data["colors"]
    syntax = {capture: theme_style(data, scope) for capture, scope in ROLES.items()}
    for capture in ["punctuation.bracket", "punctuation.delimiter", "punctuation.special", "punctuation.list_marker"]:
        syntax[capture] = theme_style(data, "$foreground")
    # Each editor query uses these exact qualified names, including injections.
    for language, overrides in OVERRIDES.items():
        for capture, scope in overrides.items():
            syntax[f"{language}.{capture}"] = theme_style(data, scope)
    style = {
        "editor.background": colors["editor.background"], "editor.foreground": colors["editor.foreground"],
        "editor.gutter.background": colors["editor.background"],
        "editor.active_line.background": colors["editor.lineHighlightBackground"],
        "editor.line_number": colors["editorLineNumber.foreground"],
        "editor.active_line_number": colors["editorLineNumber.activeForeground"],
        "editor.invisible": colors["editorWhitespace.foreground"], "syntax": syntax,
    }
    for status, role in [("error", "variable"), ("warning", "type"), ("info", "function"), ("success", "string"), ("hint", "comment")]:
        color = syntax[role]["color"]
        style.update({status: color, status + ".border": color, status + ".background": color + "26"})
    return {
        "highlight_theme": {"name": f"Atom One {mode.title()}", "appearance": mode, "style": style},
        "selection": colors["editor.selectionBackground"], "caret": colors["editorCursor.foreground"],
        "search_match": colors["editor.findMatchHighlightBackground"], "search_match_active": colors["editor.selectionBackground"],
        "muted_foreground": colors["editorLineNumber.foreground"], "border": colors["editorIndentGuide.background"],
    }


def main():
    upstream = DEST / "upstream"
    upstream.mkdir(exist_ok=True)
    for mode, (repo, commit, filename) in SOURCES.items():
        source = upstream / f"{filename}.json"
        if not source.exists():
            source.write_bytes(urlopen(f"https://raw.githubusercontent.com/akamud/vscode-theme-{repo}/{commit}/themes/{filename}.json").read())
        data = json.loads(source.read_text(encoding="utf-8"))
        fingerprint = hashlib.sha256(json.dumps(data, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
        if fingerprint != FINGERPRINTS[mode]:
            raise ValueError(f"{source} does not match the pinned upstream theme")
        (DEST / f"atom-one-{mode}.json").write_bytes((json.dumps(generate(mode, data), indent=2) + "\n").encode("utf-8"))
        print(f"{mode}: retained {len(data['tokenColors'])} upstream rules; generated explicit scope mappings")


if __name__ == "__main__":
    main()
