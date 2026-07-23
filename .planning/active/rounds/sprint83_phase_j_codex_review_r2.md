Verdict global : **GAP**. Le correctif cancel est réel, la requalification causale est globalement honnête et le correctif nested paraît correct. En revanche, le join reste contractuellement incomplet, la reprise timer peut persister une deadline issue d’une horloge refusée, et la borne `decision_label` crée un nouveau domaine runtime incompatible avec des plans que le compilateur accepte. **Aucun P0, quatre P1 résiduels.**

État vérifié inchangé pendant l’audit : HEAD `03c14d84`, 10 fichiers modifiés et 13 untracked. `git diff --check` ne signale aucune erreur hors avertissements CRLF. Aucune suite n’a été lancée ; les mesures 183/183 + 3 skip restent donc une preuve externe non revalidée ici.

**1. Cancel/première émission — CONFIRMÉ**

La branche `CancelRequested` ferme directement le run puis reboucle, sans drain ni dispatch : [orchestrator.rs:382](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:382>). Le seul appel réel à `target.apply` reste dans `run_effect` : [orchestrator.rs:819](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:819>).

Le test vérifie bien que l’intent `/b` reste pending et que son compteur d’effet reste à zéro après cancel : [orchestrator.rs test:1365](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/orchestrator.rs:1365>). La complétion tardive est ensuite injectée explicitement par la porte store et ne modifie pas le frontier : [orchestrator.rs test:1380](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/orchestrator.rs:1380>), [store.rs:1143](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1143>).

Borne : cette preuve est bien **store-level**. Aucun chemin Phase J ne détecte automatiquement un retour concurrent et ne le route vers `record_late_completion`; cette intégration reste effectivement K.

**2. Cohérence causale reducer — CONFIRMÉ pour la requalification, pas pour une fermeture**

Le reducer accepte toujours `EffectCompleted`, `BranchTaken`, `SignalConsumed` ou `HumanDecisionRecorded` sans revalider leur causalité : [reducer.rs:302](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:302>). Les seuls refus structurels ajoutés restent `UnknownNode` et `FanoutBeyondBound`.

La row principale dit maintenant exactement cela et classe correctement la propriété « fermée pour le déterminisme / bornée pour l’intégrité causale » : [THREAT_MODEL.md:1808](<C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1808>). Le chemin orchestrateur acquiert bien le lease avant ses écritures : [orchestrator.rs:280](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:280>), et chaque transaction d’un handle lease-fencé réasserte owner/epoch : [store.rs:3133](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:3133>).

Ce n’est toutefois pas une validation causale : le store conserve explicitement un mode unfenced [store.rs:576](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:576>). Le carry K doit donc rester explicite.

**3. Join par livraison d’edge — GAP**

Le correctif d’état est réel : chaque source est désormais comptée selon ses edges vers le join et `edge_delivery`; un port de routage différent devient `Dead`, pas arrivé : [reducer.rs:696](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:696>).

Mais deux P1 restent :

- Le payload du join ne réutilise pas cette livraison. `output_for_edge` collecte encore toute source dont le **node** est `Succeeded`, puis `Any` prend la première clé lexicographique : [orchestrator.rs:982](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:982>). Un switch réussi sur un autre port est donc correctement exclu de l’état du join, mais peut encore être injecté dans sa valeur aval. Le compilateur autorise ce cas pour `Any` : [compile.rs:1926](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/compile.rs:1926>).

- `Any` ne respecte toujours pas son contrat « le premier arrivé gagne, les autres sont abandonnés » : [graph.rs:360](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/graph.rs:360>). Les perdants vivants continuent, et `fully_settled` exige encore que tous les nodes soient settled : [reducer.rs:267](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:267>). Dire « cancel possible » dans r3 ne remplace pas l’abandon contractuel.

**4. Timer — GAP**

Le P1 initial « timer perdu pour toujours » est fermé : `schedule_timer` est appelé à chaque `execute`, indépendamment de `created`, et `INSERT OR IGNORE` conserve la première deadline : [orchestrator.rs:321](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:321>), [store.rs:2115](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2115>).

Mais la programmation est committée **avant** `advance_clock_watermark` : [orchestrator.rs:339](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:339>), [orchestrator.rs:347](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:347>). Les deux vivent dans des transactions distinctes : [store.rs:2131](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2131>), [store.rs:2043](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2043>).

Conséquence : sur un store dont le watermark vaut 100 000, un appel avec `now=0` peut créer le run et persister la première deadline à 10 000, puis échouer `ClockRegression`. L’appel honnête suivant ne peut plus remplacer cette deadline et annule immédiatement le run. La promesse « passe entière refusée avant décision d’horloge » est donc fausse.

Le test timer existant vérifie seulement qu’une deadline déjà écrite survit au restart : [orchestrator.rs test:1197](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/orchestrator.rs:1197>). Il ne construit ni timer absent avec `created=false`, ni insertion précédant un refus watermark.

**5. Repeat/nested — PARTIEL**

Le changement est structurellement correct : `scope_settled` ne regarde plus que les enfants directs via `parent_scope(id) == expanding_structural` : [orchestrator.rs:586](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:586>). Le reducer replie récursivement l’état des `Map`/`Repeat` imbriqués et leurs propres scopes : [reducer.rs:605](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:605>), [reducer.rs:781](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:781>).

Il manque toutefois le test discriminant demandé par ce P1. Les tests actuels couvrent un map simple puis un repeat simple : [orchestrator.rs test:672](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/orchestrator.rs:672>), [orchestrator.rs test:701](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/orchestrator.rs:701>), jamais repeat→map, repeat→repeat ou map→repeat.

**(a) Complétude des cinq traitements — GAP**

Les points 1 et 2 sont correctement traités selon leur portée annoncée. Le point 5 est convaincant dans le code mais non verrouillé par un test. Les points 3 et 4 restent incomplets fonctionnellement.

Les trois P2 store sont réellement présents : borne du label [store.rs:1839](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1839>), dépôt limité aux runs `Running` [store.rs:1848](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1848>), firing bloqué par tout `CancelRequested` existant [store.rs:2173](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2173>). Mais la borne du label introduit elle-même un P1 contractuel décrit ci-dessous.

**(b) Régressions join — GAP**

- `All` avec feuilles ordinaires : **CONFIRMÉ**, couvert par `orchestrator_parallel_branches` [orchestrator.rs test:390](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/orchestrator.rs:390>).

- Quorum avec feuilles : **CONFIRMÉ pour le firing précoce**, couvert en 1-de-2 [orchestrator.rs test:2016](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/orchestrator.rs:2016>).

- Skip-into-join : **CONFIRMÉ dans le reducer synthétique**, mais ce test construit directement un `CompiledPlan` [orchestrator.rs test:105](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/orchestrator.rs:105>) alors que le compilateur refuse précisément cet `All` sur branches exclusives : [compile.rs:1981](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/compile.rs:1981>).

- `Any` avec feuilles : **PARTIEL**. L’état s’ouvre bien dès une arrivée [reducer.rs:757](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:757>), mais il n’existe pas de test `Any`, le gagnant n’est pas tiré de l’ordre du journal et les perdants ne sont pas abandonnés.

**(c) Nouveaux P0/P1 — GAP**

Aucun P0. Trois P1 sont nouveaux ou exposés par les corrections, auxquels s’ajoute un P1 R1 non réellement fermé :

- **Nouveau P1 — join payload** : état filtré par edge, payload encore filtré par état global du node.

- **Nouveau P1 — timer/watermark** : une deadline issue d’une horloge ensuite refusée peut rester la première deadline irréversible.

- **Nouveau P1 — domaine `decision_label`** : le store refuse vide, contrôle ou plus de 512 octets via [hash.rs:93](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/hash.rs:93>), mais le contrat core accepte toute `String` [contracts.schema.json:896](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/contracts.schema.json:896>) et le compilateur ne valide que liste vide, doublons et `default` [compile.rs:1695](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/compile.rs:1695>). Un plan signé valide peut donc devenir impossible à reprendre.

- **P1 R1 résiduel — `Any`** : les perdants vivants ne sont toujours pas abandonnés.

**(d) Exactitude des rows THREAT — GAP**

- `T-RUN-REPLAY-DIVERGENCE` : **CONFIRMÉ pour la row principale**. La qualification est honnête. Le changelog reste toutefois périmé en disant simplement « fermée » : [THREAT_MODEL.md:2165](<C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:2165>).

- `T-RUN-CANCEL-DOS` : **PARTIEL**. Le stop d’émission du chemin de fermeture est exact ; « un résultat qui arrive atterrit par la porte » décrit encore une API préparée/testée manuellement, pas un routage Phase J effectif.

- `T-RUN-TIMER-SKEW` : **GAP**. Elle reste classée « fermée », affirme une proposition « à chaque passe » et ignore l’écriture timer antérieure au watermark : [THREAT_MODEL.md:1807](<C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1807>).

- `T-RUN-SIGNAL-FORGE-REPLAY` : **GAP documentaire**. Le texte dit toujours que le nonce est « journalisé à l’ouverture » [THREAT_MODEL.md:1805](<C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1805>), alors qu’il est placé dans `waits`; le journal n’est écrit qu’à la consommation.

- `T-RUN-SUBWORKFLOW-AMPLIFICATION` et `T-RUN-BACKPRESSURE-STARVATION` : **CONFIRMÉES** dans leur portée.

**GAPs résiduels classés**

- **P0 : aucun.**

- **P1 :**
  - Join state/payload incohérents pour une source routing non livrée.
  - `Any` ne discard pas les perdants et peut laisser le run parqué.
  - Timer écrit avant watermark, deadline refusée mais durable.
  - Domaine `decision_label` runtime plus strict que le contrat core.

- **P2 :**
  - Aucun test direct routing-source→join, timer absent/recovery, nested dynamic ou nouveaux gates label/run-status/cancel-timer.
  - `record_late_completion` reste une porte manuelle non conditionnée au statut cancelled et sans appelant d’arrivée Phase J.
  - La validation causale est annoncée K dans THREAT mais absente du scope K du plan.
  - DÉC‑11 reste contredite par le lecteur workspace direct [orchestrator_goldens.rs:18](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/orchestrator_goldens.rs:18>) ; A14 exige toujours des politiques par scope alors que seul `""` est consommé [preflight.md:175](<C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_phase_j_preflight.md:175>).

- **P3 :**
  - `REFERENCE.md` dit encore « dernier effet complété » malgré l’exclusion volontaire des late completions du frontier : [REFERENCE.md:62](<C:/Users/FlowUP/Documents/Code/nexus/docs/workflow/REFERENCE.md:62>).
  - Préflight toujours à « 10 états node » contre 11 dans le code : [preflight.md:188](<C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_phase_j_preflight.md:188>).
  - Changelog replay et formulations « chaque passe »/nonce non propagées jusqu’au texte final.

