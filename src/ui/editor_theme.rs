//! Source editor colors are independent of the terminal and Markdown reader.

use crate::core::config::Config;
use crate::ui::{app::Tty7App, i18n::L10nKey};
use gpui::{App, Context, rgb};
use gpui_component::{ActiveTheme as _, ThemeMode, input::CodeEditorStyle};
use std::sync::{Arc, OnceLock};

pub(crate) const CHOICES: [(&str, L10nKey); 3] = [
    ("auto", L10nKey::SettingsEditorThemeAuto),
    ("atom_one_dark", L10nKey::SettingsEditorThemeDark),
    ("atom_one_light", L10nKey::SettingsEditorThemeLight),
];

pub(crate) fn known(id: &str) -> bool {
    CHOICES.iter().any(|(choice, _)| *choice == id)
}

pub(crate) fn label(id: &str) -> L10nKey {
    CHOICES
        .iter()
        .find(|(choice, _)| *choice == id)
        .map(|(_, label)| *label)
        .unwrap_or(L10nKey::SettingsEditorThemeAuto)
}

struct Palettes {
    dark: Arc<CodeEditorStyle>,
    light: Arc<CodeEditorStyle>,
    rust_dark: Arc<CodeEditorStyle>,
    rust_light: Arc<CodeEditorStyle>,
}

fn palettes() -> &'static Palettes {
    static PALETTES: OnceLock<Palettes> = OnceLock::new();
    PALETTES.get_or_init(|| {
        let dark: CodeEditorStyle = serde_json::from_str(include_str!(
            "../../assets/editor-themes/atom-one-dark.json"
        ))
        .expect("bundled Atom One Dark palette");
        let light: CodeEditorStyle = serde_json::from_str(include_str!(
            "../../assets/editor-themes/atom-one-light.json"
        ))
        .expect("bundled Atom One Light palette");
        let rust = |base: &CodeEditorStyle, color| {
            let mut style = base.clone();
            let highlight = Arc::make_mut(&mut style.highlight_theme);
            highlight.name.push_str(" (Rust)");
            highlight.style.syntax.type_ = Some(gpui::Hsla::from(rgb(color)).into());
            Arc::new(style)
        };
        Palettes {
            rust_dark: rust(&dark, 0x56b6c2),
            rust_light: rust(&light, 0x0184bc),
            dark: Arc::new(dark),
            light: Arc::new(light),
        }
    })
}

pub(crate) fn current(language: &str, cx: &App) -> Arc<CodeEditorStyle> {
    let dark = match cx.global::<Config>().editor_theme.as_str() {
        "atom_one_dark" => true,
        "atom_one_light" => false,
        _ => cx.theme().mode == ThemeMode::Dark,
    };
    let palettes = palettes();
    match (dark, language == "rust") {
        (true, true) => palettes.rust_dark.clone(),
        (false, true) => palettes.rust_light.clone(),
        (true, false) => palettes.dark.clone(),
        (false, false) => palettes.light.clone(),
    }
}

pub(crate) fn language(language: &'static str) -> &'static str {
    if language != "rust" {
        return language;
    }
    static REGISTERED: OnceLock<()> = OnceLock::new();
    REGISTERED.get_or_init(|| {
        let registry = gpui_component::highlighter::LanguageRegistry::singleton();
        let mut config = registry.language("rust").expect("bundled Rust grammar");
        config.name = "tty7-rust".into();
        config.injections = config
            .injections
            .replace("\"rust\"", "\"tty7-rust\"")
            .into();
        for language in &mut config.injection_languages {
            if language == "rust" {
                *language = "tty7-rust".into();
            }
        }
        config.highlights = format!(
            "{}\n{}",
            config.highlights.replace("@escape", "@string.escape"),
            include_str!("../../assets/editor-themes/rust-highlights.scm"),
        )
        .into();
        registry.register("tty7-rust", &config);
    });
    "tty7-rust"
}

impl Tty7App {
    pub(crate) fn set_editor_theme(&mut self, id: &str, cx: &mut Context<Self>) {
        if known(id) {
            self.update_config(cx, |config| config.editor_theme = id.to_owned());
            cx.refresh_windows();
        }
    }
}
