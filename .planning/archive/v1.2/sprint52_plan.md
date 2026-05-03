# Sprint 52 — Plan (dette pair + docs legacy + release validation)

**Tip d'entree** : `54cf0d0` (post-audit S51 PASS + chore
.gitignore).
**Phases** : A (dette pair + docs legacy), B (release workflow
validation), C (docs + verification + wrap-up).

---

## §Phase A — Dette pair obligatoire + docs legacy cleanup

**But** : fermer 3 items dette. dispatch join order 2/3 → CLOSE.
21 docs legacy → DELETE. CLAUDE.md stale carry → fix.

### Etapes

1. **Dispatch join order fix** :
   - Ajouter `dispatch_shutdown: Option<oneshot::Sender<()>>` a
     `DaemonRuntime` struct dans `runtime.rs`
   - Creer le channel dans `start()` et passer le receiver au
     dispatch_loop
   - Modifier `dispatch_loop.rs` : `tokio::select!` entre
     `rx.recv()` et `shutdown.recv()`
   - Dans `shutdown()` de DaemonRuntime : envoyer le signal
     AVANT de join le handle (symetrique a http_shutdown)
   - Verifier que le test `dispatch_loop` existant passe

2. **Docs legacy DELETE** :
   - `git rm` les 21 fichiers :
     ```
     docs/BENCHMARK.md docs/ARCHITECTURE.md
     docs/DATABASE_SCHEMA.md docs/README_FULL.md
     docs/GUIDE-UTILISATION.md docs/PIPELINE.md
     docs/API-REFERENCE.md docs/API_REFERENCE.md
     docs/CONFIGURATION.md docs/GUIDE-INSTALLATION.md
     docs/TOOLS_MATRIX.md docs/TESTING.md
     docs/API_COMPUTE.md docs/ARCHITECTURE_GPU.md
     docs/SECURITY_GPU.md docs/BENCHMARK_COMPUTE.md
     docs/COMPUTE_STATUS.md docs/FRONTEND_NETWORK.md
     docs/GUIDE_WORKER.md docs/WORKER.md
     docs/VISION_USE_CASES.md
     ```
   - Grep `docs/{fichier}` dans crates/, web/, .github/ pour
     confirmer 0 reference
   - Nettoyer .gitignore si certains fichiers y sont listes

3. **CLAUDE.md stale carry fix** :
   - Supprimer la ligne 127 :
     `P2-REVIEW-A-1-S51 release-attest.sh dead code 1/3 ;`

4. **Verifier** : cargo nextest + vitest inchanges

### Criteres d'acceptation

- dispatch_shutdown oneshot present dans runtime.rs + dispatch_loop
- `git ls-files docs/BENCHMARK.md docs/ARCHITECTURE.md ...` = 0
- CLAUDE.md carries S52 = 3 items (pas 6)
- `cargo nextest run --workspace --locked` >= 1199
- `npm run test:unit` (web/) = 250

### Commit

```
feat(sprint52): Sprint 52 Phase A — dette pair dispatch shutdown + docs legacy cleanup + CLAUDE.md fix
```

---

## §Phase B — Release workflow validation + fixes

**But** : valider release.yml via workflow_dispatch. Fixer les
issues trouvees.

### Prerequis

Phase A commitee. GitHub remote accessible. `gh` CLI authentifie.

### Etapes

1. **Pre-check release-attest.sh** (G1 finding D3 ⚠️) :
   - Lire `scripts/release-attest.sh` pour verifier si cosign
     est invoque avec `--bundle` ou non
   - Si le script depend de cosign v2 sans --bundle, garder le
     pin v2.4.1 dans release.yml (stabilite)
   - Si le script est compatible v3, considerer upgrade (non
     obligatoire — stability > latest)

2. **Lancer le workflow** :
   ```bash
   gh workflow run release.yml
   gh run list --workflow=release.yml -L 1
   # Attendre completion
   gh run watch <run-id>
   ```

3. **Analyser les resultats** :
   - 9 jobs (3 binaires × 3 OS) : documenter pass/fail
   - Pour chaque fail : lire les logs, identifier la cause
   - Telecharger artifacts si les jobs passent :
     ```bash
     gh run download <run-id>
     ```
   - Verifier SHA256 checksums

4. **Fixer les issues** (si necessaire) :
   - release.yml : path fixes, version bumps
   - release-attest.sh : cosign invocation, dist/ paths
   - build-pkarr-image.yml : refs si impactees

5. **Re-run si fixes appliques** : relancer workflow_dispatch
   pour confirmer les fixes

### Criteres d'acceptation

- Au moins 1 workflow_dispatch run complete (PASS ou issues
  documentees)
- Si PASS : 9/9 artifacts generes avec checksums SHA256
- Si partial fail : issues documentees + fixes commites
- Aucune regression tests Rust ou Frontend

### Commit

```
feat(sprint52): Sprint 52 Phase B — release workflow validation + fixes
```

---

## §Phase C — Docs + verification + wrap-up

**But** : mettre a jour la documentation, executer la verification
fail-fast, rediger l'audit plan S53.

### Etapes

1. **CLAUDE.md** :
   - Supprimer docs legacy de la structure si references
   - Mettre a jour les compteurs tests si delta
   - Mettre a jour carries S53

2. **HARDENING_ROADMAP.md** :
   - Mettre a jour last_validated S52
   - 0 trigger actif confirme

3. **Verification fail-fast** :
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
   - 3 carries dette CLOSED
   - Release workflow result

4. **sprint53_audit_plan.md** : tracks A-F pour audit S52

5. **Compteurs tests** : confirmer ~1455 total

### Criteres d'acceptation

- verification.md 15+ rows fail-fast verts
- CLAUDE.md a jour (carries S53 corrects)
- sprint53_audit_plan.md present
- Compteurs finaux documentes

### Commit

```
chore(sprint52): Phase C — wrap-up + verification + audit plan S53 + counters
```

---

## §4 Hors scope (rappel D1..D4 + scope cuts)

- VPS deployment (S53)
- LT-1 Kudos-v2 fairness reform (S53+ dedie)
- Events SSE daemon-native (post-v1.0)
- MCP server Rust (post-v1.0)
- Pagination SQL-side (S53+)
- mk_state() refactoring (S53+)
