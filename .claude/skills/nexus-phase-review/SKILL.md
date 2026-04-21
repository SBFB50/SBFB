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

### Step 1bis — Pre-flight staging coherence check

Avant les suites §7.4, verifier que le working tree est prepare
pour un commit phase atomique :

```bash
git status --short
```

**Decision mecanique** (pas de question utilisateur) :

- **Modifs planning / docs Claude / PATTERNS.md hors `plan.md §Phase X`
  fichiers attendus** → commit `chore(planning): ...` ou
  `chore(skill): ...` AVANT le commit phase. Procedure standard,
  pas de confirmation.
- **Modifs scope cut documente kickoff §6 OU tech debt hors plan**
  → `git stash` ou commit separe `chore(debt): ...` AVANT phase.
- **Untracked accidentel** (node_modules, .env, cache, .pdb, build
  artefacts) → ajouter `.gitignore` dans le commit chore. Si
  pattern nouveau ambigu, STOP et demander.
- **Working tree clean hors scope phase** → commit phase direct.

Le hook `phase-precommit-lightcheck.sh` Check 1 (staging coherence
STRICT BLOCK) catche automatiquement les mismatch `+pub mod X;` /
`X.rs` untracked, mais il ne catch PAS les mix planning+phase ni
les scope-cut leaks — la discipline mecanique ci-dessus reste
requise. L'audit gate Phase 0 reconstitue la discipline historique
via `git log --stat` + split commits visibles, pas via artefact
body dedie.

**Anti-pattern a eviter** : demander "tu veux que je commit
chore(planning) d'abord ou je lance Phase E ?". Si le working tree
montre planning + phase fichiers, la reponse est mecanique :
chore(planning) d'abord, Phase apres. Pas de question.

### Step 2 — Verification suites (§7.4)

Lancer **les 3 blocs complets** (Rust + Python + Frontend),
independamment du langage touche par la phase. Une modification
dans un seul langage peut provoquer une regression cross-stack
(ex : wiring app.py casse un Playwright, endpoint http.rs casse
un proxy coord-side). Cout des 3 blocs ~5 min, cout d'une
regression non detectee = fix(sprint) + audit P1.

**NE PAS filtrer par "langage touche"** — c'est un anti-pattern
identifie Sprint 23 Phase E (suites web non lancees alors que
app.py modifie).

```bash
# Rust — nextest workspace + doctests
cargo fmt --all --check && \
  cargo clippy --workspace --all-targets --locked -- -D warnings && \
  cargo nextest run --workspace --locked && \
  cargo test --workspace --locked --doc

# Python
uv run ruff format --check packages/ && \
  uv run ruff check packages/ && \
  uv run pytest packages/nexus-sdk/tests/ -q && \
  uv run pytest packages/nexus-coordinator/tests/ -q && \
  uv run pytest packages/nexus-app-gov/tests/ -q

# Frontend
cd web && \
  npx tsc --noEmit -p tsconfig.app.json && \
  npm run lint && \
  npm run test:unit && \
  npm run build && \
  npm run size && \
  npx playwright test && \
  bash scripts/scan-en-strings.sh && \
  cd ..

# Release build (binary deliverable)
cargo build -p nexus-shell-daemon --release
```

**Toute suite rouge = STOP.** Remonter l'erreur a l'utilisateur pour
fix root-cause. Ne jamais suggerer `#[ignore]`, `xfail`, ou
`--no-verify`.

### Step 3 — Compter le delta tests reel

```bash
# Avant-apres pour chaque suite
# (le chiffre "apres" est ce que le user aura dans son body commit)

# Rust — nextest summary line: "Summary [   2.345s] 537 tests run: 537 passed"
# On prend le premier nombre apres "run:" (champ "passed") + on cumule
# les doctests comptes separement par cargo test --doc.
RUST_NEXTEST=$(cargo nextest run --workspace --locked 2>&1 | \
  grep -oE '[0-9]+ passed' | head -1 | awk '{print $1}')
RUST_DOC=$(cargo test --workspace --locked --doc 2>&1 | \
  grep -E '^test result:' | awk '{sum+=$4} END {print sum+0}')
RUST_AFTER=$((RUST_NEXTEST + RUST_DOC))

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

Si l'utilisateur n'a PAS fourni de draft commit body explicitement
dans la session courante, **générer systématiquement** un draft depuis
le diff en suivant le template §7.2 Cas B de `docs/claude/README.md`,
et le présenter en output markdown au user pour validation avant le
commit. Ne jamais supposer que l'user veut un body minimaliste —
default = body riche structuré avec contexte + fichiers touchés +
delta tests cumulé + scope cuts honoured + Co-Authored-By.

Checker :

1. **Format titre** : matche `(feat|fix|docs|chore|test)\(sprint{N}\): Sprint {N} Phase {X} — .+`
2. **Contexte** present (1-2 lignes expliquant le "pourquoi")
3. **Fichiers touches** listes avec rationale (pas juste la liste)
4. **Delta tests cumule** coherent avec Step 3
5. **Scope cuts honoured** liste copiee du kickoff §6
6. **Co-Authored-By: Claude <model_name> (1M context)** present — la ligne doit matcher le modèle utilisé pour la session courante

### Step 4bis — Verifier research grounding via context7

Le plan.md de chaque sprint doit avoir une section §Research consulte
(cf. docs/claude/README.md §2.2 section canonique). Cette section
documente les appels context7 (`mcp__context7__query-docs`,
`mcp__context7__resolve-library-id`) + lectures de registry / docs
officielles qui ont valide les choix d'API externe, pins de versions,
specs crypto, etc.

Check a effectuer :

1. Lire `.planning/active/sprint{N}_plan.md` §Research consulte
2. Verifier qu'elle n'est PAS vide
3. Pour chaque pin de dependance ajoute/modifie dans le diff de la phase :
   - `Cargo.toml` : nouvelle crate ou version bump -> est-il mentionne
     dans §Research consulte ?
   - `pyproject.toml` : idem cote Python
   - `package.json` : idem cote npm
4. Pour chaque usage de nouvelle API externe (crypto, spec
   standardisee comme SLSA/in-toto/JCS/Keyoxide) :
   - La source (context7 + URL + date) est-elle tracee dans §Research ?

Signal :
- **PASS** : chaque dep/API touche par le diff a une trace Research
- **CONCERN** : >= 1 dep/API sans trace mais non-critique (ex: patch
  version bump obvious, existing pattern)
- **FAIL** : >= 1 API crypto ou spec standardisee utilisee sans trace
  -> forcer la session a consulter context7 avant de committer

Exemple concret Sprint 18 Phase A (ce qui a ete fait correctement) :
```
§Research consulte :
  - /websites/rs_iroh (RelayMap API iroh 0.97) — 2026-04-12
  - /bytecodealliance/wasmtime (LTS 12-major-cycle) — 2026-04-09
  - /websites/embarkstudios_github_io_cargo-deny (advisories cfg) — 2026-04-12
  - WebSearch : RustSec recommande cargo-deny-action vs cargo-audit seul
  - WebSearch : SLSA v1.0 provenance spec
```

Anti-pattern a detecter :
```
§Research consulte : (section vide ou absente)
```
-> remonter a la session : "lancer `mcp__context7__resolve-library-id`
sur la lib $LIB avant d'ecrire ce code".

### Step 4ter — Horizon long-terme + documentation amont

Verifier l'application de la regle §6.7 `docs/claude/README.md`
(horizon long terme + doc AVANT code + solution la plus poussee).
Check a effectuer :

1. **Design doc present** pour nouveaux modules structurants
   (> 1 sprint de lifetime). Chercher dans `.planning/research/`,
   `docs/{domain}/`, ou plan §Research consulte. Absent sur
   nouveau module = P1.
2. **D1..D5 Day 0 citent alternatives rejetees + rationale**.
   Une decision sans alternative = P2 (design par reflexe).
3. **Solution la plus poussee** : si le diff choisit une lib
   ou un pattern alors qu'une alternative plus auditee /
   type-safe / fuzzed / FIPS / SLSA existe et n'est pas
   explicitement rejetee dans le plan = P1.
4. **Aucune estimation LOC** dans plan.md ou kickoff.md :

   ```bash
   grep -En 'LOC estim|~\s*[0-9]+\s*LOC|estim.*LOC' \
     .planning/active/sprint*_{plan,kickoff}.md
   ```

   Tout match = P2 (contraire a §6.7). Exception : LOC
   retrospective (mesure de gap a posteriori pour decider
   scope-cut) est legitime, ex : "le gap reel etait ~300 LOC".

Signal :
- **PASS** : design doc present + alternatives citees + choix
  techniquement justifie + aucun LOC estime au plan
- **CONCERN** : 1 item manquant mais justifiable (phase trivial
  refactor n'a pas besoin de design doc long)
- **FAIL** : choix technique courte-vue sans alternative
  documentee OU design doc manquant pour nouveau module
  structurant OU estimation LOC presente au plan

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

### Step 6 — Validation finale + rigor signal (G4)

**Critere de verdict explicite** (G4 — inverse l'incitation
"absence de finding = qualite") :

| Conditions | Verdict |
|---|---|
| 0 P0/P1 ET >= 1 finding P2+ documente | **PASS** (audit deep, autorise) |
| 0 P0/P1 ET 0 finding P2+ | **CONCERN** (audit insuffisant — re-audit requis avec dimension manquee : research-grounding ? horizon long-terme ? working tree ?) |
| 0 P0/P1 ET 1 finding P2+ avec carry-over explicite dans body | **PASS** (autorise mais entree obligatoire dans `sprint{N+1}_audit_findings.md`) |
| >= 1 P0 OU >= 1 P1 non resolu | **FAIL** (commit BLOQUE) |

**Rationale** : sur Sprint 19 Phase B, verdict CONCERN→PASS via 2
mitigations cosmetiques (commit body enrichi, design doc staged) sans
trouver le vrai probleme (Hashcash daté vs Equi-X 2023, runtime wire
silencieusement reporté S20 sans entrée audit_findings). Un audit
qui ne trouve aucun P2+ n'a pas cherché assez — par construction,
toute phase non-triviale a au moins 1 trade-off discutable.

**Trouver = qualite d'audit, pas absence = qualite**.

Produire un rapport markdown concis :

```markdown
# Phase Review — Sprint N Phase X

## Verdict : PASS | CONCERN | FAIL

(Rigor signal : N findings P2+ documentes / >=1 requis pour PASS rigoureux)

## Staging check (Step 1bis)
- Phase fichiers : <count> <list>
- Planning/docs split : chore(planning) fait ? oui/non/N/A
- Untracked accidentels : 0

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

## Horizon long-terme + documentation amont
- Design doc present (nouveaux modules) : ✅ / ❌
- D1..D5 avec alternatives + rationale : ✅ / ❌
- Solution la plus poussee (pas de courte-vue) : ✅ / ❌
- Aucune LOC estimee au plan : ✅ / ❌

## Findings (rigor signal — REQUIS >=1 P2+ pour PASS)
- **P2** : <description + file:line + carry-over si applicable>
- **P3** : <nit>
- (si 0 P2+ : VERDICT = CONCERN, lister dimensions sous-explorees)

## Recommendation
- Ready to commit : oui / non
- Carry-overs S{N+1} (P2+ non resolus) : <liste pour `sprint{N+1}_audit_findings.md`>
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
4. **Review coverage enforcement** : la presence de chaque
   `sprint{N}_phase_{X}_review.md` est verifiee au audit gate
   (cf. `docs/claude/README.md §4.4` step 5, Track F item
   "Phase review files present: N/N"). Data S23 : 1/6 reviews,
   audit gate non-detecte — le guard est desormais explicite.

## Refs

- `docs/claude/README.md` §4 (discipline de commit atomique)
- `docs/claude/README.md` §7.4 (verification script)
- `docs/claude/TOOLING.md` §4.2 (couche 2 skill nexus-phase-review)
