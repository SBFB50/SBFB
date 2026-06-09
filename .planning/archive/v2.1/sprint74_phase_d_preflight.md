# Sprint 74 Phase D Preflight

Date: 2026-06-07
HEAD: `9c2bd68`
Verdict: **SCOPE-CUT-CONSISTENT**

## Evidence Rules
- Claim policy: every claim cites a path:line, a command + output, a URL/date, or
  an explicit assumption.
- Local sources read:
  - `prompts/agent/preflight.md` (procedure portable, source of truth)
  - `.planning/active/sprint74_plan.md` (Phase D 239-281; §1 infra map; §7 scope
    cut #5 timer 22h; §8 R6 boot-safety; §5 fail-fast rows 17-20)
  - `.planning/active/sprint74_kickoff.md` (D2 gelee 301-337; §1.4 pre-launch M18
    local; §6 carries H.1/H.2; D5 segment SUR A-D)
  - `.planning/research/s74_disponibilite_ux_design.md` (read via plan/kickoff
    digest — §6/§8 pin local toggle, §13 seed volontaire; the AvailabilitySheet
    header §1-26 already encodes §4/§5/§6/§8 verbatim)
  - `.planning/active/sprint74_phase_c_preflight.md` + `..._c_review.md`
    (`finalize_deploy` canonical tail deploy.rs:370-464, `deploy_workspace`,
    `publish_announcement`, the GC-config absence is NOT touched by C)
  - `docs/security/THREAT_MODEL.md` (§5.3 deploy DoS 163-174; §5.5 loopback
    185-197; R3 disk DoS 332-338; §11 search surface header)
  - `crates/nexus-coordinator-rs/src/db.rs` (MIGRATIONS 16-261; M16 228; M17
    244-260; `open` 291-306; `open_in_memory` 308-319; `conn()` pub(crate) 699)
  - `crates/nexus-core-rs/src/blobs.rs` (add_bytes 77-88 discards tag.name;
    fetch_ticket 140-163 no tag)
  - `crates/nexus-core-rs/src/node.rs` (FsStore::load 313-325 NO GcConfig;
    blobs_store()->&Store 177; Store deref 24/108/120)
  - `crates/nexus-shell-daemon/src/runtime.rs` (CoordinatorDb::open 531-534;
    restore_browse_from_outbox call 1398-1404; fn 1750-1771; boot rebuild_from_feed
    804-807 WARN-only = H.1; handle_project_announcement 1700-1729)
  - `crates/nexus-shell-daemon/src/deploy.rs` (finalize_deploy 370-464; blob store
    428-433; publish_announcement is broadcast/persist only, no provenance)
  - `crates/nexus-coordinator-rs/src/search.rs` (rebuild_from_feed 322-334)
  - `crates/nexus-shell-daemon/src/http.rs` (build_router 231; route table
    268-363; auth_required layer 457)
  - `web/src/components/AvailabilitySheet.tsx` (Phase A toggle disabled+ON
    208-235; isOwn doc 55-64; remote voluntary CTA 236-262)
  - `web/src/components/__tests__/AvailabilitySheet.test.tsx`
    (keep_online_toggle_readonly_in_phase_a 129-148)
- Commands run:
  - `git rev-parse --short HEAD` -> `9c2bd68` (provided by task; HEAD per task)
  - `git grep -n "keep_online|keep-online"` -> only forward-looking front refs
    (AvailabilitySheet.tsx 59/213/220 + test) ; ZERO backend code, ZERO M18
  - `git show def3008 --format=%B` -> #7 restore_browse_from_outbox (forward fix,
    never reverted) ; `git show 3b7ef54 --format=%B` -> #8 deploy outbox parity
    (forward fix, never reverted) — both confirm boot re-announce = outbox replay
  - `rg "^name=iroh-blobs|rusqlite_migration|rusqlite" Cargo.lock` -> iroh-blobs
    `0.100.0`, rusqlite `0.36.0`, rusqlite_migration `2.2.0`
  - `rg "gc|Gc|GC" crates/**/*.rs` -> NO blob GC scheduled anywhere (only
    AES-GCM crypto, GCRA rate-limit, an iroh-docs doc-comment node.rs/docs.rs:167)
  - context7 `/n0-computer/iroh-blobs` Tags API: `store.tags().set/get/delete`,
    "store never GCs a blob with >=1 tag" ; `add_bytes` auto-creates a persistent
    tag with an auto-generated name

## Scope
- Plan source: `.planning/active/sprint74_plan.md §Phase D` (239-281).
- Target files (plan §D.2), verified against real code:
  - `crates/nexus-coordinator-rs/src/db.rs` — **M18** `keep_online` table +
    getters/setters. Plan cites "228-303 M-pattern"; the **real registry is the
    static `MIGRATIONS: &[M]` array at lines 16-261**; M17 is the last element
    (244-260). M18 is appended as element 18. `user_version` is tracked
    AUTOMATICALLY by `Migrations::to_latest` (db.rs:303/316) — no manual bump.
  - `crates/nexus-shell-daemon/src/deploy.rs` — set `keep_online=true` at
    self-deploy. The real chokepoint is **`finalize_deploy` (370-464)**, the
    single shared tail both `deploy_from_repo` and `deploy_workspace` route
    through (Phase C `finalize_deploy` extraction). Plan also names `publish.rs`:
    **`nexus-shell-daemon` has no `publish.rs`** (same stale ref the Phase C
    preflight flagged); the daemon publish path is `http.rs::publish_project` ->
    `finalize_deploy`/`publish_announcement`.
  - `crates/nexus-shell-daemon/src/runtime.rs` — `restore_*` reads `keep_online`.
    Plan cites 1750-1771 (`restore_browse_from_outbox` def). The boot CALL is at
    **1398-1399**; the boot `rebuild_from_feed` WARN-only (H.1) is at **804-807**.
  - `crates/nexus-core-rs/src/blobs.rs` — tag kept if `enabled`, removed on OFF.
    `add_bytes` (77-88) **discards `tag_info.name`** so today's auto-tag is NOT
    addressable for deletion; `fetch_ticket` (140-163) tags nothing (E gap, not D).
  - `crates/nexus-shell-daemon/src/http.rs` — route `POST /api/daemon/keep-online`
    registered in `build_router` (231) under `auth_required` (457).
  - `web/src/components/AvailabilitySheet.tsx` — toggle becomes functional
    (replaces the disabled-ON of Phase A 208-235).
- Deps/APIs/specs: **none new**. M18 reuses `rusqlite_migration 2.2.0` (already
  pinned). Blob tagging reuses `iroh-blobs 0.100` Tags API (already a dep). The
  HTTP route reuses `axum 0.8`/`auth_required` (already present). S1b clean.
- Security/protocol surfaces: loopback-auth toggle route (§5.5); blob retention
  policy (R3 disk); **NO new wire format, NO `*_VERSION` bump, M18 = LOCAL DB
  schema** (S4 confirmed).
- Tests expected (plan §D.3): keep_online_toggle_persists_m18 /
  pinned_app_reannounced_on_boot / keep_online_off_removes_tag /
  migration_m18_creates_keep_online_table / m17_boot_recovery_not_silent (H.1).

## S1a OSS Prior Art
- Domain: local content-pin policy (a node decides which content it keeps and
  re-announces across restarts), with an opt-OUT that stops re-announcing and
  releases the retention guarantee.
- Sources (accessed 2026-06-07):
  - IPFS pinset + `ipfs repo gc` (https://docs.ipfs.tech/concepts/persistence/,
    https://docs.ipfs.tech/how-to/work-with-pinning-services/) — a pin is a named
    retention intent that protects content from GC; unpinning makes it
    GC-ELIGIBLE but does NOT immediately delete (GC is a separate sweep). **This
    is exactly the iroh-blobs tag model** (context7: "store never GCs a blob with
    a tag"; `tags().delete` -> "may be GC'd later"). The D2 design (table =
    intent, tag = retention, OFF = delete tag) mirrors the IPFS pinset/GC split.
  - IPFS reprovide 22h / "Provide Sweep" 2025
    (https://ipshipyard.com/blog/2025-dht-provide-sweep, kubo#9389) — provider
    records expire (48h), so a node MUST re-announce after a reboot to stay
    reachable; a periodic timer is the steady-state mechanism. **SBFB's plan
    scope-cuts the 22h timer (#5)** and uses re-announce-at-boot only — aligned
    with mature practice for the closed pilot (the boot replay + NeighborUp cover
    the pilot; the timer is the post-launch refinement).
  - Radicle Heartwood seeding policy
    (https://radicle.xyz/guides/seeder, 2024-2025) — a per-node policy table of
    repos it replicates; clone/init updates the policy (auto-seed of what you
    host). **`keep_online` = the per-node retention policy table**, the same
    pattern (a local table of "what this node keeps"), seeder != delegate/author
    (R5, preserved by Phase D being pin-LOCAL of the node's OWN app).
- Finding: **APPROACH-ALIGNED**. Table-as-intent + tag-as-GC-protection +
  unpin-as-GC-eligible + re-announce-at-boot is the canonical local-pin pattern
  (IPFS/Radicle). No `LIB-EXISTS` (rusqlite + iroh-blobs Tags cover it in-repo),
  no `APPROACH-NAIVE`. Impact: none.

## S1b Dependencies, CVEs, Release Notes
- Scanned: `db.rs` migration dep, `blobs.rs` tag dep, the new HTTP route deps.
- **NO NEW DEPENDENCY.** M18 reuses `rusqlite_migration 2.2.0` /`rusqlite 0.36.0`
  (Cargo.lock 7220-7236), the append-to-`MIGRATIONS`-array pattern proven by
  M5..M17. Blob tagging reuses `iroh-blobs 0.100.0` (Cargo.lock 3976-3977) Tags
  API. The route reuses `axum 0.8` + `auth_required` (already in the router).
- Transitive graph: Phase D adds nothing to `Cargo.toml`, so the lock and
  `cargo tree -d` are unchanged from Phase C (Phase C preflight recorded the
  duplicates as the pre-existing iroh tree: base64 0.21/0.22, curve25519 4/5-pre,
  ed25519 2/3-rc, bitflags 1/2). **The S72 schemars-1.2 collision class does NOT
  apply** (no dep added/bumped) — P2-PREFLIGHT-TRANSITIVE-DEPTH satisfied.
- CVE surface: no crypto/wire/network/sandbox dep introduced. iroh-blobs 0.100 is
  the frozen Day-0 pin (kickoff §59-61), no upgrade.
- Finding: **clean**.

## S2 Historical Decisions
Each Phase D target carries a reverse-commit check.

- **Boot re-announce = outbox replay, ALREADY LIVE (`def3008` #7 +`3b7ef54` #8).**
  `git show def3008 --format=%B`: `restore_browse_from_outbox` runs once at
  gossip-task startup and re-ingests EVERY persisted `ProjectAnnouncement`
  through `handle_project_announcement` (repopulate aggregator + re-index). `git
  show 3b7ef54 --format=%B`: `deploy.rs::publish_announcement` is THE single
  canonical announce->broadcast->persist-outbox->index->cache helper; deploy and
  /publish both route through it; "persist ALWAYS, even when isolated". Both are
  forward fixes, never reverted (`restore_browse_from_outbox` is the latest
  functional code at runtime.rs:1750; `publish_announcement` is the current tail
  of `finalize_deploy`). **CONSEQUENCE (load-bearing, clarification B): the boot
  re-announce the plan asks for is ALREADY UNCONDITIONAL.** Every app whose
  announcement is in the outbox is re-broadcast at boot regardless of
  `keep_online`. So `keep_online` does NOT ADD re-announce-at-boot; it ADDS (a)
  an explicit GC-protection tag and (b) an OFF state that must STOP the existing
  re-announce + remove the tag. See clarification B.

- **`add_bytes` tag is auto-named and discarded (blobs.rs:87).** `add_bytes`
  returns `TagInfo` whose `.name` is an auto-generated tag (context7: persistent,
  protects from GC), but `BlobsClient::add_bytes` returns only `*tag_info.hash`
  and drops `name`. The doc-comment "pinned with a named tag equal to the hex of
  its hash" (blobs.rs:74-76) is **STALE/aspirational** — the code does NOT set a
  hex-named tag; it accepts the store's auto-name. Reverse check: `git grep
  "tags().set|tags().delete|create_tag"` -> 0 hits; SBFB never addresses a tag by
  name. **CONSEQUENCE (clarification C): the OFF-removes-tag path cannot delete
  the existing auto-tag (its name is unknown). Phase D must set an EXPLICIT
  deterministic-name tag (e.g. `keep-online/<project_id>` or
  `keep-online/<hash>`) it can later `tags().delete(name)`.**

- **NO GC is scheduled (node.rs:313-325).** `FsStore::load(&blobs_dir)` is called
  with no `GcConfig` and no GC task is spawned (`rg gc` -> none for blobs).
  Reverse check: no commit ever added a GC scheduler. **CONSEQUENCE
  (clarification D + S3): removing a tag today makes the blob GC-ELIGIBLE but
  nothing collects it — disk is NOT freed.** The UX line "le blob peut etre GC'd"
  is forward-true (when GC lands) but materially inert now. This is a
  honesty/scope point, NOT a blocker: it matches IPFS (unpin != delete; gc is a
  separate sweep) and the front already says "stockee mais plus diffusee".

- **H.1 boot-recovery WARN-only is at runtime.rs:804-807, NOT at M17.** The plan
  frames H.1 as "the rebuild post-DROP of M17 must log/meter a failure". Two
  distinct sites: (i) M17's DROP/recreate runs INSIDE `migrations.to_latest`
  (db.rs:303); a failure there propagates as an `Err` from `CoordinatorDb::open`
  -> the daemon fails to boot (NOT silent — it is fatal, the strongest signal).
  (ii) the boot `rebuild_from_feed` (runtime.rs:804) is the genuinely warn-noyed
  path: `Err(e) => warn!(error=%e, "search index rebuild failed, search may be
  stale")`. **H.1's real target is (ii) at 804-807** (escalate to error!/metric),
  matching the audit S73 wording "M17 boot-recovery warn-only -> index vide
  silencieux". Reverse check: this warn line is current (the boot rebuild is the
  documented recovery path, search.rs:317-321). Non-blocking carry, Phase D.

- **`keep_online` getters/setters must be public methods on `CoordinatorDb`.**
  `db.conn()` is `pub(crate)` (db.rs:699), so search.rs (same crate) reaches the
  connection directly, but the daemon (`nexus-shell-daemon`, different crate)
  cannot. Phase D must add `pub fn set_keep_online / get_keep_online /
  list_keep_online_enabled` ON `CoordinatorDb` (mirrors `insert_provenance_record`,
  `get_storage_namespace`). Clarification A.

- **Pre-launch protocol** (CLAUDE.md, kickoff §1.4): Phase D touches zero feed op,
  zero `*_VERSION`, zero canonical. M18 is a LOCAL DB migration (type M16/M17),
  reconstructible (the truth is outbox + provenance + feed). Honored.

- Finding: **clean (no blocking S2)**. Three plan framings are stale-or-imprecise
  (boot re-announce already unconditional; `add_bytes` tag not addressable; H.1
  site is runtime.rs:804 not M17) but each degrades to a documented clarification,
  not a conflict. The decision (D2) is intact and confirmed reversion-free.

## S3 Local Patterns And Threat Model (FULL — keep_online is a loopback-exposed
retention control)
The toggle is a network-exposed (loopback) mutation of a retention/announce
policy. Full threat model:

- **Asset**: blob retention (disk) + the re-announce intent (which apps this node
  broadcasts). A1-class (local node state), not the keypair.
- **Actors**: (a) the local user via the shell (loopback, authenticated);
  (b) AD2 a same-host malicious process with the bearer token (THREAT_MODEL §1.3,
  §5.5 R1); (c) AD3 a remote byzantine peer (CANNOT reach this route — loopback
  only).
- **Vector — DoS disk via pinning giant apps (R3 family, THREAT_MODEL:332-338).**
  Can a `keep-online ON` flood fill the disk? The blob is **already stored** by
  the deploy (`finalize_deploy` add_bytes, deploy.rs:428) BEFORE any keep_online
  toggle — `keep_online` is a flag over already-resident bytes, it does NOT pull
  new content (cross-node fetch+pin is Phase E). So toggling ON adds at most one
  tag row + protects bytes that already cost their disk. The disk cost is bounded
  by the existing deploy path (`MAX_DEPLOY_BYTES` cap + the existing R3 clone-DoS
  surface), NOT amplified by this toggle. **No new DoS surface vs the pre-existing
  deploy.** Residual R3 (deploy-from-repo clone DoS) is unchanged.
- **Vector — OFF releases a blob another app references (content-addressing).**
  Two apps with the SAME `archive_hash` share the SAME blob (blake3
  content-addressing). If app-A is OFF'd and its tag deleted while app-B (same
  hash) is still ON, does app-B break? With the iroh-blobs Tags model, the blob
  is GC-safe as long as >=1 tag points to it (context7: "never GC a blob with a
  tag"). **So the tag MUST be keyed per-retention-intent, not assumed unique per
  blob**: if Phase D names tags `keep-online/<project_id>`, deleting A's tag
  leaves B's tag -> blob safe. If it names tags `keep-online/<hash>` (one tag per
  blob), A's OFF would orphan B. **Recommendation (load-bearing): key the tag by
  a unit that survives a sibling OFF** — either `keep-online/<project_id>` (1 tag
  per pinned app, blob safe while any app keeps it) OR re-derive
  protection from a `list_keep_online_enabled` scan. Moreover, since **no GC runs
  today**, the worst case is currently inert (nothing collects the orphan); but
  the design must be correct for when GC lands (scope cut, post-launch). Add the
  multi-app-shared-blob case to `keep_online_off_removes_tag` assertions.
- **Vector — AD2 toggles keep_online via the bearer token.** A same-host malware
  with the token can already deploy/wipe/publish (THREAT_MODEL §5.5 R1: token
  leak = full loopback API). keep-online OFF (deny availability) or ON (retain)
  is strictly weaker than the existing panic/wipe + publish surface. The route
  MUST sit behind `auth_required` (http.rs:457) like every mutation — no new
  trust tier. No regression.
- **Vector — boot re-announce of an OFF'd app (R6 boot-safety).** The EXISTING
  `restore_browse_from_outbox` (runtime.rs:1398) re-announces EVERY outbox entry
  unconditionally. Phase D's OFF must SUPPRESS re-announce for `enabled=false`
  apps, otherwise "stockee mais plus diffusee" is a lie (the outbox would re-emit
  it at boot). **Load-bearing (clarification B/D): the boot path must consult
  `keep_online` and skip re-broadcast (NOT skip re-ingest into the local
  aggregator — the owner can still SEE their offline app) for `enabled=false`.**
  This must be additive + best-effort + warn-only (R6): a `keep_online` read
  failure must NOT abort the boot restore (fall back to current behaviour).
- **Regression check**: no covered T0-T5 threat regressed. The route inherits
  §5.5 loopback mitigations (bearer + Host + Origin); the blob path inherits the
  deploy cap. No new wire = no feed/search surface change (§10/§11 untouched).
- **HARDENING_ROADMAP**: no Phase-D pre-requirement pending. R3 (disk DoS) is a
  documented residual, NOT regressed; an actual GC reaper is out-of-scope
  (post-launch) and is the thing that would MAKE the OFF path free disk.
- Finding: **non-blocking**, provided Phase D: (1) keys the GC tag per pinned
  intent so a sibling-app OFF cannot orphan a shared blob; (2) gates the route on
  `auth_required`; (3) makes the boot re-announce skip `enabled=false` apps,
  additively + warn-only (R6); (4) does NOT claim disk is freed on OFF (no GC
  today) — the honest UX is "plus diffusee", retention released for the future
  sweep. These are design requirements the plan implies, not findings against it.

## S4 Protocol And Wire Invariants
- Wire/security files checked: `db.rs` MIGRATIONS (LOCAL schema), `blobs.rs`
  (tag = local store metadata), `runtime.rs` boot (re-ingests EXISTING
  `ProjectAnnouncement`), `http.rs` route. NO `canonical.rs`, NO `schemas/`, NO
  `DOMAIN_*`.
- `*_VERSION` status: `FEED_FORMAT_VERSION = 1`, `PROJECT_ANNOUNCEMENT_VERSION
  = 1`, `PROVENANCE_SCHEMA_VERSION = 1` — Phase D bumps NONE. M18 = LOCAL DB
  `user_version` (tracked by rusqlite_migration), NOT a wire `*_VERSION`.
- Producer->consumer trace (P2-PREFLIGHT-WIRE-CONTRACT-DEPTH) for every field
  Phase D touches:
  - **`keep_online` row (project_id, enabled, archive_hash, pinned_at)**:
    producer = `finalize_deploy` (set true on self-deploy) + `POST
    /api/daemon/keep-online` (toggle); consumer = the boot re-announce gate
    (`restore_*`) + the blob-tag apply/remove. **Both ends are LOCAL to one
    daemon process** — there is NO cross-process/cross-language consumer, so this
    is not a wire contract at all (it is a private DB schema). M18 is fully
    reconstructible: the outbox + provenance + feed are the durable truth.
  - **`POST /api/daemon/keep-online` JSON body + response**: producer = front
    `AvailabilitySheet` toggle (replaces the Phase A disabled toggle); consumer =
    `http.rs` handler. **NEW route, NEW JSON shape** — the executor owns BOTH
    ends (Rust handler + TS fetch + Zod if used), so the contract is fixed in one
    phase. Recommend an explicit shape: request `{ project_id: string, enabled:
    bool }`, response `{ enabled: bool }` (mirror existing toggle handlers).
    Choose null-vs-absent and envelope explicitly (S73 Phase E lesson: the Rust
    side serializes keys ALWAYS-present; a TS Zod consumer needs `.strict()` over
    the exact shape). The front test currently asserts NO fetch on mount
    (test:146); Phase D's interactive toggle fetches ON CLICK, not on mount —
    preserve that (do not fire on render).
  - **`ProjectAnnouncement` (re-announce payload)**: producer/consumer UNCHANGED
    — Phase D reuses the existing `publish_announcement` / `restore_*` /
    `handle_project_announcement` path (runtime.rs:1700-1729). Phase D only GATES
    whether the existing re-broadcast fires for `enabled=false`; it does NOT
    change the announcement shape. `git show 3b7ef54` confirms the per-app
    `project_id` contract is the current wire (Phase C preflight S4). No bump.
- `serde(default)` audit: M18 columns get SQL `DEFAULT` (e.g. `enabled` default
  true at deploy, `pinned_at` NOT NULL) — local schema defaults, not wire
  `serde(default)`. The route body MAY use `#[serde(default)]` for runtime
  tolerance (a minimal client posting `{project_id}` defaults `enabled` to a
  documented value) — legitimate runtime tolerance, document the rationale.
- Day 0 status: **preserved**. Phase D is D5 "Segment SUR" (pin LOCAL only). D1
  (`SeedRequest` ALPN), D3 (`SeedAnnounced` raw-op), D4 (invite/approval) are
  Phases E/F — UNTOUCHED. The `fetch_ticket` tag gap (blobs.rs:140) is explicitly
  E-scope, NOT D. D2 (table + tag + boot re-announce) honored. No central server
  (loopback only). Intentions-not-jargon CTA: "Garder en ligne" toggle, no
  `keep_online`/`tag`/`M18` jargon in the UI.
- Finding: **clean** (0 `*_VERSION` bump, 0 canonical edit, M18 = LOCAL schema,
  the only NEW wire is a self-owned loopback route whose both ends land in this
  phase, the re-announce payload is unchanged).

## Risks And Scope Cuts
- **Blocking risks: none.**
- **Non-blocking findings (the SCOPE-CUT-CONSISTENT basis):**
  1. **Boot re-announce is ALREADY UNCONDITIONAL (`def3008`).** Phase D does not
     ADD re-announce-at-boot; it ADDS the OFF gate (skip re-broadcast for
     `enabled=false`) + the explicit GC tag. The `pinned_app_reannounced_on_boot`
     test must assert (a) ON app re-announced AND (b) OFF app re-ingested-locally
     but NOT re-broadcast. Cite `def3008`/`3b7ef54` and correct the plan's
     "restore_* reads keep_online in ADDITION to outbox" framing.
  2. **`add_bytes` auto-tag is not addressable (blobs.rs:87).** OFF cannot delete
     the existing tag (unknown name). Phase D must set an EXPLICIT
     deterministic-name tag (`keep-online/<project_id>`) via `store.tags().set`
     and delete it via `tags().delete`. Update the blobs.rs doc-comment (74-76),
     which currently misdescribes the tag.
  3. **No GC runs today (node.rs:313).** OFF makes the blob GC-eligible but frees
     nothing now. Non-blocking + honest (matches IPFS unpin!=delete and the front
     copy "plus diffusee"). Do NOT claim disk is freed; the retention release is
     correct for the future GC sweep (scope cut #5 family / post-launch).
  4. **Shared-blob orphan correctness (S3).** Key the tag per pinned intent so a
     sibling-app OFF cannot orphan a blob a still-ON app references. Add the
     two-apps-same-hash case to `keep_online_off_removes_tag`.
  5. **H.1 target is runtime.rs:804-807, NOT M17.** M17 failure is already fatal
     (propagates from `open`). Escalate the boot `rebuild_from_feed` WARN to
     error!/metric (`m17_boot_recovery_not_silent` asserts the elevated signal).
     H.2 (browse-row reconstructibility) is documented/carried (depends on
     browse-indexing, Phase B) — carry note, not code.
  6. **`keep_online` API must be PUBLIC methods on `CoordinatorDb`** (db.conn()
     is pub(crate)). Add `set_keep_online`/`get_keep_online`/
     `list_keep_online_enabled` mirroring `insert_provenance_record`.
  7. **Front toggle test churn (expected).** `keep_online_toggle_readonly_in_
     phase_a` (test:129-148, asserts disabled + no fetch) is INTENTIONALLY
     replaced by Phase D's interactive toggle (fail-fast row 9 -> row 17). Update
     that test; keep "no fetch on mount, fetch on click".
- **Scope cuts still honored** (kickoff §7 / plan §7): #5 timer 22h re-announce
  (post-launch) NOT added — boot replay only. #1 GPU cross-machine (S75), #2
  quorum cross-machine (S75) untouched. E-F cross-node (`SeedRequest`, blob
  fetch+tag, `SeedAnnounced`) NOT started — `fetch_ticket` tag gap stays E.
  Pin is LOCAL only (D5 Segment SUR).

## Action
- **SCOPE-CUT-CONSISTENT: proceed with Phase D, honoring these load-bearing
  clarifications (commit body must cite this preflight under `## G8
  traceability`):**
  1. **M18**: append a new `M::up` element to the `MIGRATIONS` array (db.rs after
     line 260), creating `keep_online (project_id TEXT PRIMARY KEY, enabled
     INTEGER NOT NULL DEFAULT 1, archive_hash TEXT NOT NULL, pinned_at INTEGER
     NOT NULL)`. `user_version` is auto-bumped by `to_latest`. Add public
     `set_keep_online`/`get_keep_online`/`list_keep_online_enabled` methods on
     `CoordinatorDb`. Test `migration_m18_creates_keep_online_table` asserts a
     real `to_latest` upgrade (not just the table exists).
  2. **Set keep_online=true in `finalize_deploy`** (the single shared deploy tail,
     deploy.rs:370-464) for the LOCAL self-deploy — NOT in a non-existent
     `publish.rs`. Best-effort/non-fatal like the other finalize side-effects.
  3. **Explicit GC tag** via a new `BlobsClient::tag_blob(name, hash)` /
     `untag_blob(name)` (wrapping `store.tags().set/delete`) OR direct
     `state.node.blobs_store().tags()` calls. Name the tag per pinned intent
     (`keep-online/<project_id>`) so a sibling OFF cannot orphan a shared blob.
     `keep_online_off_removes_tag` asserts the tag is gone AND a sibling app's
     tag (same hash) survives. Fix the stale blobs.rs:74-76 doc-comment.
  4. **Boot gate**: in the boot restore path (runtime.rs around 1398), consult
     `list_keep_online_enabled`; for `enabled=false` apps, re-ingest locally
     (owner still sees the offline card) but DO NOT re-broadcast. Additive +
     best-effort + warn-only (R6: a keep_online read failure falls back to
     current unconditional behaviour, never aborts boot).
     `pinned_app_reannounced_on_boot` asserts ON re-announced, OFF not broadcast.
  5. **Route** `POST /api/daemon/keep-online` in `build_router` (http.rs ~268-363)
     under `auth_required`. Fix the explicit JSON shape (request
     `{project_id, enabled}`, response `{enabled}`), both ends owned this phase.
  6. **H.1**: escalate the boot `rebuild_from_feed` WARN (runtime.rs:804-807) to
     error! + a metric/structured field; `m17_boot_recovery_not_silent` asserts
     the elevated signal. Carry H.2 (browse-row reconstructibility) as a note.
  7. **Front**: make the `keep-online-toggle` interactive (POST on click, not on
     mount); update `keep_online_toggle_readonly_in_phase_a`. Keep the OFF copy
     "stockee mais plus diffusee" (no false "disk freed" claim). Intentions copy,
     no jargon.
  8. **No wire bump, no canonical edit.** M18 is LOCAL; the re-announce payload is
     unchanged; the only new wire is the self-owned loopback route. Reuse
     `mk_state`/`build_test_router` + an in-memory/temp DB for tests — no real
     network. Pin is LOCAL only; E-F cross-node not started.
