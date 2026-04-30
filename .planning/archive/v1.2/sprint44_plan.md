# Sprint 44 — Plan d'execution detaille

**Ref** : `sprint44_kickoff.md` D1..D3 gelees.
**Tip d'entree** : `358c6ff`.
**Critere SMART** : 28+ rows fail-fast verts.

---

## Phase A — Dette pair (7 MANDATORY 3/3 + prefix contributor)

### A.1 PATTERNS.md §P42 — ChainResult mutations contract

**Fichier** : `docs/rust/PATTERNS.md`
**Action** : Ajouter section §P42 documentant le contrat
`ChainResult.mutations: Vec<(String, String)>` (pairs reason,
replacement). Aucun guardrail n'emet Mutation aujourd'hui —
documenter le contrat pour le premier consumer post-v1.0.
**Verification** : section §P42 presente, lisible.

### A.2 PATTERNS.md §P43 — pow_keypair identity

**Fichier** : `docs/rust/PATTERNS.md`
**Action** : Ajouter section §P43 documentant l'equivalence
pow_keypair = iroh node identity = provenance signer = Python
`coordinator.keypair`. Reste valide jusqu'a S45 (suppression
Python).
**Verification** : section §P43 presente, lisible.

### A.3 babel-scraper .gitignore

**Fichier** : `.gitignore`
**Action** : Ajouter `tools/babel-scraper/` au .gitignore.
L'outil est post-v1.0 (cf. memory `babel_post_v1_app.md`),
les scripts telechargeront des corpus volumineux.
**Verification** : `git status --short` ne montre plus
`tools/babel-scraper/`.

### A.4 list_apps pagination + doc aggregate

**Fichiers** : `crates/nexus-shell-daemon/src/apps.rs`
**Action** :
- Ajouter `limit: Option<usize>` (defaut 50, max 500) et
  `offset: Option<usize>` (defaut 0) a `AppListQuery`
- Ajouter `total_count: usize` a `AppListResponse`
- Appliquer `skip(offset).take(limit)` sur le Vec resultat
- Test unitaire pagination (limit=2, offset=1)
**Verification** : cargo nextest -p nexus-shell-daemon.

### A.5 test RNG rate>1

**Fichier** : `crates/nexus-coordinator-rs/src/canary_input.rs`
**Action** : Ajouter test `injector_rate_probabilistic` qui
appelle `should_inject(rate=5)` 1000 fois et verifie
distribution 15-25% (large CI, pas strict).
**Verification** : cargo nextest -p nexus-coordinator-rs.

### A.6 Debug → as_str()

**Fichier** : `crates/nexus-shell-daemon-core/src/browse.rs`
**Action** :
- Ajouter `fn as_str(&self) -> &'static str` a BrowseStatus et
  BrowseSource
- Remplacer `format!("{:?}").to_lowercase()` par `self.as_str()`
  dans status_str()/source_str()
- Test existant couvre deja les valeurs
**Verification** : cargo nextest -p nexus-shell-daemon-core.

### A.7 Route prefix contributor → /api/v1/

**Fichier** : `crates/nexus-shell-daemon/src/http.rs`
**Action** : Renommer les 3 routes `/api/contributor/` →
`/api/v1/contributor/` pour coherence avec les autres routes
`/api/v1/`.
**Verification** : cargo nextest -p nexus-shell-daemon.

### Commit Phase A

```
feat(sprint44): Sprint 44 Phase A — dette pair 7 MANDATORY
  ChainResult+pow_keypair doc + babel gitignore + list_apps
  pagination + RNG test + Debug as_str + contributor prefix
```

---

## Phase B — Routes batch 1 : health + shell + kudos + diagnostic

### B.1 Health handler

**Fichier nouveau** : `crates/nexus-shell-daemon/src/health.rs`
**Routes** :
- GET /api/v1/health — coordinator health payload (project name,
  visibility, test counts, uptime)
- GET /api/v1/project — project metadata (name, doc_id, author_id)
- POST /api/v1/project/publish — publish projet via gossip
  (proxy interne vers /publish existant)
**Dep** : `DaemonHttpState` existant.

### B.2 Shell handler

**Fichier nouveau** : `crates/nexus-shell-daemon/src/shell.rs`
**Route** :
- GET /api/v1/shell/discover — enumerate coordinateurs running
  via fichiers registry dans le data dir.
**Dep** : lecture filesystem `~/.sbfb/registry/` ou equivalent.

### B.3 Kudos handler (completer)

**Fichier modifie** : `crates/nexus-shell-daemon/src/http.rs` +
  potentiellement `crates/nexus-coordinator-rs/src/kudos_ledger.rs`
**Routes** :
- GET /api/v1/kudos — list entries, filtre optionnel worker_pubkey
- GET /api/v1/kudos/{project_id}/leaderboard — top contributors
  par projet
**Dep** : `kudos_ledger.rs` existant, query SQL a ajouter dans
  `db.rs` si absent.

### B.4 Diagnostic handler

**Fichier nouveau** : `crates/nexus-shell-daemon/src/diagnostic_api.rs`
**Route** :
- GET /api/v1/diagnostic/fairness — retourne gini, top_5_pct,
  churn_rate, worker_count.
**Dep** : `fairness.rs` (compute_gini, compute_top_k_share,
  compute_churn_rate) deja porte S41.

### B.5 Enregistrement routes

**Fichier** : `crates/nexus-shell-daemon/src/http.rs` +
  `crates/nexus-shell-daemon/src/main.rs`
**Action** : ajouter les mod + routes au router.

### Tests Phase B

- Tests unitaires par handler (~3-5 par handler)
- Au moins 1 test integration HTTP par route nouvelle

### Commit Phase B

```
feat(sprint44): Sprint 44 Phase B — health + shell + kudos +
  diagnostic API Rust
```

---

## Phase C — Routes batch 2 : tasks + worker_state

### C.1 Tasks handler (completer)

**Fichier nouveau** : `crates/nexus-shell-daemon/src/tasks_api.rs`
**Routes** :
- GET /api/v1/tasks — list tasks filtre par state (pending/done),
  limit optionnel
- GET /api/v1/tasks/{task_id} — detail d'un task par ID
**Dep** : `db.rs` necessite `list_tasks(state, limit)` et
  `get_task(task_id)` queries. `dispatcher.rs` existant.

### C.2 Worker state handler

**Fichier nouveau** : `crates/nexus-shell-daemon/src/worker_state_api.rs`
**Route** :
- GET /api/v1/worker/state — proxy `~/.sbfb/worker/state.json`
  avec staleness check (>15s = stale). Schema `WorkerStateV1`.
**Dep** : lecture filesystem, serde_json parse, chrono pour
  staleness.

### C.3 Enregistrement routes + tests

**Action** : mod + routes au router, tests unitaires + integration.

### Commit Phase C

```
feat(sprint44): Sprint 44 Phase C — tasks + worker_state API Rust
```

---

## Phase D — Wrap-up

- verification.md (fail-fast checklist 28+ rows)
- sprint45_audit_plan.md
- Update compteurs HARDENING_ROADMAP, CLAUDE.md, SPRINT_LOG.md
- Update memory nexus_grid_pivot.md §Tip

### Commit Phase D

```
chore(sprint44): Phase D — wrap-up + verification + audit plan S45
  + counters
```

---

## Fail-fast checklist (28+ rows cible)

| # | Check |
|---|---|
| 1 | cargo fmt --all --check |
| 2 | cargo clippy --workspace --all-targets -- -D warnings |
| 3 | cargo nextest run --workspace |
| 4 | cargo test --workspace --doc |
| 5 | cargo build -p nexus-shell-daemon --release |
| 6 | uv run ruff format --check packages/ |
| 7 | uv run ruff check packages/ |
| 8 | uv run pytest packages/nexus-sdk/tests/ -q |
| 9 | uv run pytest packages/nexus-coordinator/tests/ -q |
| 10 | uv run pytest packages/nexus-app-gov/tests/ -q |
| 11 | npm run lint (web/) |
| 12 | npx tsc --noEmit -p tsconfig.app.json |
| 13 | npm run test:unit (web/) |
| 14 | npm run build (web/) |
| 15 | npm run size (web/) |
| 16 | Phase A preflight G8 |
| 17 | Phase A review |
| 18 | Phase B preflight G8 |
| 19 | Phase B review |
| 20 | Phase C preflight G8 |
| 21 | Phase C review |
| 22 | 7/7 MANDATORY items resolus |
| 23 | Health handler porte |
| 24 | Shell handler porte |
| 25 | Kudos list+leaderboard porte |
| 26 | Diagnostic fairness porte |
| 27 | Tasks list+get portes |
| 28 | Worker state porte |
| 29 | Scope cuts respectes |
| 30 | Delta tests Phase A+B+C |
