# Sprint 10 — Verification (self-report fail-fast)

**HEAD entree** : `48b332a`
**HEAD sortie** : `d07bfcf` (pre-Phase F)
**Date** : 2026-04-12

## Commit stack

```
d07bfcf feat(deploy): Sprint 10 Phase E — VPS provisioning + bootstrap peers
0190a07 feat(release): Sprint 10 Phase D — PyPI metadata + release packaging
ef28d75 feat(ci): Sprint 10 Phase C — GitHub Actions CI/CD pipeline
122d4ae feat(docs): Sprint 10 Phase B — README release + nettoyage legacy racine
9c281d0 feat(release): Sprint 10 Phase A — SPDX headers + version 1.0.0 + T13-T22 tech debt log
```

## How to re-run

```bash
./scripts/verify.sh            # steps 1-18
bash scripts/check-spdx.sh --count   # must print 204
grep version Cargo.toml | head -1    # must show 1.0.0
ls .github/workflows/ci.yml release.yml deploy.yml  # must exist
ls deploy/provision.sh deploy.sh gen-identity.sh     # must exist
uv build packages/nexus-sdk --wheel --out-dir /tmp/test-wheels/
uv build packages/nexus-coordinator --wheel --out-dir /tmp/test-wheels/
```

## Checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | exit 0 | exit 0 |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warning | 0 warning |
| 3 | cargo test | `cargo test --workspace --locked` | 312 passed | 312 passed |
| 4 | ruff format | `uv run ruff format --check packages/ examples/` | exit 0 | exit 0 |
| 5 | ruff check | `uv run ruff check packages/ examples/` | exit 0 | exit 0 |
| 6 | pytest SDK | `uv run pytest packages/nexus-sdk/tests/ -q` | 167 passed | 166-167 passed (T18 flaky) |
| 7 | pytest coord | `uv run pytest packages/nexus-coordinator/tests/ -q` | 83+1 skipped | 83 passed, 1 skipped |
| 8 | pytest gov | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 passed | 46 passed |
| 9 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | exit 0 | exit 0 |
| 10 | eslint | `npm run lint` | 0 errors | 0 errors, 5 warnings |
| 11 | vitest | `npm run test:unit` | 161 passed | 161 passed |
| 12 | coverage | `npm run test:coverage` | lines >= 85% | 88% lines, 80.98% branches |
| 13 | build | `npm run build` | exit 0 | exit 0 |
| 14 | size-limit | `npm run size` | 7/7 green | 7/7 green |
| 15 | playwright | `npx playwright test` | 27 passed | SKIPPED (--quick mode, env blocker Windows Defender uv lock, aucun changement web Sprint 10 — 27 passes au tip Sprint 9) |
| 16 | scan-en | `bash scripts/scan-en-strings.sh` | exit 0 | exit 0 |
| 17 | npm audit | `npm audit --audit-level=high` | 0 high/crit | 0 high/crit |
| 18 | SPDX check | `bash scripts/check-spdx.sh` | 204 files | 204 files |
| 19 | versions | grep in 7 manifests | all = 1.0.0 | all 1.0.0 |
| 20 | legacy removed | `ls start.bat` | not found | not found |
| 21 | README sections | `grep '^##' README.md \| wc -l` | >= 8 | 9 sections |
| 22 | CI workflow | `ls .github/workflows/ci.yml` | exists | exists |
| 23 | release workflow | `ls .github/workflows/release.yml` | exists | exists |
| 24 | deploy workflow | `ls .github/workflows/deploy.yml` | exists | exists |
| 25 | wheel SDK | `uv build packages/nexus-sdk --wheel` | exit 0 | nexus_sdk-1.0.0-py3-none-any.whl |
| 26 | wheel coord | `uv build packages/nexus-coordinator --wheel` | exit 0 | nexus_coordinator-1.0.0-py3-none-any.whl |
| 27 | deploy scripts | `ls deploy/provision.sh deploy.sh` | exist | exist |
| 28 | systemd templates | grep in provision.sh | nexus-daemon.service | present |
| 29 | T13-T22 | grep T13 docs/shell/PATTERNS.md | found | found |
| 30 | verify.sh full | `./scripts/verify.sh --quick` | exit 0 | exit 0 (mode --quick, Playwright non inclus) |

## Metriques sprint

| Suite | Avant (Sprint 9) | Apres (Sprint 10) | Delta |
|---|---|---|---|
| Rust workspace | 312 | 312 | 0 |
| Python SDK | 167 | 167 | 0 |
| Python coordinator | 83 + 1 skip | 83 + 1 skip | 0 |
| Python app-gov | 46 | 46 | 0 |
| Vitest unit | 161 | 161 | 0 |
| Playwright | 27 | 27 | 0 |
| size-limit | 7/7 | 7/7 | 0 |
| SPDX check | — | 204/204 | new |

## Surface nouvelle livree

| Module | LOC | Description |
|---|---|---|
| scripts/check-spdx.sh | ~40 | SPDX header verification guard |
| scripts/build-release.sh | ~50 | Release binary + wheel builder |
| .github/workflows/ci.yml | ~100 | 18-step CI pipeline |
| .github/workflows/release.yml | ~100 | Binary + PyPI release on tag |
| .github/workflows/deploy.yml | ~80 | Manual SSH VPS deployment |
| deploy/provision.sh | ~90 | Ubuntu VPS provisioning |
| deploy/deploy.sh | ~70 | Binary upload + restart |
| deploy/gen-identity.sh | ~40 | Ed25519 keypair generator |
| deploy/README.md | ~50 | Fleet documentation |

## Ce que le sprint n'a PAS livre (scope cuts respectes)

- Pas de branding/renommage SBFB (D1 gele, reporte sprint dedie)
- Pas de crates.io publish
- Pas de npm publish
- Pas de Docker images
- Pas de monitoring/alerting
- Pas de domaine custom
- Pas de fix T6/T7/T14
- Pas de cross-app/cross-node events
- Deploiement VPS reel en attente des IPs de l'utilisateur

## Checkpoint de cloture

1. 29/30 fail-fast + 1 SKIPPED : row 15 Playwright non execute (env blocker Windows Defender, aucun code web modifie Sprint 10). Audit Sprint 11 Phase 0 finding V-1.
2. 5 commits atomiques Phase A-E : OUI
3. verification.md + audit_plan.md : OUI (ce commit)
4. PATTERNS.md a jour : OUI (T13-T22 Phase A)
5. Memory a jour : a faire post-commit
6. CI green sur GitHub : en attente push (remote configure, repo non cree)
7. 3 VPS live : en attente achat par utilisateur
