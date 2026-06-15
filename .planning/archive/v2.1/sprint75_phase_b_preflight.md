# Sprint 75 Phase B Preflight

Date: 2026-06-09
HEAD: `96943b7`
Verdict: **EXECUTE**

## Evidence Rules
- Claim policy: every claim below cites a path, command output, URL/date, or
  explicit assumption.
- Local sources read:
  - `prompts/agent/preflight.md` (procedure portable, source of truth)
  - `.planning/active/sprint75_plan.md` (§Phase B `:98-135`), `sprint75_kickoff.md`
    (§4 `:116-158`, §5 D1 `:165-186`, §10 Q1/Q2 `:336-353`),
    `sprint75_pivot_proposal.md`, `sprint75_design_review.md` (D1 + C1-C6)
  - `crates/nexus-core-rs/src/curator.rs` (CuratorList + CuratorListEntry +
    caps + `:589-602` domain-sep test)
  - `crates/nexus-core-rs/src/canonical.rs` (`:71-219` all 17 domain tags +
    `canonical_bytes` `:240-248`)
  - `crates/nexus-core-rs/src/seed.rs` (`:1-117` S74 sibling-type precedent)
  - `crates/nexus-core-rs/src/lib.rs` (re-export pattern `:63-154`)
  - `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` (`:1-200` envelope +
    `:460-590` ingest 9-step arm — the helper target)
  - `crates/nexus-shell-daemon-core/src/browse.rs` (`:108-298` BrowseSource +
    BrowseEntry + aggregator; `:180-196` `node_id #[serde(skip)]`)
  - `crates/nexus-shell-daemon-core/src/config.rs` (`:238-310` `default_curators`)
  - `crates/nexus-shell-daemon/src/http.rs` (`:1632-1654` `mint_blob_ticket` +
    Phase A `mint_ticket_for_hash`; `:294-295` route table; `:969-1026` publish)
  - `crates/nexus-core-rs/src/blobs.rs` (`:150-210` `fetch_ticket` / `fetch_and_pin`)
  - `docs/security/THREAT_MODEL.md §15` (`:825-859` seed cross-node surface)
  - memories: `feedback_approach.md`, `feedback_context7_systematic.md` (routing)
- Commands run (relevant outputs inline below):
  - `git log --oneline -8 -- canonical.rs curator.rs seed.rs` → `DOMAIN_SEED_REQUEST_V1`
    landed `b76a084` (S74 Phase E), exact sibling-type precedent.
  - `grep -rhoE 'b"nexus-[a-z0-9-]+"' canonical.rs | sort -u` → 17 tags, listed in S4.
  - `git show 376bfe2 --format=%B` (hotfix #6) → rationale for `node_id #[serde(skip)]`.
  - `grep -nE "default_curators" config.rs` + Read `:245-251` → defaults empty.
  - `grep -nE "serde_big_array|serde_jcs|ed25519|blake3" nexus-core-rs/Cargo.toml`
    → all 4 crypto deps already direct workspace deps; Phase B adds none.

## Scope
- Plan source: `.planning/active/sprint75_plan.md` §Phase B `:98-135` (B.1-B.5).
- Target files:
  - NEW `crates/nexus-core-rs/src/node_directory.rs` — `NodeDirectoryEntry`
    struct + `CatalogApp` + sign/verify + caps.
  - `crates/nexus-core-rs/src/canonical.rs` — add `DOMAIN_NODE_DIRECTORY_V1`.
  - `crates/nexus-core-rs/src/lib.rs` — re-export.
  - `crates/nexus-shell-daemon/src/http.rs` — `POST /api/daemon/directory/publish`
    (build+sign+blob-store+gossip-announce OWN catalog; loopback auth).
  - `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` — generic helper
    `ingest_signed_list<T: SignedList>` factoring the 9-step gate (`:500-590`).
- Deps/APIs/specs: **none added**. Reuses `serde_jcs`, `ed25519-dalek`, `blake3`,
  `serde_big_array` (all already direct deps of `nexus-core-rs`, Cargo.toml `:30-33`).
- Security/protocol surfaces: NEW signing domain `DOMAIN_NODE_DIRECTORY_V1`; NEW
  signed wire type `NodeDirectoryEntry`; NEW authoring HTTP route (loopback);
  generic ingest refactor of an existing security-gated arm.
- Tests expected (plan B.3): `node_directory_sign_verify_roundtrip`;
  `node_directory_cross_domain_replay_rejected`; `node_directory_caps_enforced`
  (256 + per-field); `node_directory_revision_monotone_rollback`;
  `publish_directory_route_signs_and_announces`; `generic_ingest_helper_parity`.

## S1a OSS Prior Art
- Domain: self-published, signed, replaceable per-publisher node/relay/repo
  directory ("here is what I host / where I write"), human-renderable index,
  monotone-revision/replaceable semantics, DoS-capped.
- Sources (accessed 2026-06-09):
  - Nostr **NIP-65** kind:10002 Relay List Metadata — replaceable event
    (newest from same author replaces older), clients SHOULD keep the list
    small, spread to well-known indexers. `https://nips.nostr.com/65`,
    `https://github.com/nostr-protocol/nips/blob/master/65.md`. No 2025/2026
    spec break found; structure (per-author replaceable list) unchanged.
  - **Radicle Heartwood** Inventory + Reference Announcements — each
    announcement carries originating Node ID + Ed25519 signature + timestamp;
    peers verify before relaying; **strict size limit** on protocol messages
    (a serialize panic was fixed by enforcing the cap). Radicle 1.4.0 (2025-09).
    `https://docs.radicle.xyz/guides/protocol`, `https://radicle.xyz/2025/09/04/radicle-1.4.0`.
  - **F-Droid** index-v2 — signed per-repo index (JAR + GPG over `entry.json`),
    per-app metadata carries hashes of the app + its signing key; "repo =
    single signing key, TOFU, custom repos first-class equals".
    `https://f-droid.org/en/docs/Security_Model/`,
    `https://f-droid.org/en/2023/03/01/new-repo-format-faster-smaller-updates.html`.
  - **BEP-44** DHT mutable items — signed by public key, **cap ~1000 bytes**,
    `seq` monotone counter for replacement, periodic re-announce.
    `https://www.bittorrent.org/beps/bep_0044.html` (kickoff §0).
- Finding: **APPROACH-ALIGNED.** The plan's `NodeDirectoryEntry` shape matches
  mature OSS practice on every axis the board claimed (kickoff §0, design_review
  D1):
  1. **Own type per publisher, not overloaded curation** — NIP-65, Radicle
     INVENTORY, F-Droid index, BEP-44 each gave self-publication a dedicated
     type; none overloaded a vouching/curation primitive. The plan's "sibling
     type, not overload `CuratorList`" (D1, plan `:108`) is the prior-art
     consensus.
  2. **Monotone replacement** — NIP-65 "replaceable", BEP-44 `seq`, Radicle
     timestamp/signature. The plan reuses `CuratorList.revision` monotone
     rollback protection (curator.rs `:121-126`, runtime step 8 `:571-579`).
     ALIGNED — note OSS variance on revision-vs-timestamp (BEP-44 `seq` counter
     like ours; Radicle uses timestamp). Our **counter** choice is the stricter
     of the two (no clock dependency) and is the existing repo invariant; no
     reason to switch.
  3. **Human-renderable list, not a coverage digest** — F-Droid per-app metadata
     is a displayable catalog; the pivot deliberately rejects the Bloom/Merkle
     digest (that is the deferred SearchManifest, scope cut #12). ALIGNED with
     "you cannot render F-Droid cards from a Bloom filter" (pivot_proposal §3).
  4. **Hard size cap** — Radicle's strict message cap and BEP-44's 1000-byte cap
     confirm the `CURATOR_LIST_MAX_ENTRIES=256` + per-field caps reuse is the
     correct, prior-art-backed DoS posture (curator.rs `:63-90`).
- Impact: none. No format detail evolved in 2025/2026 that invalidates a frozen
  choice. The one OSS variance (revision-vs-timestamp) is already resolved in
  favor of the stricter counter we inherit. `LIB-EXISTS` does NOT apply: this is
  an internal SBFB wire type over our own `canonical_bytes`/iroh stack; no
  external crate covers it. context7 not queried for the type itself (pure
  internal `nexus_core_rs::canonical`/`curator` machinery, per
  `feedback_context7_systematic.md` "When NOT to query").

## S1b Dependencies, CVEs, Release Notes
- Scanned: `serde_jcs`, `ed25519-dalek`, `blake3`, `serde_big_array`, `serde`,
  `hex` — the crypto/serialization surface `NodeDirectoryEntry` reuses.
- Commands/sources:
  - `grep -nE "serde_big_array|serde_jcs|ed25519|blake3" crates/nexus-core-rs/Cargo.toml`
    → `serde_jcs` `:30`, `ed25519-dalek` `:32`, `blake3` `:33` (all
    `{ workspace = true }`). `serde_big_array` already imported in `curator.rs:49`
    + `seed.rs:39`.
  - Phase B **adds no dependency and bumps none** (B.2 file list: a new module +
    a domain constant + a re-export + an HTTP route + a generic refactor — all on
    existing crates). Therefore the P2-PREFLIGHT-TRANSITIVE-DEPTH transitive-graph
    walk (Cargo.lock resolve + `cargo tree -d`) is **not triggered**: there is no
    added/bumped crate that could pull a second major version (contrast S72
    `ollama-rs 0.3.4` -> `schemars 1.2` collision, which required the walk).
- Finding: **clean.** No new dep, no CVE surface change, no breaking release on
  any touched API. `serde_jcs` (RFC 8785 JCS), `ed25519-dalek`, `blake3` are the
  same audited primitives already signing `CuratorList`/`SeedRequest` since S7/S74.

## S2 Historical Decisions
- Commands:
  - `git log --oneline -8 -- canonical.rs curator.rs seed.rs` →
    `b76a084` (S74 Phase E) added `DOMAIN_SEED_REQUEST_V1`/`_RESPONSE_V1`;
    `ff5e349` (S61) `DOMAIN_FEED_V1`; older entries S25/S22/S20/S19.
  - `git show 376bfe2 --format=%B` (hotfix #6) → `BrowseEntry.node_id` is
    `#[serde(skip)]` **by deliberate design**: keep `/browse` JSON byte-identical
    + frontend Zod schema untouched; the shell reads reachability via `status`,
    not `node_id`.
  - `git log --all -S 'serde(skip)' -- browse.rs` → single origin `376bfe2`.
- Decisions crossed:
  1. **Sibling-type-per-signed-surface** (`canonical.rs:201-219`, S74 `b76a084`):
     `DOMAIN_SEED_REQUEST_V1` is the exact template the plan copies. Status:
     **confirmed, still valid, reused not reverted.** Non-blocking — this is the
     decision the plan honors, not contradicts.
  2. **`CuratorProjectRef` has no `archive_hash` and conflates
     `project_id==node_id`** (curator.rs `:137-154`, browse.rs `:632-634`). This
     is precisely WHY the board rejected substrate A (overload `CuratorList`) and
     chose B (new type). Status: **no reversion; rationale drives the plan.**
     Non-blocking.
  3. **`node_id #[serde(skip)]` daemon-internal** (browse.rs `:195`, `376bfe2`).
     Phase B does **not** touch this field; Phase C is scheduled to un-skip /
     view it (plan `:151`, kickoff `:184`). Reverse-commit check: no later commit
     reverts the skip; the rationale (stable wire + Zod) still holds. A future
     Phase C un-skip is an **additive** read-side change to `/browse` JSON and is
     explicitly out of Phase B scope. Non-blocking for B.
  4. **`ANNOUNCEMENT_VERSION=1`** for the gossip envelope (iroh_runtime.rs `:92`).
     The directory flow will need its own announcement envelope (or reuse the
     curator one). Pre-launch policy: stays at 1, freely redefinable (kickoff
     §1.4 `:90-97`). Non-blocking.
- Finding: **clean.** No documented decision contradicts verbatim reuse of the
  CuratorList machinery for a sibling type. Every crossed decision either *is*
  the chosen approach (sibling type) or *justifies* it (no archive_hash in
  CuratorProjectRef). No un-reverted "do not add a new domain" rule exists; the
  repo has added 17 domains across sprints exactly this way.

## S3 Local Patterns And Threat Model
- Threats/contracts checked: cross-type signature replay (domain separation),
  attribution split-brain (`node_id == author pubkey`), DoS flood (entry cap +
  per-field cap), revision rollback, Sybil residue (THREAT_MODEL §15 row D),
  lock-3 tripwire (hard-coded anchor). This is a **NEW security component (new
  signing domain + new wire type)** → **full S3 scan** performed.
- HARDENING_ROADMAP status: no S75 Phase B pre-requirement; the anti-Sybil triad
  (Ed25519 new domain + kudos-threshold aggregation + curator-signature) is a
  kickoff §4 `:151-154` invariant that travels with directory artifacts. Phase B
  delivers the **first leg** (Ed25519 new domain + signature). The kudos-threshold
  and curator-signature legs are aggregation/read-side gates that land where the
  directory is *aggregated* (Phase C ingest/aggregator + Phase D), not at the
  authoring/crypto layer Phase B builds. This is consistent: `CuratorListEntry`
  itself is signature-only; `verify_with_contributor_registry` (curator.rs
  `:320-341`) is the additive governance gate layered on top. Non-blocking for B.
- Threat-model mapping (primitive = `NodeDirectoryEntry` sign/verify/ingest):
  - **Spoofing (cross-type replay)** — `DOMAIN_NODE_DIRECTORY_V1` is a distinct
    domain prefix; `canonical_bytes` (canonical.rs `:240-248`) prepends
    `<domain>\x00<jcs>`, so a node-directory signature can never validate as
    Task/Result/Claim/Invite/Kudos/CuratorList/SeedRequest/... and vice-versa.
    Enforced structurally by the prefix; tested by mirroring `curator.rs:589-602`
    (test `node_directory_cross_domain_replay_rejected`, plan B.3 #2). **Disjoint
    confirmed** (uniqueness proven in S4). res: Nil.
  - **Spoofing (impersonation / attribution split-brain)** — reuse the
    `CuratorListEntry` envelope pattern: a redundant `node_id` on the envelope
    that MUST equal `entry.node_id` (the signing pubkey), checked before the
    signature (curator.rs `:268-272` `verify_signature` step 3). Invariant
    `node_id == author pubkey` is **enforceable and enforced at verify**: the
    signature is computed over `canonical_bytes(entry_payload)` with the node
    keypair, and `sign()` rejects `entry.node_id != keypair.public_bytes()`
    (mirror curator.rs `:214-218`). Verrou 4 (provenance = author, never seeder)
    holds: the directory carries the AUTHOR's `archive_hash`; the seeder never
    signs it (THREAT_MODEL §15 invariant `seeder != auteur` `:836-842`). res: Nil.
  - **Tampering (DoS flood, oversized catalog)** — reuse `CURATOR_LIST_MAX_ENTRIES
    = 256` + per-field caps (`PROJECT_ID 128 / NAME 128 / CATEGORY 64 /
    DESCRIPTION 280`, curator.rs `:84-90`), enforced at BOTH sign and verify
    (curator.rs `:219-226` + `:260-267`). The new `archive_hash` field needs a
    cap too (64 hex chars; see S4 / Implications). 256 entries x per-field caps
    keeps a single signed catalog well under ~200 KB (curator.rs `:76-79`
    rationale), bounding RAM/gossip. Catalog growth beyond 256 apps/node is the
    WIRE-3 reprovide property deferred to Phase C/D, not a Phase B gap. res: Nil
    at B layer.
  - **Tampering (revision rollback)** — reuse monotone `revision` + the runtime
    `revision <= stored -> reject` gate (iroh_runtime.rs `:571-579`). Phase B
    delivers the field + sign/verify; the ingest gate is exercised by the generic
    helper. Test `node_directory_revision_monotone_rollback` (plan B.3 #4). res: Nil.
  - **Disclosure (over-count / Sybil residue, THREAT_MODEL §15 row D, sev M)** —
    Phase B authoring does NOT regress this: a `NodeDirectoryEntry` is a *claim*,
    and BLAKE3 content-addressing remains the joinability truth (a forged catalog
    can over-claim but never serve bytes it does not hold — fetch verifies the
    hash, §15 `:838-842`). The live probe + BLAKE3, not the catalog count, is the
    authority (design_review C2 `:146`). No new broadcast-Sybil surface: the
    network-wide aggregated digest (SearchManifest) stays deferred (scope cut #12,
    pivot_proposal §5). res: M (unchanged, carried, not regressed).
  - **lock-3 tripwire (hard-coded anchor)** — `default_curators` defaults EMPTY
    (config.rs `:245-251`, `#[derive(Default)]` + `#[serde(default)]` on empty
    `Vec`; doc-comment `:242-244` says VPS deployments populate via *config*, not
    the binary). Phase B adds a directory **authoring** route (publish MY own
    catalog) — it does NOT add any `default_anchors`/`default_curators` compiled
    constant. The tripwire stays armed. **Guard for the livreur:** the publish
    route must sign with the LOCAL node keypair only; it must not embed any peer
    node_id as a default. res: Nil (tripwire intact).
- Finding: **clean.** Every threat the new primitive introduces is covered by the
  CuratorList machinery being reused verbatim, plus one new field cap to add
  (`archive_hash`). No regression of a covered T0-T5 threat; no missing HARDENING
  pre-requirement for Phase B. The §15 over-count residue (M) is unchanged and
  carried by design (Phase C/D probe authority), not regressed by B.

## S4 Protocol And Wire Invariants
- Wire/security files checked: `canonical.rs` (domain tags), `curator.rs`
  (sign/verify contract), `iroh_runtime.rs` (announcement envelope + ingest),
  `browse.rs` (read-side projection), `seed.rs` (sibling-type precedent).
- VERSION/domain/canonical status:
  - **Domain uniqueness** (`grep -rhoE 'b"nexus-[a-z0-9-]+"' canonical.rs |
    sort -u`): 17 existing tags — `age-witness, claim, contributor-attestation,
    curator-list, delegation-cert, duress-ack, feed, invite, key-rotation, kudos,
    pow, provenance, result, seed-request, seed-response, task, warrant-canary`,
    all `-v1`. Proposed `b"nexus-node-directory-v1"` is **NOT** in the set →
    disjoint by construction (fail-fast check #8, plan `:334`). The livreur must
    add the const, the `lib.rs` re-export, and extend the `//!` family-list
    doc-comment (canonical.rs `:48-64`) to keep the disjoint-families enumeration
    honest.
  - **No `*_FORMAT_VERSION` bump** (D5, kickoff §1.4 `:90-97`): Phase B is purely
    ADDITIVE — a new domain + a new type + a new route. `CURATOR_LIST_FORMAT_VERSION`
    (curator.rs `:61`), `SEED_FORMAT_VERSION`, `FEED_FORMAT_VERSION`,
    `ANNOUNCEMENT_VERSION` (iroh_runtime.rs `:92`) all stay at 1. The new
    `NodeDirectoryEntry` carries its OWN `version: u16 = 1` field (mirror
    `CuratorList.version`). Fail-fast #14 (`*_FORMAT_VERSION` unchanged) holds.
  - **No tolerant multi-version decoder** pre-launch (pre-launch policy): the new
    type rejects unknown `version` outright (mirror curator.rs `:254-258`). No
    legacy decoder to port (the type is net-new). Compliant.
  - **`serde(default)` justification**: the catalog entry fields are
    always-present (the author writes a full catalog). The only `#[serde(default,
    skip_serializing_if)]` candidates are optional display fields; if the struct
    keeps every field always-present (like `CuratorProjectRef`), no `serde(default)`
    is needed at all. Recommend: NO `serde(default)` on `CatalogApp` core fields —
    keep them always-present (cleaner wire contract, easier Phase F Zod).
- **Producer -> consumer trace (P2-PREFLIGHT-WIRE-CONTRACT-DEPTH)** for each new
  field the phase serializes, to prove the Phase B freeze suffices for C (ingest)
  and D (multi-provider pull):
  - `NodeDirectoryEntry.node_id` (`[u8;32]` pubkey) — **producer**: the publish
    route signs with the local node keypair and stamps `node_id = keypair.public`
    (mirror curator.rs `:231`). **consumer (Phase C)**: the ingest helper cross-
    checks `entry.node_id == announcement pubkey` (mirror iroh_runtime.rs
    `:564-569`), then the aggregator sets `BrowseEntry.node_id = entry.node_id`
    (today forced `None` at browse.rs `:632-634`; un-skipped in C). **consumer
    (Phase D)**: `node_id` becomes the FIRST provider in the multi-provider
    `download(hash, vec![node_id, ...seeders])` vector (blobs.rs `:188` already
    takes a `Vec<endpoint_id>`). Field is sufficient for C and D. **CONFIRMED.**
  - `CatalogApp.archive_hash` (64-hex BLAKE3) — **producer**: stamped from the
    local browse/deploy record at authoring time. **consumer (Phase C)**: indexed
    + stored on `BrowseEntry.archive_hash` (already a field, browse.rs `:227-228`,
    `#[serde(default, skip_serializing_if = "Option::is_none")]`). **consumer
    (Phase D)**: BLAKE3 is the integrity gate of the pull (`download` verifies the
    hash; blobs.rs `:205-207`); it is also the re-derivation root for a re-minted
    ticket (see Q1). Field is sufficient — D can re-derive a dialable ticket from
    `(node_id, archive_hash)`. **CONFIRMED.**
  - `CatalogApp.{project_id, name, category, description}` — **producer**:
    authoring. **consumer**: `BrowseEntry.{project_id, project_name, category,
    description}` (browse.rs `:179,197-203`). 1:1 shapes; per-field caps mirror
    `CuratorProjectRef`. Note the **naming delta**: plan B.2 names the field
    `name` while `BrowseEntry`/`CuratorProjectRef` use `project_name`. Pick ONE
    and document it; recommend `project_name` for symmetry with the existing
    consumer (avoids a Phase C mapping seam + a Phase F Zod alias). **CONFIRMED
    with a naming note.**
- Day 0 status: **preserved.** D1 (sibling type) implemented as-is; D5 (0-bump
  additive) honored; lock-3 tripwire armed; verrou 4 (author provenance) intact.
- Finding: **clean.** Additive new domain, new always-present type, no version
  bump, unique tag, both ends of every field traced. One naming consistency note
  (`name` vs `project_name`) and one new field cap to add (`archive_hash` 64-char)
  — both are implementation details, not wire-contract conflicts.

## Open Questions — recommendations

### Q1 (kickoff §10): `archive_ticket` re-minted at pull vs `archive_hash`-only
- **Recommendation: `archive_hash`-only on `CatalogApp` (NO stored
  `archive_ticket`).** Evidence:
  - A `BlobTicket` embeds the provider `EndpointAddr` (blobs.rs `:179-180`
    `ticket.into_parts()` -> `(addr, hash, format)`). A ticket stored in a
    signed, replicated, long-lived directory entry would freeze a STALE
    `EndpointAddr` — exactly the class of bug Phase A fixed for announcements
    (re-mint address at replay, plan A.1 `:53-57`). Embedding a ticket in the
    directory re-introduces the stale-address bug the sprint is curing.
  - `download(hash, vec![endpoint_id])` (blobs.rs `:188`) only needs the HASH +
    a provider id; the provider id comes from `node_id` (always fresh, dialed via
    pkarr/discovery) and the Phase D seeder vector — NOT from a frozen ticket.
  - Phase A already extracted `mint_ticket_for_hash(&node, hash)` (referenced at
    http.rs `:1649-1653`) which re-mints a dialable ticket from the CURRENT
    address. Phase D re-mints at pull time from `(node_id, archive_hash)` using
    this helper — the directory stays a pure `archive_hash` content reference,
    the dialable address is always fresh. This is the F-Droid model (fingerprint
    persists, the dialable index is re-fetched) the board endorsed (D4
    `:100-101`).
  - **Consequence for the struct**: `CatalogApp` carries `archive_hash: String`
    (64-hex), NOT `archive_ticket`. This keeps the signed bytes free of volatile
    transport addressing — the signature covers only content-stable fields.

### Q2 (kickoff §10): generic helper `ingest<SignedList>` vs copied arm
- **Recommendation: generic helper `ingest_signed_list<T: SignedList>`** (the
  plan's C1/R1 mitigation, design_review C1 `:145`). Evidence:
  - The curator ingest arm (iroh_runtime.rs `:500-590`) is a 9-step gate
    (parse -> version -> pubkey -> attention-set -> fetch -> verify -> envelope
    cross-check -> revision-dedup -> store). Copying it for `NodeDirectoryEntry`
    duplicates ~20 lines of security-critical gating — the exact drift risk R1
    (a future fix to one arm silently skips the other).
  - A `trait SignedList` abstracting `{ pubkey() -> [u8;32], revision() -> u64,
    verify_signature() -> Result<()>, fn domain() }` lets ONE generic function
    own steps 2/6/7/8 for both `CuratorListEntry` and `NodeDirectoryEntry`. Steps
    1/3/4 (envelope parse, attention-set membership, blob fetch) are the same
    `CuratorAnnouncement`-shaped envelope (iroh_runtime.rs `:114-148`), reusable
    as-is or via a second tiny envelope type.
  - The test `generic_ingest_helper_parity` (plan B.3 #6) proves the refactor
    produces the SAME verdict as the original curator arm on the same inputs —
    a behavior-preservation guard. This must run BEFORE the directory arm is
    wired (the refactor lands in B; the directory consumer lands in C).
  - **Caveat (non-blocking)**: the existing `process_announcement_bytes`
    callers (gossip loop + 2-node integration test, iroh_runtime.rs `:22-26`)
    must keep their signatures; introduce the generic helper and have the curator
    path call it, rather than rewriting the curator call sites. Keep the public
    `CuratorRuntime::process_announcement_bytes` API stable.

## Guardrails — confirmation (kickoff §4, 5 verrous)
1. **Zero target/host field** — CONFIRMED. `NodeDirectoryEntry` is a read-side
   projection (a node publishes ITS OWN catalog); no "publish to X" selector.
   The authoring route signs only the LOCAL node's catalog. Fail-fast #13/lock-1.
2. **Additive-not-substitutive redundancy** — CONFIRMED for B. Phase B adds a new
   signed type + route; it does not replace `CuratorList` or the gossip path.
   (Cohabitation lands in C/F.)
3. **VPS = "Mon serveur", never hard-coded** — CONFIRMED. `default_curators`
   defaults empty (config.rs `:245-251`); Phase B adds NO compiled anchor const.
   **Tripwire armed** — any compiled non-empty `default_*` anchor list =
   DESIGN-CONFLICT (design_review C5 `:149`). Guard the publish route: sign with
   LOCAL keypair only.
4. **Provenance/signature = author, never seeder** — CONFIRMED. `node_id ==
   signing pubkey == author`, enforced at sign + verify (mirror curator.rs
   `:214-218,268-272`). The directory carries the AUTHOR's `archive_hash`; the
   seeder never signs it (THREAT_MODEL §15 `:836-842`). Anti-impersonation holds.
5. **Suggestion triggered by observed state** — CONFIRMED (N/A for B; this is a
   Phase F UX property; Phase B has no push-at-publish path).
- **Anti-Sybil triad**: leg 1 (Ed25519 new domain + signature) delivered in B;
  legs 2-3 (kudos-threshold aggregation + curator-signature) land at the
  aggregation/read layer (Phase C/D), consistent with the `CuratorListEntry` +
  `verify_with_contributor_registry` layering (curator.rs `:320-341`).
- **Pre-launch 0-bump**: CONFIRMED — purely additive, S74 SeedRequest pattern.

## Risks And Scope Cuts
- Blocking risks: **none.**
- Non-blocking risks / notes (carry into implementation, do NOT change verdict):
  - Naming consistency: plan says `name`; existing consumers use `project_name`.
    Recommend `project_name`. (S4)
  - Add a per-field cap for the new `archive_hash` field (64-char hex). (S3/S4)
  - Keep `CuratorRuntime::process_announcement_bytes` public API stable when
    extracting the generic helper. (Q2 caveat)
  - The directory gossip announcement envelope: reuse a `CuratorAnnouncement`-
    shaped `{v, pubkey, ticket}` envelope (iroh_runtime.rs `:114-148`) or a
    dedicated sibling; either keeps `ANNOUNCEMENT_VERSION` at 1. (S4)
  - R1 drift (duplicated ingest arm) mitigated by the Q2 generic helper, which
    IS a Phase B deliverable (not deferred).
- Scope cuts still honored (kickoff §9): #1 SearchManifest deferred (B builds a
  distinct `NodeDirectoryEntry`, pivot_proposal §5); #12 no Bloom/Merkle digest;
  #7 no `*_FORMAT_VERSION` bump. Phase B respects all three.

## Action
- EXECUTE: proceed with Phase B as planned. Implement `NodeDirectoryEntry` +
  `DOMAIN_NODE_DIRECTORY_V1` + authoring route + generic `ingest_signed_list<T>`
  helper, reusing the CuratorList sign/verify/cap/revision machinery verbatim.
  Apply the two recommendations (Q1 `archive_hash`-only; Q2 generic helper) and
  the non-blocking notes (field naming `project_name`, `archive_hash` cap,
  stable public ingest API). No commit is authorized by this file alone: the
  Phase B commit still requires green dual-platform fail-fast, a review-deep
  `## Verdict: PASS`, Codex verification, and the 9-section commit body.

Verdict: **EXECUTE**
