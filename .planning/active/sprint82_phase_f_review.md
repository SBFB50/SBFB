# Sprint 82 Phase F — Review

Date : 2026-07-14. Review Workflow ultracode 11 agents (7 dimensions +
4 vérifications adversariales sur findings P2 ; ~969k tokens) sur le
working tree non committé, complétée par les corrections in-phase et
leurs re-vérifications machine main-thread. Diff final : 9 fichiers
(scripts/check-frontier-contracts.sh ; crates/nexus-core-rs/src/schemas/
{task_response.rs, task_response.schema.json régénéré, mod.rs} ;
crates/nexus-core-rs/src/verification.rs ;
crates/nexus-worker-core/src/llm/mod.rs ; docs/rust/PATTERNS.md ;
docs/shell/PATTERNS.md) + artefact preflight.

## Verdict: PASS

Verdicts dimensions : 3 PASS (gate-machine, scope-carries, security-deep)
+ 4 CONCERN (diff-integral, semantics-truth, livrables-plan,
patterns-regress). **0 P0, 0 P1.** Post-adversarial : 2 P2 CONFIRMÉS +
4 P3 — **tous corrigés in-phase** (détail §Findings), re-vérifiés
machine. Codex GPT-5.6 Sol round 1 : 7 CONFIRMÉ / 2 PARTIEL / 0 GAP ;
les 2 PARTIELs (même défaut : fixture until double-matchée) corrigés
puis re-prouvés machine — cf. §Codex reconciliation. Promu PASS.

## Fond vérifié — ce qui est correct

Chaque preuve re-jouée indépendamment par les reviewers (pas de
confiance aveugle envers l'orchestrateur) :

- **Rouge-avant-vert** reproduit : le motif R1 sur HEAD (pré-réécriture)
  flague EXACTEMENT {task_response.rs:14,84,93,95,100 ;
  verification.rs:31 ; llm/mod.rs:29} = 7 sites / 3 fichiers, identique
  à la cible PLAN-ADAPT ; 0 hit sur le working tree final.
- **Gate exit 0** (GNU Git Bash + BusyBox `bash:5` Docker — 2 runs, le
  second sur le script final à 3 fixtures) ; sweep 4 branches neuves sur
  tout crates/+web/src = 0 faux positif ; check-sharding-docs +
  check-factory-docs exit 0.
- **Self-test fail-closed prouvé** : mutations vacant / malformé
  (`until (Sprint [0-9]+` → grep exit 2 dans la condition `if !` →
  fail=1 sans abort set -e) / sur-large → 3/3 détectées. Post-correction
  P3-3 : mutation PER-BRANCHE 3/3 (chaque famille neuve supprimée fait
  crier exactement sa fixture — until-token / when-verb /
  capability-noun).
- **0 bump wire** : TASK_RESPONSE_VERSION=1 (:48) + DOMAIN_TAG (:54) +
  struct/attributs serde intouchés ; diff snapshot = descriptions-only
  (vérifié structurellement : 0 ligne hors `"description"`, required/
  $defs/types/default inchangés, tri des clés stable, newline finale) ;
  grammaire backends = capacité DORMANTE en prod (schema=None,
  `engine/runtime.rs:1435-1447`).
- **Vérité factuelle des réécritures** (dimension 3, tout TRUE au
  disque) : `tool_calls` sans consommateur vivant (définition + tests
  seulement) ; CAPABILITY_TOGGLES.md:42 gate `tool_calling` OFF ;
  digest = `blake3_hash(model.as_bytes())` (name-only, runtime.rs:1230/
  1474/2047) ; « client-side redaction » = couche RÉELLE livrée S21
  (pii_redactor.rs 362 l. câblé guardrails.rs:154, SDK web/src/sdk/pii/) ;
  les 3 exemples architecturaux de llm/mod.rs défendables sans tag sprint
  (ephemeral workers = livré).
- **Carries fermés** : toutes les ancres nommées S79-P2-1 {14,93,95} +
  S80-G-2 {14,84-85,93,95} couvertes + :100 (préflight) ; PROMISE_RE
  élargi à la classe demandée ; toucher fermant complet (rien à moitié),
  compteur 2 reports soldé.
- **Suites** : bloc Rust §7.4 Windows COMPLET vert (fmt / clippy -D
  warnings / **nextest 2099/2099** / doctests / release build) ; après
  corrections review : fmt OK + nextest ciblé nexus-core-rs +
  nexus-worker-core 696/696. Delta tests Rust = 0 (le self-test est du
  bash ; count nextest invariant vs baseline post-E 2099).
- **T1 = N-A** (0 fichier web/) ; **T2 = N-A** (docs-contrat pure) ;
  critère machine du plan : les DEUX moitiés prouvées. 0 dep, 0 route,
  0 frontière neuve (registre FRONTIER intact, tag ShardPlan vivant) —
  frontier_closure N/A tient. §P70 énumère fidèlement les 13 branches.
- Langue conforme (code/PATTERNS anglais, planning français) ; 0 emoji.

## Findings consolidés post-adversarial (tous traités in-phase)

1. **P2 CONFIRMED — jumeau stale `test_schema_snapshot_matches_struct`
   dans `schemas/mod.rs:30`** (repéré par 4 dimensions ; adversarial :
   1×CONFIRMED-P2, 2×DOWNGRADED-P3). Le fix :37 laissait son jumeau
   exact un fichier plus loin, seule occurrence source restante.
   **CORRIGÉ** : mod.rs:30 renommé → `schema_snapshot_matches_struct` ;
   `git grep test_schema_snapshot_matches_struct` sur crates/ = 0 hit.
2. **P2 CONFIRMED — §P30 réaffirmait « Sprint 22 (tool-calling sandbox +
   wasmtime jail) »** (`docs/rust/PATTERNS.md:1543-1545` +
   `docs/shell/PATTERNS.md:1868-1870`) alors que le commentaire réécrit
   de llm/mod.rs:48 pointe dessus — contradiction interne aiguisée par
   le diff, et wasmtime = mécanisme jamais décidé (deny.toml ban).
   **CORRIGÉ** : les 2 passages réécrits au passé immuable (redaction
   S21 livrée ; tool-calling déféré OFF, pointeur CAPABILITY_TOGGLES ;
   wasmtime jamais câblé, ban préemptif).
3. **P3 — fixture positive double-matchée** (branches until+activates)
   → la délétion silencieuse des branches when/noun restait indétectée.
   **CORRIGÉ** : 3 fixtures positives, une par famille de branche neuve ;
   mutation per-branche re-testée 3/3 ; §P70 reformulé à l'exact.
4. **P3 — asymétrie champ `arguments`** (« the tool registered in the
   allow-list » présupposait un registre actif à côté du champ `name`
   qui le nie). **CORRIGÉ** : délégation au tool author sans
   présupposer de registre ; snapshot RE-régénéré (diff toujours
   descriptions-only, 4 descriptions).
5. **P3 — sous-classe FP non documentée** : token `S<digit>` NON-sprint
   (« until S3 responds » AWS, « S5 activates » section) matcherait.
   **CORRIGÉ** : clause ajoutée à l'en-tête du script + §P70 (0 usage
   in-repo, S<n> lit toujours Sprint).
6. **P3 — édition in-passing llm/mod.rs:45-48 hors périmètre littéral
   du préflight** (item 5 ne nommait que :28-33). Jugement adversarial :
   DÉFENDABLE (même fichier, même classe, laisser « Sprint 22
   tool-calling sandbox » comme couche vivante serait contradictoire).
   **ACCEPTÉ tel quel + consigné** : à citer explicitement au commit
   body comme hygiène in-passing (avec task_response.rs:37 + mod.rs:30).

## Vérification adversariale des findings

4 passes adversariales (une par dimension à findings P2) : 4 CONFIRMED,
2 DOWNGRADED (P2→P3, calibrage), 0 REFUTED, 0 UPGRADED. Imprecisions
mineures des reviewers corrigées par leurs vérificateurs (hits
hors-source .planning/examples dans les greps ; fn à :282 pas :278 —
drift bénin du préflight, le fix cite par nom).

## Résiduels acceptés (0 action, consignés)

- Branches sandbox/allow-list/activates non tense-anchored + sous-classe
  S<digit> non-sprint : documentées en-tête script + §P70, 0 collatéral
  arbre courant (gate vert le prouve), review arbitre.
- Self-test et garde ShardPlan silencieusement supprimables (pas de
  méta-garde) : niveau d'exigence identique à la garde existante — bar
  accepté, un diff qui les supprime est visible en review.
- Classe « post-SN / for SN (frozen) / reserved for SN » (~15 occ. +
  3 *.schema.json sharding) : présent-vrai (scope-cuts encore ouverts),
  hors classe STALE-PHASE-K, documentée hors-gate au §P70 → audit
  Track K.
- Prose « client-side » pour la redaction (pii_redactor vit côté
  coordinator) : pré-existant, hors mandat du scrub Phase F.

## P2/P3 à documenter au commit body

- Hygiène in-passing (3 sites hors hit-set gate, même classe) :
  task_response.rs:37 + schemas/mod.rs:30 (réf fn morte) +
  llm/mod.rs:45-48 (§P30 pointer coherence).
- §P30 rust+shell PATTERNS re-cadrés au passé immuable (suite du
  finding 2, même thème dette-docs).
- Snapshot task_response.schema.json régénéré ×2 (4 descriptions), diff
  descriptions-only, 0 bump wire.
- Delta tests : 0 nouveau test Rust (self-test = bash) ; nextest
  Windows 2099 invariant.

## Codex reconciliation

Rapport brut : `sprint82_phase_f_codex_review.md` (output `codex exec
-m gpt-5.6-sol -c model_reasoning_effort=max -o`, non réécrit).
Round 1 : **9 livrables — 7 CONFIRMÉ, 2 PARTIEL, 0 GAP.**

Les 2 PARTIELs = le MÊME défaut réel, démontré par Codex via mutation
en mémoire par-branche ISOLÉE (plus fin que notre mutation par-famille
groupée) : la fixture positive n°1 (« until Sprint 22 activates the
gate ») double-matchait les branches `until` ET `activates` — supprimer
l'une OU l'autre seule restait indétecté (UNTIL_FAMILY STILL_MATCHES /
ACTIVATES_BRANCH STILL_MATCHES) ; et §P70 sur-affirmait donc la
garantie per-famille (livrables 2 + 8).

**Correction post-Codex (2 fichiers)** : fixture 1 → until PUR
(« promise: tool_calls stay inert until Sprint 22 », aucun verbe) +
4e fixture activates PUR (« promise: S25 activates the pump ») ;
commentaire script + §P70 reformulés à l'exact (« four positive
fixtures — each matching exactly one new branch, no overlap »).

**Re-preuve machine post-correction** : gate exit 0 (GNU + BusyBox
`bash:5` re-run) ; mutation ISOLÉE de chacune des 4 branches neuves →
4/4 détectées (chaque drop fait crier exactement sa fixture) ; preuve
0-overlap : chaque fixture matche exactement 1 branche neuve et 0
branche historique. Le claim §P70 est désormais vrai au sens fort.

Autres points Codex : les 2 artefacts planning untracked signalés au
livrable 9 sont attendus (preflight + review de la phase, committés
avec elle). Aucun autre écart. Boucle arrêtée round 1 (critère : 0
P0/P1, PARTIELs corrigés + re-prouvés, memory
feedback_codex_loop_stop_criterion).
