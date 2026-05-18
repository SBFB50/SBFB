# Recherche Sprint 66 — Durabilite

**Objectif** : Faire survivre le reseau SBFB aux redemarrages.
**Date** : 2026-05-18
**Confiance globale** : HIGH (analyse exhaustive du code source + docs iroh officielles)

---

## 1. Inventaire exhaustif de la persistence actuelle

### 1.1 Persistence iroh-docs — PERSISTANT (avec data_dir)

**Fichiers** : `crates/nexus-core-rs/src/node.rs`, `crates/nexus-core-rs/src/docs.rs`

**Etat** : iroh-docs utilise `redb` comme store embarque. Quand `NodeConfig::with_data_dir(path)` est utilise, iroh-docs ecrit `docs.redb` + `default-author` dans le repertoire fourni. Les namespaces, les auteurs et toutes les entrees survivent au redemarrage.

**Preuve dans le code** :
- `node.rs:296-308` : `Docs::persistent(path)` est appele quand `cfg.data_dir` est `Some`.
- Test `persistent_data_dir_reboots_with_same_doc_and_author()` (node.rs:388-428) : prouve que les docs et auteurs survivent au reboot.

**MAIS** : dans `runtime.rs`, le daemon ne passe PAS `data_dir` a `NodeConfig`. Il utilise seulement `with_secret_key()`. Donc **iroh-docs tourne en memoire par defaut dans le daemon** — chaque restart cree des namespaces frais.

**Correction** : `runtime.rs:290-312` ne fait que `NodeConfig::default().with_secret_key(secret_bytes)`. Il manque `.with_data_dir(...)`.

**Impact au restart** :
- Les iroh-docs namespaces (project doc, storage namespaces, feed namespace) sont PERDUS
- Le daemon reboot les recree (cf. `boot_storage_namespace`, `boot_feed_namespace` dans runtime.rs:1405-1519) a partir des IDs stockes en SQLite (table `storage_namespaces`)
- MAIS les entrees iroh-docs ecrites par les peers distants (votes Ideas Hub, feed sync) sont perdues car le namespace est recree vide

**Severite** : CRITIQUE — c'est le gap #1 de durabilite.

### 1.2 Persistence iroh-blobs — NON PERSISTANT (MemStore)

**Fichiers** : `crates/nexus-core-rs/src/node.rs`, `crates/nexus-core-rs/src/blobs.rs`

**Etat** : iroh-blobs utilise `MemStore::default()` (node.rs:294). Toutes les donnees blobs sont volatiles.

**Ce qui est stocke en blobs** :
- Archives web des apps (zip) — recues via `fetch_ticket` depuis les peers
- Listes de curateurs (JSON signe) — recues via gossip + fetch_ticket

**Impact au restart** :
- Toutes les archives d'apps telechargees disparaissent
- Toutes les listes de curateurs en cache disparaissent
- Le blob-serve cache LRU (in-memory DashMap, `blob_serve.rs`) disparait aussi
- Les apps deviennent inaccessibles jusqu'a re-fetch depuis un peer

**Solution disponible** : iroh-blobs 0.100 expose `FsStore` (feature `fs-store`) qui persiste sur le filesystem via redb. Le commentaire node.rs:23 ("Sprint 4 will add: Persistent blob store backed by filesystem") est un TODO vieux de 60+ sprints jamais implemente.

**Severite** : HAUTE — les apps disparaissent au restart.

### 1.3 Persistence SQLite — PERSISTANT et CRASH-SAFE

**Fichiers** : `crates/nexus-coordinator-rs/src/db.rs`

**Etat** : `CoordinatorDb::open()` active WAL mode (`journal_mode = WAL`) et ouvre `~/.sbfb/coordinator.db`. 13 migrations rusqlite_migration.

**Tables persistees** :
| Table | Contenu | Survit restart | Survit crash |
|-------|---------|----------------|--------------|
| `tasks` | Historique des taches | OUI | OUI (WAL) |
| `kudos` | Ledger kudos hash-chain | OUI | OUI |
| `public_feed` | Feed entries append-only | OUI | OUI |
| `feed_cursor` | Checkpoint materializer | OUI | OUI |
| `apps` | Metadonnees apps | OUI | OUI |
| `provenance_records` | Attestations SLSA L1 | OUI | OUI |
| `app_storage` | KV storage per-app | OUI | OUI |
| `storage_namespaces` | Namespace IDs iroh-docs | OUI | OUI |
| `gossip_outbox` | Enveloppes gossip replay | OUI | OUI |
| `contributor_attestations` | Couche 2 | OUI | OUI |
| `invites` | Tokens d'invitation | OUI | OUI |
| `quarantine_messages` | Messages suspects | OUI | OUI |
| `pow_task_counts` | Compteurs PoW | OUI | OUI |
| `task_results` | Resultats multi-worker | OUI | OUI |

**Point d'attention** : WAL mode avec `synchronous` non explicitement configure — SQLite default en WAL est `NORMAL`, ce qui signifie qu'une transaction committee peut etre perdue en cas de power loss (pas de crash process). Pour un daemon P2P, c'est acceptable (les donnees sont recoverables depuis les peers), mais merite d'etre documente.

**Severite** : OK — c'est la couche la plus solide du projet.

### 1.4 Persistence feed hash-chain — PARTIELLEMENT PERSISTANT

**Fichiers** : `crates/nexus-coordinator-rs/src/public_feed.rs`, `crates/nexus-shell-daemon/src/feed_sync.rs`

**Etat** :
- Les `FeedEntry` sont stockes en SQLite (table `public_feed`) — survivent au restart
- Le feed est replique via iroh-docs namespace (`sbfb-feed`) — MAIS le namespace iroh-docs est volatile (cf. 1.1)
- Au restart, le daemon recree le namespace feed a partir de l'ID en SQLite (`boot_feed_namespace` dans runtime.rs:1466-1519)

**Scenario restart** :
1. Daemon stop propre → restart : les entries feed en SQLite sont intactes. Le namespace iroh-docs est recree vide. Les peers qui font `feed_join` verront un namespace vide cote local mais le daemon repond correctement sur l'API HTTP car celle-ci lit SQLite.
2. Apres restart, le daemon NE republish PAS les entries feed existantes vers le nouveau namespace iroh-docs. C'est exactement le carry `P2-ORPHAN-REPUBLISH-RECOVERY`.

**Severite** : MOYENNE — les donnees locales survivent (SQLite) mais la capacite a servir le feed aux peers via iroh-docs est cassee apres restart.

### 1.5 Persistence identite — PERSISTANT

**Fichiers** : `crates/nexus-core-rs/src/keystore.rs`, `crates/nexus-shell-daemon/src/runtime.rs`

**Etat** : L'identite Ed25519 est persistee de deux manieres :
1. **Keystore chiffre** : `~/.sbfb/keyring/identity.enc` (Argon2id + AES-GCM + OS keyring)
2. **Fichier node_key** : `<root>/node_key` (32 bytes raw, mode 0600) — fallback quand le launcher n'est pas utilise

**Runtime.rs:128-150** : `load_or_generate_node_key()` lit ou cree `<root>/node_key`. Le daemon a TOUJOURS une identite stable apres le premier boot.

**Severite** : OK — c'est solide.

### 1.6 Persistence etat worker — PARTIELLEMENT PERSISTANT

**Fichiers** : `crates/nexus-worker-core/src/`

**Etat** :
- **Allowlist** : SQLite dans le worker data dir — persistant
- **Consent** : `consent.json` avec atomic write — persistant, surveille par file watcher
- **State snapshot** : `state.json` ecrit periodiquement — persistant mais informatif seulement
- **GPU state** : volatile (NVML queries a chaud)
- **Tasks en cours** : volatiles — une task interrompue par un crash est perdue
- **Rate limiters** : in-memory (governor GCRA) — reset au restart (acceptable)

**Severite** : FAIBLE — le worker est designed pour etre ephemere. Les tasks interrompues sont re-dispatchees par le coordinator.

### 1.7 Persistence curator runtime — PERSISTANT (subscriptions)

**Fichiers** : `crates/nexus-shell-daemon-core/src/iroh_runtime.rs`

**Etat** :
- **Attention set** (quels curateurs on suit) : persiste dans `subscriptions.json` — survit au restart
- **Listes de curateurs en cache** : DashMap in-memory — PERDUES au restart
- **Revision counter** (anti-rollback) : in-memory — PERDU au restart (un attacker pourrait replay une vieille liste)

**Apres restart** : le daemon re-subscribe aux curateurs mais doit attendre que les peers re-broadcastent leurs listes via gossip. Le gossip outbox persiste (SQLite `gossip_outbox`) donc les propres annonces du noeud sont replayed au prochain NeighborUp.

**Severite** : MOYENNE — les listes de curateurs mettent du temps a revenir.

### 1.8 Persistence blob-serve cache — NON PERSISTANT

**Fichier** : `crates/nexus-shell-daemon-core/src/blob_serve.rs`

**Etat** : `BlobServeCache` est un `DashMap` in-memory avec LRU eviction (max 32 entries). Les archives decompressees sont integralement en RAM.

**Impact** : Au restart, meme si les blobs etaient persistes (ce qui n'est pas le cas), les archives decompressees seraient a refaire. C'est un cache derive, pas une source de verite — acceptable.

**Severite** : FAIBLE — c'est un cache, pas un store.

### 1.9 Persistence RevocationCache — NON PERSISTANT

**Fichier** : `crates/nexus-core-rs/src/key_rotation.rs`

**Etat** : `RevocationCache` est un `HashMap` in-memory. Les annonces de rotation de cles sont perdues au restart.

**Impact** : Un noeud qui redemarrage perd la connaissance des rotations de cles et pourrait accepter des messages signes par des cles revoquees pendant la fenetre de transition.

**Severite** : MOYENNE — le carry `LT-5 redundancy persistence` couvre partiellement ce point.

### 1.10 Persistence trust web/cache — PERSISTANT

**Fichier** : `crates/nexus-shell-daemon-core/src/trust_web.rs`, `trust_cache.rs`

**Etat** : La trust web utilise SQLite (7d TTL). Persistant.

**Severite** : OK.

---

## 2. Matrice des scenarios de restart

### Scenario 1 : Stop propre → Restart meme machine

| Composant | Perdu ? | Consequence |
|-----------|---------|-------------|
| Identite Ed25519 | Non | Meme node_id |
| Coordinator DB (tasks, kudos, feed, apps) | Non | Tout l'historique intact |
| Storage namespaces IDs | Non (en SQLite) | IDs connus |
| iroh-docs contenu | **OUI** | Namespaces recreees vides — les peers doivent re-sync |
| iroh-blobs | **OUI** | Archives apps perdues — re-fetch necessaire |
| Curator lists cache | **OUI** | Attente re-broadcast gossip |
| Gossip outbox | Non (en SQLite) | Propres annonces replayed |
| Feed entries | Non (en SQLite) | Mais pas republishees vers iroh-docs |
| Browse entries | **OUI** | DashMap volatile |

### Scenario 2 : Crash (kill -9)

Meme que scenario 1 + risque de WAL corruption SQLite (faible avec `synchronous=NORMAL`, quasi-nul avec `synchronous=FULL`). Le `running.json` reste sur disque — le daemon suivant le detecte comme stale et le remplace.

### Scenario 3 : Nouvelle machine, meme identite

Necessite :
1. Copier `identity.enc` + transferer kek2 du keyring OS (ou copier `node_key`)
2. Copier `coordinator.db` (ou accepter de repartir de zero)
3. Les iroh-docs et blobs sont non-persistants donc non-transferables de toute facon
4. Le feed SQLite peut etre copie — les entries restent valides cryptographiquement

### Scenario 4 : Rejoin apres 24h offline

- Les feed entries recues pendant l'absence sont re-syncees via iroh-docs (SI le namespace n'a pas ete detruit par le restart)
- Les curator lists sont re-broadcastees par les peers via le gossip NeighborUp outbox replay
- Les archives apps doivent etre re-fetchees via BlobTicket
- **Gap** : si le noeud a restart et recree les namespaces, les peers ne reconnectent pas automatiquement aux nouveaux namespaces — il faut re-echanger les DocTickets

### Scenario 5 : Noeud A publie → Noeud B sync → Noeud A disparait

- Noeud B a les entries feed en SQLite — elles sont integres et verifiables
- Noeud B a les blobs (archives) en MemStore — **PERDUS** si B restart
- Noeud B n'a plus de peer pour re-fetch → les apps sont inaccessibles
- C'est le scenario le plus critique pour la durabilite : un reseau ou tous les noeuds utilisent MemStore ne peut pas survivre a un restart generalise

---

## 3. Les 5 gaps de durabilite par ordre de criticite

### GAP-1 : iroh-blobs MemStore volatil (CRITIQUE)

**Probleme** : Toutes les archives apps sont en RAM. Un restart = toutes les apps disparaissent.

**Solution** : Passer de `MemStore` a `FsStore` (iroh-blobs 0.100 feature `fs-store`).

**Impact code** :
- `crates/nexus-core-rs/src/node.rs` : remplacer `MemStore::default()` par `FsStore::load(data_dir.join("blobs"))` quand `data_dir` est fourni
- `crates/nexus-core-rs/src/blobs.rs` : le `BlobsClient` est generique sur `&MemStore` — il faut soit trait-objectiser, soit passer a un enum `Store`
- Le `Node` struct doit porter le bon type de store

**Complexite** : HAUTE — touche le coeur du crate fondation, tous les crates downstream doivent compiler.

### GAP-2 : NodeConfig sans data_dir dans le daemon (CRITIQUE)

**Probleme** : Le daemon ne passe jamais `data_dir` a `NodeConfig`. Les iroh-docs sont en memoire.

**Solution** : Dans `runtime.rs`, ajouter `.with_data_dir(opts.paths.root.join("iroh"))` au `NodeConfig`.

**Impact code** : 1 ligne dans runtime.rs.

**Complexite** : FAIBLE — mais les consequences sont enormes. Avec cette seule ligne :
- Les project docs survivent au restart
- Les storage namespaces (Ideas Hub) gardent leurs votes
- Le feed namespace garde ses entries
- Le default-author reste le meme

### GAP-3 : Pas de republish feed DB → iroh-docs au boot (HAUTE)

**Probleme** : Apres restart, les feed entries en SQLite ne sont pas republishees vers le namespace iroh-docs. Les peers ne peuvent pas sync le feed.

**Carry existant** : `P2-ORPHAN-REPUBLISH-RECOVERY` (1/3, owner S65).

**Solution** : Au boot, apres `boot_feed_namespace`, iterer toutes les entries feed en SQLite et les ecrire dans le namespace iroh-docs. C'est un one-shot au demarrage.

**Impact code** : ~20 lignes dans runtime.rs ou feed_sync.rs.

**Complexite** : FAIBLE — mais necessaire pour que le feed P2P fonctionne apres restart.

### GAP-4 : RevocationCache in-memory perdue au restart (MOYENNE)

**Probleme** : Les rotations de cles sont oubliees au restart.

**Carry existant** : Mentionne dans key_rotation.rs ("SQLite persistence deferred S26").

**Solution** : Ajouter une table SQLite `key_rotations` et charger le RevocationCache au boot.

**Impact code** : ~30 lignes (migration M14 + load au boot).

**Complexite** : FAIBLE.

### GAP-5 : feed_join fire-and-forget sans shutdown channel (FAIBLE)

**Carry existant** : `P2-FEED-JOIN-HANDLE-LEAK` (1/3, owner S65).

**Probleme** : Le JoinHandle du `feed_join` tokio::spawn n'est pas tracked — pas de shutdown propre.

**Solution** : Stocker le handle et joindre au shutdown.

---

## 4. Recherche externe — patterns de persistence P2P

### 4.1 iroh persistence model

Sources : [docs.iroh.computer/protocols/blobs](https://docs.iroh.computer/protocols/blobs), [iroh-blobs DESIGN.md](https://github.com/n0-computer/iroh-blobs/blob/main/DESIGN.md)

- iroh-docs : `Docs::persistent(path)` cree `docs.redb` qui persiste toutes les replicas
- iroh-blobs : `FsStore` (feature `fs-store`) utilise redb + filesystem hybride (inline < 16KiB, fichiers au-dela)
- Pour un usage reel, coupler `Docs::persistent(path)` avec `FsStore::load(path)` — les deux doivent pointer vers le meme directory

### 4.2 SQLite WAL crash-safety

Sources : [sqlite.org/wal](https://sqlite.org/wal.html), [avi.im/blag/2025/sqlite-fsync](https://avi.im/blag/2025/sqlite-fsync/)

- WAL + `synchronous=NORMAL` : les transactions commitees peuvent etre perdues en cas de power failure (pas de crash process)
- WAL + `synchronous=FULL` : fsync additionnel apres chaque commit — durable meme en power loss
- Pour un daemon P2P, `NORMAL` est acceptable car les donnees sont recoverable depuis le reseau

### 4.3 OrbitDB patterns

Sources : [github.com/orbitdb/orbitdb](https://github.com/orbitdb/orbitdb)

- Cache local des replicas pour reload rapide sans re-sync
- Append-only log (CRDT operation-based) — structurellement identique au feed SBFB
- La persistance passe par IPFS pinning + IndexedDB cote client

### 4.4 Patterns recommandes pour SBFB

1. **Persist everything locally** : iroh-docs (redb) + iroh-blobs (FsStore) + SQLite = triplet de durabilite
2. **Republish au boot** : after restart, re-seed iroh-docs depuis SQLite pour que les peers puissent sync
3. **Gossip outbox** : deja implemente (SQLite `gossip_outbox`) — pattern solide
4. **Idempotent ingest** : deja implemente (UNIQUE index `idx_feed_entry_hash`) — safe pour les replays

---

## 5. Analyse des carry items

### P2-FEED-JOIN-HANDLE-LEAK (1/3)

**Etat dans le code** : `feed_sync.rs:541-649` — `feed_join` fait un `tokio::spawn` dont le JoinHandle n'est jamais stocke. Pas de shutdown channel.

**Recommandation** : A resoudre dans S66 car c'est un leak de ressources qui affecte la durabilite du feed sync.

### P2-ORPHAN-REPUBLISH-RECOVERY (1/3)

**Etat dans le code** : Le code de `insert_and_publish_feed_operation` (feed_sync.rs:85-107) fait rollback si publish echoue, MAIS au boot, les entries existantes en SQLite ne sont JAMAIS republishees vers iroh-docs.

**Recommandation** : MANDATORY pour S66 — sans ca, le feed P2P ne fonctionne pas apres restart.

### LT-5 redundancy persistence

**Etat** : Reclassifie S26, jamais implemente. Concerne la persistence des caches in-memory (RevocationCache, PoW caches, etc.).

**Recommandation** : RevocationCache oui (securite). PoW caches non (reset acceptable).

---

## 6. Plan de phases S66

### Phase A : iroh data_dir + iroh-docs persistence (MANDATORY)

**Objectif** : Les iroh-docs survivent au restart.

**Changements** :
1. `runtime.rs` : ajouter `.with_data_dir(opts.paths.root.join("iroh"))` au NodeConfig
2. Supprimer la logique de recreation de namespace dans `boot_storage_namespace` et `boot_feed_namespace` — les namespaces existent deja dans redb apres restart
3. Tests : stop → restart → verifier que les entries iroh-docs sont intactes

**Risques** : Les tests existants utilisent des tempdir — data_dir change le comportement de Docs. S'assurer que les tests unitaires continuent de tourner en mode in-memory.

**Carry resolu** : P2-ORPHAN-REPUBLISH-RECOVERY partiellement (les entries survivent en iroh-docs natif).

### Phase B : iroh-blobs FsStore (MANDATORY)

**Objectif** : Les archives apps survivent au restart.

**Changements** :
1. `Cargo.toml` (workspace) : activer feature `fs-store` sur iroh-blobs
2. `node.rs` : quand `data_dir` est fourni, utiliser `FsStore::load(data_dir.join("blobs"))` au lieu de `MemStore::default()`
3. `blobs.rs` : adapter `BlobsClient` pour fonctionner avec `FsStore` ou `MemStore` (pattern enum ou trait object)
4. `node.rs` : le type `Node` doit accepter les deux stores (generique ou enum)
5. Tests : deux noeuds — A ajoute un blob, reboot, B fetch via ticket depuis le blob persiste de A

**Risques** : Refactoring profond du crate fondation. Tous les crates downstream (`nexus-coordinator-rs`, `nexus-shell-daemon-core`, `nexus-shell-daemon`, `nexus-worker-core`, `nexus-test-harness`) compilent contre `Node` — le changement de type du store est breaking.

**Option simplifiee** : Au lieu de rendre Node generique, toujours utiliser FsStore (meme en tests, avec tempdir). MemStore devient un detail d'implementation interne pour les tests qui veulent la vitesse.

### Phase C : Feed republish au boot + feed_join handle (MEDIUM)

**Objectif** : Le feed P2P fonctionne apres restart.

**Changements** :
1. Au boot dans `runtime.rs`, apres `boot_feed_namespace`, iterer les entries en SQLite (`replay_all`) et les ecrire dans le namespace iroh-docs via `publish_feed_entry_to_docs`
2. Stocker le JoinHandle de `feed_join` dans DaemonHttpState, joindre au shutdown
3. Tests : inserter 5 entries feed, restart daemon, verifier que les entries sont presentes dans iroh-docs ET accessibles via l'API

**Carry resolu** : P2-ORPHAN-REPUBLISH-RECOVERY (3/3), P2-FEED-JOIN-HANDLE-LEAK (3/3).

### Phase D : RevocationCache persistence + SQLite synchronous (FAIBLE)

**Objectif** : Securite et durabilite des caches critiques.

**Changements** :
1. Migration M14 : table `key_rotations` (old_pubkey, new_pubkey, timestamp, transition_days, signature)
2. Au boot, charger le RevocationCache depuis SQLite
3. Ajouter `synchronous = FULL` au `CoordinatorDb::open()` pour WAL crash-safety renforcee
4. Tests : rotation de cle, restart, verifier que la revocation est toujours active

### Phase E : Test E2E restart complet

**Objectif** : Prouver que le daemon survit a un cycle stop → start → verify.

**Tests** :
1. Daemon boot → publish app via deploy → insert feed entry → subscribe curator → stop
2. Daemon restart → verifier : app accessible (blob persiste), feed entries presentes (SQLite + iroh-docs), curator subscription active (subscriptions.json), meme node_id
3. Deux daemons : A publie, B sync, A restart → B a toujours les donnees, A retrouve les siennes
4. Crash simule (drop sans shutdown) → restart → tout fonctionne

---

## 7. Estimations et dependances

| Phase | Complexite | Lignes estimees | Risque |
|-------|-----------|-----------------|--------|
| A (data_dir) | Faible | ~30 | Tests a adapter |
| B (FsStore) | Haute | ~200 | Refactoring fondation, tous crates touchees |
| C (feed republish) | Moyenne | ~80 | Performance au boot si beaucoup d'entries |
| D (revocation persistence) | Faible | ~60 | Aucun |
| E (E2E tests) | Moyenne | ~150 | Temps d'execution des tests |

**Dependance critique** : Phase B depend de Phase A (FsStore necessite un `data_dir`).

**Ordre recommande** : A → B → C → D → E (sequentiel, chaque phase construit sur la precedente).

---

## 8. Sources

- [iroh docs - Blobs](https://docs.iroh.computer/protocols/blobs)
- [iroh-blobs DESIGN.md](https://github.com/n0-computer/iroh-blobs/blob/main/DESIGN.md)
- [iroh-docs GitHub](https://github.com/n0-computer/iroh-docs)
- [iroh-blobs 0.100 docs.rs](https://docs.rs/iroh-blobs/0.100.0/iroh_blobs/)
- [SQLite WAL mode](https://sqlite.org/wal.html)
- [SQLite fsync durability 2025](https://avi.im/blag/2025/sqlite-fsync/)
- [OrbitDB persistence patterns](https://github.com/orbitdb/orbitdb)
- [iroh-blobs fs-store feature issue](https://github.com/n0-computer/iroh-blobs/issues/84)
