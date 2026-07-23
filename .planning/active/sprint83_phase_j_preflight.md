# Sprint 83 Phase J — Preflight G8 (orchestrateur local déterministe)

Date : 2026-07-22. HEAD `03c14d84` (Phase I1 DONE). Spec : `sprint83_plan.md:831-877`.
Socle design : research pack §12 (`workflow_agents_app_conception_ultradeep_2026-07-15.md:951-1170`),
kickoff D3/D4. Socle code : E (spike engine + echo + journal), H (CompiledPlan.v1), I (store
durable lease/inbox/snapshots/GC/backup), I1 (routes daemon lecture seule).

Méthode : Workflow ultracode `wf_9f2458a1-0ca` — fan-out 5 scans factuels (S1a prior-art OSS
profond, S1b deps/CVE, S2 décisions historiques, S3 threat model, S4 wire format) + lentilles
adversariales (exactitude, cohérence-design, complétude). 6/8 agents Workflow Opus 4.8 1M
(946k tokens, 166 tool-uses) ; S3 + complétude rejoués en fallback Agent avant-plan texte
(cap StructuredOutput — piège documenté Phase I). Synthèse main-thread.

## Verdict G8 : PLAN-ADAPT

Aucun DESIGN-CONFLICT : les 5 scans + 3 lentilles confirment que le design gelé (D3
mémoïsation-de-steps, D4 Plan-Then-Execute, §12.3/12.6/12.7, single-writer, autorité = nœud
initiateur WA-3) est réalisable sur le socle E/H/I sans bump wire ni dep runtime. La passe
exactitude n'a RÉFUTÉ AUCUN finding (tous les fichier:ligne résolvent à HEAD). Le code suit
les adaptations §4 ci-dessous, qui TRANCHENT notamment les 3 trous P1 d'exécutabilité
(fixtures goldens C-1, identité runtime map/repeat C-2, migration M3 C-3) découverts par la
lentille complétude — chacun résolu DANS le cadre gelé, aucune Day-0 touchée.

## 1. Base mesurée à HEAD

- **Gap central (S2-F1, P1)** : le moteur E consomme un `RunSpec` LINÉAIRE
  (`engine.rs:44-51`, `execute` `:258`) ; H produit un `CompiledPlan` GRAPHE
  (`compile.rs:572-626` : `node_instances` BTreeMap, `edges` triés canoniquement — l'ordre
  est effacé, `entrypoints` par scope déterministes, `human_gates`, budgets, politiques
  error/timeout/cancellation par scope). `grep CompiledPlan` dans engine/src = 0 import.
  Le scheduler graphe de J est GREENFIELD ; les primitives durables E/I sont RÉUTILISÉES,
  jamais réécrites.
- **18 node-kinds gelés** (`graph.rs:486-541`) vs 9 exécuteurs nommés en J.2 — J dispatche
  les 18 : control-flow structurels (switch/classifier/parallel/map/join/repeat/subworkflow/
  wait.signal/human.review) + feuilles routées vers l'echo-target E
  (input/transform/output/model.invoke/agent.invoke/agent.as_tool/handoff/capability.call/
  knowledge.query). Adaptateurs réels = K/L.
- **RunEventKind = 5 variantes** (`run_event.rs:87-119`), enum FERMÉ `deny_unknown_fields`
  (`:86`), enveloppe versionnée une fois + upcasters (`RUN_EVENT_FORMAT_VERSION=1`).
  Wall-clock ABSENTE du journal (`run_event.rs:135-141`) — les tables d'état la portent.
- **RunStatus store = 3 états** (`store.rs:201-208` Running/Completed/Terminal) ; la route
  daemon I1 projette 3 bras (`workflow_api.rs:116-122`, `REFERENCE.md:51-55`). Les 11 états
  run / 10 états node §12.3 sont une RÉDUCTION du journal, pas des colonnes.
- **Portes écrivain dormantes** : `record_intent_atomic`/`record_completion` pub(crate) sans
  consommateur spike-faults-OFF (`store.rs:687/865`, warning dead_code per-crate,
  carry I1) — J est le PREMIER écrivain. Fencing lease epoch CAS monotone
  (`store.rs:1266-1338`, `LeaseLost` `:324-332`) : chaque write-txn ré-asserte (owner,epoch).
- **Crate clockless et sync** : `now_ms` injecté partout (`store.rs:74-75`), 0 async dans
  l'arbre (`cargo tree -e normal` grep tokio|crossbeam|futures|mio = vide). Versions lock :
  rusqlite 0.36.0, blake3 1.8.5, serde_jcs 0.2.0 — 0 nouveau lock node requis.
- **Census DOMAIN = 29 figé** ; `DOMAIN_COMPILED_PLAN_V1` et `DOMAIN_RUN_EVENT_V1` réservés
  0-producteur (`canonical.rs:386-404`).
- **Goldens compilés** : les 4 contiennent des nodes bloquants (human.review
  factory:103/ideas:92, wait.signal babel:18, classifier ideas:176, switch babel:80) ;
  lecteur test-support `golden_store.rs` côté core (extrait en H).

## 2. Passe adversariale — résultat

0 finding réfuté (lentille exactitude, tout fichier:ligne résolu à HEAD). Notes d'ancrage
mineures sans effet : S2-F5 citait `compile.rs:624` pour join (réel : `graph.rs:523` ;
le finding tient sur le body H verbatim « joins single-level … J re-observe les arrivées ») ;
off-by-one `engine.rs:46-52`→`44-51`. Les faits prior-art S1a (DBOS `operation_outputs`/
`DBOSUnexpectedStepError`, Restate suspend-by-entry-index/out-of-order completions, Morling
delay-as-remainder, Temporal cancel scopes, LittleHorse NodeRun position) sont des claims
web corroborés par kickoff D3:332-348 — non re-vérifiés dans l'arbre, utilisés comme
inspiration de design, pas comme evidence de code.

## 3. Décisions qui tranchent les trous de complétude (autorité : ce preflight)

- **DÉC-1 (C-1, P1) — Fixtures goldens = script de décisions déterministe.** Chaque golden
  compilé s'exécute jusqu'à `RunCompleted` avec : branches switch/classifier PRÉ-SEMÉES par
  l'input déterministe (l'echo-target renvoie le JCS de l'input → le port de sortie est une
  fonction pure de l'input du node), signaux `wait.signal` et décisions `human.review`
  injectés par le harness via le trait `Signal` (script ordonné, mono-usage). Les fixtures
  prouvent la TRAVERSÉE ENTIÈRE, et un test sépare prouve le PARKING stable
  (waiting_human/waiting_external) + reprise sans réexécution.
- **DÉC-2 (C-2, P1) — Identité runtime = extension pure de l'ID structurel.**
  `node_instance_id` runtime = ID structurel H ⊕ suffixe d'itération encodé (map index,
  repeat itération) via fonction pure documentée ; projection inverse
  `structural_id(runtime_id)` pour les lookups `capability_closure`/`node_budgets`/
  `human_gates` (clés STRUCTURELLES de H). Deux itérations ne partagent JAMAIS une identité
  d'effet (§12.2:994-996). Test dédié projection + unicité.
- **DÉC-3 (C-3, P1) — Migration M3 forward-only.** Tables neuves : timers durables
  (deadline ABSOLUE `expires_at_ms`, jamais une durée — S1a-F4) + signaux/waited-set
  mono-usage (nonce, séquence, expiration §12.6 — l'inbox M2 dédup de receipts ne peut PAS
  les porter). AUCUNE colonne d'état de node (états = réduction du journal). Co-migrent :
  check d'intégrité par version (`store.rs:2009`), validation restore (tables/colonnes/PK),
  `STORE_MIGRATION_OPS` inventaire par classe.
- **DÉC-4 (C-4/S4-A2) — Projection daemon INCHANGÉE (3 états coarse).** `RunStatus` store
  reste Running/Completed/Terminal ; `cancelled` et `outcome_unknown` = `Terminal` +
  `terminal_reason` (déjà projeté par I1) ; les états riches §12.3 restent INTERNES au
  reducer (RunView) jusqu'à ce qu'une route les projette (V). Aucun geste daemon,
  T1 = N-A tenu, REFERENCE.md : note de sémantique `frontier_node` seulement
  (= dernier complété, représentant d'un front multi-nodes — S4-F4).
- **DÉC-5 (C-9) — Timers ≠ timeouts.** J livre les TIMERS durables (deadlines absolues :
  expiration wait.signal, infra pour K) + trait `Clock`. Les TIMEOUTS d'effet/node et les
  retries (`TimeoutPolicy`/`max_retries_per_node` consommés) = Phase K (plan K.1-K.2
  « retries/backoff, timeouts »). J journalise l'expiration comme événement de décision.
- **DÉC-6 (C-7) — Validation payload wait.signal : digest-only en J.** Le payload est
  journalisé + le digest du schéma pinné attendu ; la validation STRUCTURELLE contre le
  schéma (graph.rs:450-461) exigerait un validateur JSON-Schema absent de l'arbre → différée
  (borne honnête au body, PAS de dep nouvelle — 0-dep prime, S1b).
- **DÉC-7 (C-8) — Backpressure = politique d'HÔTE injectée.** Cap in-flight = paramètre du
  scheduler (knob injecté, défaut documenté), admission en ordre CANONIQUE déterministe
  (tie-break `node_instance_id`) — PAS une borne du plan compilé (CompiledPlan n'a pas de
  champ max-concurrency) et PAS un compteur déguisé en borne structurelle. Les bornes
  STRUCTURELLES (max_iterations repeat, max_cardinality map) SONT consommées du plan.
- **DÉC-8 (C-6) — Chemin store prod « résultat tardif ».** API store additive : receipt
  tardif journalisé + outbox résolu SANS avancer le frontier ni ré-étiqueter un état clos
  (absorbance I préservée). Le seul écrivain event-sans-frontier actuel est un red-control
  spike-faults (`store.rs:985-1008`) — inatteignable en prod, un chemin prod neuf est dû.
- **DÉC-9 (C-10) — Acteur réel du test « iframe death ».** L'engine n'a pas d'iframe ; le
  test garde le nom J.3 mais son doc reformule : mort du PROCESSUS writer (pattern child
  E/I) → un NOUVEAU handle reprend le même frontier, 0 double effet. Si un 2e enfant
  `#[ignore]` est requis, le passage +1→+2 skips est CONSIGNÉ au body (précédent E).
- **DÉC-10 (C-11) — Évidence T2.** J livre des tests engine NOMMÉS citables par AI2
  (waiting_human atteint + fin sans réexécution) ; la numérotation « WA-T1.3/.5 » de
  plan:868 est une référence à la liste non-numérotée de gate_matrix:102 — citer les NOMS
  de tests exacts dans le body, pas les numéros. T2 reste NOT-RUN jusqu'à AI2.
- **DÉC-11 (C-12) — Goldens cross-crate.** Réutiliser/exposer le lecteur `golden_store.rs`
  côté core (test-support H) pour les tests engine ; toute édition core est DÉCLARÉE au
  body (commit scope workflow-engine, fichiers listés).
- **DÉC-12 (S1a-A4) — Journal = décisions + effets ; états = dérivation.** Le reducer dérive
  les 10 états de node de la topologie du plan + des événements. Les DÉCISIONS du scheduler
  non déductibles de la topologie (branche prise, signal consommé, timer expiré, cancel
  demandé, node skipped) deviennent des variantes `RunEventKind` NOUVELLES — journalisées
  AVANT usage (mitigation T-RUN-REPLAY-DIVERGENCE). Redéfinition v1 pré-launch, constante
  =1, goldens régénérés EXPLICITEMENT (S4-F7).

## 4. Adaptations que le code DOIT suivre

1. **Scheduler graphe greenfield** consommant `CompiledPlan` (frontier/ready-set dérivés de
   `edges`+`entrypoints` normalisés H, dispatch par `node_instance_id`) ; le SpikeEngine
   linéaire reste pour le spike, il n'exprime aucune construction J (S2-A1).
2. **Écrivain J acquiert le lease** (`acquire_lease`) et route TOUT effet par
   `record_intent_atomic`/`record_completion` — ferme le dead_code per-crate, honore les
   3 clauses E + fencing I ; `LeaseLost` → rollback total, jamais d'émission fenced-out
   (S2-A4, S3-F1, T-RUN-DOUBLE-EMIT-STALE-WRITER `THREAT_MODEL.md:1766`).
3. **Traits `Clock` + `Signal` hôte-neutres NEUFS** — Clock = promotion du pattern now_ms
   injecté (JAMAIS de lecture wall-clock dans le reducer) ; Signal = livraison durable
   dédupliquée mono-usage transactionnelle. Store reste CONCRET (abstraction trait store =
   sur-investissement, précédent S2A-9) (S2-A2, S1b-A1/A3).
4. **Skip-propagation / dead-path** : branche non prise → nodes aval `skipped`, un join
   s'arme quand CHAQUE edge entrant est succeeded OU skipped — jamais d'attente sur une
   branche morte. Test `switch_skips_branch_into_join` (S1a-A1, WCP-30 — le P1 prior-art).
5. **Replay divergent testé** : `deterministic_replay` couvre AUSSI un journal divergent
   fail-closed per-node sous le graphe (pas seulement le happy-path) (S1a-A2).
6. **Timers = deadlines ABSOLUES durables** (rows M3, réduites contre now_ms injecté,
   pattern lease/GC) ; test restart-mid-timer : le timer expire UNE fois à la deadline
   d'origine après replay (S1a-A3, S1b-A2).
7. **wait.signal mono-usage** : waited-set durable keyed node_instance_id + nonce ; un
   signal rejoué/dupliqué est consommé EXACTEMENT une fois, pas de double-resume ; signal
   périmé/inconnu → refus LOUD journalisé, zéro transition. Tests `signal_replay_rejected`,
   `signal_expired_rejected` (S1a-A6, S3-A2).
8. **Régression d'horloge** : `now_ms` < dernier événement → refus/clamp LOUD ; les
   décisions de timer sont JOURNALISÉES (le replay lit le journal, jamais l'horloge live).
   Test `clock_regression_loud` (S3-A2, T-RUN-TIMER-SKEW).
9. **map/join par index encodé** : réduction des enfants en ordre d'INDEX (jamais ordre
   d'arrivée) ; `map_join_cardinality` asserte cardinalité ET ordre déterministe sous
   complétions désordonnées (S1a-A7, §12.2). Joins RÉ-OBSERVÉS au runtime (cardinalité
   runtime, jamais confiance à un compte statique H — S2-A6).
10. **Cancel honnête** : `cancel_requested` journalisé SOUS lease → stop d'émission ;
    résultat tardif conservé (DÉC-8) non propagé ; le run termine `cancelled` JAMAIS
    `completed` sur résultat tardif mid-cancel ; états clos ABSORBANTS au niveau reducer
    (S1a-A8, S3-F3, §12.7). Le wire Task n'a pas de cancel fort — limite AFFICHÉE (doc).
11. **Reprise = mémoïsation, jamais re-run** : `wait_signal_resume` asserte explicitement
    que les effets mémoïsés du node repris sont servis du journal (le crux D3 vs
    replay-total, §12.6:1108-1113) (lentille cohérence).
12. **No-frontier-cache préservé** : le graph walk re-lit le store ; la table `snapshots` M2
    (cache jetable digest-vérifié) est le SEUL cache autorisé, déjà justifié (lentille
    cohérence, engine.rs:16-19).
13. **Bornes structurelles consommées du plan** : `max_iterations` (repeat) et
    `max_cardinality` (map) viennent du CompiledPlan, jamais d'un compteur runtime
    autonome ; « 0 map non borné » (J.4) tenu par construction (lentille cohérence, D4).
14. **Politiques cancel consommées du plan** : `cancellation_policies` par scope
    (compile.rs:625) pilotent le comportement cancel de J — pas de politique inventée au
    runtime (lentille cohérence). Timeout/error policies = K (DÉC-5).
15. **Backpressure testée** : ready-set borné, admission déterministe canonique, test dédié
    (S1a-A5, DÉC-7, T-RUN-BACKPRESSURE-STARVATION).
16. **subworkflow = navigation de scopes H** : l'expansion est STATIQUE (node_instances
    depth + entrypoints par scope) ; J navigue, n'expanse pas. Amplification bornée
    statiquement (T-RUN-SUBWORKFLOW-AMPLIFICATION, T-PLAN-STATIC-BYPASS).
17. **human.review = parking waiting_human** + décision opaque via trait Signal (script de
    test) ; la validation principal/signature/politique = L1, PAS J (S3-F6). Le scheduler ne
    dispatche QUE des nodes du plan compilé (part J de T-PLAN-LLM-MUTATION:1724).
18. **Les 18 kinds dispatchés**, feuilles → echo-target (un golden avec model.invoke ne
    crashe pas) (S2-A3).
19. **Machine d'états §12.3 = réduction** : RunView (11 états run / 10 états node) dérivée
    par le reducer pur ; RunStatus store inchangé (DÉC-4) (S2-A5).
20. **Doc `store.rs:449-455` rendue exacte** : « the daemon path acquires » redevient vrai —
    reformuler : le chemin écrivain RUNTIME acquiert ; les routes I1 restent lecture
    non-fencée (carry I1, S2-A8).
21. **THREAT_MODEL v25 DUE dans le commit J** : amendement §19 (bornes « = J » requalifiées
    LIVRÉ, borne unfenced re-décrite) + 6 rows nouvelles (T-RUN-SIGNAL-FORGE-REPLAY,
    T-RUN-CANCEL-DOS, T-RUN-TIMER-SKEW, T-RUN-REPLAY-DIVERGENCE,
    T-RUN-SUBWORKFLOW-AMPLIFICATION, T-RUN-BACKPRESSURE-STARVATION) + entrée §20
    (S3-F7/A1 — le précédent per-phase v22/v23/v24 l'exige).
22. **Claims bornées au body** : fenêtre effet-externe + receipts + retries/timeouts = K ;
    HITL principal/signature = L1 ; validation payload signal structurelle différée (DÉC-6) ;
    J prépare `outcome_unknown` (TerminalReason existant projeté), ne le complète pas
    (S3-A3, S2-A7).
23. **Tests J.3 ÉTENDUS** (C-5) : aux 8 du plan s'ajoutent branch switch/classifier,
    restart-mid-timer, backpressure, parking human.review + reprise, signal
    replay/expiré/régression horloge, divergence sous graphe, skip-into-join — chaque
    contrôle rouge DISCRIMINANT (mutation avant confiance, memory).
24. **0 bump / 0 dep** : census 29 intact, `DOMAIN_COMPILED_PLAN_V1` reste 0-producteur
    (consommation IN-MEMORY seulement), pas de bump rusqlite (CVE-2025-6965 = QPO-3),
    aucune étiquette FRONTIER sur clock/signal (test-acteur : interne crate, lib.rs:126-137)
    (S4-A1/A3/A4, S1b-A4).

## 5. Interdits

- Réintroduire une lecture d'horloge/random dans le reducer ou itérer un HashMap dans le
  chemin de décision (divergence de replay).
- Bypasser `record_intent_atomic`/`record_completion` ou écrire sans lease.
- Ajouter tokio/crossbeam/validateur JSON-Schema ou toute dep runtime.
- Produire `DOMAIN_COMPILED_PLAN_V1`/`DOMAIN_RUN_EVENT_V1` (sérialisation signée) ou bumper
  `RUN_EVENT_FORMAT_VERSION`.
- Étendre `RunStatus` store / la projection daemon (DÉC-4) ; toucher `SENSITIVE_ACTIONS`.
- Fermer (ou claimer fermer) la fenêtre effet-externe, les receipts, les retries = K ;
  valider le principal HITL = L1.
- Colonnes d'état de node dans le store (états = réduction du journal).
- Édition du spike E au-delà du strict nécessaire (primitives durables réutilisées telles
  quelles ; toute édition core déclarée au body).

## 6. Invariants à vérifier avant commit

- `cargo nextest run -p sbfb-workflow-engine --locked` exit 0 (toutes constructions +
  replay déterministe + divergence + cancel + signal adversarial).
- `cargo clippy -p nexus-shell-daemon --all-targets --locked -- -D warnings` exit 0
  (le dead_code per-crate résorbé par le 1er écrivain — carry I1).
- Suites §7.4 complètes (fmt dual, clippy workspace, nextest workspace Win + Docker,
  doctests, release, web+operator inchangés → re-run quand même, doctrine full-failfast).
- `bash scripts/check-frontier-contracts.sh` clean ; census DOMAIN == 29 ;
  `grep -c ".route(" crates/nexus-shell-daemon/src/http.rs` == 91 (inchangé).
- Goldens `run_event*.golden.json` régénérés EXPLICITEMENT si les traces changent ;
  golden famille §P75 double-run stable.
- Mutations actives sur les gardes neuves (join-skip, mono-usage signal, absorbance
  cancel, divergence) — un test vert non-mutation-tué ne prouve rien (memory Phase E).

## 7. Scope NON couvert par J (frontières honnêtes)

- **K** : effets réels (effect_id/attempt_id/receipts signés), retries/backoff/timeouts,
  réconciliation `outcome_unknown`, fenêtre effet-externe (fencing-token côté cible),
  effets FS/process. J prépare les états, K les complète.
- **L/L1** : capacités 7 couches, coffre secrets, HITL chrome hôte signé (principal,
  politique, digest de vue).
- **V** : timeline UI / Centre des Runs (T1 = N-A-no-frontend-change tenu par DÉC-4) ;
  projection riche des états = `WorkflowPublicProjection.v1` (déféré).
- **Routes daemon mutantes** (restore/GC/lease-force) : rows D7 T1 datées J/K — leur
  implémentation route reste différée (l'écrivain J fournit la MÉCANIQUE, pas la route).
