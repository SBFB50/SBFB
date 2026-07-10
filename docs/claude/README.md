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

**Fichiers dépendants** (à mettre à jour si ce document change) :
- `.claude/agents/nexus-audit-gate.md` — implémente Cas A §7.1, cite §3, §8
- `.claude/agents/nexus-sprint-kickoff.md` — implémente Cas C §7.1, cite §2.1, §6.1.1, §6.2.1
- `.claude/agents/nexus-phase-preflight-deep.md` — implémente G8 §6.9, cite §7.1 Cas B
- `.claude/agents/nexus-phase-review-deep.md` — implémente review §4, cite §4.5, §7.4
- `.claude/agents/nexus-phase-auditor.md` — cite §3, §6.9
- `.claude/skills/nexus-phase-review/SKILL.md` — fallback review, cite §4.3, §6.7, §7.4
- `.claude/skills/nexus-phase-preflight/SKILL.md` — fallback preflight, cite §6.9, §7.1
- `.claude/hooks/phase-auditor-gate.sh` — implémente l'audit gate §3
- `.claude/hooks/phase-precommit-lightcheck.sh` — implémente §4.2, Check 9 body format `##`
- `docs/claude/TOOLING.md` — cite §3, §4, §5
- `CLAUDE.md` — pointe vers ce document + table agents §Agents d'orchestration

---

## 0. DÉMARRAGE — prompt à coller + comment lire ce README parfaitement

**Au démarrage d'une session, le hook `SessionStart`
(`.claude/hooks/session-start-autoinstall.sh`, matcher `"*"` = toutes les
sources `startup`/`resume`/`clear`/`compact`) injecte automatiquement la
directive de bootstrap : il t'impose de lire CE README vivant sur disque (le
disque fait autorité) avant toute action.** Le bloc ci-dessous n'est donc
**plus à coller en routine** — c'est un **secours**, à utiliser uniquement si
aucune directive `[session-start]` n'apparaît (hook désactivé, autre client).
Ne le colle jamais en doublon de l'injection du hook : une copie collée peut
être **périmée** alors que le disque, lui, est à jour (c'est exactement la
cause du drift que ce mécanisme corrige). C'est le prompt de bootstrap
canonique (version courte / pointeur). La version longue/détaillée vit en
**§7.1**, entre les marqueurs `<!-- BOOTSTRAP:BEGIN -->` et
`<!-- BOOTSTRAP:END -->` (greppables, drift-proof, insensibles au numéro
de ligne).

```
Tu démarres une session sur nexus-grid (SBFB).

SOURCE DE VÉRITÉ UNIQUE : docs/claude/README.md. Avant TOUTE action (avant
le moindre Read d'un autre fichier, avant le pre-flight, avant de détecter
le cas), lis INTÉGRALEMENT le bloc de bootstrap §7.1. Pour cibler la plage
exacte sans deviner : Grep BOOTSTRAP:BEGIN et BOOTSTRAP:END dans
docs/claude/README.md -> 2 numéros de ligne -> Read en UN appel
(offset = ligne du BEGIN, limit = END - BEGIN + 5, ~450 lignes, sous le cap
Read). Tu DOIS voir « <!-- BOOTSTRAP:END --> » dans ce que tu as réellement
lu ; sinon la lecture est tronquée : augmente limit / re-Read par fenêtres
jusqu'à voir le marqueur de fin AVANT le pre-flight. (Un Read naïf du fichier
entier s'arrête au milieu du bootstrap — toujours passer par les marqueurs.)

NE LIS RIEN D'AUTRE avant d'avoir lancé le pre-flight §7.1 : c'est lui qui te
dit quels fichiers sont pertinents pour ton cas. Pas de lecture spéculative
de PATTERNS.md / THREAT_MODEL.md / plans de sprint avant.

Process courant (ne pas réinventer) :
  - Mode ULTRACODE ON : exhaustivité + correction, pas coût ni vitesse.
  - Orchestration PAR WORKFLOW à chaque étape de DÉCOUVERTE/VÉRIFICATION
    (kickoff, audit gate, preflight de phase, review de phase, recherche) :
    fan-out + vérif adversariale + synthèse. Le Workflow LIT/vérifie, il ne
    code JAMAIS une phase en parallèle ; l'écriture d'une phase reste
    main-thread, séquentielle, un commit atomique par phase.
  - PAS DE SUPERVISEUR (amendement 2026-06-17) : ne crée aucun teammate
    supervisor, n'attends aucun verdict GO-*/BLOCK-*. Codex (GPT-5.6 Sol) = vérif
    croisée externe après review Workflow PASS-PENDING. Seul gate automatisé
    au commit = hook phase-precommit-lightcheck.sh.
  - Modèle agents : ID explicite claude-opus-4-8[1m], jamais l'alias « opus »,
    jamais passer le param model à Agent().

GATE DE CONFIRMATION DE LECTURE (obligatoire, AVANT toute action). Quand le
pre-flight §7.1 a tourné, restitue en <=6 lignes :
  1. Cas détecté : A / B / C / D (+ le signal §7.1 qui le prouve).
  2. Prochaine action concrète (ex. « INVOQUER nexus-audit-gate », « Workflow
     preflight Phase X », « commit chore(planning) d'abord »).
  3. Règle EXÉCUTER vs DEMANDER appliquée ici, citée du §7.1.
  4. Read-proof : le numéro de ligne du marqueur « <!-- BOOTSTRAP:END --> »
     obtenu par Grep sur le fichier vivant (preuve que tu as lu le README sur
     disque et non un prompt collé périmé — ce numéro n'est fourni nulle part
     ailleurs, ni par le hook session-start, ni par un paste).
Si tu ne peux pas remplir ces 4 points depuis ce que tu as lu, ta lecture est
partielle : retourne lire le bootstrap. Puis enchaîne sans demander
confirmation quand le cas est procéduralement déterminé.

Langue : français (réponses, docs planning, commit bodies) ; anglais (code,
identifiants, commit titles, logs).
```

### Comment lire ce README parfaitement

Ce document est la **source de vérité unique** du process. Une lecture
partielle reproduit le bug récurrent « session fraîche qui ré-invente les
règles ». La bonne lecture est **bornée et vérifiée**, pas exhaustive ligne
à ligne.

**Lecture bornée par marqueurs (drift-proof).** La région critique d'une
session fraîche est le bloc de bootstrap §7.1, délimité par
`<!-- BOOTSTRAP:BEGIN -->` et `<!-- BOOTSTRAP:END -->`. C'est le seul passage
à lire **intégralement** avant d'agir. Ne code JAMAIS un numéro de ligne en
dur (ce fichier bouge) : Grep les deux marqueurs pour obtenir la plage
courante, puis Read `offset = ligne BEGIN`, `limit = END - BEGIN + 5`. Le
bloc (~430 lignes) tient en UN Read sous le cap de 2000 lignes ; un Read naïf
du fichier entier s'arrête au milieu du bootstrap. Le reste du README se lit
**à la demande**, section par section, quand le pre-flight ou un agent t'y
renvoie.

**Étape de confirmation (anti-lecture-bâclée).** Avant la moindre action,
après le bootstrap + le pre-flight §7.1, restitue en <=6 lignes : (1) le cas
A/B/C/D et le signal qui le prouve, (2) la prochaine action concrète, (3) la
règle EXÉCUTER vs DEMANDER appliquée, (4) le **read-proof** : le numéro de
ligne du marqueur « <!-- BOOTSTRAP:END --> » obtenu par Grep sur le fichier
vivant (preuve d'avoir lu le README sur disque, non un prompt collé). Si tu ne
peux pas produire ces quatre points, la lecture est incomplète : retourne au
bootstrap. Ce n'est pas une demande de permission — quand le cas est
procéduralement déterminé, tu enchaînes sans attendre l'humain.

**Si le bootstrap paraît tronqué.** Critère objectif : tu n'as **pas vu**
`<!-- BOOTSTRAP:END -->`. N'agis pas sur une lecture partielle ; re-Read par
fenêtres jusqu'à voir le marqueur de fin. Tant que `<!-- BOOTSTRAP:END -->`
n'est pas apparu, le pre-flight ne démarre pas.

---

## 1. Vue d'ensemble

Le projet nexus-grid (SBFB) est un réseau P2P de compute LLM
distribué. L'ingénierie se fait exclusivement via Claude Code,
sur des sessions courtes (1h30 à 3h), avec une discipline de
sprint inspirée des pratiques agile mais adaptée au fait que
l'agent n'a pas de mémoire entre sessions.

Les piliers du système :

1. **Roadmap multi-sprint** — Sprint 0 à Sprint 23+, chaque
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

Sections canoniques (pattern Sprint 20 gold) :

1. **§Sources context7 + WebSearch consultées (pre-gel)** —
   traces des recherches effectuées AVANT de rédiger les D1-D5.
   URLs, versions, dates absolues, papers. Preuve factuelle que
   G9 a été respecté. Section absente = ⚠️ G1.
2. **Constat d'entrée** — quel est le tip master au début,
   quels tests passent, quels commits ont landé depuis le
   sprint précédent, quel est le verdict de l'audit gate.
   Sous-sections : §1.1 D'où on part, §1.2 Ancrage
   HARDENING_ROADMAP, §1.3 Compteurs tests entrée (tip SHA),
   §1.4 Pre-launch protocol policy (rappel).
3. **Goal en une phrase** — ce que le sprint promet de livrer.
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
4. **Phase 0 — Audit gate du sprint précédent** (DONE avant le
   kickoff lui-même à partir de Sprint 7) — résumé du verdict
   et du commit stack de gate
5. **Décisions Day 0 (D1..D5 gelées)** — les choix
   architecturaux qui vont piloter toutes les phases. Une
   fois figées, non rebattables. Format S20 gold :
   - titre court
   - « Retenu » : la décision (paragraphe détaillé + code sample
     si applicable)
   - « Rejeté » : chaque alternative avec raison factuelle du
     rejet (1-2 lignes par alternative, minimum 2 alternatives)
   - « Implications code » : fichiers/modules verrouillés
   Suivi immédiat : **§Acknowledged review findings (G1)** —
   scoring D1✅ D2✅ D3⚠️ etc. avec adjustments inline pour
   chaque ⚠️. Format :
   ```
   Scoring : D1 ✅, D2 ✅, D3 ⚠️, D4 ✅, D5 ✅.
   Rigor signal G4 satisfait (1 ⚠️ sur 5).
   D3 ⚠️ : [finding]. Decision : adjust — [correction inline].
   ```
6. **Plan Phase outline A..F** — une section courte par phase
   avec son scope, son critère d'acceptation et son commit
   cible. **Sprints pairs** (S28, S30...) : au moins une phase
   est réservée dette/refacto (§6.2.1 Règle 1). Les sprints
   post-arc sont des sprints consolidation dédiés (§6.2.1).
   Le kickoff identifie et liste les items déférés absorbés.
7. **Items carry/dette** — reclassification explicite des
   carry-overs, avec pour chaque item : classification
   (carry confirmé / scope intégré / supprimé DEPRECATED),
   nombre de reports consécutifs, rationale, conséquences.
   Items à 3 reports : plan obligatoire (§6.2.1 Règle 2).
8. **Scope cuts** — liste **exhaustive** (10-14 items) des
   choses qu'on ne fera PAS dans ce sprint et pour quel
   sprint elles sont gardées. Pattern S20 : chaque item avec
   sprint cible explicite.
9. **Traçabilité scope** — table qui mappe chaque item « What's
   NOT » du sprint précédent sur le sprint + phase où il est
   pris en charge
10. **Risk register (R1..R7)** — risques techniques avec
    colonnes Likelihood / Impact / Mitigation. Pattern S20 :
    7 risques identifiés, dont R7 qui a prédit le conflit
    Phase E (G8 DESIGN-CONFLICT). Ce registre est vérifié
    par l'audit gate.
11. **Audit gate pattern — rappel** — confirme que la Phase 0
    a été jouée et que la phase de sortie devra produire l'audit_plan
12. **Checkpoint de validation** — 5 questions pour arbitrage
    user AVANT que l'agent attaque le plan détaillé. Pattern
    S20 : une question par D-decision pour confirmer le choix.
    Pas un rubber-stamp — c'est le dernier moment pour pivoter
    sans coût.

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
4. **Phase A..F** — une section complète par phase (pattern
   S20 gold, 5 sous-sections par phase) :
   - **§X.1 Scope** — 1-3 paragraphes détaillant les livrables
   - **§X.2 Fichiers touchés** — table `| Fichier | Rôle |`
     (chemin complet + description des changements, pas juste
     une liste plate). **Pas d'estimation LOC** — cf. §6.7
   - **§X.3 Tests plan** — tests nommés individuellement avec
     scénario (pas juste un comptage "+N tests"). Pattern S20 :
     `1. test_derive_kek_roundtrip`, `2. test_aad_integrity`,
     etc. Permet de vérifier la couverture au commit
   - **§X.4 Critère d'acceptation** — commandes exactes pour
     vérifier (ce qui doit être vert avant de commiter)
   - **§X.5 Commit cible** — template body complet incluant
     le titre exact et les sections attendues du body. Force
     la discipline avant d'écrire le code
   - **Dependencies inter-phases** documentées explicitement
     en tête de section (ex: "Phase A → Phase B : B extends
     KeyStore::unlock() de A"). Pattern S20 : graphe de
     dépendances en 5 lignes avant les phases
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

Écrit en fin de sprint, juste avant la phase de sortie.
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
   Delta` pour Rust / Vitest / size-limit
6. **Surface nouvelle livrée** — liste factuelle des LOC par
   nouveau module
7. **Ce que le sprint n'a PAS livré (scope cuts respectés)**
   — reprise **exhaustive** de §8 kickoff (10-14 items) avec
   check `❌` pour chacun. Ne pas tronquer — l'auditeur
   vérifiera que TOUS les items sont listés
8. **Findings carry-over for memory (G6)** — max 5 items qui
   valent d'être persistés dans la memory externe (P0/P1 +
   décisions long-terme + gotchas surprenants). Fusion
   manuelle au kickoff S{N+1}
9. **Checkpoint de clôture** — les N conditions du plan.md
   §checkpoint, chacune cochée

### 2.4 audit_plan.md — le plan d'audit pour le sprint suivant

Écrit dans le même commit que verification.md, en phase de
sortie. Rôle : donner à la session fraîche du sprint N+1
une feuille de route d'audit indépendante et reproductible.

**C'est le doc le plus stratégique** — une session fraîche
sans historique va le jouer et produire un verdict que l'agent
livreur ne peut pas influencer.

Sections canoniques (pattern Sprint 6/7) :

1. **Mode d'emploi pour la session fraîche** — ordre de
   lecture imposé, liste des fichiers à NE PAS lire avant
   d'avoir formé une opinion, format
   du delivrable final
2. **Tracks A..I** — une section par axe d'audit. Chaque
   track contient :
   - Question centrale (« Est-ce que X fait vraiment Y ? »)
   - Méthodes concrètes (commandes à rouler, grep à lancer,
     tests à écrire)
   - Signal d'audit (qu'est-ce qui est P0, P1, P2, P3)
3. **Track G1 presence (P1 bloquant si absent)** — verifier
   que `sprint{N}_design_review.md` existe dans archive/v{X}/.
   Absent sur sprint non-trivial = **P1** (gate bypasse, cf.
   S26 gap). Present mais sans scoring = P2. Present avec
   5/5 scores = OK. Exception : kickoff contient "G1 skipped".
4. **Track HARDENING drift (P2 informatif)** — comparer
   `HARDENING_ROADMAP.md §3` ligne S{N} (items prescrits) vs
   ce que le sprint a reellement livre. Pour chaque item prescrit
   non livre, verifier :
   - scope-cut justifie dans le kickoff §7 ? → OK
   - blocker externe documente ? → OK
   - ni l'un ni l'autre → **P2** (drift non justifie)
   Cette track est informative (P2), pas bloquante (P1). Son
   objectif est la visibilite, pas la punition. Si le drift
   cumule sur 3+ sprints sans justification, l'auditeur remonte
   le signal pour revalider le HARDENING_ROADMAP lui-meme.
5. **Verdict global attendu** — trois scénarios :
   - PASS : 0 P0, 0 P1 → sprint N+1 Phase A démarre direct
   - CONDITIONAL PASS : 1-3 P1 fixables → N+1 Phase A bloqué
     tant que les `fix(sprint{N}): ...` ne sont pas landed
   - FAIL : ≥ 1 P0 ou ≥ 3 P1 → re-conception partielle
6. **Out of scope pour l'audit** — liste explicite de ce que
   l'auditeur ne doit PAS rebattre (les D1..D5 gelées, les
   scope cuts, les choix de pin de dep)
7. **Livrable final attendu** — format exact de
   `audit_findings.md` + critère de clôture

### 2.5 audit_findings.md — le rapport d'audit indépendant

**Pas écrit par l'agent du sprint N.** Produit par la session
fraîche qui démarre le sprint N+1, en Phase 0 de ce sprint
suivant. Joue le `sprint{N}_audit_plan.md` et écrit son
verdict.

Sections canoniques (pattern Sprint 6 audit_findings) :

1. **Auditeur** — id de session, duree observee
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
   non couvert et pourquoi

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

**Data S22-S24** : seul gate à valeur causale prouvée (2
DESIGN-CONFLICT S20/S21, P1 C-1 S23 échappé quand audit
conditionnel actif). Review coverage : S22 6/6, S23 1/6 (gap non
détecté — fix §4.4 step 5), S24 3/3 (restauré).

### 3.2 Phases A..E — contenu du sprint

Les vraies livraisons de code. Une phase = un commit
atomique feat. Pattern Sprint 6/7 :

- Phase A — skeleton / fondations du sprint
- Phase B, C, D — itérations successives ajoutant des
  capacités
- Phase E — polish, intégration, tests Playwright / Vitest
- (optionnel, une phase supplémentaire si scope complexe, ex
  Sprint 6 où D a été split en D+E)

### 3.3 Phase de sortie — trois livrables obligatoires

Dans le même commit `docs(sprint{N}): verification + audit
plan for Sprint N+1` (la clôture docs-contrat peut aussi être
une phase dédiée juste avant, façon S79 Phase I) :

1. `sprint{N}_verification.md` — self-report fail-fast
2. `sprint{N}_audit_plan.md` — plan que Sprint N+1 Phase 0
   jouera + update de `docs/shell/PATTERNS.md` +
   `docs/rust/PATTERNS.md` avec les nouveaux patterns et
   tech debt items
3. **Clôture docs-contrat (§6.12)** — GUIDE + `llms.txt`
   (+ `WIRING_SPEC.md` si concerné) indexent chaque primitive
   de **frontière NEUVE** du sprint (test-acteur §6.12 : wire,
   API — y compris loopback lue par un runtime distinct —,
   contrat d'app, prompt-kind, knowledge). Si le sprint n'a
   créé aucune frontière : consigner `N-A-no-new-frontier`
   dans `verification.md`, jamais l'omettre en silence.

**Sans ces trois livrables, le sprint ne peut pas être fermé.**

**Lecture obligatoire avant d'écrire ces livrables** : §2.3
(9 sections canoniques verification.md), §2.4 (6 sections
canoniques audit_plan.md), et §4.4 (routing des findings des
phase reviews dans l'audit_plan). Ne pas dériver ces fichiers
depuis le plan ou un sprint précédent — lire la spec d'abord.

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

**Budget de phases : ouvert, jamais plafonné.** Un sprint a exactement
autant de phases que son objectif l'exige — le nombre de phases est une
*sortie* du travail, jamais une *cible* en entrée. Les phases d'impl
sont nommées `A, B, C, … Z, AA, AB, …` (bijectif base-26, illimité ;
suffixe chiffre optionnel `A1`/`AA2` pour une re-coupe / hotfix de
sous-phase). **L'ancien plafond « 4-7 phases A-G » est SUPPRIMÉ** : la
phase de wrap-up est la dernière lettre réellement utilisée, et
`verification.md` énumère chaque phase livrée. Ajouter une phase `AA`
« durcissement acceptance » quand le cœur remplit A–Z est le résultat
*attendu*, jamais un dépassement. `Phase 0` reste réservé au gate
d'audit (commit `chore(planning)` / `fix(sprint{N-1})`, déclaré dans
`sprint{N}_audit_plan.md` — convention, non parsé comme phase d'impl par
le validateur). Le regex de phase est `Phase [A-Z]+[0-9]?` partout
(`agentctl.py`, hooks lightcheck/auditor) — strict superset de l'ancien
`[A-Z][0-9]?`, donc tout sprint A-G reste byte-valide.

**Definition of done d'un sprint.** Un sprint est DONE quand (a) chaque
objectif roadmap a une phase atterrie, (b) chaque carry d'audit routé
est CLOSED ou re-routé avec rationale, (c) le **gate de testabilité
par-sprint** ci-dessous est VERT (ou `RIG-ABSENT` machine-lisible pour
le seul tier multi-machine), ET (d) la **clôture docs-contrat** (§6.12,
§3.3 livrable 3) est livrée : GUIDE + `llms.txt` à jour pour chaque
primitive de frontière neuve du sprint, ou `N-A-no-new-frontier`
consigné dans `verification.md`. Tant que (c) ou (d) n'est pas
satisfait, le sprint n'est pas fermable, peu importe le nombre de
phases déjà livrées.

**Gate de testabilité par-sprint** (évalué à la phase de wrap-up,
consigné dans `sprint{N}_verification.md` sous `## Acceptance`). Trois
tiers, chacun avec un vocabulaire de verdict FERMÉ et machine-lisible —
**fini le `DIFFERE-materiel` tapé en prose** :

| Tier | Preuve | Verdicts autorisés | Au wrap-up |
|---|---|---|---|
| **T0** Unit/Integration | `cargo nextest` + Vitest (§7.4) | counts, tous verts | déjà enforced |
| **T1** E2E hermétique (solo) | `npm run test:e2e` (Playwright, vrai daemon `--web-root`, sans Ollama) | `GREEN` / `RED` / `N-A-no-frontend-change` | **BLOQUANT** : `RED` bloque le wrap-up. Toujours exécutable (le binaire build en CI) → jamais légitimement skippable. CI relance l'hermétique à chaque push |
| **T2** Acceptance live | spec compute flagship (solo, Ollama) **et/ou** `scripts/acceptance/b3_live_pc_vps.sh` (multi-machine, **artefact JSON**) | `PASS` / `BLOCK{diagnosis}` / `RIG-ABSENT` / `N-A-no-cross-machine-feature` | l'artefact JSON DOIT exister et parser ; `PASS` ou `RIG-ABSENT` laisse fermer ; `BLOCK` exige un `diagnosis` non vide + route un carry P1 |

**Invariant d'honnêteté (mécanique).** `RIG-ABSENT` n'est émis QUE par
le préflight du harness (échec SSH / Ollama absent / binaire absent /
`project_doc_id` ≠ `PROJECT_ID`) et écrit comme champ JSON `status`.
Aucun verdict T2 ne se tape à la main. Une feature **cross-machine** ne
peut être DONE que si (a) le test d'intégration de convergence (deux
nœuds iroh se découvrant via le vrai chemin discovery, propagation d'une
entrée `task:` **incrémentale** écrite *après* subscribe) est VERT — le
maillon que les tests in-process co-localisés ne couvrent pas — ET (b)
`b3` émet `status: PASS` sur le rig. Sinon la feature est **PROVISIONAL**
dans `verification.md` (jamais DONE) + carry P1 forcé vers l'audit gate
suivant ; le test de convergence est le **prérequis dur** (probable
`Phase A` du sprint cross-machine) avant qu'aucune phase ne puisse
revendiquer une feature cross-machine.

**Enforcement (mécanique).** Ce gate n'est plus seulement documenté, il
est appliqué par trois backstops. (1) L'**audit gate** du sprint N+1 joue
une **Track J — Testabilité** (`prompts/agent/audit-gate-checks.md`) qui
vérifie que N a créé un spec T1 `web/e2e/*.spec.ts`, son statut CI, et
l'artefact JSON T2 — absence quand `web/` est touché, ou prose `DIFFERE-*`
substituée à un verdict, = **P1** (bloque le PASS, force un `fix(sprintN)`
avant la Phase A suivante). (2) Le **kickoff** (`nexus-sprint-kickoff`,
invariant #16) exige que le plan NOMME le spec T1 + l'acceptance T2 dès
l'ouverture, et que le wrap-up écrive la Track J dans l'audit_plan S{N+1}.
(3) Le hook **lightcheck** (Check 10) émet un WARN au commit de wrap-up si
`verification.md` ne porte pas de verdict T1/T2 machine-lisible. Un sprint
sans surface frontend (`N-A-no-frontend-change`) ou non cross-machine
(`N-A-no-cross-machine-feature`) passe les trois — seuls l'oubli et la
prose-au-lieu-de-JSON sont punis.

Chaque phase respecte une discipline stricte :

### 4.1 Un commit atomique par phase

Pattern commit title (format exact) :

```
type(scope): Sprint N Phase X — titre court
```

Types valides : `feat` (nouvelle fonctionnalite), `fix` (correction
bug), `docs` (documentation seule), `chore` (planning, deps, CI).
Le scope est le module principal touche (ex : `feed`, `trust`,
`factory`, `planning`). Compound scopes separes par `+` si la phase
touche 2+ modules (ex : `feed+trust`).

```
feat(scope): Sprint N Phase X — titre court

Body structuré (template — 9 sections obligatoires) :

## Contexte
[1-3 paragraphes : rationale, threat model, research grounding]

## Fichiers
| Fichier | Rôle |
|---------|------|
| crates/nexus-foo/src/bar.rs | [description changement] |
[grouper par Rust / Web / Tests]

## Delta tests
| Suite | Avant | Après | Delta |
|-------|-------|-------|-------|
| Rust workspace | 538 | 566 | +28 |
[+ décomposition per-module : "+15 keystore::tests, +8 integration, +5 unlock"]

## Verification §7.4
[CI manifest complet, chaque suite avec résultat]

## Scope cuts respectés (kickoff §8)
[TOUS les items du kickoff §8, exhaustif — pas de troncature]

## G8 traceability
- Preflight : [SHA `chore(planning)` ou "staged, no prior SHA"]
  verdict [EXECUTE plan-as-is / PLAN-ADAPT / SCOPE-CUT-CONSISTENT]
  Quand le preflight est bundled dans le commit feat (pas de
  `chore(planning)` separe), utiliser "staged, no prior SHA" + nom
  du fichier preflight.md. La tracabilite temporelle est alors
  prouvee par le HEAD reference dans le preflight lui-meme.
- Review : [SHA commit phase lui-même ou "staged"] verdict final
  [PASS] apres reconciliation Codex ([N] P0, [N] P1, [N] P2, [N] P3).
  `PASS-PENDING` est autorise uniquement dans le review.md pre-Codex ;
  il est interdit comme verdict final committable.
[chaîne explicite — permet à l'audit gate S+1 de retracer le process]

## Pre-launch protocol
[*_VERSION unchanged, wire format preservé]

## Codex verification
- Rapport : sprint{N}_phase_{X}_codex_review.md
- Livrables : {N} audités, {N} confirmés, {N} gaps corrigés
[si gaps corrigés : description 1-ligne par gap + re-verification]

## Carry closure / Unblock
[graphe de dépendances inter-sprint explicite]

Co-Authored-By: Claude <model> <noreply@anthropic.com>
```

**Enforcement format `##`** (amendement S65, constat P2 body-format
Phases A-C) : les 9 sections doivent utiliser des headings markdown
`## Nom` exactement comme dans le template ci-dessus. Le contenu en
prose informelle sans `##` — même s'il couvre les mêmes informations —
n'est PAS conforme et sera bloqué par le hook `phase-precommit-
lightcheck.sh` Check 9. Raisons : (1) parsing automatisé par l'audit
gate S{N+1} (grep `^## ` dans le commit body pour extraire chaque
section), (2) cohérence inter-phases (Phase D ne devrait pas être
"meilleure" que Phase A dans le même sprint), (3) l'agent exécuteur
doit copier le template de `.claude/templates/commit_body_phase.txt`
plutôt que d'improviser la structure.

**Gold standard courant** : copier
`.claude/templates/commit_body_phase.txt` et produire **9/9 sections**,
incluant `## Codex verification`. Les anciennes references `8/8`
datent d'avant la section Codex et sont obsoletes.

**Template** : `.claude/templates/commit_body_phase.txt` contient le
squelette complet prêt à copier. L'agent exécuteur DOIT le lire avant
d'écrire le premier commit body de chaque sprint.

**Deletions de code source** : la suppression de fichiers source
(`.rs`, `.ts`, `.tsx`, `.py`, etc.) doit être dans le commit `feat`
de la phase qui la motive ou dans un commit `chore(cleanup)` dédié
— jamais dans un `chore(planning)`. Un commit `chore(planning)` ne
touche que `.planning/`, docs workflow, et agents/skills/hooks.

**Docs techniques dans feat, pas chore** (P2-I-1) : les fichiers
de documentation technique qui accompagnent un livrable de phase
(`PATTERNS.md`, `FACTORY_GATES.md`, `THREAT_MODEL.md`, etc.)
appartiennent au commit `feat`/`docs` de la phase correspondante.
Ils ne doivent pas être relégués dans un `chore(planning)` séparé,
car leur contenu est indissociable du code livré. Seuls les
fichiers purement workflow (`.planning/`, kickoff, plan, review)
restent dans `chore(planning)`.

**Phase commit gate** : tout commit dont le titre contient
`Sprint N Phase X` avec un type valide (`feat`, `fix`, `docs`,
`test`, `refactor`) est un phase commit gate — il est soumis à la
chaîne complète preflight → review PASS-PENDING → Codex →
reconciliation → review PASS → commit body 9 sections.

**Docs-only sans exemption** : une phase purement documentation
n'exempte ni la review Claude (PASS-PENDING puis PASS après
Codex), ni la verification croisée Codex (§4.5.6 zero exemption),
ni le body 9 sections obligatoires. Seules les suites lourdes
(cargo nextest, Vitest, release build) peuvent être exemptées avec
justification écrite dans la review et le commit body.

Si une phase a besoin d'un fix post-commit (pattern Sprint 2
`de9589d` / `ed2ea76` ou Sprint 6 gate `05c96c4..8fbe07b`),
le fix vit dans un commit séparé
`fix(sprint{N}): description` — jamais d'amend.

### 4.1.1 Mécanique d'écriture du message commit (Windows Git Bash)

**Règle impérative** : pour tout commit dont le body dépasse ~30
lignes, écrire le message dans un fichier puis `git commit -F`.
**Ne PAS utiliser `git commit -m "$(cat <<'EOF' ... EOF)"`** pour les
bodies riches.

Cause : heredoc `<<'EOF'` protège l'expansion shell mais Windows Git
Bash échoue régulièrement sur :
- Apostrophes françaises répétées (« l'approche », « l'économie »,
  « l'auditor ») qui déstabilisent le parser malgré `<<'EOF'`
- Backticks markdown nombreux (`` `fichier.py` ``) dans un body
  multi-sections
- Buffering CRLF qui masque la terminaison `EOF` quand la ligne
  fermante n'est pas strict-LF
- Interaction `"$(...)"` + heredoc qui force bash à compter les
  quotes jusqu'à `)"` fermant le command substitution

Le résultat observé (Sprint 22 Phase E, S20 Phase E pivot) :
`unexpected EOF while looking for matching '` — plusieurs sessions
tentent heredoc, échouent, puis basculent sur file-based.

Pattern correct :

```bash
# 1. Écrire le body dans un fichier (Write tool en agent context,
#    ou éditeur en main). Chemin standard : .git/COMMIT_EDITMSG_<label>.txt
#    pour que le fichier vive sous le répertoire git (cleanup auto
#    possible) et soit ignoré du tracking.

cat > .git/COMMIT_EDITMSG_PHASE_E.txt <<'CANARY_END'
feat(sprint22): Phase E — titre court

Body multi-sections riche...
CANARY_END

# 2. Commit par fichier
git commit -F .git/COMMIT_EDITMSG_PHASE_E.txt

# 3. (optionnel) nettoyage post-commit
# le fichier est ignoré par git (sous .git/) ; il peut rester pour
# audit, ou être supprimé avec `rm .git/COMMIT_EDITMSG_PHASE_E.txt`
```

Pour les bodies courts (<= 30 lignes, 1 section), `git commit -m
"title$(echo -e '\n\n body')"` ou `-m "title" -m "body"` reste
acceptable — le seuil 30 lignes est empirique, les commits de phase
atomique dépassent systématiquement.

Anti-pattern à éviter : re-essayer heredoc sur erreur « unexpected
EOF » en espérant que l'erreur était transitoire. Elle ne l'est pas
— basculer immédiatement sur file-based.

### 4.2 Discipline de staging

Staging explicite (jamais `git add -A`) :

```bash
git add \
  crates/nexus-foo/src/bar.rs \
  web/src/pages/Foo.tsx \
  ...
```

Protège contre l'inclusion accidentelle de fichiers de
secrets, binaires, caches temporaires.

### 4.3 Verification obligatoire avant commit

**Deux modes** : itération pendant la phase (scope-réduit, rapide) et
verification finale avant le commit (exhaustif). `cargo test
--workspace --locked` sur le projet entier coûte 15-20 minutes sur
une machine dev et n'a pas besoin de tourner entre deux edits —
scope au crate modifié pendant l'itération et ne lance le full
workspace qu'une seule fois, juste avant le commit.

**Itération pendant la phase** (après chaque edit significatif) :

```bash
# Rust — nextest sur le crate touché, tests unit+integration
cargo nextest run -p <crate-touche> --locked

# Frontend — vitest watch mode ou un seul fichier
cd web && npx vitest run src/components/__tests__/<file>.test.tsx
```

**Verification finale avant le commit de phase** (complet) :

```bash
# Rust — nextest workspace + doctests (nextest ne gère pas les doctests)
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc

cd web
npx tsc --noEmit -p tsconfig.app.json
npm run lint
npm run test:unit
npm run build
npm run size
bash scripts/scan-en-strings.sh
```

Prerequis : `cargo install cargo-nextest --locked` (config partagée
dans [`.config/nextest.toml`](../../.config/nextest.toml), profil
`ci` utilisé par [`.github/workflows/rust-ci.yml`](../../.github/workflows/rust-ci.yml)).
Fallback `cargo test --workspace --locked` accepté si nextest
indisponible, mais l'isolation process-per-test de nextest détecte
des flakes que `cargo test` masque (panic qui contamine d'autres
tests du même binaire).

Tout rouge bloque le commit. Aucune exception « je commit et
je fix après » — le fix doit être dans le même commit ou
déclenche un nouveau cycle.

**Verification croisee Codex (depuis S65, cf. §4.5).** Apres que
toutes les suites sont vertes et que la review Claude a produit un
verdict `PASS-PENDING`, lancer la verification Codex GPT-5.6 Sol pour
TOUTES les phases sans exception (§4.5.6). Sequence complete avant
commit phase :

1. Suites §7.4 vertes (Rust + Frontend)
2. Review Workflow (fan-out dimensions + verification adversariale,
   §4.5.7) — verdict `PASS-PENDING` possible (fallback skill
   nexus-phase-review si l'opt-in Workflow est absent)
3. Codex verification croisee (§4.5) — GAPs corriges ou documentes
4. Reconciliation Claude post-Codex — review.md promu a `PASS` final
   avec section `## Codex reconciliation`
5. Commit atomique — l'unique gate automatise est le hook mecanique
   `phase-precommit-lightcheck.sh` (staging coherence, body 9
   sections, artefact Codex brut). Plus de consultation superviseur
   GO/BLOCK.

L'ordre preflight → code/tests → review `PASS-PENDING` → Codex →
reconciliation/promote review `PASS` → commit est strict.
Ne jamais committer avec un review.md encore en `PASS-PENDING`.
Ne jamais lancer Codex avant la review.

### 4.4 Phase de sortie — parse phase reviews et route les P2/P3 au audit_plan

**Fix P2-S21-4 Sprint 22.** Avant Sprint 22, la phase de sortie
écrivait `sprint{N}_verification.md` + `sprint{N}_audit_plan.md`
mais ne parsait pas systématiquement chaque
`sprint{N}_phase_*_review.md` produit par l'agent
`nexus-phase-auditor` pré-commit. Conséquence : les findings
P2/P3 documentés par l'auditeur de phase pouvaient rester
orphelins (jamais ré-injectés dans l'audit gate du sprint
suivant).

**Règle désormais imperative** :

À chaque écriture de `sprint{N}_audit_plan.md` en phase de sortie,
la session doit :

1. **Enumérer** les fichiers `.planning/active/sprint{N}_phase_
   *_review.md` présents.
2. **Parser** chaque section `## Findings` (ou équivalent
   `## Issues found`) et extraire les findings `P[0-3]-*`.
3. **Router** chaque finding dans le Track correspondant de
   `sprint{N}_audit_plan.md` :
   - P2-{Phase}-* → Track A/B/C/D/E selon la Phase source
   - P3-{Phase}-* → Track correspondant sous-section advisory
   - Findings transverses (LOC estimations, convention hygiene)
     → Meta-track dédié ou carry summary
4. **Documenter** les findings résolus inline pendant la phase
   (fix retrospective, iteration 2 pattern Sprint 22 Phase D)
   dans la section Track comme `[closed inline]` avec le commit
   SHA de résolution.

5. **Vérifier présence exhaustive** : pour chaque phase A..F
   ayant un commit `feat(sprint{N}): Phase X`, un fichier
   `sprint{N}_phase_{X}_review.md` doit exister dans
   `.planning/active/` (ou archive/ si déjà migré). Inscrire le
   ratio dans Track F de `sprint{N}_audit_plan.md` :
   `- [ ] Phase review files present: {N_reviews}/{N_phases}`
   **Ratio < N/N = P2** (Data S23 : 1/6 reviews produits, audit
   gate non-détecté — gap découvert par inspection manuelle
   post-facto).

**Garde-fou** : un `sprint{N}_audit_plan.md` sans référence à
au moins un finding par phase ayant un review.md = **CONCERN**
(probable parsing oublié). Exception acceptable : review.md
avec verdict PASS + 0 finding explicite.

**Exemple Sprint 22** : les 5 phase reviews A/B/C/D/E ont
produit 14 findings (5 P2 + 9 P3). `sprint22_audit_plan.md`
les route dans Tracks A-E (sous-section A-1/A-2/A-3/A-4/A-5
etc.) + Meta-track LOC estimations (transverse P3-S22A-1 +
P3-B-1 + P2-E-2 + Phase D déviation LOC = 4 occurrences du
même pattern à fermer S23 chore planning).

### 4.5 Verification Process — Workflow ultracode + Codex GPT-5.6 Sol

Depuis Sprint 65, chaque phase de chaque sprint est verifiee par un
processus a deux couches : une **orchestration Workflow ultracode**
(fan-out d'agents Claude ultra-profonds, 1M tokens chacun, +
verification adversariale + synthese) pour le preflight et la review,
puis une verification croisee independante par Codex CLI (GPT-5.6 Sol,
`gpt-5.6-sol` reasoning `max` — bascule 5.5→5.6 Sol 2026-07-10, le
tier flagship du plan Pro au reasoning maximal ; a exige un upgrade CLI
codex >=0.144.1, cf. §4.5.2).

Le superviseur process (`nexus-process-supervisor`) et les
consultations de gate GO/BLOCK (`G-SPAWN`/`G-PREFLIGHT`/`G-REVIEW`/
`G-CODEX`/`G-COMMIT`/`G-POST`) sont **supprimes** (amendement
2026-06-17). Ils sont remplaces par : (1) l'orchestration Workflow
elle-meme, qui porte le verdict du preflight et de la review ;
(2) Codex, la verification croisee externe ; (3) le hook mecanique
`phase-precommit-lightcheck.sh`, l'unique gate automatise avant
commit. Aucun agent ne repond plus GO/BLOCK a chaque etape — le
main thread enchaine les couches et le hook bloque mecaniquement
au commit si un invariant est viole.

Ce processus ajoute une couche de verification que ni l'auto-
attestation (verification.md) ni l'audit gate intra-sprint (Phase 0)
ne couvrent : une review par des agents en fan-out puis par un modele
fondamentalement different (Codex), sans partage de contexte avec
l'executeur, sur le code reel commite.

**Contrainte de composition Workflow (a respecter).** Les Workflows
lances en arriere-plan et le Monitor notifient en **fin de tour** :
ils composent proprement avec un **working tree propre**. En
milieu de phase, lancer l'orchestration Workflow AVANT que l'arbre
de travail ne soit committe-sale d'une maniere qui bloquerait, OU
s'appuyer sur des **agents paralleles en avant-plan** (fan-out
synchrone, un seul tour maintenu vivant) quand un tour doit rester
actif jusqu'au verdict. Le preflight et la review etant des etapes
de LECTURE/verification (pas d'ecriture concurrente des fichiers
partages), le fan-out avant-plan est le mode par defaut sur.

#### 4.5.1 Cycle de vie d'une phase

```
Plan section Phase X lue
  |
  v
Workflow preflight (fan-out agents 1M, claude-opus-4-8[1m])
  |  5 scans factuels S1a-S4 en parallele + synthese -> verdict
  v
Code ecrit par Claude (execution phase standard, main-thread)
  |
  v
Workflow phase review (fan-out dimensions + verif adversariale)
  |  6+ dimensions, synthese -> verdict PASS-PENDING/CONCERN/FAIL
  v
Codex verification (codex exec, prompt structure, findings)
  |  Review croisee independante GPT-5.6 Sol
  v
Claude reconciliation (lit Codex, corrige/documente, promeut review.md a PASS)
  |
  v
Claude correction loop (si Codex trouve des issues)
  |  Fix + re-run suites + review Workflow + Codex (boucle complete)
  v
Hook phase-precommit-lightcheck.sh (gate mecanique : staging,
  body 9 sections, artefact Codex brut)
  |
  v
Commit atomique feat(scope): Sprint N Phase X
  |
  v
Memory + chore planning a jour (G6)
```

Le preflight et la review Workflow restent les gates primaires
(G8 + G4) ; le hook lightcheck est l'unique gate automatise au
commit. Codex est une verification supplementaire — il ne
remplace ni le preflight, ni la review, ni l'audit gate. Il n'y a
plus de superviseur ni de consultation GO/BLOCK entre les etapes.

#### 4.5.2 Lancer Codex depuis Claude Code — pattern valide

**Pattern qui fonctionne** (teste et valide S65 Gate 0) :

```powershell
# 1. Ecrire le prompt dans un fichier texte
#    (Write tool en contexte agent, ou editeur)
#    Chemin standard : .git/CODEX_SPRINT{N}_PHASE_{X}.txt
#    (inclure le sprint evite d'ecraser le prompt d'un autre sprint)
#    Helper canonique :
#    python scripts/agent/agentctl.py codex-prompt-path --sprint {N} --phase {X}

# 2. Pipe via stdin vers codex exec
Get-Content ".git/CODEX_SPRINT{N}_PHASE_{X}.txt" -Raw | codex exec `
  -m gpt-5.6-sol -c model_reasoning_effort=max `
  --dangerously-bypass-approvals-and-sandbox `
  -o ".planning/active/sprint{N}_phase_{X}_codex_review.md"
```

**Parametres obligatoires :**

| Parametre | Role |
|-----------|------|
| `-m gpt-5.6-sol` | Modele de la review croisee = GPT-5.6 Sol (tier flagship Pro). SCOPE la review SBFB sans toucher le default global `~/.codex/config.toml` des autres projets. Bascule 5.5→5.6 Sol 2026-07-10. Requiert CLI codex >=0.144.1 (sinon `400 requires a newer version of Codex`). Slug exact : `sol` (PAS `solar`/`sol-pro`/`codex-sol`, tous refuses en compte ChatGPT). |
| `-c model_reasoning_effort=max` | Epingle l'effort `max` explicitement (independant du global) — la review croisee vise la profondeur maximale (directive PO 2026-07-10). Sol supporte aussi `ultra` au-dessus si une phase le justifie ; `xhigh` en-dessous si le budget temps l'exige. |
| `--dangerously-bypass-approvals-and-sandbox` | Execution sans approbation interactive (equivalent de `--yolo`) |
| `-o fichier.md` | Ecrit l'output dans un fichier lisible par Claude apres execution |

**Anti-patterns testes et echoues — NE PAS reproduire :**

| Anti-pattern | Symptome | Pourquoi |
|---|---|---|
| `-m o3` / `-m gpt-5.6-solar` / `-m gpt-5.6-sol-pro` | Erreur 400 "not supported when using Codex with a ChatGPT account" | Compte ChatGPT, pas API — le slug flagship valide est `gpt-5.6-sol` (Sol, pas `solar` ni `sol-pro`) |
| Here-string PowerShell direct | Parsing errors sur apostrophes francaises | PowerShell interprete les guillemets internes |
| Prompt inline en argument | Codex attend stdin quand pas d'argument | Le prompt doit passer par stdin ou fichier |
| Prompt sans `-o` | Output pas recuperable par Claude | Toujours `-o fichier.md` pour lecture post-exec |
| Prompt trop court (<10 lignes) | Review superficielle, faux positifs | Le prompt doit lister explicitement chaque livrable |

**Authenticite artefact Codex.** Le fichier
`sprint{N}_phase_{X}_codex_review.md` doit rester la sortie directe de
`codex exec -o`. Lightcheck Check 7 bloque les commits Phase
`feat/fix/docs/test/refactor` si le fichier est absent, vide, non stage,
modifie hors staging, ressemble a un resume Claude (`Auditeur: Claude`,
`agent independant`, `# Codex Review`), ne contient aucun verdict par
livrable (`CONFIRME` / `PARTIEL` / `GAP`) ou ne contient aucune evidence
fichier:ligne. Si l'artefact contient des `PARTIEL` ou des `GAP`, le body
`## Codex verification` doit les reporter ; `0 PARTIEL` ou `0 GAP` est
bloque si l'artefact dit le contraire.

#### 4.5.3 Template de prompt Codex — verification phase

Ce template est a ecrire dans `.git/CODEX_SPRINT{N}_PHASE_{X}.txt` avant
de lancer `codex exec`. Adapter les placeholders `{...}` a la
phase en cours. Template complet dans
`.claude/templates/codex_phase_review.txt`.
Le chemin doit etre obtenu si possible via
`python scripts/agent/agentctl.py codex-prompt-path --sprint {N} --phase {X}`.

```
Tu es un auditeur independant. Tu verifies le code source du
projet nexus-grid (SBFB) apres une phase de sprint.

Sprint : {N}
Phase : {X} — {titre court}

## Livrables attendus de cette phase

{Copier-coller les livrables depuis le plan.md section Phase X}

## Ta mission

Pour CHAQUE livrable :
1. Cherche dans le code source les fichiers concernes.
2. Verifie que le livrable est REELLEMENT implemente.
3. Cite le fichier et les numeros de ligne exacts.
4. Conclus : CONFIRME (evidence) ou GAP (ce qui manque).

Reponse en francais.
```

#### 4.5.4 Template de prompt Codex — verification preflight G8

Template dans `.claude/templates/codex_preflight_review.txt`.
Verifie que les 5 scans S1a-S4 du preflight ont ete executes
correctement.

#### 4.5.5 Cycle de correction post-Codex

Quand Codex produit des findings :

1. **Claude lit le rapport Codex** (Read tool sur le fichier -o)
2. **Triage** : GAP confirme (Claude corrige) / faux positif
   (documente dans commit body)
3. **Correction** : Claude corrige chaque GAP confirme
4. **Re-run systematique** : apres toute correction, relancer
   suites + review Claude (`PASS-PENDING`) + Codex (boucle complete)
5. **Reconciliation** : quand Codex est clean ou seulement P2/P3
   documentes, mettre a jour `sprint{N}_phase_{X}_review.md` :
   `## Verdict: PASS`, ajouter `## Codex reconciliation`, referencer
   le fichier codex_review.md et les GAPs corriges/documentes.
6. **Tracabilite** : commit body inclut section `## Codex verification`

#### 4.5.6 Codex obligatoire — zero exemption

Codex verification croisee est **obligatoire pour TOUTES les
phases sans exception** : code, docs, dette, wrap-up, hotfix.
Aucune exemption LOC, aucune exemption contenu. La seule facon
de skip est un "PO dit skip codex" explicite dans la conversation.

Si Codex n'est pas disponible, le commit reste bloque en
`PASS-PENDING`. Documenter l'indisponibilite dans la session et
demander un arbitrage PO explicite. Sans "PO dit skip codex", il
n'existe pas de review final `PASS` ni de commit autorise.

#### 4.5.7 Orchestration Workflow ultracode — preflight + review

Le preflight et la review de phase sont des etapes de LECTURE /
verification : elles s'orchestrent en **Workflow ultracode**
(fan-out d'agents independants, 1M tokens chacun, claude-opus-4-8[1m],
+ verification adversariale + synthese). Gain mesure : ~3x sur
phases > 10 fichiers. Le Workflow ne code JAMAIS une phase en
parallele (editions concurrentes des fichiers partages = conflits +
commit atomique casse) — l'ecriture reste main-thread sequentielle.

**Preflight Workflow (G8)** — fan-out des 5 scans factuels, la
synthese emet le verdict EXECUTE / PLAN-ADAPT / SCOPE-CUT-CONSISTENT
/ DESIGN-CONFLICT et ecrit `sprint{N}_phase_{X}_preflight.md` :

```
Agent S1a (OSS prior art)    --+
Agent S1b (deps/libs)        --+
Agent S2  (historiques)      --+---> Synthese Workflow -> verdict
Agent S3  (threat model)     --+      -> preflight.md
Agent S4  (wire format)      --+
```

**Review Workflow (G4)** — fan-out des dimensions de review +
verification adversariale, la synthese emet le verdict
PASS-PENDING / CONCERN / FAIL et ecrit `sprint{N}_phase_{X}_review.md` :

```
Agent R1 (diff ligne-a-ligne + branch coverage)  --+
Agent R2 (scope cuts semantiques + research)      --+
Agent R3 (securite deep + threat model)           --+--> Synthese
Agent R4 (livrables vs plan + patterns)           --+    Workflow
Agent Rv (verification adversariale des findings) --+    -> review.md
```

**Composition arriere-plan vs avant-plan** (cf. §4.5 contrainte).
Les Workflows en arriere-plan et le Monitor notifient en fin de
tour : ils composent avec un working tree propre. En milieu de
phase (arbre potentiellement sale), preferer le **fan-out
avant-plan** (synchrone, un seul tour maintenu vivant jusqu'au
verdict) ; reserver l'arriere-plan aux moments ou l'arbre est
propre. Fallback si l'opt-in Workflow est absent : plusieurs
appels `Agent` en parallele dans un meme tour, ou les skills
`nexus-phase-preflight` / `nexus-phase-review` en sequentiel.

---

## 5. Memory system externe

Fichiers persistés hors repo, lus par chaque session fraîche
au démarrage (index dans `MEMORY.md`, fichiers dédiés par topic) :

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

4. **Après chaque commit `feat(sprintN): Phase X`**, l'agent met à
   jour `nexus_grid_pivot.md` §Tip (description textuelle : phases
   livrées, compteurs tests réels, carries P2+, prochaine phase) ET
   la ligne correspondante dans `MEMORY.md`. Séquence post-commit :
   (1) commit feat → (2) update memory → (3) résumé utilisateur.

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

**Gate de testabilité par-sprint (non négociable).** Un sprint n'est pas
fermable tant que le gate de §4 n'est pas VERT : **T1** E2E Playwright
hermétique BLOQUANT au wrap-up (+ CI à chaque push), **T2** acceptance via
artefact JSON machine-lisible (`PASS` / `BLOCK{diagnosis}` / `RIG-ABSENT`).
Un `DIFFERE-materiel` en prose ne satisfait plus aucun tier ; une feature
cross-machine sans test de convergence vert + `b3 status: PASS` reste
**PROVISIONAL** avec carry P1 forcé vers l'audit gate suivant. Détail et
table des tiers : §4 (« Gate de testabilité par-sprint »).

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
   le scoring report.
4. **Le planner reste owner mais doit acknowledge chaque ⚠️ et ❌
   explicite** dans le kickoff §4 (paragraphe "Acknowledged review
   findings"). Pas de veto reviewer, pas de stalemate.

**Avantage vs adversary-agent** : pas de "trouve 3 raisons de
rejeter" (genere du bikeshedding), juste un signal de qualite des
sources. Le reviewer ne propose pas de solution → pas de bataille
d'ego entre planner et reviewer.

**Checklist crypto/spec** `[DETER]` (ajouté S19, incident S19 P3-B2
Tor PoW/Equi-X — `sprint19_audit_findings.md`) :
- [ ] D-choice crypto/spec cite ≥1 alternative concurrente <6 mois
- [ ] Source datée <2 ans ou explicitement revalidée
- [ ] Reviewer ⚠️ si alternative absente

**Checklist Rust-first** `[DETER]` (ajouté S21, incident D2 PII
nexus-pii-rs gap — `sprint21_kickoff.md §Sources`) :
- [ ] D-choice runtime cite ≥1 alternative Rust-native production
- [ ] Gap factuel documenté si alternative Rust rejetée
- [ ] Reviewer ⚠️ si gap non documenté
- Exemptions : CI tooling, frontend UX, docs, tests fixtures

**Quand skipper G1** : sprint pure-docs (S17), hotfix (cas D §7),
phase trivial refactor sans decision Day-0.

**Enforcement mécanique** : le hook `phase-precommit-lightcheck.sh`
Check 5 bloque tout commit `feat(sprint{N}): Phase A` si
`sprint{N}_design_review.md` n'existe ni dans `.planning/active/`
ni dans `.planning/archive/v*/`. Exemption : kickoff contient
"G1 skipped". Ajouté 2026-04-25 après constat que S26 a skippé G1
sans que l'audit gate ne le détecte (rule doc-based sans enforcement).

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

#### 6.2.1 Carry-overs, escalade et dette (G7)

Anti-pattern observe Sprint 18→26 : les items autonomes < 500 LOC
(journald, oslog, 8 events wire, RAG sanitization) sont reportes
sprint apres sprint, jamais prioritaires face aux features, puis
« reclassifies long-term » ce qui les enterre. Le cap 2/2 et la
reclassification a 3 sprints ne resolvent pas le probleme — ils
le cachent.

**3 regles (amendement 2026-04-24)** :

##### Regle 1 — Phase dette 1 sprint sur 2 + sprint consolidation post-arc

Les **sprints pairs** (S28, S30, S32...) reservent **au moins une
phase** (typiquement Phase B) exclusivement dediee aux items
differes, refacto ciblee, et tests manquants. Cette phase n'est pas
negociable et ne peut pas etre convertie en feature.

**Sprint consolidation post-arc** (amendement 2026-05-22) : apres
chaque arc de la roadmap, le sprint suivant est un sprint de
consolidation dedie. Les phases sont elargies (2-3 phases a
~1200-1500 LOC au lieu de 4-5 a ~600 LOC) pour reduire le ratio
process/code. Le sprint ne livre aucune feature nouvelle — il
stabilise l'arc precedent. Regle anti-derive : chaque item doit
referencer soit un bug pilote, soit un carry existant, soit un
test E2E manquant identifie. Zero item speculatif. Gate de sortie :
"l'arc precedent est defendable pour un utilisateur externe".

Rationale : l'analyse S66-S68 montre que le ratio code/process est
de 8-29% (70-90% du delta sprint est du markdown process). Le cout
process par phase est quasi-fixe (~1000-1500 lignes d'artefacts).
Reduire le nombre de phases et augmenter le scope par phase ameliore
mecaniquement le ratio. Le sprint consolidation evite de propager
la dette au niveau reseau quand l'arc suivant ajoute du P2P.

##### Regle 2 — Escalade automatique a 3 reports

Un item differe **3 sprints consecutifs** devient **obligatoire**
au sprint suivant — il doit etre integre dans le plan, pas dans
le cap carry-overs. Pas de reclassification « long-term » pour les
items < 500 LOC : ils restent carry actif jusqu'a livraison ou
suppression explicite (`docs/DEPRECATED.md` avec rationale).

**Exemptions** (avec justification renouvelee a chaque kickoff) :

- **Blocker externe** : dependance amont instable (arti pre-1.0),
  legal review en cours. La justification doit etre factuelle et
  re-evaluee (pas « meme raison que S-1 » copie-colle).
- **Dependance sequentielle interne** : l'item depend d'un output
  de phase pas encore livre. La dependance doit etre nommee
  (« attend Phase C output X du sprint courant »).

Sans exemption valide, un item a 3 reports entre dans le plan
comme phase obligatoire, pas comme carry.

**Items > 500 LOC avec blocker externe** : seuls ceux-ci peuvent
etre reclassifies dans `docs/release/ROADMAP_COMMITMENTS.md` avec
les 7 champs (ID, Title, Origine, Condition de declenchement,
Owner, Runbook pointer, Derniere revue). Condition : le blocker
est verifiable et externe au projet (API upstream, legal, budget).

##### Regle 3 — Check ROADMAP_COMMITMENTS au kickoff

Le kickoff §4 "Phase 0" inclut un check obligatoire des conditions
de declenchement de `ROADMAP_COMMITMENTS.md`. Pour chaque item LT :

```bash
# Evaluer chaque condition de declenchement
grep -A 5 "Condition de declenchement" docs/release/ROADMAP_COMMITMENTS.md
```

Si une condition est remplie (tag v1.0 pose, iroh > 0.97 publie,
Gini > 0.70 observe), l'item redevient carry actif dans le kickoff.
Pas de re-activation silencieuse — le kickoff documente le
declenchement avec evidence.

##### Mecanique carry-over (inchange)

- **La phase de sortie genere `sprint{N+1}_carry_summary.md`** (pas
  optionnel) listant les carry-overs avec :
  - ID + description (1 ligne)
  - Source : phase qui a reporte + commit SHA
  - Severite : P1 (Gate-blocker) / P2 (debt) / P3 (cosmetic)
  - Nombre de reports consecutifs (compteur incremente)
  - Owner : `<github-handle>` ou `S{N+1}` par defaut
- **Kickoff S{N+1} doit re-confirmer** chaque carry via une ligne
  explicite dans §6 "Items carry/dette" : `[x] C-1 carry confirme
  pour S{N+1} Phase A` ou `[deferred] C-1 differe S{N+2} (report
  N/3, justification : ...)`.
- **Items a 3 reports** : le kickoff NE PEUT PAS les re-confirmer
  comme carry. Soit ils entrent dans le plan comme phase, soit ils
  sont supprimes via `docs/DEPRECATED.md`.

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
- **Pas d'estimation LOC en amont dans le kickoff/plan.** Raisons :
  (1) la taille finale dépend de la solution trouvée après
  recherche, elle n'est pas connaissable au plan ; (2) une
  estimation amont devient un plafond psychologique — l'agent
  tronque la solution la plus poussée pour rentrer dans le
  budget ; (3) la "vitesse de delivery" se mesure aux phases
  livrées avec tests verts, pas aux LOC produites. La seule LOC
  qui compte est la LOC **rétrospective** (mesure de gap, ex.
  §6.2 "gap réel mesuré ~300 LOC"). **Exception** : le
  HARDENING_ROADMAP §3 fournit des bornes indicatives par item
  pour le séquençage multi-sprint (~500, ~300, etc.) — ces
  chiffres sont des **bornes de scoping** admises pour évaluer si
  un sprint est dimensionné par objectif fonctionnel (pas par
  budget LOC), pas des métriques de succès ni des plafonds de
  phase. Un plan ne les reprend pas en "budget" par phase.
- **Exemption rétrospective** : les plans écrits avant cette règle
  (Sprint ≤ 63) qui contiennent des estimations LOC dans §X.2
  sont exemptés. Aucun rework de plans archivés. La règle s'applique
  à tout nouveau plan §X.2 à partir du Sprint 64.

Preuve empirique : S14 Keyoxide, S17 VALIDATED_BLUEPRINT, S18
supply-chain → research-first, zero rework majeur.
À l'opposé : S7 singleton band-aid, S18 D-1 wire manquant → code-
first, rework commits. Corrélation directe research/doc amont ↔
réduction debug/rework aval.

### 6.8 Fraîcheur des artefacts long-life — triggers événementiels (G2)

`[DETECT]` Artefacts long-life (`HARDENING_ROADMAP.md`, `PATTERNS.md`,
`VALIDATED_BLUEPRINT.md`, memory) portent un frontmatter
`triggers_revalidate` (liste d'events : release upstream, CVE, S+2).
Au session-start, grep les triggers et re-scanner si un event s'est
réalisé depuis `last_validated`. Événementiel, pas timer.

```bash
grep -lE 'triggers_revalidate' docs/security/*.md docs/rust/PATTERNS.md
```

Origine : S19, HARDENING_ROADMAP S17 hérité sans audit → D2 PoW
Hashcash drift. Rationale complet : `fe0a8fd`.

### 6.9 Constantes nommées — pas de magic numbers pour un domaine énuméré

Tout littéral qui encode une **valeur de domaine énumérée** (un niveau,
un statut, un mode, un seuil métier réutilisé) doit être une **constante
nommée unique**, réutilisée partout — jamais un nombre nu répété dans le
code source.

- **Foyer canonique unique** : la constante vit dans le module qui
  possède le type (ex. `web/src/api/consent.ts` pour `CONSENT_LEVEL`),
  et **reflète l'énum source de vérité** quand elle existe côté Rust
  (`ConsentLevel` = `OwnProjects/OpenSource/Whitelist/All`). Pas de
  re-déclaration locale du même mapping dans deux fichiers.
- **Réutilisée PARTOUT** : comparaisons (`level === CONSENT_LEVEL.ALL`,
  pas `=== 4`), clés d'objet/`Record` (`[CONSENT_LEVEL.WHITELIST]:`, pas
  `3:`), énumérations exhaustives (`CONSENT_LEVELS_ASCENDING`, pas
  `[1,2,3,4]`), et tout sous-ensemble sémantique (`PUBLIC_SHARING_LEVELS`).
  Un sous-ensemble nommé exprime l'**intention** (« quels niveaux ouvrent
  le partage public ») mieux qu'un `=== 2 || === 4`.
- **Portée** : s'applique au **code de production/source**. Les fixtures
  de test peuvent rester littérales (input explicite, le nom du test dit
  déjà « niveau 4 ») — mais une comparaison/branche de test gagne aussi à
  être nommée si elle encode la sémantique du domaine.
- **Vérifié à la review** : la review Workflow de phase (fan-out
  dimensions, §4.5.7 ; fallback skill `nexus-phase-review`) signale
  tout magic number de domaine comme finding (P2 si récurrent, P3 si
  isolé). Grep type :
  `grep -nE '\b<champ>\s*[=!<>]==?\s*[0-9]' web/src/**/*.{ts,tsx}` hors
  `status === 404` / longueurs / versions de schéma.

Origine : S76 Phase A — les niveaux consent (1..4) comparés en littéraux
(`level === 2 || level === 4`, clés `Record` numériques, `[1,2,3,4]`)
masquaient l'intention ; remplacés par `CONSENT_LEVEL` + dérivés. Feedback
PO : « avoir des constantes nommées réutilisées partout ». Cf. memory
[[feedback-named-constants]].

---

### 6.9 Phase pre-flight factual evolution check (G8)

Anti-pattern observé : Sprint 20 plan §8 Phase E demandait un *warrant
canary auto-publish scheduler* alors que Sprint 18 E2 (`04c9621`)
avait **explicitement rejeté** ce pattern pour raisons threat-model
documentées dans le commit body. Le drift a été détecté *par hasard*
au moment du grep duplicate-check, pas par procédure. Si le grep
n'avait pas été fait, le code aurait reproduit exactement la
vulnérabilité que S18 fermait.

**Diagnostic** : entre le sprint kickoff (où G1 Design Review et G2
triggers_revalidate filtrent les drifts) et le code-time (où le hook
lightcheck catch les erreurs de staging), il y a un trou. Le
plan §Phase X peut avoir été écrit 3-5 sprints avant son exécution,
sur une compréhension partielle de l'historique. Personne ne re-grep
systématiquement les décisions intermédiaires entre plan-time et
code-time.

**G8 = gate procédural pre-implementation phase.** Avant la PREMIERE
LIGNE DE CODE de chaque phase de chaque sprint, l'agent exécute 5
scans factuels indépendants (S1a + S1b + S2 + S3 + S4). Verdict en
4 niveaux (EXECUTE / PLAN-ADAPT / SCOPE-CUT-CONSISTENT / DESIGN-
CONFLICT). Procédure systématique, pas opinion.

**Allègement process (amendement 2026-05-22)** : l'analyse S65-S68
montre que G8 preflight a produit 0 finding actionnable sur 20
phases (20/20 EXECUTE). Le preflight reste obligatoire mais le
format condensé (template Step 7 "condense") est le défaut pour les
phases standard. Le format complet (S1a OSS research profonde) est
réservé aux phases qui introduisent un nouveau composant de sécurité,
un nouveau wire format, ou une nouvelle dépendance cryptographique.
Le preflight s'orchestre en Workflow ultracode (fan-out des 5 scans
+ synthèse, §4.5.7) ; fallback skill `nexus-phase-preflight`. Il n'y
a plus de superviseur process ni de consultation de gate — le hook
mécanique `phase-precommit-lightcheck.sh` est l'unique gate
automatisé au commit.

#### Quand

Avant la 1ère ligne de code d'une phase (entre validation `gsd:plan-
phase` ou lecture du plan §Phase X et le 1er `Edit`/`Write` outil).
Pour CHAQUE phase de CHAQUE sprint, sans exception (sauf hotfix
hors-sprint cas D).

**Exemption phases post-plan** (amendement S54, constat S53 Phases
E/F/G) : une phase insérée ad hoc pendant le sprint (découverte
runtime, fix d'un P1 trouvé en smoke test) peut être exécutée sans
preflight G8 si et seulement si (a) la phase est une réponse directe
à un bug ou blocage découvert dans une phase précédente du même
sprint, (b) elle ne touche ni wire format ni composant de sécurité
nouveau, et (c) l'absence de preflight est documentée comme P2
process dans la review de la phase wrap-up. L'audit gate vérifie
cette justification.

#### Les 5 scans factuels

| Scan | Source | Output attendu |
|---|---|---|
| **S1a — OSS prior art (G10)** | `WebSearch` "comment les projets OSS matures résolvent le problème de cette phase" + `mcp__context7` sur libs/frameworks trouvés. Projets de référence par domaine : compute verification → BOINC/Folding@Home/Golem/Truebit, LLM safety → NeMo/Guardrails AI/openai-agents, P2P → libp2p/IPFS, crypto → age/Keyoxide/FROST | Findings : `APPROACH-NAIVE` (plan naïf vs SOTA OSS, bloquant), `APPROACH-ALIGNED`, `LIB-EXISTS` (lib prête, bloquant), `APPROACH-NOVEL` (justifié contexte P2P) |
| **S1b — Deps/libs versions** | `mcp__context7__query-docs` sur libs/specs touchées + `WebSearch` CVE/audit/RFC bump publiés depuis `last_validated` | Findings type *"lib X v Y.Z bump major"*, *"CVE-2026-XXXX critical"* |
| **S2 — Décisions historiques traversées** | `git log --all --grep="DEVIATION\|rejected\|scope-cut\|deliberate" -- <files-touchés-phase>` + grep `.planning/archive/v*/` + memory `feedback_*.md` | Liste de *"S{N-k} `<sha>` a explicitement rejeté/dévié pour raison Z"* |
| **S3 — Threat model coverage** | `docs/security/THREAT_MODEL.md` + `HARDENING_ROADMAP.md §3` + audit findings | Matrix *"primitive proposée → threats T0-T5 couverts vs non-couverts"* |
| **S4 — Wire format / pre-launch invariants** | grep `*_VERSION` + `canonical.rs` + memory `nexus_grid_pivot.md §Pre-launch` | Liste invariants (wire format, Day 0 figées, scope cuts) |

Les 5 scans sont **non-substituables**. S1a sans S1b = on a le bon
design mais sur une lib obsolète. S1b sans S1a = on a la bonne lib
mais le mauvais design (S24 Phase D : BLAKE3 à jour mais hash binaire
sur output stochastique = inopérant). S2 sans S3 = cohérent
historiquement mais gap threat model. S3 sans S4 = durci mais wire
cassé. S4 sans S1 = invariants préservés sur approche obsolète.

**Parallelisation via agents ultra-profonds (depuis S65).** Les 5
scans sont independants en entree. Lancer les 5 en parallele via
des agents Claude independants (Opus 4.6, 1M tokens chacun).
L'orchestrateur agrege les resultats et emit le verdict. Gain
mesure : ~3x sur phases > 10 fichiers cibles. Detail : §4.5.7.

**S1a est le scan le plus important.** Il challenge le *design* du
plan, pas juste les versions. Un plan écrit au kickoff reflète la
compréhension du moment. La recherche OSS pre-phase corrige les
angles morts avant d'écrire du code. Le plan est un point de départ,
pas un contrat — la recherche le corrige si nécessaire.

#### Efficacité mesurée (S22-S24, 17 preflights)

14/17 EXECUTE, 3/17 SCOPE-CUT-CONSISTENT, 0 DESIGN-CONFLICT,
0 PLAN-ADAPT (S1a ajouté post-S24 Phase D).

- **S1b+S2 portent 100% des findings réels (S22-S24)** : S22-B
  GLiNER ONNX mismatch, S22-D gpu/ module pré-existant, S23-E
  CVE-2025-69277 pynacl.
- **S1a (ajouté S24)** : 0 run historique (nouveau). Aurait détecté
  APPROACH-NAIVE sur S24 Phase D (hash binaire sur output LLM
  stochastique — BOINC/Truebit montrent que ça ne marche pas).
- **S3+S4 sont des gate checks** : 0 finding en 17 runs → fast-path
  grep only.
- **Consolidation** : S3+S4 fast-path, S1a obligatoire (nouveau
  volet le plus important — challenge le design pas juste les deps).
  Les 5 scans restent obligatoires.

#### Décision tree (verdict en 4 niveaux)

```
Si S1a+S1b+S2+S3+S4 = clean :
  → EXECUTE plan-as-is
  → emit ".planning/active/sprint{N}_phase_{X}_preflight.md" condensé
  → procéder code phase normalement

Si S1a finding bloquant (APPROACH-NAIVE ou LIB-EXISTS) :
  → PLAN-ADAPT
  → emit preflight.md avec §Plan adaptation (evidence OSS + approche
    corrigée + fichiers/tests impactés)
  → procéder code phase avec l'approche corrigée (PAS le plan original)
  → commit body documente la déviation vs plan §Phase X
  → PAS d'arrêt, PAS d'arbitrage user — la recherche est l'arbitre
  → le plan.md reste inchangé (snapshot kickoff), la déviation est
    tracée dans preflight.md + commit body

Si finding non-bloquant uniquement (S1b minor, S2 réversé, etc.) :
  → SCOPE-CUT-CONSISTENT
  → emit preflight.md avec finding + carry-over S+1
  → procéder code phase normalement

Si finding bloquant S1b/S2/S3/S4 (pas S1a) :
  → DESIGN-CONFLICT
  → STOP code écriture
  → emit ".planning/active/sprint{N}_phase_{X}_pivot_proposal.md"
    avec sections obligatoires :
      - Evidence factuelle (commit refs, CVE, RFC, context7, URLs)
      - 3 options minimum : [A=scope-cut, B=adapt minimal, C=deep]
      - Coût/bénéfice par option
      - Préservation invariants (wire format, Day 0, threat model)
      - Recommandation default
  → user arbitre l'option
  → si pivot accepté → commit chore(planning) AVANT feat
```

**PLAN-ADAPT vs DESIGN-CONFLICT** : PLAN-ADAPT corrige l'approche
technique (le *comment*) — le plan avait tort sur la méthode, la
recherche OSS montre la bonne. DESIGN-CONFLICT touche les
contraintes structurelles (Day 0 figées, wire format, threat model)
— ça demande un arbitrage humain car ce sont des décisions de
gouvernance, pas de technique.

#### Garde-fous (G8 ≠ blanc-seing pour scope creep)

Le pivot deep-evolution doit être maîtrisé sinon il devient un vecteur
de divergence chronique. 7 garde-fous obligatoires :

1. **Pivot evidence-based, pas opinion** : `pivot_proposal.md` REQUIRE
   au moins 1 source factuelle externe vérifiable (commit ref dans
   l'historique, CVE ID NVD-trackable, RFC section, context7 query
   timestamp + lib name, audit report DOI/URL). Opinion seule
   ("je pense que X est mieux") = invalid → reject.

2. **Pivot ne rebat pas Day 0 figées sans escalation** : si le pivot
   touche D1..D5 du sprint courant ou décisions actées dans memory
   `nexus_grid_pivot.md §Decisions actees` ou `nexus_grid_pivot.md
   §Pre-launch protocol policy` → escalation user obligatoire,
   jamais pivot auto. Le pivot peut PROPOSER une remise en question
   Day 0 mais ne peut JAMAIS la trancher.

3. **Pivot ne casse pas pre-launch wire format** : si pivot bumperait
   `*_VERSION` avant tag v1.0 → invalid sauf motivé par CVE bloquant
   sur la primitive crypto sous-jacente. Sinon redéfinir le canonical
   v1 (pattern actuel `nexus_grid_pivot.md §Pre-launch protocol`).

4. **Pivot test budget cap** : si test delta pivot > 2.5x plan
   original → split en 2 phases (E + E.bis dans même sprint), ou
   carry next sprint. Sinon le sprint déraille (ratio 8 tests
   prévus → 25 tests réels = signal scope creep masqué).

5. **Pivot reste DANS le thème sprint** : sprint S20 = "security
   hardening" → pivot dans cette zone OK (federated multi-canary,
   FROST primitive, etc.). Pivot vers "UI redesign" ou "perf
   optimization" = NON. Reste fidèle au sprint kickoff §1 thème.

6. **Pivot doit clore une gap claire, pas créer complexité YAGNI** :
   si "primitive scaffolding pour S+5" sans aucun consumer dans
   S{N}-S{N+4} → reject (You Aren't Gonna Need It). Si scaffolding
   pour consumer réel sprint dédié dans roadmap explicite (ex :
   FROST K-of-N en E.2 prepare consumer = community future réelle
   listée HARDENING_ROADMAP §3 ligne S25-30) → OK.

7. **Pivot audité retrospectivement + traçabilité G8 obligatoire** :
   `nexus-phase-auditor` reçoit dimension supplémentaire "G8
   traceability" pour CHAQUE phase auditée (pas seulement celles qui
   ont pivot). Avant d'autoriser le commit feat phase, l'auditor
   vérifie la présence de `sprint{N}_phase_{X}_preflight.md` OU
   `sprint{N}_phase_{X}_pivot_proposal.md` dans `.planning/active/`.
   Absence = P1 bloquant "G8 gate bypass" (la phase a été codée
   sans scan pre-implementation, drift plan-vers-code non-détecté).
   Exception : Cas D hotfix hors-sprint uniquement.
   Si verdict DESIGN-CONFLICT : l'auditor vérifie aussi que le
   plan §Phase X reflète le pivot via chore(planning) antérieur
   au commit feat — divergence silencieuse plan-vs-code = P1
   "pivot silencieux". Rationale : sans cette dimension, un agent
   overzealous pouvait skipper G8 et passer l'audit quand même.
   G8 produit un artefact file-based précisément pour rendre le
   skip détectable post-hoc. Si pattern de pivot répétitif sur
   2+ phases consécutives → signal méta-issue plan-phase quality
   (le plan d'origine était-il vraiment basé sur SOTA fresh ?).

#### Articulation avec G1-G7 existants

G8 n'est pas un substitut de G1/G2 — il les complète à un moment
différent du cycle. Vue d'ensemble :

Tags : `[DETECT]` = mécanique observable, drop si 0 findings en 4
sprints. `[DETER]` = principe dissuasif, drop si 3× violations MALGRÉ
la règle. `[STRUCT]` = structure du cycle, jamais drop.

| Gate | Tag | Quand | Quoi | Output |
|---|---|---|---|---|
| G1 (§6.1.1) | `[DETER]` | Kickoff, après draft D1..D5 | Design Review Board scoring report | `sprint{N}_design_review.md` |
| G2 (§6.8) | `[DETECT]` | Session-start, event upstream | Re-validation triggers_revalidate docs long-life | `last_validated` updated |
| G3 (§2.1) | `[STRUCT]` | Kickoff §2 | Goal SMART → verification.md fail-fast | `sprint{N}_kickoff.md §2` |
| G4 (§3 + auditor) | `[DETECT]` | Phase review + audit gate | Rigor signal : 0 P0/P1 + ≥1 P2+ pour PASS | Verdict PASS/CONCERN/FAIL |
| ~~G5~~ | — | ~~Supprimé S24~~ | ~~Working tree audit~~ | ~~Hook lightcheck couvre~~ |
| G6 (§5.1.1) | `[STRUCT]` | Post-commit feat + phase de sortie | Memory update §Tip + carry-over | `nexus_grid_pivot.md` updated |
| G7 (§6.2.1) | `[STRUCT]` | Phase de sortie carry generation | Escalade 3 reports + phase dette sprints pairs | `sprint{N}_carry_summary.md` |
| G8 (§6.9) | `[DETECT]` | Pre-implementation phase | 5 scans factuels OSS prior art + SOTA deps + history + threat + wire | `phase_{X}_preflight.md` |
| G9 (§6.10) | `[DETER]` | Avant proposition D-choice | Factual research gate on D-decisions | §Research consulté dans kickoff |

G8 comble le trou entre G2 (kickoff-time) et le commit (code-time).
G1 protège contre les drifts au design ; G8 protège
contre les drifts au plan-vers-code translation.

#### Anti-patterns (pivot mal exécuté)

- **Pivot opportuniste** : "tant qu'on touche ce fichier on en
  profite pour refactor X". Reject — G8 declenche sur DESIGN-CONFLICT
  factuel, pas sur opportunité d'éditeur.
- **Pivot pour pivoter** : pas de finding S1-S4 mais quelqu'un trouve
  l'archi "pas assez deep". Reject — G8 require evidence factuelle.
- **Pivot silencieux** : on adapte le code sans `pivot_proposal.md`
  ni update plan. Reject — divergence silencieuse plan/code casse
  l'audit gate.
- **Pivot qui dilue verification.md** : pivot ajoute 5 features mais
  ne met pas à jour la fail-fast checklist. Reject — checklist
  sprint-end doit refléter ce qui a été livré.
- **Pivot répétitif** : 3 pivots consécutifs sur 3 phases = le plan
  n'était pas basé sur SOTA fresh. Reject suite — signal méta vers
  re-faire `gsd:plan-phase` complet, pas accumuler des pivots.
- **PLAN-ADAPT sans evidence OSS concrete** : S1a conclut
  APPROACH-NAIVE mais cite 0 projet OSS de reference avec URL ou
  query context7. Invalid — PLAN-ADAPT require >= 1 projet OSS
  nomme avec source verifiable. Sinon c'est une opinion, pas une
  adaptation evidence-based.
- **PLAN-ADAPT qui touche Day-0 figees** : si l'approche corrigee
  modifie une D1..D5 du sprint courant, ce n'est pas PLAN-ADAPT
  (correction technique) mais DESIGN-CONFLICT (gouvernance).
  Escalation user obligatoire, pas adaptation inline.
- **PLAN-ADAPT repete** : 2+ PLAN-ADAPT consecutifs dans le meme
  sprint = le plan n'etait pas base sur SOTA au kickoff. Signal
  meta → re-faire `gsd:plan-phase` complet sur research fresh,
  pas accumuler des adaptations incrementales.

#### Mise en œuvre

Implémentation procédurale via skill `.claude/skills/nexus-phase-
preflight/SKILL.md` (cf. §7.1 bootstrap Cas B). Le skill scripte
les 5 scans + emit le bon document selon verdict. Aucun pivot
manuel sans passer par cette gate.

---

### 6.10 G9 — Factual-research-gate on D-decisions

Anti-pattern observe Sprint 21 kickoff 2026-04-18 : l'orchestrateur
a propose d'entree un draft D2 PII « alt2 Hybrid Rust-first iframe »
sans avoir fait la research factuelle sur l'ecosysteme Rust
inference ML (tract wasm32-browser support, GLiNER opset coverage,
gline-rs production readiness, ort-wasm existence, candle-onnx ops
supportees). L'user a du corriger explicite « analyse Rust-first
SBFB avant » pour declencher la recherche. Le §6.7 research-first
documente l'ordre recherche→design→code, mais il n'est pas enforced
en amont de la **proposition** du draft D-choice lui-meme. Resultat :
le planner peut poser un D-draft plausible mais non-factuel, et
la research arrive apres coup en correction plutot qu'en input.

G1 (§6.1.1) couvre « ≥ 1 alternative concurrente < 6 mois » pour
les sources **crypto/spec standardisee**, et (extension 2026-04-18)
« custom Rust stack alternatives » pour les choix architecturaux
Rust-first. Mais aucune des deux ne garantit qu'une research
**factuelle primaire** a ete effectuee AVANT que le draft soit
propose. G9 comble ce trou procedural.

**Pattern correct** : avant de **proposer** un draft D-choice dans
le kickoff §4, l'orchestrateur DOIT avoir effectue une research G2
factuelle documentee. Le draft §Retenu cite explicitement cette
research dans un bullet `§Research consulte` (ou section dediee
kickoff §Sources) :

- Sources primaires : `mcp__context7__query-docs` sur les libs
  envisagees + `WebSearch` CVE/audit/benchmarks publics recents.
- Versions + dates absolues (pas « recent » ou « latest » — ex:
  « tract 0.22.1 publie 2026-02-23, wasm32-unknown-unknown support
  non documente officiellement, seul wasm32-wasi demontre via
  examples wasmtime »).
- Benchmarks publics si existent (latency, memory, accuracy sur
  tache comparable). Si benchmarks absents : dire explicitement
  « pas de bench public tiers 2025-2026 sur ce combo, adoption
  basee sur README + audit date + production users cites ».
- Liste des alternatives explicitement comparees avec raison
  factuelle rejet (pas opinion).

**Visibilite G1** : un draft D-choice qui n'a pas de `§Research
consulte` list ou avec sources vagues (pas de version + date) = ⚠️
automatique par le scoring report G1, independamment de la date
des sources citees individuellement (une source recente mal
structuree reste un angle mort).

**Symetrie avec G8** : G8 (§6.9) est un gate pre-phase code ;
G9 est un gate pre-D-decision Day-0. Meme principe — factual
evidence avant proposition, pas en correction post-facto. G1
reste le reviewer qui verifie la qualite des sources ; G9 impose
qu'elles existent avant que le reviewer soit lance.

**Exemple concret Sprint 21 D2 PII SDK** : le draft initial
« alt2 Hybrid Rust-first iframe » citait une preference
architecturale (Option G Rust-first) mais pas de research sur
l'etat reel de l'ecosysteme Rust inference ML en 2026-04. La
research factuelle (context7 tract + WebSearch gline-rs + ort-
web + candle-onnx ops support) a revele : tract teste opset 9-18
vs GLiNER opset 19, wasm32-unknown-unknown tract non documente
officiellement, gline-rs v1.0.1 (Rust GLiNER mainstream 2026-01)
a explicitement choisi `ort` pas tract, ort-web est `onnxruntime-
web` deguise (experimental), candle-onnx manque op Attention.
Ces faits ont converti le draft alt2 en « Option 7 JS iframe +
Presidio coord » dimensionnee factuellement. Sans G9 enforced,
la decision serait partie sur un feeling « Rust-first c'est plus
propre » sans verifier que la stack Rust visee supportait
reellement le cas d'usage.

**Quand skipper** : D-choice trivial qui ne cree pas de nouvelle
dep (ex: « utilise `governor` deja pinne workspace pour rate-
limit » — crate deja connu, aucune research G2 requise). Sprint
pure-docs (ex: S17). Hotfix hors-sprint cas D. Phase trivial
refactor sans decision Day-0. Pour tout le reste — adoption d'une
nouvelle crate, changement de stack, choix d'architecture
inference/network/storage/crypto — G9 s'applique.

Rationale : les ecosystemes evoluent vite. tract a ajoute wasm32
entre 0.19 et 0.20 ; gline-rs est passe production 2026-Q1 ;
ort-web a stabilise son API en 2025-12 mais reste experimental ;
burn-onnx est active-development 0.21. Un D-draft ecrit sur
memoire training 2024 ou intuition sans research produit des
decisions decorrelees de la realite stack 2026. G9 impose la
verification factuelle comme prerequis a la proposition, pas
comme correction post-hoc.

---

### 6.11 Archive research outputs pattern

`[DETER]` Tout research output agent (Explore, general-purpose, skill
preflight) **> 2000 mots** → Write immédiat dans `.planning/research/
S{N}_research_{topic}.md` avec frontmatter (sprint, topic, date, agent)
+ prompt brut + rapport brut + décision downstream (1-3 lignes). Skip
si < 2000 mots, confirmatoire, ou déjà dans un preflight G8.

Origine : S21, 4 rapports ~9000 mots perdus dans transcript (`71de0ec`).
But : reproductibilité audit, source pivot G8, drift detection.

---

### 6.12 Cadence docs-contrat — étiquette générée par phase, GUIDE en clôture, provenance vers le passé (canon S79 Phase B)

`[DETER]` Toute **primitive de frontière** (lue par un acteur qui n'est PAS le
code : un autre nœud = wire, un client externe = API, une app réseau =
contrat/CSP, un autre LLM = prompt-kind/knowledge) porte un **contrat
source-ancré, drift-gaté**. Un helper purement interne n'est PAS une frontière
(le code + les tests suffisent). Règle de cadence (doctrine
`.planning/research/doctrine_contrat_pour_llm.md`) :

1. **Étiquette générée (schéma drift-gaté) → PAR PHASE**, dans le commit de la
   primitive. Gratuite (le schéma est généré), in-pourrissable (drift → build
   rouge). Ex. `schema_for!` snapshots, parité `BRIDGE_METHOD_ALLOWLIST`
   Rust↔TS, Zod `.strict()`, `MANIFEST.json` registre de hash par fichier des
   knowledge packs (blake3 par couche, le MANIFEST s'exclut du hash-set).
2. **GUIDE + `llms.txt` (synthèse) → UNE phase de clôture** (l'image complète
   n'est figeable qu'à la fin ; miroir S77 Phase N). Ni « une phase de doc par
   phase », ni « tout à la fin ».

   **Porteurs de la cadence (amendement 2026-07-02, root-cause S80)** — sans
   propriétaire, l'obligation flotte et personne ne la produit :
   - le **kickoff PLANIFIE** la phase de clôture quand ≥1 phase du plan touche
     une frontière (invariant #17 de `nexus-sprint-kickoff`) ;
   - le **wrap-up la LIVRE** (Definition-of-Done (d) §4 + §3.3 livrable 3) ;
   - l'**audit gate du sprint suivant la VÉRIFIE** (Track K standing,
     `prompts/agent/audit-gate-checks.md`) — frontière neuve non indexée = P1.

   **Zone grise TRANCHÉE (2026-07-02)** : une API loopback consommée par un
   runtime DISTINCT (ex. le front Operator React qui lit les routes `/api/*`
   du serveur Rust ; le contrat SSE `StreamChunk` lu par `streamChunk.ts`)
   **EST une frontière §6.12**. Le juge est le **test-acteur** (« qui LIT
   cette primitive ? un acteur qui n'est pas le code Rust lui-même »), JAMAIS
   le test « 0 wire bump / pas propagé entre nœuds » — cette conflation a
   auto-certifié à tort les 3 frontières S80 (auth cookie, /api/git/diff,
   /api/gates) pendant 9 phases.
3. **Arête de provenance in-code (rang-1)** : un commentaire `// Sprint N Phase X
   · …` pointe UNIQUEMENT vers du **passé immuable**, JAMAIS une promesse future
   (le motif « phase/sprint + verbe futur » ADJACENT). Anti STALE-PHASE-K
   (incident réel S77). Gaté par `scripts/check-frontier-contracts.sh` (BLOQUANT
   CI, scope `crates/` + `web/src/` ; docs/ exclus car ils décrivent l'anti-pattern ;
   formes non-adjacentes / parenthétiques attrapées par la review, pas le gate).

**Gate générique** `scripts/check-frontier-contracts.sh` (câblé CI 3 surfaces :
`ci.yml`, `ci-linux.yml`, `verify.sh`) : (1) anti-promesse source-ref ; (2)
couverture-étiquette sur le registre opt-in `// FRONTIER: <name>
domain=DOMAIN_X_V1 version=X_FORMAT_VERSION` (un type annoté DOIT avoir un schéma
généré ou une exemption `// FRONTIER-NO-SCHEMA:`) ; (3) non-régression
`BLOB_SERVE_CSP`. Le registre est **incrémental** : un type non-annoté n'est pas
une violation (les 22 des 25 familles `DOMAIN_*_V1` sans schéma généré sont un
carry routé vers l'audit-plan S80, créé à la clôture du sprint).

Truth-Stack canonique de la couche GUIDE :
`repo files > .planning/active/ > commits > prompts > chat` (+ « Not evidenced »
hors rangs 1-4). Détail pattern : `docs/rust/PATTERNS.md` §P70 ; doctrine
portable : `docs/agent/AGENT_SYSTEM.md`.

---

## 7. Prompt générique de bootstrap session fraîche (v3)

Ce bloc (§7.1, délimité par `<!-- BOOTSTRAP:BEGIN -->` /
`<!-- BOOTSTRAP:END -->`) est la **source de vérité vivante** que l'agent
**lit sur disque** à chaque démarrage — le hook `SessionStart` (matcher `"*"`,
toutes les sources) lui en impose la lecture (cf. §0). Il n'est **plus à
coller en routine** : un copier-coller fige un snapshot qui peut **drifter**
alors que le disque, lui, reste à jour. Ne le colle qu'en **secours**, si
aucune directive `[session-start]` n'apparaît (hook désactivé, autre client).
Il ne suppose **pas** de connaître l'état actuel — l'agent détermine seul dans
quel cas il est, en commençant par un **bloc pre-flight** d'un seul copier-
coller, puis en routant vers la procédure du cas détecté.

### 7.1 Le prompt à coller

<!-- BOOTSTRAP:BEGIN (v3) — prompt canonique à coller ; lire jusqu'à BOOTSTRAP:END -->
```
Tu démarres une session sur nexus-grid (SBFB). Ne lis RIEN tant
que tu n'as pas exécuté le pre-flight ci-dessous — il te dit
quels fichiers sont vraiment pertinents pour ton cas.

# === Principe d'autonomie (à appliquer pendant toute la session) ===

Le process est documenté. Quand le cas est procéduralement déterminé,
EXÉCUTE sans demander. Demander = friction inutile + signal que tu
n'as pas lu §6 conventions.

EXÉCUTER directement (ne pas demander) :
  - working tree montre docs planning/skills modifiés hors phase →
    commit chore(planning|skill) AVANT phase
  - plan §Phase X explicite + audit-gate précédent PASS + G8
    verdict EXECUTE OU PLAN-ADAPT OU SCOPE-CUT-CONSISTENT → enchaîner
    Phase X (si PLAN-ADAPT : code suit l'approche corrigée, pas le
    plan — mais PLAN-ADAPT require evidence OSS concrete, ne peut
    PAS toucher Day-0 figées, et 2+ consécutifs = signal méta)
  - fichiers accidentels (cache, build) + pattern .gitignore évident
    → ajouter pattern dans le commit chore
  - cas A audit gate, P0/P1 trouvés → écrire fix(sprint{N-1}): ...

DEMANDER (STOP) seulement si :
  - fichier untracked ambigu (ex: cc.json, doc hors-scope)
  - Décision Day-0 ambiguë (D1..D5 viables après research)
  - Audit-gate verdict FAIL ou >=3 P1 → re-conception requise
  - Désaccord plan §Phase X vs état réel du code (drift)
  - G8 verdict DESIGN-CONFLICT → présenter pivot_proposal, attendre
    arbitrage utilisateur sur option A/B/C

Anti-pattern : "tu confirmes que je commit chore(planning) d'abord ?"
— la procédure répond, pas l'utilisateur.

# === Mode ultracode + orchestration multi-agents (defaut session SBFB) ===

Cette session est en mode ULTRACODE par defaut : on optimise pour la
reponse la plus exhaustive et correcte, pas la plus rapide ni la moins
chere. Le cout en tokens n'est PAS une contrainte (coherent avec la
directive PO « sprints ultra-complets »).

ORCHESTRATION PAR WORKFLOW ULTRACODE (defaut pour toute etape de
DECOUVERTE et de VERIFICATION : kickoff, audit gate, preflight de
phase, review de phase, recherche multi-source). Pour chacune de ces
etapes, preferer un Workflow multi-agents (fan-out + verification
adversariale + synthese) plutot qu'un agent unique. Le pattern
Workflow brille en LECTURE/verification ; il ne code JAMAIS une phase
en parallele (editions concurrentes des fichiers partages
http.rs/runtime.rs = conflits + commit atomique casse). L'ECRITURE
d'une phase reste main-thread, sequentielle, un commit atomique par
phase.

  - Le PREFLIGHT de phase (G8) = Workflow : fan-out des 5 scans
    factuels (S1a OSS prior-art + S1b deps + S2 decisions historiques
    + S3 threat model + S4 wire format), synthese -> verdict (§4.5.7).
    Remplace l'agent unique `nexus-phase-preflight-deep`.
  - La REVIEW de phase (G4) = Workflow : fan-out des dimensions de
    review + verification adversariale, synthese -> verdict (§4.5.7).
    Remplace l'agent unique `nexus-phase-review-deep` + le gate
    superviseur. Plus aucune consultation GO/BLOCK.
  - Codex (GPT-5.6 Sol) reste la verification croisee externe apres la
    review Workflow PASS-PENDING.
  - L'unique gate AUTOMATISE avant commit est le hook mecanique
    `phase-precommit-lightcheck.sh` (staging coherence, body 9
    sections, artefact Codex brut). Il n'y a plus de superviseur
    process (`nexus-process-supervisor`) ni de gates GO/BLOCK
    `G-SPAWN`/`G-PREFLIGHT`/`G-REVIEW`/`G-CODEX`/`G-COMMIT`/`G-POST`.

L'outil Workflow exige un opt-in, satisfait quand le toggle runtime
ultracode est on (un system-reminder le confirme) : garder ce toggle
ACTIVE pour les sessions SBFB. Fallback si l'opt-in Workflow est
absent : plusieurs appels `Agent` en parallele dans un meme tour, ou
les skills `nexus-phase-preflight` / `nexus-phase-review` en
sequentiel — jamais un retour au superviseur supprime.

CONTRAINTE DE COMPOSITION (a respecter strictement). Les Workflows
lances en arriere-plan et le Monitor notifient en FIN DE TOUR : ils
composent proprement avec un working tree PROPRE. En milieu de phase,
lancer l'orchestration Workflow AVANT que l'arbre de travail ne soit
committe-sale d'une maniere qui bloquerait, OU s'appuyer sur des
agents paralleles en AVANT-PLAN (fan-out synchrone, un seul tour
maintenu vivant jusqu'au verdict) quand un tour doit rester actif.
Le preflight et la review etant des etapes de lecture, le fan-out
avant-plan est le mode par defaut sur.

Regle de decision rapide :
  - lecture/verif large (audit, preflight, review, recherche,
    balayage N fichiers) -> Workflow (fan-out) OU plusieurs Agent en
    parallele dans un meme tour ;
  - ecriture coherente d'une phase -> main-thread sequentiel, gate
    final = hook lightcheck au commit ;
  - tache conversationnelle/triviale -> solo.

Note agents : les 5 fichiers `.claude/agents/*.md` restants
(`nexus-audit-gate`, `nexus-sprint-kickoff`, `nexus-phase-preflight-deep`,
`nexus-phase-review-deep`, `nexus-phase-auditor`) restent enregistres
(incident BOM 2026-06-15 corrige) et servent de fallback pour les etapes
ou le Workflow n'est pas disponible ; `nexus-process-supervisor` est
supprime (amendement 2026-06-17) et n'est plus invoque.

# === Pre-flight (un seul copy-paste, lis tout l'output) ===

# IMPORTANT : ce bloc est execute par l'outil Bash de Claude Code.
# Ne pas traduire en PowerShell. Utiliser `2>/dev/null` et `head`,
# jamais les redirections ou cmdlets PowerShell.

git log --oneline -10
git status --short
ls .planning/active/
ls .planning/archive/
head -1 docs/claude/SPRINT_LOG.md && grep -E "^## v[0-9]" docs/claude/SPRINT_LOG.md
grep -Ei '^- \[SBFB pivot|tip ' "$HOME/.claude/projects/C--Users-FlowUP-Documents-Code-nexus/memory/MEMORY.md" 2>/dev/null || true
{ grep -Ei 'Tip ' "$HOME/.claude/projects/C--Users-FlowUP-Documents-Code-nexus/memory/nexus_grid_pivot.md" 2>/dev/null || true; } | head -1

# G2 — triggers événementiels actifs sur artefacts long-life
grep -lE 'triggers_revalidate' docs/security/*.md docs/rust/PATTERNS.md docs/shell/PATTERNS.md 2>/dev/null || true

# G6 — fraîcheur memory vs tip master (ouvrir question si > 2 sprints sans touch)
{ ls -la "$HOME/.claude/projects/C--Users-FlowUP-Documents-Code-nexus/memory/" 2>/dev/null || true; } | head -20

# G8 hint — historical decisions qui pourraient flager DESIGN-CONFLICT
# (lecture rapide, signal uniquement, le scan S2 complet vit dans skill preflight)
git log --all --extended-regexp --grep='DEVIATION|rejected|threat-model|scope-cut' --oneline | head -10 || true

# === Plan sequentiel + gate mecanique (OBLIGATOIRE, avant tout) ===

IMMEDIATEMENT apres le pre-flight, AVANT la detection de cas :

  1. Creer un plan sequentiel visible dans le contexte principal.
     Utiliser les outils Claude Code `TaskCreate` / `TaskUpdate` /
     `TaskList` si disponibles. Si cette version expose seulement
     `TodoWrite`, utiliser `TodoWrite` comme fallback.

     Plan minimal attendu (Cas B) :
       - Pre-flight resume + detection du cas
       - Lire decision + active plan files
       - Preflight Workflow de phase (fan-out 5 scans -> verdict)
       - Si EXECUTE/PLAN-ADAPT : coder uniquement dans le scope valide
       - Review Workflow (fan-out dimensions + adversarial) -> PASS-PENDING
       - Codex : codex_review brut + reconciliation review PASS
       - Commit (hook lightcheck = gate mecanique) + planning/memory a jour

     Regles plan :
       - exactement une tache `in_progress` ;
       - mise a jour AVANT et APRES chaque etape ;
       - aucune tache `completed` sans artefact/verdict correspondant
         (preflight.md, review.md PASS, codex_review.md brut, commit SHA).

  2. PLUS DE SUPERVISEUR. Le superviseur long-lived
     (`nexus-process-supervisor` en teammate Agent Team) et les
     consultations de gate GO/BLOCK sont SUPPRIMES (amendement
     2026-06-17). Ne PAS creer de teammate supervisor, ne PAS
     invoquer d'Agent `supervisor-gate`, ne PAS attendre de verdict
     GO-* / BLOCK-* entre les etapes. Le verdict du preflight et de
     la review est porte par l'orchestration Workflow elle-meme ;
     Codex est la verification croisee ; le commit n'a qu'un gate.

  3. Gate mecanique unique : le hook `phase-precommit-lightcheck.sh`
     s'execute automatiquement au commit et BLOQUE si un invariant
     est viole (staging coherence STRICT BLOCK, body 9 sections
     Check 9, artefact Codex brut Check 7, design_review.md Phase A
     Check 5). C'est le seul mecanisme d'arret automatise restant.
     Le hook `Stop` peut aussi bloquer une fin de tour qui sonne
     "termine" alors que le repo n'est pas propre. Aucune action
     manuelle de consultation de gate n'est requise — enchainer les
     etapes du plan et laisser le hook trancher au commit.

# === Regle modele agents (§7.1.1) ===

Ne JAMAIS passer le parametre `model` dans les appels Agent().
Les 6 agents ont `model: claude-opus-4-8[1m]` dans leur frontmatter
(.claude/agents/*.md). Le parametre `model` de l'outil Agent()
OVERRIDE le frontmatter — et les alias (`opus`) n'ont pas de
resolution garantie vers la bonne version. Utiliser l'ID explicite
`claude-opus-4-8[1m]`. Omission = heritage correct. Bascule 4.6 → 4.8
le 2026-05-28 ; la regression MRCR qui motivait le pin 4.6 (cf. §7.x
A/B test S22) etait specifique a Opus 4.7 et ne s'applique pas a 4.8.

# === Détection du cas + routage agent ===

Compare ce que tu vois avec les 4 cas ci-dessous. Le main thread
DETECTE le cas et INVOQUE l'agent spécialisé. Il ne joue PAS la
procédure lui-même (sauf Cas D hotfix).

  Cas A — Audit gate à jouer
    Signal : .planning/active/ vide OU contient SEULEMENT le
             kickoff/plan d'un sprint dont le précédent vient de
             fermer (audit_findings absent dans active/ ET dans
             archive/v{X}/).
    ACTION : INVOQUER agent `nexus-audit-gate`.
             L'agent lit audit_plan, ingère le diff complet du
             sprint N-1, joue les 11 tracks du canon
             `prompts/agent/audit-gate-checks.md` (A suites,
             B security, C patterns, D scope, E tests delta,
             F review files, G carry-overs, H HARDENING,
             I meta-process, J testabilite standing,
             K docs-contract closure standing), produit
             `.planning/active/sprint{N-1}_audit_findings.md` avec
             verdict PASS / CONDITIONAL PASS / FAIL, et ecrit les
             commits fix(sprint{N-1}) pour les P0/P1.
    Verdict G4 (rigor signal) : 0 P0/P1 ET 0 P2+ = CONCERN
             (pas PASS). PASS exige >=1 P2+ documente.
    Variante ultracode : jouer l'audit gate en Workflow (fan-out
             des 11 tracks + verification adversariale + synthese du
             verdict) plutot qu'un agent unique. Fallback : si ni
             Workflow ni l'agent ne sont disponibles, le main thread
             joue manuellement la procedure §3 + §8.

    Chaque commit fix(sprint{N-1}) passe par le hook lightcheck
    (gate mecanique). Pas de consultation superviseur.

  Cas B — Sprint en cours
    Signal : .planning/active/ contient sprint{N}_kickoff.md +
             sprint{N}_plan.md mais pas verification.md.
    Identifier la phase X suivante : git log + plan.md.

    AVANT code (G8 preflight = Workflow ultracode) :
      ORCHESTRER un Workflow preflight pour la phase X : fan-out des
      5 scans (S1a OSS prior art profond + S1b deps/CVE + S2 decisions
      historiques complet + S3 threat model + S4 wire format) +
      verification adversariale, puis synthese qui produit
      `.planning/active/sprint{N}_phase_{X}_preflight.md` avec
      verdict EXECUTE / PLAN-ADAPT / SCOPE-CUT-CONSISTENT /
      DESIGN-CONFLICT (§4.5.7).
      Lancer ce Workflow AVANT que l'arbre de travail ne devienne
      sale, OU en fan-out avant-plan (synchrone) si le tour doit
      rester vivant jusqu'au verdict (contrainte §4.5).
      Si DESIGN-CONFLICT : STOP, lire pivot_proposal, arbitrage
      utilisateur sur option A/B/C.
      Si PLAN-ADAPT : le code suit l'approche corrigée dans le
      preflight (pas le plan original).
      Fallback si l'opt-in Workflow est absent : plusieurs Agent en
      parallele dans un meme tour, ou skill nexus-phase-preflight
      (profondeur reduite, memes verdicts). PAS de consultation
      superviseur, PAS d'attente de GO-PREFLIGHT — le verdict du
      Workflow suffit pour enchainer.

    PENDANT code : le main thread implémente la phase
      conformément au plan (ou à l'adaptation PLAN-ADAPT).
      Avant Phase A UNIQUEMENT — verifier que
      sprint{N}_design_review.md existe (G1, §6.1.1). Le hook
      lightcheck Check 5 bloque mecaniquement le commit Phase A
      sans ce fichier.
      Avant scope cut S+1 (G7) : verifier compteur reports de
      chaque carry (§6.2.1 Regle 2).

    APRÈS code, AVANT commit (review = Workflow ultracode) :
      ORCHESTRER un Workflow review : fan-out des dimensions (diff
      complet ligne par ligne + 3 blocs verification §7.4 + branch
      coverage semantique + scope cuts semantiques + research
      grounding + securite deep + livrables + patterns) +
      verification adversariale des findings, puis synthese qui
      produit `.planning/active/sprint{N}_phase_{X}_review.md` avec
      verdict PASS-PENDING / CONCERN / FAIL (§4.5.7). PASS-PENDING
      signifie uniquement "review OK, Codex pas encore fait" ; ce
      n'est jamais un verdict final committable.
      Composition §4.5 : si le tour doit rester vivant jusqu'au
      verdict (arbre potentiellement sale), preferer le fan-out
      avant-plan synchrone ; reserver l'arriere-plan a un arbre propre.
      Si FAIL : corriger les P0/P1, ré-orchestrer le Workflow review.
      Fallback si l'opt-in Workflow est absent : plusieurs Agent en
      parallele dans un meme tour, ou skill nexus-phase-review
      (profondeur reduite). PAS de consultation superviseur, PAS
      d'attente de GO-REVIEW — le verdict PASS-PENDING du Workflow
      autorise directement le passage a Codex.

    APRÈS review PASS-PENDING, AVANT commit (Codex §4.5) :
      Lancer la verification croisee Codex GPT-5.6 Sol pour TOUTES
      les phases sans exception (§4.5.6 zero exemption).
      Ecrire prompt `.git/CODEX_SPRINT{N}_PHASE_{X}.txt`, lancer via
      `Get-Content | codex exec -m gpt-5.6-sol -c
      model_reasoning_effort=max -o .planning/active/
      sprint{N}_phase_{X}_codex_review.md` (slug exact `sol`, effort
      `max`, requiert CLI codex >=0.144.1, cf. §4.5.2 pour les parametres).
      Le fichier codex_review.md DOIT etre l'output BRUT de
      `codex exec -o`. Claude NE DOIT PAS le reecrire, le
      condenser, ni le resumer. Le hook lightcheck Check 7 verifie
      l'authenticite (format par-livrable, fichier:ligne, evidence).
      Si GAPs P0/P1 : corriger, puis BOUCLE COMPLETE :
        1. Re-run suites §7.4
        2. Ré-orchestrer le Workflow review (verdict PASS-PENDING
           attendu si clean)
        3. Re-lancer Codex
        Boucle jusqu'a CLEAN ou P2/P3 documentes uniquement.
      Si GAPs P2/P3 : documenter dans commit body.
      Quand Codex est reconcilié : promouvoir le review.md a
      `## Verdict: PASS` et ajouter `## Codex reconciliation`
      (rapport Codex lu, GAPs corriges/documentes, suites relancees
      si correction). Ne pas modifier le fichier Codex brut.
      Le fichier codex_review.md est enforce par lightcheck
      Check 7 (STRICT BLOCK sur Phase feat/fix/docs/test/refactor) :
      presence, staging, non-reecriture Claude, verdicts par livrable,
      evidence fichier:ligne, coherence PARTIEL/GAP entre artefact et body.
      Sequence stricte : review PASS-PENDING → Codex →
      reconciliation/promote review PASS → commit.
      JAMAIS committer avant le verdict Codex et le review final PASS.

    AVANT commit : verifier que tous les artefacts sont presents
      (preflight.md, review.md PASS, codex_review.md brut) et que le
      commit body couvre les 9 sections. Le hook lightcheck tranche
      mecaniquement au commit (staging + body + Codex + design_review
      Phase A) ; il n'y a plus de consultation superviseur GO/BLOCK.

    Livrable final : 1 commit feat(scope): Sprint N Phase X.

    APRÈS commit : mettre planning/memory a jour (G6 — §5.1.1,
      §7.5) et s'assurer que la phase suivante est claire. Pas de
      gate G-POST ; le hook `Stop` est le backstop si le repo reste
      sale en fin de tour.

  Cas C — Nouveau sprint à ouvrir
    Signal : .planning/active/ contient au max le
             sprint{N-1}_audit_findings.md avec verdict PASS ou
             CONDITIONAL PASS levé. Le sprint N-1 est complètement
             clos.
    ACTION : INVOQUER agent `nexus-sprint-kickoff`.
             L'agent lit l'état complet du projet, exécute une
             recherche ultra-profonde pour chaque D1..D5 (context7
             + WebSearch + code OSS), joue G1 Design Review Board,
             G2 triggers, G7 carry-overs + ROADMAP_COMMITMENTS,
             G9 factual research, et produit 3 fichiers :
             `.planning/active/sprint{N}_kickoff.md` +
             `sprint{N}_plan.md` + `sprint{N}_design_review.md`.
    Variante ultracode : orchestrer le kickoff en Workflow (fan-out
             recherche D1..D5 + G1 design review + G2/G7/G9 +
             verification adversariale + synthese) plutot qu'un agent
             unique ; l'ecriture des 3 fichiers reste sequentielle
             (1 writer par fichier — un mega-writer en un tour
             sature et meurt, cf. memory).
    Après retour Workflow/agent :
      1. Review kickoff D1..D5 + Checkpoint §11
      2. git mv migration active/ → archive/ si nécessaire
      3. Memory carry-over G6 : fusionner manuellement
         sprint{N-1}_verification.md §5 dans les memories
      4. Commit chore(planning): Sprint N kickoff + plan
         (gate mecanique = hook lightcheck ; pas de superviseur)
      5. Update memory nexus_grid_pivot.md
    Fallback : si ni Workflow ni l'agent ne sont disponibles, le main
             thread joue manuellement la procedure §2 + §6.1.1 + §6.2.1.

  Cas D — Hotfix hors sprint (main thread direct)
    Signal : utilisateur demande explicitement un fix urgent.
    Mode : commit fix(...) ciblé, ne touche pas .planning/.
    Hook lightcheck reste actif : staging coherence avant commit
                meme en hotfix.
    G8 NON applicable (pas de plan §Phase X à challenger). Mais
                si le hotfix touche threat model ou wire format
                pre-launch (rare), faire un S4 FULL SCAN manuel
                reprenant les commandes du skill
                nexus-phase-preflight SKILL.md Step 5 :
                  grep -rE "_VERSION\s*[:=]\s*[0-9]+" crates/nexus-core-rs/src/
                  grep -A 10 "Pre-launch protocol" memory/nexus_grid_pivot.md
                  grep -A 10 "Pre-launch protocol policy" CLAUDE.md
                  git log --grep="DEVIATION\|rejected" -- <fichiers hotfix>
                Si conflit -> escalation user avant fix.
    Pas d'agent specialise — le main thread gere directement.
    Gate au commit : hook lightcheck (staging coherence reste
           obligatoire meme en hotfix). Plus de consultation
           superviseur G-COMMIT / G-POST — supprimes pour tous les
           cas, y compris hotfix.

# === Lecture ciblée par cas ===

Le main thread charge le MINIMUM nécessaire pour router vers
l'agent. L'agent porte sa propre procédure de lecture approfondie
(définie dans `.claude/agents/*.md`).

  Pour TOUS les cas (lecture commune minimale) :
    1. CLAUDE.md (racine) — projet + table agents + pointeur workflow
    2. docs/claude/README.md §3 (audit gate) + §4 (commit
       discipline) + §6 (conventions)
    3. memory MEMORY.md (l'index)

  Cas A — l'agent nexus-audit-gate charge lui-même :
    audit_plan.md, kickoff.md, plan.md, verification.md du sprint
    audité, PATTERNS.md (après opinion formée), THREAT_MODEL.md,
    HARDENING_ROADMAP.md. Cf. `.claude/agents/nexus-audit-gate.md`
    §3 Step 0.

  Cas B — le main thread charge :
    - .planning/active/sprint{N}_kickoff.md (D1..D5)
    - .planning/active/sprint{N}_plan.md §Phase X visée
    Les agents preflight-deep et review-deep chargent eux-mêmes
    leurs contextes approfondis (memories, PATTERNS.md, threat
    model, etc.). Cf. `.claude/agents/nexus-phase-preflight-deep.md`
    et `.claude/agents/nexus-phase-review-deep.md`.

  Cas C — l'agent nexus-sprint-kickoff charge lui-même :
    SPRINT_LOG.md, kickoff/plan/verification du sprint précédent,
    roadmap, HARDENING_ROADMAP.md, memories, THREAT_MODEL.md,
    PATTERNS.md. Cf. `.claude/agents/nexus-sprint-kickoff.md`
    §4 Step 1.

  Cas D :
    - juste le code touché, rien d'autre
    - skill nexus-phase-review Step 1bis (staging coherence
      reste obligatoire meme hotfix)

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
  2. **N'attend PAS confirmation** quand le cas est procéduralement
     déterminé. Mêmes règles que §Principe d'autonomie ci-dessus.
  3. Respecte les D1..D5 figées et les scope cuts du sprint
     courant — ne rebats pas (G8 peut PROPOSER une remise en
     question Day 0 mais ne tranche jamais)
  4. Pas de band-aid fix, pas d'emoji, pas d'amend, pas de
     force push
  5. Avant chaque commit : verifier toutes les suites pertinentes
     (cf. §7.4 ci-dessous)
  6. Cadence docs-contrat (§6.12, canon S79 Phase B) : toute
     primitive de FRONTIÈRE porte son étiquette générée PAR PHASE
     (dans le commit de la primitive) ; GUIDE + llms.txt en UNE
     phase de clôture (livrable de fermabilité : DoD (d) §4 +
     §3.3 livrable 3). Le juge est le TEST-ACTEUR : « qui LIT
     cette primitive ? » (autre nœud = wire, client externe = API
     — Y COMPRIS une API loopback lue par un runtime distinct
     comme le front Operator —, app réseau = contrat/CSP, autre
     LLM = prompt-kind/knowledge). JAMAIS le test « 0 wire bump »
     (conflation qui a fait taire S80). À CHAQUE phase touchant
     une frontière : étiquette dans le commit + la frontière
     s'ajoute à la liste de clôture du wrap-up. Commentaires de
     provenance in-code vers le PASSÉ immuable seulement — JAMAIS
     une promesse future (anti STALE-PHASE-K). Gate BLOQUANT
     `scripts/check-frontier-contracts.sh` (câblé CI 3 surfaces ;
     OPT-IN — il ne détecte PAS une frontière neuve jamais
     annotée, et il n'y a PAS de backstop au commit : la
     détection des frontières neuves est portée par le preflight/
     review de phase, pas par un gate). Détail : §6.12 + `docs/rust/PATTERNS.md`
     §P70 + `docs/agent/AGENT_SYSTEM.md` §7.

Langue : français pour réponses utilisateur, docs planning,
commit bodies. Anglais pour code, identifiants, commit titles.
```
<!-- BOOTSTRAP:END -->

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

Co-Authored-By: Claude <model> <noreply@anthropic.com>
```

**Cas B — feat (Phase X du sprint en cours)** :

```
feat(scope): Sprint N Phase X — titre court

## Contexte
<1-3 paragraphes pourquoi cette phase existe>

## Fichiers
| Fichier | Rôle |
|---------|------|
| path/file.rs | <rôle> |

## Delta tests
| Suite | Avant | Après | Delta |
|-------|-------|-------|-------|
| Rust workspace | NNN | NNN | +X |
| Vitest unit | NNN | NNN | +X |

## Verification §7.4
<suites finales complètes>

## Scope cuts respectés (kickoff §8)
<items NOT exhaustifs>

## G8 traceability
- Preflight : <sha/fichier> verdict <verdict>
- Review : <sha/fichier> verdict PASS final après Codex reconciliation

## Pre-launch protocol
<wire/protocol invariants>

## Codex verification
- Rapport : sprint{N}_phase_{X}_codex_review.md
- Reconciliation : review.md promu PASS ; PASS-PENDING absent du commit final

## Carry closure / Unblock
<graphe de dépendances ou Aucun>

Co-Authored-By: Claude <model> <noreply@anthropic.com>
```

> **Headers des 9 sections (enforcement)** : chaque `## <titre>` doit
> **commencer** par le titre canonique ; un suffixe descriptif est toléré
> (ex. `## Scope cuts respectés (kickoff §8)`, `## Verification §7.4`,
> `## Carry closure / Unblock`). Le hook bash et `agentctl.py` matchent en
> préfixe `^## <titre>\b` (incohérence `\s*$` corrigée S80 Phase B). En cas
> de doute, le header **bare** (`## Scope cuts`) est toujours sûr. La
> review promue porte un header EXACT `## Verdict: PASS`. Pour un body long :
> `git commit -F fichier.txt` avec un **chemin sans guillemets ni pipe**
> (le hook lit `tool_input.command` ; un `-F "x" | tail` casse l'extraction).

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

Co-Authored-By: Claude <model> <noreply@anthropic.com>
```

**Cas C — docs (clôture sprint, phase de sortie)** :

**Pré-requis lecture** : §2.3 (verification.md 9 sections), §2.4
(audit_plan.md 6 sections), §4.4 (routing findings phase reviews).
Parser les `sprint{N}_phase_*_review.md` AVANT d'écrire
l'audit_plan.

```
docs(sprint{N}): verification + audit plan for Sprint N+1

Verification : NN/NN fail-fast verts, delta tests +NN cumulé
Audit plan : N tracks A..G pour Sprint N+1 Phase 0
  + §4.4 routing : N findings P2/P3 des phase reviews routés
PATTERNS.md : <ajouts pattern + tech debt T-NN>

Tip d'entrée : {SHA}
Tip de sortie : {SHA}
Commit stack : {N commits feat/test} + ce commit docs

Co-Authored-By: Claude <model> <noreply@anthropic.com>
```

**Cas D — hotfix hors sprint** :

```
fix: <résumé court>

Contexte : <pourquoi hors cycle sprint>
Root cause : <diagnostic>
Fix : <ce qui change>
Tests : <validation>

Co-Authored-By: Claude <model> <noreply@anthropic.com>
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
# Rust — nextest workspace (process-per-test, detecte les flakes)
cargo fmt --all --check && \
cargo clippy --workspace --all-targets --locked -- -D warnings && \
cargo nextest run --workspace --locked && \
cargo test --workspace --locked --doc

# Frontend
cd web && \
npx tsc --noEmit -p tsconfig.app.json && \
npm run lint && \
npm run test:unit && \
npm run build && \
npm run size && \
bash scripts/scan-en-strings.sh && \
cd ..
```

Pendant l'**itération** d'une phase, scope au crate touché plutôt
que de lancer le workspace entier à chaque edit — cf. §4.3 pour le
détail des deux modes (itération rapide vs verification finale).

**Pre-commit final : les 2 blocs sont OBLIGATOIRES**, même si la
phase ne touche qu'un seul langage. Un changement Rust
(`http.rs` endpoint) peut impacter un test frontend. Le coût des
2 blocs complets est ~5 min ; le coût d'une régression cross-stack
non détectée est un fix(sprint) + audit P1.
Ne pas filtrer par "langage touché" — lancer les 2 blocs.

Tout rouge bloque le commit. Pas de `--no-verify`, pas de
`#[ignore]` ajouté pour faire passer. Root cause d'abord.

**Pre-push obligatoire : reproduction Docker du pipeline CI.**
Le WSL Linux natif (rustc 1.95, node 22) ne reproduit PAS le VPS
Woodpecker (rustc 1.94 conteneur, node 20 conteneur). Seul un run
Docker avec les memes images garantit la parite. **AVANT tout
push**, lancer les 3 blocs Docker :

Bloc Rust :
```powershell
docker run --rm -v "${PWD}:/workspace" -w /workspace rust:1.94@sha256:b644cc33aee7a2b32ff3e1198711f8ad3a69ae29a58e1a674e97f75776b88186 sh -lc "rustup component add rustfmt clippy && cargo fmt --all --check && cargo clippy --workspace --all-targets --locked -- -D warnings && cargo test --workspace --locked && cargo test --workspace --locked --doc"
```

Bloc Frontend (tsc + lint + test + build + size) :
```powershell
docker run --rm -v "${PWD}:/workspace" -w /workspace node:20@sha256:cacf10e99285cbbc891452e31249c1b5ec3ba225f40028fae946b75aeaf1b66a sh -lc "cd web && npm ci && npx tsc --noEmit -p tsconfig.app.json && npm run lint && npm run test:unit && npm run build && npm run size"
```

Bloc SPDX :
```powershell
docker run --rm -v "${PWD}:/workspace" -w /workspace bash:5@sha256:2003051c5eb5154cbd44fd4b1a2b8f1be886517b383813c998c72cb15840357f bash scripts/check-spdx.sh
```

Cycle obligatoire = fix → Docker pipeline local → tout vert →
commit + push. Lancer les commandes Docker en `run_in_background`.

Note : le WSL reste utile pour les tests rapides unitaires pendant
le developpement, mais le gate final avant push DOIT etre Docker.

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
4. Jouer les 11 tracks (A..K, canon `prompts/agent/
   audit-gate-checks.md` ; J testabilité + K docs-contract =
   standing) avec la méthode concrète du audit_plan
5. Écrire `sprint{N-1}_audit_findings.md` avec findings
   ventilés P0 / P1 / P2 / P3
6. Si P0 ou P1 : produire les commits `fix(sprint{N-1}):
   ...` avant la fermeture de la gate
7. Committer le findings doc + les fix dans master
8. Retourner le verdict à l'utilisateur qui ouvre alors
   la Phase A du sprint en cours

Le signal prime sur le volume — pas de limite de temps.

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

**Bad** : le commit de phase de sortie contient aussi un
petit ajout de code « oublié ».
**Good** : la phase de sortie est strictement docs. Un fix de code
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

## 10.1 Note historique : process bankruptcy (2026-04-20, `2438c59`)

Le sprint 22 Phase D a conclu un A/B test Opus 4.6 vs 4.7 et
un audit causal du process accumulé S16-S22. Résultat :

- **Décision modèle** : rester Opus 4.6 (régression MRCR
  mesurée : -32.7pp @256K, -46.1pp @1M sur Opus 4.7)
- **Hooks supprimés** (0 valeur causale prouvée, 200 appels
  Haiku/session) : narration terminal, sidecar terminal,
  TDD guard, statusline, post-commit-memory
- **Hooks conservés** (valeur causale prouvée) :
  `phase-auditor-gate.sh` (2 DESIGN-CONFLICT détectés S20/S21),
  `phase-precommit-lightcheck.sh`, `verify-on-write.sh`,
  `session-start-autoinstall.sh`
- **CLAUDE.md** réduit de 503 à 232 lignes (détails redondants
  avec docs/)
- **Documents supprimés** : `MRCR_SELFTEST.md` (moot après
  décision modèle), `MODEL_AND_EFFORT.md` (optimisation
  nice-to-have, non-bloquant)

Diagnostic clé : *"la qualité code vient du modèle + instructions
CLAUDE.md, pas des hooks/reviews/cérémonies."* L'affirmation est
**partiellement fausse** : analyse rétrospective S23 montre que le
hook `phase-auditor-gate.sh` a une valeur causale prouvée (2 P1
catchés pré-commit en S22 Phase C + Phase D). Le hook conditionnel
`34dacdc` introduit par le bankruptcy a causé 5/6 phases S23 sans
review → le P1 C-1 (canonical bytes) a échappé au filet.

**Correction S24 Rework v2** (après délibération indépendante sur
le rework Opus 4.7 initial) :

- **KEEP drop hook conditionnel C1-C9** — retour audit
  inconditionnel sur tout commit `feat(sprintN Phase X)`. Valeur
  causale directe (P1 S23 échappé via regex C1 mal calibrée sur
  `task.rs`). Alternative "affiner C1-C9" rejetée : effort > valeur.
- **KEEP drop G5 Working tree audit du body commit** —
  catégorisation PHASE/CRAFT/DEBT/NOISE redondante avec hook
  lightcheck Check 1 (staging coherence STRICT BLOCK) + split
  commits `chore(planning)` visibles dans `git log --stat`. L'audit
  gate Phase 0 reconstitue la catégorisation S+1 via `git log
  --stat` + bodies split, pas via table body dédiée. **Perte
  assumée** : `.gitignore` update auto sur NOISE et grep scope-cuts
  sur DEBT ne sont plus surveillés par un mécanisme dédié — à
  compenser par discipline auditor Phase 0 qui reste obligatoire.
- **REINTRODUCE §4.4 phase de sortie parse reviews** — mode de défaillance
  documenté (P2-S21-4). Coût README ~40 lignes lu 1×/session.
- **REINTRODUCE G1 extensions crypto-spec + custom-Rust-stack** —
  les deux règles adossées à incidents datés (Tor PoW 2023 / Equi-X
  migration, S21 D2 PII / nexus-pii-rs gap). Dissuasion documentée
  > détection observable.
- **REINTRODUCE §6.7 bullet "pas d'estimation LOC amont"** —
  pattern observé 3× S22 (P2-E-2). Contre-mesure active contre
  plafond psychologique cognitif.
- **REINTRODUCE section G8 traceability dans template commit body**
  — artefact self-contained pour audit gate S+1 qui ne re-ouvre pas
  `.planning/active/` archivé. Permet retracer la chaîne preflight
  → phase commit depuis le `git log` seul.

Les hooks narratifs et sidecar restent supprimés (0 valeur causale
confirmée, 200 appels Haiku/session). Méta-finding à amender dans
un prochain sprint : un diff `docs/claude/README.md` ≥ 50 lignes
suppressions devrait déclencher un full audit orphelins (C9-like) —
le rework Opus 4.7 initial avait oublié 3 refs C1-C9 / G5 dans
`SKILL.md` nexus-phase-review + preflight + `TOOLING.md §5.2`,
cleanup inclus dans ce commit.

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
sprint peut proposer une amélioration via sa phase de sortie qui
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
- Sprint 65 : introduction du verification process a deux couches
  (§4.5). Chaque phase est verifiee par des agents Claude
  paralleles (1M tokens) pour G8 puis par Codex CLI GPT 5.5 en
  review croisee independante. Preflight G8 passe de sequentiel a
  parallele (~3x). Templates de prompt Codex normalises (§4.5.3,
  §4.5.4). Anti-patterns documentes (§4.5.2).
- Amendement 2026-06-17 : suppression du superviseur process
  (`nexus-process-supervisor`) et des consultations de gate GO/BLOCK
  (`G-SPAWN`/`G-PREFLIGHT`/`G-REVIEW`/`G-CODEX`/`G-COMMIT`/`G-POST`).
  Le preflight et la review deviennent des **Workflows ultracode**
  (fan-out + verification adversariale + synthese, §4.5.7) ; Codex
  reste la verification croisee externe ; le hook mecanique
  `phase-precommit-lightcheck.sh` devient l'unique gate automatise
  au commit. Contrainte de composition documentee (§4.5) : les
  Workflows arriere-plan / Monitor notifient en fin de tour donc
  composent avec un arbre propre — en milieu de phase, fan-out
  avant-plan synchrone.
