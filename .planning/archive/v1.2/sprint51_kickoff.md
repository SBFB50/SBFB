# Sprint 51 — Kickoff (suppression legacy + CI post-Python + carries 2/3)

**Ecrit** : 2026-05-01 (post-audit gate S50 PASS `610b521`).
**Type** : **sprint impair** — pas de phase dette obligatoire
(§6.2.1 Regle 1). 3 items a 2/3 a resoudre (approchent 3/3
MANDATORY S52).
**Tip master d'entree** : `610b521`.
**Phase 0 audit Sprint 50** : **DEJA JOUE** — `610b521` PASS
(0 P0, 0 P1, 1 P2, 3 P3).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-05-01. 5 fichiers
  security + PATTERNS avec triggers_revalidate. 0 trigger actif
  (iroh reste 0.98, arti-client reste 0.41, wasmtime absent,
  aucun evenement externe). Pas de pre-research.

- **Technologies S51** : aucune nouvelle dep externe. Sprint
  purement soustractif (suppression legacy) + fixes carries
  existants. Aucune lib a consulter via context7.

- **ROADMAP_COMMITMENTS check** : LT-1 reclassifie pre-v1.0.
  LT-2..LT-5 latents (tag v1.0 non pose). LT-6 RESOLVED S32.
  0 condition declenchee.

- **HARDENING_ROADMAP §3** : pas de ligne S51 prescrite.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 50 CLOSED + audit PASS. Le projet est Rust+Frontend pur
depuis S50 Phase B (0 LOC Python dans packages/ ou crates/).
Mais des vestiges Python existent encore :

**Inventaire residus Python (tip `610b521`)** :
- `nexus/` : 188 fichiers traces, ~43 300 LOC Python — monolithe
  cold-case/gov/forensics original pre-pivot. Aucune dependance
  depuis le codebase SBFB actif. Decision utilisateur S50 :
  "DELETE pur en S51".
- `tests/` : 36 fichiers Python traces (~5 800 LOC) — tests du
  monolithe legacy. + 4 scripts CI smoke (tests/ci-smoke/) qui
  sont des scripts SBFB actifs (S18 attestation, pkarr, repro
  build, supply chain) a preserver.
- `worker/` : 10 fichiers Python traces (~1 700 LOC) — worker
  legacy pre-S1 (remplace par nexus-worker Rust).
- `pyproject.toml` : 39 LOC — workspace Python, plus aucun
  package actif.
- `uv.lock` : 2 416 LOC — lockfile Python, stale (P2 audit S50).
- `packages/` : 0 fichier trace (git rm S50), mais 184 fichiers
  __pycache__ non-traces sur le filesystem.

**CI workflows morts** :
- `.github/workflows/build-wheels.yml` : build maturin/PyO3 pour
  nexus-core-py — crate supprime S50.
- `.github/workflows/ci.yml` : section Python (lignes 52-80) —
  setup-python, maturin develop, ruff, 3 pytest runs sur packages
  supprimes.
- `.github/workflows/release.yml` : job "Build + attest
  nexus-core-py wheel" — crate supprime S50.

**Total a supprimer** : ~240 fichiers traces, ~72 000+ LOC,
3 workflows ou sections CI.

### §1.2 Ancrage roadmap

S50 scope cut §7 item 4 : "CI/CD + binaires + installer — S51".
Decision utilisateur S50 : "`nexus/` legacy = DELETE pur en S51".
P2-REVIEW-B-1-S50 (nexus/ legacy monolith, 1/3) → CLOSE Phase A.

### §1.3 Compteurs tests entree (tip `610b521`)

| Suite | Count |
|---|---|
| Rust nextest | 1199 |
| Rust doctests | 6 passed, 1 ignored |
| Vitest | 250 |
| Playwright | 42 + 2 fail (env pre-existant) |
| size-limit | 6/6 |
| **Total** | **~1455** |

**Post-S51 attendu** : ~1455 (aucune suppression de tests actifs,
seulement suppression de code legacy non-teste par les suites
actuelles).

### §1.4 Pre-launch protocol policy (rappel)

Sprint purement soustractif — aucun wire format touche, aucun
`*_FORMAT_VERSION` impacte.

---

## §2 Goal

Eliminer tous les vestiges Python et legacy du workspace : le
monolithe nexus/ (188 fichiers), le worker legacy, les tests
Python, les configs workspace Python, et les workflows CI morts.
Les 3 carries P2 a 2/3 sont resolus pour eviter l'escalade 3/3
MANDATORY en S52.
**Critere SMART : 15+ rows fail-fast verts au verification.md,
mesure binaire au Phase C wrap-up. 0 fichier Python actif dans
le workspace. 3 carries S48 CLOSED.**

---

## §3 Phase 0 — Audit gate S50

**DEJA JOUE** : commit `610b521` PASS (0 P0, 0 P1, 1 P2, 3 P3).
Audit findings dans `.planning/archive/v1.2/sprint50_audit_findings.md`.
7 carries documentes pour S51 (cf. §6 ci-dessous).

---

## §4 Decisions Day 0 (D1..D4 gelees)

### D1 — Suppression legacy : nexus/ + tests/ + worker/ en bloc

**Retenu** : `git rm -r nexus/ worker/` + `git rm` sur les 36
fichiers Python dans tests/ (preservant tests/ci-smoke/ qui
contient 4 scripts SBFB actifs depuis S18). Deplacer
`tests/ci-smoke/` → `scripts/ci-smoke/` (coherence avec les
autres scripts). L'historique Git conserve le code pour reference
future. Le monolithe cold-case/gov/forensics est un candidat app
SBFB post-v1.0, mais il necessite une reecriture complete — le
code actuel ne tourne pas sur le modele archive.

**Rejete** :
- Garder nexus/ dans une branche separee : complexite Git inutile,
  `git log` permet de retrouver le code a tout moment via SHA.
- Convertir en app SBFB avant delete : effort demesure (20+
  modules Python interdependants, framework server-side FastAPI),
  le modele SBFB archive est HTML-in-zip — une reecriture, pas une
  conversion.
- Supprimer seulement nexus/ et garder tests/ + worker/ : les
  tests/ importent `from nexus.*` et les worker/ importent le
  monolithe — dead code sans nexus/.

**Implications code** : `git rm -r nexus/ worker/`, `git rm`
36 fichiers Python dans tests/, `git mv tests/ci-smoke/
scripts/ci-smoke/`, mise a jour references CI.

### D2 — Workspace Python residuals : pyproject.toml + uv.lock

**Retenu** : supprimer pyproject.toml et uv.lock du tracking git.
Plus aucun package Python actif dans le workspace. Les patterns
.gitignore Python (__pycache__/, *.pyc, etc.) restent en place
(defense en profondeur, cout zero). Ajouter `packages/` au
.gitignore pour eviter que les __pycache__ residuels apparaissent
dans `git status`.

**Rejete** :
- Garder pyproject.toml pour un futur SDK Python : le modele SBFB
  est archive-based (S12). Un futur SDK serait TypeScript/WASM, pas
  Python. Si besoin, pyproject.toml se recree en 2 lignes.
- Supprimer les patterns Python de .gitignore : risque zero, cout
  zero a les garder, et des fichiers Python peuvent exister dans
  les apps tierces.

### D3 — CI workflows : suppression Python + nettoyage references

**Retenu** :
- Supprimer `.github/workflows/build-wheels.yml` (workflow
  maturin/PyO3 complet, crate nexus-core-py supprime S50).
- Nettoyer `.github/workflows/ci.yml` : supprimer la section
  Python (lignes 52-80 : setup-python, maturin develop, ruff
  format, ruff check, 3 pytest runs).
- Nettoyer `.github/workflows/release.yml` : supprimer le job
  "Build + attest nexus-core-py wheel" et les references PyO3.
- Mettre a jour les references `tests/ci-smoke/` →
  `scripts/ci-smoke/` dans les workflows.
- Modifier `supply-chain-green.sh` : supprimer le step [2/3]
  pip-audit (packages Python supprimes), garder cargo-deny [1/3]
  et audit-ci [3/3]. Le script passe de 3 audits a 2.

**Rejete** :
- Garder les workflows desactives (commented out) : dead code CI,
  risque de confusion et de false positives.
- Supprimer tous les workflows Python-related incluant supply-chain :
  supply-chain.yml est independant de Python et reste pertinent.
- Supprimer supply-chain-green.sh entierement : cargo-deny et
  audit-ci restent utiles.

### D4 — 3 carries P2 a 2/3 : resolution dans une phase dediee

**Retenu** : resoudre les 3 items dans Phase B pour eviter
escalade 3/3 MANDATORY en S52 :
- **P2-REVIEW-A-1-S48** canary reload size cap (2/3) : verifier
  que le cap existe dans le code Rust (duress_ack.rs MAX, mod.rs
  MAX_HEADLINE_LEN). Si le cap est deja implemente et teste,
  CLOSE. Si un gap persiste, fixer.
- **P2-REVIEW-B-1-S48** auth.rs set_var residuel (2/3) : les
  set_var restants sont dans du code de test (SbfbHomeGuard avec
  Mutex, pattern save/restore). Acceptables en test. Verifier qu'il
  n'y a plus de set_var en code de production hors launcher
  bootstrap (qui en a besoin structurellement). Documenter
  l'acceptation si test-only.
- **P2-AUDIT-A-1-S48** doc accuracy reload_policy (2/3) : issue
  originale sur `_reload_policy_locked` suffix trompeur dans le
  Python (S22). Le code Python est supprime depuis S50. Le Rust
  equivalent (`canary_input.rs`) utilise `reload_policy()` sans
  suffix. Verifier que la doc (PATTERNS, comments) est coherente.
  Si le gap est resolu par la suppression Python, CLOSE.

**Rejete** :
- Reporter les 3 items a S52 : ils passent a 3/3 et deviennent
  MANDATORY (§6.2.1 Regle 2). Autant les resoudre maintenant que
  le contexte est frais.
- Fusionner les carries dans Phase A avec la suppression legacy :
  melange de scope (delete massif vs micro-fixes), mauvais pour
  l'atomicite du commit.

**Acknowledged review findings (G1)** :

Scoring : D1 ✅, D2 ❌, D3 ⚠️, D4 ✅.
Rigor signal G4 satisfait (1 ⚠️ + 1 ❌ sur 4, 0 ❌ non-actionnable).

D2 ❌ (ruff depend de pyproject.toml) : le reviewer signale que
`[tool.ruff]` dans pyproject.toml est utilise par ci.yml lignes
67-71 (`uv run ruff format/check packages/ examples/`). Cependant,
ces lignes font partie de la section Python (52-80) que D3
supprime de ci.yml. Apres la suppression de la section Python CI,
ruff n'est plus invoque nulle part — pyproject.toml est bien
supprimable. Si ruff redevient necessaire post-v1.0 (ex: apps
Python sur le reseau), une config `ruff.toml` standalone sera
creee. Pas de changement de decision D2.

D3 ⚠️ (supply-chain-green.sh depend de Python) : le reviewer
signale que le step [2/3] pip-audit dans
`tests/ci-smoke/supply-chain-green.sh` audite les 3 packages
Python supprimes. Finding reel : le script casse si les packages
sont absents. Decision : Phase A modifie le script pour supprimer
la section pip-audit [2/3] (packages supprimes, rien a auditer),
en gardant cargo-deny [1/3] et audit-ci [3/3]. Le script passe
de [1/3..3/3] a [1/2..2/2]. Ajuste D3 + plan Phase A.

---

## §5 Plan Phase outline A..C

### Phase A — Suppression legacy + workspace Python cleanup

**But** : eliminer tous les fichiers Python legacy et residuels
du workspace. 0 fichier Python actif dans le tracking git.

- `git rm -r nexus/` (188 fichiers, ~43 300 LOC)
- `git rm -r worker/` (10 fichiers, ~1 700 LOC)
- `git rm` 36 fichiers Python tests/ (preserving tests/ci-smoke/)
- `git mv tests/ci-smoke/ scripts/ci-smoke/`
- `git rm pyproject.toml uv.lock`
- `git rm .github/workflows/build-wheels.yml`
- Nettoyer `.github/workflows/ci.yml` (section Python)
- Nettoyer `.github/workflows/release.yml` (job nexus-core-py)
- Mettre a jour refs `tests/ci-smoke/` → `scripts/ci-smoke/` dans
  workflows
- Ajouter `packages/` au .gitignore (cache residuel)
- Commit : `feat(sprint51): Sprint 51 Phase A — suppression legacy
  nexus/ + workspace Python cleanup + CI post-Python`

### Phase B — Carries P2 a 2/3 resolution batch

**But** : fermer les 3 items P2 a 2/3 pour eviter escalade 3/3
MANDATORY en S52.

- P2-REVIEW-A-1-S48 canary reload size cap : verifier + CLOSE si
  cap deja implemente (MAX_DURESS_ACK_MESSAGE_LEN, MAX_HEADLINE_LEN)
- P2-REVIEW-B-1-S48 auth.rs set_var residuel : audit + documentation
  acceptation test-only
- P2-AUDIT-A-1-S48 doc accuracy reload_policy : verifier coherence
  post-suppression Python + CLOSE si resolu
- Commit : `feat(sprint51): Sprint 51 Phase B — carries P2 batch
  2/3 resolution (canary cap + set_var + doc accuracy)`

### Phase C — Docs + verification + wrap-up

**But** : mettre a jour la documentation pour refleter la
suppression legacy et le workspace 100% Rust+Frontend.

- CLAUDE.md : supprimer nexus/ de la structure, supprimer
  references Python tooling (maturin, miniconda, uv workspace),
  ajuster la section Stack
- docs/claude/README.md : verifier coherence post-Python
- HARDENING_ROADMAP.md : update last_validated S51
- Verification fail-fast 15+ checks (2 blocs Rust + Frontend)
- sprint52_audit_plan.md
- Compteurs tests post-sprint
- Commit : `chore(sprint51): Phase C — wrap-up + verification +
  audit plan S52 + counters`

---

## §6 Items carry/dette

### Carries confirmes S51

- [dette] **P2-REVIEW-A-1-S48** canary reload size cap 2/3 :
  **ADRESSE Phase B** → CLOSE attendu.
- [dette] **P2-REVIEW-B-1-S48** auth.rs set_var residuel 2/3 :
  **ADRESSE Phase B** → CLOSE attendu.
- [dette] **P2-AUDIT-A-1-S48** carry doc accuracy 2/3 :
  **ADRESSE Phase B** → CLOSE attendu.
- [carry] **P2-A-1** rand blocker upstream 12+/3 : exemption
  blocker externe. Justification renouvelee : pas de release rand
  0.9 ni fix getrandom upstream.
- [carry] **P2-AUDIT-2** pre-release transitives iroh : herite
  pin 0.98 (Day 0 #3).
- [close] **P2-REVIEW-A-1-S50** dispatch join order 1/3 : carry
  incremente a 2/3.
- [close] **P2-REVIEW-B-1-S50** nexus/ legacy monolith 1/3 :
  **ADRESSE Phase A** → CLOSE attendu.

### Sprint impair — pas de phase dette obligatoire

S51 impair → pas de phase reservee dette (§6.2.1 Regle 1). Mais
3 items a 2/3 adresses proactivement dans Phase B pour eviter
escalade MANDATORY S52.

### Carries residuels post-S51

| Item | Compteur S52 | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 12+/3 | exemption |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |
| P2-REVIEW-A-1-S50 dispatch join order | 2/3 | S50 review |

---

## §7 Scope cuts

1. **Binaires release cross-platform** — S52 (prebuilt binaries
   Windows/macOS/Linux dans GitHub Releases)
2. **VPS deployment + smoke test** — S52
3. **Events SSE daemon-native** — post-v1.0
4. **MCP server Rust** — post-v1.0
5. **app-gov recreation** — post-v1.0
6. **Kudos debit/stake** — interdit (Day 0 #7)
7. **Pagination SQL-side LIMIT/OFFSET** — S52+
8. **Test infra mk_state() refactoring** — S52+

---

## §8 Tracabilite scope (S50 → S51)

| S50 scope cut | S51 disposition |
|---|---|
| CI/CD + binaires + installer — S51 | **Phase A** CI cleanup. Binaires release → S52 scope cut |
| Pagination SQL-side — S51+ | Scope cut reporte S52+ |
| Test infra mk_state() — S51+ | Scope cut reporte S52+ |

| S50 decision user | S51 disposition |
|---|---|
| nexus/ legacy = DELETE pur en S51 | **Phase A** DELETE |

---

## §9 Risk register

| # | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | CI workflows cassent apres suppression sections Python | Medium | Low | Verifier ci.yml et release.yml syntaxiquement avant commit |
| R2 | References croisees vers nexus/ dans docs SBFB | Low | Low | Grep systematique `nexus/` dans docs/ avant commit |
| R3 | ci-smoke scripts referent des chemins Python supprimes | Low | Medium | Verifier les 4 scripts ci-smoke avant/apres move |
| R4 | Carries P2 resolus par la suppression Python sans verification | Medium | Low | Verification explicite de chaque carry dans Phase B |

---

## §10 Audit gate pattern — rappel

Phase 0 S50 jouee (PASS `610b521`). Phase C produira
sprint52_audit_plan.md pour la session fraiche S52.

---

## §11 Checkpoint de validation

1. **D1** : suppression legacy en bloc ?
   → en bloc (0 dependance SBFB, historique Git suffit)
2. **D2** : pyproject.toml + uv.lock ?
   → supprimer (0 package Python actif)
3. **D3** : CI workflows ?
   → supprimer build-wheels, nettoyer ci.yml + release.yml
4. **D4** : 3 carries P2 ?
   → resolution Phase B (eviter 3/3 MANDATORY S52)
