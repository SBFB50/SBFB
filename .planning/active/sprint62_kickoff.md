# Sprint 62 — Kickoff (sync P2P durable + anti-spam minimal)

**Ecrit** : 2026-05-14 (post-audit gate S61 PASS `d05c41a`).
**Type** : **sprint pair** — phase dette obligatoire (Regle 1 §6.2.1).
**Tip master d'entree** : `d05c41a` (audit findings S61 PASS).
**Phase 0 audit Sprint 61** : **DEJA JOUE** — `d05c41a` PASS
(0 P0, 0 P1, 7 P2, 5 P3). Aucun fix bloquant requis.
**Version archive** : v2.0 — Public Verifiable Protocol Feed.
**Roadmap source** : `.planning/research/public_verifiable_feed_roadmap.md`
Sprint 2 sur 6 (5+1 reserve). **Sprint a haut risque** (sync P2P
= probleme fondamentalement different du gossip live).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-05-13.
  3 triggers evalues (inchanges depuis S61) :

  1. **iroh > 0.98** : `cargo search iroh` retourne `iroh = "1.0.0-rc.0"`.
     Trigger ACTIF mais RC pre-release, pas stable.
     **Decision** : deja evalue et defere S60-S61. Rester sur iroh 0.98.
     Upgrade iroh 1.0 stable = sprint dedie post-feed.

  2. **arti-client > 0.41** : `cargo search arti-client` retourne
     `arti-client = "0.42.0"`. Trigger **ACTIF** mais non cable.
     **Decision** : evalue et defere. Upgrade arti-client 0.42 = sprint
     dedie post-feed. 0 CVE signale entre 0.41 et 0.42.

  3. **wasmtime LTS bump** : `wasmtime = "44.0.1"`. Pas une dep
     directe. Informatif. Pas d'action.

- **context7 iroh-docs** : `/websites/rs_iroh-docs_iroh_docs` consulte.
  API confirmee : `import_and_subscribe()` retourne `(Doc, Stream<LiveEvent>)`,
  `LiveEvent::InsertRemote` donne namespace + entry + from + should_download.
  Range-based set reconciliation pour sync efficace. DocTicket contient
  namespace key + peers. Modele multi-writer CRDT par (namespace, author, key).

- **Codebase audit** (agent Explore 5 scans) :
  - `nexus-core-rs/src/docs.rs` : DocsClient wrapper mature avec
    `import_ticket()`, `share_write()`, `subscribe()`, `set()`,
    `get_many_latest_per_key_prefix()`. Toutes les primitives necessaires.
  - `nexus-shell-daemon/src/storage_api.rs` : pattern AppStorage P2P
    (StorageNamespaceState, DocTicket share/join, `spawn_storage_subscribe()`
    sur `LiveEvent::InsertRemote`, version AtomicU64 increment). Precedent
    direct reutilisable pour le feed.
  - `nexus-test-harness/tests/multi_daemon.rs` : DaemonCluster avec
    `test_cross_daemon_storage_sync()` — pattern ticket→join→poll→verify.
    Gate `SBFB_INTEGRATION=1`.
  - Anti-spam : pow.rs (HashcashChallenge 18 bits, topic+pubkey bound),
    browse_limiter.rs (GCRA 10/min per peer), storage_limiter.rs (GCRA
    10/min per author-app), quarantine_queue.rs (SQLite TTL 900s). Toutes
    primitives matures mais aucune cablée au feed hot path.
  - `public_feed.rs` + `feed_materializer.rs` : FeedStore insert/replay/
    verify_chain + FeedMaterializer materialize_incremental(cursor). Points
    d'integration identifies pour sync.

- **S61 audit findings P2 critique pour sync** :
  - F2 P2-INCREMENTAL-NO-VERIFY : materialize_incremental ne verifie pas
    la chain quand cursor matche → re-carry obligatoire AVANT sync P2P
  - F3 P2-VALIDATION-STRICTE : validate_feed_operation ne verifie pas
    formats (hex, URL, reason)
  - F4 P2-TRANSACTION-ATOMIQUE : get_last_hash + insert sans transaction
  - F6 P2-SPEC-TRUST-CONTRACT : spec ne documente pas trust model

- **ROADMAP_COMMITMENTS check (G7 Regle 3)** :
  - LT-1 Kudos-v2 : **CLOSED S59**.
  - LT-2 Radicle : **trigger PENDING** — tag v1.0 pose localement
    mais pas pousse vers origin. Push prevu Sprint 65 (go-live).
  - LT-3/LT-4/LT-5 : latent. 0 condition declenchee.
  - LT-6 : RESOLVED S32.
  - LT-7 : **gate satisfait** (Tier 1+2 S55 + Tier 3 S60).

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 61 CLOSED + audit PASS (`d05c41a`). Premier sprint du roadmap
post-v1.0 livre : spec executable `PUBLIC_FEED_SPEC.md`, types Rust
`PublicFeedOperation` (2 variants), `FeedStore` SQLite M9 append-only
hash-chain BLAKE3, `FeedMaterializer` avec `PublicRegistryView` et
cursor persistant M10, tests adversariaux. Feed local rejouable
operationnel, single-writer.

Le reseau SBFB peut desormais enregistrer des evenements protocolaires
signes dans un feed local. Ce qui manque : la sync entre noeuds. Un
noeud B ne peut pas encore recevoir le feed du noeud A, le verifier,
et reconstruire la meme vue registre.

### §1.2 Ancrage HARDENING_ROADMAP

HARDENING_ROADMAP last_validated 2026-05-13. 3 triggers evalues
(voir §Sources ci-dessus) : iroh 1.0.0-rc.0 defere, arti-client
0.42 defere, wasmtime informatif. Aucune action requise pour S62.

### §1.3 Compteurs tests entree (tip `d05c41a`)

| Suite | Compte |
|---|---|
| Rust nextest | 1282 |
| Rust doctests | 0 pass, 1 ignored |
| Vitest | 258 |
| Playwright | 0 (global-setup fail pre-existant S50) |
| size-limit | 6/6 |
| **Total** | **~1546** |

### §1.4 Post-launch protocol policy

**Le tag v1.0 est pose.** La politique post-v1.0 s'applique :

- Chaque break du format bump `*_FORMAT_VERSION`
- Chaque decoder accepte un range de versions
- Chaque ajout de champ porte un `#[serde(default)]` pour compat
  ascendante
- `FEED_FORMAT_VERSION` reste a 1 (pas de breaking change S62 —
  les modifications du store sont internes, pas du wire format)

---

## §2 Goal en une phrase

Deux noeuds SBFB synchronisent le feed public via iroh-docs : un
noeud qui revient apres une periode offline rattrape l'historique,
les operations sont verifiees individuellement (Ed25519 + hash-chain
per-auteur), et le hot path feed est protege par PoW + rate-limit
— posant les bases pour la verification tiers Sprint 63.

**Critere SMART : toutes les rows fail-fast du verification.md
vertes, mesure binaire au Phase D wrap-up.** Le verification.md
§Fail-fast checklist est le critere mesurable du sprint.

---

## §3 Phase 0 — Audit gate du sprint precedent

Sprint 61 audit PASS (`d05c41a`). 0 P0, 0 P1, 7 P2, 5 P3.
Les 4 P2 critiques pour sync (F2-F4, F6) sont resolus en Phase A
dette. Les 3 P2 informatifs (F1 version-not-stored, F5 iroh-infra-
timeout, F7 plan-delta) sont carries. Sprint 62 Phase A demarre
directement.

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — Feed sync transport : iroh-docs namespace

**Retenu** : un namespace iroh-docs dedie au feed public par noeud.
Chaque coordinateur stocke ses feed entries dans le namespace avec
le schema de cles `feed/{author_hex}/{seq_zero_padded}`. La valeur
est le `FeedEntry` serialise en JSON.

Le partage utilise `DocTicket` (write capability). Un noeud rejoint
le feed d'un autre via `docs_client.import_ticket()` + `subscribe()`.
Les `LiveEvent::InsertRemote` sont consommees par un handler qui
verifie et insere dans le feed local SQLite.

Pattern : identique a AppStorage (`storage_api.rs` S58) — le feed
est une "app repliquee" specialisee avec verification cryptographique.

**Rejete** :
- Gossip-only : live-only, pas de catch-up apres offline. gossip =
  notification ephemere, pas de log persistant. L'historique serait
  perdu a chaque deconnexion.
- Custom protocol sur iroh-net : complexite inutile. iroh-docs gere
  deja la reconciliation range-based, la deduplication, et la
  persistence. Reimplementer = 2000+ LOC vs ~200 LOC de glue.
- iroh-blobs direct : pas de semantique key-value, pas de subscribe
  par entree, pas de reconciliation automatique.

**Implications code** :
- `crates/nexus-shell-daemon-core/src/feed_sync.rs` (nouveau)
- `crates/nexus-shell-daemon/src/http.rs` (endpoints ticket/join feed)
- `crates/nexus-core-rs/src/docs.rs` (reutilise tel quel)

### D2 — Chaines hash per-auteur + merge local

**Retenu** : chaque auteur (coordinateur) maintient sa propre
hash-chain Ed25519/BLAKE3 dans iroh-docs. Les recepteurs verifient
chaque chaine d'auteur independamment. Le feed SQLite local stocke
les entrees de tous les auteurs avec un `seq` autoincrement local
(ordre d'insertion, pas ordre global).

La verification a la reception d'une entree distante :
1. Deserialiser le `FeedEntry` JSON
2. Verifier la signature Ed25519 (canonical bytes JCS + domain
   `DOMAIN_FEED_V1`)
3. Verifier le hash-chain per-auteur : `prev_hash` correspond au
   dernier `entry_hash` connu pour cet auteur (ou genesis si c'est
   la premiere entree)
4. Valider l'operation (`validate_feed_operation()` strict — F3)
5. Inserer dans le SQLite local avec transaction (F4)

Le `PublicRegistryView` materialise est le merge de tous les auteurs,
ordonne par timestamp. Deduplication : meme `(author, op_type,
project_id, timestamp)` → skip.

**Rejete** :
- Chaine hash globale unique : impossible avec des ecrivains
  concurrents. 2 noeuds qui appendrent simultanement produisent
  des chaines divergentes irreconciliables.
- DAG (directed acyclic graph) : surconception pour des chaines
  lineaires per-auteur. Chaque auteur emet ses operations
  sequentiellement (un coordinateur a la fois). Le DAG serait
  necessaire si un meme auteur avait des branches, ce qui n'arrive
  pas dans notre modele.
- Noeud relais central : centralisation. Incompatible avec le
  modele P2P (Day 0 figee #1 : pivot P2P integral).

**Implications code** :
- `crates/nexus-coordinator-rs/src/public_feed.rs` : adaptation
  verify_chain() et insert pour multi-auteur
- `crates/nexus-coordinator-rs/src/db.rs` : tracking per-auteur
  (table ou index)
- `crates/nexus-coordinator-rs/src/feed_materializer.rs` : merge
  multi-auteur dans PublicRegistryView

### D3 — Pipeline anti-spam feed : 4 gates existantes

**Retenu** : pipeline d'admission feed en 4 gates sequentielles,
reutilisant les primitives existantes avec instanciation feed :

1. **Rate-limit GCRA** (keyed `author_pubkey`) : 5 operations/min
   par auteur. Instantiation de `governor::DefaultKeyedRateLimiter`
   (pattern `storage_limiter.rs`).
2. **PoW Hashcash** : chaque operation porte une preuve PoW
   (`HashcashChallenge` avec topic="public-feed", publisher=auteur).
   Verification stateless (`pow.rs:verify()`). 18 bits = ~100ms.
3. **Validation stricte** : `validate_feed_operation()` durci (F3)
   — format hex, URL valide, reason enum fermee.
4. **Signature Ed25519** : verification obligatoire avant insertion.

La quarantine queue et l'age witness gate sont scope cuts Sprint
63-64 (les 2 premieres gates + validation + signature suffisent
pour un reseau de 2-3 noeuds pilotes).

**Rejete** :
- Anti-spam complet d'emblee : quarantine + age witness + reputation
  = 3 gates supplementaires pour un reseau de 2-3 noeuds. Over-
  engineering. Le pilote externe (Sprint 65) revelera les besoins
  reels.
- Zero anti-spam : inacceptable meme pour 2-3 noeuds — un noeud
  compromis pourrait spammer le feed de milliers d'operations.
  Rate-limit + PoW = protection minimale credible.
- PoW seulement sans rate-limit : un attaquant GPU pourrait generer
  des preuves plus vite que prevu. Le rate-limit GCRA est le
  backstop.

**Implications code** :
- `crates/nexus-shell-daemon-core/src/feed_limiter.rs` (nouveau —
  GCRA keyed author pour feed)
- `crates/nexus-coordinator-rs/src/public_feed.rs` : champ
  `pow_proof` optionnel dans FeedEntry (ajout de champ
  `#[serde(default)]` — pas de bump FEED_FORMAT_VERSION, c'est
  un ajout backwards-compatible)

### D4 — Phase dette A : S61 P2 critique + carry 2/3

**Retenu** : Phase A = dette pair obligatoire (Regle 1) focalisee
sur les 4 P2 audit S61 bloquants pour sync + resolution d'1 carry
2/3 pour eviter escalade 3/3 MANDATORY :

1. **F2 P2-INCREMENTAL-NO-VERIFY** : `materialize_incremental()`
   verifie la signature Ed25519 + hash per-entree sur les nouvelles
   entrees (pas seulement cursor match).
2. **F3 P2-VALIDATION-STRICTE** : `validate_feed_operation()` durci
   — project_id hex 64 chars, repo_url URL valide, commit_sha hex
   40 chars, artifact_hash hex 64 chars, reason string non-vide.
3. **F4 P2-TRANSACTION-ATOMIQUE** : wrap `get_last_feed_entry_hash()`
   + `insert_feed_entry()` dans un `BEGIN IMMEDIATE` / `COMMIT`
   SQLite.
4. **F6 P2-SPEC-TRUST-CONTRACT** : ajouter section §5.1 "Trust
   model" dans `PUBLIC_FEED_SPEC.md` — local (trust DB) vs remote
   (verify everything).
5. **P2-NSIS-UNINSTALL** (2/3) : lister les 3 binaires (launcher,
   daemon, worker) dans la section uninstall du script NSIS.

**Rejete** :
- Reporter F2-F4 a une phase ulterieure : ils sont prerequis pour
  Phase B (sync). Les resoudre apres sync = code insecure temporaire.
- Inclure les 3 carries 2/3 dans la dette : surcharge la phase.
  Sprint 62 est a haut risque (sync P2P). 1 carry resolu suffit
  pour reduire la dette S63.
- Dette en Phase B ou C : Phase A = premier code du sprint, moment
  ideal pour durcir les fondations avant de construire dessus.

**Implications code** :
- `crates/nexus-coordinator-rs/src/feed_materializer.rs` (F2)
- `crates/nexus-coordinator-rs/src/public_feed.rs` (F3, F4)
- `docs/protocol/PUBLIC_FEED_SPEC.md` (F6)
- `packaging/windows/installer.nsi` (NSIS)

### D5 — Gate de scission + criteres E2E

**Retenu** : evaluation gate a la review Phase C. 4 criteres
binaires, tous requis pour que Sprint 62 soit considere complet :

1. **Offline catch-up** : noeud B offline pendant que noeud A
   publie N operations. B redemarre, rejoint, et son
   `PublicRegistryView` converge vers celui de A.
2. **Replay idempotent** : des entrees dupliquees (re-sync) ne
   creent pas de doublons dans le feed local.
3. **2+ noeuds** : test multi-daemon avec DaemonCluster, gate
   `SBFB_INTEGRATION=1`.
4. **Anti-spam hot path** : au moins rate-limit + PoW cables et
   testes (pas necessairement quarantine/age witness).

Si 1 des 4 criteres echoue a Phase C review → **scission** :
Sprint 62 se termine a Phase C, Phase D (anti-spam) et les criteres
restants deviennent Sprint 63. Le plan passe de 6 a 7 sprints.

**Rejete** :
- Pas de gate : le risque sync P2P justifie un checkpoint formel.
  S'engager sur 4 phases sans point de verification = risque de
  sprint entier perdu.
- Gate apres Phase D : trop tard. Si Phase C echoue, Phase D
  (anti-spam) est construite sur du sable.
- Criteres partiels (2/4 suffit) : les 4 sont necessaires. Un
  feed qui sync sans anti-spam ou qui a du anti-spam sans catch-up
  n'est pas deployable.

**Implications code** :
- `crates/nexus-test-harness/tests/multi_daemon.rs` (tests E2E)
- `.planning/active/sprint62_verification.md` (checklist gate)

### Acknowledged review findings (G1)

Scoring : D1 ⚠️, D2 ✅, D3 ⚠️, D4 ✅, D5 ✅.
Rigor signal G4 satisfait (2 ⚠️ sur 5, 0 ❌).

D1 ⚠️ : changelog iroh-docs 0.98 non trouve par WebSearch.
Decision : pas d'ajustement — le codebase AppStorage S58 utilise
l'exact meme API (`import_ticket()`, `subscribe()`,
`LiveEvent::InsertRemote`) contre iroh-docs 0.98 en production
depuis 3 sprints. Le code existant (`storage_api.rs`) EST la
validation. Phase B preflight G8 scan S1b revalidera si besoin.

D3 ⚠️ : Hashcash non compare a Equihash/client-puzzles + taux
5 ops/min non calibre sur modele d'attaque.
Decision : ajustement — (1) Hashcash est deja implemente et teste
dans pow.rs (S19), utilise pour gossip/browse/feed. Evaluer
Equihash = changement codebase-wide, scope Sprint 64 hardening.
(2) 5 ops/min = 5x headroom sur usage normal (~1-2 ops/deploy),
ajustable a runtime (pattern rate_limit_policy.toml hot-reload).
Calibration formelle documentee en Phase D review. (3) Pollution
namespace iroh-docs : le subscribe handler drop les entrees
invalides (bad sig, rate-limited). Les entrees existent dans
iroh-docs mais ne sont jamais materialisees. Cout reconciliation =
risque acceptable 2-3 noeuds. Namespace cleanup → carry S63.

D5 ⚠️ (mineur) : critere "anti-spam hot path" vague.
Decision : ajustement — clarification des 3 deliverables exacts :
(1) `FeedRateLimiter` instancie dans subscribe handler, (2) test
prouvant > 5 ops/min rejete, (3) `pow_nonce: Option<u64>` champ
dans FeedEntry avec `#[serde(default)]`.

---

## §5 Plan Phase outline A..D

### Phase A — Dette pair obligatoire (S61 P2 critiques + NSIS)

Durcissement feed store pour multi-auteur. 5 items : F2 incremental
verify per-entry, F3 validation stricte formats, F4 transaction
atomique SQLite, F6 trust model spec, P2-NSIS-UNINSTALL resolution.
Pas de changement de wire format.

**Commit cible** : `feat(feed): Sprint 62 Phase A — dette pair + feed store durci pour sync P2P`

### Phase B — Feed sync foundation via iroh-docs

Namespace iroh-docs pour le feed public. Schema de cles
`feed/{author_hex}/{seq}`. Publish local entries vers iroh-docs
on insert. Subscribe `LiveEvent::InsertRemote` → verify Ed25519 +
hash per-auteur + validate → insert local. Endpoints HTTP
`/api/daemon/feed/ticket` et `/api/daemon/feed/join`. Boot sequence
avec `boot_feed_namespace()`.

**Commit cible** : `feat(feed): Sprint 62 Phase B — feed sync P2P via iroh-docs + LiveEvent subscribe`

### Phase C — Catch-up offline + multi-daemon E2E (GATE review)

Tests multi-daemon : `test_cross_daemon_feed_sync()` (2 noeuds,
publish → join → observe), `test_feed_offline_catchup()` (B offline
→ A publie → B rejoint → rattrapage), `test_feed_replay_idempotent()`
(re-sync ne duplique pas). Cursor sync persistant. Validation
hash-chain apres sync. **Gate de scission : 4/4 criteres D5.**

**Commit cible** : `feat(feed): Sprint 62 Phase C — multi-daemon feed sync E2E + offline catch-up`

### Phase D — Anti-spam minimal + wrap-up

Wire `FeedRateLimiter` (GCRA 5/min par auteur) + PoW verification
sur le hot path feed sync. Test : operations sans PoW rejetees,
operations au-dela du rate-limit rejetees. Verification + audit_plan
S63.

**Commit cible** : `feat(feed): Sprint 62 Phase D — anti-spam feed PoW + rate-limit + wrap-up`

---

## §6 Items carry/dette

| Item | Compteur S62 | Classification | Justification |
|---|---|---|---|
| P2-A-1 rand blocker upstream | 23+/3 | exemption externe renouvelee | blocker upstream rand 0.9 crate. Pas de resolution possible cote SBFB. Exemption permanente. |
| P2-AUDIT-2 iroh transitives pre-release | herite | exemption externe renouvelee | herite du pin iroh 0.98. Tant qu'on pin 0.98, les transitives restent pre-release. Upgrade iroh 1.0 = sprint dedie. |
| P2-NSIS-UNINSTALL multi-binary | 2/3 → **RESOLU Phase A** | dette pair | 3 binaires dans la section uninstall NSIS. |
| P2-IMAGE-DEP image 0.25 footprint | 2/3 → 3/3 | carry confirme S63 MANDATORY | ~15 transitives tray icon. Pas sur le chemin critique du feed. Devient MANDATORY S63. |
| P2-G-1 exe lock intermittent | reouvert | carry confirme S63 | dev-env intermittent. Monitoring continu. |
| P2-PLAYWRIGHT-REFACTOR | 2/3 → 3/3 | carry confirme S63 MANDATORY | global-setup pyproject.toml. Devient MANDATORY S63. |
| F1 P2-VERSION-NOT-STORED | 1/3 | carry S63+ | version non stockee en DB. Pas bloquant tant que FEED_FORMAT_VERSION = 1. |
| F5 P2-IROH-INFRA-TIMEOUT | 1/3 | carry S63+ | iroh infra tests timeout intermittent (pre-existant). Gate SBFB_INTEGRATION. |
| F7 P2-PLAN-DELTA | process | process | calibration plans futurs (fix inter-phases dans delta). |

**Items a 3/3 MANDATORY** : aucun ce sprint. 2 items passent a
3/3 S63 (P2-IMAGE-DEP + P2-PLAYWRIGHT-REFACTOR) si non resolus ici.

**Regle 1 dette sprint pair** : Phase A est la phase dette
obligatoire — F2/F3/F4/F6 (S61 audit P2) + P2-NSIS-UNINSTALL (2/3).

---

## §7 Scope cuts (10 items)

| # | Item | Sprint cible |
|---|---|---|
| 1 | CuratorVouched operation implementation | Sprint 63+ |
| 2 | BuildQuorumReached operation implementation | Sprint 63+ |
| 3 | Endpoint HTTP verify-release | Sprint 63 (roadmap S3) |
| 4 | Bridge methods getProvenanceRecord/verifyRelease | Sprint 63 |
| 5 | UI proof-chain composant VerificationDetail | Sprint 63 |
| 6 | Quarantine feed (gate d'admission) | Sprint 63-64 |
| 7 | Age witness gate feed | Sprint 63-64 |
| 8 | Go-live public + tag push + pilote externe | Sprint 65 (roadmap S5) |
| 9 | Multi-forge feed sync (>3 noeuds) | Sprint 64+ |
| 10 | Feed format version bump (colonne version DB) | Sprint 64+ (quand FEED_FORMAT_VERSION > 1) |

---

## §8 Tracabilite scope

Deuxieme sprint du roadmap post-v1.0. Le scope S62 vient
directement du roadmap `.planning/research/public_verifiable_feed_roadmap.md`
Sprint 2, avec ajout de la phase dette obligatoire (Regle 1 sprint pair).
Les scope cuts S61 respectes (12/12 confirme audit) : sync P2P et
anti-spam sont maintenant dans le scope S62 comme prevu.

---

## §9 Risk register

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Multi-writer ordering causal (ReleasePublished avant SourceBecameStale sur meme projet) | Medium | Medium | Per-author chains + timestamp ordering dans le materializer. L'ordonnancement est par auteur (garanti sequentiel), pas cross-auteur |
| R2 | iroh-docs sync timing jitter (precedent S55 ±15s) | Medium | Low | Tests E2E avec polling + timeout genereux (30s). Profil nextest slow-timeout. Gate SBFB_INTEGRATION |
| R3 | Feed entry deduplication sur re-sync | Medium | Medium | Deduplication par (author, entry_hash) — le hash inclut tout le contenu. INSERT OR IGNORE SQLite |
| R4 | Anti-spam insuffisant pour reseau ouvert | Low (S62 = 2-3 noeuds) | High (si public) | Rate-limit + PoW = protection minimale. Quarantine + age witness en Sprint 63-64. Pilote ferme avant ouverture publique |
| R5 | Gate de scission declenchee — sprint 7 au lieu de 6 | Medium | Medium | Phase A dette durcit les fondations. Phase B isolee (sync pure). Phase C = checkpoint explicite avant Phase D |

---

## §10 Audit gate pattern — rappel

Phase 0 jouee (Sprint 61 audit PASS). Phase D devra produire
`sprint63_audit_plan.md` pour le prochain sprint. L'audit gate
reste actif a chaque transition de sprint.

---

## §11 Checkpoint de validation

1. Les D1..D5 gelees sont-elles coherentes avec le roadmap
   post-v1.0 Sprint 2 ?
2. Le schema de cles iroh-docs (D1) est-il compatible avec le
   pattern AppStorage existant ?
3. Les chaines hash per-auteur (D2) preservent-elles les
   garanties du feed single-writer S61 ?
4. Le pipeline anti-spam (D3) est-il suffisant pour 2-3 noeuds
   pilotes sans etre du over-engineering ?
5. La phase dette A (D4) resout-elle les P2 critiques pour sync
   sans surcharger le sprint ?
6. Les criteres de gate de scission (D5) sont-ils binaires et
   verifiables ?
