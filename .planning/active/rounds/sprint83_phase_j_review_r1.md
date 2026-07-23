# Sprint 83 Phase J — Review round r1 (Workflow ultracode)

Date : 2026-07-23. Arbre pré-commit (aucun commit). Méthode : Workflow
`wf_d68b32c1-36b` — 7 dimensions (diff-intégral, tests-sémantique,
preflight-compliance, sécurité-deep, scope-cuts, docs-contrat, wire-census)
+ 2 lentilles adversariales (exactitude, complétude-critique). 7/9 agents
Workflow Opus 4.8 1M (1.40M tokens, 254 tool-uses) ; dimensions
preflight-compliance et sécurité-deep rejouées en fallback Agent avant-plan
texte (cap StructuredOutput, piège récurrent documenté). 0 finding réfuté
par la lentille exactitude.

## Verdict: CONCERN

0 P0/P1 sur les 9 rapports. CONCERN porté par des P2 réels, TOUS traités
avant le round r2 (fixés en code/doc ou requalifiés avec preuve) :

## Findings et traitement

P2 fixés (code) :
- **J-DIFF-1** variante morte `OrchestratorError::ClockRegression` +
  doc mal attribuée → variante SUPPRIMÉE, NOTE d'attribution posée
  (le refus vit à la porte store).
- **J-DIFF-2** `structural_id` byte-as-char corrompait les ids non-ASCII
  (plan signé valide inexécutable, fail-closed) → projection UTF-8-safe
  (bytes verbatim hors marqueurs ASCII) + test `non_ascii_ids_survive_projection`.
- **TS-1 / complétude-#4** routage human.review jamais discriminé (biais
  labels[0] suite-wide) → test `orchestrator_human_review_routes_decided_port`
  (décision label[1] → port routé, autre branche skipped, side-output proof).
- **TS-5** classifier sans bras synthétique → test
  `orchestrator_classifier_routes_proposal_and_defaults` (in-set non-premier
  label + out-of-set → default §19.3).
- **Complétude-#1** chemins fail-closed du run_effect orchestré non testés →
  test `orchestrator_non_idempotent_pending_goes_outcome_unknown` (pending
  non-idempotent → terminal outcome_unknown + RunState::OutcomeUnknown rendu).
- **Complétude-#2** reducer jamais confronté à un journal adverse → test
  `orchestrator_reduce_refuses_adverse_journal` (UnknownNode +
  FanoutBeyondBound, seuls refus atteignables par journal étranger).
- **SD-2** fenêtre de dé-expiration par horloge régressive ENTRE passes →
  **watermark durable par-store** (`clock_watermark` ajoutée à M3 — non
  committée, éditable), `advance_clock_watermark` refuse LOUD toute passe
  régressive (`StoreError::ClockRegression`), câblée en tête de boucle ;
  test `orchestrator_clock_regression_loud` étendu au type exact.
- **SD-3** payload de signal non borné → `MAX_SIGNAL_PAYLOAD_BYTES`
  (256 KiB) à la porte deposit.
- **SD-5** timer traversant un run clos → fire gaté sur run Running
  (hygiène de journal).
- **TS-3** « join fires exactly once » state-only → side_output_count(/out)==1.
- **TS-4** quorum indiscriminé de all → test
  `orchestrator_join_quorum_fires_before_slow_branch` (1-de-2 fire tôt,
  2e source encore parquée).
- **TS-7** « zero rows » narré non asserté → event_count + wait open assertés.
- **J-DIFF-3** parent_scope/child_structural dupliqués → hoistés dans
  `runtime_id.rs`, consommés par les 2 modules.

P2 requalifiés/déclarés (0 code dû) :
- **SC-1 / PC-1** lecture de `workflow_timeout_seconds` en J vs DÉC-5 :
  contradiction INTERNE du preflight confirmée (A14 « cancellation_policies
  consommées » est inopérante sans la deadline). Tranché minimalement côté
  code (seul le champ run-wide est lu ; timeouts node + retries = K).
  Déclaré au commit body (PLAN-ADAPT).
- **TS-2** ordre map sous désordre : la mutation active M7 (agrégation
  inversée → rouge) est la preuve discriminante ; le désordre RÉEL
  n'existe pas dans un moteur sync mono-writer (il arrive avec les
  adaptateurs K). Acté.
- **SD-1** nonce ≠ authentification du déposant → borne honnête ajoutée à
  la row T-RUN-SIGNAL-FORGE-REPLAY (principal = L1, authz route = K).

P3 traités : J-DIFF-4 (doc amplification per-edge), J-DIFF-5 (doc
closed_at=None forensics), J-DIFF-6/SD-4 (borne squat de dépôt au THREAT,
recovery = K), SC-2 (doc 11 vs 10 états), DC-1 (REFERENCE.md clock
livré + pin v25), DC-2 (« research §19.3 »), DC-3 (STORE_MIGRATION_OPS
« Phases I/J »), PC-5 (noms de tests exacts au THREAT), TS-6 (2e bras
couvre — acté), complétude-#3/#5/#6 (actés, frontière documentée).

## Suites au moment du r1

Win nextest workspace 2313/2313+3skip ; Docker crates 176/176 ; Docker
workspace 2317/2317+3skip (`--no-fail-fast`, flake dispatch_loop re-run
solo PASS = classe charge documentée) ; clippy workspace -D warnings vert ;
fmt dual 1.94/1.95 ; doctests + release ; web 548 + operator 201 ; gate
frontier clean census 29 routes 91. 7 mutations actives tuées (M1, M2',
M3', M4, M5-composite, M6, M7) ; M2/M3 défense-en-profondeur documentées.
