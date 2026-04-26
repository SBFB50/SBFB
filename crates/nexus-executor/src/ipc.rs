// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

// ======================================================================
// JSON-RPC 2.0 message types
// ======================================================================

pub const JSONRPC_VERSION: &str = "2.0";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "_traceparent")]
    pub traceparent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

impl JsonRpcRequest {
    pub fn new(method: &str, params: serde_json::Value, id: u64) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: method.to_string(),
            params,
            id,
            traceparent: None,
        }
    }

    pub fn with_traceparent(mut self, tp: String) -> Self {
        self.traceparent = Some(tp);
        self
    }
}

impl JsonRpcResponse {
    pub fn success(id: u64, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error(id: u64, code: i32, message: String) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            result: None,
            error: Some(JsonRpcError { code, message }),
            id,
        }
    }
}

impl JsonRpcNotification {
    pub fn new(method: &str, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: method.to_string(),
            params,
        }
    }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_loaded: Option<String>,
    pub uptime_s: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShutdownParams {
    pub reason: String,
    pub grace_period_ms: u64,
}

// ======================================================================
// W3C Trace Context
// ======================================================================

pub fn extract_traceparent(request: &JsonRpcRequest) -> Option<&str> {
    request.traceparent.as_deref()
}

// ======================================================================
// Transport — newline-delimited JSON over async byte stream
// ======================================================================

pub async fn read_message<R: AsyncBufRead + Unpin>(
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

pub async fn write_message<W: AsyncWrite + Unpin>(
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

    #[test]
    fn test_json_rpc_request_roundtrip() {
        let params = TaskExecuteParams {
            task_id: "abc-123".into(),
            model: "llama3.1:8b".into(),
            prompt: "Hello".into(),
            watermark_config: Some(WatermarkConfig {
                enabled: true,
                delta: 2.0,
                window_size: 4,
            }),
            grammar: None,
            max_tokens: 1024,
            task_token: "deadbeef".into(),
        };
        let req = JsonRpcRequest::new("task.execute", serde_json::to_value(&params).unwrap(), 1)
            .with_traceparent("00-abc123-def456-01".into());

        let json = serde_json::to_string(&req).unwrap();
        let parsed: JsonRpcRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req, parsed);
        assert_eq!(parsed.jsonrpc, "2.0");
        assert_eq!(parsed.method, "task.execute");
        assert_eq!(extract_traceparent(&parsed), Some("00-abc123-def456-01"));
    }

    #[test]
    fn test_json_rpc_response_roundtrip() {
        let result = TaskExecuteResult {
            task_id: "abc-123".into(),
            output: "World".into(),
            output_token_ids: vec![42, 43],
            model_used: "llama3.1:8b".into(),
            duration_ms: 500,
            gpu_vram_peak_mb: 4200,
        };
        let resp = JsonRpcResponse::success(1, serde_json::to_value(&result).unwrap());
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: JsonRpcResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(resp, parsed);
        assert!(parsed.error.is_none());

        let err_resp = JsonRpcResponse::error(2, -32600, "invalid request".into());
        let json2 = serde_json::to_string(&err_resp).unwrap();
        let parsed2: JsonRpcResponse = serde_json::from_str(&json2).unwrap();
        assert_eq!(err_resp, parsed2);
        assert!(parsed2.result.is_none());
    }

    #[test]
    fn test_json_rpc_notification_roundtrip() {
        let health = HealthReportParams {
            status: "idle".into(),
            gpu_util_pct: 0,
            vram_used_mb: 1200,
            model_loaded: Some("llama3.1:8b".into()),
            uptime_s: 3600,
        };
        let notif =
            JsonRpcNotification::new("health.report", serde_json::to_value(&health).unwrap());
        let json = serde_json::to_string(&notif).unwrap();
        let parsed: JsonRpcNotification = serde_json::from_str(&json).unwrap();
        assert_eq!(notif, parsed);
        assert!(!json.contains("\"id\""));
    }

    #[test]
    fn test_traceparent_propagation() {
        let req = JsonRpcRequest::new("task.execute", serde_json::json!({}), 1)
            .with_traceparent("00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01".into());
        assert_eq!(
            extract_traceparent(&req),
            Some("00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
        );
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("_traceparent"));

        let req_none = JsonRpcRequest::new("task.execute", serde_json::json!({}), 2);
        assert_eq!(extract_traceparent(&req_none), None);
        let json2 = serde_json::to_string(&req_none).unwrap();
        assert!(!json2.contains("_traceparent"));
    }

    #[tokio::test]
    async fn test_transport_roundtrip() {
        let (client, server) = tokio::io::duplex(4096);
        let mut writer = tokio::io::BufWriter::new(client);
        let mut reader = tokio::io::BufReader::new(server);

        let req = JsonRpcRequest::new("test.method", serde_json::json!({"key": "value"}), 42);
        write_message(&mut writer, &req).await.unwrap();
        drop(writer);

        let msg = read_message(&mut reader).await.unwrap();
        let parsed: JsonRpcRequest = serde_json::from_value(msg).unwrap();
        assert_eq!(parsed.method, "test.method");
        assert_eq!(parsed.id, 42);
    }
}
