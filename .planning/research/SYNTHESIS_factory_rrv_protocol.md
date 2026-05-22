# Synthese — Factory, RRV et Protocole Neutre

## 0. Meta

**Date de consolidation :** 2026-05-19
**Statut :** CANON — ce document remplace et consolide 14 fichiers de recherche.
**Auteur :** Synthese FlowUP + Claude (session de consolidation)

### Documents fusionnes

| # | Document | Date | Role dans la synthese |
|---|----------|------|----------------------|
| 1 | `sbfb_rrv_code_factory_vision_pitch.md` | 2026-05 | Vision fondamentale : boucle RRV-Factory-App-Brique |
| 2 | `sbfb_project_factory_rrv_oss_research.md` | 2026-05-17 | Architecture Factory+RRV @dev, templates, broker, securite, Babel, briques OSS |
| 3 | `rrv_scoped_search_compute_groups.md` | 2026-05-16 | Scopes @dev/@network/@web, croisement sources, compute groups, sequence produit |
| 4 | `s70_s72_rrv_research.md` | 2026-05-18 | Recherche technique RRV (FTS5 vs Tantivy, SearchManifest, Proof Cards, phases S70-S72) |
| 5 | `s65_s75_factory_babel_canary_research.md` | 2026-05-18 | Pivot Factory-first, Babel canari, sequence S65-S75, gates FG0-FG10 |
| 6 | `factory_deploy_constraint_research.md` | 2026-05-18 | Contrainte node_id dans deploy, 5 options, recommandation Option D |
| 7 | `factory_first_feasibility_audit.md` | 2026-05-18 | Audit faisabilite Factory en S67, 3230 LOC sur 3 sprints, CuratorVouched minimal |
| 8 | `factory_gates_audit.md` | 2026-05-18 | Audit 11 gates FG0-FG10, testabilite, code existant reutilisable, 1840 LOC effectif |
| 9 | `babel_canary_scope_validation.md` | 2026-05-18 | Validation scope Babel canari, bridge 11/11, publish path 3 bloqueurs, tests |
| 10 | `protocol_neutrality_api_audit.md` | 2026-05-19 | Audit 68 routes HTTP daemon (40P/19W/7S/3D), primitives manquantes |
| 11 | `protocol_neutrality_prior_art.md` | 2026-05-19 | Prior art IPFS/SSB/AT Proto/Radicle/BitTorrent, 7 patterns communs |
| 12 | `factory_as_client_gap_analysis.md` | 2026-05-19 | Gap analysis Factory comme client externe, 4 primitives manquantes |
| 13 | `rrv_protocol_boundary_analysis.md` | 2026-05-19 | Frontiere RRV : 3 couches (protocolaire, service daemon, applicatif) |
| 14 | `rrv_scope_ordering_analysis.md` | 2026-05-19 | Ordonnancement @protocole avant @dev avant @web |

### Roadmaps de contexte (non fusionnes mais referencees)

- `.planning/roadmap_v3_public_trust_factory_babel_rrv.md` — roadmap v3 canon
- `.planning/roadmap_v4_neutral_protocol_factory_rrv.md` — draft v4 avec pivots

**Amendement 2026-05-22 :** D18 ajoute `S70 = Process Portable Complete
+ Gate 1 dogfood`. Les sections anciennes qui parlent de S70 comme RRV Core,
Gouvernance full UI, Tantivy/FTS MVP ou `@dev` index doivent etre lues comme
des candidats S71+ sauf si le kickoff S70 les importe explicitement apres
avoir traite `process_portable_complete_s70.md`.

### Conventions

Ce document est auto-suffisant. Chaque fait majeur reference son document
source entre parentheses : `(doc#N §X)` ou N est le numero du tableau
ci-dessus et X la section. Le texte est en francais, le code et les
identifiants en anglais.

---

## 1. Vision produit

### 1.1 La boucle fondamentale

La vision de SBFB n'est pas de tout reinventer. C'est de creer une
infrastructure capable de trouver, verifier, assembler et redistribuer
les meilleures briques open source existantes sous forme de solutions
locales, partageables et resistantes a la censure. (doc#1)

La boucle long terme :

```
RRV cherche les meilleures briques existantes
  -> Factory les assemble en applications concretes
    -> SBFB les verifie, publie et distribue
      -> Les apps deviennent des briques reutilisables
        -> retour a RRV
```

En resume : **SBFB = le protocole** (verifie, stocke, partage,
distribue). **RRV = le moteur** (cherche les meilleures briques et leurs
preuves). **Factory = l'atelier** (transforme les briques en apps).
**Babel = le premier dogfood** (prouve que la boucle fonctionne).
(doc#1 §final)

### 1.2 Ce que SBFB est

- Une plateforme P2P universelle de compute et d'hebergement d'apps.
- Un protocole de distribution d'archives web verifiees.
- Un commun logiciel anti-capture sous licence AGPL-3.0 (OSI).
- Un reseau ou chaque app publique est deployee depuis un repo Git avec
  provenance auto-attestee SLSA L1. (CLAUDE.md)

### 1.3 Ce que SBFB n'est pas

- Un "ChatGPT pour coder" ou un simple IDE.
- Une plateforme d'apps classique (pas d'app store centralise).
- Un moteur de recherche web (pas de crawling, pas de scraping).
- Un systeme de "compute correct" sans verification par quorum.
- Un projet "open source" au sens indifferencie — le code SBFB est open
  source (AGPL-3.0 OSI), mais les apps du reseau sont a "source
  verifiable", un terme qui ne presuppose pas la licence OSI. (doc#5 §4)

### 1.4 Vocabulaire exact

| Terme | Usage | Ce qu'il ne signifie PAS |
|-------|-------|--------------------------|
| Source verifiable | App deployee depuis un repo public avec provenance | "Open source" au sens OSI |
| Provenance verifiable | triplet (repo_url, commit_sha, artifact_hash) signe Ed25519 | Build reproductible |
| Commun logiciel | Projet SBFB sous AGPL-3.0, modele OpenBSD solo-maintainer | Fondation, startup, monetisation |
| Provenance auto-attestee SLSA L1 | Le coordinator clone, build, signe | SLSA L2 (build platform, isolation builder) |
| Score de completude de preuve | Mesure combien de couches de verification existent | "Trust score" social |
| Source verifiable, provenance verifiable | Formulation autorisee pour les apps reseau | "Badge verifie", "open source verifie", "code reseau = code du repo" |

(doc#5 §4, roadmap v3 §P1-P5)

---

## 2. Architecture : protocole neutre

### 2.1 Le daemon = tuyau stupide

L'analyse de 5 protocoles P2P matures (IPFS, SSB, AT Protocol, Radicle,
BitTorrent) revele un pattern convergent : **le noeud/daemon ne connait
pas la semantique du contenu qu'il transporte**. (doc#11 §6.1)

| Protocole | Ce que le noeud "comprend" | Ce qu'il ignore |
|---|---|---|
| IPFS | CID, DAG, blocs | Type de fichier, application |
| SSB | Feed append-only, JSON signe | Type de message, schema |
| AT Proto | Records Lexicon-types dans repos | Semantique des records |
| Radicle | Refs Git, identites Ed25519 | Commits, patches, issues, COBs |
| BitTorrent | Infohash, pieces, pairs | Contenu des fichiers |
| **SBFB** | **Blobs, feed, gossip, identite** | **Factory, RRV, Babel** |

Le daemon SBFB manipule des conteneurs adresses par contenu ou par
identite, pas des objets semantiques. (doc#11 §6.1)

SBFB se positionne entre Radicle (noeud tres neutre mais sans mecanisme
d'evenements) et AT Proto (noeud neutre + Lexicon + firehose +
AppViews) :

```
    Plus de semantique dans le noeud
    <---------------------------------------------->
    Moins de semantique dans le noeud

    BitTorrent     Radicle     IPFS      AT Proto     SSB
    (pur tuyau)    (refs Git)  (DAG+     (records     (log+plugins
                                UnixFS)  Lexicon)      indexes)

    SBFB ici :
                       ^
                       |
              Entre Radicle et AT Proto
              - Noeud : blobs + feed + gossip + identity + events
              - Outils : Factory, RRV, shell, futurs clients
              - Schema : convention documentee, pas enforcement
```

(doc#11 §8)

### 2.2 Audit de la surface API existante

L'audit exhaustif des 68 routes HTTP du daemon (doc#10) revele la
repartition suivante :

| Classification | Nombre | Description |
|---|---|---|
| **P** (Primitive protocolaire) | 40 | Blobs, browse, curators, feed, storage, deploy, provenance, attestation |
| **W** (Workflow/applicatif) | 19 | Tasks, kudos, consent GPU, worker state, invites, quarantine |
| **S** (Securite/identite) | 7 | FROST DKG, panic wipe, canary, auth token |
| **D** (Diagnostic) | 3 | Neighborhood, coordinator health, fairness |

Les 40 routes P forment le "protocol substrate" que tout client externe
(Factory, RRV, CLI, client mobile, Electron) peut utiliser. Les 19
routes W sont des logiques workflow qui pourraient a terme migrer vers
des modules optionnels. (doc#10 §2)

### 2.3 Ce qui reste dans le daemon

Le daemon fournit des primitives generiques :

- **Blobs** : stocker et distribuer des archives zip (par hash iroh)
- **Feed** : log append-only extensible (raw-ops serde_json::Value)
- **Gossip** : propagation et decouverte de pairs/contenu
- **Identite** : Ed25519, provenance, verification
- **Storage** : key-value par app, replique via iroh-docs
- **Curators** : listes signees, cache DashMap, gossip
- **Browse** : vue agregee des projets via curators
- **Deploy** : pipeline verifie clone+zip+hash+sign+publish
- **Blob-serve** : decompression zip, LRU cache, CSP/COOP/COEP

Le daemon NE doit PAS connaitre :

- La semantique des apps (ce qu'est un "projet Babel", une "review")
- Les algorithmes de recherche ou de classement (au-dela du FTS5 local)
- Les workflows de creation d'apps (templates, build, test)
- La moderation fine / curation avancee

(doc#10 §4, doc#11 §7.2)

### 2.4 Ce qui sort du daemon : Factory

La decision P7 de la roadmap v3 dit "Factory = module daemon/broker
Rust". L'analyse de neutralite protocolaire (doc#12) propose de reviser
cette position : **Factory devient un outil client externe** (`sbfb-factory`,
crate Rust separe dans le workspace).

Inspiration du pattern sidecar (IPFS Cluster, Radicle httpd) :

```
+---------------------------------------------------+
|                  Shell React                       |
|  (Browse, Create, Search -- client generique)      |
+-------------+------------------+-------------------+
              |                  |
       +------v------+   +------v-----------+
       |   Daemon    |   |    Factory        |
       |   SBFB      |   |    (sidecar)      |
       |             |   |                   |
       | - blobs     |   | - template engine |
       | - feed      |   | - diff engine     |
       | - gossip    |   | - secret scanner  |
       | - identite  |   | - preview client  |
       | - events    |   | - deploy workflow |
       +------+------+   +------+-----------+
              |                  |
              |    utilise l'API HTTP du daemon
              |    pour publish/feed/blobs
              +------------------+
```

Factory communique avec le daemon via l'API HTTP loopback, sans
privilege special. Le daemon ne sait pas que Factory existe. (doc#12 §3)

### 2.5 Nouvelles primitives neutres necessaires

L'analyse croisee des besoins Factory et RRV (doc#10 §3, doc#12 §2)
identifie les primitives daemon manquantes :

| Primitive | Route | Priorite | Justification | Sprint cible |
|---|---|---|---|---|
| **Feed read paginee** | `GET /api/daemon/feed/entries?project_id=...&limit=...` | **P0** | Le feed est un log append-only signe mais totalement opaque via HTTP. Aucun client ne peut lire son contenu. | S67 |
| **Preview ephemere** | `POST /api/v1/preview/load` | **P0** | Charger un zip dans le cache blob-serve sans le persister dans iroh-blobs, avec TTL ~30 min. | S68 |
| **node_id optionnel dans deploy** | Modification de `POST /api/v1/deploy-from-repo` | **P0** | Les templates Factory ne connaissent pas le node_id a l'avance. | S67 |
| **CuratorVouched/CuratorDisendorsed dans feed** | Types dans `PublicFeedOperation` | **P0** | Endorsement public dans le feed, pas seulement dans les listes. | S67 |
| **Manifest extraction** | `GET /api/v1/project/{id}/manifest` | **P1** | Extraire et servir le SBFB.json depuis l'archive zip sans telecharger le blob entier. | S67 |
| **Provenance list** | `GET /api/v1/provenance/list` | **P1** | Batch indexation pour RRV (aujourd'hui seul `get_provenance_by_project` est expose). | S70 |
| **Search FTS5** | `GET /api/daemon/search?q=...&limit=...&offset=...` | **P1** | Index FTS5 sur les donnees locales du daemon. | S67-S70 |
| **Feed entry par hash** | `GET /api/daemon/feed/entries/{entry_hash}` | **P2** | Verification granulaire d'une entry. | S68+ |
| **Webhook/subscribe feed** (SSE ou long-poll) | -- | **P2** | Reagir aux nouvelles entries en temps reel. | S68+ |

(doc#10 §3, doc#12 §5)

### 2.6 Les 3 couches RRV

RRV n'est ni purement protocolaire ni purement applicatif. Il se
decompose en trois couches distinctes (doc#13 §10) :

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
                      rate limiter / PoW anti-spam
```

1. **Couche protocolaire** (S72) : `SearchManifest` est un wire format
   signe et gossipe, au meme rang que `CuratorListEntry` ou `FeedEntry`.
   C'est la seule partie qui engage le protocole reseau et doit etre
   gelee avec la meme rigueur.

2. **Couche service daemon** (S70-S71) : l'index FTS5, le calcul
   ProofCard, et les API search/proof-card sont des services locaux que
   le daemon fournit a partir de donnees qu'il possede deja. Ils ne
   modifient pas le protocole reseau.

3. **Couche applicative** (S70-S71 UI) : l'app sbfb-search, le
   composant ProofCard HTML, et toute logique d'affichage sont des apps
   externes qui consomment les services daemon via le bridge.

(doc#13 §7, §10)

---

## 3. Factory — outil client externe

### 3.1 Position architecturale (P7 revise)

La decision originale P7 (roadmap v3) definit Factory comme un "module
daemon/broker Rust". L'analyse de neutralite (doc#10, doc#11, doc#12)
revise cette position :

**Factory est un binaire CLI Rust separe** (`sbfb-factory`), installe a
cote du daemon. Elle n'est pas une app iframe, pas un module daemon, et
pas un service reseau. Elle orchestre des primitives daemon via l'API
HTTP loopback.

Sur 19 besoins Factory identifies (doc#12 §1.1) :

| Classification | Nombre | Items |
|---|---|---|
| Pas du protocole (logique locale) | 12 | Classification, templates, diff, scan secrets, scan deps, audit JSONL, lockfile, provenance locale, publish gate, domain packs |
| Deja couvert par le daemon | 4 | Provenance SLSA L1, deploy-from-repo, deploy prive, feed insert |
| Primitive daemon manquante | 4 | Validation manifest, preview ephemere, CuratorVouched types, node_id optionnel |

**12 des 19 besoins sont de la logique locale qui ne requiert aucune
interaction reseau.** Factory est principalement un outil de generation
de fichiers qui appelle le daemon a la fin pour publier. (doc#12 §1.2)

### 3.2 Architecture cible (sbfb-factory crate)

```
crates/
  sbfb-manifest/              # NOUVEAU -- lib pure, zero reseau
    src/lib.rs                # SbfbManifest, parse, validate
    src/bridge_allowlist.rs   # methodes bridge autorisees
    Cargo.toml                # deps: serde, serde_json, thiserror

  sbfb-factory/               # NOUVEAU -- binaire CLI
    src/main.rs               # clap CLI, orchestration gates
    src/templates/             # templates embarques (include_str!)
    src/diff.rs               # diff engine (fichiers tree)
    src/secret_scanner.rs     # regex scan secrets
    src/audit_log.rs          # JSONL writer
    src/preview.rs            # HTTP client -> daemon preview/load
    src/publish.rs            # HTTP client -> daemon deploy-from-repo
    src/template_lock.rs      # factory.template.lock gen
    src/provenance_local.rs   # factory.provenance.json gen
    Cargo.toml                # deps: sbfb-manifest, clap, reqwest,
                              #       blake3, serde, zip, walkdir
```

Dependances :

```
sbfb-factory --depends-on--> sbfb-manifest (validation locale)
sbfb-factory --HTTP client--> daemon (preview, publish, feed)

nexus-shell-daemon --depends-on--> sbfb-manifest (validation deploy)
```

`sbfb-factory` n'importe PAS `nexus-shell-daemon-core`,
`nexus-coordinator-rs`, ou `nexus-core-rs`. Il parle au daemon
uniquement via HTTP. (doc#12 §3)

### 3.3 Templates et generation

**Moteur de templates :** Copie de fichiers + substitution de variables.
Pas de moteur Tera/Handlebars. Le pattern :

1. Lire le template (dossier avec fichiers + template.json)
2. Pour chaque fichier : copier, substituer les variables (`{{name}}`,
   `{{version}}`)
3. Generer SBFB.json v2 avec les valeurs
4. Copier sbfb-bridge.js
5. Init git repo
6. Ecrire factory.template.lock + factory.provenance.json

(doc#2 §8, doc#7 §3.1)

**Template pack minimal :**

```
templates/
  sbfb-app-static/
    copier.yml
    template/
      AGENTS.md
      README.md
      SBFB.json
      package.json
      web/
      docs/
      .planning/active/.gitkeep
      docs/agent/PROCESS.md
      scripts/agent/agentctl.py
      prompts/agent/
      .githooks/
```

(doc#2 §8.1)

**Artefacts generes par defaut :**

| Artefact | Role |
|---|---|
| `AGENTS.md` | Instructions agent repo-locales |
| `docs/agent/PROCESS.md` | Process vendor-neutral |
| `scripts/agent/agentctl.py` | Gates et handoff |
| `.planning/active/` | Kickoff/plan/preflight/review |
| `.githooks/` | Precommit/auditor gate |
| `SBFB.json` | App manifest v2 |
| `factory.project.json` | Metadata Factory |
| `factory.template.lock` | Template id/version/hash |
| `factory.provenance.json` | Generation lineage |
| `tests/smoke/` | Smoke minimal |

(doc#2 §8.2)

**Exemple `factory.template.lock` :**

```json
{
  "schema_version": 1,
  "template_id": "static-storage",
  "template_version": "0.1.0",
  "template_hash": "<BLAKE3 du dossier template>",
  "generated_at": "<ISO 8601>",
  "generator_version": "1.0.0",
  "variables": {
    "name": "babel-reader",
    "version": "0.1.0"
  }
}
```

(doc#7 §3.3)

**Exemple `factory.provenance.json` :**

```json
{
  "schema_version": 1,
  "generator_node_id": "<hex>",
  "template_hash": "<BLAKE3>",
  "variables_hash": "<BLAKE3 du JSON variables>",
  "output_hash": "<BLAKE3 du workspace genere>",
  "generated_at": "<ISO 8601>",
  "signature": "<Ed25519 hex>"
}
```

(doc#2 §8.4, doc#7 §3.4)

### 3.4 Broker et securite (4 zones, capabilities, gates FG0-FG10)

#### Les quatre zones de securite

| Zone | Autorite | Role | Regle |
|---|---|---|---|
| Factory UI | faible | Interaction utilisateur | Jamais FS/git/shell direct |
| Factory Broker | forte locale | Autorisation et execution | Allowlist + audit + confirmations |
| Workspace Sandbox | moyenne bornee | Codegen/build/test | Pas de secrets, FS borne, reseau limite |
| Preview Sandbox | faible | Tester l'app produite | Meme isolation qu'une app publiee |

(doc#2 §4.2)

#### Capabilities minimales

| Capability | Sens | Gate |
|---|---|---|
| `fs.read_project` | Lire workspace borne | Auto si projet ouvert |
| `fs.write_project` | Ecrire workspace borne | Diff preview + confirmation |
| `git.local` | Init/commit/branch local | Confirmation |
| `shell.build_test` | Lancer commandes allowlistees | Sandbox + logs |
| `deps.install` | Installer deps | Lockfile + registry allowlist |
| `preview.static` | Servir preview | Sandbox |
| `publish.unverified` | Publier zip direct | Warning |
| `publish.verified` | Deploy-from-repo | Provenance complete |
| `network.web_search` | Chercher web | Consentement explicite |

(doc#2 §7.2)

#### Gates Factory FG0-FG10

Les gates sont prefixees "FG" pour eviter la confusion avec les gates
workflow G1-G9 du processus sprint SBFB. (doc#8 §7.3)

| Gate | Nom | Criteres cles | Testabilite | LOC estime |
|---|---|---|---|---|
| FG0 | Classification | Domaine, risque donnees, bridge methods, network needs | Semi-auto (struct validee + checkbox humaine) | ~100 |
| FG1 | Scope | MVP borne, non-goals explicites | Humain avec trace automatique | ~30 |
| FG2 | Template | Template id/version, hash BLAKE3, lockfile | **Entierement automatisable** | ~150 |
| FG3 | Manifest | Schema v2 valide, no node_id, bridge allowlist | **Entierement automatisable** | ~250 |
| FG4 | Diff | Preview obligatoire, approbation utilisateur | Semi-auto (trace + humain) | ~170 |
| FG5 | Sandbox | Canonicalize, prefix check, symlink deny, no shell iframe | **Entierement automatisable** | ~140 |
| FG6 | Secrets/deps | Scan secrets regex, lockfile, SBOM si publish | **Automatisable** (effort significatif) | ~260 |
| FG7 | Preview | Iframe sandbox, CSP, no external fetch | **Entierement automatisable** (code existant 90%) | ~140 |
| FG8 | Provenance | factory.provenance.json, generator version, template+variables hash | **Entierement automatisable** | ~240 |
| FG9 | Publish | Repo HTTPS, commit hex-40, artifact hash, provenance, Browse, feed | **Entierement automatisable** (code existant quasi-complet) | ~260 |
| FG10 | Review | Sprint review, verdict PASS, evidence pack | Semi-auto (completude testable) | ~100 |
| **Total** | | | | **~1840** |

Code existant reutilisable : ~60-70% du code necessaire existe deja
grace au deploy path, blob-serve, provenance, et bridge protocol.
(doc#8 §2, §6)

**Ordre des gates (graphe de dependances) :**

```
FG0 -> FG1 -> FG2 -> FG3 -> FG4 -> FG5 -> FG6 -> FG7 -> FG8 -> FG9 -> FG10
```

L'ordre est correct. La seule discussion est FG4/FG5 : montrer le diff
(texte) avant de materialiser les fichiers (securite) est le bon flux.
(doc#8 §3)

**Gates manquantes identifiees :**

- Limite de taille archive (max 10 MB, max 500 fichiers) — a ajouter
  dans FG5/FG6. (doc#8 §4.1)
- Verification bridge runtime (methode appelee vs allowlist manifest) —
  enrichir FG3 ou creer FG3.5. (doc#8 §4.5)
- Invariant coherence template_hash entre FG2 et FG8. (doc#8 §5.2)
- Verification entrypoint (index.html valide + sbfb-bridge.js present).
  (doc#8 §4.6)
- Deduplication/idempotence (memes inputs = meme hash). (doc#8 §4.7)

### 3.5 Publish path (preview -> Local Draft -> Verified Release)

```
1. User: "cree Babel, bibliotheque P2P traduction"
2. Factory UI cree une intention.
3. Factory selectionne template "sbfb-app-static".
4. Factory genere un plan et un diff virtuel.
5. User valide les capabilities.
6. Workspace sandbox cree le repo.
7. Tests/build/lint passent dans sandbox.
8. Preview iframe sert l'app (POST /api/v1/preview/load).
9. Provenance locale Factory ecrite (factory.provenance.json).
10. User commit + push repo public.
11. POST /api/v1/deploy-from-repo.
12. Daemon clone, valide manifest v2/index.html, zippe, hash BLAKE3,
    signe provenance Ed25519.
13. Blob archive publie/persiste (iroh-blobs).
14. Browse entry creee (gossip announce).
15. Feed ReleasePublished cree automatiquement.
16. Evidence pack capture provenance, feed entry, Browse, tests.
```

(doc#2 §4.3, doc#5 §9, doc#9 §5)

### 3.6 Flux "creer Babel" (etape par etape)

```
sbfb-factory create --template static-storage --name babel-reader
  (local : copie template, gen SBFB.json v2, git init)
  (local : factory.template.lock, factory.provenance.json)

... developpement de l'app ...

sbfb-factory validate manifest ./babel-reader/SBFB.json
  (local : sbfb-manifest::validate)

sbfb-factory scan-secrets ./babel-reader
  (local : regex patterns)

sbfb-factory diff ./babel-reader
  (local : compare workspace vs template)

sbfb-factory preview ./babel-reader --daemon http://127.0.0.1:PORT
  (local : zip le repertoire)
  (HTTP : POST /api/v1/preview/load -> hash)
  (ouvre : http://127.0.0.1:PORT/blob-serve/{hash}/index.html)

sbfb-factory publish ./babel-reader --daemon http://127.0.0.1:PORT
  (lit running.json pour le token + port)
  (HTTP : POST /api/v1/deploy-from-repo si repo public)
  (le daemon gere : provenance, blob, gossip, feed entry)

App publiee sur le reseau.
```

(doc#12 §3.4)

### 3.7 Contrainte node_id — resolution

**Probleme :** `deploy.rs` ligne 119-128 exige `sbfb.node_id == state.node_id`.
Les templates Factory ne connaissent pas le node_id a l'avance.
`"PLACEHOLDER"` dans les SBFB.json existants est un hack. (doc#6 §1)

**5 options evaluees :**

| Option | Description | Deploy verifie | Verification tiers | Semantique |
|---|---|---|---|---|
| A (placeholder) | Factory remplace PLACEHOLDER | OUI | OUI (meme hash) | NON (hack) |
| B (optionnel) | node_id `Option<String>` | OUI | OUI | OUI |
| C (fichier separe) | DEPLOY.json + SBFB.json | OUI | OUI | MOYENNE |
| D (daemon only) | **SBFB.json sans node_id** | OUI | OUI | **OUI** |
| E (ecrasement) | daemon ecrit node_id au deploy | **NON** (casse hash) | **NON** | NON |

**Decision : Option D** (node_id dans le daemon, pas dans le manifeste).
(doc#6 §6)

Rationale :

- Le node_id est une propriete du **deploiement**, pas de l'**app**.
- La provenance signee Ed25519 porte l'attribution cryptographique.
- Le node_id dans SBFB.json n'est pas signe (dans le zip, pas dans la
  signature directe) — il n'a pas de valeur cryptographique propre.
- La reproductibilite du hash est amelioree (meme code = meme hash pour
  tous les deployeurs).
- L'analogie : un paquet Debian ne contient pas l'identite du miroir.
  Le miroir signe le Release file.

**Changement dans deploy.rs :** ~5 lignes. `node_id: String` ->
`node_id: Option<String>` avec `#[serde(default)]`, suppression du bloc
de verification, warning log si present. (doc#6 §6.2)

### 3.8 SBFB.json v2 (manifest app enrichi)

```json
{
  "schema_version": 2,
  "name": "babel-reader",
  "version": "0.1.0",
  "display_name": "Babel Reader",
  "description": "Reader offline multilingue avec provenance source",
  "category": "language",
  "license": "AGPL-3.0-or-later",
  "lang": "fr",
  "bridge": {
    "methods": [
      "storage_get", "storage_set", "storage_list",
      "storage_delete", "identity_pubkey", "node_status",
      "browse_list", "task_submit", "provenance_get",
      "provenance_verify", "feed_cursor_get"
    ],
    "events": ["storage_update", "task_result_ready"],
    "heartbeat": true
  },
  "tech": {
    "type": "static",
    "entry_point": "index.html",
    "build_command": null
  },
  "requirements": {
    "min_bridge_version": "1.0.0",
    "offline_capable": true,
    "estimated_size_kb": 50
  }
}
```

**Differences v1 -> v2 :**

| Champ | v1 | v2 |
|---|---|---|
| `schema_version` | Absent | 2 |
| `node_id` | **Obligatoire** | **Supprime** |
| `name` | Present (non lu par Rust) | Present (lu, obligatoire) |
| `version` | Optionnel | Obligatoire |
| `display_name`, `description`, `category`, `license`, `lang` | Absents | Nouveaux |
| `bridge` | Absent | Methods + events declares |
| `tech` | Absent | Type + entry_point |
| `requirements` | Absent | Min bridge version + offline + size |

Compat descendante assuree : tous les champs `#[serde(default)]`. Les
SBFB.json v1 existants parsent sans erreur. (doc#6 §7, doc#5 §7)

---

## 4. RRV — Recherche Reseau Verifiable

### 4.1 Les 3 scopes (@protocole, @dev, @web)

| Scope | Donnees | Valeur | Etat |
|---|---|---|---|
| `@protocole` / `@network` | Browse entries, feed entries, provenance records, curator lists, archives zip, tasks/kudos | Le differenciateur SBFB : recherche dans un catalogue P2P verifie avec provenance Ed25519 + feed hash-chain BLAKE3 | Donnees deja presentes dans le daemon. ~300 LOC pour FTS5. |
| `@dev` | Code source, manifests, symbols AST, capabilities, risks, tests | Aide Factory a trouver des patterns, produire des citations fichier:ligne:hash | N'existe pas. Necessite sbfb-factory + tree-sitter. S71+ par defaut apres S70 Process Portable Complete ; non requis Gate 1. |
| `@web` | Sources web externes (SearXNG sidecar) | Trouver les meilleures briques OSS, comparer avec le reseau SBFB | N'existe pas. Necessite Docker sidecar. Questions privacy non resolues. |

(doc#3 §4, doc#14 §1)

**Perimetres dans le chat :**

| Tag | Sens | Exemple |
|---|---|---|
| `@current` | App ouverte, defaut | `@current resume les erreurs recentes` |
| `@Babel` | App/projet precis | `@Babel quelles langues sont faibles ?` |
| `@network` | Catalogue SBFB verifie | `trouve une app compatible avec ma machine` |
| `@web` | Web externe | `compare avec les solutions existantes` |
| `@dev` | Recherche developpement | `cherche API, manifests, patterns` |
| `@private:<group>` | Groupe prive | `cherche dans les resultats du groupe labo-x` |

(doc#3 §4.1)

### 4.2 Ordonnancement : @protocole -> @dev booste -> @web

L'analyse factuelle (doc#14) conclut :

**Addendum 2026-05-21 :** la decision produit recente rend `@dev`
explicitement non bloquant pour Arc 2/Gate 1. Le pilote ferme valide
la chaine `@protocole` (search, feed, provenance, Proof Cards,
publish, Babel dogfood). `@dev` peut demarrer en stretch uniquement
si cela ne ralentit pas cette chaine ; sinon il glisse S71+ apres S70
Process Portable Complete.

**Commencer par @protocole**, pas par @dev. Raisons :

1. **Les donnees @protocole existent deja** (7 familles SQLite + DashMap).
   @dev et @web partent de zero.
2. **Le differenciateur SBFB est @protocole, pas @dev.** Aucun outil
   existant ne fait de la recherche dans un catalogue P2P verifie avec
   provenance Ed25519 + feed hash-chain. @dev est un terrain ou SBFB se
   mesure a Claude Code, Cursor, Copilot.
3. **Les testeurs du pilote ont besoin de @protocole** (chercher une app,
   voir sa provenance). Ils n'ont pas besoin de @dev (pas des
   developpeurs) ni de @web (pilote ferme).
4. **@dev n'a pas d'utilisateur immediat** tant que Claude Code existe et
   que le seul developpeur l'utilise deja depuis 65 sprints.
5. **@web n'a de valeur que si Factory consomme les resultats.** @web
   avant Factory = SearXNG rebadge, pas un produit SBFB.

Ordre recommande :

```
Etape 1 (S67-S68) : @protocole
  FTS5 daemon search + Proof Cards + citations
  Cout : ~300-400 LOC, zero nouvelle dependance

Etape 2 (S71+ par defaut apres S70 process portable, stretch S68-S69 si zero-impact) : @dev
  Index local/source-only dans sbfb-factory
  tree-sitter en stretch
  Cout : ~400-600 LOC dans sbfb-factory

Etape 3 (S72+, apres pilote) : @web
  SearXNG sidecar optionnel
  Cout : ~300 LOC + Docker ops + politique privacy
```

(doc#14 §5)

### 4.3 RRV @protocole : FTS5 daemon, donnees existantes

**Decision moteur : FTS5 d'abord, pas Tantivy.** (doc#5 §10, roadmap v3
§P12)

| Critere | FTS5 | Tantivy |
|---|---|---|
| Dependance ajoutee | Zero (rusqlite bundled) | Crate ~0.22, nouvelle dep |
| Integration DB existante | Transactions ACID avec tables metier | Index fichier separe |
| Fuzzy search | Non natif | Oui |
| Stemming multi-langue | Tokenizer basique | 17 langues |
| Volume supporte | Correct pour < 50K docs | Excellent |
| Decision | **MVP (S70)** | Gate post-S75 si volume/p95 le justifient |

Tantivy reste la recommandation long terme si le volume depasse 50K docs
ou si les features manquantes de FTS5 (fuzzy, stemming) deviennent
bloquantes. (doc#4 §2.3 vs doc#5 §10)

**Donnees existantes cherchables :**

| Famille | Stockage | Champs cles |
|---|---|---|
| BrowseEntry | DashMap in-memory | project_name, category, description, curator_name, status, archive_hash, repo_url, provenance_hash |
| FeedEntry | SQLite `public_feed` | op (ReleasePublished/SourceBecameStale), author_pubkey, payload JSON |
| ProvenanceRecord | SQLite `provenance_records` | repo_url, commit_sha, artifact_hash, node_id, app_version |
| CuratorListEntry | DashMap in-memory | project_id, name, category, description, curator_pubkey |
| TaskRecord / KudosEntry | SQLite `tasks` + `kudos` | task_id, project_id, model, worker_node_id |
| App archive (zip) | iroh-blobs MemStore | README.md, index.html meta, SBFB.json |

(doc#4 §1.1)

**API locale :**

```
GET /api/daemon/search?q=<query>&scope=local&limit=20&offset=0
```

Response :

```json
{
  "query": "traduction offline",
  "results": [
    {
      "score": 0.87,
      "source_type": "browse_entry",
      "project_id": "abc...",
      "project_name": "Babel",
      "excerpt": "Traduction <mark>offline</mark> P2P...",
      "citations": [
        {
          "source_type": "feed_entry",
          "entry_hash": "f81...",
          "timestamp": 1700000000
        }
      ]
    }
  ],
  "total": 3,
  "took_ms": 2
}
```

(doc#4 §4.1.5)

**Bridge method `search` :** Legitime — la recherche locale est une
capacite fondamentale. Le pattern est identique a `browse_list`. Les
requetes ne quittent jamais le loopback. (doc#13 §4.1)

### 4.4 RRV @dev : index local workspace + code apps reseau verifie

**Decision 2026-05-21 :** ce scope n'est pas une condition de succes
du pilote. Pour les gros repos OSS GitHub, `@dev` doit passer par un
contrat `source-only`/`source-index` separe : un repo source externe
n'est pas une app SBFB, ne passe pas le meme `deploy-from-repo`, et ne
doit pas recevoir le label `verified SBFB app`. Les premiers seeds OSS
doivent etre curates, bornes, hashes par commit/fichier, et etiquetes
comme `external OSS source index`.

**Objets indexes :**

| Objet | Champs minimaux |
|---|---|
| `Project` | id, path, repo_url, commit_sha, template_id, status |
| `Release` | artifact_hash, provenance_hash, source_state, build_state |
| `SourceFile` | path, language, size, content_hash, last_seen |
| `CodeChunk` | file_id, range, text_hash, text, symbols, embedding_ref |
| `Symbol` | name, kind, file_id, range, exported, references |
| `Capability` | method/name, direction, schema_ref, file_id, risk |
| `RiskFinding` | tool, rule, severity, file, range, status |

(doc#2 §5.3)

**Schema conceptuel SQLite :**

```sql
CREATE TABLE projects (
  project_id TEXT PRIMARY KEY,
  path TEXT NOT NULL,
  repo_url TEXT,
  commit_sha TEXT,
  template_id TEXT,
  created_at INTEGER NOT NULL
);

CREATE VIRTUAL TABLE chunks_fts USING fts5(
  chunk_id UNINDEXED,
  project_id UNINDEXED,
  file_id UNINDEXED,
  path, language, text,
  tokenize = 'unicode61'
);

CREATE TABLE capabilities (
  capability_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  kind TEXT NOT NULL,
  name TEXT NOT NULL,
  risk_level TEXT NOT NULL
);
```

(doc#2 §5.5)

### 4.5 RRV @web : sidecar externe, labels separes, post-pilote

@web est un sidecar optionnel (SearXNG ou equivalent) qui fournit des
resultats externes avec des labels de confiance separes. (doc#3 §5)

Regle non negociable : **fusionner le ranking est acceptable. Fusionner
la confiance ne l'est pas.** Chaque resultat garde son label de preuve.
(doc#3 §5)

### 4.6 Proof Cards : data model, formule de score, risk factors

**Structure ProofCard :**

```typescript
interface ProofCard {
  project_id: string;
  project_name: string;
  source: {
    type: "browse" | "feed" | "provenance" | "app_content";
    entry_hash?: string;
    file_path?: string;
    line?: number;
  };
  hash: {
    archive_hash?: string;
    artifact_hash?: string;
    provenance_hash?: string;
    content_hash?: string;
  };
  license: {
    spdx?: string;
    source: "manifest" | "inferred" | "unknown";
  };
  freshness: {
    last_verified_at: number;
    age_days: number;
    state: "fresh" | "aging" | "stale" | "unknown";
  };
  provenance: {
    verified: boolean;
    repo_url?: string;
    commit_sha?: string;
    slsa_level: 0 | 1;
  };
  risk: {
    level: "low" | "medium" | "high" | "critical";
    factors: string[];
  };
  curation: {
    curator_count: number;
    curator_names: string[];
  };
  confidence: number;        // 0-100
  formula_version: number;   // versionning de la formule
}
```

(doc#4 §4.2.1, doc#13 §9.2 pour formula_version)

**Formule de score (deterministe, transparente) :**

```
Base: 30 points (le resultat existe)

+ 20 si provenance.verified == true
+ 10 si is_open_source == true
+ 10 si freshness.state == "fresh"
+  5 si freshness.state == "aging"
+ 10 si curation.curator_count >= 1
+ 10 si curation.curator_count >= 3
+  5 si license.spdx != null
+  5 si hash.archive_hash != null

- 10 si risk contient "stale_source"
- 15 si risk contient "no_provenance"
- 10 si risk contient "unverified_deploy"
-  5 si risk contient "old_release"

Clamp: min(0, max(100, score))
```

Ce n'est pas un "trust score" social — c'est un "evidence score". Un
projet avec toutes les preuves a 100%. Un projet avec seulement
l'existence a 30%. (doc#4 §4.2.2, roadmap v3 §P6)

**Risk factors automatiques :**

| Facteur | Condition | Impact |
|---|---|---|
| `no_provenance` | provenance_hash absent | -15 |
| `stale_source` | SourceBecameStale dans le feed | -10 |
| `no_curator` | Aucun curator ne vouch | -10 |
| `single_curator` | Un seul curator | -5 |
| `unverified_deploy` | Pas de deploy-from-repo | -10 |
| `old_release` | Derniere release > 90 jours | -5 |
| `no_open_source` | is_open_source == false | 0 (info) |

(doc#4 §4.2.3)

### 4.7 SearchManifest : wire format, opt-in, gossip, privacy

**Format :**

```json
{
  "v": 1,
  "type": "sbfb.search_manifest",
  "node_id": "<daemon_ed25519_node_id_hex>",
  "created_at": 1700000000,
  "projects": [
    {
      "project_id": "<hex-64>",
      "project_name": "Babel",
      "category": "translation",
      "description": "Traduction P2P verifiable",
      "keywords": ["traduction", "offline", "p2p"],
      "license": "AGPL-3.0-or-later",
      "artifact_hash": "<BLAKE3 hex-64>",
      "provenance_hash": "<BLAKE3 hex-64>",
      "is_open_source": true,
      "curator_count": 2
    }
  ],
  "feed_cursor": {
    "last_seq": 150,
    "last_entry_hash": "<BLAKE3 hex-64>"
  },
  "index_stats": {
    "project_count": 5,
    "document_count": 1240,
    "total_size_bytes": 524288
  },
  "signer_pubkey": "<Ed25519 hex-64>",
  "signature": "<Ed25519 hex-128>"
}
```

**Canonical bytes :** `DOMAIN_SEARCH_MANIFEST_V1 || 0x00 || JCS(manifest)`

**Limites :** `projects.len()` <= 256, `description` <= 280 bytes,
`keywords` <= 10 par projet (64 bytes chacun), total manifest <= 1 MB.

(doc#4 §4.3.1)

**Publication opt-in :** Le daemon ne publie PAS de manifest par defaut.
Activation via `POST /api/daemon/search/publish-manifest` ou checkbox UI
"Rendre mon index decouvrable". (doc#4 §4.3.2)

**Discovery :** Deux modes — gossip topic
`"nexus-grid/search-manifest/v1"` (temps reel) et feed event
`SearchManifestPublished` (retroactif via replay). (doc#4 §4.3.4)

**Anti-spam :** Rate limiting 1 manifest/noeud/heure, taille max 1 MB,
projects <= 256, signature Ed25519 obligatoire, PoW 16-bit optionnel.
(doc#4 §4.3.5)

**Privacy :** Ce qu'un manifest revele : quels projets le noeud a
indexes (public par design, opt-in). Ce qu'il ne revele PAS : les
requetes de recherche (jamais envoyees au reseau), le contenu complet de
l'index, les donnees de stockage privees. (doc#4 §4.3.6)

### 4.8 Labels de preuve (8 labels)

| Label | Sens |
|---|---|
| `Local indexed` | Lu dans workspace local |
| `Local generated` | Cree par Factory, non publie |
| `Local tested` | Build/test local passe |
| `SBFB verified` | Provenance + artifact + source OK |
| `SBFB unverified` | Vu/publie sans chaine complete |
| `SBFB stale` | Provenance existe mais source non reverifiable |
| `Web external` | Source externe non verifiee SBFB |
| `Web claim` | Affirmation web non prouvee |

Regle : **ne jamais fusionner les labels de confiance**. Un resultat @web
ne peut pas devenir "SBFB verified". (doc#2 §5.7, doc#3 §5)

### 4.9 Ranking et citations

**Ranking MVP (sans embeddings) :**

```
score =
  0.55 * bm25
+ 0.20 * path/symbol match
+ 0.15 * proof
+ 0.10 * safety
```

**Ranking cible (avec embeddings) :**

```
score =
  0.35 * bm25
+ 0.30 * vector
+ 0.15 * proof
+ 0.10 * availability
+ 0.10 * safety
```

(doc#2 §5.6)

**Format des citations :**

```json
{
  "source_type": "feed_entry",
  "project_id": "abc123...",
  "entry_hash": "f81ced7d...",
  "timestamp": 1700000000,
  "signature": "ed25519_hex...",
  "text_excerpt": "Release published for sbfb-explorer"
}
```

Pour les fichiers dans les archives :

```json
{
  "source_type": "app_file",
  "project_id": "abc123...",
  "file_path": "README.md",
  "line": 42,
  "content_hash": "blake3_hex...",
  "archive_hash": "iroh_blob_hash...",
  "text_excerpt": "Ce fichier decrit..."
}
```

(doc#4 §4.1.4)

---

## 5. Babel — premier dogfood

### 5.1 Pourquoi Babel

Babel force les vrais besoins du protocole : corpus, provenance,
traduction IA, validation humaine, lecture offline, sync locale, compute
batch, recherche par langue/source/chunk, partage anti-censure, UX non
technique. Si Factory sait creer Babel proprement, elle valide le
template SBFB, le bridge, le storage, les tasks, la provenance, la
preview, la publication, et RRV @protocole. RRV @dev reste un
enrichissement post-pilote. (doc#2 §9.1 + recadrage 2026-05-21)

### 5.2 MVP Babel via Factory

**Scope canari (a livrer) :**

- App `babel-reader` creee avec Factory par le dogfood utilisateur
- 3 textes domaine public (Gutenberg/Wikisource), ~500 mots chacun
- ~5 langues fixtures (FR/EN/ES/DE/PT)
- Liste de textes
- Vue lecture
- Toggle langue
- Progression/bookmarks via bridge storage
- Source manifest visible
- Provenance app visible
- Feed cursor visible
- Verification provenance depuis le bridge

**Scope cuts recommandes (reporter a S74) :**

- Reviews minimales (feature sociale, complique le storage)
- Task traduction mock (le mock ne prouve rien que les fixtures ne
  prouvent deja)

Avec les scope cuts, l'effort est de ~5.5 jours sur 10j disponibles,
marge saine. (doc#9 §1)

**Bridge methods requises : 11/11 COMPLETES** (doc#9 §2)

Toutes les methodes listees existent dans les 3 couches (schema Zod,
dispatch host-side, SDK client) et ont un backend Rust fonctionnel.
Aucun effort supplementaire requis pour les methodes bridge.

**Storage : local suffit pour le canari** (doc#9 §3)

Le code `storage_api.rs` a un hardcode `REPLICATED_APPS: &[&str] = &["sbfb-ideas"]`.
Les bookmarks, reviews et progression de `babel-reader` ne sont pas
repliques entre testeurs — acceptable pour un canari ferme de 2-3
personnes. Replication Babel = item S74.

### 5.3 Tests d'acceptance Babel (25 items)

**Tests bloquants avant canari :**

1. `deploy-from-repo` accepte app sans `node_id`
2. `deploy-from-repo` refuse manifest invalide
3. `deploy-from-repo` refuse methode bridge inconnue
4. `deploy-from-repo` refuse repo non HTTPS pour feed
5. App generee contient `index.html`, `SBFB.json`, SDK bridge
6. App generee contient planning sprint
7. App generee contient `factory.template.lock`
8. App generee contient `factory.provenance.json`
9. Path traversal refuse
10. Symlink refuse
11. Secret fixture refuse
12. Preview iframe smoke test
13. Babel affiche textes fixtures
14. Babel lit/ecrit progression storage
15. Babel appelle `identity_pubkey`
16. Babel affiche provenance
17. Babel lit feed cursor
18. Babel deploy -> Browse -> open -> provenance verify
19. Feed `ReleasePublished` cree si chemin cable
20. Test negatif : aucune methode `babel_*`, `factory_*`, `shell_*`

**Tests post-S70/S71 :**

21. RRV trouve Babel localement
22. Proof Card affiche source, artifact, provenance, feed
23. Proof Card indique stale si source cassable
24. Score de completude deterministe
25. FTS5 p95 acceptable sur corpus test

(doc#5 §12, doc#9 §6)

### 5.4 Source manifest minimal Babel

Chaque texte doit avoir un manifest source stocke en JSON dans le bridge
storage (`manifests/sources/{source_id}`) :

```javascript
await bridge.setStorage("manifests/sources/gutenberg-001", {
  source_id: "gutenberg-001",
  source_url: "https://www.gutenberg.org/ebooks/11",
  source_hash: "abc123...",
  rights: {
    redistribution: true,
    translation: true,
    license: "PD-US",
    jurisdictions: ["US", "EU"],
    attribution: "Lewis Carroll, via Project Gutenberg",
    takedown_policy: "N/A (public domain)"
  },
  imported_at: "2026-05-18T12:00:00Z",
  manifest_author: "<pubkey hex>"
});
```

Le manifest source est purement metier Babel, pas un schema protocole.
Pour le canari, les manifests sont PRE-INCLUS dans le zip de l'app
(fichiers JSON statiques). (doc#9 §4)

### 5.5 Corpus et sources

Gutenberg est un bon corpus de depart (textes accessibles, metadonnees,
formats simples, volume suffisant). Mais Factory ne doit pas coder
"Gutenberg" dans le protocole. Le pipeline est generique :

```
SourceAdapter -> SourceManifest -> ContentChunks ->
TranslationTasks -> ReviewQueue -> PublishedText
```

(doc#2 §9.3)

---

## 6. Briques OSS a reutiliser

### 6.1 P0 (a reutiliser en priorite)

| Brique | Projet | Source | Role Factory | Decision |
|---|---|---|---|---|
| Templates | Copier | copier.readthedocs.io | Generation repo versionnable | P0 moteur concret |
| Catalog model | Backstage Catalog | backstage.io | Metadata/catalogue projet | P0 inspiration modele |
| Scaffolder model | Backstage Templates | backstage.io | Steps, parameters, dry-run | P0 inspiration UX |
| Coding agent ref | OpenHands | github.com/OpenHands/OpenHands | Reference agent + sandbox | P0 reference |
| Dev env | Dev Containers | devcontainers/spec | Workspace reproductible | P0 standard |
| Local search | SQLite FTS5 | sqlite.org/fts5 | Index lexical local | P0 simple |
| Code parse | tree-sitter | tree-sitter/tree-sitter | AST/symbols | P0 pour @dev |
| SAST | Semgrep CE | semgrep.dev | Scan rapide | P0 local |
| SBOM | Syft | anchore/syft | SBOM artifacts | P0 provenance |
| Scan | Trivy | aquasecurity/trivy | Vuln/misconfig/secrets | P0 pin strict |
| Provenance | in-toto | in-toto.io | Attestation chain | P0 vocabulary/format |
| Test preview | Playwright | playwright.dev | E2E preview | P0 |
| CI | Woodpecker | woodpecker-ci.org | Self-hosted CI | P0 |

### 6.2 P1 (utiles mais pas coeur MVP)

| Brique | Projet | Role | Decision |
|---|---|---|---|
| Templates alt | Cookiecutter | Fallback templates | P1 |
| Generator JS | Plop | Micro-generators | P1 |
| Search Rust | Tantivy | Gros index full-text | P1 apres FTS5 |
| Vector local | sqlite-vec | Embeddings locaux | P1 feature flag |
| Code search | Zoekt | Symbol/path ranking | P1 inspiration |
| Semantic SAST | CodeQL | Analyse profonde | P1 lourd |
| Pipelines | Dagger | Build/test portable | P1 |
| GHA local | nektos/act | Tester workflows GH | P1 |
| Policy | OPA | Policies capabilities | P1 |
| Config schema | CUE | Validation templates | P1 |
| Signing | cosign | Signature artifacts | P1 |
| Web sidecar | SearXNG | `@web` privacy sidecar | P1, jamais trust core |

### 6.3 P2 (references UX/long terme)

| Brique | Projet | Role | Decision |
|---|---|---|---|
| Browser AI app builder | bolt.diy | Reference UX | Seulement |
| Local app builder | Dyad | Reference UX | Seulement |
| SWE benchmark | SWE-agent | Agent research | Reference |
| IDE assistant | Continue | Coding assistant | Option user |
| microVM | Firecracker | Isolation forte | Long terme |
| Sandboxed container | gVisor | Isolation conteneur | Long terme |
| Bubblewrap | bubblewrap | Sandbox Linux | Pas Windows-first |

### 6.4 Anti-decisions

Ne pas faire :

- Mettre LlamaIndex/Haystack au coeur du protocole
- Imposer Qdrant/Weaviate/Elasticsearch pour MVP local
- Faire confiance a des embeddings fournis par peers
- Donner FS/git/shell directement a l'iframe
- Faire du web crawling automatique
- Melanger `Web external` et `SBFB verified`
- Publier automatiquement une app generee sans gate humain
- Faire de Factory une dependance runtime obligatoire de Babel

(doc#2 §6)

---

## 7. Securite

### 7.1 Quatre zones de securite (rappel synthetique)

Voir la table detaillee en §3.4. Le principe est simple : **l'iframe est
une interface, le broker est l'autorite.** (doc#2 §7.1)

Le code app/iframe peut demander (afficher, generer, previewer, publier).
Il ne peut pas (lire arbitrairement le FS, ecrire hors workspace, lancer
shell, modifier git, installer deps, lire secrets, acceder au reseau
host). (doc#2 §7.1)

### 7.2 Gates de securite Factory (FG0-FG10)

Table complete en §3.4. Points cles :

- **60-70% du code necessaire existe deja** dans le codebase (deploy path,
  blob-serve, provenance, bridge protocol). (doc#8 §2)
- Les gates G5 (sandbox), G7 (preview), G9 (publish) sont quasi-couvertes
  par le code existant (`validate_zip_path`, `BLOB_SERVE_CSP/COOP/COEP`,
  `deploy_from_repo`). (doc#8 §2)
- Les gaps principaux sont G0 (mirror Rust du BridgeMethodSchema), G3
  (manifest v2), G6 (secret scanner), G8 (struct factory provenance).

### 7.3 Supply chain

Regles :

- Lockfiles obligatoires
- `npm ci --ignore-scripts` en premiere passe
- Scripts postinstall seulement apres review
- Registry allowlist
- SBOM Syft
- Scan Trivy/Semgrep (pin strict, verification d'artefacts)
- Provenance in-toto/SLSA-like
- Cache content-addressed
- Zip traversal checks
- Build network-off apres fetch quand possible
- Pins de versions pour outils de securite eux-memes

(doc#2 §7.4)

### 7.4 Threat model Factory

| Risque | Impact | Mitigation |
|---|---|---|
| Factory devient protocole metier | Couplage long terme | Crate separe + primitives generiques |
| Iframe gagne FS/shell | Privilege escalation | Broker seul autoritaire |
| Templates non versionnes | Impossible a auditer | `factory.template.lock` |
| Generation sans provenance | Code opaque | `factory.provenance.json` obligatoire |
| Web externe melange a SBFB | Fausse confiance | Proof labels non fusionnes |
| RRV reseau trop tot | Produit mensonger | LocalOnly d'abord |
| Deps install non controle | Supply-chain | Lockfile, allowlist, scans |
| Babel trop large MVP | Blocage | Reader + fixture + review minimal |
| `.sbfb/apps` devient repo | Confusion source/runtime | Repo source dans `Documents/Code` |
| Process sprint cache dans modele | Non portable | Generer process en fichiers |
| Fork malveillant sans node_id | Deux miroirs meme hash | Provenance signe identifie le deployeur |

(doc#2 §13, doc#6 §10)

---

## 8. Compute distribue (vision long terme)

### 8.1 Groupes prives de compute

Produit : un groupe ferme invite ses machines et partage CPU/GPU pour
recherche, coding, traduction, audit, benchmarks, sans publier les
resultats sur le reseau public. (doc#3 §7.1)

Primitives necessaires : groupe prive avec invite, allowlist
membres/machines, chiffrement tasks/artifacts, consentement GPU/CPU,
quotas par membre, logs/audit internes, stockage prive, publication
volontaire vers le reseau public.

MVP raisonnable : `Private compute group = batch tasks + artifacts chiffres + allowlist + quotas.`

### 8.2 Compute public app-driven

Produit : une app publique demande de la puissance au reseau. Les workers
acceptent selon leurs regles de consentement. (doc#3 §7.2)

MVP raisonnable : `App-driven batch compute = une app publie des taches paralleles, les workers opt-in les executent, les resultats sont signes et verifiables.`

### 8.3 Batch distribue verifiable (bon MVP)

Exemples : traduire 10 000 chunks Babel, indexer 100 repos, lancer 500
tests, generer 20 variantes, auditer 30 fichiers. (doc#3 §8.1)

Adapte car : parallele naturellement, tolerant a la latence, verifiable
par hash/signature/quorum, compatible consentement worker.

### 8.4 Inference distribuee temps reel (non MVP)

Problemes : latence reseau, memoire distribuee, orchestration
tensor/pipeline parallel, confidentialite, drivers GPU heterogenes, cout
debug eleve. (doc#3 §8.2)

Decision : **batch distribue d'abord. Inference distribuee temps reel =
recherche long terme.** (doc#3 §8.2)

---

## 9. Decisions prises

### 9.1 Table de toutes les decisions

| # | Decision | Date | Statut | Source | Rationale |
|---|----------|------|--------|--------|-----------|
| D1 | FTS5 d'abord, Tantivy gate post-S75 | 2026-05-18 | **Gelee** | doc#5, roadmap v3 P12 | Zero dep ajoutee, tables SQLite existantes, volume < 50K pre-launch |
| D2 | Factory hors daemon (crate sbfb-factory) | 2026-05-19 | **Gelee** | doc#12 | Neutralite protocolaire, prior art convergent (IPFS Cluster, Radicle httpd) |
| D3 | node_id retire de SBFB.json (Option D) | 2026-05-18 | **Gelee** | doc#6 §6 | Attribution dans la provenance signee, pas dans le manifest. Reproductibilite hash amelioree |
| D4 | Feed raw-op extensible (pas de bump par nouvelle op) | 2026-05-18 | **Gelee** | roadmap v3, CLAUDE.md | Noeuds anciens stockent et propagent les ops inconnues |
| D5 | Babel = premier dogfood Factory (S69) | 2026-05-18 | **Gelee** | doc#5 §1 | Force les vrais besoins, prouve la boucle Factory |
| D6 | @protocole avant @dev avant @web | 2026-05-19 | **Gelee** | doc#14 §5 | Donnees existantes, differenciateur SBFB, besoin pilote |
| D7 | SBFB.json v2 (schema_version: 2) | 2026-05-18 | **Gelee** | doc#5 §7, doc#6 §7 | Manifest enrichi pour Factory, RRV, Proof Cards |
| D8 | Gates Factory prefixees FG (FG0-FG10) | 2026-05-18 | **Gelee** | doc#8 §7.3 | Evite confusion avec gates workflow G1-G9 |
| D9 | CuratorVouched minimal en S65 (Option D faisabilite) | 2026-05-18 | **Gelee** | doc#7 §5 | Cout marginal ~205 LOC, benefice structurel pour pilote et Proof Cards |
| D10 | S66 OBLIGATOIRE avant S69 | 2026-05-18 | **Gelee** | doc#7 §2, doc#9 §7 | Gate "utilisable 24h" impossible sans persistence blobs |
| D11 | Score de completude de preuve (pas "trust score") | 2026-05-18 | **Gelee** | roadmap v3 P6, doc#13 §5.2 | Deterministe, transparent, factuel, pas de ML, pas de subjectivite |
| D12 | Scope cuts Babel : pas de reviews, pas de task mock pour canari | 2026-05-18 | **Revisable** | doc#9 §1 | Reporter a S74, canari = reader + fixtures + provenance |
| D13 | Preview ephemere via daemon API, pas via Factory | 2026-05-19 | **Gelee** | doc#12 §2.2 | Primitive neutre, tout client peut previewer |
| D14 | Pas de signature Ed25519 decomposee en S67-S69 | 2026-05-19 | **Revisable** | doc#12 §2.3 | Le deploy monolithique via deploy-from-repo est suffisant et plus sur |
| D15 | SearchManifest = domain tag gele une fois deploye | 2026-05-19 | **Gelee** | doc#13 §9.2 | Meme rigueur que FeedEntryCanonical |
| D16 | formula_version dans ProofCard | 2026-05-19 | **Gelee** | doc#13 §9.2 | Apps detectent changement de formule sans casser |
| D18 | S70 = Process Portable Complete + Gate 1 dogfood | 2026-05-22 | **Gelee** | process_portable_complete_s70.md + roadmap v4 | Le process portable doit etre complet avant RRV total, Factory process packaging, SearchManifest ou broad OSS ingestion |

### 9.2 Tensions resolues

| Tension | Resolution | Source |
|---|---|---|
| FTS5 vs Tantivy | FTS5 d'abord (zero dep, tables existantes). Tantivy en gate post-S75 si >50K docs. | doc#5 §10, doc#4 §2.3 |
| Factory dans/hors daemon | Hors daemon (crate separe). Prior art convergent. | doc#12, doc#11 |
| @dev d'abord vs @protocole d'abord | @protocole d'abord (donnees existent, differenciateur, besoin pilote). Recadrage 2026-05-21 : @dev non bloquant Gate 1. Recadrage 2026-05-22 : S70 formalise le process portable ; `@dev` index devient S71+ par defaut. | doc#14 + PO 2026-05-21 + D18 |
| node_id dans/hors manifest | Hors manifest (Option D, attribution dans provenance) | doc#6 §6 |
| CuratorVouched en S65 vs S67 vs S70 | Minimal en S65 (variants + tests, pas d'UI). Full UI en S70. | doc#7 §5 Option D |
| Babel a la main vs via Factory | Via Factory (contrat). Plan B : a la main si Factory glisse. | doc#5 §1, doc#9 §7 |
| Factory monolithe daemon vs sidecar | Sidecar (sbfb-factory CLI). P7 revise. | doc#12 §3 |
| Ingestion gros repos OSS GitHub au lancement | Non pour S68-S69. Future extension `source-only`/`source-index` S70+, avec labels separes et sans confusion `verified SBFB app`. | PO 2026-05-21 |

---

## 10. Questions ouvertes

### 10.1 Questions consolidees et prioriees

Les 14 documents contenaient 50+ questions ouvertes. Apres deduplication
et resolution des questions tranchees par les decisions ci-dessus, il
reste :

| # | Question | Priorite | Sprint cible | Source |
|---|----------|----------|-------------|--------|
| Q1 | Factory dans workspace `nexus` pour MVP ou repo sibling immediat ? | **P1** | S67 | doc#5 §14 |
| Q2 | Le premier template doit-il etre static-minimal, static-storage, ou HTML pur ? | **P1** | S67 | doc#2 §14 |
| Q3 | Copier comme binaire externe ou via logique interne sbfb-factory ? | **P2** | S67 | doc#2 §14 |
| Q4 | Les embeddings @dev doivent-ils etre absents du MVP ou derriere Ollama local ? | **P2** | S70+ | doc#2 §14 |
| Q5 | Format exact factory.provenance.json : in-toto direct, SLSA-like minimal, ou format SBFB ? | **P2** | S67 | doc#2 §14 |
| Q6 | Quel niveau de confirmation utilisateur pour `deps.install` ? | **P2** | S68 | doc#2 §14 |
| Q7 | Le tag de perimetre RRV doit-il etre tape (`@Babel`) ou selectionne via UI ? | **P2** | S70 | doc#3 §11 |
| Q8 | `@web` doit-il etre disponible par defaut ou derriere un consentement ? | **P2** | S72+ | doc#3 §11 |
| Q9 | Les groupes prives doivent-ils utiliser le public feed avec chiffrement ou un feed separe ? | **P3** | S73+ | doc#3 §11 |
| Q10 | Comment publier volontairement un resultat prive vers le reseau public ? | **P3** | S73+ | doc#3 §11 |
| Q11 | Quel seuil de consentement GPU pour Babel en continu ? | **P3** | S74+ | doc#3 §11 |
| Q12 | `project_id` pour plusieurs apps d'un meme daemon — comment differencier ? | **P1** | S67 | doc#5 §14 |
| Q13 | Exigence de repo public pour le premier Babel canari ou mode local-only accepte avant push ? | **P1** | S69 | doc#5 §14 |
| Q14 | Page React `/factory` : page shell pure (Option A), Factory serveur local (Option B), ou CLI only (Option C) ? | **P1** | S68 | doc#12 §8 |
| Q15 | Faut-il un flux d'evenements daemon (WebSocket ou named pipe) pour les outils ? | **P2** | S67-S68 | doc#11 §7.2E |
| Q16 | Faut-il Tantivy ~0.22 ou ~0.23 (verifier MSRV 1.94 compat) ? | **P2** | S70+ | doc#4 §9 |
| Q17 | SearchManifest dans le feed : faut-il un nouveau FEED_FORMAT_VERSION ? | **P2** | S72 | doc#4 §9 |

### 10.2 Questions resolues (retirees des listes originales)

- "Attendre RRV complet avant Factory ?" -> **NON** (D5)
- "Factory dans ou hors daemon ?" -> **Hors daemon** (D2)
- "FTS5 ou Tantivy ?" -> **FTS5 d'abord** (D1)
- "node_id dans ou hors manifest ?" -> **Hors manifest** (D3)
- "@dev d'abord ou @protocole d'abord ?" -> **@protocole** (D6)
- "CuratorVouched quand ?" -> **Minimal en S65** (D9)

---

## 11. Sequence de travail recommandee

### Pre-requis : S66 Durabilite (en cours)

Gate : app archive + provenance + feed survivent aux restarts. Pas de
pilote externe tant que ce gate echoue.

### Phase 1 : Primitives daemon + @protocole FTS5 (S67)

```
Phase A : Primitives daemon neutres
  - sbfb-manifest crate (struct + validation + bridge allowlist)
  - CuratorVouched/CuratorDisendorsed dans PublicFeedOperation
  - GET /api/daemon/feed/entries (paginee, filtrable)
  - node_id optionnel dans deploy.rs
  - FTS5 daemon search (migration + search.rs + API)
  ~300-400 LOC daemon + ~200 LOC sbfb-manifest

Phase B : sbfb-factory crate + template engine + 2 templates
  ~300-400 LOC

Phase C : CLI sbfb-factory + diff + secret scanner
  ~200-300 LOC

Phase D : factory.template.lock + factory.provenance.json +
          factory.audit.jsonl + migration apps existantes v2
  ~150-200 LOC
```

### Phase 2 : Proof Cards + publish gate (S68)

```
Phase A : ProofCard data model + computation @protocole
  ~200 LOC coordinator/daemon

Phase B : publish gate + deploy dry-run + provenance checks
  ~300 LOC factory

Phase C : UX confiance / Proof Card display / bridge proof_card_get
  ~200-300 LOC shell/app

Phase D : preview/diff/audit log strictement si necessaire au publish
  ~200-300 LOC

Non-goal S68 : `@dev` tree-sitter, ingestion OSS GitHub, source-only
network seed, sauf stretch zero-impact.
```

### Phase 3 : Babel dogfood + pilote ferme (S69)

```
Phase A : Babel creee avec Factory par FlowUP, pas codee comme livrable
          agent autonome
  effort utilisateur + fixes infra selon retours

Phase B : Domain pack Babel + app babel-reader publiee via Factory
  ~500 LOC app HTML/JS + ~200 LOC fixtures

Phase C : Mecanisme invite + fix cross-node verification
  ~150 LOC

Phase D : Deploy verifie Babel + feed entry + proof pack minimal
  ~100 LOC

Phase E : Bilan pilote + go/no-go (documentation)
```

### Phase 4 : SearchManifest + Gouvernance UI (S70-S72)

```
S70 : Gouvernance Full UI (multi-curator, timeline, dissent, stale)
S71 : SearchManifest wire format + publication opt-in + gossip
S72 : Discovery + verification + anti-spam + privacy analysis
```

### Phase 5 : @web + Hardening (post-pilote, S73+)

```
S73 : Factory hardening + templates additionnels + 2eme app
S74 : Babel translation beta (task_submit + worker local)
S75 : Pack produit defendable + evidence pack + go/no-go public
```

(Consolide de doc#5 §10, doc#7 §10, doc#14 §5)

---

## 12. Prior art consolide

### 12.1 Synthese des 5 protocoles

| Protocole | Architecture | Recherche | Creation contenu | Extensibilite |
|---|---|---|---|---|
| **IPFS** | Daemon monolithique (Kubo : DHT + Bitswap + UnixFS + IPNS + Gateway) | Externe (ipfs-search.com crawle le DHT) | Generique (`ipfs add`) | API RPC HTTP, sidecar pattern (IPFS Cluster), Pinning Service API |
| **SSB** | Serveur + plugins (Kappa architecture : log append-only + vues) | Locale seulement (2-3 hops social graph) | Generique (`publish` JSON signe) | Plugins muxrpc (processus separes), types de messages libres |
| **AT Proto** | PDS + Relay + AppView + Feed Generators + Labelers | AppView (indexe firehose relay) | Generique (`repo.createRecord` Lexicon) | Lexicon schemas (namespace DNS), feed generators pluggables, labelers |
| **Radicle** | Noeud reseau + CLI + httpd sidecar | Basique (gossip annonces) | Generique (`git push` refs) | COBs extensibles, sidecar httpd, Radicle CI |
| **BitTorrent** | Protocole spec (BEPs) + implementations multiples | Externe (indexeurs, Bitmagnet crawl DHT) | Generique (torrent file / magnet link) | BEPs, LTEP (extension protocol), separation client/protocole naturelle |

(doc#11 §1-5)

### 12.2 Sept patterns communs

1. **Le noeud est un "tuyau stupide"** : il manipule des conteneurs
   adresses par contenu/identite, pas des objets semantiques.

2. **La recherche est TOUJOURS applicative** : aucun des 5 protocoles
   n'integre de recherche textuelle au niveau protocolaire.

3. **La creation de contenu est une primitive generique** : le protocole
   offre `write(bytes)`, l'application decide `write(what)`.

4. **Le pattern "sidecar" domine** : les extensions sont des processus
   separes qui communiquent via une API bien definie.

5. **Extensibilite du schema** : trois approches (schema libre SSB/IPFS,
   schema auto-descriptif AT Proto Lexicon, schema implicite Radicle
   COBs/BitTorrent BEPs).

6. **Le proxy preserve l'agence utilisateur** (AT Proto) : les clients
   ne parlent jamais directement aux AppViews.

7. **Evenements comme contrat d'integration** : les protocoles matures
   exposent un flux d'evenements (firehose AT Proto, socket Radicle,
   log SSB) que les outils consomment.

(doc#11 §6)

### 12.3 Anti-patterns documentes

1. **Patchwork trap (SSB)** : bundler serveur dans client en couplant la
   DB -> client inmaintenable. (doc#11 §7.4)

2. **Monolithe Kubo (IPFS)** : trop de couches dans le daemon -> lourd,
   difficile a remplacer. (doc#11 §7.4)

3. **Index sans evenements (BitTorrent)** : pas d'evenements natifs ->
   indexeurs doivent crawler activement (lent, couteux). (doc#11 §7.4)

4. **Schema trop libre (SSB)** : types JSON totalement libres ->
   fragmentation ecosysteme, clients incompatibles. (doc#11 §7.4)

### 12.4 Ce que chaque protocole apporte a SBFB

| Prior art | Pattern emprunte |
|---|---|
| AT Proto | Separation PDS/AppView/Feed Generator. Proxy. Lexicon -> namespace ops. |
| Radicle | Noeud aveugle (COBs = refs Git). Socket d'evenements. Sidecar httpd. |
| IPFS Cluster | Sidecar pattern : processus independant, API HTTP du daemon. |
| SSB | Plugins muxrpc. Kappa architecture (log = source verite, vues derivees). |
| BitTorrent | LTEP extension protocol. BEP governance. Indexeurs = services applicatifs. |

(doc#11 §7.3)

---

## 13. Risques consolides

Tous les risques identifies dans les 14 documents, dedupliques et priorises :

### 13.1 Risques P0 (bloqueurs)

| Risque | Source | Impact | Mitigation |
|---|---|---|---|
| Factory n'existe pas, tout le publish path (etapes 1-3) depend d'elle | doc#9 §7 | Babel canari impossible | Plan B : Babel a la main comme proto-canari si Factory glisse |
| Feed ReleasePublished non auto-insere dans deploy-from-repo | doc#9 §5 | Pas de trace feed des deploys | ~40 LOC dans deploy.rs, combler en S67-S68 |
| Persistence blobs volatile (MemStore) | doc#7 §2 | Apps disparaissent au restart | S66 OBLIGATOIRE avant S69 |
| node_id obligatoire dans SBFB.json | doc#6 §3 | Bloque templates portables | Fix Option D en S67 (~5 LOC) |

### 13.2 Risques P1 (hauts)

| Risque | Source | Impact | Mitigation |
|---|---|---|---|
| 3 sprints de code nouveau consecutif (S67-S69) sans hardening | doc#7 §6.1 | Fatigue, bugs imprevus | Phase D de chaque sprint inclut tests adversariaux |
| S67+S68 glissent | doc#9 §7 | Babel canari perd tout son sens | Decision PO : plan B explicit |
| Factory devient protocole metier | doc#2 §13 | Couplage long terme | Crate separe + primitives generiques |
| RRV = surface d'attaque injection | doc#4 §5.2 | Injection via champs indexes | Sanitizer (strip HTML, limit UTF-8, reject NUL bytes) |
| Manifest spoofing dans SearchManifest | doc#4 §5.2 | Faux projets dans les resultats | Signature Ed25519 + verification, PoW optionnel |
| Template engine plus complexe que prevu | doc#7 §9 | Retard S67 | Scope cut : 2 templates seulement |

### 13.3 Risques P2 (moyens)

| Risque | Source | Impact | Mitigation |
|---|---|---|---|
| DiffViewer React composant complexe | doc#7 §9 | Retard S68 | JSON expandable, pas diff inline texte |
| Preview sandbox surface d'attaque locale | doc#7 §9 | Exploitation locale | Memes CSP/COOP/COEP, hash ephemere, TTL court |
| Babel fixtures multilingues curation | doc#7 §9 | 2-3j supplementaires | 3 textes domaine public seulement |
| Privacy leak via SearchManifest | doc#4 §5.2 | Revele quels projets un noeud heberge | Opt-in explicite + documentation |
| Formule ProofCard contestee par communaute | doc#13 §9.3 | Changement d'API | formula_version + facteurs bruts |
| Tantivy version break | doc#4 §5.2 | Index inutilisable | Pin version, tester en CI |
| Gouvernance full UI retardee a S70 | doc#7 §6.3 | Trou de confiance au pilote | CuratorVouched minimal en S65 |

### 13.4 Risques P3 (faibles / long terme)

| Risque | Source | Impact | Mitigation |
|---|---|---|---|
| Inflation du daemon (trop de modules) | doc#13 §9.3 | Build time + surface d'attaque | Module search.rs isole, feature gate possible |
| Schema trop libre dans feed | doc#11 §7.4 | Fragmentation ecosysteme | Documenter schemas d'ops dans specs type BEP |
| .sbfb/apps devient repo | doc#2 §13 | Confusion source/runtime | Repo source dans Documents/Code |

---

## 14. Tests et preuves attendus

### 14.1 Tests Factory (consolides)

| Test | Pourquoi | Sprint | LOC |
|---|---|---|---|
| Template generation snapshot | Eviter drift template | S67 | ~30 |
| Generated repo `agentctl context` | Process portable | S67 | ~20 |
| Path traversal denied | Securite FS | S67 | ~20 |
| Shell command allowlist denied | Securite shell | S68 | ~20 |
| Diff preview required | Controle humain | S68 | ~30 |
| Lockfile change gate | Supply chain | S68 | ~20 |
| Preview iframe smoke | UX/runtime | S68 | ~30 |
| Provenance file hash stable | Preuve | S67 | ~20 |
| Archive > 10 MB rejected | Zip bomb/DDoS | S68 | ~20 |
| Archive > 500 files rejected | Zip bomb | S68 | ~20 |
| Bridge host rejects method not in manifest | Privilege escalation | S68 | ~30 |
| Factory deterministic same inputs same hash | Idempotence | S67 | ~30 |
| Factory provenance template_hash matches lockfile | Tracabilite | S67 | ~20 |
| Factory rejects binary exe in output | Securite | S68 | ~20 |

(doc#2 §11.1, doc#8 §8.4)

### 14.2 Tests RRV (consolides)

| Test | Pourquoi | Sprint |
|---|---|---|
| FTS query returns file/line | Citation minimale | S70 |
| Changed file reindexed | Index incremental | S70 |
| Deleted file removed | No stale local | S70 |
| Capability extracted | Mode dev utile | S70+ |
| Risk finding indexed | Audit | S70+ |
| @web off by default | Privacy | S72+ |
| @network unavailable label | Honest UX | S70 |
| Proof label preserved | No trust mixing | S70 |
| Score determinism (meme entrees = meme score) | Preuve | S71 |
| Projet sans provenance ne peut pas afficher score > 50 | Spoofing | S71 |
| Risk factor injection sanitized | Injection HTML | S71 |
| Stale detection via SourceBecameStale | Risk factor | S71 |
| SearchManifest sign/verify | Wire format | S72 |
| SearchManifest reject tampered | Securite | S72 |
| SearchManifest reject oversized | Anti-spam | S72 |
| SearchManifest rate limit 1/h/noeud | Anti-spam | S72 |

(doc#2 §11.2, doc#4 §6)

### 14.3 Tests Babel dogfood (consolides)

| Test | Pourquoi | Sprint |
|---|---|---|
| Source manifest loads | Provenance source | S69 |
| Reader opens fixture | UX de base | S69 |
| Storage saves progress | App storage | S69 |
| Task draft generated | Compute path | S74 |
| Review accept/reject | Validation humaine | S74 |
| Provenance graph visible | Trust UX | S69 |

(doc#2 §11.3)

### 14.4 Tests d'acceptance Babel (25 items)

Table complete en §5.3. Repartition par sprint :

- **S67** (Factory foundation) : #1-#8 = ~4j
- **S68** (Broker/preview) : #3, #4, #11, #12, #19 = ~3j
- **S69** (Babel canari) : #13-#18, #20 = ~4j
- **S70-S71** (post-canari) : #21-#25

(doc#9 §6)

---

---

## Annexe A. Struct Rust proposee pour SBFB.json v2

```rust
#[derive(Debug, Deserialize)]
struct SbfbJson {
    /// v1 compat: present dans les anciens JSON, ignore.
    #[serde(default)]
    node_id: Option<String>,

    /// Schema version. Absent ou 1 = v1 (ancien format). 2 = v2.
    #[serde(default = "default_schema_version")]
    schema_version: u32,

    /// App identifier. Obligatoire v2, absent v1.
    #[serde(default)]
    name: Option<String>,

    /// Semver. Optionnel v1, obligatoire v2.
    #[serde(default)]
    version: Option<String>,

    /// Display name for UI.
    #[serde(default)]
    display_name: Option<String>,

    /// Short description.
    #[serde(default)]
    description: Option<String>,

    /// Category tag.
    #[serde(default)]
    category: Option<String>,

    /// SPDX license identifier.
    #[serde(default)]
    license: Option<String>,

    /// Primary language (BCP-47).
    #[serde(default)]
    lang: Option<String>,

    /// Bridge configuration.
    #[serde(default)]
    bridge: Option<BridgeConfig>,

    /// Technology type.
    #[serde(default)]
    tech: Option<TechConfig>,

    /// Execution requirements.
    #[serde(default)]
    requirements: Option<RequirementsConfig>,
}

fn default_schema_version() -> u32 { 1 }

#[derive(Debug, Deserialize)]
struct BridgeConfig {
    #[serde(default)]
    methods: Vec<String>,
    #[serde(default)]
    events: Vec<String>,
    #[serde(default)]
    heartbeat: bool,
}

#[derive(Debug, Deserialize)]
struct TechConfig {
    #[serde(default)]
    r#type: Option<String>,
    #[serde(default)]
    build_command: Option<String>,
    #[serde(default = "default_entry_point")]
    entry_point: String,
}

fn default_entry_point() -> String { "index.html".to_string() }

#[derive(Debug, Deserialize)]
struct RequirementsConfig {
    #[serde(default)]
    min_bridge_version: Option<String>,
    #[serde(default)]
    offline_capable: bool,
    #[serde(default)]
    estimated_size_kb: Option<u32>,
}
```

**Validation deploy.rs v2 :**

```rust
fn validate_sbfb_json(sbfb: &SbfbJson, strict_v2: bool) -> Result<(), String> {
    if sbfb.schema_version == 2 || strict_v2 {
        if sbfb.name.is_none() || sbfb.name.as_deref() == Some("") {
            return Err("SBFB.json v2 requires non-empty 'name'".into());
        }
        if sbfb.version.is_none() || sbfb.version.as_deref() == Some("") {
            return Err("SBFB.json v2 requires non-empty 'version'".into());
        }
        if let Some(ref bridge) = sbfb.bridge {
            let valid_methods = [
                "task_submit", "storage_get", "storage_set", "storage_list",
                "storage_delete", "identity_pubkey", "node_status", "browse_list",
                "provenance_get", "provenance_verify", "pii_redact",
                "feed_cursor_get", "storage_version",
            ];
            for method in &bridge.methods {
                if !valid_methods.contains(&method.as_str()) {
                    return Err(format!("SBFB.json: unknown bridge method '{method}'"));
                }
            }
        }
    }
    if let Some(ref nid) = sbfb.node_id {
        tracing::debug!(node_id = %nid, "SBFB.json contains node_id (deprecated, ignored)");
    }
    Ok(())
}
```

**Compat descendante :**

| Cas | SBFB.json contenu | Resultat |
|---|---|---|
| App existante v1 | `{"node_id": "PLACEHOLDER", "name": "x", "version": "1.0.0"}` | Parse OK. schema_version defaut = 1, pas de validation stricte. |
| Nouvelle app v2 | `{"schema_version": 2, "name": "x", "version": "1.0.0"}` | Parse OK. Validation v2 passe. |
| JSON minimal v1 | `{"node_id": "abc"}` | Parse OK. Version 1, pas de validation. |
| JSON vide | `{}` | Parse OK (tout serde default). |

Aucun breaking change. (doc#6 §7)

---

## Annexe B. Pipeline d'indexation RRV

```
Source de donnees          Pipeline             Index
-----------------          --------             -----
BrowseAggregator     -->  SearchIndexer     --> FTS5 virtual table
  .aggregate()             .index_browse()      dans coordinator.db
                                               
FeedStore            -->  SearchIndexer
  .replay_all()            .index_feed()

ProvenanceRecord     -->  SearchIndexer
  DB query                 .index_provenance()

BlobServeCache       -->  SearchIndexer
  zip decompression        .index_app_content()
```

**Declenchement de la re-indexation :**

- Au boot du daemon (indexation complete)
- A chaque `ProjectAnnouncement` recu via gossip
- A chaque `deploy-from-repo` ou `publish-blob` reussi
- A chaque nouveau `FeedEntry` insere

L'indexation incrementale utilise le curseur feed
(meme pattern que `FeedMaterializer`). (doc#4 §4.1.3)

---

## Annexe C. Matrice de couverture Factory/RRV comme clients externes

Ce tableau resume si Factory et RRV peuvent etre implementes comme
clients 100% externes des primitives daemon actuelles. (doc#10 §5)

| Operation | Primitives disponibles | Verdict |
|---|---|---|
| Creer une app depuis un template | Aucune primitive | Factory genere localement puis appelle `deploy-from-repo` |
| Deployer une app | `deploy`, `deploy-from-repo`, `publish`, `publish-blob` | OK |
| Lire le registre des apps | `browse`, `apps`, `apps/{id}` | OK |
| Lire le feed d'evenements | **MANQUE** (`feed/entries`) | **BLOQUANT** |
| Inserer un evenement dans le feed | `feed/insert` (avec guard interne) | OK mais guard `X-SBFB-Feed-Internal` limitant |
| Verifier provenance | `project/{id}/provenance` | OK pour 1 projet, manque batch |
| Verifier contributeur | `contributor/verify`, `contributor/project` | OK |
| Recherche full-text | **MANQUE** (`search`) | **BLOQUANT pour RRV** |
| Synchroniser feed P2P | `feed/ticket`, `feed/join` | OK |
| Synchroniser storage P2P | `storage/ticket`, `storage/join` | OK |
| Preview sandbox | `blob-serve` + preview/load (a creer) | Partiellement couvert |
| Lire metadata app (SBFB.json) | **MANQUE** (`manifest`) | Contournable mais penible |

Avec l'ajout de 2 primitives P0 (feed read + search index), Factory et
RRV peuvent etre des clients externes a 95%. Le 5% restant est le
template engine Factory (choix de commodite). (doc#10 §5)

---

## Annexe D. CuratorVouched / CuratorDisendorsed dans le feed

**Nouvelles variantes dans `PublicFeedOperation` :**

```rust
pub enum PublicFeedOperation {
    ReleasePublished(ReleasePublishedPayload),
    SourceBecameStale(SourceBecameStalePayload),
    CuratorVouched(CuratorVouchedPayload),          // NOUVEAU
    CuratorDisendorsed(CuratorDisendorsedPayload),   // NOUVEAU
}

pub struct CuratorVouchedPayload {
    pub project_id: String,     // hex-64
    pub curator_pubkey: String, // hex-64
    pub scope: String,          // max 280 chars
    pub comment: Option<String>, // max 280 chars
}

pub struct CuratorDisendorsedPayload {
    pub project_id: String,
    pub curator_pubkey: String,
    pub reason: String,
    pub comment: Option<String>,
}
```

`FEED_FORMAT_VERSION` reste a 1 (raw-op extensible). Les noeuds anciens
(sans ces types) stockent et propagent les ops inconnues. (doc#12 §2.4)

---

## Annexe E. Estimation effort S67-S69

### Effort consolide (doc#7 §7)

| Sprint | Phases | LOC Rust | LOC TS/HTML | Tests delta | Risque |
|---|---|---|---|---|---|
| S67 (Factory Foundation) | 4 | ~770 | ~20 | +24-32 | 2/5 |
| S68 (Broker/Preview) | 4 | ~610 | ~650 | +28-38 | 3/5 |
| S69 (Babel Canari) | 5 | ~190 | ~990 | +17-24 | 4/5 |
| **Total** | **13** | **~1570** | **~1660** | **+69-94** | -- |

**Total : ~3230 LOC sur 3 sprints, +69-94 tests.**

Comparaison avec l'ancien plan (Gouvernance+Proof Pack+Pilote) :
~1900 LOC. Le pivot ajoute ~1330 LOC (+70%), principalement l'UI
React /factory (~570 LOC) et l'app Babel (~500 LOC).

### Prerequisits avant S67 (doc#7 §11)

- [ ] P2-FEED-INSERT-NO-AUTH-TIER fixe (S65 Phase A)
- [ ] CuratorVouched / CuratorDisendorsed dans le feed (S65 Phase A)
- [ ] TRUST_TAXONOMY.md ecrit et applique dans l'UI (S65 Phase B)
- [ ] iroh data_dir cable dans le daemon (S66 Phase A)
- [ ] iroh-blobs FsStore operationnel (S66 Phase B)
- [ ] Feed republish au boot (S66 Phase C)
- [ ] E2E restart test vert (S66 Phase E)

Si un de ces items echoue, S67 NE DOIT PAS demarrer.

---

## Annexe F. Evenements daemon (contrat d'integration futur)

Le prior art (doc#11 §6.7) montre que les protocoles matures exposent un
flux d'evenements pour les outils applicatifs. Le daemon SBFB devrait a
terme exposer (WebSocket ou named pipe) :

| Evenement | Donnees | Consommateur |
|---|---|---|
| `blob_added` | hash, size | RRV (indexation incrementale) |
| `feed_entry_inserted` | entry_id, op_type, author | RRV, Factory (suivi deploys) |
| `peer_connected` | node_id | Diagnostic |
| `provenance_verified` | hash, result | RRV (Proof Cards) |
| `project_announced` | project_id, project_name | RRV (indexation), Browse |
| `curator_list_updated` | curator_pubkey, revision | RRV, Browse |

Ce flux serait le contrat d'integration entre le daemon neutre et les
outils specialises. Non planifie avant S68+ mais documente ici pour
reference. (doc#11 §7.2E)

---

## Annexe G. Routes W existantes — verdict externalisation

| Route W | Verdict | Rationale |
|---|---|---|
| Tasks submit/results | Rester (pour l'instant) | Coeur du reseau de compute |
| Kudos/reputation | Rester | Lie au protocole de confiance |
| GPU Consent | Externaliser a terme | Specifique au worker binaire |
| Worker state | Externaliser a terme | Lecture d'un fichier d'un autre binaire |
| Invites | Rester | Encode un doc ticket iroh-docs, daemon seul a minter |
| Quarantine | Rester | Defense gossip, lie au daemon |

(doc#10 §4.4)

---

*Fin de synthese. Ce document consolide 14 fichiers de recherche totalisant
~15 000 lignes en une reference unique. Chaque fait est tracable a son
document source via les references (doc#N §X).*
