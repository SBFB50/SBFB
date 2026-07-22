# Sprint 83 Phase H — Review round 2 (post-Codex r2)

**Date** : 2026-07-22. **Base** : working tree sur HEAD `24ff47e2`.
**Contexte** : le round 1 (fan-out 4 agents, 15 findings corrigés, PASS-PENDING)
a été suivi de la vérification croisée Codex GPT-5.6 Sol — r1 = sandbox local
mort (artefact conservé tel quel), r2 = audit sur BUNDLE INLINE 241 Ko
(pattern Phase F), verdict **GAP** avec findings substantiels. Ce round
documente leur réconciliation : chaque GAP retenu est corrigé dans l'arbre,
chaque borne est documentée, deux points sont réfutés avec mesure.

## Findings Codex r2 → résolutions

| Finding Codex | Décision | Résolution |
|---|---|---|
| Binding document↔digest absent (`ProvidedDocument.blake3` fourni par l'appelant, jamais vérifié) | **CORRIGÉ (refonte)** | `ProvidedDocument` porte désormais les **BYTES** ; le compilateur dérive `blake3(bytes)` LUI-MÊME (`bind_provided`), refuse des bytes hors-lock ou ≠ ref (`UnpinnedReference`). Harness de test refondu : `synth_input` sérialise et patch les refs aux pins RÉELS (leaf-first) ; mutations post-construction via `remint_root`/`remint_agent`/`remint_subworkflow` qui re-scellent la chaîne bytes→pin→lock→content, la mutation testée restant le SEUL invariant cassé. |
| Cache agents court-circuite le binding (2 refs même id, digests ≠ → 2e pin ignoré) | **CORRIGÉ** | Cache = `(digest, agent)` ; tout hit re-confronte `agent_ref.blake3` au digest caché → `UnpinnedReference « two digests »`. Test `static_validation_rejects_agent_under_two_digests` (cache chaud cross-scope). |
| `record_digest` écrase silencieusement les homonymes (toutes catégories hors schemas) | **CORRIGÉ** | `record_digest` → `Result`, un id = UN digest pour TOUTES les catégories ; `semantic_views` devient same-bytes-only (bénin, documenté). |
| `subworkflow_refs` déclarés non-développés jamais `check_pin` | **CORRIGÉ** | `check_pin` sur chaque ref déclarée au moment du record. |
| `validate_port_types` AVANT chargement agents → edges agent-adjacents exemptés silencieusement | **CORRIGÉ** | Agents chargés AVANT le pass edge-level (commentaire d'ordre) ; test `static_validation_rejects_agent_edge_port_type_mismatch` (input agent id distinct pour atteindre l'edge-level, l'intra-agent plan-wide étant sinon plus précoce — les deux refusent). |
| Handoff : seuls les givers DIRECTS contrôlés (un transform blanchit l'allowlist) | **CORRIGÉ** | `upstream_agent_invokers` = remontée transitive à travers les nodes non-agents (stoppe à l'agent = donneur responsable de son chemin). Test `static_validation_rejects_transitive_unauthorized_handoff` (contrôle vert + rouge). |
| Taint : `capability.call` blanchissait (relais d'arguments modèle) | **CORRIGÉ** | `capability.call` = récursion (ne blanchit pas) ; `knowledge.query` reste clean (résultat = corpus PINNÉ, jugement v1 documenté). Test `static_validation_rejects_capability_laundered_classification`. |
| `effect_depends_on` intra-scope (switch parent → enfant effectful échappe) | **CORRIGÉ (fail-closed)** | Un `map`/`repeat`/`subworkflow` AVAL compte comme effet (sur-refus assumé documenté). |
| Bras « classification » de l'invariant = decision_labels HITL (famille testée 2×) | **CORRIGÉ** | Nouveau test `plan_pins_switch_discriminants` : mutation du `discriminant_field` du switch quality-gate de BABEL → digest change. |
| Dimension coût sans bras rouge | **CORRIGÉ** | `static_validation_rejects_cost_budget_overflow`. |
| Claims §18/module doc (« pure du pack », « une variante chacune », « self-contained », « every collection sorted », « fermée » launder) | **CORRIGÉ (docs)** | THREAT_MODEL : purity reformulée (pure de l'entrée RÉSOLUE, chaque document lié par hash) ; T-PLAN-STATIC-BYPASS : #3 = parse-carried, #2 ⊆ #16, handoff transitif ; T-PLAN-CLASSIFICATION-LAUNDER : capability ne blanchit plus + effet cross-scope ; T-PLAN-DIGEST-MALLEABILITY : un id = un digest TOUTES catégories + cache re-confronté. Module doc : collections (triées OU ordre-du-document pinné), `CompiledNodeInstance` self-contained À RÉSOLUTION DE REFS PRÈS, `check_join_policy` borne SINGLE-LEVEL énoncée (liveness, pas autorité — J re-observe), doc `entrypoints`. |
| Joins imbriqués : faux pass quorum (réseau de `all` intermédiaires) | **DOCUMENTÉ (borne v1)** | L'analyse est single-level par chooseur ; les compositions de joins ne sont pas modélisées. Classe LIVENESS : un join insatisfiable bloque le run au runtime, n'accorde rien. Énoncé dans le doc de `check_join_policy` + reste une amélioration possible (Phase J re-observe les arrivées réelles). |
| « ANY pack change moves the digest » (bras ii) | **CORRIGÉ (doc)** | Le plan dérive de (révision, lock, nom, documents pinnés) — description/claims de digests n'y entrent pas ; leur intégrité est portée par la signature du pack, pas par le plan. |
| Identité publisher = nom+révision seulement | **DOCUMENTÉ** | La chaîne : signature Ed25519 (vérifiée AVANT compile) → `content` → `pack_digest` du plan + (nom, révision). La clé signataire est liée par `verify_signed_pack`, hors du plan — conforme §12.1 (le plan ne porte pas la provenance). |
| Provenance/évaluation « par forme seulement » (#13) | **DÉJÀ DOCUMENTÉ** | Champs REQUIS du parse F ; moitié vive = domination HITL/wait.signal (inchangé). |
| `graph_semantic_digest_order_independent` : bytes réordonnés = pack différent | **AJUSTÉ (plus honnête)** | Le test compare désormais : même digest sémantique, plans identiques MODULO les bindings dérivés des bytes (pack_digest + pin du root) — c'est exactement la claim WA-T1.2. |

**Réfuté / hors-retenue** : « les 3 autres compiled goldens non inspectables »
— couverts par `golden_compiled_plans_stable` (compare en Value, vert) ; la
structured-output (#8 moitié adapters) et le scan best-effort (#10/#11)
étaient déjà des déferrals documentés que Codex marque honnêtement PARTIEL.

## Suites après réconciliation

- `cargo nextest run -p sbfb-workflow-core --locked` : **103/103** (+6 tests
  Codex : handoff transitif, agent 2-digests, capability-launder, agent-edge
  port-type, cost overflow, babel discriminant).
- `contracts.schema.json` + goldens régénérés sous build WORKSPACE ;
  **les 4 `*.compiled.golden.json` sont BYTE-STABLES** (le plan émis n'a pas
  changé — les fixes durcissent les refus et le binding, pas la forme).
- fmt + clippy `-D warnings` clean ; census 29 intact ; 0 diff Cargo.
- Workspace Win + doctests : re-run en cours au moment de l'écriture de ce
  round (fixes = crate seul) ; Docker dual-platform re-joué avant commit.

## Verdict

**PASS-PENDING** — l'arbre corrigé part en Codex round 3 (bundle mis à jour)
pour confirmation ; promotion PASS à la réconciliation r3.
