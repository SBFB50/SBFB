# S24 Process Review — Diagnostic factuel du système de travail SBFB

**Date** : 2026-04-21
**Auteur** : session Opus 4.6 (1M context), research indépendante
**Baseline** : tip `34c77ce` (master, pre-rework 4.7)
**Scope** : S16-S23 (v1.2), 8 sprints, ~743 tests Rust, ~1563 total
**Méthode** : lecture exhaustive README.md (2242L) + TOOLING.md (680L) +
agent prompt (516L) + 2 skills + 2 hooks + 8 audit_findings + 33 phase
reviews + git log complet + benchmarks modèle publics

---

## 1. Diagnostic factuel — chaque garde-fou au scanner

### 1.1 Tableau synthétique

| ID | Nom | Origine | Trigger | Findings catchés | Coût | Verdict |
|---|---|---|---|---|---|---|
| G1 | Design Review Board | S19 `fe0a8fd`, incident D2 PoW Hashcash/Equi-X | Kickoff, après draft D1..D5 | S19 P3-B2 (Hashcash daté). S21 D2 PII gap (extension Rust-first) | ~15 min agent Explore + 30L README | **KEEP** |
| G2 | Triggers revalidate | S19/20 `fe0a8fd`, incident HARDENING_ROADMAP S17 stale | Events upstream (release, CVE, S+2) | **0 findings en 4 sprints** — aucun trigger n'a effectivement fire S20-S23 | Frontmatter YAML + session-start check | **SIMPLIFIER** |
| G3 | Goal SMART → verification.md | Implicite structure sprint | Kickoff §2 | Structurel, pas mesurable isolément | 1 phrase dans kickoff | **KEEP** |
| G4 | Rigor signal (≥1 P2+) | S19 Phase B CONCERN→PASS cosmétique `fe0a8fd` | Phase review + audit gate | Appliqué systématiquement S20-S23. 33 reviews, tous sauf 1 trouvent ≥1 P2+ | ~50L dans agent prompt | **KEEP** |
| G5 | Working tree audit body | S19/20 `fe0a8fd` | Pre-commit phase | Table PHASE/CRAFT/DEBT/NOISE jamais catché un finding autonome. Hook lightcheck Check 1 (`34dacdc`) couvre le seul catch réel (P1 S22-D profile.rs untracked) | Template ~15L body + skill Step 1bis | **DROP** |
| G6 | Memory update post-commit | S20+, formalisé S23 `ab24080` | Après chaque feat commit | Comble le drift memory (pivot stale >7j). Pas de finding spécifique — c'est un effet préventif | ~30s par commit | **KEEP** |
| G7 | Cap carry-overs (2/sprint) | S18→19 `fe0a8fd`, incident C-1 DHT report gratuit | Phase F carry generation | S19 P2-A-2 (5 carries vs cap 2). Meta-1 Radicle reclassifié long-term après 4 sprints | Cap check + reclassification doc | **KEEP** |
| G8 | Phase preflight (4 scans) | S20 `59225ee`, incident canary auto-publish scheduler ré-proposé après rejet S18 | Avant 1ère ligne code phase | **2 DESIGN-CONFLICT catchés S20-S21**. 100% couverture preflights S20-S23 (toutes les phases ont un preflight.md) | ~5-15 min/phase, 4 scans factuels | **KEEP — plus forte preuve causale** |
| G9 | Factual research gate D-decisions | S21 `71de0ec`, incident D2 PII draft sans research Rust ML | Avant proposition draft D-choice | S21 D2 PII : research a révélé tract/ort/gline-rs limitations → décision changée | Research time kickoff | **KEEP** |
| C1-C9 | Audit conditionnel (amendement) | S22 `34dacdc`, mesure ROI ~97k tokens/6.5 min pour 1 finding | Hook pre-commit regex | **P1 ÉCHAPPÉ S23** : C1 regex ne matchait pas `task.rs` → `redundancy_factor` dans canonical bytes non détecté | ~120L script bash | **DROP — faux négatif prouvé** |
| §4.4 | Phase F parse reviews | P2-S21-4, wrap-up oublie carries | Phase F écriture audit_plan | Incident documenté S21 | ~40L README | **KEEP** |
| §6.7 | LOC ban kickoff/plan | 3× S22 P2-E-2 | Auditor grep + review Step 4ter | Pattern récurrent 3 occurrences même sprint | ~20L README | **KEEP** |
| §6.11 | Archive research outputs | S21 `71de0ec`, 9000 mots perdus dans transcript | Research output >2000 mots | Préventif — 4 archives retroactifs S21 | ~60L README | **SIMPLIFIER** |

### 1.2 Détail des éléments critiques

#### G5 — Working tree audit : autopsie d'un garde-fou mort

**Preuve d'absence de valeur** : j'ai relu les 33 phase reviews S18-S23.
Aucun ne mentionne un finding dont la source est la table Working tree
audit du body commit. Les catches réels de staging (profile.rs untracked
S22 Phase D) viennent du hook `phase-precommit-lightcheck.sh` Check 1
(STRICT BLOCK), pas de la catégorisation PHASE/CRAFT/DEBT/NOISE.

La table dans le body commit servait de documentation a posteriori,
pas de mécanisme de détection. Le hook lightcheck fait la détection.
Le `git log --stat` + split commits `chore(planning)` visible dans
l'historique fournit la documentation équivalente.

**Verdict** : le diff non commité dans le working tree (marquant G5
comme ~~supprimé~~) est correct. G5 n'a aucune valeur causale prouvée
au-delà de ce que le hook lightcheck et la discipline split commits
couvrent déjà.

#### C1-C9 — Audit conditionnel : autopsie d'un faux négatif

**Chronologie factuelle** :
1. `34dacdc` (S22) introduit C1-C9. C1 regex : `^crates/nexus-core-rs/src/(canonical|schemas/)`.
2. `8146db7` étend C3 à `.rs|.py`, ajoute C9 (threat docs).
3. S23 Phase D (`dc163ea`) ajoute `redundancy_factor: u8` dans `crates/nexus-core-rs/src/task.rs`.
4. `task.rs` n'est PAS sous `canonical/` ni `schemas/` → C1 ne fire pas.
5. Aucun autre C2-C9 ne fire non plus (pas de `_VERSION` bump, pas de fichier crypto nommé, etc.).
6. Résultat : **aucun review file** pour Phase D. Le P1 C-1 (redundancy_factor dans canonical bytes) arrive à l'audit gate S24.
7. L'audit gate S24 le catchait — mais seulement 1 sprint plus tard, avec un coût fixe de `fix(sprint23): ...` bloquant S24 Phase A.

**Le faux négatif est structurel, pas accidentel** : n'importe quel
fichier `.rs` qui contient un champ participant à `canonical_bytes()`
mais qui n'est pas nommé `canonical.rs` ni dans `schemas/` échappera
à C1. C'est le problème fondamental des heuristiques filename-based
sur du code sémantique : le compilateur sait quels champs participent
à la sérialisation JCS, le regex ne le sait pas.

**Alternatives à C1-C9** :
- **Affiner les regex** (ex: ajouter `task.rs`, `project.rs`, etc.) : jeu de taupe permanent, chaque nouveau struct qui participe au canonical doit être ajouté manuellement.
- **Audit inconditionnel** (retour au pre-`34dacdc`) : coût ~95k tokens × 5-6 phases/sprint = ~475-570k tokens/sprint. Mais détecte le P1 en pre-commit plutôt qu'en audit gate S+1.
- **Semgrep rule `sbfb-canonical-bytes-jcs`** : déjà listé dans TOOLING.md comme TODO. Détecte sémantiquement les structs participant à `canonical_bytes()` sans dépendre des noms de fichier. Pas encore implémenté.

**Verdict** : drop C1-C9, retour audit inconditionnel. Le coût token
est réel mais acceptable (le projet a tourné S16-S22 Phase A sans
C1-C9, la régression de qualité post-`34dacdc` en S23 est mesurable).
La Semgrep rule TODO est la bonne solution long-terme mais ne doit
pas bloquer le retour immédiat à l'audit inconditionnel.

#### G2 — Triggers revalidate : théoriquement bon, jamais testé

Les frontmatter `triggers_revalidate` existent dans :
- `docs/security/HARDENING_ROADMAP.md`
- `docs/security/THREAT_MODEL.md`
- `docs/security/VALIDATED_BLUEPRINT.md`

Aucun trigger n'a fire entre S20 et S23. Raisons :
- iroh est resté pinné à 0.97 (pas de release majeure)
- wasmtime n'a pas eu de LTS bump depuis S18
- Aucun CVE bloquant n'a été annoncé sur les deps critiques

Le mécanisme est structurellement sain : quand un trigger fire, il
force une re-validation factuelle. Mais en 4 sprints, la preuve
causale est 0. La question est : le coût de maintenir les frontmatter
justifie-t-il l'assurance théorique ?

**Verdict** : simplifier, pas drop. Le frontmatter est low-cost à
maintenir. La procédure de re-validation (grep triggers → re-scan)
est documentée dans README §6.8 en ~40 lignes. Réduire à 10 lignes
(le principe + la commande grep) suffit. Les paragraphes d'exemples
et de rationale sont de la documentation de design, pas du process
opérationnel.

---

## 2. Pattern oscillation : pourquoi ça ne converge pas

### 2.1 Chronologie factuelle de l'oscillation

| Commit | Sprint | Direction | Delta lignes README |
|---|---|---|---|
| `fe0a8fd` | S19 | **Accumulation** : G1..G7 + no-LOC + research | +~400L |
| `59225ee` | S20 | **Accumulation** : G8 preflight | +~200L |
| `71de0ec` | S21 | **Accumulation** : G1 extensions + G9 + §6.11 | +~350L |
| `34dacdc` | S22 | **Accumulation+optimisation** : C1-C9 conditionnel | +~120L hook |
| `2438c59` | S22 | **BANKRUPTCY** : purge hooks, compress context | -~300L README, 5 hooks supprimés |
| `f6d3ee5` | S23 | **Restauration** : commit body template, cleanup scories | +~30L |
| `5f35772` | S23 | **Restauration** : §6.7 LOC ban | +~20L |
| `ab24080` | S23 | **Restauration** : G6 memory update | +~30L |
| `56816ac` | S23 | **Restauration** : full fail-fast pre-commit | +~15L |
| (WIP 4.7) | S24 | **Shaving v2** : drop G5 + C1-C9, add §10.1 | -~150L README, -~120L hook |

### 2.2 Le mécanisme structurel

L'oscillation vient de **trois forces en conflit sans arbitre** :

**Force 1 — Pression d'incident** : chaque finding P1/P2 non prévenu
par le process existant génère une nouvelle règle. C'est un réflexe
correct individuellement (root cause → countermeasure) mais
cumulatif : S19-S22 = 4 sprints × 2-3 règles = 8-12 ajouts.

**Force 2 — Pression de contexte** : le README.md à 2242 lignes + le
TOOLING.md à 680 lignes + l'agent prompt à 516 lignes = ~3438 lignes
de process. Chaque session fraîche doit lire au minimum ~1500 lignes
(README §1-§7 + agent prompt) avant d'écrire la moindre ligne de
code. Ça représente ~30k tokens de contexte process sur un budget
1M — 3% du contexte, acceptable en absolu, mais chaque ligne de
process est une instruction que le modèle doit suivre. Plus il y en
a, plus la probabilité de violation stochastique augmente.

**Force 3 — Pression d'optimisation** : une session voit le volume et
cherche à "shaver". Le critère naturel est "preuve causale de
valeur". Mais ce critère est structurellement biaisé :

- Les **mécanismes de détection** (hooks, auditor regex, Semgrep
  rules) produisent des findings traçables → preuve causale facile.
- Les **mécanismes de dissuasion** (§6.7 LOC ban, G1 extensions,
  §6.11 archive) ne produisent PAS de findings traçables par
  construction — leur valeur est d'empêcher un comportement, pas de
  le détecter. Un LOC ban qui fonctionne = 0 occurrences de LOC
  dans les plans = 0 findings = "pas de preuve causale" selon le
  critère bankruptcy.
- Résultat : le bankruptcy drop les mécanismes dissuasifs (valeur
  non mesurable) et keep les mécanismes détectifs (valeur
  mesurable). Le sprint suivant, le comportement dissuadé
  réapparaît (ex: 3× LOC estimation en S22 après que la règle n'a
  PAS été appliquée pendant le bankruptcy). La règle est
  ré-ajoutée. Cycle complet.

**L'arbitre manquant** : il n'y a pas de framework stable pour
distinguer "cette règle est morte" de "cette règle fonctionne
silencieusement". Le bankruptcy utilisait "causal proof OR drop".
La restauration utilisait "incident-derived = keep". Les deux sont
raisonnables, les deux sont incomplets, et personne ne tranche
durablement entre les deux critères.

### 2.3 Pourquoi le rework Opus 4.7 a échoué (2026-04-21)

Trois causes factuelles observées dans l'historique de la session :

1. **Violation de la decision gelée README §10 ligne 1991** : la
   session 4.7 a exécuté un rework process sur un projet dont les
   instructions disent "rester Opus 4.6 (régression MRCR -46pp @1M)".
   Le modèle 4.7 a un read-to-edit ratio de 2.0 (vs 6.6 pour 4.6) —
   il a modifié README.md sans avoir pleinement intégré la contrainte
   modèle documentée dans ce même README.

2. **Oubli d'orphelins** : le rework a drop C1-C9 du hook mais pas
   nettoyé les références dans SKILL.md review + preflight +
   TOOLING.md §5.2. 3 fichiers orphelins. Ce n'est pas une erreur
   de jugement, c'est un problème de complétude mécanique — le
   modèle n'a pas grep les références croisées avant d'appliquer
   le changement.

3. **Méta-analyse infinie** : la session a produit des analyses
   v2/v3a/v3b/philosophie "principes vs implémentations" sans
   converger sur des actions. Le pattern 4.7 (action prématurée
   sur le code + correction retention fail) s'est manifesté comme
   oscillation entre "analyser" et "réécrire" sans stabiliser.

---

## 3. Propositions d'amélioration

### 3.1 Stabiliser le cycle accumulation/shaving

**Mode de défaillance visé** : le process oscille parce qu'il n'y a
pas de critère stable de valeur pour les règles dissuasives.

**Proposition** : chaque règle dans le README porte un
**tag de classification** :

```
[DETECT] — mécanisme mécanique observable (hook, regex, Semgrep)
           Critère de drop : 0 findings en 4 sprints consécutifs
[DETER]  — principe documenté dissuasif (LOC ban, research-first)
           Critère de drop : 3× violations observées MALGRÉ la règle
           (preuve que la dissuasion ne fonctionne pas)
[STRUCT] — structure du cycle sprint (audit gate, Phase F, etc.)
           Critère de drop : jamais (changement = refonte du process)
```

Chaque session bankruptcy vérifie les critères de drop par tag, pas
le critère unique "causal proof". Un `[DETER]` avec 0 violations en
4 sprints est un succès, pas un candidat au drop.

**Coût** : 1 tag par règle dans le README, vérification mécanique
en bankruptcy. ~30 minutes d'effort au moment du rework, vs des
heures de méta-analyse infructueuse.

**Critère de retrait de cette proposition** : si en 2 reworks
successifs le tag est ignoré (la session fait quand même un shaving
arbitraire), le tagging ne fonctionne pas.

### 3.2 Empêcher les reworks incohérents (anti-pattern 2026-04-21)

**Mode de défaillance visé** : une session modifie un fichier hub
(README.md, TOOLING.md) sans nettoyer les références croisées dans
les fichiers dépendants (skills, hooks, agent prompt).

**Proposition** : ajouter au header de `docs/claude/README.md` une
section `## Fichiers dépendants` qui liste les fichiers qui
référencent ce document :

```markdown
## Fichiers dépendants (à mettre à jour si ce document change)
- .claude/agents/nexus-phase-auditor.md (cite §3, §6.9, §9)
- .claude/skills/nexus-phase-review/SKILL.md (cite §4.3, §6.7, §7.4)
- .claude/skills/nexus-phase-preflight/SKILL.md (cite §6.9, §7.1)
- .claude/hooks/phase-auditor-gate.sh (implémente §5.2 TOOLING)
- .claude/hooks/phase-precommit-lightcheck.sh (implémente §5.2 TOOLING)
- docs/claude/TOOLING.md (cite §3, §4, §5, §7)
- CLAUDE.md (pointe vers ce document)
```

Un diff de README.md qui supprime une section (grep `^##.*G[0-9]` ou
`^###.*C[0-9]`) sans diff correspondant dans les fichiers dépendants
= signal d'orphelins.

**Mécanisme d'enforcement** : pas de hook (coût runtime). Discipline
documentée : le rédacteur grep les identifiants supprimés dans les
fichiers dépendants avant commit. L'audit gate Phase 0 vérifie la
cohérence cross-fichiers comme Track standard.

**Coût** : ~10 lignes de section dans README, grep mécanique lors de
reworks. Pas de hook, pas de token runtime.

**Critère de retrait** : si en 3 reworks consécutifs le grep est
fait et aucun orphelin n'est trouvé, la section est devenue
documentation morte.

### 3.3 Retour audit inconditionnel (drop C1-C9)

**Mode de défaillance visé** : P1 C-1 S23 échappé via regex C1
trop étroite sur `task.rs`.

**Proposition** : supprimer le bloc `=== Amendement criteres
conditional run ===` du hook `phase-auditor-gate.sh`. Revenir à
l'audit inconditionnel : tout commit `feat|fix(sprintN)...Phase X`
déclenche la vérification du review.md PASS.

**Ce que ça coûte** : ~95k tokens × ~5 phases/sprint = ~475k tokens
par sprint. À $0.015/1k tokens output, c'est ~$7/sprint. Le projet
a un budget token illimité (pas de facturation externe — c'est un
projet solo dev avec abonnement Claude). Le coût réel est le temps :
~6 min par phase × 5 = 30 min/sprint d'overhead auditor.

**Ce que ça gagne** : le P1 C-1 aurait été catché en pre-commit de
Phase D au lieu d'arriver à l'audit gate S+1. Coût du fix pré-commit
(inline) vs fix post-sprint (`fix(sprint23):` bloquant S24 Phase A)
= ~2h de latence économisées.

**Alternative long-terme** : implémenter la Semgrep rule
`sbfb-canonical-bytes-jcs` (détection sémantique des structs
participant à `canonical_bytes()`). Ça rendrait C1 redondant de
manière robuste. Mais la rule n'existe pas encore et ne doit pas
bloquer la correction immédiate.

**Critère de retrait** : quand la Semgrep rule est implémentée ET
a catché ≥1 finding, réévaluer si l'audit inconditionnel reste
nécessaire ou si la rule + audit conditionnel affiné suffit.

### 3.4 Compacter les extensions G1

**Mode de défaillance visé** : les extensions G1 (crypto-spec
~50 lignes, custom-Rust-stack ~80 lignes) gonflent le README sans
que la session typique ait besoin de lire le détail. Le principe
G1 (reviewer indépendant score les sources) est solide. Les
extensions ajoutent des checklists spécifiques.

**Proposition** : réduire les deux extensions à une checklist de 5
lignes chacune dans la section G1, avec un pointeur vers le design
doc d'origine dans `.planning/research/` ou l'archive sprint pour
le rationale complet.

Avant (130 lignes) :
```
### 6.1.1 Design Review Board — reviewer independant...
[30 lignes de base G1]
[50 lignes extension crypto-spec avec historique Tor/Equi-X]
[80 lignes extension custom-Rust-stack avec liste alternatives]
```

Après (~50 lignes) :
```
### 6.1.1 Design Review Board — reviewer independant...
[30 lignes de base G1, inchangé]

**Checklist crypto/spec** (ajouté S19, incident Tor PoW/Equi-X) :
- [ ] D-choice cite ≥1 alternative concurrente <6 mois
- [ ] Source crypto datée <2 ans ou explicitement revalidée
- [ ] Reviewer ⚠️ si alternative absente

**Checklist Rust-first** (ajouté S21, incident D2 PII) :
- [ ] D-choice runtime cite ≥1 alternative Rust-native production
- [ ] Gap factuel documenté si alternative Rust rejetée
- [ ] Reviewer ⚠️ si gap non documenté
- Exemptions : CI tooling, frontend UX, docs, tests fixtures

Rationale complet : `.planning/archive/v1.2/sprint19_audit_findings.md`
(crypto-spec) et `sprint21_kickoff.md §Sources` (Rust-first).
```

**Coût** : ~80 lignes économisées. Les checklists sont exécutables.
Le rationale reste accessible mais ne pollue plus la lecture par
défaut.

**Critère de retrait** : si en 3 sprints le reviewer Explore ne
consulte jamais les checklists (toutes les D-decisions passent sans
⚠️), les checklists sont devenues documentation morte — indiquer
directement dans G1 "cf. feedback_approach.md research-first" sans
checklist séparée.

### 3.5 Compacter §6.11 (archive research outputs)

**Mode de défaillance visé** : §6.11 fait ~80 lignes pour un pattern
qui se résume à "write research >2000 mots dans .planning/research/
avant de continuer la session".

**Proposition** : réduire à ~15 lignes (principe + template
frontmatter + critères de skip). Le rationale (pourquoi archiver, 4
buts, exemples S21) devient un commentaire dans le git log du commit
qui a introduit §6.11 (`71de0ec`), pas une section du README lu à
chaque session.

**Coût** : ~65 lignes économisées. Le pattern reste exécutable.

**Critère de retrait** : si en 4 sprints aucun research output n'est
archivé (parce que le projet est en mode maintenance, pas en mode
design), la section est morte.

---

## 4. Verdict par garde-fou

### 4.1 Solides — garder tel quel

| ID | Pourquoi solide |
|---|---|
| G1 (base) | Preuve causale directe (S19 Hashcash, S21 PII). Coût faible (15 min/sprint). |
| G4 | Rigor signal (≥1 P2+) appliqué systématiquement, drive la qualité des audits. |
| G7 | Cap carry-overs discipline prouvée (S19 5→cap 2, Meta-1 reclassifié). |
| G8 | Plus forte preuve causale du projet (2 DESIGN-CONFLICT catchés, 100% couverture). |
| G9 | Incident-dérivé, changement de décision factuel (D2 PII Rust→JS). |
| §4.4 | Incident documenté (P2-S21-4), coût minimal (40L lues 1×). |
| §6.7 LOC | Pattern récurrent 3× S22, contre-mesure cognitive active. |

### 4.2 À simplifier

| ID | Action | Gain |
|---|---|---|
| G1 extensions | Réduire à checklists 5L chacune + pointeur archive | -80L README |
| G2 triggers | Réduire à 10L (principe + commande grep) | -30L README |
| §6.11 archive | Réduire à 15L (pattern + template + skip) | -65L README |
| G5 working tree audit | Déjà marqué drop dans le diff non commité, confirmer | -15L template body |

### 4.3 À droper avec critère de ré-ajout

| ID | Raison du drop | Critère de ré-ajout |
|---|---|---|
| C1-C9 conditionnel | Faux négatif prouvé S23 P1 C-1 | Quand Semgrep rule `sbfb-canonical-bytes-jcs` est implémentée + a catché ≥1 finding |
| G5 (table body) | 0 finding autonome en 33 reviews. Hook lightcheck couvre | Si hook lightcheck rate un staging manquant 2× en 3 sprints |

### 4.4 Méta-action : stabiliser le cycle

| Action | Quand | Qui |
|---|---|---|
| Tagging [DETECT]/[DETER]/[STRUCT] | S24 Phase process-review | Session process |
| Section "Fichiers dépendants" dans README header | S24 Phase process-review | Session process |
| Critère de drop par tag (pas critère unique "causal proof") | Toute future session bankruptcy | Documenté dans README §12 |

---

## 5. Analyse objective des modèles — Opus 4.5 / 4.6 / 4.7

### 5.1 Benchmarks factuels (avril 2026, sources publiques)

| Métrique | Opus 4.5 | Opus 4.6 | Opus 4.7 |
|---|---|---|---|
| SWE-bench Verified | 80.9% | 80.8% | 87.6% |
| SWE-bench Pro | 45.9% | 51.9% | 64.3% |
| MRCR @256K | n/d | 93% | 59.2% |
| MRCR @1M | n/d | 76% | 32.2% |
| Terminal-Bench | n/d | 65.4% | 69.4% |
| Read-to-edit ratio | n/d | 6.6 | 2.0 |
| Correction retention cycle 7 | n/d | stable | **régression** (même erreur cycle 1→7) |
| Statut | **Deprecated** 2026-03-20 | Actif | Actif |

### 5.2 Analyse par rapport au workflow SBFB

**Opus 4.5** : deprecated depuis le 2026-03-20, API renvoie
model-not-found. N'est plus une option. Ses forces étaient la
constance en session longue autonome et la discipline. Sa faiblesse
connue : "rule-bending" (contournement créatif des policies). Avait
un SWE-bench comparable à 4.6 (80.9 vs 80.8).

**Opus 4.6** (modèle actuel du projet) :
- MRCR @1M = 76% : fiable pour retrouver des instructions dans un
  contexte de 2242 lignes README + 680 lignes TOOLING + transcripts
  session. Le process SBFB repose sur le fait que le modèle suit les
  instructions lues en début de session tout au long de l'exécution.
- Read-to-edit ratio 6.6 : lit 6.6× plus qu'il n'écrit. Correspond
  au pattern SBFB (lire kickoff + plan + code existant avant
  d'écrire).
- Correction retention stable : quand un finding est corrigé, la
  correction persiste dans la session. Critique pour le cycle
  auditor → fix → re-audit.

**Opus 4.7** :
- SWE-bench +7pp : supérieur pour résoudre des bugs isolés. Moins
  pertinent pour SBFB (le bottleneck n'est pas la résolution de bugs
  mais la discipline de process sur des sessions longues).
- MRCR @1M = 32.2% (-44pp) : **structurellement incompatible** avec
  le workflow SBFB. Le README.md seul fait ~45k tokens. Avec
  TOOLING.md + agent prompt + kickoff + plan + code lu, une session
  SBFB atteint facilement 200-400k tokens de contexte. À 256K, 4.7
  est déjà à 59.2% MRCR. Le risque de "perdre" une instruction
  process (scope cut, D-decision gelée, convention commit body) est
  élevé.
- Read-to-edit ratio 2.0 : le modèle écrit sans avoir
  suffisamment lu. C'est exactement ce qui s'est passé dans le
  rework 4.7 (modification de README.md sans vérifier les fichiers
  dépendants, suppression de C1-C9 dans le hook sans nettoyer les
  refs dans les skills).
- Correction retention fail : une erreur corrigée en début de
  session réapparaît 7 cycles plus tard. C'est le pattern observé
  dans la session rework 4.7 (oubli d'orphelins corrigé en v2 →
  réintroduit en v3a).

### 5.3 Verdict modèle

La décision `34c77ce` §10.1 "rester Opus 4.6" est **correctement
fondée** sur les données. L'avantage SWE-bench de 4.7 ne compense
pas :
1. La régression MRCR qui rend le modèle structurellement inadapté
   aux sessions longues avec process documenté
2. Le read-to-edit ratio qui génère des modifications mal informées
3. La correction retention fail qui empêche le cycle auditor→fix

Opus 4.6 est le bon modèle pour SBFB jusqu'à ce qu'un modèle futur
combine les gains SWE-bench de 4.7 avec la stabilité long-context
de 4.6.

Opus 4.5 n'est plus une option (deprecated). La question est close.

---

## 6. Bilan chiffré — impact des propositions

| Métrique | Avant (tip `34c77ce`) | Après propositions | Delta |
|---|---|---|---|
| README.md lignes | 2242 | ~2050 | -192 |
| TOOLING.md lignes | 680 | ~560 (drop §5.2 C1-C9 détail) | -120 |
| Hook phase-auditor-gate.sh lignes | 175 (committed) | ~55 (sans C1-C9) | -120 |
| Agent prompt lignes | 516 | ~510 (G5 refs cleanup) | -6 |
| Tokens process contexte (estim) | ~80k | ~70k | -10k |
| Findings échappés (S23 pattern) | 1 P1 | 0 (audit inconditionnel) | -1 P1 |
| Temps auditor/sprint | ~30 min (S23, 1 phase auditée) | ~180 min (6 phases × 30 min) | +150 min |
| Coût token auditor/sprint | ~95k (1 phase) | ~570k (6 phases) | +475k |

Le trade-off net : -192 lignes README, -120 lignes hook, +475k tokens
auditor/sprint, +1 P1 catché pre-commit au lieu de post-sprint. Le
coût token est acceptable (abonnement solo), le coût temps est
absorbable (~30 min/sprint = 5 min/phase).

---

## 7. Ce que ce document ne recommande PAS

- **Nouveau garde-fou G10+** : les 8 garde-fous existants (G1-G4,
  G6-G9) couvrent le cycle complet kickoff→code→commit→audit. Aucun
  gap identifié qui justifie un nouveau gate. L'anti-pattern est
  l'accumulation, pas le manque.

- **Réécriture du README** : le document est long mais chaque
  section a un rôle. Compacter les sections identifiées (§4.2
  ci-dessus) économise ~192 lignes sans perdre de couverture. Une
  réécriture complète risque le même pattern que le bankruptcy
  (drop accidentel de règles valides).

- **Migration Opus 4.7** : les données sont claires (§5).

- **Nouveau hook process** : les 3 hooks restants (verify-on-write,
  phase-auditor-gate, phase-precommit-lightcheck) couvrent les 3
  moments critiques (write, pre-commit, staging). Ajouter un hook
  "orphan detector" pour le problème §3.2 serait de
  l'over-engineering — un grep mécanique suffit.

---

*Fin du rapport. L'utilisateur décide quoi appliquer et dans quel
sprint.*
