// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;
use std::thread;

use axum::extract::ws::{Message, WebSocket};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use tokio::sync::mpsc;

pub async fn handle_terminal_ws(mut socket: WebSocket, cwd: &Path) {
    let pty_system = NativePtySystem::default();

    let pair = match pty_system.openpty(PtySize {
        rows: 30,
        cols: 120,
        pixel_width: 0,
        pixel_height: 0,
    }) {
        Ok(p) => p,
        Err(e) => {
            let _ = socket
                .send(Message::Text(format!("PTY error: {e}").into()))
                .await;
            return;
        }
    };

    let exe = if cfg!(windows) {
        "claude.cmd"
    } else {
        "claude"
    };
    let mut cmd = CommandBuilder::new(exe);
    cmd.cwd(cwd);

    let mut child = match pair.slave.spawn_command(cmd) {
        Ok(c) => c,
        Err(e) => {
            let _ = socket
                .send(Message::Text(format!("Spawn error: {e}").into()))
                .await;
            return;
        }
    };

    drop(pair.slave);

    let reader = match pair.master.try_clone_reader() {
        Ok(r) => r,
        Err(e) => {
            let _ = socket
                .send(Message::Text(format!("Reader error: {e}").into()))
                .await;
            return;
        }
    };

    let pty_writer = match pair.master.take_writer() {
        Ok(w) => w,
        Err(e) => {
            let _ = socket
                .send(Message::Text(format!("Writer error: {e}").into()))
                .await;
            return;
        }
    };

    let (pty_tx, mut pty_rx) = mpsc::channel::<Vec<u8>>(256);
    let (ws_tx, mut ws_rx) = mpsc::channel::<Vec<u8>>(256);

    thread::spawn(move || {
        use std::io::Read;
        let mut reader = reader;
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if pty_tx.blocking_send(buf[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let master_for_resize = pair.master;
    thread::spawn(move || {
        use std::io::Write;
        let mut writer = pty_writer;
        while let Some(data) = ws_rx.blocking_recv() {
            if data.starts_with(b"{\"type\":\"resize\"") {
                if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&data) {
                    let cols = v.get("cols").and_then(|v| v.as_u64()).unwrap_or(120) as u16;
                    let rows = v.get("rows").and_then(|v| v.as_u64()).unwrap_or(30) as u16;
                    let _ = master_for_resize.resize(PtySize {
                        rows,
                        cols,
                        pixel_width: 0,
                        pixel_height: 0,
                    });
                }
            } else {
                let _ = writer.write_all(&data);
            }
        }
    });

    loop {
        tokio::select! {
            Some(data) = pty_rx.recv() => {
                if socket.send(Message::Binary(data.into())).await.is_err() {
                    break;
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text)))
                        if ws_tx.send(text.as_bytes().to_vec()).await.is_err() =>
                    {
                        break;
                    }
                    Some(Ok(Message::Binary(data)))
                        if ws_tx.send(data.to_vec()).await.is_err() =>
                    {
                        break;
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
            else => break,
        }
    }

    let _ = child.kill();
    let _ = child.wait();
}
