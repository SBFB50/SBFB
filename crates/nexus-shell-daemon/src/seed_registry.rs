// SPDX-License-Identifier: AGPL-3.0-or-later
//! In-memory best-effort multi-seed registry (Sprint 74 Phase F).
//!
//! Aggregates `SeedAnnounced` feed ops ingested from peers into a
//! "Toi + N pairs (vus recemment)" availability count per project. The
//! registry is deliberately EPHEMERAL (never persisted): a seeder count
//! "seen recently" has no value outside its freshness window. This
//! mirrors a BitTorrent tracker scrape / IPFS reprovide — a point-in-
//! time approximation, never an exact live count (scope cut #11,
//! Checkpoint Q5). Content-addressing (BLAKE3) remains the truth of
//! reachability: a forged "I seed X" announcement cannot let a node
//! actually serve bytes it does not hold (the fetch verifies the hash),
//! so the count may OVER-state but never lies about a fetch succeeding.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use nexus_coordinator_rs::public_feed::{PublicFeedOperation, op_type, try_parse_op};
use serde_json::Value;

/// A seeder is presumed gone if it has not re-announced within this
/// window. Aligned with the IPFS reprovide interval (~22-48h): a node
/// re-emits its `SeedAnnounced` ops at every boot (Phase F), so a peer
/// that has not rebooted/re-announced in 48h drops out of the count.
pub const SEED_SEEN_TTL_SECS: u64 = 48 * 60 * 60;

/// How often `record` runs a GLOBAL sweep of all projects (self-clocked off
/// the announcement timestamps — no external timer in Phase F scope). Between
/// sweeps, reads (`count_recent`) still lazily purge the QUERIED project, so a
/// queried count is always fresh; the global sweep bounds memory for projects
/// that are announced but NEVER queried (e.g. a headless node that only
/// ingests). One hour keeps at most ~1h of distinct (project, seeder)
/// announcements resident between sweeps.
pub const SEED_SWEEP_INTERVAL_SECS: u64 = 60 * 60;

#[derive(Default)]
struct RegistryState {
    /// `(project_id, archive_hash) -> (seeder_node_id -> last_seen_unix_secs)`.
    ///
    /// Sprint 75 Phase C (WIRE-2): keyed by the `(project_id, archive_hash)`
    /// PAIR, not `project_id` alone. A seeder announces a SPECIFIC archive blob
    /// (`SeedAnnounced.archive_hash`); collapsing distinct archive versions of
    /// the same project onto one key over-counts (a seeder of version B cannot
    /// serve version A's bytes — content-addressing). Keying by the pair lets a
    /// version-aware read count exactly the seeders of the queried hash, while
    /// the version-agnostic read (`archive_hash = None`) still returns the
    /// distinct seeders across all versions (the pre-WIRE-2 semantics).
    seeders: HashMap<(String, String), HashMap<String, u64>>,
    /// The `seen_at` of the last global sweep (0 = never swept).
    last_sweep: u64,
}

/// Best-effort in-memory aggregate of distinct seeders per project.
///
/// Fed by the feed ingest path for REMOTE seeders only — self is counted at
/// query time from the local `keep_online` row, never via the echo of our own
/// announcement. Reads lazily purge the queried project; `record` additionally
/// runs a throttled global sweep so never-queried projects cannot accumulate
/// for the process lifetime.
#[derive(Default)]
pub struct SeedRegistry {
    inner: Mutex<RegistryState>,
}

/// Drop every seeder last seen before `now - SEED_SEEN_TTL_SECS`, and every
/// `(project_id, archive_hash)` bucket left empty.
fn sweep_expired(seeders: &mut HashMap<(String, String), HashMap<String, u64>>, now: u64) {
    let cutoff = now.saturating_sub(SEED_SEEN_TTL_SECS);
    seeders.retain(|_, peers| {
        peers.retain(|_, ts| *ts >= cutoff);
        !peers.is_empty()
    });
}

impl SeedRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record that `seeder_node_id` announced it seeds `archive_hash` of
    /// `project_id` at `seen_at` (unix secs). Idempotent per
    /// `(project, archive_hash, seeder)`: a repeat announcement only refreshes
    /// the last-seen timestamp, and a stale replayed ts never overwrites a
    /// fresher observation. Runs a throttled global sweep (every
    /// `SEED_SWEEP_INTERVAL_SECS` of announcement time) so the map cannot grow
    /// unbounded for projects that are never queried.
    pub fn record(&self, project_id: &str, archive_hash: &str, seeder_node_id: &str, seen_at: u64) {
        let mut state = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        if seen_at.saturating_sub(state.last_sweep) > SEED_SWEEP_INTERVAL_SECS {
            sweep_expired(&mut state.seeders, seen_at);
            state.last_sweep = seen_at;
        }
        let seeders = state
            .seeders
            .entry((project_id.to_string(), archive_hash.to_string()))
            .or_default();
        seeders
            .entry(seeder_node_id.to_string())
            .and_modify(|ts| {
                if seen_at > *ts {
                    *ts = seen_at;
                }
            })
            .or_insert(seen_at);
    }

    /// Record an ingested feed op IFF it is a well-formed REMOTE seed
    /// announcement. Returns `true` when it was recorded.
    ///
    /// Three gates, all required (F-4):
    ///  1. `op_type == "SeedAnnounced"` and the payload parses.
    ///  2. Anti-impersonation: `seeder_node_id == author_pubkey`. The
    ///     feed signature authenticates `author_pubkey`; a payload that
    ///     names a DIFFERENT node is forging someone else's seed claim,
    ///     so it is dropped (a node can only announce ITS OWN seeding).
    ///  3. Self-exclusion: `seeder_node_id != my_node_id`. Our own
    ///     announcement, echoed back through the feed, must not be
    ///     counted as a peer — "Toi" is added once at query time from
    ///     the local `keep_online` row.
    pub fn record_announced(
        &self,
        op: &Value,
        author_pubkey: &str,
        my_node_id: &str,
        seen_at: u64,
    ) -> bool {
        if op_type(op) != Some("SeedAnnounced") {
            return false;
        }
        let Some(PublicFeedOperation::SeedAnnounced(p)) = try_parse_op(op) else {
            return false;
        };
        if p.seeder_node_id != author_pubkey {
            return false;
        }
        if p.seeder_node_id == my_node_id {
            return false;
        }
        self.record(&p.project_id, &p.archive_hash, &p.seeder_node_id, seen_at);
        true
    }

    /// Count distinct seeders of `project_id` seen within the TTL at `now`
    /// (unix secs). Lazily evicts expired entries (and drops an emptied bucket)
    /// so churned projects do not leak memory.
    ///
    /// Sprint 75 Phase C (WIRE-2): `archive_hash` selects the granularity.
    ///  - `Some(hash)`: count only the seeders of that EXACT archive version —
    ///    the honest "how many peers can serve the bytes I am about to pull"
    ///    answer, since a seeder of a different version cannot serve this hash.
    ///  - `None`: count the DISTINCT seeders across every archive version of the
    ///    project — the pre-WIRE-2 "any version" semantics, preserved for a
    ///    caller that does not know which hash it is asking about (a seeder that
    ///    announced two versions is counted once).
    pub fn count_recent(&self, project_id: &str, archive_hash: Option<&str>, now: u64) -> usize {
        let mut state = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        let cutoff = now.saturating_sub(SEED_SEEN_TTL_SECS);
        match archive_hash {
            Some(hash) => {
                let key = (project_id.to_string(), hash.to_string());
                let Some(seeders) = state.seeders.get_mut(&key) else {
                    return 0;
                };
                seeders.retain(|_, ts| *ts >= cutoff);
                let count = seeders.len();
                if count == 0 {
                    state.seeders.remove(&key);
                }
                count
            }
            None => {
                let mut distinct: HashSet<String> = HashSet::new();
                let mut emptied: Vec<(String, String)> = Vec::new();
                for (key, seeders) in state.seeders.iter_mut() {
                    if key.0 != project_id {
                        continue;
                    }
                    seeders.retain(|_, ts| *ts >= cutoff);
                    if seeders.is_empty() {
                        emptied.push(key.clone());
                    } else {
                        distinct.extend(seeders.keys().cloned());
                    }
                }
                for key in emptied {
                    state.seeders.remove(&key);
                }
                distinct.len()
            }
        }
    }

    /// The distinct seeder ids of `(project_id, archive_hash)` seen within the
    /// TTL at `now`, sorted for determinism. Same lazy-purge semantics as
    /// [`Self::count_recent`] with `Some(archive_hash)`; used by tests to assert
    /// WHICH seeder was recorded (not just the count), so a mutation that
    /// corrupts the stored `seeder_node_id` is caught instead of passing on the
    /// count alone. Test-only introspection: prod reads the count via
    /// `count_recent`.
    #[cfg(test)]
    pub fn seeders_recent(&self, project_id: &str, archive_hash: &str, now: u64) -> Vec<String> {
        let mut state = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        let cutoff = now.saturating_sub(SEED_SEEN_TTL_SECS);
        let key = (project_id.to_string(), archive_hash.to_string());
        let Some(seeders) = state.seeders.get_mut(&key) else {
            return Vec::new();
        };
        seeders.retain(|_, ts| *ts >= cutoff);
        let mut ids: Vec<String> = seeders.keys().cloned().collect();
        if ids.is_empty() {
            state.seeders.remove(&key);
        }
        ids.sort();
        ids
    }

    /// Number of `(project_id, archive_hash)` buckets currently resident
    /// (test-only — asserts the global sweep actually bounds memory for
    /// never-queried projects).
    #[cfg(test)]
    pub fn bucket_count(&self) -> usize {
        self.inner
            .lock()
            .map(|s| s.seeders.len())
            .unwrap_or_else(|p| p.into_inner().seeders.len())
    }
}

impl std::fmt::Debug for SeedRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let buckets = self
            .inner
            .lock()
            .map(|s| s.seeders.len())
            .unwrap_or_else(|p| p.into_inner().seeders.len());
        f.debug_struct("SeedRegistry")
            .field("buckets", &buckets)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed_op(project_id: &str, seeder: &str, archive: &str) -> Value {
        serde_json::to_value(PublicFeedOperation::SeedAnnounced(
            nexus_coordinator_rs::public_feed::SeedAnnouncedPayload {
                project_id: project_id.to_string(),
                seeder_node_id: seeder.to_string(),
                archive_hash: archive.to_string(),
            },
        ))
        .unwrap()
    }

    #[test]
    fn seed_announced_ingested_increments_count() {
        let reg = SeedRegistry::new();
        let pid = "a".repeat(64);
        let me = "11".repeat(32);
        let peer1 = "22".repeat(32);
        let peer2 = "33".repeat(32);
        let archive = "cc".repeat(32);
        let now = 1_700_000_000u64;

        // A valid REMOTE seed announcement (author == seeder, seeder != me).
        assert!(reg.record_announced(&seed_op(&pid, &peer1, &archive), &peer1, &me, now));
        assert_eq!(reg.count_recent(&pid, Some(&archive), now), 1);
        // WHICH seeder was recorded, not just how many — catches a mutation
        // that stores the wrong seeder_node_id.
        assert_eq!(reg.seeders_recent(&pid, &archive, now), vec![peer1.clone()]);

        // A second distinct peer raises the count.
        assert!(reg.record_announced(&seed_op(&pid, &peer2, &archive), &peer2, &me, now));
        assert_eq!(reg.count_recent(&pid, Some(&archive), now), 2);
        let mut expected = vec![peer1.clone(), peer2.clone()];
        expected.sort();
        assert_eq!(reg.seeders_recent(&pid, &archive, now), expected);

        // Re-announce by an already-known peer refreshes, never double-counts.
        assert!(reg.record_announced(&seed_op(&pid, &peer1, &archive), &peer1, &me, now + 10));
        assert_eq!(reg.count_recent(&pid, Some(&archive), now + 10), 2);

        // Impersonation: payload names a peer that is NOT the entry's signer.
        assert!(!reg.record_announced(&seed_op(&pid, &peer1, &archive), &peer2, &me, now));
        // Self echo: our own announcement is never counted as a peer.
        assert!(!reg.record_announced(&seed_op(&pid, &me, &archive), &me, &me, now));
        // A non-SeedAnnounced op is ignored.
        let other = serde_json::json!({ "op_type": "SourceBecameStale", "project_id": pid });
        assert!(!reg.record_announced(&other, &peer1, &me, now));

        // None of the rejected ops changed the count.
        assert_eq!(reg.count_recent(&pid, Some(&archive), now + 10), 2);
        // The version-agnostic read agrees (one archive version, two seeders).
        assert_eq!(reg.count_recent(&pid, None, now + 10), 2);
    }

    #[test]
    fn seed_count_keyed_by_project_and_hash() {
        // WIRE-2 (Sprint 75 Phase C): two seeders of DISTINCT archive versions
        // of the same project must NOT collapse — a version-specific read counts
        // only the seeders of THAT hash (a seeder of version B cannot serve
        // version A's bytes), while the version-agnostic read sees both.
        let reg = SeedRegistry::new();
        let pid = "a".repeat(64);
        let v1 = "11".repeat(32);
        let v2 = "22".repeat(32);
        let peer_a = "aa".repeat(32);
        let peer_b = "bb".repeat(32);
        let me = "ff".repeat(32);
        let now = 1_700_000_000u64;

        // peer_a seeds version 1, peer_b seeds version 2.
        assert!(reg.record_announced(&seed_op(&pid, &peer_a, &v1), &peer_a, &me, now));
        assert!(reg.record_announced(&seed_op(&pid, &peer_b, &v2), &peer_b, &me, now));

        // Version-specific: exactly the seeder of each hash, never the other.
        assert_eq!(reg.count_recent(&pid, Some(&v1), now), 1);
        assert_eq!(reg.seeders_recent(&pid, &v1, now), vec![peer_a.clone()]);
        assert_eq!(reg.count_recent(&pid, Some(&v2), now), 1);
        assert_eq!(reg.seeders_recent(&pid, &v2, now), vec![peer_b.clone()]);

        // An unknown version of a known project is zero, never the other's count.
        assert_eq!(reg.count_recent(&pid, Some(&"cc".repeat(32)), now), 0);

        // Version-agnostic: the distinct seeders across BOTH versions.
        assert_eq!(reg.count_recent(&pid, None, now), 2);

        // A seeder that announces BOTH versions is counted once version-agnostic.
        assert!(reg.record_announced(&seed_op(&pid, &peer_a, &v2), &peer_a, &me, now));
        assert_eq!(reg.count_recent(&pid, Some(&v2), now), 2); // peer_a + peer_b on v2
        assert_eq!(reg.count_recent(&pid, None, now), 2); // distinct: {peer_a, peer_b}
    }

    #[test]
    fn seed_count_best_effort_ttl_expires() {
        let reg = SeedRegistry::new();
        let pid = "a".repeat(64);
        let archive = "cc".repeat(32);
        let fresh = "22".repeat(32);
        let stale = "33".repeat(32);
        let now = 1_700_000_000u64;

        // One peer seen now, one seen well beyond the TTL.
        reg.record(&pid, &archive, &fresh, now);
        reg.record(&pid, &archive, &stale, now - SEED_SEEN_TTL_SECS - 1);

        // The stale peer is evicted on read; only the fresh one counts.
        assert_eq!(reg.count_recent(&pid, Some(&archive), now), 1);

        // Once every peer expires the bucket is dropped (count 0).
        assert_eq!(
            reg.count_recent(&pid, Some(&archive), now + SEED_SEEN_TTL_SECS + 2),
            0
        );

        // An unknown project is simply zero, never a panic.
        assert_eq!(reg.count_recent(&"f".repeat(64), Some(&archive), now), 0);
    }

    #[test]
    fn seed_registry_global_sweep_bounds_memory() {
        // Projects that are announced but NEVER queried must not accumulate for
        // the process lifetime: a later announcement triggers a global sweep
        // that evicts the now-expired never-queried buckets (C5).
        let reg = SeedRegistry::new();
        let t0 = 1_700_000_000u64;
        let archive = "cc".repeat(32);

        reg.record(&"a".repeat(64), &archive, &"11".repeat(32), t0);
        reg.record(&"b".repeat(64), &archive, &"22".repeat(32), t0);
        assert_eq!(reg.bucket_count(), 2);

        // An announcement far enough in the future to (1) cross the sweep
        // interval and (2) age A and B past the TTL triggers the global sweep.
        let later = t0 + SEED_SEEN_TTL_SECS + SEED_SWEEP_INTERVAL_SECS + 1;
        reg.record(&"c".repeat(64), &archive, &"33".repeat(32), later);

        // A and B were swept; only the fresh project C remains resident.
        assert_eq!(reg.bucket_count(), 1);
        assert_eq!(reg.count_recent(&"c".repeat(64), Some(&archive), later), 1);
    }
}
