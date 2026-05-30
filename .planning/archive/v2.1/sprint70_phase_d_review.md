# Sprint 70 Phase D — Review Deep

Date : 2026-05-25 | HEAD pre-commit : `c68e989` (modifie)

## Verdict: PASS

Post-Codex. Suites vertes. Tous les P1/GAPs corriges. Codex reconcilie.

## Suites verification

- cargo fmt --all --check : 0 diff.
- cargo clippy --workspace --all-targets --locked -- -D warnings : 0 warnings.
- cargo nextest run -p sbfb-factory --locked : 90/90 PASS.
- cargo nextest run --workspace --locked : 979/981 (2 flaky browse DNS, non lies).
- cargo test --workspace --locked --doc : OK.
- cargo build -p sbfb-factory --release --locked : OK.
- Frontend : lint 0 errors, tsc OK, 279/279 Vitest, build OK, size OK.

## Livrables Phase D

| Fichier | Status | LOC |
|---------|--------|-----|
| crates/sbfb-factory/Cargo.toml | UPDATE | +3 deps (axum, tokio, tower-http) |
| crates/sbfb-factory/src/process.rs | UPDATE | +430 (status-sprint, lint-planning, audit-commit, context_data/prompt_data refactor) |
| crates/sbfb-factory/src/operator_server.rs | NEW | ~640 (axum server 13 endpoints, action log, chat sessions, context-pack, artifact draft) |
| crates/sbfb-factory/src/main.rs | UPDATE | +50 (StatusSprint, LintPlanning, AuditCommit, Operator Serve) |
| crates/sbfb-factory/tests/process_cli.rs | UPDATE | +10 tests (status_sprint x3, lint x3, audit x4) |
| crates/sbfb-factory/tests/operator_server.rs | NEW | 22 tests (endpoints, context-pack, chat, actions, artifact draft, traversal) |
| docs/agent/TOOLING.md | UPDATE | +60 lignes (Process Observability + Operator JSON API) |
| Cargo.lock | AUTO | mise a jour deps |

## Findings

### P1-1 path traversal (CORRIGE)
`handle_artifact_draft` manquait le rejet de `..` dans le path.
Ajoute `if normalized.contains("..") { return 403; }` avant le
check allowlist. Test ajoute :
`operator_artifact_draft_rejects_path_traversal`.

### P2-2 PASS-PENDING false positive (CORRIGE)
La detection PASS verdict dans le contenu utilisait
`contains("## Verdict: PASS")` qui matchait "PASS-PENDING".
Corrige avec un check ligne-par-ligne excluant PASS-PENDING.
Test ajoute : `operator_artifact_draft_allows_pass_pending`.

### P2-3 CORS Any (DOCUMENTE)
`CorsLayer::new().allow_origin(Any)` au lieu de localhost-only.
Acceptable pour Phase D (outil local, pas de donnees sensibles
exposees). Plan Phase F durcira si la surface persiste.

### P2-4 Missing success action test (CORRIGE)
Pas de test pour `/api/actions/run` avec commande autorisee.
Ajoute `operator_action_run_allowed_command`.

### P3-1 repo_root_pub naming (DOCUMENTE)
Nom `repo_root_pub()` inelegant. Pattern temporaire pour exposer
des fonctions privees au module operator_server. A refactorer
si sbfb-factory devient lib + bin.

### P3-2 Mutex unwrap (DOCUMENTE)
`state.action_log.lock().unwrap()` dans les handlers. Panic si
poison. Acceptable pour outil local CLI — pas de recovery
sensible sur un lock empoisonne.

### P3-3 Test artifact cleanup (CORRIGE)
Tests operator ecrivaient dans `.planning/active/` reel sans
cleanup. Corrige avec noms non-conflictuels + `fs::remove_file`
dans les tests.

## Scope cuts

Aucun scope cut. Tous les livrables du plan §7 sont livres.

## Codex reconciliation

Codex GPT 5.5 session 019e5dd2, 2026-05-25. Verdict global GAP.
Fichier brut : `.planning/active/sprint70_phase_D_codex_review.md`.

- GAP-1 PASS-PENDING substring match (process.rs:138 + :542) : CORRIGE.
  Introduit `has_final_pass_verdict()` qui exclut PASS-PENDING.
  status-sprint reporte maintenant Phase D (pas E) en presence
  de PASS-PENDING. audit-commit rejette aussi correctement.
- GAP-2 chore audit skip : DOCUMENTE. `chore` phase commits ne
  requierent pas de review/codex/body (coherent avec le process
  SBFB). `is_phase_commit: true` + `ok: true` est le comportement
  attendu. Docs clarifient.
- GAP-3 specialized_kind path leak : CORRIGE. Validation `..`, `/`,
  `\` dans le filename avant hashing. Aucun path arbitraire ne fuit.
- PARTIAL-4 allowlist prefix exact files : CORRIGE. Distinction
  entries avec trailing `/` (prefixes) et sans (fichiers exacts).
  `AGENTS.md.bak` ne passe plus.
- PARTIAL-5 tests repo-dependent : DOCUMENTE. Les tests
  integration contre le repo reel sont par design (comme les
  16 tests existants Phase C). Fixtures utilisees pour les cas
  edge (tempdir).

Suites relancees post-correction : 90/90 PASS sbfb-factory.

## Delta tests

- Rust workspace : 1449 -> 1481 (+32 Phase D, dont +10 process_cli + 22 operator_server).
- Vitest : 279 -> 279.
- Plan annoncait +30, reel +32 (+2 tests securite/regression).
