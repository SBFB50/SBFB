# Sprint 45 — Plan d'execution detaille

**Tip master d'entree** : `eccff1f`.
**Decisions Day 0** : D1 suppression maximale (pas totale),
D2 portage invite + quarantine, D3 7 carries resolus,
D4 coordinator Python gut.

---

## §1 Etat verifie a l'entree

| Suite | Count |
|---|---|
| Rust nextest | 1127 |
| Rust doctests | 6 passed, 1 ignored |
| SDK pytest | 195 |
| Coord pytest | 409 + 36 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 267 |
| Playwright | 42 + 2 fail (env pre-existant) |
| size-limit | 7/7 |
| **Total** | **~2130** |

Clippy : 0 warnings.
Size-limit : 7/7 budgets respectes.

---

## §2 Decisions Day 0 (gelees)

- **D1** : scope realiste — suppression maximale, pas totale.
  Runtime apps Python subsiste.
- **D2** : 6 routes invite + quarantine → axum Rust.
- **D3** : 7 carries resolus (SHA-256→BLAKE3, coord dead_code,
  tokio::fs, list_tasks status, TOCTOU canary, silent null canary,
  hex case-sensitivity).
- **D4** : coordinator Python gut — ~12 fichiers routes + ~14
  modules redondants + tests associes supprimes.

---

## §3 Research consulte

- `crates/nexus-coordinator-rs/src/invite.rs` : `InviteLedger`
  struct avec `create_invite()`, `list_invites()`,
  `revoke_invite()`. Utilise Ed25519 signing.
- `crates/nexus-coordinator-rs/src/quarantine_queue.rs` :
  `QuarantineQueue` struct avec `add()`, `list()`, `flush()`,
  `drop_item()`. SQLite WAL.
- `crates/nexus-coordinator-rs/src/redundancy.rs` l.7 :
  `use sha2::{Digest, Sha256}` — seul consumer de sha2 dans le
  crate.
- `crates/nexus-shell-daemon/src/http.rs` l.141-148 :
  `coord_http_client` + `coord_base_url` — 0 handler consumer.
- `crates/nexus-shell-daemon/src/worker_state_api.rs` :
  `std::fs::read_to_string` dans handler async.
- `packages/nexus-coordinator/src/nexus_coordinator/api/` :
  12+ fichiers route Python portes S35-S44.

---

## §4 Dependencies inter-phases

```
Phase A → Phase B :
  B supprime les modules Python dont les equivalents Rust viennent
  d'etre renforces en A (invite routes, quarantine routes).
  Les tests Rust de A valident que le daemon peut servir les routes
  autonomement AVANT la suppression Python de B.
```

---

## §5 Phase A — Route portage + carries resolus

### §5.1 Scope

Porter les 6 routes invite + quarantine vers des handlers axum
dans `crates/nexus-shell-daemon/src/`. Resoudre les 7 carries
identifies dans D3.

### §5.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-shell-daemon/src/invite_api.rs` (NEW) | 3 handlers : create_invite, list_invites, revoke_invite |
| `crates/nexus-shell-daemon/src/quarantine_api.rs` (NEW) | 3 handlers : list_quarantine, flush_quarantine, drop_quarantine |
| `crates/nexus-shell-daemon/src/http.rs` | +6 routes, +2 mod declarations |
| `crates/nexus-shell-daemon/src/main.rs` | +2 mod declarations |
| `crates/nexus-coordinator-rs/src/db.rs` | +6 queries (invite CRUD + quarantine CRUD) |
| `crates/nexus-coordinator-rs/src/redundancy.rs` | sha2::Sha256 → blake3::hash() |
| `crates/nexus-shell-daemon/src/worker_state_api.rs` | std::fs → tokio::fs |
| `crates/nexus-shell-daemon/src/tasks_api.rs` | validation status enum avant query |
| `crates/nexus-shell-daemon-core/src/canary_input.rs` | TOCTOU reload fix (RwLock ou AtomicBool) |
| `crates/nexus-shell-daemon/src/canary_api.rs` | silent null → 500 error |
| Source hex validation (a localiser grep) | lowercase normalisation |

### §5.3 Tests plan

1. `test_create_invite_success` — POST /api/v1/invite/create avec body valide
2. `test_create_invite_invalid_body` — POST avec body invalide → 422
3. `test_list_invites_empty` — GET /api/v1/invite → liste vide
4. `test_list_invites_populated` — POST + GET → liste contient l'invite
5. `test_revoke_invite_success` — DELETE /api/v1/invite/{id} → 200
6. `test_revoke_invite_not_found` — DELETE avec id inexistant → 404
7. `test_list_quarantine_empty` — GET /api/v1/quarantine → liste vide
8. `test_flush_quarantine_success` — POST /api/v1/quarantine/{id}/flush → 200
9. `test_drop_quarantine_success` — POST /api/v1/quarantine/{id}/drop → 200
10. `test_redundancy_uses_blake3` — hash output matches blake3::hash()
11. `test_worker_state_tokio_fs` — handler reads state.json via async fs
12. `test_list_tasks_invalid_status` — GET /api/v1/tasks?state=garbage → 400

### §5.4 Critere d'acceptation

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc
```

### §5.5 Commit cible

```
feat(sprint45): Sprint 45 Phase A — invite + quarantine API Rust + SHA-256→BLAKE3 + 6 carries resolus

Routes portees :
- invite_api.rs : 3 routes (create, list, revoke) wire invite.rs
- quarantine_api.rs : 3 routes (list, flush, drop) wire quarantine_queue.rs
- db.rs : +6 queries (invite CRUD + quarantine CRUD)

Carries resolus :
- P2-REVIEW-C-1-S40 SHA-256→BLAKE3 (6/3) : redundancy.rs sha2→blake3
- P2-REVIEW-C-1-S44 worker_state tokio::fs (1/3) : std::fs→tokio::fs
- P3-REVIEW-C-2-S44 list_tasks status invalide (1/3) : validation enum
- P3-REVIEW-A-1-S43 TOCTOU canary reload (2/3) : [detail]
- P3-AUDIT-A-2-S43 silent null canary_api (2/3) : fallback→500
- P3-AUDIT-A-3-S43 hex case-sensitivity (2/3) : lowercase normalisation

Scope cuts respectes :
- events.py SSE — S46+ (dep AppEvents bus Rust)
- App runtime migration — S46-47
- Frontend URL migration — S46

Delta tests : +N (1127→1127+N)
```

---

## §6 Phase B — Coordinator Python gut + dead code cleanup

### §6.1 Scope

Supprimer les routes/modules/tests Python redondants du coordinator.
Supprimer le dead code Rust (coord_http_client etc.). Adapter le
coordinator app.py pour ne plus monter les routes supprimees.
Mettre a jour la documentation.

### §6.2 Fichiers touches

| Fichier | Role |
|---|---|
| `packages/nexus-coordinator/src/nexus_coordinator/api/deploy.py` | DELETE (ported S42) |
| `packages/nexus-coordinator/src/nexus_coordinator/api/apps.py` | DELETE (ported S42) |
| `packages/nexus-coordinator/src/nexus_coordinator/api/consent.py` | DELETE (ported S43) |
| `packages/nexus-coordinator/src/nexus_coordinator/api/files.py` | DELETE (ported S43) |
| `packages/nexus-coordinator/src/nexus_coordinator/api/canary.py` | DELETE (ported S43) |
| `packages/nexus-coordinator/src/nexus_coordinator/api/contributor.py` | DELETE (ported S43) |
| `packages/nexus-coordinator/src/nexus_coordinator/api/health.py` | DELETE (ported S44) |
| `packages/nexus-coordinator/src/nexus_coordinator/api/shell.py` | DELETE (ported S44) |
| `packages/nexus-coordinator/src/nexus_coordinator/api/tasks.py` | DELETE (ported S35+S44) |
| `packages/nexus-coordinator/src/nexus_coordinator/api/kudos.py` | DELETE (ported S36+S44) |
| `packages/nexus-coordinator/src/nexus_coordinator/api/diagnostic.py` | DELETE (ported S44) |
| `packages/nexus-coordinator/src/nexus_coordinator/api/worker_state.py` | DELETE (ported S44) |
| `packages/nexus-coordinator/src/nexus_coordinator/api/invites.py` | DELETE (ported S45 Phase A) |
| `packages/nexus-coordinator/src/nexus_coordinator/api/quarantine.py` | DELETE (ported S45 Phase A) |
| `packages/nexus-coordinator/src/nexus_coordinator/dispatcher.py` | DELETE (→ Rust S35) |
| `packages/nexus-coordinator/src/nexus_coordinator/validator.py` | DELETE (→ Rust S35) |
| `packages/nexus-coordinator/src/nexus_coordinator/kudos.py` | DELETE (→ Rust S36) |
| `packages/nexus-coordinator/src/nexus_coordinator/output_filter.py` | DELETE (→ Rust S38) |
| `packages/nexus-coordinator/src/nexus_coordinator/guardrails.py` | DELETE (→ Rust S38) |
| `packages/nexus-coordinator/src/nexus_coordinator/result_guardrails.py` | DELETE (→ Rust S38) |
| `packages/nexus-coordinator/src/nexus_coordinator/pii_redactor.py` | DELETE (→ Rust S39) |
| `packages/nexus-coordinator/src/nexus_coordinator/canary_registry.py` | DELETE (→ Rust S39) |
| `packages/nexus-coordinator/src/nexus_coordinator/fairness.py` | DELETE (→ Rust S41) |
| `packages/nexus-coordinator/src/nexus_coordinator/pow_counter.py` | DELETE (→ Rust S41) |
| `packages/nexus-coordinator/src/nexus_coordinator/capability_store.py` | DELETE (→ Rust S41) |
| `packages/nexus-coordinator/src/nexus_coordinator/contributor_registry.py` | DELETE (→ Rust S41) |
| `packages/nexus-coordinator/src/nexus_coordinator/redundancy.py` | DELETE (→ Rust S40) |
| `packages/nexus-coordinator/src/nexus_coordinator/watermark_detector.py` | DELETE (→ Rust S40) |
| `packages/nexus-coordinator/tests/test_*.py` | DELETE tests correspondants |
| `crates/nexus-shell-daemon/src/http.rs` | -coord_http_client, -coord_base_url, -resolve_coord_base_url(), -COORD_BASE_URL_ENV, -DEFAULT_COORD_BASE_URL |
| `crates/nexus-shell-daemon/src/runtime.rs` | -coord_http_client init, -coord_base_url init |
| `packages/nexus-coordinator/src/nexus_coordinator/api/app.py` | Retirer include_router des routes supprimees |
| `CLAUDE.md` | Update etat actuel (S45 CLOSED + coordinator gut) |
| `docs/security/HARDENING_ROADMAP.md` | Update last_validated + compteurs |
| `docs/claude/SPRINT_LOG.md` | Ajouter row S45 |

### §6.3 Tests plan

1. Verifier `uv run ruff format --check packages/` PASS
2. Verifier `uv run ruff check packages/` PASS
3. Verifier `uv run pytest packages/nexus-sdk/tests/ -q` PASS (195 — unchanged)
4. Verifier coord pytest restant (runtime apps only) PASS
5. Verifier `uv run pytest packages/nexus-app-gov/tests/ -q` PASS (46 — unchanged)
6. Verifier frontend build + tests PASS (unchanged)
7. Verifier `cargo nextest run --workspace --locked` PASS

### §6.4 Critere d'acceptation

```bash
# Full fail-fast
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc
cargo build -p nexus-shell-daemon --release
uv run ruff format --check packages/ && uv run ruff check packages/
uv run pytest packages/nexus-sdk/tests/ -q
uv run pytest packages/nexus-coordinator/tests/ -q
uv run pytest packages/nexus-app-gov/tests/ -q
cd web && npm run lint && npx tsc --noEmit -p tsconfig.app.json && \
  npm run test:unit && npm run build && npm run size && \
  npx playwright test && bash scripts/scan-en-strings.sh
```

### §6.5 Commit cible

```
feat(sprint45): Sprint 45 Phase B — coordinator Python gut + dead code Rust cleanup

Coordinator Python gut :
- DELETE 14 fichiers routes API (deploy, apps, consent, files,
  canary, contributor, health, shell, tasks, kudos, diagnostic,
  worker_state, invites, quarantine) — tous portes Rust S35-S45
- DELETE 14 modules logique (dispatcher, validator, kudos,
  output_filter, guardrails, result_guardrails, pii_redactor,
  canary_registry, fairness, pow_counter, capability_store,
  contributor_registry, redundancy, watermark_detector) — tous
  portes Rust S35-S41
- DELETE tests correspondants (~N fichiers, ~M tests)
- ADAPT app.py routing (retirer include_router supprimees)

Dead code Rust :
- P2-REVIEW-B-1-S43 coord dead_code (2/3) RESOLU :
  coord_http_client + coord_base_url + resolve_coord_base_url()
  supprimes de http.rs + runtime.rs

Coordinator Python residuel : runtime apps uniquement
  (AppContext, events, commands, state, hooks, rerun, MCP,
  canary_input coordinator-side, upload_queue, tor_client)

Scope cuts respectes :
- events.py SSE — S46+ (dep AppEvents bus Rust)
- App runtime — S46-47
- Frontend URL migration — S46

Delta tests : -N coord (409→~M restants) +0 Rust
```

---

## §7 Phase C — Wrap-up

Verification.md + audit_plan S46.

---

## §8 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | `cargo fmt --all --check` | `cargo fmt --all --check` | 0 diff | |
| 2 | `cargo clippy` | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | |
| 3 | `cargo nextest` | `cargo nextest run --workspace --locked` | PASS | |
| 4 | `cargo test --doc` | `cargo test --workspace --locked --doc` | PASS | |
| 5 | `cargo build --release` | `cargo build -p nexus-shell-daemon --release` | PASS | |
| 6 | `ruff format` | `uv run ruff format --check packages/` | PASS | |
| 7 | `ruff check` | `uv run ruff check packages/` | PASS | |
| 8 | SDK pytest | `uv run pytest packages/nexus-sdk/tests/ -q` | PASS (195) | |
| 9 | Coord pytest | `uv run pytest packages/nexus-coordinator/tests/ -q` | PASS (residuel) | |
| 10 | Gov pytest | `uv run pytest packages/nexus-app-gov/tests/ -q` | PASS (46) | |
| 11 | npm lint | `npm --prefix web run lint` | PASS | |
| 12 | tsc | `npx --prefix web tsc --noEmit -p web/tsconfig.app.json` | PASS | |
| 13 | Vitest | `npm --prefix web run test:unit` | PASS (267) | |
| 14 | Vite build | `npm --prefix web run build` | PASS | |
| 15 | size-limit | `npm --prefix web run size` | PASS (7/7) | |
| 16 | Phase A preflight G8 | EXECUTE | PASS | |
| 17 | Phase A review | PASS | | |
| 18 | Phase B preflight G8 | EXECUTE | PASS | |
| 19 | Phase B review | PASS | | |
| 20 | 6 routes invite+quarantine portees | code | PASS | |
| 21 | 7 carries resolus | code | PASS | |
| 22 | ~12 fichiers routes Python supprimes | git diff --stat | PASS | |
| 23 | ~14 modules Python supprimes | git diff --stat | PASS | |
| 24 | Dead code Rust supprime | grep coord_http_client | PASS | |
| 25 | Scope cuts respectes | diff | PASS | |
| 26 | SHA-256→BLAKE3 resolu | grep sha2 redundancy.rs | PASS | |
| 27 | Delta tests Phase A+B | count | documented | |
| 28 | Docs mis a jour | CLAUDE.md + SPRINT_LOG + HARDENING | PASS | |

**Critere SMART** : 28/28 PASS.

---

## §9 Git plan

```
1. chore(planning): sprint 45 kickoff + plan + design review + migration S44 archive
2. chore(planning): sprint 45 Phase A preflight G8
3. feat(sprint45): Sprint 45 Phase A — invite + quarantine API Rust + carries resolus
4. chore(planning): sprint 45 Phase B preflight G8
5. feat(sprint45): Sprint 45 Phase B — coordinator Python gut + dead code Rust cleanup
6. chore(sprint45): Phase C — wrap-up + verification + audit plan S46 + counters
```

---

## §10 Scope cuts

1. events.py SSE streaming — S46+ (dep AppEvents bus Rust)
2. App runtime migration Rust — S46-47
3. Frontend coordinator→daemon URL migration — S46
4. MCP server migration Rust — S46+
5. PyO3 bindings removal — S46+
6. Suppression complete coordinator Python — S46-47
7. CI/VPS/v1.0 — S46-48
8. Kudos debit/stake — interdit (Day 0 #7)

---

## §11 Risks

| # | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | invite.rs API mismatch vs Python | Low | Low | Tests unitaires couvrent le contrat |
| R2 | quarantine_queue.rs SQLite lock contention | Low | Low | WAL mode deja configure |
| R3 | Suppression Python casse import chain | Medium | Medium | Tests Python detectent immediatement |
| R4 | BLAKE3 output different de SHA-256 | None | None | Pre-launch = pas de backward compat |
| R5 | Test count drop | Expected | Low | Code supprime, pas comportement |

---

## §12 Checkpoint de cloture

1. 6 routes invite+quarantine portees axum Rust
2. 7 carries resolus
3. ~12 fichiers routes Python supprimes
4. ~14 modules Python supprimes
5. Dead code Rust supprime (coord_http_client etc.)
6. 8 scope cuts respectes
7. 28/28 fail-fast checklist
