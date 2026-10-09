//! Opt-in screenshots of the production tab strip and sidebar.
//! Deterministic daemon messages keep the base/head comparison identical.

use gpui::{prelude::*, *};
use gpui_component::Root;
use serde::Deserialize;
use std::{path::PathBuf, time::Duration};

use crate::{
    core::{
        cli_agent::{AgentSessionState, AgentStatus, CLIAgent},
        config::{Config, SidebarGrouping, TabBarPosition},
    },
    daemon::{
        protocol::{DaemonMsg, RemoteContext, RemoteKind},
        transport::Stream,
    },
    terminal::view::{TerminalView, quiet_test_pane},
    ui::{
        app::{Tab, Tty7App},
        pane::{Pane, PaneSlot},
    },
};

#[derive(Deserialize)]
struct Options {
    sidebar: bool,
    output: PathBuf,
    hold_seconds: Option<u64>,
}

struct Fixture {
    owner: Entity<Tty7App>,
    _streams: Vec<Stream>,
}

impl Render for Fixture {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.owner.clone()
    }
}

const CASES: [(&str, Option<CLIAgent>, Option<RemoteKind>); 6] = [
    ("Local terminal", None, None),
    ("Local Codex", Some(CLIAgent::Codex), None),
    ("Local Claude", Some(CLIAgent::Claude), None),
    ("prod-web", None, Some(RemoteKind::NativeSsh)),
    ("Fix login", Some(CLIAgent::Codex), Some(RemoteKind::Ssh)),
    (
        "Deploy check",
        Some(CLIAgent::Claude),
        Some(RemoteKind::NativeSsh),
    ),
];

pub(crate) fn run(path: &std::ffi::OsStr) {
    let options: Options = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    gpui_platform::application()
        .with_assets(crate::ui::assets::Assets)
        .with_quit_mode(QuitMode::Explicit)
        .run(move |cx| {
            gpui_component::init(cx);
            crate::register_bundled_fonts(cx);
            let mut config = Config::default();
            config.theme_follow_system = false;
            config.theme_preset = "light".into();
            config.install_cli_on_path = false;
            config.cursor_blink = false;
            config.sidebar_width = 260.;
            config.sidebar_grouping = SidebarGrouping::None;
            config.tab_bar_position = if options.sidebar {
                TabBarPosition::Left
            } else {
                TabBarPosition::Top
            };
            cx.set_global(config);
            crate::core::session::WorkspaceStore::init(cx);
            crate::ui::windows::WindowRegistry::init(cx);
            crate::ui::presets::load_registry(cx);
            crate::ui::keymap::init(cx);
            let mut views: Vec<Entity<TerminalView>> = Vec::new();
            let mut owner_entity = None;
            let handle = cx
                .open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                            None,
                            size(px(1200.), px(660.)),
                            cx,
                        ))),
                        titlebar: Some(TitlebarOptions {
                            title: Some("SSH tab visual acceptance".into()),
                            ..Default::default()
                        }),
                        show: true,
                        focus: true,
                        ..Default::default()
                    },
                    |window, cx| {
                        crate::ui::theme::apply_theme(Some(window), cx);
                        let owner = cx.new(|cx| {
                            Tty7App::with_session(
                                None,
                                Some(crate::core::session::Session::default()),
                                window,
                                cx,
                            )
                        });
                        let mut streams = Vec::new();
                        owner.update(cx, |app, cx| {
                            for (index, (name, agent, remote)) in CASES.iter().enumerate() {
                                let (view, mut stream) =
                                    quiet_test_pane(index as u64 + 1, window, cx);
                                if let Some(kind) = remote {
                                    DaemonMsg::RemoteContext(Some(RemoteContext {
                                        kind: *kind,
                                        argv: vec!["ssh".into(), "dev-box".into()],
                                        target: if index == 3 { "prod-web" } else { "dev-box" }
                                            .into(),
                                    }))
                                    .encode(&mut stream)
                                    .unwrap();
                                }
                                DaemonMsg::Agent(*agent).encode(&mut stream).unwrap();
                                if agent.is_some() {
                                    DaemonMsg::AgentStatus(Some(AgentSessionState {
                                        status: AgentStatus::Working,
                                        rich: true,
                                        ..Default::default()
                                    }))
                                    .encode(&mut stream)
                                    .unwrap();
                                }
                                DaemonMsg::Output(format!(
                                    "\x1b]0;{name}\x07\x1b[1mSSH tab visual acceptance\x1b[0m\r\n\r\n\
                                     Production native UI, identical fixture data.\r\n\
                                     No network SSH connection is opened by this fixture.\r\n\r\n\
                                     Local: terminal / Codex / Claude Code\r\n\
                                     SSH:   terminal / Codex / Claude Code\r\n\r\n\
                                     Active session: Fix login (SSH, detected Codex)\r\n\r\n\
                                     \x1b[36mdev@dev-box\x1b[0m:/srv/api $ codex\r\n"
                                ).into_bytes()).encode(&mut stream).unwrap();
                                app.tabs.push(Tab::new(Pane::leaf(PaneSlot::Ready(view.clone()))));
                                views.push(view);
                                streams.push(stream);
                            }
                            app.active = 4;
                            app.focus_active(window, cx);
                            cx.notify();
                        });
                        owner_entity = Some(owner.clone());
                        let fixture = cx.new(|_| Fixture { owner, _streams: streams });
                        cx.new(|cx| Root::new(fixture, window, cx))
                    },
                )
                .unwrap();
            cx.activate(true);
            let owner = owner_entity.unwrap();
            cx.spawn(async move |cx| {
                let deadline = std::time::Instant::now() + Duration::from_secs(20);
                loop {
                    cx.background_executor().timer(Duration::from_millis(100)).await;
                    let ready = views.iter().zip(CASES).all(|(view, (name, agent, remote))| {
                        view.read_with(cx, |view, _| {
                            view.title == name && view.agent() == agent
                                && view.remote_context().map(|r| r.kind) == remote
                        })
                    });
                    if ready { break; }
                    assert!(std::time::Instant::now() < deadline, "SSH tab fixture did not settle");
                }
                cx.update_window(handle.into(), |_, window, cx| {
                    owner.update(cx, |app, cx| app.focus_active(window, cx));
                    window.refresh();
                    let _ = window.draw(cx);
                }).unwrap();
                cx.background_executor().timer(Duration::from_secs(2)).await;
                cx.update_window(handle.into(), |_, window, cx| {
                    window.refresh();
                    let _ = window.draw(cx);
                    window.set_window_title("SSH tab visual acceptance");
                    #[cfg(target_os = "macos")]
                    window.render_to_image().unwrap().save(&options.output).unwrap();
                    #[cfg(not(target_os = "macos"))]
                    let _ = &options.output;
                }).unwrap();
                eprintln!("NATIVE_SSH_TABS_READY");
                cx.background_executor()
                    .timer(Duration::from_secs(options.hold_seconds.unwrap_or(12)))
                    .await;
                // The platform input handler owns the focused terminal. Release
                // it while the window can still draw, before GPUI checks leaks.
                cx.update_window(handle.into(), |_, window, cx| {
                    window.blur();
                    window.refresh();
                    let _ = window.draw(cx);
                    window.remove_window();
                }).unwrap();
                drop(views);
                drop(owner);
                cx.background_executor().timer(Duration::from_millis(100)).await;
                cx.update(|cx| cx.quit());
            }).detach();
        });
}
