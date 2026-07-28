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
| A9 | `needs[]` accessibilite (PII sante/handicap) | **Critique** (LINDDUN, categorie sensible > A4) | Moyenne | Faible | Host-local UNIQUEMENT (`AccessibilityPreferences`) ; JAMAIS dans la projection `ResolvedAccessibilityProfile.v1` (le type de sortie n'a pas de champ `needs`) ni au DOM/bridge/prompt/trace du Hub. Cf. §17. |
| A10 | Plan compile immuable (`CompiledPlan.v1`) | Faible (derive du pack) | **Critique** (autorite Plan-Then-Execute) | Moyenne | Artefact LOCAL derive par `compile()` (jamais signe ni gossipe) ; autorite = digest `content` du pack Ed25519-verifie en amont. Cf. §18. *(Row versee en v23 — l'asset etait defini §18 sans ligne ici.)* |
| A11 | Etat d'execution durable (run store) | Moyenne (identifiants caller-chosen, cf. T-RUN-ID-PII) | **Critique** (journal + outbox = verite du replay) | **Haute** (un run doit survivre au crash) | Fichier SQLite local injecte par l'hote (journal `run_events` + `outbox` + `inbox` + `snapshots` + `lease` + `waits`/`signals`/`timers` + `attempts` v26) ; jamais signe ni gossipe. Cf. §19. |
| A12 | Effets externes d'un run (side outputs, arbres de processus, mutations worktree, receipts d'activite) | Moyenne (receipts locaux : digests + facts fermes, jamais de prompt) | **Critique** (un double effet ou un faux succes falsifie le run) | Moyenne | HORS transaction store par nature (backend de la cible, FS du worktree, table de processus OS) ; receipt materiel en `outbox.receipt`, digest seul au journal. Cf. §19.2. |
| A13 | Grants et autorite effective (`effective_capabilities` = intersection 7 couches, 17 classes typees) | Faible (structures host in-memory, jamais serialisees ni signees ; 0 `DOMAIN_*`) | **Critique** (un grant elargi/forge ou une couche contournee = EoP directe sur les effets sensibles) | Moyenne (fail-closed la ou l'AUTORITE est requise : une audience de GRANT absente/vide/expiree ⇒ REFUS de la ref ; les couches ATTENUATRICES absentes sont l'IDENTITE — la couche 7 HITL vide ne refuse que les classes `Human`, pas tout ; jamais un deni d'un run deja clos) | Algebre PURE host-neutre (`crates/sbfb-workflow-engine/src/capability.rs`) ; chaque couche ne peut qu'ATTENUER (meet monotone) ; grants audience-scopes, expiry fail-closed a la borne, revocation per-call. Cf. §19.3. |
| A14 | Coffre de secrets hote (materiel de credential scelle au repos, lie a la `secret_ref` opaque juste-avant-l'appel) | **Critique** (materiel de credential ; le TYPE ferme la route SERIALISATION vers les 16 surfaces §14.3 — la route COPIE reste une discipline de la reliure, cf. §19.4 borne 15 ; seule la ref opaque circule) | Moyenne (AAD lie header+ref, downgrade KDF/migration de slot fail l'open) | Moyenne | Blob chiffre LOCAL (Argon2id(PIN)→KEK, AES-256-GCM, `DOMAIN_KEYSTORE_V1` reutilise, `SecretBox`/`zeroize`) ; `BoundSecret`/`VaultSecret` non-`Serialize` + `Debug` redacteur. Cf. §19.4. |
| A15 | Approbation HITL signee chrome hote (decision humaine liee au digest exact du tuple d'effet) | **Moyenne** (artefact LOCAL jamais gossipe, mais le tuple porte principal/destination/digest — metadonnees d'exploitation ; residuel `T-RUN-ID-PII`) | **Critique** (une approbation forgee/mutee = EoP sur un effet sensible) | Moyenne | Digest injectif du tuple 15-champs (`hitl.rs`) + signature Ed25519 hote context-locale (`hitl_approval.rs`) ; rendu EXCLUSIVEMENT chrome hote (allowlist bridge FERMEE de 16 verbes, aucun de HITL) ; toute mutation invalide. **PRIMITIVE non reliee : 0 call-site en production.** Cf. §19.4 (lire son DS-1 d'abord). |

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
           |  (16 methodes typees, intersection host-side autoritaire
           |   per-call [S83 Phase C, cf. §5.2 row I] / event)
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

**Flux accessibilite hote→app (Sprint 83 Phase B, A9)** : `AccessibilityPreferences`
(needs + axes, host-local) → `resolve()` (host) → `ResolvedAccessibilityProfile.v1`
(15 axes-effet, needs-free) → event bridge `sbfb-bridge-event`/`a11y-profile` →
iframe Hub sandboxee. `needs[]` (la CAUSE) ne franchit JAMAIS la barriere
host→app ; seul l'EFFET traverse. Cf. §17.

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
| I | Bridge leak des donnees cross-iframe | M | 16 methodes bridge typees + intersection d'autorite host-side per-call (Sprint 83 Phase C — cable ACTIVEMENT `declare-par-release ∩ grant` ; les couches `politique locale` et `consentement de l'effet` sont INERTES en v0 [policy toujours `null`/absente — PAS un ensemble vide qui refuserait tout ; consent hors interface], activees en L/L1/…) ; schema Zod strict | **L** |
| D | Flood de messages, denied-of-sevice coordinator | M | Correlation ID cap 1 MB/message + callback Set timeout (Phase B/D Sprint 15 discussion) | **M** |
| T-BRIDGE-USURP | App envoie un message VALIDE pour une methode non declaree / non accordee | M | Intersection fail-closed evaluee a CHAQUE appel (choke-point `useBridge` apres le source-check, AVANT tout dispatch y compris `pii_redact` offline) ; modele canonique `sbfb_manifest::grant_v0`, miroir TS `web/src/bridge/authority.ts`, parite sous gate `scripts/check-frontier-contracts.sh` | **L** |
| T-BRIDGE-GRANT-EXPIRY | Un grant expire continue d'autoriser | M | `grant.expires_at_ms` re-evalue PAR APPEL ; expire ⇒ ensemble effectif vide (fail-closed) | **L** |
| T-BRIDGE-LEGACY-DEFAULT | Une app SANS clef `bridge` (legacy/demo) reste permissive sur les 16 verbes du dispatch | L | Grandfather EXPLICITE, residu ACCEPTE pre-launch : source verifiable + sandbox CSP `connect-src 'none'` ; bascule « toutes les apps declarent » = post-v1.0 | **L** |
| T-BRIDGE-REVOKE-INFLIGHT | Revocation pendant la fenetre async d'un appel DEJA admis (`void dispatch(...)`) ; OU un `message` arrivant entre le commit d'un re-render et l'effet passif qui rafraichit `authorityRef` (juge par l'autorite PRECEDENTE) | L | `revocation_per_call` couvre le PRE-call (preuve autoritaire = modele `grant_v0`) ; la maj host-side de `authorityRef` se fait a l'effet post-commit — micro-fenetre ACCEPTEE, alignee sur les refs bridge existantes (coordUrl/appName) et sur `react-hooks/refs` (ecriture de ref en render interdite) ; l'annulation in-flight d'un appel deja parti est routee L/V/AI — hors scope Phase C | **L** |
| T-BRIDGE-RESOLVE-FAILOPEN | Une app qui DECLARE un `bridge` (regime fail-closed) est grandfathered aux 16 verbes pendant la fenetre async de resolution au 1er montage, ET durablement si le fetch `SBFB.json` echoue (repli grandfather, cache react-query ~60s) | L | Residu ACCEPTE pre-launch, NON-escaladant : une app hostile omettrait la clef `bridge` -> grandfather de toute facon, et l'intersection ne peut que RETRECIR la surface vs l'etat pre-Phase-C (aucun verbe NEUF ouvert). Disponibilite priorisee sur le durcissement-au-montage ; a resserrer post-v1.0 (resolution bloquante ou fail-closed-on-unreachable pour une app connue-declaree) | **L** |

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
| **T**ampering | Un worker execute un GGUF/quant different ou ment sur son forward | H | primitives N0 TOPLOC + N1 VRF + N2 quorum tolerant + N3 commit-reveal CABLEES+testees hermetiquement (S77 G/H/I) ; emission signee in-vivo du `RunProof` DRIVER **LIVREE S81 I/J** (fingerprint N0 bind au dernier step) ; **attestation loaded-stage fail-closed S81 K** (ferme la MISCONFIGURATION : mauvais GGUF/fenetre/role rejetes avant tout frame — un worker qui MENT dans son self-claim reste le residuel SI-4) ; RunProofs PER-WORKER distants = re-route **slot rig-chaud** (S82/D6) | **M** |
| **R**epudiation | Un worker nie son etat-frontiere | M | `RunProof` signe DRIVER emis in-vivo depuis S81 I/J (slot `activation_fingerprint` bind au commitment N0 du dernier step) ; RunProofs per-worker via canal control-plane = re-route **slot rig-chaud** (S82/D6) | **M** |
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
| Projection accessibilite host→app (A9) | **L (accepte)** — correlation des axes-effet cote app, mitige `connect-src 'none'` + effet≠cause | L (aucune identite) | L | L (projection locale, non-gossipee) | **L — `needs[]` (PII Art.9) reste host-local, la projection est needs-free PAR CONSTRUCTION (type de sortie sans champ `needs`)** | L (operateur regle ses propres preferences) | **L (GDPR Art.9 : la categorie speciale ne quitte pas l'hote ; seul l'EFFET traverse)** |

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
| I | **Verification cross-hardware semantique manquante** (l'exact-match ne couvre pas les GPU heterogenes) | L | **etage 2 primitive CABLEE (S77 Phase G)** : primitive N0 TOPLOC (`toploc.rs`, commitment BLAKE3 du sketch entier top-k du dernier hidden state) + helper worker qui le calcule + Layer-3 `verification.rs` consommant le commitment dans `logprobs_hash` par egalite (detection ~100% du swap modele/precision par INEGALITE). L'**ecriture/emission signee** du commitment dans `RunProof.activation_fingerprint` est **LIVREE S81 Phase J** (le decode loop bind le commitment N0 du DERNIER step au `RunProof` driver signe, premiere emission production) ; le miroir `ResultPayload.logprobs_hash` sur le chemin task classique reste porte par le worker (S76). La comparaison TOLERANTE cross-GPU (`ToplocFingerprint::compare`) est recomputee independamment par N1 (Phase H) / N2 (Phase I) — voir §16 « N0 TOPLOC fingerprint ». **Phase H** cable la PRIMITIVE N1 (tirage verifiable Ed25519 `verifiable_draw.rs` + recompute tolerant `ToplocFingerprint::compare` + Token-DiFR + incentive reputationnel `spotcheck_creditable`/`kudos_ledger::credit` + mapping criticite→niveau `criticality_maps_to_verification_level`) ; le re-exec prefill REEL sur GPU + le transport du sketch complet hors du slot 32B restent gates (**re-routes slot rig-chaud** S82/D6, ex-S78 — l'orchestrateur in-vivo etant livre S81 I/J, seul le canal de retour control-plane manque), comme G a livre la primitive N0 sans cablage in-vivo. **Phase I** cable les PRIMITIVES N2 (quorum tolerant M-of-N `redundancy.rs::tolerant_quorum_accepts` + chemin ADDITIF `validator.rs::validate_tolerant_quorum_shard`, verdict sur `RunProof` SIGNES, quorum exact `validate_quorum_pre_guardrail` INCHANGE) + N3 (commit-reveal `activation_commit.rs` `DOMAIN_ACTIVATION_COMMIT_V1` + localisateur EMA forward-only `sentinel.rs`) — voir §16 « N2 » / « N3 » ; le re-exec REEL cross-GPU in-vivo + le transport du sketch sur le data-plane + la bissection-sur-litige restent gates (**re-routes slot rig-chaud** S82/D6, ex-S78) | **M (primitives N0/N1/N2/N3 CABLEES S77 G/H/I ; orchestrateur + RunProof driver in-vivo LIVRES S81 I/J ; re-exec in-vivo + transport sketch + arbitrage litige = slot rig-chaud S82/D6 → L une fois in-vivo)** |

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
  1er ancre lexicographique advertising, pas une provenance auteur. (Le residuel
  Sybil dial-tail S76 — l'ORDRE des pull-providers dans `directory_pull_providers` —
  est ferme S83 A1 : `seeders_recent` echantillonne desormais par
  `blake3(project ‖ archive ‖ seeder_id)` context-scope ; ce chemin de re-drive-ancre
  reste distinct, il selectionne un hash d'annonce, pas un ordre de dial.)

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
| **SI-5 Latence side-channel** | le temps de compute d'un layer revele la complexite du prompt (longueur, heads actifs) | Low | residuel ; padding constant-rate = raffinement post-benchmark (re-route **slot rig-chaud** S82/D6, ex-S78 — le benchmark live S81 J existe desormais comme baseline) |

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
neuf) — re-routes **slot rig-chaud** (S82/D6, ex-S78). Cote
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
hors du slot 32B que ce recompute in-vivo exige est re-route **slot rig-chaud** (S82/D6, ex-S78).
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
(re-routes **slot rig-chaud** S82/D6, ex-S78) — exactement le pattern Phase G (primitive sans
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
  le rig reel = **slot rig-chaud** (S82/D6, ex-S78 ; le run live S81 J fournit
  desormais la baseline 5080-CUDA + M2-Metal).

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
  reste re-route **slot rig-chaud** (S82/D6, ex-S78) — l'arbitrage de litige in-vivo n'est pas cable.
- **SI-10 replay cross-session du commit** : neutralise par `session_id` dans la
  signature (cf. SI-8).
- **SI-11 evasion lente / empoisonnement de baseline EMA** (Sev **M**) : un stage
  derive juste sous le seuil a chaque pas pour ne jamais flagger, ou empoisonne la
  baseline. **Mitigation partielle** : un outlier flagge ne met PAS a jour l'EMA
  (rejet d'outlier, anti-spike) ; mais le seuil est **statique** (pas la fence IQR
  adaptative du papier) et un drift lent sous-seuil reste possible. Carry honnete
  (re-calibration + signal de magnitude absolue = **slot rig-chaud** S82/D6,
  ex-S78). Garantie crypto = N4 zkML, hors-scope.

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
> **slot rig-chaud** (S82/D6) : SI-5 padding (le benchmark J en est la
> baseline), RunProofs per-worker + transport du sketch, arbitrage de
> litige in-vivo (N3-reveal).

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
**Caveat TOCTOU (SI-12, Sev L, hote-local, re-route slot rig-chaud S82/D6)** : le digest attest
est le blake3 STREAMING du FICHIER au chemin GGUF au moment du hash, calcule
APRES le load (le backend llama.cpp mmap le fichier par defaut). Un remplacement
atomique du fichier entre le load et le hash permettrait a un operateur local
de servir l'inode mmape ancien tout en attestant le digest du nouveau chemin.
C'est une surface **hote-local/trusted** (l'operateur controle deja sa machine),
donc L ; le binding n'est PAS atomique load<->hash. Durcissement (hash de la
region mmapee, ou hash-avant-load + re-verification) = re-route slot rig-chaud (S82/D6).

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

## 17. Surface profil d'accessibilite (Sprint 83 Phase B)

Le chantier Workflow Agents introduit une **projection d'accessibilite
hote→app** : l'hote (shell React / page-hote du Hub) connait le profil
d'accessibilite de l'operateur ; une app SBFB sandboxee (le Workflow Hub, dans
un iframe `allow-scripts` sans `allow-same-origin`, CSP `connect-src 'none'`)
doit rendre accessible SANS jamais apprendre POURQUOI.

**Asset A9 — `needs[]` (cause) = PII sante/handicap.** Les 13 `AccessibilityNeed`
de l'operateur (basse vision, cognitif, vestibulaire, …) sont une donnee
LINDDUN sensible (categorie plus forte que le « consent PII light » A4). Le
profil projete (`ResolvedAccessibilityProfile.v1`,
`crates/sbfb-workflow-core/src/accessibility_profile.rs` + miroir
`web/src/a11y/profile.ts`) ne porte que les 15 **axes-effet** resolus
(`motion=reduced`, `contrast=high`, …). Invariant cardinal : **`needs[]` ne
quitte JAMAIS l'hote**.

| # | Menace (STRIDE/LINDDUN) | Sev | Mitigation cablee |
|---|---|---|---|
| **T-A11Y-NEEDS-LEAK** | *Disclosure/Linkability* — la cause fuit vers le DOM (`data-needs`), les logs, un prompt LLM, la trace, le bridge ou un provider | **fermee** | Projection **needs-free par construction** : le type de sortie `ResolvedAccessibilityProfile` n'a AUCUN champ `needs` (Rust : champ retire, pas un simple `#[serde(skip)]` — donc pas de fuite via `Debug`/`{:?}`/logger/acces manuel ; TS interface sans `needs`). `applyResolvedAxes()` ecrit les 15 `data-*` mais JAMAIS `data-needs` (preboot Hub inclus). Tests : `accessibility_profile::tests::needs_never_serialized_outward` (Rust) + oracle de parite partage `resolution_fixtures` (Rust) / `profile_resolution.test.ts` (web, applique `applyResolvedAxes` sur TOUS les besoins et assert 0 `data-needs` — **preuve autoritaire** de l'invariant) ; gate statique `web/scripts/check-a11y-needs-free.sh` (garde DOM **best-effort ligne-a-ligne**, non-fail-open, CABLE verify.sh + ci.yml — le garant AUTORITAIRE reste le type sans champ `needs`, PAS ce grep) ; spec e2e `workflow-hub-a11y.spec.ts` (smoke shell, non-autoritaire). Aucun axe a11y n'entre dans un `assemble_prompt` ni dans `TraceEvent.attributes`. |
| **T-A11Y-PROJECTION-SMUGGLE** | *Tampering* — un payload d'event bridge injecte `needs` dans l'app, y compris via le canal generique non type `pushEvent(payload: unknown)` sous **n'importe quel nom d'event** (contournait le sender strict) | **fermee** (voie sanctionnee) ; canal generique = best-effort, getter-TOCTOU **ferme** (S83 Phase C), residu value/nom-encoding **accepte** | Garantie **par construction** = le TYPE de la projection (aucun champ `needs`) emis UNIQUEMENT via `useBridge.pushAccessibilityProfile`. Sur le canal generique non type, deux gardes host (`web/src/bridge/reserved-events.ts`) plus le receveur : (1) **garde best-effort independante du nom** — `useBridge.pushEvent` appelle `payloadCarriesCause` et **DROP** avant `postMessage` tout payload portant une cle `needs` a toute profondeur du graphe enumerable (indices + props propres de tableau, cles ET valeurs de Map, Set, cycles), sous n'importe quel nom (anti-ACCIDENT `pushEvent("legacy-a11y", {needs})`). **Residu scope** : l'encodage delibere des VALEURS de need sous une cle arbitraire, en tableau nu, ou dans le NOM d'event n'est PAS couvert (value-scan des 13 tokens = faux positifs sur des notifications legitimes ; l'hote appelant est de confiance) et EST re-documente comme residu **ACCEPTE**. Le leg **getter-TOCTOU** (un getter qui mute entre le scan et l'envoi) est **FERME** en Sprint 83 Phase C : `pushEvent` clone le payload UNE seule fois (`structuredClone`, chaque getter execute exactement une fois) puis scanne ET envoie ce MEME snapshot fige (le scan inspecte exactement les octets de la wire) ; (2) l'event reserve `a11y-profile` est **refuse d'emblee** sur le canal generique (`isReservedEvent`, `Set` prototype-safe — `toString`/`constructor` non confondus) et n'est emis QUE par le sender dedie type `useBridge.pushAccessibilityProfile`, qui resout `needs[]` LOCALEMENT et re-valide la projection 15-axes ; (3) module partage `ResolvedAccessibilityProfileSchema` **Zod `.strict()`** (`parseProjectedProfile` → `null` sur cle en trop) + `buildAccessibilityProfileEvent` re-valide avant emission ; (4) cote Hub, `a11y.js` REJETTE tout payload portant une cle hors des 15 axes et n'accepte que les messages du `parent`. Tests : `useBridge.test.ts` (§"reserved event egress guard" — cause top-level/imbriquee sous nom libre = AUCUN envoi ; nom reserve refuse ; noms `Object.prototype` sans throw ; getter-TOCTOU clone-once ; `pushAccessibilityProfile` emet needs-free). |
| **T-A11Y-OPERATOR-DOM** | *Confusion de surface* — l'outil-hote Factory Operator ecrit `data-needs` a son propre DOM (`preboot.js:367`, `accessibility.ts:486`) | INFO | Acceptable : l'Operator est une **surface hote first-party privilegiee** (son acceptation NE repose PAS sur une absence de connectivite : son front fait des `fetch` loopback et expose un provider Claude cloud). Le `data-needs` reste sur son PROPRE DOM first-party, JAMAIS projete vers une app untrusted sandboxee ; le risque DOM est accepte sur cette base (first-party, DOM local de l'operateur), pas sur une premisse d'absence de connectivite. La difference avec le Hub est intentionnelle — le Hub (app untrusted) ne reutilise PAS `applyAccessibilityPreferences` verbatim ; le gate `check-a11y-needs-free.sh` scope les surfaces needs-free (Hub + `web/src/a11y`), l'Operator est hors scope. |
| **T-A11Y-LINKABILITY** | *Linkability* — les axes-effet correles (`motion=reduced` + `contrast=high` + `colorVision=safe`) laissent inferer un profil sante cote app | **L (accepte)** | Residuel accepte : la projection envoie l'EFFET, jamais la CAUSE ; mitige par `connect-src 'none'` (l'app sandboxee ne peut exfiltrer) + l'absence de tout identifiant de need. Non ferme : l'inference statistique reste possible pour une app qui rendrait ET renverrait par un canal legitime — hors surface Phase B (fondation). |

**Wire / census** : `ResolvedAccessibilityProfile.v1` est une projection
hote→app LOCALE, non-signee, non-gossipee → 0 `DOMAIN_*_V1`, 0
`*_FORMAT_VERSION`, census DOMAIN inchange par cette surface (25 a l'epoque
S83-B ; 29 depuis S83-F, `DOMAIN_WORKFLOW_PACK_V1` + 3 tags reserves). Frontiere docs-contrat
gatee par son propre golden `accessibility_profile.schema.json` (drift-gate).

## 18. Surface compilateur de plan (Sprint 83 Phase H)

Le chantier Workflow Agents rend l'invariant D4 **« le graphe signe EST le
plan »** (Plan-Then-Execute / CaMeL) *enforceable* pour la premiere fois : le
compilateur Rust hote (`crates/sbfb-workflow-core/src/compile.rs`) est le
**point d'enforcement STATIQUE** de cet invariant. L'iframe Hub peut linter,
jamais decider (research §12.1) ; le resultat autoritaire est
`CompiledPlan.v1`, artefact **LOCAL derive, jamais signe ni gossipe** (0
`DOMAIN_*_V1` produit — `DOMAIN_COMPILED_PLAN_V1` reste un tag RESERVE sans
producteur ; census 29 inchange). Le digest d'AUTORITE reste `content`
(recalcule de `PACK.lock` sous la signature Ed25519, verifiee AVANT le
compilateur) ; `compiled_plan_digest` et `graph_semantic_digest` sont des
digests DERIVES et LIES, jamais une surface d'integrite autonome.

**Asset A10 — le plan compile immuable.** Ce que la signature du pack promet
("aucun LLM n'etend le graphe, n'accorde une capacite, ne change une
destination"), le plan compile le rend verifiable : topologie normalisee,
fermeture des capacites (`capability_closure`), occurrences nommees
(`node_instance_id` = chemin structurel pur, RFC 6901), budgets compares
dimension par dimension, gates HITL epingles (`human_gates`).

| # | Menace (STRIDE) | Sev | Mitigation cablee |
|---|---|---|---|
| **T-PLAN-LLM-MUTATION** | *Tampering* — une sortie de modele (prompt injection incluse) etend la topologie, ajoute un outil, change une destination, s'accorde une capacite, reclasse une donnee, altere un schema de sortie, contourne un gate HITL, gonfle un budget ou usurpe l'identite du publisher (les 9 classes research §19.3) | **fermee (moitie statique)** | `compile()` est une fonction PURE de son entree RESOLUE — le pack signe plus les documents pinnes, chacun LIE PAR HASH : le compilateur derive lui-meme `blake3(bytes)` de chaque document fourni et refuse des bytes que le lock signe ne pinne pas (Codex r2 — un pin fourni par l'appelant ne prouvait rien). A version de compilateur fixe (`compiler.version` entre dans le digest DERIVE du plan, jamais dans l'autorite `content`) ; son type d'entree n'a AUCUN canal sortie-de-modele. Chacune des 9 familles est REPRESENTEE dans le plan et couverte par `compiled_plan_digest` — toute mutation change le digest (test `plan_then_execute_invariant`, bras rouge discriminant : meme pack → memes bytes ; pack change → digest change ; 9 mutations ciblees → 9 digests differents). La moitie RUNTIME (revalidation par tool-call, binding per-effet) = Phases J/K ; l'intersection 7 couches = **LIVREE §19.3 (Phase L)** comme COMPUTATION autoritaire (aucune couche n'elargit — meet monotone, la clause « ne s'accorde une capacite » est mecaniquement fail-closed) ; sa RELIURE per-effet au dispatch reste la phase qui shippe la surface run (DS-1). |
| **T-PLAN-STATIC-BYPASS** | *Elevation of privilege* — un pack signe mais malveillant/malforme passe en execution (cycle artisanal `A→B,B→A`, capability hors manifeste, handoff non autorise, edge dupliquee gonflant un quorum, budget enfant > parent, secret literal, endpoint inline) | **fermee** | **20 classes de refus statiques** enumerees (13 research §10.5 + 6 regles freeze §8 + compatibilite de ports nominale-par-digest freeze §7), chacune REFUSEE et testee discriminant (`static_validation_rejects_*`, controle vert + mutation rouge) — deux precisions honnetes (Codex r2) : la classe `map` sans borne est portee par le PARSE strict (`InvalidDocument`, la borne est un champ REQUIS de la forme gelee), et la classe « cycle hors boucle bornee » est SUBSUMEE par le rejet de tout cycle (`CycleDetected`, strictement plus fort — preflight §9 NB). Le check handoff remonte les donneurs TRANSITIVEMENT a travers les nodes non-agents. Le DAG est prouve **par COMPILATION** (Kahn), jamais « by construction ». |
| **T-PLAN-CLASSIFICATION-LAUNDER** | *Tampering/Information disclosure* — un `switch` discrimine sur un champ de CLASSIFICATION (`visibility`, `confidentiality`, …) produit par un modele, ou un `classifier` PROPOSE des labels de classification, avec un effet (`capability.call`) en aval — la publication devient pilotee par le LLM (regle 6 freeze §8, Codex G r1 GAP1) | **fermee (nominal v1)** | Analyse **data-flow** sur les ports types EXISTANTS (jamais un champ de schema neuf — le gel FORME de G est intact) : sources modeles TAINTENT, `human.review` blanchit, `capability.call` et `knowledge.query` NE blanchissent PAS (un resultat peut relayer/selectionner selon des arguments nourris au modele — recursion, Codex r2/r3), sorties `map`/`repeat`/`subworkflow` OPAQUES = taint fail-closed, et un `map`/`repeat`/`subworkflow` AVAL compte comme effet (le switch parent routant vers un enfant effectful est couvert, Codex r2). Conservatisme v1 documente ; portee elargie a TOUT `capability.call` aval, plus stricte que la lettre §8.6. **Borne nominale honnete** : la reconnaissance = appartenance exact-lowercase aux catalogues fixes `CLASSIFICATION_DISCRIMINANTS` / `CLASSIFICATION_LABEL_TERMS` — un synonyme hors liste (`sensitivity`, `clearance`, label `world`…) echappe a ce refus, meme classe residuelle best-effort que T-PLAN-SCAN-EVASION ; les classes de capacites TYPEES existent desormais engine-side (`sbfb-workflow-engine::capability`, 17 classes §14.2, computation **LIVREE §19.3 Phase L**) — mais la reconnaissance STATIQUE du compilateur ici reste NOMINALE : le raffinement runtime engine-side (autorisation par CLASSE typee, pas par nom) fermera le gap en DEFENSE-EN-PROFONDEUR **une fois la computation reliee au dispatch** (DS-1, phase run-surface — aujourd'hui L ne juge rien per-effet, zero call-site) ; il ne durcit de toute facon PAS cette borne statique du compilateur, qui reste documentee telle quelle. Un gate de QUALITE modele reste hors-scope (pattern babel, ratifie). Publication-classe DOMINEE par un gate non-modele (`human.review` OU `wait.signal` hote — patterns ideas/babel) ; la classe publication est elle-meme reconnue par marqueurs de nom (`publish`/`egress`/`announce`/`broadcast` en segment) — **sous-inclusive au niveau STATIQUE** : une capacite publiante nommee autrement (`content.share`, `feed.upload`) echappe au marqueur du compilateur ; le runtime engine-side, une fois relie, la jugera par TYPE de classe, pas par nom. |
| **T-PLAN-DIGEST-MALLEABILITY** | *Spoofing* — deux plans differents partagent un digest (cle de presentation en position data, ordre d'edges, id de schema re-pinne sous d'autres bytes) | **fermee** | Refus `PresentationKeyAsData` (node ids + cles `field_map` ∌ `PRESENTATION_KEYS`, ferme la malleabilite `graph_semantic` §6 freeze) ; ordre des edges NORMALISE avant digest (`graph_semantic_digest_order_independent`) ; UN id = UN digest a l'echelle du plan pour TOUTES les categories d'artefacts (schemas via `PortTypeMismatch`, workflows/agents/prompts/tools/corpora/guardrails via `record_digest` fail-closed, cache agents re-confronte a chaque reference — Codex r2 : plus aucun ecrasement silencieux d'homonymes) ; determinisme byte-identique sous les 2 profils de build (`compile_is_deterministic`, piege `preserve_order`). |
| **T-PLAN-SCAN-EVASION** | *Tampering* — un secret/endpoint fractionne ou encode echappe au scan des documents graphe/agent | **L (accepte)** | Le scan (secret-shape + `://` + marqueurs executables) est **best-effort fail-closed sur ce qu'il reconnait** — la detection endpoint = sous-chaine `://` UNIQUEMENT (les schemas URI sans slash `data:`/`javascript:` ne sont pas reconnus, meme residuel) ; la garantie enforced reste le FORMAT (aucun champ endpoint/code, `secret_ref` opaques, capacites fermees). Les bytes des prompts/corpus (fichiers pinnes par digest) sont scannes a l'assemblage du pack (phase Hub/T), pas ici. |

**Bornes honnetes (ce que H ne ferme PAS)** : run store durable = **LIVRE
§19 (Phase I)** ; reducer/scheduler + cancel honnete = **LIVRES §19.1
(Phase J)** — le scheduler ne dispatche QUE des occurrences du plan compile
(la part J de la moitie RUNTIME de T-PLAN-LLM-MUTATION ; la revalidation
per-effet/tool-call reste K) ;
`effect_id`/idempotence = K ; intersection 7 couches + attenuation monotone +
17 classes typees = **LIVRE §19.3 (Phase L)** comme computation autoritaire
(la reliure per-effet au dispatch = phase run-surface, DS-1) ; coffre
`secret_ref` + HITL signe (principal/audience) = L1 ; SSRF/DNS = P ; miroir TS/canvas = T ;
signature « attestation compilateur » (§17.1 research) = role separe differe.
La facette signee `pack.digests.compiled_plan` reste une claim de FORME
(Phase F) dont ce module rend la verification POSSIBLE
(`compiled_plan_digest`) — la reconciliation claim↔artefact est une passe de
verification materialisante ulterieure. **La confrontation adversariale
complete de cette surface est Phase AH** (H seme l'asset, AH confronte —
decision preflight B5).

## 19. Surface run store durable (Sprint 83 Phase I)

Le run store (`crates/sbfb-workflow-engine/src/store.rs`) est l'**etat
d'execution durable** du moteur : journal `run_events` append-only + outbox
transactionnel (Phase E, GO) + inbox dedupliquante + snapshots jetables +
lease single-writer + retention/GC + backup/restore (Phase I). Il est
**LOCAL au noeud** : 0 octet cross-noeud, 0 signature d'entree
(`DOMAIN_RUN_EVENT_V1` reste un tag RESERVE 0-producteur — il nomme une
future forme EXPORTEE, pas ce journal ; census 29 inchange). L'enveloppe
`RunEvent.v1` est remplie cote core (`sbfb-workflow-core/src/run_event.rs`,
registre 6/7).

**Asset A11 — l'etat d'execution durable.** Trust boundary : la frontiere OS
mono-utilisateur (le fichier vit ou l'hote l'injecte — la crate ne resout ni
home ni env). L'INPUT non-fiable n'est pas le store, c'est le SPEC/pack —
verifie Ed25519 EN AMONT par le compilateur (§18) ; le store n'enregistre
que ce que le moteur single-writer a decide. Les horloges (`now_ms`) sont
INJECTEES par l'appelant : le journal lui-meme ne porte aucune valeur
d'environnement (byte-reproducible, goldens committes).

| # | Menace (STRIDE) | Sev | Mitigation cablee |
|---|---|---|---|
| **T-RUN-DOWNGRADE-MISREAD** | *Tampering* — un binaire ANCIEN ouvre un store ecrit par un moteur plus recent et mal-interprete des formes qu'il ne connait pas (la moitie « downgrade dangereux » du plan) | **fermee** | Garde DOUBLE, chacune mutation-testee : (1) niveau SCHEMA — `RunStore::open` refuse `user_version > migrations connues` (`UnsupportedSchemaVersion`) AVANT `to_latest`, ce qui couvre aussi le restore d'un backup trop recent ; (2) niveau EVENT — `load_events` passe par l'unique porte `upcast_payload` qui refuse `format_version > courant` (`NewerThanSupported`) et refuse une chaine trouee (`ChainGap` LOUD, jamais un skip silencieux). L'upcast est a l'INTERPRETATION (`Value`→`Value`, jamais de reecriture des rows), chaine testee PAR-HOP. |
| **T-RUN-DOUBLE-EMIT-STALE-WRITER** | *Elevation of privilege* — deux runtimes ouvrent le meme store et font AVANCER la meme outbox ; variante : un owner PERIME (takeover subi) commet encore | **fermee (couche STORE, mode leased)** | Fencing par **epoch monotone** (table `lease` a 1 row) : CAS `UPDATE … WHERE owner = ? AND epoch = ?` + `rows_affected == 1` DANS chaque write-transaction, sinon rollback TOTAL (`LeaseLost` — un writer fenced-out n'emet NI entree NI outbox, teste + mutation M1). L'expiry mur-horloge n'est JAMAIS une condition de commit — seulement l'heuristique de takeover d'un aspirant (Kleppmann fencing tokens ; Temporal `rangeID`). Chaque ACQUISITION bumpe l'epoch (y compris meme owner : invalide les handles zombies d'une incarnation morte). Bras out-of-process : owner reellement MORT → takeover epoch+1 → le run finit sous le nouveau fence (record_run_completed asserte). **Bornes honnetes** : (1) un handle SANS lease ecrit unfenced (posture spike E mono-process, documentee sur `RunStore`) — le chemin ECRIVAIN runtime **ACQUIERT depuis J (v25)** : `Orchestrator::execute` prend le lease avant tout write (chaque acquisition bumpe l'epoch, invalidant les zombies — bras out-of-process `orchestrator_iframe_death_does_not_kill_run`), les routes I1 restent LECTURE SEULE sans lease (v24) ; le fencing est une posture ACQUISE, pas un defaut ambiant ; (2) le CAS fence les ECRITURES DU STORE (journal/outbox), PAS la fenetre d'effet EXTERNE — **fermee TARGET-SIDE en K (v26) pour le chemin activity** : `EffectContext.fencing_epoch` porte l'epoch du writer et une cible fencing-aware REFUSE une application sous le plus haut epoch durablement vu dans SON backend (`FencedFileActivity` reference ; tests `stale_writer_rejected_at_target` + red control `fencing_red_control_double_output`). Bornes honnetes restantes : (a) etre fencing-aware est une propriete DU BACKEND de la cible — `FencedFileActivity` DEMONTRE le MECANISME (l'epoch est presente et une application d'epoch inferieur est refusee) mais reste un DOUBLE de reference : sa sequence lire-le-log → decider → append n'est PAS atomique, donc deux writers CONCURRENTS lisant le meme etat pourraient tous deux appender ; ce que le modele SINGLE-WRITER SERIALISE lease-fence rend NON-NOMINAL (un seul orchestrateur ecrit a la fois, un writer dechu a un epoch INFERIEUR deja durablement ecrit par le nouveau — sequentiel, pas concurrent) ; une cible de PRODUCTION doit fournir un check-and-apply atomique dans son propre backend (borne `activity.rs`) ; (b) `WorktreeFsWriteActivity` et `ProcessActivity` ne consultent PAS `ctx.fencing_epoch` (un FS write / un spawn n'ont pas de backend epoch-deduplicant) : leur fenetre reste bornee par l'idempotence-par-contenu / la non-collapsabilite comme le spike, pas par le fencing token ; (c) le chemin bloquant du spike (`EffectTarget`) garde la posture clause-2 (dedup seule) — c'est la preuve gelee E, pas l'adaptateur de production (Codex r1 L3, MECANISME de fencing demontre v26, atomicite check-and-apply = propriete requise du backend reel). |
| **T-RUN-RESTORE-ROLLBACK** | *Tampering/Repudiation* — un restore rembobine le journal a l'instant du backup : evenements posterieurs perdus, effets deja appliques re-tentables | **M (accepte, opere)** | Le restore est par NATURE une surface anti-rollback : assume et documente, jamais masque. Gardes cablees : validation du backup AVANT tout remplacement (ouverture read-only ; `user_version` connu ET >= 1 ; INVENTAIRE COMPLET tables + COLONNES exigees par la version reclamee [v1 = tables E, v2 = les 6, chaque table verifiee colonne par colonne via le pragma type table_info] + index `outbox_one_effect_per_node` verifie SEMANTIQUEMENT [UNIQUE + colonnes exactes (run_id, node_instance_id), pragmas index_list/index_info — un homonyme non-unique ou mal cible est refuse, Codex r3 P1] + flags PK des colonnes porteuses de semantique + presence des FK declarees des 4 tables enfants + `PRAGMA quick_check` == ok — un SQLite valide aux bons NOMS mais au schema etranger est refuse [Codex r1/r2/r3 P1] ; index NON partiel exige, PK comparees en ENSEMBLE EXACT, FK exigee MONO-COLONNE exacte run_id REFERENCES runs(run_id) [groupe de contrainte a une seule ligne — une FK composite a composante enfant NULLable ne contraindrait plus rien, Codex r5] ; bornes restantes honnetes ENUMEREES : types, NOT NULL, clauses CHECK, defauts, collations, violations FK preexistantes et colonnes surnumeraires non compares — la frontiere reste OS mono-utilisateur), store CIBLE ferme (sur Windows la copie sur un store ouvert echoue en sharing violation = la garde qui marche), siblings `-wal`/`-shm` perimes supprimes (un WAL de l'ANCIEN fichier rejoue sur le nouveau = corruption). Contre le double-effet post-restore, la garde reste cote CIBLE (`dedup_scope` journalise + `TargetScopeChanged` fail-closed, clause 2 du spike). Le geste reste OPERATEUR-LOCAL (aucune route reseau) ; le tier D7 a ete DELIBERE en I1 (v24) : rows T1 pre-classees dans l'inventaire LOOPBACK §3, **re-routees Phase V en K (v26)** vers la surface operateur « Centre des Runs » (K est engine-only, 0 route) — restore/GC ne sont PAS exposes par les routes I1 (lecture seule). |
| **T-RUN-GC-EVIDENCE-LOSS** | *Repudiation* — la purge de retention efface la preuve d'audit d'un incident | **bornee par politique** | GC = perte d'evidence ACCEPTEE et BORNEE, dite par la signature (`RetentionPolicy` sans defaut implicite : l'appelant DOIT dire ce qu'il accepte). Jamais purges, mutation-testes : un run EN VOL (jamais candidat), une outbox PENDING (completed porteur = skip compte ; terminal les conserve), le DIAGNOSTIC terminal (la row `runs` + l'entree `run_terminal` survivent a chaque passe — le post-mortem `outcome_unknown` reste possible pour toujours). `closed_at_ms` NULL = jamais purge (fail-safe : pas de date, pas de purge). Purge PER-RUN atomique (`FaultPoint::InsideGcTransaction` : crash au milieu → run entierement intact) ; statut + eligibilite + fenetre `keep_last` RE-CLASSEE + pending RELUS DANS la transaction de purge (le scan candidats n'est qu'un pre-filtre ; un run re-date devenu le plus recent est epargne — Codex r2 P2) — un diagnostic terminal pose ENTRE le scan et la purge est re-dispatche sur la row courante et survit (Codex r1 P1, teste par interleaving `BeforeGcPurgeOfRun` via connexion separee) ; et les etats clos sont ABSORBANTS (`InvalidTransition` : un terminal ne peut plus etre re-etiquete completed, la voie de purge du diagnostic par re-labelling est fermee — Codex r2 P1, mutation M9). |
| **T-RUN-SNAPSHOT-POISON** | *Tampering* — un snapshot empoisonne devient l'etat du run | **fermee** | Un snapshot n'est JAMAIS une source de verite : cache invalide par version de FORMAT (`Stale`) et par digest BLAKE3 sur le **RUN PROPRIETAIRE + TUPLE SERVI ENTIER** (`run_id`, version, `upto_sequence`, `state` — pas le state seul, sinon une borne mensongere sous state intact passerait [review D4-1], et pas sans run_id, sinon le tuple intact d'un AUTRE run se substituerait [Codex r1 L5]) (`Poisoned` — DETECTE, jamais servi ; porte degradee `read_snapshot_unverified` derriere `spike-faults` = le controle rouge qui prouve la garde load-bearing, mutation M3). Supprimer le snapshot ne change RIEN d'observable (teste) : le journal re-derive le meme etat. Etat donnees-pures JSON, `null` refuse par la porte de digest (meme regle que tous les digests du crate). |
| **T-RUN-SQLI-DESERIAL** | *Tampering/EoP* — la chaine CVE checkpointer LangGraph 2026 (CVE-2025-67644 SQLi + CVE-2026-28277 msgpack→RCE) transposee au run store | **fermee** | Doctrine D3.6 appliquee : requetes parametrees PARTOUT (`params!`, 0 concatenation SQL — verifie mecaniquement ; les seuls `format!` du module produisent des messages d'erreur ou des noms de fichiers freres `-wal`/`-shm`/`.tmp`, aucun n'atteint le SQL) ; etat serialise DONNEES-PURES JSON (jamais pickle/msgpack, enveloppe typee `deny_unknown_fields`) ; upcasters = transforms `Value`→`Value` COMPILES dans ce binaire — le store ne porte jamais de code. |
| **T-RUN-ID-PII** | *Information disclosure* (LINDDUN) — `run_id`/`node_instance_id` sont des `String` caller-chosen persistees : un identifiant portant un token/email survit au process dans le journal | **M (residuel documente)** | Le journal est payload-free PAR TYPE (`RunEventKind` sans champ de contenu, receipts en DIGEST seul) — la garantie porte sur les CHAMPS, pas sur ce que l'appelant nomme. Validation longueur + caracteres de controle **ET** caracteres invisibles / bidi format (`validate_identity_component` → `is_forbidden_in_identity`, domaine elargi en S83 L1 — cf. borne 24 du §19.4 et le jumeau `echo.rs:26-28`), sur `run_id`, `node_instance_id`, `message_id` d'inbox, `lease_owner`, `nonce` (`store.rs:1899`) et `timer_id` (`:2225`). Ce que la validation ne fait PAS : elle ne borne pas le charset a l'ASCII, donc un spoof d'affichage purement ASCII (homoglyphes, userinfo) passe — borne 24. Cote ROUTE (I1, v24) : `run_id` serveur-valide au choke point `workflow_api.rs` — charset borne `[A-Za-z0-9._-]{1,128}` → 400 sinon (plus strict que la garde engine ; un `run_id` ne forme JAMAIS un path, il ne voyage qu'en binding `params!`). Le residuel M reste : l'identifiant demeure caller-chosen au niveau engine. |

**Bornes honnetes (ce que I ne ferme PAS)** : exposition loopback des
operations du store = **Phase I1 LIVREE en LECTURE SEULE** (v24 : 2 GET
`workflow_api.rs` — agregat par-run + row lease brute — inventaire D7 §3
LOOPBACK + famille golden_http ; la famille MUTANTE restore/GC/lease-force
etait tier-classee T1 avec implementation datee J/K — **re-routee V en K
(v26)** : K est engine-only [0 route, `T1 = N-A-no-frontend-change`], la
surface operateur qui porte cancel/retry/replay/reconcile est le Centre
des Runs, rows D7 mises a jour en consequence — pas de defer silencieux) ;
reducer/scheduler et autorite du run = **LIVRES §19.1 (Phase J, v25)** ;
`effect_id`/`attempt`/receipts/retries/timeouts/park-reconcile =
**LIVRES §19.2 (Phase K, v26)** ; le mode UNFENCED reste possible par
construction (posture
documentee, cf. T-RUN-DOUBLE-EMIT) ; la retention temporelle exige un
appelant qui fournit `closed_at_ms` (le SpikeEngine clockless passe `None` ;
l'orchestrateur J/K fournit le sien via le trait `Clock` injecte) ;
l'artefact T2 par classe de store (pattern S82 T) reste `NOT-RUN` jusqu'a
AI2.

### 19.1 Surface orchestrateur deterministe (Sprint 83 Phase J)

L'orchestrateur (`crates/sbfb-workflow-engine/src/{orchestrator,reducer,
clock,signal,runtime_id}.rs`) est le **scheduler local** du run : reduction
pure du journal (etats §12.3), ready-set canonique, timers durables a
deadline ABSOLUE, signaux mono-usage, cancel honnete. Il N'AJOUTE aucune
surface reseau (0 route, 0 octet cross-noeud, census 29 inchange) : sa
surface d'attaque est l'INTERFACE HOTE (depot de signaux, horloge injectee,
demande de cancel) plus la purete du replay. Autorite du run = noeud
initiateur (WA-3) : plan compile, journal, frontier, timers, interruptions
vivent sous UN writer lease-fence.

| # | Menace (STRIDE) | Sev | Mitigation cablee |
|---|---|---|---|
| **T-RUN-SIGNAL-FORGE-REPLAY** | *Spoofing* — un signal `wait.signal` (ou une decision `human.review`) forge, rejoue ou perime resume un run parque, ou re-declenche une reprise deja consommee | **fermee** | Mono-usage §12.6 cable en QUATRE gardes, chacune LOUD : (1) nonce ENGINE-DERIVE (`derive_wait_nonce`, length-prefixed, jamais caller-chosen) journalise a l'ouverture du wait (`INSERT OR IGNORE` : UN nonce spendable par interruption, un replay de la passe n'en re-mint pas) ; (2) depot REFUSE si aucun wait OPEN ne nomme le nonce (`SignalRefused` — un nonce forge est observable A LA PORTE, zero row ecrite) ; re-livraison du meme nonce = dup collapse (`INSERT OR IGNORE`, precedent inbox S76-D) ; (3) consommation ATOMIQUE : CAS du wait (open→consumed) + flag depot + entree `signal_consumed`/`human_decision_recorded` dans UNE transaction (mutation M2' : sans le CAS, la suite rougit) — le flag depot est defense-en-profondeur documentee ; (4) expiration du DEPOSANT verifiee a la consommation contre `now_ms` injecte, label `human.review` valide contre le SET FERME du plan AVANT toute ecriture. Tests adversariaux : `signal_replay_consumed_exactly_once`, `signal_forged_nonce_rejected`, `signal_expired_rejected`, bras out-of-set de `human_review_parking_and_closed_set`. **Bornes honnetes** : (1) le nonce est MONO-USAGE, pas une AUTHENTIFICATION — derive de valeurs deductibles (run_id, node, canal), il ne prouve pas QUI depose ; l'authentification du principal (signature HITL) a ete LIVREE en L1 — mais **A COTE de ce chemin, pas dedans** : `SignalDeposit` porte toujours 4 champs, sans principal ni signature, et c'est ce chemin que l'orchestrateur emprunte pour `human.review`. La reliure doit le **REMPLACER, jamais l'ETENDRE** (§19.4 borne 13) —, l'authz de la porte exposee en route = la phase qui SHIPPE la route (re-routee **V**, v26 — K est engine-only, 0 route, cf. inventaire D7) ; aujourd'hui `deposit_signal` reste intra-frontiere OS mono-utilisateur (aucune route) et le payload est borne a la porte (`MAX_SIGNAL_PAYLOAD_BYTES` 256 KiB, review r1 SD-3). (2) Un depot EXPIRE (ou refuse au label) occupe son nonce — le dedup INSERT OR IGNORE empeche de le remplacer, donc le run reste parque LOUD (chaque consommation re-echoue) sans recuperation in-protocol : probleme OPERATEUR surfacé bruyamment, jamais un remplacement silencieux ; la re-ouverture d'un wait (nouveau nonce, geste operateur) = re-routee **V** avec la surface operateur (v26 — le PRECEDENT mecanique existe desormais : la primitive park/reconcile de §19.2 est le patron a repliquer cote waits). Tests (noms exacts) : `orchestrator_signal_replay_consumed_exactly_once`, `orchestrator_signal_forged_nonce_rejected`, `orchestrator_signal_expired_rejected`, `orchestrator_human_review_parking_and_closed_set`, `orchestrator_human_review_routes_decided_port`. |
| **T-RUN-CANCEL-DOS** | *DoS/Tampering* — le cancel comme vecteur d'arret arbitraire, ou un resultat tardif re-etiquete un run annule en succes | **bornee (autorite hote)** | Le cancel est une ECRITURE SOUS LEASE (`record_cancel_requested` idempotent : un run clos ou deja annule ne re-journalise pas) d'origine FERMEE (`CancelOrigin` : `host_operator` | `workflow_timeout` — aucun variant modele, aucun texte libre ; un pack ne peut PAS opt-out de l'autorite operateur, la porte ne consulte aucune policy). §12.7 cable : des l'entree `cancel_requested` le scheduler n'emet PLUS — le stop d'emission est ABSOLU : la fermeture n'appelle JAMAIS `target.apply`, meme pour un intent pendant idempotent (sa premiere emission pourrait sinon suivre l'annulation — Codex r1 P1) ; le pendant reste PENDANT, honnetement journalise ; un resultat OBSERVE Ready au poll du drain est ROUTE vers sa porte `record_late_completion` **par le drain du cancel (K, v26)** : les attempts ouverts sont STOPPES (arbre de processus tue ; un orphelin d'incarnation precedente est REAPE par son handle journalise, ex-K-CANCEL-01), un resultat qui a fini juste avant le stop est CONSERVE (receipt journalise en digest, frontier JAMAIS avance, zero propagation, aval jamais dispatche — test `late_result_after_cancel_kept_without_propagation`). **Borne honnete** : « conserve » vaut pour un resultat OBSERVE Ready a ce poll — un resultat qui finit APRES le poll mais avant le stop n'a pas de second poll et est jete (course inherente, jamais une promesse de conservation universelle). Le run ferme `cancelled` ABSORBANT — jamais re-etiquete completed (`record_terminal` + `InvalidTransition` I) ; un cancel deja journalise bloque le firing timer (pas de double entree operateur+timer). Le wire Task n'a pas de cancel fort : la limite est AFFICHEE, jamais promise. Tests : `honest_cancel_keeps_late_result_without_propagation` (store-level, J) + `late_result_after_cancel_kept_without_propagation` (drain du cancel, K). |
| **T-RUN-TIMER-SKEW** | *Tampering* — une horloge hote manipulee/regressive re-arme un timer, de-expire un depot, ou re-declenche un cancel | **bornee (watermark + CAS)** | Deadlines ABSOLUES en table d'etat (`timers.deadline_ms`, JAMAIS une duree — le drift delay-as-remainder est structurellement impossible : `INSERT OR IGNORE`, la PREMIERE deadline gagne pour toujours ; garde composite prouvee par mutation M5) ; firing CAS `fired=0→1` gate sur run RUNNING : un timer tire AU PLUS une fois, jamais dans un journal clos ; decisions d'horloge JOURNALISEES (`cancel_requested{workflow_timeout}` — le replay lit le journal, jamais l'horloge live) ; REGRESSION LOUD a DEUX niveaux : **watermark durable par-store** (`clock_watermark.max_now_ms`, avance a CHAQUE passe AVANT tout jugement d'expiry — une passe entiere a lecture regressive est refusee `ClockRegression`, ce qui ferme la fenetre de de-expiration ENTRE passes [review r1 SD-2]) + garde par-depot (`deposited_at_ms` futur refuse). La porte cancel OPERATEUR reste HORS watermark : l'autorite operateur n'est pas blocable par une horloge cassee. Le timer run-wide est (re)propose a CHAQUE appel d'execute, APRES le jugement watermark de la lecture (une deadline derivee d'une lecture que le watermark refuserait ne peut pas devenir la premiere ecriture irreversible — Codex r2 ; un crash entre creation du run et premiere ecriture ne perd plus le timer — Codex r1 ; residu honnete : dans cette fenetre la deadline se derive de l'horloge du RESUME, plus tardive que l'originale ; et une premiere lecture aberrante HAUTE fixe watermark et deadline — la protection est contre la REGRESSION, pas contre une horloge initialement fausse). Tests : `orchestrator_restart_mid_timer_fires_once_at_original_deadline`, `orchestrator_clock_regression_loud`. |
| **T-RUN-REPLAY-DIVERGENCE** | *Repudiation/Tampering* — un scheduler non-deterministe derive un frontier different au recovery et re-emet un effet deja fait | **fermee (determinisme) / bornee (integrite causale)** | Le reducer est une fonction PURE de `(plan, journal)` : toute decision NON deductible de la topologie est JOURNALISEE AVANT d'etre agie (`branch_taken`, `map_fanout`, consommations, cancel — le reducer ne parse JAMAIS un payload materiel) ; ready-set en ordre CANONIQUE d'ids (BTreeMap partout, 0 HashMap sur le chemin de decision) sous cap borne — l'ordre d'admission journalise est IDENTIQUE quel que soit le cap (test `backpressure_deterministic_admission`) ; divergence per-node FAIL-CLOSED sous le graphe (hash memoise vs rejoue → `StepHashMismatch` terminal, mutation M4) ; identites d'iteration DISJOINTES par construction (`~2N` inatteignable dans un id structurel RFC 6901-echappe — deux iterations ne partagent jamais une cle d'effet, §12.2). Filet store : index unique `outbox_one_effect_per_node`. **Borne honnete (Codex r1)** : les refus structurels de la reduction couvrent `UnknownNode` et `FanoutBeyondBound` ; la COHERENCE CAUSALE du journal est desormais PARTIELLEMENT re-validee a la reduction (**K, v26** : `ReduceError::CausalOrder` refuse une completion sans intent, un attempt-failure sans intent, et une completion sur un node PARQUE sans reconciliation prealable — la forme du blanchiment ; tests `completion_without_intent_refused_by_reducer`, `completion_on_parked_without_reconciliation_refused`) ; residu honnete : une decision sur un node aux inputs non satisfaits reste garantie par l'ECRIVAIN unique lease-fence, pas re-derivee a la lecture. Tests : `orchestrator_deterministic_replay_*`, `orchestrator_iframe_death_does_not_kill_run` (mort reelle du writer, reprise du MEME frontier, 0 effet double), goldens 4/4 rejouables. |
| **T-RUN-SUBWORKFLOW-AMPLIFICATION** | *DoS* — un fan-out map ou une profondeur subworkflow amplifie le travail au-dela du plan signe | **fermee (statique+runtime)** | Bornes STRUCTURELLES du plan signe consommees, jamais des compteurs runtime : `max_cardinality` verifie AVANT le journal (`MapCardinalityExceeded`, zero entree ecrite — mutation M6) et RE-verifie a la reduction (`FanoutBeyondBound` fail-closed sur journal etranger) ; `max_iterations` borne la materialisation des iterations repeat COTE reducer (un verdict `continue` au-dela de la borne ne materialise rien) ; l'expansion subworkflow est STATIQUE (Phase H, `MAX_PLAN_DEPTH` compilateur) — J navigue les scopes, il n'expanse RIEN au runtime. Test : `map_cardinality_exceeded_refused`, `bounded_repeat`. |
| **T-RUN-BACKPRESSURE-STARVATION** | *DoS* — une file saturee affame les autres nodes ou change le resultat selon la charge | **bornee (mono-run v1)** | Cap in-flight BORNE et deterministe (`max_in_flight`, knob HOTE — pas un champ du plan signe, le vocabulaire n'existe pas dans le format) : l'admission reste canonique et le cap change le PACING jamais le resultat (test cap=1 vs cap=8, sequences d'intents identiques). Borne honnete : l'equite INTER-RUNS n'existe pas encore (un orchestrateur = un run par appel, synchrone) — elle arrive avec le service multi-runs (V/AI). La COHORTE d'un join est desormais FIGEE au fire (**K, v26** : entree `join_fired{sources}` journalisee des que la policy est satisfaite — le reducer lit la PRESENCE de cette entree (le join est Succeeded, gel du verdict), et la resolution de valeurs de l'orchestrateur lit la COHORTE journalisee `sources` depuis l'evenement ; un perdant qui delivre APRES ne peut plus elargir un quorum ni remplacer un gagnant `any` ; tests `join_fired_freezes_cohort` + `join_quorum_freezes_multi_source_cohort` ; le filet request-hash reste en defense-en-profondeur). Residu honnete : l'ABANDON ACTIF d'une branche perdante n'existe pas — les perdants vivants continuent et leurs resultats sont CONSERVES au journal (jamais consommes par l'aval) ; leur annulation ciblee = surface operateur (V). L'alignement contractuel du decision_label est LIVRE (**K, v26** : `MAX_DECISION_LABEL_BYTES` compilateur == `MAX_SIGNAL_PAYLOAD_BYTES` store, refus a la compilation + test miroir `decision_label_bound_mirrors_store_bound`). |

### 19.2 Surface effets (Sprint 83 Phase K)

La couche effets (`crates/sbfb-workflow-engine/src/{activity,process,
fs_effect}.rs` + les portes attempts du store) est le premier endroit ou
un run TOUCHE LE MONDE : side outputs durables hors store, arbres de
processus OS, mutations de worktree. Elle N'AJOUTE aucune surface reseau
(0 route, 0 octet cross-noeud, census DOMAIN 29 inchange, 0 nouveau
noeud lock — deps target-gated `windows`/`libc` sur des noeuds deja
resolus) ; sa surface d'attaque est l'ADAPTATEUR (programme/args/env du
process, chemin/contenu FS, backend de dedup de la cible) plus la
semantique retry/unknown. Doctrine duress : AUCUN gate dans l'engine
(pattern I1 — la lib n'a pas d'identite ; les effets locaux ne sont pas
des operations sortantes signantes).

| # | Menace (STRIDE) | Sev | Mitigation cablee |
|---|---|---|---|
| **T-RUN-RECEIPT-FORGE-REPLAY** | *Spoofing/Repudiation* — un receipt forge ou rejoue atteste une activite qui n'a pas eu lieu, ou re-atteste cross-run le resultat d'un autre run | **bornee (anti-rejeu par contenu + journal digest-only)** | Deux proprietes VERIFIEES : (1) **anti-rejeu cross-run/cross-node** — le receipt (`ActivityReceipt`, role §17.1(6)) lie le TUPLE COMPLET `(run_id, node_instance_id, effect_id, attempt, request_hash)` dans son CONTENU JCS. La porte de RECONCILIATION `reconcile_unknown(ResultRecovered)` EXIGE un receipt structure parsable ET re-verifie run_id/node/effect_id/request_hash contre le `pending` courant (`ReconcileReceiptMismatch` — un blob opaque, un receipt-de-run-X, ou un mauvais effect_id sont tous rejetes AVANT toute ecriture). Sur le chemin de completion NORMAL, l'anti-rejeu cross-run repose sur l'`effect_id` ENGINE-DERIVE (`derive_effect_id` = hash length-prefixed incluant `run_id`+node+request_hash — deux runs ne partagent jamais un effect_id) et sur le fait que le moteur ne complete JAMAIS qu'avec SON PROPRE `pending.effect_id` : le moteur ne route jamais un receipt etranger. **Borne honnete** : le `request_hash` SEUL n'est PAS run-unique (il couvre node/activite/input, pas `run_id`) — ce n'est donc pas lui qui distingue les runs, c'est l'`effect_id` derive et le check de contenu de la reconciliation ; (2) **journal digest-only** — materiel en `outbox.receipt`, seul le `receipt_digest` atteint le journal (context-string local `hash.rs`, pas une famille DOMAIN), donc un journal qui survit au process ne fuite NI prompt NI output. **Bornes honnetes EXPLICITES** : (a) le receipt n'est PAS Ed25519-signe (census DOMAIN 29 GELE, crate sans clef — l'attestation riche signee provider/modele/CLI = **Phase O** `G-CLAUDE-CODEX-RECEIPT`) ; (b) le materiel `outbox.receipt` n'est PAS re-verifie contre son digest journalise A LA LECTURE — l'integrite du materiel repose sur l'ecrivain unique lease-fence et la frontiere fichier OS mono-utilisateur (A11 : un adversaire avec acces en ecriture au fichier SQLite re-ecrirait journal ET outbox de facon coherente ; une verification en lecture serait defaite par le meme adversaire) ; le receipt verifie-au-digest + signe est la cible **Phase O**. Un receipt RECUPERE par reconciliation passe par la porte completion (digest journalise) derriere une entree `reconciliation_recorded` — jamais une re-etiquette. |
| **T-RUN-FS-EFFECT-ESCAPE** | *Tampering* — un effet FS ecrit HORS du worktree cible (traversal `..`, symlink de chemin OU de feuille), ou mute un arbre qui a diverge de l'etat planifie | **fermee (confinement + CAS d'etat)** | `WorktreeFsWriteActivity` : chemin worktree-RELATIF (absolu et `..` refuses syntaxiquement), puis parent CANONICALISE contraint `starts_with(root canonicalise)` — un symlink DANS LE CHEMIN resout dehors et est refuse ; et le composant FEUILLE est verifie separement (`symlink_metadata` — un lien feuille existant est REFUSE plutot que suivi par `File::create`, sinon `wt/out -> /etc/x` ecrirait dehors ; test `fs_effect_leaf_symlink_refused` cfg(unix)) ; le tout AVANT tout octet. Precondition d'ETAT (`WorktreeState` head/index/worktree digests) verifiee immediatement avant l'ecriture, mismatch = refus fail-closed NON-retryable (`fs_effect_bound_to_head_index_worktree`, `fs_effect_confinement_refuses_escape`). L'etat ATTENDU est injecte, l'etat REEL vient d'un provider injecte (l'engine ne lit ni git ni env — charte). **Borne honnete** : pas de transaction OS sur « comparer puis ecrire » — un mutateur EXTERNE peut s'interposer dans la fenetre (y compris planter un symlink feuille entre le check et le create) ; un LIEN DUR feuille meme-FS n'est pas distingue d'un fichier regulier par `symlink_metadata` (ecrire a travers mute l'inode partage — meme frontiere OS mono-utilisateur que le reste du store) ; ce que la precondition ferme, c'est d'agir sur un plan fait contre un etat DEJA disparu. |
| **T-RUN-PROCESS-SPAWN-INJECTION** | *Tampering/EoP* — injection d'argument/env dans la commande spawnee ; un contenu de plan/journal atteint `Command` ; heritage de secrets par l'environnement ; un cwd ambiant herite fait designer un autre binaire/arbre a un resume | **fermee (fixation constructeur)** | `ProcessActivity` : PROGRAMME, ARGS, **CWD** et **ENVIRONNEMENT** fixes A LA CONSTRUCTION par l'hote — jamais derives d'une sortie modele ni d'une string de pack, et **le crate ne lit AUCUNE variable d'env ambiante de son propre chef** (charte host-neutre : l'env de l'enfant est la map injectee par l'hote, pas l'env du process courant — fix review K, `lib.rs` charte respectee). Le **cwd est FIGE au constructeur** (canonicalise une fois, dans le `dedup_scope` JCS, applique via `Command::current_dir` — ex-Codex r4 K-PROC-CWD-SCOPE-03 : sans lui, un resume dans un autre repertoire gardait le meme token pendant qu'un programme relatif, ou ses I/O relatives, designait un autre arbre ; tests `process_cwd_is_part_of_dedup_scope` + `process_cwd_canonicalised_in_dedup_scope` + `process_effect_runs_in_fixed_cwd`). Le **programme** est ABSOLU-ONLY (ex-Codex r5 K-PROC-PROGRAM-RESOLVE-05 + r6 K-PROC-BARE-PATH-AMBIENT-06) : chemin ABSOLU fige canonique au constructeur (test `process_program_canonicalised_in_dedup_scope`) ; TOUTE epellation non-absolue est REFUSEE a `start` AVANT tout spawn — un relatif a une resolution parent-vs-enfant plateforme-ambigue (std doc, test `process_relative_program_refused`) et un NOM NU ne peut PAS etre lie au seul `PATH` scope : le resolver std Windows (`resolve_exe`) retombe sur le repertoire de l'executable parent, les repertoires systeme puis le `PATH` AMBIANT du parent quand la map enfant ne fournit pas le nom — meme scope, deux binaires possibles (test `process_bare_program_refused`). Le moteur ne resout RIEN lui-meme (reimplementer un lookup d'executable sur une porte de securite = hors charte) — l'HOTE injecte un chemin absolu. Sous WINDOWS l'absolu doit EN PLUS finir par `.exe` (refus a `start` sinon, ex-Codex r7 K-PROC-ABS-EXE-SUFFIX-07 : `resolve_exe` std essaie le SIBLING `path.exe` avant `path` — un scope, deux fichiers possibles, bifurcation pilotable par creation/suppression du sibling ; test `process_absolute_without_exe_refused_on_windows`). AUCUN shell ; `env_clear()` + la map hote + l'input d'effet dans UNE variable (`SBFB_EFFECT_INPUT`, JCS canonique) ; stdin/stdout/stderr null. **Residus honnetes (frontiere OS mono-utilisateur)** : la fermeture vaut au perimetre des chemins ABSOLUS/EXISTANTS supportes — un cwd, une racine worktree ou un log fenced au PARENT jamais resolu (dernier recours raw de `freeze_side_output_path` ; la feuille nue est desormais figee contre le cwd de CONSTRUCTION, ex-Codex r6 K-FENCE-BARE-LEAF-06) sont des dependances ambiantes DOCUMENTEES aux constructeurs, pas des fermetures ; l'input voyage dans une variable d'env, lisible via `/proc/<pid>/environ` par un process co-utilisateur — meme frontiere de confiance que tout le run store, un adaptateur futur peut le passer par stdin ; `display()` d'un chemin non-UTF-8 est lossy dans le token de scope (epellations Unix exotiques — meme frontiere, borne `process.rs`/`fs_effect.rs`, ex-Codex r4 K-SCOPE-PATH-ENC-03). |
| **T-RUN-DESCENDANT-SURVIVAL** | *EoP/DoS* — un descendant survit au cancel/timeout/crash (double-fork escape Unix, breakaway Windows) et continue d'agir apres que le run est clos | **fermee Windows / bornee Unix** | Windows : Job Object **kill-on-close** + spawn `CREATE_SUSPENDED` → assign → resume (ToolHelp, retour `ResumeThread` verifie ≠ (DWORD)-1) — un enfant rapide ne peut pas forker AVANT d'etre dans le job (ferme la course assign-after-spawn du patron daemon), et un enfant qui echoue a etre assigne/resume est TUE avant propagation (pas de leak suspendu) ; la mort du runtime ferme le handle → l'OS tue l'arbre entier (crash-cleanup sans bookkeeping, capture TOUS les descendants sauf breakaway explicite) ; cancel/timeout = `TerminateJobObject`. Unix : `process_group(0)` + `kill(-pgid, SIGKILL)` ; Linux ajoute `PR_SET_PDEATHSIG` ; le pgid est journalise (`attempts.runtime_handle`) et un resume REAPE l'orphelin (`reap_orphan`, garde `pgid <= 1`) — y compris **le drain du cancel qui reape desormais un orphelin d'incarnation precedente** (`UnknownAttempt` → `reap_orphan(handle)`, ex-K-CANCEL-01). Test temoin ANTI-VACUITE : le petit-fils PROUVE sa vie (heartbeat fsync) avant le kill, puis sa cessation (2 lectures stables) — jamais un check PID race (`descendant_process_cleanup_{windows,unix}`). **Bornes honnetes Unix EXPLICITES** : (a) un descendant qui appelle `setsid()` cree sa propre session/groupe et ECHAPPE a `kill(-pgid)` — le process-group n'est pas une contention forte comme le Job Object Windows (meme borne que le patron `local_worker.rs` ; une contention forte exigerait cgroups/PID-namespaces, hors charte std) ; (b) le `runtime_handle` (pgid) n'est persiste qu'APRES le spawn dans une transaction distincte — un crash dans la fenetre spawn→persist laisse un pgid non-rejouable (variante irreductible de la fenetre B2 ; sur Linux `PDEATHSIG` couvre l'enfant DIRECT, pas les petits-enfants hors groupe ; macOS sans pdeathsig) ; (c) pgid Unix reutilisable → le reap est best-effort diagnostique, jamais une autorite. Windows reste FERMEE (kill-on-close ne depend d'aucun handle persiste). |
| **T-RUN-RECONCILE-LAUNDER** | *Tampering/Repudiation* — une reconciliation menteuse ou un replay re-etiquette un `outcome_unknown` en succes ; un resultat tardif « prouve » a posteriori un succes jamais observe | **fermee (evidence-only, absorbant)** | La reconciliation est une DECISION JOURNALISEE d'origine FERMEE (`ReconcileOrigin::HostOperator` seul — aucun variant modele/pack, aucun texte libre), geste OPERATEUR-LOCAL sans route (precedent restore/GC). `result_recovered` EXIGE le receipt reel (refus `ReconcileNeedsReceipt`) et le passe par la porte COMPLETION — le succes du node repose sur l'EVIDENCE, pas sur la decision ; le reducer REFUSE une completion sur un node parque sans reconciliation prealable (`CausalOrder`, la forme meme du blanchiment) ; un run TERMINAL `outcome_unknown` est ABSORBANT (`require_running` + `InvalidTransition` — reconcilier un terminal est refuse, `reconcile_abandon_is_terminal_and_absorbing`) ; `abandon` ferme honnetement en `outcome_unknown`, jamais en succes. Le PARK lui-meme est OPT-IN hote (`park_unknown_for_reconciliation`, defaut = terminal J inchange — §12.7 « pas de propagation sans politique explicite »). |
| **T-RUN-RETRY-STORM** | *DoS local* — un backoff absent/mal borne fait tourner en boucle un effet qui echoue ; l'ecriture journal des attempts s'amplifie sans plafond | **fermee (budget + pacing borne)** | Plafond DUR : `1 + max_retries_per_node` tentatives (champ du plan signe, premier consommateur K), depassement → `failed_terminal` → policy (`node_failed` terminal ou route `error`) — jamais un retry infini (`retry_budget_exhausted_closes_run_node_failed`) ; backoff exponentiel BORNE (`RETRY_BACKOFF_BASE_MS` 1s → `RETRY_BACKOFF_MAX_MS` 60s) via timers durables `er:` a deadline ABSOLUE (INSERT OR IGNORE, premiere-gagne) — pacing pur : change QUAND un retry part, jamais un resultat ; un `outcome_unknown` n'est JAMAIS retente (0 attempt-failed journalise sur ce chemin, `non_idempotent_without_receipt_becomes_unknown`). **Robustesse crash** : le failure journalise et son timer `er:` sont ecrits en DEUX transactions ; un crash entre les deux perdrait le timer et FIGERAIT le run en `failed_retryable` — le re-dispatch RE-ARME le timer idempotemment a chaque passe (INSERT OR IGNORE, miroir du re-propose du workflow-timeout, ex-K-RETRY-01), un crash retarde le retry d'un backoff mais ne le perd jamais. |
| **T-RUN-EFFECT-TIMEOUT-TOCTOU** | *Tampering* — une horloge manipulee entre le jugement de deadline et la decision re-ouvre T-RUN-TIMER-SKEW par la couche effets | **fermee (pattern J suivi)** | Toute deadline d'attempt est ABSOLUE (`attempts.deadline_ms` = lecture watermarkee + timeout du plan), jugee contre le `now` UNIQUE de la passe (lecture unique, watermark durable en tete de passe — pattern J, aucune re-lecture d'horloge dans `judge_deadline`) ; les timers `er:` reutilisent la table J (CAS `fired 0→1`, gate run RUNNING). Pas de row neuve : contrainte d'implementation TRACEE pour que T-RUN-TIMER-SKEW ne re-ouvre pas silencieusement. |

### 19.3 Surface permissions et attenuation (Sprint 83 Phase L)

Le module capability (`crates/sbfb-workflow-engine/src/capability.rs`) est la
**COMPUTATION autoritaire** de `effective_capabilities` = l'intersection
monotone-attenuante des SEPT couches d'autorite (§14.1) sur DIX-SEPT classes
de capacites TYPEES (§14.2). Il est PUR : 0 I/O, 0 crypto, 0 derive serde
(rien n'est serialise — census DOMAIN 29 inchange, aucun bump wire : les
versions de format du chantier `PACK`/`RUN_EVENT`/`SNAPSHOT`/`FEED` restent
a 1), 0 lecture d'env ambiante ; la couche 1 (OS toggle)
et la couche 2 (policy locale) viennent d'un trait `CapabilityPolicy`
INJECTE (meme couture que `Clock`/`SignalSource`), `now_ms` est un
parametre. **Pre-coupe AM-4** : L = permissions/attenuation SEUL ; le coffre
`secret_ref` et le rendu/signature HITL = L1. **DS-1** : L livre la
computation + les property tests ; la RELIURE de l'intersection dans le
dispatch d'effet (`orchestrator::run_effect`) et le journal des refus
d'autorite sont la phase qui shippe la surface run (les notes de conception
retry/replay vivent dans cette phase, pas comme call-sites ici).

**Asset A13 — grants et autorite effective.** Trust boundary : structures
host in-memory, jamais serialisees ni signees, jamais cross-noeud.
Invariant cardinal (property-teste, 21 mutations discriminantes tuees +
garde d'exhaustivite compile-time) :
**chaque couche PRESENTE ne peut qu'ATTENUER** — ni l'app, ni le pack, ni un
agent, ni un serveur MCP ne peut elargir l'ensemble. Miroir DISCIPLINE (pas
partage) de l'algebre canonique `sbfb-manifest::grant_v0` (Phase C) :
`None`=identite de l'attenuation, expiry fail-closed a la borne
(`now_ms >= exp`), `revoked` re-verifie a chaque appel jamais cache, borne
de domaine des DEUX cotes. L'engine ne depend PAS de `sbfb-manifest` (mauvais
domaine : grant_v0 est borne aux 16 verbes bridge) — la parite est tenue par
tests miroirs, pas par un appel de fonction (generaliser grant_v0 driftrait
silencieusement le miroir TS `authority.ts` non machine-gate).

| # | Menace (STRIDE) | Sev | Mitigation cablee |
|---|---|---|---|
| **T-CAP-WIDEN-BYPASS** | *Elevation of privilege* — une couche ELARGIT au lieu d'attenuer, un chemin de construction saute le meet, ou l'AGREGATION de plusieurs grants fabrique une capacite qu'aucun grant seul ne conferait | **fermee (widening) / bornee (forgeabilite du type-resultat)** | `EffectiveCapabilities` a AUCUN `insert`/`widen`/`extend` public (`inner` prive) — toute couche passe par `meet` (intersection de cles + meet par champ) ou par un `filtered` (ne retire que des cles), donc aucune COUCHE n'elargit. Borne honnete (Codex r3) : l'unique constructeur arbitraire `from_declared` produit le MEME type que le resultat ; un resultat calcule et un candidat force par l'appelant ne sont pas distinguables au niveau TYPE — un newtype-resultat non forgeable est un livrable de la RELIURE DS-1 (§Phase V req 7), pas de L. Property test `intersection_only_attenuates` : 2000 entrees seedees, chaque configuration des 7 couches → `eff.refines(base)` (temoins de non-vacuite ASSERTES : ≥1 survivant, ≥1 survivant strictement resserre — sinon la propriete etait vide). Lois du meet (`commutative`/`associative`/`idempotent`/absorbant ∅) property-testees. **Meet par CHAMP, direction PAR CHAMP** : `min(quota)`, `min(timeout)`, `∩(destinations)`, `max(niveau d'approbation)`, et **`DataPolicy` = UNION de flags orthogonaux** (`no_egress`/`no_persist` OR — jamais un `max` total qui perdrait un axe : correctif Codex/review F3) ; un `min` copie-colle sur l'approbation serait une union accidentelle (mutation `approval_meets_by_max_not_min`). **Agregation multi-grants de MEME audience/ref = DOMINATION GLOBALE ou DROP, permutation-invariante** (`dominating_grant`, Codex r1+r2 P1) : parmi TOUTES les bornes grantees d'une ref, s'il en existe UNE que toutes les autres RAFFINENT (le maximum unique de l'ordre), elle est conservee (une borne reellement grantee, jamais un LUB synthetique qui aurait rendu `{a,b}`/100/`Auto` a partir de `{a}`/100/`Human`+`{b}`/1/`Auto`) ; sinon (ensemble sans dominant, ou mismatch de classe) la ref est DROP fail-closed. Le calcul cherche le maximum sur l'ENSEMBLE, pas par fold pairwise — donc `{a}`,`{b}`,`{a,b}` conserve `{a,b}` dans TOUTES les permutations (un left-fold aurait empoisonne `{a}`+`{b}` puis droppe `{a,b}` selon l'ordre). Tests `multi_grant_same_audience_combines_soundly` (6 permutations de 3 grants, comparable dans les 2 sens). **Borne honnete** : le drop d'un ensemble sans dominant est SILENCIEUX et non-monotone (ajouter un grant peut retirer une autorite) — la reliure DS-1 doit le traiter comme erreur de config operateur, pas s'y reposer. |
| **T-CAP-CLASS-UNKNOWN-FAILOPEN** | *Elevation of privilege* — une ref sans classe mappee, ou une classe `destructive` mal-mappee, retombe sur une classe permissive par defaut | **bornee (enum ferme ; frontiere de resolution = reliure)** | Les 17 classes sont un ENSEMBLE FERME (`CapabilityClass::ALL`, `seventeen_classes_typed` : exactement 17, distinctes, garde PARTIELLE de drift par `match` sans wildcard — ajouter une variante sans l'inscrire au `match` casse la compilation ; l'oubli de `ALL` seul reste non-detecte, borne documentee : necessiterait `variant_count` ou une macro source-unique). L'autorisation keye sur la CLASSE typee, jamais sur un substring de nom de methode — le crate n'importe PAS `operator_server::SENSITIVE_ACTIONS` (structurel : pas dans `Cargo.toml`) et `commit`/`push` sont legitimement 2 des 17 classes. Une ref hors closure du plan compile est refusee des DEUX cotes (borne de domaine, `undeclared_capability_refused` discriminant). **Borne honnete (Codex r1 P2)** : le mapping `capability_ref` (String cote pack/plan `pack.rs`/`graph.rs`) → `BoundedCapability` typee n'a PAS encore de resolveur autoritaire « inconnu ⇒ drop » — il naitra a la RELIURE (phase run-surface, DS-1) ; et `from_declared` est un constructeur public du type-resultat, qu'une API de dispatch future devra distinguer d'un set calcule. Aujourd'hui L ne relie rien (0 call-site) : la surface de resolution reste a fermer cote reliure. |
| **T-CAP-GRANT-AUDIENCE-CONFUSION** | *Spoofing* — un grant accorde a la RELEASE de l'app est applique a la couche version-du-pack (ou l'inverse) = elargissement silencieux (confused deputy) | **fermee** | Chaque `CapabilityGrant` porte une `audience` (`Release`|`Pack`) ; `live_for(audience, now)` ne contribue QUE si l'audience correspond ET que le grant n'est ni revoque ni expire. Un grant offert a la mauvaise couche est INERTE. `audience_enforced` : une ref n'existe que si un grant vivant la confere dans les DEUX audiences ; un grant release place dans le slot pack → couche pack vide → refus (mutation « ignore audience » tuee). |
| **T-CAP-EXPIRY-TOCTOU-RETRY** | *Tampering* — une reemission (Phase K re-arme les retries) reutilise une autorite calculee a la premiere tentative alors que le grant a expire/ete revoque depuis | **bornee (contrainte gravee, reliure DS-1)** | La computation est PURE et prend `now_ms` en parametre : elle n'a pas de call-site en L (module autoritaire seul, DS-1). La contrainte pour la phase de reliure est GRAVEE : l'intersection est recalculee A L'EMISSION, par-attempt, contre le `now_ms` watermarke (pattern J horloge lecture-unique) — une reemission K est une NOUVELLE emission, jamais une reutilisation d'un verdict d'autorite mis en cache. `revocation_per_call` prouve deja qu'un grant revoque entre deux computations ne confere plus rien (jamais cache). |
| **T-CAP-REPLAY-REAUTHORIZE** | *Repudiation/Tampering* — le replay du journal re-autorise un effet contre la policy LIVE au lieu de l'autorite du moment | **bornee (contrainte gravee, reliure DS-1)** | Contrainte gravee pour la phase de reliure : l'autorite gate l'EMISSION cote NON-rejoue ; un refus d'autorite est une DECISION JOURNALISEE ; le replay lit le JOURNAL, jamais la policy live (sinon reouverture de StepHashMismatch / TOCTOU-retry fermes v25/v26). Aujourd'hui L ne journalise rien (pas de call-site) — la note vit dans la conception de la phase run-surface. |
| **T-CAP-DEPTH-AMPLIFICATION** | *Elevation of privilege* — empiler des couches AMPLIFIE l'autorite au lieu de l'attenuer | **fermee (chaine monotone)** | La « profondeur » en scope L est la longueur de la CHAINE de couches : `depth_enforced` prouve qu'ajouter une couche au fold ne peut qu'attenuer (`deeper.refines(acc)`, 1000 chaines seedees). **Borne honnete** : la profondeur de DELEGATION (`DelegationGrant.v1`, research §14.5) est un saut post-MVP NON modelise ici ; la garde depth/cycles des chaines `app.call` est **Phase Y** (`G-APPCALL`). L ne borne QUE la chaine de couches, pas une delegation. |

**Bornes honnetes (ce que L ne ferme PAS)** : la RELIURE de l'intersection au
dispatch d'effet et le journal des refus = phase run-surface (DS-1, `T1 =
N-A-no-frontend-change`, l'UI capacites est Phase U) ; le coffre `secret_ref`
+ le HITL signe (principal/audience/nonce) = **L1** (la couche 7 ici ne porte
qu'une reference de decision OPAQUE — VIDE en L, fail-closed sur les classes
`Human`) ; le descripteur riche §14.2 (schemas de ports) reste la
responsabilite du COMPILATEUR (typage de ports G/H), non re-modelise ; la
persistance des grants, si une phase l'ajoute, DOIT avoir des defauts serde
fail-closed (jamais `#[serde(default)]` permissif sur `revoked`/`expires_at_ms`,
doctrine grant_v0). Doctrine duress : AUCUN gate dans l'engine (pattern I1 —
le trait policy est INJECTE, la lib n'a pas d'identite). Le residuel Operator
SENSITIVE_ACTIONS substring (§ residuel operateur) N'est PAS ferme ici : L ne
l'importe pas, capacites TYPEES != gate lexical. Palier T2 WA-T5.2 (refus
grant expire / capacite non declaree) : evidence = property tests
(`expiry_enforced`, `undeclared_capability_refused`), `NOT-RUN` jusqu'a AI2.

**Note L1 (ajoutee par la phase suivante, promise au preflight L1).** La couche
7 a desormais un PRODUCTEUR : `hitl::decide_ref` (§19.4) transforme une
approbation hote-verifiee, ACCORDEE, non expiree et liee A CETTE `capability_ref`
en membre de `HitlValidity::decided_refs` ; la composition est prouvee par
`decided_ref_gates_layer_seven` (ref decidee franchit / absente tombe / tuple
MUTE tombe). Ce que cela ne change PAS : la reliure au dispatch reste DS-1, donc
en production `HitlValidity` reste VIDE et une capacite `Human` reste refusee —
la formulation de A13 (« la couche 7 HITL vide ne refuse que les classes
`Human` ») demeure exacte.

### 19.4 Surface coffre de secrets et HITL hote signe (Sprint 83 Phase L1)

L1 livre le PRODUCTEUR de la couche 7 (HITL) laissee VIDE en L — la couche reste
VIDE en production (cf. DS-1) — et ajoute le coffre de secrets hote. Deux
surfaces, chacune SPLIT en « ce qui rend / ce qui fait autorite » pour tenir la
charte engine-keyless + la Day-0 census 29 :

- **Coffre** (`crates/nexus-core-rs/src/vault.rs`) : secrets nommes scelles au
  REPOS, lies a leur reference opaque JUSTE-AVANT-l'appel. REUTILISE la crypto
  keystore (Sprint 20) — Argon2id(PIN) → KEK, AES-256-GCM, `DOMAIN_KEYSTORE_V1`
  comme domaine partage (sous-contexte `secret-vault`, pas un `DOMAIN_*_V1`
  neuf), `SecretBox`/`zeroize`, ecriture atomique 0600. La defense PRIMAIRE au
  repos est Argon2id(PIN) ; l'OS-keyring n'est PAS cable (backend MOCK dans ce
  workspace — le revendiquer serait faux). C'est un artefact LOCAL, jamais
  gossipe : 0 famille de signature, 0 bump wire.
- **Seam secret engine** (`crates/sbfb-workflow-engine/src/secret.rs`) : le
  moteur ne connait QUE la `SecretRef` opaque ; `BoundSecret` n'a AUCUN
  `Serialize` (compile-time), un `Debug` redacteur, et se zeroize au drop.
- **Liaison HITL engine** (`crates/sbfb-workflow-engine/src/hitl.rs`) :
  `hitl_binding_digest` sur le tuple EXACT (15 champs) ; `verify_binding`
  refuse toute mutation (digest != ) et l'expiration ; PUR, keyless.
- **Signature HITL hote** (`crates/nexus-core-rs/src/hitl_approval.rs`) : le
  daemon (detenteur de l'identite) signe le digest du binding avec un
  context LOCAL (`sbfb-hitl-approval-v1`, pas un `DOMAIN_*_V1`) ; l'iframe non
  fiable ne detient pas de cle et ne peut forger une approbation.
- **Affordance host-chrome** (`web/src/pages/Approvals.tsx`, route
  `/approvals`) : le tuple exact est RENDU par le shell hote (WYSIWYS), file
  host-owned, approbation accessible (focus gere, pas de keyboard-trap, pas de
  type-to-confirm, pas de delai). Portee L1 : affordance minimale ; le Centre
  des Runs plein = Phase V.

**DS-1 — ce que L1 NE RELIE PAS (lire avant de croire cette section).** L1 livre
les DEUX primitives (coffre, binding+signature) et leurs tests ; la RELIURE
n'existe pas encore : **0 call-site de production** pour `SecretVault`,
`SecretResolver`, `verify_binding`, `decide_ref`, `sign_hitl_approval` et
`verify_hitl_approval` (hors re-exports), aucune `HitlValidity` construite en
production, aucun champ resolveur dans `EffectContext`, et aucune route daemon.
Le daemon qui VERIFIE la signature, l'adaptateur d'effet qui resout le
`secret_ref` juste-avant-l'appel, la couche 7 alimentee et le refus
d'autorite journalise sont la phase qui shippe la surface run — **meme posture
DS-1 que la Phase L** (§19.3). Les statuts ci-dessous se lisent donc comme des
proprietes de la PRIMITIVE, pas d'un chemin cable : partout ou l'enforcement
depend de la reliure, le statut est **bornee**, jamais fermee.

**Asset A14 — coffre de secrets hote.** Trust boundary : blob chiffre au repos
sur le noeud, jamais cross-noeud. Confidentialite **Critique** (materiel de
credential), disponibilite Moyenne. Invariant : le materiel ne franchit AUCUNE
des surfaces interdites §14.3 (iframe/**app-KV**/pack/lock/layout/event-log/
snapshot/prompt/contexte/Task/trace/metrique/erreur/ArtifactRef/export/dataset —
16 items ; l'enumeration precedente en omettait un, `app KV`, et non le compte) ; seule
la reference OPAQUE y circule legitimement — y compris DANS le pack, ou elle est
un nom de slot par conception (`pack.rs`), ce qui est la lecture correcte de
research §14.3 : c'est le materiel qui est banni partout, pas la reference.

**Asset A15 — approbation HITL signee chrome hote.** Trust boundary : decision
LOCALE consommee avant UN effet, jamais serialisee cross-noeud ni gossipee (0
`DOMAIN_*`, census 29 tenu). Integrite **Critique** (une approbation forgee ou
mutee = EoP sur un effet sensible), confidentialite **Moyenne** (le tuple porte
un principal, une destination et un digest — des metadonnees d'exploitation, pas
du materiel), disponibilite Moyenne.

| # | Menace (STRIDE) | Sev | Mitigation cablee |
|---|---|---|---|
| **T-SECRET-REF-EXFIL** | *Information disclosure* — le materiel d'un credential fuit via une surface durable (journal/receipt/snapshot/trace/export) au lieu de rester derriere la reference opaque | **fermee par TYPE (serialisation) / bornee (reliure)** | Ce que le TYPE ferme, definitivement : `BoundSecret`/`VaultSecret` n'ont AUCUN `Serialize`, donc le materiel ne PEUT PAS etre un champ d'un `RunEvent`/`ActivityReceipt`/snapshot (erreur de compilation, pas de revue) — et cette absence est GARDEE par **DEUX doctests `compile_fail,E0277`**, un par type — `BoundSecret` cote moteur (`secret.rs`) et `VaultSecret` cote hote (`vault.rs`) —, chacun porte par le type lui-meme et chacun mesure DISCRIMINANT (injecter un `impl Serialize` qui expose le materiel fait ECHOUER le doctest correspondant). Le code d'erreur est epingle pour qu'un doctest ne puisse pas verdir sur une erreur de compilation sans rapport. Le canari, lui, n'a jamais garde cette classe : aucune de ses surfaces ne serialise l'un ou l'autre type ; `Debug` redacteur epingle en EGALITE EXACTE (un `!contains(canari)` etait aveugle a un `{:?}` du slice, qui deverse tout en decimal) ; zeroize au drop. Seule la `SecretRef` (un NOM) circule. Precedent : projection needs-free PAR CONSTRUCTION (A9, `T-A11Y-NEEDS-LEAK`). Ce que le type NE ferme PAS : rien n'empeche un appelant d'`expose()` puis de recopier les octets dans un `String` qui, lui, se serialise — c'est la reliure (adaptateur d'effet) qui doit ne jamais le faire, et il n'existe pas encore (DS-1). Canari WA-T1 `secret::tests::canary_absent_everywhere` : **bras ROUGE sur la surface de PROD** (une `ActivityReceipt` dont les `facts` portent le materiel expose DOIT etre detectee) puis bras vert sur les 5 surfaces reelles, recherche sous SIX encodages — texte, tableau decimal `serde_json`, tableau decimal `Debug`, **texte JSON-echappe** (derive du serialiseur lui-meme, pas devine), **hex**, **base64 standard et url-safe** (l'encodage d'un `Authorization: Basic`, le plus plausible d'un credential cloud) — soit SEPT bras de recherche pour six familles d'encodage (base64 standard et url-safe partagent la famille), **chacun avec son controle positif** : cinq dans `the_canary_oracle_detects_base64_hex_and_json_escaping`, les deux premiers (texte brut, tableau decimal `serde_json`) dans les bras ROUGES de `canary_absent_everywhere` lui-meme — et les encodeurs sont ecrits a la main pour ne pas depenser la Day-0 0-dep sur un helper de test ; jumeau at-rest `vault_bind_rotate_zeroize` avec ses deux controles positifs, RECOPIE et non partage (les deux crates ne dependent pas l'un de l'autre) et volontairement moins riche : il fouille un blob scelle, pas un document rendu. Sans le bras rouge, le canari passait vert meme en derivant `Serialize` sur `BoundSecret` — il ne prouvait que « je n'ai rien mis dedans ». |
| **T-SECRET-VAULT-LIFECYCLE** | *Tampering/Information disclosure* — rotation qui laisse l'ancien materiel recuperable, suppression incomplete, KEK persistant en memoire, ou downgrade des parametres KDF du blob | **fermee (falsification PARTIELLE au repos) / bornee (rollback du fichier entier)** | Chaque entree scellee AES-256-GCM sous KEK avec nonce FRAIS ; AAD = `DOMAIN_KEYSTORE_V1` + sous-contexte + header (salt+params KDF — `param_downgrade_in_header_is_refused`) + NOM de la ref length-prefixe (`tampered_ref_binding_fails`). **ENTREE-VERIFICATEUR** (`VAULT_CHECK_REF`, inatteignable car `validate_ref` refuse les caracteres de controle — `check_entry_is_unreachable_from_user_operations`) : ouverte INCONDITIONNELLEMENT a l'unlock, elle prouve le PIN meme sur un coffre VIDE (`wrong_pin_on_empty_vault_is_refused` — avant, tout coffre neuf ouvrait sous n'importe quel PIN et un `put` perdait le secret sous un KEK errone), authentifie le header meme a vide, et ne dechiffre AUCUN credential reel (avant, la sonde ouvrait l'entree lexicographiquement premiere dans un `Vec<u8>` non zeroise). Son plaintext porte `epoch ‖ blake3(set trie)` : rejeu d'une ancienne `(nonce,ct)` (`replaying_an_old_ciphertext_is_refused`) et delete silencieux (`deleting_an_entry_behind_the_vault_is_detected`) sont DETECTES — donc **la rotation est durable dans les DEUX sens**, sous les bornes ci-dessous. **Lecture ET ecriture** : `bind` relit l'epoch on-disk et fail-close en `ConcurrentModification` si le handle est perime (`stale_handle_bind_serves_nothing_after_a_revocation` — sans cette moitie, un daemon a handle long-lived continuait de SERVIR un credential qu'un autre process avait supprime ou rote, l'AAD d'une entree ne liant pas l'epoch) — **sauf apres la collision lost-update de la borne (3) ci-dessous**, ou le handle porte l'epoch du disque avec un BODY perime et ou `bind` sert donc une entree revoquee, recovery par `reload()` sans re-saisie de PIN ni Argon2 ; chaque commit relit et compare l'epoch monotone (`stale_handle_write_is_refused_not_silently_applied`). `create_new` a l'init (et suppression du fichier reserve si l'ecriture echoue — sinon `AlreadyInitialised` a vie), scratch unique par ECRITURE (`{pid}.{alea}`), `sync_all` du scratch avant le `rename` (inconditionnel) + fsync du repertoire best-effort (Unix, echec avale — cf. borne 11). KEK dans un `SecretBox` zeroize au drop ; tout plaintext ouvert l'est dans un `Zeroizing` ; le digest de derivation du KEK et le bloc retenu par le hasher sont essuyes AUX DEUX sites miroirs (`derive_vault_kek` et `combine_keks`). Vecteur dore hex sur la formule de derivation (`vault_kek_is_domain_separated_from_keystore`) : sans lui, un `blake3(kek1)` nu — sans domaine ni sous-contexte — laissait toute la suite verte. **Bornes honnetes** : (1) rembobiner le FICHIER ENTIER vers une version anterieure coherente (blob + son entree-verificateur, via backup/snapshot FS, coupure secteur incluse) reste indetectable de l'interieur — il faut un compteur hors-artefact, et le precedent maison le moins cher est un SIDECAR (`anchors.json` persiste la `revision` comme plancher anti-rollback, `iroh_runtime.rs`, test committe) : **TPM/keyring n'est PAS un prealable** ; (2) un `delete` retire du set scelle, ce n'est pas un wipe-secteur (les blocs de l'ancien inode subsistent) ; (3) le scratch etant unique par ECRITURE, il n'y a plus de collision de scratch ni pour un process ni pour deux — mais la fenetre relecture→`rename` n'est verrouillee dans aucun des deux cas : deux writers concurrents peuvent encore perdre une mise a jour (dernier `rename` gagnant, les deux blobs portant le MEME epoch, donc rollback indetectable entre eux). L'epoch est un compteur de version, pas un jeton propre au writer. **Et la consequence depasse « une mise a jour perdue »** : une fois la collision survenue, les DEUX handles portent l'epoch du disque, donc `refuse_if_stale` passe pour chacun alors que l'un detient un BODY perime. Chacun dispose alors d'UNE ecriture non gardee, et une ecriture ulterieure de l'un annule silencieusement la revocation de l'autre — la classe ex-P1-4, sans adversaire. C'est un des etats — PAS le seul — ou la moitie LECTURE ci-dessus tombe : `bind` sert alors une entree revoquee. Les deux autres, sans aucune collision (Codex round 3, borne 27) : (i) la garde d'epoch et la lecture du body ne sont pas atomiques, donc un `bind` ADMIS a l'epoch `n` peut se terminer APRES un `delete` commis a `n+1` ; (ii) meme termine AVANT la revocation, le `VaultSecret` deja rendu est consomme par le fournisseur APRES elle. Les deux sont LINEARISABLES (le bind est admis a la garde) et relevent donc de la SEMANTIQUE a enoncer, pas du defaut : un bind admis avant une revocation peut se terminer — et etre utilise — apres elle. Seule une revocation dure (bail tenu jusqu'au point d'usage du fournisseur) les fermerait. Fermeture = un verrou couvrant relecture→`rename` (verrou consultatif de repertoire, ou sentinelle `create_new`) ; c'est de la dette, comme le fsync de repertoire de la borne 11. |
| **T-HITL-APPROVAL-TAMPER** | *Tampering* — un champ du tuple (destination, digest sortant, cout max, verdict, capacite...) est mute APRES l'approbation humaine pour rediriger/amplifier l'effet | **bornee (primitive gravee, reliure DS-1)** | Ce que la primitive ferme : l'approbation est liee au `hitl_binding_digest` du tuple COMPLET — **15 champs**, dont `capability_ref` (une approbation obtenue pour une capacite ne peut en decider une autre, `approval_is_bound_to_one_capability`) et `decision` (un refus et une autorisation n'ont JAMAIS le meme digest, `a_refusal_never_decides_a_ref`) ; a l'emission le moteur recalcule le digest depuis le tuple REEL et exige l'egalite. `mutation_invalidates_approval` : un mutateur par champ, exhaustivite tenue par DEUX controles aux portees DISTINCTES — la destructuration sans `..` attrape un champ AJOUTE a la struct (ne compile plus), le `assert_eq!(mutators.len(), 15)` attrape un mutateur RETIRE. Ni l'un ni l'autre n'attrape le troisieme cas : un champ ajoute puis silencie par `champ: _,`. Ce residu est ASSUME et borne — un champ ordinaire rougit deux fois ailleurs (vecteur dore du digest, jambe `HITL-PARITY`), seul un champ `#[serde(skip)]` passerait, c'est-a-dire precisement celui pour lequel « toute mutation invalide » serait faux. Plus un controle positif de non-vacuite, et un test qui valide `HitlBinding::new` POSITION PAR POSITION (11 champs chaines, chacun avec l'assertion du `kind` rapporte — sans quoi une boucle validant onze fois le meme champ passait). Composition prouvee jusqu'a la couche 7 : `decided_ref_gates_layer_seven` (ref decidee franchit / absente tombe / tuple mute tombe). Ce qui reste borne : le moteur est KEYLESS, il ne verifie aucune signature ; il exige un temoin `HostVerifiedApproval` dont le seul constructeur est nomme d'apres sa precondition (`attest_verified_signature`), ce qui ferme l'accident de l'AUTO-VALIDATION (l'ancienne signature `signed_digest: &str` acceptait un digest que l'appelant calculait lui-meme) mais PAS le mensonge d'un hote qui attesterait sans verifier. L'ensemble d'autorite est desormais TYPE : `HitlValidity::decided_refs` est PRIVE et ne se peuple que de `DecidedRef`, dont `decide_ref` est le SEUL producteur — le contournement « je fabrique moi-meme mes refs decidees » (plus court que l'auto-validation, puisqu'il suffisait de construire l'ensemble depuis la file PENDANTE) ne compile plus depuis un autre crate. Portee exacte : `pub(crate)` ferme la porte aux autres CRATES, pas aux autres MODULES de celui-ci — c'est une propriete de TYPE qu'aucun test ne peut faire rougir, et la reliure vivant dans ce meme crate doit passer par `decide_ref`. Et **0 call-site** : aucun code de production ne franchit encore cette porte. |
| **T-HITL-FORGE-REPLAY** | *Spoofing/Repudiation* — une approbation est forgee (sans la cle hote) ou une approbation ancienne est rejouee | **fermee (forgerie de signature) / bornee (replay + reliure) / NON MITIGEE (repudiation, cf. borne 21)** | La signature hote (`sign_hitl_approval`/`verify_hitl_approval`) est une Ed25519 sur `context-local ‖ digest`, verifiable seulement contre la cle identite du noeud — un forgeur sans cle echoue (`a_different_key_is_rejected`), et une signature emise pour un autre usage ne verifie pas comme approbation (`context_separation_binds_the_signature`). La classe SMALL-ORDER est refusee NOMMEMENT (`is_weak()` + `verify_strict`, `a_small_order_key_cannot_forge_an_approval`, patron `pack.rs` de la Phase F) : sans elle, un `approver_pubkey` = point identite avec `(R = identite, S = 0)` satisfait l'equation permissive pour N'IMPORTE QUEL digest, sans cle privee — « le detenteur de la cle a signe » devient une phrase FAUSSE. Durcissement LOCAL : `crypto::verify` reste non-strict pour ses appelants historiques (canari, FROST), qui documentent en dependre. Le test porte l'assertion de sanite du patron (la forgerie satisfait bien l'equation permissive), sans laquelle il ne distinguerait pas une forgerie refusee d'une entree malformee. Replay : nonce mono-usage + seq + expiration sont DANS le tuple (donc dans le digest) et `verify_binding` refuse a la borne (`expiry_is_enforced`). Bornes : la CONSOMMATION mono-usage reste celle de `wait.signal` (`T-RUN-SIGNAL-FORGE-REPLAY` v26 : `INSERT OR IGNORE`, CAS atomique) et **rien ne relie encore les deux** — aucun code ne consomme un nonce a l'occasion d'une approbation ; c'est la reliure. |
| **T-HITL-QUEUE-CONFUSED-DEPUTY** | *Elevation of privilege/Spoofing* — l'iframe non fiable rend ou approuve une demande HITL (confused deputy), ou la file d'approbation disparait avec l'iframe | **bornee (incapacite structurelle prouvee ; file non persistante ; 0 reliure)** | L'iframe ne peut pas ATTEINDRE la file : les verbes du bridge sont une **allowlist FERMEE** de 16 methodes, epinglee par une **liste LITTERALE** de 16 + `ALL_DISPATCH_METHODS.size === 16` dans `web/src/bridge/__tests__/authority.test.ts` : c'est CE controle qui ferme l'allowlist, et il rougit sur tout verbe ajoute. L'assertion NOMMEE voisine (aucun verbe ne matche un radical d'autorite) est un **fil-piege de nommage**, pas la fermeture — portee a enoncer honnetement : elle n'ajoute quelque chose que pour un verbe ajoute EN LOCKSTEP sur les deux miroirs, et sa premiere redaction ratait le vocabulaire du depot lui-meme (`human_decision`, `effect_authorize` : « decision » ne contient pas « decide »), motif elargi depuis en liste de radicaux interdits. La jambe bridge de `check-frontier-contracts.sh` ne teste qu'une PARITE TS↔Rust et resterait VERTE sur un ajout en lockstep : la citer comme autorite de cet invariant etait une erreur (elle est l'autorite de la non-derive du miroir, pas de la fermeture de l'allowlist). L'iframe n'a par ailleurs ni `allow-same-origin` ni CSP ouverte (`csp.rs`) — mais c'est l'allowlist, non la CSP, qui lui retire l'autorite d'approuver. Le rendu est hote (`/approvals`, document top-level : l'e2e compte **tous** les `iframe` a 0, pas seulement le test-id du shell) et la file est un etat du module HOTE. Une app PEUT en revanche IMITER cet ecran dans son iframe — le libelle de la page le dit desormais, car aucune garantie technique ne l'empeche ; ce qu'elle ne peut pas, c'est lire la file ou transmettre une decision. **Bornes** : la file survit a la mort d'une iframe (elle vit dans le document hote) mais PAS a un rechargement du shell — il n'y a AUCUNE persistance, la file durable est celle du daemon (non cablee, Phase V) ; et la file de production est **VIDE** — les fixtures sont injectees par les tests, apres qu'une premiere version ait servi une approbation de demonstration dans un build reel (donnee d'autorite fabriquee, classe interdite par le projet). |
| **T-HITL-COERCION-A11Y** | *Tampering (dark-pattern)* — une approbation par coercition (keyboard-trap, type-to-confirm, delai impose) piege l'operateur ou une AT | **bornee (spec e2e BLOQUANTE)** | Affordance sans dark-pattern : focus gere au montage ET apres chaque decision (le focus ne retombe pas a la racine quand la carte est detruite), pas de keyboard-trap (tabulation reelle prouvee dans les DEUX sens : le focus atteint les boutons puis peut QUITTER la carte), AUCUN type-to-confirm (aucun champ de saisie dans la carte), AUCUN delai (`isEnabled()` — qui ne re-essaie PAS, contrairement a `toBeEnabled()` qui aurait attendu un delai impose et l'aurait appele un succes). Les **14 champs de la DEMANDE** sont affiches avec leurs VALEURS (le 15e, `decision`, est PRODUIT par le bouton : afficher « Verdict : granted » a cote d'un bouton « Refuser » faisait lire a l'humain une decision qu'il n'avait pas prise — et l'ecran enonce desormais que **Approuver et Refuser produisent DEUX digests differents**, le fait le plus important de la surface). Libelles par un `Record<keyof HitlRequest, string>` (un champ ajoute sans libelle casse le build) et ordre de rendu DERIVE des cles de ce meme `Record` (une liste tenue a la main pouvait taire un champ tout en gardant son libelle : les 15 champs etaient comptes, 14 rendus) ; la parite des NOMS de champs Rust↔TS est tenue par la jambe **`HITL-PARITY`** de `check-frontier-contracts.sh` (extraction des deux declarations, self-test de l'EXTRACTEUR sur fixture piegee — une extraction non vide mais FAUSSE rougit ; le garde `-z` couvre le cas vide et IMPRIME son diagnostic (`HITL-PARITY: could not extract fields (RS=0 TS=15)`), ce qui n'etait vrai que depuis le round 7 : sans `|| true` sur les quatre assignations d'extraction, errexit avortait le script AVANT le garde, exit 1 SANS aucune ligne — mesure faite dans les deux sens —, mutation-testee : 16e champ Rust seul → ROUGE ; ancre `pub struct HitlBinding` detruite → ROUGE NOMME ; **portee des NOMS** bornee a `[a-z0-9_]` cote Rust et `[a-zA-Z][a-zA-Z0-9]*` cote TS, donc un `pub r#type:` ou un `champ_snake:` reste invisible aux deux extractions et une derive REELLE resterait VERTE, comme cote BRIDGE), seule autorite possible puisque les controles front comparent le miroir a sa propre definition TS. **Portee exacte** : des NOMS, pas le tuple. La jambe est aveugle aux TYPES (les trois champs numeriques `max_cost`/`seq`/`expires_at_ms` sont DELIBEREMENT `string` cote TS), aux champs `pub(crate)` et a l'optionalite TS ; une derive de TYPE est attrapee par `binding_digest_matches_its_golden_vector`, pas ici. Digests en `break-all` et non tronques (un digest tronque n'est pas verifiable par l'humain) ; expiration en `<time dateTime>` avec REPLI sur la valeur brute — qui EST la preimage signee — pour une valeur HORS PLAGE (`i64::MAX` ; aucune semantique sentinelle n'existe cote Rust, cf. borne 22), dont le rendu jetait auparavant un `RangeError` et emportait toute la route via la frontiere d'erreur. Une decision ne vide pas la file : seule la carte decidee part (fixtures a 2 entrees des deux cotes). Focus porte par un element `role="status"` focalisable, car deplacer le focus dans le meme tour qu'une mise a jour de region `polite` fait couramment abandonner l'annonce — l'operateur AT n'entendait jamais le seul retour sur son acte d'autorite. Spec e2e `workflow-hub-a11y.spec.ts` (T1 BLOQUANT) + unit `Approvals.test.tsx` + axe-core sur la region HITL de `/approvals` (scan `include`-scope) avec un jeu de regles SENSIBLE AU CONTENU (contraste, `button-name`, `definition-list`/`dlitem`/`listitem`, `aria-allowed-role`, `aria-valid-attr-value`, `heading-order`) ; page-wide le scan remonte 123 violations `color-contrast` du chrome PARTAGE, dette pre-existante de toute route et propriete d'AI1 — le scope est mutation-teste (contraste casse DANS la carte => ROUGE) : le sous-ensemble FONDATION seul est document-level (lang, title, viewport) et resterait vert si tout le corps de la page etait remplace par un div vide. Borne : conformite WCAG 2.2 AA complete et panel AT reel = AI1. |

**Bornes honnetes (ce que L1 ne ferme PAS)** — la liste est longue A DESSEIN :
une primitive d'autorite non reliee doit dire ce qu'elle ne garantit pas.

1. **0 call-site de production** pour la totalite des primitives livrees (cf.
   DS-1 ci-dessus). L'enforcement HITL n'est PAS vivant ; A13 (§2) a raison de
   dire que la couche 7 vide ne refuse que les classes `Human`.
2. **Le temoin `HostVerifiedApproval` n'est pas une preuve crypto** : il ferme
   l'oubli, pas la fabrication d'evidence par l'hote lui-meme.
3. **Rollback du fichier entier du coffre** non detectable de l'interieur : il
   faut un compteur HORS-ARTEFACT. Le precedent maison le moins cher est un
   SIDECAR (`anchors.json` persiste la `revision` comme plancher anti-rollback,
   `iroh_runtime.rs`, test committe) — **0 dep, et TPM/keyring n'est PAS un
   prealable** : router cette borne vers du materiel fermait prematurement la
   conception. `delete` != wipe-secteur.
4. **OS-keychain LIVE** = decision supply-chain SEPAREE (le chemin Linux tire
   des deps neuves) ; la defense at-rest reelle est Argon2id(PIN).
5. **File d'approbation non persistante** (survit a la mort d'une iframe, pas a
   un rechargement du shell) ; la file durable du daemon (route loopback,
   inventaire D7) et le Centre des Runs plein = Phase V.
6. **Aucun point d'entree de navigation** vers `/approvals` n'est ajoute au
   shell en L1 : la page est atteignable par URL et par les tests. La rendre
   decouvrable appartient a la surface operateur (Phase V). Pire, et distinct :
   le chrome est ENTIEREMENT MASQUE pendant l'affichage d'une app
   (`AppShell.tsx`, plein ecran sur `/browse/:projectId`) — donc pendant le
   scenario meme que L1 modelise (une app qui demande un effet sensible), une
   approbation en attente n'a ni chemin ni notification.
7. **Le `0600` du blob est `#[cfg(unix)]`** : sur Windows le fichier herite de
   l'ACL du parent. La claim « atomique 0600 » vaut pour Unix.
8. **La consommation mono-usage du nonce n'est pas reliee** a l'approbation
   (cf. T-HITL-FORGE-REPLAY).
9. **Un type deserialise ne repasse pas par son constructeur validant** (serde
   reconstruit le champ). `SecretRef` a perdu son derive `Deserialize` en L1 (il
   n'avait aucun consommateur, donc il n'achetait qu'un contournement), mais
   `SealedEntry` en garde un — impact borne, c'est un ciphertext deja
   AEAD-authentifie. **Et la reliure ajoutera `Deserialize` a `HitlBinding`**,
   geste d'une ligne exige par le design : serde reconstruira alors ses 15 champs
   `pub` SANS les 11 `validate_identity_component` — le seul type ou cette classe
   touche l'AUTORITE, non plus un simple nom.
10. **Les 64 MiB de `Vec<Block>` d'argon2 0.5.3** ne sont essuyes par AUCUNE
   feature du crate (upstream n'invoque jamais `Block::zeroize` sur la memoire de
   travail) ; en params production (`p = 1`) le dernier bloc du Vec EST le
   `blockhash` final dont `blake2b_long` derive kek1 — un scrape de heap
   liberee / swap / crash-dump recalcule le KEK **en un blake2b, sans le PIN et
   sans payer le mur Argon2**. La feature `zeroize` activee en L1 essuie
   `initial_hash` et le `blockhash` final, ce qui est un vrai progres (ce code
   tourne deux fois par session), pas la fermeture du residu dominant.
11. **Le commit n'est pas entierement `fsync`e**, et les deux moities n'ont pas
   le meme statut. Le scratch l'est INCONDITIONNELLEMENT (`sync_all` avant le
   `rename`, echec propage). Le fsync du REPERTOIRE est TENTE en best-effort sur
   Unix et son echec est volontairement avale : le `rename` a deja commite, donc
   un `?` a cet endroit rendrait `Err` apres que l'ecriture est devenue visible
   et casserait l'invariant `Err ⇒ rien n'a ete commite` dont dependent les
   quatre appelants (`init` repondrait `AlreadyInitialized` a vie — la forme de
   brickage exacte que le nettoyage d'`init` existe pour empecher). Consequence
   mesurable : sur un montage ou `open(dir)`/`fsync(dirfd)` echoue (overlay, 9p,
   virtiofs), l'ecriture rend `Ok` sans entree de repertoire durable, **sans
   trace et sans test rouge**. La classe est donc **ATTENUEE, pas fermee** sur
   Unix (la redaction precedente disait « fermee ») ; sur Windows il n'y a pas
   d'equivalent portable pour un handle de repertoire et l'ordonnancement repose
   sur `MoveFileEx`. Fermer reellement exigerait de remonter l'echec par un
   canal DISTINCT du `Result` d'ecriture (`SecurityEvent`) — dette, pas L1.
12. **Le blob est lie aux params KDF COMPILES**, pas a ceux de son header : un
   bump de `KdfParams::production()` rend tous les coffres illisibles avec un
   `CryptoReject` qui dit « wrong PIN or tampered blob » et **accuse donc
   l'operateur**. Pre-launch, un bump = re-mint. Et **0 API de re-key, 0 `wipe`,
   0 journal d'usage** : `inventory()` est un INVENTAIRE `(ref, sealed_len)`,
   deliberement PAS nomme `audit` — « quel run a binde `stripe_live_key`, et
   quand ? » n'a aucune reponse en L1. C'est cette absence de remediation qui
   transforme les bornes 3, 11 et (3) de la row lifecycle en pertes
   IRREVERSIBLES plutot qu'en defauts de robustesse.
13. **Le chemin de decision humaine ACTUELLEMENT cable** (`orchestrator.rs` →
   `HUMAN_DECISION_CHANNEL` → `SignalDeposit`, 4 champs) ne porte **ni principal
   ni signature**. L1 livre la primitive signee A COTE de lui, pas dedans : la
   reliure doit le **REMPLACER, jamais l'ETENDRE**. Etendre est la route la plus
   courte et elle ressemble a un succes — le run reprend, et rien ne rougit,
   puisque les primitives signees gardent leur 0 call-site, etat que ce document
   decrit comme normal.
14. **La couche 7 est inerte des DEUX cotes.** Le cote DECISION est divulgue
   (borne 1) ; le cote BESOIN ne l'etait pas : aucun code de production ne
   construit une `BoundedCapability` ni ne pose `ApprovalLevel::Human` (le format
   de pack n'a aucun champ de classe ni d'approbation). La reliure doit livrer le
   **PLANCHER** d'approbation EN MEME TEMPS que le producteur de decisions,
   sinon la couche 7 ne gate jamais rien — un no-op silencieux qu'aucun test ne
   peut attraper, le drop fail-closed n'ayant jamais d'entree.
15. **Le canari couvre 5 rendus du moteur + le blob au repos.** Il ne parcourt
   PAS `runs.input_payload`, `snapshots.state`, `signals.payload`, l'env
   `SBFB_EFFECT_INPUT` ni la ligne side-output — vacueusement propres tant qu'il
   y a 0 call-site, mais EXISTANTS. Et l'invariant de A14 vaut pour la route
   SERIALISATION : la route COPIE (`expose()` puis recopie dans un `String`)
   reste une discipline de la reliure.
16. **`Debug` et `inventory()` divulguent la LONGUEUR** du materiel — oracle de
   format (51 o ⇒ `sk-…` OpenAI, 40 ⇒ secret AWS, 64 hex ⇒ cle HMAC) — et les
   NOMS des slots ainsi que leur NOMBRE sont en clair dans le blob.
17. **Le plafond `max_cost` est enregistre et lie, jamais APPLIQUE**
   (`decide_ref` ne recoit pas la `BoundedCapability`), et l'approbation est
   REJOUABLE N fois dans sa fenetre de fraicheur. `CapabilityClass` n'a par
   ailleurs ni `as_str`, ni `Display`, ni `Serialize` : son contrat de fait est
   la sortie de `derive(Debug)`, miroitee a la main en Rust et en TS, donc
   renommer une variante changerait tous les digests sans qu'un test rougisse.
18. **Le shell ne peut pas verifier que le tuple affiche est la preimage du
   digest signe** (aucun BLAKE3/JCS en TS, l'ajouter serait une dep interdite).
   D'ou le flux prescrit, et l'ordre compte : le chrome soumet le TUPLE + le
   verdict, le daemon RECALCULE le digest et signe CELUI-LA. La file ne porte
   d'ailleurs plus de digest pre-calcule, precisement pour rendre l'autre flux
   impossible a cabler par inadvertance.
19. **Les deux moities du seam portent des versions de contexte divergentes**
   (`…hitl-binding.v0` avec des points cote engine, `sbfb-hitl-approval-v1` avec
   des tirets cote hote) : deux chaines independantes, gelees separement. Un
   geste de maintenance « aligner en v1 » invaliderait silencieusement toute
   approbation stockee, avec un `BindingMismatch` indiagnosticable. Chaque cote
   porte desormais son vecteur dore : `binding_digest_matches_its_golden_vector`
   cote MOTEUR (le digest du tuple) et
   `approval_preimage_matches_its_golden_vector` cote HOTE, qui epingle le
   contexte, le separateur `0x1f` ET l'ordre des champs. Sans eux, mettre le
   contexte a `"x"`, changer le separateur ou retirer le `push(SEP)` laissait
   toutes les suites vertes, les deux fonctions du cote hote recalculant leur
   preimage avec le meme code. (Le KAT hex du coffre,
   `vault_kek_is_domain_separated_from_keystore`, epingle la derivation du KEK —
   il n'est PAS un cote de ce seam ; l'enumeration precedente le citait a tort.)
20. **Le seed de test de la file est un global `window` lu par le build de
   PRODUCTION** — borne ASSUMEE, pas fermee, et l'arbitrage merite d'etre ecrit
   exactement parce qu'une premiere version de cette borne l'a mal justifie.
   Fermer le seed derriere `import.meta.env.DEV` rend la fixture inerte dans le
   build que l'e2e sert (`--web-root dist`) : **mesure faite**, QUATRE des cinq
   controles HITL passent alors au ROUGE — les deux `toBeVisible()` sur une carte
   absente, la traversee clavier qui ne trouve plus le bouton, ET le scan axe,
   dont le garde `toHaveCount(2)` precede l'analyse depuis le meme lot de
   correctifs. Seule la verification « file vide » reste verte. (Une redaction
   precedente disait TROIS et comptait le scan axe parmi les verts : elle a ete
   falsifiee par son propre correctif frere. Le fait STABLE, a preferer au
   compte : tout controle qui depend du seed rougit.) Un gate rouge est donc le
   bon signal, PAS un gate aveugle : la toute premiere redaction de cette borne
   pretendait l'inverse et posait un faux dilemme.
   Le vrai arbitrage est « borne assumee CONTRE perte de la couverture e2e
   seedee », et il existe une troisieme option non retenue ici : un flag de build
   positionne pour le seul `dist` e2e (`vite build --mode e2e` + `define`),
   absent des releases — ecarte parce que l'e2e cesserait alors de tester
   l'artefact de production, propriete que ce depot tient explicitement (« un
   VRAI daemon, jamais un mock »).
   Ce que la borne coute, mesure : rien de plus que l'acces console dont elle
   depend. Qui peut ecrire ce global peut deja peindre la meme fausse carte dans
   le DOM ; le seed n'ajoute aucune CAPACITE, il raccourcit un geste deja
   disponible. Deux verrous voisins tiennent : un DOM-clobbering produit un
   `HTMLElement` que le `Array.isArray` du lecteur rejette, et l'iframe est en
   origine opaque (`sandbox="allow-scripts"` sans `allow-same-origin`, epingle en
   egalite EXACTE), donc ne peut pas ecrire dans le `window` de l'hote. Meme
   precedent assume que `__SBFB_AUTH_TOKEN`.
   Ce qui change a la reliure : la file viendra du daemon et une decision
   quittera le navigateur — le hook devient alors une porte d'entree d'AUTORITE
   et doit disparaitre DANS le commit qui cable la route.

21. **AUCUNE approbation ni aucun refus n'est PERSISTE** en L1 : `decide()` retire
   la demande de la file et affiche un message, sans store, sans journal, sans
   POST. L'axe **Repudiation** d'A15 — le label STRIDE de `T-HITL-FORGE-REPLAY`
   — n'a donc AUCUNE mitigation aujourd'hui : « qui a refuse quoi, et quand ? »
   est sans reponse. Le journal des decisions appartient a la reliure (le DS-1
   nomme deja « le refus d'autorite journalise » comme son livrable).
22. **La fenetre de fraicheur n'est pas BORNEE** : `HitlBinding::new` valide les
   11 composantes chaines et ignore les 3 numeriques, donc `expires_at_ms` peut
   valoir la borne du type. Compose avec la borne 17 (approbation REJOUABLE dans
   sa fenetre) : une fenetre non bornee et un rejeu illimite font une
   autorisation PERMANENTE. Fermer demande soit un plafond nomme verifie dans
   `new` (contre un `issued_at_ms` qui n'existe pas encore au tuple), soit la
   consommation mono-usage que la borne 8 declare non reliee.
23. **Rien n'autorise QUEL slot un run peut lier.** `SecretResolver::resolve` et
   `UnlockedVault::bind` ne recoivent ni principal, ni run, ni capacite, ni
   digest de pack : nommer un slot, c'est l'obtenir. L'asymetrie est visible
   dans le format de pack lui-meme, ou `requested_capabilities` porte « demander
   n'est pas obtenir : une autorite les intersecte ailleurs » tandis que
   `secret_refs` n'a pour garde qu'une heuristique anti-copier-coller. La
   reliure doit livrer l'autorisation en meme temps que le binding — et
   l'emplacement naturel existe : le seam est un OBJET par contexte
   (`&dyn SecretResolver`), donc scoper l'instance aux slots autorises du run
   est la forme object-capability de la reponse.
   **ARBITRAGE ECRIT (L1, pour que la reliure n'en herite pas un implicite)** :
   `capability_ref` et `secret_ref` sont **DEUX namespaces distincts**, et
   l'autorisation d'un slot passe par le **SCOPE DE L'INSTANCE** de resolveur,
   pas par un parametre d'autorite ajoute a `resolve`/`bind`. Trois raisons.
   (i) `CapabilityClass::SecretUse` gate le DROIT D'UTILISER des secrets — une
   classe — et non « quel slot » : confondre les deux namespaces ferait d'un nom
   de capacite un nom de slot, donc d'une intersection reussie un droit de
   lecture arbitraire. (ii) Le seam est deja un objet par contexte, et
   `ScriptedSecrets` en est litteralement la forme scopee : la reponse ne
   demande aucun changement de signature, donc aucun risque sur une preimage
   gelee. (iii) Un parametre d'autorite serait verifie PAR l'appelant qu'il est
   cense contraindre — le contournement le plus court, exactement la classe que
   `HostVerifiedApproval` a fermee cote HITL. Ce qui reste du a la reliure :
   construire le resolveur AVEC l'ensemble des slots autorises du run, et
   journaliser le refus.
24. **Le spoof d'AFFICHAGE purement ASCII reste ouvert.** Les caracteres de
   controle ET les caracteres invisibles/bidi sont refuses au domaine (predicat
   nomme, miroite aux deux jumeaux, teste des deux cotes) ET au RENDU des
   diagnostics (`sanitise_untrusted` partage le meme predicat — il gardait
   auparavant sur `char::is_control` seul, donc `U+202E`, qui est `Cf` et non
   `Cc`, traversait la moitie SORTIE de la classe que la moitie ENTREE fermait).
   Portee exacte du predicat : il est un **surensemble strict** de
   `nexus_coordinator_rs::output_filter::is_invisible_char` (PUA, bloc TAG
   `U+E0020..E007F`, `U+2060` — qui tombait dans le trou entre deux plages —,
   plus `U+00AD` et `U+2028/2029` que `is_control()` ne couvre pas), avec un
   temoin par plage des deux cotes. Ce module voisin traite les memes familles
   pour un but DIFFERENT (assainir une sortie LLM) et PRESERVE volontairement
   les bidi : les deux ensembles ne doivent PAS etre alignes, seul le
   confinement compte. Restent ouverts —
   `https://api.cloud-provider.example@evil.example/`, un homoglyphe de domaine
   ou un chemin trompeur passent toutes les validations et s'affichent tels
   quels. Aucune validation atteignable en L1 ne ferme cette classe : elle
   demande une normalisation d'URL et une politique de destinations, qui vivent
   avec la reliure.
   **Et le RENDU HTML n'est pas traite non plus** : le predicat ferme le
   DOMAINE (entree) et les DIAGNOSTICS (`sanitise_untrusted`), pas le troisieme
   lecteur — le `<dd>` du chrome hote, qui rend la valeur telle quelle. Deux
   consequences a nommer plutot qu'a laisser deviner. (i) Le garde TS
   (`isValidRequest`) ne verifie que `typeof === "string"` : `U+202E` et ses
   freres traversent la file seedee. Ce chemin est borne par la borne 20 (acces
   console = capacite deja disponible), et a la reliure les 11 composantes
   chaines passent par `HitlBinding::new`. (ii) Le sous-cas qui SURVIT a la
   porte d'entree est l'ESPACE ASCII : `U+0020` n'est ni un controle ni un
   format invisible, donc `principal: "admin "` et `principal: "admin"` donnent
   deux digests DIFFERENTS et s'affichent IDENTIQUEMENT (HTML replie les blancs).
   Fermer cela demande une neutralisation qui REVELE au rendu (echappement
   visible + marqueur de bord), pas un portage du predicat — lequel laisserait
   justement l'espace passer.
25. **La couche 2 du keystore (OS-keyring) est INERTE, et ce n'est pas qu'un
   trou de doc.** Mesure L1, recoupee trois fois : le lock de `keyring` ne
   depend que de `log` + `zeroize` (aucun backend natif), l'amont re-exporte
   `mock as default`, et rien dans le workspace n'appelle
   `set_default_credential_builder`. Le builder mock rend un credential VIERGE a
   chaque `Entry::new` et se declare `EntryOnly`. Consequence FONCTIONNELLE, pas
   seulement declarative : les constructeurs de production posent
   `use_keyring: true`, `init` ecrit `kek2` dans son entree puis la droppe, et
   `unlock` EXIGE cette entree — donc `unlock` echoue **des le meme process**,
   pas seulement apres un redemarrage. `kek2` etant 32 octets aleatoires, une
   identite creee par ce chemin serait irrecuperable. Le defaut est
   **PRE-EXISTANT (Sprint 20)** et **hors surface de changement de L1** (le diff
   ne touche aucune des lignes concernees) : L1 le MESURE et le declare **sur
   les sites enumeres ci-dessous**. L'enumeration EST le contrat ; elle n'est
   pas une preuve d'exhaustivite, et le round 7 a precisement montre qu'une
   formulation de completude (« partout ou la promesse etait ecrite ») se
   falsifie des qu'un site echappe au balayage. Un site trouve plus tard
   s'ajoute a la liste, il ne dement pas la borne. **TRAITES** : le doc-module
   de `keystore.rs`, `PATTERNS.md`
   §Sprint 20.1 (banniere) et §T26 (le rationale de bench, dont la conclusion
   « le scheme tient » est declaree VOIDE), `README_EN.md` (claim publique),
   `DURESS.md` (table §2.1, §wipe **et §1 Vue d'ensemble** — la promesse
   « tous les secrets daemon » du panic wipe, reduite a une enumeration exacte
   au round 7, qui nomme aussi le coffre NON reclame), `PATTERNS.md`
   §T-keystore-bench-reference, les commentaires mal-imputes de
   `tests/keystore_integration.rs` (en-tete de module, doc du probe ET le
   commentaire in-corps du cross-handle read — les TROIS ; le round 7 a montre
   que l'en-tete contredisait la correction portee 33 lignes plus bas dans le
   MEME commit), **`crates/nexus-launcher/src/unlock.rs`**
   (module + `run_init` — c'est le SEUL chemin de production, celui qui
   construit `LocalFileKeyStore::new` avec `use_keyring: true`, et il annoncait
   un wrap « optional » doublement faux : ni optionnel, ni protecteur),
   **`crates/nexus-core-rs/Cargo.toml`** (le rationale de dep),
   **`crates/nexus-shell-daemon/src/panic.rs`** (doc-module du panic wipe : il
   annoncait l'effacement des entrees keyring sans dire qu'il est
   security-neutral, et taisait les deux scratchs et le coffre non reclame)
   et le **`Cargo.toml` RACINE** (le rationale canonique de la dep `keyring`,
   vers lequel le crate renvoie explicitement — requalifie « DESIGN ONLY /
   INERT AT RUNTIME » en L1 round 7). **NON TRAITES et routes** : le CODE lui-meme,
   plus les doc-items de `LocalFileKeyStore` (`keystore.rs:466`, `:470-474`,
   qui ecrivent la promesse au present). Le ROUTAGE va vers `sprint83_audit_plan.md` — il n'est pas
   corrige ici, parce que les deux seules sorties reelles sont une feature
   backend native (borne 4, decision supply-chain) ou `use_keyring: false` par
   defaut, qui fait rougir `keystore_integration.rs` et demande donc un
   arbitrage, pas un correctif de round.

26. **Le scratch d'ecriture du keystore est desormais DURABLE avant le
   `rename`** (`sync_all`, ajoute par L1 pour fermer la fenetre « rename publie
   sans les octets »). Contrepartie a assumer : une mort de process dans cette
   fenetre laisse a cote du blob une copie COMPLETE et scellee de l'identite, la
   ou l'ecriture pre-L1 laissait au pire un fichier tronque. L1 ferme la moitie
   la plus grave — `wipe_all` reclame desormais les deux scratchs (reclamation
   **BEST-EFFORT a echec avale** : les deux unlinks sont en `let _ =`, un scratch
   verrouille par un antivirus ou une rotation en vol n'avorte donc PAS la purge,
   mais n'est pas non plus efface, et rien ne le signale), sans quoi une
   purge de duresse effacait les deux blobs nommes et laissait le materiel
   derriere elle, tout en PROUVANT par ce `.tmp` survivant qu'un effacement avait
   eu lieu (perte de deniabilite). Reste ouvert et route : le coffre nomme son
   scratch par ECRITURE (`{pid}.{alea}`), donc les orphelins s'y accumulent sans
   borne et rien ne les balaie — un balayage des freres a l'`unlock` risquerait
   d'unlink le scratch VIVANT d'un autre process, c'est un arbitrage de
   LIVENESS, pas un correctif de review.
27. **La fenetre relecture→`rename` du coffre n'est verrouillee pour PERSONNE, et
   sa collision degrade les QUATRE points d'entree.** C'est la borne 2 du doc-module
   de `crates/nexus-core-rs/src/vault.rs` et l'item (3) de la row
   `T-SECRET-VAULT-LIFECYCLE` ci-dessus ; elle recoit ici un ORDINAL parce que trois
   artefacts rank-1 la citaient par un numero qui designait autre chose (review r8
   P1-1 : « bound 2 of §19.4 » resolvait vers le temoin `HostVerifiedApproval`).
   Mecanisme : deux writers concurrents passent tous deux la garde d'epoch, calculent
   le MEME `next` et le dernier `rename` gagne. Apres la collision, les DEUX handles
   portent l'epoch DU DISQUE — donc `refuse_if_stale` PASSE pour chacun alors que l'un
   tient un BODY perime. Consequence par point d'entree, toutes mesurees en lisant la
   trace : `bind` sert une entree que l'autre handle a revoquee ; `delete` rend
   **`Ok(false)`**, qui signifie alors seulement « ma copie ne l'avait pas » et non
   « le set commis ne la contient pas » — un fail-OPEN qui rapporte un SUCCES a un
   geste de revocation ; `rotate` rend `UnknownRef` pour la meme raison ; `put`
   COMMIT le body perime, ce qui defait silencieusement l'ecriture de l'autre. La
   cecite dure jusqu'au prochain commit par l'un ou l'autre handle — ou jusqu'a un
   `reload()`, qui realigne le handle IMMEDIATEMENT et sans commit (correction
   Codex round 2 : la phrase precedente ne nommait que le commit). Ce n'est PAS une
   classe adversariale : deux process legitimes du meme utilisateur suffisent (daemon
   + CLI). Fermeture : un verrou couvrant relecture→`rename` (verrou de repertoire
   consultatif, ou sentinelle `create_new`) — de la DETTE, comme le fsync de
   repertoire de la borne 11, pas un correctif de round. Les textes qui decrivent la
   garde de fraicheur doivent la qualifier par CETTE borne. Enumeration NORMATIVE des
   sites qui la portent : `vault.rs` rustdocs de `bind`, `rotate`, `delete`, **de
   `persist`** (c'est la que la fenetre NAIT) et **du test
   `delete_and_rotate_refuse_a_stale_handle_instead_of_answering_from_the_snapshot`**
   (qui prouve le refus pour un handle EN RETARD, pas apres la collision),
   `docs/workflow/llms.txt` et `.planning/active/sprint83_gate_matrix.md` row 10.
   Un site qui decrit la garde sans citer cette borne est un defaut, pas un oubli
   benin : c'est ainsi qu'une promesse absolue a survecu deux rounds.
   **Ce n'est PAS le seul etat ou un handle sert une entree revoquee** (Codex
   round 2) : la garde d'epoch et la lecture du body ne sont pas atomiques, donc un
   `bind` ADMIS a l'epoch `n` peut se terminer APRES qu'un autre handle a commis un
   `delete` a `n+1`, sans aucune collision. Cette execution reste LINEARISABLE — le
   bind est admis au controle d'epoch, avant la revocation — c'est donc une
   SEMANTIQUE a enoncer, pas un defaut a corriger : **un bind admis avant une
   revocation peut se terminer apres elle.** Et une TROISIEME, au niveau
   SYSTEME : meme termine AVANT la revocation, le `VaultSecret` deja rendu est
   consomme par le fournisseur APRES elle — meme classe, une couche plus haut.
   C'est pourquoi une revocation dure demande un BAIL tenu jusqu'au point
   d'usage du fournisseur, et pas seulement un verrou interne au coffre ; une
   seconde relecture retrecirait la premiere fenetre sans fermer aucune des
   deux autres.


Palier T2 WA-T1.8/WA-T1.11/WA-T4 (approuver dans le chrome hote ; secret canari
absent partout) : evidence = tests canari + a11y ; `NOT-RUN` jusqu'a AI2. Le plan
gele cite « WA-T1.6 », qui est en realite « atteindre `waiting_human` » (Phase K)
— l'item L1 est WA-T1.8, et le canari WA-T1.11.

## 20. Revue et evolution

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
- **v19 (Sprint 82 Phase T, 2026-07-17)** : cloture S82 — sweep des refs
  VIVANTES « re-route S82 » (12 sites : rows STRIDE T/R :266-267, table
  hardening row I, §16 N1/N2/N3/SI-5, caveat SI-12 TOCTOU) : S82 etant le
  sprint dette docs-contrat (D6), ces items n'ont PAS ete livres par S82 —
  chaque ref vivante devient « re-route **slot rig-chaud** (S82/D6) »
  (per-worker RunProofs, transport sketch control-plane, re-exec GPU
  in-vivo, arbitrage litige, SI-5 padding, SI-12 TOCTOU). Les entrees
  HISTORIQUES v12-v18 restent verbatim (archives datees, meme doctrine
  que le sweep S78 de v17). 0 row STRIDE nouvelle, 0 frontiere nouvelle,
  0 changement de posture — re-libelle de routage uniquement. La cloture
  docs-contrat S82 (index request-bodies + verrou D7 LOOPBACK) vit dans
  les docs concernes, pas ici.
- **v20 (Sprint 83 Phase B, 2026-07-18)** : ajout **§17 Surface profil
  d'accessibilite** (chantier Workflow Agents), renommage §17 « Revue et
  evolution »→§18. Nouvel **Asset A9** `needs[]` = PII sante/handicap
  (LINDDUN, sensibilite superieure a A4) ajoute a §2/§4/§6 (regle §18.3).
  4 rows §17 : T-A11Y-NEEDS-LEAK (fermee — projection needs-free PAR
  CONSTRUCTION : le type de sortie n'a AUCUN champ `needs` cote Rust
  [champ retire, pas un `#[serde(skip)]` — pas de fuite `Debug`/logger] et
  cote TS ; `applyResolvedAxes` sans `data-needs` ; preuve autoritaire =
  unit test web `applyResolvedAxes` all-needs ; oracle de parite partage
  Rust↔TS `resolution_fixtures` ; gate `check-a11y-needs-free.sh`
  garde DOM best-effort + non-fail-open + CABLE verify.sh/ci.yml ; e2e = smoke),
  T-A11Y-PROJECTION-SMUGGLE (fermee — rejet strict aux deux bouts : Zod
  `.strict()` module partage + re-validation sender + Hub `a11y.js`
  rejette toute cle hors-axes et n'accepte que le parent), T-A11Y-OPERATOR-DOM
  (INFO — l'Operator ecrit `data-needs` a son DOM local first-party, sous sa
  propre CSP hors bac a sable, hors scope du gate), T-A11Y-LINKABILITY (**residuel L accepte** — correlation des
  axes-effet cote app, mitigee `connect-src 'none'` + effet≠cause, non
  fermee : hors surface fondation Phase B, appartient aux surfaces d'autorat
  + AI1). Codex S83-B (4 P1/6 P2/2 P3) reconcilie : needs retire du type,
  canal host→app cable (sender + receiver strict), gate durci+cable, e2e
  smoke honnete + tags WCAG cumulatifs, fixtures de parite, golden non
  fail-open. Wire : projection hote→app LOCALE non-signee → 0 `DOMAIN_*_V1`,
  census DOMAIN inchange par cette surface (25 a l'epoque S83-B ; 29 depuis
  S83-F), frontiere gatee par son golden
  `accessibility_profile.schema.json`. 0 bump wire, 2 deps DEV neuves
  (`axe-core` + `@axe-core/playwright`, hors archive, CSP inchangee). DoD
  Phase B = **fondation, pas conformite** (AM-8) : la conformite WCAG 2.2 AA
  / ATAG / COGA reelle appartient aux surfaces d'autorat + au panel AI1.
- **v21 (Sprint 83 Phase C, 2026-07-19)** : **autorite du bridge** rendue
  AUTORITAIRE per-call (ordre canon C→D→E→F : aucun verbe Workflow privilegie
  avant). §5.2 row I corrigee (**3 → 16 methodes** + intersection host-side
  `caller ∩ methode-declaree-par-release ∩ grant ∩ politique ∩ consentement` —
  cable ACTIVEMENT `declare ∩ grant` ; politique + consentement INERTES v0 ;
  l'ancre docs-contrat reelle etait `:186`, pas `:179` [ligne vide]). 5 rows
  §5.2 neuves : T-BRIDGE-USURP (fail-closed per-call au choke-point `useBridge`
  AVANT tout dispatch, `pii_redact` offline inclus ; modele canonique
  `sbfb_manifest::grant_v0`, miroir `web/src/bridge/authority.ts`, parite gatee
  `check-frontier-contracts.sh` bloc 5 [self-test de non-vacuite committe]),
  T-BRIDGE-GRANT-EXPIRY (expiry re-evalue PAR APPEL), T-BRIDGE-LEGACY-DEFAULT
  (**grandfather** des apps sans clef `bridge` = residu accepte pre-launch,
  bascule post-v1.0), T-BRIDGE-REVOKE-INFLIGHT (revocation in-flight routee
  L/V/AI, hors scope Phase C), T-BRIDGE-RESOLVE-FAILOPEN (une app DECLAREE
  grandfathered pendant la resolution async / sur echec fetch `SBFB.json` =
  residu accepte NON-escaladant).
  §17 T-A11Y-PROJECTION-SMUGGLE : leg **getter-TOCTOU FERME** (`pushEvent`
  clone-once-scan-send du meme snapshot), residu value/nom-encoding **ACCEPTE**
  (hote de confiance). Regime : `bridge` present ⇒ fail-closed sur le set declare ;
  absent ⇒ grandfather. **1 correction de declaration** : les declarations
  existantes deviennent AUTORITAIRES — 6/7 exemples declarent un `bridge`, seul
  sbfb-explorer (sans clef) est grandfathered ; `workflow-hub` = focus, declare
  `[]`. L'audit adversarial declare-⊇-appele **incluant les chemins SDK
  transitifs** a trouve `sbfb-ideas` sous-declarant `storage_version` (appele
  transitivement par `onStorageUpdate`→`getStorageVersion`, poll live-sync 3s,
  erreurs avalees) : la methode a ete ajoutee a son `SBFB.json` (live-sync
  restauree). Les 5 autres apps declarantes sont verifiees exhaustivement
  (compute-tester `[task_submit,task_result]` + factory-viewer couvrent leurs
  appels ; daisyui/velocity/workflow-hub declarent `[]` et n'invoquent aucun
  verbe). Apres correction, declare ⊇ appele est tenu ; le gate CI
  declare-⊇-appele-transitif reste un carry (non construit ce sprint). Wire :
  grant v0 =
  etat host-side ⇒ 0 `DOMAIN_*_V1`, census DOMAIN inchange par cette surface
  (25 a l'epoque S83-C ; 29 depuis S83-F), 0 bump wire
  (`FEED_FORMAT_VERSION`/`*_ANNOUNCEMENT_VERSION`=1), gate `SENSITIVE_ACTIONS`
  substring NON etendu. 0 dep runtime neuve.
- **v22 (Sprint 83 Phase H, 2026-07-22)** : ajout **§18 Surface compilateur
  de plan** (Asset A10 « le graphe signe EST le plan » — acquittement du
  carry P2 route « H, sinon AH » au freeze §8 : H SEME l'asset, la
  confrontation adversariale complete reste Phase AH), renommage §18→§19.
  T-PLAN-LLM-MUTATION ferme (moitie STATIQUE : compile() pur sans canal
  sortie-de-modele + 9 familles §19.3 couvertes par `compiled_plan_digest`,
  test discriminant) ; T-PLAN-STATIC-BYPASS ferme (20 classes de refus, un
  test par classe) ; T-PLAN-CLASSIFICATION-LAUNDER ferme (regle 6 freeze §8
  par analyse data-flow, gates `human.review`/`wait.signal`, quality-gate
  hors-scope) ; T-PLAN-DIGEST-MALLEABILITY ferme (PresentationKeyAsData +
  tri canonique des edges + un-digest-par-schema-id + determinisme
  dual-profil) ; T-PLAN-SCAN-EVASION residuel L accepte (scan best-effort,
  la garantie = le format). `CompiledPlan.v1` = artefact LOCAL derive non
  signe : 0 `DOMAIN_*_V1` produit, `DOMAIN_COMPILED_PLAN_V1` reste RESERVE
  0-producteur, census 29 INCHANGE, 0 bump wire, 0 dep runtime neuve.
- **v23 (Sprint 83 Phase I, 2026-07-22)** : ajout **§19 Surface run store
  durable** (Asset A11 — journal + outbox + inbox + snapshots + lease,
  fichier SQLite local jamais signe/gossipe), renommage §19→§20. Rows A10
  et A11 versees a la table §2 Assets (A10 etait defini §18 sans ligne —
  reconciliation). T-RUN-DOWNGRADE-MISREAD ferme (garde DOUBLE schema
  `user_version` a l'open + event `format_version` via l'unique porte
  `upcast_payload`, chaine par-hop `ChainGap` LOUD, mutation-testee) ;
  T-RUN-DOUBLE-EMIT-STALE-WRITER ferme mode leased (epoch CAS monotone
  dans CHAQUE write-txn, expiry = heuristique de takeover jamais condition
  de commit, bras out-of-process owner mort reel ; borne honnete : mode
  unfenced documente, le daemon acquiert en I1/J) ; T-RUN-RESTORE-ROLLBACK
  residuel M accepte opere (validation backup avant remplacement, siblings
  WAL purges, geste operateur-local en I) ; T-RUN-GC-EVIDENCE-LOSS borne
  par politique (jamais running/pending/diagnostic terminal, per-run
  atomique `InsideGcTransaction`) ; T-RUN-SNAPSHOT-POISON ferme (cache
  jamais-verite, digest BLAKE3 + version, red control `unverified`) ;
  T-RUN-SQLI-DESERIAL ferme (doctrine D3.6 : parametre partout,
  donnees-pures, upcasters compiles jamais stockes) ; T-RUN-ID-PII residuel
  M documente (identifiants caller-chosen, server-side en I1).
  `RunEvent.v1` REMPLI cote core (registre 6/7, `contracts.schema.json`
  rafraichi) ; `DOMAIN_RUN_EVENT_V1` reste RESERVE 0-producteur, census 29
  INCHANGE, 0 bump wire, 0 dep runtime neuve (dep intra-workspace
  engine→core seule). Re-coupe B6 : routes daemon + D7 + golden_http +
  clôture docs-contrat = Phase I1.
- **v28 (Sprint 83 Phase L1, 2026-07-24)** : ajout **§19.4 Surface coffre
  de secrets et HITL hote signe** (livre le PRODUCTEUR de la couche 7 laissee
  vide en L ; elle reste VIDE en production, cf. DS-1)
  + assets **A14** (coffre) et **A15** (approbation HITL signee). 6 rows
  neuves — statuts recopies VERBATIM de la table, ce changelog etant ce qu'un
  auditeur grep en premier : T-SECRET-REF-EXFIL **fermee par TYPE (serialisation) / bornee (reliure)** (`BoundSecret`/
  `VaultSecret` non-`Serialize` + `Debug` redacteur + zeroize ; canari
  `canary_absent_everywhere` + `vault_bind_rotate_zeroize` avec controles
  positifs de non-vacuite, recherche sous 6 encodages dont base64/hex/
  JSON-echappe), T-SECRET-VAULT-LIFECYCLE **fermee (falsification PARTIELLE au
  repos) / bornee (rollback du fichier entier)**
  (AES-256-GCM par entree, nonce frais, AAD lie header+ref, downgrade/
  migration de slot fail l'open ; **entree-verificateur `VAULT_CHECK_REF` +
  epoch monotone** — PIN prouve sur coffre VIDE, rejeu de ct et delete
  silencieux DETECTES, handle perime refuse en LECTURE comme en ECRITURE,
  scratch unique par ECRITURE, `sync_all` avant `rename` ; KEK `SecretBox`
  zeroize ; bornes : delete = retrait du set scelle et non wipe-secteur,
  rollback du FICHIER ENTIER non detectable de l'interieur — SIDECAR
  `anchors.json` suffirait, TPM non requis —, fenetre relecture→`rename` non
  verrouillee), T-HITL-APPROVAL-TAMPER **bornee (primitive gravee, reliure DS-1)**
  (digest injectif du tuple 15-champs — `capability_ref` et `decision` inclus —,
  `mutation_invalidates_approval` un mutateur par champ, exhaustivite tenue par
  DEUX controles de portees distinctes — le COMPILATEUR (destructuration sans
  `..` ⇒ champ AJOUTE) et un compte explicite (⇒ mutateur RETIRE) — avec un
  **residu ASSUME** qu'aucun des deux n'attrape : un champ ajoute puis silencie
  par `champ: _,` ; ensemble d'autorite TYPE :
  `decided_refs` prive, peuplable seulement de `DecidedRef` issus de
  `decide_ref`), T-HITL-FORGE-REPLAY **fermee (forgerie de signature) / bornee
  (replay + reliure) / non mitigee (repudiation, cf. borne 21)** (Ed25519 hote
  context-locale, forgeur sans cle rejete, **classe small-order refusee
  nommement** [`is_weak()` + `verify_strict`, durcissement LOCAL],
  nonce+seq+expiry dans le digest ; le mono-usage reste celui de
  `T-RUN-SIGNAL-FORGE-REPLAY` v26 et **rien ne relie encore les deux**),
  T-HITL-QUEUE-CONFUSED-DEPUTY **bornee (incapacite structurelle prouvee ; file
  non persistante ; 0 reliure)** (rendu+signature EXCLUSIVEMENT hote,
  iframe incapable : **allowlist bridge FERMEE de 16 verbes** epinglee par une
  **LISTE LITTERALE** de 16 + `ALL_DISPATCH_METHODS.size === 16` dans `authority.test.ts` (l'assertion NOMMEE voisine est un fil-piege de NOMMAGE, pas la fermeture) + absence d'`allow-same-origin` —
  pas la CSP, qui scelle l'origine/le reseau/les frames et non l'autorite ;
  file host-owned ; borne : cablage a la
  vraie file daemon = V), T-HITL-COERCION-A11Y **bornee (spec e2e BLOQUANTE)**
  (focus gere sur le PORTEUR du message, pas
  de keyboard-trap/type-to-confirm/delai ; **14 champs de la DEMANDE** rendus,
  ordre DERIVE des libelles, parite Rust↔TS gatee par la jambe `HITL-PARITY` ;
  e2e T1 BLOQUANT + unit + axe SENSIBLE AU CONTENU, SCOPÉ à la région HITL — page-wide, 123 violations `color-contrast` du chrome partagé, dette pré-existante de toute route).
  Le coffre REUTILISE `DOMAIN_KEYSTORE_V1` (sous-contexte `secret-vault`, pas
  un `DOMAIN_*_V1` neuf) ; la signature HITL est un context LOCAL
  (`sbfb-hitl-approval-v1`, jamais gossipe) : **census DOMAIN 29 INCHANGE,
  0 bump wire, 0 dep runtime neuve, 0 noeud de lock ajoute** — mais **DEUX
  aretes** : `sbfb-workflow-engine → zeroize` et `argon2 → zeroize`, cette
  seconde etant un **flip de feature** du KDF partage keystore+coffre (il essuie
  desormais `initial_hash` et le `blockhash` final, PAS les 64 MiB de
  `Vec<Block>`), `zeroize` etant deja declare au workspace.
  Split « ce qui rend / ce qui fait autorite » : shell (0 cle) rend, daemon
  (cle identite) signe, engine keyless calcule le binding. **45 tests Rust
  neufs** (secret 6, hitl 11, vault 20, hitl_approval 6, keystore 2 — `wipe_all_reclaims_the_scratch_files_too` et `an_unremovable_scratch_does_not_abort_the_wipe`) + front. Recompte a la source a chaque round, jamais recopie. Palier T2
  **WA-T1.8/WA-T1.11**/WA-T4 (le plan gele cite WA-T1.6, qui est « atteindre
  `waiting_human` », Phase K)
  `NOT-RUN` jusqu'a AI2. Ouvre+ferme G-SECRET-VAULT/G-SECRET/G-HITL, precede N.
  **Review r1 = FAIL (8 P1) : les 8 sont fermes AVANT le commit**, et ce qu'ils
  disent du process vaut d'etre grave. (a) Coffre — l'entree-verificateur +
  l'epoch monotone ferment d'un coup « n'importe quel PIN ouvre un coffre vide »
  (perte de secret sur une simple typo), « la sonde de PIN dechiffre un vrai
  credential dans un `Vec<u8>` non zeroise », « rejeu d'une ancienne ct ⇒ la
  rotation n'est pas durable » et « un handle perime ressuscite un credential
  revoque » ; les 4 tests correspondants etaient ROUGES avant le correctif.
  (b) Evidence — le canari ne pouvait PAS rougir (il construisait ses propres
  surfaces sans jamais y router le materiel, et son « controle positif » ne
  testait que `str::contains`) : il a desormais un bras ROUGE sur la surface de
  prod et cherche sous 6 encodages. (c) Phrase — cette section AFFIRMAIT « le
  daemon signe » et « fermee » sur **0 call-site** ; les statuts sont
  retrogrades en **bornee (primitive gravee, reliure DS-1)**, vocabulaire deja
  forge par la Phase L dans ce meme fichier. (d) Tuple — le verdict
  accorde/refuse manquait (research §14.4) et `assert_eq!(len, 13)` GELAIT
  l'ecart : le tuple porte 15 champs (+`capability_ref`, +`decision`) et
  l'exhaustivite est desormais tenue par le COMPILATEUR. (e) Front — une
  approbation de demonstration etait servie en PRODUCTION avec deux libelles
  affirmant un acte inexistant ; file vide par defaut, fixtures injectees par
  les tests, libelles rendus verifiables.
  **Review r2 = FAIL (9 P1) : fermes AVANT le commit**, et leur classe commune
  merite d'etre nommee — un correctif qui s'arrete au bord de sa propre section.
  (f) Le coffre etait ferme en ECRITURE, ouvert en LECTURE : `bind` servait le
  snapshot de l'unlock, donc un credential revoque par un AUTRE handle partait
  encore vers le fournisseur, sans aucun adversaire (garde de fraicheur +
  `reload()` + test rouge-avant-vert). (g) Le temoin d'autorite verrouillait
  l'ENTREE de `decide_ref` alors que la SORTIE — `HitlValidity{decided_refs}` —
  etait publiquement constructible : peupler l'ensemble depuis la file PENDANTE
  etait plus court que l'auto-validation deja fermee (newtype `DecidedRef`,
  champ prive). (h) `verify_hitl_approval` routait sur un `verify` NON-STRICT,
  sous le seul statut « fermee (forgerie) » restant, alors que le depot prouvait
  la classe dans le meme sprint (durcissement local + test point-identite avec
  l'assertion de sanite du patron). (i) La retrogradation r1 n'avait ete
  appliquee qu'au TABLEAU : le registre d'assets et ce changelog portaient encore
  « 13 champs », « fermee », « CSP scellee », et ce bullet **s'auto-contredisait**
  (il annoncait la retrogradation tout en gardant les anciens statuts). (j) Le
  correctif du gate bridge avait produit un test e2e qui ne pouvait JAMAIS
  rougir — 0 assertion executee — et attribue l'enforcement a un gate de PARITE,
  vert sur un ajout en lockstep. (k) Le verdict `granted` etait pre-affiche sur
  une demande PENDANTE offrant « Refuser », avec deux semantiques de refus
  mutuellement exclusives dans un seul commit (front vs moteur) : la demande
  pendante porte desormais 14 champs et le refus est signe des deux cotes.
  Le reste : borne d'epoch materiellement fausse apres son propre correctif
  (le scratch est unique par ECRITURE, l'epoch n'arbitre rien), 2 rustdoc de
  primitive affirmant « by construction » ce que le bras rouge du meme fichier
  refute, et une claim `Cargo.toml` nommant precisement le buffer qu'upstream
  n'essuie sous aucune feature. **Verification croisee** : les 4 mesures d'etat
  post-correctifs ont trouve 6 divergences de statut ABSENTES de r2, toutes
  CREEES par les correctifs eux-memes — dont une rustdoc « volee » (`DecidedRef`
  insere entre `HitlValidity` et son bloc de doc, laissant le type porteur de la
  couche 7 sans documentation).
- **v27 (Sprint 83 Phase L, 2026-07-24)** : ajout **§19.3 Surface
  permissions et attenuation** (`effective_capabilities` LIVRE comme
  computation autoritaire — G-CAPABILITY ferme : intersection monotone
  des 7 couches, 17 classes typees, property tests) + asset **A13**.
  6 rows neuves : T-CAP-WIDEN-BYPASS fermee (constructeur unique, aucun
  widen public, meet par champ direction-par-champ, agregation multi-grants
  = domination-globale-ou-drop permutation-invariante [P1 Codex r1+r2],
  `intersection_only_attenuates` 2000 seeds), T-CAP-CLASS-UNKNOWN-FAILOPEN
  **bornee** (17 classes ensemble ferme + garde partielle de drift `ALL` ;
  autorisation par TYPE jamais substring ; frontiere de resolution
  String→`BoundedCapability` = reliure DS-1), T-CAP-
  GRANT-AUDIENCE-CONFUSION fermee (grants audience-scopes release/pack,
  `live_for` inerte hors audience), T-CAP-EXPIRY-TOCTOU-RETRY **bornee**
  (contrainte gravee recompute par-attempt `now_ms` watermarke — reliure
  DS-1), T-CAP-REPLAY-REAUTHORIZE **bornee** (refus journalise, replay lit
  le journal jamais la policy live — reliure DS-1), T-CAP-DEPTH-
  AMPLIFICATION fermee (chaine monotone `depth_enforced` ; delegation depth
  §14.5 hors-MVP, garde app.call = Y). Flips §18 : T-PLAN-LLM-MUTATION
  « intersection 7 couches = Phase L » → **LIVREE §19.3** ; T-PLAN-
  CLASSIFICATION-LAUNDER requalifiee PARTIELLE HONNETE (classes typees
  engine-side LIVREES, borne STATIQUE compilateur reste nominale,
  raffinement runtime = defense-en-profondeur). Miroir DISCIPLINE de
  `grant_v0` (pas d'appel de fonction : mauvais domaine 16 verbes bridge, la
  parite d'algebre TS n'est pas machine-gatee). **DS-1** : L = module
  autoritaire PUR ; la reliure au dispatch + le journal des refus = phase
  run-surface. **21 mutations discriminantes tuees** (approval max, closure
  bypass, Ceiling min/refines, HITL gate, live_for audience, refines direction
  + `EffectiveCapabilities::refines`, ordre HITL avant grants, couche 2 no-op,
  union data_policy, agregation multi-grants left-fold/always-first/comparable
  [P1 Codex r1+r2], EffectNeed no-op, decision HITL par-ref, reorder HITL entre
  release/pack) + garde PARTIELLE de drift `ALL` (match sans wildcard) ; **P1 Codex r1+r2
  fermes** (LUB multi-grants → domination-globale-ou-drop permutation-invariante).
  0 route, `T1 = N-A`, 0 serialisation, census DOMAIN 29 INCHANGE, 0 bump
  wire, 0 dep runtime neuve (module pur intra-crate). Palier T2 WA-T5.2
  `NOT-RUN` jusqu'a AI2.
- **v26 (Sprint 83 Phase K, 2026-07-23)** : ajout **§19.2 Surface
  effets** (couche effets LIVREE — G-EFFECT/G-UNKNOWN fermes,
  G-FS-PROCESS-EFFECT ouvert et verifie K) + asset **A12**. 7 rows
  neuves : T-RUN-RECEIPT-FORGE-REPLAY **bornee** (anti-rejeu par contenu :
  reconcile exige un receipt structure parsable + binding tuple, effect_id
  engine-derive run-scope ; journal digest-only ; PAS de verif digest a la
  lecture [A11] ni signature [Phase O]), T-RUN-FS-EFFECT-ESCAPE fermee
  (confinement canonicalise chemin ET feuille reguliere-seule + CAS d'etat
  worktree LIE au dedup_scope durable), T-RUN-PROCESS-SPAWN-INJECTION
  fermee (programme/args/ENV fixes constructeur, `env_clear()` + map hote
  injectee — le crate ne lit aucune env ambiante, charte respectee),
  T-RUN-DESCENDANT-SURVIVAL **fermee Windows / bornee Unix** (Job Object
  kill-on-close + CREATE_SUSPENDED→assign→resume ; process-group +
  pdeathsig + reap-au-resume ; bornes Unix explicites setsid/fenetre
  spawn-persist/pgid-reutilisable ; temoin heartbeat anti-vacuite),
  T-RUN-RECONCILE-LAUNDER fermee (reconciliation evidence-only STRICTE —
  receipt parsable+lie obligatoire, origine fermee, terminal absorbant,
  park opt-in hote), T-RUN-RETRY-STORM fermee (budget dur du plan +
  backoff borne pacing-only + re-arm crash idempotent), T-RUN-EFFECT-
  TIMEOUT-TOCTOU fermee (deadlines absolues, lecture unique — pattern J
  suivi). CARRIES K TRAITES dans les rows existantes : T-RUN-DOUBLE-EMIT
  clause 2 **mecanisme fencing target-side DEMONTRE** (epoch presente,
  red control ; borne : cible reference non-atomique, cible de PROD doit
  fournir un check-and-apply atomique) ; T-RUN-CANCEL-DOS routage
  late-result LIVRE (drain du cancel, reap orphelin, borne « OBSERVE
  Ready ») ; T-RUN-REPLAY-DIVERGENCE validation causale a la reduction
  (CausalOrder — completion/attempt/park sans intent MATCHANT l'effect_id,
  reconciliation sans park matchant) ; T-RUN-BACKPRESSURE cohorte de
  join FIGEE (`join_fired{sources}`) + borne decision_label ALIGNEE
  compilateur (`MAX_DECISION_LABEL_BYTES`). RE-ROUTES explicites (0
  defer silencieux) : routes mutantes restore/GC/lease-force D7 +
  re-ouverture de wait + authz porte signal → **V** (Centre des Runs).
  `RunEventKind` etendu de 4 kinds effets (`effect_attempt_failed`,
  `node_outcome_unknown`, `reconciliation_recorded`, `join_fired`) +
  `TerminalReason::NodeFailed` + `ReconcileOrigin`/`ReconcileResolution`
  = REDEFINITION pre-launch de v1 (chaine upcast vide invariante,
  goldens E/J byte-identiques) ; migration store M4 forward-only
  (`attempts`, restore-validation/PK/FK co-migrees) ; GC purge desormais
  waits/signals/timers/attempts (FK latente J fermee, test dedie).
  0 route neuve (rows D7 re-routees, pas implementees), 0 bump wire,
  census DOMAIN 29 INCHANGE (receipt = context-string local, doctrine
  `hash.rs`), 0 nouveau noeud lock (`windows`/`libc` target-gated sur
  noeuds existants — precedent rusqlite).
- **v25 (Sprint 83 Phase J, 2026-07-23)** : ajout **§19.1 Surface
  orchestrateur deterministe** (reducer/scheduler LIVRES — G-AUTHORITY
  ferme avec E : replay deterministe + mort du writer sans mort du run).
  6 rows neuves : T-RUN-SIGNAL-FORGE-REPLAY fermee (nonce engine-derive
  mono-usage, refus au depot, consommation atomique CAS, expiration/labels
  fermes — 4 tests adversariaux) ; T-RUN-CANCEL-DOS bornee (CancelOrigin
  ferme, autorite operateur non-opt-outable, §12.7 stop-emission +
  resultat tardif conserve non propage via `record_late_completion`,
  terminal `cancelled` absorbant) ; T-RUN-TIMER-SKEW fermee (deadlines
  ABSOLUES premiere-gagne, firing CAS at-most-once, regression LOUD) ;
  T-RUN-REPLAY-DIVERGENCE fermee (reducer pur, decisions journalisees
  avant action, admission canonique cap-independante, divergence
  fail-closed per-node, identites d'iteration `~2N` disjointes par
  construction) ; T-RUN-SUBWORKFLOW-AMPLIFICATION fermee (bornes
  STRUCTURELLES du plan signe consommees + re-verifiees a la reduction,
  expansion statique seule) ; T-RUN-BACKPRESSURE-STARVATION bornee
  (cap deterministe pacing-only ; equite inter-runs = service multi-runs
  V/AI). Bornes requalifiees LIVREES : §18 « reducer/scheduler/cancel
  honnete = J » → §19.1 ; §19 « le chemin daemon ecrivain (J) acquiert »
  → l'Orchestrator ACQUIERT le lease avant tout write (v24 corrigeait la
  claim v23 dans un sens, v25 la ferme dans l'autre) ; « GC sans source
  d'horloge avant J » → trait `Clock` livre (la ROUTE mutante reste K).
  `RunEventKind` etendu de 5 kinds de DECISION (`branch_taken`,
  `map_fanout`, `signal_consumed`, `human_decision_recorded`,
  `cancel_requested`) + `TerminalReason::Cancelled` + `CancelOrigin` =
  REDEFINITION pre-launch de v1 (constante inchangee, chaine upcast
  vide invariante, golden famille `run_event_orchestrated` committe) ;
  migration store M3 forward-only (waits/signals/timers +
  `runs.input_payload`, restore-validation et PK/FK co-migrees).
  7 mutations actives jouees (M1 join-dead-path, M2' CAS wait, M3'
  branche cancel, M4 divergence, M5 composite deadline, M6 borne map,
  M7 ordre index) toutes TUEES ; M2 (flag depot) et M3 (gel ready
  reducer) requalifiees defense-en-profondeur, documentees in-code.
  0 route neuve, 0 bump wire, census 29 INCHANGE,
  `DOMAIN_COMPILED_PLAN_V1`/`DOMAIN_RUN_EVENT_V1` restent RESERVES
  0-producteur (plan consomme IN-MEMORY), 0 dep runtime neuve.
- **v24 (Sprint 83 Phase I1, 2026-07-22)** : concretisation de la borne
  I1 de §19 — surface loopback du run store LIVREE en **lecture seule** :
  `GET /api/daemon/workflow/run/{run_id}` (agregat scalaire allowlist :
  status/terminal_reason/event_count/frontier_node/spec_digest — jamais un
  `RunEvent` brut, ni snapshot, ni receipt) + `GET /api/daemon/workflow/
  lease` (row lease brute, verdict de liveness laisse a l'appelant), dans
  `authed_routes` (bearer + Host + Origin + peer-creds listener), inventaire
  D7 LOOPBACK §3 (+1 row T0 representative, trigger re-fired) + famille
  `golden_http_workflow_domain`. La famille MUTANTE restore/GC/lease-force
  est tier-classee **T1 dans l'inventaire avec implementation DATEE J/K**
  (precedent row-futur canary/cosign) : T1 CONFIRM_PROMPT design-only (une
  route mutante shiperait a T0 effectif), `restore_from_backup` exige un
  store CLOSED (incompatible handle boot-tenu ; garde sharing-violation
  Windows-only), GC sans source d'horloge avant J, lease-force sans
  appelant legitime (vol de writer). T-RUN-ID-PII : validation route
  livree (charset borne `[A-Za-z0-9._-]{1,128}` → 400 ; run_id ne forme
  jamais un path), residuel M engine-level conserve. Correction v23 : le
  chemin daemon I1 n'ACQUIERT PAS le lease (zero write daemon ; reads WAL
  non-fences surs) — l'acquisition est le chemin ECRIVAIN du runtime J.
  Routes NON duress-gatees (reads locaux ; le no-op duress couvre les
  operations sortantes/signantes — gater un read serait un signal
  distinctif). 0 bump wire, census 29 INCHANGE, engine/core BYTE-INTACTS,
  +1 ligne lock (dep intra-workspace daemon→engine seule).
