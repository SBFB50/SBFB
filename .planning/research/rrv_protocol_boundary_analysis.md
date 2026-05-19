# RRV Protocol Boundary Analysis

Date : 2026-05-19
Scope : S70-S72 (Arc 3 — Intelligence Verifiable)
Auteur : Analyse architecturale pre-implementation

---

## 1. Question fondamentale

La recherche et les preuves sont-elles des **primitives protocolaires**
(comme la distribution de blobs ou la verification de provenance) ou
des **fonctionnalites applicatives** (comme Factory ou Babel) ?

Reponse courte : **c'est un spectre, pas un binaire.** RRV contient
trois types distincts de fonctionnalites qui se classent dans trois
categories differentes. Les confondre produirait soit un daemon bloated,
soit un reseau incapable de decouverte.

---

## 2. Taxonomie des couches SBFB existantes

Avant de classifier RRV, identifions les couches deja presentes :

| Couche | Exemples existants | Critere d'inclusion |
|---|---|---|
| **Primitive protocolaire** | gossip subscribe/broadcast, blobs fetch, Ed25519 sign/verify, canonical bytes JCS, domain separation, PoW challenge | Sans ca, aucun client ne peut participer au reseau. Defini dans `nexus-core-rs`. |
| **Infrastructure daemon** | curator runtime (DashMap cache + gossip subscribe), browse aggregator (probe + TTL cache), feed store (hash-chain + signature + persistence SQLite), provenance records DB, blob-serve (zip decompression + LRU) | Service local que le daemon fournit a partir des donnees qu'il possede deja. Defini dans `nexus-coordinator-rs` et `nexus-shell-daemon-core`. Tout client qui heberge le daemon en beneficie. |
| **Wire format reseau** | `CuratorListEntry` (sign+gossip+blob), `FeedEntry` (hash-chain+sign+gossip+iroh-docs), `ProjectAnnouncement` (sign+gossip), `PowEnvelope` | Structure signee transmise entre noeuds. Defini dans `nexus-core-rs` ou `nexus-coordinator-rs`. Consomme par toute implementation conforme. |
| **Bridge method** | `task_submit`, `storage_get/set/list/delete`, `browse_list`, `provenance_get/verify`, `identity_pubkey`, `node_status`, `feed_cursor_get`, `pii_redact` | API que les apps iframe peuvent invoquer via postMessage. Definit la surface de capacites accessible aux apps sandboxees. |
| **App externe** | Protocol Explorer (`examples/sbfb-explorer/`), Ideas Hub (`examples/sbfb-ideas/`) | Logique applicative opinionnee, deployee comme archive zip, rendue dans un iframe. Utilise le bridge pour acceder au reseau. |

---

## 3. Classification fonctionnalite par fonctionnalite

### 3.1 S70 — RRV LocalOnly (FTS5)

| Fonctionnalite | Classification | Justification |
|---|---|---|
| **FTS5 virtual table** (M15 migration) | **Infrastructure daemon** | Extension naturelle du schema SQLite existant (`coordinator.db`). Le daemon possede deja les donnees (browse entries, feed entries, provenance records). L'index est une vue materialisee locale, pas un format reseau. Analogue : la table `public_feed` qui est aussi une persistence locale des donnees recues par gossip. |
| **`search.rs` module coordinator** (schema, index, query) | **Infrastructure daemon** | Module dans `nexus-coordinator-rs` au meme titre que `public_feed.rs`, `provenance.rs`, `kudos_ledger.rs`. Le daemon indexe ses propres donnees. Aucun impact reseau. |
| **`GET /api/daemon/search?q=...`** | **Infrastructure daemon** | Endpoint HTTP local comme `/api/daemon/browse` ou `/api/daemon/curators`. Le daemon expose une capacite de recherche sur ses donnees locales. La requete ne quitte jamais le loopback. |
| **Indexation au boot + incrementale** | **Infrastructure daemon** | Meme pattern que le `FeedMaterializer` (curseur + incremental) et le browse aggregator (cache + probe). Le daemon maintient son index a jour au fil des evenements. |
| **Indexation README.md dans archives zip** | **Infrastructure daemon** | Le daemon decompresse deja les archives via `blob_serve.rs` (LRU cache). Lire un README pour l'indexer est une extension naturelle. |
| **Bridge method `search`** | **Bridge method (legitime)** | Cf. analyse detaillee section 4. |
| **Citations exactes** (source_type, entry_hash) | **Infrastructure daemon** | Les citations sont des pointeurs vers des donnees que le daemon possede deja (provenance hash, feed entry hash). C'est de la traçabilite, pas de la logique applicative. |
| **App `sbfb-search`** | **App externe** | HTML+JS dans `examples/`, meme pattern que Explorer/Ideas Hub. Utilise le bridge `search` pour interroger le daemon. |

### 3.2 S71 — Proof Cards

| Fonctionnalite | Classification | Justification |
|---|---|---|
| **`ProofCard` struct** (data model) | **Infrastructure daemon** | Agregation de donnees que le daemon possede : provenance record, curator count, licence (SBFB.json), feed state (fresh/stale), archive hash. La struct est un DTO, pas un wire format. |
| **`compute_proof_card()`** (formule de score) | **Frontiere daemon / app** | C'est le point le plus discutable. La formule est **deterministe et transparente** (pas de ML, pas de subjectivite). Elle agrege des faits protocolaires verifiables. Mais le choix des poids (20 pts provenance, 10 pts curators, etc.) est **opinione**. Voir section 5. |
| **`GET /api/daemon/proof-card/{id}`** | **Infrastructure daemon** | Le daemon fournit le resultat du calcul sur ses donnees locales. Meme pattern que `/api/daemon/browse` qui agrege et formate. |
| **Bridge method `proof_card_get`** | **Bridge method (discutable)** | Cf. analyse detaillee section 4. |
| **Composant ProofCard HTML** (UI) | **App externe** | Rendu visuel du score, barres, actions. Appartient a l'app `sbfb-search` ou au shell Browse. |
| **Tests adversariaux** (spoofing, injection, determinisme) | **Infrastructure daemon** | Tests du module coordinator, pas de l'app. |

### 3.3 S72 — SearchManifest Opt-In

| Fonctionnalite | Classification | Justification |
|---|---|---|
| **`DOMAIN_SEARCH_MANIFEST_V1`** (domain separation) | **Primitive protocolaire** | Nouveau tag de domain separation dans `canonical.rs`, au meme titre que `DOMAIN_FEED_V1` ou `DOMAIN_CURATOR_LIST_V1`. C'est le fondement crypto d'un nouveau type signe. |
| **`SearchManifest` struct** (wire format) | **Wire format reseau** | Structure signee transmise entre noeuds. Defini dans `nexus-core-rs` ou `nexus-coordinator-rs`. Analogue a `CuratorListEntry` ou `FeedEntry`. |
| **`SearchManifestPublished` feed op** | **Wire format reseau** | Nouvelle operation dans le feed raw-op extensible (S65). Les noeuds la propagent sans l'interpreter s'ils ne la connaissent pas. |
| **Gossip topic `search-manifest/v1`** | **Primitive protocolaire** | Nouveau topic gossip, meme pattern que `curator/v1`. Le topic est derive par `BLAKE3("nexus-grid/search-manifest/v1")[..32]`. |
| **`POST /api/daemon/search/publish-manifest`** | **Infrastructure daemon** | Commande pour que l'operateur du noeud publie volontairement son index. |
| **Reception + verification + cache DashMap** | **Infrastructure daemon** | Meme pattern exact que `CuratorRuntime::process_announcement_bytes` : recevoir gossip, parser, verifier signature, stocker en memoire. |
| **Enrichissement des resultats locaux** | **Infrastructure daemon** | Le daemon enrichit ses resultats de recherche avec les projets des manifests distants. C'est de l'agregation locale. |
| **Rate limiter** (1/heure/noeud) | **Infrastructure daemon** | Meme pattern que `FEED_RATE_LIMIT_PER_MINUTE`. |
| **PoW optionnel** (16-bit) | **Infrastructure daemon** | Meme pattern que `FEED_POW_DIFFICULTY`. |
| **Privacy analysis** | Documentation | Pas du code. |

---

## 4. Analyse des methodes bridge

Le bridge iframe est la **surface de capacites** du reseau pour les
apps sandboxees. Chaque methode bridge est une decision d'API qui
affecte tout l'ecosysteme d'apps. L'ajout d'une methode doit etre
justifie par : "est-ce qu'une app sandboxee a legitimement besoin de
demander ca au reseau ?"

### 4.1 `search` — Legitime

**Pour :**
- La recherche locale est une capacite fondamentale. Une app de
  recherche ne peut pas exister sans interroger l'index du daemon.
- Le pattern est identique a `browse_list` : lire une vue agregee
  des donnees du daemon.
- Les requetes ne quittent jamais le loopback (privacy by design).
- Toute app qui veut proposer de la decouverte a besoin de `search`.
  Sans cette methode, seule l'app `sbfb-search` aurait acces a la
  recherche, ce qui serait du privilège de position.

**Contre :**
- `browse_list` retourne une liste plate ; `search` accepte un
  parametre `q` avec une grammaire (BM25, phrase queries, boolean).
  La surface d'attaque est plus large.

**Verdict : AJOUTER.** Le risque est mitigeable par validation
stricte du parametre `q` (longueur max, sanitization). C'est une
lecture seule sur des donnees locales.

### 4.2 `proof_card_get` — Discutable mais justifiable

**Pour :**
- Une app qui affiche des resultats de recherche veut montrer le
  niveau de confiance de chaque resultat. Sans `proof_card_get`,
  elle devrait recalculer le score elle-meme a partir de multiples
  appels bridge (`provenance_get` + `browse_list` + `feed_cursor_get`),
  ce qui est fragile, non-deterministe (formule dupliquee), et lent.
- La Proof Card est un agregat de donnees factuelles, pas un jugement
  subjectif. Le daemon est le seul a avoir toutes les donnees.

**Contre :**
- La formule de score est opinionnee. Exposer `proof_card_get` comme
  bridge method fixe cette formule dans la surface d'API du reseau.
  Un changement de formule casse la semantique de la methode.
- On pourrait exposer les donnees brutes (`proof_facts_get`) et
  laisser les apps calculer le score. Mais ca revient a dupliquer
  la logique partout.

**Verdict : AJOUTER, mais avec garde-fous.**
- La methode retourne les **facteurs** (provenance, curators,
  licence, fraicheur) en plus du score. Les apps qui veulent un
  affichage custom utilisent les facteurs.
- La formule est documentee et versionnee. Si elle change, le champ
  `formula_version` permet aux apps de detecter le changement.

### 4.3 Alternative consideree : `proof_facts_get` au lieu de `proof_card_get`

Exposer uniquement les faits bruts (has_provenance, curator_count,
license_spdx, freshness_state, archive_hash_present, risk_factors)
et laisser chaque app calculer son propre score.

**Rejet :** Ca viole le principe "un fait, une verite". Si 5 apps
calculent 5 scores differents a partir des memes faits, l'utilisateur
voit 5 niveaux de confiance differents pour le meme projet. Le
reseau a besoin d'un score de reference unique et deterministe. Les
apps peuvent l'enrichir (UI, explication, action) mais pas le
remplacer.

---

## 5. La frontiere daemon / app : ou passe le trait ?

### 5.1 Critere : le daemon indexe ce qu'il possede

Le daemon possede deja :
- Les browse entries (via curator gossip + aggregator)
- Les feed entries (via feed sync + iroh-docs)
- Les provenance records (via deploy verified)
- Les curator lists (via gossip subscribe + DashMap)
- Les archives zip (via blob-serve LRU)

Indexer ces donnees en FTS5 est une extension naturelle. Le daemon
ne va pas chercher de nouvelles donnees pour RRV — il indexe ce
qu'il a deja.

### 5.2 Critere : le daemon ne fait pas de jugement subjectif

La formule Proof Card est deterministe : memes inputs → meme score.
C'est un critere d'inclusion dans le daemon. Si la formule incluait
un facteur subjectif (reputation sociale, vote, ML), elle devrait
etre dans une app.

La formule actuelle (section 1002-1026 de la roadmap) est un
**score de completude de preuve** : elle mesure combien de preuves
verifiables existent pour un projet donne. Ce n'est pas un "trust
score" — c'est un "evidence score". Un projet avec provenance
verifiee, 3 curators, licence, et deploy frais a 100% de preuves
possibles. Un projet sans rien a 30% (il existe, c'est tout).

**Decision : la formule reste dans le daemon.** Elle est
l'equivalent du calcul de `BrowseStatus::Reachable` dans le browse
aggregator — un fait derive de donnees locales, pas un jugement.

### 5.3 Critere : le wire format reseau est une primitive

Le `SearchManifest` est un wire format signe, gossipe, et verifie
par tous les noeuds. C'est au meme niveau que `CuratorListEntry` ou
`FeedEntry`. Il definit ce que le reseau "sait" publier et
consommer. C'est une primitive protocolaire.

---

## 6. Comparaison avec d'autres protocoles P2P

### 6.1 IPFS — Content Addressing + Name Resolution

IPFS separe strictement :
- **Primitives protocolaires** : DHT (content routing), Bitswap
  (block exchange), IPNS (name resolution), libp2p (transport).
- **Services locaux** : pinning service, MFS (mutable file system),
  gateway HTTP locale.
- **Applications** : IPFS Companion, Brave browser integration,
  applications Web3.

La recherche dans IPFS est **entierement externe**. Il n'y a pas de
search dans le protocole. IPFS-search.com est un indexeur
centralise qui crawle le DHT. C'est un choix delibere : IPFS est
un systeme de stockage, pas un systeme de decouverte.

**Lecon pour SBFB :** IPFS a peut-etre ete trop puriste. L'absence
de recherche native a pousse les utilisateurs vers des services
centralises (Pinata, Web3.storage, IPFS-search). SBFB peut faire
mieux en integrant la recherche locale comme service daemon. Mais
la decouverte reseau (SearchManifest) doit etre opt-in et privacy-
preserving, pas un DHT crawlable.

### 6.2 SSB (Secure Scuttlebutt) — Local-First Indexes

SSB indexe localement tous les messages recus. Chaque client
construit ses propres vues materialisees (par auteur, par canal,
par mention). La recherche est locale par construction.

- **Indexes locaux** : flumeview (vues materialisees sur le log
  append-only), ssb-search (FTS sur les messages locaux).
- **Pas de recherche reseau** : un utilisateur ne peut chercher que
  dans les messages de ses pairs directs (2 hops max).
- **Pas de score de confiance protocolaire** : la confiance est
  social-graph-based (je suis ton ami → je vois tes messages).

**Lecon pour SBFB :** Le modele SSB confirme que l'index local est
un service daemon (pas une app). Mais SSB n'a pas de mecanisme de
decouverte au-dela du social graph. Le SearchManifest de SBFB
comble ce gap tout en restant opt-in.

### 6.3 AT Protocol (Bluesky) — AppView Architecture

AT Protocol separe explicitement :
- **PDS (Personal Data Server)** : stockage + authentification +
  signature. Equivalent du daemon SBFB.
- **Relay (firehose)** : collecte les evenements de tous les PDS.
  Equivalent du feed gossip SBFB.
- **AppView** : indexe les evenements, construit les vues
  materialisees (timeline, search, trending), sert les API.
  **C'est un service separe du PDS.**

La recherche dans AT Protocol est dans l'AppView, pas dans le PDS.
C'est un service applicatif qui indexe le relay firehose. Mais :
- Un AppView peut etre self-hosted (chacun son index).
- La separation PDS/AppView est motivee par l'echelle (millions
  d'utilisateurs) — un PDS ne peut pas indexer tout le reseau.
- La Proof Card n'a pas d'equivalent direct — les "labels" (trust
  & safety) sont des AppView concerns.

**Lecon pour SBFB :** A l'echelle SBFB pre-launch (< 1000 apps),
l'AppView et le PDS peuvent etre le meme processus (le daemon).
La separation fonctionnelle existe en modules, pas en processus.
Si le reseau atteint 50K+ apps, le SearchManifest gossip remplace
l'AppView centralise : chaque noeud publie volontairement un
sous-ensemble, et les pairs aggregent localement.

### 6.4 BitTorrent — DHT + Magnet Links

BitTorrent a un DHT (Mainline DHT) pour la resolution de hashes
vers des pairs, mais **aucune recherche native**. La decouverte
passe par des indexeurs externes (The Pirate Bay, etc.).

Les "magnet links" sont une forme primitive de SearchManifest : un
noeud publie un lien auto-descriptif vers un contenu. Mais il n'y a
pas de signature, pas de verification, pas de score.

**Lecon pour SBFB :** Le SearchManifest est un magnet link signe
et opt-in. C'est une amelioration significative sur le modele
BitTorrent.

---

## 7. Synthese : le spectre protocolaire-applicatif

```
Primitif pur          Infrastructure daemon        App externe
<-- reseau -->        <-- service local -->         <-- UI -->

DOMAIN_SEARCH_*       FTS5 index                   sbfb-search app
SearchManifest wire   search.rs module              ProofCard composant HTML
gossip topic          search API endpoint           score visualisation
sign/verify           ProofCard compute             UX interactions
SearchManifestPub-    indexation boot/incremental
lished feed op        manifest cache DashMap
                      proof_card API endpoint
                      bridge method search
                      bridge method proof_card_get
                      rate limiter
                      PoW anti-spam
```

---

## 8. Recommandations

### 8.1 Ce qui reste daemon (S70-S71 principalement)

1. **Index FTS5 + module search.rs** → `nexus-coordinator-rs`
   Meme pattern que `public_feed.rs`. Migration M15. Tests unitaires
   dans le crate coordinator.

2. **API search** → `nexus-shell-daemon` (`search_api.rs`)
   Meme pattern que `feed_sync.rs` ou `storage_api.rs`. Wire dans
   `http.rs`.

3. **ProofCard data model + computation** → `nexus-coordinator-rs`
   (`proof_card.rs`). Formule deterministe, versionnee, documentee.

4. **API proof-card** → `nexus-shell-daemon` dans un module API.

5. **Indexation boot + incrementale** → `nexus-shell-daemon`
   (`search_index.rs` ou integration dans `runtime.rs`).

### 8.2 Ce qui est nouveau primitif protocolaire (S72)

1. **`DOMAIN_SEARCH_MANIFEST_V1`** → `nexus-core-rs/src/canonical.rs`
   Nouveau domain separation tag.

2. **`SearchManifest` struct** → `nexus-core-rs` ou
   `nexus-coordinator-rs` (selon si d'autres crates en ont besoin).
   Sign/verify via `canonical_bytes` pattern.

3. **`SearchManifestPublished`** → `nexus-coordinator-rs/src/public_feed.rs`
   Nouvelle variante dans `PublicFeedOperation` (ou raw-op Value).

4. **Gossip topic `search-manifest/v1`** → constante dans
   `nexus-core-rs` ou `nexus-shell-daemon-core`, meme pattern que
   `CURATOR_TOPIC_SEED`.

### 8.3 Ce qui sort vers les apps externes

1. **App `sbfb-search`** → `examples/sbfb-search/`
   HTML+JS vanilla, utilise bridge `search` + `proof_card_get`.

2. **Composant ProofCard UI** → dans l'app sbfb-search et/ou
   dans le shell Browse (composant React).

3. **Formule de score custom** → les apps peuvent ignorer le score
   daemon et calculer le leur a partir des facteurs retournes.

### 8.4 Bridge : methodes a ajouter

| Methode | Type | Justification |
|---|---|---|
| `search` | lecture seule | Interroge l'index FTS5 local. Privacy-preserving (requete locale). Parametres : `q`, `limit`, `offset`. |
| `proof_card_get` | lecture seule | Retourne ProofCard + facteurs pour un `project_id`. Le score et les facteurs ensemble permettent affichage custom. |

**A NE PAS ajouter :**
- `search_manifest_publish` — la publication de manifest est une
  action d'operateur de noeud, pas une action d'app iframe. Les apps
  n'ont pas a publier d'index au nom du noeud.
- `search_manifest_list` — potentiellement utile a terme, mais pas
  dans le MVP. Les manifests distants enrichissent les resultats de
  `search`, pas besoin d'une methode separee.

---

## 9. Impact sur la roadmap S70-S72

### 9.1 Aucun changement structurel majeur

La roadmap actuelle est deja bien alignee avec cette analyse :
- S70 Phases A-B sont de l'infrastructure daemon (correct)
- S70 Phase C est du bridge (correct)
- S70 Phase D est une app externe (correct)
- S71 Phases A-B sont de l'infrastructure daemon (correct)
- S71 Phase C est un mix daemon/app (le composant ProofCard UI est
  app, mais l'integration dans Browse est shell)
- S72 Phases A-C sont des primitives + infrastructure (correct)
- S72 Phase D est du hardening (correct)

### 9.2 Ajustements recommandes

1. **S70 Phase A** : Placer `search.rs` dans `nexus-coordinator-rs`,
   pas dans `nexus-shell-daemon`. Le daemon wires l'API, mais la
   logique d'indexation et de recherche est dans le coordinator crate
   (meme pattern que `public_feed.rs`).

2. **S71 Phase A** : Ajouter un champ `formula_version: u16` dans
   `ProofCard`. Permet aux apps de detecter un changement de formule
   sans casser.

3. **S71 Phase B** : `proof_card_get` retourne les facteurs bruts
   EN PLUS du score. Pas juste le score. Les facteurs sont :
   `has_provenance`, `is_open_source`, `freshness_state`,
   `curator_count`, `license_spdx`, `archive_hash_present`,
   `risk_factors[]`. Chaque facteur est un fait verifiable.

4. **S72 Phase A** : Le `SearchManifest` devrait vivre dans
   `nexus-core-rs` (pas coordinator) car c'est un wire format que
   d'autres implementations du protocole consommeraient. Le domain
   tag est deja dans `canonical.rs` (core). Le struct devrait etre
   adjacent.

5. **S72** : La decision `DOMAIN_SEARCH_MANIFEST_V1` est un **gel
   protocolaire**. Une fois deploye sur le gossip, le domain tag
   ne peut plus changer sans invalider les signatures existantes.
   Le format du manifest doit etre soigneusement designe en Phase A
   (meme rigueur que `FeedEntryCanonical`).

### 9.3 Risques identifies

| Risque | Probabilite | Impact | Mitigation |
|---|---|---|---|
| FTS5 performance > 100ms a 500+ docs | Faible | Gate Tantivy deja prevue | Benchmarker en Phase A |
| Formule ProofCard contestee par la communaute | Moyenne | Changement de formule = changement d'API | `formula_version` + facteurs bruts |
| SearchManifest spam | Haute (gossip ouvert) | Degrade la qualite des resultats | Rate limiter + PoW + verification signature |
| Privacy leak via SearchManifest | Moyenne | Un manifest revele quels projets un noeud heberge | Opt-in explicite + documentation privacy |
| Inflation du daemon (trop de modules) | Faible | Build time + surface d'attaque | Module search.rs isole, feature gate possible |

---

## 10. Conclusion

RRV n'est ni purement protocolaire ni purement applicatif. Il se
decompose en trois couches :

1. **Couche protocolaire** (S72) : `SearchManifest` est un nouveau
   wire format signe et gossipe, au meme rang que `CuratorListEntry`
   ou `FeedEntry`. C'est la seule partie de RRV qui engage le
   protocole reseau et doit etre gelee avec la meme rigueur.

2. **Couche service daemon** (S70-S71) : l'index FTS5, le calcul
   ProofCard, et les API search/proof-card sont des services locaux
   que le daemon fournit a partir de donnees qu'il possede deja.
   Ils ne modifient pas le protocole reseau. Ils enrichissent
   l'experience de chaque noeud.

3. **Couche applicative** (S70-S71 UI) : l'app sbfb-search, le
   composant ProofCard HTML, et toute logique d'affichage sont des
   apps externes qui consomment les services daemon via le bridge.

Cette decomposition est plus nuancee que Factory (qui est
entierement service daemon) ou Babel (qui est entierement app
externe). Mais elle est coherente avec les patterns existants du
codebase, et elle respecte le principe fondateur : **le daemon
indexe ce qu'il possede, le reseau distribue ce qui est signe,
les apps affichent ce qui est expose.**
