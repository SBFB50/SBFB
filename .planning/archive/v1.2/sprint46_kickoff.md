# Sprint 46 — Kickoff (integration tests MANDATORY + dette pair S44 + frontend direct-daemon)

**Ecrit** : 2026-04-30 (post-audit gate S45 PASS `72f4083`).
**Type** : **sprint pair** — phase dette obligatoire
(§6.2.1 Regle 1).
**Tip master d'entree** : `d1dd4bd`.
**Phase 0 audit Sprint 45** : **DEJA JOUE** — `72f4083` PASS
(0 P0, 0 P1, 1 P2 carry confirme, 1 P3 nouveau).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-04-30 (S45 meme jour).
  0 trigger actif. 3 triggers surveilles : iroh > 0.98, wasmtime
  LTS bump, arti-client > 0.41 — aucun realise. Pas de
  pre-research.

- **Technologies S46** : axum (deja dep daemon, oneshot tests),
  tower (deja dep, tower::ServiceExt::oneshot pour tests router).
  Pas de nouvelle dep externe.

- **ROADMAP_COMMITMENTS check** : LT-1 reclassifie pre-v1.0
  cible S50 — pas S46. LT-2..LT-5 latents (tous requierent tag
  v1.0 non pose). LT-6 RESOLVED. 0 condition declenchee.

- **HARDENING_ROADMAP §3** : pas de ligne S46 explicite. Pas de
  drift a documenter.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 45 CLOSED + audit PASS. Migration API routes coordinator
Python quasi-complete : S35-S45 ont porte 26+ routes vers handlers
axum Rust dans `crates/nexus-shell-daemon/src/`. S45 a supprime
14 fichiers routes Python + 12 fichiers tests + dead code Rust.
Net sprint S45 -5425 LOC.

Post-audit S45, 2 commits hors-sprint :
- `1f1a017` fix(shell): daemon sert le React shell via
  `--web-root` + endpoint public `GET /auth/token` (loopback
  Host+Origin checks). Le frontend peut fonctionner sans Vite
  proxy, directement depuis le daemon.
- `d1dd4bd` docs(architecture): LAUNCHER.md — vision daemon
  unique + frontends P2P (622 LOC doc).

**Etat actuel des tests d'integration HTTP (audit factuel)** :
54 routes daemon total. 21 ont des tests Router-level (oneshot).
**33 routes manquent de tests d'integration**, dont les 12
originales du carry MANDATORY P2-AUDIT-A-1-S43.

Routes sans integration tests par fichier :
- consent.rs : 4/4 manquantes (get, set, whitelist/add, whitelist/remove)
- files.rs : 3/3 (upload, manifest, stream)
- canary_api.rs : 3/3 (freshness, inject-rate, observed-divergence)
- contributor_api.rs : 2/3 (project, envelope — verify OK)
- deploy.rs : 2/2 (deploy, deploy-from-repo)
- apps.rs : 2/2 (list, get)
- invite_api.rs : 3/3 (create, list, revoke)
- quarantine_api.rs : 3/3 (list, flush, drop)
- kudos_api.rs : 2/2 (entries, leaderboard)
- tasks_api.rs : 2/2 (list, get)
- worker_state_api.rs : 1/1 (get)
- health_api.rs : 1/1 (coordinator_health)
- shell_api.rs : 1/1 (discover)
- diagnostic_api.rs : 1/1 (fairness)
- http.rs : 1/1 (auth_token)

### §1.2 Ancrage HARDENING_ROADMAP

Pas de ligne S46 prescrite. 0 trigger actif. Pas d'item drift
a documenter.

### §1.3 Compteurs tests entree (tip `d1dd4bd`)

| Suite | Count |
|---|---|
| Rust nextest | 1132 |
| Rust doctests | 6 passed, 1 ignored |
| SDK pytest | 195 |
| Coord pytest | 323 + 23 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 268 (+1 vs S45 : auth fallback test) |
| Playwright | 42 + 2 fail (env pre-existant) |
| size-limit | 7/7 |
| **Total** | **~1949** |

### §1.4 Pre-launch protocol policy (rappel)

`*_FORMAT_VERSION` restent a 1. S46 ne touche pas de wire format
canonical. Pas de tolerant decoder multi-version. Sprint pure
tests + frontend + dette — pas d'impact wire.

---

## §2 Goal

Fermer le gap d'integration tests MANDATORY P2-AUDIT-A-1-S43
(12 routes originales + enrichissement harness mk_state()),
absorber les 5 carries S44 a 2/3 via phase dette obligatoire
(empecher 5 MANDATORY S47), et migrer le frontend vers l'API
daemon directe (leveraging hotfix `1f1a017`). **Critere SMART :
28+ rows fail-fast verts au verification.md, mesure binaire
au Phase D wrap-up.**

---

## §3 Phase 0 — Audit gate S45

**DEJA JOUE** : commit `72f4083` PASS (0 P0, 0 P1). Audit
findings dans `.planning/active/sprint45_audit_findings.md`.
11 carries documentes pour S46 (cf. §6 ci-dessous).

---

## §4 Decisions Day 0 (D1..D4 gelees)

### D1 — Integration test approach : Router oneshot harness

**Retenu** : enrichir le test harness `mk_state()` dans http.rs
pour fournir un `DaemonHttpState` complet (avec canary_input,
consent DB, files tmp dir, invite counter, quarantine queue),
puis utiliser `Router::new().oneshot(Request)` pour chaque route
testee. Un test par chemin d'erreur + un test happy-path par
route.

Le pattern existe deja dans http.rs (21 tests pour les routes
core). On etend le meme pattern aux routes manquantes.

**Rejete** :
- Tests unitaires seulement (sans Router) : ne verifient pas le
  wiring axum (path params, extractors, middleware auth). C'est
  exactement ce que le carry P2-AUDIT-A-1-S43 signale comme gap.
- Tests E2E avec daemon reel : trop lourd pour 33 routes, necessite
  un daemon reel + SQLite + iroh. Les tests Router oneshot sont
  suffisants pour le wiring.
- Mock-based tests : masquent les vrais bugs d'integration (cf.
  feedback_approach.md : no mocks).

**Implications code** : http.rs (harness mk_state()), chaque
*_api.rs (ajout tests), consent.rs, files.rs (ajout tests).

### D2 — Test coverage scope : MANDATORY 12 + extensions ciblées

**Retenu** : Phase A couvre les 12 routes originales du carry
MANDATORY (consent 4, files 3, canary_api 3, contributor_api 2).
Phase B dette absorbe les integration tests des routes ajoutees
S44-S45 (invite 3, quarantine 3, tasks 2, kudos 2, health 1,
shell 1, diagnostic 1, worker_state 1 = 14 routes). Total 26
routes couvertes.

**Rejete** :
- Couvrir les 33 routes : deploy.rs et apps.rs necessitent des
  mocks iroh-blobs ou un filesystem temp complexe. Scope cut S47.
- Couvrir seulement les 12 MANDATORY : laisserait 21 routes sans
  tests, les routes S44-S45 n'ont jamais eu de review integration.

**Implications code** : Phase A = 12 tests, Phase B += 14 tests.
Total delta prevu ~+26 tests Rust minimum.

### D3 — Frontend direct-daemon : unification API client

**Retenu** : refactorer `web/src/api/coordinator.ts` (720 LOC)
pour pointer vers le daemon API `/api/v1/*` au lieu du
coordinator Python port 8787. Le daemon sert deja toutes les
routes standard. La variable `VITE_COORDINATOR_URL` est remplacee
par `VITE_SBFB_DAEMON_URL` (defaut `""` = same origin, leveraging
`1f1a017`). Renommer le fichier `coordinator.ts` → `daemon.ts`
pour refléter le changement.

Les routes app-specific (commands, state, manifest app) qui
n'existent que sur le coordinator Python restent pointees vers
le coordinator URL via un fallback explicite. Pas de suppression
du coordinator Python.

**Rejete** :
- Dual-mode permanent (coordinator + daemon en parallele) : source
  de confusion, double surface. Le daemon est le point unique
  d'entree (cf. LAUNCHER.md vision).
- Supprimer coordinator.ts completement : les routes app runtime
  n'existent pas encore sur le daemon.

**Implications code** : web/src/api/coordinator.ts (renommage +
refactor), tous les imports dans les composants React, tests
Vitest des API helpers, web/src/api/auth.ts (deja adapte par
hotfix).

### D4 — Debt phase scope : 5 items S44 batch complet

**Retenu** : absorber les 5 items S44 a 2/3 dans la phase dette
(Phase B) EN PLUS des 14 integration tests routes recentes.
Items :
1. P2-REVIEW-A-1-S44 as_str/serde coupling — decouplage
2. P2-REVIEW-B-1-S44 kudos entries pagination — limit/offset
3. P3-REVIEW-B-2-S44 shell discover self-only — deja corrige
   post-S45, ajouter integration test
4. P3-AUDIT-A-1-S44 test pagination handler-level — tests
5. P3-AUDIT-B-1-S44 diagnostic silent fallback — Err → 500

**Rejete** :
- Cherry-pick top 3 : les 5 deviennent tous MANDATORY S47 si non
  traites. 5 MANDATORY simultanes = saturation sprint S47.
- Reporter a S47 : sprint impair S47 n'a pas de phase dette
  obligatoire, absorber 5 MANDATORY + features = explosif.

**Implications code** : kudos_api.rs, shell_api.rs,
diagnostic_api.rs, plus fichiers concernes par as_str/serde.

---

**Acknowledged review findings (G1)** :

Scoring : D1 ✅, D2 ✅, D3 ⚠️, D4 ⚠️.
Rigor signal G4 satisfait (2 ⚠️ sur 4, 0 ❌).

D3 ⚠️ (migration tracker absent) : le preflight G8 Phase C
produira un inventaire route-migration (coordinator path → daemon
/api/v1/* path) avant le premier Edit. Le scan S1a/S1b du
preflight captera les divergences de paths.

D4 ⚠️ (scope boundary 2/3 ambigu) : le "2/3" est un compteur de
reports (pas un %), clarification ajoutee. Phase B §B.4 dans le
plan specifiera un critere d'acceptation par item dette (pas
seulement un compteur global). Le re-work P3-AUDIT-B-1-S44
(diagnostic silent fallback) est justifie : le comportement
silencieux (unwrap_or_default) persiste malgre le PASS audit.

---

## §5 Plan Phase outline A..D

### Phase A — Integration tests 12 routes MANDATORY

**But** : enrichir mk_state() et ecrire des tests Router oneshot
pour les 12 routes originales du carry P2-AUDIT-A-1-S43.
- consent.rs : 4 routes (get_consent, set_consent, whitelist_add,
  whitelist_remove) — ~8 tests (happy + error par route)
- files.rs : 3 routes (upload, manifest, stream) — ~6 tests
- canary_api.rs : 3 routes (freshness, inject-rate,
  observed-divergence) — ~6 tests
- contributor_api.rs : 2 routes (project, envelope) — ~4 tests
- mk_state() enrichissement : canary_input, consent DB state,
  files tmpdir
- Commit : `feat(sprint46): Sprint 46 Phase A — integration tests
  12 routes MANDATORY P2-AUDIT-A-1-S43`

### Phase B — Dette pair + integration tests routes recentes

**But** : phase dette obligatoire (sprint pair) + extension tests
integration routes S44-S45.
- 5 items dette S44 :
  1. as_str/serde coupling → decouplage
  2. kudos entries pagination → limit/offset params
  3. shell discover self-only → filtre
  4. test pagination handler-level → tests
  5. diagnostic silent fallback → Err → 500
- Integration tests 14 routes recentes : invite (3), quarantine (3),
  tasks (2), kudos (2), health (1), shell (1), diagnostic (1),
  worker_state (1)
- Commit : `feat(sprint46): Sprint 46 Phase B — dette pair S44
  5 items + integration tests 14 routes recentes`

### Phase C — Frontend direct-daemon

**But** : migrer le frontend vers l'API daemon directe.
- Renommer coordinator.ts → daemon.ts
- Remplacer VITE_COORDINATOR_URL → VITE_SBFB_DAEMON_URL
- Adapter tous les imports dans les composants React
- Mettre a jour les tests Vitest
- Garder fallback coordinator pour routes app-specific
- Commit : `feat(sprint46): Sprint 46 Phase C — frontend
  direct-daemon migration coordinator→daemon`

### Phase D — Wrap-up

---

## §6 Items carry/dette

### Resolus S46 (plan)

- [plan] **P2-AUDIT-A-1-S43** integration test gap 12 routes 3/3
  **MANDATORY** : Phase A
- [plan] P2-REVIEW-A-1-S44 as_str/serde coupling 2/3 : Phase B
- [plan] P2-REVIEW-B-1-S44 kudos entries pagination 2/3 : Phase B
- [plan] P3-REVIEW-B-2-S44 shell discover self-only 2/3 : Phase B
  (code deja corrige post-S45, ajouter integration test)
- [plan] P3-AUDIT-A-1-S44 test pagination handler-level 2/3 :
  Phase B
- [plan] P3-AUDIT-B-1-S44 diagnostic silent fallback 2/3 : Phase B

### Carries confirmes S47

- [carry] P2-A-1 rand blocker upstream 10+/3 : exemption blocker
  externe (rand 0.8.x getrandom Tier 3 Windows Arm64 — non resolu
  upstream). Justification renouvelee : pas de release rand 0.9
  ni fix getrandom upstream depuis dernier check.
- [carry] P2-AUDIT-2 pre-release transitives iroh : herite pin
  0.98 (Day 0 #3)
- [carry] P2-REVIEW-A-1-S45 diagnostic Err path non teste 1/3 →
  2/3 S47
- [carry] P2-REVIEW-A-2-S45 invite ID collision multi-daemon 1/3
  → 2/3 S47
- [carry] P2-REVIEW-B-1-S45 modules Python suppression differee
  1/3 → 2/3 S47
- [carry] P3-AUDIT-B-4-S45 TOCTOU canary reload fenetre
  microseconde 1/3 : nouveau, acceptable pre-v1.0

### Integration tests gap residuel (nouveau carry)

- [carry] P2-INT-1-S46 integration tests deploy.rs + apps.rs
  (4 routes) : scope cut S47. Necessite mock iroh-blobs ou
  filesystem temp complexe.
- [carry] P2-INT-2-S46 integration test auth/token (1 route) :
  scope cut S47. Necessite mock AuthState.

---

## §7 Scope cuts

1. **events.py SSE streaming** — S47+ (dep AppEvents bus Rust)
2. **App runtime migration Rust** (AppContext, commands, state) —
   S47+ (multi-sprint, coordinator.py encore necessaire)
3. **MCP server migration Rust** — S47+ (dep runtime apps)
4. **PyO3 bindings removal** — S47+ (dep runtime apps portees)
5. **Suppression complete packages/nexus-coordinator/** — S47+
   (dep runtime apps Rust)
6. **CI/VPS/v1.0** — S48+
7. **Kudos debit/stake** — interdit (Day 0 #7)
8. **Integration tests deploy.rs + apps.rs** — S47 (dep mock
   iroh-blobs)
9. **Integration test auth/token** — S47 (dep mock AuthState)
10. **invite ID collision UUID fix** — S47 (P2 1/3→2/3)
11. **diagnostic Err path test** — S47 (P2 1/3→2/3)
12. **modules Python suppression differee** — S47 (P2 1/3→2/3)
13. **demos/babel-library cleanup** — hors-sprint (en attente
    decision utilisateur)

---

## §8 Tracabilite scope (S45 → S46)

| S45 scope cut | S46 disposition |
|---|---|
| events.py SSE streaming — S46+ | Scope cut reporte S47+ |
| App runtime migration Rust — S46-47 | Scope cut reporte S47+ |
| Frontend coordinator→daemon URL migration — S46 | **Phase C** |
| MCP server migration Rust — S46+ | Scope cut reporte S47+ |
| PyO3 bindings removal — S46+ | Scope cut reporte S47+ |
| Suppression complete coordinator Python — S46-47 | Scope cut reporte S47+ |
| CI/VPS/v1.0 — S46-48 | Scope cut reporte S48+ |
| Kudos debit/stake — interdit | Day 0 #7 |

---

## §9 Risk register

| # | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | mk_state() enrichissement cascade | Medium | Medium | Incremental : un champ a la fois, tests apres chaque ajout |
| R2 | files.rs tests necessitent tmpdir async | Low | Medium | Pattern tokio::fs + tempdir() deja utilise dans worker_state |
| R3 | Frontend migration casse dev flow Vite | Medium | Low | Garder VITE_SBFB_DAEMON_URL="" = same origin, proxy Vite optionnel |
| R4 | canary_input None dans mk_state() | Medium | Low | Enrichir avec canary_input: Some(Arc::new(CanaryInput::test())) |
| R5 | 5 items dette plus gros qu'estime | Low | Medium | Budget phase B = dette + tests, pas de feature ajoutee |
| R6 | Renommage coordinator.ts casse 50+ imports | Low | Medium | Batch rename via IDE/grep, tests Vitest valident |

---

## §10 Audit gate pattern — rappel

Phase 0 S45 jouee (PASS `72f4083`). Phase D produira
sprint47_audit_plan.md pour la session fraiche S47.

---

## §11 Checkpoint de validation

1. **D1** : enrichir mk_state() vs creer un nouveau harness ?
   → enrichir (pattern existant, 21 tests precedent)
2. **D2** : 12 routes MANDATORY seulement ou 26 routes batch ?
   → 26 (12 MANDATORY Phase A + 14 Phase B dette)
3. **D3** : renommer coordinator.ts → daemon.ts risque ?
   → oui, 50+ imports, mais batch rename automatisable
4. **D4** : 5 items dette en 1 phase realiste ?
   → oui, estimation ~150-200 LOC total, 5 petits items
