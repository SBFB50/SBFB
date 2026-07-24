Verdict global : **GAP — P1 process**, malgré un verdict technique **CONFIRMÉ — CLEAN** sur le delta post-r7. Aucun P0 ni P1 technique ne subsiste : le correctif Windows ferme la bifurcation `path.exe`/`path`, les attaques suffixe/alias/ADS/8.3 ne rouvrent pas la classe, et aucune régression r1–r6 n’est observée. En revanche, Round 8 a été lancé sans nouvelle review Workflow `PASS-PENDING` sur l’arbre corrigé. Le critère de scellement global n’est donc pas atteint.

Assiette finale du 24 juillet 2026 à 11:49:37 CEST : HEAD `a23510e828fab9d7e96370d0811c7d3e39ed9316`, working tree sale. Aucun processus `cargo`, `rustc`, `nextest` ou `bash` encore actif. Les blobs critiques sont restés stables pendant la vérification, notamment `process.rs = aa3ffb1e…`, `effects.rs = 11ebdfcb…`, `THREAT_MODEL.md = 278ede3a…`, `lib.rs = 40721ea3…`.

## Finding bloquant

### [P1] K-ROUND-REVIEW-AFTER-FIX-08 — review Workflow post-correctif absente

Après un GAP P0/P1, le canon impose explicitement : correction, suites, **nouvelle review Workflow**, puis nouveau round Codex sur cet arbre corrigé : [README.md:2499](/C:/Users/FlowUP/Documents/Code/nexus/docs/claude/README.md:2499). Un process gate manquant est classé P1 : [AGENT_SYSTEM.md:176](/C:/Users/FlowUP/Documents/Code/nexus/docs/agent/AGENT_SYSTEM.md:176).

Or :

- le dernier artefact Workflow reste `sprint83_phase_k_review_r4.md`, daté de 11:13, avant r7 et son correctif ;
- il se présente encore comme l’arbre « FINAL post-r6 », avec **226 actifs et 14 mutations** : [review_r4.md:113](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_k_review_r4.md:113) ;
- il dit toujours que r7 est en cours et attendu CLEAN : [review_r4.md:127](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_k_review_r4.md:127) ;
- aucun `sprint83_phase_k_review_r5.md` n’existe.

**Impact : GAP.** Une review post-correctif produite après ce rapport ne peut pas rétroactivement précéder Round 8. Ce rapport r8 doit rester brut ; il faut produire `sprint83_phase_k_review_r5.md`, puis lancer **Codex Round 9**.

## Traitement du P1 r7

**CONFIRMÉ — fermé exactement selon la correction proposée.**

- Le refus non-absolu précède toute autre interprétation : [process.rs:348](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:348).
- Le gate Windows est ensuite isolé par `#[cfg(windows)]` et utilise `eq_ignore_ascii_case("exe")` : [process.rs:362](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:362).
- Le message de refus correspond au contrat demandé : [process.rs:368](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:368).
- `Command::new` n’est atteint qu’après ces deux gates : [process.rs:373](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:373).
- `ActivityError::Refused` est contractuellement non-retryable : [activity.rs:138](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/activity.rs:138), [orchestrator.rs:2274](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:2274).
- Le test Windows vérifie `RunState::Failed` puis exactement un `EffectAttemptFailed { retryable: false }` : [effects.rs:1830](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:1830), [effects.rs:1856](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:1856), [effects.rs:1863](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:1863).
- Charte et menace sont alignées : [lib.rs:106](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/lib.rs:106), [THREAT_MODEL.md:1833](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1833).

La mutation active neutralisant le gate est annoncée tuée et son mécanisme est discriminant. **PARTIEL côté preuve indépendante** : les 15 mutations ne sont pas encore consignées dans une review post-correctif durable.

## Attaques Windows

- **`.EXE` / `.Exe` — CONFIRMÉ.** Les deux passent par `eq_ignore_ascii_case`. Le resolver Rust 1.95 utilise lui aussi un test ASCII-insensible et, dès qu’il voit `.exe`, ne tente plus le sibling suffixé : [windows.rs.html:473](/C:/Users/FlowUP/.rustup/toolchains/stable-x86_64-pc-windows-msvc/share/doc/rust/html/src/std/sys/process/windows.rs.html:473). Il n’existe pas de test mixed-case dédié, mais une régression vers un test sensible à la casse ne rouvrirait pas la bifurcation : elle sur-restreindrait seulement les entrées valides.

- **`tool.exe.` — CONFIRMÉ.** Sur une forme raw non résolue, `extension()` voit l’extension vide et le gate refuse. Si la forme Win32 résout déjà vers `tool.exe`, la canonicalisation constructeur intervient avant scope et spawn : [process.rs:278](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:278). Scope et exécution utilisent alors le même chemin final, sans choix entre deux fichiers. Rust documente que `canonicalize` obtient sous Windows le chemin final du handle : [documentation `canonicalize`](https://doc.rust-lang.org/std/fs/fn.canonicalize.html).

- **`tool.exe ` — CONFIRMÉ.** La forme raw produit l’extension `exe ` et est refusée. Une éventuelle normalisation d’un alias résolu se fait avant le calcul du scope et ne crée pas de second candidat. Les règles Win32 sur points et espaces terminaux sont documentées par [Microsoft](https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file).

- **ADS `tool.exe:stream` — CONFIRMÉ.** `extension()` voit `exe:stream`, donc refus. Une forme telle que `tool.exe:stream.exe` peut lexicalement passer, mais Rust n’ajoute alors aucun suffixe : le chemin explicite est transmis. Cela peut échouer comme image, mais ne crée pas la bifurcation `path.exe`/`path`. La syntaxe ADS est celle documentée par [Microsoft](https://learn.microsoft.com/en-us/windows/win32/fileio/file-streams).

- **8.3 short names — CONFIRMÉ, aliasing bénin dans la frontière déclarée.** Un chemin 8.3 résolu au constructeur est ouvert puis ramené à son chemin final. Un raw absolu encore absent peut plus tard résoudre via l’alias OS, mais le short name est un autre nom de la même entrée, pas un sibling sélectionnable simultanément. Une réaffectation après suppression/création est la même classe de remplacement local qu’un exécutable remplacé sous n’importe quel chemin accepté. Microsoft décrit les short names comme des alias alternatifs du nom long : [Naming Files, Paths, and Namespaces](https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file).

- **Réinterprétation résiduelle std après `.exe` — aucune trouvée.** Dans Rust 1.95, la branche suffixée contourne `program_exists` et l’ajout `.exe`, puis transmet le chemin à `CreateProcessW`. Avec `lpApplicationName` non nul et un chemin complet, Windows utilise ce module explicite et exige son extension : [CreateProcessW](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw). Les normalisations de namespace ou d’alias restantes ne constituent pas une sélection entre deux siblings.

## Régression r1–r6

**CONFIRMÉ — aucune régression observée.**

- Receipt récupéré parsable et lié à `run_id/node/effect/request` : [orchestrator.rs:440](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:440).
- Completion impossible sans intent correspondant ni reconciliation après parking : [reducer.rs:382](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:382), [reducer.rs:475](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/reducer.rs:475).
- Retry timer perdu réarmé après crash : [orchestrator.rs:1273](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1273).
- Drain cancel et reap par handle journalisé conservés : [orchestrator.rs:1705](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1705), [orchestrator.rs:1769](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:1769).
- État HEAD/index/worktree dans le scope et précondition avant mutation : [fs_effect.rs:230](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/fs_effect.rs:230), [fs_effect.rs:263](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/fs_effect.rs:263).
- Fenced log figé au constructeur, y compris feuille nue via `.` : [activity.rs:385](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/activity.rs:385), [effects.rs:1884](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/effects.rs:1884).
- Cwd, programme, arguments, environnement et politique d’échec restent tous dans le scope : [process.rs:311](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/process.rs:311).

## Invariants et validations

- **CONFIRMÉ — goldens byte-identiques à `a23510e8`** :
  - `run_event.golden.json` : `9ac8d44167bc472f93330b0c679b455c08ec4e72` ;
  - `run_event_orchestrated.golden.json` : `875166f3cc273a0c450042e71c38b569c7824b37` ;
  - `run_event_terminal.golden.json` : `1ba2be10a96ee3597158d2a7861e57d48e7c80b1`.
- **CONFIRMÉ — `RUN_EVENT_FORMAT_VERSION == 1`** : [run_event.rs:65](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:65). Upcasters toujours vides : [run_event.rs:379](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:379).
- **CONFIRMÉ — census 29** : liste vivante et liste gelée égales, `29/29`, delta nul ; contrat : [check-frontier-contracts.sh:202](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:202).
- **CONFIRMÉ — zéro nœud lock neuf** : `992` packages avant/après ; seules les dépendances déjà résolues `libc` et `windows 0.57.0` sont ajoutées à l’entrée engine.
- **CONFIRMÉ — inventaire compilé post-fix** : `234` tests découverts, `7` ignorés, donc `227` actifs ; le nouveau test Windows est présent.
- **CONFIRMÉ directement** : `cargo fmt --all --check` et `git diff --check a23510e8` sortent à zéro.
- **PARTIEL indépendamment** : les runs complets Windows/Docker, Clippy et les 15 mutations sont rapportés verts, mais la seule preuve Workflow durable reste r4 avec les anciens compteurs `226/7` et `14 mutations`.

## Claims

**CONFIRMÉ.** Les claims code, charte et THREAT ne sont plus plus fortes que l’implémentation. « Doit finir par `.exe` » s’applique au chemin stocké après canonicalisation ; les alias résolus point/space/8.3 ne contredisent donc pas ce contrat. Les chemins raw non résolus restent explicitement bornés par la frontière OS mono-utilisateur.

La seule claim obsolète est processuelle : `review_r4` qualifie encore l’arbre post-r6 de final et annonce r7 en cours. Elle est désormais plus faible et plus ancienne que le code vivant.

Comptage final : **P0 = 0 ; P1 technique = 0 ; P1 process = 1 ; P2 = 0 ; P3 = 0.**

Décision Round 8 : **GAP, scellement refusé**. Action minimale : nouvelle review Workflow immuable `sprint83_phase_k_review_r5.md` sur l’arbre corrigé et les preuves `227 + 7 / 15 mutations`, puis Codex Round 9. Ne jamais réécrire ce Round 8.

