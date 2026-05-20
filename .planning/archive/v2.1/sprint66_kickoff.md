# Sprint 66 — Kickoff (Durabilite)

**Ecrit** : 2026-05-19 (post-audit gate S65 PASS `a2fec86`).
**Type** : **sprint pair** — phase dette obligatoire (Regle 1
§6.2.1). Phase B reservee exclusivement aux items differes.
Deux items 3/3 (Regle 2) a traiter : P2-PROVENANCE-404-BRIDGE
et P2-VERIFY-LOCAL-KEY-ONLY.
**Tip master d'entree** : `a2fec86` (audit findings S65 PASS
0 P0, 0 P1, 4 P2, 2 P3).
**Phase 0 audit Sprint 65** : **DEJA JOUE** — `a2fec86` PASS.
Aucun fix bloquant requis.
**Version archive** : v2.1 — Confiance + Factory Canari + RRV.
**Roadmap source** :
`.planning/roadmap_v3_public_trust_factory_babel_rrv.md`.
Sprint 2 sur 11 (Arc 1 Fondations, 2/2 — dernier sprint de
l'arc).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated S65 kickoff 2026-05-18
  (1 jour). 3 triggers evalues (INCHANGES depuis S65) :

  1. **iroh > 0.98 + iroh-docs > 0.98** : iroh = "1.0.0-rc.0",
     iroh-docs = "0.99.0". Inchange depuis S65 kickoff.
     **Decision** : reste deferred (upgrade iroh 1.0 = sprint
     dedie Arc 1/2 Gate 1).

  2. **arti-client > 0.41** : arti-client = "0.42.0". Inchange.
     **Decision** : deferred. 0 CVE entre 0.41 et 0.42.

  3. **frost-ed25519 > 3.0** : frost-ed25519 = "3.0.0". Trigger
     INACTIF (on utilise 3.0.0, trigger > 3.x).

- **context7 queries** :

  1. context7 `/websites/rs_iroh` (2026-05-19) : API Endpoint,
     NodeConfig, Docs. Pas de changement API persistence depuis
     0.98. Confirme `Docs::persistent(path)` vs `Docs::memory()`.

  2. context7 `/n0-computer/iroh-blobs` (2026-05-19) : `FsStore`
     API confirmee — `FsStore::load(path).await?` retourne un
     `FsStore` qui `Deref<Target=Store>`. `BlobsProtocol::new`
     prend `&Store` (pas generic) — pas de changement de type
     sur `Node`. `FsStore::shutdown()` requis au teardown pour
     flush redb.

  3. context7 `/websites/rs_iroh` (2026-05-19) : `BlobsProtocol`
     non-generique. Accepte `&Store` obtenu via `Deref` depuis
     n'importe quel store backend (MemStore ou FsStore).

- **WebSearch queries** :

  1. WebSearch "iroh-blobs FsStore 0.100 persistent" (2026-05-19) :
     docs.rs/iroh-blobs — `FsStore` disponible feature `fs-store`
     (default dans iroh-blobs 0.100). Pas de type parameter sur
     `BlobsProtocol`. Migration MemStore→FsStore = changement
     d'initialisation uniquement, pas de refactor type.

  2. WebSearch "iroh-blobs BlobsProtocol FsStore type" (2026-05-19) :
     docs.rs — `BlobsProtocol` struct sans type parameter, `new`
     prend `&Store`. Confirme : le `Node` struct n'a PAS besoin
     de devenir generique.

  3. WebSearch "SQLite WAL synchronous FULL vs NORMAL" (2026-05-19) :
     sqlite.org/wal.html + avi.im/blag/2025/sqlite-fsync/ —
     `synchronous=NORMAL` en WAL mode est safe contre la
     corruption mais peut perdre la derniere transaction en cas
     de crash OS. `synchronous=FULL` ajoute un fsync par
     transaction commit (plus lent mais durable). Pour un daemon
     desktop, FULL est le bon choix (insert feed = <10ms, le
     cout fsync est negligeable).

  4. WebSearch "provenance verification UX absent vs failed"
     (2026-05-19) : npm attestation framework distingue
     `attestation_absent` de `attestation_invalid`. C2PA (2025)
     distingue "no manifest" de "verification failed". Pattern
     universel : 3 etats (absent/valide/invalide), pas 2
     (true/false).

- **G9 codebase factual scan** (lecture directe code) :

  1. **node.rs** (l.48-327) : `Node` struct contient
     `blobs_store: MemStore`. `create_node_with_config` cree
     `MemStore::default()` (l.294), passe a
     `BlobsProtocol::new(&blobs_store, None)` (l.311).
     `blobs_store()` retourne `&MemStore` (l.159). Pour FsStore :
     `Node.blobs_store` doit devenir un enum ou accepter
     `Store` directement. `BlobsProtocol::new` n'est PAS impacte
     (prend `&Store`, `FsStore` deref vers `Store`).

  2. **runtime.rs** (l.290-313) : `NodeConfig` construit sans
     `data_dir` dans le daemon. La ligne `NodeConfig::default()
     .with_secret_key(secret_bytes)` ne passe PAS `data_dir`.
     L'iroh-docs boot est donc in-memory. `boot_feed_namespace`
     et `boot_storage_namespace` creent des namespaces frais a
     chaque restart.

  3. **provenance.rs** (l.60-89) : `verify_provenance` prend
     `public_key: &[u8; 32]`. Le daemon (http.rs l.1731) passe
     `state.pow_keypair.public_bytes()` = cle LOCALE. Le
     `ProvenanceRecord` contient `node_id` (hex du deployer) —
     la cle du deployer est DANS le record. Pour cross-node :
     decoder `record.node_id` → `[u8; 32]` et passer a
     `verify_provenance`.

  4. **feed_sync.rs** (l.624) : `tokio::spawn` dans `feed_join`
     sans stocker le JoinHandle. Fire-and-forget confirme. La
     tache n'a pas de shutdown channel.

  5. **db.rs** (l.217) : `pragma "journal_mode" "WAL"` present.
     PAS de `pragma "synchronous" "FULL"`. Default WAL = NORMAL.

  6. **BrowsedProject.tsx** (l.291-328) : badge provenance a 4
     etats visuels (loading, verified, error, default) mais
     seulement 2 etats data (verified=true/false). Pas de
     distinction "absent" vs "echec".

  7. **useBridge.ts** (l.344) : `provenance_verify` retourne
     `{verified: false}` sur 404. Conflate "absent" et "echec".

- **ROADMAP_COMMITMENTS check (G7 Regle 3)** :
  - LT-1 Kudos-v2 : **CLOSED S59**.
  - LT-2 Radicle : **trigger PENDING** — tag v1.0 pose
    localement, pas pousse. Pas encore actif.
  - LT-3/LT-4/LT-5 : latent. 0 condition declenchee.
  - LT-6 : RESOLVED S32.
  - LT-7 : gate satisfait (Tier 1+2 S55 + Tier 3 S60).

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 65 a livre le "Contrat Public" (Arc 1 Fondations, sprint
1/2) : chaque texte public est aligne avec les garanties reelles,
le feed est extensible via raw-op (serde_json::Value), les gates
Factory FG0-FG10 et le manifest SBFB.json v2 sont specifies, et
9 items sont fermes dont le MANDATORY 3/3
P2-FEED-INSERT-NO-AUTH-TIER.

Le daemon SBFB fonctionne correctement TANT QU'IL NE REDEMARRAGE
PAS. A chaque restart : les iroh-docs namespaces sont recrees
vides, les archives apps (iroh-blobs MemStore) sont perdues, les
entries feed ne sont pas republishees vers iroh-docs (meme si
elles sont en SQLite), et le RevocationCache est vide.

Sprint 66 ferme l'arc Fondations en rendant le daemon durable :
chaque composant survit aux redemarrages.

### §1.2 Ancrage roadmap v3

Sprint 66 = "S66 Durabilite" dans
`roadmap_v3_public_trust_factory_babel_rrv.md`. Arc 1 Fondations,
sprint 2 sur 2 (dernier de l'arc). Dependances aval : S67 Factory
Foundation (le daemon doit survivre aux restarts pour que les apps
Factory soient persistantes), S69 pilote ferme (le daemon doit
etre fiable pour des testeurs externes).

### §1.3 Compteurs tests entree (tip `a2fec86`)

| Suite | Count |
|---|---|
| Rust nextest | 1333 |
| Vitest | 268 |
| size-limit | 6/6 |
| **Total** | **~1607** |

### §1.4 Pre-launch protocol policy (rappel)

`*_FORMAT_VERSION` reste a 1 jusqu'au go-live public.
Le sprint touche la persistence (iroh data_dir, FsStore, SQLite
pragmas) mais ne modifie PAS les wire formats. Les structures
`FeedEntry`, `ProvenanceRecord`, `ProjectAnnouncement` et
`CuratorList` ne changent pas. L'ajout du champ `status` dans
la reponse provenance (D4) est une extension non-breaking de
l'API HTTP (pas du wire format P2P).
`#[serde(default)]` reste legitime pour robustesse runtime.

---

## §2 Goal

Le sprint rend le daemon SBFB durable : les iroh-docs namespaces,
les archives apps (iroh-blobs), les entries feed, et les caches
de securite survivent aux redemarrages. Les deux MANDATORY 3/3
sont resolus : la provenance distingue "absent" de "echec" et la
verification fonctionne cross-node. Un daemon qui redemarre 10
fois consecutives ne perd ni une entree feed, ni une archive app,
ni une subscription curator.
**Critere SMART : toutes les rows fail-fast vertes au
verification.md, mesure binaire au Phase E wrap-up.**

---

## §3 Phase 0 — Audit gate Sprint 65

**DEJA JOUE** : `a2fec86` PASS (0 P0, 0 P1, 4 P2, 2 P3).
- P2-S65-BODY-FORMAT : pre-identifie [RESOLVED] — hook Check 9
  en place.
- P2-S65-G8-TRACEABILITY : pre-identifie [RESOLVED] — template
  body en place.
- P2-S65-CHORE-MISCLASSIFIED (1/3) : deletions code dans un
  commit chore(planning). Carry S66 — treatable dans phase dette.
- P2-S65-RAWOP-PATTERN-UNDOC (1/3) : raw-op pattern non documente
  dans PATTERNS.md. Carry S66 — treatable dans phase dette.
- P3-S65-CODEX-C-PARTIAL : [RESOLVED].
- P3-S65-CARRY-CLOSURE-ABSENT : [RESOLVED].
Aucun fix bloquant. Ouverture S66 autorisee.

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — iroh-docs persistence via data_dir

**Sources consultees** :
- context7 `/websites/rs_iroh` queried 2026-05-19 : API
  `Docs::persistent(path)` et `Docs::memory()` confirmees.
- WebSearch "iroh-blobs FsStore 0.100" 2026-05-19 : docs.rs —
  pas de changement API iroh-docs depuis 0.98.
- Code local `crates/nexus-core-rs/src/node.rs:296-304` : lecture
  du bloc `Docs::persistent(path)` / `Docs::memory()`. Le wiring
  existe deja, gated sur `cfg.data_dir.is_some()`.
- Code local `crates/nexus-shell-daemon/src/runtime.rs:290-312` :
  `NodeConfig::default().with_secret_key(...)` sans `data_dir`.
- Code local `crates/nexus-worker-core/src/engine/runtime.rs:256-
  258` : le worker UTILISE deja `data_dir` quand fourni.

**Retenu** : Ajouter `.with_data_dir(opts.paths.root.join("iroh"))`
au `NodeConfig` dans `runtime.rs`. Cela active `Docs::persistent`
dans `node.rs:296-304` (code existant, non-modifie). Les
namespaces iroh-docs (project doc, storage namespaces, feed
namespace) sont alors persistes dans `<root>/iroh/docs.redb` par
iroh-docs. Les fonctions `boot_storage_namespace` et
`boot_feed_namespace` n'ont PAS besoin de changement : elles
lisent deja le namespace ID depuis SQLite (`storage_namespaces`
table) et appellent `docs_client.open_doc(id)` qui fonctionne
nativement en mode persistent (le namespace existe deja dans
redb).

Adaptation requise : `boot_feed_namespace` et
`boot_storage_namespace` tentent `create_doc` quand le namespace
n'existe pas en SQLite. En mode persistent, le doc pourrait
exister dans redb mais pas encore dans SQLite (premiere migration).
Ajouter un fallback `list_docs() → find_by_id` si `create_doc`
echoue avec "already exists".

Un test `persistent_data_dir_reboots_with_same_doc_and_author`
existe deja dans `node.rs:388-428` pour valider la persistence
iroh-docs.

**Rejete** :
- Custom redb direct (sans iroh-docs persistence layer) : reinvente
  la roue. iroh-docs gere deja la persistence redb en interne,
  tester un redb custom serait une regression de fiabilite.
  Source : code iroh-docs `Docs::persistent()` (read node.rs).
- Pas de data_dir (statu quo) : incompatible avec le goal S66.
  Chaque restart recree les namespaces vides, les entries feed
  P2P sont perdues, les subscriptions storage sont cassees.
- Symlink vers le coordinator.db dir : iroh-docs utilise redb,
  pas SQLite. Les deux databases ont des schemas differents.

**Implications code** : `runtime.rs` (1 ligne : ajout
`with_data_dir`), `boot_feed_namespace` / `boot_storage_namespace`
(fallback robustesse ~10 LOC chacun).

### D2 — iroh-blobs FsStore activation

**Sources consultees** :
- context7 `/n0-computer/iroh-blobs` queried 2026-05-19 :
  `FsStore::load(path).await?` cree ou rouvre un store persistent
  redb. `FsStore` deref vers `Store`. `BlobsProtocol::new(&store)`
  accepte `&Store`.
- WebSearch "iroh-blobs BlobsProtocol FsStore type" 2026-05-19 :
  docs.rs — `BlobsProtocol` n'est PAS generique, accepte `&Store`.
  Pas de changement de type necessaire sur la struct `Node`.
- Code local `node.rs:48,121,159,294,311` : `MemStore` est le seul
  store utilise. `Node.blobs_store` est `MemStore`, et
  `blobs_store()` retourne `&MemStore`.
- Code local `blobs.rs:50-61` : `BlobsClient` wrap `&'a MemStore`.
  La methode `BlobsClient::new(inner: &'a MemStore)` prend
  specifiquement un `&MemStore`.

**Retenu** : Remplacer `MemStore` par `FsStore` dans `node.rs`
quand `data_dir` est fourni. Le changement est localise :

1. `NodeConfig` reste identique (data_dir already present).
2. `create_node_with_config` : quand `cfg.data_dir.is_some()`,
   utiliser `FsStore::load(data_dir.join("blobs")).await?` au lieu
   de `MemStore::default()`.
3. `Node.blobs_store` passe de `MemStore` a un enum
   `BlobStore { Mem(MemStore), Fs(FsStore) }` pour supporter les
   deux modes (tests sans data_dir gardent MemStore).
4. `Node::blobs_store()` retourne `&Store` (via Deref sur l'enum).
5. `BlobsClient` passe de `&'a MemStore` a `&'a Store` — les
   methodes utilisees (`reader`, `add_bytes`) existent sur `Store`.
6. `BlobsProtocol::new(&blobs_store, None)` fonctionne sans
   changement car il prend deja `&Store`.
7. A `Node::shutdown()`, appeler `blobs_store.shutdown()` pour
   flush redb (FsStore-specific, no-op pour MemStore).

Impact downstream : tous les crates qui appellent
`node.blobs_store()` verront `&Store` au lieu de `&MemStore`.
`Store` expose les memes methodes via son API publique. Revue
rapide des usages : `blob_serve.rs`, `deploy.rs`, `http.rs`
utilisent `BlobsClient::new(node.blobs_store())` — adapter le
type suffit.

**Rejete** :
- Garder MemStore + ecrire un dump/restore custom : double
  serialisation, perte de la deduplication content-addressed
  native de redb, maintenance lourde. FsStore est le chemin
  officiel iroh-blobs.
  Source : context7 iroh-blobs `FsStore::load` documentation.
- Rendre `Node` generique sur le store type `Node<S: StoreImpl>` :
  complexite explosive — chaque crate downstream devrait propager
  le parametre generique. Intenable avec 10 crates. Le enum est
  la solution Rust idiomatique (pattern `rusqlite::Connection`).
  Source : code local — 10 crates workspace dependent de `Node`.
- Ecrire les blobs dans le coordinator.db SQLite : taille des
  archives apps (1-50 MB chacune) ferait exploser le SQLite WAL.
  Un blob store content-addressed (redb) est l'outil adapte.

**Implications code** : `node.rs` (enum BlobStore, changement
create_node_with_config, changement return types),
`blobs.rs` (BlobsClient prend `&Store`), `blob_serve.rs`,
`deploy.rs`, `http.rs` (adaptation type BlobsClient).

### D3 — Feed republish au boot + feed_join handle fix

**Sources consultees** :
- Code local `runtime.rs:606-633` : boot feed namespace deja
  en place. Pas de republish des entries SQLite vers iroh-docs.
- Code local `feed_sync.rs:549-628` : `feed_join` spawn un
  tokio task sans stocker le JoinHandle.
- Code local `public_feed.rs` : `replay_all()` retourne toutes
  les entries feed depuis SQLite. Disponible pour republish.
- Code local `feed_sync.rs:287-293` : `spawn_feed_subscribe`
  retourne `JoinHandle<()>` — pattern a reproduire pour feed_join.
- Code OSS SSB (Secure Scuttlebutt) : log-replay au boot est le
  pattern standard pour les feeds append-only. Le feed SQLite est
  la source de verite, iroh-docs est le transport P2P.
  Source : SSB spec `https://ssbc.github.io/scuttlebutt-protocol-
  guide/` (pattern log-then-replicate).
- Code OSS Automerge : replay depuis persistence au boot pour
  synchroniser l'etat CRDT distribue. Meme pattern.

**Retenu** : Apres `boot_feed_namespace` dans `runtime.rs`,
ajouter un bloc qui :
1. Appelle `replay_all()` depuis le coordinator SQLite pour obtenir
   toutes les entries feed.
2. Pour chaque entry, appelle `publish_feed_entry_to_docs()` sur
   le namespace iroh-docs du feed.
3. Dedup par `entry_hash` (l'index UNIQUE dans SQLite +
   iroh-docs key = `feed/{seq}` empeche les doublons).
4. Log le nombre d'entries republishees.

Pour `feed_join` (P2-FEED-JOIN-HANDLE-LEAK 2/3→3/3 MANDATORY) :
1. Stocker le JoinHandle retourne par `tokio::spawn` dans un
   `Vec<JoinHandle<()>>` dans `DaemonHttpState` ou dans le
   `DaemonRuntime`.
2. Au shutdown, iterer les handles et `join` chacun.
3. Ajouter un shutdown channel (watch) pour signaler l'arret.

**Rejete** :
- Pas de republish (attendre le gossip) : le gossip ne garantit
  pas la delivrance — un noeud qui rejoint le reseau apres un
  restart rate les entries publiees pendant son absence. Le
  republish local vers iroh-docs est la source de verite.
  Source : code `feed_sync.rs` — subscribe ne backfill que les
  entries deja presentes dans le namespace iroh-docs du peer.
- Republish lazy (au premier access) : complexite, risque de race
  condition entre un `feed_join` externe et un republish partiel.
  Le boot complet est O(n) sur le nombre d'entries (~100 entries
  = <100ms).
- Ignorer le JoinHandle leak (tokio cleanup) : tokio drop la
  tache au drop du JoinHandle, mais ici le JoinHandle n'est meme
  pas stocke — la tache est orpheline. Un shutdown propre ne
  peut pas attendre sa completion.
  Source : code `feed_sync.rs:624` — `tokio::spawn` sans capture.

**Implications code** : `runtime.rs` (republish bloc ~20 LOC),
`feed_sync.rs` (feed_join retourne `JoinHandle`, shutdown channel),
`http.rs` ou `runtime.rs` (stockage handles).

### D4 — P2-PROVENANCE-404-BRIDGE : 3 etats provenance (MANDATORY)

**Sources consultees** :
- Code local `http.rs:1714-1758` : endpoint `/provenance` retourne
  404 sans provenance record, 200 avec `{verified: bool}`.
- Code local `useBridge.ts:337-348` : `provenance_verify` retourne
  `{verified: false}` sur 404. Conflate "absent" et "echec".
- Code local `BrowsedProject.tsx:291-328` : badge a 4 etats
  visuels mais 2 etats data (verified true/false).
- WebSearch "provenance verification UX absent vs failed"
  2026-05-19 : npm attestation framework distingue
  `attestation_absent` de `attestation_invalid`. C2PA (2025)
  distingue "no manifest" de "verification failed".
- Code OSS npm registry (GitHub `npm/cli`) : la commande
  `npm audit signatures` distingue clairement "missing
  attestation" de "invalid attestation" dans son output.
- Code OSS Sigstore : la verification retourne un enum
  `VerificationResult { Verified, NotVerified(reason), NoBundle }`
  — 3 etats, pas 2.
  Source : Sigstore docs `sigstore.dev/verify`.

**Retenu** : Ajouter un champ `status` a la reponse provenance :

```json
// Cas 1 : provenance absente
{"status": "absent", "verified": false, "record": null}

// Cas 2 : provenance presente, verification reussie
{"status": "verified", "verified": true, "record": {...}, "provenance_hash": "..."}

// Cas 3 : provenance presente, verification echouee
{"status": "failed", "verified": false, "record": {...}, "provenance_hash": "..."}
```

Le champ `verified: bool` est conserve pour backward compat.
Le nouveau champ `status: "absent" | "verified" | "failed"`
est la source de verite.

Cote bridge (`useBridge.ts`) : `provenance_verify` retourne le
`status` en plus de `verified`. Sur 404, `status = "absent"`.

Cote UI (`BrowsedProject.tsx`) : 4 etats visuels au lieu de 3 :
- Loading : "Verification..." (inchange)
- Absent : "Provenance" avec FileCheck (etat par defaut, pas
  d'erreur — l'app n'a simplement pas ete deployee via verified
  deploy)
- Verified : "Signature verifiee" vert (inchange)
- Failed : "Verification echouee" rouge (inchange)

Gain : l'utilisateur ne voit plus "echoue" pour une app qui n'a
pas de provenance. Il voit "Provenance" (neutre) — pas de fausse
alarme.

**Rejete** :
- Garder 2 etats (verified/not) : confond "app sans provenance"
  (normal pour un upload direct N0) et "provenance invalide"
  (alerte securite). L'utilisateur ne peut pas distinguer une
  situation benigne d'un probleme reel.
  Source : code BrowsedProject.tsx — les deux cas affichent
  le meme badge rouge.
- Retourner 200 avec `{verified: null}` pour absent : null
  viole le contrat JSON implicite et complique le parsing
  frontend. Un champ enum string est plus explicite.
- Supprimer le badge si absent : masque l'information. Un badge
  "Provenance" neutre informe que l'app est au niveau N0
  (Upload direct) de la taxonomie TRUST_TAXONOMY.md.

**Implications code** : `http.rs` (get_provenance modifie, ~5 LOC),
`useBridge.ts` (provenance_verify retourne status, ~5 LOC),
`BrowsedProject.tsx` (4eme etat visuel, ~10 LOC),
`BrowsedProject.test.tsx` (+1 test "absent" etat).

### D5 — P2-VERIFY-LOCAL-KEY-ONLY : verification cross-node (MANDATORY)

**Sources consultees** :
- Code local `provenance.rs:18-29` : `ProvenanceRecord` contient
  `node_id: String` (hex 64 chars de la pubkey Ed25519 du
  deployer).
- Code local `http.rs:1731` :
  `state.pow_keypair.public_bytes()` = cle LOCALE. La
  verification compare la signature a la cle du noeud local,
  pas a la cle du deployer.
- Code local `provenance.rs:60-89` : `verify_provenance` prend
  `public_key: &[u8; 32]`. Il suffit de passer la cle extraite
  du `node_id` du record au lieu de la cle locale.
- Code OSS Keyoxide : verification de signatures Ed25519 par
  extraction de la cle depuis l'URI d'identite (Keyoxide profile).
  Meme pattern : la cle publique est dans le record, pas hard-
  coded.
  Source : `codeberg.org/keyoxide/keyoxide-web`.
- Code OSS Sigstore Rekor : la verification extraite le
  certificat de l'entry du transparency log, pas du verifier
  local. Pattern : "trust the record's identity, verify the
  crypto".
  Source : `github.com/sigstore/rekor`.
- SLSA L1 spec (2023) : la provenance contient l'identite du
  builder. Le verificateur extrait cette identite et verifie
  la signature contre elle.
  Source : `slsa.dev/spec/v1.0/levels`.

**Retenu** : Modifier `get_provenance` dans `http.rs` pour
extraire la cle publique depuis `record.node_id` au lieu de
`state.pow_keypair.public_bytes()` :

```rust
// Avant :
let pub_bytes = state.pow_keypair.public_bytes();
let verified = verify_provenance(&record_json, &pub_bytes);

// Apres :
let verified = match hex::decode(&record.node_id) {
    Ok(bytes) if bytes.len() == 32 => {
        let pub_bytes: [u8; 32] = bytes.try_into().unwrap();
        verify_provenance(&record_json, &pub_bytes)
    }
    _ => false, // node_id invalide = verification echouee
};
```

Cela permet :
1. La verification des provenances deployees par le noeud local
   (meme resultat qu'avant — node_id == pow_keypair.pubkey).
2. La verification des provenances deployees par un AUTRE noeud
   (cross-node) — le `node_id` dans le record identifie le
   deployer, sa signature est verifiee contre sa cle.
3. Les apps syncees via gossip/iroh-docs depuis un pair portent
   la provenance du deployer original, pas du relayeur.

Le changement est backward-compatible : la verification locale
produit le meme resultat (la cle dans `node_id` est celle du
noeud local pour les apps deployees localement).

**Rejete** :
- Maintenir la verification locale uniquement : impossible de
  verifier les provenances recues via feed sync (les apps
  deployees par d'autres noeuds). Un utilisateur qui browse une
  app deployer par un pair voit "verification echouee" alors que
  la provenance est valide.
  Source : code http.rs l.1731 — seule la cle locale est utilisee.
- Lookup de cle via pkarr/DHT : overhead reseau pour chaque
  verification, dependance sur la disponibilite du DHT. La cle
  est DANS le record — pas besoin de lookup externe.
- Trust-on-first-use (TOFU) pour la cle : complexite (base de
  cles connues, PIN set), disproportionnee pre-launch avec 0
  noeud externe.

**Implications code** : `http.rs` (get_provenance, ~8 LOC),
`provenance.rs` (aucun changement — `verify_provenance` prend
deja un `public_key` parametre).

---

**Acknowledged review findings (G1)** :

Scoring : D1 ok, D2 ok, D3 ok, D4 ok, D5 ok.
Rigor signal G4 satisfait (1 warning sur 5 — cf. design_review.md).

D2 warning : le changement `Node.blobs_store` de `MemStore` a
enum `BlobStore` modifie l'API publique de `nexus-core-rs`. Tous
les consumers downstream (3 crates) doivent adapter. Le risque
est une erreur de compilation non-attrapee par un test unitaire.
Decision : adjust — ajouter un test d'integration dans
`nexus-test-harness` qui boot un Node avec FsStore et verifie
que `blobs_store()` est utilisable. Documenter le changement
d'API dans le commit body Phase A.

---

## §5 Plan Phase outline A..E

### Phase A — iroh data_dir + iroh-docs persistence + FsStore

Active `with_data_dir` sur le NodeConfig du daemon et remplace
MemStore par FsStore. Le coeur de la durabilite — les iroh-docs
et iroh-blobs survivent aux restarts.

- `runtime.rs` : `with_data_dir(opts.paths.root.join("iroh"))`
- `node.rs` : enum BlobStore + FsStore quand data_dir present
- `blobs.rs` : BlobsClient prend `&Store`
- Adaptation downstream : blob_serve.rs, deploy.rs, http.rs
- Tests : boot → create doc → shutdown → reboot → doc persiste.
  Boot → add blob → shutdown → reboot → blob accessible.

**Commit cible** : `feat(persistence): Sprint 66 Phase A — iroh data_dir + FsStore`
**Critere** : iroh-docs et iroh-blobs survivent a un cycle
shutdown→reboot (tests verts).

### Phase B — Dette pair (sprint pair, non-negociable)

Phase exclusivement dediee aux items differes (Regle 1 §6.2.1).

- P2-S65-CHORE-MISCLASSIFIED (1/3→CLOSED) : documenter dans
  README.md §4.1 que les deletions de source doivent etre dans
  un commit `chore(cleanup)` ou dans le feat de la phase.
- P2-S65-RAWOP-PATTERN-UNDOC (1/3→CLOSED) : ajouter un pattern
  dans `docs/rust/PATTERNS.md` pour le raw-op store+forward
  (try_parse_op, Value op, validate_feed_operation accept-unknown).
- P2-THREAT-MODEL-FEED-SURFACE (1/3→2/3) : ajouter la section
  feed dans THREAT_MODEL.md (surface, threats T-FEED-1..4, cf.
  PUBLIC_FEED_SPEC.md §12 qui les definit deja).
- SQLite synchronous FULL : ajouter `pragma "synchronous" "FULL"`
  dans `CoordinatorDb::open()` pour WAL crash-safety renforcee.

**Commit cible** : `feat(dette): Sprint 66 Phase B — dette pair + THREAT_MODEL feed + PATTERNS raw-op`
**Critere** : items CLOSED documentes, THREAT_MODEL.md a une
section feed, SQLite FULL pragma actif.

### Phase C — Feed republish + feed_join handle fix + MANDATORY 3/3

Republish des entries feed au boot, fix du handle leak
feed_join, et resolution des deux MANDATORY.

- Feed republish : apres boot_feed_namespace, iterer replay_all()
  et publish_feed_entry_to_docs() pour chaque entry.
- P2-FEED-JOIN-HANDLE-LEAK (2/3→3/3→CLOSED) : stocker le
  JoinHandle de feed_join, ajouter shutdown channel.
- P2-PROVENANCE-404-BRIDGE (3/3 MANDATORY→CLOSED) : champ
  `status` dans la reponse provenance, bridge adapte, badge
  4 etats.
- P2-VERIFY-LOCAL-KEY-ONLY (3/3 MANDATORY→CLOSED) : verification
  cross-node via node_id du record.

**Commit cible** : `feat(feed+provenance): Sprint 66 Phase C — feed republish + provenance cross-node`
**Critere** : feed republish au boot (test), feed_join JoinHandle
tracked, provenance 3 etats (test), verification cross-node (test).

### Phase D — P2-ORPHAN-REPUBLISH-RECOVERY + RevocationCache persistence

- P2-ORPHAN-REPUBLISH-RECOVERY (2/3→3/3→CLOSED) : au boot, si
  une entry feed est en SQLite mais PAS dans iroh-docs (orphan),
  la republisher. Detecter via comparaison entries SQLite vs
  entries iroh-docs. Tail-safe : skip les entries sans prev_hash
  valide.
- RevocationCache persistence SQLite : migration M14 table
  `key_rotations` (old_pubkey, new_pubkey, timestamp,
  transition_days, signature). Au boot, charger le
  RevocationCache depuis SQLite.

**Commit cible** : `feat(persistence): Sprint 66 Phase D — orphan recovery + RevocationCache SQLite`
**Critere** : orphan entries republishees (test), RevocationCache
charge depuis SQLite au boot (test).

### Phase E — Test E2E restart + wrap-up

Tests E2E restart complet + verification.md + audit_plan S67.

- Test E2E : daemon boot → deploy app → insert feed → stop →
  restart → app accessible, feed intact, meme node_id.
- Test crash simule : drop sans shutdown → restart → tout
  fonctionne.
- verification.md + sprint67_audit_plan.md
- CLAUDE.md compteurs + SPRINT_LOG.md row

**Commit cible** : `docs(sprint66): Sprint 66 Phase E — E2E restart test + wrap-up`
**Critere** : E2E restart vert, fail-fast checklist complete.

---

## §6 Items carry/dette

### Items 3/3 (traitement Sprint 66)

| Item | Reports | Phase S66 | Exit condition |
|---|---|---|---|
| P2-PROVENANCE-404-BRIDGE | 3/3 | Phase C | provenance retourne `status: absent/verified/failed`, badge 4 etats |
| P2-VERIFY-LOCAL-KEY-ONLY | 3/3 | Phase C | verification utilise node_id du record (cross-node) |

### Carry absorbes S66

| Item | Reports | Phase S66 | Exit condition |
|---|---|---|---|
| P2-FEED-JOIN-HANDLE-LEAK | 2/3→3/3 | Phase C | JoinHandle track, shutdown channel, joined at shutdown |
| P2-ORPHAN-REPUBLISH-RECOVERY | 2/3→3/3 | Phase D | orphan entries republishees au boot |
| P2-S65-CHORE-MISCLASSIFIED | 1/3 | Phase B (dette) | README.md §4.1 doc amendee |
| P2-S65-RAWOP-PATTERN-UNDOC | 1/3 | Phase B (dette) | PATTERNS.md raw-op pattern ajoute |
| P2-THREAT-MODEL-FEED-SURFACE | 1/3→2/3 | Phase B (dette) | THREAT_MODEL.md section feed ajoutee |

### Carries reconduits S67

| Item | Reports | Justification |
|---|---|---|
| P2-A-1 rand blocker | exemption externe | upstream rand 0.9 non publie — pas de nouvelle release a ce jour (crates.io/crates/rand verifie 2026-05-19) |
| P2-AUDIT-2 iroh transitives | exemption externe | iroh 0.98 pinne, transitives pre-release heritees du pin — pas d'action possible sans upgrade iroh (deferred Gate 1 Arc 1/2) |
| P2-G-1 exe lock intermittent | monitoring | non-reproductible depuis S62 (7 sprints). Si reproductible a nouveau, escalader |
| T-NN+2 iframe Rust-wasm | bloque upstream | toolchain gaps wasm-bindgen/web-sys non resolus — hors scope S66 (LT horizon) |

### Attention 3/3 S67

**P2-FEED-JOIN-HANDLE-LEAK** et **P2-ORPHAN-REPUBLISH-RECOVERY**
sont traites dans S66 (respectivement Phase C et Phase D). Ils
ne seront PAS reconduits.

P2-THREAT-MODEL-FEED-SURFACE passera 2/3 en S67. Si non traite
S67, il passera 3/3 MANDATORY S68.

---

## §7 Scope cuts

| # | Item | Sprint cible | Rationale |
|---|---|---|---|
| 1 | CuratorVouched/CuratorDisendorsed implementation | S67 | Factory Foundation, pas durabilite |
| 2 | BuildQuorumReached feed implementation | S67+ | idem |
| 3 | Quarantine feed hot path | S67+ | glue code anti-spam post-Factory |
| 4 | Age witness gate feed admission | S67+ | idem |
| 5 | T1 CONFIRM_PROMPT complet (UI nonce) | post-pilote S69 | requiert integration UI React + nonce |
| 6 | SBFB.json v2 code implementation | S67 Phase A | S66 = persistence, pas manifest |
| 7 | node_id deprecation dans deploy.rs | S67 Phase A | S66 = persistence, pas deploy refactor |
| 8 | Factory template scaffold | S67 Phase B+ | S66 = persistence, pas Factory |
| 9 | Fuzzing cargo-fuzz/proptest | post-audit | audit prep, pas sprint |
| 10 | CLI verify-release | S67+ | UX enrichissement post-durabilite |
| 11 | VerificationDetail niveau 3 | S67+ | UI enrichissement post-durabilite |
| 12 | Playwright E2E tests re-ecriture | S69 | suppression S65, re-ecriture post-Factory |
| 13 | Feed format version bump | post-launch | pre-launch policy |
| 14 | Multi-curator trust overlay | S67 Phase D (stretch) | roadmap v3 stretch S67 |

---

## §8 Tracabilite scope

| Item S65 "What's NOT" | Sprint + Phase S66 |
|---|---|
| CuratorVouched/CuratorDisendorsed implementation | Reconduit S67 (#1) |
| BuildQuorumReached feed implementation | Reconduit S67+ (#2) |
| Quarantine feed hot path | Reconduit S67+ (#3) |
| Age witness gate feed admission | Reconduit S67+ (#4) |
| T1 CONFIRM_PROMPT complet | Reconduit post-pilote S69 (#5) |
| SBFB.json v2 code implementation | Reconduit S67 (#6) |
| node_id deprecation dans deploy.rs | Reconduit S67 (#7) |
| Factory template scaffold | Reconduit S67+ (#8) |
| Fuzzing cargo-fuzz/proptest | Reconduit post-audit (#9) |
| CLI verify-release | Reconduit S67+ (#10) |
| VerificationDetail niveau 3 | Reconduit S67+ (#11) |
| Playwright E2E re-ecriture | Reconduit S69 (#12) |
| THREAT_MODEL.md section feed | Phase B dette S66 (carry absorbe) |
| Feed format version bump | Reconduit post-launch (#13) |

---

## §9 Risk register

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | FsStore refactor casse la compilation de 3+ crates downstream | Medium | High | Enum BlobStore isole le changement. Compilation incremental apres Phase A. Test d'integration dans test-harness |
| R2 | iroh-docs data_dir corrompu apres crash mid-write (redb corruption) | Low | High | redb est ACID par design (copy-on-write B-tree). SQLite FULL pragma ajoute en Phase B comme defense-in-depth |
| R3 | Feed republish au boot lent sur gros feed (>1000 entries) | Low | Medium | Le feed SBFB a <100 entries. Log le temps de republish. Si >1s, paginer (stretch S67) |
| R4 | feed_join handles s'accumulent sans limite (DoS memoire) | Medium | Medium | Rate-limiter feed_join (max 10 joins actifs). Cleanup des handles termines periodiquement |
| R5 | Provenance cross-node : node_id hex invalide crash le daemon | Low | High | Pattern matching exhaustif avec fallback `verified: false`. Pas de unwrap sur le hex decode |
| R6 | Phase B dette trop chargee (4 items) deborde | Low | Low | Items B sont documentaires (<200 LOC total). Le pragma SQLite est 1 ligne |
| R7 | Migration existante boot_feed_namespace incompatible avec data_dir persistent | Medium | Medium | Fallback list_docs() si create_doc echoue. Test de migration zero→persistent |

---

## §10 Audit gate pattern — rappel

Phase 0 a ete jouee (PASS `a2fec86`).
La Phase E du sprint devra produire :
- `sprint66_verification.md` (self-report fail-fast)
- `sprint67_audit_plan.md` (plan pour Phase 0 S67)
- Mise a jour `docs/rust/PATTERNS.md` si nouveaux patterns
  (Phase B ajoute le pattern raw-op, Phase A peut ajouter le
  pattern FsStore conditional)
- Mise a jour `docs/shell/PATTERNS.md` si pertinent (pas de
  changement frontend structurel attendu)

---

## §11 Checkpoint de validation

1. D1 — Ajouter `with_data_dir` au daemon NodeConfig suffit-il
   pour persister iroh-docs, ou faut-il adapter
   `boot_storage_namespace` et `boot_feed_namespace` pour
   detecter les namespaces deja existants dans redb ?

2. D2 — L'enum `BlobStore { Mem(MemStore), Fs(FsStore) }` est-il
   preferable a un `Box<dyn Store>` trait object, sachant que
   `Store` n'est pas object-safe dans iroh-blobs 0.100 ?

3. D3 — Le republish feed au boot est-il un one-shot synchrone
   (avant d'accepter les requetes HTTP) ou un background task
   asynchrone ? Le one-shot bloque le boot mais garantit la
   coherence.

4. D4 — Le champ `status: "absent" | "verified" | "failed"` dans
   la reponse provenance est-il une extension non-breaking de
   l'API ou un changement du contrat de l'endpoint ?

5. D5 — Pour la verification cross-node, faisons-nous confiance
   au `node_id` dans le record comme identite du deployer, ou
   faut-il une validation additionnelle (ex: le feed contient
   une entry `ReleasePublished` qui reference le meme node_id) ?
