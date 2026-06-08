# Sprint 74 Phase F Preflight

Date: 2026-06-08
HEAD: `b76a084` (Phase E landed: SeedRequest ALPN + fetch/tag/pin + invite/approval)
Verdict: **SCOPE-CUT-CONSISTENT**

> Load-bearing findings for the main thread: F-1 (the `op_type` discriminant
> collision risk on the raw-op JSON path), F-2 (boot re-announce: seeded distant
> apps are NOT in the outbox, so a NEW persistence/replay path is required —
> they are distinguishable as keep_online rows WITHOUT a provenance_record),
> F-3 (SeedAnnounced signature: seeder node key, distinct author chain — works
> with the existing per-author hash-chain), F-4 (counter lives in-memory + feed
> aggregate; TTL from `ts`). None blocking.

## Evidence Rules
- Claim policy: every claim cites a path:line, command output, or explicit assumption.
- Local sources read: `prompts/agent/preflight.md` (procedure) ;
  `.planning/active/sprint74_plan.md` (§Phase F 343-384, §2 Day 0, §7 scope cuts,
  §8 R1-R8) ; `.planning/active/sprint74_phase_e_preflight.md` (NF-3 no ticket
  column, FsStore persists blob, wire trace) ;
  `crates/nexus-coordinator-rs/src/public_feed.rs` (entier 1-1446 + tail of enum) ;
  `crates/nexus-shell-daemon/src/feed_sync.rs` (entier 1-799) ;
  `crates/nexus-coordinator-rs/src/db.rs` (M18 262-274, M19 275-299, keep_online
  getters 685-735, seed_invite 745-865) ;
  `crates/nexus-shell-daemon/src/runtime.rs` (boot restore 1425-1539, keep_online
  gate 1496/1527/1598, helpers 1805-1860) ;
  `crates/nexus-shell-daemon/src/seed_protocol.rs` (handler set_keep_online 253,
  fetch_and_pin 225-237, voluntary path comment 36-57) ;
  `web/src/components/AvailabilitySheet.tsx` (entier — Copies de secours 344-365).
- Commands run:
  - `git rev-parse --short HEAD` -> `b76a084`.
  - `git log --oneline -8` -> A `457ca05`, B `bcfc155`, C `9c2bd68`, D `4c1acc5`,
    E `b76a084` all landed.
  - Glob `crates/**/public_feed.rs` -> `nexus-coordinator-rs/src/public_feed.rs`
    (NOT shell-daemon — the plan §F.2 cites `nexus-shell-daemon/src/public_feed.rs
    :82-118`, which DOES NOT EXIST; corrected below in Scope).
  - Glob `crates/**/feed_sync.rs` -> `nexus-shell-daemon/src/feed_sync.rs`.

## Scope
- Plan source: `.planning/active/sprint74_plan.md` §Phase F (343-384).
- **Plan path correction (load-bearing)**: §F.2 names
  `crates/nexus-shell-daemon/src/public_feed.rs (82-118)`. That file does not
  exist. The real `public_feed.rs` (enum + `insert_feed_operation` + raw-op
  validation) is in **`crates/nexus-coordinator-rs/src/public_feed.rs`**. The
  ingest path is in `crates/nexus-shell-daemon/src/feed_sync.rs`
  (`ingest_doc_entry` 113-299). The helper that builds the `SeedAnnounced` raw-op
  belongs daemon-side (it captures the daemon keypair), calling the coordinator
  `insert_feed_operation`/`publish_feed_entry_to_docs` exactly as `feed_insert`
  does (`feed_sync.rs:489-545`).
- Target files (corrected file:line):
  - `crates/nexus-shell-daemon/src/runtime.rs` — boot re-announce of distant
    seeded apps + SeedAnnounced re-emit (NEW path beside `restore_browse_from_
    outbox` 1434; the existing outbox replay 1497/1528 covers SELF apps only — F-2).
  - `crates/nexus-shell-daemon/src/feed_sync.rs` — emit helper (insert + publish
    SeedAnnounced) + ingest aggregate (extend `ingest_doc_entry` or add a typed
    branch on `op_type == "SeedAnnounced"` after the existing search reindex 268).
  - `crates/nexus-coordinator-rs/src/public_feed.rs` — OPTIONAL typed payload
    `SeedAnnouncedPayload` + a validation arm (see F-1: either a 5th enum variant
    OR a dedicated raw-op validator; tranche below).
  - `crates/nexus-shell-daemon/src/seed_registry.rs` (NEW) — in-memory multi-seed
    aggregate + TTL counter (F-4).
  - `crates/nexus-shell-daemon/src/http.rs` — route exposing the seed count to the
    front (e.g. extend the existing browse JSON or a `GET /api/daemon/seed-count`).
  - `web/src/components/AvailabilitySheet.tsx` — "Qui la garde en ligne" multi-seed
    + "Copies de secours" functional (replace the inert "Bientot" 344-365).
- Deps/APIs/specs: **none new**. Composes existing `public_feed`
  (`insert_feed_operation` 327, raw-op `Value`), `feed_sync`
  (`publish_feed_entry_to_docs` 50, `ingest_doc_entry` 113), `keep_online` M18,
  `seed_protocol` E.
- Security/protocol surfaces: `SeedAnnounced` IS wire (feed op propagated via
  iroh-docs). Raw-op extensibility (`FeedEntry.op: Value`, `public_feed.rs:106`)
  -> 0 bump of `FEED_FORMAT_VERSION` (stays 1, `public_feed.rs:20`).
- Tests expected (plan §F.3): `remote_seeder_reannounces_after_reboot_e2e` (§P57),
  `seed_announced_raw_op_no_version_bump`, `seed_announced_ingested_increments_count`,
  `seed_count_best_effort_ttl_expires`, `multi_seed_state_rendered` (front).

## S1a OSS Prior Art
- Domain: best-effort registry of content seeders + periodic re-announce.
- Sources (kickoff §3, re-verified pertinent, dates 2024-2025):
  - **IPFS provider records / reprovide** (docs.ipfs.tech, Kubo reprovider): a
    node periodically re-announces ("reprovides") the CIDs it holds to the DHT;
    records expire (default 22h-48h) so re-announce is a recurring best-effort
    cost, NOT a guaranteed global count. = the boot/periodic re-announce model +
    best-effort counter. APPROACH-ALIGNED.
  - **Radicle Heartwood seeding policy** (radicle.xyz/guides/seeder): a seeder
    replicates and *announces* refs it follows; `delegates != seeders` — the
    seeder cannot sign the canonical version. = `SeedAnnounced.seeder_node_id`
    DISTINCT from the app author; the seeder signs only its OWN seed claim, never
    the app provenance. APPROACH-ALIGNED.
  - **BitTorrent swarm / tracker scrape**: the seed count is an inherently
    approximate, point-in-time observation (peers come and go); no protocol
    guarantees an exact live count. = best-effort "Toi + N pairs (vus
    recemment)" with TTL (Checkpoint Q5). APPROACH-ALIGNED.
  - **Content-addressing (IPFS/iroh-blobs)**: a forged "I seed X" announcement
    cannot let a node SERVE X if it lacks the blob — the fetch verifies blake3,
    mismatch -> reject. = the security floor of an unauthenticated/over-countable
    registry (S3). APPROACH-ALIGNED.
- Finding: **APPROACH-ALIGNED** on all axes (raw-op observable fact + periodic
  re-announce + best-effort TTL count + content-addressed reachability truth).
  No LIB-EXISTS that would replace the feed (the feed is already the SBFB
  primitive; IPFS/Radicle are whole systems, not composable libs).
- Impact: none (no PLAN-ADAPT).

## S1b Dependencies, CVEs, Release Notes
- Scanned: no new crate. Reuses serde_json (`Value`), iroh-docs 0.98 (via
  `feed_sync` DocHandle), iroh-blobs 0.100 (content fetch in ingest), ed25519
  (feed signature via `nexus_core_rs::verify`, `public_feed.rs:514`).
- Commands/sources: Phase E preflight already ran `grep '^name = "iroh"' Cargo.lock`
  -> iroh 0.98.2 / iroh-blobs 0.100.0 (unchanged) and `cargo tree -d` -> no new
  duplicate. Phase F adds ZERO dependency (pure composition of public_feed +
  feed_sync + keep_online), so the transitive graph is byte-identical to `b76a084`.
- Finding: **clean**. No dep added/bumped -> no transitive collision possible
  (P2-PREFLIGHT-TRANSITIVE-DEPTH satisfied trivially: nothing to resolve). Carries
  P2-A-1 (rand upstream) and P2-AUDIT-2 (iroh pre-release transitives) remain
  external exemptions, untouched.

## S2 Historical Decisions
- Commands: `git log --oneline -- public_feed.rs feed_sync.rs` (via the landed
  S65-S73 history captured in CLAUDE.md) ; reverse-commit checks against the
  raw-op decision and the boot-restore decision.
- Decisions crossed:
  - **Raw-op extensibility** (`public_feed.rs:97-101, 137-144`, CLAUDE.md
    Pre-launch protocol policy): `FeedEntry.op` is `serde_json::Value`; "nodes
    store and propagate unknown operation types without interpretation
    (CloudEvents-style)". Adding `SeedAnnounced` is EXACTLY the sanctioned
    forward-compat path (the doc-comment at `public_feed.rs:77-79` literally lists
    "`SearchManifestPublished`) use the raw-op forward compat path (pattern P51)
    until implemented"). **0 bump is the documented, intended mechanism.** Reverse
    check: no commit forbids adding ops; `FEED_FORMAT_VERSION=1` is preserved by
    every prior op addition (CuratorVouched/CuratorDisendorsed S67 did NOT bump).
    Non-blocking (confirmed-intended, not a reversal).
  - **Boot re-announce = outbox replay** (`runtime.rs:1434, 1497-1504, 1528-1535`,
    hotfix #7 `def3008` / #8 `3b7ef54`): the boot path re-broadcasts the
    `gossip_outbox` (self-published PROJECT announcements), gated by
    `keep_online_allows_rebroadcast` (1809). The outbox holds ONLY self-published
    apps (deploy/publish call `insert_outbox`, `db.rs:668`). **Seeded distant apps
    have NO outbox entry** (the seed handler `seed_protocol.rs:253` sets
    `keep_online` but does NOT insert into the outbox — verified: no `insert_outbox`
    in `seed_protocol.rs`). So Phase F's "re-announce distant seeded apps" is a NEW
    path, not a modification of the outbox replay. This is consistent with the
    plan's intent and with NF-3 (FsStore persists the blob; F only re-announces).
    Reverse check: no commit says "only self apps re-announce" — the outbox replay
    simply predates cross-node seeding. Non-blocking (additive).
  - **keep_online stores (project_id, enabled, archive_hash, pinned_at)**
    (`db.rs:268-273`) — no `is_self`/`is_seeded` flag. See F-2 for how to
    distinguish self vs seeded WITHOUT a schema change.
- Finding: **clean** (no reversal of a frozen decision; both hooks are the
  intended forward-compat / additive paths).

## S3 Local Patterns And Threat Model
- Threats/contracts checked: T-FEED-FORGERY / T-FEED-INTEGRITY (THREAT_MODEL §10),
  Sybil (over-count), content-addressing (§5.4). FULL scan (new wire op +
  network-propagated registry).
- `SeedAnnounced` reuses the feed's existing per-entry Ed25519 + per-author
  hash-chain (`public_feed.rs:414-454, 488-528`) and the ingest gates
  (`feed_sync.rs:161-191`: verify_entry sig, timestamp +30d window, raw-op
  validate, PoW nonce). So a `SeedAnnounced` entry is signature-verified and
  PoW-rate-limited on ingest exactly like ReleasePublished — no new crypto path.
- Threat model — Seed registry surface:

  | Vector | Mitigation (file:line) | Residual |
  |---|---|---|
  | Forged `SeedAnnounced` (bad sig) | `verify_entry` Ed25519 against `author_pubkey` (`public_feed.rs:514`) on ingest (`feed_sync.rs:161`) | Nil (crypto) |
  | A node claims it seeds an app it does NOT hold (false count) | best-effort by design (Checkpoint Q5) + **content-addressing truth**: a false seeder cannot SERVE the blob (blake3 verify on fetch, §5.4) -> count may OVER-state, real reachability is verified at fetch (the ETAT probe is the source of truth, not the count) | M (over-count only; never a false "reachable") |
  | Sybil (one actor mints N fake seeder identities) | best-effort + closed pilot (kickoff §scope: 2-3 trusted) + feed PoW (`FEED_POW_DIFFICULTY=16`, `public_feed.rs:182`) raises per-entry cost; SearchManifest/network-wide registry is a DEFERRED scope cut (#10) precisely to avoid the broadcast-Sybil surface (D3 design note) | M (pilot-bounded; documented future gap, NOT a regression) |
  | Re-attribution of authorship (R5) | `SeedAnnounced.seeder_node_id` is the SEEDER key (== feed `author_pubkey`), DISTINCT from the app author; the seeder signs ONLY its seed claim, NEVER the app provenance (Radicle delegate!=seeder) | Nil (invariant) |
  | Stale/replayed count | TTL from `ts` (best-effort eviction) + the feed's +30d future-ts gate (`public_feed.rs:533-547`) | L (a real seeder that left stays counted until TTL — acceptable for best-effort) |
  | Oversized op DoS | `MAX_OPERATION_JSON_SIZE=65536` on validate (`public_feed.rs:253`) | Nil |

- HARDENING_ROADMAP status: no missing S74 pre-requirement. The seed registry is
  the LT-5 pull-forward (kickoff §1.2), not a late hardening debt.
- Finding: **clean (1 non-blocking)**. Non-blocking: add a THREAT_MODEL §10/§16
  note for the SeedAnnounced registry (over-count vs content-addressed truth) —
  route Phase G doc lot (already aligned with Phase E's deferred §16 "Seed
  surface" note). No T0-T5 regression: the registry reuses Ed25519 + PoW +
  content-addressing already at residual L.

## S4 Protocol And Wire Invariants
- Wire/security files checked: `public_feed.rs` (FEED_FORMAT_VERSION=1, raw-op
  `Value`, validate accepts unknown), `feed_sync.rs` (ingest verify gates),
  `db.rs` (M18/M19 LOCAL).
- VERSION/domain/canonical status:
  - `FEED_FORMAT_VERSION` stays **1** (`public_feed.rs:20`). `SeedAnnounced` is a
    raw-op `Value` carried by the UNCHANGED `FeedEntry` envelope -> **no bump**
    (CLAUDE.md Pre-launch policy: "Ajouter une nouvelle operation ... ne bump PAS
    FEED_FORMAT_VERSION"). Test `seed_announced_raw_op_no_version_bump` asserts
    `FEED_FORMAT_VERSION == 1` after building/ingesting a SeedAnnounced.
  - No `*_VERSION` / `DOMAIN_*` bump. SeedAnnounced is signed under the EXISTING
    `DOMAIN_FEED_V1` (`public_feed.rs:164`, via the feed canonical bytes) — it is
    a feed entry, NOT a new signing domain. (Contrast: Phase E's `SeedRequest`
    needed `DOMAIN_SEED_REQUEST_V1` because it is a SEPARATE ALPN message; the
    feed op rides the feed's domain.)
- Day 0 status: **preserved**. heberger != publier (the seeder signs a seed claim,
  not provenance); raw-op pre-launch extensibility; M18/M19 local; iroh 0.98.
- **Wire trace producer -> consumer (P2-PREFLIGHT-WIRE-CONTRACT-DEPTH)**, op
  `{op_type:"SeedAnnounced", project_id, seeder_node_id, archive_hash, ts}`
  (note: the per-entry `sig` is NOT a payload field — it is the FeedEntry
  `signature`; see F-3):

  | Field | Producer | Consumer | Exact shape |
  |---|---|---|---|
  | `op_type` | emit helper sets `"SeedAnnounced"` | `op_type(&op)` (`public_feed.rs:142`) routes ingest aggregate; `try_parse_op` returns None for the typed enum UNLESS a 5th variant is added (F-1) | JSON string key `"op_type"` |
  | `project_id` | blake3(name) of the seeded app | aggregate key (group seeders per project) | String (validate hex-64 IF typed — F-1) |
  | `seeder_node_id` | the seeder's node pubkey hex (== FeedEntry `author_pubkey`) | display "Toi + N pairs"; DISTINCT from app author | String hex-64 |
  | `archive_hash` | hex-64 of the seeded blob | correlate the seed with the app release | String hex-64 |
  | `ts` | now secs at emit | TTL "vus recemment" basis | u64 number |
  | (FeedEntry) `author_pubkey` | seeder keypair (`feed_sync.rs:487`) | `verify_entry` (`public_feed.rs:506`) | hex-64 on the envelope |
  | (FeedEntry) `signature` | `keypair.sign(canonical)` (`feed_sync.rs:501`) | `verify_entry` Ed25519 | hex-128 on the envelope |

  Consumer FRONT: the seed count is exposed via an HTTP route (NEW or extend
  browse JSON). `AvailabilitySheet.tsx` currently has a STATIC "Aucune copie de
  secours" (line 350) + inert "Bientot" invite (361-363). Phase F replaces the
  count text with a live "Toi + N pairs (vus recemment)" from the route; the
  invite CTA may stay "Bientot" (NF-2 authenticated-invite UI deferred). The Zod
  shape for the new count field MUST be modelled exactly as the daemon serializes
  it (number, always-present-or-absent — decide and assert, S73 Phase E lesson:
  always-present-as-0 -> non-optional; absent -> `.optional()`).
- Finding: **clean**. No pre-launch bump; the new wire op rides the unchanged
  feed envelope under its existing domain.

---

## F-1 (non-blocking, load-bearing) — typed enum variant vs pure raw-op `Value`

**The question the prompt asks, answered factually.** `FeedEntry.op` IS
`serde_json::Value` (`public_feed.rs:106`), NOT a typed enum field. The enum
`PublicFeedOperation` (`public_feed.rs:82-87`) is a CONVENIENCE for KNOWN ops:
`try_parse_op` (137) tries `serde_json::from_value` and returns `None` for
unknown `op_type`. `validate_feed_operation` (251) validates KNOWN ops via the
enum and **accepts unknown ops with a size check only** (259-262). The 0-bump
invariant is therefore satisfied **either way** — the wire-format version lives
on the `FeedEntry` envelope (`version` 104), not on the op union.

**Two valid implementations; tranche:**

- **(Option A, RECOMMENDED) Add a 5th typed variant `SeedAnnounced(SeedAnnouncedPayload)`
  to `PublicFeedOperation` + a validation arm.** This is what S67 did for
  `CuratorVouched`/`CuratorDisendorsed` — adding an enum variant does NOT bump
  `FEED_FORMAT_VERSION` (proven: it's still 1 after those additions). Benefit:
  `validate_feed_operation` enforces hex-64 on `project_id`/`seeder_node_id`/
  `archive_hash` and a sane `ts`, so a malformed SeedAnnounced is REJECTED at
  ingest (`feed_sync.rs:175`) instead of silently stored as opaque junk that
  pollutes the count. `op_type(&op)` (line 142) still works for routing.
  **This is the safer, in-pattern choice and what the plan's "agregat seed" needs.**

- **(Option B) Pure raw-op `Value` with no enum variant.** Build the op as
  `serde_json::json!({"op_type":"SeedAnnounced", ...})`, sign+insert via
  `insert_feed_operation`. It validates (size-only) and propagates. Ingest routes
  on `op_type(&op) == Some("SeedAnnounced")` and parses the fields ad-hoc. Works,
  0-bump, but LOSES the insert-time field validation (a peer could emit a
  SeedAnnounced with a 3-char project_id and it would be stored + counted).

**Tranche: Option A.** The plan §F.2 says "helper construire/valider
`SeedAnnounced` raw-op (pas une 5e variante d'enum)" — but the FACTUAL evidence
(CuratorVouched added as a variant in S67 with 0 bump) shows a variant is BOTH
0-bump AND gives validation. The "pas une 5e variante" framing in the plan is a
misreading of the 0-bump mechanism (it assumed a variant bumps the version; it
does not). **Recommendation: add the variant** for ingest validation, document in
the commit body that "a typed variant does not bump FEED_FORMAT_VERSION (S67
precedent)". If the PO prefers the plan's literal raw-op-only wording, Option B
is acceptable but must add explicit field validation in the ingest router to
avoid junk-count pollution. Either way: 0 bump, the test
`seed_announced_raw_op_no_version_bump` passes.

## F-2 (non-blocking, load-bearing) — self-published vs seeded at boot

**The prompt's hardest question, answered from the schema.** The boot re-announce
loop (`runtime.rs:1434, 1497, 1528`) replays the **outbox** (`gossip_outbox`,
`db.rs:668`), which holds ONLY self-published project announcements (deploy/publish
insert there; the seed handler does NOT — verified: no `insert_outbox` in
`seed_protocol.rs`). So at boot:

- **My published apps** -> outbox entry + provenance_record + keep_online row ->
  already re-announced (the existing PROJECT announcement replay, gated by
  `keep_online_allows_rebroadcast`).
- **Apps I seed (distant)** -> keep_online row (set at `seed_protocol.rs:253`) +
  blob in FsStore (survives reboot, NF-3) + **NO outbox entry** + **NO
  provenance_record** (I am not the author; provenance is the author's).

**Distinguisher — NO new column needed.** A seeded distant app is exactly:
a `keep_online` row WHERE there is no matching `provenance_records` row for that
`project_id` (provenance is keyed by project_id, `db.rs:181-194`, inserted only at
self-deploy). Equivalently: a keep_online row WHERE no `gossip_outbox` envelope
carries that project_id. **Tranche: derive it, do NOT add a flag.** Add a getter
e.g. `list_keep_online_enabled()` (mirror of `list_keep_online_disabled` 730) and,
for each enabled project_id, check `get_provenance_record(project_id).is_none()`
(or absence in the outbox) -> that's a SEEDED app -> emit/re-emit `SeedAnnounced`
at boot. This honours NF-3 (no ticket column) AND avoids a schema change M19-bis.
- Rationale for derive-not-flag: the `keep_online` PK is `project_id`, the
  provenance table is already keyed by `project_id`, so the join is O(1) per row
  and the truth is authoritative (provenance presence == authorship), not a flag
  that could drift. Adding an `is_seeded` column would duplicate state that is
  already derivable — same class of avoidable redundancy the project rejects.
- The boot SeedAnnounced re-emit reuses the feed emit helper (insert +
  publish_feed_entry_to_docs), NOT the outbox (the outbox is for PROJECT
  announcements; SeedAnnounced is a FEED op, propagated via iroh-docs, not gossip).
- Test `remote_seeder_reannounces_after_reboot_e2e` (§P57): a peer fetch+pins a
  distant app (keep_online set, no provenance), reboot, assert a SeedAnnounced is
  re-emitted to the feed for that project_id.

## F-3 (non-blocking) — SeedAnnounced signature uses the feed's own chain

The plan §F.1 lists a `sig` field inside the op `{...,ts,sig}`. **The op does NOT
carry its own `sig` field.** The signature is the FeedEntry-level `signature`
(`public_feed.rs:111`) over `DOMAIN_FEED_V1` canonical bytes
(`compute_feed_canonical_bytes` 171), produced by `insert_feed_operation`'s
`sign_fn` (the daemon keypair, `feed_sync.rs:501`) and verified by `verify_entry`
(`public_feed.rs:488`). So: `seeder_node_id` == the FeedEntry `author_pubkey` ==
the daemon's node key. This is COHERENT with feed verification — each seeder forms
its OWN per-author hash-chain (`verify_chain` is multi-author, `public_feed.rs:557`),
distinct from the app author's chain. **Tranche: do NOT add a `sig` field to the
op payload** (it would be redundant and would have to be EXCLUDED from canonical
bytes, complicating the hash). Reuse the feed signature. Document in the commit
body that "SeedAnnounced is signed by the seeder via the standard FeedEntry
Ed25519 chain; seeder_node_id == author_pubkey, DISTINCT from the app author
(Radicle seeder!=delegate)".

## F-4 (non-blocking) — counter location + TTL + route

- **Aggregate location**: in-memory `Arc<Mutex<HashMap<project_id, HashMap<
  seeder_node_id, last_seen_ts>>>>` (mirror of the nonce cache pattern Phase E
  used, and of `BrowseRequestLimiter`/`browse_aggregator` in-memory state
  `runtime.rs:1443`). Fed by the ingest path (`feed_sync.rs` SeedAnnounced branch)
  AND by the local self-seed (so "Toi" is counted). NOT a DB table — a seeder
  count "vu recemment" has no value outside its freshness window (same reasoning
  Phase E used for the nonce cache: ephemeral observation).
- **TTL**: an entry expires when `now - last_seen_ts > SEED_SEEN_TTL`. The plan
  says "vus recemment" — propose `SEED_SEEN_TTL = 48h` aligned with the IPFS
  reprovide window (a seeder that has not re-announced in 48h is presumed gone).
  Lazy purge on read (drop expired before counting), like the nonce cache. The
  count is best-effort (Checkpoint Q5 / scope cut #11: no exact number).
- **Route**: extend the existing browse JSON (each BrowseEntry already carries
  status/last_probed_at) with a `seed_count` (number, the count of distinct
  non-expired seeders for that project_id, INCLUDING self if self seeds), OR a
  dedicated `GET /api/daemon/seed-count?project_id=`. Tranche: extend the browse
  JSON (one round-trip, the front already invalidates `["daemon-browse"]` after
  seedVoluntary, `AvailabilitySheet.tsx:122`). Front Zod: add `seed_count: number`
  to the BrowseEntry schema (always-present-as-number -> non-optional; if the
  daemon omits it for apps with no SeedAnnounced, use `.optional()` with a `?? 0`
  fallback — DECIDE at impl and assert the exact shape, S73 Phase E lesson).
- Test `seed_count_best_effort_ttl_expires`: insert two SeedAnnounced (one fresh,
  one with ts older than TTL), read count -> 1 (the stale one evicted).

## F-5 (non-blocking) — front "Copies de secours" wiring

`AvailabilitySheet.tsx:344-365` currently shows a static "Aucune copie de
secours" (350) and an inert "Bientot" invite CTA (354-363). Phase F:
- Replace the static text with the live count: "Toi + N pairs (vus recemment)"
  when `seed_count > 1` (self + N), or "Aucune copie de secours" when the user is
  the sole seeder / count==1, derived from the new `entry.seed_count`.
- Keep "Inviter un pair de confiance" as **"Bientot"** inert (NF-2: the
  authenticated-invite address-entry UI is deferred; the voluntary path is the
  shipped one). This preserves verrou §8(5) (no faux active button).
- Test `multi_seed_state_rendered`: render with `seed_count=3` -> "Toi + 2 pairs"
  visible; `seed_count=1` -> "Aucune copie de secours".

## Risks And Scope Cuts
- Blocking risks: **none** (verdict SCOPE-CUT-CONSISTENT).
- Non-blocking risks / findings:
  - F-1: typed variant vs raw-op -> Option A (typed variant, 0-bump per S67
    precedent, gains ingest validation); plan's "pas une 5e variante" wording is
    a misreading of the 0-bump mechanism — flag in commit body.
  - F-2: self vs seeded at boot derived from `keep_online row WITHOUT
    provenance_record` (no new column, honours NF-3); boot SeedAnnounced re-emit
    is a NEW feed-emit path (NOT the outbox replay).
  - F-3: SeedAnnounced signed by the feed's own per-author Ed25519 chain; NO `sig`
    field in the op payload (seeder_node_id == FeedEntry author_pubkey).
  - F-4: in-memory aggregate + TTL (48h proposed) + browse-JSON `seed_count`.
  - F-5: front count live; invite CTA stays "Bientot" inert.
  - Plan path bug: `nexus-shell-daemon/src/public_feed.rs` does not exist; real
    file is `nexus-coordinator-rs/src/public_feed.rs` + `feed_sync.rs` ingest.
  - Test-count baseline: the Phase E preflight noted plan §1's 1570/294 is STALE;
    re-measure on `b76a084` before computing the Phase F delta (Phase E body
    measured ~1639+ Rust / ~313 Vitest — the main thread MUST re-measure).
  - THREAT_MODEL §10/§16 SeedAnnounced over-count note -> Phase G doc lot.
- Scope cuts honored (kickoff §7): #4 re-allocation/failover -> post-launch;
  #5 timer 22h re-announce -> post-launch (F re-announces at BOOT, not on a timer);
  #10 SearchManifest network-wide registry -> post-launch (F is feed-local-replicated,
  D3); #11 exact count -> best-effort "Toi + N pairs" (Q5). D5: if F overruns it
  stays "Bientot" inert (never a faux active button).
- Day 0 preserved: heberger != publier (seeder signs seed claim, not provenance);
  raw-op pre-launch, FEED_FORMAT_VERSION=1; M-local (M18/M19); iroh 0.98.

## Action
- **SCOPE-CUT-CONSISTENT**: proceed with Phase F. The commit body MUST cite this
  file (G8 traceability) and carry findings F-1..F-5 + the plan path correction +
  the test-count re-measure note.
- Tranches to honour: F-1 Option A (typed variant), F-2 derive self-vs-seeded from
  provenance absence (no column), F-3 reuse feed signature (no op `sig` field),
  F-4 in-memory TTL aggregate + browse-JSON count, F-5 invite stays "Bientot".
- Carry-over to Phase G: THREAT_MODEL SeedAnnounced over-count note.
- No wire bump; SeedAnnounced rides the unchanged FeedEntry envelope under
  DOMAIN_FEED_V1; FEED_FORMAT_VERSION stays 1.
