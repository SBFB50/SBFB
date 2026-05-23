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

### Step 1.5 — Memory consultation (avant suites)

Lire `MEMORY.md` (index) et charger les memories pertinentes pour
la zone fonctionnelle de la phase. L'objectif : calibrer la review
contre les contraintes documentees dans la memory.

**Routing table** (source of truth : identique preflight et review) :

| Zone phase | Memory file | Contrainte cle |
|---|---|---|
| (toujours) | `feedback_approach.md` | pick deepest, no band-aid, research before code |
| kudos / fairness / reputation | `fairness_vision.md` + `feedback_kudos_non_monetary.md` | non-monetary, no cost/deposit/stake |
| governance / funding / modele | `vision_model.md` | OpenBSD solo maintainer, no startup |
| deploy / crypto / Ed25519 | `sprint14_keyoxide_decision.md` | from-source verified deploy |
| lib externe / dep / API spec | `feedback_context7_systematic.md` | context7 obligatoire avant code |

Matcher zone depuis Step 1 fichiers touches. Toute modification
future de cette table doit toucher les 2 skills (grep "Routing
table" dans `.claude/skills/*/SKILL.md`).

**Mecanisme** : noter dans le rapport review §Memory consultation
les contraintes verifiees et leur statut (respecte / viole / N/A).
Violation memory = P1 bloquant (la memory capture des decisions
utilisateur explicites).

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

Lancer **les 2 blocs complets** (Rust + Frontend),
independamment du langage touche par la phase. Une modification
dans un seul langage peut provoquer une regression cross-stack
(ex : endpoint http.rs casse un test frontend). Cout des 2 blocs
~5 min, cout d'une regression non detectee = fix(sprint) + audit P1.

**NE PAS filtrer par "langage touche"** — c'est un anti-pattern
identifie Sprint 23 Phase E (suites web non lancees alors que
app.py modifie).

```bash
# Rust — nextest workspace + doctests
cargo fmt --all --check && \
  cargo clippy --workspace --all-targets --locked -- -D warnings && \
  cargo nextest run --workspace --locked && \
  cargo test --workspace --locked --doc

# Python — OBSOLETE depuis pivot S50 (code Python supprime)
# Bloc conserve pour reference historique uniquement.
# Ne PAS executer — les packages/ n'existent plus.
# uv run ruff format --check packages/ && \
#   uv run ruff check packages/ && \
#   uv run pytest packages/nexus-sdk/tests/ -q && \
#   uv run pytest packages/nexus-coordinator/tests/ -q && \
#   uv run pytest packages/nexus-app-gov/tests/ -q

# Frontend
cd web && \
  npx tsc --noEmit -p tsconfig.app.json && \
  npm run lint && \
  npm run test:unit && \
  npm run build && \
  npm run size && \
  bash scripts/scan-en-strings.sh && \
  cd ..

# Release build (binary deliverable)
cargo build -p nexus-shell-daemon --release
```

**Toute suite rouge = STOP.** Remonter l'erreur a l'utilisateur pour
fix root-cause. Ne jamais suggerer `#[ignore]`, `xfail`, ou
`--no-verify`.

### Step 2bis — Modified-file branch coverage check (G9)

Le delta tests (Step 3) verifie "combien de tests ajoutes". Ce step
verifie "est-ce que chaque nouvelle branche/methode dans un fichier
EXISTANT modifie est exercee par au moins 1 test".

**Rationale** : S24 Phase D a livre `_schedule_rerun()` (40 LOC) et
la branche `if self._rerun_sampler is not None` dans
`mark_completed()` sans aucun test d'integration. Le delta tests
plan vs reel matchait (+10/+10) mais le wiring etait invisible.
Phase C avait le meme pattern (validator.py hooks non integres).
Le trou est structurel : les tests unitaires couvrent les composants
isoles, pas le wiring dans les fichiers existants.

**Procedure** :

1. Lister les fichiers existants modifies par la phase (pas les NEW) :
   ```bash
   git diff HEAD --name-only -- '*.py' '*.rs' '*.ts' '*.tsx' | \
     grep -v __pycache__
   ```

2. Pour chaque fichier, extraire les nouvelles methodes/branches :
   ```bash
   git diff HEAD -- <file> | grep -E '^\+.*(def |fn |async fn |if |match )'
   ```

3. Pour chaque nouvelle methode/branche identifiee, grep les fichiers
   test pour verifier qu'au moins 1 test l'exerce :
   - Methode `_schedule_rerun` → grep `schedule_rerun` dans tests/
   - Branche `if self._rerun_sampler` → grep `rerun_sampler` dans tests/
   - Methode publique `foo()` → grep `foo` dans tests/

4. **Signal** :
   - **PASS** : chaque methode/branche a >= 1 test qui l'exerce
   - **CONCERN** : branche defensive triviale (`if x is None: return`)
     sans test — acceptable si le path principal est teste
   - **FAIL** : methode ou branche de logique metier sans
     test → P1 bloquant, ajouter le test avant commit

**Anti-pattern** : "les tests du composant isole suffisent". Non.
Le composant peut etre correct et le wiring casse (mauvais param,
oubli d'appel, condition inversee). Le test d'integration du wiring
est le seul qui le detecte.

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

# ... (idem vitest, size-limit)
```

Comparer avec les compteurs du `memory/nexus_grid_pivot.md` ou du
commit precedent (`git log -1 --format=%B | grep -E 'Rust workspace|Vitest|size-limit'`).

Calculer le **delta** attendu dans le body du prochain commit :

```
Rust workspace:     <before> -> <after> (+<delta> Phase X)
Vitest unit:        <before> -> <after> (+<delta>)
size-limit:         <before> -> <after>
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
6. **Codex verification** section presente (meme pre-Codex, avec statut
   EN ATTENTE si le rapport n'existe pas encore)
7. **Co-Authored-By: Claude <model_name> (1M context)** present — la ligne doit matcher le modèle utilisé pour la session courante

### Step 4bis — Body format validation (§4.1)

Verifier que le draft commit body fourni par l'executeur contient les 9
headers `##` obligatoires (README §4.1) :
1. `## Contexte`
2. `## Fichiers`
3. `## Delta tests`
4. `## Verification` (ou `## Vérification` ou `## Verification §7.4`)
5. `## Scope cuts` (ou variantes `respectés`/`honoured`)
6. `## G8 traceability`
7. `## Pre-launch protocol`
8. `## Codex verification`
9. `## Carry closure` (ou `## Carry closure / Unblock`)

Header manquant = **P1 bloquant** "body-format-{section}".
Si body non fourni : **CONCERN** "draft-body-absent".

**Pattern S65 initial** : les anciennes references `8/8` sont
obsoletes depuis l'ajout de `## Codex verification`. Le standard
courant est **9/9 headers** et le skill doit detecter toute phase qui
reste sur l'ancien format.

**Template de reference** : `.claude/templates/commit_body_phase.txt`

### Step 4ter — Verifier research grounding (approche + deps)

Deux dimensions a verifier :

#### 4ter-A — Preflight G8 completeness check (G10)

Le preflight G8 doit avoir ete execute AVANT le code. Verifier
l'artefact ET son contenu :

1. Verifier que `.planning/active/sprint{N}_phase_{X}_preflight.md`
   **existe**. Si absent → P1 bloquant ("preflight G8 non execute").
2. Verifier que le fichier contient les **5 sections de scan** :
   ```bash
   grep -cE "S1a|S1b|S2|S3|S4" \
     .planning/active/sprint{N}_phase_{X}_preflight.md
   ```
   Si < 5 sections → P2 ("preflight incomplet, scans manquants").
3. La section `S1a OSS prior art` contient au moins 1 projet OSS
   de reference consulte (context7 ou WebSearch). Si 0 projet
   nomme → P2.
4. Si verdict PLAN-ADAPT : la section `§Plan adaptation` documente-
   t-elle l'evidence et l'approche corrigee ?

Signal :
- **PASS** : fichier existe + 5 scans presents + S1a avec >= 1 projet OSS
- **CONCERN** : S1a presente mais sommaire ("APPROACH-ALIGNED"
  sans nommer le projet consulte)
- **FAIL** : fichier absent (P1) OU phase implementee avec approche naive
  que l'OSS montre inadaptee (= preflight S1a n'a pas ete fait
  ou a ete ignore). APPROACH-NAIVE non detecte pre-code = P1.

**Anti-pattern cle (S24 Phase D post-mortem)** : le plan disait
"hash binaire BLAKE3 pour comparer outputs LLM re-run". BOINC et
Truebit montrent que la comparaison exacte ne marche pas sur des
outputs stochastiques. Le preflight S1a aurait du detecter
APPROACH-NAIVE et emettre PLAN-ADAPT. A la place, S1 n'a verifie
que les versions de libs (S1b) et a emis "clean". Le code livree
est structurellement inoperant sur le use case principal.

#### 4ter-B — Deps/API via context7 (existant)

Le plan.md doit avoir une section §Research consulte (cf.
docs/claude/README.md §2.2). Check :

1. Lire `.planning/active/sprint{N}_plan.md` §Research consulte
2. Verifier qu'elle n'est PAS vide
3. Pour chaque pin de dependance ajoute/modifie dans le diff :
   - `Cargo.toml` / `pyproject.toml` / `package.json` -> mentionne
     dans §Research consulte ?
4. Pour chaque API externe (crypto, spec standardisee) :
   - Source (context7 + URL + date) tracee dans §Research ?

Signal :
- **PASS** : chaque dep/API touche par le diff a une trace Research
- **CONCERN** : >= 1 dep/API sans trace mais non-critique
- **FAIL** : >= 1 API crypto ou spec sans trace → context7 avant commit

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

### Step 4quater — Horizon long-terme + documentation amont

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

### Step 5bis — Codex verification gate (§4.5)

La sequence §4.3 impose Codex verification croisee ENTRE la
review Claude et le commit. **Zero exemption** — TOUTES les phases
recoivent le meme traitement maximal, sans exception basee sur le
contenu, la taille, ou le type de phase. La seule facon de skip
est un "PO dit skip codex" explicite.

**Procedure** :

1. Verifier que `sprint{N}_phase_{X}_codex_review.md` existe dans
   `.planning/active/`
   - Si ABSENT : le verdict review NE DOIT PAS dire "Ready to commit"
     mais **"Ready for Codex verification (§4.5)"**
   - Ajouter dans le rapport :
     ```
     ## Codex gate (§4.5)
     - Status : EN ATTENTE — lancer Codex §4.5 avant commit
     - Procedure : ecrire prompt dans .git/CODEX_SPRINT{N}_PHASE_{X}.txt
       (ou imprimer le chemin via agentctl codex-prompt-path)
       (template .claude/templates/codex_phase_review.txt),
       lancer codex exec, lire rapport, corriger GAPs
     ```

2. Si le fichier codex_review.md EXISTE deja :
   - Verifier qu'il ne contient pas de GAPs non resolus
   - Documenter dans le rapport :
     ```
     ## Codex gate (§4.5)
     - Status : FAIT — {N} GAPs confirmes, {M} faux positifs
     ```

**Anti-pattern** : dire "ready to commit" quand Codex est requis
mais pas fait. C'est exactement le gap qui a cause l'incident
Sprint 65 Phase A.

### Step 5ter — Artefact review.md obligatoire

Ce skill DOIT produire un fichier
`.planning/active/sprint{N}_phase_{X}_review.md` avec le rapport
complet (template Step 6 ci-dessous). L'absence de ce fichier est
un P2 detectable par l'audit gate (Track F "Phase review files
present").

Ecrire le fichier AVANT de rendre le verdict final. Le hook
`phase-auditor-gate.sh` (Check A2) bloque mecaniquement le commit
si ce fichier est absent ou ne contient pas `## Verdict : PASS`
ou `## Verdict : PASS-PENDING`. Cette tolerance hook ne rend PAS
`PASS-PENDING` committable : le superviseur et le process canonique
bloquent tout commit tant que Codex n'a pas ete lance et que le
review.md n'a pas ete promu a `## Verdict : PASS`.

### Step 6 — Validation finale + rigor signal (G4)

**Critere de verdict explicite** (G4 — inverse l'incitation
"absence de finding = qualite") :

| Conditions | Verdict |
|---|---|
| 0 P0/P1 ET >= 1 finding P2+ documente ET Codex FAIT + reconciliation ecrite dans review.md | **PASS** (final, committable apres supervisor) |
| 0 P0/P1 ET >= 1 finding P2+ documente ET Codex EN ATTENTE | **PASS-PENDING** (review OK, Codex requis avant commit ; jamais final) |
| 0 P0/P1 ET 0 finding P2+ | **CONCERN** (audit insuffisant — re-audit requis avec dimension manquee : research-grounding ? horizon long-terme ? working tree ?) |
| 0 P0/P1 ET 1 finding P2+ avec carry-over explicite dans body ET Codex FAIT + reconciliation ecrite | **PASS** (final, entree obligatoire dans `sprint{N+1}_audit_findings.md`) |
| >= 1 P0 OU >= 1 P1 non resolu | **FAIL** (commit BLOQUE) |

**Comptage rigor signal** : le chiffre "N findings P2+" dans le
header du rapport DOIT correspondre au `grep -c '^- \*\*P[0-2]\*\*'`
de la section Findings. P3 n'est PAS P2+. Un comptage faux est un
ecart P3 detectable par l'audit gate.

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

## Verdict : PASS | PASS-PENDING | CONCERN | FAIL

`PASS-PENDING` = etat transitoire pre-Codex. Il autorise seulement
la sequence Codex §4.5. Il doit etre remplace par `PASS` apres
reconciliation Codex avant tout commit.

(Rigor signal : N findings P2+ documentes / >=1 requis pour PASS rigoureux)

## Staging check (Step 1bis)
- Phase fichiers : <count> <list>
- Planning/docs split : chore(planning) fait ? oui/non/N/A
- Untracked accidentels : 0

## Suites
- Rust : 430 -> 437 (+7)
- Vitest : 239 -> 239 (+0) (no frontend change)
- size-limit : 6/6

## Commit body validation
- Format titre : ✅ "feat(sprint18): Sprint 18 Phase B — reproducible builds"
- Delta tests coherent : ✅
- Scope cuts honoured : ✅
- Co-Authored-By present : ✅

## Body format validation (Step 4bis, §4.1)
| Section | Present | Signal |
|---------|---------|--------|
| Contexte | oui/non | ok/P1 |
| Fichiers | oui/non | ok/P1 |
| Delta tests | oui/non | ok/P1 |
| Verification §7.4 | oui/non | ok/P1 |
| Scope cuts | oui/non | ok/P1 |
| G8 traceability | oui/non | ok/P1 |
| Pre-launch protocol | oui/non | ok/P1 |
| Codex verification | oui/non | ok/P1 |
| Carry closure | oui/non | ok/P1 |

## Modified-file branch coverage (Step 2bis, G9)
- <file.py> : `new_method()` → tested by `test_X` ✅
- <file.py> : `if self._foo is not None` branch → tested by `test_Y` ✅
- (FAIL si methode de logique metier sans test)

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

## Codex gate (§4.5) — zero exemption
- Status : FAIT / EN ATTENTE
- (si FAIT : {N} GAPs confirmes, {M} faux positifs, {K} corriges)

## Codex reconciliation
- Status : N/A pre-Codex | FAIT
- Review final : PASS uniquement si le rapport Codex a ete lu,
  les GAPs P0/P1 corriges, et les P2/P3 documentes dans le body

## Recommendation
- Ready to commit : oui seulement si verdict PASS final | non si PASS-PENDING
- Carry-overs S{N+1} (P2+ non resolus) : <liste pour `sprint{N+1}_audit_findings.md`>
- Corrections needed : <liste si non>

## Post-commit obligatoire
- [ ] Update nexus_grid_pivot.md
- [ ] Update MEMORY.md
```

### Step 7 — Post-commit obligations reminder

Apres verdict PASS final post-Codex, rappeler dans le rapport les
obligations post-commit. Si le verdict est PASS-PENDING, rappeler
Codex + reconciliation/promote review PASS avant toute tentative de
commit. Ce step ne bloque pas — il documente.

```markdown
## Post-commit obligatoire
- [ ] Update nexus_grid_pivot.md (tip SHA + description sprint + compteurs tests)
- [ ] Update MEMORY.md (ligne index si pivot description changee)
- [ ] Verifier que le fichier review.md est stage dans le commit chore(planning) suivant
      ou dans le commit phase lui-meme (si pas de chore(planning) intermediaire)
```

**Rationale** : la memory `feedback_memory_update.md` prescrit cet
update mais rien ne le rappelle mecaniquement apres le commit.
L'inclusion dans le rapport review garantit que l'agent voit le
rappel dans le flux de la session courante.

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
