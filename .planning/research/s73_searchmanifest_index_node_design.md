# SearchManifest — design de la forme correcte (noeud-index opt-in)

**Statut** : **DEFERRED** (decision D3, Sprint 73). Aucune ligne de code wire
n'est livree en S73. Ce document capture la **conception dure** du futur
SearchManifest pour qu'il soit pret a coder lorsqu'un signal empirique le
declenche (PO-13 « cabler les deux a terme » honore).

**Ecrit** : 2026-06-04 (Sprint 73 Phase D).
**Decision parente** : `sprint73_kickoff.md §4 D3` + arbitrage utilisateur
Checkpoint §11 (2026-06-03 : **feed-local-replique pour S73**).
**Recherche source** : `sprint73_kickoff.md §"OSS prior art — discovery/
recherche decentralisee"` (7 modeles, workflow G9 `wq01d17lj`).

---

## §0 TL;DR

S73 cable la recherche reseau **sur le feed-local-replique** : chaque noeud
indexe (FTS5) ce que le gossip lui a deja livre. En **pilote ferme** (zero
noeud tiers en prod), la couverture feed-local **≈ couverture reseau-large** :
tout le pilote partage le meme feed. Un SearchManifest « chaque noeud diffuse
son index a tous » n'apporterait **aucun gain de couverture aujourd'hui** tout
en ouvrant une surface Sybil/spam (ARES 2024) et un cout d'announce continu
(provider records IPFS expirent a 24h). C'est precisement la forme que les
systemes matures **abandonnent**.

Quand un signal empirique le justifiera (**federation partielle post-launch**
— des noeuds rejoignant le reseau sans le feed gossip complet), la forme
correcte n'est **pas** un broadcast : c'est un **noeud-index opt-in**, signe
Ed25519, anti-spam par signature curateur + reputation kudos, **par defaut
DESACTIVE**, ou les requetes utilisateur ne sont **jamais** envoyees au reseau.
Modele : relays Nostr (NIP-50), delegated routing IPFS, index signe F-Droid,
seed-nodes Radicle.

---

## §1 Probleme

La recherche full-text SBFB (FTS5 `search_index`, S67-S73) est **locale** :
chaque noeud cherche dans le feed qu'il a recu. La question ouverte de la
roadmap est : **un noeud peut-il chercher des projets qu'il n'a pas encore
recus via gossip ?**

Deux regimes :

- **Pilote ferme (etat actuel)** : tous les noeuds suivent le meme ensemble
  de curateurs et recoivent le meme feed via gossip. Le feed-local de chaque
  noeud **converge** vers le feed global. Recherche locale = recherche
  reseau. **Aucun gap.**
- **Federation partielle (futur post-launch)** : des noeuds rejoignent avec
  un interet **partiel** (ils ne seedent qu'un sous-ensemble de curateurs,
  ou rejoignent tard et n'ont pas rejoue tout l'historique). Leur feed-local
  est un **sous-ensemble strict** du feed global. Un projet annonce dans une
  partie du reseau qu'ils ne seedent pas est **invisible** a leur recherche.
  **C'est le seul regime ou un index reseau apporte une couverture
  supplementaire.**

Le SearchManifest n'est utile que dans le **second** regime, qui n'existe pas
encore. Le construire maintenant = construire ce qu'on devrait jeter.

---

## §2 La forme NAIVE (rejetee) : broadcast-everywhere

« Chaque noeud publie son index complet a tout le reseau (op feed
`SearchManifestPublished` diffusee en gossip a tous), et chaque noeud agrege
les manifestes de tous les autres. »

Pourquoi c'est **faux** :

| Defaut | Source | Consequence |
|---|---|---|
| **Surface Sybil mono-machine** | ARES 2024, « The Sybil Attack Strikes Again » (Mainline + IPFS DHT) | Un manifeste diffuse a tous = un seul attaquant peut empoisonner / censurer / noyer l'index du reseau depuis une machine, sans cout d'identite. |
| **Cout d'announce continu** | ipshipyard.com (DHT Provide Sweep) + blog.ipfs.tech (2025) | Les provider records IPFS **expirent a 24h** → re-announce permanent. Un index diffuse a tous a le meme cout operationnel (re-broadcast a chaque churn). |
| **Spam non borne** | — | Sans cout d'identite ni reputation, un noeud peut publier des manifestes pointant vers du contenu inexistant / malveillant. La curation par signature manque. |
| **Ce que les matures abandonnent** | Nostr NIP-50, IPFS delegated routing 2025-26 | Aucun systeme moderne ne fait « chaque noeud propage son index a tous ». Ils delegent a des **noeuds specialises**. |

Rejete : zero gain de couverture en pilote ferme + ces trois surfaces.

---

## §3 Les 7 modeles OSS etudies

| Modele | Mecanisme de recherche/decouverte | Anti-spam | Lecon pour SBFB |
|---|---|---|---|
| **F-Droid** (index-v2) | Index **signe par depot** (RSA/SHA-256), diffs RFC 7396. Pas de crawl global — on cherche dans les depots ajoutes. | Signature du mainteneur du depot. | Index **signe par l'emetteur**, curation par depot. Analogue feed-local. |
| **IPFS DHT** | Provider records (expirent 24h) + delegated routing (2025-26 : noeuds specialises). | PoW / rate-limit DHT (faible — ARES 2024). | Le routing migre vers des **noeuds-index**, pas chaque noeud. Cout announce reel. |
| **Nostr NIP-50** | Recherche **deleguee aux relays** specialises avec anti-spam dedie. | Par relay (politique locale + WoT). | Modele **noeud-index** (relay), pas broadcast. Les clients choisissent leurs relays. |
| **Radicle Heartwood** | Announce **scope par interet** (on recoit les annonces des depots qu'on seede) + seed-nodes pour le scaling. | Scope par interet + seed-node trust. | Feed-local enrichi par interet ; seed-nodes = noeuds-index opt-in. |
| **SSB (Scuttlebutt)** | Replication par **proximite sociale** (follows). | Anti-spam par construction (graphe social). | Pur feed-local-replique ; couverture bornee par le graphe. |
| **pkarr / iroh discovery** | Ed25519 sur Mainline BEP44 + gossip pour la decouverte de **noeuds/projets** (deja en place SBFB). | Signature Ed25519. | S73 ne concerne QUE l'indexation full-text, pas la decouverte (deja resolue). |
| **ARES 2024 (etude Sybil)** | — (analyse d'attaque) | — | Un broadcast reseau-large ouvre la censure/DoS mono-machine. **Argument central du defer.** |

**Convergence** : les systemes matures utilisent des **noeuds-index
specialises signes** (relays, seed-nodes, index F-Droid), jamais un broadcast
« tous-vers-tous ». Le pilote ferme se contente du **feed-local-replique**.

---

## §4 La forme CORRECTE : noeud-index opt-in signe

### §4.1 Roles

- **Noeud normal (defaut)** : indexe son feed-local, cherche localement.
  **N'emet aucun manifeste. N'envoie aucune requete au reseau.** C'est l'etat
  S73 — inchange par cette conception.
- **Noeud-index (opt-in explicite)** : un operateur active le role
  `index-node` (flag de config, **default OFF**). Le noeud-index agrege les
  annonces des depots qu'il seede + accepte (sous conditions §4.3) des
  `SearchManifestPublished` d'autres noeuds-index, et **sert** une recherche
  federee a des clients qui le **choisissent explicitement** (modele relay
  Nostr / seed-node Radicle).

### §4.2 Forme wire (esquisse — NON codee en S73)

Op feed **`SearchManifestPublished`** via le chemin **raw-op** (pattern §P51) :
ajouter un variant a `PublicFeedOperation` n'est **PAS** un breaking change
(`PUBLIC_FEED_SPEC.md §9.1` : « Adding a new operation type is NOT a breaking
change » ; politique pre-launch raw-op `CLAUDE.md:355-357`). **`FEED_FORMAT_
VERSION` reste 1.** Le variant `SearchManifestPublished` est deja **nomme mais
non implemente** (`public_feed.rs:78-79`, spec §2.2) — la porte est ouverte,
aucun type mort a poser maintenant.

Payload esquisse (a figer au sprint d'implementation, pas ici) :

```
SearchManifestPublished {
  index_node_pubkey: String,   // Ed25519 hex du noeud-index emetteur
  manifest_root: String,       // BLAKE3 d'un index compact (ex. Bloom/digest)
                               //   des project_id couverts, PAS le full-text
  covered_curators: Vec<String>, // pubkeys curateurs dont ce noeud agrege le feed
  seq_high_water: u64,         // dernier feed seq couvert (fraicheur)
  endpoint_hint: Option<String>, // ticket iroh pour tirer l'index detaille
}
```

Invariant : le manifeste publie un **digest de couverture** (« je connais ces
projets jusqu'au seq N »), **pas** l'index full-text. Un client interesse
**tire** (pull) l'index detaille du noeud-index choisi — il n'est jamais
**pousse** a tous (evite l'amplification + le cout d'announce §2).

### §4.3 Anti-spam

- **Signature Ed25519** du `index_node_pubkey` sur la representation canonique
  (JCS RFC 8785, domaine `DOMAIN_SEARCH_MANIFEST_V1` — **nouveau domaine de
  separation**, comme `DOMAIN_FEED_V1`). Un manifeste non signe / mal signe
  est rejete au sync (miroir de la validation feed actuelle).
- **Reputation kudos** : un manifeste n'est agrege que si `index_node_pubkey`
  a une reputation kudos non-transferable au-dessus d'un seuil (les kudos sont
  un score de reputation per-project, non-monnaie — cf. memory
  `feedback_kudos_non_monetary`). Borne le Sybil : un attaquant doit
  **gagner** de la reputation, pas seulement generer des cles.
- **Curation par signature curateur** : un noeud-index ne couvre que des
  depots dont il seede la curator list (Ed25519). Pas de couverture
  « anonyme » d'un projet non-curate.

### §4.4 Invariant de confidentialite (NON-negociable)

- **Default OFF.** Un noeud n'emet ni ne consomme de manifeste sans opt-in
  explicite de l'operateur.
- **Les requetes utilisateur ne sont JAMAIS envoyees au reseau.** La recherche
  locale (FTS5) reste le chemin par defaut. Une recherche federee n'est
  declenchee que si l'utilisateur **choisit** un noeud-index (comme choisir un
  relay Nostr). Aucune fuite de terme de recherche par defaut.

### §4.5 Crypto

| Element | Choix |
|---|---|
| Signature | Ed25519 (coherent avec feed/curator/provenance SBFB) |
| Canonicalisation | JCS RFC 8785 (coherent `nexus-core-rs` canonical bytes) |
| Domaine de separation | `DOMAIN_SEARCH_MANIFEST_V1` (nouveau ; jamais reutiliser un domaine existant) |
| Hash | BLAKE3 (coherent feed `entry_hash`) |
| Wire version | raw-op, `FEED_FORMAT_VERSION` **inchange** (§4.2) |

---

## §5 Critere de declenchement

Implementer le SearchManifest **quand** (et seulement quand) un de ces signaux
empiriques apparait **post-launch** :

1. **Federation partielle observee** : des noeuds rejoignent le reseau avec un
   feed-local strictement partiel (interet limite, late join sans replay
   complet) — alors un projet annonce ailleurs leur est invisible.
2. **Demande mesuree** : des operateurs veulent volontairement servir un index
   federe (role seed-node / relay), et des clients veulent y souscrire.
3. **Echelle** : le corpus depasse ce qu'un noeud peut indexer localement
   (lie au gate Tantivy >50K docs post-S75 — orthogonal mais correle).

**Tant qu'aucun de ces signaux n'existe (pilote ferme), feed-local suffit.**

---

## §6 Pourquoi feed-local suffit pour S73

- Le feed est **deja** gossip-replique en DB locale (`feed_sync.rs`).
- La recherche est **deja** une FTS5 bm25 locale (`search.rs`).
- S73 indexe **a chaud** ce que le gossip a deja replique (reindex incremental
  D1) et enrichit le resultat avec le triplet provenance (D2).
- En pilote ferme, feed-local **converge** vers le feed global → couverture
  equivalente, **zero surface nouvelle**.

---

## §7 Compat / migration

- **Aucun bump wire.** `SearchManifestPublished` est un raw-op additif
  (`PUBLIC_FEED_SPEC.md §9.1`, pattern §P51). `FEED_FORMAT_VERSION = 1`
  inchange.
- **Note DESIGN-CONFLICT (resolue)** : un doc fossile
  (`s70_s72_rrv_research.md:984-986`) affirme « ajouter SearchManifestPublished
  = breaking change, bump v2 ». **Faux** sous la politique pre-launch raw-op
  actuelle. Le fossile s'auto-signale comme demoted (amendement 2026-05-22) et
  la spec vivante le contredit. **Confirme-superseded**, ne mord pas (S73
  defere tout code wire). A re-verifier au preflight wire du sprint
  d'implementation.
- **Tantivy reste gele** (`CLAUDE.md:306`, gate post-S75 >50K docs). FTS5 reste
  l'engine. Un doc fossile recommandant Tantivy ne se rouvre pas ici.

---

## §8 Questions ouvertes (pour le sprint d'implementation)

1. Granularite du `manifest_root` : Bloom filter des `project_id` vs digest
   Merkle vs simple liste signee ? (Trade-off taille / faux positifs / pull.)
2. Decouverte des noeuds-index : reutiliser pkarr/gossip (deja la) ou un
   registre signe dedie ?
3. Seuil de reputation kudos pour l'agregation (§4.3) — empirique, a calibrer
   sur des donnees post-launch.
4. UX cote shell : comment l'utilisateur **choisit** un noeud-index sans
   confusion avec la recherche locale par defaut (separation des intentions,
   cf. D4 barre Browse) ?

---

## §9 References

- F-Droid index-v2 + Security Model (2023-03 / 2024-2026)
- IPFS DHT Provide Sweep (ipshipyard.com) + delegated routing caching
  (blog.ipfs.tech 2025)
- ARES 2024, « The Sybil Attack Strikes Again » (dl.acm.org)
- Nostr NIP-50 (github.com/nostr-protocol/nips/50) + Purgatory (sept. 2024)
- Radicle Heartwood protocol guide (radicle.dev 2024-2025)
- Scuttlebutt gossip (handbook.scuttlebutt.nz 2024)
- pkarr / iroh discovery (github.com Pubky/pkarr 2024-2025)
- SBFB : `docs/protocol/PUBLIC_FEED_SPEC.md §9.1` (raw-op forward compat),
  `crates/nexus-coordinator-rs/src/public_feed.rs` (raw-op path §P51),
  `CLAUDE.md:354-366` (pre-launch raw-op policy),
  memory `feedback_kudos_non_monetary` (kudos = reputation non-transferable).
