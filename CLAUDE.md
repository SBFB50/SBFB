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
│   └── archive/v1.1/                  # S14-15 (verified deploy, bridge bidirectionnel, watchdog)
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

## Etat actuel (2026-04-14, master tip post-Sprint 17 wrap-up)
- **Sprints 0-17 CLOSED**. v1.2 en cours.
- **430 Rust** / 183 SDK / 187+3 skipped coordinator / 46 app-gov
  / 239 Vitest / 38 Playwright / 7/7 size-limit / 246+ SPDX
  (~1128 tests total) — tous verts, inchanges post-S17 (sprint
  recherche pure, 0 code)
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

## Commandes clés
```bash
# Rust workspace
cargo test --workspace --locked
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
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
