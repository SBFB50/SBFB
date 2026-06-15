# Sprint 76 Phase A Preflight

Date: 2026-06-15
HEAD: `3faee6e`
Verdict: **SCOPE-CUT-CONSISTENT**

## Evidence Rules
- Claim policy: every claim below cites a path, command output, URL/date, or
  explicit assumption.
- Local sources read:
  - `.planning/active/sprint76_plan.md` (Phase A, lines 125-213)
  - `.planning/active/sprint76_kickoff.md` (D1, lines 255-332)
  - `.planning/active/sprint76_design_review.md` (D1 finding, lines 27-39)
  - `crates/nexus-worker-core/src/engine/state_writer.rs` (lines 1-204)
  - `crates/nexus-worker-core/src/engine/runtime.rs` (lines 59-528, 925-982, 1184-1223)
  - `crates/nexus-worker-core/src/consent.rs` (lines 80-435)
  - `crates/nexus-shell-daemon/src/local_worker.rs` (lines 1-320)
  - `crates/nexus-shell-daemon/src/consent.rs` (full)
  - `crates/nexus-shell-daemon/src/http.rs` (lines 410-498)
  - `crates/nexus-shell-daemon/src/worker_state_api.rs` (lines 1-30)
  - `crates/nexus-shell-daemon-core/src/auth.rs` (lines 60-85)
  - `web/src/api/consent.ts` (full)
  - `web/src/api/coordinator.ts` (lines 1-130, 355-414, 700-707)
  - `web/src/api/auth.ts` (full)
  - `web/src/api/bootstrap.ts` (full)
  - `web/src/vite.config.ts` (full)
  - `web/src/pages/Network.tsx` (lines 1-166, 367-430)
  - `web/src/components/GpuConsentDialog.tsx` (lines 130-353)
  - `web/src/stores/projectStore.ts` (lines 31-160)
  - `docs/security/THREAT_MODEL.md` (consent/R4 sections)
- External local source: project memory `MEMORY.md` + `nexus_grid_pivot.md`
  (S76 D1 arbitrage) referenced by `CLAUDE.md`.
- Commands run:
  - `git log --oneline -8 -- web/src/api/consent.ts` -> last touch `1f79c52`
    (S29), creation `3247e88` (S16 Phase C).
  - `git log --oneline -8 -- crates/nexus-shell-daemon/src/consent.rs` ->
    creation `a766496` (S43 Phase B "files + consent API Rust").
  - `cargo tree -d --workspace` -> only pre-existing duplicates (base64
    0.21/0.22 via ron/hickory vs iroh chain); no new collision.
  - `git diff --stat -- Cargo.toml Cargo.lock crates/.../Cargo.toml` -> empty
    (no pending dep change).

## Scope
- Plan source: `.planning/active/sprint76_plan.md` §4 (lines 125-213).
- Target files:
  - `web/src/api/consent.ts` (route prefix reconciliation)
  - `web/vite.config.ts` (dev proxy assessment)
  - `crates/nexus-worker-core/src/engine/state_writer.rs` (additive
    `ConsentSnapshot`)
  - `crates/nexus-worker-core/src/engine/runtime.rs` (pump -> snapshot)
  - `crates/nexus-shell-daemon/src/local_worker.rs` (co-located enrollment)
  - `web/src/api/coordinator.ts` (`WorkerStateV1` + optional `consent`)
  - `web/src/pages/Network.tsx` + `web/src/components/GpuConsentDialog.tsx`
    (offer-my-power page + L4 double-confirm)
- Deps/APIs/specs: none added or bumped (verified empty `Cargo.toml` diff).
- Security/protocol surfaces: `consent.json` schema (NOT a signed canonical),
  `WorkerStateSnapshot`/`SCHEMA_VERSION` (file contract, not P2P wire),
  loopback HTTP route prefix (HTTP only, no canonical bytes).
- Tests expected: A.3 #1-#7 (5 Rust + 2 Vitest).

## S1a OSS Prior Art
- Domain: voluntary GPU enrollment + consent/preference exposure for a
  distributed compute node.
- Sources (read 2026-06-15):
  - BOINC local preferences / GPU computing: the GPU sharing mode (`Use GPU
    always` / `based on preferences` / `Suspend GPU`) IS the toggle; stored in
    `global_prefs_override.xml`, edited via the Manager dialog, "affect only the
    local computer". https://boinc.berkeley.edu/wiki/Local_preferences ,
    https://boinc.berkeley.edu/wiki/GPU_computing
  - Petals `--public_name` opt-in public-swarm contribution; explicit
    private-swarm-vs-public-swarm split; voluntary "give back" model.
    https://github.com/bigscience-workshop/petals/blob/main/README.md
- Finding: **APPROACH-ALIGNED**.
  - The "level IS the state, no separate `enabled` flag" decision (kickoff D1,
    rejected `worker_enabled: bool`) matches BOINC exactly (the mode is the
    control; there is no orthogonal on/off boolean). Verified locally:
    `consent.rs:397-413` already makes the level the single admission gate,
    fail-closed at `runtime.rs:974-979`.
  - The "public enrollment is opt-in, least-privilege otherwise" decision
    matches Petals (`--public_name` public-swarm opt-in vs private swarm).
  - The mature primitive (4 levels + caps + `UsageTracker` +
    `ConsentWatcher`, 25 tests) already exists; the phase is genuinely UI
    exposition + one wiring gap, NOT a missing primitive. No mature
    license-compatible library would replace an in-tree consent engine that is
    already shipped and tested (`LIB-EXISTS` does not apply).
- Impact: none (no adaptation required).

## S1b Dependencies, CVEs, Release Notes
- Scanned: `Cargo.toml` / `Cargo.lock` (no change), `cargo tree -d`,
  `web/package.json` (zod already present, used by `consent.ts` and
  `coordinator.ts`).
- Commands/sources:
  - `git diff --stat -- Cargo.toml Cargo.lock ...` -> empty: the phase adds no
    crate and bumps no version. It only reuses existing types (`ConsentLevel`,
    `UsageTracker`, `WorkerStateSnapshot`, axum routes, zod).
  - `cargo tree -d --workspace` -> duplicates are all pre-existing
    (`base64 0.21` via `ron`/`config` + `hickory`, `base64 0.22` via
    `iroh`/`igd-next`). None is introduced or aggravated by this phase. This
    matches the documented exemption posture (P2-AUDIT externals unchanged).
- Finding: **clean**. No transitive collision risk (lesson S72 ollama-rs ->
  schemars 1.2 does not recur: no dep is touched).

## S2 Historical Decisions
- Commands: `git log --oneline -- web/src/api/consent.ts`,
  `git log --oneline -- crates/nexus-shell-daemon/src/consent.rs`,
  `git log --oneline -- web/src/components/GpuConsentDialog.tsx`, plus
  `rg` on consent/Whitelist hardcode.
- Decisions crossed:
  1. **Sprint 5 D3 (worker CLI-only, file snapshot, daemon = single HTTP
     point)** — `state_writer.rs:14-20`. Status: STILL VALID and NOT violated.
     The phase adds an additive field to the file snapshot and reads
     `self.consent.current()` already present in the pump; it does NOT add an
     HTTP endpoint to the worker binary (kickoff D1 explicitly rejects the
     vast.ai daemon model). Reverse-commit check: no commit reverses D3; the
     additive-field path is exactly the mechanism D3 sanctions
     (`state_writer.rs:23-29`). Non-blocking, honored.
  2. **`SCHEMA_VERSION = 1` + "additive optional fields stay on the same
     version"** — `state_writer.rs:22-29,53-54`. Status: VALID. Adding
     `#[serde(default)] consent: Option<ConsentSnapshot>` is the canonical
     additive change the doc-comment authorizes; `SCHEMA_VERSION` stays 1.
     Consistent with the Pre-launch protocol policy in `CLAUDE.md`.
  3. **Local worker `Whitelist[own_doc]` hardcode** — `local_worker.rs:305-308`,
     created at the 2026-06-05 platform-remediation hotfix #5. The doc-comment
     (`local_worker.rs:32-36`) records the rationale: least-privilege so the
     co-located worker only serves the node's own doc, AND a live-smoke finding
     that `OwnProjects` would *reject* the coordinator's own tasks (the
     `project_id` is the doc id, not the worker node id). Status: the hardcode
     is a deliberate least-privilege default, NOT a rejection of "honor user
     consent". Reverse-commit check: no commit forbids reading the user
     `consent.json`; the comment frames it as the current default, not an
     invariant. The S76 change keeps `Whitelist[own_doc]` for
     `OwnProjects`/`Whitelist` and only widens it when the USER chose
     `OpenSource`/`All` (kickoff D1 arbitrage, gele). Non-blocking, consistent.
  4. **Consent client route `/consent/set`** — `web/src/api/consent.ts:80,122`,
     last meaningfully touched S16/S29, BEFORE the route moved to Rust at
     `/api/v1/consent` (S43 Phase B `a766496`). The client was never updated to
     the `/api/v1` prefix. See S3/S4 for the impact analysis. This is a
     pre-existing latent bug, not a rejected decision.
- Finding: **clean** (no un-reversed blocking decision). One non-blocking
  observation: the `/api/v1` prefix drift (item 4) is a real prod gap, analyzed
  below and resolved in-phase per the plan's pre-requisite task.

## S3 Local Patterns And Threat Model
- Threats/contracts checked:
  - **R4 — consent.json race (severity M)**, `THREAT_MODEL.md:342-351`. The
    co-located worker serving public when the user picks `All` widens the blast
    radius of R4 (malware writing `consent L4 + cap infini`). Mitigations
    already present and reused unchanged: caps are enforced regardless of level
    (`consent.rs:417-432`); `ConsentWatcher` re-reads on file change with
    50 ms debounce (`THREAT_MODEL.md:283`, `runtime.rs:394`); admission is
    fail-closed if consent is unreadable (`runtime.rs:974-979`). The kickoff D1
    double-confirm for `All` is an ADDED mitigation surface, consistent with
    the existing `threatNote` "Risque maximum" already shown for L4
    (`consent.rs:84-96`). Verified the dialog has NO double-confirm today
    (`GpuConsentDialog.tsx:340-353` saves directly), so the double-confirm is
    net-new but contained UI.
  - **A4 — user consent preferences asset** (`THREAT_MODEL.md:62`): stored
    `~/.sbfb/consent.json` atomic tmp+rename. The daemon write path
    (`consent.rs:135-145`) and the worker provision path
    (`local_worker.rs:309-311`) both already use atomic tmp+rename. No
    regression.
  - **Fail-closed consent gate** (`runtime.rs:936-982`): the phase does not
    touch the gate logic; it only feeds the *level* into the provisioned
    `consent.json` and the *snapshot*. The pure `should_accept_task`
    (`consent.rs:391-435`) is untouched (0 logic change, plan §4.1).
- HARDENING_ROADMAP status: no S76-specific pre-requirement is unmet for this
  phase; the consent hardening line (sign consent.json, R4 roadmap Sprint 17+,
  `THREAT_MODEL.md:351`) is a documented FUTURE gap, not a regression, and is
  out of this phase's scope.
- Finding: **clean** with one non-blocking note. The phase increases the
  practical exposure of R4 (more users will reach `All`) but adds the
  double-confirm and reuses every existing R4 mitigation; the residual is the
  same pre-existing M-severity race already accepted in the threat model. To
  honor S3, the implementation MUST (a) keep the caps + fail-closed gate
  untouched, (b) require the double-confirm before any `All` save, and (c) NOT
  introduce a worker HTTP endpoint (preserves the loopback-hardened single HTTP
  surface).

## S4 Protocol And Wire Invariants
- Wire/security files checked: `state_writer.rs` (`SCHEMA_VERSION`,
  `WorkerStateSnapshot`), `worker_state_api.rs` (daemon proxy),
  `coordinator.ts` (`WorkerStateV1Schema`), `consent.rs` (daemon + worker-core
  consent struct), `consent.ts` (front consent client), `http.rs` (route
  table).
- VERSION/domain/canonical status:
  - `SCHEMA_VERSION = 1` (`state_writer.rs:54`) MUST stay 1. The `consent`
    field is additive + optional (`#[serde(default)] Option<ConsentSnapshot>`)
    -> NO bump. This is the file-snapshot contract, NOT a signed canonical and
    NOT a P2P wire (`*_ANNOUNCEMENT_VERSION` / `FEED_FORMAT_VERSION` /
    `DOMAIN_*` are not touched). Confirmed by reading the full route table:
    consent and worker-state are loopback HTTP only.
  - No signed canonical (`Task`, `ResultPayload`, capability advert,
    `ProjectAnnouncement`, `CuratorList`) is read or written by this phase.
    `model_digest`/`logprobs_hash` (the D3/S77 surface) are NOT touched here.
- **Producer -> consumer trace (P2-PREFLIGHT-WIRE-CONTRACT-DEPTH):**
  1. `WorkerStateSnapshot.consent` (NEW additive field):
     - Producer: `state_writer.rs from_inputs` builds the struct;
       `runtime.rs:1212-1222 flush_state_snapshot` populates `SnapshotInputs`
       (this is where `self.consent.current()` + `self.usage` must be passed).
     - Transport: daemon `worker_state_api.rs:25` reads
       `<root>/worker/state.json` and returns it VERBATIM (no struct on the
       daemon side -> no daemon change needed; pass-through proven).
     - Consumer: `coordinator.ts:393 WorkerStateV1Schema` is a plain
       `z.object()` (NOT `.strict()` -- verified line 393 vs the `.strict()` at
       line 303 which belongs to a different schema). Therefore the additive
       `consent` key is silently stripped by an OLD shell and parsed by the NEW
       one. The plan's `consent?` optional Zod is the correct shape:
       **`.optional()` (key absent on an old worker), not `.nullable()`** --
       because the Rust side serializes the field as ABSENT when `None`
       (`#[serde(default)] Option`, no `serialize_with`), not as JSON `null`.
       This is the opposite of the S73 `SearchResult` case (always-present-as-
       null -> `.nullable()`): here the producer omits the key, so
       `.optional()` is right. Confirm at implementation time that the Rust
       struct does NOT carry `#[serde(skip_serializing_if = ...)]`-free `null`
       emission; default `Option` with no skip attribute emits `null`, so the
       Zod must be `z.object({...}).optional()` on the field AND tolerate
       `null` -> use `.optional().nullable()` to be safe, OR add
       `#[serde(skip_serializing_if = "Option::is_none")]` on the Rust field so
       the key is omitted and `.optional()` alone suffices. **Pick the omit +
       `.optional()` path for the cleanest contract.**
  2. `consent.json` (read by co-located worker enrollment, written by dialog):
     - Producer: dialog `setConsent` -> daemon `set_consent`
       (`consent.rs:155-178`) writes `<auth::sbfb_home>/consent.json` (the
       USER home, `auth.rs:72-85`).
     - Consumer: `local_worker.rs provision` currently writes a SEPARATE
       `consent.json` under the worker's ISOLATED home
       (`home.join("sbfb")/consent.json`, `local_worker.rs:303-311`). The S76
       change reads the USER `consent.json` (`auth::sbfb_home`) to decide the
       provisioned level. Both ends use the same `ConsentConfig` serde shape
       (`worker-core consent.rs:162-176`), so the contract holds. The two enum
       reps agree: front `ConsentLevelSchema` (int 1..=4, `consent.ts:22-27`),
       daemon `level: u8` 1..=4 (`shell-daemon/consent.rs:159`), worker-core
       `ConsentLevel` `#[serde(into="u8", try_from="u8")]` 1..=4
       (`worker-core/consent.rs:84-129`). Byte-for-byte consistent.
- Day 0 status: **preserved**. D1 arbitrage (OpenSource/All open public,
  OwnProjects/Whitelist least-priv, All double-confirmed) is implemented as
  written; no Day-0 contradiction.
- Finding: **clean**. The route-prefix bug (below) is HTTP-only, not a wire
  format -- it does not bump any `*_VERSION` and touches no canonical bytes.

## Challenge Findings (the two factual questions asked)

### Finding 1 (LOAD-BEARING) -- the route pre-requisite IS a real prod gap, in BOTH dev and packaged builds.
- Evidence:
  - Front consent client posts to un-prefixed paths: `consent.ts:80`
    (`/consent/get`), `:122` (`/consent/set`), `:129`/`:138`
    (`/consent/whitelist/*`). `baseUrl` resolves to a full origin like
    `http://127.0.0.1:8765` (`projectStore.ts:31`, seeded as
    `window.location.origin` by `bootstrap.ts:51-62`). So the runtime URL is
    `http://127.0.0.1:<port>/consent/set`.
  - Daemon mounts ONLY `/api/v1/consent`, `/api/v1/consent/set`,
    `/api/v1/consent/whitelist/add|remove` (`http.rs:423-432`). There is NO
    `/consent/set` route, NO `.nest("/consent", ...)`, and NO catch-all rewrite
    (verified by `rg`). Unmatched non-`/api` paths fall to the SPA fallback
    `ServeDir + ServeFile(index.html)` (`http.rs:497-498`), which is GET-only
    static serving -> a `POST /consent/set` returns 405/404 and never reaches
    `set_consent`. **The dialog save is inert in a packaged build.**
  - Dev proxy does NOT cover it either: `vite.config.ts:54-56` proxies only
    `'/api'` -> `http://localhost:8000`. `/consent/*` is NOT under `/api`, so
    the Vite dev server handles it itself and returns its own 404. The dialog
    is inert in dev too (only every OTHER daemon call -- all `/api/v1/*` --
    works).
  - Corroborating drift: every other daemon client in `coordinator.ts` uses
    `/api/v1/*` (e.g. `getWorkerState` -> `/api/v1/worker/state`,
    `coordinator.ts:706`), and the module doc-comment states "Standard routes
    call the daemon at `/api/v1/*`" (`coordinator.ts:9-11`). `consent.ts`
    predates the Rust route move (S43 `a766496`) and was left behind.
- Verdict on the question: **it is a `fix(sprint76)` legitimately inside Phase
  A, not a report.** The cleanest fix is to add the `/api/v1` prefix in
  `consent.ts` (4 call sites: `consentGet` `/api/v1/consent`, `consentPost`
  `/api/v1/consent/set`, whitelist add/remove `/api/v1/consent/whitelist/*`).
  Do NOT widen the Vite proxy to `/consent` (that masks the prod gap and
  diverges from the `/api/v1` convention). Acceptance criterion #5: a POST
  consent from the front client reaches `/api/v1/consent/set` and writes
  `consent.json` (test A.3 #5). Note: the front Zod expects an enriched body on
  GET (`level_threat_note`, `residual_threats_acknowledged`) which the daemon
  `enrich()` provides (`shell-daemon/consent.rs:117-121,151-153`); the
  round-trip shape already matches.

### Finding 2 -- the co-located enrollment change is THIN wiring, not a provisioning refactor.
- Evidence: `local_worker.rs provision` (`:259-313`) already: resolves a
  dedicated worker home, writes `worker.toml` + key + allowlist, then writes a
  `consent.json` whose ONLY enrollment-specific lines are `:305-308`
  (`ConsentConfig::default_for("local-worker")`, `level = Whitelist`,
  `allowed_project_ids.insert(project_id)`). The S76 change is localized to
  those lines plus one read of the user `consent.json`:
  - read the user level via `auth::sbfb_home()/consent.json` (the same path the
    daemon writes, `auth.rs:72-85`);
  - if user level is `OpenSource` or `All`, set the provisioned
    `consent.level` to that level (and for `All`, gate on the double-confirm
    already captured in the user file); otherwise keep
    `Whitelist[own_doc]` least-privilege as today.
  No change to home resolution, key generation, allowlist enroll, ticket
  minting, the Job-Object lifetime, or the supervisor. The pure consent gate
  (`should_accept_task`) is untouched. **Contained wiring, confirmed.**
- One implementation caveat to honor: the provisioned worker's
  `consent.own_node_id` must remain the worker's own id (so `OwnProjects` still
  matches the node's own doc); only `level` (and, for the public case, the caps
  copied from the user file) changes. Do not blindly copy the user file's
  `own_node_id` into the worker file -- copy `level` + `caps`, keep the
  worker's own node id and the own-doc whitelist entry as the floor.

## Risks And Scope Cuts
- Blocking risks: none.
- Non-blocking risks (carry/track):
  - The pre-existing `/api/v1` prefix bug also affects `addToWhitelist` /
    `removeFromWhitelist` from `BrowsedProject.tsx:703-704` (same wrong
    prefix). The in-phase fix in `consent.ts` resolves all four call sites at
    once -- ensure the "Contribuer mon GPU" L3 flow is re-verified after the
    fix (no separate ticket needed; same file).
  - The `WorkerStateSnapshot` additive field must use omit-on-None
    (`#[serde(skip_serializing_if = "Option::is_none")]`) + Zod `.optional()`
    to keep the cleanest contract; otherwise use `.optional().nullable()`.
    Decide at implementation, do not ship a `.strict()` schema for
    `WorkerStateV1`.
  - `Network.tsx:82` shows a stale display string `{url}/worker-state` (the
    real path is `/api/v1/worker/state`). Cosmetic; fix opportunistically while
    on the page or leave -- not load-bearing.
  - THREAT_MODEL.md:400-401 still references the stale `/consent/get`+
    `/consent/set` (un-prefixed) paths; update to `/api/v1/consent*` while
    documenting the R4 exposure delta (doc-only, in the phase's THREAT_MODEL
    touch if any, else a non-blocking carry).
- Scope cuts still honored (kickoff §4 D1 "Rejete"):
  - No BOINC `global_prefs` scheduler (idle detection / day-of-week).
  - No separate `worker_enabled: bool` flag (level is the state).
  - No HTTP endpoint on the worker binary (Sprint 5 D3 preserved).
  - No blocking self-test / benchmark of enrollment.

## Action
- **SCOPE-CUT-CONSISTENT**: proceed with Phase A as planned. The only findings
  are non-blocking: (1) the route-prefix reconciliation is a real prod gap that
  the plan already scopes as the in-phase pre-requisite `fix(sprint76)` -- fix
  it in `consent.ts` by adding the `/api/v1` prefix (4 call sites), not by
  widening the Vite proxy; (2) the additive snapshot field must use
  omit-on-None + Zod `.optional()` and `WorkerStateV1` must stay non-`.strict()`;
  (3) copy only `level` + `caps` (not `own_node_id`) into the co-located
  worker's provisioned consent, keeping the own-doc whitelist as the floor;
  (4) require the `All` double-confirm before save (net-new but contained UI).
  Track the carry-overs above. No pivot proposal required; Day 0 (D1) preserved.
