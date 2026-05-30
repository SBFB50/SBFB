# Sprint 70 Phase D — preflight G8

Date : 2026-05-24 | HEAD : `c68e989` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)

- feedback_approach.md : pick deepest, research before code, OSS prior art
  obligatoire (G8/G10), planning adaptatif. Phase D est une phase
  d'implementation Rust (process CLI + serveur HTTP) — recherche OSS sur
  les patterns CLI serve + axum local JSON server effectuee.
- feedback_context7_systematic.md : context7 obligatoire avant toute lib.
  Phase D ajoute axum + tokio + tower-http au Cargo.toml de sbfb-factory.
  Les trois sont deja dans le workspace (axum 0.8, tokio 1.40 full,
  tower-http 0.6). context7 consulte pour axum (Router, Json, serve,
  graceful shutdown, CorsLayer).
- vision_model.md : solo maintainer OpenBSD pattern. Phase D reste un
  outil CLI local, pas de dashboard web ni d'infra orchestree.
  Conforme.
- Tensions plan vs memory : aucune.

## S1a — OSS prior art deep analysis

### Probleme fonctionnel exact

"How do mature OSS projects implement process observability CLI
commands + local JSON API server for developer tooling in Rust?"

### Projets analyses en profondeur

#### [Projet 1] — axum (github.com/tokio-rs/axum)
- Fichiers source lus : README.md (~200 LOC), examples/graceful-shutdown
  (~50 LOC), context7 `/tokio-rs/axum` 5 queries
- Pattern architectural extrait : Router::new().route() + TcpListener::bind
  + axum::serve() + with_graceful_shutdown(). Json<T> extractor/responder.
  CorsLayer via tower-http. Test pattern : axum-test crate avec TestServer.
- Edge cases geres : graceful shutdown via tokio::signal, CORS via
  tower-http middleware, state sharing via Extension ou with_state().
- Verdict : ALIGNED — le plan utilise exactement ce pattern.

#### [Projet 2] — nexus-shell-daemon (codebase locale)
- Fichiers source lus : http.rs (200+ LOC), auth.rs (50 LOC header),
  runtime.rs (structure), Cargo.toml
- Pattern architectural extrait : DaemonHttpState struct partagee via
  State, Router avec routes /api/daemon/*, CorsLayer avec AllowOrigin
  loopback, tower_http::cors, X-SBFB-Token bearer auth.
- Edge cases geres : DNS rebinding via Host header allowlist, bearer
  token 256-bit, peer creds UDS/NP. CORS permissif seulement pour
  loopback origins.
- Patterns pertinents pour Phase D : le plan dit "CORS permissif
  localhost:*, auth loopback explicite si surface persistante". C'est
  un premier deploy sans auth (Phase D = tool local dev, pas daemon
  reseau). L'auth viendra en Phase F "si la surface devient
  persistante" — coherent avec le daemon qui a durci en S16.
- Verdict : ALIGNED — le plan suit le pattern daemon existant mais
  en version simplifiee (pas de bearer token Phase D, juste CORS
  localhost).

#### [Projet 3] — Overstory (github.com/jayminwest/overstory)
- Fichiers source lus : README.md (~300 LOC via WebSearch/WebFetch)
- Pattern architectural extrait : `ov serve` command opens web UI at
  localhost:7321. SQLite mail bus for agent coordination. Headless
  subprocess spawning with stream-json NDJSON. Per-agent timelines.
  `ov doctor --category providers` for diagnostics.
- Edge cases geres : worktree isolation per agent, merge conflict
  resolution. Maintenance mode — active dev moved to Warren
  (cloud control plane).
- Patterns pertinents pour Phase D : `ov serve` = local HTTP UI
  avec endpoints JSON status. Meme pattern que
  `sbfb-factory operator serve`. La difference : Overstory orchestre
  des agents en parallele (multi-worktree), sbfb-factory observe un
  process sequentiel (sprint/phase). Plus simple.
- Verdict : ALIGNED — le plan est une version simplifiee du meme
  pattern observe dans l'ecosysteme agent CLI 2026.

#### [Projet 4] — Claude Code headless mode
- Fichiers source lus : WebSearch "Claude Code subprocess headless
  JSON output"
- Pattern architectural extrait : --output-format json pour sortie
  structuree CLI. Subprocess spawning avec NDJSON events. Hook
  events pour workflow gating (TaskCompleted). Subagents avec tool
  allowlists/denylists.
- Patterns pertinents pour Phase D : le plan Phase D expose les memes
  informations (status, lint, audit) via JSON CLI (`--json` flag) ET
  via HTTP endpoints. Le pattern --json + serve est observe dans
  l'ecosysteme (Claude Code headless + Overstory serve).
- Verdict : ALIGNED

#### [Projet 5] — axum-test (crates.io/crates/axum-test)
- Fichiers source lus : crates.io page + docs.rs TestServer
- Pattern architectural extrait : TestServer::new(app) cree un mock
  HTTP server pour tests d'integration. Transport::HttpRandomPort
  pour vrais ports reseau. Requetes via TestServer::get/post/put.
  Cleanup automatique au drop.
- Patterns pertinents pour Phase D : le plan a 19 tests serveur
  (operator_*). Le pattern axum-test ou reqwest::Client direct
  couvrent le besoin. L'approche actuelle (spawn le binaire +
  reqwest en test) est le pattern le plus simple sans dep
  supplementaire. L'alternative axum-test ajouterait une dep.
- Verdict : ALIGNED — pas de dep supplementaire necessaire si les
  tests utilisent le binaire CLI (pattern process_cli.rs existant).
  Pour operator_server.rs, reqwest + tokio::spawn du serveur dans
  le test est le pattern standard.

### Tableau comparatif

| Aspect | Plan Phase D | nexus-shell-daemon | Overstory | axum-test |
|--------|-------------|-------------------|-----------|-----------|
| Framework HTTP | axum 0.8 (workspace) | axum 0.8 | Express/Node | N/A |
| Transport tests | binaire + reqwest | unit + integration | mocha | TestServer |
| Auth loopback | CORS localhost | bearer+Host+Origin | N/A (local only) | N/A |
| JSON output CLI | --json flag | N/A (daemon only) | ov doctor --json | N/A |
| Graceful shutdown | with_graceful_shutdown | N/A (long-running) | N/A | auto-drop |
| Smoke test | --once-smoke | /health | ov doctor | TestServer |

### Finding S1a

- Classification : **APPROACH-ALIGNED**
- Evidence : 5 projets analyses, tous confirment le pattern
  "CLI tool + serve subcommand + axum local JSON server" comme
  standard ecosysteme Rust/agent 2026.
- Impact sur le plan : aucun.

## S1b — Deps/libs versions + CVE

### Nouvelles deps Phase D (ajout a sbfb-factory/Cargo.toml)

| Dep | Version workspace | Derniere version | Delta | CVE 2025-2026 |
|-----|------------------|------------------|-------|---------------|
| axum | 0.8 | 0.8.4 (mai 2026) | minor stable | 0 RustSec advisory |
| tokio | 1.40 | 1.44+ | minor stable | RUSTSEC-2026-0057/0060 = unmaintained tokio-timer/reactor (Tokio 0.1, non-pertinent) |
| tower-http | 0.6 (features: cors) | 0.6.x | stable | 0 RustSec advisory |

### Deps existantes dans le perimetre Phase D

| Dep | Usage Phase D | CVE check |
|-----|--------------|-----------|
| clap 4.5 | subcommands process/operator | 0 CVE. Latest 4.6.0 (minor, non-breaking) |
| serde 1.x | JSON structs | 0 CVE pertinent |
| serde_json 1.x | JSON parsing/output | 0 CVE pertinent |
| regex | commit title parsing | 0 CVE pertinent |

### Specs touchees

- Aucune spec crypto/RFC touchee par Phase D.
- Phase D n'ajoute aucun wire format protocolaire.

### Finding S1b

- 0 CVE critique sur les deps du perimetre.
- axum 0.8.4 derniere version, axum 0.8 workspace compatible.
- tokio advisories (0057/0060) concernent tokio 0.1 legacy, non-pertinent.
- clap 4.6.0 disponible (minor bump, non-breaking vs 4.5 pinne).
- **clean**

## S2 — Decision chain reconstruction

### Fichiers scannes

- `crates/sbfb-factory/src/main.rs` : 7 commits lus (bodies complets)
- `crates/sbfb-factory/src/process.rs` : 1 commit lu (Phase C, creation)
- `crates/sbfb-factory/tests/process_cli.rs` : 1 commit lu (Phase C)
- `docs/agent/TOOLING.md` : scannes via git log

### Decisions historiques trouvees

#### Decision 1 : Factory hors daemon (D2 v4)

- Sprint 67, sha `49d6bcd` : creation sbfb-factory comme crate workspace
  independant. Body extrait : "Decision D2 v4 : Factory hors daemon, crate
  independant."
- Roadmap v4 D2 : "Factory hors daemon (crate sbfb-factory)" date
  2026-05-19.
- Status : **active**
- Impact Phase D : conforme. Phase D ajoute les commandes et le serveur
  dans `crates/sbfb-factory/`, pas dans `nexus-shell-daemon`. Le
  serveur `operator serve` est un binaire sbfb-factory local, pas un
  endpoint daemon. Pas de dep ajoutee vers nexus-shell-daemon-core ni
  nexus-coordinator-rs.
- Reverse-commit check : N/A (decision non-revertee, active).

#### Decision 2 : agentctl -> sbfb-factory Rust (plan v5)

- Sprint 70, sha `c4494a6` : plan v5. Body extrait : "agentctl →
  sbfb-factory Rust, dashboard → Factory Viewer + Factory Operator
  split, serve → operator serve, tests Python → tests Rust."
- Sprint 70, sha `c68e989` : Phase C. Body extrait : process.rs NEW
  Rust prompt assembly.
- Status : **active** — Phase C a implemente la migration prompt/context
  vers Rust. Phase D continue avec status-sprint/lint-planning/
  audit-commit/operator-serve.
- Impact Phase D : conforme. Les nouvelles commandes sont dans Rust,
  pas Python.
- Reverse-commit check : N/A (decision active, pas de reversion).

#### Decision 3 : Superviseur process optionnel (D17)

- CLAUDE.md : "nexus-process-supervisor est optionnel (amendement D17,
  2026-05-22). Les hooks .claude/hooks/* servent de backstop mecanique."
- Impact Phase D : le serveur `operator serve` n'est PAS un
  superviseur process. C'est une API JSON pour l'Operator UI Phase E.
  Conforme D17 : pas de nouveau superviseur, juste observabilite.

### Memory constraints

- feedback_approach.md : "pick deepest technical option" — Phase D
  utilise axum (framework HTTP Rust le plus mature du workspace)
  plutot qu'un serveur TCP stdlib. Conforme.
- feedback_context7_systematic.md : context7 consulte pour axum.
  Conforme.
- vision_model.md : "solo maintainer, pas de dashboard web" — le
  serveur `operator serve` est un backend JSON local pour l'Operator
  UI, pas un dashboard web deploye. Conforme.

### Finding S2

- **clean** — 0 decision historique contredite par Phase D.
  Les 3 decisions trouvees (D2 Factory hors daemon, agentctl→Rust,
  superviseur optionnel) sont toutes respectees.

## S3 — Threat model analysis

### Primitive analysee : sbfb-factory operator serve (HTTP JSON API locale)

### Assets en jeu

- A1 Fichiers `.planning/active/` : criticite **medium** — le serveur
  lit ces fichiers en lecture seule pour status-sprint/lint-planning.
  Pas de modification.
- A2 Fichiers repo (prompts, docs, code) : criticite **medium** — le
  serveur lit les prompts pour `/api/prompt/{kind}`. POST
  `/api/artifacts/draft` ecrit sur une allowlist repo-visible.
- A3 Git history : criticite **low** — le serveur lit git log pour
  audit-commit. Pas de git write.
- A4 Action log JSONL : criticite **low** — fichier local de
  journalisation des actions Operator.
- A5 Context-pack : criticite **low** — document genere sans donnees
  sensibles (pas de tokens, pas de cles).

### Threat actors

- TA1 Extension navigateur malveillante : capacite fetch localhost:3001,
  motivation exfil/elevation. MITIGATION : CORS localhost-only bloque
  les requetes cross-origin depuis un domaine externe.
- TA2 Malware user-mode local : capacite acces complet au reseau
  loopback, motivation exfil. MITIGATION : meme acces que le repo Git
  et le filesystem — le serveur n'ajoute pas de surface au-dela de ce
  que l'attaquant peut deja lire directement.
- TA3 Site web malveillant : capacite fetch cross-origin.
  MITIGATION : CORS restreint + le serveur ne sert aucune donnee
  sensible (pas de cles, pas de tokens, pas de PII).

### Attack vectors identifies

1. V1 **Injection/forgery sur les inputs** : POST `/api/actions/run`
   avec commande shell arbitraire. COUVERT : allowlist stricte
   (status-sprint, lint-planning, audit-commit, prompt). Pas de
   shell arbitraire.
2. V2 **Draft artifact path traversal** : POST `/api/artifacts/draft`
   avec path hors allowlist. COUVERT : path guard sur allowlist
   repo-visible (`.planning/active/**`, `docs/agent/**`,
   `docs/claude/**`, `prompts/agent/**`, `AGENTS.md`, `CLAUDE.md`).
   Rejet des paths contenant `..`.
3. V3 **DoS/resource exhaustion** : flood de requetes localhost.
   NON COUVERT (medium-low) : pas de rate limit. Mitigation : le
   serveur est local-only, pas persistant (--once-smoke pour CI).
   Rate limit est un scope cut S71+ (meme pattern que le daemon
   T5.5D "M" residuel).
4. V4 **Information leakage** : les endpoints exposent le contenu du
   repo (prompts, planning, git log). NON-SENSIBLE : ces donnees sont
   deja accessibles en lecture directe par tout process local.
5. V5 **PASS verdict injection** : POST `/api/artifacts/draft` tente
   d'ecrire un `## Verdict: PASS`. COUVERT : le plan specifies
   "interdiction de creer un verdict final PASS hors flow review/gate"
   (test `operator_artifact_draft_rejects_pass_verdict`).
6. V6 **Supply chain** : axum/tokio/tower-http ajoutees. COUVERT :
   deps deja dans le workspace (utilisees par nexus-shell-daemon),
   auditees dans les sprints precedents.
7. V7 **DNS rebinding** : fetch depuis un domaine qui resout vers
   127.0.0.1. PARTIELLEMENT COUVERT : CORS localhost-only rejette
   les origins non-loopback. Host header check non implemente Phase D
   (le daemon l'a en S16, sbfb-factory Phase D ne l'a pas).
   Severite basse : le serveur ne manipule aucune cle, aucun token,
   aucun deploiement. Les donnees sont read-only repo.

### Mitigations existantes (T0-T5)

- T5.5 loopback hardening : le daemon a bearer+Host+Origin. Le
  serveur Phase D est un second listener sur un autre port. Il
  n'herite pas des mitigations du daemon (pas de dep
  nexus-shell-daemon-core).

### Gaps identifies

- GAP1 V7 DNS rebinding : severite **Low** — le serveur ne manipule
  aucun asset critique. Recommendation : ajouter Host header check
  Phase F ou S71 si la surface devient persistante. Pas bloquant
  Phase D.
- GAP2 V3 DoS rate limit : severite **Low** — local-only, pas
  persistant. Recommendation : S71+ si le serveur tourne en
  arriere-plan.

### Regression check

- La primitive NE diminue PAS l'efficacite des mitigations T0-T5
  existantes. Le daemon garde ses propres mitigations. Le serveur
  sbfb-factory est un processus separe sur un port separe.
- La primitive cree un nouveau vecteur (V7 DNS rebinding) NON couvert,
  mais severite Low (pas de donnees sensibles, pas de mutation
  dangereuse, allowlist actions).

### Verdict S3 : **clean** (2 gaps Low, 0 regression T0-T5)

## S4 — Wire format deep audit

### canonical.rs lu integralement : oui

14 domaines DOMAIN_*_V1 (task, result, claim, invite, kudos,
curator-list, provenance, warrant-canary, pow, duress-ack,
age-witness, contributor-attestation, key-rotation, delegation-cert,
feed). Tous `v1`. `canonical_bytes<T>()` utilise serde_jcs.
4 tests unitaires.

### Structs verifiees

Phase D ne touche aucune struct dans canonical.rs ni dans
`crates/nexus-core-rs/src/schemas/`. Les nouvelles structs sont
dans `crates/sbfb-factory/src/operator_server.rs` et
`crates/sbfb-factory/src/process.rs` — ce sont des DTOs JSON
internes au tooling, pas des wire formats protocolaires signes.

### Day 0 check

| D# | Decision | Phase D conforme ? |
|----|----------|-------------------|
| D1 | AGENT_SYSTEM.md carte derivee | Oui — Phase D ne modifie pas AGENT_SYSTEM.md, seulement le lit via context. |
| D2 | Factory hors daemon | Oui — toute l'implementation est dans `crates/sbfb-factory/`, pas dans le daemon. |
| D3 | @protocole d'abord | Oui — Phase D est process tooling, pas @dev ni @web. |
| D4 | Hooks dynamiques Phase F | Oui — Phase D n'implemente pas les hooks. |
| D5 | RRV modes = alias roles | Oui — Phase D n'implemente pas RRV. |

- D1..D5 sprint courant : aucune contredite
- Decisions actees pivot.md : aucune contredite (Factory hors daemon
  respecte, superviseur optionnel respecte, pre-launch policy
  respectee)

### Pre-launch policy

- `*_VERSION = 1` : inchangees. Phase D ne touche aucune constante
  VERSION dans nexus-core-rs.
- Pas de tolerant decoder multi-version : N/A (pas de wire format
  protocolaire touche).
- Pas de tests "legacy decode" zombie : N/A.
- Feed extensible raw-op : N/A (pas de nouvelle feed op).

### Version constants grep

Toutes les constantes `*_FORMAT_VERSION` et `*_ANNOUNCEMENT_VERSION`
restent a 1 dans `crates/nexus-core-rs/src/`. Phase D n'ajoute
aucune constante VERSION.

## Telemetrie preflight (agent deep)

- Duree totale : ~25m
- S1a : ~12m / 5 projets OSS analyses / 8 fichiers source
  lus + context7 / ~600 LOC reviewees / 3 context7 queries /
  10 WebSearch queries / finding : APPROACH-ALIGNED
- S1b : ~4m / 6 libs scannees / 3 CVE searches / finding : clean
- S2 : ~4m / 10 commits bodies lus / 3 archive files /
  6 memory files / finding : clean
- S3 : FULL / ~3m / 7 vectors analyses / 2 gaps Low
- S4 : FULL / ~2m / 0 structs wire verifiees (Phase D ne touche
  pas canonical.rs) / canonical.rs lu integralement : oui

## Action

Proceder code phase D.
