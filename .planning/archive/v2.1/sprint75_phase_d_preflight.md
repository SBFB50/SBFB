# Sprint 75 Phase D Preflight

Date: 2026-06-10
HEAD: `9f7de7f`
Verdict: **PLAN-ADAPT**

## Evidence Rules
- Claim policy: every claim below cites a path, command output, URL/date, or
  explicit assumption.
- Local sources read: `prompts/agent/preflight.md` (full procedure);
  `.planning/active/sprint75_plan.md` (§Phase D D.1-D.5, dep graph, fail-fast);
  `.planning/active/sprint75_kickoff.md` (§4 verrous, §5 D1-D5, §6 R4, §10 Q5/Q7);
  `.planning/active/sprint75_phase_d_handoff.md` (§3 scope);
  `.planning/active/sprint75_phase_c_preflight.md` (DQ2 PLAN-ADAPT locator) +
  `sprint75_phase_c_review.md` (Codex R5 the 2 deferred GAPs);
  `crates/nexus-core-rs/src/blobs.rs` (1-60, 150-220 fetch_ticket / fetch_and_pin);
  `crates/nexus-core-rs/src/discovery.rs` (1-115 pkarr N0);
  `crates/nexus-core-rs/src/node.rs` (120-205, boot preset);
  `crates/nexus-core-rs/src/node_directory.rs` (122-147 CatalogApp);
  `crates/nexus-shell-daemon/src/seed_registry.rs` (full, 1-385);
  `crates/nexus-shell-daemon-core/src/browse.rs` (100-256 BrowseEntry/Source/Status,
  560-826 aggregate + 3rd directory loop + SCOPE comment, find_archive_ticket_by_hash);
  `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` (239-293 AnchorLocator,
  1041-1133 repull_one_directory);
  `crates/nexus-shell-daemon/src/http.rs` (180-339 state+router, 1405-1574
  seed_voluntary + seed_count, 1860-1976 blob_serve + mint helpers);
  `crates/nexus-shell-daemon/src/runtime.rs` (1855-2010 mint_ticket_for_hash +
  remint_and_wrap_for_replay);
  `docs/security/THREAT_MODEL.md` (§15 seed cross-node surface);
  memory `MEMORY.md`, `nexus_grid_pivot.md` (Tip), `feedback_approach.md`,
  `feedback_context7_systematic.md`.
- Commands run:
  - `git rev-parse --short HEAD` -> `9f7de7f`; `git status -sb` -> `ahead 10`,
    working tree clean.
  - Cargo.lock resolved versions: `iroh 0.98.2`, `iroh-base 0.98.0`,
    `iroh-blobs 0.100.0`, `iroh-docs 0.98.0`, `iroh-gossip 0.98.0` (matches the
    CLAUDE.md pin). No `Cargo.toml` edit in Phase D scope.
  - iroh-blobs 0.100.0 source on disk
    (`~/.cargo/.../iroh-blobs-0.100.0/src/api/downloader.rs`): `Downloader::download`
    sig (`362-373`), `ContentDiscovery` trait (`501`), blanket impl
    `ContentDiscovery for C where C: IntoIterator<Item=I>, I: Into<EndpointId>`
    (`505-516`), `Shuffled` (`518-540`).
  - context7 `/n0-computer/iroh-blobs` (2026-06-10): "Downloader manages parallel,
    resumable downloads from multiple peers ... retrying failed providers";
    `download(HashAndFormat, Shuffled::new(vec![addr.id]))`.
  - grep: `find_archive_ticket_by_hash` / `get_direct_entry` operate on
    `direct_entries` only (`browse.rs:622`, `http.rs:1413`); no `/api/daemon/nodes`
    route exists (`http.rs` grep = no match); `presets::N0` at boot (`node.rs:306`).
  - grep `seeders_recent` -> `#[cfg(test)]` (`seed_registry.rs:200`), one prod-ish
    caller already inside a `#[cfg(test)]` feed_sync test (`feed_sync.rs:1037`).

## Scope
- Plan source: `.planning/active/sprint75_plan.md` §Phase D (D.1-D.5), lines 179-208.
- Target files:
  - `crates/nexus-core-rs/src/blobs.rs` — NEW `fetch_hash_multi` (multi-provider
    download by hash from a `Vec<EndpointId>`); reuse the `Downloader` already wired
    in `fetch_ticket` (`:186-190`).
  - `crates/nexus-shell-daemon/src/seed_registry.rs` — promote `seeders_recent` to
    prod (drop `#[cfg(test)]`); SEED-1 clamp `seen_at`; SEED-2 cap bucket count.
  - `crates/nexus-shell-daemon-core/src/browse.rs` — Q7 honest
    "reachable-via-seeder" status surface; helper to resolve a directory-only
    `(node_id, archive_hash)` for the daemon pull path.
  - `crates/nexus-shell-daemon/src/http.rs` — `blob_serve` directory-only render
    (GAP R5a) + `seed_voluntary` directory-only seed (GAP R5b) via `fetch_hash_multi`;
    NEW `GET /api/daemon/nodes` (node identity exposure).
  - `crates/nexus-shell-daemon-core/src/browse.rs` SCOPE comment (`:751-763`):
    retire/adjust the Phase-C deferral note now that D pulls.
- Deps/APIs/specs: NO new dependency, NO bump. `iroh-blobs 0.100.0` `Downloader`
  is already a workspace dep used at `blobs.rs:186`. `iroh::EndpointId` already
  used (`discovery.rs:30`).
- Security/protocol surfaces: multi-provider fetch (must preserve BLAKE3 = truth
  of reachability, verrou 4 / THREAT_MODEL §15 invariant `seeder != auteur`);
  SeedRegistry (best-effort, over-count residual row D = M); a NEW read-only
  `/api/daemon/nodes` route surface; the `BrowseEntry.node_id` `#[serde(skip)]`
  decision (`browse.rs:204`).
- Tests expected (plan D.3): `fetch_falls_back_to_seeder_when_anchor_offline`,
  `fetch_provider_ordering`, `seed_registry_clamps_future_ts`,
  `seed_registry_size_bounded`, `reachable_via_seeder_status`,
  `nodes_endpoint_groups_by_node_id`.

## S1a OSS Prior Art
- Domain: multi-provider content fetch with fallback in a content-addressed P2P
  network (download a known hash from an ordered/shuffled set of providers, retry
  failures, integrity-check by hash).
- Sources (accessed 2026-06-10):
  - **iroh-blobs 0.100.0** `Downloader` (context7 `/n0-computer/iroh-blobs` +
    on-disk source `src/api/downloader.rs`): `pub fn download(&self, request: impl
    SupportedRequest, providers: impl ContentDiscovery) -> DownloadProgress`
    (`:362-373`). `ContentDiscovery::find_providers(hash) -> Stream<EndpointId>`
    (`:501`). A **blanket impl** covers `C: Debug + Clone + IntoIterator<Item=I>,
    I: Into<EndpointId>` (`:505-516`) — this is why the current
    `download(hash, vec![endpoint_id])` (`blobs.rs:188`) compiles: a `Vec<EndpointId>`
    IS a `ContentDiscovery` and yields its providers **in iteration order**.
    `Shuffled::new(vec![...])` (`:518-540`) is the randomizing variant. context7
    confirms the downloader "manages parallel, resumable downloads from multiple
    peers ... retrying failed providers". So passing a multi-element `Vec` is the
    native, supported multi-provider path — no new API, no new dep.
  - **IPFS Bitswap / delegated routing** (kickoff §0,
    `docs.ipfs.tech/concepts/ipni`): a fetcher discovers multiple providers for a
    CID and races/falls-back across them; the CID (hash) is the integrity gate —
    a provider can only serve the exact content or be dropped. Same shape as the
    plan.
  - **BitTorrent multi-peer** (kickoff §0): a leecher pulls pieces from many peers,
    verifies each piece against the torrent's piece hashes; a lying peer's piece is
    rejected and the peer dropped. Same integrity-by-hash invariant.
- Finding: **APPROACH-ALIGNED** for the multi-provider `download()` itself, the
  SeedRegistry bornes (SEED-1/SEED-2), and the `/api/daemon/nodes` read surface —
  all match mature OSS practice and the iroh-blobs API as written.
- **APPROACH-NAIVE (narrow, blocking-class -> PLAN-ADAPT)** on ONE sub-claim of the
  plan/handoff: the plan repeatedly says a puller "RE-MINTS a ticket from
  `(node_id, archive_hash)`" using the Phase-A helper `mint_ticket_for_hash`
  (handoff:108-111, plan D.2, kickoff D5, the SCOPE comment `browse.rs:755`).
  `mint_ticket_for_hash` (`runtime.rs:1868-1882`) **bails if the blob is NOT held
  locally** (`if !blobs.has(...) anyhow::bail!`) — by design, it mints a ticket FOR
  A BLOB THE NODE ALREADY HOLDS (the producer/re-announce case, Phase A). A puller
  fetching a directory-only app does NOT yet hold the archive blob, so
  `mint_ticket_for_hash` cannot be the consumer-side fetch primitive; it is the
  wrong tool for this leg. The correct prior-art shape (iroh-blobs `Downloader`,
  IPFS, BitTorrent) is: **fetch the known hash directly from a provider set**
  (`download(HashAndFormat::raw(hash), providers)`), which needs NO pre-existing
  ticket — pkarr (`presets::N0`, `node.rs:306`, `discovery.rs:4-9`) resolves a bare
  `EndpointId` to a dialable address. The "re-mint a ticket then fetch_ticket"
  framing is an unnecessary (and impossible-for-an-absent-blob) detour. See
  `## Plan Adaptation`.
- Impact: multi-provider, SEED bornes, `/nodes` proceed as planned. The R5a/R5b
  directory-only render+seed legs implement the fetch as a **direct
  `download(hash, providers)`** (no ticket re-mint), then pin/serve — the
  evidence-backed corrected approach.

## S1b Dependencies, CVEs, Release Notes
- Scanned: no `Cargo.toml` add/bump in Phase D scope (plan §D.2 lists only
  crate-internal Rust files). `iroh-blobs 0.100.0` `Downloader` already a dep;
  `iroh::EndpointId` already imported.
- Commands/sources: Cargo.lock resolved `iroh-blobs = 0.100.0`, `iroh = 0.98.2` —
  exactly the CLAUDE.md pin. The P2-PREFLIGHT-TRANSITIVE-DEPTH gate (lock +
  `cargo tree -d`) is **N/A**: Phase D adds and bumps NOTHING, so no new
  transitive major-version collision can be introduced (contrast S72 Phase C/D
  `ollama-rs -> schemars 1.2`, which was a bump; Phase D bumps nothing). No
  advisory surface change because the dependency set is unchanged.
- Finding: **clean** (no dependency surface change).

## S2 Historical Decisions
- Commands: `git log --oneline -- crates/nexus-core-rs/src/blobs.rs` ->
  single-endpoint `fetch_ticket` originates Sprint 2 (`ed2ea76`), last touched
  S74 Phase E (`b76a084`); `git log -- seed_registry.rs` -> created S74 Phase F
  (`66a9409`), re-keyed S75 Phase C (`821aa8c`); reverse-commit check on D5 /
  PULL-2 / SEED-1/2 / Q5 / Q7.
- Decisions crossed:
  - **D5 / PULL-2 multi-provider (kickoff:235-246)**: "Fetch multi-provider
    IN-SCOPE ... plumber les `seeder_node_id` de `SeedRegistry` dans le vecteur
    providers de `download()`." This is a forward decision FOR Phase D, not against
    it. `fetch_ticket` single-endpoint (`blobs.rs:170-193`) is the Sprint-2 original
    impl, never a decision rejecting multi-provider. **No reversion**; Phase D is the
    planned realization.
  - **D2 / FIX-A re-mint (`479a87c`)**: `mint_ticket_for_hash` (`runtime.rs:1868`)
    re-mints a ticket for a HELD blob; `MAX_PROOF_AGE_SECS=1800` unchanged
    (confirmed `pow.rs`). Phase D does NOT touch this; it consumes the producer
    re-mint as-is for OWN apps and uses a separate consumer fetch for directory-only
    apps (see S1a). **No reversion.**
  - **D4 / locator-not-content (`821aa8c`, kickoff:224-234)**: directory ENTRIES
    are RAM-only, re-pulled at boot. Phase D's directory-only render/seed fetches
    the APP ARCHIVE blob (a separate content-addressed object) into the local store
    on demand; this is the F-Droid "download an app you discovered" step, NOT
    durably persisting the catalog. **No reversion** — orthogonal to D4.
  - **Codex R5 deferral (Phase C review §Reconciliation round 5)**: directory-only
    apps "visibles mais pas rendables/seedables" was explicitly scoped to Phase D
    (pull re-mint) + Phase F (front action), with a SCOPE comment landed at
    `browse.rs:751-763`. Phase D is the planned closure of exactly that deferral.
    **No reversion**; the deferral marker is consumed as designed.
  - **`BrowseEntry.node_id` `#[serde(skip)]` (`browse.rs:204`, S11/S75-C)**: the
    field never crosses daemon->frontend, keeping `/browse` JSON byte-identical. The
    plan offers two node-identity options: un-skip (changes `/browse` bytes) OR a new
    `/api/daemon/nodes` route. Un-skipping would REVERSE the deliberate
    byte-identical decision; a new route is purely additive and preserves it. See S4
    + Plan Adaptation for the recommended choice (new route, NOT un-skip).
  - **SeedRegistry "deliberately EPHEMERAL / best-effort" (`seed_registry.rs:2-13`,
    THREAT_MODEL §15 row D)**: the registry over-states but never lies (content-
    addressing is the truth). SEED-1 (clamp future-ts) and SEED-2 (cap size)
    TIGHTEN this posture; they do not reverse it. **No reversion.**
- Finding: **clean** (no DESIGN-CONFLICT). Every crossed decision is a
  forward-planned continuation. The single nuance — node_id un-skip would reverse
  the byte-identical `/browse` decision — is avoided by choosing the additive
  `/api/daemon/nodes` route (the plan's own first option).

## S3 Local Patterns And Threat Model
- Full scan performed (Phase D adds a NEW network-fetch path from an UNTRUSTED
  provider set + a NEW HTTP read surface = new security surface, per preflight
  Step 4 escalation). Threat mapping of the Phase D multi-provider pull primitive,
  anchored to THREAT_MODEL §15:
  - **Assets**: the local blob store (a fetched archive becomes renderable +
    seedable); the SeedRegistry availability count; the new `/nodes` projection.
  - **Actors**: a malicious seeder advertised in `SeedRegistry` (forged
    `SeedAnnounced`); a Sybil swarm padding the provider `Vec`; an anchor serving a
    substituted archive; a future-ts spammer.
  - **Vectors + mitigations**:
    - **Provider serves wrong bytes (R5 re-attribution, §15 row I = Nil)** ->
      `Downloader.download(HashAndFormat::raw(hash), providers)` is the integrity
      gate: the requested object IS the BLAKE3 hash, so a provider can only serve
      the exact content or fail, and the downloader **retries the next provider**
      (context7: "retrying failed providers"; source `downloader.rs` find_providers
      stream). The render/seed callers ALREADY re-check the returned hash:
      `seed_voluntary` rejects `Ok(_) if h != want_hash` (`http.rs:1448,1481`),
      blob-serve reads back by the same `hash_bytes` (`http.rs:1906`). **A
      malicious seeder in the `Vec` can ONLY waste a connection attempt, never
      poison the store** — the verrou-4 / §15 invariant `seeder != auteur` holds
      STRUCTURALLY through the multi-provider path. (Must keep the explicit
      hash re-check on the new directory-only paths — a regression test asserts it.)
    - **Sybil padding of the provider `Vec` (§15 row D = M)** -> the `Vec` is built
      from (a) the ONE anchor node_id of the subscribed directory + (b) the
      `SeedRegistry` seeders, which are already gated by `record_announced`
      (`seeder_node_id == author_pubkey`, self-exclusion, `seed_registry.rs:120-141`)
      and bounded by TTL + sweep. **SEED-2 (cap bucket count)** ADDS a hard upper
      bound on registry size; the provider `Vec` must be capped (a small N, e.g.
      reuse the registry's per-key seeder set which is already TTL-bounded) so an
      attacker cannot make the downloader attempt thousands of dials. This is a
      tightening, not a regression. Row D stays M (best-effort, pilote-ferme,
      content-addressing = truth) and is not worsened.
    - **Future-ts seeder (§15 row D)** -> `record(seen_at)` (`seed_registry.rs:87`)
      currently accepts a raw `seen_at`; a hostile `SeedAnnounced` with a far-future
      ts would evade the TTL purge (stay "fresh" forever). **SEED-1 clamp
      `seen_at = min(feed_ts, recv_clock)`** closes this — a genuine additive
      hardening of an existing best-effort surface (not a covered-threat regression;
      §15 row D already accepts over-count, SEED-1 reduces it).
    - **`/api/daemon/nodes` read surface** -> behind the same loopback bearer +
      Host + Origin gate as every authenticated route (`http.rs:274` `authed_routes`).
      Read-only projection of `directory_snapshot()` (already-verified entries). No
      new write surface, no new trust boundary.
  - **Q7 honest status (verrou 4 + §15 "la sonde ETAT est l'autorite")**: today a
    directory app whose ANCHOR is down is marked `Unreachable` (`browse.rs:766-787`
    probes the anchor node_id), even when a seeder holds the BLAKE3. Surfacing a
    "reachable-via-seeder" signal is an HONESTY improvement (never claim
    `Reachable` on a dead anchor; never claim availability without a content-holder).
    **Crate-boundary constraint (load-bearing for the design)**: `BrowseAggregator`
    lives in `nexus-shell-daemon-core` and has NO access to `SeedRegistry` (which
    lives in `nexus-shell-daemon`, `http.rs:193` `DaemonHttpState.seed_registry`).
    So the seeder-aware status CANNOT be computed inside `aggregate()` without a new
    dependency injection. See Plan Adaptation for the resolved seam.
  - **Anti-Sybil triad (kickoff:151-154)**: leg 1 (Ed25519) + leg 3 (subscription)
    present from B/C; leg 2 (kudos threshold) remains satisfied-by-subscription for
    S75 (documented scope cut §9.10). Phase D does not regress this — the provider
    set is derived from a SUBSCRIBED anchor + signature-gated seeders.
  - **Verrou 5 / confidentiality default-OFF**: a multi-provider fetch only fires
    on an EXPLICIT user action (open/render or voluntary-seed), never a silent boot
    fetch. Phase D adds no new boot network call (the boot re-pull is Phase C, the
    PRODUCER re-announce is Phase E). Preserved.
- HARDENING_ROADMAP status: no S75/Phase-D pre-requirement is listed as a blocker;
  the §15 seed surface hardening is already shipped (S74). SEED-1/SEED-2 are
  improvements ON §15 row D/row "registre", not gated pre-requirements.
- Finding: **clean / non-blocking**. No regression of a covered T0-T5 / §15 threat.
  The multi-provider path preserves content-addressing structurally; SEED-1/SEED-2
  tighten an existing best-effort surface; `/nodes` is gated read-only. One
  documented non-blocking carry: the provider `Vec` must be capped (SEED-2 informs
  it) and the explicit post-fetch hash re-check must be kept on the new
  directory-only render/seed legs (regression test).

## S4 Protocol And Wire Invariants
- Wire/security files checked: `blobs.rs` (transport, NOT wire schema);
  `seed_registry.rs` (in-memory, NOT wire — and NOT persisted, `:2-13`);
  `browse.rs` (`BrowseEntry`/`BrowseSource`/`BrowseStatus` serde);
  `node_directory.rs` (`CatalogApp`, `NODE_DIRECTORY_FORMAT_VERSION=1`,
  `DOMAIN_NODE_DIRECTORY_V1` — Phase D adds NO new domain, bumps nothing);
  `http.rs` (`/browse` JSON, new `/nodes` route).
- VERSION/domain/canonical status: Phase D bumps NO `*_FORMAT_VERSION` /
  `*_ANNOUNCEMENT_VERSION` / `*_SCHEMA_VERSION`; adds NO `DOMAIN_*`. `MAX_PROOF_AGE_SECS=1800`
  untouched. iroh pins unchanged.
- Day 0 status: **preserved** (D1-D5, verrous 1-5; see S2).
- Producer -> consumer trace (P2-PREFLIGHT-WIRE-CONTRACT-DEPTH) for every touched
  wire/serialized field:
  - **`/browse` JSON (`BrowseEntry`)**: Phase D's Q7 status writes into the EXISTING
    `BrowseEntry.status` enum. If a "reachable-via-seeder" signal is needed, the
    SAFE additive shape is a NEW `BrowseStatus` variant (e.g.
    `ReachableViaSeeder`) on the `#[serde(rename_all="lowercase")]` enum
    (`browse.rs:146-157`). TRACE: PRODUCER = `aggregate()` / the daemon `/browse`
    handler; CONSUMER = the frontend Zod `BrowseStatusSchema` + the React shell's
    status rendering. **A new enum variant IS a wire change to `/browse`**: an old
    frontend Zod `.enum(["reachable","unreachable","unknown"])` would REJECT a
    `"reachableviaseeder"` value (Zod enum is closed, not additive-by-default unlike
    a `#[serde(default)]` struct field). LESSON S73-E: an enum/shape the frontend
    does not expect breaks the consumer. **Therefore**: either (a) add the variant
    AND update the frontend Zod + rendering in the SAME phase with a full web
    fail-fast (this makes Phase D touch `web/`, which the plan's commit subject
    `feat(core+daemon)` does not anticipate and the handoff scopes the front to
    Phase F), OR (b) keep `/browse` byte-identical in D and defer the VISIBLE
    seeder-status to Phase F, surfacing seeder availability in D only through the
    existing per-app `seed-count` route + the new `/nodes` route. See Plan
    Adaptation: **recommend (b)** to honor the C/D/F phase boundary and keep D
    backend-only.
  - **`BrowseEntry.node_id` `#[serde(skip)]` (`browse.rs:199-205`)**: un-skipping
    it to expose node identity WOULD change `/browse` bytes and require a frontend
    Zod `.optional()` add + web fail-fast (a wire change the `feat(core+daemon)`
    subject hides). TRACE: PRODUCER = aggregator sets `node_id` server-side
    (`browse.rs:777`); CONSUMER = today NOTHING reads it across the boundary (it is
    skipped). **Recommend NOT un-skipping**; expose node identity through a NEW
    additive `GET /api/daemon/nodes` route (reads `directory_snapshot()`), which
    introduces its OWN response schema (a fresh Zod schema in Phase F, no existing
    consumer to break). This keeps `/browse` byte-identical (S2 decision preserved)
    and is purely additive.
  - **`/api/daemon/nodes` (NEW route)**: PRODUCER = a new handler serializing
    `directory_snapshot()` grouped by node_id (each `NodeDirectoryEntry` already
    carries `node_id` + `catalog`). CONSUMER = a new Phase-F Zod schema (no existing
    consumer). Confirm the response is an ENVELOPE (e.g.
    `{ nodes: [{ node_id, catalog: [...] }] }`) not a bare array, matching the S73-E
    lesson (the daemon search route uses an envelope `{results,total,took_ms}`).
    The shape is consumed only in Phase F, so D must pick a stable, documented
    envelope and pin it with a Rust serde round-trip test
    (`nodes_endpoint_groups_by_node_id`).
  - **`SeedRegistry` (SEED-1/SEED-2)**: NOT wire, NOT persisted (`:2-13`). SEED-1
    changes the `seen_at` STORED (clamp) — pure in-memory. SEED-2 caps bucket count
    — pure in-memory. The ON-WIRE `SeedAnnouncedPayload` (`public_feed.rs`,
    `project_id`/`seeder_node_id`/`archive_hash`) is UNCHANGED; Phase D reads the
    same fields. **Zero wire impact.**
  - **`fetch_hash_multi` (NEW, `blobs.rs`)**: a transport helper, NOT a wire schema.
    It calls `Downloader.download(HashAndFormat::raw(hash), providers_vec)` — the
    SAME `Downloader` API `fetch_ticket` uses (`:186-190`), just with a multi-element
    ordered `Vec<EndpointId>` instead of a single one (blanket `ContentDiscovery`
    impl, source `downloader.rs:505-516`). No serialization change.
- Finding: **clean**. No `*_VERSION` bump, no new domain, no tolerant
  multi-version decoder. The ONLY wire-contract risk is the node-identity / Q7
  status exposure: handled by choosing the additive `/api/daemon/nodes` route and
  deferring the VISIBLE seeder-status to Phase F (keeping `/browse` byte-identical),
  per Plan Adaptation. No Day-0 contradiction.

## Plan Adaptation
- **Original plan** (`.planning/active/sprint75_plan.md` §D.2, handoff:108-111,
  kickoff D5, SCOPE comment `browse.rs:755`): a puller makes a directory-only app
  renderable/seedable by "RE-MINTING a ticket from `(node_id, archive_hash)`" using
  the Phase-A helper `mint_ticket_for_hash`, then fetching via the ticket.
- **Evidence requiring adaptation (S1a APPROACH-NAIVE)**:
  `mint_ticket_for_hash` (`runtime.rs:1868-1882`) **bails if `!blobs.has(hash)`** —
  it mints a ticket only for a blob the node ALREADY HOLDS (the producer/re-announce
  case, Phase A by design). A puller of a directory-only app does NOT yet hold the
  archive, so this helper cannot drive the consumer fetch. The mature shape
  (iroh-blobs `Downloader.download(HashAndFormat::raw(hash), providers)`, source
  `downloader.rs:362-373` + context7 2026-06-10; IPFS/BitTorrent) fetches a known
  hash DIRECTLY from a provider set with NO pre-existing ticket — pkarr
  (`presets::N0`, `node.rs:306`; `discovery.rs:4-9`) resolves a bare `EndpointId`
  to a dialable address.
- **Corrected approach** (drop the "re-mint then fetch_ticket" detour for the
  consumer leg; keep `mint_ticket_for_hash` only for the producer/re-announce path
  it was built for):
  1. `blobs.rs`: add
     `pub async fn fetch_hash_multi(&self, endpoint: &Endpoint, lookup: &MemoryLookup,
     hash: [u8;32], providers: Vec<EndpointId>) -> Result<[u8;32]>` that calls
     `Downloader::new(self.inner, endpoint).download(HashAndFormat::raw(Hash::from_bytes(hash)),
     providers)` (the ordered `Vec` is a `ContentDiscovery` via the blanket impl;
     order = Q5 = anchor first, then seeders). Pkarr resolves each `EndpointId`;
     `lookup` is seeded opportunistically if a ticket address is known (none for
     directory-only — pkarr suffices). Return the verified hash. A `fetch_and_pin_multi`
     variant composes `+ set_tag` for the seed path (mirrors `fetch_and_pin`,
     `blobs.rs:208-220`).
  2. `http.rs` `blob_serve` (GAP R5a): after the existing
     `find_archive_ticket_by_hash` tier fails (`:1892`), add a tier that resolves
     the hash against `directory_snapshot()` — find the anchor `node_id` whose
     `catalog` contains a `CatalogApp.archive_hash == hash` — builds
     `providers = [anchor_node_id] + seed_registry.seeders_recent(project_id, hash, now)`
     (parsed to `EndpointId`), calls `fetch_hash_multi`, then reads back by `hash`
     (KEEP the existing post-fetch read-by-hash integrity check, `:1906`).
  3. `http.rs` `seed_voluntary` (GAP R5b): when `get_direct_entry` is `None`
     (`:1414`), fall back to the same directory resolution + `fetch_and_pin_multi`,
     then the existing `Ok(h) if h == want_hash` guard (`:1448`) + keep_online +
     `SeedAnnounced` emit (unchanged downstream).
  4. `seed_registry.rs`: drop `#[cfg(test)]` from `seeders_recent` (`:200`) so the
     prod provider-vector builder can read it; SEED-1 clamp `seen_at` in `record`
     (`:87`, `min(seen_at, recv_clock)` where `recv_clock` is the caller's `now`);
     SEED-2 cap the resident bucket count (e.g. a `MAX_REGISTRY_BUCKETS` evicting
     the oldest-`last_seen` bucket on insert overflow — additive to the existing
     TTL sweep).
  5. Node identity (`/api/daemon/nodes`): NEW additive route reading
     `directory_snapshot()`, grouped by node_id, envelope-shaped. Do NOT un-skip
     `BrowseEntry.node_id` (preserves the byte-identical `/browse` decision, S2/S4).
  6. Q7 status: keep `/browse` byte-identical in D. Surface seeder availability in D
     ONLY via the existing per-app `seed-count` route + the new `/nodes` route. The
     VISIBLE "reachable-via-seeder" badge (a new `BrowseStatus` variant + frontend
     Zod + rendering) is deferred to Phase F with the rest of the front work,
     because adding the variant in D would force a `/browse` wire change + a `web/`
     fail-fast that the `feat(core+daemon)` subject and the C/D/F phase boundary do
     not anticipate. **If the PO wants the visible badge in D**, that is a scope
     expansion into `web/` (commit subject + fail-fast change) — flag, do not assume.
     Adjust the test `reachable_via_seeder_status` to assert the BACKEND signal
     (e.g. the daemon can report a seeder count > 0 for an app whose anchor probe is
     `Unreachable`) rather than a new `/browse` enum value.
  7. Retire/adjust the SCOPE comment (`browse.rs:751-763`): it currently says
     directory rows are "DISCOVERABLE-but-not-yet-pulled ... only act on it once
     Phase D has pulled it". Update to: D now resolves directory-only render/seed via
     `fetch_hash_multi` against `(anchor node_id + seeders)`; the ticket-re-mint
     phrasing is replaced by direct hash download.
- **File/test delta vs the original plan**:
  - `fetch_falls_back_to_seeder_when_anchor_offline` (D.3 #1): a 2-provider
    `fetch_hash_multi` test where the first provider (anchor) is unreachable and the
    second (seeder) serves the blob -> download succeeds. Asserts the fallback, the
    load-bearing test.
  - `fetch_provider_ordering` (D.3 #2, Q5): assert the provider `Vec` is built
    anchor-first then seeders (ordered, NOT `Shuffled`) — a unit test on the vector
    builder (the actual race is internal to iroh; we test OUR ordering input).
  - `seed_registry_clamps_future_ts` (D.3 #3, SEED-1) + `seed_registry_size_bounded`
    (D.3 #4, SEED-2): unit tests in `seed_registry.rs`.
  - `reachable_via_seeder_status` (D.3 #5, Q7): assert the BACKEND seeder-count
    signal for an anchor-unreachable directory app (per item 6), NOT a `/browse`
    enum change.
  - `nodes_endpoint_groups_by_node_id` (D.3 #6): serde round-trip + grouping shape
    of the new `/api/daemon/nodes` envelope.
  - ADD a regression test that a malicious provider serving wrong bytes is rejected
    by the post-fetch hash check on the directory-only render/seed legs (S3
    integrity invariant).
- **Commit body must document**: "Plan D.2 proposed re-minting a ticket from
  `(node_id, archive_hash)` for the consumer pull; preflight S1a found
  `mint_ticket_for_hash` bails on a non-held blob (it is the producer helper), and
  iroh-blobs `Downloader.download(HashAndFormat::raw(hash), providers)` fetches a
  known hash directly with no ticket (pkarr resolves the bare EndpointId) — adapted
  to a direct multi-provider hash download (`fetch_hash_multi`). Node identity
  exposed via additive `GET /api/daemon/nodes` (NOT un-skipping `BrowseEntry.node_id`,
  preserving byte-identical `/browse`); visible reachable-via-seeder badge deferred
  to Phase F to keep `/browse` byte-identical, surfaced in D via the backend
  seed-count + /nodes signals."
- Plan file remains unchanged (snapshot); the deviation is traced here + in the
  commit body only (preflight Plan Adaptation rule 5).

## Risks And Scope Cuts
- Blocking risks: **none**. The only blocking-class finding is the S1a
  APPROACH-NAIVE on the ticket-re-mint detour, which maps to PLAN-ADAPT (direct
  hash download). The corrected approach is fully code-grounded and implementable.
- Non-blocking risks / carry-over:
  - **Provider `Vec` cap**: bound the providers passed to `fetch_hash_multi` (a
    small N from the TTL-bounded `seeders_recent` + the one anchor) so a Sybil
    swarm cannot make the downloader attempt unbounded dials. SEED-2 informs the
    bound; document it.
  - **Post-fetch hash re-check kept on the new legs**: blob-serve reads back by
    `hash_bytes` (`http.rs:1906`) and seed_voluntary guards `h == want_hash`
    (`:1448`) — these MUST be preserved on the directory-only paths (regression
    test). The downloader already integrity-checks by hash, so this is
    defense-in-depth, but it is the verrou-4 / §15 invariant — keep it explicit.
  - **Q7 visible badge deferred to Phase F**: D delivers the BACKEND honest signal
    (seed-count + /nodes), the visible "reachable-via-seeder" UI is Phase F (avoids
    a `/browse` wire change in a `core+daemon` phase). Flag for PO if a visible
    badge in D is wanted (scope expansion into `web/`).
  - **Crate boundary**: `BrowseAggregator` (core) cannot see `SeedRegistry`
    (daemon); the seeder-aware status/availability is computed in the daemon HTTP
    layer (which holds both), not inside `aggregate()`. Documented design seam.
  - **`/api/daemon/nodes` envelope**: pick + pin a stable envelope shape now (serde
    round-trip test) even though the Phase-F consumer is not written yet — avoids
    the S72-D "no producer / late add" and S73-E "bare array vs envelope" traps.
- Scope cuts still honored (kickoff §9 / plan §7): SearchManifest DEFER (#1) — D
  fetches an app blob, not a coverage digest; GC reaper deferred (#3) — the
  directory-only render fetch is NOT pinned skip-GC unless it goes through
  `fetch_and_pin` (seed path), matching the C carry "pin required when a GC exists";
  multi-anchor advanced UX deferred (#11); 0-bump wire (#7) — D bumps nothing; front
  node-Browse deferred to Phase F (#F boundary); VPS headless deferred to Phase E.

## Action
- **PLAN-ADAPT**: proceed with the corrected directory-only pull approach (direct
  `fetch_hash_multi(hash, ordered providers)` via the iroh-blobs `Downloader`
  blanket `ContentDiscovery` impl, pkarr-resolved EndpointIds; NO ticket re-mint for
  the consumer leg). Multi-provider `download()`, SEED-1/SEED-2 bornes, and the
  additive `GET /api/daemon/nodes` route are implementable as planned. Node identity
  via the new route (NOT un-skipping `BrowseEntry.node_id`); the visible
  reachable-via-seeder badge deferred to Phase F (D delivers the backend signal),
  keeping `/browse` byte-identical. The commit body MUST cite this file and document
  the D.2 deviation. C/D/F phase boundary preserved; 0 wire bump; verrous 1-5 held;
  THREAT_MODEL §15 invariant `seeder != auteur` preserved structurally by
  content-addressing through the multi-provider fetch.
