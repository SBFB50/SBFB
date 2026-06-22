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
| E | RCE via deserialization iroh | C | Version pinnee 0.98 (upgrade Sprint 32, Day 0 #3 leve) + `cargo-audit` | **M** |

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
niveau choisi). Le coordinator les calcule au `GET /api/v1/consent` et
au `POST /api/v1/consent/set`. Cf. `LOOPBACK_ENDPOINTS_TRUST_TIERS.md` §4.

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

- **CuratorVouched implemente Sprint 67 Phase A** — le feed
  supporte les endorsements curator. Pas encore de quorum
  multi-curator requis (S68+).
- **Pas de quarantine feed** par auteur suspect (Sprint 68+).
- **Pas de feed-level revocation** — une entry publiee ne peut
  pas etre retiree du log append-only (by design).

---

## 11. Search surface (Sprint 67 Phase B)

Le search FTS5 local expose une surface d'attaque specifique.
Le search_index est un virtual table SQLite FTS5 peuple depuis
les feed entries et les browse entries. Toutes les queries sont
parametrisees — pas de SQL dynamique.

### T-SEARCH-INJECTION — FTS5 query syntax injection

Un attaquant soumet une query crafted contenant des operateurs
FTS5 (`OR`, `AND`, `"`, `*`, `NEAR`) pour contourner le scoring
ou provoquer un crash. Mitigation : `sanitize_query()` wrappe
chaque token dans des double-quotes et escape les `"` internes.
Les NUL bytes sont strippes avant indexation et avant query.

| Dimension | Valeur |
|---|---|
| Severite | M |
| Likelihood | M (public endpoint derriere bearer auth) |
| Mitigation | Token quoting + NUL strip + parameterized SQL |
| Residual | L (pathological query performance sur corpus > 50K) |

### T-CURATOR-VOUCH — Endorsement spam via feed

Un attaquant publie un grand nombre de CuratorVouched operations
pour gonfler la visibilite d'un projet dans le search index ou
le browse. Mitigation : rate limiter GCRA existant (5 ops/min
per-author, T-FEED-SPAM), chaque vouch est signe Ed25519 donc
attributable. Le search index est reindexe a chaud a l'ingest
(Sprint 73 Phase C, apres les gates dedup + rate-limit) et reste
reconstructible au boot — les entries spam admises restent
visibles mais attribuables et bornees par le rate limiter.

| Dimension | Valeur |
|---|---|
| Severite | M |
| Likelihood | M (open network, rate limit contournable par Sybil) |
| Mitigation | GCRA 5 ops/min + Ed25519 attribution + hot/boot reindex |
| Residual | L (Sybil multi-keypair, cf. T-FEED-SPAM) |

### T-SEARCH-DOS — Search endpoint rate exhaustion

Un attaquant flood le endpoint GET /api/daemon/search pour
epuiser les ressources CPU/IO du daemon. Mitigation : le search
est local (pas de network round-trip), le bearer token protege
le endpoint (pas d'acces anonyme), et le corpus est < 500 entries
pre-launch. Limite future : rate limiter per-client sur les
endpoints search (S68+).

| Dimension | Valeur |
|---|---|
| Severite | L |
| Likelihood | L (bearer auth + local-only) |
| Mitigation | Bearer auth + small corpus + FTS5 O(1) index lookup |
| Residual | L (pas de rate limit per-client, acceptable pre-launch) |

**D.1 recadrage (carry S73)** : le residual n'est PAS une course
de debounce ni un vecteur multi-client — la surface est
**loopback single-user** : seul un process du meme compte local,
deja porteur du bearer (`/auth/token` same-origin), peut flood, et
ce process a deja des moyens plus directs (§5.7). La mitigation
durable retenue n'est donc pas un rate-limit reseau mais un **clamp
de `q` (longueur) et `offset` (borne haute)** cote handler, pour
borner le cout d'une requete pathologique unique ; differe S75
(`SEARCH-VIEW`/clamp, faible priorite pre-launch).

### Closure P2-THREAT-MODEL-FEED-SURFACE 3/3

Sprint 66 Phase B a livre 2/3 (T-FEED-1..4). Sprint 67 Phase B
complete 3/3 avec T-SEARCH-INJECTION, T-CURATOR-VOUCH, et
T-SEARCH-DOS. Le carry P2-THREAT-MODEL-FEED-SURFACE est
**FERME**.

---

## 12. ProofCard surface (Sprint 68 Phase D)

La ProofCard est un artefact **local compute** — le daemon assemble
les donnees qu'il possede deja (browse entry, provenance record,
curator lists, feed entries) et produit un score d'evidence-completeness
deterministe 0-100. Elle n'est PAS un wire format signe et ne
transite PAS par `canonical_bytes`. Le score est affiche dans le
shell Browse via un composant React expandable.

### T-PROOFCARD-FORMULA-GAME — Score gaming sans substance

Un attaquant optimise les metadonnees de son projet pour maximiser
le score ProofCard sans fournir de substance reelle. Vecteurs :

1. **Provenance factice** : generer une provenance auto-attestee
   (Ed25519 self-sign) pointant vers un repo vide ou un commit
   trivial. Le score accorde +20 pour provenance verified.
2. **Curator collusion** : creer plusieurs curator keypairs et
   publier des CuratorVouched mutuels (Sybil). Le score accorde
   +10 pour >= 1 curator et +10 pour >= 3 curators.
3. **License tag gaming** : declarer une licence SPDX dans le
   manifest sans que le code source soit reellement sous cette
   licence. Le score accorde +5 pour licence presente.
4. **Freshness gaming** : re-deployer periodiquement sans
   changement reel pour maintenir le tag "fresh" (+10).

Mitigations :

- La provenance est verifiable : quiconque peut cloner le repo,
  rebuilder l'archive, et comparer le hash. Un repo vide ou un
  commit trivial est detectable par inspection humaine.
- La collusion curator est limitee par le GCRA rate limiter
  (T-FEED-SPAM, 5 ops/min per-author) et l'attribution Ed25519.
  Le score ne depasse pas +20 meme avec 100 curators.
- La licence n'affecte que +5 du score. L'inspection source
  reste le mecanisme de confiance (source verifiable).
- Le `formula_version` (v1) est expose dans l'UI pour que les
  utilisateurs sachent quelle formule est utilisee.
- Le score est clairement presente comme "completude de preuve"
  (evidence-completeness), pas comme "securite" ou "confiance
  absolue". Le composant UI affiche les couches individuelles
  pour que l'utilisateur juge par lui-meme.

| Dimension | Valeur |
|---|---|
| Severite | M |
| Likelihood | M (auto-attestation inherente au modele P2P) |
| Mitigation | Score capped, evidence decomposee, provenance verifiable, curator attribution |
| Residual | M (Sybil multi-keypair reste possible sans quorum externe) |

---

## 13. Preview ephemere surface (Sprint 69 Phase A)

Le `PreviewStore` du daemon heberge des archives zip chargees par
`sbfb-factory preview` pour tester une app localement avant
publication. Les previews sont ephemeres (TTL 30 min), accessibles
uniquement via loopback authentifie, et servies dans un iframe
sandbox identique aux blobs P2P.

### T-PREVIEW-EXHAUSTION — Memory exhaustion via preview flooding

Un attaquant local (ou un script malveillant sur la meme machine)
charge des previews en boucle pour epuiser la memoire du daemon.

Vecteurs :

1. **Volume d'entries** : charger des previews distincts en rafale.
   Chaque entry est limitee a 10 MB (`MAX_PREVIEW_BYTES`), mais sans
   cap sur le nombre d'entries le store grandit sans borne.
2. **Taille maximale** : charger des entries de 10 MB chacune pour
   maximiser l'impact memoire par entry.

Mitigations :

- `MAX_PREVIEW_BYTES = 10 MB` par entry (Sprint 68 Phase B).
- `MAX_PREVIEW_ENTRIES = 10` entries simultanees (Sprint 69 Phase A).
  Le 11e load retourne `PreviewError::TooManyEntries`. L'impact
  memoire maximum est borne a 10 * 10 MB = 100 MB.
- TTL 30 min avec eviction automatique (`evict_expired`).
- Loopback-only : le endpoint `/api/v1/preview/load` est accessible
  uniquement via localhost avec bearer token. Un attaquant distant ne
  peut pas charger de previews.
- Bearer token authentification : meme un processus local doit
  connaitre le token genere au demarrage du daemon.

| Dimension | Valeur |
|---|---|
| Severite | L (impact borne a 100 MB, loopback-only) |
| Likelihood | L (requiert acces loopback + bearer token) |
| Mitigation | MAX_PREVIEW_BYTES + MAX_PREVIEW_ENTRIES + TTL + auth |
| Residual | L (attaquant local avec bearer = compromission machine deja) |

---

## 14. Operator surface (Sprint 72 Phase A)

Le **Factory Operator** (`crates/sbfb-factory/src/operator_server.rs`,
port `:3001` par defaut) est un serveur HTTP loopback **distinct du
daemon** : process separe, **TCP loopback uniquement** (pas de UDS /
peer-creds — un sous-ensemble token+Host+Origin du modele S16). Il
**ecrit des fichiers** (`POST /api/artifacts/draft`) et **spawn des
sous-processus agent** (`claude --permission-mode bypassPermissions`)
via le stream chat SSE (`GET /api/chat/{id}/stream`). Ces deux
capacites — write disque + spawn de processus autonome — en font une
surface critique au meme titre que les endpoints write du daemon.

Le bloc off-sprint qui a livre l'Operator l'avait expose avec **CORS
`Any` et zero auth** (G7/P1) et un **stream SSE qui contournait le gate
`SENSITIVE_ACTIONS`** que les endpoints JSON appliquaient deja (G2/P0).
Sprint 71 Phase C (`a0337c6`) l'a ramene sous le modele loopback du
daemon. Ce catalogue documente la surface a posteriori (P2-H-1, audit
S71 Track H — la defense etait livree+testee, seul le threat model
accusait le retard). Ref defense complete : `docs/shell/PATTERNS.md
§P35` + `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md §3.1`.

### T-OPERATOR-CSRF — CSRF / DNS-rebinding sur surface write + spawn

Un site web malveillant ouvert dans le navigateur de l'utilisateur (ou
une resolution DNS-rebinding pointant vers `127.0.0.1:3001`) tente de
declencher un write artefact ou un spawn agent en forgeant des requetes
vers l'Operator. Meme vecteur que CVE-2025-49596 (cf. §5.5 loopback).

Mitigation (S71 G7, `a0337c6`) : middleware `auth_required`
(`auth.rs:229`) applique sur chaque route data-bearing —
(1) `X-SBFB-Token` bearer per-boot compare en `constant_time_eq` (401
sinon) ; (2) header `Host:` doit etre loopback (403 sinon) ; (3) header
`Origin:` doit etre loopback ou absent (403 sinon) ; (4) `CorsLayer`
epingle a `is_loopback_origin` (`operator_server.rs:103`, plus de
`allow_origin(Any)`). Un navigateur tiers ne connait pas le token et ne
peut forger un `Origin` loopback.

| Dimension | Valeur |
|---|---|
| Severite | H (write disque + spawn bypassPermissions) |
| Likelihood | L (token bearer 256-bit + Host/Origin loopback bloque le navigateur) |
| Mitigation | `X-SBFB-Token` (constant_time_eq) + Host + Origin + CORS epingle (S71 G7) |
| Residual | Processus local hostile lisant `~/.sbfb/auth_token` (frontiere OS-sandbox, accepte — cf. AD2 « abuse de auth_token » / §5.7, meme modele que daemon loopback) |

### T-OPERATOR-SPAWN — Spawn agent autonome non gate

Le stream chat SSE spawn un agent `claude --permission-mode
bypassPermissions`. Un message portant une action sensible
(`shell` / `commit` / `push` / `PASS`) pourrait declencher une action
irreversible (commit, push, shell arbitraire) sans confirmation.

Mitigation (S71 G2, `a0337c6`) : `handle_chat_stream`
(`operator_server.rs:822`) applique le **meme** filtre `SENSITIVE_ACTIONS`
(`const` ligne 34 : `shell`/`commit`/`push`/`PASS`) que les endpoints
JSON, **AVANT** le spawn (gate `:866`, spawn `:898`). Un dernier message
sensible retourne `requires_gate` au lieu de spawner. `bypassPermissions`
est **conserve** (PO-2 : le mode « prompt de base + discussion agent
autonome » est un contrat, pas un bug) mais jamais sur un chemin non
gate. Les messages non-sensibles streament normalement.

| Dimension | Valeur |
|---|---|
| Severite | H (action irreversible : commit/push/shell autonome) |
| Likelihood | L (gate `SENSITIVE_ACTIONS` avant spawn ; requiert deja le token loopback) |
| Mitigation | Gate `SENSITIVE_ACTIONS` dans `handle_chat_stream` avant `spawn_claude_stream` (S71 G2) + timeout/diagnostic spawn (S71 G12) |
| Residual | Gate **keyword-based** (`shell`/`commit`/`push`/`PASS`), pas capability-based : un prompt qui declenche une action destructive sans ces mots-cles n'est pas gate. Le perimetre de l'agent spawn = les privileges user-mode du process (pas un sandbox strict repo). Mitigation residuelle = `bypassPermissions` reste un contrat utilisateur explicite (PO-2), pas une exposition reseau. Renforcement capability-based = candidat futur (cf. carry S73) |

### Anticipation NetworkProvider (Sprint 72 ProviderRouter)

Le ProviderRouter S72 (`provider_router.rs`, bras `Network`) est un
**client sortant** de `POST /api/v1/tasks/submit` puis
`GET /api/v1/tasks/{id}` / `GET /api/v1/tasks/{id}/result` (daemon
loopback, tier T0, deja inventorie LOOPBACK §3) — **pas une nouvelle
surface entrante** sur l'Operator. Le dispatch reseau reste dans la
frontiere loopback durcie. Le gate `SENSITIVE_ACTIONS` reste applique
AVANT le dispatch quel que soit le provider selectionne (Claude /
Ollama / Network) — l'invariant gate-avant-dispatch (S72 Phase D)
preserve la mitigation T-OPERATOR-SPAWN sur tous les chemins.

**Nouvelle route de lecture `GET /api/v1/tasks/{id}/result` (S72 Phase D,
option A)** : pour rendre une reponse reseau dans le chat, le daemon
expose le `result_text` accepte d'une tache `completed`. C'est une route
**daemon entrante** (pas Operator) — tier T0 loopback, **lecture seule**,
sous le meme middleware `auth_required` (X-SBFB-Token + Host + Origin)
que le reste de l'API tasks, aucun nouveau tier de confiance, aucun spawn.
Le `result_text` ne devient `completed`/lisible **qu'apres** passage du
guardrail de sortie. Sur les **deux** chemins d'ingestion d'un resultat —
HTTP `coordinator_submit_result` et la boucle gossip `validator_loop` —
le `default_output_chain` tourne AVANT `set_task_result` (Sprint 73 Phase
A, D5 : split `validate_result_pre_guardrail` → guardrail →
`validate_result_post_guardrail`). Un texte qui declenche un tripwire
n'est **jamais persiste** (aucune ligne `completed`, rien a relire) et ne
credite aucun kudos. La route ne peut donc relire qu'un texte deja filtre.
Delta menace minimal : un lecteur loopback authentifie obtient le texte
d'une tache qu'il pouvait deja voir par `result_hash`.

### Residual risks Operator

- **Token bearer en clair sur disque** (`~/.sbfb/auth_token`) — un
  processus local du meme utilisateur peut le lire (frontiere
  OS-sandbox acceptee, identique au daemon loopback §5.7). Pas de gate
  T1/T2 (CONFIRM_PROMPT / BIOMETRIC) sur l'Operator a ce stade ; les
  actions vraiment destructives passent par le gate `SENSITIVE_ACTIONS`
  cote chat, pas par un tier biometrique OS.
- **Pas de UDS / peer-creds** — l'Operator est TCP loopback only,
  contrairement au daemon (UDS SO_PEERCRED / Named Pipe SDDL, §5.5
  menace I). Le scenario « autre process du meme user local » (§5.7
  menace E) n'est donc pas filtre par peer-creds cote Operator ; la
  mitigation repose sur le token bearer per-boot (`X-SBFB-Token`).

---

## 15. Surface seed cross-noeud (Sprint 74)

Le programme « Disponibilite » (Arc 3.5, ex-LT-5 tire en avant)
ajoute trois primitives de seed cross-noeud : (E) un protocole
authentifie `SeedRequest` sur ALPN dedie `sbfb/seed/0` (invite
revocable liee a la paire `(project_id, archive_hash)`, M19), (E) un
seed VOLONTAIRE communautaire (un noeud fetch+pin une app publique
distante, sans approbation auteur — sur par content-addressing), et
(F) une operation de feed `SeedAnnounced` + un registre best-effort
en memoire qui agrege « Toi + N pairs (vus recemment) ».

**Invariant cardinal** : *seeder != auteur*. Un seeder signe une
revendication de seed (sa propre annonce), JAMAIS la provenance de
l'app. Le content-addressing BLAKE3 reste la VERITE de joignabilite :
une annonce forgee ne permet jamais de servir des octets qu'on ne
detient pas (le fetch verifie le hash, rejette en cas de mismatch).
Le compteur peut donc SUR-estimer, mais ne ment jamais sur la
joignabilite reelle (la sonde ETAT est l'autorite, pas le compteur).

| Menace | Exemple | Sev. brute | Mitigation (file:line) | res |
|---|---|:---:|---|:---:|
| S | `SeedAnnounced` forge (mauvaise sig) | H | FeedEntry Ed25519 `verify_entry` + PoW (`FEED_POW_DIFFICULTY=16`) a l'ingest (`feed_sync.rs ingest_doc_entry`) | **Nil** |
| S | Impersonation (annoncer le seed d'un AUTRE noeud) | M | `record_announced` exige `seeder_node_id == FeedEntry.author_pubkey` (`seed_registry.rs`) — un noeud n'annonce que SON propre seed | **Nil** |
| T | Capability-over-content (invite redeemed pour contenu etranger) | H | Invite liee a la paire `(project_id, archive_hash)` (M19) ; `consume_seed_invite` verifie la paire ; mint derive l'archive_hash du browse local | **Nil** |
| I | Re-attribution d'auteur (R5) | H | `seeder_node_id` distinct de l'auteur app ; le seeder ne re-signe jamais la provenance (Radicle delegate != seeder) | **Nil** |
| D | Sur-comptage / Sybil (faux « je seed X ») | M | best-effort par design (Q5) + content-addressing = verite joignabilite + PoW feed + pilote ferme ; registre reseau-large (SearchManifest) DIFFERE (scope cut #10) pour eviter la surface broadcast-Sybil (D3) | **M** |
| D | Croissance non-bornee du registre / feed | M | registre : TTL 48h purge paresseuse + sweep global auto-cadence (`SEED_SWEEP_INTERVAL_SECS`, `seed_registry.rs`) ; feed : re-annonce reprovide best-effort bornee au pilote (cout assume, modele IPFS reprovide) | **L** |
| E | Anti-replay `SeedRequest` (rejouer une requete capturee) | M | nonce + fenetre temporelle, `NonceCache` TTL `2*window+1` (`seed_protocol.rs`, Phase E Codex C3) | **L** |

**Residual** : le sur-comptage (D, **M**) est un cout assume du
compteur best-effort pilote-ferme. Le registre reseau-large signe
(SearchManifest) est un scope cut explicite (#10/D3) precisement pour
ne pas ouvrir la surface broadcast-Sybil avant un design noeud-index
opt-in (PO-13, post-launch). Carry : re-credit d'une invite single-use
brulee sur un fetch transitoire (Phase E P3, S75).

### 15.1 Extension Sprint 75 — decouverte PULL node-centrique

S75 pivote la decouverte de PUSH-ephemere vers PULL node-centrique :
`NodeDirectoryEntry` signe (`DOMAIN_NODE_DIRECTORY_V1`, machinerie
CuratorList reutilisee, ingest subscription-gated), locator persiste
`anchors.json {pubkey, ticket, revision}` re-valide signature+revision
au re-pull (floor anti-rollback durable), pull multi-provider
`fetch_hash_multi` (ancre d'abord puis seeders `SeedRegistry`), routes
additives `GET /api/daemon/nodes` + `POST /api/daemon/seed/request`,
driver seed boot config-driven (`[seed] keep_online_projects`), front
`/nodes` + `/node/:id`. Rows deferes des phases D/E/F, consolides ici
(Phase G).

| Menace | Exemple | Sev. brute | Mitigation (file:line) | res |
|---|---|:---:|---|:---:|
| I/D | **Oracle blob-serve drive-by + amplification de dials** : un `GET /blob-serve/{hash}` sur un hash absent declenche le 4e tier directory-only → dials sortants vers ancre+seeders (observation du graphe, amplification) | M | resolution UNIQUEMENT sur annuaires ABONNES (`verrou 5`, attention-set explicite) ; cap `MAX_FETCH_PROVIDERS=16` enforce DANS la primitive (`blobs.rs fetch_hash_multi`) ; timeout appelant. **S76 B8 (THREAT-BLOBSERVE-BEARER) : `/blob-serve` est PUBLIQUE par construction (`public_routes` http.rs:248-255 — SANS bearer/Host/Origin, car un iframe sandboxe `allow-scripts` sans `allow-same-origin` ne peut pas porter le bearer pour charger ses assets) ; l'amplification est bornee par le subscribed-only + le cap + le timeout, JAMAIS par un bearer. La revendication anterieure « loopback bearer requis sur la route » etait fausse — corrigee.** | **L** |
| I | **Inventaire /nodes** : enumeration des catalogues connus du noeud | L | loopback bearer ; contenu = annuaires signes deja publics par construction ; route additive, `/browse` byte-identique | **Nil** |
| S/D | **Timestamp futur dans `SeedAnnounced`** (monopoliser la fraicheur du registre) | M | SEED-1 : clamp `seen_at = min(seen_at, now)` DANS `SeedRegistry::record` (pas une convention d'appelant) | **Nil** |
| D | **Gonflement du registre seeders** (buckets/slots illimites, variantes de casse d'une meme pubkey) | M | SEED-2 : double cap 1024 buckets / 64 seeders + eviction stalest-si-newcomer-plus-frais ; normalisation hex lowercase write+read (2^64 variantes de casse = 1 slot) | **L** |
| D | **Fresh-flood displacement** : annonces continues fraiches evincent les vrais seeders du registre cappe | M | residuel assume best-effort (doc `MAX_REGISTRY_BUCKETS`) : le compteur n'est jamais l'autorite, la sonde live + BLAKE3 le sont ; sampling anti-Sybil du tail route audit S76 | **M** |
| E/T | **Boot seed driver** : config `[seed]` rejouee sous identite duress ; annuaire divergent ecrasant le pin local | M | duress short-circuit EN TETE du driver (`http.rs run_boot_seed_driver`) ; resolution direct > row M18 > annuaires FIGEE par test ; clamp lowercase-64-hex au load ; defaut compile VIDE (verrou 3 tripwire) | **L** |
| E/D | **Boot feed-emit sous duress** (audit S75 DURESS-BOOT-LEAK, P1 ferme) : `reannounce_seeds_at_boot` (re-annonce les lignes `keep_online` REELLES) et la republication feed S66 (`replay_all` + orphan recovery — rejoue l'INTEGRALITE du feed reel vers iroh-docs) emettent sous la cle au boot ; NON gardes avant le fix, ils correlaient l'identite leurre au vrai data root a CHAQUE boot, zero interaction utilisateur | H | duress short-circuit EN TETE des DEUX chemins (`gossip_publish_in_duress(identity_mode) == Noop`, miroir du driver) : un noeud leurre n'emet AUCUN `SeedAnnounced` ni feed reel sur le reseau ; tests `reannounce_seeds_noop_in_duress` (feed iroh-docs vide sous duress) + primitif `duress_mode_noop_publishes` | **Nil** |
| E | **Requester route `/seed/request`** : self-designation, replay, mint sans detention | M | invite M19 TOUJOURS requise ; self-guard sur identites PARSEES (anti-base32) ; mint frais gate-detention 409 ; echo nonce verifie ; timeout 120s documente (504 ≠ echec, invite consommee AVANT fetch) | **L** |
| E | **Surfaces front F sans duress gate** : `seed_voluntary` + `set_keep_online` exposes par l'UI /nodes-/node/:id alors que leurs handlers PRE-EXISTANTS (S74) n'etaient pas duress-gates | M | **S76 B1 (DURESS-FRERES-LOCAL, FERME) : les DEUX handlers court-circuitent EN TETE en duress (`gossip_publish_in_duress(identity_mode) == Noop` → early-return + reponse leurre benigne, miroir du driver) → ZERO mutation du vrai data root (pas de row M18, pas de tag blob) ET ZERO emit `SeedAnnounced` sous la cle leurre (le seul early-return couvre le pin local ET l'emit). Tests `seed_voluntary_noop_in_duress` + `set_keep_online_noop_in_duress` (zero ecriture, zero tag).** Le lot duress freres LOCAL-ONLY est clos ; tous les chemins boot wire-emit l'etaient deja (audit S75 DURESS-BOOT-LEAK, row ci-dessus) | **Nil** |
| S/D | **Registre observed RAM (UX-ARRIVAL post-S75)** : ingest non-sollicite de metadonnees d'annuaire (pubkey entendue sur gossip sans abonnement) — flood Sybil de pubkeys forgees, spam de re-publications d'une identite, usurpation visuelle d'un node_id dans /nodes | M | NO-FETCH/NO-DIAL absolu pour un non-abonne (la metadata = enveloppe gossip seule, jamais le blob — l'amplification BitTorrent-DRDoS/libp2p-PR#577 n'est pas introduite) ; bornes DANS la primitive (`iroh_runtime.rs record_observed_directory`) : cap 256 + eviction stalest, TTL 48h, rate-limit 1/min par identite RECLAMEE (exigence PO) ; self-guard (notre node_id n'est jamais observe, ni par echo ni par forge). Residuels assumes (review UX-OBS-RATELIMIT-UNAUTH) : (1) le champ `node` de l'enveloppe est NON authentifie et le PoW est lie a (publisher, topic), PAS au payload — un seul PoW couvre N annonces de pubkeys forgees distinctes : le rate-limit borne la churn PAR identite et le cap borne la taille, mais rien ne tarife les identites forgees une a une (classe fresh-flood SeedRegistry). **S76 B1 — decision (b) : le registre observed est AVAILABILITY-ONLY, non publisher-authentifie, par construction (un indice de joignabilite, jamais une attestation). Lier la capture au PoW publisher exigerait l'auteur d'enveloppe VERIFIE a la couche `process_directory_announcement_bytes` (le call-site ne passe que content+node) — durcissement differe, non sur-promis ; la vraie defense anti-flood reste le self-guard + l'exclusion des abonnes + le registre borne/rate-limite (test `observed_capture_is_availability_only`).** ; l'etat du limiteur EST l'entree residente — une identite evincee par le cap est re-acceptee immediatement si elle re-annonce, mais s'evincer exige d'etre la stalest du registre entier (256 identites plus fraiches = deja le regime flood ci-dessus) : UNE identite ne peut pas s'auto-churner, et hors flood l'entree ne sort que par le TTL 48h >> 60s ; (2) l'identite observed n'est PAS Ed25519-verifiee (le blob n'est pas fetche) — non-autoritaire par construction : la seule action offerte est un subscribe explicite, et un node_id usurpe ne produit aucun catalogue (ligne « en attente » honnete) | **L** |
| S | **Spoof du placement « Tes sources » via `ProjectAnnouncement.node_id` (UX-ARRIVAL)** : l'annonce direct ne porte AUCUNE signature — un annonceur (un PoW) peut nommer la pubkey d'une ancre publique abonnee et viser le placement de confiance de la grille (et le hero « En vedette ») | H | `from_subscribed` est CATALOG-BACKED, jamais derive de la seule appartenance du node_id reclame a l'attention set (`http.rs browse_views`) : un `direct` n'est classe « mes sources » que si sa paire (project_id, archive_hash) figure dans le catalogue Ed25519-VERIFIE de l'annuaire signe du noeud reclame — un spoofer ne peut pas inserer de row dans un catalogue signe ; les vraies apps d'un noeud abonne y sont par construction du pivot PULL (publish → revision>0 → re-annonce boot). Une entry sans archive_hash n'est jamais classee sur claim nu. Test decisif `browse_views_derives_from_subscribed` (fixture SpoofApp) | **L** |
| S | **Badge `is_open_source` spoofable a l'ingress `/browse` (S76 B2, CARRY-3)** : un `ProjectAnnouncement` gossip (non signe) peut porter `is_open_source=true` SANS `provenance_hash`/`repo_url` — le badge servi par `/browse` (et le « verrou 4 » front, qui lit `source=="direct"` + `is_open_source`) afficherait alors une fausse source-verifiable, pilotant le fork-consumer et le consent L2 worker sur une revendication forgee | M | **S76 B2 : downgrade `is_open_source`→`false` a l'INGRESS aggregator (`runtime::handle_project_announcement`, AVANT `add_direct_entry`), pas seulement a l'index FTS5 (S74 B.6) — meme predicat partage `trustworthy_open_source(is_open_source && provenance_hash.is_some() && repo_url.is_some())`. Test `aggregator_downgrades_open_source_without_provenance`. Note cardinale : le « verrou 4 » est une garantie DECLARATIVE (provenance presente), JAMAIS une attestation cryptographique que l'archive a ete batie depuis ce repo — un tiers peut verifier la provenance, pas (encore) reproduire le build.** | **L** |

**Residuals S75** : fresh-flood (M, sonde=autorite) et duress des
freres pre-existants LOCAL-ONLY (L, route S76 ; les chemins boot
wire-emit sont gates par l'audit S75 DURESS-BOOT-LEAK). Le sur-comptage §15 row D reste
M : `known_entry_count` agrege le 3e bras nodedirectory en best-effort
(sur-estimation toleree, jamais une preuve de joignabilite).

### 15.2 Extension Sprint 76 Phase D — quorum compute redundancy>1 cross-machine

S76 Phase D prouve le quorum deterministe `redundancy_factor > 1` par
composition (tests hermetiques 2-auteurs sur le VRAI bridge + validator loop +
DB, rouge-avant-vert ; la replication iroh cross-nœud est deja prouvee par
`worker_result_syncs_into_coordinator_db_across_two_nodes` en redundancy=1 ;
D3 etage 1 palier 2) ; la preuve LITTERALEMENT cross-machine (VPS + PC + Mac,
processus OS distincts) est l'acceptance LIVE Phase G. La cohorte homogene (gate Phase C,
`required_runtime`) est un **routage ADVISORY** : elle co-localise les
workers homogenes pour que l'exact-match tienne, mais n'est JAMAIS une
frontiere de confiance. La vraie defense reste le quorum exact-match a
majorite stricte (`validate_quorum_pre_guardrail`, **INCHANGE** ce phase) qui
rejette tout `result_text` divergent comme outlier. Le fix du bridge
result-sync (dedup `(worker_pubkey, task_id)` au lieu de `task_id` seul,
miroir de la cle du validator) debloque la formation du quorum cross-machine
sans deplacer cette frontiere.

| Menace | Exemple | Sev. brute | Mitigation (file:line) | res |
|---|---|:---:|---|:---:|
| S/D | **Self-inflation du quorum** (un worker soumet 2 fois pour fabriquer une majorite) | M | dedup `(worker_id, task_id)` aux DEUX couches : `insert_task_result` (validator) ET `forward_result_entry` (bridge, S76-D) — un meme `worker_pubkey` = une seule voix. Verrou `validator_quorum_unchanged` | **Nil** |
| T/D | **Sybil multi-keypair** (N identites forgees votent la meme reponse pour forcer un faux consensus) | H | inchange par le fix (le fix forwarde une voix par pubkey distincte, il n'en cree aucune) ; gonfler le quorum exige N keypairs reels = surface Sybil pre-existante (PoW / AgeWitness + pilote ferme) ; le quorum n'est PAS une frontiere anti-Sybil par lui-meme | **M** |
| T | **Worker menteur** (un GGUF/poids different ou un mensonge sur la cohorte) | M | rejet outlier exact-match (`validator.rs:290-336`) : un `result_text` divergent ne forme jamais la majorite — la cohorte advisory ne sert qu'au routage, pas a la confiance ; tests `quorum_redundancy_diverging_outputs_rejected` (bridge) + `quorum_rejects_nondeterministic_divergence` (in-DB) | **L** |
| Faux-vert | **Divergence cross-GPU lue comme un bug** | M | anti faux-vert (T1) : exact-match garanti HOMOGENE seulement (meme model/quant/runtime) ; divergence cross-GPU heterogene = ATTENDUE (float reordering, Thinking Machines/Ingonyama) et rejetee comme outlier, ECRITE comme resultat attendu dans l'acceptance LIVE (PATTERNS §P60.2) | **M** |
| I | **Verification cross-hardware semantique manquante** (l'exact-match ne couvre pas les GPU heterogenes) | L | **etage 2 primitive CABLEE (S77 Phase G)** : primitive N0 TOPLOC (`toploc.rs`, commitment BLAKE3 du sketch entier top-k du dernier hidden state) + helper worker qui le calcule + Layer-3 `verification.rs` consommant le commitment dans `logprobs_hash` par egalite (detection ~100% du swap modele/precision par INEGALITE). L'**ecriture/emission signee** du commitment dans `RunProof.activation_fingerprint` / `ResultPayload.logprobs_hash` ride le data-plane (Phase H/I/J). La comparaison TOLERANTE cross-GPU (`ToplocFingerprint::compare`) est recomputee independamment par N1 (Phase H) / N2 (Phase I) — voir §16 « N0 TOPLOC fingerprint ». **Phase H** cable la PRIMITIVE N1 (tirage verifiable Ed25519 `verifiable_draw.rs` + recompute tolerant `ToplocFingerprint::compare` + Token-DiFR + incentive reputationnel `spotcheck_creditable`/`kudos_ledger::credit` + mapping criticite→niveau `criticality_maps_to_verification_level`) ; le re-exec prefill REEL sur GPU + le transport du sketch complet hors du slot 32B restent gates (Phase I/K), comme G a livre la primitive N0 sans cablage in-vivo | **M (primitive N1 CABLEE Phase H ; re-exec prefill in-vivo + transport sketch = Phase I/K → L pour l'echantillon tire une fois in-vivo)** |

**Residual S76-D** : le Sybil multi-keypair (T/D, **M**) reste le cout
assume du quorum par accord de sortie — le quorum prouve la reproductibilite
deterministe entre voix, pas l'unicite des votants (mitige hors-quorum :
PoW/AgeWitness + pilote ferme). La divergence cross-GPU (**M**) est un cout
physique assume, rendu visible (pas masque) ; sa resorption = TOPLOC etage 2
(S77). Le fix bridge result-sync n'ajoute AUCUNE surface : il fait passer une
voix par worker distinct la ou il les collapsait sur la premiere.

### 15.3 Extension Sprint 76 Phase E — dashboard contributeur (D4)

S76 Phase E expose une **deuxieme vue de lecture** sur le ledger kudos
existant (agregation keyee `worker_node_id`, route authentifiee
`GET /api/v1/contributor/{node_id}` sous `authed_routes` =
bearer + Host + Origin loopback). La vue n'ecrit RIEN : elle agrege en lecture
des lignes deja creditees apres acceptation quorum. Deux champs self-declares
du payload signe (`tokens_generated`, `generation_time_ms`, `task.rs:476-481`)
alimentent le credit ; ils sont HORS quorum (le validator ne compare que
`result_text`). Phase E **durcit** le credit via un sanity-bound de
plausibilite (`sanity_bounded_tokens`, `kudos_ledger.rs`).

| Menace | Exemple | Sev. brute | Mitigation (file:line) | res |
|---|---|:---:|---|:---:|
| T | **Gonflage de kudos** (un worker solo declare `tokens_generated` absurde, ex. 1e9 tokens en 5 ms, pour farmer la reputation) | M | sanity-bound `tokens <= TOKENS_PER_MS_CEILING * max(1, generation_time_ms)` AVANT `log_utility` (`kudos_ledger::credit`, applique aux 2 sites prod `validator_loop.rs` + `http.rs`) ; ferme la fuite de valeur absolue que `log_utility` (<10x marginal) laisse ouverte ; ancrage BOINC `wu.rsc_fpops_bound` | **L** |
| T/D | **Forge coherente des deux champs** (l'adversaire qui controle le payload declare `tokens` ET `generation_time_ms` mutuellement plausibles) | M | NON couvert par le sanity-bound : les deux champs vivent dans le MEME payload signe — c'est un **plausibility-check**, PAS une attestation anti-Sybil. Residuel = Sybil multi-keypair pre-existant §15.2 (PoW/AgeWitness + pilote ferme). Le `median` du groupe d'accord est DOC-P2 (infaisable sans casser « validator INCHANGE » ; inerte a `redundancy=1` ; non OSS-fidele) | **M** |
| I | **Sur-promesse GPU-heures** (presenter les heures locales comme une metrique reseau verifiable) | L | GPU-heures lues du `usage.json` worker LOCAL (`consent.rs`, `hours_used_today`), JAMAIS repliquees ni agregees cross-nœud ; libelle UI honnete « donnees par cette machine aujourd'hui (non attestees) » (`Network.tsx` ContributorCard) | **L** |
| I/D | **Fuite via la route** (un appelant lit l'activite kudos d'un node arbitraire) | L | route sous `authed_routes` (loopback bearer + Host + Origin) ; le ledger est deja local et `worker_node_id` = pubkey Ed25519 publique en clair ; self-view per-node, PAS de ranking reseau-wide (rejet EigenTrust tenu) | **L** |

**Residual S76-E** : le gonflage coherent des deux champs self-declares
(T/D, **M**) reste le cout assume — il se confond avec le Sybil multi-keypair
pre-existant (§15.2) et se mitige hors-quorum (PoW/AgeWitness + pilote ferme).
Le sanity-bound est une borne ASYMETRIQUE : il attrape le bug et l'exageration
grossiere du worker naif, pas l'adversaire qui forge un couple plausible. La
vue ne cree aucune frontiere de confiance nouvelle ; elle rend visible une
contribution deja creditee.

---

### §15.3 Convergence delivery WAN — keepalive de voisinage gossip (Sprint 77 Phase A)

Le fix de convergence (`nexus-core-rs/doc_sync.rs`, keepalive cable dans l'engine
worker) re-emet `Doc::start_sync(peers)` quand le voisinage gossip du doc de taches
tombe, pour que les ecritures `task:` incrementales du coordinateur continuent
d'arriver apres une rupture de transport (NAT rebind, relay change, adresses ticket
perimees, swap binaire). **Aucune frontiere d'admission nouvelle** : le worker re-dial
le MEME coordinateur dont il detient deja le `DocTicket` write-capable (minte par
l'invite loopback authentifie M19, `invite_api.rs`) ; il ne joint aucun pair
supplementaire. La re-resolution d'adresse passe par la decouverte pkarr native de
`presets::N0` — exactement le chemin de confiance de tout dial SBFB, deja couvert par
le canary Eclipse-by-DHT (`dht_quorum` / `pkarr_resolver`) : un relay pkarr malveillant
peut au pire refuser/perimer une reponse, pas forger une adresse (paquet signe par la
cle du node). Le `task:` reste une cle de DOCUMENT hors bytes canonical (0 bump wire) ;
la subscription ajoutee est observabilite-seule (drain best-effort, pas de
backpressure, le claim reste poll-based). **Caveat amplification** : le keepalive est
borne par `min_rejoin_interval` (cooldown) pour eviter un storm de re-join sur des
`NeighborDown` en rafale ; il ne s'execute que pour les docs importes via ticket (pas
pour les docs injectes en test). Surface inchangee, residual nul au-dela du modele de
confiance pkarr deja accepte.

---

## 16. Surface sharding inference (Sprint 77)

Le sharding pipeline (modele eclate sur 2+ machines, une `ShardAssignment`
contigue `[layer_start,layer_end)` par worker, hand-off des activations de
frontiere sur le data plane `sbfb/shard/1`) met un worker **a l'interieur** du
pipeline d'inference : il voit en clair les activations intermediaires des
couches qu'il execute. Source du catalogue : `SPLIT_INFERENCE_DESIGN.md §3.1`.

> **Provenance doc.** `SPLIT_INFERENCE_DESIGN.md` est anterieur (S30) a la
> decision S77 de **forker** llama.cpp (Phase F, arbitrage PO option (a),
> livre par F1). Sa recommandation §4.2 « ne pas forker les runtimes, preferer
> un wrapper » est donc **superseded** : seul son §3.1 (catalogue de menaces,
> ci-dessous) reste vivant. Ne pas lire §4.2 comme une regle courante.

### Catalogue SI (severites de `SPLIT_INFERENCE_DESIGN.md §3.1`)

| Vecteur | Description | Severite | Statut S77 |
|---|---|---|---|
| **SI-1 Activation reconstruction** | un worker reconstruit l'input via un modele inverse entraine sur le meme modele (demontre sur CNN, plus dur sur transformers mais pas impossible) | **High** | **residuel ASSUME** — limite physique, pas de TEE GPU consumer 2026 (scope cut #4) |
| **SI-2 Layer gradient leakage** | leak via backward pass (fine-tuning distribue) | N/A | **non applicable** — SBFB est inference-only, aucun gradient transmis |
| **SI-3 Activation fingerprinting** | les patterns statistiques des activations identifient le TYPE de prompt (langue/domaine), pas l'input exact | Medium | residuel assume ; corrélation bornee par le groupe prive (pas d'observateur tiers) |
| **SI-4 Collusion inter-workers** | la collusion de TOUS les workers du pipeline reconstruit le calcul complet ; la confidentialite ne tient que si >=1 worker est honnete (modele honest-but-curious) | **High** | **residuel ASSUME** — l'allowlist borne QUI participe, pas l'honnetete ; mitige par le pilote ferme (D5) |
| **SI-5 Latence side-channel** | le temps de compute d'un layer revele la complexite du prompt (longueur, heads actifs) | Low | residuel ; padding constant-rate = raffinement post-benchmark (Phase K) |

### Caveat d'usage cardinal

L'admission `ComputeGroup` (allowlist Ed25519 signee, `compute_group.rs`) est un
controle d'**ADMISSION** (qui peut participer), **PAS** de la confidentialite des
activations : celles-ci circulent **en clair** (aucun TEE GPU grand public en
2026, scope cut #4) et l'allowlist ne garantit pas une majorite honnete (SI-4
residuel — deja documente cote code `nexus-core-rs/src/shard.rs`). En
consequence : **aucun secret applicatif ne doit transiter par les prompts d'une
session shardee** — un membre admis mais curieux voit les activations de son
segment. Le sharding sert a executer un GROS modele public eclate, pas a traiter
des entrees confidentielles.

### Mitigations cablees (Sprint 77 Phase B + F2)

- **Admission server-side** (`ShardProtocol::accept`, Phase B) : `is_member` sur
  `conn.remote_id()` (Ed25519 QUIC non-spoofable) AVANT tout `accept_bi` — un
  non-membre est ferme au handshake (`SHARD_REJECT_NOT_MEMBER`).
- **Verif signature manifest cote claim** (Phase F2, `engine/shard_claim.rs`
  `authorize_claim`) : la signature du `ShardedSessionManifest`
  (`DOMAIN_SHARD_PLAN_V1`) est verifiee AVANT toute I/O — un membre admis ne peut
  pas se voir imposer un plan non signe par l'initiateur de session.
- **Cap VRAM fail-closed au claim** (Phase F2, `assess_capacity`) : estimation
  header-only over-estimee (headroom backend) comparee au `vram_free_bytes`
  MESURE (snapshot ponctuel, pas de pompe live — scope cut #7) ; pre-valide aussi
  la fenetre `[layer_start,layer_end) ⊆ [0,n_layer)` AVANT le load natif (anti
  process-abort sur fenetre hors-bornes). DoS : un manifeste forge/non-membre
  n'atteint jamais le read GGUF ni le snapshot GPU (crypto-avant-I/O).
- **Cap frame data-plane** (`MAX_SHARD_FRAME_BYTES=256 MiB`,
  `MAX_SHARD_N_CTX=8192`) : un frame d'activation au-dela du cap est rejete AVANT
  allocation ; borner `n_ctx` au placement borne simultanement le frame et le
  KV-cache.

### N0 TOPLOC fingerprint (Sprint 77 Phase G)

Phase G cable le **premier etage de la verification graduee, N0** : la primitive
`nexus-core-rs/src/toploc.rs` calcule le **commitment BLAKE3 32B** de l'encodage
canonique tout-entier du top-k (`TOPLOC_TOP_K=128`) du dernier hidden state, et le
worker dispose du helper (`toploc_commitment`) qui le produit apres son bloc de
couches. Les slots porteurs `[u8;32]` existent deja
(`RunProof.activation_fingerprint` chemin shard, `ResultPayload.logprobs_hash`
chemin modele entier) ; leur **ecriture dans un proof signe et leur emission
on-wire sont cablees avec le data-plane de session (Phase H/I/J)** — G livre la
primitive + le calcul cote worker, pas encore l'emission signee. Cote
consommateur, la Layer-3 de `verification.rs` traite deja `logprobs_hash` comme un
commitment TOPLOC (egalite), remplacant le Layer-3 logprob inerte. 0 bump wire
(slots deja v1), 0 nouveau `DOMAIN_*`.

**Ce que N0 detecte** : un worker qui execute un GGUF / une quantification
DIFFERENTE produit un top-k (indices + exposants bf16) divergent → commitment
different → swap detecte par inegalite (~100%, propriete LSH TOPLOC arXiv
2501.16007).

**Ce que N0 ne detecte PAS** (honnetete) :
- la **confidentialite** des activations — SI-1 (reconstruction, High) et SI-4
  (collusion, High) restent INCHANGES, le fingerprint ne chiffre rien ;
- une **activation/fingerprint forge mais coherent** par un worker qui controle
  son propre payload signe ;
- la **correction du calcul** en general — N0 est un detecteur de swap, PAS une
  preuve de calcul correct.

**Caveat auto-attestation (cardinal)** : le commitment qu'un worker publie pour
SON propre run est un **self-claim**, jamais une preuve, tant qu'un verifieur
independant ne le **recompute** pas. La comparaison TOLERANTE (exposant/mantisse,
`ToplocFingerprint::compare`) exige le sketch complet des deux cotes ; un hash
BLAKE3 detruit la localite (1 bit → avalanche) donc le commitment seul binde mais
ne tolere rien. Le recompute cross-worker in-vivo est N1 spot-check (Phase H) +
N2 redondance tolerante (Phase I) ; le transport du sketch complet hors du slot
32B y est aussi cable. Le live result path reste le quorum exact-match
`result_text` (`validate_quorum_pre_guardrail`), INCHANGE.

**Backend** : N0 exige les hidden states → le mode sharding impose
`llm_llama_cpp` (fork F1/F2). Sur Ollama/HTTP le slot reste `[0u8;32]` (N0
infaisable, pas de fork HTTP).

**Surface SI-3 (heritage)** : le fingerprint est lui-meme derive du dernier
hidden state → il herite de SI-3 (Activation fingerprinting, Medium) : il correle
au TYPE de prompt (langue/domaine), pas a l'input exact. Il circule sur le
control-plane (`RunProof`, iroh-docs) et n'est lisible que par le **groupe
prive** — pas d'observateur tiers. **Retention/GC** (addendum sharding §10 q.210,
OUVERT) : le fingerprint persiste tant que le `RunProof` persiste ; le GC
post-fenetre-de-contestation n'est PAS cable (renvoi N3 / Phase I).

### N1 spot-check VRF (Sprint 77 Phase H)

Phase H cable la **primitive du 2e etage, N1** : un verifieur est **tire au sort
de facon verifiable** (`nexus-core-rs/src/verifiable_draw.rs`) pour re-executer un
prefill (~1%, VeriLLM arXiv:2509.24257) et recomparer le fingerprint N0 du prover
via la comparaison **tolerante** (`ToplocFingerprint::compare`, jamais l'egalite a
`temperature > 0`). La selection remplace l'ancien `simple_hash(task_id)`
publiquement predictible (Sprint 40) — un prover ne sait plus ex-ante s'il sera
audite. Le verdict combine l'activation-fingerprint ET les **tokens sous seed
partage** (Token-DiFR, arXiv:2511.20621) : comparer seulement l'activation
laisserait un worker forger des tokens puis recalculer un fingerprint coherent.

**Construction et honnetete cardinale** : le tirage est une **signature Ed25519
deterministe** (`DOMAIN_VRF_DRAW_V1`) hashee, PAS un ECVRF (RFC 9381). Ed25519
est malleable → l'**unicite** du tirage n'est pas prouvee, et Ed25519 n'etant pas
une PRF, l'**imprevisibilite** ne l'est pas non plus. C'est une **mitigation sous
l'hypothese one-honest-verifier pour un echantillon 1-5%**, pas une garantie. Le
choix 0-dep (reutilisation de `crypto.rs`, precedent Phase D
`blake3(session_id||pubkey)`) est assume contre l'ajout d'une crate ECVRF lourde
sur une 2e courbe ; un ECVRF formel et la garantie N4 zkML restent hors-scope S77.

**Ce que N1 livre (Phase H) et ne livre PAS** : Phase H livre les primitives
(tirage, recompute tolerant, Token-DiFR, gate d'incentive, mapping criticite) +
leurs tests hermetiques. Le **re-exec prefill REEL sur GPU** et le **transport du
sketch complet** (hors du slot commitment 32B, qui reste binding-only) sont gates
(Phase I/K) — exactement le pattern Phase G (primitive sans cablage in-vivo).

**Nouvelles surfaces N1** (toutes Sev **M**) :
- **Predictibilite / grinding du tirage** : qui detient la cle peut faire varier
  l'entree pour biaiser le verifieur tire. **Mitigation cablee** : le `seed` du
  tirage DOIT etre une valeur que le worker verifie ne peut PAS choisir
  (`session_id || epoch || result_commitment`, deja signes) ; documente dans la
  primitive et teste (`vrf_verify` rejette seed/cle alteres).
- **Farming de kudos / Sybil-verifieur** : un verifieur pourrait reclamer du
  credit sans travail, ou un collusionnaire s'auto-tirer. **Mitigation** :
  `spotcheck_creditable` exige (1) un tirage VRF re-verifie + (2) un `RunProof`
  N1 SIGNE par le verifieur (preuve du travail), jamais une auto-declaration ;
  Sybil amont borne par PoW/AgeWitness + pilote ferme (D5).
- **Criticite auto-declaree / non-signee pour echapper a N2** : un initiateur
  pourrait taguer sa tache « faible-criticite ». **Provenance honnete des champs** :
  `criticality_maps_to_verification_level` derive le niveau de `Task.verifiable`
  (SIGNE — partie de l'identite canonique, un MITM ne peut le retourner sans casser
  la signature) ET de `Task.redundancy_factor` qui n'est PAS signe (dispatch policy
  EXCLUE des bytes canoniques, Sprint 23 `34c77ce`). Le niveau retourne est donc
  **advisory** vis-a-vis de la redondance : un MITM applicatif peut baisser
  `redundancy_factor` pour suggerer N1 a la place de N2. **Mitigation** : le niveau
  MINIMAL liant est impose par la policy du groupe/consommateur, jamais fait
  confiance au hint non-signe ni auto-declare par l'initiateur ; et le tirage N1
  s'applique INDEPENDAMMENT du tag (un downgrade ne supprime pas le risque d'etre
  tire). 0 champ wire nouveau.

**Confidentialite INCHANGEE** : SI-1 (reconstruction, High) et SI-4 (collusion,
High) restent identiques — N1 ne chiffre rien, il re-execute.

### Incentive a verifier (residuel economique, non-monetaire) — CABLE Phase H

L'incentive de S77 a executer/verifier honnetement est **reputationnel**
(kudos non-monetaire, jamais monetaire — PO-12 interdit stake/token, cf.
risk **R8** du plan) : c'est une **mitigation**, pas une garantie economique.
**Phase H le cable** : un verifieur tire et prouve credite du kudos reputationnel
via le mecanisme **existant** `kudos_ledger::credit` (il n'existe AUCUN module
`curator` ; le ledger kudos EST la reputation), gate par `spotcheck_creditable`.
La sanction d'un verifieur faux/paresseux est **strictement non-economique**
(non-credit / trust-delta negatif sur le chemin prover), **jamais** slash/bond/
burn — VeriLLM tire sa defense game-theoretique du slashing, INTERDIT ici. Il n'y
a donc **pas de defense anti-verifieur-paresseux** en S77 (carry honnete) : un
verifieur paresseux rationnel peut ne pas verifier ; la garantie cryptographique
(N4 zkML) est hors-scope S77 (scope cut #1). Le pilote ferme (D5) + l'anti-Sybil
amont (PoW/AgeWitness) bornent l'exposition. Severite residuelle **M**.

> **Completion Phase K** : l'integration STRIDE formelle (§5.x) + LINDDUN (§6) +
> les lignes §2 Assets / §4 DFD pour le composant sharding, ainsi que la
> mitigation SI-5 (padding constant-rate) derivee du benchmark reel, sont
> finalisees au wrap-up Phase K. Le present §16 fige le catalogue de surface et
> les mitigations co-localisees avec le code claim/wiring que F2 introduit.

---

## 17. Revue et evolution

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
- **v4 (Sprint 67 Phase B, 2026-05-20)** : ajout §11 Search surface
  (T-SEARCH-INJECTION, T-CURATOR-VOUCH, T-SEARCH-DOS), closure
  P2-THREAT-MODEL-FEED-SURFACE 3/3, renommage §11→§12.
- **v5 (Sprint 68 Phase D, 2026-05-21)** : ajout §12 ProofCard
  surface (T-PROOFCARD-FORMULA-GAME), renommage §12→§13.
- **v6 (Sprint 69 Phase A, 2026-05-22)** : ajout §13 Preview
  ephemere surface (T-PREVIEW-EXHAUSTION), renommage §13→§14.
- **v7 (Sprint 72 Phase A, 2026-05-31)** : ajout §14 Operator surface
  (T-OPERATOR-CSRF, T-OPERATOR-SPAWN + anticipation NetworkProvider),
  renommage §14→§15. Closure P2-H-1 (audit S71 Track H) — la defense
  (S71 G7 token+Host+CORS / G2 gate SENSITIVE_ACTIONS) etait deja
  livree ; ce catalogue rattrape le retard documentaire avant
  l'extension de surface SSE par le ProviderRouter S72.
- **v8 (Sprint 75 Phase G, 2026-06-11)** : ajout §15.1 extension
  decouverte PULL node-centrique (oracle blob-serve 4e tier, /nodes,
  SEED-1/SEED-2, fresh-flood, boot seed driver, requester route,
  surfaces front sans duress gate). Rows deferes des phases D/E/F
  consolides ; residuals fresh-flood + duress freres routes audit S76.
- **v8.1 (mini-cycle UX-ARRIVAL post-S75, 2026-06-11)** : +2 rows §15.1
  — registre observed RAM (ingest non-sollicite de metadonnees
  d'annuaire, no-fetch/no-dial, cap+TTL+rate-limit+self-guard dans la
  primitive ; residuels honnetes : PoW non lie au payload → identites
  forgees non tarifees une a une [classe fresh-flood, publisher-binding
  route S76], identite non-Ed25519-verifiee, non-autoritaire) + spoof
  du placement « Tes sources » via ProjectAnnouncement.node_id non
  signe (review P1 SEC-UXARR-1 → mitigation : from_subscribed
  CATALOG-BACKED contre l'annuaire Ed25519-verifie du noeud reclame).
- **v9 (Sprint 76 Phase G, 2026-06-17)** : consolidation de la surface
  **compute partage cross-machine** (GPU partage, Arc 3.5 6/6 clos). Pas de
  nouvelle row STRIDE — les surfaces sont deja documentees par les phases :
  **§15.2** (Phase D) quorum cross-machine redundancy>1 = la cohorte homogene
  (`RuntimeTuple`) n'est qu'un ROUTAGE ADVISORY, jamais une frontiere de
  confiance ; la vraie defense est le quorum exact-match `(worker_pubkey,
  task_id)` sur `result_text` (fix bridge `d75ae77`, miroir du validator) ;
  Sybil multi-keypair = residuel M pre-existant (PoW/AgeWitness + pilote
  ferme). **§15.3** (Phase E) dashboard contributeur : `tokens_generated`
  self-declare hors-quorum → sanity-bound plausibilite (clamp vs
  `generation_time_ms` reel) = catch-the-bug, PAS anti-Sybil ; option
  median-de-groupe DEFERRED P2. **Duress-freres (B1)** : deja FERME §15.1
  (row « Surfaces front F sans duress gate », residual Nil) — `seed_voluntary`
  + `set_keep_online` no-op en duress (early-return, bytes leurre==succes).
  **Acceptance compute LIVE** (B-3 palier 1 + quorum palier 2) : differe-
  materiel-operateur, harness `b3_live_pc_vps.sh` runnable (`REDUNDANCY`) ;
  le chemin compute (dispatch/pompe/result-sync/validator/sign-verify) est
  couvert in-process. **Etage-2 TOPLOC** (`logprobs_hash` commitment hidden
  state) = S77, requis pour le quorum cross-GPU heterogene (impossible en
  stock : meme GGUF diverge cross-GPU).
- **v10 (Sprint 77 Phase F2, 2026-06-22)** : ajout **§16 Surface sharding
  inference** (catalogue SI-1..SI-5 de `SPLIT_INFERENCE_DESIGN.md §3.1` +
  caveat « activations en clair / pas de TEE GPU consumer / aucun secret app
  dans les prompts » + mitigations cablees admission/manifest-verify/cap-VRAM-
  fail-closed/cap-frame + incentive reputationnel residuel M), renommage
  §16 « Revue et evolution »→§17. Section figee co-localisee avec le claim +
  cablage `sbfb/shard/1` que F2 introduit ; STRIDE/LINDDUN formel + SI-5 padding
  derives du benchmark = Phase K.
- **v11 (Sprint 77 Phase G, 2026-06-22)** : cablage **N0 TOPLOC fingerprint** —
  ajout sous-section §16 « N0 TOPLOC fingerprint (Phase G) » (detection swap
  modele/precision par INEGALITE de commitment BLAKE3 du sketch top-k entier
  `toploc.rs` ; ne couvre PAS la confidentialite SI-1/SI-4 ni la correction de
  calcul ; caveat auto-attestation cardinal = self-claim tant que N1/N2 ne
  recomputent pas ; comparaison tolerante = `ToplocFingerprint::compare` recompute
  N1 Phase H / N2 Phase I ; backend `llm_llama_cpp` requis, slot `[0u8;32]` sur
  Ollama ; SI-3 herite + retention/GC ouvert) + MAJ §15.2 row I (reserve etage 2
  → CABLE Phase G, residuel **M** tant que le recompute N1/N2 Phase H/I n'est pas
  livre). 0 nouvelle row STRIDE (surfaces SI-1..5 deja v10), 0 bump wire (slots
  `logprobs_hash` / `activation_fingerprint` deja v1).
- **v12 (Sprint 77 Phase H, 2026-06-22)** : cablage **primitive N1 spot-check
  VRF + incentive reputationnel** — ajout sous-section §16 « N1 spot-check VRF
  (Phase H) » (tirage verifiable Ed25519 `verifiable_draw.rs` NON-ECVRF assume +
  recompute tolerant `ToplocFingerprint::compare` + Token-DiFR + 3 nouvelles
  surfaces Sev M : grinding-tirage / farming-kudos-Sybil-verifieur /
  criticite-auto-declaree, chacune avec mitigation cablee) ; sous-section
  « Incentive a verifier » passee de **concu** a **CABLE Phase H**
  (`spotcheck_creditable` + `kudos_ledger::credit`, sanction strictement
  non-economique, carry honnete « pas de defense anti-verifieur-paresseux ») ;
  MAJ §15.2 row I (primitive N1 cablee Phase H, re-exec prefill in-vivo +
  transport sketch = Phase I/K → L pour l'echantillon tire). Confidentialite
  SI-1/SI-4 High INCHANGEE (N1 ne chiffre rien). 0 nouvelle row STRIDE,
  0 bump wire (1 `DOMAIN_VRF_DRAW_V1` additif, slots deja v1), 0 dep nouvelle.
