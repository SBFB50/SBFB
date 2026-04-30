# Architecture du launcher SBFB

**Date** : 2026-04-30
**Statut** : draft
**Auteur** : session S46 analysis

---

## 1. Vision

Le daemon SBFB est le protocole. On distribue un seul binaire
(~21 Mo) qui contient tout le necessaire pour rejoindre le reseau
P2P, servir des apps dans le navigateur, et gerer l'identite
locale. Le frontend (shell React, futur client mobile, etc.) est
une app comme les autres : une archive zip distribuee par le reseau
via iroh-blobs. L'utilisateur double-clique le binaire, le daemon
telecharge le frontend par defaut, ouvre le navigateur, et tout
fonctionne. Zero prerequis. Zero CLI. Zero serveur central.

---

## 2. Architecture

### 2.1 Composants

```
+----------------------------------------------------------+
|                    Binaire unique                         |
|                  nexus-shell-daemon                       |
|                      (~21 Mo)                             |
|                                                          |
|  +-------------+  +----------+  +--------------------+   |
|  | iroh node   |  | HTTP     |  | Auth               |   |
|  | P2P + QUIC  |  | server   |  | token generate     |   |
|  | blobs       |  | axum     |  | loopback checks    |   |
|  | gossip      |  |          |  |                    |   |
|  | relay       |  |          |  |                    |   |
|  +------+------+  +----+-----+  +--------------------+   |
|         |              |                                  |
|  +------+------+  +----+-----+  +--------------------+   |
|  | Coordinator |  | blob-    |  | Frontend           |   |
|  | dispatcher  |  | serve    |  | manager            |   |
|  | validator   |  | CSP      |  | fetch / cache      |   |
|  | kudos       |  | sandbox  |  | registry           |   |
|  +-------------+  +----------+  +--------------------+   |
+----------------------------------------------------------+
```

Le launcher actuel (`crates/nexus-launcher/`) sera fusionne dans
le daemon. Le daemon absorbe les responsabilites restantes :
token generate, browser open, lifecycle management. Un seul
processus, un seul binaire.

### 2.2 Decomposition taille (~21 Mo)

```
+------------------------------------------+
| iroh (P2P, QUIC, relay, blobs)  8-10 Mo  | 48%
+------------------------------------------+
| tokio + hyper + axum             3-4 Mo   | 17%
+------------------------------------------+
| rustls + crypto (ed25519, FROST) 3 Mo     | 14%
+------------------------------------------+
| coordinator-rs                   2 Mo     | 10%
+------------------------------------------+
| reqwest (HTTP client)            2 Mo     |  9%
+------------------------------------------+
| reste (zip, blake3, tracing)     2 Mo     |  ~%
+------------------------------------------+
```

Le plancher incompressible pour un noeud iroh qui fetch des blobs
et les sert en HTTP = ~15 Mo. Les ~6 Mo restants sont de la
logique metier (coordinator, FROST, deploy). Comparable :

| Logiciel     | Taille  | Stack                       |
|--------------|---------|-----------------------------|
| SBFB daemon  | ~21 Mo  | iroh P2P + HTTP + crypto    |
| Syncthing    | ~22 Mo  | Go P2P + HTTP               |
| IPFS Kubo    | ~50 Mo  | Go P2P + HTTP               |
| qBittorrent  | ~30 Mo  | C++ BitTorrent + Qt GUI     |
| Tor          | ~8 Mo   | C (pas de blobs/HTTP)       |

---

## 3. Flows

### 3.1 Premier lancement

```
User double-clic nexus-shell-daemon.exe
           |
           v
  [1] Genere token auth 256-bit
      Persiste ~/.sbfb/auth_token
           |
           v
  [2] Boot iroh node
      Rejoint le reseau P2P
           |
           v
  [3] Frontend local present ?
      ~/.sbfb/frontends/shell-react/index.html
           |
     +-----+------+
     |             |
    OUI           NON
     |             |
     |        [4] Sert page "Chargement..."
     |            (HTML inline, zero dep externe)
     |             |
     |        [5] Fetch frontend via iroh-blobs
     |            Hash connu (hardcode ou curator list)
     |            BlobsClient::fetch_ticket()
     |             |
     |        [6] Decompresse zip
     |            → ~/.sbfb/frontends/shell-react/
     |            Ecrit meta.json (version, hash)
     |             |
     +-----+------+
           |
           v
  [7] Sert le frontend sur /
      (ServeDir, route publique sans auth)
           |
           v
  [8] Ouvre le navigateur
      http://127.0.0.1:{port}/
           |
           v
  [9] Le shell fetch GET /auth/token
      (public, Host+Origin loopback checks)
      Cache le token bearer
           |
           v
  [10] Le shell appelle les API daemon
       avec X-SBFB-Token header
       L'utilisateur voit le shell.
```

### 3.2 Lancement normal (frontend deja en cache)

```
User double-clic nexus-shell-daemon.exe
           |
           v
  [1] Charge token existant
      ~/.sbfb/auth_token
           |
           v
  [2] Boot iroh node
           |
           v
  [3] Frontend present → sert immediatement
      ServeDir sur ~/.sbfb/frontends/shell-react/
           |
           v
  [4] Ouvre le navigateur
      Shell charge instantanement
           |
           v
  [5] Background : check mise a jour
      Resout le hash courant du frontend
      via curator list ou canal dedie
           |
      +----+----+
      |         |
   Meme hash   Hash different
   = a jour    = update dispo
      |              |
    (rien)     [6] Telecharge en arriere-plan
                   Ecrit dans ~/.sbfb/frontends/
                   staging/ temporaire
                        |
                   [7] Au prochain lancement :
                       swap staging → actif
```

### 3.3 Mise a jour du frontend

```
Reseau P2P                         Daemon local
-----------                        ------------

Mainteneur publie                  Daemon tourne, shell actif
shell-react v1.1.0                       |
comme blob iroh                          v
     |                             [1] Check periodique
     |                                 (gossip subscribe OU
     |                                  curator list poll)
     |                                      |
     |  <--- iroh-blobs fetch_ticket --->   |
     |                                      v
     |                             [2] Hash nouveau ≠ local
     |                                      |
     |                                      v
     |                             [3] Download zip via P2P
     |                                 Verifie hash (integrite
     |                                 iroh-blobs native)
     |                                      |
     |                                      v
     |                             [4] Decompresse dans staging/
     |                                      |
     |                                      v
     |                             [5] Prochain lancement :
     |                                 registry.json pointe
     |                                 vers la nouvelle version
```

### 3.4 Relation daemon / frontends / reseau P2P

```
+------------------------------------------------------------------+
|                        Reseau P2P (iroh)                         |
|                                                                  |
|  [blob abc12...]          [blob def34...]      [blob 789ef...]   |
|  shell-react v1.0.0      app-gov v0.3.0       shell-mobile      |
|  (zip, 1.9 Mo)           (zip, 200 Ko)        v0.1.0 (futur)   |
|                                                                  |
+----------+-------------------+-------------------+---------------+
           |                   |                   |
           | fetch_ticket      | fetch_ticket      |
           |                   |                   |
+----------v-------------------v-------------------v---------------+
|                       Daemon local                               |
|                                                                  |
|  iroh node ---------> BlobsClient                                |
|                           |                                      |
|                           v                                      |
|  ~/.sbfb/frontends/                                              |
|      registry.json  <-- choix actif                              |
|      shell-react/   <-- decompresse, servi sur /                 |
|      app-gov/       <-- decompresse, servi via blob-serve        |
|                                                                  |
|  HTTP server (axum)                                              |
|      GET /               → frontend actif (ServeDir)             |
|      GET /auth/token     → token bearer (public, loopback)       |
|      GET /blob-serve/... → apps sandboxees (iframe + CSP)        |
|      GET /api/v1/...     → API daemon (bearer auth)              |
|                                                                  |
+------------------------------------------------------------------+
           |
           v
+------------------------------------------------------------------+
|                     Navigateur (127.0.0.1)                       |
|                                                                  |
|  +---------------------------+  +----------------------------+   |
|  | Shell React (frontend     |  | iframe sandbox             |   |
|  | actif, servi sur /)       |  | app-gov via blob-serve     |   |
|  |                           |  | CSP connect-src 'none'     |   |
|  | Appelle API daemon avec   |  | Communique via postMessage |   |
|  | X-SBFB-Token header       |  | bridge (3 methodes)        |   |
|  +---------------------------+  +----------------------------+   |
+------------------------------------------------------------------+
```

---

## 4. Structure fichiers

```
~/.sbfb/
|
+-- auth_token                      256-bit hex, genere au 1er boot
|                                   Jamais transmis sur le reseau.
|
+-- running.json                    Singleton marker du daemon :
|                                   {api_host, api_port, pid}
|
+-- frontends/
|   +-- registry.json               Liste des frontends installes :
|   |                                [{id, version, blob_hash,
|   |                                  path, active, installed_at}]
|   |
|   +-- shell-react/                Frontend par defaut
|   |   +-- index.html
|   |   +-- assets/
|   |   |   +-- index-[hash].js     (~500 Ko, shell pur)
|   |   |   +-- index-[hash].css    (~120 Ko, Tailwind)
|   |   +-- meta.json               {version, blob_hash,
|   |                                 installed_at, source}
|   |
|   +-- staging/                    Download en cours (atomicite)
|       +-- shell-react-v1.1.0/    Pas servi tant que pas valide
|
+-- coordinator.db                  SQLite WAL (tasks, kudos, etc.)
|
+-- logs/
|   +-- daemon.log                  Rotation quotidienne
|
+-- run/
|   +-- daemon.sock                 UDS (Unix) / Named Pipe (Win)
|
+-- canary-key.key                  Ed25519 maintainer key (opt-in)
+-- tokens.json                     Rotation state (opt-in)
```

### Distinction frontend actif vs apps blob-serve

| Concept         | Chemin de service          | Auth requis | Sandbox |
|-----------------|----------------------------|-------------|---------|
| Frontend actif  | `GET /` (ServeDir)         | Non         | Non (same-origin, trusted) |
| App distante    | `GET /blob-serve/{hash}/`  | Non         | Oui (iframe, CSP strict)   |
| API daemon      | `GET /api/v1/...`          | Oui (bearer)| N/A     |
| Token bootstrap | `GET /auth/token`          | Non (Host+Origin check) | N/A |

Le frontend actif est le chrome de l'application — le shell qui
host les apps distantes dans des iframes. Il n'est pas sandboxe
car il a besoin d'appeler les API daemon (avec le bearer token).
Les apps distantes sont sandboxees (CSP `connect-src 'none'`) et
communiquent via le bridge postMessage.

---

## 5. Securite

### 5.1 Token auth loopback

Le token bearer est un secret local de 256 bits (64 hex chars)
genere au premier boot. Il n'est jamais transmis sur le reseau P2P.

```
Token lifecycle :

  [1] Daemon boot
      → generate_token() (32 random bytes → hex)
      → persist ~/.sbfb/auth_token (mode 0600 Unix, user ACL Win)

  [2] GET /auth/token (public route, loopback only)
      → Host header must be 127.0.0.1 / localhost / [::1]
      → Origin header (if present) must be loopback http://
      → Returns {"token": "<hex>"}

  [3] Shell caches token, injects X-SBFB-Token on every API call

  [4] auth_required middleware validates on every authenticated route
      → Constant-time compare (pas de timing leak)
      → Rejet = 401 "missing or invalid token"
```

Meme modele que Syncthing (API key loopback), Jupyter (token URL),
BOINC (gui_rpc_auth).

### 5.2 Integrite des frontends

Les frontends sont des blobs iroh. L'integrite est garantie par
le hash BLAKE3 du contenu — le protocole iroh-blobs verifie le
hash a la reception. Un blob corrompu ou modifie est rejete avant
ecriture disque.

```
Mainteneur publie frontend :

  [1] npm run build → web/dist/
  [2] zip web/dist/ → shell-react-v1.0.0.zip
  [3] BlobsClient::add_bytes(zip_bytes) → hash [32 bytes]
  [4] Publie le hash sur gossip / curator list
      (signe Ed25519 par le mainteneur)

User recoit frontend :

  [1] Daemon recoit le hash via gossip / curator list
  [2] Verifie la signature Ed25519 du mainteneur
  [3] BlobsClient::fetch_ticket(hash) → telecharge via P2P
  [4] iroh-blobs verifie hash == contenu (BLAKE3)
  [5] Decompresse zip, valide paths (pas de traversal)
  [6] Ecrit dans ~/.sbfb/frontends/{id}/
```

Pas de TLS entre pairs (le hash EST l'authentification).
Pas de CDN/serveur central (le reseau EST la distribution).

### 5.3 Isolation des apps

Le frontend actif (shell React) tourne en same-origin sur
le daemon — il a acces aux API avec le bearer token. Les apps
tierces sont servies via `/blob-serve/{hash}/` dans des iframes
sandboxees :

```
<iframe sandbox="allow-scripts"
        src="/blob-serve/{hash}/index.html">
```

CSP injecte sur chaque reponse blob-serve :
- `Content-Security-Policy: connect-src 'none'` (pas de fetch)
- `Cross-Origin-Opener-Policy: same-origin`
- `Cross-Origin-Embedder-Policy: require-corp`
- `X-Content-Type-Options: nosniff`

Les apps communiquent avec le daemon uniquement via le bridge
postMessage (3 methodes whitelisted : `task_submit`,
`storage_get`, `storage_set`).

### 5.4 Le seul point non-P2P

Le binaire daemon initial (~21 Mo) doit venir de quelque part :

- Site web du projet (HTTPS, signature verifiable)
- Cle USB entre pairs
- Message direct (Signal, email, etc.)
- Package manager (winget, brew, apt — futur)

C'est le trust anchor du systeme. Apres ce premier binaire, tout
transite par le protocole P2P. Meme les mises a jour du daemon
seront distribuees comme blobs signes (futur).

---

## 6. Taille et performance

### 6.1 Budget taille

| Artefact                | Taille   | Transit     |
|-------------------------|----------|-------------|
| Daemon (binaire)        | ~21 Mo   | Non-P2P (1x)|
| Shell React (sans WASM) | ~1.9 Mo  | P2P (blob)  |
| WASM SynthID (optionnel)| ~47 Mo   | P2P (blob)  |
| App tierce typique      | 50-500 Ko| P2P (blob)  |

Le shell React "pur" (JS + CSS + HTML) fait 1.9 Mo. Les 47 Mo de
WASM ONNX Runtime (watermark SynthID) sont un asset optionnel
telecharge a la demande — pas au premier boot. L'experience
premier lancement est : 21 Mo binaire + 1.9 Mo frontend = ~23 Mo
au total avant que le shell soit fonctionnel.

### 6.2 Temps de demarrage

| Etape                      | Temps estime |
|----------------------------|--------------|
| Token generate/load        | < 1 ms       |
| iroh node boot             | 1-3 s        |
| Frontend fetch (1er boot)  | 2-10 s (P2P) |
| Frontend serve (cache)     | < 1 ms       |
| Browser open               | < 500 ms     |
| **Total premier boot**     | **5-15 s**   |
| **Total boot normal**      | **2-4 s**    |

Le frontend est en cache apres le premier boot — le deuxieme
lancement est quasi-instantane.

### 6.3 Reduction de taille future

Le daemon a 20.9 Mo est domine par le stack iroh P2P (~50%). Les
pistes de reduction :

- **Feature gates Cargo** : coordinator-rs, FROST ceremony, deploy
  from repo pourraient etre des features optionnelles. Impact :
  -3 a -5 Mo.
- **LTO + strip** : `lto = true` + `strip = "symbols"` dans le
  profil release. Impact : -10 a -20% (typique Rust).
- **UPX compression** : compresse le binaire a ~40% de sa taille
  originale. Decompression transparente au lancement (+100 ms).
  Impact : 21 Mo → ~9 Mo. Trade-off : antivirus faux positifs
  sur Windows.

---

## 7. Etat actuel vs cible

### 7.1 Ce qui existe (code present dans le repo)

| Composant              | Fichier(s)                              | Etat    |
|------------------------|-----------------------------------------|---------|
| Daemon P2P complet     | `crates/nexus-shell-daemon/`            | Prod    |
| iroh-blobs fetch       | `crates/nexus-core-rs/src/blobs.rs`     | Prod    |
| blob-serve (zip→HTTP)  | `nexus-shell-daemon-core/blob_serve.rs` | Prod    |
| Token auth loopback    | `nexus-shell-daemon-core/auth.rs`       | Prod    |
| `GET /auth/token`      | `nexus-shell-daemon/http.rs`            | Prod    |
| `--web-root` (ServeDir)| `nexus-shell-daemon/http.rs`            | Prod    |
| Launcher (spawn+open)  | `crates/nexus-launcher/`                | Prod    |
| Browser open           | `crates/nexus-launcher/src/main.rs`     | Prod    |
| Console cachee release | `main.rs` `windows_subsystem="windows"` | Prod    |
| Icone .exe Windows     | `nexus-launcher/build.rs` + winresource | Prod    |
| postMessage bridge     | `sbfb-bridge.js`, Sprint 13             | Prod    |
| Verified deploy        | `deploy.rs`, Sprint 42                  | Prod    |

### 7.2 Ce qui manque (ecart vers la cible)

| Gap                           | Description                                          | Effort |
|-------------------------------|------------------------------------------------------|--------|
| **Fusion launcher+daemon**    | Absorber token generate, browser open, lifecycle dans le daemon. Supprimer le crate launcher. | S      |
| **Frontend manager**          | Module daemon qui fetch un blob par hash, decompresse dans `~/.sbfb/frontends/`, ecrit `registry.json`. | S      |
| **Page "Chargement..."**      | HTML inline servi sur `/` quand le frontend n'est pas encore telecharge. Pas de dep externe. | trivial |
| **Hash frontend par defaut**  | Hardcode dans le binaire OU resolu via curator list bien connue. Decision a prendre. | trivial |
| **Registry JSON**             | Schema pour `~/.sbfb/frontends/registry.json` : id, version, hash, path, active. | trivial |
| **Update check background**   | Au boot, compare le hash local vs hash reseau. Telecharge si different, swap au prochain lancement. | S      |
| **MessageBox erreurs Win32**  | Afficher une boite de dialogue sur erreur fatale en mode release (pas de console = silencieux sinon). | S      |
| **System tray (optionnel)**   | Icone dans la zone de notification avec menu Quit/Open Browser. Pas bloquant pour v1. | M      |
| **Auto-update daemon (futur)**| Le daemon lui-meme distribue comme blob signe. Verification Ed25519 avant remplacement binaire. | M      |

### 7.3 Chemin d'implementation

```
Phase 1 — Fusion (S46)
    Merger launcher dans daemon.
    Un seul binaire, un seul processus.
    Supprimer auth server separe (daemon sert deja /auth/token).
    Supprimer token rotation (single-user loopback, pre-v1.0).
    Garder : token generate, browser open, daemon lifecycle.

Phase 2 — Frontend P2P (S46-47)
    Frontend manager dans le daemon.
    Fetch blob par hash au premier boot.
    Page "Chargement..." inline.
    registry.json + meta.json.
    Update check background.

Phase 3 — Polish Windows (S47)
    MessageBox Win32 pour erreurs fatales.
    LTO + strip dans le profil release.
    Tester le flow one-click E2E sur machine vierge.

Phase 4 — System tray (S48+, optionnel)
    Icone zone de notification.
    Menu Quit / Open Browser / Status.
    Shutdown propre via menu.
```

---

## 8. Frontends

### 8.1 Qu'est-ce qu'un frontend SBFB ?

Un frontend est une archive zip contenant un site web statique
avec un `index.html` a la racine. Techniquement, c'est identique
a une app SBFB publiee sur le reseau — meme format, meme
distribution par iroh-blobs. La seule difference : le frontend
actif est servi en same-origin sur `/` (pas dans un iframe
sandboxe).

### 8.2 Comment un developpeur publie un frontend

```
[1] Developper le frontend
    N'importe quelle techno qui produit du HTML statique :
    React, Vue, Svelte, plain HTML, Elm, etc.

[2] Builder
    npm run build → dist/
    Le resultat doit contenir index.html a la racine.

[3] Zipper
    zip -r shell-react-v1.1.0.zip dist/*

[4] Publier sur le reseau
    Via le daemon (endpoint /publish-blob ou CLI) :
    → BlobsClient::add_bytes(zip) → hash BLAKE3
    → Annonce le hash sur gossip ou via curator list

[5] Distribuer le hash
    Le hash du blob est la reference unique.
    Peut etre publie sur :
    - Un curator list signe (canal de distribution P2P)
    - Le repo Git du projet (fichier texte)
    - Un site web / README

[6] Les daemons fetchent le frontend
    Quand un daemon recoit le nouveau hash
    (via gossip, curator list, ou configuration manuelle) :
    → fetch_ticket(hash)
    → decompresse zip
    → ecrit meta.json
    → sert au prochain lancement (ou hot-swap si supporte)
```

### 8.3 Format meta.json

```json
{
  "id": "shell-react",
  "name": "SBFB Shell",
  "version": "1.0.0",
  "blob_hash": "abc123def456...",
  "installed_at": "2026-04-30T19:00:00Z",
  "source": "curator:mainteneur_pubkey_hex"
}
```

### 8.4 Format registry.json

```json
{
  "schema_version": 1,
  "active": "shell-react",
  "frontends": [
    {
      "id": "shell-react",
      "name": "SBFB Shell",
      "version": "1.0.0",
      "path": "shell-react",
      "blob_hash": "abc123def456...",
      "installed_at": "2026-04-30T19:00:00Z"
    }
  ]
}
```

### 8.5 Multi-frontend (futur)

Quand plusieurs frontends sont installes, le daemon sert celui
designe par `registry.json → active`. Le changement de frontend
actif se fait via l'API daemon :

```
POST /api/v1/frontends/activate
{"id": "shell-mobile"}
```

Ou via la page de bienvenue du shell (UI de selection).

---

## 9. Decisions ouvertes

| #   | Question                                         | Options                              |
|-----|--------------------------------------------------|--------------------------------------|
| D-1 | Hash du frontend par defaut : hardcode ou resolu | (a) Hardcode dans le binaire, bump a chaque release daemon. (b) Curator list "official-frontends" resolue au boot. (c) Les deux : hardcode = fallback, curator list = update channel. |
| D-2 | Hot-swap frontend ou swap au reboot               | (a) Hot-swap : le daemon surveille staging/, switch ServeDir live. (b) Reboot : plus simple, pas de race condition. |
| D-3 | WASM SynthID : inclus dans le zip frontend ou blob separe | (a) Inclus : 49 Mo par frontend, lourd. (b) Blob separe reference par le shell : le shell fetch /blob-serve/{wasm_hash}/ort-wasm.wasm a la demande. Plus leger, lazy loading. |
| D-4 | Merge launcher crate ou nouveau mode daemon       | (a) Supprimer `crates/nexus-launcher/`, absorber dans `nexus-shell-daemon start --gui`. (b) Garder le launcher comme thin wrapper qui exec le daemon. |
