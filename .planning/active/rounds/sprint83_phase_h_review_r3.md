# Sprint 83 Phase H — Review (canonique, PASS)

## Verdict: PASS

Compilateur Rust autoritaire `CompiledPlan.v1` (`crates/sbfb-workflow-core/src/compile.rs`).
Verdict promu après réconciliation Codex complète. Détail des rounds :
`rounds/sprint83_phase_h_review_r1.md` (fan-out 4 dimensions), `_r2.md`
(réconciliation Codex r2), et les 4 rounds Codex bruts
`rounds/sprint83_phase_h_codex_review_r{1..4}.md`.

## Déroulé de la boucle

1. **Review interne r1** (fan-out 4 agents adversariaux : conformité preflight,
   tests/anti-vacuité, sécurité/claims, grounding/scope) → 15 findings, 1 P1
   (taint non testé) + convergence 3-voix sur la sur-affirmation « fermée ».
   Tous corrigés in-phase. PASS-PENDING.
2. **Codex r1** = sandbox local mort (artefact conservé, trace honnête).
3. **Codex r2** (bundle inline 241 Ko) = **GAP**, 12 findings substantiels dont
   le binding document↔digest jamais vérifié. Refonte : `ProvidedDocument`
   porte les BYTES, le compilateur dérive `blake3` lui-même. + cache agents
   re-confronté, `record_digest` fail-closed toutes catégories, handoff
   transitif, taint capability, ordre des passes corrigé. +6 tests.
4. **Codex r3** = **GAP**, 5 demandes résiduelles : `knowledge.query` taint,
   contrat `agent.as_tool` non-donneur statué, bras rouge effet-opaque,
   sur-claims « any pack change » retirés, autorité de la racine documentée.
   Tous traités, +3 tests.
5. **Codex r4** (confirmation ciblée) = **3 CONFIRME-FERME + 2 PARTIELS**
   (tests à rendre plus discriminants, code correct) : test orchestrateur
   `agent.invoke → agent.as_tool → handoff` (vert/rouge selon allowlist parent)
   et isolation du bras subworkflow (seul effet aval). Renforcés → tous
   discriminants.

## Codex reconciliation

Rapports Codex bruts lus (`rounds/..._codex_review_r{1..4}.md`), jamais
réécrits. Aucune sur-affirmation résiduelle : purity reformulée (pure de
l'entrée RÉSOLUE, chaque document lié par hash) ; « 20 classes » nuancée
(#3 parse-carried, #2 ⊆ #16) ; « self-contained à résolution de refs près » ;
joins single-level énoncés comme borne de LIVENESS (Phase J re-observe) ;
`knowledge.query`/`capability.call` ne blanchissent pas le taint ; autorité de
la racine = hôte lanceur (auditable via `root_workflow_id` + digest), pas une
désignation cryptographique absente du format gelé F. Bornes v1 assumées et
documentées : compat de ports nominale (ids hétérogènes non jugés),
publication/classification reconnues par catalogue nominal (Phase L §14.2
rendra précis), scan endpoint `://`-only. GAPs P0/P1 : 0 restant. PARTIELS
Codex = déferrals documentés (structured-output → adapters N/O, scan
best-effort, joins → J).

## Suites (dernier état vert)

- `cargo nextest run -p sbfb-workflow-core --locked` : **106/106** (+36 vs
  Phase G ; contrôle-vert + mutation-rouge par classe, 20 refus + branches taint
  + bornes inclusives + invariant 9-familles + digest malléabilité).
- fmt + clippy `-D warnings` clean ; `check-frontier-contracts.sh` exit 0
  [29 frozen] ; `git diff Cargo.toml Cargo.lock` vide ; 0 diff
  web/tools/factory-operator/sbfb-workflow-engine ; graph.rs prod byte-identique.
- 4 `*.compiled.golden.json` BYTE-STABLES à travers toute la boucle (le
  durcissement change les refus et le binding, jamais la forme du plan émis).
- Dual-platform : Win nextest workspace + Docker `sbfb-ci` no-fail-fast, verts
  (re-joués après chaque round de fixes prod).
