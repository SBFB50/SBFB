// SPDX-License-Identifier: AGPL-3.0-or-later
//! [`SignedCanaryProcessor`] — Ed25519-signed trace events for
//! tamper-evident audit trails.
//!
//! Each [`TraceEvent`] is signed with domain separation
//! [`DOMAIN_TRACE_EVENT_V1`] + `0x00` + JSON canonical bytes,
//! producing a [`SignedTraceEvent`] that auditors can verify
//! independently. Output is JSONL (one signed event per line).

use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::{DOMAIN_TRACE_EVENT_V1, TraceEvent, TraceProcessor};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTraceEvent {
    pub event: TraceEvent,
    pub signature: String,
    pub public_key: String,
}

pub struct SignedCanaryProcessor {
    signing_key: SigningKey,
    public_key_hex: String,
    inner: Mutex<Option<BufWriter<File>>>,
}

impl SignedCanaryProcessor {
    pub fn new(signing_key: SigningKey, path: impl Into<PathBuf>) -> std::io::Result<Self> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        let public_key_hex = hex::encode(signing_key.verifying_key().as_bytes());
        Ok(Self {
            signing_key,
            public_key_hex,
            inner: Mutex::new(Some(BufWriter::new(file))),
        })
    }

    fn sign_event(&self, event: &TraceEvent) -> SignedTraceEvent {
        let msg = domain_message(event);
        let signature = self.signing_key.sign(&msg);
        SignedTraceEvent {
            event: event.clone(),
            signature: hex::encode(signature.to_bytes()),
            public_key: self.public_key_hex.clone(),
        }
    }
}

fn domain_message(event: &TraceEvent) -> Vec<u8> {
    let json = serde_json::to_vec(event).expect("TraceEvent serializes");
    let mut msg = Vec::with_capacity(DOMAIN_TRACE_EVENT_V1.len() + 1 + json.len());
    msg.extend_from_slice(DOMAIN_TRACE_EVENT_V1);
    msg.push(0x00);
    msg.extend_from_slice(&json);
    msg
}

pub fn verify_signed_event(signed: &SignedTraceEvent) -> bool {
    let pk_bytes: [u8; 32] = match hex::decode(&signed.public_key) {
        Ok(b) if b.len() == 32 => {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&b);
            arr
        }
        _ => return false,
    };
    let vk = match VerifyingKey::from_bytes(&pk_bytes) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let sig_bytes: [u8; 64] = match hex::decode(&signed.signature) {
        Ok(b) if b.len() == 64 => {
            let mut arr = [0u8; 64];
            arr.copy_from_slice(&b);
            arr
        }
        _ => return false,
    };
    let sig = Signature::from_bytes(&sig_bytes);
    let msg = domain_message(&signed.event);
    vk.verify(&msg, &sig).is_ok()
}

impl TraceProcessor for SignedCanaryProcessor {
    fn process(&self, event: &TraceEvent) {
        let signed = self.sign_event(event);
        let line = match serde_json::to_vec(&signed) {
            Ok(mut v) => {
                v.push(b'\n');
                v
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to serialize signed trace event");
                return;
            }
        };
        if let Ok(mut guard) = self.inner.lock()
            && let Some(ref mut w) = *guard
            && w.write_all(&line).is_ok()
        {
            let _ = w.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::tempdir;

    fn sample_event() -> TraceEvent {
        TraceEvent {
            trace_id: "a".repeat(32),
            span_id: "b".repeat(16),
            parent_span_id: None,
            timestamp_unix_ns: 1_000_000_000,
            name: "canary.test".into(),
            service_name: "test".into(),
            attributes: HashMap::new(),
        }
    }

    fn test_key() -> SigningKey {
        SigningKey::from_bytes(&[42u8; 32])
    }

    #[test]
    fn test_signed_processor_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("signed.jsonl");
        let proc = SignedCanaryProcessor::new(test_key(), &path).unwrap();

        proc.process(&sample_event());

        let content = fs::read_to_string(&path).unwrap();
        let signed: SignedTraceEvent =
            serde_json::from_str(content.lines().next().unwrap()).unwrap();
        assert!(verify_signed_event(&signed));
        assert_eq!(signed.event.name, "canary.test");
    }

    #[test]
    fn test_signed_processor_tamper_detect() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("signed.jsonl");
        let proc = SignedCanaryProcessor::new(test_key(), &path).unwrap();

        proc.process(&sample_event());

        let content = fs::read_to_string(&path).unwrap();
        let mut signed: SignedTraceEvent =
            serde_json::from_str(content.lines().next().unwrap()).unwrap();

        assert!(verify_signed_event(&signed));

        signed.event.name = "tampered.event".into();
        assert!(!verify_signed_event(&signed));
    }
}
