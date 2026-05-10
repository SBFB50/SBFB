# Research — P2P App Storage Replication via iroh-docs

**Date** : 2026-05-10
**Contexte** : Ideas Hub (sbfb-ideas) stocke actuellement les idees
et votes dans un HashMap local + SQLite (coordinator DB). Chaque noeud
voit uniquement ses propres donnees. La question : comment repliquer
ces donnees entre tous les noeuds SBFB via iroh-docs pour que chaque
pair voie le meme etat.

**Confiance globale** : MEDIUM-HIGH (iroh-docs 0.98 documente +
code source SBFB verifie, alternatives basees sur WebSearch)

---

## 1. iroh-docs : semantique exacte (0.98)

### 1.1 Modele de donnees

Chaque document iroh-docs est un **namespace** identifie par un
`NamespaceId` (cle publique Ed25519 du namespace). Les entrees sont
indexees par un triplet `(namespace, author, key)` :

- **NamespaceId** : identifie le document (un "replicas set")
- **AuthorId** : cle publique Ed25519 de l'auteur qui a ecrit l'entree
- **Key** : tableau d'octets arbitraire (les apps utilisent des
  prefixes comme `"task:"`, `"claim:"`, `"result:"`)
- **Value** : hash BLAKE3 + taille + timestamp (le contenu reel
  est un blob dans le store iroh-blobs)

**Critique** : iroh-docs stocke TOUTES les paires (author, key)
independamment. Deux auteurs ecrivant la meme cle produisent DEUX
entrees distinctes, pas un conflit. La "resolution de conflit" est
une vue applicative, pas un mecanisme protocole.

### 1.2 Resolution de conflits — Last-Write-Wins (LWW)

iroh-docs est LWW **par (author, key)** :

- Pour un meme auteur et une meme cle, seule la derniere ecriture
  (timestamp le plus recent) est conservee
- Pour des auteurs differents ecrivant la meme cle, **toutes les
  entrees coexistent** dans le store

L'API `Query` offre deux modes de lecture :

| Query | Comportement |
|---|---|
| `Query::all()` | Retourne TOUTES les entrees (N auteurs x M cles) |
| `Query::single_latest_per_key()` | Retourne 1 entree par cle : celle avec le timestamp le plus recent tous auteurs confondus |
| `Query::author(id)` | Filtre par auteur specifique |
| `Query::key_prefix(p)` | Filtre par prefixe de cle |
| `Query::key_exact(k)` | Filtre par cle exacte |

**Ordre des filtres pour `single_latest_per_key`** : le filtre cle
est applique AVANT le groupement, le filtre auteur APRES. Cela
signifie que `single_latest_per_key().key_prefix("ideas/")` retourne
la derniere ecriture par cle parmi toutes les cles commencant par
`ideas/`.

**SortBy** : `KeyAuthor` (tri par cle puis auteur) ou `AuthorKey`
(tri par auteur puis cle). Utile pour l'iteration ordonnee.

### 1.3 LiveEvent — notification temps reel

Le subscribe sur un document emet 7 variantes :

| Variante | Semantique |
|---|---|
| `InsertLocal` | Entree ecrite localement |
| `InsertRemote { from, entry, content_status }` | Entree recue d'un pair |
| `ContentReady(hash)` | Le blob correspondant est telecharge et disponible |
| `PendingContentReady` | Tous les downloads en attente sont termines |
| `NeighborUp(PublicKey)` | Un pair a rejoint le swarm du document |
| `NeighborDown(PublicKey)` | Un pair a quitte le swarm |
| `SyncFinished(SyncEvent)` | Reconciliation terminee avec un pair |

### 1.4 Sync / Bootstrap / Catchup

Le protocole de sync utilise la **reconciliation d'ensembles par
plages** (Meyer 2022) :

1. Les pairs partitionnent recursivement leur ensemble d'entrees
2. Ils comparent les fingerprints de chaque partition
3. Les partitions qui different sont echangees

**Propriete cle** : deux pairs deja synchronises n'echangent qu'un
seul fingerprint pour le confirmer. Le cout est proportionnel au
delta, pas a la taille totale.

**Bootstrap d'un nouveau noeud** :

1. Le noeud recoit un `DocTicket` (contient namespace + adresses
   des pairs existants)
2. Il importe le ticket via `Docs::import(ticket)` ou
   `import_and_subscribe(ticket)`
3. Le protocole de sync se declenche automatiquement : le nouveau
   noeud recoit toutes les entrees existantes via la reconciliation
4. Les blobs (contenu des valeurs) sont telecharges en parallele
   via iroh-blobs
5. Les `LiveEvent::InsertRemote` notifient au fur et a mesure

**Pas de replay gossip** : la sync iroh-docs ne passe PAS par le
gossip replay. Le gossip sert a notifier les pairs qu'un document
a change (trigger de sync), mais les donnees elles-memes transitent
par le protocole de reconciliation dedie. Le gossip est le "signal",
pas le "transport des donnees".

### 1.5 ShareMode et capabilities

| Mode | Permission | Ticket contient |
|---|---|---|
| `ShareMode::Write` | Lecture + ecriture (le namespace secret est partage) | Namespace secret + adresses |
| `ShareMode::Read` | Lecture seule | Namespace public key + adresses |

**Important** : un ticket Write donne un acces ecriture COMPLET au
namespace. N'importe quel pair avec le ticket peut creer son propre
AuthorId et ecrire des entrees. Il n'y a pas de granularite
cle-par-cle dans les permissions iroh-docs.

### 1.6 Ce que SBFB utilise deja

Le codebase utilise iroh-docs pour les **task documents** :

- 1 namespace par projet
- Le coordinateur ecrit les `TaskEntry` sous `"task:{id}"`
- Les workers ecrivent les `ClaimEntry` sous `"claim:{id}"` et
  les `ResultEntry` sous `"result:{id}"`
- Chaque acteur a son propre `AuthorId`
- Le coordinateur observe les `InsertRemote` via `subscribe()`

Le storage applicatif (bridge `storage_get`/`storage_set`) est
**local uniquement** : `HashMap<String, HashMap<String, Value>>`
en memoire + persist SQLite dans coordinator DB. Aucune replication
P2P.

Source : `crates/nexus-shell-daemon/src/storage_api.rs` (Sprint 56
Phase C).

---

## 2. Alternatives CRDT evaluees

### 2.1 OrbitDB (IPFS)

- **Transport** : IPFS + Libp2p Pubsub
- **Modele** : operation-based CRDT sur ipfs-log (append-only DAG)
- **Types de DB** : events (log), documents (JSON), keyvalue
- **Multi-writer** : ACL explicite, createur = admin par defaut
- **Conflit** : Merkle-CRDT (merge automatique)
- **Verdict** : JavaScript uniquement (Go partiel via Berty).
  Stack IPFS orthogonale a iroh. Non pertinent — SBFB n'utilise
  pas IPFS et ajouter une deuxieme stack P2P serait aberrant.
  **REJETE.**

### 2.2 Automerge

- **Transport** : agnostique (automerge-repo gere la sync)
- **Modele** : JSON-like CRDT (map, list, text)
- **Multi-writer** : natif, merge automatique sur structure
- **Performance** : Automerge 3 = ~10x reduction memoire vs v2.
  Rust core + WASM/JS bindings
- **Conflit** : resolution semantique par type (LWW pour scalaires,
  merge pour maps, positional insert pour listes/texte)
- **Verdict** : excellent pour la collaboration temps reel sur des
  documents structures. MAIS : ajouter automerge = deuxieme
  couche de replication parallele a iroh-docs. Complexite non
  justifiee pour un use case "ideas + votes" qui est fondamentalement
  un key-value store, pas un document collaboratif.
  **REJETE pour Ideas Hub. Candidat pour un futur use case
  collaborative editing (post-v1.0).**

### 2.3 Yjs

- **Transport** : agnostique (WebRTC, WebSocket, Hyper)
- **Modele** : YATA algorithm, optimise pour le texte
- **Performance** : le CRDT le plus rapide pour l'edition de texte
- **Multi-writer** : natif
- **Verdict** : JavaScript uniquement. Optimise pour le texte
  collaboratif, pas pour un key-value store d'idees/votes.
  Meme probleme qu'Automerge : deuxieme stack parallele.
  **REJETE.**

### 2.4 GUN.js

- **Transport** : WebSocket/WebRTC entre pairs, relay servers
- **Modele** : graphe decentralise, LWW par champ
- **Multi-writer** : natif, SEA framework pour crypto
- **Anti-spam** : PoW optionnel (SEA), rate limiting basique
- **Limitations** : localStorage (5MB), relays = bottleneck, API
  instable, communaute reduite, JavaScript uniquement
- **Verdict** : stack fragile, pas de binding Rust.
  **REJETE.**

### 2.5 Conclusion alternatives

**iroh-docs est le bon choix.** SBFB l'utilise deja pour les task
documents, les semantiques LWW + multi-author + sync automatique
correspondent exactement au use case Ideas Hub, et il n'y a aucune
raison d'ajouter une deuxieme stack de replication. La question
n'est pas "quel CRDT utiliser" mais "comment modeler les idees et
votes dans le data model iroh-docs existant".

---

## 3. Anti-spam sur stockage replique

### 3.1 iroh-docs natif

iroh-docs **n'a aucun mecanisme anti-spam natif**. Tout pair avec
un ticket Write peut ecrire des entrees sans limite. C'est par
design : iroh-docs est une brique de stockage, pas un systeme
applicatif.

### 3.2 Mecanismes SBFB existants applicables

SBFB a deja un arsenal anti-spam construit pour le gossip et les
taches :

| Mecanisme | Sprint | Applicable au storage |
|---|---|---|
| Hashcash PoW (gossip publish) | S19 | Oui — gate l'ecriture |
| Age witness (gossip join) | S22 | Non (specifique gossip) |
| Sybil resistance (Ed25519 identite) | S22 | Oui — 1 identite = 1 auteur |
| Rate limiting GCRA (worker-side) | S21 | Oui — adapter au storage |
| Curator lists Ed25519 | S7+ | Oui — reputation gate |
| Kudos score | S16+ | Oui — poids des actions |

### 3.3 Strategie recommandee pour le storage P2P

**Couche 1 — Write ticket = capability** : seuls les noeuds qui
possedent le ticket Write du namespace de l'app peuvent ecrire.
Le ticket n'est distribue qu'aux noeuds du reseau SBFB via le
bootstrap normal (gossip + pkarr). Un attaquant externe doit
d'abord rejoindre le reseau.

**Couche 2 — Rate limit per-author** : le daemon local refuse de
relayer les ecritures au-dela d'un quota par AuthorId. Exemple :
max 10 idees/heure, max 50 votes/heure. Le rate limiter utilise
le `governor` GCRA deja en workspace dep.

**Couche 3 — Validation applicative** : le daemon qui recoit une
ecriture (via `InsertRemote`) valide le contenu avant de l'accepter
dans sa vue locale. Idee sans titre ? Ignoree. Vote sans idee
correspondante ? Ignore. Entree > 10KB ? Ignoree.

**Couche 4 (post-v1.0) — Reputation gate** : les actions d'un
auteur sont ponderees par son score Kudos. Un auteur avec 0 Kudos
ne peut pas voter. Un auteur avec Kudos negatif est ignore.

### 3.4 Limitation structurelle

iroh-docs replique TOUT le namespace entre pairs. Un spammeur avec
un ticket Write peut ecrire N entrees qui seront propagees a tous
les pairs. Le rate limit (couche 2) est local : il empeche le noeud
local de propager, mais le spammeur peut avoir ses propres noeuds.

**Mitigation** : chaque daemon valide les entrees recues
(couche 3). Les entrees qui violent les regles sont stockees dans
iroh-docs (on ne peut pas les empecher d'etre repliquees) mais
ne sont pas exposees dans l'UI de l'app. Le cout du spam est le
stockage, pas la visibilite.

**Futur** : si le spam stockage devient un probleme, un mecanisme
de "purge consensuelle" (N curators votent pour supprimer un auteur)
peut nettoyer les entrees. Post-v1.0.

---

## 4. Data model optimal pour Ideas Hub

### 4.1 Principes de design

1. **1 namespace iroh-docs par app** : l'app Ideas Hub a son propre
   document, distinct des task documents de chaque projet
2. **Chaque noeud ecrit avec son propre AuthorId** : l'identite de
   l'auteur est cryptographique, pas un champ texte
3. **Les cles encodent la semantique** : prefixes `ideas/`, `votes/`
4. **Pas de conflits par construction** : le schema utilise des cles
   uniques qui ne peuvent pas entrer en conflit entre auteurs

### 4.2 Schema propose

```
Namespace: ideas-hub-v1 (1 seul pour toute l'app)
Author: chaque noeud utilise son AuthorId default

--- Idees ---
Key:   ideas/{uuid}
Value: JSON { title, description, tags[], created_at, version: 1 }
Rule:  Seul l'auteur de l'entree (AuthorId) peut la modifier/supprimer
       L'AuthorId EST l'identite de l'auteur — pas de champ author_key

--- Votes ---
Key:   votes/{idea_uuid}
Value: JSON { timestamp }
Rule:  1 AuthorId = 1 vote par idee (la cle est commune mais chaque
       auteur ecrit sa propre entree). Utiliser Query::all() avec
       key_exact("votes/{uuid}") retourne N entrees = N votes.

--- Metadonnees (optionnel, post-MVP) ---
Key:   profile/{author_id_hex}
Value: JSON { display_name, bio }
Rule:  Seul le proprietaire de l'AuthorId ecrit cette cle
```

### 4.3 Pourquoi ce schema fonctionne sans conflits

**Idees** : la cle `ideas/{uuid}` contient un UUID genere
localement. Deux noeuds ne genereront jamais le meme UUID. Chaque
idee est ecrite par un seul auteur sous son AuthorId. Aucun conflit
possible.

**Votes** : la cle `votes/{idea_uuid}` est la meme pour tous les
votants, MAIS chaque votant ecrit avec un AuthorId different. Dans
iroh-docs, `(author_A, "votes/uuid-42")` et
`(author_B, "votes/uuid-42")` sont deux entrees distinctes. Pour
compter les votes :

```rust
// Compter les votes pour une idee
let entries = doc.get_many(Query::key_exact(b"votes/{idea_uuid}")).await?;
let vote_count = entries.len(); // 1 entree par auteur = 1 vote
```

**Desvote** : l'auteur ecrit une valeur vide ou un tombstone JSON
`{ "retracted": true }` sous sa propre entree. Le comptage ignore
les entrees retractees.

**Suppression d'idee** : l'auteur ecrit un tombstone
`{ "deleted": true }` sous `ideas/{uuid}`. Les autres noeuds
filtrent les idees marquees deleted. La cle reste dans iroh-docs
(pas de suppression physique dans un CRDT) mais l'UI ne l'affiche
plus.

### 4.4 Queries pour l'UI

| Operation | Query iroh-docs |
|---|---|
| Lister toutes les idees | `Query::all().key_prefix("ideas/")` puis filtrer `deleted != true` |
| Lire une idee | `Query::single_latest_per_key().key_exact("ideas/{id}")` |
| Compter les votes | `Query::all().key_exact("votes/{id}")` → `.len()` |
| Verifier si j'ai vote | `doc.get_exact(my_author, "votes/{id}")` |
| Mon vote | `doc.set(my_author, "votes/{id}", timestamp_json)` |
| Retirer mon vote | `doc.set(my_author, "votes/{id}", retracted_json)` |

### 4.5 Avantages vs schema actuel

| Aspect | Actuel (local SQLite) | Propose (iroh-docs) |
|---|---|---|
| Visibilite | Noeud local uniquement | Tous les pairs |
| Persistance | Fichier SQLite local | redb + blobs + P2P |
| Identite | Champ texte `author_key` | AuthorId cryptographique |
| Anti-sybil votes | Aucun | 1 AuthorId = 1 vote natif |
| Bootstrap | Pas de donnees au demarrage | Sync automatique |
| Offline | Oui | Oui (ecritures locales, merge au reconnect) |

---

## 5. Architecture d'integration

### 5.1 Vue d'ensemble

```
[App Ideas Hub (iframe)]
    |
    | postMessage bridge
    |
[Shell React (host)]
    |
    | HTTP fetch → daemon
    |
[nexus-shell-daemon]
    |
    +-- storage_api.rs  ← ACTUELLEMENT: HashMap + SQLite
    |                    ← FUTUR: proxy vers iroh-docs namespace
    |
    +-- iroh-docs (Docs protocol)
        |
        +-- sync automatique avec les autres noeuds
```

### 5.2 Plan de migration

**Phase 1 — Namespace dedie (1 sprint, MVP)** :

1. Au demarrage du daemon, creer ou ouvrir un namespace dedie
   `ideas-hub` (stocker le `NamespaceId` dans la config daemon)
2. Modifier `storage_api.rs` pour router les ecritures de l'app
   `sbfb-ideas` vers le namespace iroh-docs au lieu du HashMap
3. `storage_set("ideas/{id}", data)` → `doc.set(author, key, json)`
4. `storage_list("ideas/")` → `doc.get_many_by_prefix("ideas/")`
5. `storage_get("ideas/{id}")` → `doc.get_exact(author, key)`
6. `storage_delete("ideas/{id}")` → ecrire un tombstone
7. Subscribe aux `InsertRemote` pour notifier l'UI via bridge
   push events (`onEvent("storage_changed", ...)`)

**Phase 2 — Generalisation (post-MVP)** :

- Toute app peut demander un namespace replique via un nouveau
  champ dans son manifest (SBFB.json)
- Le daemon cree un namespace par app qui le demande
- Le ticket Write est distribue via gossip aux pairs qui ont
  l'app installee

**Phase 3 — Optimisations (post-v1.0)** :

- Cache local pour eviter les lectures iroh-docs systematiques
- Rate limiting per-author via governor
- Purge consensuelle des spammeurs
- Notifications push granulaires (par prefixe de cle)

### 5.3 Distribution du ticket Write

Le probleme : comment un nouveau noeud obtient le ticket Write
pour le namespace Ideas Hub ?

**Option A (recommandee pre-v1.0) — Ticket dans le manifest** :

Le `DocTicket` en mode Write est embarque dans le zip de l'app
Ideas Hub. Quand un noeud installe l'app, il recupere le ticket
et rejoint le namespace. Simple, pas de coordination runtime.

Risque : le ticket est public (l'archive est open source). Tout
noeud qui a le zip peut ecrire. C'est acceptable pre-v1.0 car le
reseau est petit et l'anti-spam couches 2-3 suffit.

**Option B (post-v1.0) — Ticket via gossip topic** :

Le daemon annonce les namespaces d'apps repliquees sur un gossip
topic dedie. Un nouveau noeud qui installe l'app recoit le ticket
via ce canal. Plus dynamique, permet la rotation des tickets.

**Option C (post-v1.0) — Ticket via verified deploy provenance** :

Le ticket Write est genere par le coordinateur au moment du
`deploy-from-repo` et signe dans la provenance SLSA. Seuls les
noeuds qui verifient la provenance obtiennent le ticket. Maximum
securite, mais complexite elevee.

### 5.4 Impact sur le bridge SDK

Le bridge `sbfb-bridge.js` n'a PAS besoin de changements. Les
methodes `storage_get`, `storage_set`, `storage_list`,
`storage_delete` restent identiques cote client. Le changement
est transparent : le daemon route vers iroh-docs au lieu du HashMap.

Un ajout utile : un push event `storage_remote_update` que le
daemon emet quand il recoit un `InsertRemote`. L'app peut alors
recharger ses donnees via `listStorage()`.

---

## 6. Risques et limitations

### 6.1 Risques critiques

**R1 — Pas de suppression physique** : iroh-docs est un CRDT
append-only. On peut ecrire des tombstones mais pas supprimer des
entrees. Le stockage croit monotoniquement. Pour Ideas Hub
(milliers d'entrees max), ce n'est pas un probleme. Pour une app
de chat (millions de messages), ca le serait.

**R2 — Ticket Write = acces total** : iroh-docs n'a pas de
permissions par cle. Un pair malveillant avec le ticket Write peut
ecrire n'importe quelle cle, y compris ecraser l'idee d'un autre
(s'il connait son AuthorId, il ne peut pas ecrire sous cet AuthorId
sans la cle privee — mais il peut ecrire la meme cle sous son
propre AuthorId et polluer le namespace). Le schema propose mitigue
ca : les cles UUID sont imprevisibles, les votes sont par-author
natif.

**R3 — Clock skew** : le LWW utilise le timestamp de l'auteur.
Un noeud avec une horloge avancee "gagne" tous les conflits LWW.
Pour Ideas Hub, les conflits LWW sont rares (chaque idee est
ecrite par un seul auteur). Les votes n'ont pas de conflit LWW
car chaque AuthorId ecrit sa propre entree.

### 6.2 Risques moderes

**R4 — Taille du namespace** : si 10 000 noeuds postent chacun
10 idees, le namespace contient 100 000 entrees. iroh-docs
utilise redb (embedded B-tree) qui gere bien ce volume. La sync
initiale d'un nouveau noeud transferera ~100K entrees x ~500B
= ~50MB, acceptable.

**R5 — Latence de convergence** : la sync iroh-docs n'est pas
instantanee. Un vote poste sur le noeud A peut prendre 5-30s pour
apparaitre sur le noeud B (selon la latence reseau + intervalle de
sync). L'UI doit afficher "en cours de synchronisation" pendant ce
delai.

**R6 — Donnees orphelines** : si un noeud poste une idee puis
quitte le reseau definitivement, l'idee reste dans le namespace
mais personne ne peut la modifier/supprimer. C'est acceptable
pour Ideas Hub (les tombstones ne sont ecrivables que par
l'AuthorId original).

### 6.3 Risques faibles

**R7 — Migration de schema** : le pre-launch protocol policy
autorise la redefinition du schema v1 sans compat historique. Pas
de risque de migration complexe avant v1.0.

**R8 — Conflit de namespace** : deux instances de Ideas Hub
(forks differents) partageant le meme namespace pourraient
melanger leurs donnees. Mitigation : le namespace est derive du
hash de l'app ou de sa cle de deploy, rendant chaque instance
unique.

---

## 7. Recommandation

### Pour le MVP Ideas Hub pre-v1.0

1. **Creer un namespace iroh-docs dedie** au demarrage du daemon
   pour l'app Ideas Hub
2. **Modifier `storage_api.rs`** pour router les ecritures de
   `sbfb-ideas` vers ce namespace au lieu du HashMap local
3. **Utiliser le schema section 4.2** : cles `ideas/{uuid}` et
   `votes/{idea_uuid}`, 1 AuthorId par noeud
4. **Embarquer le DocTicket Write dans le zip** de l'app pour le
   bootstrap
5. **Ajouter un push event** `storage_remote_update` dans le
   bridge pour la reactivite UI
6. **Rate limit local** : 10 idees/h, 50 votes/h par AuthorId
   via governor

### Budget estimate

- Modification `storage_api.rs` : ~200 LOC Rust
- Namespace lifecycle (create/open/share) : ~150 LOC Rust
- InsertRemote listener + push event : ~100 LOC Rust
- Adaptation `app.js` pour le push event : ~20 LOC JS
- Tests : ~300 LOC Rust
- **Total : ~770 LOC, 1 phase d'un sprint**

### Ce qui ne change PAS

- Le bridge SDK (`sbfb-bridge.js`) : aucune modification
- L'API HTTP du daemon : les routes restent identiques
- Le schema de l'app `app.js` : memes cles, memes operations
- Les autres apps : leur storage reste local sauf opt-in

---

## Sources

- [iroh-docs Documentation](https://docs.iroh.computer/protocols/documents) — MEDIUM confidence (description haut-niveau, details techniques insuffisants)
- [iroh-docs GitHub](https://github.com/n0-computer/iroh-docs) — HIGH confidence (code source)
- [iroh-docs crates.io](https://crates.io/crates/iroh-docs) — HIGH confidence (version 0.98 confirmee)
- [iroh-docs docs.rs Query API](https://docs.rs/iroh-docs/latest/iroh_docs/store/struct.Query.html) — HIGH confidence (API Rust directe)
- [iroh-docs docs.rs LiveEvent](https://docs.rs/iroh-docs/latest/iroh_docs/engine/enum.LiveEvent.html) — HIGH confidence
- Code source SBFB : `crates/nexus-core-rs/src/docs.rs`, `src/task.rs`, `src/node.rs` — HIGH confidence
- Code source SBFB : `crates/nexus-shell-daemon/src/storage_api.rs` — HIGH confidence
- [OrbitDB GitHub](https://github.com/orbitdb/orbitdb) — MEDIUM confidence (WebSearch)
- [Automerge](https://automerge.org/) — MEDIUM confidence (WebSearch)
- [Yjs](https://yjs.dev/) — MEDIUM confidence (WebSearch)
- [GUN.js](https://gun.eco/) — LOW confidence (WebSearch)
- [p2panda Access Control CRDT](https://p2panda.org/2025/08/27/notes-convergent-access-control-crdt.html) — MEDIUM confidence (patterns P2P access control)
- [libp2p Privacy-preserving spam protection](https://github.com/libp2p/specs/issues/374) — LOW confidence (discussion, pas implementation)
