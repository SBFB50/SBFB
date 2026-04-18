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

## Quand l'utiliser

- Tu es en Cas B (sprint en cours) et tu t'apprete a ecrire le code
  d'une phase A/B/C/D/E/F
- AVANT le premier `Edit` ou `Write` outil sur du code de la phase
- Avant les commits chore(planning) qui split CRAFT — non, ceux-la
  passent par `nexus-phase-review` Step 1bis G5 (post-code time)
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
4. Lire `docs/claude/README.md §6.9` pour la procedure verdict

### Step 2 — Scan S1 : SOTA 2026 vs design

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

Findings type :
- `lib X v Y.Z` — major bump publie depuis plan, breaking changes
- `RFC W` — section X revisee Aug 2026, change semantique
- `CVE-2026-XXXX` critical sur dep transitive Z
- API deprecated, remplacement = nouvelle methode

Output Step 2 : liste de findings ou "S1: clean" si aucun delta.

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

Output Step 3 : liste des decisions historiques + sha + raison, ou
"S2: clean".

### Step 4 — Scan S3 : Threat model coverage

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

### Step 5 — Scan S4 : Wire format / pre-launch invariants

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

Combiner les 4 scans :

```
S1 = clean | findings
S2 = clean | findings
S3 = clean | findings
S4 = clean | findings
```

Decision tree (cf. README.md §6.9) :

```
Tous clean :
  -> verdict EXECUTE plan-as-is
  -> emit .planning/active/sprint{N}_phase_{X}_preflight.md (1-3 lignes)

Findings non-bloquants (sub-optimal selon SOTA mais plan reste
executable, decisions historiques pas contredites, threat model OK,
wire format OK) :
  -> verdict SCOPE-CUT-CONSISTENT
  -> emit sprint{N}_phase_{X}_preflight.md avec finding documente
     + recommandation S+1 carry-over
  -> proceder code phase normalement
  -> note dans verification.md fail-fast checklist

Findings bloquants (DESIGN-CONFLICT) — au moins UN parmi :
  - S1 : CVE bloquant sur dep crypto critique
  - S2 : plan contredit decision documentee historique avec rationale
    threat-model encore valide
  - S3 : phase introduirait regression sur threat couvert ailleurs
  - S4 : phase casserait wire format pre-launch sans CVE bloquant
        OU contredirait Day 0 figee
:
  -> verdict DESIGN-CONFLICT
  -> STOP code ecriture
  -> emit sprint{N}_phase_{X}_pivot_proposal.md avec sections
     obligatoires (cf. template Step 7)
  -> alerter user avec resume verdict + 3 options
  -> attendre arbitrage user
```

### Step 7 — Templates de documents

#### Template `sprint{N}_phase_{X}_preflight.md` (verdict CLEAN ou SCOPE-CUT-CONSISTENT)

```markdown
# Sprint {N} Phase {X} — preflight G8

Date : YYYY-MM-DD
HEAD : <git rev-parse --short HEAD>
Verdict : EXECUTE plan-as-is | SCOPE-CUT-CONSISTENT

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

Si pivot accepte :
1. commit chore(planning) inline qui update plan §Phase X
2. commit feat phase X avec body documentant pivot + ce document
3. nexus-phase-auditor receive dimension "Pivot retrospective" en review

Si pivot refuse :
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
- `.claude/skills/nexus-phase-review/SKILL.md` (G5 working tree
  audit pre-commit, complement post-code)
- `.claude/agents/nexus-phase-auditor.md` (audit retrospective
  recoit dimension "Pivot retrospective" si phase a declenche G8)
- memory `feedback_approach.md` (principe pick-deepest dont G8 est
  le mecanisme procedural)
