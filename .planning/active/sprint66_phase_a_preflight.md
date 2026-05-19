# Sprint 66 Phase A — preflight G8

Date : 2026-05-19 | HEAD : `276173a` | Verdict : **EXECUTE plan-as-is**

## Step 0 — G1 pre-condition

Phase A : design review `sprint66_design_review.md` present dans
`.planning/active/`. Scoring : D1 ok, D2 warning, D3 ok, D4 ok,
D5 ok. Rigor signal G4 satisfait (1 warning sur 5).

## Memory consultation (Step 1.5)

- **feedback_approach.md** : "pick deepest technical option",
  "research BEFORE code", "G8 = procedural pick-deepest". Phase A
  est une activation de l'API officielle iroh (pas un recode custom)
  — conforme au principe "lib robuste > from scratch".
- **feedback_context7_systematic.md** : context7 obligatoire avant
  code touchant lib/API/spec. iroh-blobs et iroh-docs ont ete
  queries au kickoff (2026-05-19) et re-confirmes dans ce preflight.
- **nexus_grid_pivot.md** : iroh 0.98 pinne, iroh-blobs 0.100.
  Decision actee #3 "iroh 0.98 pinne post-S32". Phase A ne bumpe
  aucune version iroh.
- **vision_model.md** : pas de tension (Phase A = infrastructure
  interne, pas de funding/startup pattern).
- Tensions plan vs memory : aucune.

## S1a — OSS prior art deep analysis

### Probleme fonctionnel exact

"How do mature P2P projects implement conditional persistent vs
in-memory blob storage with shutdown cleanup?"

### Projets analyses en profondeur

#### [iroh-blobs] — n0-computer/iroh-blobs (https://github.com/n0-computer/iroh-blobs)
- Fichiers source lus :
  - `src/store/fs.rs` (~LOC examinees via WebFetch raw GitHub) : `FsStore` struct,
    `FsStore::load(path)` API, `Deref<Target=Store>` impl, `shutdown()` async
  - `src/store/mem.rs` (~LOC examinees via WebFetch raw GitHub) : `MemStore` struct,
    `Deref<Target=Store>` impl
  - `DESIGN.md` : architecture hybride (small blobs inline redb, large blobs filesystem),
    crash consistency model (no fsync on every write, copy-on-write redb)
  - `examples/transfer.rs` : `MemStore::new()` + `BlobsProtocol::new(&store, None)`,
    Router shutdown pattern
- Pattern architectural extrait :
  - `Store` est le type concret unifie. `MemStore` et `FsStore` implementent tous deux
    `Deref<Target=Store>`, ce qui permet a `BlobsProtocol::new(&store, None)` de prendre
    indifferemment l'un ou l'autre via coercion `&MemStore -> &Store` ou `&FsStore -> &Store`.
  - `BlobsProtocol::new` prend `&Store` (pas generique, pas trait object) — confirme par
    docs.rs BlobsProtocol page.
  - `FsStore::shutdown()` est REQUIS au teardown pour flush l'etat ephemere vers le disque
    (context7 + DESIGN.md).
  - `FsStore::load(path)` cree ou rouvre le store redb. Path doit etre un repertoire.
- Edge cases geres par iroh-blobs :
  - Crash mid-write : redb est ACID copy-on-write, pas de corruption.
  - Blobs orphelins : GC background via tags (tagged blobs protegees, untagged = deletables).
  - Concurrent access : redb single-process exclusif (1 seul FsStore par directory).
- Patterns abandonnes : aucun visible dans DESIGN.md ou releases (le passage MemStore -> FsStore
  est le chemin naturel documente).
- Verdict : **ALIGNED** — le plan utilise exactement l'API officielle iroh-blobs.

#### [iroh-docs] — n0-computer/iroh-docs (https://github.com/n0-computer/iroh-docs)
- Fichiers source lus :
  - API `Docs::persistent(path)` vs `Docs::memory()` (confirme via context7 /websites/rs_iroh
    queried 2026-05-19 au kickoff)
  - Pattern d'initialisation dans node.rs local (lines 296-304) : le wiring `Docs::persistent`
    vs `Docs::memory()` gate sur `cfg.data_dir.is_some()` existe DEJA dans le codebase
- Pattern architectural extrait :
  - `Docs::persistent(path)` cree ou rouvre une base redb dans `path/docs.redb` +
    `path/default-author`. La persistence est transparente pour les consumers
    (`DocsClient` API identique en mode persistent et in-memory).
  - Le daemon ne passait simplement pas `data_dir` — le code node.rs est ready.
- Verdict : **ALIGNED** — activation d'un chemin existant, pas de nouveau code.

#### [IPFS Kubo] — ipfs/kubo (https://github.com/ipfs/kubo)
- Analyse via WebSearch : Kubo utilise un "flatfs" datastore par defaut (fichiers plats sur
  disque), avec fallback in-memory pour les tests. Le pattern "conditional persistent store
  gate on config" est universel dans les daemons P2P.
- Verdict : **ALIGNED** — pattern identique (config gate, persistent default en prod).

#### [libp2p] — libp2p/rust-libp2p (https://github.com/libp2p/rust-libp2p)
- Analyse via WebSearch : libp2p Rust utilise des "providers" de store (trait-based) avec
  implementations in-memory et filesystem. Le pattern enum ou trait pour abstraire le backend
  est standard.
- Verdict : **ALIGNED** — le choix d'un enum (`BlobStore { Mem, Fs }`) est le pattern Rust
  idiomatique quand `Store` n'est pas trait-object-safe.

#### [SSB (Secure Scuttlebutt)] — ssb-ngi-pointer
- Cite dans le kickoff D3 pour le pattern "log-replay au boot". Pas directement pertinent
  pour Phase A (persistence blob store), mais confirme le pattern "SQLite source-of-truth +
  P2P transport sync au boot" utilise dans les phases C/D.
- Verdict : N/A pour Phase A.

### Tableau comparatif

| Aspect | Plan Phase A | iroh-blobs (officiel) | IPFS Kubo | libp2p |
|--------|-------------|----------------------|-----------|--------|
| Store abstraction | enum `BlobStore { Mem, Fs }` | `Deref<Target=Store>` sur chaque impl | flatfs/badger/memory provider | trait-based providers |
| Initialisation conditionnelle | `if data_dir.is_some()` | `FsStore::load` vs `MemStore::new` | config JSON gate | trait selection |
| Shutdown FsStore | `store.shutdown()` dans `Node::shutdown` | `FsStore::shutdown()` requis | close datastore | cleanup trait method |
| BlobsProtocol interaction | `BlobsProtocol::new(&store, None)` via Deref | identique (`&Store`) | N/A (different protocol) | N/A |
| Crash consistency | redb ACID (iroh-blobs natif) | redb copy-on-write | leveldb/badger WAL | N/A |

### Finding S1a
- Classification : **APPROACH-ALIGNED**
- Evidence :
  - iroh-blobs `FsStore::load` + `Deref<Target=Store>` + `BlobsProtocol::new(&Store)` :
    https://docs.rs/iroh-blobs/latest/iroh_blobs/struct.BlobsProtocol.html
  - iroh-blobs `DESIGN.md` : https://github.com/n0-computer/iroh-blobs/blob/main/DESIGN.md
  - context7 `/n0-computer/iroh-blobs` queried 2026-05-19 : `FsStore::load`, `shutdown()`,
    `sync_db()` confirmes
  - Kickoff sources context7 + WebSearch 2026-05-19 (3 queries iroh-blobs confirmees)
- Impact sur le plan : aucun. Le plan utilise l'API officielle iroh-blobs exactement
  comme documentee.

### Note technique S1a : `Store` n'est PAS un trait

Verification critique (car le plan mentionne un enum au lieu de `Box<dyn Store>`) :
`Store` dans iroh-blobs est un **struct concret** (pas un trait). Les deux backends
(`MemStore`, `FsStore`) implementent `Deref<Target=Store>` pour exposer l'API commune.
`Store` n'est pas trait-object-safe — le choix d'un enum est CORRECTEMENT motive dans
le kickoff D2 "Rejete : Rendre Node generique sur le store type". Le pattern enum est
la solution Rust idiomatique confirmee.

---

## S1b — Deps/libs versions + CVE

### Libs dans le perimetre Phase A

| Lib | Version pinnee | Derniere release | CVE search 2026 | Status |
|-----|---------------|-----------------|-----------------|--------|
| iroh-blobs | 0.100 (workspace) | 0.100.x | WebSearch "iroh-blobs CVE 2026" : 0 resultat specifique | clean |
| iroh-docs | 0.98 (workspace) | 0.99.0 (kickoff G2 scan) | WebSearch "iroh-docs CVE 2026" : 0 resultat specifique | clean |
| iroh | 0.98 (workspace) | 1.0.0-rc.0 (kickoff G2 scan) | Deferred (upgrade = sprint dedie Gate 1) | clean (pin) |
| iroh-gossip | 0.98 (workspace) | 0.98.x | (pas touche Phase A) | clean |
| redb | transitive (via iroh-blobs 0.100) | (embedded) | WebSearch "redb Rust CVE 2025 2026" : 0 resultat specifique | clean |
| serde_json | workspace | (stable, pas bumpe) | (pas touche Phase A) | clean |
| hex | workspace | (stable) | (pas touche Phase A) | clean |

### Specs

| Spec | Status Phase A |
|------|---------------|
| RFC 8785 (JCS) | Non touche — canonical.rs inchange |
| SLSA L1 | Non touche — provenance inchangee |

### Finding S1b
- 0 CVE critique/high sur les deps du perimetre Phase A.
- 0 lib bump necessaire (toutes les deps restent aux versions pinnees workspace).
- iroh-docs 0.99.0 et iroh 1.0.0-rc.0 existent mais sont deferred (Gate 1 Arc 1/2,
  decision gellee kickoff S66 G2).
- redb est une dep transitive de iroh-blobs — pas de CVE rustsec publie en 2025-2026.

---

## S2 — Decision chain reconstruction

### Fichiers scannes

| Fichier | Commits touches | Bodies lus |
|---------|----------------|------------|
| `node.rs` | 9 commits (S1 a S54) | 4 bodies complets (S2 S4, S2 fix shutdown, S18 Phase C, S54 Phase A) |
| `blobs.rs` | 7 commits (S2 a S54) | 2 bodies (S2 S4, S54 Phase A) |
| `runtime.rs` | 40+ commits (S7 a S65) | 5 bodies complets (S53 Phase E, S58 Phase C, S62 Phase B, S54 Phase A, S65 Phase A) |
| `blob_serve.rs` | 15+ commits (S12 a S57) | 1 body (S30 Phase B COOP/COEP) |
| `http.rs` | 50+ commits (S7 a S65) | 1 body (S65 Phase A) |

### Decisions historiques trouvees

#### Decision 1 : MemStore comme store blobs
- Sprint 2, sha `e51123ee` : decision originale
  Body extrait : "Note on the blobs_store dance: `BlobsProtocol::new` borrows the store
  via `&MemStore`, so after the router is built we move the original MemStore handle into
  the Node."
- Commentaire node.rs line 23-25 (actuel) : "Sprint 4 will add: Persistent blob store
  backed by filesystem (we use MemStore now)"
- Reverse-commit check :
  1. `git log --all --oneline e51123ee..HEAD -- crates/nexus-core-rs/src/node.rs` :
     8 commits, aucun ne mentionne "revert/undo/FsStore/persistent blob"
  2. `git log --all --grep="e51123ee" --oneline` : 0 match
  3. Conclusion : pas de reversion. La decision "MemStore now, FsStore later" est ACTIVE et
     Phase A la complete enfin (S66 = "Sprint 4" reference dans le commentaire).
- Status : **active, completion programmee Phase A**
- Impact phase : **aucun** — Phase A REALISE la decision differee, pas la contredit

#### Decision 2 : data_dir wiring pour Docs persistence
- Sprint 4 Phase A (commit implicite, code present node.rs:296-304) : `Docs::persistent(path)`
  vs `Docs::memory()` gate sur `cfg.data_dir.is_some()`. Le wiring existe mais le daemon ne
  passait jamais `data_dir`.
- Sprint 53 Phase E, sha `3cc972ab` : file-based persistent node identity
  Body extrait : "load_or_generate_node_key() lit <root>/node_key s'il existe, sinon genere
  et persiste. Le chemin None utilise maintenant create_node_with_config() avec le secret
  file-backed."
- Reverse-commit check : N/A (decision de deferral, pas de rejet)
- Status : **active, activation programmee Phase A**
- Impact phase : **aucun** — Phase A active le `with_data_dir` manquant

#### Decision 3 : Node::shutdown graceful via Router::shutdown
- Sprint 2 fix, sha `de9589da` : "Replace with `self.router.shutdown().await`, which drives
  the whole graceful sequence itself"
- Status : **active**
- Impact phase : Phase A doit ajouter `blobs_store.shutdown()` AVANT `router.shutdown()`
  pour le flush FsStore. Le plan le mentionne ("A shutdown, appeler store.shutdown() pour
  FsStore"). Coherent avec la decision historique (shutdown explicite > drop implicite).

#### Decision 4 : BlobsClient wraps &MemStore
- Sprint 2 S4 : `BlobsClient::new(inner: &'a MemStore)`
- Impact phase : Phase A change vers `&'a Store`. Pas de reversion needed — c'est une
  generalisation. Aucun consumer utilise de methodes MemStore-specifiques (verifie via grep :
  tous passent par `BlobsClient::new(node.blobs_store())`).

### Memory constraints

| Fichier | Contrainte | Relevance Phase A |
|---------|-----------|-------------------|
| feedback_approach.md | "pick deepest", pas de band-aid | Phase A = activation de l'API officielle, pas un band-aid |
| feedback_context7_systematic.md | context7 obligatoire avant code | Fait au kickoff (3 queries iroh-blobs) + confirme dans ce preflight |
| nexus_grid_pivot.md | iroh 0.98 pinne | Phase A ne bumpe aucune version iroh |
| vision_model.md | no startup patterns | N/A |

### Finding S2
- 0 decision historique contredite.
- Toutes les decisions trouvees sont soit (a) completees par Phase A (MemStore -> FsStore,
  data_dir activation), soit (b) coherentes avec le plan (shutdown explicite).
- Le commentaire node.rs:23-25 "Sprint 4 will add: Persistent blob store backed by
  filesystem" est un engagement historique que Phase A realise avec 62 sprints de retard
  — pas un conflit.

---

## S3 — Threat model analysis

### Primitive analysee : FsStore activation + data_dir persistence

### Assets en jeu

- A1 **redb blobs database** (NEW) : `<root>/iroh/blobs/blobs.db` — contient les archives
  apps (zip) en content-addressed storage. Criticite : **medium** (les blobs sont publics
  et verifiables par BLAKE3 hash, mais un attaquant local pourrait supprimer/corrompre le
  fichier pour causer un DoS).
- A2 **redb docs database** (NEW) : `<root>/iroh/docs.redb` — contient les namespaces
  iroh-docs (feed, storage, project). Criticite : **medium** (meme raisonnement).
- A3 **Keypair Ed25519 node_id** (existant) : `<root>/node_key`. Inchange par Phase A.

### Threat actors

- TA1 **Malware user-mode local** (AD2 THREAT_MODEL.md) : peut acceder aux fichiers redb
  dans le home dir user. Capacite : read/write/delete sur les fichiers du user.
- TA2 **Crash OS / power failure** : perte de courant pendant une ecriture redb.

### Attack vectors identifies

| # | Vecteur | Asset(s) | Couverture T0-T5 |
|---|---------|---------|-----------------|
| V1 | Corruption redb par crash mid-write | A1, A2 | Couvert par design redb (ACID copy-on-write B-tree). Phase B ajoute SQLite FULL pragma comme defense-in-depth sur la DB coord. |
| V2 | Suppression malveillante des fichiers redb | A1, A2 | Partiellement couvert : le daemon detecterait l'absence au prochain boot (erreur FsStore::load). Pas de self-healing pre-launch. |
| V3 | Tampering redb (modification bits) | A1, A2 | Couvert pour blobs : BLAKE3 hash verifie l'integrite au read. Partiellement couvert pour docs : iroh-docs redb est un detail interne, pas de verification applicative supplementaire. |
| V4 | DoS via fichier redb enorme | A1 | Hors scope pre-launch (<100 blobs en pratique). GC background iroh-blobs elimine les blobs untagged. |
| V5 | Concurrent access redb (2 daemons meme dir) | A1, A2 | Couvert par le singleton check registry (`check_stale_or_bail` + running.json). Un seul daemon par root directory. |
| V6 | FsStore::shutdown non appele (crash daemon) | A1 | Risque : etat ephemere non flush. Mitigation : redb est ACID, les donnees commitees sont safe. L'etat ephemere (tags en cours) peut etre perdu — le daemon re-tag au prochain boot. Accepte pre-launch. |
| V7 | Supply chain (dep transitive redb) | A1, A2 | Couvert par P2-AUDIT-2 carry (iroh transitives pre-release heritees du pin 0.98). Pas de nouvelle dep ajoutee par Phase A. |

### Mitigations existantes

- T0 (loopback auth bearer) : non impacte par Phase A.
- T1 (Host + Origin check) : non impacte.
- T2 (singleton daemon) : couvre V5 (concurrent access).
- T3 (CSP iframe) : non impacte.
- T4 (Ed25519 provenance) : couvre V3 partiellement (integrite blobs via BLAKE3).
- T5 (iroh QUIC encryption) : non impacte.

### Gaps identifies

- GAP1 V2 (suppression malveillante redb) : severity **low** pre-launch (pas de noeud
  externe, machine dev seule). Self-healing (re-download depuis pairs) est un objectif
  post-launch. Pas de nouvelle mitigation requise Phase A.
- GAP2 V6 (shutdown non appele) : severity **low** (redb ACID protege les donnees
  commitees). Le plan inclut explicitement `FsStore::shutdown()` dans `Node::shutdown()`.

### Regression check

- La primitive NE diminue PAS l'efficacite d'une mitigation T0-T5 existante.
- La primitive cree 2 nouveaux assets (fichiers redb) mais pas de nouveau vecteur
  non couvert de severity high.
- Aucun nouveau T necessaire.

### Verdict S3 : **clean** (0 regression, 2 gaps low severity documentes)

---

## S4 — Wire format deep audit

### canonical.rs lu integralement : oui

Lu en entier (296 lignes) : 14 domain separation tags (DOMAIN_TASK_V1 a
DOMAIN_FEED_V1), fonction `canonical_bytes<T>()` avec JCS + domain prefix + null
separator, 4 tests. Aucune struct dans canonical.rs — c'est une lib de serialisation
pure.

### Structs verifiees

Phase A ne touche AUCUNE struct wire format. Les changements sont :
- `Node.blobs_store` : champ interne `MemStore` -> enum `BlobStore` (pas serialise)
- `BlobsClient.inner` : `&'a MemStore` -> `&'a Store` (pas serialise)
- `NodeConfig.data_dir` : `Option<PathBuf>` (existant, pas serialise)

Aucune struct avec `#[derive(Serialize, Deserialize)]` n'est modifiee par Phase A.

### Constantes version verifiees

| Constante | Fichier | Valeur | Phase A touche | Status |
|-----------|---------|--------|---------------|--------|
| `TASK_FORMAT_VERSION` | task.rs | 1 | non | ok |
| `CURATOR_LIST_FORMAT_VERSION` | curator.rs | 1 | non | ok |
| `KEY_ROTATION_FORMAT_VERSION` | key_rotation.rs | 1 | non | ok |
| `POW_FORMAT_VERSION` | pow.rs | 1 | non | ok |
| `PIN_FILE_FORMAT_VERSION` | tls_pinning.rs | 1 | non | ok |
| `FEED_FORMAT_VERSION` | public_feed.rs (nexus-coordinator-rs) | 1 | non | ok |

### `serde_json::to_string` audit

Grep `serde_json::to_string` dans `crates/nexus-core-rs/src/` : 8 occurrences,
TOUTES dans des blocs `#[cfg(test)]` (tests unitaires). Aucune utilisation
production hors JCS — conforme a la policy JCS-only pour wire format.

### Day 0 check

D1-D5 sprint 66 :
- D1 iroh-docs persistence via data_dir : **Phase A la realise** — conforme
- D2 iroh-blobs FsStore activation : **Phase A la realise** — conforme
- D3 Feed republish au boot : **Phase C** — non touche Phase A
- D4 Provenance 3 etats MANDATORY : **Phase C** — non touche Phase A
- D5 Verification cross-node MANDATORY : **Phase C** — non touche Phase A

Aucune D1-D5 contredite.

### Decisions actees pivot.md

Verification des decisions actees pertinentes :
1. iroh 0.98 pinne : Phase A ne bumpe pas — ok
2. MemStore -> FsStore : Phase A la realise — ok (note S2 : commentaire
   node.rs:23-25 "Sprint 4 will add persistent blob store")
3. Archive zip format universel : non touche — ok
4. postMessage bridge seul canal : non touche — ok
5. Deploy verifie from source : non touche — ok

Aucune decision actee contredite.

### Pre-launch policy

- `*_VERSION = 1` : toutes les constantes restent a 1 — ok
- Pas de tolerant decoder multi-version : non touche — ok
- Pas de tests "legacy decode" zombie : non touche — ok
- Phase A ne modifie aucun wire format — conformite totale

### Verdict S4 : **clean**

---

## Telemetrie preflight (agent deep)

- S1a : 5 projets OSS analyses (iroh-blobs, iroh-docs, IPFS Kubo, libp2p,
  SSB) / 6 fichiers source lus (FsStore, MemStore, DESIGN.md, transfer.rs,
  BlobsProtocol docs.rs, node.rs local) / ~2000 LOC reviewees / 2 context7
  queries (resolve iroh-blobs + query FsStore) / 7 WebSearch queries / finding :
  APPROACH-ALIGNED
- S1b : 7 libs scannees / 3 CVE searches (iroh-blobs, iroh-docs, redb) /
  finding : clean (0 CVE)
- S2 : 12 commits bodies lus / 0 archive files (Phase A sur infrastructure
  courante, pas d'archive pertinente) / 5 memory files lus (feedback_approach,
  feedback_context7, nexus_grid_pivot, vision_model, MEMORY.md) / finding : clean
  (0 decision contredite)
- S3 : FULL / 7 vectors analyses / 2 gaps (low severity) / finding : clean
- S4 : FULL / 0 structs wire format touchees / canonical.rs lu integralement :
  oui / 6 constantes version verifiees = 1 / 8 serde_json::to_string = all
  #[cfg(test)] / finding : clean

## Action

Proceder code phase A. Aucun finding bloquant. Le plan est aligne avec l'API
officielle iroh-blobs (`FsStore::load`, `Deref<Target=Store>`,
`BlobsProtocol::new(&Store)`, `FsStore::shutdown()`). Le wiring `data_dir` dans
node.rs existe deja (lines 296-304, code Sprint 4 Phase A) — Phase A l'active
cote daemon et ajoute le pendant FsStore pour les blobs.

Points d'attention pour l'implementation (non-bloquants) :
1. `FsStore::shutdown()` doit etre appele AVANT `Router::shutdown()` dans
   `Node::shutdown()` pour eviter que le router abort le flush redb.
2. L'enum `BlobStore` doit implementer `Deref<Target=Store>` pour que
   `BlobsProtocol::new(&blobs_store, None)` continue a compiler via coercion.
3. Le worker (nexus-worker-core) utilise aussi `BlobsClient::new(node.blobs_store())`
   (runtime.rs:864) — le changement de type est transparent car le worker n'utilise
   pas `data_dir` (reste MemStore via l'enum).
4. `FsStore::load(path)` est async — `create_node_with_config` est deja async,
   pas de changement de signature necessaire.
