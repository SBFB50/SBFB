// SPDX-License-Identifier: AGPL-3.0-or-later
//! Backend-agnostic trace infrastructure for SBFB.
//!
//! Three concrete [`TraceProcessor`] backends ship with this crate:
//!
//! - [`batch_log::BatchLogProcessor`] — JSON structured → rotating
//!   JSONL file (default backend, zero external deps).
//! - [`otel::OtelProcessor`] — bridge to OpenTelemetry 0.31
//!   `SdkTracerProvider`. Plug any OTel exporter (OTLP, stdout,
//!   Jaeger) at construction time.
//! - [`signed::SignedCanaryProcessor`] — Ed25519-signed trace
//!   events for tamper-evident audit trails.
//!
//! W3C Trace Context propagation helpers live in [`propagation`].

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod batch_log;
pub mod otel;
pub mod propagation;
pub mod signed;

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use serde::{Deserialize, Serialize};

pub const DOMAIN_TRACE_EVENT_V1: &[u8] = b"nexus-trace-event-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    pub trace_id: String,
    pub span_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
    pub timestamp_unix_ns: u64,
    pub name: String,
    pub service_name: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, serde_json::Value>,
}

pub trait TraceProcessor: Send + Sync {
    fn process(&self, event: &TraceEvent);
    fn shutdown(&self) {}
}

static PROCESSORS: OnceLock<RwLock<Vec<Box<dyn TraceProcessor>>>> = OnceLock::new();

fn processors() -> &'static RwLock<Vec<Box<dyn TraceProcessor>>> {
    PROCESSORS.get_or_init(|| RwLock::new(Vec::new()))
}

pub fn add_trace_processor(processor: Box<dyn TraceProcessor>) {
    let mut lock = processors().write().expect("trace processors lock");
    lock.push(processor);
}

pub fn set_trace_processors(new_processors: Vec<Box<dyn TraceProcessor>>) {
    let mut lock = processors().write().expect("trace processors lock");
    for p in lock.drain(..) {
        p.shutdown();
    }
    *lock = new_processors;
}

pub fn emit(event: &TraceEvent) {
    if let Some(lock) = PROCESSORS.get()
        && let Ok(guard) = lock.read()
    {
        for p in guard.iter() {
            p.process(event);
        }
    }
}

pub fn shutdown_processors() {
    if let Some(lock) = PROCESSORS.get()
        && let Ok(mut guard) = lock.write()
    {
        for p in guard.drain(..) {
            p.shutdown();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct CountingProcessor(Arc<AtomicU32>);

    impl TraceProcessor for CountingProcessor {
        fn process(&self, _event: &TraceEvent) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn sample_event() -> TraceEvent {
        TraceEvent {
            trace_id: "a".repeat(32),
            span_id: "b".repeat(16),
            parent_span_id: None,
            timestamp_unix_ns: 1_000_000_000,
            name: "test.event".into(),
            service_name: "nexus-test".into(),
            attributes: HashMap::new(),
        }
    }

    #[test]
    fn test_multi_processor_pipeline() {
        let c1 = Arc::new(AtomicU32::new(0));
        let c2 = Arc::new(AtomicU32::new(0));
        set_trace_processors(vec![
            Box::new(CountingProcessor(Arc::clone(&c1))),
            Box::new(CountingProcessor(Arc::clone(&c2))),
        ]);
        emit(&sample_event());
        assert_eq!(c1.load(Ordering::Relaxed), 1);
        assert_eq!(c2.load(Ordering::Relaxed), 1);
        shutdown_processors();
    }

    #[test]
    fn test_set_trace_processors_replaces() {
        let c1 = Arc::new(AtomicU32::new(0));
        let c2 = Arc::new(AtomicU32::new(0));
        set_trace_processors(vec![Box::new(CountingProcessor(Arc::clone(&c1)))]);
        emit(&sample_event());
        assert_eq!(c1.load(Ordering::Relaxed), 1);

        set_trace_processors(vec![Box::new(CountingProcessor(Arc::clone(&c2)))]);
        emit(&sample_event());
        assert_eq!(c2.load(Ordering::Relaxed), 1);
        assert_eq!(c1.load(Ordering::Relaxed), 1);
        shutdown_processors();
    }

    #[test]
    fn test_domain_trace_event_v1() {
        assert_eq!(DOMAIN_TRACE_EVENT_V1, b"nexus-trace-event-v1");
        assert!(!DOMAIN_TRACE_EVENT_V1.contains(&b'\0'));
        assert!(!DOMAIN_TRACE_EVENT_V1.is_empty());
    }
}
