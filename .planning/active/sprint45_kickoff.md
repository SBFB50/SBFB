# Sprint 45 — Kickoff (suppression maximale coordinator Python + portage routes restantes)

**Ecrit** : 2026-04-30 (post-audit gate S44 PASS `c3adbe7`).
**Type** : **sprint impair** — pas de phase dette obligatoire
(§6.2.1 Regle 1).
**Tip master d'entree** : `eccff1f`.
**Phase 0 audit Sprint 44** : **DEJA JOUE** — `c3adbe7` PASS.

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-04-30 (S44 meme jour).
  0 trigger actif. Pas de pre-research.

- **Technologies S45** : axum (deja dep daemon), blake3 (deja dep),
  tokio (deja dep). Pas de nouvelle dep externe.

- **Roadmap** : `.planning/roadmap_v1_migration_rust.md` §S45
  "Suppression coordinator Python + cleanup".

- **ROADMAP_COMMITMENTS** : aucun declenche (tous requierent tag
  v1.0, pas encore pose). LT-1 reclassifie pre-v1.0 mais cible
  S50 — pas S45.

- **Analyse factuelle ecart roadmap** : la roadmap prescrivait
  "Supprimer packages/nexus-coordinator/ entierement". L'analyse
  revele que le coordinator Python heberge encore le runtime apps
  (AppContext, NexusApp ABC, events bus pub/sub, commands, state)
  qui depend du SDK Python — ~2500+ LOC non portables en 1 sprint.
  Routes encore non portees : invite (3 routes, 97 LOC Python),
  quarantine (3 routes, 113 LOC Python), events SSE (2 routes,
  195 LOC, dep AppEvents bus), app runtime (commands/submit/state,
  ~400+ LOC, dep AppContext), MCP server (176 LOC, dep runtime).
  S45 ajuste le scope : porter les routes autonomes (invite,
  quarantine), supprimer les routes/modules redondants, nettoyer
  le dead code. Le coordinator subsiste comme runtime apps minimal.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 44 CLOSED + audit PASS. Migration API routes Tier 5
complete S35-S44 : 20+ routes portees vers handlers axum Rust
dans `crates/nexus-shell-daemon/src/`. Restent non portees :

| Fichier | LOC Python | Routes | Dep |
|---|---|---|---|
| invites.py | 97 | 3 (create, list, revoke) | invite.rs (S41) |
| quarantine.py | 113 | 3 (list, flush, drop) | quarantine_queue.rs (S21) |
| events.py | 195 | 2 (SSE stream, publish) | AppEvents bus SDK |
| app routes (apps.py §AppContext) | ~400 | 4 (commands, submit, state, manifest) | AppContext SDK |
| mcp_server.py | 176 | mount Streamable HTTP | runtime coordinator |

Routes autonomes portables maintenant : invite + quarantine (210 LOC,
logique Rust existante). Routes dependant du runtime apps : events +
app-specific + MCP (~770 LOC, dep SDK/AppContext → multi-sprint).

Frontend `web/src/api/coordinator.ts` : 20 fonctions exportees
appelant le coordinatorUrl (Python port 8787). Toutes les routes
standard ont des equivalents Rust sur le daemon (port 3000). Les
routes app-specific (`/app/{name}/commands`, `/app/{name}/state`,
etc.) n'existent que sur le coordinator Python.

Le launcher (`nexus-launcher`) spawne UNIQUEMENT le daemon, pas le
coordinator Python. Le coordinator est un process optionnel demarre
separement.

Dead code Rust identifie : `coord_http_client`, `coord_base_url`,
`resolve_coord_base_url()` dans `http.rs` — references par aucun
handler, vestige du proxy coordinator→daemon.

### §1.2 Compteurs tests entree (tip `eccff1f`)

| Suite | Count |
|---|---|
| Rust nextest | 1127 |
| Rust doctests | 6 passed, 1 ignored |
| SDK pytest | 195 |
| Coord pytest | 409 + 36 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 267 |
| Playwright | 42 + 2 fail (env pre-existant) |
| size-limit | 7/7 |
| **Total** | **~2130** |

---

## §2 Goal en une phrase

Le sprint **porte les 2 dernieres routes autonomes** (invite,
quarantine) vers le daemon Rust, **resout 6 carries** (SHA-256→
BLAKE3, coord dead_code, worker_state tokio::fs, list_tasks status,
TOCTOU canary, silent null canary, hex case-sensitivity),
**supprime les routes/modules Python redondants** (~12 fichiers
API + modules associes), et **nettoie le dead code** Rust + Python.
**Critere SMART : 28+ rows fail-fast verts au verification.md.**

---

## §3 Phase 0 — Audit gate Sprint 44

**DONE** — `c3adbe7`. Verdict PASS. 0 P0/P1, 3 P2 carries,
2 P3 nouveaux + 1 P3 carry confirme.

---

## §4 Decisions Day 0 (D1..D4 gelees)

### D1 — Scope realiste S45 : suppression maximale, pas totale

**Retenu** : porter les routes autonomes (invite, quarantine),
supprimer les routes/modules Python redondants (~12 fichiers API),
resoudre les carries eligibles (7 items), nettoyer dead code Rust.
Le coordinator Python subsiste comme runtime apps minimal
(AppContext, events, commands, state, MCP) jusqu'au portage Rust
du runtime apps.

**Rejete** :
- Suppression complete coordinator Python : necessite portage
  runtime apps (AppContext, NexusApp ABC, events bus, commands,
  state) ~2500+ LOC. Depend SDK Python (nexus_sdk). Multi-sprint.
  Le runtime apps est le coeur fonctionnel du coordinator — pas
  un residuel suprimable.
- Tout reporter a S46 : 6 routes autonomes (invite + quarantine)
  non portees, carry SHA-256→BLAKE3 a exemption remplie (Python
  removed), 3 P3 a 2/3 qui deviendraient 3/3 MANDATORY. Retarder
  accumule les MANDATORY inutilement.
- Porter events.py SSE : depend AppEvents bus Python (pub/sub
  async `nexus_sdk.AppEvents`). Pas de bus Rust equivalent. Scope
  creep — porter le bus est un sous-projet.

### D2 — Route portage batch : invite + quarantine

**Retenu** : porter les 6 routes Python restantes autonomes
(3 invite + 3 quarantine) vers handlers axum dans
`crates/nexus-shell-daemon/src/`. Pattern identique S42-S44
(State extractor + Json + coordinator_db).

(a) **invite routes** (97 LOC Python → ~80-100 LOC Rust) :
    - `POST /api/v1/invite/create` — creer une invitation Ed25519
    - `GET /api/v1/invite` — lister les invitations
    - `DELETE /api/v1/invite/{invite_id}` — revoquer une invitation
    Depend `invite.rs` dans `nexus-coordinator-rs` (deja porte S41
    Phase B). Queries DB `create_invite()`, `list_invites()`,
    `revoke_invite()` a ajouter dans `db.rs`.

(b) **quarantine routes** (113 LOC Python → ~90-110 LOC Rust) :
    - `GET /api/v1/quarantine` — lister la quarantine queue
    - `POST /api/v1/quarantine/{row_id}/flush` — liberer un item
    - `POST /api/v1/quarantine/{row_id}/drop` — supprimer un item
    Depend `quarantine_queue.rs` dans `nexus-coordinator-rs` (deja
    porte S21 Phase D). Queries DB `list_quarantine()`,
    `flush_quarantine()`, `drop_quarantine()` a ajouter dans
    `db.rs`.

**Rejete** :
- Garder invite/quarantine en Python : logique Rust existante
  (invite.rs, quarantine_queue.rs), pas de blocker. Reporter
  rend la suppression Python plus complexe.
- Porter events.py SSE avec : depend AppEvents bus Python, pas de
  bus Rust.

### D3 — Carries resolus S45

**Retenu** : resoudre 7 carries dans le sprint :

(a) **P2-REVIEW-C-1-S40 SHA-256→BLAKE3** (6/3, exemption remplie) :
    `redundancy.rs` utilise `sha2::Sha256`. Avec la suppression des
    routes Python, la parite wire Python/Rust n'est plus necessaire.
    Migration vers `blake3::hash()`. ~10 LOC.

(b) **P2-REVIEW-B-1-S43 coord dead_code cleanup** (2/3) :
    `coord_http_client`, `coord_base_url`, `resolve_coord_base_url()`
    dans `http.rs` + `runtime.rs` ne sont references par aucun
    handler. Dead code herite du proxy coordinator→daemon. Suppression
    + retrait dep reqwest si plus aucun consumer. ~30 LOC.

(c) **P2-REVIEW-C-1-S44 worker_state tokio::fs** (1/3) :
    `worker_state_api.rs` utilise `std::fs::read_to_string` bloquant
    dans handler async. Migration vers `tokio::fs::read_to_string`.
    ~5 LOC.

(d) **P3-REVIEW-C-2-S44 list_tasks status invalide** (1/3) :
    `tasks_api.rs` passe un `state` invalide a SQL → 0 resultats au
    lieu de 400. Ajouter validation enum avant query. ~10 LOC.

(e) **P3-REVIEW-A-1-S43 TOCTOU canary reload** (2/3) :
    `canary_input.rs` reload pattern sans lock atomique. Ajouter
    RwLock ou AtomicBool pour prevenir TOCTOU. ~20 LOC.

(f) **P3-AUDIT-A-2-S43 silent null canary_api** (2/3) :
    `canary_api.rs` fallback silencieux sur erreur. Retourner 500
    au lieu de donnees vides trompeuses. ~10 LOC.

(g) **P3-AUDIT-A-3-S43 hex case-sensitivity** (2/3) :
    Normaliser les hex strings en lowercase dans la validation
    pour eviter les faux negatifs case-dependent. ~10 LOC.

**Rejete** :
- Reporter SHA-256→BLAKE3 : exemption conditionnee a S45 (Python
  wire), condition remplie maintenant. 6/3 = bien au-dela du seuil.
- Reporter les 3 P3 a 2/3 : deviendraient 3/3 MANDATORY S46,
  creant 4 MANDATORY au lieu de 1. Resoudre maintenant (total ~40
  LOC) est plus efficient.

### D4 — Coordinator Python gut

**Retenu** : supprimer les ~12 fichiers routes Python deja portes
en Rust + les modules coordinateur redondants + leurs tests.
Conserver uniquement le runtime apps (AppContext, events, commands,
state, MCP, SDK integration). Adapter `app.py` pour ne plus monter
les routes supprimees.

Fichiers routes a supprimer :
- `api/deploy.py` (ported S42)
- `api/apps.py` (ported S42)
- `api/consent.py` (ported S43)
- `api/files.py` (ported S43)
- `api/canary.py` (ported S43)
- `api/contributor.py` (ported S43)
- `api/health.py` (ported S44)
- `api/shell.py` (ported S44)
- `api/tasks.py` (ported S35+S44)
- `api/kudos.py` (ported S36+S44)
- `api/diagnostic.py` (ported S44)
- `api/worker_state.py` (ported S44)

Modules a supprimer (logique portee en Rust) :
- `dispatcher.py` (→ nexus-coordinator-rs/dispatcher.rs S35)
- `validator.py` (→ nexus-coordinator-rs/validator.rs S35)
- `kudos.py` (→ nexus-coordinator-rs/kudos_ledger.rs S36)
- `output_filter.py` (→ nexus-coordinator-rs/output_filter.rs S38)
- `guardrails.py` (→ nexus-coordinator-rs/guardrails.rs S38)
- `result_guardrails.py` (→ nexus-coordinator-rs S38)
- `pii_redactor.py` (→ nexus-coordinator-rs/pii_redactor.rs S39)
- `canary_registry.py` (→ nexus-coordinator-rs/canary_registry.rs S39)
- `fairness.py` (→ nexus-coordinator-rs/fairness.rs S41)
- `pow_counter.py` (→ nexus-coordinator-rs/pow_counter.rs S41)
- `capability_store.py` (→ nexus-coordinator-rs/capability_store.rs S41)
- `contributor_registry.py` (→ nexus-coordinator-rs/contributor_registry.rs S41)
- `redundancy.py` (→ nexus-coordinator-rs/redundancy.rs S40)
- `watermark_detector.py` (→ nexus-coordinator-rs/watermark_detector.rs S40)

Tests a supprimer : tous les fichiers `test_*.py` correspondant
aux modules/routes supprimes. Conserver uniquement les tests du
runtime apps (events, commands, state, hooks, rerun, canary_input
coordinator-side).

**Rejete** :
- Supprimer `packages/nexus-coordinator/` entierement : runtime
  apps (AppContext, NexusApp ABC, events bus, commands, state,
  hooks, MCP) encore necessaire. Le coordinator est le host des
  apps Python — pas supprimable tant que les apps ne sont pas
  portees.
- Garder les routes Python redondantes : source de confusion,
  maintenance double, surface d'attaque inutile. Toutes les
  routes sont portees en Rust.

---

**Acknowledged review findings (G1)** :

Scoring : D1 ✅, D2 ✅, D3 ✅, D4 ✅.
Rigor signal G4 satisfait (2 shadows documentes, 0 ❌).

Shadow-1 (D3b Low-Med) : `resolve_coord_base_url()` appelee dans
`runtime.rs:511` au boot. Decision : verifier en G8 preflight
Phase B — si le boot init l'utilise, supprimer l'appel au boot
en meme temps que la fonction. Pas de modification du plan.

Shadow-2 (D4 Medium) : modules Python non-route a conserver pas
listes explicitement. Decision : le plan §6.2 liste les modules
a DELETE, tout ce qui n'est pas dans la liste survit. Le risque
import chain est couvert par les tests Python restants (R3 dans
le risk register). Phase B executera un `uv run pytest` intermediaire
apres chaque batch de suppressions pour detecter les breakages.

---

## §5 Plan Phase outline A..C

### Phase A — Route portage + carries resolus

**But** : porter les 6 routes invite + quarantine vers daemon Rust
+ resoudre les 7 carries.
- `invite_api.rs` (NEW) : 3 routes create/list/revoke
- `quarantine_api.rs` (NEW) : 3 routes list/flush/drop
- `db.rs` : 6 queries (create_invite, list_invites, revoke_invite,
  list_quarantine, flush_quarantine, drop_quarantine)
- `http.rs` : ajouter 6 routes + mod declarations
- `redundancy.rs` : sha2::Sha256 → blake3::hash()
- `worker_state_api.rs` : std::fs → tokio::fs
- `tasks_api.rs` : validation status enum
- `canary_input.rs` : TOCTOU reload fix
- `canary_api.rs` : silent null → 500
- Normalisation hex lowercase
- Tests unitaires + integration HTTP par handler
- Commit : `feat(sprint45): Sprint 45 Phase A — invite + quarantine
  API Rust + SHA-256→BLAKE3 + 6 carries resolus`

### Phase B — Coordinator Python gut + dead code cleanup

**But** : supprimer les routes/modules/tests Python redondants +
dead code Rust + workspace cleanup.
- Supprimer ~12 fichiers routes Python portes
- Supprimer ~14 modules Python redondants
- Supprimer tests Python des routes/modules supprimes
- Supprimer dead code Rust : coord_http_client, coord_base_url,
  resolve_coord_base_url() + test associe
- Adapter coordinator app.py pour ne plus monter les routes
  supprimees
- Nettoyer pyproject.toml workspace si applicable
- Mettre a jour CLAUDE.md, PATTERNS.md, HARDENING_ROADMAP
- Commit : `feat(sprint45): Sprint 45 Phase B — coordinator Python
  gut + dead code Rust cleanup + docs update`

### Phase C — Wrap-up

---

## §6 Items carry/dette

### Resolus S45 (plan)

- [plan] P2-REVIEW-C-1-S40 SHA-256→BLAKE3 (6/3) : Phase A
- [plan] P2-REVIEW-B-1-S43 coord dead_code cleanup (2/3) : Phase B
- [plan] P2-REVIEW-C-1-S44 worker_state tokio::fs (1/3) : Phase A
- [plan] P3-REVIEW-C-2-S44 list_tasks status invalide (1/3) : Phase A
- [plan] P3-REVIEW-A-1-S43 TOCTOU canary reload (2/3) : Phase A
- [plan] P3-AUDIT-A-2-S43 silent null canary_api (2/3) : Phase A
- [plan] P3-AUDIT-A-3-S43 hex case-sensitivity (2/3) : Phase A

### Carries confirmes S46

- [carry] P2-A-1 rand blocker upstream 10+/3 : exemption blocker
  externe (rand 0.8.x getrandom Tier 3 Windows Arm64 — non resolu
  upstream)
- [carry] P2-AUDIT-2 transitives iroh : herite pin 0.98
- [carry] P2-AUDIT-A-1-S43 integration test gap 12 routes 3/3 :
  **MANDATORY S46** (§6.2.1 Regle 2). Tous les handlers Rust
  manquent de tests d'integration HTTP pour les chemins d'erreur.
- [carry] P2-REVIEW-A-1-S44 as_str/serde coupling 2/3
- [carry] P2-REVIEW-B-1-S44 kudos entries pagination 2/3
- [carry] P3-REVIEW-B-2-S44 shell discover self-only 2/3
- [carry] P3-AUDIT-A-1-S44 test pagination handler-level 2/3
- [carry] P3-AUDIT-B-1-S44 diagnostic silent fallback 2/3

---

## §7 Scope cuts

1. **events.py SSE streaming** — S46+ (dep AppEvents bus Rust)
2. **App runtime migration Rust** (AppContext, commands, state) —
   S46-47 (multi-sprint)
3. **Frontend coordinator→daemon URL migration** — S46 (apres
   routes restantes portees)
4. **MCP server migration Rust** — S46+ (dep runtime apps)
5. **PyO3 bindings removal** — S46+ (si runtime apps en Rust)
6. **Suppression complete packages/nexus-coordinator/** — S46-47
   (dep runtime apps Rust)
7. **CI/VPS/v1.0** — S46-48
8. **Kudos debit/stake** — interdit (Day 0 #7)

---

## §8 Risk register

| # | Risque | Impact | Mitigation |
|---|---|---|---|
| R1 | invite.rs queries DB inexistantes | Low | Pattern db.rs existant (S36-S44), SQLite basique |
| R2 | quarantine_queue.rs API surface inconnue | Low | Module deja teste unitairement S21, adapter |
| R3 | Suppression module Python casse import chain | Medium | Tests Python residuels detectent immediatement |
| R4 | coord_http_client supprime mais utilise ailleurs | Low | Grep exhaustif confirme 0 consumer |
| R5 | Test count drop significatif (coord -300+) | Low | Attendu — on supprime du code, pas du comportement |
| R6 | deploy.rs utilise reqwest — dep pas supprimable | Low | Verifier consumers avant retrait |

---

## §9 Checkpoint de validation

1. **D1** : scope realiste vs roadmap ? → oui, drift documente,
   runtime apps non portable en 1 sprint
2. **D2** : 6 routes en 1 phase ? → oui, ~200 LOC Rust total,
   logique existante en Rust
3. **D3** : 7 carries resolvables ? → oui, ~100 LOC total, tous
   localises
4. **D4** : coordinator gut safe ? → oui, toutes les routes sont
   portees, tests detectent les regressions
