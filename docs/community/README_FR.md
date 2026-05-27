# SBFB — reseau P2P d'apps web verifiables

Un protocole libre (AGPL-3.0) pour publier, distribuer et verifier
des applications web entre pairs. Pas de serveur central, pas de
store, pas de compte cloud. Le code source est verifiable, les
recommandations sont communautaires, le calcul IA/GPU est partage
sur consentement.

## Ce que c'est

SBFB permet a n'importe qui de publier une application web sous
forme d'archive verifiable, puis de la diffuser sur un reseau P2P.
Les utilisateurs l'ouvrent dans leur navigateur, dans un iframe
sandboxe. La provenance de chaque app est signee (Ed25519, SLSA L1)
et verifiable par tous les noeuds du reseau. Des listes de curators
signees remplacent la moderation centralisee : chaque communaute
recommande ou deconseille les apps qu'elle a testees.

Le deuxieme axe est le partage de puissance IA/GPU. Une app SBFB
peut demander, avec consentement explicite, des taches de traduction,
d'analyse ou de generation a des workers GPU/CPU volontaires. La
puissance de calcul ne depend plus de quelques plateformes cloud.

## Architecture

```
Developpeur                     Reseau P2P                   Utilisateur
     |                               |                            |
     |  sbfb-factory create myapp    |                            |
     |  -> SBFB.json + index.html    |                            |
     |                               |                            |
     |  sbfb-factory publish         |                            |
     |  -> clone repo git            |                            |
     |  -> zip archive               |                            |
     |  -> signe Ed25519 (SLSA L1)   |                            |
     |  -> broadcast gossip -------> | annonce propagee           |
     |  -> upload iroh-blobs ------> | blob distribue             |
     |                               |                            |
     |                               | <-- daemon local subscribe |
     |                               | -> telecharge le blob      |
     |                               | -> verifie hash + signature|
     |                               | -> iframe sandbox -------> | app affichee
     |                               |                            |
     |                               | curators: vouch/disendorse |
     |                               | -> listes signees Ed25519  |
```

## Fonctionnalites

- Publication d'apps web depuis un depot source (Git multi-forge)
- Provenance signee Ed25519 + SLSA L1 automatique
- Distribution P2P via iroh (gossip + blobs content-addressed BLAKE3)
- Donnees applicatives synchronisees entre noeuds (CRDT iroh-docs, offline-first)
- Sandbox iframe 5 couches (CSP strict, zero requete reseau sortante)
- Bridge postMessage avec 17 methodes whitelistees (SDK `sbfb-bridge.js`)
- Curators Ed25519 : listes signees de recommandations, sans moderateur central
- Recherche locale full-text (FTS5)
- Proof Cards : score de preuve 0-100 par app, facteurs de risque
- Feed public append-only verifie par hash-chain BLAKE3
- Compute GPU distribue opt-in (Ollama + llama.cpp embarque)
- Consentement GPU explicite a 4 niveaux avec caps watts/VRAM/heures
- Reputation kudos non-monetaire (rendement logarithmique, decroissance EMA)
- Metriques fairness en temps reel (Gini, top-K, churn rate)
- Factory CLI : pipeline de publication en 11 gates (7 hors-ligne)

## Comment ca marche

1. Un developpeur cree une app (`sbfb-factory create`) avec un
   manifeste `SBFB.json` et un `index.html`.
2. Il publie (`sbfb-factory publish`) : le daemon clone le repo,
   zip l'archive, signe la provenance Ed25519 et diffuse l'annonce
   en P2P via gossip.
3. Les noeuds du reseau recoivent l'annonce, telecharge le blob
   (iroh-blobs, content-addressed BLAKE3) et le mettent en cache.
4. L'utilisateur ouvre l'app dans son navigateur : le daemon local
   sert l'archive dans un iframe sandbox, avec un bridge
   postMessage comme seul canal vers le reseau.
5. Des curators (associations, CHATONS, collectifs) signent des
   listes de recommandations. L'utilisateur choisit les curators
   qu'il suit. Personne ne peut supprimer une app du reseau.

## Demarrage rapide

```bash
# Telecharger le daemon et lancer
./nexus-shell-daemon
# -> cree ~/.sbfb/ (identite Ed25519 + token)
# -> ouvre le navigateur
# -> ecoute sur 127.0.0.1 (loopback uniquement)
# -> ~150 Mo RAM en idle

# Parcourir les apps disponibles
# -> page Browse dans le navigateur
```

Aucun port public expose. Pas de certificat TLS a renouveler. Le
daemon tourne derriere un NAT sans configuration (decouverte via
Pkarr DHT + DNS TXT fallback + relais WebSocket).

## Publier une app

```bash
# Creer le squelette
sbfb-factory create mon-app

# Modifier index.html, ajouter le SDK bridge
# <script src="/sbfb-bridge.js"></script>

# Valider le manifeste et scanner les secrets
sbfb-factory validate mon-app
sbfb-factory scan-secrets mon-app

# Publier sur le reseau (clone + signature + broadcast)
sbfb-factory publish mon-app
```

Les gates FG0 a FG7 fonctionnent hors-ligne. La publication (FG9)
est immediate. La review curator (FG10) est asynchrone et volontaire.

## Apps exemples

### Protocol Explorer (`examples/sbfb-explorer/`)

Documentation interactive du protocole en 6 sections : architecture,
cycle de vie des apps, cycle de vie des taches, securite,
verification et provenance, philosophie. Panneau live qui interroge
le daemon via bridge. Demo de verification de provenance. HTML/JS
pur, 0 dependance npm.

### Ideas Hub (`examples/sbfb-ideas/`)

App de vote decentralise : proposer des idees, voter (1 vote par
identite Ed25519, toggle), trier par votes ou date. Donnees stockees
en storage P2P via bridge. Synchronisation automatique entre noeuds.
HTML/JS pur, 0 dependance npm.

### Factory Viewer (`examples/sbfb-factory-viewer/`)

Grille des apps du reseau avec Proof Cards, recherche full-text et
dark theme SBFB. Lecture seule, aucun acces privilegie. Utilise le
bridge pour `browse_list`, `search` et `proof_card_get`.

## SDK Bridge

Les apps communiquent avec le reseau via `sbfb-bridge.js` (423
lignes, inclus dans l'archive) :

```javascript
const bridge = new SBFBBridge({ timeout: 10000, heartbeatInterval: 1000 });

// Stockage P2P (CRDT iroh-docs, synchro entre noeuds)
await bridge.setStorage("mon-vote", { choix: "A" });
const data = await bridge.getStorage("mon-vote");
const all  = await bridge.listStorage("votes/");
await bridge.deleteStorage("old-key");

// Introspection reseau
const status = await bridge.getNodeStatus();
const apps   = await bridge.getBrowseList();
const proof  = await bridge.verifyRelease(projectId);
const card   = await bridge.getProofCard(projectId);

// Recherche full-text locale (FTS5)
const results = await bridge.search("traduction", { limit: 20 });

// Identite locale
const me = await bridge.getIdentityPubkey();

// Compute distribue (consentement explicite requis)
await bridge.submitTask({ prompt: "analyse ce texte" });

// Notifications push (daemon -> app)
bridge.onStorageUpdate("ideas-app", () => { reload(); });
```

Le bridge est le seul canal entre l'app sandboxee et le reseau.
Aucune requete reseau directe n'est possible depuis l'iframe
(CSP `connect-src 'none'`).

## Calcul GPU distribue

Un noeud SBFB peut partager du calcul GPU au reseau. Le worker
supporte deux backends LLM :

- **Ollama** : parle HTTP au daemon Ollama local, zero dependance build
- **llama.cpp embarque** : runtime integre au worker Rust, decodage
  contraint via llguidance (JSON Schema)

Le consentement est explicite a 4 niveaux :

| Niveau | Portee |
|--------|--------|
| L1 | Mes projets uniquement |
| L2 | Projets open source verifies |
| L3 | Whitelist manuelle (projet par projet) |
| L4 | Tous les projets |

Chaque niveau accepte des caps par projet : watts max, VRAM max,
heures max. Le worker refuse automatiquement les taches hors-scope.

Les resultats sont signes par le worker, valides par quorum SHA256
quand plusieurs workers participent. Les prompts sont filtrables
(redaction PII via GLiNER ONNX + regex). Les taches suspectes vont
en quarantaine.

La reputation (kudos) suit un rendement logarithmique :
`floor(1000 * log2(1 + tokens))`. 1000 tokens donnent 10 000
kudos, pas un million. Une decroissance temporelle EMA (alpha 0.97,
demi-vie ~23 jours) favorise les contributions recentes. Le ledger
kudos est une chaine de hash BLAKE3 + Ed25519, auditable par tous.

## Modele de securite

5 couches cumulatives protegent l'utilisateur :

1. **Iframe sandbox** : `sandbox="allow-scripts"` sans
   `allow-same-origin`. L'app ne voit pas le DOM parent, pas de
   cookies, pas de localStorage du shell.
2. **CSP strict** : `connect-src 'none'`. Zero requete reseau
   sortante (pas de fetch, pas de WebSocket, pas de XHR).
3. **Origin separee** : blob-serve sur un port distinct du daemon.
   Isolation cookies/storage cross-origin.
4. **Bridge postMessage** : seule voie de communication, avec
   correlation UUID, timeout 10 s et heartbeat 1 s (watchdog CPU).
5. **Loopback authentifie** : bearer 256-bit + Host allowlist +
   Origin check + peer creds (UDS SO_PEERCRED Linux, Named Pipe
   DACL Windows).

Couches supplementaires : anti-spam Hashcash 16-bit sur gossip,
resistance Sybil couche 1 (age witness 7 jours min), rotation de
cle Ed25519 (fenetre 14 jours max), domain separation tags RFC 8785
JCS (14 domaines).

Crypto utilisee : Ed25519 (RFC 8032) pour les signatures, BLAKE3
pour les hash, ChaCha20-Poly1305 pour le transport QUIC, Argon2id
pour le chiffrement de la cle au repos. Pas de RSA, pas de wallet,
pas de blockchain.

## Niveaux de confiance (N0-N5)

| Niveau | Label | Ce qui est garanti |
|--------|-------|--------------------|
| N0 | Upload direct | L'archive existe sur le reseau. Origine inconnue. |
| N1 | Source lisible | Un depot source public est reference. |
| N2 | Provenance | Commit + hash + signature Ed25519 du noeud sont lies (SLSA L1). |
| N3 | Signature verifiee | Le daemon local a verifie live la signature et le hash. |
| N4 | Build reproductible | Un tiers a reconstruit le meme hash (futur). |
| N5 | Feed verifie | L'historique complet du projet est integre (hash-chain BLAKE3). |

Les niveaux sont cumulatifs : N2 implique N1 qui implique N0.
N4 n'est pas encore implemente (necessite une infrastructure de
build tiers).

## Structure du projet

```
sbfb/
├── Cargo.toml                         # workspace Rust
├── crates/
│   ├── nexus-core-rs/                 # iroh 0.98 wrapper (docs, gossip, blobs,
│   │                                  # discovery, curator crypto, canonical bytes JCS)
│   ├── nexus-events-core/             # evenements securite + writers JSONL / ETW
│   ├── nexus-worker-core/             # moteur worker headless (state machine,
│   │                                  # allowlist SQLite, GPU monitor, Ollama, llama.cpp)
│   ├── nexus-worker/                  # worker binaire (CLI + TUI)
│   ├── nexus-shell-daemon-core/       # P2P discovery (curator runtime, browse,
│   │                                  # registry singleton, feed)
│   ├── nexus-shell-daemon/            # daemon binaire (HTTP loopback + gossip)
│   ├── nexus-launcher/                # lanceur minimal (spawn daemon + ouvre navigateur)
│   ├── nexus-coordinator-rs/          # base de donnees + dispatcher + validator +
│   │                                  # kudos + invite + quarantaine + capabilities
│   ├── nexus-executor/                # executeur de taches
│   ├── nexus-trace-core/              # infrastructure OpenTelemetry
│   ├── nexus-test-harness/            # harness de test
│   ├── sbfb-manifest/                 # parseur SBFB.json v2
│   └── sbfb-factory/                  # CLI Factory (create, validate, publish)
├── web/                               # shell React (Browse, Curators, Network, Deploy)
├── examples/
│   ├── sbfb-explorer/                 # Protocol Explorer (HTML/JS pur)
│   ├── sbfb-ideas/                    # Ideas Hub — vote decentralise
│   └── sbfb-factory-viewer/           # Viewer apps + Proof Cards
└── docs/
    ├── security/THREAT_MODEL.md       # modele de menace STRIDE + LINDDUN
    └── trust/TRUST_TAXONOMY.md        # niveaux de confiance N0-N5
```

## Stack technique

- **Langage** : Rust 1.85+ (workspace 14 crates)
- **P2P** : iroh 0.98 / iroh-docs 0.98 / iroh-gossip 0.98 / iroh-blobs 0.100
- **Crypto** : Ed25519 (ed25519-dalek), BLAKE3, Argon2id, AES-256-GCM, FROST Ed25519
- **Frontend** : React 19 + TypeScript + Vite + Tailwind + shadcn/ui + Zustand + React Query
- **LLM** : Ollama (HTTP) + llama-cpp-2 embarque + llguidance (decodage contraint)
- **Base locale** : rusqlite (SQLite embarque)
- **Reseau** : QUIC (iroh), Pkarr DHT, DNS fallback, Tor optionnel (arti-client)
- **CI** : Woodpecker (ci.sbfb.world) + GitHub Actions

## Tests

Environ 1800 tests automatises :

- 1486 tests Rust (`cargo nextest run --workspace`)
- 279 tests frontend Vitest (`npm run test:unit`)
- 6 checks de taille bundle (`npm run size`)
- Doctests Rust (`cargo test --workspace --doc`)

Les tests couvrent le protocole P2P, la crypto, le sandbox, le
bridge, le feed, les kudos, le worker, le coordinator, la Factory,
le manifest et le frontend.

## Licence

AGPL-3.0-or-later. Le code du protocole et du daemon reste libre,
meme utilise comme service reseau. Les apps publiees sur SBFB ne
deviennent pas automatiquement AGPL : chaque app a sa propre licence.

## Etat du projet

Ce projet est une experimentation avancee, libre et testable, mais
encore en pilote ferme. Il n'est pas pret pour une mise en production.

Ce qui fonctionne :

- Daemon local, interface web, sandbox, bridge, publication depuis
  source, provenance signee, feed verifiable, recherche locale,
  Proof Cards, Factory CLI, worker GPU/CPU avec consentement opt-in.
- P2P valide : LAN (Windows/Mac), WAN (dev/VPS Helsinki).
- Installeurs Windows (NSIS), Linux (.deb), macOS (.dmg).

Ce qui reste a faire :

- Audit de securite formel de la pile complete.
- Build reproductible par un tiers independant.
- Worker quorum E2E public.
- Gouvernance communautaire complete (UI curator, dissent, timeline).
- Distribution publique large (bloquee par R-iroh-audit P0).

Le projet est maintenu en solo, sur le modele OpenBSD : pas de
startup, pas de fondation, pas de financement.

## Contribuer

Le depot est sur Codeberg. Pour le moment, l'acces pilote se fait
en lecture seule le temps de garder un cadre de test ferme.

Si le projet vous interesse (en tant qu'hebergeur alternatif,
association, CHATONS, collectif local ou developpeur curieux),
vous pouvez :

- Ouvrir une issue pour poser une question ou signaler un probleme.
- Demander un acces pilote pour tester un noeud a cote de vos
  services existants.
- Lire le code source et remonter les angles morts.

Les critiques sont bienvenues, surtout les critiques dures. C'est
le meilleur moment pour les recevoir.
