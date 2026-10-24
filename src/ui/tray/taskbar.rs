//! Compose attention into the window icon so every taskbar keeps it upper-right.
use super::{TraySnapshot, icon};
use anyhow::Context as _;
use gpui::App;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::collections::HashMap;
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{
        CreateIconFromResourceEx, DestroyIcon, GCLP_HICON, GCLP_HICONSM, GetClassLongPtrW, HICON,
        ICON_BIG, ICON_SMALL, IsWindow, LR_DEFAULTCOLOR, SendMessageW, WM_GETICON, WM_SETICON,
    },
};

const KINDS: [u32; 2] = [ICON_SMALL, ICON_BIG];

struct Icon(HICON);

impl Icon {
    fn new(size: u32) -> anyhow::Result<Self> {
        let image = icon::render_sized(true, size).context("could not render attention icon")?;
        let mut png = std::io::Cursor::new(Vec::new());
        image::RgbaImage::from_raw(image.width, image.height, image.data)
            .context("invalid attention icon")?
            .write_to(&mut png, image::ImageFormat::Png)?;
        // Windows accepts PNG-compressed icon resources. The input buffer only
        // needs to live for this call; the returned HICON owns its pixels.
        let icon = unsafe {
            CreateIconFromResourceEx(
                png.get_ref(),
                true,
                0x30000,
                size as i32,
                size as i32,
                LR_DEFAULTCOLOR,
            )?
        };
        Ok(Self(icon))
    }
}

impl Drop for Icon {
    fn drop(&mut self) {
        let _ = unsafe { DestroyIcon(self.0) };
    }
}

pub(super) struct Taskbar {
    icon: Icon,
    originals: HashMap<isize, [HICON; 2]>,
}

impl Taskbar {
    fn new() -> anyhow::Result<Self> {
        Ok(Self {
            // Explorer may ignore a small bitmap for ICON_SMALL at high DPI.
            // Let Windows scale the full-resolution artwork for both slots.
            icon: Icon::new(256)?,
            originals: HashMap::new(),
        })
    }

    fn apply(&mut self, hwnd: HWND, attention: bool) {
        if attention {
            let current = KINDS.map(|kind| unsafe {
                let icon = SendMessageW(hwnd, WM_GETICON, WPARAM(kind as usize), LPARAM(0)).0;
                if icon != 0 {
                    return HICON(icon as *mut _);
                }
                // Null selects a class icon, but sending null later does not
                // refresh Explorer. Preserve the borrowed class handle itself.
                let index = if kind == ICON_SMALL {
                    GCLP_HICONSM
                } else {
                    GCLP_HICON
                };
                let icon = GetClassLongPtrW(hwnd, index);
                HICON(if icon == 0 {
                    GetClassLongPtrW(hwnd, GCLP_HICON)
                } else {
                    icon
                } as *mut _)
            });
            if current == [self.icon.0; 2] {
                return;
            }
            // Preserve GPUI's effective icons, including its class fallback.
            // Repeated polls must never save our own artwork as the original.
            let original = self.originals.entry(hwnd.0 as isize).or_insert(current);
            for (original, current) in original.iter_mut().zip(current) {
                if current != self.icon.0 {
                    *original = current;
                }
            }
            for kind in KINDS {
                unsafe {
                    SendMessageW(
                        hwnd,
                        WM_SETICON,
                        WPARAM(kind as usize),
                        LPARAM(self.icon.0.0 as isize),
                    );
                }
            }
        } else if let Some(original) = self.originals.remove(&(hwnd.0 as isize)) {
            self.restore(hwnd, original);
        }
    }

    fn restore(&self, hwnd: HWND, original: [HICON; 2]) {
        for (kind, original) in KINDS.into_iter().zip(original) {
            unsafe {
                // A closed/reused HWND or another icon owner must be left alone.
                if SendMessageW(hwnd, WM_GETICON, WPARAM(kind as usize), LPARAM(0)).0
                    == self.icon.0.0 as isize
                {
                    SendMessageW(
                        hwnd,
                        WM_SETICON,
                        WPARAM(kind as usize),
                        LPARAM(original.0 as isize),
                    );
                }
            }
        }
    }
}

impl Drop for Taskbar {
    fn drop(&mut self) {
        // WM_SETICON borrows the HICON. Detach it before freeing its pixels.
        for (&hwnd, &original) in &self.originals {
            self.restore(HWND(hwnd as *mut _), original);
        }
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
    let taskbar = state.as_mut().unwrap();
    taskbar
        .originals
        .retain(|&hwnd, _| unsafe { IsWindow(HWND(hwnd as *mut _)).as_bool() });
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
            // Every window in the group carries the same aggregate reminder.
            // Explorer can recreate its buttons directly from these icons.
            taskbar.apply(HWND(handle.hwnd.get() as *mut _), snap.attention());
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::{
        Win32::{
            Foundation::{LPARAM, WPARAM},
            UI::WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, GCLP_HICON, GCLP_HICONSM,
                GetClassLongPtrW, ICON_BIG, ICON_SMALL, IDI_APPLICATION, LoadIconW, RegisterClassW,
                SendMessageW, UnregisterClassW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_GETICON,
                WM_SETICON, WNDCLASSW,
            },
        },
        core::w,
    };

    struct TestWindow(HWND);

    impl TestWindow {
        fn new() -> Self {
            Self::for_class(w!("STATIC"))
        }

        fn for_class(class: windows::core::PCWSTR) -> Self {
            Self(unsafe {
                CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    class,
                    w!("attention test"),
                    WINDOW_STYLE::default(),
                    0,
                    0,
                    32,
                    32,
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap()
            })
        }

        fn icons(&self) -> [isize; 2] {
            [ICON_SMALL, ICON_BIG].map(|kind| unsafe {
                SendMessageW(self.0, WM_GETICON, WPARAM(kind as usize), LPARAM(0)).0
            })
        }
    }

    #[test]
    fn clearing_attention_supplies_the_class_icon_to_explorer() {
        unsafe extern "system" fn window_proc(
            hwnd: HWND,
            message: u32,
            wparam: WPARAM,
            lparam: LPARAM,
        ) -> windows::Win32::Foundation::LRESULT {
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
        let class = w!("ctty7-taskbar-class-icon-test");
        let icon = unsafe { LoadIconW(None, IDI_APPLICATION).unwrap() };
        assert_ne!(
            unsafe {
                RegisterClassW(&WNDCLASSW {
                    lpfnWndProc: Some(window_proc),
                    hIcon: icon,
                    lpszClassName: class,
                    ..Default::default()
                })
            },
            0
        );
        {
            let window = TestWindow::for_class(class);
            assert_eq!(
                window.icons(),
                [0, 0],
                "class icons are not window overrides"
            );
            let expected = [GCLP_HICONSM, GCLP_HICON]
                .map(|kind| unsafe { GetClassLongPtrW(window.0, kind) as isize });
            assert!(expected.iter().all(|icon| *icon != 0));
            let mut taskbar = Taskbar::new().unwrap();
            for _ in 0..3 {
                taskbar.apply(window.0, true);
                taskbar.apply(window.0, false);
                assert_eq!(
                    window.icons(),
                    expected,
                    "Explorer keeps the attention artwork when WM_SETICON restores a null handle"
                );
            }
            taskbar.apply(window.0, true);
            drop(taskbar);
            assert_eq!(
                window.icons(),
                expected,
                "the restored class icons remain borrowed"
            );
        }
        unsafe { UnregisterClassW(class, None).unwrap() };
    }

    impl Drop for TestWindow {
        fn drop(&mut self) {
            let _ = unsafe { DestroyWindow(self.0) };
        }
    }

    #[test]
    fn attention_uses_window_artwork_and_restores_icons() {
        let first = TestWindow::new();
        let second = TestWindow::new();
        let original = first.icons();
        let mut taskbar = Taskbar::new().unwrap();
        taskbar.apply(first.0, true);
        let marked = first.icons();
        assert!(
            marked.iter().all(|icon| *icon != 0),
            "the upper-right badge must be part of the window artwork, not a shell-positioned overlay"
        );
        assert_ne!(marked, original);
        taskbar.apply(first.0, true);
        assert_eq!(first.icons(), marked);
        taskbar.apply(second.0, true);
        assert_eq!(second.icons(), marked);
        taskbar.apply(first.0, false);
        assert_eq!(first.icons(), original);
        assert_eq!(second.icons(), marked);
        // Preserve a real per-window icon as well as the null class fallback.
        let custom = unsafe { LoadIconW(None, IDI_APPLICATION).unwrap() };
        unsafe {
            SendMessageW(
                first.0,
                WM_SETICON,
                WPARAM(ICON_BIG as usize),
                LPARAM(custom.0 as isize),
            );
        }
        let custom_original = first.icons();
        taskbar.apply(first.0, true);
        taskbar.apply(first.0, true);
        drop(taskbar);
        assert_eq!(first.icons(), custom_original);
        assert_eq!(second.icons(), original);
    }
}
