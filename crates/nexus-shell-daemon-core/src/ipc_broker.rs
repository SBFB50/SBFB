// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

use nexus_events_core::SecurityEvent;

// ======================================================================
// JSON-RPC 2.0 message types (mirrored from nexus-executor/src/ipc.rs —
// the contract is the JSON wire format, not the Rust type)
// ======================================================================

const JSONRPC_VERSION: &str = "2.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "_traceparent")]
    pub traceparent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<JsonRpcError>,
    pub id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

// ======================================================================
// Domain types
// ======================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskExecuteParams {
    pub task_id: String,
    pub model: String,
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub watermark_config: Option<WatermarkConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grammar: Option<String>,
    pub max_tokens: u32,
    pub task_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WatermarkConfig {
    pub enabled: bool,
    pub delta: f64,
    pub window_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskExecuteResult {
    pub task_id: String,
    pub output: String,
    pub output_token_ids: Vec<u32>,
    pub model_used: String,
    pub duration_ms: u64,
    pub gpu_vram_peak_mb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthReportParams {
    pub status: String,
    pub gpu_util_pct: u32,
    pub vram_used_mb: u32,
    #[serde(default)]
    pub model_loaded: Option<String>,
    pub uptime_s: u64,
}

// ======================================================================
// Task token — HMAC-SHA256(master_token, task_id || timestamp)
// ======================================================================

pub fn generate_task_token(master_token: &[u8], task_id: &str, timestamp: u64) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(master_token).expect("HMAC accepts any key length");
    mac.update(task_id.as_bytes());
    mac.update(&timestamp.to_le_bytes());
    hex::encode(mac.finalize().into_bytes())
}

// ======================================================================
// Backoff state — exponential 1s → 30s cap
// ======================================================================

const MIN_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_secs(30);

#[derive(Debug)]
pub struct BackoffState {
    restart_count: u32,
    current_delay: Duration,
}

impl BackoffState {
    pub fn new() -> Self {
        Self {
            restart_count: 0,
            current_delay: MIN_BACKOFF,
        }
    }

    pub fn record_crash(&mut self) {
        self.restart_count += 1;
        self.current_delay = (self.current_delay * 2).min(MAX_BACKOFF);
    }

    pub fn reset(&mut self) {
        self.restart_count = 0;
        self.current_delay = MIN_BACKOFF;
    }

    pub fn delay(&self) -> Duration {
        self.current_delay
    }

    pub fn restart_count(&self) -> u32 {
        self.restart_count
    }
}

impl Default for BackoffState {
    fn default() -> Self {
        Self::new()
    }
}

// ======================================================================
// IpcBroker — orchestrates executor subprocess lifecycle + IPC
// ======================================================================

pub struct IpcBroker {
    socket_path: String,
    executor_path: PathBuf,
    master_token: Vec<u8>,
    backoff: BackoffState,
    next_id: u64,
}

impl IpcBroker {
    pub fn new(socket_path: String, executor_path: PathBuf, master_token: Vec<u8>) -> Self {
        Self {
            socket_path,
            executor_path,
            master_token,
            backoff: BackoffState::new(),
            next_id: 1,
        }
    }

    fn next_request_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub async fn spawn_executor(&self) -> std::io::Result<tokio::process::Child> {
        tokio::process::Command::new(&self.executor_path)
            .arg("--ipc-path")
            .arg(&self.socket_path)
            .kill_on_drop(true)
            .spawn()
    }

    pub fn generate_token_for_task(&self, task_id: &str) -> String {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        generate_task_token(&self.master_token, task_id, ts)
    }

    pub async fn send_task<W: AsyncWrite + Unpin, R: AsyncBufRead + Unpin>(
        &mut self,
        writer: &mut W,
        reader: &mut R,
        params: TaskExecuteParams,
        traceparent: Option<String>,
    ) -> std::io::Result<TaskExecuteResult> {
        let id = self.next_request_id();
        let req = JsonRpcRequest {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: "task.execute".to_string(),
            params: serde_json::to_value(&params)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?,
            id,
            traceparent,
        };
        write_message(writer, &req).await?;

        let resp_val = read_message(reader).await?;
        let resp: JsonRpcResponse = serde_json::from_value(resp_val)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        if let Some(err) = resp.error {
            return Err(std::io::Error::other(format!(
                "executor error {}: {}",
                err.code, err.message
            )));
        }

        serde_json::from_value(resp.result.ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "missing result")
        })?)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub async fn send_shutdown<W: AsyncWrite + Unpin, R: AsyncBufRead + Unpin>(
        &mut self,
        writer: &mut W,
        reader: &mut R,
        reason: &str,
        grace_period_ms: u64,
    ) -> std::io::Result<()> {
        let id = self.next_request_id();
        let req = JsonRpcRequest {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: "executor.shutdown".to_string(),
            params: serde_json::json!({
                "reason": reason,
                "grace_period_ms": grace_period_ms,
            }),
            id,
            traceparent: None,
        };
        write_message(writer, &req).await?;

        let _resp_val = read_message(reader).await?;
        Ok(())
    }

    pub fn record_crash(&mut self, pid: u32, exit_code: Option<i32>) -> SecurityEvent {
        self.backoff.record_crash();
        SecurityEvent::ExecutorCrash {
            pid,
            exit_code,
            restart_count: self.backoff.restart_count(),
        }
    }

    pub fn backoff(&self) -> &BackoffState {
        &self.backoff
    }

    pub fn reset_backoff(&mut self) {
        self.backoff.reset();
    }
}

// ======================================================================
// Transport helpers
// ======================================================================

async fn read_message<R: AsyncBufRead + Unpin>(
    reader: &mut R,
) -> std::io::Result<serde_json::Value> {
    let mut line = String::new();
    let n = reader.read_line(&mut line).await?;
    if n == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "connection closed",
        ));
    }
    serde_json::from_str(line.trim())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

async fn write_message<W: AsyncWrite + Unpin>(
    writer: &mut W,
    msg: &impl Serialize,
) -> std::io::Result<()> {
    let mut buf = serde_json::to_vec(msg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    buf.push(b'\n');
    writer.write_all(&buf).await?;
    writer.flush().await
}

// ======================================================================
// Tests
// ======================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::BufReader;

    fn test_master_token() -> Vec<u8> {
        b"test-master-token-32bytes-long!!".to_vec()
    }

    #[test]
    fn test_task_token_ephemeral() {
        let master = test_master_token();
        let t1 = generate_task_token(&master, "task-1", 1000);
        let t2 = generate_task_token(&master, "task-2", 1000);
        let t3 = generate_task_token(&master, "task-1", 1001);

        assert_ne!(t1, t2, "different task_id must produce different token");
        assert_ne!(t1, t3, "different timestamp must produce different token");
        assert_eq!(
            t1,
            generate_task_token(&master, "task-1", 1000),
            "same inputs must be deterministic"
        );
        assert_eq!(t1.len(), 64, "HMAC-SHA256 = 32 bytes = 64 hex chars");
    }

    #[test]
    fn test_executor_crash_backoff() {
        let mut state = BackoffState::new();
        assert_eq!(state.delay(), Duration::from_secs(1));
        assert_eq!(state.restart_count(), 0);

        state.record_crash();
        assert_eq!(state.delay(), Duration::from_secs(2));

        state.record_crash();
        assert_eq!(state.delay(), Duration::from_secs(4));

        state.record_crash();
        assert_eq!(state.delay(), Duration::from_secs(8));

        state.record_crash();
        assert_eq!(state.delay(), Duration::from_secs(16));

        state.record_crash();
        assert_eq!(state.delay(), Duration::from_secs(30));

        state.record_crash();
        assert_eq!(state.delay(), Duration::from_secs(30));
        assert_eq!(state.restart_count(), 6);

        state.reset();
        assert_eq!(state.delay(), Duration::from_secs(1));
        assert_eq!(state.restart_count(), 0);
    }

    #[tokio::test]
    async fn test_executor_spawn_and_connect() {
        let (broker_end, executor_end) = tokio::io::duplex(8192);
        let (exec_read, exec_write) = tokio::io::split(executor_end);
        let (broker_read, _broker_write) = tokio::io::split(broker_end);
        let mut broker_reader = BufReader::new(broker_read);
        let mut exec_writer = tokio::io::BufWriter::new(exec_write);

        let health = JsonRpcNotification {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: "health.report".to_string(),
            params: serde_json::to_value(HealthReportParams {
                status: "starting".into(),
                gpu_util_pct: 0,
                vram_used_mb: 0,
                model_loaded: None,
                uptime_s: 0,
            })
            .unwrap(),
        };
        write_message(&mut exec_writer, &health).await.unwrap();
        drop(exec_writer);
        drop(exec_read);

        let msg = read_message(&mut broker_reader).await.unwrap();
        let notif: JsonRpcNotification = serde_json::from_value(msg).unwrap();
        assert_eq!(notif.method, "health.report");
        let params: HealthReportParams = serde_json::from_value(notif.params).unwrap();
        assert_eq!(params.status, "starting");
    }

    #[tokio::test]
    async fn test_task_execute_roundtrip() {
        let (broker_end, executor_end) = tokio::io::duplex(8192);
        let (exec_read, exec_write) = tokio::io::split(executor_end);
        let (broker_read, broker_write) = tokio::io::split(broker_end);

        let mut broker_reader = BufReader::new(broker_read);
        let mut broker_writer = tokio::io::BufWriter::new(broker_write);

        let executor_handle = tokio::spawn(async move {
            let mut er = BufReader::new(exec_read);
            let mut ew = tokio::io::BufWriter::new(exec_write);

            let msg = read_message(&mut er).await.unwrap();
            let req: JsonRpcRequest = serde_json::from_value(msg).unwrap();
            assert_eq!(req.method, "task.execute");
            assert!(
                req.traceparent.is_some(),
                "traceparent should be propagated"
            );

            let params: TaskExecuteParams = serde_json::from_value(req.params).unwrap();
            let result = TaskExecuteResult {
                task_id: params.task_id,
                output: "Hello world".into(),
                output_token_ids: vec![42, 43, 44],
                model_used: params.model,
                duration_ms: 150,
                gpu_vram_peak_mb: 4200,
            };
            let resp = JsonRpcResponse {
                jsonrpc: JSONRPC_VERSION.to_string(),
                result: Some(serde_json::to_value(&result).unwrap()),
                error: None,
                id: req.id,
            };
            write_message(&mut ew, &resp).await.unwrap();
        });

        let mut broker = IpcBroker::new("test".into(), PathBuf::from("test"), test_master_token());
        let params = TaskExecuteParams {
            task_id: "task-42".into(),
            model: "llama3.1:8b".into(),
            prompt: "Say hello".into(),
            watermark_config: None,
            grammar: None,
            max_tokens: 1024,
            task_token: broker.generate_token_for_task("task-42"),
        };
        let result = broker
            .send_task(
                &mut broker_writer,
                &mut broker_reader,
                params,
                Some("00-trace-span-01".into()),
            )
            .await
            .unwrap();

        assert_eq!(result.task_id, "task-42");
        assert_eq!(result.output, "Hello world");
        assert_eq!(result.output_token_ids, vec![42, 43, 44]);
        assert_eq!(result.duration_ms, 150);

        executor_handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_executor_crash_detection() {
        let (broker_end, executor_end) = tokio::io::duplex(8192);
        let (broker_read, _broker_write) = tokio::io::split(broker_end);
        let mut broker_reader = BufReader::new(broker_read);

        drop(executor_end);

        let result = read_message(&mut broker_reader).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            std::io::ErrorKind::UnexpectedEof
        );
    }

    #[tokio::test]
    async fn test_executor_shutdown_graceful() {
        let (broker_end, executor_end) = tokio::io::duplex(8192);
        let (exec_read, exec_write) = tokio::io::split(executor_end);
        let (broker_read, broker_write) = tokio::io::split(broker_end);

        let mut broker_reader = BufReader::new(broker_read);
        let mut broker_writer = tokio::io::BufWriter::new(broker_write);

        let executor_handle = tokio::spawn(async move {
            let mut er = BufReader::new(exec_read);
            let mut ew = tokio::io::BufWriter::new(exec_write);

            let msg = read_message(&mut er).await.unwrap();
            let req: JsonRpcRequest = serde_json::from_value(msg).unwrap();
            assert_eq!(req.method, "executor.shutdown");

            let resp = JsonRpcResponse {
                jsonrpc: JSONRPC_VERSION.to_string(),
                result: Some(serde_json::json!({"ack": true})),
                error: None,
                id: req.id,
            };
            write_message(&mut ew, &resp).await.unwrap();
        });

        let mut broker = IpcBroker::new("test".into(), PathBuf::from("test"), test_master_token());
        broker
            .send_shutdown(
                &mut broker_writer,
                &mut broker_reader,
                "idle_timeout",
                30000,
            )
            .await
            .unwrap();

        executor_handle.await.unwrap();
    }

    #[test]
    fn test_executor_crash_security_event() {
        let mut broker = IpcBroker::new("test".into(), PathBuf::from("test"), test_master_token());
        let event = broker.record_crash(12345, Some(137));
        match event {
            SecurityEvent::ExecutorCrash {
                pid,
                exit_code,
                restart_count,
            } => {
                assert_eq!(pid, 12345);
                assert_eq!(exit_code, Some(137));
                assert_eq!(restart_count, 1);
            }
            _ => panic!("expected ExecutorCrash"),
        }

        let event2 = broker.record_crash(12346, None);
        match event2 {
            SecurityEvent::ExecutorCrash { restart_count, .. } => {
                assert_eq!(restart_count, 2);
            }
            _ => panic!("expected ExecutorCrash"),
        }
    }
}
