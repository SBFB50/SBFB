// SPDX-License-Identifier: AGPL-3.0-or-later
//! Seed / keep-online loopback HTTP domain — extracted verbatim from
//! `http.rs` (Sprint 82 Phase O, PO-10 extended discipline: the domain's
//! tests co-migrated below via the shared `crate::test_support` harness).
//!
//! Voluntary community seeding (S74 E), keep-online pin (S74 D, M18),
//! seed invites bound to (project_id, archive_hash) (M19), the
//! cross-node seed request (`sbfb/seed/0`) and the headless boot seed
//! driver (S75 E). Routes stay registered in `crate::http::build_router`
//! inside `authed_routes` (loopback bearer + Host + Origin) and re-point
//! here by full path; route paths, JSON shapes and status codes are
//! unchanged. Invariant: heberger != publier, seeder != auteur.

use std::sync::Arc;
use std::time::SystemTime;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use tracing::{debug, info, warn};

use crate::http::{
    DIRECTORY_PULL_TIMEOUT_SECS, DaemonHttpState, directory_pull_providers,
    find_directory_app_by_project, mint_blob_ticket,
};

/// `POST /api/daemon/keep-online` — Sprint 74 Phase D — toggle a self-deployed
/// app's LOCAL pin. ON re-tags the archive blob (skip-GC) and lets the boot
/// re-broadcast diffuse it; OFF removes the per-intent tag (GC-eligible — no GC
/// runs today, so "stored but no longer diffused") and gates the re-broadcast
/// on EVERY outbox replay path (NeighborUp, browse_request, periodic republish).
/// Loopback-authenticated like every `/api/daemon` route.
#[derive(Debug, serde::Deserialize)]
pub(crate) struct KeepOnlineRequest {
    project_id: String,
    enabled: bool,
}

pub(crate) async fn set_keep_online(
    State(state): State<Arc<DaemonHttpState>>,
    Json(req): Json<KeepOnlineRequest>,
) -> impl IntoResponse {
    debug!(project = %req.project_id, enabled = req.enabled, "POST /api/daemon/keep-online");

    // Sprint 76 Phase B (B1, duress siblings): short-circuit BEFORE any local
    // mutation. A decoy node must perform ZERO keep_online persistence and ZERO
    // blob (un)tag — the duress launcher shares the operator's REAL
    // coordinator.db + blob store, so an un-gated toggle would pin/persist the
    // operator's real app set under the fake keypair, correlating the decoy
    // with the real node. Mirrors `run_boot_seed_driver` + `seed_voluntary`:
    // reply a plausible benign success so an observer cannot tell duress from a
    // normal toggle (the local-mutation half of the P1 wire-emit fix 23a08c9).
    if crate::noop_identity::gossip_publish_in_duress(state.identity_mode)
        == crate::noop_identity::PublishOutcome::Noop
    {
        return (
            StatusCode::OK,
            Json(serde_json::json!({"ok": true, "enabled": req.enabled})),
        )
            .into_response();
    }

    // The archive blob to (un)pin comes from the app's own Browse card.
    let archive_hash = state
        .browse_aggregator
        .get_direct_entry(&req.project_id)
        .and_then(|e| e.archive_hash.clone());

    {
        let db = state
            .coordinator_db
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        if let Err(e) = db.set_keep_online(&req.project_id, req.enabled, archive_hash.as_deref()) {
            warn!(error = %e, "keep_online DB write failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "keep_online write failed"})),
            )
                .into_response();
        }
    }

    // Tag/untag the archive blob (best-effort — the DB row is the source of
    // truth; a tag hiccup must not fail the toggle).
    let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
    let tag = crate::deploy::keep_online_tag(&req.project_id);
    if req.enabled {
        if let Some(arr) = archive_hash
            .as_deref()
            .and_then(crate::deploy::decode_hash_hex)
            && let Err(e) = blobs.set_tag(&tag, arr).await
        {
            debug!(error = %e, "keep-online tag set failed (non-fatal)");
        }
    } else if let Err(e) = blobs.delete_tag(&tag).await {
        debug!(error = %e, "keep-online tag delete failed (non-fatal)");
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({"ok": true, "enabled": req.enabled})),
    )
        .into_response()
}

/// Sprint 75 Phase E (D3): the headless boot seed driver. For every
/// project id the operator EXPLICITLY listed under `[seed]
/// keep_online_projects`, acquire the app's archive — an app this node may
/// have NEVER deployed locally — pin it under the keep-online tag
/// (skip-GC), persist the `keep_online` row, and announce the seed to the
/// feed. This is how an always-on anchor seeds its operator's chosen apps
/// without a UI session. An EMPTY list does zero work and zero network
/// calls (verrou 5: the boot fetch is config-driven explicit, never a
/// shipped default — verrou 3 keeps the compiled default empty).
///
/// Resolution order per project id (most-authoritative content source
/// first): the local direct browse entry (an app this node hosts,
/// restored from its own outbox), then the persisted `keep_online` row's
/// archive hash (M18, the hash source-of-truth across reboots), then the
/// SUBSCRIBED node directories (the "configured app I never had" case).
/// Acquisition picks the FIRST APPLICABLE source ONLY — bytes already
/// held locally (re-pin, no network), else the direct entry's ticket,
/// else the Phase D multi-provider chain (`directory_pull_providers` →
/// `fetch_and_pin_multi`, a bare-hash download — NEVER a ticket re-mint:
/// `mint_ticket_for_hash` is the producer helper and bails on a blob we
/// do not hold). There is NO cross-tier failover: a dead ticket tier is
/// one warn + skip (same shape as PULL-3, deferred to the S76 audit).
///
/// Sequential on purpose (one bounded network budget per app — the Phase
/// C re-pull pattern): a long list cannot fan out unbounded dials at
/// boot, and a fully-dead provider set costs at most one timeout per app.
/// Best-effort, ONE-SHOT per invocation: a failed app is logged and skipped,
/// the rest proceed. At boot this runs once; Sprint 82 Phase A additionally
/// re-drives it whenever a subscribed anchor's node directory is accepted via
/// gossip (`crate::runtime::maybe_redrive_seed_on_ingest`, cooldown-coalesced),
/// so the former "first-boot dead window" — a FRESH anchor (no persisted
/// `anchors.json` yet) whose configured app only exists in a not-yet-ingested
/// directory — now closes without a daemon restart or a manual
/// `POST /api/daemon/seed {project_id}`. Returns the number of apps pinned
/// (newly acquired or re-pinned).
pub(crate) async fn run_boot_seed_driver(
    state: &Arc<DaemonHttpState>,
    configured: &[String],
) -> u64 {
    // Duress short-circuit (mirrors every signing/publishing surface,
    // and the sibling `reannounce_directory_at_boot` via its DuressNoop):
    // a decoy node must perform ZERO seed acquisition, ZERO keep_online
    // mutation and ZERO `SeedAnnounced` emission. The launcher's duress
    // path swaps only the identity — config.toml, coordinator.db and the
    // blob store are the operator's REAL ones — so an un-gated driver
    // would re-pin and announce the real configured app set under the
    // fake keypair, correlating the decoy with the real node.
    if crate::noop_identity::gossip_publish_in_duress(state.identity_mode)
        == crate::noop_identity::PublishOutcome::Noop
    {
        return 0;
    }
    let mut pinned = 0u64;
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for pid in configured {
        if !seen.insert(pid.as_str()) {
            continue;
        }

        // --- Resolve the archive hash (+ the anchor when directory-resolved).
        let direct = state.browse_aggregator.get_direct_entry(pid);
        // Lexical block: the DB guard must provably never cross an await
        // (clippy::await_holding_lock reasons on scopes, not drop()).
        let keep_online_row = {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            db.get_keep_online(pid).ok().flatten()
        };
        // Trust boundary: the subscribed anchor IS the gate — a directory
        // hit pins whatever hash the FIRST advertising anchor (snapshot
        // sorted by node_id) signed for this project id, with BLAKE3 as
        // the only integrity check (no author-provenance verification at
        // auto-seed time). Multiple subscribed anchors advertising the
        // same project id resolve lexicographic-first; tracked with the
        // Sybil-sampling residual in the S76 audit.
        let dir_hit =
            find_directory_app_by_project(&state.curator_runtime.directory_snapshot(), pid, None);
        let Some(hash_hex) = direct
            .as_ref()
            .and_then(|e| e.archive_hash.clone())
            .or_else(|| keep_online_row.as_ref().and_then(|(_, h)| h.clone()))
            .or_else(|| dir_hit.as_ref().map(|(h, _)| h.clone()))
        else {
            warn!(
                project = %pid,
                "boot seed driver: configured app not resolvable yet (no direct entry, no keep_online hash, not in any subscribed directory) — skipped"
            );
            continue;
        };
        let Some(want_hash) = crate::deploy::decode_hash_hex(&hash_hex) else {
            warn!(project = %pid, hash = %hash_hex, "boot seed driver: malformed archive hash — skipped");
            continue;
        };

        // --- Acquire (or re-pin) the bytes, one bounded budget per app.
        let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
        let tag = crate::deploy::keep_online_tag(pid);
        let already_held = matches!(blobs.has(want_hash).await, Ok(true));
        let acquired = if already_held {
            // Re-pin (plan §E.3 #2): the blob survived in the store; make
            // sure the keep-online skip-GC tag does too — idempotent.
            match blobs.set_tag(&tag, want_hash).await {
                Ok(()) => true,
                Err(e) => {
                    warn!(project = %pid, error = %e, "boot seed driver: re-pin set_tag failed");
                    false
                }
            }
        } else if let Some(ticket) = direct.as_ref().and_then(|e| e.archive_ticket.clone()) {
            match tokio::time::timeout(
                std::time::Duration::from_secs(DIRECTORY_PULL_TIMEOUT_SECS),
                blobs.fetch_and_pin(
                    state.node.endpoint(),
                    state.node.memory_lookup(),
                    &ticket,
                    &tag,
                ),
            )
            .await
            {
                Ok(Ok(h)) if h == want_hash => true,
                Ok(Ok(_)) => {
                    // The ticket's content disagrees with the resolved hash:
                    // drop the misplaced pin (mirrors the seed handler).
                    let _ = blobs.delete_tag(&tag).await;
                    warn!(project = %pid, "boot seed driver: ticket content does not match the resolved archive hash — skipped");
                    false
                }
                Ok(Err(e)) => {
                    warn!(project = %pid, error = %e, "boot seed driver: ticket fetch failed");
                    false
                }
                Err(_) => {
                    warn!(project = %pid, "boot seed driver: ticket fetch timed out");
                    false
                }
            }
        } else if let Some((_, anchor_hex)) = dir_hit.as_ref() {
            let now = SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let providers = directory_pull_providers(
                &state.seed_registry,
                &state.node_id,
                anchor_hex,
                pid,
                &hash_hex,
                now,
            );
            if providers.is_empty() {
                warn!(project = %pid, "boot seed driver: no dialable provider for this app — skipped");
                false
            } else {
                match tokio::time::timeout(
                    std::time::Duration::from_secs(DIRECTORY_PULL_TIMEOUT_SECS),
                    blobs.fetch_and_pin_multi(state.node.endpoint(), want_hash, providers, &tag),
                )
                .await
                {
                    Ok(Ok(h)) if h == want_hash => true,
                    // Defensively unreachable: fetch_and_pin_multi returns
                    // the requested hash by construction (content-addressed
                    // download). Kept as the verrou-4 belt-and-braces guard.
                    Ok(Ok(_)) => {
                        let _ = blobs.delete_tag(&tag).await;
                        warn!(project = %pid, "boot seed driver: fetched content does not match the requested hash — skipped");
                        false
                    }
                    Ok(Err(e)) => {
                        warn!(project = %pid, error = %e, "boot seed driver: multi-provider pull failed");
                        false
                    }
                    Err(_) => {
                        warn!(project = %pid, "boot seed driver: multi-provider pull timed out across all providers");
                        false
                    }
                }
            }
        } else {
            warn!(
                project = %pid,
                "boot seed driver: hash known but no acquisition source (no local bytes, no ticket, no directory anchor) — skipped"
            );
            false
        };
        if !acquired {
            continue;
        }

        // --- Persist + announce.
        let was_already_announced = seed_already_announced(&keep_online_row, &hash_hex);
        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            if let Err(e) = db.set_keep_online(pid, true, Some(&hash_hex)) {
                warn!(project = %pid, error = %e, "boot seed driver: keep_online persist failed");
            }
        }
        // `reannounce_seeds_at_boot` already re-emitted `SeedAnnounced` for
        // every row that was ALREADY enabled with this hash when the daemon
        // booted — only emit for an app this driver newly acquired/enabled,
        // so a configured app never double-announces in one boot.
        if !was_already_announced
            && let Some(ref fs) = state.feed_sync_state
            && let Err(e) = crate::feed_sync::emit_seed_announced(
                fs,
                &state.coordinator_db,
                &state.pow_keypair,
                pid,
                &hash_hex,
            )
            .await
        {
            warn!(project = %pid, error = %e, "boot seed driver: seed announce failed (non-fatal)");
        }
        info!(project = %pid, held_locally = already_held, "boot seed driver: app pinned + kept online");
        pinned += 1;
    }
    pinned
}

/// Pure predicate behind the driver's anti-double-emission guard: was this
/// app ALREADY enabled with this EXACT hash when the daemon booted? If so,
/// `reannounce_seeds_at_boot` (awaited inline before the driver spawns)
/// already re-emitted its `SeedAnnounced` this boot, and the driver must
/// not emit a second one. A row that is disabled, hash-less, or enabled
/// for a DIFFERENT hash was not covered by the boot re-announce for the
/// hash being pinned now — emit.
pub(crate) fn seed_already_announced(row: &Option<(bool, Option<String>)>, hash_hex: &str) -> bool {
    matches!(row, Some((true, Some(h))) if h == hash_hex)
}

/// `POST /api/daemon/seed` — Sprint 74 Phase E — VOLUNTARY community seed.
/// This node helps keep a DISTANT public app online: it fetches the app's
/// archive blob, pins it under the keep-online tag (skip-GC), and records a
/// local `keep_online` row so the boot re-announce (Phase F) re-diffuses it.
/// No `SeedRequest`, no invite, no author approval — the content is already
/// public and content-addressed (BLAKE3), so a supporter can only ever hold
/// the author's exact bytes and never re-signs any provenance (the author
/// stays the author). Loopback-authenticated.
///
/// Two acquisition paths (Sprint 75 Phase D closed GAP R5b):
///  - a DIRECT (gossip) entry carries an archive ticket → single-provider
///    `fetch_and_pin` via the ticket, the original Phase-E path;
///  - a DIRECTORY-ONLY app (discovered through a subscribed node directory)
///    has NO ticket, only `(anchor node_id, archive_hash)` → multi-provider
///    `fetch_and_pin_multi` from the anchor + the best-effort seeders.
#[derive(Debug, serde::Deserialize)]
pub(crate) struct SeedVoluntaryRequest {
    project_id: String,
    /// Sprint 75 Phase F (review-D deferral): optional version discriminator.
    /// When present, the seed targets the EXACT archive version the user was
    /// shown — a direct entry carrying a DIFFERENT version no longer shadows
    /// the requested one, and the directory first-match is narrowed to this
    /// hash (multi-anchor collision). `#[serde(default)]` = runtime tolerance:
    /// a body omitting it keeps the pre-F version-agnostic behaviour.
    #[serde(default)]
    archive_hash: Option<String>,
}

/// How `seed_voluntary` acquires the archive bytes for the requested app.
enum SeedFetchPlan {
    /// Direct entry: dial the single provider embedded in the BlobTicket.
    Ticket(String),
    /// Directory-only app: ordered multi-provider fetch by bare hash (Q5).
    Multi(Vec<iroh::EndpointId>),
}

pub(crate) async fn seed_voluntary(
    State(state): State<Arc<DaemonHttpState>>,
    Json(req): Json<SeedVoluntaryRequest>,
) -> impl IntoResponse {
    debug!(project = %req.project_id, "POST /api/daemon/seed (voluntary)");

    // Sprint 76 Phase B (B1, duress siblings): short-circuit BEFORE any fetch,
    // pin, keep_online persist, or SeedAnnounced emit. A decoy node must perform
    // ZERO voluntary-seed work — the duress launcher shares the operator's REAL
    // blob store + coordinator.db, so an un-gated seed would pin the operator's
    // app set AND emit a SeedAnnounced under the fake keypair (the local-mutation
    // sibling of the P1 wire-emit fix 23a08c9; this single early-return covers
    // BOTH the local pin and the emit). Reply a plausible benign success.
    if crate::noop_identity::gossip_publish_in_duress(state.identity_mode)
        == crate::noop_identity::PublishOutcome::Noop
    {
        return (
            StatusCode::OK,
            Json(serde_json::json!({"ok": true, "seeding": req.project_id})),
        )
            .into_response();
    }

    // The app must be visible in Browse so we know its archive hash. A user
    // can only seed what they can see. A direct (gossip) entry wins WHEN it
    // can serve the request — it carries a ready ticket; otherwise fall back
    // to the subscribed node directories (directory-only apps have no ticket
    // by design: a stored ticket would freeze a stale address, the Phase A
    // bug). Sprint 75 Phase F (review-D deferral): a direct entry is skipped
    // when it has NO archive (a ticket-less card must not shadow a pullable
    // directory listing) or when the caller pinned a SPECIFIC version the
    // direct entry does not carry.
    // Reads normalize like writes (hex-case lesson, Phase D SeedRegistry): a
    // mixed-case hash from a raw client must match the lowercase hashes the
    // daemon mints everywhere, never miss on case alone.
    let requested_hash = req.archive_hash.as_deref().map(str::to_ascii_lowercase);
    let requested_hash = requested_hash.as_deref();
    let direct_entry = state.browse_aggregator.get_direct_entry(&req.project_id);
    let had_direct_entry = direct_entry.is_some();
    // The direct card's DISPLAYED hash even when it carries no ticket: the
    // agnostic fallback below must not silently pin a DIFFERENT version than
    // the one the direct card shows the user (review F P3 — pre-F this shape
    // was a 400, never a divergent pin).
    let direct_hash_no_ticket = direct_entry.as_ref().and_then(|e| {
        if e.archive_ticket.is_none() {
            e.archive_hash.clone()
        } else {
            None
        }
    });
    let direct_plan =
        direct_entry.and_then(|entry| match (entry.archive_ticket, entry.archive_hash) {
            (Some(ticket), Some(hash_hex)) => match requested_hash {
                Some(want) if want != hash_hex => None,
                _ => Some((hash_hex, SeedFetchPlan::Ticket(ticket))),
            },
            _ => None,
        });

    // Sprint 76 Phase B (B3, PULL-3): resolve the directory tier UP FRONT, even
    // when a direct ticket exists, so a dead ticket can fall through to it
    // instead of returning a terminal BAD_GATEWAY (audit S75 Track E: the
    // iroh-blobs downloader's intra-vector failover never covers a single dead
    // ticket, which is not a provider SEQUENCE). The fallback targets the SAME
    // content the direct tier would have served (cross-tier = same bytes,
    // different source); for a ticket-less app it targets the requested/displayed
    // version, exactly as before.
    let directory_constraint = direct_plan
        .as_ref()
        .map(|(h, _)| h.as_str())
        .or(requested_hash)
        .or(direct_hash_no_ticket.as_deref());
    let directory_hit = find_directory_app_by_project(
        &state.curator_runtime.directory_snapshot(),
        &req.project_id,
        directory_constraint,
    );
    let mut directory_hit_without_provider = false;
    let directory_plan: Option<(String, SeedFetchPlan)> =
        if let Some((hash_hex, anchor_hex)) = directory_hit {
            let now = SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let providers = directory_pull_providers(
                &state.seed_registry,
                &state.node_id,
                &anchor_hex,
                &req.project_id,
                &hash_hex,
                now,
            );
            if providers.is_empty() {
                directory_hit_without_provider = true;
                None
            } else {
                Some((hash_hex, SeedFetchPlan::Multi(providers)))
            }
        } else {
            None
        };

    let chain = build_seed_fetch_chain(direct_plan, directory_plan);
    if chain.is_empty() {
        // No tier resolved — preserve the precise pre-B3 error disambiguation.
        if directory_hit_without_provider {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({"error": "no dialable provider for this app"})),
            )
                .into_response();
        } else if had_direct_entry && requested_hash.is_none() {
            // A direct card with nothing to pull (no archive) and no directory
            // fallback is a 400, not an unknown app (pre-F behaviour preserved).
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "app has no archive to seed"})),
            )
                .into_response();
        } else if requested_hash.is_some() {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "no source for the requested app version"})),
            )
                .into_response();
        } else {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "unknown app (not in browse)"})),
            )
                .into_response();
        }
    }

    let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
    let tag = crate::deploy::keep_online_tag(&req.project_id);
    // Try each tier in order; the first that returns the wanted bytes wins. A
    // dead tier-1 ticket falls through to the tier-2 directory multi-provider.
    let mut last_error: (StatusCode, &'static str) =
        (StatusCode::BAD_GATEWAY, "could not fetch the app archive");
    for (hash_hex, plan) in chain {
        let Some(want_hash) = crate::deploy::decode_hash_hex(&hash_hex) else {
            last_error = (StatusCode::BAD_REQUEST, "app has a malformed archive hash");
            continue;
        };
        let fetched = match plan {
            SeedFetchPlan::Ticket(ticket) => {
                blobs
                    .fetch_and_pin(
                        state.node.endpoint(),
                        state.node.memory_lookup(),
                        &ticket,
                        &tag,
                    )
                    .await
            }
            SeedFetchPlan::Multi(providers) => {
                match tokio::time::timeout(
                    std::time::Duration::from_secs(DIRECTORY_PULL_TIMEOUT_SECS),
                    blobs.fetch_and_pin_multi(state.node.endpoint(), want_hash, providers, &tag),
                )
                .await
                {
                    Ok(r) => r,
                    Err(_) => Err(nexus_core_rs::NexusError::Blobs(
                        "directory pull timed out across all providers".into(),
                    )),
                }
            }
        };
        match fetched {
            Ok(h) if h == want_hash => {
                {
                    let db = state
                        .coordinator_db
                        .lock()
                        .unwrap_or_else(|p| p.into_inner());
                    if let Err(e) = db.set_keep_online(&req.project_id, true, Some(&hash_hex)) {
                        warn!(error = %e, "voluntary seed: keep_online persist failed");
                    }
                }
                // Sprint 74 Phase F: announce to the feed that this node now seeds
                // the distant app, so the author + other peers see "Toi + N pairs"
                // rise. The lock is taken+dropped inside the helper (never across
                // the await). Best-effort: a feed hiccup must not undo the pin.
                if let Some(ref fs) = state.feed_sync_state
                    && let Err(e) = crate::feed_sync::emit_seed_announced(
                        fs,
                        &state.coordinator_db,
                        &state.pow_keypair,
                        &req.project_id,
                        &hash_hex,
                    )
                    .await
                {
                    warn!(error = %e, "voluntary seed: SeedAnnounced emit failed (non-fatal)");
                }
                return (
                    StatusCode::OK,
                    Json(serde_json::json!({"ok": true, "seeding": req.project_id})),
                )
                    .into_response();
            }
            Ok(_) => {
                // Content hash disagreed with the declared hash — unpin and try
                // the next tier (a mismatched tier never wins).
                let _ = blobs.delete_tag(&tag).await;
                last_error = (StatusCode::BAD_GATEWAY, "fetched content hash mismatch");
            }
            Err(e) => {
                debug!(error = %e, "voluntary seed: tier fetch failed (trying next tier if any)");
                last_error = (StatusCode::BAD_GATEWAY, "could not fetch the app archive");
            }
        }
    }
    // Every tier failed (dead ticket AND no live directory provider).
    let (code, msg) = last_error;
    (code, Json(serde_json::json!({"error": msg}))).into_response()
}

/// Build the ordered cross-tier fetch chain for a voluntary seed (Sprint 76
/// Phase B, B3 PULL-3). Tier 1 is the direct entry's embedded ticket (a ready
/// provider address); tier 2 is the subscribed node directories' multi-provider
/// fetch by bare hash. A dead tier-1 ticket falls THROUGH to tier 2 instead of a
/// terminal BAD_GATEWAY — the cross-tier failover audit S75 Track E (PULL-3)
/// flagged as missing (a single ticket is not a provider SEQUENCE, so the
/// iroh-blobs downloader's intra-vector retry never covers it). Order is
/// load-bearing: the ticket is the cheapest single dial, the directory is the
/// resilient fallback. Pure + total so the chain shape is unit-testable without
/// a network.
fn build_seed_fetch_chain(
    direct_plan: Option<(String, SeedFetchPlan)>,
    directory_plan: Option<(String, SeedFetchPlan)>,
) -> Vec<(String, SeedFetchPlan)> {
    let mut chain = Vec::with_capacity(2);
    if let Some(p) = direct_plan {
        chain.push(p);
    }
    if let Some(p) = directory_plan {
        chain.push(p);
    }
    chain
}

/// Query string for [`seed_count`] (Sprint 75 Phase C, WIRE-2).
#[derive(Debug, serde::Deserialize)]
pub(crate) struct SeedCountQuery {
    /// Optional EXACT archive version to count. When present, `peer_count` is
    /// the seeders of that specific BLAKE3 hash (the honest "peers that can serve
    /// the bytes I am about to pull" answer) and `self_seeding` is true only if
    /// this node's own pin IS that version. When absent, the count is STRICTLY
    /// version-agnostic — the distinct seeders across all versions, the exact
    /// pre-WIRE-2 semantics (no silent substitution of this node's own pinned
    /// hash). Backward compatible: an old caller that omits it keeps the
    /// previous behaviour.
    #[serde(default)]
    archive_hash: Option<String>,
}

/// `GET /api/daemon/seed-count/{project_id}` — Sprint 74 Phase F — the
/// best-effort multi-seed availability count for an app.
///
/// Returns `{ peer_count, self_seeding, self_pin_enabled }`:
///  - `peer_count`: distinct REMOTE seeders seen within the TTL (from the
///    in-memory `SeedRegistry`, fed by ingested `SeedAnnounced` feed ops).
///  - `self_seeding`: whether THIS node actively keeps the app online (an
///    `enabled = 1` keep_online row). The front renders the pair as "Toi + N
///    pairs (vus recemment)" — `self_seeding` is the "Toi", `peer_count` the N.
///  - `self_pin_enabled` (Sprint 75 Phase F, WEB-1): the operator's PERSISTED
///    keep-online intent, three-valued — `null` = never toggled (no
///    keep_online row; the app still rebroadcasts by default, only an
///    explicit OFF row gates the outbox replay), `true`/`false` = the
///    explicit toggle state. Distinct from `self_seeding`, which is
///    version-scoped serving truth: the shell's "Garder en ligne" toggle
///    must reflect INTENT, and a fresh never-toggled own app must not render
///    OFF (it is still diffused via the outbox replay).
///
/// Best-effort by design (scope cut #11): content-addressing (BLAKE3) is the
/// truth of reachability, this count is only a freshness hint. A dedicated
/// route (vs a `seed_count` field on every BrowseEntry) keeps the count fetched
/// live with its TTL semantics and avoids churning every BrowseEntry site.
///
/// Sprint 75 Phase C (WIRE-2): an optional `?archive_hash=` scopes `peer_count`
/// to the seeders of that exact version (see [`SeedCountQuery`]).
pub(crate) async fn seed_count(
    State(state): State<Arc<DaemonHttpState>>,
    Path(project_id): Path<String>,
    axum::extract::Query(params): axum::extract::Query<SeedCountQuery>,
) -> impl IntoResponse {
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let keep_online_row = {
        let db = state
            .coordinator_db
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        db.get_keep_online(&project_id).ok().flatten()
    };
    // WEB-1: the raw persisted intent, BEFORE the row-absent default collapses
    // it — `None` (never toggled) must stay distinguishable from `Some(false)`
    // (explicit OFF) for the shell toggle.
    let self_pin_enabled: Option<bool> = keep_online_row.as_ref().map(|(enabled, _)| *enabled);
    let (keep_online_enabled, own_hash) = keep_online_row.unwrap_or((false, None));
    // Reads normalize like writes (hex-case lesson): without this, a
    // mixed-case query would still COUNT the version's peers (the registry
    // normalizes internally) while denying the "Toi" (the own_hash compare
    // below is byte-exact) — an inconsistent answer from one handler.
    let requested = params.archive_hash.as_deref().map(str::to_ascii_lowercase);
    let requested = requested.as_deref();
    // WIRE-2: `peer_count` is scoped to the EXACT version the caller asks about
    // (`?archive_hash=`), else a version-agnostic distinct count across all
    // versions when omitted. The omitted case is STRICTLY the pre-WIRE-2
    // non-regression semantics (`None`) — we do NOT silently substitute our own
    // pinned hash, which would surprise a caller that asked for an aggregate count
    // (Codex GAP). The shell passes the displayed entry's archive_hash on every
    // surface that knows it, so the version-specific path is the practical one.
    let peer_count = state
        .seed_registry
        .count_recent(&project_id, requested, now);
    // `self_seeding` ("Toi") must be HONEST about the queried version: when the
    // caller asks about a SPECIFIC archive_hash, this node only counts as a
    // self-seeder if its pinned hash IS that exact version. Without this check a
    // node pinning version Y would falsely claim "Toi" for a query about version
    // X (Codex GAP). With no version requested, it reflects the enabled state.
    let self_seeding = keep_online_enabled
        && match requested {
            Some(req) => own_hash.as_deref() == Some(req),
            None => true,
        };
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "peer_count": peer_count,
            "self_seeding": self_seeding,
            "self_pin_enabled": self_pin_enabled,
        })),
    )
        .into_response()
}

/// `POST /api/daemon/seed/invite` — Sprint 74 Phase E — mint a revocable seed
/// invite token (Tailscale model). The token authorizes a trusted peer to ask
/// THIS node, over the `sbfb/seed/0` protocol, to seed the given app. The invite
/// is bound to the app's CURRENT archive hash (derived from this node's own
/// browse view — "you can only authorize what you can see"), so an invited peer
/// cannot redeem it to make this node pin foreign content (review P2). Returns
/// the opaque token; the row stays local (only the token id ever travels).
#[derive(Debug, serde::Deserialize)]
pub(crate) struct SeedInviteMintRequest {
    project_id: String,
    /// Lifetime in seconds; defaults to 30 days (Tailscale default).
    expires_in_secs: Option<u64>,
    /// Optional cap on redemptions; `None` = reusable until expiry/revoke.
    max_uses: Option<i64>,
}

pub(crate) async fn seed_invite_mint(
    State(state): State<Arc<DaemonHttpState>>,
    Json(req): Json<SeedInviteMintRequest>,
) -> impl IntoResponse {
    // Bind the invite to the exact content this node currently sees for the app
    // (the operator can only authorize what is in their own browse view), not to
    // an attacker-chosen hash (review P2).
    let archive_hash = state
        .browse_aggregator
        .get_direct_entry(&req.project_id)
        .and_then(|e| e.archive_hash.clone());
    let Some(archive_hash) = archive_hash else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "app not visible (or has no archive) to authorize"})),
        )
            .into_response();
    };
    let token = hex::encode(nexus_core_rs::random_nonce());
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let ttl = req.expires_in_secs.unwrap_or(30 * 24 * 3600);
    let expires_at = now.saturating_add(ttl) as i64;
    {
        let db = state
            .coordinator_db
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        if let Err(e) = db.mint_seed_invite(
            &token,
            &req.project_id,
            &archive_hash,
            expires_at,
            req.max_uses,
        ) {
            warn!(error = %e, "seed invite mint failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "seed invite mint failed"})),
            )
                .into_response();
        }
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "token": token,
            "expires_at": expires_at,
            "archive_hash": archive_hash,
        })),
    )
        .into_response()
}

/// `POST /api/daemon/seed/invite/revoke` — Sprint 74 Phase E — revoke a seed
/// invite token in real time (the next `SeedRequest` carrying it is refused).
#[derive(Debug, serde::Deserialize)]
pub(crate) struct SeedInviteRevokeRequest {
    token: String,
}

pub(crate) async fn seed_invite_revoke(
    State(state): State<Arc<DaemonHttpState>>,
    Json(req): Json<SeedInviteRevokeRequest>,
) -> impl IntoResponse {
    let revoked = {
        let db = state
            .coordinator_db
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        db.revoke_seed_invite(&req.token).unwrap_or(false)
    };
    (
        StatusCode::OK,
        Json(serde_json::json!({"revoked": revoked})),
    )
        .into_response()
}

/// `GET /api/daemon/seed/invites/{project_id}` — Sprint 74 Phase E — list the
/// seed invites minted for an app, for the local management UI.
pub(crate) async fn seed_invite_list(
    State(state): State<Arc<DaemonHttpState>>,
    axum::extract::Path(project_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let rows = {
        let db = state
            .coordinator_db
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        db.list_seed_invites(&project_id).unwrap_or_default()
    };
    let items: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "token": r.token,
                "project_id": r.project_id,
                "archive_hash": r.archive_hash,
                "expires_at": r.expires_at,
                "max_uses": r.max_uses,
                "uses_count": r.uses_count,
                "revoked_at": r.revoked_at,
                "created_at": r.created_at,
            })
        })
        .collect();
    (StatusCode::OK, Json(serde_json::json!({"invites": items}))).into_response()
}

/// Wall-clock budget for one outbound `sbfb/seed/0` request (dial +
/// request + the seeder's own fetch of our archive + signed response).
/// The seeder side fetches the app archive BEFORE replying, so this is
/// aligned on [`DIRECTORY_PULL_TIMEOUT_SECS`] — the budget the codebase
/// already grants the equivalent transfer. NOTE for callers: a
/// 504 from the route does NOT prove the seed failed — the seeder may
/// still complete its fetch + pin after our deadline (and a single-use
/// invite is consumed BEFORE the fetch), so verify via the per-app
/// seed-count rather than blind-retrying a fresh invite.
const SEED_REQUEST_TIMEOUT_SECS: u64 = DIRECTORY_PULL_TIMEOUT_SECS;

/// `POST /api/daemon/seed/request` — Sprint 75 Phase E — the REQUESTER leg
/// of the authenticated `sbfb/seed/0` protocol (S74 Phase E), and the
/// first production caller of [`crate::seed_protocol::request_seed`].
///
/// "Ask a DESIGNATED peer (typically my always-on VPS anchor) to fetch,
/// pin and keep online an app whose archive THIS node holds." Loopback-
/// authenticated and fully scriptable — the headless operational model:
/// after a deploy, a script (or the future peer-designation UI) posts
/// here to hand the app to the anchor, no browser required.
///
/// Roles (do not conflate, preflight delta #4): this is the AUTHOR-side
/// REQUESTER — the voluntary community-seed path (`POST /api/daemon/seed`)
/// is the SEEDER-side unilateral act and never uses `SeedRequest`. The
/// designated peer enforces its own gates (Ed25519 + dialer cross-check +
/// nonce + ts window + the M19 invite ledger bound to
/// `(project_id, archive_hash)`); an `invite_token` minted BY THE PEER is
/// ALWAYS required — the S74 handler rejects an empty token
/// unconditionally (`"no-invite"`), there is no same-key exemption in the
/// wire protocol.
///
/// Anti-recentralization: the peer is the operator's EXPLICIT choice per
/// request — no default peer exists anywhere (verrou 3), and the archive
/// ticket is minted fresh from `my_endpoint_addr()` at request time
/// (Phase A: never a stored snapshot). The seeder ends up with the
/// author's exact BLAKE3 bytes and re-signs no provenance (verrou 4).
#[derive(Debug, serde::Deserialize)]
pub(crate) struct SeedRequestPeerRequest {
    /// Hex Ed25519 endpoint id of the designated seeder peer.
    peer_node_id: String,
    project_id: String,
    /// Invite token minted by the PEER for `(project_id, archive_hash)`.
    /// ALWAYS required by the seeder's M19 handler (an empty token is
    /// rejected `"no-invite"`); the `#[serde(default)]` is runtime
    /// tolerance only — an omitted field deserializes to empty instead of
    /// a 422, then fails the peer's gate with a clear reason.
    #[serde(default)]
    invite_token: String,
}

pub(crate) async fn seed_request_peer(
    State(state): State<Arc<DaemonHttpState>>,
    Json(req): Json<SeedRequestPeerRequest>,
) -> Response {
    debug!(project = %req.project_id, peer = %req.peer_node_id, "POST /api/daemon/seed/request");

    // Duress short-circuit BEFORE signing — never sign a SeedRequest under
    // the fake keypair (mirrors publish_project / publish_directory).
    if crate::noop_identity::gossip_publish_in_duress(state.identity_mode)
        == crate::noop_identity::PublishOutcome::Noop
    {
        return (
            StatusCode::OK,
            Json(serde_json::json!({ "requested": false })),
        )
            .into_response();
    }

    use std::str::FromStr as _;
    let Ok(peer_id) = iroh::EndpointId::from_str(&req.peer_node_id) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "malformed peer_node_id (expected an iroh endpoint id)"})),
        )
            .into_response();
    };
    // Compare PARSED identities, not raw strings: `from_str` also accepts
    // the base32 rendering of an endpoint id, which a raw string compare
    // against our hex-lowercase node_id would let through.
    if peer_id.to_string() == state.node_id {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "cannot designate this node as its own seeder"})),
        )
            .into_response();
    }

    // The app must be a local direct entry with a known archive: the
    // requester PROPOSES a source, so it must actually hold the bytes.
    let Some(entry) = state.browse_aggregator.get_direct_entry(&req.project_id) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "unknown app (not in browse)"})),
        )
            .into_response();
    };
    let Some(hash_hex) = entry.archive_hash else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "app has no archive to seed"})),
        )
            .into_response();
    };
    // Fresh ticket from my_endpoint_addr() at request time. The producer
    // helper also enforces local blob presence — a node can never ask a
    // peer to seed bytes it does not itself hold.
    let ticket = match mint_blob_ticket(&state, &hash_hex).await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({
                    "error": format!("archive blob not mintable locally: {e}")
                })),
            )
                .into_response();
        }
    };

    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let request = nexus_core_rs::seed::SeedRequest {
        version: nexus_core_rs::seed::SEED_FORMAT_VERSION,
        project_id: req.project_id.clone(),
        archive_hash: hash_hex.clone(),
        archive_ticket: ticket,
        requester_node_id: state.pow_keypair.public_bytes(),
        nonce: nexus_core_rs::seed::random_nonce(),
        ts: now,
        invite_token: req.invite_token.clone(),
    };
    let sent_nonce = request.nonce.clone();
    // The daemon signs with its node keypair — the SAME Ed25519 secret the
    // iroh endpoint boots with (runtime.rs), so the seeder's
    // `author_pubkey == conn.remote_id()` dialer cross-check holds.
    let envelope =
        match nexus_core_rs::seed::SeedRequestEnvelope::sign(request, state.pow_keypair.as_ref()) {
            Ok(env) => env,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("failed to sign seed request: {e}")})),
                )
                    .into_response();
            }
        };

    // A bare EndpointId is dialable: pkarr (presets::N0) resolves it in
    // production; tests pre-seed the node's MemoryLookup (which merges,
    // never overwrites, so the empty-addr add inside request_seed is
    // harmless).
    let peer_addr = iroh::EndpointAddr::from(peer_id);
    let resp = match tokio::time::timeout(
        std::time::Duration::from_secs(SEED_REQUEST_TIMEOUT_SECS),
        crate::seed_protocol::request_seed(
            state.node.endpoint(),
            state.node.memory_lookup(),
            peer_addr,
            &envelope,
        ),
    )
    .await
    {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({"error": format!("seed request failed: {e}")})),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::GATEWAY_TIMEOUT,
                Json(serde_json::json!({"error": "seed request timed out"})),
            )
                .into_response();
        }
    };
    // Correlation defence-in-depth on top of request_seed's signature +
    // dialed-peer checks: the signed response must echo OUR nonce, so a
    // (signed) response to some other request cannot be confused in.
    if resp.response.nonce != sent_nonce {
        return (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"error": "seed response does not echo the request nonce"})),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "accepted": resp.response.decision == nexus_core_rs::seed::SeedDecision::Accepted,
            "reason": resp.response.reason,
            "seeder_node_id": hex::encode(resp.author_pubkey),
        })),
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

    use crate::test_support::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn seed_voluntary_directory_only_app() {
        // Sprint 75 Phase D closed GAP R5b: a directory-only app (no direct
        // entry, no ticket) becomes voluntarily seedable. The anchor identity
        // here is DEAD (never booted), so the pull must fall back to the
        // SeedRegistry seeder that actually holds the bytes — the full
        // multi-provider chain, E2E through the HTTP route.
        let state = mk_state().await;
        let seeder_node = create_node().await.expect("boot seeder node");

        // The seeder holds the app archive (author bytes, content-addressed).
        let payload = b"the-author-exact-archive-bytes".to_vec();
        let blobs_seeder = nexus_core_rs::BlobsClient::new(seeder_node.blobs_store());
        let archive_hash_bytes = blobs_seeder.add_bytes(&payload).await.unwrap();
        let archive_hash = hex::encode(archive_hash_bytes);

        // A dead anchor advertises the app in its (validly signed) directory,
        // whose blob the seeder node hosts.
        let kp_anchor = KeyPair::generate();
        let pid = "e".repeat(64);
        ingest_remote_directory(
            &state,
            &seeder_node,
            &kp_anchor,
            vec![catalog_app(&pid, &archive_hash, "Fallback App")],
            1,
        )
        .await;
        // NOT a direct entry — this is the directory-only shape.
        assert!(state.browse_aggregator.get_direct_entry(&pid).is_none());

        // The live seeder announced this exact version; seed its address so
        // the fallback dial resolves without live pkarr propagation timing.
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        state
            .seed_registry
            .record(&pid, &archive_hash, &seeder_node.node_id(), now, now);
        let seeder_addr = nexus_core_rs::DiscoveryClient::new(seeder_node.endpoint())
            .my_endpoint_addr()
            .await
            .expect("seeder must expose an address");
        state.node.memory_lookup().add_endpoint_info(seeder_addr);

        // Phase F: the request pins the EXACT displayed version (the shell
        // passes the entry's archive_hash on every surface that knows it) —
        // this E2E exercises the discriminated resolution path end-to-end,
        // not the agnostic one.
        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"project_id": pid, "archive_hash": archive_hash})
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "voluntary seed of a directory-only app must succeed via the seeder fallback"
        );

        // The node now HOLDS the author bytes under the exact hash...
        let blobs_local = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
        assert!(blobs_local.has(archive_hash_bytes).await.unwrap());
        let got = blobs_local.get_bytes(archive_hash_bytes).await.unwrap();
        assert_eq!(got, payload, "content-addressing: the author's exact bytes");
        // ...pinned skip-GC under the keep-online tag — this is the ONLY test
        // exercising fetch_and_pin_multi, so the pin half of its contract
        // must be asserted here (mirror of the ticket-path test).
        assert!(
            has_tag(&state, &crate::deploy::keep_online_tag(&pid)).await,
            "fetch_and_pin_multi must leave the keep-online pin tag behind"
        );
        // ...and the keep-online row records the seed for the boot re-announce.
        // Lexical block so the MutexGuard provably never crosses the await
        // below (clippy::await_holding_lock reasons on scopes, not drop()).
        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            let row = db.get_keep_online(&pid).expect("keep_online read");
            assert_eq!(row, Some((true, Some(archive_hash.clone()))));
        }

        seeder_node.shutdown().await.ok();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn seed_voluntary_version_discriminator_local_rejects() {
        // Sprint 75 Phase F (review-D deferral closed): the optional
        // `archive_hash` on POST /api/daemon/seed pins the EXACT version.
        // Local-rejection paths only — no fetch is ever started.
        let state = mk_state().await;

        // A direct card carries version A (ready ticket).
        let version_a = "aa".repeat(32);
        let pid = "5".repeat(64);
        let mut entry = own_browse_entry(&pid, "Two Versions", None);
        entry.archive_ticket = Some("ticket-version-a".into());
        entry.archive_hash = Some(version_a.clone());
        state.browse_aggregator.add_direct_entry(entry);

        // Asking for version B (listed nowhere) must NOT silently fall back
        // to the direct card's version A — 404, version-specific message.
        let version_b = "bb".repeat(32);
        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"project_id": pid, "archive_hash": version_b})
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::NOT_FOUND,
            "a direct entry of a DIFFERENT version must not shadow the requested one"
        );
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json["error"], "no source for the requested app version",
            "the rejection names the version miss, not an unknown app"
        );

        // Pre-F behaviour preserved: an archive-less direct card with no
        // requested version (and no directory fallback) is still a 400.
        let pid_bare = "6".repeat(64);
        state
            .browse_aggregator
            .add_direct_entry(own_browse_entry(&pid_bare, "No Archive", None));
        // own_browse_entry sets a placeholder hash — strip it to model the
        // archive-less card shape.
        let mut bare = state.browse_aggregator.get_direct_entry(&pid_bare).unwrap();
        bare.archive_hash = None;
        bare.archive_ticket = None;
        state.browse_aggregator.add_direct_entry(bare);
        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"project_id": pid_bare}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "app has no archive to seed");

        // The MATCHING-version branch takes the Ticket arm (review F P2: the
        // main prod path was never selection-pinned). The ticket is malformed
        // so the fetch fails fast — 502 "could not fetch", which proves the
        // selection entered the Ticket arm instead of 404ing the version.
        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"project_id": pid, "archive_hash": version_a})
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::BAD_GATEWAY,
            "a matching requested version must select the direct ticket arm"
        );
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "could not fetch the app archive");

        // Case normalization: the SAME request with an UPPERCASE hash must
        // reach the same arm (hex-case lesson), never 404 on case alone.
        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({
                            "project_id": pid,
                            "archive_hash": version_a.to_ascii_uppercase()
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);

        // A direct card with a hash but NO ticket (restored-from-outbox shape)
        // and no directory fallback: still the pre-F 400 — and the agnostic
        // fallback is narrowed by the card's own hash, so it can never pin a
        // DIFFERENT version than the one displayed (review F P3).
        let pid_hash_only = "8".repeat(64);
        let mut hash_only = own_browse_entry(&pid_hash_only, "Hash No Ticket", None);
        hash_only.archive_ticket = None;
        hash_only.archive_hash = Some("dd".repeat(32));
        state.browse_aggregator.add_direct_entry(hash_only);
        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"project_id": pid_hash_only}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "app has no archive to seed");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn seed_count_exposes_self_pin_intent() {
        // WEB-1 (Sprint 75 Phase F): `self_pin_enabled` is the THREE-valued
        // persisted intent — null (never toggled, still diffused by default),
        // true (explicit ON), false (explicit OFF). `self_seeding` stays the
        // version-scoped serving truth and must NOT be conflated with it.
        let state = mk_state().await;
        let pid = "7".repeat(64);
        let hash = "cd".repeat(32);

        let get_count = |uri: String| {
            let state = state.clone();
            async move {
                let app = build_test_router(state);
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
                serde_json::from_slice::<serde_json::Value>(&body).unwrap()
            }
        };

        // Never toggled: intent is null, not false.
        let json = get_count(format!("/api/daemon/seed-count/{pid}")).await;
        assert_eq!(json["self_pin_enabled"], serde_json::Value::Null);
        assert_eq!(json["self_seeding"], false);

        // Explicit ON.
        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            db.set_keep_online(&pid, true, Some(&hash)).unwrap();
        }
        let json = get_count(format!("/api/daemon/seed-count/{pid}")).await;
        assert_eq!(json["self_pin_enabled"], true);
        assert_eq!(json["self_seeding"], true);
        // Intent is NOT version-scoped: a query about a DIFFERENT version
        // keeps the intent (true) while the serving truth drops to false.
        let other = "ef".repeat(32);
        let json = get_count(format!("/api/daemon/seed-count/{pid}?archive_hash={other}")).await;
        assert_eq!(json["self_pin_enabled"], true);
        assert_eq!(json["self_seeding"], false);

        // Explicit OFF.
        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            db.set_keep_online(&pid, false, Some(&hash)).unwrap();
        }
        let json = get_count(format!("/api/daemon/seed-count/{pid}")).await;
        assert_eq!(json["self_pin_enabled"], false);
        assert_eq!(json["self_seeding"], false);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn boot_seed_driver_pins_configured_projects() {
        // Plan §E.3 #1 — THE Phase E test: a headless anchor seeds an app
        // it NEVER deployed locally, purely from its operator-written
        // `[seed]` accept-list. The app resolves through a subscribed node
        // directory (whose anchor identity is dead) and the bytes come
        // from a live seeder — the same Phase D multi-provider consumer
        // chain as seed_voluntary, never a ticket re-mint.
        let state = mk_state().await;
        let seeder_node = create_node().await.expect("boot seeder node");

        let payload = b"vps-config-driven-seed-bytes".to_vec();
        let blobs_seeder = nexus_core_rs::BlobsClient::new(seeder_node.blobs_store());
        let archive_hash_bytes = blobs_seeder.add_bytes(&payload).await.unwrap();
        let archive_hash = hex::encode(archive_hash_bytes);

        let kp_anchor = KeyPair::generate();
        let pid = "f".repeat(64);
        ingest_remote_directory(
            &state,
            &seeder_node,
            &kp_anchor,
            vec![catalog_app(&pid, &archive_hash, "Configured App")],
            1,
        )
        .await;
        assert!(
            state.browse_aggregator.get_direct_entry(&pid).is_none(),
            "the configured app must NOT be a local/direct app (never deployed here)"
        );

        // A live seeder announced this exact version; pre-seed its address
        // so the dial resolves without live pkarr propagation timing.
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        state
            .seed_registry
            .record(&pid, &archive_hash, &seeder_node.node_id(), now, now);
        let seeder_addr = nexus_core_rs::DiscoveryClient::new(seeder_node.endpoint())
            .my_endpoint_addr()
            .await
            .expect("seeder must expose an address");
        state.node.memory_lookup().add_endpoint_info(seeder_addr);

        let pinned = run_boot_seed_driver(&state, std::slice::from_ref(&pid)).await;
        assert_eq!(pinned, 1, "the configured app must be acquired + pinned");

        // The anchor now HOLDS the author's exact bytes (content-addressed)...
        let blobs_local = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
        assert!(blobs_local.has(archive_hash_bytes).await.unwrap());
        assert_eq!(
            blobs_local.get_bytes(archive_hash_bytes).await.unwrap(),
            payload
        );
        // ...pinned skip-GC under the keep-online tag...
        assert!(
            has_tag(&state, &crate::deploy::keep_online_tag(&pid)).await,
            "the boot driver must leave the keep-online pin tag behind"
        );
        // ...with the keep_online row recorded for future boots.
        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            assert_eq!(
                db.get_keep_online(&pid).expect("keep_online read"),
                Some((true, Some(archive_hash.clone())))
            );
        }

        seeder_node.shutdown().await.ok();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn redrive_on_ingest_pins_configured_app_without_restart() {
        // Sprint 82 Phase A — ANCRE red→green. The one-shot boot driver runs
        // during the S75 "first-boot dead window" (directory not yet ingested)
        // and pins nothing; when a subscribed anchor's directory later ingests
        // and covers a configured keep_online app, the re-drive-on-ingest hook
        // pins it WITHOUT a daemon restart — closing the S81-G-ESC-1 boot-SEED
        // escalation. The re-drive is single-flight + dirty-coalesced (Codex
        // P1-1) and duress-gated (Codex P1-2); the chain handle is awaited for
        // determinism (prod fires and forgets).
        let state = mk_state().await;
        let seeder_node = create_node().await.expect("boot seeder node");

        let payload = b"redrive-on-ingest-seed-bytes".to_vec();
        let blobs_seeder = nexus_core_rs::BlobsClient::new(seeder_node.blobs_store());
        let archive_hash_bytes = blobs_seeder.add_bytes(&payload).await.unwrap();
        let archive_hash = hex::encode(archive_hash_bytes);
        let pid = "e".repeat(64);
        let tag = crate::deploy::keep_online_tag(&pid);

        // A live seeder announced this exact version; pre-seed its address +
        // registry so the eventual pull resolves without live pkarr timing
        // (same trick as `boot_seed_driver_pins_configured_projects`).
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        state
            .seed_registry
            .record(&pid, &archive_hash, &seeder_node.node_id(), now, now);
        let seeder_addr = nexus_core_rs::DiscoveryClient::new(seeder_node.endpoint())
            .my_endpoint_addr()
            .await
            .expect("seeder must expose an address");
        state.node.memory_lookup().add_endpoint_info(seeder_addr);

        let configured = vec![pid.clone()];
        let lock = std::sync::Arc::new(tokio::sync::Mutex::new(()));
        let coord = std::sync::Arc::new(tokio::sync::Mutex::new(
            crate::runtime::RedriveCoord::default(),
        ));
        // Short grace window so awaiting a chain does not wait out the prod
        // REDRIVE_MIN_INTERVAL (the pace is a param precisely so tests stay fast).
        let pace = std::time::Duration::from_millis(50);

        // CONTROL (red): no directory ingested yet — the re-drive chain runs one
        // pass that pins nothing (the dead window reproduced). Awaiting the chain
        // is deterministic: with nothing dirty it does one pass and exits.
        let control =
            crate::runtime::maybe_redrive_seed_on_ingest(&state, &configured, &lock, &coord, pace)
                .await
                .expect("non-empty config + non-duress starts a chain");
        control.await.expect("control chain joins");
        assert!(
            !has_tag(&state, &tag).await,
            "control: the app must NOT be pinned before its directory ingests"
        );

        // Ingest the subscribed anchor's directory covering the configured pid
        // (the effect the live gossip receive path has on the directory store).
        let kp_anchor = KeyPair::generate();
        ingest_remote_directory(
            &state,
            &seeder_node,
            &kp_anchor,
            vec![catalog_app(&pid, &archive_hash, "Configured App")],
            1,
        )
        .await;

        // FIX (green): re-drive after ingest -> the app is now resolvable and is
        // pinned WITHOUT a daemon restart.
        let fixed =
            crate::runtime::maybe_redrive_seed_on_ingest(&state, &configured, &lock, &coord, pace)
                .await
                .expect("a fresh chain starts (the previous one converged)");
        fixed.await.expect("fix chain joins");

        // The anchor now HOLDS the author's exact bytes, pinned skip-GC, with
        // the keep_online row recorded — all without a daemon restart.
        let blobs_local = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
        assert!(blobs_local.has(archive_hash_bytes).await.unwrap());
        assert!(
            has_tag(&state, &tag).await,
            "the re-drive must leave the keep-online pin tag behind"
        );
        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            assert_eq!(
                db.get_keep_online(&pid).expect("keep_online read"),
                Some((true, Some(archive_hash.clone())))
            );
        }

        // EMPTY-config revert-proof: no operator accept-list -> never a chain.
        assert!(
            crate::runtime::maybe_redrive_seed_on_ingest(&state, &[], &lock, &coord, pace)
                .await
                .is_none(),
            "empty accept-list must never re-drive"
        );

        // DURESS revert-proof (Codex P1-2): a decoy node re-drives nothing, even
        // with a resolvable configured app — gated BEFORE cloning the real list.
        let duress = mk_state_with_mode(nexus_core_rs::IdentityMode::Duress).await;
        assert!(
            crate::runtime::maybe_redrive_seed_on_ingest(&duress, &configured, &lock, &coord, pace)
                .await
                .is_none(),
            "a decoy (duress) node must never re-drive"
        );

        // COALESCING (Codex P1-1): while a chain is active (blocked on the held
        // lock), a second ingest COALESCES (returns None) and marks the chain
        // dirty; releasing the lock lets the chain run to completion (first pass
        // + one trailing pass over the coalesced ingest) — the trigger is NEVER
        // dropped (vs the old leading-edge cooldown that discarded it). The short
        // `pace` keeps the grace window fast enough to await here.
        let hold = lock.lock().await;
        let chain =
            crate::runtime::maybe_redrive_seed_on_ingest(&state, &configured, &lock, &coord, pace)
                .await
                .expect("first ingest starts a chain (blocked on the held lock)");
        let coalesced =
            crate::runtime::maybe_redrive_seed_on_ingest(&state, &configured, &lock, &coord, pace)
                .await;
        assert!(
            coalesced.is_none(),
            "a second ingest during an active chain must coalesce, not spawn a second chain"
        );
        drop(hold);
        chain
            .await
            .expect("the coalesced chain joins after its trailing pass");
        assert!(
            has_tag(&state, &tag).await,
            "the coalesced chain must leave the app pinned — the trigger was covered, not dropped"
        );

        seeder_node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn boot_repins_keep_online_blobs() {
        // Plan §E.3 #2 — re-pin, not just re-announce: a kept-online app
        // whose blob survived in the store but whose skip-GC tag is gone
        // gets its pin re-asserted at boot, with ZERO network involved
        // (the keep_online row's hash is the M18 source-of-truth).
        let state = mk_state().await;
        let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
        let payload = b"locally-held-keep-online-bytes".to_vec();
        let hash = blobs.add_bytes(&payload).await.unwrap();
        let hash_hex = hex::encode(hash);
        let pid = "9".repeat(64);
        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            db.set_keep_online(&pid, true, Some(&hash_hex)).unwrap();
        }
        let tag = crate::deploy::keep_online_tag(&pid);
        assert!(
            !has_tag(&state, &tag).await,
            "precondition: no keep-online tag before the driver runs"
        );

        // Deliberate duplicate in the configured list: the `seen` dedup
        // guarantees ONE acquisition — the counter discriminates (without
        // the guard, the idempotent set_tag would yield 2).
        let pinned = run_boot_seed_driver(&state, &[pid.clone(), pid.clone()]).await;
        assert_eq!(pinned, 1);
        assert!(
            has_tag(&state, &tag).await,
            "the driver must re-assert the skip-GC pin on locally-held bytes"
        );
    }

    #[tokio::test]
    async fn boot_seed_driver_empty_config_is_noop() {
        // Verrou 5: an empty accept-list (the compiled default, verrou 3)
        // does zero work. An unresolvable configured id is skipped loudly,
        // never fabricated into a keep_online row.
        let state = mk_state().await;
        assert_eq!(run_boot_seed_driver(&state, &[]).await, 0);

        let unknown = "8".repeat(64);
        assert_eq!(
            run_boot_seed_driver(&state, std::slice::from_ref(&unknown)).await,
            0
        );
        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            assert_eq!(
                db.get_keep_online(&unknown).unwrap(),
                None,
                "an unresolvable configured app must leave no keep_online row"
            );
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn request_seed_prod_caller() {
        // Plan §E.3 #3 (preflight delta #4 honored — REQUESTER role, not
        // the seed driver): `request_seed`'s first production caller is
        // the loopback route `POST /api/daemon/seed/request` — the author
        // asks a DESIGNATED peer (its anchor) to seed an app the author
        // holds. The peer runs the real `sbfb/seed/0` handler with its M19
        // invite ledger; the route signs with the node identity (the same
        // Ed25519 secret as the QUIC dialer, exactly the prod boot shape).
        use nexus_core_rs::node::{SEED_ALPN, create_node_with_protocols};
        use nexus_core_rs::{NodeConfig, create_node_with_config};

        // Requester state whose pow_keypair IS the node identity.
        let secret = KeyPair::generate().secret_bytes();
        let kp = KeyPair::from_secret_bytes(&secret);
        let node = create_node_with_config(NodeConfig::default().with_secret_key(secret))
            .await
            .expect("requester node");
        let mut state = (*mk_state().await).clone();
        state.node_id = node.node_id();
        state.node = Arc::new(node);
        state.pow_keypair = Arc::new(kp);
        let state = Arc::new(state);

        // The app: a local direct entry whose blob THIS node holds (the
        // route mints a fresh ticket — producer side, blob presence gated).
        let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
        let payload = b"author-app-handed-to-anchor".to_vec();
        let hash = blobs.add_bytes(&payload).await.unwrap();
        let hash_hex = hex::encode(hash);
        let pid = "6".repeat(64);
        let mut entry = own_browse_entry(&pid, "HandedApp", Some(state.node_id.clone()));
        entry.archive_hash = Some(hash_hex.clone());
        state.browse_aggregator.add_direct_entry(entry);

        // The designated seeder peer: real SeedProtocol handler + invite
        // minted for exactly (project_id, archive_hash) — M19.
        let seeder_secret = KeyPair::generate().secret_bytes();
        let seeder_kp = Arc::new(KeyPair::from_secret_bytes(&seeder_secret));
        let seeder_db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().expect("seeder db"),
        ));
        let factory = crate::seed_protocol::seed_protocol_factory(
            std::sync::Arc::clone(&seeder_db),
            Arc::clone(&seeder_kp),
            Arc::new(crate::seed_protocol::NonceCache::default()),
        );
        let seeder_node = create_node_with_protocols(
            NodeConfig::default().with_secret_key(seeder_secret),
            vec![(SEED_ALPN.to_vec(), factory)],
        )
        .await
        .expect("seeder node");
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        seeder_db
            .lock()
            .unwrap()
            .mint_seed_invite(
                "tok-prod-caller",
                &pid,
                &hash_hex,
                (now + 1000) as i64,
                Some(1),
            )
            .unwrap();
        // Tests skip live pkarr: pre-seed the requester's lookup (it
        // merges, so request_seed's empty-addr add cannot clobber it).
        let seeder_addr = nexus_core_rs::DiscoveryClient::new(seeder_node.endpoint())
            .my_endpoint_addr()
            .await
            .expect("seeder addr");
        state.node.memory_lookup().add_endpoint_info(seeder_addr);

        let body = serde_json::json!({
            "peer_node_id": seeder_node.node_id(),
            "project_id": pid,
            "invite_token": "tok-prod-caller",
        });
        let resp = build_test_router(Arc::clone(&state))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed/request")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(json["accepted"], true, "reason: {}", json["reason"]);
        assert_eq!(json["seeder_node_id"], seeder_node.node_id());

        // The designated peer now holds + keeps the author's exact bytes
        // (it re-signed no provenance — the author stays the author).
        let blobs_seeder = nexus_core_rs::BlobsClient::new(seeder_node.blobs_store());
        assert!(blobs_seeder.has(hash).await.unwrap());
        assert_eq!(blobs_seeder.get_bytes(hash).await.unwrap(), payload);
        {
            let db = seeder_db.lock().unwrap();
            assert_eq!(
                db.get_keep_online(&pid).unwrap(),
                Some((true, Some(hash_hex)))
            );
        }

        seeder_node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn boot_seed_driver_noop_in_duress() {
        // Review P1 (security): a decoy node must perform ZERO seed work —
        // no fetch, no keep_online mutation, no SeedAnnounced — even with a
        // resolvable configured list (the duress launcher shares the real
        // data root, so the driver would otherwise replay the operator's
        // real app set under the fake keypair).
        let state = mk_state_with_mode(nexus_core_rs::IdentityMode::Duress).await;
        let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
        let hash = blobs
            .add_bytes(b"duress-held-bytes".to_vec())
            .await
            .unwrap();
        let hash_hex = hex::encode(hash);
        let pid = "4".repeat(64);
        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            db.set_keep_online(&pid, true, Some(&hash_hex)).unwrap();
        }

        assert_eq!(
            run_boot_seed_driver(&state, std::slice::from_ref(&pid)).await,
            0,
            "a decoy node must perform zero seed work"
        );
        assert!(
            !has_tag(&state, &crate::deploy::keep_online_tag(&pid)).await,
            "no pin tag may appear under duress"
        );
    }

    #[test]
    fn seed_already_announced_predicate() {
        // The driver's anti-double-emission guard, as pure logic: only an
        // app ALREADY enabled with the EXACT hash being pinned was covered
        // by reannounce_seeds_at_boot — everything else must emit.
        let h = "ab".repeat(32);
        assert!(seed_already_announced(&Some((true, Some(h.clone()))), &h));
        assert!(!seed_already_announced(
            &Some((true, Some("cd".repeat(32)))),
            &h
        ));
        assert!(!seed_already_announced(&Some((false, Some(h.clone()))), &h));
        assert!(!seed_already_announced(&Some((true, None)), &h));
        assert!(!seed_already_announced(&None, &h));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn boot_driver_prefers_keep_online_hash_over_directory() {
        // Pins the resolution priority (direct > keep_online row M18 >
        // subscribed directories): an anchor advertising a DIFFERENT hash
        // for the same project id must not override the M18 row's
        // source-of-truth hash, trigger a network fetch, or rewrite the row.
        let state = mk_state().await;
        let host = create_node().await.expect("host node");

        let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
        let payload = b"version-A-bytes".to_vec();
        let hash_a = blobs.add_bytes(&payload).await.unwrap();
        let hash_a_hex = hex::encode(hash_a);
        let pid = "5".repeat(64);
        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            db.set_keep_online(&pid, true, Some(&hash_a_hex)).unwrap();
        }
        // A subscribed anchor advertises ANOTHER version of the same app.
        let kp_anchor = KeyPair::generate();
        let hash_b_hex = "bb".repeat(32);
        ingest_remote_directory(
            &state,
            &host,
            &kp_anchor,
            vec![catalog_app(&pid, &hash_b_hex, "Other Version")],
            1,
        )
        .await;

        let pinned = run_boot_seed_driver(&state, std::slice::from_ref(&pid)).await;
        assert_eq!(pinned, 1);
        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            assert_eq!(
                db.get_keep_online(&pid).unwrap(),
                Some((true, Some(hash_a_hex.clone()))),
                "the M18 row must keep hash A — never rewritten to the directory's hash"
            );
        }
        assert!(has_tag(&state, &crate::deploy::keep_online_tag(&pid)).await);
        let hash_b: [u8; 32] = hex::decode(&hash_b_hex).unwrap().try_into().unwrap();
        assert!(
            !blobs.has(hash_b).await.unwrap(),
            "the directory's other version must never be fetched"
        );

        host.shutdown().await.ok();
    }

    #[tokio::test]
    async fn seed_request_peer_noop_in_duress() {
        // Mirrors publish_directory_noop_in_duress: never sign a
        // SeedRequest under the fake keypair — short-circuit BEFORE parse,
        // mint, or dial.
        let state = mk_state_with_mode(nexus_core_rs::IdentityMode::Duress).await;
        let body = serde_json::json!({
            "peer_node_id": "ab".repeat(32),
            "project_id": "1".repeat(64),
            "invite_token": "tok",
        });
        let resp = build_test_router(state)
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed/request")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 1024).await.unwrap()).unwrap();
        assert_eq!(
            json["requested"], false,
            "duress must short-circuit before signing"
        );
    }

    #[tokio::test]
    async fn set_keep_online_noop_in_duress() {
        // Sprint 76 Phase B (B1): a decoy node must perform ZERO keep_online
        // mutation — no DB row, no blob skip-GC tag — and reply a plausible
        // benign success. The duress launcher shares the operator's REAL
        // coordinator.db + blob store, so an un-gated toggle would persist the
        // real app set under the fake keypair (local-mutation sibling of the
        // P1 wire-emit fix 23a08c9).
        let state = mk_state_with_mode(nexus_core_rs::IdentityMode::Duress).await;
        let pid = "7".repeat(64);
        // The app is visible, so a NON-duress toggle WOULD write a row + tag.
        state
            .browse_aggregator
            .add_direct_entry(own_browse_entry(&pid, "Decoy App", None));

        let resp = set_keep_online(
            State(state.clone()),
            Json(KeepOnlineRequest {
                project_id: pid.clone(),
                enabled: true,
            }),
        )
        .await
        .into_response();
        assert_eq!(resp.status(), StatusCode::OK);

        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            assert_eq!(
                db.get_keep_online(&pid).unwrap(),
                None,
                "duress must not persist a keep_online row"
            );
        }
        assert!(
            !has_tag(&state, &crate::deploy::keep_online_tag(&pid)).await,
            "duress must not tag the archive blob"
        );
    }

    #[tokio::test]
    async fn seed_voluntary_noop_in_duress() {
        // Sprint 76 Phase B (B1): a decoy node must perform ZERO voluntary-seed
        // work — no fetch, no pin, no keep_online row, no SeedAnnounced — and
        // reply a plausible benign success. The single early-return covers BOTH
        // the local pin and the emit (the local-mutation sibling of 23a08c9).
        let state = mk_state_with_mode(nexus_core_rs::IdentityMode::Duress).await;
        let pid = "8".repeat(64);
        state
            .browse_aggregator
            .add_direct_entry(own_browse_entry(&pid, "Decoy App", None));

        let resp = seed_voluntary(
            State(state.clone()),
            Json(SeedVoluntaryRequest {
                project_id: pid.clone(),
                archive_hash: None,
            }),
        )
        .await
        .into_response();
        assert_eq!(resp.status(), StatusCode::OK);

        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            assert_eq!(
                db.get_keep_online(&pid).unwrap(),
                None,
                "duress must not persist a keep_online row"
            );
        }
        assert!(
            !has_tag(&state, &crate::deploy::keep_online_tag(&pid)).await,
            "duress must not tag/pin under the fake keypair"
        );
    }

    #[test]
    fn pull_falls_back_across_tiers_when_ticket_dead() {
        // Sprint 76 Phase B (B3, PULL-3): when a direct entry carries a ticket
        // AND the subscribed directories resolve the app, the fetch chain has
        // BOTH tiers IN ORDER — ticket FIRST, directory multi-provider SECOND.
        // The handler loop tries them in order, so a dead tier-1 ticket falls
        // through to tier 2 instead of a terminal BAD_GATEWAY (pre-B3 a
        // ticket-bearing entry produced only [Ticket]).
        let chain = build_seed_fetch_chain(
            Some(("aa".repeat(32), SeedFetchPlan::Ticket("dead-ticket".into()))),
            Some(("aa".repeat(32), SeedFetchPlan::Multi(vec![]))),
        );
        assert_eq!(
            chain.len(),
            2,
            "both tiers must be present so a dead ticket can fail over to the directory"
        );
        assert!(
            matches!(chain[0].1, SeedFetchPlan::Ticket(_)),
            "the cheap ticket tier must be tried first"
        );
        assert!(
            matches!(chain[1].1, SeedFetchPlan::Multi(_)),
            "the resilient directory tier must be the fallback"
        );

        // Ticket-only (no directory hit) → single tier, unchanged.
        let only_ticket = build_seed_fetch_chain(
            Some(("bb".repeat(32), SeedFetchPlan::Ticket("t".into()))),
            None,
        );
        assert_eq!(only_ticket.len(), 1);
        assert!(matches!(only_ticket[0].1, SeedFetchPlan::Ticket(_)));

        // Directory-only (ticket-less app) → single directory tier.
        let only_dir =
            build_seed_fetch_chain(None, Some(("cc".repeat(32), SeedFetchPlan::Multi(vec![]))));
        assert_eq!(only_dir.len(), 1);
        assert!(matches!(only_dir[0].1, SeedFetchPlan::Multi(_)));

        // No tier → empty chain (the handler then returns the precise 400/404).
        assert!(build_seed_fetch_chain(None, None).is_empty());
    }

    #[tokio::test]
    async fn seed_request_peer_rejects_local_errors() {
        // The four pure-local rejections of the requester route — no
        // network, no peer: malformed id, self-designation, unknown app,
        // and the held-bytes gate (a node never proposes bytes it does not
        // hold — the producer-side mint enforces it).
        let state = mk_state().await;

        // (1) malformed peer id -> 400.
        let resp = build_test_router(Arc::clone(&state))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed/request")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"peer_node_id": "zzz", "project_id": "1".repeat(64)})
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        // (2) self-designation -> 400 (parsed-identity compare).
        let resp = build_test_router(Arc::clone(&state))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed/request")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"peer_node_id": state.node_id, "project_id": "1".repeat(64)})
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        // (3) unknown app -> 404.
        let other_peer = hex::encode(KeyPair::generate().public_bytes());
        let resp = build_test_router(Arc::clone(&state))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed/request")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"peer_node_id": other_peer, "project_id": "2".repeat(64)})
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        // (4) app whose archive blob is NOT held locally -> 409.
        let pid = "3".repeat(64);
        let mut entry = own_browse_entry(&pid, "GhostBytes", Some(state.node_id.clone()));
        entry.archive_hash = Some("ee".repeat(32));
        state.browse_aggregator.add_direct_entry(entry);
        let resp = build_test_router(Arc::clone(&state))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed/request")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"peer_node_id": other_peer, "project_id": pid})
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::CONFLICT,
            "a node must never ask a peer to seed bytes it does not itself hold"
        );
    }

    /// True iff a tag with the exact `name` exists in the blob store.
    async fn has_tag(state: &Arc<DaemonHttpState>, name: &str) -> bool {
        use futures_lite::StreamExt;
        let store = state.node.blobs_store();
        let mut stream = store
            .tags()
            .list_prefix(name.as_bytes())
            .await
            .expect("list tags");
        stream.next().await.is_some()
    }

    #[tokio::test]
    async fn voluntary_seed_distant_public_app_no_approval() {
        // Sprint 74 Phase E (amendement PO §13): a node may VOLUNTARILY keep a
        // DISTANT public app online — fetch+pin its archive + record keep_online —
        // with NO SeedRequest, NO invite, NO author approval (the content is
        // public + content-addressed). This test also covers
        // `voluntary_seeder_serves_author_provenance_intact`: the seeder ends up
        // with the AUTHOR's exact bytes (it re-signs no provenance). Real
        // frontier: a 2nd iroh node hosts the blob, the route fetches it P2P.
        use nexus_shell_daemon_core::browse::{BrowseEntry, BrowseSource, BrowseStatus};

        // A distant node hosts the public app archive and mints a ticket.
        let remote = create_node().await.expect("remote node");
        let blobs_r = nexus_core_rs::BlobsClient::new(remote.blobs_store());
        let payload = b"distant-public-app-author-signed-bytes".to_vec();
        let hash = blobs_r.add_bytes(&payload).await.unwrap();
        let r_addr = nexus_core_rs::discovery::DiscoveryClient::new(remote.endpoint())
            .my_endpoint_addr()
            .await
            .expect("remote addr");
        let ticket = iroh_blobs::ticket::BlobTicket::new(
            r_addr,
            iroh_blobs::Hash::from_bytes(hash),
            iroh_blobs::BlobFormat::Raw,
        )
        .to_string();

        // The local node (the seeder) learned the app via gossip → a direct
        // browse entry carrying the ticket + hash.
        let state = mk_state().await;
        let pid = "distant-public-app";
        state.browse_aggregator.add_direct_entry(BrowseEntry {
            project_id: pid.to_string(),
            node_id: Some(remote.node_id()),
            project_name: "Distant App".into(),
            category: "demo".into(),
            description: "a public app".into(),
            curator_pubkey: String::new(),
            curator_name: "Distant".into(),
            source: BrowseSource::Direct,
            status: BrowseStatus::Reachable,
            last_probed_at: None,
            archive_ticket: Some(ticket),
            archive_hash: Some(hex::encode(hash)),
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        });

        // Voluntary seed via the real route — the body is ONLY the project_id:
        // no invite, no token, no approval anywhere in the request.
        let body = serde_json::json!({"project_id": pid});
        let resp = build_test_router(Arc::clone(&state))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // The seeder fetched + pinned the blob and recorded keep_online — with no
        // approval step.
        let blobs_local = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
        assert!(
            blobs_local.has(hash).await.unwrap(),
            "the seeder fetched the distant blob"
        );
        assert!(
            has_tag(&state, &crate::deploy::keep_online_tag(pid)).await,
            "the seeder pinned the blob (skip-GC)"
        );
        assert_eq!(
            state
                .coordinator_db
                .lock()
                .unwrap()
                .get_keep_online(pid)
                .unwrap(),
            Some((true, Some(hex::encode(hash))))
        );
        // Provenance intact: the seeder serves the AUTHOR's exact bytes.
        assert_eq!(
            blobs_local.get_bytes(hash).await.unwrap(),
            payload,
            "the seeder serves the author's exact bytes (no re-provenance)"
        );

        remote.shutdown().await.ok();
    }

    #[tokio::test]
    async fn keep_online_off_removes_tag() {
        // OFF removes ONLY this app's per-intent keep-online tag
        // (keep-online/<project_id>); a sibling app's pin survives. Per-intent
        // keying is exactly what makes a shared archive blob safe to unpin per
        // app (preflight S3). Real frontier: real route + real blob-store tags.
        let state = mk_state().await;
        assert_eq!(
            deploy_workspace_app(&state, "Pin A", make_zip(&[("index.html", b"a")])).await,
            StatusCode::OK
        );
        assert_eq!(
            deploy_workspace_app(&state, "Pin B", make_zip(&[("index.html", b"b")])).await,
            StatusCode::OK
        );
        let pid_a = hex::encode(nexus_core_rs::crypto::blake3_hash(b"Pin A"));
        let pid_b = hex::encode(nexus_core_rs::crypto::blake3_hash(b"Pin B"));
        let tag_a = crate::deploy::keep_online_tag(&pid_a);
        let tag_b = crate::deploy::keep_online_tag(&pid_b);
        assert!(has_tag(&state, &tag_a).await, "A pinned at deploy");
        assert!(has_tag(&state, &tag_b).await, "B pinned at deploy");

        // Turn A OFF via the real route.
        let body = serde_json::json!({"project_id": pid_a, "enabled": false});
        let resp = build_test_router(Arc::clone(&state))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/keep-online")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        assert!(
            !has_tag(&state, &tag_a).await,
            "A's keep-online tag removed"
        );
        assert!(
            has_tag(&state, &tag_b).await,
            "B's pin (different intent) survives"
        );

        // Deploy wrote keep_online=true rows (review P2: pin the deploy-time DB
        // write + recorded archive_hash, not just the tag).
        {
            let db = state.coordinator_db.lock().unwrap();
            assert_eq!(
                db.get_keep_online(&pid_a).unwrap().map(|(e, _)| e),
                Some(false)
            );
            assert!(
                matches!(db.get_keep_online(&pid_b).unwrap(), Some((true, Some(_)))),
                "B still ON with a recorded archive_hash"
            );
            assert_eq!(db.list_keep_online_disabled().unwrap(), vec![pid_a.clone()]);
        }

        // Turn A back ON via the route (the "remettre en ligne" cycle) — the ON
        // arm must re-resolve archive_hash and re-pin the blob (review P2:
        // ON->re-tag path was otherwise untested).
        let on_body = serde_json::json!({"project_id": pid_a, "enabled": true});
        let resp = build_test_router(Arc::clone(&state))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/keep-online")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&on_body).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(has_tag(&state, &tag_a).await, "A re-pinned after toggle ON");
        {
            let db = state.coordinator_db.lock().unwrap();
            assert_eq!(
                db.get_keep_online(&pid_a).unwrap().map(|(e, _)| e),
                Some(true)
            );
            assert!(db.list_keep_online_disabled().unwrap().is_empty());
        }
    }
}
