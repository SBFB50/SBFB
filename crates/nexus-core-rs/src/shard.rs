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
/// One frame carries a boundary hidden-state tensor between two
/// layer-blocks: for a 70B-class model (hidden dim ~8K, fp16) a multi-token
/// prefill activation can reach tens of MiB, so the cap is generous (64
/// MiB) but bounded — mirrors `MAX_SEED_MSG_BYTES` (a few-hundred-byte seed
/// request needed only 64 KiB). Enforced at both [`write_frame`] and
/// [`read_frame`].
pub const MAX_SHARD_FRAME_BYTES: usize = 64 * 1024 * 1024;

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

/// The shard data-plane ALPN handler for `sbfb/shard/1`.
///
/// Holds a verified [`ComputeGroupEntry`] allowlist. On each inbound
/// connection it admits the peer iff `conn.remote_id()` is a member,
/// rejecting a non-member at the handshake before any frame is read.
///
/// Phase B processes admitted frames with an **echo** — this proves the
/// bidirectional length-prefixed data plane works over a reused long-lived
/// `open_bi`. Phase F replaces the echo body with the real layer-block
/// forward (load `layer_start..layer_end`, forward, send the boundary
/// hidden state downstream).
#[derive(Debug, Clone)]
pub struct ShardProtocol {
    admission: std::sync::Arc<ComputeGroupEntry>,
}

impl ShardProtocol {
    /// Build a handler from an allowlist, verifying its signature up front
    /// so a malformed / forged group can never be installed.
    pub fn new(group: ComputeGroupEntry) -> Result<Self> {
        group.verify_signature()?;
        Ok(ShardProtocol {
            admission: std::sync::Arc::new(group),
        })
    }

    /// Build a handler from an already-verified allowlist (used by the
    /// factory, which verifies once before constructing the closure).
    fn from_verified(group: std::sync::Arc<ComputeGroupEntry>) -> Self {
        ShardProtocol { admission: group }
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

        // Admitted: serve the long-lived bi-stream. Phase B echoes each
        // frame (Phase F forwards the layer block instead).
        let (mut send, mut recv) = conn.accept_bi().await?;
        while let Some(frame) = read_frame(&mut recv).await.map_err(AcceptError::from_err)? {
            write_frame(&mut send, &frame)
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
/// the in-memory allowlist — so the factory closure ignores all three.
pub fn shard_protocol_factory(group: ComputeGroupEntry) -> Result<ExtraProtocolFactory> {
    group.verify_signature()?;
    let group = std::sync::Arc::new(group);
    Ok(Box::new(move |_store, _ep, _ml| {
        Box::new(ShardProtocol::from_verified(group)) as Box<dyn DynProtocolHandler>
    }))
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

    // ---- Two-node data-plane tests (in-process, mirror seed fixture) ----

    /// Spin up client node A (known identity) and server node B running a
    /// [`ShardProtocol`] whose allowlist contains A iff `admit_a`.
    async fn two_node_shard_fixture(admit_a: bool) -> (Node, Node, EndpointAddr) {
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
        let factory = shard_protocol_factory(entry).expect("verified allowlist");
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
}
