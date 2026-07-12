Verdict global : **GAP — commit atomique BLOQUÉ**.

Le P1 fingerprint est bien résolu. En revanche, je trouve **un nouveau P1 réel** introduit par la mitigation du binding manifeste : le calcul du digest relit le GGUF entier de 16,3 Go en mémoire après chargement du backend. Sur le Mac M2 8 Go déjà à ~5,9 Go RSS, ce chemin peut empêcher `serve --model` de redémarrer.

Je n’ai exécuté aucune commande, conformément à l’instruction. Les lignes ci-dessous sont celles des hunks fournis.

## Rapport par livrable

1. **OK — payloads applicatifs**

- `SHARD_STEP_PAYLOAD_V`, `ShardStepRequest` et `ShardStepReply` sont bien des payloads JSON internes aux frames opaques, avec `deny_unknown_fields` et garde de version : `crates/nexus-core-rs/src/shard.rs:379-472`.
- Les deux sens couvrent roundtrip, fp32 mal routé, version invalide et rejets croisés request↔reply : `crates/nexus-core-rs/src/shard.rs:518-602`.
- Les exports sont cohérents : `crates/nexus-core-rs/src/lib.rs:196-201`.
- Aucune modification de l’ALPN, du framing length-prefixed, de l’admission ou de `MAX_SHARD_FRAME_BYTES` n’apparaît dans le diff fourni. `SHARD_STEP_PAYLOAD_V` est correctement documenté comme garde applicative, pas comme `*_FORMAT_VERSION`.

2. **PARTIEL — worker-core réel correct, preuve GGUF non attachée**

- Le strict-last est bien imposé, y compris rejet first+last : `crates/nexus-worker-core/src/llm/shard.rs:498-514`.
- Injection embeddings, `embeddings_ith(last)`, `get_logits_ith(last)`, argmax avec tie-break au plus petit index, `is_eog_token` et detokenize sont correctement enchaînés : `crates/nexus-worker-core/src/llm/shard.rs:535-589`.
- Les helpers fp32-LE et les trois rôles first/mid/last sont correctement séparés : `crates/nexus-worker-core/src/llm/shard.rs:650-762`.
- Le test GGUF E2E existe et teste deux forwards déterministes puis un step suivant : `crates/nexus-worker-core/src/llm/shard.rs:1015-1085`.

Réserves :

- Le PASS Mistral-7B CPU n’est attesté que par la prose du T2, sans sortie brute jointe : `.planning/active/sprint81_t2_j_shard_inference.json:7`.
- Des logits tous NaN produiraient silencieusement le token 0.
- `token_to_str` peut échouer sur certains byte-tokens qui ne forment pas isolément de l’UTF-8 valide. Non bloquant pour le run démontré, mais dette backend réelle.

3. **OK — boucle de décodage, fingerprint corrigé**

- Dispatch digest non nul vers le decode réel, digest nul vers l’ancien echo : `crates/nexus-shell-daemon/src/shard_session.rs:1022-1050`.
- Liens persistants et write+read sous un seul timeout : `crates/nexus-shell-daemon/src/shard_session.rs:954-1014`.
- Churn : fallback consommé via `take()`, readiness re-probée, input courant repris depuis le replay, puis fallback conservé : `crates/nexus-shell-daemon/src/shard_session.rs:1278-1371`.
- Participants dédupliqués à partir des exécuteurs ayant effectivement répondu : `crates/nexus-shell-daemon/src/shard_session.rs:1367-1370`.
- Cap texte fail-clean puis FIN sur le chemin succès : `crates/nexus-shell-daemon/src/shard_session.rs:1380-1415`.
- Tokens, métriques, preuve signée et fingerprint sont produits : `crates/nexus-shell-daemon/src/shard_session.rs:1420-1465`.
- Le taux réel reste déflooré, l’echo conserve le floor de liveness : `crates/nexus-shell-daemon/src/shard_session.rs:452-474`.

**P1 fingerprint confirmé résolu** : `fingerprint = parse_toploc_hex(...)` est exécuté sans condition à chaque réponse, avant le test EOS. Une dernière réponse vide/invalide donne donc zéro, jamais la valeur précédente. Le helper total et le cas « dernier toploc vide » sont testés : `crates/nexus-shell-daemon/src/shard_session.rs:1209-1230` et `:2260-2440`.

Résidus non bloquants : les sorties d’erreur avant le teardown ne font pas de FIN propre, et un plan extrême avec primaires+fallbacks peut dépasser le cap de participants du `RunProof`.

4. **PARTIEL — tests hermétiques solides, deux sur-promesses**

- Les tests utilisent de vrais nœuds iroh loopback ; seuls les calculs LLM sont simulés : `crates/nexus-shell-daemon/src/shard_session.rs:2163-2550`.
- EOS, borne de tokens, fingerprint final vide et reroute mid-decode sont couverts.
- Le passage des fixtures echo à `model_digest=[0;32]` est cohérent avec le nouveau dispatch et aucune assertion n’est retirée dans le diff : `crates/nexus-shell-daemon/src/shard_session.rs:1635-1695`.

Gaps :

- `decode_loop_reroutes_mid_decode_to_fallback` ne compare pas programmatiquement son résultat à un run témoin sans churn ; les deux tests vérifient séparément le même littéral. Cela teste bien la séquence attendue, mais la claim « assert IDENTIQUE au run sans churn » est plus forte que le code.
- Aucun test dédié ne couvre le nouveau cap `MAX_RESULT_TEXT_BYTES`, ni la divergence de floor `echo=≥1` / réel sub-1 tok/s=`0`.

Le compte annoncé est également incohérent avec la chronologie écrite : depuis `2081/2081 pre-fixes`, le test clap ajoute `+1` et les deux nouveaux tests fingerprint ajoutent `+2`; l’attendu serait donc `2084`, pas `2082`. L’assert cross-reject ajoutée dans un test existant ne change pas le compteur.

5. **GAP — CLI présente, nouveau P1 dans `serve --model`**

Les éléments demandés sont bien câblés :

- Options modèle/fenêtre/GPU/contexte et feature-gate clair : `crates/nexus-shell-daemon/src/cli.rs:190-249`, `crates/nexus-shell-daemon/src/main.rs:270-382`.
- Précondition `start < end` : `crates/nexus-shell-daemon/src/main.rs:296-301`.
- Plan déterministe utilisant `plan_placement` : `crates/nexus-shell-daemon/src/main.rs:392-467`.
- `generate --max-tokens` : `crates/nexus-shell-daemon/src/cli.rs:286-290`, `main.rs:510-530`.
- Collision clap corrigée et testée : `crates/nexus-shell-daemon/src/cli.rs:264-273`, `:648-688`.

Mais la mitigation digest est bloquante :

```rust
let bytes = std::fs::read(gguf)?;
hex::encode(blake3_hash(&bytes))
```

Evidence : `crates/nexus-shell-daemon/src/main.rs:331-336`.

Elle alloue et remplit un `Vec` de **16 283 594 912 octets**, après le chargement du shard backend. Sur le tail M2 8 Go annoncé à ~5,9 Go RSS et 77 Mo libres, cela peut OOM, provoquer un swap massif ou tuer le processus avant même la création du nœud. Le live PASS antérieur ne valide pas cette mitigation ajoutée après la précédente passe.

Correction exigée : hash BLAKE3 streaming avec buffer borné, puis smoke-test du binaire Metal/CUDA final.

6. **OK — HTTP**

- `max_tokens` optionnel est ajouté au body : `crates/nexus-shell-daemon/src/http.rs:2309-2318`.
- Défaut à `DEFAULT_MAX_NEW_TOKENS` puis transmission au drive : `crates/nexus-shell-daemon/src/http.rs:2373-2389`.
- Projection `tokens` dans `/result` : `crates/nexus-shell-daemon/src/http.rs:2413-2417`.
- La borne 256 est appliquée dans la boucle de décodage. Aucun retrait du duress-gate existant n’apparaît dans le diff.

7. **OK — schémas et whitelist**

- Champ `tokens` documenté et requis-nullable : `crates/nexus-core-rs/src/schemas/shard.rs:134-151`.
- Whitelist exact-keys mise à jour : `crates/nexus-core-rs/src/schemas/shard.rs:421-430`.
- Les deux snapshots ajoutent exactement le champ et sa clé required :
  `shard_session_result_response.schema.json:31-64` et
  `shard_session_result_view.schema.json:24-57`.

8. **OK — features et dépendances**

- Cascade correcte `llm_llama_cpp` → CUDA/Metal : `crates/nexus-shell-daemon/Cargo.toml:162-173`.
- Aucune nouvelle dépendance n’apparaît ; `Cargo.lock` est absent du diff.
- Réserve de validation : ces features ne sont pas construites par le CI standard et le nouveau chemin de hash post-live n’a donc aucune preuve d’exécution fournie.

9. **PARTIEL — harness corrigé dans le source, artefact non réconcilié**

Corrections confirmées :

- Les tells `result_text != prompt`, `tokens >= 2`, preuve DRIVER et taux déflooré sont bloquants : `scripts/acceptance/b3_shard_pipeline.sh:410-445`.
- Les libellés mensongers per-shard/verified ont été remplacés : `scripts/acceptance/b3_shard_pipeline.sh:15-63`, `:430-445`.
- `tokens` est présent dans les encodeurs Python et bash : `scripts/acceptance/b3_shard_pipeline.sh:129-184`.
- Le body generate est construit via `json.dumps`; le fallback refuse quotes/backslashes : `scripts/acceptance/b3_shard_pipeline.sh:345-371`.
- `RESULT_RESPONSE` préserve bien la dernière réponse `/result` : `scripts/acceptance/b3_shard_pipeline.sh:374-405`.
- `rig.local.env.example` est à jour, y compris la note sur le nom d’exécutable : `scripts/acceptance/rig.local.env.example:22-55`.

Mais :

- Les appels poll `/result` et `drop-shard` utilisent encore `eval` : `b3_shard_pipeline.sh:376` et `:402`. L’injection du body generate est corrigée, pas l’ensemble du harness.
- Le validateur dit « positive integer » mais accepte `MAX_TOKENS=0` : `b3_shard_pipeline.sh:339-343`.
- Le commentaire indique que la réponse churn est « recorded separately », mais `CHURN` n’est écrit dans aucun champ de `emit_artifact`.
- Le source courant préserverait `/result` comme `last_response`, alors que l’artefact T2 contient encore la réponse churn. Le fix est donc correct par inspection, mais non exercé par l’artefact committable.

10. **PARTIEL — T2 honnête sur la portée, mais raw issu de l’ancien harness**

Points corrects :

- Enveloppe palier, lock iroh, LAN-not-WAN, path-type non asserté et carries explicites : `.planning/active/sprint81_t2_j_shard_inference.json:1-8`, `:36-42`.
- Gates raw cohérents : 16 tokens, 2 tok/s, RTT 14 ms, preuve non vide : `:12-25`.
- Les deux continuations déterministes sont présentes et identiques : `:27-33`.
- Les carries per-shard proofs, SI-9 live et KV F2 correspondent au code : `:37-40`.
- Aucun token d’authentification, secret ou préfixe de pubkey membre n’apparaît. La signature `run_proof` et le digest modèle sont des données publiques. Le préflight contient toutefois un nom utilisateur et une IP LAN explicites : métadonnées privées, pas des credentials.

Incohérence matérielle :

- Le raw T2 contient `"last_response":"{\"found\":true,\"dropped\":true}"` : `.planning/active/sprint81_t2_j_shard_inference.json:23`.
- Le harness courant remplace au contraire `LAST_RESPONSE` par `RESULT_RESPONSE` après le churn : `scripts/acceptance/b3_shard_pipeline.sh:400-405`.

Ces deux faits ne peuvent pas provenir de la même version du script. Le raw est vraisemblablement authentique mais **pré-fix**. Il doit être étiqueté comme tel, ou le harness final doit être rejoué et l’artefact régénéré. Si les deux preuves sont voulues, il faut conserver séparément `result_response` et `churn_response`.

## Invariants non négociables

- **0 bump wire : OK sur le diff fourni.** Aucun `*_FORMAT_VERSION`, ALPN, frame ou admission n’est modifié. `SHARD_STEP_PAYLOAD_V` est explicitement applicatif.
- **0 dépendance externe : OK.** Features seulement ; aucun diff `Cargo.lock`.
- **Echo transport byte-identique : OK.** Digest nul tombe dans le corps historique inchangé ; seul son résultat expose désormais `tokens=1`.
- **Duress-gate generate : aucune altération visible**, mais le corps complet du gate n’est pas inclus dans le texte collé.
- **SI-3/SI-4 : OK** pour les clés projetées et snapshots.
- **Secrets/tokens : OK.** Aucun credential n’est présent dans les trois artefacts fournis.

## GAPs P0/P1

- **P0 : aucun nouveau P0 identifié.**
- **P1 nouveau — lecture intégrale du GGUF pour afficher son digest.** `std::fs::read` charge 16,3 Go après le backend et peut empêcher le tail M2 8 Go de servir : `crates/nexus-shell-daemon/src/main.rs:331-336`. Fix streaming et validation sur le binaire feature-gaté final exigés avant commit.
- **P1 manifeste-binding : carry PO accepté, non recompté comme nouveau.** Il reste techniquement non résolu : l’affichage `{digest,window,role}` est une mitigation opérateur, pas un binding readiness au manifeste. Le review devrait dire « P1 accepté/différé Phase K », pas simplement « 0 P1 survivant ».

## P2/P3

À documenter ou corriger :

- **P2** : artefact raw produit par l’ancien comportement `LAST_RESPONSE`, incompatible avec le harness final.
- **P2** : nextest/Docker encore sans verdict fourni ; le compte nextest attendu paraît être 2084 avec la chronologie documentée.
- **P2** : cleanup FIN uniquement sur succès ; les retours anticipés droppent les streams.
- **P2** : la limite 64 KiB intervient après réception/désérialisation possible d’une frame de 256 MiB.
- **P2** : pas de validation finie/NaN des logits et detokenize byte-token fragile.
- **P2** : `end > n_layer` et `n_ctx=0` peuvent encore atteindre les assertions natives.
- **P2** : `eval` subsiste sur poll/churn ; extraction `result_text` par `sed` reste fragile aux quotes.
- **P2** : frontiere `ShardStepRequest/Reply` absente du protocole documentaire, carry Phase K.
- **P3** : `p95_token_latency_ms` est une moyenne.
- **P3** : participants primaires+fallbacks potentiellement >256.
- **P3** : le test churn ne lance pas un témoin no-churn dans le même test.
- **P3** : `MAX_TOKENS=0` accepté par le validateur.
- **P3** : déterminisme et byte-identité par hôte sont transcrits, sans réponses brutes ou captures de hash par machine.
- **P3** : `review.md` annonce un quatrième fichier Codex à committer alors que le périmètre fourni contient trois untracked.

## Verdict global

**GAP — NO-COMMIT.**

Le cœur inference/decode est convaincant, le P1 fingerprint est réellement fermé et les trois défauts harness ciblés sont corrigés dans le source. Le gate reste toutefois bloqué par :

1. le nouveau P1 `std::fs::read` du GGUF ;
2. l’absence de résultats finaux nextest/Docker et le compteur attendu incohérent ;
3. le T2 raw non réconcilié avec le harness final.

Après hash streaming, smoke-test feature Metal/CUDA, suites vertes avec compteur expliqué et artefact T2 régénéré ou explicitement versionné pré-fix, le verdict pourra passer à PASS sous le carry PO manifeste-binding Phase K.