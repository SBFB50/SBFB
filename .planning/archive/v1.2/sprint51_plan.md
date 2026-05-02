# Sprint 51 — Plan (suppression legacy + carries 2/3)

**Tip d'entree** : `610b521` (S50 audit PASS).
**Phases** : A (suppression legacy + CI), B (carries P2 batch),
C (docs + verification + wrap-up).

---

## §Phase A — Suppression legacy + workspace Python cleanup + CI

**But** : eliminer tous les fichiers Python legacy et les
workflows CI morts. 0 fichier Python dans le workspace git.

### Etapes

1. **Deplacer ci-smoke** : `git mv tests/ci-smoke/ scripts/ci-smoke/`
   (4 scripts SBFB actifs : attestation-schema.sh,
   pkarr-relay-healthcheck.sh, reproducible-build.sh,
   supply-chain-green.sh)

2. **Supprimer legacy Python** :
   - `git rm -r nexus/` (188 fichiers)
   - `git rm -r worker/` (10 fichiers)
   - `git rm` les 36 fichiers Python dans tests/ (tout sauf
     ci-smoke/ deja deplace)
   - `git rm pyproject.toml uv.lock`

3. **Supprimer build-wheels.yml** :
   `git rm .github/workflows/build-wheels.yml`

4. **Nettoyer ci.yml** : supprimer la section Python (lignes
   52-80 : setup-python, maturin, ruff, 3 pytest). Garder les
   sections Rust et Frontend intactes.

5. **Nettoyer release.yml** : supprimer le job "Build + attest
   nexus-core-py wheel" et les references PyO3/maturin.

6. **Mettre a jour refs ci-smoke** : dans
   build-pkarr-image.yml (ligne 127), changer
   `tests/ci-smoke/` → `scripts/ci-smoke/`.

6b. **Modifier supply-chain-green.sh** (G1 finding D3 ⚠️) :
    supprimer le step [2/3] pip-audit qui audite les 3 packages
    Python supprimes. Garder cargo-deny [1/3] et audit-ci [3/3].
    Renumeroter [1/3] → [1/2], [3/3] → [2/2]. Mettre a jour
    le header du script (description, exit codes). Ajuster
    `REPO_ROOT` si necessaire apres deplacement vers
    `scripts/ci-smoke/`.

7. **Gitignore** : ajouter `packages/` (cache __pycache__
   residuel non-tracke). Supprimer la ligne
   `packages/nexus-coordinator/nexus-coordinator.spec` (fichier
   legacy) de .gitignore.
   Supprimer `crates/nexus-core-py/target/` (crate supprime S50).

8. **.gitignore cleanup** : supprimer les entrees specifiques aux
   fichiers legacy desormais supprimes (test_agentctl.py, etc.)
   qui n'ont plus de raison d'etre. Garder les patterns generiques
   Python (__pycache__/, *.pyc) en defense-in-profondeur.

9. **Verifier** : grep `nexus/` dans docs/ et crates/ pour
   references orphelines.

10. **Nettoyer filesystem** : `rm -rf packages/` (untracked
    __pycache__). Pas de commit (pas tracke).

### Criteres d'acceptation

- `git ls-files nexus/ tests/*.py worker/ pyproject.toml uv.lock` = 0
- `git ls-files .github/workflows/build-wheels.yml` = 0
- `scripts/ci-smoke/` contient les 4 scripts intacts
- ci.yml ne reference plus Python/pytest/maturin
- release.yml ne reference plus nexus-core-py
- `cargo nextest run --workspace --locked` = 1199 (inchange)
- `npm run test:unit` (web/) = 250 (inchange)

### Commit

```
feat(sprint51): Sprint 51 Phase A — suppression legacy nexus/ + workspace Python cleanup + CI post-Python
```

---

## §Phase B — Carries P2 a 2/3 resolution batch

**But** : fermer les 3 items P2 a 2/3 de S48. Eviter escalade 3/3
MANDATORY en S52 (§6.2.1 Regle 2).

### Etapes

1. **P2-REVIEW-A-1-S48 canary reload size cap** :
   - Verifier `MAX_DURESS_ACK_MESSAGE_LEN` dans duress_ack.rs et
     `MAX_HEADLINE_LEN` dans canary/mod.rs
   - Verifier que les tests `duress_ack_rejects_oversize_message`
     et `build_canary_rejects_oversize_headline` existent et passent
   - Si cap implemente et teste → documenter CLOSE
   - Si gap → implementer le cap manquant

2. **P2-REVIEW-B-1-S48 auth.rs set_var residuel** :
   - Auditer tous les `std::env::set_var` dans crates/
   - Categoriser : test-only (SbfbHomeGuard pattern avec Mutex) vs
     production (launcher bootstrap)
   - Les set_var test-only avec Mutex serialization sont acceptables
     (Rust 1.94 `unsafe` env ops proposal pas encore stabilise)
   - Les set_var production dans launcher sont structurellement
     necessaires (spawn daemon herite env)
   - Documenter le verdict dans un commentaire `docs/rust/PATTERNS.md`
     si pas deja present

3. **P2-AUDIT-A-1-S48 doc accuracy reload_policy** :
   - Issue originale : `_reload_policy_locked` suffix trompeur dans
     canary_input.py (Python code supprime S50)
   - Verifier que le Rust canary_input.rs n'a pas le meme probleme
     de naming
   - Verifier coherence doc dans PATTERNS.md + security docs
   - Si resolu par suppression Python → CLOSE avec evidence

### Criteres d'acceptation

- 3 carries CLOSED avec evidence factuelle
- Aucune regression tests Rust ou Frontend
- Si des modifications code sont necessaires : delta tests documente

### Commit

```
feat(sprint51): Sprint 51 Phase B — carries P2 batch 2/3 resolution (canary cap + set_var + doc accuracy)
```

---

## §Phase C — Docs + verification + wrap-up

**But** : mettre a jour la documentation, executer la verification
fail-fast, rediger l'audit plan S52.

### Etapes

1. **CLAUDE.md** :
   - Supprimer `nexus/` de la structure
   - Supprimer references Python tooling (maturin, miniconda, uv,
     Python 3.13) de la section Stack
   - Mettre a jour la commande fail-fast (plus de bloc Python)
   - Ajuster le text "~1455 tests" si delta

2. **docs/claude/README.md** :
   - Verifier coherence post-Python (si references Python restent)

3. **HARDENING_ROADMAP.md** :
   - Mettre a jour last_validated S51
   - 0 trigger actif confirme

4. **Verification fail-fast** :
   - cargo fmt --all --check
   - cargo clippy --workspace --all-targets --locked -- -D warnings
   - cargo nextest run --workspace --locked
   - cargo test --workspace --locked --doc
   - cargo build -p nexus-shell-daemon --release
   - npm run lint (web/)
   - npx tsc --noEmit -p tsconfig.app.json (web/)
   - npm run test:unit (web/)
   - npm run build (web/)
   - npm run size (web/)
   - G8 preflights 2/2
   - Phase reviews 2/2
   - Scope cuts respectes
   - Delta tests cumule
   - 3 carries CLOSED

5. **sprint52_audit_plan.md** : tracks A-E pour audit S51

6. **Compteurs tests** : confirmer 1199 Rust / 250 Vitest /
   ~1455 total (inchange — sprint soustractif)

### Criteres d'acceptation

- verification.md 15+ rows fail-fast verts
- CLAUDE.md a jour (0 reference Python active)
- sprint52_audit_plan.md present
- Compteurs finaux documentes

### Commit

```
chore(sprint51): Phase C — wrap-up + verification + audit plan S52 + counters
```

---

## §4 Hors scope (rappel D1..D4 + scope cuts)

- Binaires release cross-platform (S52)
- VPS deployment (S52)
- Conversion nexus/ en app SBFB (post-v1.0)
- MCP server Rust (post-v1.0)
- Pagination SQL-side (S52+)
- mk_state() refactoring (S52+)
