# Prior Art : Neutralite Protocolaire dans les Reseaux P2P Matures

> **Date** : 2026-05-19
> **Contexte** : Recherche pour SBFB — comment garder le daemon/noeud
> protocolairement neutre tout en permettant des outils applicatifs
> riches (Factory, RRV) qui ne sont PAS integres au daemon.
> **Methode** : WebSearch systematique sur 5 protocoles P2P matures,
> documentation officielle et specs. Pas de speculation — les lacunes
> sont signalees explicitement.

---

## 1. IPFS / Kubo

### 1.1 Frontiere daemon vs outils

Le daemon Kubo (Go) est un noeud IPFS monolithique qui integre 8
sous-couches protocolaires : identite, reseau, routage, echange
(Bitswap), objets (DAG), fichiers (UnixFS), nommage (IPNS),
application.

**Ce que le daemon fait :**
- Bitswap : echange de blocs content-addressed entre pairs
- DHT Kademlia (Amino) : routage et decouverte de providers
- Pinning local : persistance de CID contre le garbage collector
- MFS (Mutable File System) : systeme de fichiers mutable local
  au-dessus du DAG immuable
- UnixFS : encodage fichiers/repertoires en DAG merkle
- IPNS : noms mutables bases sur PKI (hash de cle publique)
- HTTP Gateway : service de contenu CID vers navigateurs web

**Ce que les outils externes font :**
- **IPFS Cluster** : orchestration de pinning distribue — tourne
  comme sidecar independant du daemon, communique via l'API HTTP
  du daemon (localhost:5001). Forme son propre reseau prive
  libp2p avec `cluster_secret`. Peut gerer des millions de pins
  sur des centaines de daemons.
- **IPFS Desktop** : GUI Electron, wrapper autour de l'API RPC
- **Pinata, Filebase** : services de pinning tiers, implementent
  l'IPFS Pinning Service API (spec vendor-agnostique)
- **ipfs-cluster-follow** : client simplifie pour noeuds suiveurs

### 1.2 API surface du noeud

Le daemon expose une **API RPC HTTP** (port 5001 par defaut) qui
miroir exactement les commandes CLI. Categories principales :
- `add` / `cat` / `get` : ajout et recuperation de contenu
- `dag put` / `dag get` / `dag resolve` : manipulation DAG brut
- `block put` / `block get` : blocs bruts
- `pin add` / `pin rm` / `pin ls` : gestion du pinning local
- `files/*` (MFS) : `cp`, `ls`, `mkdir`, `mv`, `read`, `write`,
  `flush`, `stat` — systeme de fichiers mutable
- `name publish` / `name resolve` : IPNS
- `swarm peers` / `swarm connect` : gestion du reseau
- `dht findpeer` / `dht findprovs` : requetes DHT
- `routing provide` / `routing findprovs` : content routing

### 1.3 Recherche et decouverte

**Protocolaire (partiel)** : la DHT Kademlia permet de trouver
des providers pour un CID donne (`findprovs`). C'est de la
decouverte par hash, pas de la recherche semantique.

**Applicatif** : il n'existe aucune recherche textuelle ou
semantique au niveau du protocole. Les moteurs de recherche IPFS
(ipfs-search.com) sont des services externes qui crawlent le
reseau et indexent le contenu.

### 1.4 Creation de contenu

**Protocolaire** : `ipfs add` transforme un fichier en DAG merkle
UnixFS et le rend disponible. C'est une primitive generique —
le daemon ne connait pas la semantique du contenu.

**Applicatif** : tout outil de creation (editeurs, CI/CD,
pipelines de build) est externe. Le daemon ne sait pas si le
contenu est un site web, un dataset, ou une image.

### 1.5 Modele d'extensibilite

- **API RPC plate** : toute extension interagit via l'API HTTP,
  pas de systeme de plugins natif dans Kubo.
- **Sidecar pattern** : IPFS Cluster est le modele canonique —
  un processus separe qui orchestre le daemon sans le modifier.
- **Pinning Service API** : spec standardisee pour les services
  de pinning tiers — le daemon sait deleguer le pinning a un
  service distant.
- **Gateway spec** : standardisation de l'interface HTTP Gateway
  pour servir du contenu IPFS aux navigateurs.

**Sources :**
- [Kubo RPC API v0 reference](https://docs.ipfs.tech/reference/kubo/rpc/)
- [IPFS Specs Architecture](https://github.com/ipfs/specs/blob/main/ARCHITECTURE.md)
- [IPFS Cluster Architecture](https://ipfscluster.io/documentation/deployment/architecture/)
- [Kubo GitHub](https://github.com/ipfs/kubo)
- [IPFS Pinning Service API](https://docs.ipfs.tech/how-to/work-with-pinning-services/)

---

## 2. SSB (Secure Scuttlebutt)

### 2.1 Frontiere serveur vs clients

SSB a une architecture **Kappa** : un log append-only immuable
(le feed) est la source de verite, et des vues materialisees sont
calculees a partir du log.

**Ce que le serveur fait (ssb-server / go-ssb) :**
- Gestion du feed append-only (ecriture, verification signature)
- Gossip replication : synchronisation des logs entre pairs
- Decouverte LAN (multicast) et Internet (pubs, rooms)
- Stockage du log brut (LevelDB historique, puis bipf pour db2)
- API muxrpc pour exposition des fonctions aux clients
- **Systeme de plugins** : les plugins tournent comme processus
  separes, communiquent via muxrpc sur stdio. Peuvent etre
  ecrits dans n'importe quel langage.

**Ce que les clients font (Patchwork, Manyverse, Oasis) :**
- Rendu UI (feeds sociaux, profils, threads)
- Interpretation semantique des messages (type `post`, `vote`,
  `contact`, `about`, etc.)
- Construction des vues utilisateur (timeline, notifications)

### 2.2 API surface du noeud

Le serveur expose via **muxrpc** (RPC multiplexe sur streams) :
- `createFeedStream` / `createLogStream` : lecture du log brut
- `publish` : ecriture d'un message dans le log local
- `get` : recuperation d'un message par hash
- `whoami` : identite du noeud
- `gossip.*` : controle de la replication
- `blobs.*` : stockage et recuperation de blobs (fichiers)
- APIs des plugins charges (chaque plugin expose ses propres
  methodes via muxrpc)

### 2.3 Indexes : serveur ou client ?

**C'est un continuum, pas une frontiere nette.**

- **ssb-db (original)** : indexes dans LevelDB cote serveur,
  integrees au processus ssb-server. Vues par type, par auteur,
  par liens.
- **ssb-db2** : remplacement avec JITDB (indexes just-in-time).
  Les indexes sont des bitvectors et prefix indexes crees
  automatiquement a partir des requetes. Stockes cote serveur
  dans `ssb/db2/indexes/` et `ssb/db2/jit/`.
- **Patchwork** : ssb-db profondement integre dans le client.
  Patchwork ne fonctionne qu'avec son propre ssb-server bundle,
  rendant la separation serveur/client poreuse.
- **Manyverse** : utilise ssb-db2 nativement, meilleure
  separation. Le serveur fournit les indexes, le client les
  consomme.

**Verdict** : les indexes sont cote serveur (dans le processus
ssb-server), mais leur configuration est dictee par les besoins
applicatifs. C'est une faiblesse architecturale reconnue — le
serveur doit connaitre les schemas de messages pour indexer
efficacement.

### 2.4 Recherche et decouverte

**Ni protocolaire ni resolu.** SSB n'a pas d'index global. La
decouverte de contenu depend du graphe social : tu ne vois que
les messages des gens que tu suis (ou que tes suivis suivent, a
2-3 hops). La recherche textuelle est locale, sur les messages
deja repliques. C'est une limitation fondamentale du modele
"subjectif" de SSB.

### 2.5 Creation de contenu

**Protocolaire (minimal)** : `publish` ecrit un message JSON
signe dans le log. Le serveur ne connait pas la semantique —
n'importe quel objet JSON avec un champ `type` est accepte.

**Applicatif** : les clients decident quels types de messages
creer (`post`, `vote`, `contact`, `about`, `channel`, etc.) et
comment les structurer.

### 2.6 Modele d'extensibilite

- **Plugins muxrpc** : mecanisme officiel. Un plugin est un
  processus separe communiquant via muxrpc sur stdio. Peut
  ajouter de nouvelles APIs au serveur sans le modifier.
- **Types de messages libres** : n'importe quel client peut
  inventer de nouveaux types de messages JSON. Les autres clients
  qui ne connaissent pas le type les ignorent.
- **ssb-db2 operators** : DSL de requetes composables pour creer
  des indexes custom sans modifier le core.

**Sources :**
- [Scuttlebutt Protocol Guide](https://ssbc.github.io/scuttlebutt-protocol-guide/)
- [ssb-server GitHub](https://github.com/ssbc/ssb-server)
- [ssb-server Context Handbook](https://handbook.scuttlebutt.nz/guides/ssb-server-context)
- [ssb-db2 GitHub](https://github.com/ssbc/ssb-db2)
- [ssb-plugins GitHub](https://github.com/ssbc/ssb-plugins)

---

## 3. AT Protocol (Bluesky)

### 3.1 Frontiere PDS vs AppViews

AT Proto est le modele le plus explicite de separation entre
infrastructure protocolaire et couche applicative. L'architecture
a trois niveaux est codifiee dans des specs IETF (Internet Drafts
soumis en septembre 2025).

**Ce que le PDS (Personal Data Server) fait :**
- Stockage des repos utilisateurs (arbres Merkle de records)
- Authentification et gestion des sessions
- Proxy transparent : le client envoie toutes ses requetes au
  PDS, qui les route vers le bon service (AppView, labeler, feed
  generator) via des headers HTTP
- Firehose websocket (`com.atproto.sync.subscribeRepos`) :
  stream de tous les changements en temps reel
- Gestion des blobs (images, videos)
- Resolution DID (identite decentralisee)
- **Le PDS ne connait PAS la semantique des records.** Il stocke
  des records Lexicon-types sans les interpreter.

**Ce que les Relays (BGS) font :**
- Crawlent les PDS du reseau
- Agregent les updates dans un firehose global
- Indexation brute (pas semantique)
- Le relay est un "tuyau" : il transmet sans interpreter

**Ce que les AppViews font :**
- Consomment le firehose du relay
- Construisent des vues semantiques : feeds, compteurs de likes,
  threads, profils enrichis
- L'AppView Bluesky interprete les lexicons `app.bsky.*`
- D'autres AppViews peuvent interpreter d'autres lexicons
  (ex: une app de check-in type Foursquare)
- **Analogie officielle** : le relay est un prisme brut, l'AppView
  est la lentille qui focalise les donnees pour une app specifique

**Ce que les Feed Generators font :**
- Services independants qui s'abonnent au firehose
- Implementent un algorithme de curation/classement
- Retournent des "squelettes de feed" (listes d'URIs de posts)
- L'AppView hydrate les squelettes avec les donnees completes
- Un feed generator ne modifie rien — il ne fait que selectionner

**Ce que les Labelers font :**
- Produisent des labels (source DID + sujet URI + valeur string)
- Labels signes separement, pas dans le repo
- Le client decide comment reagir aux labels (cacher, flouter,
  avertir)
- Utilises pour la moderation mais aussi pour l'information
  (pronoms, topics, badges)

### 3.2 API surface du PDS

Le PDS expose une API HTTP (XRPC) avec deux familles de methodes :

**com.atproto.* (protocole neutre)** :
- `repo.createRecord` / `repo.putRecord` / `repo.deleteRecord` :
  CRUD generique sur les records
- `repo.listRecords` / `repo.getRecord` : lecture
- `sync.subscribeRepos` : firehose websocket
- `sync.getRepo` / `sync.getBlob` : synchronisation
- `identity.resolveHandle` : resolution d'identite
- `server.createSession` : authentification

**app.bsky.* (couche applicative Bluesky)** :
- `feed.getTimeline` / `feed.getAuthorFeed` : requetes de feed
- `feed.getFeedSkeleton` : requete vers un feed generator
- `actor.getProfile` : profils utilisateurs
- `notification.listNotifications` : notifications

**Mecanisme de proxy** : les clients n'appellent jamais les
AppViews directement. Toute requete passe par le PDS qui la
proxifie, preservant l'agence utilisateur.

### 3.3 Recherche et decouverte

**Applicatif, pas protocolaire.** La recherche est implementee
par des services d'indexation qui consomment le firehose. Le PDS
ne sait pas chercher. L'AppView Bluesky a un search backend, mais
c'est un service separe, pas une primitive du protocole.

### 3.4 Creation de contenu

**Protocolaire (generique)** : `repo.createRecord` ecrit un
record JSON dans le repo de l'utilisateur. Le PDS ne connait pas
la semantique — il valide la structure Lexicon mais pas le sens.

**Applicatif** : les clients (et les AppViews qui les servent)
definissent les schemas Lexicon pour les types de contenu
(posts, likes, follows, check-ins, etc.).

### 3.5 Modele d'extensibilite

- **Lexicon schema system** : n'importe qui peut definir de
  nouveaux types de records sous son namespace DNS. Les schemas
  sont self-describing. Un client qui ne connait pas un schema
  l'ignore mais conserve le record (open unions). C'est le
  mecanisme d'extensibilite le plus elegant des 5 protocoles.
- **Feed generators pluggables** : n'importe qui peut deployer
  un feed generator, l'enregistrer dans son PDS, et les
  utilisateurs peuvent s'y abonner.
- **Labelers pluggables** : meme principe pour la moderation.
- **AppViews alternatives** : plusieurs AppViews peuvent coexister
  pour le meme namespace de records, offrant des fonctionnalites
  ou politiques differentes (ex: AppViewLite pour faible conso
  ressources).

**Sources :**
- [AT Protocol Overview](https://atproto.com/guides/overview)
- [Federation Architecture (Bluesky docs)](https://docs.bsky.app/docs/advanced-guides/federation-architecture)
- [AppViews (AT Protocol Community Wiki)](https://atproto.wiki/en/wiki/reference/core-architecture/appview)
- [Lexicon Guide](https://atproto.com/guides/lexicon)
- [Feed Generator Starter Kit](https://github.com/bluesky-social/feed-generator)
- [Bluesky Moderation Architecture](https://docs.bsky.app/blog/blueskys-moderation-architecture)
- [Labels Spec](https://atproto.com/specs/label)
- [Custom Schemas Guide](https://docs.bsky.app/docs/advanced-guides/custom-schemas)
- [XRPC Spec](https://atproto.com/specs/xrpc)
- [Introduction to AT Protocol (mackuba.eu)](https://mackuba.eu/2025/08/20/introduction-to-atproto/)

---

## 4. Radicle

### 4.1 Frontiere noeud vs outils

Radicle separe nettement le noeud reseau du CLI et du web UI.

**Ce que le noeud (radicle-node) fait :**
- Gossip protocol : decouverte et annonce de repos et pairs
- Replication Git : synchronisation des repos via le protocole
  Git natif (pack files)
- Stockage de repos bare Git avec namespaces (chaque pair a un
  namespace identifie par son Node ID)
- Gestion des identites cryptographiques (Ed25519)
- Socket de controle Unix : expose un flux d'evenements en
  lecture seule et une API d'introspection
- **Le noeud ne connait PAS la semantique Git.** Il replique
  des refs Git sans savoir ce qu'est un commit, un patch, ou
  un issue.

**Ce que le CLI (rad) fait :**
- `rad init` / `rad clone` / `rad push` / `rad pull` : workflow
  Git classique
- `rad patch create` / `rad patch review` : gestion des patches
  (equivalent pull requests)
- `rad issue create` / `rad issue list` : gestion des issues
- COBs (Collaborative Objects) : structures CRDT stockees comme
  refs Git speciales, interpretees par le CLI, ignorees par le
  noeud
- `rad node start` / `rad node status` : controle du daemon
- `rad cob migrate` : migration des objets collaboratifs

**Ce que le HTTP daemon (radicle-httpd) fait :**
- Gateway HTTP JSON read-only vers le stockage du noeud
- Expose l'etat des repos, patches, issues via API REST
- N'a PAS d'acces en ecriture (dans la version officielle)
- Deploye comme sidecar a cote du noeud

**Ce que le web UI (Radicle Explorer) fait :**
- Interface web pour parcourir les repos, patches, issues
- Consomme l'API de radicle-httpd
- N'a aucun acces direct au noeud

### 4.2 API surface du noeud

Le noeud expose un **socket de controle Unix** (SOCK_STREAM) :
- Flux d'evenements read-only (nouvelles refs, connexions pairs)
- Introspection de l'etat (pairs connectes, repos trackes, seeds)
- Le CLI `rad` communique avec le noeud via ce socket
- L'API HTTP de radicle-httpd accede directement au stockage
  (pas via le socket) en lecture seule

### 4.3 Recherche et decouverte

**Protocolaire (basique)** : le gossip protocol annonce les repos
disponibles sur le reseau. Les pairs decouvrent les repos via le
gossip. Pas de recherche textuelle au niveau protocolaire.

**Applicatif** : le web UI et le CLI permettent de parcourir les
repos. Radicle Explorer (app.radicle.xyz) fournit une interface
de navigation. Pas de moteur de recherche integre.

### 4.4 Creation de contenu

**Protocolaire (generique)** : le noeud replique des refs Git.
N'importe quelle structure Git est supportee.

**Applicatif** : les COBs (Collaborative Objects) sont une
abstraction applicative. Ce sont des CRDTs stockes comme refs Git
speciales (sous `refs/cobs/`). Le noeud les replique comme
n'importe quelles refs — c'est le CLI qui sait les interpreter.
Les patches et issues sont des COBs.

### 4.5 Modele d'extensibilite

- **COBs extensibles** : nouveaux types de COBs possibles sans
  modifier le noeud. Le noeud replique aveuglément les refs Git.
- **Sidecar pattern** : radicle-httpd est un sidecar read-only.
- **Radicle CI** : systeme CI deploye comme service independant,
  declenche par les evenements du noeud.
- **rad-pi** : extensions et skills pour le CLI rad.

**Sources :**
- [Radicle Protocol Guide](https://radicle.dev/guides/protocol)
- [Radicle User Guide](https://radicle.dev/guides/user)
- [Radicle FAQ](https://radicle.dev/faq)
- [Radicle 1.7.0 Release](https://radicle.dev/2026/03/18/radicle-1.7.0)
- [radicle-link RFC 0696 P2P Node](https://github.com/radicle-dev/radicle-link/blob/master/docs/rfc/0696-p2p-node.adoc)
- [Radicle Seeder Guide](https://radicle.xyz/guides/seeder)
- [LWN: Radicle peer-to-peer collaboration with Git](https://lwn.net/Articles/966869/)

---

## 5. BitTorrent

### 5.1 Frontiere protocole vs clients/outils

BitTorrent est le protocole P2P le plus ancien et le plus mature.
La separation protocole/application est naturelle car le protocole
est specifie par des BEPs (BitTorrent Enhancement Proposals)
et les implementations sont multiples.

**Ce que le protocole fait :**
- **Peer wire protocol** : echange de pieces de fichiers entre
  pairs via TCP (choke/unchoke, interested/not-interested,
  have/bitfield, request/piece)
- **DHT Kademlia (Mainline)** : decouverte decentralisee de
  pairs pour un torrent donne (par infohash). BEP 5.
- **PEX (Peer Exchange)** : echange direct de listes de pairs
  entre pairs connectes. BEP 11. Jusqu'a 50 pairs par message.
- **Tracker protocol** : requetes HTTP/UDP a un serveur
  centralisé pour obtenir des listes de pairs
- **Metadata exchange** : transfert des metadonnees torrent
  (info dict) entre pairs. BEP 9.

**Ce que les clients font (qBittorrent, Transmission, etc.) :**
- UI de gestion des torrents (telechargement, seeding, files)
- Gestion des priorites, limites bande passante, scheduling
- Recherche integree (via plugins vers des indexeurs)
- RSS feeds pour telechargement automatique

**Ce que les outils externes font :**
- **Trackers** : registres centralisés de pairs (le protocole
  supporte aussi le trackerless via DHT)
- **Indexeurs** (The Pirate Bay, etc.) : bases de donnees
  de metadonnees torrent, recherche textuelle
- **Bitmagnet** : indexeur self-hosted qui crawle la DHT pour
  decouvrir les torrents sans dependance a un tracker/indexeur
  centralisé
- **Seedbox managers** : orchestration de seeding sur serveurs

### 5.2 API surface du protocole

Le protocole BitTorrent n'a pas d'API au sens REST/RPC. Il definit
des **messages wire** entre pairs :
- Handshake (protocol string + infohash + peer id)
- Messages standard : choke, unchoke, interested, not_interested,
  have, bitfield, request, piece, cancel
- Extension messages via LTEP (BEP 10)
- DHT messages : ping, find_node, get_peers, announce_peer

Les clients exposent leurs propres APIs pour le controle :
- qBittorrent : API HTTP REST
- Transmission : RPC JSON
- rTorrent : XML-RPC
- libtorrent : API C++ avec bindings Python

### 5.3 Recherche et decouverte

**Protocolaire (par hash uniquement)** : la DHT permet de trouver
les pairs qui seedent un torrent donne son infohash. BEP 51
permet de crawler la DHT pour decouvrir les infohashes existants,
mais c'est prevu pour les indexeurs, pas les utilisateurs finaux.

**Applicatif** : toute recherche textuelle/semantique est
externalisee dans les indexeurs (sites web, APIs). Le protocole
ne sait pas chercher — il sait seulement "qui a ce hash ?".

### 5.4 Creation de contenu

**Protocolaire** : creer un torrent = generer un fichier .torrent
(metadonnees + pieces hashes) ou un magnet link (infohash). Le
protocole ne sait pas ce que le contenu represente.

**Applicatif** : les outils de creation de torrents, les
plateformes de publication, les trackers specialises.

### 5.5 Modele d'extensibilite

- **BEP (BitTorrent Enhancement Proposals)** : processus formel
  d'extension du protocole, comparable aux RFC. Chaque BEP a un
  numero et un statut (draft, accepted, final).
- **LTEP (LibTorrent Extension Protocol, BEP 10)** : mecanisme
  generique d'extension des messages wire. Chaque extension
  negocie ses IDs de messages au handshake — pas de registre
  global, juste une convention de nommage (prefixe 2 lettres du
  client). Permet d'ajouter des fonctionnalites (PEX, metadata
  exchange, encryption) sans casser la compatibilite.
- **Separation client/protocole naturelle** : le protocole est
  une spec, pas un daemon. N'importe qui peut implementer un
  client compatible.

**Sources :**
- [BitTorrent Specification (theory.org)](https://wiki.theory.org/BitTorrentSpecification)
- [BEP 5 - DHT Protocol](https://www.bittorrent.org/beps/bep_0005.html)
- [BEP 10 - Extension Protocol](https://www.bittorrent.org/beps/bep_0010.html)
- [BEP 11 - PEX](https://www.bittorrent.org/beps/bep_0011.html)
- [BEP 51 - DHT Infohash Indexing](https://www.bittorrent.org/beps/bep_0051.html)
- [libtorrent Extension Protocol](https://www.libtorrent.org/extension_protocol.html)
- [Bitmagnet](https://bitmagnet.io/)

---

## 6. Patterns Communs Extraits

### 6.1 Le noeud est un "tuyau stupide" (dumb pipe)

**Convergence forte** : dans les 5 protocoles, le noeud/daemon
ne connait PAS la semantique du contenu qu'il transporte.

| Protocole | Ce que le noeud "comprend" | Ce qu'il ignore |
|---|---|---|
| IPFS | CID, DAG, blocs | Type de fichier, application |
| SSB | Feed append-only, JSON signe | Type de message, schema |
| AT Proto | Records Lexicon-types dans repos | Semantique des records |
| Radicle | Refs Git, identites Ed25519 | Commits, patches, issues, COBs |
| BitTorrent | Infohash, pieces, pairs | Contenu des fichiers |

**Pattern** : le noeud manipule des **conteneurs adresses par
contenu ou par identite**, pas des objets semantiques.

### 6.2 La recherche est TOUJOURS applicative

**Convergence unanime** : aucun des 5 protocoles n'integre de
recherche textuelle/semantique au niveau protocolaire.

- IPFS : pas de recherche, indexeurs externes
- SSB : recherche locale seulement, pas d'index global
- AT Proto : services d'indexation consommant le firehose
- Radicle : pas de recherche integree
- BitTorrent : indexeurs externes (sites web, Bitmagnet)

**Pattern** : la recherche est un **service applicatif** qui
crawle/indexe les donnees du reseau. Le protocole fournit les
primitives de decouverte par hash/identite, pas par contenu.

### 6.3 La creation de contenu est une primitive generique

**Convergence forte** : les protocoles fournissent une primitive
d'ecriture generique. La semantique est applicative.

- IPFS : `add` (n'importe quel fichier → DAG)
- SSB : `publish` (n'importe quel JSON signe → log)
- AT Proto : `repo.createRecord` (n'importe quel record Lexicon)
- Radicle : `git push` (n'importe quelles refs Git)
- BitTorrent : creation de torrent (n'importe quel fichier)

**Pattern** : le protocole offre **write(bytes)**, l'application
decide **write(what)**.

### 6.4 Le pattern "sidecar" domine pour l'extensibilite

| Protocole | Outil | Pattern |
|---|---|---|
| IPFS | ipfs-cluster | Sidecar independant, API HTTP du daemon |
| SSB | plugins | Processus separes, communication muxrpc |
| AT Proto | Feed generators | Services HTTP independants |
| AT Proto | Labelers | Services independants, labels signes |
| Radicle | radicle-httpd | Sidecar read-only, acces direct stockage |
| Radicle | Radicle CI | Service ecoute evenements du noeud |
| BitTorrent | Indexeurs | Services independants crawlant la DHT |

**Pattern** : les extensions sont des **processus separes** qui
communiquent avec le noeud via une API bien definie (HTTP, RPC,
socket, ou acces direct stockage en read-only).

### 6.5 Extensibilite du schema de donnees

Trois approches se degagent :

1. **Schema libre** (SSB, IPFS) : n'importe quel JSON/bytes est
   accepte. Pas de validation au niveau protocole. Simple mais
   fragile — risque de fragmentation.

2. **Schema auto-descriptif** (AT Proto Lexicon) : schemas types
   avec namespace DNS, validation optionnelle, open unions pour
   la forward-compatibility. Le plus elegant mais le plus complexe
   a implementer.

3. **Schema implicite via convention** (Radicle COBs, BitTorrent
   BEPs) : le protocole transporte des structures dont la
   semantique est documentee dans des specs externes. Les clients
   qui ne connaissent pas un schema propagent les donnees sans
   les interpreter.

### 6.6 Le proxy preserve l'agence utilisateur

**AT Proto seulement, mais pattern puissant** : les clients ne
parlent jamais directement aux AppViews. Tout passe par le PDS
de l'utilisateur qui proxifie. Cela :
- Preserve la vie privee (l'AppView ne voit pas l'IP du client)
- Permet de changer d'AppView sans modifier le client
- Centralise l'authentification au PDS
- Donne a l'utilisateur le controle de ses donnees

### 6.7 Evenements comme contrat d'integration

| Protocole | Mecanisme d'evenements |
|---|---|
| AT Proto | Firehose websocket (subscribeRepos) |
| SSB | Log append-only comme source d'evenements |
| Radicle | Socket Unix read-only, flux d'evenements |
| IPFS | PubSub (experimental) |
| BitTorrent | Pas d'evenements natifs |

**Pattern** : les protocoles matures exposent un **flux
d'evenements** que les outils applicatifs consomment. C'est
l'interface d'integration la plus propre entre noeud et outils.

---

## 7. Recommandations pour SBFB

### 7.1 Etat actuel de SBFB

Le daemon SBFB (`nexus-shell-daemon`) fait aujourd'hui :
- Gossip protocol (iroh-gossip) : annonces et replication
- Distribution d'archives web (iroh-blobs)
- Feed public (append-only, raw-op extensible, serde_json::Value)
- Provenance Ed25519 + SLSA L1
- Coordinator : DB + dispatch + validation + kudos + invite +
  quarantine + capability
- HTTP API loopback pour le shell React

Les outils envisages (Factory, RRV) sont des fonctionnalites
*applicatives* riches qui ne devraient PAS etre dans le daemon.

### 7.2 Modele recommande : "Daemon Neutre + Clients Specialises"

Inspire principalement de **AT Proto** (separation PDS/AppView)
et **Radicle** (noeud aveugle + CLI semantique + sidecar httpd),
avec des elements de **IPFS Cluster** (sidecar independant) et
**BitTorrent** (extensibilite par convention).

#### A. Le daemon SBFB reste un "tuyau stupide"

Le daemon ne devrait connaitre QUE :
- **Blobs** : stocker et distribuer des archives zip (par hash)
- **Feed** : log append-only extensible (raw-ops)
- **Gossip** : propagation et decouverte de pairs/contenu
- **Identite** : Ed25519, provenance, verification
- **Evenements** : flux d'evenements consommable (WebSocket
  ou named pipe) pour notifier les outils des changements
- **API generique** : CRUD sur les primitives ci-dessus

Le daemon NE devrait PAS connaitre :
- La semantique des apps (ce qu'est un "projet", une "review")
- Les algorithmes de recherche ou de classement
- Les workflows de creation d'apps (build, test, deploy)
- La moderation fine / curation avancee

#### B. Factory = sidecar/service specialise

Inspire du pattern IPFS Cluster + AT Proto Feed Generator :

```
┌─────────────────────────────────────────────────┐
│                  Shell React                     │
│  (Browse, Create, Search — client generique)     │
└───────────┬──────────────┬──────────────────────┘
            │              │
     ┌──────▼──────┐ ┌────▼──────────────┐
     │   Daemon    │ │    Factory         │
     │   SBFB      │ │    (sidecar)       │
     │             │ │                    │
     │ - blobs     │ │ - build pipeline   │
     │ - feed      │ │ - template engine  │
     │ - gossip    │ │ - preview server   │
     │ - identite  │ │ - deploy workflow  │
     │ - events WS │ │                    │
     └──────┬──────┘ └────┬───────────────┘
            │              │
            │    utilise l'API du daemon
            │    pour publish/feed/blobs
            └──────────────┘
```

- Factory est un **processus separe** (binaire Rust ou module
  du daemon avec frontiere claire)
- Communique avec le daemon via l'API HTTP loopback
- Consomme les evenements du daemon pour reagir aux changements
- Ecrit dans le feed via les primitives generiques du daemon
- Le daemon ne sait pas que Factory existe

#### C. RRV = indexeur/service applicatif

Inspire du pattern AT Proto AppView + BitTorrent indexeur :

```
┌─────────────────────────────────────────────────┐
│                  Shell React                     │
│      (Search UI consomme l'API RRV)              │
└───────────┬──────────────┬──────────────────────┘
            │              │
     ┌──────▼──────┐ ┌────▼──────────────┐
     │   Daemon    │ │    RRV             │
     │   SBFB      │ │    (indexeur)      │
     │             │ │                    │
     │ - blobs     │ │ - FTS5/Tantivy    │
     │ - feed      │ │ - scoring          │
     │ - gossip    │ │ - proof cards      │
     │ - events WS │ │ - search API       │
     └──────┬──────┘ └────┬───────────────┘
            │              │
            │    consomme le firehose/events
            │    du daemon pour indexer
            └──────────────┘
```

- RRV est un **service d'indexation** qui consomme les evenements
  du daemon (comme un AppView AT Proto consomme le firehose)
- Construit un index FTS5/Tantivy local
- Expose une API de recherche que le shell React consomme
- Le daemon ne sait pas que RRV existe
- Verification : RRV peut verifier les preuves cryptographiques
  en accedant aux blobs/provenance via l'API du daemon

#### D. Feed extensible = Lexicon light

Le feed raw-op de SBFB (serde_json::Value) est deja proche du
pattern SSB (JSON libre) et AT Proto (records types). Pour
capitaliser sur ce design :

1. **Les outils inventent de nouvelles operations sans modifier
   le daemon** — le daemon stocke et propage les ops inconnues
   (deja le cas avec `try_parse_op` + `validate unknown ops`).
2. **Convention de nommage** : nommer les ops avec un namespace
   (`factory.build_started`, `rrv.search_manifest_published`)
   pour eviter les collisions — inspire du namespace DNS de
   Lexicon AT Proto.
3. **Schema optionnel** : documenter les schemas des ops dans
   des specs (comme les BEPs BitTorrent), pas les enforcer au
   niveau daemon.

#### E. Evenements comme contrat d'integration

Le daemon SBFB devrait exposer un **flux d'evenements** (WebSocket
ou named pipe) que les outils consomment :
- `blob_added(hash, size)`
- `feed_entry_inserted(entry_id, op_type)`
- `peer_connected(node_id)`
- `provenance_verified(hash, result)`

C'est le contrat d'integration entre le daemon neutre et les
outils specialises. Inspire du firehose AT Proto et du socket
d'evenements Radicle.

### 7.3 Ce que chaque protocole apporte a SBFB

| Prior art | Pattern emprunte pour SBFB |
|---|---|
| AT Proto | Separation PDS/AppView/Feed Generator. Le daemon = PDS neutre, Factory = AppView, RRV = search AppView. Proxy optionnel pour l'agence utilisateur. Lexicon → namespace ops. |
| Radicle | Noeud aveugle aux semantiques (COBs = refs Git). Socket d'evenements read-only. Sidecar httpd. Le daemon replique ce qu'il ne comprend pas. |
| IPFS Cluster | Sidecar pattern : processus independant, API HTTP du daemon, reseau prive optionnel. Factory suit ce modele. |
| SSB | Plugins muxrpc = processus separes. Kappa architecture (log source de verite, vues derivees). Feed SBFB = meme modele. |
| BitTorrent | Extension protocol (LTEP) = negotiation au handshake. BEP process = gouvernance des extensions. Indexeurs = services applicatifs sur DHT brute. |

### 7.4 Anti-patterns a eviter (documentes dans le prior art)

1. **Patchwork trap** (SSB) : bundler le serveur dans le client
   en couplant fortement la DB. Resultat : client inmaintenable,
   migration de DB impossible. → SBFB doit garder le daemon et
   le shell React strictement separes.

2. **Monolithe Kubo** (IPFS) : integrer trop de couches dans le
   daemon (MFS, UnixFS, IPNS, Gateway). Resultat : daemon lourd,
   difficile a remplacer par morceaux. → SBFB ne devrait pas
   integrer Factory/RRV dans le daemon meme si c'est "plus simple
   a court terme".

3. **Index sans evenements** (BitTorrent) : pas de mecanisme
   natif d'evenements → les indexeurs doivent crawler activement,
   ce qui est lent et couteux. → SBFB doit exposer un flux
   d'evenements proactif.

4. **Schema trop libre** (SSB) : types de messages JSON
   totalement libres → fragmentation de l'ecosysteme, clients
   incompatibles. → SBFB devrait documenter les schemas d'ops
   (meme sans validation stricte au daemon).

### 7.5 Plan d'action concret

| Etape | Sprint | Description |
|---|---|---|
| 1 | S66-S67 | Stabiliser l'API events du daemon (WebSocket ou named pipe). C'est le prerequis pour que tout outil puisse s'integrer sans modifier le daemon. |
| 2 | S67 | Definir le contrat API du daemon : quelles primitives sont "protocolaires" (blobs, feed, gossip, identity, events) vs "applicatives" (sera dans les outils). |
| 3 | S67-S68 | Factory comme sidecar : processus Rust separe (ou module avec frontiere claire), communique uniquement via l'API du daemon. |
| 4 | S70 | RRV comme service d'indexation : consomme les events du daemon, construit un index FTS5, expose sa propre API. |
| 5 | Post-S72 | Documenter les schemas d'ops du feed dans une spec type BEP/Lexicon pour les developpeurs tiers. |

---

## 8. Synthese : Le spectre de la neutralite

```
    Plus de semantique dans le noeud
    ◄──────────────────────────────────────────────►
    Moins de semantique dans le noeud

    BitTorrent     Radicle     IPFS      AT Proto     SSB
    (pur tuyau)    (refs Git)  (DAG+     (records     (log+plugins
                                UnixFS)  Lexicon)      indexes)

    SBFB idealement ici :
                       ▲
                       │
              Entre Radicle et AT Proto
              - Noeud : blobs + feed + gossip + identity + events
              - Outils : Factory, RRV, shell, futurs clients
              - Schema : convention documentee, pas enforcement
```

SBFB devrait se positionner entre Radicle (noeud tres neutre mais
sans mecanisme d'evenements) et AT Proto (noeud neutre + Lexicon
elegant + firehose + AppViews). Le feed raw-op extensible est deja
un excellent debut — il faut maintenant :

1. **Firehose/events** : exposer un flux temps reel pour les outils
2. **Factory en sidecar** : ne PAS l'integrer au daemon
3. **RRV en service** : indexeur qui consomme le firehose
4. **Schemas documentes** : convention + specs, pas enforcement
5. **API daemon minimaliste** : resister a la tentation d'ajouter
   des endpoints "applicatifs" au daemon
