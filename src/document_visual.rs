//! Opt-in native document capture using the full production application.
//! Keep this fixture independent of document chrome so it also runs on a baseline.

use gpui::{prelude::*, *};
use gpui_component::Root;
use serde::Deserialize;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::{
    core::{
        config::{Config, DocumentLayout},
        session::{Session, WorkspaceStore},
    },
    ui::{
        app::{Tab, Tty7App},
        pane::Pane,
        windows::WindowRegistry,
    },
};

#[derive(Deserialize)]
struct Options {
    root: PathBuf,
    files: Vec<PathBuf>,
    output: PathBuf,
    #[serde(default)]
    image: Option<PathBuf>,
    #[serde(default)]
    active: usize,
    #[serde(default)]
    preview: Option<bool>,
    #[serde(default = "default_width")]
    width: f32,
    #[serde(default = "default_height")]
    height: f32,
    #[serde(default)]
    dark: bool,
    #[serde(default)]
    dock: bool,
    #[serde(default = "default_hold_seconds")]
    hold_seconds: u64,
}

fn default_width() -> f32 {
    1200.
}
fn default_height() -> f32 {
    900.
}
fn default_hold_seconds() -> u64 {
    30
}

pub(crate) fn run(path: &std::ffi::OsStr) {
    let mut options: Options = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert!(!options.files.is_empty(), "supply at least one document");
    assert!(
        options.active < options.files.len(),
        "active document is out of range"
    );
    assert!((480. ..=2400.).contains(&options.width));
    assert!((400. ..=1600.).contains(&options.height));
    options.root = std::fs::canonicalize(&options.root).unwrap();
    let files: Vec<_> = options
        .files
        .iter()
        .map(|file| std::fs::canonicalize(options.root.join(file)).unwrap())
        .collect();
    gpui_platform::application()
        .with_assets(crate::ui::assets::Assets)
        .with_quit_mode(QuitMode::Explicit)
        .run(move |cx| {
            gpui_component::init(cx);
            crate::register_bundled_fonts(cx);
            let mut config = Config::default();
            config.theme_follow_system = false;
            config.theme_preset = if options.dark { "dark" } else { "light" }.into();
            config.install_cli_on_path = false;
            config.cursor_blink = false;
            config.show_tray_icon = false;
            config.document_layout = if options.dock {
                DocumentLayout::Dock
            } else {
                DocumentLayout::Fill
            };
            cx.set_global(config);
            WorkspaceStore::init(cx);
            WindowRegistry::init(cx);
            crate::ui::presets::load_registry(cx);
            crate::ui::keymap::init(cx);
            crate::ui::markdown_preview::init(Default::default(), cx);
            let mut owner = None;
            let handle = cx
                .open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                            None,
                            size(px(options.width), px(options.height)),
                            cx,
                        ))),
                        titlebar: Some(TitlebarOptions {
                            title: Some("ctty7 document acceptance".into()),
                            ..Default::default()
                        }),
                        show: true,
                        focus: true,
                        ..Default::default()
                    },
                    |window, cx| {
                        crate::ui::theme::apply_theme(Some(window), cx);
                        let app = cx.new(|cx| {
                            Tty7App::with_session(None, Some(Session::default()), window, cx)
                        });
                        app.update(cx, |app, cx| {
                            app.tabs.push(Tab::new(Pane::Empty));
                            app.active = app.tabs.len() - 1;
                            app.tab_code_mut_or_init()
                                .unwrap()
                                .roots
                                .push(options.root.clone());
                            cx.notify();
                        });
                        owner = Some(app.clone());
                        cx.new(|cx| Root::new(app, window, cx))
                    },
                )
                .unwrap();
            cx.activate(true);
            let app = owner.unwrap();
            // The production tray loop quits when it sees no registered
            // windows and the tray is disabled, even if a native window exists.
            WindowRegistry::register(cx, app.read(cx).workspace, handle.into(), app.downgrade());
            cx.spawn(async move |cx| {
                // Production open requests cancel older requests. Await each
                // installation before opening the next fixture document.
                for path in files.iter().chain(std::iter::once(&files[options.active])) {
                    cx.update_window(handle.into(), |_, window, cx| {
                        app.update(cx, |app, cx| {
                            app.editor_open_on_host(
                                tty7_core::host::local::LocalHost::new(),
                                path,
                                window,
                                cx,
                            );
                        })
                    })
                    .unwrap();
                    let deadline = Instant::now() + Duration::from_secs(30);
                    loop {
                        cx.background_executor()
                            .timer(Duration::from_millis(50))
                            .await;
                        if app.read_with(cx, |app, _| {
                            app.tab_code()
                                .and_then(|code| code.active_file())
                                .is_some_and(|file| file.path == *path)
                        }) {
                            break;
                        }
                        assert!(
                            Instant::now() < deadline,
                            "document did not open: {}",
                            path.display()
                        );
                    }
                }
                if let Some(preview) = options.preview {
                    cx.update_window(handle.into(), |_, window, cx| {
                        app.update(cx, |app, cx| {
                            if app.tab_code().unwrap().active_file().unwrap().preview != preview {
                                app.editor_toggle_preview(window, cx);
                            }
                        })
                    })
                    .unwrap();
                    assert_eq!(
                        app.read_with(cx, |app, _| app
                            .tab_code()
                            .unwrap()
                            .active_file()
                            .unwrap()
                            .preview),
                        preview,
                        "requested mode is unavailable"
                    );
                }
                let deadline = Instant::now() + Duration::from_secs(30);
                loop {
                    cx.background_executor()
                        .timer(Duration::from_millis(100))
                        .await;
                    let ready = app.read_with(cx, |app, cx| {
                        let file = app.tab_code().unwrap().active_file().unwrap();
                        !file.preview
                            || file.reading.as_ref().is_none_or(|view| {
                                let view = view.read(cx);
                                !view.is_loading(cx) && !view.pending_diagrams_for_profile()
                            })
                    });
                    if ready {
                        break;
                    }
                    assert!(Instant::now() < deadline, "Markdown parse timed out");
                }
                #[cfg(any(windows, target_os = "macos"))]
                while app.read_with(cx, |app, _| {
                    let file = app.tab_code().unwrap().active_file().unwrap();
                    file.preview
                        && matches!(
                            file.path.extension().and_then(|s| s.to_str()),
                            Some("html" | "htm")
                        )
                }) {
                    assert!(
                        Instant::now() < deadline,
                        "HTML page did not finish loading"
                    );
                    let view = app.read_with(cx, |app, _| app.editor.html.clone());
                    let Some(view) = view else {
                        cx.background_executor()
                            .timer(Duration::from_millis(100))
                            .await;
                        continue;
                    };
                    let rx = view
                        .read_with(cx, |view, cx| view.inspect("document.readyState", cx))
                        .unwrap();
                    let response = smol::future::or(async { rx.recv().await.ok() }, async {
                        cx.background_executor()
                            .timer(Duration::from_millis(250))
                            .await;
                        None
                    })
                    .await;
                    if response.as_deref() == Some("\"complete\"") {
                        break;
                    }
                    assert!(
                        Instant::now() < deadline,
                        "HTML page did not finish loading"
                    );
                    cx.background_executor()
                        .timer(Duration::from_millis(100))
                        .await;
                }
                // Settle native layout/font shaping after content readiness.
                cx.background_executor()
                    .timer(Duration::from_millis(500))
                    .await;
                let evidence = cx
                    .update_window(handle.into(), |_, window, cx| {
                        window.set_window_title("ctty7 document acceptance");
                        window.refresh();
                        let _ = window.draw(cx);
                        let file = app.read(cx).tab_code().unwrap().active_file().unwrap();
                        assert_eq!(
                            file.path, files[options.active],
                            "selected document changed before capture"
                        );
                        if let Some(preview) = options.preview {
                            assert_eq!(
                                file.preview, preview,
                                "requested preview failed before capture"
                            );
                        }
                        let html = file.preview
                            && matches!(
                                file.path.extension().and_then(|s| s.to_str()),
                                Some("html" | "htm")
                            );
                        let capture = if html {
                            "Use native screenshot: GPUI images omit the child WebView".to_owned()
                        } else if let Some(path) = &options.image {
                            match window
                                .render_to_image()
                                .and_then(|image| image.save(path).map_err(Into::into))
                            {
                                Ok(()) => format!("Saved {}", path.display()),
                                Err(error) => format!("Use native screenshot: {error}"),
                            }
                        } else {
                            "No GPU image requested".to_owned()
                        };
                        serde_json::json!({
                            "ready": true,
                            "title": "ctty7 document acceptance",
                            "os": std::env::consts::OS,
                            "scale_factor": window.scale_factor(),
                            "width": f32::from(window.viewport_size().width),
                            "height": f32::from(window.viewport_size().height),
                            "dark": options.dark,
                            "dock": options.dock,
                            "files": files,
                            "active": file.path,
                            "preview": file.preview,
                            "open_files": app.read(cx).tab_code().unwrap().files.len(),
                            "capture": capture,
                        })
                    })
                    .unwrap();
                std::fs::write(
                    &options.output,
                    serde_json::to_vec_pretty(&evidence).unwrap(),
                )
                .unwrap();
                eprintln!(
                    "NATIVE_DOCUMENT_READY_FOR_CAPTURE {}",
                    options.output.display()
                );
                cx.background_executor()
                    .timer(Duration::from_secs(options.hold_seconds.min(600)))
                    .await;
                cx.update(|cx| cx.quit());
            })
            .detach();
        });
}
