# Sprint 10 — Audit Findings (Sprint 11 Phase 0)

**Auditeur** : session fraiche Claude Code 2026-04-12
**Tip audite** : `f89319c` (HEAD = `docs(sprint10): verification + audit plan for Sprint 11`)
**Commit stack Sprint 10** : `9c281d0..f89319c` (5 commits Phase A-E + 1 Phase F)
**Base d'entree Sprint 10** : `48b332a` (post audit gate Sprint 9)
**Timebox** : ~2h

---

## Verdict global : CONDITIONAL PASS

0 P0, 1 P1, 3 P2, 3 P3.

Le P1 est un finding de verification (self-report incomplet sur Playwright),
pas un defaut de code. Sprint 10 n'a introduit aucun code applicatif nouveau
et tous les compteurs de tests sont stables. Le P1 se fixe par une mise a
jour de la verification + confirmation Playwright.

---

## Track A — SPDX headers + version bump

**Verdict : PASS**

| Check | Resultat |
|---|---|
| `check-spdx.sh` | exit 0, 204 fichiers conformes |
| Sample 2 .rs | SPDX line 1 OK (`crates/nexus-core-rs/src/lib.rs`, `crates/nexus-shell-daemon/src/main.rs`) |
| Sample 1 .tsx | SPDX line 1 OK (`web/src/App.tsx`) |
| Sample 2 .py (`packages/`) | SPDX line 1 OK |
| `Cargo.toml` workspace | `version = "1.0.0"` |
| `packages/nexus-sdk/pyproject.toml` | `version = "1.0.0"` |
| `packages/nexus-coordinator/pyproject.toml` | `version = "1.0.0"` |
| `packages/nexus-app-gov/pyproject.toml` | `version = "1.0.0"` |
| `web/package.json` | `"version": "1.0.0"` |
| `pyproject.toml` racine | `version = "1.0.0"` |
| `cargo check --locked` | exit 0 (Cargo.lock a jour) |

**Finding A-1** (P2) : le scope du script `check-spdx.sh` couvre `crates/`,
`packages/`, `web/src/` (204 fichiers) mais exclut `nexus/` (legacy app code,
~30+ fichiers .py). L'intention D6 disait "chaque fichier source" ; le plan §3.2
a restraint le scope aux modules actifs. Ecart mineur car `nexus/` est du code
gele legacy, mais si le projet est AGPL-3.0, techniquement tous les .py devraient
porter le header. A traiter quand `nexus/` sera touche dans un sprint futur.

---

## Track B — README + docs publiques

**Verdict : PASS**

| Check | Resultat |
|---|---|
| README.md sections | 7 `##` headings (What is / Architecture / Quick start / Development / Test counts / Contributing / License) |
| Quick Start | Present, complet (Prerequisites, Setup, Write an app, Run a worker, Host a project) |
| Legacy fichiers tracked | `start.bat`, `start_nexus.py`, `monitor_bench.sh`, `docker-compose.yml`, `requirements.txt`, `Modelfile.*` — tous supprimes (git ls-files vide) |
| Legacy dirs `prompts/`, `searxng/` | Supprimes |
| `nexus/` | Present (utilise par apps, correct) |
| URLs GitHub | Pointent vers `https://github.com/SBFB50/SBFB/` — correct |
| CONTRIBUTING.md | A jour pour le monorepo (crates/, packages/, web/, scripts/, deploy/) |
| SECURITY.md | Complet (Ed25519, QUIC, CAS, migration runner SHA256) |

**Note sur robin.env, logs/, models/** : ces fichiers/dossiers existent
localement mais sont tous `.gitignored` (confirme par `git ls-files` vide).
Ils ne sont pas tracked dans le repo. Le commit Phase B mentionne "Deleted 8
legacy root files" mais robin.env n'etait jamais tracked — c'est un artefact
local. Pas un finding.

**Finding B-1** (P3) : README a 7 sections `##`, le plan specifait >= 8.
Le contenu est complet (Quick Start contient 5 sous-sections `###`), mais
la structure est legerement sous spec. Nit redactionnel.

---

## Track C — CI/CD workflows

**Verdict : PASS**

Les 3 workflows ont ete lus et compares a `scripts/verify.sh`.

**ci.yml** (24 steps) :
- Reproduit les 18 steps de verify.sh dans le bon ordre
- PyO3 wheel build via `maturin develop --release` avant pytest (correct)
- Playwright enveloppe dans `xvfb-run` (correct pour ubuntu headless)
- Actions pinnees (`checkout@v4`, `setup-python@v5`, `cache@v4`)
- Cache correct : cargo registry, .venv, node_modules
- `check-spdx.sh` et `scan-en-strings.sh` presents

**release.yml** :
- Trigger sur tag `v*`
- Matrice Linux + Windows pour binaires
- Publish PyPI via `uv publish` avec `UV_PUBLISH_TOKEN` / `secrets.PYPI_TOKEN`
- GitHub Release via `softprops/action-gh-release@v2`

**deploy.yml** :
- `workflow_dispatch` avec inputs (target + version)
- Matrice 3 regions (EU/US/Asia) avec roles
- SSH via secrets (VPS_*_HOST, VPS_*_SSH_KEY)
- Smoke test `systemctl is-active` apres restart

**Finding C-1** (P3) : dans deploy.yml, le restart du coordinator utilise
`|| true` comme fallback silencieux pour les VPS non-coordinator. Fonctionnel
mais la logique serait plus claire avec un `if` explicite sur le role.

---

## Track D — Release packaging

**Verdict : PASS**

| Check | Resultat |
|---|---|
| nexus-sdk pyproject.toml | Metadata complete : description, readme, license, classifiers, urls |
| nexus-coordinator pyproject.toml | Idem + `console_scripts: nexus-coordinator = nexus_coordinator.cli.main:app` |
| `uv build nexus-sdk --wheel` | exit 0 → `nexus_sdk-1.0.0-py3-none-any.whl` |
| `uv build nexus-coordinator --wheel` | exit 0 → `nexus_coordinator-1.0.0-py3-none-any.whl` |
| `scripts/build-release.sh` | `set -euo pipefail`, detection plateforme, cargo build + uv build, copie dist/ |

**Finding D-1** (P3) : `Cargo.toml` `[workspace.package]` definit `repository`
mais pas `homepage`. Non bloquant car crates.io publish est scope cut (D5),
mais si on publie un jour, le champ manquera.

---

## Track E — Deploy scripts

**Verdict : PASS avec P2**

| Check | Resultat |
|---|---|
| `provision.sh` : user nexus | `useradd --system nexus` — correct, pas root |
| `provision.sh` : `set -euo pipefail` | Present (line 12) |
| `provision.sh` : UFW | `default deny incoming` + `allow ssh` + `allow proto udp` |
| `provision.sh` : dirs | `/opt/nexus-grid/{bin,identity,data,logs}` + `chown nexus:nexus` |
| systemd daemon | `User=nexus`, `Restart=always`, `RestartSec=5`, `WatchdogSec=30` |
| systemd coordinator | `User=nexus`, `Restart=always`, `After=nexus-daemon.service` |
| `gen-identity.sh` | `dd if=/dev/urandom bs=32 count=1` + `chmod 600` — correct |
| `deploy.sh` | SCP + SSH + restart + smoke test (`systemctl is-active`) |
| `deploy.sh` : `set -euo pipefail` | Present |

**Finding E-1** (P2) : `provision.sh` ligne 47 `ufw allow proto udp from any to any`
ouvre TOUS les ports UDP. C'est necessaire pour iroh QUIC qui utilise des ports
ephemeres, mais c'est plus large que strictement requis. Un attaquant pourrait
scanner tous les services UDP du VPS. Recommandation : documenter la raison
(iroh QUIC ephemere) dans un commentaire inline et dans deploy/README.md.
Si iroh supporte un port fixe, restraindre la regle.

**Finding E-2** (P2) : `deploy/README.md` est un bon point de depart mais manque
de sections operationnelles : procedure de rollback, backup des identites,
monitoring/logs, gestion des cles SSH. Un operateur devrait pouvoir suivre le
README de A a Z pour deployer ET operer 3 VPS. Actuellement il couvre le
deploiement mais pas l'exploitation.

---

## Track F — T13-T22 tech debt logging

**Verdict : PASS**

Les 10 items ont ete verifies dans les fichiers PATTERNS.md et cross-references
avec `.planning/sprint9_audit_findings.md` :

| T-item | Fichier | Reference audit | Status |
|---|---|---|---|
| T13 | shell/PATTERNS.md | H1-A/B/C | Present, correct |
| T14 | shell/PATTERNS.md | A3-COV / G2-A | Present, correct |
| T15 | shell/PATTERNS.md | E3-A | Present, correct |
| T16 | shell/PATTERNS.md | E3-B | Present, correct |
| T17 | shell/PATTERNS.md | E6-A | Present, correct |
| T18 | shell/PATTERNS.md | E-FLAKY | Present, correct |
| T19 | rust/PATTERNS.md | I3-F2 | Present, correct |
| T20 | shell/PATTERNS.md | C3-1 | Present, correct |
| T21 | shell/PATTERNS.md | C4-1 | Present, correct |
| T22 | shell/PATTERNS.md | D4-A | Present, correct |

10/10 items, 0 manquant, references correctes.

---

## Track G — Regression guard

**Verdict : PASS (compteurs) + P1 (verification Playwright)**

| Suite | Attendu | Observe | Delta |
|---|---|---|---|
| Rust workspace | 312 | **312** | 0 |
| Python SDK | 167 | **167** | 0 |
| Python coordinator | 83 + 1 skip | **83 + 1 skip** | 0 |
| Python app-gov | 46 | **46** | 0 |
| Vitest unit | 161 | **161** | 0 |
| size-limit | 7/7 | **7/7** | 0 |
| SPDX check | 204 | **204** | 0 |
| Playwright | 27 | **0 (env blocker)** | -27 |

**Finding V-1** (P1) : la verification Sprint 10 (`sprint10_verification.md`)
a utilise le mode `--quick` pour verify.sh (row 30) et a skippe Playwright
(row 15: "(--quick skipped, 27 at Sprint 9 tip)"). Le checkpoint de cloture
affirme "30/30 fail-fast: OUI" alors que row 15 n'a jamais ete execute.

L'audit a tente de relancer Playwright : le global-setup crash car `uv run
--package nexus-coordinator` tente de remplacer `.venv/Scripts/nexus-coordinator.exe`
qui est verrouille par Windows Defender (OS error 32). C'est un blocage
environnemental Windows, pas une regression code Sprint 10. Sprint 10 n'a
modifie aucun code web (seulement des headers SPDX comment en ligne 1).

**Fix requis** : mettre a jour `sprint10_verification.md` row 15 pour indiquer
explicitement SKIPPED + raison, et corriger le checkpoint "30/30" en "29/30 + 1
SKIPPED (env blocker, aucun changement web Sprint 10)". Alternativement,
relancer Playwright apres exclusion `.venv/Scripts/` de Windows Defender.

---

## Findings list sorted by severity

| # | Track | ID | Sev | Description | Action |
|---|---|---|---|---|---|
| 1 | G | V-1 | **P1** | Verification self-report row 15 (Playwright) jamais execute, checkpoint "30/30" trompeur | Corriger verification.md + re-run si possible |
| 2 | A | A-1 | P2 | SPDX scope exclut `nexus/` legacy (~30 .py sans header) | Logger, traiter quand nexus/ touche |
| 3 | E | E-1 | P2 | `provision.sh` UDP all ports open (iroh QUIC) | Documenter rationale, restraindre si possible |
| 4 | E | E-2 | P2 | deploy/README.md manque sections operationnelles | Enrichir au fil de l'exploitation |
| 5 | B | B-1 | P3 | README 7 sections vs 8 planifiees | Nit, contenu adequat |
| 6 | D | D-1 | P3 | Cargo.toml workspace manque `homepage` | Non bloquant (crates.io scope cut) |
| 7 | C | C-1 | P3 | deploy.yml coordinator restart `|| true` silencieux | Nit, fonctionnel |

---

## Commits fix attendus

**P1 V-1** : un commit `fix(sprint10): update verification row 15 to reflect
Playwright skip` qui corrige `sprint10_verification.md` :
- Row 15 : "(--quick skipped, 27 at Sprint 9 tip)" → "SKIPPED (--quick, env blocker Windows Defender, aucun changement web Sprint 10)"
- Checkpoint : "30/30" → "29/30 + 1 SKIPPED (env blocker, no web code changes)"

---

## P2 a logger en tech debt

| # | Finding | Fichier PATTERNS cible |
|---|---|---|
| A-1 | SPDX scope nexus/ legacy | shell/PATTERNS.md (T23) |
| E-1 | UDP firewall all ports | shell/PATTERNS.md (T24) |
| E-2 | deploy README operationnel | shell/PATTERNS.md (T25) |

---

## P3 laisses sans action

- B-1 : README section count — le contenu est la, la structure est adequate
- D-1 : homepage Cargo.toml — sera ajoute si crates.io publish un jour
- C-1 : deploy.yml || true — fonctionnel, amelioration optionnelle

---

## Notes on audit completeness

- **7/7 tracks audites** conformement a `sprint10_audit_plan.md`
- **PATTERNS.md NON lus** avant formation d'opinion (respect protocole §8)
- Playwright non verifiable en live (env blocker Windows Defender) — le
  finding V-1 documente la situation. Sprint 10 n'a touche aucun code web
  applicatif (seulement SPDX headers) donc le risque de regression est
  quasi-nul
- Les fichiers robin.env, logs/, models/ existent localement mais sont
  `.gitignored` — confirme par `git ls-files`. Pas un finding repo.
- Toutes les suites compilent et passent (312 Rust, 167 SDK, 83+1 coord,
  46 gov, 161 Vitest, 7/7 size-limit, 204 SPDX)
