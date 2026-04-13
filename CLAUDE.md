# nexus-grid / SBFB

## Projet
Plateforme P2P universelle de compute et d'hebergement d'apps.
"Decentralized P2P compute network for apps. No central server.
No admin. Just protocol." N'importe qui publie une app (React,
Python/Pyodide, WASM, HTML pur, notebook, etc.) sous forme
d'archive web. Le reseau la distribue. Les clients la rendent
dans un iframe sandboxe via le daemon Rust blob-serve.

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
├── .planning/                         # sprint{N}_{kickoff,plan,verification,audit_plan,audit_findings}.md
├── docs/
│   ├── claude/README.md               # WORKFLOW SOURCE OF TRUTH (lire d'abord)
│   ├── rust/PATTERNS.md               # patterns Rust + tech debt tracking
│   └── shell/PATTERNS.md              # patterns shell/coordinator + T1..T7 tech debt
└── examples/hello-world-app/
```

## Etat actuel (2026-04-13, master tip `08853ff`)
- **Sprints 0-13 CLOSED**. v1.0.0 released.
- **369 Rust** / 183 SDK / 99+1 coordinator / 46 app-gov
  / 191 Vitest / 30 Playwright / 7/7 size-limit / 220 SPDX
  (~908 tests total) — tous verts
- Sprint 12 a livre le rendu universel cross-node (archive zip
  → daemon blob-serve → iframe sandboxee)
- Sprint 13 a livre le bridge postMessage (iframe ↔ coordinator),
  open source enforcement (public = repo_url obligatoire), UI
  Netflix glassmorphism, launcher Rust minimal
- **Sprint 14 EN ATTENTE** : audit gate Sprint 13 a jouer en
  Phase 0, puis solidification plateforme (CPU watchdog, runtime
  templates, re-publish auto, verification repo_url, bridge push)

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
- **Sprint 13** : public = open source (repo_url obligatoire),
  postMessage bridge = seul canal iframe ↔ reseau (3 methodes
  whitelist), launcher Rust minimal (pas Tauri, browser = client),
  UI Netflix glassmorphism dark-first

## Principe de conception — sessions fraiches
**Ne jamais propager les scope cuts des sprints precedents comme
des verites techniques.** A chaque nouveau sprint, verifier dans
le code actuel si un item "differe" est toujours un vrai gap.
Lancer un agent Explore avant de declarer quelque chose "trop
gros". Penser produit/plateforme d'abord, implementation ensuite.

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
