# Analyse comparative du process workflow S23-S24

Date : 2026-04-22
Perimetre : commits `9628e63` (audit gate S23) a `9351bb4` (S24 Phase F close)
Methode : diff artefacts process (docs/claude/, .claude/skills/, .claude/hooks/, .claude/agents/)

---

## 1. Resume executif

Le process a subi entre S23 et S24 une **phase de maturation par soustraction + ajout cible**.
L'evenement declencheur principal est le post-mortem S24 Phase D (hash binaire sur output LLM
stochastique = structurellement inoperant, non detecte par le G8 4-scan). La reponse process a
ete triple : (1) ajout du scan S1a "OSS prior art" et du verdict PLAN-ADAPT dans le modele G8,
(2) ajout du gate G10 en review, (3) soustraction simultanee des mecanismes prouves inefficaces
(C1-C9 audit conditionnel, G5 working tree audit standalone). Le resultat net est un systeme a
5 scans / 4 verdicts avec ~168 LOC de moins sur README.md (2217 -> 2049), ~188 LOC de moins sur
TOOLING.md (680 -> 492), et +204 LOC sur les skills (889 -> 1169) ou la logique operationnelle
a migre. Les hooks sont restes stables (memes 4 fichiers). La couverture review a bondi de 1/6
phases (S23) a 5/5 phases (S24). Zero DESIGN-CONFLICT emis en 12 preflights cumules S23+S24.

---

## 2. Tableau comparatif par dimension

| Dimension | Etat S23 (audit gate `9628e63`) | Etat S24 (close `9351bb4`) | Delta |
|---|---|---|---|
| **Gates** | G1-G8 (8 gates). G5 = Working tree audit standalone, G8 = 4 scans / 3 verdicts (EXECUTE, SCOPE-CUT-CONSISTENT, DESIGN-CONFLICT) | G1-G4, G6-G10 (9 gates). G5 supprime. G8 = 5 scans / 4 verdicts (+S1a OSS prior art, +PLAN-ADAPT). G9 (factual research D-decisions) documente. G10 (OSS prior art review) ajoute | +G9, +G10, +S1a, +PLAN-ADAPT, -G5, -C1-C9 |
| **Preflight skill** | 500 LOC, 4 scans (S1 SOTA, S2 history, S3 threat, S4 wire), 3 verdicts, pas de memory consultation | 704 LOC (+204), 5 scans (+S1a OSS prior art), 4 verdicts (+PLAN-ADAPT), Step 1.5 memory consultation, routing table zone->memory | +S1a, +PLAN-ADAPT, +memory, +Step 1.5 |
| **Review skill** | 389 LOC, Steps 1-6, pas de Step 2bis coverage, pas de Step 4bis-A OSS check | 465 LOC (+76), +Step 2bis (G9 modified-file branch coverage), +Step 4bis-A (OSS research verification G10), memory consultation integree | +Step 2bis, +Step 4bis-A, +memory |
| **Auditor agent** | 552 LOC, 6 dimensions + 2 steps contexte, pas de focus post-code, pas de timebox explicite, audit profond par defaut | 470 LOC (-82), 8 dimensions post-code (security, patterns, working-tree, G8-traceability, scope-cuts, research-grounding, horizon-LT, tests-delta) + acknowledge preflight S1a-S4 si present, timebox 25/45 min, output < 300 LOC, G4 anti-hallucination renforce | -82 LOC, +focus post-code, +acknowledge G8 |
| **Bootstrap §7** | Cas B cite G8 4 scans + G5 pre-commit, anti-pattern CRAFT/DEBT/NOISE | Cas B cite G8 5 scans + PLAN-ADAPT, reference staging coherence hook, drop vocabulaire G5 CRAFT/DEBT/NOISE standalone | +PLAN-ADAPT routing, -G5 |
| **Commit discipline §4** | Body 7 sections obligatoires, pas de section G8 traceability | Body 8 sections (+G8 traceability : SHA preflight + SHA review + verdict), §4.4 Phase F parse reviews + route P2/P3, review presence guard N/N | +G8 traceability, +§4.4 |
| **Audit gate §3** | Pattern inchange (Phase 0 blocking) | Inchange (CONDITIONAL PASS -> PASS apres fix S23 `34c77ce`) | Stable |
| **Hooks** | 4 scripts : phase-auditor-gate.sh, phase-precommit-lightcheck.sh, session-start-autoinstall.sh, verify-on-write.sh | 4 scripts (memes noms). phase-auditor-gate.sh -123 LOC (drop C1-C9 conditionnel), phase-precommit-lightcheck.sh commentaire orphelin corrige | -123 LOC auditor hook, cosmetic lightcheck |
| **Anti-patterns** | §9 : 6 anti-patterns documentes (9.1-9.6) | §9 inchange + 6 anti-patterns G8 dans §6.9 + anti-patterns auditor G4 (hallucination findings, quota findings) | +~12 anti-patterns |
| **README.md** | 2217 LOC | 2049 LOC (-168) | -7.6% |
| **TOOLING.md** | 680 LOC | 492 LOC (-188) | -27.6% |
| **Skills total** | 889 LOC (500 preflight + 389 review) | 1169 LOC (704 + 465) | +280 LOC (+31.5%) |
| **Agent** | 552 LOC | 470 LOC | -82 LOC (-14.9%) |
| **LOC net process** | 4338 LOC (README+TOOLING+skills+agent) | 4180 LOC | -158 LOC (-3.6%) |

---

## 3. Ameliorations majeures (top 3)

### 3.1 Scan S1a "OSS prior art" + verdict PLAN-ADAPT (`3cf7548`)

**Evidence** : S24 Phase D a shippe du code structurellement inoperant (hash binaire BLAKE3 sur
output LLM stochastique = 100% false positives). Le plan etait suivi rigoureusement — le
probleme etait que le *design* du plan etait faux. BOINC/Truebit montrent que l'approche exacte
ne fonctionne pas. Le G8 4-scan ne challengeait que les versions (S1b) et l'historique (S2),
pas le design (S1a).

**Mecanisme** : S1a recherche "comment les projets OSS matures resolvent ce probleme" AVANT
d'ecrire du code. Le verdict PLAN-ADAPT permet une correction inline sans STOP user (la
recherche est l'arbitre). Le plan.md reste inchange (snapshot kickoff) ; la deviation est
tracee dans preflight.md + commit body.

**Impact** : le modele passe de 4 scans / 3 verdicts a 5 scans / 4 verdicts. S1a est declare
"le scan le plus important" dans README §6.9 (challenge le design, pas juste les deps). Les
5 scans sont documentes comme non-substituables avec matrice d'explication croisee.

### 3.2 Suppression C1-C9 + G5 — soustraction empirique (`466f826`)

**Evidence** : analyse factuelle independante (research doc 450 LOC `S24_process_review_2026-04-21.md`)
sur le process accumule S16-S23. Resultats :
- C1-C9 (audit conditionnel par regex) : faux negatif prouve S23 P1 C-1 (regex C1 ne matchait
  pas `task.rs` -> `redundancy_factor` dans canonical bytes non detecte). Le mecanisme suppose
  detecter les zones sensibles ne les detectait pas.
- G5 Working tree audit standalone : 0 finding autonome en 33 phase reviews S18-S23. Le hook
  `phase-precommit-lightcheck.sh` Check 1 couvrait le seul catch reel (staging coherence).

**Mecanisme** : retour audit inconditionnel pre-commit (phase-auditor-gate.sh -123 LOC bloc
C1-C9). G5 supprime de la table gates. Les LOC ainsi liberees ont finance les ajouts S1a et
PLAN-ADAPT sans inflation nette.

**Impact** : -168 LOC README, -188 LOC TOOLING, -123 LOC hook. Le process est plus court a
lire, les gates restants ont tous une preuve empirique de valeur.

### 3.3 Couverture review 1/6 -> 5/5 + guard mecaniste (`609a66b`)

**Evidence** : S23 n'a produit que 1 review sur 6 phases (Phase E uniquement). L'audit gate
S24 Phase 0 n'a pas detecte ce gap (pas de check). En S24, 5/5 phases feat (A-E) ont chacune
un review PASS. La Phase F (docs-only) a un preflight EXECUTE mais pas de review (coherent
avec le process).

**Mecanisme** : README §4.4 step 5 ajoute un checklist item "Phase review files present: N/N"
dans sprint{N}_audit_plan.md. Ratio < N/N = P2. Le guard est mecaniste (count de fichiers),
pas subjectif.

**Impact** : le gap "reviews manquants non-detectes" identifie sur S23 est ferme structurellement.
L'auditeur Phase 0 S25 ne pourra pas passer a cote d'un sprint avec couverture review
partielle.

---

## 4. Regressions ou dettes process

### 4.1 PLAN-ADAPT sans donnee empirique (0 run historique)

Le verdict PLAN-ADAPT a ete ajoute post-S24 Phase D (`3cf7548`) mais n'a ete execute par
aucune phase de S24 (les phases E et F post-ajout ont emis EXECUTE). Il est donc **non teste
en conditions reelles**. Le S1a n'a egalement aucun run historique (README §6.9 le note :
"S1a (ajoute S24) : 0 run historique").

**Risque** : le mecanisme est theoriquement sain mais sa premiere execution reelle sera en S25,
potentiellement avec des edge cases non anticipes (ex: que se passe-t-il si S1a et S2
produisent des findings contradictoires ? S1a dit APPROACH-NAIVE, S2 dit la decision
historique rejettait l'alternative OSS ?).

### 4.2 Inflation LOC skills (+31.5%) vs deflation docs (-7.6%)

La logique operationnelle migre des docs generales (README/TOOLING) vers les skills specifiques.
C'est un mouvement sain (la logique executable est pres de son point d'invocation) mais cree
un risque de **divergence docs ↔ skills** si une mise a jour touche l'un sans l'autre. Le
README §6.9 et le skill SKILL.md preflight documentent tous les deux le modele 5-scan — un
changement futur devra toucher les deux.

### 4.3 S3+S4 fast-path non formalise dans le skill

README §6.9 note "S3+S4 fast-path grep only" base sur 0 finding en 17 runs. Mais le skill
SKILL.md preflight Steps 4/5 (S3/S4) n'ont pas ete simplifies en consequence — ils decrivent
toujours la procedure complete. Le fast-path est documente dans README mais pas reflete dans
le skill executable.

### 4.4 Aucun anti-pattern PLAN-ADAPT documente

Les anti-patterns §6.9 couvrent le pivot DESIGN-CONFLICT (opportuniste, silencieux, dilue,
repetitif). Aucun anti-pattern ne couvre PLAN-ADAPT (ex: adapter excessivement sans evidence
suffisante, confondre "APPROACH-ALIGNED with minor improvement" avec APPROACH-NAIVE, etc.).

**Corrige** : 3 anti-patterns PLAN-ADAPT ajoutes dans README §6.9 (evidence OSS concrete,
Day-0 figees = DESIGN-CONFLICT, repetition = signal meta).

### 4.5 (P1) Perte detection wire-format hooks post-C1-C9

L'ancien hook `phase-auditor-gate.sh` contenait un bloc C1-C9 avec detection `TOUCHES_WIRE`
sur `canonical.rs` + `schemas/`. Supprime `466f826`. Le hook actuel n'a plus aucune regex
wire-format. Combine avec S4 preflight en "fast-path grep-only" (§4.3), une modification
subtile de `canonical.rs` pouvait passer sans alerte.

**Corrige** : Check 4 "wire-format staging alert" ajoute dans `phase-precommit-lightcheck.sh`
+ clause d'escalation S4 full scan dans skill preflight Step 5.

### 4.6 (P2) Surveillance DEBT/NOISE sans mecanisme dedie

`.gitignore` auto sur NOISE et grep scope-cuts sur DEBT etaient implicitement couverts par
G5 working tree audit. Apres suppression G5, ces checks dependent de la discipline de
l'executeur et de l'auditeur Phase 0. Pas de regression fonctionnelle immediate (le hook
lightcheck Check 1 couvre le staging) mais la categorisation explicite n'est plus forcee.

### 4.7 (P2) Dimension count auditor sous-estime dans §2

Le §2 original disait "4 dimensions post-code (security+patterns+scope-cuts+tests)".
L'auditor documente en realite 8 dimensions dans ses Steps 2-5 : security, patterns,
working-tree (G5 residuel), G8-traceability, scope-cuts, research-grounding, horizon-LT,
tests-delta. La sous-estimation masquait le scope reel de l'audit.

**Corrige** : tableau §2 mis a jour avec "8 dimensions".

### 4.8 (P3) G9 documentaire sans enforcement mecanique

G9 (§6.10) est un gate "factual research AVANT D-choice kickoff" documente dans README mais
sans skill ni hook qui l'enforce. Le skill review Step 2bis/G9 couvre "modified-file branch
coverage", pas "D-research quality". L'enforcement repose entierement sur G1 (Design Review
Board) qui verifie la presence de sources dans les D-choices.

---

## 5. Recommandations S25+ (max 5, priorisees)

### R1 (P1) — Tester PLAN-ADAPT en conditions reelles des la premiere phase S25 non-triviale

S1a/PLAN-ADAPT n'ont aucun run empirique. Recommandation : sur la premiere phase S25 qui
touche un domaine avec des projets OSS de reference (compute verification, safety, P2P),
forcer l'execution complete de S1a meme si le design semble aligne, pour valider le
mecanisme et identifier les edge cases.

### R2 (P2) — Aligner le skill preflight Steps 4/5 sur le fast-path S3/S4

Le README documente le fast-path mais le skill ne l'implemente pas. Cela cree deux sources
de verite. Recommandation : simplifier Steps 4/5 du skill en "grep-only gate check" avec
sortie conditionnelle vers full-scan si finding.

### R3 (P2) — Documenter les anti-patterns PLAN-ADAPT dans §6.9

Le verdict PLAN-ADAPT permet une correction inline sans arbitrage user. Sans garde-fous
specifiques documentes, il pourrait devenir un vecteur de scope creep subtil (l'agent
adapte le design en pretextant une recherche OSS superficielle). Recommandation : ajouter
2-3 anti-patterns PLAN-ADAPT (ex: "adapt sans evidence OSS concrete = invalid",
"adapt qui touche des Day-0 figees = DESIGN-CONFLICT, pas PLAN-ADAPT").

### R4 (P3) — Single source of truth skill ↔ docs

Le modele 5-scan est documente dans README §6.9 ET dans skill SKILL.md. Pour eviter la
divergence, envisager de faire du skill la source de verite operationnelle et du README
un pointeur avec resume. Cela reduirait README §6.9 de ~200 LOC a ~30 LOC (description +
table + renvoi skill).

### R5 (P3) — Metriques automatisees de couverture process

Le guard "review files present: N/N" est un bon precedent. Etendre le pattern a d'autres
metriques mecanistes : "preflights present: N/N", "G8 verdicts distribution", "delta tests
annonce vs reel par phase". Un script `.claude/hooks/` ou un check Phase F pourrait
automatiser la collecte.

### R6 (P1) — Auditer couverture hooks post-C1-C9 (ajoute post-review agents)

Tester que le pipeline hooks+preflight couvre les vrais positifs historiques. Cas test :
S23 `redundancy_factor` dans canonical bytes (`34c77ce`). Verifier que Check 4 wire-format
alert + S4 full scan escalation detectent ce pattern. Si gap residuel, renforcer le grep
dans Check 4 ou ajouter une dimension S4 dans le preflight.

**Partiellement adresse** : Check 4 + escalation S4 ajoutes dans cette session. Validation
empirique a faire sur premiere phase S25 touchant wire-format.

### R7 (P2) — Coherence inter-skills memory consultation (ajoute post-review agents)

Les Step 1.5 des skills preflight et review avaient des formulations differentes pour la
meme routing table memory. Risque de divergence silencieuse lors de futures mises a jour.

**Adresse** : routing table unifiee mot-pour-mot dans les 2 skills avec instruction de
synchronisation ("grep Routing table dans .claude/skills/*/SKILL.md").

### R8 (P2) — Telemetrie preflight pour mesurer le cout reel (ajoute post-review agents)

Zero mesure du temps d'execution des preflights. Le cout cognitif de +10 steps aux skills
(S1a, Step 1.5, etc.) n'est pas quantifie. Sans telemetrie, impossible de savoir si le
process S24 est plus efficace ou juste plus lourd.

**Partiellement adresse** : section §Telemetrie ajoutee dans les templates preflight.md.
La collecte des donnees commencera en S25.

---

## 6. Annexe : liste complete commits process S23-S24

| SHA | Description |
|---|---|
| `466f826` | S24 process review — drop C1-C9 + G5, compact auditor, stabilize cycle |
| `af573c4` | Cleanup prompt §7 — drop CRAFT/DEBT/NOISE G5 vocabulary, deduplicate autonomy rules |
| `7b00475` | Fix 6 stale refs in README + skill — sprint count, G5 ghost, memory count, hook ref, Co-Authored-By template |
| `91589ea` | Batch cleanup — drop dead TOOLING §6-7 (hooks supprimes), fix session-start orphan, fix review template, fix test counts |
| `6cb6e72` | Open S24 — kickoff + plan + migrate S23 archive + G1 design review |
| `ff4c7d5` | Phase A — P2 cleanup batch S23 audit + PATTERNS §P35 ephemeral + §P36 redundancy |
| `609a66b` | Consolidate G8/audit data S22-S24 — empirical annotations + fast-path S3/S4 + review presence guard |
| `1f027ff` | Integrate explicit memory consultation into preflight + review skills |
| `da67567` | Add Step 2bis G9 modified-file branch coverage check to nexus-phase-review |
| `3cf7548` | G10 OSS prior art research + PLAN-ADAPT verdict + adaptive planning |
| `bd132fa` | Align README §6.9 with G10 OSS prior art + PLAN-ADAPT + 5-scan model |
| `1fcf7ec` | Align bootstrap §7.1 Cas B with 5-scan / 4-verdict G8 model |
| `9b2686c` | S24 Phase E — G8 preflight verdict EXECUTE (DNS-based DHT fallback DoH+DoT) |
| `72bc0b1` | S24 Phase E — review verdict PASS (2 P2 findings carry S25) |
| `1feb2a9` | S24 Phase F — G8 preflight verdict EXECUTE (wrap-up docs/planning only) |
| `9351bb4` | Phase F — wrap-up + verification + audit plan S25 + migration planning archive/v1.2/ |
| `34c77ce` | fix(sprint23): exclude redundancy_factor from canonical bytes (R3 mitigation) |

Total : 17 commits process (10 `chore(claude|skill|planning)`, 2 `chore(planning)` preflight, 2 `chore(planning)` review, 1 `fix(sprint23)`, 1 feat Phase A cleanup, 1 chore sprint close).

---

## 7. Annexe : metriques empiriques G8 et reviews

### G8 preflights

| Sprint | Phase | Verdict | Findings |
|---|---|---|---|
| S23 | A | EXECUTE | S1-S4 clean |
| S23 | B | EXECUTE | S1-S4 clean |
| S23 | C | EXECUTE | S1-S4 clean |
| S23 | D | EXECUTE | S1-S4 clean |
| S23 | E | SCOPE-CUT-CONSISTENT | S1 CVE-2025-69277 pynacl (non-bloquant, code path non-affecte) |
| S23 | F | EXECUTE | S1-S4 clean |
| S24 | A | EXECUTE | S1-S4 clean |
| S24 | B | EXECUTE | S1-S4 clean |
| S24 | C | EXECUTE | S1-S4 clean |
| S24 | D | EXECUTE | S1-S4 clean (S1a non encore ajoute — aurait detecte APPROACH-NAIVE) |
| S24 | E | EXECUTE | S1-S4 clean (S1a ajoute mais 0 finding) |
| S24 | F | EXECUTE | S1-S4 clean |

Distribution : 11 EXECUTE, 1 SCOPE-CUT-CONSISTENT, 0 PLAN-ADAPT, 0 DESIGN-CONFLICT.

### Phase reviews

| Sprint | Phases feat | Reviews produits | Ratio |
|---|---|---|---|
| S23 | 6 (A-F) | 1 (Phase E) | 1/6 = 17% |
| S24 | 5 (A-E) | 5 (A-E) | 5/5 = 100% |

Tous les verdicts S24 : PASS.

### Audit gate

| Sprint audite | Verdict initial | Verdict final |
|---|---|---|
| S22 (audite en S23 Phase 0) | CONDITIONAL PASS (1 P1 `redundancy_factor`) | PASS apres fix `34c77ce` |
| S23 (audite en S24 Phase 0) | PASS (apres cleanup Phase A `ff4c7d5`) | PASS |
