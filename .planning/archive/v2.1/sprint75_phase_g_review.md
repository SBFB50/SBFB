# Review — Sprint 75 Phase G (wrap-up + survives-VPS-death + carries S74)

Méthode : Workflow multi-agent 5 dimensions adversariales (correctness,
security, tests, wire-docs, completeness) → skeptics refute-by-default sur
P0/P1 → synthèse. Run `wf_08072259-7a9`, 6 agents, ~611k tokens.

## Verdict: PASS

0 P0 / 0 P1 confirmé par les skeptics (0 finding bloquant à réfuter — aucun
émis). 3 findings non-bloquants, tous traités in-phase (voir table ci-dessous).
Promu de PASS-PENDING à PASS après la gate Codex (round 2 = 22 CONFIRMED,
0 GAP, OVERALL: PASS).

## Vérifications code-ancrées passées

- **CARRY-2** `reject_result_on_guardrail_trip` (validator.rs:190) appelé
  symétriquement aux 2 ingress (`http.rs` coordinator_submit_result +
  `validator_loop.rs`) sur la branche `Accepted + Some(pending)` ;
  `update_task_status` → Rejected (db.rs:485) ; terminalité garantie par le
  status gate de `validate_result_pre_guardrail` (refuse Rejected→re-soumission
  = RejectedTaskNotPending).
- **PULL-1** `strip_zip_member` inséré AVANT hash + AVANT l'inject
  `add_to_zip`/`new_append` (qui appende réellement) ; no-op byte-identique en
  branche absente (`zip_bytes.to_vec()`) ; `by_index_raw`/`raw_copy_file`
  API-corrects (zip 8.6.0).
- **FORK-1** `MAX_ARCHIVE_ENTRIES=4096` rejette AVANT tout write disque
  (test `!dest.exists()`).
- **CARRY-5** `offset.min(10000)` + `truncate_on_char_boundary(q, 1024)`
  UTF-8-safe ; `limit` déjà `.min(100)` (non re-touché, conforme au scope —
  delta preflight respecté).
- 0 bump wire (tous `*_FORMAT_VERSION` = 1) ; pas de nouveau DOMAIN ; pas de
  dep ajoutée.

## Findings (3 non-bloquants — tous corrigés in-phase)

| ID | Sév. | Titre | Statut |
|---|---|---|---|
| WD-1 | P3 | breakdown per-phase Rust double-comptait Phase A (somme 80 ≠ 72 annoncé) | **CORRIGÉ** : reframe sur baseline S74-sortie 1674 → +81 (A +8, B +32, C +10, D +11, E +13, F +2, G +5 = 81) dans verification.md §5, CLAUDE.md, SPRINT_LOG ; hedge « vague » retiré |
| COMPLETE-1 | P3 | sprint76_audit_plan.md routait l'auditeur vers 2 tests fantômes (rows 7 + 9) | **CORRIGÉ** : ligne 111 `stale_announcement_…` → les 3 tests réels `replay_*` (runtime.rs) ; ligne 112 `…_replay_rejected` → `node_directory_cross_domain_signature_rejected` |
| COMPLETE-2 | NIT | terminalité guardrail-trip non testée sur le chemin quorum (redundancy>1) que le docstring prétend couvrir | **CORRIGÉ** (implémenté, pas différé) : `guardrail_trip_on_quorum_path_sets_rejected_terminal` (validator_loop.rs) — 2 workers agree sur texte trippant → quorum → Rejected terminal + 0 texte + clean ultérieur refusé. PASS |

## Différé (consigné dans les bons exutoires)

- Surfaces front sans duress gate (`seed_voluntary`/`set_keep_online`) →
  THREAT_MODEL §15.1 row E/T, routé audit S76.
- Fresh-flood + sampling anti-Sybil → THREAT_MODEL §15.1, audit S76.
- Fenêtre-morte 1er-boot + `SeedAnnounced` non-convergé + seeder
  `catalog_len:0` (observés acceptance live) → verification.md §8 + audit S76.
- LT-2 flip Radicle publié → ARMÉ + dry-run privé fait ; flip réel = décision
  PO hors-sprint (irréversible).

## Codex reconciliation

Gate Codex GPT-5.5 (`sprint75_phase_g_codex_review.md`, sortie brute `codex exec -o`).

- **Round 1** : 15 CONFIRMED + **1 GAP** — incohérence de chiffre doc : la
  checklist de clôture §9 de verification.md disait Docker `1758/1758` alors que
  le row 6 + la table métriques disaient `1759/1759`. Corrigé (édition pure,
  0 code touché : verification.md §9 → 1759/1759).
- **Round 2** : **22 CONFIRMED, 0 GAP, OVERALL: PASS.** Codex a vérifié les 4
  carries avec evidence file:line (clamp offset/q UTF-8-safe, helper Rejected
  terminal aux 2 ingress + couverture single ET quorum, strip-before-inject +
  no-op byte-identique, entry-cap avant `create_dir_all`), a re-exécuté lui-même
  les 7 tests Phase G (passed), a re-compté `nextest list --workspace` = 1755
  (confirme l'ajustement +1 quorum), et a confirmé l'honnêteté des docs
  (THREAT_MODEL §15.1 route les résiduels vers S76 sans prétendre les fermer ;
  §P59/§P37 décrivent le code réel ; 0 bump wire ; 0 delta dep).

Livrables : 22 audités, 22 confirmés, 1 GAP (round 1) corrigé.
