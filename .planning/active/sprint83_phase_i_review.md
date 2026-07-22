# Sprint 83 Phase I — Review (canonique, PASS)

## Verdict: PASS

Run store durable SQLite (`crates/sbfb-workflow-engine/src/store.rs` +
`crates/sbfb-workflow-core/src/run_event.rs`). Verdict promu après
réconciliation Codex complète (5 rounds, arrêt au critère canonique
« 0 P0/P1, P2/P3 fermés ou documentés »). Détail des rounds :
`rounds/sprint83_phase_i_review_r{1..5}.md` + les 5 rounds Codex bruts
`rounds/sprint83_phase_i_codex_review_r{1..5}.md` (canonique = copie
byte-identique de r5).

## Déroulé de la boucle

1. **Preflight Workflow PLAN-ADAPT** (5 scans + adversarial ; 2 agents
   morts au cap StructuredOutput + scan S1a placeholder → rejoués en
   fan-out Agent, §9bis) : 23 adaptations, décisions B1-B8 (B6 re-coupe
   I → I + I1 ; B8 traits store → J).
2. **Review interne r1** (6 dimensions + adversarial, mode dégradé
   documenté : 4 dims du Workflow interrompu + 3 rejouées avant-plan) →
   0 P0/P1 après correctifs, dont un trou de monotonie du lease trouvé
   en instruisant un gap de couverture (release/re-acquire zombie).
3. **Codex r1 GAP** (3 P1) → fenêtre effet-externe RESCOPÉE couche STORE
   (carry Phase K tracé), relecture in-txn du GC (interleaving testé),
   validation backup inventaire. **r2 GAP** (2 P1) → états clos
   ABSORBANTS (`InvalidTransition`, sémantique divergent-sur-completed
   tracée, golden terminal régénéré), colonnes backup. **r3 GAP** (1 P1)
   → sémantique index/PK/FK. **r4 GAP** (1 P1) → index non-partiel, PK
   ensemble exact, FK ciblée. **r5 PARTIEL 0 P0/P1** → FK mono-colonne
   exacte + docs, arrêt de boucle.
4. **11/11 mutations actives tuées** (M1 CAS lease, M2 downgrade, M3
   digest snapshot, M4 garde GC pending, M5 user_version, M6 inbox
   dedup, M7 digest tuple, M8 relecture GC, M9 transitions absorbantes,
   M10 colonnes backup, M11 unicité index).

## Codex reconciliation

Rapports Codex bruts lus (`rounds/..._codex_review_r{1..5}.md`), jamais
réécrits. Chaque P1 fermé par code + test + mutation quand mutable ;
résiduels finaux P3 documentés : forgeage FK-composite sans bras de test
dédié (fix = recette Codex r5 ligne à ligne, borne dans §19) ; bornes de
validation restore ÉNUMÉRÉES (types, NOT NULL, CHECK, défauts,
collations, violations FK préexistantes, colonnes surnuméraires — la
frontière reste OS mono-utilisateur, THREAT_MODEL §19 T-RUN-RESTORE).

## Suites finales

Win : fmt --check OK ; clippy workspace -D warnings OK ; nextest
workspace **2273 passed + 2 skipped** (2251 fin H → +22 : +5 core
run_event, +17 tests/run_store.rs ; skip 1→2 = les 2 childs `#[ignore]`
out-of-process spawnés par leurs parents) ; doctests OK ; release OK ;
check-frontier-contracts clean (census DOMAIN 29 frozen) ; SPDX 394.
Docker canonique `sbfb-ci` 1.94 : fmt --check OK ; crates workflow
146/146 + 2 skipped. Workspace Docker : 2272/2273 + 1 flake env
solo-PASS (`start_writes_running_json_and_responds_to_health`, classe
Docker-on-Windows documentée, hors périmètre phase). Web 548 + coverage
+ build + size + operator 201 : verts (T1 = N-A-no-frontend-change).
