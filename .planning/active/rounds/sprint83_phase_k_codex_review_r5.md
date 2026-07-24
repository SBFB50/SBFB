Verdict global : **GAP**. Aucun P0. Les quatre modifications annoncées depuis R4 sont bien présentes, mais la fermeture n’est pas complète : deux chemins P1 conservent une identité durable ambiguë. Le critère d’arrêt — CLEAN ou seulement P2/P3 documentables — n’est donc pas atteint.

Assiette auditée : HEAD `a23510e828fab9d7e96370d0811c7d3e39ed9316`, working tree sale, état scellé le 24 juillet 2026 à 00:01:10 CEST après les dernières modifications concurrentes.

## Findings

### [P1] K-PROC-PROGRAM-RESOLVE-05 — le programme relatif reste dépendant du runtime

`ProcessActivity::new` fige `cwd`, mais conserve `program` tel quel :

- stockage brut : [process.rs:242–259](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:242) ;
- scope JCS basé sur la seule épellation du programme : [process.rs:278–298](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:278) ;
- spawn via `Command::new(&self.program)` : [process.rs:306–312](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:306).

Pour un programme relatif, Rust documente explicitement que la résolution par rapport au cwd parent ou au `current_dir` enfant est ambiguë, spécifique à la plateforme et instable ; un chemin absolu est recommandé : [documentation locale `Command`:225–231](/C:/Users/FlowUP/.rustup/toolchains/stable-x86_64-pc-windows-msvc/share/doc/rust/html/std/process/struct.Command.html:225).

Séquence fautive possible :

1. intent avec `program = "bin/tool"` dans le cwd runtime A ;
2. reprise dans le cwd runtime B ;
3. mêmes `program`, `cwd`, `args`, `env` dans le scope ;
4. résolution possible vers un autre exécutable.

Le test réel cwd utilise justement `current_exe()`, donc un chemin absolu, et ne discrimine pas ce cas : [effects.rs:2526–2531](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:2526).

Cela contredit la fermeture annoncée par [lib.rs:108–114](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/lib.rs:108) et `T-RUN-PROCESS-SPAWN-INJECTION` dans [THREAT_MODEL.md:1833](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1833).

Correction attendue : résoudre ou refuser le programme au constructeur, puis utiliser exactement le même chemin absolu pour le scope et le spawn, avec un test reprise A→B utilisant deux exécutables relatifs discriminants.

### [P1] K-FENCE-SCOPE-REL-05 — le backend fenced peut changer sans changer de scope

`FencedFileActivity` accepte un `log_path` relatif sans le figer :

- constructeur brut : [activity.rs:385–393](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/activity.rs:385) ;
- `dedup_scope` égal à l’épellation brute : [activity.rs:478–489](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/activity.rs:478) ;
- lecture et append résolvent à nouveau ce chemin relativement au cwd ambiant : [activity.rs:491–529](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/activity.rs:491).

Or le contrat exige l’identité stable du backend durable : [activity.rs:153–165](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/activity.rs:153). La garde de reprise ne compare que la chaîne journalisée : [orchestrator.rs:1137–1169](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1137).

Avec `effects.log`, un intent dans A puis une reprise dans B gardent le même scope tout en consultant deux journaux différents. Le second backend ne voit ni la clé déjà appliquée ni l’epoch du premier : la déduplication et le fencing peuvent être contournés.

Correction attendue : figer une cible absolue au constructeur — parent canonique plus nom de feuille si le fichier n’existe pas — et ajouter un test de reprise entre deux cwd.

### [P2] K-RAW-PATH-PRECONDITION-05 — préconditions « chemin existant » non imposées et claims absolues

Les deux constructeurs documentent désormais leur fallback raw, mais retournent toujours `Self` sans imposer que la cible existe :

- cwd process raw relatif résolu contre le cwd ambiant au spawn : [process.rs:230–255](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:230) ;
- racine FS raw relative pouvant partager le même scope entre deux cwd : [fs_effect.rs:91–111](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/fs_effect.rs:91).

Pour le FS, deux arbres distincts peuvent parfaitement avoir les mêmes tokens head/index/worktree : la précondition d’état ne remplace donc pas l’identité de la racine.

Les bornes locales sont honnêtes, mais les claims « never ambient » et « fermée » restent plus fortes dans [process.rs:210–215](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:210), [lib.rs:108–114](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/lib.rs:108) et [THREAT_MODEL.md:1832–1833](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1832).

Acceptable en P2 documentable si les claims sont explicitement bornées au chemin existant supporté ; fermeture plus forte si les constructeurs deviennent faillibles et refusent le fallback relatif.

### [P3] K-DOC-WORKFLOW-STATUS-05 — documentation de phase incohérente

- `REFERENCE.md` dit encore que les routes mutantes sont différées vers K : [REFERENCE.md:94–104](/C:/Users/FlowUP/Documents/Code/nexus/docs/workflow/REFERENCE.md:94).
- La décision Phase K actuelle les reroute vers V : [llms.txt:49–56](/C:/Users/FlowUP/Documents/Code/nexus/docs/workflow/llms.txt:49).
- Le même `llms.txt` présente encore J et K comme des phases futures : [llms.txt:11–15](/C:/Users/FlowUP/Documents/Code/nexus/docs/workflow/llms.txt:11).

Aucun impact runtime, mais la Truth Stack rank-1 est incohérente.

## Confirmation des quatre findings R4

| Finding R4 | Verdict | Évidence |
|---|---|---|
| K-PROC-CWD-SCOPE-03 | **PARTIEL** | `cwd` requis, figé, présent dans le JCS et appliqué par `current_dir` : [process.rs:207–312](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:207). Les tests discriminants existent : [effects.rs:1622–1687](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:1622) et [effects.rs:2505–2560](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:2505). Fermeture incomplète à cause du programme relatif P1 et du cwd raw relatif P2. |
| K-FS-ROOT-FREEZE-02 | **PARTIEL** | Pour une racine existante, résolution unique au constructeur et absence de résolution live dans le scope : [fs_effect.rs:105–111](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/fs_effect.rs:105), [fs_effect.rs:230–260](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/fs_effect.rs:230). Le cas raw relatif non résolu reste dépendant de l’ambient, désormais documenté. |
| K-SCOPE-PATH-ENC-03 | **CONFIRME** | JCS injectif avec `root`, `expected_head`, `expected_index_digest`, `expected_worktree_digest`, et borne non-UTF-8 explicite : [fs_effect.rs:230–260](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/fs_effect.rs:230). L’ancien join `0x1f` et la claim « control-free by validation » ont disparu. |
| K-FS-ROOT-FALLBACK-DOC-04 | **CONFIRME** | L’exception raw déjà canonique absolue est correctement décrite, ainsi que le cas supplémentaire des deux racines relatives absentes : [fs_effect.rs:91–104](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/fs_effect.rs:91). |

## Verdict par livrable Phase K

| Livrable | Verdict | Évidence principale |
|---|---|---|
| Contrat `Activity` non bloquant | **CONFIRME** | [activity.rs:150–199](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/activity.rs:150) |
| `effect_id`, intent et garde request/scope | **CONFIRME** | [orchestrator.rs:1137–1169](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1137), [orchestrator.rs:1249–1267](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1249) |
| Retries bornés et re-arm du timer après crash | **CONFIRME** | [orchestrator.rs:1273–1293](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1273), [orchestrator.rs:1557–1564](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1557) |
| Receipt lié et reconciliation | **CONFIRME** dans la frontière locale/non signée déclarée | [orchestrator.rs:391–480](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:391) |
| Effet FS, state-binding et confinement | **PARTIEL** | Chemin nominal confirmé ; fallback relatif P2 : [fs_effect.rs:91–111](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/fs_effect.rs:91), [fs_effect.rs:263–272](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/fs_effect.rs:263) |
| Process effect et identité durable | **GAP** | Programme relatif P1 et fallback cwd P2 : [process.rs:242–312](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:242) |
| Backend fenced/idempotent | **GAP** | Scope relatif P1 : [activity.rs:385–529](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/activity.rs:385) |
| Drain cancel et reap d’incarnation précédente | **CONFIRME** | [orchestrator.rs:1705–1803](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1705) |
| Cleanup descendants | **CONFIRME** dans les bornes Windows/Unix documentées | [process.rs:76–199](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:76) |
| Gardes causales reducer | **CONFIRME** | [reducer.rs:363–510](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:363) |
| Claims/docs de sécurité | **GAP** | Sur-claims process/FS et dérive J/K→V ci-dessus |

## Invariants et validations

- Goldens byte-identiques à `a23510e8` : blobs `9ac8d441…`, `1ba2be10…`, `875166f3…`.
- Census : **29**, `check-frontier-contracts.sh` PASS.
- `RUN_EVENT_FORMAT_VERSION == 1` : [run_event.rs:65](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:65).
- Aucun upcaster neuf : [run_event.rs:390](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:390).
- Cargo.lock : 992 nœuds avant/après, delta **0 nœud neuf**.
- Changement de scope FS : aucun golden, artefact SQLite workflow ou fixture committée ne fixe l’ancien format. `dedup_scope` reste un TEXT opaque dans l’outbox ; changement pré-launch sans migration nécessaire.
- `cargo fmt --all --check` : PASS.
- `git diff --check a23510e8` : PASS, hors avertissements CRLF.
- Inventaire live : **221 tests actifs + 7 ignored**, et non 220+7, à cause du nouveau `process_cwd_canonicalised_in_dedup_scope`.
- Exécutions directement observées : core `111/111`, engine unit `8/8`, plus les tests ciblés state-binding FS, alias FS, env process et alias cwd process.
- Les deux tests utilisant `tempfile` n’ont pas pu être réexécutés dans ce sandbox (`PermissionDenied` sous `%TEMP%`). Leur discrimination est confirmée par inspection, pas par exécution indépendante.
- Les résultats finaux des reruns Windows/Docker annoncés en parallèle n’étaient pas accessibles ; ils ne sont donc pas convertis en PASS dans ce rapport.

**Décision Round 5 : GAP — Phase K non confirmable tant que K-PROC-PROGRAM-RESOLVE-05 et K-FENCE-SCOPE-REL-05 restent ouverts.**

