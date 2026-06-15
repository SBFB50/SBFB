# Sprint 75 Phase A Preflight

Date: 2026-06-09
HEAD: `a9a2ea7`
Verdict: **SCOPE-CUT-CONSISTENT**

> FIX-A re-mint-on-replay. No blocking finding. The frozen D2 approach is
> confirmed correct by mature OSS prior art (IPFS reprovide, BEP-44, Nostr
> NIP-65) and by SBFB's own anti-replay rationale. Three non-blocking
> completeness notes raise the touched-site count above the plan's framing
> (an extra decode site + the carrier/test shape) — they widen the diff, they
> do not change the design. Proceed with Phase A as designed, applying the
> completeness deltas below.

## Evidence Rules
- Claim policy: every claim cites a path, a command/grep output, a URL+date, or
  an explicit assumption.
- Local sources read: `prompts/agent/preflight.md`,
  `.planning/active/sprint75_plan.md` (§Phase A), `.planning/active/sprint75_kickoff.md`
  (§5 D2, §4 verrous), `crates/nexus-core-rs/src/pow.rs` (full),
  `crates/nexus-core-rs/src/pow_gossip.rs` (full),
  `crates/nexus-shell-daemon/src/runtime.rs` (:1040-1067, :1460-1637, :1703-1799,
  :1818-1897), `crates/nexus-shell-daemon/src/deploy.rs` (:600-714),
  `crates/nexus-shell-daemon/src/http.rs` (:915-1016, :1625-1662, :2872-2911),
  `crates/nexus-shell-daemon-core/src/publish.rs` (:20-216),
  `crates/nexus-coordinator-rs/src/db.rs` (:128-156, :656-681),
  `crates/nexus-shell-daemon/src/local_worker.rs` (:250-289),
  `crates/nexus-core-rs/src/discovery.rs` (grep :84/:122/:144), `Cargo.lock`,
  `docs/security/THREAT_MODEL.md` (§5.4, §15), memory `feedback_approach.md`,
  `feedback_context7_systematic.md`.
- Commands run: `git rev-parse --short HEAD` -> `a9a2ea7`;
  `git log --oneline -- crates/nexus-core-rs/src/pow.rs` (S19/S23/S24/S54, no
  weakening); Grep `^name = "iroh"` Cargo.lock -> iroh `0.98.2`, iroh-blobs
  `0.100.0`, rusqlite `0.36.0`; WebSearch IPFS reprovide / BEP-44 / NIP-65
  (2026-06-09).

## Scope
- Plan source: `.planning/active/sprint75_plan.md` §"Phase A — FIX-A
  re-mint-on-replay (D2)", A.1-A.5.
- Target files (plan A.2): `runtime.rs` (outbox store + 3 replay sites
  :1513/:1544/:1615 + restore :1876-1897), `deploy.rs` (:661-687),
  `http.rs` (re-mint helper near `mint_blob_ticket` :1639-1662), `pow.rs`
  (read-only, window unchanged), `runtime.rs` tests.
- Deps/APIs/specs: iroh 0.98 `EndpointAddr` + `BlobTicket::new` +
  `my_endpoint_addr()`; `PowSolveCache`/`PowEnvelope` (nexus-core-rs); rusqlite
  outbox BLOB. **No Cargo.toml change in Phase A** (no new dep).
- Security/protocol surfaces: PoW anti-replay/anti-flood/liveness window
  (`MAX_PROOF_AGE_SECS=1800`); gossip `ProjectAnnouncement` wire (v1); local
  outbox storage form; THREAT_MODEL §5.4 (iroh stack) + §15 (seed surface,
  seeder!=auteur attribution).
- Tests expected (plan A.3): `outbox_stores_unwrapped_payload`,
  `replay_rewraps_with_fresh_pow`, `replay_remints_endpoint_addr`,
  `stale_announcement_accepted_by_fresh_receiver` (the live bug),
  `remint_helper_reused_shape`, E2E `cross_machine_discovery_after_30min`
  (G gate).

## S1a OSS Prior Art
- Domain: keeping a signed/PoW-stamped, freshness-bounded announcement alive
  across re-publish in a decentralized network — re-mint vs widen-window.
- Sources (accessed 2026-06-09):
  - IPFS Kubo provider records: republish 22h / expire 48h; a reprovide cycle
    re-advertises a **fresh** provider record, it does not replay an expired
    stamp. https://github.com/ipfs/kubo/issues/9389 ;
    https://ipshipyard.com/blog/2025-dht-provide-sweep/
  - BitTorrent BEP-44 mutable items: expire after 1h; nodes "periodically
    re-announce by replaying the put message", and the put carries `seq`+`sig`
    so freshness/authority is re-established at each republish (not a stale
    proof accepted past expiry). https://www.bittorrent.org/beps/bep_0044.html
  - Nostr NIP-65 kind:10002 replaceable events: "newest wins" by `created_at`;
    "the replacement mechanism prevents replay attacks by design — the older
    version is discarded". https://nips.nostr.com/65 ; https://nips.nostr.com/1
- Finding: **APPROACH-ALIGNED**. The three references converge unanimously:
  freshness is re-established by **re-stamping at re-publish**, never by widening
  the acceptance window and never by replaying a stale stamp. This is exactly
  D2's `re-mint a fresh PoW (fresh issued_at) at replay` and exactly D2's
  rejection of `weaken MAX_PROOF_AGE_SECS`. The rejected alternative (widen the
  window) is the one the OSS evidence shows is wrong (it would keep dead
  records discoverable, the IPFS-expiry / BEP-44-expiry anti-pattern).
- Impact: none. Confirms the plan. (S1a blocking would map to PLAN-ADAPT; not
  triggered.)

## S1b Dependencies, CVEs, Release Notes
- Scanned: iroh, iroh-blobs, rusqlite, nexus-core-rs PoW primitives.
- Commands/sources: Grep `Cargo.lock` -> iroh **0.98.2**, iroh-blobs
  **0.100.0**, rusqlite **0.36.0** (the workspace pin per CLAUDE.md Day 0:
  iroh 0.98 / iroh-blobs 0.100). Phase A.2 lists no `Cargo.toml` edit; the plan
  explicitly says "Aucune nouvelle dep attendue".
- Transitive-depth (P2-PREFLIGHT-TRANSITIVE-DEPTH): Phase A adds/bumps **zero**
  dependencies, so no transitive graph change is possible and `cargo tree -d`
  is not load-bearing here. The S72 ollama-rs/schemars collision class cannot
  recur in a phase that edits no manifest. If implementation discovers a needed
  dep (not expected), re-run `cargo tree -d` before declaring S1b clean.
- API stability: the re-mint helper re-uses symbols **already compiled in
  production** under the frozen iroh 0.98 pin — `DiscoveryClient::my_endpoint_addr`
  (`discovery.rs:84`), `BlobTicket::new(addr, hash, BlobFormat::Raw)` and
  `my_endpoint_addr()` inside `mint_blob_ticket` (`http.rs:1657-1660`). FIX-A
  re-calls these at replay time; it introduces no new iroh symbol and no version
  change. (context7 systematic rule: the locked, compiling 0.98.2 production
  call sites are stronger evidence of API shape than a context7 snapshot of a
  *different* version — Context7's indexed iroh is v0.95, older than the pin —
  so no doc-vs-code drift risk exists for symbols already in the tree.)
- Finding: **clean**. No CVE surface introduced (no new dep, no crypto/wire
  primitive change). The PoW SHA256 Hashcash primitive is unchanged.

## S2 Historical Decisions
- Commands: `git log --oneline -- crates/nexus-core-rs/src/pow.rs` ->
  `edfc51b` (S19 Phase B introduce), `6102dc2` (S23 escalating ramp),
  `ff4c7d5` (S24 cleanup), `1d010b0` (S54 edition 2024) — every touch is
  additive; **no commit ever weakened `MAX_PROOF_AGE_SECS` or the freshness
  bound**. Reverse-commit check: no `git log <sha>..HEAD` candidate reverses
  the S19 30-minute policy; the constant is `1_800` today (`pow.rs:109`).
- Decisions crossed:
  - **S19 PoW window rationale** (`pow.rs:24-31, 105-109`,
    HARDENING_ROADMAP §3 S19 item 1): the `issued_at` / `MAX_PROOF_AGE_SECS`
    bound exists for **three** intents — (1) anti-replay ("a captured solution
    cannot be replayed indefinitely"), (2) anti-flood (per-`(pubkey,topic)`
    cost, escalating ramp S23), (3) liveness/freshness signal (a proof attests a
    live, recent publisher). FIX-A **preserves all three**: re-minting produces
    a *genuinely fresh* proof from a *live* publisher who *pays* the cost again
    (PowSolveCache makes the re-pay ~free within the 15-min session window but
    the proof's `issued_at` is current) — this is a legitimate new proof, not a
    bypass. The window stays `1800`.
  - **Hotfix #7** (`def3008`, `restore_browse_from_outbox` runtime.rs:1876,
    doc :1807-1812): boot-restore decodes the outbox **without PoW
    re-verification** — "these are our own trusted envelopes, and a
    difficulty-policy bump since they were minted must not drop them". This is
    the precedent that grounds FIX-A's S3 trust boundary (own outbox = trusted
    local input; PoW guards the untrusted *network* input). Not reverted; FIX-A
    extends the same OWN-only trust assumption to the re-mint step.
  - **Hotfix #8** (`deploy.rs:619-687`): the single canonical
    announce->broadcast->persist-to-outbox path. FIX-A modifies the persist
    *form* (envelope -> unwrapped payload) but keeps the single-path invariant.
- Finding: **clean** (confirmed-reversion / valid-rationale-preserved). FIX-A
  honors the S19 rationale rather than contradicting it. Non-blocking.

## S3 Local Patterns And Threat Model
- Threats/contracts checked: THREAT_MODEL §5.4 iroh stack (S: byzantine forged
  announcement; D: gossip flood), §15 seed surface (S: impersonation /
  seeder!=auteur; I: author re-attribution), PoW anti-replay/anti-flood/liveness.
- HARDENING_ROADMAP status: S19 PoW is a *delivered* mitigation, not a pending
  pre-requirement for S75; FIX-A does not regress it. No S75 HARDENING
  pre-requirement is unmet by Phase A.
- Analysis of the re-mint surface:
  - **Re-mint does NOT re-open anti-replay** because the actor re-stamping is
    the node re-broadcasting **its OWN outbox**. The outbox holds only
    self-published announcements: the deploy/publish paths push via
    `GossipCmd::Outbox` (`deploy.rs:682-685`, `http.rs` publish), while
    third-party announcements arriving over gossip go to
    `handle_project_announcement` -> aggregator (`runtime.rs:1521-1527,
    :1718-1799), **never to the outbox**. A node re-minting its own proof is the
    same trust class as hotfix #7's PoW-skip restore. The network-facing
    anti-replay (`verify_envelope` at :1488-1500) is **unchanged** and still
    rejects a stale stamp from an untrusted peer.
  - **Re-minting the address is attribution-safe (verrou 4, seeder!=auteur)**
    *because the outbox is OWN-only*. The re-mint points `archive_ticket`'s
    `EndpointAddr` at `my_endpoint_addr()` of the **re-broadcasting node, which
    is the original author** for every outbox entry. There is no path for node X
    to re-point node Y's announcement: Y's announcement never enters X's outbox.
    NON-BLOCKING INVARIANT TO PRESERVE (record in commit body + a guard test):
    if any future change ever relays third-party announcements into the outbox,
    re-mint would let a relayer hijack the address/`node_id` pointer — so the
    re-mint helper must remain confined to the OWN-outbox replay sites and must
    never be applied to ingested third-party announcements. Pin this with a test
    asserting a re-minted ticket carries the local endpoint id, and a comment at
    each call site.
  - **`ProjectAnnouncement` has no Ed25519 signature field** (`publish.rs:32-87`
    — only `v`, `type`, `node_id` string, ...). The THREAT_MODEL §5.4 row S
    claim "Annonces signees Ed25519" overstates the current
    `ProjectAnnouncement` (its only authenticity layer is the PoW envelope's
    `publisher_pubkey`, plus content-addressing BLAKE3 on the actual blob fetch).
    FIX-A does not change this posture (it neither adds nor removes a signature),
    but the re-mint preserves the existing property that the *broadcaster* pays
    PoW; the `node_id` string remains self-asserted exactly as today. Since the
    blob fetch verifies BLAKE3 (§15 invariant: "annonce forgee ne sert jamais
    d'octets absents"), a wrong address only fails the dial, never serves wrong
    bytes. NON-BLOCKING doc note: do not let the commit/THREAT_MODEL claim
    "signed announcement" for `ProjectAnnouncement`; the integrity authority is
    PoW-cost + BLAKE3, matching the §15 framing.
- Finding: **clean with two non-blocking invariants to record** (OWN-only
  re-mint confinement; PoW+BLAKE3 not signature is the integrity authority). No
  regression of a covered T0-T5 threat. Non-blocking.

## S4 Protocol And Wire Invariants
- Wire/security files checked: `pow.rs` (POW_FORMAT_VERSION),
  `pow_gossip.rs` (PowEnvelope framing), `publish.rs`
  (PROJECT_ANNOUNCEMENT_VERSION + `to_gossip_bytes`/`from_gossip_bytes`),
  `canonical.rs` (DOMAIN_* table), `db.rs` (M6 `gossip_outbox` schema).
- VERSION/domain/canonical status:
  - `POW_FORMAT_VERSION = 1` (`pow.rs:85`) — **unchanged**.
  - `PROJECT_ANNOUNCEMENT_VERSION = 1` (`publish.rs:24`) — **unchanged**;
    `from_gossip_bytes` still enforces `v == 1` (`publish.rs:183`).
  - No new `DOMAIN_*` (Phase A adds no canonical type; the DOMAIN table
    `canonical.rs:71-219` is untouched).
- Producer -> consumer trace (P2-PREFLIGHT-WIRE-CONTRACT-DEPTH):
  - **What changes is LOCAL STORAGE only.** Producer side: today
    `deploy.rs:673-685` and `http.rs` publish wrap the announcement
    (`wrap_payload_with_pow`) and persist the **wrapped `PowEnvelope` bytes** as
    the `gossip_outbox.envelope BLOB` (`db.rs:131-136 M6`,
    `insert_outbox` :668-677). FIX-A persists the **unwrapped
    `ProjectAnnouncement` gossip bytes** instead. `gossip_outbox` is a
    same-process SQLite table (`load_outbox` :656-665) — it never crosses a node
    boundary, so it is NOT a wire format and carries no `*_FORMAT_VERSION`
    contract. 0 bump.
  - **What stays identical is the WIRE.** Consumer side: a remote receiver runs
    `pow_verify_cache.verify_envelope` (`runtime.rs:1488-1500`) then
    `ProjectAnnouncement::from_gossip_bytes`. After FIX-A, replay re-wraps the
    stored payload via `wrap_payload_with_pow` (the existing encoder,
    `http.rs:948-967` / `runtime.rs:1703-1716`) before `sender.broadcast`. The
    bytes on the wire are byte-identical in shape to today: `[u32 proof_len]
    [proof json][ProjectAnnouncement v1 json]`. The receiver sees **no format
    difference** — only the `issued_at` inside the proof and the `EndpointAddr`
    inside the `archive_ticket` are fresher. Confirmed unchanged consumer:
    `from_gossip_bytes` (publish.rs:181), `PowEnvelope::decode`
    (pow_gossip.rs:162), `handle_project_announcement` (runtime.rs:1718).
- Legacy-decode zombie check: Grep for outbox/legacy/decode tests found **no**
  "legacy decode" outbox test (the only outbox tests are
  `db.rs::insert_and_load_outbox`/`clear_outbox`/`outbox_survives_reopen`, which
  test BLOB persistence generically and are form-agnostic — they pass arbitrary
  `b"envelope-N"` bytes, not a PoW envelope, so they do NOT become zombies).
- Day 0 status: **preserved**. D2 (re-mint, window 1800 unchanged), D5 (additive
  0-bump). No `*_FORMAT_VERSION` / `*_ANNOUNCEMENT_VERSION` bump (pre-launch
  policy honored). No `default_*` anchor introduced (lock-3 tripwire N/A to
  Phase A).
- Finding: **clean** (legitimate local-storage reshape; wire unchanged, both
  ends read). Non-blocking. (S4 blocking would map to DESIGN-CONFLICT; not
  triggered.)

## Plan Adaptation
Not a PLAN-ADAPT (S1a is APPROACH-ALIGNED). The following are
**SCOPE-CUT-CONSISTENT completeness deltas** — the plan's file list under-counts
the decode/carrier sites coupled to the storage-form change. They widen the diff
within the same design; record them so the review/Codex gate does not flag a
"missed site":

1. **4th decode site, not 3+restore.** `keep_online_allows_rebroadcast`
   (`runtime.rs:1825-1847`) currently `PowEnvelope::decode(envelope)` then
   `from_gossip_bytes(payload)` to read the `project_id` for the Phase-D OFF
   gate. If the outbox stores the unwrapped payload, this function must drop the
   `PowEnvelope::decode` step and read `from_gossip_bytes(envelope)` directly.
   The plan A.2 lists "3 sites replay + restore" but this OFF-gate decode is a
   fifth coupled read. Touch it in the same commit or the OFF gate silently
   regresses (every entry would fail to decode and fall through to "replay-all",
   re-broadcasting OFF apps).

2. **Carrier + restore consume the new form.** `GossipCmd::Outbox(envelope)`
   (`runtime.rs:1572-1587`) pushes to both `insert_outbox` and the in-memory
   `outbox: Vec<Vec<u8>>`, and the replay sites call `sender.broadcast(envelope
   .clone())` directly. With unwrapped storage, the replay sites must
   `wrap_payload_with_pow_static(...)` (fresh PoW) + re-mint the ticket *before*
   broadcast, and `restore_browse_from_outbox` (`runtime.rs:1876-1897`) must
   call `from_gossip_bytes` directly (no `PowEnvelope::decode`). Already implied
   by D2; making it explicit in the file list avoids a "broadcast stored bytes
   verbatim" oversight.

3. **Existing http.rs outbox test adapts (not a zombie).**
   `publish_announcement_persists_to_outbox_for_replay` (`http.rs:2872-2911`)
   asserts the carried command decodes via `PowEnvelope::decode`. After FIX-A it
   should assert the carrier holds the **unwrapped** `ProjectAnnouncement` bytes
   (and a separate test asserts replay re-wraps with fresh PoW). Add
   `crates/nexus-shell-daemon/src/http.rs` (tests) to A.2's touched-files. This
   is a live-invariant update, not a pre-launch legacy zombie to delete.

4. **Re-mint helper reuse handle.** The plan extracts `remint_*` near
   `mint_blob_ticket` (`http.rs:1639-1662`) `pub(crate)` for Phase C. Note the
   *address* freshness is already implemented inside `mint_blob_ticket` (it
   calls `my_endpoint_addr()` at mint time, `http.rs:1657`): the helper is a
   thin re-call of that path keyed by `archive_hash`, plus the OWN-only guard
   from S3. No new iroh surface.

## Risks And Scope Cuts
- Blocking risks: **none**.
- Non-blocking risks / carry:
  - Preserve OWN-only re-mint confinement (S3) — guard test + call-site comment.
  - Do not claim "signed ProjectAnnouncement" (S3) — integrity authority is PoW
    cost + BLAKE3 (THREAT_MODEL §15 framing); optionally correct §5.4 row S
    wording opportunistically, else leave for Phase G doc hygiene.
  - 4th decode site (`keep_online_allows_rebroadcast`) + http.rs test must be
    in-scope of the Phase A commit (completeness delta 1 & 3).
  - E2E `cross_machine_discovery_after_30min` is the live-bug acceptance; plan
    A.3 #6 unit-simulates here and gates the real cross-machine run in Phase G
    (C6 gate: A E2E green BEFORE pull is gated on it).
- Scope cuts still honored (kickoff §9): no `*_FORMAT_VERSION` bump (#7); window
  1800 not weakened (D2); no SearchManifest / no new DOMAIN in Phase A (#1, #12);
  no anchor hard-coded (#lock-3, N/A Phase A). Phase A is dette/fix-only,
  independent of the pivot, landed first (kickoff §7, D2).

## Action
- **SCOPE-CUT-CONSISTENT**: proceed with Phase A as designed (D2 re-mint, window
  unchanged), applying the four completeness deltas above. The commit body must
  cite this preflight, record the two S3 invariants (OWN-only re-mint;
  PoW+BLAKE3 not signature), and document the 0-bump local-storage reshape
  (only the WHEN of the mint changes, the wire is byte-shape-identical). No
  pivot proposal required (no DESIGN-CONFLICT).
