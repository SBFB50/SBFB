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
| **Coordinator** | `crates/nexus-coordinator-rs` | Lib Rust embarquee dans le daemon (DB, dispatcher, validator, kudos — porte de Python S50-S51 ; le handler deploy vit dans le daemon `deploy.rs`) |
| **Blob-serve** | `crates/nexus-shell-daemon-core/src/blob_serve.rs` (handler dans le daemon `blob_serve_http.rs`, route cablee dans `http.rs`) | Route publique du listener daemon (`public_routes`) ; isolation iframe par origin OPAQUE (sandbox sans allow-same-origin), pas par port separe |
| **Worker** | `crates/nexus-worker*` | Binaire GPU, claim-loop, Ollama runtime |
| **NexusApp iframe** | `packages/nexus-app-*`, apps externes | Contenu untrusted, bridge postMessage |
| **iroh stack** | `crates/nexus-core-rs` | QUIC, docs, gossip, blobs — iroh =1.0.1 / docs 0.101 / gossip 0.101 / blobs 0.103 (S81) |
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
| A8 | Activations sharding + RunProof | **Haute** (activations en clair, SI-1) | **Critique** (verdict N0-N3) | Moyenne | frames `sbfb/shard/1` en transit (groupe prive) + `RunProof` signe (slot `activation_fingerprint`) |

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
    |  (route /blob-serve du |           sans allow-same-origin
    |   daemon, origin       |           => origin OPAQUE,
    |   opaque, untrusted)   |           CSP connect-src 'none'
    +------------------------+

    authFetch(X-SBFB-Token + Host + Origin)
           |
    =======v=========================================
           |
      LOOPBACK HOST (trust-B)
    +------+-----------------+         +-----------+
    |  Shell daemon          +-------->|  Worker   |
    |  :ephemere (api_port 0,|         |  :none    |
    |   publie running.json) |         | (GPU/CPU) |
    |  (bearer+Host+Origin   |         +-----+-----+
    |   enforced ; embarque  |               |
    |   nexus-coordinator-rs |               |
    |   en lib : DB/dispatch |               | consent.json
    |   /kudos, S50-S51)     |               | usage.json
    +------+-----------------+               v
           |                          +------+-----+
           |                          |  Ollama    |
           |                          |  runtime   |
           |                          +------------+
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

**Flux sharding (Sprint 77, `sbfb/shard/1`)** : un noeud-tete (trust-A,
membre `ComputeGroup` Ed25519) ouvre une connexion QUIC dediee vers chaque
worker-shard du groupe prive ; chaque worker charge UNIQUEMENT son bloc de
couches (P-D) et forwarde CHAQUE frame d'etat-frontiere recue sur le bi-stream
long-lived (activations fp32 en clair, SI-1) apres admission `is_member`. Le
verdict d'integrite est porte HORS-bande par les `RunProof` signes (N0-N3), pas
par le transport lui-meme.
L'orchestrateur de session qui pilote une generation token-par-token cross-shard,
mesure TTFT/tok-s et emet un `RunProof` in-vivo est CABLE depuis S81 Phases I/J
(`shard_session.rs` : mount barrier + drive HUB + decode loop + premiere emission
production du `RunProof` signe, benchmark live 5080+M2 PASS) ; depuis S81 Phase K
chaque etablissement de stage-link d'une session reelle exige en plus
l'ATTESTATION du stage charge (binding loaded-stage <-> manifeste signe,
fail-closed ; le chemin echo digest-zeros ne sert aucune session reelle et
n'emet ni n'exige d'attestation — cf. §16). Cf. §16.

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
| E | RCE via deserialization iroh | C | Version pinnee =1.0.1 (upgrade S81 ; le wire-freeze 1.0 reduit le churn de la surface de deserialisation — neutre-a-positif, jamais un durcissement de confiance) + `cargo-deny check advisories` (RustSec) en CI | **M** |

**Note S81 (upgrade ≠ audit)** : l'upgrade iroh 0.98→1.0.1 ne franchit
PAS Gate 1 / Gate 3 — **R-iroh-audit reste une zone rouge P0** (iroh
1.0 n'a aucun audit tiers public connu), le pilote reste ferme. Le
residuel E ci-dessus reste **M** pour cette raison.

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
| T | Consent.json edite par un autre process | H | Atomic tmp+rename cote daemon (`save_consent`, `nexus-shell-daemon/src/consent.rs`, chemin UI `POST /api/v1/consent/set`) ; le worker ne fait que LIRE (`ConsentWatcher`, `notify` 50 ms debounce re-read) | **M** |
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
| T | npm dep postinstall script | H | `package-lock.json` commited ; CI `cargo-deny check advisories` (subsume `cargo-audit`) + `audit-ci` (npm, seuil critical-only : advisories CONNUES seulement — un postinstall malveillant frais sans advisory n'est PAS borne) — jobs PR fail-non-zero par config, S18, `.github/workflows/supply-chain.yml` ; la jambe `pip-audit` S18 est INOPERANTE depuis la purge Python S50-S51 (3 packages cibles supprimes, reparation/suppression du job routee au ledger) | **H** |
| T | PyO3 wheel remplacee | H | `maturin develop --release` depuis sources locales ; pas de wheel telechargee | **M** |

---

### 5.9 Sharding inference (`sbfb/shard/1`, Sprint 77)

Composant : un noeud-tete eclate un modele ~20 Go sur N workers-shard d'un
`ComputeGroup` prive (pipeline-parallel, P-D). Le catalogue de surface detaille
vit en §16 (SI-1..SI-11 + N0-N3 + incentive) ; resume STRIDE par composant :

| Menace | Exemple | Severite brute | Mitigation | res |
|---|---|:---:|---|:---:|
| **S**poofing | Un worker non-membre revendique un shard | H | admission `is_member` Ed25519 sur `ComputeGroup` signe (`shard.rs::accept`) ; claim crypto-avant-IO (`shard_claim.rs`) | **L** |
| **T**ampering | Un worker execute un GGUF/quant different ou ment sur son forward | H | primitives N0 TOPLOC + N1 VRF + N2 quorum tolerant + N3 commit-reveal CABLEES+testees hermetiquement (S77 G/H/I) ; emission signee in-vivo du `RunProof` DRIVER **LIVREE S81 I/J** (fingerprint N0 bind au dernier step) ; **attestation loaded-stage fail-closed S81 K** (ferme la MISCONFIGURATION : mauvais GGUF/fenetre/role rejetes avant tout frame — un worker qui MENT dans son self-claim reste le residuel SI-4) ; RunProofs PER-WORKER distants = re-route **S82** | **M** |
| **R**epudiation | Un worker nie son etat-frontiere | M | `RunProof` signe DRIVER emis in-vivo depuis S81 I/J (slot `activation_fingerprint` bind au commitment N0 du dernier step) ; RunProofs per-worker via canal control-plane = re-route **S82** | **M** |
| **I**nfo disclosure | Les activations transitent en clair vers les workers-shard | H | **SI-1 residuel ASSUME** : pas de TEE GPU consumer 2026 ; groupe prive explicite + caveat « aucun secret app dans les prompts » | **H** |
| **D**oS | Frame surdimensionnee / epuisement VRAM | M | cap-frame 256 MiB + cap-VRAM fail-closed (`assess_capacity`, geometrie degeneree rejetee) | **L** |
| **E**levation | Un worker-shard depasse son bloc de couches | M | forward de frames sur le bi-stream, aucune autorite sur l'orchestration ; admission-gated, pas d'orchestrateur expose au worker | **L** |

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
| Shard frontier forward (`sbfb/shard/1`) | M (node_id lie aux shards du groupe) | L (pseudo) | M (RunProof signe = trace) | M (membres du groupe voient les frames) | **H (activations en clair vers les workers-shard, SI-1)** | L (groupe prive explicite + consentement worker) | L (groupe prive, pas de PII dans les activations sauf si le prompt en porte) |

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

> **Note de lecture (S82 Phase I).** Cette table est un journal de
> livraison date (colonne Commit). Les chemins
> `packages/nexus-coordinator/**/*.py` cites dans « Fichier cle » sont
> HISTORIQUES : le coordinator Python a ete porte en Rust
> (`crates/nexus-coordinator-rs`, S50-S51) et la source .py est purgee
> de l'arbre. Les equivalents vivants sont dans
> `nexus-coordinator-rs` + `nexus-shell-daemon` (surface HTTP axum).

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
| `is_open_source` flag PA v5 | Core-rs + Coord + Shell | `10bbc63` | `crates/nexus-shell-daemon-core/src/publish.rs` (PA v5 = 5e iteration canonique ; la constante wire `PROJECT_ANNOUNCEMENT_VERSION` reste 1 pre-launch) + `packages/nexus-coordinator/src/nexus_coordinator/api/deploy.py:1-475` (derive true/false) | **LIVRE S16D** |
| Zod schema `BrowseEntry.is_open_source: z.boolean().optional()` | Shell | `10bbc63` | `web/src/api/daemon.ts` (distingue legacy undefined de `false` explicite) | **LIVRE S16D** |
| Iframe sandbox strict | Shell + Blob-serve | S12-S13 | `crates/nexus-shell-daemon/src/blob_serve.rs` (CSP connect-src 'none') | **LIVRE** |
| postMessage bridge whitelist | Shell + iframe | S13 | `web/public/sbfb-bridge.js` + `web/src/bridge/protocol.ts` | **LIVRE** |
| Verified deploy Keyoxide + SLSA L1 | Coordinator | S14 | `packages/nexus-coordinator/src/nexus_coordinator/provenance.py` + `deploy.py` | **LIVRE** |
| CPU watchdog heartbeat | Shell bridge | S15 | `web/src/bridge/useBridge.ts` (watchdog state machine) | **LIVRE** |
| Encryption at rest keypair | Daemon | — | Keychain / DPAPI / libsecret | **DIFFERE S17+** |
| VM isolation auto-install | Launcher | — | Cf. `RUNTIME_ISOLATION.md` | **DIFFERE S17+** |
| CI `cargo-deny check advisories` (subsume `cargo-audit`) / `audit-ci` (npm) — la jambe `pip-audit` S18 est INOPERANTE depuis la purge Python S50-S51 (3 packages cibles supprimes) | Infra | S18 | `.github/workflows/supply-chain.yml` | **LIVRE S18** |
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

### R2 — Supply chain : residuel malgre CI advisories (severite M a H selon dep)

- **Asset** : tout le repo compile
- **Adversaire** : AD3 / AD4 + proprietaire dep typo-squatte
- **Impact** : RCE au build-time ou exec-time.
- **Mitigation residuelle** : `Cargo.lock`, `package-lock.json`,
  revue manuelle ajout deps ; CI `cargo-deny check advisories`
  (meme base RustSec que `cargo-audit`, qu'il subsume — S18 D3) +
  `audit-ci` (npm, seuil critical-only) — jobs declenches sur PR,
  fail non-zero par config (`.github/workflows/supply-chain.yml`,
  S18). La jambe `pip-audit` S18 est INOPERANTE (packages Python
  purges S50-S51).
- **Residuel** : un scanner d'advisories ne borne que le CONNU —
  fenetre zero-day avant publication d'advisory, postinstall
  malveillant frais non couvert (d'ou le H de la row §5.8) + gap
  solo-maintainer sur la revue des nouvelles deps
  (`cargo-vet`/`osv-scanner` restent futurs, cf.
  `VALIDATED_BLUEPRINT.md`).

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

Le rate-limit engine (`governor` 0.10 GCRA) gate les claims
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
| Residual | M (S81 wf4 : le fold materializer ordonne par chaine prev_hash intra-auteur + tie-break (timestamp, author, hash) inter-auteurs — l'antidatage intra-auteur ne reordonne plus rien ; RESIDUEL : un auteur post-datant jusqu'a +30j gagne le tie-break inter-auteurs de facon convergente sur un project_id partage, tant que le binding author→project_id n'existe pas — vrai correctif carry §10) |

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

### 13.1 Gate CSP authoring au publish (Sprint 79 Phase E)

Le gate `sbfb-factory::gates::run_gate_csp_authoring` scanne **statiquement**
les assets d'une app au moment du **publish** et bloque la publication si elle
violerait la CSP du bac a sable (`nexus_core_rs::csp::BLOB_SERVE_CSP`, source
unique). C'est une defense de **nature differente** de la mitigation runtime
deja modelisee en §5.1 (App iframe) : la CSP runtime, reinjectee par blob-serve
sur chaque reponse (`blob_serve_http.rs` → `blob_serve::BLOB_SERVE_CSP`), bloque
l'exfiltration **a l'execution chez le client** ; le gate publish-time
**empeche de distribuer** une app non conforme et donne a l'auteur un
diagnostic immediat. Defense en profondeur — ni l'un ni l'autre n'est suffisant
seul.

Surface couverte qui dépassait `connect-src 'none'` (§5.1) : `form-action 'none'`
(soumission `<form action=remote>` = navigation, pas une connexion fetch) et
`base-uri 'none'` (`<base href=remote>` detourne les URL relatives) — deux
vecteurs d'exfiltration que `connect-src` n'arrete pas. Le gate ajoute leur
**detection statique** (le contrat CSP les declarait deja `'none'` mais aucun
lint ne les verifiait), plus `object-src`/`frame-src` et `<script type=module>`
(echoue sous COEP `require-corp`).

Limites assumees : un scanner regex est aveugle au code/URL assemble au runtime
(`fetch` via `atob`, `action`/`href`/`url()` construits dynamiquement). Ces
faux-negatifs sont rattrapes par la CSP runtime (browser-enforced) + le
self-check runtime qui rejoue l'app sous la CSP reelle (Sprint 79 Phase H). Le
gate ne pretend PAS prouver l'absence d'exfiltration — il prouve la conformite
des assets *statiques livres*.

| Dimension | Valeur |
|---|---|
| Severite | N/A (defense additive, pas une nouvelle menace) |
| Role | Filtre de distribution + feedback auteur au publish |
| Couverture | Detection statique des directives `'none'` du contrat CSP (anti-drift par test cross-crate) |
| Limite | Faux-negatifs runtime → couverts par CSP runtime §5.1 + self-check Phase H |
| Determinisme | Scan regex/statique pur, aucun ML (FACTORY_GATES.md principe 4) |

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
(`sbfb-factory/src/auth.rs`, fn `auth_required`) applique sur chaque
route data-bearing —
(1) `X-SBFB-Token` bearer per-boot compare en `constant_time_eq` (401
sinon) ; (2) header `Host:` doit etre loopback (403 sinon) ; (3) header
`Origin:` doit etre loopback ou absent (403 sinon) ; (4) `CorsLayer`
epingle a `is_loopback_origin` (bloc `CorsLayer::new()` de
`operator_server.rs`, plus de `allow_origin(Any)`).

**Amendement Sprint 80 Phase A (cookie de transport + garde cross-port).**
Pour rendre le streaming exploitable en prod same-origin sous
`ServeDir`, le bearer a desormais **deux transports**. Precision de
nommage (S81 G, nit audit S80-H-4) : le front S80 consomme le SSE via
`fetch`+`ReadableStream` (jamais `EventSource`, invariant S80 C) —
seule la correction du NOM d'API ; le raisonnement de fond est
inchange : le front navigateur s'authentifie par le **cookie** pour
TOUT, SSE inclus (`useTokenStream.ts:135` `credentials:
'same-origin'`, aucun header pose) comme pour le WebSocket PTY (qui,
lui, ne PEUT pas poser d'en-tete custom). Le header `x-sbfb-token`
reste le transport des clients NON-navigateur (CLI, scripts, proxy
Vite) et est essaye D'ABORD par le middleware. La protection CSRF du
chemin cookie ne vient pas d'un en-tete : elle vient de
SameSite=Strict + la garde `Sec-Fetch-Site: same-origin` (exigee sur
le chemin cookie uniquement, `auth.rs`). Les deux transports : le header `x-sbfb-token` (essaye
D'ABORD, inchange, intrinsequement CSRF-immune car le JS d'une page ne
peut pas le poser cross-origin) ; et un **cookie `sbfb_operator`**
HttpOnly + SameSite=Strict pose par le bootstrap `GET /?token` (D5). Le
cookie change le modele de menace et **invalide l'affirmation S71 « un
navigateur tiers ne connait pas le token »** : un cookie est une
autorite **ambiante** envoyee automatiquement par le navigateur. Deux
P1 specifiques au **cross-port loopback** sont donc fermes ici :

- **CSRF cross-port** : SameSite et `is_loopback_origin` ne sont PAS
  port-scopes — une page hostile sur `http://127.0.0.1:<autre-port>` est
  *same-site* et son `Origin` loopback passe, donc le cookie partirait.
  Garde : sur le **chemin cookie uniquement**, `auth_required` exige
  `Sec-Fetch-Site: same-origin` (en-tete *forbidden* que le JS ne peut
  forger, emis y compris sur GET/SSE/WS same-origin qui omettent
  `Origin`). Le chemin header n'exige rien de plus (CLI/Vite). Pas de
  `allow_credentials(true)` au CORS.
- **Fuite du bearer maitre** : les cookies ne sont pas isoles par port
  (RFC 6265 §8.5) ; pour qu'un cookie vole/cross-port ne livre jamais le
  bearer partage `~/.sbfb/auth_token`, la **valeur du cookie est un
  secret de session per-boot distinct** (`AuthState.session_secret`,
  64 hex CSPRNG), jamais le token. Compare en `constant_time_eq` cote
  serveur.

Defense-en-profondeur additionnelle : CSP self-origin de l'Operator
(`default-src 'self'; connect-src 'self'`, hors `BLOB_SERVE_CSP`
scellee) ; `Referrer-Policy: no-referrer` + 303 qui retire `?token` de
la barre d'adresse sur le bootstrap.

| Dimension | Valeur |
|---|---|
| Severite | H (write disque + spawn bypassPermissions) |
| Likelihood | L (token bearer 256-bit + Host/Origin loopback ; chemin cookie garde par `Sec-Fetch-Site: same-origin`) |
| Mitigation | `X-SBFB-Token` (constant_time_eq) + Host + Origin + CORS epingle (S71 G7) ; **+ S80 Phase A** : cookie HttpOnly/SameSite=Strict a secret de session distinct, garde `Sec-Fetch-Site: same-origin` sur le chemin cookie, CSP self-origin |
| Residual | (1) Processus local hostile lisant `~/.sbfb/auth_token` (frontiere OS-sandbox, accepte — cf. AD2 / §5.7) ; (2) `?token` survit dans l'history/Referer du navigateur apres le 303 (mitige `no-referrer` ; aucun `TraceLayer` ne logge la query — attaquant local = deja T0/AD2, accepte) |

### T-OPERATOR-SPAWN — Spawn agent autonome non gate

Le stream chat SSE spawn un agent `claude --permission-mode
bypassPermissions`. Un message portant une action sensible
(`shell` / `commit` / `push` / `PASS`) pourrait declencher une action
irreversible (commit, push, shell arbitraire) sans confirmation.

Mitigation (S71 G2, `a0337c6`) : `handle_chat_stream`
(`operator_server.rs`) applique le **meme** filtre `SENSITIVE_ACTIONS`
(`const SENSITIVE_ACTIONS` : `shell`/`commit`/`push`/`PASS`) que les
endpoints JSON, **AVANT** le spawn (gate `is_sensitive` -> `sse_gate`,
dispatch `target.run`). Un dernier message
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
| I/D | **Oracle blob-serve drive-by + amplification de dials** : un `GET /blob-serve/{hash}` sur un hash absent declenche le 4e tier directory-only → dials sortants vers ancre+seeders (observation du graphe, amplification) | M | resolution UNIQUEMENT sur annuaires ABONNES (`verrou 5`, attention-set explicite) ; cap `MAX_FETCH_PROVIDERS=16` enforce DANS la primitive (`blobs.rs fetch_hash_multi`) ; timeout appelant. **S76 B8 (THREAT-BLOBSERVE-BEARER) : `/blob-serve` est PUBLIQUE par construction (binding `public_routes` dans `http.rs build_router` — SANS bearer/Host/Origin, car un iframe sandboxe `allow-scripts` sans `allow-same-origin` ne peut pas porter le bearer pour charger ses assets) ; l'amplification est bornee par le subscribed-only + le cap + le timeout, JAMAIS par un bearer. La revendication anterieure « loopback bearer requis sur la route » etait fausse — corrigee.** | **L** |
| I | **Inventaire /nodes** : enumeration des catalogues connus du noeud | L | loopback bearer ; contenu = annuaires signes deja publics par construction ; route additive, `/browse` byte-identique | **Nil** |
| S/D | **Timestamp futur dans `SeedAnnounced`** (monopoliser la fraicheur du registre) | M | SEED-1 : clamp `seen_at = min(seen_at, now)` DANS `SeedRegistry::record` (pas une convention d'appelant) | **Nil** |
| D | **Gonflement du registre seeders** (buckets/slots illimites, variantes de casse d'une meme pubkey) | M | SEED-2 : double cap 1024 buckets / 64 seeders + eviction stalest-si-newcomer-plus-frais ; normalisation hex lowercase write+read (2^64 variantes de casse = 1 slot) | **L** |
| D | **Fresh-flood displacement** : annonces continues fraiches evincent les vrais seeders du registre cappe | M | residuel assume best-effort (doc `MAX_REGISTRY_BUCKETS`) : le compteur n'est jamais l'autorite, la sonde live + BLAKE3 le sont ; sampling anti-Sybil du tail route audit S76 | **M** |
| E/T | **Boot seed driver** : config `[seed]` rejouee sous identite duress ; annuaire divergent ecrasant le pin local | M | duress short-circuit EN TETE du driver (`seed_api.rs run_boot_seed_driver`, ex-http.rs S82 Phase O) ; resolution direct > row M18 > annuaires FIGEE par test ; clamp lowercase-64-hex au load ; defaut compile VIDE (verrou 3 tripwire) | **L** |
| E/D | **Boot feed-emit sous duress** (audit S75 DURESS-BOOT-LEAK, P1 ferme) : `reannounce_seeds_at_boot` (re-annonce les lignes `keep_online` REELLES) et la republication feed S66 (`replay_all` + orphan recovery — rejoue l'INTEGRALITE du feed reel vers iroh-docs) emettent sous la cle au boot ; NON gardes avant le fix, ils correlaient l'identite leurre au vrai data root a CHAQUE boot, zero interaction utilisateur | H | duress short-circuit EN TETE des DEUX chemins (`gossip_publish_in_duress(identity_mode) == Noop`, miroir du driver) : un noeud leurre n'emet AUCUN `SeedAnnounced` ni feed reel sur le reseau ; tests `reannounce_seeds_noop_in_duress` (feed iroh-docs vide sous duress) + primitif `duress_mode_noop_publishes` | **Nil** |
| E | **Requester route `/seed/request`** : self-designation, replay, mint sans detention | M | invite M19 TOUJOURS requise ; self-guard sur identites PARSEES (anti-base32) ; mint frais gate-detention 409 ; echo nonce verifie ; timeout 120s documente (504 ≠ echec, invite consommee AVANT fetch) | **L** |
| E | **Surfaces front F sans duress gate** : `seed_voluntary` + `set_keep_online` exposes par l'UI /nodes-/node/:id alors que leurs handlers PRE-EXISTANTS (S74) n'etaient pas duress-gates | M | **S76 B1 (DURESS-FRERES-LOCAL, FERME) : les DEUX handlers court-circuitent EN TETE en duress (`gossip_publish_in_duress(identity_mode) == Noop` → early-return + reponse leurre benigne, miroir du driver) → ZERO mutation du vrai data root (pas de row M18, pas de tag blob) ET ZERO emit `SeedAnnounced` sous la cle leurre (le seul early-return couvre le pin local ET l'emit). Tests `seed_voluntary_noop_in_duress` + `set_keep_online_noop_in_duress` (zero ecriture, zero tag).** Le lot duress freres LOCAL-ONLY est clos ; tous les chemins boot wire-emit l'etaient deja (audit S75 DURESS-BOOT-LEAK, row ci-dessus) | **Nil** |
| S/D | **Registre observed RAM (UX-ARRIVAL post-S75)** : ingest non-sollicite de metadonnees d'annuaire (pubkey entendue sur gossip sans abonnement) — flood Sybil de pubkeys forgees, spam de re-publications d'une identite, usurpation visuelle d'un node_id dans /nodes | M | NO-FETCH/NO-DIAL absolu pour un non-abonne (la metadata = enveloppe gossip seule, jamais le blob — l'amplification BitTorrent-DRDoS/libp2p-PR#577 n'est pas introduite) ; bornes DANS la primitive (`iroh_runtime.rs record_observed_directory`) : cap 256 + eviction stalest, TTL 48h, rate-limit 1/min par identite RECLAMEE (exigence PO) ; self-guard (notre node_id n'est jamais observe, ni par echo ni par forge). Residuels assumes (review UX-OBS-RATELIMIT-UNAUTH) : (1) le champ `node` de l'enveloppe est NON authentifie et le PoW est lie a (publisher, topic), PAS au payload — un seul PoW couvre N annonces de pubkeys forgees distinctes : le rate-limit borne la churn PAR identite et le cap borne la taille, mais rien ne tarife les identites forgees une a une (classe fresh-flood SeedRegistry). **S76 B1 — decision (b) : le registre observed est AVAILABILITY-ONLY, non publisher-authentifie, par construction (un indice de joignabilite, jamais une attestation). Lier la capture au PoW publisher exigerait l'auteur d'enveloppe VERIFIE a la couche `process_directory_announcement_bytes` (le call-site ne passe que content+node) — durcissement differe, non sur-promis ; la vraie defense anti-flood reste le self-guard + l'exclusion des abonnes + le registre borne/rate-limite (test `observed_capture_is_availability_only`).** ; l'etat du limiteur EST l'entree residente — une identite evincee par le cap est re-acceptee immediatement si elle re-annonce, mais s'evincer exige d'etre la stalest du registre entier (256 identites plus fraiches = deja le regime flood ci-dessus) : UNE identite ne peut pas s'auto-churner, et hors flood l'entree ne sort que par le TTL 48h >> 60s ; (2) l'identite observed n'est PAS Ed25519-verifiee (le blob n'est pas fetche) — non-autoritaire par construction : la seule action offerte est un subscribe explicite, et un node_id usurpe ne produit aucun catalogue (ligne « en attente » honnete) | **L** |
| S | **Spoof du placement « Tes sources » via `ProjectAnnouncement.node_id` (UX-ARRIVAL)** : l'annonce direct ne porte AUCUNE signature — un annonceur (un PoW) peut nommer la pubkey d'une ancre publique abonnee et viser le placement de confiance de la grille (et le hero « En vedette ») | H | `from_subscribed` est CATALOG-BACKED, jamais derive de la seule appartenance du node_id reclame a l'attention set (`browse_api.rs browse_views`, ex-http.rs S82 Phase S2) : un `direct` n'est classe « mes sources » que si sa paire (project_id, archive_hash) figure dans le catalogue Ed25519-VERIFIE de l'annuaire signe du noeud reclame — un spoofer ne peut pas inserer de row dans un catalogue signe ; les vraies apps d'un noeud abonne y sont par construction du pivot PULL (publish → revision>0 → re-annonce boot). Une entry sans archive_hash n'est jamais classee sur claim nu. Test decisif `browse_views_derives_from_subscribed` (fixture SpoofApp) | **L** |
| S | **Badge `is_open_source` spoofable a l'ingress `/browse` (S76 B2, CARRY-3)** : un `ProjectAnnouncement` gossip (non signe) peut porter `is_open_source=true` SANS `provenance_hash`/`repo_url` — le badge servi par `/browse` (et le « verrou 4 » front, qui lit `source=="direct"` + `is_open_source`) afficherait alors une fausse source-verifiable, pilotant le fork-consumer et le consent L2 worker sur une revendication forgee | M | **S76 B2 : downgrade `is_open_source`→`false` a l'INGRESS aggregator (`runtime::handle_project_announcement`, AVANT `add_direct_entry`), pas seulement a l'index FTS5 (S74 B.6) — meme predicat partage `trustworthy_open_source(is_open_source && provenance_hash.is_some() && repo_url.is_some())`. Test `aggregator_downgrades_open_source_without_provenance`. Note cardinale : le « verrou 4 » est une garantie DECLARATIVE (provenance presente), JAMAIS une attestation cryptographique que l'archive a ete batie depuis ce repo — un tiers peut verifier la provenance, pas (encore) reproduire le build.** | **L** |

**Residuals S75** : fresh-flood (M, sonde=autorite) et duress des
freres pre-existants LOCAL-ONLY (L, route S76 ; les chemins boot
wire-emit sont gates par l'audit S75 DURESS-BOOT-LEAK). Le sur-comptage §15 row D reste
M : `known_entry_count` agrege le 3e bras nodedirectory en best-effort
(sur-estimation toleree, jamais une preuve de joignabilite).

**Residuel — decouvrabilite d'une app pur-seedee (`catalog_len:0` ;
decision fermante PO-8 accept-and-document, S82 Phase I, 2026-07).**
Constat (acceptance live S75-G, re-observe au flip S81, S81-G-3) :
l'annuaire signe d'un noeud qui SEEDE l'app d'autrui sans l'avoir
publiee reste `catalog_len:0`. C'est PAR CONSTRUCTION : le catalogue
est bati exclusivement depuis `own_entries(&my_node_id)`
(`build_sign_announce_directory` dans `publish_api.rs` ; `own_entries` dans
`browse.rs` filtre sur `node_id == my_node_id`) — une app
volontairement seedee garde le node_id de l'AUTEUR et n'est jamais un
direct-entry (test `seed_voluntary_directory_only_app`). Semantique de
signature (contrat `node_directory.rs`) : la signature d'annuaire
atteste l'HEBERGEMENT (« I claim to host these hashes »), PAS la
paternite — le catalogue PEUT au contrat porter des apps « hosts (or
seeds) », et verrou-4 garantit seulement que le seeder ne signe jamais
la PROVENANCE de l'app (l'`archive_hash` reste celui de l'auteur).
L'exclusion own-published-only est donc une POLITIQUE du daemon
(`own_entries`), choix conservateur code-side, pas une contrainte du
wire. Nature : trou de DECOUVRABILITE borne, PAS de securite — la
joignabilite des octets reste intacte (blob content-adresse servi par
le seeder, fetch multi-provider une fois le hash connu) ; seule la
decouverte INITIALE via un pur-seeder manque. Si l'auteur disparait
(annuaire plus jamais re-publie), un pair frais dont la seule ancre
abonnee est un pur-seeder perd le chemin de decouverte meme si les
octets restent disponibles (disponibilite != decouvrabilite). Le
comptage de joignabilite passe par seed-count/BLAKE3, jamais par le
catalogue d'annuaire. Decision PO-8 : accept-and-document — l'item
sort du cycle de carry (report repete S75 origine -> S76 [2/3] ->
S77 [3/3] -> S78 -> S81-G-3 ; compteur §6.2.1 solde). REOUVERTURE
uniquement si declenchee par une perte de decouvrabilite observee a
l'echelle pilote, par l'une de ces voies : (a) inclusion NON etiquetee
des hashes seedes dans le catalogue — CODE-ONLY, deja compatible avec
le wire actuel (« hosts (or seeds) ») mais perd la distinction
publie/seede ; (b) une section « seeded » DISTINCTE et NON-autoritaire
dans `NodeDirectoryEntry` — changement wire ; (c) l'index reseau-large
signe opt-in (SearchManifest, post-launch). Miroir EN :
`docs/rust/PATTERNS.md` §P59.8.

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
| I | **Verification cross-hardware semantique manquante** (l'exact-match ne couvre pas les GPU heterogenes) | L | **etage 2 primitive CABLEE (S77 Phase G)** : primitive N0 TOPLOC (`toploc.rs`, commitment BLAKE3 du sketch entier top-k du dernier hidden state) + helper worker qui le calcule + Layer-3 `verification.rs` consommant le commitment dans `logprobs_hash` par egalite (detection ~100% du swap modele/precision par INEGALITE). L'**ecriture/emission signee** du commitment dans `RunProof.activation_fingerprint` est **LIVREE S81 Phase J** (le decode loop bind le commitment N0 du DERNIER step au `RunProof` driver signe, premiere emission production) ; le miroir `ResultPayload.logprobs_hash` sur le chemin task classique reste porte par le worker (S76). La comparaison TOLERANTE cross-GPU (`ToplocFingerprint::compare`) est recomputee independamment par N1 (Phase H) / N2 (Phase I) — voir §16 « N0 TOPLOC fingerprint ». **Phase H** cable la PRIMITIVE N1 (tirage verifiable Ed25519 `verifiable_draw.rs` + recompute tolerant `ToplocFingerprint::compare` + Token-DiFR + incentive reputationnel `spotcheck_creditable`/`kudos_ledger::credit` + mapping criticite→niveau `criticality_maps_to_verification_level`) ; le re-exec prefill REEL sur GPU + le transport du sketch complet hors du slot 32B restent gates (**re-routes S82**, ex-S78 — l'orchestrateur in-vivo etant livre S81 I/J, seul le canal de retour control-plane manque), comme G a livre la primitive N0 sans cablage in-vivo. **Phase I** cable les PRIMITIVES N2 (quorum tolerant M-of-N `redundancy.rs::tolerant_quorum_accepts` + chemin ADDITIF `validator.rs::validate_tolerant_quorum_shard`, verdict sur `RunProof` SIGNES, quorum exact `validate_quorum_pre_guardrail` INCHANGE) + N3 (commit-reveal `activation_commit.rs` `DOMAIN_ACTIVATION_COMMIT_V1` + localisateur EMA forward-only `sentinel.rs`) — voir §16 « N2 » / « N3 » ; le re-exec REEL cross-GPU in-vivo + le transport du sketch sur le data-plane + la bissection-sur-litige restent gates (**re-routes S82**, ex-S78) | **M (primitives N0/N1/N2/N3 CABLEES S77 G/H/I ; orchestrateur + RunProof driver in-vivo LIVRES S81 I/J ; re-exec in-vivo + transport sketch + arbitrage litige = S82 → L une fois in-vivo)** |

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
| T | **Gonflage de kudos** (un worker solo declare `tokens_generated` absurde, ex. 1e9 tokens en 5 ms, pour farmer la reputation) | M | sanity-bound `tokens <= TOKENS_PER_MS_CEILING * max(1, generation_time_ms)` AVANT `log_utility` (`kudos_ledger::credit`, applique aux 2 sites prod `validator_loop.rs` + `coordinator_api.rs`) ; ferme la fuite de valeur absolue que `log_utility` (<10x marginal) laisse ouverte ; ancrage BOINC `wu.rsc_fpops_bound` | **L** |
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

#### §15.3.1 Extension Sprint 82 Phase A — cold-boot catch-up (boot-SEED, S81-G-ESC-1)

Deux ajouts sous l'invariant unifie « le broadcast gossip est un HINT non fiable ;
l'etat durable synchronise est la VERITE ; tout consommateur cold-boot RECONCILIE ».
**Aucune frontiere d'admission nouvelle, 0 bump wire, 0 dep.**

- **WORKER — cadence de rejoin cold-boot** (`doc_sync.rs`, `KeepaliveConfig::
  cold_boot_aggressive`) : jusqu'au 1er `NeighborUp` apres (re)boot **OU la deadline
  wall-clock `DEFAULT_COLD_BOOT_WINDOW` (60s, filet si l'edge NeighborUp est manque)**,
  le keepalive re-emet `start_sync` toutes les ~1s au lieu de 15s, puis relache vers le
  backstop S77. Meme `start_sync`, memes `peers` (le coordinateur du ticket), meme chemin pkarr
  — seule la FREQUENCE change pendant la fenetre cold-boot. Amplification bornee par
  `cold_boot_min_rejoin_interval` (cooldown ~1s) : au plus 1 `start_sync`/s vers le
  seul coordinateur deja detenu, jamais un fan-out. Surface identique a §15.3 ; residual
  nul.
- **ANCRE — re-drive-on-ingest** (`runtime.rs::maybe_redrive_seed_on_ingest`) : quand
  un annuaire d'un ancre ABONNE est accepte (gate S75 Phase C : subscription + Ed25519
  + attribution + anti-rollback), le boot seed driver est rejoue pour re-pin une app
  `keep_online` dont l'annuaire arrive apres le boot (fermeture de la « first-boot dead
  window »). Surface d'amplification/DoS **doublement gardee** : (a) le driver itere
  UNIQUEMENT `configured = [seed] keep_online_projects` (accept-list operateur) — un
  pid venu du reseau n'y entre JAMAIS ; (b) **single-flight + dirty** (`RedriveCoord`) :
  au plus UNE chaine de re-drive tourne a la fois ; un ingest arrivant pendant une passe
  positionne `dirty` et la chaine fait UNE passe trailing qui le re-couvre — vrai
  COALESCING (aucun trigger perdu, pas d'empilement sur le lock), les passes trailing
  espacees par `REDRIVE_MIN_INTERVAL` (30s) pour qu'un ancre bumpant vite ne spinne pas
  le driver reseau-lourd `fetch_and_pin_multi` ; (c) **duress-gate en TETE de
  `maybe_redrive_seed_on_ingest`** (retour avant meme le clone de `configured`) ET de
  `run_boot_seed_driver` (defense-in-depth, retour 0 avant toute resolution/fetch/DB/log) —
  aucune lecture OBSERVABLE (log / DB / resolution / fetch / emit) de donnees reelles
  avant l'un des deux gates (classe DURESS-BOOT-LEAK ; le clone in-memory de la liste
  configuree, comme celui du boot driver, n'est pas une exfiltration) ; (d) BLAKE3 = verite
  joignabilite (guard `h == want_hash`, `delete_tag` sur mismatch) — une annonce forgee
  ne fait jamais servir d'octets absents. Course double-annonce `SeedAnnounced` vs boot
  driver en vol : serialisee par un `seed_driver_lock` (mutex partage). Invariant
  cardinal `heberger != publier, seeder != auteur` tenu : le re-drive re-pin le hash du
  1er ancre lexicographique advertising (residuel Sybil-sampling S76 inchange), pas une
  provenance auteur.

### 15.4 Extension Sprint 81 — mode zero-n0 self-hosted + hot-join + stores migres (E2/E3/F)

S81 prepare l'EOL des services n0 (relais + pkarr publics, 2026-09-30)
par un mode **zero-n0 opt-in gated-env** (`SBFB_ZERO_N0`,
`discovery_override.rs` fonction de decision pure + chokepoint unique
`apply_zero_n0_discovery` dans `node.rs` ; `presets::N0` reste le
DEFAUT). Topologie live actee PO 2026-07-05 (`bf07960`) : **B
co-logee** — iroh-relay + iroh-dns-server derriere Caddy sur l'ancre
existante, cout 0 ; convergence LIVE prouvee (`a085853`, T2 PASS).
Detail operationnel : `docs/release/IROH_SELFHOST_OPS.md` (§8 du
runbook pointe vers cette section).

**Relocation de confiance n0 → operateur, BORNEE Ed25519.** Le paquet
pkarr reste signe par la cle du node : un relais pkarr hostile (ou
compromis) peut au pire **refuser ou perimer** une reponse, jamais
forger une adresse — meme borne que le modele pkarr deja accepte
(§15.3 keepalive). Le relais iroh self-hosted voit les **metadonnees**
de connexion (qui parle a qui, quand), jamais le contenu (QUIC
chiffre de bout en bout). La relocation deplace la confiance
*disponibilite + metadonnees* de n0 vers l'operateur ; elle ne cree
aucune capacite de forge.

| Menace | Exemple | Sev. brute | Mitigation | res |
|---|---|:---:|---|:---:|
| D | **SPOF operateur** : zero-n0 CONCENTRE relais + pkarr + ancre sur l'infra d'UN operateur ; la mort du host coupe discovery ET relay d'un coup (complementaire, PAS superset, de survives-VPS-death S75 — S75 prouvait la survie du RESEAU a la mort de l'ancre, pas la survie du mode zero-n0 a la mort de son host) | H | Runbook exige **>=2 relais pkarr DISTINCTS non-n0** (`SBFB_ZERO_N0_PKARR_RELAYS` liste) + recommande **host relais != host ancre** (`IROH_SELFHOST_OPS §7/§8`) ; Topologie B (co-logee) ACCEPTE temporairement ce cumul, cf. re-decision ci-dessous | **M** |
| I | **Jointure metadonnees-relais x contenu-ancre** : en Topologie B le MEME operateur voit les metadonnees de connexion (relais) et sert le contenu (ancre/blob-serve) — capacite de correlation superieure a n0 (qui ne voyait pas le contenu) | M | Borne Ed25519 (0 forge) + contenu public par design (pilote ferme) ; separation des hosts = Topologie A, cf. re-decision | **M** |
| D | **Silent-loss discovery ELARGIE** : 1 host operateur vs flotte n0 multi-region — une panne discovery silencieuse est plus probable et plus totale | M | **Fail-loud coupling** (E2) : `SBFB_ZERO_N0` sans relais pkarr configure = `Err` au boot, jamais un demarrage silencieusement sourd ; tripwire survie URL pkarr (Phase E, check nomme) | **L** |
| S | Relais pkarr hostile forge une adresse | H | IMPOSSIBLE par construction : paquet pkarr signe Ed25519 par la cle du node, verifie par le resolveur pkarr d'iroh (`iroh::…::PkarrResolver`, cable au chokepoint zero-n0 `node.rs`) — meme modele de confiance pkarr que §15.3 | **Nil** |

**Re-decision Topologie A-vs-B — AVANT le 25/08 (decision PO OUVERTE,
tracee ici, PAS tranchee par S81-G).** B (co-logee, actuelle) accepte
le SPOF-cumul + la jointure ci-dessus + **QUIC address discovery off**
(constat acceptance `a085853`) contre un cout 0. A (host dedie pour
relais+pkarr) reduit SPOF et jointure contre un cout host. Echeance
alignee sur le gate calendaire C8 (bascule flotte 25/08).

**Residuel T20 (carry, NON ferme par S81)** : la posture TLS du
resolveur pkarr HTTP reste **WebPKI-only** — le hook amont
`tls_pinning`/`PinValidator` existe dans iroh 1.0.1 mais
`apply_zero_n0_discovery` prod ne pose pas de `ca_tls_config`
(`insecure_skip_verify` reste `#[cfg(test)]`-only). Cablage
PinValidator = carry S82+.

**Hot-join du curateur souscrit (E3, `e05338f`)** — residuels doc :

- **Asymetrie unsubscribe** : iroh-gossip 0.101 n'expose aucun verbe
  `leave` (`Command` = Broadcast/BroadcastNeighbors/JoinPeers) → apres
  desabonnement, le pair reste voisin HyParView jusqu'au churn
  naturel ; l'ingest est droppe par `is_subscribed=false` → fuite
  bornee au transport (metadonnees de voisinage), zero ingest.
- **Boot-duress (residu PRE-EXISTANT, elargi par la surface)** : sous
  cle leurre, le boot dial les subscribes + re-pull + rejoue l'outbox
  (patterns reseau observables). E3 ne l'AGGRAVE pas : le hot-join est
  duress-safe-par-placement (push apres l'early-return duress, 0 dial
  nouveau sous duress, verrouille par test negatif).
- **Reconnexion-apres-drop** : le hot-join n'ajoute PAS le pair au
  bootstrap-set du topic fige — apres une rupture de transport, le
  re-bootstrap ne reprend qu'au reboot (carry boot-only pattern, K).

**Stores migres redb 2→4 (F, `70dd845`)** — residuels doc (detail
operationnel : `docs/release/STORE_MIGRATION_OPS.md`) :

- **T-STORE-MIGRATION-CRASHWINDOW (residuel L)** : fenetre
  rename↔persist de la migration docs (temp+swap one-way) ; la garde
  `refuse_recreate_on_interrupted_migration` (F, aux 2 boundaries)
  refuse le recreate silencieux quand le backup a survecu (rename
  FIRST par construction) ; tar snapshot OBLIGATOIRE avant toute
  migration reelle (Win + Mac PRIS) ; caveat Linux rename-clobber.
- **T-STORE-FIXTURE-LEAK** : la migration cree
  `docs.redb.backup-redb-v2-tuples` (contient l'ancien
  `NamespaceSecret`) + `docs.db.migrate<rand>` non auto-nettoyes →
  nettoyage manuel au runbook.
- **T-BLOBS-DURABILITY (degrade, note d'honnetete)** : ce §15 couvre
  l'INTEGRITE (BLAKE3), pas la DURABILITE — un wipe blobs reste
  re-importable depuis `iroh/blobs/data/*.data` (content-addressed),
  et blobs v3 ouvre in-place sous 0.103 (le scenario wipe est moot
  pour l'upgrade S81).

### 15.5 Extension Sprint 81 — operation de flip LIVE 0.98 → 1.0.1 (H)

§15.4 couvre le MECANISME (mode zero-n0, hot-join, migration des
stores) ; cette section couvre l'OPERATION : la session same-day qui
bascule les 3 noeuds live (dev Win, Mac M2, ancre VPS) du binaire
0.98 au binaire 1.0.1. Runbook : `docs/release/LIVE_FLIP_RUNBOOK.md` ;
mecanique store + rollback : `docs/release/STORE_MIGRATION_OPS.md`.
Contexte de menace decisif (C4/C5) : **aucun noeud tiers n'existe** —
toutes les menaces du flip sont self-inflicted et ne touchent que
les donnees de l'operateur ; les severites residuelles sont bornees
en consequence.

| Menace | Exemple | Sev. brute | Mitigation | res |
|---|---|:---:|---|:---:|
| D | **Partition totale intra-fenetre** : wire docs/gossip 0.98↔1.0 non-retrocompat — toute paire mixte est partitionnee, quel que soit l'ordre de flip | M | Flag-day same-day UNE session + gel publish/ingest (discipline operateur, aucun verrou code) ; l'ordre ne borne PAS la partition, il minimise le downtime du seeder (VPS dernier) ; sous C4/C5 seuls NOS 3 noeuds perdent la cross-gossip pendant la fenetre | **L** |
| D/I | **Perte de store sur flip rate** : crash mid-migration ou rollback incorrect — restaurer le tar PUIS rebooter sous 1.0.1 RE-MIGRE immediatement (migration automatique a l'ouverture, one-way) et rejoue le flip rate au lieu de l'annuler | H | Rollback = **DEUX gestes** (restore tar + redeploy binaire 0.98 conserve cote-a-cote) ; tar per-noeud daemon-ARRETE NON-skippable (seul filet universel de la crash-window) ; garde `refuse_recreate_on_interrupted_migration` (§15.4) ; sur VPS Linux : TAR, jamais rename (clobber) | **L** |
| S/D | **Regression d'identite silencieuse** : `load_or_generate_node_key` REGENERE en warn-only si `node_key` != 32 octets (tar tronque, restore partiel) → nouveau `node_id`, locators abonnes casses SANS erreur | M | Assert empirique post-boot : `flip_convergence_check.sh` compare `node_id` a la reference capturee pre-flip (`EXPECT_NODE_ID`) ; sur le VPS le mode **fail-closed `REQUIRE_NODE_ID=1` est OBLIGATOIRE** (une reference absente = RIG-ABSENT, jamais un skip silencieux — cet assert est le SEUL backstop automatique de la regen warn-only ; la sante LOCALE seule ne detecte PAS une regeneration, blob-serve est content-addressed) → BLOCK + STOP + rollback ; re-install stock INTERDIT (D3 cond.5/R1) ; verif restaurabilite du tar (node_key 32 octets) ; `SBFB_IDENTITY_SECRET_HEX` interdit dans la session. Le residuel L est CONTINGENT a cette discipline sur le VPS | **L** |
| I | **Faux verdict de convergence** : verdict prose/curl manuel — un flip rate passe pour reussi et la fenetre se referme sur un etat divergent | M | Harness committe `scripts/acceptance/flip_convergence_check.sh` (contrat JSON vocabulaire ferme PASS/BLOCK/RIG-ABSENT) : sante LOCALE par noeud + convergence CROSS-noeud des le 2e noeud 1.0 (couple E3 : browse reachable + sha256 byte-identique) ; artefact T2 committe | **L** |
| D | **Flip pas fait avant l'EOL n0 (30/09)** : la flotte reste sur 0.98 avec des relais publics morts | H | Gate calendaire C8 15/09 → plan B self-hosted ACTIF (`IROH_SELFHOST_OPS.md`, pre-provisionne E2, T2 zero-n0 PASS `a085853`) ; ~2 mois de runway au preflight H (09/07) | **L** |

**Non-menaces (cadrage honnete)** : le gel publish/ingest est une
discipline anti-split-brain, pas une frontiere de securite (aucun
attaquant tiers pendant la fenetre sous C4/C5) ; la "fenetre bornee"
vient du same-day + zero-tiers, PAS de l'ordre de bascule ; la
propriete "partition totale 0.98↔1.0" est une propriete UPSTREAM
(iroh) non verifiable depuis SBFB — a confirmer empiriquement au flip
et logger dans l'artefact T2.

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
| **SI-5 Latence side-channel** | le temps de compute d'un layer revele la complexite du prompt (longueur, heads actifs) | Low | residuel ; padding constant-rate = raffinement post-benchmark (re-route **S82**, ex-S78 — le benchmark live S81 J existe desormais comme baseline) |

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
chemin modele entier) ; l'**ecriture dans un proof signe est LIVREE S81 Phase J**
pour le chemin shard : le decode loop (`drive_decode_loop`) bind le commitment N0
du DERNIER step (`toploc_hex` du `ShardStepReply` de la queue) dans le `RunProof`
DRIVER signe — le dernier step decide toujours, zeros = « not provided », jamais
un commitment perime. Les commitments PER-WORKER signes par chaque shard distant
exigent un canal de retour control-plane (feed raw-op / iroh-docs, jamais un ALPN
neuf) — re-routes **S82** (ex-S78). Cote
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
ne tolere rien. Les PRIMITIVES de recompute cross-worker sont N1 spot-check
(Phase H) + N2 redondance tolerante (Phase I) ; le transport du sketch complet
hors du slot 32B que ce recompute in-vivo exige est re-route **S82** (ex-S78).
Le live result path reste le quorum exact-match
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
(re-routes **S82**, ex-S78) — exactement le pattern Phase G (primitive sans
cablage in-vivo ; l'orchestrateur in-vivo, lui, est livre depuis S81 I/J).

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

### N2 redondance tolerante (Sprint 77 Phase I)

Phase I cable le **3e etage, N2** : une tache haute-criticite tourne sur
`redundancy_factor` workers et est acceptee ssi un **quorum tolerant** d'entre
eux corrobore le meme calcul. Comme les workers shard tournent sur des GPU
heterogenes (non-determinisme flottant), la corroboration est la comparaison
**TOLERANTE** `ToplocFingerprint::compare` generalisee M-of-N
(`redundancy.rs::tolerant_quorum_accepts`), **jamais** l'egalite byte de
`result_text`. Le quorum exact existant (`validate_quorum_pre_guardrail`) reste
**byte-pour-byte INCHANGE** : N2 est un chemin ADDITIF distinct
(`validator.rs::validate_tolerant_quorum_shard`), sur les fingerprints, pas sur
le texte.

**Non-falsifiabilite (load-bearing)** : *quel* niveau N2 s'applique est selectionne
depuis `criticality_maps_to_verification_level`, **advisory** car
`redundancy_factor` est exclu des canonical bytes (S23 `34c77ce`). Le verdict
ACCEPT/REJECT, lui, repose **uniquement sur des `RunProof` SIGNES**
(`DOMAIN_RUN_PROOF_V1`) : `validate_tolerant_quorum_shard` ne vote que sur les
soumissions dont (1) la signature verifie et (2) le sketch porte ouvre le
commitment N0 signe (`sketch.commitment() == proof.activation_fingerprint`) — le
carrier hors-slot ne peut donc pas etre falsifie, et une preuve forgee/non-signee
n'atteint jamais le vote.

**Mutual-agreement, pas pivot-star** : l'accord tolerant n'est PAS transitif
(`A ≈ B` et `B ≈ C` n'impliquent pas `A ≈ C`). Compter les fingerprints qui
s'accordent avec un seul pivot sur-compterait (un straddler isole gonflerait un
quorum inexistant) ; N2 exige une **clique** (`largest_agreeing_cluster`), le plus
gros ensemble deux-a-deux tolerant.

**Nouvelles surfaces N2** :
- **SI-6 collusion-dans-tolerance** (Sev **M**, instanciation INTEGRITE de SI-4) :
  M workers s'accordent sur un fingerprint *proche-mais-faux* dans la bande de
  tolerance → fausse acceptation. **Mitigation** : pilote ferme (D5) + anti-Sybil
  amont (PoW/AgeWitness) bornent une coalition ; jamais un stake economique
  (PO-12). Carry honnete (pas de garantie, comme l'anti-lazy-verifier N1).
- **SI-7 calibration du seuil de tolerance** (faux-accept si trop large Sev **H** /
  faux-reject cross-GPU si trop etroit Sev **L**) : le seuil est un **parametre de
  securite**. **Mitigation** : N2 REUTILISE les seuils calibres `TOPLOC_THRESH_*`
  (toploc.rs, arXiv:2501.16007 bf16) plutot que d'en inventer ; re-calibration sur
  le rig reel = **S82** (ex-S78 ; le run live S81 J fournit desormais la baseline
  5080-CUDA + M2-Metal).

**Confidentialite INCHANGEE** : SI-1/SI-4 High identiques — N2 recompute/compare,
ne chiffre rien.

### N3 commit-reveal d'activation + SENTINEL (Sprint 77 Phase I)

Phase I cable le **4e etage, N3** (escalade de litige uniquement, jamais derive de
la criticite) en **deux primitives orthogonales** :

- **`activation_commit` (opML-style)** : un worker s'engage, par frontiere
  inter-stage, sur `BLAKE3(sketch || nonce)` de son fingerprint
  (`DOMAIN_ACTIVATION_COMMIT_V1`, Ed25519 + JCS). En cas de litige il revele le
  sketch complet + nonce ; le verdict est en **deux temps** : binding (le reveal
  ouvre le commitment) PUIS **correction TOLERANTE** (`compare`), **jamais**
  l'egalite du commitment BLAKE3 (qui faux-rejette tout re-run honnete cross-GPU,
  avalanche 1 bit). Ce n'est donc **PAS** la soundness fraud-proof d'opML : SBFB
  n'a pas de VM deterministe bit-exacte (raison d'etre de TOPLOC) ; N3 ancre *quel*
  fingerprint un worker assume et localise une frontiere contestee, il ne prouve
  pas cryptographiquement la correction (= N4 zkML, hors-scope).
- **`sentinel` (EMA forward-only)** : localise *quelle* frontiere disputer, par EMA
  entiere (basis-points, no-float) du signal d'activation forward inter-stage. La
  localisation est **directe O(1)** par frontiere — PAS une bissection (qui serait
  O(log L)) ; assimiler « O(1) » et « bissection » est une erreur de categorie. La
  moitie gradient/backward de SENTINEL (arXiv:2603.03592, training) est inapplicable
  en inference forward-only et n'est pas repliquee.

**Nouvelles surfaces N3** :
- **SI-8 grinding du commitment** (Sev **M**) : un worker re-mappe son commit vers
  une autre frontiere/session a posteriori. **Mitigation** : `session_id` +
  `frontier_index` + `worker_pubkey` sont dans la pre-image SIGNEE (anti-replay
  cross-frontiere/cross-session), meme discipline que le seed N1 non-choisi.
- **SI-9 refus de reveal / withholding** (Sev **M**) : un worker conteste mais ne
  revele pas — et, plus large, un worker admis qui RETIENT sa reponse data-plane.
  **Mitigation CABLEE S81 Phases I/J, certifiee K** (cf. §16 « Certification
  SI-9 ») : deadline par-hop couvrant open+write+read, fallback resume-from-cache,
  coupe comptee. Le volet N3-reveal (withholding d'un REVEAL de litige in-vivo)
  reste re-route **S82** (ex-S78) — l'arbitrage de litige in-vivo n'est pas cable.
- **SI-10 replay cross-session du commit** : neutralise par `session_id` dans la
  signature (cf. SI-8).
- **SI-11 evasion lente / empoisonnement de baseline EMA** (Sev **M**) : un stage
  derive juste sous le seuil a chaque pas pour ne jamais flagger, ou empoisonne la
  baseline. **Mitigation partielle** : un outlier flagge ne met PAS a jour l'EMA
  (rejet d'outlier, anti-spike) ; mais le seuil est **statique** (pas la fence IQR
  adaptative du papier) et un drift lent sous-seuil reste possible. Carry honnete
  (re-calibration + signal de magnitude absolue = **S82**, ex-S78). Garantie
  crypto = N4 zkML, hors-scope.

**Gouvernance « qui arbitre »** : l'arbitre du reveal est un verifieur N1-style
(coordinateur ou pair tire), **PAS de smart-contract** (design fige). La sanction
d'une frontiere localisee corrompue est un **verdict de correction / rejet**,
**jamais** un slash monetaire (PO-12 ; `DOMAIN_KUDOS_V1` / `HashableKudosEntry`
intouches).

**Confidentialite INCHANGEE** : SI-1/SI-4 High identiques — N3 recompute/localise,
ne chiffre rien (le nonce du commit cache le fingerprint avant reveal, il ne
chiffre pas les activations en transit, qui circulent en clair dans le groupe
prive — SI-1).

> **Completion Phase K S77 (wrap-up, livre — REQUALIFIE S81 Phase K)** : STRIDE
> formel §5.9 + ligne LINDDUN §6 (flux shard frontier) + asset §2 A8 + note §4
> DFD (flux `sbfb/shard/1`) AJOUTES a S77. Ce que S77-K laissait en carry a
> depuis bouge : le stub `live_shard_session -> None` est SUPPRIME (S81 Phase I
> livre l'orchestrateur reel + les 5 routes `/api/daemon/shard-session/*`,
> projection whitelist inchangee : `member_count` agrege, jamais
> `worker_pubkey`/`initiator`) ; le benchmark cross-machine n'est PLUS
> RIG-ABSENT (S81 Phase J : CodeLlama-34B eclate 5080-CUDA + M2-Metal, PASS,
> `sprint81_t2_j_shard_inference.json`) ; l'emission signee in-vivo du
> `RunProof` DRIVER est LIVREE (S81 I/J) ; le cablage SI-9 timeout/fallback
> data-plane est LIVRE (S81 I/J, certifie ci-dessous). Restent re-routes
> **S82** : SI-5 padding (le benchmark J en est la baseline), RunProofs
> per-worker + transport du sketch, arbitrage de litige in-vivo (N3-reveal).

### Attestation loaded-stage <-> manifeste signe + certification SI-9 (Sprint 81 I/J/K)

**Le trou ferme (S81 Phase K, carry P1 de Phase J)** : jusqu'a J inclus, le
chemin `shard-session serve` choisissait fenetre et role par flags CLI et ne
voyait JAMAIS le manifeste signe (`authorize_claim` ne vit que sur le chemin
claim-engine, mort sur serve) ; la readiness etait transport-only (handshake +
RTT, 0 frame). Un stage mal configure — mauvais GGUF, mauvaise fenetre, echo
transport-only laisse en service — produisait un resultat plausible-faux que le
driver SIGNAIT dans son RunProof.

**Enforcement** : a l'etablissement de CHAQUE stage-link d'une session REELLE
(`model_digest != 0` — premier drive, re-dial, ET re-route fallback), le driver
exige une **attestation applicative** (`ShardStageAttestationRequest`/
`ShardStageAttestation`, message JSON dans les frames opaques `sbfb/shard/1`,
pattern `SHARD_STEP_PAYLOAD_V`, 0 bump wire) : le stage declare
`{model_digest (blake3 STREAMING du FICHIER au chemin GGUF, cf. caveat TOCTOU
SI-12 ci-dessous), layer_start, layer_end,
is_first, is_last}` ; le driver compare au `ShardedSessionManifest` signe +
`ShardAssignment` du stage et **fail-close** tout mismatch AVANT le moindre
frame de donnees (`attest_stage_link`, meme budget deadline SI-9 que tout hop).
`ShardProtocol::accept` repond a la requete AVANT le forwarder — un backend
reel ne voit jamais la sonde comme des activations (concern R-I-1 preserve) ;
le chemin echo (digest zeros) est exempte : ses frames echo/transport restent
byte-identiques a S77 Phase B COTE DRIVER, tandis que l'accept-loop partage
(non byte-identique, lui) porte la branche d'interception d'attestation pour
toute session de l'ALPN — branche qui ne se declenche que pour un stage REEL
(garde `is_real_stage` : un echo transport-only n'intercepte jamais).

**Honnetete cardinale** : l'attestation est un **self-claim d'un membre admis**
(famille N0). Elle ferme la classe **MISCONFIGURATION** ; un stage qui MENT
deliberement sur son digest reste le residuel SI-4/N0 (detection par
inegalite de commitment TOPLOC + quorum tolerant N2, inchanges).
**Caveat TOCTOU (SI-12, Sev L, hote-local, re-route S82)** : le digest attest
est le blake3 STREAMING du FICHIER au chemin GGUF au moment du hash, calcule
APRES le load (le backend llama.cpp mmap le fichier par defaut). Un remplacement
atomique du fichier entre le load et le hash permettrait a un operateur local
de servir l'inode mmape ancien tout en attestant le digest du nouveau chemin.
C'est une surface **hote-local/trusted** (l'operateur controle deja sa machine),
donc L ; le binding n'est PAS atomique load<->hash. Durcissement (hash de la
region mmapee, ou hash-avant-load + re-verification) = re-route S82.

**Certification SI-9 data-plane (S81 I/J, perimetre honnete)** :
- deadline PAR-HOP couvrant open_bi + write + read (`drive_hop`, `step_hop` —
  le write path backpressure sur le flow-control QUIC, donc borner la lecture
  seule laisserait un byzantin bloquer un write large indefiniment) ;
- fallback plan-time re-probe + attestation + **resume-from-cache**
  (`ActivationReplayCache`, replay stateless du step input — correct par
  construction, aucun etat perdu), coupe explicite comptee
  (`worker_drop_count`) sans fallback ;
- prouve HERMETIQUEMENT (deadline write-path ferme, re-route mid-decode
  rejoue) + LIVE post-drive (coupe comptee) ; le drop mid-decode LIVE
  (arracher une machine pendant le decode) exigerait une 3e machine rig
  (R-J-5, `sprint81_t2_j_shard_inference.json`) — compte, pas simule.

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
- **v13 (Sprint 77 Phase I, 2026-06-22)** : cablage **primitives N2 redondance
  tolerante + N3 commit-reveal/SENTINEL** — ajout sous-section §16 « N2 redondance
  tolerante (Phase I) » (quorum tolerant M-of-N `redundancy.rs::tolerant_quorum_accepts`
  reutilisant `ToplocFingerprint::compare`, clique d'accord mutuel anti-straddle ;
  chemin ADDITIF `validator.rs::validate_tolerant_quorum_shard` sur `RunProof`
  SIGNES, quorum exact `validate_quorum_pre_guardrail` byte-pour-byte INCHANGE ;
  2 surfaces SI-6 collusion-dans-tolerance Sev M + SI-7 calibration-seuil
  H/L mitigee par reutilisation `TOPLOC_THRESH_*`) + sous-section « N3 commit-reveal
  d'activation + SENTINEL » (commit-reveal `activation_commit.rs`
  `DOMAIN_ACTIVATION_COMMIT_V1` verdict tolerant `compare` JAMAIS egalite-commitment ;
  localisateur EMA forward-only entier `sentinel.rs` O(1) direct, PAS bissection ;
  4 surfaces SI-8 grinding / SI-9 withholding / SI-10 replay / SI-11 drift-EMA, toutes
  Sev M, mitigations binding-signe / outlier-no-poison / carries honnetes) ; MAJ
  §15.2 row I (N2/N3 CABLEES Phase I, re-exec in-vivo + transport sketch + arbitrage
  litige = Phase J/K). Verdict ACCEPT/REJECT N2 sur inputs SIGNES (`redundancy_factor`
  reste advisory non-signe). PO-12 non-monetaire tenu (verdict de correction, jamais
  slash). Confidentialite SI-1/SI-4 High INCHANGEE (N2/N3 recomputent/localisent, ne
  chiffrent rien). 0 nouvelle row STRIDE, 0 bump wire (1 `DOMAIN_ACTIVATION_COMMIT_V1`
  additif, `*_FORMAT_VERSION` deja v1), 0 dep nouvelle.
- **v14 (Sprint 77 Phase K, 2026-06-22)** : **wrap-up + gate produit**, 0 code
  fonctionnel net. Harness d'acceptance `scripts/acceptance/b3_shard_pipeline.sh`
  (artefact JSON T2 `{status,stage,model,n_shards,ttft_s,toks_per_s,
  rtt_frontier_ms,run_proof,diagnosis,last_response}`, exit PASS=0/BLOCK=1/
  RIG-ABSENT=3, gate anti-faux-vert : `pass()` exige `run_proof` non-vide ET
  `toks_per_s >= 1`). **Statut T2 = RIG-ABSENT** : aucun orchestrateur de session
  prod ne monte/pilote une generation cross-shard ni n'emet de `RunProof` in-vivo
  (aucun caller prod de `RunProof::new`/`RunProofEntry::sign` ; les seuls appels
  vivent sous `#[cfg(test)]`), la route `GET /api/daemon/shard-session`
  est un stub `None` → la feature shard reste **PROVISIONAL + carry P1 S78**.
  Ajouts THREAT : §2 A8 (activations+RunProof), §4 note flux `sbfb/shard/1`,
  §5.9 STRIDE sharding, ligne §6 LINDDUN shard frontier. **Correction d'honnetete**
  : les forward-refs « Phase J/K » / « Phase J/data-plane » / « = Phase K » des
  blocs anterieurs (re-exec in-vivo, transport sketch, arbitrage litige, SI-9
  timeout/fallback, SI-11 re-calibration, SI-5 padding) sont re-cibles **S78** —
  Phase K (wrap-up) ne les livre pas. Le coeur sharding (placement D, routing E,
  fork F, claim F2, primitives N0-N3 G/H/I, front J) reste LIVRE + teste
  hermetiquement. 0 bump wire, 0 dep, 0 nouvelle row STRIDE de surface (§5.9 =
  resume du catalogue §16 deja fige).
- **v15 (Sprint 81 Phase G, 2026-07-08)** : consolidation doc de l'upgrade
  **iroh 0.98 → =1.0.1** (docs/gossip 0.101, blobs 0.103). MAJ §1.1 (stack
  versions), §5.4 row E (pin =1.0.1 + rationale wire-freeze reduit le churn
  de deserialisation, **residuel reste M** + note « upgrade ≠ Gate 1/Gate 3,
  R-iroh-audit P0 INCHANGE, pilote reste ferme »), §14 nit S80-H-4 (le SSE
  front = fetch+ReadableStream, jamais EventSource ; le cookie couvre le
  WebSocket PTY). Ajout **§15.4** — surface zero-n0 self-hosted (E2 :
  relocation trust n0→operateur bornee Ed25519, SPOF-cumul + jointure
  metadonnees×contenu en Topologie B, silent-loss elargie fail-loud,
  re-decision Topologie A-vs-B AVANT 25/08 TRACEE decision PO ouverte,
  residuel T20 PinValidator WebPKI-only carry) + residuels hot-join E3
  (asymetrie unsubscribe sans verbe leave, boot-duress pre-existant
  non-aggrave, reconnexion-apres-drop boot-only) + residuels stores F
  (T-STORE-MIGRATION-CRASHWINDOW L, T-STORE-FIXTURE-LEAK, T-BLOBS-DURABILITY
  degrade). Gate convergence supply-chain JOUE : lock NON convergent
  (ed25519-dalek 2.2.0 + 3.0.0-rc.0 interne iroh) → **P2-AUDIT-2-RESIDUEL
  carry S82**, `deny.toml` multiple-versions reste warn ; advisories
  remediees (2 cargo update + 6 ignore-with-reason racines
  hickory-0.24/quick-xml-iroh, carry HICKORY-024-RUSTSEC S82). 0 bump wire,
  0 dep runtime neuve, 0 nouvelle row STRIDE hors §15.4.
- **v16 (Sprint 81 Phase H, 2026-07-09)** : ajout **§15.5** — l'OPERATION
  de flip LIVE 0.98 → 1.0.1 comme surface distincte du mecanisme §15.4
  (gap de completude releve au preflight H, verdict PLAN-ADAPT). 5 rows
  STRIDE-lite, toutes residuelles **L** sous C4/C5 (aucun noeud tiers,
  menaces self-inflicted) : partition totale intra-fenetre (flag-day
  same-day, l'ordre ne borne PAS la partition), perte de store sur flip
  rate (rollback corrige = **DEUX gestes** restore tar + redeploy 0.98 —
  le restore seul RE-MIGRE au reboot ; R2 REFUTED au preflight), regression
  d'identite silencieuse (regeneration warn-only `node_key` != 32 octets →
  assert empirique `EXPECT_NODE_ID` du harness), faux verdict de
  convergence (harness committe `flip_convergence_check.sh`, contrat JSON
  vocabulaire ferme), EOL n0 30/09 (gate C8 15/09 → plan B). Runbook
  operationnel neuf `docs/release/LIVE_FLIP_RUNBOOK.md` + corrections
  `STORE_MIGRATION_OPS.md` (rollback 2 gestes, portee snapshot DEUX roots
  + checklist survivants, TAR-pas-rename sur VPS Linux). 0 bump wire,
  0 dep, 0 code runtime (phase operationnelle, delta tests 0).
- **v17 (Sprint 81 Phase K, 2026-07-11)** : wrap-up bi-axe. **Sweep GLOBAL
  des refs « S78 »** (15 sites) : le S78 differe est ABSORBE par S81 —
  chaque ref vivante est requalifiee « LIVRE S81 I/J/K » (orchestrateur de
  session in-vivo, emission production du `RunProof` driver + binding
  fingerprint N0 dernier-step, benchmark live 5080+M2 PASS, SI-9
  data-plane) ou « re-route S82 » (RunProofs per-worker + transport sketch
  via canal control-plane, arbitrage litige N3-reveal in-vivo, SI-5
  padding, SI-7/SI-11 re-calibration) ; les mentions S78 des entrees
  HISTORIQUES v12-v14 restent verbatim (archives datees). Ajout §16
  **« Attestation loaded-stage <-> manifeste signe + certification
  SI-9 »** : le carry P1 de Phase J est FERME — attestation applicative
  fail-closed a l'etablissement de chaque stage-link (message 0-bump dans
  les frames opaques, self-claim N0, ferme la MISCONFIGURATION, SI-4
  byzantin residuel inchange) + certification SI-9 au perimetre honnete
  (hermetique + live post-drive ; drop mid-decode live = 3e machine rig,
  R-J-5). **Note N/A trigger GUARDRAILS** : `sanitize_diagnostic`
  (shard_session.rs, S81 Phase I disposition Codex) est un scrubber
  d'hygiene de diagnostics transport (anti log-injection + redaction
  identites) — ce n'est PAS un checker de task-output et il ne rejoint
  pas la GuardrailChain (`GUARDRAILS_ARCHITECTURE.md` intouche, motif :
  surface = erreurs QUIC attacker-influenced, jamais du contenu LLM).
  Carry v17 : l'assertion machine `conn_type == direct` au
  readiness-barrier (J-D5-1) reste portee par le label honnete de
  l'agregat T2 (`sprint81_t2_acceptance.json`) → route S82. 0 bump wire,
  0 dep ; +1 surface applicative DANS l'ALPN existant (attestation),
  documentee ci-dessus.
- **v18 (Sprint 82 Phase K, 2026-07-15)** : supply-chain DNS fallback —
  bump hickory-resolver 0.24 → 0.26 (construction resolver reecrite
  `dns_fallback.rs` : Resolver/builder + `NameServerConfig::new` +
  `ConnectionConfig` ; per-endpoint TLS name P2-E-1 PRESERVE ;
  `trust_negative_responses=false` rendu EXPLICITE — le defaut upstream
  a bascule a `true` et le caching negatif defairait la course DoH/DoT).
  Les 4 RUSTSEC racine hickory-0.24 sont FERMEES, ignores `deny.toml`
  RETIRES : RUSTSEC-2026-0119 (hickory-proto O(n^2) name-compression,
  classe DoS) close par hickory-proto 0.26.1 (seuil exact 0.26.1) ;
  RUSTSEC-2026-0098/0099 (laxite name-constraints, classe
  AUTHENTIFICATION, pas DoS) + RUSTSEC-2026-0104 (panic parsing CRL,
  DoS/availability) closes par rustls-webpki 0.103.13 (seuils 0.103.12
  / 0.103.13). La validation de certificats DoH/DoT est RENFORCEE
  (rustls 0.21.12 → 0.23.40) — l'argument residuel S81-G
  « name-constrained intermediates hors trust path » devient sans
  objet. Magasin de racines desormais EXPLICITE (feature
  `webpki-roots`, deja au lock : 0.26 ne fournit AUCUN root store par
  defaut, l'omission aurait casse tous les handshakes en silence).
  Backend crypto : `ring` conserve, aucun `aws-lc-rs` au lock
  (verifie). Threat model du module INCHANGE (« DNS is not a trust
  anchor », pkarr Ed25519-verifie en aval) ; fallback toujours opt-in
  default-off. Remediation yanked meme commit : spin 0.9.8/0.10.0
  yankes upstream → 0.9.9/0.10.1 (lock-only, classe S81-G anyhow/
  crossbeam). 0 bump wire, 0 dep runtime neuve (l'arbre 0.24 legacy
  quitte le lock, l'arbre 0.26 etait deja present via iroh — collapse
  2→1 versions), 0 nouvelle row STRIDE, 0 nouvelle frontiere.
