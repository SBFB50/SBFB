# p2panda pour SBFB - briques utiles au protocole public verifiable

**Date:** 2026-05-13  
**Statut:** recherche produit/protocole, non engagee en sprint  
**Audience:** product owner, protocole, architecture, sprint planning  
**Source externe analysee:** `p2panda/p2panda` main `7b27406` clone local du 2026-05-13  
**Decision courte:** p2panda est interessant pour SBFB uniquement si on le lit comme une boite a briques local-first. Pour le protocole public SBFB, les briques prioritaires sont les logs signes, la sync durable, les cursors et les flux publics auditables. Les briques de confidential discovery, group auth et encryption sont secondaires ou reservees aux espaces prives, car la vision SBFB exige que tout projet public soit open source, verifiable et auditable.

---

## 0. Pourquoi cette note existe

La question initiale etait: quelles briques de p2panda sont interessantes pour le protocole SBFB ?

Le cadrage produit a ete corrige explicitement:

> Tout projet public SBFB doit etre open source et verifiable.

Cette correction change le classement des briques. Une analyse centree sur la confidentialite ferait de `p2panda-discovery`, `p2panda-auth` et `p2panda-encryption` les briques les plus attirantes. Pour SBFB public, ce serait une mauvaise priorite. Le coeur du protocole public n'a pas besoin de cacher l'existence des projets publics. Il doit rendre leurs annonces, releases, preuves, curations, builds et indexes publics impossibles a falsifier discretement.

Cette note lie donc p2panda a la vision SBFB actuelle:

- [`docs/architecture/PUBLISH_MODEL.md`](../../docs/architecture/PUBLISH_MODEL.md)
- [`docs/architecture/SELF_HOSTED_BUILD.md`](../../docs/architecture/SELF_HOSTED_BUILD.md)
- [`docs/architecture/LAUNCHER.md`](../../docs/architecture/LAUNCHER.md)
- [`docs/release/ROADMAP_COMMITMENTS.md`](../../docs/release/ROADMAP_COMMITMENTS.md)
- [`crates/nexus-shell-daemon-core/src/publish.rs`](../../crates/nexus-shell-daemon-core/src/publish.rs)
- [`crates/nexus-shell-daemon-core/src/browse.rs`](../../crates/nexus-shell-daemon-core/src/browse.rs)
- [`crates/nexus-core-rs/src/task.rs`](../../crates/nexus-core-rs/src/task.rs)
- [`crates/nexus-coordinator-rs/src/validator.rs`](../../crates/nexus-coordinator-rs/src/validator.rs)
- [`crates/nexus-coordinator-rs/src/provenance.rs`](../../crates/nexus-coordinator-rs/src/provenance.rs)
- [`web/public/sbfb-bridge.js`](../../web/public/sbfb-bridge.js)
- [`.planning/research/chat_ia_reseau_recherche_reseau_rnd.md`](chat_ia_reseau_recherche_reseau_rnd.md)
- [`.planning/active/sprint61_audit_plan.md`](../active/sprint61_audit_plan.md)

---

## 1. These produit

### 1.1 Ce que SBFB doit proteger

Pour les projets publics, SBFB ne doit pas proteger le secret de l'existence du projet. Il doit proteger:

1. L'identite exacte de la release publiee.
2. Le lien entre source publique et artefact executable.
3. L'historique public des annonces et curations.
4. Les preuves de build, de quorum et de provenance.
5. La capacite des noeuds offline a rattraper l'historique.
6. La capacite d'un pair tiers a re-verifier ce qui est annonce.

La phrase canonique du modele de publication existe deja:

```text
repo_url + commit_sha + artifact_hash + provenance_hash
```

Cette chaine doit rester le centre de gravite. Toute brique externe qui rend cette chaine moins claire ou moins verifiable doit etre rejetee ou gardee hors chemin public.

### 1.2 Ce que p2panda peut apporter sans casser cette vision

p2panda est pertinent pour trois besoins publics:

1. **Public signed release log**
   - Un journal append-only public des annonces, releases, curations, preuves de build et etats `Stale Source`.

2. **Durable public sync**
   - Une facon de rattraper l'historique quand un noeud revient en ligne, au lieu de dependre seulement du gossip live.

3. **Cursor / replay / audit**
   - Un modele de lecture incremental qui permet a une app, un noeud, un indexeur ou un auditeur de dire: "j'ai traite jusqu'a cette position, donne-moi la suite".

p2panda est moins prioritaire pour:

- cacher les topics publics;
- chiffrer le protocole public;
- gerer les droits des releases publiques;
- transporter les blobs d'artefacts deja servis par `iroh-blobs`.

---

## 2. SBFB actuel: surfaces a respecter

### 2.1 Publication verifiee

Source: [`PUBLISH_MODEL.md`](../../docs/architecture/PUBLISH_MODEL.md)

SBFB distingue deja:

| Etat | Signification protocolaire |
|---|---|
| `Local Draft` | travail local mutable, non publie |
| `Unverified Build` | artefact immutable mais sans preuve source |
| `Verified Release` | commit public + provenance + artefact lie |
| `Stale Source` | artefact toujours present, source devenue non re-verifiable |

Implication produit: une brique p2panda ne doit jamais permettre a un `Local Draft` de se presenter comme public/open-source. Le protocole public doit publier des releases, pas des etats locaux.

### 2.2 Annonce projet actuelle

Source: [`publish.rs`](../../crates/nexus-shell-daemon-core/src/publish.rs)

`ProjectAnnouncement` transporte deja:

- `project_name`
- `category`
- `description`
- `apps`
- `archive_ticket`
- `repo_url`
- `provenance_hash`
- `is_open_source`

Le commentaire local est important: `is_open_source` est derive par le coordinator a publish time et ne doit pas etre user-settable. Les workers L2 acceptent seulement les projets qui portent ce flag.

Limite actuelle: le format est une annonce gossip JSON. Il peut etre valide, mais il n'est pas encore naturellement un log public synchronisable, avec cursors, replay, pruning, vues materialisees et audit historique.

### 2.3 Browse public actuel

Source: [`browse.rs`](../../crates/nexus-shell-daemon-core/src/browse.rs)

`BrowseEntry` est deja proche d'une ligne de registre public:

- `project_id`
- `project_name`
- `category`
- `description`
- `curator_pubkey`
- `curator_name`
- `source`
- `status`
- `archive_ticket`
- `archive_hash`
- `repo_url`
- `provenance_hash`
- `is_open_source`

C'est la surface produit a enrichir ou a materialiser a partir d'un flux durable. Si p2panda est utile, il devrait aider a reconstruire ce type de vue depuis un flux d'operations signees.

### 2.4 Taches, resultats et quorum

Sources:

- [`task.rs`](../../crates/nexus-core-rs/src/task.rs)
- [`SELF_HOSTED_BUILD.md`](../../docs/architecture/SELF_HOSTED_BUILD.md)
- [`validator.rs`](../../crates/nexus-coordinator-rs/src/validator.rs)

SBFB a deja une structure canonique forte:

- `Task` est signe avec bytes canoniques RFC 8785 + domain separation.
- `task_type: "build"` existe comme extension de wire format via metadata.
- `redundancy_factor` pilote le quorum.
- `ResultValidator` compare les SHA256 pour accepter ou rejeter un build.
- `AwaitingQuorum`, `QuorumRejected`, outlier detection sont deja modeles.

Implication: p2panda ne doit pas remplacer ce modele. Il peut aider a publier et synchroniser l'historique public des resultats, pas redefinir la signature canonique des tasks/results.

### 2.5 Provenance

Source: [`provenance.rs`](../../crates/nexus-coordinator-rs/src/provenance.rs)

La provenance SBFB prouve:

1. le repo;
2. le commit;
3. le hash d'artefact;
4. le `node_id` builder;
5. une signature Ed25519;
6. un hash BLAKE3 de record.

Implication: le flux public peut contenir une operation `ReleasePublished`, mais cette operation doit pointer vers la provenance SBFB existante. Elle ne doit pas devenir une deuxieme provenance concurrente.

### 2.6 Bridge et recherche active

Source: [`sbfb-bridge.js`](../../web/public/sbfb-bridge.js)

Le bridge expose deja des methodes utiles:

- `submitTask`
- `getBrowseList`
- `getNodeStatus`
- `getStorageVersion`
- `piiRedact`
- storage app

Pour la Recherche Reseau Verifiable, un futur `search_verify` ou `search_manifest` peut se brancher sur cette surface sans demander aux apps d'integrer un SDK complexe cote protocole.

---

## 3. p2panda: inventaire utile

### 3.1 `p2panda-core`

Nature: types fondamentaux, append-only logs signes, operations avec header/body, BLAKE3, Ed25519, CBOR, log IDs, sequence numbers, timestamps, payload hashes, extensions.

Interet SBFB:

- Tres fort pour modeler un flux public signe.
- Tres fort pour separer header prioritaire et body plus lourd.
- Fort pour construire des vues materialisees (`BrowseEntry`, index RRV, audit trail).
- Interessant pour accepter du multi-writer via "single-writer logs combines".

Ce qu'il ne doit pas remplacer:

- Les signatures canoniques de `Task`, `Claim`, `Result`.
- La provenance release SBFB.
- Le hash content-addressable d'artefact `iroh-blobs`.

Potentiel produit:

```text
PublicProtocolOperation {
  kind: ReleasePublished | ReleaseDeprecated | CuratorVouched |
        BuildQuorumReached | BuildQuorumRejected |
        SearchManifestPublished | SourceBecameStale
  body_hash: ...
  author_pubkey: ...
  seq_num: ...
  backlink: ...
  signature: ...
}
```

Le flux public devient alors un historique verifiable, pas seulement un etat courant.

### 3.2 `p2panda-sync`

Nature: traits et implementations pour sync local-first de logs append-only. Les managers orchestrent des sessions concurrentes. Les noeuds peuvent rattraper des messages manques puis passer en live-mode.

Interet SBFB:

- Tres fort pour corriger une faiblesse structurelle du gossip: le gossip est bon pour live propagation, moins bon comme source unique de verite historique.
- Tres fort pour la reprise apres offline.
- Tres fort pour les indexeurs et auditeurs qui doivent rejouer l'historique.

Usage cible:

```text
Noeud rejoint le reseau
  -> decouvre un public feed
  -> sync depuis cursor local
  -> reconstruit Browse/PublicRegistry
  -> passe en live mode
  -> persiste cursor
```

Ce serait complementaire au `gossip_outbox` actuel dans [`db.rs`](../../crates/nexus-coordinator-rs/src/db.rs). L'outbox donne une durabilite best-effort locale; un log syncable donnerait une durabilite reseau.

### 3.3 `p2panda-store`

Nature: traits et implementations SQLite pour operations, logs, topics, cursors, groups, address book.

Interet SBFB:

- Utile si et seulement si SBFB adopte `core + sync`.
- Interessant pour les cursors d'indexation et les vues materialisees.
- Interessant pour eviter de recreer trop vite un stockage specifique aux protocol feeds.

Risque:

- SBFB a deja `CoordinatorDb` et des migrations SQLite. Ajouter un second modele store peut creer de la dette si l'integration n'est pas tres bornee.

Decision recommandee:

```text
Ne pas adopter p2panda-store seul.
L'adopter seulement dans un crate experimental dedie si le spike core/sync passe.
```

### 3.4 `p2panda-net`

Nature: pile reseau local-first avec iroh endpoint, address book, mDNS, discovery, gossip, LogSync, supervisors optionnels.

Interet SBFB:

- Fort comme reference d'architecture: separation endpoint, address book, discovery, gossip, sync.
- Interessant parce qu'il utilise aussi iroh 0.98.x dans la version analysee.
- Utile pour comparer notre separation actuelle entre pkarr/discovery/gossip/blob serving.

Risque:

- SBFB a deja une pile iroh/gossip/blobs integree au daemon, au launcher et au shell.
- Adopter `p2panda-net` completement pourrait dupliquer une pile reseau et compliquer la politique de securite SBFB.

Decision recommandee:

```text
Ne pas adopter p2panda-net comme remplacement.
Evaluer des patterns: address book, non-blocking sync/live mode, cursors, supervisor.
```

### 3.5 `p2panda-discovery`

Nature: confidential topic discovery via PET/PSI, random walk, topics secrets.

Interet SBFB public:

- Faible a moyen pour le registre public, car les projets publics ne doivent pas etre caches.
- Moyen pour eviter de leak des interets de recherche d'un utilisateur.
- Moyen pour des curators prives, equipes, beta tests, apps privees, collectifs.

Decision produit:

```text
Pas coeur du protocole public.
Optionnel pour espaces prives ou requetes utilisateur sensibles.
```

Important: ne pas confondre "discovery confidentielle" et "open-source verifie". Une release publique verifiee doit etre visible et auditable.

### 3.6 `p2panda-auth`

Nature: group management decentralise avec permissions `Pull`, `Read`, `Write`, `Manage`, resolution de conflits, strong removal.

Interet SBFB public:

- Faible pour definir si une release publique est open source.
- Moyen pour gouvernance de collectifs de curators.
- Moyen pour des groupes qui publient ensemble.
- Moyen pour permissionner la replication d'un dataset prive.

Decision produit:

```text
Ne pas l'utiliser comme gate de verite publique.
L'evaluer plus tard pour teams, curators collectifs, moderation ou espaces prives.
```

### 3.7 `p2panda-encryption` et `p2panda-spaces`

Nature: encryption de groupe, data encryption, message encryption, PCS/FS, spaces multi-device.

Interet SBFB public:

- Faible pour projets publics.
- Fort pour apps privees ou equipes privees.
- A garder hors chemin critique tant que non audite et APIs non stables.

Risque note dans p2panda:

- APIs non stables avant v1.
- Pas d'audit de securite.
- Messages de controle de groupe non chiffres dans l'implementation actuelle.
- Pas de crypto post-quantum.

Decision produit:

```text
Experimental only.
Interdit comme dependance critique du registre public verifie.
```

### 3.8 `p2panda-blobs`

Nature annoncee: blob storage, retrieval and synchronisation.

Constat source:

```text
// TODO: Needs refactoring since p2panda-net refactor.
```

Decision:

```text
Ne pas prioriser.
SBFB garde iroh-blobs pour artefacts immutables.
```

---

## 4. Priorisation produit pour SBFB public

| Priorite | Brique | Valeur SBFB | Statut recommande |
|---|---|---|---|
| P0 | `p2panda-core` patterns | operations signees, append-only, replay, headers/bodies | Spike conceptuel |
| P0 | `p2panda-sync` patterns | rattrapage offline, public feed durable, live mode | Spike technique |
| P1 | `p2panda-store` cursors/logs | persistance cursor et replay si core/sync adoptes | Seulement avec P0 |
| P1 | `p2panda-net` architecture | reference pour separation discovery/gossip/sync/address book | Lecture/design, pas remplacement |
| P2 | `p2panda-auth` | curators collectifs, teams, moderation | Hors registre public |
| P2 | `p2panda-discovery` | espaces prives, recherche sensible, non-public topics | Optionnel, pas coeur public |
| P3 | `p2panda-encryption/spaces` | apps privees, groupes prives | Experimental only |
| Reject now | `p2panda-blobs` | redondant avec iroh-blobs, crate en refactor TODO | Ne pas integrer |

---

## 5. Vision cible: Public Verifiable Protocol Feed

### 5.1 Probleme actuel

SBFB a deja des annonces publiques et des preuves, mais elles sont reparties:

- gossip `ProjectAnnouncement`;
- browse aggregator;
- provenance record;
- task/result/quorum;
- docs architecture;
- DB locale;
- bridge frontend.

Ce modele fonctionne pour un MVP, mais il manque une surface produit canonique:

```text
Quel est le journal public verifiable du reseau ?
```

Sans journal public:

- un noeud qui revient offline depend de replay local ou d'autres endpoints;
- un moteur RRV doit combiner plusieurs sources;
- un auditeur doit reconstituer l'historique a partir d'etats courants;
- les curations et changements `Stale Source` peuvent etre moins naturels a rejouer.

### 5.2 Proposition

Creer un flux public SBFB append-only:

```text
SBFB Public Feed v1
  - operation signee
  - payload type
  - payload hash
  - author pubkey
  - previous/backlink
  - timestamp
  - optional body
  - references vers artefacts existants
```

Le flux ne remplace pas les preuves existantes. Il les reference.

### 5.3 Types d'operations

| Operation | Reference actuelle | Pourquoi |
|---|---|---|
| `ReleasePublished` | `ProjectAnnouncement`, `ProvenanceRecord`, `BrowseEntry` | publier une release verifiee |
| `ReleaseSuperseded` | futur lifecycle | annoncer qu'une release remplace une autre |
| `SourceBecameStale` | `PUBLISH_MODEL.md` | source non reverifiable |
| `SourceRecovered` | `PUBLISH_MODEL.md` | source a nouveau clonable |
| `CuratorVouched` | curator list + browse | curator recommande une release |
| `CuratorRevoked` | curator list + browse | curator retire son vouch |
| `BuildQuorumSubmitted` | `Task`, `ResultEntry` | un worker publie un resultat |
| `BuildQuorumReached` | `ResultValidator` | hash majoritaire accepte |
| `BuildQuorumRejected` | `ResultValidator` | divergence detectee |
| `SearchManifestPublished` | RRV doc | index public d'un projet/release |
| `CapabilityDeclared` | bridge/app manifest futur | app expose une capacite |
| `SecurityAdvisoryPublished` | hardening/security futur | avertir sur une release |

### 5.4 Exemple de payload public

```json
{
  "schema_version": 1,
  "type": "ReleasePublished",
  "project_id": "node_pubkey_hex",
  "project_name": "traducteur-offline",
  "release_id": "repo_url#commit_sha#artifact_hash",
  "repo_url": "https://github.com/org/app",
  "commit_sha": "40_hex_chars",
  "artifact_hash": "blake3_or_iroh_blob_hash",
  "archive_ticket": "iroh_blob_ticket",
  "provenance_hash": "blake3_hex",
  "is_open_source": true,
  "published_at": "2026-05-13T00:00:00Z"
}
```

Regle: `is_open_source: true` ne suffit jamais seul. Il est valide seulement si la chaine `repo_url + commit_sha + artifact_hash + provenance_hash` est presente et verifiable.

---

## 6. Lien avec Recherche Reseau Verifiable

Source: [RRV note](chat_ia_reseau_recherche_reseau_rnd.md)

La RRV veut chercher des objets reseau executables et verifiables:

- apps;
- code;
- artefacts;
- provenance;
- capabilities;
- tasks/results;
- workers;
- forks;
- curations;
- quorum outputs.

p2panda est interessant pour la RRV non pas parce qu'il cache, mais parce qu'il donne un modele de flux signable et replayable.

### 6.1 `SearchManifestPublished`

Un `SearchManifest` public peut contenir:

- `project_id`;
- `release_id`;
- `commit_sha`;
- `artifact_hash`;
- `provenance_hash`;
- `index_hash`;
- `index_blob`;
- `schema_version`;
- `languages`;
- `capabilities`;
- `license`;
- `symbols_hash`;
- `embedding_model`;
- `generated_by`;
- `generated_at`;
- `signature`.

La recherche ne renvoie pas seulement un resultat. Elle renvoie:

```text
resultat + release_id + preuves + chemin de verification
```

### 6.2 Pourquoi un log public aide la recherche

Sans log public, la RRV doit interroger l'etat courant du reseau.

Avec log public:

1. Elle rejoue les releases.
2. Elle rejoue les curations.
3. Elle rejoue les manifests.
4. Elle filtre les releases non verifiees.
5. Elle detecte les sources stale.
6. Elle peut prouver "pourquoi ce resultat apparait".

Exemple produit:

```text
Question:
  Trouve une app OCR offline open source verifiee avec build quorum.

Reponse:
  1 resultat.
  Release: org/ocr-app@abc123
  Artefact: iroh hash ...
  Provenance: validee par node ...
  Build quorum: 2/3 SHA256 identiques
  Curators: 3 vouches publics
  SearchManifest: index signe ...
```

---

## 7. Integration possible avec fichiers actuels

### 7.1 `publish.rs`

Evolution minimale:

- conserver `ProjectAnnouncement`;
- ajouter un type interne `PublicFeedOperation::ReleasePublished`;
- generer l'operation au moment ou `ProjectAnnouncement` est construit;
- publier en gossip live comme aujourd'hui;
- stocker dans un feed append-only pour sync/replay.

### 7.2 `browse.rs`

Evolution minimale:

- `BrowseAggregator` continue de produire `BrowseEntry`;
- nouvelle source: materialisation depuis Public Feed;
- les entries direct/gossip restent compatibles;
- le statut reachability reste un probe local, pas une verite globale.

### 7.3 `provenance.rs`

Evolution minimale:

- ne pas changer le record de provenance;
- ajouter seulement un pointeur depuis `ReleasePublished`;
- garder la verification Ed25519 existante.

### 7.4 `task.rs` et `validator.rs`

Evolution minimale:

- garder le wire format `Task`/`ResultEntry`;
- publier des operations publiques derivees quand le quorum change d'etat;
- ne pas signer deux fois la meme verite avec deux semantics incompatibles.

Operations derivees:

```text
BuildResultObserved
BuildQuorumReached
BuildQuorumRejected
BuildOutlierDetected
```

### 7.5 `db.rs`

Evolution minimale:

- ne pas remplacer les tables actuelles;
- ajouter un store experimental pour feed operations;
- garder `gossip_outbox` comme mecanisme de recovery local jusqu'a ce qu'un vrai sync feed existe.

### 7.6 `sbfb-bridge.js`

Evolution produit:

- exposer plus tard `getPublicFeedCursor`;
- exposer `getSearchManifest`;
- exposer `verifyRelease`;
- exposer `searchVerified`.

Ne pas exposer une API qui permet a une app d'affirmer elle-meme `is_open_source`.

---

## 8. Architecture de spike recommandee

### 8.1 Objectif du spike

Verifier si les patterns p2panda peuvent devenir une brique SBFB sans importer tout le framework.

Question testable:

> Peut-on representer les annonces publiques SBFB comme un flux append-only signe, syncable apres offline, et materialiser `BrowseEntry` sans perdre les invariants `Verified Release` ?

### 8.2 Hors scope du spike

- Pas de remplacement de `iroh-blobs`.
- Pas de remplacement de `Task`/`ResultEntry`.
- Pas de remplacement du daemon iroh.
- Pas de confidential discovery pour projets publics.
- Pas de group encryption.
- Pas de bump wire public sans decision sprint.

### 8.3 Prototype possible

Nouveau crate experimental:

```text
crates/nexus-public-feed/
```

Responsabilites:

- types `PublicFeedOperation`;
- validation de payload;
- materialisation `PublicRegistryView`;
- cursor local;
- import depuis `ProjectAnnouncement`;
- export JSON pour UI/CLI;
- tests sans reseau.

Phase 2 seulement:

- sync entre deux noeuds;
- comparaison avec p2panda-core/sync;
- integration daemon optionnelle derriere feature flag.

### 8.4 Criteres d'acceptation

Le spike est interessant si:

1. une release verifiee devient une operation publique signeable;
2. un noeud neuf peut reconstruire un `BrowseEntry` depuis le feed;
3. une source stale peut etre representee sans muter l'ancienne release;
4. un event quorum peut etre lie a un `Task`/`ResultEntry` existant;
5. un cursor permet de reprendre la lecture;
6. les tests prouvent que `Local Draft` ne peut pas apparaitre comme `Verified Release`;
7. aucune signature canonique SBFB existante n'est affaiblie.

Le spike est rejete si:

1. il force une double pile iroh trop lourde;
2. il rend floue la provenance;
3. il introduit un deuxieme statut open source;
4. il impose une dependance non stable sur le chemin release public;
5. il melange projets publics et espaces prives.

---

## 9. Risques et garde-fous

### 9.1 Risque: sur-privatiser le protocole

Symptome: on met `p2panda-discovery` au centre et les projets publics deviennent des topics secrets.

Correction:

```text
Le registre public SBFB est public by design.
La confidentialite est une option d'espace prive, pas la verite publique.
```

### 9.2 Risque: double provenance

Symptome: une operation p2panda devient "la preuve" a la place de `ProvenanceRecord`.

Correction:

```text
Le feed reference la provenance SBFB. Il ne la remplace pas.
```

### 9.3 Risque: confusion entre gossip live et historique

Symptome: on continue de traiter le gossip comme source unique.

Correction:

```text
Gossip = propagation live.
Public feed = historique replayable.
Browse = vue materialisee.
```

### 9.4 Risque: importer trop gros

Symptome: adoption de `p2panda-net` complet, double endpoint, double address book, double policies.

Correction:

```text
Commencer par patterns et types.
Importer une crate seulement si le spike prouve un gain net.
```

### 9.5 Risque: dependance non auditee en chemin critique

Symptome: encryption/spaces deviennent obligatoires pour publier une release publique.

Correction:

```text
Les briques non stables ou non auditees restent experimental only.
```

---

## 10. Roadmap produit proposee

### Phase R0 - Spec publique

Livrable:

- cette note;
- schema `PublicFeedOperation`;
- mapping vers `ProjectAnnouncement`, `BrowseEntry`, `ProvenanceRecord`, `Task`, `ResultEntry`;
- decision explicite: public feed ne remplace pas provenance.

### Phase R1 - Local materialization

Livrable:

- crate ou module experimental;
- operations en memoire;
- materialisation `PublicRegistryView`;
- tests statut `Verified Release` / `Stale Source`.

### Phase R2 - Cursor/replay

Livrable:

- cursor local;
- replay depuis zero;
- reprise apres interruption;
- export audit JSON.

### Phase R3 - Sync P2P experiment

Livrable:

- deux ou trois noeuds;
- sync public feed;
- offline catch-up;
- comparaison directe avec gossip-only.

### Phase R4 - RRV integration

Livrable:

- `SearchManifestPublished`;
- recherche verified-only;
- preuve retournee avec chaque resultat.

### Phase R5 - Product UI

Livrable:

- Browse montre "pourquoi ce projet est verifie";
- vue audit trail d'une release;
- filtres `Verified Release`, `Stale Source`, `BuildQuorum`.

---

## 11. Decision recommandee

### A faire

1. Utiliser p2panda comme reference pour:
   - append-only signed logs;
   - sync durable;
   - cursors;
   - separation gossip live vs sync historique.

2. Prototyper un `Public Verifiable Feed` SBFB:
   - public;
   - signe;
   - replayable;
   - derive des preuves SBFB existantes;
   - compatible avec Browse et RRV.

3. Garder le modele open source verifie actuel:
   - `repo_url`;
   - `commit_sha`;
   - `artifact_hash`;
   - `provenance_hash`;
   - `is_open_source` derive server-side.

### A ne pas faire maintenant

1. Ne pas mettre confidential discovery au centre du registre public.
2. Ne pas remplacer `iroh-blobs` par `p2panda-blobs`.
3. Ne pas remplacer les signatures `Task`/`ResultEntry`.
4. Ne pas utiliser group encryption sur le chemin public.
5. Ne pas creer un deuxieme statut open-source concurrent.

### Formulation produit finale

```text
SBFB public ne cache pas les projets.
SBFB public rend chaque projet prouvable, synchronisable, auditable et executable.

p2panda est utile si ses patterns aident a construire le journal public
verifiable du reseau. Il est dangereux si on le lit comme une invitation
a rendre le registre public prive ou opaque.
```

---

## 12. Placement sprint

Cette note ne doit pas interrompre Sprint 61 Phase 0. Le sprint courant reste l'audit S60 selon:

- [`.planning/active/sprint61_audit_plan.md`](../active/sprint61_audit_plan.md)

Placement recommande:

| Moment | Action |
|---|---|
| S61 Phase 0 | Lire comme contexte, pas comme blocker |
| S61 post-audit | Decider si un spike R0/R1 merite un sprint dedie |
| Sprint dedie | Spec `PublicFeedOperation` + tests de materialisation locale |
| Post-v1.0 public adoption | Sync P2P et integration RRV |

---

## 13. Open questions

1. Le public feed doit-il etre un topic global unique ou un feed par projet plus un index global ?
2. Le cursor doit-il etre par noeud, par app, par indexeur ou par utilisateur ?
3. Les operations publiques doivent-elles utiliser JSON canonique SBFB ou CBOR p2panda ?
4. Quel est le seuil minimal de preuve pour afficher une release dans Browse ?
5. Comment eviter les attaques spam sur le public feed ?
6. Les curators doivent-ils signer des operations publiques dediees ou continuer via curator lists materialisees ?
7. Le feed doit-il inclure les events de build quorum complets ou seulement les decisions finales ?
8. Comment representer un fork et un lineage sans recreer un graphe trop lourd ?
9. Quel subset RRV doit etre public par defaut ?
10. Quelles donnees de recherche utilisateur doivent rester locales meme si les projets publics sont publics ?

---

## 14. Resume executif

Le meilleur apport p2panda pour SBFB n'est pas la confidentialite. C'est le modele:

```text
operation signee + append-only log + sync durable + cursor + replay
```

Ce modele peut transformer les annonces SBFB en journal public verifiable:

```text
ProjectAnnouncement live
  -> PublicFeedOperation durable
  -> BrowseEntry materialisee
  -> SearchManifest verifiable
  -> audit trail rejouable
```

La priorite protocole est donc:

1. `p2panda-core` comme reference de log signe.
2. `p2panda-sync` comme reference de rattrapage offline.
3. `p2panda-store` seulement si les deux precedents passent le spike.
4. `p2panda-discovery/auth/encryption` seulement pour espaces prives, curators collectifs ou requetes sensibles.
5. `p2panda-blobs` non prioritaire.

La vision SBFB reste:

```text
Tout projet public doit etre open source, verifiable, audit trail public,
artefact immutable, source publique pinnee, provenance signee.
```

