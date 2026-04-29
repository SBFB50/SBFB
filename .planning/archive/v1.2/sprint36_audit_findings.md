# Sprint 36 — Audit findings (Phase 0 gate S37)

**Auditeur** : session fraiche post-S36.
**Tip audite** : `911274f` (chore sprint36 Phase D wrap-up)
**Sprint audite** : S36 (migration Rust native Phase 2 — integration)
**Date** : 2026-04-28

## Verdict : PASS (0 P0/P1, 2 P2, 1 P3)

Aucun finding bloquant. Sprint correctement implemente. Les
28 checks de l'audit plan passent. 2 P2 documentes.

---

## Track A — Phase A correctness (6/6 PASS)

| # | Check | Status | Evidence |
|---|---|---|---|
| A1 | CoordinatorDb double-open fix | PASS | `db.rs:62-70` — single `Connection::open(path)`, WAL, migrations sur `&mut conn`. |
| A2 | WAL mode active | PASS | `db.rs:64` — `pragma_update("journal_mode", "WAL")`. Test `open_file_activates_wal_mode` L345 confirme. |
| A3 | DaemonHttpState coordinator_db field | PASS | `http.rs:151` — `pub coordinator_db: Arc<Mutex<CoordinatorDb>>`. |
| A4 | Runtime boot init | PASS | `runtime.rs:468-471` — `CoordinatorDb::open(path)` -> `Arc::new(Mutex::new(...))` -> `DaemonHttpState` L506. |
| A5 | Handler uses shared DB | PASS | `http.rs:1276` — `state.coordinator_db.lock()`. 0 appels `open_in_memory()` dans les handlers. |
| A6 | submit_task free fn | PASS | `dispatcher.rs:18-80` — `pub fn submit_task(&CoordinatorDb, &KeyPair, TaskSubmission)`. `TaskDispatcher::submit` delegue L93. |

## Track B — Phase B correctness (5/5 PASS)

| # | Check | Status | Evidence |
|---|---|---|---|
| B1 | Route wired | PASS | `http.rs:262` — `.route("/api/v1/results/submit", post(coordinator_submit_result))` dans `authed_routes`. |
| B2 | validate_result free fn | PASS | `validator.rs:23-73` — `pub fn validate_result(&CoordinatorDb, &ResultEntry)`. `ResultValidator` delegue L84-85. |
| B3 | Handler returns correct status | PASS | `http.rs:1329-1377` — 200 Accepted, 400 Rejected*, 500 Error. Formate JSON `{"outcome": "accepted"/"rejected", "reason": "..."}`. |
| B4 | Auth middleware | PASS | Routes L261-263 dans `authed_routes` block avec `.layer(auth_required)` L264. |
| B5 | Integration tests | PASS | 4 tests : `result_submit_accepts_valid` L3204, `rejects_bad_signature` L3248, `rejects_unknown_task` L3283, `rejects_completed_task` L3309. |

## Track C — Phase C correctness (5/5 PASS)

| # | Check | Status | Evidence |
|---|---|---|---|
| C1 | credit() inserts kudos | PASS | `kudos_ledger.rs:13-48` — cree `KudosEntry` avec `amount = tokens_generated`, appelle `db.insert_kudos()`. |
| C2 | get_project_kudos returns total + contributors | PASS | `kudos_ledger.rs:63-82` — retourne `ProjectKudos { project_id, total, contributors: Vec<ContributorKudos> }`. |
| C3 | GET endpoint wired | PASS | `http.rs:263` — `.route("/api/v1/kudos/{project_id}", get(coordinator_get_kudos))`. |
| C4 | Wire validator -> kudos | PASS | `http.rs:1331-1343` — apres Accepted, appelle `credit()` avec project_id + worker_id + task_id + tokens. Non-fatal (warn log). |
| C5 | Non-monetary constraint | PASS | Grep `debit\|stake\|burn\|transfer\|cost\|deposit\|refund` dans `kudos_ledger.rs` = 0 matches. Doc L4 "non-monetary, non-transferable". |

## Track D — Cross-phase integration (3/3 PASS)

| # | Check | Status | Evidence |
|---|---|---|---|
| D1 | DB shared dispatcher+validator+kudos | PASS | 3 handlers (`coordinator_submit_task` L1276, `coordinator_submit_result` L1317, `coordinator_get_kudos` L1388) utilisent `state.coordinator_db.lock()`. |
| D2 | E2E pipeline | PASS | Test `e2e_task_result_kudos_credited` L3354 : submit_task -> submit_result -> kudos credited -> `total > 0`. |
| D3 | No Python modified | PASS | `git diff 148c65f..f906bda --stat -- packages/` = 0 fichiers. |

## Track E — Security & wire format (4/4 PASS)

| # | Check | Status | Evidence |
|---|---|---|---|
| E1 | FORMAT_VERSION all v1 | PASS | Grep : TASK=1, CURATOR_LIST=1, PIN_FILE=1, POW=1, KEY_ROTATION=1. |
| E2 | Bearer auth on new endpoints | PASS | 3 routes S36 dans `authed_routes` avec `auth_required` middleware. |
| E3 | No secrets in code | PASS | Grep `AKIA\|ghp_\|pat_\|sbfb_\|password=\|secret=` dans coordinator-rs/src/ = 0 matches. |
| E4 | Kudos non-monetary | PASS | 0 API debit/transfer/stake exposee. `credit()` et `get_project_kudos()` seulement. |

## Track F — Meta-process (5/5 PASS)

| # | Check | Status | Evidence |
|---|---|---|---|
| F1 | G8 preflight 3/3 | PASS | `archive/v1.2/sprint36_phase_{A,B,C}_preflight.md` presents. Verdicts EXECUTE. |
| F2 | Phase review 3/3 | PASS | `archive/v1.2/sprint36_phase_{A,B,C}_review.md` presents. Verdicts PASS. |
| F3 | Commit bodies structured | PASS | 3 feat commits avec Context, Fichiers, Delta tests, Scope cuts, G8, Pre-launch protocol. |
| F4 | Carry counters correct | PASS* | Verification §5 regroupe correctement A-1/B-1 + C-1/C-2. *P2-REVIEW-A-2 absent car prevu resolu Phase D — cf. P2-AUDIT-1. |
| F5 | Sprint pair dette | PASS | Phase A = dette obligatoire. Commit body cite §6.2.1 Regle 1. |

---

## Findings

### P2-AUDIT-1 : HARDENING_ROADMAP compteurs non consolides Phase D

`docs/security/HARDENING_ROADMAP.md:3` — le `last_validated` montre
les compteurs Phase A (~927 Rust / ~1930 total) alors que le sprint
a livre 936 Rust / ~1939 total (+12 en 3 phases).

`sprint36_verification.md` check #20 affirme "PASS — updated
(936 Rust / ~1939 total)" mais le fichier n'a PAS ete mis a jour
apres Phase A. Le P2-REVIEW-A-2 ("HARDENING compteurs approximatifs")
etait prevu comme resolu en Phase D (Phase A review L69 : "Le
verification.md Phase D les consolidera") mais le Phase D wrap-up
ne l'a pas fait.

**Impact** : compteurs stale dans un document long-life avec
`triggers_revalidate`. Pas bloquant (delta faible +9 tests Rust)
mais l'auto-attestation check #20 est factuellement incorrecte.

**Fix recommande** : `fix(sprint36): update HARDENING compteurs
936 Rust / ~1939 total` en S37 ou carry S37.

### P2-AUDIT-2 : unwrap_or_default() sur serde_json::to_value

`http.rs:1290` (`coordinator_submit_task`) et `http.rs:1402`
(`coordinator_get_kudos`) :

```rust
serde_json::to_value(&entry).unwrap_or_default()
```

Si la serialisation echoue (impossible avec les types actuels
derivant Serialize, possible si un futur sprint ajoute un champ
non-serialisable), le handler retourne 200 OK avec `null` au
lieu de 500 Internal Server Error. Le pattern masque silencieusement
l'erreur au lieu de la remonter.

**Impact** : faible actuellement. Contraire au principe
defense-in-depth. Un changement futur casserait silencieusement.

**Fix recommande** : remplacer par `.expect("type derives Serialize")`
ou `.map_err(|e| { tracing::error!(...); (500, Json(...)) })`.

### P3-AUDIT-1 : Clone inutile de pow_keypair

`http.rs:1287` — `let keypair = (*state.pow_keypair).clone()` cree
une copie du materiel cryptographique alors que `submit_task` prend
`&KeyPair`. `&*state.pow_keypair` suffirait.

Nit cosmetique — la copie est ephemere sur le stack, pas de risque.

---

## Scope cuts verifies

12/12 scope cuts du kickoff §7 : tous respectes. 0 fichier
packages/ touche. Hash-chain vide (prev_hash/entry_hash = "")
confirme scope cut §7.12. Kudos debit/stake interdit (Day 0 #7).

## Carry S37 consolide

### Phase reviews (verification §5)
- P2-REVIEW-A-1 / B-1 : mutex poisoned branch non testee (3 handlers)
- P2-REVIEW-C-1 : hash-chain vide (prev_hash + entry_hash placeholders)
- P2-REVIEW-C-2 : double query project_id dans result handler

### Audit gate (ce document)
- P2-AUDIT-1 : HARDENING compteurs stale (fix trivial)
- P2-AUDIT-2 : unwrap_or_default() handlers (carry S37)

### Herites (inchanges)
- P2-A-1 rand blocker upstream (exemption blocker externe)
- P2-B-1-S34 log convergence 3/3 **MANDATORY S37**
- P2-C-1-S34 .icns macOS 3/3 **MANDATORY S37**
- P2-REVIEW-C-1-S35 validator_loop tokio 2/3 -> 3/3 **MANDATORY si reporte S38**
- P2-AUDIT-2-S35 pre-release transitives iroh
