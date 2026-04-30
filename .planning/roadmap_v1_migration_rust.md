<!--
written: 2026-04-29
author: FlowUP + Claude S37 wrap-up
status: ACTIVE
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

### S45 — Suppression coordinator Python + cleanup

- Supprimer `packages/nexus-coordinator/` entierement
- Supprimer `packages/nexus-app-gov/` deps Python coordinator
- Nettoyer Cargo.toml (retirer pyo3 si plus aucun binding)
- Nettoyer `pyproject.toml` workspace
- Adapter les 409 tests Python coord → Rust (ou supprimer ceux
  qui testent du code supprime)

### S46 — CI/CD + binaires + installer

- GitHub Actions multi-OS (Linux/macOS/Windows)
- Release artifacts (binaires pre-build, checksums)
- Installer script (`curl | sh` pattern)

### S47 — VPS deployment + smoke test reseau

- Premier noeud live (VPS Hetzner/OVH)
- Smoke test P2P multi-noeuds (2-3 noeuds, task submit → result)
- Monitoring baseline (uptime, latency)

### S48 — Polish UX + docs utilisateur

- README utilisateur (pas dev)
- Guide d'installation + troubleshooting
- Onboarding first-run UX (launcher double-click → reseau)
- Error messages francais/anglais
- Pas de tag v1.0 ici — S56

### S49 — Stabilisation + buffer migration

- Integration tests E2E multi-daemon (3+ noeuds)
- Cleanup carries techniques residuels (S45 dead_code etc.)
- Performance profiling baseline (latence task submit → result)
- Buffer sprint pour absorber le slippage S45-S48

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
- Corpus initial : subset Gutenberg domaine public
- Pipeline traduction NLLB-200 via taches SBFB distribuees
- Legal : verification domaine public par juridiction, licence
  corpus output, DMCA/EUCD policy
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
