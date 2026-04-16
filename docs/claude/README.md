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
- [`docs/claude/TOOLING.md`](TOOLING.md) — process tooling au-dessus
  de Claude Code vanilla (hooks, skills Trail of Bits, agents review,
  Semgrep regles SBFB). Install en un script, ajoute 5 couches de
  qualite independantes au cycle sprint.

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

Depuis Sprint 16, les sprints sont rangés selon le **PARA pattern** :

```
.planning/
├── README.md              # explication détaillée du layout
├── active/                # UN seul sprint à la fois
│   └── sprint{N}_*.md     # 5 docs du sprint en cours
├── archive/v{X}/          # sprints fermés, groupés par version livrée
│   ├── v1.0/              # S0-13
│   └── v1.1/              # S14-15
├── codebase/              # cross-sprint, cartographie codebase (snapshot 2026-04-06)
├── research/              # cross-sprint, notes de recherche
└── *_ROADMAP.md           # docs thématiques evergreen
```

Pour le sprint N **en cours**, les 5 documents en
`.planning/active/sprint{N}_*.md` sont :

```
sprint{N}_kickoff.md        # écrit EN ENTRÉE du sprint
sprint{N}_plan.md           # écrit EN ENTRÉE du sprint
sprint{N-1}_audit_findings.md # écrit en Phase 0 (gate du sprint précédent)
sprint{N}_verification.md   # écrit EN SORTIE du sprint
sprint{N}_audit_plan.md     # écrit EN SORTIE du sprint
```

Les 4 premiers (kickoff/plan/verification/audit_plan) sont livrés
par l'agent qui exécute le sprint. L'audit_findings est produit
par une session Claude Code fraîche qui joue l'audit au démarrage
du sprint N+1.

À la **clôture** du sprint N (= ouverture Sprint N+1), ses 5 docs
sont déplacés via `git mv` depuis `active/` vers `archive/v{X}/`
(la version dont le sprint fait partie). Le nouveau sprint N+1
écrit ses propres docs dans `active/`. Détail dans
[`.planning/README.md`](../../.planning/README.md) §« Cycle de
vie d'un sprint ».

### 2.1 kickoff.md — le contrat d'entrée

Écrit par l'agent qui démarre le sprint. Rôle : figer les
décisions non-rebattables avant d'écrire la moindre ligne de
code.

Sections canoniques (pattern Sprint 6/7) :

1. **Constat d'entrée** — quel est le tip master au début,
   quels tests passent, quels commits ont landé depuis le
   sprint précédent, quel est le verdict de l'audit gate
2. **Goal en une phrase** — ce que le sprint promet de livrer.
   Le goal §2 reste litteraire (lecture humaine) mais **DOIT
   pointer explicitement vers `verification.md §Fail-fast checklist`
   comme source of truth mesurable** (G3). Exemple :

   > "Le sprint durcit la chaine transport P2P en imposant... —
   > debloquant S21 rate-limit fin S21. **Critere SMART : 28+
   > rows fail-fast verts au verification.md, mesure binaire au
   > Phase F wrap-up.**"

   Sans cette liaison, "atteint ?" est gameable a la sortie. La
   verification.md fail-fast checklist existe deja (24-32 rows
   executables, cf. §2.2 plan §5) — c'est le critere SMART du
   sprint. Le goal §2 n'a pas besoin de 3 critres SMART
   supplementaires (duplication §2/§5/§10), juste un pointeur.
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
   - Fichiers ajoutés / modifiés (chemin + 3 à 5 lignes de
     structure). **Pas d'estimation LOC** — quand on vise la
     solution la plus poussée, la taille finale est inconnue
     avant que la recherche soit terminée, et les estimations
     amont biaisent vers la solution minimale qui rentre dans
     l'estimation. Cf. §6.7.
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

#### 5.1.1 Carry-over discipline (G6 — fusion manuelle, pas merge auto)

Anti-pattern observe : `project_nexus_state.md` reste inerte 7+ jours
apres le pivot 2026-04-10 qui l'a rendu obsolete. Les memory files
sont read-mostly entre sessions, updates rares.

**Pattern correct** (utilise les artefacts existants, pas de nouveau
fichier) :

1. Chaque session de sprint termine en ecrivant `sprint{N}_
   verification.md §5 "Findings carry-over for memory"` listant les
   items qui valent d'etre persistes (P0/P1 + decisions long-terme +
   gotchas surprenants), max 5 items par sprint.
2. **Au kickoff S{N+1}**, la session fraiche fusionne MANUELLEMENT
   ces items dans la memory concernee (`nexus_grid_pivot.md`,
   `feedback_approach.md`, ou nouveau fichier dedie). Pas de merge
   automatique (conflits + pollution).
3. **Pre-kickoff check obligatoire** (G2 lite) : avant ecrire le
   nouveau kickoff, l'agent verifie les dates de mtime des memory
   files vs tip master. Si un memory file n'a pas ete touche depuis
   > 2 sprints, ouvrir une question explicite "ce fichier est-il
   encore pertinent ?".

   ```bash
   # Check rapide
   ls -la "$HOME/.claude/projects/C--Users-FlowUP-Documents-Code-nexus/memory/"
   git log -1 --format=%cd master
   ```

**Pas de** `last_session_findings.md` mergé auto : merge conflicts
garantis + pollution graduelle de noise.

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

### 6.1.1 Design Review Board — reviewer independant pre-gel D1..D5 (G1)

Anti-pattern observe Sprint 19 : les "Rejete" sous chaque D1..D5 sont
ecrits par le **planner lui-meme**. Pas de challenge independant.
Resultat : D2 PoW Hashcash SHA256 cite "Tor 2023" sans verifier que
Tor a abandonne Hashcash en 2023 pour Equi-X. Le rationale est
plausible mais circulaire.

**Procedure** :

Avant de figer D1..D5 dans `kickoff §4`, le planner :

1. Ecrit un **draft** des decisions avec sources cited (context7 +
   WebSearch + URL).
2. Lance un **agent Explore independant** (subagent_type=Explore,
   session fraiche, contexte minimal) avec le prompt :

   ```
   Tu review un draft de decisions Day-0 pour Sprint {N}.
   Pour chaque D1..D5, produire un scoring report :
   - ✅ source recente (<= 90j) + alternative concurrente verifiee
   - ⚠️ source presente mais pas a jour OU alternative non comparee
   - ❌ pas de source OU choix techniquement contredit par WebSearch 2026
   Ne propose PAS de solution alternative — tu signales les angles
   morts seulement. Le planner reste owner de la decision finale.
   ```

3. L'agent ecrit `.planning/active/sprint{N}_design_review.md` avec
   le scoring report (5-15 min, prompt court).
4. **Le planner reste owner mais doit acknowledge chaque ⚠️ et ❌
   explicite** dans le kickoff §4 (paragraphe "Acknowledged review
   findings"). Pas de veto reviewer, pas de stalemate.

**Avantage vs adversary-agent** : pas de "trouve 3 raisons de
rejeter" (genere du bikeshedding), juste un signal de qualite des
sources. Le reviewer ne propose pas de solution → pas de bataille
d'ego entre planner et reviewer.

**Quand skipper** : sprint pure-docs (S17), hotfix (cas D §7),
phase trivial refactor sans decision Day-0.

### 6.2 Scope cuts — stricts DANS un sprint, reevalues ENTRE sprints

Chaque sprint liste explicitement ce qu'il ne fera PAS, et
pour quel sprint cet item est réservé. L'auditeur fait un
`grep` pour vérifier qu'aucune ligne de code ne touche un
scope cut. Un item qui fuite du scope cut est un **P1**
(blocker sprint suivant).

**Regle critique pour les sessions fraiches** : les scope
cuts d'un sprint N sont des decisions de priorisation, pas
des verites techniques. Au kickoff du sprint N+1, chaque
item « differe a Sprint N+2+ » doit etre reevalue contre
le code actuel. Lancer un agent Explore pour mesurer le gap
reel en LOC avant de re-reporter l'item. Si le gap est
< 500 LOC et que l'item sert le goal du sprint, l'inclure.

Exemple reel : « blob web apps (iframe) » a ete scope-cut
de Sprint 5 a Sprint 13+ sur 6 sprints consecutifs. A
Sprint 12, 60% du code existait deja (WebAppFrame.tsx
fonctionnel, CAS upload avec GET /files/{sha256}, BlobsClient
complet). Le gap reel etait ~300 LOC, pas un sprint entier.

**Ne jamais propager un scope cut sans verification.** C'est
le pattern le plus couteux identifie dans le projet — il
retarde des features quasi-pretes en les traitant comme des
chantiers majeurs.

#### 6.2.1 Cap carry-overs : max 2 par sprint (G7)

Anti-pattern observe Sprint 18→19 : C-1 (DHT quorum wire) reporte
S19 documente seulement en commit body + PATTERNS.md, sans entree
explicite dans `sprint19_audit_plan.md`. Sprint 19 Phase B reporte
runtime gossip wire S20 sans aucune entree carry. Le report devient
gratuit → choix par defaut sous pression de fin de sprint.

**Regle** :

- **Max 2 carry-overs par sprint** (scope cuts vers S{N+1} via Phase
  B/C/etc. report). Au-dela, soit livrer dans le sprint courant,
  soit abandonner explicitement (entree `docs/DEPRECATED.md` avec
  rationale).
- **Phase F wrap-up genere `sprint{N+1}_carry_summary.md`** (pas
  optionnel) listant les carry-overs avec :
  - ID + description (1 ligne)
  - Source : phase qui a reporte + commit SHA
  - Severite : P1 (Gate-blocker) / P2 (debt) / P3 (cosmetic)
  - Owner : `<github-handle>` ou `S{N+1}` par defaut
- **Kickoff S{N+1} doit re-confirmer** chaque carry via une ligne
  explicite dans §6 "Items carry/dette" : `[x] C-1 carry confirme
  pour S{N+1} Phase A` ou `[deferred] C-1 differe S{N+2}`.

Decision P1 vs P2 vs P3 : prise par l'auditeur Phase 0 du sprint
suivant en jouant `sprint{N}_audit_plan.md`. Pas par l'agent qui
livre le carry (auto-evaluation biaisee).

**Pourquoi cap a 2** : empiriquement (S17→S18→S19), 2 carry-overs
sont absorbables sans diluer le scope du sprint suivant. 3+ degrade
le sprint cible en "rattrapage du sprint precedent" → escalade.

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

### 6.7 Horizon long terme + documentation AVANT code

Règle forte (cf. memory `feedback_approach.md` §« Regle critique :
horizon long terme + documentation AVANT code ») :

- **Toute décision technique s'évalue à 2 ans / 10× charge / 100
  contributeurs**, pas au sprint courant. La dette courte-vue coûte
  plus cher que l'effort initial d'une solution durable.
- **Ordre correct** : (1) recherche (context7 + WebSearch + registry
  local + docs officielles), (2) design doc court dans
  `.planning/research/` ou `docs/{domain}/` avec alternatives +
  rationale + limites connues, (3) self-challenge, (4) code.
  Inverser cet ordre = 80 % des bugs architecturaux rétrospectifs.
- **Toujours la solution la plus poussée techniquement**, pas la
  plus simple à livrer. Si une lib plus auditée / FIPS / fuzzed /
  SLSA existe, la choisir. Si un pattern type-safe / compile-time
  / zero-copy / const-time existe, le choisir.
- **Le design doc est un livrable du sprint**, pas un nice-to-have.
  Avant Phase A : trace écrite des alternatives considérées, libs
  comparées (versions + CVE + last-audit-date via context7),
  rationale du choix retenu. `nexus-phase-auditor` bloque §Research
  consulte vide sur APIs crypto / spec standardisées.
- **D1..D5 Day 0 citent explicitement les alternatives rejetées**.
  Sans rationale, ce n'est pas une décision, c'est un réflexe.
- **Défaut "deep" en cas de doute** entre deep et quick — l'user
  re-demande deep 3× sur 3, autant l'assumer d'entrée.
- **Pas d'estimation LOC en amont.** Aucun plan ni kickoff ne
  fournit un chiffre "~NNNN LOC sur N phases". Raisons :
  (1) la taille finale dépend de la solution trouvée après
  recherche, elle n'est pas connaissable au plan ; (2) une
  estimation amont devient un plafond psychologique — l'agent
  tronque la solution la plus poussée pour rentrer dans le
  budget ; (3) la "vitesse de delivery" se mesure aux phases
  livrées avec tests verts, pas aux LOC produites. La seule LOC
  qui compte est la LOC **rétrospective** (mesure de gap, ex.
  §6.2 "gap réel mesuré ~300 LOC").

Preuve empirique : S14 Keyoxide, S17 VALIDATED_BLUEPRINT, S18
supply-chain → research-first, zero rework majeur.
À l'opposé : S7 singleton band-aid, S18 D-1 wire manquant → code-
first, rework commits. Corrélation directe research/doc amont ↔
réduction debug/rework aval.

### 6.8 Fraîcheur des artefacts long-life — triggers événementiels (G2)

Anti-pattern observé : `HARDENING_ROADMAP.md` écrit S17 (octobre 2025)
hérite Sprint 19 (avril 2026) sans audit fraîcheur. D2 PoW Hashcash
2^18 dérive d'une recommandation S17 ; entre-temps Tor a abandonné
Hashcash pour Equi-X (août 2023). 6 mois entre écriture et
consommation = drift réel.

**Pattern correct** (événementiel, pas compteur jours absolu) :

Tout artefact long-life (`HARDENING_ROADMAP.md`, `PATTERNS.md`,
`VALIDATED_BLUEPRINT.md`, memory `nexus_grid_pivot.md`) **DOIT**
inclure dans son frontmatter :

```yaml
---
written: 2026-04-10
last_validated: 2026-04-16
triggers_revalidate:
  - "iroh release > 0.97"
  - "wasmtime LTS bump"
  - "CVE annonce sur dep critique"
  - "Sprint S+2 commence (S19+2 = S21)"
---
```

**Quand re-valider** (events, pas timer) :

- Une release upstream majeure d'une lib critique cite (iroh, wasmtime,
  arti-client, pkarr, libp2p) → re-scan §pertinente.
- Un CVE annonce sur une dep listee → re-scan §securite.
- Le sprint S+2 demarre apres l'ecriture → re-scan §roadmap pour ce
  sprint specifique.
- Un finding audit S{N} contredit le contenu → re-scan immediate.

**Discipline session-start** (cas A/B/C du prompt §7) :

```bash
# Verifier triggers actifs
grep -lE 'triggers_revalidate' docs/security/*.md docs/rust/PATTERNS.md
# Pour chaque match, l'agent verifie si un trigger s'est realise depuis last_validated
```

**Pas de** compteur "stale apres N jours" : un doc fige sur un
sujet dormant peut rester valide 1 an. Un doc sur une lib en pleine
evolution est stale apres 1 release. C'est l'event qui declenche,
pas le calendrier.

**Maintenance** : `last_validated` mis a jour au commit qui re-audite
la section. Pas obligatoire en lecture seule.

---

## 7. Prompt générique de bootstrap session fraîche (v2)

Ce prompt est conçu pour être collé tel quel au démarrage d'une
nouvelle session Claude Code sur le projet. Il ne suppose **pas**
de connaître l'état actuel — l'agent détermine seul dans quel cas
il est, en commençant par un **bloc pre-flight** d'un seul copier-
coller, puis en routant vers la procédure du cas détecté.

### 7.1 Le prompt à coller

```
Tu démarres une session sur nexus-grid (SBFB). Ne lis RIEN tant
que tu n'as pas exécuté le pre-flight ci-dessous — il te dit
quels fichiers sont vraiment pertinents pour ton cas.

# === Pre-flight (un seul copy-paste, lis tout l'output) ===

git log --oneline -10
git status --short
ls .planning/active/
ls .planning/archive/
head -1 docs/claude/SPRINT_LOG.md && grep -E "^## v[0-9]" docs/claude/SPRINT_LOG.md
grep "^- \[SBFB pivot\|tip \`" "$HOME/.claude/projects/C--Users-FlowUP-Documents-Code-nexus/memory/MEMORY.md" || true
grep "Tip \`" "$HOME/.claude/projects/C--Users-FlowUP-Documents-Code-nexus/memory/nexus_grid_pivot.md" | head -1

# G2 — triggers événementiels actifs sur artefacts long-life
grep -lE 'triggers_revalidate' docs/security/*.md docs/rust/PATTERNS.md docs/shell/PATTERNS.md 2>/dev/null

# G6 — fraîcheur memory vs tip master (ouvrir question si > 2 sprints sans touch)
ls -la "$HOME/.claude/projects/C--Users-FlowUP-Documents-Code-nexus/memory/" 2>/dev/null | head -20

# === Détection du cas ===

Compare ce que tu vois avec :

  Cas A — Audit gate à jouer
    Signal : .planning/active/ vide OU contient SEULEMENT le
             kickoff/plan d'un sprint dont le précédent vient de
             fermer (audit_findings absent dans active/ ET dans
             archive/v{X}/).
    Lecture ciblée : .planning/archive/v{X}/sprint{N-1}_audit_plan.md
    Mode : audit indépendant, pas implémentation.
    Livrable : .planning/active/sprint{N-1}_audit_findings.md +
               commits fix(sprint{N-1}): ... pour P0/P1.
    Verdict G4 (rigor signal) : 0 P0/P1 ET 0 P2+ trouve = CONCERN
               (pas PASS — re-auditer dimension manquee). PASS exige
               >=1 P2+ documente. Cf. §6.1.1 + agent-auditor.

  Cas B — Sprint en cours
    Signal : .planning/active/ contient sprint{N}_kickoff.md +
             sprint{N}_plan.md mais pas verification.md.
    Lecture ciblée : sprint{N}_plan.md §Phase X (où X = phase
                     suivante non encore committée selon git log).
    Mode : implémentation atomique.
    Livrable : 1 commit feat(scope): Sprint N Phase X — titre.
    Avant CHAQUE commit phase (G5) : invoquer skill
               nexus-phase-review Step 1bis "working tree audit"
               -> categoriser PHASE/CRAFT/DEBT/NOISE chaque modif,
               splitter en chore(planning) si CRAFT, refuser NOISE.
               Body commit DOIT contenir section "Working tree audit".
    Avant scope cut S+1 (G7) : verifier cap 2 carry-overs/sprint
               max. Si depassement, livrer en sprint courant ou
               ajouter a docs/DEPRECATED.md.

  Cas C — Nouveau sprint à ouvrir
    Signal : .planning/active/ contient au max le
             sprint{N-1}_audit_findings.md avec verdict PASS ou
             CONDITIONAL PASS levé. Le sprint N-1 est complètement
             clos.
    Préalable : lire SPRINT_LOG.md pour décider la version cible
                (continuer v1.x ou ouvrir v1.x+1 selon le thème).
    Pre-research OBLIGATOIRE (G2) : avant figer D1..D5, verifier
                triggers_revalidate sur HARDENING_ROADMAP §3 S{N}
                + memory nexus_grid_pivot.md §Sprint S{N} carry. Si
                trigger active depuis last_validated, re-fetch
                context7 + WebSearch fresh AVANT le draft kickoff.
    Design Review Board (G1, sauf sprint pure-docs ou trivial) :
                apres draft D1..D5 mais AVANT gel, lancer agent
                Explore independant (cf. §6.1.1) -> scoring report
                ⚠️/✅/❌ par decision -> planner ack chaque ⚠️/❌
                explicite dans kickoff §4 paragraphe "Acknowledged
                review findings". Le reviewer ne propose pas, il
                signale les angles morts.
    Goal §2 (G3) : DOIT pointer explicite vers verification.md
                fail-fast checklist comme critere SMART (ne pas
                inventer 3 KPIs supplementaires — duplication).
    Carry-overs (G7) : §6 "Items carry/dette" liste max 2 items
                re-confirmes ligne par ligne `[x] C-N carry confirme
                pour S{N} Phase A` ou `[deferred] -> S{N+1}`.
    Memory carry-over (G6) : fusionner manuellement
                `sprint{N-1}_verification.md §5 Findings carry-over
                for memory` dans nexus_grid_pivot.md / feedback_*.md
                concernes. Pas de merge auto.
    Mode : design + écriture planning.
    Livrable : sprint{N}_kickoff.md + sprint{N}_plan.md +
               sprint{N}_design_review.md (G1) + carry-over
               summary + frontmatter triggers_revalidate sur
               nouveaux docs long-life. D1..D5 a valider AVANT
               toute ligne de code.
    Migration préalable : si sprint N-1 est encore dans active/,
    le déplacer vers archive/v{X}/ via git mv.

  Cas D — Hotfix hors sprint
    Signal : utilisateur demande explicitement un fix urgent.
    Mode : commit fix(...) ciblé, ne touche pas .planning/.
    G5 reste actif : working tree audit obligatoire avant commit
                meme en hotfix.

# === Lecture ciblée par cas ===

Tu lis dans l'ordre les fichiers PERTINENTS pour ton cas, pas
toute la doc. Charger tout sature le contexte pour rien.

  Pour TOUS les cas (lecture commune minimale) :
    1. CLAUDE.md (racine) — projet + pointeur workflow
    2. docs/claude/README.md §3 (audit gate) + §4 (commit
       discipline) + §6 (conventions, dont 6.1.1 + 6.2.1 + 6.8)
    3. memory MEMORY.md (l'index)

  Cas A en plus :
    - .planning/archive/v{X}/sprint{N-1}_audit_plan.md
    - .planning/archive/v{X}/sprint{N-1}_kickoff.md (D1..D5
      gelées à NE PAS rebattre)
    - docs/claude/README.md §3 et §8
    - .claude/agents/nexus-phase-auditor.md (calibration G4 +
      Step 3bis working tree audit G5)

  Cas B en plus :
    - .planning/active/sprint{N}_kickoff.md (D1..D5)
    - .planning/active/sprint{N}_plan.md §Phase X visée
    - docs/claude/README.md §4 (atomic commit, body riche) +
      §6.2.1 (cap carry-overs G7)
    - .claude/skills/nexus-phase-review/SKILL.md (Step 1bis G5
      + Step 6 rigor signal G4)

  Cas C en plus :
    - docs/claude/SPRINT_LOG.md (versions livrées + thèmes)
    - .planning/archive/v{X}/sprint{N-1}_kickoff.md (pour reprendre
      le format) + sprint{N-1}_verification.md §5 carry-over G6
    - docs/claude/README.md §2.1 (goal SMART G3) + §6.1.1
      (Design Review Board G1) + §6.2.1 (cap carry-overs G7) +
      §5.1.1 (memory carry-over G6) + §6.8 (triggers G2)
    - memory nexus_grid_pivot.md (roadmap + compteurs tests)
    - HARDENING_ROADMAP.md frontmatter triggers_revalidate (G2 —
      verifier si trigger actif depuis last_validated)

  Cas D :
    - juste le code touché, rien d'autre
    - .claude/skills/nexus-phase-review/SKILL.md Step 1bis (G5
      working tree audit reste obligatoire meme hotfix)

# === Stale memory check (obligatoire avant tout commit) ===

Compare le `Tip` extrait de memory/nexus_grid_pivot.md avec le
HEAD actuel :

  HEAD_SHA=$(git rev-parse --short HEAD)
  MEM_TIP=$(grep -oE 'Tip \`[a-f0-9]+\`' \
    "$HOME/.claude/projects/C--Users-FlowUP-Documents-Code-nexus/memory/nexus_grid_pivot.md" \
    | head -1 | grep -oE '[a-f0-9]+')

Si HEAD_SHA != MEM_TIP : la memory est en retard d'au moins un
sprint. Lire `git log --oneline ${MEM_TIP}..HEAD` pour rattraper,
NE PAS recommander en se basant sur la memory frozen, mettre à
jour la description frontmatter de nexus_grid_pivot.md à la fin
de la session.

# === Livraison standard (tous cas) ===

Avant d'écrire du code :

  1. Résume en 5-10 lignes : cas détecté, dernier tip master,
     compteurs tests memory vs réel, ce que tu t'apprêtes à faire
  2. Attends confirmation utilisateur sauf si le cas est
     trivialement déterminé (sprint{N}_plan.md §Phase X explicite)
  3. Respecte les D1..D5 figées et les scope cuts du sprint
     courant — ne rebats pas
  4. Pas de band-aid fix, pas d'emoji, pas d'amend, pas de
     force push
  5. Avant chaque commit : verifier toutes les suites pertinentes
     (cf. §7.4 ci-dessous)

Langue : français pour réponses utilisateur, docs planning,
commit bodies. Anglais pour code, identifiants, commit titles.
```

### 7.2 Templates de commit par cas

Format universel : `<type>(<scope>): Sprint N Phase X — titre court`
+ body structuré. Voir §4 pour la discipline générale.

**Cas A — fix post-audit (P0/P1 du sprint précédent)** :

```
fix(sprint{N-1}): <résumé du finding>

Audit Sprint N-1 Phase 0 finding {ID} ({severity}):
<copie verbatim du finding pertinent depuis sprint{N-1}_audit_findings.md>

Root cause : <diagnostic>
Fix : <ce qui change dans le code>
Tests : <suites + delta>

Refs : .planning/active/sprint{N-1}_audit_findings.md §{ID}

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
```

**Cas B — feat (Phase X du sprint en cours)** :

```
feat(scope): Sprint N Phase X — titre court

Contexte : <1-2 lignes pourquoi cette phase>
Fichiers touchés :
  - path/file.rs : <rôle>
  - path/file.py : <rôle>
Delta tests cumulé :
  Rust workspace : NNN -> NNN (+X Phase Y)
  Python coord   : NN+1 -> NN+1 (+X)
  Vitest unit    : NNN -> NNN (+X)
  Playwright     : NN -> NN (+X)
Scope cuts honoured : <items NOT, copie du kickoff §6>

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
```

**Cas C — docs (planning d'ouverture sprint)** :

```
docs(sprint{N}): kickoff + plan for Sprint N

Theme : <résumé en 1 ligne du goal>
Décisions Day 0 figées : D1, D2, D3, D4, D5 (cf. kickoff §4)
Scope cuts : <liste des items NOT, cf. kickoff §6>
Audit gate Sprint N-1 : PASS / CONDITIONAL PASS levé via {SHA}

Phases prévues :
  A — <titre>
  B — <titre>
  ...

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
```

**Cas C — docs (clôture sprint, sortie Phase E)** :

```
docs(sprint{N}): verification + audit plan for Sprint N+1

Verification : NN/NN fail-fast verts, delta tests +NN cumulé
Audit plan : N tracks A..G pour Sprint N+1 Phase 0
PATTERNS.md : <ajouts pattern + tech debt T-NN>

Tip d'entrée : {SHA}
Tip de sortie : {SHA}
Commit stack : {N commits feat/test} + ce commit docs

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
```

**Cas D — hotfix hors sprint** :

```
fix: <résumé court>

Contexte : <pourquoi hors cycle sprint>
Root cause : <diagnostic>
Fix : <ce qui change>
Tests : <validation>

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
```

### 7.3 Détection version cible (Cas C uniquement)

Quand on ouvre un nouveau sprint, savoir s'il appartient à la
version courante (`v1.x`) ou s'il ouvre une nouvelle version
(`v1.x+1`) détermine où ses docs seront archivés.

```bash
# Quelle version est en cours ?
grep -A 1 "^## v" docs/claude/SPRINT_LOG.md | head -4
```

Heuristique de décision :

| Signal | Décision |
|---|---|
| Le sprint continue le thème de la version courante (ex: encore du security hardening dans v1.2) | Reste sur la même version |
| Le sprint ouvre un thème nouveau (ex: passe de "security" à "scaling") | Nouvelle version `v1.x+1` |
| Une release officielle vient d'être publiée | Toujours nouvelle version |
| Doute | **Demander à l'utilisateur** lors de la validation des D1..D5 |

À l'ouverture d'une nouvelle version : créer
`.planning/archive/v1.x+1/` (vide au départ), ajouter une
nouvelle section `## v1.x+1 — <thème>` au-dessus de v1.x dans
`SPRINT_LOG.md`.

### 7.4 Verification avant commit (script unique)

```bash
# Rust
cargo fmt --all --check && \
cargo clippy --workspace --all-targets --locked -- -D warnings && \
cargo test --workspace --locked

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
```

Tout rouge bloque le commit. Pas de `--no-verify`, pas de
`#[ignore]` ajouté pour faire passer. Root cause d'abord.

### 7.5 Mise à jour memory en fin de session

Quand un sprint avance d'au moins une phase ou clôt, mettre à
jour avant de fermer la session :

1. `memory/nexus_grid_pivot.md` frontmatter `description:` —
   nouveau tip + nouveaux compteurs de tests
2. `memory/MEMORY.md` ligne `[SBFB pivot ...]` — résumé court
   sur 1 ligne
3. Si nouveau sprint clos : `docs/claude/SPRINT_LOG.md` row
   ajoutée dans la section v1.x correspondante

Sans cette étape, la prochaine session démarrera avec une
memory stale et le pre-flight §7 le détectera comme un
warning.

### 7.6 Au tout premier sprint (nouveau projet)

Template différent — il n'y a pas de `sprint{N-1}` à
auditer :

1. Écrire `nexus_grid_pivot.md` (memory) avec la roadmap
   haut niveau
2. Écrire `sprint0_kickoff.md` + `sprint0_plan.md` dans
   `.planning/active/` (créer le dossier si besoin)
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

## 10. Historique sprint — pointeur

L'historique des sprints livrés ne vit plus dans ce document
(il deviendrait ingérable à 30+ sprints). Voir :

- **[`SPRINT_LOG.md`](SPRINT_LOG.md)** — table synthétique de tous
  les sprints livrés, regroupés par version majeure (v1.0, v1.1,
  v1.2…), avec tip de clôture, nombre de commits et faits
  saillants
- **[`.planning/active/`](../../.planning/active/)** — les docs du
  sprint en cours (`kickoff`, `plan`, `audit_findings` du sprint
  précédent, `verification`, `audit_plan`)
- **[`.planning/archive/v{X}/`](../../.planning/archive/)** — les
  docs des sprints fermés, regroupés par version livrée

À la clôture d'un sprint N, ses 5 docs `sprint{N}_*.md` sont
déplacés via `git mv` depuis `.planning/active/` vers
`.planning/archive/v{X}/` (la version dont N fait partie), et une
nouvelle row est ajoutée dans `SPRINT_LOG.md`. Détail dans
[`.planning/README.md`](../../.planning/README.md) §« Cycle de
vie d'un sprint ».

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

Changements majeurs surveilles :

- Sprint 8 : premier cycle d'audit gate retroactif complet
  — FAIT, le pattern fonctionne
- Sprint 10 : release v1.0 + 3 VPS bootstrap — premier
  sprint ops, le pattern a absorbe les taches infra
- Sprint 12 : rendu universel cross-node — premier sprint
  avec archive zip + daemon blob-serve + iframe sandbox
- Sprint 13 : bridge postMessage — premier sprint avec
  communication iframe ↔ reseau, open source enforcement,
  et launcher Rust. Le pattern sprint s'est stabilise :
  les sessions livrent 4 phases en une seule session.
