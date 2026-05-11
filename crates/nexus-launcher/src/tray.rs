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
    let img = image::load_from_memory(png_bytes)
        .map_err(|e| anyhow::anyhow!("icon decode failed: {e}"))?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let icon = tray_icon::Icon::from_rgba(rgba.into_raw(), w, h)
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

#[cfg(test)]
mod tests {
    #[test]
    fn embedded_icon_decodes_to_valid_rgba() {
        let png_bytes = include_bytes!("../../../assets/nexus-launcher.png");
        let img = image::load_from_memory(png_bytes).expect("PNG decode");
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        assert!(w > 0 && h > 0);
        assert_eq!(rgba.len() as u32, w * h * 4);
    }
}
