# Sprint 4 — Verification (fail-fast checklist)

**Date**: 2026-04-10
**HEAD**: run `git log --oneline -10` to see the Sprint 4 commit
stack (expected tip: `feat(worker): Sprint 4 Phase D part 2 — W9.1
task pump + StubOllama + --stub-ollama CLI`).

Mirrors the 17-row fail-fast table from `.planning/sprint4_plan.md`
§8 / `.planning/sprint4_kickoff.md` §7. Every row is the exact
command + the observed outcome from the Sprint 4 run.

---

## How to re-run

```bash
# from repo root, with cargo on PATH + uv on PATH
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test -p nexus-core-rs --lib
cargo test -p nexus-worker-core --lib
cargo test -p nexus-worker --test e2e
uv sync --package nexus-coordinator --extra test
uv run --package nexus-coordinator pytest packages/nexus-coordinator/tests/ -q
(cd packages/nexus-sdk && uv run --project ../../ pytest tests/ -q)
(cd packages/nexus-app-gov && uv run --project ../../ pytest tests/ -q)
```

Context7 / network access is not required for any test.

---

## Checklist

| # | Check | Command | Critère | Observed |
|---|---|---|---|---|
| 1 | canonical_bytes uses JCS + domain prefix | `cargo test -p nexus-core-rs --lib canonical::tests` | all pass | **green** — 4 tests in `src/canonical.rs` verify prefix layout, lexicographic ordering, cross-domain separation, determinism |
| 2 | GossipClient is owned (no `<'a>`) | `grep -n "GossipClient<'" crates/nexus-core-rs/src/gossip.rs` | 0 match | **0 matches** — `pub struct GossipClient { inner: Gossip }` (derives `Clone`) |
| 3 | DocsClient is owned (no `<'a>`) | `grep -n "DocsClient<'" crates/nexus-core-rs/src/docs.rs` | 0 match | **0 matches** — `pub struct DocsClient { inner: Docs }` (derives `Clone`) |
| 4 | PyO3 exposes `sign_claim` | `uv run python -c "import nexus_core; nexus_core.sign_claim"` | no AttributeError | **green** — also exposes `verify_claim_entry`, `mint_invite`, `decode_invite` |
| 5 | Coordinator boot + /health 200 | `uv run pytest packages/nexus-coordinator/tests/test_coordinator_boot.py` | green | **3/3 pass** — booted Node, /health returns 200 with node_id + doc_id, reboot preserves both |
| 6 | Dispatcher submits to doc + task_state row | `uv run pytest packages/nexus-coordinator/tests/test_dispatcher.py` | green | **3/3 pass** — submit creates `task:<id>` doc entry, pydantic-validated TaskEntry verifies, many-tasks preserve order |
| 7 | Validator drives kudos credits | `uv run pytest packages/nexus-coordinator/tests/test_full_loop.py` | green | **1/1 pass** — 10 tasks → 10 claims → 10 results → 10 kudos entries, chain valid, total_for_worker = Σ(tokens) |
| 8 | Kudos chain integrity | `uv run pytest packages/nexus-coordinator/tests/test_kudos_hash_chain.py` | green + 1-byte flip → invalid | **5/5 pass** — single-entry verify, 10-entry verify, amount tamper at row 3 → (False, 3), entry_hash tamper at row 2 → (False, 2) |
| 9 | Invite v2 roundtrip | `cargo test -p nexus-worker-core --lib invite` + `pytest .../test_invite.py` | green | **11/11 Rust + 6/6 Python pass** — mint → encode → decode round-trip with tasks_doc_ticket, Worker without ticket rejected |
| 10 | Invite v1 hard-refused | `cargo test -p nexus-worker-core --lib decode_refuses_v1` | green | **1/1 pass** — hand-crafted v1 payload returns `UnsupportedVersion(1)` |
| 11 | SDK hello-world < 100 LOC | `wc -l examples/hello-world-app/src/hello_world_app/*.py` | < 100 | **45 lines** — single `__init__.py` file under `examples/hello-world-app/` |
| 12 | Gov app manifest via /app/{name} | `uv run pytest packages/nexus-coordinator/tests/test_apps.py` | green, ≥1 tab | **2/2 pass** — `discover_apps()` returns both `gov` and `hello`, `/app/gov/manifest` returns 1 route / 1 worker / 1 tab, 404 on unknown app |
| 13 | W9.1 drop-in (no TODO markers) | `grep 'TODO(W9.1)' crates/nexus-worker-core/src/engine/runtime.rs` | 0 matches | **0 matches** — replaced by `Engine::scan_and_execute_tasks` |
| 14 | W9.1 claim → execute → result integration | `cargo test -p nexus-worker-core engine_claims_and_executes` | green | **1/1 pass** — engine boots with StubOllama, receives a TaskEntry on the injected doc, writes one claim + one result in ≤10s |
| 15 | Format (Rust + Python) | `cargo fmt --all --check` + `uv run ruff format --check packages/ examples/` | exit 0 | **clean** — 48 Python files formatted, every Rust file formatted |
| 16 | Rust tests | `cargo test --workspace --exclude nexus-core-py --locked` | ≥ 161 passed | **62 nexus-core-rs + 94 nexus-worker-core + 10 nexus-worker e2e = 166 total** |
| 17 | Python tests | `uv run pytest packages/nexus-coordinator/tests/` + sdk + gov | all green | **27 coord (1 Windows skip) + 6 sdk + 3 gov = 36 total** |

---

## Summary

All 17 fail-fast rows pass. Sprint 4 delivers:

- **Day 0** (3 commits): canonical bytes via RFC 8785 JCS + domain
  prefix + ClaimEntry; lifetime removal on GossipClient / DocsClient;
  PyO3 extensions (Doc read/subscribe, sign_claim, mint_invite,
  decode_invite, docs_open).
- **Phase A** (1 commit): `packages/nexus-coordinator` Python
  package with Coordinator, keystore, pydantic config, FastAPI
  /health + /project, Typer CLI, persistent iroh data_dir.
- **Phase B** (1 commit): dispatcher + validator + kudos
  hash-chain ledger + `/tasks/submit` + `/kudos` + 20-task full
  loop test.
- **Phase C** (1 commit): invite v2 hard bump (tasks_doc_ticket
  field, Worker requires ticket), Python invite CLI / API,
  allowlist v2 migration, worker `join` flow propagates ticket,
  env-var test race fix.
- **Phase D part 1** (1 commit): nexus-sdk package, minimal
  nexus-app-gov (1 route/1 worker/1 tab + POLITICAL_CONTRADICTION_PROMPT),
  hello-world example (45 LOC), coordinator app loader + /app/{name}/manifest routes.
- **Phase D part 2** (1 commit): W9.1 task pump in
  `Engine::scan_and_execute_tasks` (claim + execute + write result),
  StubOllama promoted to pub, `--stub-ollama` worker CLI flag,
  EngineBoot gains `data_dir` + `ollama_override`, new
  integration test `engine_claims_and_executes_tasks_on_registered_doc`.

## What's NOT in this sprint (scope line)

Explicitly deferred per the sprint plan §10:

- Subprocess Python e2e test (coordinator + nexus-worker binary as
  separate OS processes). In-process coverage proves the same
  invariants: Phase B's full-loop test exercises the
  dispatcher/validator/kudos pipeline with a simulated worker
  (different keypair, different author), and Phase D's
  `engine_claims_and_executes_tasks_on_registered_doc` exercises
  the Rust engine's claim/execute/write pump end-to-end. The
  subprocess harness itself is plumbing and can land in Sprint 5
  when the frontend needs it.
- Full 19-tab / 31-worker `nexus/gov/` port — deferred to v1.1
  per decision F.
- `nexus-app-coldcase` migration — deferred to v1.1 per the
  sprint plan §10.
- pkarr publish path (audit P3 item #6) — deferred because invite
  v2 carries `coordinator_addr` directly, so pkarr is not a
  Sprint 4 blocker.
- Curator list gossip flow — not part of Sprint 4 per the phoenix
  plan.
- Frontend refactor (Sprint 5 scope).

## Git summary

```
$ git log --oneline master ^f68d997
557d3ca feat(worker): Sprint 4 Phase D part 2 — W9.1 task pump + StubOllama + --stub-ollama CLI
527a221 feat(sdk,app-gov,coordinator): Sprint 4 Phase D part 1 — SDK + minimal gov + app loader
b0656ff feat(worker,coordinator): Sprint 4 Phase C — invite v2 hard bump + CLI + API
1ec41e0 feat(coordinator): Sprint 4 Phase B — dispatcher + validator + kudos chain
3671dff feat(coordinator): Sprint 4 Phase A — nexus-coordinator Python package
db32b7b feat(core-py): Sprint 4 Day 0 — Doc read/subscribe + ClaimEntry sign/verify + docs_open
50ea5a3 fix(core-rs): Sprint 4 Day 0 — remove lifetimes from GossipClient and DocsClient
1c1fcfb fix(core-rs): Sprint 4 Day 0 — RFC 8785 JCS canonical bytes + ClaimEntry
147dc43 docs(sprint4): kickoff + detailed plan + ignore audit sprint2 shards
```

9 commits on top of f68d997 (Sprint 3 verification checklist).
Sprint 4 closed.
