# Sprint 83 Phase H — Review round 1

**Date** : 2026-07-22. **Base** : working tree sur HEAD `24ff47e2` (diff non committé).
**Méthode** : fan-out 4 agents adversariaux parallèles (fallback multi-agents §7.1,
opt-in Workflow inactif) — dimension A conformité-preflight, B tests/anti-vacuité,
C sécurité/honnêteté-des-claims, D research-grounding/scope — chacun avec obligation
de mesure (fichier:ligne, re-run des suites, refus de toute claim non vérifiée),
synthèse main-thread. Autorité de la phase : `sprint83_phase_h_preflight.md`
(PLAN-ADAPT).

## Verdicts par dimension (avant fixes)

| Dim | Agent | Verdict | Findings |
|---|---|---|---|
| A conformité preflight | review-h-conformite | **PASS** | 2 P3 (A-1 claim nominal, A-2 renumérotation ratifiée) |
| B tests / anti-vacuité | review-h-tests | **CONCERN** | 1 P1 (B-1 taint non testé) + 3 P2 (B-2 bornes égalité, B-3 garde hex masquée, B-4 règle plan-wide masquée) + 2 P3 (B-5, B-6) |
| C sécurité / claims | review-h-securite | **CONCERN** | 2 P2 (C-1 « fermée » sur listes nominales, C-2 = B-1) + 2 P3 (C-3 COMPILER_VERSION, C-4 scan `://`-only) |
| D grounding / scope | review-h-grounding | **PASS** | 1 P2 (D-1 marqueurs publication sous-inclusifs) + 4 P3 (D-2..D-5) |

Convergence indépendante notable : A-1, C-1 et D-1 pointent le MÊME pattern
(sur-affirmation « fermée » sur des reconnaissances par catalogue nominal fixe,
asymétrique avec le scan voisin honnêtement « L accepté ») ; B-1 et C-2 pointent
le MÊME manque (branches de `field_taints_from_model` sans contrôle rouge).

## Findings et résolutions (tous traités in-phase sauf mention)

| ID | Sév | Résolution |
|---|---|---|
| B-1 / C-2 | P1/P2 | **CORRIGÉ** — test `static_validation_taint_traversal_branches` : 4 bras rouges (transform-rename ne blanchit pas ; champ NON mappé = provenance inconnue fail-closed [tue la mutation `None => false` citée par B-1] ; sortie map opaque = taint ; join clean+model = taint) + contrôle vert (chaîne rename alimentée par input compile). 97/97. |
| C-1 / A-1 / D-1 | P2/P3/P2 | **CORRIGÉ** — docs des 3 constantes (`CLASSIFICATION_DISCRIMINANTS`, `CLASSIFICATION_LABEL_TERMS`, `PUBLICATION_CAPABILITY_MARKERS`) : borne NOMINALE v1 explicitée (synonyme hors liste échappe, même classe résiduelle que le scan ; Phase L §14.2 rend précis) ; THREAT_MODEL §18 : T-PLAN-CLASSIFICATION-LAUNDER requalifié « fermée (nominal v1) » avec caveat synonymes + sous-inclusion des marqueurs publication (`content.share` non capturé avant L). |
| B-2 | P2 | **CORRIGÉ** — bornes inclusives testées des deux côtés : `deep_input(MAX_PLAN_DEPTH)` compile / `+1` refusé ; `budget_equality_is_within_bounds` (node budget == plafond compile — tue la mutation `>`→`>=`). |
| B-3 | P2 | **CORRIGÉ** — la garde hex REF-level est réellement exercée : mutation post-construction du `input_schema_ref` workflow-level (hors-edge, hors-lock) — le lock reste propre (assert de contrôle) et ni le lock-level ni le port-type pass n'interceptent. |
| B-4 | P2 | **CORRIGÉ** — la règle plan-wide un-digest-par-schema_id (`check_schema_ref`) exercée OFF-edge (workflow output_schema_ref divergent du input de même id). |
| C-3 | P3 | **CORRIGÉ** — couplage documenté (doc `COMPILER_VERSION` + THREAT_MODEL : « pure à version de compilateur fixe » ; bump workspace ⇒ refresh goldens). |
| C-4 | P3 | **CORRIGÉ** (voie doc) — borne `://`-only énoncée dans le doc du scan + THREAT_MODEL T-PLAN-SCAN-EVASION (schémas sans slash non reconnus, même résiduel L ; la garantie = le FORMAT). |
| B-5 | P3 | **CORRIGÉ** — commentaire de `compile_is_deterministic` reformulé (le test prouve l'intra-profil ; l'inter-profil = mécanique + goldens sous 2 profils). |
| B-6 | P3 | **CORRIGÉ** — commentaire de `plan_then_execute_invariant` précisé (le discriminant = gardes d'existence + garde no-op, pas le flip de digest trivial). |
| D-3a | P3 | **CORRIGÉ** — doc `effect_depends_on` : élargissement délibéré à TOUT `capability.call` (plus strict que la lettre §8.6, fail-closed jusqu'aux classes typées L). |
| D-5 | P3 | **CORRIGÉ** — module doc : « miroir TS/canvas = Phase T » ajouté aux bornes. |
| D-2 | P3 | **BODY** — scope cut à dater au commit : compat de ports nominale = même-id-digests-différents refusé ; edges à schema_id hétérogènes NON jugés v1 (pas de sous-typage structurel). |
| D-3b | P3 | **BODY** — `join.source_node_ids` restent des ids-template dans les instances compilées (les `CompiledEdge` sont ré-encodés) ; résolution per-scope = Phase J, choix noté. |
| D-4 | P3 | **AUCUNE ACTION** — binding provider par préfixe : nominal v1 documenté + déferral N/O tracé (cité pour complétude). |
| A-2 | P3 | **AUCUNE ACTION** — insertion §18 + renumérotation §19 conforme à la décision B5 telle qu'exécutée, 0 cross-ref cassée, pattern historique du doc respecté. |

## Suites re-vertes après fixes

- `cargo nextest run -p sbfb-workflow-core --locked` : **97/97** (67 G + 30 H ; +2 vs r0 : taint traversal + budget equality).
- `cargo clippy -p sbfb-workflow-core --all-targets -- -D warnings` : clean ; `cargo fmt --check` : clean.
- Blocs déjà verts sur l'arbre pré-fixes (à re-jouer avant commit — fixes = tests + docs uniquement, 0 code prod modifié depuis) : Win nextest workspace 2240+1skip ; **Docker sbfb-ci 2244/2244+1skip no-fail-fast** (flake `sigint` env carry 7 : rouge au premier run fail-fast, vert solo + vert au run complet) ; doctests ; release build ; SPDX ; web complet + operator 201 ; `check-frontier-contracts.sh` exit 0 census 29 ; `git diff Cargo.toml Cargo.lock` vide ; 0 diff web/tools/engine.

## Points positifs confirmés par les 4 dimensions

20/20 classes de refus implémentées en code réel + testées variant-précis avec
contrôles verts ; interdits §11.5 tous respectés à la mesure (graph.rs prod
byte-identique, refactor tests imports-only strict, delta +28 puis +30 exact) ;
9 familles §12.1 toutes présentes annotées ; frontières byte-propres
(engine/web/operator intacts) ; DAG « par compilation » corrige réellement la
sur-affirmation de G ; unicité schéma robuste à l'ordre ; fermeture
PresentationKeyAsData complète ; gate `wait.signal` exercé par babel ;
pre-launch propre (0 legacy, deny_unknown_fields, 0 serde-default) ; census 29
et registre len==7 intacts.

## Verdict

**PASS-PENDING** (review OK après corrections in-phase ; Codex pas encore passé —
jamais committable en l'état). Prochaine étape : vérification croisée Codex
GPT-5.6 Sol sur l'arbre CORRIGÉ, puis reconciliation et promotion.
