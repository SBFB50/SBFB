---
name: nexus-phase-preflight
description: Pre-implementation factual evolution check (G8) before writing first code line of a SBFB phase. Runs 4 factual scans (S1 SOTA delta + S2 historical decisions traversed + S3 threat model coverage + S4 wire format invariants) and emits verdict EXECUTE / SCOPE-CUT-CONSISTENT / DESIGN-CONFLICT. Outputs sprint{N}_phase_{X}_preflight.md or sprint{N}_phase_{X}_pivot_proposal.md. Invoquer "preflight phase X", "G8 scan phase", "before coding phase X", ou systematiquement avant le 1er Edit/Write d'une phase Cas B.
allowed-tools:
  - Read
  - Bash
  - Grep
  - Glob
  - Write
  - WebSearch
  - WebFetch
  - mcp__context7__query-docs
  - mcp__context7__resolve-library-id
---

# nexus-phase-preflight

Skill de scan factuel pre-implementation pour une phase SBFB.
Materialise le gate G8 documente dans `docs/claude/README.md §6.9`.
Complete la couche tooling (cf. `docs/claude/TOOLING.md`).

## Efficacite mesuree (S22-S25, 21 preflights)

18/21 EXECUTE, 3/21 SCOPE-CUT-CONSISTENT, 0 DESIGN-CONFLICT.
S1+S2 portent 100% des findings reels (3/21). S3+S4 = 0 finding
en 21 runs. Consolidation post-S25 : S3 passe en FULL quand la
phase introduit un nouveau composant de securite ou wire format
(cf. Step 4 escalation obligatoire S3). S4 FULL deja conditionne
sur canonical.rs/schemas. S1+S2 restent full scans systematiques.

## Quand l'utiliser

- Tu es en Cas B (sprint en cours) et tu t'apprete a ecrire le code
  d'une phase A/B/C/D/E/F
- AVANT le premier `Edit` ou `Write` outil sur du code de la phase
- Avant les commits chore(planning) — non, ceux-la passent par
  `nexus-phase-review` Step 1bis (post-code time)
- L'utilisateur dit "preflight phase X", "G8 scan", "before coding"

## Quand NE PAS l'utiliser

- Hotfix hors sprint (cas D) — pas de plan §Phase X a challenger.
  Mini-S4 manuel suffit si touche threat model ou wire format.
- Sprint kickoff (cas C) — c'est G1 Design Review Board qui filtre,
  pas G8. G8 = pre-phase, pas pre-sprint.
- Phase 0 audit gate (cas A) — tu joues `sprint{N-1}_audit_plan.md`,
  ce n'est pas une nouvelle phase a coder.
- Pre-commit verification — c'est `nexus-phase-review` qui catch
  apres le code, pas G8.

## Procedure

### Step 1 — Identifier le contexte

1. Lire `.planning/active/` pour trouver `sprint{N}_kickoff.md` et
   `sprint{N}_plan.md`
2. Identifier la phase X visee (la prochaine non commitee selon
   `git log --oneline -10`)
3. Extraire du plan §Phase X :
   - Fichiers ciblees (table 8.2 ou equivalent)
   - Libs/deps a ajouter ou bumper (Cargo.toml, pyproject.toml,
     package.json mentions)
   - APIs externes touchees (specs crypto, RFC, etc.)
   - Wire format touche (TaskEntry, ProjectAnnouncement, etc.)
   - Threat model claim (ex : "defense vs Sybil", "anti-DPI")

3bis. **Cas phase vaste (>10 fichiers ciblees Step 1.3)** : activer
   sampling pour eviter timeout `git log` et noise overload. Partitionner
   les fichiers en sous-ensembles par module principal (max 3 groupes,
   ex : crates/rust / packages/python / docs). Pour S2 (Step 3), passer
   `--max-count=100` a `git log` et scanner par groupe, pas fichier-par-
   fichier. Pour S1 (Step 2), prioriser les libs **crypto + wire format
   + network-exposed** en premier, differ les libs purement internes
   (`anyhow`, `tracing`, `serde` struct derive) en sampling LITE
   (version string check uniquement, pas context7 full).

3ter. **Cas phase ad-hoc (plan §Phase X absent)** : une phase peut
   avoir ete inseree via `gsd:insert-phase` (ex : Phase 5.2 decimal)
   ou un hotfix-in-sprint sans update plan.md. Detection : Step 1.2
   trouve une phase en git log ou working tree mais Step 1.3 ne
   trouve pas la section correspondante dans plan.md. Fallback :
   - Utiliser le **commit body** (si la phase a un draft commit fourni
     par l'user) OU le **working tree diff** (si phase partiellement
     code) comme source-of-truth pour "Files touched / Libs / APIs /
     Wire format".
   - Si absence totale (phase mentionnee en code/log mais aucun artefact
     de contexte) -> STOP + demander a user : "Phase X trouvee hors
     plan.md. Est-ce un insert-phase ad-hoc legitime ? Source-of-truth
     a utiliser (commit body / diff) ?"
   - **Jamais skip G8 silencieusement** sur phase ad-hoc : emit un
     preflight.md avec verdict CLEAN minimal ou demander fallback.

4. Lire `docs/claude/README.md §6.9` pour la procedure verdict

### Step 1.5 — Memory consultation (avant scans)

Lire `MEMORY.md` (index) et charger les memories pertinentes pour
la zone fonctionnelle de la phase. L'objectif : eviter de proposer
un design que les memories rejettent ou contraignent.

**Routing table** (source of truth : identique preflight et review) :

| Zone phase | Memory file | Contrainte cle |
|---|---|---|
| (toujours) | `feedback_approach.md` | pick deepest, no band-aid, research before code |
| kudos / fairness / reputation | `fairness_vision.md` + `feedback_kudos_non_monetary.md` | non-monetary, no cost/deposit/stake |
| governance / funding / modele | `vision_model.md` | OpenBSD solo maintainer, no startup |
| deploy / crypto / Ed25519 | `sprint14_keyoxide_decision.md` | from-source verified deploy |
| lib externe / dep / API spec | `feedback_context7_systematic.md` | context7 obligatoire avant code |

Matcher zone depuis Step 1.3 fichiers cibles. Toute modification
future de cette table doit toucher les 2 skills (grep "Routing
table" dans `.claude/skills/*/SKILL.md`).

**Mecanisme** : lire le fichier, extraire en 1-2 lignes la
contrainte pertinente pour cette phase, la noter dans le
preflight.md §Memory consultation. Si une contrainte memory
entre en tension avec le plan §Phase X → signal S2 finding
(reversion check obligatoire).

### Step 2 — Scan S1 : SOTA + OSS prior art + deps

S1 a deux volets : **S1a** (OSS prior art sur l'approche) et
**S1b** (deps/libs versions). S1a est le volet le plus important
— il challenge le *design* du plan, pas juste les versions.

#### S1a — OSS prior art research (OBLIGATOIRE)

La question : **"comment les projets open source matures resolvent
le meme probleme que cette phase ?"**

1. Identifier le domaine fonctionnel de la phase depuis Step 1.3
   (ex : "divergence detection on distributed compute",
   "guardrail pipeline for LLM safety", "DNS fallback DHT")
2. Rechercher :
   ```
   WebSearch "<domaine> open source solution site:github.com"
   WebSearch "<domaine> how <project-reference> implements"
   ```
   Projets de reference par domaine (non-exhaustif, adapter) :
   - compute verification → BOINC, Folding@Home, Golem, Truebit
   - LLM safety/guardrails → NeMo Guardrails, Guardrails AI,
     LangChain, openai-agents-python
   - P2P networking → libp2p, iroh, IPFS, BitTorrent
   - crypto/identity → age, Keyoxide, OpenPGP.js, FROST
   - DNS/transport → hickory-resolver, stubby, dnscrypt-proxy

3. Pour chaque projet pertinent trouve :
   - `mcp__context7__resolve-library-id` si c'est une lib
   - `mcp__context7__query-docs` sur l'API/pattern specifique
   - Lire le code source si necessaire (README, design docs)

4. Comparer l'approche du plan §Phase X avec l'approche OSS :
   - Le plan est-il aligne avec l'etat de l'art ?
   - L'approche OSS revele-t-elle une faille dans le design du plan ?
     (ex : "hash binaire sur output stochastique = 100% faux positifs")
   - Existe-t-il une lib prête qui fait deja ce que la phase code
     from scratch ?

**Findings type S1a** :
- `APPROACH-NAIVE` : le plan propose une approche que les projets
  matures ont abandonnee ou ne recommandent pas (avec evidence URL)
- `APPROACH-ALIGNED` : le plan est conforme au SOTA OSS
- `LIB-EXISTS` : une lib OSS fait deja le job — adopter au lieu
  de recoder (avec licence + audit status)
- `APPROACH-NOVEL` : le plan propose quelque chose que l'OSS ne
  fait pas — acceptable si justifie par le contexte P2P specifique

**Classification** :
- `APPROACH-NAIVE` = finding **bloquant** si evidence claire (pas
  opinion) → le plan DOIT etre adapte avant code
- `LIB-EXISTS` = finding **bloquant** si lib mature + compatible
  licence → evaluer adoption vs recode
- `APPROACH-ALIGNED` / `APPROACH-NOVEL` = non-bloquant

#### S1b — Deps/libs versions (fast-path si pas de nouvelle dep)

Pour chaque lib/spec extraite Step 1.3 :

```bash
# Verifier libs Rust
grep -E "^(name|version)" $(find . -name Cargo.toml -path "*/<crate-touche>/*") 2>/dev/null
```

Puis pour chaque dep critique :

1. `mcp__context7__resolve-library-id` sur la lib name
2. `mcp__context7__query-docs` sur l'API touchee + version stream
3. `WebSearch` "rustsec advisory <crate-name> 2026"
4. `WebSearch` "<lib> CVE 2026" si crypto/security-critical
5. Pour les specs RFC/standards : `WebSearch "<spec> revision 2026"`

Findings type S1b :
- `lib X v Y.Z` — major bump publie depuis plan, breaking changes
- `RFC W` — section X revisee Aug 2026, change semantique
- `CVE-2026-XXXX` critical sur dep transitive Z
- API deprecated, remplacement = nouvelle methode

Output Step 2 : S1a findings + S1b findings, ou "S1: clean" si
aucun delta sur les deux volets.

### Step 2bis — Plan adaptation (si S1a finding bloquant)

Si S1a produit un finding `APPROACH-NAIVE` ou `LIB-EXISTS` :

1. **Ne PAS emettre un DESIGN-CONFLICT** (reserve aux Day 0
   contredites). Emettre un **PLAN-ADAPT**.
2. Rediger dans le preflight.md une section `## Plan adaptation`
   avec :
   - L'evidence OSS (URL + extrait)
   - L'approche corrigee (concrete, pas abstraite)
   - Les fichiers/tests impactes vs le plan original
3. **Adapter le plan inline** : le code de la phase suit
   l'approche corrigee, pas le plan original. Le commit body
   documente la deviation : "Plan §Phase X proposait <ancien>,
   preflight S1a a identifie <evidence>, adapte vers <nouveau>."
4. Le plan.md reste inchange (c'est un snapshot kickoff). La
   deviation est tracee dans preflight.md + commit body.

**Rationale** : un plan fige qui ignore la recherche pre-phase
est pire qu'un plan adapte. Le S24 Phase D illustre le probleme :
le plan disait "hash binaire BLAKE3" pour des outputs LLM
stochastiques, l'OSS (BOINC, Truebit) montre que ca ne marche pas.
Suivre le plan aveuglement a produit du code inoperant. Le plan
est un point de depart, pas un contrat — la recherche pre-phase
le corrige si necessaire.

### Step 3 — Scan S2 : Decisions historiques traversees

```bash
# Pour chaque fichier ciblee Step 1, scanner l'historique
FILES_PHASE=<liste des fichiers plan §Phase X>
for f in $FILES_PHASE; do
  git log --all --grep="DEVIATION\|rejected\|scope-cut\|deliberate\|threat-model" -- "$f" 2>/dev/null
done

# Scanner les commit bodies dans .planning/archive/v*/ pour mots-cles
# de rejet sur la zone fonctionnelle de la phase
grep -rE "DEVIATION deliberee|rejected for|scope-cut at|threat-model" \
  .planning/archive/v*/sprint*_*.md 2>/dev/null

# Scanner memory feedback pour patterns evites
grep -rE "do not|never|reject|avoid" \
  "$HOME/.claude/projects/C--Users-FlowUP-Documents-Code-nexus/memory/feedback_*.md"
```

Findings type :
- `S{N-k} <sha>` a explicitement rejete <pattern> pour raison Z
- Memory feedback dit "ne JAMAIS faire X parce que Y"
- Phase Y du sprint courant ou precedent a deja livre une primitive
  qui resout ce que la phase X reproduit (duplicate)

**Reverse-commit check (obligatoire par finding S2)** : un commit
"rejected" peut avoir ete reverte par un commit ulterieur (decision
changee apres re-research, nouveau contexte, CVE leve). Sans ce
check, S2 produit des faux positifs DESIGN-CONFLICT qui bloquent
inutilement. Pour chaque finding S2 potentiel :

```bash
# Pour chaque commit "rejected" finding, chercher une reversion
FILES_IN_FINDING=<fichiers mentionnes par le commit rejected>
REJECTED_SHA=<sha du commit rejected>

# 1. Grep body commits posterieurs au rejected sur les memes fichiers
git log --all --oneline "${REJECTED_SHA}..HEAD" -- $FILES_IN_FINDING | \
  grep -iE "revert|undo|unblock|now allowed|reopen|supersed"

# 2. Grep commits qui mentionnent explicitement le rejected SHA
git log --all --grep="${REJECTED_SHA}" --oneline

# 3. Lire les bodies des candidats reversion pour confirmer
git show <candidate-sha> --no-patch --format=%B
```

Classification :
- Reversion **confirmee** (body explicite type "revert S{N-k}" +
  rationale "threat closed / CVE fixed / decision updated") → finding
  S2 **declasse** en "historiquement adressee, verifiee le YYYY-MM-
  DD". Log dans preflight.md §S2 mais ne declenche PAS DESIGN-
  CONFLICT.
- Reversion **ambigue** (body mentionne le sujet mais sans revert
  explicite) → finding S2 **CONCERN**, proposer pivot_proposal.md
  section §2 evidence avec les 2 commits (rejected + ambigu) et
  laisser user arbitrer.
- **Pas de reversion** trouvee → finding S2 **DESIGN-CONFLICT**
  plein, suivre procedure Step 6.

Output Step 3 : liste des decisions historiques + sha + raison + **reversion status** (confirmee / ambigue / absente), ou
"S2: clean".

### Step 4 — Scan S3 : Threat model coverage (fast-path gate check)

**Escalation obligatoire S3** : si la phase introduit un **nouveau
wire format** (nouveau `DOMAIN_*_V1`, nouveau `*_FORMAT_VERSION`)
OU un **nouveau composant de securite** (capability store, admin
check, revocation cache, key rotation, credential store) → **S3
FULL SCAN obligatoire** (pas fast-path). Verifier : (a) quel T0-T5
couvre le nouveau composant, (b) quel nouveau vecteur d'attaque il
introduit non couvert par le threat model existant, (c) si un T
supplementaire doit etre documente. Le fast-path ne detecte que les
regressions sur l'existant — il ne detecte pas les gaps sur le
nouveau. (Rationale : analyse post-sprint S25 — Phase B key rotation
et Phase D capabilities etaient en S3 fast-path alors qu'elles
introduisent des composants de securite avec des vecteurs non
modelises dans THREAT_MODEL.md.)

**Fast-path** (autorise UNIQUEMENT si la phase ne cree pas de
nouveau composant de securite ni de nouveau wire format — 0 finding
en 21 runs S22-S25) : S3 est un gate check de contrainte, pas un
scan de decouverte. Executer les grep commands ci-dessous mais NE
PAS lancer context7/WebSearch sauf si un grep renvoie un signal
inattendu (regression threat, pre-requirement manquant). Si tous
greps clean → "S3: clean (fast-path verified)".

```bash
# Lire la threat matrix
test -f docs/security/THREAT_MODEL.md && \
  grep -E "^### T[0-9]|^## " docs/security/THREAT_MODEL.md | head -30

# Lire HARDENING_ROADMAP §3 ligne sprint courant
grep -A 5 "S{N}" docs/security/HARDENING_ROADMAP.md 2>/dev/null

# Lire les findings audit recents
ls .planning/active/sprint*_phase_*_review.md 2>/dev/null
ls .planning/archive/v*/sprint*_audit_findings.md 2>/dev/null | tail -3
```

Pour la primitive proposee par phase X, mapper contre les threats
T0-T5 :
- Quels threats sont couverts ?
- Quels threats sont laisses ouverts ?
- Y a-t-il un threat couvert ailleurs qui devient REGRESSION si
  cette primitive landed (vector reintroduit) ?

Findings type :
- Primitive X couvre T2 mais introduit regression T4 cf. S{N-k}
- Threat Y reste non-adresse, prevu Sprint S{N+m} mais pas dans
  scope phase X — OK si plan documente
- HARDENING_ROADMAP §3 ligne S{N} mentionne pre-requirement non
  livre = blocking

Output Step 4 : matrix coverage + regression flags, ou "S3: clean".

### Step 5 — Scan S4 : Wire format / pre-launch invariants (fast-path gate check)

**Escalation obligatoire** : si les fichiers cibles Step 1.3 incluent
`canonical.rs`, `schemas/`, ou tout fichier contenant `*_VERSION` →
**S4 FULL SCAN obligatoire** (pas fast-path). Lire canonical.rs en
entier + verifier structs touches + Day 0 check complet + context7
sur la spec wire si applicable. Le hook `phase-precommit-lightcheck.sh`
Check 4 emettra un WARN si des fichiers wire-format sont stages sans
trace S4 full dans le preflight.

**Fast-path** (autorise UNIQUEMENT si aucun fichier wire-format dans
le perimetre phase — 0 finding en 17 runs S22-S24) : S4 est un gate
check de contrainte, pas un scan de decouverte. Executer les grep
commands ci-dessous. Si `*_VERSION = 1` et Day 0 preservees → "S4:
clean (fast-path verified)". Si version bump ou Day 0 deviation
detectes → escalade full scan (context7/WebSearch sur la spec touchee).

```bash
# Scanner les wire format versions
grep -rE "_VERSION\s*[:=]\s*[0-9]+" \
  crates/nexus-core-rs/src/canonical.rs \
  crates/nexus-core-rs/src/schemas/ 2>/dev/null

# Lire le canonical schema pour les structures touchees
cat crates/nexus-core-rs/src/canonical.rs 2>/dev/null | head -50

# Scanner memory pre-launch policy
grep -A 10 "Pre-launch protocol" \
  "$HOME/.claude/projects/C--Users-FlowUP-Documents-Code-nexus/memory/nexus_grid_pivot.md"

# Scanner CLAUDE.md root pre-launch policy
grep -A 10 "Pre-launch protocol policy" CLAUDE.md
```

Pour la phase X, verifier :
- Les structs touches gardent `version = 1` (sauf CVE bloquant) ?
- Pas de tolerant decoder multi-version introduit ?
- `#[serde(default)]` ajoutes sont legitimes (runtime tolerance) et
  documentes inline ?
- DOMAIN_* signatures preservees ?
- D1..D5 Day 0 du sprint courant ne sont PAS rebattues ?
- Decisions actees dans `nexus_grid_pivot.md §Decisions actees` ne
  sont PAS contredites ?

Findings type :
- Phase bumperait `TASK_VERSION 1 -> 2` sans CVE bloquant = invalid
- `#[serde(default)]` ajoute sans rationale runtime tolerance = P2
- Decision Day 0 D3 contredite par implementation proposee =
  escalation user obligatoire (jamais auto-pivot)
- Pre-launch protocol policy violee = invalid

Output Step 5 : liste invariants verifies + violations potentielles,
ou "S4: clean".

### Step 6 — Synthese verdict + emit document

Combiner les 5 scans :

```
S1 = clean | findings
S2 = clean | findings
S3 = clean | findings
S4 = clean | findings
```

**Classifier chaque finding individuel en bloquant vs non-bloquant
AVANT d'agreger** (evite l'ambiguite multi-findings) :

| Scan | Finding bloquant si | Finding non-bloquant si |
|---|---|---|
| **S1a** | `APPROACH-NAIVE` avec evidence OSS (projet mature montre que l'approche du plan est fondamentalement inadaptee) ; `LIB-EXISTS` lib mature + licence compatible couvre le besoin | `APPROACH-ALIGNED` ; `APPROACH-NOVEL` justifie par contexte P2P specifique |
| **S1b** | CVE critical/high affectant crypto/wire/network ; lib bump MAJOR breaking sur API utilisee ; RFC revision avec impact security | CVE low/medium avec mitigation alternative lib documentee ; lib bump PATCH/MINOR semver-stable ; RFC revision non-semantique |
| **S2** | Decision historique documentee + rationale threat-model encore valide + pas de reversion confirmee (cf. Step 3 reverse-commit check) | Decision revertee (reversion confirmee) ; decision sur contexte revolu ; mention indirecte sans rationale explicite |
| **S3** | Regression sur threat T0-T5 couvert actuellement ; pre-requirement HARDENING_ROADMAP §3 ligne S{N} manquant | Gap documente prevu sprint futur (non-regression) ; threat non-adresse mais hors-scope phase courante |
| **S4** | Bump `*_VERSION` pre-launch sans CVE bloquant justificatif ; Day 0 figee contredite par implementation ; pre-launch protocol policy violee | `#[serde(default)]` legitime avec rationale runtime tolerance inline ; wire format unchanged malgre nouveau field optional |

**Regle d'agregation** :
- **>= 1 finding S1a bloquant** (APPROACH-NAIVE ou LIB-EXISTS) → **PLAN-ADAPT**
- **>= 1 finding bloquant S1b/S2/S3/S4** (pas S1a) → **DESIGN-CONFLICT**
- **0 finding bloquant + >= 1 finding non-bloquant** → SCOPE-CUT-CONSISTENT
- **0 finding tout court** → EXECUTE plan-as-is

**PLAN-ADAPT vs DESIGN-CONFLICT** :
- PLAN-ADAPT = la recherche OSS montre une meilleure approche. Le
  plan s'adapte inline (Step 2bis), pas d'arret, pas d'arbitrage
  user. Le code suit l'approche corrigee. Tracabilite dans
  preflight.md §Plan adaptation + commit body.
- DESIGN-CONFLICT = une Day 0 est contredite, ou un threat model
  est viole, ou un wire format pre-launch est bumpe. Ca ne se
  resout PAS par adaptation inline — ca demande un arbitrage user
  explicite sur les options.

Decision tree complet :

```
Aucun finding (S1a+S1b+S2+S3+S4 tous clean) :
  -> verdict EXECUTE plan-as-is
  -> emit .planning/active/sprint{N}_phase_{X}_preflight.md
     **Output condense autorise** (format ci-dessous) :

Finding S1a bloquant (APPROACH-NAIVE ou LIB-EXISTS) :
  -> verdict PLAN-ADAPT
  -> emit sprint{N}_phase_{X}_preflight.md avec §Plan adaptation
     (evidence OSS + approche corrigee + impact fichiers/tests)
  -> proceder code phase avec l'approche corrigee
  -> commit body documente la deviation vs plan original
  -> PAS d'arret, PAS d'arbitrage user (la recherche est l'arbitre)

Findings non-bloquants uniquement :
  -> verdict SCOPE-CUT-CONSISTENT
  -> emit sprint{N}_phase_{X}_preflight.md avec finding documente
     + recommandation S+1 carry-over
  -> proceder code phase normalement
  -> note dans verification.md fail-fast checklist

>= 1 finding bloquant S1b/S2/S3/S4 :
  -> verdict DESIGN-CONFLICT
  -> STOP code ecriture
  -> emit sprint{N}_phase_{X}_pivot_proposal.md avec sections
     obligatoires (cf. template Step 7)
  -> Si multiple bloquants : section §2 Evidence liste CHACUN
     avec son scan source (S1b/S2/S3/S4), pas d'agregation silencieuse
  -> Si 2+ escalations distinctes (ex : CVE bloquant S1b + Day 0
     rebattu S4) : proposal §2 marque "MULTIPLE BLOCKING FINDINGS"
     + chaque Option A/B/C doit adresser ou acknowledger chacun
  -> alerter user avec resume verdict + 3 options
  -> attendre arbitrage user
```

### Step 7 — Templates de documents

#### Template condense (verdict EXECUTE plan-as-is, 5 scans clean)

```markdown
# Sprint {N} Phase {X} — preflight G8

Date : YYYY-MM-DD | HEAD : `<sha>` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : <contrainte pertinente ou N/A>
- <zone-specific>.md : <contrainte ou N/A>

## Scans (all clean)
- S1a OSS prior art : <N> projets recherches (<noms>), APPROACH-ALIGNED — clean
- S1b deps : <N> libs scannees, 0 delta — clean
- S2 historiques : <N> fichiers, <N> commits scannes — clean
- S3 threat model : fast-path verified, HARDENING_ROADMAP aligned — clean
- S4 wire format : fast-path | full / VERSION=1, Day 0 preserved — clean

## Telemetrie preflight
- Duree totale : {mm}m{ss}s
- S1a : {duree} / {N} projets OSS consultes / finding : clean | APPROACH-NAIVE | LIB-EXISTS
- S1b : {duree} / {N} libs scannees / finding : clean | {type}
- S2 : {duree} / {N} commits scannes / finding : clean | {type}
- S3 : fast-path | full / {duree}
- S4 : fast-path | full / {duree}

## Action
Proceder code phase {X}.
```

#### Template PLAN-ADAPT (verdict PLAN-ADAPT, S1a finding bloquant)

```markdown
# Sprint {N} Phase {X} — preflight G8

Date : YYYY-MM-DD | HEAD : `<sha>` | Verdict : **PLAN-ADAPT**

## Memory consultation (Step 1.5)
- feedback_approach.md : <contrainte pertinente>

## S1a — OSS prior art research
- Projets recherches : <list avec URLs>
- context7 queries : <list avec resultats cles>
- Finding : **APPROACH-NAIVE** — <description evidence>
  Plan proposait : <approche plan original>
  OSS montre : <approche mature avec evidence URL>

## Plan adaptation
- Approche corrigee : <description concrete>
- Fichiers impactes vs plan : <delta>
- Tests impactes : <delta>

## Scans S1b/S2/S3/S4
- S1b deps : clean
- S2/S3/S4 : clean (ou findings documentes)

## Telemetrie preflight
- Duree totale : {mm}m{ss}s
- S1a : {duree} / {N} projets OSS consultes / finding : APPROACH-NAIVE
- S1b : {duree} / {N} libs scannees / finding : clean | {type}
- S2 : {duree} / {N} commits scannes
- S3 : fast-path | full / {duree}
- S4 : fast-path | full / {duree}

## Action
Proceder code phase {X} avec approche corrigee. Commit body
documente deviation vs plan §Phase X.
```

#### Template complet (verdict SCOPE-CUT-CONSISTENT ou DESIGN-CONFLICT)

```markdown
# Sprint {N} Phase {X} — preflight G8

Date : YYYY-MM-DD
HEAD : <git rev-parse --short HEAD>
Verdict : EXECUTE plan-as-is | SCOPE-CUT-CONSISTENT

## Memory consultation (Step 1.5)
- feedback_approach.md : <contrainte pertinente>
- <zone-specific>.md : <contrainte ou N/A>
- Tensions plan vs memory : aucune | <description>

## Scans

### S1 — SOTA 2026 vs design
- libs scannes : <list>
- context7 queries : <list avec timestamp>
- WebSearch CVE : <list>
- Verdict : clean | findings

### S2 — Decisions historiques traversees
- git log scan : <command run>
- archive scan : <findings>
- memory feedback scan : <findings>
- Verdict : clean | findings

### S3 — Threat model coverage
- threats mapped T0-T5 : <matrix>
- regression flags : <list>
- HARDENING_ROADMAP gaps : <list>
- Verdict : clean | findings

### S4 — Wire format / pre-launch invariants
- _VERSION fields touches : <list>
- canonical.rs touche : oui/non
- Day 0 preserved : oui/non
- Verdict : clean | findings

## Findings (si SCOPE-CUT-CONSISTENT)

- <finding 1> : carry-over recommande S{N+k}
- <finding 2> : carry-over recommande S{N+k}

## Action

Procede code phase {X}. Carry-over docs ajoutees a
sprint{N+1}_audit_plan.md track approprie (si SCOPE-CUT-CONSISTENT).
```

#### Template `sprint{N}_phase_{X}_pivot_proposal.md` (verdict DESIGN-CONFLICT)

```markdown
# Sprint {N} Phase {X} — pivot proposal G8

Date : YYYY-MM-DD
HEAD : <git rev-parse --short HEAD>
Verdict : DESIGN-CONFLICT (STOP code, attendre arbitrage user)

## 1. Le conflit

Plan §Phase {X} propose : <description courte>

Conflit avec : <S1/S2/S3/S4 + reference precise>

## 2. Evidence factuelle

(REQUIRE >= 1 source factuelle externe verifiable)

- Commit ref : `<sha>` `<sprint{N-k} body extract>`
- CVE : `CVE-YYYY-XXXX` `<NVD URL>`
- RFC : `<RFC ####> §X.Y revision YYYY-MM`
- Context7 query : `<lib-id>` queried YYYY-MM-DD
- Audit report : `<DOI/URL>` published YYYY-MM
- Memory : `feedback_*.md` ligne X "rule + why"

## 3. Options

### Option A — Scope-cut conforme historique

Description : <que livre Phase X reduit, que defer S+1>
Coût : <test delta, fichiers touches>
Bénéfice : <SOTA gap ferme, conforme decision historique>
Invariants preserves : wire format OK | threat model OK | Day 0 OK
Recommandation : default | alternative

### Option B — Adapt minimal

Description : <pivot reduit qui contourne le conflit>
Coût : ...
Bénéfice : ...
Invariants preserves : ...
Recommandation : ...

### Option C — Deep-evolution

Description : <pivot maximal alignement SOTA + threat model>
Coût : ...
Bénéfice : ...
Invariants preserves : ...
Recommandation : ...

## 4. Recommandation default

Option <X> parce que <raison technique chiffree>.

## 5. Garde-fous (cf. README §6.9)

- [ ] Pivot evidence-based (>=1 source externe ci-dessus) ✅
- [ ] Pivot ne rebat pas Day 0 sans escalation ✅ / escalation requise
- [ ] Pivot ne casse pas pre-launch wire ✅ / brisure justifiee CVE
- [ ] Test budget cap respecte (<= 2.5x plan original) ✅ / split
- [ ] Pivot dans theme sprint (kickoff §1) ✅
- [ ] Pivot ferme gap claire (pas YAGNI) ✅
- [ ] Pivot retrospective trackee dans audit_plan S{N} ✅

## 6. Suite

Si pivot accepte (user choisit A, B, ou C) :
1. commit chore(planning) inline qui update plan §Phase X
2. commit feat phase X avec body documentant pivot + ce document
3. nexus-phase-auditor receive dimension "Pivot retrospective" en review

Si user refuse les 3 options et propose Option D (rejeu contraint) :
1. Agent construit **Option D evidence-grounded** depuis le
   feedback user (garde-fou 1 reste obligatoire : Option D doit
   referencer les memes sources externes verifiables §2)
2. Emit `sprint{N}_phase_{X}_pivot_proposal.v2.md` (incremente
   numero version dans le nom) avec Option D + A/B/C conservees
   pour reference + motivation concrete pourquoi D resout ce que
   A/B/C ne resolvent pas
3. User arbitre v2. **Max 1 rejeu** : si user rejette aussi v2,
   default Option A scope-cut conforme + log carry-over explicite
   sprint{N+1}_audit_plan.md "G8 pivot rejected 2x Phase X,
   defer redesign to sprint dedie"
4. Jamais proceder code sans arbitrage accepte OU fallback Option A

Si pivot refuse definitivement (user dit "scope-cut minimal") :
1. proceder Option A (scope-cut conforme)
2. carry-over ajoute sprint{N+1}_audit_plan.md
```

### Step 8 — Garde-fous explicites a verifier

Avant d'emettre `pivot_proposal.md`, verifier les 7 garde-fous
README §6.9 :

1. **Evidence-based** : >=1 source externe verifiable listee §2
2. **Day 0 respect** : si pivot touche D1..D5 → escalation user
   obligatoire signalee dans le proposal (pas de pivot auto)
3. **Wire format** : pivot ne bumpe pas `*_VERSION` avant tag v1.0
   sauf CVE bloquant signe documente
4. **Test budget cap** : pivot test delta < 2.5x plan, sinon
   propose split phase ou carry
5. **Theme sprint** : pivot reste dans la zone fonctionnelle du
   kickoff §1
6. **Pas YAGNI** : si scaffolding pour S+5 sans consumer dans
   roadmap explicite → reject
7. **Retrospective trackee** : note ajouter ligne "Pivot retrospective
   Phase X" dans `sprint{N}_audit_plan.md` track meta-process

Si un garde-fou echoue, rejeter le pivot dans le proposal et
recommander Option A (scope-cut conforme) par defaut.

## Anti-patterns a eviter

1. **Skipper G8 "parce que la phase est triviale"**. Une phase
   trivial n'a normalement aucun finding S1-S4 → verdict CLEAN
   instantane, log 1 ligne. Le cout du scan est < 5 minutes.
   Skipper systematiquement = rater le drift quand il arrive.

2. **Confondre G8 avec G2**. G2 (§6.8) re-valide les artefacts
   long-life au sprint kickoff. G8 (§6.9) re-valide le plan §Phase
   X au pre-implementation. Les 2 sont complementaires, pas
   substituables. G2 ne catch pas les drifts plan-vers-code, G8
   ne catch pas les drifts SOTA depuis kickoff.

3. **Lancer G8 et ignorer le verdict**. Si verdict DESIGN-CONFLICT,
   ne PAS commencer le code en attendant l'arbitrage user. Le
   skill emit le proposal, attend, puis l'agent reprend selon
   l'arbitrage.

4. **Proposer un pivot sans evidence factuelle externe**. "J'ai
   l'impression que X serait mieux" = invalid. Le proposal est
   rejete dans son propre Step 8 garde-fou 1.

5. **Pivot silencieux** : adapter le code sans emettre proposal,
   sans update plan, sans documenter. Casse l'audit gate. Toujours
   emettre proposal + commit chore(planning) update plan AVANT le
   feat commit phase.

6. **Pivot opportuniste** : "tant qu'on touche le module on
   refactor X". Reject — G8 declenche sur DESIGN-CONFLICT factuel
   (S1-S4), pas sur opportunite editeur.

## Refs

- `docs/claude/README.md §6.9` (G8 source-of-truth)
- `docs/claude/README.md §7.1` (bootstrap Cas B integration G8)
- `.claude/skills/nexus-phase-review/SKILL.md` (staging coherence
  pre-commit, complement post-code)
- `.claude/agents/nexus-phase-auditor.md` (audit retrospective
  recoit dimension "Pivot retrospective" si phase a declenche G8)
- memory `feedback_approach.md` (principe pick-deepest dont G8 est
  le mecanisme procedural)
