# Sprint 74 Phase C Preflight

Date: 2026-06-07
HEAD: `bcfc155`
Verdict: **SCOPE-CUT-CONSISTENT**

## Evidence Rules
- Claim policy: every claim below cites a path:line, a command + output, a
  URL/date, or an explicit assumption.
- Local sources read:
  - `prompts/agent/preflight.md` (procedure portable, source of truth)
  - `.planning/active/sprint74_plan.md` (Phase C lines 196-235; B 155-193; D
    239-281; §1 infra map; §7 scope cuts; §8 risks R5/R7)
  - `.planning/active/sprint74_kickoff.md` (D1-D5; §6 carries OFF-SPRINT-2/2b;
    §1.4 pre-launch; §4 D1 seeder!=author; D-DISPO amendement)
  - `.planning/active/sprint74_phase_b_preflight.md` + `..._b_review.md`
    (Phase B primitive `fork.rs`, the 6 load-bearing clarifications, P0/P1 fixed)
  - `.planning/research/s74_disponibilite_ux_design.md` (§4 invariants, §5
    mockups greffe D "remettre en ligne", §8 5 verrous, §11 Q2/Q3 arbitrage)
  - `.planning/archive/v2.1/sprint73_audit_findings.md` (OFF-SPRINT-2 line 287,
    OFF-SPRINT-2b line 288, source findings)
  - `docs/security/THREAT_MODEL.md` (§5.3 deploy-from-repo 163-172; §5.6 L2
    open_source 200-208; AD4 squat 76)
  - `crates/sbfb-factory/src/fork.rs` (full — the Phase B primitive,
    `#[allow(dead_code)]`), `main.rs:1-119` (Command enum, fork wiring 9-14),
    `pipeline.rs` (full — `run_publish_pipeline` -> `post_deploy_from_repo`),
    `template_engine.rs:102-185` (TEMPLATES const, `create`)
  - `crates/nexus-shell-daemon/src/deploy.rs` (deploy_from_repo 65-318;
    project_id derivation 135-143; provenance gen 167-173; publish_announcement
    helper 375-470; deploy_private 320-359)
  - `crates/nexus-shell-daemon/src/http.rs` (publish_project gate 934 +
    per-app project_id 947-953 + publish_announcement call 959-973;
    index_browse_entry B.6 988-1029; multiple_apps_get_distinct_browse_cards
    6932-6959)
  - `crates/nexus-shell-daemon/src/runtime.rs` (handle_project_announcement
    1648-1729; gossip per-app id tests 1980-2074; 1569 =
    jittered_republish_duration NOT a project_id site)
  - `crates/nexus-shell-daemon-core/src/publish.rs` (ProjectAnnouncement struct
    32-52, node_id + project_id fields; from_gossip_bytes validation 181-205;
    with_project_id builder 135; tests 239-269)
  - `crates/nexus-coordinator-rs/src/provenance.rs` (generate_provenance 31-58,
    signs with caller keypair + node_id; canonical_bytes 102-124)
  - `web/src/pages/BrowsedProject.tsx:538-564` (greffe D "La remettre en ligne"
    LIVE), `web/src/pages/Deploy.tsx:54-64` (searchParams prefill LIVE)
- Commands run:
  - `git rev-parse --short HEAD` -> `bcfc155`
  - `git log --oneline -6 -- crates/nexus-shell-daemon-core/src/publish.rs` ->
    `aed2303` is the most recent functional change (the per-app project_id fix)
  - `git log -S "with_project_id" --oneline -- .../publish.rs` -> `aed2303`
  - `git show aed2303 --no-patch --format=%B` -> "per-app identity on the wire,
    ALL paths: publish.rs, http.rs publish_project, deploy.rs, runtime.rs
    handle_project_announcement" + "Tests (+8)" -> OFF-SPRINT-2b RESOLVED
  - `git show bcfc155 --no-patch --format=%B` -> Phase B commit: "PAS de per-app
    project_id OFF-SPRINT-2b (Phase C, R7)" -> STALE vs aed2303 reality
  - `git grep -n "project_id: &state.node_id\|project_id = state.node_id"` -> 0
    PRODUCTION matches; the only `project_id = state.node_id.clone()` are 4
    `#[tokio::test]` fixtures (http.rs:6383/6428/6466/6505)
  - `git grep "PROJECT_ANNOUNCEMENT_VERSION\|FEED_FORMAT_VERSION\|
    PROVENANCE_SCHEMA_VERSION"` -> all `= 1`
  - `cargo tree -d` -> duplicates are pre-existing iroh-tree (base64 0.21/0.22,
    curve25519 pre-release, ed25519, bitflags); none introduced by Phase C

## Scope
- Plan source: `.planning/active/sprint74_plan.md §Phase C` (lines 196-235).
- Target files (plan §C.2):
  - `crates/sbfb-factory/src/fork.rs` + `crates/nexus-shell-daemon/src/deploy.rs`
    — redeploy the forked workspace via `publish_announcement`; provenance
    re-signed locally.
  - `crates/nexus-shell-daemon/src/http.rs` (1004) + `runtime.rs` (1569) +
    `publish.rs` (39) — **OFF-SPRINT-2b** per-app project_id on /publish +
    gossip. **(STALE — see S2; already shipped in `aed2303`.)**
  - `web/src/pages/Browse.tsx` / `BrowsedProject.tsx` — "Forker dans l'atelier"
    + "La remettre en ligne" (greffe D). **(greffe D already LIVE — see S2.)**
- Deps/APIs/specs: **none new** (S1b clean). Forge re-clone reuses the git CLI
  already in `deploy.rs`/`fork.rs`; redeploy reuses the existing
  `deploy-from-repo` route + `publish_announcement` helper.
- Security/protocol surfaces: provenance RE-SIGNING (R5 seeder!=author),
  untrusted forge content (already guarded in Phase B `fork.rs`), the
  `is_open_source=>provenance` L2 invariant (B.6, already at the chokepoint).
  **No new wire format, no `*_VERSION` bump, no migration** (S4 confirmed).
- Tests expected (plan §C.3):
  1. `fork_redeploy_resigns_provenance_as_local_node`
  2. `fork_redeploy_loop_e2e_single_node`
  3. `deploy_per_app_distinct_browse_cards` (OFF-SPRINT-2) — **already exists as
     `multiple_apps_get_distinct_browse_cards` http.rs:6932**
  4. `publish_and_gossip_use_per_app_project_id` (OFF-SPRINT-2b) — **already
     proven by `aed2303` tests (publish.rs round-trip + runtime.rs gossip)**
  5. `remettre_en_ligne_prefills_deploy` (front) — **greffe D already LIVE**

## S1a OSS Prior Art
- Domain: re-deploying a fork under a NEW local-author provenance attestation
  (the forker re-signs; the original author's provenance is never inherited —
  R5 seeder!=author, the SBFB embodiment of "rebuild from source, signed by the
  rebuilder").
- Sources (accessed 2026-06-07):
  - F-Droid Reproducible Builds / "Making reproducible builds visible" (2025-05)
    — https://f-droid.org/2025/05/21/making-reproducible-builds-visible.html +
    https://f-droid.org/docs/Reproducible_Builds/. F-Droid rebuilds the APK from
    source on its own infra and the rebuilder's attestation is distinct from the
    developer's signature; "build the app from source, then compare". **This is
    exactly the SBFB redeploy: clone the (forked) source, re-zip, re-sign
    provenance with the LOCAL key.** A fork that re-deploys produces ITS OWN
    attestation.
  - Radicle Heartwood — `rad clone` "creates a fork under your public key".
    **Delegate (signing authority) != seeder.** Validates that a fork becomes a
    new authorship under the forker's key (Phase C re-signs) and that hosting
    != authoring (D1/R5).
  - SLSA build provenance (https://slsa.dev/spec/draft/build-provenance, v1.0
    build track, no breaking change since the S14 Keyoxide decision) —
    provenance is "the signed story of which repo/commit produced the artifact,
    by which builder". A redeploy = a fresh build => a fresh provenance signed
    by the redeploying builder. **No SOTA evolution invalidates the S14
    auto-attestation model.**
- Finding: **APPROACH-ALIGNED**. The clone -> re-zip -> re-sign-with-local-key
  redeploy is precisely the mature OSS rebuild-from-source pattern. No
  `LIB-EXISTS` (the re-sign reuses the in-repo `generate_provenance` Ed25519),
  no `APPROACH-NAIVE`.
- Impact: none — proceed.

## S1b Dependencies, CVEs, Release Notes
- Scanned: `crates/sbfb-factory/Cargo.toml` + `Cargo.lock` for any redeploy dep.
- **NO NEW DEPENDENCY (clean).** Phase C wires `fork.rs` (already shipped Phase
  B, zero new dep) to a CLI command and reuses the existing daemon
  `deploy-from-repo` route (git CLI clone + `generate_provenance`, both already
  present). `sbfb-factory` deps are unchanged (`Cargo.toml:11-39`: zip, walkdir,
  reqwest, ollama-rs 0.3.4 pinned, tokio, axum — all pre-existing).
- Transitive graph (`cargo tree -d`): duplicates are the pre-existing iroh tree
  (base64 0.21/0.22, curve25519-dalek 4/5-pre, ed25519 2/3-rc, bitflags 1/2).
  Phase C adds nothing => no new collision (the S72 schemars-1.2 DESIGN-CONFLICT
  class does not apply — P2-PREFLIGHT-TRANSITIVE-DEPTH satisfied: lock + tree -d
  unchanged by this phase).
- CVE surface: untrusted-forge-clone and zip-slip are S3 concerns already
  mitigated in Phase B `fork.rs` (P0 git-arg-injection + P1 zip-bomb fixed,
  review §findings). No dependency CVE introduced.
- Finding: **clean**.

## S2 Historical Decisions
Each Phase C target carries a reverse-commit check. THREE plan items are
materially STALE (work already landed in the 9 post-audit Cas D hotfixes the
S73 audit predates), ONE is genuinely open with an under-specified mechanism.

- **OFF-SPRINT-2b (/publish + gossip per-app project_id) — ALREADY FULLY
  RESOLVED by `aed2303` (load-bearing).** The S73 audit
  (`sprint73_audit_findings.md:288`, ran on `845bea6..9472085`) flagged
  "Fix per-app incomplet : /publish + gossip gardent node_id". But hotfix
  `aed2303` ("per-app project_id on ProjectAnnouncement so a node hosts distinct
  Browse cards") landed AFTER the audit SHA. `git show aed2303 --format=%B`:
  "Fix (per-app identity on the wire, all paths): publish.rs ... http.rs
  publish_project ... deploy.rs ... runtime.rs handle_project_announcement".
  Verified in current code:
  - `/publish` derives `project_id = blake3(project_name)` (http.rs:951-953) and
    passes it to `publish_announcement` (http.rs:962) — NOT node_id.
  - `publish_announcement` sets `.with_project_id(project_id)` on the
    announcement (deploy.rs:405) AND the BrowseEntry (deploy.rs:446).
  - The wire struct `ProjectAnnouncement` carries BOTH `node_id` (hosting/
    dialable, publish.rs:40) AND `project_id` (per-app blake3, publish.rs:51),
    validated empty-or-64-hex (publish.rs:198-203).
  - The gossip RECEIVER keys the card on `ann.project_id`, falling back to
    `node_id` only for a legacy/empty announcement (runtime.rs:1660-1664).
  - Tests already exist: `project_id_round_trips_through_gossip_bytes`
    (publish.rs:239), `empty_project_id_is_tolerated_as_legacy` (publish.rs:259),
    `gossip_announcement_uses_per_app_id_and_indexes` (runtime.rs:1980),
    `gossip_legacy_announcement_falls_back_to_node_id` (runtime.rs:2057).
  Reverse-commit check: `aed2303` is a forward fix, never reverted (it is the
  latest functional change to publish.rs). **The plan's C.2 line numbers
  (http.rs:1004, runtime.rs:1569, publish.rs:39) are STALE: 1004 is the B.6 log
  line, 1569 is `jittered_republish_duration`, and `nexus-shell-daemon` has no
  `publish.rs`.** Consequence: OFF-SPRINT-2b degrades to a REGRESSION-TEST /
  no-op carry, NOT new code. SCOPE-CONSISTENT reduction.

- **OFF-SPRINT-2 (deploy per-app non-regression test) — ALREADY SHIPPED.** The
  audit (`sprint73_audit_findings.md:287`) said "0 test non-regression".
  `aed2303` added `multiple_apps_get_distinct_browse_cards` (http.rs:6932-6959):
  two published apps -> two distinct cards (HashSet of project_id len==2) + each
  individually searchable. The plan's `deploy_per_app_distinct_browse_cards` is
  the same assertion already present. Reduces to verifying/strengthening the
  existing test, not authoring it.

- **B.6 `is_open_source=>provenance` at the browse-index chokepoint — ALREADY
  DONE in Phase B (`bcfc155`).** `index_browse_entry` (http.rs:988-1012)
  downgrades `is_open_source` to `false` when `provenance_hash`/`repo_url` is
  absent + logs (http.rs:1000-1005). This covers all three callers (deploy,
  /publish, gossip-ingest runtime.rs:1715). Not a Phase C item, but relevant:
  the fork CONSUMER relies on this invariant being trustworthy (a hit claiming
  open-source-without-provenance is downgraded before it can drive a fork).
  Phase B preflight clarification #2 honored. No Phase C action.

- **Greffe D "La remettre en ligne" front — ALREADY LIVE (Phase A `457ca05`).**
  `BrowsedProject.tsx:538-564` renders the fallen-app card with a Link to
  `/deploy?repo_url=...&project_name=...` (gated `!archive_hash &&
  isHttpsUrl(entry.repo_url)`); `Deploy.tsx:57-64` reads `searchParams` to
  prefill the form. The plan's `remettre_en_ligne_prefills_deploy` front test
  and the "/deploy prerempli" item are mostly delivered. Phase C's remaining
  front work is "Forker dans l'atelier" (NOT yet present in Browse.tsx/
  BrowsedProject.tsx — `git grep "Forker"` = 0 UI matches) + connecting it to a
  real fork backend trigger.

- **Provenance re-signing on redeploy (R5 seeder!=author) — GENUINELY OPEN,
  mechanism UNDER-SPECIFIED.** `generate_provenance(repo_url, commit_sha,
  artifact_hash, &state.node_id, &state.pow_keypair)` (deploy.rs:167-173) signs
  with the LOCAL keypair and embeds the LOCAL node_id; `artifact_hash` is the
  blake3 of the FRESH local re-zip (deploy.rs:164). So any redeploy through
  `deploy-from-repo` STRUCTURALLY re-signs locally => R5 satisfied. BUT the
  plan says "redeploy via `publish_announcement`" and `publish_announcement`
  (deploy.rs:380-470) does NOT generate provenance — it only broadcasts/persists
  an already-built announcement. The ONLY provenance-signing path is
  `deploy-from-repo`, which CLONES from a `repo_url` (it does NOT deploy a local
  workspace directory). `deploy_private` (`/api/v1/deploy`, deploy.rs:320-359)
  stores a blob with `provenance_hash: None` and never announces. **There is no
  "deploy this local forked workspace under a locally-signed provenance" route
  today.** See clarification B — this is the load-bearing Phase C design choice,
  non-blocking (two valid in-repo paths exist) but must be decided explicitly.

- **CLI cabling — `fork.rs` is `#[allow(dead_code)]` (main.rs:13-14).** Phase C
  must add a `Fork` subcommand (the `Command` enum, main.rs:42-119) and/or wire
  the fork into the redeploy flow. The existing `Publish` command (main.rs:70-82)
  already takes `--repo-url` and runs `run_publish_pipeline` ->
  `post_deploy_from_repo` (pipeline.rs:48,72-118). G17/`repo_root` untouched
  (Phase B preflight S2: the fork workspace lives OUTSIDE `repo_root()`;
  `fork.rs` never derives `dest` from it).

- **Pre-launch protocol** (CLAUDE.md, kickoff §1.4): Phase C touches zero feed
  op, zero `*_VERSION`, zero migration. `ReleasePublished` already auto-inserted
  on deploy (deploy.rs:261-306) — a redeploy re-emits it, no change. Honored.

- Finding: **clean (no blocking S2)**. THREE plan items are already delivered
  (OFF-SPRINT-2b, OFF-SPRINT-2, greffe D) => regression-test/verify-only carry.
  ONE item (redeploy provenance mechanism) is open and under-specified =>
  load-bearing clarification (non-blocking; the re-sign invariant holds via
  deploy-from-repo).

## S3 Local Patterns And Threat Model (FULL — provenance re-signing is a
security-component change)
Phase C re-attributes authorship on redeploy and consumes untrusted forge
content. Full threat model:

- **Asset**: the provenance.json attestation binding (repo_url, commit_sha,
  artifact_hash, node_id, signature). The redeploy must produce an attestation
  signed by the LOCAL key with the LOCAL node_id.
- **Actors**: (a) the honest forker (local node, re-deploys an edited fork);
  (b) a byzantine forker who wants to republish someone else's app while
  CLAIMING to be the original author (re-attribution / usurpation, R5/AD4).
- **Vector — re-attribution / fork usurpation (R5)**: Can a node fork app X and
  republish it under its OWN identity pretending to BE the original author?
  Mitigations, all already in code:
  1. `generate_provenance` always signs with `state.pow_keypair` and embeds
     `state.node_id` (deploy.rs:171) — a forker CANNOT mint a provenance bearing
     the original author's node_id without the original author's private key
     (`verify_provenance` re-derives canonical bytes incl. node_id and checks
     the Ed25519 sig, provenance.rs:60-90). The attestation is NON-transferable.
  2. Content-addressing: the redeployed artifact is a FRESH blake3 hash (fresh
     re-zip, deploy.rs:164) => a different blob from the original => iroh-blobs
     blake3-verify on fetch (THREAT_MODEL §5.4 line 179) prevents serving altered
     bytes under the original hash.
  3. The browse card keys on `blake3(project_name)` (deploy.rs:141). A NAME
     collision is possible (two apps named "babel") but the panel renders the
     AUTHOR (provenance/signature) before any action (D-DISPO §8 verrou 4: "seed
     != autorite"). The fork is honestly a NEW author act, not a claim to be the
     original.
  - **Residual**: name-squat at discovery is the SAME pre-existing surface as
    any publish (AD4 "repo squat" THREAT_MODEL:76) — curator lists + quarantine
    remain the discovery-trust layer. Phase C does not regress it; the fork is
    transparently re-signed.
- **Vector — voluntary-seed must NOT re-sign (D-DISPO amendement, R5)**: the
  amendement (research §13) says a voluntary seeder re-announces "je detiens
  l'artefact *signe par l'auteur*" (same archive_hash, author provenance intact)
  and creates ZERO provenance. **Phase C must keep the FORK (re-sign) path
  strictly distinct from the SEED (provenance-intact) path.** Fork = new local
  author (re-sign); seed = serve the author's bytes (no re-sign). These land in
  different phases (fork=C, seed=D/E/F) — no conflict, but the distinction is
  load-bearing for the invariant.
- **Vector — untrusted forge/blob into the workspace**: already mitigated in
  Phase B `fork.rs` (git-arg-injection P0 fixed, zip-slip + symlink + zip-bomb +
  clone caps, https-only, workspace outside repo_root). The forked content is
  not executed during fork; it is re-zipped and deployed into an iframe sandbox
  later. No Phase C regression as long as the redeploy reuses the deploy-from-
  repo guards (it does — same route).
- **Vector — L2 consent on a forged open_source flag (THREAT_MODEL §5.6 line
  208)**: the redeploy goes through deploy-from-repo which only sets
  `is_open_source: true` after a verified clone (deploy.rs:256), and the
  /publish gate (http.rs:934) + the B.6 chokepoint (http.rs:1000) both enforce
  provenance presence. A fork redeploy cannot set a lying flag. Covered.
- **Regression check**: no covered T0-T5 threat is regressed. The redeploy
  inherits the §5.3 deploy-from-repo mitigations (clone caps, depth-1, https,
  path-traversal refuse) and the §5.6 L2 invariant.
- **HARDENING_ROADMAP**: no Phase-C pre-requirement pending; the relevant
  hardening (deploy-from-repo §5.3) already exists and is reused.
- Finding: **non-blocking**, provided Phase C (1) routes the redeploy through a
  path that re-signs with the LOCAL key (deploy-from-repo does; a new "deploy
  local workspace" route MUST call `generate_provenance` with
  `state.pow_keypair`/`state.node_id`), (2) keeps fork-re-sign strictly distinct
  from seed-no-re-sign, (3) reuses the deploy-from-repo untrusted-content guards.
  These are hard requirements the plan implies (R5), not findings against it.

## S4 Protocol And Wire Invariants
- Wire/security files checked: `ProjectAnnouncement` (publish.rs), `provenance.rs`,
  `deploy.rs` announcement build. NO `canonical.rs` edit, NO `schemas/`, NO new
  `DOMAIN_*`.
- `*_VERSION` status: `PROJECT_ANNOUNCEMENT_VERSION = 1` (publish.rs:24),
  `FEED_FORMAT_VERSION = 1` (public_feed.rs:20), `PROVENANCE_SCHEMA_VERSION = 1`
  (provenance.rs:15) — ALL stay 1. Phase C bumps none.
- Producer->consumer trace (P2-PREFLIGHT-WIRE-CONTRACT-DEPTH) for every field
  Phase C touches:
  - **`ProjectAnnouncement.project_id` (per-app id)**: producer =
    `publish_announcement` `.with_project_id` (deploy.rs:405) for self-publish/
    deploy, set from `blake3(project_name)` (http.rs:951, deploy.rs:141);
    consumer = the gossip receiver `handle_project_announcement` (runtime.rs:1660
    keys the card on `ann.project_id`, falls back to `node_id` when empty). Wire
    shape: `#[serde(default)]` String, validated empty-or-64-hex on decode
    (publish.rs:198-203). The legacy-empty tolerance is `serde(default)` runtime
    tolerance (pre-launch policy), proven by `empty_project_id_is_tolerated_as_
    legacy` (publish.rs:259). **Both ends agree; ALREADY shipped `aed2303`;
    Phase C does NOT change this contract** — it only verifies it.
  - **`provenance.json` (the re-signed attestation)**: producer =
    `generate_provenance` -> `provenance_to_json` injected into the zip
    (deploy.rs:204-216) + `insert_provenance_record` keyed by per-app project_id
    (deploy.rs:240); consumer = `GET /api/v1/project/{id}/provenance` ->
    front `VerificationDetail` (verify via the node's public key). Canonical
    bytes are `DOMAIN_PROVENANCE_V1` + JSON of {artifact_hash, commit_sha,
    node_id, repo_url, schema_version, timestamp} (provenance.rs:110-123). A
    redeploy produces a NEW record (new artifact_hash, local node_id) at the
    same project_id — overwrites the fallen app's stale record. **No shape
    change; the signing domain is stable.**
  - **`ReleasePublished` feed op (auto-emitted on redeploy)**: producer =
    deploy.rs:261-306 (raw-op `serde_json::Value`, no version bump per pre-launch
    policy); consumer = `feed_sync` ingest + search index. Unchanged by Phase C
    (a redeploy simply re-emits it). **No bump.**
- `serde(default)` audit: the only `serde(default)` in scope is
  `ProjectAnnouncement.project_id` (legacy-empty runtime tolerance, documented)
  and `ProvenanceRecord.app_version` (provenance.rs:27, optional). Both
  legitimate runtime tolerance, not silent wire drift.
- Day 0 status: **preserved**. Phase C is D5 "Segment SUR" (fork-redeploy under
  local identity, no new cross-node protocol). D1/D3/D4 (ALPN SeedRequest,
  SeedAnnounced, invite) are Phases E/F — untouched. R5 (seeder!=author) honored
  by local re-sign. D-DISPO invariants (0 host field; author sealed separately;
  intentions-not-jargon CTA) honored: "Forker dans l'atelier" / "La remettre en
  ligne" are intentions, the redeploy is an author act.
- Finding: **clean** (0 `*_VERSION` bump, 0 canonical edit, 0 migration, both
  ends of every touched field read and confirmed unchanged).

## Risks And Scope Cuts
- **Blocking risks: none.**
- **Non-blocking findings (the SCOPE-CUT-CONSISTENT basis):**

  1. **OFF-SPRINT-2b is ALREADY DONE (`aed2303`).** The plan's C.2 line
     references (http.rs:1004, runtime.rs:1569, publish.rs:39) are stale and the
     work shipped in a post-audit hotfix the S73 audit predates. Phase C must
     NOT re-implement per-app project_id; it adds/verifies a regression test
     (`publish_and_gossip_use_per_app_project_id`) pinning the existing behavior
     (publish.rs round-trip + runtime.rs gossip-id tests already prove it).

  2. **OFF-SPRINT-2 non-regression test ALREADY EXISTS**
     (`multiple_apps_get_distinct_browse_cards` http.rs:6932). The plan's
     `deploy_per_app_distinct_browse_cards` is the same assertion. Verify/rename;
     do not author from scratch.

  3. **Greffe D "La remettre en ligne" + /deploy prefill ALREADY LIVE**
     (Phase A `457ca05`, BrowsedProject.tsx:538-564 + Deploy.tsx:57-64). Phase
     C's remaining front work is the NEW "Forker dans l'atelier" CTA (0 UI
     matches today) wired to a real fork backend trigger, NOT the redeploy
     prefill (done).

  4. **Redeploy provenance mechanism is under-specified (load-bearing,
     clarification B).** "Redeploy via `publish_announcement`" is imprecise:
     `publish_announcement` does NOT sign provenance. The re-sign happens only
     in `deploy-from-repo` (which clones a repo_url) — there is no "deploy a
     local forked workspace under a locally-signed provenance" route. The
     executor must pick one of two in-repo paths and state it in the commit body
     (see clarification B). Either path satisfies R5 (local key/node_id), so
     this is NON-blocking — it is a design choice, not a conflict.

  5. **Templates react/pyodide DO NOT EXIST** (template_engine.rs:102-119 has
     only `static` + `static-reader`). PO Q7 (kickoff §11, plan §C.1 line 205,
     scope cut #9 "retire des scope cuts") INCLUDES react+pyodide => Phase C
     must CREATE two new `TemplateConfig` entries + their `templates/react/*`
     and `templates/pyodide/*` `include_str!` asset files (index.html,
     sbfb-bridge.js, README.md, gitignore each) + tests mirroring the
     `static-reader` test block (template_engine.rs:418-466). This is real new
     code (clarification C) — NOT scope creep, it is an explicit PO inclusion;
     but it is the largest concrete deliverable left in Phase C and must not be
     under-scoped. The fork-redeploy loop itself is mostly wiring existing
     primitives.

  6. **CLI cabling: `fork.rs` is `#[allow(dead_code)]` (main.rs:13).** Phase C
     adds a `Fork`/redeploy subcommand to the `Command` enum and removes the
     `#[allow(dead_code)]`. The fork workspace MUST be created outside
     `repo_root()` (Phase B preflight; `fork.rs` already never derives it).

  7. **E2E test infra (clarification D).** `fork_redeploy_loop_e2e_single_node`:
     the fork clones a real forge (network) — the unit tests avoid this by
     cloning a LOCAL fixture repo via `run_git_clone` (fork.rs:448, two-commit
     fixture) and by reconstructing from in-memory zip bytes (fork.rs:514). The
     E2E should reuse `mk_state()`/`build_test_router` (http.rs test harness)
     for the daemon side and a local git fixture (or the blob path) for the fork
     side — NO real network. The §P57 "real boundary both sides" rule applies to
     the deploy/announce/index round-trip, not to reaching an external forge.

- **Scope cuts still honored** (kickoff §7 / plan §7): #8 Monaco editor (never)
  untouched; #1 GPU cross-machine (S75), #2 quorum cross-machine (S75), E-F
  cross-node protocol NOT started. #9 templates react/pyodide are PULLED IN
  (PO Q7) — honored as inclusion, not a cut.

## Action
- **SCOPE-CUT-CONSISTENT: proceed with Phase C, honoring these load-bearing
  clarifications:**
  1. **OFF-SPRINT-2b = regression test only.** Do NOT re-implement per-app
     project_id (shipped `aed2303`, all paths). Add
     `publish_and_gossip_use_per_app_project_id` pinning the existing wire
     contract (or extend the existing publish.rs/runtime.rs id tests). Cite
     `aed2303` in the commit body and correct the stale line numbers.
  2. **OFF-SPRINT-2 = verify the existing test** `multiple_apps_get_distinct_
     browse_cards` (http.rs:6932); rename/alias to the plan name if useful.
  3. **Redeploy MUST re-sign with the LOCAL key.** Pick ONE in-repo path and
     state it in the commit body: (A) the forker pushes the edited fork to their
     OWN https forge, then redeploy via the existing `deploy-from-repo`
     (clones the new repo_url, re-zips, `generate_provenance(... &state.node_id,
     &state.pow_keypair)` — R5 free); or (B) add a "deploy local workspace"
     daemon path that re-zips the fork dir + calls `generate_provenance` with the
     LOCAL keypair/node_id + `publish_announcement`. `publish_announcement` alone
     does NOT sign — do not present it as the re-sign mechanism.
  4. **Keep fork-re-sign STRICTLY distinct from seed-no-re-sign** (R5 /
     D-DISPO amendement): fork = new local author (re-sign); voluntary seed
     (Phase D/F) = serve the author's bytes, provenance intact, ZERO new
     provenance. The `fork_redeploy_resigns_provenance_as_local_node` test must
     assert the record's `node_id` == local node_id and the signature verifies
     under the LOCAL public key.
  5. **Create the react + pyodide templates** (PO Q7): two `TemplateConfig` +
     `templates/{react,pyodide}/*` assets + tests. Largest concrete deliverable
     — do not under-scope.
  6. **Cable `fork.rs`**: add the `Fork`/redeploy subcommand, drop
     `#[allow(dead_code)]`, keep the workspace outside `repo_root()`.
  7. **Front**: add the NEW "Forker dans l'atelier" CTA (Browse/BrowsedProject)
     wired to the fork backend; the "remettre en ligne" prefill is already LIVE
     (do not duplicate). Normalize `isHttpsUrl` on the new repo_url anchor.
  8. **No wire bump, no canonical edit, no migration.** All Phase C surfaces are
     LOCAL (provenance record, FTS5 index, factory workspace, templates) or
     already-shipped wire (per-app project_id). E2E reuses `mk_state`/local git
     fixture — no real network.
- The commit body must cite this preflight under `## G8 traceability`, note the
  OFF-SPRINT-2/2b reclassification (already shipped `aed2303` -> regression
  test), the greffe-D already-live status, the chosen redeploy re-sign path
  (A or B), and the react/pyodide template creation.
