// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::thread;

use axum::extract::ws::{Message, WebSocket};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use tokio::sync::mpsc;

fn session_log_path(root: &Path) -> PathBuf {
    let ctx = crate::process::context_data(root);
    let sprint = ctx.get("sprint").and_then(|v| v.as_u64()).unwrap_or(0);
    let phase = ctx
        .get("phase")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let ts = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
        .replace(':', "-");

    let dir = root.join(".planning").join("terminal");
    let _ = fs::create_dir_all(&dir);
    dir.join(format!("sprint{sprint}_phase_{phase}_{ts}.cast"))
}

fn write_asciicast_header(file: &mut fs::File, cols: u16, rows: u16) {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let header = format!(
        r#"{{"version":2,"width":{cols},"height":{rows},"timestamp":{ts},"env":{{"TERM":"xterm-256color","SHELL":"claude"}}}}"#
    );
    let _ = writeln!(file, "{header}");
}

fn write_asciicast_event(file: &mut fs::File, start: std::time::Instant, data: &[u8]) {
    let elapsed = start.elapsed().as_secs_f64();
    let text = String::from_utf8_lossy(data);
    let escaped = serde_json::to_string(&*text).unwrap_or_default();
    let _ = writeln!(file, "[{elapsed:.6}, \"o\", {escaped}]");
}

pub async fn handle_terminal_ws(mut socket: WebSocket, cwd: &Path, resume_session: Option<&str>) {
    let pty_system = NativePtySystem::default();

    let cols: u16 = 120;
    let rows: u16 = 30;

    let pair = match pty_system.openpty(PtySize {
        rows,
        cols,
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
    if let Some(sid) = resume_session {
        cmd.arg("--resume");
        cmd.arg(sid);
    }

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

    let log_path = session_log_path(cwd);
    let log_path_display = log_path.display().to_string();

    let _ = socket
        .send(Message::Text(
            format!("\x1b[90m[session → {log_path_display}]\x1b[0m\r\n").into(),
        ))
        .await;

    let (pty_tx, mut pty_rx) = mpsc::channel::<Vec<u8>>(256);
    let (ws_tx, mut ws_rx) = mpsc::channel::<Vec<u8>>(256);

    let log_path_clone = log_path.clone();
    thread::spawn(move || {
        use std::io::Read;
        let mut reader = reader;
        let mut buf = [0u8; 4096];

        let mut log_file = fs::File::create(&log_path_clone).ok();
        if let Some(ref mut f) = log_file {
            write_asciicast_header(f, cols, rows);
        }
        let start = std::time::Instant::now();

        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = buf[..n].to_vec();
                    if let Some(ref mut f) = log_file {
                        write_asciicast_event(f, start, &chunk);
                    }
                    if pty_tx.blocking_send(chunk).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let master_for_resize = pair.master;
    thread::spawn(move || {
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

pub fn list_sessions(root: &Path) -> Vec<serde_json::Value> {
    let dir = root.join(".planning").join("terminal");
    let mut sessions = Vec::new();

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("cast") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                sessions.push(serde_json::json!({
                    "name": name,
                    "path": path.display().to_string(),
                    "size_bytes": size,
                }));
            }
        }
    }

    sessions.sort_by(|a, b| {
        let na = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let nb = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
        nb.cmp(na)
    });
    sessions
}

pub fn list_claude_sessions(root: &Path) -> Vec<serde_json::Value> {
    let home = dirs_next().unwrap_or_default();
    let dir = home.join(".claude").join("sessions");
    let root_str = dunce::canonicalize(root)
        .unwrap_or_else(|_| root.to_path_buf())
        .display()
        .to_string();

    let mut sessions = Vec::new();

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) else {
                continue;
            };

            let cwd = v.get("cwd").and_then(|c| c.as_str()).unwrap_or("");
            let canon_cwd = dunce::canonicalize(cwd)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| cwd.to_string());

            if canon_cwd != root_str {
                continue;
            }

            let session_id = v
                .get("sessionId")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            let name = v
                .get("name")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            let updated = v.get("updatedAt").and_then(|u| u.as_u64()).unwrap_or(0);

            sessions.push(serde_json::json!({
                "session_id": session_id,
                "name": name,
                "updated_at": updated,
            }));
        }
    }

    sessions.sort_by(|a, b| {
        let ua = a.get("updated_at").and_then(|v| v.as_u64()).unwrap_or(0);
        let ub = b.get("updated_at").and_then(|v| v.as_u64()).unwrap_or(0);
        ub.cmp(&ua)
    });
    sessions
}

fn dirs_next() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var("USERPROFILE").ok().map(PathBuf::from)
    }
    #[cfg(not(windows))]
    {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Sprint 71 Phase D / G6: the off-sprint terminal-recording code shipped
    // with zero tests. These exercise the file-system + asciicast surface
    // WITHOUT spawning a real PTY (the `handle_terminal_ws` spawn path drives
    // a live `claude` process and is not hermetically testable — mirrors the
    // OSS PTY-test rule of never launching the real interactive program).

    #[test]
    fn session_log_roundtrip() {
        // The log writers must emit a valid asciicast v2 stream:
        // line 1 = a JSON header object with "version":2; subsequent lines =
        // 3-element `[time, "o", data]` event arrays (docs.asciinema.org).
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("session.cast");

        let mut file = fs::File::create(&path).expect("create cast");
        write_asciicast_header(&mut file, 120, 30);
        let start = std::time::Instant::now();
        write_asciicast_event(&mut file, start, b"hello\r\n");
        drop(file);

        let content = fs::read_to_string(&path).expect("read cast");
        let mut lines = content.lines();

        let header: serde_json::Value =
            serde_json::from_str(lines.next().expect("header line")).expect("header is json");
        assert_eq!(header["version"], 2, "asciicast v2 header");
        assert_eq!(header["width"], 120);
        assert_eq!(header["height"], 30);

        let event: serde_json::Value =
            serde_json::from_str(lines.next().expect("event line")).expect("event is json");
        let arr = event.as_array().expect("event is an array");
        assert_eq!(arr.len(), 3, "event = [time, channel, data]");
        assert_eq!(arr[1], "o", "output channel");
        assert_eq!(arr[2], "hello\r\n", "payload round-trips verbatim");
    }

    #[test]
    fn list_sessions_filters_correct_extension() {
        // D7 (resolved Phase A) kept the asciicast `.cast` extension; the
        // listing must surface only `.cast` files, never `.log`/`.txt`.
        let root = tempfile::tempdir().expect("tempdir");
        let term_dir = root.path().join(".planning").join("terminal");
        fs::create_dir_all(&term_dir).expect("mkdir terminal");

        fs::write(term_dir.join("sprint71_phase_D_2026.cast"), "x").expect("write cast");
        fs::write(term_dir.join("stray.log"), "y").expect("write log");
        fs::write(term_dir.join("notes.txt"), "z").expect("write txt");

        let sessions = list_sessions(root.path());
        assert_eq!(sessions.len(), 1, "only the .cast file is a session");
        assert_eq!(sessions[0]["name"], "sprint71_phase_D_2026");
    }
}
