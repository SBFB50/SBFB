---
name: nexus-phase-review
description: Review une phase SBFB avant son commit atomique. Lance toutes les suites §7.4 verification, valide le format du commit body (feat/fix/docs scope Sprint N Phase X), verifie coherence du delta tests annonce vs reel, scope cuts respectes. Invoquer avec "review my phase", "ready to commit phase", "verify phase X", ou apres avoir fini d'implementer une phase.
allowed-tools:
  - Read
  - Bash
  - Grep
  - Glob
  - Write
---

# nexus-phase-review

Skill de verification systematique pre-commit pour une phase SBFB.
Complete la couche 2 du process tooling (cf. `docs/claude/TOOLING.md`).

## Quand l'utiliser

- Tu viens de finir d'ecrire le code d'une phase A/B/C/D/E/F
- Avant `git commit feat(scope): Sprint N Phase X — ...`
- L'utilisateur dit "review my phase", "ready to commit", "verify phase"

## Quand NE PAS l'utiliser

- Hotfix hors sprint (cas D de README.md §7) -> verification ad-hoc
- Premier commit d'un nouveau sprint (kickoff+plan) -> juste ruff/prettier/tsc manuel
- Phase 0 audit gate -> tu joues le `sprint{N-1}_audit_plan.md`, pas une review phase

## Procedure

### Step 1 — Identifier le contexte

1. Lire `.planning/active/` pour trouver `sprint{N}_kickoff.md` et `sprint{N}_plan.md`
2. Extraire :
   - Numero de sprint N
   - Phase en cours (Phase X — la prochaine non commitee selon `git log`)
   - Scope cuts gelees (kickoff §6)
   - Delta tests attendus (plan.md §Phase X critere d'acceptation)
3. Lire `docs/claude/README.md` §4.3 pour la commande exacte §7.4 verification

### Step 2 — Verification suites (§7.4)

Lancer les suites pertinentes selon les langages touches dans le diff :

```bash
# Identifier les langages modifies
CHANGED_FILES=$(git diff --name-only HEAD)
HAS_RUST=$(echo "$CHANGED_FILES" | grep -E '\.rs$|Cargo\.toml' | wc -l)
HAS_PY=$(echo "$CHANGED_FILES" | grep -E '\.py$|pyproject\.toml' | wc -l)
HAS_WEB=$(echo "$CHANGED_FILES" | grep -E '^web/.*\.(ts|tsx|js|jsx|css)$' | wc -l)

# Rust (si touche)
[ "$HAS_RUST" -gt 0 ] && cargo fmt --all --check && \
  cargo clippy --workspace --all-targets --locked -- -D warnings && \
  cargo test --workspace --locked

# Python (si touche)
[ "$HAS_PY" -gt 0 ] && uv run ruff format --check packages/ && \
  uv run ruff check packages/ && \
  uv run pytest packages/nexus-sdk/tests/ -q && \
  uv run pytest packages/nexus-coordinator/tests/ -q && \
  uv run pytest packages/nexus-app-gov/tests/ -q

# Frontend (si touche)
[ "$HAS_WEB" -gt 0 ] && cd web && \
  npx tsc --noEmit -p tsconfig.app.json && \
  npm run lint && \
  npm run test:unit && \
  npm run build && \
  npm run size && \
  npx playwright test && \
  bash scripts/scan-en-strings.sh && \
  cd ..
```

**Toute suite rouge = STOP.** Remonter l'erreur a l'utilisateur pour
fix root-cause. Ne jamais suggerer `#[ignore]`, `xfail`, ou
`--no-verify`.

### Step 3 — Compter le delta tests reel

```bash
# Avant-apres pour chaque suite
# (le chiffre "apres" est ce que le user aura dans son body commit)
RUST_AFTER=$(cargo test --workspace --locked 2>&1 | \
  grep -E '^test result:' | awk '{sum+=$4} END {print sum}')
PY_SDK_AFTER=$(uv run pytest packages/nexus-sdk/tests/ -q 2>&1 | \
  grep -E 'passed' | tail -1 | awk '{print $1}')
# ... (idem coord, app-gov, vitest, playwright)
```

Comparer avec les compteurs du `memory/nexus_grid_pivot.md` ou du
commit precedent (`git log -1 --format=%B | grep -E 'Rust workspace|Python coord|Vitest|Playwright'`).

Calculer le **delta** attendu dans le body du prochain commit :

```
Rust workspace:     <before> -> <after> (+<delta> Phase X)
Python coord:       <before> -> <after> (+<delta>)
Vitest unit:        <before> -> <after> (+<delta>)
Playwright:         <before> -> <after> (+<delta>)
```

### Step 4 — Verifier le draft commit body

Demander a l'utilisateur le draft commit body (ou le generer a partir
du diff et lui soumettre).

Checker :

1. **Format titre** : matche `(feat|fix|docs|chore|test)\(sprint{N}\): Sprint {N} Phase {X} — .+`
2. **Contexte** present (1-2 lignes expliquant le "pourquoi")
3. **Fichiers touches** listes avec rationale (pas juste la liste)
4. **Delta tests cumule** coherent avec Step 3
5. **Scope cuts honoured** liste copiee du kickoff §6
6. **Co-Authored-By: Claude Opus 4.6 (1M context)** present

### Step 5 — Verifier scope cuts respectes

Pour chaque item "Scope cuts" du kickoff §6 du sprint en cours :

```bash
# Extraire les scope cuts du kickoff
SCOPE_CUTS=$(grep -A 20 '^## 6\. Scope cuts' \
  .planning/active/sprint*_kickoff.md | \
  grep -oE '`[^`]+`' | tr -d '`')

# Grep le diff pour chaque scope cut
git diff HEAD --name-only | while read f; do
  for cut in $SCOPE_CUTS; do
    if grep -l "$cut" "$f" > /dev/null 2>&1; then
      echo "WARN: $f touche le scope cut '$cut'"
    fi
  done
done
```

**Tout fichier qui touche un scope cut = P1 bloquant.** Remonter a
l'utilisateur : soit re-defer a un sprint futur, soit rouvrir le cut
dans le kickoff (mais alors le sprint doit etre re-valide).

### Step 6 — Validation finale

Produire un rapport markdown concis :

```markdown
# Phase Review — Sprint N Phase X

## Verdict : PASS | CONCERN | FAIL

## Suites
- Rust : 430 -> 437 (+7) ✅
- Python coord : 190 -> 192 (+2) ✅
- Vitest : 239 -> 239 (+0) ✅ (no frontend change)
- Playwright : 38 -> 38 (+0) ✅

## Commit body validation
- Format titre : ✅ "feat(sprint18): Sprint 18 Phase B — reproducible builds"
- Delta tests coherent : ✅
- Scope cuts honoured : ✅
- Co-Authored-By present : ✅

## Scope cuts verification
- "multi-relai phase 2" : 0 fichiers diff ✅
- "mobile client" : 0 fichiers diff ✅

## Recommendation
- Ready to commit : oui / non
- Corrections needed : <liste si non>
```

## Anti-patterns a eviter

1. **Ne PAS skipper une suite** "parce qu'elle a ete verte tout a
   l'heure". Le hook verify-on-write catch par-fichier, la suite
   complete catch les regressions cross-fichier.
2. **Ne PAS accepter un delta tests "a peu pres"**. Si le body dit
   `+7` et le reel est `+6`, il y a un test skip/ignore cache ou un
   test supprime non documente.
3. **Ne PAS faire un fix "dans la review"**. Un fix trouve ici
   devient sa propre iteration : user fixe, relance le skill.

## Refs

- `docs/claude/README.md` §4 (discipline de commit atomique)
- `docs/claude/README.md` §7.4 (verification script)
- `docs/claude/TOOLING.md` §4.2 (couche 2 skill nexus-phase-review)
