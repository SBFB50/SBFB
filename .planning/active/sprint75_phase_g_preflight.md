# Sprint 75 Phase G Preflight

Date: 2026-06-11
HEAD: `035a4f7`
Verdict: **SCOPE-CUT-CONSISTENT**

## Evidence Rules
- Claim policy: every claim below cites a path, a command output, a URL/date, or
  an explicit assumption.
- Local sources read: `prompts/agent/preflight.md`, `.planning/active/sprint75_plan.md`
  (Phase G + §5 fail-fast + §9), `.planning/active/sprint75_kickoff.md`
  (§2/§4/§8/§9/§13), `.planning/active/sprint75_phase_g_handoff.md`,
  `crates/nexus-shell-daemon/src/http.rs` (search_handler 3627-3705),
  `crates/nexus-coordinator-rs/src/validator.rs` (146-310),
  `crates/nexus-shell-daemon/src/validator_loop.rs` (60-130),
  `crates/nexus-shell-daemon/src/deploy.rs` (233-285, 358-437, 924-935),
  `crates/sbfb-factory/src/fork.rs` (1-50, 196-256),
  `crates/nexus-coordinator-rs/src/search.rs` (139-165),
  `crates/nexus-shell-daemon-core/src/config.rs` (243-367, 601-642),
  `deploy/nexus-shell-daemon.service`, `deploy/config.toml.example`,
  `docs/security/THREAT_MODEL.md` (§15 825-859, §16 historique),
  `crates/nexus-core-rs/src/canonical.rs` (via grep DOMAIN/VERSION).
- Commands run: `git rev-parse --short HEAD`, `git status -sb`, `git log --oneline -12`,
  `git log --oneline -8 -- <4 carry files>`, `git diff 0e2fb6b..035a4f7 -- Cargo.lock`,
  `git show bede850 --no-patch --format=%B | grep '^## '`, grep for the 4 carry
  test names, grep for hardcoded VPS / default_curators, grep for *_FORMAT_VERSION.

## Scope
- Plan source: `.planning/active/sprint75_plan.md` Phase G (lines 291-320) + §5
  fail-fast 24 rows (323-350) + §9 checkpoint (380-385).
- Target files (Rust code, 4 fixes):
  - `crates/nexus-shell-daemon/src/http.rs` `search_handler` (CARRY-5 clamp offset/q)
  - `crates/nexus-coordinator-rs/src/validator.rs` + `crates/nexus-shell-daemon/src/validator_loop.rs` (CARRY-2 guardrail-trip => Rejected terminal)
  - `crates/nexus-shell-daemon/src/deploy.rs` (PULL-1 strip pre-existing provenance.json)
  - `crates/sbfb-factory/src/fork.rs` (FORK-1 entry-count cap)
- Target files (docs/planning): `sprint75_verification.md` (NEW),
  `sprint76_audit_plan.md` (NEW), `docs/security/THREAT_MODEL.md` (§15 rows),
  `docs/rust/PATTERNS.md`, `docs/shell/PATTERNS.md`, `docs/claude/SPRINT_LOG.md`,
  `CLAUDE.md`, `.planning/roadmap_v5_factory_complete_vision.md`.
- Deps/APIs/specs: **none added or bumped** (see S1b — `git diff 0e2fb6b..035a4f7
  -- Cargo.lock` is empty for the whole sprint; Phase G adds no dep).
- Security/protocol surfaces touched: zip extraction bound (fork.rs), provenance
  archive content (deploy.rs), task terminal-state machine (validator). No wire
  format, no `*_VERSION`, no `DOMAIN_*`, no canonical bytes, no signing domain.
- Tests expected (none pre-exist — grep confirmed): `search_clamps_offset_and_query`,
  `guardrail_trip_sets_rejected_terminal`, `deploy_strips_existing_provenance`,
  `fork_entry_count_capped`; + 24-row fail-fast (Windows nextest 1750 baseline +
  Docker Linux canonique row 6) + acceptance survives-VPS-death cross-machine
  (consigned checklist) + C6 E2E + LT-2 Radicle private dry-run.

## S1a OSS Prior Art
- Domain: (a) zip-bomb / decompression-bomb entry-count defense; (b) REST
  pagination offset/limit clamp DoS prevention. The other two carries (provenance
  dedup, task terminal-state) are internal hygiene, not an external problem domain.
- Sources (accessed 2026-06-11):
  - Zip-bomb defense incl. entry-count limits + CVE-2026-32630 "many-entries
    evades the size rule" class — `https://github.com/advisories/GHSA-j47w-4g3g-c36v`,
    `https://www.huntress.com/cybersecurity-101/topic/what-is-zip-bomb`,
    `https://en.wikipedia.org/wiki/Zip_bomb`. Best practice: cap entry COUNT in
    addition to compressed/decompressed byte caps; a flood of tiny entries passes
    a pure byte cap.
  - Pagination clamp `min(requested, MAX)` server-side, offset deep-pagination
    DoS on SQL `OFFSET` — `https://knowledgelib.io/software/patterns/rest-pagination/2026`,
    `https://ivopereira.net/efficient-pagination-dont-use-offset-limit`. Best
    practice: never trust a client limit/offset; clamp or 400.
- Finding: **APPROACH-ALIGNED**.
  - FORK-1: `fork.rs::extract_zip` (198-255) already enforces `MAX_ARCHIVE_BYTES`
    + `MAX_DECOMPRESSED_BYTES` but loops `for i in 0..archive.len()` with NO
    entry-COUNT cap — exactly the gap OSS flags. The plan's entry-cap closes it,
    matching mature practice. Clean precedent in-repo: `curator.rs:69`
    `CURATOR_LIST_MAX_ENTRIES = 256` (count-based cap).
  - CARRY-5: `search_handler` (http.rs:3655) already clamps `limit`
    (`params.limit.min(100)`) but passes `offset` (3633) and `q` straight into
    `search.rs::search` -> SQL `OFFSET ?3` (search.rs:162-165) with no clamp; the
    plan's offset/q clamp is the textbook server-side mitigation.
- Impact: none — the plan's two security-relevant carries are the recommended
  fix; no plan adaptation required.

## S1b Dependencies, CVEs, Release Notes
- Scanned: whole-sprint dependency delta + Phase G specifically.
- Commands/sources: `git diff 0e2fb6b..035a4f7 -- Cargo.toml Cargo.lock
  crates/*/Cargo.toml` -> **empty** (no add, no bump across S75 A-F). Phase G's 4
  fixes use only crates already in the workspace (`zip`, `rusqlite`, `axum`,
  `serde`, `blake3` — all pinned, unchanged). No new external API, no new spec.
  The zip-bomb CVE class (GHSA-j47w-4g3g-c36v, CVE-2026-32630) concerns the
  `file-type` JS package's parser, NOT the Rust `zip` crate this repo uses; it is
  cited only as evidence that the entry-count vector is real, not as a dep advisory.
- Finding: **clean**. Zero deps added/bumped; no CVE on the in-use crypto/wire/
  network/sandbox/signing surface introduced by this phase. P2-PREFLIGHT-
  TRANSITIVE-DEPTH N/A (no dep delta to walk the transitive graph for).

## S2 Historical Decisions
- Commands: `git log --oneline -8 -- crates/nexus-shell-daemon/src/deploy.rs
  crates/sbfb-factory/src/fork.rs crates/nexus-coordinator-rs/src/validator.rs
  crates/nexus-shell-daemon/src/validator_loop.rs`; grep for the 4 carry test names.
- Decisions crossed / state-of-code (the handoff explicitly asked to re-verify —
  deploy.rs and http.rs were heavily retouched in S75 C/D/E/F):
  - **CARRY-5** (http.rs `search_handler`): `limit.min(100)` ALREADY present
    (3655); offset + q NOT clamped (3633, passed to search.rs:165). **Partial** —
    limit-clamp done, offset/q clamp OPEN. Plan still valid; the test must assert
    offset and q-length specifically (limit-clamp is already covered elsewhere).
  - **CARRY-2** (validator/validator_loop guardrail-trip): S74 B.2
    (`bede850`/B.2) made *quorum-impossible* and *quorum-divergence* terminal
    (validator.rs:231-240, 302-308 set `TaskStatus::Rejected`). BUT the
    *guardrail-trip* path is DISTINCT and still OPEN: `validator_loop.rs:82-90` on
    a tripwire logs a warn and `return`s WITHOUT
    `db.update_task_status(..., Rejected, ...)` — the task stays in its prior
    non-terminal status (Pending, or AwaitingQuorum on the redundancy path) =
    the zombie the carry targets. The existing test
    `guardrail_tripwire_does_not_complete` (validator_loop.rs:~284) only asserts
    "not completed", NOT "set to Rejected terminal". So the carry is a REAL,
    unclosed gap; the new test `guardrail_trip_sets_rejected_terminal` is net-new
    (grep: name absent).
  - **PULL-1** (deploy.rs `add_to_zip`): `finalize_deploy` (370-437) calls
    `add_to_zip(zip, "provenance.json", ...)` (420-425) which uses
    `ZipWriter::new_append` + `start_file` unconditionally (924-935) — appends a
    SECOND `provenance.json` if the input zip already has one. The fork->redeploy
    path `deploy_workspace` (233-285) accepts an arbitrary uploaded zip (a blob-
    reconstructed fork carries the ORIGINAL author's baked-in `provenance.json`)
    and flows through `finalize_deploy`, so the duplicate is reachable in prod.
    **REAL, unclosed**. R5 invariant (seeder/forker re-signs FRESH local
    provenance) is preserved by the fix — stripping the stale one BEFORE injecting
    the fresh local one strengthens, not weakens, "provenance is the local author".
  - **FORK-1** (fork.rs `extract_zip`): byte caps present, entry-count cap absent
    (198-255). **REAL, unclosed**. Test name absent.
- Reverse-commit check: the 4 zones' last feature touches are S74 B/C/D/F + S75
  A/C (`bede850 9c2bd68 bcfc155 66a9409 4c1acc5 821aa8c 479a87c`). None of these
  reverted or pre-empted the 4 carries; they introduced the very surfaces (fork
  redeploy, guardrail-before-persist split, quorum-terminal) the carries now
  finish. No "rejected decision being re-litigated".
- Finding: **clean** (no conflicting frozen decision). All 4 carries confirmed
  open and correctly scoped; CARRY-5 is partial-already-done (limit only) — the
  Phase G test/fix must target offset + q, not re-do the limit clamp.

## S3 Local Patterns And Threat Model
- Threats/contracts checked:
  - Lock-3 tripwire (kickoff §4 #3, R5, the DESIGN-CONFLICT tripwire of this
    sprint): grep `135.181.42.188` / hardcoded VPS node_id across `crates/`
    `--include=*.rs` minus tests = **none**. `default_curators` defaults to empty
    `Vec<String>` (config.rs:252, validated 64-hex 340-346) and `[seed]
    keep_online_projects` defaults empty (config.rs:283, 359-367).
    `deploy/config.toml.example:28,46` ship `default_curators = []` /
    `keep_online_projects = []`. The "survives-VPS-death (a) no hard-wired
    discovery on the VPS node_id" acceptance is already structurally true and
    test-pinned (runtime.rs:3394 `auto_subscribe_default_curators_at_boot`); Phase
    G's job is to DEMONSTRATE it live, not change code. **PASS, no regression.**
  - T-SEARCH-DOS (THREAT_MODEL §11 583): CARRY-5 offset/q clamp is additive
    hardening on an already-catalogued threat — strengthens, never regresses.
  - Fork/deploy disk-safety (THREAT_MODEL §5.3 deploy-from-repo; fork.rs zip-slip/
    symlink/byte caps): FORK-1 entry-count + PULL-1 dedup are additive defense-in-
    depth on the existing fork/deploy surface. No covered threat regressed.
  - CARRY-2 guardrail-before-persist (S73 Phase A, THREAT_MODEL §14/LOOPBACK §3):
    making the trip terminal removes a zombie, does not weaken the guardrail.
- HARDENING_ROADMAP status: no S75/Phase-G pre-requirement is gated on this phase;
  Phase G is the sprint's wrap + carry-closure, not a hardening pre-req. (No
  `docs/security/HARDENING_ROADMAP.md` row references S75 Phase G as blocking;
  assumption recorded — the kickoff §11 risk register R1-R7 are all mitigated by
  A-F, G only documents.)
- THREAT_MODEL §15 status: §15 exists from S74 (825-859) covering the S74 seed
  primitives (SeedRequest, voluntary seed, SeedAnnounced registry). It does NOT
  yet carry the S75-specific rows the plan requires (directory pull route /
  blob-serve oracle + dial amplification, /nodes, SEED-1/SEED-2 clamps, fresh-
  flood displacement, boot seed driver + requester route E, front-F surface
  exposure of seed_voluntary/set_keep_online without a duress gate). Adding these
  is **documenting a NEW surface for the new S75 primitives** = a documented
  future-gap update, **not a regression of a covered threat**. Non-blocking.
  Note §16 process: a new surface must also touch §7 mitigations table + §2 Assets
  + §4 DFD (the auditor will expect those companion edits, not only the §15 row).
- Finding: **clean** (no T0-T5 regression, no missing blocking pre-requirement).
  Non-blocking carry: §15 rows + companion §7/§2/§4 edits are documentation work
  owned by this phase, plus the front-F duress-gate P2 stays DEFERRED to S76
  (kickoff/plan scope, not a Phase G code item).

## S4 Protocol And Wire Invariants
- Wire/security files checked: grep `*_FORMAT_VERSION` / `*_ANNOUNCEMENT_VERSION`
  / `DOMAIN_*` across `crates/nexus-core-rs/src/`; the 4 carry zones' serialized
  surfaces.
- VERSION/domain/canonical status: all constants at `1` —
  `CURATOR_LIST_FORMAT_VERSION=1`, `KEY_ROTATION_FORMAT_VERSION=1`,
  `NODE_DIRECTORY_FORMAT_VERSION=1` (S75-B), `POW_FORMAT_VERSION=1`,
  `SEED_FORMAT_VERSION=1`, `TASK_FORMAT_VERSION=1`, `PIN_FILE_FORMAT_VERSION=1`.
  Phase G touches NONE.
- Producer -> consumer trace (P2-PREFLIGHT-WIRE-CONTRACT-DEPTH) for the 4 fixes:
  - CARRY-5: `offset`/`q` are HTTP QUERY params (`SearchQuery` struct http.rs:3627,
    `#[serde(default)]` on offset — runtime tolerance, legitimate), NOT a serialized
    response field. The clamp is applied between the inbound query and the SQL
    `OFFSET ?3` (search.rs:165). The JSON RESPONSE shape `{results,total,took_ms}`
    (http.rs:3696-3703) is UNCHANGED — the same envelope the TS Zod `.strict()`
    consumer expects (Phase E `searchBrowse`). Clamping the input never changes
    the response contract. No consumer drift.
  - CARRY-2: `TaskStatus::Rejected` is an EXISTING enum value already produced on
    the quorum-impossible/divergence paths (validator.rs:232,302) and already
    consumed by `GET /api/v1/tasks/{id}` / the task poller. Adding one more
    producer site (the guardrail-trip branch) emits a value the consumer ALREADY
    handles — no new variant, no schema change.
  - PULL-1: operates on ARCHIVE BYTES inside the zip (stripping a duplicate
    `provenance.json` member before injecting the fresh one). The `provenance.json`
    SCHEMA is unchanged; only the de-duplication of the member is changed. The
    on-wire `ProjectAnnouncement` / `ReleasePublished` envelope is untouched.
  - FORK-1: a guard on the extraction LOOP (count of members materialised). No
    serialized output; the fork produces a filesystem workspace, not a protocol
    message.
- Day 0 status: **preserved**. D1-D5 (kickoff §5) untouched; pre-launch 0-bump
  policy honored (CLAUDE.md "Pre-launch protocol policy"); lock-3 tripwire green
  (S3). No tolerant multi-version decoder introduced. The one `#[serde(default)]`
  in scope (`SearchQuery.offset`) is pre-existing runtime tolerance for a query
  param, not wire drift.
- Finding: **clean**. 0 wire bump, 0 canonical/domain change, no Day 0 contradiction.

## Plan Adaptation
Not applicable (no PLAN-ADAPT). All scans either clean or non-blocking. One
in-plan refinement to carry into the commit/test (documented here, not a
deviation): CARRY-5's `limit` clamp is ALREADY shipped (http.rs:3655) — the Phase
G fix + `search_clamps_offset_and_query` test must target `offset` (clamp) and
`q` (length bound) specifically, not re-implement the limit clamp.

## Risks And Scope Cuts
- Blocking risks: **none**.
- Non-blocking risks / carry-over:
  - **Environment (highest operational risk)**: the survives-VPS-death acceptance
    + C6 E2E require cross-machine networking (SSH mac 192.168.1.53 + vps
    135.181.42.188, systemd live) and the iroh-networked suite + Docker Linux
    canonique (row 6, not re-run since S74). This is the S74 `create_node` hang
    class (host network-stack sensitive). Mitigation: run cargo single-threaded
    (never 2 parallel), never cargo during a Codex round, prove any hang by
    stash/pop on HEAD, reboot machine (NEVER `wsl --shutdown`) as remedy. If the
    live cross-machine leg is env-blocked, the platform-agnostic core (the 4 Rust
    fixes + unit-simulated `stale_announcement_accepted_by_fresh_receiver`, lock-3
    test-pin, web) is fully verifiable on Windows non-networked + clippy +
    release + doctests; consign the live acceptance as a horodated checklist and
    treat Docker/dual-platform as the gate BEFORE PUSH (per `feedback_wsl_before_push`),
    not before commit.
  - THREAT_MODEL §15 new rows are documentation owned by this phase + companion
    §7/§2/§4 edits (§16 process); the auditor will check for those companions.
  - Front-F duress gate (seed_voluntary / set_keep_online / reannounce_seeds_at_boot
    pre-existing absence) stays a P2 DEFERRED to S76 (route to sprint76_audit_plan,
    do not fix in G).
  - All S76 consolidated deferrals (handoff §7: PULL-3 cross-tier failover, anti-
    Sybil seeder-tail sampling, re-drive-on-ingest one-shot driver, curator-vs-
    anchor discriminator on /nodes waiting rows, seed.rs:111-116 self-designation
    doc, F NITs, externes P2-A-1/P2-AUDIT-2/T-NN+2/P3-OS-1/LT-3/4/7) must be
    ROUTED into sprint76_audit_plan from all 6 phase reviews (ratio 6/6) — this is
    Phase G content, not a finding.
- Scope cuts still honored (kickoff §9 / plan §7, all 12): SearchManifest
  deferred (#1), Tantivy frozen (#2), GC reaper/disk budget deferred (#3),
  federated cross-node search out (#4), peer-approval seed unchanged (#5),
  mobile/Electron no (#6), no wire migration / 0-bump (#7 — S4 confirms), GPU->S76
  (#8), sharding->S77 (#9), kudos-threshold tuning post-launch (#10), advanced
  multi-anchor UX deferred (#11), Bloom/Merkle digest not introduced (#12).

## Action
- **SCOPE-CUT-CONSISTENT**: proceed with Phase G as planned. The 4 carry fixes are
  all confirmed REAL and open against current code (CARRY-5 partial: offset+q only,
  limit already clamped; CARRY-2 guardrail-trip path open despite S74 B.2 quorum
  work; PULL-1 duplicate provenance.json reachable via fork->redeploy; FORK-1
  entry-count cap absent). No dep delta, no wire bump, no Day 0 conflict, lock-3
  tripwire green. Track the non-blocking carry-over: THREAT_MODEL §15 rows +
  companion edits, the env-exposed live acceptance (consign + gate-before-push),
  and the full S76 deferral routing. Commit body must cite this preflight under
  `## G8 traceability` and note the CARRY-5 limit-already-clamped refinement.
