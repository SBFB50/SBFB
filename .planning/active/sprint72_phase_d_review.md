# Phase Review — Sprint 72 Phase D

## Verdict: PASS

Promu de PASS-PENDING à PASS après reconciliation Codex (rapport brut lu,
9/9 livrables CONFIRME, 0 GAP, 0 PARTIEL — aucune correction requise ; les
3 P2 sont des carry-overs documentés, pas des bloquants).

(Rigor signal : **3 findings P2+** documentés — ≥1 requis pour un PASS rigoureux.)

Phase D livre l'exécution réseau end-to-end (DESIGN-CONFLICT G8 résolu par
arbitrage PO → Option A). Bloc daemon : route `GET /api/v1/tasks/{id}/result`
+ persistance `result_text` (migration M16). Bloc factory : bras `Network`
submit→poll (un seul `Done`, PO-14) + câblage `provider` backend +
`PATTERNS §P55`.

## Staging check (Step 1bis)
- Phase fichiers : 10 modifiés (6 crates Rust + 1 test + 3 docs) + 2 untracked
  (`sprint72_phase_d_preflight.md`, `sprint72_phase_d_pivot_proposal.md`).
- Planning/docs split : N/A — les artefacts planning sont les **gate
  artefacts de la phase** (preflight + pivot_proposal + ce review + le codex
  à venir), stagés AVEC le commit phase (pattern Phases A/B/C). Pas de
  chore(planning) intermédiaire requis.
- Untracked accidentels : 0 (aucun cache/build/.pdb).
- web/ + tools/factory-operator : **zéro diff** (Phase D backend-only,
  conforme plan §1 ; le front UX est Phase E).

## Suites (§7.4)
- `cargo fmt --all` : appliqué, 0 diff résiduel.
- `cargo clippy -p nexus-coordinator-rs -p nexus-shell-daemon -p sbfb-factory
  --all-targets --locked -- -D warnings` : **0 warning** (les 3 crates touchées,
  `--all-targets` compile aussi le code de test).
- `cargo nextest run -p nexus-coordinator-rs -p sbfb-factory --locked` :
  **395 passed / 0 failed / 0 skipped**.
- `cargo nextest run -p nexus-shell-daemon -E 'test(task_result_route)'` :
  **1 passed** (`task_result_route_404_then_text_on_completed`).
- Release build `cargo build -p nexus-shell-daemon --release` : lancé (vérifié
  avant commit).
- Doctests : aucun `\`\`\`rust` exécutable ajouté (doc-comments en prose + tables) →
  delta doctest 0 ; suite doctest inchangée depuis Phase C.
- Frontend : N/A (web/ + factory-operator zéro diff). Cross-stack : la nouvelle
  route daemon n'altère aucune route existante ; les tests Vitest du shell
  mockent l'API et ne dépendent pas du daemon réel.

## Delta tests
- Rust workspace (canonique CI Linux) : **1537 → 1544 (+7 Phase D)**.
  - `nexus-coordinator-rs/db.rs` : +2 (`set_task_result_persists_retrievable_text`,
    `get_task_result_none_for_missing_task`).
  - `nexus-coordinator-rs/validator.rs` : +0 (assertion `result_text` ajoutée à
    `accepts_valid_result_and_transitions_to_completed` ; 3 call-sites
    `set_task_result` mis à jour).
  - `nexus-shell-daemon/http.rs` : +1 (`task_result_route_404_then_text_on_completed`).
  - `sbfb-factory/provider_router.rs` : net +1 (−1 `network_target_reports_not_implemented`,
    +2 `network_provider_submit_poll_yields_single_done`, `network_provider_poll_timeout`).
  - `sbfb-factory/tests/operator_server.rs` : +3 (`chat_stream_routes_by_session_provider`,
    `chat_session_persists_provider`, `sensitive_action_gated_regardless_of_provider`).
- Vitest : 279 → 279 (+0, web/ non touché). size-limit : 6/6.

## Commit body validation
- Format titre : ✅ `feat(factory): Sprint 72 Phase D — NetworkProvider submit-poll + result-text primitive + provider routing`
- Delta tests cohérent : ✅ (+7, breakdown ci-dessus).
- Scope cuts honoured : ✅ (voir §Scope cuts).
- Co-Authored-By : ✅ (Claude Opus 4.8 1M context).

## Body format validation (Step 4bis, §4.1)
| Section | Présent | Signal |
|---|---|---|
| Contexte | oui | ok |
| Fichiers | oui | ok |
| Delta tests | oui | ok |
| Verification §7.4 | oui | ok |
| Scope cuts | oui | ok |
| G8 traceability | oui | ok |
| Pre-launch protocol | oui | ok |
| Codex verification | oui (FAIT) | ok |
| Carry closure | oui | ok |

## Modified-file branch coverage (Step 2bis, G9)
- `db.rs::set_task_result` (nouveau param `result_text`, colonne écrite) →
  `set_task_result_persists_retrievable_text` + `accepts_valid_..._completed` ✅
- `db.rs::get_task_result` (nouvelle méthode + `TaskResultDetail`) →
  `set_task_result_persists_retrievable_text` (pending+completed) +
  `get_task_result_none_for_missing_task` (None) ✅
- `validator.rs` single path (`result_text` persisté) →
  `accepts_valid_result_and_transitions_to_completed` (assertion `result_text`) ✅
- `validator.rs` quorum path (`best_hash` doublé) → couvert par les 4 tests
  quorum S71 re-verts (toujours verts) ✅
- `tasks_api.rs::get_task_result` (handler 200/404/404/500) →
  `task_result_route_404_then_text_on_completed` (404 pending + 200 completed) ✅
- `http.rs` route registration → même test (oneshot route réelle) ✅
- `provider_router.rs::network_stream` (submit/poll/result/timeout/erreurs) →
  `network_provider_submit_poll_yields_single_done` (happy + single Done + 0 Delta) +
  `network_provider_poll_timeout` (deadline → 1 Error) ✅
- `operator_server.rs::handle_chat_stream` (dispatch ExecutionTarget) →
  `chat_stream_routes_by_session_provider` (ollama vs claude) ✅
- `operator_server.rs::handle_chat_send` (persist provider) →
  `chat_session_persists_provider` (override /send → route ollama) ✅
- Gate avant dispatch préservé → `sensitive_action_gated_regardless_of_provider`
  (ollama + network gated) ✅

## Scope cuts verification (plan §11)
- #12 « streaming token-par-token worker réseau distant → jamais (PO-14) » :
  le bras Network émet **un seul `Done`**, zéro `Delta` token — asserté par
  `network_provider_submit_poll_yields_single_done`. ✅ Respecté (renforcé).
- #2-#6 (feed-distant/reindex/SearchResult/barre recherche/SearchManifest → S73) :
  0 fichier diff. ✅
- #6-#8 (search/open/fork, projet cible distinct, templates → S74) : 0 diff. ✅
- #9/#10 (GPU partagé, quorum redundancy>1 cross-machine → S75) : 0 diff. ✅
- #11 (sharding → S76), #13-#16 : 0 diff. ✅
- La route daemon `/result` + persistance n'est PAS un scope cut : c'est la
  primitive Option A approuvée par le PO (arbitrage DESIGN-CONFLICT 2026-06-03).

## Horizon long-terme + documentation amont (Step 4quater)
- Primitive durable : la route `/result` + colonne `result_text` est la
  primitive de récupération réseau réutilisée par S75 (GPU partagé). Documentée
  dans le preflight §Resolution + `PATTERNS §P55`. ✅
- Alternatives rejetées : pivot_proposal options A/B/C (B reçu-hash le plus
  faible, C diffère à S73) — arbitrage PO tracé. ✅
- Solution la plus poussée : Option A (vraie réponse réseau dans le chat) vs
  le stub. Crate isolation préservée (pas de dep `nexus-coordinator-rs` dans
  sbfb-factory). ✅
- Aucune LOC estimée au plan. ✅

## Findings (rigor signal — 3 P2+)
- **P2 — sync FS dans contexte async** : `network_stream` appelle
  `resolve_daemon()` → `DaemonConnection::discover()` qui fait des
  `std::fs::read_to_string` synchrones à l'intérieur de l'`async_stream`
  (`provider_router.rs`, fn `resolve_daemon`). Lecture locale d'un petit
  fichier (running.json + auth_token), one-shot → impact réel négligeable,
  mais bloque brièvement le thread executor. Carry-over S73 audit : envelopper
  dans `tokio::task::spawn_blocking` si jamais le profil le montre.
- **P2 — perte de spécificité diagnostique au poll** : le poll de statut
  traite `Ok(_) | Err(_) => continue` (HTTP non-2xx ou blip réseau) en
  re-bouclant jusqu'au timeout global, par résilience aux blips transitoires.
  Conséquence : un daemon qui répond durablement 500 produit un message
  générique « timed out » plutôt que l'erreur HTTP réelle. Trade-off résilience
  vs diagnostic ; carry-over S73 : mémoriser la dernière erreur et la surfacer
  dans le message de timeout.
- **P2 — `project_id` placeholder backend** : `default_project_id()` =
  `"operator-chat"` est un placeholder ; la vraie sélection de projet (sous
  quel projet réseau la tâche est soumise) est une décision UX Phase E. Le
  backend route correctement mais le projet par défaut n'a pas de sémantique
  produit définitive. Carry-over Phase E.
- **P3 — double écriture quorum** : sur le chemin quorum,
  `set_task_result(.., best_hash, best_hash, ..)` écrit le texte agréé dans
  `tasks.result_hash` ET `tasks.result_text` (même valeur). Sémantiquement
  correct (les deux sont le texte agréé sur ce chemin, cf. §P53) ; documenté
  en commentaire. Cosmétique.

## Codex gate (§4.5) — zero exemption
- Status : **FAIT** — `codex exec` GPT 5.5 (reasoning xhigh), output brut dans
  `sprint72_phase_d_codex_review.md` (non réécrit).
- Résultat : 9 livrables audités, **9 CONFIRME, 0 GAP, 0 PARTIEL**. PO-14
  vérifié (`dones.len()==1`, `deltas==0`, provider_router.rs:778-782) ; ordre
  gate-avant-dispatch confirmé (operator_server.rs:896-910 gate avant :934-935
  dispatch) ; stub `network_not_implemented` confirmé supprimé.

## Codex reconciliation
- Status : **FAIT**.
- Le rapport Codex brut a été lu. 0 GAP P0/P1/P2 → aucune correction de code.
  Les 3 P2 de cette review (sync-FS-async, diagnostic-poll-générique,
  project_id-placeholder) sont des carry-overs documentés (→ S73 audit / Phase E),
  pas des défauts d'implémentation — Codex a confirmé l'implémentation réelle de
  chaque livrable. Aucune re-exécution de suites nécessaire (aucun code modifié
  post-Codex). Review promu PASS-PENDING → PASS.

## Pre-launch protocol
- Aucun `*_VERSION` bumpé. `TASK_FORMAT_VERSION` / `*_ANNOUNCEMENT_VERSION`
  inchangés. La migration M16 (`ALTER TABLE tasks ADD COLUMN result_text`) est
  un schéma DB **local**, pas un wire format (politique pré-launch §1.4). La
  soumission réseau consomme `TaskSubmission` (JSON inline, serde defaults),
  aucun champ wire ajouté. La nouvelle route `/result` est une route loopback
  locale, pas un wire format tiers.

## Recommendation
- Ready to commit : **oui** — verdict PASS final, Codex FAIT (9/9 CONFIRME,
  0 GAP) et reconcilié.
- Carry-overs S73 (P2+ non résolus → `sprint73_audit_plan.md`) : P2 sync-FS-async,
  P2 diagnostic-poll-générique, P2 project_id-placeholder (ce dernier suivi
  Phase E).

## Post-commit obligatoire
- [ ] Update `nexus_grid_pivot.md` (tip SHA + description + compteurs 1544 Rust)
- [ ] Update `MEMORY.md` (ligne index)
- [ ] Vérifier que ce `review.md` + le `codex_review.md` sont stagés dans le
      commit phase
