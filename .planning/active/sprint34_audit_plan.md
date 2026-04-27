# Sprint 34 — Audit plan (audit gate S33→S34)

**Sprint audité** : S33 (multi-node readiness)
**Phases livrées** : A (CORS + LOC guard + nits), B (deploy infra), C (test harness + smoke)
**Tip cloture** : `3d3bd96`

## Track A — Phase-level correctness

| # | Check | Fichiers | Quoi vérifier |
|---|---|---|---|
| A1 | CORS daemon opt-in preserves loopback default | `http.rs` | Sans `--cors-origin`, external origins toujours rejetés |
| A2 | CORS coordinator opt-in preserves localhost default | `app.py`, `start.py` | Sans `--cors-origin`, regex loopback seul |
| A3 | LOC guard hook blocks `~NNN LOC` in plan staging | `phase-precommit-lightcheck.sh` | Check 6 grep patterns correct |
| A4 | systemd units ExecStart matches real CLI | `configs/systemd/*.service` | Binary paths + flags matchent `--headless` / `--host` / `--port` |
| A5 | install-node.sh idempotent | `scripts/install-node.sh` | Second run avec `--yes` ne casse pas |
| A6 | Test harness isolation | `lib.rs` | `NEXUS_GRID_ROOT` + `SBFB_HOME` par daemon = 0 collision |
| A7 | Test harness cleanup | `lib.rs` | `kill_on_drop` + TempDir drop = pas de zombie processes |

## Track B — Cross-phase integration

| # | Check | Quoi vérifier |
|---|---|---|
| B1 | CORS flag documented in systemd comment | `nexus-daemon.service` override example mentions `--cors-origin` |
| B2 | install-node.sh builds daemon that test harness can spawn | Binary path consistency debug/release |
| B3 | Smoke test script uses same env isolation as harness | `NEXUS_GRID_ROOT` + `SBFB_HOME` pattern consistent |

## Track C — Security & hardening

| # | Check | Quoi vérifier |
|---|---|---|
| C1 | CORS origin validation rejects malformed | `http.rs` validate_cors_origin() rejects `javascript:` / no scheme |
| C2 | Auth token per-daemon-instance | Test harness reads distinct tokens per SBFB_HOME |
| C3 | systemd User=nexus (least privilege) | Services don't run as root |

## Track D — Meta-process

| # | Check | Quoi vérifier |
|---|---|---|
| D1 | G8 preflight ran 3/3 phases | `sprint33_phase_{A,B,C}_preflight.md` exist, verdicts documented |
| D2 | Phase review ran 3/3 phases | `sprint33_phase_{A,B,C}_review.md` exist, verdicts PASS |
| D3 | Commit bodies have delta tests + scope cuts | All 3 feat commits have structured body |
| D4 | No LOC estimates in plan/kickoff | Grep `~NNN LOC` = 0 matches hors LOC guard description |
| D5 | Carry counters incremented correctly | carry_summary.md counters match review findings |

## Track E — MANDATORY carries S34

| # | Item | Compteur | Action audit |
|---|---|---|---|
| E1 | P2-A-1 rand triple | 3/3 MANDATORY | Vérifier unification possible rand workspace |
| E2 | P2-B-1 tor-rtcompat | 3/3 MANDATORY | Vérifier cleanup résidu tokio-runtime-compat |
| E3 | P2-REVIEW-C-2 COEP E2E | 3/3 MANDATORY | Vérifier faisabilité blob-serve zip réel test |

## Track F — Sprint 33 specifics

| # | Check | Quoi vérifier |
|---|---|---|
| F1 | Multi-daemon tests reliable | Run `cargo nextest run -p nexus-test-harness` 3x, 0 flakes |
| F2 | Smoke test portable | `scripts/test-multi-node.sh` runs on Linux (CI target) |
| F3 | Phase review files present | 3/3 reviews in archive |
