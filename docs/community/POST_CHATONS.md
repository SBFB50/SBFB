# Post forum CHATONS — SBFB distribution d'apps web P2P

**Categorie recommandee** : Cafe du commerce (introduction informelle)
**Titre** : SBFB — distribution d'apps web en P2P, source verifiable, sans serveur central (AGPL-3.0)

---

Salut a toutes et tous,

Je m'appelle Theophile, dev solo sur un protocole libre (AGPL-3.0) qui
pourrait interesser les CHATONS. Je cherche pas de financement — je
cherche des retours terrain de gens qui comprennent l'hebergement
alternatif.

## Le probleme

Aujourd'hui, pour distribuer une app web, il faut passer par un
hebergeur centralise (Vercel, Netlify, GitHub Pages) ou par un app
store (Google Play qui exige maintenant ta piece d'identite depuis
mars 2026, Apple qui broie les indie devs). Si le serveur tombe, l'app
disparait. Si le store refuse, elle n'existe plus.

F-Droid a resolu ce probleme pour les apps Android. SBFB fait la meme
chose pour les apps web.

## Ce que c'est

SBFB est un protocole P2P (ecrit en Rust, base sur iroh) ou n'importe
qui publie une app web (HTML/JS/CSS, React, Python/Pyodide, WASM,
notebook Jupyter...). Le reseau la distribue. Les clients la rendent
dans un iframe sandbox. Pas de serveur central, pas d'admin, pas de
store.

```
Developpeur                     Reseau P2P                   Utilisateur
     |                               |                            |
     |  sbfb-factory create myapp    |                            |
     |  -> scaffold SBFB.json + HTML |                            |
     |                               |                            |
     |  sbfb-factory publish         |                            |
     |  -> clone repo git            |                            |
     |  -> zip l'archive             |                            |
     |  -> signe Ed25519 (SLSA L1)   |                            |
     |  -> broadcast gossip -------> | annonce P2P propagee       |
     |  -> upload iroh-blobs ------> | blob distribue             |
     |                               |                            |
     |                               | <-- daemon local subscribe |
     |                               | -> telecharge le blob      |
     |                               | -> decompresse (LRU cache) |
     |                               | -> iframe sandbox -------> | app affichee
```

## Architecture technique

### Protocole P2P (iroh 0.98)

Le daemon Rust (`nexus-shell-daemon`, ~21 Mo) fait tourner un noeud
iroh complet :

- **Gossip** (iroh-gossip) : propagation temps reel des annonces
  d'apps (`ProjectAnnouncement`) via topics P2P
- **Blobs** (iroh-blobs) : distribution des archives zip en
  content-addressed (hash BLAKE3). Comme BitTorrent mais avec des apps.
- **Docs** (iroh-docs) : stockage replique CRDT pour les donnees
  des apps (votes, contenus, preferences). Synchronisation P2P
  offline-first — si le reseau se partitionne, chaque partition
  continue de fonctionner, les donnees fusionnent a la reconnexion.
- **Discovery** : DHT Pkarr + DNS TXT fallback + relais WebSocket.
  Fonctionne derriere un NAT, pas besoin d'IP publique.

### Crypto (standards, pas blockchain)

| Primitif | Usage |
|----------|-------|
| **Ed25519** (RFC 8032) | Signatures : provenance apps, curators, feed public, kudos |
| **BLAKE3** | Hash des archives (content-addressed) + chaines de hash |
| **ChaCha20-Poly1305** | Chiffrement transport (QUIC via iroh) |
| **Hashcash** (16-bit) | Anti-spam optionnel sur le gossip |

Pas de RSA, pas de TLS certificate a renouveler, pas de wallet.

Chaque message signe utilise un **domain separation tag** (RFC 8785
JCS) pour empecher les attaques cross-domain : une signature sur une
tache ne peut pas etre rejouee comme une annonce de projet. 14 domaines
definis (`DOMAIN_TASK_V1`, `DOMAIN_CURATOR_LIST_V1`,
`DOMAIN_PROVENANCE_V1`, etc.).

### Provenance verifiable (SLSA L1)

Chaque app publiee genere un `provenance.json` :

```json
{
  "repo_url": "https://codeberg.org/alice/mon-app",
  "commit_sha": "a1b2c3d4...",
  "artifact_hash": "blake3:c0ffee...",
  "node_id": "ed25519:cle_du_noeud...",
  "signature": "ed25519_sig:...",
  "timestamp": "2026-05-26T10:00:00Z",
  "schema_version": 1
}
```

N'importe qui peut verifier : `git ls-remote <repo> <commit>` = valide,
signature Ed25519 = valide, hash BLAKE3 de l'archive = correspond. Auto-
attestation — le noeud qui publie signe la preuve. Pas une autorite
centrale qui valide, mais une preuve cryptographique verifiable par
tous.

## Curators au lieu de moderation

Pas de moderateur central. A la place, des **listes de curators**
signees Ed25519, propagees par gossip :

- Chaque CHATONS pourrait etre un curator pour sa communaute
- Vous signez une liste d'apps que vous recommandez (max 256 entries)
- Les utilisateurs s'abonnent aux curators qu'ils choisissent
- Un curator peut endorser (`CuratorVouched`) ou deconseiller
  (`CuratorDisendorsed`) une app
- Les listes ont un compteur de revision monotone (anti-rollback)
- Personne ne peut **supprimer** une app du reseau — juste changer
  sa visibilite dans sa liste
- Support rotation de cle Ed25519 (fenetre de transition 14 jours max)

C'est le modele F-Droid, mais distribue : au lieu d'une seule
fondation qui signe les APK, N curators independants signent leurs
recommandations.

Niveaux de confiance (Trust Taxonomy N0-N5) :
- N0 : public (annonce gossip, pas examine)
- N1 : source declaree (repo_url present)
- N2 : deploy verifie (provenance Ed25519 signee)
- N3 : signature verifiee live par le daemon
- N4+ : build reproductible (futur)

## Sandbox strict — 5 couches d'isolation

L'app tourne dans un iframe avec des protections cumulatives :

1. **iframe sandbox** : `sandbox="allow-scripts"` sans
   `allow-same-origin` — l'app ne voit pas le DOM parent, pas de
   cookies, pas de localStorage du shell
2. **CSP strict** : `connect-src 'none'` — **zero requete reseau
   sortante** (pas de fetch, pas de WebSocket, pas de XMLHttpRequest)
3. **Origin separee** : blob-serve tourne sur un port different du
   daemon — isolation cookies/storage cross-origin
4. **Bridge postMessage** : seule voie de communication, avec
   correlation UUID + timeout 10s + heartbeat 1s (watchdog CPU)
5. **Loopback authentifie** : bearer 256-bit + Host allowlist +
   Origin check + peer creds (UDS SO_PEERCRED sur Linux, Named Pipe
   DACL sur Windows)

Une app malveillante ne peut ni lire vos fichiers, ni faire de
requetes reseau, ni acceder aux donnees d'une autre app, ni voler
le token du daemon.

## Le bridge — 17 methodes pour les apps

Les apps utilisent un SDK JavaScript (`sbfb-bridge.js`, 423 lignes)
avec des methodes whitelistees :

```javascript
const bridge = new SBFBBridge({timeout: 10000, heartbeatInterval: 1000});

// Storage P2P (CRDT iroh-docs, synchro entre noeuds)
await bridge.setStorage("mon-vote", {choix: "A"});
const data = await bridge.getStorage("mon-vote");
const all  = await bridge.listStorage("ideas/");
await bridge.deleteStorage("old-key");

// Introspection reseau
const status = await bridge.getNodeStatus();    // uptime, pairs
const apps   = await bridge.getBrowseList();    // apps disponibles
const proof  = await bridge.verifyRelease(id);  // verifier provenance
const card   = await bridge.getProofCard(id);   // score qualite 0-100
const cursor = await bridge.getPublicFeedCursor(); // feed position

// Recherche full-text (FTS5)
const results = await bridge.search("dark theme", {limit: 20});

// Identite
const me = await bridge.getIdentityPubkey();    // Ed25519 pubkey

// Compute distribue (optionnel, GPU consent explicite)
await bridge.submitTask({prompt: "analyse ce texte"});

// Evenements push (daemon -> app)
bridge.onStorageUpdate("ideas-app", (version) => { reload(); });
```

Les donnees des apps sont stockees en iroh-docs (CRDT last-write-wins)
— synchronisation P2P entre noeuds, offline-first. Pas de base de
donnees centrale.

## Reputation et vote — sans monnaie

### Kudos (reputation compute)

Quand un noeud contribue du calcul GPU au reseau, il recoit des points
de reputation. Pas une monnaie — pas de transfert, pas de marche, pas
de speculation.

- **Rendements decroissants** : `amount = floor(1000 * log2(1 + tokens))`.
  1 token = 1000 kudos, 1000 tokens = 10 000 kudos (pas 1M). Empeche
  les gros workers de dominer.
- **Decroissance temporelle** : EMA decay alpha=0.97 (demi-vie ~23
  jours). Les contributions recentes comptent plus.
- **Chaine de hash** : chaque entree Kudos est signee Ed25519 et
  chainee par BLAKE3 (`prev_hash -> entry_hash`). N'importe qui peut
  auditer le ledger complet. Impossible de revenir en arriere.
- **Metriques fairness** : coefficient de Gini (0=egal, 1=monopole),
  top-K share, churn rate. Alerte si Gini > 0.70.
- **API transparente** : `GET /api/v1/kudos/{project}/leaderboard` +
  `GET /api/v1/diagnostic/fairness` (Gini + top-K en temps reel)

### Vote P2P (apps)

L'app Ideas Hub montre comment faire du vote decentralise sans serveur.
Chaque vote est une cle `votes/{ideaId}/{pubkey_votant}` dans le
storage P2P. 1 identite Ed25519 = 1 vote. Re-cliquer retire le vote.
Les votes se synchronisent entre noeuds automatiquement.

Ce modele de vote est generique — n'importe quelle app SBFB peut
l'utiliser pour du budget participatif, des sondages communautaires,
ou du peer-review.

## Factory — pipeline de publication en 11 gates

L'outil CLI `sbfb-factory` guide la publication :

| Gate | Verification | Bloquant | Local/Reseau |
|------|-------------|----------|-------------|
| FG0 | Classification (static-html, react, pyodide, wasm) | Non | Local |
| FG1 | Scope bridge (quelles methodes l'app demande) | Non | Local |
| FG2 | Template scaffold (index.html + SBFB.json + SDK) | Non | Local |
| FG3 | Manifest SBFB.json v2 schema validation | Oui | Local |
| FG4 | Diff des fichiers modifies vs template | Non | Local |
| FG5 | Sandbox : symlinks, path traversal, Windows backslash | Oui | Local |
| FG6 | Secrets : regex AWS AKIA, ghp_, PEM, API keys | Oui | Local |
| FG7 | Preview : test live en iframe sur localhost (30 min TTL) | Non | Local |
| FG8 | Provenance : clone repo, signe Ed25519, SLSA L1 | Oui | Local |
| FG9 | Publish : broadcast gossip + upload iroh-blobs | - | P2P |
| FG10 | Review : curators endorsent/deconseillent (asynchrone) | Non | P2P |

FG0 a FG7 fonctionnent **hors-ligne**. La publication (FG9) est
immediate — pas d'attente de review. Le review curator (FG10) est
asynchrone et volontaire.

## Manifest SBFB.json v2

```json
{
  "schema_version": 2,
  "name": "sbfb-ideas",
  "display_name": "Ideas Hub",
  "description": "Proposez et votez pour des idees",
  "category": "social",
  "license": "AGPL-3.0-or-later",
  "lang": "fr",
  "bridge": {
    "methods": ["storage_get", "storage_set", "storage_list",
                "storage_delete", "identity_pubkey"]
  },
  "tech": {
    "type": "static-html",
    "build_command": null
  }
}
```

Parser forward-compatible : un manifest v1 ou v3+ est accepte, les
champs inconnus sont ignores.

## Compute distribue (optionnel)

Un noeud SBFB peut contribuer du compute GPU au reseau (Ollama pour
l'inference LLM). Le consentement est **explicite a 4 niveaux** :

- L1 : mes projets uniquement
- L2 : projets open source verifies
- L3 : whitelist manuelle (projet par projet)
- L4 : tous les projets

Avec des caps par projet : watts max, VRAM max, heures max. Le worker
refuse automatiquement les taches hors-scope du consentement.

Les resultats sont valides par le coordinateur (quorum SHA256 si
plusieurs workers), les prompts/resultats sont filtrables (PII
redaction via GLiNER ONNX + regex), et les taches suspectes vont en
quarantaine.

## Les 3 apps exemples

### Protocol Explorer (`examples/sbfb-explorer/`)

Documentation interactive du protocole en 6 sections : architecture,
cycle de vie app, cycle de vie tache, securite, verification &
provenance, philosophie. Panneau live "Statut reseau" qui interroge le
daemon via bridge (node_status, identity_pubkey, browse_list). Demo
interactive de verification de provenance (verifyRelease).

HTML/JS pur, 0 dependance npm, 5 fichiers.

### Ideas Hub (`examples/sbfb-ideas/`)

App de vote decentralise : proposer des idees (titre + description),
voter (1 vote par identite Ed25519, toggle), trier par votes ou
date. Donnees stockees en storage P2P via bridge (storage_get/set/
list/delete). Synchronisation automatique entre noeuds.

Structure des donnees :
- `ideas/{uuid}` → `{title, description, author: pubkey, created_at}`
- `votes/{ideaId}/{voterPubkey}` → `{timestamp}`

HTML/JS pur, 0 dependance npm, 3 fichiers.

### Factory Viewer (`examples/sbfb-factory-viewer/`)

App sandboxee qui affiche les apps du reseau via bridge (browse_list,
search, proof_card_get). Grille d'apps avec Proof Cards. Dark theme
SBFB. Lecture seule — aucun endpoint Operator, aucun localhost/token.

## Differences avec ce que vous connaissez

| | Yunohost | PeerTube | IPFS/Fleek | F-Droid | SBFB |
|---|---|---|---|---|---|
| **Hebergement** | VPS obligatoire | Instance serveur | Pinning nodes | APK signes centralement | P2P natif, zero serveur |
| **Type contenu** | Apps systeme | Video | Fichiers statiques | Apps Android | Apps web universelles |
| **Catalogue** | Serveur central | ActivityPub federe | Pas de catalogue | Store centralise | Gossip P2P + curators Ed25519 |
| **Provenance** | GPG optionnel | Limitee | Aucune | APK signature | Ed25519 + SLSA L1 automatique |
| **Offline** | Non | Partiel | Si pinne | Non | Oui (CRDT iroh-docs) |
| **Isolation** | OS-level (systemd) | N/A | N/A | Sandbox Android | iframe CSP strict 5 couches |
| **GPU compute** | Non | Non | Non | Non | Oui (consent 4 niveaux) |
| **Donnees apps** | PostgreSQL centralise | N/A | Immutable | N/A | CRDT P2P (last-write-wins) |
| **Vote/reputation** | Non | Non | Non | Non | Kudos hash-chain + vote P2P |

### Precision Yunohost

Yunohost exige un point d'entree stable (DNS + VPS). SBFB peut tourner
sur un Raspberry Pi derriere un NAT. La decouverte utilise Pkarr DHT
avec fallback DNS TXT + WebSocket. Si le catalogue Yunohost tombe, zero
nouvelles installs. SBFB : la decouverte est decentralisee par
construction (3 niveaux fallback).

### Precision F-Droid

F-Droid a une equipe de review humaine et une seule cle de signature.
SBFB : zero moderation centralisee. Les gates FG0-FG9 sont locales et
automatiques. La gate FG10 (review curator) est asynchrone et
volontaire. Une app publiee est live immediatement, pas en attente.

## Le shell React (ce que voit l'utilisateur)

6 pages lazy-loaded (React 19 + TypeScript + Tailwind + shadcn/ui +
Zustand + React Query) :

- **Browse** : grille glassmorphism des apps du reseau, statut
  reachable/unreachable, badges source (direct/P2P/verified)
- **BrowsedProject** : iframe full-screen immersif avec barre auto-
  masquee, VerificationDetail (provenance), ProofCard (score 0-100),
  watchdog CPU (heartbeat 1s)
- **Curators** : abonnement/desabonnement aux listes Ed25519, revision
  counter, count apps
- **Network** : etat live du worker (GPU snapshot, VRAM, temperature,
  projets servis, kudos gagnes, consent L1-L4)
- **Projects** : coordinateurs locaux enregistres
- **Deploy** : formulaire deploy-from-repo (URL git, nom, description)

## Installation pour un hebergeur

Un binaire Rust (~21 Mo), zero config obligatoire :

```bash
# Telecharger et lancer
./nexus-shell-daemon
# -> ouvre le navigateur, cree ~/.sbfb/ avec identite Ed25519 + token
# -> ecoute sur 127.0.0.1 (loopback uniquement)
# -> consomme ~150 Mo RAM en idle
```

Pour un service systemd :

```bash
sudo useradd -m -s /bin/false sbfb
sudo cp nexus-shell-daemon /usr/local/bin/
# systemd unit: User=sbfb, WorkingDirectory=/var/lib/sbfb,
# CapabilityBoundingSet=CAP_NET_BIND_SERVICE CAP_NET_RAW
sudo systemctl enable sbfb-daemon && sudo systemctl start sbfb-daemon
```

Aucun port public expose — tout passe par loopback (127.0.0.1). Pas de
surface d'attaque reseau. Pas de TLS certificate a renouveler.

## Securite — ce qu'on garantit et ce qu'on ne garantit pas

### Garanti v1.0

- Keypair Ed25519 en fichier perm 0600
- Loopback bearer 256-bit + Host/Origin check + UDS peer creds
- Sandbox iframe 5 couches (CSP, COOP, COEP)
- Feed public append-only (chaine de hash per-author)
- Kudos ledger verifiable (BLAKE3 hash chain + Ed25519)
- Consent GPU explicite 4 niveaux + caps W/VRAM/h
- Anti-spam Hashcash 16-bit sur gossip
- Sybil resistance couche 1 (age witness 7 jours min)

### Pas encore garanti (risques residuels documentes)

- Keypair non chiffree au repos (OS perm 0600 seul — Keychain/DPAPI
  prevu post-v1.0)
- Pas de cargo-audit en CI (carry post-v1.0)
- Rate limit absent sur deploy-from-repo (bearer token suffit pour le
  pilote ferme)
- Audit iroh upstream pas fait (R-iroh-audit P0, bloquant pour
  distribution publique)

## L'etat du projet

- 70 sprints, ~1800 tests (1486 Rust, 279 Vitest, 6/6 size-limit)
- Installeurs Windows NSIS, Linux .deb, macOS .dmg
- P2P valide : LAN Win<->Mac, WAN dev<->VPS Helsinki
- Licence AGPL-3.0 (le code reste libre, meme en SaaS)
- Solo maintainer, modele OpenBSD — pas de startup, pas de fondation
- Tag v1.0 pose localement, pas encore pousse vers origin

## Factory Operator (outil de gestion local)

En plus du CLI, un outil graphique local (Vite + React + Tailwind +
shadcn/ui) connecte au daemon :

- **Sprint Overview** : etat du sprint, phases, verdicts
- **Sprint History** : historique complet de tous les sprints (67
  sprints, diff inline par fichier, commits detailles, delta tests
  par phase, carries, scope cuts) via endpoint JSON `/api/sprint-
  history` qui retourne ~48 Ko de donnees structurees
- **Agent Chat** : discussion avec context project
- **Phase Assistant** : guide preflight/review
- **Lint Planning** : validation artefacts
- **Commit Auditor** : audit commit body
- **Agent Transfer** : handoff inter-provider
- **Context Pack Builder** : packing contextuel
- **Action Center** : commandes Factory allowlistees
- **Action Log** : journal des actions

11 pages, i18n FR/EN (~170 cles), dark theme SBFB.

## Ce que je cherche

Un ou deux CHATONS motives pour tester ensemble. L'idee :

1. Installer un noeud SBFB a cote de vos services existants
2. Publier 2-3 petites apps utiles a votre communaute
3. Voir si le P2P tient, si le bridge est pratique, si le modele
   curator fonctionne
4. Me remonter les problemes

Je ne cherche pas de financement. Je cherche la preuve terrain qu'un
reseau d'apps web P2P source-verifiable peut fonctionner en conditions
reelles, avec des gens qui comprennent l'infrastructure alternative.

Le code est sur [lien repo]. Dispo pour une demo en visio ou pour
repondre ici.

Bonne journee a tous

---

## Annexe technique — endpoints daemon (pour les curieux)

70+ routes HTTP loopback, voici les principales :

**Curators** : GET /api/daemon/curators, POST subscribe, DELETE unsubscribe
**Browse** : GET /api/daemon/browse, POST browse/pull
**Publish** : POST /api/daemon/publish, POST publish-blob
**Feed** : GET feed/entries (pagine), POST feed/join, GET feed/cursor
**Search** : GET /api/daemon/search (FTS5 full-text, q + limit + offset)
**Proof Card** : GET /api/daemon/proof-card/{id} (score 0-100, 7 risk factors)
**Provenance** : GET /api/v1/project/{id}/provenance (chaine Ed25519)
**Storage** : GET/POST/DELETE /app/{name}/state/{key} (CRDT P2P)
**Kudos** : GET /api/v1/kudos/{project}/leaderboard, GET fairness
**Tasks** : POST /api/v1/tasks/submit, GET tasks, GET tasks/{id}
**Worker** : GET /api/v1/worker/state, consent GET/POST
**Canary** : FROST DKG (round1/round2/aggregate), network-health
**Diagnostics** : neighborhood, fairness Gini + top-K
