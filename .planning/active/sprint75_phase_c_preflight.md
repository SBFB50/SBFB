# Sprint 75 Phase C Preflight

Date: 2026-06-09
HEAD: `cc8e329`
Verdict: **PLAN-ADAPT**

## Evidence Rules
- Claim policy: every claim below cites a path, command output, URL/date, or
  explicit assumption.
- Local sources read: `prompts/agent/preflight.md`; `.planning/active/sprint75_plan.md`
  (§Phase C C.1-C.5); `.planning/active/sprint75_kickoff.md` (§4 verrous, §5 D1-D5,
  §6 R4, §10 Q1-Q8, §11 risks); `.planning/active/sprint75_phase_c_handoff.md`;
  `.planning/research/s75_discovery_pull_anchor_kickoff_prompt.md` (§5-§6);
  `crates/nexus-core-rs/src/node_directory.rs`; `crates/nexus-core-rs/src/signed_list.rs`;
  `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` (385-423, 140-211, 666-740);
  `crates/nexus-shell-daemon-core/src/browse.rs` (100-298, 566-734);
  `crates/nexus-shell-daemon/src/runtime.rs` (1400-1571, 1800-1906, 2071-2091);
  `crates/nexus-shell-daemon/src/http.rs` (1077-1304, 1335-1384);
  `crates/nexus-core-rs/src/blobs.rs` (160-220); `crates/nexus-core-rs/src/discovery.rs`
  (70-112, 270-327); `crates/nexus-coordinator-rs/src/public_feed.rs` (25-91);
  `crates/nexus-coordinator-rs/src/search.rs` (190-254); `crates/nexus-shell-daemon/src/seed_registry.rs`
  (40-167); `crates/nexus-coordinator-rs/src/db.rs` (685-757);
  `crates/nexus-shell-daemon-core/src/config.rs` (238-320); `docs/security/THREAT_MODEL.md`
  (§15, 825-859); memory `feedback_approach.md`, `feedback_context7_systematic.md`.
- Commands run: `git rev-parse --short HEAD` -> `cc8e329`; `git log --oneline -6`;
  grep `MAX_PROOF_AGE_SECS` -> `pow.rs:109 = 1_800`; grep version constants ->
  `FEED_FORMAT_VERSION=1`, `CURATOR_LIST_FORMAT_VERSION=1`, `ANNOUNCEMENT_VERSION=1`,
  `PROJECT_ANNOUNCEMENT_VERSION=1`, `NODE_DIRECTORY_FORMAT_VERSION=1`; context7
  `/n0-computer/iroh-blobs` Downloader.download signature.

## Scope
- Plan source: `.planning/active/sprint75_plan.md` §Phase C (C.1-C.5), lines 137-176.
- Target files:
  - `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` (sibling ingest arm for
    `NodeDirectoryEntry` via `verify_signed_list_ingest`).
  - `crates/nexus-shell-daemon-core/src/browse.rs` (`BrowseSource::NodeDirectory`,
    aggregator branch setting `node_id`, directory store).
  - `crates/nexus-shell-daemon/src/runtime.rs` (gossip dispatch arm replacing the
    drop-at-debug at `:1559`; boot re-pull).
  - `crates/nexus-coordinator-rs/src/public_feed.rs` (WIRE-1: `ReleasePublishedPayload`
    + `project_name`/`category`).
  - `crates/nexus-coordinator-rs/src/search.rs` (WIRE-1: `extract_index_fields`
    reads `field("category")`).
  - `crates/nexus-shell-daemon/src/seed_registry.rs` (WIRE-2: re-key by
    `(project_id, archive_hash)`).
  - `crates/nexus-coordinator-rs/src/db.rs` + `crates/nexus-shell-daemon/src/http.rs:1342`
    (DBQ-1: `set_keep_online` coalesces the stored archive_hash).
- Deps/APIs/specs: no new dependency. iroh-blobs `Downloader.download(hash, providers)`
  (already used by `fetch_ticket`, `blobs.rs:186-190`). No `Cargo.toml` edit in scope.
- Security/protocol surfaces: NEW receive-side ingest of a remote-authored signed
  type (`NodeDirectoryEntry`) over gossip + blob fetch; the 5 anti-recentralization
  verrous; the anti-Sybil triad; THREAT_MODEL §15 over-count residual (row D).
- Tests expected (plan C.3): `node_directory_ingest_subscription_gated`,
  `browse_aggregator_sets_node_id_from_directory`, `boot_repull_restores_remote_catalogs`,
  `release_published_searchable_by_name`, `seed_count_keyed_by_project_and_hash`,
  `set_keep_online_coalesces_known_hash`.

## S1a OSS Prior Art
- Domain: decentralized PULL discovery — self-published signed node catalogs that
  must survive a reboot when the publisher is offline (no persisted remote entries,
  re-fetched on demand).
- Sources (accessed 2026-06-09, via kickoff §0 research + this scan):
  - F-Droid Security Model (`f-droid.org/docs/Security_Model`): each repo is a
    signing key (TOFU); the client persists the repo *fingerprint*, NOT the index;
    the index is re-downloaded from the repo URL/mirror on refresh. The repo carries
    a stable fetch locator (the URL).
  - Nostr NIP-65 / Outbox model (`nips.nostr.com/65`): a client persists a small
    *relay list* (locators), then re-fetches content from those relays. The relay
    is addressed by a stable URL, not just a pubkey.
  - Radicle Heartwood (`docs.radicle.xyz/guides/protocol`): a seed node is addressed
    by a stable `<nid>@<host>:<port>` locator; `INVENTORY`/`REFS` announcements are
    re-fetched from seeds the node *follows*; seed != authority.
  - iroh-blobs `Downloader.download(HashAndFormat, Shuffled::new(vec![addr.id]))`
    (context7 `/n0-computer/iroh-blobs`, 2026-06-09): downloads a *known hash* from
    a list of `EndpointId`s; the address is resolved through the endpoint's
    `address_lookup` (pkarr/DNS) OR a seeded `MemoryLookup`. Confirms a bare node_id
    CAN be dialed, but the **content hash must already be known**.
- Finding: **APPROACH-NAIVE** on one specific sub-claim of the plan (boot re-pull
  "from persisted node_ids alone"); APPROACH-ALIGNED on every other element
  (sibling type, F-Droid fingerprint-persist + index-refetch, RAM-only entries,
  subscription-gate, additive wire).
- Impact: the plan's C.1/C.2 (ingest arm, aggregator, BrowseSource) are aligned and
  implementable as written. The plan's C.3 boot re-pull as literally phrased —
  "iterer les pubkeys d'ancre, re-fetch leurs blobs" (plan:152, handoff:74) — is
  under-specified against the F-Droid/Radicle prior art it cites: every mature
  system persists a **stable fetch locator** (URL / `<nid>@host`), then re-fetches.
  A bare Ed25519 `node_id` is **not** a sufficient locator to re-fetch the directory
  *blob*, because (a) iroh download needs the *content hash* of the directory blob,
  which is not derivable from a node_id (`blobs.rs:170-193` requires a `BlobTicket`
  carrying both hash and addr; `Downloader.download` needs the hash), and (b) there
  is **no "ask node X for its current catalog" RPC** in the codebase
  (research §5 GAP, `s75_discovery_pull_anchor_kickoff_prompt.md:99,101`). The plan
  must adapt to the prior-art shape: persist a fetch locator, not just a node_id.
  Per preflight rules an S1a blocking finding maps to **PLAN-ADAPT**, not
  DESIGN-CONFLICT — see `## Plan Adaptation`.

## S1b Dependencies, CVEs, Release Notes
- Scanned: no `Cargo.toml` change in Phase C scope (plan §C.2 lists only crate-internal
  files; the handoff:111 expects "probablement pur Rust" with no dep delta).
  `iroh-blobs` Downloader is already a workspace dep (pinned 0.100 per CLAUDE.md;
  used at `blobs.rs:186`). `serde`/`serde_json` already in use for the additive
  WIRE-1 fields.
- Commands/sources: grep of `crates/**/Cargo.toml` not triggered (no add/bump). The
  P2-PREFLIGHT-TRANSITIVE-DEPTH gate (lock + `cargo tree -d`) is N/A this phase: no
  dependency is added or bumped, so no new transitive major-version collision can be
  introduced. (Contrast S72 Phase C/D ollama-rs->schemars 1.2 collision — that was a
  bump; Phase C bumps nothing.)
- Finding: **clean** (no dependency surface changes).

## S2 Historical Decisions
- Commands: `git log --oneline -6`; reverse-commit checks on the directory ingest
  decisions landed in B (`f6637d3`) and A (`479a87c`); read of D1-D5 (kickoff §5)
  and the 5 verrous (kickoff §4).
- Decisions crossed:
  - **D4 (kickoff:224-234)** "durabilite = persister les node_ids d'ancre, PAS les
    entrees distantes (invite over-count/stale)." Rationale still valid (anti
    over-count, THREAT_MODEL §15 row D). **No reversion.** The PLAN-ADAPT below does
    NOT persist remote entries — it persists a *locator* (anchor identity + last-seen
    directory ticket/hash as a re-fetch hint), which is metadata about WHERE to
    re-fetch, not the catalog content itself. This must be framed carefully so it
    does not cross D4: the persisted hint is re-validated by signature + revision on
    re-fetch, exactly as F-Droid re-validates a re-downloaded index against the
    persisted fingerprint. **Assessment: consistent with D4, not a reversion** — but
    the plan's own phrasing ("PAS le dernier ticket d'annonce") in DQ2 framing must
    be reconciled (see DQ2 resolution).
  - **D2 / FIX-A (`479a87c`, kickoff:188-204)**: re-mint address at replay,
    `MAX_PROOF_AGE_SECS=1800` unchanged (verified `pow.rs:109`). The re-mint helper
    `mint_ticket_for_hash` (`runtime.rs:1812`, `pub(crate)`) is reused by the pull
    path. **No reversion**; Phase C consumes it as designed.
  - **D1 / Phase B (`f6637d3`)**: sibling type + generic `verify_signed_list_ingest`
    + `is_node_directory_announcement` discriminator + `announcement_claims_own_node_id`
    guard + drop-at-debug at `runtime.rs:1559-1568`. The handoff (93-95) and the arm
    comment (`runtime.rs:1560-1564`) EXPLICITLY scope the full ingest arm to Phase C.
    **No reversion**; Phase C is the planned continuation.
  - **publish_directory LIVE-ONLY non-persisted (`http.rs:1211-1216`, `f6637d3`)**:
    the comment states the directory announce "does NOT persist to the outbox, so it
    does not replay on NeighborUp / boot yet. Durable replay (outbox persist + a
    directory-aware branch in remint_and_wrap_for_replay) and the receive-side ingest
    arm ... are both Phase C deliverables." This is a deferred-to-C marker, not a
    decision against persistence. **No reversion.**
  - **Verrou 3 (kickoff:128-131, plan fail-fast row 13)**: anchor never hard-coded
    in a compiled `default_*`. `config.rs:245-251` `default_curators` defaults empty
    (`#[serde(default)]`, test `absent [curator] section must yield empty`,
    `config.rs:508-509`). DQ3 below confirms reuse of `default_curators` keeps this
    invariant; a `default_anchors` field would too IF empty-by-default. **No
    reversion**; tripwire respected.
- Finding: **clean** (no DESIGN-CONFLICT — all crossed decisions are forward-planned
  continuations, no rejected decision is being silently reversed). The D4 framing
  nuance is handled by the PLAN-ADAPT locator design, which is *consistent* with D4
  (it persists a re-fetch hint, not catalog content).

## S3 Local Patterns And Threat Model
- Threats/contracts checked: THREAT_MODEL §15 (seed cross-node surface, over-count
  residual row D = M); the 5 verrous; the anti-Sybil triad; the subscription-gate
  (attention-set) DoS posture.
- Full scan performed (new receive-side ingest of a remote-authored signed type +
  a boot network-fetch path = new security surface, per preflight Step 4 escalation).
  Threat mapping of the Phase C directory-ingest primitive:
  - **Assets**: the local `direct_entries`/directory store (integrity of what Browse
    shows); the attention-set (which anchors are trusted); the boot re-pull network
    budget.
  - **Actors**: a subscribed-but-malicious anchor; a non-subscribed flooder; a
    forwarder stapling a foreign signature; a Sybil swarm of fake anchors.
  - **Vectors + mitigations (all already factored in B, reused by C)**:
    - Forged signature / wrong domain -> `NodeDirectoryEntry::verify_signature`
      (version, caps, attribution split-brain, Ed25519 over `DOMAIN_NODE_DIRECTORY_V1`),
      cross-domain replay rejected (`node_directory.rs:563-582`). **Reused via the
      generic gate step 6.**
    - Envelope/payload attribution split-brain -> `verify_signed_list_ingest` step 7
      (`iroh_runtime.rs:403-410`). **Reused.**
    - Revision rollback -> step 8 (`iroh_runtime.rs:412-420`). **Reused** — Phase C
      MUST pass the stored directory revision (per-anchor) into the gate, mirroring
      the curator arm (`iroh_runtime.rs:728`).
    - Non-subscribed flood -> attention-set drop BEFORE blob fetch (the curator
      step-4 filter, `iroh_runtime.rs:684-698`). **Phase C MUST gate the directory
      arm on the same attention-set** (kickoff garde-fou; plan C.3 test #1
      `node_directory_ingest_subscription_gated`).
    - Catalog DoS (oversized) -> `NODE_DIRECTORY_MAX_ENTRIES=256` + per-field caps,
      enforced at verify (`node_directory.rs:276-283`). **Reused.**
    - Over-claim of hosting -> content-addressing: the directory advertises hashes,
      a puller verifies the BLAKE3 on fetch and never serves bytes the host lacks
      (THREAT_MODEL §15 invariant; `node_directory.rs:44-53`). **Structural, holds.**
  - **Anti-Sybil triad (kickoff:151-154)**: leg 1 (Ed25519 signature) = present
    (new domain, B). Leg 3 (curation by signature = subscription/attention-set) =
    present (the ingest is subscription-gated). **Leg 2 (kudos reputation threshold
    for aggregation) is NOT exercised by Phase C** and is a documented scope cut
    (kickoff §9.10 "Kudos-threshold tuning empirique — post-launch", plan §7.10).
    Because aggregation is gated by *explicit subscription* (the user chose this
    anchor), the absence of an automated kudos threshold does NOT regress row D
    below its current M: a subscribed anchor is already a deliberate trust choice
    (curation leg), so the over-count surface is bounded by the user's own
    attention-set, not an open broadcast. This matches the THREAT_MODEL §15 residual
    framing (best-effort, pilote-ferme, content-addressing = authority). **Non-blocking
    documented gap**, not a regression. Record in the commit body + verification that
    leg 2 is satisfied-by-subscription for S75 and the numeric kudos knob stays
    post-launch.
  - **Verrou 5 / confidentiality default-OFF (kickoff:155-158)**: the boot re-pull
    must only fetch anchors the user EXPLICITLY subscribed to (attention-set), never
    a silent default. With `default_curators`/`default_anchors` empty in the shipped
    binary (verrou 3), a fresh install with no subscription performs ZERO boot
    network fetch. **The PLAN-ADAPT boot re-pull MUST iterate the persisted
    attention-set only** (not a compiled default), preserving verrou 5.
- HARDENING_ROADMAP status: no S75/Phase-C pre-requirement is listed as a blocker
  (no `docs/security/HARDENING_ROADMAP.md` Phase-C gate crossed; the seed surface
  hardening §15 is already shipped S74). Gate C6 (Phase A E2E cross-machine before
  gating pull) is a *sequencing* gate, not a HARDENING pre-requirement — see Risks.
- Finding: **clean / non-blocking**. No regression of a covered T0-T5 threat (the
  directory ingest reuses the exact B-verified gate). One documented non-blocking
  gap: anti-Sybil leg 2 (kudos threshold) satisfied-by-subscription for S75, numeric
  calibration post-launch (already a kickoff scope cut). The triad's three legs are
  all present in structural form for the subscription-gated path.

## S4 Protocol And Wire Invariants
- Wire/security files checked: `node_directory.rs` (`NODE_DIRECTORY_FORMAT_VERSION=1`,
  `DOMAIN_NODE_DIRECTORY_V1`); `public_feed.rs` (`FEED_FORMAT_VERSION=1`,
  `ReleasePublishedPayload`); `search.rs` (`extract_index_fields`); `browse.rs`
  (`BrowseEntry` serde, `BrowseSource`); `seed_registry.rs` (in-memory, NOT wire);
  `db.rs` (M18 keep_online, local SQLite, NOT wire).
- VERSION/domain/canonical status: all constants confirmed `= 1` (command output:
  `FEED_FORMAT_VERSION=1`, `CURATOR_LIST_FORMAT_VERSION=1`, `ANNOUNCEMENT_VERSION=1`,
  `PROJECT_ANNOUNCEMENT_VERSION=1`, `NODE_DIRECTORY_FORMAT_VERSION=1`). Phase C bumps
  NONE. `DOMAIN_NODE_DIRECTORY_V1` already posed in B; Phase C adds no new domain.
  `MAX_PROOF_AGE_SECS=1800` unchanged (`pow.rs:109`).
- Day 0 status: **preserved** (D1-D5, verrous 1-5; see S2).
- Producer -> consumer trace (P2-PREFLIGHT-WIRE-CONTRACT-DEPTH) for every touched
  wire/serialized field:
  - **WIRE-1 `ReleasePublishedPayload.project_name`/`category`** (new, additive):
    - PRODUCER: `public_feed.rs:32-40` `ReleasePublishedPayload` (the deploy/finalize
      path serializes this into the feed op `serde_json::Value`). Adding two
      `Option<String>` fields with `#[serde(default, skip_serializing_if = "Option::is_none")]`
      is additive: an op without them deserializes to `None`, and is omitted on
      serialize when `None` -> byte-identical for existing producers (0-bump per
      pre-launch policy + the field stays absent until a producer sets it).
    - CONSUMER: `search.rs:223-250` `extract_index_fields` ALREADY reads
      `field("project_name")` (line 241), returning `""` when absent (current behavior
      for feed ops). The plan adds `field("category")` reading where line 244 currently
      hard-codes `category: String::new()`. After WIRE-1, a `ReleasePublished` op that
      carries `project_name` becomes FTS5-searchable by name (closes
      FRESHNESS-RELEASE-UNINDEXED). **Both ends agree**: `serde_json::Value::get(key)`
      tolerates absent keys -> `None`/`""`. No envelope/null-vs-absent drift (the
      consumer uses `.get().and_then(as_str).unwrap_or("")`, `search.rs:224-228`).
      **Confirmed additive, 0-bump.** NOTE the plan must wire a PRODUCER that actually
      sets `project_name`/`category` on the release op (otherwise the field is dead,
      the S72-Phase-D lesson: no producer => the consumer change is inert). Verify the
      finalize_deploy/ReleasePublished emit site is updated, not only the struct.
  - **WIRE-2 `SeedRegistry` re-key by `(project_id, archive_hash)`** (NOT wire):
    - The registry is `HashMap<project_id, HashMap<seeder, ts>>` (`seed_registry.rs:30-42`),
      an IN-MEMORY best-effort aggregate fed by `record_announced` (`:107-128`). It is
      NOT serialized on any wire and NOT persisted. The `SeedAnnouncedPayload` ON-WIRE
      shape (`public_feed.rs:86-91`: `project_id`, `seeder_node_id`, `archive_hash`)
      ALREADY carries `archive_hash` — so re-keying the in-memory map to
      `(project_id, archive_hash)` reads a field the wire already provides. **No wire
      change.** Consumer of the count = `count_recent` (`:133`) + the Availability
      panel; re-keying changes only which observations collapse together (separating
      seeders of distinct archive versions of the same project_id). **Additive to the
      data model, zero wire impact.** Caveat: confirm the read-side caller(s) of
      `count_recent`/`seeders_recent` (http.rs Availability) pass the archive_hash so
      the new key is queryable; otherwise the count regresses to "any version".
  - **DBQ-1 `set_keep_online` coalesce** (NOT wire, local SQLite M18):
    - PRODUCER of the volatile hash: `http.rs:1342-1345` sources `archive_hash` from
      `browse_aggregator.get_direct_entry(project_id)` -> `.archive_hash`. After a
      reboot the aggregator no longer holds remote entries and is repopulated late for
      own apps, so a toggle can write `archive_hash=NULL`, dropping the M18 hash the
      skip-GC tag + boot re-announce (`db.rs:742-757` `list_keep_online_enabled` skips
      NULL rows) depend on. CONSUMER: `db.rs:690-706` `set_keep_online` (INSERT OR
      REPLACE, single row per project_id) + `list_keep_online_enabled` (`:753`). DBQ-1
      = when the aggregator returns `None`, COALESCE with the already-stored M18
      `archive_hash` (read `get_keep_online` first, `db.rs:710`) so a re-toggle never
      NULLs a known hash. **Pure local-DB consistency fix, zero wire.** Confirm the fix
      lives in the caller (`http.rs:1342`) or as a `db.rs` coalescing variant; both are
      acceptable, the db-layer coalesce is the more robust (root-cause) option.
  - **`BrowseSource::NodeDirectory`** (`browse.rs:111-120`): a new enum variant on a
    `#[serde(rename_all="lowercase")]` enum with `#[serde(default)] source` on
    `BrowseEntry` (`browse.rs:211-212`). Additive variant; pre-existing daemons
    default to `Curator` on an absent field. NOTE: `BrowseEntry.node_id` is
    `#[serde(skip)]` (`browse.rs:195`) so it never crosses daemon->frontend; Phase C
    sets it server-side for the aggregator probe, exactly as the Direct arm does
    (`runtime.rs:1956`). Setting `node_id` from the directory entry does NOT change
    the `/browse` JSON bytes (the field is skipped) -> frontend Zod untouched.
    **Confirmed: no frontend wire contract change in Phase C** (the node_id exposure /
    `/api/daemon/nodes` is Phase D, plan §D.2). Provenance fields
    (archive_hash/repo_url/provenance_hash/is_open_source) already serialize with
    `skip_serializing_if=Option::is_none` / `serde(default)` (`browse.rs:221-246`):
    carrying them from a directory entry is additive and already frontend-compatible.
- Finding: **clean**. Every touched field is additive with both ends traced; no
  `*_VERSION` bump; no new domain; no tolerant multi-version decoder; `serde(default)`
  uses are runtime-tolerance (documented). No Day-0 contradiction.

## DQ Resolutions (code-anchored)

- **DQ1 — Directory store**: **Extend `CuratorRuntime` minimally OR add a sibling
  `DirectoryRuntime` that mirrors `CuratorRuntime`'s shape.** The least-risk,
  highest-parity option grounded in code: a sibling store keyed by anchor pubkey,
  reusing the exact pattern of `CuratorRuntime.lists: DashMap<pubkey, CuratorListEntry>`
  + `attention: DashMap<pubkey, ()>` + `subscriptions.json` persistence
  (`iroh_runtime.rs:440-466`). The directory store needs the SAME three things:
  (a) `DashMap<pubkey, NodeDirectoryEntry>` for verified entries, (b) an attention-set
  of anchor pubkeys (for the subscription-gate + boot re-pull iteration), (c) a
  persistence file for the attention-set (the locator, see DQ2). Reusing
  `CuratorRuntime`'s attention-set directly (DQ3) avoids a second `subscriptions.json`.
  **Resolution**: store verified `NodeDirectoryEntry` in a new
  `DashMap<[u8;32], NodeDirectoryEntry>` (a field on a `DirectoryRuntime` sibling or
  an added field on `CuratorRuntime`); reuse `CuratorRuntime`'s attention-set as the
  subscription gate (DQ3) so the directory ingest's step-4 filter and the boot re-pull
  iterate `subscribed_pubkeys_hex()`. The verified entries are RAM-only (D4); only the
  attention-set persists. Picking a sibling `DirectoryRuntime` keeps the curator type
  surface clean and mirrors the B precedent of a sibling type; extending `CuratorRuntime`
  with one more DashMap is also acceptable and slightly less plumbing. **Either is
  R1-safe** because the security gate is already the shared `verify_signed_list_ingest`,
  not duplicated.

- **DQ2 — Source of address at boot re-pull (CRUX)**: **PLAN-ADAPT.** The plan's
  literal phrasing ("persist node_ids, re-fetch their blobs at boot") is NOT directly
  realizable, because re-fetching a directory blob needs the directory's *content
  hash* (iroh `Downloader.download(hash, [endpoint_id])`, `blobs.rs:186-190`;
  context7 confirms the hash is required), and a bare node_id yields neither the hash
  nor (without a live pkarr record) the address. There is no "ask node X for its
  catalog" RPC (`s75_..._kickoff_prompt.md:99,101`). Three options, ranked:
  - **Option A (RECOMMENDED) — persist the locator (anchor pubkey + last directory
    announcement ticket) as a re-fetch HINT; re-validate on re-pull.** At ingest, store
    alongside the verified entry the `BlobTicket` from the announcement that delivered
    it (the announcement carries `blob_ticket`, `iroh_runtime.rs:177-178`). Persist a
    small `anchors.json` of `{ pubkey, last_directory_ticket }` (the ticket = the
    F-Droid "repo URL" analog: a stable fetch locator, NOT the catalog content). At
    boot, for each subscribed anchor, dial via the persisted ticket (re-mint not
    needed for the *fetch* — the puller dials the anchor's CURRENT address, and a stale
    ticket address is tolerated because pkarr re-resolves the node_id; iroh
    `presets::N0` wires pkarr DHT discovery, `discovery.rs:4-6`). Run the fetched blob
    through `verify_signed_list_ingest` (signature + revision) exactly as live ingest,
    so a stale/forged persisted ticket cannot poison the store: a re-fetch that fails
    verify or whose anchor is offline simply yields nothing (the catalog stays absent
    until the anchor re-announces live). **This does NOT cross D4**: the persisted
    ticket is a *locator* (where to re-fetch), re-validated by signature+revision on
    use — the prior-art F-Droid pattern (persist fingerprint+URL, re-download+verify
    index). The *catalog content* (the `CatalogApp` entries) is NOT persisted, so the
    over-count anti-pattern D4 rejects (durably storing remote entries) is avoided.
  - **Option B — persist node_id only, rely on a live re-announce.** At boot, dial each
    subscribed anchor node_id (pkarr-resolvable) and wait for the anchor to re-announce
    its directory via gossip. This is the "purest" D4 reading but is **strictly weaker
    than the kickoff's own goal**: it provides ZERO durability when the anchor is
    momentarily offline at the puller's boot (the catalog reappears only when the
    anchor next re-announces). It also requires the anchor to re-announce on a timer
    (publish_directory is currently LIVE-ONLY, one-shot, `http.rs:1211-1216`), which is
    Phase E (VPS driver) work, not guaranteed for a peer anchor. **Does not deliver the
    load-bearing "remote catalog survives reboot" promise** the kickoff §1.1 calls the
    second load-bearing trou.
  - **Option C — persist the verified entries durably (rejected by D4).** Not pursued.
  - **Resolution: Option A.** It is the only option that delivers the load-bearing
    durability (catalog survives the puller's reboot even if the anchor is briefly
    offline at that instant, because the locator lets the puller actively re-fetch the
    moment the anchor is reachable) while honoring D4 (persist locator, not content)
    and verrou 5 (only subscribed anchors; empty default => zero boot fetch). The plan
    text must be amended from "persist node_ids" to "persist anchor locator (pubkey +
    last directory ticket) and re-validate the re-fetched blob by signature+revision";
    this is the PLAN-ADAPT delta. **C6 interaction**: the re-fetch reuses `fetch_ticket`
    + the verify gate; the FIX-A re-mint helper (`mint_ticket_for_hash`) is for the
    PRODUCER (own re-announce), not strictly needed for the consumer re-fetch (the
    consumer dials the anchor, it does not re-point an address) — confirm at code time
    whether the persisted ticket's address is dialed directly or only the node_id +
    pkarr is used; both work, pkarr-by-node_id is the more robust against a stale
    persisted address.

- **DQ3 — Anchor config**: **Reuse `default_curators` (pubkeys), do NOT add
  `default_anchors`.** `config.rs:245-251` `default_curators: Vec<String>` is empty by
  default (`#[serde(default)]`, validated 64-hex at `config.rs:307-317`,
  test `config.rs:508-509`), auto-subscribed at boot (`runtime.rs:404-427`). A node
  directory is signed by the node's Ed25519 pubkey = the SAME key family as a curator
  pubkey (both 64-hex Ed25519). Reusing `default_curators` as the unified
  attention-set (DQ1) means: (a) verrou 3 is already satisfied (empty default, no
  hard-coded anchor in the shipped binary — fail-fast row 13 holds with zero new
  code), (b) one attention-set/`subscriptions.json` covers both curator lists and node
  directories (the subscription-gate at ingest is the same `is_subscribed` check), (c)
  no new config surface to audit. The kickoff frames the VPS as "mon curator par
  defaut" in MY `config.toml` `default_curators` (kickoff:128-130) — reusing the field
  is the kickoff's own intent. **Note the node_id<->pubkey encoding seam** (review B
  P2: "meme secret, encodage different" — `node.node_id()` is z-base-32, the directory
  `node_id`/curator pubkey is 64-hex; `http.rs:1094-1097` documents both derive from
  one secret). The directory store key = the 64-hex Ed25519 pubkey (the
  `directory.node_id` bytes), matching the attention-set hex encoding; the dialable
  identity for re-pull is the same key re-encoded z-base-32 — Phase C must convert
  consistently (the helper `parse_pubkey_hex` exists, `iroh_runtime.rs:681`).

- **DQ4 — Aggregator**: **Add a third loop in `aggregate()` over the directory store**,
  after the curator loop (`browse.rs:643-678`) and the direct_entries loop (`:702-726`).
  For each subscribed anchor's verified `NodeDirectoryEntry`, flatten its `catalog`
  into `BrowseEntry`s with `source: BrowseSource::NodeDirectory`, `node_id:
  Some(entry.directory.node_id hex)`, status probed against the **node_id** (the
  dialable anchor, reusing the `direct_entries` probe branch `:709-716`, NOT the
  project_id which is `blake3(name)`), and the provenance fields
  (archive_hash/repo_url=None-for-now/provenance_hash/is_open_source) carried from the
  `CatalogApp`. **Verrou 2 (additive)**: this loop ADDS rows; it does not replace the
  curator or direct loops. **Verrou 4 (author provenance)**: the `BrowseEntry`
  `archive_hash` = the author's BLAKE3 from `CatalogApp.archive_hash`; `node_id` = the
  *seeder/anchor* identity for dialing, NEVER the authority badge — the authority is
  the author signature, surfaced in Phase F. The aggregator must keep these distinct
  (it already does: `node_id` is dial-only and `#[serde(skip)]`, `browse.rs:182-196`).
  `known_browse_entries` honesty (verrou 2): the count must now include directory-
  sourced apps; confirm the count source (`http.rs:196-214` per research §6 Q7) sums
  curator + direct + directory without double-counting the same `(project_id, source)`.
  **Note `CatalogApp` has no `repo_url`** (`node_directory.rs:122-147`) — the directory
  entry cannot carry repo_url, so `BrowseEntry.repo_url` from a directory source is
  `None`; the verrou-4 provenance display in Phase F derives repo_url/provenance from
  the fetched `provenance.json` at pull time, not from the directory listing. Flag this
  for Phase F (the directory is a discovery index, not the provenance source).

- **DQ5 — WIRE-1 freshness**: confirmed (see S4). `ReleasePublishedPayload`
  (`public_feed.rs:32-40`) currently has NO `project_name`/`category`. Adding both as
  `Option<String>` with `#[serde(default, skip_serializing_if="Option::is_none")]` is
  additive 0-bump. `extract_index_fields` (`search.rs:223-250`) already reads
  `field("project_name")` (line 241) and must change line 244 from hard-coded
  `String::new()` to `field("category")`. **The 0-bump holds.** Caveat (S72-Phase-D
  lesson): a PRODUCER must actually populate these fields on the ReleasePublished emit,
  or the search-by-name remains empty (the consumer change alone is inert). Verify the
  emit site (finalize_deploy / the feed op construction) sets `project_name`.

## Plan Adaptation
- Original plan: `.planning/active/sprint75_plan.md:152` and handoff:74-77 —
  "re-pull boot : iterer les ancres abonnees, re-fetch leurs `NodeDirectoryEntry`
  blobs (reutilise path curator gossip+blob + helper re-mint A)"; kickoff D4:228-233
  "persister les **node_ids d'ancre** + re-pull actif au boot".
- Evidence requiring adaptation (S1a APPROACH-NAIVE): iroh `Downloader.download`
  requires the directory blob's CONTENT HASH (context7 `/n0-computer/iroh-blobs`,
  2026-06-09; `blobs.rs:170-193` requires a `BlobTicket` = addr + hash); a bare
  node_id yields no hash and there is no "fetch node X's catalog" RPC
  (`s75_..._kickoff_prompt.md:99,101`). Mature prior art (F-Droid, NIP-65, Radicle)
  uniformly persists a stable fetch LOCATOR (repo URL / relay URL / `<nid>@host`),
  re-downloads, and re-validates — never re-fetches "from an identity alone".
- Corrected approach (Option A, DQ2): persist a small `anchors.json` of
  `{ anchor_pubkey, last_directory_blob_ticket }` (the ticket = the locator, captured
  from the announcement at ingest, `iroh_runtime.rs:177-178`). At boot, iterate the
  SUBSCRIBED anchors (attention-set = `default_curators`-seeded, DQ3), dial via the
  persisted ticket/node_id (pkarr re-resolves a stale address, `discovery.rs:4-6`),
  fetch the directory blob, and run it through `verify_signed_list_ingest`
  (signature + per-anchor revision) before storing RAM-only. A re-fetch that fails
  verify, hits a non-monotonic revision, or finds the anchor offline yields nothing
  (graceful: the catalog reappears on the next live announce). This honors D4 (persist
  locator, not catalog content; content re-validated on use), verrou 5 (subscribed-
  only, empty default => zero boot fetch), and the anti-Sybil triad (signature +
  revision + subscription).
- File/test delta vs the original plan:
  - ADD a persisted `anchors.json` (or extend `subscriptions.json` with a per-curator
    `last_directory_ticket` field — the latter avoids a second file but mixes curator
    and directory locators; a separate `anchors.json` is cleaner). Either is additive,
    schema-versioned like `SubscriptionsFile` (`iroh_runtime.rs:224-235`).
  - The boot re-pull routine (`runtime.rs`, near the `restore_browse_from_outbox`
    call `:1452`) iterates the persisted anchor locators, fetches + verifies + stores.
  - Test `boot_repull_restores_remote_catalogs` (plan C.3 #3) becomes a 2-node test:
    node A publishes a directory; node B subscribes, ingests, persists the locator,
    "reboots" (new aggregator + directory store from the persisted `anchors.json`),
    re-pulls A's blob, and the remote catalog is present post-reboot. (The prior
    drop-at-debug had no durability; this is the load-bearing assertion.)
  - The commit body must document: "Plan C.3 proposed re-pull from persisted node_ids;
    preflight S1a (iroh-blobs download needs the content hash; F-Droid/Radicle persist
    a locator) adapted to persist an anchor locator (pubkey + last directory ticket)
    re-validated by signature+revision on re-fetch — D4 honored (locator, not content)."
- Plan file remains unchanged (snapshot); the deviation is traced here + in the commit
  body only (per preflight Plan Adaptation rule 5).

## Risks And Scope Cuts
- Blocking risks: **none** (the only blocking-class finding is the S1a APPROACH-NAIVE
  on DQ2, which maps to PLAN-ADAPT, not a stop). The corrected approach is fully
  code-grounded and implementable in Phase C.
- Non-blocking risks / carry-over:
  - **C6 sequencing gate**: Phase A FIX-A E2E cross-machine is NOT yet validated
    (the cross-machine run is scheduled for Phase G via SSH, plan A.3 #6 is
    "unit-simule ici", and the live env needs the SSH assets). The boot re-pull
    (consumer side) does NOT depend on FIX-A re-mint (that fixes the PRODUCER replay);
    the consumer dials the anchor + pkarr. **Resolution: C6 is an ACCEPTANCE-DEFERRED-
    TO-G gate, not a Phase-C code blocker.** Phase C can land the ingest + boot re-pull
    with in-process/2-node tests; the cross-machine "survives-VPS-death" acceptance is
    proven in Phase G (plan G.1, fail-fast row 22). Document in the commit that the
    cross-machine gate is deferred to G (matches the kickoff R6 mitigation: "FIX-A
    lande + E2E cross-machine AVANT que pull soit gated dessus" — the *gating* is the
    G acceptance, the Phase-C code is testable in-process).
  - **Anti-Sybil leg 2 (kudos threshold)**: satisfied-by-subscription for S75; numeric
    calibration is a kickoff scope cut (§9.10). Non-regression of THREAT_MODEL §15
    row D (M stays M, bounded by the user's attention-set).
  - **`CatalogApp` has no repo_url**: directory-sourced `BrowseEntry.repo_url=None`;
    Phase F provenance display must derive repo_url/provenance from the fetched
    `provenance.json`, not the directory listing (flagged for Phase F).
  - **WIRE-1 producer**: ensure a producer sets `project_name` on ReleasePublished
    (else the consumer change is inert — S72 Phase D lesson).
  - **WIRE-2 read-side**: ensure the Availability count caller passes archive_hash so
    the new `(project_id, archive_hash)` key is queryable.
- Scope cuts still honored (kickoff §9 / plan §7): SearchManifest DEFER (#1) — the
  directory is NOT a SearchManifest (distinct type, distinct domain); Tantivy frozen
  (#2); GC reaper deferred (#3); cross-node federated search out of scope (#4);
  multi-anchor advanced UX deferred (#11); 0-bump wire (#7). The PLAN-ADAPT locator
  persistence is NOT a SearchManifest and does not re-open the DEFER (it persists a
  per-anchor fetch hint, not a coverage digest).

## Action
- PLAN-ADAPT: proceed with the corrected boot re-pull approach (Option A: persist
  anchor locator = pubkey + last directory ticket, re-validate by signature+revision
  on re-fetch). The commit body MUST cite this file and document the C.3 deviation.
  C.1 (ingest arm via the shared gate), C.2 (`BrowseSource::NodeDirectory` + aggregator
  node_id), WIRE-1/WIRE-2/DBQ-1 are implementable as planned. C6 cross-machine
  acceptance is deferred to Phase G (non-blocking sequencing). DQ1=sibling/extended
  store reusing the shared gate; DQ3=reuse `default_curators` (verrou 3 holds);
  DQ4=additive third aggregate loop (verrous 2+4 held); DQ5=additive 0-bump WIRE-1.
