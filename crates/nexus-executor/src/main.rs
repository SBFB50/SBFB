// SPDX-License-Identifier: AGPL-3.0-or-later

#[allow(dead_code)]
mod ipc;
mod task_runner;

use std::time::{Duration, Instant};

use clap::Parser;
use tokio::io::BufReader;
use tracing::{error, info, warn};

use crate::ipc::{
    read_message, write_message, HealthReportParams, JsonRpcNotification, JsonRpcRequest,
    JsonRpcResponse, ShutdownParams, TaskExecuteParams,
};

#[derive(Parser)]
#[command(name = "nexus-executor", about = "SBFB isolated compute executor")]
struct Cli {
    #[arg(long)]
    ipc_path: String,
}

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    tracing_subscriber::fmt::init();
    info!(ipc_path = %cli.ipc_path, "executor starting");

    let (read_half, write_half) = connect_ipc(&cli.ipc_path).await?;
    let mut reader = BufReader::new(read_half);
    let mut writer = tokio::io::BufWriter::new(write_half);

    let start = Instant::now();
    send_health(&mut writer, "starting", 0, 0, None, 0).await?;
    info!("connected to broker, sent initial health report");

    let mut heartbeat = tokio::time::Instant::now() + HEARTBEAT_INTERVAL;
    loop {
        tokio::select! {
            msg = read_message(&mut reader) => {
                match msg {
                    Ok(value) => {
                        if let Err(e) = handle(value, &mut writer, &start).await {
                            if e.to_string() == "shutdown" {
                                info!("graceful shutdown");
                                return Ok(());
                            }
                            error!(err = %e, "error handling message");
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                        warn!("broker disconnected");
                        return Ok(());
                    }
                    Err(e) => {
                        error!(err = %e, "IPC read error");
                        return Err(e);
                    }
                }
            }
            _ = tokio::time::sleep_until(heartbeat) => {
                send_health(&mut writer, "idle", 0, 0, None, start.elapsed().as_secs()).await?;
                heartbeat = tokio::time::Instant::now() + HEARTBEAT_INTERVAL;
            }
        }
    }
}

async fn handle<W: tokio::io::AsyncWrite + Unpin>(
    value: serde_json::Value,
    writer: &mut W,
    _start: &Instant,
) -> std::io::Result<()> {
    let method = value.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let id = value.get("id").and_then(|i| i.as_u64());

    match method {
        "task.execute" => {
            let id = id.ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "missing id")
            })?;
            let req: JsonRpcRequest = serde_json::from_value(value)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            if let Some(tp) = &req.traceparent {
                tracing::debug!(traceparent = %tp, "received traceparent");
            }
            let params: TaskExecuteParams = serde_json::from_value(req.params)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            let t0 = Instant::now();
            let mut result = task_runner::execute_task(&params);
            result.duration_ms = t0.elapsed().as_millis() as u64;

            let resp = JsonRpcResponse::success(
                id,
                serde_json::to_value(&result).expect("result serializes"),
            );
            write_message(writer, &resp).await?;
        }
        "executor.shutdown" => {
            let id = id.ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "missing id")
            })?;
            let params: ShutdownParams = value
                .get("params")
                .cloned()
                .map(serde_json::from_value)
                .transpose()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
                .unwrap_or(ShutdownParams {
                    reason: "unknown".into(),
                    grace_period_ms: 0,
                });
            info!(reason = %params.reason, "shutdown requested");

            let resp = JsonRpcResponse::success(id, serde_json::json!({"ack": true}));
            write_message(writer, &resp).await?;
            return Err(std::io::Error::other("shutdown"));
        }
        _ => {
            if let Some(id) = id {
                let resp =
                    JsonRpcResponse::error(id, -32601, format!("method not found: {method}"));
                write_message(writer, &resp).await?;
            }
        }
    }
    Ok(())
}

async fn send_health<W: tokio::io::AsyncWrite + Unpin>(
    writer: &mut W,
    status: &str,
    gpu_util_pct: u32,
    vram_used_mb: u32,
    model_loaded: Option<String>,
    uptime_s: u64,
) -> std::io::Result<()> {
    let notif = JsonRpcNotification::new(
        "health.report",
        serde_json::to_value(HealthReportParams {
            status: status.into(),
            gpu_util_pct,
            vram_used_mb,
            model_loaded,
            uptime_s,
        })
        .expect("health params serialize"),
    );
    write_message(writer, &notif).await
}

#[cfg(unix)]
async fn connect_ipc(
    path: &str,
) -> std::io::Result<(
    impl tokio::io::AsyncRead + Unpin,
    impl tokio::io::AsyncWrite + Unpin,
)> {
    let stream = tokio::net::UnixStream::connect(path).await?;
    Ok(tokio::io::split(stream))
}

#[cfg(windows)]
async fn connect_ipc(
    path: &str,
) -> std::io::Result<(
    impl tokio::io::AsyncRead + Unpin,
    impl tokio::io::AsyncWrite + Unpin,
)> {
    use tokio::net::windows::named_pipe::ClientOptions;

    let client = loop {
        match ClientOptions::new().open(path) {
            Ok(c) => break c,
            Err(e) if e.raw_os_error() == Some(231) => {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Err(e) => return Err(e),
        }
    };
    Ok(tokio::io::split(client))
}
