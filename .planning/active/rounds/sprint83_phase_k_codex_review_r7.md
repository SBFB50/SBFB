Verdict global : **GAP**. Aucun P0. Les deux findings r6 sont correctement traités, mais le contrat « programme absolu = exécutable exactement lié » reste contournable sous Windows par la résolution automatique du suffixe `.exe` de `std::process::Command`. Un nouveau P1 subsiste dans la même classe d’identité durable ; le critère d’arrêt n’est pas atteint et la Phase K ne peut pas être scellée.

Assiette finale du 24 juillet 2026 à 11:25:23 CEST : HEAD `a23510e828fab9d7e96370d0811c7d3e39ed9316`, working tree sale. Les huit fichiers critiques ont gardé les mêmes blob hashes entre les deux captures finales.

## Findings

### [P1] K-PROC-ABS-EXE-SUFFIX-07 — un absolu Windows sans suffixe `.exe` ne désigne pas nécessairement le fichier injecté

Le nouveau gate refuse bien tout chemin non absolu :

- canonicalisation de tout programme lexicalement absolu : [process.rs:273](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:273) ;
- scope fondé sur ce chemin stocké : [process.rs:306](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:306) ;
- refus `!self.program.is_absolute()` avant `Command::new` : [process.rs:334](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:334).

Cela ferme `PATH`, mais pas toute résolution Windows. Pour un chemin complet ne finissant pas, insensiblement à la casse, par `.exe`, Rust 1.95 :

1. annonce qu’il suppose d’abord que `.exe` était omis, puis retombe sur le nom complet sans suffixe : [windows.rs.html:452](/C:/Users/FlowUP/.rustup/toolchains/stable-x86_64-pc-windows-msvc/share/doc/rust/html/src/std/sys/process/windows.rs.html:452) ;
2. ajoute réellement `.exe`, choisit ce sibling s’il existe, sinon retourne le chemin initial : [windows.rs.html:482](/C:/Users/FlowUP/.rustup/toolchains/stable-x86_64-pc-windows-msvc/share/doc/rust/html/src/std/sys/process/windows.rs.html:482), [windows.rs.html:491](/C:/Users/FlowUP/.rustup/toolchains/stable-x86_64-pc-windows-msvc/share/doc/rust/html/src/std/sys/process/windows.rs.html:491).

La [documentation officielle de `Command::new`](https://doc.rust-lang.org/stable/std/process/struct.Command.html#method.new) confirme également que le suffixe `.exe` peut être omis sous Windows.

Scénario discriminant :

- l’hôte injecte l’absolu `D:\bin\tool`, fichier existant et donc canonicalisé exactement ;
- `D:\bin\tool.exe` existe aussi ;
- le scope journalise `D:\bin\tool`, mais le premier spawn sélectionne `D:\bin\tool.exe` ;
- après un échec retryable, si le sibling `.exe` disparaît, le même `ProcessActivity`, sous le même `dedup_scope`, retombe sur `D:\bin\tool`.

Le problème existe même sans reprise : si les deux fichiers existent, le programme exécuté n’est déjà pas celui que l’hôte a fait canonicaliser. `\\?\C:\x` est donc bien absolu, mais reste vulnérable à cette bifurcation si `x` ne finit pas par `.exe`.

Les tests actuels ne discriminent pas ce cas : le test de canonicalisation utilise explicitement `prog.exe`, puis les deux tests de refus ne couvrent que relatif et nom nu : [effects.rs:1689](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:1689), [effects.rs:1719](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:1719), [effects.rs:1765](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:1765).

Correction cohérente avec la réduction de surface : sous Windows, refuser également tout chemin stocké post-canonicalisation qui ne finit pas par `.exe` sans tenir compte de la casse. Ajouter un test Windows `process_absolute_without_exe_refused` vérifiant `RunState::Failed` et l’unique `EffectAttemptFailed{retryable:false}` ; mutation active = supprimer ce second gate, le test doit devenir rouge tandis qu’un `current_exe()` absolu reste vert.

Les claims « chemin absolu figé », « le moteur ne résout rien » et « fermeture par fixation constructeur » restent donc trop fortes : [process.rs:210](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:210), [lib.rs:113](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/lib.rs:113), [THREAT_MODEL.md:1833](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1833).

## Traitement des deux findings r6

| Finding r6 | Verdict r7 | Évidence |
|---|---|---|
| `K-PROC-BARE-PATH-AMBIENT-06` | **CONFIRME** | `C:tool`, nom nu et rooted-sans-drive sont refusés par [process.rs:343](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:343). Sous Windows, `is_absolute` exige racine **et** préfixe : [windows.rs.html:216](/C:/Users/FlowUP/.rustup/toolchains/stable-x86_64-pc-windows-msvc/share/doc/rust/html/src/std/sys/path/windows.rs.html:216). Le test vérifie `Failed` et exactement `[false]` : [effects.rs:1775](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:1775). La fermeture reste néanmoins incomplète à cause du nouveau P1 suffixe. |
| `K-FENCE-BARE-LEAF-06` | **CONFIRME** | Parent vide remplacé par `.` puis canonicalisé : [activity.rs:394](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/activity.rs:394). Scope et I/O utilisent ensuite le même chemin figé : [activity.rs:522](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/activity.rs:522), [activity.rs:526](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/activity.rs:526). Test bare/absolu : [effects.rs:1819](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:1819). |

La substitution `.` suit bien le contrat construction-time de `cwd`/`worktree_root`. Aucun `set_current_dir` n’existe dans les sources ou tests engine/core ; l’invariant du binaire de test est explicitement documenté : [effects.rs:881](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:881), [effects.rs:1826](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:1826). Un autre binaire de tests est un autre processus et ne peut pas modifier ce cwd. Seul un futur `set_current_dir` dans ce même binaire rendrait ces assertions concurrentes flaky.

## Verdict par livrable Phase K

| Livrable | Verdict | Évidence |
|---|---|---|
| `effect_id` stable, tentatives distinctes, retries/backoff/timeouts | **CONFIRME** | Dérivation length-prefixed [hash.rs:310](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/hash.rs:310), PK par tentative [store.rs:241](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:241), re-arm crash [orchestrator.rs:1273](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1273). |
| Receipt local structuré et réconciliation liée | **CONFIRME** selon DS-1 | Tuple structuré [activity.rs:201](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/activity.rs:201), vérification run/node/effect/request [orchestrator.rs:440](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:440). |
| `outcome_unknown` jamais transformé sans preuve | **CONFIRME** | Ordre intent/park/reconcile/completion et égalité d’`effect_id` : [reducer.rs:377](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:377), [reducer.rs:475](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:475). |
| FS lié à HEAD/index/worktree et confiné | **CONFIRME** | Scope d’état [fs_effect.rs:230](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/fs_effect.rs:230), précondition avant mutation [fs_effect.rs:263](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/fs_effect.rs:263), confinement [fs_effect.rs:142](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/fs_effect.rs:142). |
| Processus lié à une commande stable | **GAP — P1** | L’absolu sans `.exe` peut sélectionner le sibling `.exe`, puis retomber sur le fichier sans suffixe. |
| Nettoyage des descendants | **CONFIRME dans les bornes documentées** | Job Object Windows [process.rs:8](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:8), process-group Unix et résidus [process.rs:18](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:18). |
| Trait `Activity`, drain cancel, orphan reap | **CONFIRME** | Drain `UnknownAttempt` vers handle journalisé [orchestrator.rs:1705](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1705), [orchestrator.rs:1769](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1769). |
| Fencing cible de référence | **CONFIRME dans sa borne mécanisme** | Refus epoch inférieur et dédup sur chemin figé [activity.rs:526](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/activity.rs:526) ; atomicité production honnêtement exclue [activity.rs:19](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/activity.rs:19). |
| Reroutage documentaire des routes mutantes | **CONFIRME** | [REFERENCE.md:94](/C:/Users/FlowUP/Documents/Code/nexus/docs/workflow/REFERENCE.md:94), [LOOPBACK_ENDPOINTS_TRUST_TIERS.md:103](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md:103). |

## Régression r1–r5

Aucune autre régression observée :

- receipt parsable et lié : [orchestrator.rs:440](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:440) ;
- état FS dans le scope et CAS avant écriture : [fs_effect.rs:230](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/fs_effect.rs:230) ;
- égalité causale d’`effect_id` : [reducer.rs:382](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:382) ;
- re-arm du timer perdu : [orchestrator.rs:1273](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1273) ;
- drain cancel de l’orphan : [orchestrator.rs:1769](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1769) ;
- canonicalisations racine/cwd/program/log et refus relatif présents ;
- le nouveau P1 est une sous-branche Windows jusque-là non testée de la canonicalisation programme.

## Invariants et validations

- **Goldens byte-identiques à `a23510e8` — CONFIRME** :
  - `run_event.golden.json` : `9ac8d44167bc472f93330b0c679b455c08ec4e72` ;
  - `run_event_orchestrated.golden.json` : `875166f3cc273a0c450042e71c38b569c7824b37` ;
  - `run_event_terminal.golden.json` : `1ba2be10a96ee3597158d2a7861e57d48e7c80b1`.
- **`RUN_EVENT_FORMAT_VERSION == 1` — CONFIRME** : [run_event.rs:50](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:50).
- **Upcasters vides — CONFIRME** : [run_event.rs:379](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:379).
- **Census 29 — CONFIRME** : `check-frontier-contracts.sh` exécuté via Git Bash compatible, exit 0, `DOMAIN census [29 frozen]`.
- **Zéro nouveau nœud lock — CONFIRME** : `992` packages avant/après ; diff lock limité à `libc` et `windows 0.57.0` déjà résolus.
- **Inventaire courant — CONFIRME** : sept binaires de tests compilés après les sources, `233` découverts, `7` ignored, donc `226` actifs ; les deux tests r6 sont présents.
- **Rejoués directement** : `cargo fmt --all --check`, frontier gate et `git diff --check`, tous exit 0.
- Les réussites runtime Windows/Docker et les 14 mutations sont consignées dans [sprint83_phase_k_review_r4.md:113](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_k_review_r4.md:113), mais n’ont pas été indépendamment rejouées dans ce sandbox lecture seule. Les anciens logs `target/phasek/*` datent de juin et n’ont pas été utilisés comme preuve du run actuel.

## Claims fermés

Le seul groupe de claims encore plus fort que le code est désormais :

- « programme absolu figé » ;
- « le moteur ne résout rien » ;
- fermeture `T-RUN-PROCESS-SPAWN-INJECTION` par fixation constructeur.

Un absolu ferme la recherche `PATH`, mais la std Windows continue à résoudre le choix `path.exe` versus `path`. Les bornes fenced bare-leaf, raw-parent jamais résolu, receipts locaux non signés, atomicité du backend de référence, limites Unix et TOCTOU FS restent alignées avec le code.

**Décision Round 7 : GAP — scellement refusé. P0=0, P1=1, P2=0, P3=0. Après réduction Windows aux chemins post-canonicalisation finissant par `.exe`, test/mutation et réexécution des gates, un ROUND 8 distinct est requis ; r7 ne doit jamais être réécrit.**

Le sandbox étant en lecture seule, aucun fichier `sprint83_phase_k_codex_review_r7.md` n’a été écrit ; le présent bloc est le rapport brut.

