# NEXUS Compute — Feature status matrix

_Last updated: Sprint 0 stabilization pass (SBFB pivot, 2026-04-10)_

This document is the honest current state of every feature in
`nexus/compute/` after Sprint 0. It is the reference for scoping the
SBFB rewrite — Rust `nexus-core-rs` + `nexus-worker` in Sprints 2-3
should reach **stable** on everything marked stable below, and can
defer or drop anything marked experimental.

## Legend

| Level | Meaning |
|---|---|
| **stable** | Code works in production, has regression tests, no known bugs |
| **beta** | Code works in common paths but has limitations or partial test coverage |
| **experimental** | Works in specific conditions only; may be dropped or replaced in the SBFB rewrite |
| **disabled** | Present in the tree but off by default; scheduled for replacement or removal |

## Feature matrix

| Feature | Module | Status | Notes |
|---|---|---|---|
| Compute database schema (tables, indexes, FTS) | `db.py` | **stable** | 16 tables, idempotent init, WAL mode. 1136 LOC. |
| Node registration + API key hashing | `db.py` (`generate_api_key`, `hash_api_key`, `hash_ip`) | **stable** | Public helpers after J7 rename. SHA-256 hashing, no raw IP stored. |
| Task queue CRUD | `db.py::create_task`, `get_task`, `list_tasks`, `count_tasks` | **stable** | JSON metadata field, priority ordering, status state machine. |
| Atomic task pulling | `db.py::pull_next_task` | **stable** | `BEGIN IMMEDIATE` optimistic lock, model affinity, prevents double-assignment. |
| Task expiry / stale reaper | `dispatcher.py::_reaper_loop` | **stable** | Periodic sweep configurable via `settings.compute_reaper_interval`. |
| Heartbeat monitor | `dispatcher.py::_heartbeat_monitor` | **stable** | Marks offline nodes, unassigns their tasks, recalculates model tier. |
| 3-layer verification (Ed25519 + digest + logprob) | `verification.py`, `crypto.py` | **stable** | 265 + 176 LOC. `cryptography>=43` pinned in Sprint 0 J2. Silent import fallback if lib missing. |
| BOINC-style spot-check probability | `verification.py::spot_check_needed`, `dispatcher.py::_get_spot_check_rate` | **stable** | 1/5/20% by trust tier. Covered by tests. |
| Spot-check event publication | `dispatcher.py::validate_result` | **stable** | Publishes `COMPUTE_SPOT_CHECK_NEEDED` on flagged results. |
| Spot-check consumer + cross-verification | `dispatcher.py::SpotCheckCoordinator` | **beta** | **NEW in Sprint 0 J6.** Subscribes to the event, creates duplicate tasks with `metadata.spot_check_for`. `_resolve_spot_check()` compares normalized strings: match = +5 trust, mismatch = −20 trust + `COMPUTE_RESULT_REJECTED`. 7 regression tests. Beta because string-equality comparison is a minimal comparator and should be upgraded to embedding cosine in v1.1. |
| Network stats + leaderboard | `db.py::get_network_stats`, `get_leaderboard` | **stable** | Aggregated online/offline/total, VRAM sum, tasks today. |
| Kudos / badges ledger | `db.py` (compute_badges, compute_uptime_log) | **beta** | Tables present, no hash-chain integrity yet (planned for SBFB v1.4). |
| Auto-scaling model selector | `model_selector.py` | **stable** | 489 LOC. Recalculates target model tier from total VRAM and node count. |
| Hybrid router (local / distributed / petals / exo) | `hybrid.py` | **beta** | 305 LOC. Works for local and distributed paths. Petals path is disabled (see below). Exo mode is experimental — not tested in Sprint 0. |
| Self-worker (embedded GPU contribution) | `self_worker.py` | **beta** | 357 LOC. Auto-registers on boot, auto-detects GPU via nvidia-smi or `pynvml` (optional dep). J7 cleaned dead imports. Works on Windows+Linux with NVIDIA; AMD and Apple Silicon untested. |
| Ollama client backend | `self_worker.py` (Ollama generate path) | **stable** | Uses `ollama>=0.4` async client. |
| Petals distributed swarm manager | `swarm.py` | **disabled** | **FIXED in Sprint 0 J5.** Was loading full Petals model as a "health check" every 60s (production bug). Now guarded by `settings.petals_enabled` (default False). When enabled, uses proper `hivemind.DHT` + `get_remote_module_infos` probe. Scheduled for replacement by llama.cpp RPC split inference in SBFB v1.3. |
| Petals backend (model loading) | `petals_backend.py` | **disabled** | 204 LOC. Alpha upstream, off critical path. Not imported unless `petals_enabled=True` AND the `petals` package is installed. |
| Exo peer discovery | `hybrid.py`, `exo_peer.py` (in `worker/`) | **experimental** | Placeholder for MLX heterogeneous clusters. Not wired into the Sprint 0 reference run. |
| Compute HTTP API | `nexus/api/compute.py` | **stable** | 18 endpoints. Rate-limited by hashed IP. `hash_ip` public after J7. |
| Compute event types | `events.py` (16 `ComputeEventType` values) | **stable** | Enum covers node lifecycle, task lifecycle, validation, model changes, ticks. |
| Compute database proxy for long-lived workers | `events.py::ComputeDatabaseProxy` | **stable** | Opens a fresh connection per method call, avoids holding conn across await boundaries. |
| Compute manager lifecycle | `manager.py` | **stable** | Starts/stops model selector, dispatcher, spot-check coordinator (NEW J6), and self-worker in order. |

## Silent imports

The Sprint 0 J1 audit identified four `try/except ImportError` blocks
that let the module run in degraded mode when a dependency is
missing. After Sprint 0 the status is:

| Dep | Pinned in | Fallback behavior if missing |
|---|---|---|
| `cryptography>=43` | `requirements.txt` (J2) | Verification layer 1 (Ed25519 signature check) becomes a no-op; other layers still fire. |
| `rich>=13` | `requirements.txt` (J2), already in `worker/pyproject.toml` | Would crash `start_nexus.py` and `worker/dashboard.py` — now pinned so this cannot happen on a fresh install. |
| `pynvml` (nvidia-ml-py) | `worker/pyproject.toml` optional extra `[nvidia]` | `SelfWorker._detect_gpu()` falls back to `nvidia-smi` CLI, then to `(vram_mb=0, gpu_model="CPU-only")`. |
| `petals` | unpinned (intentional) | `SwarmManager` stays in `OFFLINE` state and `HybridRouter` does not route to `PETALS` mode. Controlled by `settings.petals_enabled` (default False). |

## Tests

After Sprint 0: **797 tests passing** (up from 786 on the starting
state). Breakdown of compute-specific coverage:

- `tests/test_compute.py` — 109+ tests (DB CRUD, auth helpers,
  dispatcher, event types, Pydantic models, config, module
  imports). NEW in Sprint 0: `TestSpotCheckCoordinator` × 7 tests.
- `tests/test_compute_phase2.py` — model selector basics
- `tests/test_compute_phase4.py` — heartbeat and reaper
- `tests/test_compute_phase5.py` — 3-layer verification
- `tests/test_compute_phase6.py` — verification + spot-check
  probability
- `tests/test_compute_phase7.py` — Petals/swarm. NEW in Sprint 0:
  4 tests for the `petals_enabled` guard and DHT probe guards.
- `tests/test_compute_phase8.py` — hybrid routing
- `tests/test_sync.py`, `tests/test_worker.py`, `tests/test_e2e_gov.py`
  — supporting

## Known limitations (not yet addressed)

These are outside Sprint 0's scope and should be picked up later,
either in a hotfix phase or in the SBFB rewrite.

1. **No 2-machine LLM benchmark**. Only single-machine synthetic
   numbers exist (`docs/BENCHMARK_COMPUTE.md`). Real tokens/sec,
   heartbeat latency and crash behavior measurements need a
   second GPU host.
2. **Spot-check comparator is string equality**. Upgrading to an
   embedding cosine similarity (reusing the existing nomic-embed
   pipeline) is a v1.1 concern. A sophisticated cheater that
   changes wording but preserves meaning would currently escape
   detection.
3. **Kudos ledger has no hash chain**. Append-only writes but no
   integrity proof. Planned for SBFB v1.4.
4. **No Exo peer testing**. The Exo hybrid mode is shipped
   experimentally and has never been exercised in the Sprint 0
   reference run.
5. **AMD + Apple Silicon GPU detection** in `self_worker.py` is
   NVIDIA-only today. Cross-platform detection is a Sprint 3
   concern for the Rust `nexus-worker` binary.
6. **Petals replacement**. The whole `swarm.py` + `petals_backend.py`
   pair is disabled and scheduled for replacement by `llama.cpp
   --rpc` in SBFB v1.3.

## How Sprint 0 changed this file's reality

This document exists as of Sprint 0 J8. Every row above reflects
the state on the `stabilize/compute` branch _after_ the following
Sprint 0 commits landed:

1. `feat(compute): import full distributed GPU stack` — brings
   the whole module into git (it was uncommitted before)
2. `feat(gov,web): new workers, frontend shell refactor, OSS repo
   hygiene` — the rest of the working tree
3. `test(vram_scheduler): bind light-model test to settings.model_fast`
4. `deps(compute): pin cryptography and rich in requirements.txt`
5. `fix(compute): stop SwarmManager from loading full Petals model
   as health check` ← **fixes a production bug**
6. `feat(compute): consume COMPUTE_SPOT_CHECK_NEEDED and resolve
   cross-checks` ← **closes a dead-letter event handler**
7. `refactor(compute): promote private auth helpers + drop dead
   imports` ← **removes convention violations**
8. `bench(compute): synthetic dispatcher benchmark + Sprint 0
   baseline numbers` ← **reference for the Rust port**

`pytest tests/ -q` → **797 passed, 0 failed** on the final state.
