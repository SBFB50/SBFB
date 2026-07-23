Verdict global : **GAP**. La Phase J possède un socle solide — réduction pure, stockage M3, contrats v1, identités runtime et quatre goldens — mais elle ne peut pas recevoir un `PASS` dans l’état actuel. Les blocages principaux sont fonctionnels : le chemin de cancel peut produire un **nouvel effet après l’annulation**, le reducer accepte des événements causalement impossibles et calcule certains joins avec une sémantique différente du plan compilé, un crash peut perdre définitivement le timer du workflow, et un repeat contenant une portée dynamique imbriquée peut ne jamais avancer.

Je n’ai lancé aucune suite lourde. Les mesures vertes fournies sont donc acceptées comme mesures externes, mais elles ne couvrent pas ces défauts logiques. Aucun fichier n’a été modifié.

## 1. `reducer.rs` — GAP

Le socle demandé est bien présent :

- `reduce(&CompiledPlan, &[RunEvent])` est une réduction pure, sans I/O ni lecture de payload matériel : [reducer.rs:217](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:217>), [reducer.rs:302](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:302>).
- Les machines contiennent bien **11 états run et 11 états node** : [reducer.rs:62](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:62>), [reducer.rs:96](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:96>).
- L’overlay cancel, le gel du ready-set et `fully_settled` existent : [reducer.rs:232](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:232>), [reducer.rs:252](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:252>), [reducer.rs:267](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:267>).
- Map et repeat ne matérialisent pas au-delà des bornes signées : [reducer.rs:335](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:335>), [reducer.rs:625](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:625>).

Mais l’autorité du journal n’est pas fail-closed :

- `SignalConsumed`/`HumanDecisionRecorded`, `BranchTaken` et `EffectIntent`/completion prennent autorité **avant la vérification des inputs**. Un événement forgé peut donc faire réussir ou démarrer un node inatteignable : [reducer.rs:529](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:529>), [reducer.rs:565](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:565>), [reducer.rs:655](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:655>).
- Le join compte l’état global des nodes sources, pas la livraison effective de chaque edge/port : [reducer.rs:675](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:675>). Or le routage par port n’est calculé que dans `edge_delivery` : [reducer.rs:833](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:833>). Un switch/classifier réussi sur un autre port peut donc être compté comme source arrivée au join.
- `All` est redéfini sur les sources « vivantes » : [reducer.rs:702](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:702>), alors que le contrat core définit les sources déclarées et précise que les composites résiduels insatisfaisables doivent bloquer : [graph.rs:349](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/graph.rs:349>), [compile.rs:1911](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/compile.rs:1911>).
- Pour `Any`/quorum, les perdants vivants ne sont jamais éliminés. Comme `fully_settled` exige que tous les nodes soient settled, un perdant externe peut parquer le run indéfiniment, contrairement au contrat `Any` où les autres sources sont abandonnées : [graph.rs:360](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/graph.rs:360>).
- Repeat respecte la borne de matérialisation, mais ne rejette pas les séquences de verdicts illégales : port incorrect, doublons ou `continue` excédentaire sont silencieusement interprétés via le nombre de verdicts : [reducer.rs:625](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:625>).

## 2. `orchestrator.rs` — GAP

Sont confirmés :

- Lease acquis avant les écritures runtime : [orchestrator.rs:280](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:280>).
- Watermark avancé à chaque passe : [orchestrator.rs:341](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:341>).
- Ready-set trié, dédupliqué et borné : [orchestrator.rs:449](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:449>).
- Les 18 kinds ont un chemin de dispatch ; switch/classifier journalisent le choix avant toute consommation aval, map vérifie `max_cardinality` avant `MapFanout`, repeat et subworkflow ont leurs chemins dédiés : [orchestrator.rs:641](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:641>), [orchestrator.rs:688](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:688>), [orchestrator.rs:720](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:720>).
- La reprise d’une feuille `Running` vérifie step hash, target idempotency/scope et produit `OutcomeUnknown` pour une cible non idempotente : [orchestrator.rs:739](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:739>).

Trois défauts importants subsistent :

1. Après `cancel_requested`, `drain_cancelled` rappelle `target.apply()` pour un intent pending : [orchestrator.rs:508](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:508>), [orchestrator.rs:532](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:532>). Un intent est écrit **avant** l’appel cible ; il ne prouve donc pas qu’un effet existe déjà. Ce rappel peut produire le premier effet après cancel. `record_late_completion` n’avance effectivement pas le frontier, mais cela ne rend pas cette émission « tardive ».

2. Création du run et création du timer ne sont pas atomiques. `ensure_run_with_input` commit d’abord, puis le timer n’est programmé que si `created == true` : [orchestrator.rs:305](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:305>), [orchestrator.rs:321](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:321>). Un crash entre les deux laisse un run existant sans timer, et la reprise ne le reprogramme jamais.

3. `scope_settled` parcourt les descendants directs **et imbriqués**, puis n’ajoute que le suffixe runtime de la portée extérieure : [orchestrator.rs:604](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:604>), [orchestrator.rs:1251](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1251>). Pour un repeat contenant un map/repeat, les vrais descendants possèdent un suffixe intérieur supplémentaire ; la portée extérieure peut donc ne jamais être considérée settled.

## 3. `store.rs` M3 — PARTIEL

La migration et les portes principales sont correctement câblées :

- M3 est additive après M1/M2 et ajoute `waits`, `signals`, `timers`, `runs.input_payload` et `clock_watermark` : [store.rs:180](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:180>).
- `open_wait` est idempotent ; `deposit_signal` impose un wait OPEN, déduplique et borne le payload : [store.rs:1717](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1717>), [store.rs:1770](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1770>).
- Consommation, CAS du wait, validation d’expiration/label et événement sont dans une transaction : [store.rs:1861](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1861>).
- Première deadline gagnante et firing gate `Running` : [store.rs:2077](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2077>), [store.rs:2120](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2120>).
- `record_late_completion` n’avance pas le frontier ; terminal et completed sont absorbants : [store.rs:1110](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1110>), [store.rs:1196](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1196>).
- Le restore v3 vérifie colonnes, tables, PK, FK, index et `quick_check` : [store.rs:2682](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2682>).

Écarts :

- `MAX_SIGNAL_PAYLOAD_BYTES` ne borne que le JSON sérialisé ; `decision_label` peut être arbitrairement grand avant insertion : [store.rs:1785](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1785>). Cela laisse une amplification disque non documentée.
- Fermer un run ne ferme pas ses waits. `deposit_signal` ne vérifie pas le statut du run et peut encore stocker un signal sur un wait OPEN après terminal : [store.rs:1815](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1815>), [store.rs:1205](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1205>).
- `record_cancel_requested` est idempotent, mais `fire_timer_with_cancel` possède un chemin distinct. Si un cancel opérateur est déjà journalisé alors que le statut est encore `Running`, le timer peut ajouter un second `CancelRequested`.

## 4. `run_event.rs` core — CONFIRME

- Version maintenue à `1` : [run_event.rs:62](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:62>).
- `TerminalReason::Cancelled` et `CancelOrigin` fermé sont présents : [run_event.rs:67](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:67>), [run_event.rs:100](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:100>).
- Les cinq variantes de décision sont additives : [run_event.rs:158](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:158>).
- Chaîne d’upcast vide et invariant de longueur testés : [run_event.rs:272](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:272>), [run_event.rs:489](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:489>).
- `contracts.schema.json` contient les ajouts correspondants sans suppression des formes v1 : [contracts.schema.json:1407](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/contracts.schema.json:1407>), [contracts.schema.json:1534](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/contracts.schema.json:1534>).

## 5. `runtime_id.rs` — CONFIRME

Le suffixe `~2N` est inatteignable dans un segment structurel correctement échappé RFC 6901, les extensions map/repeat sont pures et la projection inverse ne tranche qu’à une frontière ASCII donc UTF‑8-safe : [runtime_id.rs:18](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/runtime_id.rs:18>), [runtime_id.rs:44](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/runtime_id.rs:44>), [runtime_id.rs:83](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/runtime_id.rs:83>). Les tests couvrent inverse, unicité, nesting, échappement et non-ASCII : [runtime_id.rs:133](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/runtime_id.rs:133>).

## 6. Tests et goldens — PARTIEL

Confirmé :

- Les quatre vrais artefacts `.compiled.golden.json` sont désérialisés comme `CompiledPlan`, exécutés jusqu’à `RunCompleted`, puis rejoués avec croissance nulle : [orchestrator_goldens.rs:38](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/orchestrator_goldens.rs:38>), [orchestrator_goldens.rs:99](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/orchestrator_goldens.rs:99>), [orchestrator_goldens.rs:144](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/orchestrator_goldens.rs:144>), [orchestrator_goldens.rs:167](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/orchestrator_goldens.rs:167>).
- Signal forge/replay/expiry, régression d’horloge exacte, divergence, outcome unknown, quorum, routage `human.review` et classifier default ont des bras dédiés.
- Le child-process sort réellement avec le code 87 et un nouveau handle reprend sans double effet : [orchestrator.rs test:1482](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/orchestrator.rs:1482>).

Tests verts pour une mauvaise raison ou couverture insuffisante :

- Le test WCP fabrique directement un `CompiledPlan` via le helper générique : [orchestrator.rs test:105](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/orchestrator.rs:105>), puis construit un `JoinPolicy::All` sur deux branches mutuellement exclusives : [orchestrator.rs test:458](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/orchestrator.rs:458>). Le compilateur core refuse précisément cette forme : [compile.rs:1981](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/compile.rs:1981>). Ce test ne démontre donc pas le comportement d’un plan réellement productible.
- Le test cancel forge un intent `/b` alors que son prédécesseur est encore parqué, demande cancel, puis laisse l’orchestrateur appeler la cible : [orchestrator.rs test:1282](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/orchestrator.rs:1282>), [orchestrator.rs test:1343](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/orchestrator.rs:1343>). Il prouve l’émission après cancel, pas la conservation d’un résultat déjà produit.
- Le « désordre réel » map n’est pas provoqué ; la revue reconnaît que seule la mutation d’agrégation inversée est testée dans le moteur synchrone : [sprint83_phase_j_review_r1.md:61](<C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_j_review_r1.md:61>).
- Le journal adverse ne couvre que node inconnu et fanout hors borne : [orchestrator.rs test:1938](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/orchestrator.rs:1938>). Il manque les décisions prématurées, mauvais kind/node, doublons contradictoires, séquences repeat illégales et compositions dynamiques imbriquées.

## 7. THREAT_MODEL v25, REFERENCE et migration — GAP

Lecture ligne par ligne des six rows de [THREAT_MODEL.md:1805](<C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1805>) :

- `T-RUN-SIGNAL-FORGE-REPLAY` — **PARTIEL** : wait OPEN, dédup, atomicité, expiry et closed labels correspondent au code. En revanche le nonce n’est pas « journalisé à l’ouverture » : il est écrit dans la table d’état `waits`, puis journalisé seulement à la consommation. La taille de `decision_label` n’est pas bornée.
- `T-RUN-CANCEL-DOS` — **GAP** : l’affirmation « le scheduler n’émet plus » est fausse puisque `drain_cancelled` peut appeler `target.apply`.
- `T-RUN-TIMER-SKEW` — **PARTIEL** : deadline absolue, CAS, watermark et régression loud sont présents, mais la qualification « fermée » ignore la fenêtre de crash création-run/programmation-timer.
- `T-RUN-REPLAY-DIVERGENCE` — **GAP** : la réduction est déterministe, mais elle n’est pas fail-closed face à des décisions causalement impossibles ou contradictoires ; « pure » ne suffit pas à garantir l’intégrité du replay.
- `T-RUN-SUBWORKFLOW-AMPLIFICATION` — **CONFIRME pour la borne d’amplification** : map/repeat ne dépassent pas les bornes signées et l’expansion subworkflow reste statique. Cela ne couvre toutefois pas le défaut distinct de liveness des scopes dynamiques imbriqués.
- `T-RUN-BACKPRESSURE-STARVATION` — **CONFIRME** : cap local borné, admission canonique et limite mono-run explicitement reconnue.

`STORE_MIGRATION_OPS` est cohérent et honnête sur ses validations : [STORE_MIGRATION_OPS.md:71](<C:/Users/FlowUP/Documents/Code/nexus/docs/release/STORE_MIGRATION_OPS.md:71>). `REFERENCE.md` est globalement cohérent, mais décrit `frontier_node` comme le dernier effet complété alors qu’une late completion est délibérément exclue du frontier : [REFERENCE.md:51](<C:/Users/FlowUP/Documents/Code/nexus/docs/workflow/REFERENCE.md:51>).

## 8. Périmètre — CONFIRME

- `git diff -- Cargo.toml Cargo.lock` est vide.
- Aucun fichier sous daemon, `web/` ou `tools/factory-operator/` n’apparaît dans le status.
- `canonical.rs` est intact ; le comptage reste à 29 domaines et les réservations CompiledPlan/RunEvent restent sans producteur : [canonical.rs:395](<C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/canonical.rs:395>), [canonical.rs:404](<C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/canonical.rs:404>).
- `crates/sbfb-workflow-engine/src/engine.rs` est intact.
- Le comptage direct de `.route(` dans `crates/nexus-shell-daemon/src/http.rs` reste à **91**.
- `git diff --check` ne rapporte aucune erreur de whitespace, seulement les avertissements CRLF de l’environnement Windows.

## 9. Préflight compliance — GAP

La majorité des décisions structurelles est tenue : contrats additifs, runtime IDs, M3, projection daemon inchangée, payload digest-only, cap hôte, child-process réel, 18 kinds et périmètre zéro dépendance.

Mais le préflight n’est pas satisfait dans son ensemble :

- L’écart timeout est **borné dans le code**, mais pas correctement fermé dans le contrat. DEC‑5 reporte les timeouts à K : [preflight.md:95](<C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_phase_j_preflight.md:95>), alors qu’A14 exige les `cancellation_policies` par scope : [preflight.md:175](<C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_phase_j_preflight.md:175>). L’implémentation ne lit que les politiques racine et `workflow_timeout_seconds`; retries, timeouts node et budget restent absents. La revue r1 déclare cette arbitration minimale : [sprint83_phase_j_review_r1.md:55](<C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_j_review_r1.md:55>), mais le préflight autoritatif reste contradictoire.
- DEC‑11 demandait la réutilisation/exposition du lecteur core : [preflight.md:120](<C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_phase_j_preflight.md:120>). Le test documente explicitement que cette décision a été rejetée et lit directement le chemin workspace : [orchestrator_goldens.rs:18](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/orchestrator_goldens.rs:18>).
- L’adaptation « cancel honnête » n’est pas tenue : [preflight.md:162](<C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_phase_j_preflight.md:162>).
- Le contrôle WCP utilise un plan synthétique que le compilateur refuserait.
- Le préflight conserve « 11 run / 10 node » : [preflight.md:188](<C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_phase_j_preflight.md:188>), tandis que la research et le code en ont 11/11.
- La revue r2 reste `PASS-PENDING`, pas `PASS` : [sprint83_phase_j_review_r2.md:7](<C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_j_review_r2.md:7>).

## GAPs classés

- **P0 — aucun identifié.**

- **P1**
  - Cancel pouvant produire le premier effet après `cancel_requested`.
  - Reducer donnant autorité à des décisions/intents causalement impossibles.
  - Join basé sur l’état du node plutôt que la livraison d’edge ; divergence avec le contrat core et blocage possible des perdants `Any`/quorum.
  - Timer workflow définitivement perdu lors d’un crash entre création du run et programmation.
  - Repeat extérieur pouvant ne jamais avancer avec map/repeat imbriqué.

- **P2**
  - `decision_label` non borné et dépôts encore acceptés sur des waits OPEN d’un run terminal.
  - Double `CancelRequested` possible entre cancel opérateur et firing timer.
  - Bras adversariaux insuffisants ; WCP/cancel verts pour une mauvaise raison ; aucun test de portée dynamique imbriquée.
  - DEC‑11 et A14 non tenues ; arbitrage timeout seulement consigné dans r1.
  - Sur-affirmations des rows cancel, timer et replay du THREAT_MODEL.

- **P3**
  - Dérive documentaire 10 états node contre 11.
  - Formulations imprécises « nonce journalisé à l’ouverture » et « frontier = dernier effet complété ».

