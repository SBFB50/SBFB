Verdict round 2 : les 3 P1 sont corrigés. En revanche, le P2 concurrent reste partiellement ouvert et la prose P3 du harness n’est corrigée qu’à moitié. Verdict global : pas `CLEAN`.

1. RunProof participants — CORRIGÉ

- `executed_by` suit l’ordre des stages et reçoit exactement le primaire après succès ou le fallback après reroutage : [shard_session.rs:940-945](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:940), [shard_session.rs:983-989](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:983), [shard_session.rs:1038-1043](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:1038).
- Ce vecteur est transmis sans reconstruction à `RunProof::new` : [shard_session.rs:1067-1080](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:1067).
- Le test exige bien la présence du fallback et l’absence du primaire : [shard_session.rs:1725-1744](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:1725).
- Test exécuté : PASS.

2. État/résultat stale — CORRIGÉ

- Sous le même verrou, la transition vers `Generating` efface immédiatement `outcome` et `failure` : [shard_session.rs:841-856](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:841).
- Le test réalise une première génération réussie, tue les workers, relance une génération défaillante puis vérifie l’absence de l’ancien texte et de l’ancien RunProof : [shard_session.rs:1777-1827](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:1777), [shard_session.rs:1828-1850](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:1828).
- Test exécuté : PASS.

3. Fuite d’identité QUIC — CORRIGÉ

- `sanitize_diagnostic` remplace les runs hexadécimaux d’au moins 32 caractères, neutralise les caractères de contrôle/espaces et borne la sortie à 240 caractères : [shard_session.rs:778-816](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:778).
- Les trois erreurs de `drive_hop` sont nettoyées : [shard_session.rs:750-765](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:750).
- Readiness et re-dial primaire sont nettoyés : [shard_session.rs:601-615](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:601), [shard_session.rs:959-977](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:959).
- Le fallback repasse par la readiness nettoyée, puis toutes les erreurs sont rescrubbées avant log et stockage dans `failure` : [shard_session.rs:1017-1037](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:1017), [shard_session.rs:905-912](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:905).
- Une pubkey contiguë de 64 caractères hex ne peut donc plus atteindre `failure` par ces chemins. Les encodages alternatifs ou hex séparé ne font pas partie du motif traité.
- Le test couvre la clé 64-hex, les contrôles, la conservation des identifiants courts et la longueur : [shard_session.rs:1749-1774](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:1749). PASS. Il reste un test unitaire du sanitizer, pas une injection QUIC end-to-end.

4. Generate concurrent 202/202 — PARTIEL

- Le précheck existe et renvoie bien `409 already generating` lorsqu’il observe déjà `Generating` : [http.rs:2346-2365](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:2346).
- La garde atomique sous verrou demeure : [shard_session.rs:841-850](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:841), avec test PASS : [shard_session.rs:2001-2040](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:2001).
- Mais le défaut HTTP 202/202 reste possible : deux handlers peuvent lire `Ready` avant que l’une des tâches créées ligne 2371 n’exécute la transition atomique. Les deux réponses seront alors `202`, même si une seule génération s’exécute : [http.rs:2367-2387](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:2367).
- Le commentaire reconnaît lui-même ce TOCTOU. Il faut réserver atomiquement la session avant d’émettre `202` pour fermer réellement le P2.

5. Proses P3 — PARTIEL

- `ShardSessionView` ne dit plus « two fields » et énumère correctement les trois champs : [shard.rs:60-66](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/schemas/shard.rs:60). CORRIGÉ.
- Le bloc preflight du harness dit maintenant que l’orchestrateur existe : [b3_shard_pipeline.sh:223-239](/C:/Users/FlowUP/Documents/Code/nexus/scripts/acceptance/b3_shard_pipeline.sh:223).
- Mais l’en-tête du même fichier affirme encore exactement l’inverse : orchestrateur « NOT yet wired », route « Phase J STUB », aucun caller production et gate structurellement inaccessible : [b3_shard_pipeline.sh:41-56](/C:/Users/FlowUP/Documents/Code/nexus/scripts/acceptance/b3_shard_pipeline.sh:41). Le P3 n’est donc pas entièrement fermé.

6. Régressions — AUCUNE NOUVELLE RÉGRESSION RUST DÉTECTÉE

- `cargo fmt --all --check` : PASS.
- Tests ciblés `shard_session` : 18/18 PASS.
- `cargo nextest run -p nexus-shell-daemon -p nexus-core-rs --locked --no-fail-fast` : 912/912 PASS.
- Clippy sur les deux paquets, tous targets, `-D warnings` : PASS.
- `bash -n scripts/acceptance/b3_shard_pipeline.sh` et `git diff --check bb6c4f9` : PASS.
- Un test de convergence hors diff a échoué une première fois par timeout, puis a passé isolément et dans le rerun exhaustif ; aucune régression attribuable au diff n’est démontrée.

GAPs restants :

- P0 : aucun.
- P1 : aucun.
- P2 : fenêtre TOCTOU permettant encore deux réponses HTTP `202/202`.
- P3 : prose contradictoire aux lignes 41–56 du harness `b3_shard_pipeline.sh`.
