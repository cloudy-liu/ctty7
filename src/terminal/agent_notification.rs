//! Local delivery boundary for Agent events and the settings preview.
#[cfg(test)]
#[derive(Default)]
pub(crate) struct RecordingDelivery {
    pub sounds: usize,
    pub hold_sound: bool,
    pub defer_dispatch: Option<smol::channel::Receiver<()>>,
    pub desktops: Vec<(String, String, gpui::EntityId, bool)>,
}
#[cfg(test)]
impl gpui::Global for RecordingDelivery {}

#[cfg(test)]
impl RecordingDelivery {
    /// Complete the held playback at the native audio boundary.
    pub(crate) fn complete_sound(cx: &mut gpui::App) {
        cx.default_global::<Playback>()
            .0
            .store(false, std::sync::atomic::Ordering::Release);
    }
}

use crate::core::config::{Config, NotifyMode};
use gpui::{App, EntityId};

pub(crate) fn deliver(title: &str, body: &str, pane: EntityId, source_focused: bool, cx: &mut App) {
    // The source window is leased by its view update, so include its active
    // state directly. Other windows include Settings and auxiliary windows.
    let focused_at_event = source_focused || app_focused(cx);
    let cfg = cx.global::<Config>();
    let sound =
        cfg!(target_os = "windows") && allowed(cfg.agent_notification_sound, focused_at_event);
    let desktop = allowed(cfg.notify_on_command_finish, focused_at_event);
    if !sound && !desktop {
        return;
    }
    let title = title.to_owned();
    let body = body.to_owned();
    #[cfg(test)]
    let deferred = cx
        .default_global::<RecordingDelivery>()
        .defer_dispatch
        .clone();
    cx.spawn(async move |cx| {
        #[cfg(test)]
        if let Some(deferred) = deferred {
            let _ = deferred.recv().await;
        }
        let _ = cx.update(|cx| {
            let focused = app_focused(cx);
            let cfg = cx.global::<Config>();
            let sound = sound && allowed(cfg.agent_notification_sound, focused);
            let desktop = desktop && allowed(cfg.notify_on_command_finish, focused);
            if sound {
                play(cx);
            }
            if desktop {
                #[cfg(test)]
                {
                    cx.default_global::<RecordingDelivery>()
                        .desktops
                        .push((title, body, pane, true));
                }
                #[cfg(not(test))]
                super::remote::notify_agent_desktop_for_pane(&title, &body, pane);
            }
        });
    })
    .detach();
}

fn app_focused(cx: &mut App) -> bool {
    cx.windows().into_iter().any(|window| {
        window
            .update(cx, |_, window, _| window.is_window_active())
            .unwrap_or(false)
    })
}

fn allowed(mode: NotifyMode, app_focused: bool) -> bool {
    match mode {
        NotifyMode::Never => false,
        NotifyMode::Unfocused => !app_focused,
        NotifyMode::Always => true,
    }
}

pub(crate) fn preview(cx: &mut App) {
    play(cx);
}

#[derive(Default)]
struct Playback(std::sync::Arc<std::sync::atomic::AtomicBool>);
impl gpui::Global for Playback {}

fn play(cx: &mut App) {
    use std::sync::atomic::Ordering;
    let playing = cx.default_global::<Playback>().0.clone();
    if playing
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }
    #[cfg(test)]
    {
        let recording = cx.default_global::<RecordingDelivery>();
        recording.sounds += 1;
        if !recording.hold_sound {
            playing.store(false, Ordering::Release);
        }
        return;
    }
    #[cfg(all(target_os = "windows", not(test)))]
    play_native(playing);
    #[cfg(all(not(target_os = "windows"), not(test)))]
    playing.store(false, Ordering::Release);
}

#[cfg(all(target_os = "windows", not(test)))]
fn play_native(playing: std::sync::Arc<std::sync::atomic::AtomicBool>) {
    use std::sync::atomic::Ordering;
    use windows_sys::Win32::Media::Audio::{PlaySoundW, SND_MEMORY, SND_NODEFAULT, SND_SYNC};
    let worker_playing = playing.clone();
    // Play synchronously on one worker so overlapping requests are dropped,
    // and the embedded buffer remains alive until native playback completes.
    if let Err(error) = std::thread::Builder::new()
        .name("tty7-agent-sound".into())
        .spawn(move || {
            static WAV: &[u8] = include_bytes!("../../assets/agent-notification.wav");
            if unsafe {
                PlaySoundW(
                    WAV.as_ptr().cast(),
                    std::ptr::null_mut(),
                    SND_MEMORY | SND_NODEFAULT | SND_SYNC,
                )
            } == 0
            {
                log::warn!("failed to play Agent notification sound");
            }
            worker_playing.store(false, Ordering::Release);
        })
    {
        playing.store(false, Ordering::Release);
        log::warn!("failed to start Agent notification sound: {error}");
    }
}
