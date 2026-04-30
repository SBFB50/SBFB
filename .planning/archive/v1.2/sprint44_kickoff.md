# Sprint 44 — Kickoff (dette pair MANDATORY 7 items + Tier 5 routes API fin)

**Ecrit** : 2026-04-30 (post-audit gate S43 PASS `358c6ff`).
**Type** : **sprint pair** — phase dette obligatoire
(§6.2.1 Regle 1). 7 items a 3/3 MANDATORY (§6.2.1 Regle 2).
**Tip master d'entree** : `358c6ff`.
**Phase 0 audit Sprint 43** : **DEJA JOUE** — `358c6ff` PASS.

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-04-30 (S43 meme jour).
  0 trigger actif. Pas de pre-research.

- **Technologies S44** : axum (deja dep daemon), serde_json (deja
  dep), rusqlite (deja dep), tokio (deja dep). Pas de nouvelle
  dep externe.

- **Roadmap** : `.planning/roadmap_v1_migration_rust.md` §S44
  "routes restantes (~700 LOC, 7 fichiers : health, shell, tasks,
  kudos, events, diagnostic, worker_state)".

- **ROADMAP_COMMITMENTS** : aucun declenche (tous requierent tag
  v1.0, pas encore pose).

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 43 CLOSED + audit PASS. Tier 5 routes API complet S42+S43 :
8/8 routes portees (deploy 679 + apps 275 + consent 230 + files 190
+ canary_api 100 + contributor_api 140). Prochaine etape per
roadmap : porter les routes API restantes (health, shell, tasks,
kudos, diagnostic, worker_state) + resoudre les 7 items MANDATORY.

Analyse des routes Python restantes (agents Explore) :

| Fichier | LOC Python | Routes | Etat |
|---|---|---|---|
| health.py | 66 | 3 (health, project, project/publish) | Non porte |
| shell.py | 56 | 1 (shell/discover) | Non porte |
| tasks.py | 140 | 2 (submit partial, list) | Partiellement porte (submit S35) |
| kudos.py | 54 | 2 (list, verify) | Partiellement porte (get+verify S36) |
| events.py | 195 | 2 (SSE stream, emit) | Non porte, dep AppEvents bus SDK |
| diagnostic.py | 93 | 1 (fairness) | Non porte, fairness.rs deja en Rust |
| worker_state.py | 137 | 1 (worker state proxy) | Non porte |

Total hors events.py : 546 LOC Python → ~400-500 LOC Rust estime.

### §1.2 Compteurs tests entree (tip `358c6ff`)

| Suite | Count |
|---|---|
| Rust nextest | 1111 |
| Rust doctests | 6 passed, 1 ignored |
| SDK pytest | 195 |
| Coord pytest | 409 + 36 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 267 |
| Playwright | 42 + 2 fail (env pre-existant) |
| size-limit | 7/7 |
| **Total** | **~2114** |

**Note** : test `probe_and_cache_with_quorum_majority_continues_to_dial`
observe flaky au pre-flight (reseau-dependant, non reproductible
systematique, passe 1111/1111 en S43 verification).

---

## §2 Goal en une phrase

Le sprint **resout les 7 items MANDATORY 3/3** (ChainResult doc,
pow_keypair doc, babel-scraper .gitignore, list_apps pagination,
RNG rate>1 test, Debug→as_str, pagination limit/offset) et
**termine la migration Tier 5** en portant les 6 routes API Python
restantes (health, shell, tasks, kudos, diagnostic, worker_state)
vers des handlers axum Rust. events.py scope-cut S45 (dep SDK).
**Critere SMART : 28+ rows fail-fast verts au verification.md.**

---

## §3 Phase 0 — Audit gate Sprint 43

**DONE** — `358c6ff`. Verdict PASS. 0 P0/P1, 1 P2 (gap integration
test 12 routes), 2 P3 (silent null canary_api, hex case-sensitivity).

---

## §4 Decisions Day 0 (D1..D3 gelees)

### D1 — MANDATORY batch : 7 items 3/3 resolus Phase A

**Retenu** : resoudre les 7 items MANDATORY dans la phase dette
obligatoire (Phase A) :

(a) **P2-REVIEW-A-1-S42 ChainResult mutations target** (3/3) :
    Documenter le contrat mutations dans `PATTERNS.md` §P42.
    `guardrails.rs` `ChainResult.mutations: Vec<(String, String)>`
    = pairs (reason, replacement). Aucun guardrail n'emet Mutation
    aujourd'hui — documenter le contrat pour le premier consumer
    post-v1.0. ~15-20 LOC doc.

(b) **P2-REVIEW-B-1-S42 pow_keypair identity doc** (3/3) :
    Documenter pow_keypair = iroh node identity = provenance signer
    dans `PATTERNS.md` §P43. Equivalence Python
    `coordinator.keypair`. ~20-25 LOC doc.

(c) **P3-REVIEW-A-2-S42 babel-scraper untracked** (3/3) :
    `tools/babel-scraper/` (709 LOC) est un outil post-v1.0
    (cf. memory `babel_post_v1_app.md`). Ajouter
    `tools/babel-scraper/` a `.gitignore` — pas commit car les
    scripts telechargeront des corpus volumineux et l'outil est
    hors-scope pre-v1.0. ~1 LOC.

(d) **P3-REVIEW-C-1-S42 list_apps aggregate probe** (3/3) :
    `apps.rs:107-139` appelle `aggregate()` a chaque requete.
    Cache TTL 60s amortit le cout. Fix : documenter le
    comportement dans un commentaire inline + ajouter `limit`
    query param pour ne pas retourner toutes les apps. ~30-40 LOC.

(e) **P3-AUDIT-A-1-S42 couverture RNG rate>1** (3/3) :
    Ajouter test `injector_rate_probabilistic` qui appelle
    `should_inject(rate=5)` 1000 fois et verifie 15-25% hits
    (intervalle de confiance large). ~10-15 LOC test.

(f) **P3-AUDIT-C-1-S42 Debug vs serde** (3/3) :
    `browse.rs` `BrowseStatus`/`BrowseSource` utilisent
    `format!("{:?}").to_lowercase()`. Remplacer par methode
    `as_str()` sur chaque enum. ~20-25 LOC.

(g) **P3-AUDIT-C-2-S42 pagination limit/offset** (3/3) :
    `apps.rs` `AppListQuery` n'a pas `limit/offset`. Ajouter
    `limit: Option<usize>` (defaut 50, max 500) +
    `offset: Option<usize>` (defaut 0) + `total_count` dans la
    reponse. ~25-35 LOC.

**Rejete** :
- Reporter encore : impossible, §6.2.1 Regle 2 (3+ reports).
- babel-scraper commit : hors-scope pre-v1.0, scripts de scraping
  avec donnees volumineuses ne conviennent pas au repo.

### D2 — Tier 5 routes API fin : 6 fichiers (hors events.py)

**Retenu** : porter les 6 routes API Python restantes vers des
handlers axum natifs dans `crates/nexus-shell-daemon/src/`. Pattern
identique S42-S43 (State extractor + Json + coordinator_db).

(a) **health.py** (66 LOC) : 3 routes (health payload, project
    metadata, project/publish proxy). Le GET /health existe deja
    (liveness probe basique) — enrichir avec health_payload
    coordinator-side.
(b) **shell.py** (56 LOC) : 1 route (discover coordinateurs
    running via fichiers registry).
(c) **tasks.py** (140 LOC, partiel) : 2 routes restantes (GET
    /tasks list + GET /tasks/{id}). submit deja porte S35.
    Necessite `list_tasks()` query dans `db.rs`.
(d) **kudos.py** (54 LOC, partiel) : 2 routes restantes
    (list entries + leaderboard per project). get+verify deja
    portes S36.
(e) **diagnostic.py** (93 LOC) : 1 route (fairness metrics).
    `fairness.rs` (gini, top_k, churn) deja porte S41, juste
    le handler HTTP a wire.
(f) **worker_state.py** (137 LOC) : 1 route (proxy state.json
    worker avec staleness check 15s).

**Rejete** :
- Porter events.py (195 LOC SSE) : depend AppEvents bus
  (`nexus_sdk` Python). Bus inexistant en Rust. SSE streaming
  complexe (heartbeat, cleanup, cancellation). Scope cut S45.
- Porter quarantine.py : non liste dans la roadmap S44. S45.
- Tout porter en une phase : trop de surface, split en 2 phases.

### D3 — Scope cuts S44

1. **events.py SSE streaming** — S45 (dep AppEvents bus SDK Python)
2. **quarantine.py API routes** — S45 (hors roadmap S44)
3. **Suppression coordinator Python** — S45
4. **CI/VPS/v1.0** — S46-48
5. **Kudos debit/stake** — interdit (Day 0 #7)
6. **P2-AUDIT-A-1-S43 integration test gap complet** — partiel
   S44 (nouvelles routes), complet S45

---

**Acknowledged review findings (G1)** :

Scoring : D1 ✅, D2 ✅, D3 ✅.
Rigor signal G4 satisfait (3 blind spots documentes, 0 ⚠️/❌).

Blind spots signales par le reviewer independant :
- D2(a) health.py payload contract : a verifier en G8 preflight
  Phase B — coordinator.health_payload() specification.
- D2(c) tasks.py list_tasks() SQL : a verifier en G8 preflight
  Phase C — schema db.rs et query faisabilite.
- D2(e) diagnostic.py fairness.rs wire : a verifier en G8
  preflight Phase B — signature fonctions fairness.rs.

Decision : les 3 points seront adresses dans les G8 preflights
respectifs (S1b deps scan + S2 historical decisions). Pas de
modification du plan — les preflights sont systematiques.

---

## §5 Plan Phase outline A..D

### Phase A — Dette pair (7 MANDATORY 3/3)

**But** : resoudre les 7 items MANDATORY.
- (a-b) PATTERNS.md §P42 ChainResult + §P43 pow_keypair
- (c) .gitignore babel-scraper
- (d) list_apps limit query param + doc aggregate
- (e) test injector_rate_probabilistic
- (f) BrowseStatus/BrowseSource as_str()
- (g) AppListQuery limit/offset + total_count
- Absorbe P3-REVIEW-C-1-S43 prefix route /api/contributor/ →
  /api/v1/contributor/ (normalisation, ~5 LOC)
- Commit : `feat(sprint44): Sprint 44 Phase A — dette pair 7
  MANDATORY ChainResult+pow_keypair doc + babel gitignore +
  list_apps pagination + RNG test + Debug as_str + contributor
  prefix`

### Phase B — Routes batch 1 : health + shell + kudos + diagnostic

**But** : porter les 4 routes les plus simples.
- health handler : enrichir liveness avec coordinator state
- shell handler : discover coordinateurs running
- kudos handler : list entries + leaderboard (completer S36)
- diagnostic handler : fairness metrics (wire fairness.rs)
- Tests unitaires + integration HTTP par handler
- Commit : `feat(sprint44): Sprint 44 Phase B — health + shell +
  kudos + diagnostic API Rust`

### Phase C — Routes batch 2 : tasks + worker_state

**But** : porter les 2 routes plus complexes.
- tasks handler : list_tasks SQL query + GET /tasks/{id}
  + list_tasks() dans db.rs
- worker_state handler : lecture state.json + staleness check
- Tests unitaires + integration HTTP par handler
- Commit : `feat(sprint44): Sprint 44 Phase C — tasks + worker_state
  API Rust`

### Phase D — Wrap-up

---

## §6 Items carry/dette

### Resolus S44 (plan)

- [plan] P2-REVIEW-A-1-S42 ChainResult mutations target : Phase A
- [plan] P2-REVIEW-B-1-S42 pow_keypair identity doc : Phase A
- [plan] P3-REVIEW-A-2-S42 babel-scraper untracked : Phase A
- [plan] P3-REVIEW-C-1-S42 list_apps aggregate probe : Phase A
- [plan] P3-AUDIT-A-1-S42 couverture RNG rate>1 : Phase A
- [plan] P3-AUDIT-C-1-S42 Debug vs serde : Phase A
- [plan] P3-AUDIT-C-2-S42 pagination limit/offset : Phase A
- [plan] P3-REVIEW-C-1-S43 prefix route contributor : Phase A

### Carries confirmes S45

- [carry] P2-A-1 rand blocker upstream 10+/3 : exemption blocker
  externe
- [carry] P2-AUDIT-2 transitives iroh : herite pin 0.98
- [carry] P2-REVIEW-C-1-S40 SHA-256 vs BLAKE3 6/3 : exemption
  dependance sequentielle S45 (Python wire parite)
- [carry] P2-REVIEW-B-1-S43 coord dead_code cleanup 2/3
- [carry] P2-AUDIT-A-1-S43 integration test gap 12 routes 2/3
  (partiel S44 via tests nouvelles routes)
- [carry] P3-REVIEW-A-1-S43 TOCTOU canary reload 2/3
- [carry] P3-AUDIT-A-2-S43 silent null canary_api 2/3
- [carry] P3-AUDIT-A-3-S43 hex case-sensitivity 2/3

---

## §7 Scope cuts

1. events.py SSE streaming — S45 (dep AppEvents bus SDK Python)
2. quarantine.py API routes — S45 (hors roadmap S44)
3. Suppression Python — S45
4. CI/VPS/v1.0 — S46-48
5. Kudos debit/stake — interdit (Day 0 #7)
6. Integration test gap complet — partiel S44, complet S45

---

## §8 Risk register

| # | Risque | Impact | Mitigation |
|---|---|---|---|
| R1 | 7 MANDATORY items touchent 4 fichiers + 2 docs | Low | Tous < 80 LOC, pattern connu |
| R2 | tasks.py list_tasks SQL query complexe | Medium | Pattern db.rs existant, SQLite basique |
| R3 | worker_state.py lit filesystem (state.json) | Low | Lecture seule, pas de lock |
| R4 | Test flaky probe_and_cache browse (observe pre-flight) | Low | Non reproductible, reseau-dependant |

---

## §9 Checkpoint de validation

1. **D1** : 7 items MANDATORY faisables Phase A ? → oui, ~150-175
   LOC total, tous localises
2. **D2** : 6 routes en 2 phases ? → oui, ~400-500 LOC total,
   pattern S42-S43 etabli
3. **D3** : scope cuts coherents roadmap ? → oui, events.py S45
   per dep SDK, S45 = suppression Python per roadmap
