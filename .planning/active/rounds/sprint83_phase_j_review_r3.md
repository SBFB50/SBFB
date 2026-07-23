# Sprint 83 Phase J — Review round r3 (post-Codex r1)

Date : 2026-07-23. Codex GPT-5.6 Sol r1 = GAP (0 P0, 5 P1, 5 P2, 2 P3) —
chaque P1 vérifié adversarialement puis TRAITÉ sur l'arbre :

1. **P1 cancel/première émission (RÉEL, fixé)** : `drain_cancelled`
   SUPPRIMÉ — la fermeture n'appelle plus jamais `target.apply` (un intent
   B1 jamais tenté aurait eu sa PREMIÈRE émission post-cancel). Pending
   reste pending, honnêtement journalisé ; `record_late_completion` =
   porte d'un résultat qui ARRIVE (twin spike-faults, testée store-level
   dans le test honest_cancel ajusté : side_output /b == 0 post-cancel).
2. **P1 cohérence causale reducer (sur-vente doc, requalifiée)** : row
   T-RUN-REPLAY-DIVERGENCE « fermée (déterminisme) / bornée (intégrité
   causale) » — refus structurels UnknownNode/FanoutBeyondBound seuls ;
   la cohérence causale est portée par l'écrivain unique lease-fencé ;
   validation causale à la lecture = durcissement K.
3. **P1 join par état vs livraison (RÉEL, fixé)** : `join_state` compte
   désormais la LIVRAISON des edges source→join (`edge_delivery`
   réutilisé) — une source routing Succeeded sur un autre port n'arrive
   jamais au join. Sémantique perdants Any/quorum : le join fire, les
   perdants vivants continuent (pas un deadlock — attente réifiée par le
   test quorum, cancel possible) ; élision des perdants = design K/plan.
4. **P1 timer perdu (RÉEL, fixé)** : `schedule_timer` appelé à CHAQUE
   execute (INSERT OR IGNORE = première deadline gagne, prouvé par la
   mutation M5-composite) ; résidu deadline-du-resume documenté THREAT.
5. **P1 repeat/nested liveness (RÉEL, fixé)** : `scope_settled` ne compte
   que les enfants DIRECTS — un map/repeat imbriqué se replie dans SON
   état Succeeded par récursion du reducer ; helper mort
   `is_direct_or_nested_child` supprimé.

P2 : `decision_label` borné (validate_identity_component) ; dépôt refusé
sur run non-Running ; firing timer bloqué si cancel déjà journalisé (pas
de double entrée) ; rows THREAT cancel/timer/replay requalifiées ;
formulation « nonce journalisé à l'ouverture » corrigée (table d'état à
l'ouverture, journal à la consommation) ; arbitrage A14/DÉC-5 déclaré au
commit body. P3 : formulations reprises.

## Verdict: PASS-PENDING

Suites re-mesurées sur l'arbre corrigé : crates core+engine 183/183+3skip,
fmt dual + clippy -D warnings verts ; workspace Win + Docker
--no-fail-fast re-lancés (résultats consignés au body). Codex r2 lancé sur
cet arbre — la promotion PASS du canonique attend sa réconciliation.
