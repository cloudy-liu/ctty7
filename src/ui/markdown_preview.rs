use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, LazyLock},
};

use gpui::prelude::*;
use gpui::{
    App, BoxShadow, Context, Entity, FocusHandle, Focusable, FontWeight, Global, Hsla, Pixels,
    Render, ScrollHandle, SharedString, StyleRefinement, Subscription, Window, div, point, px,
    relative, rems,
};
use gpui_component::{
    ActiveTheme as _, IconName, Sizable as _, ThemeMode,
    button::{Button, ButtonVariants as _},
    highlighter::{
        FontWeightContent, HighlightTheme, HighlightThemeStyle, SyntaxColors, ThemeStyle,
    },
    input::InputState,
    text::{
        AlertStyle, InlineCodeStyle, KeyboardStyle, LinkUnderline, TaskCheckboxStyle, TextView,
        TextViewState, TextViewStyle,
    },
};

use crate::core::markdown_document::{self, Target};
use crate::core::{
    config::Config,
    markdown_frontmatter,
    markdown_theme::{self, Color, Entry, Registry, Snapshot, Theme},
};
use crate::ui::i18n::{L10nKey, t};
use crate::ui::{
    app::Tty7App,
    host_ops::{HostOps, SharedHost},
};
use gpui_component::{WindowExt as _, text::TextViewImageSource};

fn extensions() -> gpui_component::text::MarkdownExtensions {
    static EXTENSIONS: std::sync::OnceLock<gpui_component::text::MarkdownExtensions> =
        std::sync::OnceLock::new();
    EXTENSIONS
        .get_or_init(|| gpui_component::text::MarkdownExtensions::default().github_alerts())
        .clone()
}

impl Global for Registry {}

struct ReadingFonts(Arc<Vec<String>>);
impl Global for ReadingFonts {}

impl crate::ui::app::Tty7App {
    pub(crate) fn set_markdown_theme(&mut self, id: &str, cx: &mut Context<Self>) {
        if !cx
            .try_global::<Registry>()
            .is_some_and(|registry| registry.entries.contains_key(id))
        {
            return;
        }
        self.update_config(cx, |config| config.markdown_theme = id.to_owned());
        cx.refresh_windows();
    }
}

pub(crate) fn init(snapshot: Snapshot, cx: &mut App) {
    let mut registry = Registry::default();
    registry.replace(snapshot, &cx.global::<Config>().markdown_theme);
    cx.set_global(registry);
}

pub(crate) fn apply_snapshot(snapshot: Snapshot, cx: &mut App) {
    let selected = cx.global::<Config>().markdown_theme.clone();
    if cx.try_global::<Registry>().is_none() {
        cx.set_global(Registry::default());
    }
    cx.update_global::<Registry, _>(|registry, _| registry.replace(snapshot, &selected));
    cx.refresh_windows();
}

pub(crate) fn current(cx: &App) -> Entry {
    cx.try_global::<Registry>()
        .map(|registry| {
            registry
                .resolve(&cx.global::<Config>().markdown_theme)
                .clone()
        })
        .unwrap_or_else(|| Entry {
            theme: markdown_theme::builtin(),
            source: None,
            revision: 0,
        })
}

pub(crate) fn color(color: Color) -> Hsla {
    let [r, g, b, a] = color.0;
    gpui::Rgba { r, g, b, a }.into()
}

pub(crate) fn theme_label(entry: &Entry) -> String {
    match (entry.source.is_none(), entry.theme.id.as_str()) {
        (true, "github") => t(L10nKey::SettingsMarkdownGithub).to_owned(),
        _ => entry.theme.name.clone(),
    }
}

pub(crate) fn theme_description(entry: &Entry) -> String {
    match (entry.source.is_none(), entry.theme.id.as_str()) {
        (true, "github") => t(L10nKey::SettingsMarkdownGithubDesc).to_owned(),
        _ => entry.theme.description.clone(),
    }
}

fn refinement(mut element: gpui::Div) -> StyleRefinement {
    element.style().clone()
}

fn available_reading_font(
    fonts: &[String],
    available: &[String],
    fallback: SharedString,
) -> SharedString {
    for family in fonts {
        if cfg!(target_os = "macos") && family == ".AppleSystemUIFont" {
            return ".SystemUIFont".into();
        }
        if available
            .iter()
            .any(|name| name.eq_ignore_ascii_case(family))
        {
            return family.clone().into();
        }
    }
    fallback
}

fn rust_reading_language() -> SharedString {
    static LANGUAGE: std::sync::OnceLock<SharedString> = std::sync::OnceLock::new();
    LANGUAGE
        .get_or_init(|| {
            let registry = gpui_component::highlighter::LanguageRegistry::singleton();
            let mut config = registry.language("rust").expect("bundled Rust grammar");
            config.name = "ctty7-markdown-rust".into();
            let captures = regex::Regex::new(r"@type(?:\.[A-Za-z0-9_.-]+)?\b").unwrap();
            config.highlights = format!(
                "(reference_type \"&\" @github.rust.reference)\n{}",
                captures.replace_all(&config.highlights, "@github.rust.type")
            )
            .into();
            registry.register("ctty7-markdown-rust", &config);
            config.name
        })
        .clone()
}

fn highlight(theme: &Theme, dark: bool, revision: u64) -> Arc<HighlightTheme> {
    let palette = theme.palette(dark);
    let s = &palette.syntax;
    let token = |c| Some(ThemeStyle::from(color(c)));
    Arc::new(HighlightTheme {
        name: format!("{}:{}:{}", theme.id, dark, revision),
        appearance: if dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        },
        style: HighlightThemeStyle {
            editor_background: Some(color(palette.code_background)),
            editor_foreground: Some(color(palette.code_foreground)),
            syntax: SyntaxColors {
                title: s
                    .markup_heading
                    .map(|c| ThemeStyle::from(color(c)).weight(FontWeightContent::Bold)),
                punctuation_list_marker: s.markup_list.and_then(token),
                link_uri: s.link_uri.map(|c| ThemeStyle::from(color(c)).underline()),
                diff_added: s.diff_added.as_ref().map(|d| {
                    ThemeStyle::from(color(d.foreground)).background(d.background.map(color))
                }),
                diff_deleted: s.diff_deleted.as_ref().map(|d| {
                    ThemeStyle::from(color(d.foreground)).background(d.background.map(color))
                }),
                diff_changed: s.diff_changed.as_ref().map(|d| {
                    ThemeStyle::from(color(d.foreground)).background(d.background.map(color))
                }),
                diff_hunk: s
                    .diff_hunk
                    .map(|c| ThemeStyle::from(color(c)).weight(FontWeightContent::Bold)),
                diff_header: token(s.constant.unwrap_or(s.number)),
                variable_builtin: s.constant.and_then(token),
                keyword: token(s.keyword),
                boolean: token(s.constant.unwrap_or(s.keyword)),
                preproc: token(s.keyword),
                number: token(s.number),
                constant: token(s.constant.unwrap_or(s.number)),
                function: token(s.entity.unwrap_or(s.function)),
                constructor: token(s.entity.unwrap_or(s.function)),
                variable: token(s.variable),
                variable_special: token(s.variable_special),
                property: token(s.constant.unwrap_or(s.variable_special)),
                type_: token(s.entity.unwrap_or(s.r#type)),
                enum_: token(s.entity.unwrap_or(s.r#type)),
                variant: token(s.entity.unwrap_or(s.r#type)),
                captures: [
                    (
                        "github.rust.type".into(),
                        ThemeStyle::from(color(
                            if s.r#type == markdown_theme::builtin().palette(dark).syntax.r#type {
                                palette.code_foreground
                            } else {
                                s.r#type
                            },
                        )),
                    ),
                    (
                        "github.rust.reference".into(),
                        ThemeStyle::from(color(s.constant.unwrap_or(s.number))),
                    ),
                ]
                .into(),
                comment: token(s.comment),
                comment_doc: token(s.comment),
                string: token(s.string),
                string_escape: s
                    .escape
                    .map(|c| ThemeStyle::from(color(c)).weight(FontWeightContent::Bold))
                    .or(token(s.string)),
                string_regex: token(s.string),
                string_special: token(s.string),
                string_special_symbol: token(s.string),
                tag: token(s.tag_name.unwrap_or(s.tag)),
                tag_doctype: token(s.tag_name.unwrap_or(s.tag)),
                attribute: token(s.entity.unwrap_or(s.variable_special)),
                operator: token(palette.code_foreground),
                punctuation: token(palette.code_foreground),
                punctuation_bracket: token(palette.code_foreground),
                punctuation_delimiter: token(palette.code_foreground),
                ..Default::default()
            },
            ..Default::default()
        },
    })
}

pub(crate) fn reading_style(
    entry: &Entry,
    dark: bool,
    scale: f32,
    mono: SharedString,
) -> TextViewStyle {
    let theme = &entry.theme;
    let p = theme.palette(dark);
    let t = &theme.typography;
    let l = &theme.layout;
    let code_font = t
        .code_fonts
        .first()
        .cloned()
        .map(SharedString::from)
        .unwrap_or(mono);
    let mut style = TextViewStyle {
        code_block_languages: [("rust".into(), rust_reading_language())].into(),
        heading_permalink_icon: Some("icons/github/link.svg".into()),
        task_checkbox: Some(TaskCheckboxStyle {
            size: px(13. * scale),
            radius: px(2. * scale),
            border: gpui::rgb(if dark { 0x858585 } else { 0x767676 }).into(),
            background: gpui::rgb(if dark { 0x3b3b3b } else { 0xffffff }).into(),
            checked_background: gpui::rgb(if dark { 0x6b6b6b } else { 0xc8c8c8 }).into(),
            foreground: gpui::rgb(if dark { 0xaaaaaa } else { 0xffffff }).into(),
        }),
        is_dark: dark,
        bold_weight: FontWeight(t.bold_weight),
        link_underline: match l.link_underline {
            markdown_theme::LinkUnderline::Always => LinkUnderline::Always,
            markdown_theme::LinkUnderline::Hover => LinkUnderline::Hover,
            markdown_theme::LinkUnderline::Never => LinkUnderline::Never,
        },
        roman_ordered_lists: l.roman_ordered_lists,
        list_indent: l.list_indent.map(|v| px(v * scale)),
        list_paragraph_gap: l.list_paragraph_gap.map(|v| px(v * scale)),
        table_fill: l.table_fill,
        table_radius: l.table_radius.map(|v| px(v * scale)),
        table_gap: l.table_gap.map(|v| px(v * scale)),
        table_stripe: p.table_stripe.map(color),
        table_row_border: p.table_row_border.map(color),
        inline_code_border: p.inline_code_border.map(color),
        inline_code: l.inline_code_radius.map(|radius| InlineCodeStyle {
            radius: px(radius * scale),
            padding_x: px(l.inline_code_padding_x * scale),
            padding_y: px(l.inline_code_padding_y * scale),
            font_size: px(t.inline_code_size.unwrap_or(t.code_size) * scale),
        }),
        heading_inline_code: std::array::from_fn(|ix| {
            let radius = l.inline_code_radius?;
            let size = t.heading_sizes[ix] * scale;
            Some(InlineCodeStyle {
                radius: px(radius * scale),
                padding_x: px(l
                    .heading_code_padding_x_em
                    .map_or(l.inline_code_padding_x * scale, |em| em * size)),
                padding_y: px(l
                    .heading_code_padding_y_em
                    .map_or(l.inline_code_padding_y * scale, |em| em * size)),
                font_size: px(size),
            })
        }),
        keyboard: t.kbd_size.map(|size| KeyboardStyle {
            background: color(p.kbd_background.unwrap_or(p.inline_code_background)),
            border: color(p.kbd_border.unwrap_or(p.border)),
            shadow: color(p.kbd_shadow.unwrap_or(p.border)),
            radius: px(l.kbd_radius * scale),
            padding: px(l.kbd_padding * scale),
            font_size: px(size * scale),
            line_height: px(l.kbd_line_height * scale),
        }),
        alerts: std::array::from_fn(|ix| {
            p.alerts.as_ref().map(|alerts| AlertStyle {
                container: refinement(
                    div()
                        .bg(color(p.alert_background))
                        .text_color(color(p.alert_foreground.unwrap_or(p.foreground)))
                        .border_color(color(alerts[ix].border))
                        .px(px(l.alert_padding_x.unwrap_or(l.quote_padding) * scale))
                        .py(px(l.alert_padding_y.unwrap_or(l.quote_padding) * scale)),
                ),
                title: refinement(
                    div()
                        .text_color(color(alerts[ix].title))
                        .font_weight(FontWeight(l.alert_title_weight))
                        .gap(px(8. * scale))
                        .pb(px(8. * scale))
                        .line_height(relative(l.alert_title_line_height.unwrap_or(t.line_height))),
                ),
                icon: alerts[ix].icon.clone().into(),
                icon_size: px(16. * scale),
            })
        }),
        horizontal_rule: if l.rule_gap.is_none() && l.rule_height == 2. {
            StyleRefinement::default()
        } else {
            refinement(
                div()
                    .h(px(l.rule_height * scale))
                    .bg(color(p.rule.unwrap_or(p.border))),
            )
        },
        horizontal_rule_container: l
            .rule_gap
            .map(|gap| refinement(div().pt(px(gap * scale)).pb(px(gap * scale))))
            .unwrap_or_default(),
        paragraph_gap: rems(t.paragraph_gap / crate::core::config::UI_FONT_SIZE_DEFAULT),
        highlight_theme: highlight(theme, dark, entry.revision),
        link_color: Some(color(p.link)),
        link_hover_color: Some(color(p.link_hover)),
        inline_code_color: Some(color(p.inline_code_foreground)),
        inline_code_background: Some(color(p.inline_code_background)),
        inline_code_font: Some(code_font.clone()),
        inline_code_fallbacks: Some(gpui::FontFallbacks::from_fonts(t.code_fonts.clone())),
        selection_color: Some(color(p.selection)),
        border_color: Some(color(p.border)),
        task_color: Some(color(p.accent)),
        task_foreground: Some(color(p.paper)),
        quote_link_color: Some(color(p.quote_link)),
        quote_code_color: Some(color(p.quote_code_foreground)),
        quote_code_background: Some(color(p.quote_code_background)),
        table_code_background: Some(color(p.table_code_background)),
        table_hover_background: p.table_hover.map(color),
        code_block: refinement(
            div()
                .p(px(l.code_padding * scale))
                .rounded(px(l.code_radius * scale))
                .bg(color(p.code_background))
                .text_color(color(p.code_foreground))
                .border(px(l.code_border_width))
                .border_color(color(p.code_border))
                .font_family(code_font)
                .text_size(px(t.code_size * scale)),
        ),
        blockquote: refinement(
            div()
                .bg(color(p.quote_background))
                .text_color(color(p.quote_foreground))
                .border_l(px(l.quote_border * scale))
                .border_color(color(p.quote_border))
                .rounded_r(px(l.quote_radius * scale))
                .p(px(l.quote_padding * scale))
                .when_some(l.quote_padding_y, |el, padding| el.py(px(padding * scale))),
        ),
        alert: refinement(
            div()
                .bg(color(p.alert_background))
                .text_color(color(p.alert_foreground.unwrap_or(p.foreground)))
                .border_color(color(p.alert_border)),
        ),
        nested_blockquote: refinement(
            div()
                .bg(color(p.nested_quote_background))
                .border_color(color(p.nested_quote_border)),
        ),
        table_cell: refinement(
            div()
                .px(px(l.table_padding_x.unwrap_or(l.table_padding) * scale))
                .py(px(l.table_padding_y.unwrap_or(l.table_padding) * scale))
                .when(l.table_fill, |el| el.border_r_0()),
        ),
        table_header: refinement(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(color(p.heading)),
        ),
        list_item: refinement(div().pb(px(l.list_gap * scale))),
        ..Default::default()
    };
    if let Some(line_height) = t.code_line_height {
        style.code_block.text.line_height = Some(relative(line_height));
    }
    style.table.overflow.x = Some(gpui::Overflow::Scroll);
    style.code_block.overflow.x = Some(gpui::Overflow::Scroll);
    style.code_block.text.font_fallbacks =
        Some(gpui::FontFallbacks::from_fonts(t.code_fonts.clone()));
    for level in 0..6 {
        style.headings[level] = refinement(
            div()
                .text_size(px(t.heading_sizes[level] * scale))
                .font_weight(FontWeight(t.heading_weights[level]))
                .line_height(relative(t.heading_line_height))
                .text_color(color(
                    p.heading_colors
                        .map(|colors| colors[level])
                        .unwrap_or(p.heading),
                ))
                .mt(px(t.heading_gap * scale))
                .when_some(t.heading_bottom_gap, |el, gap| el.mb(px(gap * scale)))
                .pb(px(t
                    .heading_padding
                    .map(|values| values[level])
                    .unwrap_or(t.paragraph_gap * 0.5)
                    * scale))
                .when(t.heading_borders[level], |el| {
                    el.border_b_1()
                        .border_color(color(p.heading_border.unwrap_or(p.border)))
                }),
        );
        if let Some(font) = t.heading_fonts.first() {
            style.headings[level].text.font_family = Some(font.clone().into());
            style.headings[level].text.font_fallbacks =
                Some(gpui::FontFallbacks::from_fonts(t.heading_fonts.clone()));
        }
    }
    style
}

/// The bundled GitHub palette uses `#f0f6fc` for dark-mode body text. That is
/// faithful to github.com, but it is brighter than tty7's surrounding dark
/// surfaces. Keep GitHub's semantic and syntax colors while borrowing the
/// application's neutral foreground for the Markdown reading surface.
fn align_dark_github_foreground(
    entry: &Entry,
    dark: bool,
    foreground: Hsla,
    style: &mut TextViewStyle,
) {
    if !dark || entry.theme.id != markdown_theme::DEFAULT_ID {
        return;
    }

    let neutral = ThemeStyle::from(foreground);
    style.code_block.text.color = Some(foreground);
    style.inline_code_color = Some(foreground);
    style.quote_code_color = Some(foreground);
    style.table_header.text.color = Some(foreground);
    style.alert.text.color = Some(foreground);
    for alert in style.alerts.iter_mut().flatten() {
        alert.container.text.color = Some(foreground);
    }
    for heading in &mut style.headings {
        heading.text.color = Some(foreground);
    }

    let highlight = Arc::make_mut(&mut style.highlight_theme);
    highlight.style.editor_foreground = Some(foreground);
    highlight.style.syntax.variable = Some(neutral);
    highlight.style.syntax.operator = Some(neutral);
    highlight.style.syntax.punctuation = Some(neutral);
    highlight.style.syntax.punctuation_bracket = Some(neutral);
    highlight.style.syntax.punctuation_delimiter = Some(neutral);
    highlight
        .style
        .syntax
        .captures
        .insert("github.rust.type".into(), neutral);
}

/// The source buffer owns all editable state. This entity retains only the
/// derived document, selection, reading focus and scroll position.
pub(crate) struct MarkdownPreview {
    source: Entity<InputState>,
    host: SharedHost,
    path: PathBuf,
    app: gpui::WeakEntity<Tty7App>,
    images: HashMap<PathBuf, TextViewImageSource>,
    diagrams: HashMap<String, TextViewImageSource>,
    image_bytes: usize,
    pending_anchor: Option<String>,
    last_position: Option<(usize, Pixels)>,
    restore_position: Option<(usize, Pixels)>,
    pub(crate) text: Entity<TextViewState>,
    pub(crate) scroll: ScrollHandle,
    revision: u64,
    layout_selection: Option<gpui_component::text::TextViewSelectionSnapshot>,
    width: Pixels,
    style_key: Option<(String, u64, bool, u32, SharedString)>,
    style: TextViewStyle,
    available_fonts: Option<Arc<Vec<String>>>,
    mermaid_enabled: bool,
    diagram_generation: u64,
    diagram_task: Option<gpui::Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl MarkdownPreview {
    pub(crate) fn is_loading(&self, cx: &App) -> bool {
        self.text.read(cx).source().is_empty() && self.source.read(cx).text().len() != 0
    }

    pub(crate) fn new(
        source: Entity<InputState>,
        host: SharedHost,
        path: PathBuf,
        app: gpui::WeakEntity<Tty7App>,
        mermaid_enabled: bool,
        cx: &mut Context<Self>,
    ) -> Self {
        let content = source.read(cx).text().to_string();
        let reading_content = markdown_frontmatter::preprocess(&content);
        let processed_content = if mermaid_enabled {
            Self::mermaid_placeholders(&reading_content).0
        } else {
            reading_content.into_owned()
        };
        let text = cx.new(|cx| {
            // Use the component's streaming parser for the initial document.
            // Parsing markdown-rs on the UI thread delays the click response;
            // this reading pane owns its scroll extent and can reflow when the
            // background parse lands (unlike an item measured by an outer list).
            let mut text = TextViewState::markdown_with_extensions("", extensions(), cx);
            text.push_str(&processed_content, cx);
            text
        });
        let subscriptions = vec![
            cx.observe_global::<Registry>(|_, cx| cx.notify()),
            cx.observe_global::<gpui_component::Theme>(|_, cx| cx.notify()),
        ];
        cx.on_release(|this, cx| {
            for image in this.images.values().chain(this.diagrams.values()) {
                if let TextViewImageSource::Ready(source) = image {
                    source.remove_asset(cx);
                }
            }
        })
        .detach();
        let mut this = Self {
            source,
            host,
            path,
            app,
            images: HashMap::new(),
            diagrams: HashMap::new(),
            image_bytes: 0,
            pending_anchor: None,
            last_position: None,
            restore_position: None,
            text,
            scroll: ScrollHandle::new(),
            revision: 0,
            layout_selection: None,
            width: px(0.),
            style_key: None,
            style: TextViewStyle::default(),
            available_fonts: None,
            mermaid_enabled,
            diagram_generation: 0,
            diagram_task: None,
            _subscriptions: subscriptions,
        };
        this.start_diagrams(content, cx);
        this
    }

    pub(crate) fn sync(&mut self, revision: u64, cx: &mut Context<Self>) {
        if self.revision == revision {
            return;
        }
        self.revision = revision;
        // A new source revision gets a fresh resource budget. Completions from
        // the previous revision must not repopulate this cache.
        for (_, image) in self.images.drain() {
            if let TextViewImageSource::Ready(source) = image {
                source.remove_asset(cx);
            }
        }
        self.image_bytes = 0;
        self.last_position = None;
        self.restore_position = None;
        let content = self.source.read(cx).text().to_string();
        self.start_diagrams(content, cx);
        cx.notify();
    }

    pub(crate) fn remember_layout_selection(
        &mut self,
        snapshot: Option<gpui_component::text::TextViewSelectionSnapshot>,
    ) {
        self.layout_selection = snapshot;
    }

    pub(crate) fn restore_layout_selection(&mut self, cx: &mut Context<Self>) {
        if let Some(snapshot) = self.layout_selection.take() {
            self.text.update(cx, |text, cx| {
                text.restore_selection(snapshot, cx);
            });
        }
    }

    pub(crate) fn navigate_anchor(&mut self, anchor: String, cx: &mut Context<Self>) {
        self.pending_anchor = Some(anchor);
        cx.notify();
    }

    fn open_link(&mut self, target: &str, window: &mut Window, cx: &mut Context<Self>) {
        match markdown_document::resolve(&self.path, target, self.host.id().is_local()) {
            Ok(Target::Web(url)) => cx.open_url(&url),
            Ok(Target::Anchor(anchor)) => self.navigate_anchor(anchor, cx),
            Ok(Target::File { path, fragment }) => {
                let host = self.host.clone();
                let _ = self.app.update(cx, |app, cx| {
                    app.editor_open_markdown_link(host, &path, fragment, window, cx)
                });
            }
            Err(error) => window.push_notification(error, cx),
        }
    }

    fn image_source(
        &mut self,
        url: &gpui::SharedUri,
        cx: &mut Context<Self>,
    ) -> TextViewImageSource {
        if let Some(diagram) = self.diagrams.get(url.as_ref()) {
            return diagram.clone();
        }
        let resource =
            match markdown_document::resolve(&self.path, url.as_ref(), self.host.id().is_local()) {
                Ok(Target::Web(url)) => Target::Web(url),
                Ok(target @ Target::File { .. }) => target,
                Ok(Target::Anchor(_)) => {
                    return TextViewImageSource::Failed("image has no resource path".into());
                }
                Err(error) => return TextViewImageSource::Failed(error.into()),
            };
        // Web cache keys never reach the owning Host's filesystem methods.
        let path = match &resource {
            Target::Web(url) => PathBuf::from(url),
            Target::File { path, .. } => path.clone(),
            Target::Anchor(_) => unreachable!(),
        };
        if let Some(image) = self.images.get(&path) {
            return image.clone();
        }
        if self.images.len() >= 64 {
            return TextViewImageSource::Failed("document image limit reached (64)".into());
        }
        self.images
            .insert(path.clone(), TextViewImageSource::Loading);
        let revision = self.revision;
        let client = cx.http_client();
        HostOps::run(
            self.host.clone(),
            cx,
            move |host| {
                let bytes = match resource {
                    Target::Web(url) => smol::block_on(fetch_image(client, &url))?,
                    Target::File { path, .. } => {
                        let metadata = host.stat(&path).map_err(|e| e.to_string())?;
                        if metadata.len > markdown_document::MAX_IMAGE_BYTES {
                            return Err("image exceeds 8 MiB".to_string());
                        }
                        host.read_file(&path, markdown_document::MAX_IMAGE_BYTES)
                            .map_err(|e| e.to_string())?
                    }
                    Target::Anchor(_) => unreachable!(),
                };
                prepare_image(bytes)
            },
            move |this, result, cx| {
                if this.revision != revision {
                    return;
                }
                let image = match result {
                    Ok((source, cost)) if this.image_bytes + cost <= 32 * 1024 * 1024 => {
                        this.image_bytes += cost;
                        TextViewImageSource::Ready(source.into_source())
                    }
                    Ok(_) => {
                        TextViewImageSource::Failed("document image cache exceeds 32 MiB".into())
                    }
                    Err(error) => TextViewImageSource::Failed(error.into()),
                };
                this.images.insert(path, image);
                cx.notify();
            },
        );
        TextViewImageSource::Loading
    }

    fn on_key_down(&mut self, event: &gpui::KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let key = &event.keystroke;
        if key.modifiers.alt || key.modifiers.shift && key.key != "space" {
            return;
        }
        let document_edge = if cfg!(target_os = "macos") {
            key.modifiers.platform && !key.modifiers.control
        } else {
            key.modifiers.control && !key.modifiers.platform
        };
        let modified = key.modifiers.control || key.modifiers.platform;
        let page = self.scroll.bounds().size.height * 0.9;
        let offset = self.scroll.offset();
        let y = if modified {
            match (document_edge, cfg!(target_os = "macos"), key.key.as_str()) {
                (true, true, "up") | (true, false, "home") => px(0.),
                (true, true, "down") | (true, false, "end") => -self.scroll.max_offset().y,
                _ => return,
            }
        } else {
            match key.key.as_str() {
                "up" => offset.y + px(40.),
                "down" => offset.y - px(40.),
                "pageup" => offset.y + page,
                "pagedown" => offset.y - page,
                "space" if key.modifiers.shift => offset.y + page,
                "space" => offset.y - page,
                "home" => px(0.),
                "end" => -self.scroll.max_offset().y,
                _ => return,
            }
        };
        self.scroll.set_offset(point(
            offset.x,
            y.clamp(-self.scroll.max_offset().y, px(0.)),
        ));
        cx.stop_propagation();
        cx.notify();
    }

    /// Replace ```` ```mermaid ```` fences with inline SVG rendered in the
    /// active Markdown theme. A diagram that fails to render stays a plain
    /// code block, followed by an HTML comment carrying the error.
    fn mermaid_fences() -> &'static regex::Regex {
        static MERMAID_FENCE: LazyLock<regex::Regex> = LazyLock::new(|| {
            regex::Regex::new(r"(?m)^```mermaid[ \t]*\r?\n([\s\S]*?)^```")
                .expect("mermaid fence regex is valid")
        });
        &MERMAID_FENCE
    }

    fn mermaid_placeholders(content: &str) -> (String, usize) {
        let mut count = 0;
        let processed = Self::mermaid_fences()
            .replace_all(content, |_: &regex::Captures| {
                let image = format!("\n![Mermaid diagram](ctty7-mermaid://{count})\n");
                count += 1;
                image
            })
            .into_owned();
        (processed, count)
    }

    fn start_diagrams(&mut self, content: String, cx: &mut Context<Self>) {
        let content = markdown_frontmatter::preprocess(&content).into_owned();
        self.diagram_generation = self.diagram_generation.wrapping_add(1);
        self.diagram_task.take();
        let (processed, count) = if self.mermaid_enabled {
            Self::mermaid_placeholders(&content)
        } else {
            (content.clone(), 0)
        };
        for image in self.diagrams.values() {
            if let TextViewImageSource::Ready(source) = image {
                source.remove_asset(cx);
            }
        }
        self.diagrams = (0..count)
            .map(|i| (format!("ctty7-mermaid://{i}"), TextViewImageSource::Loading))
            .collect();
        self.text
            .update(cx, |text, cx| text.set_text(&processed, cx));
        if count == 0 {
            return;
        }
        let generation = self.diagram_generation;
        let theme = current(cx).theme;
        let dark = cx.theme().mode.is_dark();
        let render = cx
            .background_executor()
            .spawn(async move { Self::preprocess_mermaid(&content, &theme, dark) });
        self.diagram_task = Some(cx.spawn(async move |this, cx| {
            let (processed, diagrams) = render.await;
            let _ = this.update(cx, |this, cx| {
                if this.diagram_generation != generation {
                    return;
                }
                this.restore_position = this
                    .text
                    .read(cx)
                    .reading_position(this.scroll.bounds().top())
                    .or(this.last_position);
                this.diagrams = diagrams
                    .into_iter()
                    .map(|(url, svg)| {
                        (
                            url,
                            TextViewImageSource::Ready(gpui::ImageSource::Image(Arc::new(
                                gpui::Image::from_bytes(gpui::ImageFormat::Svg, svg.into_bytes()),
                            ))),
                        )
                    })
                    .collect();
                this.text
                    .update(cx, |text, cx| text.set_text(&processed, cx));
                cx.notify();
            });
        }));
    }

    fn preprocess_mermaid(
        content: &str,
        theme: &Theme,
        dark: bool,
    ) -> (String, HashMap<String, String>) {
        let mermaid_fence = Self::mermaid_fences();

        if !mermaid_fence.is_match(content) {
            return (content.to_string(), HashMap::new());
        }

        let mut diagrams = HashMap::new();
        let mut index = 0;
        let processed = mermaid_fence
            .replace_all(content, |caps: &regex::Captures| {
                let source = &caps[1];
                let url = format!("ctty7-mermaid://{index}");
                index += 1;
                match crate::ui::markdown_mermaid::render_mermaid(source, theme, dark) {
                    Ok(svg) => {
                        diagrams.insert(url.clone(), svg);
                        format!("\n![Mermaid diagram]({url})\n")
                    }
                    Err(err) => {
                        format!("```mermaid\n{source}```\n<!-- Mermaid render error: {err} -->")
                    }
                }
            })
            .into_owned();
        (processed, diagrams)
    }
}

async fn fetch_image(
    client: Arc<dyn gpui::http_client::HttpClient>,
    url: &str,
) -> Result<Vec<u8>, String> {
    use smol::{future::FutureExt as _, io::AsyncReadExt as _};
    async {
        let mut response = client
            .get(url, ().into(), true)
            .await
            .map_err(|e| e.to_string())?;
        if !response.status().is_success() {
            return Err(format!("image HTTP status: {}", response.status()));
        }
        let mut bytes = Vec::new();
        response
            .body_mut()
            .take(markdown_document::MAX_IMAGE_BYTES + 1)
            .read_to_end(&mut bytes)
            .await
            .map_err(|e| e.to_string())?;
        if bytes.len() as u64 > markdown_document::MAX_IMAGE_BYTES {
            return Err("image exceeds 8 MiB".into());
        }
        Ok(bytes)
    }
    .or(async {
        smol::Timer::after(std::time::Duration::from_secs(15)).await;
        Err("image request timed out".into())
    })
    .await
}

enum PreparedImage {
    Encoded(Arc<gpui::Image>),
    Animated(Arc<gpui::RenderImage>),
}

impl PreparedImage {
    fn into_source(self) -> gpui::ImageSource {
        match self {
            Self::Encoded(image) => gpui::ImageSource::Image(image),
            Self::Animated(image) => image.into(),
        }
    }
}

fn prepare_image(bytes: Vec<u8>) -> Result<(PreparedImage, usize), String> {
    let format = markdown_document::image_format(&bytes)?;
    let mut cost = bytes.len();
    if matches!(format, gpui::ImageFormat::Svg)
        && let Some(image) = crate::core::markdown_svg::render(&bytes)?
    {
        cost += (0..image.frame_count())
            .map(|i| image.as_bytes(i).map_or(0, <[u8]>::len))
            .sum::<usize>();
        return Ok((PreparedImage::Animated(image), cost));
    }
    Ok((
        PreparedImage::Encoded(Arc::new(gpui::Image::from_bytes(format, bytes))),
        cost,
    ))
}

impl Focusable for MarkdownPreview {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.text.read(cx).focus_handle(cx)
    }
}

impl Render for MarkdownPreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entry = current(cx);
        let dark = cx.theme().mode.is_dark();
        let scale = f32::from(window.rem_size()) / crate::core::config::UI_FONT_SIZE_DEFAULT;
        let mono = cx.theme().mono_font_family.clone();
        let key = (
            entry.theme.id.clone(),
            entry.revision,
            dark,
            scale.to_bits(),
            mono.clone(),
        );
        let available = self.available_fonts.get_or_insert_with(|| {
            if let Some(fonts) = cx.try_global::<ReadingFonts>() {
                return fonts.0.clone();
            }
            let fonts = Arc::new(window.text_system().all_font_names());
            cx.set_global(ReadingFonts(fonts.clone()));
            fonts
        });
        let body_font = if entry.theme.typography.resolve_font_stack {
            available_reading_font(
                &entry.theme.typography.fonts,
                available,
                ".SystemUIFont".into(),
            )
        } else {
            entry
                .theme
                .typography
                .fonts
                .first()
                .cloned()
                .map(SharedString::from)
                .unwrap_or_else(|| cx.theme().font_family.clone())
        };
        let code_font =
            available_reading_font(&entry.theme.typography.code_fonts, available, mono.clone());
        let heading_font = available_reading_font(
            &entry.theme.typography.heading_fonts,
            available,
            body_font.clone(),
        );
        if self.style_key.as_ref() != Some(&key) {
            self.restore_position = self
                .text
                .read(cx)
                .reading_position(self.scroll.bounds().top())
                .or(self.last_position);
            self.style = reading_style(&entry, dark, scale, mono);
            if entry.theme.typography.resolve_font_stack {
                self.style.code_block.text.font_family = Some(code_font.clone());
                self.style.inline_code_font = Some(code_font);
                for heading in &mut self.style.headings {
                    heading.text.font_family = Some(heading_font.clone());
                }
            }
            if self.style_key.is_some() && self.mermaid_enabled {
                let content = self.source.read(cx).text().to_string();
                self.start_diagrams(content, cx);
            }
            self.style_key = Some(key);
        }
        let palette = entry.theme.palette(dark);
        let reading_foreground = if dark && entry.theme.id == markdown_theme::DEFAULT_ID {
            cx.theme().foreground
        } else {
            color(palette.foreground)
        };
        align_dark_github_foreground(&entry, dark, reading_foreground, &mut self.style);
        // The bundled theme shares the application's reading background.
        // Custom themes retain their explicit background and paper colors.
        let reading_background: gpui::Background = if entry.theme.id == markdown_theme::DEFAULT_ID {
            cx.theme().tokens.background.into()
        } else {
            color(palette.background).into()
        };
        let paper_background = if entry.theme.id == markdown_theme::DEFAULT_ID {
            reading_background
        } else {
            color(palette.paper).into()
        };
        let typography = &entry.theme.typography;
        let layout = &entry.theme.layout;
        let compact = self.width < px(layout.compact_below * scale);
        let padding = if compact {
            layout.compact_padding
        } else {
            layout.padding
        } * scale;
        let outer = if compact {
            0.
        } else {
            layout.outer_padding * scale
        };
        let copy_color = color(palette.muted);
        let link_owner = cx.weak_entity();
        let image_owner = cx.weak_entity();
        let diagram_owner = cx.weak_entity();
        let text = TextView::new(&self.text)
            .markdown_extensions(extensions())
            .selectable(true)
            .style(self.style.clone())
            .on_link(move |url, window, cx| {
                let _ = link_owner.update(cx, |this, cx| this.open_link(url, window, cx));
            })
            .image_source(move |url, _, cx| {
                image_owner
                    .update(cx, |this, cx| this.image_source(url, cx))
                    .unwrap_or(TextViewImageSource::Failed("document closed".into()))
            })
            .image_actions(move |url, _, cx| {
                diagram_owner
                    .update(cx, |this, cx| {
                        let TextViewImageSource::Ready(image) = this.diagrams.get(url.as_ref())?
                        else {
                            return None;
                        };
                        let index: usize = url.strip_prefix("ctty7-mermaid://")?.parse().ok()?;
                        let content = this.source.read(cx).text().to_string();
                        let code: SharedString =
                            Self::mermaid_fences().captures_iter(&content).nth(index)?[1]
                                .to_owned()
                                .into();
                        let expanded_image = image.clone();
                        Some(
                            div()
                                .flex()
                                .gap_1()
                                .child(
                                    Button::new("expand-diagram")
                                        .icon(IconName::Maximize)
                                        .ghost()
                                        .xsmall()
                                        .text_color(copy_color)
                                        .tooltip(t(L10nKey::MarkdownExpandDiagram))
                                        .on_click(move |_, window, cx| {
                                            cx.stop_propagation();
                                            crate::ui::markdown_mermaid::open_diagram(
                                                expanded_image.clone(),
                                                window,
                                                cx,
                                            );
                                        }),
                                )
                                .child(
                                    Button::new("copy-diagram")
                                        .icon(IconName::Copy)
                                        .ghost()
                                        .xsmall()
                                        .text_color(copy_color)
                                        .tooltip(t(L10nKey::EditorCopyCode))
                                        .on_click(move |_, _, cx| {
                                            cx.stop_propagation();
                                            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                                                code.to_string(),
                                            ));
                                        }),
                                ),
                        )
                    })
                    .ok()
                    .flatten()
            })
            .code_block_actions(move |block, _, _| {
                let code = block.code();
                Button::new("copy-code")
                    .icon(IconName::Copy)
                    .ghost()
                    .text_color(copy_color)
                    .tooltip(t(L10nKey::EditorCopyCode))
                    .on_click(move |_, _, cx| {
                        cx.write_to_clipboard(gpui::ClipboardItem::new_string(code.to_string()));
                    })
            });
        let mut card = div()
            .w_full()
            .max_w(px(layout.max_width * scale))
            .mx_auto()
            .p(px(padding))
            .text_size(px(typography.font_size * scale))
            .line_height(relative(typography.line_height))
            .font_family(body_font)
            .text_color(reading_foreground)
            .bg(paper_background);
        card.style().text.font_fallbacks =
            Some(gpui::FontFallbacks::from_fonts(typography.fonts.clone()));
        let card = card
            .when(!compact, |el| {
                el.rounded(px(layout.radius * scale))
                    .border(px(layout.paper_border_width))
                    .border_color(color(palette.paper_border))
                    .shadow(vec![BoxShadow {
                        color: color(palette.shadow),
                        offset: point(px(0.), px(layout.shadow_offset * scale)),
                        blur_radius: px(layout.shadow_blur * scale),
                        spread_radius: px(0.),
                        inset: false,
                    }])
            })
            .when(self.is_loading(cx), |card| {
                card.child(t(L10nKey::PanelLoading))
            })
            .child(text);
        let entity = cx.weak_entity();
        let scroll = self.scroll.clone();
        let scroll_area = crate::ui::scrollbar::with_vertical_scrollbar(
            "markdown-reading-scrollbar",
            div()
                .id("markdown-reading")
                .size_full()
                .key_context("MarkdownPreview")
                .bg(reading_background)
                .overflow_y_scroll()
                .track_scroll(&scroll)
                .p(px(outer))
                .on_key_down(cx.listener(Self::on_key_down))
                .child(card)
                .child(
                    gpui::canvas(
                        move |_, window, cx| {
                            let _ = entity.update(cx, |this, cx| {
                                // Observe layout without adding a viewport-sized child
                                // after the document. Read the viewport from the handle.
                                let bounds = this.scroll.bounds();
                                if this.width != bounds.size.width {
                                    this.width = bounds.size.width;
                                    this.restore_position =
                                        this.restore_position.or(this.last_position);
                                    cx.notify();
                                    return;
                                }
                                // An anchor requested while opening the file must wait
                                // for its initial background parse and first layout.
                                if this.is_loading(cx) {
                                    return;
                                }
                                if let Some(anchor) = this.pending_anchor.take() {
                                    this.restore_position = None;
                                    if anchor.is_empty() {
                                        this.scroll.set_offset(point(px(0.), px(0.)));
                                    } else if let Some(target) =
                                        this.text.read(cx).anchor_bounds(&anchor)
                                    {
                                        let offset = this.scroll.offset();
                                        let y = offset.y - (target.top() - bounds.top()) + px(12.);
                                        this.scroll.set_offset(point(
                                            offset.x,
                                            y.clamp(-this.scroll.max_offset().y, px(0.)),
                                        ));
                                    } else {
                                        window.push_notification(
                                            format!(
                                                "#{anchor}: {}",
                                                t(L10nKey::MarkdownAnchorMissing)
                                            ),
                                            cx,
                                        );
                                    }
                                    cx.notify();
                                    return;
                                }
                                if let Some((index, inside)) = this.restore_position.take() {
                                    if let Some(target) = this.text.read(cx).block_bounds(index) {
                                        let offset = this.scroll.offset();
                                        let y = offset.y + bounds.top() - inside - target.top();
                                        let y = y.clamp(-this.scroll.max_offset().y, px(0.));
                                        if (y - offset.y).abs() > px(0.1) {
                                            this.scroll.set_offset(point(offset.x, y));
                                            cx.notify();
                                            return;
                                        }
                                    }
                                }
                                this.last_position =
                                    this.text.read(cx).reading_position(bounds.top());
                            });
                        },
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .top_0()
                    .left_0()
                    .size(px(0.)),
                ),
            &scroll,
        );
        // Cached views lay out their contents as a root; provide the definite
        // flex-column height required by the shared scrollbar wrapper.
        div().flex().flex_col().size_full().child(scroll_area)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{TestAppContext, VisualTestContext};
    use std::{
        io,
        path::Path,
        sync::{
            Mutex,
            atomic::{AtomicBool, Ordering},
            mpsc,
        },
        time::{Duration, Instant},
    };
    use tty7_core::host::{self, Host, HostId, Meta};

    #[test]
    fn markdown_mermaid_crlf_fences_reach_the_renderer() {
        let content = "```mermaid\r\nsequenceDiagram\r\n    participant User\r\n    participant UI\r\n    participant Daemon\r\n    User->>UI: choose command\r\n    UI->>Daemon: send expanded prompt\r\n    Daemon-->>UI: stream result\r\n```\r\n";
        let (placeholder, count) = MarkdownPreview::mermaid_placeholders(content);
        assert_eq!(
            count, 1,
            "Windows line endings must not hide Mermaid fences"
        );
        let (processed, diagrams) =
            MarkdownPreview::preprocess_mermaid(content, &markdown_theme::builtin(), false);
        assert_eq!(processed, placeholder);
        let svg = &diagrams["ctty7-mermaid://0"];
        assert!(svg.contains("<svg"));
        assert!(svg.contains("choose command"));
        assert!(svg.contains("stream result"));
    }

    #[test]
    fn markdown_mermaid_invalid_crlf_source_remains_a_code_block() {
        let content = "```mermaid\r\nnot-a-diagram\r\n```\r\n";
        let (processed, diagrams) =
            MarkdownPreview::preprocess_mermaid(content, &markdown_theme::builtin(), false);
        assert!(diagrams.is_empty());
        assert!(processed.starts_with("```mermaid\nnot-a-diagram\r\n```"));
        assert!(processed.contains("Mermaid render error:"));
        assert!(!processed.contains("ctty7-mermaid://"));
    }

    #[gpui::test]
    fn markdown_skill_frontmatter_is_a_metadata_table(cx: &mut TestAppContext) {
        let (app, mut vcx) = crate::ui::app::test_window::harness(cx);
        let content = "---\nname: show-me\ndescription: Show a diagram.\ndisable-model-invocation: true\n---\n\nBody.\n";
        for (mermaid_enabled, newline) in
            [(false, "\n"), (true, "\n"), (false, "\r\n"), (true, "\r\n")]
        {
            let content = content.replace('\n', newline);
            let source = vcx.update(|window, cx| {
                cx.new(|cx| {
                    let mut source = InputState::new(window, cx).multi_line(true);
                    source.set_value(&content, window, cx);
                    source
                })
            });
            let reading = vcx.new(|cx| {
                MarkdownPreview::new(
                    source.clone(),
                    tty7_core::host::local::LocalHost::new(),
                    PathBuf::from("/repo/SKILL.md"),
                    app.downgrade(),
                    mermaid_enabled,
                    cx,
                )
            });
            let text = reading.read_with(&vcx, |reading, _| reading.text.clone());
            settle(&mut vcx, |cx| {
                text.update(cx, |text, cx| text.select_all(cx));
                !text
                    .read_with(cx, |text, _| text.selected_text())
                    .is_empty()
            });
            text.update(&mut vcx, |text, cx| text.select_all(cx));
            let selected = text.read_with(&vcx, |text, _| text.selected_text());
            assert!(
                selected.starts_with(
                    "name show-me\ndescription Show a diagram.\ndisable-model-invocation true\n"
                ),
                "frontmatter must copy as table rows, not a heading: {selected:?}"
            );
            assert_eq!(
                source.read_with(&vcx, |source, _| source.text().to_string()),
                content
            );
            let edited = content.replace("show-me", "updated");
            source.update_in(&mut vcx, |source, window, cx| {
                source.set_value(&edited, window, cx)
            });
            reading.update(&mut vcx, |reading, cx| reading.sync(1, cx));
            text.update(&mut vcx, |text, cx| text.select_all(cx));
            assert!(
                text.read_with(&vcx, |text, _| text.selected_text())
                    .starts_with("name updated\n")
            );
            assert_eq!(
                source.read_with(&vcx, |source, _| source.text().to_string()),
                edited
            );
            source.update_in(&mut vcx, |source, window, cx| {
                source.set_value("# Body only\n", window, cx)
            });
            reading.update(&mut vcx, |reading, cx| reading.sync(2, cx));
            text.update(&mut vcx, |text, cx| text.select_all(cx));
            assert_eq!(
                text.read_with(&vcx, |text, _| text.selected_text()).trim(),
                "Body only"
            );
        }
    }

    #[test]
    fn reading_code_matches_live_github_token_styles() {
        use gpui_component::highlighter::SyntaxHighlighter;
        #[derive(serde::Deserialize)]
        struct Token {
            start: usize,
            end: usize,
            text: String,
            color: String,
            background: String,
            weight: String,
        }
        #[derive(serde::Deserialize)]
        struct Sample {
            language: String,
            source: String,
            tokens: Vec<Token>,
        }
        #[derive(serde::Deserialize)]
        struct Capture {
            samples: Vec<Sample>,
        }
        fn rgb(value: &str) -> Hsla {
            let components: Vec<u32> = value
                .trim_start_matches("rgb(")
                .trim_end_matches(')')
                .split(',')
                .map(|part| part.trim().parse().unwrap())
                .collect();
            gpui::rgb((components[0] << 16) | (components[1] << 8) | components[2]).into()
        }
        let entry = Entry {
            theme: markdown_theme::builtin(),
            source: None,
            revision: 0,
        };
        let mut failures = Vec::new();
        for (dark, fixture) in [
            (
                false,
                include_str!("../../tests/fixtures/github-live-2026-10-06/code-light.json"),
            ),
            (
                true,
                include_str!("../../tests/fixtures/github-live-2026-10-06/code-dark.json"),
            ),
        ] {
            let capture: Capture = serde_json::from_str(fixture).unwrap();
            let style = reading_style(&entry, dark, 1., "Mono".into());
            for sample in capture.samples {
                let language = if sample.language.contains("source-rust") {
                    "rust"
                } else {
                    "diff"
                };
                let alias = style
                    .code_block_languages
                    .get(language)
                    .map(|name| name.as_ref())
                    .unwrap_or(language);
                let mut highlighter = SyntaxHighlighter::new(alias);
                highlighter.update(None, &sample.source.as_str().into(), None);
                let runs = highlighter.styles(&(0..sample.source.len()), &style.highlight_theme);
                for token in sample.tokens {
                    for offset in token.start..token.end {
                        if sample.source.as_bytes()[offset].is_ascii_whitespace() {
                            continue;
                        }
                        let actual = runs
                            .iter()
                            .find(|(range, _)| range.contains(&offset))
                            .map(|(_, run)| *run)
                            .unwrap_or_default();
                        let foreground = actual
                            .color
                            .or(style.highlight_theme.style.editor_foreground);
                        let expected_background = if token.background.starts_with("rgb(") {
                            rgb(&token.background)
                        } else {
                            style.highlight_theme.style.editor_background.unwrap()
                        };
                        let background = actual
                            .background_color
                            .or(style.highlight_theme.style.editor_background);
                        let weight = actual.font_weight.unwrap_or(FontWeight::NORMAL).0;
                        if foreground != Some(rgb(&token.color))
                            || background != Some(expected_background)
                            || weight != token.weight.parse::<f32>().unwrap()
                        {
                            failures.push(format!("{language} {:?} at {offset}: foreground={foreground:?}, background={background:?}, weight={weight}", token.text));
                            break;
                        }
                    }
                }
            }
        }
        assert!(
            failures.is_empty(),
            "Live GitHub style differences:\n{}",
            failures.join("\n")
        );
    }

    #[test]
    #[ignore = "requires the native Windows font collection with Noto Sans SC"]
    #[cfg(target_os = "windows")]
    fn native_github_cjk_uses_noto_at_body_and_heading_weights() {
        let platform = gpui_platform::current_platform(false);
        let native = platform.text_system();
        let system = Arc::new(gpui::TextSystem::new(native.clone()));
        let shaper = gpui::WindowTextSystem::new(system.clone());
        let theme = markdown_theme::builtin();
        let names = system.all_font_names();
        let body = available_reading_font(&theme.typography.fonts, &names, ".SystemUIFont".into());
        assert!(names.iter().any(|name| name == "Noto Sans SC"));
        for weight in [FontWeight::NORMAL, FontWeight::SEMIBOLD] {
            let mut expected = gpui::font("Noto Sans SC");
            expected.weight = weight;
            let expected_id = native.font_id(&expected).unwrap();
            let mut font = gpui::font(body.clone());
            font.weight = weight;
            font.fallbacks = Some(gpui::FontFallbacks::from_fonts(
                theme.typography.fonts.clone(),
            ));
            let text: SharedString = "编程中文现代终端".into();
            let line = shaper.shape_line(
                text.clone(),
                px(16.),
                &[gpui::TextRun {
                    len: text.len(),
                    font,
                    color: gpui::black(),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                }],
                None,
            );
            assert!(!line.runs.is_empty());
            assert!(
                line.runs.iter().all(|run| run.font_id == expected_id),
                "CJK fallback differs from Noto Sans SC at {weight:?}: {:?}",
                line.runs
            );
        }
    }

    #[test]
    fn markdown_remote_svg_uses_the_same_animation_pipeline_as_host_images() {
        use gpui::http_client::{FakeHttpClient, Response};
        let bytes = crate::core::markdown_svg::tests::typing_with_embedded_font().into_bytes();
        let body = bytes.clone();
        let client = FakeHttpClient::create(move |request| {
            assert_eq!(
                request.uri().to_string(),
                "https://example.com/animated.svg"
            );
            let body = body.clone();
            async move { Ok(Response::builder().status(200).body(body.into()).unwrap()) }
        });
        let remote =
            smol::block_on(fetch_image(client, "https://example.com/animated.svg")).unwrap();
        assert_eq!(remote, bytes);
        for bytes in [bytes, remote] {
            let (PreparedImage::Animated(image), cost) = prepare_image(bytes).unwrap() else {
                panic!("local and remote SVGs must both animate");
            };
            assert!(cost > 20 * 160 * 30 * 4);
            assert_ne!(image.as_bytes(0), image.as_bytes(10));
        }
    }

    #[test]
    fn markdown_remote_image_rejects_http_errors_and_large_bodies() {
        use gpui::http_client::{FakeHttpClient, Response};
        for (status, size, error) in [
            (404, 0, "HTTP status"),
            (
                200,
                markdown_document::MAX_IMAGE_BYTES as usize + 1,
                "8 MiB",
            ),
        ] {
            let client = FakeHttpClient::create(move |_| async move {
                Ok(Response::builder()
                    .status(status)
                    .body(vec![0; size].into())
                    .unwrap())
            });
            assert!(
                smol::block_on(fetch_image(client, "https://example.com/image"))
                    .unwrap_err()
                    .contains(error)
            );
        }
    }

    #[test]
    fn minimal_v2_has_github_reading_style_at_all_scales_and_modes() {
        let custom = Entry {
            theme: Arc::new(
                markdown_theme::parse(
                    "schema_version: 2\nid: empty\nname: Empty\nlight: {}\ndark: {}\n",
                )
                .unwrap(),
            ),
            source: None,
            revision: 0,
        };
        let github = Entry {
            theme: markdown_theme::builtin(),
            ..custom.clone()
        };
        for dark in [false, true] {
            for scale in [1., 1.5, 2.] {
                let mut actual = reading_style(&custom, dark, scale, "Mono".into());
                let expected = reading_style(&github, dark, scale, "Mono".into());
                // Theme IDs distinguish highlighter caches, not visible styles.
                Arc::make_mut(&mut actual.highlight_theme).name =
                    expected.highlight_theme.name.clone();
                assert!(actual == expected);
            }
        }
    }

    #[test]
    fn github_dark_neutral_text_can_follow_the_application_foreground() {
        let entry = Entry {
            theme: markdown_theme::builtin(),
            source: None,
            revision: 0,
        };
        let foreground: Hsla = gpui::rgb(0xabb2bf).into();
        let mut style = reading_style(&entry, true, 1., "Mono".into());

        align_dark_github_foreground(&entry, true, foreground, &mut style);

        assert_eq!(style.code_block.text.color, Some(foreground));
        assert_eq!(style.inline_code_color, Some(foreground));
        assert_eq!(style.quote_code_color, Some(foreground));
        assert_eq!(style.table_header.text.color, Some(foreground));
        assert_eq!(style.alert.text.color, Some(foreground));
        assert!(
            style
                .alerts
                .iter()
                .flatten()
                .all(|alert| { alert.container.text.color == Some(foreground) }),
            "GitHub alert bodies must use the same neutral text as paragraphs"
        );
        assert!(
            style
                .headings
                .iter()
                .all(|heading| heading.text.color == Some(foreground))
        );
        assert_eq!(
            style.highlight_theme.style.editor_foreground,
            Some(foreground)
        );
        assert_eq!(
            style.highlight_theme.style.syntax.variable,
            Some(ThemeStyle::from(foreground))
        );
    }

    #[test]
    fn github_dark_rust_types_follow_neutral_text_without_changing_keywords() {
        use gpui_component::highlighter::SyntaxHighlighter;

        let entry = Entry {
            theme: markdown_theme::builtin(),
            source: None,
            revision: 0,
        };
        let foreground: Hsla = gpui::rgb(0xabb2bf).into();
        let mut style = reading_style(&entry, true, 1., "Mono".into());
        align_dark_github_foreground(&entry, true, foreground, &mut style);
        let source = "struct Widget;\nlet item: Widget;\n";
        let mut highlighter = SyntaxHighlighter::new(&style.code_block_languages["rust"]);
        highlighter.update(None, &source.into(), None);
        let runs = highlighter.styles(&(0..source.len()), &style.highlight_theme);
        for (word, expected) in [
            ("Widget", foreground),
            ("item", foreground),
            (";", foreground),
            ("struct", color(entry.theme.dark.syntax.keyword)),
        ] {
            for (offset, _) in source.match_indices(word) {
                let actual = runs
                    .iter()
                    .find(|(range, _)| range.contains(&offset))
                    .and_then(|(_, run)| run.color)
                    .or(style.highlight_theme.style.editor_foreground);
                assert_eq!(actual, Some(expected), "Rust token {word} at {offset}");
            }
        }
    }

    #[test]
    fn github_dark_foreground_alignment_leaves_custom_themes_untouched() {
        let theme = markdown_theme::parse(
            "schema_version: 2\nid: custom\nname: Custom\nlight: {}\ndark: {}\n",
        )
        .unwrap();
        let entry = Entry {
            theme: Arc::new(theme),
            source: None,
            revision: 0,
        };
        let foreground: Hsla = gpui::rgb(0xabb2bf).into();
        let before = reading_style(&entry, true, 1., "Mono".into());
        let mut after = before.clone();

        align_dark_github_foreground(&entry, true, foreground, &mut after);

        assert_eq!(before.code_block.text.color, after.code_block.text.color);
        assert_eq!(before.inline_code_color, after.inline_code_color);
        assert_eq!(
            before.highlight_theme.style.editor_foreground,
            after.highlight_theme.style.editor_foreground
        );
    }

    #[test]
    fn markdown_heading_fonts_are_optional_and_independent_of_body_and_code() {
        let yaml = "schema_version: 2\nid: fonts\nname: Fonts\nlight: {}\ndark: {}\ntypography:\n  fonts: [Body]\n  code_fonts: [Code]\n";
        for headings in [
            "",
            "  heading_fonts: []\n",
            "  heading_fonts: [Heading, Fallback]\n",
        ] {
            let theme = markdown_theme::parse(&format!("{yaml}{headings}")).unwrap();
            let style = reading_style(
                &Entry {
                    theme: Arc::new(theme),
                    source: None,
                    revision: 1,
                },
                false,
                1.,
                "Mono".into(),
            );
            let expected = if headings.contains("Heading") {
                Some(SharedString::from("Heading"))
            } else {
                None
            };
            for heading in &style.headings {
                assert_eq!(heading.text.font_family, expected);
            }
            assert_eq!(style.code_block.text.font_family, Some("Code".into()));
        }
    }

    // Only the Host I/O boundary is substituted. The editor, resource adapter,
    // renderer and async completion all run through their application paths.
    struct DocumentsHost {
        id: HostId,
        files: HashMap<PathBuf, Vec<u8>>,
        reads: Mutex<Vec<PathBuf>>,
        image_gate: Mutex<Option<mpsc::Receiver<()>>>,
        image_completed: AtomicBool,
    }

    impl DocumentsHost {
        fn new(id: u64, label: &str, gate: Option<mpsc::Receiver<()>>) -> Arc<Self> {
            Arc::new(Self {
                id: HostId(id),
                files: HashMap::from([
                    ("/repo/docs/readme.md".into(), format!("# {label}\n\n![{label}](images/icon.svg)").into_bytes()),
                    ("/repo/docs/images/icon.svg".into(), format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\"><title>{label}</title><rect width=\"16\" height=\"16\" fill=\"blue\"/></svg>").into_bytes()),
                ]),
                reads: Mutex::default(), image_gate: Mutex::new(gate), image_completed: AtomicBool::new(false),
            })
        }
    }

    fn unsupported<T>() -> io::Result<T> {
        Err(io::ErrorKind::Unsupported.into())
    }

    impl Host for DocumentsHost {
        fn id(&self) -> HostId {
            self.id
        }
        fn separator(&self) -> char {
            '/'
        }
        fn is_absolute(&self, path: &Path) -> bool {
            path.to_string_lossy().starts_with('/')
        }
        fn read_dir(&self, _: &Path, _: Option<&Path>) -> io::Result<Vec<host::Entry>> {
            Ok(Vec::new())
        }
        fn stat(&self, path: &Path) -> io::Result<Meta> {
            let bytes = self.files.get(path).ok_or(io::ErrorKind::NotFound)?;
            Ok(Meta {
                is_dir: false,
                is_symlink: false,
                len: bytes.len() as u64,
                mtime: None,
                readonly: false,
            })
        }
        fn read_file(&self, path: &Path, max: u64) -> io::Result<Vec<u8>> {
            self.reads.lock().unwrap().push(path.to_owned());
            let bytes = self.files.get(path).ok_or(io::ErrorKind::NotFound)?;
            if bytes.len() as u64 > max {
                return Err(io::ErrorKind::InvalidData.into());
            }
            if path.extension().is_some_and(|ext| ext == "svg") {
                if let Some(gate) = self.image_gate.lock().unwrap().take() {
                    gate.recv_timeout(Duration::from_secs(10))
                        .map_err(|_| io::ErrorKind::TimedOut)?;
                }
                self.image_completed.store(true, Ordering::SeqCst);
            }
            Ok(bytes.clone())
        }
        fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
            Ok(path.to_owned())
        }
        fn search(
            &self,
            _: &[PathBuf],
            _: &str,
            _: usize,
            _: usize,
            _: bool,
        ) -> io::Result<Vec<host::SearchHit>> {
            Ok(Vec::new())
        }
        fn write_file(&self, _: &Path, _: &[u8]) -> io::Result<Meta> {
            unsupported()
        }
        fn create_file_new(&self, _: &Path) -> io::Result<()> {
            unsupported()
        }
        fn create_dir(&self, _: &Path, _: bool) -> io::Result<()> {
            unsupported()
        }
        fn rename(&self, _: &Path, _: &Path) -> io::Result<()> {
            unsupported()
        }
        fn remove(&self, _: &Path, _: bool) -> io::Result<()> {
            unsupported()
        }
        fn repo_root(&self, _: &Path) -> io::Result<Option<PathBuf>> {
            Ok(None)
        }
        fn git(&self, _: &Path, _: &[&str]) -> io::Result<host::Output> {
            unsupported()
        }
        fn shells(&self) -> io::Result<host::ShellInventory> {
            unsupported()
        }
        fn watch(&self, _: &[PathBuf]) -> io::Result<host::WatchSub> {
            unsupported()
        }
    }

    #[track_caller]
    fn settle(vcx: &mut VisualTestContext, mut ready: impl FnMut(&mut VisualTestContext) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            vcx.run_until_parked();
            vcx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            if ready(vcx) {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "document resource operation did not settle"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[gpui::test]
    fn github_dark_preview_tracks_the_terminal_foreground_after_theme_changes(
        cx: &mut TestAppContext,
    ) {
        let (app, mut vcx) = crate::ui::app::test_window::harness(cx);
        app.update_in(&mut vcx, |app, window, cx| {
            init(Default::default(), cx);
            app.tabs
                .push(crate::ui::app::Tab::new(crate::ui::pane::Pane::Empty));
            app.active = app.tabs.len() - 1;
            app.set_theme_follow_system(false, window, cx);
            app.set_preset("one_dark_pro", window, cx);
            app.editor_open_on_host(
                DocumentsHost::new(906, "Foreground", None),
                Path::new("/repo/docs/readme.md"),
                window,
                cx,
            );
        });
        settle(&mut vcx, |cx| {
            app.read_with(cx, |app, _| {
                app.tab_code()
                    .and_then(|code| code.active_file())
                    .is_some_and(|file| file.reading.is_some())
            })
        });
        let reading = app.read_with(&vcx, |app, _| {
            app.tab_code()
                .unwrap()
                .active_file()
                .unwrap()
                .reading
                .clone()
                .unwrap()
        });
        for (preset, expected) in [
            ("one_dark_pro", 0xabb2bf),
            ("dracula", 0xf8f8f2),
            ("light", 0x1f2328),
            ("one_dark_pro", 0xabb2bf),
        ] {
            app.update_in(&mut vcx, |app, window, cx| {
                app.set_preset(preset, window, cx)
            });
            vcx.run_until_parked();
            vcx.update(|window, cx| {
                window.refresh();
                let _ = window.draw(cx);
            });
            reading.read_with(&vcx, |reading, cx| {
                let expected = gpui::rgb(expected).into();
                if cx.theme().mode.is_dark() {
                    assert_eq!(cx.theme().foreground, expected);
                }
                assert_eq!(
                    reading.style.code_block.text.color,
                    Some(expected),
                    "{preset}"
                );
                assert_eq!(
                    reading.style.headings[0].text.color,
                    Some(expected),
                    "{preset}"
                );
            });
        }
    }

    #[gpui::test]
    fn markdown_scroll_range_ends_at_the_last_block(cx: &mut TestAppContext) {
        let (app, mut vcx) = crate::ui::app::test_window::harness(cx);
        vcx.simulate_resize(gpui::size(px(1400.), px(900.)));
        let mut host = DocumentsHost::new(907, "Scroll bounds", None);
        let content = format!(
            "{}\n## End marker\n",
            "Paragraph of reading text.\n\n".repeat(60)
        );
        Arc::get_mut(&mut host)
            .unwrap()
            .files
            .insert("/repo/docs/readme.md".into(), content.into_bytes());
        app.update_in(&mut vcx, |app, window, cx| {
            init(Default::default(), cx);
            app.tabs
                .push(crate::ui::app::Tab::new(crate::ui::pane::Pane::Empty));
            app.active = app.tabs.len() - 1;
            app.editor_open_on_host(host, Path::new("/repo/docs/readme.md"), window, cx);
        });
        settle(&mut vcx, |cx| {
            app.read_with(cx, |app, cx| {
                app.tab_code()
                    .and_then(|code| code.active_file())
                    .and_then(|file| file.reading.as_ref())
                    .is_some_and(|reading| {
                        reading
                            .read(cx)
                            .text
                            .read(cx)
                            .anchor_bounds("end-marker")
                            .is_some()
                    })
            })
        });
        let reading = app.read_with(&vcx, |app, _| {
            app.tab_code()
                .unwrap()
                .active_file()
                .unwrap()
                .reading
                .clone()
                .unwrap()
        });
        for _ in 0..2 {
            vcx.run_until_parked();
            vcx.update(|window, cx| {
                let _ = window.draw(cx);
            });
        }
        for layout in [
            crate::core::config::DocumentLayout::Dock,
            crate::core::config::DocumentLayout::Fill,
        ] {
            if layout == crate::core::config::DocumentLayout::Fill {
                let button = vcx.debug_bounds("document-fill-toggle").unwrap();
                vcx.simulate_click(button.center(), gpui::Modifiers::none());
            }
            vcx.simulate_keystrokes(if cfg!(target_os = "macos") {
                "cmd-down"
            } else {
                "ctrl-end"
            });
            vcx.run_until_parked();
            vcx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            reading.read_with(&vcx, |reading, cx| {
                let viewport = reading.scroll.bounds();
                let end = reading.text.read(cx).anchor_bounds("end-marker").unwrap();
                assert!(end.bottom() > viewport.top() && end.top() < viewport.bottom(),
                    "document end must remain visible: end={end:?}, viewport={viewport:?}, offset={:?}, max={:?}",
                    reading.scroll.offset(), reading.scroll.max_offset());
                assert!(viewport.bottom() - end.bottom() < px(80.),
                    "only document padding should follow the final block: end={end:?}, viewport={viewport:?}");
            });
            let viewport = reading.read_with(&vcx, |reading, _| reading.scroll.bounds());
            let panel = vcx.debug_bounds("code-panel").unwrap();
            assert!(
                viewport.top() >= panel.top()
                    && viewport.bottom() <= panel.bottom()
                    && viewport.size.height > panel.size.height - px(90.),
                "reading viewport must fit the visible panel: viewport={viewport:?}, panel={panel:?}"
            );
        }
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_toggle_preview(window, cx)
        });
        vcx.update(|_, cx| {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                "## Short document\n".into(),
            ));
        });
        vcx.simulate_keystrokes("secondary-a secondary-v");
        vcx.run_until_parked();
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_toggle_preview(window, cx)
        });
        settle(&mut vcx, |cx| {
            reading.read_with(cx, |reading, cx| {
                reading
                    .text
                    .read(cx)
                    .anchor_bounds("short-document")
                    .is_some()
            })
        });
        reading.read_with(&vcx, |reading, _| {
            assert_eq!(
                reading.scroll.max_offset().y,
                px(0.),
                "a short document must not scroll"
            );
            assert_eq!(
                reading.scroll.offset().y,
                px(0.),
                "shrinking content must clamp the old offset"
            );
        });
    }

    #[gpui::test]
    fn fill_and_restore_preserve_a_partial_markdown_selection(cx: &mut TestAppContext) {
        let (app, mut vcx) = crate::ui::app::test_window::harness(cx);
        vcx.simulate_resize(gpui::size(px(1600.), px(900.)));
        let host = DocumentsHost::new(904, "Selection test", None);
        app.update_in(&mut vcx, |app, window, cx| {
            init(Default::default(), cx);
            app.tabs
                .push(crate::ui::app::Tab::new(crate::ui::pane::Pane::Empty));
            app.active = app.tabs.len() - 1;
            app.editor_open_on_host(host, Path::new("/repo/docs/readme.md"), window, cx);
        });
        settle(&mut vcx, |cx| {
            app.read_with(cx, |app, _| {
                app.tab_code()
                    .and_then(|code| code.active_file())
                    .is_some_and(|file| file.reading.is_some())
            })
        });
        let reading = app.read_with(&vcx, |app, _| {
            app.tab_code()
                .unwrap()
                .active_file()
                .unwrap()
                .reading
                .as_ref()
                .unwrap()
                .clone()
        });
        let text = reading.read_with(&vcx, |reading, _| reading.text.clone());
        settle(&mut vcx, |cx| {
            text.read_with(cx, |text, _| text.anchor_bounds("selection-test").is_some())
        });
        // The first prepaint records the reading width, which can change the
        // card's padding. Let that reflow finish before choosing a text hit.
        vcx.run_until_parked();
        vcx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        // Hit the heading itself, independently of reading-card padding.
        let position = text.read_with(&vcx, |text, _| {
            text.anchor_bounds("selection-test").unwrap().origin + point(px(15.), px(12.))
        });
        vcx.simulate_mouse_move(position, None, gpui::Modifiers::default());
        vcx.simulate_event(gpui::MouseDownEvent {
            position,
            modifiers: gpui::Modifiers::default(),
            button: gpui::MouseButton::Left,
            click_count: 2,
            first_mouse: false,
        });
        vcx.simulate_event(gpui::MouseUpEvent {
            position,
            modifiers: gpui::Modifiers::default(),
            button: gpui::MouseButton::Left,
            click_count: 2,
        });
        vcx.run_until_parked();
        assert_eq!(
            text.read_with(&vcx, |text, _| text.selected_text()).trim(),
            "Selection"
        );

        for layout in [
            crate::core::config::DocumentLayout::Fill,
            crate::core::config::DocumentLayout::Dock,
        ] {
            let button = vcx.debug_bounds("document-fill-toggle").unwrap();
            vcx.simulate_click(button.center(), gpui::Modifiers::default());
            vcx.run_until_parked();
            vcx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            assert_eq!(
                text.read_with(&vcx, |text, _| text.selected_text()).trim(),
                "Selection"
            );
            assert_eq!(
                app.read_with(&vcx, |app, cx| app.document_layout(cx)),
                layout
            );
        }
    }

    #[gpui::test]
    fn markdown_mermaid_theme_switch_replaces_svg_without_reparsing_or_losing_selection(
        cx: &mut TestAppContext,
    ) {
        let (app, mut vcx) = crate::ui::app::test_window::harness(cx);
        let content = "---\r\nname: show-me\r\n---\r\n\r\n# Diagram\r\n\r\nWords to select.\r\n\r\n```mermaid\r\nflowchart LR\r\n A --> B\r\n```\r\n";
        let mut host = DocumentsHost::new(905, "Diagram", None);
        Arc::get_mut(&mut host)
            .unwrap()
            .files
            .insert("/repo/docs/readme.md".into(), content.as_bytes().to_vec());
        app.update_in(&mut vcx, |app, window, cx| {
            init(Default::default(), cx);
            app.tabs
                .push(crate::ui::app::Tab::new(crate::ui::pane::Pane::Empty));
            app.active = app.tabs.len() - 1;
            app.editor_open_on_host(host.clone(), Path::new("/repo/docs/readme.md"), window, cx);
        });
        settle(&mut vcx, |cx| {
            app.read_with(cx, |app, _| {
                app.tab_code()
                    .and_then(|code| code.active_file())
                    .is_some_and(|file| file.reading.is_some())
            })
        });
        let reading = app.read_with(&vcx, |app, _| {
            app.tab_code()
                .unwrap()
                .active_file()
                .unwrap()
                .reading
                .clone()
                .unwrap()
        });
        let text = reading.read_with(&vcx, |reading, _| reading.text.clone());
        text.update(&mut vcx, |text, cx| text.select_all(cx));
        let parsed = text.read_with(&vcx, |text, _| text.source());
        assert!(parsed.starts_with("<table style=\"white-space: normal\">\n"));
        assert!(parsed.contains("ctty7-mermaid://0"));
        let selected = text.read_with(&vcx, |text, _| text.selected_text());
        let first = reading.read_with(
            &vcx,
            |reading, _| match &reading.diagrams["ctty7-mermaid://0"] {
                TextViewImageSource::Ready(gpui::ImageSource::Image(image)) => image.clone(),
                _ => panic!("diagram SVG must be ready"),
            },
        );
        let themes = tempfile::tempdir().unwrap();
        std::fs::write(themes.path().join("custom.yaml"), "schema_version: 2\nid: custom\nname: Custom\nlight: {paper: '#eeeeee'}\ndark: {paper: '#222222'}\n").unwrap();
        vcx.update(|_, cx| apply_snapshot(markdown_theme::scan(Some(themes.path())), cx));
        for id in ["custom", "github"] {
            app.update_in(&mut vcx, |app, _, cx| app.set_markdown_theme(id, cx));
            for _ in 0..3 {
                vcx.update(|window, cx| {
                    let _ = window.draw(cx);
                });
                vcx.run_until_parked();
            }
            assert_eq!(text.read_with(&vcx, |text, _| text.source()), parsed);
            assert_eq!(
                text.read_with(&vcx, |text, _| text.selected_text()),
                selected
            );
            reading.read_with(&vcx, |reading, cx| {
                assert_eq!(reading.source.read(cx).text().to_string(), content);
                assert!(reading.images.is_empty());
                let TextViewImageSource::Ready(gpui::ImageSource::Image(image)) =
                    &reading.diagrams["ctty7-mermaid://0"]
                else {
                    panic!("diagram")
                };
                assert!(!Arc::ptr_eq(image, &first));
            });
        }
        assert!(
            host.reads
                .lock()
                .unwrap()
                .iter()
                .all(|path| path.extension().is_some_and(|ext| ext == "md"))
        );
    }

    #[gpui::test]
    fn markdown_edits_release_image_budget_and_ignore_old_completions(cx: &mut TestAppContext) {
        let (app, mut vcx) = crate::ui::app::test_window::harness(cx);
        app.update_in(&mut vcx, |app, _, cx| {
            init(Default::default(), cx);
            app.tabs
                .push(crate::ui::app::Tab::new(crate::ui::pane::Pane::Empty));
            app.active = app.tabs.len() - 1;
        });
        let (release, gate) = mpsc::channel();
        let host = DocumentsHost::new(903, "Old image", Some(gate));
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_open_on_host(host.clone(), Path::new("/repo/docs/readme.md"), window, cx);
        });
        let image = Path::new("/repo/docs/images/icon.svg");
        settle(&mut vcx, |_| {
            host.reads.lock().unwrap().iter().any(|path| path == image)
        });
        let reading = app.read_with(&vcx, |app, _| {
            app.tab_code()
                .unwrap()
                .active_file()
                .unwrap()
                .reading
                .clone()
                .unwrap()
        });
        // Failed paths consume slots too. Fill the old revision's budget while
        // its real image is still blocked at the Host boundary.
        reading.update(&mut vcx, |reading, cx| {
            for n in 0..63 {
                reading.image_source(&format!("missing-{n}.svg").into(), cx);
            }
            assert_eq!(reading.images.len(), 64);
        });
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_toggle_preview(window, cx)
        });
        vcx.update(|_, cx| {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string("# No images".into()))
        });
        vcx.simulate_keystrokes("secondary-a secondary-v");
        vcx.run_until_parked();
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_toggle_preview(window, cx)
        });
        settle(&mut vcx, |cx| {
            reading.read_with(cx, |reading, _| reading.images.is_empty())
        });
        release.send(()).unwrap();
        settle(&mut vcx, |_| host.image_completed.load(Ordering::SeqCst));
        vcx.run_until_parked();
        reading.read_with(&vcx, |reading, _| {
            assert!(reading.images.is_empty());
            assert_eq!(reading.image_bytes, 0);
        });
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_toggle_preview(window, cx)
        });
        vcx.update(|_, cx| {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                "# New revision\n\n![image](images/icon.svg)".into(),
            ))
        });
        vcx.simulate_keystrokes("secondary-a secondary-v");
        vcx.run_until_parked();
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_toggle_preview(window, cx)
        });
        settle(&mut vcx, |cx| {
            reading.read_with(cx, |reading, _| {
                matches!(
                    reading.images.get(image),
                    Some(TextViewImageSource::Ready(_))
                )
            })
        });
        assert!(reading.read_with(&vcx, |reading, _| reading.image_bytes) > 0);
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_toggle_preview(window, cx)
        });
        vcx.update(|_, cx| {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string("# Empty again".into()))
        });
        vcx.simulate_keystrokes("secondary-a secondary-v");
        vcx.run_until_parked();
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_toggle_preview(window, cx)
        });
        settle(&mut vcx, |cx| {
            reading.read_with(cx, |reading, _| {
                reading.images.is_empty() && reading.image_bytes == 0
            })
        });
    }

    #[gpui::test]
    fn markdown_resources_use_the_owning_host_and_ignore_closed_document_completions(
        cx: &mut TestAppContext,
    ) {
        let (app, mut vcx) = crate::ui::app::test_window::harness(cx);
        app.update_in(&mut vcx, |app, _, cx| {
            init(Default::default(), cx);
            app.tabs
                .push(crate::ui::app::Tab::new(crate::ui::pane::Pane::Empty));
            app.active = app.tabs.len() - 1;
        });
        let (release, gate) = mpsc::channel();
        let first = DocumentsHost::new(901, "First host", Some(gate));
        let second = DocumentsHost::new(902, "Second host", None);
        let document = Path::new("/repo/docs/readme.md");
        let image = Path::new("/repo/docs/images/icon.svg");
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_open_on_host(first.clone(), document, window, cx)
        });
        settle(&mut vcx, |_| {
            first.reads.lock().unwrap().iter().any(|path| path == image)
        });
        let closed = app.read_with(&vcx, |app, _| {
            app.tab_code()
                .unwrap()
                .active_file()
                .unwrap()
                .reading
                .as_ref()
                .unwrap()
                .downgrade()
        });

        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_open_on_host(second.clone(), document, window, cx)
        });
        settle(&mut vcx, |cx| {
            app.read_with(cx, |app, _| app.tab_code().unwrap().files.len() == 2)
        });
        let reading = app.read_with(&vcx, |app, _| {
            app.tab_code()
                .unwrap()
                .active_file()
                .unwrap()
                .reading
                .clone()
                .unwrap()
        });
        settle(&mut vcx, |cx| {
            reading.read_with(cx, |reading, _| {
                matches!(
                    reading.images.get(image),
                    Some(TextViewImageSource::Ready(_))
                )
            })
        });
        reading.read_with(&vcx, |reading, cx| {
            assert!(reading.text.read(cx).source().contains("Second host"))
        });

        app.update_in(&mut vcx, |app, window, cx| {
            let index = app
                .tab_code()
                .unwrap()
                .files
                .iter()
                .position(|file| file.host.id() == first.id())
                .unwrap();
            app.editor_close_file(index, window, cx)
        });
        settle(&mut vcx, |_| closed.upgrade().is_none());
        release.send(()).unwrap();
        settle(&mut vcx, |_| first.image_completed.load(Ordering::SeqCst));
        vcx.run_until_parked();
        reading.read_with(&vcx, |reading, _| {
            let Some(TextViewImageSource::Ready(gpui::ImageSource::Image(data))) =
                reading.images.get(image)
            else {
                panic!("loaded host image");
            };
            assert!(String::from_utf8_lossy(&data.bytes).contains("Second host"));
            assert_eq!(reading.host.id(), second.id());
        });
        assert_eq!(
            second
                .reads
                .lock()
                .unwrap()
                .iter()
                .filter(|path| *path == image)
                .count(),
            1
        );
    }
}
