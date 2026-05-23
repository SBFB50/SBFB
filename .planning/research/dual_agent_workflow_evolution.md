# Dual-Agent Workflow Evolution — Claude Teams + Codex GPT 5.5

**Date :** 2026-05-18
**Auteur :** Recherche agent Claude Opus 4.6 (1M context)
**Objectif :** Documenter les modifications exactes a apporter a
`docs/claude/README.md` pour integrer le processus dual-agent
(Claude teams ultra-profonds + Codex CLI GPT 5.5 verification).

**Mode d'emploi :** Chaque section ci-dessous contient le texte
copier-collable + les marqueurs `old_string` / `new_string` pour
localiser l'insertion ou le remplacement dans README.md.

---

## 1. Nouvelle section 4.5 — Dual-Agent Verification Process

**Emplacement :** apres la section 4.4 (Phase F wrap-up) et avant
la section 5 (Memory system externe). La section 4.4 se termine
par la ligne :

```
orphelins (probable parsing oublié). Exception acceptable : review.md
avec verdict PASS + 0 finding explicite.
```

**Insertion :** ajouter la section suivante apres le `---` qui
ferme la section 4.4 et avant le `## 5. Memory system externe`.

### old_string (marqueur de position)

```
---

## 5. Memory system externe
```

### new_string (remplacement)

```
### 4.5 Dual-Agent Verification Process — Claude Teams + Codex GPT 5.5

Depuis Sprint 65, chaque phase de chaque sprint est verifiee par un
processus dual-agent : equipe d'agents Claude ultra-profonds (Opus
4.6, 1M tokens chacun) pour le preflight, l'execution et la review,
puis verification croisee independante par Codex CLI (GPT 5.5).

Ce processus ajoute une couche de verification que ni l'auto-
attestation (verification.md) ni l'audit gate intra-sprint (Phase 0)
ne couvrent : une review par un modele fondamentalement different,
sans partage de contexte avec l'executeur, sur le code reel commite.

#### 4.5.1 Cycle de vie d'une phase avec dual-agent

```
Plan section Phase X lue
  |
  v
Claude team preflight (skill nexus-phase-preflight + agents 1M)
  |  5 scans factuels S1a-S4 en parallele
  v
Code ecrit par Claude (execution phase standard)
  |
  v
Claude team phase review (skill nexus-phase-review + agent auditor)
  |  6 dimensions, verdict PASS/CONCERN/FAIL
  v
Codex verification (codex exec, prompt structure, findings)
  |  Review croisee independante GPT 5.5
  v
Claude correction loop (si Codex trouve des issues)
  |  Fix + optionnel re-run Codex si > 10 LOC modifies
  v
Commit atomique feat(scope): Sprint N Phase X
```

Le preflight et la review Claude restent les gates primaires
(G8 + G4). Codex est une verification supplementaire — il ne
remplace ni le preflight, ni la review, ni l'audit gate.

#### 4.5.2 Lancer Codex depuis Claude Code — pattern valide

**Pattern qui fonctionne** (teste et valide) :

```powershell
# 1. Ecrire le prompt dans un fichier texte
#    (Write tool en contexte agent, ou editeur)
#    Chemin standard : .git/CODEX_SPRINT{N}_PHASE_{X}.txt
#    (inclure le sprint evite d'ecraser le prompt d'un autre sprint)
#    Helper canonique :
#    python scripts/agent/agentctl.py codex-prompt-path --sprint {N} --phase {X}

# 2. Pipe via stdin vers codex exec
Get-Content ".git/CODEX_SPRINT{N}_PHASE_{X}.txt" -Raw | codex exec `
  --dangerously-bypass-approvals-and-sandbox `
  -o ".planning/active/sprint{N}_phase_{X}_codex_review.md"
```

**Parametres obligatoires :**

| Parametre | Role |
|-----------|------|
| `--dangerously-bypass-approvals-and-sandbox` | Execution sans approbation interactive (equivalent de `--yolo`) |
| `-o fichier.md` | Ecrit l'output dans un fichier lisible par Claude apres execution |

**Anti-patterns testes et echoues — NE PAS reproduire :**

| Anti-pattern | Symptome | Pourquoi |
|---|---|---|
| `-m o3` | Erreur "model not available" | Compte ChatGPT, pas API — utiliser le default GPT 5.5 |
| Here-string PowerShell direct (`@" ... "@`) | Parsing errors sur apostrophes francaises | PowerShell interprete les guillemets internes |
| Prompt inline en argument (`codex exec "..."`) | Codex attend stdin quand pas d'argument positional | Le prompt doit passer par stdin ou fichier |
| Prompt sans `-o` | Output affiche en console, pas recuperable par Claude | Toujours `-o fichier.md` pour lecture post-exec |
| Prompt trop court (<10 lignes) | Review superficielle, faux positifs | Le prompt doit lister explicitement chaque livrable |

#### 4.5.3 Template de prompt Codex — verification phase

Ce template est a ecrire dans `.git/CODEX_SPRINT{N}_PHASE_{X}.txt` avant
de lancer `codex exec`. Adapter les placeholders `{...}` a la
phase en cours.
Le chemin doit etre obtenu si possible via
`python scripts/agent/agentctl.py codex-prompt-path --sprint {N} --phase {X}`.

```
Tu es un auditeur independant. Tu ne connais PAS l'historique de
cette session. Tu verifies le code source du projet nexus-grid
(SBFB) apres une phase de sprint.

Sprint : {N}
Phase : {X} — {titre court}
Branch : master

## Livrables attendus de cette phase

{Copier-coller les livrables depuis le plan.md section Phase X,
verbatim. Ne PAS resumer — lister chaque item.}

Exemple :
1. Fix P2-FEED-INSERT-NO-AUTH-TIER : check auth tier dans feed_insert() handler
2. Migration FeedEntry.op vers serde_json::Value
3. Tests : unknown op roundtrip, canonical bytes, auth tier reject
4. TRUST_TAXONOMY.md documente (6 niveaux)
...

## Ta mission

Pour CHAQUE livrable ci-dessus :

1. Cherche dans le code source les fichiers concernes.
2. Verifie que le livrable est REELLEMENT implemente (pas juste
   un TODO, un stub, ou un test sans assertion).
3. Cite le fichier et les numeros de ligne exacts.
4. Conclus : CONFIRME (avec evidence) ou GAP (avec description
   de ce qui manque).

## Format de reponse

Pour chaque livrable :

### Livrable N : {titre}
- Statut : CONFIRME | GAP | PARTIEL
- Fichier(s) : {chemin:ligne}
- Evidence : {extrait code 3-5 lignes}
- Si GAP : estimation en nombre de lignes du fix manquant

## Resume final

- Total livrables : N
- Confirmes : N
- Gaps : N
- Partiels : N
- Estimation totale LOC fixes manquants : N

## Contraintes

- Reponse en francais.
- Pas d'opinion sur l'architecture — tu verifies, tu ne redesigns pas.
- Si un test existe mais n'a pas d'assertion utile, c'est un GAP.
- Si un fichier documente est mentionne mais n'existe pas, c'est un GAP.
```

#### 4.5.4 Template de prompt Codex — verification preflight G8

Ce second template verifie que le preflight G8 a ete correctement
execute (les 5 scans factuels). A utiliser quand le PO veut une
assurance supplementaire sur la qualite du preflight.

```
Tu es un auditeur independant. Tu verifies qu'un preflight G8
(gate pre-implementation) a ete correctement execute sur le
projet nexus-grid (SBFB).

Sprint : {N}
Phase : {X}

## Fichier preflight a auditer

Lis le fichier :
.planning/active/sprint{N}_phase_{X}_preflight.md

## Verification requise

Pour chaque scan du preflight :

### S1a — OSS prior art
- Le preflight cite-t-il au moins 1 projet OSS de reference ?
- L'approche du plan est-elle comparee au SOTA ?
- Verdict S1a present et justifie ?

### S1b — Deps/libs versions
- Les libs critiques sont-elles scannees (version + CVE) ?
- context7 ou WebSearch traces ?

### S2 — Decisions historiques
- git log scan effectue sur les fichiers cibles ?
- Reverse-commit check documente si finding ?

### S3 — Threat model
- Fast-path justifie ou full scan si composant securite ?
- HARDENING_ROADMAP consulte ?

### S4 — Wire format
- *_VERSION fields verifies ?
- Day 0 preservees ?

## Format de reponse

| Scan | Present | Complet | Evidence |
|------|---------|---------|----------|
| S1a  | OUI/NON | OUI/PARTIEL/NON | {details} |
| S1b  | ...     | ...     | ... |
| S2   | ...     | ...     | ... |
| S3   | ...     | ...     | ... |
| S4   | ...     | ...     | ... |

## Verdict global

- PASS : 5/5 scans presents et complets
- CONCERN : 1+ scan partiel mais pas de gap critique
- FAIL : scan manquant ou finding bloquant non detecte

Reponse en francais.
```

#### 4.5.5 Cycle de correction post-Codex

Quand Codex produit des findings :

1. **Claude lit le rapport Codex** :
   ```powershell
   # Le rapport est dans le fichier -o specifie
   # Claude le lit via Read tool
   ```

2. **Triage des findings** : chaque finding Codex est classifie :
   - **GAP confirme** : le livrable est reellement manquant ou
     incomplet dans le code. Claude corrige.
   - **Faux positif** : Codex n'a pas vu le code (mauvais fichier,
     mauvais chemin, code present mais dans un autre module). Claude
     documente le faux positif dans le commit body.
   - **GAP cosmetic** : le livrable est present mais la doc ou un
     commentaire manque. Claude corrige si < 5 LOC, sinon carry P3.

3. **Correction par Claude** : l'agent Claude corrige chaque GAP
   confirme.

4. **Re-run Codex conditionnel** : si la correction est significative
   (> 10 LOC de code metier modifie, pas juste des commentaires ou
   de la doc), relancer Codex sur les fichiers modifies uniquement :
   ```powershell
   # Prompt cible sur les fichiers corriges
   Get-Content ".git/CODEX_SPRINT{N}_PHASE_{X}_RECHECK_01.txt" -Raw | codex exec `
     --dangerously-bypass-approvals-and-sandbox `
     -o ".planning/active/sprint{N}_phase_{X}_codex_recheck.md"
   ```
   Le prompt de recheck liste seulement les GAPs corriges + les
   fichiers modifies, pas toute la phase.

5. **Si Codex confirme OK** : commit atomique phase standard.

6. **Tracabilite** : le commit body de la phase inclut une section
   `## Codex verification` :
   ```
   ## Codex verification
   - Review : sprint{N}_phase_{X}_codex_review.md
   - Findings : N gaps / N faux positifs
   - Recheck : oui/non (si > 10 LOC corriges)
   - Verdict final : PASS
   ```

#### 4.5.6 Quand NE PAS lancer Codex

| Cas | Raison |
|-----|--------|
| Phase purement docs (TRUST_TAXONOMY.md, COMMONS.md, FACTORY_GATES.md) | Pas de code a verifier |
| Phase cosmetique (< 5 LOC de code metier) | Cout disproportionne |
| Hotfix cas D hors sprint | Urgence, pas de plan §Phase X a verifier |
| PO dit "skip codex" | Decision explicite documentee dans le commit body |
| Phase de sortie (verification.md + audit_plan.md) | Pas de code metier, c'est de la doc planning |

Dans ces cas, documenter dans le commit body :
`## Codex verification : skipped ({raison})`.

#### 4.5.7 Parallelisation Claude teams

Les agents Claude ultra-profonds (Opus 4.6, 1M tokens) travaillent
en parallele pour les operations qui ne dependent pas les unes des
autres :

**Preflight G8 (5 scans) :**
Les 5 scans S1a / S1b / S2 / S3 / S4 sont lances en parallele
via des agents independants (chacun 1M tokens). Chaque agent
produit son output, l'agent orchestrateur agrege les resultats
et emit le verdict. Gain mesure : ~3x plus rapide qu'un scan
sequentiel sur les phases avec > 10 fichiers cibles.

```
Agent S1a (OSS prior art)    ──┐
Agent S1b (deps/libs)        ──┤
Agent S2  (historiques)      ──┼──→ Orchestrateur → verdict
Agent S3  (threat model)     ──┤
Agent S4  (wire format)      ──┘
```

**Review phase (6 dimensions) :**
Les dimensions sont partiellement parallelisables :
- Groupe 1 (independant) : Security + Scope-cuts + G8 traceability
- Groupe 2 (sequentiel apres suites) : Tests-delta + Research-grounding + Horizon long-terme

Le groupe 1 peut demarrer pendant que les suites §7.4 tournent
(en `run_in_background`). Le groupe 2 attend les resultats des
suites pour valider le delta tests.

---

## 5. Memory system externe
```

---

## 2. Modification section 7.1 — Prompt bootstrap avec dual-agent

**Emplacement :** Dans le bloc de code du prompt bootstrap section
7.1, a l'interieur du `Cas B — Sprint en cours`. Le texte actuel
du Cas B se termine par les instructions sur `nexus-phase-review`
et `G7 carry-overs`.

### old_string (dans le bloc code du prompt 7.1, section Cas B)

```
    Avant CHAQUE commit phase : invoquer skill
               nexus-phase-review Step 1bis "staging coherence"
               -> verifier staging coherent (hook lightcheck le fait
               automatiquement). Separer chore(planning) si docs
               planning modifies hors-phase.
    Avant scope cut S+1 (G7) : verifier compteur reports de
               chaque carry. Items a 3 reports = obligatoire sprint
               suivant (§6.2.1 Regle 2). Items < 500 LOC ne peuvent
               pas etre reclassifies long-term.
```

### new_string

```
    Avant CHAQUE commit phase : invoquer skill
               nexus-phase-review Step 1bis "staging coherence"
               -> verifier staging coherent (hook lightcheck le fait
               automatiquement). Separer chore(planning) si docs
               planning modifies hors-phase.
    Apres review Claude, AVANT commit : Codex verification
               croisee (§4.5). Sauf si phase docs-only, < 5 LOC,
               hotfix cas D, ou PO dit "skip codex". Procedure :
               1. Ecrire prompt Codex dans .git/CODEX_SPRINT{N}_PHASE_{X}.txt
                  (template §4.5.3, adapter livrables depuis plan)
               2. Lancer :
                  Get-Content ".git/CODEX_SPRINT{N}_PHASE_{X}.txt" -Raw | codex exec `
                    --dangerously-bypass-approvals-and-sandbox `
                    -o ".planning/active/sprint{N}_phase_{X}_codex_review.md"
               3. Lire le rapport, trier GAPs vs faux positifs
               4. Corriger les GAPs confirmes
               5. Si > 10 LOC corriges : re-run Codex cible
               6. Ajouter section ## Codex verification au commit body
    Avant scope cut S+1 (G7) : verifier compteur reports de
               chaque carry. Items a 3 reports = obligatoire sprint
               suivant (§6.2.1 Regle 2). Items < 500 LOC ne peuvent
               pas etre reclassifies long-term.
```

---

## 3. Modification section 6.9 — G8 avec agents ultra-profonds en parallele

**Emplacement :** Dans la section 6.9, sous-section "Les 5 scans
factuels". Le paragraphe actuel decrit les 5 scans dans un tableau
mais ne mentionne pas la parallelisation.

### old_string (apres le tableau des 5 scans)

```
Les 5 scans sont **non-substituables**. S1a sans S1b = on a le bon
design mais sur une lib obsolète. S1b sans S1a = on a la bonne lib
mais le mauvais design (S24 Phase D : BLAKE3 à jour mais hash binaire
sur output stochastique = inopérant). S2 sans S3 = cohérent
historiquement mais gap threat model. S3 sans S4 = durci mais wire
cassé. S4 sans S1 = invariants préservés sur approche obsolète.
```

### new_string

```
Les 5 scans sont **non-substituables**. S1a sans S1b = on a le bon
design mais sur une lib obsolète. S1b sans S1a = on a la bonne lib
mais le mauvais design (S24 Phase D : BLAKE3 à jour mais hash binaire
sur output stochastique = inopérant). S2 sans S3 = cohérent
historiquement mais gap threat model. S3 sans S4 = durci mais wire
cassé. S4 sans S1 = invariants préservés sur approche obsolète.

**Parallelisation via agents ultra-profonds (depuis S65).** Les 5
scans sont independants en entree (chacun lit des sources differentes)
et ne dependent que de Step 1 (identification contexte) qui est
partage. Lancer les 5 scans en parallele via des agents Claude
independants (Opus 4.6, 1M tokens chacun) :

```
# Lancement parallele des 5 scans G8
Agent 1 : S1a OSS prior art (WebSearch + context7)
Agent 2 : S1b deps/libs versions (context7 + CVE scan)
Agent 3 : S2 decisions historiques (git log + grep archives)
Agent 4 : S3 threat model (grep THREAT_MODEL + HARDENING)
Agent 5 : S4 wire format (grep canonical + _VERSION + Day 0)

# Agregation par l'orchestrateur
Orchestrateur : collecter outputs → classifier findings →
                emettre verdict EXECUTE / PLAN-ADAPT /
                SCOPE-CUT-CONSISTENT / DESIGN-CONFLICT
```

**Gain mesure** : phases avec > 10 fichiers cibles passent de
~15 min sequentiel a ~5 min parallele. Phases simples (< 5
fichiers) : gain marginal, les scans S3/S4 fast-path finissent
en secondes de toute facon.

**Prerequis** : chaque agent recoit en input le Step 1 complet
(plan §Phase X, fichiers cibles, libs, APIs, wire format) mais
ne lit que les sources de son scan. L'orchestrateur ne lance PAS
le Step 6 (synthese verdict) tant que les 5 agents n'ont pas
retourne leur output.
```
```

---

## 4. Modification section 4.3 — Verification avec Codex

**Emplacement :** Dans la section 4.3 "Verification obligatoire
avant commit", apres le bloc de verification finale. Le texte
actuel se termine par :

```
Tout rouge bloque le commit. Aucune exception « je commit et
je fix après » — le fix doit être dans le même commit ou
déclenche un nouveau cycle.
```

### old_string

```
Tout rouge bloque le commit. Aucune exception « je commit et
je fix après » — le fix doit être dans le même commit ou
déclenche un nouveau cycle.
```

### new_string

```
Tout rouge bloque le commit. Aucune exception « je commit et
je fix après » — le fix doit être dans le même commit ou
déclenche un nouveau cycle.

**Verification croisee Codex (depuis S65, cf. §4.5).** Apres que
toutes les suites sont vertes et que la review Claude (skill
nexus-phase-review + agent nexus-phase-auditor) a produit un
verdict PASS, lancer la verification Codex GPT 5.5 (sauf phases
exemptees §4.5.6). Le rapport Codex est un artefact supplementaire :
il ne bloque pas le commit si le verdict Claude est PASS, mais
tout GAP confirme par Codex doit etre corrige ou documente comme
faux positif dans le commit body avant de commiter.

Sequence complete avant commit phase :

```
1. Suites §7.4 vertes (Rust + Frontend)
2. Review Claude (skill nexus-phase-review) — verdict PASS
3. Agent nexus-phase-auditor — verdict PASS
4. Codex verification croisee (§4.5) — GAPs corriges ou documentes
5. Commit atomique
```
```

---

## 5. Modification section 12 — Evolution du systeme

**Emplacement :** A la fin de la section 12, apres la derniere
entree chronologique. Le texte actuel se termine par :

```
- Sprint 13 : bridge postMessage — premier sprint avec
  communication iframe ↔ reseau, open source enforcement,
  et launcher Rust. Le pattern sprint s'est stabilise :
  les sessions livrent 4 phases en une seule session.
```

### old_string

```
- Sprint 13 : bridge postMessage — premier sprint avec
  communication iframe ↔ reseau, open source enforcement,
  et launcher Rust. Le pattern sprint s'est stabilise :
  les sessions livrent 4 phases en une seule session.
```

### new_string

```
- Sprint 13 : bridge postMessage — premier sprint avec
  communication iframe ↔ reseau, open source enforcement,
  et launcher Rust. Le pattern sprint s'est stabilise :
  les sessions livrent 4 phases en une seule session.
- Sprint 65 : introduction du dual-agent verification process
  (§4.5). Chaque phase est verifiee par Claude teams
  (Opus 4.6, 1M tokens, agents paralleles pour G8) puis par
  Codex CLI GPT 5.5 en review croisee independante. Le
  preflight G8 passe de sequentiel (15 min) a parallele
  (5 min) via 5 agents dedies. Le cycle de correction
  post-Codex ajoute une boucle conditionnelle (re-run si
  > 10 LOC corriges). Les templates de prompt Codex sont
  normalises (§4.5.3, §4.5.4). Anti-patterns documentes
  (§4.5.2 : pas de `-m o3`, pas de here-string PowerShell,
  pas de prompt inline).
```

---

## 6. Mise a jour fichiers dependants

Le frontmatter de README.md liste les fichiers dependants qui
citent des sections specifiques. L'ajout de §4.5 impacte :

| Fichier | Modification necessaire |
|---------|------------------------|
| `.claude/skills/nexus-phase-review/SKILL.md` | Ajouter reference §4.5 dans la section Refs |
| `.claude/skills/nexus-phase-preflight/SKILL.md` | Ajouter reference §4.5.7 (parallelisation) dans la section Refs |
| `.claude/agents/nexus-phase-auditor.md` | Ajouter mention Codex post-audit dans la section Procedure |
| `docs/claude/TOOLING.md` | Ajouter couche 4 "Codex cross-review" au tableau §1 |
| `CLAUDE.md` | Pas de changement (pointe deja vers README.md) |

### TOOLING.md — ajout couche 4

**old_string** dans TOOLING.md section 1 :

```
| # | Couche | Moment | Outil principal |
|---|---|---|---|
| 1 | Garde-fous automatiques | PostToolUse (chaque write) | `.claude/hooks/verify-on-write.sh` + Semgrep |
| 2 | Skills qualite specialises | Sur demande Claude | Trail of Bits skills + `nexus-phase-review` |
| 3 | Subagent review intra-sprint | Pre-commit d'une phase | `nexus-phase-auditor` agent (inconditionnel) |
```

**new_string** :

```
| # | Couche | Moment | Outil principal |
|---|---|---|---|
| 1 | Garde-fous automatiques | PostToolUse (chaque write) | `.claude/hooks/verify-on-write.sh` + Semgrep |
| 2 | Skills qualite specialises | Sur demande Claude | Trail of Bits skills + `nexus-phase-review` |
| 3 | Subagent review intra-sprint | Pre-commit d'une phase | `nexus-phase-auditor` agent (inconditionnel) |
| 4 | Cross-review dual-agent | Post-review Claude, pre-commit | Codex CLI GPT 5.5 (`codex exec`) |
```

---

## 7. Template prompt Codex — fichiers prets a l'emploi

### 7.1 Fichier template generique phase verification

Chemin suggere : `.claude/templates/codex_phase_review.txt`

Ce fichier est un template avec placeholders. L'agent Claude le
copie dans `.git/CODEX_SPRINT{N}_PHASE_{X}.txt` en remplacant les placeholders
avant chaque lancement.

```
Tu es un auditeur independant. Tu ne connais PAS l'historique de
cette session. Tu verifies le code source du projet nexus-grid
(SBFB) apres une phase de sprint.

Sprint : {SPRINT_N}
Phase : {PHASE_X} — {PHASE_TITLE}
Branch : master

## Livrables attendus de cette phase

{DELIVERABLES_LIST}

## Ta mission

Pour CHAQUE livrable ci-dessus :

1. Cherche dans le code source les fichiers concernes.
2. Verifie que le livrable est REELLEMENT implemente (pas juste
   un TODO, un stub, ou un test sans assertion).
3. Cite le fichier et les numeros de ligne exacts.
4. Conclus : CONFIRME (avec evidence) ou GAP (avec description
   de ce qui manque).

## Format de reponse

Pour chaque livrable :

### Livrable N : {titre}
- Statut : CONFIRME | GAP | PARTIEL
- Fichier(s) : {chemin:ligne}
- Evidence : {extrait code 3-5 lignes}
- Si GAP : estimation en nombre de lignes du fix manquant

## Resume final

- Total livrables : N
- Confirmes : N
- Gaps : N
- Partiels : N
- Estimation totale LOC fixes manquants : N

## Contraintes

- Reponse en francais.
- Pas d'opinion sur l'architecture — tu verifies, tu ne redesigns pas.
- Si un test existe mais n'a pas d'assertion utile, c'est un GAP.
- Si un fichier documente est mentionne mais n'existe pas, c'est un GAP.
```

### 7.2 Fichier template preflight verification

Chemin suggere : `.claude/templates/codex_preflight_review.txt`

```
Tu es un auditeur independant. Tu verifies qu'un preflight G8
(gate pre-implementation) a ete correctement execute sur le
projet nexus-grid (SBFB).

Sprint : {SPRINT_N}
Phase : {PHASE_X}

## Fichier preflight a auditer

Lis le fichier :
.planning/active/sprint{SPRINT_N}_phase_{PHASE_X}_preflight.md

## Verification requise

Pour chaque scan du preflight :

### S1a — OSS prior art
- Le preflight cite-t-il au moins 1 projet OSS de reference ?
- L'approche du plan est-elle comparee au SOTA ?
- Verdict S1a present et justifie ?

### S1b — Deps/libs versions
- Les libs critiques sont-elles scannees (version + CVE) ?
- context7 ou WebSearch traces ?

### S2 — Decisions historiques
- git log scan effectue sur les fichiers cibles ?
- Reverse-commit check documente si finding ?

### S3 — Threat model
- Fast-path justifie ou full scan si composant securite ?
- HARDENING_ROADMAP consulte ?

### S4 — Wire format
- *_VERSION fields verifies ?
- Day 0 preservees ?

## Format de reponse

| Scan | Present | Complet | Evidence |
|------|---------|---------|----------|
| S1a  | OUI/NON | OUI/PARTIEL/NON | {details} |
| S1b  | ...     | ...     | ... |
| S2   | ...     | ...     | ... |
| S3   | ...     | ...     | ... |
| S4   | ...     | ...     | ... |

## Verdict global

- PASS : 5/5 scans presents et complets
- CONCERN : 1+ scan partiel mais pas de gap critique
- FAIL : scan manquant ou finding bloquant non detecte

Reponse en francais.
```

### 7.3 Fichier template recheck cible

Chemin suggere : `.claude/templates/codex_recheck.txt`

```
Tu es un auditeur independant. Tu fais un recheck cible apres
correction de GAPs identifies lors d'une review precedente.

Sprint : {SPRINT_N}
Phase : {PHASE_X}

## GAPs corriges

{GAPS_LIST_WITH_FILES}

## Fichiers modifies

{MODIFIED_FILES_LIST}

## Ta mission

Pour CHAQUE GAP corrige :

1. Lis le fichier modifie aux numeros de ligne indiques.
2. Verifie que le fix est complet (pas partiel, pas un stub).
3. Verifie qu'aucune regression n'a ete introduite dans les
   lignes voisines (+-20 lignes de contexte).

## Format de reponse

### GAP N : {titre}
- Statut : CORRIGE | ENCORE GAP | REGRESSION
- Fichier(s) : {chemin:ligne}
- Evidence : {extrait code}

## Verdict

- PASS : tous les GAPs corriges sans regression
- FAIL : au moins 1 GAP non corrige ou regression

Reponse en francais.
```

---

## 8. Diagramme de sequence complet (reference)

Le diagramme suivant montre le flux complet d'une phase standard
depuis S65, integrant tous les agents :

```
PO / Plan.md                  Claude Team              Codex GPT 5.5
     |                             |                        |
     | Plan §Phase X               |                        |
     |---------------------------→ |                        |
     |                             |                        |
     |                     ┌───────┴───────┐                |
     |                     │ G8 Preflight  │                |
     |                     │ 5 agents //   │                |
     |                     │ S1a,S1b,S2,   │                |
     |                     │ S3,S4         │                |
     |                     └───────┬───────┘                |
     |                             |                        |
     |                     Verdict G8                       |
     |                     (EXECUTE/PLAN-ADAPT)             |
     |                             |                        |
     |                     ┌───────┴───────┐                |
     |                     │ Code phase    │                |
     |                     │ (implementation)│               |
     |                     └───────┬───────┘                |
     |                             |                        |
     |                     ┌───────┴───────┐                |
     |                     │ Suites §7.4   │                |
     |                     │ (Rust+Web)    │                |
     |                     └───────┬───────┘                |
     |                             |                        |
     |                     ┌───────┴───────┐                |
     |                     │ Review Claude │                |
     |                     │ (skill+agent) │                |
     |                     │ Verdict PASS  │                |
     |                     └───────┬───────┘                |
     |                             |                        |
     |                     Ecrit prompt .git/CODEX_*.txt    |
     |                             |                        |
     |                             | Get-Content | codex exec
     |                             |──────────────────────→ |
     |                             |                        |
     |                             |           Review code  |
     |                             |           source GPT 5.5|
     |                             |                        |
     |                             | ←─────── rapport .md   |
     |                             |                        |
     |                     ┌───────┴───────┐                |
     |                     │ Triage GAPs   │                |
     |                     │ Correction    │                |
     |                     └───────┬───────┘                |
     |                             |                        |
     |                     [si > 10 LOC corriges]           |
     |                             | recheck ──────────────→|
     |                             | ←─────── verdict       |
     |                             |                        |
     |                     ┌───────┴───────┐                |
     |                     │ Commit        │                |
     |                     │ atomique      │                |
     |                     └───────────────┘                |
```

---

## 9. Articulation avec les gates existants

Le dual-agent s'integre sans modifier les gates G1-G9 :

| Gate | Role existant | Impact dual-agent |
|------|---------------|-------------------|
| G1 (Design Review Board) | Scoring D1..D5 kickoff | Inchange |
| G2 (Triggers revalidate) | Re-validation docs long-life | Inchange |
| G3 (Goal SMART) | Kickoff §2 → verification.md | Inchange |
| G4 (Rigor signal) | Review + audit gate verdicts | **Enrichi** : Codex findings ajoutes comme evidence supplementaire |
| G6 (Memory update) | Post-commit + Phase F | Inchange |
| G7 (Carry-overs) | Escalade 3 reports + dette | Inchange |
| G8 (Preflight) | 5 scans pre-implementation | **Parallelise** : 5 agents independants |
| G9 (Factual research) | Research avant D-choice | Inchange |

Le dual-agent ne cree PAS de nouveau gate G10. C'est une
implementation supplementaire dans le perimetre de G4 (quality
signal).

---

## 10. Metriques de suivi

Pour evaluer l'utilite du dual-agent au fil des sprints :

| Metrique | Comment mesurer | Seuil d'action |
|----------|----------------|----------------|
| Taux de GAPs confirmes par Codex / phase | Count GAPs / total livrables | Si < 5% sur 3 sprints consecutifs : evaluer si le cout Codex vaut le benefice |
| Taux de faux positifs Codex / phase | Count faux positifs / total findings | Si > 50% : ameliorer le template prompt |
| Temps total dual-agent / phase | Mesure dans commit body | Si > 20 min : optimiser (prompt plus court, recheck seulement si critique) |
| Bugs trouves par Codex mais pas par Claude | GAPs confirmes non detectes par review Claude | La metrique de valeur unique du dual-agent |
| Sprints depuis dernier finding Codex utile | Compteur incremente | Si > 4 sprints : reevaluer le besoin du dual-agent |

---

## 11. Pre-submission checklist

Avant d'appliquer ces modifications a README.md :

- [ ] Section 4.5 inseree entre 4.4 et 5
- [ ] Section 7.1 Cas B enrichie avec etape Codex
- [ ] Section 6.9 enrichie avec parallelisation agents
- [ ] Section 4.3 enrichie avec sequence Codex
- [ ] Section 12 enrichie avec entree chronologique S65
- [ ] TOOLING.md enrichi avec couche 4
- [ ] Templates de prompts Codex prets dans .claude/templates/
- [ ] Anti-patterns documentes (PowerShell, -m o3, here-string)
- [ ] Cycle de correction documente (triage + recheck conditionnel)
- [ ] Exemptions documentees (docs-only, < 5 LOC, cas D, PO skip)
- [ ] Tracabilite dans commit body documentee (section ## Codex verification)
- [ ] Metriques de suivi documentees
- [ ] Fichiers dependants listes pour mise a jour
