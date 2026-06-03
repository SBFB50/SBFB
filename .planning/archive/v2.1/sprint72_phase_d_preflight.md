# Sprint 72 Phase D Preflight

Date: 2026-06-03
HEAD: `3c9ea1b`
Verdict: **DESIGN-CONFLICT**

## Evidence Rules
- Claim policy: every claim below cites a path, command output, URL/date, or
  explicit assumption.
- Local sources read: `prompts/agent/preflight.md`,
  `.planning/active/sprint72_plan.md` (§2, §7, §12),
  `.planning/active/sprint72_kickoff.md` (§4 D1-D5, §9 R1-R7),
  `crates/sbfb-factory/src/provider_router.rs`,
  `crates/sbfb-factory/src/operator_server.rs`,
  `crates/sbfb-factory/src/llm_bridge.rs`,
  `crates/sbfb-factory/src/daemon_client.rs`,
  `crates/sbfb-factory/src/auth.rs`,
  `crates/sbfb-factory/Cargo.toml`, `Cargo.toml` (workspace),
  `crates/nexus-shell-daemon/src/http.rs`,
  `crates/nexus-shell-daemon/src/tasks_api.rs`,
  `crates/nexus-shell-daemon-core/src/auth.rs`,
  `crates/nexus-coordinator-rs/src/types.rs`,
  `crates/nexus-coordinator-rs/src/validator.rs`,
  `crates/nexus-coordinator-rs/src/dispatcher.rs`,
  `crates/nexus-coordinator-rs/src/db.rs`,
  `crates/nexus-core-rs/src/task.rs`,
  `docs/security/THREAT_MODEL.md` (§14),
  `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md` (§3.1),
  `docs/rust/PATTERNS.md` (§P53).
- Commands run: `git rev-parse --short HEAD` -> `3c9ea1b`;
  `git log --all --oneline -- <target files>`;
  `cargo tree -p sbfb-factory -e normal` (reqwest 0.12.28 direct, async-stream
  0.3.6, ollama-rs 0.3.4); `cargo tree -i 'reqwest@0.13.3'` (0.13.3 is iroh
  transitive, not sbfb-factory's direct dep); `grep` on Cargo.lock
  (async-stream 0.3.6, ollama-rs 0.3.4, reqwest 0.12.28 + 0.13.3, tokio
  1.52.3).

## Scope
- Plan source: `.planning/active/sprint72_plan.md` §7 (Phase D), with Day 0
  D3/D4/D5 from §2 and kickoff §4.
- Target files:
  - `crates/sbfb-factory/src/provider_router.rs` (D3 `Network` arm — replaces
    the `network_not_implemented()` stub at lines 241-249).
  - `crates/sbfb-factory/src/operator_server.rs` (D4 — `ChatSession +provider`
    at :52, persist at :758, dispatch at :822-898).
  - `docs/rust/PATTERNS.md` (D5 §P55, 3 axes).
- Deps/APIs/specs: `reqwest` (async client), `async-stream`, `tokio::time`.
  All already present (no new dependency).
- Security/protocol surfaces: daemon loopback `POST /api/v1/tasks/submit`,
  `GET /api/v1/tasks/{task_id}` (auth `X-SBFB-Token` + Host + Origin);
  Operator SSE `GET /chat/{id}/stream`; `SENSITIVE_ACTIONS` gate.
- Tests expected (plan §7.3): `chat_session_persists_provider`,
  `chat_stream_routes_by_session_provider`,
  `network_provider_submit_poll_yields_single_done`,
  `network_provider_poll_timeout`,
  `sensitive_action_gated_regardless_of_provider`.

## S1a OSS Prior Art
- Domain: HTTP submit-then-poll adapted into a `Pin<Box<dyn Stream>>` of
  unified chunks (async, non-streaming source).
- Sources:
  - kickoff §"OSS prior art — adapter polling -> stream" (medium.com Mitesh S.
    Jat 2026-03 "Submit and Poll"; developer.atlassian.com Forge Realtime 2026;
    tokio.rs/tokio/tutorial/streams; docs.rs `async-stream` 0.3.6) — all dated
    2026-05-31.
  - In-tree reference: `provider_router.rs:158-238` (`ollama_stream`) already
    implements the exact `async_stream::stream! { loop { match
    tokio::time::timeout(idle, stream.next()).await { ... } yield ... } }`
    pattern. The Network arm reuses this mechanism with a `tokio::time::interval`
    poll instead of an upstream chunk stream.
- Finding: **APPROACH-ALIGNED**. Submit+poll-to-single-final-chunk is the
  mature standard for async (non-streaming) backends; the repo already proves
  the async-stream poll-loop mechanics compile and pass in this crate.
- Impact: none on the chosen mechanism. (The conflict below is a wire/contract
  gap, not an approach flaw.)

## S1b Dependencies, CVEs, Release Notes
- Scanned: `reqwest`, `async-stream`, `tokio`, `ollama-rs` (Phase C carry).
- Commands/sources:
  - `crates/sbfb-factory/Cargo.toml:20` — `reqwest = { workspace = true,
    features = ["blocking"] }`. Workspace `Cargo.toml:233` — `reqwest = {
    version = "0.12", default-features = false, features = ["json",
    "rustls-tls"] }`. So sbfb-factory has the async `reqwest::Client` + `json`
    AND `blocking`. **No new dependency needed** for the Network arm.
  - `cargo tree -p sbfb-factory -e normal` -> direct `reqwest v0.12.28`,
    `async-stream v0.3.6`, `ollama-rs v0.3.4`. The `reqwest v0.13.3` in
    `Cargo.lock` is pulled transitively by `iroh v0.98.2`
    (`cargo tree -i 'reqwest@0.13.3'`), NOT by sbfb-factory's direct edge.
  - TLS posture: the daemon loopback is plain HTTP
    (`operator_server.rs:165` binds `127.0.0.1`, `daemon_client.rs:50`
    builds `http://{host}:{port}`). The Network arm calls plain HTTP on
    loopback; `rustls-tls` is unused on this path. No TLS handshake risk.
- Finding: **clean**. No CVE on `reqwest`/`async-stream`/`tokio`/`ollama-rs`
  versions in use (kickoff §"OSS prior art — Ollama" confirmed 0 RustSec
  advisory on `ollama-rs`; the 2026 Ollama CVEs target the server, not the
  crate). No major breaking release on the consumed `reqwest 0.12` line.

## S2 Historical Decisions
- Commands: `git log --all --oneline -- crates/sbfb-factory/src/provider_router.rs`
  (`3c9ea1b` only — created Phase C); `... operator_server.rs` (last touched
  `f19ed83` S71 Phase D + `a0337c6` S71 Phase C gate/auth); `... tasks_api.rs`
  (`9942d70` S44 Phase C, last meaningful); `... coordinator/types.rs`
  (`0daff81` S71 Phase B quorum); grep PO-14 across planning + SPRINT_LOG.
- Decisions crossed:
  - **PO-14 (single Done, never WAN token-by-token)** — kickoff §4 D3,
    plan §2 D3, scope cut #12 ("jamais"). Frozen, valid, not reverted.
    Phase D must emit exactly one terminal `Done` for the Network arm.
  - **Gate-before-dispatch (S71 D3, `a0337c6`)** — `operator_server.rs:866-879`
    runs the `SENSITIVE_ACTIONS` gate BEFORE prompt assembly (:885) and
    dispatch (:898). Not reverted; Phase D must preserve it. Confirmed
    structurally intact.
  - **R3 (daemon auth token)** — kickoff §9 R3 / plan §12 R3 flagged "verify
    at preflight". RESOLVED below (S4 / not blocking): `daemon_client.rs`
    already resolves `running.json` + `auth_token` and attaches
    `X-SBFB-Token` + `Host: 127.0.0.1`; the Operator and daemon share the
    same `auth_token` file (`auth.rs:99`).
- Finding: **clean** (no reversion conflict). PO-14 and gate-before-dispatch
  are honored by the plan; R3 is satisfied by existing plumbing.

## S3 Local Patterns And Threat Model
- Threats/contracts checked: T-OPERATOR-CSRF, T-OPERATOR-SPAWN
  (THREAT_MODEL §14), daemon T0 loopback (LOOPBACK §3 line 55).
- HARDENING status / Phase A coverage:
  - `THREAT_MODEL.md:768-777` ("Anticipation NetworkProvider") states the
    `Network` arm is an **outbound client** of `POST /api/v1/tasks/submit`,
    **not a new inbound surface**, stays inside the hardened loopback
    boundary, and that `SENSITIVE_ACTIONS` stays applied before dispatch on
    all providers.
  - `LOOPBACK_ENDPOINTS_TRUST_TIERS.md:107-114` mirrors this: no new inbound
    Operator endpoint is added by the ProviderRouter.
- Finding: **clean (non-blocking)**. No regression of a covered T0-T5 threat.
  The Network arm introduces no inbound surface. The gate-before-dispatch
  invariant is preserved (`operator_server.rs:866-879` before `:898`).
  NOTE: the corrective options for the S4 conflict below (esp. Option B —
  a new daemon read route) WOULD add a daemon inbound endpoint; that must be
  scoped under the existing daemon auth middleware and re-checked against §14
  if chosen.

## S4 Protocol And Wire Invariants
- Wire/security files checked: `nexus-coordinator-rs/src/types.rs`
  (`TaskSubmission`, `TaskStatus`), `nexus-core-rs/src/task.rs`
  (`Task`, `ResultPayload`, `ResultEntry`), `nexus-shell-daemon/src/tasks_api.rs`
  (`TaskResponse`, `get_task`), `nexus-coordinator-rs/src/validator.rs`,
  `nexus-coordinator-rs/src/db.rs` (`tasks` + `task_results` schema).
- VERSION/domain/canonical status:
  - `TASK_FORMAT_VERSION` and `*_ANNOUNCEMENT_VERSION` stay `1`. The Network
    arm needs no new wire field on `TaskSubmission` — it already carries
    `project_id`, `task_type`, `prompt`, `system_prompt`, `model`
    (`types.rs:71-102`). The prompt-assembled-plus-model submission fits the
    existing struct. **No wire bump needed for submission.** (PO-14 / §1.4
    honored on the submit side.)
- Day 0 status: **conflict** (see finding).
- Finding (BLOCKING): **the plan's result-retrieval contract is contradicted
  by the actual daemon endpoints.** Plan §7.1 and kickoff §4 D3 promise: poll
  `GET /api/v1/tasks/{task_id}` (asserted "inchanges") until `completed`, then
  emit "le `result_text` final ... comme un seul `StreamChunk::Done`". The
  real endpoints expose NO result text:
  - `GET /api/v1/tasks/{task_id}` (`tasks_api.rs:107-149`) returns only
    `result_hash` (plus status/model/timestamps). The `tasks.result_hash`
    column for the default single-result path (`redundancy_factor == 1`) is
    `hex::encode(entry.signature)` — an **Ed25519 signature hex**, not the
    text (`validator.rs:71-72`).
  - `get_task_results` (`db.rs:400`) returns `TaskResultRow { ..., sha256, ...
    }`; for single-result tasks `sha256` is also the signature hex
    (`validator.rs:72`). For redundant tasks it holds the raw `result_text`
    (PATTERNS §P53:2742; `validator.rs:86-90`) — but `get_task_results` is
    **not exposed by any HTTP route** (only routes:
    `/api/v1/tasks/submit`, `/api/v1/results/submit`, `/api/v1/tasks`,
    `/api/v1/tasks/{task_id}` — `http.rs:306,307,404,405`).
  - The worker-submitted `ResultEntry.payload.result_text`
    (`task.rs:359`) is consumed only for the output guardrail
    (`http.rs:1501`) and kudos, then dropped. It is never persisted in a
    retrievable place for `redundancy_factor == 1`.
  Downstream impact: the Operator SSE handler renders
  `StreamChunk::Done { result }` as the assistant chat message
  (`operator_server.rs:903-911`). A Network arm that can only obtain a
  signature hash would emit an empty/garbage assistant reply — exactly the
  "fausse promesse" R5 (kickoff §9) warns about. The test
  `network_provider_submit_poll_yields_single_done` (plan §7.3) implicitly
  assumes a retrievable result text in the `Done`.

  This is BLOCKING because the corrective action crosses a **crate boundary
  and the daemon HTTP surface** (a structural decision: change the daemon to
  expose the completed task's text, OR change the Network-arm `Done` contract)
  and cannot be silently chosen by the implementing phase. It maps to
  DESIGN-CONFLICT per the verdict tree (a Day 0 / wire-contract contradiction
  that needs PO arbitration), NOT PLAN-ADAPT (which is reserved for S1a
  APPROACH-NAIVE/LIB-EXISTS).

## Plan Adaptation
N/A — verdict is DESIGN-CONFLICT, not PLAN-ADAPT. See
`sprint72_phase_d_pivot_proposal.md` for options A/B/C.

## Risks And Scope Cuts
- Blocking risks:
  - S4 result-text retrieval gap (above) — `provider_router.rs` `Network`
    arm cannot emit `result_text` in `Done` using only the endpoints the plan
    names as "inchanges". STOP and arbitrate.
- Non-blocking risks / carry-over:
  - R3 (daemon auth) RESOLVED: reuse `daemon_client.rs` discovery +
    `X-SBFB-Token` + `Host: 127.0.0.1`. But `daemon_client.rs:55` returns a
    `reqwest::blocking::Client`; the Network arm needs the ASYNC
    `reqwest::Client` inside `async_stream` — implement the async variant
    (the async client + `json` is available; `auth_token`/`running.json`
    discovery is reusable as-is). Non-blocking implementation note.
  - The `Debug`/progress label during the poll (kickoff §4 D3) maps to
    `StreamChunk::Debug { label, content }` (`llm_bridge.rs:58`) — available,
    no new variant needed.
  - Two `reqwest` major versions in the lock (0.12.28 direct, 0.13.3 iroh
    transitive) — pre-existing, not a Phase D regression; sbfb-factory's
    direct edge is 0.12.28.
- Scope cuts still honored (kickoff §7): no streaming token-by-token from a
  remote worker (#12, PO-14); no cross-machine proof (#9/#10, S75); no
  search/fork/packaging (#2-#8, S73/S74); no `*_VERSION` bump on submission.

## Action
- DESIGN-CONFLICT: stop. Do NOT write the Phase D `Network` arm until the PO
  picks how a completed network task's result reaches the operator chat.
  See `.planning/active/sprint72_phase_d_pivot_proposal.md`.
- D4 backend wiring (`ChatSession +provider`, persist, route-by-provider,
  gate-before-dispatch) and D5 (PATTERNS §P55) have NO conflict and could
  land independently of the Network result-text decision — but the plan binds
  them into one Phase D commit alongside the `Network` arm, so resolve the
  conflict first, then proceed (the chosen option may split Phase D).

## Resolution (PO arbitrage 2026-06-03) — OPTION A → effective verdict EXECUTE

The PO chose **Option A** (`sprint72_phase_d_pivot_proposal.md`): add a daemon
read route that returns the completed result text + persist that text. The
network reply must reach the operator chat this sprint. The DESIGN-CONFLICT is
**resolved**; the code follows the revised, code-grounded spec below (this
section supersedes plan §7 for Phase D, PLAN-ADAPT-style). No Day 0 figée is
touched: PO-14 (single `Done`), gate-before-dispatch, and the pre-launch wire
policy all hold (the new route is a local loopback read, not a wire bump).

### Block 1 — Daemon result-text primitive (`nexus-coordinator-rs` + `nexus-shell-daemon`)

1. **Migration M16** (`db.rs` MIGRATIONS, append-only — mirrors M5/M13
   `ALTER TABLE`): `ALTER TABLE tasks ADD COLUMN result_text TEXT;`. Local DB
   schema, not a wire format (pre-launch policy unaffected); `rusqlite_migration`
   tracks `user_version`, no `schema_version` row bump.
2. **`db.rs::set_task_result`** — add a `result_text: &str` param, persisted in
   the new column on the `UPDATE tasks SET status='completed' ...`. Both
   completion paths feed it: single (`validator.rs:72`) passes
   `entry.payload.result_text`; quorum (`validator.rs:143`) passes `best_hash`
   (which on that path already IS the agreed `result_text`, PATTERNS §P53). So
   `tasks.result_text` always holds the human-readable output after completion.
   Call sites updated: `validator.rs:72,143` (prod) + `db.rs` / `validator.rs` /
   `http.rs` tests.
3. **`db.rs::get_task_result`** (new) → `Option<TaskResultDetail { status,
   result_text: Option<String>, result_hash: Option<String> }>`, one focused
   `SELECT status, result_text, result_hash FROM tasks WHERE task_id=?1`.
   `TaskRecord` is left unchanged (no ripple through insert/list/get).
4. **`tasks_api.rs::get_task_result`** (new handler) — `Some+result_text` → 200
   `{task_id,status,result_text,result_hash}`; `Some` without text → 404 (status
   in message, "404 on pending"); `None` → 404 not found; lock-poison/err → 500.
5. **`http.rs` route** — `.route("/api/v1/tasks/{task_id}/result", get(...))`
   after :405, INSIDE the `auth_required` middleware (same T0 loopback tier,
   read-only). matchit distinguishes it from `/api/v1/tasks/{task_id}`.
6. **THREAT_MODEL §14 + LOOPBACK §3 re-check** (S3 note above): catalogue the
   new `/result` read route as a T0 loopback, read-only, `auth_required`-gated
   endpoint — minimal delta, no new trust tier, no autonomous spawn.

### Block 2 — Network arm + provider wiring (`sbfb-factory`, crate-isolated)

7. **`provider_router.rs` `Network` arm** — replace `network_not_implemented()`:
   resolve the daemon (`SBFB_DAEMON_ENDPOINT`/`SBFB_DAEMON_TOKEN` env override,
   else `DaemonConnection::discover()` for base_url + `auth_token` — R3 reuse);
   async `reqwest::Client`; `POST /api/v1/tasks/submit` with the inline JSON body
   (`{project_id,task_type:"inference",prompt,model}` — **no `nexus-coordinator-rs`
   dependency**, daemon serde-defaults fill the rest; respects Factory crate
   isolation, guardrail #4); extract `task_id` from `response["task"]["task_id"]`;
   `async_stream` poll loop on `tokio::time::interval` (`SBFB_NETWORK_POLL_INTERVAL_MS`,
   default 2000) bounded by a global timeout (`SBFB_NETWORK_TIMEOUT_SECS`, default
   600): `GET /api/v1/tasks/{id}` → on `completed` fetch `GET /api/v1/tasks/{id}/result`
   and emit exactly one `StreamChunk::Done { result }` (PO-14); on
   `rejected`/`timed_out` or global timeout → one `StreamChunk::Error`. A
   `StreamChunk::Debug{label:"network-poll",content:status}` per tick gives the
   front progress without any `Delta` token (PO-14 preserved).
8. **`operator_server.rs` D4** — `ChatSession { ..., provider, project_id }`;
   `ChatSessionRequest`/`ChatSendRequest` already carry `provider`
   (`default_provider`="claude"); add `project_id` (default "operator-chat") to
   the session request; persist `provider` at `/chat/{id}/send` (symmetry with
   `model`, :758); `handle_chat_stream` reads `session.provider` + `project_id`,
   builds `ExecutionTarget::from_provider(&provider,&model,&project_id).run(prompt,root)`
   in place of the direct `spawn_claude_stream` (:898). The `SENSITIVE_ACTIONS`
   gate at :866-879 stays BEFORE dispatch — unchanged, provider-independent.
9. **`docs/rust/PATTERNS.md` §P55** (D5) — 3 orthogonal axes: `ExecutionTarget`
   (chat routing) vs `Provider` (prompt-adapt, `process.rs`) vs `LlmBackend`
   (worker quorum runtime).

### Tests (≈ +8 Rust)
- `db.rs`: `set_task_result_persists_result_text`, `get_task_result_*`
  (text-on-completed / none-on-pending / none-on-missing).
- `validator.rs`: accepted single-path result persists retrievable text.
- `provider_router.rs`: `network_provider_submit_poll_yields_single_done`
  (mock daemon submit→poll→/result, exactly one `Done`, zero `Delta`),
  `network_provider_poll_timeout` (never-completes mock → one `Error`).
- `tests/operator_server.rs`: `chat_session_persists_provider` (send override
  routes to Ollama), `chat_stream_routes_by_session_provider` (session provider
  routes Ollama vs Claude), `sensitive_action_gated_regardless_of_provider`.

### Commit (single, atomic)
One Phase D commit. The two blocks are ONE end-to-end feature — the factory
network arm *consumes* the daemon `/result` route, so the route with no consumer
is dead code and the consumer with no route is broken; splitting would ship a
non-atomic half. Matches the Phase C precedent (`3c9ea1b` bundled two blocks).
- `feat(factory): Sprint 72 Phase D — NetworkProvider submit-poll + result-text primitive + provider routing`
Carries the full review + Codex gate (review-deep skill fallback, Codex zero-exemption).
