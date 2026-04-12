# Sprint 10 — Plan detaille (release v1.0 + CI/CD + 3 VPS bootstrap)

**Ecrit** : 2026-04-12, apres kickoff valide.
**Tip master d'entree** : `48b332a`
**Decisions Day 0** : D1-D7 gelees (cf. `sprint10_kickoff.md` §4)

---

## 1. Etat verifie a l'entree

| Suite | Count | Status |
|---|---|---|
| Rust workspace | 312 | green |
| Python SDK | 167 | green |
| Python coordinator | 83 + 1 skipped | green |
| Python app-gov | 46 | green |
| Vitest unit | 161 | green |
| Playwright | 27 | green |
| size-limit | 7/7 | green |
| `verify.sh` | exit 0 | green |

**Versions actuelles** :
- `Cargo.toml` workspace : `0.1.0`
- `packages/nexus-sdk/pyproject.toml` : `0.1.0`
- `packages/nexus-coordinator/pyproject.toml` : `0.1.0`
- `packages/nexus-app-gov/pyproject.toml` : `0.1.0`
- `web/package.json` : `0.0.0`

**Fichiers source a SPDX** : 204 total (45 .rs + 83 .py + 76 .ts/.tsx)

**Legacy a supprimer** (fichiers) : `start.bat`, `start_nexus.py`,
`robin.env`, `monitor_bench.sh`, `docker-compose.yml`,
`requirements.txt`, `Modelfile.gemma4-heretic`, `Modelfile.qwen3-30b`
**Legacy a supprimer** (dossiers) : `prompts/` (5 fichiers),
`searxng/` (2), `logs/` (1), `models/` (3)
**Note** : `data/` est deja `.gitignored` — pas dans le repo.
`nexus/` reste (utilise par les apps).

---

## 2. Decisions Day 0 (gelees — rappel synthetique)

- **D1** : pas de branding SBFB ce sprint, tout reste `nexus-*`
- **D2** : repo `SBFB50/SBFB` sur github.com
- **D3** : GitHub Actions, 3 workflows (ci/release/deploy)
- **D4** : Hetzner CX32 EU + Vultr HF US + Vultr HF Asia
- **D5** : v1.0.0, GitHub Release + PyPI wheels
- **D6** : SPDX one-liner headers
- **D7** : seuils coverage 85/78 maintenus

---

## 3. Phase A — SPDX headers + version 1.0.0 + T13-T22 tech debt log

### 3.1 T13-T22 dans PATTERNS.md

**`docs/shell/PATTERNS.md`** — ajouter apres T12 :

- **T13** — Size-limit headroom fragile. vendor-react 5.3%,
  css 4.8%, vendor-ui 8.9%. Surveiller a chaque dep ajoutee.
  Audit Sprint 9 H1-A/B/C.
- **T14** — `FileUploadBlock.tsx` couverture Vitest sous seuils
  temporaires (lines 85%, branches 78%). Ecrire tests + remonter
  a 90/85. Sprint 9 A3-COV + G2-A.
- **T15** — SVG BOM UTF-8 false negative. `lstrip()` ne strip
  pas `\xef\xbb\xbf`. Sprint 9 E3-A.
- **T16** — `content_type` dans manifest CAS est client-controlled
  (header multipart). Canonicaliser post-magic-bytes. Sprint 9 E3-B.
- **T17** — `AppFileStore.open()` lit tout en memoire avant de
  chunker. Borne par max_size_bytes (50 MB). Sprint 9 E6-A.
- **T18** — `test_concurrent_store_same_sha256_dedup_safe` flaky
  Windows. `os.replace` race. Retry avec backoff. Sprint 9 E-FLAKY.
- **T20** — `asyncio.wait_for()` dans SSE generator (code anyio).
  Remplacer par `anyio.fail_after()`. Sprint 9 C3-1.
- **T21** — `useAppEvents` cree un EventSource par mount composant.
  Extraire en singleton. Sprint 9 C4-1.
- **T22** — Schema test `test_gov_documents.py` diverge de
  `001_documents.sql` (noms de colonnes). Sprint 9 D4-A.

**`docs/rust/PATTERNS.md`** — ajouter apres la section Sprint 7 :

- **T19** — `unsubscribe` rollback test manquant dans
  `iroh_runtime.rs`. Ajouter test `persist_subscriptions` failure
  path pour unsubscribe. Sprint 9 I3-F2.

### 3.2 SPDX headers

Script `scripts/check-spdx.sh` :
- Glob `crates/**/*.rs`, `packages/**/*.py` (excl. `__pycache__`),
  `web/src/**/*.{ts,tsx}`
- Verifie que la 1ere ou 2e ligne contient
  `SPDX-License-Identifier: AGPL-3.0-or-later`
- Exit 1 avec la liste des fichiers non-conformes

Ajouter le header a chaque fichier source (204 fichiers) :
- `.rs` : `// SPDX-License-Identifier: AGPL-3.0-or-later`
- `.py` : `# SPDX-License-Identifier: AGPL-3.0-or-later`
- `.ts`/`.tsx` : `// SPDX-License-Identifier: AGPL-3.0-or-later`

Integrer `check-spdx.sh` dans `verify.sh` comme step 18.

### 3.3 Version bump 1.0.0

- `Cargo.toml` : `version = "0.1.0"` → `"1.0.0"` (workspace)
- `Cargo.toml` : `repository` → `"https://github.com/SBFB50/SBFB"`
- `packages/nexus-sdk/pyproject.toml` : `version = "0.1.0"` → `"1.0.0"`
- `packages/nexus-coordinator/pyproject.toml` : idem
- `packages/nexus-app-gov/pyproject.toml` : idem
- `pyproject.toml` racine : `version = "0.2.0"` → `"1.0.0"`
- `web/package.json` : `"version": "0.0.0"` → `"1.0.0"`

### 3.4 Critere d'acceptation Phase A

- `scripts/check-spdx.sh` exit 0 (204 fichiers conformes)
- `scripts/verify.sh` exit 0 (17 steps + step 18 SPDX)
- Tous les compteurs de tests inchanges
- T13-T22 visibles dans PATTERNS.md
- Versions 1.0.0 dans les 7 manifests

### 3.5 Commit cible

```
feat(release): Sprint 10 Phase A — SPDX headers + version 1.0.0 + T13-T22 tech debt log

- SPDX one-liner AGPL-3.0-or-later on 204 source files (45 .rs + 83 .py + 76 .ts/.tsx)
- scripts/check-spdx.sh guard integrated as verify.sh step 18
- Version bump: Cargo.toml workspace 0.1.0 → 1.0.0, 4 pyproject.toml → 1.0.0,
  package.json → 1.0.0
- Cargo.toml repository URL → https://github.com/SBFB50/SBFB
- T13-T22 tech debt logged in docs/shell/PATTERNS.md + docs/rust/PATTERNS.md
  (Sprint 9 audit gate P2 items)

Test delta: unchanged (312 Rust / 167 SDK / 83+1 coord / 46 gov / 161 Vitest / 27 Playwright)
Scope cuts honoured: no branding, no rename, no code changes
```

---

## 4. Phase B — README release + nettoyage legacy + GitHub push

### 4.1 README.md rewrite

Structure cible :
1. Titre + one-liner (nexus-grid — P2P LLM compute network)
2. Badges (CI, license, PyPI version)
3. Pitch court (5 lignes)
4. Quick start : `pip install nexus-sdk` + hello-world
5. Architecture (diagramme ASCII, Rust + Python + React)
6. Run a worker : download binary + `nexus-worker start`
7. Host a project : `nexus-coordinator init && start`
8. Contributing (pointer vers CONTRIBUTING.md)
9. License AGPL-3.0

### 4.2 Nettoyage legacy

Supprimer les fichiers racine :
- `start.bat`, `start_nexus.py`, `robin.env`
- `monitor_bench.sh`, `docker-compose.yml`, `requirements.txt`
- `Modelfile.gemma4-heretic`, `Modelfile.qwen3-30b`

Supprimer les dossiers :
- `prompts/` (5 fichiers, prompts legacy cold-case)
- `searxng/` (2 fichiers, config SearXNG locale)
- `logs/` (1 fichier)
- `models/` (3 fichiers, configs Ollama)

**Garder** : `nexus/` (utilise par les apps), `data/` (gitignored),
`examples/`, `docs/`

### 4.3 Mise a jour docs supplementaires

- `CONTRIBUTING.md` : instructions pour le monorepo Rust + Python
- `CODE_OF_CONDUCT.md` : verifier qu'il est a jour
- `SECURITY.md` : point de contact + politique disclosure

### 4.4 GitHub push

Prerequis : l'utilisateur a cree le repo `SBFB50/SBFB` en
parallele.

```bash
git remote add origin https://github.com/SBFB50/SBFB.git
git push -u origin master
```

Si le repo n'est pas encore cree au moment de Phase B, le push
est differe a Phase C sans bloquer le commit.

### 4.5 Critere d'acceptation Phase B

- README.md reecrit avec les 9 sections
- 8 fichiers legacy + 4 dossiers legacy supprimes
- `verify.sh` exit 0 (pas de regression)
- remote `origin` configure (si repo dispo)

### 4.6 Commit cible

```
feat(docs): Sprint 10 Phase B — README release + nettoyage legacy racine

- README.md rewritten: pitch, quick start, architecture, contributing
- Deleted 8 legacy root files: start.bat, start_nexus.py, robin.env,
  monitor_bench.sh, docker-compose.yml, requirements.txt, Modelfile.* 
- Deleted 4 legacy directories: prompts/, searxng/, logs/, models/
- Updated CONTRIBUTING.md, SECURITY.md for monorepo workflow
- git remote add origin https://github.com/SBFB50/SBFB.git

Test delta: unchanged
Scope cuts honoured: no branding, nexus/ kept (app dependency)
```

---

## 5. Phase C — GitHub Actions CI/CD pipeline

### 5.1 `.github/workflows/ci.yml`

Trigger : push to master, pull_request.

Job `test` sur `ubuntu-latest` :
1. checkout
2. Setup Rust (rustup, cargo-cache)
3. Setup Python 3.13 (uv)
4. Setup Node 20
5. `cargo fmt --all --check`
6. `cargo clippy --workspace --all-targets --locked -- -D warnings`
7. `cargo test --workspace --locked`
8. Build nexus-core-py wheel : `maturin build --release -m crates/nexus-core-py/Cargo.toml`
9. Install wheel in venv : `uv pip install target/wheels/nexus_core-*.whl`
10. `uv run ruff format --check packages/ examples/`
11. `uv run ruff check packages/ examples/`
12. `uv run pytest packages/nexus-sdk/tests/ -q`
13. `uv run pytest packages/nexus-coordinator/tests/ -q`
14. `uv run pytest packages/nexus-app-gov/tests/ -q`
15. `cd web && npm ci`
16. `npx tsc --noEmit -p tsconfig.app.json`
17. `npm run lint`
18. `npm run test:unit`
19. `npm run test:coverage`
20. `npm run build`
21. `npm run size`
22. Playwright install + test
23. `bash scripts/scan-en-strings.sh`
24. `bash scripts/check-spdx.sh`

Cache : `~/.cargo/registry`, `~/.cargo/git`, `target/`,
`.venv/`, `web/node_modules/`

### 5.2 `.github/workflows/release.yml`

Trigger : push tag `v*`

Job matrix `build` :
- `linux-x86_64` : `ubuntu-latest`, cibles `nexus-worker` +
  `nexus-shell-daemon`, `cargo build --release`
- `windows-x86_64` : `windows-latest`, idem

Job `publish-pypi` (needs `build`) :
- Build wheels nexus-sdk + nexus-coordinator via `uv build`
- `uv publish` avec `PYPI_TOKEN`

Job `release` (needs `build`, `publish-pypi`) :
- Create GitHub Release avec les 4 binaires en assets

### 5.3 `.github/workflows/deploy.yml`

Trigger : `workflow_dispatch` avec inputs (which VPS: eu/us/asia/all)

Job `deploy` :
- Download latest release binaires
- SSH vers le(s) VPS via secrets
- Upload binaires dans `/opt/nexus-grid/bin/`
- `sudo systemctl restart nexus-daemon`
- Smoke test : curl health endpoint

### 5.4 Adaptation verify.sh

- `scripts/setup.sh` : detecter CI (`CI=true` env) et adapter
  les paths (pas de conda, pas de `.venv` existant)
- `scripts/verify.sh` : fonctionnel tel quel sur Linux si les
  toolchains sont installees

### 5.5 Critere d'acceptation Phase C

- Les 3 workflows existent dans `.github/workflows/`
- `verify.sh` exit 0 localement
- Push sur GitHub + CI passe (si repo dispo)

### 5.6 Commit cible

```
feat(ci): Sprint 10 Phase C — GitHub Actions CI/CD pipeline

- .github/workflows/ci.yml: 24-step verification on ubuntu-latest
  (Rust + Python + Web + Playwright + SPDX check)
- .github/workflows/release.yml: build Linux + Windows binaries,
  publish PyPI wheels, create GitHub Release on tag v*
- .github/workflows/deploy.yml: manual VPS deployment via SSH
- Caching: cargo registry, .venv, node_modules

Test delta: unchanged
Scope cuts honoured: no Docker, no monitoring
```

---

## 6. Phase D — Release packaging + PyPI metadata

### 6.1 PyPI metadata

`packages/nexus-sdk/pyproject.toml` :
- `description`, `readme`, `license`, `classifiers`
- `urls` : homepage, repository, documentation
- `[project.optional-dependencies]` si applicable

`packages/nexus-coordinator/pyproject.toml` :
- idem + `[project.scripts]` console entry points :
  `nexus-coordinator = "nexus_coordinator.cli:app"`

### 6.2 Cargo.toml metadata

`Cargo.toml` workspace :
- `description` pour chaque crate
- `homepage = "https://github.com/SBFB50/SBFB"`
- `license = "AGPL-3.0-or-later"` (corrige depuis `AGPL-3.0`)

### 6.3 Cross-compilation scripts

`scripts/build-release.sh` :
- Detecte l'OS (Linux natif / Windows cross)
- `cargo build --release -p nexus-worker -p nexus-shell-daemon`
- Si CI Linux : build natif direct
- Si Windows local : instructions pour `cross` ou `cargo-zigbuild`
- Copie les binaires dans `dist/`

### 6.4 Test wheels

```bash
uv build packages/nexus-sdk --wheel
uv build packages/nexus-coordinator --wheel
twine check dist/*.whl  # ou uv publish --check
```

### 6.5 Critere d'acceptation Phase D

- `uv build` produit des wheels valides pour SDK + coordinator
- `twine check` passe sur les wheels
- `scripts/build-release.sh` produit les binaires locaux
- `verify.sh` exit 0

### 6.6 Commit cible

```
feat(release): Sprint 10 Phase D — PyPI metadata + release packaging

- nexus-sdk + nexus-coordinator pyproject.toml: full PyPI metadata
  (classifiers, urls, readme, console_scripts)
- Cargo.toml: updated homepage, description, license SPDX
- scripts/build-release.sh: native + cross-compile release builder
- Wheels validated: uv build + twine check

Test delta: unchanged
Scope cuts honoured: no crates.io, no npm, no Docker
```

---

## 7. Phase E — VPS provisioning + 3 bootstrap nodes

### 7.1 Scripts de provisioning

`deploy/provision.sh` (a executer sur chaque VPS via SSH) :
1. `apt update && apt upgrade -y`
2. Creer user `nexus` (non-root)
3. `mkdir -p /opt/nexus-grid/{bin,identity,data,logs}`
4. Configurer UFW : allow SSH (22), allow UDP (iroh QUIC)
5. Installer les binaires depuis GitHub Release (ou upload)
6. Copier les templates systemd

`deploy/gen-identity.sh` :
- Genere une keypair Ed25519 persistante pour le daemon
- Stocke dans `/opt/nexus-grid/identity/`
- Affiche le node ID public

### 7.2 Templates systemd

`deploy/nexus-daemon.service` :
```ini
[Unit]
Description=nexus-grid shell daemon (DHT bootstrap)
After=network-online.target

[Service]
Type=simple
User=nexus
ExecStart=/opt/nexus-grid/bin/nexus-shell-daemon start
WorkingDirectory=/opt/nexus-grid
Restart=always
RestartSec=5
WatchdogSec=30

[Install]
WantedBy=multi-user.target
```

`deploy/nexus-coordinator.service` (VPS EU seulement) :
```ini
[Unit]
Description=nexus-grid coordinator (official apps)
After=network-online.target nexus-daemon.service

[Service]
Type=simple
User=nexus
ExecStart=/opt/nexus-grid/bin/nexus-coordinator-start.sh
WorkingDirectory=/opt/nexus-grid
Restart=always
RestartSec=5
Environment=NEXUS_GRID_ROOT=/opt/nexus-grid/data

[Install]
WantedBy=multi-user.target
```

`deploy/nexus-coordinator-start.sh` :
- `cd /opt/nexus-grid/data`
- `nexus-coordinator init` (si premier boot)
- `nexus-coordinator start`

### 7.3 Script de deployment

`deploy/deploy.sh` :
- Arguments : `--host <ip>` `--key <ssh_key>` `--role daemon|coordinator`
- Upload binaires via scp
- Restart systemd service
- Smoke test : `curl localhost:port/health` via SSH

### 7.4 Deploiement interactif

L'utilisateur fournit les 3 IPs + cles SSH. Le deploiement
se fait en session interactive :
1. Provisionner chaque VPS via `provision.sh`
2. Generer les identites Ed25519
3. Deployer les binaires
4. Verifier : les 3 daemons se decouvrent via DHT

### 7.5 Hardcoded bootstrap peers

Les 3 node IDs publics des VPS sont hardcodes dans
`crates/nexus-shell-daemon-core/src/config.rs` comme
bootstrap peers par defaut. Un nouveau noeud qui demarre
contacte ces 3 peers pour rejoindre le DHT.

Configuration : fichier `deploy/bootstrap-peers.json` avec
les 3 node IDs + adresses. Le daemon lit ce fichier au
demarrage.

### 7.6 Critere d'acceptation Phase E

- 3 scripts dans `deploy/` (provision, deploy, gen-identity)
- 2 templates systemd
- Scripts testes en dry-run localement
- Si VPS disponibles : 3 daemons live, decouverte mutuelle OK
- `verify.sh` exit 0

### 7.7 Commit cible

```
feat(deploy): Sprint 10 Phase E — VPS provisioning + bootstrap peers

- deploy/provision.sh: Ubuntu 24.04 setup (user, dirs, firewall, systemd)
- deploy/deploy.sh: binary upload + service restart + smoke test
- deploy/gen-identity.sh: Ed25519 persistent keypair generator
- deploy/nexus-daemon.service + nexus-coordinator.service: systemd templates
- deploy/nexus-coordinator-start.sh: init + start wrapper
- Bootstrap peers config for 3 VPS (Hetzner EU + Vultr US + Vultr Asia)

Test delta: unchanged (deploy scripts are infrastructure, not app code)
Scope cuts honoured: no Docker, no monitoring, no domain
```

---

## 8. Phase F — Verification + audit plan

### 8.1 Livrables

- `.planning/sprint10_verification.md` — checklist fail-fast
  remplie avec colonnes Observed
- `.planning/sprint10_audit_plan.md` — plan pour Sprint 11
  Phase 0

### 8.2 Mises a jour

- `docs/claude/README.md` §10 : ajouter Sprint 10 dans la table
- `docs/shell/PATTERNS.md` : nouveaux patterns si applicable
  (P15 CI/CD, P16 VPS deployment)
- Memory `nexus_grid_pivot.md` : update tip, compteurs, Sprint 10
  summary

### 8.3 Commit cible

```
docs(sprint10): verification + audit plan for Sprint 11

- .planning/sprint10_verification.md: N/N fail-fast checklist
- .planning/sprint10_audit_plan.md: tracks for Sprint 11 Phase 0
- docs/claude/README.md §10: Sprint 10 added to cross-reference table
- Memory nexus_grid_pivot.md updated
```

---

## 9. Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | exit 0 | |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warning | |
| 3 | cargo test | `cargo test --workspace --locked` | 312 passed | |
| 4 | ruff format | `uv run ruff format --check packages/ examples/` | exit 0 | |
| 5 | ruff check | `uv run ruff check packages/ examples/` | exit 0 | |
| 6 | pytest SDK | `uv run pytest packages/nexus-sdk/tests/ -q` | 167 passed | |
| 7 | pytest coord | `uv run pytest packages/nexus-coordinator/tests/ -q` | 83 passed + 1 skipped | |
| 8 | pytest gov | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 passed | |
| 9 | tsc | `cd web && npx tsc --noEmit -p tsconfig.app.json` | exit 0 | |
| 10 | eslint | `npm run lint` | 0 errors | |
| 11 | vitest | `npm run test:unit` | 161 passed | |
| 12 | coverage | `npm run test:coverage` | lines >= 85%, branches >= 78% | |
| 13 | build | `npm run build` | exit 0, no warnings | |
| 14 | size-limit | `npm run size` | 7/7 green | |
| 15 | playwright | `npx playwright test` | 27 passed | |
| 16 | scan-en | `bash scripts/scan-en-strings.sh` | exit 0 | |
| 17 | npm audit | `npm audit --audit-level=high` | 0 high/crit | |
| 18 | SPDX check | `bash scripts/check-spdx.sh` | exit 0 (204 files) | |
| 19 | versions | grep version in 7 manifests | all = 1.0.0 | |
| 20 | SPDX header count | `scripts/check-spdx.sh --count` | 204 | |
| 21 | legacy removed | `ls start.bat robin.env` | file not found | |
| 22 | README sections | grep headings README.md | >= 8 sections | |
| 23 | CI workflow exists | `ls .github/workflows/ci.yml` | exists | |
| 24 | release workflow | `ls .github/workflows/release.yml` | exists | |
| 25 | deploy workflow | `ls .github/workflows/deploy.yml` | exists | |
| 26 | wheel SDK | `uv build packages/nexus-sdk --wheel` | exit 0 | |
| 27 | wheel coord | `uv build packages/nexus-coordinator --wheel` | exit 0 | |
| 28 | deploy scripts | `ls deploy/provision.sh deploy/deploy.sh` | exist | |
| 29 | systemd templates | `ls deploy/nexus-daemon.service` | exists | |
| 30 | verify.sh full | `./scripts/verify.sh` | exit 0 | |

---

## 10. Git plan

| # | Phase | Commit |
|---|---|---|
| 1 | A | `feat(release): Sprint 10 Phase A — SPDX headers + version 1.0.0 + T13-T22 tech debt log` |
| 2 | B | `feat(docs): Sprint 10 Phase B — README release + nettoyage legacy racine` |
| 3 | C | `feat(ci): Sprint 10 Phase C — GitHub Actions CI/CD pipeline` |
| 4 | D | `feat(release): Sprint 10 Phase D — PyPI metadata + release packaging` |
| 5 | E | `feat(deploy): Sprint 10 Phase E — VPS provisioning + bootstrap peers` |
| 6 | F | `docs(sprint10): verification + audit plan for Sprint 11` |

---

## 11. Scope cuts (copie kickoff §6)

- Pas de branding/renommage SBFB
- Pas de crates.io publish
- Pas de npm publish
- Pas de Docker images
- Pas de monitoring/alerting
- Pas de domaine custom
- Pas de fix T6/T7/T14 (sauf si budget)
- Pas de cross-app/cross-node events

---

## 12. Risks

| # | Risk | Impact | Mitigation |
|---|---|---|---|
| R1 | Cross-compilation Windows → Linux echoue | Bloque release.yml | Fallback : build natif dans le CI GitHub (ubuntu runner) |
| R2 | Repo GitHub non cree au moment de Phase B | Bloque push + CI | Phase B commit local, push differe. CI teste en Phase C post-push |
| R3 | VPS non disponibles au moment de Phase E | Phase E partielle | Scripts de deploy commites, deploiement reel en session suivante |
| R4 | PyPI name `nexus-sdk` deja pris | Bloque publish | Verifier avant Phase D. Fallback : `nexus-grid-sdk` |
| R5 | Playwright flaky en CI (no display) | CI rouge | `xvfb-run` wrapper dans ci.yml |
| R6 | maturin wheel build echoue en CI | Bloque tests Python | Build wheel comme step separe, cacher le resultat |
| R7 | 204 SPDX headers cassent un parser | Tests rouges | Ajouter apres shebang/encoding line, pas avant |

---

## 13. Checkpoint de cloture

Le sprint est ferme quand :
1. 30/30 fail-fast checklist green
2. 6 commits atomiques landed sur master
3. `sprint10_verification.md` + `sprint10_audit_plan.md` ecrits
4. PATTERNS.md a jour (T13-T22 + nouveaux patterns)
5. Memory `nexus_grid_pivot.md` mise a jour
6. CI green sur GitHub (si repo dispo)
7. (Optionnel) 3 VPS live avec decouverte mutuelle
