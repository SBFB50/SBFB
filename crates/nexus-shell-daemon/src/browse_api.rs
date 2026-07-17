// SPDX-License-Identifier: AGPL-3.0-or-later
//! Browse + nodes loopback HTTP domain — extracted verbatim from `http.rs`
//! (Sprint 82 Phase S2, PO-10 extended discipline: the domain's 5
//! router-driven and direct-call tests co-migrated below via
//! `crate::test_support`).
//!
//! `GET /api/daemon/browse` serves the reachability-annotated view of
//! every project across every cached curator list, each row carrying the
//! derived `is_own` (KEEP-ONLINE-READ-PATH, S74 Phase G) and
//! CATALOG-BACKED `from_subscribed` (UX-ARRIVAL, SEC-UXARR-1) flags;
//! `POST /api/daemon/browse/pull` broadcasts a browse_request over
//! gossip behind the duress gate; `GET /api/daemon/nodes` is the Sprint
//! 75 Phase D additive node-identity projection chosen over un-skipping
//! `BrowseEntry.node_id`, keeping the `/browse` surface byte-identical
//! (verrou 4: the anchor is a DISCOVERY source, never an authority).
//! T0 tier: the routes stay registered in `crate::http::build_router`
//! inside `authed_routes` (loopback bearer + Host + Origin) and re-point
//! here by full path; route paths, JSON shapes and status codes are
//! unchanged. The SHARED test-only `BrowseListResponse` DTO, the
//! directory-only pull-resolution cluster and the browse-index
//! chokepoint (`index_browse_entry` / `trustworthy_open_source`) stay in
//! `http.rs` (multi-domain consumers: `blob_serve`, `seed_api`,
//! `deploy`, `runtime`).

use std::sync::Arc;
use std::time::SystemTime;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use nexus_shell_daemon_core::browse::BrowseEntry;
use serde::Serialize;
use tracing::debug;

use crate::http::DaemonHttpState;

/// `GET /browse` — Phase D reachability-annotated view of every
/// project across every cached curator list.
///
/// The aggregator flattens the Phase C curator runtime's list
/// snapshot, probes each referenced project endpoint (honouring
/// the TTL cache), and returns a sorted vector the React shell
/// renders as a Browse page. If the curator runtime is empty
/// (no subscribed curators, or no announcements received yet)
/// this returns `{"entries": []}` at 200 rather than an error —
/// the shell renders an empty-state card in that case.
/// A browse entry plus the daemon-derived `is_own` flag (KEEP-ONLINE-READ-PATH,
/// carry S74 Phase G). `is_own` is true iff the entry's hosting `node_id` equals
/// THIS node's id — the precise "did this node publish it" signal. It fixes the
/// shell's old `isOwn = (node_id === project_id)` heuristic, which is always
/// false for per-app deploys whose `project_id = blake3(name) != node_id`, so
/// the owner "Garder en ligne" toggle never rendered. A voluntarily-seeded
/// distant app keeps the AUTHOR's node_id, so it is correctly `is_own = false`
/// (the shell shows the volunteer CTA, never the owner toggle). `node_id` itself
/// stays `#[serde(skip)]`; only this derived boolean crosses to the shell.
/// UX-ARRIVAL (post-S75): `from_subscribed` is the second derived flag — the
/// shell uses it to split the arrival grid (MY sources) from the separate
/// "Découvert sur le réseau" section without un-skipping `node_id`.
///
/// The flag is CATALOG-BACKED, never attention-set-membership of the claimed
/// `node_id` alone (review SEC-UXARR-1/WIRE-UXA-1, skeptics-confirmed P1): a
/// `ProjectAnnouncement` carries NO signature, so its `node_id` is a freely
/// claimed string — deriving trust placement from "claimed node_id is
/// subscribed" would let one PoW-paying announcer name a public anchor's
/// pubkey and land an attacker app inside "Tes sources" (and the hero). So a
/// `direct` entry is `from_subscribed` only when the `(project_id,
/// archive_hash)` pair appears in the claimed node's Ed25519-VERIFIED signed
/// directory catalog (the PULL substrate): a spoofer cannot put rows into an
/// anchor's signed catalog, while a subscribed node's real apps are listed
/// there by construction (publish → directory revision > 0 → boot
/// re-announce). A subscribed node that never published a directory has its
/// pushed `direct` entries land in the discovery section instead — honest,
/// and consistent with the `/nodes` "waiting for first announcement" row.
///
/// Only DECISIVE for `direct` entries: the shell already classes `curator` /
/// `nodedirectory` rows by `source` (both subscription-gated at ingest — a
/// `curator` row's `node_id` is `None`, so the flag reads `false` there
/// without meaning "unsolicited"; a `nodedirectory` row matches its own
/// catalog by construction). Serialize-only, like `is_own` (§P58.2): zero
/// churn on the ~26 `BrowseEntry` construction sites.
#[derive(Serialize)]
struct BrowseEntryView {
    #[serde(flatten)]
    entry: BrowseEntry,
    is_own: bool,
    from_subscribed: bool,
}

/// `node_id_hex → {(project_id, archive_hash)}` of every SUBSCRIBED anchor's
/// Ed25519-verified catalog, all lowercase, empty hashes skipped (a
/// placeholder row proves nothing about a fetchable app). Built from
/// `directory_snapshot()`, which is itself `is_subscribed`-gated.
fn subscribed_catalog_index(
    dirs: &[nexus_core_rs::NodeDirectoryEntry],
) -> std::collections::HashMap<String, std::collections::HashSet<(String, String)>> {
    let mut index: std::collections::HashMap<String, std::collections::HashSet<(String, String)>> =
        std::collections::HashMap::new();
    for dir in dirs {
        let claims = index.entry(hex::encode(dir.directory.node_id)).or_default();
        for app in &dir.directory.catalog {
            if app.archive_hash.is_empty() {
                continue;
            }
            claims.insert((
                app.project_id.to_ascii_lowercase(),
                app.archive_hash.to_ascii_lowercase(),
            ));
        }
    }
    index
}

/// Pure projection from the aggregator rows to the `/browse` payload —
/// extracted so the derived `is_own` / `from_subscribed` flags are pinned by
/// unit tests (own / catalog-backed / spoofed / unknown) without a network
/// boot.
fn browse_views(
    entries: Vec<BrowseEntry>,
    me: &str,
    catalog_index: &std::collections::HashMap<String, std::collections::HashSet<(String, String)>>,
) -> Vec<BrowseEntryView> {
    entries
        .into_iter()
        .map(|entry| {
            let is_own = entry.node_id.as_deref() == Some(me);
            // Catalog-backed check: the claimed node must have a VERIFIED
            // signed catalog listing exactly this (project_id, archive_hash).
            // Everything is normalized lowercase (§P59.3) so hex case can
            // neither dodge nor fake the classification. An entry with no
            // archive_hash has no content address to match — never classed
            // as "from my sources" on a bare claim.
            let from_subscribed = is_own
                || match (entry.node_id.as_deref(), entry.archive_hash.as_deref()) {
                    (Some(node), Some(hash)) => catalog_index
                        .get(&node.to_ascii_lowercase())
                        .map(|claims| {
                            claims.contains(&(
                                entry.project_id.to_ascii_lowercase(),
                                hash.to_ascii_lowercase(),
                            ))
                        })
                        .unwrap_or(false),
                    _ => false,
                };
            BrowseEntryView {
                is_own,
                from_subscribed,
                entry,
            }
        })
        .collect()
}

pub(crate) async fn list_browse(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    debug!("GET /browse");
    let entries = state
        .browse_aggregator
        .aggregate(&state.curator_runtime, &state.node)
        .await;
    let catalog_index = subscribed_catalog_index(&state.curator_runtime.directory_snapshot());
    let views = browse_views(entries, state.node_id.as_str(), &catalog_index);
    (
        StatusCode::OK,
        Json(serde_json::json!({ "entries": views })),
    )
}

/// `POST /api/daemon/browse/pull` — broadcast a browse_request
/// via gossip so peers replay their outbox. Returns immediately.
pub(crate) async fn browse_pull(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    if crate::noop_identity::gossip_publish_in_duress(state.identity_mode)
        == crate::noop_identity::PublishOutcome::Noop
    {
        return (
            StatusCode::OK,
            Json(serde_json::json!({"requested": false})),
        );
    }
    let _ = state
        .gossip_cmd_tx
        .send(crate::runtime::GossipCmd::RequestBrowse)
        .await;
    (StatusCode::OK, Json(serde_json::json!({"requested": true})))
}

/// `GET /api/daemon/nodes` response envelope (Sprint 75 Phase D).
///
/// ENVELOPE, not a bare array (S73-E lesson — the search route pins
/// `{results,total,took_ms}` for the same reason): the Phase-F frontend Zod
/// schema validates `{ nodes: [...] }` and additive fields stay possible.
/// One element per SUBSCRIBED publishing node — the directory store is keyed
/// by node pubkey, so the grouping is structural, never recomputed.
#[derive(Debug, Serialize)]
struct NodesResponse {
    nodes: Vec<NodeSummary>,
    /// UX-ARRIVAL (post-S75): NON-subscribed publishers heard on gossip —
    /// cheap-envelope metadata only (the catalog blob is never fetched for an
    /// unsolicited announce, THREAT_MODEL §15.1), surfaced so the arrival
    /// screen can offer a subscribe CTA. ALWAYS serialized (even empty): the
    /// frontend envelope schema is `.strict()`, so this key must never be
    /// conditional.
    observed: Vec<ObservedNodeView>,
}

/// One observed (non-subscribed) publisher in [`NodesResponse`]. Two fields
/// by design — `revision`/`app_count` live in the signed blob, which is never
/// fetched for a non-subscribed node (preflight UX-ARRIVAL, S4 trace 1): this
/// identity is PoW-backed metadata, not an Ed25519-verified catalog claim.
#[derive(Debug, Serialize)]
struct ObservedNodeView {
    /// Lowercase hex Ed25519 pubkey the announcement named.
    node_id: String,
    /// Unix seconds (LOCAL receive clock) of the last accepted announce.
    last_seen: u64,
}

/// One catalog-publishing node in [`NodesResponse`].
#[derive(Debug, Serialize)]
struct NodeSummary {
    /// Lowercase hex Ed25519 pubkey — the node's dialable identity AND the
    /// signing identity of its directory (they are the same key).
    node_id: String,
    /// The directory's monotonic revision (anti-rollback floor).
    revision: u64,
    /// Convenience count of catalog rows.
    app_count: usize,
    /// The advertised apps, verbatim from the verified signed directory.
    /// The anchor is a DISCOVERY source, never an authority: provenance is
    /// derived from the author-signed provenance.json at pull time (verrou 4).
    catalog: Vec<nexus_core_rs::CatalogApp>,
}

/// Pure projection from the verified directory snapshot to the `/nodes`
/// response — extracted so the envelope shape is pinned by a unit test
/// without a network boot.
fn nodes_response(
    dirs: Vec<nexus_core_rs::NodeDirectoryEntry>,
    observed: Vec<([u8; 32], u64)>,
) -> NodesResponse {
    NodesResponse {
        nodes: dirs
            .into_iter()
            .map(|d| NodeSummary {
                node_id: hex::encode(d.directory.node_id),
                revision: d.directory.revision,
                app_count: d.directory.catalog.len(),
                catalog: d.directory.catalog,
            })
            .collect(),
        observed: observed
            .into_iter()
            .map(|(pubkey, last_seen)| ObservedNodeView {
                // `hex::encode` is lowercase by contract (§P59.3 read side).
                node_id: hex::encode(pubkey),
                last_seen,
            })
            .collect(),
    }
}

/// `GET /api/daemon/nodes` — Sprint 75 Phase D — node identity exposure.
///
/// Read-only projection of every SUBSCRIBED node directory (already
/// signature-verified + revision-gated at ingest), grouped by publishing
/// node. This is the additive route chosen over un-skipping
/// `BrowseEntry.node_id`, which would have changed the `/browse` bytes —
/// the preflight S2/S4 trace keeps that surface byte-identical. The full
/// node-Browse front (`/nodes` page) consumes this in Phase F.
pub(crate) async fn list_nodes(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    debug!("GET /api/daemon/nodes");
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    (
        StatusCode::OK,
        Json(nodes_response(
            state.curator_runtime.directory_snapshot(),
            state.curator_runtime.observed_snapshot(now),
        )),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::{Method, Request};
    use nexus_core_rs::{KeyPair, create_node};
    use tower::ServiceExt;

    use crate::http::BrowseListResponse;
    use crate::test_support::*;

    #[test]
    fn browse_views_derives_from_subscribed() {
        // UX-ARRIVAL: the derived flag that splits MY sources from the
        // unsolicited ambient is CATALOG-BACKED (review SEC-UXARR-1, P1
        // skeptics-confirmed): a ProjectAnnouncement's node_id is a free
        // unsigned claim, so naming a subscribed anchor must NOT buy the
        // "Tes sources" placement — only an app the claimed node lists in
        // its Ed25519-verified signed catalog qualifies.
        let me = "11".repeat(32);
        let kp_friend = KeyPair::generate();
        let friend = hex::encode(kp_friend.public_bytes());
        let stranger = "33".repeat(32);
        let listed_hash = "ab".repeat(32);

        // The subscribed friend's VERIFIED catalog: lists (friend-app,
        // listed_hash) plus a placeholder row (empty hash) that the index
        // must skip — a placeholder proves nothing fetchable.
        let mut dir = nexus_core_rs::NodeDirectory::new(kp_friend.public_bytes(), 1);
        dir.catalog = vec![
            catalog_app("friend-app", &listed_hash, "FriendApp"),
            catalog_app("placeholder-app", "", "Placeholder"),
        ];
        let entry = nexus_core_rs::NodeDirectoryEntry::sign(dir, &kp_friend).unwrap();
        let index = subscribed_catalog_index(&[entry]);
        assert!(
            !index[&friend].contains(&("placeholder-app".into(), String::new())),
            "an empty-hash placeholder row must not enter the index"
        );

        let with_hash = |pid: &str, name: &str, owner: Option<String>, hash: Option<String>| {
            let mut e = own_browse_entry(pid, name, owner);
            e.archive_hash = hash;
            e
        };
        let views = browse_views(
            vec![
                with_hash("own-app", "OwnApp", Some(me.clone()), None),
                with_hash(
                    "friend-app",
                    "FriendApp",
                    Some(friend.clone()),
                    Some(listed_hash.clone()),
                ),
                // THE spoof: claims the SUBSCRIBED friend's node_id, but the
                // (pid, hash) pair is NOT in the friend's signed catalog.
                with_hash(
                    "spoof-app",
                    "SpoofApp",
                    Some(friend.clone()),
                    Some("cc".repeat(32)),
                ),
                with_hash(
                    "stranger-app",
                    "StrangerApp",
                    Some(stranger),
                    Some(listed_hash.clone()),
                ),
                // Hex-case probe: node_id AND hash uppercased.
                with_hash(
                    "friend-app",
                    "MixedCase",
                    Some(friend.to_ascii_uppercase()),
                    Some(listed_hash.to_ascii_uppercase()),
                ),
                // A bare claim with no content address is never "my sources".
                with_hash("no-hash-app", "NoHash", Some(friend.clone()), None),
                with_hash("curator-app", "CuratorApp", None, None),
            ],
            &me,
            &index,
        );

        let by_name = |name: &str| {
            views
                .iter()
                .find(|v| v.entry.project_name == name)
                .expect("fixture row present")
        };
        let own = by_name("OwnApp");
        assert!(own.is_own, "hosting node_id == me");
        assert!(own.from_subscribed, "own implies from_subscribed");
        let friend_view = by_name("FriendApp");
        assert!(!friend_view.is_own);
        assert!(
            friend_view.from_subscribed,
            "a catalog-listed app of a subscribed node belongs to MY sources"
        );
        assert!(
            !by_name("SpoofApp").from_subscribed,
            "naming a subscribed node_id without a signed catalog row must NOT buy the placement (SEC-UXARR-1)"
        );
        assert!(
            !by_name("StrangerApp").from_subscribed,
            "an unknown announcer is the ambient (unsolicited) class"
        );
        // Hex-case normalization: case can neither fake nor dodge the split.
        assert!(by_name("MixedCase").from_subscribed);
        assert!(
            !by_name("NoHash").from_subscribed,
            "no archive_hash = no content address to verify against the catalog"
        );
        assert!(
            !by_name("CuratorApp").from_subscribed,
            "a None-node_id row reads false (non-decisive: classed by source)"
        );

        // The serialized row carries BOTH derived keys (the Zod entry schema
        // is .strict(): key and schema ship in the same commit).
        let json = serde_json::to_value(friend_view).unwrap();
        assert_eq!(json["from_subscribed"], true);
        assert_eq!(json["is_own"], false);
    }

    #[test]
    fn nodes_response_pins_envelope_and_grouping() {
        // Plan D.3 #6 (renamed from `nodes_endpoint_groups_by_node_id` for
        // honesty: this pins the PROJECTION — the entire handler body — and
        // the route itself is traversed over HTTP in
        // `reachable_via_seeder_status` part (c)). The /api/daemon/nodes
        // ENVELOPE shape is pinned now, before the Phase-F frontend consumer
        // exists (S73-E lesson: envelope, not bare array; S72-D lesson: never
        // ship a consumer-less shape without a producer-side pin test). Two
        // apps of one node stay grouped under ONE node element.
        let kp_a = KeyPair::generate();
        let kp_b = KeyPair::generate();
        let mut dir_a = nexus_core_rs::NodeDirectory::new(kp_a.public_bytes(), 3);
        dir_a.catalog = vec![
            catalog_app(&"1".repeat(64), &"a1".repeat(32), "Babel"),
            catalog_app(&"2".repeat(64), &"a2".repeat(32), "Atlas"),
        ];
        let entry_a = nexus_core_rs::NodeDirectoryEntry::sign(dir_a, &kp_a).unwrap();
        let mut dir_b = nexus_core_rs::NodeDirectory::new(kp_b.public_bytes(), 7);
        dir_b.catalog = vec![catalog_app(&"3".repeat(64), &"b1".repeat(32), "Solo")];
        let entry_b = nexus_core_rs::NodeDirectoryEntry::sign(dir_b, &kp_b).unwrap();

        // UX-ARRIVAL: the envelope also carries the observed (non-subscribed)
        // publishers — two cheap-envelope fields, freshest-first order pinned
        // by `observed_snapshot`, lowercase hex out.
        let observed_pk = [0xabu8; 32];
        let json = serde_json::to_value(nodes_response(
            vec![entry_a, entry_b],
            vec![(observed_pk, 1_700_000_123)],
        ))
        .unwrap();

        let nodes = json["nodes"]
            .as_array()
            .expect("envelope: a top-level `nodes` array, never a bare array");
        assert_eq!(nodes.len(), 2, "one element per publishing node");
        let observed = json["observed"]
            .as_array()
            .expect("envelope: a top-level `observed` array (always present — the frontend envelope is .strict())");
        assert_eq!(observed.len(), 1);
        assert_eq!(observed[0]["node_id"], hex::encode(observed_pk));
        assert_eq!(observed[0]["last_seen"], 1_700_000_123u64);
        assert_eq!(
            observed[0].as_object().unwrap().len(),
            2,
            "observed rows are cheap-envelope metadata: node_id + last_seen, never revision/app_count (no fetch for a non-subscribed node)"
        );
        // The envelope key-count is pinned (review WIRE-UXA-2): the frontend
        // schema is .strict() on the envelope, so ANY new top-level key must
        // ship both sides in the same commit — this assertion is the seam.
        assert_eq!(
            json.as_object().unwrap().len(),
            2,
            "envelope = exactly {{nodes, observed}}"
        );
        // The empty shape still serializes the key (the .strict() contract).
        let empty = serde_json::to_value(nodes_response(vec![], vec![])).unwrap();
        assert!(empty["observed"].as_array().unwrap().is_empty());
        assert!(empty["nodes"].as_array().unwrap().is_empty());
        assert_eq!(empty.as_object().unwrap().len(), 2);
        assert_eq!(nodes[0]["node_id"], hex::encode(kp_a.public_bytes()));
        assert_eq!(nodes[0]["revision"], 3);
        assert_eq!(nodes[0]["app_count"], 2);
        let cat = nodes[0]["catalog"].as_array().unwrap();
        assert_eq!(cat.len(), 2, "both apps grouped under their node");
        assert_eq!(cat[0]["project_id"], "1".repeat(64));
        assert_eq!(cat[0]["archive_hash"], "a1".repeat(32));
        assert_eq!(cat[0]["project_name"], "Babel");
        assert_eq!(nodes[1]["node_id"], hex::encode(kp_b.public_bytes()));
        assert_eq!(nodes[1]["revision"], 7);
        assert_eq!(nodes[1]["app_count"], 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn reachable_via_seeder_status() {
        // Q7 (plan D.3 #5) — the HONEST backend signal pair for a
        // directory-only app whose anchor is dead but whose bytes a seeder
        // still holds: (a) the Browse row NEVER lies `Reachable` on the dead
        // anchor, (b) the version-exact seed-count reports the live seeder.
        // The visible "reachable-via-seeder" badge that renders this pair is
        // Phase F (keeping `/browse` byte-identical in a core+daemon phase).
        let state = mk_state().await;
        let host = create_node().await.expect("boot host node");

        // The anchor identity never boots a node → its probe can only fail.
        let kp_anchor = KeyPair::generate();
        let pid = "d".repeat(64);
        let archive_hash = "ee".repeat(32);
        ingest_remote_directory(
            &state,
            &host,
            &kp_anchor,
            vec![catalog_app(&pid, &archive_hash, "Ghost App")],
            1,
        )
        .await;

        // A live seeder announced it holds this exact archive version.
        let seeder = hex::encode(KeyPair::generate().public_bytes());
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        state
            .seed_registry
            .record(&pid, &archive_hash, &seeder, now, now);

        // (a) The browse row for the directory app reports the ANCHOR truth.
        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/browse")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 1 << 20).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let entries = json["entries"].as_array().expect("entries array");
        let row = entries
            .iter()
            .find(|e| e["project_id"] == pid)
            .expect("the directory app must be discoverable (verrou 2)");
        assert_eq!(row["source"], "nodedirectory");
        assert_eq!(
            row["status"], "unreachable",
            "a dead anchor must never be reported Reachable (Q7 honesty)"
        );

        // (b) The version-exact seed-count carries the live-seeder signal.
        let app = build_test_router(state.clone());
        let uri = format!("/api/daemon/seed-count/{pid}?archive_hash={archive_hash}");
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&uri)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json["peer_count"], 1,
            "the seeder holding the BLAKE3 must be visible in the backend signal"
        );
        assert_eq!(json["self_seeding"], false);
        // WEB-1 (Phase F): never-toggled app → the persisted intent is null,
        // NOT false — the shell toggle must not render OFF for it.
        assert_eq!(json["self_pin_enabled"], serde_json::Value::Null);

        // (c) Route-level coverage of GET /api/daemon/nodes (the envelope
        // shape itself is pinned by `nodes_response_pins_envelope_and_grouping`):
        // the registered path serves the subscribed anchor's catalog.
        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/nodes")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 1 << 20).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let nodes = json["nodes"].as_array().expect("envelope over HTTP");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0]["node_id"], hex::encode(kp_anchor.public_bytes()));
        assert_eq!(nodes[0]["catalog"][0]["project_id"], pid);

        host.shutdown().await.ok();
    }

    #[tokio::test]
    async fn browse_returns_empty_list_when_no_curators_cached() {
        // Phase D smoke test: with an empty curator runtime the
        // aggregator has nothing to flatten, so /browse returns
        // `{"entries": []}` at 200. The full Reachable/Unreachable
        // behaviour is covered by the 2-node integration tests
        // in `browse::tests::aggregate_probes_seeded_peer_*`.
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/browse")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let list: BrowseListResponse = serde_json::from_slice(&body).unwrap();
        assert!(list.entries.is_empty());
    }

    #[tokio::test]
    async fn api_daemon_browse_still_returns_json_with_web_root() {
        let state = mk_state().await;
        let (app, _tmp) = build_test_router_with_web_root(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/browse")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let list: BrowseListResponse = serde_json::from_slice(&body).unwrap();
        assert!(list.entries.is_empty());
    }
}
