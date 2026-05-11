# Sprint 58 — Kickoff (AppStorage P2P + MANDATORY carries + dette pair)

**Ecrit** : 2026-05-10 (post-audit gate S57 PASS `4cf8bba`).
**Type** : **sprint pair** — phase dette obligatoire
(§6.2.1 Regle 1). 2 items 3/3 MANDATORY a traiter Phase A.
**Tip master d'entree** : `4cf8bba`.
**Phase 0 audit Sprint 57** : **DEJA JOUE** — `4cf8bba` PASS
(0 P0, 0 P1, 1 P2, 2 P3).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-05-10 (0j). 5 fichiers
  security avec triggers_revalidate. 0 trigger actif pertinent pour
  le theme S58. HARDENING_ROADMAP frais (S57). Pas de pre-research
  supplementaire.

- **iroh-docs 0.98 API** (context7 `/websites/rs_iroh-docs_iroh_docs`) :
  API confirmee — `create()`, `open()`, `set_bytes()`, `get_exact()`,
  `get_many()`, `subscribe()` → `LiveEvent::InsertRemote`, `share_write()`
  → `DocTicket`. Notre wrapper `DocsClient` dans `nexus-core-rs/src/docs.rs`
  expose deja toutes ces operations (`set`, `get_exact`,
  `get_many_by_prefix`, `subscribe`, `share_write`,
  `import_and_subscribe`). Pas de gap API.

- **P2P storage research** : `.planning/research/p2p_storage_replication_iroh_docs.md`
  (2026-05-10). iroh-docs confirme comme bon choix (alternatives CRDT
  rejetees : OrbitDB/Automerge/Yjs/GUN.js). Data model ideas/{uuid} +
  votes/{idea_uuid} conflict-free par construction. Anti-spam 3 couches
  (ticket capability + rate-limit per-author + validation applicative).
  Ticket Write dans archive zip (Option A pre-v1.0).

- **ROADMAP_COMMITMENTS check** :
  - LT-1 Kudos-v2 : reclassifie pre-v1.0 (Owner S50, jamais livre
    S50-S57). Le signal de contribution n'est pas consomme par une
    feature S58. Carry continu S59. Acknowledged.
  - LT-2..LT-5 : latent. 0 condition declenchee.
  - LT-6 : RESOLVED S32.
  - LT-7 : Tier 1+2 DONE (S55). PRE-V1.0 obligation satisfaite.
    Tier 3 (N builders, auto-deploy) reste S59+.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 57 CLOSED + audit PASS (`4cf8bba`). 2 apps SBFB
fonctionnelles (Protocol Explorer + Ideas Hub). Bridge 9 methodes.
Storage persistent SQLite M7. E2E multi-noeuds test vert. CI
operationnel (Woodpecker + GHA). 4 docs research S58+.

**Etat technique (tip `4cf8bba`)** :
- Workspace clean, edition 2024, Rust 1.94
- 2 apps SBFB dans examples/ (sbfb-explorer, sbfb-ideas)
- Storage backend : in-memory HashMap + SQLite M7 write-through
  (storage_api.rs, S57 Phase B)
- iroh-docs : 1 namespace projet pour dispatch (runtime.rs:521)
- DocsClient wrapper : set, get_exact, get_many_by_prefix,
  subscribe, share_write, import_and_subscribe (docs.rs)
- Gossip outbox persistent SQLite M6
- Browse rate-limit governor GCRA per-peer
- Blob-serve : CSP sandbox + CORP header (3 fixes security S57)
- sbfb-bridge.js : 3 copies identiques (SHA256 match), sync manuelle
- jittered_republish_duration() : 30-60s range (S55 Phase D)
- INVITE_FORMAT_VERSION = 2 (u16, S55 Phase D)
- retain_recent() : existe sur browse_limiter + rate_limit, jamais
  appele periodiquement

**Carries entrants S58** :

| Item | Compteur | Source |
|---|---|---|
| P2-JITTER-SCOPE | **3/3 MANDATORY** | S55 Phase D |
| P2-INVITE-U16-WIRE | **3/3 MANDATORY** | S55 Phase D |
| P2-RETAIN-RECENT | 2/3 | S56 audit |
| P2-BRIDGE-SYNC | NEW 1/3 | S57 audit P2 |
| P2-A-1 rand blocker upstream | 17+/3 | exemption externe |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |
| AppStorage P2P | NEW pre-v1.0 | decision utilisateur 2026-05-10 |

### §1.2 Ancrage roadmap

S57 a livre les 2 premieres apps SBFB. S58 connecte le storage
des apps au reseau P2P — la premiere feature ou les donnees de
l'utilisateur sont visibles par tous les noeuds.

Roadmap pre-v1.0 (mise a jour 2026-05-10) :
- **S56** : gossip resilience + bridge extensions ✓
- **S57** : Protocol Explorer + Ideas Hub MVPs ✓
- **S58** : AppStorage P2P replication ← **ici**
- **S59** : stabilisation + verified deploy E2E + LT-1 Kudos-v2
  + tag v1.0

AppStorage P2P est le dernier feature bloc pre-v1.0. S59 est
stabilisation pure.

### §1.3 Compteurs tests entree (tip `4cf8bba`)

| Suite | Count |
|---|---|
| Rust nextest | 1232 |
| Rust doctests | 6 passed |
| Vitest | 256 |
| Playwright | 42 + 2 fail (env pre-existant) |
| size-limit | 6/6 |
| **Total** | **~1494** |

**Post-S58 attendu** : ~1510+ (iroh-docs storage tests + jitter
test + retain_recent test + sync E2E).

### §1.4 Pre-launch protocol policy (rappel)

Phases C et D touchent le storage qui utilise iroh-docs. Le
namespace iroh-docs est INTERNE au daemon (pas un wire format
inter-projets). Les cles applicatives (ideas/{uuid},
votes/{idea_uuid}) sont un schema applicatif local, pas un wire
format P2P. Aucun `*_FORMAT_VERSION` a changer. Le
`TASK_FORMAT_VERSION` du dispatch reste inchange.

---

## §2 Goal

Sprint 58 connecte le storage des apps SBFB au reseau P2P via
iroh-docs, fermant le dernier gap fonctionnel pre-v1.0. Les
donnees Ideas Hub (idees + votes) sont visibles par tous les
noeuds du reseau.
**Critere SMART : 26+ rows fail-fast verts au verification.md,
mesure binaire au Phase E wrap-up. Ideas Hub data repliquee
entre 2 noeuds (test E2E). 2 MANDATORY FERMES. retain_recent
appele periodiquement.**

---

## §3 Phase 0 — Audit gate S57

**DEJA JOUE** : commit `4cf8bba` PASS
(0 P0, 0 P1, 1 P2, 2 P3).
Audit findings dans `.planning/archive/v1.2/sprint57_audit_findings.md`.
7 carries documentes pour S58 (cf. §1.1 ci-dessus).

---

## §4 Decisions Day 0 (D1..D4 gelees)

### D1 — AppStorage P2P via iroh-docs namespace dedie

**Retenu** : creer un namespace iroh-docs dedie par app qui
demande la replication. En S58 MVP, seule l'app `sbfb-ideas`
(Ideas Hub) est repliquee. Le daemon cree/ouvre le namespace au
boot et route les operations `storage_set`/`storage_get`/
`storage_list`/`storage_delete` de l'app vers iroh-docs au lieu
du HashMap+SQLite local.

Schema data (conflict-free par construction, cf. research §4) :
- `ideas/{uuid}` : JSON { title, description, created_at }.
  1 auteur par idee (AuthorId).
- `votes/{idea_uuid}` : JSON { timestamp }. Chaque AuthorId =
  1 vote (entries multi-author iroh-docs).
- Tombstones `{ "deleted": true }` / `{ "retracted": true }`
  pour suppression/retrait vote.

Ticket Write embarque dans l'archive zip (Option A du research,
pre-v1.0). Bridge API inchangee cote app (`storage_get/set/list/
delete`). Push event `storage_remote_update` pour notifications
live.

L'ancien backend HashMap+SQLite reste pour les apps qui ne
demandent pas de replication (default). La detection se fait
par nom d'app (hardcode `sbfb-ideas` en S58, generalise S59+
via manifest).

**Rejete** :
- Automerge / Yjs : 2eme stack de replication parallele a
  iroh-docs. Overhead non justifie pour du key-value simple.
  (Automerge = candidat post-v1.0 pour collaborative editing.)
- OrbitDB : JavaScript, stack IPFS orthogonale au reseau iroh.
- GUN.js : stack fragile, pas de binding Rust, localStorage
  5MB limite.
- Namespace partage avec project doc (dispatch) : pollution des
  espaces de noms, impossible de distinguer storage applicatif
  et task dispatch.
- Replication toutes apps immediatement : scope trop large.
  MVP = 1 app. Generalisation S59+ via manifest.

**Implications code** : `storage_api.rs` (routing iroh-docs),
`runtime.rs` (namespace boot + subscribe InsertRemote), helpers
optionnels dans `docs.rs`.

### D2 — MANDATORY JITTER-SCOPE : test unitaire bounds

**Retenu** : test unitaire verifiant que `jittered_republish_
duration()` retourne toujours une Duration dans [30s, 60s].
Test statistique sur 100+ iterations, assertion bounds. Le test
est dans `runtime.rs` tests module (meme fichier que la
fonction).

**Rejete** :
- Test E2E multi-noeuds pour le jitter : overhead massif pour
  tester une fonction pure. Le jitter est deja implicitement
  teste par l'E2E gossip (le timer fonctionne).
- Mocking rand : `gen_range(30..=60)` est deterministe dans ses
  bornes. Un test de bounds sur 100 iterations suffit.

**Implications code** : `runtime.rs` (1 test).

### D3 — MANDATORY INVITE-U16-WIRE : doc PATTERNS §P47

**Retenu** : documenter dans `docs/rust/PATTERNS.md` section
§P47 le changement `INVITE_VERSION → INVITE_FORMAT_VERSION` +
`u8 → u16` (S55 Phase D). Section courte :
- Historique du rename pour coherence avec `TASK_FORMAT_VERSION`
- u16 = range 0-65535, suffisant pour toute la vie du projet
- Pre-launch policy : version = 2, pas de compat multi-version
- Post-v1.0 : bumper a chaque break, decoder accepte un range

**Rejete** :
- Doc dans un fichier separe : un pattern de nommage ne merite
  pas son propre fichier. §P47 dans PATTERNS.md suffit.
- Bumper la version maintenant : aucun changement wire n'a eu
  lieu depuis S55. Le pre-launch protocol policy s'applique.

**Implications code** : `docs/rust/PATTERNS.md` (§P47).

### D4 — Phase dette : retain_recent + bridge sync

**Retenu** :
- **retain_recent housekeeping** : appeler `retain_recent()`
  periodiquement (toutes les 60s) dans la boucle gossip runtime
  pour nettoyer les entries expirees du rate limiter browse.
  Timer dans le `tokio::select!` de la gossip loop.
- **sbfb-bridge.js sync** : script `scripts/sync-bridge-sdk.sh`
  qui copie `web/public/sbfb-bridge.js` vers toutes les apps
  `examples/*/` et verifie SHA256 post-copie. Documente dans
  CLAUDE.md comme etape build.

**Rejete** :
- npm workspace dep : les apps examples ne sont pas des packages
  npm. Un script simple suffit.
- git hook pre-commit : overhead pour un fichier qui change
  rarement (SDK bridge stable depuis S56 Phase C).
- retain_recent dans un spawn separe : inutile. Timer dans
  select loop = plus simple, pas de thread supplementaire.

**Implications code** : `runtime.rs` (retain_recent timer),
`scripts/sync-bridge-sdk.sh` (NEW).

### Acknowledged review findings (G1)

Scoring : D1 ✅, D2 ✅, D3 ✅, D4 ⚠️.
Rigor signal G4 satisfait (1 ⚠️ sur 4).

D1 ⚠️ minor (p2panda/Loro non evalues) : le reviewer note que
`.planning/research/p2p_storage_replication_iroh_docs.md §2` ne
mentionne pas **p2panda** (Rust-native CRDT append-log) ni
**Loro** (Rust-native RichText CRDT). Impact nul sur D1 : les
deux sont des stacks de replication paralleles a iroh-docs, meme
raison de rejet qu'Automerge (2eme couche, overhead non justifie
pour key-value simple). p2panda est un candidat post-v1.0 pour
collaborative editing. Acknowledged, pas d'action S58.

D4 ⚠️ (scope Phase A vs Phase B) : le reviewer questionne si
D4(a)+(b) sont Phase A MANDATORY ou Phase B dette. **Pas
d'ambiguite** : P2-RETAIN-RECENT est 2/3 (pas 3/3 MANDATORY),
P2-BRIDGE-SYNC est 1/3 (NEW). Les 2 items D4 vont en Phase B
(dette pair obligatoire §6.2.1 Regle 1). Seuls D2 (JITTER-SCOPE
3/3) et D3 (INVITE-U16-WIRE 3/3) sont MANDATORY Phase A.

---

## §5 Plan Phase outline A..E

### Phase A — MANDATORY carries (JITTER-SCOPE + INVITE-U16-WIRE)

**But** : fermer les 2 items 3/3 MANDATORY.

- Test `jittered_republish_duration()` bounds [30, 60] ×100
- Doc PATTERNS.md §P47 wire format invite
- Commit : `feat(sprint58): Sprint 58 Phase A — MANDATORY carries
  JITTER-SCOPE + INVITE-U16-WIRE`

### Phase B — Phase dette pair obligatoire

**But** : resoudre la dette P2 accumulee.

- retain_recent timer 60s dans gossip select loop
- `scripts/sync-bridge-sdk.sh` + verification SHA256
- Commit : `feat(sprint58): Sprint 58 Phase B — dette pair
  retain_recent + bridge sync`

### Phase C — AppStorage P2P namespace + migration storage_api

**But** : le storage de sbfb-ideas passe par iroh-docs.

- Creer/ouvrir namespace iroh-docs dedie au boot
- Router storage_set/get/list/delete vers iroh-docs pour apps
  repliquees (detection par nom d'app)
- Ticket Write generation
- Tests Rust (CRUD via iroh-docs, tombstones, read after write)
- Commit : `feat(sprint58): Sprint 58 Phase C — AppStorage P2P
  iroh-docs namespace + migration`

### Phase D — AppStorage P2P live events + sync test E2E

**But** : replication effective entre 2 noeuds.

- Subscribe InsertRemote sur le namespace storage
- Push event `storage_remote_update` via bridge
- sbfb-bridge.js : methode `onEvent()` pour push events
- Test E2E : 2 daemons, noeud A ecrit, noeud B recoit
- Ideas Hub app.js : handler refresh on storage_remote_update
- Commit : `feat(sprint58): Sprint 58 Phase D — AppStorage P2P
  live events + sync E2E`

### Phase E — Wrap-up + verification + audit plan S59

**But** : cloturer le sprint.

- CLAUDE.md : update S58 CLOSED, carries S59
- HARDENING_ROADMAP : update last_validated S58
- verification.md : 26+ fail-fast rows
- sprint59_audit_plan.md : 7+ tracks
- Commit : `chore(sprint58): Phase E — wrap-up + verification +
  audit plan S59`

---

## §6 Items carry/dette

### Carries confirmes S58

- [phase A] **P2-JITTER-SCOPE** 3/3 MANDATORY :
  **ADRESSE Phase A** → CLOSE attendu.
- [phase A] **P2-INVITE-U16-WIRE** 3/3 MANDATORY :
  **ADRESSE Phase A** → CLOSE attendu.
- [phase B] **P2-RETAIN-RECENT** 2/3 :
  **ADRESSE Phase B** → CLOSE attendu.
- [phase B] **P2-BRIDGE-SYNC** NEW 1/3 :
  **ADRESSE Phase B** → CLOSE attendu.
- [carry] **P2-A-1** rand blocker upstream 17+/3 : exemption
  externe.
- [carry] **P2-AUDIT-2** iroh transitives : herite pin 0.98.
- [LT] **LT-1** Kudos-v2 : pre-v1.0, carry S59.

### Carries residuels post-S58

| Item | Compteur S59 | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 18+/3 | exemption |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |
| LT-1 Kudos-v2 | pre-v1.0 | ROADMAP_COMMITMENTS |

---

## §7 Scope cuts

1. **Verified deploy E2E from repos Git separes** — S59
2. **Protocol Explorer F3 avance** (gossip stats) — S59+
3. **Protocol Explorer F4** (tutoriel interactif) — S59+
4. **Ideas Hub F3** (lier repos Git) — S59
5. **Ideas Hub F4** (groupes de travail) — post-v1.0
6. **Ideas Hub F5** (integration reseau) — post-v1.0
7. **Kudos-weighted voting** — S59+
8. **AppStorage Phase 2** (namespace per manifest) — S59+
9. **AppStorage Phase 3** (optimisations, purge) — post-v1.0
10. **LT-1 Kudos-v2 fairness reform** — S59 pre-v1.0
11. **LT-7 Tier 3** (N builders, auto-deploy) — S59+
12. **Ticket Write rotation dynamique** (Option B/C) — post-v1.0

---

## §8 Tracabilite scope (S57 → S58)

| S57 scope cut | S58 disposition |
|---|---|
| AppStorage replication P2P | **Phase C + D** (pre-v1.0) |
| P2-JITTER-SCOPE test 3/3 MANDATORY | **Phase A** |
| P2-INVITE-U16-WIRE doc 3/3 MANDATORY | **Phase A** |
| P2-RETAIN-RECENT housekeeping | **Phase B** dette |
| sbfb-bridge.js sync script | **Phase B** dette |
| Verified deploy E2E | Scope cut S59 |
| Protocol Explorer F3/F4 | Scope cut S59+ |
| Ideas Hub F3 | Scope cut S59 |
| LT-7 Tier 3 | Scope cut S59+ |
| LT-1 Kudos-v2 | Scope cut S59 |

---

## §9 Risk register

| # | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | iroh-docs namespace storage overhead au boot | Low | Low | 1 namespace supplementaire (storage) vs 1 existant (dispatch). Overhead minimal. |
| R2 | Sync latence InsertRemote 5-30s | Medium | Medium | UI affiche "synchronisation en cours". Pre-v1.0 reseau petit, acceptable. |
| R3 | Ticket Write expose dans archive publique | Medium | Low | Pre-v1.0 acceptable (reseau controle). Anti-spam couches 2-3 suffisent. Post-v1.0 : Option B/C. |
| R4 | Tombstones croissance monotone iroh-docs | Low | Low | Ideas Hub = milliers d'entrees max. Post-v1.0 : purge consensuelle. |
| R5 | Dual backend (HashMap local vs iroh-docs) complexity | Medium | Medium | Routing par nom d'app, default = local. Generalisation manifest S59+. |

---

## §10 Audit gate pattern — rappel

Phase 0 S57 jouee (PASS `4cf8bba`). Phase E produira
sprint59_audit_plan.md pour la session fraiche S59.

---

## §11 Checkpoint de validation

1. **D1** : AppStorage P2P via iroh-docs namespace dedie ?
   → oui (wrapper DocsClient existant, pattern project doc
   runtime.rs:521, 1 namespace dispatch + 1 namespace storage)
2. **D2** : JITTER-SCOPE = test unitaire bounds ?
   → oui (fonction pure gen_range(30..=60), test statistique)
3. **D3** : INVITE-U16-WIRE = doc PATTERNS §P47 ?
   → oui (pattern existant §P46, section courte historique+policy)
4. **D4** : Dette = retain_recent timer + bridge sync script ?
   → oui (retain_recent() existe deja, sync = copie + SHA256)
