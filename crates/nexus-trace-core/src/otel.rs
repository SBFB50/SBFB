// SPDX-License-Identifier: AGPL-3.0-or-later
//! [`OtelProcessor`] — bridge [`TraceEvent`] to an OpenTelemetry
//! `SdkTracerProvider`. Plug any OTel-compatible exporter (OTLP,
//! stdout, Jaeger) at construction time via the provider builder.

use opentelemetry::KeyValue;
use opentelemetry::trace::{Span, Tracer, TracerProvider};
use opentelemetry_sdk::trace::SdkTracerProvider;

use crate::{TraceEvent, TraceProcessor};

pub struct OtelProcessor {
    provider: SdkTracerProvider,
}

impl OtelProcessor {
    pub fn new(provider: SdkTracerProvider) -> Self {
        Self { provider }
    }
}

impl TraceProcessor for OtelProcessor {
    fn process(&self, event: &TraceEvent) {
        let tracer = self.provider.tracer("nexus-trace");
        let mut span = tracer.start(event.name.clone());
        span.set_attribute(KeyValue::new("service.name", event.service_name.clone()));
        span.set_attribute(KeyValue::new("trace.id", event.trace_id.clone()));
        for (k, v) in &event.attributes {
            span.set_attribute(KeyValue::new(k.clone(), v.to_string()));
        }
        span.end();
    }

    fn shutdown(&self) {
        let _ = self.provider.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Clone)]
    struct RecordingExporter {
        count: Arc<Mutex<u32>>,
    }

    impl opentelemetry_sdk::trace::SpanExporter for RecordingExporter {
        fn export(
            &self,
            batch: Vec<opentelemetry_sdk::trace::SpanData>,
        ) -> impl std::future::Future<Output = opentelemetry_sdk::error::OTelSdkResult> + Send
        {
            let mut c = self.count.lock().unwrap();
            *c += batch.len() as u32;
            std::future::ready(Ok(()))
        }
    }

    #[test]
    fn test_otel_processor_export_mock() {
        let count = Arc::new(Mutex::new(0u32));
        let exporter = RecordingExporter {
            count: Arc::clone(&count),
        };
        let provider = SdkTracerProvider::builder()
            .with_simple_exporter(exporter)
            .build();
        let proc = OtelProcessor::new(provider);

        let event = TraceEvent {
            trace_id: "a".repeat(32),
            span_id: "b".repeat(16),
            parent_span_id: None,
            timestamp_unix_ns: 1_000_000_000,
            name: "test.otel".into(),
            service_name: "otel-test".into(),
            attributes: HashMap::from([("key".into(), serde_json::json!("value"))]),
        };
        proc.process(&event);
        proc.shutdown();

        let exported = *count.lock().unwrap();
        assert!(
            exported >= 1,
            "exporter must have received at least 1 span, got {exported}"
        );
    }
}
