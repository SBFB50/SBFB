// SPDX-License-Identifier: AGPL-3.0-or-later
//! [`BatchLogProcessor`] — JSON structured trace events to a
//! rotating JSONL file. Default backend, zero external deps.

use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::{TraceEvent, TraceProcessor};

pub struct BatchLogProcessor {
    inner: Mutex<Inner>,
}

struct Inner {
    path: PathBuf,
    max_bytes: u64,
    writer: Option<BufWriter<File>>,
    bytes_written: u64,
    rotation_count: u32,
}

impl BatchLogProcessor {
    pub fn new(path: impl Into<PathBuf>, max_bytes: u64) -> std::io::Result<Self> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        let bytes_written = file.metadata()?.len();
        Ok(Self {
            inner: Mutex::new(Inner {
                path,
                max_bytes,
                writer: Some(BufWriter::new(file)),
                bytes_written,
                rotation_count: 0,
            }),
        })
    }

    pub fn rotation_count(&self) -> u32 {
        self.inner.lock().expect("lock").rotation_count
    }

    pub fn bytes_written(&self) -> u64 {
        self.inner.lock().expect("lock").bytes_written
    }
}

impl Inner {
    fn rotate(&mut self) {
        if let Some(mut w) = self.writer.take() {
            let _ = w.flush();
        }
        let rotated = format!("{}.{}", self.path.display(), self.rotation_count);
        let _ = fs::rename(&self.path, rotated);
        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            Ok(file) => {
                self.writer = Some(BufWriter::new(file));
                self.bytes_written = 0;
                self.rotation_count += 1;
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to open trace log after rotation");
            }
        }
    }
}

impl TraceProcessor for BatchLogProcessor {
    fn process(&self, event: &TraceEvent) {
        let mut inner = self.inner.lock().expect("lock");
        let line = match serde_json::to_vec(event) {
            Ok(mut v) => {
                v.push(b'\n');
                v
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to serialize trace event");
                return;
            }
        };
        if inner.bytes_written + line.len() as u64 > inner.max_bytes && inner.bytes_written > 0 {
            inner.rotate();
        }
        if let Some(ref mut w) = inner.writer {
            if w.write_all(&line).is_ok() {
                let _ = w.flush();
                inner.bytes_written += line.len() as u64;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::tempdir;

    fn sample_event(name: &str) -> TraceEvent {
        TraceEvent {
            trace_id: "a".repeat(32),
            span_id: "b".repeat(16),
            parent_span_id: None,
            timestamp_unix_ns: 1_000_000_000,
            name: name.into(),
            service_name: "test".into(),
            attributes: HashMap::new(),
        }
    }

    #[test]
    fn test_batch_log_processor_write_and_read() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("trace.jsonl");
        let proc = BatchLogProcessor::new(&path, 1_000_000).unwrap();

        proc.process(&sample_event("event.one"));
        proc.process(&sample_event("event.two"));

        let content = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);

        let parsed: TraceEvent = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(parsed.name, "event.one");
        let parsed2: TraceEvent = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(parsed2.name, "event.two");
    }

    #[test]
    fn test_batch_log_processor_rotation() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("trace.jsonl");
        let proc = BatchLogProcessor::new(&path, 100).unwrap();

        for i in 0..5 {
            proc.process(&sample_event(&format!("event.{i}")));
        }

        assert!(
            proc.rotation_count() >= 1,
            "should have rotated at least once"
        );
        let rotated = format!("{}.0", path.display());
        assert!(
            std::path::Path::new(&rotated).exists(),
            "rotated file must exist"
        );
    }
}
