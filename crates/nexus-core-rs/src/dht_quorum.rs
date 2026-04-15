// SPDX-License-Identifier: AGPL-3.0-or-later
//! Redundant DHT lookup with 2/3 quorum.
//!
//! Part of Sprint 18 Phase C (multi-relai federation + DHT
//! redundant lookup). The goal : mitigate the single-point-of-
//! failure that Eclipse-by-DHT introduces when one pkarr relay
//! lies about a node's published address. By issuing three
//! concurrent lookups across distinct relays and accepting the
//! answer only when at least two agree byte-for-byte, an attacker
//! must compromise a majority of our relay quorum to poison a
//! resolution.
//!
//! # Design
//!
//! The module is deliberately generic over the resolver type. A
//! caller provides an implementation of [`QuorumResolver`] — in
//! production this wraps three pkarr relay clients ; in tests it
//! wraps three mock functions. The core algorithm
//! ([`redundant_resolve`]) does not know about pkarr.
//!
//! This separation is what makes the module unit-testable without
//! standing up real pkarr infrastructure. It also keeps the
//! quorum logic reusable if future sprints want to apply the same
//! pattern to a different resolver kind (e.g. curator list
//! lookups across gossip bootstrap peers).
//!
//! # Quorum rules
//!
//! Given three concurrent resolvers :
//!
//! - **3/3 match** → return `Ok(record)`.
//! - **2/3 match**, third errored or timed out → return `Ok(record)`.
//! - **2/3 match** but their answers differ byte-for-byte from the
//!   third (one relay is lying) → still `Ok(record)` with a warn
//!   log (majority wins, but we record the disagreement).
//! - **1/3 or 0/3 match** → return `Err(QuorumError::NoMajority)`.
//! - **All three errored** → return `Err(QuorumError::AllFailed)`.
//!
//! Byte-for-byte equality is the comparison key : each resolver
//! returns a raw `Vec<u8>` that the caller interprets afterwards
//! (typically a signed pkarr packet). The quorum layer does not
//! parse — if two relays return identical bytes, we treat them as
//! confirming the same record.
//!
//! # Timeout handling
//!
//! The caller supplies a per-lookup budget. Each resolver runs
//! under a [`tokio::time::timeout`] ; a lookup that exceeds the
//! budget contributes to the "errored" bucket. A global deadline
//! is not enforced : three concurrent timeouts of `T` seconds
//! each complete in worst-case ~`T` seconds wall clock.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use thiserror::Error;
use tokio::task::JoinSet;
use tracing::{debug, warn};

/// The resolver abstraction the quorum layer delegates to.
///
/// Implementors fetch a record for `node_id` from exactly one
/// source (one pkarr relay, one curator gossip bootstrap peer,
/// one DNS-over-HTTPS server, …). The output bytes are returned
/// as-is — the quorum layer compares them byte-for-byte without
/// parsing.
#[async_trait]
pub trait QuorumResolver: Send + Sync {
    /// Short human-readable label (e.g. `"relay1.n0.iroh.link"`)
    /// used in warn logs when this resolver disagrees with the
    /// majority or errors.
    fn label(&self) -> &str;

    /// Fetch the raw record bytes for `node_id`. Any error —
    /// network, parse, signature, malformed — collapses to a
    /// single [`anyhow::Error`] that contributes to the failed
    /// count in the quorum tally.
    async fn resolve(&self, node_id: &str) -> anyhow::Result<Vec<u8>>;
}

/// Failure modes of [`redundant_resolve`].
#[derive(Debug, Error)]
pub enum QuorumError {
    /// Fewer than 2 of the 3 resolvers returned the same bytes.
    /// At least one succeeded but no majority emerged — the
    /// quorum has no consensus and refuses to pick a side.
    #[error("no quorum majority: {ok_count}/3 ok, biggest bucket {max_agreement}/3")]
    NoMajority {
        /// Number of resolvers that returned a non-error response.
        ok_count: usize,
        /// Size of the largest equal-bytes bucket across the ok
        /// responses. Always `< 2` when this variant is raised.
        max_agreement: usize,
    },

    /// Every resolver errored or timed out. Indicates either a
    /// network outage or a misconfiguration (all relays
    /// unreachable). Callers should degrade gracefully rather
    /// than block.
    #[error("all {count} resolvers failed")]
    AllFailed {
        /// Always 3 in the current implementation, but carried
        /// for future flexibility if the quorum size changes.
        count: usize,
    },

    /// No resolvers were supplied. Caller bug.
    #[error("quorum requires at least one resolver, got 0")]
    Empty,
}

/// Outcome of a successful quorum resolution.
#[derive(Debug, Clone)]
pub struct QuorumRecord {
    /// The bytes agreed upon by the majority.
    pub bytes: Vec<u8>,
    /// Labels of the resolvers that returned the majority bytes.
    /// Always `len >= 2` when wrapped in `Ok(_)`.
    pub agreeing: Vec<String>,
    /// Labels of resolvers that returned different bytes or
    /// errored. Empty on full agreement (3/3 match).
    pub dissenting: Vec<String>,
}

/// Run `resolvers` in parallel and return the bytes agreed by
/// majority (≥ 2 of 3).
///
/// # Arguments
///
/// - `node_id` — identifier the caller wants to resolve. Passed
///   through to each resolver's `resolve` method unchanged.
/// - `resolvers` — slice of 3 resolvers in the typical case.
///   `len == 0` is a caller bug and yields [`QuorumError::Empty`] ;
///   `len == 1` or `len == 2` still works (quorum reduces to
///   unanimity) but the mitigation intent is weaker.
/// - `per_lookup_timeout` — individual lookup budget ; resolvers
///   that exceed it count as errored.
///
/// # Returns
///
/// `Ok(QuorumRecord)` when at least ⌈N/2⌉+1 resolvers (2 out of
/// 3 for the standard case) returned the same bytes. `Err(..)`
/// otherwise. See [`QuorumError`] for the failure taxonomy.
pub async fn redundant_resolve(
    node_id: &str,
    resolvers: &[Arc<dyn QuorumResolver>],
    per_lookup_timeout: Duration,
) -> Result<QuorumRecord, QuorumError> {
    if resolvers.is_empty() {
        return Err(QuorumError::Empty);
    }

    let total = resolvers.len();
    let quorum_threshold = quorum_threshold_for(total);

    // Spawn one task per resolver. JoinSet is the tokio-native
    // "collect results from N tasks" primitive ; it drops all
    // in-flight tasks when dropped so an early exit via `?` does
    // not leak work.
    let mut set: JoinSet<(String, anyhow::Result<Vec<u8>>)> = JoinSet::new();
    for resolver in resolvers {
        let resolver = Arc::clone(resolver);
        let node_id = node_id.to_string();
        let label = resolver.label().to_string();
        set.spawn(async move {
            let outcome =
                match tokio::time::timeout(per_lookup_timeout, resolver.resolve(&node_id)).await {
                    Ok(Ok(bytes)) => Ok(bytes),
                    Ok(Err(e)) => Err(e),
                    Err(_elapsed) => Err(anyhow::anyhow!(
                        "per-lookup timeout {:?} elapsed",
                        per_lookup_timeout
                    )),
                };
            (label, outcome)
        });
    }

    let mut successes: Vec<(String, Vec<u8>)> = Vec::with_capacity(total);
    let mut failures: Vec<String> = Vec::new();

    while let Some(joined) = set.join_next().await {
        let (label, outcome) = joined.unwrap_or_else(|join_err| {
            // JoinError = task panicked. Count as failure.
            ("<panicked>".into(), Err(anyhow::Error::from(join_err)))
        });

        match outcome {
            Ok(bytes) => {
                debug!(resolver = %label, bytes_len = bytes.len(), "quorum resolver returned bytes");
                successes.push((label, bytes));
            }
            Err(err) => {
                warn!(resolver = %label, error = %err, "quorum resolver failed");
                failures.push(label);
            }
        }
    }

    if successes.is_empty() {
        return Err(QuorumError::AllFailed { count: total });
    }

    // Bucket successes by their byte payload. The bucket with the
    // largest count is the majority candidate.
    let mut buckets: HashMap<Vec<u8>, Vec<String>> = HashMap::new();
    for (label, bytes) in &successes {
        buckets
            .entry(bytes.clone())
            .or_default()
            .push(label.clone());
    }

    let (winning_bytes, winning_labels) = buckets
        .into_iter()
        .max_by_key(|(_, labels)| labels.len())
        .expect("successes non-empty → at least one bucket");

    if winning_labels.len() < quorum_threshold {
        return Err(QuorumError::NoMajority {
            ok_count: successes.len(),
            max_agreement: winning_labels.len(),
        });
    }

    // Disagreeing set = all resolver labels that did not land in
    // the winning bucket. This includes both successes with
    // different bytes and outright failures.
    let mut dissenting: Vec<String> = failures;
    for (label, bytes) in &successes {
        if *bytes != winning_bytes {
            dissenting.push(label.clone());
        }
    }

    if !dissenting.is_empty() {
        warn!(
            agreeing = winning_labels.len(),
            dissenting = dissenting.len(),
            ok_count = successes.len(),
            total = total,
            "quorum reached with disagreement — majority wins",
        );
    }

    Ok(QuorumRecord {
        bytes: winning_bytes,
        agreeing: winning_labels,
        dissenting,
    })
}

/// Smallest number of agreeing resolvers required for quorum.
/// For the canonical N=3 case this is 2 ; more generally
/// `⌊N/2⌋ + 1` (strict majority).
#[inline]
fn quorum_threshold_for(total: usize) -> usize {
    total / 2 + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Mock resolver driven by a scripted response.
    struct MockResolver {
        label: String,
        response: Mutex<Option<anyhow::Result<Vec<u8>>>>,
        /// Fake network delay so we can test that timeouts kick in.
        delay: Duration,
    }

    impl MockResolver {
        fn ok(label: &str, bytes: &[u8]) -> Arc<dyn QuorumResolver> {
            Arc::new(Self {
                label: label.into(),
                response: Mutex::new(Some(Ok(bytes.to_vec()))),
                delay: Duration::ZERO,
            })
        }

        fn fail(label: &str, msg: &str) -> Arc<dyn QuorumResolver> {
            Arc::new(Self {
                label: label.into(),
                response: Mutex::new(Some(Err(anyhow::anyhow!("{}", msg)))),
                delay: Duration::ZERO,
            })
        }

        fn slow(label: &str, bytes: &[u8], delay: Duration) -> Arc<dyn QuorumResolver> {
            Arc::new(Self {
                label: label.into(),
                response: Mutex::new(Some(Ok(bytes.to_vec()))),
                delay,
            })
        }
    }

    #[async_trait]
    impl QuorumResolver for MockResolver {
        fn label(&self) -> &str {
            &self.label
        }

        async fn resolve(&self, _node_id: &str) -> anyhow::Result<Vec<u8>> {
            if !self.delay.is_zero() {
                tokio::time::sleep(self.delay).await;
            }
            self.response
                .lock()
                .unwrap()
                .take()
                .expect("MockResolver consumed twice — use a fresh one per test")
        }
    }

    #[tokio::test]
    async fn returns_record_on_3_of_3_match() {
        let resolvers = vec![
            MockResolver::ok("r1", b"payload"),
            MockResolver::ok("r2", b"payload"),
            MockResolver::ok("r3", b"payload"),
        ];
        let rec = redundant_resolve("node-a", &resolvers, Duration::from_secs(1))
            .await
            .expect("3/3 match must yield Ok");
        assert_eq!(rec.bytes, b"payload");
        assert_eq!(rec.agreeing.len(), 3);
        assert!(rec.dissenting.is_empty());
    }

    #[tokio::test]
    async fn returns_record_on_2_of_3_match_third_errored() {
        let resolvers = vec![
            MockResolver::ok("r1", b"payload"),
            MockResolver::ok("r2", b"payload"),
            MockResolver::fail("r3", "network unreachable"),
        ];
        let rec = redundant_resolve("node-a", &resolvers, Duration::from_secs(1))
            .await
            .expect("2 matching + 1 error = majority");
        assert_eq!(rec.bytes, b"payload");
        assert_eq!(rec.agreeing.len(), 2);
        assert_eq!(rec.dissenting, vec!["r3"]);
    }

    #[tokio::test]
    async fn returns_record_on_2_of_3_match_third_lied() {
        // Two relays agree, a third returns different bytes — the
        // "one relay is lying" scenario. Quorum still accepts the
        // majority but flags the dissenting label.
        let resolvers = vec![
            MockResolver::ok("honest-a", b"truth"),
            MockResolver::ok("honest-b", b"truth"),
            MockResolver::ok("liar", b"forgery"),
        ];
        let rec = redundant_resolve("node-a", &resolvers, Duration::from_secs(1))
            .await
            .expect("2 matching + 1 lie = majority truth");
        assert_eq!(rec.bytes, b"truth");
        assert_eq!(rec.agreeing.len(), 2);
        assert_eq!(rec.dissenting, vec!["liar"]);
    }

    #[tokio::test]
    async fn errs_on_1_of_3_match() {
        // Three successes but all three return different bytes.
        // No majority possible — refuse to pick.
        let resolvers = vec![
            MockResolver::ok("r1", b"one"),
            MockResolver::ok("r2", b"two"),
            MockResolver::ok("r3", b"three"),
        ];
        let err = redundant_resolve("node-a", &resolvers, Duration::from_secs(1))
            .await
            .expect_err("all-different bytes → no majority");
        match err {
            QuorumError::NoMajority {
                ok_count,
                max_agreement,
            } => {
                assert_eq!(ok_count, 3);
                assert_eq!(max_agreement, 1);
            }
            other => panic!("expected NoMajority, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn errs_when_all_failed() {
        let resolvers = vec![
            MockResolver::fail("r1", "boom"),
            MockResolver::fail("r2", "boom"),
            MockResolver::fail("r3", "boom"),
        ];
        let err = redundant_resolve("node-a", &resolvers, Duration::from_secs(1))
            .await
            .expect_err("all errors → AllFailed");
        assert!(matches!(err, QuorumError::AllFailed { count: 3 }));
    }

    #[tokio::test(start_paused = true)]
    async fn per_lookup_timeout_kicks_in() {
        // Two resolvers respond quickly with matching bytes, one
        // sleeps past the deadline — the slow one counts as
        // errored, the two fast ones carry the quorum.
        let resolvers = vec![
            MockResolver::ok("fast-1", b"payload"),
            MockResolver::ok("fast-2", b"payload"),
            MockResolver::slow("slow", b"would-match", Duration::from_secs(60)),
        ];
        let rec = redundant_resolve("node-a", &resolvers, Duration::from_millis(50))
            .await
            .expect("2 fast matches still carry quorum when slow resolver times out");
        assert_eq!(rec.bytes, b"payload");
        assert_eq!(rec.agreeing.len(), 2);
        assert_eq!(rec.dissenting, vec!["slow"]);
    }

    #[test]
    fn quorum_threshold_for_matches_strict_majority() {
        assert_eq!(quorum_threshold_for(1), 1);
        assert_eq!(quorum_threshold_for(2), 2);
        assert_eq!(quorum_threshold_for(3), 2);
        assert_eq!(quorum_threshold_for(5), 3);
    }

    #[tokio::test]
    async fn errs_on_empty_resolvers() {
        let resolvers: Vec<Arc<dyn QuorumResolver>> = Vec::new();
        let err = redundant_resolve("any", &resolvers, Duration::from_secs(1))
            .await
            .expect_err("no resolvers → Empty");
        assert!(matches!(err, QuorumError::Empty));
    }
}
