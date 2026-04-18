# nexus-grid / SBFB

## Projet
Plateforme P2P universelle de compute et d'hebergement d'apps.
"Decentralized P2P compute network for apps. No central server.
No admin. Just protocol." N'importe qui publie une app (React,
Python/Pyodide, WASM, HTML pur, notebook, etc.) sous forme
d'archive web. Le reseau la distribue. Les clients la rendent
dans un iframe sandboxe via le daemon Rust blob-serve.

**App store open source par construction** : chaque app publique
est deployee depuis un repo Git verifie. Les utilisateurs
peuvent en 1 clic voir le code source, signaler un bug, proposer
une feature, contribuer via PR, ou forker l'app et deployer leur
propre version sur le reseau. Le modele F-Droid/Linux applique
aux apps web P2P.

Pivot 2026-04-10 depuis l'ancien NEXUS cold-case (toujours present
sous `nexus/` comme future app mais plus le projet principal).

## Source de vérité pour le workflow Claude
**Avant toute action, lire `docs/claude/README.md`.** Ce document
capture le système de travail multi-sprint qu'on utilise : cycle
kickoff → plan → code → verification → audit_plan, audit gate
pattern entre sprints, commit discipline atomique, memory system
externe, anti-patterns. Une session fraîche sans cette lecture
ré-invente les règles et produit du code hors-convention.

## Modele de rendu — plateforme universelle (Sprint 12+)
Chaque projet publie une **archive web** (zip avec index.html).
Le reseau distribue l'archive via iroh-blobs. Les clients la
rendent via le **daemon Rust blob-serve** (`GET /blob-serve/{hash}/{path}`)
qui decompresse le zip (crate `zip`), cache les fichiers en LRU memoire,
et les sert dans un **iframe sandbox** (`sandbox="allow-scripts"` sans
`allow-same-origin`, CSP `connect-src 'none'` pour contenu untrusted). Toute techno qui produit du HTML est supportee
(React, Vue, Python/Pyodide, WASM, Jupyter/JupyterLite, HTML pur).
Le shell React est un client parmi d'autres — un futur client
mobile ou Electron utiliserait le meme chemin blob → iframe.
Les apps TabView SDK existantes sont pre-rendues en HTML statique
par le coordinator au moment du publish.
Les apps dans les iframes communiquent avec le reseau via un
**bridge postMessage** (Sprint 13) : 3 methodes whitelist
(`task_submit`, `storage_get`, `storage_set`), correlation IDs,
validation source iframe. Le SDK client `sbfb-bridge.js` est
inclus par les apps.

## Securite apps publiques — deploy verifie (Sprint 14)
Les apps publiques sont deployees **depuis le repo source** par
le coordinateur lui-meme. Le publisher ne fournit pas de zip —
il donne une URL de repo + commit, et le coordinateur :
1. Clone le repo (`git clone --depth 1`, max 500 MB, timeout 30s)
2. Verifie `SBFB.json` a la racine (node_id Ed25519 matche le
   daemon local — pattern Keyoxide, preuve de propriete)
3. Verifie que le repo est public (API forge retourne 200)
4. Verifie `index.html` existe
5. Zip le contenu (exclut `.git/`, valide les chemins)
6. Genere `provenance.json` signe (SLSA L1 : repo_url, commit_sha,
   artifact_hash BLAKE3, node_id, timestamp, signature Ed25519)
7. Deploie le zip sur iroh-blobs

Garantie : le code sur le reseau = le code du repo. Multi-forge
(GitHub, GitLab, Codeberg, Gitea). Zero OAuth, zero token.
Verification offline par n'importe quel noeud via la signature.
Cf. `sprint14_keyoxide_decision.md` (memory) pour le detail.

## Architecture Option G (hybride Rust + Python)
- **Rust workspace** (`crates/`) : `nexus-core-rs` (iroh 0.97
  wrapper), `nexus-core-py` (PyO3 bindings), `nexus-worker-core`
  (headless engine lib) + `nexus-worker` (binary),
  `nexus-shell-daemon-core` + `nexus-shell-daemon` (Sprint 7 —
  P2P discovery + curator pipeline + pkarr browse),
  `nexus-launcher` (Sprint 13 — spawn daemon + open browser)
- **Python workspace** (`packages/`) : `nexus-coordinator`
  (FastAPI + dispatcher + kudos ledger + TabView pre-render),
  `nexus-sdk` (NexusApp ABC + TabView), `nexus-app-gov` /
  `-coldcase` / `-forensics` (apps officielles)
- **Frontend** (`web/`) : React + Vite + TypeScript + Tailwind
  + shadcn/ui + Zustand + React Query.
  Pages : Browse, Curators, Network, OnboardingEmpty,
  ProjectDetail, Projects. Le shell est un **iframe host** pour
  les apps distantes — il ne connait pas la techno de l'app.
- **iroh stack** pinne : iroh 0.97 / iroh-docs 0.97 / iroh-gossip
  0.97 / iroh-blobs 0.99

## Stack
- Windows 11, RTX 5080 16GB VRAM
- Rust 1.94 (rustup / cargo), maturin 1.13
- Python 3.13 (uv workspace `.venv/` + miniconda base pour
  installation wheels via `maturin develop --release`)
- Node.js (frontend React dans `web/`)
- Ollama (worker-side LLM runtime)

## Structure des crates / packages
```
nexus-grid/
├── Cargo.toml                         # workspace Rust
├── crates/
│   ├── nexus-core-rs/                 # iroh wrapper (docs, gossip, blobs,
│   │                                  # discovery, curator crypto, canonical bytes JCS)
│   ├── nexus-core-py/                 # PyO3 bindings (sign/verify task/result/claim/curator)
│   ├── nexus-worker-core/             # engine lib headless (state machine,
│   │                                  # allowlist SQLite, GPU monitor, Ollama client)
│   ├── nexus-worker/                  # worker binary (CLI + TUI + state writer)
│   ├── nexus-shell-daemon-core/       # P2P discovery lib (curator runtime,
│   │                                  # browse aggregator, registry singleton)
│   ├── nexus-shell-daemon/            # shell daemon binary (HTTP + gossip subscribe)
│   └── nexus-launcher/                # minimal launcher (spawn daemon + open browser)
├── packages/
│   ├── nexus-coordinator/             # FastAPI coord + dispatcher + kudos + /daemon proxy
│   ├── nexus-sdk/                     # NexusApp ABC + TabView + decorators
│   ├── nexus-app-gov/                 # gov app (WIP, 19 tabs migration Sprint 8)
│   ├── nexus-app-coldcase/            # port de l'ancien NEXUS en tant qu'app
│   └── nexus-app-forensics/           # BPA + acoustique + traces (legacy forensics)
├── web/                               # shell React (Browse, Curators, Network, etc.)
├── .planning/                         # sprints (active/ + archive/v{X}/ + roadmaps + research)
│   ├── active/                        # sprint en cours uniquement (kickoff, plan, audit_findings du precedent, verification, audit_plan)
│   ├── archive/v1.0/                  # S0-13 (pivot, P2P, universal render, bridge, launcher)
│   ├── archive/v1.1/                  # S14-15 (verified deploy, bridge bidirectionnel, watchdog)
│   └── archive/v1.2/                  # S16-20 (loopback hardening, research, supply chain, transport hardening, Gate 2 prerequisites)
├── docs/
│   ├── claude/README.md               # WORKFLOW SOURCE OF TRUTH (lire d'abord)
│   ├── rust/PATTERNS.md               # patterns Rust + tech debt tracking
│   └── shell/PATTERNS.md              # patterns shell/coordinator + T1..T7 tech debt
└── examples/hello-world-app/
```

## Securite loopback + GPU consent (Sprint 16)
Depuis le Sprint 16, la couche **loopback HTTP** est durcie par
defense en profondeur : bearer token 256-bit X-SBFB-Token (genere
par le launcher, perm 0600) + Host allowlist `{localhost,
127.0.0.1, [::1]}` + Origin check (mitigation CVE-2025-49596 DNS
rebinding). Sur Unix, un second listener UDS avec SO_PEERCRED
rejette les uid != geteuid(). Sur Windows, un Named Pipe avec
DACL custom via SDDL `D:(A;;GA;;;<current-user-SID>)` bloque les
autres utilisateurs. Le `PeerCredsVerified` marker est un type
prive Rust non-spoofable. L'exception unique est `/health`.

Le worker enforce un **consent GPU** explicite (4 niveaux : mes
projets / open source verifies / whitelist manuelle / tous) avec
caps W/VRAM/heures par jour via
`crates/nexus-worker-core::consent::should_accept_task`. La
config `~/.sbfb/consent.json` est re-lue live via un `notify`
watcher (50 ms debounce), le daily counter `usage.json` reset a
minuit-local (chrono::Local).

ProjectAnnouncement bumpe en **v5** avec `is_open_source` derive
automatiquement par le coordinator (true pour deploy-from-repo,
false pour zip prive), non-user-settable. Le decoder reste
tolerant (v4 legacy default false).

Threat model STRIDE + LINDDUN complet dans
[`docs/security/THREAT_MODEL.md`](docs/security/THREAT_MODEL.md).
Roadmap runtime isolation (WSL2 / Virtualization.framework /
systemd-nspawn) pour Sprint 17+ dans
[`docs/security/RUNTIME_ISOLATION.md`](docs/security/RUNTIME_ISOLATION.md).

## Etat actuel (2026-04-18, master tip post-S20 Phase F wrap-up)
- **Sprints 0-20 CLOSED**. Audit gate S19 leve via 2 commits
  `1af90b3..3a7f0a3` (0 P0 + 0 P1 + 9 P2 + 2 P3 resolus inline,
  verdict PASS). Sprint 20 livre les **6 big rocks Gate 2
  prerequis** (encryption at rest keypair + duress PIN + panic
  wipe + PoW runtime wire + structured output dual-backend +
  warrant canary federation foundations + dual-transport WSS
  observability). **Premier pivot G8 effectif** sur Phase E :
  skill `nexus-phase-preflight` introduit commit `59225ee` a
  detecte un conflit threat-model vs commit S18 E2 `04c9621`
  (auto-publish scheduler rejete pour raison cle Ed25519
  accessible) et declenche pivot Option C deep-evolution
  federation foundations arbitre user 2026-04-18 (plan mis a
  jour avant code via `bd16e64`). v1.2 en cours.
- **642 Rust** / 185 SDK / 213+3 skipped coordinator / 46 app-gov
  / 241 Vitest / 38 Playwright / 7/7 size-limit / 246+ SPDX
  (~1371 tests total) — tous verts. Delta S20 : **+111** (+104
  Rust encryption+duress+wipe+PoW wire+structured output+FROST,
  +5 coord canary_registry, +2 Vitest PanicWipe).
- Sprint 12 a livre le rendu universel cross-node (archive zip
  → daemon blob-serve → iframe sandboxee)
- Sprint 13 a livre le bridge postMessage (iframe ↔ coordinator),
  open source enforcement (public = repo_url obligatoire), UI
  Netflix glassmorphism, launcher Rust minimal
- Sprint 14 a livre le deploy verifie (Keyoxide + SLSA L1
  provenance + ProjectAnnouncement v4 + badge "Verifie")
- Sprint 15 a livre le bridge push bidirectionnel + CPU watchdog
  par heartbeat + CLI `sbfb init` (html/react/pyodide) + tests
  Playwright iframe reels
- Sprint 16 a livre le loopback hardening complet (Phase A
  `d7c265a` bearer + Host + Origin, Phase B `1cfde89` UDS/NP peer
  creds, Phase C `3247e88` consent 4 niveaux + caps worker-side,
  Phase D `10bbc63` is_open_source, Phase E docs security
  + roadmap VM isolation). Audit gate CONDITIONAL PASS leve via
  7 commits `0230589`..`d18e19e`.
- **Sprint 17 CLOSED** (sprint **recherche pure**, 0 code, ~4823
  LOC docs security) : livre Phase A `297fd50` adversary taxonomy
  T0-T5 + 12 attack scenarios, Phase B `c275ebd` P2P attack
  surface (Sybil/Eclipse/gossip/DHT/BGP/traffic/ISP), Phase C
  `7dea299` GPU compute threats (prompt leak/spoof/theft/extract/
  inject/side-channel/DoS), Phase D `872f48a` hardening roadmap
  (matrix 27 threats + Sprint 18-30 sequencee + gates 1-4),
  `721686c` VALIDATED_BLUEPRINT (13 couches long-terme, briques
  OSS validees 2026 contre docs/advisories/CVE via WebSearch +
  context7 MCP). Phase E scope-cut (RELEASE_GATES + PARTNERSHIPS
  + DISCLOSURE) officialise — couvert partiellement par
  BLUEPRINT, items ONG-facing restants reportes sprint OpSec
  dedie futur. 3 zones rouges documentees :
  - R-iroh-audit P0 : iroh 0.97 sans audit public + sans SECURITY.md
  - R-wasmtime-cve P0 : 12 CVE avril 2026 dont 2 Critical,
    pinning 43.0.1+ ou LTS 36.0.7+ obligatoire
  - R-libcrux-hax P2 : Symbolic Software 7 avril 2026 demontre
    5 semantic gaps pipeline hax->F*, ML-KEM prod via `aws-lc-rs`
    FIPS 140-3 plutot que libcrux
- Audit gate S17 = Sprint 18 Phase 0 via
  `.planning/archive/v1.2/sprint17_audit_plan.md` (tracks A-G).
- **Sprint 18 CLOSED** (quick wins + supply chain + multi-relai +
  canary + Codeberg mirror, ~1460 LOC code + ~350 tests + ~440 docs,
  +44 Rust tests). 8 commits + 1 chore-close : Phase A supply-chain
  CI (cargo-deny + pip-audit + npm audit + wasmtime pin D2), Phase
  B `4ab0211` reproducible builds + SLSA in-toto, Phase C `9d0ad7a`
  multi-relai federation (n0 + 2 fallbacks) + DHT pkarr 3-quorum-2/3,
  Phase D `94cccb2` coord-side wire TaskEntry + X-SBFB-Token rotation,
  Phase E1 `9f4d19f` NVIDIA driver CVE check launcher, Phase E2
  `04c9621` warrant canary monthly Ed25519 signe (gossip + CANARY.txt
  + verify-canary.sh), Phase E3 `95807b1` Codeberg private
  disaster-recovery mirror (pivot depuis Radicle : repo GitHub prive
  pre-launch, Radicle P2P public-only incompatible, differe au v1.0
  go-live — doc flip sequence self-contained MIRROR_FALLBACK.md §3).
  Gate 1 effectivement debloque (supply chain + repro + multi-relai
  + wire + driver check + canary + mirror = DnD Forge beta fermee
  deployable). Audit gate S18 = Sprint 19 Phase 0 via
  `.planning/archive/v1.2/sprint18_audit_plan.md` (tracks A-F +
  meta-track Radicle-v1.0 activation tracking). Audit findings S18
  livres verdict CONDITIONAL PASS (1 P1 + 5 P2 + 6 P3), tous leves
  via 6 commits : `677556f` D-1 wire TokenRotator + file-watcher
  tokens.json ; `0fb8458` F-1+F-2 docs hygiene (phase_E1_review
  presence + file-count 10 + tip placeholders) ; `9661485` A-1
  drop `--workspace` cargo-deny job (deprecated v0.14+) ; `6fe2dce`
  B-1 wheel SLSA attestation in release matrix ; `e223ec7` C-1
  DHT quorum primitive-only clarification (runtime wire S19+) ;
  `1a606a3` P3 batch (buildType URI + parse_version warn +
  RADICLE casing). Carry-over S19 : C-1 wire `redundant_resolve`
  au browse aggregator (primitive prete) + Meta-1 Radicle-v1.0
  activation tracking.
- **Sprint 19 CLOSED** (transport hardening : PoW Hashcash gossip +
  TLS pinning relays + delayed upload queue + pkarr self-hosted +
  carry DHT wire). Phases : A `ab6985c` DHT quorum runtime wire
  (carry S18 C-1 — `PkarrQuorumResolver` + wiring browse aggregator
  + curator runtime, flip S18 verification `[~]→[x]`), B `edfc51b`
  + `08f4e41` PoW Hashcash primitive + gossip subscribe integration
  (difficulty 2^18 default + per-relai policy `relay_pow_policy.toml`),
  C `540bb51` TLS cert pinning relays (`tls_pinning.rs` SPKI hash
  extract + `PinValidator` + pinset bootstrap), D `f238d31` delayed
  upload queue (async queue + scheduler 30s flush + jitter 0-5min
  anti-correlation), E `2fd4d72` pkarr relay self-hosted docker
  image (`docker/pkarr-relay/Dockerfile` + `build-pkarr-image.yml`
  + `docs/release/PKARR_RELAY_OPS.md` §1-§7 self-contained), F
  wrap-up. Audit gate S19 leve via 2 commits `1af90b3..3a7f0a3`
  session fraiche 2026-04-16 (verdict PASS, 0 P0 + 0 P1 + 9 P2 + 2
  P3 resolus inline). Carry-over S20 : Meta-1 Radicle-v1.0
  activation tracking.
- **Sprint 20 CLOSED** (Gate 2 prerequis : encryption at rest
  keypair + duress PIN + panic wipe + PoW runtime wire + structured
  output dual-backend + warrant canary federation foundations +
  WSS fallback observability). Phases : A `05271fa` encryption at
  rest double layer (Argon2id m=64 MiB/t=3/p=1 + AES-256-GCM
  `aes-gcm 0.10` + OS keyring wrap `keyring-rs 3.6`, blob v1
  `~/.sbfb/keyring/identity.enc`, bench 82 ms RTX 5080 T26
  calibration Pi 4 TBD, deviation NASM Windows build T25 FIPS
  path), B `c32ecb3` duress PIN fake-keypair noop (`IdentityMode::
  Duress` + `noop_identity` helpers gossip publish fake / curator
  subscribe noop / task dispatch reject, indistinguabilite wire
  blobs 96 bytes identical) + panic wipe 5-tap `Ctrl+Shift+Alt+W`
  x5 3s → `POST /panic/wipe` loopback auth → zeroize RAM +
  secure-unlink + delete OS keyring + `ExitStrategy::exit(0)`,
  C `16b94ba` PoW runtime wire (subscribe_with_pow au runtime pour
  curator + browse + task dispatch gossip, `pow_policy_loader.rs`
  hot-reload pattern TokenRotator S18, audit P2-C-SEC-1 leve
  in-phase via `2e045f1`), D `c85397b` structured output dual-
  backend `LlmBackend` trait (Ollama `format` JSON schema +
  llama.cpp llguidance 1.7 matcher state machine `ff_tokens` +
  `consume_token` post-selection, logit-bias wire S21 carry,
  `TaskResponse` struct + `task_response.schema.json` draft-07 +
  `schemars` schema generation, validation finale
  `validate_task_response` garde-fou, follow-up `7ea68a6` audit
  P2-1 commentaire honnete llama_cpp sampler + PATTERNS §P30 note
  Sprint 20 etat), E `6a3f199` pivot G8 Option C federation
  foundations (`CanarySigner` trait + `Ed25519CanarySigner` impl
  baseline + `FrostCanarySigner` K-of-N RFC 9591 jan 2025 +
  `CanaryRegistry` coord-side observational-only + duress ack
  channel `nexus-grid/canary-duress-ack/v1` CLI `sbfb canary ack`
  + `AttestationProvider` trait + `NoopAttestation` prep TEE S25-30
  + `transport_probe.rs` UDP QUIC probe 3x 10s → WSS TCP 443 warn
  log observability-only — S1 finding preflight absorbe `relay_wss_
  only` n'existe pas client-side iroh 0.97), F wrap-up
  (`<Phase F>`). Pivot G8 retrospective : premier declenchement
  effectif du skill `nexus-phase-preflight` (commit `59225ee`
  codification) sur Phase E, verdict DESIGN-CONFLICT suite scan S2
  historical decisions (rejet threat-model S18 E2 `04c9621`
  auto-publish scheduler) → `sprint20_phase_E_pivot_proposal.md`
  → arbitrage user Option C deep-evolution 2026-04-18 → plan mis
  a jour avant code via `bd16e64`. Aucune signature canary
  automatisee — CLI manuel uniquement, clef Ed25519 jamais
  exposee scheduler. `CanarySigned v1` wire format preserved
  (FROST sigs Ed25519 RFC 8032 byte-identique verifiable via
  verifier standard). Carries G7 2/2 respectes : Meta-1 Radicle-
  v1.0 re-carry S21 + P2-2 gitignore NOISE inline open S20
  `1b1f9cb`. Delta tests S20 : **+111** (+104 Rust / +5 coord /
  +2 Vitest). Audit gate S20 = Sprint 21 Phase 0 via
  `.planning/archive/v1.2/sprint20_audit_plan.md` (tracks A-F +
  meta-track Radicle-v1.0 re-carry S21). Chore tooling G4 hors-
  sprint inclus dans le range : `59225ee` + `b634c23` + `e2e8595`
  workflow G8 introduction + robustness follow-up + bootstrap
  §7.1 reference, `b6da3a4` skill hook process cleanup, `3c18908`
  narrate-action mutex, `b7d8d74` narrate-action.lock gitignore.
  Pre-launch protocol policy respectee : `BLOB_VERSION = 0x01`,
  `TASK_RESPONSE_VERSION = 1`, `CANARY_VERSION = 1` inchanges,
  aucun tolerant decoder multi-version introduit. Pas de nouvelle
  zone rouge — R-wasmtime-cve / R-iroh-audit / R-libcrux-hax /
  R-pyodide-escape inchangees.

## Commandes clés
```bash
# Rust workspace — iteration pendant phase (rapide, cible le crate modifie)
cargo nextest run -p <crate-touche> --locked

# Rust workspace — verification finale avant commit phase (complet)
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc   # doctests only (nextest ne gere pas)
cargo build -p nexus-shell-daemon --release

# Python (trois packages tournés séparément — collision de nom tests/)
uv run ruff format --check packages/ && uv run ruff check packages/
uv run pytest packages/nexus-sdk/tests/ -q
uv run pytest packages/nexus-coordinator/tests/ -q
uv run pytest packages/nexus-app-gov/tests/ -q

# Wheel PyO3 dans .venv uv (attention au conflit CONDA_PREFIX)
unset CONDA_PREFIX CONDA_DEFAULT_ENV && \
  VIRTUAL_ENV=$PWD/.venv maturin develop --release \
    --manifest-path crates/nexus-core-py/Cargo.toml

# Frontend
cd web && npm install && npm run lint && \
  npx tsc --noEmit -p tsconfig.app.json && \
  npm run test:unit && npm run test:coverage && \
  npm run build && npm run size && \
  npx playwright test && bash scripts/scan-en-strings.sh
```

## Decisions architecturales gelees
Cf. `nexus_grid_pivot.md` (memory) §« Decisions actees (a ne PAS
re-debattre) » — 12 items originaux + extensions Sprint 12/13 :
- Pivot P2P integral, Option G hybride Rust+Python
- iroh 0.97 pinne, visibilite 2 etats public/prive
- Zero moderation centrale, curator lists Ed25519+gossip+blobs
- Kudos per-project, HTTP loopback via coordinator proxy
- Singleton strict shell daemon, AGPL-3.0 maintenue
- **Sprint 12** : archive zip = format universel de publication,
  daemon blob-serve = rendu universel (origin separee port 7000,
  CSP `connect-src 'none'`), le shell est un iframe host
  agnostique (ne connait pas la techno de l'app)
- **Sprint 13** : public = open source (repo_url, sera remplace
  par preuve crypto Keyoxide en Sprint 14),
  postMessage bridge = seul canal iframe ↔ reseau (3 methodes
  whitelist), launcher Rust minimal (pas Tauri, browser = client),
  UI Netflix glassmorphism dark-first
- **Sprint 14 (decision)** : deploy verifie from source —
  le coordinateur clone le repo, verifie SBFB.json (Keyoxide
  Ed25519), build le zip lui-meme, signe provenance.json
  (SLSA L1). Le code sur le reseau = le code du repo. L'ancien
  `POST /project/deploy` (upload zip) reste pour le prive.
  Securite clone : depth 1, 500 MB max, 30s timeout, pas de
  .git/, pas de submodules, validation paths, MIME scan.

## Principe de conception — sessions fraiches
**Ne jamais propager les scope cuts des sprints precedents comme
des verites techniques.** A chaque nouveau sprint, verifier dans
le code actuel si un item "differe" est toujours un vrai gap.
Lancer un agent Explore avant de declarer quelque chose "trop
gros". Penser produit/plateforme d'abord, implementation ensuite.

## Pre-launch protocol policy
**Le projet n'a pas encore de deploiement live** : aucun noeud
tiers ne parle les protocoles SBFB en prod, aucun cache iroh-docs
externe ne contient d'historique, aucune installation user n'est
en dehors de la machine dev. Consequence sur les wire formats
(`Task`, `ProjectAnnouncement`, `CuratorList`, etc.) :

- **`*_FORMAT_VERSION` / `*_ANNOUNCEMENT_VERSION` restent a 1**
  jusqu'au premier tag `v1.0`. Un sprint qui change le canonical
  ne bump PAS la version — il redefinit la v1 courante.
- **Pas de tolerant decoder multi-version** (`v == 1` seul, pas
  `v1..v5`). Pas de rationale "decode un legacy JSON qui n'a pas
  ce champ" — le seul legacy est le master d'il y a 2 commits,
  c'est du refactor, pas de la compat.
- **`#[serde(default)]` reste legitime** pour la **robustesse
  runtime** (un client Python qui envoie un JSON minimal a l'API
  daemon → les champs omis deserializent a zero/false plutot que
  422 parse error). Ecrire le rationale dans la doc du champ pour
  eviter la confusion "runtime tolerance vs historical compat".
- **Tests "legacy decode"** qui simulent une version anterieure
  du format = **a supprimer immediatement** apres la redefinition.
  Ce sont des zombies qui protegent un scenario inexistant.

Apres le tag `v1.0`, la politique bascule : chaque break bump la
version, chaque decoder accepte un range, chaque ajout de champ
carry un `#[serde(default)]` assume pour la compat ascendante.
Jusque-la, on edite le canonical librement.

## Discipline de travail
**Tout est dans `docs/claude/README.md`.** Résumé ultra-court :
- un sprint = kickoff + plan + 4-6 phases A-F + verification +
  audit_plan, tous dans `.planning/`
- un commit par phase (`feat(scope): Sprint N Phase X — titre`),
  body riche avec delta de tests cumulé et scope cuts respectés
- pas de band-aid fix — toujours root cause
- pas d'emoji sauf demande explicite
- Day 0 decisions figées, scope cuts stricts
- memory system externe : lire MEMORY.md + nexus_grid_pivot.md
  + sprint_audit_gate.md + feedback_approach.md au démarrage de
  chaque session fraîche

## Langue
Francophone. Réponses utilisateur, docs planning, commit bodies,
commentaires dans `docs/claude/` → **français**. Code, identifiants,
commit titles, logs, strings d'erreur → **anglais**. `PATTERNS.md`
est majoritairement anglais (consommé par l'agent et futurs
contributeurs externes). Le script `web/scripts/scan-en-strings.sh`
garde le code React côté utilisateur en français.
