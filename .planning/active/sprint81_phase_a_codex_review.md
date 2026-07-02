**Finding Principal**
P2 doc-contract : `PUBLIC_FEED_SPEC.md` garde encore une formulation trop forte en §5.1. Elle dit, pour chaque entrée remote reçue, de vérifier la “Per-author prev_hash chain linkage” (`docs/protocol/PUBLIC_FEED_SPEC.md:170-180`). Le code réel fait volontairement l’inverse à l’ingest : `feed_sync` appelle `verify_entry` avant insert (`crates/nexus-shell-daemon/src/feed_sync.rs:269-272`), et `verify_entry` documente/implémente “format only, no linkage/existence” (`crates/nexus-coordinator-rs/src/public_feed.rs:587-616`). À corriger en doc : linkage sur set disponible / replay, pas rejet pré-insert d’un prédécesseur manquant.

**Verdicts Par Livrable**
1. `feed_materializer.rs` : OK. Le fold part d’une forêt par auteur (`crates/nexus-coordinator-rs/src/feed_materializer.rs:180-216`), suit `prev_hash` depuis genesis avec stop isolé sur fork/gap (`:219-238`), puis k-way merge par `(timestamp, author_pubkey, entry_hash)` (`:242-274`). `materialize_full` et `materialize_verified` partagent `fold_all` (`:302-316`). L’incrémental refuse l’append sauf vue propre, extension stricte des tips, et clé > frontier (`:371-395`). `materialize_up_to` garde le préfixe `seq` puis fold déterministe (`:427-433`).

2. `public_feed.rs` : OK. `FEED_FORMAT_VERSION` reste `1` (`crates/nexus-coordinator-rs/src/public_feed.rs:18-20`), `FeedEntry` et `FeedEntryCanonical` restent intacts (`:147-178`), `to_canonical` reste inchangé (`:191-200`), et JCS/domain feed restent via `DOMAIN_FEED_V1` (`:208-218`). La garde `prev_hash` rejette seulement non-`genesis` / non-lowercase-hex64 (`:605-616`) ; aucune existence/linkage n’est vérifiée dans `verify_entry` (`:618-649`). Le test accepte explicitement un prédécesseur absent mais bien formé (`:1414-1419`).

3. `PUBLIC_FEED_SPEC.md` : PARTIEL. §6 est aligné code : distinction `seq` local vs ordre de projection (`docs/protocol/PUBLIC_FEED_SPEC.md:203-212`), ordre intra-auteur par chaîne et inter-auteurs par tie-break signé (`:213-232`). §7 fallback incrémental est aligné (`:285-292`). Vecteur 14 est aligné avec verify_chain order-independent + materializer déterministe (`:402-408`). Gap restant : §5.1 sur “chain linkage” par entrée reçue, détaillé ci-dessus.

4. `THREAT_MODEL.md` : OK. `T-FEED-CLOCK-SKEW` ne parle plus d’ordering par `seq`, décrit le nouveau fold par chaîne + tie-break, et trace le résiduel postdatage +30j inter-auteurs avec binding author->project_id en carry (`docs/security/THREAT_MODEL.md:549-560`).

5. Tests : OK. Les 7 nouveaux tests materializer couvrent convergence full/incremental, monotone guard, tie-break, backdating intra-auteur, fork isolation, gap fill (`crates/nexus-coordinator-rs/src/feed_materializer.rs:774`, `:822`, `:864`, `:908`, `:954`, `:991`, `:1038`). Le test public_feed couvre formats rejetés et hors-ordre accepté (`crates/nexus-coordinator-rs/src/public_feed.rs:1376-1419`).

**Checks Adversariaux**
Pas de scénario de non-convergence résiduel trouvé pour deux nœuds ayant le même set d’entries valides. Les doublons `entry_hash` ne sont pas un canal réaliste normal : index unique DB (`crates/nexus-coordinator-rs/src/db.rs:177-178`).

Pas de soundness-hole trouvé dans `materialize_incremental`. Le garde `max_applied_key` est conservateur avec timestamps intra-auteur décroissants : il peut forcer un full rebuild inutile, mais pas produire une vue différente du full rebuild.

Pas de casse légitime trouvée sur cursor / verified / up_to : `materialize_verified` garde son contrat all-or-nothing via `verify_chain` (`crates/nexus-coordinator-rs/src/feed_materializer.rs:313-316`), tandis que `materialize_full` reste disponibilité maximale avec préfixes par auteur.

Scope invariants OK : `git diff --name-only` ne contient que les 4 fichiers attendus ; aucun `Cargo.toml`, `Cargo.lock`, migration DB, `nexus-core-rs`, ni `nexus-shell-daemon*` modifié. `git diff --check` clean.

Tests lancés localement :
`cargo test -p nexus-coordinator-rs --locked out_of_order_ingest_converges` : 2 passed.  
`cargo test -p nexus-coordinator-rs --locked test_verify_entry_prev_hash_format` : 1 passed.

**GAPs**
P0 : aucun.  
P1 : aucun.  
P2 : `PUBLIC_FEED_SPEC.md §5.1` sur-promet encore un check de linkage remote “for each received entry”, contradictoire avec l’ingest out-of-order voulu.  
P3 : process seulement, si ces fichiers partent dans le commit : `.planning/active/sprint81_phase_a_review.md` est encore `## Verdict: PASS-PENDING` (`.planning/active/sprint81_phase_a_review.md:139-144`), donc pas encore verdict committable AGENTS.

