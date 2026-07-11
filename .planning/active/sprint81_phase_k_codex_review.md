État audité : `HEAD=43623a53bcef3bc7b56c5d8e7eec8637d7c4e080`, inchangé pendant la review. Working tree toujours pré-commit. Suites non rejouées, conformément à la consigne.

1. **ENCORE GAP — provenance P2, mais exécution P1 déchargée.**  
   L’agrégat est valide, déclare bien le vocabulaire fermé incluant `MIXED` ([l. 6](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint81_t2_acceptance.json:6)), porte `PASS` top-level ([l. 7-8](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint81_t2_acceptance.json:7)) et `b3_p2_quorum=PASS` ([l. 59-64](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint81_t2_acceptance.json:59)). Tous les statuts actifs recalculés sont exclusivement `PASS`, `ACTED` ou `MIXED` ; les `BLOCK/NOT-RUN` restants sont historiques dans les notes. Le [raw PASS](/C:/Users/FlowUP/Documents/Code/nexus/scripts/acceptance/.b3_quorum_k.json:1) est identique caractère par caractère à l’objet embarqué, et le [raw run 1 BLOCK](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint81_t2_quorum_k_block.json:1) est préservé.  
   En revanche, le raw ne contient ni `REDUNDANCY`, ni modèle, ni identités : son schéma le confirme ([harness l. 32-33](/C:/Users/FlowUP/Documents/Code/nexus/scripts/acceptance/b3_live_pc_vps.sh:32)) et `REDUNDANCY` vaut même `1` par défaut ([l. 130](/C:/Users/FlowUP/Documents/Code/nexus/scripts/acceptance/b3_live_pc_vps.sh:130)). L’agrégat est honnête sur cette limite ([l. 64](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint81_t2_acceptance.json:64)), mais les détails opérateur sont ensuite répétés sans qualificatif dans [verification.md l. 156-168](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint81_verification.md:156), [CLAUDE.md l. 189-191](/C:/Users/FlowUP/Documents/Code/nexus/CLAUDE.md:189), [SPRINT_LOG l. 19](/C:/Users/FlowUP/Documents/Code/nexus/docs/claude/SPRINT_LOG.md:19) et [sprint82_audit_plan.md l. 64-69](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint82_audit_plan.md:64). La contrainte « aucune claim au-delà du committé sans operator-inspected/corroborated, uncommitted » n’est donc pas satisfaite partout.

2. **CORRIGÉ.**  
   `observe.curl.md` décrit désormais un identifiant inconnu et la réponse `{found:false, session:null}` ([l. 37-43](/C:/Users/FlowUP/Documents/Code/nexus/docs/sharding/examples/observe.curl.md:37)). Aucun résidu `empty store (today, every id)` dans `docs/sharding/`.

3. **CORRIGÉ.**  
   `RunProof` est signé par le driver ([REFERENCE l. 37-43](/C:/Users/FlowUP/Documents/Code/nexus/docs/sharding/REFERENCE.md:37)), et le data-plane est explicitement `driver↔stage`, jamais une paire de workers adjacents ([l. 68-74](/C:/Users/FlowUP/Documents/Code/nexus/docs/sharding/REFERENCE.md:68)). `EXPLANATION.md` précise le relais par le driver et l’absence de communication worker↔worker ([l. 28-32](/C:/Users/FlowUP/Documents/Code/nexus/docs/sharding/EXPLANATION.md:28)), puis le signer driver ([l. 48-61](/C:/Users/FlowUP/Documents/Code/nexus/docs/sharding/EXPLANATION.md:48)).

4. **CORRIGÉ.**  
   `MIXED` est déclaré et défini dans le vocabulaire ([l. 6](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint81_t2_acceptance.json:6)), utilisé pour `baseline_098` ([l. 13-16](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint81_t2_acceptance.json:13)), et la phrase borne correctement le « never PASSed » à la baseline 0.98 avant le premier PASS post-bump Phase K ([l. 16](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint81_t2_acceptance.json:16)).

5. **ENCORE GAP — P3 documentaire.**  
   Les deux corrections annoncées sont présentes : `LoadedStageDescriptor` explique que `ShardProtocol` n’émet pas l’all-zero pour `None` ([l. 546-552](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/shard.rs:546)) et `from_loaded_stage(None)` est borné aux appels directs/tests ([l. 645-651](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/shard.rs:645)).  
   Mais le commentaire de test affirme encore que l’all-zero couvre le cas « echo left serving in a real session » ([l. 1050-1052](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/shard.rs:1050)), alors que le chemin réel `None` renvoie désormais la requête en écho et échoue au décodage ([l. 328-346](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/shard.rs:328)). C’est l’ancien modèle, encore vivant dans un commentaire.

6. **CORRIGÉ.**  
   La footgun est explicitement documentée : `loaded_stage=None`, aucun montage production, fail-closed si monté tel quel ([l. 641-646](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-worker-core/src/llm/shard.rs:641)). Le montage production utilise effectivement `ShardStageForwarder` ([main.rs l. 356-362](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/main.rs:356)).

7. **CORRIGÉ.**  
   Aucun résidu « 8 champs ». La review annonce `9 champs` ([l. 27-28](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint81_phase_k_review.md:27)) et réitère le contrat aux neuf champs ([l. 63-64](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint81_phase_k_review.md:63)).

8. **CORRIGÉ.**  
   Le commentaire précise que `retries=1` couvre tout `[profile.ci]`, nightly `binary(multi_daemon)` compris ([nextest l. 53-63](/C:/Users/FlowUP/Documents/Code/nexus/.config/nextest.toml:53)); le workflow utilise réellement `--profile ci` ([l. 59-65](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/integration-nightly.yml:59)). Les deux tests portent les avertissements honnêtes « second CALL same-process », pas reboot réel ([runtime l. 4491-4494](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:4491), [l. 4546-4548](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:4546)).

9. **ENCORE GAP — même P2 de provenance que le point 1.**  
   Toutes les mises à jour sont présentes : section quorum dans [verification.md](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint81_verification.md:156), row 81 dans [SPRINT_LOG](/C:/Users/FlowUP/Documents/Code/nexus/docs/claude/SPRINT_LOG.md:19), agrégat et LT-7 fermé dans [CLAUDE.md](/C:/Users/FlowUP/Documents/Code/nexus/CLAUDE.md:189) et [l. 467-468](/C:/Users/FlowUP/Documents/Code/nexus/CLAUDE.md:467), carry éteint et signal cold-boot dans [sprint82_audit_plan.md](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint82_audit_plan.md:61) et [l. 163-165](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint82_audit_plan.md:163), addendum final dans [review l. 372-400](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint81_phase_k_review.md:372). Le header reste normalement `PASS-PENDING` ([l. 3](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint81_phase_k_review.md:3)).  
   Les valeurs concordent : `2026-07-11`, `6s`, budget `30s`, `redundancy=2`, `a424d8e748`, `81cfeab05c`; le task ID run 3 est identique dans le [raw](/C:/Users/FlowUP/Documents/Code/nexus/scripts/acceptance/.b3_quorum_k.json:1) et l’[objet embarqué](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint81_t2_acceptance.json:61). Le seul défaut est l’absence du qualificatif de provenance sur plusieurs synthèses citées au point 1.

## Verdict global

**Gate bloquante levée : aucun P0/P1 restant**, en appliquant le modèle de preuve explicitement autorisé — run matériel attesté par l’opérateur, avec logs non committés clairement qualifiés dans l’agrégat.

Verdict round 3 : **PASS avec résiduels P2/P3, pas CLEAN** :

- un P2 unique de propagation de provenance sur quatre surfaces canoniques ;
- un P3 de commentaire `all-zero echo` encore obsolète.

Ce n’est donc plus un `FAIL` bloquant, mais ces deux corrections restent nécessaires avant de présenter la reconciliation comme totalement propre.

