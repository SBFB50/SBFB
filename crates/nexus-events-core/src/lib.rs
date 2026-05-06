// SPDX-License-Identifier: AGPL-3.0-or-later
//! OS audit SecurityEvent system.
//!
//! Typed [`SecurityEvent`] enum covering 14 security-relevant event
//! categories across the SBFB stack. [`EventWriter`] trait abstracts
//! platform-specific output: [`JsonFileWriter`] (append-only JSONL with
//! size-based rotation, primary), [`TracingWriter`] (cross-platform
//! tracing structured events), [`JournaldWriter`] (Linux journald via
//! `libsystemd` pure-Rust), [`OsLogWriter`] (macOS Unified Logging
//! via `oslog`). Non-target platforms get stub fallbacks.
//!
//! A global [`emit_event`] singleton routes events to the writer
//! initialized at daemon/worker startup via [`init_emitter`] or
//! [`init_platform_emitter`] (auto-selects per platform).

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use chrono::Utc;
use serde::{Deserialize, Serialize};

// ======================================================================
// SecurityEvent
// ======================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event_type", content = "payload")]
pub enum SecurityEvent {
    ConsentChange {
        previous: String,
        current: String,
    },
    PanicFired {
        trigger: String,
    },
    TokenRotation {
        rotated_at: String,
    },
    DuressUnlock {
        mode: String,
    },
    QuarantineDrop {
        task_id: String,
        reason: String,
    },
    SybilAdmissionReject {
        node_id: String,
        reason: String,
    },
    PowVerifyFail {
        difficulty: u32,
        peer: String,
    },
    CanaryPublished {
        version: u32,
    },
    CanaryDeadMansSwitchTripped {
        last_seen: String,
    },
    TransportDegraded {
        mode: String,
        reason: String,
    },
    RateLimitTierBreach {
        consumer: String,
        tier: String,
    },
    CapabilityChanged {
        name: String,
        enabled: bool,
    },
    ExecutorCrash {
        pid: u32,
        exit_code: Option<i32>,
        restart_count: u32,
    },
    BrokerCrash {
        reason: String,
    },
}

// ======================================================================
// AuditRecord
// ======================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub timestamp: String,
    pub event: SecurityEvent,
}

// ======================================================================
// Errors
// ======================================================================

#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("platform: {0}")]
    Platform(String),
}

// ======================================================================
// EventWriter trait
// ======================================================================

pub trait EventWriter: Send + Sync {
    fn write_event(&self, event: &SecurityEvent) -> Result<(), EventError>;
}

// ======================================================================
// JsonFileWriter — append-only JSONL audit log
// ======================================================================

const DEFAULT_MAX_BYTES: u64 = 10 * 1024 * 1024; // 10 MiB
const MAX_ROTATED_FILES: u32 = 5;

pub struct JsonFileWriter {
    path: PathBuf,
    max_bytes: u64,
}

impl JsonFileWriter {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            max_bytes: DEFAULT_MAX_BYTES,
        }
    }

    pub fn with_max_bytes(path: impl Into<PathBuf>, max_bytes: u64) -> Self {
        Self {
            path: path.into(),
            max_bytes,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn rotate(&self) -> Result<(), EventError> {
        for i in (1..MAX_ROTATED_FILES).rev() {
            let src = self.path.with_extension(format!("jsonl.{i}"));
            let dst = self.path.with_extension(format!("jsonl.{}", i + 1));
            if src.exists() {
                std::fs::rename(&src, &dst)?;
            }
        }
        let dst = self.path.with_extension("jsonl.1");
        std::fs::rename(&self.path, &dst)?;
        Ok(())
    }
}

impl EventWriter for JsonFileWriter {
    fn write_event(&self, event: &SecurityEvent) -> Result<(), EventError> {
        if self.path.exists() {
            if let Ok(meta) = std::fs::metadata(&self.path) {
                if meta.len() >= self.max_bytes {
                    self.rotate()?;
                }
            }
        }
        let record = AuditRecord {
            timestamp: Utc::now().to_rfc3339(),
            event: event.clone(),
        };
        let line = serde_json::to_string(&record)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        writeln!(file, "{line}")?;
        Ok(())
    }
}

// ======================================================================
// TracingWriter — cross-platform tracing structured events
// ======================================================================
//
// Emits security events as tracing::info! with target
// `sbfb_security_events`. When the binary configures a platform-specific
// subscriber layer (ETW on Windows, journald on Linux), these events
// flow to the OS audit system. Otherwise they appear as regular tracing
// output (zero-cost if no subscriber is active).

pub struct TracingWriter;

impl EventWriter for TracingWriter {
    fn write_event(&self, event: &SecurityEvent) -> Result<(), EventError> {
        let json = serde_json::to_string(event)?;
        tracing::info!(
            target: "sbfb_security_events",
            event_json = %json,
            "security_event"
        );
        Ok(())
    }
}

// ======================================================================
// Shared format helpers (testable cross-platform)
// ======================================================================

pub fn event_type_name(event: &SecurityEvent) -> &'static str {
    match event {
        SecurityEvent::ConsentChange { .. } => "ConsentChange",
        SecurityEvent::PanicFired { .. } => "PanicFired",
        SecurityEvent::TokenRotation { .. } => "TokenRotation",
        SecurityEvent::DuressUnlock { .. } => "DuressUnlock",
        SecurityEvent::QuarantineDrop { .. } => "QuarantineDrop",
        SecurityEvent::SybilAdmissionReject { .. } => "SybilAdmissionReject",
        SecurityEvent::PowVerifyFail { .. } => "PowVerifyFail",
        SecurityEvent::CanaryPublished { .. } => "CanaryPublished",
        SecurityEvent::CanaryDeadMansSwitchTripped { .. } => "CanaryDeadMansSwitchTripped",
        SecurityEvent::TransportDegraded { .. } => "TransportDegraded",
        SecurityEvent::RateLimitTierBreach { .. } => "RateLimitTierBreach",
        SecurityEvent::CapabilityChanged { .. } => "CapabilityChanged",
        SecurityEvent::ExecutorCrash { .. } => "ExecutorCrash",
        SecurityEvent::BrokerCrash { .. } => "BrokerCrash",
    }
}

pub fn format_journal_fields(event: &SecurityEvent) -> Vec<(&'static str, String)> {
    let json = serde_json::to_string(event).unwrap_or_default();
    vec![
        ("SBFB_EVENT_TYPE", event_type_name(event).to_string()),
        ("SBFB_DETAILS", json),
    ]
}

pub fn format_oslog_message(event: &SecurityEvent) -> String {
    let json = serde_json::to_string(event).unwrap_or_default();
    format!("[sbfb:{}] {}", event_type_name(event), json)
}

// ======================================================================
// JournaldWriter — Linux journald
// ======================================================================

#[cfg(target_os = "linux")]
pub struct JournaldWriter;

#[cfg(target_os = "linux")]
impl EventWriter for JournaldWriter {
    fn write_event(&self, event: &SecurityEvent) -> Result<(), EventError> {
        use libsystemd::logging::{Priority, journal_send};
        let fields = format_journal_fields(event);
        let msg = format!("sbfb security event: {}", event_type_name(event));
        journal_send(
            Priority::Info,
            &msg,
            fields.iter().map(|(k, v)| (*k, v.as_str())),
        )
        .map_err(|e| EventError::Platform(e.to_string()))?;
        Ok(())
    }
}

#[cfg(not(target_os = "linux"))]
pub struct JournaldWriter;

#[cfg(not(target_os = "linux"))]
impl EventWriter for JournaldWriter {
    fn write_event(&self, _event: &SecurityEvent) -> Result<(), EventError> {
        tracing::debug!("journald writer stub — not on Linux");
        Ok(())
    }
}

// ======================================================================
// OsLogWriter — macOS Unified Logging
// ======================================================================

#[cfg(target_os = "macos")]
pub struct OsLogWriter;

#[cfg(target_os = "macos")]
impl EventWriter for OsLogWriter {
    fn write_event(&self, event: &SecurityEvent) -> Result<(), EventError> {
        let log = oslog::OsLog::new("com.sbfb.security", "events");
        let msg = format_oslog_message(event);
        log.with_level(oslog::Level::Default, &msg);
        Ok(())
    }
}

#[cfg(not(target_os = "macos"))]
pub struct OsLogWriter;

#[cfg(not(target_os = "macos"))]
impl EventWriter for OsLogWriter {
    fn write_event(&self, _event: &SecurityEvent) -> Result<(), EventError> {
        tracing::debug!("oslog writer stub — not on macOS");
        Ok(())
    }
}

// ======================================================================
// Global emitter singleton
// ======================================================================

static SECURITY_EMITTER: OnceLock<Box<dyn EventWriter>> = OnceLock::new();

pub fn init_emitter(writer: Box<dyn EventWriter>) {
    let _ = SECURITY_EMITTER.set(writer);
}

pub fn emit_event(event: &SecurityEvent) {
    if let Some(writer) = SECURITY_EMITTER.get() {
        if let Err(e) = writer.write_event(event) {
            tracing::warn!(error = %e, "failed to emit security event");
        }
    }
}

pub fn init_platform_emitter() {
    #[cfg(target_os = "linux")]
    init_emitter(Box::new(JournaldWriter));

    #[cfg(target_os = "macos")]
    init_emitter(Box::new(OsLogWriter));

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    init_emitter(Box::new(TracingWriter));
}

// ======================================================================
// Tests
// ======================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockWriter {
        events: Mutex<Vec<SecurityEvent>>,
    }

    impl MockWriter {
        fn new() -> Self {
            Self {
                events: Mutex::new(Vec::new()),
            }
        }

        fn events(&self) -> Vec<SecurityEvent> {
            self.events.lock().unwrap().clone()
        }
    }

    impl EventWriter for MockWriter {
        fn write_event(&self, event: &SecurityEvent) -> Result<(), EventError> {
            self.events.lock().unwrap().push(event.clone());
            Ok(())
        }
    }

    fn all_variants() -> Vec<SecurityEvent> {
        vec![
            SecurityEvent::ConsentChange {
                previous: "L1".into(),
                current: "L3".into(),
            },
            SecurityEvent::PanicFired {
                trigger: "5-tap".into(),
            },
            SecurityEvent::TokenRotation {
                rotated_at: "2026-04-24T12:00:00Z".into(),
            },
            SecurityEvent::DuressUnlock {
                mode: "fake-keypair".into(),
            },
            SecurityEvent::QuarantineDrop {
                task_id: "t-123".into(),
                reason: "pii-detected".into(),
            },
            SecurityEvent::SybilAdmissionReject {
                node_id: "abc123".into(),
                reason: "age<7d".into(),
            },
            SecurityEvent::PowVerifyFail {
                difficulty: 20,
                peer: "peer-xyz".into(),
            },
            SecurityEvent::CanaryPublished { version: 3 },
            SecurityEvent::CanaryDeadMansSwitchTripped {
                last_seen: "2026-04-20".into(),
            },
            SecurityEvent::TransportDegraded {
                mode: "udp".into(),
                reason: "relay-only".into(),
            },
            SecurityEvent::RateLimitTierBreach {
                consumer: "worker-1".into(),
                tier: "burst".into(),
            },
            SecurityEvent::CapabilityChanged {
                name: "tool_calling".into(),
                enabled: true,
            },
            SecurityEvent::ExecutorCrash {
                pid: 12345,
                exit_code: Some(137),
                restart_count: 3,
            },
            SecurityEvent::BrokerCrash {
                reason: "ipc-disconnect".into(),
            },
        ]
    }

    #[test]
    fn security_event_serialize_all_variants() {
        for event in all_variants() {
            let json = serde_json::to_string(&event).unwrap();
            let roundtrip: SecurityEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(event, roundtrip);
        }
    }

    #[test]
    fn event_type_tag_correct() {
        let event = SecurityEvent::CapabilityChanged {
            name: "mcp_server_expose".into(),
            enabled: true,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains(r#""event_type":"CapabilityChanged""#));
    }

    #[test]
    fn audit_record_has_timestamp() {
        let record = AuditRecord {
            timestamp: "2026-04-24T12:00:00+00:00".into(),
            event: SecurityEvent::PanicFired {
                trigger: "test".into(),
            },
        };
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains(r#""timestamp":"2026-04-24T12:00:00+00:00""#));
        assert!(json.contains(r#""event_type":"PanicFired""#));
    }

    #[test]
    fn json_file_writer_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("audit.jsonl");

        let writer = JsonFileWriter::new(&path);
        writer
            .write_event(&SecurityEvent::PanicFired {
                trigger: "test".into(),
            })
            .unwrap();

        assert!(path.exists());
    }

    #[test]
    fn json_file_writer_append() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("audit.jsonl");

        let writer = JsonFileWriter::new(&path);
        writer
            .write_event(&SecurityEvent::PanicFired {
                trigger: "first".into(),
            })
            .unwrap();
        writer
            .write_event(&SecurityEvent::TokenRotation {
                rotated_at: "now".into(),
            })
            .unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.trim().split('\n').collect();
        assert_eq!(lines.len(), 2);

        let r1: AuditRecord = serde_json::from_str(lines[0]).unwrap();
        let r2: AuditRecord = serde_json::from_str(lines[1]).unwrap();
        assert!(matches!(r1.event, SecurityEvent::PanicFired { .. }));
        assert!(matches!(r2.event, SecurityEvent::TokenRotation { .. }));
    }

    #[test]
    fn json_file_writer_invalid_path() {
        let writer = JsonFileWriter::new("/nonexistent/dir/audit.jsonl");
        let result = writer.write_event(&SecurityEvent::PanicFired {
            trigger: "test".into(),
        });
        assert!(result.is_err());
    }

    #[test]
    fn stub_writers_noop() {
        let j = JournaldWriter;
        let o = OsLogWriter;
        j.write_event(&SecurityEvent::PanicFired {
            trigger: "test".into(),
        })
        .unwrap();
        o.write_event(&SecurityEvent::PanicFired {
            trigger: "test".into(),
        })
        .unwrap();
    }

    #[test]
    fn event_type_name_matches_serde_tag() {
        for event in all_variants() {
            let name = event_type_name(&event);
            let json = serde_json::to_string(&event).unwrap();
            assert!(
                json.contains(&format!(r#""event_type":"{name}""#)),
                "event_type_name mismatch for {name}: {json}"
            );
        }
    }

    #[test]
    fn format_journal_fields_structured() {
        let event = SecurityEvent::PanicFired {
            trigger: "5-tap".into(),
        };
        let fields = format_journal_fields(&event);
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].0, "SBFB_EVENT_TYPE");
        assert_eq!(fields[0].1, "PanicFired");
        assert_eq!(fields[1].0, "SBFB_DETAILS");
        assert!(fields[1].1.contains("PanicFired"));
        assert!(fields[1].1.contains("5-tap"));
    }

    #[test]
    fn format_oslog_message_structured() {
        let event = SecurityEvent::CanaryPublished { version: 3 };
        let msg = format_oslog_message(&event);
        assert!(msg.starts_with("[sbfb:CanaryPublished]"));
        assert!(msg.contains(r#""version":3"#));
    }

    #[test]
    fn format_journal_fields_all_variants() {
        for event in all_variants() {
            let fields = format_journal_fields(&event);
            assert_eq!(fields.len(), 2);
            let json: serde_json::Value = serde_json::from_str(&fields[1].1).unwrap();
            assert!(json.get("event_type").is_some());
        }
    }

    #[test]
    fn format_oslog_message_all_variants() {
        for event in all_variants() {
            let msg = format_oslog_message(&event);
            let name = event_type_name(&event);
            assert!(msg.starts_with(&format!("[sbfb:{name}]")));
        }
    }

    #[test]
    fn mock_writer_receives_events() {
        let mock = MockWriter::new();
        mock.write_event(&SecurityEvent::CapabilityChanged {
            name: "tool_calling".into(),
            enabled: true,
        })
        .unwrap();
        mock.write_event(&SecurityEvent::PanicFired {
            trigger: "test".into(),
        })
        .unwrap();

        let events = mock.events();
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], SecurityEvent::CapabilityChanged { .. }));
        assert!(matches!(events[1], SecurityEvent::PanicFired { .. }));
    }

    #[test]
    fn json_file_writer_rotation() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("audit.jsonl");

        // Use a tiny max_bytes to trigger rotation quickly.
        let writer = JsonFileWriter::with_max_bytes(&path, 100);
        // Write events until we exceed max_bytes.
        for i in 0..10 {
            writer
                .write_event(&SecurityEvent::PanicFired {
                    trigger: format!("event-{i}"),
                })
                .unwrap();
        }

        // The current file should exist, plus at least one rotated file.
        assert!(path.exists());
        let rotated_1 = path.with_extension("jsonl.1");
        assert!(
            rotated_1.exists(),
            "expected audit.jsonl.1 to exist after rotation"
        );

        // Rotated files cap at MAX_ROTATED_FILES (5).
        let rotated_6 = path.with_extension("jsonl.6");
        assert!(!rotated_6.exists(), "should not exceed 5 rotated files");
    }

    #[test]
    fn tracing_writer_compiles() {
        let writer = TracingWriter;
        let _ = writer.write_event(&SecurityEvent::PanicFired {
            trigger: "test".into(),
        });
    }

    #[test]
    fn init_platform_emitter_does_not_panic() {
        init_platform_emitter();
        assert!(
            SECURITY_EMITTER.get().is_some(),
            "emitter singleton should be initialized"
        );
    }

    #[test]
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    fn init_platform_emitter_selects_tracing_on_windows() {
        // On Windows (neither linux nor macOS), init_platform_emitter
        // picks TracingWriter. Verify the emitter handles an event
        // without error after init.
        init_platform_emitter();
        emit_event(&SecurityEvent::PanicFired {
            trigger: "platform-test".into(),
        });
    }

    #[test]
    fn security_event_executor_crash_serde() {
        let event = SecurityEvent::ExecutorCrash {
            pid: 9999,
            exit_code: Some(137),
            restart_count: 2,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains(r#""event_type":"ExecutorCrash""#));
        assert!(json.contains(r#""pid":9999"#));
        let roundtrip: SecurityEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, roundtrip);

        let broker = SecurityEvent::BrokerCrash {
            reason: "heartbeat timeout".into(),
        };
        let json2 = serde_json::to_string(&broker).unwrap();
        let rt2: SecurityEvent = serde_json::from_str(&json2).unwrap();
        assert_eq!(broker, rt2);
    }

    #[test]
    fn emit_capability_changed_produces_jsonl() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("emit_test.jsonl");

        let writer = JsonFileWriter::new(&path);
        writer
            .write_event(&SecurityEvent::CapabilityChanged {
                name: "mcp_server_expose".into(),
                enabled: true,
            })
            .unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let record: AuditRecord = serde_json::from_str(content.trim()).unwrap();
        assert!(!record.timestamp.is_empty());
        assert!(matches!(
            record.event,
            SecurityEvent::CapabilityChanged {
                ref name,
                enabled: true
            } if name == "mcp_server_expose"
        ));
    }
}
