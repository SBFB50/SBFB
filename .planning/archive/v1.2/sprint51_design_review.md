# Sprint 51 — Design Review (G1)

**Reviewer** : Design agent (independent verification).
**Timestamp** : 2026-05-01.
**Reference** : sprint51_kickoff.md §4 (D1..D4).
**Verdict gate** : Enters Phase A contingent on D2/D3 remediation (see Rigor signal G4).

---

## Scoring

### D1 — Suppression legacy : nexus/ + tests/ + worker/ en bloc
**✅ PASS** — Source vérifiée + codebase checked

- **Vérification codebase** : zéro import ou référence directe depuis crates/, web/ vers le dossier Python `nexus/`.
- **Source** : récente (kickoff S51, commit 610b521, 2026-05-01).
- **Alternative concurrente** : *non documentée* dans le kickoff — aucune discussion d'alternatives au suppression en bloc (ex: archivage en branche, conversion partielle). Le kickoff §4 énumère les rejets de design mais sans source externe.
- **Scoring** : ✅ (source présente, récente, codebase validation passe, mais alternative concurrente non formellement comparée).

### D2 — Workspace Python residuals : pyproject.toml + uv.lock
**❌ FAIL** — Dépendance active détectée, source incomplète

- **Problème détecté** : `ruff` (linter Python) est **actuellement configuré et utilisé** via le root `pyproject.toml` :
  - Configuration : `pyproject.toml` lignes 31-40 : `[tool.ruff]` + `[tool.ruff.lint]`.
  - CI exécution : `.github/workflows/ci.yml` lignes 67-71 : `uv run ruff format --check packages/ examples/` et `uv run ruff check packages/ examples/`.
  - Cette exécution dépend du root `pyproject.toml` pour les paramètres `line-length`, `target-version`, `extend-exclude`.
- **Source incomplète** : Le kickoff D2 affirme "Plus aucun package Python actif dans le workspace" mais ne mentionne pas `ruff` comme outil dev, ni qu'il utilise la config du root `pyproject.toml`. La décision présente pyproject.toml comme "workspace Python, plus aucun package actif" — ambigu sur les outils dev.
- **Alternative concurrente** : non documentée. Aucune discussion d'alternative (ex: déplacer ruff config vers `ruff.toml` au root, ou dépendre d'un pyproject.toml décentralisé dans `packages/*/`).
- **Scoring** : ❌ (source présente mais incomplete sur dépendance ruff active, alternative concurrente absente).

### D3 — CI workflows : suppression Python + nettoyage references
**⚠️ WARN** — Dépendance Python non documentée dans un ci-smoke script

- **Problème détecté** : Un des 4 scripts ci-smoke cités en D1 comme "actifs depuis S18" **dépend de Python** :
  - Script : `tests/ci-smoke/supply-chain-green.sh`.
  - Lignes 42-58 : audit des 3 packages Python via `uv export ... | uv run --with 'pip-audit>=2.9,<3' pip-audit`.
  - Dépendance : packages Python actifs (`nexus-sdk`, `nexus-coordinator`, `nexus-app-gov`) + outil audit (`pip-audit`).
- **Implication** : Si Phase A supprime les packages Python et/ou le pyproject.toml, le script `supply-chain-green.sh` devient cassé (uv export échouera sur packages supprimés, uv sync échouera sans pyproject.toml).
- **Documenté dans kickoff** : D3 mentionne "préserver tests/ci-smoke/" mais n'élabore pas sur le contenu Python du script. Le kickoff note (§1.1, note risque R3) : "ci-smoke scripts referent des chemins Python supprimes" — accepté comme risque Medium/Low, non résolu en D3.
- **Alternative concurrente** : non documentée. Aucune discussion d'alternative (ex: isoler le supply-chain audit en workflow séparé, refactoriser ci-smoke en shell pur, ou dépendre d'une suite test Python post-v1.0).
- **Scoring** : ⚠️ (source présente, dépendance Python en ci-smoke non explicitement traitée en D3, alternative absente).

### D4 — 3 carries P2 a 2/3 : resolution Phase B
**✅ PASS** — Factualité vérifiée, sources précises

- **P2-REVIEW-A-1-S48** canary reload size cap :
  - **Mentionné** : "cap existe dans le code Rust (duress_ack.rs MAX, mod.rs MAX_HEADLINE_LEN)".
  - **Vérification** : 
    - `crates/nexus-shell-daemon-core/src/canary/duress_ack.rs:55` : `pub const MAX_DURESS_ACK_MESSAGE_LEN: usize = 256;` ✅
    - `crates/nexus-shell-daemon-core/src/canary/mod.rs:89` : `pub const MAX_HEADLINE_LEN: usize = 512;` ✅
    - Lignes 132, 205 : vérification du cap en runtime ✅
    - Lignes 238, 538-549 : tests sur cap ✅
  - **Scoring** : ✅ (cap existe, testé, localisé précisément).

- **P2-REVIEW-B-1-S48** auth.rs set_var residuel :
  - **Mentionné** : "set_var restants sont dans du code de test (SbfbHomeGuard avec Mutex, pattern save/restore)".
  - **Vérification** : 
    - `crates/nexus-shell-daemon-core/src/auth.rs:1073, 1077, 1086, 1096, 1114, 1118` : tous dans des `#[test]` functions (lignes 1061-1119) ✅
    - Pattern save/restore observé : sauvegarde `let prev = std::env::var().ok()`, restore en cleanup ✅
  - **Scoring** : ✅ (set_var test-only, pattern correct, localisé précisément).

- **P2-AUDIT-A-1-S48** doc accuracy reload_policy :
  - **Mentionné** : "issue originale sur `_reload_policy_locked` suffix trompeur dans le Python (S22). [...] Rust equivalent (`canary_input.rs`) utilise `reload_policy()` sans suffix".
  - **Vérification** :
    - `crates/nexus-coordinator-rs/src/canary_input.rs:500` : `fn reload_policy(&self)` (pas de suffix `_locked`) ✅
    - Contexte : charge de policy depuis fichier, déclenche reload_set(), pas d'appel lock direct ✅
    - Code Python original : supprimé depuis S50 ✅
  - **Scoring** : ✅ (suffix supprimé, Rust design correct, dépendance Python éliminée).

- **Scoring global D4** : ✅ (3/3 carries vérifiés factuellement, localisations précises, acceptabilité test-only documentée pour B-1).

---

## Rigor signal G4

### Anomalies détectées

| Anomalie | Type | Gravité | Recommandation |
|---|---|---|---|
| D2: ruff active via root pyproject.toml non documenté | Source incomplete | Medium | Clarifier si ruff migration vers `ruff.toml` est inclu Phase A, ou si pyproject.toml reste pour ruff uniquement |
| D3: supply-chain-green.sh dépend Python non adressé en D3 | Design gap | Medium | Phase A doit adresser soit la refactorisation supply-chain audit, soit sa suppression, soit son déplacement hors ci-smoke |
| D1: Alternative concurrente (ex: branche archive) non comparée | Design process | Low | Future decisions : documenter alternates même rejetées |

### Summary

**⚠️ 2 sur 4 décisions ont des angles morts** (D2 source incomplète, D3 dépendance non adressée).
**❌ 1 sur 4 : D2 n'a pas pu être valider au-dessus de "passe conditionnellement"** (ruff dépend toujours de pyproject.toml à aujourd'hui).
**✅ 1 sur 4 : D4 passe à 100%** (factualité confirmée, localisations précises).

### Gate recommendation

**Proceed to Phase A contingent on** :
1. **D2 clarification** : Confirm plan for ruff (migrate to `ruff.toml` at root, or keep pyproject.toml for ruff config only).
2. **D3 clarification** : Confirm plan for supply-chain audit (refactor, suppress, or keep as external suite).

Once D2/D3 are addressed in Phase A plan, proceed.

---

## Tracabilite

- kickoff.md §4 D1..D4 (lines 118-210).
- Verification performed on tip 610b521 (2026-05-01).
- Source files checked : pyproject.toml, .github/workflows/ci.yml, tests/ci-smoke/*.sh, crates/nexus-*/*.rs.

