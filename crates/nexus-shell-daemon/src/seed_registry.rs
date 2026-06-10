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
/// the local receive clock — no external timer in Phase F scope). Between
/// sweeps, reads (`count_recent`) still lazily purge the QUERIED project, so a
/// queried count is always fresh; the global sweep bounds memory for projects
/// that are announced but NEVER queried (e.g. a headless node that only
/// ingests). One hour keeps at most ~1h of distinct (project, seeder)
/// announcements resident between sweeps.
pub const SEED_SWEEP_INTERVAL_SECS: u64 = 60 * 60;

/// SEED-2 (Sprint 75 Phase D): hard cap on resident `(project_id,
/// archive_hash)` buckets. The TTL sweep already bounds memory over TIME;
/// this bounds it in SPACE against a burst of distinct forged keys arriving
/// faster than the sweep interval. When full, a fresher newcomer evicts the
/// globally stalest bucket; a staler newcomer is dropped. 1024 buckets ×
/// 64 seeders × ~200 bytes ≈ 13 MB worst case — far above pilote-ferme scale.
///
/// Accepted residual (THREAT_MODEL §15 row D posture): a SUSTAINED flood of
/// FRESH forged keys (each costing a feed PoW) can displace honest buckets
/// one by one — the inverse policy (drop newcomers when full) would let an
/// attacker pre-fill instead, so neither side of a bounded best-effort cache
/// can win that trade. Impact is availability-of-the-hint only: the anchor
/// stays first in every provider vector and content-addressing (BLAKE3)
/// remains the truth of reachability; the attack decays after one TTL.
pub const MAX_REGISTRY_BUCKETS: usize = 1024;

/// SEED-2 (Sprint 75 Phase D): hard cap on distinct seeders per bucket. The
/// feed gate already requires `seeder_node_id == author_pubkey` (one Ed25519
/// identity per claimed seeder), but Ed25519 identities are free — a Sybil
/// swarm could otherwise grow one bucket unbounded. The provider vector
/// handed to the multi-provider fetch is capped much lower; counts beyond
/// this are availability noise, not signal.
pub const MAX_SEEDERS_PER_BUCKET: usize = 64;

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
    /// Local receive clock (`now`) of the last global sweep (0 = never
    /// swept). Receive-clock based since SEED-1 — never the announcement's
    /// own (forgeable) timestamp.
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
    /// `project_id` at `seen_at` (unix secs), observed at local clock `now`.
    /// Idempotent per `(project, archive_hash, seeder)`: a repeat announcement
    /// only refreshes the last-seen timestamp, and a stale replayed ts never
    /// overwrites a fresher observation.
    ///
    /// Defenses live IN the registry, not as caller conventions (lesson
    /// P2 guardrail-before-persist):
    ///  - SEED-1 (Sprint 75 Phase D): `seen_at` is clamped to
    ///    `min(seen_at, now)`. A forged far-future feed timestamp would
    ///    otherwise never age past the TTL purge ("fresh forever"); after the
    ///    clamp it expires `SEED_SEEN_TTL_SECS` after the moment WE saw it,
    ///    like every honest announcement.
    ///  - SEED-2: resident size is capped in space, not just time — see
    ///    [`MAX_REGISTRY_BUCKETS`] / [`MAX_SEEDERS_PER_BUCKET`]. When a cap is
    ///    hit, a fresher newcomer evicts the stalest resident; a staler
    ///    newcomer is dropped (deterministic stable tie-break; only the single
    ///    victim key is cloned during the scan).
    ///  - Hex-case normalization: all three keys are stored (and read back)
    ///    lowercase. The feed layer accepts mixed-case hex, so without this a
    ///    single Ed25519 key could sign entries under 2^64 case variants of
    ///    its own pubkey — each passing the `seeder == author` gate — and
    ///    monopolize a bucket's seeder slots, evicting honest seeders from
    ///    the multi-provider dial set (review Phase D, security dim).
    ///
    /// Runs a throttled global sweep (every `SEED_SWEEP_INTERVAL_SECS` of
    /// receive-clock time) so the map cannot grow unbounded for projects that
    /// are never queried.
    pub fn record(
        &self,
        project_id: &str,
        archive_hash: &str,
        seeder_node_id: &str,
        seen_at: u64,
        now: u64,
    ) {
        // SEED-1: a future-dated announcement is recorded as seen NOW.
        let seen_at = seen_at.min(now);
        // One identity = one key, whatever hex case the announcement used.
        let project_id = project_id.to_ascii_lowercase();
        let archive_hash = archive_hash.to_ascii_lowercase();
        let seeder_node_id = seeder_node_id.to_ascii_lowercase();
        let mut state = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        if now.saturating_sub(state.last_sweep) > SEED_SWEEP_INTERVAL_SECS {
            sweep_expired(&mut state.seeders, now);
            state.last_sweep = now;
        }
        let key = (project_id, archive_hash);
        // SEED-2 bucket-count cap: applies only when inserting a NEW key.
        if !state.seeders.contains_key(&key) && state.seeders.len() >= MAX_REGISTRY_BUCKETS {
            // Cheap relief first: drop expired entries before evicting live ones.
            sweep_expired(&mut state.seeders, now);
            state.last_sweep = now;
            if state.seeders.len() >= MAX_REGISTRY_BUCKETS {
                // Evict the globally stalest bucket (smallest freshest-seeder
                // ts) IFF the newcomer is fresher than it; otherwise drop the
                // newcomer — a flood of stale keys cannot displace live data.
                // (A flood of FRESH keys can — accepted residual, see the
                // MAX_REGISTRY_BUCKETS doc.) Scan over references; clone only
                // the victim key.
                let stalest = state
                    .seeders
                    .iter()
                    .map(|(k, peers)| (k, peers.values().copied().max().unwrap_or(0)))
                    .min_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(b.0)))
                    .map(|(k, ts)| (k.clone(), ts));
                match stalest {
                    Some((victim, victim_ts)) if seen_at > victim_ts => {
                        state.seeders.remove(&victim);
                    }
                    _ => return,
                }
            }
        }
        let seeders = state.seeders.entry(key).or_default();
        // SEED-2 per-bucket cap: applies only when inserting a NEW seeder.
        if !seeders.contains_key(&seeder_node_id) && seeders.len() >= MAX_SEEDERS_PER_BUCKET {
            let stalest = seeders
                .iter()
                .map(|(id, ts)| (id, *ts))
                .min_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(b.0)))
                .map(|(id, ts)| (id.clone(), ts));
            match stalest {
                Some((victim, victim_ts)) if seen_at > victim_ts => {
                    seeders.remove(&victim);
                }
                _ => return,
            }
        }
        seeders
            .entry(seeder_node_id)
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
        now: u64,
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
        // Case-insensitive self check: our own node id is lowercase, but a
        // (hypothetical) case-variant echo of our identity must still never
        // count as a peer. Strict equality stays on the author gate above —
        // stricter = rejects, never admits.
        if p.seeder_node_id.eq_ignore_ascii_case(my_node_id) {
            return false;
        }
        self.record(
            &p.project_id,
            &p.archive_hash,
            &p.seeder_node_id,
            seen_at,
            now,
        );
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
        // Reads normalize like writes (one identity = one lowercase key).
        let project_id = project_id.to_ascii_lowercase();
        let archive_hash = archive_hash.map(|h| h.to_ascii_lowercase());
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
    /// [`Self::count_recent`] with `Some(archive_hash)`.
    ///
    /// Promoted from test-only to prod in Sprint 75 Phase D (carry PULL-2):
    /// the multi-provider pull builds its ordered provider vector from the
    /// anchor node_id followed by these seeder ids (Q5). The IDs are only ever
    /// used as DIAL candidates — content-addressing (BLAKE3) keeps a forged
    /// announcement from serving wrong bytes, so a lying entry costs one
    /// failed dial attempt, never integrity.
    pub fn seeders_recent(&self, project_id: &str, archive_hash: &str, now: u64) -> Vec<String> {
        let mut state = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        let cutoff = now.saturating_sub(SEED_SEEN_TTL_SECS);
        // Reads normalize like writes (one identity = one lowercase key).
        let key = (
            project_id.to_ascii_lowercase(),
            archive_hash.to_ascii_lowercase(),
        );
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
        assert!(reg.record_announced(&seed_op(&pid, &peer1, &archive), &peer1, &me, now, now));
        assert_eq!(reg.count_recent(&pid, Some(&archive), now), 1);
        // WHICH seeder was recorded, not just how many — catches a mutation
        // that stores the wrong seeder_node_id.
        assert_eq!(reg.seeders_recent(&pid, &archive, now), vec![peer1.clone()]);

        // A second distinct peer raises the count.
        assert!(reg.record_announced(&seed_op(&pid, &peer2, &archive), &peer2, &me, now, now));
        assert_eq!(reg.count_recent(&pid, Some(&archive), now), 2);
        let mut expected = vec![peer1.clone(), peer2.clone()];
        expected.sort();
        assert_eq!(reg.seeders_recent(&pid, &archive, now), expected);

        // Re-announce by an already-known peer refreshes, never double-counts.
        assert!(reg.record_announced(
            &seed_op(&pid, &peer1, &archive),
            &peer1,
            &me,
            now + 10,
            now + 10
        ));
        assert_eq!(reg.count_recent(&pid, Some(&archive), now + 10), 2);

        // Impersonation: payload names a peer that is NOT the entry's signer.
        assert!(!reg.record_announced(&seed_op(&pid, &peer1, &archive), &peer2, &me, now, now));
        // Self echo: our own announcement is never counted as a peer.
        assert!(!reg.record_announced(&seed_op(&pid, &me, &archive), &me, &me, now, now));
        // A non-SeedAnnounced op is ignored.
        let other = serde_json::json!({ "op_type": "SourceBecameStale", "project_id": pid });
        assert!(!reg.record_announced(&other, &peer1, &me, now, now));

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
        assert!(reg.record_announced(&seed_op(&pid, &peer_a, &v1), &peer_a, &me, now, now));
        assert!(reg.record_announced(&seed_op(&pid, &peer_b, &v2), &peer_b, &me, now, now));

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
        assert!(reg.record_announced(&seed_op(&pid, &peer_a, &v2), &peer_a, &me, now, now));
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
        reg.record(&pid, &archive, &fresh, now, now);
        reg.record(&pid, &archive, &stale, now - SEED_SEEN_TTL_SECS - 1, now);

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
    fn seed_registry_clamps_future_ts() {
        // SEED-1 (Sprint 75 Phase D): a forged far-future feed timestamp must
        // not buy immortality in the registry. The stored ts is clamped to the
        // local receive clock, so the entry expires SEED_SEEN_TTL_SECS after
        // WE saw it — exactly like an honest announcement.
        let reg = SeedRegistry::new();
        let pid = "a".repeat(64);
        let archive = "cc".repeat(32);
        let peer = "22".repeat(32);
        let me = "11".repeat(32);
        let now = 1_700_000_000u64;
        let forged_future = now + 10 * 365 * 24 * 60 * 60; // ten years ahead

        // Through the prod ingest gate (the path a hostile feed entry takes).
        assert!(reg.record_announced(
            &seed_op(&pid, &peer, &archive),
            &peer,
            &me,
            forged_future,
            now
        ));
        // Counted now (it WAS just seen)...
        assert_eq!(reg.count_recent(&pid, Some(&archive), now), 1);
        // ...but expired one TTL after the local clock, NOT ten years later.
        assert_eq!(
            reg.count_recent(&pid, Some(&archive), now + SEED_SEEN_TTL_SECS + 1),
            0,
            "a clamped future-ts entry must age out on the local clock"
        );

        // An honest past ts is stored as-is (min(seen_at, now) = seen_at).
        reg.record(&pid, &archive, &peer, now - 100, now);
        assert_eq!(reg.count_recent(&pid, Some(&archive), now), 1);
        assert_eq!(
            reg.count_recent(&pid, Some(&archive), now - 100 + SEED_SEEN_TTL_SECS + 1),
            0
        );
    }

    #[test]
    fn seed_registry_normalizes_hex_case() {
        // Security (review Phase D): the feed layer accepts mixed-case hex,
        // so one Ed25519 key signing under case variants of its own pubkey
        // must collapse to ONE seeder — not 2^64 distinct identities able to
        // monopolize a bucket's capped slots. Writes and reads normalize.
        let reg = SeedRegistry::new();
        let now = 1_700_000_000u64;
        let pid_lower = "ab".repeat(32);
        let pid_upper = pid_lower.to_ascii_uppercase();
        let hash_lower = "cd".repeat(32);
        let hash_upper = hash_lower.to_ascii_uppercase();
        let peer_lower = "ef".repeat(32);
        let peer_mixed = "Ef".repeat(32);

        // Same identity announced under different case shapes of every key.
        reg.record(&pid_lower, &hash_lower, &peer_lower, now, now);
        reg.record(&pid_upper, &hash_upper, &peer_mixed, now + 1, now + 1);

        // One bucket, one seeder — never two.
        assert_eq!(reg.bucket_count(), 1);
        assert_eq!(reg.count_recent(&pid_lower, Some(&hash_lower), now + 1), 1);
        // Reads normalize too: an uppercase query hits the same bucket.
        assert_eq!(reg.count_recent(&pid_upper, Some(&hash_upper), now + 1), 1);
        // The stored id is the lowercase canonical form (what the provider
        // vector parses into an EndpointId).
        assert_eq!(
            reg.seeders_recent(&pid_upper, &hash_upper, now + 1),
            vec![peer_lower.clone()]
        );

        // Self-echo stays excluded even under a case-variant payload.
        let me = "11".repeat(32);
        let me_upper = me.to_ascii_uppercase();
        assert!(!reg.record_announced(
            &seed_op(&pid_lower, &me_upper, &hash_lower),
            &me_upper,
            &me,
            now,
            now
        ));
    }

    #[test]
    fn seed_registry_size_bounded() {
        // SEED-2 (Sprint 75 Phase D): resident size is capped in SPACE, not
        // just time. Fresh keys within the TTL cannot grow the map past
        // MAX_REGISTRY_BUCKETS, and one bucket cannot grow past
        // MAX_SEEDERS_PER_BUCKET distinct seeders.
        let reg = SeedRegistry::new();
        let now = 1_700_000_000u64;
        let archive = "cc".repeat(32);

        // (a) Bucket-count cap: MAX + 10 distinct fresh keys, all within the
        // TTL so the sweep cannot help — only the eviction policy bounds it.
        for i in 0..(MAX_REGISTRY_BUCKETS + 10) {
            let pid = format!("{i:064x}");
            reg.record(
                &pid,
                &archive,
                &"22".repeat(32),
                now + i as u64,
                now + i as u64,
            );
        }
        // Exactly MAX resident (1-for-1 eviction), never fewer — an
        // over-aggressive eviction (off-by-one, multi-evict) would show here.
        assert_eq!(reg.bucket_count(), MAX_REGISTRY_BUCKETS);
        let t_end = now + (MAX_REGISTRY_BUCKETS + 10) as u64;
        // Victim choice pinned (insertion ts order): the 10 stalest keys were
        // evicted — key 9 gone, key 10 the oldest survivor, freshest present.
        assert_eq!(
            reg.count_recent(&format!("{:064x}", 9), Some(&archive), t_end),
            0,
            "the stalest resident must be the eviction victim"
        );
        assert_eq!(
            reg.count_recent(&format!("{:064x}", 10), Some(&archive), t_end),
            1,
            "the oldest non-victim bucket must survive"
        );
        let last = format!("{:064x}", MAX_REGISTRY_BUCKETS + 9);
        assert_eq!(reg.count_recent(&last, Some(&archive), t_end), 1);
        // Anti-displacement: a STALER-than-every-resident newcomer is dropped.
        let stale_pid = "f".repeat(64);
        reg.record(&stale_pid, &archive, &"22".repeat(32), now - 1, t_end);
        assert_eq!(
            reg.count_recent(&stale_pid, Some(&archive), t_end),
            0,
            "a staler-than-every-resident newcomer must be dropped when full"
        );

        // (b) Per-bucket cap: MAX + 10 distinct seeders on one key.
        let reg2 = SeedRegistry::new();
        let pid = "a".repeat(64);
        for i in 0..(MAX_SEEDERS_PER_BUCKET + 10) {
            let seeder = format!("{i:064x}");
            reg2.record(&pid, &archive, &seeder, now + i as u64, now + i as u64);
        }
        assert_eq!(
            reg2.count_recent(&pid, Some(&archive), t_end),
            MAX_SEEDERS_PER_BUCKET,
            "per-bucket seeder count must be exactly the cap after 1-for-1 eviction"
        );
        let resident = reg2.seeders_recent(&pid, &archive, t_end);
        // Victim choice pinned: the stalest seeders (0..=9) were evicted, the
        // oldest non-victim (10) and the freshest both survive.
        assert!(
            !resident.contains(&format!("{:064x}", 9)),
            "the stalest seeder must be the per-bucket eviction victim"
        );
        assert!(
            resident.contains(&format!("{:064x}", 10)),
            "the oldest non-victim seeder must survive"
        );
        let freshest = format!("{:064x}", MAX_SEEDERS_PER_BUCKET + 9);
        assert!(
            resident.contains(&freshest),
            "the freshest seeder must survive the per-bucket eviction"
        );
        // Anti-displacement on the per-bucket cap too: a staler-than-every-
        // resident newcomer bounces off a full bucket (the `_ => return`
        // branch — without it a stale Sybil flood could displace live
        // seeders, the exact defense SEED-2 claims).
        let stale_seeder = "f".repeat(64);
        reg2.record(&pid, &archive, &stale_seeder, now - 1, t_end);
        assert!(
            !reg2
                .seeders_recent(&pid, &archive, t_end)
                .contains(&stale_seeder),
            "a staler-than-every-resident seeder must be dropped on a full bucket"
        );
        assert_eq!(
            reg2.count_recent(&pid, Some(&archive), t_end),
            MAX_SEEDERS_PER_BUCKET
        );
    }

    #[test]
    fn seed_registry_global_sweep_bounds_memory() {
        // Projects that are announced but NEVER queried must not accumulate for
        // the process lifetime: a later announcement triggers a global sweep
        // that evicts the now-expired never-queried buckets (C5).
        let reg = SeedRegistry::new();
        let t0 = 1_700_000_000u64;
        let archive = "cc".repeat(32);

        reg.record(&"a".repeat(64), &archive, &"11".repeat(32), t0, t0);
        reg.record(&"b".repeat(64), &archive, &"22".repeat(32), t0, t0);
        assert_eq!(reg.bucket_count(), 2);

        // An announcement far enough in the future to (1) cross the sweep
        // interval and (2) age A and B past the TTL triggers the global sweep.
        let later = t0 + SEED_SEEN_TTL_SECS + SEED_SWEEP_INTERVAL_SECS + 1;
        reg.record(&"c".repeat(64), &archive, &"33".repeat(32), later, later);

        // A and B were swept; only the fresh project C remains resident.
        assert_eq!(reg.bucket_count(), 1);
        assert_eq!(reg.count_recent(&"c".repeat(64), Some(&archive), later), 1);
    }
}
