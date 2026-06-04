# Sprint 73 Phase B — Review

Date: 2026-06-04
HEAD (pre-commit): `5361fd8` (Phase A). Working tree = Phase B (uncommitted).
Preflight: `sprint73_phase_b_preflight.md` (verdict EXECUTE).
Review method: adversarial multi-agent workflow (6 dimensions × adversarial
verification, 17 agents) + main-thread spot-verification + full suites.

## Verdict: PASS

Review Claude OK (adversarial multi-agent, 1 P1 fixed in-phase, all P2/P3
handled); Codex cross-check **8/8 CONFIRME, 0 GAP** (§ Codex reconciliation);
all suites green including the canonical Docker Linux full-workspace run
**1560/1560** under load. Committable.

---

## Scope (7 P2 dette items, non-convertible)

| Item | Status | Evidence |
|---|---|---|
| P2-A-1 worker-pump 3/3 MANDATORY | CLOSED by fix | 7 pump tests → `multi_thread`; 2 virtual-time tests kept `current_thread`; dispatch recv-vs-shutdown race fixed (poll-before-shutdown). Green Windows native 9/9 + Docker Linux 9/9. §P54 rewritten. |
| P2-TEST-ZOMBIE | CLOSED | `audit_commit_valid_phase_commit` (ex `6fb95df`) + `audit_commit_non_phase_commit` (ex `c4494a6`) de-hardcoded via self-contained git fixtures. |
| P2-OPERATOR-TIMEOUT | CLOSED | client timeout configurable (`SBFB_TEST_HTTP_TIMEOUT_SECS`, default 30s). |
| P2-OPERATOR-NO-TEST-RUNNER | CLOSED | Vitest infra (`vitest.config.ts` + `src/test/setup.ts` EventSource stub) + 7 tests (5 lib + 2 component). |
| P2-POLL-DIAGNOSTIC-LOSS | CLOSED | `last_err` recorded + surfaced on timeout; mock-401 test. |
| P2-SYNC-FS-ASYNC | CLOSED | `resolve_daemon` async + `spawn_blocking` (discover stays sync for 4 sync callers); 2 tests. |
| P2-OLLAMA-MODEL-PICKER | CLOSED | backend `default_model_for_provider` + per-provider resolution (Claude keeps `claude-opus-4-8[1m]`); frontend model-picker (non-Claude) + i18n fr/en; 3 unit + 1 mock-Ollama integration + Vitest. |

---

## §7.4 Verification blocks

### Bloc Rust
- `cargo fmt --all --check` : exit 0.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` : 0 warning.
- `cargo nextest run --workspace --locked` : **1556/1556** passed, 0 skipped
  (delta +7 vs Phase A 1549 ; 1 poll-diagnostic + 2 sync-fs + 4 model-picker).
- `cargo test --workspace --locked --doc` : 0 fail.
- `cargo build -p nexus-shell-daemon --release` : OK.
- **P1 fix proof** — `cargo test -p sbfb-factory --bin sbfb-factory --locked`
  ×5 (plain cargo test = shared process, the ci.yml/verify.sh/.woodpecker
  gate) : **5/5 green** (76 passed/run) after `#[serial(sbfb_env)]`
  (was ~4/5 FAIL before). Full gate `cargo test --workspace --locked` exit 0.
- **D6 cross-platform** — Docker Linux (`sbfb-ci`) worker-pump **9/9** ;
  full-workspace nextest Docker Linux (`--profile ci`, `libgtk-3-dev`
  installed for the launcher's `atk-sys`) : **1560/1560 passed, 0 skipped**
  across 26 binaries (race-fix + serial fix hold under load on Linux; 1560
  Linux vs 1556 Windows = platform-gated test variance).

### Bloc Frontend (factory-operator — exemption Rust-first)
- `npm run test:unit` (Vitest) : 7/7 (2 files).
- `npm run build` (tsc -b + vite) : exit 0 (test files type-check).
- `npm run lint` : 0 error (3 pre-existing warnings on untouched UI files).
- web/ : not touched this phase (279 Vitest / 6 size unchanged).

---

## Adversarial review findings & dispositions

**0 P0 · 1 P1 (fixed) · 1 P2 (addressed) · 5 P3 (documented) · 4 refuted.**

### P1 (FIXED in-phase) — env-mutating sbfb-factory unit tests flake under plain `cargo test`
Empirically reproduced (~4/5 FAIL). `ci.yml`, `scripts/verify.sh`,
`.woodpecker/ci-linux.yml` all run `cargo test --workspace` (one shared
process per crate); 11 unit tests `set_var`/`remove_var` process-global env
(`SBFB_DAEMON_ENDPOINT` ×5, `NEXUS_GRID_ROOT` ×3 cross-file,
`SBFB_OLLAMA_ENDPOINT` ×2, …). nextest (process-per-test) hid it; Phase B
added 3 new participants and the in-code comments wrongly claimed isolation.
**Fix:** `serial_test` workspace dev-dep + `#[serial(sbfb_env)]` on all 11
env-mutating unit tests (provider_router ×7, operator_server ×1,
daemon_client ×1, auth ×1, publish ×1); misleading comments rewritten.
No-op cost under nextest, serialized under cargo test. Verified 5/5 green.

### P2 (ADDRESSED) — Docker Linux full-workspace run for the carry-CLOSED claim
kickoff §6 + plan rows 12/13 require worker-pump green on **both** Windows
native AND Docker Linux. Ran the canonical full-workspace `cargo nextest run`
in Docker (`sbfb-ci`) — records the race-fix holds under load on Linux. Count
recorded in the commit body.

### P3 (documented, no code defect)
1. **Zombie de-hardcode extended to a 2nd test** (`c4494a6` /
   `audit_commit_non_phase_commit`) beyond the single test named in
   kickoff/plan — same zombie class, located during implementation. Legitimate
   "located-not-scoped"; enumerated in the commit body.
2. **multi_thread scope 2→7 pump tests** (plan/kickoff body said "the 2
   tests"). Pre-located in preflight note 1; the 7-test set is the complete
   correct set per the §P54 rule; the 2 virtual-time tests stay current_thread.
   Cited in the commit body (G8 traceability).
3. **Misleading "nextest isolates" comments** — rewritten alongside the P1 fix
   to state the `#[serial]` mechanism.
4. **§P54 snippet `Docs::builder().spawn()`** imprecise vs real
   `docs_builder.spawn(...)` — doc prose softened.
5. **Plan B.2 path drift** — zombie lives in `tests/process_cli.rs` (not
   `src/`); `daemon_client.rs` deliberately left sync (the async offload is
   confined to `resolve_daemon` via `spawn_blocking`). Noted in the commit body.

### Refuted (correct-by-design, 4)
Virtual-time current_thread rationale; spawn_blocking JoinError defensive arm;
sync-fs boundary (discover stays sync); mock-Ollama substring scan (ollama-rs
0.3.4 pinned). No action.

---

## Pre-launch protocol
Wire-neutral: no `*_VERSION` / `*_ANNOUNCEMENT_VERSION` touched, no decoder
range. `#[serde(default)]` on `ChatSendRequest.model` = runtime tolerance
(pre-launch legitimate). Model rule preserved for Claude (`claude-opus-4-8[1m]`);
Ollama/Network correctly get their own default.

## Codex reconciliation
Codex GPT 5.5 (`codex exec`, raw output in `sprint73_phase_b_codex_review.md`,
NOT rewritten) audited the 8 deliverables against the working tree:
**8/8 CONFIRME, 0 GAP, 0 PARTIEL.** No fix loop required (no GAP). Codex
independently confirmed: the 7 multi_thread + 2 virtual-time split and zero
`#[cfg(windows)]` (L1); both zombie fixtures self-contained (L2); configurable
timeout (L3); real Vitest tests (L4); `last_err` surfaced on timeout (L5);
`resolve_daemon` async + `spawn_blocking`, discover stays sync (L6); per-provider
model with Claude pinned + SENSITIVE_ACTIONS gate before dispatch + mock-Ollama
captures `qwen2.5-coder:7b` not the Claude id (L7); `#[serial(sbfb_env)]` on the
env-mutating tests across the 5 files (L8 = the review P1 fix). Cross-model
validation clean.

## Next
Promote to `## Verdict: PASS` once Docker P2 (`br5y1rb3y`) confirms the
full-workspace Linux run, then commit
`fix(sprint73): Sprint 73 Phase B — close P2-A-1 worker-pump 3/3 (multi_thread) + test debt + NetworkProvider/Operator hardening`.
