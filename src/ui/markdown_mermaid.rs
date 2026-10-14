//! Mermaid diagram rendering and native viewport controls.

use crate::core::markdown_theme::{Color, Theme};
use crate::ui::i18n::{L10nKey, t};
use gpui::{
    App, AvailableSpace, Context, ImageSource, MouseButton, ObjectFit, Pixels, Point, Render,
    ScrollDelta, ScrollWheelEvent, SharedString, StyledImage as _, Window, canvas, div, img, point,
    prelude::*, px,
};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, WindowExt as _,
    button::{Button, ButtonVariants as _},
};
use merman::render::{
    HeadlessRenderer, HostThemeAppearance, HostThemeOutput, HostThemeProfile, HostThemeRoles,
    HostThemeRootBackground,
};

pub(super) fn aspect_ratio(svg: &str) -> f32 {
    let ratio = || {
        let doc = resvg::usvg::roxmltree::Document::parse(svg).ok()?;
        let mut values = doc
            .root_element()
            .attribute("viewBox")?
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|s| !s.is_empty());
        let width: f32 = values.nth(2)?.parse().ok()?;
        let height: f32 = values.next()?.parse().ok()?;
        let ratio = width / height;
        (width > 0. && height > 0. && ratio.is_finite() && ratio > 0.).then_some(ratio)
    };
    ratio().unwrap_or(2.)
}

pub(crate) fn open_diagram(
    image: ImageSource,
    code: SharedString,
    aspect: f32,
    window: &mut Window,
    cx: &mut App,
) -> gpui::Entity<DiagramViewer> {
    let width = (window.viewport_size().width * 0.95).min(px(1300.));
    let viewer = cx.new(|_| DiagramViewer::new(image, code, aspect, true));
    let dialog_viewer = viewer.clone();
    window.open_dialog(cx, move |dialog, _, _| {
        let viewer = dialog_viewer.clone();
        dialog
            .width(width)
            .title("Mermaid")
            .content(move |content, _, _| content.child(viewer.clone()))
    });
    let focus = viewer.update(cx, |viewer, cx| {
        viewer
            .focus
            .get_or_insert_with(|| cx.focus_handle())
            .clone()
    });
    window.focus(&focus, cx);
    viewer
}

pub(crate) struct DiagramViewer {
    image: ImageSource,
    code: SharedString,
    aspect: f32,
    expanded: bool,
    zoom: f32,
    pan: Point<Pixels>,
    width: Pixels,
    image_width: Pixels,
    natural_width: Pixels,
    height: Pixels,
    drag: Option<Point<Pixels>>,
    drag_start: Point<Pixels>,
    dragged: bool,
    focus: Option<gpui::FocusHandle>,
    viewport: gpui::Bounds<Pixels>,
    #[cfg(any(test, feature = "markdown-visual-tests"))]
    pub(super) rendered_transform: (f32, Point<Pixels>),
}

impl DiagramViewer {
    pub(super) fn new(image: ImageSource, code: SharedString, aspect: f32, expanded: bool) -> Self {
        Self {
            image,
            code,
            aspect,
            expanded,
            zoom: 1.,
            pan: point(px(0.), px(0.)),
            width: px(0.),
            image_width: px(0.),
            natural_width: px(0.),
            height: px(0.),
            drag: None,
            drag_start: Default::default(),
            dragged: false,
            focus: None,
            viewport: Default::default(),
            #[cfg(any(test, feature = "markdown-visual-tests"))]
            rendered_transform: (1., point(px(0.), px(0.))),
        }
    }

    #[cfg(feature = "markdown-visual-tests")]
    pub(crate) fn prepare_profile(&mut self, cx: &mut Context<Self>) {
        self.zoom = 4.;
        cx.notify();
    }

    #[cfg(feature = "markdown-visual-tests")]
    pub(crate) fn profile_bounds(&self) -> gpui::Bounds<Pixels> {
        self.viewport
    }

    #[cfg(feature = "markdown-visual-tests")]
    pub(crate) fn profile_pan(&self) -> Point<Pixels> {
        self.pan
    }

    #[cfg(feature = "markdown-visual-tests")]
    pub(crate) fn verify_profile_frame(&self) {
        assert_eq!(
            self.rendered_transform,
            (self.zoom, self.pan),
            "the drawn transform must follow the input; a stale cached frame is not a fast frame"
        );
    }

    fn pan_by(&mut self, delta: Point<Pixels>) -> bool {
        let old = self.pan;
        // Keep fitted diagrams movable without losing the zoomed edge limits.
        let max_x = (self.image_width * (self.zoom - 1.) / 2.).max(self.image_width / 2.);
        let max_y = (self.height * (self.zoom - 1.) / 2.).max(self.height / 2.);
        self.pan = point(
            (self.pan.x + delta.x).clamp(-max_x, max_x),
            (self.pan.y + delta.y).clamp(-max_y, max_y),
        );
        self.pan != old
    }

    fn zoom_by(&mut self, factor: f32) {
        self.zoom_at(factor, self.viewport.center());
    }

    fn zoom_at(&mut self, factor: f32, position: Point<Pixels>) {
        let previous = self.zoom;
        self.zoom = (self.zoom * factor).clamp(0.25, 8.);
        let anchor = position - self.viewport.center();
        self.pan = anchor + (self.pan - anchor) * (self.zoom / previous);
        self.pan_by(point(px(0.), px(0.)));
    }

    fn reset(&mut self) {
        self.zoom = 1.;
        self.pan = point(px(0.), px(0.));
    }

    fn on_key_down(&mut self, event: &gpui::KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        if event.keystroke.modifiers.control
            || event.keystroke.modifiers.platform
            || event.keystroke.modifiers.alt
        {
            return;
        }
        let key = event.keystroke.key.as_str();
        match key {
            "+" | "=" => self.zoom_by(1.25),
            "-" => self.zoom_by(1. / 1.25),
            "0" | "home" if self.expanded || self.zoom != 1. => self.reset(),
            "up" if self.expanded || self.zoom > 1. => {
                self.pan_by(point(px(0.), px(64.)));
            }
            "down" if self.expanded || self.zoom > 1. => {
                self.pan_by(point(px(0.), px(-64.)));
            }
            "left" if self.expanded || self.zoom > 1. => {
                self.pan_by(point(px(64.), px(0.)));
            }
            "right" if self.expanded || self.zoom > 1. => {
                self.pan_by(point(px(-64.), px(0.)));
            }
            "pageup" if self.expanded => {
                self.pan_by(point(px(0.), self.height * 0.9));
            }
            "pagedown" | "space" if self.expanded => {
                self.pan_by(point(px(0.), -self.height * 0.9));
            }
            "end" if self.expanded => {
                self.pan_by(point(px(0.), px(-1_000_000.)));
            }
            _ => return,
        }
        cx.stop_propagation();
        cx.notify();
    }

    fn control(id: &'static str, icon: impl Into<Icon>, label: &'static str, cx: &App) -> Button {
        let theme = super::markdown_preview::current(cx).theme;
        let palette = theme.palette(cx.theme().mode.is_dark());
        Button::new(id)
            .debug_selector(move || id.into())
            .icon(icon)
            .ghost()
            .border_1()
            .bg(super::markdown_preview::color(palette.code_background))
            .border_color(super::markdown_preview::color(palette.border))
            .text_color(super::markdown_preview::color(palette.muted))
            .tooltip(label)
    }
}

impl Render for DiagramViewer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle()).clone();
        #[cfg(any(test, feature = "markdown-visual-tests"))]
        {
            self.rendered_transform = (self.zoom, self.pan);
        }
        let scale = window.rem_size() / px(16.);
        if self.natural_width == px(0.) {
            let mut probe = img(self.image.clone()).into_any_element();
            self.natural_width = probe
                .layout_as_root(AvailableSpace::min_size(), window, cx)
                .width;
        }
        let width = if self.width > px(0.) {
            self.width
        } else {
            window.viewport_size().width - px(64.)
        };
        let available = (width - px(120. * scale)).max(px(64. * scale));
        let image_width = if self.natural_width > px(0.) && !self.expanded {
            available.min(self.natural_width)
        } else {
            available
        };
        self.image_width = image_width;
        self.height = if self.expanded {
            window.viewport_size().height * 0.65
        } else {
            (image_width / self.aspect).clamp(px(200. * scale), px(600. * scale))
        };
        let controls = [
            (
                "diagram-pan-up",
                IconName::ChevronUp,
                L10nKey::MarkdownPanUp,
                0.,
                64.,
            ),
            (
                "diagram-pan-left",
                IconName::ChevronLeft,
                L10nKey::MarkdownPanLeft,
                64.,
                0.,
            ),
            (
                "diagram-pan-right",
                IconName::ChevronRight,
                L10nKey::MarkdownPanRight,
                -64.,
                0.,
            ),
            (
                "diagram-pan-down",
                IconName::ChevronDown,
                L10nKey::MarkdownPanDown,
                0.,
                -64.,
            ),
        ]
        .map(|(id, icon, label, x, y)| {
            Self::control(id, icon, t(label), cx).on_click(cx.listener(move |this, _, _, cx| {
                cx.stop_propagation();
                if this.pan_by(point(px(x * scale), px(y * scale))) {
                    cx.notify();
                }
            }))
        });
        let [up, left, right, down] = controls;
        let zoom_in = Self::control(
            "diagram-zoom-in",
            Icon::default().path("icons/diagram-zoom-in.svg"),
            t(L10nKey::MarkdownZoomIn),
            cx,
        )
        .on_click(cx.listener(|this, _, _, cx| {
            cx.stop_propagation();
            this.zoom_by(1.25);
            cx.notify();
        }));
        let zoom_out = Self::control(
            "diagram-zoom-out",
            Icon::default().path("icons/diagram-zoom-out.svg"),
            t(L10nKey::MarkdownZoomOut),
            cx,
        )
        .on_click(cx.listener(|this, _, _, cx| {
            cx.stop_propagation();
            this.zoom_by(1. / 1.25);
            cx.notify();
        }));
        let reset = Self::control(
            "diagram-reset",
            Icon::default().path("icons/diagram-reset.svg"),
            t(L10nKey::Reset),
            cx,
        )
        .on_click(cx.listener(|this, _, _, cx| {
            cx.stop_propagation();
            this.reset();
            cx.notify();
        }));
        let toolbar = div()
            .flex()
            .gap_2()
            .when(!self.expanded, |el| {
                el.child(
                    Self::control(
                        "expand-diagram",
                        Icon::default().path("icons/diagram-expand.svg"),
                        t(L10nKey::MarkdownExpandDiagram),
                        cx,
                    )
                    .on_click(cx.listener(|this, _, window, cx| {
                        cx.stop_propagation();
                        open_diagram(
                            this.image.clone(),
                            this.code.clone(),
                            this.aspect,
                            window,
                            cx,
                        );
                    })),
                )
            })
            .child(
                Self::control(
                    "copy-diagram",
                    IconName::Copy,
                    t(L10nKey::EditorCopyCode),
                    cx,
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    cx.stop_propagation();
                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(this.code.to_string()));
                })),
            );
        let owner = cx.weak_entity();
        div()
            .id("diagram-viewport")
            .debug_selector(|| "diagram-viewport".into())
            .relative()
            .track_focus(&focus)
            .key_context("DiagramViewer")
            .cursor(if self.drag.is_some() {
                gpui::CursorStyle::ClosedHand
            } else {
                gpui::CursorStyle::OpenHand
            })
            .w_full()
            .h(self.height)
            .overflow_hidden()
            .when(self.expanded, |el| el.bg(cx.theme().tokens.background))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &gpui::MouseDownEvent, window, cx| {
                    cx.stop_propagation();
                    this.drag = Some(event.position);
                    this.drag_start = event.position;
                    this.dragged = false;
                    window.focus(this.focus.as_ref().unwrap(), cx);
                    cx.notify();
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, _, cx| {
                if event.pressed_button != Some(MouseButton::Left) {
                    this.drag = None;
                    return;
                }
                if let Some(previous) = this.drag {
                    let distance = event.position - this.drag_start;
                    this.dragged |= distance.x.abs() > px(3.) || distance.y.abs() > px(3.);
                    this.drag = Some(event.position);
                    let moved = this.pan_by(event.position - previous);
                    cx.stop_propagation();
                    if moved {
                        cx.notify();
                    }
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.drag = None;
                    cx.notify();
                }),
            )
            .on_key_down(cx.listener(Self::on_key_down))
            .when(!self.expanded, |el| {
                el.on_click(cx.listener(|this, _, window, cx| {
                    if !this.dragged {
                        cx.stop_propagation();
                        open_diagram(
                            this.image.clone(),
                            this.code.clone(),
                            this.aspect,
                            window,
                            cx,
                        );
                    }
                }))
            })
            .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, _, cx| {
                let delta = match event.delta {
                    ScrollDelta::Pixels(delta) => delta,
                    ScrollDelta::Lines(delta) => point(px(delta.x * 20.), px(delta.y * 20.)),
                };
                if delta.y != px(0.) && (event.modifiers.control || event.modifiers.platform) {
                    this.zoom_at(1.1_f32.powf(f32::from(delta.y) / 40.), event.position);
                    cx.stop_propagation();
                    cx.notify();
                } else if this.expanded || this.zoom > 1. {
                    let delta = if event.modifiers.shift && delta.x == px(0.) {
                        point(delta.y, px(0.))
                    } else {
                        delta
                    };
                    if !this.pan_by(delta) && !this.expanded {
                        return;
                    }
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
            .child(
                div()
                    .id("diagram-content")
                    .debug_selector(|| "diagram-content".into())
                    .absolute()
                    .left(self.pan.x + (width - image_width * self.zoom) / 2.)
                    .top(self.pan.y + self.height * (1. - self.zoom) / 2.)
                    .w(image_width * self.zoom)
                    .h(self.height * self.zoom)
                    .child(
                        img(self.image.clone())
                            .id("diagram-image")
                            .debug_selector(|| "diagram-image".into())
                            .absolute()
                            .min_w_0()
                            .min_h_0()
                            .w(image_width * self.zoom)
                            .h(self.height * self.zoom)
                            .object_fit(ObjectFit::Contain),
                    ),
            )
            .child(
                div()
                    .absolute()
                    .top_2()
                    .right_2()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(toolbar),
            )
            .child(
                div()
                    .absolute()
                    .bottom_2()
                    .right_2()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(
                        div()
                            .flex()
                            .gap_1()
                            .child(div().w_8())
                            .child(up)
                            .child(zoom_in),
                    )
                    .child(div().flex().gap_1().child(left).child(reset).child(right))
                    .child(
                        div()
                            .flex()
                            .gap_1()
                            .child(div().w_8())
                            .child(down)
                            .child(zoom_out),
                    ),
            )
            .child(
                canvas(
                    move |bounds, _, cx| {
                        let _ = owner.update(cx, |this, cx| {
                            this.viewport = bounds;
                            if this.width != bounds.size.width {
                                this.width = bounds.size.width;
                                cx.notify();
                            }
                        });
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full(),
            )
    }
}

/// Render Mermaid `source` to an SVG string styled with `theme`'s palette.
pub fn render_mermaid(source: &str, theme: &Theme, dark: bool) -> Result<String, String> {
    let mut renderer = HeadlessRenderer::new();
    if theme.id == crate::core::markdown_theme::DEFAULT_ID
        && renderer
            .parse_metadata_sync(source)
            .map_err(|err| err.to_string())?
            .is_some_and(|meta| meta.diagram_type.starts_with("flowchart"))
    {
        renderer = renderer.with_text_measurer(std::sync::Arc::new(GithubFlowchartText::default()));
    }
    renderer
        .render_svg_with_host_theme_sync(source, &build_theme_profile(theme, dark))
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "not a recognized Mermaid diagram".to_string())
}

/// GitHub's ordinary HTML flowchart labels use nowrap in a capped box.
/// Markdown labels use the renderer's raw measurement path and still wrap.
#[derive(Default)]
struct GithubFlowchartText(merman::render::VendoredFontMetricsTextMeasurer);

impl merman::render::TextMeasurer for GithubFlowchartText {
    fn measure(
        &self,
        text: &str,
        style: &merman_render::text::TextStyle,
    ) -> merman_render::text::TextMetrics {
        self.0.measure(text, style)
    }

    fn measure_wrapped(
        &self,
        text: &str,
        style: &merman_render::text::TextStyle,
        max_width: Option<f64>,
        mode: merman_render::text::WrapMode,
    ) -> merman_render::text::TextMetrics {
        if mode == merman_render::text::WrapMode::HtmlLike {
            let mut metrics = self.0.measure_wrapped(text, style, None, mode);
            metrics.width = metrics.width.min(max_width.unwrap_or(200.));
            metrics
        } else {
            self.0.measure_wrapped(text, style, max_width, mode)
        }
    }

    fn measure_wrapped_raw(
        &self,
        text: &str,
        style: &merman_render::text::TextStyle,
        max_width: Option<f64>,
        mode: merman_render::text::WrapMode,
    ) -> merman_render::text::TextMetrics {
        self.0.measure_wrapped_raw(text, style, max_width, mode)
    }
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
            .site_config("flowchart", serde_json::json!({ "diagramPadding": 48 }))
            .site_config("sequence", serde_json::json!({ "diagramMarginY": 40 }))
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

    #[gpui::test]
    fn mermaid_unmodified_wheel_pans_a_tall_diagram_to_the_bottom(cx: &mut gpui::TestAppContext) {
        let (_, mut vcx) = crate::ui::app::test_window::harness(cx);
        vcx.simulate_resize(gpui::size(px(1000.), px(800.)));
        vcx.update(|window, cx| {
            super::super::markdown_preview::init(Default::default(), cx);
            let source =
                "flowchart TD\nA[First] --> B[Second] --> C[Third] --> D[Fourth] --> E[Last]";
            let svg =
                render_mermaid(source, &crate::core::markdown_theme::builtin(), false).unwrap();
            open_diagram(
                ImageSource::Image(std::sync::Arc::new(gpui::Image::from_bytes(
                    gpui::ImageFormat::Svg,
                    svg.as_bytes().to_vec(),
                ))),
                source.into(),
                aspect_ratio(&svg),
                window,
                cx,
            );
        });
        vcx.run_until_parked();
        vcx.update(|window, cx| {
            window.refresh();
            let _ = window.draw(cx);
        });
        let zoom_button = vcx.debug_bounds("diagram-zoom-in").unwrap();
        vcx.simulate_click(zoom_button.center(), gpui::Modifiers::none());
        vcx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let zoomed = vcx.debug_bounds("diagram-content").unwrap();
        let viewport = vcx.debug_bounds("diagram-viewport").unwrap();
        vcx.simulate_event(ScrollWheelEvent {
            position: viewport.center(),
            delta: ScrollDelta::Pixels(point(px(0.), px(-10000.))),
            modifiers: gpui::Modifiers::none(),
            touch_phase: gpui::TouchPhase::Moved,
        });
        vcx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let scrolled = vcx.debug_bounds("diagram-content").unwrap();
        assert_eq!(
            scrolled.size, zoomed.size,
            "wheel down must navigate the enlarged diagram without shrinking it"
        );
        assert!(
            scrolled.origin.y < zoomed.origin.y,
            "wheel down must move toward the diagram's bottom"
        );
        let viewport = vcx.debug_bounds("diagram-viewport").unwrap();
        assert!(
            scrolled.bottom() <= viewport.bottom() + px(1.),
            "the bottom of the enlarged diagram must be reachable"
        );
    }

    #[gpui::test]
    fn expanded_mermaid_buttons_and_wheel_repaint_the_diagram(cx: &mut gpui::TestAppContext) {
        let (_, mut vcx) = crate::ui::app::test_window::harness(cx);
        vcx.simulate_resize(gpui::size(px(1000.), px(800.)));
        vcx.update(|window, cx| {
            super::super::markdown_preview::init(Default::default(), cx);
            let source = "graph TD; A-->B";
            let svg =
                render_mermaid(source, &crate::core::markdown_theme::builtin(), false).unwrap();
            let aspect = aspect_ratio(&svg);
            open_diagram(
                ImageSource::Image(std::sync::Arc::new(gpui::Image::from_bytes(
                    gpui::ImageFormat::Svg,
                    svg.into_bytes(),
                ))),
                source.into(),
                aspect,
                window,
                cx,
            );
        });
        vcx.run_until_parked();
        vcx.update(|window, cx| {
            window.refresh();
            let _ = window.draw(cx);
        });
        let fitted = vcx.debug_bounds("diagram-content").unwrap();
        let viewport = vcx.debug_bounds("diagram-viewport").unwrap();
        let zoom = vcx.debug_bounds("diagram-zoom-in").unwrap();
        let reset = vcx.debug_bounds("diagram-reset").unwrap();
        vcx.simulate_click(zoom.center(), gpui::Modifiers::none());
        vcx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        assert_eq!(
            vcx.debug_bounds("diagram-content").unwrap().size.width,
            fitted.size.width * 1.25
        );
        vcx.simulate_event(ScrollWheelEvent {
            position: viewport.origin + point(px(32.), px(64.)),
            delta: ScrollDelta::Lines(point(0., 1.)),
            modifiers: gpui::Modifiers {
                control: true,
                ..gpui::Modifiers::none()
            },
            touch_phase: gpui::TouchPhase::Moved,
        });
        vcx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        assert!(
            vcx.debug_bounds("diagram-content").unwrap().size.width > fitted.size.width * 1.25,
            "wheel must enlarge the expanded diagram"
        );
        for (delta, expected) in [(20_000., 8.), (-20_000., 0.25)] {
            vcx.simulate_event(ScrollWheelEvent {
                position: viewport.origin + point(px(32.), px(64.)),
                delta: ScrollDelta::Pixels(point(px(0.), px(delta))),
                modifiers: gpui::Modifiers {
                    control: true,
                    ..gpui::Modifiers::none()
                },
                touch_phase: gpui::TouchPhase::Moved,
            });
            vcx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            assert_eq!(
                vcx.debug_bounds("diagram-content").unwrap().size.width,
                fitted.size.width * expected
            );
        }
        vcx.simulate_click(reset.center(), gpui::Modifiers::none());
        vcx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let restored = vcx.debug_bounds("diagram-content").unwrap();
        assert_eq!(restored.size, fitted.size);
        // The dialog can still be animating; reset is relative to its viewport.
        assert_eq!(
            restored.origin,
            vcx.debug_bounds("diagram-viewport").unwrap().origin
                + point(
                    (vcx.debug_bounds("diagram-viewport").unwrap().size.width
                        - restored.size.width)
                        / 2.,
                    px(0.)
                )
        );
    }

    #[test]
    fn fitted_diagrams_can_be_dragged_and_moved_with_the_controls() {
        for expanded in [false, true] {
            let mut viewer = DiagramViewer::new("diagram.svg".into(), "".into(), 2., expanded);
            viewer.image_width = px(600.);
            viewer.height = px(400.);
            assert!(viewer.pan_by(point(px(32.), px(-32.))));
            assert_eq!(viewer.pan, point(px(32.), px(-32.)));
        }
    }

    #[test]
    fn zoom_preserves_the_diagram_point_under_the_pointer() {
        let mut viewer = DiagramViewer::new("diagram.svg".into(), "".into(), 2., true);
        viewer.viewport =
            gpui::Bounds::new(point(px(40.), px(60.)), gpui::size(px(600.), px(400.)));
        viewer.image_width = px(480.);
        viewer.height = px(400.);
        let pointer = point(px(200.), px(150.));
        let before = (pointer - viewer.viewport.center() - viewer.pan) / viewer.zoom;
        viewer.zoom_at(1.25, pointer);
        let after = (pointer - viewer.viewport.center() - viewer.pan) / viewer.zoom;
        assert_eq!(before, after);
    }

    #[test]
    fn expanded_viewer_pan_reaches_the_bottom_of_a_zoomed_diagram() {
        let mut viewer = DiagramViewer::new("diagram.svg".into(), "".into(), 2., true);
        viewer.image_width = px(600.);
        viewer.height = px(400.);
        viewer.zoom = 4.;

        viewer.pan_by(point(px(0.), px(-10_000.)));

        let top = viewer.pan.y + viewer.height * (1. - viewer.zoom) / 2.;
        let bottom = top + viewer.height * viewer.zoom;
        assert_eq!(bottom, viewer.height);
    }

    #[test]
    fn builtin_mermaid_matches_live_github_canvas_padding() {
        let source = "flowchart LR\n  Choose[Choose reading theme] --> Palette[Resolve light or dark palette]\n  Palette --> Preview[Render Markdown preview]\n";
        let svg = render_mermaid(source, &crate::core::markdown_theme::builtin(), false).unwrap();
        let live: serde_json::Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/github-live-2026-10-06/mermaid-light.json"
        ))
        .unwrap();
        let viewbox = regex::Regex::new(r#"viewBox="([^"]+)""#).unwrap();
        let native: Vec<f32> = viewbox.captures(&svg).unwrap()[1]
            .split_whitespace()
            .map(|v| v.parse().unwrap())
            .collect();
        let website: Vec<f32> = live["viewBox"]
            .as_str()
            .unwrap()
            .split_whitespace()
            .map(|v| v.parse().unwrap())
            .collect();
        assert!(
            (native[3] - website[3]).abs() < 1.,
            "Mermaid canvas height differs from live GitHub: {native:?} vs {website:?}"
        );
    }

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
