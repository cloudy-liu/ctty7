//! Windows owns overlay placement and scaling; the app supplies one unread mark.
use super::{TraySnapshot, icon};
use anyhow::Context as _;
use gpui::App;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::{
    Win32::{
        Foundation::HWND,
        System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance},
        UI::{
            Shell::{ITaskbarList3, TaskbarList},
            WindowsAndMessaging::{CreateIconFromResourceEx, DestroyIcon, HICON, LR_DEFAULTCOLOR},
        },
    },
    core::{HSTRING, PCWSTR},
};

pub(super) struct Taskbar {
    list: ITaskbarList3,
    icon: HICON,
}

impl Taskbar {
    fn new() -> anyhow::Result<Self> {
        // GPUI initializes COM on its Windows UI thread.
        let list: ITaskbarList3 =
            unsafe { CoCreateInstance(&TaskbarList, None, CLSCTX_INPROC_SERVER)? };
        unsafe { list.HrInit()? };
        let image = icon::attention_mark(32).context("could not render attention mark")?;
        let mut png = std::io::Cursor::new(Vec::new());
        image::RgbaImage::from_raw(image.width, image.height, image.data)
            .context("invalid attention mark")?
            .write_to(&mut png, image::ImageFormat::Png)?;
        // Windows accepts PNG-compressed icon resources. The input buffer only
        // needs to live for this call; the returned HICON owns its pixels.
        let icon = unsafe {
            CreateIconFromResourceEx(png.get_ref(), true, 0x30000, 32, 32, LR_DEFAULTCOLOR)?
        };
        Ok(Self { list, icon })
    }
}

impl Drop for Taskbar {
    fn drop(&mut self) {
        // SetOverlayIcon retains a shell-owned copy, never this handle.
        let _ = unsafe { DestroyIcon(self.icon) };
    }
}

pub(super) fn update(state: &mut Option<Taskbar>, snap: &TraySnapshot, cx: &mut App) {
    if state.is_none() {
        match Taskbar::new() {
            Ok(taskbar) => *state = Some(taskbar),
            Err(error) => {
                log::debug!("taskbar attention unavailable: {error:#}");
                return;
            }
        }
    }
    let taskbar = state.as_ref().unwrap();
    let description = HSTRING::from(snap.tooltip());
    let mark = if snap.attention() {
        taskbar.icon
    } else {
        HICON::default()
    };
    for (workspace, _) in crate::ui::windows::WindowRegistry::open_windows(cx) {
        let Some(handle) = crate::ui::windows::WindowRegistry::window_for(cx, workspace) else {
            continue;
        };
        let _ = handle.update(cx, |_, window, _| {
            let Ok(handle) = HasWindowHandle::window_handle(window) else {
                return;
            };
            let RawWindowHandle::Win32(handle) = handle.as_raw() else {
                return;
            };
            // Set the same aggregate state on every window in the taskbar group.
            // Reapplying during the existing tray poll also restores it after
            // Explorer recreates its taskbar buttons.
            if let Err(error) = unsafe {
                taskbar.list.SetOverlayIcon(
                    HWND(handle.hwnd.get() as *mut _),
                    mark,
                    PCWSTR(description.as_ptr()),
                )
            } {
                log::debug!("could not update taskbar attention: {error}");
            }
        });
    }
}
