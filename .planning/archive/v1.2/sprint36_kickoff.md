# Sprint 36 — Kickoff (migration Rust native — Phase 2 integration)

**Ecrit** : 2026-04-28 (session fraiche post-audit gate S35 `3013e44`).
**Type** : **sprint pair** — phase dette obligatoire
(§6.2.1 Regle 1 : S36 pair). Integration du coordinator-rs dans le
daemon + KudosLedger Rust natif.
**Tip master d'entree** : `3013e44` (chore(planning) audit findings
S35 PASS).
**Phase 0 audit Sprint 35** : **DEJA JOUE** — findings dans
`.planning/active/sprint35_audit_findings.md` (verdict **PASS**,
0 P0/P1, 2 P2 confirms, 1 P3).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** (2026-04-28) : HARDENING_ROADMAP last_validated
  `2026-04-28` (S34 Phase D). S35 kickoff confirme 0 trigger actif
  meme jour.

  Triggers verifies :
  - iroh > 0.98 : pas de release 0.99 — NOT FIRED
  - arti-client > 0.41 : stable 0.41.0 inchange — NOT FIRED
  - wasmtime LTS bump : pas de dep directe — INACTIVE
  - Tor PoW spec, NIST PQC, RFC 9591 erratum : NOT FIRED

  **0 trigger actif.** Pas de pre-research supplementaire requise.

- **Technologies utilisees S36** : toutes deja dans le workspace
  (axum 0.8, rusqlite 0.36, tokio 1.x, nexus-core-rs types). Pas
  de nouvelle dep introduite ce sprint. Patterns DaemonHttpState +
  Arc<Mutex<>> deja utilises (pow_policy RwLock, blob_serve_cache
  Arc). rusqlite WAL mode deja configure dans CoordinatorDb::open().

- **ROADMAP_COMMITMENTS check** (G7 Regle 3) :
  LT-1 Gini trigger, LT-2 Radicle, LT-3 app ecosystem : tous
  requierent tag v1.0 → aucun declenche.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 35 CLOSED. 3 phases A-C livrees + Phase D wrap-up :
- Phase A : 3 MANDATORY 3/3 fermes + crate nexus-coordinator-rs
  fondation (types + db schema v1 + 10 tests)
- Phase B : dispatcher Rust natif TaskDispatcher::submit +
  endpoint axum POST /api/v1/tasks/submit (+6 tests)
- Phase C : validator Rust natif ResultValidator::validate
  signature Ed25519 + task status guard (+5 tests)

Audit gate S35 : **PASS** (0 P0/P1, 2 P2 confirms
[open_in_memory per-request + iroh pre-release transitives],
1 P3 [cross-daemon sans cross-fetch]).

### §1.2 Ancrage HARDENING_ROADMAP

last_validated : 2026-04-28 (S34 Phase D). 0 trigger ACTIF.
Prochain trigger possible : iroh 0.99 quand publie.

### §1.3 Compteurs tests entree (tip `3013e44`)

| Suite | Count |
|---|---|
| Rust nextest | 924 |
| Rust doctests | 0 pass (1 ignored) |
| SDK pytest | 195 (1 flaky Windows file-lock) |
| Coord pytest | 409 + 36 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 267 |
| Playwright | 42 + 2 fail (env pre-existing) |
| size-limit | 7/7 |
| **Total** | **~1927** |

### §1.4 Pre-launch protocol policy (rappel)

Pas de deploiement live. `*_FORMAT_VERSION` restent a 1.
Pas de tolerant decoder multi-version. Cf. CLAUDE.md.

---

## §2 Goal en une phrase

Le sprint **integre le coordinator-rs dans le daemon** en rendant
la DB persistante dans DaemonHttpState, en ajoutant le endpoint de
soumission de resultats, et en portant le KudosLedger en Rust natif,
tout en absorbant la dette sprint pair obligatoire.
**Critere SMART : 28+ rows fail-fast verts au verification.md,
mesure binaire au Phase D wrap-up.**

---

## §3 Phase 0 — Audit gate Sprint 35

**DONE** — `3013e44`. Verdict PASS (0 P0/P1, 2 P2 + 1 P3).
Cf. `.planning/active/sprint35_audit_findings.md`.

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — CoordinatorDb persistent dans DaemonHttpState (singleton partage)

**Retenu** : ajouter un champ `coordinator_db: Arc<Mutex<CoordinatorDb>>`
dans `DaemonHttpState`. La DB est ouverte au boot du daemon depuis le
fichier `~/.sbfb/coordinator.db` (chemin deja defini S35). WAL mode
active pour permettre les lectures concurrentes pendant les ecritures.
Le dispatcher et le validator partagent la meme instance DB via le
state axum. Cela resout P2-REVIEW-B-1 (open_in_memory per-request)
et P2-REVIEW-A-2 (double-open).

**Rejete** :
- Connection pool (r2d2/deadpool) : overkill pour un processus local
  single-writer qui ne depasse pas 10 req/s en loopback.
- DB separee par handler : casse la coherence dispatcher↔validator
  (le validator doit lire les taches inserees par le dispatcher).
- Channel actor pattern : complexite async excessive pour un SQLite
  local. Le Mutex suffit car les operations sont rapides (~1 ms).

**Implications code** : `crates/nexus-shell-daemon/src/http.rs`
(DaemonHttpState + handlers), `crates/nexus-shell-daemon/src/main.rs`
(boot). `crates/nexus-coordinator-rs/src/db.rs` (open() avec chemin
fichier + WAL mode).

### D2 — Endpoint result submission HTTP (pas LiveEvents loop)

**Retenu** : nouveau endpoint `POST /api/v1/results/submit` dans le
daemon axum. Le handler recoit un `ResultEntry` JSON, appelle
`ResultValidator::validate()` avec la DB partagee, retourne le
verdict. Le validateur s'execute synchroniquement dans le handler
(meme pattern que le dispatcher S35 Phase B). C'est le chemin
principal pour S36 : les workers soumettent les resultats via HTTP.

**Rejete** :
- Validator loop tokio (iroh LiveEvents subscription) : requiert
  l'exposition du Doc handle depuis CuratorRuntimeHandle — refactor
  significant du runtime hors scope. Carry S37 (P2-REVIEW-C-1).
- gRPC/tonic : overkill pour loopback, l'API REST axum est le
  standard du projet.
- Validation asynchrone (queue → background task) : complexifie
  l'API client sans benefice pour un processus local.

**Implications code** : `crates/nexus-shell-daemon/src/http.rs`
(+route +handler), `crates/nexus-coordinator-rs/src/validator.rs`
(inchange, deja pret).

### D3 — KudosLedger Rust natif (port fidele)

**Retenu** : port fidele du ledger Python
(`packages/nexus-coordinator/src/nexus_coordinator/kudos.py`, ~344
LOC) en Rust dans `crates/nexus-coordinator-rs/src/kudos_ledger.rs`.
La table `kudos` est deja creee dans coordinator.db (S35 Phase A).
Le ledger expose `credit()` (appele par le validator apres
`Accepted`) et `get_project_kudos()` (query lecture). Endpoint
`GET /api/v1/kudos/{project_id}` dans le daemon.

**Rejete** :
- Kudos en memoire (HashMap) : perte au restart, violait persistence
  ledger hash-chain (Day 0 decision #7).
- Ledger separe (propre fichier DB) : fragmentait les donnees,
  compliquait les transactions cross-table task→kudos.
- Skip kudos S36 : violait le scope S36 commite en S35 kickoff §7.3.

**Implications code** : `crates/nexus-coordinator-rs/src/kudos_ledger.rs`
(NEW), `crates/nexus-coordinator-rs/src/lib.rs` (+pub mod),
`crates/nexus-shell-daemon/src/http.rs` (+route GET kudos).

### D4 — Phase dette sprint pair absorbe P2 batch

**Retenu** : la Phase A dette (obligatoire sprint pair §6.2.1 Regle
1) absorbe les items de dette accumulee :
- P2-REVIEW-B-1 : DaemonHttpState persistent CoordinatorDb (= D1)
- P2-REVIEW-A-2 : double-open DB refactor (resolu par D1 singleton)
- P2-A-2 : PATTERNS.md aggressive update lesson (doc)
- P2-REVIEW-A-1 : LOC estimations kickoff nettoyage (chore doc)
- HARDENING_ROADMAP last_validated → S35/S36

**Rejete** :
- Fusionner dette dans Phase B : dette = phase dediee (Regle 1).
- Skipper la dette : viole §6.2.1 sprint pair.

**Implications code** : `crates/nexus-shell-daemon/src/http.rs`
(DaemonHttpState refactor), `docs/rust/PATTERNS.md` (update),
`docs/security/HARDENING_ROADMAP.md` (last_validated).

### D5 — Validator loop LiveEvents = scope cut explicit S37

**Retenu** : la subscription tokio aux iroh LiveEvents (pour detecter
les resultats publies par les workers directement dans le DHT, sans
passer par HTTP) est **explicitement differee a S37**. Raison : le
Doc handle iroh vit dans `CuratorRuntimeHandle` et n'est pas expose
pour le coordinator. L'extraction requiert un refactor du runtime
(exposer `Arc<Doc>` ou un channel de LiveEvents). Pour S36, le
chemin HTTP (`POST /api/v1/results/submit`) est suffisant — les
workers soumettent les resultats via HTTP loopback.

**Rejete** :
- Forcer l'extraction du Doc handle dans S36 : scope creep, touche
  CuratorRuntime qui est stable et teste.
- Passer par gossip topics pour les resultats : non-standard, les
  resultats sont des entries iroh-docs (canonical bytes + signature).
- Dual path (HTTP + LiveEvents) S36 : complexite excessive, risque
  de double-validation sur le meme resultat.

**Implications code** : aucune — la decision est une non-action
documentee. Le carry S37 est cree dans §6.

---

**Acknowledged review findings (G1)** :

Scoring : D1 ⚠️, D2 ⚠️, D3 ⚠️, D4 ❌, D5 ⚠️.
Rigor signal G4 satisfait (4 ⚠️ + 1 ❌ sur 5).

D4 ❌ (2 findings) :
- D4-1 P2-REVIEW-A-2 "double-open" non trouve dans audit_findings :
  **clarification** — la source est sprint35_phase_A_review.md +
  verification §5 L82, pas l'audit_findings. Le "double-open" refere
  a l'instanciation multiple de CoordinatorDb (une par handler call)
  au lieu d'un singleton. Resolu par D1 (Arc<Mutex<CoordinatorDb>>
  dans DaemonHttpState). Item source corrige dans §6.
- D4-2 HARDENING last_validated circulaire : **clarification** — S34
  Phase D et S35 se sont deroules le 2026-04-28. Le last_validated
  est la date reelle de S34 Phase D, pas un placeholder. La Phase A
  dette mettra a jour vers "S35/S36" avec le bon tip.

D3 ⚠️ : JCS canonicalization byte-identity Python ↔ Rust non prouvee.
**Decision** : accept — S36 Phase C scope = credit() + query seulement,
PAS le hash-chain cryptographique (scope cut §7.12 "hash-chain append-
only = S37+"). Le hash-chain qui depend de JCS equivalence sera porte
et teste en S37 quand les entry_hash seront computes en Rust. Pour S36,
les inserts dans la table kudos ne computent pas de hash — ils
ecrivent amount + worker_id + task_id directement.

D2 ⚠️ : iroh 0.98 LiveEvents API non researche (Doc handle claim
non prouve). **Decision** : accept — D5 scope-cut S37 est explicite.
Le research iroh LiveEvents sera fait en pre-gel S37 (G8 preflight
scan S1b). Le chemin HTTP est suffisant et prouve pour S36.

D1 ⚠️ : async-SQLite non evalue. **Decision** : accept — le Mutex
blocking est < 1 ms pour des operations SQLite locales, et le pattern
est identique aux 15+ champs Arc<> deja dans DaemonHttpState. Si S37+
ajoute concurrence significative, la migration vers tokio-rusqlite
sera evaluee a ce moment.

D5 ⚠️ : worker submission protocol non defini. **Decision** : accept —
c'est un design doc S37 quand le LiveEvents path sera porte. S36 est
loopback-only (meme machine), le protocol selection ne se pose pas.

---

## §5 Plan Phase outline A..D

### Phase A — Dette pair + DaemonHttpState persistent

**But** : absorber les P2 de dette + rendre la DB coordinator
persistante dans le daemon.
- `CoordinatorDb::open(path)` ouvre `~/.sbfb/coordinator.db` avec
  WAL mode (non `open_in_memory()`)
- Ajouter `coordinator_db: Arc<Mutex<CoordinatorDb>>` a
  DaemonHttpState, initialise au boot daemon
- Refactorer `coordinator_submit_task` handler pour utiliser la DB
  partagee au lieu de `open_in_memory()`
- P2-A-2 PATTERNS.md update (doc)
- P2-REVIEW-A-1 LOC kickoff nettoyage (doc)
- HARDENING_ROADMAP last_validated S35/S36
- Commit : `feat(sprint36): Sprint 36 Phase A — dette pair +
  DaemonHttpState persistent CoordinatorDb`

### Phase B — Result submission endpoint + validator wire

**But** : le daemon recoit et valide les resultats via HTTP.
- Nouveau endpoint `POST /api/v1/results/submit` dans `authed_routes`
- Handler deserialise ResultEntry, appelle
  `ResultValidator::new(db).validate(&entry)` via la DB partagee
- Retourne le verdict (Accepted/Rejected*) en JSON
- Tests integration : submit task → submit result → task completed
- Commit : `feat(sprint36): Sprint 36 Phase B — result submission
  endpoint + validator wire`

### Phase C — KudosLedger Rust natif

**But** : porter le ledger Python en Rust et wirer le credit
post-validation.
- `kudos_ledger.rs` : KudosLedger struct, `credit()` appele par
  le validator apres Accepted, `get_project_kudos()` lecture
- Endpoint `GET /api/v1/kudos/{project_id}` dans le daemon
- Wire validator → kudos : apres `Accepted`, appeler
  `kudos_ledger.credit(project_id, worker_node_id, tokens)`
- Tests unitaires kudos_ledger + integration E2E task → result →
  kudos credited
- Commit : `feat(sprint36): Sprint 36 Phase C — KudosLedger Rust
  natif + wire post-validation`

### Phase D — Wrap-up

- verification.md fail-fast 28+ rows
- sprint37_audit_plan.md
- SPRINT_LOG.md row S36
- CLAUDE.md etat actuel
- HARDENING_ROADMAP.md compteurs + last_validated S36
- Migration active/ → archive/v1.2/
- Commit : `chore(sprint36): Phase D — wrap-up + verification
  + audit plan S37 + migration`

---

## §6 Items carry/dette

### Resolus S36 (plan)

- [x] P2-REVIEW-B-1 dispatcher DB persistent DaemonHttpState : Phase A
- [x] P2-REVIEW-A-2 double-open DB refactor : Phase A (singleton)
- [x] P2-REVIEW-C-2 kudos credit : Phase C (KudosLedger.credit())
- [x] P2-A-2 aggressive update PATTERNS.md : Phase A (doc)
- [x] P2-REVIEW-A-1 LOC kickoff nettoyage : Phase A (chore doc)

### Carries confirmes S37

- [carry] P2-A-1 rand blocker upstream 5+/3 : blocker externe
  inchange (frost-core rand_core 0.6 + iroh stack disjoints). Pas
  de convergence observee. Exemption §6.2.1 blocker externe.
- [carry] P2-B-1-S34 log convergence 3/3 : **MANDATORY S37** —
  3 reports consecutifs (S34→S35→S36). Sera integre dans le plan
  S37 comme phase obligatoire. Design log directory partagee
  daemon+launcher.
- [carry] P2-C-1-S34 .icns macOS 3/3 : **MANDATORY S37** —
  3 reports consecutifs. Exemption possible : blocker externe
  (necessite macOS ou outil tiers, dev env = Windows).
  Re-evaluation S37.
- [carry] P2-REVIEW-C-1 validator_loop tokio 2/3 : carry S37.
  Requiert refactor CuratorRuntimeHandle pour exposer Doc handle.
  = D5 scope cut S37.
- [carry] P2-AUDIT-2 pre-release transitives iroh : condition
  heritee pin 0.98. Re-evaluer a chaque upgrade.

### MANDATORY evalues — DEFER justifie

- P3-grammar executor 3/3+ : **DEFER** — le pipeline Rust natif
  est en cours de construction (S35 fondations, S36 integration).
  Le wiring grammar depend du endpoint inference Rust non encore
  porte. Exemption §6.2.1 dependance sequentielle interne.
- P3-watermark executor 3/3+ : **DEFER** — meme justification.
  SynthID wiring depend du pipeline inference Rust.

### Long-terme inchanges

- T-NN+2 iframe Rust-wasm (PATTERNS §P34)
- LT-2 Radicle sortie cap G7 (trigger tag v1.0)
- LT-3/LT-4 hors-sprint (post-v1.0)
- LT-5 redundancy persistence (reclassifie S26)

---

## §7 Scope cuts

1. **Migration complete coordinator** — S37+ (S36 = integration +
   kudos seulement, pas OutputFilter/PiiRedactor/CanaryRegistry)
2. **Suppression coordinator Python** — post-migration complete
   (le coordinator Python reste fonctionnel pendant la transition)
3. **OutputFilter/PiiRedactor Rust** — S37+ (tier 2, guardrail)
4. **CanaryRegistry Rust** — S37+ (tier 2, compliance)
5. **Validator loop LiveEvents** — S37 (D5, requiert Doc handle)
6. **CI pipeline multi-OS** — S37+ (inchange)
7. **VPS deployment** — S37+ (inchange)
8. **Code signing macOS** — post-v1.0 (inchange)
9. **P3 grammar/watermark** — post-pipeline Rust (defer justifie)
10. **SDK Python rewrite** — hors-scope (reste Python pour binding)
11. **Kudos debit/stake** — interdit (Day 0 decision #7, non-monnaie)
12. **KudosLedger hash-chain append-only** — S37+ (S36 = credit +
    query seulement, pas la chaine cryptographique complete)

---

## §8 Tracabilite scope (S35 → S36)

| Item S35 NOT | Ou dans S36 |
|---|---|
| Migration complete coordinator | §7.1 scope cut S37+ |
| Suppression coordinator Python | §7.2 scope cut post-migration |
| KudosLedger Rust | §5 Phase C — integre |
| OutputFilter/PiiRedactor Rust | §7.3 scope cut S37+ |
| CanaryRegistry Rust | §7.4 scope cut S37+ |
| CI pipeline multi-OS | §7.6 scope cut S37+ |
| VPS deployment | §7.7 scope cut S37+ |
| Code signing macOS | §7.8 scope cut post-v1.0 |
| P3 grammar/watermark | §7.9 scope cut post-pipeline Rust |
| SDK Python rewrite | §7.10 scope cut hors-scope |

---

## §9 Risk register

| # | Risque | Impact | Mitigation |
|---|---|---|---|
| R1 | Mutex contention sur CoordinatorDb si burst requetes | Low | Operations SQLite < 1 ms, Mutex relache immediatement. Pas de burst realiste en loopback single-client. |
| R2 | WAL mode rusqlite : fichier .wal/.shm residuels apres crash | Low | WAL checkpoint automatique. Fichiers transitoires normaux SQLite. |
| R3 | KudosLedger Rust diverge du ledger Python (double-credit, etc.) | Medium | Tests cross-validation : meme scenario → memes kudos. Python reste le source of truth pendant cohabitation. |
| R4 | coordinator.db corruption si daemon tue brutalement | Low | WAL mode + journal_mode = WAL survivent aux crash. SQLite est crash-safe par design. |
| R5 | Result submission sans endpoint public (workers remote) | Medium | S36 = loopback only (meme machine). Workers remote passent par la route Python existante. Migration endpoint public = post-S36. |
