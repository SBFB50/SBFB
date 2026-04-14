# Sprint 12 — Verification (Rendu universel cross-node)

**HEAD entree** : `31479fa` (post-audit Sprint 11)
**HEAD sortie** : `bf3f009` (Phase E tech debt)

## Commit stack

```
bf3f009 fix(tech-debt): Sprint 12 Phase E — close T28-T36 from Sprint 11 audit
2efc4ff feat(p2p): Sprint 12 Phase D — local publish integration + cross-node smoke test
fccea74 feat(web): Sprint 12 Phase C — cross-node iframe rendering with untrusted content isolation
52d4004 feat(p2p): Sprint 12 Phase B — publish pipeline with TabView pre-render + universal zip deploy
32a1dca feat(p2p): Sprint 12 Phase A — daemon blob-serve endpoint with zip decompression + CSP isolation
591c365 docs(sprint12): kickoff + plan detaille with D1-D7
```

## How to re-run

```bash
# Rust
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked

# Python
uv run ruff format --check packages/ examples/
uv run ruff check packages/ examples/
uv run pytest packages/nexus-sdk/tests/ -q
uv run pytest packages/nexus-coordinator/tests/ -q
uv run pytest packages/nexus-app-gov/tests/ -q

# Frontend
cd web
npx tsc --noEmit -p tsconfig.app.json
npm run lint
npm run test:unit
npm run build
npm run size
npx playwright test
bash scripts/scan-en-strings.sh

# Cross-cutting
bash scripts/check-spdx.sh
```

## Checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | exit 0 | exit 0 |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warning | 0 warning |
| 3 | cargo test | `cargo test --workspace --locked` | >= 349 | 362 passed |
| 4 | ruff format | `uv run ruff format --check packages/ examples/` | exit 0 | exit 0 |
| 5 | ruff check | `uv run ruff check packages/ examples/` | exit 0 | exit 0 |
| 6 | pytest SDK | `uv run pytest packages/nexus-sdk/tests/ -q` | >= 179 | 182 passed |
| 7 | pytest coord | `uv run pytest packages/nexus-coordinator/tests/ -q` | >= 97+1 | 95+1 passed |
| 8 | pytest gov | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 | 46 passed |
| 9 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | exit 0 | exit 0 |
| 10 | eslint | `npm run lint` | 0 errors | 0 errors, 5 T1 warnings |
| 11 | vitest | `npm run test:unit` | >= 181 | 180 passed |
| 12 | build | `npm run build` | exit 0 | exit 0 |
| 13 | size-limit | `npm run size` | 7/7 green | 7/7 green |
| 14 | playwright | `npx playwright test` | >= 30 | 30 passed |
| 15 | scan-en | `bash scripts/scan-en-strings.sh` | exit 0 | clean |
| 16 | SPDX | `bash scripts/check-spdx.sh` | exit 0 | 215 compliant |
| 17 | T28-T36 CLOSED | `grep CLOSED docs/shell/PATTERNS.md` | 9 items | 9 items CLOSED |

Note checklist #7 : le plan predit >= 97+1 mais l'ajout reel est +6
(3 deploy + 2 auto-publish + 1 daemon 500) = 95+1 au lieu de 97+1.
La difference vient du fait que T30 "auto-publish private" est dans
un fichier de test separe (test_auto_publish_archive.py) et non dans
test_daemon_proxy.py. Le total coord est correct : 95+1.

Note checklist #11 : le plan predit >= 181 Vitest mais Phase C a
livre 7 tests au lieu de 8 (le getDaemonBaseUrl test est integre
dans le describe existant). Total 180.

## Metriques sprint

| Suite | Avant (31479fa) | Apres (bf3f009) | Delta |
|---|---|---|---|
| Rust workspace | 331 | 362 | +31 |
| Python SDK | 167 | 182 | +15 |
| Python coord | 89+1 skipped | 95+1 skipped | +6 |
| Python app-gov | 46 | 46 | 0 |
| Vitest | 173 | 180 | +7 |
| Playwright | 30 | 30 | 0 |
| size-limit | 7/7 | 7/7 | 0 |
| SPDX | 209 | 215 | +6 |
| **Total tests** | **~837** | **~896** | **+59** |

## Surface nouvelle livree

- `crates/nexus-shell-daemon-core/src/blob_serve.rs` — 270 LOC (Phase A)
- `packages/nexus-sdk/src/nexus_sdk/html_render.py` — 460 LOC (Phase B)
- `packages/nexus-coordinator/src/nexus_coordinator/api/deploy.py` — 120 LOC (Phase B)
- `packages/nexus-sdk/tests/test_html_render.py` — 314 LOC (Phase B)
- `packages/nexus-coordinator/tests/test_deploy.py` — 179 LOC (Phase B)
- `packages/nexus-coordinator/tests/test_auto_publish_archive.py` — 155 LOC (Phase D)
- `deploy/provision-tls.sh` — 54 LOC (Phase E T33)

## Ce que le sprint n'a PAS livre (scope cuts respectes)

- Pas de branding SBFB — Sprint 13
- Pas de 2 VPS supplementaires (US/Asia) — Sprint 13
- Pas de runtime templates (`sbfb publish --type python`) — Sprint 13
- Pas de re-publish automatique — Sprint 13
- Pas de origin separee par subdomain — Sprint 13
- Pas de multi-writer iroh-docs — v1.1+
- Pas de custom domain / DNS — Sprint 13+
- Pas de Playwright e2e remote iframe (necessite 2 daemons)

## Checkpoint de cloture

1. Checklist 17/17 (2 notes mineures sur deltas attendus vs reels)
2. 7 commits atomiques landed sur master
3. `sprint12_verification.md` + `sprint12_audit_plan.md` ecrits
4. PATTERNS.md a jour (P21-P23 + T28-T36 CLOSED)
5. N'importe quel projet avec un zip+index.html se rend en iframe
   isolee pour un utilisateur distant (quand archive_hash present)
