**Findings**

P1 GAP: no shell injection, but real Git option injection remains. `sprint_history.rs` uses direct argv, so not shell-based (`crates/sbfb-factory/src/sprint_history.rs:678`), but `handle_commit_diff` only rejects short strings, `..`, and `/` (`crates/sbfb-factory/src/operator_server.rs:993`). Raw `sha` then reaches `git log` / `git diff` (`crates/sbfb-factory/src/sprint_history.rs:938`, `:944`). I live-probed `/api/sprint-history/diff/--output=<temp>`: HTTP 404, but Git still created the temp file. Same class exists on `/api/audit/{rev}`: raw `rev` reaches `git log` (`crates/sbfb-factory/src/operator_server.rs:212`, `crates/sbfb-factory/src/process.rs:548`), and `/api/audit/--output=<temp>` returned 2xx while writing a 9142-byte temp file. Temp files were removed.

P2 hardening gap: terminal session content joins unsanitized `{name}.cast` (`crates/sbfb-factory/src/operator_server.rs:944`). Validate against listed session basenames or canonicalize and prefix-check `.planning/terminal`.

**Per-Deliverable Verdicts**

1. **B-1: CONFIRME.** Dispatcher writes `task:{id}` at `crates/nexus-shell-daemon/src/dispatch_loop.rs:41`; worker scans `b"task:"` and strips `task:` at `crates/nexus-worker-core/src/engine/runtime.rs:847` and `:859`. I found no production `tasks/` doc writer; remaining `tasks/` hits are HTTP routes/comments.

2. **B-2: CONFIRME.** `Task::verifiable` is signed/defaulted in `crates/nexus-core-rs/src/task.rs:145`. Worker uses deterministic params for verifiable tasks at `crates/nexus-worker-core/src/engine/runtime.rs:1243`; seed derives from task id at `:1265`. Ollama forwards temperature/seed at `crates/nexus-worker-core/src/llm/ollama.rs:180`. Validator still rejects divergence at `crates/nexus-coordinator-rs/src/validator.rs:130` and `:166`. Note: the validator field is still named `sha256`, but comments state it stores raw `result_text`, not a hash (`validator.rs:86`).

3. **G2: CONFIRME.** SSE extracts the last user message at `crates/sbfb-factory/src/operator_server.rs:816`, checks `SENSITIVE_ACTIONS` before prompt assembly/spawn at `:845`, and returns `requires_gate` at `:856`. Spawn only happens later at `:878`.

4. **G7: CONFIRME for route auth.** All declared operator routes are built before the auth layer (`crates/sbfb-factory/src/operator_server.rs:112`, `:152`). Middleware enforces loopback `Host`, loopback/absent `Origin`, and `x-sbfb-token` at `crates/sbfb-factory/src/auth.rs:232`, `:241`, `:252`. CORS is restricted with `AllowOrigin::predicate`, GET/POST, and explicit headers at `operator_server.rs:99`. Only CORS preflight/404 remain non-data-bearing paths.

5. **G9: CONFIRME.** Default model is `claude-opus-4-8[1m]` at `crates/sbfb-factory/src/operator_server.rs:271`; sessions initialize with it at `:609`; stream falls back to it at `:873`. `sonnet` remains only in comments/tests.

6. **G12: PARTIEL.** There is a real idle timeout: default 120s at `crates/sbfb-factory/src/llm_bridge.rs:12`, enforced with `tokio::time::timeout` and kill/reap at `:198`. Missing-Claude diagnostic is real at `:161`. But I do not see a pre-spawn resolver/check; it is a spawn-error diagnostic, not a pre-spawn check. Also this is idle-only, not a total runtime deadline.

7. **G6: PARTIEL.** The +13 tests exist and pass: process 3 (`process.rs:841`), sprint_history 3 (`sprint_history.rs:1059`), terminal 2 (`terminal.rs:318`), operator endpoints 5 (`tests/operator_server.rs:662`). Pure parser/process/terminal assertions are meaningful. Endpoint tests are shallow: diff only checks `title`/`files` array (`tests/operator_server.rs:684`), terminal only checks arrays (`:708`), and invalid SHA only covers too-short input (`:699`), missing the `--output` class above.

8. **New issue: GAP.** The missed P1 is Git option injection/arbitrary file write through rev/sha parameters. Fix by accepting only resolved commit IDs, e.g. strict hex SHA or `HEAD` resolved server-side with `git rev-parse --verify`, then use the resolved hex value only. Do not pass raw user revs after Git options.

**Verification Run**

Passed:
`cargo test -p sbfb-factory --bin sbfb-factory --locked`
`cargo test -p sbfb-factory --test operator_server --locked`
`cargo test -p nexus-shell-daemon dispatch_loop_writes_to_doc --locked`
`cargo test -p nexus-shell-daemon dispatched_task_is_claimed_and_executed_by_worker_engine --locked`
`cargo test -p nexus-worker-core verifiable_task_uses_greedy_seed --locked`
`cargo test -p nexus-worker-core deterministic_options_wire_temperature_and_seed --locked`
`cargo test -p nexus-coordinator-rs quorum_ --locked`
`cargo test -p nexus-core-rs verifiable --locked`

**Overall Verdict**

Not fully reconciled. B-1, B-2, G2, G7, and G9 are closed. G12 is only partially closed. G6 improves coverage but leaves shallow endpoint/security assertions. A real P1 remains in the Operator Git rev/sha handling, so S71 A-D does not fully close the off-sprint debt.