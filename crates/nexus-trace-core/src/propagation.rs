// SPDX-License-Identifier: AGPL-3.0-or-later
//! W3C Trace Context propagation helpers.
//!
//! Format: `00-<trace_id:32hex>-<span_id:16hex>-<flags:2hex>`
//!
//! Used by the broker/executor JSON-RPC IPC layer via the
//! `_traceparent` field (Phase C). This module handles generation,
//! parsing, and inject/extract for `HashMap<String, Value>`.

use std::collections::HashMap;

pub const TRACEPARENT_KEY: &str = "_traceparent";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
    pub flags: u8,
}

pub fn new_trace_id() -> String {
    let buf: [u8; 16] = rand::random();
    hex::encode(buf)
}

pub fn new_span_id() -> String {
    let buf: [u8; 8] = rand::random();
    hex::encode(buf)
}

impl TraceContext {
    pub fn new() -> Self {
        Self {
            trace_id: new_trace_id(),
            span_id: new_span_id(),
            flags: 0x01,
        }
    }

    pub fn to_traceparent(&self) -> String {
        format!("00-{}-{}-{:02x}", self.trace_id, self.span_id, self.flags)
    }

    pub fn from_traceparent(header: &str) -> Option<Self> {
        let parts: Vec<&str> = header.split('-').collect();
        if parts.len() != 4 || parts[0] != "00" {
            return None;
        }
        let trace_id = parts[1];
        let span_id = parts[2];
        let flags_str = parts[3];
        if trace_id.len() != 32 || span_id.len() != 16 || flags_str.len() != 2 {
            return None;
        }
        if !trace_id.chars().all(|c| c.is_ascii_hexdigit())
            || !span_id.chars().all(|c| c.is_ascii_hexdigit())
        {
            return None;
        }
        let flags = u8::from_str_radix(flags_str, 16).ok()?;
        Some(Self {
            trace_id: trace_id.to_string(),
            span_id: span_id.to_string(),
            flags,
        })
    }

    pub fn child(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: new_span_id(),
            flags: self.flags,
        }
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn inject(ctx: &TraceContext, carrier: &mut HashMap<String, serde_json::Value>) {
    carrier.insert(
        TRACEPARENT_KEY.to_string(),
        serde_json::Value::String(ctx.to_traceparent()),
    );
}

pub fn extract(carrier: &HashMap<String, serde_json::Value>) -> Option<TraceContext> {
    carrier
        .get(TRACEPARENT_KEY)
        .and_then(|v| v.as_str())
        .and_then(TraceContext::from_traceparent)
}

pub fn extract_from_json_rpc(request: &serde_json::Value) -> Option<TraceContext> {
    request
        .get(TRACEPARENT_KEY)
        .and_then(|v| v.as_str())
        .and_then(TraceContext::from_traceparent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_context_inject_extract() {
        let ctx = TraceContext {
            trace_id: "a".repeat(32),
            span_id: "b".repeat(16),
            flags: 0x01,
        };
        let mut carrier = HashMap::new();
        inject(&ctx, &mut carrier);

        let extracted = extract(&carrier).expect("extract succeeds");
        assert_eq!(extracted, ctx);
    }

    #[test]
    fn test_trace_context_from_json_rpc() {
        let tp = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "task.execute",
            "params": {},
            "id": 1,
            "_traceparent": tp
        });
        let ctx = extract_from_json_rpc(&request).expect("extract from JSON-RPC");
        assert_eq!(ctx.trace_id, "0af7651916cd43dd8448eb211c80319c");
        assert_eq!(ctx.span_id, "b7ad6b7169203331");
        assert_eq!(ctx.flags, 0x01);
    }

    #[test]
    fn test_traceparent_roundtrip() {
        let ctx = TraceContext::new();
        let header = ctx.to_traceparent();
        let parsed = TraceContext::from_traceparent(&header).expect("parse");
        assert_eq!(parsed, ctx);
    }

    #[test]
    fn test_child_shares_trace_id() {
        let parent = TraceContext::new();
        let child = parent.child();
        assert_eq!(child.trace_id, parent.trace_id);
        assert_ne!(child.span_id, parent.span_id);
    }

    #[test]
    fn test_invalid_traceparent_rejected() {
        assert!(TraceContext::from_traceparent("").is_none());
        assert!(TraceContext::from_traceparent("01-abc-def-00").is_none());
        assert!(TraceContext::from_traceparent("00-short-short-00").is_none());
        assert!(
            TraceContext::from_traceparent(
                "00-zzzz0000111122223333444455556666-bbbb000011112222-01"
            )
            .is_none()
        );
    }
}
