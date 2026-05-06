// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration coverage for Sprint 18 Phase C relay federation +
//! DHT quorum primitives.
//!
//! Lives outside the crate so it exercises the public surface
//! (`nexus_core_rs::load_relay_map`, `redundant_resolve`,
//! `QuorumResolver`) exactly the way external callers would wire
//! it. Unit tests under `src/` already cover the internals ;
//! these scenarios stitch the modules together and verify they
//! compose as expected.

use std::fs;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use nexus_core_rs::{
    CUSTOM_RELAYS_ENV, DEV_MODE_ENV, QuorumError, QuorumResolver, RELAYS_FILE_NAME, SBFB_HOME_ENV,
    load_relay_map, redundant_resolve,
};

/// Serialise env-mutating tests. Integration tests and unit
/// tests share the process env, so without a guard we race.
static ENV_GUARD: Mutex<()> = Mutex::new(());

struct EnvSnapshot {
    pairs: Vec<(&'static str, Option<String>)>,
}

impl EnvSnapshot {
    fn capture(keys: &[&'static str]) -> Self {
        let pairs = keys
            .iter()
            .map(|k| (*k, std::env::var(k).ok()))
            .collect::<Vec<_>>();
        for (k, _) in &pairs {
            // SAFETY: test-only; nextest runs each test in its own process.
            unsafe { std::env::remove_var(k) };
        }
        EnvSnapshot { pairs }
    }
}

impl Drop for EnvSnapshot {
    fn drop(&mut self) {
        for (k, v) in &self.pairs {
            match v {
                // SAFETY: test-only; nextest runs each test in its own process.
                Some(val) => unsafe { std::env::set_var(k, val) },
                // SAFETY: test-only; nextest runs each test in its own process.
                None => unsafe { std::env::remove_var(k) },
            }
        }
    }
}

#[test]
fn relay_config_env_takes_precedence_over_file() {
    let _g = ENV_GUARD.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    fs::write(
        tmp.path().join(RELAYS_FILE_NAME),
        r#"{"relays":[{"url":"https://file-only.example.org"}]}"#,
    )
    .unwrap();

    let _snap = EnvSnapshot::capture(&[CUSTOM_RELAYS_ENV, SBFB_HOME_ENV, DEV_MODE_ENV]);
    // SAFETY: test-only; nextest runs each test in its own process.
    unsafe { std::env::set_var(SBFB_HOME_ENV, tmp.path()) };
    // SAFETY: test-only; nextest runs each test in its own process.
    unsafe {
        std::env::set_var(
            CUSTOM_RELAYS_ENV,
            "https://env-a.example.org,https://env-b.example.org,https://env-c.example.org",
        )
    };

    let map = load_relay_map()
        .expect("both sources present should still resolve")
        .expect("env set → Some");
    assert_eq!(
        map.len(),
        3,
        "env overrides file: expected 3 URLs from env, not 1 from file"
    );
}

#[test]
fn relay_config_rejects_http_scheme_at_boundary() {
    let _g = ENV_GUARD.lock().unwrap();
    let _snap = EnvSnapshot::capture(&[CUSTOM_RELAYS_ENV, SBFB_HOME_ENV, DEV_MODE_ENV]);
    // SAFETY: test-only; nextest runs each test in its own process.
    unsafe { std::env::set_var(CUSTOM_RELAYS_ENV, "http://insecure.example.org") };

    let err = load_relay_map().expect_err("http:// must be rejected at load_relay_map boundary");
    assert!(
        err.to_string().contains("https"),
        "error must mention https scheme requirement (got: {err})"
    );
}

// -- QuorumResolver integration -------------------------------------

struct LabeledMock {
    label: String,
    bytes: Mutex<Option<Result<Vec<u8>, &'static str>>>,
}

#[async_trait]
impl QuorumResolver for LabeledMock {
    fn label(&self) -> &str {
        &self.label
    }

    async fn resolve(&self, _node_id: &str) -> anyhow::Result<Vec<u8>> {
        match self.bytes.lock().unwrap().take() {
            Some(Ok(b)) => Ok(b),
            Some(Err(e)) => Err(anyhow::anyhow!(e)),
            None => Err(anyhow::anyhow!("mock exhausted")),
        }
    }
}

fn ok_mock(label: &str, bytes: &[u8]) -> Arc<dyn QuorumResolver> {
    Arc::new(LabeledMock {
        label: label.into(),
        bytes: Mutex::new(Some(Ok(bytes.to_vec()))),
    })
}

fn fail_mock(label: &str, msg: &'static str) -> Arc<dyn QuorumResolver> {
    Arc::new(LabeledMock {
        label: label.into(),
        bytes: Mutex::new(Some(Err(msg))),
    })
}

#[tokio::test]
async fn quorum_end_to_end_primary_up_two_fallbacks_down() {
    // Realistic deployment : the primary returns the record
    // correctly, two fallback relays are unreachable. Quorum
    // policy rejects the single success : a 1/3 result is not a
    // majority, we refuse to commit to it rather than silently
    // accept a potentially poisoned lone relay.
    let resolvers = vec![
        ok_mock("primary", b"record-v1"),
        fail_mock("fallback-a", "dns timeout"),
        fail_mock("fallback-b", "dns timeout"),
    ];
    let err = redundant_resolve("node-abc", &resolvers, Duration::from_secs(2))
        .await
        .expect_err("1/3 success must NOT satisfy quorum");
    assert!(
        matches!(
            err,
            QuorumError::NoMajority {
                ok_count: 1,
                max_agreement: 1
            }
        ),
        "expected NoMajority(1,1), got {err:?}"
    );
}

#[tokio::test]
async fn quorum_end_to_end_two_agree_third_errors() {
    // Two fallbacks agree, primary down. Quorum still succeeds
    // and the primary is listed as dissenting.
    let resolvers = vec![
        fail_mock("primary", "network partition"),
        ok_mock("fallback-a", b"record-v1"),
        ok_mock("fallback-b", b"record-v1"),
    ];
    let rec = redundant_resolve("node-abc", &resolvers, Duration::from_secs(2))
        .await
        .expect("2/3 match is quorum");
    assert_eq!(rec.bytes, b"record-v1");
    assert_eq!(rec.agreeing.len(), 2);
    assert_eq!(rec.dissenting, vec!["primary"]);
}

#[tokio::test]
async fn quorum_end_to_end_all_down_degrades_to_all_failed() {
    // Complete outage. Quorum surfaces an AllFailed error so the
    // caller can degrade (retry later, show offline UX) without
    // conflating it with "one relay happens to disagree".
    let resolvers = vec![
        fail_mock("primary", "boom"),
        fail_mock("fallback-a", "boom"),
        fail_mock("fallback-b", "boom"),
    ];
    let err = redundant_resolve("node-abc", &resolvers, Duration::from_secs(2))
        .await
        .expect_err("all errors → AllFailed");
    assert!(matches!(err, QuorumError::AllFailed { count: 3 }));
}
