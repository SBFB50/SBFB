# Système de travail Claude Code sur nexus-grid (SBFB)

Cette doc capture la méthode de travail qu'on a construite avec
Claude Code depuis le pivot 2026-04-10. Elle existe pour qu'une
session fraîche puisse reprendre le projet sans perdre de temps
à ré-inventer la discipline.

**Cible** : l'utilisateur (FlowUP) et tout agent (Claude, humain,
autre LLM) qui doit ouvrir, livrer ou auditer un sprint.

**Sources primaires vivantes** (à lire si un détail manque ici) :

- `.planning/sprint{N}_kickoff.md`, `sprint{N}_plan.md`,
  `sprint{N}_verification.md`, `sprint{N}_audit_plan.md`,
  `sprint{N}_audit_findings.md` — une instance de la méthode
  par sprint
- `C:\Users\FlowUP\.claude\projects\C--Users-FlowUP-Documents-Code-nexus\memory\` —
  mémoire cross-session (profile user, état projet, feedback
  approach, audit gate pattern)
- `docs/shell/PATTERNS.md`, `docs/rust/PATTERNS.md` — patterns
  techniques + tech debt tracking

---

## 1. Vue d'ensemble

Le projet nexus-grid (SBFB) est un réseau P2P de compute LLM
distribué. L'ingénierie se fait exclusivement via Claude Code,
sur des sessions courtes (1h30 à 3h), avec une discipline de
sprint inspirée des pratiques agile mais adaptée au fait que
l'agent n'a pas de mémoire entre sessions.

Les piliers du système :

1. **Roadmap multi-sprint** — Sprint 0 à Sprint 11+, chaque
   sprint a un scope défini dans la memory
   `nexus_grid_pivot.md`
2. **Découpage en phases** — chaque sprint est 4 à 6 phases
   (A..F typiquement), une phase = un commit atomique
3. **4 documents de planification par sprint** — kickoff, plan,
   verification, audit_plan (écrits par le sprint en cours) +
   audit_findings (produit par la session fraîche du sprint
   suivant en Phase 0)
4. **Audit gate pattern** — à partir de Sprint 7, chaque sprint
   ouvre par une Phase 0 qui audite le sprint précédent. Les
   P0/P1 doivent être fixés avant le premier commit de la
   Phase A du nouveau sprint
5. **Memory system externe** — cinq fichiers persistés hors
   repo qu'une session fraîche lit au démarrage pour retrouver
   le contexte
6. **Commit discipline atomique** — un commit par phase,
   pattern `feat(scope): Sprint N Phase X — titre`, body riche
   qui documente delta de tests et scope cuts respectés

---

## 2. Structure du fichier `.planning/` par sprint

Pour le sprint N, les documents en `.planning/sprint{N}_*.md`
sont :

```
sprint{N}_kickoff.md        # écrit EN ENTRÉE du sprint
sprint{N}_plan.md           # écrit EN ENTRÉE du sprint
sprint{N}_verification.md   # écrit EN SORTIE du sprint
sprint{N}_audit_plan.md     # écrit EN SORTIE du sprint
sprint{N}_audit_findings.md # écrit par la session fraîche DU SPRINT N+1 en Phase 0
```

Les 4 premiers sont livrés par l'agent qui exécute le sprint.
Le 5e est produit par une session Claude Code fraîche qui joue
l'audit au démarrage du sprint N+1.

### 2.1 kickoff.md — le contrat d'entrée

Écrit par l'agent qui démarre le sprint. Rôle : figer les
décisions non-rebattables avant d'écrire la moindre ligne de
code.

Sections canoniques (pattern Sprint 6/7) :

1. **Constat d'entrée** — quel est le tip master au début,
   quels tests passent, quels commits ont landé depuis le
   sprint précédent, quel est le verdict de l'audit gate
2. **Goal en une phrase** — ce que le sprint promet de livrer
3. **Phase 0 — Audit gate du sprint précédent** (DONE avant le
   kickoff lui-même à partir de Sprint 7) — résumé du verdict
   et du commit stack de gate
4. **Décisions Day 0 (D1..D5 gelées)** — les choix
   architecturaux qui vont piloter toutes les phases. Une
   fois figées, non rebattables. Format :
   - titre court
   - « Retenu » : la décision
   - « Rejeté » : les alternatives considérées avec raison
   - « Implications » : ce que ça verrouille dans le code
5. **Plan Phase outline A..F** — une section courte par phase
   avec son scope, son critère d'acceptation et son commit
   cible
6. **Scope cuts** — liste des choses qu'on ne fera PAS dans
   ce sprint et pour quel sprint elles sont gardées
7. **Traçabilité scope** — table qui mappe chaque item « What's
   NOT » du sprint précédent sur le sprint + phase où il est
   pris en charge
8. **Audit gate pattern — rappel** — confirme que la Phase 0
   a été jouée et que la Phase F devra produire l'audit_plan
9. **Checkpoint de validation** — ce que l'utilisateur doit
   valider avant que l'agent attaque le plan détaillé

Cas particulier Sprint 5 (950 lignes) : le kickoff contenait
initialement aussi le plan détaillé. Depuis Sprint 6, kickoff
et plan sont deux fichiers séparés — kickoff est court (300
à 600 lignes), plan est long (700 à 950 lignes).

### 2.2 plan.md — le plan d'exécution détaillé

Écrit juste après le kickoff, avant le premier commit feat du
sprint. Rôle : donner à l'agent exécuteur (souvent la même
session) une feuille de route ligne-par-ligne pour chaque
phase.

Sections canoniques (pattern Sprint 6/7) :

1. **État vérifié à l'entrée** — re-constate le tip master,
   les compteurs de tests par suite, les warnings clippy, les
   budgets size-limit. Source de vérité pour les cellules
   « Observed » du verification.md final
2. **Décisions Day 0 (gelées)** — rappel synthétique des
   D1..D5 du kickoff avec leur implication code
3. **Research consulté** — traces des recherches Context7,
   lectures de registry local `~/.cargo/registry/`, grep de
   patterns précédents. L'audit gate peut re-challenger ces
   sources si elles semblent incomplètes
4. **Phase A..F** — une section complète par phase avec :
   - Fichiers ajoutés / modifiés (chemin + LOC estimée +
     3 à 5 lignes de structure)
   - Tests à écrire (nommage + scénario)
   - Critère d'acceptation (ce qui doit être vert avant de
     commiter)
   - Commit cible (titre complet)
5. **Fail-fast checklist** — table `| # | Check | Commande
   | Critère | Observed |` qui liste 24 à 32 rows exécutables.
   Chaque row est la vérif qu'on rejouera au verification.md.
   La colonne `Observed` est vide au plan, remplie au
   verification
6. **Git plan** — liste ordonnée des commits atomiques
   attendus avec leur scope et titre exact
7. **Scope cuts** — copie de la liste du kickoff §6, répétée
   ici pour que l'agent exécuteur n'ait pas à switcher de
   fichier
8. **Risks (R1..RN)** — liste des risques techniques avec
   mitigation proposée. L'audit gate ira vérifier que
   chaque mitigation est réellement en place
9. **Checkpoint de clôture** — les N conditions pour dire
   « sprint fermé » (ex: 32/32 fail-fast, N commits, 2
   fichiers planning écrits, PATTERNS.md à jour)

### 2.3 verification.md — le self-report fail-fast

Écrit en fin de sprint, juste avant la Phase F « sortie ».
Rôle : rejouer la checklist fail-fast du plan.md et remplir
la colonne `Observed` avec les valeurs réelles.

**C'est un self-report** — l'agent qui a écrit le code écrit
aussi la vérification. Valeur limitée par construction : les
rows vont passer parce que l'agent a écrit le code pour les
faire passer. C'est exactement la raison d'être de l'audit
gate (§3 ci-dessous).

Sections canoniques :

1. **HEAD entrée / HEAD sortie** — les deux SHAs git qui
   bornent le sprint
2. **Commit stack** — `git log --oneline master ^<entrée>`
3. **How to re-run** — le bloc bash exhaustif qu'un humain
   peut copier pour rejouer tous les checks
4. **Checklist** — la même table que plan.md §Fail-fast mais
   avec la colonne Observed remplie et un `✅` pour chaque
   row verte
5. **Métriques sprint** — table `Suite | Avant | Après |
   Delta` pour Rust / Python SDK / coord / app-gov / Vitest /
   Playwright / size-limit
6. **Surface nouvelle livrée** — liste factuelle des LOC par
   nouveau module
7. **Ce que le sprint n'a PAS livré (scope cuts respectés)**
   — reprise de §6 kickoff avec check `❌` pour chacun
8. **Checkpoint de clôture** — les N conditions du plan.md
   §checkpoint, chacune cochée

### 2.4 audit_plan.md — le plan d'audit pour le sprint suivant

Écrit dans le même commit que verification.md, en Phase F
« sortie ». Rôle : donner à la session fraîche du sprint N+1
une feuille de route d'audit indépendante et reproductible.

**C'est le doc le plus stratégique** — une session fraîche
sans historique va le jouer et produire un verdict que l'agent
livreur ne peut pas influencer.

Sections canoniques (pattern Sprint 6/7) :

1. **Mode d'emploi pour la session fraîche** — ordre de
   lecture imposé, liste des fichiers à NE PAS lire avant
   d'avoir formé une opinion, timebox suggéré (2-3h), format
   du delivrable final
2. **Tracks A..I** — une section par axe d'audit. Chaque
   track contient :
   - Question centrale (« Est-ce que X fait vraiment Y ? »)
   - Méthodes concrètes (commandes à rouler, grep à lancer,
     tests à écrire)
   - Signal d'audit (qu'est-ce qui est P0, P1, P2, P3)
3. **Verdict global attendu** — trois scénarios :
   - PASS : 0 P0, 0 P1 → sprint N+1 Phase A démarre direct
   - CONDITIONAL PASS : 1-3 P1 fixables → N+1 Phase A bloqué
     tant que les `fix(sprint{N}): ...` ne sont pas landed
   - FAIL : ≥ 1 P0 ou ≥ 3 P1 → re-conception partielle
4. **Out of scope pour l'audit** — liste explicite de ce que
   l'auditeur ne doit PAS rebattre (les D1..D5 gelées, les
   scope cuts, les choix de pin de dep)
5. **Livrable final attendu** — format exact de
   `audit_findings.md` + critère de clôture

### 2.5 audit_findings.md — le rapport d'audit indépendant

**Pas écrit par l'agent du sprint N.** Produit par la session
fraîche qui démarre le sprint N+1, en Phase 0 de ce sprint
suivant. Joue le `sprint{N}_audit_plan.md` et écrit son
verdict.

Sections canoniques (pattern Sprint 6 audit_findings) :

1. **Auditeur** — id de session, timebox réellement observé
2. **Tip audité** — SHA master que l'auditeur a pris comme
   base
3. **Verdict global** — PASS / CONDITIONAL PASS / FAIL
4. **Une section par track** avec son verdict (PASS /
   CONCERN / FAIL) et sa liste de findings
5. **Findings list sorted by severity** — table récap P0 →
   P3
6. **Commits fix attendus** — si verdict CONDITIONAL PASS,
   liste des `fix(sprint{N}): ...` à landed avant que le
   N+1 Phase A démarre
7. **P2 à logger en tech debt** — items qui vont dans
   `docs/shell/PATTERNS.md` ou `docs/rust/PATTERNS.md` sans
   code change
8. **P3 laissés sans action** — nits explicitement ignorés
9. **Notes on audit completeness** — ce que l'auditeur a
   skippé (timebox) et pourquoi

Exemple vécu : Sprint 6 audit_findings a produit un
**CONDITIONAL PASS** avec 2 P1 + 8 P2 + 7 P3. Les 2 P1
(F-1 ctrl-k case-insensitive + A-3 cross-language canonical
fixture) ont été fixés en 2 commits `fix(sprint6): ...`
landed sur master AVANT le premier commit Sprint 7 Phase A.

---

## 3. Audit gate pattern — la convention permanente

Instauré en fin de Sprint 6 après constat que
`sprint6_verification.md` est une auto-attestation sans
valeur de vérification indépendante. Documenté dans :

- `C:\Users\FlowUP\.claude\projects\C--Users-FlowUP-Documents-Code-nexus\memory\sprint_audit_gate.md`
- `.planning/sprint6_kickoff.md` §8 « Audit gate pattern (convention permanente) »

**Règle** : à partir de Sprint 7, chaque sprint suit ce cycle
strict :

### 3.1 Phase 0 — Audit du sprint précédent (blocking gate)

- Session Claude Code fraîche, sans historique du sprint
  qu'elle audite
- L'utilisateur ouvre cette session avec un prompt qui
  pointe vers `sprint{N-1}_audit_plan.md` comme feuille de
  route
- L'auditeur NE LIT PAS les fichiers PATTERNS.md
  correspondants avant d'avoir formé son opinion — pour
  challenger, pas ratifier
- Produit `sprint{N-1}_audit_findings.md` avec verdict
  PASS / CONDITIONAL PASS / FAIL
- **P0 + P1 doivent être fixés** en commits
  `fix(sprint{N-1}): ...` atterissant sur master AVANT le
  premier commit Phase A du sprint en cours
- **P2 loggés** dans `docs/shell/PATTERNS.md` ou
  `docs/rust/PATTERNS.md` tech debt sections
- **P3 optionnels** — laissés tels quels

### 3.2 Phases A..E — contenu du sprint

Les vraies livraisons de code. Une phase = un commit
atomique feat. Pattern Sprint 6/7 :

- Phase A — skeleton / fondations du sprint
- Phase B, C, D — itérations successives ajoutant des
  capacités
- Phase E — polish, intégration, tests Playwright / Vitest
- (optionnel Phase F — scope complexe, ex Sprint 6 où D a
  été split en D+E)

### 3.3 Phase de sortie — deux livrables obligatoires

Dans le même commit `docs(sprint{N}): verification + audit
plan for Sprint N+1` :

1. `sprint{N}_verification.md` — self-report fail-fast
2. `sprint{N}_audit_plan.md` — plan que Sprint N+1 Phase 0
   jouera + update de `docs/shell/PATTERNS.md` +
   `docs/rust/PATTERNS.md` avec les nouveaux patterns et
   tech debt items

**Sans ces deux fichiers, le sprint ne peut pas être fermé.**

### 3.4 Pourquoi

Un fail-fast self-reporté confirme « le code compile, les
tests passent ». Seule une vérification faite par un
contexte indépendant, qui ne connaît pas les compromis pris
pendant l'écriture, peut challenger les choix et trouver les
blind spots.

Exemples réels de blind spots trouvés en rétro-analyse
Sprint 6 :

- `ButtonBlock.task_submit` shippé comme dead code
  (`console.warn`)
- Ctrl+K jamais testé dans un vrai browser → cassé sur AZERTY
- Snapshot cross-langue Python-only qui ne détecterait pas
  une drift Zod
- `legacy_descriptor` fallback sans sentinelle de retrait
  dans le code

Ces findings auraient été invisibles à un self-report parce
que l'agent les a écrits et les considérait comme corrects.

### 3.5 Exception

Seule exception au pattern : l'utilisateur demande
explicitement de skipper l'audit (hotfix urgent par
exemple). Dans ce cas, noter dans le kickoff du sprint
« Phase 0 audit skipped per user decision YYYY-MM-DD » et
prévoir un audit rétroactif au sprint suivant.

---

## 4. Phase breakdown dans un sprint

Chaque phase respecte une discipline stricte :

### 4.1 Un commit atomique par phase

Pattern commit :

```
feat(scope): Sprint N Phase X — titre court

Body structuré :
- Contexte (1-2 lignes)
- Fichiers touchés avec rationale (pas seulement la liste)
- Delta de tests cumulé :
    Rust workspace:           193 → 254 (+61 Phase X)
    Python coord:             47+1 → 57+1 (+10 daemon proxy)
    Vitest unit:              99 → 114 (+15 daemon.ts)
    Playwright:               10 → 13 (+5 Phase E, -2 stub-pages)
- Scope cuts honoured (liste explicite de NOT)
- Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
```

Si une phase a besoin d'un fix post-commit (pattern Sprint 2
`de9589d` / `ed2ea76` ou Sprint 6 gate `05c96c4..8fbe07b`),
le fix vit dans un commit séparé
`fix(sprint{N}): description` — jamais d'amend.

### 4.2 Discipline de staging

Staging explicite (jamais `git add -A`) :

```bash
git add \
  crates/nexus-foo/src/bar.rs \
  packages/nexus-foo/tests/test_bar.py \
  ...
```

Protège contre l'inclusion accidentelle de fichiers de
secrets, binaires, caches temporaires.

### 4.3 Verification obligatoire avant commit

Avant chaque commit de phase :

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked

uv run ruff format --check packages/
uv run ruff check packages/
uv run pytest packages/nexus-sdk/tests/ -q
uv run pytest packages/nexus-coordinator/tests/ -q
uv run pytest packages/nexus-app-gov/tests/ -q

cd web
npx tsc --noEmit -p tsconfig.app.json
npm run lint
npm run test:unit
npm run build
npm run size
npx playwright test
bash scripts/scan-en-strings.sh
```

Tout rouge bloque le commit. Aucune exception « je commit et
je fix après » — le fix doit être dans le même commit ou
déclenche un nouveau cycle.

---

## 5. Memory system externe

Cinq fichiers persistés hors repo, lus par chaque session
fraîche au démarrage :

```
C:\Users\FlowUP\.claude\projects\C--Users-FlowUP-Documents-Code-nexus\memory\
├── MEMORY.md                    # index, une ligne par entrée
├── user_profile.md              # FlowUP, dev francophone, RTX 5080
├── nexus_grid_pivot.md          # état projet, roadmap sprints, tips
├── sprint_audit_gate.md         # la convention documentée §3
├── feedback_approach.md         # « no band-aids, stabilize first »
└── (autres entrées ponctuelles)
```

**MEMORY.md** est toujours chargé dans le contexte de la
session. Les autres fichiers sont lus sur demande.

### 5.1 Quand mettre à jour la memory

- **Après chaque sprint fermé** : update de
  `nexus_grid_pivot.md` avec le nouveau tip master et les
  compteurs de tests
- **Quand l'utilisateur donne un feedback** :
  `feedback_approach.md` gagne une ligne
- **Quand l'utilisateur corrige l'agent** : entrée feedback
  dédiée avec « Why: » et « How to apply: » structurés
- **Jamais** : code patterns, git log, who-changed-what —
  ces infos sont déjà dans le repo

### 5.2 Format MEMORY.md

```markdown
- [User profile](user_profile.md) — hook court < 150 chars
- [Project state](nexus_grid_pivot.md) — hook court
- ...
```

Chaque entrée est une ligne < 150 chars. Pas de paragraphes,
c'est un index.

---

## 6. Conventions non négociables

### 6.1 Décisions Day 0 gelées

Chaque kickoff §4 contient D1..D5 (parfois D1..D9) qui sont
figés pour toute la durée du sprint. L'agent exécuteur et
l'auditeur ne peuvent PAS les rebattre. Si l'auditeur trouve
un argument technique new qui les invalide, il le note comme
« à rouvrir en Sprint N+1 Day 0 » mais ne bloque pas le
sprint en cours.

### 6.2 Scope cuts stricts

Chaque sprint liste explicitement ce qu'il ne fera PAS, et
pour quel sprint cet item est réservé. L'auditeur fait un
`grep` pour vérifier qu'aucune ligne de code ne touche un
scope cut. Un item qui fuite du scope cut est un **P1**
(blocker sprint suivant).

### 6.3 Pas de band-aid fixes

Quand un test échoue, l'agent cherche la cause racine, pas
le contournement. Exemple Sprint 7 Phase A : un test unit
singleton a échoué parce que le binary de test cargo
(`nexus_shell_daemon-<hash>`) ne matchait pas le nom de
production (`nexus-shell-daemon`). Le band-aid aurait été
de supprimer le test. Le deep fix a été de normaliser
hyphen/underscore dans `process_name_matches` et de
restaurer le test.

Discipline feedback :
`feedback_approach.md` le documente — « No band-aids,
parallel agent teams, real benchmarks, stabilize before
extending ».

### 6.4 Langue

- Docs, commentaires, commits bodies, chat utilisateur :
  **français**
- Code, identifiants, commit titles, logs, errors :
  **anglais**
- `docs/*/PATTERNS.md` est majoritairement **anglais**
  parce qu'il est consommé par l'agent et par des
  contributeurs potentiels
- `.planning/sprint*_*.md` est **français**

Le script `web/scripts/scan-en-strings.sh` est un guard qui
vérifie que le code React côté utilisateur reste en
français (les strings affichées). Rouler à chaque commit
touchant `web/src/pages/` ou `web/src/components/`.

### 6.5 Pas d'emojis dans le code ni dans les commits

Sauf demande explicite utilisateur. Le feedback
`feedback_approach.md` contient cette règle.

### 6.6 Pins de dépendances

- iroh 0.97 (pin exact, upgrade manuel en Sprint dédié)
- iroh-blobs 0.99
- axum 0.7 (pas 0.8, breaking trait lifetime)
- tower-http 0.6 features `["cors"]`
- sysinfo 0.32
- pyo3 0.28 / pyo3-async-runtimes 0.28

Toute dérive doit être justifiée dans un commit body avec
rationale et test de non-régression.

---

## 7. Comment démarrer un nouveau sprint

Template prompt à coller dans une session fraîche :

```
État au démarrage ({date}, master tip `{SHA}`) :
  - Sprint {N-1} CLOSED côté code. Résumé très court.
  - Compteurs de tests verts (Rust / Python / Vitest /
    Playwright / size-limit)
  - Audit gate de Sprint {N-1} : PASS / CONDITIONAL PASS /
    (pas encore joué, à faire en Phase 0)

Lis d'abord :
  1. memory MEMORY.md + nexus_grid_pivot.md +
     sprint_audit_gate.md + feedback_approach.md
  2. docs/claude/README.md (le présent fichier)
  3. .planning/sprint{N-1}_audit_plan.md (si Phase 0 gate
     à jouer) OU .planning/sprint{N}_kickoff.md + plan.md
     (si gate déjà fermée)

Ce que tu produis :
  - Si Phase 0 gate : sprint{N-1}_audit_findings.md +
    fix(sprint{N-1}): ... commits pour les P0/P1
  - Sinon : commit feat(scope): Sprint N Phase X — titre
    avec discipline §4 de docs/claude/README.md

Langue : français docs, anglais code. Pas d'emojis. No
band-aids. Scope cuts respectés.
```

### 7.1 Au tout premier sprint (nouveau projet)

Template différent — il n'y a pas de `sprint{N-1}` à
auditer :

1. Écrire `nexus_grid_pivot.md` (memory) avec la roadmap
   haut niveau
2. Écrire `sprint0_kickoff.md` + `sprint0_plan.md` dans
   `.planning/`
3. Valider les D1..D5 avec l'utilisateur avant d'écrire du
   code
4. Enchaîner les phases A..F
5. À la sortie, écrire `sprint0_verification.md` +
   `sprint0_audit_plan.md` pour que la première session
   fraîche de Sprint 1 puisse jouer la gate

---

## 8. Comment auditer un sprint précédent (Phase 0 gate)

Template prompt déjà produit dans l'historique de la
session Sprint 7 Phase F. Résumé :

1. Session fraîche sans historique du sprint audité
2. Lire dans l'ordre : memory → git log du sprint →
   kickoff → plan → verification → audit_plan
3. **Ne pas lire** `PATTERNS.md` correspondants avant
   d'avoir formé une opinion track par track
4. Jouer les 9 tracks (A..I typiquement) avec la méthode
   concrète du audit_plan
5. Écrire `sprint{N-1}_audit_findings.md` avec findings
   ventilés P0 / P1 / P2 / P3
6. Si P0 ou P1 : produire les commits `fix(sprint{N-1}):
   ...` avant la fermeture de la gate
7. Committer le findings doc + les fix dans master
8. Retourner le verdict à l'utilisateur qui ouvre alors
   la Phase A du sprint en cours

Timebox suggéré : 2-3h. Le signal prime sur le volume.

---

## 9. Anti-patterns rencontrés et à éviter

### 9.1 Band-aid fix quand un test échoue

**Bad** : supprimer le test ou ajouter un `#[ignore]`.
**Good** : diagnostic root cause, deep fix. Voir §6.3.

### 9.2 Amend ou force push

**Bad** : `git commit --amend` après qu'un commit est
landed, `git push --force`.
**Good** : nouveau commit `fix(sprint{N}): ...` avec
rationale dans le body.

### 9.3 Scope creep pendant une phase

**Bad** : pendant Phase C d'un sprint, l'agent touche un
fichier hors scope « tant que j'y suis ».
**Good** : noter en tech debt, respecter le scope de la
phase, le fix devient son propre commit ou sa propre phase.

### 9.4 Commit `feat(sprint{N}): Phase X` sans body riche

**Bad** : `feat(sprint7): Phase A`.
**Good** : le body contient fichiers touchés, delta de
tests cumulé, scope cuts respectés, et ideally un court
paragraph sur le « pourquoi » des choix. Les commits sont
la documentation de référence pour l'auditeur qui relit
à froid.

### 9.5 Commit `docs(sprint{N})` en tant que sortie qui mélange
     doc + code

**Bad** : le commit de sortie Phase F contient aussi un
petit ajout de code « oublié ».
**Good** : Phase F est strictement docs. Un fix de code
oublié devient un `fix(sprint{N}): ...` séparé, même si
c'est 3 lignes.

### 9.6 Lire les PATTERNS.md avant d'auditer

**Bad** : l'auditeur lit `docs/shell/PATTERNS.md` P9 et
se dit « OK c'est bien justifié, PASS ».
**Good** : l'auditeur forme son opinion d'abord sur le
code, puis compare à ce que PATTERNS.md prétend. Si les
deux divergent, c'est un finding.

---

## 10. Table de cross-reference des sprints passés

Historique des 8 sprints livrés (mise à jour à chaque fin
de sprint) :

| Sprint | État | Tip fermeture | Nb commits | Docs planning présents |
|---|---|---|---|---|
| 0 | DONE | `stabilize/compute` mergée | 9 | - |
| 1 | DONE | `e631325` | - | - |
| 2 | DONE + audité rétro | `ed2ea76` | 6 | audit rétro dans `audit_sprint2/` |
| 3 | DONE | `9476be8` | 12 (W1..W12) | `sprint3_verification.md` |
| 4 | DONE | `3b5c162` | 9 | `sprint4_kickoff`, `_plan`, `_verification`, `_verify_prompt` |
| 5 | DONE | `cdf4467` | 9 | `sprint5_kickoff` (monolithique), `_plan`, `_verification` |
| 6 | DONE + CONDITIONAL PASS levé | `504c6aa` puis `2926383` post-gate | 8 + 10 (gate) | 4 docs + `audit_findings` |
| 7 | DONE | `9cc0796` | 8 | 4 docs + attend `audit_findings` du Sprint 8 Phase 0 |
| 8 | DONE + CONDITIONAL PASS levé | `9339bb6` | 7 | 4 docs + `audit_findings` |
| 9 | DONE + CONDITIONAL PASS levé | `eb81c27` puis `48b332a` post-gate | 7 + 2 (gate) | 4 docs + `audit_findings` |
| 10 | DONE | `d07bfcf` (pre-Phase F) | 5 | 4 docs (kickoff, plan, verification, audit_plan) |

Sprint 6 est **le premier** à avoir les 4 docs planning
complets dès le démarrage. Sprint 7 est **le premier cycle
complet** de l'audit gate pattern. Sprint 10 est **le premier
sprint ops** (CI/CD + VPS deployment, pas de code applicatif).

---

## 11. Fichiers pointant vers ce document

Pour qu'une session fraîche trouve cette doc :

- `C:\Users\FlowUP\.claude\projects\C--Users-FlowUP-Documents-Code-nexus\memory\MEMORY.md`
  doit contenir une ligne
  `- [Claude workflow](docs/claude/README.md) — sprint
  lifecycle + audit gate pattern + conventions`
  (à ajouter si absent)
- `CLAUDE.md` à la racine du repo peut mentionner
  `docs/claude/README.md` dans sa section « Doing tasks »
  si l'utilisateur veut que toutes les sessions le
  chargent par défaut

---

## 12. Évolution du système

Cette méthode de travail est un pattern vivant. Chaque
sprint peut proposer une amélioration via sa Phase F qui
documente soit :

- un nouveau pattern dans `PATTERNS.md` (exemple Sprint 7
  P9 = proxy daemon discipline)
- un nouveau item dans `sprint_audit_gate.md` memory
- une mise à jour de `docs/claude/README.md` si la
  convention elle-même change

Changements majeurs à surveiller :

- Sprint 8 : premier cycle d'audit gate rétroactif complet
  (la session fraîche de Sprint 8 Phase 0 joue
  `sprint7_audit_plan.md` — c'est le vrai test du pattern
  inventé en fin Sprint 6)
- Sprint 9 : branding / renommage — risque de drift dans
  les noms (nexus-grid vs SBFB vs autre)
- Sprint 10 : release v1.0 + 3 VPS bootstrap — premier
  sprint avec déploiement infra, le pattern devra absorber
  un nouveau type de tâche (ops, pas seulement code)

Ce fichier doit être mis à jour quand ces évolutions
landent, pas avant.
