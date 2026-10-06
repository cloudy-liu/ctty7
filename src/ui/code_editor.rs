use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui::prelude::*;
use gpui::{
    AnyElement, Context, Entity, Focusable as _, PromptLevel, SharedString, Subscription, Window,
    div, px,
};
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::input::{Input, InputEvent, InputState, Position, TabSize};
use gpui_component::menu::ContextMenuExt as _;
use gpui_component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, WindowExt as _, h_flex, v_flex,
};

use crate::ui::app::Tty7App;
use crate::ui::document_column::DocumentChrome;
use crate::ui::host_ops::{HostId, HostOps, MTime, SharedHost, WatchSub};
use crate::ui::i18n::{L10nKey, t, t_fmt};

const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;

const RELOAD_DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(200);

pub(crate) struct OpenFile {
    pub(crate) path: PathBuf,
    /// The machine `path` lives on, held rather than looked up. Saves,
    /// reloads and duplicate detection all key on its id — an SFTP file and a
    /// local file can share the string `/etc/hosts` without being the same
    /// file — and saving goes straight through this handle, so a buffer stays
    /// saveable however the window's own machine has changed underneath it.
    pub(crate) host: SharedHost,
    pub(crate) input: Entity<InputState>,
    pub(crate) dirty: bool,
    disk_mtime: Option<MTime>,
    edit_seq: u64,
    saving: Option<u64>,
    save_pending: bool,
    save_then_close: bool,
    reload_seq: u64,
    pub(crate) conflict: bool,
    pub(crate) preview: bool,
    pub(crate) wrap: bool,
    pub(crate) reading: Option<Entity<crate::ui::markdown_preview::MarkdownPreview>>,
    source_highlighter_ready: std::cell::Cell<bool>,
    _sub: Subscription,
    _observe: Subscription,
}

impl OpenFile {
    fn prepare_source(&self, cx: &mut Context<Tty7App>) {
        if !self.source_highlighter_ready.replace(true) {
            self.input.update(cx, |input, cx| {
                input.set_highlighter(crate::ui::editor_theme::language("markdown"), cx);
            });
        }
    }

    fn label(&self) -> SharedString {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| self.path.display().to_string())
            .into()
    }
}

pub(crate) struct TabCode {
    pub(crate) visible: bool,
    pub(crate) files: Vec<OpenFile>,
    pub(crate) active: usize,
    pub(crate) roots: Vec<PathBuf>,
    pub(crate) expanded: std::collections::HashSet<PathBuf>,
    /// Roots start open. Their folds belong to this tab and last for the session.
    pub(crate) collapsed_roots: HashSet<PathBuf>,
    pub(crate) selected: Option<crate::ui::file_tree::TreeSelection>,
}

impl TabCode {
    pub(crate) fn new() -> Self {
        Self {
            visible: false,
            files: Vec::new(),
            active: 0,
            roots: Vec::new(),
            expanded: std::collections::HashSet::new(),
            collapsed_roots: HashSet::new(),
            selected: None,
        }
    }

    pub(crate) fn active_file(&self) -> Option<&OpenFile> {
        self.files.get(self.active)
    }
}

enum EditorNavigation {
    Source { line: u32, column: u32 },
    PreviewAnchor(String),
}

pub(crate) struct EditorPanelState {
    /// Only the latest open request may install a file or move focus. Its
    /// navigation target travels with the request and is dropped on failure.
    open_seq: u64,
    watch: Option<Arc<WatchSub>>,
    watch_host: Option<SharedHost>,
    watch_opening: bool,
    watch_busy: bool,
    watch_dirty: bool,
    watched_dirs: HashSet<PathBuf>,
    watched_files: HashSet<PathBuf>,
    events_tx: smol::channel::Sender<Vec<PathBuf>>,
}

impl EditorPanelState {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Tty7App>) -> Self {
        let (tx, rx) = smol::channel::unbounded::<Vec<PathBuf>>();
        cx.spawn_in(window, async move |app, cx| {
            while let Ok(first) = rx.recv().await {
                cx.background_executor().timer(RELOAD_DEBOUNCE).await;
                let mut changed: HashSet<PathBuf> = first.into_iter().collect();
                while let Ok(more) = rx.try_recv() {
                    changed.extend(more);
                }
                let ok = app.update_in(cx, |app, window, cx| {
                    for path in changed {
                        if app.editor.watched_files.contains(&path) {
                            app.editor_handle_external_change(&path, window, cx);
                        }
                    }
                });
                if ok.is_err() {
                    break;
                }
            }
        })
        .detach();
        Self {
            open_seq: 0,
            watch: None,
            watch_host: None,
            watch_opening: false,
            watch_busy: false,
            watch_dirty: false,
            watched_dirs: HashSet::new(),
            watched_files: HashSet::new(),
            events_tx: tx,
        }
    }
}

pub(crate) fn language_for_path(path: &Path) -> &'static str {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        let lowered = name.to_ascii_lowercase();
        match lowered.as_str() {
            "makefile" | "gnumakefile" => return "make",
            "cmakelists.txt" => return "cmake",
            _ => {}
        }
        if lowered.starts_with('.') && (lowered.contains("shrc") || lowered.ends_with("profile")) {
            return "bash";
        }
    }
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return "text";
    };
    match ext.to_ascii_lowercase().as_str() {
        "rs" => "rust",
        "go" => "go",
        "py" | "pyi" => "python",
        "js" | "mjs" | "cjs" | "jsx" => "javascript",
        "ts" | "mts" | "cts" => "typescript",
        "tsx" => "tsx",
        "json" | "jsonc" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "html" | "htm" => "html",
        "css" => "css",
        "md" | "markdown" => "markdown",
        "sh" | "bash" | "zsh" => "bash",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" | "hh" => "cpp",
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "lua" => "lua",
        "rb" => "ruby",
        "php" => "php",
        "sql" => "sql",
        "swift" => "swift",
        "scala" => "scala",
        "zig" => "zig",
        "proto" => "proto",
        "diff" | "patch" => "diff",
        "ex" | "exs" => "elixir",
        "erb" => "erb",
        "ejs" => "ejs",
        "svelte" => "svelte",
        "astro" => "astro",
        "graphql" | "gql" => "graphql",
        "cs" => "csharp",
        "cmake" => "cmake",
        _ => "text",
    }
}

/// How many frames a jump-to-line may wait for the editor to be laid out.
///
/// `InputState::scroll_to` gives up when the buffer has never been painted,
/// and that is exactly the state a file that just opened is in: the cursor
/// lands on the right line and the view stays at the top of the file, which
/// is the one thing a `:120` link exists to avoid. Three frames is the same
/// bounded-retry shape `prefill::select_all_when_drawn` uses against the same
/// class of problem.
const CURSOR_SCROLL_ATTEMPTS: u8 = 3;

/// Puts the cursor at `position`, re-trying on later frames until the scroll
/// that follows it can actually be computed.
///
/// Applied immediately as well as on the retry: the cursor itself lands with
/// no layout, so the position is right even for a pane that never paints.
fn place_cursor(
    input: Entity<InputState>,
    position: Position,
    left: u8,
    window: &mut Window,
    cx: &mut gpui::App,
) {
    input.update(cx, |state, cx| {
        state.set_cursor_position(position, window, cx);
    });
    // A target near the top of the file scrolls nowhere and is already done;
    // so is one that has landed. Either way this stops.
    if left <= 1 || input.read(cx).scroll_offset().y != px(0.) {
        return;
    }
    window.on_next_frame(move |window, cx| {
        place_cursor(input, position, left - 1, window, cx);
    });
    // Registering a callback does not by itself ask for a frame, and an
    // overlay that has finished drawing has no other reason to produce one.
    window.refresh();
}

fn looks_binary(bytes: &[u8]) -> bool {
    bytes.iter().take(8192).any(|b| *b == 0)
}

#[derive(Debug, PartialEq, Eq)]
enum ExternalChange {
    Ignore,
    Conflict,
    Reload,
}

fn classify_external_change(
    saving: bool,
    dirty: bool,
    disk_mtime: Option<MTime>,
    observed: Option<MTime>,
) -> ExternalChange {
    if saving {
        return ExternalChange::Ignore;
    }
    if observed.is_some() && observed == disk_mtime {
        return ExternalChange::Ignore;
    }
    if dirty {
        ExternalChange::Conflict
    } else {
        ExternalChange::Reload
    }
}

#[derive(Debug, PartialEq, Eq)]
struct SaveLanding {
    clean: bool,
    requeue: bool,
}

fn settle_save(ok: bool, wrote_seq: u64, current_seq: u64, pending: bool) -> SaveLanding {
    SaveLanding {
        clean: ok && wrote_seq == current_seq,
        requeue: ok && pending,
    }
}

impl Tty7App {
    pub(crate) fn tab_code(&self) -> Option<&TabCode> {
        self.tabs.get(self.active)?.code.as_deref()
    }

    pub(crate) fn tab_code_mut(&mut self) -> Option<&mut TabCode> {
        self.tabs.get_mut(self.active)?.code.as_deref_mut()
    }

    pub(crate) fn tab_code_mut_or_init(&mut self) -> Option<&mut TabCode> {
        let tab = self.tabs.get_mut(self.active)?;
        Some(tab.code.get_or_insert_with(|| Box::new(TabCode::new())))
    }

    pub(crate) fn code_panel_visible(&self) -> bool {
        self.tab_code().is_some_and(|c| c.visible)
    }

    fn editor_rebuild_watcher(&mut self, cx: &mut Context<Self>) {
        // Only files on the host the watch itself runs on. A path from
        // another machine — an SFTP file, say — does not exist under that
        // watcher's feet, and would either miss or, worse, match a local file
        // that happens to share its name.
        //
        // A host that cannot watch therefore gets no external-change
        // detection at all: an SFTP buffer will not notice the file changing
        // underneath it, and saving overwrites whatever is there. Catching
        // that at save time needs a "keep mine" that survives to the next
        // save, which the conflict banner does not have yet.
        let watch_host = self.spawn_host(cx);
        let files: HashSet<PathBuf> = self
            .tabs
            .iter()
            .filter_map(|t| t.code.as_deref())
            .flat_map(|c| c.files.iter())
            .filter(|f| f.host.id() == watch_host)
            .map(|f| f.path.clone())
            .collect();
        let dirs: HashSet<PathBuf> = files
            .iter()
            .filter_map(|p| p.parent().map(Path::to_path_buf))
            .collect();
        self.editor.watched_files = files;
        if dirs == self.editor.watched_dirs {
            return;
        }
        self.editor.watched_dirs = dirs;
        self.editor_watch_apply(cx);
    }

    fn editor_watch_apply(&mut self, cx: &mut Context<Self>) {
        let want: Vec<PathBuf> = self.editor.watched_dirs.iter().cloned().collect();
        let Some(host) = self.active_host(cx) else {
            return;
        };

        if !self
            .editor
            .watch_host
            .as_ref()
            .is_some_and(|opened_with| Arc::ptr_eq(opened_with, &host))
        {
            self.editor.watch = None;
            self.editor.watch_host = None;
            self.editor.watch_busy = false;
            self.editor.watch_dirty = false;
        }

        if let Some(sub) = self.editor.watch.clone() {
            if self.editor.watch_busy {
                self.editor.watch_dirty = true;
                return;
            }
            self.editor.watch_busy = true;
            HostOps::run(
                host,
                cx,
                move |_| sub.set_dirs(&want),
                |app: &mut Self, result: std::io::Result<()>, cx| {
                    app.editor.watch_busy = false;
                    if let Err(e) = result {
                        log::warn!("editor: could not update the watched set: {e}");
                    }
                    if std::mem::take(&mut app.editor.watch_dirty) {
                        app.editor_watch_apply(cx);
                    }
                },
            );
            return;
        }
        if self.editor.watch_opening {
            return;
        }
        self.editor.watch_opening = true;
        let opened_host = Arc::clone(&host);
        let opened_with = self.editor.watched_dirs.clone();
        HostOps::run(
            host,
            cx,
            {
                let want = want.clone();
                move |h| h.watch(&want).map(Arc::new)
            },
            move |app, result: std::io::Result<Arc<WatchSub>>, cx| {
                app.editor.watch_opening = false;
                let sub = match result {
                    Ok(sub) => sub,
                    Err(e) => {
                        log::warn!("editor: external-change watcher unavailable: {e}");
                        return;
                    }
                };
                let events = sub.events().clone();
                app.editor.watch = Some(sub);
                app.editor.watch_host = Some(opened_host);
                cx.spawn(async move |app, cx| {
                    while let Ok(batch) = events.recv().await {
                        let ok = app.update(cx, |app, _cx| {
                            let _ = app.editor.events_tx.try_send(batch);
                        });
                        if ok.is_err() {
                            break;
                        }
                    }
                })
                .detach();
                if app.editor.watched_dirs != opened_with {
                    app.editor_watch_apply(cx);
                }
            },
        );
    }

    /// Shows what a file link in the grid pointed at.
    ///
    /// A file opens in the editor, on the line the link named; the tree
    /// follows along so "what does it say?" and "where does it live?" are
    /// answered by the same click. A directory has no contents to show, so it
    /// is only ever the tree.
    pub(crate) fn open_linked_file(
        &mut self,
        path: &Path,
        line: Option<u32>,
        column: Option<u32>,
        is_dir: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if is_dir {
            self.file_tree_show(path, window, cx);
            return;
        }
        self.open_file_in_editor_at(path, line, column, window, cx);
        self.file_tree_reveal_path(path, cx);
    }

    /// [`Self::open_file_in_editor`], landing the cursor on a line the caller
    /// already knows — what a `src/main.rs:120:3` in the grid was pointing at.
    pub(crate) fn open_file_in_editor_at(
        &mut self,
        path: &Path,
        line: Option<u32>,
        column: Option<u32>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(host) = self.active_host(cx) else {
            return;
        };
        self.editor_open_with_navigation(
            host,
            path,
            line.map(|line| EditorNavigation::Source {
                line,
                column: column.unwrap_or(1),
            }),
            window,
            cx,
        );
    }

    fn apply_editor_navigation(
        &mut self,
        host: HostId,
        opened: &Path,
        navigation: Option<EditorNavigation>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(navigation) = navigation else {
            return;
        };
        let Some(file) = self.tab_code_mut().and_then(|c| {
            c.files
                .iter_mut()
                .find(|f| f.host.id() == host && f.path == *opened)
        }) else {
            return;
        };
        match navigation {
            EditorNavigation::Source { line, column } => {
                file.prepare_source(cx);
                let input = file.input.clone();
                file.preview = false;
                input.update(cx, |input, cx| input.focus(window, cx));
                // File links count from one; Position counts from zero.
                let position = Position {
                    line: line.saturating_sub(1),
                    character: column.saturating_sub(1),
                };
                place_cursor(input, position, CURSOR_SCROLL_ATTEMPTS, window, cx);
            }
            EditorNavigation::PreviewAnchor(anchor) => {
                if let Some(preview) = &file.reading {
                    file.preview = true;
                    preview.update(cx, |preview, cx| preview.navigate_anchor(anchor, cx));
                    window.focus(&preview.focus_handle(cx), cx);
                }
            }
        }
    }

    pub(crate) fn editor_open_markdown_link(
        &mut self,
        host: SharedHost,
        path: &Path,
        fragment: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editor_open_with_navigation(
            host,
            path,
            fragment.map(EditorNavigation::PreviewAnchor),
            window,
            cx,
        );
    }

    pub(crate) fn open_file_in_editor(
        &mut self,
        path: &Path,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(host) = self.active_host(cx) else {
            return;
        };
        self.editor_open_on_host(host, path, window, cx);
    }

    /// [`Self::open_file_in_editor`] against an explicit host — the SFTP
    /// browser's files live on a host that is never the active one.
    pub(crate) fn editor_open_on_host(
        &mut self,
        host: SharedHost,
        path: &Path,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editor_open_with_navigation(host, path, None, window, cx);
    }

    fn editor_open_with_navigation(
        &mut self,
        host: SharedHost,
        path: &Path,
        navigation: Option<EditorNavigation>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editor.open_seq = self.editor.open_seq.wrapping_add(1);
        let open_seq = self.editor.open_seq;
        if self.tabs.get(self.active).is_none() {
            return;
        }
        self.raise_code_overlay();
        if self.editor_activate_open(host.id(), path, window, cx) {
            self.apply_editor_navigation(host.id(), path, navigation, window, cx);
            return;
        }
        let host_id = host.id();
        let p = path.to_path_buf();
        HostOps::run_in(
            host.clone(),
            window,
            cx,
            move |h| -> Result<(PathBuf, String, Option<MTime>), String> {
                let path = h.canonicalize(&p).unwrap_or(p);
                let meta = match h.stat(&path) {
                    Ok(m) => m,
                    Err(e) => {
                        return Err(t_fmt(
                            L10nKey::EditorCantOpen,
                            &[("path", &path.display().to_string()), ("e", &e.to_string())],
                        ));
                    }
                };
                if meta.len > MAX_FILE_BYTES {
                    return Err(t_fmt(
                        L10nKey::EditorFileTooLarge,
                        &[
                            ("path", &path.display().to_string()),
                            ("size", &(meta.len / (1024 * 1024)).to_string()),
                        ],
                    ));
                }
                let bytes = match h.read_file(&path, MAX_FILE_BYTES) {
                    Ok(b) => b,
                    Err(e) => {
                        return Err(t_fmt(
                            L10nKey::EditorCantRead,
                            &[("path", &path.display().to_string()), ("e", &e.to_string())],
                        ));
                    }
                };
                if looks_binary(&bytes) {
                    return Err(t_fmt(
                        L10nKey::EditorBinaryFile,
                        &[("path", &path.display().to_string())],
                    ));
                }
                let text = String::from_utf8(bytes).map_err(|_| {
                    t_fmt(
                        L10nKey::EditorNotUtf8,
                        &[("path", &path.display().to_string())],
                    )
                })?;
                Ok((path, text, meta.mtime))
            },
            move |app, opened, window, cx| {
                if app.editor.open_seq != open_seq {
                    return;
                }
                match opened {
                    Ok((path, text, mtime)) => {
                        app.editor_install_file(host, path.clone(), text, mtime, window, cx);
                        // Apply this request to the canonical path returned by
                        // its Host, including when the link named a symlink.
                        app.apply_editor_navigation(host_id, &path, navigation, window, cx);
                    }
                    Err(message) => window.push_notification(message, cx),
                }
            },
        );
    }

    fn editor_activate_open(
        &mut self,
        host: HostId,
        path: &Path,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(code) = self.tab_code_mut() else {
            return false;
        };
        let Some(ix) = code
            .files
            .iter()
            .position(|f| f.host.id() == host && f.path == *path)
        else {
            return false;
        };
        code.visible = true;
        let f = code.files.remove(ix);
        code.files.insert(0, f);
        code.active = 0;
        self.focus_editor(window, cx);
        cx.notify();
        true
    }

    fn editor_install_file(
        &mut self,
        host: SharedHost,
        path: PathBuf,
        text: String,
        mtime: Option<MTime>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let host_id = host.id();
        if self.editor_activate_open(host_id, &path, window, cx) {
            return;
        }
        if self.tabs.get(self.active).is_none() {
            return;
        }
        let language = language_for_path(&path);
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                // Markdown opens in reading mode. Defer the source grammar and
                // its embedded-language parsers until the editor is visible.
                .code_editor(if language == "markdown" {
                    "text"
                } else {
                    crate::ui::editor_theme::language(language)
                })
                .multi_line(true)
                .tab_size(TabSize {
                    tab_size: 4,
                    hard_tabs: false,
                })
                .line_number(true)
                .searchable(true)
                .replaceable(true)
                .folding(true)
                .soft_wrap(false)
                .default_value(text)
        });
        let sub = cx.subscribe_in(&input, window, {
            let path = path.clone();
            move |this: &mut Tty7App, _input, ev, _window, cx| {
                if matches!(ev, InputEvent::Change) {
                    let Some(f) = this
                        .tabs
                        .iter_mut()
                        .filter_map(|t| t.code.as_deref_mut())
                        .flat_map(|c| c.files.iter_mut())
                        .find(|f| f.host.id() == host_id && f.path == path)
                    else {
                        return;
                    };
                    f.dirty = true;
                    f.edit_seq = f.edit_seq.wrapping_add(1);
                    cx.notify();
                }
            }
        });
        let app = cx.weak_entity();
        let reading = (language == "markdown").then(|| {
            cx.new(|cx| {
                crate::ui::markdown_preview::MarkdownPreview::new(
                    input.clone(),
                    host.clone(),
                    path.clone(),
                    app,
                    true,
                    cx,
                )
            })
        });
        let tab = self
            .tabs
            .get_mut(self.active)
            .expect("checked at function entry");
        let code = tab.code.get_or_insert_with(|| Box::new(TabCode::new()));
        let observe = cx.observe(&input, |_, _, cx| cx.notify());
        code.files.insert(
            0,
            OpenFile {
                path,
                host,
                input,
                dirty: false,
                disk_mtime: mtime,
                edit_seq: 0,
                saving: None,
                save_pending: false,
                save_then_close: false,
                reload_seq: 0,
                conflict: false,
                preview: reading.is_some(),
                wrap: false,
                reading,
                source_highlighter_ready: std::cell::Cell::new(language != "markdown"),
                _sub: sub,
                _observe: observe,
            },
        );
        code.active = 0;
        code.visible = true;
        self.editor_rebuild_watcher(cx);
        self.focus_editor(window, cx);
        cx.notify();
    }

    pub(crate) fn toggle_code_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(tab) = self.tabs.get_mut(self.active) else {
            return;
        };
        let buried = tab.overlay_top == crate::ui::app::OverlayTop::Diff
            && tab.diff_overlay.is_some()
            && tab.code.as_ref().is_some_and(|c| c.visible);
        tab.overlay_top = crate::ui::app::OverlayTop::Code;
        if buried {
            self.focus_editor(window, cx);
            cx.notify();
            return;
        }
        let Some(tab) = self.tabs.get_mut(self.active) else {
            return;
        };
        let code = tab.code.get_or_insert_with(|| Box::new(TabCode::new()));
        if code.visible {
            code.visible = false;
            self.file_tree.editing = None;
            self.focus_active(window, cx);
            cx.notify();
            return;
        }
        code.visible = true;
        self.file_tree_refresh_roots(window, cx);
        if self.tab_code().is_some_and(|c| c.active_file().is_some()) {
            self.focus_editor(window, cx);
        } else {
            // With no file to show, the panel says "Open a file from the file
            // tree" and hands the tree the focus — but nothing was putting the
            // tree on screen, so ⌘⇧E on a fresh tab opened an empty editor
            // pointing at a panel the reader could not see or reach from
            // there. Reveal it, then focus it.
            if !self.file_tree_on_screen(cx) {
                self.set_right_panel_tab(crate::core::config::RightPanelTab::Files, cx);
            }
            self.file_tree.focus_handle.focus(window, cx);
        }
        cx.notify();
    }

    fn raise_code_overlay(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active) {
            tab.overlay_top = crate::ui::app::OverlayTop::Code;
        }
    }

    fn focus_editor(&self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(f) = self.tab_code().and_then(|c| c.active_file()) {
            if f.preview
                && let Some(reading) = &f.reading
            {
                reading.read(cx).focus_handle(cx).focus(window, cx);
            } else {
                f.prepare_source(cx);
                f.input.update(cx, |input, cx| input.focus(window, cx));
            }
        }
    }

    pub(crate) fn editor_toggle_preview(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(code) = self.tab_code_mut() {
            let ix = code.active;
            if let Some(file) = code.files.get_mut(ix).filter(|f| f.reading.is_some()) {
                file.preview = !file.preview;
            }
        }
        self.focus_editor(window, cx);
        cx.notify();
    }

    pub(crate) fn editor_has_focus(&self, window: &Window, cx: &Context<Self>) -> bool {
        self.code_panel_visible()
            && self
                .tab_code()
                .and_then(|c| c.active_file())
                .is_some_and(|f| {
                    if f.preview {
                        f.reading.as_ref().is_some_and(|reading| {
                            reading
                                .read(cx)
                                .focus_handle(cx)
                                .contains_focused(window, cx)
                        })
                    } else {
                        f.input
                            .read(cx)
                            .focus_handle(cx)
                            .contains_focused(window, cx)
                    }
                })
    }

    pub(crate) fn editor_source_has_focus(&self, window: &Window, cx: &Context<Self>) -> bool {
        self.tab_code()
            .and_then(|code| code.active_file())
            .is_some_and(|file| !file.preview)
            && self.editor_has_focus(window, cx)
    }

    pub(crate) fn editor_save_active(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self
            .tab_code()
            .and_then(|c| c.active_file())
            .map(|f| f.input.entity_id())
        else {
            return;
        };
        self.editor_save_file(id, false, window, cx);
    }

    fn editor_save_file(
        &mut self,
        id: gpui::EntityId,
        then_close: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // The file's own host, not the active one: the buffer keeps pointing
        // at the machine it was read from, however the focus has moved since.
        let Some(host) = self.editor_file_mut(id).map(|f| f.host.clone()) else {
            return;
        };
        let Some(f) = self.editor_file_mut(id) else {
            return;
        };
        f.save_then_close |= then_close;
        if f.saving.is_some() {
            f.save_pending = true;
            return;
        }
        let seq = f.edit_seq;
        f.saving = Some(seq);
        let text = f.input.read(cx).text().to_string();
        let target = f.path.clone();
        let host_id = host.id();
        let saved_in = target.parent().map(std::path::Path::to_path_buf);
        HostOps::run_in(
            host,
            window,
            cx,
            move |h| h.write_file(&target, text.as_bytes()).map(|m| m.mtime),
            move |app, result: std::io::Result<Option<MTime>>, window, cx| {
                let Some(f) = app.editor_file_mut(id) else {
                    return;
                };
                f.saving = None;
                let landing = settle_save(
                    result.is_ok(),
                    seq,
                    f.edit_seq,
                    std::mem::take(&mut f.save_pending),
                );
                let wrote = result.is_ok();
                match result {
                    Ok(mtime) => {
                        f.disk_mtime = mtime;
                    }
                    Err(e) => {
                        // "Save failed" did not say which file, and with more
                        // than one editor tab open that is the first thing you
                        // need to know.
                        let name = f
                            .path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| f.path.display().to_string());
                        let context = t_fmt(L10nKey::EditorSaveFailed, &[("name", &name)]);
                        HostOps::notify_err(window, cx, &context, &e);
                    }
                }
                if landing.clean {
                    f.dirty = false;
                    f.conflict = false;
                }
                // A save is a working-tree edit the `.git` watch cannot see,
                // and the file tree only sees it while it happens to be showing
                // that directory.
                if wrote && let Some(dir) = &saved_in {
                    app.scm_invalidate_cwd(host_id, dir, cx);
                }
                if landing.requeue {
                    app.editor_save_file(id, false, window, cx);
                    cx.notify();
                    return;
                }
                let close = app
                    .editor_file_mut(id)
                    .is_some_and(|f| std::mem::take(&mut f.save_then_close) && !f.dirty);
                if close && let Some((tab_ix, ix)) = app.editor_file_position(id) {
                    app.editor_remove_file_in(tab_ix, ix, cx);
                }
                cx.notify();
            },
        );
        cx.notify();
    }

    fn editor_file_mut(&mut self, id: gpui::EntityId) -> Option<&mut OpenFile> {
        self.tabs
            .iter_mut()
            .filter_map(|t| t.code.as_deref_mut())
            .flat_map(|c| c.files.iter_mut())
            .find(|f| f.input.entity_id() == id)
    }

    fn editor_file_position(&self, id: gpui::EntityId) -> Option<(usize, usize)> {
        self.tabs.iter().enumerate().find_map(|(tab_ix, t)| {
            let code = t.code.as_deref()?;
            let ix = code.files.iter().position(|f| f.input.entity_id() == id)?;
            Some((tab_ix, ix))
        })
    }

    pub(crate) fn editor_close_file(
        &mut self,
        ix: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(f) = self.tab_code().and_then(|c| c.files.get(ix)) else {
            return;
        };
        if !f.dirty {
            self.editor_remove_file(ix, cx);
            return;
        }
        let name = f.label();
        let answer = window.prompt(
            PromptLevel::Warning,
            &t_fmt(L10nKey::EditorUnsavedChanges, &[("name", &name)]),
            None,
            // Cancel sits between Save and Discard on purpose. The platform
            // renders the first button as the default and lays the rest out
            // beside it, so Discard was landing directly next to the key that
            // Return presses. Apple separates them for exactly this reason.
            // Three answers, so the shared helper does not fit: Save keeps
            // index 0 (rightmost, Return), Cancel takes Escape, and Discard
            // sits on the far left where nothing lands by reflex.
            &[
                gpui::PromptButton::ok(t(L10nKey::Save)),
                gpui::PromptButton::cancel(t(L10nKey::Cancel)),
                gpui::PromptButton::ok(t(L10nKey::EditorDiscard)),
            ],
            cx,
        );
        let id = f.input.entity_id();
        cx.spawn_in(window, async move |app, cx| {
            let Ok(choice) = answer.await else { return };
            let _ = app.update_in(cx, |app, window, cx| match choice {
                0 => app.editor_save_file(id, true, window, cx),
                2 => {
                    if let Some((tab_ix, ix)) = app.editor_file_position(id) {
                        app.editor_remove_file_in(tab_ix, ix, cx);
                    }
                }
                _ => {}
            });
        })
        .detach();
    }

    pub(crate) fn editor_close_active_if_focused(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.editor_has_focus(window, cx) {
            return false;
        }
        let Some(code) = self.tab_code_mut() else {
            return false;
        };
        if code.files.is_empty() {
            code.visible = false;
            cx.notify();
            return true;
        }
        let active = code.active;
        self.editor_close_file(active, window, cx);
        true
    }

    fn editor_remove_file(&mut self, ix: usize, cx: &mut Context<Self>) {
        self.editor_remove_file_in(self.active, ix, cx);
    }

    fn editor_remove_file_in(&mut self, tab_ix: usize, ix: usize, cx: &mut Context<Self>) {
        let Some(code) = self
            .tabs
            .get_mut(tab_ix)
            .and_then(|t| t.code.as_deref_mut())
        else {
            return;
        };
        if ix >= code.files.len() {
            return;
        }
        code.files.remove(ix);
        if code.active >= ix && code.active > 0 {
            code.active -= 1;
        }
        self.editor_rebuild_watcher(cx);
        cx.notify();
    }

    pub(crate) fn editor_handle_external_change(
        &mut self,
        path: &Path,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(host) = self.active_host(cx) else {
            return;
        };
        let host_id = host.id();
        let p = path.to_path_buf();
        let landed = p.clone();
        HostOps::run_in(
            host,
            window,
            cx,
            move |h| h.stat(&p).ok().and_then(|m| m.mtime),
            move |app, mtime, window, cx| {
                app.editor_apply_external_change(host_id, &landed, mtime, window, cx)
            },
        );
    }

    fn editor_apply_external_change(
        &mut self,
        host: HostId,
        path: &Path,
        mtime: Option<MTime>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mut reload: Vec<(usize, usize)> = Vec::new();
        let mut changed = false;
        for (tab_ix, tab) in self.tabs.iter_mut().enumerate() {
            let Some(code) = tab.code.as_deref_mut() else {
                continue;
            };
            for (ix, f) in code.files.iter_mut().enumerate() {
                if f.host.id() != host || f.path != *path {
                    continue;
                }
                match classify_external_change(f.saving.is_some(), f.dirty, f.disk_mtime, mtime) {
                    ExternalChange::Ignore => {}
                    ExternalChange::Conflict => {
                        f.conflict = true;
                        changed = true;
                    }
                    ExternalChange::Reload => reload.push((tab_ix, ix)),
                }
            }
        }
        for (tab_ix, ix) in reload {
            self.editor_reload_from_disk(tab_ix, ix, window, cx);
        }
        if changed {
            cx.notify();
        }
    }

    pub(crate) fn editor_reload_from_disk(
        &mut self,
        tab_ix: usize,
        ix: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(f) = self
            .tabs
            .get_mut(tab_ix)
            .and_then(|t| t.code.as_deref_mut())
            .and_then(|c| c.files.get_mut(ix))
        else {
            return;
        };
        let target = f.path.clone();
        let host = f.host.clone();
        let id = f.input.entity_id();
        f.reload_seq = f.reload_seq.wrapping_add(1);
        let seq = f.reload_seq;
        HostOps::run_in(
            host,
            window,
            cx,
            move |h| {
                let bytes = h.read_file(&target, MAX_FILE_BYTES)?;
                let text = String::from_utf8(bytes).map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "not valid UTF-8")
                })?;
                let mtime = h.stat(&target).ok().and_then(|m| m.mtime);
                Ok((text, mtime))
            },
            move |app, result: std::io::Result<(String, Option<MTime>)>, window, cx| {
                let Some(f) = app.editor_file_mut(id) else {
                    return;
                };
                if f.reload_seq != seq {
                    return;
                }
                let Ok((text, mtime)) = result else {
                    f.dirty = true;
                    f.conflict = false;
                    cx.notify();
                    return;
                };
                f.disk_mtime = mtime;
                f.dirty = false;
                f.conflict = false;
                f.edit_seq = f.edit_seq.wrapping_add(1);
                let input = f.input.clone();
                input.update(cx, |input, cx| input.set_value(text, window, cx));
                cx.notify();
            },
        );
    }
}

impl Tty7App {
    pub(crate) fn render_code_overlay(
        &mut self,
        chrome: DocumentChrome,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        if !self.code_panel_visible() {
            return None;
        }
        let body = match self.tab_code().and_then(|c| c.active_file()) {
            None => self.render_editor_empty(cx).into_any_element(),
            Some(f) if f.preview => {
                let reading = f.reading.as_ref().expect("Markdown preview state").clone();
                let revision = f.edit_seq;
                reading.update(cx, |reading, cx| reading.sync(revision, cx));
                // Keep the expensive Markdown layout while the document pane
                // changes between docked and filled modes. The preview entity
                // remains the same, so GPUI can reuse its measured tree until
                // the preview notifies it about a real document or style change.
                gpui::AnyView::from(reading)
                    .cached(gpui::StyleRefinement::default())
                    .into_any_element()
            }
            Some(f) => {
                f.prepare_source(cx);
                let input = f.input.clone();
                Input::new(&input)
                    .appearance(false)
                    .editor_style(crate::ui::editor_theme::current(cx))
                    .font_family(crate::ui::editor_theme::font_family(cx))
                    .text_size(px(cx
                        .global::<crate::core::config::Config>()
                        .editor_font_size))
                    .line_height(gpui::relative(
                        cx.global::<crate::core::config::Config>()
                            .editor_line_height,
                    ))
                    .size_full()
                    .into_any_element()
            }
        };
        let conflict_banner = self
            .tab_code()
            .and_then(|c| c.active_file())
            .filter(|f| f.conflict)
            .map(|_| self.render_editor_conflict_banner(cx));

        let header = chrome
            .renders_own_header()
            .then(|| self.render_editor_header(chrome, window, cx));
        let editor_col = v_flex()
            .flex_1()
            .min_w_0()
            .h_full()
            .children(header)
            .when_some(conflict_banner, |this, b| this.child(b))
            .child(div().flex_1().min_h_0().child(body));

        // The panel's own paint is the same either way; only the box is not.
        // Filling the workspace means stopping the window's translucency and
        // repainting the theme image the root's copy now sits under; docking
        // means sitting in the same plane as the right panel, which the column
        // wrapper has already painted.
        let shell = v_flex()
            .id("code-panel")
            .debug_selector(|| "code-panel".into());
        let shell = match chrome {
            DocumentChrome::Fill => shell
                .absolute()
                .inset_0()
                .occlude()
                // Opaque on purpose: this overlay covers the whole workspace
                // (everything but the detail panel) and an open file must never
                // let the window translucency / backdrop material show through
                // it. The preset's gradient fill is preserved, just with
                // alpha 1 — the same paint the settings overlay uses. The
                // theme background image is repainted on top of it, since the
                // root's copy now sits below this fill.
                .bg(crate::ui::theme::overlay_background(cx))
                .children(crate::ui::app::overlay_surface_layers(cx)),
            DocumentChrome::Dock | DocumentChrome::DockHoisted => shell.size_full().min_w_0(),
        };
        let shell = shell.when(
            !chrome.is_dock() && self.document_header_below_chrome(window, cx),
            |shell| shell.pt(px(crate::ui::app::TITLE_BAR_HEIGHT)),
        );
        Some(
            shell
                .on_key_down(cx.listener(|this, ev: &gpui::KeyDownEvent, window, cx| {
                    if ev.keystroke.key == "escape" {
                        this.toggle_code_panel(window, cx);
                    }
                }))
                .child(h_flex().flex_1().min_h_0().w_full().child(editor_col))
                .child(self.render_code_status_bar(window, cx))
                .into_any_element(),
        )
    }

    /// The editor header alone, for the strip above a docked column.
    pub(crate) fn render_editor_header_only(
        &self,
        chrome: DocumentChrome,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        self.render_editor_header(chrome, window, cx)
            .into_any_element()
    }

    fn render_editor_header(
        &self,
        chrome: DocumentChrome,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let active = self.tab_code().and_then(|c| c.active_file());
        let name = active.map(|f| f.label());
        let dirty = active.is_some_and(|f| f.dirty);
        // `TITLE_BAR_LEAD` is the room macOS's traffic lights need. Only a
        // header that starts at the left edge of the window has them to clear,
        // and a docked column never does.
        let lead = if self.left_panel_open(cx) || chrome.is_dock() {
            crate::ui::app::CONTENT_INSET
        } else {
            crate::ui::app::TITLE_BAR_LEAD
        };
        let below_chrome = !chrome.is_dock() && self.document_header_below_chrome(window, cx);
        let row = h_flex().id("editor-header");
        let row = if chrome.header_is_title_strip() && !below_chrome {
            crate::ui::app::title_bar_drag(row, "editor-header", window, cx)
        } else {
            row
        };
        let menu_app = cx.entity().downgrade();
        row.flex_none()
            .h(px(crate::ui::app::TITLE_BAR_HEIGHT))
            .items_center()
            .gap_1p5()
            .pl(px(lead))
            .pr(px(crate::ui::app::tile_trailing_inset()))
            .when(
                chrome.renders_own_header() && !self.right_panel_open(cx) && !below_chrome,
                |row| {
                    row.pr(px(crate::ui::app::tile_trailing_inset()
                        + crate::ui::tab_strip::trailing_chrome_w()))
                },
            )
            .border_b_1()
            .border_color(cx.theme().border)
            .children(name.as_ref().map(|name| {
                crate::ui::file_icons::FileIcon::for_file(name)
                    .render(px(crate::ui::file_icons::ROW_ICON), window)
            }))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_ellipsis()
                    .text_sm()
                    .when(name.is_none(), |d| {
                        d.text_color(cx.theme().muted_foreground)
                    })
                    .child(
                        name.unwrap_or_else(|| SharedString::from(t(L10nKey::EditorNoFileOpen))),
                    ),
            )
            .when(dirty, |d| {
                d.child(
                    div()
                        .flex_none()
                        .size(px(6.))
                        .rounded_full()
                        .bg(cx.theme().warning),
                )
            })
            .child(self.document_fill_button(cx))
            .child(
                div().occlude().flex_shrink_0().child(
                    crate::ui::tab_strip::chrome_tile_sized(
                        Button::new("editor-panel-close").icon(Icon::new(IconName::Close)),
                        crate::ui::app::TILE_SIZE,
                        crate::ui::app::TILE_GLYPH_LINE,
                        false,
                        cx,
                    )
                    .rounded_lg()
                    .tooltip(t(L10nKey::EditorBackToTerminal))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.toggle_code_panel(window, cx);
                    })),
                ),
            )
            .context_menu(move |menu, _window, cx| {
                Tty7App::document_header_menu(menu, &menu_app, cx)
            })
    }

    fn render_code_status_bar(&self, _window: &Window, cx: &mut Context<Self>) -> gpui::Div {
        // The roots below belong to this window's own machine. A file read
        // over SFTP is on another one, where they mean nothing, so it shows
        // its own full path rather than borrowing the local repo's name.
        let tree_host = self.spawn_host(cx);
        let code = self.tab_code();
        let muted = cx.theme().muted_foreground;
        let path_text: Option<SharedString> = code.map(|c| {
            let repo = c
                .roots
                .first()
                .and_then(|r| r.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            match c.active_file() {
                Some(f) if f.host.id() != tree_host => f.path.display().to_string().into(),
                Some(f) => {
                    let rel = c
                        .roots
                        .iter()
                        .find_map(|r| f.path.strip_prefix(r).ok())
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| f.label().to_string());
                    format!("{repo} › {rel}").into()
                }
                None => repo.into(),
            }
        });
        let active = code.and_then(|c| c.active_file());
        let cursor: Option<SharedString> = active.filter(|f| !f.preview).map(|f| {
            let pos = f.input.read(cx).cursor_position();
            t_fmt(
                L10nKey::EditorLnCol,
                &[
                    ("line", &(pos.line + 1).to_string()),
                    ("column", &(pos.character + 1).to_string()),
                ],
            )
            .into()
        });
        let wrap: Option<bool> = active.filter(|f| !f.preview).map(|f| f.wrap);
        let is_markdown = active.is_some_and(|f| language_for_path(&f.path) == "markdown");
        let preview = active.is_some_and(|f| f.preview);

        h_flex()
            .flex_none()
            .w_full()
            .h(px(26.))
            .bg(cx.theme().background)
            .items_center()
            .gap_3()
            .px_3()
            .border_t_1()
            .border_color(cx.theme().border)
            .text_xs()
            .text_color(muted)
            .child(div().flex_1().min_w_0().truncate().children(path_text))
            .when(is_markdown, |this| {
                this.child(
                    Button::new("status-md-preview")
                        .label(if preview {
                            t(L10nKey::EditorEdit)
                        } else {
                            t(L10nKey::EditorPreview)
                        })
                        .custom(crate::ui::tab_strip::chrome_tile_variant(cx))
                        .xsmall()
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.editor_toggle_preview(window, cx)
                        })),
                )
            })
            .when_some(wrap, |this, wrap| {
                this.child(
                    Button::new("status-wrap")
                        .label(if wrap {
                            t(L10nKey::EditorWrapOn)
                        } else {
                            t(L10nKey::EditorWrapOff)
                        })
                        .custom(crate::ui::tab_strip::chrome_tile_variant(cx))
                        .xsmall()
                        .on_click(cx.listener(|this, _, window, cx| {
                            let Some(code) = this.tab_code_mut() else {
                                return;
                            };
                            let ix = code.active;
                            if let Some(f) = code.files.get_mut(ix) {
                                f.wrap = !f.wrap;
                                let wrap = f.wrap;
                                f.input.clone().update(cx, |st, cx| {
                                    st.set_soft_wrap(wrap, window, cx);
                                });
                            }
                        })),
                )
            })
            .when_some(cursor, |this, t| {
                this.child(div().flex_none().whitespace_nowrap().child(t))
            })
    }

    fn render_editor_empty(&self, cx: &Context<Self>) -> gpui::Div {
        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .gap_2()
            .child(
                Icon::new(IconName::File)
                    .large()
                    .text_color(cx.theme().muted_foreground),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(crate::ui::i18n::t(
                        crate::ui::i18n::L10nKey::OpenFileFromTree,
                    )),
            )
    }

    fn render_editor_conflict_banner(&self, cx: &mut Context<Self>) -> AnyElement {
        let tab_ix = self.active;
        let ix = self.tab_code().map(|c| c.active).unwrap_or(0);
        h_flex()
            .flex_none()
            .w_full()
            .items_center()
            .gap_2()
            .px_2()
            .py_1()
            .bg(cx.theme().warning.opacity(0.15))
            .border_b_1()
            .border_color(cx.theme().border)
            .text_sm()
            .child(div().flex_1().child(crate::ui::i18n::t(
                crate::ui::i18n::L10nKey::FileChangedOnDisk,
            )))
            .child(
                Button::new("editor-conflict-reload")
                    .label(crate::ui::i18n::t(crate::ui::i18n::L10nKey::Reload))
                    .small()
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.editor_reload_from_disk(tab_ix, ix, window, cx);
                    })),
            )
            .child(
                Button::new("editor-conflict-keep")
                    .label(crate::ui::i18n::t(crate::ui::i18n::L10nKey::KeepMine))
                    .ghost()
                    .small()
                    .on_click(cx.listener(move |this, _, _w, cx| {
                        if let Some(f) = this.tab_code_mut().and_then(|c| c.files.get_mut(ix)) {
                            f.conflict = false;
                            cx.notify();
                        }
                    })),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{TestAppContext, VisualTestContext};

    fn packed(color: gpui::Hsla) -> u32 {
        let color = crate::terminal::palette::hsla_to_rgb(color);
        (color.r as u32) << 16 | (color.g as u32) << 8 | color.b as u32
    }

    #[gpui::test]
    fn editor_search_highlights_follow_the_resolved_editor_palette(cx: &mut TestAppContext) {
        use crate::ui::{editor_theme, presets};
        let (app, mut vcx) = markdown_window(cx);
        for preset in presets::builtins() {
            app.update_in(&mut vcx, |app, window, cx| {
                app.set_theme_follow_system(false, window, cx);
                app.set_preset(&preset.id, window, cx);
                let base = editor_theme::current(cx);
                let background = packed(base.highlight_theme.style.editor_background.unwrap());
                let foreground = packed(base.highlight_theme.style.editor_foreground.unwrap());
                for accent in [preset.neutrals().accent, 0x434750, background] {
                    cx.set_global(presets::ActiveAccent(accent));
                    let style = editor_theme::current(cx);
                    let hit = packed(style.search_match);
                    let active = packed(style.search_match_active);
                    assert!(
                        presets::contrast(hit, active) >= 1.2,
                        "{}: current and other matches must differ",
                        preset.id
                    );
                    assert!(presets::contrast(background, active) >= 1.89);
                    assert_eq!(style.search_match.a, 1.0);
                    assert_eq!(style.search_match_active.a, 1.0);
                    for fill in [hit, active] {
                        assert!(presets::contrast(foreground, fill) >= 2.95);
                    }
                    assert_eq!(style.selection, base.selection);
                    assert_eq!(style.caret, base.caret);
                    assert_eq!(style.highlight_theme, base.highlight_theme);
                    assert_eq!(style.search_match, base.search_match);
                }
            });
        }
    }

    fn markdown_window(cx: &mut TestAppContext) -> (Entity<Tty7App>, VisualTestContext) {
        let (app, mut vcx) = crate::ui::app::test_window::harness(cx);
        app.update_in(&mut vcx, |app, _, cx| {
            crate::ui::markdown_preview::init(Default::default(), cx);
            app.tabs
                .push(crate::ui::app::Tab::new(crate::ui::pane::Pane::Empty));
            app.active = app.tabs.len() - 1;
            cx.notify();
        });
        (app, vcx)
    }

    #[gpui::test]
    fn editor_palette_dismiss_restores_source_and_reading_focus(cx: &mut TestAppContext) {
        let (app, mut vcx) = markdown_window(cx);
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_install_file(
                tty7_core::host::local::LocalHost::new(),
                "/focus.md".into(),
                "# Focus\n".into(),
                None,
                window,
                cx,
            );
        });
        for _ in 0..2 {
            app.update_in(&mut vcx, |app, window, cx| {
                assert!(app.editor_has_focus(window, cx));
                app.open_palette("theme", window, cx);
                assert!(!app.editor_has_focus(window, cx));
                app.close_palette(window, cx);
                assert!(
                    app.editor_has_focus(window, cx),
                    "closing the palette must return the keyboard to the document"
                );
                app.editor_toggle_preview(window, cx);
            });
        }
    }

    fn wait_for_markdown(
        vcx: &mut VisualTestContext,
        mut ready: impl FnMut(&mut VisualTestContext) -> bool,
    ) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            vcx.run_until_parked();
            if ready(vcx) {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "Markdown operation did not settle"
            );
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        vcx.update(|window, cx| {
            let _ = window.draw(cx);
        });
    }

    #[gpui::test]
    fn editor_theme_colors_rust_bindings_without_recoloring_module_paths(cx: &mut TestAppContext) {
        use crate::ui::editor_theme;
        use gpui_component::highlighter::SyntaxHighlighter;
        let (app, mut vcx) = markdown_window(cx);
        let source = "use std::borrow::Cow;\nmod core;\nfn make(cx: &mut App) {\n    let fonts = \"a\\n\";\n    let file: std::path::PathBuf;\n    cx.add_fonts(fonts);\n    let outer = LIMIT;\n    let generic = Some::<u8>(1);\n    let qualified = Option::Some::<u16>(1);\n    wrap! { let inner = load_fonts(); let count = OTHER_LIMIT; let maybe = Some(inner); }\n}\n";
        for (preference, variable, keyword, function, ty, foreground) in [
            ("dark", 0xe06c75, 0xc678dd, 0x61afef, 0x56b6c2, 0xabb2bf),
            ("light", 0x1f2328, 0xcf222e, 0x6639ba, 0x1f2328, 0x1f2328),
        ] {
            app.update_in(&mut vcx, |app, window, cx| {
                app.set_theme_follow_system(false, window, cx);
                app.set_preset(preference, window, cx);
                let style = editor_theme::current(cx);
                let mut highlighter = SyntaxHighlighter::new(editor_theme::language("rust"));
                highlighter.update(None, &source.into(), None);
                let runs = highlighter.styles(&(0..source.len()), &style.highlight_theme);
                for (token, expected) in [
                    ("fonts", variable),
                    ("fn", keyword),
                    ("make", function),
                    ("App", ty),
                    ("std", foreground),
                    ("borrow", foreground),
                    ("core", foreground),
                    ("path", foreground),
                    ("load_fonts", function),
                    ("inner", variable),
                    (
                        "LIMIT",
                        if preference == "dark" {
                            0xd19a66
                        } else {
                            0x0550ae
                        },
                    ),
                    (
                        "OTHER_LIMIT",
                        if preference == "dark" {
                            0xd19a66
                        } else {
                            0x0550ae
                        },
                    ),
                    ("Some::<u8>", function),
                    ("Some::<u16>", function),
                    ("Some(inner)", function),
                    ("\\n", if preference == "dark" { ty } else { 0x0550ae }),
                ] {
                    let offset = source.find(token).unwrap();
                    let color = runs
                        .iter()
                        .find(|(range, _)| range.contains(&offset))
                        .and_then(|(_, run)| run.color)
                        .or(style.highlight_theme.style.editor_foreground);
                    assert_eq!(
                        color,
                        Some(gpui::rgb(expected).into()),
                        "{preference}: {token}"
                    );
                }
            });
        }
    }

    #[gpui::test]
    fn editor_theme_python_variables_match_the_palette_foreground(cx: &mut TestAppContext) {
        use crate::ui::editor_theme;
        use gpui_component::highlighter::SyntaxHighlighter;
        let (app, mut vcx) = markdown_window(cx);
        let source = "import json\nDEST = Path(__file__).parent\nroot = Path(\"icons\")\nversion = json.loads(root.read_text())\n";
        for (preference, foreground) in [("dark", 0xabb2bf), ("light", 0x1f2328)] {
            app.update_in(&mut vcx, |app, window, cx| {
                app.set_theme_follow_system(false, window, cx);
                app.set_preset(preference, window, cx);
                let style = editor_theme::current(cx);
                let mut highlighter = SyntaxHighlighter::new(editor_theme::language("python"));
                highlighter.update(None, &source.into(), None);
                let runs = highlighter.styles(&(0..source.len()), &style.highlight_theme);
                for token in ["json", "DEST", "root", "version", "parent"] {
                    let offset = source.find(token).unwrap();
                    let color = runs
                        .iter()
                        .find(|(range, _)| range.contains(&offset))
                        .and_then(|(_, run)| run.color)
                        .or(style.highlight_theme.style.editor_foreground);
                    assert_eq!(
                        color,
                        Some(gpui::rgb(foreground).into()),
                        "{preference}: {token}"
                    );
                }
            });
        }
    }

    #[test]
    fn editor_theme_preserves_exact_capture_styles_and_base_fallbacks() {
        let colors: gpui_component::highlighter::SyntaxColors = serde_json::from_str(
            r##"{"variable":{"color":"#E06C75"},"variable.parameter":{"color":"#ABB2BF"},"function":{"color":"#61AFEF"},"function.builtin":{"color":"#56B6C2"}}"##,
        ).unwrap();
        for (capture, expected) in [
            ("variable.parameter", 0xabb2bf),
            ("variable.member", 0xe06c75),
            ("function.builtin", 0x56b6c2),
            ("function.method", 0x61afef),
        ] {
            assert_eq!(
                colors.style(capture).unwrap().color,
                Some(gpui::rgb(expected).into()),
                "{capture}"
            );
        }
    }

    #[gpui::test]
    fn editor_theme_matches_authored_color_samples(cx: &mut TestAppContext) {
        use crate::ui::editor_theme;
        use gpui_component::highlighter::SyntaxHighlighter;
        #[derive(serde::Deserialize)]
        struct Token {
            text: String,
            start: usize,
            end: usize,
            color: String,
        }
        #[derive(serde::Deserialize)]
        struct Sample {
            mode: String,
            language: String,
            source: String,
            tokens: Vec<Token>,
        }
        let mut samples: Vec<Sample> = serde_json::from_str(include_str!(
            "../../assets/editor-themes/tests/textmate-colors.json"
        ))
        .unwrap();
        samples.extend(
            serde_json::from_str::<Vec<Sample>>(include_str!(
                "../../assets/editor-themes/tests/github-light-colors.json"
            ))
            .unwrap(),
        );
        samples.extend(
            serde_json::from_str::<Vec<Sample>>(include_str!(
                "../../assets/editor-themes/tests/github-website-colors.json"
            ))
            .unwrap(),
        );
        let (app, mut vcx) = markdown_window(cx);
        let mut failures = Vec::new();
        for sample in samples {
            app.update_in(&mut vcx, |app, window, cx| {
                app.set_theme_follow_system(false, window, cx);
                app.set_preset(&sample.mode, window, cx);
                let style = editor_theme::current(cx);
                // The registry owns static language names, just like path detection.
                let language = gpui_component::highlighter::Language::all()
                    .find(|language| language.name() == sample.language)
                    .unwrap()
                    .name();
                let alias = editor_theme::language(language);
                let mut highlighter = SyntaxHighlighter::new(alias);
                if highlighter.language().as_ref() != alias {
                    failures.push(format!("{language} query must compile"));
                    return;
                }
                highlighter.update(None, &sample.source.as_str().into(), None);
                let runs = highlighter.styles(&(0..sample.source.len()), &style.highlight_theme);
                for token in &sample.tokens {
                    assert_eq!(
                        &sample.source[token.start..token.end],
                        token.text,
                        "{} {}: reference token range",
                        sample.mode,
                        language
                    );
                    let expected =
                        u32::from_str_radix(token.color.trim_start_matches('#'), 16).unwrap();
                    for offset in token.start..token.end {
                        if sample.source.as_bytes()[offset].is_ascii_whitespace() {
                            continue;
                        }
                        let color = runs
                            .iter()
                            .find(|(range, _)| range.contains(&offset))
                            .and_then(|(_, run)| run.color)
                            .or(style.highlight_theme.style.editor_foreground);
                        if color != Some(gpui::rgb(expected).into()) {
                            failures.push(format!(
                                "{} {} {:?} at {}: expected {}, got {:?}",
                                sample.mode,
                                language,
                                token.text,
                                offset,
                                token.color,
                                color.map(crate::terminal::palette::hsla_to_rgb)
                            ));
                            break;
                        }
                    }
                }
            });
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    #[gpui::test]
    fn editor_theme_queries_compile_without_changing_shared_languages(cx: &mut TestAppContext) {
        let _ = markdown_window(cx);
        use gpui_component::highlighter::{Language, LanguageRegistry, SyntaxHighlighter};
        let registry = LanguageRegistry::singleton();
        let originals: Vec<_> = Language::all()
            .map(|language| (language.name(), registry.language(language.name()).unwrap()))
            .collect();
        let mut errors = Vec::new();
        for (name, original) in originals {
            let alias = crate::ui::editor_theme::language(name);
            let config = registry.language(alias).unwrap();
            if let Err(error) = tree_sitter::Query::new(
                &config.language,
                &format!(
                    "{}{}{}",
                    config.injections, config.locals, config.highlights
                ),
            ) {
                errors.push(format!("{name}: {error}"));
                continue;
            }
            let highlighter = SyntaxHighlighter::new(alias);
            assert_eq!(highlighter.language().as_ref(), alias, "{name}");
            assert_eq!(registry.language(name).unwrap(), original, "shared {name}");
        }
        assert!(errors.is_empty(), "{}", errors.join("\n"));
    }

    #[gpui::test]
    fn editor_theme_control_colors_match_the_authored_sources(cx: &mut TestAppContext) {
        let (app, mut vcx) = markdown_window(cx);
        for (preference, source) in [
            (
                "dark",
                include_str!("../../assets/editor-themes/upstream/OneDark.json"),
            ),
            (
                "light",
                include_str!("../../assets/editor-themes/upstream/GitHubLight.json"),
            ),
        ] {
            let source: serde_json::Value = serde_json::from_str(source).unwrap();
            app.update_in(&mut vcx, |app, window, cx| {
                app.set_theme_follow_system(false, window, cx);
                app.set_preset(preference, window, cx);
                let style = crate::ui::editor_theme::current(cx);
                let colors = &style.highlight_theme.style;
                for (actual, key, github_key) in [
                    (
                        colors.editor_background.unwrap(),
                        "editor.background",
                        "codeMirror-bgColor",
                    ),
                    (
                        colors.editor_foreground.unwrap(),
                        "editor.foreground",
                        "codeMirror-fgColor",
                    ),
                    (
                        colors.editor_gutter_background.unwrap(),
                        "editor.background",
                        "codeMirror-gutters-bgColor",
                    ),
                    (
                        colors.editor_active_line.unwrap(),
                        "editor.lineHighlightBackground",
                        "codeMirror-activeline-bgColor",
                    ),
                    (
                        colors.editor_line_number.unwrap(),
                        "editorLineNumber.foreground",
                        "codeMirror-lineNumber-fgColor",
                    ),
                    (
                        colors.editor_active_line_number.unwrap(),
                        "editorLineNumber.activeForeground",
                        "codeMirror-fgColor",
                    ),
                    (
                        colors.editor_invisible.unwrap(),
                        "editorWhitespace.foreground",
                        "borderColor-default",
                    ),
                    (
                        style.selection,
                        "editor.selectionBackground",
                        "selection-bgColor",
                    ),
                    (
                        style.caret,
                        "editorCursor.foreground",
                        "codeMirror-cursor-fgColor",
                    ),
                    (
                        style.muted_foreground,
                        "editorLineNumber.foreground",
                        "fgColor-muted",
                    ),
                    (
                        style.border,
                        "editorIndentGuide.background",
                        "borderColor-default",
                    ),
                ] {
                    let value = if preference == "dark" {
                        &source["colors"][key]
                    } else {
                        &source["tokens"][github_key]
                    };
                    let expected: gpui::Hsla = serde_json::from_value(value.clone()).unwrap();
                    assert_eq!(actual, expected, "{preference}: {key}");
                }
            });
        }
    }

    #[gpui::test]
    fn editor_theme_preserves_language_overrides_inside_embedded_code(cx: &mut TestAppContext) {
        let (app, mut vcx) = markdown_window(cx);
        for (preference, foreground, orange, cyan, red, green, blue) in [
            (
                "dark", 0xabb2bf, 0xd19a66, 0x56b6c2, 0xe06c75, 0x98c379, 0x61afef,
            ),
            (
                "light", 0x1f2328, 0x0550ae, 0x0550ae, 0x953800, 0x0a3069, 0x6639ba,
            ),
        ] {
            app.update_in(&mut vcx, |app, window, cx| {
                app.set_theme_follow_system(false, window, cx);
                app.set_preset(preference, window, cx);
                for (language, source, tokens) in [
                    ("html", "<script>let root = obj.parent + 2;</script>", vec![("root", foreground), ("=", if preference == "dark" { cyan } else { 0xcf222e }), ("parent", red), ("2", orange)]),
                    ("tsx", "const root: string = obj.parent + 2; const view = <div id=\"main\">{root}</div>;", vec![("root", orange), ("string", cyan), ("parent", red), ("div", if preference == "dark" { red } else { 0x0550ae }), ("id", orange), ("main", green)]),
                    ("javascript", "const view = <div id=\"main\">{item}</div>;", vec![("div", if preference == "dark" { red } else { 0x0550ae }), ("id", orange), ("main", green), ("item", foreground)]),
                    ("markdown", "```rust\nstruct Widget;\n```\n", vec![("Widget", if preference == "dark" { cyan } else { foreground })]),
                    ("python", "from pathlib import Path\nDEST = Path(__file__).parent\n", vec![("Path", foreground), ("Path(__file__)", blue), ("__file__", if preference == "dark" { red } else { 0x0550ae }), ("parent", foreground)]),
                ] {
                    let style = crate::ui::editor_theme::current(cx);
                    let mut highlighter = gpui_component::highlighter::SyntaxHighlighter::new(crate::ui::editor_theme::language(language));
                    highlighter.update(None, &source.into(), None);
                    let runs = highlighter.styles(&(0..source.len()), &style.highlight_theme);
                    for (token, expected) in tokens {
                        let offset = source.find(token).unwrap();
                        let color = runs.iter().find(|(range, _)| range.contains(&offset)).and_then(|(_, run)| run.color).or(style.highlight_theme.style.editor_foreground);
                        assert_eq!(color, Some(gpui::rgb(expected).into()), "{preference} {language}: {token}");
                    }
                }
            });
        }
    }

    #[gpui::test]
    fn editor_theme_highlights_cross_language_samples(cx: &mut TestAppContext) {
        use crate::ui::editor_theme;
        use gpui_component::highlighter::{LanguageRegistry, SyntaxHighlighter};
        let (app, mut vcx) = markdown_window(cx);
        let shared_rust = LanguageRegistry::singleton().language("rust").unwrap();
        editor_theme::language("rust");
        assert_eq!(
            LanguageRegistry::singleton().language("rust").unwrap(),
            shared_rust
        );
        for (preference, keyword, function, string, number) in [
            ("dark", 0xc678dd, 0x61afef, 0x98c379, 0xd19a66),
            ("light", 0xcf222e, 0x6639ba, 0x0a3069, 0x0550ae),
        ] {
            app.update_in(&mut vcx, |app, window, cx| {
                app.set_theme_follow_system(false, window, cx);
                app.set_preset(preference, window, cx);
                for (language, source, tokens) in [
                    (
                        "typescript",
                        "function greet() { return \"hello\"; }",
                        vec![
                            ("function", keyword),
                            ("greet", function),
                            ("hello", string),
                        ],
                    ),
                    (
                        "javascript",
                        "function greet() { return 42; }",
                        vec![("function", keyword), ("greet", function), ("42", number)],
                    ),
                    (
                        "python",
                        "def greet():\n    return \"hello\"",
                        vec![("def", keyword), ("greet", function), ("hello", string)],
                    ),
                    (
                        "json",
                        "{\"count\": 42, \"text\": \"hello\"}",
                        vec![("42", number), ("hello", string)],
                    ),
                ] {
                    let style = editor_theme::current(cx);
                    let mut highlighter = SyntaxHighlighter::new(editor_theme::language(language));
                    highlighter.update(None, &source.into(), None);
                    let runs = highlighter.styles(&(0..source.len()), &style.highlight_theme);
                    for (token, expected) in tokens {
                        let offset = source.find(token).unwrap();
                        let color = runs
                            .iter()
                            .find(|(range, _)| range.contains(&offset))
                            .and_then(|(_, run)| run.color);
                        assert_eq!(
                            color,
                            Some(gpui::rgb(expected).into()),
                            "{preference} {language}: {token}"
                        );
                    }
                }
            });
        }
    }

    #[gpui::test]
    fn editor_theme_switches_with_app_appearance_and_preserves_edits(cx: &mut TestAppContext) {
        use crate::core::config::Config;
        use crate::ui::editor_theme;
        let (app, mut vcx) = markdown_window(cx);
        let original = "fn main() {}\n".repeat(150);
        let host: SharedHost = tty7_core::host::local::LocalHost::new();
        app.update_in(&mut vcx, |app, window, cx| {
            let mut themes = crate::ui::presets::builtins();
            for (id, base, background) in [
                ("custom-dark-name", "light", 0xf1ede1),
                ("custom-light-name", "dark", 0x121827),
            ] {
                let mut theme = themes
                    .iter()
                    .find(|theme| theme.id == base)
                    .unwrap()
                    .clone();
                theme.id = id.into();
                theme.name = id.into();
                theme.background = crate::ui::presets::Fill::Vertical {
                    top: background,
                    bottom: background,
                };
                theme.opacity = Some(0.6);
                themes.push(theme);
            }
            cx.set_global(crate::ui::presets::Themes(themes));
            app.set_theme_follow_system(false, window, cx);
            app.set_preset("light", window, cx);
            app.editor_install_file(host, "/theme.rs".into(), original.clone(), None, window, cx);
        });
        let input = app.read_with(&vcx, |app, _| {
            app.tab_code().unwrap().active_file().unwrap().input.clone()
        });
        vcx.run_until_parked();
        vcx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        vcx.simulate_keystrokes("x");
        vcx.simulate_keystrokes("shift-right shift-right");
        input.update(&mut vcx, |input, cx| {
            input.set_scroll_offset(gpui::point(px(0.), px(-250.)), cx)
        });
        vcx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let edited = input.read_with(&vcx, |input, _| input.value());
        let cursor = input.read_with(&vcx, |input, _| input.cursor_position());
        let selected = input.read_with(&vcx, |input, _| input.selected_range());
        let scroll = input.read_with(&vcx, |input, _| input.scroll_offset());
        assert!(!selected.is_empty());
        assert!(scroll.y < px(0.));
        for (legacy, preset, background) in [
            ("auto", "light", 0xffffff),
            ("auto", "dark", 0x282c34),
            ("atom_one_dark", "light", 0xffffff),
            ("atom_one_light", "dark", 0x282c34),
            ("one_dark_pro", "light", 0xffffff),
            ("one_dark_pro", "one_dark_pro", 0x282c34),
            ("future-theme", "dracula", 0x282c34),
            ("auto", "nord", 0x282c34),
            ("atom_one_dark", "custom-dark-name", 0xffffff),
            ("atom_one_light", "custom-light-name", 0x282c34),
        ] {
            app.update_in(&mut vcx, |app, window, cx| {
                let mut saved = serde_json::to_value(&cx.global::<Config>().0).unwrap();
                saved["editor_theme"] = serde_json::json!(legacy);
                cx.set_global(Config(serde_json::from_value(saved).unwrap()));
                app.set_preset(preset, window, cx);
                let style = editor_theme::current(cx);
                assert_eq!(
                    style.highlight_theme.style.editor_background,
                    Some(gpui::rgb(background).into())
                );
                assert_eq!(
                    style.highlight_theme.style.editor_gutter_background,
                    Some(gpui::rgb(background).into())
                );
                assert_eq!(
                    style.highlight_theme.style.editor_background.unwrap().a,
                    1.0
                );
            });
            vcx.run_until_parked();
            vcx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            input.read_with(&vcx, |input, _| {
                assert_eq!(input.value(), edited);
                assert_eq!(input.cursor_position(), cursor);
                assert_eq!(input.selected_range(), selected);
                assert_eq!(input.scroll_offset(), scroll);
            });
            app.read_with(&vcx, |app, _| {
                assert!(app.tab_code().unwrap().active_file().unwrap().dirty)
            });
        }
        app.update_in(&mut vcx, |app, window, cx| app.focus_editor(window, cx));
        vcx.simulate_keystrokes("secondary-z");
        input.read_with(&vcx, |input, _| assert_eq!(input.value(), original));
    }

    #[gpui::test]
    fn editor_theme_and_typography_are_independent_and_keep_the_buffer(cx: &mut TestAppContext) {
        use crate::core::config::Config;
        use crate::ui::editor_theme;
        let (app, mut vcx) = markdown_window(cx);
        let original = "// comment\nconst root = obj.parent;\n".repeat(60);
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_install_file(
                tty7_core::host::local::LocalHost::new(),
                "/theme.js".into(),
                original.clone(),
                None,
                window,
                cx,
            );
            app.set_editor_font_family("Hack".into(), cx);
            app.set_editor_font_size(16., cx);
            app.set_editor_line_height(1.8, cx);
        });
        let input = app.read_with(&vcx, |app, _| {
            app.tab_code().unwrap().active_file().unwrap().input.clone()
        });
        vcx.run_until_parked();
        vcx.simulate_keystrokes("x shift-right shift-right");
        let edited = input.read_with(&vcx, |input, _| input.value());
        let selected = input.read_with(&vcx, |input, _| input.selected_range());
        app.update_in(&mut vcx, |app, window, cx| {
            app.set_theme_follow_system(false, window, cx);
            app.set_preset("one_dark_pro", window, cx);
            assert_eq!(editor_theme::resolved_name(cx), "Atom One Dark");
            app.preview_preset("light", window, cx);
            assert_eq!(editor_theme::resolved_name(cx), "GitHub Light");
            app.cancel_preset_preview(window, cx);
            assert_eq!(editor_theme::resolved_name(cx), "Atom One Dark");
            app.set_preset("dracula", window, cx);
            assert_eq!(editor_theme::resolved_name(cx), "Atom One Dark");
            app.set_preset("light", window, cx);
            assert_eq!(editor_theme::resolved_name(cx), "GitHub Light");
            assert_eq!(cx.global::<Config>().editor_font_size, 16.);
            assert_eq!(cx.global::<Config>().editor_line_height, 1.8);
            assert_eq!(editor_theme::font_family(cx).as_ref(), "Hack");
            let component_size = cx.theme().mono_font_size;
            let terminal_size = cx.global::<Config>().font_size;
            app.focus_editor(window, cx);
            app.change_focused_font_size(1., window, cx);
            assert_eq!(cx.global::<Config>().editor_font_size, 17.);
            assert_eq!(cx.global::<Config>().font_size, terminal_size);
            assert_eq!(cx.theme().mono_font_size, component_size);
            app.change_ui_font_size(2., cx);
        });
        vcx.run_until_parked();
        vcx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        input.read_with(&vcx, |input, _| {
            assert_eq!(input.value(), edited);
            assert_eq!(input.selected_range(), selected);
            // GPUI rounds line heights to whole logical pixels.
            assert_eq!(
                input.line_height(),
                Some(px((17.0_f32 * 1.8).round())),
                "source line height uses its font size, independently of UI scale"
            );
        });
        app.read_with(&vcx, |app, _| {
            assert!(app.tab_code().unwrap().active_file().unwrap().dirty)
        });
        app.update_in(&mut vcx, |app, window, cx| {
            app.set_editor_font_family("terminal".into(), cx);
            assert_eq!(
                editor_theme::font_family(cx).as_ref(),
                cx.global::<Config>().font_family
            );
            app.reset_focused_font_size(window, cx);
            assert_eq!(cx.global::<Config>().editor_font_size, 13.);
            app.focus_editor(window, cx);
        });
        vcx.simulate_keystrokes("secondary-z");
        input.read_with(&vcx, |input, _| assert_eq!(input.value(), original));
    }

    #[gpui::test]
    fn editor_theme_tracks_preview_cancel_and_config_reload(cx: &mut TestAppContext) {
        use crate::core::config::{Config, LoadOutcome};
        use crate::ui::editor_theme;
        let (app, mut vcx) = markdown_window(cx);
        vcx.update(|_, cx| crate::ui::windows::WindowRegistry::init(cx));
        app.update_in(&mut vcx, |app, window, cx| {
            app.set_theme_follow_system(false, window, cx);
            app.set_preset("light", window, cx);
            app.preview_preset("dark", window, cx);
            assert_eq!(
                editor_theme::current(cx)
                    .highlight_theme
                    .style
                    .editor_background,
                Some(gpui::rgb(0x282c34).into())
            );
            app.cancel_preset_preview(window, cx);
            assert_eq!(
                editor_theme::current(cx)
                    .highlight_theme
                    .style
                    .editor_background,
                Some(gpui::rgb(0xffffff).into())
            );
        });
        vcx.update(|_, cx| {
            for (preset, background) in [("one_dark_pro", 0x282c34), ("light", 0xffffff)] {
                let mut saved = serde_json::to_value(&cx.global::<Config>().0).unwrap();
                saved["editor_theme"] = serde_json::json!("atom_one_dark");
                saved["theme_preset"] = serde_json::json!(preset);
                let config = Config(serde_json::from_value(saved).unwrap());
                crate::apply_reloaded_config(cx, (config, LoadOutcome::Parsed), &mut false);
                assert_eq!(
                    editor_theme::current(cx)
                        .highlight_theme
                        .style
                        .editor_background,
                    Some(gpui::rgb(background).into())
                );
            }
        });
    }

    #[gpui::test]
    fn markdown_opening_keeps_an_anchor_while_its_background_parse_is_pending(
        cx: &mut TestAppContext,
    ) {
        let (app, mut vcx) = markdown_window(cx);
        let host: SharedHost = tty7_core::host::local::LocalHost::new();
        let path = PathBuf::from("/markdown-tests/background.md");
        let content = (0..40)
            .map(|n| format!("## Section {n}\n\nParagraph {n}.\n\n"))
            .collect::<String>();
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_install_file(host.clone(), path.clone(), content, None, window, cx);
            let reading = app
                .tab_code()
                .unwrap()
                .active_file()
                .unwrap()
                .reading
                .clone()
                .unwrap();
            assert!(
                reading.read(cx).is_loading(cx),
                "opening must enqueue the parse"
            );
            app.editor_open_markdown_link(host, &path, Some("section-30".into()), window, cx);
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
        wait_for_markdown(&mut vcx, |cx| {
            reading.read_with(cx, |reading, cx| !reading.is_loading(cx))
        });
        for _ in 0..4 {
            vcx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            vcx.run_until_parked();
        }
        reading.read_with(&vcx, |reading, cx| {
            let target = reading.text.read(cx).anchor_bounds("section-30").unwrap();
            assert!((target.top() - reading.scroll.bounds().top() - px(12.)).abs() < px(1.));
            assert!(reading.scroll.offset().y < px(-300.));
        });
        vcx.update(|window, cx| {
            assert!(
                gpui_component::Root::read(window, cx)
                    .notification
                    .read(cx)
                    .notifications()
                    .is_empty(),
                "opening must not report a missing anchor"
            );
        });
    }

    #[gpui::test]
    fn repository_readme_opens_and_draws_in_preview(cx: &mut TestAppContext) {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("README.md");
        let host: SharedHost = tty7_core::host::local::LocalHost::new();
        let (app, mut vcx) = markdown_window(cx);
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_open_on_host(host, &path, window, cx)
        });
        wait_for_markdown(&mut vcx, |cx| {
            app.read_with(cx, |app, _| {
                app.tab_code().is_some_and(|code| !code.files.is_empty())
            })
        });
        for _ in 0..3 {
            vcx.update(|window, cx| {
                window.refresh();
                let _ = window.draw(cx);
            });
            vcx.run_until_parked();
        }
        app.read_with(&vcx, |app, _| {
            let file = app.tab_code().unwrap().active_file().unwrap();
            assert!(file.preview);
            assert!(file.reading.is_some());
        });
    }

    #[gpui::test]
    #[ignore = "manual Markdown opening and switching profile; run with --ignored --nocapture"]
    fn markdown_open_profile(cx: &mut TestAppContext) {
        let (app, mut vcx) = markdown_window(cx);
        let host: SharedHost = tty7_core::host::local::LocalHost::new();
        for filename in [
            "README.md",
            "docs/examples/markdown-github-theme.md",
            "README.zh-CN.md",
        ] {
            let source = std::fs::read_to_string(filename).unwrap();
            let started = std::time::Instant::now();
            app.update_in(&mut vcx, |app, window, cx| {
                app.editor_install_file(host.clone(), filename.into(), source, None, window, cx)
            });
            let installed = started.elapsed();
            vcx.update(|window, cx| {
                window.refresh();
                let _ = window.draw(cx);
            });
            let response_frame = started.elapsed();
            wait_for_markdown(&mut vcx, |cx| {
                app.read_with(cx, |app, cx| {
                    !app.tab_code()
                        .unwrap()
                        .active_file()
                        .unwrap()
                        .reading
                        .as_ref()
                        .unwrap()
                        .read(cx)
                        .is_loading(cx)
                })
            });
            vcx.update(|window, cx| {
                window.refresh();
                let _ = window.draw(cx);
            });
            eprintln!(
                "Markdown profile {filename}: install={installed:?}, response frame={response_frame:?}, content drawn={:?}",
                started.elapsed()
            );
        }
        let started = std::time::Instant::now();
        for _ in 0..10 {
            for filename in [
                "README.md",
                "docs/examples/markdown-github-theme.md",
                "README.zh-CN.md",
            ] {
                app.update_in(&mut vcx, |app, window, cx| {
                    assert!(app.editor_activate_open(host.id(), Path::new(filename), window, cx))
                });
                vcx.update(|window, cx| {
                    window.refresh();
                    let _ = window.draw(cx);
                });
            }
        }
        eprintln!("Markdown profile 30 warm switches: {:?}", started.elapsed());
    }

    #[gpui::test]
    fn markdown_opens_in_preview_and_preserves_edit_undo_save_and_focus(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("README.MARKDOWN");
        let original = "# Reading\n\nThe source buffer owns this text.\n";
        std::fs::write(&path, original).unwrap();
        let host: SharedHost = tty7_core::host::local::LocalHost::new();
        let (app, mut vcx) = markdown_window(cx);
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_open_on_host(host, &path, window, cx)
        });
        wait_for_markdown(&mut vcx, |cx| {
            app.read_with(cx, |app, _| {
                app.tab_code().is_some_and(|code| !code.files.is_empty())
            })
        });
        let (input, reading) = app.update_in(&mut vcx, |app, window, cx| {
            let file = app.tab_code().unwrap().active_file().unwrap();
            assert!(file.preview);
            assert!(!file.dirty);
            assert!(
                file.reading
                    .as_ref()
                    .unwrap()
                    .focus_handle(cx)
                    .is_focused(window)
            );
            assert!(!file.input.focus_handle(cx).is_focused(window));
            (file.input.clone(), file.reading.clone().unwrap())
        });
        vcx.simulate_keystrokes("x backspace secondary-v secondary-z");
        assert_eq!(
            input.read_with(&vcx, |input, _| input.text().to_string()),
            original
        );
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_toggle_preview(window, cx)
        });
        vcx.simulate_keystrokes("x");
        let edited = input.read_with(&vcx, |input, _| input.text().to_string());
        assert_ne!(edited, original);
        let cursor = input.read_with(&vcx, |input, _| input.cursor_position());
        for _ in 0..2 {
            vcx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            let control = vcx
                .debug_bounds("document-fill-toggle")
                .expect("the editor offers fill and restore");
            vcx.simulate_click(control.center(), gpui::Modifiers::none());
            vcx.run_until_parked();
            app.read_with(&vcx, |app, _| {
                let file = app.tab_code().unwrap().active_file().unwrap();
                assert_eq!(file.input.entity_id(), input.entity_id());
                assert!(file.dirty, "layout switches keep unsaved edits");
            });
            assert_eq!(
                input.read_with(&vcx, |input, _| input.text().to_string()),
                edited
            );
            assert_eq!(
                input.read_with(&vcx, |input, _| input.cursor_position()),
                cursor
            );
        }
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_toggle_preview(window, cx)
        });
        vcx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        reading.read_with(&vcx, |preview, cx| {
            assert_eq!(preview.text.read(cx).source().as_ref(), edited)
        });
        app.read_with(&vcx, |app, _| {
            assert!(app.tab_code().unwrap().active_file().unwrap().dirty)
        });
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            original,
            "mode switches must not save"
        );
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_toggle_preview(window, cx)
        });
        assert_eq!(
            input.read_with(&vcx, |input, _| input.cursor_position()),
            cursor
        );
        vcx.simulate_keystrokes("secondary-z");
        assert_eq!(
            input.read_with(&vcx, |input, _| input.text().to_string()),
            original,
            "undo must survive the preview round trip"
        );
        vcx.simulate_keystrokes("y");
        let saved = input.read_with(&vcx, |input, _| input.text().to_string());
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_toggle_preview(window, cx)
        });
        vcx.simulate_keystrokes("secondary-s");
        wait_for_markdown(&mut vcx, |cx| {
            app.read_with(cx, |app, _| {
                app.tab_code()
                    .unwrap()
                    .active_file()
                    .is_some_and(|file| !file.dirty && file.saving.is_none())
            })
        });
        assert_eq!(std::fs::read_to_string(&path).unwrap(), saved);
        app.update_in(&mut vcx, |app, window, cx| {
            assert!(app.editor_close_active_if_focused(window, cx));
            assert!(app.tab_code().unwrap().files.is_empty());
        });
    }

    #[gpui::test]
    fn markdown_reactivation_keeps_mode_but_explicit_line_navigation_opens_source(
        cx: &mut TestAppContext,
    ) {
        let (app, mut vcx) = markdown_window(cx);
        let host: SharedHost = tty7_core::host::local::LocalHost::new();
        let path = PathBuf::from("/markdown-tests/first.md");
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_install_file(
                host.clone(),
                path.clone(),
                "# First\n\nText\n".into(),
                None,
                window,
                cx,
            );
            app.editor_toggle_preview(window, cx);
            app.editor_install_file(
                host.clone(),
                "/markdown-tests/source.rs".into(),
                "fn main() {}".into(),
                None,
                window,
                cx,
            );
            assert!(!app.tab_code().unwrap().active_file().unwrap().preview);
            assert!(app.editor_activate_open(host.id(), &path, window, cx));
            assert!(!app.tab_code().unwrap().active_file().unwrap().preview);
            app.editor_toggle_preview(window, cx);
            app.open_file_in_editor_at(&path, Some(3), Some(2), window, cx);
            let file = app.tab_code().unwrap().active_file().unwrap();
            assert!(!file.preview);
            assert!(file.input.focus_handle(cx).is_focused(window));
            app.editor_close_file(0, window, cx);
            app.editor_install_file(host, path, "# Reopened".into(), None, window, cx);
            assert!(app.tab_code().unwrap().active_file().unwrap().preview);
        });
    }

    #[gpui::test]
    fn markdown_failed_anchor_does_not_override_later_source_navigation(cx: &mut TestAppContext) {
        let (app, mut vcx) = markdown_window(cx);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("guide.md");
        let host: SharedHost = tty7_core::host::local::LocalHost::new();
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_open_markdown_link(host, &path, Some("intro".into()), window, cx);
        });
        wait_for_markdown(&mut vcx, |cx| {
            cx.update(|window, cx| {
                !gpui_component::Root::read(window, cx)
                    .notification
                    .read(cx)
                    .notifications()
                    .is_empty()
            })
        });
        std::fs::write(&path, "# Intro\n\nTarget line\n").unwrap();
        app.update_in(&mut vcx, |app, window, cx| {
            app.open_file_in_editor_at(&path, Some(3), Some(2), window, cx);
        });
        wait_for_markdown(&mut vcx, |cx| {
            app.read_with(cx, |app, _| {
                app.tab_code().is_some_and(|code| !code.files.is_empty())
            })
        });
        app.update_in(&mut vcx, |app, window, cx| {
            let file = app.tab_code().unwrap().active_file().unwrap();
            assert!(!file.preview);
            assert!(file.input.focus_handle(cx).is_focused(window));
            assert_eq!(
                file.input.read(cx).cursor_position(),
                Position {
                    line: 2,
                    character: 1
                }
            );
        });
    }

    #[gpui::test]
    fn markdown_newer_source_request_supersedes_an_inflight_anchor(cx: &mut TestAppContext) {
        let (app, mut vcx) = markdown_window(cx);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("guide.md");
        std::fs::write(&path, "# Intro\n\nTarget line\n").unwrap();
        let host: SharedHost = tty7_core::host::local::LocalHost::new();
        // Both requests start before the UI executor can deliver either Host
        // result. The last click must choose source, regardless of read order.
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_open_markdown_link(host, &path, Some("intro".into()), window, cx);
            app.open_file_in_editor_at(&path, Some(3), Some(2), window, cx);
        });
        wait_for_markdown(&mut vcx, |cx| {
            app.read_with(cx, |app, _| {
                app.tab_code().is_some_and(|code| !code.files.is_empty())
            })
        });
        app.update_in(&mut vcx, |app, window, cx| {
            let code = app.tab_code().unwrap();
            assert_eq!(code.files.len(), 1);
            let file = code.active_file().unwrap();
            assert!(!file.preview);
            assert!(file.input.focus_handle(cx).is_focused(window));
            assert_eq!(
                file.input.read(cx).cursor_position(),
                Position {
                    line: 2,
                    character: 1
                }
            );
        });
    }

    #[gpui::test]
    fn markdown_platform_keys_scroll_without_changing_source(cx: &mut TestAppContext) {
        let (app, mut vcx) = markdown_window(cx);
        let content = "# Heading\n\nParagraph.\n\n".repeat(80);
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_install_file(
                tty7_core::host::local::LocalHost::new(),
                "/markdown-tests/keys.md".into(),
                content.clone(),
                None,
                window,
                cx,
            );
        });
        for _ in 0..3 {
            vcx.run_until_parked();
            vcx.update(|window, cx| {
                let _ = window.draw(cx);
            });
        }
        let reading = app.read_with(&vcx, |app, _| {
            app.tab_code()
                .unwrap()
                .active_file()
                .unwrap()
                .reading
                .clone()
                .unwrap()
        });
        let (end, home) = if cfg!(target_os = "macos") {
            ("cmd-down", "cmd-up")
        } else {
            ("ctrl-end", "ctrl-home")
        };
        vcx.simulate_keystrokes(end);
        reading.read_with(&vcx, |reading, _| {
            assert!(reading.scroll.max_offset().y > px(0.));
            assert_eq!(reading.scroll.offset().y, -reading.scroll.max_offset().y);
        });
        vcx.simulate_keystrokes(home);
        assert_eq!(
            reading.read_with(&vcx, |reading, _| reading.scroll.offset().y),
            px(0.)
        );
        vcx.simulate_keystrokes("pagedown");
        assert!(reading.read_with(&vcx, |reading, _| reading.scroll.offset().y) < px(0.));
        vcx.simulate_keystrokes("secondary-a secondary-c");
        reading.read_with(&vcx, |reading, cx| {
            assert!(reading.text.read(cx).selected_text().contains("Paragraph."));
        });
        app.read_with(&vcx, |app, cx| {
            let file = app.tab_code().unwrap().active_file().unwrap();
            assert!(!file.dirty);
            assert_eq!(file.input.read(cx).text().to_string(), content);
        });
    }

    #[gpui::test]
    fn markdown_theme_reload_preserves_document_selection_and_scroll(cx: &mut TestAppContext) {
        use crate::core::{config::Config, markdown_theme};
        let (app, mut vcx) = markdown_window(cx);
        let dir = tempfile::tempdir().unwrap();
        let theme_path = dir.path().join("study.yaml");
        let yaml = "schema_version: 2\nid: study\nname: Study\nlight: {link: '#123456'}\ndark: {link: '#654321'}\n";
        std::fs::write(&theme_path, yaml).unwrap();
        let host: SharedHost = tty7_core::host::local::LocalHost::new();
        let content = (0..80)
            .map(|n| format!("## Section {n}\n\nReading paragraph {n}.\n\n"))
            .collect::<String>();
        app.update_in(&mut vcx, |app, window, cx| {
            app.set_theme_follow_system(false, window, cx);
            app.set_preset("light", window, cx);
            app.set_markdown_theme("github", cx);
            crate::ui::markdown_preview::apply_snapshot(markdown_theme::scan(Some(dir.path())), cx);
            app.editor_install_file(
                host,
                "/markdown-tests/theme.md".into(),
                content.clone(),
                None,
                window,
                cx,
            );
        });
        vcx.run_until_parked();
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
        // The first frame measures the viewport; the next applies its compact
        // or wide layout. Establish selection only after that resize, as a
        // user does in an already visible document.
        for _ in 0..2 {
            vcx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            vcx.run_until_parked();
        }
        text.update(&mut vcx, |text, cx| text.select_all(cx));
        reading.update(&mut vcx, |reading, cx| {
            reading.scroll.set_offset(gpui::point(px(0.), px(-300.)));
            cx.notify();
        });
        vcx.run_until_parked();
        vcx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let selected = text.read_with(&vcx, |text, cx| {
            assert_eq!(
                cx.global::<gpui_component::Theme>().mode,
                gpui_component::ThemeMode::Light
            );
            let selected = text.selected_text();
            assert!(selected.contains("Section 79"));
            selected
        });
        let before = reading.read_with(&vcx, |reading, _| reading.scroll.offset());
        assert!(before.y < px(0.));
        for _ in 0..2 {
            let control = vcx
                .debug_bounds("document-fill-toggle")
                .expect("Markdown offers fill and restore");
            vcx.simulate_click(control.center(), gpui::Modifiers::none());
            vcx.run_until_parked();
            text.read_with(&vcx, |text, _| {
                assert_eq!(text.source().as_ref(), content);
                assert_eq!(text.selected_text(), selected);
            });
            app.read_with(&vcx, |app, _| {
                assert_eq!(
                    app.tab_code()
                        .unwrap()
                        .active_file()
                        .unwrap()
                        .reading
                        .as_ref()
                        .unwrap()
                        .entity_id(),
                    reading.entity_id()
                );
            });
        }
        assert_eq!(
            reading.read_with(&vcx, |reading, _| reading.scroll.offset()),
            before
        );
        app.update_in(&mut vcx, |app, window, cx| {
            app.set_markdown_theme("study", cx);
            app.set_preset("dark", window, cx);
        });
        vcx.run_until_parked();
        vcx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        assert_eq!(
            reading.read_with(&vcx, |reading, _| reading.scroll.offset()),
            before
        );
        text.read_with(&vcx, |text, cx| {
            assert_eq!(
                cx.global::<gpui_component::Theme>().mode,
                gpui_component::ThemeMode::Dark
            );
            assert_eq!(text.source().as_ref(), content);
            assert_eq!(text.selected_text(), selected);
        });
        std::fs::write(&theme_path, "invalid: [").unwrap();
        vcx.update(|_, cx| {
            crate::ui::markdown_preview::apply_snapshot(markdown_theme::scan(Some(dir.path())), cx);
            assert_eq!(cx.global::<Config>().markdown_theme, "study");
            assert_eq!(crate::ui::markdown_preview::current(cx).theme.id, "study");
        });
        std::fs::write(&theme_path, yaml.replace("#654321", "#abcdef")).unwrap();
        vcx.update(|_, cx| {
            crate::ui::markdown_preview::apply_snapshot(markdown_theme::scan(Some(dir.path())), cx)
        });
        vcx.run_until_parked();
        vcx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        assert_eq!(
            reading.read_with(&vcx, |reading, _| reading.scroll.offset()),
            before
        );
        text.read_with(&vcx, |text, _| {
            assert_eq!(text.source().as_ref(), content);
            assert_eq!(text.selected_text(), selected);
        });
        app.read_with(&vcx, |app, _| {
            let file = app.tab_code().unwrap().active_file().unwrap();
            assert_eq!(
                file.reading.as_ref().unwrap().entity_id(),
                reading.entity_id()
            );
            assert!(file.preview);
            assert!(!file.dirty);
        });
    }

    #[gpui::test]
    fn markdown_document_links_scroll_to_the_requested_heading_and_keep_reading_position(
        cx: &mut TestAppContext,
    ) {
        let (app, mut vcx) = markdown_window(cx);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("guide.md");
        let content = (0..80)
            .map(|n| format!("## Section {n}\n\nReading paragraph {n}.\n\n"))
            .collect::<String>();
        std::fs::write(&path, &content).unwrap();
        let host: SharedHost = tty7_core::host::local::LocalHost::new();
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_open_markdown_link(
                host.clone(),
                &path,
                Some("section-30".into()),
                window,
                cx,
            )
        });
        wait_for_markdown(&mut vcx, |cx| {
            app.read_with(cx, |app, _| {
                app.tab_code().is_some_and(|code| !code.files.is_empty())
            })
        });
        for _ in 0..4 {
            vcx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            vcx.run_until_parked();
        }
        let reading = app.read_with(&vcx, |app, _| {
            app.tab_code()
                .unwrap()
                .active_file()
                .unwrap()
                .reading
                .clone()
                .unwrap()
        });
        reading.read_with(&vcx, |reading, cx| {
            let target = reading.text.read(cx).anchor_bounds("section-30").unwrap();
            assert!(
                (target.top() - reading.scroll.bounds().top() - px(12.)).abs() < px(1.),
                "anchor should be at the top of the reading viewport: {target:?}, viewport {:?}, offset {:?}", reading.scroll.bounds(), reading.scroll.offset()
            );
            assert!(reading.scroll.offset().y < px(-300.));
        });

        // A package that changes typography should retain the visible paragraph,
        // even though its absolute pixel offset changes after layout.
        let position = reading.read_with(&vcx, |reading, cx| {
            reading
                .text
                .read(cx)
                .reading_position(reading.scroll.bounds().top())
                .unwrap()
        });
        std::fs::write(dir.path().join("large.yaml"), "schema_version: 2\nid: large\nname: Large\ntypography: {font_size: 22, heading_sizes: [42, 36, 30, 27, 24, 22]}\nlight: {}\ndark: {}\n").unwrap();
        app.update_in(&mut vcx, |app, _, cx| {
            crate::ui::markdown_preview::apply_snapshot(
                crate::core::markdown_theme::scan(Some(dir.path())),
                cx,
            );
            app.set_markdown_theme("large", cx);
        });
        for _ in 0..4 {
            vcx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            vcx.run_until_parked();
        }
        let updated = reading.read_with(&vcx, |reading, cx| {
            let target = reading.text.read(cx).block_bounds(position.0).unwrap();
            reading.scroll.bounds().top() - target.top()
        });
        // The previous block can grow into the gap above this anchor after
        // reflow. Verify the saved content anchor rather than reclassifying
        // whichever block now touches the viewport's first pixel.
        assert!((updated - position.1).abs() < px(1.));

        // A fragment can also bring an already-open source editor back to its
        // existing preview, without creating a second source buffer.
        app.update_in(&mut vcx, |app, window, cx| {
            app.editor_toggle_preview(window, cx);
            app.editor_open_markdown_link(host, &path, Some("section-10".into()), window, cx);
        });
        wait_for_markdown(&mut vcx, |cx| {
            app.read_with(cx, |app, _| {
                app.tab_code().unwrap().active_file().unwrap().preview
            })
        });
        for _ in 0..4 {
            vcx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            vcx.run_until_parked();
        }
        app.read_with(&vcx, |app, _| {
            assert_eq!(app.tab_code().unwrap().files.len(), 1)
        });
        reading.read_with(&vcx, |reading, cx| {
            let target = reading.text.read(cx).anchor_bounds("section-10").unwrap();
            assert!((target.top() - reading.scroll.bounds().top() - px(12.)).abs() < px(1.));
        });
    }

    #[test]
    fn language_map_covers_common_extensions() {
        for (path, lang) in [
            ("a/b/main.rs", "rust"),
            ("x.tsx", "tsx"),
            ("x.jsx", "javascript"),
            ("x.yml", "yaml"),
            ("Makefile", "make"),
            ("CMakeLists.txt", "cmake"),
            (".zshrc", "bash"),
            ("notes.md", "markdown"),
            ("notes.MD", "markdown"),
            ("notes.markdown", "markdown"),
            ("notes.MARKDOWN", "markdown"),
            ("query.SQL", "sql"),
            ("unknown.xyz", "text"),
            ("no_ext", "text"),
        ] {
            assert_eq!(language_for_path(Path::new(path)), lang, "path {path}");
        }
    }

    #[test]
    fn binary_sniff_flags_nul_bytes_only() {
        assert!(looks_binary(b"\x7fELF\x00\x01"));
        assert!(!looks_binary("plain text\nwith lines".as_bytes()));
        assert!(!looks_binary("中文 UTF-8 内容".as_bytes()));
    }

    fn t(secs: i64, nanos: u32) -> Option<MTime> {
        Some(MTime { secs, nanos })
    }

    #[test]
    fn external_changes_are_told_apart_from_our_own_saves() {
        let ours = t(100, 0);

        assert_eq!(
            classify_external_change(false, false, ours, ours),
            ExternalChange::Ignore
        );

        assert_eq!(
            classify_external_change(false, false, ours, t(101, 0)),
            ExternalChange::Reload
        );

        assert_eq!(
            classify_external_change(false, true, ours, t(101, 0)),
            ExternalChange::Conflict
        );

        assert_eq!(
            classify_external_change(false, false, t(100, 0), t(100, 1)),
            ExternalChange::Reload
        );

        assert_eq!(
            classify_external_change(true, false, ours, t(101, 0)),
            ExternalChange::Ignore
        );

        assert_eq!(
            classify_external_change(false, false, None, None),
            ExternalChange::Reload
        );
    }

    #[test]
    fn a_landed_save_only_cleans_a_buffer_that_did_not_move() {
        assert_eq!(
            settle_save(true, 7, 7, false),
            SaveLanding {
                clean: true,
                requeue: false
            }
        );

        assert_eq!(
            settle_save(true, 7, 9, false),
            SaveLanding {
                clean: false,
                requeue: false
            }
        );

        assert_eq!(
            settle_save(true, 7, 9, true),
            SaveLanding {
                clean: false,
                requeue: true
            }
        );

        assert_eq!(
            settle_save(false, 7, 7, true),
            SaveLanding {
                clean: false,
                requeue: false
            }
        );
    }
}
