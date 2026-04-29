# Sprint 37 — Audit findings (Phase 0 Sprint 38)

**Auditeur** : session fraiche (pas la session qui a code S37).
**Tip d'entree** : `24e93d3` (HEAD master).
**Tip reference S37** : `c53f663` (Phase B, dernier feat commit).
**Documents source** : sprint37_kickoff.md, sprint37_plan.md,
sprint37_verification.md, sprint38_audit_plan.md.

---

## Verdict : **PASS** (0 P0, 0 P1, 1 P2, 1 P3)

Signal G4 rigor : 1 P2 + 1 P3 = satisfait (PASS exige >=1 P2+).

---

## Track A — Securite / crypto

### A-1 : hash-chain BLAKE3+JCS — PASS

`compute_entry_hash` (kudos_ledger.rs:24-38) utilise
`canonical_bytes(&hashable, nexus_core_rs::DOMAIN_KUDOS_V1)` (L34).
`HashableKudosEntry` (L14-22) exclut correctement `entry_hash`
pour eviter la circularite — seuls 7 champs (entry_id,
worker_node_id, task_id, project_id, amount, created_at,
prev_hash) entrent dans le hash.

### A-2 : verify_chain — PASS

`verify_chain` (kudos_ledger.rs:82-98) verifie les 2 invariants :
- `entry.prev_hash != expected_prev` → false (L87)
- `recomputed != entry.entry_hash` → false (L90-91)

Test `verify_chain_tampered` (L211-222) utilise un vrai
`UPDATE SQL` pour simuler le tamper — pas un mock.

### A-3 : unwrap_or_default remplaces — PASS

Aucune occurrence de `unwrap_or_default` dans http.rs.
Les 2 handlers (L1289 et L1403) utilisent desormais
`serde_json::to_value` avec match exhaustif : branche Ok →
200, branche Err → `tracing::error!` + 500 + json error body.

---

## Track B — Architecture / code quality

### B-1 : log convergence — PASS

`paths::log_dir()` (paths.rs:102-104) retourne `<root>/logs/`
(pas `<root>/shell-daemon/logs/`). Le launcher (main.rs:46-53)
appelle `nexus_shell_daemon_core::paths::log_dir()` puis ecrit
`launcher.log` via `tracing_appender::rolling::daily` (L74).
Test `log_dir_is_under_grid_root_not_daemon_dir` (paths.rs:195)
verifie explicitement que log_dir n'est PAS sous shell-daemon/.

### B-2 : validate_result retourne TaskRecord — PASS

`validate_result` (validator.rs:23-26) retourne
`(ValidationOutcome, Option<TaskRecord>)`. Le handler
`coordinator_submit_result` (http.rs:1336) destructure
`Some(task_record)` et utilise `task_record.project_id` (L1340)
directement pour `kudos_ledger::credit()` — pas de double
`db.get_task()`.

### B-3 : rowid tiebreaker — PASS (code), P3 (documentation)

Les 2 queries kudos utilisent `rowid` comme tiebreaker :
- `get_last_entry_hash` : `ORDER BY created_at DESC, rowid DESC`
  (db.rs:202)
- `get_project_entries` : `ORDER BY created_at ASC, rowid ASC`
  (db.rs:217)

Le commit body Phase B documente le rationale ("rowid tiebreaker
pour determinisme intra-seconde"). Pas de commentaire inline dans
db.rs ni de section dans docs/. Le carry P2-REVIEW-B-1-S37
(1/3 → 2/3) reste pertinent.

---

## Track C — Tests / coverage

### C-1 : delta tests 936→946 (+10) — PASS

- Phase A : +4 (3 mutex poisoned + 1 log_dir_is_under_grid_root)
- Phase B : +6 (credit_sets_entry_hash, credit_genesis_hash,
  credit_chains_prev_hash, verify_chain_valid,
  verify_chain_tampered, cross_project_chains_independent)

Chaque test verifie une branche reelle. Les 6 tests Phase B
utilisent `CoordinatorDb::open_in_memory()` avec des operations
SQL reelles.

### C-2 : mutex poisoned tests — PASS

Les 3 tests (http.rs ~L3438, ~L3476, ~L3515) empoisonnent le
mutex correctement :
1. `std::thread::spawn` + `let _guard = db_arc.lock().unwrap()` +
   `panic!("intentional poison")`
2. `.join()` pour recuperer le panic
3. Assert `lock().is_err()` (mutex effectivement empoisonne)
4. Appel au handler → assert 500

### C-3 : verify_chain_tampered — PASS

Le test (kudos_ledger.rs:211-222) utilise un vrai
`db.conn().execute("UPDATE kudos SET entry_hash = 'tampered' ...")`
— pas de mock. La verification recompute le hash et detecte le
tamper.

---

## Track D — Process / meta

### D-1 : G8 preflights coherence — PASS

Phase B preflight (sprint37_phase_B_preflight.md) verdict
EXECUTE plan-as-is. Scans S1a (hash-chain pattern standard),
S1b (blake3 deja workspace), S2 (0 DEVIATION), S3 (fast-path),
S4 (fast-path, DOMAIN_KUDOS_V1 utilise pas modifie). Le code
livre est coherent avec les scans : meme approche BLAKE3+JCS,
meme domain, pas de deviation.

### D-2 : scope cuts §7 respectes — PASS

Les 12 scope cuts sont respectes. Aucune occurrence dans le code
de : verify_chain endpoint HTTP, OutputFilter, PiiRedactor,
CanaryRegistry, LiveEvents validator, debit/stake/burn kudos,
CI multi-OS, VPS deployment, code signing macOS.

### D-3 : 2 MANDATORY 3/3 fermes — PASS

- P2-B-1-S34 log convergence : confirme ferme (paths::log_dir()
  + launcher tracing-appender)
- P2-C-1-S34 .icns macOS : confirme ferme (tools/png-to-icns +
  bundle-macos.sh wire)

---

## Track E — Dependencies

### E-1 : blake3 dep coordinator-rs — **P2**

**Finding P2-AUDIT-1-S37 : Cargo.lock desync**

`blake3 = { workspace = true }` est correctement declare dans
`nexus-coordinator-rs/Cargo.toml`. blake3 1.5 est une workspace
dep existante (Cargo.toml:58). Pas de nouvelles transitives.

**Mais** : le commit Phase B (`c53f663`) n'a PAS mis a jour
`Cargo.lock`. Le lockfile committe (HEAD `24e93d3`) ne liste pas
`blake3` dans les dependencies de nexus-coordinator-rs. Le
working tree contient la correction (1 ligne ajoutee).

Verification :
```
git show HEAD:Cargo.lock | grep -A 15 'name = "nexus-coordinator-rs"'
→ blake3 ABSENT de la liste
git diff Cargo.lock
→ + "blake3" dans nexus-coordinator-rs deps
```

Impact : `cargo check --locked` sur un checkout frais echouerait
potentiellement. Risque mitige car blake3 est deja dans le
lockfile via nexus-core-rs, mais la declaration par-crate est
manquante.

**Fix** : committer le Cargo.lock corrige.

### E-2 : icns 0.3 + image 0.25 (build-only) — PASS

`tools/png-to-icns/Cargo.toml` est un crate separe avec
`publish = false`. Les deps icns + image ne sont PAS dans les
crates runtime (daemon, launcher, coordinator-rs, worker).
Le crate est dans `[workspace.members]` (Cargo.toml:16) mais
n'est pas importe par aucun autre crate.

---

## Track F — Doc coherence

### F-1 : HARDENING_ROADMAP compteurs — PASS

`last_validated: 2026-04-29`, compteurs `946 Rust / ~1949 total`.
Coherent avec verification.md.

### F-2 : CLAUDE.md etat actuel — PASS

CLAUDE.md §Etat actuel : "Sprints 0-37 CLOSED", "~1949 tests
total (946 Rust / 195 SDK / 409+36f+6s coord / 46 gov / 267
Vitest / 42+2f PW / 7/7 size)". Coherent avec verification.md.

### F-3 : Phase review files — PASS (2/2)

- sprint37_phase_A_review.md : present dans archive/v1.2/
- sprint37_phase_B_review.md : present dans archive/v1.2/

### F-4 : Phase preflight files — PASS (2/2)

- sprint37_phase_A_preflight.md : present dans archive/v1.2/
- sprint37_phase_B_preflight.md : present dans archive/v1.2/

---

## Resume findings

| # | Track | Severite | Description |
|---|---|---|---|
| P2-AUDIT-1-S37 | E-1 | **P2** | Cargo.lock desync : blake3 manquant dans deps coordinator-rs du lockfile committe |
| P3-AUDIT-1-S37 | B-3 | P3 | rowid tiebreaker documente dans commit body seulement, pas inline (carry existant P2-REVIEW-B-1-S37) |

---

## Carries S38 (confirmes)

| Item | Compteur | Action |
|---|---|---|
| P2-A-1 rand blocker upstream | 6+/3 | exemption blocker externe |
| P2-REVIEW-C-1-S35 validator_loop tokio | 3/3 **MANDATORY** | refactor CuratorRuntimeHandle |
| P2-AUDIT-2-S35 pre-release transitives iroh | herite | pin 0.98 |
| P3-grammar executor | 3/3+ | defer Rust pipeline |
| P3-watermark executor | 3/3+ | defer Rust pipeline |
| P2-REVIEW-A-1-S37 launcher logging test | 1/3 | Phase A review carry |
| P2-REVIEW-B-1-S37 rowid documentation | 2/3 (confirme audit) | B-3 finding |
| **P2-AUDIT-1-S37 Cargo.lock desync** | **FIX NOW** | commit lockfile |
