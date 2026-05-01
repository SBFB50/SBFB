# Sprint 46 — Plan d'execution detaille

**Tip master d'entree** : `d1dd4bd`.
**Decisions Day 0** : D1 Router oneshot harness, D2 MANDATORY 12 +
extensions 14, D3 frontend direct-daemon unification, D4 dette
batch 5 items S44.

---

## §1 Etat verifie a l'entree

| Suite | Count |
|---|---|
| Rust nextest | 1132 (0 fail, 0 skip) |
| Rust doctests | 6 passed, 1 ignored |
| SDK pytest | 195 |
| Coord pytest | 323 + 23 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 268 |
| Playwright | 42 + 2 fail (env pre-existant) |
| size-limit | 7/7 |
| **Total** | **~1949** |

clippy 0 warnings, fmt clean.

## §2 Decisions Day 0 (gelees)

- **D1** : enrichir mk_state() + Router::oneshot() pattern
  existant (21 precedents dans http.rs)
- **D2** : Phase A = 12 routes MANDATORY, Phase B += 14 routes
  recentes
- **D3** : coordinator.ts paths → `/api/v1/*` daemon, daemon.ts
  drop proxy envelope, callers passent daemon URL
- **D4** : 5 items S44 2/3 batch complet en Phase B

## §3 Research consulte

- Audit factuel integration tests : 54 routes total, 21 ont
  integration tests (http.rs mod tests), 33 manquent
- coordinator.ts : 720 LOC, 20 fonctions exportees, paths sans
  prefix `/api/v1/`
- daemon.ts : 442 LOC, proxy envelope `{ kind, status, body }`,
  paths `/daemon/*`
- mk_state() dans http.rs : fournit DaemonHttpState pour tests,
  actuellement canary_input None pour certains champs

## §4 Dependencies inter-phases

Phase A (tests MANDATORY 12 routes) → independant
Phase B (dette + tests 14 routes) → depend du harness enrichi en A
Phase C (frontend direct-daemon) → independant de A/B (frontend)
Phase D (wrap-up) → depend de A + B + C

---

## Phase A — Integration tests 12 routes MANDATORY P2-AUDIT-A-1-S43

### §A.1 Scope

Enrichir le test harness `mk_state()` dans http.rs pour fournir
un DaemonHttpState complet (canary_input Some(...), consent state,
files tmpdir), puis ecrire des tests Router::oneshot() pour les
12 routes du carry MANDATORY.

Routes ciblees :
1. `consent.rs` : GET /consent, POST /consent/set,
   POST /consent/whitelist/add, POST /consent/whitelist/remove
2. `files.rs` : POST /files/upload, GET /files/{sha256}/manifest,
   GET /files/{sha256}
3. `canary_api.rs` : GET /canary/freshness/{pubkey},
   POST /canary/inject-rate, GET /canary/observed-divergence
4. `contributor_api.rs` : GET /contributor/project/{project_id},
   GET /contributor/envelope/{project_id}/{node_id_hex}

### §A.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-shell-daemon/src/http.rs` | Enrichir mk_state() avec champs manquants (canary_input, consent, files dir) |
| `crates/nexus-shell-daemon/src/consent.rs` | Ajouter tests mod avec Router oneshot (4 routes × 2 tests) |
| `crates/nexus-shell-daemon/src/files.rs` | Ajouter tests Router oneshot (3 routes × 2 tests) |
| `crates/nexus-shell-daemon/src/canary_api.rs` | Ajouter tests Router oneshot (3 routes × 2 tests) |
| `crates/nexus-shell-daemon/src/contributor_api.rs` | Ajouter tests Router oneshot (2 routes × 2 tests) |

### §A.3 Tests plan

1. test_get_consent_returns_current_config — happy path GET
2. test_set_consent_updates_config — happy path POST
3. test_whitelist_add_project — happy path
4. test_whitelist_remove_project — happy path
5. test_consent_set_invalid_body_400 — error path
6. test_upload_file_success — happy path multipart
7. test_get_manifest_found — happy path
8. test_get_manifest_not_found_404 — error path
9. test_stream_file_found — happy path
10. test_stream_file_not_found_404 — error path
11. test_canary_freshness_valid_pubkey — happy path
12. test_canary_freshness_invalid_pubkey_400 — error path
13. test_canary_inject_rate_success — happy path
14. test_canary_observed_divergence — happy path
15. test_contributor_project_found — happy path
16. test_contributor_project_not_found — error path
17. test_contributor_envelope_found — happy path
18. test_contributor_envelope_not_found — error path

Minimum +18 tests Rust. Si certains tests sont triviaux ou
redondants avec des tests unitaires existants, documenter le
rationale de non-duplication dans le commit body.

### §A.4 Critere d'acceptation

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked  # >= 1150 tests, 0 fail
cargo test --workspace --locked --doc
```

Les 12 routes ont chacune au moins 1 test Router oneshot dans le
crate nexus-shell-daemon.

### §A.5 Commit cible

```
feat(sprint46): Sprint 46 Phase A — integration tests 12 routes MANDATORY P2-AUDIT-A-1-S43

Enrichissement mk_state() (canary_input, consent, files dir) +
18 tests Router::oneshot() pour les 12 routes originales du
carry MANDATORY : consent (4), files (3), canary_api (3),
contributor_api (2).

Ferme P2-AUDIT-A-1-S43 (3/3 MANDATORY depuis S43).

Delta tests : Rust X→Y (+Z).
Scope cuts respectes : [liste].

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
```

---

## Phase B — Dette pair S44 + integration tests 14 routes recentes

### §B.1 Scope

Phase dette obligatoire (sprint pair §6.2.1 Regle 1). Absorbe
les 5 items S44 a 2/3 + etend la couverture integration tests
aux 14 routes ajoutees S44-S45.

**5 items dette S44** :

1. **P2-REVIEW-A-1-S44 as_str/serde coupling** : decouplage
   entre les methodes `as_str()` et les attributs `#[serde(rename)]`
   dans les enums du daemon. Verifier et corriger tout couplage
   redondant.

2. **P2-REVIEW-B-1-S44 kudos entries pagination** : le handler
   `kudos_api.rs` list entries ne supporte pas limit/offset.
   Ajouter pagination query params.

3. **P3-REVIEW-B-2-S44 shell discover self-only** : le handler
   `shell_api.rs` discover peut retourner le noeud daemon
   lui-meme dans les resultats. Filtrer le self.

4. **P3-AUDIT-A-1-S44 test pagination handler-level** : les
   handlers avec pagination (tasks, kudos) n'ont pas de tests
   verifiant le comportement limit/offset au niveau handler.

5. **P3-AUDIT-B-1-S44 diagnostic silent fallback** : le handler
   `diagnostic_api.rs` peut retourner un vec![] au lieu d'une
   erreur 500 quand le calcul echoue. S'assurer que les erreurs
   sont propagees.

**14 integration tests routes recentes** : invite (3), quarantine
(3), tasks (2), kudos (2), health (1), shell (1), diagnostic (1),
worker_state (1).

### §B.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-shell-daemon/src/kudos_api.rs` | Ajouter limit/offset + tests |
| `crates/nexus-shell-daemon/src/shell_api.rs` | Filtrer self node + tests |
| `crates/nexus-shell-daemon/src/diagnostic_api.rs` | Propager erreurs + tests |
| `crates/nexus-shell-daemon/src/tasks_api.rs` | Tests pagination + integration |
| `crates/nexus-shell-daemon/src/invite_api.rs` | Tests integration 3 routes |
| `crates/nexus-shell-daemon/src/quarantine_api.rs` | Tests integration 3 routes |
| `crates/nexus-shell-daemon/src/health_api.rs` | Test integration 1 route |
| `crates/nexus-shell-daemon/src/worker_state_api.rs` | Test integration 1 route |
| Fichiers enums concernes par as_str/serde | Decouplage |

### §B.3 Tests plan

Integration tests :
1. test_invite_create_success
2. test_invite_list_empty
3. test_invite_revoke_not_found_404
4. test_quarantine_list_empty
5. test_quarantine_flush_not_found
6. test_quarantine_drop_not_found
7. test_tasks_list_default
8. test_tasks_get_not_found_404
9. test_kudos_entries_empty
10. test_kudos_leaderboard_empty
11. test_health_coordinator_ok
12. test_shell_discover_result
13. test_diagnostic_fairness_ok
14. test_worker_state_ok

Tests pagination :
15. test_kudos_entries_with_limit_offset
16. test_tasks_list_with_limit
17. test_shell_discover_self_filtered

Tests dette :
18. test_diagnostic_error_propagation

Minimum +18 tests Rust. Total avec Phase A : +36 tests.

### §B.4 Critere d'acceptation

**Par item dette (G1 shadow D4 ack)** :

1. P2-REVIEW-A-1-S44 as_str/serde : 0 enum avec a la fois
   `#[serde(rename)]` ET `as_str()` dupliquant les memes valeurs.
   Grep `as_str.*match` dans browse.rs/types.rs = 0 match.
2. P2-REVIEW-B-1-S44 kudos pagination : `GET /api/v1/kudos/entries`
   accepte `?limit=N&offset=M`, cap limit <= 500, test verifie.
3. P3-REVIEW-B-2-S44 discover self-only : test integration verifie
   `count: 1` et contient uniquement self node_id. Code deja
   corrige post-S45 — seul le test manque.
4. P3-AUDIT-A-1-S44 pagination tests : au moins 1 test handler-level
   par handler pagine (tasks, kudos, canary observed-divergence)
   verifiant skip(offset) + take(limit).
5. P3-AUDIT-B-1-S44 diagnostic fallback : 0 `unwrap_or_default()`
   dans diagnostic_api.rs sur les appels DB. Les erreurs propagent
   → 500.

**Global** :

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked  # >= 1168 tests, 0 fail
cargo test --workspace --locked --doc
```

Les 5 items dette S44 sont resolus (criteres ci-dessus). Les 14
routes recentes ont integration tests.

### §B.5 Commit cible

```
feat(sprint46): Sprint 46 Phase B — dette pair S44 5 items + integration tests 14 routes recentes

Phase dette obligatoire (sprint pair §6.2.1).
Items resolus :
- P2-REVIEW-A-1-S44 as_str/serde coupling (2/3 → CLOSED)
- P2-REVIEW-B-1-S44 kudos entries pagination (2/3 → CLOSED)
- P3-REVIEW-B-2-S44 shell discover self-only (2/3 → CLOSED)
- P3-AUDIT-A-1-S44 test pagination handler-level (2/3 → CLOSED)
- P3-AUDIT-B-1-S44 diagnostic silent fallback (2/3 → CLOSED)
+ 14 integration tests Router::oneshot() routes recentes.

Delta tests : Rust X→Y (+Z).
Scope cuts respectes : [liste].

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
```

---

## Phase C — Frontend direct-daemon migration

### §C.1 Scope

Migrer le frontend React pour appeler le daemon directement au
lieu de passer par le coordinator Python. Le hotfix `1f1a017`
fait deja servir le shell par le daemon et fournit GET /auth/token.

Changements :
1. **coordinator.ts** : mettre a jour les paths pour utiliser les
   paths daemon (ex: `/tasks` → `/api/v1/tasks`,
   `/kudos` → `/api/v1/kudos/{project_id}`, etc.). Renommer les
   classes d'erreur Coordinator* → ApiProtocolError, ApiHttpError.
2. **daemon.ts** : supprimer le `callProxy()` envelope handling
   (le frontend appelle le daemon directement, plus de proxy
   `{ kind, status, body }`). Adapter les fonctions pour utiliser
   le meme pattern getJson/postJson que coordinator.ts.
3. **Callers** : les composants qui passent un coordinatorUrl
   passent maintenant le daemon URL (same-origin par defaut).
   Le store ou contexte qui fournit l'URL est adapte.
4. **Routes app-specific** : les routes `/app/{name}/commands`,
   `/app/{name}/state`, `/app/{name}/events` qui n'existent que
   sur le coordinator Python sont gardees avec un fallback
   coordinatorUrl explicite. useAppEvents.ts reste pointe vers
   le coordinator.
5. **Tests Vitest** : adapter les mocks et paths dans les tests
   existants.

### §C.2 Fichiers touches

| Fichier | Role |
|---|---|
| `web/src/api/coordinator.ts` | Update paths /api/v1/*, rename error classes |
| `web/src/api/daemon.ts` | Drop proxy envelope, use direct calls |
| `web/src/api/auth.ts` | Verifier coherence (deja adapte par hotfix) |
| `web/src/stores/projectStore.ts` | Adapter store si coordinatorUrl change |
| `web/src/components/AddCoordinatorDialog.tsx` | Adapter si renommage |
| `web/src/components/GpuConsentDialog.tsx` | Update coordinatorUrl prop |
| `web/src/components/AppShell.tsx` | Update import/usage |
| `web/src/bridge/useBridge.ts` | Update import |
| `web/src/components/command-palette/CommandPalette.tsx` | Update import |
| `web/src/components/project/InvitesTab.tsx` | Update import/usage |
| `web/src/components/project/AppsTab.tsx` | Update import/usage |
| `web/src/components/project/TasksTab.tsx` | Update import |
| `web/src/components/PanicWipeKeybind.tsx` | Update import/usage |
| `web/src/hooks/useAppEvents.ts` | Garder coordinatorUrl pour SSE |
| `web/src/api/__tests__/coordinator.test.ts` | Adapter paths + noms |
| `web/src/api/__tests__/daemon.test.ts` | Adapter ou recrire |

### §C.3 Tests plan

1. test_api_calls_daemon_directly — verifie paths /api/v1/*
2. test_auth_fallback_same_origin — existant, verifier inchange
3. Adapter tests Vitest existants (coordinator.test.ts,
   daemon.test.ts, CommandPalette.test.ts)
4. test_app_routes_still_use_coordinator — verifie que les
   routes app runtime gardent le coordinator URL

### §C.4 Critere d'acceptation

```bash
cd web && npm run lint && \
  npx tsc --noEmit -p tsconfig.app.json && \
  npm run test:unit && npm run build && npm run size
```

Le frontend compile, les tests passent, le build est sous les
limites size-limit. Le shell charge correctement depuis le daemon.

### §C.5 Commit cible

```
feat(sprint46): Sprint 46 Phase C — frontend direct-daemon migration coordinator→daemon

Le frontend appelle le daemon directement (same-origin) au lieu
de passer par le coordinator Python proxy. Paths mis a jour
/api/v1/*, proxy envelope supprime, error classes renommees.
Routes app-specific (commands, state, events SSE) restent
pointees vers le coordinator Python.

Leveraging hotfix 1f1a017 (daemon --web-root + GET /auth/token).

Delta tests : Vitest X→Y.
Scope cuts respectes : [liste].

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
```

---

## Phase D — Wrap-up

Verification.md + audit_plan S47 + mise a jour CLAUDE.md,
SPRINT_LOG.md, HARDENING_ROADMAP.md (last_validated), PATTERNS.md
si applicable. Memory update nexus_grid_pivot.md + MEMORY.md.

---

## §5 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1168, 0 fail | |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | 6+ passed | |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | |
| 6 | ruff format | `uv run ruff format --check packages/` | 0 diff | |
| 7 | ruff check | `uv run ruff check packages/` | 0 error | |
| 8 | SDK pytest | `uv run pytest packages/nexus-sdk/tests/ -q` | 195 | |
| 9 | Coord pytest | `uv run pytest packages/nexus-coordinator/tests/ -q` | 323+23f+6s | |
| 10 | Gov pytest | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 | |
| 11 | npm lint | `npm run lint` (web/) | 0 error | |
| 12 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error | |
| 13 | Vitest | `npm run test:unit` (web/) | >= 268 | |
| 14 | build | `npm run build` (web/) | ok | |
| 15 | size-limit | `npm run size` (web/) | 7/7 | |
| 16 | Phase A preflight G8 | EXECUTE | | |
| 17 | Phase A review | PASS | | |
| 18 | Phase B preflight G8 | EXECUTE | | |
| 19 | Phase B review | PASS | | |
| 20 | Phase C preflight G8 | EXECUTE | | |
| 21 | Phase C review | PASS | | |
| 22 | 12 routes MANDATORY integration tests | 12/12 | | |
| 23 | 14 routes recentes integration tests | 14/14 | | |
| 24 | 5 items dette S44 resolus | 5/5 | | |
| 25 | Frontend direct-daemon compile | ok | | |
| 26 | Scope cuts respectes | 13/13 | | |
| 27 | Delta tests documente | cumule | | |
| 28 | CLAUDE.md, SPRINT_LOG.md a jour | ok | | |

## §6 Git plan

```
1. chore(planning): sprint 46 kickoff + plan + design review + migration S45 archive
2. chore(planning): sprint 46 Phase A preflight G8
3. chore(planning): sprint 46 Phase A review
4. feat(sprint46): Sprint 46 Phase A — integration tests 12 routes MANDATORY
5. chore(planning): sprint 46 Phase B preflight G8
6. chore(planning): sprint 46 Phase B review
7. feat(sprint46): Sprint 46 Phase B — dette pair S44 + integration tests 14 routes
8. chore(planning): sprint 46 Phase C preflight G8
9. chore(planning): sprint 46 Phase C review
10. feat(sprint46): Sprint 46 Phase C — frontend direct-daemon migration
11. chore(sprint46): Phase D — wrap-up + verification + audit plan S47 + counters
```

## §7 Scope cuts

1. events.py SSE streaming — S47+
2. App runtime migration Rust — S47+
3. MCP server migration Rust — S47+
4. PyO3 bindings removal — S47+
5. Suppression complete coordinator Python — S47+
6. CI/VPS/v1.0 — S48+
7. Kudos debit/stake — interdit (Day 0 #7)
8. Integration tests deploy.rs + apps.rs — S47
9. Integration test auth/token — S47
10. invite ID collision UUID fix — S47 (1/3→2/3)
11. diagnostic Err path test — S47 (1/3→2/3)
12. modules Python suppression differee — S47 (1/3→2/3)
13. demos/babel-library cleanup — hors-sprint

## §8 Risks

| # | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | mk_state() cascade enrichissement | Medium | Medium | Incremental, un champ a la fois |
| R2 | files.rs tests tmpdir async | Low | Medium | Pattern tokio tempdir existant |
| R3 | Frontend paths mismatch | Medium | Low | Grep exhaustif daemon routes vs coordinator paths |
| R4 | canary_input None dans harness | Medium | Low | Enrichir avec test fixture |
| R5 | Proxy envelope removal casse callers | Low | Medium | Tests Vitest valident |
| R6 | Renommage 50+ imports | Low | Medium | Batch, tsc catch erreurs |

## §9 Checkpoint de cloture

1. 12/12 routes MANDATORY avec integration tests
2. 14/14 routes recentes avec integration tests
3. 5/5 items dette S44 resolus
4. Frontend direct-daemon fonctionnel
5. 28/28 fail-fast checklist
6. 11 commits dans le git plan
7. CLAUDE.md + SPRINT_LOG.md + memory a jour
8. sprint47_audit_plan.md ecrit
