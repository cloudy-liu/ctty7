//! One lazily-created browser view per window. The source editor remains the
//! owner of the file; the browser loads its saved, original file URL.

use std::{
    cell::Cell,
    path::{Path, PathBuf},
    rc::Rc,
    time::Instant,
};

use gpui::{App, AppContext, Context, Entity, FocusHandle, Focusable, Render, Window, prelude::*};
use raw_window_handle::{RawWindowHandle, WindowHandle};

pub(crate) struct PendingHtmlPreview {
    path: PathBuf,
    webview: wry::WebView,
    started: Rc<Cell<Instant>>,
}

pub(crate) struct HtmlPreview {
    pub(crate) path: PathBuf,
    view: Entity<gpui_wry::WebView>,
    started: Rc<Cell<Instant>>,
}

impl HtmlPreview {
    /// Run on the foreground executor, outside an App update: WebView2 pumps
    /// Windows messages during construction, which may re-enter GPUI.
    pub(crate) fn build(
        path: &Path,
        parent: RawWindowHandle,
    ) -> anyhow::Result<PendingHtmlPreview> {
        let url = url::Url::from_file_path(path)
            .map_err(|_| anyhow::anyhow!("HTML preview requires an absolute local file path"))?;
        let mut context = wry::WebContext::new(
            crate::core::config::config_dir_path().map(|dir| dir.join("html-preview")),
        );
        let started = Rc::new(Cell::new(Instant::now()));
        let webview = wry::WebViewBuilder::new_with_web_context(&mut context)
            .with_url(url.as_str())
            .with_visible(false)
            .with_focused(false)
            .with_devtools(cfg!(debug_assertions))
            .with_new_window_req_handler(|_, _| wry::NewWindowResponse::Deny)
            .with_on_page_load_handler({
                let started = started.clone();
                move |event, _| {
                    if matches!(event, wry::PageLoadEvent::Finished) {
                        let ms = started.get().elapsed().as_secs_f64() * 1000.;
                        log::debug!("HTML preview load finished in {ms:.2} ms");
                        #[cfg(feature = "markdown-visual-tests")]
                        eprintln!("HTML_PREVIEW_LOAD_MS={ms:.2}");
                    }
                }
            })
            // The handle comes from the current GPUI window and is used only
            // synchronously on its UI thread. No native handle crosses threads.
            .build_as_child(&unsafe { WindowHandle::borrow_raw(parent) })?;
        Ok(PendingHtmlPreview {
            path: path.to_path_buf(),
            webview,
            started,
        })
    }

    pub(crate) fn attach(pending: PendingHtmlPreview, window: &mut Window, cx: &mut App) -> Self {
        let PendingHtmlPreview {
            path,
            webview,
            started,
        } = pending;
        let view = cx.new(|cx| gpui_wry::WebView::new(webview, window, cx));
        // The adapter starts visible internally even when the native builder
        // starts hidden. Keep both sides in agreement until it is laid out.
        view.update(cx, |view, _| view.hide());
        Self {
            path,
            view,
            started,
        }
    }

    pub(crate) fn navigate(&mut self, path: &Path, cx: &mut App) -> anyhow::Result<()> {
        if self.path == path {
            return Ok(());
        }
        let url = url::Url::from_file_path(path)
            .map_err(|_| anyhow::anyhow!("HTML preview requires an absolute local file path"))?;
        self.started.set(Instant::now());
        self.view.read(cx).raw().load_url(url.as_str())?;
        self.path = path.to_path_buf();
        Ok(())
    }

    pub(crate) fn reload(&self, cx: &App) -> anyhow::Result<()> {
        self.started.set(Instant::now());
        self.view.read(cx).raw().reload()?;
        Ok(())
    }

    pub(crate) fn focus(&self, window: &mut Window, cx: &mut App) {
        self.focus_handle(cx).focus(window, cx);
        if self.view.read(cx).visible() {
            let _ = self.view.read(cx).raw().focus();
        }
    }

    pub(crate) fn set_visible(&self, visible: bool, window: &Window, cx: &mut App) {
        let focused = self.focus_handle(cx).is_focused(window);
        self.view.update(cx, |view, _| {
            if view.visible() != visible {
                if visible {
                    view.show();
                    if focused {
                        let _ = view.raw().focus();
                    }
                } else {
                    view.hide();
                }
            }
        });
    }
}

impl Focusable for HtmlPreview {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.view.read(cx).focus_handle(cx)
    }
}

impl Render for HtmlPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        gpui::div()
            .size_full()
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, window, cx| this.focus(window, cx)),
            )
            .child(self.view.clone())
    }
}
