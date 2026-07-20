La gate est bloquante : aucun P0, mais trois P1 remettent directement en cause le `GO`.

LIVRABLE 1 — CONFIRME — Le schéma impose la PK `(run_id, sequence)` et les unicités d’outbox (`store.rs:50-89`). L’intention, l’incrément de séquence et l’outbox sont atomiques (`store.rs:397-431`) ; completion, outbox et frontière le sont aussi (`store.rs:483-529`). `BEGIN IMMEDIATE` sérialise le lecture-puis-incrément de `next_sequence` (`store.rs:675-727`) et `ensure_run` traite la concurrence de création dans la transaction (`store.rs:292-315`).

LIVRABLE 2 — PARTIEL — Le chemin `SpikeEngine` dérive bien `effect_id` sans champ appelant (`engine.rs:35-51,313-376`) et construit ensuite la clé (`engine.rs:404-408`). Les séparateurs sont réellement interdits (`hash.rs:93-112,178-224`) et le préimage du digest est désormais préfixé par longueur (`hash.rs:299-319`) : U+001F ne collisionne plus le digest, tab/LF/U+001F ne peuvent plus corrompre la TSV via ce chemin. Mais `IdempotencyKey::new` accepte publiquement un `effect_id` choisi (`hash.rs:178-195`) et les mutateurs publics du store acceptent aussi cet identifiant (`store.rs:397-463`). Scénario : un appelant contourne `SpikeEngine`, fabrique une clé choisie et appelle directement la cible ou le store. Correctif : type `DerivedEffectId` à constructeur privé et mutateurs d’écriture `pub(crate)`.

LIVRABLE 3 — CONFIRME — La canonicalisation structurée passe par `canonical_bytes`, avec rejet récursif de `null` (`hash.rs:246-291`) ; step et spec l’utilisent (`hash.rs:294-297`, `engine.rs:53-70`), comme l’echo (`echo.rs:156-159`). Le hash est persisté puis comparé sur les chemins completed et pending (`engine.rs:287-340`) ; la divergence journalise le terminal et renvoie le variant typé (`engine.rs:113-126,292-304,327-339`; `store.rs:531-558`). `Option::None` est effectivement refusé et documenté (`hash.rs:229-245`), même s’il manque un test explicite.

LIVRABLE 4 — CONFIRME — Dans le modèle sérialisé effectivement testé, la cible canonicalise l’entrée, déduplique la clé, append la ligne puis appelle `sync_all()` (`echo.rs:141-170`). Le test vérifie déterminisme et absence de seconde ligne (`store_spike.rs:436-449`). La limite concurrente est documentée plus bas.

LIVRABLE 5 — PARTIEL — Les trois frontières sont bien parcourues in-process (`store_spike.rs:269-324`) puis avec un writer enfant qui termine par `exit(87)` sans déroulement de pile (`store_spike.rs:895-1030`). L’état laissé par le mort est contrôlé avant reprise (`store_spike.rs:923-979`). Mais ce harness ne mesure que les lignes externes, pas le nombre de tentatives moteur. Scénario : crash B2, reprise, second crash B2 ; chaque reprise relit `pending` puis rappelle `apply` (`engine.rs:313-365,404-411`). Une cible dédupliquante conserve une ligne, donc le test reste vert malgré deux ou N tentatives. Correctif : cible instrumentée avec journal fsync distinct des invocations, scénario B2 répété et reprise concurrente ; surtout remplacer la clause par « une intention durable et une clé stable pour toutes les réémissions ». La qualification `"durability_claim": "logic-only"` reste honnête (`lib.rs:36-43`; artefact `:9-22`).

LIVRABLE 6 — GAP — L’artefact reste `GO` tout en répétant la clause moteur fausse (`sprint83_e_durable_spike.json:6,24-31,143`). Son validateur cherche seulement la sous-chaîne `fn <nom>(` et croit les chaînes `"PASS"` (`store_spike.rs:779-830`) ; il ne vérifie ni `#[test]`, ni que les deux bras existent, ni `nextest.exit`. Scénario concret : supprimer le bras rouge de `store_journal_outbox_atomic` (`store_spike.rs:237-258`) laisse le nom, le nombre de tests et les `"PASS"` inchangés ; le validateur reste vert. Correctif : mettre l’artefact en `NO-GO`, lier le résultat à un digest/commit et à une exécution sans retry, imposer un registre exact de scénarios `#[test]` non ignorés et des marqueurs machine-vérifiables pour chaque bras.

## GAPS P0/P1

P0 : aucun.

- P1 — La garantie centrale est fausse sous le sens opérationnel de « tentative ». Après B2, une tentative a déjà atteint `apply`, mais aucune completion n’est durable ; la reprise rappelle explicitement `apply` (`engine.rs:313-365,404-419`). Le garde `updated == 1` intervient seulement après cet appel (`store.rs:505-522`) : il empêche une seconde completion journalisée, pas une seconde tentative externe. La transaction prouve une seule intention/outbox durable, rien de plus. Cette confusion apparaît dans le préflight (`sprint83_phase_e_preflight.md:159-168`) et fonde le `GO` (`sprint83_e_durable_spike.json:25,143`).

- P1 — La même faille devient un double effet réel avec deux reprises simultanées : les deux peuvent lire la même ligne pending avant completion, puis appeler la cible. `FileEchoTarget` effectue un read-check suivi d’un append sans verrou (`echo.rs:148-170`) ; deux processus peuvent donc lire « absent » puis écrire chacun une ligne. L’artefact exclut bien la concurrence multi-writers (`sprint83_e_durable_spike.json:9-14,126-130`), mais cette exclusion contredit la clause moteur non qualifiée. Soit le `GO` devient explicitement « single-writer sérialisé », soit fencing/verrouillage et test concurrent deviennent prérequis.

- P1 — Le contrôle rouge d’ambiguïté d’identité est en trompe-l’œil. Il construit bien deux clés invalides (`store_spike.rs:1082-1089`), puis « démontre » la collision par `assert_eq!` entre deux chaînes littéralement identiques (`store_spike.rs:1090-1093`). Cette assertion ne dépend d’aucun code de production et ne peut jamais rougir. L’artefact affirme pourtant une collision démontrée avec `red_control_kind = guarantee-removed` (`sprint83_e_durable_spike.json:71-77`). Il faut exercer un encodeur brut réel/test-only, puis démontrer le contournement de déduplication ou reclasser correctement le contrôle.

- P1 — L’artefact n’est pas lié aux résultats qu’il atteste. Le validateur contrôle les noms et compte les attributs source (`store_spike.rs:799-880`), mais ne lit même pas `nextest.exit`, ne vérifie pas l’unicité/exhaustivité des scénarios et ne sait pas qu’un bras rouge a été supprimé. La formule « artefact machine-validé » (`sprint83_e_durable_spike.json:89-95`) est donc plus forte que le mécanisme.

## GAPS P2/P3

- P2 — Surface publique fail-open. `SpikeKnobs::without_*` (`fault.rs:108-126`), `FileEchoTarget::lying` (`echo.rs:104-113`), `FaultHook` (`fault.rs:52-64`) et tous les mutateurs du store (`store.rs:397-570`) sont accessibles en production. La review l’assume pour un crate sans consommateur (`sprint83_phase_e_review.md:157-167`) : c’est défendable comme dette temporaire de spike, pas comme frontière de garantie pour les 38 phases suivantes. À rendre `pub(crate)` ou `#[cfg(any(test, feature = "spike-faults"))]` avant le premier consommateur.

- P2 — L’explication de canonicalisation attribue la collision au mauvais composant. Le code provoque d’abord le collapse via `serde_json::to_value` (`hash.rs:259-263`), et le test ne mesure que cette opération (`store_spike.rs:491-497`). `serde_jcs` direct refuse au contraire les floats non finis (`C:/Users/FlowUP/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_jcs-0.2.0/src/lib.rs:604-618`), comme le préflight l’indiquait (`sprint83_phase_e_preflight.md:212-214`). La garde reste fonctionnellement nécessaire dans le pipeline actuel, mais `hash.rs:231-245` et l’artefact `:68` doivent être reformulés. Le contrôle est « anti-vacuity », pas « guarantee-removed ». Ajouter aussi un test direct sur `null` et `Option::None`.

- P2 — `crash_model_not_proven` omet au moins la mort réelle à l’intérieur de la transaction d’intention, les crashs répétés pendant une reprise B2 et l’interruption au milieu du read/append/sync de la cible. Le processus réel n’est tué qu’en B1/B2/B3 (`store_spike.rs:899-912`) ; l’intérieur de transaction n’est couvert que par retour d’erreur et rollback normal (`store_spike.rs:162-196`).

- P3 — Plusieurs formulations réélargissent silencieusement le zéro-double-effet : le plan reste absolu (`sprint83_plan.md:518-534,550-555`) et le commentaire nextest affirme que chaque test tue un run et obtient une seule sortie (`.config/nextest.toml:89-95`), alors que plusieurs tests ne crashent rien. Les clauses honnêtes existent bien dans `lib.rs:17-34` et l’artefact `:24-31`; il faut aligner le reste.

Aucun gap constaté sur les contraintes mécaniques suivantes : membre workspace présent (`Cargo.toml:17-18`), dépendances héritées (`crates/sbfb-workflow-engine/Cargo.toml:11-29`), seul bloc de package ajouté au lock (`Cargo.lock:7523-7535`), registre toujours à sept (`contracts.rs:133-147`), census gelé à 25 (`scripts/check-frontier-contracts.sh:180-199`), SPDX ligne 1 sur chaque `.rs`, chemins injectés (`store.rs:205-224`; `echo.rs:93-129`) et lectures d’environnement confinées au test enfant (`store_spike.rs:1009-1029`). Tous les noms de tests commencent par `store_` et aucun ne contient `effect` (`store_spike.rs:10-14,152-1253`). Je n’ai pas exécuté Cargo/nextest, car ils écriraient dans `target`; les exits annoncés par la review interne ne sont donc pas revalidés dans ce passage lecture seule.

## Ce que la review interne a manqué

La review affirme que tous les P1 sont fermés et qu’aucun livrable n’est en trompe-l’œil (`sprint83_phase_e_review.md:13-21,148-155`). Elle a pourtant manqué :

- la différence fondamentale entre une ligne pending unique et plusieurs appels `apply` après B2 ;
- la course read-then-append de la cible déclarée idempotente ;
- le `assert_eq!` tautologique du contrôle rouge d’identité ;
- l’absence de liaison entre `"PASS"`/`nextest.exit` et l’exécution réelle dans le validateur ;
- la contradiction documentaire `serde_jcs` versus `serde_json::to_value`.

## Verdict global: GAPS

Le `status: GO` doit être retiré ou réécrit autour de la garantie réellement prouvée : une intention durable atomique, une même clé pour toutes les réémissions, et un effet observable unique seulement contre une cible dédupliquante dans un modèle single-writer sérialisé. En l’état, la Phase E ne doit pas débloquer F (`sprint83_plan.md:518-523,550-555`).

