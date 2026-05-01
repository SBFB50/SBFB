# Sprint 47 — Kickoff (carry resolution batch S45 + integration tests completion + happy path tests)

**Ecrit** : 2026-05-01 (post-audit gate S46 PASS `d1ef20d`).
**Type** : **sprint impair** — pas de phase dette obligatoire
(§6.2.1 Regle 1).
**Tip master d'entree** : `d1ef20d`.
**Phase 0 audit Sprint 46** : **DEJA JOUE** — `d1ef20d` PASS
(0 P0, 0 P1, 1 P2, 2 P3).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-04-30 (S46 meme jour).
  0 trigger actif. 5 triggers surveilles : iroh > 0.98, wasmtime
  LTS bump, arti-client > 0.41, Tor PoW hspow, NIST PQC FIPS —
  aucun realise. Pas de pre-research.

- **Technologies S47** : aucune nouvelle dep externe. Sprint pure
  tests + fix carry + cleanup. Memes patterns axum
  Router::oneshot + mk_state() que S46.

- **ROADMAP_COMMITMENTS check** : LT-1 reclassifie pre-v1.0
  cible S50. LT-2..LT-5 latents (tag v1.0 non pose). LT-6
  RESOLVED. 0 condition declenchee.

- **HARDENING_ROADMAP §3** : pas de ligne S47 prescrite. Pas de
  drift a documenter.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 46 CLOSED + audit PASS. S46 a livre :
- Phase A : 19 tests Router::oneshot() pour 12 routes MANDATORY
  (P2-AUDIT-A-1-S43 CLOSED apres 3 sprints)
- Phase B : 5 items dette S44 CLOSED + 17 tests pour 14 routes
  recentes
- Phase C : frontend direct-daemon migration coordinator.ts →
  /api/v1/*, daemon.ts proxy envelope supprime, -260 LOC

**Etat actuel des tests d'integration HTTP (audit factuel)** :
54 routes daemon total. 49 ont des tests Router-level (21
pre-S46 + 19 Phase A + 17 Phase B = 57 tests, certaines routes
testees plusieurs fois). **5 routes manquent de tests
d'integration** :
- deploy.rs : 2/2 (deploy_private, deploy_from_repo)
- apps.rs : 2/2 (list_apps, get_app)
- http.rs : 1/1 (auth_token_public)

deploy.rs a 8 tests unitaires (validation) et apps.rs a 11 tests
unitaires, mais aucun test Router::oneshot() d'integration.

**3 items S45 a 2/3** — deviennent MANDATORY S48 si non resolus :
- P2-REVIEW-A-1-S45 diagnostic Err path non teste (4 error paths
  dans diagnostic_api.rs, tous non testes)
- P2-REVIEW-A-2-S45 invite ID collision multi-daemon (format
  `inv-{ts}-{seq}`, collision si 2 daemons demarrent meme seconde)
- P2-REVIEW-B-1-S45 modules Python suppression differee (21+
  modules avec equivalents Rust, mais coordinator.py les importe
  encore pour le runtime)

### §1.2 Ancrage HARDENING_ROADMAP

Pas de ligne S47 prescrite. 0 trigger actif. Pas d'item drift
a documenter.

### §1.3 Compteurs tests entree (tip `d1ef20d`)

| Suite | Count |
|---|---|
| Rust nextest | 1168 |
| Rust doctests | 6 passed, 1 ignored |
| SDK pytest | 195 |
| Coord pytest | 323 + 23 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 267 |
| Playwright | 42 + 2 fail (env pre-existant) |
| size-limit | 5/5 |
| **Total** | **~1984** |

### §1.4 Pre-launch protocol policy (rappel)

`*_FORMAT_VERSION` restent a 1. S47 ne touche pas de wire format
canonical. Pas de tolerant decoder multi-version. Sprint pure
tests + fix carry + cleanup — pas d'impact wire.

---

## §2 Goal

Resoudre les 3 items S45 a 2/3 avant qu'ils deviennent MANDATORY
S48, completer la couverture integration tests des 5 routes
daemon restantes (deploy 2 + apps 2 + auth/token 1), et ajouter
les happy path tests consent/files manquants depuis S46.
**Critere SMART : 28+ rows fail-fast verts au verification.md,
mesure binaire au Phase D wrap-up.**

---

## §3 Phase 0 — Audit gate S46

**DEJA JOUE** : commit `d1ef20d` PASS (0 P0, 0 P1, 1 P2,
2 P3). Audit findings dans
`.planning/active/sprint46_audit_findings.md`. 13 carries
documentes pour S47 (cf. §6 ci-dessous).

---

## §4 Decisions Day 0 (D1..D4 gelees)

### D1 — Invite ID collision fix : node_id prefix

**Retenu** : prefixer l'ID invite avec les 8 premiers caracteres
du `node_id` hex du daemon. Format resultant :
`inv-{node_id_8}-{timestamp}-{seq}`. Le node_id est unique par
instance daemon (derive de la keypair Ed25519 iroh). Le prefixe
8 hex = 32 bits = collision negligeable entre daemons distincts.

Pas de nouvelle dep. Le `state.node_id` est deja disponible dans
le handler `create_invite` via `DaemonHttpState`.

**Rejete** :
- UUID v4 pur : ajoute dep uuid, perd la lisibilite temporelle
  de l'ID invite (timestamp utile pour debug et tri).
- Counter seeded alea : complexite inutile, le node_id resout
  le probleme multi-daemon proprement.
- Statu quo : collision garantie si 2 daemons demarrent meme
  seconde (P2-REVIEW-A-2-S45 docummente le risque).

**Implications code** : invite_api.rs (format ID), 1-2 tests
integration valident unicite cross-daemon.

### D2 — Integration tests deploy.rs + apps.rs : approche fixture

**Retenu** : creer un test fixture zip minimal (quelques fichiers
HTML valides) et un `DaemonHttpState` enrichi avec un
`BrowseAggregator` peuple de `BrowseEntry` mock. Pour deploy, le
test envoie le zip via `Router::oneshot(POST /api/v1/deploy)` et
verifie le 200 + hash retourne. Pour apps, le test peuple le
browse aggregator avec des entries et verifie list/get.

Pour `deploy_from_repo`, le test se limite aux **error paths**
(repo_url invalide, payload trop grand, format invalide) car le
happy path requiert un git clone reel qui sort du scope des tests
Router-level. Le happy path deploy_from_repo est couvert par les
8 tests unitaires existants dans deploy.rs.

**Rejete** :
- Node iroh reel en test : lent (3-5s boot), flaky (ports
  reseau), disproportionne pour des tests de wiring axum.
- Mock BlobsClient trait : BlobsClient n'est pas trait-based,
  mockable seulement via test double ou feature gate. Trop
  intrusif pour 2 tests.
- Skip deploy tests : laisserait 2 routes sans aucune couverture
  integration, le carry P2-INT-1-S46 resterait ouvert.

**Implications code** : http.rs (mk_state enrichi browse entries),
deploy test (fixture zip + error paths), apps test (fixture
entries + list/get).

### D3 — Python modules suppression : audit-then-delete bounded

**Retenu** : auditer les 21+ modules Python avec equivalents Rust
pour determiner lesquels sont **dead code effectif** (non appeles
a runtime par coordinator.py ni par les routes Python restantes).
Supprimer ceux qui sont dead code. Pour les modules encore appeles
par coordinator.py, documenter la dependance et reclassifier vers
"App Runtime Migration" (scope cut multi-sprint).

Audit factuel pre-code : grep coordinator.py + app.py + remaining
api/*.py pour chaque module candidat. Le grep donne 14 imports
dans coordinator.py → la majorite des modules sont encore appeles.
Les modules qui n'ont PAS d'import dans coordinator.py/app.py
sont candidats a suppression.

**Rejete** :
- Suppression totale 21 modules : briserait coordinator.py qui
  importe 14 d'entre eux pour le runtime apps.
- Refactoring coordinator.py pour appeler le daemon API : c'est
  le scope de "App Runtime Migration Rust" (multi-sprint, pas S47).
- Reporter indefiniment : l'item atteint 2/3, devient MANDATORY
  S48 sans resolution. L'audit + deletion partielle + reclassif
  resolve le carry.

**Implications code** : packages/nexus-coordinator/ (delete modules
dead code + tests associes), carry resolution documentee.

### D4 — Happy path tests consent/files : mk_state() enrichi

**Retenu** : enrichir mk_state() avec un consent config directory
temporaire et un files storage directory temporaire. Utiliser
Router::oneshot() pour tester :
- Consent : set level OK → get returns persisted → whitelist
  add OK → whitelist remove OK (4 tests)
- Files : upload petit fichier OK → manifest OK → stream OK
  (3 tests)

Meme pattern que les error path tests S46 Phase A.

**Rejete** :
- Tests seulement Python-side : les routes consent/files sont
  dans le daemon Rust, pas dans le coordinator Python.
- Tests E2E Playwright : trop lourds pour des handlers CRUD
  simples, les tests Router-level sont suffisants.
- Skip happy paths : les error paths seuls ne garantissent pas
  que le wiring fonctionne en cas nominal.

**Implications code** : http.rs (mk_state enrichi tmpdir consent
+ tmpdir files), consent tests (4 happy path), files tests
(3 happy path).

---

**Acknowledged review findings (G1)** :

Scoring : D1 ⚠️, D2 ⚠️, D3 ⚠️, D4 ⚠️.
Rigor signal G4 satisfait (4 ⚠️ sur 4, 0 ❌).

D1 ⚠️ (schema compat invite ID format) : acceptable pre-v1.0
(pre-launch protocol policy — pas de ledger externe qui stocke
des invite IDs). Le format est redefini, pas bumpe.

D2 ⚠️ (deploy happy path sous-estime) : le G8 Phase B re-evaluera
la feasibility du deploy_private happy path. mk_state() a un iroh
Node reel avec blobs_store() fonctionnel. De plus,
`BrowseAggregator::add_direct_entry()` existe deja — simplifie le
setup apps tests. Le scope cut deploy_private happy path est
reconsidere "try first".

D3 ⚠️ (zero-delete outcome) : risque ajoute au R2 — si l'audit
revele 0 module dead code, le carry est clos par evidence
documentee (tous modules encore utilises par coordinator.py,
suppression depend de App Runtime Migration). L'audit
systematique `grep -r` sur tout packages/nexus-coordinator/
sera fait en pre-code Phase A.

D4 ⚠️ (sbfb_home vs state) : **correction factuelle** — les
handlers consent.rs et files.rs utilisent `sbfb_home()` (env var
`SBFB_HOME` ou `~/.sbfb/`), PAS un champ DaemonHttpState.
L'approche correcte est de setter `SBFB_HOME` dans le test
harness vers un tmpdir, pas d'enrichir mk_state(). Plan §C.1/§C.2
corrige en consequence. Le G8 Phase C verifiera.

---

## §5 Plan Phase outline A..D

### Phase A — S45 carries resolution (3 items 2/3)

**But** : resoudre les 3 items S45 a 2/3 avant escalade MANDATORY
S48.
- P2-REVIEW-A-1-S45 : diagnostic Err path — 4 tests integration
  pour les 4 error paths de diagnostic_api.rs (DB lock poisoned,
  worker_contributions error, active_workers_since error ×2)
- P2-REVIEW-A-2-S45 : invite ID collision — prefixer avec
  node_id hex 8 chars
- P2-REVIEW-B-1-S45 : Python modules — audit callers + delete
  dead code modules + reclassifier modules encore appeles
- Commit : `feat(sprint47): Sprint 47 Phase A — S45 carries
  resolution diagnostic Err tests + invite ID fix + Python
  modules audit`

### Phase B — Integration tests 5 routes restantes

**But** : fermer le gap integration tests pour les 5 dernieres
routes sans couverture Router-level.
- deploy.rs : deploy_private happy path (fixture zip) + deploy_
  from_repo error paths (invalide, trop grand)
- apps.rs : list_apps (fixture browse entries) + get_app (found +
  not_found)
- http.rs : auth_token_public (valid + invalid bearer)
- Commit : `feat(sprint47): Sprint 47 Phase B — integration tests
  5 routes deploy+apps+auth completion`

### Phase C — Happy path tests + deprecated aliases cleanup

**But** : combler les happy path manquants consent/files et
nettoyer les aliases deprecated frontend.
- consent : set level 2 OK → get returns persisted → whitelist
  add OK → whitelist remove OK (4 tests)
- files : upload petit fichier OK → manifest OK → stream OK
  (3 tests)
- P2-REVIEW-C-2-S46 deprecated aliases : migrer les 12+ refs
  restantes dans AddCoordinatorDialog.tsx, projectStore.ts,
  coordinator.test.ts vers ApiProtocolError/ApiHttpError, puis
  supprimer les 3 alias exports
- Commit : `feat(sprint47): Sprint 47 Phase C — happy path tests
  consent+files + deprecated aliases cleanup`

### Phase D — Wrap-up

---

## §6 Items carry/dette

### Resolus S47 (plan)

- [plan] **P2-REVIEW-A-1-S45** diagnostic Err path non teste
  **2/3** : Phase A (4 tests)
- [plan] **P2-REVIEW-A-2-S45** invite ID collision multi-daemon
  **2/3** : Phase A (node_id prefix fix)
- [plan] **P2-REVIEW-B-1-S45** modules Python suppression
  differee **2/3** : Phase A (audit + delete dead code +
  reclassif modules vivants)
- [plan] P2-INT-1-S46 integration tests deploy.rs + apps.rs
  1/3 : Phase B
- [plan] P2-INT-2-S46 integration test auth/token 1/3 : Phase B
- [plan] P2-REVIEW-A-1-S46 consent happy path 1/3 : Phase C
- [plan] P2-REVIEW-A-2-S46 files upload happy path 1/3 : Phase C
- [plan] P2-REVIEW-C-2-S46 deprecated error class aliases 1/3 :
  Phase C

### Carries confirmes S48

- [carry] P2-A-1 rand blocker upstream 11+/3 : exemption blocker
  externe (rand 0.8.x getrandom Tier 3 Windows Arm64 — non resolu
  upstream). Justification renouvelee : pas de release rand 0.9
  ni fix getrandom upstream.
- [carry] P2-AUDIT-2 pre-release transitives iroh : herite pin
  0.98 (Day 0 #3)
- [carry] P3-AUDIT-B-4-S45 TOCTOU canary reload fenetre
  microseconde 1/3→2/3 : risque faible, fenetre microseconde,
  acceptable pre-v1.0
- [carry] P2-REVIEW-B-1-S46 kudos SQL pagination 1/3→2/3 :
  carry confirme (pas dans scope S47 tests-only)
- [carry] P2-REVIEW-C-1-S46 app-specific schema drift 1/3→2/3 :
  carry confirme (pas dans scope S47, depend app runtime migration)

---

## §7 Scope cuts

1. **events.py SSE streaming** — S48+ (dep AppEvents bus Rust)
2. **App runtime migration Rust** (AppContext, commands, state) —
   S48+ (multi-sprint, coordinator.py encore necessaire)
3. **MCP server migration Rust** — S48+ (dep runtime apps)
4. **PyO3 bindings removal** — S48+ (dep runtime apps portees)
5. **Suppression complete packages/nexus-coordinator/** — S48+
   (dep runtime apps Rust)
6. **CI/VPS/v1.0** — S49+
7. **Kudos debit/stake** — interdit (Day 0 #7)
8. **deploy_from_repo happy path test** — hors scope (git clone
   reel, couvert par 8 tests unitaires existants)
9. **kudos SQL pagination runtime fix** — S48 (carry 2/3)
10. **app-specific schema drift fix** — S48+ (dep app runtime)
11. **TOCTOU canary reload fix** — S48 (carry 2/3, fenetre
    microseconde, risque faible)

---

## §8 Tracabilite scope (S46 → S47)

| S46 scope cut | S47 disposition |
|---|---|
| events.py SSE streaming — S47+ | Scope cut reporte S48+ |
| App runtime migration Rust — S47+ | Scope cut reporte S48+ |
| MCP server migration Rust — S47+ | Scope cut reporte S48+ |
| PyO3 bindings removal — S47+ | Scope cut reporte S48+ |
| Suppression complete coordinator Python — S47+ | Scope cut reporte S48+ |
| CI/VPS/v1.0 — S48+ | Scope cut reporte S49+ |
| Kudos debit/stake — interdit | Day 0 #7 |
| Integration tests deploy.rs + apps.rs — S47 | **Phase B** |
| Integration test auth/token — S47 | **Phase B** |
| invite ID collision UUID fix — S47 | **Phase A** (D1) |
| diagnostic Err path test — S47 | **Phase A** |
| modules Python suppression differee — S47 | **Phase A** (D3) |

---

## §9 Risk register

| # | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | deploy_private test necessite BlobStore reel | Medium | Medium | Tester seulement les error paths deploy si fixture impossible |
| R2 | Python modules audit revele 0 module dead code | Low | Low | L'audit lui-meme resolve le carry (evidence documentee) |
| R3 | diagnostic Err tests necessitent mock Mutex poisoned | Low | Medium | Pattern existant dans S37 http.rs (`diagnostic_fairness_returns_500_on_poisoned_mutex`) |
| R4 | Happy path consent necessite filesystem writable | Low | Low | tempdir() + ConsentConfig::load() pattern existant |
| R5 | Deprecated aliases ont plus de callers que prevu | Low | Low | Batch rename grep-based, pas de risque architectural |
| R6 | apps.rs test necessite browse aggregator peuple | Low | Medium | apps.rs a deja 11 tests unitaires avec make_entry(), pattern reutilisable |

---

## §10 Audit gate pattern — rappel

Phase 0 S46 jouee (PASS `d1ef20d`). Phase D produira
sprint48_audit_plan.md pour la session fraiche S48.

---

## §11 Checkpoint de validation

1. **D1** : node_id prefix vs UUID v4 ?
   → node_id (pas de dep, lisibilite temporelle gardee)
2. **D2** : fixture zip vs iroh Node reel ?
   → fixture zip (rapide, pas de dep reseau)
3. **D3** : supprimer tous les modules vs audit-then-delete ?
   → audit-then-delete (coordinator.py les importe encore)
4. **D4** : mk_state() enrichi vs nouveau harness ?
   → mk_state enrichi (pattern existant, cohesion S46)
