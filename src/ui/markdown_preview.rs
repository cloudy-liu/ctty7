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
    ActiveTheme as _, ElementExt as _, IconName, Sizable as _, ThemeMode,
    button::{Button, ButtonVariants as _},
    highlighter::{
        FontWeightContent, HighlightTheme, HighlightThemeStyle, SyntaxColors, ThemeStyle,
    },
    input::InputState,
    text::{
        AlertStyle, InlineCodeStyle, KeyboardStyle, LinkUnderline, TextView, TextViewState,
        TextViewStyle,
    },
};

use crate::core::markdown_document::{self, Target};
use crate::core::{
    config::Config,
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
        (true, "paperglow") => t(L10nKey::SettingsMarkdownPaperglow).to_owned(),
        _ => entry.theme.name.clone(),
    }
}

pub(crate) fn theme_description(entry: &Entry) -> String {
    match (entry.source.is_none(), entry.theme.id.as_str()) {
        (true, "github") => t(L10nKey::SettingsMarkdownGithubDesc).to_owned(),
        (true, "paperglow") => t(L10nKey::SettingsMarkdownPaperglowDesc).to_owned(),
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
                diff_header: s
                    .markup_heading
                    .map(|c| ThemeStyle::from(color(c)).weight(FontWeightContent::Bold)),
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

#[cfg(test)]
#[path = "../../tests/fixtures/markdown_preview_before_github.rs"]
mod before_github;

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
    available_fonts: Option<Vec<String>>,
    mermaid_enabled: bool,
    _subscriptions: Vec<Subscription>,
}

impl MarkdownPreview {
    pub(crate) fn new(
        source: Entity<InputState>,
        host: SharedHost,
        path: PathBuf,
        app: gpui::WeakEntity<Tty7App>,
        mermaid_enabled: bool,
        cx: &mut Context<Self>,
    ) -> Self {
        let content = source.read(cx).text().to_string();
        let (processed_content, diagrams) = if mermaid_enabled {
            Self::preprocess_mermaid(&content, cx)
        } else {
            (content, HashMap::new())
        };
        let text = cx.new(|cx| {
            TextViewState::markdown_with_extensions(&processed_content, extensions(), cx)
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
        Self {
            source,
            host,
            path,
            app,
            images: HashMap::new(),
            diagrams,
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
            _subscriptions: subscriptions,
        }
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
        for diagram in self.diagrams.values() {
            if let TextViewImageSource::Ready(source) = diagram {
                source.remove_asset(cx);
            }
        }
        let (processed_content, diagrams) = if self.mermaid_enabled {
            Self::preprocess_mermaid(&content, cx)
        } else {
            (content, HashMap::new())
        };
        self.diagrams = diagrams;
        self.text
            .update(cx, |state, cx| state.set_text(&processed_content, cx));
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
        let path =
            match markdown_document::resolve(&self.path, url.as_ref(), self.host.id().is_local()) {
                Ok(Target::Web(url)) => {
                    return TextViewImageSource::Ready(gpui::SharedUri::from(url).into());
                }
                Ok(Target::File { path, .. }) => path,
                Ok(Target::Anchor(_)) => {
                    return TextViewImageSource::Failed("image has no resource path".into());
                }
                Err(error) => return TextViewImageSource::Failed(error.into()),
            };
        if let Some(image) = self.images.get(&path) {
            return image.clone();
        }
        if self.images.len() >= 64 {
            return TextViewImageSource::Failed("document image limit reached (64)".into());
        }
        self.images
            .insert(path.clone(), TextViewImageSource::Loading);
        let requested = path.clone();
        let revision = self.revision;
        HostOps::run(
            self.host.clone(),
            cx,
            move |host| {
                let metadata = host.stat(&requested).map_err(|e| e.to_string())?;
                if metadata.len > markdown_document::MAX_IMAGE_BYTES {
                    return Err("image exceeds 8 MiB".to_string());
                }
                let bytes = host
                    .read_file(&requested, markdown_document::MAX_IMAGE_BYTES)
                    .map_err(|e| e.to_string())?;
                let format = markdown_document::image_format(&bytes)?;
                Ok((format, bytes))
            },
            move |this, result, cx| {
                if this.revision != revision {
                    return;
                }
                let image = match result {
                    Ok((format, bytes)) if this.image_bytes + bytes.len() <= 32 * 1024 * 1024 => {
                        this.image_bytes += bytes.len();
                        TextViewImageSource::Ready(gpui::ImageSource::Image(Arc::new(
                            gpui::Image::from_bytes(format, bytes),
                        )))
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
    fn preprocess_mermaid(
        content: &str,
        cx: &mut Context<Self>,
    ) -> (String, HashMap<String, TextViewImageSource>) {
        static MERMAID_FENCE: LazyLock<regex::Regex> = LazyLock::new(|| {
            regex::Regex::new(r"(?m)^```mermaid[ \t]*\n([\s\S]*?)^```")
                .expect("mermaid fence regex is valid")
        });

        if !MERMAID_FENCE.is_match(content) {
            return (content.to_string(), HashMap::new());
        }
        let theme = current(cx).theme;
        let dark = cx.theme().mode.is_dark();

        let mut diagrams = HashMap::new();
        let processed = MERMAID_FENCE
            .replace_all(content, |caps: &regex::Captures| {
                let source = &caps[1];
                match crate::ui::markdown_mermaid::render_mermaid(source, &theme, dark) {
                    Ok(svg) => {
                        let url = format!("ctty7-mermaid://{}", diagrams.len());
                        diagrams.insert(
                            url.clone(),
                            TextViewImageSource::Ready(gpui::ImageSource::Image(Arc::new(
                                gpui::Image::from_bytes(gpui::ImageFormat::Svg, svg.into_bytes()),
                            ))),
                        );
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
        let available = self
            .available_fonts
            .get_or_insert_with(|| window.text_system().all_font_names());
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
                let (processed, diagrams) = Self::preprocess_mermaid(&content, cx);
                for diagram in self.diagrams.values() {
                    if let TextViewImageSource::Ready(source) = diagram {
                        source.remove_asset(cx);
                    }
                }
                self.diagrams = diagrams;
                // Stable diagram URLs retain parsed text, byte selections and
                // source revision while the underlying SVG changes palette.
                self.text.update(cx, |text, cx| {
                    if text.source().as_ref() != processed.as_str() {
                        text.set_text(&processed, cx);
                    }
                });
            }
            self.style_key = Some(key);
        }
        let palette = entry.theme.palette(dark);
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
            .code_block_actions(move |block, _, _| {
                let code = block.code();
                Button::new("copy-code")
                    .icon(IconName::Copy)
                    .ghost()
                    .xsmall()
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
            .text_color(color(palette.foreground))
            .bg(color(palette.paper));
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
            .child(text);
        let entity = cx.weak_entity();
        let scroll = self.scroll.clone();
        crate::ui::scrollbar::with_vertical_scrollbar(
            "markdown-reading-scrollbar",
            div()
                .id("markdown-reading")
                .size_full()
                .key_context("MarkdownPreview")
                .bg(color(palette.background))
                .overflow_y_scroll()
                .track_scroll(&scroll)
                .p(px(outer))
                .on_key_down(cx.listener(Self::on_key_down))
                .child(card)
                .on_prepaint(move |_, window, cx| {
                    let _ = entity.update(cx, |this, cx| {
                        // The helper canvas is a scrolling child. The handle
                        // supplies the fixed viewport in window coordinates.
                        let bounds = this.scroll.bounds();
                        if this.width != bounds.size.width {
                            this.width = bounds.size.width;
                            this.restore_position = this.restore_position.or(this.last_position);
                            cx.notify();
                            return;
                        }
                        if let Some(anchor) = this.pending_anchor.take() {
                            this.restore_position = None;
                            if anchor.is_empty() {
                                this.scroll.set_offset(point(px(0.), px(0.)));
                            } else if let Some(target) = this.text.read(cx).anchor_bounds(&anchor) {
                                let offset = this.scroll.offset();
                                let y = offset.y - (target.top() - bounds.top()) + px(12.);
                                this.scroll.set_offset(point(
                                    offset.x,
                                    y.clamp(-this.scroll.max_offset().y, px(0.)),
                                ));
                            } else {
                                window.push_notification(
                                    format!("#{anchor}: {}", t(L10nKey::MarkdownAnchorMissing)),
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
                        this.last_position = this.text.read(cx).reading_position(bounds.top());
                    });
                }),
            &scroll,
        )
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
    fn paperglow_and_existing_blue_paper_keep_the_pre_github_reading_style() {
        for source in [
            markdown_theme::PAPERGLOW,
            include_str!("../../tests/fixtures/paperglow-before-github.yaml"),
            include_str!("../../docs/examples/markdown-themes/blue-paper.yaml"),
        ] {
            let entry = Entry {
                theme: Arc::new(markdown_theme::parse(source).unwrap()),
                source: None,
                revision: 0,
            };
            for dark in [false, true] {
                for scale in [1., 1.5, 2.] {
                    assert!(
                        reading_style(&entry, dark, scale, "Mono".into())
                            == before_github::reading_style(&entry, dark, scale, "Mono".into()),
                        "{} dark={dark} scale={scale}",
                        entry.theme.id
                    );
                }
            }
        }
    }

    #[test]
    fn markdown_heading_fonts_are_optional_and_independent_of_body_and_code() {
        let yaml = "schema_version: 1\nid: fonts\nname: Fonts\nlight: {}\ndark: {}\ntypography:\n  fonts: [Body]\n  code_fonts: [Code]\n";
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
        let position = reading.read_with(&vcx, |reading, _| {
            reading.scroll.bounds().origin + point(px(30.), px(28.))
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
        let content = "# Diagram\n\nWords to select.\n\n```mermaid\nflowchart LR\n A --> B\n```\n";
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
        assert!(parsed.contains("ctty7-mermaid://0"));
        let selected = text.read_with(&vcx, |text, _| text.selected_text());
        let first = reading.read_with(
            &vcx,
            |reading, _| match &reading.diagrams["ctty7-mermaid://0"] {
                TextViewImageSource::Ready(gpui::ImageSource::Image(image)) => image.clone(),
                _ => panic!("diagram SVG must be ready"),
            },
        );
        for id in ["paperglow", "github"] {
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
