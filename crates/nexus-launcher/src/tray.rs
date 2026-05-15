// SPDX-License-Identifier: AGPL-3.0-or-later
use std::time::Duration;

use muda::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIconBuilder, TrayIconEvent};

pub struct TrayState {
    _tray: tray_icon::TrayIcon,
    open_id: MenuId,
    quit_id: MenuId,
}

pub fn create_tray() -> anyhow::Result<TrayState> {
    let png_bytes = include_bytes!("../../../assets/nexus-launcher.png");
    let decoder = png::Decoder::new(std::io::Cursor::new(png_bytes));
    let mut reader = decoder
        .read_info()
        .map_err(|e| anyhow::anyhow!("icon decode failed: {e}"))?;
    let buf_size = reader
        .output_buffer_size()
        .ok_or_else(|| anyhow::anyhow!("icon buffer size unknown"))?;
    let mut buf = vec![0u8; buf_size];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| anyhow::anyhow!("icon frame failed: {e}"))?;
    buf.truncate(info.buffer_size());
    let icon = tray_icon::Icon::from_rgba(buf, info.width, info.height)
        .map_err(|e| anyhow::anyhow!("icon creation failed: {e}"))?;

    let open_item = MenuItem::new("Open browser", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    let open_id = open_item.id().clone();
    let quit_id = quit_item.id().clone();

    let menu = Menu::new();
    menu.append(&open_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&quit_item)?;

    let tray = TrayIconBuilder::new()
        .with_icon(icon)
        .with_menu(Box::new(menu))
        .with_tooltip("SBFB Nexus Grid")
        .build()
        .map_err(|e| anyhow::anyhow!("tray build failed: {e}"))?;

    Ok(TrayState {
        _tray: tray,
        open_id,
        quit_id,
    })
}

pub fn run_event_loop(state: &TrayState, url: &str, ctrl_c: &std::sync::mpsc::Receiver<()>) {
    let menu_rx = MenuEvent::receiver();
    let tray_rx = TrayIconEvent::receiver();

    loop {
        #[cfg(windows)]
        pump_win32_messages();

        if let Ok(event) = menu_rx.try_recv() {
            if event.id == state.open_id {
                let _ = open::that(url);
            } else if event.id == state.quit_id {
                break;
            }
        }
        if let Ok(TrayIconEvent::DoubleClick { .. }) = tray_rx.try_recv() {
            let _ = open::that(url);
        }
        if ctrl_c.try_recv().is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Drain the Win32 message queue for the current thread so the
/// tray-icon hidden window's `tray_proc` receives WM_USER events.
/// tray-icon 0.24 does NOT create its own message pump — the caller
/// must pump messages on the thread that called `TrayIconBuilder::build`.
#[cfg(windows)]
fn pump_win32_messages() {
    #[repr(C)]
    struct Msg {
        hwnd: *mut core::ffi::c_void,
        message: u32,
        wparam: usize,
        lparam: isize,
        time: u32,
        pt_x: i32,
        pt_y: i32,
    }

    unsafe extern "system" {
        fn PeekMessageW(
            msg: *mut Msg,
            hwnd: *mut core::ffi::c_void,
            filter_min: u32,
            filter_max: u32,
            remove: u32,
        ) -> i32;
        fn TranslateMessage(msg: *const Msg) -> i32;
        fn DispatchMessageW(msg: *const Msg) -> isize;
    }

    const PM_REMOVE: u32 = 0x0001;

    // SAFETY: msg is zeroed, PeekMessageW fills it on success.
    // hwnd = null → drain all windows owned by this thread.
    unsafe {
        let mut msg: Msg = std::mem::zeroed();
        while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn embedded_icon_decodes_to_valid_rgba() {
        let png_bytes = include_bytes!("../../../assets/nexus-launcher.png");
        let decoder = png::Decoder::new(std::io::Cursor::new(png_bytes));
        let mut reader = decoder.read_info().expect("PNG decode");
        let buf_size = reader.output_buffer_size().expect("PNG buffer size");
        let mut buf = vec![0u8; buf_size];
        let info = reader.next_frame(&mut buf).expect("PNG frame");
        buf.truncate(info.buffer_size());
        assert!(info.width > 0 && info.height > 0);
        assert_eq!(buf.len() as u32, info.width * info.height * 4);
    }
}
