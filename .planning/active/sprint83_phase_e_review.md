# Sprint 83 Phase E — Review

**Phase** : E — Spike durable BLOQUANT (GO/NO-GO), crate NEUF `crates/sbfb-workflow-engine`.
**Méthode** : fan-out avant-plan de 5 agents de review Opus 4.8 1M (une dimension chacun,
arbre sale ⇒ mode synchrone §4.5), puis **3 rounds Codex GPT-5.6 Sol** en vérification
croisée externe. D1 protocole durable, D2 sécurité profonde, D3 sémantique des tests **avec
mutation active du code de production**, D4 livrables/scope/conformité preflight, D5
patterns/docs-contrat.

Verdicts 1re passe : D1 CONCERN (3 P1), D2 CONCERN (1 P1 + 5 P2), D3 CONCERN (2 P1),
D4 CONCERN (4 P2), D5 CONCERN (5 P2, 0 P0/P1).
Codex : round 1 GAPS (4 P1) → round 2 GAPS (5 P1) → round 3 GAPS (5 P1) → round 4 GAPS
(1 P1) → **tous fermés, critère d'arrêt « CLEAN ou P2/P3 documentés » atteint**.

## Verdict: PASS

0 P0. **21 P1 trouvés, 21 P1 fermés**, chacun re-mesuré. Suite du crate 11 → **18 tests
exécutés** (+ 1 `#[ignore]`), workspace 2149 → **2167**.

Ce que cette review a réellement produit, et qui n'était visible dans aucune suite verte :
**deux garanties déclarées `PASS` n'étaient prouvées par aucune assertion**, **un contrôle
rouge était une tautologie**, et surtout **la clause centrale de la phase était fausse — deux
fois de suite**. Sur une primitive dont 38 phases dépendent, c'est la formulation qui s'est
révélée le point faible, pas l'implémentation.

## Les P1 de la review interne (6)

1. **D3-P1-1 — l'atomicité journal+outbox n'était prouvée par aucun test.** Mutation
   mesurée : scinder `record_intent_atomic` en deux transactions laissait la suite
   **entièrement verte**. Le point de faute n'existait que sur le chemin dégradé, donc le
   chemin garanti n'avait aucune fenêtre où crasher. L'assertion
   `hits(BetweenOutboxAndIntentEvent) == 0` était trivialement vraie.
   → `FaultPoint::InsideIntentTransaction` fire DANS la transaction ; le halt provoque un
   `ROLLBACK` et le test assert que ni l'entrée ni la ligne ne survivent. **Mutation
   rejouée : ROUGE.**
2. **D3-P1-2 — le bras out-of-process était vacueux.** Mutation mesurée : un enfant sortant
   en 87 **sans rien faire** laissait le test vert aux 3 frontières.
   → état du writer mort asserté **par frontière**. **Mutation rejouée : ROUGE.**
3. **D1-P1-3 / D2-P1-1 — la grandeur falsifiable était forgeable.** Mesuré par sonde
   compilée : un TAB désactive la déduplication (double effet, compteur à `0`) ; un LF rend
   le log illisible pour tous les runs ; une chaîne choisie écrit à l'avance la ligne de
   dedup d'un autre run → reçu **forgé** que le moteur journalise comme un succès.
   → `validate_identity_component` fail-closed + champs privés.
4. **D2-P2-1 (promu) — `derive_effect_id` préfixe-ambigu**, collision mesurée, neutralisée
   seulement **par accident**. → composants **length-prefixed**.
5. **D1-P1-1 / D1-P1-2 — run non lié à sa clôture ni à sa spec.** Run `Completed`
   extensible ; spec tronquée renvoyant `Ok`. → `spec_digest` journalisé,
   `RunAlreadyCompleted`, `assert_distinct_nodes`.
6. **D2-P2-2 (promu) — promesse d'idempotence de la cible non journalisée.** → journalisée
   à l'intention, écart au resume fail-closed.

## Les P1 Codex (14, sur 3 rounds)

**Round 1 (4)** — (a) la clause « au plus UNE **tentative** non confirmée par clé » est
**FAUSSE** : après B2 l'effet a déjà été tenté et chaque reprise rappelle la cible ; ce qui
est borné est l'INTENTION et la stabilité de la CLÉ. (b) Le modèle de concurrence
(single-writer sérialisé) était relégué en note au lieu d'être nommé dans la garantie.
(c) **Mon contrôle rouge d'identité était une tautologie** : un `assert_eq!` entre deux
littéraux identiques, présenté comme une démonstration de collision — dans la phase dont le
sujet entier est qu'un test vert peut ne rien valoir. (d) L'artefact se disait
« machine-validé » plus fortement que son mécanisme.
→ trois clauses explicites, `raw_key_line` (encodeur non validé) pour une vraie
démonstration, validateur exigeant de vrais `#[test]` + limites déclarées.

**Round 2 (5)** — (a) ma reformulation « AT-LEAST-ONCE » était **elle-même** sur-affirmée :
crash B1 + cible non idempotente = **zéro** livraison. (b) **Le domaine de déduplication
n'était lié à rien** : deux cibles honnêtes, même nom, même promesse, backends différents →
deux sorties externes. (c) L'atomicité de completion était codée mais **non falsifiée**.
(d) La branche `pending` du contrôle de divergence n'était **jamais** exercée, et rien
n'assertait qu'un resume ne mint pas un second `effect_intended`. (e) Le check GO/NO-GO du
plan utilisait `test(spike_)`, qui ne sélectionne que le validateur de forme.
→ `dedup_scope()` journalisé + `TargetScopeChanged`, `InsideCompletionTransaction`,
3 tests neufs, plan aligné sur la suite complète.

**Round 3 (5)** — (a) « ≥ 1 livraison » est une promesse de **liveness** que le code ne peut
pas tenir (`apply` peut échouer indéfiniment) → réduit à « ré-émission sous clé stable ».
(b) Le scope de dedup est un **token déclaré**, pas le backend physique (deux cwd, même
chemin relatif) → la garantie dit maintenant exactement cela, et la responsabilité de rendre
le token discriminant est nommée côté cible. (c) « chaque garantie a un switch qui la
retire » restait faux pour la completion → `without_atomic_completion` + bras rouge asserant
l'état incohérent (journal `completed`, outbox `pending`). (d) Le plan gardait « 0 double
effet » inconditionnel et le preflight son ancienne clause → alignés, l'ancien texte
**annoté comme réfuté** plutôt que réécrit. (e) `format_version` n'était pas persistée :
`load_events` réinjectait la constante courante → colonne ajoutée, valeur **lue** depuis la
ligne (« une version que le lecteur fournit n'est pas une version »).

## Un fait Codex réfuté par la mesure

Codex (round 2) attribuait le collapse des floats non finis à `serde_json::to_value` et
affirmait que `serde_jcs` les refuse. **Trois chemins mesurés** : un float **nu** au
top-level renvoie bien `Err` — mais **imbriqué dans une struct**, le seul cas que ce crate
hache, `NaN`/`+Inf`/`-Inf` s'effondrent tous sur `null` **sans erreur**, avec ou sans
`to_value`. La garde reste nécessaire ; la doc porte désormais le tableau des trois chemins.
Le preflight portait la même erreur (présentée comme vérifiée en lecture de source) : elle y
est **corrigée en place, avec la trace**, parce qu'une lecture de code ne remplace pas une
mesure.

## Vérification §7.4 (les 3 blocs)

- Rust : `cargo nextest run --workspace --locked` **2167** ; crate seul 18/18 + 1 skipped ;
  `fmt --check` 0 ; `clippy --workspace --all-targets -D warnings` 0 ; doctests 0 ;
  `build -p nexus-shell-daemon --release` OK.
- Frontend : web vitest **529**, lint 0 erreur, `tsc` 0, build, `size` 6/6, scan-en-strings
  clean ; operator vitest **201**. T1 = `N-A-no-frontend-change` (0 diff `web/`).
- Gates : `check-frontier-contracts.sh` exit 0 (**census DOMAIN 25**, `len == 7` intact,
  anti-promesse) ; `check-spdx.sh` 386 ; `check-factory-docs.sh` exit 0.
- Matrice de gates, mesurée : `… store` **18 tests exit 0** (G-STORE non-vacuous) ;
  `… effect` **EXIT 4** (G-EFFECT, propriété de K) ; `… single_writer` **EXIT 4**.
- Feature-gate vérifié : `cargo check -p sbfb-workflow-engine` (sans feature) **ne compile
  pas** `without_*`, `lying`, `record_intent_*_only`, `record_completion_event_only`.
- `Cargo.lock` : **+15/−0, un seul nœud** — 0 dépendance runtime neuve.
- Flakes connus re-run SOLO : `dispatch_loop::boot_path_reenters_sync_set…` (5,4 s),
  `convergence_remote_write_visible_to_local_subscriber` (1,9 s),
  `boot_storage_namespace_reenters…` (4,3 s) — carry STANDING, pas des régressions de phase.

## Livrables E.2

Tous présents, aucun en trompe-l'œil : journal append-only PK `(run_id, sequence)` avec
version **persistée** · outbox transactionnel + index unique par node · clé d'idempotence
dérivée moteur, length-prefixed · hash de requête re-comparé sur **les deux** branches ·
echo-target déterministe dont la sortie est un append **fsync'd hors SQLite** · harness de
crash aux 3 frontières **deux fois** (in-process ET mort réelle du processus) · artefact
GO/NO-GO machine-validé, limites déclarées · goldens `RunEvent` v0 (happy + terminal),
round-trippés par le TYPE.

## Frontière §6.12 — pas de frontière, jugée par le TEST-ACTEUR

Les seuls lecteurs sont le Rust de ce crate, ses tests, et **un processus ultérieur exécutant
le même code** (c'est ce qu'est le fichier SQLite) — la persistance n'est pas un acteur.
0 route, 0 wire, 0 miroir TS, 0 contrat d'app. **Aucune étiquette ni clôture docs-contrat
due**, cohérent avec la Phase D. `lib.rs` est ordonné pour que le test-acteur tranche et que
le fait wire soit explicitement **accessoire** — le lire POUR le critère est la conflation
qui a laissé de vraies frontières non documentées pendant un sprint entier.

## Scope cuts (consignés dans le body)

Bornes/rétention/GC → G-STORE. Coût quadratique du log de sorties externes → déclaré.
Fencing single-writer → G-SINGLE-WRITER ; le modèle single-writer sérialisé est **nommé dans
la garantie**. Snapshots/upcasters/backup → hors périmètre. THREAT_MODEL : report explicite.
Confidentialité at-rest déléguée au répertoire injecté. `effect_id` reste injectable via
l'API bas-niveau (la garantie « dérivé moteur » vaut pour `SpikeEngine::execute`) : dette
déclarée, à fermer avant le premier consommateur. `§P74` est PRIS → viser **`§P76`**.

## Carry closure

Ouvre `G-AUTHORITY` (fermé J) et `G-STORE` (fermé I). Ferme **B8** (echo-target
déterministe). `WA-T1`/`WA-T5` restent `NOT-RUN` jusqu'à AI2. E ne ferme aucun gate.
`CLAUDE.md` : la formule « 0-skip » devient fausse — `store_child_writer_exits_mid_run` est le
**premier `#[ignore]` non feature-gaté du dépôt** ; le compteur s'écrit « +18 exécutés
+ 1 ignoré ».

## Codex reconciliation

**Round 4 (1 P1)** — le bras rouge de l'atomicité de completion **restait vacuable** :
il assertait `pending_effect().is_some()` et `completed_effect().is_none()`, or
`completed_effect` ne lit que l'outbox, jamais le journal. Supprimer complètement
l'écriture du chemin dégradé laissait donc le bras vert — il aurait prouvé l'ABSENCE d'une
écriture au lieu de l'INCOHÉRENCE entre deux écritures. Fermé par une assertion directe sur
les événements (exactement un `EffectCompleted`, `event_count == 4`, frontière immobile) +
les assertions manquantes sur `atomic_completion` dans `store_knobs_default_is_guaranteed`.
**Contre-mutation de Codex rejouée : le test ROUGIT**, restauration → vert. P2/P3 du round 4
traités (« processus NEUF » résiduel, « journalisée plutôt que redemandée » propagé,
portée de la formule « chaque garantie a un contrôle rouge » resserrée dans `lib.rs`).

4 rounds joués sur l'arbre **corrigé à chaque fois** (leçon Phase C : un artefact Codex périmé
fait réfuter des corrections déjà faites). 14 P1 au total, **tous fermés** : 8 par correction
de code (validation d'identité, length-prefixing, `spec_digest`, `dedup_scope`, deux points de
faute intra-transaction, `without_atomic_completion`, `format_version` persistée), 6 par
correction de formulation (clause de garantie réécrite deux fois, sémantique de livraison
qualifiée par la nature de la cible, token de scope vs backend physique, plan et preflight
alignés, artefact et ses limites). Les P2/P3 résiduels sont documentés dans l'artefact
(`disclosed_residuals`, `artifact_validation_limits`, `crash_model_not_proven`) plutôt que
fermés : surface publique bas-niveau, absence de point de faute dans les transactions non
composites, égalité de token ne prouvant pas l'identité du backend. Un fait Codex a été
**réfuté par la mesure** (cf. section dédiée) — la vérification croisée s'applique dans les
deux sens.
