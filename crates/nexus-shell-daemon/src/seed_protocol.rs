// SPDX-License-Identifier: AGPL-3.0-or-later
//! Cross-node seed protocol handler + client (Sprint 74 Phase E).
//!
//! ALPN `sbfb/seed/0`. A node (the *requester*, typically the app's
//! author) dials a chosen peer (the *seeder*) and asks it to keep an app
//! online. The seeder verifies the [`SeedRequestEnvelope`], fetches +
//! pins the archive blob, persists a local `keep_online` row, and replies
//! with a signed [`SeedResponseEnvelope`].
//!
//! ## Where the handler lives
//!
//! In the daemon, not in `nexus-core-rs`, because it carries the
//! coordinator DB (`keep_online` + `seed_invite` ledger) and the node
//! keypair. The iroh [`Router`] accepts no post-spawn protocol
//! registration, so the handler is built by a factory closure
//! (`nexus_core_rs::node::ExtraProtocolFactory`) that
//! `create_node_with_protocols` invokes with the freshly-created store /
//! endpoint / lookup. See `runtime.rs` for the wiring.
//!
//! ## Security (preflight S3 threat model)
//!
//! - **Forged request** → Ed25519 verify against `author_pubkey`
//!   (domain-separated) — rejected.
//! - **Signed but relayed by a third party** → cross-check
//!   `author_pubkey == conn.remote_id()` (the QUIC-authenticated dialer).
//! - **Replay** → 32-byte nonce cache (TTL) + `ts` freshness window.
//! - **No / revoked / expired invite** → checked against the local
//!   revocable `seed_invite` ledger (Tailscale model).
//! - **Malicious seeder serving altered content** → impossible: the blob
//!   is content-addressed (BLAKE3), the fetch verifies the hash, and the
//!   seeder re-pins the AUTHOR-signed archive bytes unchanged (R5 — the
//!   seeder signs no provenance).
//!
//! The **voluntary community seed** path (a node helping keep a public
//! app online of its own accord) does NOT go through this handler — it is
//! a unilateral local act (`http::seed_voluntary` → `fetch_and_pin` +
//! `set_keep_online`), since the content is already public and
//! content-addressed.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use iroh::Endpoint;
use iroh::address_lookup::memory::MemoryLookup;
use iroh::endpoint::Connection;
use iroh::protocol::{AcceptError, ProtocolHandler};
use iroh_blobs::api::Store;
use nexus_coordinator_rs::db::{CoordinatorDb, SeedInviteOutcome};
use nexus_core_rs::seed::{
    SEED_FORMAT_VERSION, SEED_NONCE_LEN, SEED_TS_WINDOW_SECS, SeedRequestEnvelope, SeedResponse,
    SeedResponseEnvelope,
};
use nexus_core_rs::{BlobsClient, KeyPair};
use tracing::{debug, warn};

use crate::deploy::{decode_hash_hex, keep_online_tag};

/// Maximum bytes accepted on the seed bi-stream (anti-DoS). A
/// `SeedRequestEnvelope` is a few hundred bytes; 64 KiB is generous.
const MAX_SEED_MSG_BYTES: usize = 64 * 1024;

type SharedDb = Arc<Mutex<CoordinatorDb>>;

/// Nonce retention in the anti-replay cache. Set to `2 * window + 1`
/// seconds on purpose (Codex C3): the `ts` gate accepts a request while
/// `abs_diff(now, ts) <= SEED_TS_WINDOW_SECS` (INCLUSIVE both ends), so a
/// single request is `ts`-fresh over the closed real-time interval
/// `[ts - window, ts + window]` — a span of exactly `2 * window` seconds
/// (e.g. a future-skewed `ts = now + window`). The cache purges an entry
/// once `elapsed() >= TTL`. To guarantee the nonce is still cached at the
/// LAST instant the request would pass the `ts` gate (`elapsed == 2*window`
/// when first seen at `ts - window`), the TTL must be strictly greater
/// than `2 * window` — hence the `+ 1`. A bare `2 * window` would leave a
/// one-second boundary hole where the nonce is purged but the timestamp is
/// still fresh.
const SEED_NONCE_TTL_SECS: u64 = SEED_TS_WINDOW_SECS * 2 + 1;

/// In-memory anti-replay cache of seen nonces, keyed by the 32-byte
/// nonce. Entries older than [`SEED_NONCE_TTL_SECS`] are purged lazily on
/// each insert — a nonce only has value while a request bearing it could
/// still pass the `ts` gate, so no DB persistence is needed.
#[derive(Debug, Default)]
pub struct NonceCache {
    seen: Mutex<HashMap<[u8; SEED_NONCE_LEN], Instant>>,
}

impl NonceCache {
    /// Record a nonce, returning `true` if it was fresh (newly recorded)
    /// or `false` if it was already seen within the retention window (a
    /// replay).
    pub fn check_and_record(&self, nonce: [u8; SEED_NONCE_LEN]) -> bool {
        let ttl = Duration::from_secs(SEED_NONCE_TTL_SECS);
        let mut guard = self.seen.lock().unwrap_or_else(|p| p.into_inner());
        guard.retain(|_, seen_at| seen_at.elapsed() < ttl);
        if guard.contains_key(&nonce) {
            return false;
        }
        guard.insert(nonce, Instant::now());
        true
    }
}

/// The seeder-side ALPN handler for `sbfb/seed/0`.
#[derive(Debug, Clone)]
pub struct SeedProtocol {
    store: Store,
    endpoint: Endpoint,
    memory_lookup: MemoryLookup,
    db: SharedDb,
    keypair: Arc<KeyPair>,
    nonce_cache: Arc<NonceCache>,
}

impl SeedProtocol {
    /// Build a handler. Typically called from the
    /// `ExtraProtocolFactory` closure with the node's store/endpoint/
    /// lookup, capturing the daemon's DB + keypair + nonce cache.
    pub fn new(
        store: Store,
        endpoint: Endpoint,
        memory_lookup: MemoryLookup,
        db: SharedDb,
        keypair: Arc<KeyPair>,
        nonce_cache: Arc<NonceCache>,
    ) -> Self {
        SeedProtocol {
            store,
            endpoint,
            memory_lookup,
            db,
            keypair,
            nonce_cache,
        }
    }

    /// Verify + (if authorized) fetch+pin, returning the [`SeedResponse`]
    /// to sign and send back. `dialer` is the QUIC-authenticated remote
    /// endpoint id bytes. Pure of any stream I/O so the accept handler
    /// stays a thin transport shell.
    async fn handle_request(&self, req_bytes: &[u8], dialer: &[u8; 32]) -> SeedResponse {
        let now = now_secs();
        let env: SeedRequestEnvelope = match serde_json::from_slice(req_bytes) {
            Ok(e) => e,
            Err(e) => {
                debug!(error = %e, "seed: undecodable request");
                return SeedResponse::rejected(String::new(), Vec::new(), "bad-request", now);
            }
        };
        let pid = env.request.project_id.clone();
        let nonce = env.request.nonce.clone();

        // 1. Version.
        if env.request.version != SEED_FORMAT_VERSION {
            return SeedResponse::rejected(pid, nonce, "bad-request", now);
        }
        // 2. Signature + attribution (requester_node_id == author_pubkey).
        if env.verify_signature().is_err() {
            return SeedResponse::rejected(pid, nonce, "bad-sig", now);
        }
        // 3. Dialer cross-check: the QUIC peer that opened this connection
        //    must be the same identity that signed the request. A third
        //    party cannot re-dial under someone else's id (no TLS key).
        if &env.author_pubkey != dialer {
            return SeedResponse::rejected(pid, nonce, "bad-request", now);
        }
        // 4. Freshness window.
        if env.request.ts.abs_diff(now) > SEED_TS_WINDOW_SECS {
            return SeedResponse::rejected(pid, nonce, "stale-ts", now);
        }
        // 5. Nonce shape + anti-replay.
        let nonce_arr: [u8; SEED_NONCE_LEN] =
            match <[u8; SEED_NONCE_LEN]>::try_from(nonce.as_slice()) {
                Ok(a) => a,
                Err(_) => return SeedResponse::rejected(pid, nonce, "bad-request", now),
            };
        if !self.nonce_cache.check_and_record(nonce_arr) {
            return SeedResponse::rejected(pid, nonce, "replay", now);
        }
        // 6. Invite authorization (authenticated path requires a valid,
        //    non-revoked, unexpired token bound to THIS app). The
        //    voluntary path never reaches this handler.
        if env.request.invite_token.is_empty() {
            return SeedResponse::rejected(pid, nonce, "no-invite", now);
        }
        let outcome = {
            let db = self.db.lock().unwrap_or_else(|p| p.into_inner());
            // The invite is a capability over the (project_id, archive_hash)
            // PAIR: passing the request's archive_hash makes consume reject a
            // token minted for different content, so an invited peer cannot
            // make us pin foreign content under the app's tag (review P2).
            db.consume_seed_invite(
                &env.request.invite_token,
                &pid,
                &env.request.archive_hash,
                now as i64,
            )
        };
        match outcome {
            Ok(SeedInviteOutcome::Ok) => {}
            Ok(SeedInviteOutcome::NotFound) => {
                return SeedResponse::rejected(pid, nonce, "no-invite", now);
            }
            Ok(SeedInviteOutcome::Revoked) => {
                return SeedResponse::rejected(pid, nonce, "invite-revoked", now);
            }
            Ok(SeedInviteOutcome::Expired) => {
                return SeedResponse::rejected(pid, nonce, "invite-expired", now);
            }
            Ok(SeedInviteOutcome::NoUsesLeft) => {
                return SeedResponse::rejected(pid, nonce, "invite-exhausted", now);
            }
            Err(e) => {
                warn!(error = %e, "seed: invite check DB error");
                return SeedResponse::rejected(pid, nonce, "not-approved", now);
            }
        }
        // 7. Validate the declared hash shape.
        let want_hash = match decode_hash_hex(&env.request.archive_hash) {
            Some(h) => h,
            None => return SeedResponse::rejected(pid, nonce, "bad-request", now),
        };
        // 8. Fetch + pin (content-addressing guarantees integrity).
        let blobs = BlobsClient::new(&self.store);
        let tag = keep_online_tag(&pid);
        let fetched = match blobs
            .fetch_and_pin(
                &self.endpoint,
                &self.memory_lookup,
                &env.request.archive_ticket,
                &tag,
            )
            .await
        {
            Ok(h) => h,
            Err(e) => {
                debug!(error = %e, project = %pid, "seed: fetch_and_pin failed");
                return SeedResponse::rejected(pid, nonce, "fetch-failed", now);
            }
        };
        // Defence in depth: the ticket's content hash must match the
        // declared archive_hash (content-addressing already guarantees the
        // bytes match the ticket hash; this rejects a request whose
        // declared hash disagrees with its ticket).
        if fetched != want_hash {
            let _ = blobs.delete_tag(&tag).await;
            return SeedResponse::rejected(pid, nonce, "fetch-failed", now);
        }
        // 9. Persist the seeder-side keep_online pin so the boot
        //    re-announce (Phase F) re-diffuses it after a reboot.
        {
            let db = self.db.lock().unwrap_or_else(|p| p.into_inner());
            if let Err(e) = db.set_keep_online(&pid, true, Some(&env.request.archive_hash)) {
                warn!(error = %e, "seed: keep_online persist failed (non-fatal)");
            }
        }
        debug!(project = %pid, "seed: accepted — fetched, pinned, kept online");
        SeedResponse::accepted(pid, nonce, now)
    }
}

impl ProtocolHandler for SeedProtocol {
    async fn accept(&self, conn: Connection) -> Result<(), AcceptError> {
        let dialer = *conn.remote_id().as_bytes();
        let (mut send, mut recv) = conn.accept_bi().await?;
        let req_bytes = recv
            .read_to_end(MAX_SEED_MSG_BYTES)
            .await
            .map_err(AcceptError::from_err)?;

        let response = self.handle_request(&req_bytes, &dialer).await;

        let resp_env =
            SeedResponseEnvelope::sign(response, &self.keypair).map_err(AcceptError::from_err)?;
        let resp_bytes = serde_json::to_vec(&resp_env).map_err(AcceptError::from_err)?;
        send.write_all(&resp_bytes)
            .await
            .map_err(AcceptError::from_err)?;
        send.finish().map_err(AcceptError::from_err)?;
        conn.closed().await;
        Ok(())
    }
}

/// Client side: dial `peer_addr` over `sbfb/seed/0`, send the signed
/// request, and return the seeder's signed response (verified).
///
/// Cross-checks that the response is signed by the very peer dialed
/// (`response.author_pubkey == conn.remote_id()`), so a relay cannot
/// forge an "accepted" ack on the seeder's behalf.
///
/// The production caller is `http::seed_request_peer`
/// (`POST /api/daemon/seed/request`, Sprint 75 Phase E): the loopback-
/// scriptable requester leg of the headless anchor model — after a
/// deploy, a script asks a designated peer (typically the operator's VPS)
/// to seed the app, no browser required. The richer peer-designation UI
/// remains deferred ("Bientot") and will reuse the same route.
pub async fn request_seed(
    endpoint: &Endpoint,
    memory_lookup: &MemoryLookup,
    peer_addr: iroh::EndpointAddr,
    envelope: &SeedRequestEnvelope,
) -> Result<SeedResponseEnvelope, String> {
    // Seed the address lookup so the endpoint can dial without pkarr.
    memory_lookup.add_endpoint_info(peer_addr.clone());
    let conn = endpoint
        .connect(peer_addr, nexus_core_rs::node::SEED_ALPN)
        .await
        .map_err(|e| format!("seed dial failed: {e}"))?;
    let peer_id = *conn.remote_id().as_bytes();

    let (mut send, mut recv) = conn
        .open_bi()
        .await
        .map_err(|e| format!("seed open_bi failed: {e}"))?;
    let body = serde_json::to_vec(envelope).map_err(|e| format!("seed encode failed: {e}"))?;
    send.write_all(&body)
        .await
        .map_err(|e| format!("seed write failed: {e}"))?;
    send.finish()
        .map_err(|e| format!("seed finish failed: {e}"))?;

    let resp_bytes = recv
        .read_to_end(MAX_SEED_MSG_BYTES)
        .await
        .map_err(|e| format!("seed read failed: {e}"))?;
    let resp: SeedResponseEnvelope =
        serde_json::from_slice(&resp_bytes).map_err(|e| format!("seed decode failed: {e}"))?;
    resp.verify_signature()
        .map_err(|e| format!("seed response signature invalid: {e}"))?;
    if resp.author_pubkey != peer_id {
        return Err("seed response signed by a different peer than dialed".into());
    }
    conn.close(0u32.into(), b"done");
    Ok(resp)
}

/// Build the `ExtraProtocolFactory` that `create_node_with_protocols`
/// invokes (with the node's freshly-created store/endpoint/lookup) to wire
/// the seed handler onto the `sbfb/seed/0` ALPN. Keeps the iroh
/// protocol-handler types out of `runtime.rs`.
pub fn seed_protocol_factory(
    db: SharedDb,
    keypair: Arc<KeyPair>,
    nonce_cache: Arc<NonceCache>,
) -> nexus_core_rs::node::ExtraProtocolFactory {
    Box::new(move |store: &Store, ep: &Endpoint, ml: &MemoryLookup| {
        Box::new(SeedProtocol::new(
            store.clone(),
            ep.clone(),
            ml.clone(),
            db,
            keypair,
            nonce_cache,
        )) as Box<dyn iroh::protocol::DynProtocolHandler>
    })
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use iroh_blobs::{BlobFormat, Hash};
    use nexus_core_rs::discovery::DiscoveryClient;
    use nexus_core_rs::node::{SEED_ALPN, create_node_with_protocols};
    use nexus_core_rs::seed::{SeedRequest, random_nonce};
    use nexus_core_rs::{BlobsClient, KeyPair, NodeConfig, create_node, create_node_with_config};

    fn mk_db() -> SharedDb {
        Arc::new(Mutex::new(CoordinatorDb::open_in_memory().expect("db")))
    }

    /// Spin up author node A (known identity, hosts a blob) and seeder
    /// node B (runs SeedProtocol). Returns the pieces the tests drive.
    async fn two_node_fixture() -> (
        nexus_core_rs::Node, // A (author)
        KeyPair,             // A's signing key (== A's iroh identity)
        String,              // archive_ticket
        [u8; 32],            // archive hash
        nexus_core_rs::Node, // B (seeder)
        SharedDb,            // B's db
        iroh::EndpointAddr,  // B's addr
    ) {
        let a_secret = KeyPair::generate().secret_bytes();
        let a_kp = KeyPair::from_secret_bytes(&a_secret);
        let node_a = create_node_with_config(NodeConfig::default().with_secret_key(a_secret))
            .await
            .expect("node A");
        let blobs_a = BlobsClient::new(node_a.blobs_store());
        let payload = b"author-signed-app-archive-bytes".to_vec();
        let hash = blobs_a.add_bytes(&payload).await.unwrap();
        let a_addr = DiscoveryClient::new(node_a.endpoint())
            .my_endpoint_addr()
            .await
            .expect("A addr");
        let ticket =
            iroh_blobs::ticket::BlobTicket::new(a_addr, Hash::from_bytes(hash), BlobFormat::Raw)
                .to_string();

        let b_db = mk_db();
        // The seeder signs its SeedResponse with `b_kp`; the requester
        // cross-checks `response.author_pubkey == conn.remote_id()`, so the
        // seeder's signing key MUST match node B's iroh identity (in prod the
        // daemon derives both from the same secret — runtime.rs).
        let b_secret = KeyPair::generate().secret_bytes();
        let b_kp = Arc::new(KeyPair::from_secret_bytes(&b_secret));
        let nonce_cache = Arc::new(NonceCache::default());
        let factory = seed_protocol_factory(Arc::clone(&b_db), Arc::clone(&b_kp), nonce_cache);
        let node_b = create_node_with_protocols(
            NodeConfig::default().with_secret_key(b_secret),
            vec![(SEED_ALPN.to_vec(), factory)],
        )
        .await
        .expect("node B");
        let b_addr = DiscoveryClient::new(node_b.endpoint())
            .my_endpoint_addr()
            .await
            .expect("B addr");

        (node_a, a_kp, ticket, hash, node_b, b_db, b_addr)
    }

    fn mk_request(
        a_kp: &KeyPair,
        project_id: &str,
        hash: &[u8; 32],
        ticket: &str,
        invite_token: &str,
    ) -> SeedRequestEnvelope {
        let req = SeedRequest {
            version: SEED_FORMAT_VERSION,
            project_id: project_id.to_string(),
            archive_hash: hex::encode(hash),
            archive_ticket: ticket.to_string(),
            requester_node_id: a_kp.public_bytes(),
            nonce: random_nonce(),
            ts: now_secs(),
            invite_token: invite_token.to_string(),
        };
        SeedRequestEnvelope::sign(req, a_kp).unwrap()
    }

    #[test]
    fn nonce_cache_rejects_replay() {
        let cache = NonceCache::default();
        let n = [7u8; SEED_NONCE_LEN];
        assert!(cache.check_and_record(n), "first sighting is fresh");
        assert!(!cache.check_and_record(n), "second sighting is a replay");
        // A different nonce is still fresh.
        assert!(cache.check_and_record([9u8; SEED_NONCE_LEN]));
    }

    #[tokio::test]
    async fn seed_e2e_two_nodes_peer_keeps_app_reachable() {
        let (node_a, a_kp, ticket, hash, node_b, b_db, b_addr) = two_node_fixture().await;
        let pid = "seed-e2e-app";
        // The seeder (B) mints an invite for this app + its exact content; the
        // author (A) redeems it.
        b_db.lock()
            .unwrap()
            .mint_seed_invite("tok-e2e", pid, &hex::encode(hash), 9_999_999_999, Some(1))
            .unwrap();

        let env = mk_request(&a_kp, pid, &hash, &ticket, "tok-e2e");
        let resp = request_seed(node_a.endpoint(), node_a.memory_lookup(), b_addr, &env)
            .await
            .expect("request_seed");
        assert_eq!(
            resp.response.decision,
            nexus_core_rs::seed::SeedDecision::Accepted,
            "reason: {}",
            resp.response.reason
        );

        // B now holds the blob (pinned) and recorded keep_online.
        let blobs_b = BlobsClient::new(node_b.blobs_store());
        assert!(
            blobs_b.has(hash).await.unwrap(),
            "seeder must hold the blob"
        );
        assert_eq!(
            b_db.lock().unwrap().get_keep_online(pid).unwrap(),
            Some((true, Some(hex::encode(hash))))
        );

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    #[tokio::test]
    async fn seed_requires_invite_and_approval() {
        let (node_a, a_kp, ticket, hash, node_b, _b_db, b_addr) = two_node_fixture().await;
        let pid = "seed-noinvite-app";
        // No invite minted; request carries an empty token.
        let env = mk_request(&a_kp, pid, &hash, &ticket, "");
        let resp = request_seed(node_a.endpoint(), node_a.memory_lookup(), b_addr, &env)
            .await
            .expect("request_seed");
        assert_eq!(
            resp.response.decision,
            nexus_core_rs::seed::SeedDecision::Rejected
        );
        assert_eq!(resp.response.reason, "no-invite");
        // The seeder did NOT fetch the blob.
        let blobs_b = BlobsClient::new(node_b.blobs_store());
        assert!(!blobs_b.has(hash).await.unwrap());

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    #[tokio::test]
    async fn seeded_app_keeps_author_provenance_intact() {
        // The seeder re-pins the AUTHOR's exact bytes (content-addressed),
        // never re-signing provenance: B's copy is byte-identical to A's.
        let (node_a, a_kp, ticket, hash, node_b, b_db, b_addr) = two_node_fixture().await;
        let pid = "seed-prov-app";
        b_db.lock()
            .unwrap()
            .mint_seed_invite("tok-prov", pid, &hex::encode(hash), 9_999_999_999, None)
            .unwrap();
        let env = mk_request(&a_kp, pid, &hash, &ticket, "tok-prov");
        let resp = request_seed(node_a.endpoint(), node_a.memory_lookup(), b_addr, &env)
            .await
            .unwrap();
        assert_eq!(
            resp.response.decision,
            nexus_core_rs::seed::SeedDecision::Accepted
        );
        // B's stored bytes == A's original (the author-signed archive).
        let blobs_a = BlobsClient::new(node_a.blobs_store());
        let blobs_b = BlobsClient::new(node_b.blobs_store());
        let from_a = blobs_a.get_bytes(hash).await.unwrap();
        let from_b = blobs_b.get_bytes(hash).await.unwrap();
        assert_eq!(from_a, from_b, "seeder must serve the author's exact bytes");
        // The keep_online row carries the AUTHOR's archive_hash unchanged.
        assert_eq!(
            b_db.lock().unwrap().get_keep_online(pid).unwrap(),
            Some((true, Some(hex::encode(hash))))
        );

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    #[tokio::test]
    async fn seed_request_signature_verified_over_the_wire() {
        // A tampered envelope (mutated payload after signing) is rejected
        // by the seeder with "bad-sig".
        let (node_a, a_kp, ticket, hash, node_b, b_db, b_addr) = two_node_fixture().await;
        let pid = "seed-badsig-app";
        b_db.lock()
            .unwrap()
            .mint_seed_invite("tok-bs", pid, &hex::encode(hash), 9_999_999_999, None)
            .unwrap();
        let mut env = mk_request(&a_kp, pid, &hash, &ticket, "tok-bs");
        // Tamper: change the archive_hash after signing.
        env.request.archive_hash = "f".repeat(64);
        let resp = request_seed(node_a.endpoint(), node_a.memory_lookup(), b_addr, &env)
            .await
            .unwrap();
        assert_eq!(
            resp.response.decision,
            nexus_core_rs::seed::SeedDecision::Rejected
        );
        assert_eq!(resp.response.reason, "bad-sig");

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    // ---- Handler-level rejection-path tests (review P1) ----
    //
    // These call `handle_request` directly (no wire) so each
    // security-critical rejection branch is asserted by reason code. The
    // pre-fetch branches (version, dialer, ts, replay, invite) need no peer;
    // the content-hash-mismatch branch needs a real source node.

    /// Build a standalone seeder handler on a fresh node (no running ALPN
    /// needed — tests call `handle_request` directly).
    async fn mk_standalone_handler() -> (SeedProtocol, SharedDb, nexus_core_rs::Node) {
        let node = create_node().await.expect("handler node");
        let db = mk_db();
        let h = SeedProtocol::new(
            node.blobs_store().clone(),
            node.endpoint().clone(),
            node.memory_lookup().clone(),
            Arc::clone(&db),
            Arc::new(KeyPair::generate()),
            Arc::new(NonceCache::default()),
        );
        (h, db, node)
    }

    /// JSON-encode a signed SeedRequest with explicit ts/nonce control.
    #[allow(clippy::too_many_arguments)]
    fn req_bytes(
        req_kp: &KeyPair,
        version: u16,
        pid: &str,
        archive_hash_hex: &str,
        ticket: &str,
        token: &str,
        ts: u64,
        nonce: Vec<u8>,
    ) -> Vec<u8> {
        let req = SeedRequest {
            version,
            project_id: pid.to_string(),
            archive_hash: archive_hash_hex.to_string(),
            archive_ticket: ticket.to_string(),
            requester_node_id: req_kp.public_bytes(),
            nonce,
            ts,
            invite_token: token.to_string(),
        };
        serde_json::to_vec(&SeedRequestEnvelope::sign(req, req_kp).unwrap()).unwrap()
    }

    #[tokio::test]
    async fn handler_rejects_dialer_mismatch() {
        // A validly-signed request relayed by a third party (dialer != the
        // signer) must be rejected — the central anti-relay defense.
        let (h, _db, node) = mk_standalone_handler().await;
        let req_kp = KeyPair::generate();
        let other = KeyPair::generate();
        let bytes = req_bytes(
            &req_kp,
            SEED_FORMAT_VERSION,
            "app",
            &"a".repeat(64),
            "tkt",
            "tok",
            now_secs(),
            random_nonce(),
        );
        let resp = h.handle_request(&bytes, &other.public_bytes()).await;
        assert_eq!(resp.decision, nexus_core_rs::seed::SeedDecision::Rejected);
        assert_eq!(resp.reason, "bad-request");
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn handler_rejects_stale_ts() {
        let (h, _db, node) = mk_standalone_handler().await;
        let req_kp = KeyPair::generate();
        let stale = now_secs().saturating_sub(SEED_TS_WINDOW_SECS + 60);
        let bytes = req_bytes(
            &req_kp,
            SEED_FORMAT_VERSION,
            "app",
            &"a".repeat(64),
            "tkt",
            "tok",
            stale,
            random_nonce(),
        );
        let resp = h.handle_request(&bytes, &req_kp.public_bytes()).await;
        assert_eq!(resp.reason, "stale-ts");
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn handler_rejects_bad_version() {
        let (h, _db, node) = mk_standalone_handler().await;
        let req_kp = KeyPair::generate();
        let bytes = req_bytes(
            &req_kp,
            SEED_FORMAT_VERSION + 1,
            "app",
            &"a".repeat(64),
            "tkt",
            "tok",
            now_secs(),
            random_nonce(),
        );
        let resp = h.handle_request(&bytes, &req_kp.public_bytes()).await;
        assert_eq!(resp.reason, "bad-request");
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn handler_rejects_replay_through_handler() {
        // The same nonce twice through the handler: the nonce is recorded on
        // the first pass (step 5, before invite), so the second is "replay".
        let (h, _db, node) = mk_standalone_handler().await;
        let req_kp = KeyPair::generate();
        let nonce = random_nonce();
        let bytes = req_bytes(
            &req_kp,
            SEED_FORMAT_VERSION,
            "app",
            &"a".repeat(64),
            "tkt",
            "",
            now_secs(),
            nonce.clone(),
        );
        // First pass: no invite -> "no-invite", but the nonce is now recorded.
        let r1 = h.handle_request(&bytes, &req_kp.public_bytes()).await;
        assert_eq!(r1.reason, "no-invite");
        // Second pass with the SAME nonce -> "replay" (before invite check).
        let r2 = h.handle_request(&bytes, &req_kp.public_bytes()).await;
        assert_eq!(r2.reason, "replay");
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn handler_rejects_revoked_invite() {
        let (h, db, node) = mk_standalone_handler().await;
        let req_kp = KeyPair::generate();
        let pid = "app";
        let ah = "a".repeat(64);
        db.lock()
            .unwrap()
            .mint_seed_invite("tok-r", pid, &ah, (now_secs() + 1000) as i64, None)
            .unwrap();
        assert!(db.lock().unwrap().revoke_seed_invite("tok-r").unwrap());
        let bytes = req_bytes(
            &req_kp,
            SEED_FORMAT_VERSION,
            pid,
            &ah,
            "tkt",
            "tok-r",
            now_secs(),
            random_nonce(),
        );
        let resp = h.handle_request(&bytes, &req_kp.public_bytes()).await;
        assert_eq!(resp.reason, "invite-revoked");
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn handler_rejects_expired_invite() {
        let (h, db, node) = mk_standalone_handler().await;
        let req_kp = KeyPair::generate();
        let pid = "app";
        let ah = "a".repeat(64);
        // Invite already expired (expires_at in the past) but the REQUEST ts is
        // fresh, so the ts gate passes and the invite-expiry branch fires.
        db.lock()
            .unwrap()
            .mint_seed_invite("tok-x", pid, &ah, (now_secs() - 10) as i64, None)
            .unwrap();
        let bytes = req_bytes(
            &req_kp,
            SEED_FORMAT_VERSION,
            pid,
            &ah,
            "tkt",
            "tok-x",
            now_secs(),
            random_nonce(),
        );
        let resp = h.handle_request(&bytes, &req_kp.public_bytes()).await;
        assert_eq!(resp.reason, "invite-expired");
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn handler_rejects_exhausted_invite() {
        // A single-use invite already redeemed maps to a DISTINCT reason
        // "invite-exhausted" (Codex C5: not collapsed into "invite-expired").
        // Pre-exhaust it at the DB level so the handler hits NoUsesLeft
        // without needing a real fetch.
        let (h, db, node) = mk_standalone_handler().await;
        let req_kp = KeyPair::generate();
        let pid = "app";
        let ah = "a".repeat(64);
        let exp = (now_secs() + 1000) as i64;
        {
            let g = db.lock().unwrap();
            g.mint_seed_invite("tok-e", pid, &ah, exp, Some(1)).unwrap();
            // Burn the single use directly.
            assert_eq!(
                g.consume_seed_invite("tok-e", pid, &ah, now_secs() as i64)
                    .unwrap(),
                nexus_coordinator_rs::db::SeedInviteOutcome::Ok
            );
        }
        let bytes = req_bytes(
            &req_kp,
            SEED_FORMAT_VERSION,
            pid,
            &ah,
            "tkt",
            "tok-e",
            now_secs(),
            random_nonce(),
        );
        let resp = h.handle_request(&bytes, &req_kp.public_bytes()).await;
        assert_eq!(resp.reason, "invite-exhausted");
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn handler_rejects_invite_for_different_archive_hash() {
        // P2: an invite minted for content X must NOT authorize seeding
        // content Y (capability over the (project, content) pair).
        let (h, db, node) = mk_standalone_handler().await;
        let req_kp = KeyPair::generate();
        let pid = "app";
        let authorized = "a".repeat(64);
        let attacker_hash = "b".repeat(64);
        db.lock()
            .unwrap()
            .mint_seed_invite("tok-p2", pid, &authorized, (now_secs() + 1000) as i64, None)
            .unwrap();
        // The request swaps in a DIFFERENT archive_hash than the invite bound.
        let bytes = req_bytes(
            &req_kp,
            SEED_FORMAT_VERSION,
            pid,
            &attacker_hash,
            "tkt",
            "tok-p2",
            now_secs(),
            random_nonce(),
        );
        let resp = h.handle_request(&bytes, &req_kp.public_bytes()).await;
        // Token does not authorize THIS content -> treated as no-invite.
        assert_eq!(resp.reason, "no-invite");
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn handler_rejects_content_hash_mismatch() {
        // The invite + declared hash are consistent (so the invite passes), but
        // the TICKET points at different bytes than declared -> after fetch the
        // hash check fires "fetch-failed" and the speculative pin is removed.
        let host = create_node().await.expect("host node");
        let blobs_host = BlobsClient::new(host.blobs_store());
        let real_hash = blobs_host.add_bytes(b"the-real-bytes").await.unwrap();
        let host_addr = DiscoveryClient::new(host.endpoint())
            .my_endpoint_addr()
            .await
            .expect("host addr");
        let real_ticket = iroh_blobs::ticket::BlobTicket::new(
            host_addr,
            Hash::from_bytes(real_hash),
            BlobFormat::Raw,
        )
        .to_string();

        let (h, db, node) = mk_standalone_handler().await;
        let req_kp = KeyPair::generate();
        let pid = "app";
        // Declare (and authorize) a DIFFERENT hash than the ticket actually serves.
        let declared = "c".repeat(64);
        db.lock()
            .unwrap()
            .mint_seed_invite("tok-mm", pid, &declared, (now_secs() + 1000) as i64, None)
            .unwrap();
        let bytes = req_bytes(
            &req_kp,
            SEED_FORMAT_VERSION,
            pid,
            &declared,
            &real_ticket,
            "tok-mm",
            now_secs(),
            random_nonce(),
        );
        let resp = h.handle_request(&bytes, &req_kp.public_bytes()).await;
        assert_eq!(resp.reason, "fetch-failed");
        // The speculative keep-online tag was rolled back (no leaked pin).
        let tag = keep_online_tag(pid);
        let tags = node.blobs_store().tags();
        assert!(
            tags.get(tag.as_bytes()).await.expect("tags get").is_none(),
            "a content-mismatch fetch must not leave a pin behind"
        );

        host.shutdown().await.ok();
        node.shutdown().await.ok();
    }
}
