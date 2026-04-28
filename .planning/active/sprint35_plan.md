# Sprint 35 — Plan (migration Rust native — Phase 1)

**Decisions gelees** : D1..D5 dans `sprint35_kickoff.md`.
**Version** : v1.2 continue (hardening + migration).

---

## §1 Phases

### Phase A — MANDATORY 3/3 + fondation crate coordinator-rs

**Objectif** : fermer les 3 carries MANDATORY + poser le squelette
du nouveau crate coordinator Rust.

#### A.1 shellcheck CI (P2-B-1-S33 3/3)

Creer `.github/workflows/shellcheck.yml` :
- Trigger : push + PR sur `scripts/**/*.sh`
- Job : ubuntu-latest, `apt install shellcheck`, run shellcheck
  sur tous les `.sh` dans `scripts/`
- Exclure les scripts tiers (si existants)

#### A.2 cross-daemon E2E (P2-C-1-S33 3/3)

Nouveau test dans `crates/nexus-test-harness/tests/` :
- Spawn `DaemonCluster::spawn(2)` (2 daemons)
- Daemon A : `POST /publish-blob` avec un zip minimal
- Daemon B : `GET /blob-serve/{hash}/index.html` via iroh-blobs
  cross-fetch (le hash du blob publie sur A doit etre fetchable
  depuis B apres sync)
- Assert : status 200 + body contient le contenu du zip

Prerequis : verifier que `DaemonCluster` connecte les 2 noeud via
iroh relay (sinon ajouter un `connect_peers()` helper).

#### A.3 REPO_URL (P2-B-2-S33 3/3)

`scripts/install-node.sh` ligne `REPO_URL=...` : remplacer le
placeholder par un `TODO(v1.0)` comment explicite avec la
justification blocker externe.

#### A.4 Fondation crate coordinator-rs

Creer `crates/nexus-coordinator-rs/` :
- `Cargo.toml` : deps nexus-core-rs, rusqlite, tokio, serde,
  thiserror, tracing
- `src/lib.rs` : re-exports modules
- `src/types.rs` : `TaskSubmission`, `ValidationResult`,
  `KudosEntry` (structs Rust natifs, serde compatible JSON)
- `src/db.rs` : `CoordinatorDb` struct + schema init
  (tables `tasks`, `kudos_entries`) + `open()` + `migrate()`
- `src/error.rs` : `CoordinatorError` enum
- Tests : schema init + roundtrip insert/select

Ajouter `nexus-coordinator-rs` au workspace `Cargo.toml`.

**Commit** : `feat(sprint35): Sprint 35 Phase A — MANDATORY 3/3 +
crate nexus-coordinator-rs fondation`

---

### Phase B — Dispatcher Rust natif

**Objectif** : le dispatcher de taches vit en Rust et signe/soumet
les taches nativement.

#### B.1 TaskDispatcher struct

`crates/nexus-coordinator-rs/src/dispatcher.rs` :
- `TaskDispatcher` struct holding `CoordinatorDb` + `KeyPair` +
  iroh `Doc` handle
- `submit(&self, submission: TaskSubmission) -> Result<TaskEntry>`
  1. Valide les champs (model, prompt non-vide, etc.)
  2. Construit `Task` avec `canonical_bytes` + `sign_task`
  3. Insert dans la DB locale (status = pending)
  4. Insert dans le iroh doc (gossip broadcast)
  5. Return `TaskEntry` avec hash + signature

#### B.2 Endpoint axum

`crates/nexus-shell-daemon/src/http.rs` :
- `POST /api/v1/tasks/submit` route
- Extracts `TaskSubmission` from JSON body
- Calls `dispatcher.submit()`
- Returns `TaskEntry` JSON
- Auth : bearer token (meme middleware que les routes existantes)

Note : le prefix `/api/v1/` distingue les endpoints Rust des
endpoints Python (`/api/` sans version ou via coordinator proxy).

#### B.3 Tests

- Unit : `dispatcher.submit()` avec mock iroh doc
- Integration : via `DaemonCluster::spawn(1)` + HTTP POST
- Verification que `Task` signe est identique au format Python
  (canonical bytes JCS, meme domain separation)

**Commit** : `feat(sprint35): Sprint 35 Phase B — Dispatcher Rust
natif task submission pipeline`

---

### Phase C — Validator Rust natif

**Objectif** : le validateur de resultats tourne en Rust avec un
subscription loop tokio.

#### C.1 ResultValidator struct

`crates/nexus-coordinator-rs/src/validator.rs` :
- `ResultValidator` struct holding `CoordinatorDb` + `KeyPair` +
  `KudosConfig`
- `validate_result(&self, result: &ResultEntry) -> Result<ValidationOutcome>`
  1. Verify signature (`verify_result_entry`)
  2. Verify digest matches (blake3 content hash)
  3. Check task exists in DB + status pending
  4. If valid : update task status = completed, trigger kudos credit
  5. If invalid : update task status = rejected, log reason

#### C.2 Validation loop

`crates/nexus-coordinator-rs/src/validator_loop.rs` :
- `spawn_validation_loop(doc: Doc, validator: Arc<ResultValidator>)`
- Subscribe aux iroh `LiveEvent::ContentReady` sur le doc
- Pour chaque event : parse `ResultEntry`, appeler `validate_result()`
- Tokio select! avec cancellation token pour shutdown propre

Wire dans `nexus-shell-daemon/src/runtime.rs` : spawn la validation
loop au demarrage si le mode coordinator est actif.

#### C.3 Tests

- Unit : `validate_result()` avec results valides et invalides
- Unit : validation loop avec mock events (cancel apres N events)
- Integration : submit task via Phase B endpoint → publish result
  via gossip → assert task status = completed dans la DB

**Commit** : `feat(sprint35): Sprint 35 Phase C — Validator Rust
natif result verification loop`

---

### Phase D — Wrap-up

- verification.md fail-fast checklist 30+ rows
- sprint36_audit_plan.md (tracks A-F couvrant les 3 phases feat)
- SPRINT_LOG.md row S35
- CLAUDE.md etat actuel (tip, compteurs)
- HARDENING_ROADMAP.md last_validated S35
- Migration .planning/active/ → .planning/archive/v1.2/
- Memory update nexus_grid_pivot.md

**Commit** : `chore(sprint35): Phase D — wrap-up + verification +
audit plan S36 + migration`

---

## §2 Fichiers touches (prevision)

| Phase | Fichiers |
|---|---|
| A | `.github/workflows/shellcheck.yml` (NEW), `crates/nexus-test-harness/tests/cross_daemon_blob.rs` (NEW), `scripts/install-node.sh` (TODO comment), `crates/nexus-coordinator-rs/` (NEW crate), `Cargo.toml` (workspace member) |
| B | `crates/nexus-coordinator-rs/src/dispatcher.rs` (NEW), `crates/nexus-shell-daemon/src/http.rs` (+route), `crates/nexus-shell-daemon/Cargo.toml` (+dep coordinator-rs) |
| C | `crates/nexus-coordinator-rs/src/validator.rs` (NEW), `crates/nexus-coordinator-rs/src/validator_loop.rs` (NEW), `crates/nexus-shell-daemon/src/runtime.rs` (spawn loop) |
| D | `.planning/active/` (5 docs), SPRINT_LOG.md, CLAUDE.md, HARDENING_ROADMAP.md |

**Scope** : nouveau crate Rust + tests + CI YAML. 0 modification Python.

---

## §3 Dependencies entre phases

```
A (fondation + MANDATORY) → B (dispatcher utilise types A.4)
                           → C (validator utilise types A.4 + DB A.4)
B (dispatcher) → C (validator teste le submit endpoint de B)
```

Phases B et C sont sequentielles : C depend du dispatcher B pour
ses tests d'integration.

---

## §4 Criteres d'acceptation par phase

| Phase | Critere |
|---|---|
| A | shellcheck CI green sur `scripts/*.sh` + cross-daemon E2E test pass + coordinator-rs crate compile + DB schema roundtrip test pass |
| B | `POST /api/v1/tasks/submit` retourne TaskEntry signe + Task canonical bytes identique au format Python + integration test pass via DaemonCluster |
| C | Validation loop spawne au demarrage daemon + result valide → task completed dans DB + result invalide → task rejected + cancellation propre |
| D | 30+ rows fail-fast verts + 5 docs planning complets + migration archive |

---

## §5 Fail-fast checklist (prevision)

| # | Check | Critere |
|---|---|---|
| 1 | Rust compile workspace | 0 errors |
| 2 | Rust nextest pass | 905+ pass (902 + ~3 nouveaux minimum) |
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
| 17 | Playwright | 42+ pass |
| 18 | en-strings | clean |
| 19 | FORMAT_VERSION v1 | all = 1 |
| 20 | HARDENING compteurs | updated S35 |
| 21 | Planning docs | complets |
| 22 | shellcheck CI workflow | exists + valid YAML |
| 23 | cross-daemon E2E test | pass dans nextest |
| 24 | REPO_URL documented | TODO(v1.0) in install-node.sh |
| 25 | coordinator-rs crate | compiles, tests pass |
| 26 | dispatcher submit endpoint | POST /api/v1/tasks/submit 200 |
| 27 | Task canonical bytes | identical to Python format |
| 28 | validator loop spawns | daemon boots with coordinator mode |
| 29 | validator accepts valid | task status = completed |
| 30 | validator rejects invalid | task status = rejected |
| 31 | no Python code modified | 0 diff packages/ |
