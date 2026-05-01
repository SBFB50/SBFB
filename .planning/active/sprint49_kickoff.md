# Sprint 49 — Kickoff (coordinator lifecycle → daemon Rust)

**Ecrit** : 2026-05-01 (post-audit gate S48 PASS `ebf14e7`).
**Type** : **sprint impair** — pas de phase dette obligatoire.
**Tip master d'entree** : `ebf14e7`.
**Phase 0 audit Sprint 48** : **DEJA JOUE** — `ebf14e7` PASS
(0 P0, 0 P1, 1 P2, 2 P3).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-05-01 (intra-day,
  meme jour que S48). 0 trigger actif. 11 triggers surveilles
  inchanges. Pas de pre-research.

- **Technologies S49** : iroh-docs 0.98 (deja dans Cargo.toml,
  utilise par nexus-core-rs), dispatcher.rs + validator_loop.rs +
  kudos_ledger.rs (tous existants dans nexus-coordinator-rs), clap
  (deja dans nexus-shell-daemon). Aucune nouvelle dep externe.

- **ROADMAP_COMMITMENTS check** : LT-1 cible S50+. LT-2..LT-5
  latents (tag v1.0 non pose). LT-6 RESOLVED. 0 condition
  declenchee.

- **HARDENING_ROADMAP §3** : pas de ligne S49 prescrite. Pas de
  drift a documenter.

- **Roadmap migration** : `roadmap_v1_migration_rust.md` prescrit
  S49 = "coordinator lifecycle → daemon Rust". Recherche detaillee
  dans `.planning/research/S49_coordinator_rust_migration.md`.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 48 CLOSED + audit PASS. S46-S48 ont porte sur tests
integration, carries resolution, et dette pair. 3 sprints de
derive par rapport a la roadmap migration Python→Rust qui
prescrivait la suppression Python a S45.

**Etat factuel de la migration** (cf. recherche S49) :
- 14 modules metier portes en Rust (dispatcher, validator,
  kudos_ledger, canary_*, pii_redactor, output_filter, etc.)
- Le daemon Rust SERT deja les 54 routes HTTP et SPAWN deja
  le validator_loop (Sprint 38)
- MAIS le coordinator Python RESTE le proprietaire du lifecycle :
  il boot le node iroh-docs project, dispatch les tasks, orchestre
  le pipeline
- ~9400 LOC Python encore actives (coordinator + SDK + app-gov +
  PyO3 bindings)
- ~3165 LOC Python sans equivalent Rust a porter ou supprimer
- Zero gap de dependances Python→Rust — toutes les deps Python
  ont un equivalent Rust en place

**Le daemon est a 90% du chemin** — il manque le wiring lifecycle
(project doc + dispatch loop) et les CLI subcommands pour
remplacer le coordinator Python comme point d'entree.

### §1.2 Ancrage roadmap migration

`roadmap_v1_migration_rust.md` §S49 : "coordinator lifecycle →
daemon Rust — le daemon DEVIENT le coordinator". 3 phases
prescrites : dispatch+validator wire, CLI migration, app-gov
conversion. La recherche S49 precise le scope factuel.

### §1.3 Compteurs tests entree (tip `ebf14e7`)

| Suite | Count |
|---|---|
| Rust nextest | 1186 |
| Rust doctests | 6 passed, 1 ignored |
| SDK pytest | 195 |
| Coord pytest | 264 + 17 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 267 |
| Playwright | 42 + 2 fail (env pre-existant) |
| size-limit | 5/5 |
| **Total** | **~1937** |

### §1.4 Pre-launch protocol policy (rappel)

`*_FORMAT_VERSION` restent a 1. S49 ne devrait pas toucher de
wire format canonical — le dispatch ecrit des TaskEntry dont le
format est deja defini. Pas de tolerant decoder multi-version.

---

## §2 Goal

Absorber le role de coordinator dans le daemon Rust : le daemon
cree/reopen le project doc iroh-docs, dispatch les taches,
valide les resultats via le validator_loop existant, et credite
les kudos — le coordinator Python n'est plus necessaire pour le
core path. Les CLI subcommands coordinator (init, invite,
quarantine, capability) sont portes dans le binaire daemon.
**Critere SMART : 28+ rows fail-fast verts au verification.md,
mesure binaire au Phase C wrap-up.**

---

## §3 Phase 0 — Audit gate S48

**DEJA JOUE** : commit `ebf14e7` PASS (0 P0, 0 P1, 1 P2, 2 P3).
Audit findings dans `.planning/archive/v1.2/sprint48_audit_findings.md`.
5 carries documentes pour S49 (cf. §6 ci-dessous).

---

## §4 Decisions Day 0 (D1..D4 gelees)

### D1 — Project doc lifecycle : le daemon gere le doc iroh

**Retenu** : etendre `runtime.rs start()` pour creer ou reopen
un document iroh-docs projet. Le daemon possede le doc en tant
que coordinator du projet local. Un daemon = un projet coordonne
(single-project). Le doc stocke les TaskEntry dispatches et les
ResultEntry soumis par les workers. L'endpoint `/api/v1/tasks`
existant est adapte pour ecrire dans le doc au lieu de seulement
stocker en SQLite.

Le wiring suit le pattern existant du daemon : le doc est cree
dans `start()`, stocke dans `DaemonHttpState`, et les modules
(dispatcher, validator) y accedent via l'Arc partagé.

**Rejete** :
- Multi-project daemon : un daemon gere N projets simultanement.
  Complexite disproportionnee pour v1.0 — un contributeur qui
  coordonne 2 projets lance 2 daemons (ou un futur mode multi).
- Lazy doc creation : le doc n'est cree qu'au premier dispatch.
  Inelegant — le daemon ne sait pas s'il est coordinator tant
  qu'on n'a pas dispatch. Le mode coordinator devrait etre
  explicite au start.
- Garder la creation doc en Python : contredit la migration.

**Implications code** : `runtime.rs` (start + DaemonRuntime) +
`DaemonHttpState` (champ project_doc) + research iroh-docs
create/open API 0.98.

### D2 — Dispatch pipeline : wiring modules existants

**Retenu** : le dispatch utilise `dispatcher.rs` existant dans
nexus-coordinator-rs. Le daemon spawn un dispatch loop dans
`start()` qui :
1. Lit les taches pendantes depuis la queue (HTTP submission ou
   doc subscription)
2. Ecrit les TaskEntry signes dans le project doc
3. Annonce via gossip (optionnel — les workers subscribed au doc
   voient les entries)

Le validator_loop.rs est DEJA spawned (Sprint 38, runtime.rs:585).
Il recoit les ResultEvent via le broadcast channel existant
(result_event_tx dans DaemonHttpState). Le seul ajout : le gossip
subscription qui forward les result entries du doc vers
result_event_tx.

**Rejete** :
- Re-ecrire le dispatch depuis zero : les modules existent, LOC
  gaspillees.
- Polling au lieu de subscription : latence + gaspillage CPU.
- Dispatch via HTTP entre daemon et coordinator Python : maintient
  le split deux-process.

**Implications code** : `runtime.rs` (spawn dispatch loop) +
wiring `dispatcher.rs` → project doc.

### D3 — CLI migration : clap subcommands dans nexus-shell-daemon

**Retenu** : ajouter les subcommands `init`, `invite`,
`quarantine`, `capability` au binaire `nexus-shell-daemon` via
clap derive. La commande `start` existante est enrichie pour
booter le daemon en mode coordinator (avec project doc + dispatch).
Les subcommands delegent aux modules Rust existants (invite.rs,
quarantine_queue.rs, capability_store.rs). `nexus-shell-daemon
start` = ce que fait aujourd'hui `nexus-coordinator start` +
`nexus-shell-daemon start` combines. Un seul process.

**Rejete** :
- Binaire CLI separe `nexus-coordinator-cli` : ajoute de la
  complexite d'installation (2 binaires au lieu de 1).
- Garder la CLI Python : contredit la migration.
- Pas de CLI, tout via HTTP : mauvaise UX pour les operations
  one-shot (init, invite create).

**Implications code** : `cli.rs` (4 nouveaux subcommands) +
`main.rs` (4 handlers) + eventuellement `coordinator_cli.rs`
module dedie.

### D4 — app-gov conversion : defer S50

**Retenu** : la conversion de app-gov en archive HTML est
differee a S50 (sprint de suppression Python). S49 se concentre
sur le core path : daemon = coordinator. app-gov reste
fonctionnel via le coordinator Python pendant S49.

**Rejete** :
- Inclure app-gov dans S49 : surcharge le sprint. Le core path
  (dispatch + validator + CLI) est plus critique.
- Supprimer app-gov sans conversion : perte de la vitrine
  gouvernance (19 tabs) sans alternative.

**Implications** : scope cut documente §7.

---

**Acknowledged review findings (G1)** :

Scoring : D1 ✅, D2 ⚠️, D3 ⚠️, D4 ✅.
Rigor signal G4 satisfait (2 ⚠️ sur 4, 0 ❌).

D2 ⚠️ (concurrent doc writes dispatch + HTTP) : le reviewer
signale que le dispatch loop et les endpoints HTTP `/api/v1/tasks`
pourraient ecrire concurremment dans le doc. Decision : le
dispatch loop est le **seul ecrivain dans le doc** — les endpoints
HTTP soumettent les TaskSubmission dans une queue (channel tokio),
le dispatch loop drain la queue et ecrit sequentiellement. Pattern
MPSC existant dans le daemon (cf. result_event_tx broadcast). Pas
de contention doc.

D3 ⚠️ (CLI subcommands offline vs online) : le reviewer signale
que les subcommands init/invite/quarantine/capability necessitent
un acces DB, et la question offline (direct DB open) vs online
(HTTP client vers daemon running) n'est pas tranchee. Decision :
les subcommands one-shot (init, invite create, invite list)
operent en mode **offline** — ils ouvrent le CoordinatorDb
directement, sans daemon running. Seul `start` lance le daemon.
Pattern identique a `nexus-worker register` qui ecrit dans la DB
worker sans daemon.

---

## §5 Plan Phase outline A..C

### Phase A — Coordinator lifecycle dans le daemon

**But** : le daemon cree/reopen le project doc iroh-docs au
demarrage, spawn le dispatch loop qui ecrit les TaskEntry dans
le doc, et forward les result events du doc vers le validator_loop
existant (result_event_tx). Le coordinator Python n'est plus
necessaire pour le core path dispatch → validate → kudos.

- Project doc creation/reopen dans runtime.rs start()
- DaemonHttpState enrichi avec project_doc handle
- Dispatch loop spawn (TaskEntry → doc write)
- Doc subscription → result_event_tx forward
- Integration tests : dispatch → doc write → result → validate →
  kudos credit (E2E pipeline test)
- Commit : `feat(sprint49): Sprint 49 Phase A — coordinator
  lifecycle in daemon dispatch + validator doc wiring`

### Phase B — CLI migration coordinator subcommands

**But** : les operations coordinator (init project, gestion
invites, quarantine, capabilities) sont accessibles via le
binaire daemon unique. `nexus-shell-daemon start` boot le
daemon+coordinator. Les subcommands delegent aux modules Rust.

- 4 subcommands clap : init, invite, quarantine, capability
- Handlers dans main.rs ou coordinator_cli.rs
- Tests unitaires pour chaque subcommand (parsing + delegation)
- Commit : `feat(sprint49): Sprint 49 Phase B — CLI coordinator
  subcommands init + invite + quarantine + capability`

### Phase C — Wrap-up

- Verification fail-fast 28+ checks
- sprint50_audit_plan.md
- Compteurs tests
- Commit : `chore(sprint49): Phase C — wrap-up + verification +
  audit plan S50 + counters`

---

## §6 Items carry/dette

### Carries confirmes S49

- [carry] **P2-A-1** rand blocker upstream 12+/3 : exemption
  blocker externe. Justification renouvelee : pas de release rand
  0.9 ni fix getrandom upstream.
- [carry] **P2-AUDIT-2** pre-release transitives iroh : herite
  pin 0.98 (Day 0 #3).
- [carry] **P2-REVIEW-A-1-S48** canary reload size cap 1/3 :
  NEW S48. Pre-v1.0 fichier local controle par operateur, risque
  accepte. S50+ si temps.
- [carry] **P2-REVIEW-B-1-S48** auth.rs set_var residuel 1/3 :
  NEW S48. 4 set_var dans auth.rs avec pattern save/restore. Non
  bloquant (nextest process-per-test). S50+ si refactoring
  sbfb_home s'etend.
- [carry] **P2-AUDIT-A-1-S48** carry doc accuracy reload_policy
  1/3 : NEW S48 audit. Asymetrie lock scope documentation. Non
  bloquant.

### Sprint impair

S49 impair → pas de phase dette obligatoire (§6.2.1 Regle 1).
0 item a 2/3 ou 3/3. Aucune escalade §6.2.1 Regle 2.

---

## §7 Scope cuts

1. **app-gov conversion archive HTML** — S50 (D4 retenu defer)
2. **events.py SSE streaming port** — S50 Phase C
3. **MCP server migration Rust** — S50 Phase C (evaluer si
   critique v1.0)
4. **PyO3 bindings removal** — S50 Phase A (DELETE bulk)
5. **Suppression complete coordinator Python** — S50 Phase A
6. **Suppression SDK Python** — S50 Phase A
7. **CI/CD + binaires + installer** — S51
8. **VPS deployment + smoke test** — S52
9. **Kudos debit/stake** — interdit (Day 0 #7)
10. **Test infra mk_state() refactoring** — S50+ (hors core path)
11. **Pagination SQL-side LIMIT/OFFSET** — S50+
12. **auth.rs set_var cleanup** — carry S49, non adresse S49
    (scope S50+ si sbfb_home s'etend)

---

## §8 Tracabilite scope (S48 → S49)

| S48 scope cut | S49 disposition |
|---|---|
| events.py SSE streaming — S49+ | Scope cut reporte S50 |
| App runtime migration Rust — S49+ | **Phase A + B** (D1+D2+D3) |
| MCP server migration Rust — S49+ | Scope cut reporte S50 |
| PyO3 bindings removal — S49+ | Scope cut reporte S50 |
| Suppression complete coordinator Python — S49+ | Scope cut reporte S50 |
| CI/VPS/v1.0 — S49+ | Scope cut reporte S51+ |
| Kudos debit/stake — interdit | Day 0 #7 |
| Pagination SQL-side — S49+ | Scope cut reporte S50+ |
| Test infra mk_state() — S49+ | Scope cut reporte S50+ |
| auth.rs set_var cleanup — S49 carry | Carry non adresse S49, S50+ |

---

## §9 Risk register

| # | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | iroh-docs 0.98 API create/open semantique incomplete pour single-writer coordinator | Medium | High | Tests integration pre-commit, fallback create-if-not-exists pattern |
| R2 | Dispatch loop contention avec les HTTP endpoints sur le coordinator_db mutex | Low | Medium | Les 2 acces sont sequentiels (dispatch ecrit, HTTP lit), pas de contention reelle |
| R3 | CLI subcommands init/invite necesitent un daemon running (pour acceder au DB) | Medium | Low | Mode CLI direct (open DB sans daemon) ou mode client HTTP vers daemon running |
| R4 | Le coordinator Python ne peut plus coexister avec le daemon en mode coordinator (port conflict / doc conflict) | Medium | Medium | Le daemon en mode coordinator remplace le coordinator — pas de coexistence. Documentation claire |
| R5 | Tests E2E pipeline (dispatch → validate → kudos) fragiles (iroh node boot lent dans tests) | High | Low | Reutiliser mk_state() existant qui boot un iroh node, timeouts genereux |

---

## §10 Audit gate pattern — rappel

Phase 0 S48 jouee (PASS `ebf14e7`). Phase C produira
sprint50_audit_plan.md pour la session fraiche S50.

---

## §11 Checkpoint de validation

1. **D1** : daemon single-project vs multi-project ?
   → single-project (simplicite v1.0, un daemon par projet)
2. **D2** : re-ecrire dispatch vs wirer modules existants ?
   → wirer (dispatcher.rs + validator_loop.rs existent)
3. **D3** : CLI dans daemon vs binaire separe ?
   → dans daemon (1 binaire, UX simple)
4. **D4** : app-gov S49 vs S50 ?
   → S50 (focus core path S49)
