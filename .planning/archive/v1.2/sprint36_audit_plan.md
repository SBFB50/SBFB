# Sprint 36 — Audit plan (audit gate S35→S36)

**Sprint audite** : S35 (migration Rust native — Phase 1 fondations)
**Phases livrees** : A (MANDATORY 3/3 + fondation crate), B (dispatcher Rust natif), C (validator Rust natif)
**Tip cloture** : `de054f9`

## Track A — Phase A correctness (MANDATORY + fondation)

| # | Check | Fichiers | Quoi verifier |
|---|---|---|---|
| A1 | shellcheck CI workflow valid | `.github/workflows/shellcheck.yml` | YAML valid, trigger paths correct, severity=warning |
| A2 | cross-daemon E2E test functional | `cross_daemon_blob.rs` | DaemonCluster(2) spawn, publish + serve on A, health on B |
| A3 | REPO_URL documented | `install-node.sh` | TODO(v1.0) comment present, URL updated |
| A4 | CoordinatorDb schema v1 | `db.rs` | schema_version table exists, returns 1 |
| A5 | DB CRUD roundtrip | `db.rs` tests | insert/get/update/set_result all tested |

## Track B — Phase B correctness (dispatcher)

| # | Check | Fichiers | Quoi verifier |
|---|---|---|---|
| B1 | TaskEntry::sign canonical bytes | `dispatcher.rs` | Uses TASK_FORMAT_VERSION + DOMAIN_TASK_V1 |
| B2 | Dispatcher validates input | `dispatcher.rs` | Rejects empty prompt, empty model |
| B3 | Dispatcher persists task | `dispatcher.rs` | insert_task called, get_task returns pending |
| B4 | Unique task IDs | `dispatcher.rs` | Two submits produce different task_ids |
| B5 | Endpoint wired | `http.rs` | POST /api/v1/tasks/submit route exists |

## Track C — Phase C correctness (validator)

| # | Check | Fichiers | Quoi verifier |
|---|---|---|---|
| C1 | Signature verification | `validator.rs` | Tampered signature → RejectedBadSignature |
| C2 | Task existence check | `validator.rs` | Unknown task → RejectedTaskNotFound |
| C3 | Status guard | `validator.rs` | Completed task → RejectedTaskNotPending |
| C4 | Accept dispatched | `validator.rs` | Dispatched task accepts result |
| C5 | State transition | `validator.rs` | Accepted → task.status = Completed in DB |

## Track D — Cross-phase integration

| # | Check | Quoi verifier |
|---|---|---|
| D1 | Dispatcher + validator use same types | TaskRecord, TaskStatus shared between modules |
| D2 | Coordinator-rs dep in daemon | nexus-shell-daemon Cargo.toml has dep |
| D3 | Endpoint uses daemon keypair | pow_keypair from DaemonHttpState |

## Track E — Security & wire format

| # | Check | Quoi verifier |
|---|---|---|
| E1 | FORMAT_VERSION all v1 | grep _VERSION in canonical.rs/schemas |
| E2 | No Python code modified | 0 diff packages/ |
| E3 | Bearer auth on new endpoint | /api/v1/tasks/submit behind auth_required |
| E4 | No secrets in new code | No hardcoded tokens or keys in coordinator-rs |

## Track F — Meta-process

| # | Check | Quoi verifier |
|---|---|---|
| F1 | G8 preflight ran 3/3 phases | sprint35_phase_{A,B,C}_preflight.md exist, verdicts EXECUTE |
| F2 | Phase review ran 3/3 phases | sprint35_phase_{A,B,C}_review.md exist, verdicts PASS |
| F3 | Commit bodies structured | 3 feat commits have delta tests + scope cuts + G8 trace |
| F4 | Carry counters correct | §5 findings match review findings |
| F5 | MANDATORY 3/3 resolved | shellcheck + cross-daemon + REPO_URL all closed |
