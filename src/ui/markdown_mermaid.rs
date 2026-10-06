//! Mermaid diagram rendering for the Markdown preview.
//!
//! Diagrams are rendered to SVG by the pure-Rust `merman` renderer. The
//! GitHub uses Mermaid's default/dark diagram themes; custom YAML themes
//! supply semantic colors from their reading palette.

use merman::render::{
    HeadlessRenderer, HostThemeAppearance, HostThemeOutput, HostThemeProfile, HostThemeRoles,
    HostThemeRootBackground,
};

use crate::core::markdown_theme::{Color, Theme};

use crate::ui::i18n::{L10nKey, t};
use gpui::{
    App, Context, ImageSource, ObjectFit, Render, ScrollHandle, StyledImage as _, Window, div, img,
    point, prelude::*, px,
};
use gpui_component::{
    ActiveTheme as _, IconName, Sizable as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
};

pub(super) fn open_diagram(image: ImageSource, window: &mut Window, cx: &mut App) {
    let width = (window.viewport_size().width * 0.8).min(px(1100.));
    let height = window.viewport_size().height * 0.6;
    let viewer = cx.new(|_| DiagramViewer {
        image,
        zoom: 1.,
        width: width - px(48.),
        height,
        scroll: ScrollHandle::default(),
    });
    window.open_dialog(cx, move |dialog, _, _| {
        let viewer = viewer.clone();
        dialog
            .width(width)
            .title("Mermaid")
            .content(move |content, _, _| content.child(viewer.clone()))
    });
}

struct DiagramViewer {
    image: ImageSource,
    zoom: f32,
    width: gpui::Pixels,
    height: gpui::Pixels,
    scroll: ScrollHandle,
}

impl Render for DiagramViewer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut controls = div()
            .flex()
            .gap_1()
            .child(
                Button::new("diagram-zoom-out")
                    .icon(IconName::Minus)
                    .ghost()
                    .xsmall()
                    .tooltip(t(L10nKey::MarkdownZoomOut))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.zoom = (this.zoom / 1.25).max(0.25);
                        cx.notify();
                    })),
            )
            .child(
                Button::new("diagram-zoom-in")
                    .icon(IconName::Plus)
                    .ghost()
                    .xsmall()
                    .tooltip(t(L10nKey::MarkdownZoomIn))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.zoom = (this.zoom * 1.25).min(8.);
                        cx.notify();
                    })),
            )
            .child(
                Button::new("diagram-reset")
                    .icon(IconName::Undo)
                    .ghost()
                    .xsmall()
                    .tooltip(t(L10nKey::Reset))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.zoom = 1.;
                        this.scroll.set_offset(point(px(0.), px(0.)));
                        cx.notify();
                    })),
            );
        for (id, icon, x, y) in [
            ("diagram-pan-left", IconName::ArrowLeft, 64., 0.),
            ("diagram-pan-right", IconName::ArrowRight, -64., 0.),
            ("diagram-pan-up", IconName::ArrowUp, 0., 64.),
            ("diagram-pan-down", IconName::ArrowDown, 0., -64.),
        ] {
            controls = controls.child(Button::new(id).icon(icon).ghost().xsmall().on_click(
                cx.listener(move |this, _, _, cx| {
                    let offset = this.scroll.offset();
                    let max = this.scroll.max_offset();
                    this.scroll.set_offset(point(
                        (offset.x + px(x)).clamp(-max.x, px(0.)),
                        (offset.y + px(y)).clamp(-max.y, px(0.)),
                    ));
                    cx.notify();
                }),
            ));
        }
        div().flex().flex_col().gap_2().child(controls).child(
            div()
                .id("diagram-viewport")
                .w_full()
                .h(self.height)
                .overflow_scroll()
                .track_scroll(&self.scroll)
                .bg(cx.theme().tokens.background)
                .child(
                    img(self.image.clone())
                        .w(self.width * self.zoom)
                        .object_fit(ObjectFit::Contain),
                ),
        )
    }
}

/// Render Mermaid `source` to an SVG string styled with `theme`'s palette.
pub fn render_mermaid(source: &str, theme: &Theme, dark: bool) -> Result<String, String> {
    HeadlessRenderer::new()
        .render_svg_with_host_theme_sync(source, &build_theme_profile(theme, dark))
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "not a recognized Mermaid diagram".to_string())
}

/// Map the Markdown theme palette onto merman's semantic theme roles.
fn build_theme_profile(theme: &Theme, dark: bool) -> HostThemeProfile {
    if theme.id == crate::core::markdown_theme::DEFAULT_ID {
        // GitHub uses Mermaid's own default/dark theme, rather than recoloring
        // diagram nodes with the surrounding Markdown paper palette.
        let mut output = HostThemeOutput::resvg_safe_editor();
        // Portable SVG output otherwise bakes in a white canvas in dark mode.
        // Keep Mermaid's node palette, but let the app paint the reading surface.
        output.root_background = HostThemeRootBackground::Color("transparent".into());
        return HostThemeProfile::builder()
            .output(output)
            .site_config("theme", if dark { "dark" } else { "default" })
            .build();
    }
    let p = theme.palette(dark);
    let hex = |c: &Color| Some(color_to_hex(c));

    HostThemeProfile::builder()
        .output(HostThemeOutput::resvg_safe_editor())
        .appearance(if dark {
            HostThemeAppearance::Dark
        } else {
            HostThemeAppearance::Light
        })
        .roles(HostThemeRoles {
            canvas: hex(&p.background),
            surface: hex(&p.paper),
            surface_alt: None,
            surface_muted: None,
            text: hex(&p.foreground),
            subtle_text: hex(&p.quote_foreground),
            border: hex(&p.border),
            line: hex(&p.border),
            edge_label_background: hex(&p.paper),
            cluster_background: hex(&p.paper),
            cluster_border: hex(&p.border),
            note_background: hex(&p.quote_background),
            note_border: hex(&p.quote_border),
            note_text: hex(&p.quote_foreground),
            actor_background: hex(&p.code_background),
            actor_border: hex(&p.border),
            actor_text: hex(&p.foreground),
            activation_background: hex(&p.accent),
            activation_border: hex(&p.accent),
            // Semantic colors stay fixed across themes.
            error: Some("#EF4444".to_string()),
            warning: Some("#F59E0B".to_string()),
            success: Some("#10B981".to_string()),
        })
        .build()
}

fn color_to_hex(color: &Color) -> String {
    let [r, g, b, a] = color.0.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8);
    if a == u8::MAX {
        format!("#{r:02X}{g:02X}{b:02X}")
    } else {
        format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagram_canvas_matches_the_builtin_and_custom_background_contract() {
        let custom = crate::core::markdown_theme::parse(
            "schema_version: 2\nid: diagram-background-test\nname: Diagram background test\nlight:\n  background: '#f2e9df'\n  paper: '#fffaf3'\ndark:\n  background: '#172a3a'\n  paper: '#20384d'\n",
        )
        .unwrap();
        let builtin = crate::core::markdown_theme::builtin();
        let background = regex::Regex::new(r#"background-color:\s*([^;"<>]+)"#).unwrap();
        for (name, source) in [
            (
                "flowchart",
                "flowchart LR\n A[Start] -->|Continue| B[Finish]",
            ),
            (
                "sequence",
                "sequenceDiagram\n Alice->>Bob: Hello\n Bob-->>Alice: Reply",
            ),
            (
                "cluster",
                "flowchart LR\n subgraph Group\n A[Start] --> B[Finish]\n end",
            ),
        ] {
            for dark in [false, true] {
                for (theme, expected) in [
                    (builtin.as_ref(), "transparent".to_string()),
                    (&custom, color_to_hex(&custom.palette(dark).background)),
                ] {
                    let svg = render_mermaid(source, theme, dark).unwrap();
                    let root = &svg[..svg.find('>').unwrap()];
                    let actual = background.captures(root).map(|c| c[1].trim().to_string());
                    assert_eq!(
                        actual.as_deref(),
                        Some(expected.as_str()),
                        "{name}, theme={}, dark={dark}, root={root}",
                        theme.id
                    );
                }
            }
        }
    }

    #[test]
    fn diagram_labels_are_portable_to_the_native_svg_renderer() {
        let svg = render_mermaid(
            "flowchart LR\n A[Visible label] --> B[Another label]",
            &crate::core::markdown_theme::builtin(),
            false,
        )
        .unwrap();
        assert!(
            svg.contains("<text"),
            "native SVG rendering requires portable text for HTML node labels"
        );
        assert!(
            svg.contains("Visible label"),
            "node labels must remain in the SVG"
        );
    }

    #[test]
    fn color_to_hex_formats_opaque_and_translucent() {
        assert_eq!(color_to_hex(&Color([1.0, 0.0, 0.5, 1.0])), "#FF0080");
        assert_eq!(color_to_hex(&Color([0.0, 0.0, 0.0, 0.5])), "#00000080");
    }
}
