# Audit de readiness protocole — S66 Durabilite / S67+ Factory+RRV

**Date** : 2026-05-19
**Auditeur** : Claude Opus 4.6 (audit technique independant)
**Tip master** : `3360c45`
**Source de verite** : code reel + tests + compilations, pas les docs de planification

---

## 1. Etat des tests et de la compilation

| Suite | Attendu (memoire) | Observe | Statut |
|---|---|---|---|
| Rust nextest | 1333 | **1333** (0 skip, 0 fail) | CONFORME |
| Vitest | 268 | **268** (0 fail) | CONFORME |
| cargo fmt | 0 diff | 0 diff | CONFORME |
| cargo clippy | 0 warnings | 0 warnings | CONFORME |

Le workspace compile et les compteurs de tests correspondent exactement
a la memoire. Aucune regression detectee.

---

## 2. Sprint 66 — Livrables : FAIT vs PAS FAIT

### Phase A — iroh data_dir + FsStore

| Livrable | Statut | Evidence code |
|---|---|---|
| Enum `BlobStore { Mem, Fs }` dans `node.rs` | **PAS FAIT** | `blobs_store: MemStore` (l.121), aucun `FsStore` importe, aucun enum |
| `node.blobs_store()` retourne `&Store` | **PAS FAIT** | Retourne `&MemStore` (l.159) |
| `with_data_dir` appele dans `runtime.rs` | **PAS FAIT** | `grep "data_dir" runtime.rs` = 0 resultats. Le daemon ne passe jamais `with_data_dir` au `NodeConfig`. |
| `Docs::persistent` utilise dans le daemon | **PAS FAIT** | Le daemon boot avec `NodeConfig::default().with_secret_key(...)` seulement (runtime.rs l.294-295). `data_dir` reste `None` donc `Docs::memory()` est utilise. |
| boot_*_namespace fallback list_docs() | **PARTIELLEMENT EXISTANT** | `boot_storage_namespace` et `boot_feed_namespace` existent (runtime.rs l.1405-1519) et font un create-or-reopen via M8 table, mais utilisent iroh-docs en mode memoire. Les namespaces sont recrees a chaque restart. |
| Tests FsStore persistent | **PAS FAIT** | Aucun test `persistent_fsstore` ou `boot_*namespace.*persistent` |

**Note importante** : `node.rs` a le code `Docs::persistent` (l.296-304)
mais le daemon ne l'active jamais car il ne passe jamais `with_data_dir`.
La capacite existe dans nexus-core-rs mais n'est pas cablee dans le daemon.
Les blobs restent en `MemStore` pur — les archives zip disparaissent au
restart.

### Phase B — Dette pair

| Livrable | Statut | Evidence code |
|---|---|---|
| THREAT_MODEL.md section feed (T-FEED-*) | **PAS FAIT** | `grep "T-FEED" docs/security/THREAT_MODEL.md` = 0 resultats. `grep "feed\|Feed" THREAT_MODEL.md` = 0 resultats. |
| PATTERNS.md pattern raw-op | **PAS FAIT** | `grep -i "raw.op" docs/rust/PATTERNS.md` = 0 resultats |
| `PRAGMA synchronous = FULL` dans db.rs | **PAS FAIT** | `grep "synchronous" crates/nexus-coordinator-rs/src/db.rs` = 0 resultats. L.217 : seuls `journal_mode WAL` et `foreign_keys ON` sont configures. |

### Phase C — Feed republish + provenance cross-node

| Livrable | Statut | Evidence code |
|---|---|---|
| Bloc republish feed SQLite→iroh-docs au boot | **PAS FAIT** | `grep "republish\|replay_all" runtime.rs` = seul le gossip outbox republish periodique (l.1135-1277), pas de replay feed SQLite vers iroh-docs au boot. |
| feed_join handle tracke | **PAS FAIT** | `feed_join` (feed_sync.rs l.624) fait `tokio::spawn(...)` sans stocker le JoinHandle. Fire-and-forget confirme. |
| Provenance 3 etats (absent/verified/failed) | **PAS FAIT** | `get_provenance` (http.rs l.1714-1759) retourne 404 pour `Ok(None)` (l.1745-1749), pas un status "absent". La verification utilise `state.pow_keypair` (cle locale), pas la cle du noeud d'origine — donc verification locale seulement, pas cross-node. |
| useBridge.ts + BrowsedProject.tsx badge 4 etats | **PAS FAIT** | Depend des changements http.rs ci-dessus |

### Phase D — Orphan recovery + RevocationCache persistence

| Livrable | Statut | Evidence code |
|---|---|---|
| Migration M14 `key_rotations` | **PAS FAIT** | `grep -i "key_rotation\|migration.*m14" db.rs` = 0 resultats. Derniere migration = M13 (l.196). |
| Orphan detection boot | **PAS FAIT** | Aucune logique de comparaison SQLite vs iroh-docs au boot. |
| RevocationCache persistence SQLite | **PAS FAIT** | Aucun `load_key_rotations` ou `insert_key_rotation`. |

### Phase E — E2E restart test

| Livrable | Statut | Evidence code |
|---|---|---|
| Test `e2e_restart_full_cycle` | **PAS FAIT** | `grep -i "e2e.*restart" crates/` = 0 resultats |
| Test `e2e_crash_recovery` | **PAS FAIT** | Idem |

### Resume S66

| Phase | Items | FAIT | PAS FAIT |
|---|---|---|---|
| A | 6 | 0 | **6** |
| B | 3 | 0 | **3** |
| C | 4 | 0 | **4** |
| D | 3 | 0 | **3** |
| E | 2 | 0 | **2** |
| **Total** | **18** | **0** | **18** |

**S66 n'a AUCUN code livre. Seuls le kickoff et le plan existent (commit dcd8270).**

---

## 3. Carries S66 — analyse de blocage pour S67

| Carry | Severite | Bloquant pour S67 ? | Justification |
|---|---|---|---|
| P2-PROVENANCE-404-BRIDGE (3/3 MANDATORY) | **3/3** | **OUI** | La synthese identifie `CuratorVouched` et provenance cross-node comme prerequis Factory. Un Factory externe a besoin d'un endpoint provenance qui retourne un statut exploitable, pas un 404 binaire. |
| P2-VERIFY-LOCAL-KEY-ONLY (3/3 MANDATORY) | **3/3** | **OUI** | Verification provenance utilise `state.pow_keypair` (cle locale l.1731-1733). Un noeud qui recoit une provenance d'un autre noeud ne peut pas la verifier. Factory + pilote cross-node impossible sans. |
| P2-FEED-JOIN-HANDLE-LEAK (2/3) | **2/3** | **Non strict** | Le leak est un probleme de proprete (taches orphelines au shutdown). Factory n'appelle pas feed_join directement. Amelioration S66 Phase C mais pas bloquant. |
| P2-ORPHAN-REPUBLISH-RECOVERY (2/3) | **2/3** | **Non strict** | Ameliore la durabilite mais Factory peut fonctionner sans si les blobs sont recrees au deploy. |
| P2-THREAT-MODEL-FEED-SURFACE (1/3) | **1/3** | **Non** | Documentation de securite. Important mais pas bloquant pour du code. |

---

## 4. Prerequis S67 selon la synthese (SYNTHESIS §Annexe E) — statut

| Prerequis | Statut | Detail |
|---|---|---|
| P2-FEED-INSERT-NO-AUTH-TIER fixe | **FAIT** (S65 Phase A) | `x-sbfb-feed-internal` present dans feed_sync.rs l.444 |
| CuratorVouched dans le feed | **PARTIELLEMENT** | Passe comme raw-op (teste l.1673), mais pas de struct typee dans `PublicFeedOperation`. Le plan S66 §11 dit scope cut S67. |
| TRUST_TAXONOMY.md ecrit + UI applique | **FAIT** (S65 Phase B) | `docs/trust/TRUST_TAXONOMY.md` existe. UI migration S65 Phase B confirmee. |
| iroh data_dir cable dans le daemon | **PAS FAIT** | runtime.rs ne passe pas `with_data_dir`. Docs en mode memoire. |
| iroh-blobs FsStore operationnel | **PAS FAIT** | MemStore uniquement. Pas d'enum BlobStore. |
| Feed republish au boot | **PAS FAIT** | Pas de replay feed SQLite→iroh-docs au boot. |
| E2E restart test vert | **PAS FAIT** | Aucun test E2E restart. |

**4 des 7 prerequis echouent.**

La synthese dit explicitement (§Annexe E, l.1951) :
> "Si un de ces items echoue, S67 NE DOIT PAS demarrer."

---

## 5. Primitives daemon manquantes pour S67 — gap dans le code

| Primitive | Route cible | Existe deja ? | Detail |
|---|---|---|---|
| Feed read paginee | `GET /api/daemon/feed/entries` | **PAS D'ENDPOINT** | La DB a `get_feed_entries()` et `get_feed_entries_after_seq()` (db.rs l.765, l.811) mais aucune route HTTP ne les expose. Le feed est un trou noir cote client. |
| Preview ephemere | `POST /api/v1/preview/load` | **NON** | Aucune route preview dans http.rs. Cible S68. |
| CuratorVouched/Disendorsed types | Variants dans `PublicFeedOperation` | **NON** (raw-op seulement) | Commentaire l.52-54 dit "Future variants". Test raw-op passe mais pas de validation semantique. |
| node_id optionnel dans deploy | Modification deploy.rs | **NON** | `SbfbJson` (deploy.rs l.544) a `node_id: String` obligatoire. Le check l.123 rejette si node_id ne match pas. |
| Search FTS5 | `GET /api/daemon/search` | **NON** | `grep "fts5\|FTS5" crates/` = 0 resultats. Aucune migration, aucun index, aucun module. |
| SBFB.json v2 validation | Struct dans sbfb-manifest crate | **NON** | `SbfbJson` actuel = `{ node_id, version? }` (deploy.rs l.544-547). Spec v2 existe dans `docs/protocol/SBFB_JSON_V2.md` mais pas de code. |
| Manifest extraction | `GET /api/v1/project/{id}/manifest` | **NON** | Pas de route. |
| Provenance list | `GET /api/v1/provenance/list` | **NON** | Seul `get_provenance_by_project` existe (1 projet a la fois). |

### Estimation effort pour les primitives S67

| Primitive | LOC estimees | Phase cible |
|---|---|---|
| Feed read paginee (endpoint + pagination) | ~80 | S67-A |
| CuratorVouched/Disendorsed (struct + validation + tests) | ~120 | S67-A |
| node_id optionnel deploy (refactor + tests) | ~60 | S67-A |
| FTS5 migration + search.rs + API endpoint | ~300 | S67-A |
| sbfb-manifest crate + validation | ~200 | S67-A |
| **Sous-total S67** | **~760** | |

---

## 6. Risques structurels identifies

### R1 — MemStore = donnees volatiles
Le risque le plus grave : **TOUTES les archives zip, TOUS les iroh-docs
namespaces, TOUS les blobs sont en memoire.** Un restart du daemon =
perte totale des blobs. Les apps publiees disparaissent.

Consequence pour Factory : un `sbfb-factory create` qui deploie une app
verrait l'app disparaitre au prochain restart daemon. Inutilisable pour
un pilote.

### R2 — Provenance non verifiable cross-node
`get_provenance` verifie avec la cle locale (`state.pow_keypair`).
Un observateur externe ne peut pas verifier la provenance d'un projet
deploye par un autre noeud. Factory et le pilote ferme (2-3 personnes)
sont directement bloques.

### R3 — Feed opaque
Le feed a 5 endpoints (ticket, join, status, insert, cursor) mais
aucun pour LIRE les entries. RRV ne peut pas indexer le feed. Factory
ne peut pas lire les events de deploy.

---

## 7. Verdict final

### Le protocole est-il pret pour S67 Factory+RRV ?

## NON.

**S66 n'a aucun code livre.** Les 5 phases (A-E) sont a 0% de realisation.
Le seul artefact existant est le plan (commit dcd8270).

### Ce qui bloque S67

1. **iroh persistence (S66 Phase A)** — sans `with_data_dir` et `FsStore`,
   les blobs et les docs sont volatiles. Un pilote ferme est impossible.
   C'est le bloqueur numero 1.

2. **Provenance cross-node (S66 Phase C)** — MANDATORY 3/3. Sans cela,
   la verification entre noeuds est non fonctionnelle.

3. **Provenance 3 etats (S66 Phase C)** — MANDATORY 3/3. Le 404 actuel
   est inexploitable par un client externe (Factory).

4. **Feed republish au boot (S66 Phase C)** — les entries feed inserees
   en SQLite ne sont pas republicees dans iroh-docs apres restart.
   Le feed P2P perd son historique.

### Ce qui est moins urgent pour S67 mais reste dette

- THREAT_MODEL section feed (Phase B) : documentation
- PRAGMA synchronous FULL (Phase B) : defense-in-depth
- feed_join handle leak (Phase C) : proprete
- Orphan recovery + RevocationCache (Phase D) : robustesse
- E2E restart test (Phase E) : verification formelle

### Estimation de ce qui reste

| Ce qu'il faut finir | Effort estime | Duree |
|---|---|---|
| S66 Phase A (persistence) | ~200-300 LOC, 5 tests | 1-2 jours |
| S66 Phase B (dette pair) | ~50 LOC + docs | 0.5 jour |
| S66 Phase C (feed+provenance) | ~250-350 LOC, 6 tests | 1-2 jours |
| S66 Phase D (orphan+revocation) | ~150-200 LOC, 3 tests | 1 jour |
| S66 Phase E (E2E + wrap-up) | ~100 LOC, 2 tests | 0.5 jour |
| **Total S66** | **~750-1200 LOC, ~16 tests** | **~4-6 jours** |

### Recommandation

**Finir S66 integralement avant de toucher a S67.** Le plan S66 est solide
et bien decoupe. Les 5 phases sont sequentielles avec des dependances
claires (A→B→C→D→E). Il n'y a pas de raccourci possible : la persistence
(Phase A) est prerequis pour tout le reste.

La synthese elle-meme le dit sans ambiguite :
> "Si un de ces items echoue, S67 NE DOIT PAS demarrer."

4 des 7 items echouent. S67 est interdit a ce stade.
