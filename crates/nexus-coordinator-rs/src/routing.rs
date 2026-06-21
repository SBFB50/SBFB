// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 77 Phase E — Parallax routing DAG + Petals active churn (phase 2).
//!
//! This is the **routing** half of the two-phase Parallax-style scheduler
//! (the **placement** half is Phase D, [`crate::placement`]). Where placement
//! decides, once per session, how to split a model into a contiguous pipeline
//! of layer blocks, routing decides, **per request**, which concrete worker
//! serves each pipeline stage when several replicas can — and how to recover
//! when one of them drops mid-run.
//!
//! It contains three pure, in-memory, deterministic pieces:
//!
//! - **Routing DAG sweep DP** ([`route_min_latency`]): a single
//!   left-to-right shortest-path dynamic program over a layer-indexed DAG,
//!   `dp(s,g') = min_g (dp(s-1,g) + rho<g,g'>) + tau<g',s>` (Parallax phase 2),
//!   selecting the minimum-latency chain across the stages. `O(S·R²)` —
//!   negligible at the 3-5 replicas a private group has in practice.
//! - **Active churn** ([`replace_failed_server`], [`assign_fallback_nodes`],
//!   [`ActivationReplayCache`]): the **Petals** active-rebalancing model — a
//!   dropped worker is replaced by the next best **allowlisted** server for
//!   its stage (a bounded fallback heap), and the client replays the cached
//!   boundary activation onto the replacement so the inference continues. This
//!   is deliberately **not** Parallax's "DHT key expires" model, which never
//!   re-routes mid-inference (kickoff D3 rejected it as a churn flaw).
//! - **Perf-map** ([`PerfMap`]): the `(rho, tau)` cost snapshot the routing DP
//!   consumes, (de)serialised as an **unsigned raw-op** for the control plane.
//!
//! ## What it is NOT (Phase E preflight S2-F1/S4-F1/S4-F2)
//!
//! - It signs nothing and mints **no new `DOMAIN_*`** (the sprint's wire
//!   budget §19 is closed at four: compute_group / shard_plan / run_proof /
//!   activation_commit). The [`PerfMap`] is an **unsigned** raw-op
//!   (`serde_json::Value`, the `FeedEntry.op` body) — control-plane data, not
//!   an authorisation/non-repudiation attestation. It carries **zero
//!   `*_FORMAT_VERSION` bump**. Who published a perf-map is authenticated by
//!   the **existing** signed `FeedEntry` envelope plus a
//!   `ComputeGroup::is_member` ingest gate; that wiring is daemon-side.
//! - It does **no** iroh-docs I/O. `nexus-coordinator-rs` has no iroh
//!   dependency at all — the literal 1-2s republish of the perf-map to a doc
//!   (every [`PERF_MAP_REPUBLISH_INTERVAL`]) is thin daemon glue
//!   (`nexus-shell-daemon` owns the doc handle). This module only **produces /
//!   consumes** a [`PerfMap`] and runs the routing/churn compute, exactly as
//!   [`crate::placement`] only produces a `ShardPlan`.
//!
//! ## Why integer micros (the no-float house style, not the A/B/C streak)
//!
//! `rho` (measured round-trip RTT per pair) and `tau` (profiled per-layer/GPU
//! latency) are kept as **integer microseconds** (`u64`), mirroring
//! [`crate::placement::RttMatrix`] (`as_micros()`) and
//! [`nexus_core_rs::shard_plan::RunMetrics`] (all-integer). An unsigned raw-op
//! would *technically* tolerate a floating-point latency, but a float cannot
//! round-trip bit-identically across platforms and would break `Eq` on
//! [`PerfMap`] (which the perf-map-update re-route test relies on) and make the
//! `dp` sweep non-deterministic. Integer micros keep every output
//! `Eq`-comparable and the selected chain reproducible.
//!
//! ## Advisory `tau`, measured `rho` (SI-3 / SI-4)
//!
//! `rho` is **measured** by the routing node from its own QUIC paths
//! ([`nexus_core_rs::shard::conn_rtt`]); `tau` is intrinsically **self-reported**
//! by each worker. A member who under-reports its `tau` could bias a selection
//! toward itself (SI-3). The two selectors treat `tau` differently, on purpose:
//!
//! - The **routing DP** ([`route_min_latency`]) *does* use `tau` — it is
//!   choosing the minimum-latency chain and `tau` is its only compute-cost
//!   signal — so `tau` there is **advisory**, not a trust boundary. A worker
//!   that wins routing by lying about `tau` only biases the latency
//!   optimisation; it cannot forge a valid downstream N0-N3 `RunProof`
//!   fingerprint (Phases G-I), which is the real integrity authority.
//! - **Churn fallback** ([`replace_failed_server`] / [`assign_fallback_nodes`])
//!   orders candidates by the **measured `rho` only** ([`fallback_link_cost`]),
//!   with a deterministic `worker_pubkey` tie-break — `tau` is **excluded**, so
//!   a replacement cannot be steered by a self-reported lie (SI-3, binding
//!   constraint #4b).
//!
//! Both selectors only ever draw from the session's **`ComputeGroup`
//! allowlist** (no admission relaxation at churn — SI-4).

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use nexus_core_rs::shard_plan::ShardPlan;

use crate::error::CoordinatorError;

/// Cadence at which the daemon republishes the perf-map to iroh-docs (design
/// addendum §2 "republished every 1-2s"). A named constant rather than a bare
/// literal (README §6.9); consumed by the daemon glue, not by this crate's
/// pure compute. 1.5s sits in the middle of the 1-2s window.
pub const PERF_MAP_REPUBLISH_INTERVAL: Duration = Duration::from_millis(1500);

/// Hard upper bound on the number of `rho` (and, separately, `tau`) entries a
/// single deserialised [`PerfMap`] may carry — a **defence-in-depth count cap**
/// enforced in [`PerfMap`]'s `TryFrom` before the `BTreeMap`s are built (mirror
/// of [`nexus_core_rs::shard_plan::SHARD_PLAN_MAX_ASSIGNMENTS`]). The raw byte
/// size of an inbound perf-map is bounded **upstream** by the signed
/// `FeedEntry` envelope the daemon ingests; this cap is the secondary structural
/// bound on the count, so a within-envelope-but-pathological payload cannot drive
/// an unbounded `BTreeMap` build. A private group is 3-5 machines; even a
/// 256-worker × 80-layer pathological group stays far below this.
pub const PERF_MAP_MAX_ENTRIES: usize = 65_536;

/// Capacity of the bounded client-side activation replay cache used to resume
/// a stage on a worker drop (Petals churn). **This is the bounded churn cache
/// (scope cut #5 in-scope), NOT a distributed / unbounded big-context KV cache
/// (scope cut #5 post-S77).** Eviction is oldest-frontier-first, so a peer
/// that triggers repeated drops cannot grow retained memory without bound.
pub const ACTIVATION_REPLAY_CACHE_MAX: usize = 64;

/// Latency (microseconds) attributed to a `(worker, layer)` pair with no
/// profiled `tau` sample. Large enough to push an unprofiled candidate to the
/// back of the DP / fallback heap, small enough that a saturating sum over a
/// realistic pipeline cannot overflow `u64`. Mirrors
/// [`crate::placement::MISSING_RTT_PENALTY_MICROS`].
pub const MISSING_TAU_PENALTY_MICROS: u64 = 60_000_000;

/// Latency (microseconds) attributed to a worker pair with no measured `rho`
/// sample (e.g. a path the routing node has not probed yet). Same rationale as
/// [`MISSING_TAU_PENALTY_MICROS`].
pub const MISSING_RHO_PENALTY_MICROS: u64 = 60_000_000;

/// The `(rho, tau)` cost snapshot the routing DP consumes.
///
/// `rho_micros` is keyed by the **sorted** pubkey pair (so lookups are
/// symmetric, like [`crate::placement::RttMatrix`]); `tau_micros` is keyed by
/// `(worker_pubkey, layer)`. Both are integer microseconds (see the module
/// note). Serialised as an **unsigned** raw-op via [`PerfMapWire`] — a JSON
/// object cannot carry tuple/array keys directly, so the wire form is two
/// flat, deterministically-ordered lists.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "PerfMapWire", try_from = "PerfMapWire")]
pub struct PerfMap {
    /// Measured round-trip RTT per worker pair, integer microseconds, keyed by
    /// the sorted pubkey pair.
    rho_micros: BTreeMap<([u8; 32], [u8; 32]), u64>,
    /// Self-reported (advisory) per-layer compute latency, integer
    /// microseconds, keyed by `(worker_pubkey, layer)`.
    tau_micros: BTreeMap<([u8; 32], u32), u64>,
}

/// Flat, serde-friendly wire form of a [`PerfMap`] (JSON cannot key an object
/// by a tuple/array). Built from the `BTreeMap`s in sorted order, so the wire
/// bytes are deterministic.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PerfMapWire {
    rho: Vec<RhoEntry>,
    tau: Vec<TauEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RhoEntry {
    a: [u8; 32],
    b: [u8; 32],
    micros: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TauEntry {
    worker: [u8; 32],
    layer: u32,
    micros: u64,
}

impl From<PerfMap> for PerfMapWire {
    fn from(m: PerfMap) -> Self {
        let rho = m
            .rho_micros
            .into_iter()
            .map(|((a, b), micros)| RhoEntry { a, b, micros })
            .collect();
        let tau = m
            .tau_micros
            .into_iter()
            .map(|((worker, layer), micros)| TauEntry {
                worker,
                layer,
                micros,
            })
            .collect();
        PerfMapWire { rho, tau }
    }
}

impl TryFrom<PerfMapWire> for PerfMap {
    type Error = CoordinatorError;

    fn try_from(w: PerfMapWire) -> Result<Self, Self::Error> {
        // Cap BEFORE building the maps so a hostile raw-op cannot blow memory.
        if w.rho.len() > PERF_MAP_MAX_ENTRIES {
            return Err(CoordinatorError::Validation(format!(
                "perf-map has {} rho entries, exceeds PERF_MAP_MAX_ENTRIES={}",
                w.rho.len(),
                PERF_MAP_MAX_ENTRIES
            )));
        }
        if w.tau.len() > PERF_MAP_MAX_ENTRIES {
            return Err(CoordinatorError::Validation(format!(
                "perf-map has {} tau entries, exceeds PERF_MAP_MAX_ENTRIES={}",
                w.tau.len(),
                PERF_MAP_MAX_ENTRIES
            )));
        }
        let mut map = PerfMap::new();
        for e in w.rho {
            map.set_rho_micros(e.a, e.b, e.micros);
        }
        for e in w.tau {
            map.set_tau_micros(e.worker, e.layer, e.micros);
        }
        Ok(map)
    }
}

impl PerfMap {
    /// An empty perf-map (every cost unknown).
    pub fn new() -> Self {
        PerfMap::default()
    }

    fn rho_key(a: [u8; 32], b: [u8; 32]) -> ([u8; 32], [u8; 32]) {
        if a <= b { (a, b) } else { (b, a) }
    }

    /// Record the measured round-trip RTT between two workers (symmetric),
    /// in integer microseconds.
    pub fn set_rho_micros(&mut self, a: [u8; 32], b: [u8; 32], micros: u64) {
        self.rho_micros.insert(Self::rho_key(a, b), micros);
    }

    /// Record the measured round-trip RTT between two workers from a
    /// [`Duration`] (e.g. straight from [`nexus_core_rs::shard::conn_rtt`]),
    /// stored as integer microseconds for exact reproducibility.
    pub fn set_rho(&mut self, a: [u8; 32], b: [u8; 32], rtt: Duration) {
        self.set_rho_micros(a, b, rtt.as_micros().min(u64::MAX as u128) as u64);
    }

    /// Measured RTT (microseconds) between two workers, or `None` if unknown.
    /// `a == b` is defined as `0`.
    pub fn get_rho(&self, a: [u8; 32], b: [u8; 32]) -> Option<u64> {
        if a == b {
            return Some(0);
        }
        self.rho_micros.get(&Self::rho_key(a, b)).copied()
    }

    /// Record a worker's self-reported (advisory) compute latency for a layer,
    /// in integer microseconds.
    pub fn set_tau_micros(&mut self, worker: [u8; 32], layer: u32, micros: u64) {
        self.tau_micros.insert((worker, layer), micros);
    }

    /// A worker's profiled (advisory) compute latency for a layer, or `None`
    /// if unprofiled.
    pub fn get_tau(&self, worker: [u8; 32], layer: u32) -> Option<u64> {
        self.tau_micros.get(&(worker, layer)).copied()
    }

    /// Serialise the perf-map into its **unsigned** raw-op JSON value (the
    /// `FeedEntry.op` body). No Ed25519 signature, no `canonical_bytes`, no
    /// `DOMAIN_*` — authentication of *who* published rides the existing signed
    /// `FeedEntry` envelope (daemon-side). The actual `doc.set()` republish is
    /// daemon glue; this is the data the daemon writes.
    pub fn to_raw_op(&self) -> Result<serde_json::Value, CoordinatorError> {
        Ok(serde_json::to_value(self)?)
    }

    /// Reconstruct a perf-map from its raw-op JSON value, enforcing the
    /// [`PERF_MAP_MAX_ENTRIES`] count cap (in [`PerfMap`]'s `TryFrom`) before the
    /// `BTreeMap`s are built. The raw value itself is bounded upstream by the
    /// signed `FeedEntry` envelope the daemon ingests; this cap bounds the
    /// structural entry count.
    pub fn from_raw_op(value: serde_json::Value) -> Result<Self, CoordinatorError> {
        Ok(serde_json::from_value(value)?)
    }
}

/// One pipeline stage of a routing request: the contiguous layer block to be
/// served, and the set of **candidate** worker pubkeys (replicas) that can
/// serve it. In the no-replica case the candidate list is a single element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingStage {
    /// Inclusive lower bound of this stage's layer block.
    pub layer_start: u32,
    /// Exclusive upper bound of this stage's layer block.
    pub layer_end: u32,
    /// Candidate worker pubkeys able to serve this stage. Each MUST already be
    /// a member of the session's `ComputeGroup` allowlist (routing never widens
    /// admission); the DP picks one per stage.
    pub candidates: Vec<[u8; 32]>,
}

/// A per-request routing problem: the ordered pipeline stages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingRequest {
    /// The stages in pipeline order (stage `i` feeds stage `i+1`).
    pub stages: Vec<RoutingStage>,
}

/// The min-latency chain the DP selected: one worker per stage, plus the total
/// modelled latency in integer microseconds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingChain {
    /// One chosen worker pubkey per stage, in pipeline order.
    pub hops: Vec<[u8; 32]>,
    /// Total modelled latency of the chain, integer microseconds.
    pub total_latency_micros: u64,
}

/// The result of re-routing one stage after a worker drop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReroutedHop {
    /// Which pipeline stage was re-routed.
    pub stage_index: usize,
    /// The replacement worker (always an allowlisted member, never the dropped
    /// one).
    pub replacement: [u8; 32],
    /// The marginal latency (microseconds) the replacement adds at this stage
    /// given its fixed neighbours.
    pub added_latency_micros: u64,
}

/// The stage's representative compute cost: the worker's profiled `tau` at the
/// stage's first layer (a block's interior layers cross no frontier, so they
/// add no `rho`). Missing `tau` is penalised, never panics.
fn stage_tau(perf: &PerfMap, worker: [u8; 32], stage: &RoutingStage) -> u64 {
    perf.get_tau(worker, stage.layer_start)
        .unwrap_or(MISSING_TAU_PENALTY_MICROS)
}

/// Measured link cost between two consecutive-stage workers, penalised if
/// unknown.
fn link_rho(perf: &PerfMap, from: [u8; 32], to: [u8; 32]) -> u64 {
    perf.get_rho(from, to).unwrap_or(MISSING_RHO_PENALTY_MICROS)
}

/// Churn fallback ordering cost: the **measured** `rho` of placing `worker`
/// between its fixed neighbours (the link to the previous and next hops, when
/// they exist).
///
/// **Deliberately excludes the self-reported `tau`** (SI-3, binding constraint
/// #4b / preflight S3-E4): a dropped slot is replaced by the network-closest
/// allowlisted member, and that choice must be **un-gameable** — a worker
/// cannot improve its fallback ranking by under-reporting its compute latency,
/// because only the routing node's own measured RTT counts here. (The routing
/// DP, [`route_min_latency`], *does* use `tau` — it is selecting the
/// minimum-latency chain and `tau` is its only compute-cost signal — but there
/// `tau` is advisory and the integrity authority is the downstream `RunProof`
/// verification, not the cost.)
fn fallback_link_cost(
    perf: &PerfMap,
    worker: [u8; 32],
    prev_hop: Option<[u8; 32]>,
    next_hop: Option<[u8; 32]>,
) -> u64 {
    let mut cost = 0u64;
    if let Some(p) = prev_hop {
        cost = cost.saturating_add(link_rho(perf, p, worker));
    }
    if let Some(n) = next_hop {
        cost = cost.saturating_add(link_rho(perf, worker, n));
    }
    cost
}

/// Select the minimum-latency pipeline chain for `req` under `perf`.
///
/// A single left-to-right sweep dynamic program over the layer-indexed DAG
/// (Parallax phase 2): `dp(s,g') = min_g (dp(s-1,g) + rho<g,g'>) + tau<g',s>`.
/// The layer index topologically orders the DAG, so one forward pass is a
/// valid shortest-path relaxation (no fixed-point iteration). `O(S·R²)` where
/// `R` is the per-stage replica count (3-5 in practice).
///
/// Deterministic: all costs are integer micros and every tie is broken by the
/// smaller `worker_pubkey`, so the same `(req, perf)` always yields the same
/// [`RoutingChain`].
pub fn route_min_latency(
    req: &RoutingRequest,
    perf: &PerfMap,
) -> Result<RoutingChain, CoordinatorError> {
    if req.stages.is_empty() {
        return Err(CoordinatorError::Validation(
            "routing: request has no stages".into(),
        ));
    }
    if let Some(i) = req.stages.iter().position(|s| s.candidates.is_empty()) {
        return Err(CoordinatorError::Validation(format!(
            "routing: stage {i} has no candidate workers"
        )));
    }

    // Cost of reaching each candidate of the current stage, and a backpointer
    // (index into the previous stage's candidate list) per candidate.
    let s0 = &req.stages[0];
    let mut cost: Vec<u64> = s0
        .candidates
        .iter()
        .map(|&c| stage_tau(perf, c, s0))
        .collect();
    // back[s-1][j] = index into stage (s-1)'s candidates chosen for candidate j
    // of stage s.
    let mut back: Vec<Vec<usize>> = Vec::with_capacity(req.stages.len().saturating_sub(1));

    for s in 1..req.stages.len() {
        let stage = &req.stages[s];
        let prev = &req.stages[s - 1];
        let mut new_cost = vec![u64::MAX; stage.candidates.len()];
        let mut new_back = vec![0usize; stage.candidates.len()];

        for (j, &c) in stage.candidates.iter().enumerate() {
            let t = stage_tau(perf, c, stage);
            // Pick the predecessor minimising (cost, predecessor pubkey).
            let mut best: Option<(u64, [u8; 32], usize)> = None;
            for (i, &p) in prev.candidates.iter().enumerate() {
                let cand = cost[i]
                    .saturating_add(link_rho(perf, p, c))
                    .saturating_add(t);
                let replace = match best {
                    Some((bc, bp, _)) => (cand, p) < (bc, bp),
                    None => true,
                };
                if replace {
                    best = Some((cand, p, i));
                }
            }
            let (bc, _, bi) = best.expect("previous stage has >= 1 candidate");
            new_cost[j] = bc;
            new_back[j] = bi;
        }

        back.push(new_back);
        cost = new_cost;
    }

    // Final stage: pick the min-cost candidate, tie-break smaller pubkey.
    let last = req.stages.len() - 1;
    let last_cands = &req.stages[last].candidates;
    let mut best_final: Option<(u64, [u8; 32], usize)> = None;
    for (j, &c) in last_cands.iter().enumerate() {
        let replace = match best_final {
            Some((bc, bp, _)) => (cost[j], c) < (bc, bp),
            None => true,
        };
        if replace {
            best_final = Some((cost[j], c, j));
        }
    }
    let (total, _, mut idx) = best_final.expect("last stage has >= 1 candidate");

    // Walk the backpointers to reconstruct the chain.
    let mut hops_rev = vec![req.stages[last].candidates[idx]];
    for s in (1..req.stages.len()).rev() {
        let prev_idx = back[s - 1][idx];
        hops_rev.push(req.stages[s - 1].candidates[prev_idx]);
        idx = prev_idx;
    }
    hops_rev.reverse();

    Ok(RoutingChain {
        hops: hops_rev,
        total_latency_micros: total,
    })
}

/// Re-route a single dropped stage (Petals active `replace_failed_server`).
///
/// "Active" in the Petals sense: replacing one dropped stage is **independent
/// of the pipeline length** `L` — the rest of the chain is untouched, no full
/// re-plan — unlike a stale-DHT-entry model that re-routes nothing. The
/// per-stage selection scans only that stage's `R` replicas (`R` is 3-5 in a
/// private group) through the fallback heap, so it is `O(R)`, not `O(L)`.
///
/// Given the current `chain` and the worker that just dropped at
/// `stage_index`, pick the next best **allowlisted** candidate for that stage
/// from a fallback heap ordered by the **measured** link cost
/// (`(rho, pubkey)`, see [`fallback_link_cost`]), keeping the neighbouring hops
/// fixed. Only one assignment is replaced — the rest of the pipeline is
/// untouched (active re-balancing, not a full re-plan).
///
/// Admission is never relaxed at churn: the replacement is drawn **only** from
/// `allowlist` (the session's `ComputeGroup` members), so a dropped member is
/// replaced by another Ed25519-admitted member, never an opportunistic
/// outsider (SI-4, scope cut #8). The ordering uses **only the measured `rho`**
/// and never the self-reported `tau`, so the choice is un-gameable and
/// deterministic (SI-3, binding constraint #4b).
pub fn replace_failed_server(
    req: &RoutingRequest,
    chain: &RoutingChain,
    perf: &PerfMap,
    allowlist: &BTreeSet<[u8; 32]>,
    stage_index: usize,
    failed: [u8; 32],
) -> Result<ReroutedHop, CoordinatorError> {
    if req.stages.len() != chain.hops.len() {
        return Err(CoordinatorError::Validation(
            "routing: chain length does not match request stages".into(),
        ));
    }
    if stage_index >= req.stages.len() {
        return Err(CoordinatorError::Validation(format!(
            "routing: stage_index {stage_index} out of range"
        )));
    }
    if chain.hops[stage_index] != failed {
        return Err(CoordinatorError::Validation(
            "routing: failed worker is not the current hop at stage_index".into(),
        ));
    }

    let stage = &req.stages[stage_index];
    let prev_hop = (stage_index > 0).then(|| chain.hops[stage_index - 1]);
    let next_hop = (stage_index + 1 < chain.hops.len()).then(|| chain.hops[stage_index + 1]);

    // Min-heap (via Reverse) over allowlisted candidates != failed, keyed by
    // (measured rho, pubkey) so the pop is the deterministic, un-gameable best
    // fallback (self-reported tau is excluded, SI-3).
    let mut heap: BinaryHeap<Reverse<(u64, [u8; 32])>> = BinaryHeap::new();
    for &c in &stage.candidates {
        if c == failed || !allowlist.contains(&c) {
            continue;
        }
        let cost = fallback_link_cost(perf, c, prev_hop, next_hop);
        heap.push(Reverse((cost, c)));
    }

    match heap.pop() {
        Some(Reverse((added_latency_micros, replacement))) => Ok(ReroutedHop {
            stage_index,
            replacement,
            added_latency_micros,
        }),
        None => Err(CoordinatorError::Validation(format!(
            "routing: no allowlisted fallback for stage {stage_index}"
        ))),
    }
}

/// Populate each assignment's `fallback_node` at **plan time** from the best
/// allowlisted alternative for its stage (the fallback heap), returning a
/// **new** [`ShardPlan`]. An assignment whose stage has **no** allowlisted
/// alternative (the primary is the only candidate / member) keeps
/// `fallback_node == None` — there is genuinely nothing to fall back to.
///
/// This is how the Phase D placement's deliberately-`None` `fallback_node`
/// (`placement.rs` leaves it unset "assigned by Phase E churn handling") gets
/// filled. It does **not** mutate a signed manifest: the caller (the session
/// initiator) wraps the returned plan in a fresh
/// [`nexus_core_rs::shard_plan::ShardedSessionManifest`] with `revision + 1` and
/// re-signs it. The `fallback_node` field is `#[serde(default)] Option`, so a
/// plan with `Some` fallbacks and an all-`None` plan are both valid wire v1 —
/// additive, zero `*_FORMAT_VERSION` bump, zero new `DOMAIN_*`.
///
/// `req.stages` must align with `plan.assignments` by index (both in pipeline
/// order).
pub fn assign_fallback_nodes(
    plan: &ShardPlan,
    req: &RoutingRequest,
    perf: &PerfMap,
    allowlist: &BTreeSet<[u8; 32]>,
) -> Result<ShardPlan, CoordinatorError> {
    if plan.assignments.len() != req.stages.len() {
        return Err(CoordinatorError::Validation(
            "routing: plan assignments do not align with request stages".into(),
        ));
    }

    let n = plan.assignments.len();
    let mut out = Vec::with_capacity(n);
    for (i, assignment) in plan.assignments.iter().enumerate() {
        let stage = &req.stages[i];
        let primary = assignment.worker_pubkey;
        let prev_hop = (i > 0).then(|| plan.assignments[i - 1].worker_pubkey);
        let next_hop = (i + 1 < n).then(|| plan.assignments[i + 1].worker_pubkey);

        // Best allowlisted alternative for this stage, ordered by measured rho
        // then pubkey (deterministic, un-gameable — no self-reported tau),
        // excluding the primary.
        let mut heap: BinaryHeap<Reverse<(u64, [u8; 32])>> = BinaryHeap::new();
        for &c in &stage.candidates {
            if c == primary || !allowlist.contains(&c) {
                continue;
            }
            let cost = fallback_link_cost(perf, c, prev_hop, next_hop);
            heap.push(Reverse((cost, c)));
        }
        let fallback_node = heap.pop().map(|Reverse((_, pubkey))| pubkey);

        let mut next = assignment.clone();
        next.fallback_node = fallback_node;
        out.push(next);
    }

    Ok(ShardPlan::new(out))
}

/// A bounded, client-side ring of recent boundary activations, used to resume a
/// pipeline stage on a worker drop (Petals churn).
///
/// Keyed by the **frontier layer** (a stage's `layer_end`, where the activation
/// crosses to the next stage). Capacity is hard-bounded by
/// [`ACTIVATION_REPLAY_CACHE_MAX`] with oldest-frontier-first eviction, so this
/// is the **bounded churn cache** of scope cut #5 — never the unbounded /
/// distributed big-context KV cache (post-S77). It stays local (no iroh-docs
/// distribution of activations).
#[derive(Debug, Clone)]
pub struct ActivationReplayCache {
    cap: usize,
    /// Frontier layers in insertion order (front = oldest), for eviction.
    order: VecDeque<u32>,
    frames: BTreeMap<u32, Vec<u8>>,
}

impl Default for ActivationReplayCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ActivationReplayCache {
    /// A cache bounded by [`ACTIVATION_REPLAY_CACHE_MAX`].
    pub fn new() -> Self {
        Self::with_capacity(ACTIVATION_REPLAY_CACHE_MAX)
    }

    /// A cache with an explicit capacity (`>= 1`; `0` is clamped to `1` so an
    /// insert always retains the most recent frontier).
    pub fn with_capacity(cap: usize) -> Self {
        ActivationReplayCache {
            cap: cap.max(1),
            order: VecDeque::new(),
            frames: BTreeMap::new(),
        }
    }

    /// Cache the boundary activation produced at `frontier_layer`. Re-inserting
    /// the same frontier refreshes its bytes without changing its age. When the
    /// cache is full, the oldest frontier is evicted first.
    pub fn insert(&mut self, frontier_layer: u32, activation: Vec<u8>) {
        if self.frames.insert(frontier_layer, activation).is_some() {
            // Refresh in place — already present, age unchanged.
            return;
        }
        self.order.push_back(frontier_layer);
        while self.order.len() > self.cap {
            if let Some(evicted) = self.order.pop_front() {
                self.frames.remove(&evicted);
            }
        }
    }

    /// The cached boundary activation for `frontier_layer`, if still retained,
    /// so a replacement worker can replay the stage and the inference
    /// continues.
    pub fn get(&self, frontier_layer: u32) -> Option<&[u8]> {
        self.frames.get(&frontier_layer).map(|v| v.as_slice())
    }

    /// Number of frontiers currently cached.
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// Whether the cache holds no frontier.
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use nexus_core_rs::crypto::KeyPair;
    use nexus_core_rs::shard_plan::{
        KvCachePolicy, ShardAssignment, ShardRole, ShardedSessionManifestEntry,
    };

    fn pk(byte: u8) -> [u8; 32] {
        [byte; 32]
    }

    fn allowlist(bytes: &[u8]) -> BTreeSet<[u8; 32]> {
        bytes.iter().map(|&b| pk(b)).collect()
    }

    /// A two-replicas-per-stage, three-stage request over workers 1..6.
    fn three_stage_req() -> RoutingRequest {
        RoutingRequest {
            stages: vec![
                RoutingStage {
                    layer_start: 0,
                    layer_end: 10,
                    candidates: vec![pk(1), pk(2)],
                },
                RoutingStage {
                    layer_start: 10,
                    layer_end: 20,
                    candidates: vec![pk(3), pk(4)],
                },
                RoutingStage {
                    layer_start: 20,
                    layer_end: 30,
                    candidates: vec![pk(5), pk(6)],
                },
            ],
        }
    }

    #[test]
    fn routing_dag_sweep_selects_min_latency_chain() {
        // Make the chain 1 -> 3 -> 5 strictly cheapest. Give every node a flat
        // tau, then make the rho along 1->3->5 small and every other link big.
        let req = three_stage_req();
        let mut perf = PerfMap::new();
        for w in [1u8, 2, 3, 4, 5, 6] {
            // tau at each stage's first layer (0, 10, 20).
            for layer in [0u32, 10, 20] {
                perf.set_tau_micros(pk(w), layer, 1_000);
            }
        }
        // Cheap spine 1->3->5.
        perf.set_rho_micros(pk(1), pk(3), 1_000);
        perf.set_rho_micros(pk(3), pk(5), 1_000);
        // Expensive everything else that the DP could pick.
        perf.set_rho_micros(pk(1), pk(4), 50_000);
        perf.set_rho_micros(pk(2), pk(3), 50_000);
        perf.set_rho_micros(pk(2), pk(4), 50_000);
        perf.set_rho_micros(pk(3), pk(6), 50_000);
        perf.set_rho_micros(pk(4), pk(5), 50_000);
        perf.set_rho_micros(pk(4), pk(6), 50_000);

        let chain = route_min_latency(&req, &perf).expect("route");
        assert_eq!(chain.hops, vec![pk(1), pk(3), pk(5)]);
        // 3 tau (1000*3) + 2 rho (1000*2).
        assert_eq!(chain.total_latency_micros, 3_000 + 2_000);
        // Deterministic across calls.
        assert_eq!(route_min_latency(&req, &perf).unwrap(), chain);
    }

    #[test]
    fn routing_tie_breaks_on_smaller_pubkey() {
        // Two equal-cost chains: the deterministic tie-break must pick the one
        // ending (and routing through) the smaller pubkey.
        let req = RoutingRequest {
            stages: vec![
                RoutingStage {
                    layer_start: 0,
                    layer_end: 1,
                    candidates: vec![pk(2), pk(1)],
                },
                RoutingStage {
                    layer_start: 1,
                    layer_end: 2,
                    candidates: vec![pk(4), pk(3)],
                },
            ],
        };
        let mut perf = PerfMap::new();
        for w in [1u8, 2, 3, 4] {
            perf.set_tau_micros(pk(w), 0, 1_000);
            perf.set_tau_micros(pk(w), 1, 1_000);
        }
        // Every link identical -> all chains cost the same; tie-break decides.
        for a in [1u8, 2] {
            for b in [3u8, 4] {
                perf.set_rho_micros(pk(a), pk(b), 5_000);
            }
        }
        let chain = route_min_latency(&req, &perf).unwrap();
        assert_eq!(chain.hops, vec![pk(1), pk(3)], "smallest-pubkey chain wins");
    }

    #[test]
    fn route_single_stage_returns_min_tau_candidate() {
        let req = RoutingRequest {
            stages: vec![RoutingStage {
                layer_start: 0,
                layer_end: 8,
                candidates: vec![pk(1), pk(2), pk(3)],
            }],
        };
        let mut perf = PerfMap::new();
        perf.set_tau_micros(pk(1), 0, 9_000);
        perf.set_tau_micros(pk(2), 0, 3_000);
        perf.set_tau_micros(pk(3), 0, 7_000);
        let chain = route_min_latency(&req, &perf).unwrap();
        assert_eq!(chain.hops, vec![pk(2)]);
        assert_eq!(chain.total_latency_micros, 3_000);
    }

    #[test]
    fn routing_rejects_empty_or_candidateless() {
        let empty = RoutingRequest { stages: vec![] };
        assert!(route_min_latency(&empty, &PerfMap::new()).is_err());

        let no_cands = RoutingRequest {
            stages: vec![RoutingStage {
                layer_start: 0,
                layer_end: 1,
                candidates: vec![],
            }],
        };
        assert!(route_min_latency(&no_cands, &PerfMap::new()).is_err());
    }

    #[test]
    fn routing_missing_costs_penalised_not_panicking() {
        // No perf data at all — the sweep must still return a deterministic
        // chain (everything penalised equally) without panicking.
        let req = three_stage_req();
        let perf = PerfMap::new();
        let chain = route_min_latency(&req, &perf).expect("route under penalties");
        assert_eq!(chain.hops.len(), 3);
        assert_eq!(route_min_latency(&req, &perf).unwrap(), chain);
    }

    #[test]
    fn routing_recomputed_on_perf_map_update() {
        // First perf-map favours the 1->3->5 spine; an update makes 2->4->6
        // strictly cheaper, so the recomputed chain must change.
        let req = three_stage_req();
        let mut perf = PerfMap::new();
        for w in [1u8, 2, 3, 4, 5, 6] {
            for layer in [0u32, 10, 20] {
                perf.set_tau_micros(pk(w), layer, 1_000);
            }
        }
        perf.set_rho_micros(pk(1), pk(3), 1_000);
        perf.set_rho_micros(pk(3), pk(5), 1_000);
        perf.set_rho_micros(pk(2), pk(4), 90_000);
        perf.set_rho_micros(pk(4), pk(6), 90_000);
        let first = route_min_latency(&req, &perf).unwrap();
        assert_eq!(first.hops, vec![pk(1), pk(3), pk(5)]);

        // Update: collapse the 2->4->6 links, blow up the 1->3->5 spine.
        let mut perf2 = perf.clone();
        perf2.set_rho_micros(pk(1), pk(3), 90_000);
        perf2.set_rho_micros(pk(3), pk(5), 90_000);
        perf2.set_rho_micros(pk(2), pk(4), 500);
        perf2.set_rho_micros(pk(4), pk(6), 500);
        assert_ne!(perf, perf2, "the perf-map actually changed");

        let second = route_min_latency(&req, &perf2).unwrap();
        assert_eq!(second.hops, vec![pk(2), pk(4), pk(6)]);
        assert_ne!(first.hops, second.hops, "a perf-map update re-routes");
    }

    #[test]
    fn churn_replaces_failed_server_oturn() {
        // Route 1->3->5, then drop the middle worker (3). The fallback for that
        // stage is 4; the cached boundary activation lets the inference resume.
        let req = three_stage_req();
        let mut perf = PerfMap::new();
        for w in [1u8, 2, 3, 4, 5, 6] {
            for layer in [0u32, 10, 20] {
                perf.set_tau_micros(pk(w), layer, 1_000);
            }
        }
        perf.set_rho_micros(pk(1), pk(3), 1_000);
        perf.set_rho_micros(pk(3), pk(5), 1_000);
        // Fallback 4's measured links exist but are pricier than 3's.
        perf.set_rho_micros(pk(1), pk(4), 4_000);
        perf.set_rho_micros(pk(4), pk(5), 4_000);

        let group = allowlist(&[1, 2, 3, 4, 5, 6]);
        let chain = route_min_latency(&req, &perf).unwrap();
        assert_eq!(chain.hops, vec![pk(1), pk(3), pk(5)]);

        // Upstream cached the boundary activation at the frontier layer 10.
        let mut cache = ActivationReplayCache::new();
        cache.insert(10, b"hidden-state-at-frontier-10".to_vec());

        let rerouted = replace_failed_server(&req, &chain, &perf, &group, 1, pk(3))
            .expect("a fallback exists");
        assert_eq!(rerouted.stage_index, 1);
        assert_eq!(
            rerouted.replacement,
            pk(4),
            "the only other allowlisted candidate"
        );
        assert!(
            group.contains(&rerouted.replacement),
            "replacement stays in the group"
        );
        // The inference can resume: the replacement replays the cached frontier.
        assert_eq!(
            cache.get(10),
            Some(b"hidden-state-at-frontier-10".as_slice()),
            "the bounded cache lets the new server resume the stage"
        );
    }

    #[test]
    fn replace_failed_server_only_draws_from_allowlist() {
        // Stage 1 candidates are {3, 4}; 3 drops; 4 is NOT in the allowlist, so
        // no allowlisted fallback exists -> error (no admission relaxation).
        let req = three_stage_req();
        let perf = PerfMap::new();
        let chain = RoutingChain {
            hops: vec![pk(1), pk(3), pk(5)],
            total_latency_micros: 0,
        };
        // Allowlist deliberately omits 4 (the only alternative at stage 1).
        let group = allowlist(&[1, 3, 5]);
        let err = replace_failed_server(&req, &chain, &perf, &group, 1, pk(3))
            .expect_err("no allowlisted fallback");
        assert!(matches!(err, CoordinatorError::Validation(_)));
    }

    #[test]
    fn replace_failed_server_orders_by_measured_rho_not_self_tau() {
        // The discriminating case (binding constraint #4b): the values are
        // chosen so that INCLUDING self-reported tau would pick the liar, but
        // ordering by MEASURED rho only picks the honest node. The assertion
        // therefore proves tau is *excluded* from churn selection, not merely
        // dominated.
        //   liar 9 : tau = 1      (lie),    rho(prev,9) = 10_000
        //   honest 8: tau = 9_000 (honest), rho(prev,8) =  5_000
        //   tau+rho : liar 10_001 < honest 14_000  -> liar would win
        //   rho-only: liar 10_000 > honest  5_000  -> honest wins (correct)
        let req = RoutingRequest {
            stages: vec![
                RoutingStage {
                    layer_start: 0,
                    layer_end: 1,
                    candidates: vec![pk(1)],
                },
                RoutingStage {
                    layer_start: 1,
                    layer_end: 2,
                    candidates: vec![pk(2), pk(9), pk(8)], // 2 = primary, 9 = liar, 8 = honest
                },
            ],
        };
        let mut perf = PerfMap::new();
        perf.set_tau_micros(pk(1), 0, 1_000);
        perf.set_tau_micros(pk(2), 1, 1_000);
        // Liar 9: near-zero self-reported tau, but a pricier measured link.
        perf.set_tau_micros(pk(9), 1, 1);
        perf.set_rho_micros(pk(1), pk(9), 10_000);
        // Honest 8: larger tau, but a genuinely cheaper measured link.
        perf.set_tau_micros(pk(8), 1, 9_000);
        perf.set_rho_micros(pk(1), pk(8), 5_000);

        let chain = RoutingChain {
            hops: vec![pk(1), pk(2)],
            total_latency_micros: 0,
        };
        let group = allowlist(&[1, 2, 8, 9]);
        let rerouted = replace_failed_server(&req, &chain, &perf, &group, 1, pk(2)).unwrap();
        assert_eq!(
            rerouted.replacement,
            pk(8),
            "churn must order by measured rho only; a tau lie must not win"
        );
        // The honest node's measured link cost is what was recorded.
        assert_eq!(rerouted.added_latency_micros, 5_000);
    }

    #[test]
    fn replace_failed_server_rejects_wrong_failed_hop() {
        let req = three_stage_req();
        let chain = RoutingChain {
            hops: vec![pk(1), pk(3), pk(5)],
            total_latency_micros: 0,
        };
        let group = allowlist(&[1, 2, 3, 4, 5, 6]);
        // pk(9) is not the current hop at stage 1.
        assert!(replace_failed_server(&req, &chain, &perf_empty(), &group, 1, pk(9)).is_err());
        // stage_index out of range.
        assert!(replace_failed_server(&req, &chain, &perf_empty(), &group, 9, pk(3)).is_err());
    }

    #[test]
    fn replace_failed_server_rejects_chain_length_mismatch() {
        // The chain must have exactly one hop per request stage; a mismatch is
        // a caller bug, rejected before any heap work (Codex Phase E round 2).
        let req = three_stage_req(); // 3 stages
        let short_chain = RoutingChain {
            hops: vec![pk(1), pk(3)], // only 2 hops
            total_latency_micros: 0,
        };
        let group = allowlist(&[1, 2, 3, 4, 5, 6]);
        assert!(
            replace_failed_server(&req, &short_chain, &perf_empty(), &group, 1, pk(3)).is_err(),
            "a chain whose length != request stages must be rejected"
        );
    }

    fn perf_empty() -> PerfMap {
        PerfMap::new()
    }

    #[test]
    fn assign_fallback_nodes_populates_and_resign_verifies() {
        // A placement-style plan (all fallback_node = None) gets fallbacks
        // assigned, then the initiator re-signs a NEW manifest with revision+1
        // and it verifies (no in-place mutation of a signed manifest).
        let initiator = KeyPair::generate();
        let w1 = KeyPair::generate();
        let w3 = KeyPair::generate();
        let alt = KeyPair::generate();

        let mk = |w: [u8; 32], start: u32, end: u32| ShardAssignment {
            worker_pubkey: w,
            layer_start: start,
            layer_end: end,
            role: ShardRole::LayerWorker,
            shard_hashes: vec![[7u8; 32]],
            kv_cache_policy: KvCachePolicy::LocalEphemeral,
            fallback_node: None,
            launch_profile_hash: [9u8; 32],
        };
        let plan = ShardPlan::new(vec![
            mk(w1.public_bytes(), 0, 16),
            mk(w3.public_bytes(), 16, 32),
        ]);
        assert!(plan.assignments.iter().all(|a| a.fallback_node.is_none()));

        let req = RoutingRequest {
            stages: vec![
                RoutingStage {
                    layer_start: 0,
                    layer_end: 16,
                    candidates: vec![w1.public_bytes()],
                },
                RoutingStage {
                    layer_start: 16,
                    layer_end: 32,
                    candidates: vec![w3.public_bytes(), alt.public_bytes()],
                },
            ],
        };
        let mut perf = PerfMap::new();
        perf.set_tau_micros(alt.public_bytes(), 16, 1_000);
        perf.set_rho_micros(w1.public_bytes(), alt.public_bytes(), 2_000);
        let group: BTreeSet<[u8; 32]> = [w1.public_bytes(), w3.public_bytes(), alt.public_bytes()]
            .into_iter()
            .collect();

        let with_fb = assign_fallback_nodes(&plan, &req, &perf, &group).unwrap();
        assert_eq!(
            with_fb.assignments[0].fallback_node, None,
            "stage 0 has no alternative"
        );
        assert_eq!(
            with_fb.assignments[1].fallback_node,
            Some(alt.public_bytes()),
            "stage 1 gets its only allowlisted alternative as fallback"
        );

        // Re-plan: sign a NEW manifest with revision + 1, it verifies.
        let manifest = nexus_core_rs::shard_plan::ShardedSessionManifest::new(
            initiator.public_bytes(),
            "session-e",
            "pilot-70b",
            2, // revision bumped from the original 1
            with_fb,
            [1u8; 32],
            [2u8; 32],
            [3u8; 32],
        );
        let entry = ShardedSessionManifestEntry::sign(manifest, &initiator).unwrap();
        entry
            .verify_signature()
            .expect("a re-signed manifest with populated fallbacks verifies");
    }

    #[test]
    fn assign_fallback_nodes_rejects_misaligned_lengths() {
        let plan = ShardPlan::new(vec![]);
        let req = three_stage_req();
        assert!(assign_fallback_nodes(&plan, &req, &PerfMap::new(), &BTreeSet::new()).is_err());
    }

    #[test]
    fn perf_map_republished_to_doc() {
        // Reformulated (Phase E preflight S4-F2): the coordinator crate has NO
        // iroh-docs handle, so "republished to the doc" is exercised as the
        // raw-op (de)serialise round-trip the daemon would write/read. The
        // literal doc.set() is daemon glue.
        let mut perf = PerfMap::new();
        perf.set_rho_micros(pk(1), pk(2), 1_500);
        perf.set_rho_micros(pk(2), pk(3), 2_500);
        perf.set_tau_micros(pk(1), 0, 900);
        perf.set_tau_micros(pk(3), 20, 1_100);

        // Round-trip through the raw-op JSON value (the FeedEntry.op body).
        let value = perf.to_raw_op().expect("serialise raw-op");
        assert!(value.is_object(), "the raw-op is a JSON object");
        let back = PerfMap::from_raw_op(value).expect("deserialise raw-op");
        assert_eq!(back, perf, "the perf-map round-trips byte-stably");

        // And through bytes (what actually lands on the doc).
        let bytes = serde_json::to_vec(&perf).unwrap();
        let from_bytes: PerfMap = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(from_bytes, perf);

        // Symmetric lookups survive the round-trip.
        assert_eq!(back.get_rho(pk(2), pk(1)), Some(1_500));
        assert_eq!(back.get_tau(pk(3), 20), Some(1_100));
    }

    #[test]
    fn perf_map_raw_op_rejects_oversized_rho() {
        // A hostile raw-op declaring more rho entries than the cap is rejected
        // on the count cap before the BTreeMaps are built.
        let rho: Vec<serde_json::Value> = (0..(PERF_MAP_MAX_ENTRIES + 1))
            .map(|_| {
                serde_json::json!({
                    "a": vec![0u8; 32],
                    "b": vec![1u8; 32],
                    "micros": 1u64,
                })
            })
            .collect();
        let value = serde_json::json!({ "rho": rho, "tau": [] });
        let err = PerfMap::from_raw_op(value).expect_err("over-cap raw-op rejected");
        assert!(
            err.to_string().contains("exceeds"),
            "rejected on the DoS cap"
        );
    }

    #[test]
    fn perf_map_raw_op_rejects_oversized_tau() {
        // The separate tau-count branch is enforced too (Codex Phase E round 1).
        let tau: Vec<serde_json::Value> = (0..(PERF_MAP_MAX_ENTRIES + 1))
            .map(|i| {
                serde_json::json!({
                    "worker": vec![0u8; 32],
                    "layer": (i % 256) as u32,
                    "micros": 1u64,
                })
            })
            .collect();
        let value = serde_json::json!({ "rho": [], "tau": tau });
        let err = PerfMap::from_raw_op(value).expect_err("over-cap tau raw-op rejected");
        assert!(
            err.to_string().contains("exceeds"),
            "rejected on the tau DoS cap"
        );
    }

    #[test]
    fn perf_map_set_rho_is_symmetric() {
        let mut perf = PerfMap::new();
        perf.set_rho(pk(5), pk(1), Duration::from_micros(3_333));
        assert_eq!(perf.get_rho(pk(1), pk(5)), Some(3_333));
        assert_eq!(perf.get_rho(pk(5), pk(1)), Some(3_333));
        assert_eq!(perf.get_rho(pk(7), pk(7)), Some(0), "self-distance is zero");
        assert_eq!(perf.get_rho(pk(2), pk(9)), None, "unknown pair is None");
    }

    #[test]
    fn activation_replay_cache_is_bounded() {
        let mut cache = ActivationReplayCache::with_capacity(3);
        for layer in 0u32..5 {
            cache.insert(layer, vec![layer as u8; 4]);
        }
        // Only the last 3 frontiers survive (0 and 1 evicted oldest-first).
        assert_eq!(cache.len(), 3);
        assert!(cache.get(0).is_none(), "oldest frontier evicted");
        assert!(cache.get(1).is_none());
        assert_eq!(cache.get(2), Some([2u8; 4].as_slice()));
        assert_eq!(cache.get(4), Some([4u8; 4].as_slice()));
        assert!(!cache.is_empty());
    }

    #[test]
    fn activation_replay_cache_refresh_keeps_age() {
        // Re-inserting an existing frontier refreshes bytes without making it
        // "newer" for eviction purposes.
        let mut cache = ActivationReplayCache::with_capacity(2);
        cache.insert(10, vec![1]);
        cache.insert(20, vec![2]);
        cache.insert(10, vec![9]); // refresh 10's bytes, age unchanged
        cache.insert(30, vec![3]); // must evict the oldest (10), not 20
        assert!(
            cache.get(10).is_none(),
            "refreshed-in-place frontier kept its age"
        );
        assert_eq!(cache.get(20), Some([2u8].as_slice()));
        assert_eq!(cache.get(30), Some([3u8].as_slice()));
    }

    #[test]
    fn perf_map_republish_interval_in_window() {
        // The named cadence sits inside the 1-2s design window (addendum §2).
        assert!(PERF_MAP_REPUBLISH_INTERVAL >= Duration::from_secs(1));
        assert!(PERF_MAP_REPUBLISH_INTERVAL <= Duration::from_secs(2));
    }
}
