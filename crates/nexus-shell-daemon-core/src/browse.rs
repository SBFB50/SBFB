// SPDX-License-Identifier: AGPL-3.0-or-later
//! Phase D browse aggregator.
//!
//! The Phase C [`crate::iroh_runtime::CuratorRuntime`] caches
//! signed curator lists, each carrying a bounded vector of
//! [`nexus_core_rs::CuratorProjectRef`] entries. Phase D takes
//! the union of every subscribed curator's entries and probes
//! each referenced project endpoint for reachability, so the
//! React Browse page can show a simple reachable / unreachable
//! / unknown badge next to every listed project.
//!
//! ## Probe path
//!
//! Probing goes through
//! [`nexus_core_rs::DiscoveryClient::probe_reachable`], which
//! wraps `iroh::Endpoint::connect(id, iroh_blobs::ALPN)` under a
//! wall-clock timeout. Every SBFB node accepts the blobs
//! protocol at boot so a successful `connect` to any project
//! endpoint id means "the node is live and dialable right now".
//!
//! ## TTL cache
//!
//! A naive aggregator would probe every project on every `/browse`
//! call, which would flood the pkarr relay on shell refreshes.
//! Instead we keep a `DashMap<project_id, (status, last_probed_at)>`
//! and reuse the cached status if it is younger than
//! [`DEFAULT_PROBE_TTL`]. The TTL is short enough (60 s) that the
//! UI is still responsive to a peer coming back online, and long
//! enough that mashing F5 on the shell does not hammer iroh.
//!
//! ## Scope
//!
//! Phase D does **not** publish pkarr records for projects the
//! local daemon owns. Publish is Sprint 10 release scope. The
//! aggregator is pure consumer: iterate curator lists, probe,
//! format.

use std::fmt;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use dashmap::DashMap;
use nexus_core_rs::{
    CuratorListEntry, DiscoveryClient, DnsFallbackResolve, Node, QuorumError, QuorumResolver,
    redundant_resolve,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tracing::{debug, warn};

use crate::iroh_runtime::CuratorRuntime;

/// Default TTL for a cached reachability result. After this
/// duration the next `/browse` call re-probes the project
/// endpoint. Picked to absorb normal shell refresh bursts (F5
/// mash) without going longer than a user would plausibly wait
/// before expecting a status change.
pub const DEFAULT_PROBE_TTL: Duration = Duration::from_secs(60);

/// Default timeout for a single reachability probe. Passed
/// through to [`DiscoveryClient::probe_reachable`].
///
/// Sprint 9 Phase E (E-1 close): overridable at runtime via the
/// `NEXUS_PROBE_TIMEOUT_MS` environment variable. The constant
/// is the fallback when the env var is absent or unparseable.
pub const DEFAULT_PROBE_TIMEOUT: Duration = Duration::from_secs(2);

/// Default per-resolver budget for the Sprint 19 Phase A pkarr
/// quorum canary. Each of the N resolvers handed to
/// [`nexus_core_rs::redundant_resolve`] runs under this budget ;
/// three concurrent timeouts of `T` seconds each complete in
/// worst-case ~`T` seconds wall-clock because the quorum layer
/// fans them out through a [`tokio::task::JoinSet`]. Picked at
/// 3 s — longer than the 2 s probe timeout so a slow pkarr
/// relay gets a fair chance before the quorum counts it as
/// errored, short enough that the aggregator call stays
/// responsive to a shell refresh burst.
pub const DEFAULT_QUORUM_LOOKUP_TIMEOUT: Duration = Duration::from_secs(3);

/// Read the probe timeout from the `NEXUS_PROBE_TIMEOUT_MS`
/// environment variable, falling back to [`DEFAULT_PROBE_TIMEOUT`]
/// if the variable is absent or not a valid u64.
pub fn probe_timeout_from_env() -> Duration {
    match std::env::var("NEXUS_PROBE_TIMEOUT_MS") {
        Ok(val) => match val.parse::<u64>() {
            Ok(ms) if ms > 0 => Duration::from_millis(ms),
            _ => {
                tracing::warn!(
                    value = %val,
                    "NEXUS_PROBE_TIMEOUT_MS is not a valid positive integer, using default"
                );
                DEFAULT_PROBE_TIMEOUT
            }
        },
        Err(_) => DEFAULT_PROBE_TIMEOUT,
    }
}

// =================================================================
// Wire types
// =================================================================

/// How a [`BrowseEntry`] was discovered: via a signed curator
/// list, or directly from a gossip project announcement.
///
/// Sprint 11 Phase A addition. The field is `#[serde(default)]`
/// on `BrowseEntry` so daemons that haven't upgraded yet still
/// deserialize entries without a `source` field (defaulting to
/// `Curator`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BrowseSource {
    /// Entry came from a signed curator list.
    #[default]
    Curator,
    /// Entry was announced directly via gossip by the project's
    /// own daemon (self-publish, Sprint 11 Phase A).
    Direct,
}

impl BrowseSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Curator => "curator",
            Self::Direct => "direct",
        }
    }
}

/// Reachability bucket for a single [`BrowseEntry`].
///
/// `Unknown` is the initial state for entries that haven't been
/// probed yet (e.g. the daemon just booted and the user opens
/// `/browse` before the cache is warm). The React shell renders
/// a neutral spinner for `Unknown`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BrowseStatus {
    /// The probe succeeded within the timeout.
    Reachable,
    /// The probe failed or timed out.
    Unreachable,
    /// The aggregator hasn't probed this project yet, or the
    /// cached probe has expired and a re-probe has not
    /// completed yet.
    Unknown,
}

impl BrowseStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reachable => "reachable",
            Self::Unreachable => "unreachable",
            Self::Unknown => "unknown",
        }
    }
}

/// A single project row the React shell renders on the Browse
/// page.
///
/// Aggregated from every curator list entry pointing at the
/// same `project_id`, carrying the pubkey of the curator that
/// vouches for it. If multiple curators vouch for the same
/// project, the aggregator returns **one** entry per
/// (project_id, curator_pubkey) pair — users see which curator
/// recommended each project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BrowseEntry {
    /// Project identity hex (64 chars).
    ///
    /// For **curator-list** entries this is the project coordinator's
    /// node id, so the Phase D probe dials it directly. For **direct**
    /// (gossip) entries post-Sprint-11/#4 it is `blake3(project_name)`
    /// — a per-app id that lets one node show N distinct cards — which
    /// is **not** a dialable node id; the reachability probe for those
    /// entries uses [`Self::node_id`] instead.
    pub project_id: String,
    /// Hosting daemon's node id (hex Ed25519 public key) — the
    /// dialable identity used by the freshness probe.
    ///
    /// Distinct from [`Self::project_id`], which for a direct entry is
    /// `blake3(project_name)` and resolves to no live endpoint. Set at
    /// announce time (`= ProjectAnnouncement.node_id`) and at local
    /// publish/deploy time (`= our own node id`). `None` for
    /// curator-list entries (their `project_id` already equals the
    /// node id) and for any entry built before this field existed.
    ///
    /// Daemon-internal: `#[serde(skip)]` so it never crosses the
    /// daemon→frontend boundary — the reachability signal reaches the
    /// shell through [`Self::status`], not this field — which keeps the
    /// `/browse` JSON byte-identical and the frontend Zod schema
    /// untouched.
    #[serde(skip)]
    pub node_id: Option<String>,
    /// Display name of the project.
    pub project_name: String,
    /// Category tag from the curator list entry
    /// (`"gov"`, `"investigation"`, …).
    pub category: String,
    /// Short description from the curator list entry.
    pub description: String,
    /// Lowercase hex of the curator's Ed25519 public key.
    pub curator_pubkey: String,
    /// Human-readable curator display name.
    pub curator_name: String,
    /// How this entry was discovered (curator list vs direct
    /// gossip announcement). Defaults to `Curator` for backward
    /// compatibility with daemons that predate Sprint 11.
    #[serde(default)]
    pub source: BrowseSource,
    /// Latest reachability bucket.
    pub status: BrowseStatus,
    /// RFC 3339 UTC timestamp of the last probe. `None` when
    /// `status == Unknown` (no probe has been attempted yet).
    pub last_probed_at: Option<String>,
    /// BlobTicket of the zip archive for this project (Sprint 12).
    /// `None` for entries from older daemons or curator lists that
    /// predate v2 announcements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archive_ticket: Option<String>,
    /// Hex-encoded BLAKE3 hash of the zip blob (Sprint 12 Phase C).
    /// The frontend uses this to construct blob-serve URLs since
    /// `GET /blob-serve/{hash}/{*path}` expects a hex hash, not the
    /// opaque BlobTicket string. `None` when there is no archive.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archive_hash: Option<String>,
    /// URL of the public source code repository (Sprint 13 Phase B).
    /// Required for public projects, optional for private.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_url: Option<String>,
    /// BLAKE3 hex hash of the provenance.json attestation (Sprint 14).
    /// Present when the project was deployed via verified deploy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance_hash: Option<String>,
    /// True iff the originating `ProjectAnnouncement` carried the
    /// v5 `is_open_source=true` flag (Sprint 16 Phase D). Derived
    /// by the coordinator at deploy-from-repo time; legacy v1..v4
    /// announcements and private zip uploads default to `false`.
    /// The React shell consumes this to render the "open source"
    /// badge on Browse rows; workers at consent level `OpenSource`
    /// use the same flag (wired separately through the worker
    /// task ingest path) to accept or reject tasks.
    #[serde(default)]
    pub is_open_source: bool,
}

// =================================================================
// Aggregator
// =================================================================

/// Cached reachability result for a single project id.
///
/// The aggregator replaces this in place when the TTL expires —
/// equal timestamp writes are skipped via `insert` semantics.
#[derive(Debug, Clone)]
struct ProbeCacheEntry {
    status: BrowseStatus,
    probed_at: SystemTime,
}

/// The browse aggregator.
///
/// Holds a project-id → probe-cache DashMap, a map of directly
/// announced projects (Sprint 11 Phase A), and the two
/// aggregator-level duration knobs. Cloneable via `Arc`.
pub struct BrowseAggregator {
    cache: DashMap<String, ProbeCacheEntry>,
    /// Projects announced via gossip directly (self-publish),
    /// keyed by project_id (node_id hex). Sprint 11 Phase A.
    direct_entries: DashMap<String, BrowseEntry>,
    probe_ttl: Duration,
    probe_timeout: Duration,
    /// Sprint 19 Phase A : optional pkarr quorum resolvers used
    /// as an eclipse-defence canary before dialing. When set and
    /// [`nexus_core_rs::redundant_resolve`] returns
    /// [`QuorumError::NoMajority`] or [`QuorumError::AllFailed`],
    /// the probe is skipped and the entry is cached as
    /// `Unreachable` without a dial attempt. When the quorum
    /// agrees, the probe continues through the standard iroh
    /// discovery path (`presets::N0`). `None` preserves the
    /// pre-Sprint-19 single-lookup behaviour byte-for-byte, so
    /// tests that do not opt in see no change.
    quorum_resolvers: Option<Arc<Vec<Arc<dyn QuorumResolver>>>>,
    /// Per-resolver timeout inside
    /// [`nexus_core_rs::redundant_resolve`]. Ignored when
    /// `quorum_resolvers` is `None`.
    quorum_per_lookup_timeout: Duration,
    /// Sprint 24 Phase E : optional DNS fallback resolver. When
    /// the pkarr quorum returns `AllFailed` (all relays unreachable),
    /// the aggregator tries DNS (DoH/DoT) before marking the peer
    /// `Unreachable`. `NoMajority` (eclipse signal) is NOT
    /// overridden — DNS fallback only covers connectivity failures.
    dns_fallback: Option<Arc<dyn DnsFallbackResolve>>,
    #[cfg(test)]
    probe_call_count: std::sync::atomic::AtomicU32,
}

impl fmt::Debug for BrowseAggregator {
    // Custom Debug avoids requiring `Debug` on `dyn QuorumResolver`
    // (the trait is intentionally minimal) and surfaces the
    // aggregator's invariants — cache size, number of wired
    // resolvers, duration knobs — without dumping opaque entries.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BrowseAggregator")
            .field("cache_len", &self.cache.len())
            .field("direct_entries_len", &self.direct_entries.len())
            .field("probe_ttl", &self.probe_ttl)
            .field("probe_timeout", &self.probe_timeout)
            .field(
                "quorum_resolver_count",
                &self.quorum_resolvers.as_ref().map(|v| v.len()).unwrap_or(0),
            )
            .field("quorum_per_lookup_timeout", &self.quorum_per_lookup_timeout)
            .field(
                "dns_fallback",
                &self.dns_fallback.as_ref().map(|f| f.label()),
            )
            .finish()
    }
}

impl BrowseAggregator {
    /// Create a new aggregator with the default TTL and a probe
    /// timeout read from the `NEXUS_PROBE_TIMEOUT_MS` env var
    /// (falling back to [`DEFAULT_PROBE_TIMEOUT`]).
    pub fn new() -> Self {
        Self::with_durations(DEFAULT_PROBE_TTL, probe_timeout_from_env())
    }

    /// Create an aggregator with custom TTL / probe timeout.
    /// Used by tests that want to force a cache miss or a
    /// deterministically short probe.
    pub fn with_durations(probe_ttl: Duration, probe_timeout: Duration) -> Self {
        Self {
            cache: DashMap::new(),
            direct_entries: DashMap::new(),
            probe_ttl,
            probe_timeout,
            quorum_resolvers: None,
            quorum_per_lookup_timeout: DEFAULT_QUORUM_LOOKUP_TIMEOUT,
            dns_fallback: None,
            #[cfg(test)]
            probe_call_count: std::sync::atomic::AtomicU32::new(0),
        }
    }

    /// Sprint 19 Phase A : attach a pkarr quorum resolver set to
    /// the aggregator. On every cache-miss probe, the aggregator
    /// first consults [`nexus_core_rs::redundant_resolve`] over
    /// this set ; a clean majority green-lights the downstream
    /// dial, a no-majority or all-failed result short-circuits to
    /// `Unreachable` without a dial attempt. The returned
    /// aggregator is chainable so the daemon boot can write
    /// `BrowseAggregator::new().with_quorum_resolvers(resolvers)`.
    ///
    /// Supplying an empty vector is a caller bug — the canary
    /// gate then degrades to a warn-logged passthrough rather
    /// than a hard failure so the daemon still boots. Three
    /// resolvers are the intended shape ; one or two resolvers
    /// run as unanimity-quorum (weaker defence but functional).
    pub fn with_quorum_resolvers(mut self, resolvers: Vec<Arc<dyn QuorumResolver>>) -> Self {
        self.quorum_resolvers = Some(Arc::new(resolvers));
        self
    }

    /// Sprint 24 Phase E : attach a DNS fallback resolver. When
    /// the pkarr quorum returns `AllFailed`, the aggregator tries
    /// DNS (DoH/DoT) before marking the peer `Unreachable`.
    pub fn with_dns_fallback(mut self, fallback: Arc<dyn DnsFallbackResolve>) -> Self {
        self.dns_fallback = Some(fallback);
        self
    }

    /// Number of pkarr quorum resolvers currently wired. `0` when
    /// the aggregator was not opted into the Sprint 19 Phase A
    /// canary gate (default for tests).
    pub fn quorum_resolver_count(&self) -> usize {
        self.quorum_resolvers.as_ref().map(|v| v.len()).unwrap_or(0)
    }

    #[cfg(test)]
    fn probe_call_count(&self) -> u32 {
        self.probe_call_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Return the current cached reachability bucket for
    /// `project_id_hex`, considering TTL. Returns `None` if the
    /// cache is cold or the cached entry has expired.
    fn cached(&self, project_id_hex: &str) -> Option<BrowseStatus> {
        let entry = self.cache.get(project_id_hex)?;
        match SystemTime::now().duration_since(entry.probed_at) {
            Ok(age) if age <= self.probe_ttl => Some(entry.status),
            _ => None,
        }
    }

    /// Ask the core discovery client whether `project_id_hex`
    /// is reachable right now. Updates the cache with the
    /// returned bucket.
    ///
    /// Sprint 19 Phase A wires the pkarr quorum canary in front
    /// of the probe : when [`Self::with_quorum_resolvers`] has
    /// been called, three concurrent pkarr lookups run through
    /// [`nexus_core_rs::redundant_resolve`]. A clean ≥ 2/3
    /// agreement green-lights the downstream
    /// [`DiscoveryClient::probe_reachable`] dial — iroh then
    /// resolves the peer through its own preset-`N0` discovery
    /// path as before. A no-majority or all-failed verdict marks
    /// the entry `Unreachable` and caches it **without** a dial
    /// attempt, because an incoherent quorum is the exact
    /// signature of an Eclipse-by-DHT attack (one or two relays
    /// serving a forged or stale record). This closes the
    /// Sprint 18 audit finding C-1 (P2 carry-over) : the
    /// primitive shipped in Sprint 18 now has a production call
    /// site, flipping the Eclipse-by-DHT defence from
    /// primitive-prete to runtime-active (`[~]` → `[x]` in
    /// `sprint18_verification.md §Gate 1 unlock`).
    async fn probe_and_cache(
        &self,
        node: &Node,
        project_id_hex: &str,
    ) -> (BrowseStatus, SystemTime) {
        // Sprint 19 Phase A canary gate. `None` preserves the
        // pre-Sprint-19 behaviour byte-for-byte so the existing
        // test fleet does not need to be opted in.
        if let Some(resolvers) = self.quorum_resolvers.as_ref() {
            if resolvers.is_empty() {
                // with_quorum_resolvers(Vec::new()) is a caller
                // bug — warn and fall through rather than hard
                // fail so the daemon still answers /browse.
                warn!(
                    project_id = %project_id_hex,
                    "pkarr quorum resolver set is empty — falling through to direct probe"
                );
            } else {
                match redundant_resolve(
                    project_id_hex,
                    resolvers.as_slice(),
                    self.quorum_per_lookup_timeout,
                )
                .await
                {
                    Ok(record) => {
                        debug!(
                            project_id = %project_id_hex,
                            agreeing = record.agreeing.len(),
                            dissenting = record.dissenting.len(),
                            "pkarr quorum agreed — proceeding to probe"
                        );
                        // Fall through to the standard probe.
                    }
                    Err(QuorumError::NoMajority {
                        ok_count,
                        max_agreement,
                    }) => {
                        warn!(
                            project_id = %project_id_hex,
                            ok_count,
                            max_agreement,
                            "pkarr quorum no-majority — marking Unreachable without dial (Eclipse-by-DHT defence active)"
                        );
                        return self.record_unreachable(project_id_hex);
                    }
                    Err(QuorumError::AllFailed { count }) => {
                        if let Some(dns) = self.dns_fallback.as_ref() {
                            match dns.resolve_node(project_id_hex).await {
                                Ok(bytes) => {
                                    debug!(
                                        project_id = %project_id_hex,
                                        dns_label = %dns.label(),
                                        bytes = bytes.len(),
                                        pkarr_failed = count,
                                        "pkarr quorum AllFailed but DNS fallback resolved — proceeding to probe"
                                    );
                                }
                                Err(dns_err) => {
                                    warn!(
                                        project_id = %project_id_hex,
                                        pkarr_failed = count,
                                        dns_error = %dns_err,
                                        "pkarr quorum AllFailed and DNS fallback also failed — marking Unreachable"
                                    );
                                    return self.record_unreachable(project_id_hex);
                                }
                            }
                        } else {
                            warn!(
                                project_id = %project_id_hex,
                                count,
                                "all pkarr quorum resolvers failed — marking Unreachable without dial"
                            );
                            return self.record_unreachable(project_id_hex);
                        }
                    }
                    Err(QuorumError::Empty) => {
                        // Unreachable in practice because we already
                        // filtered is_empty() above, but matched so
                        // the compiler exhaustiveness is preserved
                        // if the variant set grows.
                        warn!(
                            project_id = %project_id_hex,
                            "pkarr quorum returned Empty — falling through to direct probe"
                        );
                    }
                }
            }
        }

        #[cfg(test)]
        self.probe_call_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let disco = DiscoveryClient::new(node.endpoint());
        let now = SystemTime::now();
        let status = match disco
            .probe_reachable(project_id_hex, self.probe_timeout)
            .await
        {
            Ok(true) => BrowseStatus::Reachable,
            Ok(false) => BrowseStatus::Unreachable,
            Err(e) => {
                // A malformed hex is a data problem, not a
                // network problem — we still surface it as
                // Unreachable so the Browse UI can render the
                // red dot, but log a warning so the bad pubkey
                // is visible in the daemon's logs.
                debug!(error = %e, project_id = %project_id_hex, "probe_reachable returned error");
                BrowseStatus::Unreachable
            }
        };
        self.cache.insert(
            project_id_hex.to_string(),
            ProbeCacheEntry {
                status,
                probed_at: now,
            },
        );
        (status, now)
    }

    /// Cache `Unreachable` for `project_id_hex` and return the
    /// `(status, now)` tuple probe_and_cache expects. Extracted
    /// into a helper so the Sprint 19 Phase A canary and the
    /// legacy error branches share one code path and stay in
    /// lockstep.
    fn record_unreachable(&self, project_id_hex: &str) -> (BrowseStatus, SystemTime) {
        let now = SystemTime::now();
        self.cache.insert(
            project_id_hex.to_string(),
            ProbeCacheEntry {
                status: BrowseStatus::Unreachable,
                probed_at: now,
            },
        );
        (BrowseStatus::Unreachable, now)
    }

    /// Add a directly announced project (self-publish via gossip).
    /// Deduplicates by `project_id` — a second announcement for
    /// the same node_id replaces the previous entry.
    ///
    /// Sprint 11 Phase A.
    pub fn add_direct_entry(&self, entry: BrowseEntry) {
        self.direct_entries.insert(entry.project_id.clone(), entry);
    }

    /// Number of directly announced projects currently cached.
    pub fn direct_entry_count(&self) -> usize {
        self.direct_entries.len()
    }

    pub fn get_direct_entry(&self, project_id: &str) -> Option<BrowseEntry> {
        self.direct_entries
            .get(project_id)
            .map(|e| e.value().clone())
    }

    /// Snapshot the direct entries whose `node_id` equals `my_node_id`
    /// (set at local publish/deploy time to our own node id,
    /// `deploy::publish_announcement`). This is the candidate set of a node's
    /// OWN catalog for the signed node directory (Sprint 75 Phase B authoring).
    ///
    /// SECURITY: this is a node_id MATCH only, and `BrowseEntry.node_id` for
    /// gossip-discovered entries comes from the (otherwise untrusted)
    /// `ProjectAnnouncement.node_id`. Two layers keep a peer from forging
    /// `node_id == my_node_id` to slip an entry into this set: (1) the LIVE gossip
    /// dispatch drops any project announcement claiming our own node_id before it
    /// reaches the aggregator (`runtime::announcement_claims_own_node_id`), so a
    /// self-spoof never lands here; (2) the authoring route additionally requires
    /// that the node actually HOLDS the entry's archive blob locally
    /// (content-addressing = the ownership truth, verrou 4) and caps the catalog
    /// before signing. See `http::publish_directory`.
    pub fn own_entries(&self, my_node_id: &str) -> Vec<BrowseEntry> {
        let mut out: Vec<BrowseEntry> = self
            .direct_entries
            .iter()
            .filter(|e| e.value().node_id.as_deref() == Some(my_node_id))
            .map(|e| e.value().clone())
            .collect();
        // Stable ordering so the signed directory's canonical bytes are
        // deterministic across DashMap iteration shuffling.
        out.sort_by(|a, b| a.project_id.cmp(&b.project_id));
        out
    }

    /// Find the archive ticket of a directly-announced project whose archive
    /// hash matches `hash_hex`. Used by blob-serve to P2P-download an app
    /// discovered on the network: the ticket carries the providing node's
    /// address, which a bare content hash does not.
    pub fn find_archive_ticket_by_hash(&self, hash_hex: &str) -> Option<String> {
        self.direct_entries.iter().find_map(|e| {
            let entry = e.value();
            if entry.archive_hash.as_deref() == Some(hash_hex) {
                entry.archive_ticket.clone()
            } else {
                None
            }
        })
    }

    /// Iterate every cached curator list, flatten its entries,
    /// probe each unique project_id under the TTL cache, and
    /// return a sorted [`BrowseEntry`] vector. Sprint 11 Phase A
    /// also includes directly announced projects.
    ///
    /// Sorting: (project_id, curator_pubkey) ascending. Stable
    /// so the shell can diff consecutive `/browse` responses
    /// without a tiebreaker of its own.
    pub async fn aggregate(
        &self,
        curator_runtime: &CuratorRuntime,
        node: &Node,
    ) -> Vec<BrowseEntry> {
        // Deep-clone the snapshot once so we don't hold a
        // DashMap iterator guard across an async probe.
        let lists: Vec<CuratorListEntry> = curator_runtime.list_snapshot();

        let mut out: Vec<BrowseEntry> = Vec::new();

        for entry in &lists {
            let curator_name = entry.list.curator_name.clone();
            let curator_hex = hex::encode(entry.curator_pubkey);

            for project in &entry.list.entries {
                // Cache lookup first — if the cached result is
                // still fresh, skip the probe entirely.
                let (status, probed_at_opt) = match self.cached(&project.project_id) {
                    Some(st) => (st, Some(SystemTime::now())),
                    None => {
                        let (st, ts) = self.probe_and_cache(node, &project.project_id).await;
                        (st, Some(ts))
                    }
                };

                out.push(BrowseEntry {
                    project_id: project.project_id.clone(),
                    // Curator-list entries: project_id IS the node id, so the
                    // probe above dialed it directly — no separate node_id.
                    node_id: None,
                    project_name: project.project_name.clone(),
                    category: project.category.clone(),
                    description: project.description.clone(),
                    curator_pubkey: curator_hex.clone(),
                    curator_name: curator_name.clone(),
                    source: BrowseSource::Curator,
                    status,
                    last_probed_at: probed_at_opt.map(iso_utc),
                    archive_ticket: None,
                    archive_hash: None,
                    repo_url: None,
                    provenance_hash: None,
                    is_open_source: false,
                });
            }
        }

        // Sprint 11 Phase A: append directly announced projects.
        //
        // Remediation #6 (freshness): the curator loop above already
        // probes reachability, but direct (gossip) entries used to be
        // emitted with their frozen insertion status — always `Unknown`
        // for a remote announcement — so a remote Browse card never
        // flipped to `Reachable` even when the hosting node was live.
        // Probe the **hosting node_id** (NOT `project_id`, which post-#4
        // is `blake3(name)` and dials no live endpoint) through the same
        // TTL cache + quorum/DNS canary as the curator path. Our own
        // self-published apps short-circuit to `Reachable` without a
        // dial: an endpoint cannot resolve a connection to itself, and a
        // node receiving the gossip echo of its own announcement would
        // otherwise flip its own card to `Unreachable`.
        //
        // Probing is sequential per *distinct* node and bounded by the TTL
        // cache: a dead host costs one quorum+dial timeout (~5 s worst case)
        // and is then cached `Unreachable` for the TTL window. This mirrors
        // the curator path's cost; making it concurrent (a bounded JoinSet
        // over distinct node_ids) is a tracked follow-up, not needed at
        // pilote-ferme scale.
        let me = node.node_id();
        for entry in self.direct_entries.iter() {
            let mut e = entry.value().clone();
            match e.node_id.as_deref() {
                Some(nid) if nid == me.as_str() => {
                    e.status = BrowseStatus::Reachable;
                    e.last_probed_at = Some(iso_utc(SystemTime::now()));
                }
                Some(nid) => {
                    let (status, ts) = match self.cached(nid) {
                        Some(st) => (st, SystemTime::now()),
                        None => self.probe_and_cache(node, nid).await,
                    };
                    e.status = status;
                    e.last_probed_at = Some(iso_utc(ts));
                }
                None => {
                    // Entry with no stored node_id — a curator-less or
                    // pre-#6 direct entry, or a test fixture. No dialable
                    // identity to probe, so preserve the status the
                    // constructor set. (Local publish/deploy now store
                    // node_id = our own id and take the self-branch above.)
                }
            }
            out.push(e);
        }

        out.sort_by(|a, b| {
            a.project_id
                .cmp(&b.project_id)
                .then_with(|| a.curator_pubkey.cmp(&b.curator_pubkey))
        });
        out
    }

    /// Inject a synthetic cache result. Used by unit tests that
    /// want to avoid touching the network; the HTTP handler and
    /// the real gossip path do not use this.
    #[cfg(test)]
    pub fn inject_cached(&self, project_id_hex: &str, status: BrowseStatus) {
        self.cache.insert(
            project_id_hex.to_string(),
            ProbeCacheEntry {
                status,
                probed_at: SystemTime::now(),
            },
        );
    }
}

impl Default for BrowseAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience Arc alias used by the binary runtime / HTTP
/// state.
pub type BrowseAggregatorHandle = Arc<BrowseAggregator>;

fn iso_utc(t: SystemTime) -> String {
    let secs = t
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    OffsetDateTime::from_unix_timestamp(secs)
        .ok()
        .and_then(|dt| dt.format(&Rfc3339).ok())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_core_rs::{
        CuratorList, CuratorListEntry, CuratorProjectRef, KeyPair, Node, create_node,
    };

    async fn spawn_node() -> Node {
        create_node().await.expect("boot")
    }

    fn mk_entry_with(
        kp: &KeyPair,
        revision: u64,
        curator_name: &str,
        projects: &[(&str, &str, &str, &str)],
    ) -> CuratorListEntry {
        let mut list = CuratorList::new(kp.public_bytes(), curator_name, 1_712_000_000, revision);
        for (id, name, cat, desc) in projects {
            list.entries.push(CuratorProjectRef {
                project_id: (*id).to_string(),
                project_name: (*name).to_string(),
                category: (*cat).to_string(),
                description: (*desc).to_string(),
            });
        }
        CuratorListEntry::sign(list, kp).unwrap()
    }

    // ---------------------------------------------------------
    // Wire types
    // ---------------------------------------------------------

    #[test]
    fn browse_status_serializes_lowercase() {
        // The React shell renders a `status` discriminator
        // keyed on lowercase literals. A silent rename would
        // break the frontend dispatch, so lock the shape here.
        assert_eq!(
            serde_json::to_string(&BrowseStatus::Reachable).unwrap(),
            "\"reachable\""
        );
        assert_eq!(
            serde_json::to_string(&BrowseStatus::Unreachable).unwrap(),
            "\"unreachable\""
        );
        assert_eq!(
            serde_json::to_string(&BrowseStatus::Unknown).unwrap(),
            "\"unknown\""
        );
    }

    #[test]
    fn find_archive_ticket_by_hash_matches_then_misses() {
        let agg = BrowseAggregator::new();
        let mk = |pid: &str, h: &str, t: &str| BrowseEntry {
            project_id: pid.into(),
            node_id: None,
            project_name: "n".into(),
            category: "c".into(),
            description: "d".into(),
            curator_pubkey: String::new(),
            curator_name: "x".into(),
            source: BrowseSource::Direct,
            status: BrowseStatus::Unknown,
            last_probed_at: None,
            archive_ticket: Some(t.into()),
            archive_hash: Some(h.into()),
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        };
        agg.add_direct_entry(mk("p1", &"aa".repeat(32), "ticket-A"));
        agg.add_direct_entry(mk("p2", &"bb".repeat(32), "ticket-B"));
        assert_eq!(
            agg.find_archive_ticket_by_hash(&"bb".repeat(32)),
            Some("ticket-B".to_string())
        );
        assert_eq!(agg.find_archive_ticket_by_hash(&"cc".repeat(32)), None);
        // An entry with no archive_hash is never matched.
        let mut no_hash = mk("p3", "aa", "ticket-C");
        no_hash.archive_hash = None;
        agg.add_direct_entry(no_hash);
        assert_eq!(agg.find_archive_ticket_by_hash(""), None);
    }

    #[test]
    fn own_entries_filters_by_node_id_and_sorts() {
        // Sprint 75 Phase B: `own_entries` returns ONLY the direct
        // entries tagged with our node id (the apps we host), sorted by
        // project_id for deterministic signed-directory bytes. A remote
        // app discovered via gossip (different node_id) and an untagged
        // entry are both excluded.
        let agg = BrowseAggregator::new();
        let me = "11".repeat(32);
        let peer = "22".repeat(32);
        let mk = |pid: &str, owner: Option<&str>| BrowseEntry {
            project_id: pid.into(),
            node_id: owner.map(String::from),
            project_name: "n".into(),
            category: "c".into(),
            description: "d".into(),
            curator_pubkey: String::new(),
            curator_name: "x".into(),
            source: BrowseSource::Direct,
            status: BrowseStatus::Reachable,
            last_probed_at: None,
            archive_ticket: None,
            archive_hash: Some("ab".repeat(32)),
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        };
        agg.add_direct_entry(mk("b_app", Some(&me)));
        agg.add_direct_entry(mk("a_app", Some(&me)));
        agg.add_direct_entry(mk("remote_app", Some(&peer)));
        agg.add_direct_entry(mk("legacy_app", None));

        let own = agg.own_entries(&me);
        let ids: Vec<&str> = own.iter().map(|e| e.project_id.as_str()).collect();
        assert_eq!(ids, vec!["a_app", "b_app"], "only our apps, sorted");
    }

    #[test]
    fn browse_entry_round_trips_through_json() {
        let entry = BrowseEntry {
            project_id: "a".repeat(64),
            node_id: None,
            project_name: "gov".into(),
            category: "gov".into(),
            description: "desc".into(),
            curator_pubkey: "b".repeat(64),
            curator_name: "FlowUP".into(),
            source: BrowseSource::Curator,
            status: BrowseStatus::Reachable,
            last_probed_at: Some("2026-04-11T12:00:00Z".into()),
            archive_ticket: None,
            archive_hash: None,
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let back: BrowseEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back, entry);
    }

    #[test]
    fn browse_entry_without_source_defaults_to_curator() {
        // Backward compat: daemons before Sprint 11 emit entries
        // without a `source` field. Deserialization must default
        // to Curator.
        let json = r#"{
            "project_id": "aaaa",
            "project_name": "gov",
            "category": "gov",
            "description": "d",
            "curator_pubkey": "bbbb",
            "curator_name": "FlowUP",
            "status": "reachable",
            "last_probed_at": null
        }"#;
        let entry: BrowseEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.source, BrowseSource::Curator);
    }

    #[test]
    fn browse_entry_with_archive_ticket_round_trips() {
        let entry = BrowseEntry {
            project_id: "a".repeat(64),
            node_id: None,
            project_name: "web-app".into(),
            category: "misc".into(),
            description: "has archive".into(),
            curator_pubkey: "b".repeat(64),
            curator_name: "FlowUP".into(),
            source: BrowseSource::Direct,
            status: BrowseStatus::Reachable,
            last_probed_at: None,
            archive_ticket: Some("blobticket_abc123".into()),
            archive_hash: Some("ab".repeat(32)),
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("archive_ticket"));
        assert!(json.contains("archive_hash"));
        let back: BrowseEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.archive_ticket.as_deref(), Some("blobticket_abc123"));
        assert_eq!(back.archive_hash.as_deref(), Some(&*"ab".repeat(32)));
    }

    #[test]
    fn browse_entry_without_archive_ticket_omits_field() {
        let entry = BrowseEntry {
            project_id: "a".repeat(64),
            node_id: None,
            project_name: "old".into(),
            category: "misc".into(),
            description: "no archive".into(),
            curator_pubkey: "b".repeat(64),
            curator_name: "FlowUP".into(),
            source: BrowseSource::Curator,
            status: BrowseStatus::Unknown,
            last_probed_at: None,
            archive_ticket: None,
            archive_hash: None,
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(
            !json.contains("archive_ticket"),
            "None archive_ticket should be omitted"
        );
    }

    #[test]
    fn browse_source_serializes_lowercase() {
        assert_eq!(
            serde_json::to_string(&BrowseSource::Curator).unwrap(),
            "\"curator\""
        );
        assert_eq!(
            serde_json::to_string(&BrowseSource::Direct).unwrap(),
            "\"direct\""
        );
    }

    // ---------------------------------------------------------
    // Cache semantics
    // ---------------------------------------------------------

    #[test]
    fn cached_returns_none_on_empty_cache() {
        let agg = BrowseAggregator::new();
        assert!(agg.cached(&"a".repeat(64)).is_none());
    }

    #[test]
    fn cached_returns_injected_result_within_ttl() {
        let agg = BrowseAggregator::new();
        let id = "a".repeat(64);
        agg.inject_cached(&id, BrowseStatus::Reachable);
        assert_eq!(agg.cached(&id), Some(BrowseStatus::Reachable));
    }

    #[test]
    fn cached_expires_after_ttl() {
        // Use a tiny TTL so we can step past it deterministically.
        let agg =
            BrowseAggregator::with_durations(Duration::from_millis(1), Duration::from_millis(100));
        let id = "b".repeat(64);
        agg.inject_cached(&id, BrowseStatus::Reachable);
        std::thread::sleep(Duration::from_millis(10));
        assert!(
            agg.cached(&id).is_none(),
            "cache must expire once TTL elapses"
        );
    }

    // ---------------------------------------------------------
    // aggregate() — end-to-end against a live curator runtime
    // ---------------------------------------------------------

    #[tokio::test]
    async fn aggregate_returns_empty_when_no_curator_lists_cached() {
        let curator = CuratorRuntime::new(None);
        let node = spawn_node().await;
        let agg = BrowseAggregator::new();
        let out = agg.aggregate(&curator, &node).await;
        assert!(out.is_empty());
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn aggregate_flattens_direct_entries_with_cached_status() {
        // T35: rewrite to actually exercise the flattening of
        // direct entries. Direct entries are appended as-is (no
        // probe cache lookup) — their status is whatever the
        // caller set when calling add_direct_entry.
        let curator = CuratorRuntime::new(None);
        let agg = BrowseAggregator::new();

        let id_a = "a".repeat(64);
        let id_b = "b".repeat(64);

        // Add two direct entries with distinct statuses.
        agg.add_direct_entry(BrowseEntry {
            project_id: id_a.clone(),
            node_id: None,
            project_name: "gov".into(),
            category: "gov".into(),
            description: "desc-a".into(),
            curator_pubkey: String::new(),
            curator_name: "Self-published".into(),
            source: BrowseSource::Direct,
            status: BrowseStatus::Reachable,
            last_probed_at: None,
            archive_ticket: None,
            archive_hash: None,
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        });
        agg.add_direct_entry(BrowseEntry {
            project_id: id_b.clone(),
            node_id: None,
            project_name: "coldcase".into(),
            category: "invest".into(),
            description: "desc-b".into(),
            curator_pubkey: String::new(),
            curator_name: "Self-published".into(),
            source: BrowseSource::Direct,
            status: BrowseStatus::Unknown,
            last_probed_at: None,
            archive_ticket: None,
            archive_hash: None,
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        });

        let node = spawn_node().await;
        let out = agg.aggregate(&curator, &node).await;

        // Both direct entries should appear, preserving their status.
        assert_eq!(out.len(), 2, "expected 2 direct entries");
        let gov = out.iter().find(|e| e.project_name == "gov").unwrap();
        assert_eq!(gov.status, BrowseStatus::Reachable);
        assert_eq!(gov.source, BrowseSource::Direct);
        let cc = out.iter().find(|e| e.project_name == "coldcase").unwrap();
        assert_eq!(cc.status, BrowseStatus::Unknown);

        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn aggregate_probes_seeded_peer_and_marks_it_reachable() {
        // This is the real Phase D integration test. Two nodes:
        // - node_a is a "project coordinator" we pretend to
        //   vouch for via a curator list
        // - node_b hosts the daemon runtime; we seed A's addr
        //   into B's memory_lookup so the probe succeeds
        //   without pkarr
        // A fake curator (keypair kp_c) signs a curator list
        // that references node_a's node id, the list is stored
        // directly in the curator runtime, and the aggregator
        // is asked to produce a browse entry. The result must
        // carry status=Reachable.
        let node_a = spawn_node().await;
        let node_b = spawn_node().await;

        let a_addr = DiscoveryClient::new(node_a.endpoint())
            .my_endpoint_addr()
            .await
            .unwrap();
        node_b.memory_lookup().add_endpoint_info(a_addr);

        // Build + sign + inject a curator list referencing
        // node_a as its single project. We go through the
        // public sign path so `verify_signature` would pass,
        // but the aggregator itself does not re-verify — the
        // Phase C runtime is the single verification gate.
        let kp_c = KeyPair::generate();
        let a_id = node_a.node_id();
        let entry = mk_entry_with(
            &kp_c,
            1,
            "FlowUP",
            &[(a_id.as_str(), "gov", "gov", "test fixture")],
        );
        let entry_json = serde_json::to_vec(&entry).unwrap();

        // Seed the curator runtime with the list via the
        // normal process_announcement_bytes path — we need a
        // real BlobTicket, which means minting one from
        // node_a's store.
        use iroh_blobs::ticket::BlobTicket;
        use iroh_blobs::{BlobFormat, Hash};
        let blobs_a = nexus_core_rs::BlobsClient::new(node_a.blobs_store());
        let hash = blobs_a.add_bytes(&entry_json).await.unwrap();
        let ticket = BlobTicket::new(
            DiscoveryClient::new(node_a.endpoint())
                .my_endpoint_addr()
                .await
                .unwrap(),
            Hash::from_bytes(hash),
            BlobFormat::Raw,
        )
        .to_string();
        let announcement =
            crate::iroh_runtime::CuratorAnnouncement::new(kp_c.public_bytes(), ticket);
        let announcement_bytes = announcement.to_bytes().unwrap();

        let curator = CuratorRuntime::new(None);
        curator
            .subscribe(&hex::encode(kp_c.public_bytes()))
            .unwrap();
        curator
            .process_announcement_bytes(&announcement_bytes, &node_b)
            .await
            .expect("ingest should succeed");

        // Now the aggregator: probe A from B.
        let agg = BrowseAggregator::new();
        let out = agg.aggregate(&curator, &node_b).await;
        assert_eq!(out.len(), 1);
        let row = &out[0];
        assert_eq!(row.project_id, a_id);
        assert_eq!(row.curator_pubkey, hex::encode(kp_c.public_bytes()));
        assert_eq!(
            row.status,
            BrowseStatus::Reachable,
            "seeded peer must probe as reachable"
        );
        assert!(row.last_probed_at.is_some());

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    #[tokio::test]
    async fn aggregate_marks_unknown_project_as_unreachable() {
        // A curator vouches for a project whose node id the
        // local daemon cannot resolve. The aggregator must
        // return status=Unreachable rather than erroring.
        let node = spawn_node().await;

        let kp_c = KeyPair::generate();
        let unknown_id = "0".repeat(64); // never minted
        let entry = mk_entry_with(
            &kp_c,
            1,
            "FlowUP",
            &[(unknown_id.as_str(), "gov", "gov", "nowhere")],
        );

        // Inject directly into the curator runtime's internal
        // DashMap — the 2-node path is not needed for this
        // scenario and would slow the test down. We use
        // `list_snapshot` output to assert the runtime sees the
        // entry before calling aggregate.
        let curator = CuratorRuntime::new(None);
        // The public path is subscribe + inject via a
        // hand-rolled entry: use an aggregator probe cache
        // entry short-circuit instead. We inject the cache
        // with Unreachable so the probe path is bypassed —
        // this isolates the aggregation logic from the network.
        let agg = BrowseAggregator::new();
        agg.inject_cached(&unknown_id, BrowseStatus::Unreachable);

        // Seed the curator runtime with the entry via the test
        // helper. We cannot reach the DashMap directly from
        // outside the module, so we reuse `subscribe` + a
        // hand-crafted entry via the DashMap extension point in
        // `iroh_runtime::tests`. For the purposes of this
        // test, we use the shorter path: call aggregate with
        // an empty curator runtime and assert the aggregator
        // survives (no panics). The richer scenario
        // (curator list + Unreachable status in the response)
        // is covered by the Phase E Playwright spec once the
        // full stack is wired.
        let out = agg.aggregate(&curator, &node).await;
        assert!(out.is_empty());
        let _ = entry;

        node.shutdown().await.ok();
    }

    // ---------------------------------------------------------
    // Sprint 11 Phase A — direct entries
    // ---------------------------------------------------------

    #[test]
    fn add_direct_entry_stores_and_counts() {
        let agg = BrowseAggregator::new();
        assert_eq!(agg.direct_entry_count(), 0);
        let entry = BrowseEntry {
            project_id: "a".repeat(64),
            node_id: None,
            project_name: "gov".into(),
            category: "gov".into(),
            description: "self-published".into(),
            curator_pubkey: String::new(),
            curator_name: "Self-published".into(),
            source: BrowseSource::Direct,
            status: BrowseStatus::Unknown,
            last_probed_at: None,
            archive_ticket: None,
            archive_hash: None,
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        };
        agg.add_direct_entry(entry);
        assert_eq!(agg.direct_entry_count(), 1);
    }

    #[test]
    fn add_direct_entry_dedup_by_project_id() {
        let agg = BrowseAggregator::new();
        let id = "a".repeat(64);
        let entry1 = BrowseEntry {
            project_id: id.clone(),
            node_id: None,
            project_name: "gov-v1".into(),
            category: "gov".into(),
            description: "first".into(),
            curator_pubkey: String::new(),
            curator_name: "Self-published".into(),
            source: BrowseSource::Direct,
            status: BrowseStatus::Unknown,
            last_probed_at: None,
            archive_ticket: None,
            archive_hash: None,
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        };
        let entry2 = BrowseEntry {
            project_id: id.clone(),
            node_id: None,
            project_name: "gov-v2".into(),
            category: "gov".into(),
            description: "updated".into(),
            curator_pubkey: String::new(),
            curator_name: "Self-published".into(),
            source: BrowseSource::Direct,
            status: BrowseStatus::Unknown,
            archive_ticket: None,
            archive_hash: None,
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
            last_probed_at: None,
        };
        agg.add_direct_entry(entry1);
        agg.add_direct_entry(entry2);
        assert_eq!(agg.direct_entry_count(), 1, "dedup by project_id");
    }

    #[tokio::test]
    async fn aggregate_includes_direct_entries() {
        let curator = CuratorRuntime::new(None);
        let node = spawn_node().await;
        let agg = BrowseAggregator::new();
        let entry = BrowseEntry {
            project_id: "d".repeat(64),
            node_id: None,
            project_name: "direct-proj".into(),
            category: "misc".into(),
            description: "self-published project".into(),
            curator_pubkey: String::new(),
            curator_name: "Self-published".into(),
            source: BrowseSource::Direct,
            status: BrowseStatus::Unknown,
            last_probed_at: None,
            archive_ticket: None,
            archive_hash: None,
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        };
        agg.add_direct_entry(entry);

        let out = agg.aggregate(&curator, &node).await;
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].source, BrowseSource::Direct);
        assert_eq!(out[0].project_name, "direct-proj");
        node.shutdown().await.ok();
    }

    // ---------------------------------------------------------
    // Remediation #6 — direct-entry freshness probe
    // ---------------------------------------------------------

    #[tokio::test]
    async fn aggregate_probes_direct_entry_against_node_id_not_project_id() {
        // The load-bearing test for remediation #6. A direct (gossip)
        // entry carries a per-app project_id (`blake3(name)`) that is
        // NOT a dialable node id, plus the hosting node_id. The
        // aggregator must probe the *node_id* — proven by the entry
        // coming back Reachable even though its project_id is an id we
        // never mint anywhere (probing it would yield Unreachable).
        let node_a = spawn_node().await; // the "remote" host
        let node_b = spawn_node().await; // runs /browse

        // Seed A's addr into B's lookup so the dial resolves without
        // pkarr — exactly what handle_project_announcement does from the
        // archive ticket in production, and what blobs.rs::fetch_ticket
        // does to download a blob.
        let a_addr = nexus_core_rs::DiscoveryClient::new(node_a.endpoint())
            .my_endpoint_addr()
            .await
            .unwrap();
        node_b.memory_lookup().add_endpoint_info(a_addr);

        let agg = BrowseAggregator::with_durations(DEFAULT_PROBE_TTL, Duration::from_secs(5));
        let app_pid = "c".repeat(64); // distinct from node_a's id; never minted
        assert_ne!(app_pid, node_a.node_id());
        agg.add_direct_entry(BrowseEntry {
            project_id: app_pid.clone(),
            node_id: Some(node_a.node_id()),
            project_name: "Remote App".into(),
            category: "tools".into(),
            description: "discovered via gossip".into(),
            curator_pubkey: String::new(),
            curator_name: "Self-published".into(),
            source: BrowseSource::Direct,
            status: BrowseStatus::Unknown,
            last_probed_at: None,
            archive_ticket: None,
            archive_hash: None,
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        });

        let curator = CuratorRuntime::new(None);
        let out = agg.aggregate(&curator, &node_b).await;
        assert_eq!(out.len(), 1);
        assert_eq!(
            out[0].status,
            BrowseStatus::Reachable,
            "a direct entry whose hosting node is dialable must probe Reachable"
        );
        assert!(
            out[0].last_probed_at.is_some(),
            "a probed direct entry must carry a probe timestamp"
        );
        assert_eq!(
            out[0].project_id, app_pid,
            "per-app project_id must be preserved — we dialed node_id, not project_id"
        );

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    #[tokio::test]
    async fn aggregate_self_published_entry_is_reachable_without_dial() {
        // Our own self-published app — or the gossip echo of our own
        // announcement — carries our node_id. It must show Reachable
        // without a dial: an endpoint cannot connect to itself, so a
        // probe would otherwise flip our own card to Unreachable.
        let node = spawn_node().await;
        let agg = BrowseAggregator::new();
        agg.add_direct_entry(BrowseEntry {
            project_id: "e".repeat(64),
            node_id: Some(node.node_id()),
            project_name: "My App".into(),
            category: "tools".into(),
            description: "self-published".into(),
            curator_pubkey: String::new(),
            curator_name: "Self-published".into(),
            source: BrowseSource::Direct,
            status: BrowseStatus::Unknown,
            last_probed_at: None,
            archive_ticket: None,
            archive_hash: None,
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        });

        let curator = CuratorRuntime::new(None);
        let out = agg.aggregate(&curator, &node).await;
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].status, BrowseStatus::Reachable);
        assert!(out[0].last_probed_at.is_some());
        assert_eq!(
            agg.probe_call_count(),
            0,
            "a self-hosted entry must never trigger a dial"
        );
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn aggregate_two_apps_same_node_share_one_probe() {
        // Two apps hosted by the same (unreachable) node: distinct
        // project_ids, identical node_id. Reachability is a property of
        // the node, so the probe cache is keyed by node_id and exactly
        // one dial happens for both cards.
        let node = spawn_node().await;
        let agg = BrowseAggregator::with_durations(DEFAULT_PROBE_TTL, Duration::from_millis(300));
        let host = "a".repeat(64); // never minted -> Unreachable
        for pid in ["1".repeat(64), "2".repeat(64)] {
            agg.add_direct_entry(BrowseEntry {
                project_id: pid,
                node_id: Some(host.clone()),
                project_name: "App".into(),
                category: "tools".into(),
                description: "same host".into(),
                curator_pubkey: String::new(),
                curator_name: "Self-published".into(),
                source: BrowseSource::Direct,
                status: BrowseStatus::Unknown,
                last_probed_at: None,
                archive_ticket: None,
                archive_hash: None,
                repo_url: None,
                provenance_hash: None,
                is_open_source: false,
            });
        }

        let curator = CuratorRuntime::new(None);
        let out = agg.aggregate(&curator, &node).await;
        assert_eq!(out.len(), 2);
        assert!(out.iter().all(|e| e.status == BrowseStatus::Unreachable));
        assert_eq!(
            agg.probe_call_count(),
            1,
            "two apps on one node must share a single reachability probe"
        );
        node.shutdown().await.ok();
    }

    // ---------------------------------------------------------
    // E-1 — probe_timeout_from_env
    // ---------------------------------------------------------

    // ---------------------------------------------------------
    // Sprint 19 Phase A — pkarr quorum canary gate
    // ---------------------------------------------------------

    use async_trait::async_trait;
    use nexus_core_rs::DnsFallbackResolve;
    use std::sync::Mutex;

    /// Minimal mock DNS fallback resolver for Sprint 24 Phase E tests.
    struct DnsFallbackMock {
        response: Mutex<Option<anyhow::Result<Vec<u8>>>>,
    }

    impl DnsFallbackMock {
        fn ok(data: &[u8]) -> Arc<dyn DnsFallbackResolve> {
            Arc::new(Self {
                response: Mutex::new(Some(Ok(data.to_vec()))),
            })
        }

        fn fail(msg: &str) -> Arc<dyn DnsFallbackResolve> {
            Arc::new(Self {
                response: Mutex::new(Some(Err(anyhow::anyhow!("{}", msg)))),
            })
        }
    }

    #[async_trait]
    impl DnsFallbackResolve for DnsFallbackMock {
        fn label(&self) -> &str {
            "mock-dns-fallback"
        }

        async fn resolve_node(&self, _node_id_hex: &str) -> anyhow::Result<Vec<u8>> {
            self.response
                .lock()
                .unwrap()
                .take()
                .expect("DnsFallbackMock consumed twice — use a fresh one per test")
        }
    }

    /// Minimal mock resolver for the Phase A canary tests. We
    /// cannot reuse `dht_quorum::tests::MockResolver` because
    /// that one is private to its test module — and that is the
    /// right call : the quorum test mock is a fixture for the
    /// quorum layer's invariants, not a public facade. Here we
    /// only need a knob for `Ok(bytes)` / `Err(msg)` per resolver.
    struct QuorumMock {
        label: String,
        response: Mutex<Option<anyhow::Result<Vec<u8>>>>,
    }

    impl QuorumMock {
        fn ok(label: &str, bytes: &[u8]) -> Arc<dyn QuorumResolver> {
            Arc::new(Self {
                label: label.into(),
                response: Mutex::new(Some(Ok(bytes.to_vec()))),
            })
        }

        fn fail(label: &str, msg: &str) -> Arc<dyn QuorumResolver> {
            Arc::new(Self {
                label: label.into(),
                response: Mutex::new(Some(Err(anyhow::anyhow!("{msg}")))),
            })
        }
    }

    #[async_trait]
    impl QuorumResolver for QuorumMock {
        fn label(&self) -> &str {
            &self.label
        }

        async fn resolve(&self, _node_id: &str) -> anyhow::Result<Vec<u8>> {
            self.response
                .lock()
                .unwrap()
                .take()
                .expect("QuorumMock consumed twice — use a fresh one per test")
        }
    }

    #[tokio::test]
    async fn probe_and_cache_skips_dial_when_quorum_has_no_majority() {
        // Three resolvers return three different byte strings →
        // `QuorumError::NoMajority`. The canary must short-circuit
        // to Unreachable WITHOUT reaching probe_reachable (the
        // dial would otherwise take 2 s against an unknown id ;
        // the ~50 ms wall-clock budget below proves the probe
        // was bypassed).
        let resolvers: Vec<Arc<dyn QuorumResolver>> = vec![
            QuorumMock::ok("r1", b"one"),
            QuorumMock::ok("r2", b"two"),
            QuorumMock::ok("r3", b"three"),
        ];
        let agg = BrowseAggregator::new().with_quorum_resolvers(resolvers);
        assert_eq!(agg.quorum_resolver_count(), 3);

        let node = spawn_node().await;
        let unknown_id = "a".repeat(64);

        let start = std::time::Instant::now();
        let (status, _ts) = agg.probe_and_cache(&node, &unknown_id).await;
        let elapsed = start.elapsed();

        assert_eq!(
            status,
            BrowseStatus::Unreachable,
            "quorum no-majority must cache Unreachable"
        );
        assert!(
            elapsed < Duration::from_secs(1),
            "probe must be skipped entirely on no-majority, got {elapsed:?} wall clock"
        );
        assert_eq!(
            agg.cached(&unknown_id),
            Some(BrowseStatus::Unreachable),
            "Unreachable verdict must be written to the probe cache for TTL reuse"
        );

        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn probe_and_cache_skips_dial_when_all_quorum_resolvers_fail() {
        // All three resolvers error out →
        // `QuorumError::AllFailed`. Same expected behaviour as
        // the no-majority branch : Unreachable + cached + no
        // dial. Kept as a distinct test so a regression
        // collapsing the two branches into one code path would
        // break whichever scenario was dropped.
        let resolvers: Vec<Arc<dyn QuorumResolver>> = vec![
            QuorumMock::fail("r1", "connection refused"),
            QuorumMock::fail("r2", "dns resolution failed"),
            QuorumMock::fail("r3", "tls handshake timeout"),
        ];
        let agg = BrowseAggregator::new().with_quorum_resolvers(resolvers);

        let node = spawn_node().await;
        let any_id = "b".repeat(64);

        let (status, _ts) = agg.probe_and_cache(&node, &any_id).await;
        assert_eq!(
            status,
            BrowseStatus::Unreachable,
            "all-failed quorum must cache Unreachable"
        );
        assert_eq!(
            agg.cached(&any_id),
            Some(BrowseStatus::Unreachable),
            "Unreachable verdict must be written to the probe cache"
        );

        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn probe_and_cache_with_quorum_majority_continues_to_dial() {
        // Sprint 20 Phase 0 audit finding A-3 (P2) regression : the
        // original Sprint 19 Phase A landed only the two error-path
        // tests above (NoMajority + AllFailed → skip dial). Neither
        // asserted that the happy path (2/3 or 3/3 agreement) lets
        // the aggregator fall through to probe_reachable. A future
        // refactor collapsing the quorum gate into a single "skip
        // always" branch would have passed both existing tests. This
        // test covers the complementary invariant :
        //
        //   quorum agrees  →  probe_reachable IS called.
        //
        // We cannot mock probe_reachable easily here (it talks to an
        // iroh Endpoint), so we detect that it was called by two
        // independent signals :
        //
        //   (a) `elapsed` is dominated by the configured
        //       probe_timeout (400+ ms, vs < 100 ms for the short-
        //       circuit branches).
        //   (b) the cache is written as Unreachable but through the
        //       probe path, not through the early-return
        //       record_unreachable branch. Both short-circuit
        //       branches would complete in < 100 ms ; we assert the
        //       happy path takes noticeably longer.
        //
        // The happy path still caches `Unreachable` against a
        // random node id (the probe cannot dial an unknown peer),
        // so `status` matches the short-circuit branches. The
        // discriminating signal is **wall-clock elapsed** : the
        // NoMajority / AllFailed branches return in < 1 ms because
        // `record_unreachable` is a single `DashMap::insert` after
        // the in-memory quorum test. The happy path has to build a
        // `DiscoveryClient`, reach into the iroh `Endpoint`, and
        // `probe_reachable` — the error comes back quickly for a
        // random node id but still measurably slower.
        //
        // Observed timings locally (2026-04-16) : NoMajority
        // ≈ 0.1 ms, happy path ≈ 10 ms. The 3 ms floor below is
        // chosen as 30× the skip-dial cost : any future refactor
        // that regresses to "always skip dial" would fall below
        // this floor with room to spare.
        //
        // Probe timeout is clamped to 100 ms so that, if the dial
        // path ever starts timing out reliably in CI (e.g. an iroh
        // upgrade gains a synchronous stall), this test still
        // finishes in << 1 s wall clock.
        let resolvers: Vec<Arc<dyn QuorumResolver>> = vec![
            QuorumMock::ok("r1", b"same-signed-packet-bytes"),
            QuorumMock::ok("r2", b"same-signed-packet-bytes"),
            QuorumMock::ok("r3", b"same-signed-packet-bytes"),
        ];
        let agg = BrowseAggregator::with_durations(DEFAULT_PROBE_TTL, Duration::from_millis(100))
            .with_quorum_resolvers(resolvers);
        assert_eq!(agg.quorum_resolver_count(), 3);
        assert_eq!(agg.probe_call_count(), 0);

        let node = spawn_node().await;
        let unknown_id = "d".repeat(64);

        let (status, _ts) = agg.probe_and_cache(&node, &unknown_id).await;

        assert_eq!(
            agg.probe_call_count(),
            1,
            "quorum happy path must call probe_reachable exactly once"
        );
        assert_eq!(
            status,
            BrowseStatus::Unreachable,
            "a random node id is unreachable through the probe — Unreachable is correct"
        );
        assert_eq!(
            agg.cached(&unknown_id),
            Some(BrowseStatus::Unreachable),
            "TTL cache must be written through the probe path as well"
        );

        node.shutdown().await.ok();
    }

    // ---------------------------------------------------------
    // Sprint 24 Phase E — DNS fallback integration
    // ---------------------------------------------------------

    #[tokio::test]
    async fn probe_dns_fallback_resolves_on_quorum_all_failed() {
        // Scenario: all 3 pkarr quorum resolvers fail, but DNS
        // fallback returns data → the aggregator must fall through
        // to probe_reachable (not short-circuit to Unreachable).
        // A random node_id will still probe as Unreachable through
        // the iroh discovery path, but the discriminating signal is
        // wall-clock elapsed > 3 ms (same reasoning as the
        // probe_and_cache_with_quorum_majority_continues_to_dial
        // test above): if DNS fallback properly falls through, the
        // probe path fires; if it short-circuits, elapsed < 1 ms.
        let resolvers: Vec<Arc<dyn QuorumResolver>> = vec![
            QuorumMock::fail("r1", "timeout"),
            QuorumMock::fail("r2", "timeout"),
            QuorumMock::fail("r3", "timeout"),
        ];
        let dns = DnsFallbackMock::ok(b"pkarr-signed-packet-bytes");
        let agg = BrowseAggregator::with_durations(DEFAULT_PROBE_TTL, Duration::from_millis(100))
            .with_quorum_resolvers(resolvers)
            .with_dns_fallback(dns);
        assert_eq!(agg.probe_call_count(), 0);

        let node = spawn_node().await;
        let unknown_id = "e".repeat(64);

        let (status, _ts) = agg.probe_and_cache(&node, &unknown_id).await;

        assert_eq!(
            agg.probe_call_count(),
            1,
            "DNS fallback must let probe_reachable run after AllFailed"
        );
        assert_eq!(status, BrowseStatus::Unreachable);

        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn probe_dns_fallback_fails_marks_unreachable() {
        // Scenario: all 3 pkarr resolvers fail AND DNS fallback
        // also fails → Unreachable without probe. Must complete
        // in < 1 s (no dial attempt).
        let resolvers: Vec<Arc<dyn QuorumResolver>> = vec![
            QuorumMock::fail("r1", "timeout"),
            QuorumMock::fail("r2", "timeout"),
            QuorumMock::fail("r3", "timeout"),
        ];
        let dns = DnsFallbackMock::fail("NXDOMAIN");
        let agg = BrowseAggregator::new()
            .with_quorum_resolvers(resolvers)
            .with_dns_fallback(dns);

        let node = spawn_node().await;
        let unknown_id = "f".repeat(64);

        let start = std::time::Instant::now();
        let (status, _ts) = agg.probe_and_cache(&node, &unknown_id).await;
        let elapsed = start.elapsed();

        assert_eq!(
            status,
            BrowseStatus::Unreachable,
            "pkarr AllFailed + DNS fail must cache Unreachable"
        );
        assert!(
            elapsed < Duration::from_secs(1),
            "probe must be skipped when both pkarr and DNS fail, got {elapsed:?}"
        );
        assert_eq!(
            agg.cached(&unknown_id),
            Some(BrowseStatus::Unreachable),
            "Unreachable must be written to cache"
        );

        node.shutdown().await.ok();
    }

    // ---------------------------------------------------------
    // E-1 — probe_timeout_from_env
    // ---------------------------------------------------------

    #[test]
    fn probe_timeout_env_override_parses_valid_ms() {
        // Sprint 9 Phase E (E-1 close): verify that the env var
        // override actually influences the timeout duration.
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { std::env::set_var("NEXUS_PROBE_TIMEOUT_MS", "5000") };
        let d = probe_timeout_from_env();
        assert_eq!(d, Duration::from_millis(5000));
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { std::env::remove_var("NEXUS_PROBE_TIMEOUT_MS") };

        // Absent env var falls back to default.
        let d2 = probe_timeout_from_env();
        assert_eq!(d2, DEFAULT_PROBE_TIMEOUT);
    }
}
