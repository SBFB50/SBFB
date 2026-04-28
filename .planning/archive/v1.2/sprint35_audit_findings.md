# Sprint 35 — Audit findings (gate S35→S36)

**Date** : 2026-04-28
**Auditeur** : session fraiche, sans historique S35
**Sprint audite** : S35 (migration Rust native — Phase 1 fondations)
**Tip cloture** : `de054f9` (Phase C), wrap-up `12eb11f` (Phase D)
**Audit plan** : `.planning/active/sprint36_audit_plan.md`
**Verdict** : **PASS** (0 P0, 0 P1, 2 P2, 1 P3)

---

## Track A — Phase A correctness (MANDATORY + fondation)

| # | Check | Verdict | Detail |
|---|---|---|---|
| A1 | shellcheck CI workflow valid | ✅ PASS | YAML valide, `paths: ['scripts/**/*.sh']` push+PR, `--severity=warning` |
| A2 | cross-daemon E2E test | ✅ PASS | `DaemonCluster::spawn(2)`, publish+serve blob sur daemon A, health check daemon B |
| A3 | REPO_URL documented | ✅ PASS | `scripts/install-node.sh` L25 `TODO(v1.0)` + L27 `REPO_URL="..."` |
| A4 | CoordinatorDb schema v1 | ✅ PASS | `schema_version` table, `INSERT (1)`, `schema_version()` returns 1 |
| A5 | DB CRUD roundtrip | ✅ PASS | 6 tests : insert/get, update status, set_result, reject completed, nonexistent, kudos sum |

## Track B — Phase B correctness (dispatcher)

| # | Check | Verdict | Detail |
|---|---|---|---|
| B1 | TaskEntry::sign canonical bytes | ✅ PASS | `TASK_FORMAT_VERSION=1` dans Task, `DOMAIN_TASK_V1` dans canonical_bytes, sign Ed25519 |
| B2 | Dispatcher validates input | ✅ PASS | `prompt.is_empty()` + `model.is_empty()` → `CoordinatorError::Validation` |
| B3 | Dispatcher persists task | ✅ PASS | `TaskRecord{status: Pending}` + `insert_task()` + `get_task()` roundtrip |
| B4 | Unique task IDs | ✅ PASS | `assert_ne!(e1.task.task_id, e2.task.task_id)` — format `{timestamp:016x}-{rand:016x}` |
| B5 | Endpoint wired | ✅ PASS | `POST /api/v1/tasks/submit` dans `authed_routes` (http.rs L256) |

## Track C — Phase C correctness (validator)

| # | Check | Verdict | Detail |
|---|---|---|---|
| C1 | Signature verification | ✅ PASS | `entry.verify_signature().is_err()` → `RejectedBadSignature`, test `signature[0] ^= 0xff` |
| C2 | Task existence check | ✅ PASS | `get_task()` → `None` → `RejectedTaskNotFound` |
| C3 | Status guard | ✅ PASS | `!= Pending && != Dispatched` → `RejectedTaskNotPending`, test `set_task_result` puis re-submit |
| C4 | Accept dispatched | ✅ PASS | `update_task_status(Dispatched)` puis validate → `Accepted` |
| C5 | State transition | ✅ PASS | `set_task_result()` SQL `status='completed'` + `WHERE IN ('pending','dispatched')`, double-complete returns false |

## Track D — Cross-phase integration

| # | Check | Verdict | Detail |
|---|---|---|---|
| D1 | Shared types | ✅ PASS | `TaskRecord` + `TaskStatus` dans `types.rs`, importe par dispatcher, validator, db |
| D2 | Daemon dep | ✅ PASS | `nexus-coordinator-rs = { path = "../nexus-coordinator-rs" }` dans daemon Cargo.toml L30 |
| D3 | pow_keypair usage | ✅ PASS | `(*state.pow_keypair).clone()` dans handler (http.rs L1280), keypair du daemon |

## Track E — Security & wire format

| # | Check | Verdict | Detail |
|---|---|---|---|
| E1 | FORMAT_VERSION all v1 | ✅ PASS | TASK=1, TASK_RESPONSE=1, CURATOR_LIST=1, 6 DOMAIN_*_V1 constants |
| E2 | No Python modified | ✅ PASS | `git diff 2a79c8e..12eb11f -- packages/` = 0 lignes |
| E3 | Bearer auth on submit | ✅ PASS | Route dans `authed_routes` block + `auth_required` middleware (http.rs L257) |
| E4 | No secrets in code | ✅ PASS | Grep secrets/tokens/keys/passwords = 0 match dans coordinator-rs |

## Track F — Meta-process

| # | Check | Verdict | Detail |
|---|---|---|---|
| F1 | G8 preflight 3/3 | ✅ PASS | phase_{A,B,C}_preflight.md existent, 3× EXECUTE plan-as-is |
| F2 | Phase review 3/3 | ✅ PASS | phase_{A,B,C}_review.md existent, 3× PASS avec findings documentes |
| F3 | Commit bodies structures | ✅ PASS | 3 feat + 1 fix, tous avec delta tests + scope cuts + G8 trace + Co-Authored-By |
| F4 | Carry counters correct | ✅ PASS | verification §5 = 5 P2 + 1 P2-AUDIT = match exact reviews A(2)+B(1)+C(2) |
| F5 | MANDATORY 3/3 resolved | ✅ PASS | shellcheck CI + cross-daemon E2E + REPO_URL documentes et verifies |

---

## Findings independants

### P2-AUDIT-1 — Handler submit ouvre in-memory DB par requete

**Fichier** : `crates/nexus-shell-daemon/src/http.rs` L1269
**Constat** : `coordinator_submit_task` appelle
`CoordinatorDb::open_in_memory()` a chaque requete HTTP. Les taches
soumises via l'endpoint ne sont pas persistees entre requetes.
L'endpoint signe et retourne un TaskEntry valide mais la tache
n'est pas recuperable apres la reponse.
**Deja identifie** : P2-REVIEW-B-1, documente dans le commit body
Phase B comme "proof of concept" avec carry S36.
**Action** : S36 — integrer `CoordinatorDb` persistant dans
`DaemonHttpState`, passer la reference au dispatcher.

### P2-AUDIT-2 — Transitives iroh pre-release

**Constat** : le crate graph inclut des transitives pre-release via
iroh 0.98 (iroh-dns-resolver, iroh-relay, etc.). Condition heritee
du pin Day-0 iroh 0.98.
**Deja identifie** : S35 verification §5 carry-over.
**Action** : re-evaluer a chaque upgrade iroh.

### P3-AUDIT-1 — Cross-daemon E2E sans cross-fetch

**Fichier** : `crates/nexus-test-harness/tests/cross_daemon_blob.rs`
**Constat** : le test publie et sert sur daemon A, verifie la sante
de daemon B, mais ne teste pas le fetch cross-daemon (B recuperant
un blob publie par A). Le MANDATORY 3/3 est satisfait (un test E2E
fonctionnel multi-daemon existe) mais le scenario cross-fetch reste
non couvert.
**Deja identifie** : P3-REVIEW-A-1.
**Action** : enrichir le test S36+ quand iroh-blobs fetch cross-node
sera wired.

---

## Recoupement review findings → audit

| Review finding | Confirme ? | Commentaire |
|---|---|---|
| P2-REVIEW-A-1 LOC kickoff | ✅ | Estimations §4 D5 imprecises, chore(planning) futur |
| P2-REVIEW-A-2 double-open DB | ✅ | `open()` peut etre factorise, S36 carry |
| P2-REVIEW-B-1 dispatcher DB in-memory | ✅ | = P2-AUDIT-1 confirme independamment |
| P2-REVIEW-C-1 validator_loop tokio | ✅ | Scope-cut coherent, S36 carry |
| P2-REVIEW-C-2 kudos credit | ✅ | KudosLedger Rust = S36 carry |
| P3-REVIEW-A-1 cross-fetch | ✅ | = P3-AUDIT-1 confirme independamment |
| P3-REVIEW-B-1 no HTTP integration test | ✅ | Thin handler, P3 acceptable |
| P3-REVIEW-C-1 model_digest/logprobs | ✅ | Couche 2/3 future, S36+ carry |

---

## Verdict

**PASS** — 0 P0, 0 P1, 2 P2 (confirmes carry S36), 1 P3.

27/27 checks du audit plan verifies. Les 3 MANDATORY 3/3
(shellcheck CI + cross-daemon E2E + REPO_URL) sont resolus.
Le crate `nexus-coordinator-rs` (dispatcher + validator) est
correctement implemente avec types partages, canonical bytes
JCS, signature Ed25519, persistence SQLite, et protection
auth middleware. Les scope cuts du kickoff §7 sont tous
respectes (0 Python modifie, 0 scope creep).

G4 rigor signal : 2 P2 + 1 P3 documentes (>= 1 P2+ exige).

Gate S36 : **OUVERTE**. Le sprint 36 peut proceder a sa
Phase A.
