//! Opt-in native visual fixture using the production Markdown reading view.

use gpui::{prelude::*, *};
use gpui_component::{Root, input::InputState};
use serde::Deserialize;
use std::{path::PathBuf, time::Duration};

use crate::{
    core::config::Config,
    ui::{app::Tty7App, markdown_preview::MarkdownPreview},
};

#[derive(Deserialize)]
struct Options {
    source: PathBuf,
    width: f32,
    height: f32,
    scale: f32,
    dark: bool,
    #[serde(default)]
    anchor: String,
    #[serde(default)]
    output: Option<PathBuf>,
    #[serde(default = "default_hold_seconds")]
    hold_seconds: u64,
}

fn default_hold_seconds() -> u64 {
    12
}

struct Fixture {
    reading: Entity<MarkdownPreview>,
    _owner: Entity<Tty7App>,
    scale: f32,
}

impl Render for Fixture {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(16. * self.scale));
        div().size_full().child(self.reading.clone())
    }
}

pub(crate) fn run(path: &std::ffi::OsStr) {
    let options: Options = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert!((400. ..=1600.).contains(&options.width));
    assert!((400. ..=1600.).contains(&options.height));
    assert!((1. ..=2.).contains(&options.scale));
    let source = std::fs::read_to_string(&options.source).unwrap();
    let started = std::time::Instant::now();
    gpui_platform::application().with_assets(crate::ui::assets::Assets)
        .with_quit_mode(QuitMode::Explicit).run(move |cx| {
            gpui_component::init(cx);
            crate::register_bundled_fonts(cx);
            let mut config = Config::default();
            config.theme_follow_system = false;
            config.theme_preset = if options.dark { "dark" } else { "light" }.into();
            config.install_cli_on_path = false;
            cx.set_global(config);
            crate::core::session::WorkspaceStore::init(cx);
            crate::ui::windows::WindowRegistry::init(cx);
            crate::ui::presets::load_registry(cx);
            crate::ui::keymap::init(cx);
            crate::ui::markdown_preview::init(Default::default(), cx);
            let mut reading = None;
            let handle = cx.open_window(WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None, size(px(options.width), px(options.height)), cx))),
                titlebar: Some(TitlebarOptions { title: Some("Markdown visual acceptance".into()), ..Default::default() }),
                show: true,
                focus: true,
                ..Default::default()
            }, |window, cx| {
                crate::ui::theme::apply_theme(Some(window), cx);
                let owner = cx.new(|cx| Tty7App::with_session(None,
                    Some(crate::core::session::Session::default()), window, cx));
                window.set_window_title("Markdown visual acceptance");
                let input = cx.new(|cx| {
                    let mut input = InputState::new(window, cx).multi_line(true);
                    input.set_value(source, window, cx);
                    input
                });
                let view = cx.new(|cx| MarkdownPreview::new(input,
                    tty7_core::host::local::LocalHost::new(), options.source.clone(),
                    owner.downgrade(), true, cx));
                if !options.anchor.is_empty() {
                    view.update(cx, |view, cx| view.navigate_anchor(options.anchor.clone(), cx));
                }
                window.focus(&view.focus_handle(cx), cx);
                reading = Some(view.clone());
                let fixture = cx.new(|_| Fixture { reading: view, _owner: owner, scale: options.scale });
                cx.new(|cx| Root::new(fixture, window, cx))
            }).unwrap();
            cx.activate(true);
            let reading = reading.unwrap();
            cx.spawn(async move |cx| {
                // Real executors and native font shaping are retained here.
                // Wait for the production background parser rather than a fixed screenshot delay.
                let deadline = std::time::Instant::now() + Duration::from_secs(20);
                loop {
                    cx.background_executor().timer(Duration::from_millis(100)).await;
                    let ready = reading.read_with(cx, |view, cx| !view.is_loading(cx));
                    if ready { break; }
                    assert!(std::time::Instant::now() < deadline, "Markdown parse timed out");
                }
                cx.background_executor().timer(Duration::from_secs(2)).await;
                handle.update(cx, |_, window, cx| {
                    window.refresh();
                    eprintln!("native Markdown: os={}, dpi_scale={}, app_scale={}, viewport={:?}, ready={:?}",
                        std::env::consts::OS, window.scale_factor(), options.scale,
                        window.viewport_size(), started.elapsed());
                    let view = reading.read(cx);
                    assert!(!view.text.read(cx).source().is_empty());
                    if let Some(output) = &options.output {
                        #[cfg(target_os = "macos")]
                        {
                            let screenshot = window.render_to_image().unwrap();
                            let distinct = screenshot.pixels().filter(|p| p[0].abs_diff(p[1]) > 20
                                || p[1].abs_diff(p[2]) > 20).count();
                            assert!(distinct > 200, "native Markdown screenshot has no colored content");
                            screenshot.save(output).unwrap();
                        }
                        #[cfg(not(target_os = "macos"))]
                        let _ = output;
                    }
                }).unwrap();
                eprintln!("NATIVE_MARKDOWN_READY");
                cx.background_executor().timer(Duration::from_secs(options.hold_seconds.min(600))).await;
                cx.update(|cx| cx.quit());
            }).detach();
        });
}
