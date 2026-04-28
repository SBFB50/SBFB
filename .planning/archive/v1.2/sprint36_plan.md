# Sprint 36 — Plan (migration Rust native — Phase 2 integration)

**Kickoff** : `sprint36_kickoff.md`
**Tip d'entree** : `3013e44`

---

## §1 Phase A — Dette pair + DaemonHttpState persistent

### Scope

1. **CoordinatorDb::open(path)** — ajouter une methode publique qui
   ouvre le fichier SQLite avec WAL mode (`PRAGMA journal_mode=WAL`).
   Le chemin par defaut est `~/.sbfb/coordinator.db`. La methode
   `open_in_memory()` reste pour les tests.
   **Fichier** : `crates/nexus-coordinator-rs/src/db.rs`

2. **DaemonHttpState integration** — ajouter le champ
   `coordinator_db: Arc<Mutex<CoordinatorDb>>` dans DaemonHttpState.
   Initialiser au boot du daemon (`main.rs`) avec
   `CoordinatorDb::open(data_dir.join("coordinator.db"))`.
   **Fichiers** : `crates/nexus-shell-daemon/src/http.rs`,
   `crates/nexus-shell-daemon/src/main.rs`

3. **Refactorer coordinator_submit_task** — remplacer
   `CoordinatorDb::open_in_memory()` par `state.coordinator_db`
   (lock + clone ref). Le dispatcher utilise la DB partagee.
   **Fichier** : `crates/nexus-shell-daemon/src/http.rs`

4. **P2-A-2 PATTERNS.md update** — ajouter lecon "aggressive
   update" dans `docs/rust/PATTERNS.md` tech debt section.
   **Fichier** : `docs/rust/PATTERNS.md`

5. **HARDENING_ROADMAP last_validated** — mettre a jour le
   last_validated a S35/S36 avec compteurs actualises.
   **Fichier** : `docs/security/HARDENING_ROADMAP.md`

### Tests attendus

- `open_file_creates_db` : CoordinatorDb::open(tempdir) cree le
  fichier et retourne schema_version = 1
- `open_file_wal_mode` : PRAGMA journal_mode retourne "wal"
- `shared_db_dispatcher_persists` : two sequential submits via
  shared Arc<Mutex<CoordinatorDb>> voient les memes task_ids
- Tests existants (21 coordinator-rs) restent verts

### Delta tests prevu

+3 Rust (open_file, wal_mode, shared_db)

### Commit

```
feat(sprint36): Sprint 36 Phase A — dette pair + DaemonHttpState persistent CoordinatorDb
```

---

## §2 Phase B — Result submission endpoint + validator wire

### Scope

1. **Endpoint POST /api/v1/results/submit** — nouveau endpoint dans
   `authed_routes`. Le handler :
   - Deserialise `ResultEntry` depuis le body JSON
   - Lock la DB partagee, cree un `ResultValidator::new(db)`
   - Appelle `validator.validate(&entry)`
   - Retourne le verdict (200 Accepted, 400 Rejected*)
   **Fichier** : `crates/nexus-shell-daemon/src/http.rs`

2. **Integration test submit→validate** — test qui enchaine :
   - POST /api/v1/tasks/submit → TaskEntry
   - Construire ResultEntry signe par un worker keypair
   - POST /api/v1/results/submit → Accepted
   - Verifier que la tache est Completed dans la DB
   **Fichier** : `crates/nexus-shell-daemon/src/http.rs` (tests mod)

3. **ResultEntry deserialization** — verifier que les types
   `ResultEntry` et `ResultPayload` derivent `Deserialize` pour
   l'endpoint. Ajouter si absent.
   **Fichier** : `crates/nexus-core-rs/src/task.rs` (si necessaire)

### Tests attendus

- `result_submit_accepts_valid` : task submitted → result submitted
  → Accepted + task Completed
- `result_submit_rejects_bad_signature` : tampered ResultEntry →
  400 + RejectedBadSignature
- `result_submit_rejects_unknown_task` : result for nonexistent
  task → 400 + RejectedTaskNotFound
- `result_submit_rejects_completed` : result for already-completed
  task → 400 + RejectedTaskNotPending

### Delta tests prevu

+4 Rust (integration tests dans http.rs)

### Commit

```
feat(sprint36): Sprint 36 Phase B — result submission endpoint + validator wire DaemonHttpState
```

---

## §3 Phase C — KudosLedger Rust natif + wire post-validation

### Scope

1. **kudos_ledger.rs** — nouveau module dans nexus-coordinator-rs :
   - `KudosLedger` struct wrapping `&CoordinatorDb`
   - `credit(project_id, worker_node_id, tokens_generated, task_id)`
     insere une row dans la table kudos
   - `get_project_kudos(project_id)` retourne le total + liste
     contributeurs
   - `get_worker_kudos(worker_node_id)` retourne le total du worker
   **Fichier** : `crates/nexus-coordinator-rs/src/kudos_ledger.rs`

2. **Wire validator → kudos** — dans le handler result_submit,
   apres `Accepted`, appeler `KudosLedger::credit()` avec les
   infos du ResultEntry.
   **Fichier** : `crates/nexus-shell-daemon/src/http.rs`

3. **Endpoint GET /api/v1/kudos/{project_id}** — endpoint lecture
   dans `authed_routes`. Retourne JSON avec total + contributeurs.
   **Fichier** : `crates/nexus-shell-daemon/src/http.rs`

4. **Tests E2E** — submit task → submit result → verify kudos
   credited via GET endpoint.
   **Fichier** : `crates/nexus-shell-daemon/src/http.rs` (tests)

### Tests attendus

- `credit_increases_total` : credit(10) + credit(20) → total = 30
- `get_project_kudos_empty` : projet inconnu → total = 0
- `get_worker_kudos` : credit sous 2 projets → total worker correct
- `e2e_task_result_kudos` : task submit → result accept → GET kudos
  → total > 0
- `kudos_endpoint_returns_json` : GET /api/v1/kudos/{id} → 200 JSON

### Delta tests prevu

+5 Rust (3 kudos_ledger unit + 2 integration http)

### Commit

```
feat(sprint36): Sprint 36 Phase C — KudosLedger Rust natif + wire post-validation + endpoint
```

---

## §4 Phase D — Wrap-up

### Scope

1. verification.md fail-fast 28+ rows
2. sprint37_audit_plan.md (6 tracks A-F)
3. SPRINT_LOG.md row S36
4. CLAUDE.md etat actuel (compteurs, carries, etc.)
5. HARDENING_ROADMAP.md compteurs finaux + last_validated S36
6. Migration `.planning/active/sprint36_*.md` → `.planning/archive/v1.2/`
7. Memory nexus_grid_pivot.md update tip + compteurs

### Commit

```
chore(sprint36): Phase D — wrap-up + verification + audit plan S37 + migration
```

---

## §5 Fail-fast checklist (preview)

| # | Check | Critere |
|---|---|---|
| 1 | Rust compile workspace | 0 errors |
| 2 | Rust nextest pass | 930+ pass, 0 fail |
| 3 | Rust doctests pass | 0 fail |
| 4 | Rust clippy clean | 0 warnings |
| 5 | Rust fmt clean | no output |
| 6 | Release build daemon | Finished |
| 7 | Python ruff format | clean |
| 8 | Python ruff check | pass |
| 9 | SDK 195 pass | 195 pass |
| 10 | Coord 406+ pass | 406+ pass + ~36 fail stale |
| 11 | Gov 46 pass | 46 pass |
| 12 | Frontend lint | 0 errors |
| 13 | Frontend tsc | clean |
| 14 | Vitest 267+ pass | 267+ pass |
| 15 | Frontend build | success |
| 16 | size-limit 7/7 | 7/7 pass |
| 17 | Playwright | 42+ pass (no frontend change) |
| 18 | en-strings | clean |
| 19 | FORMAT_VERSION v1 | all = 1 |
| 20 | HARDENING compteurs | updated S36 |
| 21 | Planning docs | complets |
| 22 | CoordinatorDb file persistent | open(path) creates file |
| 23 | WAL mode active | PRAGMA journal_mode = wal |
| 24 | DaemonHttpState shared DB | dispatcher uses shared DB |
| 25 | Result submit accepted | valid result → Accepted |
| 26 | Result submit rejected | bad sig → Rejected |
| 27 | KudosLedger credit | credit() increments total |
| 28 | Kudos endpoint | GET /api/v1/kudos/{id} → JSON |
| 29 | E2E task → result → kudos | full pipeline works |
| 30 | No Python code modified | 0 diff packages/ |
| 31 | G8 systematique 3/3 | 3 preflights + 3 reviews |

---

## §6 Research consultee

Technologies toutes deja dans le workspace :
- axum 0.8 : state management via `State<Arc<T>>`, pattern
  DaemonHttpState existant avec 15+ champs
- rusqlite 0.36 : WAL mode via `PRAGMA journal_mode=WAL`, deja
  utilise dans nexus-worker-core pour allowlist.db
- tokio 1.x : runtime async, deja dans daemon
- nexus-core-rs types : ResultEntry, ResultPayload, TaskEntry,
  Task — deja derives Serialize, verifier Deserialize

Pas de nouvelle dependance introduite.
