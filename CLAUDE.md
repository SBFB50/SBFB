# nexus-grid / SBFB

## Projet
Plateforme P2P universelle de compute et d'hebergement d'apps.
"Decentralized P2P compute network for apps. No central server.
No admin. Just protocol." N'importe qui publie une app (React,
Python/Pyodide, WASM, HTML pur, notebook, etc.) sous forme
d'archive web. Le reseau la distribue. Les clients la rendent
dans un iframe sandboxe via le daemon Rust blob-serve.

**Plateforme a source verifiable** : chaque app publique est
deployee depuis un repo Git avec provenance auto-attestee SLSA L1
(clone → Ed25519 → zip → provenance.json). Les utilisateurs
peuvent voir le code source, signaler un bug, contribuer via PR,
ou forker l'app et deployer leur propre version sur le reseau.
Inspire par F-Droid — les apps sont deployees depuis leur code
source. Le terme "open source" est reserve au code SBFB lui-meme
(AGPL-3.0 OSI). Les apps du reseau sont a "source verifiable".

Pivot 2026-04-10 depuis l'ancien NEXUS cold-case (supprime S51,
code dans l'historique Git pour reference).

## Source de vérité pour le workflow Claude
**Avant toute action, lire `docs/claude/README.md`.** Ce document
capture le système de travail multi-sprint qu'on utilise : cycle
kickoff → plan → code → verification → audit_plan, audit gate
pattern entre sprints, commit discipline atomique, memory system
externe, anti-patterns. Une session fraîche sans cette lecture
ré-invente les règles et produit du code hors-convention.

## Agents d'orchestration (depuis S65+)
Le main thread est un **ROUTEUR**. Il detecte le cas (A/B/C/D)
via le bootstrap §7.1 puis invoque l'agent specialise. Les agents
ecrivent leurs artefacts dans `.planning/active/` — le main thread
lit le verdict et avance ou s'arrete.

| Agent | Cas | Artefact | Fallback |
|---|---|---|---|
| `nexus-audit-gate` | A | audit_findings.md | main thread + README §3 |
| `nexus-sprint-kickoff` | C | kickoff.md + plan.md + design_review.md | main thread + README §2 |
| `nexus-phase-preflight-deep` | B pre-code | preflight.md | skill nexus-phase-preflight |
| `nexus-phase-review-deep` | B post-code | review.md | skill nexus-phase-review |

Ref detaillee : `docs/claude/README.md §7` + `.claude/agents/*.md`.

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
SLSA L1). La provenance lie un commit source au hash de l'archive
via une signature Ed25519 du noeud qui a deploye. C'est une
auto-attestation — un tiers peut verifier la provenance mais pas
encore reproduire le build independamment. Multi-forge, zero
OAuth. Cf. `sprint14_keyoxide_decision.md` (memory).

## Architecture (Rust + Frontend, post-S50)
- **Rust workspace** (`crates/`) : `nexus-core-rs` (iroh 0.98
  wrapper), `nexus-worker-core` (headless engine lib) +
  `nexus-worker` (binary), `nexus-shell-daemon-core` +
  `nexus-shell-daemon` (P2P discovery + curator pipeline + pkarr
  browse + coordinator lifecycle + dispatch + CLI),
  `nexus-launcher` (spawn daemon + open browser),
  `nexus-coordinator-rs` (DB + dispatcher + validator + kudos +
  invite + quarantine + capability + canary + guardrails),
  `nexus-events-core`, `nexus-executor`, `nexus-trace-core`,
  `nexus-test-harness`
- **Frontend** (`web/`) : React + Vite + TypeScript + Tailwind
  + shadcn/ui + Zustand + React Query.
  Pages : Browse, Curators, Network, OnboardingEmpty,
  ProjectDetail, Projects. Le shell est un **iframe host** pour
  les apps distantes — il ne connait pas la techno de l'app.
- **iroh stack** pinne : iroh 0.98 / iroh-docs 0.98 / iroh-gossip
  0.98 / iroh-blobs 0.100

## Stack
- Windows 11, RTX 5080 16GB VRAM
- Rust 1.94 (rustup / cargo)
- Node.js (frontend React dans `web/`)
- Ollama (worker-side LLM runtime)

## Structure des crates / packages
```
nexus-grid/
├── Cargo.toml                         # workspace Rust
├── crates/
│   ├── nexus-core-rs/                 # iroh wrapper (docs, gossip, blobs,
│   │                                  # discovery, curator crypto, canonical bytes JCS)
│   ├── nexus-events-core/             # SecurityEvent enum + EventWriter trait +
│   │                                  # JsonFileWriter JSONL + EtwWriter (Windows)
│   ├── nexus-worker-core/             # engine lib headless (state machine,
│   │                                  # allowlist SQLite, GPU monitor, Ollama client)
│   ├── nexus-worker/                  # worker binary (CLI + TUI + state writer)
│   ├── nexus-shell-daemon-core/       # P2P discovery lib (curator runtime,
│   │                                  # browse aggregator, registry singleton)
│   ├── nexus-shell-daemon/            # shell daemon binary (HTTP + gossip subscribe)
│   └── nexus-launcher/                # minimal launcher (spawn daemon + open browser)
├── web/                               # shell React (Browse, Curators, Network, etc.)
├── .planning/                         # sprints (active/ + archive/v{X}/ + roadmaps + research)
│   ├── active/                        # sprint en cours uniquement (kickoff, plan, audit_findings du precedent, verification, audit_plan)
│   ├── archive/v1.0/                  # S0-13 (pivot, P2P, universal render, bridge, launcher)
│   ├── archive/v1.1/                  # S14-15 (verified deploy, bridge bidirectionnel, watchdog)
│   └── archive/v1.2/                  # S16-32 (loopback hardening, research, supply chain, transport hardening, Gate 2 prerequisites, rate-limit + PII defense-in-depth, Sybil-resistance, ephemeral workers, guardrails + hooks + re-run + DNS fallback, key rotation + C3 handoffs + D5 capabilities, MCP server + OS audit + task_handler, watermark SynthID + Couche 3 multi-forge + Gate 3 showcase, dette pair blob-serve COOP/COEP + warrant canary FROST DKG, task_runner reel + output filter E2E + Tor transport phase 1, dette pair iroh 0.98 + rusqlite 0.36 + arti-client activation, gossip resilience + bridge extensions + dette pair)
├── docs/
│   ├── claude/README.md               # WORKFLOW SOURCE OF TRUTH (lire d'abord)
│   ├── rust/PATTERNS.md               # patterns Rust + tech debt tracking
│   └── shell/PATTERNS.md              # patterns shell/coordinator + T1..T7 tech debt
├── examples/hello-world-app/
├── examples/sbfb-explorer/            # Protocol Explorer (5 sections + verification demo)
└── examples/sbfb-ideas/               # Ideas Hub (vote + storage P2P)
```

## Securite
Loopback HTTP durci (bearer + Host + Origin + peer creds UDS/NP).
GPU consent 4 niveaux worker-side. Threat model dans
[`docs/security/THREAT_MODEL.md`](docs/security/THREAT_MODEL.md).
Runtime isolation roadmap dans
[`docs/security/RUNTIME_ISOLATION.md`](docs/security/RUNTIME_ISOLATION.md).

## Etat actuel
- **Sprints 0-65 CLOSED**, v2.1 ouverte. **Tag v1.0 pose.**
  Projet Rust+Frontend pur depuis S50-S51.
  S65 contrat public (1er sprint roadmap v3 Arc 1 Fondations) :
  Phase A MANDATORY P2-FEED-INSERT-NO-AUTH-TIER 3/3 CLOSED
  (auth tier header X-SBFB-Feed-Internal guard feed_insert) +
  P2-VERIFY-ENTRY-VERSION-GUARD CLOSED (verify_entry rejette
  version != 1) + raw-op migration FeedEntry.op →
  serde_json::Value (try_parse_op + validate unknown ops) +
  deploy→feed wiring ReleasePublished auto-insert + https
  enforce + TRUST_TAXONOMY.md 6 niveaux + COMMONS.md + tests
  auth tier + version guard + unknown op + canonical + deploy +
  Phase B badges UI migration vocabulaire TRUST_TAXONOMY
  (Browse + BrowsedProject + GpuConsentDialog + Network +
  Curators + Protocol Explorer + PUBLISH_MODEL) +
  Phase C badge dynamique post-verification 3 etats +
  scan-trust-wording.sh script CI non-regression +
  Phase D FACTORY_GATES.md 11 gates FG0-FG10 spec +
  SBFB_JSON_V2.md manifest v2 spec + dette pair 5 items CLOSED
  (COMMIT-TITLE-FORMAT + REVIEW-ORDER + PYTHON-BLOCK-EXEMPTION
  reclassifie resolved + EXPLORER-ESCAPE-SINGLE-QUOTE +
  PLAYWRIGHT-SPECS-STALE 30 fichiers supprimes) + wrap-up.
  P2P valide cross-machine : LAN Win↔Mac, WAN dev↔VPS Helsinki.
  CI operationnel : Woodpecker ci.sbfb.world + GHA.
- **~1607 tests total** (1333 Rust / 268 Vitest / 6/6 size-limit)
  — tous verts code. S65 : +7 delta Rust (1326→1333,
  Phase A +7), +3 delta Vitest (265→268, Phase C +3).
- Carry S66 :
  P2-A-1 rand blocker upstream (exemption externe).
  P2-AUDIT-2 pre-release transitives iroh (herite pin 0.98).
  P2-G-1 exe lock intermittent (monitoring).
  **P2-PROVENANCE-404-BRIDGE (3/3 MANDATORY S66)** —
  enrichissement UX provenance.
  **P2-VERIFY-LOCAL-KEY-ONLY (3/3 MANDATORY S66)** —
  cross-node verification.
  P2-FEED-JOIN-HANDLE-LEAK (2/3 — feed_join fire-and-forget,
  pas de shutdown channel).
  P2-ORPHAN-REPUBLISH-RECOVERY (2/3 — pas de republish DB→iroh-docs
  apres publish fail + tail-safe skip).
  P2-THREAT-MODEL-FEED-SURFACE (1/3 — THREAT_MODEL.md ne couvre
  pas le feed, carry S64 audit).
  T-NN+2 iframe Rust-wasm (PATTERNS §P34).
  LT-2 Radicle sortie cap G7 — **trigger PENDING** (tag v1.0 pose
  localement, pas encore pousse vers origin).
  LT-3/LT-4 hors-sprint (post-v1.0).
  LT-5 redundancy persistence (ex-P2-D-1, reclassifie S26).
  LT-6 iroh neighborhood enrichment — **RESOLVED S32 Phase A**.
  LT-7 self-hosted build — Tier 1+2 DONE (S55). Tier 3 P2P infra
  validee (S60). Worker quorum E2E carry post-tag. Diversite
  publique post-launch.
- **Roadmap v1.0 livree** :
  S59 = launcher readiness + verified deploy E2E + LT-1 Kudos-v2
  + stabilisation → **early adopter ready** ✓ DONE.
  S60 = installer NSIS + tray icon + LT-7 Tier 3 + frontend
  bundling → **end user ready** ✓ DONE → **tag v1.0**.
- **Roadmap v3 — Confiance + Factory Canari + RRV** :
  11 sprints (S65-S75) en 4 arcs. Arc 1 Fondations (S65 contrat
  public + S66 durabilite). Arc 2 Factory + Canari (S67 Factory
  Foundation + S68 Broker/Preview + S69 Babel Reader canari + pilote
  ferme). Arc 3 Intelligence Verifiable (S70 RRV FTS5 + S71 Proof
  Cards + S72 SearchManifest). Arc 4 Industrialisation (S73
  Gouvernance complete + S74 Babel translation beta + S75 Pack
  produit defendable). ~24 semaines, mai-novembre 2026.
  Detail : `.planning/roadmap_v3_public_trust_factory_babel_rrv.md`.
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

# Frontend
cd web && npm install && npm run lint && \
  npx tsc --noEmit -p tsconfig.app.json && \
  npm run test:unit && npm run test:coverage && \
  npm run build && npm run size && \
  bash scripts/scan-en-strings.sh
```

## Decisions architecturales gelees
Cf. `nexus_grid_pivot.md` (memory) — **a ne PAS re-debattre** :
- Pivot P2P integral, Option G hybride Rust+Python
- iroh 0.98 pinne (upgrade S32, Day 0 #3 leve), visibilite 2 etats public/prive
- Zero moderation centrale, curator lists Ed25519+gossip+blobs
- Kudos per-project, HTTP loopback via coordinator proxy
- Singleton strict shell daemon, AGPL-3.0 maintenue
- Archive zip = format universel, daemon blob-serve = rendu,
  shell = iframe host agnostique
- postMessage bridge = seul canal iframe ↔ reseau (3 methodes)
- Deploy verifie from source (Keyoxide + SLSA L1 provenance)
- Launcher Rust minimal (pas Tauri, browser = client)
- iroh 0.98 pour Arc 1-2 (S65-S69), evaluer upgrade 1.0 Gate 1
- OS sandbox pour Factory, pas wasmtime (12 CVE avril 2026)
- Pilote ferme 2-3 personnes (R-iroh-audit P0 → pas public)
- Vocabulaire "source verifiable" (pas "open source" pour apps)
- Factory = module daemon/broker Rust, pas app iframe
- Feed raw-op extensible (serde_json::Value), pas de bump par op
- FTS5 pour RRV S70, Tantivy en gate post-S75 si >50K docs

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

- **Feed extensible via raw-op** : `FeedEntry.op` est stocke
  comme `serde_json::Value`. Ajouter une nouvelle operation
  (CuratorVouched, SearchManifestPublished) ne bump PAS
  `FEED_FORMAT_VERSION`. Les noeuds anciens stockent et propagent
  les operations inconnues sans les interpreter. Le bump n'est
  necessaire QUE si la structure de l'enveloppe `FeedEntry` change.
- **`*_ANNOUNCEMENT_VERSION` restent a 1** jusqu'au go-live.
  Un sprint qui change le canonical redefinit la v1 courante.
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
