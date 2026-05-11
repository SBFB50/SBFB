<!--
written: 2026-04-29
author: FlowUP + Claude S37 wrap-up
revised: 2026-05-01 (post-S48 — constat derive S46-S48, plan migration restant)
status: ACTIVE — REVISION S49+
triggers_revalidate:
  - "Tier N complete ahead of schedule → compacter sprints suivants"
  - "Nouveau blocker dep (ex: ONNX Rust binding instable) → re-evaluer tier"
-->

# Roadmap migration Python → Rust + features pre-v1.0 (S38-S56)

Decision utilisateur 2026-04-29 post-S37 wrap-up, etendue 2026-04-30
post-S44 (ajout S49-S56 features pre-v1.0). Le coordinator Python
est progressivement remplace par le crate Rust. Le core path est
livre S35-S44. Cette roadmap planifie la suite : suppression Python,
CI/CD, deploy, Kudos v2, app package/trust/review, plugin
capabilities, privacy, Babel, puis freeze + tag v1.0.

## Etat d'entree (tip S37 `c53f663`)

- **Migre** : dispatcher, validator, kudos_ledger, db, types
  (~1276 LOC Rust, 3 endpoints HTTP live)
- **Reste** : ~5400 LOC Python repartis en 5 tiers
- **Hors-scope** : ~2900 LOC Python qui disparaissent naturellement

## Plan sprint-par-sprint

### S38 — validator_loop MANDATORY + OutputFilter Rust (Tier 1 part 1)

- MANDATORY P2-REVIEW-C-1-S35 validator_loop tokio 3/3 : refactor
  CuratorRuntimeHandle pour exposer LiveEvents au coordinator
- OutputFilter Rust : migrer `output_filter.py` (397 LOC) → module
  `output_filter.rs` dans nexus-coordinator-rs. Safety check
  post-result (regex patterns, blocklist matching)
- Guardrails pipeline orchestration (`guardrails.py` 137 LOC) :
  migrer le pipeline qui chaine OutputFilter + futur PiiRedactor

### S39 — PiiRedactor Rust (Tier 1 part 2) + canary registry (Tier 2 debut)

- PiiRedactor : migrer `pii_redactor.py` (483 LOC). Dep ONNX a
  evaluer — alternatives possibles : regex-only Rust (suffisant
  pre-v1.0) ou `ort` crate (ONNX Runtime Rust binding)
- CanaryRegistry : migrer `canary_registry.py` (366 LOC). State
  machine deja scaffoldee Rust-side (DKG/ceremony dans
  shell-daemon-core)

### S40 — Canary input + redundancy/re-run batch (Tier 2 fin + Tier 3)

- Canary input : migrer `canary_input.py` (782 LOC). Templates +
  generation
- Batch Tier 3 : migrer `redundancy.py` (158 LOC) +
  `rerun.py` (~140 LOC) + `watermark_detector.py` (119 LOC) +
  `honeypot.py` (221 LOC)

### S41 — Infra batch (Tier 4) → jalon "Python supprimable"

- Batch : `quarantine_queue.py` (369) + `upload_queue.py` (396) +
  `fairness.py` (62) + `pow_counter.py` (132) +
  `contributor_registry.py` (281) + `invite.py` (216) +
  `capability_store.py` (274)
- **Jalon** : a la fin de S41, toute la logique metier est Rust.
  Le coordinator Python ne sert plus que de proxy pour les routes
  HTTP secondaires

### S42-S44 — Routes API migration incrementale (Tier 5)

- S42 : `api/deploy.py` (505 LOC, verified deploy — la plus grosse)
  + `api/apps.py` (350 LOC)
- S43 : `api/files.py` (323) + `api/consent.py` (255) +
  `api/canary.py` (212) + `api/contributor.py` (141)
- S44 : routes restantes (~700 LOC, 7 fichiers : health, shell,
  tasks, kudos, events, diagnostic, worker_state)

### S45 — Suppression coordinator Python + cleanup (REVISE)

**Plan original** : supprimer `packages/nexus-coordinator/` entierement.
**Realite S45** : ~2500+ LOC app runtime (AppContext, events, commands,
state, MCP) non portables en 1 sprint. Scope reduit a "suppression
maximale, pas totale" — 14 fichiers routes API + 12 tests DELETE,
2 API routes portees (invite, quarantine), -5838 LOC Python.
**S46-S48 : derive** — tests integration + carries + dette au lieu de
continuer la migration. 3 sprints de retard sur le roadmap.

### Constat S48 — Etat reel de la migration (2026-05-01)

**PORTE EN RUST (14 modules, actifs via daemon HTTP)** :
dispatcher, validator, kudos_ledger, invite, canary_registry,
canary_input, pii_redactor, output_filter, upload_queue,
quarantine_queue, contributor_registry, capability_store, rerun,
guardrails + 14 route modules (54 routes HTTP).

**PYTHON ENCORE ACTIF (~9400 LOC, 25 modules + CLI + API)** :
Le coordinator Python reste le proprietaire du lifecycle iroh
(boot node, dispatch tasks, validate results, credit kudos).
Les modules Rust dans nexus-coordinator-rs existent mais ne
sont PAS dans le chemin runtime principal — ils servent
uniquement les endpoints HTTP du daemon.

**SANS EQUIVALENT RUST (~3165 LOC a porter ou supprimer)** :
- coordinator.py (833) — orchestrateur lifecycle
- CLI 8 fichiers (1025) — commandes utilisateur
- api/daemon.py (290) — proxy FastAPI → DELETE
- api/events.py (195) — SSE bridge
- api/app.py (134) — FastAPI factory → DELETE
- mcp_server.py (176) — MCP tools
- keystore.py (114) — Ed25519 persistence
- hooks.py (94) — dispatch hooks
- tor_client.py (92) — Tor wrapper
- peer_creds.py (92) — peer credentials
- admin_check.py (74) — OS privilege
- db/migrations.py (46) — migration runner

**INSIGHT CLE** : depuis Sprint 12, le modele de rendu est
archive-based (HTML zip dans iframe via blob-serve). Le SDK
Python NexusApp est LEGACY. Les apps publient des archives,
pas des plugins Python. Le systeme AppContext/commands/state
n'est utilise que par app-gov (app officielle de gouvernance).
La migration complete implique : (1) fusionner le coordinator
dans le daemon Rust, (2) convertir app-gov en archive HTML,
(3) supprimer Python/PyO3/SDK.

### S49 — Coordinator lifecycle → daemon Rust

**But** : le daemon Rust DEVIENT le coordinator. Plus de
process Python separe.

- **Phase A** : task dispatch actif dans le daemon
  Le daemon appelle `dispatcher.rs` pour ecrire des TaskEntry
  signees dans le doc iroh, au lieu de proxier au coordinator
  Python. Le coordinator Python n'est plus necessaire pour le
  dispatch. Wire : runtime.rs start() integre le dispatch loop.
  Fichiers : runtime.rs, dispatcher wiring.

- **Phase B** : validator subscription dans le daemon
  Le daemon subscribe au doc iroh et valide les results/claims
  via `validator.rs`, credite kudos via `kudos_ledger.rs`. La
  boucle validator Python est remplacee. Le coordinator Python
  n'est plus necessaire pour la validation.
  Fichiers : runtime.rs, validator_loop.rs enrichi.

- **Phase C** : CLI coordinator → daemon CLI
  Les commandes `init`, `start`, `canary`, `invite`, `quarantine`,
  `capability` sont portees dans le binaire nexus-shell-daemon
  (clap subcommands). `start` boot le daemon directement (pas
  uvicorn+FastAPI). ~1025 LOC Python → ~400 LOC Rust (clap derive).

### S50 — Suppression Python + cleanup

- **Phase A** : supprimer `packages/nexus-coordinator/` (~9400 LOC)
  Supprimer `packages/nexus-sdk/` (~4088 LOC)
  Supprimer `packages/nexus-app-gov/` (~2800 LOC) OU convertir en
  archive HTML (si app-gov est encore utile comme vitrine)
  Supprimer `crates/nexus-core-py/` (PyO3 bindings, ~2000 LOC)
  Nettoyer Cargo.toml workspace (retirer pyo3, maturin)
  Nettoyer pyproject.toml, uv workspace, .venv references
  Adapter CLAUDE.md, README, docs

- **Phase B** : adapter les tests
  Tests Python coord (264 + 17f + 6s) → equivalent Rust (ceux qui
  testent de la logique metier deja couverte par Rust) ou DELETE
  (ceux qui testent du code supprime).
  Tests SDK (195) → DELETE (SDK supprime)
  Tests app-gov (46) → DELETE ou adapter si archive

- **Phase C** : MCP server + hooks + events SSE + tor + peer_creds
  + admin_check + keystore + migrations.py
  Porter les modules restants sans equivalent Rust :
  - MCP server : port mcp crate Rust (ou supprimer si non-critique v1.0)
  - hooks.py : integrer dans dispatcher.rs
  - events SSE : axum SSE (tower layer)
  - tor_client : deja arti-client feature-gated dans nexus-core-rs
  - peer_creds : integrer dans auth.rs daemon
  - admin_check : integrer dans launcher
  - keystore : le daemon gere deja les keypairs via iroh
  - migrations.py : rusqlite_migration deja en place

### S51 — CI/CD + binaires + installer (ex-S46)

- GitHub Actions multi-OS (Linux/macOS/Windows)
- Release artifacts (binaires pre-build, checksums)
- Installer script (`curl | sh` pattern)
- **Simplification massive** : 1 binaire Rust (daemon = coordinator),
  plus de Python/uv/maturin a installer

### S52 — VPS deployment + smoke test reseau (ex-S47)

- Premier noeud live (VPS Hetzner/OVH)
- Smoke test P2P multi-noeuds
- Monitoring baseline

### S53 — Polish UX + docs utilisateur (ex-S48)

### S50 — Kudos v2 pre-v1 (LT-1 reclassifie)

- Reclassification LT-1 : declenchement avance pre-v1.0 (decision
  utilisateur 2026-04-30, cf. ROADMAP_COMMITMENTS.md)
- Migration DB kudos_v2 : schema familles contribution
  (Compute / Code / Review / Relay / Storage / Docs / Design /
  Accessibility / Moderation)
- Formule : log-utility (rendement decroissant hardware) + decay
  EMA fitness-aging (anti-decrochage nouveaux noeuds)
- Score composite multi-famille par worker
- API : GET /api/v1/kudos/v2/{project_id} avec breakdown famille
- Shell UI minimale : table par famille dans ProjectDetail
- Ref : docs/FAIRNESS_VISION.md + research S21

### S51 — App Package Protocol v1

- SBFB.json v2 manifest : version semantique app, categories
  structurees, dependencies manifest (min_bridge_version,
  required_capabilities), licence SPDX, artifact_hash canonical
- Source snapshot enrichi : commit_sha + tree_hash + provenance
  chain complete
- Build provenance v2 : etendre provenance.json (build env,
  reproducibility hints, dep lockfile hash)
- Distribution P2P : app package = blob iroh-blobs, resolvable
  par artifact_hash, partage hors internet via sync local
- Backward compat : ancien zip sans SBFB.json v2 = app legacy,
  fonctionne mais pas reviewable

### S52 — App Review / Trust / Vote v1

- AppReviewEntry payload signe Ed25519 : lie a artifact_hash +
  commit_sha + provenance_hash. Score 1-5 + texte optionnel
- Aggregation locale (pas de consensus global) : chaque daemon
  calcule son trust score a partir de ses curator lists +
  attestations + trust-web + kudos v2
- Poids plafonnes : aucun signal ne depasse 30% du score
  (anti-gaming). Signaux : curator trust, contributor attestation,
  trust-web cross-forge, kudos v2 score composite, anciennete
  node_id, diversite geographique (IP prefix heuristique)
- UI Browse : badge trust (high/medium/low/unknown), reviews
  visibles dans ProjectDetail
- Wire format : APP_REVIEW_VERSION = 1, DOMAIN_APP_REVIEW_V1

### S53 — Plugin System / Capabilities apps

- **Garde-fou : Phase A/B ne conferent aucune permission reelle.**
  Phase A = manifest declaratif (capabilities listees dans
  SBFB.json v2, parsees et validees, mais 0 enforcement runtime).
  Phase B = permission prompt UI (user consent enregistre, mais
  le bridge ne change pas). Seules Phase C (bridge etendu) et
  Phase D (CSP selective) conferent des permissions reelles. Si
  S53 deborde, on ship A+B et on defere C+D en S53.1.
- Capabilities manifest dans SBFB.json v2 : declarations
  (compute_request, storage_read, storage_write, network_peer,
  bridge_extended)
- Permission prompt UI : "App X demande compute GPU — accepter ?"
  pattern Android/iOS
- Bridge etendu : methodes additionnelles gate-par-capability
  (pas dans le bridge 3-methodes de base)
- Sandbox enforcement : iframe CSP adapte par capability
  (connect-src selective, pas blanket)
- Pas d'execution code natif — tout reste dans l'iframe sandbox
- **Dette securite active : blob-serve renderer.** Le daemon
  blob-serve decompresse et sert les zips dans le meme process
  que le broker (cf. PROCESS_ARCHITECTURE.md §broker). Cette
  surface d'attaque est reconnue et documentee dans
  RUNTIME_ISOLATION.md comme migration post-v1.0 vers un
  executor/renderer dedie. S53 ne doit PAS aggraver cette
  surface : les capabilities ne donnent jamais d'acces direct
  au filesystem ou au process broker. Tout passe par le bridge
  postMessage, le daemon valide cote serveur. Cette dette
  influence directement la qualite de S53 et doit etre visible
  dans le kickoff S53 D-decision (pas seulement dans les
  release notes S56)

### S54 — Privacy modes

- Mode 1 "visible" (defaut) : IP visible, contenu/provenance
  proteges par Ed25519, pas d'anonymat pretendu. Documentation
  claire "ce que le reseau voit de vous"
- Mode 2 "privacy experimental" : warning explicite dans le
  launcher, VPN recommande, Tor coordinator HTTP via Arti
  (feature-gate active), iroh-tor preparation (feature-gated
  non-stable). Pas promettre "anonyme" tant que pas teste
- Threat model §9 update : residual risks par mode
- UI : toggle dans settings avec avertissement

### S55 — Babel corpus / legal prep

- Babel comme premiere app vitrine SBFB
- Corpus initial : subset Gutenberg domaine public, mais modele
  evolutif multi-source (Gutenberg, Wikisource, Internet Archive,
  BnF/Gallica si source-policy valide)
- Source-policy gate obligatoire pour toute source, Gutenberg inclus :
  domaine public/licence par juridiction, redistribution P2P,
  traduisibilite, attribution, takedown/opt-out
- Pipeline traduction NLLB-200 via taches SBFB distribuees
- Registre de provenance contributive : source -> chunks -> draft LLM
  -> validations automatiques -> corrections/revues humaines ->
  attestations consensus -> traduction publiee
- Kudos par role : worker GPU, validateur automatique, traducteur /
  correcteur humain, reviewer native-speaker, temoin consensus,
  replicateur corpus
- Legal : licence corpus output, DMCA/EUCD policy, preuves
  redistribuable/traduisible inscrites dans les manifests
- Signal-testing plan (cf. babel_translation_protocol.md) :
  Newby/Kahle/NLnet/Masakhane outreach au tag v1.0

### S56 — Freeze / Audit / Tag v1.0

- Code freeze (merge freeze sauf P0/P1 hotfix)
- Docs securite finaux : THREAT_MODEL visible vs privacy modes,
  EXTERNAL_AUDIT_SCOPE update, release notes
- Tests migration complet : fresh install → onboard → deploy app →
  contribute → earn kudos → review app → trust score visible
- Smoke test reseau (VPS + machine locale + invite link)
- Security self-audit final (Semgrep sweep + OWASP check)
- Tag v1.0 sur master
- Trigger LT-2 Radicle, Babel signal-testing plan
- Post-tag : annonce README, CHANGELOG.md, release artifacts

## Carries herites qui doivent etre resolus en route

| Item | Sprint cible | Notes |
|---|---|---|
| P2-A-1 rand blocker upstream | exemption | blocker externe, re-evaluer chaque sprint |
| P2-AUDIT-2 transitives iroh | S38+ | herite pin 0.98 |
| P3-grammar executor | S40 (Tier 3) | integre dans re-run/redundancy |
| P3-watermark executor | S40 (Tier 3) | integre dans watermark_detector |
| T-NN+2 iframe Rust-wasm | post-v1.0 | PATTERNS §P34 |

## Regles

1. Chaque sprint kickoff DOIT referencer cette roadmap dans §1
   "D'ou on part" pour le tier courant
2. Si un tier est termine en avance, compacter le sprint suivant
   (trigger revalidate ci-dessus)
3. Si un blocker dep apparait (ex: ONNX Runtime Rust instable),
   re-evaluer le tier concerne et noter dans le kickoff
4. Le jalon S41 "Python supprimable" est la cible intermediaire —
   si atteint, le projet peut fonctionner 100% Rust meme si les
   routes secondaires sont encore en proxy
