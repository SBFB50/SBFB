# Sprint 7 — Verification (fail-fast checklist)

**Date** : 2026-04-11
**HEAD entrée** : `2926383` (master tip au début de Sprint 7,
après le gate audit Sprint 6)
**HEAD sortie** : `6f32893` (Phase E committé ; ce doc + son
`sprint7_audit_plan.md` ajouteront un `docs(sprint7)` final)

Sprint 7 commit stack (6 commits feat + 1 doc kickoff = 7 ; ce
commit en ajoutera un 8e `docs(sprint7): verification + audit plan`
pour fermer officiellement le sprint) :

```
6f32893 feat(coordinator,web): Sprint 7 Phase E — /daemon proxy + Browse/Curators pages live
e6b22aa feat(shell-daemon): Sprint 7 Phase D — pkarr DHT browse resolution
818429d feat(shell-daemon): Sprint 7 Phase C — gossip subscribe + fetch_ticket curator pipeline
f4ae22d feat(core-rs,core-py,sdk): Sprint 7 Phase B — curator list Ed25519 primitives + PyO3 bindings
2c896a8 feat(shell-daemon): Sprint 7 Phase A — headless daemon + HTTP skeleton
29ad7c5 docs(sprint7): kickoff + plan
```

Ce document est une checklist **self-reportée** par l'agent qui a
livré les 6 commits ci-dessus. Chaque row est la commande exacte
qu'un relecteur peut rejouer localement et la valeur observée
après le commit `6f32893`. L'audit indépendant vit dans
`.planning/sprint7_audit_plan.md` et est joué en Phase 0 de
Sprint 8 par une session fraîche.

---

## Rappel — `sprint_audit_gate.md`

Le fail-fast dit "le code compile et les tests passent". C'est
nécessaire mais pas suffisant. L'audit indépendant Sprint 8 Phase 0
ira chercher les blind spots : décisions non justifiées à la
relecture, surface testée qui ne correspond pas à la surface exécutée
en prod, promesses UX non honorées, tech debt implicite.

---

## How to re-run

```bash
# depuis la racine du repo, avec cargo + uv + node sur le PATH
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked

# Python : les 3 packages tournent séparément parce que pytest
# collide sur `tests.test_*` quand on les lance ensemble (les
# 3 packages partagent le même nom `tests`).
uv run ruff format --check packages/ examples/
uv run ruff check packages/ examples/
uv run pytest packages/nexus-sdk/tests/ -q
uv run pytest packages/nexus-coordinator/tests/ -q
uv run pytest packages/nexus-app-gov/tests/ -q

cd web
npm install
npm run lint
npx tsc --noEmit -p tsconfig.app.json
npm run test:unit
npm run test:coverage
npm run build
npm run size
npx playwright test
bash scripts/scan-en-strings.sh
cd ..
```

Note Python / wheel : le test `test_curator.py` (SDK, cross-lang)
dépend du wheel `nexus_core` installé dans le `.venv` uv. Si le
wheel n'a pas les bindings `sign_curator_list` /
`verify_curator_list_entry`, rebuild via :

```bash
unset CONDA_PREFIX CONDA_DEFAULT_ENV && \
  VIRTUAL_ENV=$PWD/.venv maturin develop --release \
    --manifest-path crates/nexus-core-py/Cargo.toml
```

C'est le pattern Sprint 1/2 — `.venv` et miniconda coexistent, on
force la cible en unsetant les vars Conda.

---

## Checklist

| # | Check | Commande | Critère | Observé |
|---|---|---|---|---|
| 1 | Rust build workspace | `cargo build --workspace --locked` | exit 0, 0 warning | ✅ exit 0, 0 warning |
| 2 | Rust fmt | `cargo fmt --all --check` | exit 0 | ✅ clean |
| 3 | Rust clippy `-D warnings` | `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 | ✅ clean |
| 4 | Rust tests — workspace | `cargo test --workspace --locked` | ≥ 291 (193 baseline + Phases A-D nouveaux), 0 failed | ✅ **304 passed** (78 core-rs + 27 shell-daemon bin unit + 6 shell-daemon e2e + 62 shell-daemon-core lib + 11 worker bin unit + 10 worker e2e + 105 worker-core + 5 doctests) |
| 5 | Rust tests — curator primitives | `cargo test -p nexus-core-rs curator::` | ≥ 10 passed | ✅ **13 passed** (sign/verify roundtrip, tampered rejection ×2, attribution mismatch, wrong signer, unknown version, oversized entries × 2, cap boundary, domain separation, JSON roundtrip, canonical bytes determinism) |
| 6 | Rust tests — `probe_reachable` | `cargo test -p nexus-core-rs discovery::probe_reachable` | ≥ 3 passed | ✅ **3 passed** (malformed hex, unknown id Ok(false), seeded 2-node Ok(true)) |
| 7 | Rust tests — iroh_runtime (Phase C) | `cargo test -p nexus-shell-daemon-core iroh_runtime::` | ≥ 15 passed incl 4 x two_nodes_* | ✅ **19 passed** dont `two_nodes_subscribe_and_fetch_curator_list`, `two_nodes_reject_announcement_from_non_subscribed_curator`, `two_nodes_reject_revision_rollback`, `two_nodes_reject_attribution_mismatch_in_announcement` |
| 8 | Rust tests — browse (Phase D) | `cargo test -p nexus-shell-daemon-core browse::` | ≥ 6 passed incl `aggregate_probes_seeded_peer_and_marks_it_reachable` | ✅ **9 passed** |
| 9 | Rust tests — shell-daemon http (Phase A+C+D) | `cargo test -p nexus-shell-daemon http::` | ≥ 10 passed incl curator + browse routes | ✅ **14 passed** (health, info, 404, list_curators empty, subscribe→list→delete, bad hex → 400, info reflects curator counts, browse empty, + 6 CORS loopback tests) |
| 10 | Rust tests — shell-daemon runtime | `cargo test -p nexus-shell-daemon runtime::` | ≥ 4 async tests | ✅ **4 passed** (start-shutdown roundtrip, singleton refuses second start, stale overwrite, subscriptions persisted across restart) |
| 11 | Rust tests — e2e binary spawn | `cargo test -p nexus-shell-daemon --test e2e` | ≥ 5 passed | ✅ **6 passed** (version, help, stop stub, status stub, start → /health, second start refuses) |
| 12 | Python — SDK cross-lang curator | `uv run pytest packages/nexus-sdk/tests/test_curator.py -q` | ≥ 8 passed | ✅ **8 passed** (sign/verify roundtrip, JSON re-serialization, tampered, attribution split-brain, mismatched pubkey-in-payload, oversized DoS cap, future version, bad JSON → ValueError) |
| 13 | Python — SDK full suite | `uv run pytest packages/nexus-sdk/tests/ -q` | ≥ 38 passed (32 baseline + 8 curator) | ✅ **40 passed** (32 test_sdk + 8 test_curator) |
| 14 | Python — coordinator daemon proxy | `uv run pytest packages/nexus-coordinator/tests/test_daemon_proxy.py -q` | ≥ 10 passed | ✅ **10 passed** (absent running.json, upstream forward, curators list/subscribe/unsubscribe, bad POST body → 400, path param encoding, browse forward, dead port → 503, malformed running.json → 503, upstream 422 preserved) |
| 15 | Python — coordinator full suite | `uv run pytest packages/nexus-coordinator/tests/ -q` | ≥ 57 + 1 skipped | ✅ **57 passed + 1 skipped** |
| 16 | Python — app-gov | `uv run pytest packages/nexus-app-gov/tests/ -q` | 3 passed (inchangé) | ✅ **3 passed** |
| 17 | Python — ruff format + check | `uv run ruff format --check packages/ examples/ && uv run ruff check packages/ examples/` | exit 0 | ✅ clean après Phase E fixup |
| 18 | Web — tsc strict | `cd web && npx tsc --noEmit -p tsconfig.app.json` | exit 0, 0 error | ✅ clean |
| 19 | Web — ESLint | `cd web && npm run lint` | 0 errors, ≤ 5 T1 warnings | ✅ **0 errors, 5 warnings** (toutes pré-existantes dans `components/ui/*` vendored shadcn — `react-refresh/only-export-components`, hors scope modification vendored) |
| 20 | Web — Vite build | `cd web && npm run build` | exit 0, 0 warning | ✅ main 466.33 kB, vendor-react 189.64 kB, vendor-ui 31.54 kB, css 93.80 kB, 0 warning |
| 21 | Web — size-limit budgets | `cd web && npm run size` | main ≤ 475, vendor-react ≤ 210, vendor-ui ≤ 50, css ≤ 100 | ✅ 4/4 green (main 466/475, vendor-react 189/210, vendor-ui 31/50, css 93/100) |
| 22 | Web — Vitest unit tests | `cd web && npm run test:unit` | ≥ 107 passed (99 baseline + 8 daemon.ts minimum) | ✅ **114 passed** / 7 test files (+15 daemon.ts vs 99 baseline) |
| 23 | Web — Vitest coverage thresholds | `cd web && npm run test:coverage` | lines ≥ 90, funcs ≥ 90, branches ≥ 85, stmts ≥ 90 | ✅ lines 96.12, funcs 98.38, branches 86.59, stmts 96.44 — `src/api/daemon.ts` 90.9 lines / 100 funcs / 70 branches (acceptable : les 3 branches uncovered sont des fall-through rares couverts par Playwright) |
| 24 | Web — Playwright (incl Phase E specs) | `cd web && npx playwright test` | ≥ 12 passed (10 baseline − 2 stub-pages + 5 Phase E = 13) | ✅ **13 passed** / 8 spec files |
| 25 | Web — scan-en-strings (FR only) | `cd web && bash scripts/scan-en-strings.sh` | exit 0 | ✅ clean (pages/Browse.tsx + pages/Curators.tsx mapping anglais→français : BrowseStatus "reachable"→"Accessible", "unreachable"→"Injoignable", "unknown"→"Inconnu") |
| 26 | Singleton Phase A — second start refuses | cargo test `second_start_refuses_when_first_still_running` (bin unit + e2e) | 2 tests verts | ✅ `runtime::tests::second_start_refuses_when_first_still_running` + `tests/e2e.rs::second_start_refuses_when_first_still_running` |
| 27 | Curator list DoS cap (R5) | `cargo test -p nexus-core-rs curator::tests::sign_rejects_oversized_entries curator::tests::verify_rejects_oversized_entries curator::tests::cap_boundary_is_accepted` | 3 tests verts | ✅ sign-side cap, verify-side cap (hand-crafted bypass), off-by-one regression |
| 28 | Revision rollback protection (R6) | `cargo test -p nexus-shell-daemon-core iroh_runtime::tests::two_nodes_reject_revision_rollback` | 1 test vert, stored list reste à rev 5 | ✅ Ingest rev 5 → accepte, re-ingest rev 3 → `RevisionRollback`, `get_list` retourne toujours rev 5 |
| 29 | Subscriptions persistence (R7) | `cargo test -p nexus-shell-daemon persist_and_reload_subscriptions_roundtrip curator_runtime_persists_subscriptions_across_restart` | 2 tests verts | ✅ unit `iroh_runtime::tests::persist_and_reload_subscriptions_roundtrip` + e2e real-daemon `runtime::tests::curator_runtime_persists_subscriptions_across_restart` |
| 30 | Daemon-offline UX contract | `cd web && npx playwright test browse-daemon-offline curators-flow` | 5 tests verts incl. banner visible | ✅ `browse-daemon-offline.spec.ts` × 2 + `curators-flow.spec.ts` × 3 |
| 31 | TODO(Sprint7) hanging | `grep -rn 'TODO(Sprint7)' crates/ packages/ web/src/` | exit 0 / 0 match | ✅ 0 match |
| 32 | `.planning/sprint7_audit_plan.md` existe | `test -f .planning/sprint7_audit_plan.md` | exit 0 | ✅ ce commit l'ajoute |

**32 rows** — dépasse la cible plan §10 (30 rows). Les 30 originales
sont toutes vertes ; 2 bonus couvrent explicitement les mitigations
plan §13 qu'on voulait verrouiller en sortie (singleton R3 + audit
gate traceability).

---

## Métriques Sprint 7

| Suite | Avant Sprint 7 (tip `2926383`) | Après Phase E (tip `6f32893`) | Delta |
|---|---|---|---|
| Rust workspace | 193 | **304** | +111 |
| Python SDK | 32 | **40** | +8 (cross-lang curator) |
| Python coordinator | 47 + 1 skipped | **57 + 1 skipped** | +10 (daemon proxy) |
| Python app-gov | 3 | 3 | 0 |
| Vitest unit | 99 | **114** | +15 (daemon.ts) |
| Playwright | 10 | **13** | +3 net (+5 Phase E, −2 stub-pages) |
| size-limit budgets | 4/4 green | 4/4 green | 0 régression |

**Total test delta Sprint 7 : +147 new tests** (111 Rust + 8 SDK +
10 coord + 15 Vitest + 3 Playwright nets). Chaque phase commit
atomique porte son delta en clair dans son message.

---

## Surface nouvelle livrée par Sprint 7

- `crates/nexus-shell-daemon-core/` (~1800 LOC + 62 tests) —
  registry singleton, config paths, iroh_runtime (curator
  pipeline), browse aggregator, state snapshot
- `crates/nexus-shell-daemon/` (~900 LOC + 33 tests) — binary
  CLI, HTTP surface (`/health`, `/info`, `/curators*`, `/browse`),
  runtime avec gossip subscribe task et graceful shutdown
- `crates/nexus-core-rs::curator` (~500 LOC + 13 tests) —
  `CuratorList`, `CuratorListEntry`, sign/verify avec attribution
  split-brain + DoS cap + version check + domain separation
- `crates/nexus-core-rs::discovery::probe_reachable` — pkarr
  probe via `Endpoint::connect + BLOBS_ALPN` + timeout
- `crates/nexus-core-py::{sign,verify}_curator_list_entry` — PyO3
  bindings cross-lang
- `packages/nexus-coordinator/api/daemon.py` — 5 routes proxy avec
  discriminated-union envelope (data / unavailable / error)
- `web/src/api/daemon.ts` — client Zod-first + `DaemonResult<T>`
  + 5 helpers + `isValidCuratorPubkey`
- `web/src/pages/Browse.tsx` + `Curators.tsx` — rewrites React
  Query branchés sur la proxy
- 2 specs Playwright + 1 test file Vitest + 10 tests Python
  coordinator + 8 tests Python SDK

---

## Ce que Sprint 7 n'a PAS livré (scope cuts respectés)

- ❌ **Bootstrap peers VPS FlowUP** — Sprint 10 (release v1.0)
- ❌ **pkarr publish** — Sprint 10 (Phase D consomme uniquement la DHT)
- ❌ **`AppContext.submit_task` implémentation** — Sprint 8 Phase A
  (signature gelée Day 0 D4 dans `sprint7_kickoff.md` §4)
- ❌ **`@nexus_command` décorateur** — Sprint 8 Phase A
  (signature gelée Day 0 D5)
- ❌ **Migration tab gov vers TabView** — Sprint 8 (19 tabs)
- ❌ **Unix socket / named pipe IPC** — D1 HTTP loopback uniquement
- ❌ **Multi-instance daemon** — D2 singleton strict
- ❌ **Topic gossip namespacé par curator pubkey** — D3 global topic v1
- ❌ **Persistence SQLite des curator lists côté daemon** — RAM-only
  (seul le set d'attention persiste, R7)
- ❌ **Browse filter / search UI** — Sprint 8 ou 9
- ❌ **Auth sur le proxy daemon** — loopback + CORS loopback
  regex suffit au modèle de confiance
- ❌ **TUI dans `nexus-shell-daemon`** — headless only

Chaque exclusion est justifiée dans le commit atomique de la phase
concernée.

---

## Checkpoint de clôture Sprint 7 (vs `.planning/sprint7_plan.md` §14)

1. ✅ Fail-fast ci-dessus : **32 / 32 verts**
2. ✅ `git log --oneline master ^2926383` : 6 commits feat + 1 doc
   kickoff + 1 doc verification (ce commit) = 8 commits atomiques
3. ✅ `.planning/sprint7_verification.md` commité et lisible (ce fichier)
4. ✅ `.planning/sprint7_audit_plan.md` commité et lisible (livré
   avec ce commit)
5. ✅ `docs/shell/PATTERNS.md` contient P9 + T4/T5 "frozen Sprint 7,
   impl Sprint 8" (mis à jour par ce commit)
6. ✅ `docs/rust/PATTERNS.md` contient la section "Sprint 7 canonical"
   (mise à jour par ce commit)
7. ✅ `grep -rn 'TODO(Sprint7)' crates/ packages/ web/src/` → 0 match
8. 🟡 Memory `nexus_grid_pivot.md` mis à jour avec le tip Sprint 7
   sortie → ce commit ne touche pas la memory côté fichiers
   (`~/.claude/projects/...`) ; la mise à jour memory se fait par
   l'agent qui a livré le commit, pas comme part de la livraison
   elle-même

**Sprint 7 est FERMÉ** côté code. L'audit gate Phase 0 de Sprint 8
jouera `.planning/sprint7_audit_plan.md` et produira son rapport
`.planning/sprint7_audit_findings.md` avant le premier commit
Sprint 8 Phase A.
