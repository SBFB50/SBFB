Verdict global : **GAP**. Les traitements 1 et 5 sont confirmés ; 2 et 3 ne ferment que partiellement leur P1 ; l’arbitrage 4 documente correctement l’absence de cancellation de scope, mais sa prémisse « valeurs ignorées » n’est pas vraie dans le moteur. **Aucun P0 et aucun P1 distinct nouvellement introduit**, mais **trois P1 r2 restent ouverts**. Aucune suite relancée conformément à la consigne ; les résultats 73/73 + 3 skip et workspace restent donc des preuves rapportées. HEAD est resté `03c14d84`.

1. **Join-payload — CONFIRMÉ**

La branche `Join` recherche les edges exactes `source → join`, appelle `edge_delivered`, puis n’insère la valeur qu’en cas de livraison : [orchestrator.rs:1003](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1003>), [orchestrator.rs:1009](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1009>), [orchestrator.rs:1018](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1018>). `edge_delivered` respecte les ports pris par switch/classifier/human-review : [orchestrator.rs:929](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:929>).

C’est symétrique du reducer, qui applique `edge_delivery` aux mêmes edges source→join : [reducer.rs:704](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:704>), [reducer.rs:722](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:722>). Le P1 payload/state est fermé.

2. **Timer/watermark — PARTIEL**

L’ordre textuel est corrigé : watermark à [orchestrator.rs:328](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:328>), puis programmation à [orchestrator.rs:342](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:342>). La row documente honnêtement l’horloge initialement haute et devient « bornée » : [THREAT_MODEL.md:1807](<C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1807>).

Mais les deux opérations ne partagent pas la même lecture : `clock.now_ms()` est rappelé pour calculer la deadline. Une horloge renvoyant successivement `100000` au watermark puis `0` à la deadline fait persister `10000`, alors que cette seconde lecture est régressive. `schedule_timer` n’effectue aucune vérification watermark : [store.rs:2124](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2124>). La garantie « une lecture refusable ne devient jamais la première deadline » reste donc fausse : **P1 résiduel TOCTOU horloge**.

3. **Domaine `decision_label` — PARTIEL**

La borne charset est réellement supprimée ; seule subsiste la limite de 256 KiB : [store.rs:1839](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1839>). Le set fermé est bien contrôlé avant les CAS et l’écriture de l’événement : [store.rs:1995](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1995>), [store.rs:2015](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2015>).

Cependant, le domaine reste désaligné sur le core : le schéma ne porte aucun `maxLength` [contracts.schema.json:896](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/contracts.schema.json:896>) et le compilateur ne contrôle que set vide, doublons et `default` [compile.rs:1695](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/compile.rs:1695>). Les documents workflow fournis sont parsés sans borne de chaîne [compile.rs:961](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/compile.rs:961>) et un fichier épinglé peut atteindre 256 MiB [pack.rs:123](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/pack.rs:123>).

Un label valide de 262145 octets compile donc mais reste impossible à déposer. Le P1 contractuel est déplacé de 512 octets à 256 KiB, pas fermé.

4. **Any-perdants — GAP**

L’arbitrage est bien écrit, et `CANCEL-DOS` qualifie correctement `record_late_completion` de porte préparée avec routage effectif en K : [THREAT_MODEL.md:1806](<C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1806>). Accepter que les branches continuent est une décision produit recevable.

En revanche, leurs valeurs ne sont pas réellement figées/ignorées :

- Le contrat dit que la première source arrivée gagne : [graph.rs:360](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/graph.rs:360>).
- Le moteur recollecte toutes les sources actuellement livrées puis choisit la première clé canonique, explicitement « jamais l’ordre d’arrivée » : [orchestrator.rs:1024](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1024>). Pour Quorum, toutes les valeurs actuellement livrées sont retournées.
- Le scheduler réduit d’abord [orchestrator.rs:372](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:372>), peut ensuite consommer une nouvelle source [orchestrator.rs:411](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:411>), puis dispatche avec l’ancien ready-set [orchestrator.rs:462](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:462>). Une valeur arrivée après l’ouverture du join peut donc remplacer le gagnant `Any` ou agrandir la cohorte Quorum avant l’effet aval.
- Sur reprise d’un effet aval pending, ce changement peut provoquer `StepHashMismatch` : [orchestrator.rs:760](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:760>).

La row BACKPRESSURE affirmant que les valeurs sont ignorées est donc factuellement trop forte : [THREAT_MODEL.md:1810](<C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1810>). L’arbitrage cancellation ne ferme pas ce **P1 de stabilité du gagnant/de la cohorte**.

5. **Nested — CONFIRMÉ**

`scope_settled` ne considère que les enfants directs : [orchestrator.rs:592](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:592>). Le reducer replie récursivement Map/Repeat dans leur propre état : [reducer.rs:605](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:605>).

Le test ajouté est discriminant : il exige la complétion du repeat englobant [orchestrator.rs test:2149](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/orchestrator.rs:2149>) et vérifie les deux identités à doubles suffixes `/loop~20/inner~20/...` et `...~21/...` : [orchestrator.rs test:2154](<C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/orchestrator.rs:2154>).

### GAPs résiduels classés

- **P0 : aucun.**
- **P1 :**
  - deadline calculée depuis une seconde lecture non passée au watermark ;
  - borne `decision_label` runtime sans borne équivalente au contrat/compilateur ;
  - gagnant `Any`/cohorte Quorum non figé et valeurs tardives réobservables.
- **P2 :**
  - aucun test engine `JoinPolicy::Any` ;
  - aucun test direct routing-source→join vérifiant le payload ;
  - aucun clock séquencé couvrant la régression intra-appel ;
  - aucun test de parité sur un label valide dépassant la borne store.
- **P3 documentaire :**
  - le changelog dit encore `T-RUN-TIMER-SKEW fermée` malgré la row « bornée » : [THREAT_MODEL.md:2163](<C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:2163>).

