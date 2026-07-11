// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sharded-inference data plane: ALPN `sbfb/shard/1` (Sprint 77 Phase B).
//!
//! The control plane (manifests, claims, fingerprints, perf-map) rides on
//! the existing iroh-docs / blobs / gossip stack. The **data plane** — the
//! intermediate activations that flow between consecutive layer-blocks of
//! a sharded model — needs a point-to-point, high-frequency, low-latency
//! channel. This module is that channel: a custom ALPN over iroh-QUIC,
//! mirroring the Sprint 74 `sbfb/seed/0` registration exactly
//! ([`crate::node::SHARD_ALPN`], `extra_protocols`).
//!
//! ## Why a custom ALPN, not blobs / docs / RPC
//!
//! - **iroh-blobs** content-addresses every message (BLAKE3) — dead
//!   latency on a live activation stream.
//! - **iroh-docs / gossip** is an eventually-consistent log — wrong shape
//!   for point-to-point, ordered, high-frequency frames.
//! - **llama.cpp RPC** (`-DGGML_RPC=ON`) is blind-trust TCP for LAN/Jetson,
//!   no Byzantine check, no NAT traversal — rejected (kickoff D2).
//!
//! So: one long-lived QUIC [`Connection`] per pair of consecutive shards,
//! a single `open_bi` reused for all tokens (never reconnect-per-token),
//! length-prefixed framing.
//!
//! ## Framing (Phase B adaptation A2 — NEW code, not the seed one-shot)
//!
//! [`crate::seed`] does a single `read_to_end` request/response. The shard
//! data plane is multi-frame over a reused stream: each frame is a 4-byte
//! big-endian length prefix followed by that many payload bytes
//! ([`write_frame`] / [`read_frame`]). A clean stream FIN between frames
//! (`ReadExactError::FinishedEarly`) means "no more frames" → [`read_frame`]
//! returns `Ok(None)`. Every frame is capped at [`MAX_SHARD_FRAME_BYTES`]
//! at both write and read (anti-DoS, mirror of `MAX_SEED_MSG_BYTES`).
//!
//! ## Admission (Phase B — [`ShardProtocol`] + [`crate::compute_group`])
//!
//! A node serving `sbfb/shard/1` admits ONLY the worker public keys on its
//! [`ComputeGroupEntry`] allowlist. The check runs at the very top of
//! [`ProtocolHandler::accept`] on `conn.remote_id()` (the QUIC-authenticated
//! Ed25519 peer, non-spoofable) — BEFORE any `accept_bi` / frame read. A
//! non-member's connection is closed at the handshake. This is **admission
//! control** (who may participate), NOT activation confidentiality: the
//! activations still flow in the clear (no consumer GPU TEE in 2026, scope
//! cut #4), and the allowlist does not guarantee an honest majority (SI-4
//! collusion residual — `SPLIT_INFERENCE_DESIGN.md`).
//!
//! ## RTT for the perf-map (Phase B adaptation A1)
//!
//! [`conn_rtt`] exposes the current path RTT estimate for the Phase D
//! scheduler perf-map. The real installed transport is `noq` (iroh's quinn
//! fork): its `ConnectionStats` carries NO `rtt` field (`Connection::stats()`
//! ignores rtt/cwnd/mtu by construction), so the correct primitive is
//! `Connection::rtt(PathId::ZERO)` — the current (primary) path's
//! round-trip estimate. `PathId::ZERO` is a constant for the default path,
//! not a multipath resolution. Per-path RTT for multi-path scheduling
//! stays a Phase D concern.

use std::time::Duration;

use iroh::address_lookup::memory::MemoryLookup;
use iroh::endpoint::{Connection, PathId, ReadExactError, RecvStream, SendStream};
use iroh::protocol::{AcceptError, DynProtocolHandler, ProtocolHandler};
use iroh::{Endpoint, EndpointAddr};
use tracing::debug;

use crate::compute_group::ComputeGroupEntry;
use crate::error::{NexusError, Result};
use crate::node::{ExtraProtocolFactory, SHARD_ALPN};

/// Maximum bytes accepted in a single shard data-plane frame (anti-DoS).
///
/// One frame carries a boundary hidden-state tensor between two layer-blocks:
/// a `[n_tokens, n_embd]` tensor at **fp32** (4 bytes/elem — the boundary is
/// fp32 on both ends, `llama_batch.embd` / `llama_get_embeddings` are `float*`
/// regardless of the GGUF quant; see `nexus-worker-core` `llm/shard.rs`). The
/// worst case is a full prefill: `n_embd × n_ctx × 4`. For the ~20 GB
/// arch-llama target (n_embd ≈ 8192) bounded by [`MAX_SHARD_N_CTX`] tokens,
/// that is `8192 × 8192 × 4 = 256 MiB` — so the cap is **256 MiB**, raised
/// from the Sprint 77 Phase B 64 MiB (which the original doc mis-stated as
/// fp16, under-counting 2×: 64 MiB held only 2048 tokens at n_embd 8192, too
/// tight for a long prefill). A frame larger than the cap is rejected BEFORE
/// any allocation (see [`header_to_frame_len`]). The *effective* bound is the
/// placement scheduler's choice of n_ctx (Phase D); the cap is the absolute
/// DoS ceiling. Enforced at both [`write_frame`] and [`read_frame`].
pub const MAX_SHARD_FRAME_BYTES: usize = 256 * 1024 * 1024;

/// Upper bound on the context length (`n_ctx`) a shard session may run with,
/// used by the placement scheduler (Phase D) and the worker claim gate.
///
/// Bounding `n_ctx` bounds BOTH the data-plane frame size (`n_embd × n_ctx ×
/// 4`, see [`MAX_SHARD_FRAME_BYTES`]) AND the per-shard KV-cache VRAM (`2 ×
/// n_layers × n_ctx × n_kv_heads × head_dim × dtype`). At `n_embd = 8192` this
/// value keeps a full-prefill frame at exactly 256 MiB; a larger `n_embd`
/// model requires the placement to pick a smaller `n_ctx` to stay under the
/// frame cap. This is a placement/claim policy constant, never serialised on
/// the wire (0-bump).
pub const MAX_SHARD_N_CTX: u32 = 8192;

/// QUIC application close code used when rejecting a peer that is not on
/// the [`ComputeGroupEntry`] allowlist. Distinct from a graceful close
/// (`0`) so the dialer can tell admission-refused from done.
pub const SHARD_REJECT_NOT_MEMBER: u32 = 1;

/// Encode a frame length into its 4-byte big-endian header, rejecting a
/// payload larger than [`MAX_SHARD_FRAME_BYTES`]. Pure (no I/O) so the cap
/// logic is unit-testable without a live stream.
fn frame_len_to_header(len: usize) -> Result<[u8; 4]> {
    if len > MAX_SHARD_FRAME_BYTES {
        return Err(NexusError::Other(format!(
            "shard frame is {len} bytes, exceeds MAX_SHARD_FRAME_BYTES={MAX_SHARD_FRAME_BYTES}"
        )));
    }
    Ok((len as u32).to_be_bytes())
}

/// Decode a 4-byte big-endian header into a frame length, rejecting a
/// declared length larger than [`MAX_SHARD_FRAME_BYTES`] before any
/// allocation. Pure (no I/O).
fn header_to_frame_len(header: [u8; 4]) -> Result<usize> {
    let len = u32::from_be_bytes(header) as usize;
    if len > MAX_SHARD_FRAME_BYTES {
        return Err(NexusError::Other(format!(
            "shard frame declares {len} bytes, exceeds MAX_SHARD_FRAME_BYTES={MAX_SHARD_FRAME_BYTES}"
        )));
    }
    Ok(len)
}

/// Write one length-prefixed frame onto a shard bi-stream.
///
/// 4-byte big-endian length + payload. Does not finish the stream — the
/// same [`SendStream`] is reused for every frame (the `open_bi` long-lived
/// reuse contract). Rejects an over-cap payload before touching the wire.
pub async fn write_frame(send: &mut SendStream, payload: &[u8]) -> Result<()> {
    let header = frame_len_to_header(payload.len())?;
    send.write_all(&header)
        .await
        .map_err(|e| NexusError::Other(format!("shard frame length write failed: {e}")))?;
    send.write_all(payload)
        .await
        .map_err(|e| NexusError::Other(format!("shard frame payload write failed: {e}")))?;
    Ok(())
}

/// Read one length-prefixed frame from a shard bi-stream.
///
/// Returns `Ok(None)` on a clean stream FIN observed at a frame boundary
/// (the peer finished its send stream — no more frames). A mid-frame
/// truncation, a reset, or a connection close surfaces as `Err`. Enforces
/// [`MAX_SHARD_FRAME_BYTES`] before allocating the payload buffer.
pub async fn read_frame(recv: &mut RecvStream) -> Result<Option<Vec<u8>>> {
    let mut header = [0u8; 4];
    match recv.read_exact(&mut header).await {
        Ok(()) => {}
        // Clean FIN exactly at a frame boundary: no more frames.
        Err(ReadExactError::FinishedEarly(0)) => return Ok(None),
        Err(e) => {
            return Err(NexusError::Other(format!(
                "shard frame length read failed: {e}"
            )));
        }
    }
    let len = header_to_frame_len(header)?;
    let mut payload = vec![0u8; len];
    recv.read_exact(&mut payload)
        .await
        .map_err(|e| NexusError::Other(format!("shard frame payload read failed: {e}")))?;
    Ok(Some(payload))
}

/// Current best estimate of a shard connection's round-trip time, for the
/// Phase D scheduler perf-map.
///
/// Reads the primary path's RTT (`PathId::ZERO`). Returns `None` only if
/// the path has no estimate yet (e.g. the connection just opened and the
/// transport has not sampled an RTT). See the module docs for why this is
/// `Connection::rtt(PathId::ZERO)` and not `conn.stats().rtt` (the latter
/// does not exist on the installed `noq` transport).
pub fn conn_rtt(conn: &Connection) -> Option<Duration> {
    conn.rtt(PathId::ZERO)
}

/// Dial a peer over `sbfb/shard/1`, returning the established
/// [`Connection`]. The caller opens a long-lived `open_bi` on it and
/// drives [`write_frame`] / [`read_frame`].
///
/// Seeds the address lookup so the endpoint can dial without pkarr (mirror
/// of `seed_protocol::request_seed`). A successful return means the ALPN
/// was negotiated; admission (allowlist) is enforced server-side at the
/// handshake, so a non-member dial succeeds here but yields no frames.
pub async fn open_shard_connection(
    endpoint: &Endpoint,
    memory_lookup: &MemoryLookup,
    peer_addr: EndpointAddr,
) -> Result<Connection> {
    memory_lookup.add_endpoint_info(peer_addr.clone());
    endpoint
        .connect(peer_addr, SHARD_ALPN)
        .await
        .map_err(|e| NexusError::Endpoint(format!("shard dial failed: {e}")))
}

/// Server-side hook that turns one inbound boundary activation frame into
/// the outbound boundary frame to forward downstream.
///
/// **Dependency-inversion seam (Sprint 77 Phase F2).** `nexus-core-rs` owns
/// the `sbfb/shard/1` data plane but cannot depend on `nexus-worker-core`
/// (where the forked `ShardBackend` lives) without a crate cycle — the
/// dependency edge runs the other way (worker-core → core-rs). So
/// [`ShardProtocol`] holds a `dyn ShardForwarder` and the worker injects the
/// concrete layer-block forwarder ([`crate`] users implement this trait over
/// their backend). This mirrors the dyn-dispatch seam Petals / llama.cpp-RPC /
/// exo use to split transport from compute.
///
/// The frame bytes are the opaque `[n_tokens, n_embd]` row-major fp32 boundary
/// tensor documented at the module level; this trait stays agnostic to the
/// backend that produced them.
///
/// `accept()` is always a downstream / intermediate shard (it RECEIVES an
/// upstream hidden state and forwards through its layer block); the first
/// shard, which embeds raw tokens, is driven by the session orchestrator on
/// the DIALER side, never through `accept()`.
pub trait ShardForwarder: Send + Sync + std::fmt::Debug {
    /// Process one inbound boundary frame, returning the boundary frame to
    /// send downstream. An `Err` aborts THIS connection cleanly (the accept
    /// loop finishes the stream and closes); it must never panic.
    fn forward(&self, upstream_frame: &[u8]) -> Result<Vec<u8>>;

    /// The stage this forwarder ACTUALLY loaded, self-declared for the
    /// stage-attestation exchange (Sprint 81 Phase K — see
    /// [`ShardStageAttestation`]). `None` (the default) means "no real
    /// backend" — a transport-only node. [`ShardProtocol`] does NOT
    /// intercept the attestation for such a node: it echoes every frame
    /// through [`Self::forward`], so an echo left serving inside a REAL
    /// session echoes the attestation request back and the driver
    /// fail-closes (it cannot decode a valid reply). Only a forwarder
    /// that returns `Some(..)` answers an attestation.
    fn loaded_stage(&self) -> Option<LoadedStageDescriptor> {
        None
    }
}

/// A [`ShardForwarder`] that returns every frame unchanged.
///
/// This is the Sprint 77 Phase B data-plane proof preserved as an explicit
/// forwarder: it exercises the length-prefixed framing + admission control
/// without any model. It is the right forwarder for a node that registers the
/// shard ALPN purely to prove transport, and the default the data-plane tests
/// inject. A node serving a real layer block injects a backend forwarder
/// instead (the worker's feature-gated `ShardBackendForwarder`).
#[derive(Debug, Clone, Default)]
pub struct EchoForwarder;

impl ShardForwarder for EchoForwarder {
    fn forward(&self, upstream_frame: &[u8]) -> Result<Vec<u8>> {
        Ok(upstream_frame.to_vec())
    }
}

/// The shard data-plane ALPN handler for `sbfb/shard/1`.
///
/// Holds a verified [`ComputeGroupEntry`] allowlist and an injected
/// [`ShardForwarder`]. On each inbound connection it admits the peer iff
/// `conn.remote_id()` is a member, rejecting a non-member at the handshake
/// before any frame is read.
///
/// Each admitted frame is run through the [`ShardForwarder`] (the layer
/// block) and its output sent downstream. Sprint 77 Phase B proved this loop
/// with an [`EchoForwarder`]; Phase F2 wires the real layer-block forward
/// through the dependency-inversion seam (the worker injects a backend
/// forwarder that loads `layer_start..layer_end`, runs it, and emits the
/// boundary hidden state).
#[derive(Debug, Clone)]
pub struct ShardProtocol {
    admission: std::sync::Arc<ComputeGroupEntry>,
    forwarder: std::sync::Arc<dyn ShardForwarder>,
}

impl ShardProtocol {
    /// Build a handler from an allowlist + forwarder, verifying the
    /// allowlist signature up front so a malformed / forged group can never
    /// be installed.
    pub fn new(
        group: ComputeGroupEntry,
        forwarder: std::sync::Arc<dyn ShardForwarder>,
    ) -> Result<Self> {
        group.verify_signature()?;
        Ok(ShardProtocol {
            admission: std::sync::Arc::new(group),
            forwarder,
        })
    }

    /// Build a handler from an already-verified allowlist (used by the
    /// factory, which verifies once before constructing the closure).
    fn from_verified(
        group: std::sync::Arc<ComputeGroupEntry>,
        forwarder: std::sync::Arc<dyn ShardForwarder>,
    ) -> Self {
        ShardProtocol {
            admission: group,
            forwarder,
        }
    }
}

impl ProtocolHandler for ShardProtocol {
    // NOTE: `Result` is aliased to `Result<T, NexusError>` crate-wide, so
    // the iroh handler contract's `Result<(), AcceptError>` must be spelled
    // with the std path here (the seed handler lives in the daemon crate,
    // which has no such alias).
    async fn accept(&self, conn: Connection) -> std::result::Result<(), AcceptError> {
        // Admission FIRST — mirror seed_protocol.rs:264. `remote_id()` is
        // the QUIC-authenticated Ed25519 peer; a third party cannot dial
        // under someone else's id. A non-member is closed before any frame.
        let peer = *conn.remote_id().as_bytes();
        if !self.admission.is_member(&peer) {
            debug!("shard: rejecting non-member at handshake");
            conn.close(SHARD_REJECT_NOT_MEMBER.into(), b"not-a-member");
            return Ok(());
        }

        // Stage attestation (Sprint 81 Phase K): answered HERE, before the
        // forwarder, so a real layer-block backend never sees the probe as
        // activations (the R-I-1 concern that kept the readiness barrier
        // 0-frame). TWO guards keep this from colliding with legitimate
        // data (Codex GPT-5.6 Sol P1):
        //  - a **transport-only** stage (`loaded_stage() == None`) NEVER
        //    intercepts — it echoes EVERY frame byte-identical, so an echo
        //    session whose prompt happens to be the attestation JSON is not
        //    swallowed. An echo left serving inside a REAL session simply
        //    echoes the request back; the driver then fails to decode it as
        //    an attestation reply and fail-closes (binding preserved).
        //  - only the **first** frame of a real stage is a candidate: the
        //    driver always sends the attestation first on a fresh link, and
        //    a data frame (fp32 tensor / `ShardStepRequest` — the latter
        //    fails `deny_unknown_fields`) can never decode as a request
        //    anyway, so this is belt-and-braces, not correctness-bearing.
        let is_real_stage = self.forwarder.loaded_stage().is_some();
        let (mut send, mut recv) = conn.accept_bi().await?;
        let mut first_frame = true;
        while let Some(frame) = read_frame(&mut recv).await.map_err(AcceptError::from_err)? {
            let was_first = first_frame;
            first_frame = false;
            if is_real_stage && was_first && ShardStageAttestationRequest::decode(&frame).is_ok() {
                let att = ShardStageAttestation::from_loaded_stage(self.forwarder.loaded_stage());
                write_frame(&mut send, &att.encode())
                    .await
                    .map_err(AcceptError::from_err)?;
                continue;
            }
            let out = self
                .forwarder
                .forward(&frame)
                .map_err(AcceptError::from_err)?;
            write_frame(&mut send, &out)
                .await
                .map_err(AcceptError::from_err)?;
        }
        send.finish().map_err(AcceptError::from_err)?;
        conn.closed().await;
        Ok(())
    }
}

/// Build the [`ExtraProtocolFactory`] that `create_node_with_protocols`
/// invokes to wire the shard handler onto the `sbfb/shard/1` ALPN.
///
/// The allowlist signature is verified ONCE here (fail-fast at wiring): a
/// node refuses to serve an unverifiable compute group. The shard handler
/// needs none of the node's store / endpoint / lookup (unlike the seed
/// handler) — admission is decided purely from the QUIC peer id against
/// the in-memory allowlist — so the factory closure ignores all three. The
/// `forwarder` is the injected layer-block compute (an [`EchoForwarder`] for a
/// transport-only node, or the worker's feature-gated backend forwarder).
pub fn shard_protocol_factory(
    group: ComputeGroupEntry,
    forwarder: std::sync::Arc<dyn ShardForwarder>,
) -> Result<ExtraProtocolFactory> {
    group.verify_signature()?;
    let group = std::sync::Arc::new(group);
    Ok(Box::new(move |_store, _ep, _ml| {
        Box::new(ShardProtocol::from_verified(group, forwarder)) as Box<dyn DynProtocolHandler>
    }))
}

// ---------------------------------------------------------------------
// Application-layer step payloads (Sprint 81 Phase J — real inference,
// PO arbitrage Option B)
// ---------------------------------------------------------------------
//
// The `sbfb/shard/1` framing above is OPAQUE bytes. A REAL inference
// session needs two application-level payload shapes INSIDE those frames,
// in addition to the raw `[n_tokens, n_embd]` fp32-LE boundary tensor:
//
//   - driver -> FIRST shard: a step request (prompt + tokens generated so
//     far). The first shard owns the tokenizer/vocab and re-derives its
//     input ids each step, so the driver never needs the model on disk.
//   - LAST shard -> driver: a step reply (the greedy-sampled token id,
//     its detokenized piece, the EOS flag, and the N0 TOPLOC commitment
//     hex of the post-norm hidden state at the sampled position).
//
// Both are JSON inside an opaque frame: they are NOT wire structs, carry
// no `*_FORMAT_VERSION` governance and are never signed — the signed
// artefacts stay the manifest and the RunProof (`shard_plan.rs`). The `v`
// field is an application-level guard so a role mismatch (a JSON payload
// reaching a mid shard, an fp32 tensor reaching a JSON decoder) fails
// LOUD instead of feeding a backend garbage. Middle shards keep receiving
// raw fp32 frames untouched (0 wire bump: the ALPN, the framing and the
// admission are byte-identical to Sprint 77 Phase B).

/// Application-level version guard for [`ShardStepRequest`] /
/// [`ShardStepReply`] payloads. NOT a wire `*_FORMAT_VERSION` (the frame
/// stays opaque); a decoder rejects a mismatch loud.
pub const SHARD_STEP_PAYLOAD_V: u16 = 1;

/// Driver -> first-shard step payload: the prompt plus every token id the
/// pipeline generated so far. The first shard tokenizes the prompt with
/// its own vocab, appends `generated`, and forwards the whole sequence
/// (stateless per-step recompute — no cross-step KV reuse, which is what
/// makes the SI-9 fallback replay of a step input CORRECT by
/// construction: any stage's step input can be replayed on a fallback
/// worker with no lost state).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShardStepRequest {
    /// Must equal [`SHARD_STEP_PAYLOAD_V`].
    pub v: u16,
    /// The user prompt (tokenized by the first shard each step).
    pub prompt: String,
    /// Token ids generated so far, in order (empty on the first step).
    #[serde(default)]
    pub generated: Vec<i32>,
}

impl ShardStepRequest {
    /// Build a step request at the current payload version.
    #[must_use]
    pub fn new(prompt: impl Into<String>, generated: Vec<i32>) -> Self {
        ShardStepRequest {
            v: SHARD_STEP_PAYLOAD_V,
            prompt: prompt.into(),
            generated,
        }
    }

    /// Encode to the frame payload bytes (JSON).
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("step request serialization is infallible")
    }

    /// Decode a frame payload, rejecting non-JSON bytes (e.g. an fp32
    /// tensor mis-routed to a first shard) and a version mismatch.
    pub fn decode(frame: &[u8]) -> Result<Self> {
        let req: ShardStepRequest = serde_json::from_slice(frame)
            .map_err(|e| NexusError::Other(format!("shard step request decode: {e}")))?;
        if req.v != SHARD_STEP_PAYLOAD_V {
            return Err(NexusError::Other(format!(
                "shard step request payload v{} (expected v{SHARD_STEP_PAYLOAD_V})",
                req.v
            )));
        }
        Ok(req)
    }
}

/// Last-shard -> driver step payload: one greedy-sampled decode step.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShardStepReply {
    /// Must equal [`SHARD_STEP_PAYLOAD_V`].
    pub v: u16,
    /// The sampled token id (greedy argmax — deterministic, ties broken
    /// by the lowest vocab index).
    pub token_id: i32,
    /// The detokenized piece for `token_id` (empty for an EOS token).
    pub piece: String,
    /// Whether `token_id` is an end-of-generation token.
    pub is_eos: bool,
    /// N0 TOPLOC commitment (lowercase blake3 hex, 64 chars) of the last
    /// shard's post-norm hidden state at the sampled position; empty when
    /// the backend cannot provide one.
    #[serde(default)]
    pub toploc_hex: String,
}

impl ShardStepReply {
    /// Encode to the frame payload bytes (JSON).
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("step reply serialization is infallible")
    }

    /// Decode a frame payload, rejecting non-JSON bytes and a version
    /// mismatch (e.g. an fp32 tensor returned by a mis-roled tail).
    pub fn decode(frame: &[u8]) -> Result<Self> {
        let reply: ShardStepReply = serde_json::from_slice(frame)
            .map_err(|e| NexusError::Other(format!("shard step reply decode: {e}")))?;
        if reply.v != SHARD_STEP_PAYLOAD_V {
            return Err(NexusError::Other(format!(
                "shard step reply payload v{} (expected v{SHARD_STEP_PAYLOAD_V})",
                reply.v
            )));
        }
        Ok(reply)
    }
}

// ---------------------------------------------------------------------
// Stage attestation (Sprint 81 Phase K — binding loaded-stage ↔ signed
// manifest, THREAT_MODEL §16)
// ---------------------------------------------------------------------
//
// Through Sprint 81 Phase J the serve path chose its layer window and role
// from CLI flags and never saw the signed manifest: a mis-windowed /
// mis-modeled stage — or a transport-only echo left serving inside a REAL
// session — produced a plausible-but-wrong result that the driver then
// SIGNED into a RunProof. This exchange closes that MISCONFIGURATION
// class: before any data frame flows on a stage link, the driver requests
// the stage's self-declared [`LoadedStageDescriptor`] and fail-closes on
// any mismatch with the signed `ShardedSessionManifest` +
// [`crate::shard_plan::ShardAssignment`].
//
// Same wire posture as the step payloads above: JSON inside an opaque
// `sbfb/shard/1` frame, no `*_FORMAT_VERSION` governance, never signed —
// 0 wire bump. The attestation is a SELF-CLAIM by an admitted group
// member (family N0): it closes operator misconfiguration, NOT a
// deliberately byzantine stage (SI-4 residual). [`ShardProtocol`]
// intercepts the request BEFORE the forwarder, so a real layer-block
// backend is never fed the probe as activations.

/// Application-level version guard for [`ShardStageAttestationRequest`] /
/// [`ShardStageAttestation`] payloads. NOT a wire `*_FORMAT_VERSION` (the
/// frame stays opaque); a decoder rejects a mismatch loud.
pub const SHARD_ATTEST_PAYLOAD_V: u16 = 1;

/// Required `kind` of a [`ShardStageAttestationRequest`] — the explicit
/// discriminant that keeps this minimal payload from ever colliding with
/// another JSON payload carried in the same opaque frames.
pub const SHARD_ATTEST_REQUEST_KIND: &str = "attest-stage-request";

/// Required `kind` of a [`ShardStageAttestation`] reply.
pub const SHARD_ATTEST_REPLY_KIND: &str = "stage-attestation";

/// What a serving stage ACTUALLY loaded, self-declared through
/// [`ShardForwarder::loaded_stage`] for the attestation exchange. A
/// transport-only node has no descriptor (`None`) — [`ShardProtocol`]
/// then does NOT intercept the attestation at all (the request frame is
/// echoed like any other and the driver fails closed on the undecodable
/// reply); an all-zeros digest reply only exists for direct callers of
/// [`ShardStageAttestation::from_loaded_stage`] with `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadedStageDescriptor {
    /// BLAKE3 digest of the model file the backend loaded
    /// (streaming-hashed — [`crate::crypto::blake3_hash_file`]).
    pub model_digest: [u8; 32],
    /// Inclusive start layer of the loaded window.
    pub layer_start: u32,
    /// Exclusive end layer of the loaded window (resolved, never a `0`
    /// "to the end" sentinel).
    pub layer_end: u32,
    /// Whether the loaded window owns layer 0 (embeds input tokens).
    pub is_first: bool,
    /// Whether the loaded window owns the final layer (norm + lm_head).
    pub is_last: bool,
}

/// Driver -> stage attestation request. Carries no data beyond its
/// discriminant: the stage answers with what IT loaded, never with
/// anything echoed from the request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShardStageAttestationRequest {
    /// Must equal [`SHARD_ATTEST_PAYLOAD_V`].
    pub v: u16,
    /// Must equal [`SHARD_ATTEST_REQUEST_KIND`].
    pub kind: String,
}

impl ShardStageAttestationRequest {
    /// Build a request at the current payload version.
    #[must_use]
    pub fn new() -> Self {
        ShardStageAttestationRequest {
            v: SHARD_ATTEST_PAYLOAD_V,
            kind: SHARD_ATTEST_REQUEST_KIND.to_string(),
        }
    }

    /// Encode to the frame payload bytes (JSON).
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("attestation request serialization is infallible")
    }

    /// Decode a frame payload, rejecting non-JSON bytes, a version
    /// mismatch and a wrong `kind`.
    pub fn decode(frame: &[u8]) -> Result<Self> {
        let req: ShardStageAttestationRequest = serde_json::from_slice(frame)
            .map_err(|e| NexusError::Other(format!("stage attestation request decode: {e}")))?;
        if req.v != SHARD_ATTEST_PAYLOAD_V {
            return Err(NexusError::Other(format!(
                "stage attestation request payload v{} (expected v{SHARD_ATTEST_PAYLOAD_V})",
                req.v
            )));
        }
        if req.kind != SHARD_ATTEST_REQUEST_KIND {
            return Err(NexusError::Other(format!(
                "stage attestation request kind '{}' (expected '{SHARD_ATTEST_REQUEST_KIND}')",
                req.kind
            )));
        }
        Ok(req)
    }
}

impl Default for ShardStageAttestationRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Stage -> driver attestation reply: the self-declared loaded stage.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShardStageAttestation {
    /// Must equal [`SHARD_ATTEST_PAYLOAD_V`].
    pub v: u16,
    /// Must equal [`SHARD_ATTEST_REPLY_KIND`].
    pub kind: String,
    /// Lowercase blake3 hex (64 chars) of the loaded model file;
    /// ALL-ZEROS = "no real backend loaded" (transport-only echo).
    pub model_digest_hex: String,
    /// Inclusive start layer of the loaded window (0 when unloaded).
    pub layer_start: u32,
    /// Exclusive end layer of the loaded window (0 when unloaded).
    pub layer_end: u32,
    /// Whether the loaded window embeds input tokens.
    pub is_first: bool,
    /// Whether the loaded window samples the output token.
    pub is_last: bool,
}

impl ShardStageAttestation {
    /// Build the reply for a forwarder's self-declared stage. `None`
    /// (transport-only) builds an all-zeros reply — honest "nothing
    /// loaded", which a driver fail-closes on. NOTE: since the Phase K
    /// interception gate, [`ShardProtocol`] never reaches this `None` arm
    /// in a session (a `None` forwarder is not intercepted — the request
    /// is echoed instead); the arm stays for direct callers and tests.
    #[must_use]
    pub fn from_loaded_stage(desc: Option<LoadedStageDescriptor>) -> Self {
        match desc {
            Some(d) => ShardStageAttestation {
                v: SHARD_ATTEST_PAYLOAD_V,
                kind: SHARD_ATTEST_REPLY_KIND.to_string(),
                model_digest_hex: hex::encode(d.model_digest),
                layer_start: d.layer_start,
                layer_end: d.layer_end,
                is_first: d.is_first,
                is_last: d.is_last,
            },
            None => ShardStageAttestation {
                v: SHARD_ATTEST_PAYLOAD_V,
                kind: SHARD_ATTEST_REPLY_KIND.to_string(),
                model_digest_hex: hex::encode([0u8; 32]),
                layer_start: 0,
                layer_end: 0,
                is_first: false,
                is_last: false,
            },
        }
    }

    /// Encode to the frame payload bytes (JSON).
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("stage attestation serialization is infallible")
    }

    /// Decode a frame payload, rejecting non-JSON bytes, a version
    /// mismatch, a wrong `kind` and a malformed digest. The digest MUST
    /// be exactly 64 lowercase ASCII hex chars — the peer controls this
    /// string, so the strict charset check here is what lets every
    /// downstream consumer (diagnostics included) slice it safely.
    pub fn decode(frame: &[u8]) -> Result<Self> {
        let att: ShardStageAttestation = serde_json::from_slice(frame)
            .map_err(|e| NexusError::Other(format!("stage attestation decode: {e}")))?;
        if att.v != SHARD_ATTEST_PAYLOAD_V {
            return Err(NexusError::Other(format!(
                "stage attestation payload v{} (expected v{SHARD_ATTEST_PAYLOAD_V})",
                att.v
            )));
        }
        if att.kind != SHARD_ATTEST_REPLY_KIND {
            return Err(NexusError::Other(format!(
                "stage attestation kind '{}' (expected '{SHARD_ATTEST_REPLY_KIND}')",
                att.kind
            )));
        }
        if att.model_digest_hex.len() != 64
            || !att
                .model_digest_hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(NexusError::Other(
                "stage attestation model_digest_hex is not 64 lowercase hex chars".into(),
            ));
        }
        Ok(att)
    }
}

/// Driver-side attestation exchange on an OPEN stage bi-stream: write the
/// request frame, read exactly one reply frame, decode it. No intrinsic
/// timeout (mirror of [`write_frame`] / [`read_frame`]) — the caller owns
/// the deadline, exactly like every other hop on the link.
pub async fn request_stage_attestation(
    send: &mut SendStream,
    recv: &mut RecvStream,
) -> Result<ShardStageAttestation> {
    write_frame(send, &ShardStageAttestationRequest::new().encode()).await?;
    let frame = read_frame(recv).await?.ok_or_else(|| {
        NexusError::Other("stage finished the stream before answering the attestation".into())
    })?;
    ShardStageAttestation::decode(&frame)
}

/// Truncated (8-byte / 16-hex) digest prefix for diagnostics — long enough
/// to disambiguate, short enough to survive the daemon's hex-run redaction
/// (which scrubs full 32-byte identities). `get` (never a raw slice): the
/// input can be peer-controlled, and a multi-byte char across the boundary
/// must degrade to the full string, never panic (review S81-K-R-1 — under
/// `panic = "abort"` a slice panic here would be a remote daemon kill).
fn digest_prefix_hex(digest_hex: &str) -> &str {
    digest_hex.get(..16).unwrap_or(digest_hex)
}

/// Fail-closed comparison of a stage's self-declared attestation against
/// the SIGNED plan expectations: the manifest's `model_digest` and the
/// stage's [`crate::shard_plan::ShardAssignment`] window, plus the
/// pipeline-position roles (stage 0 embeds, the final stage samples).
/// Pure (no I/O) so every mismatch branch is hermetically testable.
pub fn verify_stage_attestation(
    att: &ShardStageAttestation,
    manifest_model_digest: &[u8; 32],
    assignment: &crate::shard_plan::ShardAssignment,
    stage_index: usize,
    stage_count: usize,
) -> Result<()> {
    let expected_hex = hex::encode(manifest_model_digest);
    if att.model_digest_hex != expected_hex {
        return Err(NexusError::Other(format!(
            "stage attests model digest {}... but the signed manifest pins {}... \
             (an all-zeros attestation means no real backend is loaded)",
            digest_prefix_hex(&att.model_digest_hex),
            digest_prefix_hex(&expected_hex),
        )));
    }
    if att.layer_start != assignment.layer_start || att.layer_end != assignment.layer_end {
        return Err(NexusError::Other(format!(
            "stage attests layer window [{},{}) but the signed plan assigns [{},{})",
            att.layer_start, att.layer_end, assignment.layer_start, assignment.layer_end,
        )));
    }
    let expect_first = stage_index == 0;
    let expect_last = stage_index + 1 == stage_count;
    if att.is_first != expect_first || att.is_last != expect_last {
        return Err(NexusError::Other(format!(
            "stage attests roles (is_first={}, is_last={}) but pipeline position {}/{} \
             requires (is_first={expect_first}, is_last={expect_last})",
            att.is_first,
            att.is_last,
            stage_index + 1,
            stage_count,
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute_group::ComputeGroup;
    use crate::crypto::KeyPair;
    use crate::discovery::DiscoveryClient;
    use crate::node::{
        Node, NodeConfig, create_node, create_node_with_config, create_node_with_protocols,
    };

    // ---- Pure framing-codec tests (hermetic, no network) ----

    #[test]
    fn frame_header_roundtrips() {
        for len in [
            0usize,
            1,
            255,
            256,
            65_535,
            1_048_576,
            MAX_SHARD_FRAME_BYTES,
        ] {
            let header = frame_len_to_header(len).expect("within cap");
            assert_eq!(header_to_frame_len(header).unwrap(), len);
        }
    }

    #[test]
    fn frame_header_rejects_oversize() {
        let over = MAX_SHARD_FRAME_BYTES + 1;
        assert!(
            frame_len_to_header(over).is_err(),
            "encode must reject over-cap"
        );
        // A crafted header declaring an over-cap length is rejected on read
        // before any allocation.
        let crafted = (over as u32).to_be_bytes();
        assert!(
            header_to_frame_len(crafted).is_err(),
            "decode must reject over-cap"
        );
    }

    // ---- Application-layer step payload codecs (pure, Phase J) ----

    #[test]
    fn step_request_roundtrips_and_rejects_garbage() {
        let req = ShardStepRequest::new("The quick brown fox", vec![3, 17, 42]);
        let bytes = req.encode();
        assert_eq!(
            ShardStepRequest::decode(&bytes).expect("roundtrip"),
            req,
            "encode/decode must roundtrip"
        );

        // An fp32 tensor mis-routed to a first shard is NOT JSON — loud reject.
        let fp32: Vec<u8> = 1.5_f32.to_le_bytes().to_vec();
        assert!(
            ShardStepRequest::decode(&fp32).is_err(),
            "raw fp32 bytes must not decode as a step request"
        );

        // A version mismatch is rejected even when the JSON parses.
        let wrong_v = br#"{"v":9,"prompt":"x","generated":[]}"#;
        assert!(
            ShardStepRequest::decode(wrong_v).is_err(),
            "payload version mismatch must be rejected"
        );

        // Unknown fields are rejected (deny_unknown_fields): a reply
        // mis-routed as a request fails loud instead of half-parsing.
        let reply_bytes = ShardStepReply {
            v: SHARD_STEP_PAYLOAD_V,
            token_id: 7,
            piece: "ok".into(),
            is_eos: false,
            toploc_hex: String::new(),
        }
        .encode();
        assert!(
            ShardStepRequest::decode(&reply_bytes).is_err(),
            "a step REPLY must not decode as a step REQUEST"
        );
    }

    #[test]
    fn step_reply_roundtrips_and_rejects_garbage() {
        let reply = ShardStepReply {
            v: SHARD_STEP_PAYLOAD_V,
            token_id: 1234,
            piece: " fox".into(),
            is_eos: false,
            toploc_hex: "ab".repeat(32),
        };
        let bytes = reply.encode();
        assert_eq!(
            ShardStepReply::decode(&bytes).expect("roundtrip"),
            reply,
            "encode/decode must roundtrip"
        );
        // Raw fp32 boundary bytes must not decode as a reply (the driver
        // would otherwise mis-read a mid shard's tensor as a token).
        let fp32: Vec<u8> = [0.25_f32, -1.0]
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();
        assert!(
            ShardStepReply::decode(&fp32).is_err(),
            "raw fp32 bytes must not decode as a step reply"
        );
        let wrong_v = br#"{"v":2,"token_id":1,"piece":"","is_eos":true}"#;
        assert!(
            ShardStepReply::decode(wrong_v).is_err(),
            "payload version mismatch must be rejected"
        );
        // Cross-reject symmetry (review J J-D2-1): a step REQUEST payload
        // must not half-parse as a step REPLY (deny_unknown_fields +
        // disjoint required fields make this structural — asserted here).
        let request_bytes = ShardStepRequest::new("prompt", vec![1, 2]).encode();
        assert!(
            ShardStepReply::decode(&request_bytes).is_err(),
            "a step REQUEST must not decode as a step REPLY"
        );
    }

    // ---- Stage attestation codecs + verification (pure, Phase K) ----

    fn attested_assignment(start: u32, end: u32) -> crate::shard_plan::ShardAssignment {
        crate::shard_plan::ShardAssignment {
            worker_pubkey: [1u8; 32],
            layer_start: start,
            layer_end: end,
            role: crate::shard_plan::ShardRole::LayerWorker,
            shard_hashes: vec![],
            kv_cache_policy: crate::shard_plan::KvCachePolicy::LocalEphemeral,
            fallback_node: None,
            launch_profile_hash: [0u8; 32],
        }
    }

    #[test]
    fn attestation_codecs_roundtrip_and_reject_garbage() {
        // Request roundtrip + loud rejects on version / kind / non-JSON.
        let req = ShardStageAttestationRequest::new();
        assert_eq!(
            ShardStageAttestationRequest::decode(&req.encode()).expect("roundtrip"),
            req
        );
        let fp32: Vec<u8> = 1.5_f32.to_le_bytes().to_vec();
        assert!(
            ShardStageAttestationRequest::decode(&fp32).is_err(),
            "raw fp32 bytes must not decode as an attestation request"
        );
        assert!(
            ShardStageAttestationRequest::decode(br#"{"v":9,"kind":"attest-stage-request"}"#)
                .is_err(),
            "payload version mismatch must be rejected"
        );
        assert!(
            ShardStageAttestationRequest::decode(br#"{"v":1,"kind":"something-else"}"#).is_err(),
            "a wrong kind must be rejected"
        );
        // Cross-collision guards: a step request never decodes as an
        // attestation request (deny_unknown_fields) and vice versa
        // (missing required fields) — the accept-loop interception can
        // never swallow legitimate traffic.
        let step = ShardStepRequest::new("prompt", vec![1]).encode();
        assert!(
            ShardStageAttestationRequest::decode(&step).is_err(),
            "a step REQUEST must not decode as an attestation request"
        );
        assert!(
            ShardStepRequest::decode(&req.encode()).is_err(),
            "an attestation request must not decode as a step REQUEST"
        );

        // Reply roundtrip + rejects, from a real descriptor and from the
        // transport-only None (all-zeros wire form).
        let desc = LoadedStageDescriptor {
            model_digest: [9u8; 32],
            layer_start: 4,
            layer_end: 8,
            is_first: false,
            is_last: true,
        };
        let att = ShardStageAttestation::from_loaded_stage(Some(desc));
        assert_eq!(
            ShardStageAttestation::decode(&att.encode()).expect("roundtrip"),
            att
        );
        assert_eq!(att.model_digest_hex, "09".repeat(32));
        let none = ShardStageAttestation::from_loaded_stage(None);
        assert_eq!(
            none.model_digest_hex,
            "00".repeat(32),
            "a transport-only stage attests all-zeros, never a fake digest"
        );
        assert!(
            ShardStageAttestation::decode(&fp32).is_err(),
            "raw fp32 bytes must not decode as an attestation"
        );
        let wrong_kind = br#"{"v":1,"kind":"nope","model_digest_hex":"00","layer_start":0,"layer_end":0,"is_first":false,"is_last":false}"#;
        assert!(ShardStageAttestation::decode(wrong_kind).is_err());
    }

    #[test]
    fn attestation_digest_is_strictly_validated_and_never_panics() {
        // Review S81-K-R-1: `model_digest_hex` is PEER-CONTROLLED. The
        // decoder must reject anything that is not 64 lowercase ASCII hex
        // (length, case, charset), and the diagnostic path must never
        // slice across a multi-byte char boundary (panic = remote daemon
        // kill under `panic = "abort"`).
        let mk = |digest: &str| {
            format!(
                r#"{{"v":1,"kind":"stage-attestation","model_digest_hex":"{digest}","layer_start":4,"layer_end":8,"is_first":false,"is_last":true}}"#
            )
        };
        assert!(
            ShardStageAttestation::decode(mk(&"ab".repeat(32)).as_bytes()).is_ok(),
            "a well-formed 64-lowercase-hex digest must decode"
        );
        for (bad, why) in [
            ("abcd".to_string(), "wrong length"),
            ("AB".repeat(32), "uppercase rejected"),
            ("zz".repeat(32), "non-hex rejected"),
            // 16 x U+00E9 'é' (2 bytes each): 32 bytes, 16 chars — the
            // exact shape that panicked a naive `[..16]` byte slice.
            ("é".repeat(16), "multi-byte rejected"),
        ] {
            assert!(
                ShardStageAttestation::decode(mk(&bad).as_bytes()).is_err(),
                "malformed digest must be rejected loud: {why}"
            );
        }

        // Defense in depth: even a HAND-BUILT attestation (pub fields,
        // no decode) with a multi-byte digest must fail-close cleanly —
        // never panic in the diagnostic formatting.
        let forged = ShardStageAttestation {
            v: SHARD_ATTEST_PAYLOAD_V,
            kind: SHARD_ATTEST_REPLY_KIND.to_string(),
            model_digest_hex: "é".repeat(40),
            layer_start: 4,
            layer_end: 8,
            is_first: false,
            is_last: true,
        };
        let err = verify_stage_attestation(&forged, &[9u8; 32], &attested_assignment(4, 8), 1, 2)
            .expect_err("a forged digest must fail closed, not panic");
        assert!(err.to_string().contains("model digest"));
    }

    #[test]
    fn verify_stage_attestation_fail_closes_on_every_mismatch() {
        let manifest_digest = [9u8; 32];
        let tail = attested_assignment(4, 8);
        let good = ShardStageAttestation::from_loaded_stage(Some(LoadedStageDescriptor {
            model_digest: manifest_digest,
            layer_start: 4,
            layer_end: 8,
            is_first: false,
            is_last: true,
        }));
        verify_stage_attestation(&good, &manifest_digest, &tail, 1, 2)
            .expect("a conforming tail attestation must pass");

        // Digest mismatch — an all-zeros reply only arises from a direct
        // `from_loaded_stage(None)` call (since the Phase K interception
        // gate, an echo left serving in a REAL session echoes the request
        // back instead — that path is covered by the fail-closed decode
        // tests); the binding must still reject all-zeros vs a pinned
        // manifest digest.
        let echo = ShardStageAttestation::from_loaded_stage(None);
        let err = verify_stage_attestation(&echo, &manifest_digest, &tail, 1, 2).unwrap_err();
        assert!(
            err.to_string().contains("model digest"),
            "all-zeros vs pinned digest must fail closed, got: {err}"
        );

        // Window mismatch: loaded [0,4) but the plan assigns [4,8).
        let wrong_window = ShardStageAttestation::from_loaded_stage(Some(LoadedStageDescriptor {
            model_digest: manifest_digest,
            layer_start: 0,
            layer_end: 4,
            is_first: true,
            is_last: false,
        }));
        let err =
            verify_stage_attestation(&wrong_window, &manifest_digest, &tail, 1, 2).unwrap_err();
        assert!(
            err.to_string().contains("layer window"),
            "a mis-windowed stage must fail closed, got: {err}"
        );

        // Role mismatch: window matches the assignment but the loaded
        // backend does not own the final layer (mis-sized model — the
        // "plausible-but-wrong tail" the RunProof would otherwise sign).
        let not_last = ShardStageAttestation::from_loaded_stage(Some(LoadedStageDescriptor {
            model_digest: manifest_digest,
            layer_start: 4,
            layer_end: 8,
            is_first: false,
            is_last: false,
        }));
        let err = verify_stage_attestation(&not_last, &manifest_digest, &tail, 1, 2).unwrap_err();
        assert!(
            err.to_string().contains("is_last=true"),
            "a tail that does not own the final layer must fail closed, got: {err}"
        );
    }

    // ---- Two-node data-plane tests (in-process, mirror seed fixture) ----

    /// A forwarder that doubles every frame (`f -> f ++ f`) — proves the
    /// accept loop runs the INJECTED forwarder, not a hard-coded echo.
    #[derive(Debug)]
    struct DoublingForwarder;
    impl ShardForwarder for DoublingForwarder {
        fn forward(&self, upstream_frame: &[u8]) -> Result<Vec<u8>> {
            let mut out = upstream_frame.to_vec();
            out.extend_from_slice(upstream_frame);
            Ok(out)
        }
    }

    /// A doubling forwarder that ALSO self-declares a loaded stage — the
    /// serve shape of a real backend for the attestation tests.
    #[derive(Debug)]
    struct AttestedDoublingForwarder(LoadedStageDescriptor);
    impl ShardForwarder for AttestedDoublingForwarder {
        fn forward(&self, upstream_frame: &[u8]) -> Result<Vec<u8>> {
            let mut out = upstream_frame.to_vec();
            out.extend_from_slice(upstream_frame);
            Ok(out)
        }
        fn loaded_stage(&self) -> Option<LoadedStageDescriptor> {
            Some(self.0)
        }
    }

    /// A forwarder that always errors — proves a forwarder failure aborts the
    /// connection cleanly (no panic, no frame returned).
    #[derive(Debug)]
    struct FailingForwarder;
    impl ShardForwarder for FailingForwarder {
        fn forward(&self, _upstream_frame: &[u8]) -> Result<Vec<u8>> {
            Err(NexusError::Other("forwarder refused this frame".into()))
        }
    }

    /// Spin up client node A (known identity) and server node B running a
    /// [`ShardProtocol`] whose allowlist contains A iff `admit_a`, with the
    /// given `forwarder` wired as the layer-block compute.
    async fn two_node_shard_fixture_with(
        admit_a: bool,
        forwarder: std::sync::Arc<dyn ShardForwarder>,
    ) -> (Node, Node, EndpointAddr) {
        let a_secret = KeyPair::generate().secret_bytes();
        let a_kp = KeyPair::from_secret_bytes(&a_secret);
        let node_a = create_node_with_config(NodeConfig::default().with_secret_key(a_secret))
            .await
            .expect("node A");

        let b_secret = KeyPair::generate().secret_bytes();
        let b_kp = KeyPair::from_secret_bytes(&b_secret);
        let mut group = ComputeGroup::new(b_kp.public_bytes(), "pilot-shard", 1);
        if admit_a {
            group = group.with_member(a_kp.public_bytes());
        }
        let entry = ComputeGroupEntry::sign(group, &b_kp).unwrap();
        let factory = shard_protocol_factory(entry, forwarder).expect("verified allowlist");
        let node_b = create_node_with_protocols(
            NodeConfig::default().with_secret_key(b_secret),
            vec![(SHARD_ALPN.to_vec(), factory)],
        )
        .await
        .expect("node B");
        let b_addr = DiscoveryClient::new(node_b.endpoint())
            .my_endpoint_addr()
            .await
            .expect("B addr");
        (node_a, node_b, b_addr)
    }

    /// The Phase B data-plane fixture: admission + echo forwarder (proves the
    /// framing / admission without a backend).
    async fn two_node_shard_fixture(admit_a: bool) -> (Node, Node, EndpointAddr) {
        two_node_shard_fixture_with(admit_a, std::sync::Arc::new(EchoForwarder)).await
    }

    #[tokio::test]
    async fn shard_alpn_registered_in_router() {
        // A node that registered the shard protocol negotiates SHARD_ALPN;
        // a vanilla node (no factory) refuses it — proving the registration
        // is what wires the ALPN, not a default.
        let (node_a, node_b, b_addr) = two_node_shard_fixture(true).await;
        node_a.memory_lookup().add_endpoint_info(b_addr.clone());
        assert!(
            node_a.endpoint().connect(b_addr, SHARD_ALPN).await.is_ok(),
            "a node with the shard protocol must accept SHARD_ALPN"
        );

        let vanilla = create_node().await.expect("vanilla node");
        let v_addr = DiscoveryClient::new(vanilla.endpoint())
            .my_endpoint_addr()
            .await
            .expect("vanilla addr");
        node_a.memory_lookup().add_endpoint_info(v_addr.clone());
        assert!(
            node_a.endpoint().connect(v_addr, SHARD_ALPN).await.is_err(),
            "a node WITHOUT the shard protocol must refuse SHARD_ALPN"
        );

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
        vanilla.shutdown().await.ok();
    }

    #[tokio::test]
    async fn shard_frame_roundtrip_two_nodes() {
        // Multiple frames over ONE long-lived open_bi (the D2 reuse
        // contract): each is echoed back identically.
        let (node_a, node_b, b_addr) = two_node_shard_fixture(true).await;
        let conn = open_shard_connection(node_a.endpoint(), node_a.memory_lookup(), b_addr)
            .await
            .expect("dial");
        let (mut send, mut recv) = conn.open_bi().await.expect("open_bi");
        for payload in [
            b"activation-frame-one".as_slice(),
            b"activation-frame-two",
            b"activation-frame-three",
        ] {
            write_frame(&mut send, payload).await.expect("write");
            let echo = read_frame(&mut recv)
                .await
                .expect("read ok")
                .expect("a frame, not EOF");
            assert_eq!(echo, payload, "the data plane must round-trip the frame");
        }
        send.finish().ok();
        conn.close(0u32.into(), b"done");
        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    #[tokio::test]
    async fn shard_handshake_admits_member() {
        // A member's connection is admitted: it can exchange a frame.
        let (node_a, node_b, b_addr) = two_node_shard_fixture(true).await;
        let conn = open_shard_connection(node_a.endpoint(), node_a.memory_lookup(), b_addr)
            .await
            .expect("dial");
        let (mut send, mut recv) = conn.open_bi().await.expect("open_bi");
        write_frame(&mut send, b"hello-from-member")
            .await
            .expect("write");
        let echo = read_frame(&mut recv)
            .await
            .expect("read ok")
            .expect("member receives the echoed frame");
        assert_eq!(echo, b"hello-from-member");
        send.finish().ok();
        conn.close(0u32.into(), b"done");
        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    #[tokio::test]
    async fn shard_handshake_rejects_non_member() {
        // A peer absent from the allowlist is rejected at the handshake:
        // the ALPN dial succeeds (rejection is post-handshake), but the
        // data plane never yields a frame — the server closed the
        // connection before accept_bi.
        let (node_a, node_b, b_addr) = two_node_shard_fixture(false).await;
        let conn = open_shard_connection(node_a.endpoint(), node_a.memory_lookup(), b_addr)
            .await
            .expect("ALPN dial succeeds; admission is checked after");
        let rejected = match conn.open_bi().await {
            // The connection was already closed by the admission check.
            Err(_) => true,
            Ok((mut send, mut recv)) => {
                let _ = write_frame(&mut send, b"should-be-rejected").await;
                // A non-member must NOT receive a framed response.
                read_frame(&mut recv).await.is_err()
            }
        };
        assert!(
            rejected,
            "a non-member must be rejected at the handshake (no frame echoed)"
        );
        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    #[tokio::test]
    async fn shard_conn_stats_exposes_rtt() {
        // After a member establishes + uses a connection, the perf-map RTT
        // primitive returns a sane estimate (feeds the Phase D scheduler).
        let (node_a, node_b, b_addr) = two_node_shard_fixture(true).await;
        let conn = open_shard_connection(node_a.endpoint(), node_a.memory_lookup(), b_addr)
            .await
            .expect("dial");
        let (mut send, mut recv) = conn.open_bi().await.expect("open_bi");
        write_frame(&mut send, b"ping").await.expect("write");
        let _ = read_frame(&mut recv).await.expect("read ok").expect("echo");

        let rtt = conn_rtt(&conn);
        assert!(
            rtt.is_some(),
            "an established shard connection must expose a path RTT estimate"
        );
        assert!(
            rtt.unwrap() < Duration::from_secs(60),
            "the RTT estimate must be a sane value, not a garbage duration"
        );
        send.finish().ok();
        conn.close(0u32.into(), b"done");
        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    #[tokio::test]
    async fn shard_forward_invokes_forwarder() {
        // The accept loop must run the INJECTED forwarder, not a hard-coded
        // echo: a doubling forwarder turns "ab" into "abab" on the wire.
        let (node_a, node_b, b_addr) =
            two_node_shard_fixture_with(true, std::sync::Arc::new(DoublingForwarder)).await;
        let conn = open_shard_connection(node_a.endpoint(), node_a.memory_lookup(), b_addr)
            .await
            .expect("dial");
        let (mut send, mut recv) = conn.open_bi().await.expect("open_bi");
        write_frame(&mut send, b"ab").await.expect("write");
        let out = read_frame(&mut recv)
            .await
            .expect("read ok")
            .expect("a forwarded frame, not EOF");
        assert_eq!(
            out, b"abab",
            "the data plane must run the injected forwarder, not echo"
        );
        send.finish().ok();
        conn.close(0u32.into(), b"done");
        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    #[tokio::test]
    async fn shard_attestation_answered_before_the_forwarder() {
        // The attestation request must be intercepted by the protocol —
        // NEVER forwarded: a doubling forwarder would otherwise echo the
        // request back doubled. Data frames on the SAME stream keep
        // flowing through the forwarder after the exchange.
        let desc = LoadedStageDescriptor {
            model_digest: [9u8; 32],
            layer_start: 4,
            layer_end: 8,
            is_first: false,
            is_last: true,
        };
        let (node_a, node_b, b_addr) =
            two_node_shard_fixture_with(true, std::sync::Arc::new(AttestedDoublingForwarder(desc)))
                .await;
        let conn = open_shard_connection(node_a.endpoint(), node_a.memory_lookup(), b_addr)
            .await
            .expect("dial");
        let (mut send, mut recv) = conn.open_bi().await.expect("open_bi");
        let att = request_stage_attestation(&mut send, &mut recv)
            .await
            .expect("the stage must answer its attestation");
        assert_eq!(
            att,
            ShardStageAttestation::from_loaded_stage(Some(desc)),
            "the attestation must carry the forwarder's self-declared stage"
        );
        // The stream is still a live data plane after the exchange.
        write_frame(&mut send, b"ab").await.expect("write data");
        let out = read_frame(&mut recv)
            .await
            .expect("read ok")
            .expect("a forwarded frame");
        assert_eq!(out, b"abab", "data frames still run through the forwarder");
        send.finish().ok();
        conn.close(0u32.into(), b"done");

        // A transport-only stage (no loaded_stage impl) does NOT intercept
        // the attestation — it ECHOES the request frame (Codex P1: a
        // transport-only stage must never swallow a frame). A driver of a
        // REAL session that dials such a stage therefore receives the
        // request bytes back, fails to decode them as an attestation reply,
        // and fail-closes — the binding is preserved WITHOUT the collision.
        let (node_c, node_d, d_addr) =
            two_node_shard_fixture_with(true, std::sync::Arc::new(DoublingForwarder)).await;
        let conn2 = open_shard_connection(node_c.endpoint(), node_c.memory_lookup(), d_addr)
            .await
            .expect("dial");
        let (mut send2, mut recv2) = conn2.open_bi().await.expect("open_bi");
        assert!(
            request_stage_attestation(&mut send2, &mut recv2)
                .await
                .is_err(),
            "a transport-only stage must NOT answer an attestation — it echoes, \
             so the driver fails to decode a reply and fail-closes"
        );
        send2.finish().ok();
        conn2.close(0u32.into(), b"done");

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
        node_c.shutdown().await.ok();
        node_d.shutdown().await.ok();
    }

    #[tokio::test]
    async fn shard_echo_never_swallows_an_attestation_shaped_prompt() {
        // Codex GPT-5.6 Sol P1 — the collision regression: a transport-only
        // echo session whose prompt IS the attestation-request JSON must be
        // echoed BYTE-IDENTICAL, never intercepted (that would break the
        // echo invariant + the "0-bump by construction" claim). Also test
        // the 3-leading-spaces 40-byte variant Codex called out.
        for prompt in [
            br#"{"v":1,"kind":"attest-stage-request"}"#.to_vec(),
            br#"   {"v":1,"kind":"attest-stage-request"}"#.to_vec(),
        ] {
            let (node_a, node_b, b_addr) =
                two_node_shard_fixture_with(true, std::sync::Arc::new(EchoForwarder)).await;
            let conn = open_shard_connection(node_a.endpoint(), node_a.memory_lookup(), b_addr)
                .await
                .expect("dial");
            let (mut send, mut recv) = conn.open_bi().await.expect("open_bi");
            write_frame(&mut send, &prompt).await.expect("write");
            let echo = read_frame(&mut recv)
                .await
                .expect("read ok")
                .expect("a frame, not EOF");
            assert_eq!(
                echo, prompt,
                "an echo stage must return an attestation-shaped prompt UNCHANGED"
            );
            send.finish().ok();
            conn.close(0u32.into(), b"done");
            node_a.shutdown().await.ok();
            node_b.shutdown().await.ok();
        }
    }

    #[tokio::test]
    async fn shard_forwarder_error_closes_cleanly() {
        // A forwarder that refuses a frame must abort the connection cleanly:
        // the server surfaces an AcceptError (no panic), and the client gets
        // no forwarded frame back (an error or a clean EOF, never a payload).
        let (node_a, node_b, b_addr) =
            two_node_shard_fixture_with(true, std::sync::Arc::new(FailingForwarder)).await;
        let conn = open_shard_connection(node_a.endpoint(), node_a.memory_lookup(), b_addr)
            .await
            .expect("dial");
        let (mut send, mut recv) = conn.open_bi().await.expect("open_bi");
        let _ = write_frame(&mut send, b"will-be-refused").await;
        let got = read_frame(&mut recv).await;
        assert!(
            matches!(got, Err(_) | Ok(None)),
            "a refused frame must not yield a forwarded payload (got {got:?})"
        );
        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }
}
