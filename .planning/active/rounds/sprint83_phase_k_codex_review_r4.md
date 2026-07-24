Verdict global : **GAP**. Aucun P0. Le correctif `K-FS-ROOT-REL` ferme correctement le cas visé — racine résoluble, cwd stable entre comparaison et usage — et le test neuf est discriminant. En revanche, un **P1 distinct** subsiste dans l’identité durable des processus : le cwd hérité influence le programme et ses effets sans être fixé ni incorporé au `dedup_scope`. Le critère d’arrêt « CLEAN ou seulement P2/P3 » n’est donc pas atteint.

## Sévérités

### [P1 K-PROC-CWD-SCOPE-03] Le scope process omet une entrée sémantique normale : le cwd

`ProcessActivity` conserve programme, arguments, environnement et politique d’échec, mais aucun répertoire de travail : [process.rs:207](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:207). Son JCS ne contient que `program`, `args`, `env` et `failure_retryable` : [process.rs:257](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:257). Le spawn construit ensuite `Command` sans aucun `current_dir` : [process.rs:278](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:278).

Séquence fautive concrète :

1. intent journalisé dans le cwd A ;
2. runtime repris dans le cwd B ;
3. même name/input, donc même request hash : [orchestrator.rs:1105](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1105) ;
4. même JCS process, donc la garde `TargetScopeChanged` passe : [orchestrator.rs:1163](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1163) ;
5. un programme relatif peut désigner un autre exécutable, et même un programme absolu peut produire ses I/O relatives dans un autre arbre.

C’est exactement la classe durable que le scope prétend fermer. Le test nominal autorise explicitement un process idempotent : [effects.rs:2393](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:2393).

Les claims sont donc trop forts :

- « everything that changes observable work » : `process.rs:258-260` ;
- « no hidden host contract » : [lib.rs:106](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/lib.rs:106) ;
- `T-RUN-PROCESS-SPAWN-INJECTION` « fermée » : [THREAT_MODEL.md:1833](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1833).

Fermeture attendue : cwd explicite injecté, figé/canonicalisé, inclus dans le JCS et appliqué via `Command::current_dir`; ou refus explicite des programmes/effets dépendant du cwd.

### [P2 K-FS-ROOT-FREEZE-02] La racine relative est résolue deux fois, pas figée

`dedup_scope` canonicalise la racine à la demande : `fs_effect.rs:219-222`. `confine` canonicalise de nouveau le `PathBuf` brut : `fs_effect.rs:145-148`.

Une modification process-wide du cwd entre la garde de scope et `start/confine`, ou un retargeting du symlink racine dans cette fenêtre, peut donc faire comparer l’arbre A puis utiliser l’arbre B. Le test reconnaît désormais explicitement que `canonicalize` consulte le cwd vivant et exige qu’aucun test n’appelle `set_current_dir` : [effects.rs:881](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:881).

C’est un cas non nominal et proche de la borne TOCTOU déjà déclarée, donc P2, mais le fix n’est complet qu’avec cwd stable. Une racine absolue résolue une seule fois supprimerait cette dépendance.

### [P2 K-SCOPE-PATH-ENC-03] L’encodage des chemins n’est pas injectif sur tout le domaine `PathBuf`

Le scope FS utilise `root.display()` puis un séparateur `0x1f` : `fs_effect.rs:223-228`. Les trois états sont des `String` publics sans validation : `fs_effect.rs:42-49`, malgré le commentaire « control-free by validation ».

Deux conséquences :

- sur Unix, `Path::display()` est lossy pour les chemins non UTF-8 : deux racines distinctes peuvent produire la même chaîne ;
- deux états contenant `0x1f` peuvent déplacer les frontières du format et produire le même scope.

Le process passe également `program.display().to_string()` avant le JCS : `process.rs:265-270`. Le JCS reste injectif sur les chaînes obtenues, mais ne répare pas une conversion de chemin déjà lossy.

P2 car cela exige un chemin natif atypique ou un état host invalide, mais l’affirmation d’injectivité générale est trop large.

### [P3 K-FS-ROOT-FALLBACK-DOC-04] Le fallback brut n’est pas systématiquement fail-closed

Le fallback n’incorpore aucun marqueur `raw/canonical` : `fs_effect.rs:219-228`.

- Racine relative non résolue puis résolue : brut relatif versus canonique absolu, donc `TargetScopeChanged` — fail-closed.
- Racine absolue déjà sous sa forme canonique, particulièrement Unix : la chaîne brute peut être identique à la future chaîne canonique. Le resume ne produit alors aucun `TargetScopeChanged`.
- `confine` n’échoue que si la racine est encore absente lorsqu’il s’exécute : `fs_effect.rs:145-148`.

Sous la clause E, le même chemin absolu reste le même token déclaré et la précondition d’état continue de protéger la sémantique ; ce n’est donc pas un P1. Mais `fs_effect.rs:215-217` sur-affirme que « l’effet échoue de toute façon ». Il faut documenter cette borne ou préfixer la forme par son statut.

## Verdict par livrable

| Livrable | Verdict | Évidence |
|---|---|---|
| Trait non bloquant `Activity`, voie E gelée | **CONFIRME** | `activity.rs:150-195`; séparation driver à `orchestrator.rs:1176-1230`. |
| Retries, deadline, budget, re-arm crash | **CONFIRME** | Re-arm avant pacing à `orchestrator.rs:1273-1293`; timer normal à `orchestrator.rs:1557-1564`; test `effects.rs:516`. |
| `outcome_unknown` et réconciliation | **CONFIRME** | Receipt obligatoire et parsé, bindings run/node/effect/request à `orchestrator.rs:440-477`; terminal absorbant à `orchestrator.rs:1498-1539`. |
| Receipt local DS-1 | **CONFIRME dans la portée adaptée** | Non signé et non revérifié au read explicitement borné : `activity.rs:30-44`, `THREAT_MODEL.md:1831`. |
| FS lié à l’état et confinement | **PARTIEL** | État dans le scope et canonicalisation nominale : `fs_effect.rs:204-230`; garde réelle : `fs_effect.rs:232-257`. Résiduels P2/P3 ci-dessus. |
| Scope et stabilité du process | **GAP** | P1 cwd absent : `process.rs:207-220`, `process.rs:257-293`. |
| Nettoyage descendant | **CONFIRME dans les bornes documentées** | Windows Job Object : `process.rs:94-114`, `process.rs:384-418`; Unix process-group/reap : `process.rs:77-92`, `process.rs:141-190`. |
| Cancel drain/reap/propagation | **CONFIRME** | `UnknownAttempt → reap_orphan` : `orchestrator.rs:1769-1783`; autres erreurs propagées : `orchestrator.rs:1796-1800`. |
| Reducer pur et causalité `effect_id` | **CONFIRME** | Fonction `(plan, events)` : `reducer.rs:269-277`; intent/park/reconcile liés : `reducer.rs:445-499`. |
| Documentation de sécurité | **GAP** | `THREAT_MODEL.md:1833` et `lib.rs:106-112` ne bornent pas le cwd ambiant. |
| Wire/census/goldens/lock | **CONFIRME** | Version 1 : `run_event.rs:65`; upcasters vides : `run_event.rs:390`; census 29 : `check-frontier-contracts.sh:202-255`. |

## Test neuf

`fs_effect_root_canonicalised_in_dedup_scope` est **non vacueux** :

- relatif `../<cwd-name>` et absolu du même dossier doivent être égaux : [effects.rs:886](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:886) ;
- retirer `canonicalize` rend bien les deux chaînes différentes ;
- le voisin state-binding reste indépendant.

Il passe sur les deux binaires Phase K construits après l’ajout. Sous nextest, l’isolation process protège le cwd. Sous `cargo test`, le binaire est partagé, mais aucun appel réel à `set_current_dir` n’existe dans `effects.rs`; la dépendance est maintenant documentée à `effects.rs:881-884`.

Limite : c’est une preuve d’aliasing du token, pas un scénario end-to-end `intent → changement de cwd → resume → TargetScopeChanged`.

## Invariants et validations

- `cargo fmt --all --check` : PASS.
- `git diff --check a23510e8` : PASS.
- `check-frontier-contracts.sh` : PASS, census **29**.
- Trois goldens run-event : hashes Git bruts identiques à `a23510e8`.
- `Cargo.lock` : **992 → 992** nœuds, aucun package ajouté ou supprimé.
- Inventaire binaire courant : **224 découverts = 218 actifs + 6 ignorés**, conforme à l’attendu.
- Exécutés : core **111/111**, engine unit **8/8**, plus les quatre tests ciblés scope/root/bound.
- La suite complète n’a pas pu être réexécutée dans ce sandbox lecture seule : Cargo est refusé sur `target/debug/.cargo-lock`, puis les intégrations sur la création des `tempdir`. Ces échecs sont environnementaux, pas des assertions produit, mais ils interdisent de revendiquer un nouveau nextest complet dans ce round.
- `HEAD` final : `a23510e8`. Worktree toujours sale. Un changement concurrent limité au commentaire `effects.rs:881-884` a été détecté, relu et incorporé ; aucun changement de production observé ensuite.

**VERDICT FINAL SPRINT 83 PHASE K : GAP.** `K-FS-ROOT-REL` est confirmé sur sa trajectoire nominale, mais `K-PROC-CWD-SCOPE-03` bloque encore un `## Verdict: PASS`.

