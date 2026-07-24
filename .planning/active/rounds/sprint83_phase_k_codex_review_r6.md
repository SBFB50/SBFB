Verdict global : **GAP**. Aucun P0. Les correctifs documentaires r5 #3 et #4 sont confirmés, et les branches « programme absolu » / « relatif avec séparateurs » sont correctes. Mais le nom de programme nu reste dépendant du `PATH` ambiant du processus parent sous Windows : un P1 demeure. Le correctif du journal fenced est également incomplet pour le cas exact `effects.log`, désormais documenté mais non figé. Le critère d’arrêt n’est donc pas atteint et la Phase K ne peut pas être scellée.

Assiette scellée le 24 juillet 2026 à 00:41:30 CEST : HEAD `a23510e828fab9d7e96370d0811c7d3e39ed9316`, working tree sale. Les fichiers critiques n’ont pas changé entre les deux captures finales.

## Findings

### [P1] K-PROC-BARE-PATH-AMBIENT-06 — le nom nu consulte encore le `PATH` ambiant Windows

Les branches suivantes sont confirmées :

- programme absolu canonisé uniquement lorsqu’il est absolu : [process.rs:263-279](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:263) ;
- programme, cwd, arguments, environnement et politique inclus dans le scope JCS : [process.rs:302-327](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:302) ;
- relatif avec plusieurs composants refusé avant le spawn : [process.rs:330-341](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:330) ;
- `./tool` produit bien `CurDir + Normal`, donc deux composants et est refusé ;
- un absolu inexistant reste absolu raw, puis échoue en `Io`, classé retryable : [process.rs:217-220](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:217), [orchestrator.rs:2274-2280](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:2274). Cette borne est acceptable.

En revanche, pour un nom nu, la std Rust Windows 1.95 utilisée par ce checkout :

1. extrait bien le `PATH` enfant de la map de `Command` ;
2. appelle ensuite `resolve_exe(..., || env::var_os("PATH"), child_paths)` : [windows.rs.html:281-289](/C:/Users/FlowUP/.rustup/toolchains/stable-x86_64-pc-windows-msvc/share/doc/rust/html/src/std/sys/process/windows.rs.html:281) ;
3. recherche successivement le `PATH` enfant, le répertoire de l’exécutable parent, les répertoires système, puis le **`PATH` du processus parent** : [windows.rs.html:535-580](/C:/Users/FlowUP/.rustup/toolchains/stable-x86_64-pc-windows-msvc/share/doc/rust/html/src/std/sys/process/windows.rs.html:535).

Scénario discriminant :

- même nom nu `tool`, même cwd figé, mêmes arguments et même map env scopée ;
- le `PATH` scopé ne contient pas `tool.exe` ;
- incarnation A avec un `PATH` parent menant à `A\tool.exe` ;
- incarnation B avec un `PATH` parent menant à `B\tool.exe`.

Les deux incarnations ont exactement le même `dedup_scope`, mais exécutent deux programmes différents. `env_clear()` configure l’environnement enfant ; il n’empêche pas le resolver Windows de lire le `PATH` parent avant `CreateProcess`.

Les claims « lookup via le `PATH` scopé » et « aucune variable ambiante lue » sont donc plus forts que le comportement réel : [process.rs:31-40](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:31), [process.rs:209-220](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:209), [lib.rs:108-119](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/lib.rs:108), [THREAT_MODEL.md:1833](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1833).

Les tests ne couvrent que l’alias absolu et `bin/tool` : [effects.rs:1689-1717](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:1689), [effects.rs:1719-1763](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:1719).

Correction attendue : résoudre explicitement le nom nu contre le seul `PATH` injecté, figer le résultat absolu avant l’intent, puis refuser l’absence de résultat — ou refuser les noms nus sous Windows. Ajouter un test où le `PATH` enfant manque la commande tandis que deux `PATH` parents concurrents la fournissent.

### [P2] K-FENCE-BARE-LEAF-06 — `effects.log` reste raw

`freeze_side_output_path` canonise le fichier existant, puis son parent lorsqu’il est non vide : [activity.rs:385-405](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/activity.rs:385).

Pour un chemin relatif à un seul composant, `Path::parent()` retourne `Some("")` : [Path docs:114-117](/C:/Users/FlowUP/.rustup/toolchains/stable-x86_64-pc-windows-msvc/share/doc/rust/html/std/path/struct.Path.html:114). Par conséquent, lorsque `effects.log` n’existe pas :

- la canonicalisation entière échoue ;
- `!parent.as_os_str().is_empty()` est faux ;
- le helper retourne `effects.log` raw ;
- le scope, la lecture et l’append restent relatifs au cwd ambiant : [activity.rs:514-520](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/activity.rs:514), [activity.rs:543-548](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/activity.rs:543).

Le scénario exact du r5 — intent dans A, reprise dans B, même scope `effects.log`, deux journaux — subsiste.

Le test neuf utilise deux chemins absolus sous `tempdir`, dont `sub/../effects.log` : [effects.rs:1765-1785](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:1765). Il tue la suppression complète du helper, mais pas la branche `parent == ""`.

Je rétrograde ce résidu en P2 parce que le constructeur, `lib.rs` et THREAT bornent désormais honnêtement les entrées supportées et déclarent le fallback raw comme dépendance ambiante : [activity.rs:407-418](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/activity.rs:407), [lib.rs:113-119](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/lib.rs:113). Il reste néanmoins incompatible avec l’affirmation que le cas nominal du futur log est figé.

Correction : traiter le parent vide comme `.` avant canonicalisation, ou refuser explicitement tout fallback relatif.

## Traitement des quatre findings r5

| Finding r5 | Verdict R6 | Évidence |
|---|---|---|
| K-PROC-PROGRAM-RESOLVE-05 | **GAP / PARTIEL — P1** | Absolu figé et relatif avec séparateurs refusé, mais nom nu encore lié au `PATH` parent Windows. |
| K-FENCE-SCOPE-REL-05 | **PARTIEL — P2** | Helper appliqué aux deux constructeurs [activity.rs:414-430](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/activity.rs:414), mais `effects.log` reste raw. |
| K-RAW-PATH-PRECONDITION-05 | **CONFIRME** | Borne absolute/existing et fallbacks raw documentés dans [lib.rs:113-119](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/lib.rs:113) et [THREAT_MODEL.md:1833](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1833). La même row reste toutefois fausse sur le lookup du nom nu. |
| K-DOC-WORKFLOW-STATUS-05 | **CONFIRME** | Phase V et mécanique J/K livrée : [REFERENCE.md:94-106](/C:/Users/FlowUP/Documents/Code/nexus/docs/workflow/REFERENCE.md:94). J/K déplacées dans « Delivered » et futur limité à L/N/O/Hub : [llms.txt:11-16](/C:/Users/FlowUP/Documents/Code/nexus/docs/workflow/llms.txt:11). |

## Inventaire de classe

Aucun autre champ de chemin porté par un `Activity` ou `EffectTarget` durable de production n’a été trouvé.

| Porteur | Verdict |
|---|---|
| `ProcessActivity.program` | **GAP** — absolu fermé, relatif séparé refusé, nom nu Windows encore ambiant. |
| `ProcessActivity.cwd` | **CONFIRME** au périmètre existant supporté ; canonicalisé au constructeur et inclus dans le scope : [process.rs:222-228](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:222), [process.rs:263-267](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:263). |
| `WorktreeFsWriteActivity.worktree_root` | **CONFIRME** au périmètre existant supporté ; racine et état HEAD/index/worktree liés : [fs_effect.rs:230-260](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/fs_effect.rs:230), précondition avant mutation [fs_effect.rs:263-275](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/fs_effect.rs:263). |
| `FencedFileActivity.log_path` | **PARTIEL** — fermeture des chemins absolus/parents explicites, bare leaf raw. |
| `FileEchoTarget.log_path` | **CONFIRME hors classe de production** — preuve E gelée, exclusion explicitée dans [activity.rs:385-393](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/activity.rs:385). |
| Chemins du store | **CONFIRME hors `dedup_scope` Activity** — le fichier SQLite et son lease constituent directement l’autorité durable : [store.rs:670-692](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:670). |

Les chemins éventuellement transportés dans les arguments, l’environnement ou l’input ne constituent pas un champ caché supplémentaire : arguments/env sont intégrés au scope process et l’input FS est intégré au request hash puis confiné.

## Verdict par livrable Phase K

| Livrable | Verdict | Évidence |
|---|---|---|
| `effect_id` stable, tentatives distinctes, retries/backoff/timeouts | **CONFIRME** | Intent unique et dérivation moteur [orchestrator.rs:1249-1267](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1249) ; re-arm crash [orchestrator.rs:1273-1299](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1273) ; budget et timer [orchestrator.rs:1543-1564](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1543). |
| Receipt local structuré et réconciliation | **CONFIRME** selon DS-1, pas Ed25519 | Structure liée [activity.rs:201-243](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/activity.rs:201) ; récupération exige run/node/effect/request correspondants [orchestrator.rs:440-477](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:440) ; adaptation DS-1 [preflight:176-195](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_phase_k_preflight.md:176). |
| `outcome_unknown` jamais transformé sans preuve | **CONFIRME** | Ordre intent → park → reconcile → completion imposé par le reducer : [reducer.rs:377-403](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:377), [reducer.rs:445-499](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:445). |
| FS lié à HEAD/index/worktree et confiné | **CONFIRME** avec borne chemin existant | Scope/state [fs_effect.rs:230-260](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/fs_effect.rs:230), confinement et refus des symlinks/special files [fs_effect.rs:142-217](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/fs_effect.rs:142). |
| Processus lié à une commande stable | **GAP — P1** | Le resolver du nom nu Windows peut sortir du `PATH` scopé. |
| Nettoyage des descendants | **CONFIRME** dans les bornes documentées | Job Object Windows avec spawn suspendu [process.rs:94-114](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:94), process group Unix [process.rs:77-92](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:77), test heartbeat anti-vacuité [effects.rs:2462-2538](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:2462). |
| Trait `Activity`, drain cancel, orphan reap | **CONFIRME** | Contrat non bloquant [activity.rs:150-199](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/activity.rs:150), drain/reap et propagation d’erreurs [orchestrator.rs:1719-1803](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1719). |
| Fencing cible de référence | **PARTIEL — P2** | Mécanisme présent, mais l’identité du backend bare-leaf reste ambiante ; atomicité production déjà honnêtement bornée dans [THREAT_MODEL.md:1769](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1769). |
| Reroutage documentaire des routes mutantes | **CONFIRME** | [REFERENCE.md:94-106](/C:/Users/FlowUP/Documents/Code/nexus/docs/workflow/REFERENCE.md:94), [LOOPBACK_ENDPOINTS_TRUST_TIERS.md:103-105](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md:103). |

## Régression r1–r4

Aucune régression supplémentaire observée :

- garde `request_hash`, idempotence et `dedup_scope` sur reprise : [orchestrator.rs:1105-1169](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1105) ;
- receipt récupéré parsable et lié : [orchestrator.rs:440-477](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:440) ;
- re-arm du timer perdu : [effects.rs:506-591](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:506) ;
- drain cancel de l’orphan via handle journalisé : [effects.rs:1467-1524](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:1467) ;
- causalité completion/failure/park/reconcile : [effects.rs:1787-1845](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:1787), [effects.rs:2201-2250](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:2201) ;
- cwd process figé, env et état FS dans le scope : [effects.rs:1600-1717](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:1600).

## Invariants et validations

- **Goldens byte-identiques à `a23510e8` — CONFIRME** :
  - `run_event.golden.json` : `9ac8d44167bc472f93330b0c679b455c08ec4e72`;
  - `run_event_orchestrated.golden.json` : `875166f3cc273a0c450042e71c38b569c7824b37`;
  - `run_event_terminal.golden.json` : `1ba2be10a96ee3597158d2a7861e57d48e7c80b1`.
- **`RUN_EVENT_FORMAT_VERSION == 1` — CONFIRME** : [run_event.rs:57-65](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:57).
- **Upcasters vides — CONFIRME** : [run_event.rs:379-390](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:379).
- **Census 29 — CONFIRME** : `check-frontier-contracts.sh` exécuté directement, exit 0, `DOMAIN census [29 frozen]`; garde par liste exacte [check-frontier-contracts.sh:202-260](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:202).
- **Zéro nouveau nœud lock — CONFIRME** : `992` packages avant et après ; le diff ajoute seulement `libc` et `windows 0.57.0` aux dépendances du package engine existant.
- **Inventaire tests — CONFIRME** : `231` découverts, `7` ignored, donc **224 actifs** dans les binaires courants. Les trois nouveaux tests sont présents.
- **Directement exécutés ici** : `cargo fmt --all --check` vert, frontier vert, `git diff --check` exit 0.
- **Non convertis en preuve indépendante** : les 224 exécutions, clippy, les 12 mutations et les reruns complets Windows/Docker sont annoncés et cohérents avec les artefacts compilés, mais n’ont pas été rejoués dans ce sandbox lecture seule. `--list` prouve la découverte, pas le succès runtime.

## Claims restantes plus fortes que le code

Un seul groupe de claims bloquants subsiste : « nom nu résolu uniquement via le `PATH` scopé » et « aucune lecture d’environnement ambiant ». Ils doivent être corrigés en code ou la row `T-RUN-PROCESS-SPAWN-INJECTION` doit perdre son statut « fermée ».

Les bornes raw-relative, l’atomicité du backend fenced de référence, les receipts locaux non Ed25519, les limites Unix et le statut J/K/V sont désormais honnêtes.

**Décision Round 6 : GAP — scellement refusé. Un P1 reste ouvert (`K-PROC-BARE-PATH-AMBIENT-06`) ; le P2 `K-FENCE-BARE-LEAF-06` est documentable mais ne suffit pas à lever le blocage.**

Le sandbox étant en lecture seule, aucun fichier `sprint83_phase_k_codex_review_r6.md` n’a été écrit ; le présent bloc est le rapport brut scellé.

