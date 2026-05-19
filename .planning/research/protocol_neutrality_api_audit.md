# Audit de neutralite protocolaire — Surface API HTTP du daemon SBFB

Date : 2026-05-19
Branche : master (3360c45)
Sources lues : `crates/nexus-shell-daemon/src/http.rs` + tous les modules
handler (`deploy.rs`, `storage_api.rs`, `feed_sync.rs`, `contributor_api.rs`,
`apps.rs`, `consent.rs`, `files.rs`, `health_api.rs`, `shell_api.rs`,
`kudos_api.rs`, `tasks_api.rs`, `worker_state_api.rs`, `invite_api.rs`,
`quarantine_api.rs`, `diagnostic_api.rs`, `canary_api.rs`) +
`crates/nexus-coordinator-rs/src/` (DB, dispatcher, validator, feed, provenance,
kudos, canary, invite, quarantine, guardrails, fairness, forge, etc.)

---

## 1. Table complete des routes HTTP

### Legende classification

- **P** = Primitive protocolaire — neutre, utile a tout client du reseau
- **W** = Logique workflow/applicative — opinionne sur comment creer/gerer
- **D** = Diagnostic/operationnel — monitoring, pas de semantique reseau
- **S** = Securite/identite — transversal, pas specifique a un workflow

| # | Methode | Route | Classification | Justification |
|---|---------|-------|---------------|---------------|
| 1 | `GET` | `/health` | **P** | Sonde de vivacite universelle. Tout client/launcher en a besoin. |
| 2 | `GET` | `/auth/token` | **S** | Bootstrap loopback du bearer token. Mecanique d'auth transversale. |
| 3 | `GET` | `/blob-serve/{hash}/{*path}` | **P** | Distribution de contenu depuis archive zip via hash iroh-blobs. Coeur du protocole de rendu. |
| 4 | `GET` | `/api/daemon/info` | **P** | Snapshot d'etat du daemon (node_id, version, compteurs). Primitif pour tout client. |
| 5 | `GET` | `/api/daemon/curators` | **P** | Liste des curator lists cachees. Discovery curators = coeur protocole SBFB. |
| 6 | `POST` | `/api/daemon/curators/subscribe` | **P** | S'abonner a un curator. Souscription = primitif de participation au reseau. |
| 7 | `DELETE` | `/api/daemon/curators/{pubkey}` | **P** | Se desabonner d'un curator. |
| 8 | `GET` | `/api/daemon/browse` | **P** | Vue agregee des projets via curators. Browse = primitif de decouverte reseau. |
| 9 | `POST` | `/api/daemon/browse/pull` | **P** | Demande de replay gossip pour peuplement browse. Primitif de sync. |
| 10 | `POST` | `/api/daemon/publish` | **P** | Broadcast d'une ProjectAnnouncement via gossip. Primitif de publication reseau. |
| 11 | `POST` | `/api/daemon/publish-blob` | **P** | Upload d'un blob brut dans iroh-blobs, retour du hash. Primitif de stockage. |
| 12 | `GET` | `/api/daemon/default-curators` | **P** | Liste des curators par defaut (config). Helper de bootstrap. |
| 13 | `POST` | `/api/daemon/panic/wipe` | **S** | Destruction irreversible de l'identite. Securite/duress transversale. |
| 14 | `GET` | `/api/daemon/diagnostic/neighborhood` | **D** | Snapshot des peers connus. Diagnostic reseau pur. |
| 15 | `POST` | `/api/canary/frost/trusted-dealer` | **S** | FROST DKG generation de shares. Warrant canary. Infrastructure de confiance. |
| 16 | `POST` | `/api/canary/frost/round1` | **S** | FROST ceremony round 1. |
| 17 | `POST` | `/api/canary/frost/round2` | **S** | FROST ceremony round 2. |
| 18 | `POST` | `/api/canary/frost/aggregate` | **S** | FROST signature aggregation. |
| 19 | `POST` | `/api/v1/tasks/submit` | **W** | Soumission d'une tache de compute (LLM). Logique workflow coordinator: guardrails, dispatch, model selection. |
| 20 | `POST` | `/api/v1/results/submit` | **W** | Soumission d'un resultat de tache. Logique workflow coordinator: validation, quorum, kudos credit, guardrails output. |
| 21 | `GET` | `/api/v1/kudos/{project_id}` | **W** | Lecture kudos par projet. Kudos = systeme de reputation specifique au workflow de compute. |
| 22 | `GET` | `/api/v1/kudos/{project_id}/verify` | **W** | Verification integrite de la chaine kudos. |
| 23 | `POST` | `/api/canary/observed` | **S** | Enregistrement d'une observation canary. Infrastructure de confiance. |
| 24 | `GET` | `/api/canary/network-health` | **S** | Sante reseau du systeme canary (freshness des signataires). |
| 25 | `GET` | `/api/canary/freshness/{pubkey}` | **S** | Freshness d'un signataire canary specifique. |
| 26 | `POST` | `/api/canary/inject-rate` | **S/D** | Reglage du taux d'injection canary. Admin/diagnostic. |
| 27 | `GET` | `/api/canary/observed-divergence` | **S/D** | Liste des divergences observees dans le canary. |
| 28 | `GET` | `/api/v1/apps` | **P** | Liste des apps connues du reseau (browse aggregator). Vue structuree du browse. |
| 29 | `GET` | `/api/v1/apps/{project_id}` | **P** | Detail d'une app par project_id. |
| 30 | `GET` | `/app/{name}/state` | **P** | Lecture du storage key-value d'une app. Bridge postMessage storage primitif. |
| 31 | `GET` | `/app/{name}/state/{key}` | **P** | Lecture d'une cle specifique du storage app. |
| 32 | `POST` | `/app/{name}/state/{key}` | **P** | Ecriture d'une cle dans le storage app. |
| 33 | `DELETE` | `/app/{name}/state/{key}` | **P** | Suppression d'une cle du storage app. |
| 34 | `GET` | `/api/daemon/storage/ticket/{app}` | **P** | Ticket iroh-docs pour la replication du storage d'une app. Primitif de sync P2P. |
| 35 | `POST` | `/api/daemon/storage/join` | **P** | Rejoindre le namespace iroh-docs d'un storage app via ticket. Primitif de sync. |
| 36 | `GET` | `/api/daemon/storage/{app}/version` | **P** | Version courante du storage replique (compteur InsertRemote). Poll primitif. |
| 37 | `GET` | `/api/daemon/feed/ticket` | **P** | Ticket iroh-docs pour la replication du feed public. Primitif de sync P2P. |
| 38 | `POST` | `/api/daemon/feed/join` | **P** | Rejoindre le feed d'un noeud distant via ticket. |
| 39 | `GET` | `/api/daemon/feed/status` | **P** | Statut du feed local (count, last_seq, authors). |
| 40 | `POST` | `/api/daemon/feed/insert` | **P** | Insertion d'une operation dans le feed (interne, signe, chaine). Primitif d'ecriture feed. |
| 41 | `GET` | `/api/daemon/feed/cursor` | **P** | Position du curseur feed (last_seq, last_entry_hash). |
| 42 | `POST` | `/api/v1/deploy` | **P** | Deploy prive (upload zip brut). Primitif de publication d'archive. |
| 43 | `POST` | `/api/v1/deploy-from-repo` | **P** | Deploy verifie depuis un repo git (clone+verify+provenance). Primitif de publication verifiable. |
| 44 | `GET` | `/api/v1/project/{project_id}/provenance` | **P** | Lecture du record de provenance d'un projet. Verification publique. |
| 45 | `GET` | `/api/v1/consent` | **W** | Lecture de la config GPU consent (4 niveaux). Specifique au workflow worker. |
| 46 | `POST` | `/api/v1/consent/set` | **W** | Ecriture de la config GPU consent. |
| 47 | `POST` | `/api/v1/consent/whitelist/add` | **W** | Ajout d'un project_id a la whitelist consent. |
| 48 | `POST` | `/api/v1/consent/whitelist/remove` | **W** | Retrait d'un project_id de la whitelist consent. |
| 49 | `POST` | `/api/v1/files/upload` | **P** | Upload CAS fichier (SHA-256 keyed). Primitif de stockage generique. |
| 50 | `GET` | `/api/v1/files/{sha256}/manifest` | **P** | Lecture du manifest d'un fichier uploade. |
| 51 | `GET` | `/api/v1/files/{sha256}` | **P** | Streaming d'un fichier par hash SHA-256. |
| 52 | `GET` | `/api/v1/coordinator/health` | **D** | Health enrichi du coordinator (uptime, version). |
| 53 | `GET` | `/api/v1/shell/discover` | **P** | Decouverte des coordinators visibles. Primitif de bootstrap reseau. |
| 54 | `GET` | `/api/v1/kudos/entries` | **W** | Liste paginee de toutes les entrees kudos. |
| 55 | `GET` | `/api/v1/kudos/{project_id}/leaderboard` | **W** | Leaderboard des contributeurs d'un projet. |
| 56 | `GET` | `/api/v1/diagnostic/fairness` | **D/W** | Metriques Gini/top-5% du systeme kudos. Diagnostic du workflow compute. |
| 57 | `GET` | `/api/v1/tasks` | **W** | Liste paginee des taches (pending/dispatched/completed). |
| 58 | `GET` | `/api/v1/tasks/{task_id}` | **W** | Detail d'une tache par ID. |
| 59 | `GET` | `/api/v1/worker/state` | **W** | Etat du worker local (state.json). Specifique au workflow compute. |
| 60 | `POST` | `/api/v1/invite/create` | **W** | Creation d'une invitation (worker/observer). Specifique au workflow compute+pilote. |
| 61 | `GET` | `/api/v1/invite` | **W** | Liste des invitations emises. |
| 62 | `DELETE` | `/api/v1/invite/{invite_id}` | **W** | Revocation d'une invitation. |
| 63 | `GET` | `/api/v1/quarantine` | **W** | Liste de la queue de quarantaine gossip. Specifique au pipeline de moderation. |
| 64 | `POST` | `/api/v1/quarantine/{row_id}/flush` | **W** | Acceptation d'un message en quarantaine. |
| 65 | `POST` | `/api/v1/quarantine/{row_id}/drop` | **W** | Rejet d'un message en quarantaine. |
| 66 | `GET` | `/api/v1/contributor/verify/{project_id}/{node_id_hex}` | **P** | Verification d'attestation de contributeur. Primitif de provenance. |
| 67 | `GET` | `/api/v1/contributor/project/{project_id}` | **P** | Liste des contributeurs attestes d'un projet. |
| 68 | `GET` | `/api/v1/contributor/envelope/{project_id}/{node_id_hex}` | **P** | Enveloppe d'attestation brute. |

---

## 2. Synthese de la classification

### Primitives protocolaires neutres (P) — 40 routes

Tout ce qui est distribution de blobs, decouverte reseau, publication
gossip, feed append-only, storage P2P, deploy verifie, provenance,
attestation de contributeurs, blob-serve, apps listing. Ces routes
constituent le "protocol layer" que tout client externe (Factory, RRV,
CLI, client mobile, Electron) doit pouvoir utiliser.

### Logique workflow/applicative (W) — 19 routes

| Groupe | Routes | Raison |
|--------|--------|--------|
| **Task dispatch + results** | #19, #20, #57, #58 | Opinionne sur le cycle de vie d'une tache de compute (LLM). Presuppose un modele specifique (submit→dispatch→result→validate→credit). |
| **Kudos/reputation** | #21, #22, #54, #55 | Systeme de reputation lie au workflow de compute. Les kudos n'ont de sens que dans le contexte task→result→credit. |
| **GPU consent** | #45-48 | Gestion du consentement GPU worker-side. Specifique au workflow compute, pas au protocole de distribution d'apps. |
| **Worker state** | #59 | Lecture d'un fichier state.json local. Specifique au binaire worker. |
| **Invites** | #60-62 | Invitation worker/observer avec tasks doc ticket. Lie au modele pilote ferme. |
| **Quarantine** | #63-65 | Pipeline de moderation des messages gossip. Operationnel, pas primitif protocole. |
| **Fairness diagnostic** | #56 | Gini sur les kudos = diagnostic du systeme de reputation. |

### Securite/identite (S) — 7 routes

FROST DKG (#15-18), panic wipe (#13), canary (#23-27), auth token (#2).
Transversales — ni primitives ni workflow, infrastructure de confiance
du reseau.

### Diagnostic (D) — 3 routes

Neighborhood (#14), coordinator health (#52), fairness (#56 partage W/D).

---

## 3. Primitives manquantes pour Factory et RRV

### 3.1 Ce dont Factory a besoin (S67-S69)

Factory est defini comme un "module daemon/broker Rust" (decision P7)
qui expose `/api/v1/factory/*` routes. Voici ce qu'il doit consommer
du daemon existant et ce qui manque :

| Besoin Factory | Route existante | Manque |
|----------------|-----------------|--------|
| Generer un scaffold d'app (templates) | -- | **Template engine + `POST /api/v1/factory/create`** (prevu S67 Phase B, dans le daemon) |
| Deployer une app generee | `POST /api/v1/deploy-from-repo` | OK |
| Deployer une app privee (preview) | `POST /api/v1/deploy` | OK |
| Lire le manifest SBFB.json v2 | -- | **`GET /api/v1/project/{id}/manifest`** — extraire et servir le SBFB.json depuis l'archive zip (aujourd'hui il faut parser manuellement) |
| Inserer CuratorVouched dans le feed | `POST /api/daemon/feed/insert` | OK (raw-op extensible), mais besoin du type `CuratorVouched` dans `PublicFeedOperation` (prevu S67 Phase A) |
| Verifier une provenance existante | `GET /api/v1/project/{id}/provenance` | OK |
| Lister les deploys d'un projet (historique) | -- | **`GET /api/v1/project/{id}/deploys`** — historique des deploiements. Aujourd'hui, seul le dernier provenance record est accessible. La DB `provenance_records` existe mais pas de endpoint "list by project". |
| Diff source entre 2 versions | -- | **`GET /api/v1/project/{id}/diff?from=sha&to=sha`** (nice-to-have, pas bloquant S67-S68) |
| Lire les feed entries d'un projet | -- | **`GET /api/daemon/feed/entries?project_id=...&limit=...`** — le feed existe en DB mais il n'y a AUCUN endpoint de lecture paginee des entries. Seulement `feed/status` (compteurs) et `feed/cursor` (position). C'est le manque le plus critique. |
| Preview sandbox d'une app en cours de creation | `GET /blob-serve/{hash}/{*path}` | OK (apres store blob) |
| Audit trail des operations Factory | -- | **`GET /api/v1/factory/audit`** + ecriture JSONL (prevu S67 Phase D dans le daemon) |

### 3.2 Ce dont RRV a besoin (S70-S72)

| Besoin RRV | Route existante | Manque |
|------------|-----------------|--------|
| Recherche full-text locale | -- | **`GET /api/daemon/search?q=...&limit=...&offset=...`** — l'endpoint n'existe pas. FTS5 a creer (S70 Phase A). |
| Indexation des browse entries | `GET /api/daemon/browse` + `GET /api/v1/apps` | OK comme source de donnees, mais **pas de hook d'indexation incrementale**. |
| Lecture du feed pour citations | -- | **`GET /api/daemon/feed/entries`** (meme manque que Factory). |
| Lecture de la provenance pour Proof Cards | `GET /api/v1/project/{id}/provenance` | OK pour un projet. Manque **`GET /api/v1/provenance/list`** pour batch indexation. |
| Publication d'un SearchManifest signe | `POST /api/daemon/feed/insert` | OK (raw-op extensible pour `SearchManifestPublished`), mais le type n'existe pas encore dans `PublicFeedOperation`. |
| Verification d'un SearchManifest distant | -- | **`GET /api/v1/search/manifest/{node_id}`** — pas de endpoint pour lire un manifest distant (S72). |
| Metadata enrichie SBFB.json v2 | -- | Meme manque que Factory : **endpoint de lecture manifest**. |

### 3.3 Primitives manquantes transversales (ni Factory ni RRV specifiquement)

| Primitive manquante | Justification | Priorite |
|---------------------|---------------|----------|
| **`GET /api/daemon/feed/entries`** (paginee, filtrable par project_id, op_type, author) | Le feed existe en DB avec 7+ operations types, mais est totalement opaque via HTTP. Aucun client ne peut lire le contenu du feed. C'est le trou beant. | **P0 (bloquant S67)** |
| **`GET /api/v1/project/{id}/manifest`** (SBFB.json v2 depuis archive) | Un client qui veut inspecter les metadata d'une app deployee doit today fetcher le blob zip entier et extraire le fichier. Besoin d'un endpoint dedie. | **P1 (S67 Phase A)** |
| **`GET /api/v1/provenance/list`** (tous les records, paginee) | Aujourd'hui seul `get_provenance_by_project(id)` est expose. Pour un indexeur (RRV) qui doit scanner toutes les provenances au boot. | **P1 (S70 Phase B)** |
| **`GET /api/daemon/feed/entries/{entry_hash}`** (single entry by hash) | Verification granulaire d'une entry de feed. | **P2** |
| **Webhook/subscribe sur feed** (SSE ou long-poll) | Factory et RRV ont tous deux besoin de reagir aux nouvelles entries du feed en temps reel sans poll. Aujourd'hui seul `feed/status` + poll est possible. | **P2 (S68+)** |
| **`POST /api/daemon/feed/verify-entry`** | Verification d'une entry de feed (signature + hash chain + PoW) sans l'inserer. Utile pour un validateur externe. | **P3** |

---

## 4. Recommandations : ce qui reste dans le daemon vs ce qui est externalise

### 4.1 Rester dans le daemon (primitives protocolaires)

Les 40 routes P de la table ci-dessus restent dans le daemon. Elles
forment le "protocol substrate" sur lequel tout client se branche.
Ajouts recommandes :

1. **Feed read API** (`/api/daemon/feed/entries`) — OBLIGATOIRE avant S67.
   Le feed est un log append-only signe. Sa lecture est une primitive au
   meme titre que `GET /browse` ou `GET /curators`. Sans cette route, le
   feed est un one-way pipe qui ne sert qu'a l'interne.

2. **Manifest extraction** (`/api/v1/project/{id}/manifest`) — le daemon
   a deja le zip en cache blob-serve. Extraire un fichier nomme est une
   extension naturelle de blob-serve.

3. **Search index** (`/api/daemon/search`) — FTS5 sur coordinator.db
   est un index local. La decision FTS5-pas-Tantivy est deja prise. Le
   daemon est le seul a avoir la DB, donc l'endpoint est dans le daemon.

4. **Provenance list** — extension triviale de `db.rs`.

### 4.2 Externaliser (ne PAS mettre dans le daemon)

| Element | Pourquoi externaliser | Client de quoi |
|---------|----------------------|----------------|
| **Factory template engine** | Opinionne sur la structure d'un projet. Un template "static-storage" est un choix workflow, pas un primitif reseau. Le generateur devrait etre un crate/binaire separe qui appelle `POST /deploy` a la fin. | `/api/v1/deploy-from-repo`, `/api/daemon/feed/insert`, `/api/daemon/publish` |
| **Factory UI state** | L'etat de progression d'une creation d'app (quel step, quel template, quel nom) est du state applicatif. | Storage app standard (`/app/factory/state/{key}`) |
| **Factory audit trail** | Un JSONL d'audit Factory est un concern de l'outil Factory, pas du protocole. | Fichier local ou storage app. |
| **RRV Proof Cards rendering** | La facon de presenter une preuve est un concern UI. | `GET /provenance`, `GET /feed/entries` |
| **RRV SearchManifest generation** | La decision de quels champs indexer et comment composer un manifest signe est workflow RRV. Le daemon fournit la primitive feed insert. | `POST /api/daemon/feed/insert` (op: SearchManifestPublished) |
| **Babel domain packs** | Un "domain pack" (corpus de traduction) est un asset specifique a Babel, pas un concept reseau. | Blob storage standard. |

### 4.3 Cas limite : la decision P7 "Factory = module daemon/broker"

La roadmap v3 (P7, P13) definit Factory comme un "module daemon/broker
Rust, pas une app iframe". Cela implique des routes `/api/v1/factory/*`
DANS le daemon. C'est en tension avec la neutralite protocolaire.

**Recommandation :** Respecter P7 pour S67-S69 (MVP Factory dans le
daemon) mais avec une architecture "broker thin" :

- Le broker Factory dans le daemon est un orchestrateur mince qui
  enchaine des primitives existantes (`deploy-from-repo` + `feed/insert`
  + `publish` + template substitution).
- Les templates sont des fichiers sur disque, pas du code Rust hardcode.
- La logique metier Factory (quelles gates appliquer, quel template
  choisir, quel audit log ecrire) est dans un module separe
  (`crates/nexus-factory-core/`) et le daemon l'appelle, mais le
  protocole ne depend pas de Factory.
- Si post-S75 Factory devient un outil externe (CLI + UI standalone),
  les routes `/api/v1/factory/*` disparaissent et Factory devient un
  client des primitives P.

### 4.4 Routes W existantes : verdict

| Route W | Verdict | Rationale |
|---------|---------|-----------|
| Tasks submit/results (#19-20) | **Rester** (pour l'instant) | Le dispatch de taches est coeur au reseau de compute. Si SBFB evolue vers un reseau pur de distribution d'apps sans compute, ces routes migrent vers un module optionnel. |
| Kudos (#21-22, #54-55) | **Rester** | La reputation est liee au protocole de confiance (TRUST_TAXONOMY). |
| Consent (#45-48) | **Externaliser a terme** | Le consent GPU est specifique au worker. Si le worker devient un binaire independant avec sa propre API, ces routes migrent. |
| Invites (#60-62) | **Rester** | Les invites encodent un doc ticket iroh-docs. Le daemon est le seul a detenir la cle pour minter. |
| Quarantine (#63-65) | **Rester** | La quarantine est un mecanisme de defense du gossip. Intrinsequement lie au daemon qui recoit les messages. |
| Worker state (#59) | **Externaliser a terme** | Lecture d'un fichier d'un autre binaire. Un proxy fragile. |

---

## 5. Matrice de couverture Factory/RRV

Ce tableau resume si Factory et RRV peuvent etre implementes comme
clients 100% externes des primitives daemon actuelles.

| Operation | Primitives disponibles | Verdict |
|-----------|----------------------|---------|
| Creer une app depuis un template | Aucune primitive de template | Factory doit etre un outil qui genere des fichiers localement puis appelle `deploy-from-repo` |
| Deployer une app | `deploy`, `deploy-from-repo`, `publish`, `publish-blob` | OK |
| Lire le registre des apps | `browse`, `apps`, `apps/{id}` | OK |
| Lire le feed d'evenements | **MANQUE** (`feed/entries`) | **BLOQUANT** |
| Inserer un evenement dans le feed | `feed/insert` (avec guard interne) | OK mais guard `X-SBFB-Feed-Internal` limitant — a ouvrir ou proxyer |
| Verifier provenance | `project/{id}/provenance` | OK pour 1 projet, manque batch |
| Verifier contributeur | `contributor/verify`, `contributor/project` | OK |
| Recherche full-text | **MANQUE** (`search`) | **BLOQUANT pour RRV** |
| Synchroniser feed P2P | `feed/ticket`, `feed/join` | OK |
| Synchroniser storage P2P | `storage/ticket`, `storage/join` | OK |
| Preview sandbox | `blob-serve` | OK |
| Lire metadata app (SBFB.json) | **MANQUE** (`manifest`) | Contournable mais penible |

**Conclusion :** 2 primitives P0 manquent (feed read, search index).
Avec leur ajout, Factory et RRV peuvent etre des clients externes a
95%. Le 5% restant est le template engine Factory qui est un choix
de commodite (integrer au daemon pour UX `sbfb create`) mais pas une
necessite protocolaire.
