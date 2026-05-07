# Research — Apps pre-v1.0 : Protocol Explorer + Ideas Hub

**Date** : 2026-05-07.
**Statut** : ACCEPTE pre-v1.0 (decision utilisateur).
**Prerequis** : S55 LT-7 self-hosted build + bridge postMessage
fonctionnel + verified deploy operationnel.

---

## Contexte strategique

Deux apps "dogfooding" deployees sur le reseau SBFB avant le tag
v1.0. Elles servent triple objectif :
1. **Preuve vivante** que le protocole fonctionne (la doc tourne
   sur le reseau qu'elle documente)
2. **Onboarding** des premiers utilisateurs/contributeurs
3. **Validation E2E** du chemin complet verified deploy → iframe
   → bridge → storage

Le reseau sans apps n'est qu'une infrastructure invisible. Ces
deux apps rendent le reseau tangible et utile des le jour 1.

---

## App 1 — Protocol Explorer ("sbfb-explorer")

### Vision

Une documentation vivante et interactive du protocole SBFB,
deployee **sur** SBFB lui-meme. L'utilisateur ouvre l'app dans
son navigateur via le daemon local, et decouvre comment le reseau
fonctionne — tout en etant connecte au reseau en temps reel.

Le medium est le message : l'app prouve que SBFB marche en
existant.

### Fonctionnalites cibles

#### F1 — Explication du protocole (statique)
- Architecture du reseau : noeud → daemon → coordinateur → workers
- Cycle de vie d'une app : repo Git → verified deploy → archive
  zip → distribution iroh-blobs → iframe sandbox
- Cycle de vie d'une tache : submit → dispatch → execution →
  validation → kudos
- Modele de securite : loopback bearer + iframe sandbox + CSP +
  curator lists + Sybil resistance
- Philosophie : zero moderation centrale, allowlist per-worker,
  open source par construction
- Format : pages HTML interactives, diagrammes SVG animes,
  exemples de code inline

#### F2 — Navigation code source (lien Git)
- Chaque concept explique est lie au fichier source correspondant
  dans le repo Git SBFB (lien direct vers le repo forge)
- L'app est elle-meme deployee depuis source (verified deploy) →
  le lien "voir le code de cette app" pointe vers son propre repo
- Possibilite de voir les diffs entre versions du protocole
  (via tags Git)

#### F3 — Etat live du noeud (bridge postMessage)
- Affichage en temps reel via le bridge :
  - Pairs connectes (nombre, latence)
  - Apps disponibles sur le reseau (via browse)
  - Etat gossip (topics, messages recents)
  - Statut du daemon (uptime, version, edition Rust)
- Necessite potentiellement de nouvelles methodes bridge :
  - `node_status` : uptime, version, pairs, gossip stats
  - `browse_list` : apps disponibles (deja accessible via
    /api/daemon/browse mais pas via bridge)

#### F4 — Tutoriel interactif "Publie ta premiere app"
- Guide pas-a-pas dans l'app elle-meme :
  1. Creer un repo avec index.html
  2. Ajouter SBFB.json (Keyoxide Ed25519)
  3. Publier via le coordinateur
  4. Voir son app apparaitre sur le reseau
- L'utilisateur fait le tutoriel tout en voyant le resultat
  en direct sur son noeud

### Stack technique

- **Tech** : HTML/CSS/JS pur (pas de framework, minimaliste)
  ou Astro/11ty pour generation statique avec composants
  interactifs. Le choix depend du budget LOC — HTML pur est
  plus simple mais Astro permet une meilleure organisation.
- **Bridge** : `storage_get`/`storage_set` pour preferences
  utilisateur (theme, progression tutoriel). Nouvelles methodes
  `node_status` et `browse_list` pour F3.
- **Deploy** : verified deploy standard depuis repo Git.
  Archive zip avec index.html + assets SVG/CSS/JS.
- **Taille** : cible < 500KB zip (pas de framework lourd,
  pas d'images bitmap, SVG uniquement)
- **Offline** : fonctionne partiellement sans bridge (F1 + F2
  sont statiques). F3 degrade gracieusement si le daemon ne
  repond pas.

### Primitives SBFB utilisees

| Primitive | Usage |
|---|---|
| Verified deploy | L'app est deployee depuis son repo Git |
| iframe sandbox | L'app tourne dans l'iframe sandbox standard |
| postMessage bridge | `storage_get`/`storage_set` + nouvelles methodes |
| iroh-blobs | Distribution de l'archive zip sur le reseau |
| Browse | L'app est decouvrable via le browse network |

### Extensions bridge requises

1. **`node_status`** : retourne `{ uptime_s, version, peers_count,
   gossip_topics, daemon_edition }`. Lecture seule, pas de risque
   securite. Ajouter a la whitelist bridge.
2. **`browse_list`** : retourne `[{ name, hash, publisher, ... }]`.
   Deja disponible via HTTP GET /api/daemon/browse mais pas expose
   via bridge. Proxy simple.
3. **`network_stats`** (optionnel) : `{ total_apps, total_peers,
   gossip_messages_24h }`. Agrege depuis les donnees locales du
   daemon. Nice-to-have, pas bloquant v1.

### Risques et mitigations

| Risque | Impact | Mitigation |
|---|---|---|
| Bridge extensions retardent | Medium | F1+F2 fonctionnent sans bridge, F3 est bonus |
| Contenu devenu obsolete | Low | Genere depuis le code, pas ecrit a la main |
| Trop ambitieux pour 1 sprint | Medium | MVP = F1 seulement, F2-F4 iteratif |

### MVP vs Full

- **MVP (1 sprint)** : F1 explication protocole + F2 liens code.
  HTML statique, 0 bridge. Prouve que le verified deploy + iframe
  fonctionne.
- **Full (2 sprints)** : + F3 etat live + F4 tutoriel interactif.
  Necessite les extensions bridge.

---

## App 2 — Ideas Hub ("sbfb-ideas")

### Vision

Un espace communautaire decentralise ou n'importe qui peut proposer
une idee d'app, voter, discuter, et former des groupes de travail.
Les idees evoluent naturellement vers des projets concrets lies a
des repos Git. Le tout tourne sur SBFB — la communaute utilise le
reseau pour construire le reseau.

### Fonctionnalites cibles

#### F1 — Proposer une idee
- Formulaire simple : titre, description, tags (categorie, difficulte)
- Chaque idee est un document JSON stocke via `storage_set`
- L'auteur est identifie par sa cle Ed25519 (anonyme mais verifiable)
- Pas de moderation centrale — chacun voit tout, filtre par pertinence

#### F2 — Voter et reagir
- Vote simple (upvote) par cle Ed25519 — 1 vote par identite
- Sybil-resistant via le systeme de Kudos existant : le poids du
  vote est proportionne a la reputation du votant sur le reseau
- Tri par score (votes × poids Kudos), date, activite recente

#### F3 — Lier un repo Git
- Quand quelqu'un commence a coder l'idee, il lie son repo Git
  (GitHub/GitLab/Codeberg/Gitea — multi-forge comme le deploy)
- L'idee passe de "proposition" a "en cours" automatiquement
- Le repo est verifiable via le meme systeme Keyoxide que le deploy
- Plusieurs repos peuvent etre lies a la meme idee (forks, alternatives)

#### F4 — Groupes de travail
- N'importe qui peut creer un groupe autour d'une idee
- Membres identifes par cle Ed25519
- Canal de discussion asynchrone (messages stockes via storage)
- Pas de roles/permissions complexes — modele plat, tout le monde
  contribue. Si desaccord, fork du groupe (meme philosophie que
  le reseau)

#### F5 — Integration reseau
- Quand un projet lie atteint le stade "deployable", lien direct
  pour le deployer sur SBFB via verified deploy
- Les idees populaires apparaissent dans le Browse du reseau
  comme "projets en recherche de contributeurs"
- Notification via gossip quand une idee qu'on suit evolue

### Stack technique

- **Tech** : HTML/CSS/JS, potentiellement avec un micro-framework
  reactif (Preact ~3KB ou Alpine.js ~15KB) pour la reactivite UI.
  Pas de React complet (trop lourd pour une app SBFB).
- **Donnees** : documents JSON dans iroh-docs via bridge
  `storage_get`/`storage_set`. Schema :
  ```
  ideas/{id} → { title, description, author_key, created_at, tags, repos[] }
  votes/{idea_id}/{voter_key} → { weight, timestamp }
  groups/{id} → { name, idea_id, members[], created_at }
  messages/{group_id}/{timestamp} → { author_key, content }
  ```
- **Identite** : cle Ed25519 du noeud (pas de login/password).
  L'utilisateur est son noeud.
- **Taille** : cible < 300KB zip

### Primitives SBFB utilisees

| Primitive | Usage |
|---|---|
| Verified deploy | Deploy depuis repo Git |
| iframe sandbox | Execution sandboxee |
| postMessage bridge | `storage_get`/`storage_set` pour toutes les donnees |
| Kudos | Ponderation des votes par reputation |
| Curator lists | Decouverte des idees via le reseau gossip |
| Ed25519 identite | Authentification sans compte, anti-Sybil |

### Extensions bridge requises

1. **`storage_list`** : lister les cles avec un prefixe
   (`ideas/*`, `votes/idea_42/*`). Indispensable pour enumerer
   les idees et les votes. Actuellement `storage_get` ne supporte
   que la lecture par cle exacte.
2. **`storage_delete`** : supprimer une entree (retirer son vote,
   supprimer une idee qu'on a creee). Actuellement absent du bridge.
3. **`identity_pubkey`** : obtenir la cle publique Ed25519 du noeud
   local. Necessaire pour identifier l'auteur d'une idee/vote.
   Lecture seule, risque securite minimal.
4. **`kudos_score`** (optionnel) : obtenir le score Kudos d'une
   cle pour ponderer les votes. Read-only.

### Risques et mitigations

| Risque | Impact | Mitigation |
|---|---|---|
| Extensions bridge bloquantes | High | MVP sans votes ponderes, ajout iteratif |
| Spam sans moderation | Medium | Rate-limit bridge + Kudos minimum pour poster |
| Donnees perdues (iroh-docs) | Medium | Le reseau replique les docs entre pairs |
| UX trop spartiate | Low | Focus fonctionnel, UX polish post-v1.0 |
| Schema donnees evolue | Medium | Pre-launch policy : pas de compat, on redefini |

### MVP vs Full

- **MVP (1 sprint)** : F1 proposer + F2 voter (vote simple sans
  ponderation Kudos). storage_list requis. ~200 LOC JS.
- **V2 (1 sprint)** : + F3 lier repo + ponderation Kudos.
- **Full (1 sprint)** : + F4 groupes + F5 integration reseau.

---

## Impact sur la roadmap pre-v1.0

### Prerequis techniques (a livrer AVANT les apps)

1. **Bridge extensions** : `storage_list`, `storage_delete`,
   `identity_pubkey` (minimum pour Ideas Hub MVP). `node_status`,
   `browse_list` (pour Protocol Explorer F3). Ces extensions sont
   des ajouts a la whitelist bridge existante — pas de refonte.
   **Budget : 1 phase d'un sprint** (~300-400 LOC Rust daemon +
   JS SDK).

2. **LT-7 self-hosted build** : les apps doivent pouvoir etre
   deployees via verified deploy. LT-7 est deja prevu S55.

3. **Verified deploy operationnel** : le chemin clone → Keyoxide
   → zip → provenance est implemente depuis S14. A valider E2E
   avec une vraie app (Protocol Explorer = premier test).

### Sequencage propose

| Sprint | Contenu apps |
|---|---|
| S55 | LT-7 self-hosted build (prerequis) |
| S56 | Bridge extensions (storage_list, identity_pubkey, node_status) + dette pair |
| S57 | Protocol Explorer MVP (F1+F2) = premiere app sur SBFB + Ideas Hub MVP (F1+F2) |
| S58 | Protocol Explorer Full (F3+F4) + Ideas Hub V2 (F3 repos) + stabilisation |
| v1.0 | Tag — 2 apps live sur le reseau, preuve fonctionnelle |

### Alternative : Protocol Explorer S57, Ideas Hub post-v1.0

Si le budget sprints est serre, Protocol Explorer seul suffit
comme preuve pre-v1.0. Ideas Hub peut etre la premiere app
communautaire post-v1.0, construite par la communaute elle-meme
(ce qui serait encore plus puissant narrativement).

---

## Modele de cout

**Cout pour FlowUP** : zero supplementaire. Chaque app est un zip
statique distribue via iroh-blobs (P2P). Le VPS FlowUP sert de
bootstrap node mais ne porte pas le trafic des apps — les pairs
se servent entre eux. Le build initial consomme ~30s CPU sur le
coordinateur (clone + zip), puis c'est du P2P pur.

**Cout pour les contributeurs** : leur propre bande passante pour
seeder les blobs. Modele BitTorrent — proportionnel a la popularite
de l'app, reparti entre tous les seeders.

---

## Coherence avec les decisions Day 0

| Decision | Compatible |
|---|---|
| D1 Pivot P2P integral | oui — apps P2P natives |
| D5 Zero moderation centrale | oui — Ideas Hub sans moderation |
| D7 Kudos per-project non-monnaie | oui — votes ponderes Kudos |
| D10 Worker binaire single-file | N/A — apps frontend |
| D12 AGPL-3.0 | oui — apps open source par construction |
| Sprint 12 archive zip = format universel | oui — les 2 apps sont des zips |
| Sprint 13 postMessage bridge seul canal | oui — tout passe par le bridge |
| Sprint 14 verified deploy from source | oui — les 2 apps deployees from source |

Aucun conflit avec les decisions gelees.

---

## Questions ouvertes

1. **Nom definitif des apps** : "sbfb-explorer" et "sbfb-ideas"
   ou autre chose ? Le nom apparaitra dans le Browse reseau.
2. **Langue de l'UI** : francais-first (coherent avec scan-en-strings)
   ou anglais-first (audience internationale) ? Ou i18n des le
   depart ?
3. **Repo separe ou monorepo** : chaque app dans son propre repo
   Git (plus propre pour le verified deploy, chacun peut forker
   independamment) ou dans le monorepo SBFB sous examples/ ?
4. **Qui build les apps** : le coordinateur FlowUP les deploie en
   premier, mais n'importe qui peut redeploy son propre fork.
