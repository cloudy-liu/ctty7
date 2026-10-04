// Immutable behavior fixture from ctty7@44aa4510; only Option adapters reflect the extended palette type.
use super::*;
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
                keyword: token(s.keyword),
                boolean: token(s.keyword),
                preproc: token(s.keyword),
                number: token(s.number),
                constant: token(s.number),
                function: token(s.function),
                constructor: token(s.function),
                variable: token(s.variable),
                variable_special: token(s.variable_special),
                property: token(s.variable_special),
                type_: token(s.r#type),
                enum_: token(s.r#type),
                variant: token(s.r#type),
                comment: token(s.comment),
                comment_doc: token(s.comment),
                string: token(s.string),
                string_escape: token(s.string),
                string_regex: token(s.string),
                string_special: token(s.string),
                string_special_symbol: token(s.string),
                tag: token(s.tag),
                tag_doctype: token(s.tag),
                attribute: token(s.variable_special),
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
        is_dark: dark,
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
        table_hover_background: Some(color(p.table_hover.unwrap())),
        code_block: refinement(
            div()
                .p(px(l.code_padding * scale))
                .rounded(px(l.code_radius * scale))
                .bg(color(p.code_background))
                .text_color(color(p.code_foreground))
                .border_1()
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
                .p(px(l.quote_padding * scale)),
        ),
        alert: refinement(
            div()
                .bg(color(p.alert_background))
                .text_color(color(p.alert_foreground.unwrap()))
                .border_color(color(p.alert_border)),
        ),
        nested_blockquote: refinement(
            div()
                .bg(color(p.nested_quote_background))
                .border_color(color(p.nested_quote_border)),
        ),
        table_cell: refinement(div().p(px(l.table_padding * scale)).border_r_0()),
        table_header: refinement(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(color(p.heading)),
        ),
        list_item: refinement(div().pb(px(l.list_gap * scale))),
        ..Default::default()
    };
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
                .text_color(color(p.heading))
                .mt(px(t.heading_gap * scale))
                .pb(px(t.paragraph_gap * scale * 0.5))
                .when(level == 1, |el| {
                    el.border_b_1().border_color(color(p.border))
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
