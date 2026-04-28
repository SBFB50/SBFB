# Sprint 37 — Audit plan (audit gate S36->S37)

**Sprint audite** : S36 (migration Rust native Phase 2 — integration coordinator-rs daemon)
**Phases livrees** : A (dette pair + DaemonHttpState persistent), B (result submission endpoint), C (KudosLedger Rust natif)
**Tip cloture** : `f906bda`

## Track A — Phase A correctness (dette + DaemonHttpState)

| # | Check | Fichiers | Quoi verifier |
|---|---|---|---|
| A1 | CoordinatorDb double-open fix | `db.rs` | open() utilise une seule connexion (pas 2 open() successifs) |
| A2 | WAL mode active | `db.rs` | PRAGMA journal_mode=WAL dans open() |
| A3 | DaemonHttpState coordinator_db field | `http.rs` | Arc<Mutex<CoordinatorDb>> present dans struct |
| A4 | Runtime boot init | `runtime.rs` | CoordinatorDb::open(~/.sbfb/coordinator.db) au boot |
| A5 | Handler uses shared DB | `http.rs` | coordinator_submit_task lock() state.coordinator_db, pas open_in_memory() |
| A6 | submit_task free fn | `dispatcher.rs` | submit_task(&CoordinatorDb, &KeyPair, TaskSubmission) existe, TaskDispatcher delegue |

## Track B — Phase B correctness (result submission)

| # | Check | Fichiers | Quoi verifier |
|---|---|---|---|
| B1 | Route wired | `http.rs` | POST /api/v1/results/submit dans authed_routes |
| B2 | validate_result free fn | `validator.rs` | validate_result(&CoordinatorDb, &ResultEntry) existe, ResultValidator delegue |
| B3 | Handler returns correct status | `http.rs` | 200 Accepted, 400 Rejected*, 500 Error |
| B4 | Auth middleware | `http.rs` | Route dans authed_routes block avec auth_required |
| B5 | Integration tests | `http.rs` tests | 4 tests (accepts_valid, rejects_bad_sig, rejects_unknown, rejects_completed) |

## Track C — Phase C correctness (KudosLedger)

| # | Check | Fichiers | Quoi verifier |
|---|---|---|---|
| C1 | credit() inserts kudos | `kudos_ledger.rs` | insert_kudos appele avec project_id + worker_id + tokens |
| C2 | get_project_kudos returns total + contributors | `kudos_ledger.rs` | ProjectKudos avec total + Vec<ContributorKudos> |
| C3 | GET endpoint wired | `http.rs` | GET /api/v1/kudos/{project_id} dans authed_routes |
| C4 | Wire validator -> kudos | `http.rs` | credit() appele apres Accepted dans result handler |
| C5 | Non-monetary constraint | code + types | Pas de debit/stake/burn/cost dans kudos_ledger.rs |

## Track D — Cross-phase integration

| # | Check | Quoi verifier |
|---|---|---|
| D1 | DB shared dispatcher+validator+kudos | Trois modules utilisent la meme Arc<Mutex<CoordinatorDb>> via state |
| D2 | E2E pipeline | task submit -> result submit -> kudos credited -> GET kudos |
| D3 | No Python modified | 0 diff packages/ |

## Track E — Security & wire format

| # | Check | Quoi verifier |
|---|---|---|
| E1 | FORMAT_VERSION all v1 | grep _VERSION = 1 |
| E2 | Bearer auth on new endpoints | /api/v1/results/submit + /api/v1/kudos/{id} behind auth_required |
| E3 | No secrets in code | No hardcoded tokens in coordinator-rs |
| E4 | Kudos non-monetary | No debit/transfer/stake API exposed |

## Track F — Meta-process

| # | Check | Quoi verifier |
|---|---|---|
| F1 | G8 preflight 3/3 | sprint36_phase_{A,B,C}_preflight.md exist, verdicts EXECUTE |
| F2 | Phase review 3/3 | sprint36_phase_{A,B,C}_review.md exist, verdicts PASS |
| F3 | Commit bodies structured | 3 feat commits have delta tests + scope cuts + G8 trace |
| F4 | Carry counters correct | verification §5 match review findings |
| F5 | Sprint pair dette | Phase A = dette obligatoire (§6.2.1 Regle 1) |
