# Sprint 75 Phase F Preflight

Date: 2026-06-10
HEAD: `491b3c8`
Verdict: **PLAN-ADAPT**

## Evidence Rules
- Claim policy: every claim below cites a path, command output, URL/date, or
  explicit assumption.
- Local sources read: `prompts/agent/preflight.md`; `.planning/active/sprint75_plan.md`
  (Phase F lines 249-287, Phase G 291-320, fail-fast 323-348); `sprint75_kickoff.md`
  (full); `web/src/api/daemon.ts` (full); `web/src/pages/Browse.tsx` (full);
  `web/src/pages/Curators.tsx` (full); `web/src/pages/BrowsedProject.tsx` (full);
  `web/src/components/AvailabilitySheet.tsx` (full); `web/src/components/VerificationDetail.tsx`
  (full); `web/src/App.tsx` (full); `crates/nexus-shell-daemon/src/http.rs`
  (1832-1892 nodes route, 830-918 subscribe + list_browse, 2075-2134 seed_count,
  1894-1951 + 2300-2401 seed_voluntary, 4546-4692 nodes/Q7 tests);
  `crates/nexus-shell-daemon-core/src/browse.rs` (130-218 BrowseStatus/BrowseEntry/CatalogApp);
  `crates/nexus-core-rs/src/node_directory.rs` (110-218 CatalogApp/NodeDirectory/NodeDirectoryEntry);
  `crates/nexus-shell-daemon-core/src/config.rs` (default_curators); `sprint75_phase_d_preflight.md`
  (Q7 plan-adaptation 255-313); `.planning/archive/v2.1/sprint74_audit_findings.md` (WEB-1 row 109).
- Commands run:
  - `git rev-parse --short HEAD` -> `491b3c8`
  - `git grep -n "reachableviaseeder|ReachableViaSeeder"` over `crates/` -> ZERO hits
    in Rust (only planning prose references the never-built variant).
  - `git grep -n "WEB-1" sprint74_audit_findings.md` -> row 109 defines the carry.
  - `node -e` on `web/package.json` -> react-router-dom ^7.14.0, zod ^3.25.76,
    @tanstack/react-query ^5.96.2, vitest ^4.1.4.
  - `git grep -nE "135\.181\.42\.188"` over `crates/**/*.rs` -> ZERO hits (lock-3 clean).

## Scope
- Plan source: `.planning/active/sprint75_plan.md` Phase F lines 249-287.
- Target files: `web/src/App.tsx` (routes lazy `/nodes` + `/node/:nodeId`);
  `web/src/pages/Nodes.tsx` (NEW); `web/src/pages/NodeCatalog.tsx` (NEW);
  `web/src/pages/Browse.tsx` (Q6 cohabitation + honest `known_browse_entries`);
  `web/src/components/AddAnchorDialog.tsx` (NEW); `web/src/api/daemon.ts`
  (`listNodes`, `nodeCatalog`/derived, `addAnchor` alias, Zod schemas);
  `web/src/components/AvailabilitySheet.tsx` (WEB-1 + Q7 badge).
- Deps/APIs/specs: NONE new. Front-only phase reusing react-router-dom v7,
  zod v3, react-query v5 exactly as the existing pages do. Backend routes
  `GET /api/daemon/nodes`, `GET /api/daemon/seed-count/{pid}?archive_hash=`,
  `POST /api/daemon/curators/subscribe`, `POST /api/daemon/seed`,
  `GET /api/daemon/browse` ALL already exist (Phases B-E).
- Security/protocol surfaces: NO new wire type, NO new `DOMAIN_*`, NO
  `*_VERSION` bump. Phase F is the CONSUMER of the additive `/nodes` envelope
  pinned in Phase D + provenance display (verrou 4) + lock-1/2/3/4/5 UI proofs.
- Tests expected (plan F.3, Vitest): `Nodes` render + empty/cold-start;
  `NodeCatalog` pull; `AddAnchorDialog`; `daemon` `.strict()` envelope schemas;
  WEB-1 toggle reconciled; lock-1 (0 host field at publish); lock-4 provenance
  (a) author signature shown, (b) fork "version derivee" marker not original
  badge, (c) seeder never authority; badge Q7.

## S1a OSS Prior Art
- Domain: node-centric / repo-centric PULL discovery client UI (list of
  publishers -> per-publisher catalog -> add-publisher intention -> provenance
  display) — F-Droid client repository model.
- Sources (accessed 2026-06-10):
  - F-Droid "Setup an F-Droid App Repo" + repository management docs —
    `https://f-droid.org/en/docs/Setup_an_F-Droid_App_Repo/` ; the client UX is
    Settings -> Repositories -> "+" (paste repo URL + SHA-256 fingerprint),
    per-repo app catalog, TOFU on the signing key.
  - Kickoff §0 already cites the underlying protocol prior art consumed by this
    UI: Nostr NIP-65 outbox, Radicle Heartwood seed!=authority, F-Droid Security
    Model (repo = single signing key, custom repos first-class equal, no central
    authority), BEP-44 — `sprint75_kickoff.md:16-42`.
- Finding: **APPROACH-ALIGNED**. The Phase F UX maps 1:1 onto the mature
  F-Droid client: `/nodes` = the Repositories screen; `/node/:nodeId` = a repo's
  app list; AddAnchorDialog = "add a repository" (a node's pubkey is its repo
  fingerprint AND signing key — `NodeDirectory.node_id` is both the dialable
  identity and the signing identity, `node_directory.rs:162-167`); the anchor is
  a discovery source, never an authority, and app provenance stays attached to
  the app (verrou 4 = F-Droid keeping per-app signing distinct from repo signing).
- Impact: none. The aligned model REINFORCES the verdict drivers below (Q6
  additive cohabitation, AddAnchor = subscribe-to-pubkey, seeder != authority).

## S1b Dependencies, CVEs, Release Notes
- Scanned: `web/package.json` — react-router-dom ^7.14.0, zod ^3.25.76,
  @tanstack/react-query ^5.96.2, vitest ^4.1.4, lucide-react. No Cargo changes
  (front-only phase).
- Commands/sources: `node -e` dump of `web/package.json` versions (above). No
  new dependency is added or bumped: every page (`Browse.tsx`, `Curators.tsx`,
  `BrowsedProject.tsx`) already imports these exact APIs (`createBrowserRouter`
  + `lazy`, `useQuery`/`useMutation`, `z.object().strict()`), so the transitive
  graph is unchanged from HEAD `491b3c8` (which builds green per S75 entry
  counters, kickoff §1.3).
- Finding: **clean**. Front-only, zero dep delta, no transitive collision
  possible (no lockfile change). P2-PREFLIGHT-TRANSITIVE-DEPTH N/A (no add/bump).

## S2 Historical Decisions
- Commands: `git grep -n "reachableviaseeder|ReachableViaSeeder"` over `crates/`
  (ZERO Rust hits); `git log --all --grep="WEB-1"`; read of
  `sprint75_phase_d_preflight.md` Q7 section + `sprint74_audit_findings.md:109`.
- Decisions crossed:
  1. **`BrowseEntry.node_id` stays `#[serde(skip)]`** (`browse.rs:199-205`):
     deliberate Phase D decision (preflight D `sprint75_phase_d_preflight.md:288-297`)
     to keep `/browse` bytes byte-identical and expose node identity via the
     additive `GET /api/daemon/nodes` route instead. Reversion status: NOT
     reversed — STILL VALID and HONORED by the plan ("node_id reste
     `#[serde(skip)]` sauf besoin justifie"). Phase F has no justified need to
     un-skip it. Non-blocking; preserved.
  2. **Q7 "reachable-via-seeder" = honest signal PAIR, not a new `BrowseStatus`
     variant** (commit `0010450` Phase D, test `reachable_via_seeder_status`
     `http.rs:4587-4692`). Phase D preflight EXPLICITLY recommended option (b)
     (`sprint75_phase_d_preflight.md:269-287`: "keep `/browse` byte-identical in
     D and defer the VISIBLE seeder-status to Phase F, surfacing seeder
     availability ... through the existing per-app `seed-count` route + the new
     `/nodes` route"). The Phase D IMPLEMENTATION executed (b): the test asserts
     a dead-anchor directory app reports `status:"unreachable"` on `/browse`
     AND `peer_count:1` on `/seed-count?archive_hash=` — the two existing signals
     the front composes into the badge. Reversion status: NOT reversed; this is
     the design of record. See S4 + Plan Adaptation.
  3. **lock-3 tripwire** (`config.rs:252` `default_curators` defaults empty,
     validated 64-hex `:340-346`, absent section => empty `:562-563`): no
     hard-coded anchor in any compiled default; `git grep 135.181.42.188` over
     Rust = ZERO. Phase F is front-only and touches no config default.
     Non-blocking; structurally preserved.
- Finding: **clean** (no unreversed decision contradicted). One drift to flag,
  routed to PLAN-ADAPT (not DESIGN-CONFLICT) because it is the plan TEXT that is
  stale, not the code: the Phase F plan/handoff still frames Q7 as "nouveau
  variant `BrowseStatus` `reachableviaseeder` = LE changement wire de `/browse`"
  (`sprint75_phase_f_handoff.md:107-108`), which CONTRADICTS the Phase D decision
  of record (item 2). Implementing the handoff literally would (a) add a
  `BrowseStatus::ReachableViaSeeder` variant, changing `/browse` bytes; (b)
  require the front Zod `BrowseStatusSchema` enum to gain the value; (c) duplicate
  a signal already delivered by `/seed-count`. Correct approach is the
  already-built honest pair. Classified APPROACH-corrective (see Plan Adaptation).

## S3 Local Patterns And Threat Model
- Threats/contracts checked: verrou 1-5 (kickoff §4); THREAT_MODEL §15 (seed
  cross-node over-count surface, deferred to Phase G per kickoff deferrals).
- HARDENING_ROADMAP status: no Phase F pre-requirement; the anti-Sybil triad
  (signature + kudos threshold + curator curation, kickoff §4 test cardinal) is
  enforced at the BACKEND ingest layer (already shipped Phases B/C/D); Phase F is
  a read-side projection and adds no aggregation authority.
- Verrou audit for the planned UI:
  - **Verrou 1 (zero host/target field)**: `/nodes` + `/node/:nodeId` are
    read-side projections of `directory_snapshot()`; AddAnchorDialog is a
    subscribe-to-pubkey input (same shape as the existing curator pubkey input,
    `Curators.tsx:157-166`), NOT a "publish to X" selector. The publish path
    (`Deploy.tsx`) is untouched and already carries 0 host field (lock-1).
    Preserved — must be asserted by the `lock-1` Vitest.
  - **Verrou 2 (additive, never substitutive)**: drives Q6 (below). The grid
    must remain a reachable view; node-Browse is an ADDITION. Preserved by design.
  - **Verrou 3 (no compiled anchor)**: front cannot hard-code an anchor that
    matters — but the AddAnchor cold-start MUST NOT ship a pre-filled default
    pubkey placeholder that auto-subscribes (R4/C4). The dialog's example/
    placeholder must be inert text (mirror `Curators.tsx:163` `placeholder="abcd1234..."`),
    never an auto-`subscribe`. Preserved if AddAnchor requires explicit paste+submit.
  - **Verrou 4 (provenance = author, never seeder)**: the acceptance criterion.
    See S4 wire-contract gap — this is the load-bearing finding.
  - **Verrou 5 (state-triggered suggestion)**: the cold-start "ajouter une ancre"
    prompt is shown because the observed state is "0 nodes / empty browse", not
    pushed at publish. Preserved if the empty-state renders the AddAnchor CTA.
- Finding: **clean** as threats; verrou 4 raises a wire-contract requirement
  handled in S4 (non-blocking, achievable with existing routes — see Plan
  Adaptation). No covered-threat regression.

## S4 Protocol And Wire Invariants
- Wire/security files checked: `node_directory.rs` (`CatalogApp` fields,
  `NODE_DIRECTORY_FORMAT_VERSION=1` unchanged); `browse.rs` (`BrowseStatus`
  enum = `reachable|unreachable|unknown` ONLY, `BrowseEntry`/`BrowseSource`
  serde); `http.rs` (`NodesResponse`/`NodeSummary` projection 1832-1875,
  `/seed-count` 2080-2134, `seed_voluntary` + `SeedVoluntaryRequest` 1909-1912).
- VERSION/domain/canonical status: Phase F bumps NOTHING and adds no `DOMAIN_*`.
  Pure front consumer of routes shipped B-E.
- Day 0 status: **preserved** (D1-D5, verrous 1-5; node_id `#[serde(skip)]` and
  Q7-honest-pair are both Day-0-consistent S2 decisions).
- Producer -> consumer trace (P2-PREFLIGHT-WIRE-CONTRACT-DEPTH) for every field
  Phase F reads:
  - **`GET /api/daemon/nodes` envelope `{ nodes: [{node_id, revision, app_count,
    catalog}] }`**: PRODUCER = `nodes_response()` (`http.rs:1863-1875`), pinned
    by `nodes_response_pins_envelope_and_grouping` (`http.rs:4547-4585`) AND
    traversed over HTTP in `reachable_via_seeder_status` part (c)
    (`http.rs:4669-4689`). `node_id`=lowercase hex string (always present),
    `revision`=u64 (always), `app_count`=usize (always), `catalog`=array of
    `CatalogApp`. CONSUMER = NEW Phase-F Zod `NodesResponseSchema`. CONTRACT:
    Zod `.strict()` on the ENVELOPE `{nodes}` only; `catalog` rows must NOT be
    `.strict()`-locked against future additive fields per the deferred review-D
    rule (an additive `CatalogApp` field 0-bump would 422 a strict row schema).
    All four envelope keys + the four current `CatalogApp` keys are
    ALWAYS-present (Rust `Serialize` structs, no `Option`, no `skip`) =>
    non-`.optional()`, non-`.nullable()` (S73-E always-present rule).
  - **`CatalogApp` = `{project_id, archive_hash, name(project_name), category,
    description}`** (`node_directory.rs:122-150`): PRODUCER = signed directory
    blob, verbatim. CONSUMER = `/node/:nodeId` catalog cards. **WIRE-CONTRACT GAP
    (verrou 4)**: `CatalogApp` carries NO `provenance_hash`, NO `repo_url`, NO
    `is_open_source`. The catalog row alone CANNOT (a) show the author-signature
    badge (`VerificationDetail` needs `provenanceHash` + opens by `projectId`),
    nor (b) distinguish a fork (`is_open_source=false`, distinct hash) from the
    original. RESOLUTION (no new wire needed): `VerificationDetail` fetches
    provenance by `projectId` from `${coordUrl}/api/v1/project/{projectId}/provenance`
    (`VerificationDetail.tsx:55-56`) — it does NOT require the catalog to carry
    `provenance_hash`; it can be opened with `provenanceHash={null}` (the prop is
    `string | null`, `VerificationDetail.tsx:30`) and still renders the verified
    author signature + repo + commit + node. The fork-vs-original distinction is
    STRUCTURAL and already true on the wire: a fork has a DISTINCT `archive_hash`
    (kickoff §4(4): "hash BLAKE3 different") and a distinct `project_id`
    (`blake3(name)`), so it appears as a SEPARATE catalog row — the UI marks
    "version derivee" by detecting that the provenance fetch returns a node_id
    != the original author OR by `is_open_source=false` learned from the
    `/browse` entry (when the same app is ALSO in `/browse`). For the catalog-only
    case, the honest minimum is: render `VerificationDetail` (author signature is
    the authority), and label the catalog card's seeder/anchor node distinctly
    from the author ("publie par <author signature>", "catalogue de <node>") so
    the anchor is NEVER rendered as authority (verrou 4). No `/nodes` byte change.
  - **Q7 badge "joignable-via-seeder"**: PRODUCER = the PAIR already on the wire
    — `/browse` `BrowseEntry.status` (= `"unreachable"` for a dead-anchor
    directory app, never falsely `reachable`, asserted `http.rs:4642-4645`) +
    `/seed-count?archive_hash=` `peer_count > 0` (asserted `http.rs:4663-4666`).
    CONSUMER = a FRONT-computed badge (no Rust change, no new `BrowseStatus`
    variant, no `/browse` byte change). The front renders "joignable via un
    seeder" when `status === "unreachable" && seedCount.peer_count > 0`. This is
    the option-(b) of record (S2 item 2). `BrowseStatusSchema`
    (`daemon.ts:128`) stays `["reachable","unreachable","unknown"]` UNCHANGED.
  - **`SeedVoluntaryRequest`** (`http.rs:1909-1912`): TODAY `{ project_id }`
    only. Deferred D->F: add `archive_hash: Option<String>` with `#[serde(default)]`
    as a discriminator for the multi-anchor first-match collision (review-D
    finding). PRODUCER = front `seedVoluntary()` (`daemon.ts:353-362`, currently
    sends `{project_id}`); CONSUMER = `seed_voluntary` handler. CONTRACT: adding
    an OPTIONAL `archive_hash` to the request struct is runtime-tolerant
    (`#[serde(default)]` legitimate per pre-launch policy: a body omitting it
    deserializes to `None` = today's behaviour). The handler's
    `find_directory_app_by_project` first-match (`http.rs:1946-1947`) would gain a
    hash filter when `Some`. This is a Rust + Zod touch INSIDE a "feat(shell)"
    commit — acceptable but means Phase F is NOT purely front (see Risks). The
    front passes `entry.archive_hash` (already in `BrowseEntry`,
    `daemon.ts:165`) so the version is disambiguated. NOTE: `seedCount` already
    threads `archive_hash` (`daemon.ts:382-395`); `seedVoluntary` should mirror it.
- Finding: **clean wire** (no `*_VERSION` bump, no new domain, `/browse` and
  `/nodes` byte-identical). The verrou-4 catalog gap and the Q7 framing are
  resolved WITHOUT wire changes via existing routes (see Plan Adaptation). The
  `SeedVoluntaryRequest.archive_hash` add is a legitimate `#[serde(default)]`
  optional, non-blocking.

## Plan Adaptation
Two plan-text corrections; both implement an already-shipped or
evidence-aligned approach (NOT a pivot away from the kickoff design).

1. **Q7 badge — DO NOT add a `BrowseStatus::ReachableViaSeeder` variant.**
   - Original plan/handoff: "nouveau variant `BrowseStatus` (`reachableviaseeder`
     lowercase serde) + Zod + rendu ... C'est LE changement wire de `/browse`
     assume en F" (`sprint75_phase_f_handoff.md:107-110`).
   - Evidence requiring adaptation: Phase D preflight recommended option (b)
     (`sprint75_phase_d_preflight.md:269-287`); Phase D commit `0010450` IMPLEMENTED
     it; `git grep "reachableviaseeder"` over `crates/` returns ZERO Rust hits;
     `BrowseStatus` enum has exactly `Reachable|Unreachable|Unknown`
     (`browse.rs:146-167`); test `reachable_via_seeder_status` (`http.rs:4587`)
     pins the honest signal PAIR (`status:"unreachable"` + `peer_count:1`).
   - Corrected approach: compute the "joignable via un seeder" badge FRONT-SIDE
     from the two existing signals — `BrowseEntry.status === "unreachable"`
     (from `/browse`) AND `seedCount(projectId, archive_hash).peer_count > 0`
     (from `/seed-count`). No Rust change, no new enum variant, `BrowseStatusSchema`
     unchanged, `/browse` byte-identical. Render the badge in the catalog card /
     AvailabilitySheet ETAT section.
   - File/test delta vs plan: REMOVE the planned `browse.rs` enum-variant edit and
     the `BrowseStatusSchema` enum-extension edit. ADD a small front helper +
     Vitest "badge Q7" asserting the pair-derived badge appears when
     unreachable+peer_count>0 and is absent otherwise. The fail-fast becomes
     web-only for Q7 (the Rust signal is already green at HEAD).

2. **Verrou-4 catalog card provenance — open `VerificationDetail` by projectId,
   not by a catalog `provenance_hash` field that does not exist.**
   - Original plan: "chaque carte du catalogue AFFICHE la preuve de provenance ...
     via le composant `VerificationDetail` existant" (plan F.1 256-261).
   - Evidence: `CatalogApp` (`node_directory.rs:122-150`) has NO `provenance_hash`/
     `is_open_source`/`repo_url`; `VerificationDetail` fetches by `projectId`
     (`VerificationDetail.tsx:55-56`) and accepts `provenanceHash: string | null`
     (`:30`), so it works with `null`.
   - Corrected approach: catalog card opens `VerificationDetail` with
     `projectId={app.project_id}` and `provenanceHash={null}` (the daemon's
     provenance route is the authority; the optional hash-mismatch check just
     stays inert when null). The card labels the ANCHOR/seeder node distinctly
     from the AUTHOR (anchor = "catalogue de ce noeud", author = the verified
     signature inside `VerificationDetail`) so the seeder is never rendered as
     authority (verrou 4(c)). Fork distinction is structural: a fork is a
     DISTINCT catalog row (distinct `archive_hash` + `project_id`); when the same
     app is also present in `/browse`, reuse `is_open_source` there to render the
     "version derivee" marker; for catalog-only rows, surface the author node_id
     from the provenance fetch and mark "version derivee" when it differs from
     the app's canonical author (or when `is_open_source=false`).
   - File/test delta: NO new wire field. `lock-4` Vitest asserts (a)
     `VerificationDetail` opens from a catalog card and shows the author
     signature, (b) a fork row (distinct hash, `is_open_source=false`) renders
     "version derivee" not the original badge, (c) the anchor/seeder node label
     is never styled as the authority badge.

3. **`addAnchor` = the existing `subscribeCurator` route, not a new route.**
   - Evidence: kickoff D1/Q3/DQ3 — the anchor IS a subscription in the SAME
     attention set (`default_curators`/subscriptions), "PAS de section [directory]"
     (MEMORY.md S75 Phase E note). The HTTP surface is
     `POST /api/daemon/curators/subscribe { curator_pubkey_hex }`
     (`http.rs:830-862`), which calls `curator_runtime.subscribe(pubkey)` — the
     SAME runtime whose `directory_snapshot()` feeds `/nodes`. Directory ingest is
     gossip-driven + boot `repull_directories` (subscription-gated,
     `iroh_runtime.rs:950/1055`), NOT triggered synchronously by subscribe.
   - Corrected approach: AddAnchorDialog reuses `subscribeCurator` (alias it
     `addAnchor` in `daemon.ts` for vocabulary clarity, or call it directly).
     Cold-start UX: after subscribe, the node appears in `/nodes` only once a
     gossip directory announcement arrives or after a boot re-pull — mirror the
     existing curator "En attente d'une premiere annonce gossip..."
     (`Curators.tsx:294`) so the user is not shown a dead empty node. Invalidate
     `["daemon-curators"]`, `["daemon-browse"]`, and a new `["daemon-nodes"]`
     query key on success.

## Q6 decision (node-Browse vs grille — TRANCHE)
**Additive cohabitation, grid stays the default landing; node-Browse is a new
peer surface.** Rationale + verrou 2 binding:
- Verrou 2 (kickoff §4(2)) REQUIRES node-Browse be "additif / sur-ensemble
  strict de la grille curator-agregee, jamais substitutif silencieux." A
  replace-the-grid design violates this.
- F-Droid prior art (S1a): the app list and the Repositories screen are SEPARATE
  surfaces; the catalog-by-repo view never replaces the flat app list.
- Concrete UX:
  1. `/browse` (grid) STAYS the index landing and the default app discovery
     surface (curator-aggregated + direct + nodedirectory entries all flow into
     `/browse` already — `aggregate()` adds the `BrowseSource::NodeDirectory`
     boucle, `browse.rs:767`). Add a visible entry point ("Parcourir par noeud"
     / "Sources") linking to `/nodes`. `known_browse_entries` (info endpoint,
     `daemon.ts:43`) keeps counting ALL discoverable apps honestly (verrou 2) —
     the grid is the honest superset, `/nodes` is the by-publisher lens.
  2. `/nodes` = the Repositories screen: list of subscribed catalog-publishers
     (`listNodes`), each row showing node_id (truncated), revision, app_count,
     plus the AddAnchorDialog CTA. Empty-state renders the AddAnchor cold-start
     prompt (verrou 5: state-triggered).
  3. `/node/:nodeId` = that publisher's catalog (cards from `NodeSummary.catalog`),
     each card opening provenance (verrou 4) and offering pull/seed.
- This keeps the grid (reachable-superset) and the by-node lens BOTH present;
  the grid is never silently substituted. Asserted by a Vitest that `/browse`
  still renders the grid and `known_browse_entries` is unchanged by adding `/nodes`.

## Risks And Scope Cuts
- Blocking risks: **none**. The two plan-text drifts are PLAN-ADAPT corrections
  to already-shipped backend decisions, not blocking S1b/S2/S3/S4 findings.
- Non-blocking risks / carry-over:
  - `SeedVoluntaryRequest.archive_hash` add makes this "feat(shell)" commit touch
    Rust (`http.rs` request struct + handler filter) + a Rust test, not pure
    front. Acceptable (`#[serde(default)]` optional, pre-launch policy) but the
    fail-fast MUST run the FULL dual-platform Rust suite + web, not web-only —
    flag to the implementer (kickoff feedback_full_failfast). If kept strictly
    front-only this sprint, the multi-anchor first-match collision stays a
    documented carry to Phase G/S76 (the front can still pass `entry.archive_hash`
    once the handler accepts it).
  - Cold-start latency: a freshly added anchor is invisible in `/nodes` until a
    gossip announcement or boot re-pull (subscribe is not synchronous-ingest,
    `http.rs:853`). Mirror the existing curator "waiting" affordance; not a defect.
  - Q7 badge is best-effort (content-addressing is the truth, kickoff scope cut
    #11); the badge must read "joignable via un seeder (best-effort)" honesty,
    never assert hard reachability of a dead anchor.
  - THREAT_MODEL §15 rows, PULL-3 cross-tier failover, anti-Sybil sampling =
    deferred to Phase G / S76 (kickoff deferrals) — not Phase F.
- Scope cuts still honored (kickoff §9): #1 SearchManifest stays deferred (Phase
  F adds no manifest); #6 front stays the React shell (no new client); #7 no
  `*_VERSION` bump (front-only + one `#[serde(default)]` optional); #11
  multi-anchor advanced ordering deferred (AddAnchor subscribes to N anchors,
  no priority UX). Verrous 1-5 all preserved (S3).

## Action
- **PLAN-ADAPT**: proceed with Phase F using the corrected approaches above.
  The commit body MUST cite this preflight and document:
  "Plan/handoff framed Q7 as a new `BrowseStatus::reachableviaseeder` wire
  variant; preflight S2/S4 found Phase D (`0010450`, test
  `reachable_via_seeder_status`) already delivers the honest signal PAIR and
  the variant was never built (`git grep` ZERO) — adapted to a front-computed
  badge from `status==unreachable && seed_count.peer_count>0`, `/browse`
  byte-identical. Verrou-4 catalog provenance opens `VerificationDetail` by
  projectId (CatalogApp has no provenance_hash field). AddAnchor reuses the
  existing `/curators/subscribe` route (anchor = subscription, kickoff D1/Q3)."
  Q6 = additive cohabitation (grid stays default, `/nodes` is a new lens).
  The plan file remains an unchanged snapshot; the deviation is traced here +
  in the commit body only.
