# Review r4 — Sprint 83 Phase K — Effets, retries et idempotence

## Verdict: PASS-PENDING

Round r4 = état après la boucle Codex r4→r6 (3 rounds GAP successifs, tous
traités) + 3 passes de review Workflow (wf_be0ef371 r4-delta 3 agents,
wf_c47f7765 r5-delta 2 agents, wf_d6d72bd1 r6-delta 2 agents, Opus 4.8 1M
chacun) + fermeture de CLASSE de la famille « chemin ambiant dans une
identité durable ». Succède à la review r3 (PASS-PENDING, boucle Codex
r1→r3). PASS-PENDING = review OK, round Codex r7 de scellement en cours.

## La classe fermée (fil conducteur r3→r7)

Le sujet unique de toute la boucle post-r3 : un INGRÉDIENT qui influence
l'effet mais manque dans `dedup_scope` permet à un resume de « refaire le
même travail » en visant autre chose. Fermetures successives, chacune avec
test discriminant + mutation ACTIVE jouée :

- r3 : racine FS relative → canonicalisée (puis r4 : FIGÉE une fois au
  constructeur, plus de résolution live).
- r4 : cwd du process (hérité, ni fixé ni scopé) → champ REQUIS figé
  constructeur, dans le JCS, appliqué `Command::current_dir`.
- r5 : program (résolution relative plateforme-ambiguë) + log du
  FencedFileActivity (backend interchangeable sous même scope) → freeze.
- r6 : nom nu Windows (fait STD prouvé source : `resolve_exe` retombe sur
  le PATH AMBIANT du parent) → programme ABSOLU-ONLY, toute épellation
  non-absolue REFUSÉE à `start` ; feuille nue du log fenced
  (`parent() == Some("")` échappait au freeze) → substitution `.`
  (figée contre le cwd de CONSTRUCTION).

Inventaire de classe EXHAUSTIF tranché (vérifié indépendamment par la
review wf r6-delta ET par Codex r6) : program (refusé/figé), cwd (figé),
worktree_root (figé), FencedFileActivity.log_path (figé),
FileEchoTarget (echo.rs BYTE-INTACT vs HEAD — preuve GELÉE Phase E
clause-2, délibérément non rétrofittée, documenté), chemins store (hors
classe : le fichier SQLite + lease EST l'autorité durable, pas un token
de scope). Args/env/input ne sont pas des champs cachés (scopés ou
confinés).

## Traitement des findings Codex r4 (GAP 1 P1 + 2 P2 + 1 P3)

- **K-PROC-CWD-SCOPE-03 (P1)** : FIX CODE — cwd requis, figé
  (canonicalize-une-fois, fallback raw documenté), clé "cwd" du JCS,
  `current_dir` au spawn. Tests : `process_cwd_is_part_of_dedup_scope`
  (mutation drop-"cwd" tuée), `process_effect_runs_in_fixed_cwd` (enfant
  réel marqueur relatif ; mutation drop-current_dir tuée),
  `process_cwd_canonicalised_in_dedup_scope` (miroir aliasing, ajouté sur
  finding review wf r5-delta K-PROC-CWD-FREEZE-NOTEST — la mutation du
  freeze SURVIVAIT sans lui ; tuée depuis).
- **K-FS-ROOT-FREEZE-02 (P2)** : FIX CODE — racine figée UNE fois au
  constructeur ; `dedup_scope` sans résolution live ; `confine`
  re-canonicalise la forme figée (identité au nominal, commentaire exact).
- **K-SCOPE-PATH-ENC-03 (P2)** : FIX CODE — scope FS en JCS
  {root, expected_head, expected_index_digest, expected_worktree_digest}
  (fini le join 0x1f) ; claim « control-free by validation » RETIRÉE ;
  borne `display()`-lossy non-UTF-8 documentée aux 2 scopes + THREAT.
- **K-FS-ROOT-FALLBACK-DOC-04 (P3)** : FIX DOC — dichotomie du fallback
  raw remplacée par les bornes exactes (y compris le 3e chemin
  racine-absente-aux-deux-incarnations, finding review wf r5-delta).

## Traitement des findings Codex r5 (GAP 2 P1 + 1 P2 + 1 P3)

- **K-PROC-PROGRAM-RESOLVE-05 (P1)** : FIX CODE — absolu figé canonique
  (`process_program_canonicalised_in_dedup_scope`, mutation tuée) ;
  relatif-avec-séparateurs refusé à start
  (`process_relative_program_refused`, mutation drop-gate tuée) ;
  complété r6 par le refus du nom nu (voir plus bas).
- **K-FENCE-SCOPE-REL-05 (P1)** : FIX CODE — `freeze_side_output_path`
  (canonique entier → parent-canonique+feuille → raw dernier recours)
  appliqué aux 2 constructeurs (`fenced_log_path_frozen_in_dedup_scope`,
  mutation tuée).
- **K-RAW-PATH-PRECONDITION-05 (P2)** : FIX DOC — claims bornées au
  périmètre absolu/existant (lib.rs charte + THREAT), fallbacks raw =
  dépendances ambiantes DOCUMENTÉES, pas des fermetures.
- **K-DOC-WORKFLOW-STATUS-05 (P3)** : FIX DOC — REFERENCE.md « re-routed
  to Phase V » (mécanique J/K livrée) + llms.txt J/K passées Delivered.

## Traitement des findings Codex r6 (GAP 1 P1 + 1 P2)

- **K-PROC-BARE-PATH-AMBIENT-06 (P1)** : FIX CODE par RÉDUCTION DE
  SURFACE — fait std prouvé par Codex dans la SOURCE Windows 1.95
  (`resolve_exe` : PATH enfant → répertoire exe parent → répertoires
  système → PATH AMBIANT du parent) ⇒ un nom nu ne peut PAS être lié au
  seul PATH scopé sans réimplémenter le lookup (hors charte). Programme
  ABSOLU-ONLY : gate `!is_absolute()` à start (couvre relatif, nom nu,
  et rooted-sans-drive Windows). Test `process_bare_program_refused` ;
  mutation « gate rétréci à components>1 » tuée. Docs alignées (field
  doc, module-doc, lib.rs, THREAT row avec citation resolve_exe).
- **K-FENCE-BARE-LEAF-06 (P2)** : FIX CODE — `Path::parent()` d'une
  feuille nue == Some("") esquivait le tier-2 → substitution `.` avant
  canonicalisation (feuille nue figée contre le cwd de CONSTRUCTION,
  même contrat qu'un cwd/root). Test
  `fenced_bare_leaf_frozen_in_dedup_scope` ; mutation
  drop-substitution tuée. Résidu raw restant = parent JAMAIS résolu
  (documenté lib.rs + THREAT).

## Findings des reviews Workflow (tous traités)

- wf r4-delta : 1 P3 (invariant cwd-immuable du binaire de test non
  documenté) → doc au test ; claims-vs-arbre 0 finding (ledger complet
  des 5 P1 r1 + fixes r2 vérifiés présents) ; staging-inventory 0
  finding (15 M + 4 crate untracked + 7 planning ; plan_po.md EXCLU vers
  chore séparé ; Cargo.lock +2 lignes target-gated 0 nœud).
- wf r5-delta : 1 P2 réel K-PROC-CWD-FREEZE-NOTEST (mutation du freeze
  process survivante → test miroir ajouté, mutation tuée) + 2 P3 doc
  (bornes raw process symétriques ; dichotomie fs_effect non exhaustive)
  → fermés.
- wf r6-delta : classe jugée FERMÉE 0 P0/P1/P2 ; echo.rs byte-intact
  VÉRIFIÉ ; 2 P3 → fermés (commentaire dupliqué verbatim dans la porte
  [artefact de restauration de mutation] ; énumération des fallbacks
  raw complétée lib.rs/THREAT + doc FencedFileActivity conditionnée).

## Vérification §7.4 (arbre FINAL post-r6)

- Windows : fmt 0 ; clippy workspace -D warnings OK ; nextest
  **2363/2363 + 7 skip, 0 ROUGE** ; doctests OK ; release OK ; rustdoc
  engine OK.
- Docker sbfb-ci : fmt OK ; nextest **2369/2369 + 7 skip, 0 ROUGE**.
- Crates engine+core : **226 run + 7 skip** (217 r3 + 9 tests de la
  boucle r4→r7). **14 mutations actives tuées** cumulées.
- Runs intermédiaires : 2 flakes des classes STANDING carry 7 (sigint,
  dispatch_loop), chacun re-run solo PASS ; web 548 + operator 201
  verts (blocs complets) ; frontier clean ; census 29 ; goldens E/J
  byte-identiques (hashes git confirmés Codex r6) ; 0 nœud lock neuf
  (992 avant/après).

## Codex reconciliation (7 rounds, canon §4.5.8)

r1 GAP 5 P1 → 2 fixes code + 3 rétrogradations honnêtes + 4 P2 code.
r2 GAP 2 P1 → receipt lié + state-binding + 4 P2 + 5 rétrogradations.
r3 GAP 1 P1 → canonicalisation racine. r4 GAP 1 P1 + 2 P2 + 1 P3 → cwd
+ freeze + JCS + doc. r5 GAP 2 P1 + 1 P2 + 1 P3 → program + fenced log
+ claims bornées + docs V. r6 GAP 1 P1 + 1 P2 → ABSOLU-only + feuille
nue. r7 = round de scellement EN COURS sur l'arbre final (verdict au
canonique). Chaque round a écrit son propre fichier
`rounds/sprint83_phase_k_codex_review_r{R}.md`, jamais écrasé.

## RESTE À FAIRE avant commit

1. Verdict Codex r7 (attendu CLEAN ou P2/P3 documentés — la surface
   program est réduite à UN cas, l'inventaire de classe est tranché).
2. Promotion des canoniques : ce fichier → `sprint83_phase_k_review.md`
   avec `## Verdict: PASS` + section réconciliation r7 ;
   `sprint83_phase_k_codex_review.md` = copie byte-identique du r7.
3. Commit atomique `feat(workflow-engine): Sprint 83 Phase K` (body 9
   sections prêt, staging inventorié par wf r4-delta).
4. Post-commit : chore(planning) pour `sprint83_plan_po.md` (vue PO
   hors-phase, garanties absolues reformulées honnêtement) + MAJ
   memory `nexus_grid_pivot.md` + `MEMORY.md`.
