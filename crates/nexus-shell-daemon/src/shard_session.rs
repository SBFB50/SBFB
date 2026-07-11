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
//!   verification anchor. Since Sprint 81 Phase K the driver ENFORCES
//!   that anchor at every stage-link establishment: the stage attests the
//!   `{model_digest, window, roles}` it actually loaded and any mismatch
//!   with the signed plan fail-closes the drive BEFORE a step frame flows
//!   ([`attest_stage_link`] — closes the misconfiguration class; a
//!   deliberately lying stage stays the SI-4 residual).
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
use nexus_core_rs::shard::{
    conn_rtt, open_shard_connection, read_frame, request_stage_attestation,
    verify_stage_attestation, write_frame,
};
use nexus_core_rs::shard_plan::{
    RunMetrics, RunProof, RunProofEntry, ShardAssignment, ShardedSessionManifest,
    ShardedSessionManifestEntry,
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

/// Default number of new tokens a REAL inference drive generates when the
/// request does not say (Sprint 81 Phase J, Option B). Short by design:
/// the per-step recompute (no cross-step KV reuse, the F2 carry) makes
/// long generations quadratic — the live benchmark measures a bounded,
/// deterministic decode, not a chat.
pub const DEFAULT_MAX_NEW_TOKENS: u32 = 16;

/// Hard cap on `max_tokens` accepted by the generate route — bounds the
/// drive duration (anti-DoS on the loopback surface; the per-step frame
/// is already bounded by `MAX_SHARD_FRAME_BYTES`).
pub const MAX_NEW_TOKENS_CAP: u32 = 256;

/// Hard cap on the accumulated `result_text` of one decode drive (review
/// J D3-1). A byzantine ADMITTED tail controls each reply's `piece` up to
/// the 256 MiB frame cap; without a cumulative bound the registry would
/// hold (and re-serialize on every `GET /result`) gigabytes. 64 KiB is
/// ~256 bytes per token at the `MAX_NEW_TOKENS_CAP` — generous for any
/// legitimate detokenized text, unreachable except by misbehaviour.
pub const MAX_RESULT_TEXT_BYTES: usize = 64 * 1024;

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
    /// Output tokens of the last drive (transport-only echo = 1; a REAL
    /// decode reports its generated token count — the harness'
    /// anti-false-green tell, Phase J).
    pub tokens: Option<u64>,
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

    /// Lifecycle status of a mounted session (`None` = not mounted). The
    /// generate route reads it for a best-effort 409 precheck (review Cible
    /// 2 P2 — GPT-5.6 Sol): the atomic check-and-set in `generate_session`
    /// remains the real backstop against a double-drive, but the precheck
    /// lets a concurrent generate see `409 already generating` instead of a
    /// misleading `202 accepted` whose spawned drive silently no-ops.
    pub fn status_of(&self, session_id: &str) -> Option<ShardSessionStatus> {
        self.lock().get(session_id).map(|r| r.status)
    }

    /// Result data for the result route (`None` = not mounted). Fields
    /// stay `None` until a drive completes; `failure` carries the clean
    /// diagnostic of a failed drive.
    pub fn result_data(&self, session_id: &str) -> Option<SessionResultData> {
        let sessions = self.lock();
        sessions.get(session_id).map(|r| {
            let outcome = r.outcome.as_ref();
            let transport_only = r.entry.manifest.model_digest == [0u8; 32];
            SessionResultData {
                session_id: r.entry.manifest.session_id.clone(),
                result_text: outcome.map(|o| o.result_text.clone()),
                ttft_s: outcome.map(|o| o.ttft_ms / 1000),
                toks_per_s: outcome.map(|o| {
                    let rate = o.tokens.saturating_mul(1000) / o.decode_ms.max(1);
                    if transport_only {
                        // Echo plumbing proof: the "token" is one frame
                        // pass; floor-guard an instant drive so the value
                        // stays a liveness signal, never a rate claim.
                        rate.max(1)
                    } else {
                        // REAL decode (Phase J): report the measured rate
                        // UNFLOORED — a sub-1 tok/s pipeline must surface
                        // as 0 so the harness' >=1 gate BLOCKs honestly
                        // (preflight R-J-4: no anti-false-green bypass).
                        rate
                    }
                }),
                tokens: outcome.map(|o| o.tokens),
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
            .map_err(|e| {
                format!(
                    "readiness: shard {worker_hex} dial failed: {}",
                    sanitize_diagnostic(&e.to_string())
                )
            })?;
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
            .map_err(|e| format!("open_bi failed: {}", sanitize_diagnostic(&e.to_string())))?;
        write_frame(&mut send, input).await.map_err(|e| {
            format!(
                "frame write failed: {}",
                sanitize_diagnostic(&e.to_string())
            )
        })?;
        let out = read_frame(&mut recv)
            .await
            .map_err(|e| format!("frame read failed: {}", sanitize_diagnostic(&e.to_string())))?
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

/// Scrub a transport-error string before it is logged or projected into
/// `/result` (review Cible 2 P1 — GPT-5.6 Sol). A byzantine worker
/// controls its QUIC application-close reason, so `open_bi failed: {e}` /
/// `frame read failed: {e}` carry attacker-influenced bytes. This:
/// - strips control characters (log-injection / newline splitting) and
///   collapses whitespace to single spaces;
/// - **redacts any hex run >= 32 chars** — a full 32-byte pubkey is 64 hex
///   chars, so an identity echoed into a close reason never reaches the
///   projection (SI-3/SI-4). Our own 8-byte (16-hex) truncations survive;
/// - caps the length so a pathological reason cannot bloat the store/log.
pub fn sanitize_diagnostic(s: &str) -> String {
    const REDACT_HEX_RUN: usize = 32;
    const MAX_LEN: usize = 240;
    let mut out = String::with_capacity(s.len().min(MAX_LEN));
    let mut hexrun = String::new();
    let flush = |out: &mut String, hexrun: &mut String| {
        if hexrun.len() >= REDACT_HEX_RUN {
            out.push_str("[redacted-hex]");
        } else {
            out.push_str(hexrun);
        }
        hexrun.clear();
    };
    for ch in s.chars() {
        if ch.is_ascii_hexdigit() {
            hexrun.push(ch);
        } else {
            flush(&mut out, &mut hexrun);
            out.push(if ch.is_control() { ' ' } else { ch });
        }
    }
    flush(&mut out, &mut hexrun);
    out.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(MAX_LEN)
        .collect()
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
    max_new_tokens: u32,
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
        // Clear the PREVIOUS run's outcome/failure now (review Cible 2 P1 —
        // GPT-5.6 Sol): otherwise `/result` would serve the stale
        // result_text / RunProof of the last drive while this one is in
        // flight, and the harness poll loop (breaks on non-empty
        // result_text) would false-green on old data.
        record.outcome = None;
        record.failure = None;
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
        max_new_tokens,
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
            // Belt-and-braces: every drive_hop/dial diagnostic is already
            // sanitized at its source, but scrub once more at the single
            // chokepoint that both logs and projects it (review Cible 2 P1).
            let safe = sanitize_diagnostic(&diagnostic);
            warn!(session = %session_id, diagnostic = %safe, "shard drive failed clean");
            registry.mark_failed(session_id, safe.clone());
            Err(safe)
        }
    }
}

/// Persistent per-stage bi-stream for the multi-step decode drive
/// (Sprint 81 Phase J, Option B). The worker's accept loop serves frames
/// on ONE bi-stream until FIN, so a real generation keeps a single
/// stream per stage for its whole lifetime instead of a stream per hop
/// (the D2 long-lived reuse contract, now actually exercised multi-frame).
struct StageLink {
    conn: Connection,
    send: iroh::endpoint::SendStream,
    recv: iroh::endpoint::RecvStream,
}

/// Open the stage's long-lived bi-stream, bounded by `deadline`.
async fn open_stage_link(conn: Connection, deadline: Duration) -> Result<StageLink, String> {
    let (send, recv) = tokio::time::timeout(deadline, conn.open_bi())
        .await
        .map_err(|_| format!("open_bi exceeded its {deadline:?} deadline"))?
        .map_err(|e| format!("open_bi failed: {}", sanitize_diagnostic(&e.to_string())))?;
    Ok(StageLink { conn, send, recv })
}

/// One step exchange on a persistent stage link — the WHOLE write + read
/// under ONE deadline (SI-9, same rationale as [`drive_hop`]: the write
/// path backpressures on QUIC flow control, so a byzantine worker that
/// never drains its recv would otherwise stall a large-frame write
/// forever).
async fn step_hop(
    link: &mut StageLink,
    input: &[u8],
    deadline: Duration,
) -> Result<Vec<u8>, String> {
    tokio::time::timeout(deadline, async {
        write_frame(&mut link.send, input).await.map_err(|e| {
            format!(
                "frame write failed: {}",
                sanitize_diagnostic(&e.to_string())
            )
        })?;
        read_frame(&mut link.recv)
            .await
            .map_err(|e| format!("frame read failed: {}", sanitize_diagnostic(&e.to_string())))?
            .ok_or_else(|| "stream finished before an output frame".to_string())
    })
    .await
    .map_err(|_| {
        format!(
            "hop exceeded its {deadline:?} deadline \
             (SI-9 withholding guard — covers write/read)"
        )
    })?
}

/// Sprint 81 Phase K — the loaded-stage ↔ signed-manifest binding
/// (THREAT_MODEL §16), run at the establishment of EVERY stage link of a
/// REAL session: the stage self-declares `{model_digest, layer window,
/// roles}` of the backend it actually loaded
/// ([`request_stage_attestation`]) and the driver fail-closes on any
/// mismatch with the signed manifest + [`ShardAssignment`]
/// ([`verify_stage_attestation`]). The stage link is the single chokepoint
/// every data frame of a real session crosses (first drive, re-dials AND
/// fallback re-routes), so no step frame ever reaches an unattested
/// executor. Attestation is a SELF-CLAIM by an admitted member: it closes
/// the MISCONFIGURATION class (echo left serving, wrong window, wrong
/// model), not a deliberately byzantine stage (SI-4 residual). The whole
/// exchange runs under the hop deadline (SI-9, same budget as any hop).
async fn attest_stage_link(
    link: &mut StageLink,
    manifest_model_digest: &[u8; 32],
    assignment: &ShardAssignment,
    stage_index: usize,
    stage_count: usize,
    deadline: Duration,
) -> Result<(), String> {
    tokio::time::timeout(deadline, async {
        let att = request_stage_attestation(&mut link.send, &mut link.recv)
            .await
            .map_err(|e| {
                format!(
                    "attestation exchange failed: {}",
                    sanitize_diagnostic(&e.to_string())
                )
            })?;
        verify_stage_attestation(
            &att,
            manifest_model_digest,
            assignment,
            stage_index,
            stage_count,
        )
        .map_err(|e| sanitize_diagnostic(&e.to_string()))
    })
    .await
    .map_err(|_| format!("attestation exceeded its {deadline:?} deadline (SI-9 guard)"))?
}

/// The measured pipeline walk. Split from [`generate_session`] so the
/// teardown + status bookkeeping wrap it exactly once. Dispatches on the
/// manifest's `model_digest`: 32 zeros = the S77/Phase-I transport-only
/// echo pass (one frame through the pipeline, plumbing proof);
/// anything else = the Phase J REAL decode loop
/// ([`drive_decode_loop`], PO arbitrage Option B).
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
    max_new_tokens: u32,
) -> Result<ShardRunOutcome, String> {
    if entry.manifest.model_digest != [0u8; 32] {
        return drive_decode_loop(
            endpoint,
            lookup,
            keypair,
            registry,
            entry,
            addrs,
            conns,
            used,
            hop_deadline,
            readiness_deadline,
            session_id,
            prompt,
            max_new_tokens,
        )
        .await;
    }
    let started = Instant::now();
    let mut replay = ActivationReplayCache::new();
    let mut frame: Vec<u8> = prompt.as_bytes().to_vec();
    let mut rx_bytes: u64 = 0;
    let mut tx_bytes: u64 = 0;
    let mut drops: u32 = 0;
    // The workers that ACTUALLY executed each stage (primary on the healthy
    // path, fallback on a reroute). The RunProof `participants` must be the
    // real executors, never the plan's primaries (review Cible 2 P1 —
    // GPT-5.6 Sol: a signed proof that names a dropped worker as participant
    // is a factually false attestation).
    let mut executed_by: Vec<[u8; 32]> = Vec::with_capacity(entry.manifest.plan.assignments.len());

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
                .map_err(|e| {
                    format!(
                        "stage {i} dial failed: {}",
                        sanitize_diagnostic(&e.to_string())
                    )
                })?
            }
        };

        tx_bytes = tx_bytes.saturating_add(frame.len() as u64);
        let hop_result = drive_hop(&conn, &frame, hop_deadline).await;
        let out = match hop_result {
            Ok(out) => {
                // Spent (one bi-stream per connection): park for teardown,
                // never back into the reusable pool.
                used.push(conn);
                executed_by.push(a.worker_pubkey);
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
                // The FALLBACK executed this stage — record it, not the
                // dropped primary (review Cible 2 P1).
                executed_by.push(fallback);
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
    // measured (the first production RunProofEntry::sign call-site). The
    // participants are the workers that ACTUALLY executed each stage
    // (fallbacks substituted for dropped primaries), never the plan's
    // primaries — an honest signed attestation (review Cible 2 P1).
    let participants: Vec<[u8; 32]> = executed_by;
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

/// Decode a last-shard reply's `toploc_hex` (lowercase blake3 hex, 64
/// chars) into the 32-byte N0 TOPLOC commitment, returning zeros ("not
/// provided") for an empty / wrong-length / non-hex value. Total (never
/// errors) so [`drive_decode_loop`] can assign the fingerprint on EVERY
/// step — the LAST reply always decides what the RunProof signs
/// (Codex GPT-5.6 Sol P1).
fn parse_toploc_hex(toploc_hex: &str) -> [u8; 32] {
    if toploc_hex.len() != 64 {
        return [0u8; 32];
    }
    match hex::decode(toploc_hex) {
        Ok(bytes) => <[u8; 32]>::try_from(bytes.as_slice()).unwrap_or([0u8; 32]),
        Err(_) => [0u8; 32],
    }
}

/// Sprint 81 Phase J (PO arbitrage Option B) — the REAL autoregressive
/// decode drive over `sbfb/shard/1`.
///
/// Per generated token, the HUB walks the pipeline once (stateless
/// per-step recompute — no cross-step KV reuse, the F2 carry): the FIRST
/// stage receives a [`nexus_core_rs::ShardStepRequest`] JSON payload
/// (prompt + generated ids — the first shard owns the tokenizer), every
/// middle stage exchanges the raw fp32-LE boundary tensor, and the LAST
/// stage answers a [`nexus_core_rs::ShardStepReply`] (greedy-sampled
/// token + N0 TOPLOC commitment). Statelessness is what makes the SI-9
/// churn semantics CORRECT mid-decode: a stage's step input replays on
/// its plan-time fallback with zero lost state (the fallback then owns
/// the stage for the remaining steps).
///
/// Measurement: TTFT = time to the FIRST step reply; the decode rate is
/// tokens over the whole drive; the LAST step's TOPLOC commitment lands
/// in `RunProof::activation_fingerprint` (first production binding of the
/// Phase G slot). Participants = every worker that actually executed at
/// least one step (fallbacks included), never the plan's primaries.
#[allow(clippy::too_many_arguments)]
async fn drive_decode_loop(
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
    max_new_tokens: u32,
) -> Result<ShardRunOutcome, String> {
    use nexus_core_rs::{ShardStepReply, ShardStepRequest};

    let started = Instant::now();
    let mut replay = ActivationReplayCache::new();
    let mut rx_bytes: u64 = 0;
    let mut tx_bytes: u64 = 0;
    let mut drops: u32 = 0;
    let mut participants: Vec<[u8; 32]> = Vec::new();

    let assignments = entry.manifest.plan.assignments.clone();

    /// Live driving state of one pipeline stage.
    struct StageState {
        /// The worker currently owning this stage (fallback after churn).
        exec: [u8; 32],
        /// The plan-time fallback, consumed by the first churn.
        fallback: Option<[u8; 32]>,
        /// The persistent bi-stream (opened lazily on first use).
        link: Option<StageLink>,
    }
    let mut stages: Vec<StageState> = assignments
        .iter()
        .map(|a| StageState {
            exec: a.worker_pubkey,
            fallback: a.fallback_node,
            link: None,
        })
        .collect();

    let max_new = max_new_tokens.clamp(1, MAX_NEW_TOKENS_CAP);
    let mut generated: Vec<i32> = Vec::with_capacity(max_new as usize);
    let mut text = String::new();
    let mut ttft_ms: Option<u64> = None;
    let mut fingerprint = [0u8; 32];

    'steps: for _step in 0..max_new {
        let mut frame: Vec<u8> = ShardStepRequest::new(prompt, generated.clone()).encode();

        for (i, a) in assignments.iter().enumerate() {
            // Resume point for THIS stage at THIS step: its input frame,
            // keyed by the frontier layer (mirror of the transport path).
            replay.insert(a.layer_start, frame.clone());
            let st = &mut stages[i];

            // Ensure the stage link exists: reuse the readiness-barrier
            // connection on the first step, else dial fresh (bounded by
            // the readiness deadline, review D2-2).
            if st.link.is_none() {
                let conn = match conns.remove(&st.exec) {
                    Some(c) => c,
                    None => {
                        let addr = addrs
                            .get(&st.exec)
                            .ok_or_else(|| format!("no dial address for stage {i} worker"))?
                            .clone();
                        tokio::time::timeout(
                            readiness_deadline,
                            open_shard_connection(endpoint, lookup, addr),
                        )
                        .await
                        .map_err(|_| format!("stage {i} re-dial exceeded {readiness_deadline:?}"))?
                        .map_err(|e| {
                            format!(
                                "stage {i} dial failed: {}",
                                sanitize_diagnostic(&e.to_string())
                            )
                        })?
                    }
                };
                let mut link = open_stage_link(conn, hop_deadline)
                    .await
                    .map_err(|e| format!("stage {i} stream open failed: {e}"))?;
                // Phase K binding: NO step frame flows to this executor
                // until it attests the loaded stage the signed plan expects.
                attest_stage_link(
                    &mut link,
                    &entry.manifest.model_digest,
                    a,
                    i,
                    assignments.len(),
                    hop_deadline,
                )
                .await
                .map_err(|e| format!("stage {i} attestation rejected: {e} — failing closed"))?;
                st.link = Some(link);
            }

            tx_bytes = tx_bytes.saturating_add(frame.len() as u64);
            let link = st.link.as_mut().expect("link ensured above");
            let out = match step_hop(link, &frame, hop_deadline).await {
                Ok(out) => out,
                Err(hop_err) => {
                    // Churn (SI-9 fired MID-DECODE): count the drop, close
                    // the stalled link, re-route the stage to its plan-time
                    // fallback and REPLAY this step's stage input — or fail
                    // clean. The fallback owns the stage from here on.
                    let stalled = st.link.take().expect("link existed");
                    stalled.conn.close(0u32.into(), b"hop-deadline");
                    used.push(stalled.conn);
                    drops = drops.saturating_add(1);
                    registry.count_drop(session_id);
                    let worker_hex = hex::encode(&st.exec[..8]);
                    let Some(fallback) = st.fallback.take() else {
                        return Err(format!(
                            "stage {i} worker {worker_hex} failed mid-decode ({hop_err}) and \
                             the plan carries no fallback_node — failing clean instead of \
                             hanging"
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
                        "mid-decode hop deadline fired — re-routing to fallback \
                         (stateless resume-from-cache)"
                    );
                    st.exec = fallback;
                    let (fb_conn, _rtt) =
                        probe_shard_readiness(endpoint, lookup, fb_addr, readiness_deadline)
                            .await
                            .map_err(|e| format!("stage {i} fallback {fb_hex} not ready: {e}"))?;
                    let mut fb_link =
                        open_stage_link(fb_conn, hop_deadline).await.map_err(|e| {
                            format!("stage {i} fallback {fb_hex} stream open failed: {e}")
                        })?;
                    // Phase K binding, fallback edition: the re-routed
                    // executor must attest the SAME stage window the plan
                    // assigns before the cached input is replayed to it.
                    attest_stage_link(
                        &mut fb_link,
                        &entry.manifest.model_digest,
                        a,
                        i,
                        assignments.len(),
                        hop_deadline,
                    )
                    .await
                    .map_err(|e| {
                        format!(
                            "stage {i} fallback {fb_hex} attestation rejected: {e} — \
                             failing closed"
                        )
                    })?;
                    st.link = Some(fb_link);
                    let cached = replay
                        .get(a.layer_start)
                        .expect("stage input was inserted before dispatch")
                        .to_vec();
                    tx_bytes = tx_bytes.saturating_add(cached.len() as u64);
                    let link = st.link.as_mut().expect("fallback link set above");
                    step_hop(link, &cached, hop_deadline).await.map_err(|e| {
                        format!("stage {i} fallback {fb_hex} also failed ({e}) — failing clean")
                    })?
                }
            };
            if !participants.contains(&st.exec) {
                participants.push(st.exec);
            }
            rx_bytes = rx_bytes.saturating_add(out.len() as u64);
            frame = out;
        }

        // The last stage answered one greedy decode step.
        let reply = ShardStepReply::decode(&frame).map_err(|e| {
            format!(
                "last stage answered an undecodable step reply: {}",
                sanitize_diagnostic(&e.to_string())
            )
        })?;
        if ttft_ms.is_none() {
            ttft_ms = Some(started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64);
        }
        generated.push(reply.token_id);
        // Bounded accumulation (review J D3-1): `piece` is
        // attacker-controlled by an admitted-but-byzantine tail (the SI-9
        // adversary already modeled), and MAX_SHARD_FRAME_BYTES alone
        // bounds ONE reply at 256 MiB — the cumulative product would be
        // gigabytes held in the registry and re-serialized on every
        // GET /result. A legitimate greedy decode piece is a few bytes;
        // blowing the cap is byzantine behaviour → fail CLEAN, never a
        // silent truncation of a signed-run's text.
        if text.len().saturating_add(reply.piece.len()) > MAX_RESULT_TEXT_BYTES {
            return Err(format!(
                "last stage reply grows result_text past MAX_RESULT_TEXT_BYTES \
                 ({MAX_RESULT_TEXT_BYTES} bytes) — byzantine oversized piece, failing clean"
            ));
        }
        text.push_str(&reply.piece);
        // Re-assign the fingerprint on EVERY step (Codex GPT-5.6 Sol P1):
        // the RunProof contract is « the LAST step's N0 TOPLOC commitment ».
        // A conditional update kept the previous step's fingerprint when the
        // final reply carried an empty/invalid toploc (the payload allows an
        // empty toploc) — signing a stale commitment as if it were the last
        // step's. Assign unconditionally, defaulting to zeros (« not
        // provided ») so the LAST reply always decides.
        fingerprint = parse_toploc_hex(&reply.toploc_hex);
        if reply.is_eos {
            break 'steps;
        }
    }

    // Park every live link for the caller's teardown, FINishing each send
    // stream first (review J J1b-1): the worker's accept loop reads frames
    // until a clean FIN — without it the QUIC close surfaces as an
    // AcceptError on every healthy session instead of the documented
    // clean-FIN termination (`drive_hop` honors the same contract).
    for st in stages {
        if let Some(mut link) = st.link {
            link.send.finish().ok();
            used.push(link.conn);
        }
    }

    let tokens = generated.len() as u64;
    if tokens == 0 {
        return Err("decode loop produced zero tokens".to_string());
    }
    let ttft_ms =
        ttft_ms.unwrap_or_else(|| started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64);
    let decode_ms = started
        .elapsed()
        .as_millis()
        .min(u128::from(u64::MAX))
        .max(1) as u64;

    let metrics = RunMetrics {
        ttft_ms,
        decode_milli_tokens_per_sec: tokens.saturating_mul(1_000_000) / decode_ms,
        p95_token_latency_ms: decode_ms / tokens,
        network_rx_bytes: rx_bytes,
        network_tx_bytes: tx_bytes,
        worker_drop_count: drops,
    };

    let mut proof = RunProof::new(
        keypair.public_bytes(),
        session_id,
        entry.manifest.model_digest,
        nexus_core_rs::crypto::blake3_hash(prompt.as_bytes()),
        metrics,
        participants,
    );
    // First production binding of the Phase G slot: the LAST step's N0
    // TOPLOC commitment from the last shard (zeros when the tail could
    // not provide one — "not provided", never a fake).
    proof.activation_fingerprint = fingerprint;
    let run_proof =
        RunProofEntry::sign(proof, keypair).map_err(|e| format!("run proof sign failed: {e}"))?;
    run_proof
        .verify_signature()
        .map_err(|e| format!("freshly signed run proof failed verification: {e}"))?;

    Ok(ShardRunOutcome {
        result_text: text,
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

        /// Shut down only the worker nodes (keep the head alive) so a
        /// subsequent drive re-dials into dead addresses and fails —
        /// used to exercise the stale-result-clear path.
        async fn shutdown_workers(&mut self) {
            for (n, _, _) in self.workers.drain(..) {
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
    /// MUST shard across both (never EndpointFederation). `model_digest`
    /// is ZEROS: these fixtures drive echo forwarders, i.e. TRANSPORT
    /// sessions — the drive dispatches on the digest (Phase J), and a
    /// non-zero digest would route the echo rig into the real decode loop.
    fn two_shard_model() -> ShardModelSpec {
        ShardModelSpec {
            total_layers: 8,
            quantized_vram_bytes: 1_500,
            model_digest: [0u8; 32],
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
        // Digest ZEROS = a transport-echo session (see `two_shard_model`);
        // the Phase J decode tests build their record with
        // `hand_built_record_with_digest` instead.
        hand_built_record_with_digest(head_kp, group, assignments, addrs, [0u8; 32])
    }

    fn hand_built_record_with_digest(
        head_kp: &KeyPair,
        group: &ComputeGroupEntry,
        assignments: Vec<ShardAssignment>,
        addrs: BTreeMap<[u8; 32], EndpointAddr>,
        model_digest: [u8; 32],
    ) -> ShardSessionRecord {
        let manifest = ShardedSessionManifest::new(
            head_kp.public_bytes(),
            "hand-built",
            group.group.group_id.clone(),
            1,
            ShardPlan::new(assignments),
            model_digest,
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
            DEFAULT_MAX_NEW_TOKENS,
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
            DEFAULT_MAX_NEW_TOKENS,
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
            DEFAULT_MAX_NEW_TOKENS,
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
        // Review Cible 2 P1 (GPT-5.6 Sol): the signed RunProof must name the
        // worker that ACTUALLY executed (the healthy fallback), NEVER the
        // dropped primary — an honest attestation.
        {
            let sessions = registry.lock();
            let run = sessions
                .get("hand-built")
                .unwrap()
                .outcome
                .as_ref()
                .unwrap();
            let participants = &run.run_proof.proof.participants;
            assert!(
                participants.contains(&healthy),
                "the fallback that executed must be a signed participant"
            );
            assert!(
                !participants.contains(&stalled),
                "the dropped primary must NOT be signed as a participant"
            );
        }
        rig.shutdown().await;
    }

    // ---- Phase J (Option B) — real decode loop, faked stages ----

    /// Wrap any fake forwarder with a self-declared loaded-stage
    /// descriptor — the serve shape of a real backend for the Phase K
    /// attestation the decode drive requires at stage-link establishment.
    #[derive(Debug)]
    struct AttestedForwarder {
        inner: Arc<dyn ShardForwarder>,
        desc: nexus_core_rs::LoadedStageDescriptor,
    }
    impl ShardForwarder for AttestedForwarder {
        fn forward(&self, frame: &[u8]) -> nexus_core_rs::Result<Vec<u8>> {
            self.inner.forward(frame)
        }
        fn loaded_stage(&self) -> Option<nexus_core_rs::LoadedStageDescriptor> {
            Some(self.desc)
        }
    }

    /// The conforming attestations for the `decode_record` fixture: the
    /// manifest pins digest `[9; 32]`, the head owns `[0,4)` and the tail
    /// `[4,8)` (fallback tails attest the SAME window as the stage they
    /// cover).
    fn attested(
        inner: Arc<dyn ShardForwarder>,
        layer_start: u32,
        layer_end: u32,
        is_first: bool,
        is_last: bool,
    ) -> Arc<dyn ShardForwarder> {
        Arc::new(AttestedForwarder {
            inner,
            desc: nexus_core_rs::LoadedStageDescriptor {
                model_digest: [9u8; 32],
                layer_start,
                layer_end,
                is_first,
                is_last,
            },
        })
    }

    fn attested_head(inner: Arc<dyn ShardForwarder>) -> Arc<dyn ShardForwarder> {
        attested(inner, 0, 4, true, false)
    }

    fn attested_tail(inner: Arc<dyn ShardForwarder>) -> Arc<dyn ShardForwarder> {
        attested(inner, 4, 8, false, true)
    }

    /// Fake FIRST-shard forwarder: decodes the step-request JSON and emits
    /// a deterministic `[1, 2]`-shaped fp32 boundary encoding (prompt
    /// length, generated count) — enough for a fake tail to "sample" from.
    #[derive(Debug)]
    struct FakeHeadForwarder;
    impl ShardForwarder for FakeHeadForwarder {
        fn forward(&self, frame: &[u8]) -> nexus_core_rs::Result<Vec<u8>> {
            let req = nexus_core_rs::ShardStepRequest::decode(frame)?;
            let vals = [req.prompt.len() as f32, req.generated.len() as f32];
            let mut out = Vec::with_capacity(8);
            for v in vals {
                out.extend_from_slice(&v.to_le_bytes());
            }
            Ok(out)
        }
    }

    /// Fake LAST-shard forwarder: reads the boundary, "samples" token
    /// `100 + n_generated`, flags EOS once `eos_after` tokens exist.
    #[derive(Debug)]
    struct FakeTailForwarder {
        eos_after: i32,
    }
    impl FakeTailForwarder {
        fn reply_for(&self, frame: &[u8]) -> nexus_core_rs::Result<Vec<u8>> {
            if !frame.len().is_multiple_of(4) {
                return Err(nexus_core_rs::NexusError::Other(
                    "fake tail fed a non-fp32 frame".into(),
                ));
            }
            let vals: Vec<f32> = frame
                .chunks_exact(4)
                .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect();
            let n_generated = vals.get(1).copied().unwrap_or(0.0) as i32;
            let is_eos = n_generated + 1 >= self.eos_after;
            Ok(nexus_core_rs::ShardStepReply {
                v: nexus_core_rs::SHARD_STEP_PAYLOAD_V,
                token_id: 100 + n_generated,
                piece: if is_eos {
                    String::new()
                } else {
                    format!("tok{n_generated} ")
                },
                is_eos,
                toploc_hex: "cd".repeat(32),
            }
            .encode())
        }
    }
    impl ShardForwarder for FakeTailForwarder {
        fn forward(&self, frame: &[u8]) -> nexus_core_rs::Result<Vec<u8>> {
            self.reply_for(frame)
        }
    }

    /// Fake tail that answers its FIRST step then stalls forever on the
    /// second — the SI-9 mid-decode withholding worker.
    #[derive(Debug)]
    struct FlakyTailForwarder {
        calls: AtomicUsize,
        inner: FakeTailForwarder,
        stall: Duration,
    }
    impl ShardForwarder for FlakyTailForwarder {
        fn forward(&self, frame: &[u8]) -> nexus_core_rs::Result<Vec<u8>> {
            if self.calls.fetch_add(1, Ordering::SeqCst) >= 1 {
                std::thread::sleep(self.stall);
            }
            self.inner.reply_for(frame)
        }
    }

    /// Fake tail whose EOS reply carries an EMPTY toploc_hex while its
    /// non-final replies carry a valid one — exercises the Codex P1
    /// regression (the last step's absent commitment must NOT let the
    /// previous step's fingerprint survive into the RunProof).
    #[derive(Debug)]
    struct BlankFinalToplocTail {
        eos_after: i32,
    }
    impl ShardForwarder for BlankFinalToplocTail {
        fn forward(&self, frame: &[u8]) -> nexus_core_rs::Result<Vec<u8>> {
            let vals: Vec<f32> = frame
                .chunks_exact(4)
                .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect();
            let n_generated = vals.get(1).copied().unwrap_or(0.0) as i32;
            let is_eos = n_generated + 1 >= self.eos_after;
            Ok(nexus_core_rs::ShardStepReply {
                v: nexus_core_rs::SHARD_STEP_PAYLOAD_V,
                token_id: 100 + n_generated,
                piece: if is_eos {
                    String::new()
                } else {
                    format!("tok{n_generated} ")
                },
                is_eos,
                // valid commitment on non-final steps, EMPTY on the last
                toploc_hex: if is_eos {
                    String::new()
                } else {
                    "ab".repeat(32)
                },
            }
            .encode())
        }
    }

    #[test]
    fn parse_toploc_hex_defaults_to_zeros_on_absent_or_invalid() {
        // Codex P1: the fingerprint helper must be total — empty / wrong
        // length / non-hex all map to zeros ("not provided"), a full 64-hex
        // round-trips to its bytes.
        assert_eq!(super::parse_toploc_hex(""), [0u8; 32], "empty → zeros");
        assert_eq!(
            super::parse_toploc_hex("abcd"),
            [0u8; 32],
            "wrong length → zeros"
        );
        assert_eq!(
            super::parse_toploc_hex(&"zz".repeat(32)),
            [0u8; 32],
            "non-hex → zeros"
        );
        assert_eq!(
            super::parse_toploc_hex(&"cd".repeat(32)),
            [0xcd; 32],
            "valid 64-hex → bytes"
        );
    }

    /// A REAL-session record (non-zero model digest) over explicit head /
    /// tail assignments — bypasses the placement so stage ORDER is pinned
    /// (the decode loop requires stage 0 = the tokenizing head).
    fn decode_record(
        rig: &Rig,
        head_idx: usize,
        tail_idx: usize,
        fallback_for_tail: Option<usize>,
    ) -> ShardSessionRecord {
        let head_pk = rig.workers[head_idx].2.public_bytes();
        let tail_pk = rig.workers[tail_idx].2.public_bytes();
        let mut addrs = BTreeMap::new();
        addrs.insert(head_pk, rig.workers[head_idx].1.clone());
        addrs.insert(tail_pk, rig.workers[tail_idx].1.clone());
        let mut tail_assignment = assignment(tail_pk, 4, 8);
        if let Some(fb) = fallback_for_tail {
            let fb_pk = rig.workers[fb].2.public_bytes();
            addrs.insert(fb_pk, rig.workers[fb].1.clone());
            tail_assignment.fallback_node = Some(fb_pk);
        }
        hand_built_record_with_digest(
            &rig.head_kp,
            &rig.group,
            vec![assignment(head_pk, 0, 4), tail_assignment],
            addrs,
            [9u8; 32],
        )
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn decode_loop_generates_until_eos() {
        // A REAL session (non-zero digest) drives the autoregressive loop:
        // step requests to the head, fp32 through the pipe, step replies
        // from the tail, stop on EOS. The outcome must carry the REAL
        // token count, the concatenated pieces, an UNFLOORED rate, and the
        // LAST step's TOPLOC commitment inside the signed RunProof. Both
        // stages attest their loaded window (Phase K binding) — the happy
        // path is byte-identical through a conforming attestation.
        let rig = shard_rig(vec![
            attested_head(Arc::new(FakeHeadForwarder)),
            attested_tail(Arc::new(FakeTailForwarder { eos_after: 3 })),
        ])
        .await;
        let registry = ShardSessionRegistry::default();
        registry
            .insert_gated(decode_record(&rig, 0, 1, None))
            .expect("gated insert");

        generate_session(
            rig.head.endpoint(),
            rig.head.memory_lookup(),
            &rig.head_kp,
            &registry,
            "hand-built",
            "decode-me",
            16,
        )
        .await
        .expect("decode drive completes");

        let result = registry.result_data("hand-built").expect("mounted");
        assert_eq!(
            result.result_text.as_deref(),
            Some("tok0 tok1 "),
            "pieces concatenate in decode order (EOS piece is empty)"
        );
        assert_eq!(result.tokens, Some(3), "EOS on the third sampled token");
        assert!(
            result.failure.is_none(),
            "a completed decode has no failure"
        );
        {
            let sessions = registry.lock();
            let run = sessions
                .get("hand-built")
                .unwrap()
                .outcome
                .as_ref()
                .unwrap();
            assert_eq!(run.tokens, 3);
            assert_eq!(
                run.run_proof.proof.activation_fingerprint, [0xcd; 32],
                "the LAST step's N0 TOPLOC commitment binds the proof"
            );
            run.run_proof
                .verify_signature()
                .expect("driver proof verifies");
            let participants = &run.run_proof.proof.participants;
            assert!(
                participants.contains(&rig.workers[0].2.public_bytes())
                    && participants.contains(&rig.workers[1].2.public_bytes()),
                "both executing stages are signed participants"
            );
        }
        rig.shutdown().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn decode_loop_signs_last_step_fingerprint_even_when_blank() {
        // Codex P1: a final reply with an EMPTY toploc must NOT let the
        // previous step's commitment survive into the signed RunProof — the
        // LAST step decides, defaulting to zeros ("not provided").
        let rig = shard_rig(vec![
            attested_head(Arc::new(FakeHeadForwarder)),
            attested_tail(Arc::new(BlankFinalToplocTail { eos_after: 3 })),
        ])
        .await;
        let registry = ShardSessionRegistry::default();
        registry
            .insert_gated(decode_record(&rig, 0, 1, None))
            .expect("gated insert");

        generate_session(
            rig.head.endpoint(),
            rig.head.memory_lookup(),
            &rig.head_kp,
            &registry,
            "hand-built",
            "blank-final",
            16,
        )
        .await
        .expect("decode completes");

        // Copy the two Copy values out under the lock, then release it
        // BEFORE the await (no MutexGuard held across `shutdown`).
        let (tokens, fingerprint) = {
            let sessions = registry.lock();
            let run = sessions
                .get("hand-built")
                .unwrap()
                .outcome
                .as_ref()
                .unwrap();
            (run.tokens, run.run_proof.proof.activation_fingerprint)
        };
        assert_eq!(tokens, 3, "EOS on the third token");
        assert_eq!(
            fingerprint, [0u8; 32],
            "the EMPTY last-step toploc must sign as zeros, NEVER the prior \
             step's 0xab commitment"
        );
        rig.shutdown().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn decode_loop_respects_max_tokens() {
        // A tail that never EOSes: the drive must stop at the requested
        // max_tokens bound (clamped by MAX_NEW_TOKENS_CAP), never spin.
        let rig = shard_rig(vec![
            attested_head(Arc::new(FakeHeadForwarder)),
            attested_tail(Arc::new(FakeTailForwarder {
                eos_after: i32::MAX,
            })),
        ])
        .await;
        let registry = ShardSessionRegistry::default();
        registry
            .insert_gated(decode_record(&rig, 0, 1, None))
            .expect("gated insert");

        generate_session(
            rig.head.endpoint(),
            rig.head.memory_lookup(),
            &rig.head_kp,
            &registry,
            "hand-built",
            "bounded",
            4,
        )
        .await
        .expect("bounded decode completes");

        let result = registry.result_data("hand-built").expect("mounted");
        assert_eq!(result.tokens, Some(4), "the max_tokens bound is honored");
        assert_eq!(
            result.result_text.as_deref(),
            Some("tok0 tok1 tok2 tok3 "),
            "every bounded step contributed its piece"
        );
        rig.shutdown().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn decode_loop_reroutes_mid_decode_to_fallback() {
        // SI-9 MID-DECODE churn (the R-J-5 live-exercisable path): the tail
        // answers step 0 then withholds step 1; the drive must count the
        // drop, re-probe the plan-time fallback, REPLAY step 1's stage
        // input (stateless recompute makes the replay exact) and finish
        // the generation on the fallback tail.
        let rig = shard_rig(vec![
            attested_head(Arc::new(FakeHeadForwarder)),
            attested_tail(Arc::new(FlakyTailForwarder {
                calls: AtomicUsize::new(0),
                inner: FakeTailForwarder { eos_after: 3 },
                stall: Duration::from_secs(5),
            })),
            // The fallback covers the SAME [4,8) stage — it attests the
            // stage window, and the drive verifies it at re-route time.
            attested_tail(Arc::new(FakeTailForwarder { eos_after: 3 })),
        ])
        .await;
        let registry = ShardSessionRegistry::default();
        let mut record = decode_record(&rig, 0, 1, Some(2));
        record.hop_deadline = Duration::from_millis(500);
        record.readiness_deadline = Duration::from_secs(10);
        registry.insert_gated(record).expect("gated insert");

        generate_session(
            rig.head.endpoint(),
            rig.head.memory_lookup(),
            &rig.head_kp,
            &registry,
            "hand-built",
            "churn-me",
            16,
        )
        .await
        .expect("the mid-decode fallback re-route must complete the drive");

        let result = registry.result_data("hand-built").expect("mounted");
        assert_eq!(result.tokens, Some(3), "the generation completed to EOS");
        assert_eq!(
            result.result_text.as_deref(),
            Some("tok0 tok1 "),
            "the replayed step continued the SAME sequence on the fallback"
        );
        assert_eq!(
            result.worker_drop_count, 1,
            "exactly one counted mid-decode drop"
        );
        {
            let sessions = registry.lock();
            let run = sessions
                .get("hand-built")
                .unwrap()
                .outcome
                .as_ref()
                .unwrap();
            let participants = &run.run_proof.proof.participants;
            assert!(
                participants.contains(&rig.workers[1].2.public_bytes()),
                "the flaky tail EXECUTED step 0 — an honest participant"
            );
            assert!(
                participants.contains(&rig.workers[2].2.public_bytes()),
                "the fallback tail executed the remaining steps"
            );
        }
        rig.shutdown().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn decode_loop_fails_closed_on_unattested_echo_stage() {
        // The exact hole the Phase K binding closes: a transport-only echo
        // stage left serving inside a REAL session. Before the binding it
        // echoed the head's boundary back and the drive SIGNED a
        // plausible-but-wrong result; now the drive fail-closes before any
        // step frame reaches it. The tail is a genuine ECHO stage (Codex P1
        // fix: a transport-only stage does NOT intercept — it echoes the
        // attestation request back) — the
        // driver then fails to decode a valid attestation reply and
        // fail-closes at the exchange, still before any step frame.
        let rig = shard_rig(vec![
            attested_head(Arc::new(FakeHeadForwarder)),
            Arc::new(EchoForwarder), // transport-only echo, NO loaded_stage
        ])
        .await;
        let registry = ShardSessionRegistry::default();
        registry
            .insert_gated(decode_record(&rig, 0, 1, None))
            .expect("gated insert");

        let err = generate_session(
            rig.head.endpoint(),
            rig.head.memory_lookup(),
            &rig.head_kp,
            &registry,
            "hand-built",
            "must-fail-closed",
            16,
        )
        .await
        .expect_err("an unattested stage must fail the drive closed");
        assert!(
            err.contains("attestation rejected"),
            "the diagnostic must name the attestation rejection, got: {err}"
        );
        let result = registry.result_data("hand-built").expect("mounted");
        assert!(
            result.result_text.is_none(),
            "no result_text may survive a fail-closed attestation"
        );
        assert!(
            result.failure.is_some(),
            "the failure diagnostic must be recorded on the session"
        );
        rig.shutdown().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn decode_loop_fails_closed_on_window_mismatch() {
        // A tail that loaded the WRONG layer window ([0,4) instead of the
        // assigned [4,8)) — the mis-windowed fallback scenario from the
        // Phase J carry. The drive must reject it at attestation, never
        // dispatch into it.
        let rig = shard_rig(vec![
            attested_head(Arc::new(FakeHeadForwarder)),
            attested(
                Arc::new(FakeTailForwarder { eos_after: 3 }),
                0,
                4,
                true,
                false,
            ),
        ])
        .await;
        let registry = ShardSessionRegistry::default();
        registry
            .insert_gated(decode_record(&rig, 0, 1, None))
            .expect("gated insert");

        let err = generate_session(
            rig.head.endpoint(),
            rig.head.memory_lookup(),
            &rig.head_kp,
            &registry,
            "hand-built",
            "wrong-window",
            16,
        )
        .await
        .expect_err("a mis-windowed stage must fail the drive closed");
        assert!(
            err.contains("attestation rejected") && err.contains("layer window"),
            "the diagnostic must name the window mismatch, got: {err}"
        );
        rig.shutdown().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn decode_loop_fails_closed_on_model_digest_mismatch() {
        // A stage that loaded a DIFFERENT model than the manifest pins
        // (digest [7;32] vs the signed [9;32]) — right window, wrong
        // weights. The binding must catch it before any step frame.
        let rig = shard_rig(vec![
            attested_head(Arc::new(FakeHeadForwarder)),
            Arc::new(AttestedForwarder {
                inner: Arc::new(FakeTailForwarder { eos_after: 3 }),
                desc: nexus_core_rs::LoadedStageDescriptor {
                    model_digest: [7u8; 32],
                    layer_start: 4,
                    layer_end: 8,
                    is_first: false,
                    is_last: true,
                },
            }),
        ])
        .await;
        let registry = ShardSessionRegistry::default();
        registry
            .insert_gated(decode_record(&rig, 0, 1, None))
            .expect("gated insert");

        let err = generate_session(
            rig.head.endpoint(),
            rig.head.memory_lookup(),
            &rig.head_kp,
            &registry,
            "hand-built",
            "wrong-model",
            16,
        )
        .await
        .expect_err("a wrong-model stage must fail the drive closed");
        assert!(
            err.contains("attestation rejected") && err.contains("model digest"),
            "the diagnostic must name the digest mismatch, got: {err}"
        );
        rig.shutdown().await;
    }

    #[test]
    fn sanitize_diagnostic_redacts_identity_and_control_chars() {
        // Review Cible 2 P1 (GPT-5.6 Sol): a byzantine peer's QUIC close
        // reason is attacker-controlled. A full 64-hex pubkey echoed into a
        // transport error must be redacted before it reaches `failure` /
        // logs; control chars (log injection) stripped; length capped.
        let leak = "ab".repeat(32); // 64 hex chars = a full pubkey
        let dirty = format!("open_bi failed: closed by peer\nreason={leak}\r\tinjected");
        let clean = sanitize_diagnostic(&dirty);
        assert!(
            !clean.contains(&leak),
            "a full-pubkey hex run must be redacted, got: {clean}"
        );
        assert!(clean.contains("[redacted-hex]"), "redaction marker present");
        assert!(
            !clean.contains('\n') && !clean.contains('\r') && !clean.contains('\t'),
            "control chars must be stripped (no log injection)"
        );
        // Our own 8-byte (16-hex) truncations survive (< 32-hex threshold).
        let ours = sanitize_diagnostic("stage 0 worker 0011223344556677 failed");
        assert!(
            ours.contains("0011223344556677"),
            "a 16-hex truncation is preserved, got: {ours}"
        );
        // Length capped.
        assert!(sanitize_diagnostic(&"z".repeat(1000)).len() <= 240);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn new_generation_clears_stale_result() {
        // Review Cible 2 P1 (GPT-5.6 Sol): a second generation must not let
        // `/result` serve the PREVIOUS run's result_text / RunProof (the
        // harness polls on non-empty result_text -> false green). After a
        // successful drive, a second drive that FAILS must clear the old
        // success and surface the failure, not the stale text.
        let mut rig = shard_rig(vec![Arc::new(EchoForwarder), Arc::new(EchoForwarder)]).await;
        let registry = ShardSessionRegistry::default();
        // Short deadlines so the second drive's re-dial into the dead
        // workers fails fast rather than waiting the 10s default.
        let req = MountSessionRequest {
            session_id: "s81-i-stale".into(),
            group: rig.group.clone(),
            workers: rig.worker_specs(&[1_000, 1_000]),
            model: two_shard_model(),
            readiness_deadline_ms: Some(2_000),
            hop_deadline_ms: Some(2_000),
        };
        mount_session(
            rig.head.endpoint(),
            rig.head.memory_lookup(),
            &rig.head_kp,
            &registry,
            req,
        )
        .await
        .expect("mount");
        generate_session(
            rig.head.endpoint(),
            rig.head.memory_lookup(),
            &rig.head_kp,
            &registry,
            "s81-i-stale",
            "first-run",
            DEFAULT_MAX_NEW_TOKENS,
        )
        .await
        .expect("first drive");
        assert_eq!(
            registry
                .result_data("s81-i-stale")
                .unwrap()
                .result_text
                .as_deref(),
            Some("first-run"),
            "first drive result is readable"
        );

        // Drop the workers so the second drive re-dials into the void and
        // fails — the OLD success must be gone, not served stale.
        rig.shutdown_workers().await;
        let _ = generate_session(
            rig.head.endpoint(),
            rig.head.memory_lookup(),
            &rig.head_kp,
            &registry,
            "s81-i-stale",
            "second-run",
            DEFAULT_MAX_NEW_TOKENS,
        )
        .await;
        let result = registry.result_data("s81-i-stale").unwrap();
        assert!(
            result.result_text.is_none(),
            "the stale first-run result must be cleared, got: {:?}",
            result.result_text
        );
        assert!(
            result.run_proof.is_none(),
            "the stale RunProof must be cleared"
        );
        assert!(
            result.failure.is_some(),
            "the second drive's failure is surfaced"
        );
        rig.head.shutdown().await.ok();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn status_of_reports_lifecycle() {
        // Review Cible 2 P2 (GPT-5.6 Sol): the generate route's 409 precheck
        // reads status_of. A mounted-but-undriven session is Ready; an
        // unmounted id is None.
        let rig = shard_rig(vec![Arc::new(EchoForwarder), Arc::new(EchoForwarder)]).await;
        let registry = ShardSessionRegistry::default();
        mount_session(
            rig.head.endpoint(),
            rig.head.memory_lookup(),
            &rig.head_kp,
            &registry,
            mount_request(&rig, "s81-i-status", &[1_000, 1_000]),
        )
        .await
        .expect("mount");
        assert_eq!(
            registry.status_of("s81-i-status"),
            Some(ShardSessionStatus::Ready),
            "a mounted, undriven session is Ready"
        );
        assert_eq!(registry.status_of("missing"), None, "unmounted -> None");
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
            DEFAULT_MAX_NEW_TOKENS,
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
            DEFAULT_MAX_NEW_TOKENS,
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
            DEFAULT_MAX_NEW_TOKENS,
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
