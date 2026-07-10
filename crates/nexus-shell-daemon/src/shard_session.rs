// SPDX-License-Identifier: AGPL-3.0-or-later
//! In-vivo shard-session orchestrator (Sprint 81 Phase I — the ex-S78 core).
//!
//! Sprint 77 delivered every sharded-inference primitive hermetically —
//! signed wire types ([`nexus_core_rs::shard_plan`]), the `sbfb/shard/1`
//! data plane ([`nexus_core_rs::shard`]), Parallax placement
//! ([`nexus_coordinator_rs::placement`]), churn routing
//! ([`nexus_coordinator_rs::routing`]) — but NO production code ever
//! composed them into a running session: `live_shard_session` was a stub,
//! `RunProofEntry::sign` had zero call-sites outside `cfg(test)`, and the
//! live benchmark harness (`scripts/acceptance/b3_shard_pipeline.sh`) calls
//! routes that did not exist. This module is that missing composition: the
//! **session driver** the S77 audit tracked as the RIG-ABSENT carry.
//!
//! ## Six-step session lifecycle (Phase I preflight, prior-art grounded)
//!
//! 1. **Placement** — [`nexus_coordinator_rs::placement::plan_placement`]
//!    (water-fill + RTT clustering + anti-Sybil sampling), then
//!    [`nexus_coordinator_rs::routing::assign_fallback_nodes`] fills each
//!    stage's churn fallback at plan time.
//! 2. **Manifest** — the initiator signs the plan
//!    ([`ShardedSessionManifestEntry::sign`], `DOMAIN_SHARD_PLAN_V1`).
//! 3. **Readiness barrier** — every downstream shard is probed over the
//!    EXISTING transport before any dispatch frame is emitted. The S77
//!    live gap's root cause was exactly this missing step (exo prefers
//!    nodes by download-status, Petals assembles the full chain before
//!    injecting the first token): dispatching into a shard that is not
//!    ready hangs the pipeline. A shard that fails the probe fails the
//!    mount with a `BLOCK`-style diagnostic instead of hanging.
//!    **0-wire by construction** (preflight R-I-1): the probe is the QUIC
//!    handshake (ALPN negotiated, an authenticated network round-trip)
//!    plus a sampled path RTT ([`nexus_core_rs::shard::conn_rtt`]) — never
//!    a probe *frame*, so a real layer-block backend is never fed garbage
//!    activations and no frame-type discriminator touches the wire.
//! 4. **Dispatch** — the HUB drive: the orchestrator (dialer side, the
//!    star topology the S77 data plane froze) walks the pipeline order,
//!    sending each stage's input frame and reading its output over the
//!    stage worker's long-lived bi-stream. Every hop runs WHOLE —
//!    `open_bi` + write + read — under a **per-hop deadline** (SI-9
//!    withholding, Sev M: neither `write_frame` nor `read_frame` has an
//!    intrinsic timeout and the write path backpressures on QUIC flow
//!    control, so the dialer is the only place liveness can be
//!    enforced). On a hop timeout the drive re-routes to the assignment's
//!    plan-time `fallback_node` (re-probed first) and **resumes from the
//!    bounded activation replay cache**
//!    ([`nexus_coordinator_rs::routing::ActivationReplayCache`], the
//!    Petals-style churn actif the design froze) — or fails CLEAN with a
//!    diagnostic when no fallback exists. Churn semantics (preflight
//!    deliverable 7): resume-from-cache when a fallback is available,
//!    explicit counted cut otherwise; both increment `worker_drop_count`.
//! 5. **Measure** — all-integer [`RunMetrics`] measured with
//!    [`std::time::Instant`] (the S76 `generation_time_ms` root-cause
//!    precedent): TTFT, decode rate in milli-tokens/sec, frame bytes.
//! 6. **Collect + teardown** — the driver signs a [`RunProofEntry`] over
//!    the measured run (the FIRST production emission of a RunProof;
//!    until now every call-site was `cfg(test)`), stores the outcome in
//!    the in-memory registry, then closes every shard connection
//!    gracefully (QUIC close ⇒ the worker's accept loop finishes ⇒ its
//!    `LocalEphemeral` KV cache is discarded per its
//!    [`nexus_core_rs::shard_plan::KvCachePolicy`] contract).
//!
//! ## Scope and honesty
//!
//! - **Operator tool, not a product feature**: this re-certifies the S77
//!   PROVISIONAL deliverable. The head's own [`RunProof`] covers the run
//!   it DROVE (its measured session metrics, non-repudiable); per-worker
//!   RunProofs from remote shards need a control-plane return channel
//!   (existing feed raw-op / iroh-docs — never a new ALPN) and are wired
//!   with the live benchmark (Phase J).
//! - The manifest travels initiator→workers OUT OF BAND for the operator
//!   flow (the worker's layer window is its launch configuration); the
//!   signed manifest is the initiator's authorisation record and the
//!   verification anchor.
//! - The registry is **in-memory and node-local** (never wire, never
//!   on-disk): a session status has no value beyond the process that
//!   drives it. Session status therefore lives here, NOT as a field on
//!   any signed canonical payload (0 wire bump).
//! - The daemon head does NOT serve `sbfb/shard/1` itself: the star
//!   data plane is dialer-driven, so the head only dials out. Serving a
//!   shard stays worker-side (`shard-session serve`, transport-only echo;
//!   a real layer-block forwarder is the worker's feature-gated backend).
//!
//! ## Security posture (THREAT_MODEL §16, §5.9 — no new threat class)
//!
//! Registry insertion is gated on the `DOMAIN_SHARD_PLAN_V1` signature +
//! `is_member` allowlist checks BEFORE insert (the contract the stub
//! documented at `http.rs` since S77 Phase J), so the status route can
//! never serve an unauthenticated manifest. The HTTP projection stays
//! privacy-whitelisted (aggregate `member_count`, never a
//! `worker_pubkey`/`initiator` — SI-3/SI-4). The orchestrator reuses the
//! node's long-lived Ed25519 keypair and the hardened loopback tier; no
//! new credential, no new `DOMAIN_*`, no new invite surface.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use iroh::EndpointAddr;
use iroh::address_lookup::memory::MemoryLookup;
use iroh::endpoint::Connection;
use iroh::{Endpoint, EndpointId};
use nexus_coordinator_rs::placement::{
    ModelSpec, PlacementOutcome, RttMatrix, WorkerPlacementProfile, plan_placement,
};
use nexus_coordinator_rs::routing::{
    ActivationReplayCache, PerfMap, RoutingRequest, RoutingStage, assign_fallback_nodes,
};
use nexus_core_rs::compute_group::{ComputeGroup, ComputeGroupEntry};
use nexus_core_rs::crypto::KeyPair;
use nexus_core_rs::shard::{conn_rtt, open_shard_connection, read_frame, write_frame};
use nexus_core_rs::shard_plan::{
    RunMetrics, RunProof, RunProofEntry, ShardedSessionManifest, ShardedSessionManifestEntry,
};
use serde::Deserialize;
use tracing::{debug, info, warn};

/// Default deadline for the per-shard readiness probe (step 3). A shard
/// that cannot complete the QUIC handshake + RTT sample within this window
/// is not ready; the mount fails with a diagnostic instead of hanging the
/// first dispatched frame. Overridable per mount (`readiness_deadline_ms`)
/// — tests tighten it, a WAN operator may widen it.
pub const READINESS_DEADLINE_DEFAULT_MS: u64 = 15_000;

/// Default per-hop dispatch deadline (step 4, SI-9). Bounds each WHOLE
/// hop the drive performs — `open_bi`, `write_frame` (QUIC-flow-control
/// backpressured) and `read_frame` together — so an admitted-but-silent
/// worker (withholding, Sev M) can stall the pipeline for at most one
/// deadline before the fallback re-route (or a clean diagnosed failure),
/// whether it withholds on the read or the write side. Generous by
/// default because a real ~20 GB layer-block prefill over WAN is slow;
/// overridable per mount (`hop_deadline_ms`).
pub const HOP_DEADLINE_DEFAULT_MS: u64 = 120_000;

/// Poll interval while waiting for the transport to sample a path RTT
/// during the readiness probe.
const RTT_SAMPLE_POLL_MS: u64 = 25;

// ---------------------------------------------------------------------
// Mount request DTOs (loopback HTTP body / CLI config file)
// ---------------------------------------------------------------------

/// One candidate worker the operator offers to the placement, with the
/// address the orchestrator dials it at. The worker's identity is
/// `addr.id` (the QUIC-authenticated Ed25519 endpoint id) — never a
/// separately-declared pubkey that could disagree with the transport.
#[derive(Debug, Clone, Deserialize)]
pub struct ShardWorkerSpec {
    /// Where to dial this worker (`sbfb/shard/1`). Printed by
    /// `shard-session serve` on the worker machine.
    pub addr: EndpointAddr,
    /// Measured free VRAM in bytes (feeds the water-fill placement).
    pub vram_free_bytes: u64,
    /// BLAKE3 hash-pins of the shard weight artifacts this worker loads.
    /// Optional for a transport-only (echo) worker.
    #[serde(default)]
    pub shard_hashes: Vec<[u8; 32]>,
    /// BLAKE3 hash of the launch profile this worker boots with.
    /// Optional for a transport-only (echo) worker.
    #[serde(default)]
    pub launch_profile_hash: [u8; 32],
}

/// The model being mounted, as the operator declares it.
#[derive(Debug, Clone, Deserialize)]
pub struct ShardModelSpec {
    /// Total transformer layers to cover, `[0..total_layers)`.
    pub total_layers: u32,
    /// Quantized in-VRAM footprint of the whole model, bytes.
    pub quantized_vram_bytes: u64,
    /// BLAKE3 digest of the model (zeros for a transport-only session).
    #[serde(default)]
    pub model_digest: [u8; 32],
    /// BLAKE3 digest of the tokenizer.
    #[serde(default)]
    pub tokenizer_hash: [u8; 32],
    /// BLAKE3 digest of the chat template.
    #[serde(default)]
    pub chat_template_hash: [u8; 32],
}

/// Full mount request: the signed private group + candidate workers +
/// model. `group` is minted once (`POST /api/daemon/shard-session/group`)
/// and shared verbatim with every `shard-session serve` worker, so the
/// admission allowlist and the mount gate check the SAME signed bytes.
#[derive(Debug, Clone, Deserialize)]
pub struct MountSessionRequest {
    /// Stable session handle (bounded by
    /// [`nexus_core_rs::shard_plan::SESSION_ID_MAX`] at the signed layer).
    pub session_id: String,
    /// The signed compute-group allowlist admitting this session's
    /// workers AND the head (the dialer must be a member — the worker-side
    /// admission checks `conn.remote_id()` against this list).
    pub group: ComputeGroupEntry,
    /// Candidate workers (placement selects; non-selected candidates stay
    /// available as churn fallbacks).
    pub workers: Vec<ShardWorkerSpec>,
    /// The model to place.
    pub model: ShardModelSpec,
    /// Readiness-probe deadline override, milliseconds.
    #[serde(default)]
    pub readiness_deadline_ms: Option<u64>,
    /// Per-hop dispatch deadline override, milliseconds (SI-9).
    #[serde(default)]
    pub hop_deadline_ms: Option<u64>,
}

/// What a successful mount reports back to the operator.
#[derive(Debug, Clone)]
pub struct MountReport {
    /// The mounted session id.
    pub session_id: String,
    /// Pipeline width (`plan.assignments.len()`).
    pub member_count: usize,
    /// Worst measured frontier RTT across the readiness probes,
    /// milliseconds (the honest pessimistic bound the live gate reads).
    pub rtt_frontier_ms: Option<u64>,
}

// ---------------------------------------------------------------------
// Registry (in-memory, node-local, never wire)
// ---------------------------------------------------------------------

/// Runtime lifecycle of one mounted session. Registry-local — NEVER a
/// field on a signed canonical payload (0 wire bump; a session status has
/// no meaning outside the process driving it).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShardSessionStatus {
    /// Mounted, readiness barrier passed, no generation driven yet.
    Ready,
    /// A generation drive is in flight.
    Generating,
    /// The last generation completed and its outcome is readable.
    Complete,
    /// The last generation failed; `failure` carries the diagnostic.
    Failed,
}

/// The measured outcome of one driven generation.
#[derive(Debug, Clone)]
pub struct ShardRunOutcome {
    /// The final boundary frame decoded as UTF-8 (lossy) — with a real
    /// tail shard this is the generated text; with the transport-only
    /// echo forwarder it echoes the prompt (plumbing proof).
    pub result_text: String,
    /// Time to first output frame from the LAST stage, milliseconds.
    pub ttft_ms: u64,
    /// Whole-drive decode duration, milliseconds.
    pub decode_ms: u64,
    /// Output frames observed from the last stage ("tokens" at the
    /// transport level; real token counts arrive with the real backend).
    pub tokens: u64,
    /// The signed run proof the driver emitted over this run.
    pub run_proof: RunProofEntry,
}

/// One mounted session: the signed authorisation record + live driving
/// state. Connections are kept from the readiness barrier so the first
/// generation reuses the probed connection (one long-lived QUIC connection
/// per pair, the D2 contract); a later generation re-dials because the
/// worker's accept loop serves exactly one bi-stream per connection.
pub struct ShardSessionRecord {
    /// The initiator-signed manifest (verified at insert).
    pub entry: ShardedSessionManifestEntry,
    /// The signed admission allowlist (verified at insert).
    pub group: ComputeGroupEntry,
    /// Dial addresses for every plan worker AND every fallback.
    pub addrs: BTreeMap<[u8; 32], EndpointAddr>,
    /// Probed connections from the readiness barrier, consumed by the
    /// first drive.
    pub conns: BTreeMap<[u8; 32], Connection>,
    /// Lifecycle status.
    pub status: ShardSessionStatus,
    /// Outcome of the last completed drive.
    pub outcome: Option<ShardRunOutcome>,
    /// Worst frontier RTT (ms) measured at the readiness barrier.
    pub rtt_frontier_ms: Option<u64>,
    /// Mid-run drops the drive observed (churn), plus explicit
    /// `drop-shard` cuts.
    pub worker_drop_count: u32,
    /// Diagnostic when `status == Failed` (the `BLOCK{diagnosis}`
    /// vocabulary the harness surfaces — never a silent hang).
    pub failure: Option<String>,
    /// Per-hop dispatch deadline for this session (SI-9).
    pub hop_deadline: Duration,
    /// Readiness deadline used for fallback re-probes.
    pub readiness_deadline: Duration,
}

/// Read-model of a session for the status route (privacy whitelist:
/// aggregate only — the HTTP layer maps this onto
/// [`nexus_core_rs::ShardSessionView`]).
#[derive(Debug, Clone)]
pub struct SessionStatusData {
    pub session_id: String,
    pub member_count: usize,
    pub rtt_frontier_ms: Option<u64>,
}

/// Read-model of a session's result for the result route.
#[derive(Debug, Clone)]
pub struct SessionResultData {
    pub session_id: String,
    pub result_text: Option<String>,
    pub ttft_s: Option<u64>,
    pub toks_per_s: Option<u64>,
    pub run_proof: Option<String>,
    pub rtt_frontier_ms: Option<u64>,
    pub worker_drop_count: u32,
    pub failure: Option<String>,
}

/// In-memory registry of mounted shard sessions. This is the live store
/// `live_shard_session` was stubbed for since Sprint 77 Phase J.
#[derive(Default)]
pub struct ShardSessionRegistry {
    sessions: Mutex<HashMap<String, ShardSessionRecord>>,
}

// Manual Debug (the registry lives inside the `derive(Debug)`
// `DaemonHttpState`): an aggregate count only — a record dump would print
// worker/initiator identities into logs, the exact leak the SI-3/SI-4
// projection whitelist exists to prevent.
impl std::fmt::Debug for ShardSessionRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShardSessionRegistry")
            .field("sessions", &self.lock().len())
            .finish()
    }
}

impl ShardSessionRegistry {
    /// The signature + membership gate the S77 stub mandated: verify the
    /// `DOMAIN_SHARD_PLAN_V1` manifest signature, the group signature, the
    /// manifest↔group binding, and that every plan worker (and fallback)
    /// is an allowlisted member with a known dial address — BEFORE insert.
    /// The route can therefore never serve an unauthenticated manifest.
    pub fn gate_session(
        entry: &ShardedSessionManifestEntry,
        group: &ComputeGroupEntry,
        addrs: &BTreeMap<[u8; 32], EndpointAddr>,
    ) -> Result<(), String> {
        entry
            .verify_signature()
            .map_err(|e| format!("manifest signature rejected: {e}"))?;
        group
            .verify_signature()
            .map_err(|e| format!("compute group signature rejected: {e}"))?;
        if entry.manifest.group_id != group.group.group_id {
            return Err(format!(
                "manifest group_id '{}' does not match compute group '{}'",
                entry.manifest.group_id, group.group.group_id
            ));
        }
        if entry.manifest.initiator != group.group.initiator {
            return Err("manifest initiator does not match compute group initiator".into());
        }
        if !entry.manifest.plan.is_pipeline_contiguous() {
            return Err("plan is not pipeline-contiguous".into());
        }
        for a in &entry.manifest.plan.assignments {
            if !group.is_member(&a.worker_pubkey) {
                return Err(format!(
                    "plan worker {} is not a compute-group member",
                    hex::encode(&a.worker_pubkey[..8])
                ));
            }
            if !addrs.contains_key(&a.worker_pubkey) {
                return Err(format!(
                    "plan worker {} has no dial address",
                    hex::encode(&a.worker_pubkey[..8])
                ));
            }
            if let Some(fb) = a.fallback_node {
                if !group.is_member(&fb) {
                    return Err(format!(
                        "fallback worker {} is not a compute-group member",
                        hex::encode(&fb[..8])
                    ));
                }
                if !addrs.contains_key(&fb) {
                    return Err(format!(
                        "fallback worker {} has no dial address",
                        hex::encode(&fb[..8])
                    ));
                }
            }
        }
        Ok(())
    }

    /// Gate ([`Self::gate_session`]) then insert. Rejects a duplicate
    /// session id (a re-mount is a new session, not a silent overwrite).
    pub fn insert_gated(&self, record: ShardSessionRecord) -> Result<(), String> {
        Self::gate_session(&record.entry, &record.group, &record.addrs)?;
        let session_id = record.entry.manifest.session_id.clone();
        let mut sessions = self.lock();
        if sessions.contains_key(&session_id) {
            return Err(format!("session '{session_id}' is already mounted"));
        }
        sessions.insert(session_id, record);
        Ok(())
    }

    /// Aggregate status data for the status route (`None` = not mounted).
    pub fn status_data(&self, session_id: &str) -> Option<SessionStatusData> {
        let sessions = self.lock();
        sessions.get(session_id).map(|r| SessionStatusData {
            session_id: r.entry.manifest.session_id.clone(),
            member_count: r.entry.manifest.plan.assignments.len(),
            rtt_frontier_ms: r.rtt_frontier_ms,
        })
    }

    /// Result data for the result route (`None` = not mounted). Fields
    /// stay `None` until a drive completes; `failure` carries the clean
    /// diagnostic of a failed drive.
    pub fn result_data(&self, session_id: &str) -> Option<SessionResultData> {
        let sessions = self.lock();
        sessions.get(session_id).map(|r| {
            let outcome = r.outcome.as_ref();
            SessionResultData {
                session_id: r.entry.manifest.session_id.clone(),
                result_text: outcome.map(|o| o.result_text.clone()),
                ttft_s: outcome.map(|o| o.ttft_ms / 1000),
                toks_per_s: outcome.map(|o| {
                    // Integer tokens/sec floor-guarded against a sub-ms
                    // drive (never divide by zero, never report 0 for a
                    // completed instant drive).
                    (o.tokens.saturating_mul(1000) / o.decode_ms.max(1)).max(1)
                }),
                run_proof: outcome.map(|o| hex::encode(o.run_proof.signature)),
                rtt_frontier_ms: r.rtt_frontier_ms,
                worker_drop_count: r.worker_drop_count,
                failure: r.failure.clone(),
            }
        })
    }

    /// Explicit operator-driven churn cut (`POST .../drop-shard`): close
    /// the LAST stage's connection if still held and count the drop.
    /// Semantics: an explicit counted cut — the next drive re-dials (and a
    /// mid-drive drop is handled by the SI-9 fallback path instead).
    pub fn drop_tail_shard(&self, session_id: &str) -> Option<bool> {
        let mut sessions = self.lock();
        let record = sessions.get_mut(session_id)?;
        let tail = record
            .entry
            .manifest
            .plan
            .assignments
            .last()
            .map(|a| a.worker_pubkey)?;
        if let Some(conn) = record.conns.remove(&tail) {
            conn.close(0u32.into(), b"drop-shard");
        }
        record.worker_drop_count = record.worker_drop_count.saturating_add(1);
        Some(true)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, ShardSessionRecord>> {
        // A poisoned registry lock only means a panicking thread died while
        // holding it; the map itself stays consistent (same recovery the
        // coordinator_db handle uses).
        self.sessions.lock().unwrap_or_else(|p| p.into_inner())
    }

    fn mark_failed(&self, session_id: &str, diagnostic: String) {
        if let Some(r) = self.lock().get_mut(session_id) {
            r.status = ShardSessionStatus::Failed;
            r.failure = Some(diagnostic);
        }
    }

    fn count_drop(&self, session_id: &str) {
        if let Some(r) = self.lock().get_mut(session_id) {
            r.worker_drop_count = r.worker_drop_count.saturating_add(1);
        }
    }
}

// ---------------------------------------------------------------------
// Group minting (operator flow step 1)
// ---------------------------------------------------------------------

/// Mint the signed private compute group for a session: the listed worker
/// pubkeys PLUS the head itself (the star data plane is dialer-driven, so
/// the worker-side admission checks the HEAD's `conn.remote_id()` — the
/// initiator is NOT implicitly a member, `ComputeGroup::is_member`).
/// The returned entry is shared verbatim with every `shard-session serve`
/// worker so admission and the mount gate check the same signed bytes.
pub fn mint_compute_group(
    keypair: &KeyPair,
    group_id: &str,
    revision: u64,
    worker_pubkeys: &[[u8; 32]],
) -> Result<ComputeGroupEntry, String> {
    let mut group = ComputeGroup::new(keypair.public_bytes(), group_id, revision)
        .with_member(keypair.public_bytes());
    for w in worker_pubkeys {
        group = group.with_member(*w);
    }
    ComputeGroupEntry::sign(group, keypair).map_err(|e| format!("group sign failed: {e}"))
}

// ---------------------------------------------------------------------
// Step 1+2 — placement + signed manifest (pure, hermetic)
// ---------------------------------------------------------------------

/// Run the Parallax placement over the operator's candidates, fill the
/// plan-time fallbacks, and sign the session manifest. Pure (no network):
/// unit-testable without nodes, and the mount composes it before touching
/// the transport.
pub fn place_and_sign(
    keypair: &KeyPair,
    session_id: &str,
    group: &ComputeGroupEntry,
    candidates: &[WorkerPlacementProfile],
    model: &ShardModelSpec,
) -> Result<ShardedSessionManifestEntry, String> {
    let spec = ModelSpec {
        total_layers: model.total_layers,
        quantized_vram_bytes: model.quantized_vram_bytes,
    };
    // The RTT matrix is empty at mount time (no live samples yet): the
    // clusterer degrades to capacity order via MISSING_RTT_PENALTY_MICROS.
    // A perf-map fed by driven sessions refines later placements.
    let rtt = RttMatrix::new();
    let plan = match plan_placement(candidates, &rtt, &spec, session_id)
        .map_err(|e| format!("placement failed: {e}"))?
    {
        PlacementOutcome::Sharded(plan) => plan,
        PlacementOutcome::EndpointFederation => {
            return Err(
                "model fits a single worker's free VRAM — use S76 endpoint federation, \
                 not a shard session"
                    .into(),
            );
        }
    };

    // Plan-time churn fallbacks: every candidate is a stage candidate (the
    // non-selected candidates are exactly the fallback pool), admission
    // bounded by the group members.
    let all_candidates: Vec<[u8; 32]> = candidates.iter().map(|c| c.worker_pubkey).collect();
    let allowlist: BTreeSet<[u8; 32]> = group.group.members.iter().copied().collect();
    let routing_req = RoutingRequest {
        stages: plan
            .assignments
            .iter()
            .map(|a| RoutingStage {
                layer_start: a.layer_start,
                layer_end: a.layer_end,
                candidates: all_candidates.clone(),
            })
            .collect(),
    };
    let perf = PerfMap::new();
    let plan = assign_fallback_nodes(&plan, &routing_req, &perf, &allowlist)
        .map_err(|e| format!("fallback assignment failed: {e}"))?;

    let manifest = ShardedSessionManifest::new(
        keypair.public_bytes(),
        session_id,
        group.group.group_id.clone(),
        1,
        plan,
        model.model_digest,
        model.tokenizer_hash,
        model.chat_template_hash,
    );
    ShardedSessionManifestEntry::sign(manifest, keypair)
        .map_err(|e| format!("manifest sign failed: {e}"))
}

// ---------------------------------------------------------------------
// Step 3 — readiness barrier (transport-level, 0-wire)
// ---------------------------------------------------------------------

/// Probe one shard's readiness: complete the `sbfb/shard/1` QUIC handshake
/// and wait for a sampled path RTT, the WHOLE probe under one `deadline`
/// (Codex round 1 P2: a bounded dial followed by a RESTARTED RTT budget
/// let the pair run to almost 2x the deadline — one shared timeout now
/// bounds handshake + sample together, mirroring `drive_hop`). Returns
/// the established connection (kept for the first drive) and the RTT.
///
/// Deliberately NO probe frame (preflight R-I-1): a frame probe would feed
/// a real layer-block backend garbage activations and would need a wire
/// frame-type discriminator. The completed handshake IS an authenticated
/// network round-trip; a dead / unreachable / not-yet-listening shard
/// fails here instead of hanging the first dispatch.
async fn probe_shard_readiness(
    endpoint: &Endpoint,
    lookup: &MemoryLookup,
    addr: EndpointAddr,
    deadline: Duration,
) -> Result<(Connection, Duration), String> {
    let worker_hex = hex::encode(&addr.id.as_bytes()[..8]);
    tokio::time::timeout(deadline, async {
        let conn = open_shard_connection(endpoint, lookup, addr)
            .await
            .map_err(|e| format!("readiness: shard {worker_hex} dial failed: {e}"))?;
        // Wait (inside the same shared deadline) for the transport to
        // sample the path RTT — the perf-map signal the placement consumes.
        loop {
            if let Some(rtt) = conn_rtt(&conn) {
                return Ok((conn, rtt));
            }
            tokio::time::sleep(Duration::from_millis(RTT_SAMPLE_POLL_MS)).await;
        }
    })
    .await
    .map_err(|_| {
        format!(
            "readiness: shard {worker_hex} did not answer (handshake + RTT) within {deadline:?}"
        )
    })?
}

// ---------------------------------------------------------------------
// Mount (steps 1-3 composed) — the operator entry point
// ---------------------------------------------------------------------

/// Mount a shard session: placement → signed manifest → readiness barrier
/// → gated registry insert. On success the session is LIVE
/// (`live_shard_session` misses no more) and ready to drive. On a
/// readiness failure the mount returns the diagnostic and inserts
/// NOTHING — the session never existed, no dispatch frame was ever
/// emitted (the T1 assertion).
pub async fn mount_session(
    endpoint: &Endpoint,
    lookup: &MemoryLookup,
    keypair: &KeyPair,
    registry: &ShardSessionRegistry,
    req: MountSessionRequest,
) -> Result<MountReport, String> {
    // Candidate profiles + dial addresses, identity taken from the
    // transport-authenticated `addr.id`.
    let mut addrs: BTreeMap<[u8; 32], EndpointAddr> = BTreeMap::new();
    let mut candidates: Vec<WorkerPlacementProfile> = Vec::with_capacity(req.workers.len());
    for w in &req.workers {
        let pubkey = *w.addr.id.as_bytes();
        addrs.insert(pubkey, w.addr.clone());
        candidates.push(WorkerPlacementProfile {
            worker_pubkey: pubkey,
            vram_free_bytes: w.vram_free_bytes,
            shard_hashes: w.shard_hashes.clone(),
            launch_profile_hash: w.launch_profile_hash,
        });
    }

    let entry = place_and_sign(
        keypair,
        &req.session_id,
        &req.group,
        &candidates,
        &req.model,
    )?;

    // Fail fast on the authoritative gate BEFORE any network probe, so a
    // non-member plan never even dials (the same gate re-runs at insert).
    ShardSessionRegistry::gate_session(&entry, &req.group, &addrs)?;

    let readiness_deadline = Duration::from_millis(
        req.readiness_deadline_ms
            .unwrap_or(READINESS_DEADLINE_DEFAULT_MS),
    );
    let hop_deadline =
        Duration::from_millis(req.hop_deadline_ms.unwrap_or(HOP_DEADLINE_DEFAULT_MS));

    // Readiness barrier: EVERY plan shard must ACK (handshake + RTT)
    // before the session exists — no dispatch frame is ever emitted ahead
    // of the barrier.
    let mut conns: BTreeMap<[u8; 32], Connection> = BTreeMap::new();
    let mut worst_rtt_ms: Option<u64> = None;
    for a in &entry.manifest.plan.assignments {
        let addr = addrs
            .get(&a.worker_pubkey)
            .expect("gate_session guarantees an address per plan worker")
            .clone();
        let (conn, rtt) = probe_shard_readiness(endpoint, lookup, addr, readiness_deadline)
            .await
            .inspect_err(|_| {
                // Teardown the shards already probed — the mount failed as
                // a unit, no half-open session survives.
                for c in conns.values() {
                    c.close(0u32.into(), b"mount-failed");
                }
            })?;
        let rtt_ms = rtt.as_millis().min(u128::from(u64::MAX)) as u64;
        worst_rtt_ms = Some(worst_rtt_ms.map_or(rtt_ms, |w| w.max(rtt_ms)));
        conns.insert(a.worker_pubkey, conn);
        debug!(
            worker = %hex::encode(&a.worker_pubkey[..8]),
            rtt_ms,
            "shard readiness ACK"
        );
    }

    let member_count = entry.manifest.plan.assignments.len();
    let session_id = entry.manifest.session_id.clone();
    registry.insert_gated(ShardSessionRecord {
        entry,
        group: req.group,
        addrs,
        conns,
        status: ShardSessionStatus::Ready,
        outcome: None,
        rtt_frontier_ms: worst_rtt_ms,
        worker_drop_count: 0,
        failure: None,
        hop_deadline,
        readiness_deadline,
    })?;
    info!(session = %session_id, member_count, "shard session mounted (readiness barrier passed)");

    Ok(MountReport {
        session_id,
        member_count,
        rtt_frontier_ms: worst_rtt_ms,
    })
}

// ---------------------------------------------------------------------
// Steps 4-6 — the HUB drive
// ---------------------------------------------------------------------

/// Send one input frame to a stage worker and read its output frame over a
/// fresh bi-stream on `conn`. The WHOLE hop — `open_bi` + `write_frame` +
/// `read_frame` — runs under ONE `deadline` (SI-9): `write_frame` is
/// backpressured by QUIC stream flow control (`write_all`, shard.rs), so
/// an admitted-but-byzantine worker that accepts the bi-stream without
/// ever draining its recv would stall a large-frame write FOREVER if only
/// the read were bounded (review D1-1/D2-1 — the write path is where the
/// Phase-J workload actually blocks).
async fn drive_hop(conn: &Connection, input: &[u8], deadline: Duration) -> Result<Vec<u8>, String> {
    tokio::time::timeout(deadline, async {
        let (mut send, mut recv) = conn
            .open_bi()
            .await
            .map_err(|e| format!("open_bi failed: {e}"))?;
        write_frame(&mut send, input)
            .await
            .map_err(|e| format!("frame write failed: {e}"))?;
        let out = read_frame(&mut recv)
            .await
            .map_err(|e| format!("frame read failed: {e}"))?
            .ok_or_else(|| "stream finished before an output frame".to_string())?;
        send.finish().ok();
        Ok(out)
    })
    .await
    .map_err(|_| {
        format!(
            "hop exceeded its {deadline:?} deadline \
             (SI-9 withholding guard — covers open/write/read)"
        )
    })?
}

/// Drive one generation through the mounted session's pipeline (steps 4-6).
///
/// HUB topology (frozen S77 data plane): the head walks the assignments in
/// pipeline order, forwarding each stage's output as the next stage's
/// input — every frontier crosses through the head. The live benchmark
/// (Phase J) is therefore judged against a HUB baseline, never the Petals
/// direct-server-to-server envelope.
///
/// On a hop deadline: count the drop, re-probe the assignment's plan-time
/// `fallback_node`, and RESUME by replaying the stage's input frame from
/// the bounded [`ActivationReplayCache`]; with no fallback, fail CLEAN
/// with a diagnostic. Completion signs the driver's [`RunProofEntry`]
/// (first production emission) and tears every connection down.
pub async fn generate_session(
    endpoint: &Endpoint,
    lookup: &MemoryLookup,
    keypair: &KeyPair,
    registry: &ShardSessionRegistry,
    session_id: &str,
    prompt: &str,
) -> Result<(), String> {
    // Snapshot what the drive needs, then release the lock (never hold it
    // across an await).
    let (entry, addrs, mut conns, hop_deadline, readiness_deadline) = {
        let mut sessions = registry.lock();
        let record = sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("session '{session_id}' is not mounted"))?;
        if record.status == ShardSessionStatus::Generating {
            return Err(format!("session '{session_id}' is already generating"));
        }
        record.status = ShardSessionStatus::Generating;
        (
            record.entry.clone(),
            record.addrs.clone(),
            std::mem::take(&mut record.conns),
            record.hop_deadline,
            record.readiness_deadline,
        )
    };

    // Connections the drive consumed. The `sbfb/shard/1` accept loop
    // serves exactly ONE bi-stream per connection (it finishes and awaits
    // close after the stream's FIN), so a driven connection is spent — it
    // must NEVER return to the reusable pool (re-dispatching on it reads
    // nothing and would mis-diagnose a healthy worker as SI-9 churn).
    let mut used: Vec<Connection> = Vec::new();

    let outcome = drive_pipeline(
        endpoint,
        lookup,
        keypair,
        registry,
        &entry,
        &addrs,
        &mut conns,
        &mut used,
        hop_deadline,
        readiness_deadline,
        session_id,
        prompt,
    )
    .await;

    // Teardown (step 6): graceful close on every connection the drive
    // touched (spent) or left unconsumed — the worker's accept loop
    // finishes and discards its LocalEphemeral KV cache.
    for conn in used.iter().chain(conns.values()) {
        conn.close(0u32.into(), b"done");
    }

    match outcome {
        Ok(run) => {
            let mut sessions = registry.lock();
            if let Some(record) = sessions.get_mut(session_id) {
                record.outcome = Some(run);
                record.status = ShardSessionStatus::Complete;
            }
            Ok(())
        }
        Err(diagnostic) => {
            warn!(session = %session_id, %diagnostic, "shard drive failed clean");
            registry.mark_failed(session_id, diagnostic.clone());
            Err(diagnostic)
        }
    }
}

/// The measured pipeline walk. Split from [`generate_session`] so the
/// teardown + status bookkeeping wrap it exactly once.
#[allow(clippy::too_many_arguments)]
async fn drive_pipeline(
    endpoint: &Endpoint,
    lookup: &MemoryLookup,
    keypair: &KeyPair,
    registry: &ShardSessionRegistry,
    entry: &ShardedSessionManifestEntry,
    addrs: &BTreeMap<[u8; 32], EndpointAddr>,
    conns: &mut BTreeMap<[u8; 32], Connection>,
    used: &mut Vec<Connection>,
    hop_deadline: Duration,
    readiness_deadline: Duration,
    session_id: &str,
    prompt: &str,
) -> Result<ShardRunOutcome, String> {
    let started = Instant::now();
    let mut replay = ActivationReplayCache::new();
    let mut frame: Vec<u8> = prompt.as_bytes().to_vec();
    let mut rx_bytes: u64 = 0;
    let mut tx_bytes: u64 = 0;
    let mut drops: u32 = 0;

    let assignments = entry.manifest.plan.assignments.clone();
    for (i, a) in assignments.iter().enumerate() {
        // Resume point for THIS stage: its input frame, keyed by the
        // frontier layer where the activation enters the stage.
        replay.insert(a.layer_start, frame.clone());

        // The readiness-barrier connection is reused when present (first
        // drive); otherwise dial fresh (a later drive, or a fallback) —
        // bounded by the SAME deadline the readiness barrier uses, so a
        // re-drive toward an unreachable stage stays under the session's
        // liveness budget instead of iroh's internal connect timeout
        // (review D2-2).
        let conn = match conns.remove(&a.worker_pubkey) {
            Some(c) => c,
            None => {
                let addr = addrs
                    .get(&a.worker_pubkey)
                    .ok_or_else(|| format!("no dial address for stage {i} worker"))?
                    .clone();
                tokio::time::timeout(
                    readiness_deadline,
                    open_shard_connection(endpoint, lookup, addr),
                )
                .await
                .map_err(|_| format!("stage {i} re-dial exceeded {readiness_deadline:?}"))?
                .map_err(|e| format!("stage {i} dial failed: {e}"))?
            }
        };

        tx_bytes = tx_bytes.saturating_add(frame.len() as u64);
        let hop_result = drive_hop(&conn, &frame, hop_deadline).await;
        let out = match hop_result {
            Ok(out) => {
                // Spent (one bi-stream per connection): park for teardown,
                // never back into the reusable pool.
                used.push(conn);
                out
            }
            Err(hop_err) => {
                // Churn (SI-9 fired): count the drop, close the stalled
                // connection, and re-route to the plan-time fallback with
                // the replay-cached stage input — or fail clean.
                conn.close(0u32.into(), b"hop-deadline");
                drops = drops.saturating_add(1);
                registry.count_drop(session_id);
                let worker_hex = hex::encode(&a.worker_pubkey[..8]);
                let Some(fallback) = a.fallback_node else {
                    return Err(format!(
                        "stage {i} worker {worker_hex} failed ({hop_err}) and the plan carries \
                         no fallback_node — failing clean instead of hanging"
                    ));
                };
                let fb_hex = hex::encode(&fallback[..8]);
                let fb_addr = addrs
                    .get(&fallback)
                    .ok_or_else(|| format!("fallback {fb_hex} has no dial address"))?
                    .clone();
                info!(
                    session = %session_id,
                    stage = i,
                    failed = %worker_hex,
                    fallback = %fb_hex,
                    "hop deadline fired — re-routing to fallback (resume-from-cache)"
                );
                // Re-readiness on the fallback before dispatching to it.
                let (fb_conn, _rtt) =
                    probe_shard_readiness(endpoint, lookup, fb_addr, readiness_deadline)
                        .await
                        .map_err(|e| format!("stage {i} fallback {fb_hex} not ready: {e}"))?;
                let cached = replay
                    .get(a.layer_start)
                    .expect("stage input was inserted before dispatch")
                    .to_vec();
                tx_bytes = tx_bytes.saturating_add(cached.len() as u64);
                let out = drive_hop(&fb_conn, &cached, hop_deadline)
                    .await
                    .inspect_err(|_| {
                        // Explicit close on the failed fallback hop (Codex
                        // round 1 P3) — dropping the handle closes too, but
                        // an application close code beats an implicit drop.
                        fb_conn.close(0u32.into(), b"fallback-hop-failed");
                    })
                    .map_err(|e| {
                        format!("stage {i} fallback {fb_hex} also failed ({e}) — failing clean")
                    })?;
                // Spent, same as the healthy path — park for teardown.
                used.push(fb_conn);
                out
            }
        };
        rx_bytes = rx_bytes.saturating_add(out.len() as u64);
        frame = out;
    }

    // Step 5 — measure. One full pipeline pass produced one output frame
    // from the last stage: TTFT is the time to that first output, and the
    // transport-level "token" count is the observed output frames (real
    // token counts arrive with the real layer-block backend, Phase J).
    let ttft_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    let decode_ms = ttft_ms.max(1);
    let tokens: u64 = 1;

    let metrics = RunMetrics {
        ttft_ms,
        decode_milli_tokens_per_sec: tokens.saturating_mul(1_000_000) / decode_ms,
        p95_token_latency_ms: decode_ms,
        network_rx_bytes: rx_bytes,
        network_tx_bytes: tx_bytes,
        worker_drop_count: drops,
    };

    // Step 6 — collect: the driver's signed RunProof over the run it
    // measured (the first production RunProofEntry::sign call-site).
    let participants: Vec<[u8; 32]> = assignments.iter().map(|a| a.worker_pubkey).collect();
    let proof = RunProof::new(
        keypair.public_bytes(),
        session_id,
        entry.manifest.model_digest,
        nexus_core_rs::crypto::blake3_hash(prompt.as_bytes()),
        metrics,
        participants,
    );
    let run_proof =
        RunProofEntry::sign(proof, keypair).map_err(|e| format!("run proof sign failed: {e}"))?;
    // Sanity: what we just signed must verify — a driver that emits an
    // unverifiable proof is a bug, not a degraded result.
    run_proof
        .verify_signature()
        .map_err(|e| format!("freshly signed run proof failed verification: {e}"))?;

    Ok(ShardRunOutcome {
        result_text: String::from_utf8_lossy(&frame).into_owned(),
        ttft_ms,
        decode_ms,
        tokens,
        run_proof,
    })
}

// ---------------------------------------------------------------------
// Helpers shared with the CLI serve tool
// ---------------------------------------------------------------------

/// Parse a lowercase-hex Ed25519 pubkey (64 hex chars) into its 32 bytes.
pub fn parse_pubkey_hex(s: &str) -> Result<[u8; 32], String> {
    let bytes = hex::decode(s.trim()).map_err(|e| format!("malformed pubkey hex: {e}"))?;
    let arr: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| format!("pubkey must be 32 bytes, got {}", bytes.len()))?;
    // Round-trip through EndpointId so a non-curve-point key is rejected
    // here, not at dial time.
    EndpointId::from_bytes(&arr).map_err(|e| format!("not a valid Ed25519 pubkey: {e}"))?;
    Ok(arr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_core_rs::node::{
        Node, NodeConfig, create_node_with_config, create_node_with_protocols,
    };
    use nexus_core_rs::shard::{EchoForwarder, ShardForwarder, shard_protocol_factory};
    use nexus_core_rs::shard_plan::{KvCachePolicy, ShardAssignment, ShardPlan, ShardRole};
    use nexus_core_rs::{DiscoveryClient, SHARD_ALPN};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A forwarder that counts the frames it sees before echoing — proves
    /// the readiness barrier emits NO dispatch frame.
    #[derive(Debug, Default)]
    struct CountingForwarder(AtomicUsize);
    impl ShardForwarder for CountingForwarder {
        fn forward(&self, upstream_frame: &[u8]) -> nexus_core_rs::Result<Vec<u8>> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(upstream_frame.to_vec())
        }
    }

    /// A forwarder that stalls longer than any test hop deadline —
    /// simulates SI-9 withholding (admitted member that goes silent).
    #[derive(Debug)]
    struct StallingForwarder(Duration);
    impl ShardForwarder for StallingForwarder {
        fn forward(&self, upstream_frame: &[u8]) -> nexus_core_rs::Result<Vec<u8>> {
            std::thread::sleep(self.0);
            Ok(upstream_frame.to_vec())
        }
    }

    struct Rig {
        head: Node,
        head_kp: KeyPair,
        group: ComputeGroupEntry,
        /// (node, addr, keypair) per worker, plan order = spawn order.
        workers: Vec<(Node, EndpointAddr, KeyPair)>,
    }

    impl Rig {
        async fn shutdown(self) {
            self.head.shutdown().await.ok();
            for (n, _, _) in self.workers {
                n.shutdown().await.ok();
            }
        }

        fn worker_specs(&self, vram: &[u64]) -> Vec<ShardWorkerSpec> {
            self.workers
                .iter()
                .zip(vram)
                .map(|((_, addr, _), v)| ShardWorkerSpec {
                    addr: addr.clone(),
                    vram_free_bytes: *v,
                    shard_hashes: vec![],
                    launch_profile_hash: [0u8; 32],
                })
                .collect()
        }
    }

    /// Boot a head + N in-process worker nodes serving `sbfb/shard/1` with
    /// the given forwarders, all admitted by one signed group (the head is
    /// an explicit member — it is the dialer the workers admit).
    async fn shard_rig(forwarders: Vec<Arc<dyn ShardForwarder>>) -> Rig {
        let head_secret = KeyPair::generate().secret_bytes();
        let head_kp = KeyPair::from_secret_bytes(&head_secret);

        let worker_keys: Vec<KeyPair> = (0..forwarders.len())
            .map(|_| {
                let s = KeyPair::generate().secret_bytes();
                KeyPair::from_secret_bytes(&s)
            })
            .collect();
        let group = mint_compute_group(
            &head_kp,
            "test-shard-group",
            1,
            &worker_keys
                .iter()
                .map(|k| k.public_bytes())
                .collect::<Vec<_>>(),
        )
        .expect("group mints");

        let head = create_node_with_config(NodeConfig::default().with_secret_key(head_secret))
            .await
            .expect("head node");

        let mut workers = Vec::new();
        for (kp, forwarder) in worker_keys.into_iter().zip(forwarders) {
            let factory = shard_protocol_factory(group.clone(), forwarder).expect("verified group");
            let node = create_node_with_protocols(
                NodeConfig::default().with_secret_key(kp.secret_bytes()),
                vec![(SHARD_ALPN.to_vec(), factory)],
            )
            .await
            .expect("worker node");
            let addr = DiscoveryClient::new(node.endpoint())
                .my_endpoint_addr()
                .await
                .expect("worker addr");
            workers.push((node, addr, kp));
        }
        Rig {
            head,
            head_kp,
            group,
            workers,
        }
    }

    /// A model spec too large for any single test worker, so the placement
    /// MUST shard across both (never EndpointFederation).
    fn two_shard_model() -> ShardModelSpec {
        ShardModelSpec {
            total_layers: 8,
            quantized_vram_bytes: 1_500,
            model_digest: [1u8; 32],
            tokenizer_hash: [2u8; 32],
            chat_template_hash: [3u8; 32],
        }
    }

    fn mount_request(rig: &Rig, session_id: &str, vram: &[u64]) -> MountSessionRequest {
        MountSessionRequest {
            session_id: session_id.into(),
            group: rig.group.clone(),
            workers: rig.worker_specs(vram),
            model: two_shard_model(),
            readiness_deadline_ms: Some(10_000),
            hop_deadline_ms: Some(10_000),
        }
    }

    // ---- Pure gate tests (hermetic, no network) ----

    fn hand_built_record(
        head_kp: &KeyPair,
        group: &ComputeGroupEntry,
        assignments: Vec<ShardAssignment>,
        addrs: BTreeMap<[u8; 32], EndpointAddr>,
    ) -> ShardSessionRecord {
        let manifest = ShardedSessionManifest::new(
            head_kp.public_bytes(),
            "hand-built",
            group.group.group_id.clone(),
            1,
            ShardPlan::new(assignments),
            [1u8; 32],
            [2u8; 32],
            [3u8; 32],
        );
        let entry = ShardedSessionManifestEntry::sign(manifest, head_kp).expect("sign");
        ShardSessionRecord {
            entry,
            group: group.clone(),
            addrs,
            conns: BTreeMap::new(),
            status: ShardSessionStatus::Ready,
            outcome: None,
            rtt_frontier_ms: None,
            worker_drop_count: 0,
            failure: None,
            hop_deadline: Duration::from_millis(HOP_DEADLINE_DEFAULT_MS),
            readiness_deadline: Duration::from_millis(READINESS_DEADLINE_DEFAULT_MS),
        }
    }

    fn assignment(worker: [u8; 32], start: u32, end: u32) -> ShardAssignment {
        ShardAssignment {
            worker_pubkey: worker,
            layer_start: start,
            layer_end: end,
            role: ShardRole::LayerWorker,
            shard_hashes: vec![],
            kv_cache_policy: KvCachePolicy::LocalEphemeral,
            fallback_node: None,
            launch_profile_hash: [0u8; 32],
        }
    }

    fn dummy_addr(pubkey: [u8; 32]) -> EndpointAddr {
        EndpointAddr::new(EndpointId::from_bytes(&pubkey).expect("valid key"))
    }

    #[test]
    fn registry_gate_rejects_tampered_manifest() {
        let head = KeyPair::generate();
        let worker = KeyPair::generate();
        let group = mint_compute_group(&head, "g", 1, &[worker.public_bytes()]).expect("group");
        let mut addrs = BTreeMap::new();
        addrs.insert(worker.public_bytes(), dummy_addr(worker.public_bytes()));
        let mut record = hand_built_record(
            &head,
            &group,
            vec![assignment(worker.public_bytes(), 0, 8)],
            addrs,
        );
        // Tamper AFTER signing: the DOMAIN_SHARD_PLAN_V1 gate must reject
        // the insert (the stub's documented contract).
        record.entry.manifest.revision += 1;
        let registry = ShardSessionRegistry::default();
        let err = registry.insert_gated(record).unwrap_err();
        assert!(
            err.contains("manifest signature rejected"),
            "tampered manifest must be rejected by the signature gate, got: {err}"
        );
        assert!(registry.status_data("hand-built").is_none());
    }

    #[test]
    fn registry_gate_rejects_non_member_worker() {
        let head = KeyPair::generate();
        let member = KeyPair::generate();
        let outsider = KeyPair::generate();
        let group = mint_compute_group(&head, "g", 1, &[member.public_bytes()]).expect("group");
        let mut addrs = BTreeMap::new();
        addrs.insert(outsider.public_bytes(), dummy_addr(outsider.public_bytes()));
        let record = hand_built_record(
            &head,
            &group,
            vec![assignment(outsider.public_bytes(), 0, 8)],
            addrs,
        );
        let registry = ShardSessionRegistry::default();
        let err = registry.insert_gated(record).unwrap_err();
        assert!(
            err.contains("not a compute-group member"),
            "a non-member plan worker must be rejected before insert, got: {err}"
        );
    }

    #[test]
    fn registry_gate_rejects_group_binding_mismatch() {
        let head = KeyPair::generate();
        let worker = KeyPair::generate();
        let group = mint_compute_group(&head, "g", 1, &[worker.public_bytes()]).expect("group");
        // A DIFFERENT group id than the manifest's: binding must fail.
        let other_group =
            mint_compute_group(&head, "other-group", 1, &[worker.public_bytes()]).expect("group");
        let mut addrs = BTreeMap::new();
        addrs.insert(worker.public_bytes(), dummy_addr(worker.public_bytes()));
        let mut record = hand_built_record(
            &head,
            &group,
            vec![assignment(worker.public_bytes(), 0, 8)],
            addrs,
        );
        record.group = other_group;
        let registry = ShardSessionRegistry::default();
        let err = registry.insert_gated(record).unwrap_err();
        assert!(
            err.contains("does not match compute group"),
            "manifest↔group binding mismatch must be rejected, got: {err}"
        );
    }

    #[test]
    fn registry_rejects_duplicate_session_id() {
        let head = KeyPair::generate();
        let worker = KeyPair::generate();
        let group = mint_compute_group(&head, "g", 1, &[worker.public_bytes()]).expect("group");
        let mut addrs = BTreeMap::new();
        addrs.insert(worker.public_bytes(), dummy_addr(worker.public_bytes()));
        let registry = ShardSessionRegistry::default();
        let mk = || {
            hand_built_record(
                &head,
                &group,
                vec![assignment(worker.public_bytes(), 0, 8)],
                addrs.clone(),
            )
        };
        registry.insert_gated(mk()).expect("first mount");
        let err = registry.insert_gated(mk()).unwrap_err();
        assert!(
            err.contains("already mounted"),
            "a duplicate session id must never silently overwrite, got: {err}"
        );
    }

    #[test]
    fn place_and_sign_produces_full_coverage_plan_with_fallback() {
        // 3 candidates, a model needing 2 of them: the plan covers
        // [0..total_layers) contiguously and the left-out candidate
        // becomes each stage's plan-time fallback.
        let head = KeyPair::generate();
        let keys: Vec<KeyPair> = (0..3).map(|_| KeyPair::generate()).collect();
        let pubs: Vec<[u8; 32]> = keys.iter().map(|k| k.public_bytes()).collect();
        let group = mint_compute_group(&head, "g", 1, &pubs).expect("group");
        let candidates: Vec<WorkerPlacementProfile> = pubs
            .iter()
            .map(|p| WorkerPlacementProfile {
                worker_pubkey: *p,
                vram_free_bytes: 1_000,
                shard_hashes: vec![],
                launch_profile_hash: [0u8; 32],
            })
            .collect();
        let entry = place_and_sign(&head, "s", &group, &candidates, &two_shard_model())
            .expect("place and sign");
        entry.verify_signature().expect("signed manifest verifies");
        let plan = &entry.manifest.plan;
        assert!(plan.is_pipeline_contiguous());
        assert_eq!(plan.assignments.first().unwrap().layer_start, 0);
        assert_eq!(plan.assignments.last().unwrap().layer_end, 8);
        assert!(
            plan.assignments.iter().all(|a| a.fallback_node.is_some()),
            "with a spare allowlisted candidate every stage gets a plan-time fallback"
        );
        // The fallback is a real member and never the stage's own primary.
        for a in &plan.assignments {
            let fb = a.fallback_node.unwrap();
            assert_ne!(fb, a.worker_pubkey);
            assert!(group.is_member(&fb));
        }
    }

    #[test]
    fn place_and_sign_rejects_single_worker_fit() {
        // A model that fits the largest candidate must NOT mount as a
        // shard session (S76 endpoint federation owns that path).
        let head = KeyPair::generate();
        let worker = KeyPair::generate();
        let group = mint_compute_group(&head, "g", 1, &[worker.public_bytes()]).expect("group");
        let candidates = vec![WorkerPlacementProfile {
            worker_pubkey: worker.public_bytes(),
            vram_free_bytes: 1_000_000,
            shard_hashes: vec![],
            launch_profile_hash: [0u8; 32],
        }];
        let err = place_and_sign(&head, "s", &group, &candidates, &two_shard_model()).unwrap_err();
        assert!(
            err.contains("endpoint federation"),
            "single-worker fit must route to federation, got: {err}"
        );
    }

    #[test]
    fn parse_pubkey_hex_roundtrip_and_reject() {
        let kp = KeyPair::generate();
        let hexed = hex::encode(kp.public_bytes());
        assert_eq!(parse_pubkey_hex(&hexed).unwrap(), kp.public_bytes());
        assert!(parse_pubkey_hex("zz").is_err(), "non-hex rejected");
        assert!(
            parse_pubkey_hex(&"ab".repeat(16)).is_err(),
            "wrong length rejected"
        );
    }

    // ---- In-process two/three-node lifecycle tests (iroh-networked
    // family: green on native Windows + Linux CI, env-blocked under
    // Docker-on-Windows like every `create_node` test) ----

    #[tokio::test(flavor = "multi_thread")]
    async fn mount_readiness_blocks_on_unreachable_shard() {
        // One live echo worker + one dead address: the readiness barrier
        // must fail the mount with a diagnostic, insert NOTHING, and emit
        // ZERO dispatch frames to the live worker.
        let counting = Arc::new(CountingForwarder::default());
        let rig = shard_rig(vec![counting.clone()]).await;

        // A second "worker": a keypair that is a group member with a
        // dial-able-looking address but NO listening node.
        let ghost = KeyPair::generate();
        let group = mint_compute_group(
            &rig.head_kp,
            "test-shard-group-ghost",
            1,
            &[rig.workers[0].2.public_bytes(), ghost.public_bytes()],
        )
        .expect("group");

        let registry = ShardSessionRegistry::default();
        let req = MountSessionRequest {
            session_id: "ghost-session".into(),
            group,
            workers: vec![
                ShardWorkerSpec {
                    addr: rig.workers[0].1.clone(),
                    vram_free_bytes: 1_000,
                    shard_hashes: vec![],
                    launch_profile_hash: [0u8; 32],
                },
                ShardWorkerSpec {
                    addr: dummy_addr(ghost.public_bytes()),
                    vram_free_bytes: 1_000,
                    shard_hashes: vec![],
                    launch_profile_hash: [0u8; 32],
                },
            ],
            model: two_shard_model(),
            readiness_deadline_ms: Some(2_000),
            hop_deadline_ms: Some(2_000),
        };
        let err = mount_session(
            rig.head.endpoint(),
            rig.head.memory_lookup(),
            &rig.head_kp,
            &registry,
            req,
        )
        .await
        .unwrap_err();
        assert!(
            err.contains("readiness"),
            "the mount must fail AT the readiness barrier, got: {err}"
        );
        assert!(
            registry.status_data("ghost-session").is_none(),
            "a session that failed readiness must never exist in the registry"
        );
        assert_eq!(
            counting.0.load(Ordering::SeqCst),
            0,
            "no dispatch frame may be emitted before the barrier passes"
        );
        rig.shutdown().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn session_shard_in_process_full_lifecycle() {
        // T1 sub-test (6): the complete six-step lifecycle over in-process
        // loopback nodes with the transport-only echo seam — placement →
        // signed manifest → readiness ACK → gated insert (live status) →
        // HUB drive → measured outcome + first production RunProof →
        // teardown.
        let rig = shard_rig(vec![Arc::new(EchoForwarder), Arc::new(EchoForwarder)]).await;
        let registry = ShardSessionRegistry::default();

        let report = mount_session(
            rig.head.endpoint(),
            rig.head.memory_lookup(),
            &rig.head_kp,
            &registry,
            mount_request(&rig, "s81-i-lifecycle", &[1_000, 1_000]),
        )
        .await
        .expect("mount");
        assert_eq!(report.member_count, 2, "the model needs both workers");
        assert!(
            report.rtt_frontier_ms.is_some(),
            "the readiness barrier must sample a frontier RTT"
        );

        // The stub contract flips: the session is LIVE.
        let status = registry.status_data("s81-i-lifecycle").expect("mounted");
        assert_eq!(status.member_count, 2);

        // Drive one generation through the pipeline.
        generate_session(
            rig.head.endpoint(),
            rig.head.memory_lookup(),
            &rig.head_kp,
            &registry,
            "s81-i-lifecycle",
            "boundary-activation-prompt",
        )
        .await
        .expect("drive");

        let result = registry.result_data("s81-i-lifecycle").expect("mounted");
        assert_eq!(
            result.result_text.as_deref(),
            Some("boundary-activation-prompt"),
            "the echo pipeline must round-trip the prompt through every stage"
        );
        assert!(result.ttft_s.is_some(), "TTFT must be measured");
        assert!(
            result.toks_per_s.unwrap_or(0) >= 1,
            "the measured rate must satisfy the harness >=1 floor"
        );
        assert_eq!(result.worker_drop_count, 0);
        assert!(result.failure.is_none());
        let proof_hex = result.run_proof.expect("a run proof must be collected");
        assert_eq!(proof_hex.len(), 128, "hex of a 64-byte Ed25519 signature");

        // The stored proof itself verifies and binds THIS session.
        {
            let sessions = registry.lock();
            let record = sessions.get("s81-i-lifecycle").unwrap();
            assert_eq!(record.status, ShardSessionStatus::Complete);
            let run = record.outcome.as_ref().unwrap();
            run.run_proof
                .verify_signature()
                .expect("prod proof verifies");
            assert_eq!(run.run_proof.proof.session_id, "s81-i-lifecycle");
            assert_eq!(
                run.run_proof.proof.participants.len(),
                2,
                "the proof lists the pipeline participants"
            );
        }
        rig.shutdown().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn hop_deadline_without_fallback_fails_clean() {
        // SI-9: an admitted-but-silent worker (stalling forwarder) must
        // trip the per-hop deadline and fail the drive CLEAN with a
        // diagnostic — never hang. The plan is HAND-BUILT with
        // `fallback_node: None`: a mount-placed multi-candidate plan
        // always carries one (the spare candidate IS the fallback pool),
        // so the no-fallback branch — the explicit counted cut — is only
        // reachable through a crafted plan.
        let rig = shard_rig(vec![Arc::new(StallingForwarder(Duration::from_secs(5)))]).await;
        let stalled = rig.workers[0].2.public_bytes();
        let mut addrs = BTreeMap::new();
        addrs.insert(stalled, rig.workers[0].1.clone());
        let mut record = hand_built_record(
            &rig.head_kp,
            &rig.group,
            vec![assignment(stalled, 0, 8)],
            addrs,
        );
        record.hop_deadline = Duration::from_millis(500);
        let registry = ShardSessionRegistry::default();
        registry.insert_gated(record).expect("gated insert");

        let err = generate_session(
            rig.head.endpoint(),
            rig.head.memory_lookup(),
            &rig.head_kp,
            &registry,
            "hand-built",
            "prompt",
        )
        .await
        .unwrap_err();
        assert!(
            err.contains("no fallback_node"),
            "the failure must diagnose the missing fallback, got: {err}"
        );
        let result = registry.result_data("hand-built").expect("mounted");
        assert_eq!(result.worker_drop_count, 1, "the drop must be counted");
        assert!(
            result
                .failure
                .as_deref()
                .unwrap_or("")
                .contains("failing clean"),
            "the diagnostic must be readable from the result route"
        );
        assert!(result.result_text.is_none(), "no hollow result on failure");
        rig.shutdown().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn hop_deadline_reroutes_to_fallback_and_resumes() {
        // Churn resume-from-cache: a hand-built plan whose stalled stage
        // carries a healthy fallback member. The drive must count the
        // drop, re-probe the fallback, replay the stage input from the
        // activation cache, and COMPLETE.
        let rig = shard_rig(vec![
            Arc::new(StallingForwarder(Duration::from_secs(5))),
            Arc::new(EchoForwarder),
        ])
        .await;
        let stalled = rig.workers[0].2.public_bytes();
        let healthy = rig.workers[1].2.public_bytes();
        let mut addrs = BTreeMap::new();
        addrs.insert(stalled, rig.workers[0].1.clone());
        addrs.insert(healthy, rig.workers[1].1.clone());

        let mut a = assignment(stalled, 0, 8);
        a.fallback_node = Some(healthy);
        let mut record = hand_built_record(&rig.head_kp, &rig.group, vec![a], addrs);
        record.hop_deadline = Duration::from_millis(500);
        record.readiness_deadline = Duration::from_secs(10);
        let registry = ShardSessionRegistry::default();
        registry.insert_gated(record).expect("gated insert");

        generate_session(
            rig.head.endpoint(),
            rig.head.memory_lookup(),
            &rig.head_kp,
            &registry,
            "hand-built",
            "resume-me",
        )
        .await
        .expect("the fallback re-route must complete the drive");

        let result = registry.result_data("hand-built").expect("mounted");
        assert_eq!(
            result.result_text.as_deref(),
            Some("resume-me"),
            "the replayed frame must traverse the fallback"
        );
        assert_eq!(
            result.worker_drop_count, 1,
            "the churn drop must be counted exactly once"
        );
        assert!(
            result.failure.is_none(),
            "a recovered drive is not a failure"
        );
        rig.shutdown().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn drop_shard_counts_an_explicit_cut() {
        let rig = shard_rig(vec![Arc::new(EchoForwarder), Arc::new(EchoForwarder)]).await;
        let registry = ShardSessionRegistry::default();
        mount_session(
            rig.head.endpoint(),
            rig.head.memory_lookup(),
            &rig.head_kp,
            &registry,
            mount_request(&rig, "s81-i-drop", &[1_000, 1_000]),
        )
        .await
        .expect("mount");

        assert_eq!(registry.drop_tail_shard("s81-i-drop"), Some(true));
        let result = registry.result_data("s81-i-drop").expect("mounted");
        assert_eq!(result.worker_drop_count, 1);
        assert_eq!(
            registry.drop_tail_shard("missing-session"),
            None,
            "dropping an unmounted session reports a miss, not a panic"
        );
        rig.shutdown().await;
    }

    /// A `sbfb/shard/1` handler that ADMITS the peer and accepts the
    /// bi-stream but never reads a byte (byzantine withholding on the
    /// WRITE side): the dialer's flow-control window fills and its
    /// `write_all` blocks. Both streams are HELD open (dropping the recv
    /// would send STOP_SENDING and error the peer's write instead of
    /// stalling it — the stall is the point).
    #[derive(Debug, Clone)]
    struct BlackholeProtocol;
    impl iroh::protocol::ProtocolHandler for BlackholeProtocol {
        async fn accept(
            &self,
            conn: iroh::endpoint::Connection,
        ) -> std::result::Result<(), iroh::protocol::AcceptError> {
            let (send, recv) = conn.accept_bi().await?;
            conn.closed().await;
            drop((send, recv));
            Ok(())
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn hop_deadline_bounds_the_write_path() {
        // Review D1-1/D2-1 (the P1): an admitted worker that accepts the
        // bi-stream but never drains its recv stalls the head's
        // `write_all` on a frame larger than the QUIC flow-control
        // window. The SI-9 hop deadline must free the WRITE path too —
        // the drive fails CLEAN instead of hanging forever.
        let head_secret = KeyPair::generate().secret_bytes();
        let head_kp = KeyPair::from_secret_bytes(&head_secret);
        let blackhole_secret = KeyPair::generate().secret_bytes();
        let blackhole_kp = KeyPair::from_secret_bytes(&blackhole_secret);
        let group = mint_compute_group(
            &head_kp,
            "blackhole-group",
            1,
            &[blackhole_kp.public_bytes()],
        )
        .expect("group");

        let head = create_node_with_config(NodeConfig::default().with_secret_key(head_secret))
            .await
            .expect("head node");
        let factory: nexus_core_rs::node::ExtraProtocolFactory =
            Box::new(move |_store, _ep, _ml| {
                Box::new(BlackholeProtocol) as Box<dyn iroh::protocol::DynProtocolHandler>
            });
        let blackhole = create_node_with_protocols(
            NodeConfig::default().with_secret_key(blackhole_secret),
            vec![(SHARD_ALPN.to_vec(), factory)],
        )
        .await
        .expect("blackhole node");
        let bh_addr = DiscoveryClient::new(blackhole.endpoint())
            .my_endpoint_addr()
            .await
            .expect("blackhole addr");

        let mut addrs = BTreeMap::new();
        addrs.insert(blackhole_kp.public_bytes(), bh_addr);
        let mut record = hand_built_record(
            &head_kp,
            &group,
            vec![assignment(blackhole_kp.public_bytes(), 0, 8)],
            addrs,
        );
        record.hop_deadline = Duration::from_millis(1_000);
        let registry = ShardSessionRegistry::default();
        registry.insert_gated(record).expect("gated insert");

        // 64 MiB frame >> any default QUIC stream/connection window: the
        // write MUST backpressure before completing.
        let big_prompt = "x".repeat(64 * 1024 * 1024);
        let err = generate_session(
            head.endpoint(),
            head.memory_lookup(),
            &head_kp,
            &registry,
            "hand-built",
            &big_prompt,
        )
        .await
        .unwrap_err();
        assert!(
            err.contains("SI-9"),
            "a write-side stall must be freed by the hop deadline, got: {err}"
        );
        let result = registry.result_data("hand-built").expect("mounted");
        assert_eq!(
            result.worker_drop_count, 1,
            "the write-stall is counted churn"
        );
        assert!(result.result_text.is_none(), "no hollow result on failure");

        head.shutdown().await.ok();
        blackhole.shutdown().await.ok();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn generate_rejects_concurrent_drive() {
        // Review D3-1 (P2): the `status == Generating` check-and-set under
        // the registry lock is the ONLY guard against two concurrent
        // drives (double RunProof, clobbered outcome). Pin it.
        let head_secret = KeyPair::generate().secret_bytes();
        let head_kp = KeyPair::from_secret_bytes(&head_secret);
        let head = create_node_with_config(NodeConfig::default().with_secret_key(head_secret))
            .await
            .expect("head node");
        let worker = KeyPair::generate();
        let group = mint_compute_group(&head_kp, "g", 1, &[worker.public_bytes()]).expect("group");
        let mut addrs = BTreeMap::new();
        addrs.insert(worker.public_bytes(), dummy_addr(worker.public_bytes()));
        let record = hand_built_record(
            &head_kp,
            &group,
            vec![assignment(worker.public_bytes(), 0, 8)],
            addrs,
        );
        let registry = ShardSessionRegistry::default();
        registry.insert_gated(record).expect("gated insert");

        // Simulate an in-flight drive: the guard must reject BEFORE any
        // network activity (the dummy addr is never dialed).
        registry.lock().get_mut("hand-built").unwrap().status = ShardSessionStatus::Generating;
        let err = generate_session(
            head.endpoint(),
            head.memory_lookup(),
            &head_kp,
            &registry,
            "hand-built",
            "prompt",
        )
        .await
        .unwrap_err();
        assert!(
            err.contains("already generating"),
            "a concurrent drive must be rejected by the status guard, got: {err}"
        );
        head.shutdown().await.ok();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn successful_mount_emits_zero_frames_before_generate() {
        // Review D3-2 (P2): the flagship invariant — a SUCCESSFUL mount's
        // readiness barrier emits ZERO dispatch frames (the probe is the
        // handshake + RTT sample only); the frames start with generate.
        // The unreachable-shard test asserts count==0 on a FAILED mount,
        // which is trivially true; this pins the happy path.
        let c1 = Arc::new(CountingForwarder::default());
        let c2 = Arc::new(CountingForwarder::default());
        let rig = shard_rig(vec![c1.clone(), c2.clone()]).await;
        let registry = ShardSessionRegistry::default();
        mount_session(
            rig.head.endpoint(),
            rig.head.memory_lookup(),
            &rig.head_kp,
            &registry,
            mount_request(&rig, "s81-i-zeroframe", &[1_000, 1_000]),
        )
        .await
        .expect("mount");
        assert_eq!(
            c1.0.load(Ordering::SeqCst) + c2.0.load(Ordering::SeqCst),
            0,
            "a successful mount must emit ZERO dispatch frames"
        );

        generate_session(
            rig.head.endpoint(),
            rig.head.memory_lookup(),
            &rig.head_kp,
            &registry,
            "s81-i-zeroframe",
            "frame-me",
        )
        .await
        .expect("drive");
        assert_eq!(
            c1.0.load(Ordering::SeqCst),
            1,
            "the drive sends exactly one frame through stage 1"
        );
        assert_eq!(
            c2.0.load(Ordering::SeqCst),
            1,
            "the drive sends exactly one frame through stage 2"
        );
        rig.shutdown().await;
    }
}
