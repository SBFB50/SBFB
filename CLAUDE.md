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

## Deploy verifie (Sprint 14)
Apps publiques deployees **depuis le repo source** par le
coordinateur (clone → Keyoxide Ed25519 → zip → provenance.json
SLSA L1). Code sur le reseau = code du repo. Multi-forge, zero
OAuth. Cf. `sprint14_keyoxide_decision.md` (memory).

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
│   ├── nexus-events-core/             # SecurityEvent enum + EventWriter trait +
│   │                                  # JsonFileWriter JSONL + EtwWriter (Windows)
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
│   └── archive/v1.2/                  # S16-25 (loopback hardening, research, supply chain, transport hardening, Gate 2 prerequisites, rate-limit + PII defense-in-depth, Sybil-resistance, ephemeral workers, guardrails + hooks + re-run + DNS fallback, key rotation + C3 handoffs + D5 capabilities)
├── docs/
│   ├── claude/README.md               # WORKFLOW SOURCE OF TRUTH (lire d'abord)
│   ├── rust/PATTERNS.md               # patterns Rust + tech debt tracking
│   └── shell/PATTERNS.md              # patterns shell/coordinator + T1..T7 tech debt
└── examples/hello-world-app/
```

## Securite
Loopback HTTP durci (bearer + Host + Origin + peer creds UDS/NP).
GPU consent 4 niveaux worker-side. Threat model dans
[`docs/security/THREAT_MODEL.md`](docs/security/THREAT_MODEL.md).
Runtime isolation roadmap dans
[`docs/security/RUNTIME_ISOLATION.md`](docs/security/RUNTIME_ISOLATION.md).

## Etat actuel
- **Sprints 0-26 CLOSED**, v1.2 en cours. Audit gate S26 = S27
  Phase 0 (`.planning/active/sprint26_audit_plan.md`).
- **~1752 tests total** (802 Rust / 193 SDK / 377+6 coord / 46
  app-gov / 264 Vitest / 43 Playwright / 7/7 size-limit) — tous
  verts code (45 coord fail PyO3 wheel stale + 4 gov collect errors
  + 16 PW env fail — meme root cause wheel stale, pas regression).
- Carry S27 : T-NN+2 iframe Rust-wasm (PATTERNS §P34).
  LT-2 Radicle sortie cap G7 (trigger tag v1.0).
  LT-3/LT-4 hors-sprint (post-v1.0).
  LT-5 redundancy persistence (ex-P2-D-1, reclassifie S26).
  LT-6 iroh neighborhood enrichment (ex-P2-E-1-iroh, reclassifie S26).
- Zones rouges : R-iroh-audit P0 / R-wasmtime-cve P0 /
  R-libcrux-hax P2 / R-pyodide-escape (inchangees).
- Historique sprint-par-sprint → `docs/claude/SPRINT_LOG.md`.

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
Cf. `nexus_grid_pivot.md` (memory) — **a ne PAS re-debattre** :
- Pivot P2P integral, Option G hybride Rust+Python
- iroh 0.97 pinne, visibilite 2 etats public/prive
- Zero moderation centrale, curator lists Ed25519+gossip+blobs
- Kudos per-project, HTTP loopback via coordinator proxy
- Singleton strict shell daemon, AGPL-3.0 maintenue
- Archive zip = format universel, daemon blob-serve = rendu,
  shell = iframe host agnostique
- postMessage bridge = seul canal iframe ↔ reseau (3 methodes)
- Deploy verifie from source (Keyoxide + SLSA L1 provenance)
- Launcher Rust minimal (pas Tauri, browser = client)

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
