La gate reste bloquée : aucun P0, mais cinq P1 empêchent de conserver le `GO` et de débloquer la Phase F.

LIVRABLE 1 — PARTIEL — Le schéma, la PK, l’outbox et les transactions sont correctement codés (`store.rs:52-89,302-315,407-432,498-533`). En revanche, seul le transactionnel d’intention possède un point de faute interne et un contrôle rouge (`fault.rs:19-40`; `store_spike.rs:159-265`) : `ensure_run`, `record_completion` et `record_run_completed` ne sont pas falsifiés pendant leur transaction. Un split de `record_completion` suivi d’un crash entre l’événement et l’update laisserait soit un journal `completed` avec outbox `pending`, soit un reçu sans événement, sans que le harness actuel puisse injecter ce crash. Correctif : points `InsideRunCreationTransaction`, `InsideCompletionTransaction`, `InsideRunCloseTransaction`, avec rollback vert et chemins scindés rouges.

LIVRABLE 2 — PARTIEL — `SpikeEngine` dérive l’`effect_id`, reprend celui du `pending` et construit la clé validée (`engine.rs:313-419`; `hash.rs:181-224,309-329`). Les tabulations, LF et U+001F sont réellement refusés, et la préimage est désormais préfixée par longueur (`hash.rs:93-112,213-224,313-329`). Mais `IdempotencyKey::new` et `RunStore::record_intent_atomic` restent publics et acceptent un `effect_id` fourni (`hash.rs:178-195`; `store.rs:398-405`). Correctif : `DerivedEffectId` opaque à constructeur moteur privé et mutateurs d’écriture `pub(crate)`.

LIVRABLE 3 — PARTIEL — Les branches `completed` et `pending` recomparent bien le hash et terminalisent avec `StepHashMismatch` (`engine.rs:287-340`; `store.rs:538-563`). Toutefois, le seul test de divergence termine d’abord entièrement le run : il n’exerce que `completed`, jamais le replay `pending` après B1/B2 (`store_spike.rs:368-435`; artefact `:58-64`). Correctif : crash B1 puis reprise avec input divergent, avec variant exact, run terminal, zéro `apply`, et contrôle rouge associé.

LIVRABLE 4 — CONFIRME — Pour une même instance/log, `FileEchoTarget` canonicalise, déduplique, append puis appelle `sync_all()` (`echo.rs:144-175`), et le test vérifie reçu déterministe et une seule ligne (`store_spike.rs:443-505`). Cette confirmation ne couvre pas le remplacement du domaine de déduplication, traité en P1 ci-dessous.

LIVRABLE 5 — PARTIEL — B1/B2/B3 sont parcourus in-process (`store_spike.rs:275-331`) et par un writer enfant qui appelle réellement `exit(87)` (`store_spike.rs:129-140,943-1078`), avec assertions discriminantes de l’état laissé (`store_spike.rs:972-1028`). Mais la reprise out-of-process est exécutée par le parent préexistant, pas par un second processus lancé après la mort : l’unique `Command::new` lance le writer (`store_spike.rs:950-962`), puis le parent construit directement le moteur (`store_spike.rs:1030-1034`). Correctif : lancer un second enfant « recoverer », ou remplacer « processus NEUF » par « processus distinct, sans connexion/store pré-ouvert ».

LIVRABLE 6 — GAP — Le validateur vérifie désormais les vrais `#[test]`, les ignorés, doublons, deux bras, types de contrôle et comptes (`store_spike.rs:786-930`), et ses limites sont honnêtement déclarées (`store_spike.rs:827-831`; artefact `:149-153`). Mais l’artefact conserve `GO` avec une garantie encore fausse (`:6,27-36,148`), et le check sprint nommé « spike durable E » utilise `test(spike_)` (`sprint83_plan.md:2228`) : parmi les scénarios comportementaux, seul `store_spike_artifact_matches_suite` contient `spike_` (`store_spike.rs:736`), contrairement aux crash-tests (`:160,276,368,443,944`). Nextest distingue explicitement `test(...)` du nom du binaire selon sa [référence officielle](https://nexte.st/docs/filtersets/reference/). Supprimer une assertion d’un crash-test peut donc laisser le JSON et ce check verts. Correctif : utiliser la commande complète déjà prescrite en E.4 (`sprint83_plan.md:550-552`) ou `-E 'test(store_)'`.

## Corrections du round 1 : TENUES | NON TENUES

1. **NON TENUE** — L’ancienne « tentative unique » a disparu de `lib.rs` et de l’artefact, mais elle est remplacée par un `AT-LEAST-ONCE` non qualifié qui est faux après B1 pour une cible non idempotente (`lib.rs:21-27`; artefact `:28-32`; `engine.rs:352-364`). Le preflight conserve aussi textuellement l’ancienne clause (`sprint83_phase_e_preflight.md:159-168,349-352`).

2. **TENUE** — La concurrence read-then-append et le modèle single-writer sont explicitement nommés (`lib.rs:33-38`; artefact `:16,31,134`).

3. **TENUE** — `raw_key_line` encode réellement sans validation, les tuples sont distincts et les deux clés sont ensuite refusées (`store_spike.rs:92-97,1139-1163`).

4. **TENUE dans sa limite déclarée** — Le validateur contrôle exactement la forme annoncée et reconnaît explicitement qu’il ne lie pas le JSON à une exécution passée (`store_spike.rs:822-930`; artefact `:149-153`).

5. **TENUE techniquement** — Les constructeurs dégradés et demi-écritures sont derrière `spike-faults`, non défaut (`Cargo.toml:28-39`; `fault.rs:108-132`; `echo.rs:104-116`; `store.rs:435-486`). La review interne est devenue obsolète sur ce point (`sprint83_phase_e_review.md:163-166`).

6. **TENUE fonctionnellement** — La table des trois chemins est présente et exacte (`hash.rs:229-255`), et toute canonicalisation structurée passe par `canonical_bytes` puis `reject_nulls` (`hash.rs:256-301`; `echo.rs:159-162`). La source locale de `serde_jcs 0.2.0` confirme que le wrapper contrôle le scalaire, tandis que les sérialiseurs structurés sont délégués à `serde_json` (`serde_jcs-0.2.0/src/lib.rs:525-555,604-618`). `Option::None` est bien refusé par le code, mais sans test ciblé.

7. **TENUE pour les trois ajouts** — Mort réelle dans la transaction d’intention, crashs B2 répétés et interruption de cible figurent bien dans `crash_model_not_proven` (`artefact:13-15`). La liste reste néanmoins incomplète pour les autres transactions et le changement de domaine cible.

8. **NON TENUE globalement** — `.config/nextest.toml` dit correctement « most of its tests » (`:89-95`), mais le plan garde les formulations absolues « SANS double effet » et « 0 double effet » (`sprint83_plan.md:518-520,533-555`), et le preflight garde l’ancienne tentative bornée (`sprint83_phase_e_preflight.md:159-168`).

## GAPS P0/P1

P0 : aucun.

- **P1 — `AT-LEAST-ONCE` est faux sans qualification.** B1 intervient après l’intention et avant `apply()` (`fault.rs:33-34`; `engine.rs:413-420`). À la reprise, toute ligne `pending` non idempotente devient immédiatement `OutcomeUnknown`, sans appel cible (`engine.rs:320-365`). Donc crash B1 + cible non idempotente = zéro livraison, alors que l’artefact affirme `AT-LEAST-ONCE` (`:28`) et que son test non idempotent ne couvre que B2 après une première sortie (`store_spike.rs:536-580`). Corriger en « réémission sous clé stable lors d’une reprise autorisée contre une cible idempotente »; pour les non-idempotentes, la sémantique est au-plus-une tentative avec zéro possible.

- **P1 — Le domaine de déduplication n’est pas lié.** `FileEchoTarget` porte un `log_path`, mais expose toujours le même nom `"echo"` (`echo.rs:85-101,135-142`). Le hash ne voit que `target.name()` (`engine.rs:280-285`) et la reprise ne compare que le booléen d’idempotence (`engine.rs:341-365`). Scénario single-writer : B2 avec `idempotent(log_A)` écrit une ligne; reprise avec `idempotent(log_B)` passe toutes les gardes, réutilise la même clé et écrit une seconde ligne dans `log_B`. Les deux cibles sont honnêtes et dédupliquent localement. Correctif : persister et hasher un `dedup_scope_id`/`target_instance_id` stable, ou déclarer explicitement l’hypothèse « même backend durable de déduplication » et tester son changement fail-closed.

- **P1 — L’atomicité complète est codée mais non falsifiée.** `record_completion` protège correctement `updated == 1` dans la transaction (`store.rs:498-533`), mais aucun `FaultPoint` n’existe à l’intérieur de cette transaction, de `ensure_run` ou de la clôture (`fault.rs:19-40`; `store.rs:302-315,567-575`). La formule « chaque garantie porte un contrôle rouge » de l’artefact est donc fausse (`:148`). Ajouter les trois points de faute et tester aussi une deuxième completion : erreur typée, event count inchangé, outbox/frontière inchangées.

- **P1 — Les deux propriétés centrales réécrites ne sont pas protégées contre une régression ciblée.** Le test hash ne traverse pas la branche `pending` (`store_spike.rs:368-435`), et les tests de reprise n’assertent jamais directement qu’un seul `EffectIntended` subsiste après reprise (`store_spike.rs:306-330,1030-1054`). Supprimer seulement le check pending ferait appliquer l’input B sous l’`effect_id` dérivé de A; l’update de completion ne remplace pas le `request_hash` de l’outbox (`store.rs:510-520`). Ajouter un scénario divergent après B1/B2 et compter explicitement un unique événement d’intention, idéalement après plusieurs B2.

- **P1 — Le check GO/NO-GO final peut ne lancer que le validateur de forme.** `sprint83_plan.md:2228` utilise `test(spike_)`, alors que la commande complète correcte existe à `:550-552`. Avec les limites admises du validateur (`store_spike.rs:827-831`), cette divergence permet précisément qu’une assertion comportementale disparaisse sans rougir ce check. Aligner toutes les surfaces de gate sur la suite complète.

## GAPS P2/P3

- **P2 — Surface publique résiduelle.** Les désactivateurs sont correctement feature-gatés, mais `FaultHook` reste public et obligatoire dans `execute` (`fault.rs:52-64`; `engine.rs:224-229`), et les mutateurs normaux du store restent publics (`store.rs:398-405,490-497,538-575`). Défendable pour le spike sans consommateur, dette à fermer avant le premier appelant.

- **P2 — « Journal source of truth » et « journalisé » sont trop forts.** `event.rs:4-6` affirme que le journal seul porte les décisions, mais le reçu, le hash pending et `target_idempotent` sont lus dans `outbox` (`store.rs:330-386`), et `RunEventKind` ne porte ni reçu ni booléen d’idempotence (`event.rs:44-75`). Reformuler en « store durable journal + outbox », ou rendre l’outbox reconstructible.

- **P2 — L’exclusion secret/PII n’est pas garantie par le type.** `event.rs:15-18` affirme qu’aucun champ ne peut porter prompt ou secret, mais `run_id` et `node_instance_id` restent des `String` (`event.rs:112-122`) et la validation ne refuse que vide, taille et caractères de contrôle (`hash.rs:93-112`). Un token ou email monoligne est accepté et persisté. Soit utiliser des identités opaques moteur, soit réduire la doc à « aucun champ dédié au payload/receipt ».

- **P2 — `crash_model_not_proven` doit aussi citer la mort réelle dans `ensure_run`, `record_completion` et `record_run_completed`, ainsi que le remplacement/perte du backend de déduplication.** L’artefact ne cite actuellement que la transaction d’intention (`:13-16`).

- **P3 — « processus NEUF » est littéralement suraffirmé**, puisque le parent préexistant reprend. La preuve importante — autre processus que le writer, sans ancienne connexion SQLite — reste solide, mais la formulation doit être exacte (`store_spike.rs:950-1034`; artefact `:8,88`).

- **P3 — La review interne est stale.** Elle affirme que le feature-gate a été écarté (`sprint83_phase_e_review.md:163-166`) alors qu’il est présent, et sa réconciliation Codex reste vide (`:177-179`).

Aucun gap mécanique constaté sur le census `DOMAIN_*` gelé à 25 (`scripts/check-frontier-contracts.sh:180-199`), le registre de sept contrats (`contracts.rs:132-147`), le membre workspace (`Cargo.toml:15-19`), le lockfile qui n’ajoute que le paquet (`Cargo.lock:7523-7536`), les SPDX ligne 1, l’absence d’env/path hôte dans `src/`, ou le nommage `store`/absence de `effect` (`store_spike.rs:10-14,159-1314`; `sprint83_gate_matrix.md:39,46`). La porte de canonicalisation et `durability_claim: logic-only` sont honnêtes (`hash.rs:256-301`; `lib.rs:50-57`; artefact `:9-25`). Je n’ai pas exécuté Cargo/nextest, conformément à la sandbox lecture seule.

## Ce que la review interne a manqué

La review affirme « 6 P1 fermés » et « aucun livrable en trompe-l’œil » (`sprint83_phase_e_review.md:11-21,148-155`). Elle a manqué :

- le zéro-livraison après B1 contre une cible non idempotente, qui invalide `AT-LEAST-ONCE`;
- le remplacement d’un backend de déduplication honnête par un autre backend honnête de même nom;
- l’absence de contrôles rouges pour completion/création/clôture;
- l’absence de test de divergence sur `pending` et de comptage direct de l’intention;
- le filtre `test(spike_)` qui sélectionne le validateur de forme plutôt que le harness comportemental;
- les contradictions documentaires journal/outbox, secret par type et feature-gate.

## Verdict global: GAPS

Le `status: GO` n’est pas accepté. La Phase F ne doit pas être débloquée tant que les P1 ci-dessus ne sont pas corrigés ou que la garantie n’est pas réduite explicitement à ce qui est réellement prouvé (`sprint83_plan.md:550-555`; artefact `:6,148`).

