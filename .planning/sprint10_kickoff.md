# Sprint 10 — Kickoff (release v1.0 + CI/CD + 3 VPS bootstrap)
# Decisions finales confirmees par l'utilisateur 2026-04-12

**Ecrit** : 2026-04-12, apres cloture audit gate Sprint 9
(verdict CONDITIONAL PASS leve par commits `cb610ff` + `48b332a`).

**Tip master d'entree** : `48b332a` (post `fix(sprint9): resolve
1 P0 + 6 P1 from Sprint 10 Phase 0 audit gate`).

---

## 1. Constat d'entree

### 1.1 Etat du repo

- Sprints 0-9 **CLOSED**. Sprint 10 Phase 0 audit gate (session
  fraiche jouant `sprint9_audit_plan.md`) a produit
  `sprint9_audit_findings.md` avec verdict **CONDITIONAL PASS**
  (1 P0, 6 P1, 20+ P2, 7 P3).
- Les 7 P0+P1 sont **fixes** sur master en 1 commit :
  - `48b332a fix(sprint9): resolve 1 P0 + 6 P1 from Sprint 10
    Phase 0 audit gate` — tamper bypass (D1-A P0), coordinator
    continue apres tamper (D1-B), max_size_bytes enforce (E6-B),
    Zod v1/v2 split (F4-1), semaphore wired (I2-F2), unsubscribe
    persist-first (I3-F1), coverage step verify.sh (G2-A)
- **T13-T22 PAS encore logges** dans PATTERNS.md — a faire Phase A
  (blocking gate item de l'audit)
- Pas de git remote configure — repo local uniquement
- LICENSE dit "NEXUS GOV" — a mettre a jour
- README.md partiellement mis a jour avec le pivot SBFB

### 1.2 Compteurs de tests a l'entree (tip `48b332a`)

| Suite | Count | Baseline Sprint 9 entree | Delta Sprint 9 |
|---|---|---|---|
| Rust workspace | **312** | 309 | +3 |
| Python SDK | **167** | 71 | +96 |
| Python coordinator | **83** + 1 skipped | 63 + 1 | +20 |
| Python app-gov | **46** | 30 | +16 |
| Vitest unit | **161** | 142 | +19 |
| Playwright | **27** | 24 | +3 |
| size-limit | **7/7 green** | 4/4 | +3 budgets |

Tout vert. verify.sh exit 0. Seuils coverage temporaires 85/78
(T14 tech debt).

### 1.3 Tech debt ouverte a l'entree

**Sprint 9 audit P2 — a logger (T13-T22)** :

| # | Source | Sujet | Fichier PATTERNS cible |
|---|---|---|---|
| T13 | H1-A/B/C | vendor-react/css/vendor-ui headroom fragile (<10%) | shell |
| T14 | A3-COV | FileUploadBlock.tsx couverture Vitest sous seuils 85/78 | shell |
| T15 | E3-A | SVG BOM false negative (`lstrip` ne strip pas BOM UTF-8) | shell |
| T16 | E3-B | content_type manifest client-controlled (non canonicalise) | shell |
| T17 | E6-A | `AppFileStore.open()` lit tout en memoire | shell |
| T18 | E-FLAKY | test dedup Windows flaky (`os.replace` race) | shell |
| T19 | I3-F2 | unsubscribe rollback test manquant | rust |
| T20 | C3-1 | `asyncio.wait_for` dans code anyio | shell |
| T21 | C4-1 | `useAppEvents` EventSource par mount (pas global) | shell |
| T22 | D4-A | Schema test 001_documents diverge de migration reelle | shell |

**Sprint 9 audit P2 supplementaires (non T-numerotes, documentes
dans findings)** : B2-1 (race flush), B2-2 (flush non-lifespan),
B4-1 (exception flush), B5-1 (namespace enum), C2-1 (WouldBlock
silencieux), C3-2 (test SSE disconnect), F1-1 (test naming),
F2-1 (backward compat test), F3-1 (model_dump sans mode=json),
F4-2 (renderer sans version switch), A4-SHA (SHA labels), G1-A
(sha256sum non-portable), I1-F1/F2 (probe edge cases + env var
test), I2-F1 (test semaphore backpressure), H2-B (vendor-ui
composition), H3-A (FileUploadBlock chunk).

**Tech debt anterieure ouverte** :
- T6 (Sprint 6) : renderer fuzz + chart edge-case tests
- T7 (Sprint 6) : Playwright data-testid + caplog assertion
- F-2 (Sprint 7 P3) : CommandPalette loading state

### 1.4 Contexte specifique Sprint 10

C'est le **premier sprint ops** du projet. Les 9 precedents
etaient purement code. Sprint 10 introduit :
- Deploiement sur 3 VPS distants (Linux)
- Build cross-platform (dev Windows → prod Linux)
- CI/CD pipeline (pas de remote git configure aujourd'hui)
- Release publique (PyPI, binaires)

L'adaptation du pattern : certaines phases produiront des
scripts d'infrastructure et des fichiers de configuration, pas
seulement du code applicatif. Le deploiement VPS necessite des
actions interactives de l'utilisateur (achat VPS, DNS, SSH) que
Claude ne peut pas executer directement — l'utilisateur confirme
que ces actions se feront **en parallele** du travail code.

---

## 2. Goal en une phrase

**Shipper nexus-grid v1.0 : CI/CD sur GitHub Actions, wheels PyPI
+ binaires Linux, 3 VPS bootstrap DHT operationnels, README public
pret pour l'adoption. Branding SBFB reporte a un sprint dedie.**

---

## 3. Phase 0 — Audit gate Sprint 9

**DONE** avant ce kickoff. Session fraiche a joue
`.planning/sprint9_audit_plan.md` le 2026-04-12, produisant
`sprint9_audit_findings.md` (verdict CONDITIONAL PASS) + commit
fix `48b332a` (1 P0 + 6 P1). Gate levee. Sprint 10 Phase A
peut demarrer.

---

## 4. Decisions Day 0 (D1..D7 gelees)

### D1 — Pas de branding/renommage ce sprint

**Retenu** : tout reste `nexus-*` (crates, packages, imports,
UI, docs). Le branding complet SBFB (renommage namespace, nom
produit partout, domaine, etc.) est **reporte a un sprint
dedie futur**.

**Raison** : l'utilisateur a confirme « on laisse nexus pour
tester on fera ca dans un autre sprint ». Sprint 10 se
concentre sur release + infra.

**Implications** :
- LICENSE reste « NEXUS GOV » pour l'instant (sera mis a jour
  au sprint branding)
- PyPI packages : `nexus-sdk`, `nexus-coordinator`
- README.md : mis a jour pour la release mais sans rebrand
- Pas de changement de titre UI, pas de changement de noms
  de modules

### D2 — Repo GitHub : `SBFB50/SBFB`

**Retenu** : repo public `SBFB50/SBFB` sous l'organisation
https://github.com/SBFB50. L'utilisateur cree le repo en
parallele du travail code.

Le `.git` local n'a pas de remote — ajouter `origin` en
Phase B apres creation du repo par l'utilisateur.

**Implications** :
- L'utilisateur cree le repo sur github.com/SBFB50/SBFB en
  parallele de Phase A
- Phase B ajoute le remote + premier push
- Les workflows CI/CD Phase C supposent le repo existant

### D3 — CI/CD : GitHub Actions, 3 workflows

**Retenu** : GitHub Actions avec 3 workflows :
1. **ci.yml** (push + PR) : cargo fmt/clippy/test + uv
   ruff/pytest SDK/coord/gov + npm lint/tsc/test:unit/
   test:coverage/build/size + Playwright
2. **release.yml** (tag `v*`) : build release Linux x86_64 +
   Windows x86_64 binaires (nexus-worker, nexus-shell-daemon)
   + publish wheels PyPI + GitHub Release avec assets
3. **deploy.yml** (manual trigger) : deploy sur les 3 VPS via
   SSH (binaire + restart systemd)

**Rejete** : CircleCI, Jenkins, GitLab CI. Raison : le code
est sur GitHub, Actions est natif, gratuit pour l'OSS, et le
runner Linux est disponible pour cross-compilation.

**Implications** :
- Besoin d'un `PYPI_TOKEN` secret GitHub pour publish
- Besoin de 3 secrets SSH pour deploy VPS
- Le workflow `ci.yml` doit reproduire `scripts/verify.sh` a
  l'identique

### D4 — 3 VPS : setup mixte Hetzner + Vultr, 3 regions (EU/US/Asia)

**Retenu** : setup mixte meilleur peering par region.
Budget utilisateur : jusqu'a 40 EUR/mois, pas de contrainte.

| VPS | Provider | Plan | Specs | Region | Prix/mois | Role |
|---|---|---|---|---|---|---|
| EU | Hetzner | CX32 | 3 vCPU, 8 GB, 80 GB NVMe | Falkenstein DE | 7.49 EUR | DHT + coordinator + 3 apps |
| US | Vultr | High Frequency | 2 vCPU, 4 GB, 64 GB NVMe | Chicago IL | ~11 EUR | DHT bootstrap |
| Asia | Vultr | High Frequency | 2 vCPU, 4 GB, 64 GB NVMe | Tokyo JP | ~11 EUR | DHT bootstrap |

**Total : ~29.50 EUR/mois**

Chaque VPS execute :
- `nexus-shell-daemon` (bootstrap DHT + curator pipeline)
- Identite Ed25519 persistante dans `/opt/nexus-grid/identity/`
- Systemd service `nexus-daemon.service` (auto-restart,
  watchdog 30s)
- Ubuntu 24.04, UDP non-rate-limited (requis pour QUIC/iroh)

Le VPS EU (Falkenstein) execute aussi :
- `nexus-coordinator` + 3 apps officielles (gov, coldcase,
  forensics)
- Systemd service `nexus-coordinator.service`

**Pourquoi mixte** : Hetzner a le meilleur peering EU
(DE-CIX direct), mais son datacenter US (Ashburn) est plus
recent avec un peering moins mature. Vultr High Frequency
offre le meilleur reseau US/Asia (Intel Ice Lake, 10 Gbps
uplink, NVMe, peering JPIX/BBIX en Asie). Les deux
providers autorisent UDP sans restriction et incluent la
protection DDoS.

**Rejete** :
- Hetzner full (3 regions) : peering US/Asia inferieur
- DigitalOcean : peering plus faible que Vultr HF, plus cher
- Linode/Akamai : CPU dedie overkill pour du relay P2P
- OVH : throttling trafic sortant signale

**Parallele** : l'achat des VPS se fait en parallele des
phases code. L'utilisateur fournira les IPs + cles SSH.

### D5 — Release v1.0 : tag + GitHub Release + PyPI wheels

**Retenu** : le premier tag `v1.0.0` est pose sur master
apres que toutes les phases sont green. La release contient :
- **GitHub Release** avec 4 binaires :
  `nexus-worker-linux-x86_64`,
  `nexus-shell-daemon-linux-x86_64`,
  `nexus-worker-windows-x86_64.exe`,
  `nexus-shell-daemon-windows-x86_64.exe`
- **PyPI** : `nexus-sdk==1.0.0`, `nexus-coordinator==1.0.0`
  (l'app-gov n'est pas publiee sur PyPI — elle est embarquee)
- **Web** : pas de publish npm (le shell est servi par le
  coordinator, pas un package independant)

**Rejete** : crates.io publish pour les crates Rust. Raison :
`nexus-core-rs` est une lib interne, `nexus-worker` est un
binaire distribue en release asset. Pas de dependance externe
sur ces crates.

**Implications** :
- `pyproject.toml` : version bump 0.1.0 → 1.0.0 pour SDK +
  coordinator
- `Cargo.toml` workspace : version bump → 1.0.0
- `package.json` : version bump → 1.0.0
- AGPL-3.0 SPDX headers sur tous les fichiers source

### D6 — AGPL-3.0 : SPDX header, pas de header complet par fichier

**Retenu** : ajouter un one-liner SPDX en tete de chaque
fichier source (.rs, .py, .ts, .tsx) :
```
// SPDX-License-Identifier: AGPL-3.0-or-later
```
(ou `#` pour Python). Le fichier `LICENSE` racine contient le
texte complet. Pas de bloc copyright multi-lignes dans chaque
fichier — c'est lourd, sujet a drift, et le SPDX est le
standard moderne (Linux kernel, CNCF, Fedora).

**Implications** :
- Script `scripts/check-spdx.sh` qui verifie la presence du
  header (integre dans CI)

### D7 — T14 coverage : seuils 85/78 maintenus, fix si budget

**Retenu** : les seuils temporaires de coverage (lines 85%,
branches 78%) restent en place pour Sprint 10. Si le budget le
permet en Phase E (polish), ecrire les tests FileUploadBlock
manquants et remonter a 90/85. Sinon, T14 roule en Sprint 11.

**Rejete** : bloquer le sprint sur les seuils de coverage.
Raison : Sprint 10 est un sprint ops/release, pas un sprint
qualite frontend.

---

## 5. Plan Phase outline A..F

### Phase A — Tech debt logging + release foundations (1j)

- Logger T13-T22 dans `docs/shell/PATTERNS.md` et
  `docs/rust/PATTERNS.md` (gate item audit Sprint 9)
- Ajouter SPDX headers sur tous les `.rs`, `.py`, `.ts`, `.tsx`
- Script `scripts/check-spdx.sh` (verifie presence du header)
- Version bump : Cargo.toml 0.1.0 → 1.0.0, pyproject.toml →
  1.0.0, package.json → 1.0.0
- **Commit** : `feat(release): Sprint 10 Phase A — SPDX headers
  + version 1.0.0 + T13-T22 tech debt log`

### Phase B — README + nettoyage legacy + GitHub push (1j)

- Rewrite `README.md` : pitch nexus-grid, architecture, quick
  start (install SDK, run worker, host project), contributing,
  licence. Pas de rebrand SBFB — on garde nexus-grid.
- Mettre a jour `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`,
  `SECURITY.md` pour refleter l'etat actuel du projet
- Nettoyer les fichiers legacy racine qui ne servent plus
  (`start.bat`, `start_nexus.py`, `robin.env`,
  `monitor_bench.sh`, `docker-compose.yml`, `requirements.txt`,
  `Modelfile.*`, dossiers `prompts/`, `searxng/`, `data/`,
  `logs/`, `models/`)
- `git remote add origin` + premier push sur `SBFB50/<repo>`
  (prerequis : l'utilisateur a cree le repo en parallele)
- **Commit** : `feat(docs): Sprint 10 Phase B — README release
  + nettoyage legacy + premier push GitHub`

### Phase C — CI/CD pipeline (1-2j)

- `.github/workflows/ci.yml` — reproduit verify.sh sur
  ubuntu-latest (cargo + uv + npm + Playwright)
- `.github/workflows/release.yml` — build release binaires
  (Linux + Windows) + publish PyPI wheels sur tag `v*`
- `.github/workflows/deploy.yml` — deploy VPS manual trigger
- Adapter `scripts/verify.sh` pour compatibilite CI Linux
  (paths, Playwright browser install, maturin wheel)
- **Commit** : `feat(ci): Sprint 10 Phase C — GitHub Actions
  CI/CD pipeline (ci + release + deploy)`

### Phase D — Release packaging + PyPI prep (1-2j)

- `pyproject.toml` : metadata PyPI pour nexus-sdk +
  nexus-coordinator (classifiers, description, urls, readme,
  entry_points console_scripts)
- `Cargo.toml` : metadata pour binaires release (description,
  homepage, license)
- Script `scripts/build-release.sh` : cross-compile Linux
  x86_64 binaires via `cross` ou `cargo-zigbuild`
- Test local du wheel : `uv build packages/nexus-sdk` +
  `uv build packages/nexus-coordinator`
- Dry-run PyPI : `twine check dist/*`
- **Commit** : `feat(release): Sprint 10 Phase D — release
  packaging + PyPI metadata + cross-compile scripts`

### Phase E — VPS provisioning + deployment (2-3j)

- Script `deploy/provision.sh` : setup Ubuntu 24.04
  (firewall, user nexus, dirs, systemd)
- Templates systemd : `deploy/nexus-daemon.service`,
  `deploy/nexus-coordinator.service`
- Script `deploy/deploy.sh` : upload binaires + restart
  services + smoke test
- Script `deploy/gen-identity.sh` : generer + stocker
  identite Ed25519 persistante
- Deploiement interactif avec l'utilisateur sur les 3 VPS
- Smoke test : les 3 daemons se decouvrent mutuellement via
  DHT, le coordinator sert les apps officielles
- **Commit** : `feat(deploy): Sprint 10 Phase E — VPS
  provisioning scripts + 3 bootstrap nodes live`

### Phase F — Verification + audit plan + docs update (0.5j)

- `.planning/sprint10_verification.md` — self-report fail-fast
- `.planning/sprint10_audit_plan.md` — plan d'audit Sprint 11
- Mettre a jour `docs/claude/README.md` §10 table des sprints
- Mettre a jour `docs/shell/PATTERNS.md` si nouveaux patterns
- Mettre a jour memory `nexus_grid_pivot.md` avec le tip
  post-Sprint 10
- **Commit** : `docs(sprint10): verification + audit plan for
  Sprint 11`

---

## 6. Scope cuts (a respecter strictement)

### 6.1 Ce que Sprint 10 ne livre PAS

- **Pas de branding/renommage SBFB** — reporte a un sprint
  dedie. Tout reste `nexus-*` (D1 gele)
- **Pas de rename `nexus-*` → `sbfb-*`** dans les imports,
  crates, packages
- **Pas de crates.io publish** — D5 gele, les crates Rust
  sont internes
- **Pas de npm publish** — le web shell est servi par le
  coordinator
- **Pas de multi-writer iroh-docs v1.1** — Sprint 11+
- **Pas de pinning volontaire v1.2** — Sprint 11+
- **Pas de Docker images** — les binaires statiques suffisent
  pour 3 VPS
- **Pas de monitoring/alerting** (Grafana, Prometheus) — Sprint
  11+ si needed
- **Pas de domaine custom** — les VPS sont identifies par IP +
  identite Ed25519, pas de DNS requis pour le DHT bootstrap
- **Pas de fix T6/T7** (renderer fuzz, Playwright data-testid)
  — tech debt Sprint 11+
- **Pas de fix T14** (FileUploadBlock coverage) sauf si budget
  en Phase E — sinon Sprint 11
- **Pas de cross-app events / cross-node events** — scope
  inchange depuis Sprint 9

### 6.2 Tech debt non traitee Sprint 10

Les P2 non T-numerotes de l'audit Sprint 9 restent documentes
dans `sprint9_audit_findings.md` et seront traites au fil de
l'eau quand les fichiers concernes seront touches dans des
sprints futurs. Les T13-T22 sont logges dans PATTERNS.md en
Phase A pour tracabilite.

---

## 7. Tracabilite Sprint 9 → Sprint 10 (scope cuts → pris en charge)

| Item Sprint 9 scope cut | Pris en charge par | Raison |
|---|---|---|
| Branding / renommage / docs public | **Sprint 11+ dedie** | Decision utilisateur 2026-04-12 |
| Release v1.0 / PyPI publish | **Sprint 10 Phase D + tag** | Coeur Sprint 10 |
| 3 VPS bootstrap | **Sprint 10 Phase E** | Coeur Sprint 10 |
| F-2 CommandPalette loading state | **Sprint 11+** | P3, pas de budget |
| T14 FileUploadBlock coverage | **Sprint 10 si budget** / Sprint 11 | D7 gele |

---

## 8. Audit gate pattern — rappel

- **Sprint 10 Phase 0** : DONE avant ce kickoff. A produit
  `sprint9_audit_findings.md` + 1 commit fix gate (`48b332a`).
  Gate levee.
- **Sprint 10 Phase F** : OBLIGATOIRE → doit livrer
  `.planning/sprint10_verification.md` +
  `.planning/sprint10_audit_plan.md`. Le Sprint 11 Phase 0
  jouera `sprint10_audit_plan.md` dans une session fraiche.

---

## 9. Checkpoint de validation — TOUTES DECISIONS CONFIRMEES

L'utilisateur a confirme le 2026-04-12 :

1. **D1** : pas de branding ce sprint, tout reste `nexus-*`
2. **D2** : repo = `SBFB50/SBFB` (https://github.com/SBFB50)
3. **D3** : GitHub Actions, 3 workflows
4. **D4** : Hetzner CX32 EU + Vultr HF US + Vultr HF Asia, ~29.50 EUR/mois
5. **D5** : version 1.0.0 + GitHub Release + PyPI wheels
6. **D6** : SPDX one-liner headers
7. **D7** : seuils coverage 85/78 maintenus
8. **Nettoyage legacy** : oui — supprimer `start.bat`,
   `robin.env`, `docker-compose.yml`, `Modelfile.*`,
   `prompts/`, `searxng/`, `data/`, `logs/`, `models/`,
   `start_nexus.py`, `monitor_bench.sh`, `requirements.txt`
9. **Actions paralleles** : achat VPS + creation repo GitHub
   se font en parallele du travail code

**Plan detaille pret a etre ecrit.**
