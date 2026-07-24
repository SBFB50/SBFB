Verdict global : **PARTIEL — scellement techniquement CLEAN, critère d’arrêt atteint avec un unique P3 documentaire**. Le P1 processuel `K-ROUND-REVIEW-AFTER-FIX-08` est **CONFIRMÉ fermé**. Aucun P0, P1 technique, P1 processuel ou P2 ne subsiste. Je ne rends toutefois pas `CLEAN` strict : la provenance historique des deux mutations ajoutées en commentaire n’est pas documentée dans `review_r3`, contrairement à la claim de `review_r5`.

Assiette finale du 24 juillet 2026 à 12:16:55 CEST : HEAD `a23510e828fab9d7e96370d0811c7d3e39ed9316`, working tree sale attendu, aucun artefact r9 préexistant.

## P1 r8 — CONFIRMÉ fermé

`review_r5` est un nouvel artefact distinct, en `PASS-PENDING`, explicitement post-correctif r7→r8, portant le Workflow `wf_e402653d`, ses deux agents, leurs verdicts et l’attente de r9 : [review_r5.md:3](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_k_review_r5.md:3), [review_r5.md:5](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_k_review_r5.md:5).

Ordre durable observé :

- r4 créé à 09:13:33Z ;
- r8 créé à 09:54:27Z ;
- `effects.rs` documenté à 10:08:28Z ;
- r5 créé à 10:09:22Z ;
- r9 lancé ensuite.

r4 n’a pas été réécrit : il conserve ses anciens compteurs `226 + 7 / 14 mutations` et son r7 « en cours » : [review_r4.md:113](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_k_review_r4.md:113), [review_r4.md:127](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_k_review_r4.md:127). SHA-256 r4 : `4D39DD39…`; SHA-256 r5 : `2578E842…`.

Les preuves fraîches `227 + 7` et `15 mutations actives` sont consignées dans r5 : [review_r5.md:37](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_k_review_r5.md:37), [review_r5.md:60](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_k_review_r5.md:60), [review_r5.md:69](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_k_review_r5.md:69). L’inventaire `227 + 7` était déjà observé indépendamment en r8 : [codex_review_r8.md:69](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_k_codex_review_r8.md:69).

## Delta r8→r9 — CONFIRMÉ doc-only

Blobs vivants :

- `process.rs = aa3ffb1ebfd809fc125feec4ecb635ce65b7f387` — inchangé ;
- `THREAT_MODEL.md = 278ede3a96bf41e880117a056cacb002c4b6262a` — inchangé ;
- `lib.rs = 40721ea3ae8e11c2ff9bf11baeb992e19d53a594` — inchangé ;
- `effects.rs = 7853a286…` après annotations.

En retirant exclusivement les deux blocs [effects.rs:1371](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:1371) et [effects.rs:1571](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:1571), le blob reconstruit est exactement `11ebdfcb70209a7748ff060f2921e9fc4c507456`, soit le blob scellé en r8 : [codex_review_r8.md:3](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_k_codex_review_r8.md:3).

L’inventaire des fichiers modifiés après r8 ne contient que `tests/effects.rs` et `review_r5.md`. **Aucun code de production n’a bougé depuis r8.**

## [P3] K-MUTATION-PROVENANCE-R9 — provenance durable inexacte

Les annotations décrivent correctement deux mutants discriminés par les tests :

- `parse-laxiste` : le code exige réellement un `ActivityReceipt` parsable et lié : [orchestrator.rs:440](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:440). Les blobs opaques sont explicitement refusés et l’absence de completion est vérifiée : [effects.rs:1438](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:1438).
- `contains_key` : le reducer exige l’égalité avec l’`effect_id` parqué, pas la simple présence d’une clé : [reducer.rs:475](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:475), [reducer.rs:490](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:490). Le test injecte `e1` puis `e2` et exige l’erreur causale : [effects.rs:1575](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:1575).

**GAP documentaire limité :** r5 affirme que ces deux slugs ont été « jouées boucle r2, documentées review r3 » : [review_r5.md:51](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_k_review_r5.md:51). Or `review_r3` ne contient ni `parse-laxiste`, ni `contains_key`, ni leur exécution rouge. Sa seule mention explicite d’une « mutation ACTIVE jouée » concerne le réarmement du timer de retry : [review_r3.md:12](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_k_review_r3.md:12). Sa réconciliation r2 énumère seulement les corrections `K-RECONCILE-RAW-01` et `K-CAUSAL-02` : [review_r3.md:102](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_k_review_r3.md:102).

La discriminance est donc **CONFIRMÉE** ; leur exécution historique réelle reste seulement attestée par r5, pas recoupable avec r3 comme annoncé. Impact P3 documentaire uniquement, sans incidence technique ou sécuritaire.

## Invariants — CONFIRMÉS

- Goldens byte-identiques à HEAD :
  - `run_event.golden.json` : `9ac8d44167bc472f93330b0c679b455c08ec4e72` ;
  - `run_event_orchestrated.golden.json` : `875166f3cc273a0c450042e71c38b569c7824b37` ;
  - `run_event_terminal.golden.json` : `1ba2be10a96ee3597158d2a7861e57d48e7c80b1`.
- Census vivant = census gelé = **29**, delta nul ; liste contractuelle : [check-frontier-contracts.sh:202](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:202).
- `RUN_EVENT_FORMAT_VERSION == 1` : [run_event.rs:65](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:65).
- Upcasters de production toujours vides : [run_event.rs:390](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:390).
- Cargo.lock : **992 packages HEAD / 992 worktree**, donc **0 nœud neuf**.
- `cargo fmt --all --check` : exit 0.
- `git diff --check a23510e8` : exit 0.

Les claims techniques restent alignées avec le code : refus absolu puis suffixe Windows avant `Command::new` : [process.rs:348](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:348), [process.rs:362](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:362), [process.rs:373](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:373) ; charte et menace inchangées : [lib.rs:106](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/lib.rs:106), [THREAT_MODEL.md:1833](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1833).

Comptage final : **P0 = 0 ; P1 technique = 0 ; P1 process = 0 ; P2 = 0 ; P3 documentaire = 1.**

Décision Round 9 : **PARTIEL, critère d’arrêt atteint — scellement autorisé sous la règle “uniquement P2/P3 documentables”, mais pas CLEAN strict.** Aucun correctif code ni r10 n’est requis. Le P3 doit rester visible dans la copie canonique byte-identique ; r3, r5 et ce rapport ne doivent pas être réécrits.

