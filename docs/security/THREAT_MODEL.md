# Threat model — nexus-grid / SBFB

**Ecrit** : Sprint 16 Phase E (2026-04-14)
**Tip reference** : `10bbc63` (Phase D PA v5 livre)
**Methodologie** : STRIDE (Microsoft) + LINDDUN (KU Leuven) via
pattern OWASP Threat Dragon.

---

## 1. Scope & assumptions

### 1.1 Composants dans le scope

| Composant | Code | Surface |
|---|---|---|
| **Shell React** | `web/src/` | Browser tab hote, iframe parent |
| **Shell daemon** | `crates/nexus-shell-daemon*` | Binaire local, P2P, HTTP loopback |
| **Coordinator** | `packages/nexus-coordinator/` | FastAPI local, dispatcher, deploy |
| **Blob-serve** | `crates/nexus-shell-daemon/src/blob_serve.rs` | HTTP origin separee iframe |
| **Worker** | `crates/nexus-worker*` | Binaire GPU, claim-loop, Ollama runtime |
| **NexusApp iframe** | `packages/nexus-app-*`, apps externes | Contenu untrusted, bridge postMessage |
| **iroh stack** | `crates/nexus-core-rs` | QUIC, docs, gossip, blobs 0.97 |
| **Keypair Ed25519** | `~/.sbfb/daemon.key` + `auth_token` | Identite node + bearer loopback |

### 1.2 Composants hors scope

- Infrastructure OS (kernel, drivers, CUDA runtime)
- Reseau externe (ISP, DNS root, CA publiques)
- Repos Git publics (GitHub, GitLab, Codeberg, Gitea) — trust
  chain gere par `provenance.json` Sprint 14
- Binaires tiers (Ollama, Python interpreter, Node)

### 1.3 Hypotheses

- H1. **Machine locale partiellement trusted** : l'utilisateur
  controle son compte user-mode mais peut cohabiter avec des
  extensions navigateur et des malwares user-mode de severite
  faible a moyenne (pas root).
- H2. **Cryptographie classique** : Ed25519, BLAKE3 et
  ChaCha20-Poly1305 (iroh QUIC) sont consideres solides.
  Compromission quantique hors scope (v2+).
- H3. **Trust-on-first-use** : la premiere fois qu'un peer voit
  un node_id via gossip, il l'accepte (pas de Web-of-Trust). Les
  curator lists sont la couche de reputation volontaire.
- H4. **Verified deploy** : tout projet public a ete clone par
  le coordinator depuis un repo public verifie (Keyoxide pattern
  Sprint 14). Le code sur le reseau = le code du repo.
- H5. **Boundary iframe browser-enforced** : l'utilisateur a un
  navigateur moderne (Chromium 2024+ / Firefox 120+) qui
  implemente correctement `sandbox="allow-scripts"` sans
  `allow-same-origin` et la CSP `connect-src 'none'`.

---

## 2. Assets

| # | Asset | Confidentialite | Integrite | Disponibilite | Emplacement |
|---|---|:---:|:---:|:---:|---|
| A1 | Keypair Ed25519 node_id | **Critique** | **Critique** | Haute | `~/.sbfb/daemon.key` (plaintext, perm 0600) |
| A2 | Bearer token loopback | **Haute** | Haute | Moyenne | `~/.sbfb/auth_token` (hex 64 chars, perm 0600) |
| A3 | Provenance signatures | Faible (publique) | **Critique** | Haute | `provenance.json` signe inclus dans zip blob |
| A4 | User consent preferences | Moyenne (PII light) | **Haute** | Haute | `~/.sbfb/consent.json` (atomic tmp+rename) |
| A5 | GPU usage counters | Faible | Haute | Moyenne | `~/.sbfb/usage.json` (daily reset) |
| A6 | Project archives (zip) | Faible (publique) | **Critique** | Haute | iroh-blobs + BLAKE3 hash |
| A7 | Task results + kudos ledger | Moyenne | **Haute** | Moyenne | iroh-docs + hash-chain append-only |

---

## 3. Adversary model

| # | Persona | Capacite | Motivation | Scenario clef |
|---|---|---|---|---|
| **AD1** | Extension navigateur malveillante | `host_permissions: "http://localhost/*"` | Exfil / elevation | Fetch `POST /project/deploy-from-repo` pour publier sous l'identite user |
| **AD2** | Malware user-mode local | Accesses le home dir user, pas root | Exfil keypair, pivot | Lit `~/.sbfb/daemon.key` ou abuse de `auth_token` pour signer des annonces |
| **AD3** | Noeud byzantin P2P | Peer distant, connect via QUIC | DoS, fake provenance | Publie annonces invalides, remplit les curator lists de spam |
| **AD4** | Repo Git squatte | Proprietaire du repo public | Supply chain | Push un commit backdoored, attend re-clone du coordinator |
| **AD5** | Fournisseur d'app malveillant | Contributeur NexusApp | RCE shell via bridge | Exploite le `task_submit` pour exfiltrer via payload crafted |

Non modelises :
- AD6 (nation-state / 0-day navigateur) : hors scope projet solo
- AD7 (physical access) : assume que la machine n'est pas saisie

---

## 4. DFD (Data Flow Diagram)

Flux principaux, notation ASCII. Trust boundaries materialises
par `=======`.

```
      USER BROWSER (trust-A)
    +------------------------+
    |   Shell React          |       <-- CSP strict, same-origin
    |   (window.top)         |
    +------+-----------------+
           |  postMessage bridge
           |  (task_submit / storage_get / storage_set / event)
           v
    +------+-----------------+
    |  App iframe            |       <-- sandbox="allow-scripts"
    |  (blob-serve origin    |           sans allow-same-origin,
    |   :7000, untrusted)    |           CSP connect-src 'none'
    +------------------------+

    authFetch(X-SBFB-Token + Host + Origin)
           |
    =======v=========================================
           |
      LOOPBACK HOST (trust-B)
    +------+-----------------+         +-----------+
    |  Coordinator FastAPI   +-------->|  Worker   |
    |  :8080 (bearer+Host+   |         |  :none    |
    |   Origin enforced)     |         | (GPU/CPU) |
    +------+-----------------+         +-----+-----+
           |                                  |
           | proxy /daemon/*                  | consent.json
           v                                  | usage.json
    +------+-----------------+                v
    |  Shell daemon :7777    |         +------+-----+
    |  (bearer enforced)     |         |  Ollama    |
    +------+-----------------+         |  runtime   |
           |                            +-----------+
           |  UDS (SO_PEERCRED) / Named Pipe (DACL)
           |  bypass auth via PeerCredsVerified marker
           v
    =======+=========================================
           |  iroh QUIC (ChaCha20-Poly1305 + Ed25519)
           |
      P2P NETWORK (trust-C)
    +------+-----------------+
    |   Remote peers         |
    |   gossip, blobs, docs  |
    +------------------------+
```

---

## 5. STRIDE par composant

Severites post-Sprint 16 (`10bbc63`). "res" = residuel apres
mitigations livrees.

### 5.1 App iframe

| Menace | Exemple | Severite brute | Mitigation | res |
|---|---|:---:|---|:---:|
| **S**poofing | App fait passer son origine pour celle du shell | H | `window.parent` + origin check cote bridge ; iframe sur origin distincte `:7000` | **L** |
| **T**ampering | App modifie le DOM du shell parent | H | `sandbox="allow-scripts"` sans `allow-same-origin` : le DOM du parent est inatteignable | **L** |
| **R**epudiation | App nie avoir envoye une task | M | `task_submit` logge avec `app_id` + timestamp cote coordinator | **L** |
| **I**nfo disclosure | App lit les cookies / localStorage du shell | H | Origin `:7000` differente de `:8080` / `:8765` : isolation storage browser-enforced | **L** |
| **D**oS | App boucle infinie JS | M | CPU watchdog heartbeat 1s + timeout 5s (Sprint 15) + overlay "ne repond plus" | **M** |
| **E**oP | App execute du natif via `eval` / WASM escapes | H | Pas d'`allow-same-origin`, pas de `sandbox` flag laxiste ; connect-src 'none' coupe l'exfil | **M** |

### 5.2 postMessage bridge

| Menace | Exemple | Severite brute | Mitigation | res |
|---|---|:---:|---|:---:|
| S | Extension navigateur injecte `postMessage` craftes | H | Bridge valide `event.source === iframe.contentWindow` | **M** |
| T | App fake `response.id` pour corrompre reply d'une autre app | M | Correlation ID UUID par call + Map ownership | **L** |
| I | Bridge leak des donnees cross-iframe | M | 3 methodes whitelist (`task_submit`, `storage_get`, `storage_set`) + 1 canal event push ; schema Zod strict | **L** |
| D | Flood de messages, denied-of-sevice coordinator | M | Correlation ID cap 1 MB/message + callback Set timeout (Phase B/D Sprint 15 discussion) | **M** |

### 5.3 Deploy-from-repo

| Menace | Exemple | Severite brute | Mitigation | res |
|---|---|:---:|---|:---:|
| S | Attaquant deploie sous une identite volee | H | SBFB.json verifie via Keyoxide Ed25519 (node_id matche la key locale) | **L** |
| T | Attaquant push un commit malveillant dans un repo legit | H | `provenance.json` pinne `commit_sha` full 40 hex, signe Ed25519 | **L** |
| R | Publisher nie avoir deploye | L | `provenance.json` signe et archive dans le blob | **L** |
| I | Clone revele creds embarques dans l'historique | M | `git clone --depth 1` (pas d'historique) | **L** |
| D | Clone 50 GB | H | Cap 500 MB + timeout 30s (Sprint 14 D4) | **L** |
| E | Path traversal dans le zip | H | Validation paths `../` refuses cote coordinator | **L** |

### 5.4 iroh stack

| Menace | Exemple | Severite brute | Mitigation | res |
|---|---|:---:|---|:---:|
| S | Peer byzantin forge une annonce v5 avec un node_id vole | H | Annonces signees Ed25519, verification cote recepteur | **L** |
| T | Tampering d'un blob en transit | H | BLAKE3 hash + iroh-blobs verify on retrieval | **L** |
| R | Peer nie avoir envoye une task | M | Task signee avec `task_id` + `claimant_id` (`crates/nexus-core-rs`) | **L** |
| I | Metadata de l'annonce revele info sensible | M | ProjectAnnouncement champs publics par design (publique = public) | **L** |
| D | Gossip flood | M | Rate limit iroh-gossip builtin + curator lists volontaires | **M** |
| E | RCE via deserialization iroh | C | Version pinnee 0.97 + `cargo-audit` scope cut Sprint 17+ | **M** |

### 5.5 Loopback HTTP (coordinator + daemon)

**Surface la plus critique avant Sprint 16** : pre-S16 c'etait
"aucune auth, trust implicite". Post-S16 A-B : triple validation
+ peer creds.

| Menace | Exemple | Severite brute | Mitigation | res |
|---|---|:---:|---|:---:|
| S | Extension navigateur signe une deploy sous identite user | **C** | `d7c265a` : X-SBFB-Token 256-bit `auth_token` file perm 0600 | **L** |
| S | DNS rebinding (CVE-2025-49596 pattern Anthropic MCP) | **C** | `d7c265a` : Host header allowlist `{localhost, 127.0.0.1, [::1]}` + Origin check | **L** |
| T | Site malveillant hitte `/project/deploy-from-repo` cross-origin | H | `d7c265a` : Origin allowlist shell React uniquement | **L** |
| I | Autre user local lit les tasks via UDS | H | `1cfde89` : SO_PEERCRED rejette uid != geteuid() ; DACL Windows user-only | **L** |
| D | Flood HTTP loopback | M | Rate limit scope cut S17+ (explicite kickoff §6) | **M** |
| E | Bypass middleware via `X-SBFB-PeerCreds: true` spoofe | H | `1cfde89` : `PeerCredsVerified` est un marker **type prive** injecte par accept loop UDS/NP, jamais lisible depuis headers | **L** |

### 5.6 Worker-core (Ollama + consent)

| Menace | Exemple | Severite brute | Mitigation | res |
|---|---|:---:|---|:---:|
| S | Task crafted pour consommer plus de ressources que claim | H | `3247e88` : `should_accept_task` check `estimated_watts` / `estimated_vram_mb` contre caps avant accept | **L** |
| T | Consent.json edite par un autre process | H | Atomic tmp+rename cote coordinator (`consent.py`) ; `notify` watcher 50 ms debounce re-read | **M** |
| I | Usage.json divulgue patterns de contribution | L | Local-only, perm 0600 dossier parent | **L** |
| D | Task boucle infinie saturant GPU | M | Caps heures/jour enforced ; Ollama timeout per-request | **M** |
| E | L2 (open source) accepte un projet qui ment sur le flag | H | **res** : `d7c265a` + `10bbc63` — coordinator force `is_open_source=true` uniquement sur deploy-from-repo (repo public clone + verifie) ; non-user-settable | **L** |

### 5.7 Key storage

| Menace | Exemple | Severite brute | Mitigation | res |
|---|---|:---:|---|:---:|
| I | Malware user-mode lit `~/.sbfb/daemon.key` | **C** | Perm 0600, parent 0700. **Encryption at rest scope cut S17+** (Keychain macOS / DPAPI Windows / libsecret Linux) | **H** |
| T | Malware re-ecrit la keypair | **C** | Idem + file watcher alertable Sprint 17+ | **H** |
| E | Autre process du meme user signe des annonces | **C** | Idem perm 0600 | **H** |

**Note** : le trio key storage reste H apres Sprint 16 car
encryption at rest est explicitement differe Sprint 17+. C'est
le risque residuel #1. La roadmap `RUNTIME_ISOLATION.md` le
reduit de 95% via la mise en VM.

### 5.8 Supply chain (deps Rust/Python/npm)

| Menace | Exemple | Severite brute | Mitigation | res |
|---|---|:---:|---|:---:|
| T | Dep Rust malveillante injectee via typosquat | H | `Cargo.lock` commited, review manuelle ajout deps | **M** |
| T | npm dep postinstall script | H | `package-lock.json` commited ; **CI `cargo-audit`/`pip-audit`/`npm audit` scope cut S17+** | **H** |
| T | PyO3 wheel remplacee | H | `maturin develop --release` depuis sources locales ; pas de wheel telechargee | **M** |

---

## 6. LINDDUN par flux

Focus GDPR, pertinent pour un reseau P2P qui collecte des stats
de compute. Severites en contexte RGPD Art.5/6/9/35.

| Flux | **L**inkability | **I**dentifiability | **N**on-rep. | **D**etectability | Di**s**closure | **U**nawareness | **N**on-compliance |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| Worker claim task | M (node_id lie aux claims cross-projets) | L (node_id = pseudo) | L (task signee = trace) | M (peers voient claims) | L | L (Phase C Dialog explicite) | **L (GDPR ok : consentement L1-L4 + caps)** |
| Deploy-from-repo | L (repo_url = info publique) | L | M (provenance signee non-niable) | L | L | L | L |
| GPU consent changes | L | L | L | L | M (consent.json local uniquement) | L (dialog affiche 4 radios) | **L (GDPR Art.7 opt-in explicite, withdrawal meme UX)** |
| Kudos ledger | **H** (hash-chain permet cross-project linking) | M (node_id) | H (append-only = non-repudiation by design) | M | L | M (pas de UI explicite "tes contributions sont publiques") | **M — Sprint 17+ : UI explicitant la publicite du ledger** |
| Gossip annonces | M | L | L | **H (gossip = public par definition)** | L | L | L |

### 6.1 GDPR mapping (Sprint 16 livre)

| Article | Exigence | Implementation |
|---|---|---|
| Art.5 (1)(a) Lawfulness | Transparence | Dialog GPU consent §C + threat model publie |
| Art.5 (1)(c) Data minimization | Ne collecter que le necessaire | Worker n'envoie que `task_id` + `result_hash` aux peers |
| Art.6 (1)(a) Lawful basis | Consentement | `3247e88` : Dialog 4 niveaux, default L1 (zero partage), cap W/VRAM/h |
| Art.7 (3) Withdrawal | Aussi simple que le donner | "Modifier consentement" bouton Network page, meme dialog |
| Art.13 Information | Informer avant collecte | Dialog explique chaque niveau, lien vers `docs/security/` |
| Art.25 Privacy by design | Default privacy-friendly | L1 "mes projets uniquement" = default, pas de pre-cochage |
| Art.32 Security | Mesures techniques | Sprint 16 A-D + roadmap isolation Sprint 17+ |

### 6.2 Residuals LINDDUN

- **Kudos ledger linkability** (M) : la hash-chain permet a un
  peer de correler la contribution d'un worker sur plusieurs
  projets. **Differe v2+** : rotatable `contribution_id` par
  projet, pattern CPID reversible de BOINC.
- **Gossip detectability** (H inherent) : par design un peer sait
  qui annonce quoi. Pas de mitigation sans casser la decouverte.
  **Accepte**.

---

## 7. Mitigations table (Sprint 16 livre + roadmap)

| Mitigation | Composant | Commit | Fichier cle | Statut |
|---|---|---|---|---|
| Bearer 256-bit loopback | Daemon + Coord + Shell | `d7c265a` | `crates/nexus-shell-daemon-core/src/auth.rs:274-383` (AuthState + auth_required) / `packages/nexus-coordinator/src/nexus_coordinator/auth.py:1-229` | **LIVRE S16A** |
| Host header allowlist | Daemon + Coord | `d7c265a` | `crates/nexus-shell-daemon-core/src/auth.rs:218-272` (is_loopback_host/origin) | **LIVRE S16A** |
| Origin header check | Daemon + Coord | `d7c265a` | Idem | **LIVRE S16A** |
| `/auth/token` launcher endpoint | Launcher | `d7c265a` | `crates/nexus-launcher/src/auth.rs` (460 LOC, AuthServer + token file perm 0600) | **LIVRE S16A** |
| UDS SO_PEERCRED | Daemon + Coord (Unix) | `1cfde89` | `crates/nexus-shell-daemon/src/uds_server.rs` (366 LOC) + `packages/nexus-coordinator/src/nexus_coordinator/peer_creds.py` (92 LOC) | **LIVRE S16B** |
| Named Pipe DACL user-only | Daemon (Windows) | `1cfde89` | `crates/nexus-shell-daemon/src/named_pipe_server.rs` (417 LOC) via SDDL `D:(A;;GA;;;<sid>)` | **LIVRE S16B** |
| `PeerCredsVerified` bypass marker | Daemon core | `1cfde89` | `crates/nexus-shell-daemon-core/src/auth.rs:293-305` (type prive, non-spoofable) | **LIVRE S16B** |
| Consent dialog 4 niveaux + whitelist L3 | Shell React | `3247e88` | `web/src/components/GpuConsentDialog.tsx` (385 LOC) | **LIVRE S16C** |
| Caps W/VRAM/h enforced | Worker-core | `3247e88` | `crates/nexus-worker-core/src/consent.rs:381-428` (should_accept_task pure-fn) | **LIVRE S16C** |
| File watcher consent.json | Worker-core | `3247e88` | `crates/nexus-worker-core/src/consent.rs:438-540` (ConsentWatcher, notify + 50 ms debounce) | **LIVRE S16C** |
| Usage daily counter + midnight-local reset | Worker-core | `3247e88` | `crates/nexus-worker-core/src/consent.rs:254-326` (UsageTracker, chrono::Local) | **LIVRE S16C** |
| `is_open_source` flag PA v5 | Core-rs + Coord + Shell | `10bbc63` | `crates/nexus-shell-daemon-core/src/publish.rs:22-110` (VERSION=5) + `packages/nexus-coordinator/src/nexus_coordinator/api/deploy.py:1-475` (derive true/false) | **LIVRE S16D** |
| Zod schema `BrowseEntry.is_open_source: z.boolean().optional()` | Shell | `10bbc63` | `web/src/api/daemon.ts` (distingue legacy undefined de `false` explicite) | **LIVRE S16D** |
| Iframe sandbox strict | Shell + Blob-serve | S12-S13 | `crates/nexus-shell-daemon/src/blob_serve.rs` (CSP connect-src 'none') | **LIVRE** |
| postMessage bridge whitelist | Shell + iframe | S13 | `web/public/sbfb-bridge.js` + `web/src/bridge/protocol.ts` | **LIVRE** |
| Verified deploy Keyoxide + SLSA L1 | Coordinator | S14 | `packages/nexus-coordinator/src/nexus_coordinator/provenance.py` + `deploy.py` | **LIVRE** |
| CPU watchdog heartbeat | Shell bridge | S15 | `web/src/bridge/useBridge.ts` (watchdog state machine) | **LIVRE** |
| Encryption at rest keypair | Daemon | — | Keychain / DPAPI / libsecret | **DIFFERE S17+** |
| VM isolation auto-install | Launcher | — | Cf. `RUNTIME_ISOLATION.md` | **DIFFERE S17+** |
| CI `cargo-audit` / `pip-audit` / `npm audit` | Infra | — | CI pipeline | **DIFFERE S17+** |
| Rate limit `/project/deploy-from-repo` | Coord | — | middleware | **DIFFERE S17+** |
| CSP report-uri | Shell + blob-serve | — | `/security/csp-report` endpoint | **DIFFERE S17+** |
| Token rotation automatique | Launcher | — | Scheduled regen | **DIFFERE S17+** |
| MIME scan zip deploy | Coord | — | libmagic check pre-blob | **DIFFERE S17+** |
| Revocation node_id (CRL Ed25519) | Core-rs | — | Signed revocation msg via gossip | **DIFFERE v2+** |
| PyO3 wheel bytecode signing | Infra | — | Sigstore / cosign attach | **DIFFERE v2+** |
| Audit externe (Trail of Bits / Cure53) | Project | — | Budget hors scope solo | **POST v1.1** |

---

## 8. Residual risks

Apres Sprint 16, les risques les plus serieux encore presents :

### R1 — Keypair au repos non chiffree (severite H)

- **Asset** : A1 (daemon.key), A2 (auth_token)
- **Adversaire** : AD2 (malware user-mode)
- **Impact** : usurpation identite P2P, signature annonces
  frauduleuses, prise de controle du node.
- **Mitigation residuelle Sprint 16** : perm 0600 + parent 0700.
  Un process user-mode a le meme user peut lire.
- **Roadmap** : Sprint 17+ `RUNTIME_ISOLATION.md` elimine 95% via
  la mise en VM (malware sur Windows n'a pas acces au FS WSL2).
  Fallback non-VM : Keychain macOS / DPAPI Windows / libsecret
  Linux, decision D Sprint 17.

### R2 — Supply chain sans CI audit (severite M a H selon dep)

- **Asset** : tout le repo compile
- **Adversaire** : AD3 / AD4 + proprietaire dep typo-squatte
- **Impact** : RCE au build-time ou exec-time.
- **Mitigation residuelle** : `Cargo.lock`, `package-lock.json`,
  pas de wheel telechargee. Revue manuelle ajout deps.
- **Roadmap** : Sprint 17+ ajoute CI workflows `cargo-audit`,
  `pip-audit`, `npm audit --audit-level=moderate` bloquants sur
  PR.

### R3 — Rate limiting absent sur deploy-from-repo (severite M)

- **Asset** : A6 (project archives) + bande passante locale
- **Adversaire** : AD1 si le bearer leak
- **Impact** : abuse du clone (500 MB x N repos) → DoS disque.
- **Mitigation residuelle Sprint 16** : cap 500 MB + timeout 30s
  par clone (Sprint 14), bearer loopback empeche un web externe.
- **Roadmap** : Sprint 17+ ajoute rate limit N cloning/min par
  node_id + circuit-breaker si ratio echec eleve.

### R4 — Watcher consent.json race (severite M)

- **Asset** : A4 (consent.json)
- **Adversaire** : AD2
- **Impact** : malware ecrit un consent L4 + cap infini,
  worker accepte toute task publique pendant 50 ms.
- **Mitigation residuelle** : le write est atomic ; le malware
  peut ecrire mais il perd son effet des que l'UI re-save
  correctement.
- **Roadmap** : Sprint 17+ signe consent.json avec la keypair du
  node (self-HMAC) — rejette les ecritures non-signees.

### R5 — Kudos ledger linkability (severite M RGPD)

- **Asset** : A7 (kudos ledger)
- **Angle** : LINDDUN Linkability
- **Impact** : un peer curieux peut correler les contributions
  d'un worker sur plusieurs projets.
- **Mitigation residuelle** : aucune. Le ledger est public par
  design (audit trail).
- **Roadmap** : v2+ introduit `contribution_id` rotatable par
  projet (pattern BOINC CPID reversible), optionnellement le
  worker peut choisir "anonyme par defaut".

### R6 — Gossip detectability (severite L, accepte)

- **Asset** : A6 + annonces publiques
- **Angle** : LINDDUN Detectability
- **Impact** : un peer sait qui annonce quoi.
- **Decision** : accepte. Inherent au modele P2P public.

---

## 9. Residual risks per-configuration

Les risques residuels §8 s'appliquent uniformement. Cette section
decompose l'exposition reelle **par configuration active** : le
choix de consent level, de trust tier, de duress mode, de rate-limit
policy, de guardrails toggle et de capability gate change la surface
d'attaque effective. L'auditeur peut ainsi evaluer la posture de
securite d'un user L1 vs L4 sans ambiguite.

### 9.1 Consent GPU 4 niveaux (Sprint 16 Phase C)

Le dialog `GpuConsentDialog.tsx` offre 4 niveaux. Le worker
(`nexus-worker-core::consent::should_accept_task`) enforce les caps
et le niveau selectionne.

| Niveau | Surface exposee | Residuals §8 actifs | Impact incremental |
|---|---|---|---|
| **L1** — Mes projets uniquement | Zero exposition tierce. Le worker n'accepte que les tasks dont le `project_id` == `own_node_id`. | R1, R4, R6 (baseline) | Aucun — posture la plus securisee. |
| **L2** — Open source verifies | Apps deployees depuis un repo public, provenance SLSA L1 + Keyoxide Ed25519. | R1, R2, R4, R5, R6 | +R2 supply chain (code tiers compile et execute) +R5 kudos linkability (contribution cross-projets) |
| **L3** — Whitelist manuelle | Apps selectionnees explicitement par l'user. Pas de verification provenance automatique. | R1, R2, R3, R4, R5, R6 | +R3 rate-limit (user peut whitelister un projet abusif) |
| **L4** — Tous les projets publics | Toute app acceptee par au moins un curator souscrit. | R1, R2, R3, R4, R5, R6 | Exposition maximale. R2 amplifie (plus de code tiers). R3 amplifie (plus de projets potentiellement abusifs). |

**Annotation in-product** : `consent.json` porte un champ
`level_threat_note` (texte court, tooltip UI) et
`residual_threats_acknowledged` (liste threats §8 actifs pour le
niveau choisi). Le coordinator les calcule au `GET /consent/get` et
au `POST /consent/set`. Cf. `LOOPBACK_ENDPOINTS_TRUST_TIERS.md` §4.

### 9.2 Loopback 3 trust tiers (Sprint 22 Phase F)

Cf. `LOOPBACK_ENDPOINTS_TRUST_TIERS.md`. 3 tiers de confiance
loopback, configures par le host, pas par l'user final.

| Tier | Surface | Residuals §8 actifs | Impact |
|---|---|---|---|
| **AUTO** | Requetes loopback acceptees si bearer + Host + Origin valides. Default. | R1 (keypair plaintext = bearer exposable via AD2) | Si bearer leak via AD2 : acces complet API loopback. |
| **CONFIRM_PROMPT** | Requetes sensibles (deploy, consent change) exigent confirmation UI. | R1 (attenue — AD2 ne peut pas confirmer sans UI) | Elevation via AD2 bloquee pour les actions destructrices. |
| **BIOMETRIC_GATE** | Actions critiques (key export, consent L4) exigent biometrie OS. | R1 (quasi-elimine — keypair inaccessible sans biometrie) | Posture la plus securisee. Prerequis : LT-4 (post-v1.0). |

### 9.3 Duress PIN (Sprint 20 Phase B)

Le duress PIN permet a un user sous contrainte d'activer un mode
degrade qui detruit les cles et envoie un signal canari silent.

| Mode | Comportement | Residuals | Impact |
|---|---|---|---|
| **Normal** | Operations standard. Keypair active. | R1 (keypair accessible) | Posture courante. |
| **Duress** | Keypair detruite, canari emis, sessions invalidees. | Aucun residual (autodestruction) | Donnees perdues, identite P2P revelee. |

Limites : le duress mode est une defense against AD2 (malware
user-mode) et scenario coercition physique. Il ne protege pas
contre AD7 (nation-state root access).

### 9.4 Rate-limit tiers (Sprint 22 Phase A)

Le rate-limit engine (`governor 0.10.2` GCRA) gate les claims
worker-side. Configuration via `rate_limit_policy.toml`.

| Tier | Config | Residuals | Impact |
|---|---|---|---|
| **Disabled** (`policy: none`) | Aucun rate limit. Worker accepte au max throughput. | R3 amplifie (DoS disque via flood claims) | Deconseille en production L3/L4. |
| **Default** (`policy: default`) | 10 claims/min, burst 20. | R3 attenue | Posture recommandee. |
| **Strict** (`policy: strict`) | 2 claims/min, burst 5. | R3 quasi-elimine | Pour nodes a bande passante limitee. |

### 9.5 Pipeline guardrails disabled combos (Sprint 23 Phase B)

Le pipeline declaratif (`GUARDRAILS_ARCHITECTURE.md`) chaine les
guardrails : PII filter, watermark, output filter, rate limit.
Chaque guardrail est togglable.

> **Status** : output filter designed Sprint 23, wire end-to-end
> target Sprint 31. PII filter + watermark + rate limit sont wired
> (S21-S24). Output filter reste design-only (carry P2-REVIEW-B-2).

| Combo desactive | Risque residuel | Impact |
|---|---|---|
| PII filter OFF | Donnees personnelles dans les outputs ne sont pas detectees. | R5 linkability amplifie + RGPD Art.25 non-conformite. |
| Watermark OFF | Outputs non-traces. Repudiation complete. | R6 detectabilite perdue, pas d'audit trail output. |
| Output filter OFF | Contenu non filtre (toxicite, CSAM potential). | Reputation reseau, risque legal operateur. |
| **Tous OFF** | Zero defense-in-depth. Worker = proxy GPU brut. | Posture de securite pre-S21 (inacceptable post-Gate 3). |

### 9.6 Capability toggles (Sprint 25 Phase D)

Capabilities gate-off-by-default via `capabilities.toml` et le
binaire `nexus-admin`. Cf. `CAPABILITY_TOGGLES.md`.

| Capability | Default | Si active sans prerequis | Residual |
|---|---|---|---|
| `compute.gpu` | OFF | Worker accepte tasks GPU sans consent dialog complete. | R4 race (consent non enforce). |
| `network.gossip` | ON | — | R6 detectabilite (inherent). |
| `deploy.verified` | ON | — | R2 supply chain (inherent avec code tiers). |
| `deploy.unverified` | OFF | Accepte du code non-verifie SLSA. | R2 amplifie (aucune provenance). |
| `admin.key_export` | OFF | Export keypair Ed25519 en clair via CLI. | R1 critique (keypair exposable). |
| `admin.consent_override` | OFF | Bypass consent dialog programmatiquement. | R4 amplifie (consent race sans UI gate). |

---

## 10. Feed surface (Sprint 66 Phase B)

Le feed public (`public_feed.rs`, spec `PUBLIC_FEED_SPEC.md`)
expose une surface d'attaque specifique transposee ici depuis
la spec §12 Security considerations.

### T-FEED-INTEGRITY — Feed integrity tampering

Un attaquant modifie une entry feed en transit ou au repos.
Mitigation : chaine de hash BLAKE3 + signature Ed25519 sur
chaque entry. Le tampering est detectable a la verification
(`verify_entry`). Ref spec §4, §10.2.

| Dimension | Valeur |
|---|---|
| Severite | H |
| Likelihood | M (transport iroh-docs untrusted) |
| Mitigation | BLAKE3 hash-chain + Ed25519 signature |
| Residual | Nil (cryptographic guarantee) |

### T-FEED-SPAM — Feed spam / rate-limit bypass

Un attaquant flood le feed avec des operations pour epuiser le
stockage ou noyer les entries legitimes. Mitigation : rate
limiter GCRA per-author (5 ops/min, spec §10.1 #1), payload
size limit (64 KB, spec §10.1 #2), PoW optionnel 16-bit.

| Dimension | Valeur |
|---|---|
| Severite | M |
| Likelihood | M (open network) |
| Mitigation | GCRA 5 ops/min + 64 KB limit + PoW |
| Residual | L (Sybil multi-keypair, cf. T-FEED-4) |

### T-FEED-FORGERY — Cross-author forgery

Un attaquant publie des entries sous l'identite d'un autre
auteur. Mitigation : verification Ed25519 de la signature
contre le `author_pubkey` declare (spec §10.1 #6, §10.2 #7).

| Dimension | Valeur |
|---|---|
| Severite | H |
| Likelihood | L (requires Ed25519 break) |
| Mitigation | Ed25519 signature verification |
| Residual | Nil (cryptographic guarantee) |

### T-FEED-CLOCK-SKEW — Clock skew manipulation

Un attaquant place des timestamps far-future pour manipuler
l'ordre ou la detection de staleness. Mitigation : gate 30
jours futur (spec §10.2 #10).

| Dimension | Valeur |
|---|---|
| Severite | M |
| Likelihood | L (detectable, limited impact) |
| Mitigation | 30-day future timestamp gate |
| Residual | L (past timestamps accepted, ordering by seq) |

### Residual risks feed

- **Pas de resistance Sybil** tant que `CuratorVouched` n'est
  pas implemente (Sprint 67+). Tout keypair Ed25519 peut etre
  auteur.
- **Pas de quarantine feed** par auteur suspect (Sprint 67+).
- **Pas de feed-level revocation** — une entry publiee ne peut
  pas etre retiree du log append-only (by design).

---

## 11. Revue et evolution

Ce document est vivant. Chaque sprint qui livre une mitigation
ou deplace un residual doit :

1. Mettre a jour §7 (table mitigations) avec le commit hash.
2. Mettre a jour §8 (residuals) — retirer ou reduire la
   severite.
3. Si nouveau composant : ajouter §5.x STRIDE + §6 LINDDUN
   + ligne dans §2 Assets + §4 DFD.
4. §9 per-configuration : ajouter une sous-section si un
   nouveau mode ou toggle change la surface d'attaque.
5. En fin de sprint, le verification.md du sprint pointe vers
   les lignes modifiees ici.

Historique versions :

- **v1 (Sprint 16 Phase E, 2026-04-14)** : version initiale post-
  livraison des 4 phases hardening A-D. Baseline pour Sprint 17+.
- **v2 (Sprint 29 Phase B, 2026-04-26)** : ajout §9 residual risks
  per-configuration (6 sous-sections), renommage §9→§10.
- **v3 (Sprint 66 Phase B, 2026-05-19)** : ajout §10 Feed surface
  (T-FEED-1..T-FEED-4), renommage §10→§11.
