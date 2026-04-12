# nexus-grid / SBFB

## Projet
Réseau P2P de compute LLM distribué. "Decentralized P2P compute
network for LLM apps. No central server. No admin. Just protocol."
Pivot 2026-04-10 depuis l'ancien NEXUS cold-case (toujours présent
sous `nexus/` comme future app mais plus le projet principal).

## Source de vérité pour le workflow Claude
**Avant toute action, lire `docs/claude/README.md`.** Ce document
capture le système de travail multi-sprint qu'on utilise : cycle
kickoff → plan → code → verification → audit_plan, audit gate
pattern entre sprints, commit discipline atomique, memory system
externe, anti-patterns. Une session fraîche sans cette lecture
ré-invente les règles et produit du code hors-convention.

## Architecture Option G (hybride Rust + Python)
- **Rust workspace** (`crates/`) : `nexus-core-rs` (iroh 0.97
  wrapper), `nexus-core-py` (PyO3 bindings), `nexus-worker-core`
  (headless engine lib) + `nexus-worker` (binary),
  `nexus-shell-daemon-core` + `nexus-shell-daemon` (Sprint 7 —
  P2P discovery + curator pipeline + pkarr browse)
- **Python workspace** (`packages/`) : `nexus-coordinator`
  (FastAPI + dispatcher + kudos ledger), `nexus-sdk` (NexusApp
  ABC + TabView), `nexus-app-gov` / `-coldcase` / `-forensics`
  (apps officielles qui tournent sur le socle P2P)
- **Frontend** (`web/`) : React + Vite + TypeScript + Tailwind
  + shadcn/ui + Zustand + React Query. Pages : Browse, Curators,
  Network, OnboardingEmpty, ProjectDetail, Projects
- **iroh stack** pinné : iroh 0.97 / iroh-docs 0.97 / iroh-gossip
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
│   └── nexus-shell-daemon/            # shell daemon binary (HTTP + gossip subscribe)
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

## État actuel (2026-04-11, master tip `9cc0796`)
- **Sprints 0-7 CLOSED**
- **304 Rust tests** / 40 SDK / 57+1 coordinator / 3 app-gov
  / 114 Vitest / 13 Playwright / 4/4 size-limit — tous verts
- Sprint 7 a livré le nexus-shell-daemon P2P discovery layer
  (curator list Ed25519 + gossip subscribe + pkarr browse +
  coordinator proxy + React pages live)
- **Prochain pas** : Sprint 8 Phase 0 audit gate joue
  `.planning/sprint7_audit_plan.md` dans une session fraîche et
  produit `sprint7_audit_findings.md` avant que Sprint 8 Phase A
  puisse démarrer

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

## Décisions architecturales gelées
Cf. `nexus_grid_pivot.md` (memory) §« Décisions actées (à ne PAS
re-débattre) » — 12 items dont pivot P2P intégral, Option G
hybride Rust+Python, iroh 0.97 pinné, visibilité 2 états
public/privé, zéro modération centrale, curator lists
Ed25519+gossip+blobs, kudos per-project, HTTP loopback via
coordinator proxy (Sprint 7 D1), singleton strict shell daemon
(Sprint 7 D2), AGPL-3.0 maintenue.

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
