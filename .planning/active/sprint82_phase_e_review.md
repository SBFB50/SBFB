# Sprint 82 Phase E — Review (Workflow)

Réconciliation des ledgers de dette (S79/S80/S81) + purge des zombies
Python (D9) : re-audit LIVE par item des trois ledgers, purge de 15
zombies shell via tombstones `docs/DEPRECATED.md` (jamais `git rm`
muet), résolution de la collision d'ID T15/T16, exclusion + re-ancrage
de T49 (v4→v1, `:131→:183`), nettoyage des steps Python morts de
`verify.sh` (21→16), extensions D9 constatées au codage (`setup.sh`,
`.githooks/post-merge`, `CONTRIBUTING.md`) tombstonnées, bannière
SUPERSEDED (PO-9) au blueprint workflow-engine, correction du décompte
8/11-vs-4/10. Diff **100 % docs/planning/scripts sur 12 fichiers
d'implémentation** (+ cet artefact de review et l'artefact Codex ; 0
fichier `crates/`, 0 `web/src`, 0 bump wire, 0 dep, invariant
`heberger != publier` intact, aucune frontière de sécurité réouverte).

Review ultracode = Workflow 10 agents (7 dimensions en parallèle +
vérification adversariale des findings + synthèse). Date : 2026-07-14.
HEAD à l'entrée : `c7b6790` (Phases A-D + T2 DONE). Le Workflow rend
**PASS-PENDING** : **0 P0/P1** ; après verdict adversarial, **1 P2 tenu**
(citation load-bearing fausse `61412bb`) + **3 P3** (miscomptes) + une
grappe de NIT doc-honnêteté cosmétiques. **0 statut de ledger réfuté**,
tous les critères machine re-joués verts, purge exacte (15 zombies, ni
plus ni moins), grep-history intégralement préservé.

## Verdict: PASS

Review Workflow OK côté fond (0 P0/P1 ; les critères machine — `git
ls-files packages/=0` et `nexus/=0`, 0 collision d'ID, ancres OPEN
résolvables, `bash -n verify.sh` OK, purge byte-exacte — re-joués verts
et confirmés par l'adversarial). Le PASS-PENDING initial a été **promu
PASS après la réconciliation Codex** (cf. section finale) : le P2 de
citation et les 3 P3 de miscompte de cette review ont été **corrigés
in-phase AVANT le lancement de Codex** (dispositions ci-dessous), puis
Codex round 1 (0 P0/P1 ; 3 P2 + 8 P3) a été réconcilié — tous ses P2 et
la quasi-totalité de ses P3 également corrigés in-phase, le reste
documenté au commit body.

### Dispositions post-review (fixes appliqués avant Codex)

1. [P2] Citation S80-G-1 : les deux `61412bb:95-99/:519` remplacés par
   l'ancre du fichier VIVANT `sprint81_kickoff.md:95-96` (archive v2.1,
   vérifiée par grep — « DOC-LINT-SEMANTIC (S80-G-1) → ACCEPT-AND-CLOSE
   acté »). Note : la review situait le passage à `:53-57` DANS LA
   VERSION au commit `61412bb` ; le fichier archivé vivant l'a à
   `:95-96` — l'ancre committée cible le fichier vivant, seul
   grep-résolvable aujourd'hui. FIXED.
2. [P3] « 6 in-gate dcc3eea » → « 4 CLOSED in-gate `dcc3eea`, 4 CLOSED
   par `50f05c1`, 5 ACCEPT-DOCUMENTED ». FIXED.
3. [P3] « OPEN à ancre vivante (9) » → « (10) ». FIXED.
4. [P3] Preflight « 240 fichiers » → « 247 ». FIXED.
5. NITs sur textes ajoutés : tombstone T16(a) reformulé
   « content-type dérivé serveur, extension-first + fallback
   magic-bytes » ; tombstone zone S9 précise « T23 (Sprint 10, formerly
   after P23 below) » ; temporalité bannière SUPERSEDED « consignation
   roadmap = livrable Phase T » ; §1 réconciliation aligné sur la
   réserve preflight (« levant la réserve ») ; asymétrie P1 §2/§3
   comblée (note S80-K-1 CLOSED `2c85b28`). FIXED.
6. NITs restants NON corrigés (routés) : prose historique H-3
   `rust/PATTERNS.md:~862` (setup.sh au présent S9-era + réf README
   §4.3) = récit passé immuable par convention → candidat balayage
   Phase H ; naming T15b/T16b en prose planning = convention d'artefact
   documentée. Documentés au commit body.

## Fond vérifié — ce qui est correct (aucune régression, 0 perte)

- **Purge EXACTE** : les 15 blocs shell supprimés = exactement
  {T15a, T16a, T17, T18, T20, T21, T22, T23, T44-T48, T50, T51}. Rien
  d'autre n'a disparu (grep des en-têtes `### T` restants : 37 shell +
  8 rust). T24-T27 (deploy VPS) conservés légitimement OPEN.
- **Collision T15/T16 résolue** par suppression des zombies T15a/T16a
  (l'original avait littéralement DEUX `### T15` et DEUX `### T16`) ;
  chaque numéro est unique post-purge (`uniq -d` = vide).
- **Tombstones `docs/DEPRECATED.md`** : 15 lignes shell + les scripts,
  chacune avec rationale + pointeur `git show c7b6790:...` — mécanisme
  README §6.2.1 respecté, jamais de suppression muette. Attributions de
  sprint (S9/S10/S14) toutes conformes aux corps d'origine.
- **T49 exclu + re-ancré** OPEN (ancre Rust vivante) : `publish.rs:24`
  = `PROJECT_ANNOUNCEMENT_VERSION = 1`, reject `ann.v != VERSION` à
  `:183` dans `from_gossip_bytes` (fn `:181`) — re-ancrage `:131→:183`
  exact et conforme à la pre-launch policy.
- **Re-ancrages code exacts** : rust-T23 `Dockerfile:19`
  (`FROM rust:1.94-slim-bookworm`, aucun `@sha256`), `blob_serve.rs:215`
  = `detect_content_type`.
- **`verify.sh`** : 16 steps contigus, `bash -n` OK, `set -euo pipefail`
  intact, 0 résidu `packages/`/`uv`/`pytest`/`ruff` (seul « Python »
  restant = commentaire d'explication), refs internes renumérotées
  cohérentes, ordre fail-fast + étapes sécurité préservés.
- **`CONTRIBUTING.md`** réconcilié (arbre `packages/` retiré,
  `examples/` ajouté, section Python supprimée, « 18 »→« 16 steps »).
- **rust-T20 INTANGIBLE** : header `### T20` UNTOUCHED (hunks rust
  limités à T21 et T23), ID+OPEN préservés, defer RÉAFFIRMÉ MOTIVÉ
  consigné §5.2 ; `THREAT_MODEL.md`/`HARDENING_ROADMAP.md` non touchés.
- **Bannière SUPERSEDED (PO-9)** ajoutée au `verification_blueprint.md`
  (blockquote seule, +11/-0), le `.json` compagnon intact (donnée brute).

## Dimensions (findings consolidés post-adversarial)

### Dimension 1 — Diff ligne à ligne (erreurs factuelles, incohérences) : CONCERN
- **[P3 — downgradé de P2 par l'adversarial]** Auto-contradiction
  prose-vs-table en §3 « recompte CONFIRMÉ » :
  `sprint82_phase_e_ledger_reconciliation.md:81-82` écrit « 6 in-gate
  `dcc3eea`, 4 par S81 Phase G `50f05c1` », alors que sa propre table
  (§3, l.63-79) n'attribue que **4** CLOSED à `dcc3eea` (S80-F-1, A-1,
  E-3, J-1) + 4 à `50f05c1` (H-1..H-4), la ligne tally 80 disant « 8
  CLOSED ». 6+4=10 est arithmétiquement impossible contre la table.
  `git show --stat dcc3eea` confirme 4 fixes docs. La contradiction est
  RÉELLE (fait confirmé) mais de classe cosmétique (même classe que
  l'off-by-one unanime), 0 impact machine — d'où le downgrade P2→P3.
  Fix : « **4** in-gate `dcc3eea`, 4 par `50f05c1`, 5 ACCEPT-DOCUMENTED ».
- **[P3]** Off-by-one §5.1 : « OPEN à ancre vivante (9) » énumère 10 IDs
  distincts (T1, T6, T7, T13, T24-T27 = 4, T16b, T49). Vérifié par tally
  croisé (52 en-têtes shell = 24 CLOSED-antérieurs + 1 SUPERSEDED[T41] +
  15 purgés + 1 CLOSED[T15b] + 1 CLOSED[T14] + 10 OPEN-vivante).
  Fix : « (10) » l.139.
- **[NIT]** Traitement asymétrique du palier P1 : le ledger S79 (§2)
  inclut P1-1, le ledger S80 (§3) omet le P1 K-1 (pourtant existant,
  CLOSED in-gate `2c85b28`, non-carry — rien d'orphelin). Fix optionnel :
  note d'une ligne en §3.

### Dimension 2 — Livrables vs Plan + PLAN-ADAPT : PASS
Tous les livrables du plan §Phase E présents ; les 11 notes PLAN-ADAPT
du preflight respectées (T49 exclu/re-ancré, compte 60 pas ~80, re-audit
3 ledgers jamais swap, tombstone jamais `git rm` muet, collision par
suppression, rust-T20 ID+OPEN, 15 zombies, compteurs croisés §6.2.1,
`verify.sh` 4-8 + T15b même commit, §6 audit_plan + staging SUPERSEDED).
Extensions D9 (`setup.sh`, post-merge, CONTRIBUTING) toutes tombstonnées.
Critères machine du plan vérifiés.
- **[NIT]** Même off-by-one « (9) vs 10 » (cf. Dim 1).
- **[NIT]** Bannière SUPERSEDED / §8 datent l'amendement roadmap « acté
  Phase T S82 » alors que Phase T (wrap-up) n'est pas encore jouée : la
  décision PO-9 (2026-07-11) est actée, sa **consignation roadmap** est
  future. Fix : « décision PO-9 actée 2026-07-11 ; consignation roadmap
  prévue Phase T ».

### Dimension 3 — Exactitude factuelle des claims : CONCERN
- **[P2 — TENU par l'adversarial]** Citation load-bearing fausse pour
  l'accept-and-close de S80-G-1 (doc-lint), citée à DEUX sites
  (`sprint82_phase_e_ledger_reconciliation.md:73` et `:200`) :
  `61412bb:95-99/:519`. Vérifié : `git show
  61412bb:.planning/active/sprint81_kickoff.md` = 488 lignes → **`:519`
  n'existe pas** ; les lignes 95-99 sont du contenu bi-axe/sharding sans
  rapport ; le vrai « ACCEPT-AND-CLOSE / l'item sort des carries » est
  aux **lignes 53-57** (:57). Un vérificateur qui suit l'ancre tombe sur
  une ligne inexistante — viole le contrat auto-imposé de la phase
  (« chaque item cite l'état vérifié »). Fix : remplacer les deux
  occurrences par `sprint81_kickoff.md:53-57`.
- **[P3 — TENU]** Preflight « 240 fichiers » pour `49782a9` (suppression
  Python S51 Phase A) : `git show --stat 49782a9` = **247** files changed
  (le « −72k LOC » est correct). Non-bloquant (la purge n'en dépend pas)
  mais la tâche demandait de vérifier ce nombre. Fix : « 247 fichiers ».
  Réf : `sprint82_phase_e_preflight.md:88`.
- **[P3]** Même off-by-one « (9) → 10 » (cf. Dim 1).
- **[NIT]** Contradiction de framing entre reconciliation §1 (« re-compté
  ligne à ligne indépendamment ») et la réserve honnête du preflight
  (`:260-262` : le 4/10 « n'a pas été re-compté ligne-à-ligne
  indépendamment »). Le chiffre 4/10 reste EXACT (table
  `sprint80_audit_findings.md:388-389`) ; réconcilié en substance car le
  re-audit par item §2-4 EST le re-comptage promis. Cosmétique.

### Dimension 4 — Sécurité : PASS
5 sous-vérifications propres : aucune frontière réouverte, aucune garde
vivante retirée. rust-T20 INTACT (carry TLS relay vivant préservé,
distingué en 3 endroits du shell-T20 zombie purgé). Purge threat-safe
(0 ID purgé cross-référencé dans `docs/security/*.md`). post-merge
supprimé = rôle rappel seul, le vrai backstop (`.githooks/pre-commit` +
`commit-msg`) INTACT. `verify.sh` : ordre fail-fast + toutes les étapes
sécurité (npm audit, scan-en-strings, check-spdx, 3 gates docs)
présentes. Frontières non affaiblies (le tombstone T16a AFFIRME la
frontière content-type dérivé serveur).
- **[NIT]** Entrée pattern H-3 CLOSED de `rust/PATTERNS.md:862-871`
  décrit encore `setup.sh` + `.githooks/post-merge` (supprimés ce commit)
  au présent, et affirme qu'ils sont « documented in the README … §4.3 »
  — grep confirme 0 référence dans les deux README. Non-sécuritaire
  (H-3 = drift wheel dev, pas une frontière) ; entrée historique CLOSED,
  hors diff Phase E, explicitement exclue par la réserve preflight S2.
- **[NIT]** Tombstone T16(a) « magic-bytes fait autorité » légèrement
  imprécis : `detect_content_type` est extension-FIRST, magic-bytes en
  fallback. La substance sécuritaire (type dérivé serveur, plus l'en-tête
  client) est correcte ; la frontière n'est pas réouverte.

### Dimension 5 — Scope et débordements : PASS
Aucun débordement. Périmètre 100 % docs/planning/scripts confirmé
mécaniquement (0 `.rs/.ts/.tsx`, 0 `crates/`, 0 `web/src/`). Les 3
extensions constatées au codage (`setup.sh`, post-merge, CONTRIBUTING)
sont de MÊME classe D9 que les steps `verify.sh` du plan, documentées
comme décisions §5.3 + tombstones dédiés. Aucun ticket hors-thème D10
codé (les 8 routés « slot rig-chaud S83+ » sans les toucher). Aucune
décision Day-0/PO re-débattue (purge Python, `*_ANNOUNCEMENT_VERSION=1`,
PO-9, PO-5 toutes réaffirmées, pas ré-ouvertes). Scope Phases F/G/H/I/J
non anticipé (vérifié fichier par fichier).
- **[NIT]** post-merge purgé sous étiquette « zombie Python » alors que
  son grep déclencheur matche aussi `crates/nexus-core-rs/` (vivant) —
  mais l'action unique du hook (rebuild wheel `nexus_core`) EST morte et
  `DEPRECATED.md:47` énonce le vrai rationale. Aucune action.
- **[NIT]** rust-T20 « décision fermante MOTIVÉE » consignée Phase E, le
  re-ancrage du pointeur laissé à Phase H (S81-C-3) : split cohérent et
  conforme à la spec. Aucune action ; vérifier au préflight Phase H que
  C-3 re-pointe le cross-ref sans re-litiger le defer déjà acté.

### Dimension 6 — Critères machine (re-run) + doc-honnêteté : PASS
Tous les critères machine re-joués verts indépendamment :
`git ls-files packages/`=0 et `nexus/`=0 ; `### T` shell=37 / rust=8 ;
collision `^### T15 `=1, `^### T16 `=1, T15a/T16a résiduels=0 ; 0 header
zombie shell résiduel ; `bash -n verify.sh` OK + renum 1-16 continue ;
`check-{sharding,frontier,factory,spdx}` exit 0 ; chaque `### T` OPEN
pointe un fichier existant. Ancêtres CLOSED tous vérifiés
(`git merge-base --is-ancestor`). Honnêteté explicite (réserves §5.2,
§6.5, note migration-vs-resolution).
- **[NIT]** Naming « T15b »/« T16b » en prose vs en-têtes réels
  `### T15`/`### T16` (désambiguïsés « (a)/(b) » dans DEPRECATED.md) —
  cosmétique, convention documentée.
- **[NIT]** Entrée historique H-3 `rust/PATTERNS.md:862` au présent sur
  `setup.sh` supprimé (même que Dim 4) — explicitement hors-scope par la
  réserve preflight S2, tracé candidat balayage doc futur.

### Dimension 7 — Contrat grep-history + cohérence docs-process : PASS
Contrat grep-history intégralement préservé : les 15 IDs shell purgés
tous retrouvables dans `DEPRECATED.md` avec rationale + pointeur
`git show` ; tombstones in-file couvrant leurs zones (S9-S10 : 8 IDs ;
S14 : 7 IDs = 15/15) ; mécanisme README §6.2.1 respecté en lettre ;
corrections de décompte = notes datées+attribuées (jamais réécriture
silencieuse) ; bannière SUPERSEDED + `.json` intact ; **aucun
commentaire-promesse futur** (anti STALE-PHASE-K respecté). 0 orphelin
dans le ledger vivant.
- **[P3]** Même off-by-one « (9) → 10 » l.139 (défaut de cohérence de
  tally uniquement, grep-history intact car les 10 IDs sont nommés).
- **[NIT]** Le tombstone de la zone S9 affirme que T23 (Sprint 10)
  « lived in this zone » alors que le bloc T23 (SPDX) siégeait en zone
  S10 (~120 lignes plus bas). T23 reste greppable dans `DEPRECATED.md` +
  nommé — contrat grep-history tenu. Fix cosmétique optionnel.

## Vérification adversariale des findings

| Finding | Statut adversarial | Sévérité finale | Disposition |
|---|---|---|---|
| Citation `61412bb:95-99/:519` inexistante pour l'accept-close S80-G-1 (2 sites) — vraie ancre `sprint81_kickoff.md:53-57` | **CONFIRMÉ** (`:519` prouvé absent, fichier=488 l. ; `:95-99` hors-sujet) | P2 | À corriger avant commit (édition texte) |
| « 6 in-gate `dcc3eea` » vs table (4 CLOSED) + tally « 8 CLOSED » | **CONFIRMÉ mais DOWNGRADÉ** P2→P3 (fait réel, classe cosmétique, 0 impact machine) | P3 | Corriger « 4 in-gate `dcc3eea`, 4 par `50f05c1`, 5 ACCEPT-DOCUMENTED » |
| Off-by-one « OPEN à ancre vivante (9) » = 10 IDs (l.139) | **CONFIRMÉ** (4 dimensions concordantes + tally croisé 52 en-têtes) | P3 | Corriger « (10) » |
| Preflight « 240 fichiers » pour `49782a9` = 247 files | **CONFIRMÉ** (`git show --stat`) | P3 | Corriger « 247 fichiers » |

**Aucun finding réfuté ; aucun P0/P1 soulevé ni tenu à aucun stade.**
Bilan adversarial : 3 CONFIRMÉ (1 P2 fausse-citation + 2 P3 miscomptes) +
1 DOWNGRADÉ (P2→P3). Le seul défaut réellement load-bearing est la
citation `61412bb` (P2) ; le reste sont des incohérences de comptage/prose
dans l'artefact de planning central, sans effet sur les critères machine,
le grep-history ou le code. Les NIT échantillonnés (H-3 present-tense +
« documented in README §4.3 » faux ; §1 « re-compté indépendamment » vs
réserve preflight) sont correctement notés NIT — hors diff Phase E ou
réconciliés en substance.

## P2/P3 à documenter / corriger au commit body

1. **[P2] Citation load-bearing** : remplacer les deux `61412bb:95-99/:519`
   (`sprint82_phase_e_ledger_reconciliation.md:73` et `:200`) par
   `sprint81_kickoff.md:53-57` (phrase « sort des carries » :57).
2. **[P3] Contradiction dcc3eea** : `:81-82` « 4 in-gate `dcc3eea`, 4 par
   `50f05c1`, 5 ACCEPT-DOCUMENTED » (cohérent avec la table §3 + tally 8
   CLOSED).
3. **[P3] Off-by-one** : `:139` « OPEN à ancre vivante (**10**) ».
4. **[P3] Compte fichiers** : `sprint82_phase_e_preflight.md:88` « **247**
   fichiers » (ou omettre le compte exact).
5. **NIT (optionnels, doc-honnêteté)** : asymétrie P1 §2/§3 ; temporalité
   « acté Phase T » → « consignation prévue Phase T » ; §1 « re-compté
   indépendamment » aligné sur la réserve preflight ; H-3
   `rust/PATTERNS.md:862` (status-update ou passé + retirer « documented
   in README §4.3 ») — candidat balayage Phase H ; tombstone T16(a)
   « content-type dérivé serveur, extension d'abord, magic-bytes en
   fallback » ; naming T15b/T16b ; tombstone T23 « lived in this zone ».

## Codex reconciliation

Codex GPT-5.6 Sol (`model_reasoning_effort=max`, CLI 0.144.1) joué en
round 1 sur bundle diff-inline auto-contenu (mode Phase B/D). Artefact
BRUT : `sprint82_phase_e_codex_review.md` (non réécrit). Verdict Codex :
**0 P0/P1, 0 régression code/sécurité** ; « negative hunts » tous verts
(0 bloc non-zombie supprimé, 15/15 IDs greppables avec rationale, T49 +
rust-T20 préservés, 0 `crates/`/`web/src`, 0 dep/wire, phases futures
routées jamais implémentées) ; `GAP{...}` = 3 P2 + 8 P3, tous
d'édition de texte. Réconciliation (critère d'arrêt « CLEAN ou P2/P3
documentés » atteint — pas de round 2 requis, aucun P0/P1) :

- [P2 D8] Taxonomie S79 : P2-7 reclassé OPEN→STALE (artefact archivé
  immuable, non-actionable) + P3-11 OPEN→ACCEPT-DOCUMENTED (recoupe
  P2-5) + ancres explicites ajoutées aux OPEN P3-8/9/10 ; tally S79
  recompté 3/2/3/5/7 ; la phrase « 9 OPEN mitigés CSP » remplacée par
  la décomposition honnête (2 gate-sécurité CSP-mitigés + 5 dettes
  doc/lint ancrées). CORRIGÉ.
- [P2 D8] §4 S81 réécrit en **table per-item** (34 findings + carries),
  chaque item avec statut + evidence/route ; **S81-G-2 explicitement
  réconcilié** (ROUTED Phase I/standing, désambiguïsé de S80-G-2→F et
  de G-D5-1). CORRIGÉ.
- [P2 D11] Cette review était stale vs le diff (elle décrivait ses
  propres findings comme restant à corriger) : sections « Dispositions
  post-review » + présente réconciliation ajoutées, verdict promu,
  ancre 53-57-vs-95-96 expliquée (drift de lignes commit-vs-vivant).
  CORRIGÉ.
- [P3 D1] Tuple T14 étiqueté par métrique (lines 86.91≥85, branches
  78.63≥78, functions 85.82≥85, statements 88.23≥85). CORRIGÉ.
- [P3 D3] Entrée verify-steps : pointeur `git show
  c7b6790:scripts/verify.sh` ajouté ; « jamais réutilisés » qualifié
  (T15/T16 numériques restent portés par les tickets S77 — résolution,
  pas réutilisation). CORRIGÉ.
- [P3 D4] Header verify.sh : mention `npx playwright install chromium`
  one-time pour le run complet. CORRIGÉ.
- [P3 D5] Tombstone setup.sh : « échoue au premier sync du workspace
  Python » (précis) au lieu d'« abort à la première commande ». CORRIGÉ.
- [P3 D7] CONTRIBUTING : « core workspaces are Rust + Frontend »
  (nuance : `examples/` peut porter des sources d'app en tout langage).
  CORRIGÉ.
- [P3 D8] §9 réconciliation : résultat `check-spdx.sh` ajouté. CORRIGÉ.
- [P3 D9] Dates calendaires `2026-07-14` ajoutées aux 3 notes de
  correction (audit_plan ×2, sprint81_audit_findings). CORRIGÉ.
- [P3 D11] « 12 fichiers » précisé « 12 fichiers d'implémentation +
  artefacts review/Codex ». CORRIGÉ.
- Non repris (documentés commit body) : prose historique H-3
  `rust/PATTERNS.md:~862` (passé immuable → balayage Phase H) ; la
  nuance Codex sur post-merge (« grep déclencheur matche aussi
  nexus-core-rs vivant ») était déjà couverte par le NIT Dim 5 — le
  rationale DEPRECATED est le bon.

Séquence stricte respectée : review PASS-PENDING → fixes → Codex →
réconciliation → **PASS** (ce document) → commit.
