// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 77 Phase D — Parallax placement scheduler (phase 1).
//!
//! This is the **placement** half of the two-phase Parallax-style scheduler
//! (the per-request **routing** half is Phase E). It is a pure in-memory
//! computation run by the session **initiator**: given the candidate workers
//! that a private [`nexus_core_rs::compute_group::ComputeGroup`] has already
//! admitted (their **measured** free VRAM and a **measured** pairwise RTT
//! matrix), it decides how to split a model too large for any single GPU into
//! a contiguous pipeline of layer blocks and emits a
//! [`nexus_core_rs::shard_plan::ShardPlan`].
//!
//! ## What it is NOT
//!
//! - It signs nothing and touches no wire format. The [`ShardPlan`] it
//!   produces is wrapped (and signed) later by the initiator inside a
//!   [`nexus_core_rs::shard_plan::ShardedSessionManifest`] (already additive
//!   from Phase C). There is **zero new `DOMAIN_*`, zero `*_FORMAT_VERSION`
//!   bump** here — placement is internal compute (Sprint 77 §17 pre-launch
//!   policy holds).
//! - It does not read live VRAM from a worker, nor touch the runtime
//!   admission pump (`nexus-worker-core`'s `consent.rs` `estimated_vram_mb`
//!   is unchanged): the
//!   measured `vram_free_bytes` is consumed at **placement** only (S77 scope
//!   cut #7). The caller builds [`WorkerPlacementProfile`]s from the worker's
//!   `GpuStats.vram_free_bytes` and from [`nexus_core_rs::shard::conn_rtt`].
//!
//! ## Why floats are allowed *here*
//!
//! The repo's no-float rule binds only **signed JCS payloads** (see
//! [`nexus_core_rs::shard_plan`]'s module note): an `f64` cannot round-trip
//! bit-identically across platforms, so it must never reach a canonical
//! pre-image. This module is the opposite case — a scheduling computation in
//! memory whose **output is 100% integer** (`ShardAssignment.layer_start /
//! layer_end : u32`, `[u8; 32]` hashes). We therefore keep the arithmetic
//! integer-exact anyway (largest-remainder apportionment, integer-microsecond
//! RTT), so the produced [`ShardPlan`] is deterministic and `Eq`-comparable
//! in tests; no float leaks into the plan.
//!
//! ## Anti-recentralisation
//!
//! Layer grouping uses **k-medoids on the measured pairwise RTT matrix only**
//! (D3): there is no geo-IP table, ASN lookup or central region authority.
//! The k-medoids initialisation is the deterministic PAM BUILD pass (no
//! randomness), so the same inputs always yield the same plan.

use std::collections::BTreeMap;
use std::time::Duration;

use nexus_core_rs::shard_plan::{
    KvCachePolicy, SHARD_PLAN_MAX_ASSIGNMENTS, ShardAssignment, ShardPlan, ShardRole,
};

use crate::error::CoordinatorError;

/// Minimum number of workers a sharded plan must span. Below this the model
/// either fits a single worker (returned as
/// [`PlacementOutcome::EndpointFederation`]) or the aggregate VRAM is
/// insufficient (an error). A "shard" of one worker is a contradiction.
pub const MIN_SHARD_WORKERS: usize = 2;

/// Default number of k-medoids clusters used to group low-RTT workers into
/// consecutive pipeline positions. Clamped to `[1, n_workers]`. The fan-out
/// is 3-5 machines in practice (addendum §1); two clusters already separate
/// a low-RTT neighbourhood from the rest. Phase E may tune this per request.
pub const KMEDOIDS_DEFAULT_K: usize = 2;

/// Upper bound on k-medoids swap iterations — a convergence safety net. PAM
/// on 3-5 points converges in a handful of swaps; the bound only guards a
/// pathological matrix from looping.
pub const KMEDOIDS_MAX_ITER: usize = 64;

/// RTT (microseconds) attributed to a worker pair with no measured sample
/// yet (`conn_rtt` returned `None`, e.g. a freshly opened connection). Large
/// enough to behave as "effectively unreachable" so the clusterer pushes
/// such a pair apart, small enough that a sum over the cap of workers cannot
/// overflow `u64` (`60s × 256 ≪ u64::MAX`).
pub const MISSING_RTT_PENALTY_MICROS: u64 = 60_000_000;

/// A candidate worker for placement, built by the caller from local
/// measurements. **Internal, non-wire, unsigned** — deliberately not
/// `Serialize` for the network and deliberately *not* named `WorkerCapability`
/// (which would suggest a signed wire payload; none exists).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerPlacementProfile {
    /// Ed25519 public key of the worker (already a member of the session's
    /// `ComputeGroup` allowlist — placement never widens admission).
    pub worker_pubkey: [u8; 32],

    /// **Measured** free VRAM in bytes (`GpuStats.vram_free_bytes` on the
    /// worker, not `GpuInfo` which only carries the total). The placement
    /// reads this measured value, never a declared estimate.
    pub vram_free_bytes: u64,

    /// BLAKE3 hash-pin(s) of the shard weight artifact(s) this worker would
    /// load; copied verbatim into the emitted [`ShardAssignment`].
    pub shard_hashes: Vec<[u8; 32]>,

    /// BLAKE3 hash of the launch profile this worker would boot with; copied
    /// into the emitted [`ShardAssignment`].
    pub launch_profile_hash: [u8; 32],
}

/// A symmetric pairwise round-trip-time matrix, in integer microseconds.
///
/// Entries come from [`nexus_core_rs::shard::conn_rtt`] (a `Duration`); a
/// missing pair (no sample yet) reads back as `None` and is treated as
/// [`MISSING_RTT_PENALTY_MICROS`] by the clusterer.
#[derive(Debug, Clone, Default)]
pub struct RttMatrix {
    /// Keyed by the *sorted* pubkey pair so lookups are symmetric.
    entries: BTreeMap<([u8; 32], [u8; 32]), u64>,
}

impl RttMatrix {
    /// An empty matrix (every pair unknown).
    pub fn new() -> Self {
        RttMatrix {
            entries: BTreeMap::new(),
        }
    }

    fn key(a: [u8; 32], b: [u8; 32]) -> ([u8; 32], [u8; 32]) {
        if a <= b { (a, b) } else { (b, a) }
    }

    /// Record the measured RTT between two workers (symmetric). The
    /// `Duration` is stored as integer microseconds for exact reproducibility.
    pub fn set(&mut self, a: [u8; 32], b: [u8; 32], rtt: Duration) {
        self.set_micros(a, b, rtt.as_micros().min(u64::MAX as u128) as u64);
    }

    /// Record the measured RTT in integer microseconds (symmetric).
    pub fn set_micros(&mut self, a: [u8; 32], b: [u8; 32], micros: u64) {
        self.entries.insert(Self::key(a, b), micros);
    }

    /// Measured RTT (microseconds) between two workers, or `None` if no
    /// sample has been recorded. `a == b` is defined as `0`.
    pub fn get_micros(&self, a: [u8; 32], b: [u8; 32]) -> Option<u64> {
        if a == b {
            return Some(0);
        }
        self.entries.get(&Self::key(a, b)).copied()
    }
}

/// The model being placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelSpec {
    /// Total number of transformer layers to cover, `[0..total_layers)`.
    pub total_layers: u32,
    /// Quantized in-VRAM footprint of the whole model, in bytes (e.g. a 70B
    /// Q4 ≈ 40 GiB). The sharding threshold compares this against the
    /// largest single worker's measured free VRAM.
    pub quantized_vram_bytes: u64,
}

/// The result of a placement decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlacementOutcome {
    /// The model is too large for any single worker and was split into a
    /// pipeline. The plan is guaranteed gap-free, non-overlapping and to
    /// cover exactly `[0..total_layers)`.
    Sharded(ShardPlan),
    /// The model fits a single worker's measured free VRAM — no shard. The
    /// caller falls back to S76 endpoint federation (one worker, one model),
    /// which is simpler and avoids the latency cost of frontier crossings.
    EndpointFederation,
}

/// Decide a placement for `model` across `candidates`, grouping by the
/// measured `rtt` and binding the anti-Sybil candidate sampling to
/// `session_id`.
///
/// Returns:
/// - [`PlacementOutcome::EndpointFederation`] when the model fits the largest
///   single worker's measured free VRAM (sharding threshold, addendum §5);
/// - [`PlacementOutcome::Sharded`] with a contiguous full-coverage plan
///   otherwise;
/// - [`CoordinatorError::Validation`] on degenerate input (no candidates,
///   zero-layer / zero-byte model) or when the aggregate measured VRAM
///   cannot hold the model.
pub fn plan_placement(
    candidates: &[WorkerPlacementProfile],
    rtt: &RttMatrix,
    model: &ModelSpec,
    session_id: &str,
) -> Result<PlacementOutcome, CoordinatorError> {
    if candidates.is_empty() {
        return Err(CoordinatorError::Validation(
            "placement: no candidate workers".into(),
        ));
    }
    if model.total_layers == 0 {
        return Err(CoordinatorError::Validation(
            "placement: model has zero layers".into(),
        ));
    }
    if model.quantized_vram_bytes == 0 {
        return Err(CoordinatorError::Validation(
            "placement: model has zero VRAM footprint".into(),
        ));
    }

    // Sharding threshold (addendum §5): only shard when the model does not
    // fit the largest single worker's MEASURED free VRAM. Otherwise endpoint
    // federation (S76) is simpler and lower-latency.
    let max_free = candidates
        .iter()
        .map(|w| w.vram_free_bytes)
        .max()
        .expect("candidates non-empty");
    if model.quantized_vram_bytes <= max_free {
        return Ok(PlacementOutcome::EndpointFederation);
    }

    // Per-layer VRAM cost (ceil), so a worker's layer capacity never claims
    // VRAM it does not have.
    let per_layer_bytes = model
        .quantized_vram_bytes
        .div_ceil(model.total_layers as u64)
        .max(1);

    // (c) SYBIL-SEEDER-TAIL absorption + capacity coverage: choose the
    // minimal set of workers whose measured VRAM covers the model, ordered
    // by capacity then by an anti-crowding sampling key (never lexicographic
    // pubkey order).
    let selected = select_candidates(candidates, model.total_layers, per_layer_bytes, session_id)?;
    if selected.len() < MIN_SHARD_WORKERS {
        return Err(CoordinatorError::Validation(
            "placement: fewer than two workers required to shard".into(),
        ));
    }
    if selected.len() > SHARD_PLAN_MAX_ASSIGNMENTS {
        return Err(CoordinatorError::Validation(
            "placement: more shards than SHARD_PLAN_MAX_ASSIGNMENTS".into(),
        ));
    }

    // (b) Order the selected workers into a pipeline so low-RTT peers hold
    // consecutive layer blocks (most frontier crossings stay intra-cluster).
    let order = cluster_order_by_rtt(&selected, rtt, KMEDOIDS_DEFAULT_K);
    let ordered: Vec<&WorkerPlacementProfile> = order.iter().map(|&i| &selected[i]).collect();

    // (a) Water-fill: distribute the layers across the ordered workers in
    // proportion to their measured free VRAM, capped by each worker's layer
    // capacity, summing to exactly total_layers.
    let alloc = water_fill(&ordered, model.total_layers, per_layer_bytes, session_id)?;

    // Assemble contiguous assignments in pipeline order, dropping any worker
    // that ended up with zero layers.
    let mut assignments = Vec::new();
    let mut cursor: u32 = 0;
    for (w, &layers) in ordered.iter().zip(alloc.iter()) {
        if layers == 0 {
            continue;
        }
        let start = cursor;
        let end = cursor + layers;
        cursor = end;
        assignments.push(ShardAssignment {
            worker_pubkey: w.worker_pubkey,
            layer_start: start,
            layer_end: end,
            role: ShardRole::LayerWorker,
            shard_hashes: w.shard_hashes.clone(),
            kv_cache_policy: KvCachePolicy::LocalEphemeral,
            // Re-balancing / fallback targets are assigned by Phase E churn
            // handling, not at placement time.
            fallback_node: None,
            launch_profile_hash: w.launch_profile_hash,
        });
    }

    let plan = ShardPlan::new(assignments);

    // BLOCKER S2-F5: the stateful "covers exactly [0..L)" check is delegated
    // to this phase by shard_plan.rs:209. Enforce both invariants here.
    if !covers_full_model(&plan, model.total_layers) {
        return Err(CoordinatorError::Validation(
            "placement: produced plan does not cover [0..total_layers) contiguously".into(),
        ));
    }
    if plan.assignments.len() < MIN_SHARD_WORKERS {
        return Err(CoordinatorError::Validation(
            "placement: collapsed to a single non-empty shard".into(),
        ));
    }

    Ok(PlacementOutcome::Sharded(plan))
}

/// Whether `plan` covers the whole model: gap-free / non-overlapping
/// ([`ShardPlan::is_pipeline_contiguous`]) **and** spanning exactly
/// `[0..total_layers)` (first block starts at 0, last ends at
/// `total_layers`). The second half is the stateful check `shard_plan.rs`
/// explicitly delegates to the scheduler.
pub(crate) fn covers_full_model(plan: &ShardPlan, total_layers: u32) -> bool {
    if !plan.is_pipeline_contiguous() {
        return false;
    }
    match (plan.assignments.first(), plan.assignments.last()) {
        (Some(first), Some(last)) => first.layer_start == 0 && last.layer_end == total_layers,
        _ => false,
    }
}

/// Deterministic, session-scoped, non-lexicographic sampling key for a
/// worker. `blake3(session_id || pubkey)` so the ordering is reproducible for
/// a given session yet a worker cannot bias its position by minting a
/// low-prefix pubkey (the SYBIL-SEEDER-TAIL crowding the carry targets).
fn sampling_key(session_id: &str, pubkey: &[u8; 32]) -> [u8; 32] {
    let mut input = Vec::with_capacity(session_id.len() + 32);
    input.extend_from_slice(session_id.as_bytes());
    input.extend_from_slice(pubkey);
    *blake3::hash(&input).as_bytes()
}

/// Layer capacity of a worker: how many whole layers fit in its measured
/// free VRAM at `per_layer_bytes` each.
fn layer_capacity(vram_free_bytes: u64, per_layer_bytes: u64) -> u64 {
    vram_free_bytes / per_layer_bytes
}

/// Select the minimal set of workers whose measured VRAM covers the model.
///
/// Workers are ordered by `(capacity desc, sampling_key asc)` — capacity
/// first to minimise the number of shards (fewest frontier crossings), then
/// the anti-Sybil `sampling_key` to break ties among equal-capacity workers
/// (the "tail") **without** the lexicographic pubkey crowding the
/// SYBIL-SEEDER-TAIL carry warns about. Workers that cannot hold a single
/// layer are excluded.
fn select_candidates(
    candidates: &[WorkerPlacementProfile],
    total_layers: u32,
    per_layer_bytes: u64,
    session_id: &str,
) -> Result<Vec<WorkerPlacementProfile>, CoordinatorError> {
    let mut usable: Vec<&WorkerPlacementProfile> = candidates
        .iter()
        .filter(|w| layer_capacity(w.vram_free_bytes, per_layer_bytes) >= 1)
        .collect();

    usable.sort_by(|a, b| {
        b.vram_free_bytes.cmp(&a.vram_free_bytes).then_with(|| {
            sampling_key(session_id, &a.worker_pubkey)
                .cmp(&sampling_key(session_id, &b.worker_pubkey))
        })
    });

    let mut chosen = Vec::new();
    let mut cap_sum: u64 = 0;
    for w in usable {
        chosen.push(w.clone());
        cap_sum += layer_capacity(w.vram_free_bytes, per_layer_bytes);
        if cap_sum >= total_layers as u64 && chosen.len() >= MIN_SHARD_WORKERS {
            break;
        }
    }

    if cap_sum < total_layers as u64 {
        return Err(CoordinatorError::Validation(
            "placement: aggregate measured VRAM cannot hold the model".into(),
        ));
    }
    Ok(chosen)
}

/// Distribute `total_layers` across `ordered` workers in proportion to their
/// measured free VRAM (largest-remainder apportionment, integer-exact),
/// capped by each worker's layer capacity, summing to exactly `total_layers`.
///
/// Returns a vector aligned with `ordered`. The proportional split is the
/// "water-filling" of free VRAM; the largest-remainder pass plus a
/// `sampling_key` tie-break keeps it deterministic.
fn water_fill(
    ordered: &[&WorkerPlacementProfile],
    total_layers: u32,
    per_layer_bytes: u64,
    session_id: &str,
) -> Result<Vec<u32>, CoordinatorError> {
    let n = ordered.len();
    let caps: Vec<u32> = ordered
        .iter()
        .map(|w| layer_capacity(w.vram_free_bytes, per_layer_bytes).min(u32::MAX as u64) as u32)
        .collect();
    let cap_total: u64 = caps.iter().map(|&c| c as u64).sum();
    if cap_total < total_layers as u64 {
        return Err(CoordinatorError::Validation(
            "placement: ordered workers cannot hold the model".into(),
        ));
    }

    let sum_w: u128 = ordered.iter().map(|w| w.vram_free_bytes as u128).sum();
    if sum_w == 0 {
        return Err(CoordinatorError::Validation(
            "placement: ordered workers report zero free VRAM".into(),
        ));
    }
    let total = total_layers as u128;

    // Proportional floor, clamped to capacity.
    let mut alloc: Vec<u32> = ordered
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let q = (total * w.vram_free_bytes as u128 / sum_w) as u32;
            q.min(caps[i])
        })
        .collect();
    let mut assigned: u32 = alloc.iter().sum();

    // Distribution priority: largest integer remainder first (true
    // largest-remainder apportionment), anti-Sybil sampling_key as tie-break.
    let mut prio: Vec<usize> = (0..n).collect();
    prio.sort_by(|&a, &b| {
        let ra = total * ordered[a].vram_free_bytes as u128 % sum_w;
        let rb = total * ordered[b].vram_free_bytes as u128 % sum_w;
        rb.cmp(&ra).then_with(|| {
            sampling_key(session_id, &ordered[a].worker_pubkey)
                .cmp(&sampling_key(session_id, &ordered[b].worker_pubkey))
        })
    });

    // Hand out the remaining layers one at a time, cycling the priority order
    // and skipping workers already at capacity.
    while assigned < total_layers {
        let mut progress = false;
        for &i in &prio {
            if alloc[i] < caps[i] {
                alloc[i] += 1;
                assigned += 1;
                progress = true;
                if assigned == total_layers {
                    break;
                }
            }
        }
        if !progress {
            // cap_total >= total_layers guarantees spare capacity exists, so
            // this is unreachable; guard against a logic regression anyway.
            return Err(CoordinatorError::Validation(
                "placement: could not distribute all layers within capacity".into(),
            ));
        }
    }

    Ok(alloc)
}

/// Order `workers` into a pipeline so that low-RTT peers occupy consecutive
/// positions (and therefore consecutive layer blocks), using k-medoids on
/// the measured pairwise RTT matrix.
///
/// Returns a permutation of `0..workers.len()`. Deterministic: the PAM BUILD
/// initialisation uses no randomness and every tie is broken by `pubkey`, so
/// the same inputs always yield the same ordering (and thus the same
/// [`ShardPlan`]).
pub(crate) fn cluster_order_by_rtt(
    workers: &[WorkerPlacementProfile],
    rtt: &RttMatrix,
    k: usize,
) -> Vec<usize> {
    let n = workers.len();
    if n <= 1 {
        return (0..n).collect();
    }
    let k = k.clamp(1, n);

    let pubkeys: Vec<[u8; 32]> = workers.iter().map(|w| w.worker_pubkey).collect();

    // Symmetric distance matrix in microseconds; an unknown pair is "far".
    let mut d = vec![vec![0u64; n]; n];
    for (i, row) in d.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            if i == j {
                continue;
            }
            *cell = rtt
                .get_micros(pubkeys[i], pubkeys[j])
                .unwrap_or(MISSING_RTT_PENALTY_MICROS);
        }
    }

    let medoids = pam_swap(&d, &pubkeys, pam_build(&d, &pubkeys, k));

    // Assign each worker to its nearest medoid (tie-break: smaller medoid
    // pubkey), grouping members of a cluster together. Clusters are ordered
    // by medoid pubkey; within a cluster, by distance to the medoid then
    // pubkey (the medoid itself, distance 0, comes first).
    let mut clusters: BTreeMap<[u8; 32], (usize, Vec<usize>)> = BTreeMap::new();
    for (p, row) in d.iter().enumerate() {
        let m = *medoids
            .iter()
            .min_by(|&&a, &&b| {
                row[a]
                    .cmp(&row[b])
                    .then_with(|| pubkeys[a].cmp(&pubkeys[b]))
            })
            .expect("at least one medoid");
        clusters
            .entry(pubkeys[m])
            .or_insert_with(|| (m, Vec::new()))
            .1
            .push(p);
    }

    let mut order = Vec::with_capacity(n);
    for (_mk, (m, mut members)) in clusters {
        members.sort_by(|&a, &b| {
            d[a][m]
                .cmp(&d[b][m])
                .then_with(|| pubkeys[a].cmp(&pubkeys[b]))
        });
        order.extend(members);
    }
    order
}

/// Total clustering cost: sum over points of the distance to the nearest
/// medoid.
fn total_cost(d: &[Vec<u64>], medoids: &[usize]) -> u64 {
    let mut sum: u64 = 0;
    for row in d {
        let best = medoids
            .iter()
            .map(|&m| row[m])
            .min()
            .unwrap_or(MISSING_RTT_PENALTY_MICROS);
        sum = sum.saturating_add(best);
    }
    sum
}

/// PAM BUILD: deterministic greedy selection of `k` initial medoids (no
/// randomness). The first medoid minimises the sum of distances to all
/// points; each subsequent one most reduces the total clustering cost. Ties
/// break on `pubkey`.
fn pam_build(d: &[Vec<u64>], pubkeys: &[[u8; 32]], k: usize) -> Vec<usize> {
    let n = d.len();
    let mut medoids: Vec<usize> = Vec::with_capacity(k);

    let first = (0..n)
        .min_by(|&a, &b| {
            // Saturating so an adversarial RTT row (huge `set_micros`
            // values) cannot overflow / debug-panic; mirrors `total_cost`.
            let sa = d[a].iter().copied().fold(0u64, u64::saturating_add);
            let sb = d[b].iter().copied().fold(0u64, u64::saturating_add);
            sa.cmp(&sb).then_with(|| pubkeys[a].cmp(&pubkeys[b]))
        })
        .expect("n >= 1");
    medoids.push(first);

    while medoids.len() < k {
        let next = (0..n).filter(|i| !medoids.contains(i)).min_by(|&a, &b| {
            let mut ma = medoids.clone();
            ma.push(a);
            let mut mb = medoids.clone();
            mb.push(b);
            total_cost(d, &ma)
                .cmp(&total_cost(d, &mb))
                .then_with(|| pubkeys[a].cmp(&pubkeys[b]))
        });
        match next {
            Some(i) => medoids.push(i),
            None => break,
        }
    }
    medoids
}

/// PAM SWAP: greedily swap a medoid with a non-medoid whenever it lowers the
/// total cost, until no improving swap remains or [`KMEDOIDS_MAX_ITER`] is
/// hit. Among equally-improving swaps the choice is deterministic (lower new
/// cost, then `pubkey`).
fn pam_swap(d: &[Vec<u64>], pubkeys: &[[u8; 32]], mut medoids: Vec<usize>) -> Vec<usize> {
    for _ in 0..KMEDOIDS_MAX_ITER {
        let current = total_cost(d, &medoids);
        let mut best: Option<(u64, [u8; 32], usize, usize)> = None; // (cost, h_pubkey, medoid_pos, h)
        for mi in 0..medoids.len() {
            for (h, h_pubkey) in pubkeys.iter().enumerate() {
                if medoids.contains(&h) {
                    continue;
                }
                let mut cand = medoids.clone();
                cand[mi] = h;
                let c = total_cost(d, &cand);
                if c < current {
                    let key = (c, *h_pubkey, mi, h);
                    match best {
                        Some((bc, bk, _, _)) if (bc, bk) <= (key.0, key.1) => {}
                        _ => best = Some(key),
                    }
                }
            }
        }
        match best {
            Some((_, _, mi, h)) => medoids[mi] = h,
            None => break,
        }
    }
    medoids
}

#[cfg(test)]
mod tests {
    use super::*;

    const GIB: u64 = 1024 * 1024 * 1024;

    fn pk(byte: u8) -> [u8; 32] {
        [byte; 32]
    }

    fn worker(byte: u8, vram_gib: u64) -> WorkerPlacementProfile {
        WorkerPlacementProfile {
            worker_pubkey: pk(byte),
            vram_free_bytes: vram_gib * GIB,
            shard_hashes: vec![[byte; 32]],
            launch_profile_hash: [byte.wrapping_add(1); 32],
        }
    }

    /// A 70B-class model: 80 transformer layers, ~40 GiB quantized (Q4).
    fn model_70b() -> ModelSpec {
        ModelSpec {
            total_layers: 80,
            quantized_vram_bytes: 40 * GIB,
        }
    }

    fn layers_for(plan: &ShardPlan, pubkey: [u8; 32]) -> u32 {
        plan.assignments
            .iter()
            .filter(|a| a.worker_pubkey == pubkey)
            .map(|a| a.layer_end - a.layer_start)
            .sum()
    }

    #[test]
    fn placement_water_fills_vram_free() {
        // Three workers that are ALL needed (any two cannot cover 80 layers),
        // with measured free VRAM in a 1:2:1 ratio.
        let candidates = vec![worker(1, 12), worker(2, 24), worker(3, 12)];
        let rtt = RttMatrix::new(); // uniform/unknown — layer counts are
        // proportional to vram regardless of pipeline order.
        let outcome = plan_placement(&candidates, &rtt, &model_70b(), "sess-water").unwrap();
        let plan = match outcome {
            PlacementOutcome::Sharded(p) => p,
            other => panic!("expected Sharded, got {other:?}"),
        };
        let l1 = layers_for(&plan, pk(1));
        let l2 = layers_for(&plan, pk(2));
        let l3 = layers_for(&plan, pk(3));
        assert_eq!(l1 + l2 + l3, 80, "every layer is placed exactly once");
        // The 24 GiB worker carries ~2x the layers of each 12 GiB worker.
        assert_eq!(l2, l1 + l3, "double VRAM => double the layer share");
        assert_eq!(l1, l3, "equal VRAM => equal share");
        assert_eq!(l2, 40);
        assert!(covers_full_model(&plan, 80));
    }

    #[test]
    fn placement_refuses_when_model_fits_single_worker() {
        // 8 GiB model, a 16 GiB worker holds it whole => no shard.
        let candidates = vec![worker(1, 16), worker(2, 16)];
        let model = ModelSpec {
            total_layers: 32,
            quantized_vram_bytes: 8 * GIB,
        };
        let outcome = plan_placement(&candidates, &RttMatrix::new(), &model, "sess-fed").unwrap();
        assert_eq!(outcome, PlacementOutcome::EndpointFederation);
    }

    #[test]
    fn placement_federation_at_exact_fit() {
        // Boundary: the model footprint is EXACTLY the largest worker's free
        // VRAM. The threshold is `<=`, so an exact fit is federation, never a
        // degenerate single-worker "shard".
        let candidates = vec![worker(1, 16), worker(2, 8)];
        let model = ModelSpec {
            total_layers: 32,
            quantized_vram_bytes: 16 * GIB,
        };
        let outcome = plan_placement(&candidates, &RttMatrix::new(), &model, "sess-eq").unwrap();
        assert_eq!(outcome, PlacementOutcome::EndpointFederation);
    }

    #[test]
    fn kmedoids_groups_low_rtt_consecutive_layers() {
        // A & B are mutually close; C & D are mutually close; the two pairs
        // are far apart. k=2 must group {A,B} and {C,D}, so each pair lands
        // in consecutive pipeline positions.
        let workers = vec![
            worker(10, 16),
            worker(11, 16),
            worker(12, 16),
            worker(13, 16),
        ];
        let (a, b, c, dd) = (pk(10), pk(11), pk(12), pk(13));
        let mut rtt = RttMatrix::new();
        rtt.set_micros(a, b, 1_000); // 1 ms intra-pair
        rtt.set_micros(c, dd, 1_000);
        for (x, y) in [(a, c), (a, dd), (b, c), (b, dd)] {
            rtt.set_micros(x, y, 120_000); // 120 ms inter-pair
        }

        let order = cluster_order_by_rtt(&workers, &rtt, 2);
        assert_eq!(order.len(), 4);
        // Positions of each worker index in the pipeline.
        let pos = |idx: usize| order.iter().position(|&o| o == idx).unwrap();
        let (pa, pb, pc, pd) = (pos(0), pos(1), pos(2), pos(3));
        // A,B adjacent and C,D adjacent (each low-RTT pair is consecutive).
        assert_eq!(pa.abs_diff(pb), 1, "low-RTT pair A,B must be consecutive");
        assert_eq!(pc.abs_diff(pd), 1, "low-RTT pair C,D must be consecutive");
        // The two clusters do not interleave.
        assert!(
            (pa.max(pb) < pc.min(pd)) || (pc.max(pd) < pa.min(pb)),
            "clusters must not interleave in the pipeline"
        );
    }

    #[test]
    fn placement_handles_5_workers_70b() {
        // Five 9 GiB workers: each holds 18 layers (9 GiB / 0.5 GiB), so all
        // five are required to cover 80 (any four cover only 72).
        let candidates = vec![
            worker(1, 9),
            worker(2, 9),
            worker(3, 9),
            worker(4, 9),
            worker(5, 9),
        ];
        // A plausible measured RTT mesh.
        let mut rtt = RttMatrix::new();
        for i in 1u8..=5 {
            for j in (i + 1)..=5 {
                rtt.set_micros(pk(i), pk(j), 10_000 + (i as u64 + j as u64) * 1_000);
            }
        }
        let outcome = plan_placement(&candidates, &rtt, &model_70b(), "sess-5x").unwrap();
        let plan = match outcome {
            PlacementOutcome::Sharded(p) => p,
            other => panic!("expected Sharded, got {other:?}"),
        };
        assert_eq!(plan.assignments.len(), 5, "all five workers are shards");
        assert!(plan.is_pipeline_contiguous());
        assert!(
            covers_full_model(&plan, 80),
            "the plan must cover exactly [0..80)"
        );
        let placed: u32 = plan
            .assignments
            .iter()
            .map(|a| a.layer_end - a.layer_start)
            .sum();
        assert_eq!(placed, 80);
        // Every assignment is a non-empty layer-worker with local KV cache.
        for a in &plan.assignments {
            assert!(a.layer_start < a.layer_end);
            assert_eq!(a.role, ShardRole::LayerWorker);
            assert_eq!(a.kv_cache_policy, KvCachePolicy::LocalEphemeral);
            assert_eq!(a.fallback_node, None);
        }
    }

    #[test]
    fn sybil_seeder_tail_sampling_is_deterministic_non_lexicographic() {
        // A 24 GiB base worker plus six equal 16 GiB "tail" workers with
        // pubkeys [1..6] (lexicographically ordered). The base (capacity 48)
        // plus one tail (capacity 32) covers 80, so exactly ONE tail is
        // chosen — and which one must be decided by the anti-crowding
        // sampling key, not by the lexicographic pubkey.
        let mut candidates = vec![worker(100, 24)];
        for i in 1u8..=6 {
            candidates.push(worker(i, 16));
        }
        let session = "sess-sybil";

        let selected =
            select_candidates(&candidates, 80, 40 * GIB / 80, session).expect("feasible");
        // Base first (highest capacity), then exactly one tail.
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].worker_pubkey, pk(100));
        let chosen_tail = selected[1].worker_pubkey;

        // The chosen tail is the one minimising sampling_key over the six —
        // NOT the lexicographically smallest (pk(1)) in general.
        let mut tails: Vec<[u8; 32]> = (1u8..=6).map(pk).collect();
        tails.sort_by_key(|p| sampling_key(session, p));
        assert_eq!(
            chosen_tail, tails[0],
            "the tail is selected by blake3 sampling order"
        );
        // The blake3 order differs from the lexicographic order (proves the
        // selection is non-lexicographic, the crux of the carry).
        let lex: Vec<[u8; 32]> = (1u8..=6).map(pk).collect();
        assert_ne!(tails, lex, "sampling order must not be lexicographic");

        // Reproducible: same inputs => same selection.
        let again = select_candidates(&candidates, 80, 40 * GIB / 80, session).expect("feasible");
        assert_eq!(again[1].worker_pubkey, chosen_tail);
    }

    #[test]
    fn placement_rejects_insufficient_aggregate_vram() {
        // Two 4 GiB workers (capacity 8 layers each = 16) cannot hold an
        // 80-layer / 40 GiB model.
        let candidates = vec![worker(1, 4), worker(2, 4)];
        let err = plan_placement(&candidates, &RttMatrix::new(), &model_70b(), "sess-x")
            .expect_err("must fail when aggregate VRAM is too small");
        assert!(matches!(err, CoordinatorError::Validation(_)));
    }

    #[test]
    fn placement_rejects_degenerate_model_or_empty_candidates() {
        assert!(plan_placement(&[], &RttMatrix::new(), &model_70b(), "s").is_err());
        let candidates = vec![worker(1, 16), worker(2, 16)];
        assert!(
            plan_placement(
                &candidates,
                &RttMatrix::new(),
                &ModelSpec {
                    total_layers: 0,
                    quantized_vram_bytes: 40 * GIB
                },
                "s"
            )
            .is_err()
        );
        assert!(
            plan_placement(
                &candidates,
                &RttMatrix::new(),
                &ModelSpec {
                    total_layers: 80,
                    quantized_vram_bytes: 0
                },
                "s"
            )
            .is_err()
        );
    }

    #[test]
    fn missing_rtt_is_treated_as_far_not_panic() {
        // No RTT recorded at all — clustering must still produce a full,
        // deterministic ordering without panicking.
        let workers = vec![worker(1, 16), worker(2, 16), worker(3, 16)];
        let order = cluster_order_by_rtt(&workers, &RttMatrix::new(), 2);
        let mut sorted = order.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, vec![0, 1, 2], "ordering is a permutation of inputs");
        // Deterministic across calls.
        assert_eq!(order, cluster_order_by_rtt(&workers, &RttMatrix::new(), 2));
    }

    #[test]
    fn sampling_key_is_session_scoped_and_stable() {
        let p = pk(7);
        assert_eq!(
            sampling_key("sess-a", &p),
            sampling_key("sess-a", &p),
            "stable for identical inputs"
        );
        assert_ne!(
            sampling_key("sess-a", &p),
            sampling_key("sess-b", &p),
            "a different session yields a different key"
        );
    }

    #[test]
    fn covers_full_model_rejects_partial_or_gapped_plans() {
        let a = worker(1, 16);
        let b = worker(2, 16);
        let mk = |start: u32, end: u32, w: &WorkerPlacementProfile| ShardAssignment {
            worker_pubkey: w.worker_pubkey,
            layer_start: start,
            layer_end: end,
            role: ShardRole::LayerWorker,
            shard_hashes: w.shard_hashes.clone(),
            kv_cache_policy: KvCachePolicy::LocalEphemeral,
            fallback_node: None,
            launch_profile_hash: w.launch_profile_hash,
        };
        // Full coverage [0..32).
        let good = ShardPlan::new(vec![mk(0, 16, &a), mk(16, 32, &b)]);
        assert!(covers_full_model(&good, 32));
        // Does not start at 0.
        let off = ShardPlan::new(vec![mk(4, 20, &a), mk(20, 32, &b)]);
        assert!(!covers_full_model(&off, 32));
        // Does not reach total_layers.
        let short = ShardPlan::new(vec![mk(0, 16, &a), mk(16, 30, &b)]);
        assert!(!covers_full_model(&short, 32));
        // Gap in the middle.
        let gap = ShardPlan::new(vec![mk(0, 16, &a), mk(20, 32, &b)]);
        assert!(!covers_full_model(&gap, 32));
    }
}
