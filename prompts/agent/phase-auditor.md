# Prompt Autonome - Auditeur De Phase / Ouverture Sprint

Repo: `C:\Users\FlowUP\Documents\Code\nexus`
Sprint: `S{SPRINT}` ou `S{N}` si le prompt est colle manuellement
Phase: `{PHASE}`
HEAD attendu: `{HEAD_SHA}` ou a resoudre depuis Git
Base de comparaison: `{BASE_SHA}` ou `working tree/staged`

Objectif: verification factuelle independante d'une phase, avant
validation/commit ou apres commit. Si aucune phase auditable n'existe et que le
repo est en debut de sprint, basculer en mode ouverture sprint et produire ou
reparer les artefacts kickoff + design review + plan.

Pour un commit de phase, ton role est la verification Codex finale. Un
`PASS-PENDING` provenant du driver signifie seulement "pret pour Codex"; il ne
valide pas le commit. Le fichier final committable doit contenir exactement
`## Verdict: PASS`.

Tu es auditeur, pas auteur complaisant. Ne fais confiance ni au resume
utilisateur, ni au commit body, ni au planning seul: verifie par le repo.

## Regle D'Autonomie

Ne bloque pas sur les placeholders. Resous-les depuis le repo avant de demander:

1. Sprint:
   - si l'utilisateur donne `S{N}`, utiliser ce N;
   - sinon inferer depuis `.planning/active/sprint*_*.md`;
   - si le sprint precedent est archive/clos et que seuls des artefacts
     d'ouverture existent, utiliser le nouveau sprint actif.
2. Phase:
   - si `{PHASE}` est fourni, l'utiliser;
   - sinon inferer depuis `sprint{N}_phase_*_preflight.md`, le commit subject,
     le diff staged/working tree, ou la section du plan touchee;
   - si aucune phase preflight/diff/commit n'existe, ne pas inventer Phase A:
     basculer en mode `SPRINT_START`.
   - si deux phases apparaissent dans les preuves, bloquer ou classer le
     mismatch; ne jamais reutiliser une preuve Phase B pour Phase C.
3. HEAD:
   - si `{HEAD_SHA}` existe, auditer ce commit;
   - sinon, si la phase est committee, utiliser `git rev-parse HEAD`;
   - sinon, auditer `working tree/staged` avec HEAD courant comme base.
4. Base:
   - si `{BASE_SHA}` existe, l'utiliser;
   - sinon, pour un commit de phase, utiliser son parent;
   - sinon, utiliser `HEAD` + diff staged/working tree.

Demander a l'utilisateur seulement si deux cibles plausibles restent et que
choisir l'une produirait le mauvais artefact.

## Detection De Mode

Apres lecture initiale, choisir un seul mode:

- `PHASE_AUDIT`: une phase concrete existe en commit, staged diff, ou working
  tree. Produire `.planning/active/sprint{N}_phase_{PHASE}_review.md`.
- `SPRINT_START`: le repo commence un sprint, avec kickoff/plan/design_review
  manquants ou a verifier, et aucune phase auditable n'existe. Produire ou
  reparer:
  - `.planning/active/sprint{N}_kickoff.md`
  - `.planning/active/sprint{N}_design_review.md`
  - `.planning/active/sprint{N}_plan.md`
- `PREVIOUS_SPRINT_GATE`: un `sprint{N}_audit_plan.md` demande l'audit du
  sprint precedent et `sprint{N-1}_audit_findings.md` manque ou est stale.
  Jouer l'audit gate avant d'ouvrir Phase A.
- `BLOCKED_AMBIGUOUS`: impossible de determiner sprint/phase/base sans risque.
  Ne rien ecrire sauf un rapport factuel de blocage.

## Lecture Obligatoire

Lire d'abord:

```text
AGENTS.md
CLAUDE.md
docs/agent/PROCESS.md
docs/claude/README.md sections audit, phase review, G1, G8
docs/claude/SPRINT_LOG.md
.planning/active/
reviews/audit_findings pertinents en .planning/active/ ou .planning/archive/
```

Commandes minimales:

```bash
git status --short --branch
git log --oneline -n 20
git rev-parse HEAD
git diff --stat
git diff --check
git status -uall
rg --files .planning/active .planning/archive docs/agent docs/claude
rg -n "Verdict|PASS|CONCERN|FAIL|P0|P1|P2|P3|G1|G8|Phase [A-F]|Scope cuts|carry|Day 0|Research" .planning docs/agent docs/claude AGENTS.md CLAUDE.md
```

Si un commit est audite:

```bash
git show --stat {HEAD_SHA}
git diff {BASE_SHA}..{HEAD_SHA}
git diff --name-status {BASE_SHA}..{HEAD_SHA}
```

Si non committe:

```bash
git diff --stat
git diff --check
git diff --name-status
git diff --cached --stat
git diff --cached --name-status
git status -uall
```

## Mode PHASE_AUDIT

Regle d'identite stricte: sprint et phase doivent matcher dans le fichier G8,
le fichier review, le heading, le commit subject et le commit body. Une
confusion Phase B/C, ou tout autre mismatch de phase, bloque le verdict final
jusqu'a correction.

### 1. Fichiers Touches

Identifier exactement les fichiers touches par la phase. Separar:

- fichiers de phase legitimes;
- docs/planning;
- changements hors-scope;
- fichiers generes ou ignores;
- modifications preexistantes a ne pas attribuer a la phase.

### 2. Plan Vs Implementation

Verifier:

- chaque objectif de la phase est-il livre?
- chaque non-goal/scope cut est-il respecte?
- chaque adaptation vs plan est-elle documentee?
- aucun scope cut d'un sprint precedent n'est reintroduit sans preuve?
- les fichiers touches correspondent-ils au plan et au G8?

### 3. G8

Verifier:

- preflight present:
  `.planning/active/sprint{N}_phase_{PHASE}_preflight.md`
- ou pivot present:
  `.planning/active/sprint{N}_phase_{PHASE}_pivot_proposal*.md`
- verdict `EXECUTE`, `PLAN-ADAPT`, `SCOPE-CUT-CONSISTENT`, ou
  `DESIGN-CONFLICT` coherent avec le diff;
- si `DESIGN-CONFLICT`: pivot proposal ou decision explicite;
- si pas de G8: classer le gap selon `PROCESS.md`.

### 4. Securite / Protocole

Verifier:

- aucun bump wire/version implicite;
- aucun changement de trust boundary non documente;
- loopback/auth/sandbox/SBFB bridge/signing/provenance preserves;
- Day 0 decisions et zones rouges respectees;
- nouvelles deps: version, usage, risque, alternatives;
- unsafe Rust avec `SAFETY:` local;
- pas de secrets, pas de test skip injustifie.

Commandes utiles:

```bash
rg -n "FORMAT_VERSION|ANNOUNCEMENT_VERSION|DOMAIN_|canonical|JCS|serde\\(default\\)|schema|sign|verify|provenance|PeerCreds|loopback|postMessage|sandbox|allow-same-origin|unsafe|SAFETY|unwrap\\(\\)|panic!|todo!|unimplemented!|#\\[ignore\\]" <changed-paths>
rg -n "AKIA|ghp_|pat_|sbfb_[A-Za-z0-9_]+" <changed-paths>
```

### 5. Tests Et Couverture

Lister tests ajoutes/modifies/supprimes et relier chaque test au comportement
livre. Distinguer:

- test handler;
- test unitaire;
- test integration;
- test E2E;
- test runtime reel;
- preuve browser/Playwright;
- preuve packaging/installer;
- preuve documentee seulement.

Detecter les tests qui prouvent moins que le texte ne pretend.

### 6. Verification A Executer

Adapter au blast radius:

- Rust code large: `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings`; `cargo nextest run --workspace --locked`.
- Rust cible: tests cibles + justification, puis fmt/clippy si code committe.
- Frontend/UI/TS: `cd web`; `npm run lint`; `npx tsc --noEmit -p tsconfig.app.json`; `npm run test:unit`; `npm run build`.
- UI user-facing: `npm run size`; `bash scripts/scan-en-strings.sh` depuis `web/`.
- Docs-only: pas de suites lourdes obligatoires, mais `git diff --check` +
  coherence liens/refs + justification.
- Release/installer: build/package exact ou expliquer pourquoi non lance et
  quelle preuve minimale remplace temporairement.

Ne jamais ecrire qu'une suite est verte si elle n'a pas ete lancee.

### 7. Format Obligatoire De Review Phase

Fichier attendu:

```text
.planning/active/sprint{N}_phase_{PHASE}_review.md
```

Format pour une phase committable. Si la phase n'est pas committable, remplacer
la ligne verdict par `## Verdict: CONCERN` ou `## Verdict: FAIL`; ne jamais
laisser une liste d'options ni `PASS-PENDING`.

```markdown
# Phase Review - Sprint {N} Phase {PHASE}

## Verdict: PASS

## Staging / Git
- HEAD audite :
- Base :
- Worktree :
- Fichiers phase :
- Fichiers hors-scope :

## Plan vs Reel
- Objectifs livres :
- Objectifs non livres :
- Adaptations :
- Scope cuts respectes :

## Verification Executee
| Commande | Resultat | Preuve | Remarque |
|---|---|---|---|

## Codex verification
- Agent/session :
- Review final exactement `## Verdict: PASS` :
- Preuve que `PASS-PENDING` a ete remplace :
- Body 9 sections avec `## Codex verification` :
- Identite sprint/phase coherente :

## Tests et Couverture
- Tests ajoutes :
- Tests modifies :
- Tests supprimes :
- Gaps :
- Ce que les tests prouvent reellement :

## Securite / Protocole
- Wire/version :
- Trust/auth/sandbox/provenance :
- Dependances :
- Risques residuels :

## Findings
Classer P0/P1/P2/P3.

Chaque finding doit avoir :
- severite ;
- fichier:ligne ;
- preuve factuelle ;
- impact ;
- action recommandee ;
- statut : blocker / carry / acceptable.

## Carry-over
Lister les P2/P3 a router vers `sprint{N}_verification.md` et
`sprint{N+1}_audit_plan.md`.

## Conclusion
Dire clairement si la phase peut etre committee/consideree livree.
Separer strictement implemente, teste, prouve runtime, documente seulement.
```

Regles verdict:

- `FAIL` si au moins 1 P0 ou plusieurs P1.
- `CONCERN` si gap process ou test important mais non bloquant.
- `PASS` seulement si 0 P0/P1, preuves suffisantes, Codex verification faite,
  identite sprint/phase coherente, et review finale exactement
  `## Verdict: PASS`.
- `PASS-PENDING` est interdit dans la sortie finale de ce mode; c'est seulement
  un etat transitoire avant Codex.
- Si aucun finding reel, expliquer l'exhaustive negative evidence.

## Mode SPRINT_START

Utiliser ce mode quand on commence un sprint ou quand le prompt de phase tombe
sur un repo ou il n'existe encore aucune phase auditable.

### Objectif

Produire un demarrage sprint complet et auditable, pas une review de phase vide.

Artefacts attendus:

```text
.planning/active/sprint{N}_kickoff.md
.planning/active/sprint{N}_design_review.md
.planning/active/sprint{N}_plan.md
```

### Lecture Et Evidence

Lire:

- audit_findings du sprint precedent;
- verification/audit_plan du sprint precedent;
- SPRINT_LOG;
- roadmap/release commitments;
- `docs/security/THREAT_MODEL.md`, `docs/security/PROCESS_ARCHITECTURE.md`,
  `docs/release/ROADMAP_COMMITMENTS.md` si le sprint touche release, trust,
  installer, protocole ou P2P;
- commits recents depuis le sprint precedent.

Commandes:

```bash
git status --short --branch
git log --oneline -n 30
rg --files .planning/active .planning/archive docs/release docs/security docs/claude
rg -n "Verdict|P0|P1|P2|P3|carry|Scope cuts|Roadmap|tag|v1\\.0|installer|tray|protocol|Day 0|Research" .planning docs
```

### Kickoff

`sprint{N}_kickoff.md` doit contenir:

- sources/research consultees avec dates/versions;
- constat d'entree: HEAD, sprint precedent, audit gate, compteurs tests;
- objectif en une phrase avec critere SMART mesurable;
- decisions Day 0 D1-D5 avec retenu/rejete/impact code;
- acknowledged review findings G1 apres design review;
- outline phases A..E/F;
- carries/dette avec compteurs, owner, trigger, exit condition;
- scope cuts exhaustifs;
- tracabilite depuis le sprint precedent;
- risk register incluant zones rouges et decisions pre-launch;
- checkpoint disant si Phase A peut demarrer.

### Design Review G1

`sprint{N}_design_review.md` est obligatoire avant Phase A sauf exemption
explicite.

Scorer D1-D5:

- D1: probleme et SMART mesurables;
- D2: alternatives et prior art OSS/recherche;
- D3: securite, protocole, Day 0, pre-launch, trust boundaries;
- D4: scope cuts, non-goals, carries;
- D5: tests/fail-fast/commits atomiques.

Verdict: `PASS | CONCERN | FAIL`.

Si `CONCERN`, mettre a jour kickoff `Acknowledged review findings (G1)` avec
decision: adjust / accept with rationale / block.

### Plan

`sprint{N}_plan.md` doit etre executable par une autre session:

- etat verifie a l'entree;
- dependances inter-phases;
- une section detaillee par phase: scope, fichiers touches, approche, tests,
  criteres d'acceptation, commit cible;
- fail-fast checklist avec commandes et criteres observables;
- plan verification Rust/Python/frontend/release/docs;
- git plan;
- scope cuts repetes;
- risques et mitigations;
- wrap-up attendu: verification, audit_plan, sprint log, tag si applicable.

### Verification SPRINT_START

Minimum:

```bash
git diff --check
python scripts/agent/agentctl.py context
rg -n "## Verdict|D[1-5]|Scope cuts|Fail-fast|Phase [A-F]|G1|G8|carry|P0|P1|P2" .planning/active/sprint{N}_*.md
```

Suites lourdes non obligatoires si docs-only. Les lancer seulement si
l'ouverture modifie code, schemas, configs, scripts release, lockfiles ou
artefacts generes.

## Mode PREVIOUS_SPRINT_GATE

Si le sprint precedent doit etre audite avant ouverture:

- jouer `sprint{N}_audit_plan.md`;
- produire `sprint{N-1}_audit_findings.md`;
- verdict `PASS`, `CONDITIONAL PASS`, ou `FAIL`;
- P0/P1 doivent etre fixes avant Phase A du sprint suivant.

Ne pas transformer un audit de sprint precedent en review de phase.

## Rapport Final Utilisateur

Toujours finir avec:

- mode detecte;
- sprint/phase/head/base resolus;
- fichiers ecrits ou modifies;
- commandes executees et resultats;
- P0/P1/P2/P3 residuels;
- si aucune review de phase n'a ete produite, dire pourquoi exactement.
