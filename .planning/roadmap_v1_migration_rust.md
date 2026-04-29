<!--
written: 2026-04-29
author: FlowUP + Claude S37 wrap-up
status: ACTIVE
triggers_revalidate:
  - "Tier N complete ahead of schedule → compacter sprints suivants"
  - "Nouveau blocker dep (ex: ONNX Rust binding instable) → re-evaluer tier"
-->

# Roadmap migration Python coordinator → Rust natif (S38-S48)

Decision utilisateur 2026-04-29 post-S37 wrap-up. Le coordinator
Python (`packages/nexus-coordinator/`) est progressivement remplace
par le crate Rust `nexus-coordinator-rs` wire dans le daemon. Le
core path (submit → validate → kudos) est livre S35-S37. Cette
roadmap planifie la suite jusqu'a suppression Python + tag v1.0.

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

### S48 — Polish UX + docs + tag v1.0

- README utilisateur (pas dev)
- Guide d'installation + troubleshooting
- Onboarding first-run UX
- Tag `v1.0`
- Trigger LT-2 Radicle, Babel signal-testing plan

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
