# Sprint 71 Phase D Preflight

Date: 2026-05-30
HEAD: `a0337c6`
Verdict: **PLAN-ADAPT**

## Evidence Rules
- Claim policy: every claim below cites a path, command output, URL/date, or
  explicit assumption.
- Local sources read: `prompts/agent/preflight.md`, `.planning/active/sprint71_plan.md` (S8),
  `.planning/active/sprint71_kickoff.md` (D1-D8, S11, R8), `.planning/active/sprint70_audit_findings.md`,
  `crates/sbfb-factory/src/terminal.rs`, `crates/sbfb-factory/src/process.rs`,
  `crates/sbfb-factory/src/sprint_history.rs`, `crates/sbfb-factory/tests/operator_server.rs`,
  `crates/sbfb-factory/tests/process_cli.rs` (head), `crates/sbfb-factory/src/operator_server.rs` (router + handlers),
  `crates/sbfb-factory/src/llm_bridge.rs` (tests), `crates/sbfb-factory/Cargo.toml`,
  `crates/sbfb-factory/src/main.rs` (mod decls), `docs/claude/README.md` (S4.5.x).
- Commands run: `git stash list`; `git show 2f9238d --no-patch --format=%B`;
  `git log --oneline 2ec72e8..HEAD -- <factory src>`; `git log --all --oneline -- terminal.rs sprint_history.rs`;
  `cargo check -p sbfb-factory --tests` (Finished, 0 error);
  `cargo nextest list -p sbfb-factory` (112 tests today); grep visibility scans.
- External: asciinema asciicast v2 spec (docs.asciinema.org, retrieved 2026-05-30);
  RustSec advisory DB (rustsec.org, retrieved 2026-05-30 — no portable-pty advisory).

## Scope
- Plan source: `.planning/active/sprint71_plan.md §8` (Phase D — reconciliation off-sprint G5/G6).
- Target files (plan §D.2):
  - `.planning/active/sprint71_offsprint_retro_review.md` (NEW, G5 retro-review 11 dims).
  - `.planning/active/sprint71_offsprint_codex_review.md` (NEW, G5 raw `codex exec -o`).
  - `crates/sbfb-factory/tests/terminal.rs` (NEW per plan) — session log, extension.
  - `crates/sbfb-factory/tests/sprint_history.rs` (NEW per plan) — parsing, endpoint.
  - `crates/sbfb-factory/tests/operator_server.rs` (extend) — chat/sprint-history/diff.
  - `crates/sbfb-factory/src/process.rs` (inline unit tests).
- Deps/APIs/specs: NONE added. Exercises existing `portable-pty 0.9.0`, `serde_json`,
  `tempfile` (dev), `reqwest` (dev path via spawned binary). Asciicast v2 wire of the `.cast` log.
- Security/protocol surfaces: NONE new. Tests touch already-gated Phase C surfaces
  (auth/token/Host/CORS/SSE gate) — must reuse the existing harness, not weaken it.
- Tests expected (plan §D.3): terminal::session_log_roundtrip,
  terminal::list_sessions_filters_correct_extension, sprint_history::parses_active_and_archive,
  sprint_history::diff_endpoint_returns_inline_code, operator_server::chat_session_lifecycle,
  process::resolve_kind_aliases, process::repo_root_resolves.

## S1a OSS Prior Art
- Domain: (1) PTY session capture / terminal-recording test patterns; (2) asciicast v2
  round-trip validation; (3) coverage harness for a binary-only Rust daemon crate.
- Sources:
  - asciicast v2 spec — https://docs.asciinema.org/manual/asciicast/v2/ (2026-05-30).
    Header = single JSON object line `{"version":2,"width":..,"height":..,"timestamp":..,"env":{..}}`;
    events = newline-delimited 3-element arrays `[time, "o"|"i"|"r", data]`. Suggested
    extension `.cast`, media type `application/x-asciicast`.
  - wezterm/portable-pty (the dep itself) — PTY tests in mature projects spawn a cheap,
    deterministic child (e.g. `echo`/`true`), never the real interactive program; they assert
    on captured bytes, not on a live TTY. Mirrored already by this crate's own llm_bridge tests
    (`crates/sbfb-factory/src/llm_bridge.rs:340-390`: spawn a non-existent or `sleep`/`waitfor`
    process, assert on the stream, never a real `claude`).
- Finding: **APPROACH-ALIGNED** for the asciicast assertion and the "never spawn the real
  agent" rule. `terminal.rs:30-46` emits exactly the v2 header+event shape, so a structural
  round-trip test is correct and needs no new dependency (parse with `serde_json`).
- Impact: none on the *intent*. The OSS evidence reinforces that the terminal test must hit
  the file-system functions (`session_log_path`/`list_sessions`), NOT drive a real PTY spawn
  (`handle_terminal_ws` launches `claude.cmd`/`claude` over a PTY — untestable hermetically).

## S1b Dependencies, CVEs, Release Notes
- Scanned: `portable-pty 0.9.0` (the off-sprint PTY/spawn dep behind `terminal.rs`),
  `serde_json`, `async-stream`, `futures`, `tempfile`. `Cargo.toml` confirms no Phase D
  addition is required (`crates/sbfb-factory/Cargo.toml:13-33`; dev-deps = `tempfile` only).
- Commands/sources: `grep portable-pty Cargo.toml` → `portable-pty = "0.9.0"`;
  RustSec per-package page `rustsec.org/packages/portable-pty.html` → HTTP 404 (RustSec only
  emits a package page when an advisory exists); RustSec advisory index → no portable-pty entry
  (retrieved 2026-05-30). The 3 off-sprint deps (`portable-pty`, `async-stream`, `futures`)
  were already passed to S1b in Phase B (G13, `sprint71_plan.md §B.3 item 5`).
- Finding: **clean** (non-blocking). No new dep; no CVE on the PTY/spawn surface; Phase D is a
  test-only delta. G13/S1b trace: portable-pty 0.9.0 carries no published RustSec advisory as of
  2026-05-30; it is a process-spawn surface and is NOT invoked by the Phase D tests (the tests use
  the file-system path, see PLAN-ADAPT below), so no new exposure is introduced.

## S2 Historical Decisions
- Commands: `git log --all --oneline -- crates/sbfb-factory/src/terminal.rs` →
  `0aa06db` resume sessions, `864b005` "persist terminal sessions as asciicast + session list",
  `c3f4813` embedded terminal. `git log --all --oneline -- sprint_history.rs` →
  `e73c9fb`, `5f2cc9a` (diff endpoint), `a8a273f` (sprint-history endpoint).
  `git stash list` → only `pre-reset: gossip bootstrap` and an unrelated WIP remain
  (the `.cast`->`.log` "WIP terminal plaintext-logging" stash is GONE).
  `git show 2f9238d --no-patch --format=%B` → Phase A body header says "tranche le WIP
  terminal (G1)" but the `## Fichiers` table lists only `dispatch_loop.rs` and `runtime.rs`
  (terminal.rs NOT edited).
- Decisions crossed:
  - **D7 (WIP terminal, G1)** — RESOLVED at Phase A `2f9238d` by **dropping the stash and
    keeping HEAD asciicast `.cast`**. Reverse-commit check: terminal.rs is unchanged since
    `0aa06db` (pre-S71); the plaintext refactor was abandoned, not re-applied. Confirmed
    reversion of the abandoned plaintext branch — non-blocking. CONSEQUENCE FOR PHASE D:
    the active extension is `.cast` everywhere (`terminal.rs:27` write, `terminal.rs:213`
    `list_sessions` filter `Some("cast")`, `operator_server.rs:952` serve `{name}.cast`).
    The plan test `list_sessions_filters_correct_extension` MUST target `.cast` (the plan's
    own §A.3 left this conditional on D7; D7 chose `.cast`).
  - **D8 (provider vs backend, dette)** — RESOLVED at Phase B `0daff81`. `process.rs:24-34`
    now carries the documented distinction (prompt-provider vs runtime `LlmBackend`), NOT
    unified. The `process::resolve_kind_aliases`/`providers` test asserts the *current*
    `PROVIDERS = ["claude","codex","gpt","local","human"]` (`process.rs:34`) and aliases
    `review->phase-review`, `auditor->phase-auditor`, `audit->audit-gate` (`process.rs:18-22`).
  - asciicast format choice — `864b005` is the deliberate "persist as asciicast" decision;
    no documented decision is being reverted by Phase D (PATTERNS "plaintext" hits are
    kudos/key-encryption, unrelated).
- Finding: **clean** (non-blocking confirmed reversions). Phase D reverts nothing; it must
  *encode* the D7/D8 resolutions into the tests (extension `.cast`, providers/aliases current).

## S3 Local Patterns And Threat Model
- Threats/contracts checked: the Phase D tests touch the Phase C-gated Operator HTTP surface
  (G2 SSE gate, G7 token+Host+CORS). T-mapping: the Operator server is a loopback-only,
  token-authenticated control plane (the daemon loopback-hardening threat class, CLAUDE.md
  "Securite"). Phase D must NOT regress these gates.
- HARDENING status: N/A — no HARDENING_ROADMAP pre-requirement is attached to a test-coverage
  phase. The existing `tests/operator_server.rs` already proves the gates
  (`server_rejects_missing_token`, `server_rejects_foreign_host`, `cors_restricts_origin`,
  `sse_gates_sensitive_action`, `sse_allows_nonsensitive`, `chat_stream_uses_opus_model`).
- Regression guard: the existing harness `TestServer` (`tests/operator_server.rs:24-101`) sets
  `SBFB_AUTH_TOKEN`, `SBFB_HOME` (tempdir sandbox), `SBFB_CLAUDE_BIN=...nonexistent` so no test
  ever launches a real `bypassPermissions` agent. New endpoint tests MUST reuse this harness
  verbatim (same env, same `--port 0`/`READY ` handshake, same `x-sbfb-token` header) — building
  a second, unauthenticated client would itself be a regression.
- Finding: **clean** (non-blocking). One standing constraint to honor, not a finding: do not
  add a non-loopback or token-less HTTP path; extend, do not fork, the harness.

## S4 Protocol And Wire Invariants
- Wire/security files checked: `terminal.rs`, `sprint_history.rs`, `process.rs` —
  `grep "_VERSION|canonical|DOMAIN_|schema|serde(default)|sign"` returns only the
  `resolve_kind` local var `canonical` (prompt-kind canonicalization), `dunce::canonicalize`
  path calls, and the `design_conflict` field name. No `*_VERSION`, no signing domain, no
  canonical JCS bytes, no protocol schema, no `serde(default)` wire-drift.
- VERSION/domain/canonical status: untouched. The only serialized structures are Operator
  JSON response bodies (`SprintHistoryResult`, `CommitDiffResult`, etc.), which are local UI
  contracts, not propagated wire formats; the asciicast `.cast` is a local on-disk log, not a
  network format.
- Day 0 status: **preserved**. D1-D8 unaffected; D7 (`.cast`) and D8 (provider/backend split)
  are *encoded* by the tests, not changed.
- Finding: **clean**. Phase D changes zero wire format / zero `*_VERSION` — consistent with the
  pre-launch protocol policy and the plan's own `test(factory)` (non-feature) framing.

## Plan Adaptation
PLAN-ADAPT driver is **structural**, surfaced by S1a (binary-only crate test architecture) and
S2 (existing Phase C coverage). It does NOT change Phase D intent (G5 artefacts + G6 coverage);
it corrects the *vehicle* and *target* of three planned tests so they compile and do not
duplicate existing coverage.

- Original plan (`§8 D.2`, `§8 D.3`):
  - `crates/sbfb-factory/tests/terminal.rs` (NEW) and `tests/sprint_history.rs` (NEW) as
    integration files; `operator_server.rs` extended for "chat/sprint-history/diff".
  - tests `terminal::session_log_roundtrip`, `terminal::list_sessions_filters_correct_extension`,
    `sprint_history::parses_active_and_archive`, `sprint_history::diff_endpoint_returns_inline_code`,
    `operator_server::chat_session_lifecycle`.

- Evidence requiring adaptation:
  1. **Binary-only crate** — `crates/sbfb-factory/Cargo.toml` has `[package] name=sbfb-factory`
     and NO `[lib]`; `main.rs:5-21` declares modules (`mod terminal; mod process;
     pub mod sprint_history;`). `cargo nextest list` reports tests under
     `sbfb-factory::bin/sbfb-factory`. Integration files in `tests/` therefore have NO
     `use sbfb_factory::...` library to import; they can only spawn `CARGO_BIN_EXE_sbfb-factory`
     (the binary) — exactly what `tests/process_cli.rs` and `tests/operator_server.rs` do.
     => A `tests/terminal.rs` calling `terminal::session_log_path()` directly will NOT compile.
  2. **Private functions** — `session_log_path` (terminal.rs:12), `resolve_kind`
     (process.rs:62), `repo_root` (process.rs:49) are private. Reaching them requires an
     INLINE `#[cfg(test)] mod tests { use super::* }` (the pattern of every other module:
     auth, gates, llm_bridge, diff, pipeline, provenance). `process.rs` exposes only
     `repo_root_pub`/`providers_list` publicly.
  3. **`sprint_history` private parsers** — `parse_scope_cuts`, `parse_carries`,
     `parse_verification`, `extract_section`, `parse_unified_diff` are private; the only `pub`
     entry points are `all_sprints_data(root)`, `sprint_history_data(root)`,
     `sprint_history_for(root,sprint)`, `commit_diff_data(sha)`. Granular parser tests must be
     INLINE; end-to-end parse/endpoint tests go through the HTTP harness.
  4. **chat lifecycle already covered** — `tests/operator_server.rs` already has
     `operator_chat_session_starts_from_context_pack`, `operator_chat_message_endpoint`,
     `operator_chat_log_endpoint`, `operator_chat_logs_messages_and_actions`, plus the SSE/auth
     suite. The GENUINELY uncovered endpoints are `/api/sprint-history`,
     `/api/sprint-history/all`, `/api/sprint-history/{n}`, `/api/sprint-history/diff/{sha}`,
     and `/api/terminal/sessions` (router `operator_server.rs:128-139`). `chat_session_lifecycle`
     as written would largely duplicate existing tests.
  5. **diff endpoint needs a real SHA** — `commit_diff_data` shells `git diff -U3 {sha}^..{sha}`
     against the process cwd repo (`sprint_history.rs:938-952`); `handle_commit_diff` rejects
     a sha containing `..`/`/` and `<4` chars (`operator_server.rs:993-1000`). The diff test
     must pass a real short SHA (e.g. an off-sprint commit such as `864b005`, or `HEAD`), not a
     synthetic fixture, OR build a throwaway git repo in a tempdir for `commit_diff_data`.

- Corrected approach (concrete):
  - **terminal coverage -> INLINE `#[cfg(test)] mod tests` in `crates/sbfb-factory/src/terminal.rs`**
    (NOT a `tests/terminal.rs` file). `session_log_roundtrip`: build a tempdir root, call the
    log writer path against a `.cast` file, then read it back and assert line 1 parses as a JSON
    object with `"version":2` and a subsequent line parses as a 3-element array whose 2nd element
    is `"o"` (asciicast v2 per docs.asciinema.org). NOTE: `session_log_path` derives its path
    from `process::context_data` and only builds the path; the header/event writers
    (`write_asciicast_header`/`write_asciicast_event`) are the unit under test for the round-trip
    — test those directly (they are private, reachable inline). `list_sessions_filters_correct_extension`:
    seed a tempdir `.planning/terminal/` with one `*.cast` and one `*.log`/`*.txt`, call the
    public `list_sessions(root)`, assert only the `.cast` entry is returned (extension `.cast`,
    per D7).
  - **process coverage -> INLINE `#[cfg(test)] mod tests` in `crates/sbfb-factory/src/process.rs`**
    (plan §D.2 already lists this under the src column — correct). `resolve_kind_aliases`: assert
    `resolve_kind("review")==Some("phase-review")`, `"auditor"->"phase-auditor"`,
    `"audit"->"audit-gate")`, a passthrough canonical (`"preflight"->"preflight"`), and `None`
    for garbage; assert `providers_list()` equals the 5-tuple. `repo_root_resolves`: call
    `repo_root_pub()` and assert it ends in a real dir containing `.git` (or equals
    `git rev-parse --show-toplevel`).
  - **sprint_history coverage -> SPLIT**: (a) INLINE unit tests in `src/sprint_history.rs` for
    pure parsers using string fixtures — `parse_unified_diff` (assert add/del/ctx line kinds),
    `extract_section`, the verdict extractor; (b) HTTP/endpoint tests in the EXISTING
    `tests/operator_server.rs` via the `TestServer` harness for `/api/sprint-history` (active
    sprint present in repo -> 200 + `sprint`/`phases` fields) and
    `/api/sprint-history/diff/{real_sha}` (200 + `files[].hunks[].lines[]` with `kind:"add"`).
    Drop the standalone `tests/sprint_history.rs` file in favor of these two homes (matches the
    crate's established test architecture).
  - **operator_server extension** -> add the sprint-history + diff + terminal-sessions endpoint
    tests to the EXISTING `tests/operator_server.rs` (reusing `TestServer`); SKIP a redundant
    `chat_session_lifecycle` (already covered) or, if kept, make it assert something new (e.g.
    the `context_pack.chat_history_authoritative==false` invariant across a full
    session->send->stream->log sequence) rather than re-proving the message/log endpoints.

- File/test delta vs original plan:
  | Plan item | Adapted home | Reason |
  |-----------|--------------|--------|
  | `tests/terminal.rs` (NEW) | INLINE `src/terminal.rs #[cfg(test)]` | binary-only crate, private fns |
  | `tests/sprint_history.rs` (NEW) | INLINE `src/sprint_history.rs` (parsers) + `tests/operator_server.rs` (endpoints) | private parsers + existing HTTP harness |
  | `operator_server::chat_session_lifecycle` | replaced by sprint-history/diff/terminal-sessions endpoint tests | chat lifecycle already covered Phase C |
  | `process::resolve_kind_aliases` / `repo_root_resolves` | INLINE `src/process.rs #[cfg(test)]` (as planned) | no change |
  | `terminal::list_sessions_filters_correct_extension` | target `.cast` (not `.log`) | D7 chose asciicast |
  | `diff_endpoint_returns_inline_code` | use a real short SHA / tempdir git repo | endpoint shells real `git diff` |

## Risks And Scope Cuts
- Blocking risks: none. Verdict is PLAN-ADAPT (S1a structural), not DESIGN-CONFLICT
  (S1b/S2/S3/S4 all clean). No Day 0 touched; no wire format touched.
- Non-blocking risks / carry-over:
  - R8 (plan §13, kickoff §11): Phase D is the largest phase and the scindage trigger. This
    preflight does NOT pre-decide the split — per the guard, the scindage decision is taken at
    the moment of overflow, not at preflight. See "Feasibility" below.
  - Two Codex artefacts are in play for Phase D (process nuance): the off-sprint retro-Codex
    `sprint71_offsprint_codex_review.md` (G5, raw `codex exec -o`, verifies the ~14-commit
    block) AND the phase's own `sprint71_phase_D_codex_review.md` (verifies Phase D's coverage
    deliverables). Lightcheck Check 7 / audit-commit enforce the `sprint{N}_phase_{X}_codex_review.md`
    name for the `test(factory)` phase commit (`process.rs:610-630`); the off-sprint file name
    is non-standard and satisfies G5, not the phase-commit gate. Phase D must produce BOTH; both
    must be raw `codex exec -o` output (docs/claude/README §4.5.2 authenticity rule), never
    rewritten by Claude.
  - portable-pty 0.9.0 carries no RustSec advisory today (clean), but it is a process-spawn
    surface; keep it under the standing carry watch (G13 already logged it Phase B).
- Scope cuts still honored (kickoff §8): Phase D touches no file under scope cuts #1-#16. It
  adds tests + planning artefacts only; ProviderRouter (#1), network search (#3-#6),
  atelier/fork (#7-#9), GPU share (#10), redundancy>1 cross-machine (#11), sharding (#12),
  logprobs (#13) are all untouched.

## Action
- PLAN-ADAPT: proceed with the corrected test architecture above (inline `#[cfg(test)]` for
  terminal/process/sprint_history parsers; extend the existing `tests/operator_server.rs`
  `TestServer` harness for the sprint-history/diff/terminal-sessions endpoints; target the
  `.cast` extension per D7; use a real SHA for the diff endpoint). The Phase D commit body must
  cite this file and state: "Plan proposed `tests/terminal.rs`+`tests/sprint_history.rs` as
  integration files / `chat_session_lifecycle`; preflight S1a identified the binary-only crate
  test architecture + private-fn visibility + existing Phase C chat coverage; adapted to inline
  `#[cfg(test)]` modules + HTTP-harness endpoint tests, retargeted to the genuinely uncovered
  sprint-history/diff/terminal-sessions endpoints and the `.cast` extension." The plan file
  stays unchanged (snapshot); the deviation is traced here and in the commit body only.

## Feasibility: mono-phase vs R8 split
- The G6 coverage is **structurally lighter than the LOC suggests**. `sprint_history.rs` is 1048
  lines but is dominated by serde structs and private parsers; the testable public surface is 4
  functions, and the highest-value pure-parser test (`parse_unified_diff`) is a single string
  fixture. terminal coverage is 2 small file-system/asciicast tests. process coverage is 2
  inline tests. The endpoint tests reuse a harness that already exists and already boots in ~CI
  time. This is a few hundred lines of test code, not a 5500-line re-implementation.
- The G5 retro-review (11 dims, §4.5) + retro-Codex (raw exec) are bounded document tasks over
  a diff already mapped in `sprint70_audit_findings.md` (the audit-absorb already enumerated the
  off-sprint findings B-1..G13) — the retro-review largely *cites* that absorbed audit rather
  than re-discovering it.
- Therefore mono-phase is plausible. BUT per the guard and kickoff §11/R8, the scindage decision
  is NOT taken here. If, during execution, the retro-audit + coverage exceed the session budget,
  the documented fallback applies (retro-review + retro-Codex done + priority tests
  terminal/process landed; exhaustive sprint_history endpoint tests + full retro-audit to
  S71-bis/S72 on PO arbitration §11 option (a)). Decision at overflow, not at preflight.
