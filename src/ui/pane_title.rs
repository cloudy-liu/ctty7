//! The identity a recognized agent pane carries above its terminal, and the
//! pane actions its top edge offers on hover.
//!
//! Both are gated on the pane running a recognized agent (`TerminalView::agent`)
//! so an ordinary shell keeps exactly the presentation it has today. The title
//! row is a user preference, off by default; the toolbar is not: it is the
//! hover-revealed home for split, zoom and close on agent panes whether or not
//! the title is drawn.

use std::cell::Cell;
use std::rc::Rc;

use gpui::{AnyElement, App, EntityId, FocusHandle, div, prelude::*, px};
use gpui_component::ActiveTheme as _;
use gpui_component::button::Button;
use gpui_component::{Icon, IconName};

use crate::core::actions::{CloseActiveTab, SplitDown, SplitRight, ToggleMaximizePane};
use crate::core::cli_agent::CLIAgent;
use crate::ui::app::{TILE_GLYPH_XS, TILE_SIZE_XS};
use crate::ui::i18n::{L10nKey, t};
use crate::ui::pane::PaneSlot;

/// Height of the identity row: the pitch the other secondary bars share.
pub(crate) const TITLE_ROW: f32 = crate::ui::app::SECONDARY_BAR_HEIGHT;
/// The agent mark in that row.
pub(crate) const TITLE_ICON: f32 = 14.;
const TITLE_PAD_X: f32 = 10.;
const TITLE_GAP: f32 = 7.;
/// How far in from the pane's top-right corner the tiles sit. With the tile
/// box that keeps them inside the grip's reveal band
/// (`pane_drag::REVEAL_MIN`), so hovering a tile is still hovering the band
/// that revealed it.
const TOOLBAR_INSET: f32 = 2.;
const TOOLBAR_GAP: f32 = 2.;

/// What a pane draws beyond its terminal this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AgentPaneChrome {
    /// The icon-and-name row above the terminal.
    pub title: bool,
    /// The split / zoom / close tiles the top edge reveals on hover.
    pub toolbar: bool,
}

/// `agent` is what the pane is running right now; `titles_enabled` is the
/// user's `show_agent_pane_titles` setting. An ordinary terminal gets nothing
/// either way: the setting only decides whether an agent pane's identity is
/// spelled out, never whether a shell grows chrome.
pub(crate) fn chrome_for(agent: Option<CLIAgent>, titles_enabled: bool) -> AgentPaneChrome {
    AgentPaneChrome {
        title: agent.is_some() && titles_enabled,
        toolbar: agent.is_some(),
    }
}

/// The row above an agent pane's terminal: its mark and its name, nothing
/// else. Status stays in the sidebar, where it already is.
pub(crate) fn title_row(pane: EntityId, agent: CLIAgent, dim: f32, cx: &App) -> AnyElement {
    let ink = cx.global::<crate::ui::presets::AgentIcons>().ink(agent);
    div()
        .debug_selector(move || format!("agent-pane-title-{}", pane.as_u64()))
        .flex_shrink_0()
        .w_full()
        .h(px(TITLE_ROW))
        .flex()
        .items_center()
        .gap(px(TITLE_GAP))
        .px(px(TITLE_PAD_X))
        .when(dim < 1., |d| d.opacity(dim))
        .child(
            gpui::svg()
                .path(agent.icon_path())
                .flex_shrink_0()
                .size(px(TITLE_ICON))
                .text_color(gpui::rgb(ink)),
        )
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().foreground)
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
                .child(agent.display_name().to_string()),
        )
        .into_any_element()
}

/// Split, zoom and close for one agent pane, at its top-right corner, drawn
/// only while the pointer is on the pane's top edge. `hovered` is the cell the
/// drag grip reads, written by `pane_drag::reveal_band`, so one band serves
/// both. Focus does not reveal the tiles and a click does not pin them: the
/// band's leave hides them again.
pub(crate) fn toolbar(
    slot: &PaneSlot,
    zoomed: bool,
    hovered: &Rc<Cell<Option<EntityId>>>,
    cx: &App,
) -> AnyElement {
    let pane = slot.entity_id();
    if hovered.get() != Some(pane) {
        return div().into_any_element();
    }
    let focus = slot.focus_handle(cx);
    let (zoom_icon, zoom_tip) = if zoomed {
        (Icon::new(IconName::Minimize), L10nKey::PaneToolbarRestore)
    } else {
        (Icon::new(IconName::Maximize), L10nKey::PaneToolbarZoom)
    };
    let theme = cx.theme();
    div()
        .debug_selector(move || format!("agent-pane-toolbar-{}", pane.as_u64()))
        .occlude()
        .absolute()
        .top(px(TOOLBAR_INSET))
        .right(px(TOOLBAR_INSET))
        .flex()
        .items_center()
        .gap(px(TOOLBAR_GAP))
        .p(px(1.5))
        .rounded(px(6.))
        .bg(theme.sidebar)
        .border_1()
        .border_color(theme.border)
        .shadow_sm()
        .child(tile(
            "split-right",
            pane,
            Icon::default().path("icons/split-right.svg"),
            L10nKey::PaneToolbarSplitRight,
            focus.clone(),
            || Box::new(SplitRight),
            cx,
        ))
        .child(tile(
            "split-down",
            pane,
            Icon::default().path("icons/split-down.svg"),
            L10nKey::PaneToolbarSplitDown,
            focus.clone(),
            || Box::new(SplitDown),
            cx,
        ))
        .child(tile(
            "zoom",
            pane,
            zoom_icon,
            zoom_tip,
            focus.clone(),
            || Box::new(ToggleMaximizePane),
            cx,
        ))
        .child(tile(
            "close",
            pane,
            Icon::new(IconName::Close),
            L10nKey::PaneToolbarClose,
            focus,
            // What the palette's "close-pane" dispatches too: with the pane
            // focused it closes that pane, busy confirmation included.
            || Box::new(CloseActiveTab),
            cx,
        ))
        .into_any_element()
}

/// One tile. The existing operations act on the focused pane, so a press
/// first makes this pane that one: the tile belongs to it, not to whichever
/// pane the keyboard was in.
fn tile(
    name: &'static str,
    pane: EntityId,
    icon: Icon,
    tip: L10nKey,
    focus: FocusHandle,
    action: fn() -> Box<dyn gpui::Action>,
    cx: &App,
) -> AnyElement {
    div()
        .debug_selector(move || format!("agent-pane-{name}-{}", pane.as_u64()))
        .child(
            crate::ui::tab_strip::chrome_tile_sized(
                Button::new((name, pane.as_u64() as usize)).icon(icon),
                TILE_SIZE_XS,
                TILE_GLYPH_XS,
                false,
                cx,
            )
            .tooltip(t(tip))
            .on_click(move |_, window, cx| {
                window.focus(&focus, cx);
                window.dispatch_action(action(), cx);
            }),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_ordinary_terminal_gets_neither_title_nor_toolbar() {
        for titles_enabled in [false, true] {
            assert_eq!(
                chrome_for(None, titles_enabled),
                AgentPaneChrome {
                    title: false,
                    toolbar: false
                }
            );
        }
    }

    #[test]
    fn an_agent_pane_keeps_its_toolbar_whether_or_not_titles_are_on() {
        assert_eq!(
            chrome_for(Some(CLIAgent::Claude), false),
            AgentPaneChrome {
                title: false,
                toolbar: true
            }
        );
        assert_eq!(
            chrome_for(Some(CLIAgent::Codex), true),
            AgentPaneChrome {
                title: true,
                toolbar: true
            }
        );
    }
}

// A window with an agent pane beside a shell. Each test reads the screen the
// way a user would: what is drawn, where, after the pointer moves or a tile is
// pressed. Unix only, like every pane test: that is where quiet test panes run.
#[cfg(all(test, unix))]
mod gpui_tests {
    use gpui::{
        Axis, Entity, EntityId, Modifiers, TestAppContext, VisualTestContext, point, px, size,
    };

    use crate::core::cli_agent::CLIAgent;
    use crate::core::config::Config;
    use crate::daemon::protocol::DaemonMsg;
    use crate::terminal::view::quiet_test_pane;
    use crate::ui::app::{Tab, Tty7App, test_window::harness};
    use crate::ui::pane::{Pane, PaneSlot};

    type Streams = Vec<std::os::unix::net::UnixStream>;

    fn split(
        cx: &mut TestAppContext,
    ) -> (Entity<Tty7App>, VisualTestContext, [EntityId; 2], Streams) {
        let (app, mut vcx) = harness(cx);
        vcx.update(|_, cx| {
            let mut cfg = cx.global::<Config>().clone();
            cfg.cursor_blink = false;
            cx.set_global(cfg);
        });
        vcx.simulate_resize(size(px(1100.), px(700.)));
        let (ids, streams) = app.update_in(&mut vcx, |app, window, cx| {
            let (agent, mut agent_stream) = quiet_test_pane(1, window, cx);
            let (shell, shell_stream) = quiet_test_pane(2, window, cx);
            DaemonMsg::Agent(Some(CLIAgent::Claude))
                .encode(&mut agent_stream)
                .unwrap();
            let ids = [agent.entity_id(), shell.entity_id()];
            app.tabs.push(Tab::new(Pane::split_node(
                Axis::Horizontal,
                0.5,
                Pane::leaf(PaneSlot::Ready(agent)),
                Pane::leaf(PaneSlot::Ready(shell)),
            )));
            app.active = 0;
            app.focus_active(window, cx);
            cx.notify();
            (ids, vec![agent_stream, shell_stream])
        });
        settle(&mut vcx);
        (app, vcx, ids, streams)
    }

    fn settle(vcx: &mut VisualTestContext) {
        for _ in 0..3 {
            vcx.run_until_parked();
            vcx.update(|window, cx| {
                let _ = window.draw(cx);
            });
        }
    }

    fn drawn(vcx: &mut VisualTestContext, what: &str, pane: EntityId) -> bool {
        vcx.debug_bounds(&format!("{what}-{}", pane.as_u64()))
            .is_some()
    }

    fn hover_top_edge(vcx: &mut VisualTestContext, pane: EntityId) {
        let bounds = vcx
            .debug_bounds(&format!("pane-leaf-{}", pane.as_u64()))
            .expect("pane drawn");
        let edge = bounds.origin + point(bounds.size.width / 2., px(4.));
        vcx.simulate_mouse_move(edge, None, Modifiers::none());
        settle(vcx);
    }

    #[gpui::test]
    fn the_top_edge_reveals_a_toolbar_on_the_agent_pane_and_never_on_the_shell(
        cx: &mut TestAppContext,
    ) {
        let (_app, mut vcx, [agent, shell], _streams) = split(cx);
        assert!(
            !drawn(&mut vcx, "agent-pane-toolbar", agent),
            "nothing before the pointer arrives"
        );
        assert!(
            !drawn(&mut vcx, "agent-pane-title", agent),
            "titles are off by default"
        );

        hover_top_edge(&mut vcx, agent);
        assert!(drawn(&mut vcx, "agent-pane-toolbar", agent));
        assert!(!drawn(&mut vcx, "agent-pane-toolbar", shell));

        hover_top_edge(&mut vcx, shell);
        assert!(
            !drawn(&mut vcx, "agent-pane-toolbar", shell),
            "a shell grows no chrome"
        );
        assert!(
            !drawn(&mut vcx, "agent-pane-toolbar", agent),
            "leaving the edge hides it"
        );
    }

    #[gpui::test]
    fn the_setting_adds_and_removes_the_title_row_for_the_agent_pane_only(cx: &mut TestAppContext) {
        let (app, mut vcx, [agent, shell], _streams) = split(cx);
        app.update_in(&mut vcx, |app, _, cx| {
            app.set_show_agent_pane_titles(true, cx)
        });
        settle(&mut vcx);
        assert!(drawn(&mut vcx, "agent-pane-title", agent));
        assert!(!drawn(&mut vcx, "agent-pane-title", shell));
        assert!(
            !drawn(&mut vcx, "agent-pane-toolbar", agent),
            "a title does not pin the toolbar"
        );

        app.update_in(&mut vcx, |app, _, cx| {
            app.set_show_agent_pane_titles(false, cx)
        });
        settle(&mut vcx);
        assert!(!drawn(&mut vcx, "agent-pane-title", agent));
    }

    #[gpui::test]
    fn a_tile_acts_on_its_own_pane_not_on_the_focused_one(cx: &mut TestAppContext) {
        let (app, mut vcx, [agent, shell], _streams) = split(cx);
        app.update_in(&mut vcx, |app, window, cx| {
            let shell = app.tabs[0].pane.leaves()[1].clone();
            window.focus(&shell.focus_handle(cx), cx);
        });
        settle(&mut vcx);

        hover_top_edge(&mut vcx, agent);
        let zoom = vcx
            .debug_bounds(&format!("agent-pane-zoom-{}", agent.as_u64()))
            .expect("zoom tile");
        vcx.simulate_click(zoom.center(), Modifiers::default());
        settle(&mut vcx);
        assert!(
            drawn(&mut vcx, "pane-leaf", agent),
            "the agent pane fills the tab"
        );
        assert!(
            !drawn(&mut vcx, "pane-leaf", shell),
            "the focused shell is the one that went away"
        );

        hover_top_edge(&mut vcx, agent);
        let restore = vcx
            .debug_bounds(&format!("agent-pane-zoom-{}", agent.as_u64()))
            .expect("restore tile");
        vcx.simulate_click(restore.center(), Modifiers::default());
        settle(&mut vcx);
        assert!(
            drawn(&mut vcx, "pane-leaf", shell),
            "restore brings the split back"
        );
    }

    #[gpui::test]
    fn an_agent_that_exits_takes_its_chrome_with_it(cx: &mut TestAppContext) {
        let (app, mut vcx, [agent, _shell], mut streams) = split(cx);
        app.update_in(&mut vcx, |app, _, cx| {
            app.set_show_agent_pane_titles(true, cx)
        });
        settle(&mut vcx);
        assert!(drawn(&mut vcx, "agent-pane-title", agent));

        DaemonMsg::Agent(None).encode(&mut streams[0]).unwrap();
        settle(&mut vcx);
        hover_top_edge(&mut vcx, agent);
        assert!(!drawn(&mut vcx, "agent-pane-title", agent));
        assert!(!drawn(&mut vcx, "agent-pane-toolbar", agent));
    }
}
