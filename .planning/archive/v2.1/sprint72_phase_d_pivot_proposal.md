# Sprint 72 Phase D — Pivot Proposal (DESIGN-CONFLICT)

Date: 2026-06-03
HEAD: `3c9ea1b`
Trigger: S4 wire/contract conflict found in preflight
(`sprint72_phase_d_preflight.md`).

## The conflict (factual)

Plan §7.1 and kickoff §4 D3 specify the `Network` arm of `ExecutionTarget`:

> soumet une `TaskSubmission` ... via `POST /api/v1/tasks/submit`, recupere
> le `task_id`, puis poll `GET /api/v1/tasks/{task_id}` ... jusqu'a
> `completed` ou `rejected`. A `completed`, le **`result_text` final est emis
> comme un seul** `StreamChunk::Done`.

Both endpoints are asserted **"inchanges"** (consume the existing S71
contract, no wire bump).

The actual daemon code exposes **no result text** for the default
single-result path (`redundancy_factor == 1`, what the Operator uses):

| Source path | What it returns | Cite |
|---|---|---|
| `GET /api/v1/tasks/{task_id}` | `result_hash` only (+ status/model/ts) | `tasks_api.rs:107-149` |
| `tasks.result_hash` (single path) | `hex::encode(entry.signature)` = Ed25519 signature hex, NOT text | `validator.rs:71-72` |
| `task_results.sha256` (single path) | same signature hex; for redundant tasks it is raw `result_text` (PATTERNS §P53) | `validator.rs:72,86-90` |
| `get_task_results` (raw text on redundant path) | NOT exposed by any HTTP route | routes: `http.rs:306,307,404,405` |
| `ResultEntry.payload.result_text` (worker submission) | consumed for guardrail + kudos, then dropped | `http.rs:1501`, `task.rs:359` |

Downstream: the Operator SSE renders `Done { result }` as the assistant
chat message (`operator_server.rs:903-911`). A `Done` carrying only a
signature hash produces a broken/empty assistant reply (R5 "fausse
promesse", kickoff §9).

The submission side is fine: `TaskSubmission` (`types.rs:71-102`) already
carries `project_id`/`task_type`/`prompt`/`system_prompt`/`model` — no wire
field is missing to submit. The gap is purely **result retrieval**.

R3 (auth) is NOT the conflict: `daemon_client.rs` already resolves the
shared `auth_token` + `running.json` and attaches `X-SBFB-Token` +
`Host: 127.0.0.1`. The Operator can authenticate the submit and poll calls
today.

## Guardrails checked (against the procedure)

1. Evidence-backed, not opinion: every claim cites a `file:line`.
2. Not Day 0 re-litigation: PO-14 (single `Done`) and gate-before-dispatch
   are preserved by all options below; the conflict is upstream of them.
3. Pre-launch wire policy: no option requires a `*_VERSION` bump. Adding a
   loopback read route is a new local endpoint, not a wire-format change to
   `Task`/`TaskSubmission`/announcements.
4. Crate-boundary respected: `sbfb-factory` consumes daemon HTTP; it does
   not pull `nexus-worker-core`/iroh (Factory isolation, CLAUDE.md).

## Options

### Option A — Add a daemon read route that returns the completed result text (RECOMMENDED)

Add `GET /api/v1/tasks/{task_id}/result` to the daemon (behind the same
`auth_required` middleware, T0 loopback). It returns the accepted
`result_text` for a `completed` task. Persist the text so the route can
read it:

- The single-result path (`validator.rs:71-72`) currently stores the
  signature hex in `tasks.result_hash` / `task_results.sha256`. Add a
  `result_text` column (or a dedicated `task_outputs` row) written on
  `ValidationOutcome::Accepted`, before the `result_text` is dropped at
  `http.rs:1501`. For redundant tasks the agreed text already exists in
  `task_results.sha256` (PATTERNS §P53) — surface it the same way.
- The `Network` arm polls `GET /api/v1/tasks/{id}` for status, then on
  `completed` fetches `GET /api/v1/tasks/{id}/result` and emits its text as
  the single `Done` (PO-14 unchanged). On `rejected`/timeout -> `Error`.

Cost: touches `nexus-shell-daemon/src/http.rs` (route + handler),
`nexus-shell-daemon/src/tasks_api.rs`, `nexus-coordinator-rs/src/db.rs`
(schema + persist), `nexus-coordinator-rs/src/validator.rs` (write text on
accept). Crosses crate boundary into the daemon/coordinator (not just
`sbfb-factory`). Re-check THREAT_MODEL §14 for the new inbound route (T0,
same auth — minimal delta). New tests: daemon route returns text on
completed, 404 on pending. This is the durable, product-correct answer
(it is the missing primitive the whole network-execution vision needs; S75
GPU-share reuses it). No wire bump (loopback-only read route, pre-launch
policy §1.4).

Scope impact: Phase D grows by a daemon-side sub-task. Recommend splitting
Phase D into D-backend (the daemon result route + persist) and the
`Network` arm, or absorbing the route as a small first block of Phase D.

### Option B — Network arm returns status + hash only, no result text (minimal, honest stub)

The `Network` arm submits, polls, and on `completed` emits a single `Done`
whose `result` is a **status/provenance summary** (e.g. "Task <id> completed
on the network; result signature <hash>. Open the task to inspect the
output.") rather than the model text. No daemon change.

Cost: `sbfb-factory` only. Honors PO-14 and the single-`Done` contract, and
is truthful (it never fakes text it cannot fetch). But it does NOT deliver
the plan's promise that the network reply appears in the chat — the assistant
message is a receipt, not an answer. UX (Phase E) must frame the network
intention as "submit a verified task" not "chat over the network".

Scope impact: Phase D stays inside `sbfb-factory`. The result-text route
becomes an explicit S73 carry. Lowest risk, lowest value; risks shipping a
feature that looks broken to a chat user (R5).

### Option C — Defer the `Network` arm entirely to S73; land only D4+D5 this phase

Keep the `network_not_implemented()` stub (`provider_router.rs:241-249`).
Phase D ships ONLY the D4 backend wiring (`ChatSession +provider`, persist,
route-by-provider for Claude/Ollama, gate-before-dispatch) and D5
(PATTERNS §P55). Selecting "network" in the UI yields the existing clear
"not implemented yet (Phase D)" diagnostic until S73 adds the daemon result
route + arm together with the search-network work (S73 already owns the
network-result surface area).

Cost: smallest, cleanest. `chat_stream_routes_by_session_provider`,
`chat_session_persists_provider`, `sensitive_action_gated_regardless_of_
provider` still land and prove the routing spine. Drops
`network_provider_submit_poll_yields_single_done` /
`network_provider_poll_timeout` from S72 (moved to S73). Honors the
"quick win strict" framing (kickoff §1.2) and the rule that the network
result primitive belongs with S73 network work. Phase E network "in
progress" states become S73 too (or a documented placeholder).

## Default recommendation

**Option A** if the PO wants the network reply to actually appear in the
operator chat this sprint (it is the missing primitive, durable, reused by
S75). **Option C** if "quick win strict" wins and the network result text is
acceptable to land coupled with S73's network surface. Option B only if a
truthful receipt-style network reply is explicitly desired now.

Recommend **A with a Phase D split** (daemon result route as a first block,
then the arm), or **C** — both are consistent; B is the weakest.

## User decision needed before code starts

1. Does the completed network task's **model output text** need to reach the
   operator chat in S72 (-> A), or is a receipt/hash acceptable (-> B), or is
   the whole `Network` arm deferred to S73 (-> C)?
2. If A: approve touching the daemon/coordinator crates (new
   `GET /api/v1/tasks/{id}/result` route + a `result_text` persistence
   column) and splitting Phase D into a daemon block + the arm.
3. If C: confirm dropping the two `network_provider_*` tests from S72 and
   moving them (and the Phase E network "in progress" UX) to S73.

No Phase D `Network` arm code is written until this is answered. D4 backend
wiring + D5 carry no conflict and can proceed once the option is chosen
(the option may re-shape what Phase D commits).
