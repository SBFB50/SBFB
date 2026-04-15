# Sprint 18 Phase E3 — nexus-phase-auditor review

**HEAD pre-commit** : `04c96214ebad40fff71744074ca23bbdd432edf4` (post-E2)
**Draft commit body** : `feat(sprint18): Phase E3 — Codeberg private disaster-recovery mirror`
**Timebox** : ~45m (couvert dans les 50 tool uses ; analyse complete sur les 10 dimensions, redaction sous deadline)

---

## Verdict : PASS

0 finding P0. 0 finding P1. 4 findings P2 (deviations doc + edge `git push --mirror` destructif silencieux + landing spot Radicle-v1.0 tracking). 3 findings P3 (SPDX header + cohabitation orgs + concurrency burst).

Le diff Phase E3 est un sprint **ops/docs pur** (1 workflow YAML 50 LOC + 1 doc 278 lignes + mises a jour planning) qui livre exactement le scope annonce post-pivot Radicle→Codeberg. Aucun code Rust/Python/JS modifie. Tests `cargo test --workspace --locked` = **474 passing** (identique baseline post-E2, 0 regression). YAML valide, secrets correctement injectes via env, auth via `http.extraheader` (anti URL-credential-leak), `git push --mirror` documente comme "vrai mirror", permissions GHA minimales `contents: read`. Cohabitation avec les 8 workflows existants OK (concurrency group dedie). Pivot Radicle→Codeberg pleinement trace dans plan.md §Phase E3 (block "Pivot 2026-04-15" dedie + flip-sequence v1.0 self-contained dans MIRROR_FALLBACK.md §3).

---

## Dimensions

### Security

- [x] **Pas de credentials hardcodes** : `git diff --cached | grep -iE "(AKIA|ghp_|gho_|ghu_|ghs_|pat_|sbfb_|BEGIN.*PRIVATE KEY)"` → 0 match.
- [x] **Token injection via `http.extraheader`** : le workflow utilise `git -c "http.https://codeberg.org/.extraheader=Authorization: token ${CODEBERG_TOKEN}"` plutot qu'un URL-embedded `https://user:token@codeberg.org/...`. C'est le pattern recommande pour eviter la fuite via process listing, history shell, et logs git verbose. Conforme.
- [x] **Pas de `set -x` ni echo du token** : `set -eu` (non `set -euo pipefail` — minor inconsistence vs `canary-monthly.yml` qui utilise pipefail, P3). La commande expandue n'est jamais print. GHA mask automatique des secrets en backstop.
- [x] **Permissions GHA minimales** : `permissions: contents: read` au niveau workflow. Le workflow ne peut pas modifier le repo source. Conforme au pattern de tous les workflows S18 (`canary-monthly.yml`, `supply-chain.yml`, `rust-ci.yml`).
- [x] **Scope token Codeberg `repository` Read+Write only** : documente §4 MIRROR_FALLBACK.md, justifie pour push-mirror. Pas de scope `org/issue/user/package`. Moindre privilege correct pour la fonction.
- [x] **Pin action externe** : seule action utilisee = `actions/checkout@v4` (tag mutable mais **official GitHub action**, et **convention projet uniforme** sur les 8 workflows existants — tous `@v4`). Le diff n'introduit aucune action third-party non-officielle. Pas un finding (le workflow Radicle futur, lui, pin SHA — bonne pratique anticipee).
- [x] **Guard secret missing** : `if [ -z "${CODEBERG_TOKEN:-}" ]; then echo "::error::..."; exit 1; fi` — fail-fast propre, le message d'erreur n'inclut PAS le token (juste "missing"). Bon design.
- [x] **YAML valide** : `python -c "import yaml; yaml.safe_load(...)"` → OK. Note : `on:` parse comme `True` (YAML 1.1 boolean ambiguity) mais GHA lit le fichier source texte, comportement attendu.
- [x] **`fetch-depth: 0` justifie** : `git push --mirror` requiert TOUS les refs et l'historique complet. Sans `fetch-depth: 0`, checkout fait shallow clone (depth 1) qui ne contiendrait pas les autres branches. Justifie.
- [x] **Origin check / DNS rebinding** : N/A (workflow GHA, pas surface loopback).

**Edge case identifie (P2)** : `git push --mirror` **supprime silencieusement les refs presentes cote Codeberg mais absentes cote GitHub**. Comportement desire (mirror strict) MAIS NON DOCUMENTE dans MIRROR_FALLBACK.md (le plan.md §Decisions techniques le mentionne ligne 844 : "delete refs absentes sur source = vrai mirror" ; la doc user-facing pas). Risque : un maintainer qui cree une branche manuellement cote Codeberg verra sa branche wipee au prochain push GitHub sans avertissement.

### Patterns

- [x] **Format YAML coherent autres workflows** : structure (name, on, permissions, concurrency, jobs, steps) suit le meme pattern que `canary-monthly.yml`, `supply-chain.yml`, `rust-ci.yml`. Header de commentaire rationale conforme au style des autres workflows S18.
- [x] **Concurrency group** : pattern deja present dans 3 autres workflows (`rust-ci.yml`, `build-wheels.yml`, `build-worker.yml`). Le nouveau utilise `group: mirror-codeberg` (global, sans `${{ github.ref }}`) + `cancel-in-progress: false` — **divergence intentionnelle et justifiee** (serialiser GLOBALEMENT les pushes mirror, pas par branche, sinon race cote Codeberg sur `git push --mirror` concurrent). Documente dans plan.md §Decisions techniques.
- [x] **`timeout-minutes: 15`** : safety net coherent avec autres workflows S18 (canary-monthly = 10, supply-chain = 15, release = N/A).
- [x] **Structure doc MIRROR_FALLBACK.md coherente** : §1 Rationale + §2 Usage + §3 Procedure + §4-§7 reference, tres similaire a REPRODUCIBLE_BUILDS.md (§1 Verifier SHA256, §2 Verifier attestation, etc.). Pas de drift de style.

**Pattern drift detecte (P3)** : `mirror-codeberg.yml` n'a PAS de SPDX header `# SPDX-License-Identifier: AGPL-3.0-or-later` alors que **6/9 workflows en ont** (canary-monthly, ci, deploy, release, supply-chain + 1 autre). Inconsistance mineure — le projet n'a pas de regle stricte (build-wheels, build-worker, rust-ci n'en ont pas non plus), mais tous les workflows S18 recemment ajoutes (canary-monthly, supply-chain, release) en ont. P3 trivial fix.

### Scope-cuts

Scan exhaustif sur les keywords §6 du kickoff :
```
iroh-audit, pyodide-escape, PoW-gossip, encryption-at-rest,
tls-pinning, pkarr-relay, ONG-relays, PQC, ML-DSA, ML-KEM
```
**Resultat : zero match dans le diff.** `git diff --cached | grep -iE "..."` → "ZERO match scope-cut keywords". Conforme.

### Tests-delta

- [x] **Plan annonce** : 0 nouveau test (ops CI pur).
- [x] **Reel mesure** : `cargo test --workspace --locked 2>&1 | grep "^test result:" | awk -summing` → **474 passing** (baseline post-E2 inchange). Delta = **+0**. Correspond exactement.
- [x] **Aucun fichier code modifie** : `git diff --cached --name-only` → 4 fichiers (`.github/workflows/mirror-codeberg.yml`, `sprint18_kickoff.md`, `sprint18_plan.md`, `docs/release/MIRROR_FALLBACK.md`). 0 fichier `.rs`, `.py`, `.ts`, `.tsx`. Confirme.
- [x] **`cargo fmt --all --check`** : OK (pas de code Rust modifie, mais verifie pour completude).
- [x] **Suites Python / JS / Playwright** : non-relancees (Phase E3 ops-only ne peut pas casser ces suites — aucun fichier source touche). Verification standard `docs/claude/README.md §7.4` partielle justifiee.

### Research-grounding (pivot Radicle → Codeberg)

- [x] **Pivot trace dans plan.md** : block dedie ligne 801 "### Pivot 2026-04-15 (Radicle → Codeberg + Radicle differe)" avec 3 raisons numerotees + recherche deep mentionnee. Auditeur S19 lisant uniquement le plan comprend immediatement E3=Codeberg.
- [x] **Pivot trace dans kickoff.md** : 8 occurrences "Radicle" toutes mises a jour pour mentionner "Radicle differe v1.0" dans la meme phrase ou dans block §D5 dedie. `grep -n "Radicle" .planning/active/sprint18_kickoff.md` confirme. Aucune occurrence orpheline.
- [x] **Self-contained MIRROR_FALLBACK.md §3 flip sequence** : 8 sous-sections (3.1 visibilite, 3.2 setup Radicle 25min commandes, 3.3 secrets GHA 5 entrees tableau, 3.4 workflow YAML COMPLET avec SHA pin `gsaslis/mirror-to-radicle@514707f3...`, 3.5 canary update + commande regenerate signature, 3.6 verification clone, 3.7 docs tracking, 3.8 rotation machine-account). **Excellent** : un maintainer peut executer le v1.0 flip sans re-research.

**Tracking item Radicle-v1.0 (P2)** : `MIRROR_FALLBACK.md §3.7` dit "Fermer item tracking `sprint18_audit_plan.md` §Radicle-v1.0" et plan.md §Risques dit "tracker item S19 sprint18_audit_plan.md avec owner + deadline tag v1.0". Or `sprint18_audit_plan.md` n'existe pas encore (sera cree en Phase F). Plan.md §Phase F **ne mentionne PAS** explicitement l'ajout de cet item dans son `Livrables`. Risque : oubli de creer la section dedie en Phase F = "Radicle activation v1.0" tombe entre les mailles. **Fix Phase F** : ajouter une bullet `- Item tracking "Radicle-v1.0 activation" avec owner + deadline tag v1.0` dans le scope du `sprint18_audit_plan.md` futur.

---

## Reponses directes aux questions #1-10

### #1 Security workflow YAML

PASS. Token via `env: ${{ secrets.CODEBERG_TOKEN }}` injecte dans la step, utilise via `git -c "http.<url>.extraheader=Authorization: token ${TOKEN}"` sans URL embedding ni `set -x`. Permissions `contents: read`. Scope PAT documente Read+Write `repository` only. Pas de pin SHA pour `actions/checkout@v4` mais convention projet uniforme `@v4` partout (cf. dimension Patterns). `git diff --cached | grep -iE "(password|secret|token|key)"` → uniquement les references documentaires legitimes (CODEBERG_TOKEN, RADICLE_*_KEY/PASSPHRASE), aucun hardcoded. **Pattern `git push --mirror` destructif silencieux non-mentionne dans MIRROR_FALLBACK.md** = P2.

### #2 Patterns alignement

PASS. Format YAML conforme aux 8 workflows existants. Structure doc MIRROR_FALLBACK.md alignee REPRODUCIBLE_BUILDS.md. Concurrency pattern deja present (3 autres workflows). `fetch-depth: 0` justifie (push --mirror requiert all refs + history). **SPDX header manquant** = P3 (6/9 workflows en ont, 3/9 non — pas de regle stricte mais tous les workflows S18 recents en ont).

### #3 Scope-cuts

PASS. Zero match sur les 10 keywords scope-cut.

### #4 Tests-delta

PASS. 474 tests Rust verts (baseline post-E2 inchange). 0 test ajoute. 0 fichier code modifie. Conforme au plan "ops CI pur".

### #5 Pivot rationale documentation

PASS. Bloc dedie "Pivot 2026-04-15" dans plan.md ligne 801. Toutes les 8 occurrences "Radicle" dans kickoff.md mises a jour avec "Radicle differe v1.0". Auditeur S19 lisant uniquement le plan voit immediatement E3=Codeberg + flip Radicle au v1.0.

### #6 Radicle tracking futur

CONCERN (P2). MIRROR_FALLBACK.md §3 est self-contained pour executer le flip v1.0 (commandes rad, 5 secrets, workflow YAML complet pin SHA, verif). MAIS le tracking item "Radicle-v1.0 activation deadline + owner" n'existe que comme **reference forward** vers `sprint18_audit_plan.md` qui sera cree en Phase F. **Plan.md §Phase F ne liste pas explicitement l'ajout de cet item dans `sprint18_audit_plan.md`**. Risque oubli — fix : amender Phase F livrables avec une bullet dediee.

### #7 Deviations plan → code

CONCERN (P2). Plan annonce `~50 LOC YAML + ~110 LOC docs + 0 tests`. Livre : 50 YAML (exact) + **278 lignes docs** (+152% vs ~110 annonce) + 0 tests. Inflation due aux §3.1-§3.8 detailles (objectif self-contained explicite). Justifie sur le fond, mais **commit body devrait mentionner la deviation** et plan.md §Livrables ne liste pas §3.5/3.6/3.7/3.8 (uniquement §1-§7 top-level). Fix mineur Phase F.

### #8 YAML validite

PASS. `python -c "import yaml; yaml.safe_load(...)"` ok. Note benigne : `on:` parse comme `True` en YAML 1.1 (ambiguity). GHA lit le fichier texte directement, comportement attendu sur tous les workflows projet.

### #9 Cohabitation workflows existants

PASS. Concurrency `group: mirror-codeberg` est unique (pas de collision avec les 3 autres concurrency groups qui utilisent `${{ github.workflow }}-${{ github.ref }}`). Trigger `branches: ['**']` + `tags: ['**']` est INTENTIONNEL pour vrai mirror = chaque push sur n'importe quelle branche/tag est repliquee. **P3 mineur** : un burst de N push de feature branches simultanes file d'attente `cancel-in-progress: false` peut consommer ~N×15min de minutes GHA (timeout). Acceptable pre-launch (1 maintainer, faible volume), peut-etre revisiter post-v1.0.

### #10 Verification workflow fonctionnel

PASS sur l'analyse statique. Syntaxe `git -c "http.https://codeberg.org/.extraheader=Authorization: token ${PAT}"` est correcte : git config key `http.<URL>.extraheader` avec URL prefix matching (URL trailing `/` matche `https://codeberg.org/SBFB/SBFB.git`). Header `Authorization: token <PAT>` est la syntaxe standard Forgejo/Gitea PAT (acceptee par l'API v1). Verification fonctionnelle reelle (push reel) impossible sans pousser — c'est le but du `workflow_dispatch` post-merge pour smoke test.

---

## Findings

- **P2** — `docs/release/MIRROR_FALLBACK.md` ne documente pas le comportement destructif silencieux de `git push --mirror` (suppression refs Codeberg absentes de GitHub). Le plan.md le mentionne ligne 844 mais pas la doc user-facing. **Fix** : ajouter dans §6 Threat model fit > "Does NOT protect against" une bullet `- Branche cree manuellement cote Codeberg (pas via push GitHub) — sera supprimee silencieusement au prochain run mirror`. Ou §1 Rationale ajouter `Note : git push --mirror est destructif, Codeberg est strict mirror, ne pas commit directement cote Codeberg`. (`docs/release/MIRROR_FALLBACK.md:1-60`)

- **P2** — Tracking item "Radicle-v1.0 activation" n'a pas de landing spot concret. MIRROR_FALLBACK.md §3.7 et plan.md §Risques referencent `sprint18_audit_plan.md §Radicle-v1.0` mais ce fichier n'existe pas encore + plan.md §Phase F ne liste pas explicitement l'ajout de cet item. Risque oubli au wrap-up. **Fix Phase F** : amender plan.md §Phase F > Livrables > `sprint18_audit_plan.md` avec une bullet `- Track Radicle-v1.0 activation : pre-requis tag v1.0, owner maintainer, deadline = jour du tag, runbook MIRROR_FALLBACK.md §3.2-§3.8`. (`.planning/active/sprint18_plan.md:932-957`)

- **P2** — Plan.md §Phase E3 Livrables annonce `~110 lignes docs` mais livre 278 lignes (+152%). Inflation justifiee (§3.1-§3.8 self-contained pour executer flip v1.0 sans re-research) mais non-trace dans plan §Livrables (qui liste seulement §1-§7 top-level, pas les sous-sections). **Fix** : plan.md §Phase E3 Livrables MIRROR_FALLBACK.md devrait noter `(§3 decompose en 3.1-3.8 self-contained, ~170 lignes pour Radicle flip sequence detaille — au-dela de l'estimation initiale ~110)`. (`.planning/active/sprint18_plan.md:868-875`)

- **P2** — Commit body draft (cf. plan.md ligne 920-928) ne mentionne pas la deviation LOC docs (~110 → 278) ni la decision d'inclure §3.8 rotation machine-account key (non-listee dans plan original §Livrables). **Fix** : etoffer le commit body avec `Doc volontairement etendue (~278 lignes vs ~110 annonce) pour rendre §3 flip sequence v1.0 entierement self-contained — un maintainer peut executer le flip sans re-research`. (planning, commit message)

- **P3** — `mirror-codeberg.yml` n'a pas de header `# SPDX-License-Identifier: AGPL-3.0-or-later`. 6/9 workflows en ont, dont les 3 autres workflows ajoutes en S18 (canary-monthly, supply-chain, release). **Fix trivial** : ajouter `# SPDX-License-Identifier: AGPL-3.0-or-later` ligne 1 + ligne vide. (`.github/workflows/mirror-codeberg.yml:1`)

- **P3** — Cohabitation orgs GitHub `SBFB50` vs Codeberg `SBFB` non-explicitee. Le diff fait `git remote add codeberg "https://codeberg.org/SBFB/SBFB.git"` (org=`SBFB`) alors que GitHub est `SBFB50/SBFB`. Probablement parce que l'org `SBFB` etait disponible cote Codeberg sans le `50`. **Fix** : noter dans MIRROR_FALLBACK.md §1 Rationale : `Note : org Codeberg = "SBFB" (sans le 50 du GitHub), historique disponibilite namespace`. (`docs/release/MIRROR_FALLBACK.md:1-30`, `.github/workflows/mirror-codeberg.yml:48`)

- **P3** — `set -eu` au lieu de `set -euo pipefail`. Le workflow `canary-monthly.yml` (meme sprint) utilise `set -euo pipefail` pour le bash inline. Le workflow E3 fait juste `git remote add` puis `git push` (pas de pipe), donc `pipefail` n'apporterait rien fonctionnellement, mais cosmetique inconsistance avec le style canary. **Fix optionnel** : `set -euo pipefail` pour uniformite. (`.github/workflows/mirror-codeberg.yml:43`)

---

## Recommendation

**Commit autorise.** Aucun finding P0 ni P1.

Les 4 findings P2 sont tous des **ameliorations doc/planning sans impact fonctionnel** :
1. Documenter destructivite `git push --mirror` (ajout ~3 lignes dans MIRROR_FALLBACK.md §6).
2. Tracking item Radicle-v1.0 a ajouter explicitement dans Phase F livrables plan.md.
3. Note inflation LOC docs §Livrables plan.md.
4. Etoffer commit body avec mention deviation LOC + §3.8 ajoute.

Les 3 findings P3 sont des nits cosmetiques (SPDX header, note orgs cohabitation, `set -euo pipefail`).

**Recommendation operationnelle** :
- **Pre-commit** : optionnellement fixer P3-1 (SPDX header) inline (3 secondes de patch) pour conformite avec les workflows S18 voisins.
- **Phase F** : adresser les 4 P2 (tous des updates docs/planning sans impact code) + les 2 P3 restants (note orgs + pipefail) en bulk. Pattern E2 = inline-fix-in-same-commit pas applicable ici car certains P2 touchent Phase F future.

Phase E3 livre un mirror disaster-recovery Codeberg fonctionnel, securise (token via extraheader, scope minimal, permissions GHA `contents: read`), self-contained pour le flip Radicle au v1.0 (via MIRROR_FALLBACK.md §3.1-§3.8 detaille). Le pivot Radicle→Codeberg est correctement justifie et trace dans le plan + kickoff. VALIDATED_BLUEPRINT couche 10 opsec deuxieme brique livree (E2 = warrant canary, E3 = mirror redundancy).
