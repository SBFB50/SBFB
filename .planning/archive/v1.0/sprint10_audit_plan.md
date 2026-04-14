# Sprint 10 — Audit Plan (pour Sprint 11 Phase 0)

## Mode d'emploi pour la session fraiche

1. Lire dans cet ordre :
   - `memory/MEMORY.md` + `nexus_grid_pivot.md` + `sprint_audit_gate.md`
   - `docs/claude/README.md`
   - `.planning/sprint10_kickoff.md` (decisions Day 0)
   - `.planning/sprint10_plan.md` (plan detaille)
   - `.planning/sprint10_verification.md` (self-report)
   - Ce fichier (le plan d'audit que vous jouez)

2. **NE PAS lire** `docs/shell/PATTERNS.md` ni `docs/rust/PATTERNS.md`
   avant d'avoir forme une opinion track par track.

3. Timebox sugere : 2h. Signal prime sur volume.

4. Delivrable final : `.planning/sprint10_audit_findings.md` au format
   standard (verdict + findings P0..P3 + commits fix + P2 a logger).

---

## Track A — SPDX headers + version bump

**Question** : les 204 SPDX headers sont-ils tous presents et corrects ?
Les versions sont-elles coherentes partout ?

**Methode** :
1. `bash scripts/check-spdx.sh --count` — doit dire 204
2. Prendre 5 fichiers au hasard (2 .rs, 2 .py, 1 .tsx), verifier que
   la ligne SPDX est en position 1 ou 2 (pas apres un shebang absent)
3. `grep 'version' Cargo.toml | head -1` — doit montrer 1.0.0
4. `grep 'version' packages/nexus-sdk/pyproject.toml` — idem
5. `grep 'version' packages/nexus-coordinator/pyproject.toml` — idem
6. `grep 'version' web/package.json | head -1` — idem
7. Verifier que `Cargo.lock` est a jour (`cargo check --locked` exit 0)

**Signal** :
- P0 : version mismatch entre manifests
- P1 : SPDX header manquant sur un fichier source
- P2 : position incorrecte du header (apres shebang manquant)

---

## Track B — README + docs publiques

**Question** : le README.md est-il precis et fonctionnel ? Les liens
sont-ils corrects ? Les fichiers legacy sont-ils vraiment supprimes ?

**Methode** :
1. Lire README.md integralement — verifier que chaque commande
   fonctionne (ou fonctionnerait sur un clone frais)
2. `ls start.bat robin.env docker-compose.yml` — doivent etre absents
3. `ls prompts/ searxng/` — doivent etre absents
4. Verifier que `nexus/` existe toujours (utilise par les apps)
5. Verifier les URLs GitHub dans README, CONTRIBUTING, SECURITY

**Signal** :
- P1 : commande Quick Start cassee
- P1 : fichier legacy toujours present
- P2 : lien mort dans le README
- P3 : nit redactionnel

---

## Track C — CI/CD workflows

**Question** : les 3 workflows GitHub Actions sont-ils syntaxiquement
corrects et fonctionnellement complets ?

**Methode** :
1. Lire `.github/workflows/ci.yml` — verifier que les 18 steps
   matchent `scripts/verify.sh` dans le bon ordre
2. Verifier les actions pinned (versions @v4 etc.)
3. Lire `release.yml` — verifier la matrice build (Linux + Windows),
   le publish PyPI, la creation de release
4. Lire `deploy.yml` — verifier les secrets references, la logique
   de region, le smoke test
5. `yamllint .github/workflows/*.yml` si disponible
6. Verifier que `ci.yml` inclut le step PyO3 wheel build (maturin)

**Signal** :
- P0 : CI ne reproduit pas verify.sh (step manquant ou dans le
  mauvais ordre)
- P1 : secret reference incorrecte
- P1 : Playwright sans xvfb dans CI
- P2 : cache key suboptimale
- P3 : nit YAML formatting

---

## Track D — Release packaging

**Question** : les wheels se buildent correctement ? Les metadonnees
PyPI sont completes ?

**Methode** :
1. `uv build packages/nexus-sdk --wheel --out-dir /tmp/audit/`
2. `uv build packages/nexus-coordinator --wheel --out-dir /tmp/audit/`
3. Inspecter les wheels : `unzip -l /tmp/audit/*.whl | grep METADATA`
4. Verifier les classifiers, URLs, description dans les metadonnees
5. `scripts/build-release.sh` — lire le script, verifier la logique
   de detection de plateforme
6. Verifier que `nexus-coordinator` a un entry point console_scripts

**Signal** :
- P0 : wheel ne se build pas
- P1 : metadonnees PyPI incorrectes (mauvais nom, mauvaise license)
- P2 : classifiers manquants
- P3 : nit description

---

## Track E — Deploy scripts

**Question** : les scripts de provisioning et deployment sont-ils
fonctionnels et securises ?

**Methode** :
1. Lire `deploy/provision.sh` — verifier les commandes de securite
   (UFW rules, permissions, user creation)
2. Verifier que le service systemd a `User=nexus` (pas root)
3. Verifier les permissions sur `/opt/nexus-grid/identity/` (600)
4. Lire `deploy/deploy.sh` — verifier le flow SCP + SSH
5. Lire `deploy/gen-identity.sh` — verifier la generation de cle
6. Dry-run mental : est-ce qu'un operateur peut suivre le README
   de deploy/ et obtenir 3 VPS fonctionnels ?

**Signal** :
- P0 : service systemd tourne en root
- P1 : identite Ed25519 en permissions 644 (lisible par tous)
- P1 : firewall trop permissif (TCP all open)
- P2 : script fragile (pas de `set -euo pipefail`)
- P3 : nit documentation

---

## Track F — T13-T22 tech debt logging

**Question** : les 10 items T13-T22 sont-ils correctement documentes
dans PATTERNS.md avec les bonnes references ?

**Methode** :
1. `grep -n 'T13\|T14\|T15\|T16\|T17\|T18\|T19\|T20\|T21\|T22' docs/shell/PATTERNS.md`
2. `grep -n 'T19' docs/rust/PATTERNS.md`
3. Verifier que chaque T-item reference le bon finding de
   `sprint9_audit_findings.md`
4. Verifier qu'aucun T-item n'a ete oublie

**Signal** :
- P1 : T-item manquant
- P2 : reference au mauvais finding
- P3 : nit redactionnel

---

## Track G — Regression guard

**Question** : aucune regression n'a ete introduite par Sprint 10 ?

**Methode** :
1. `cargo test --workspace --locked` — 312 passed
2. `uv run pytest packages/nexus-sdk/tests/ -q` — 167 passed
   (T18 flaky acceptable)
3. `uv run pytest packages/nexus-coordinator/tests/ -q` — 83+1 skip
4. `uv run pytest packages/nexus-app-gov/tests/ -q` — 46 passed
5. `cd web && npm run test:unit` — 161 passed
6. `npm run size` — 7/7 green
7. `npx playwright test` — 27 passed
8. `bash scripts/check-spdx.sh` — 204 files

**Signal** :
- P0 : compteur de tests en regression
- P1 : size-limit en regression
- P2 : warning nouveau

---

## Verdict global attendu

- **PASS** : 0 P0, 0 P1 → Sprint 11 Phase A demarre direct
- **CONDITIONAL PASS** : 1-3 P1 → Sprint 11 bloque sur fix
- **FAIL** : >= 1 P0 ou >= 3 P1 → re-conception partielle

## Out of scope pour l'audit

- Les decisions D1-D7 gelees (pas de branding, Hetzner/Vultr, etc.)
- Le contenu du code applicatif (pas de nouveau code fonctionnel)
- Le deploiement VPS reel (en attente des IPs)
- Les scope cuts declares dans le kickoff §6
