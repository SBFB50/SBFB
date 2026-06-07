# Sprint 74 Phase B Preflight

Date: 2026-06-07
HEAD: `457ca05`
Verdict: **SCOPE-CUT-CONSISTENT**

## Evidence Rules
- Claim policy: every claim below cites a path:line, a command + output, a
  URL/date, or an explicit assumption.
- Local sources read:
  - `prompts/agent/preflight.md` (procedure portable, source of truth)
  - `.planning/active/sprint74_plan.md` (Phase B: lines 155-193; §1 infra map;
    §7 scope cuts; §8 risks)
  - `.planning/active/sprint74_kickoff.md` (D1-D5, §8 traceability table lines
    660-674, §11 Checkpoint arbitrages PO lines 746-769)
  - `.planning/active/sprint74_phase_a_preflight.md` (inherited S1b/S4 baseline)
  - `.planning/active/sprint74_audit_plan.md` (C.3 / B.6 routing lines 187-242)
  - `.planning/archive/v2.1/sprint73_audit_findings.md` (C.3, B.6, OFF-SPRINT-2b
    source findings)
  - `.planning/research/s74_disponibilite_ux_design.md` (fork loop context — not
    Phase B backend, but the "remettre en ligne" consumer is Phase C)
  - `crates/sbfb-factory/Cargo.toml`, `crates/sbfb-factory/src/process.rs`
    (repo_root G17, lines 49-60), `template_engine.rs` (create→output_dir,
    lines 128-185), `pipeline.rs` (post_deploy_from_repo HTTP, lines 72-118),
    `gates.rs` (check_path_containment:117, run_gate_fg5_sandbox:65), `preview_cmd.rs`
  - `crates/nexus-shell-daemon/src/deploy.rs` (clone_repo via git CLI, lines
    498-571; publish_announcement helper 380-470; B.6-relevant deploy gate)
  - `crates/nexus-shell-daemon/src/http.rs` (publish gate is_open_source:934;
    index_browse_entry:984-1011)
  - `crates/nexus-shell-daemon/src/runtime.rs` (gossip ingest
    handle_project_announcement:1648-1729; index_browse_entry calls 1387, 1715)
  - `crates/nexus-coordinator-rs/src/search.rs` (BROWSE_ROWID_BASE:81,
    browse_rowid:90-97, index_entry:105-137, extract_index_fields:223-266,
    upsert_feed_entry:286-313)
  - `crates/nexus-core-rs/src/blobs.rs` (fetch_ticket:140-163 — no tag;
    add_bytes tag 77-88), `crates/nexus-core-rs/src/lib.rs` (re-exports)
  - `crates/nexus-shell-daemon-core/src/blob_serve.rs` (validate_zip_path:181;
    extraction loop 98-140)
  - `docs/security/THREAT_MODEL.md` (§5.3 deploy-from-repo 163-172; AD4 squatted
    repo; §5.4 iroh; R3 clone rate-limit 332-342)
- Commands run:
  - `git rev-parse --short HEAD` -> `457ca05`
  - `git log --oneline -8 -- crates/nexus-coordinator-rs/src/search.rs` -> rowid
    partition landed in `47b8c59`
  - `git show 47b8c59 --stat` -> rowid partition + prod browse->index wiring +16
    tests (one of the 9 post-audit Cas D hotfixes)
  - `grep -nE '^name = "(git2|gix|libgit2-sys)"' Cargo.lock` -> NO matches (no
    git library; clone is the git CLI subprocess)
  - `grep -nA1 '^name = "zip"' Cargo.lock` -> `zip 8.6.0`; `walkdir 2.5.0`;
    `dunce 1.0.5` (all already present)
  - `cargo tree -d -p sbfb-factory` -> duplicates are pre-existing iroh-tree
    (base64 0.21/0.22, reqwest 0.12/0.13); none introduced by Phase B

## Scope
- Plan source: `.planning/active/sprint74_plan.md §Phase B` (lines 155-193).
- Target files (plan §B.2):
  - `crates/sbfb-factory/src/fork.rs` (NEW) or `process.rs` —
    `fork_from_search_hit(triplet)`: clone forge OR blob reconstruction →
    workspace distinct from nexus repo.
  - `crates/sbfb-factory/src/process.rs` (repo_root, G17) — "target project
    distinct from nexus repo" notion.
  - `crates/nexus-coordinator-rs/src/search.rs` (index_entry, rowid) — **C.3**
    rowid partition.
  - `crates/nexus-shell-daemon/src/http.rs` (index_browse_entry) or `deploy.rs`
    — **B.6** re-apply `is_open_source⇒provenance_hash` at the browse-index path.
- Deps/APIs/specs: **none new** (decisive S1b finding below). The forge clone
  reuses the git CLI subprocess pattern already in `deploy.rs`/`process.rs`; the
  blob reconstruction reuses `zip` + `validate_zip_path` (both vendored).
- Security/protocol surfaces: untrusted forge content (clone) + untrusted blob
  (reconstruction) → zip-slip + workspace isolation + the B.6 provenance
  invariant. **No wire format, no `*_VERSION`, no canonical bytes** (S4 confirmed).
- Tests expected (plan §B.3):
  1. `fork_from_forge_clones_repo_at_commit`
  2. `fork_from_blob_reconstructs_archive`
  3. `fork_target_workspace_distinct_from_nexus_repo`
  4. `browse_rowid_partitioned_from_feed_seq` (C.3)
  5. `browse_index_rejects_open_source_without_provenance` (B.6)

## S1a OSS Prior Art
- Domain: creating a "fork workspace" from a source-verifiable provenance record
  (clone `repo_url@commit_sha`, OR content-addressed reconstruction from an
  archive hash as a fallback).
- Sources (accessed 2026-06-07):
  - F-Droid Reproducible Builds — https://f-droid.org/docs/Reproducible_Builds/
    + Submitting Quick Start. The build "starts by cloning the source code from
    the repo, then checking out the specific commit" (metadata `commit` field).
    Fork support tracks reference binaries / signing keys per commit. **This is
    exactly the plan's clone `repo_url@commit_sha` path** — the canonical
    source-verifiable platform clones-and-checks-out a pinned commit.
  - Radicle Heartwood — https://radicle.dev/guides/user +
    https://github.com/radicle-dev/heartwood/blob/master/rad.1.adoc. `rad clone`
    "creates a fork of the repository that is under your public key" and a local
    checkout; `rad checkout` materialises a working copy from local storage. A
    **delegate** (signing authority) is distinct from a seeder. **This validates
    the SBFB invariant that the fork becomes a NEW authorship under the forker's
    key** (Phase C re-signs; Phase B just materialises the workspace) and that
    seeder != author (D1/R5, not Phase B).
  - Content-addressed reconstruction: the SBFB archive is a BLAKE3-hashed zip
    blob (THREAT_MODEL A6, line 64); `fetch_ticket` + `get_bytes` materialises
    the exact bytes by hash. This mirrors IPFS/iroh content addressing — the
    blob path is the integrity-guaranteed fallback when the forge is gone.
- Finding: **APPROACH-ALIGNED**. The clone-OR-blob-reconstruct dual path is
  precisely the mature OSS pattern: clone-from-forge-at-commit (F-Droid) with a
  content-addressed fallback (Radicle local-storage checkout / IPFS-style
  reconstruction). No `LIB-EXISTS` blocker: the forge clone needs no fork
  library (git CLI is the established mechanism here, see S1b), and the blob
  fallback reuses the in-repo zip toolchain. No `APPROACH-NAIVE` signal.
- Impact: none — proceed as planned.

## S1b Dependencies, CVEs, Release Notes
- Scanned: `crates/sbfb-factory/Cargo.toml` + `Cargo.lock` for any clone/zip dep
  the plan might add.
- Decisive finding — **NO NEW DEPENDENCY (clean)**:
  - **There is no `git2`/`gix`/`libgit2-sys` anywhere in `Cargo.lock`**
    (`grep -nE '^name = "(git2|gix|libgit2-sys)"' Cargo.lock` → 0 matches). The
    daemon clones a forge repo via the **git CLI subprocess**:
    `deploy.rs:548-571 run_git()` shells `tokio::process::Command::new("git")`
    with `clone --depth 1 --single-branch` (deploy.rs:498-512) + `fetch`/
    `checkout FETCH_HEAD` for a pinned `commit_sha` (deploy.rs:514-543). The
    factory already shells `std::process::Command::new("git")` in
    `process.rs:50` (repo_root) — **the forge-clone path in `fork.rs` reuses the
    git CLI with zero new dep.** This honors the `feedback_context7_systematic`
    constraint trivially (no external lib touched).
  - `zip 8.6.0`, `walkdir 2.5.0`, `dunce 1.0.5` are **already direct deps** of
    `sbfb-factory` (`Cargo.toml:25-27`) — the blob-reconstruction unzip needs
    nothing new. `tempfile` is a dev-dep; if the workspace is created under a
    persistent dir (not a tempdir) Phase B needs no tempfile in non-test code,
    but if it does, `tempfile` is already a workspace dep used by `deploy.rs`.
  - Transitive graph (`cargo tree -d -p sbfb-factory`): the only duplicates are
    **pre-existing** in the iroh tree (`base64` 0.21 via hickory vs 0.22 via
    iroh; `reqwest` 0.12.28 [sbfb-factory + ollama-rs] vs 0.13.3 [iroh-relay]).
    Phase B adds no dep, so **no new collision** (the S72 schemars-1.2 class of
    DESIGN-CONFLICT does not apply — P2-PREFLIGHT-TRANSITIVE-DEPTH satisfied:
    the lock + `cargo tree -d` show a stable graph unchanged by this phase).
- CVE surface: none introduced. Clone-of-untrusted-repo and zip-slip are S3
  threat-model concerns, not CVEs in a dependency.
- Finding: **clean**. No dependency added or bumped; no transitive collision.

## S2 Historical Decisions
Decisions crossed by Phase B target files, each with a reverse-commit check:

- **C.3 ROWID-PARTITION — ALREADY RESOLVED by hotfix `47b8c59` (load-bearing).**
  The audit findings (`sprint73_audit_findings.md`, diff `845bea6..9472085`,
  dated 2026-06-04) flagged C.3 as a TRIPWIRE: `index_entry` INSERTed without an
  explicit rowid, so a feed upsert at `rowid=seq` could clobber a browse row, and
  the browse->index bridge had ZERO prod callers (test-only at the time). **The
  audit was written BEFORE one of the 9 post-audit Cas D hotfixes.**
  `git show 47b8c59` (`fix(search): wire Browse->index in production + prefix
  matching + rowid partition`, 2026-06-05) implemented exactly C.3 atomically
  with the prod wiring:
  - `search.rs:81 BROWSE_ROWID_BASE = 1 << 48`; `search.rs:90-97 browse_rowid()`
    (FNV-1a of `project_id` folded into the 48-bit range); `index_entry:115` now
    INSERTs `INSERT OR REPLACE` with the deterministic rowid. Feed owns
    `[1, 2^48)`, browse owns `[2^48, ...)` — **disjoint, cannot clobber**
    (search.rs:281-285 comment + test `browse_and_feed_rows_share_index_
    without_clobbering` search.rs:474-508).
  - `index_browse_entry` (http.rs:984) is now wired to THREE production callers:
    `deploy.rs:467`, `runtime.rs:1715` (gossip-announce), and via
    `publish_announcement` from `/publish` (http.rs:959).
  Reverse-commit check: `git log --oneline -8 -- search.rs` → `47b8c59` is the
  most recent functional change after `0f86e5a` (S73 Phase D). It is a forward
  resolution, not a reversion. **Consequence for Phase B: C.3 is already done.
  The plan's framing ("partition rowid AVANT tout cablage prod") is stale — the
  hotfix did both. Phase B's C.3 work degrades to a REGRESSION TEST
  (`browse_rowid_partitioned_from_feed_seq`) that pins the invariant, NOT new
  code.** This is a SCOPE-CONSISTENT reduction, not a conflict.
- **B.6 `is_open_source⇒provenance_hash` at the browse-index path — GENUINELY
  OPEN.** Reverse check: the invariant is enforced at the `/publish` HTTP gate
  (`http.rs:934`, "Sprint 16 audit finding D-1", res `d7c265a`/`10bbc63`,
  THREAT_MODEL §5.6 line 208) and STRUCTURALLY at deploy-from-repo
  (`deploy.rs:256` only passes `is_open_source: true` after a verified clone).
  BUT `index_browse_entry` (http.rs:984-1011) reads `entry.is_open_source` and
  `entry.provenance_hash` **independently** and passes both to `index_entry`
  with **no cross-check** (http.rs:988-994). The audit (B.6, CONFIRMED, latent
  S74) is correct and NOT reverted. The real attack surface is the gossip
  ingest: `handle_project_announcement` (runtime.rs:1648) ingests an UNTRUSTED
  peer's `ann.is_open_source` (line 1709) + `ann.provenance_hash`, then calls
  `index_browse_entry` (runtime.rs:1715) — a byzantine peer can gossip
  `is_open_source=true, provenance_hash=null` and the index will carry the lie
  that drives the fork consumer (and worker L2 consent, THREAT_MODEL §5.6).
  **This is the load-bearing Phase B carry — see clarification below.**
- **G17 / `process::repo_root` points at the nexus repo.** `repo_root()`
  (process.rs:49-60) runs `git rev-parse --show-toplevel` = the nexus checkout.
  It is consumed ONLY by the factory's process/operator/status commands
  (status-sprint, lint-planning, audit-commit, prompt/context) — the
  DOGFOODING surface where the factory operates on nexus itself. It is NOT used
  by the scaffolding path: `template_engine::create(template, name, output_dir)`
  (template_engine.rs:128) writes to an ARBITRARY `output_dir`. **So "fork
  workspace distinct from nexus repo" does NOT conflict with `repo_root` — the
  fork workspace must simply be created OUTSIDE `repo_root()` (its own dir, like
  `create`'s `output_dir`), and `fork_from_search_hit` must NOT call/derive from
  `repo_root()`.** The plan's "process.rs (repo_root, G17)" line is about
  ADDING a distinct-target notion, not editing repo_root's existing meaning.
  No reversion; consistent.
- **OFF-SPRINT-2b (/publish + gossip keep node_id)** is routed to **Phase C**
  (plan §C.2, kickoff §6, R7), NOT Phase B. Phase B must not touch it. Honored.
- **Pre-launch protocol — feed raw-op extensible, `*_VERSION` stay 1, M-schema
  local** (CLAUDE.md). Phase B touches zero feed op, zero migration. N/A but
  honored.

- Finding: **clean** (no blocking S2). One material reclassification: C.3 is
  already implemented (`47b8c59`), so Phase B carries a regression TEST not new
  code. B.6 is the real open carry (non-blocking, in-scope, see Action).

## S3 Local Patterns And Threat Model
- The fork path materialises UNTRUSTED forge content (clone) OR an UNTRUSTED
  blob (reconstruction) into a workspace. Threat mapping:
  - **Zip-slip (path traversal in the blob reconstruction)** — the
    decompression-to-disk of an attacker-controlled archive. THREAT_MODEL §5.3
    line 172 already names "Path traversal dans le zip | refuses cote
    coordinator" as res L. There is a CANONICAL guard:
    `nexus_shell_daemon_core::blob_serve::validate_zip_path(path) -> bool`
    (blob_serve.rs:181-195) — rejects `..`, leading `/` or `\`, embedded `\`,
    empty. The existing extraction loop (blob_serve.rs:108-133) calls it BEFORE
    insert. **Phase B's blob-reconstruction-to-disk MUST call `validate_zip_path`
    (or `gates::check_path_containment` after join) on every entry name BEFORE
    `File::create`/`write_all`, and skip symlinks** (the deploy zip path already
    skips `..`, leading `/`, and symlinks at deploy.rs:626-636; mirror that).
    The factory's existing `run_gate_fg5_sandbox` (gates.rs:65) catches
    symlink-escapes AFTER extraction, but the extraction itself is the
    write-time defense — do not rely on a post-hoc gate alone.
  - **Clone of a malicious / squatted repo** (THREAT_MODEL AD4, line 76;
    §5.3). The clone itself is bounded by the existing deploy mitigations the
    fork path should inherit: `--depth 1` (no history creds leak, §5.3 line
    170), 500 MB cap + 30s timeout (deploy.rs:28-30 MAX_CLONE_BYTES /
    CLONE_TIMEOUT_SECS; THREAT_MODEL R3 line 332). **Phase B's forge clone
    should carry the same depth/size/timeout caps as `deploy.rs::clone_repo`**,
    and the repo_url must pass the https-only guard (`normalize_clone_url` +
    `starts_with("https://")`, deploy.rs:71-74). The forked content is NOT
    executed during the fork — it lands as files in an iframe-sandboxed app
    later; the fork itself is file I/O.
  - **Workspace isolation from the nexus repo** — the fork workspace MUST live
    outside `repo_root()` (S2 above). Writing under the nexus checkout would let
    untrusted forge content land inside the maintainer's repo. Tested by
    `fork_target_workspace_distinct_from_nexus_repo`.
  - **B.6 invariant (`is_open_source⇒provenance_hash`)** — WHY it matters at the
    browse-index path: the index row drives (a) the S74 fork consumer (a hit
    claiming open-source-with-no-provenance is an integrity lie about
    forkability), and (b) worker L2 consent (THREAT_MODEL §5.6 line 208: "L2
    accepte un projet qui ment sur le flag"). The `/publish` gate protects the
    local write but the gossip ingest path (runtime.rs:1715) does not — a
    byzantine peer bypasses it. Re-applying the invariant at the shared
    `index_browse_entry` chokepoint closes all three callers at once.
- HARDENING_ROADMAP: no Phase-B pre-requirement is pending; the relevant
  hardening is the THREAT_MODEL §5.3 deploy-from-repo mitigations, which already
  exist and Phase B inherits. No regression of a covered threat as long as the
  zip-slip guard and clone caps are reused.
- Finding: **non-blocking** — provided Phase B (1) reuses `validate_zip_path`
  before any disk write in the blob path, (2) inherits the clone depth/size/
  timeout/https caps, (3) creates the workspace outside `repo_root()`, and (4)
  re-applies the B.6 invariant at `index_browse_entry`. These are hard
  requirements the plan already implies (B.6 is an explicit plan task); they are
  clarifications for the executor, not findings against the plan.

## S4 Protocol And Wire Invariants
- Wire/security files checked: NONE in scope. No `canonical.rs`, no `schemas/`,
  no `*_VERSION`, no `DOMAIN_*`, no signing domain. `git grep` confirms Phase B
  touches `fork.rs`(NEW)/`process.rs`/`search.rs`/`http.rs` — none serialise a
  gossip message or bump a version.
- Producer→consumer trace (P2-PREFLIGHT-WIRE-CONTRACT-DEPTH) for the fields
  Phase B reads/writes:
  - **The S73 provenance triplet (the fork input)**: producer = the
    `SearchResult` DTO (`search.rs:7-34`, `Serialize`-only) returned by
    `GET /api/daemon/search`; the four provenance keys
    (`repo_url`/`commit_sha`/`archive_hash`/`provenance_hash`) are
    `Option<String>` serialised ALWAYS-present-as-`null`, `is_open_source` a
    bare bool — the S73 Phase E consumer pins this with Zod `.nullable()` (not
    `.optional()`) on the front. **Phase B's `fork_from_search_hit(triplet)`
    CONSUMES this DTO server-side (Rust→Rust), so no Zod/wire change.** The
    `archive_hash` NAME BRIDGE is load-bearing: the feed payload field is
    `artifact_hash` while the consumer name is `archive_hash`
    (`search.rs:253-259`); the fork's blob path must read `archive_hash` (the
    DTO/column name), which is already correctly populated. **Unchanged contract.**
  - **The browse-index row (B.6 write)**: producer = `index_entry`
    (`search.rs:105`) via `index_browse_entry` (http.rs:984); consumer = `search`
    (search.rs:139) → `SearchResult` DTO. The columns are LOCAL FTS5 (M17
    schema, reconstructible from feed). Re-applying the B.6 invariant CHANGES no
    column shape — it only refuses to store `is_open_source=true` when
    `provenance_hash`/`repo_url` is absent (mirroring the http.rs:934 gate),
    matching the gate's existing JSON shape. **No wire/schema change; LOCAL
    index only.**
  - **The rowid space (C.3)**: producer = `browse_rowid` (search.rs:90, browse)
    vs `seq` (search.rs:299, feed); consumer = `search` ORDER BY bm25. Already
    partitioned and disjoint (`47b8c59`). **No change; verified by regression
    test.**
- `*_VERSION`: `FEED_FORMAT_VERSION` stays 1 (no feed op touched);
  `PROJECT_ANNOUNCEMENT_VERSION` stays 1 (no announcement shape change). **No
  M-migration needed** — C.3's schema already shipped via M17 (the UNINDEXED
  columns) + the `47b8c59` rowid logic (no schema change, pure rowid choice).
  B.6 adds a validation branch, not a column. **No new migration.**
- Day 0 status: **preserved**. Phase B is D5 "Segment SUR" (fork backend, no new
  cross-node protocol). D1/D3/D4 (cross-node ALPN, SeedAnnounced, invite) are
  Phases E/F — untouched here.
- Finding: **clean** (0 wire, 0 `*_VERSION`, 0 migration, confirmed explicitly).

## Risks And Scope Cuts
- **Blocking risks: none.**
- **Non-blocking findings (the SCOPE-CUT-CONSISTENT basis):**

  1. **C.3 is already implemented (`47b8c59`); Phase B's C.3 deliverable is a
     REGRESSION TEST, not new code.** The plan (and audit_plan §C.3, lines
     236-242) describe `index_entry` "INSERT sans rowid explicite" at
     `search.rs:67-97` — that code was rewritten by the hotfix. The executor
     must NOT re-implement the partition; it must add
     `browse_rowid_partitioned_from_feed_seq` to PIN it (a feed upsert at
     `seq=N` and a browse row whose `browse_rowid` could collide both survive).
     The existing `browse_and_feed_rows_share_index_without_clobbering`
     (search.rs:474) already proves coexistence; the new test should additionally
     assert the disjoint-range property explicitly (e.g. `browse_rowid(id) >=
     BROWSE_ROWID_BASE` and `seq < BROWSE_ROWID_BASE`).

  2. **B.6 belongs at `index_browse_entry` (http.rs:984), the shared
     chokepoint** — NOT in `deploy.rs` (which already passes a verified flag)
     and NOT only in `search.rs::index_entry` (which is a generic indexer also
     used by feed upserts that legitimately have no provenance). Placing the
     cross-check in `index_browse_entry` covers all THREE production callers
     (`deploy.rs:467`, `runtime.rs:1715` gossip ingest, `/publish` via
     `publish_announcement`) — critically the **gossip-ingest path
     (runtime.rs:1715)** which is the actual byzantine surface the `/publish`
     gate (http.rs:934) does not protect. The fix: in `index_browse_entry`, if
     `entry.is_open_source && (entry.provenance_hash.is_none() ||
     entry.repo_url.is_none())`, downgrade `is_open_source` to `false` (or skip
     indexing the flag) rather than storing the lie — and log it. The test
     `browse_index_rejects_open_source_without_provenance` should drive a
     `BrowseEntry { is_open_source: true, provenance_hash: None }` through
     `index_browse_entry` and assert the stored `is_open_source` column is
     `false` (invariant re-applied). Mirror the http.rs:934 error message
     semantics in the log.

  3. **Fork architecture: `fork_from_search_hit` lives in `sbfb-factory` and
     clones via the git CLI; the blob fallback needs a daemon-reachable blob.**
     The factory has NO live iroh `Endpoint`/`Node` — it talks to the daemon
     over HTTP (`daemon_client.rs::DaemonConnection`, `pipeline.rs:72-118`,
     `preview_cmd.rs`). Two consequences the executor must honor:
     - **Forge clone**: `fork.rs` shells `git clone --depth 1 --single-branch
       <repo_url>` then (if `commit_sha`) `git fetch --depth 1 origin <sha>` +
       `git checkout FETCH_HEAD` — the exact deploy.rs:498-543 sequence — into
       the fork workspace dir (outside repo_root). Carry the https-only guard
       (`normalize_clone_url` + `starts_with("https://")`) and the 500 MB /
       30s / 10s caps. **No new dep.**
     - **Blob reconstruction (fallback)**: the factory cannot `fetch_ticket`
       itself (no Endpoint). The realistic in-repo paths are (a) fetch the zip
       bytes from the daemon's existing blob-serve / a blob-bytes route and
       unzip locally, or (b) add a thin daemon route that returns the raw
       archive bytes for an `archive_hash`. **EITHER WAY the unzip MUST call
       `validate_zip_path` before each disk write** (the canonical guard,
       blob_serve.rs:181) and skip symlinks. If the plan's test
       `fork_from_blob_reconstructs_archive` is unit-level, it can reconstruct
       from in-memory zip bytes (no daemon) and assert the workspace files +
       zip-slip rejection; the daemon round-trip can be a Phase C/E2E concern.
       **The executor should pick the lowest-friction path that keeps the
       untrusted-unzip guard at write time** — flag in the commit body which
       blob-byte source was used.

  4. **Templates (react/pyodide) are Phase C, NOT Phase B.** PO Q7 (kickoff
     §11 line 765) adds react+pyodide, but the kickoff §8 traceability table
     (line 665) and plan §C.1 (line 205) route templates to **Phase C**. Plan
     §B.2/§B.3 correctly contain only fork + rowid + invariant. **Phase B must
     NOT add templates** — that is Phase C scope creep if pulled forward.

  5. **The fork is NOT a redeploy.** Phase B produces the workspace only; the
     re-signing/redeploy under local identity (provenance re-attribution, the
     R5 invariant) is **Phase C** (plan §C.1, `publish_announcement`). Phase B
     must not call `publish_announcement` or generate provenance — it stops at
     "workspace materialised, distinct from nexus".

- **Scope cuts still honored** (kickoff §7 / plan §7): #2 (templates etendus) is
  Phase C not B; #8 (Monaco editor — never) untouched; the fork backend is
  D5 "Segment SUR". No cross-node protocol (E-F) started.

## Action
- **SCOPE-CUT-CONSISTENT: proceed with Phase B as planned, honoring these
  load-bearing clarifications:**
  1. **C.3 = regression test only.** Do NOT re-implement the rowid partition —
     it already shipped in `47b8c59` (`BROWSE_ROWID_BASE = 1<<48`,
     `browse_rowid`, `index_entry` deterministic `INSERT OR REPLACE`). Add
     `browse_rowid_partitioned_from_feed_seq` pinning the disjoint-range
     invariant (feed `[1,2^48)` vs browse `[2^48,...)`).
  2. **B.6 = re-apply the invariant at `index_browse_entry` (http.rs:984)**, the
     shared chokepoint covering deploy + `/publish` + the byzantine gossip-ingest
     path (runtime.rs:1715). Downgrade `is_open_source` to `false` when
     `provenance_hash`/`repo_url` is absent (mirror the http.rs:934 gate) + log;
     do NOT put it in `search.rs::index_entry` (also used by provenance-less
     feed upserts) or only in `deploy.rs`.
  3. **`fork_from_search_hit` in `sbfb-factory`; forge clone via the git CLI
     (no new dep), inheriting the deploy.rs https-only + 500 MB + 30s/10s caps.**
     The workspace MUST be created OUTSIDE `repo_root()` (do not derive the fork
     target from `process::repo_root`; that stays the dogfood-nexus root).
  4. **Blob reconstruction MUST call `nexus_shell_daemon_core::blob_serve::
     validate_zip_path` before every disk write and skip symlinks** (zip-slip
     guard, the canonical in-repo defense). Pick the lowest-friction blob-byte
     source (daemon blob-serve/route or in-memory for the unit test); state it
     in the commit body.
  5. **No templates (react/pyodide = Phase C), no redeploy/provenance
     re-signing (= Phase C), no OFF-SPRINT-2b (= Phase C).** Phase B stops at a
     materialised, isolated workspace + the two browse-index hardenings.
  6. **No wire, no `*_VERSION`, no migration.** All Phase B surfaces are LOCAL
     (FTS5 index, factory workspace).
- The commit body must cite this preflight under `## G8 traceability`, note the
  C.3 reclassification (already shipped `47b8c59` → regression test) and the B.6
  chokepoint placement, and the no-new-dep / git-CLI clone decision.
