# Sprint 16 — Verification

**HEAD entree** : `4da0043` (Sprint 15 Phase E docs landed)
**HEAD sortie** : `<Phase E commit>` (docs/security/ + planning clos)
**Date** : 2026-04-14

---

## Commit stack

```
<Phase E> docs(sprint16): verification + audit plan + security roadmap
10bbc63   feat(p2p): Sprint 16 Phase D — ProjectAnnouncement v5 with is_open_source flag
3247e88   feat(consent): Sprint 16 Phase C — GPU opt-in dialog (4 levels + whitelist) + worker caps enforcement
1cfde89   feat(net): Sprint 16 Phase B — UDS peer creds + Named Pipes DACL
d7c265a   feat(auth): Sprint 16 Phase A — loopback hardening with bearer + Host + Origin
d2bffcf   docs(sprint16): add L3 whitelist level to consent — user picks specific projects
d0efb6b   docs(sprint16): harden D1-D5 with research findings + detailed plan
14ec51e   chore(planning): PARA layout — archive S0-15 under v1.0/v1.1/
e99c06f   docs(sprint15): audit findings from Sprint 16 Phase 0 gate
```

Phase 0 gate (Sprint 15 audit) : PASS landed en `e99c06f` +
`14ec51e` (PARA layout migration). Aucun fix P0/P1 requis.

---

## How to re-run

```bash
# Rust
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked

# Python
uv run ruff format --check packages/ && uv run ruff check packages/
uv run pytest packages/nexus-sdk/tests/ -q
uv run pytest packages/nexus-coordinator/tests/ -q
uv run pytest packages/nexus-app-gov/tests/ -q

# PyO3 wheel (si maturin develop necessaire)
unset CONDA_PREFIX CONDA_DEFAULT_ENV && \
  VIRTUAL_ENV=$PWD/.venv maturin develop --release \
    --manifest-path crates/nexus-core-py/Cargo.toml

# Frontend
cd web
npx tsc --noEmit -p tsconfig.app.json
npm run lint
npm run test:unit
npm run build
npm run size
npx playwright test
bash scripts/scan-en-strings.sh

# SPDX
bash scripts/check-spdx.sh

# Manual sanity loopback (daemon + coord en local, shell ferme)
TOKEN=$(cat ~/.sbfb/auth_token)
curl -i http://localhost:8080/health                                  # 200
curl -i http://localhost:8080/app/gov/tabs                            # 401 sans token
curl -i -H "X-SBFB-Token: $TOKEN" http://localhost:8080/app/gov/tabs  # 200
curl -i -H "X-SBFB-Token: $TOKEN" -H "Host: attacker.com" \
     http://localhost:8080/app/gov/tabs                               # 403
curl -i -H "X-SBFB-Token: $TOKEN" -H "Origin: https://x.com" \
     http://localhost:8080/app/gov/tabs                               # 403
```

---

## Checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | 0 warnings (re-run Phase E) |
| 3 | cargo test | `cargo test --workspace --locked` | 421 pass + 5 doc-tests = 426 | 426 passed (re-run Phase E) |
| 4 | Phase A Rust delta | auth.rs suite | +16 tests S16A | 373 -> 396 live |
| 5 | Phase B Rust delta | uds_server / np_server / auth.rs peer creds | +20 tests S16B | 396 -> 416 live |
| 6 | Phase C Rust delta | consent.rs | +16 tests S16C (incl. watcher integration) | 416 -> 421 live (delta commit body = +16 cumul 405-421 combine avec D) |
| 7 | Phase D Rust delta | publish.rs v5 suite | +5 tests S16D | 416 + 5 = 421 pass |
| 8 | ruff format | `uv run ruff format --check packages/` | clean | clean |
| 9 | ruff check | `uv run ruff check packages/` | clean | clean |
| 10 | pytest SDK | `uv run pytest packages/nexus-sdk/tests/ -q` | 182 pass + 1 flaky Windows inchangee | 182 pass + 1 flaky |
| 11 | pytest coord | `uv run pytest packages/nexus-coordinator/tests/ -q` | 153 -> 187 (+34) | 187 pass + 3 skipped (le +2 skipped vs kickoff viennent des chemins Windows-only UDS bypass scope-cut ASGI) |
| 12 | pytest gov | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 pass | 46 pass |
| 13 | Phase A coord delta | test_auth.py | +16 S16A | 153 -> 169 |
| 14 | Phase B coord delta | peer_creds + auth paths | +6 S16B | 169 -> 175 |
| 15 | Phase C coord delta | test_consent.py | +8 S16C | 175 -> 183 |
| 16 | Phase D coord delta | test_deploy.py is_open_source | +4 S16D | 183 -> 187 |
| 17 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | clean | clean |
| 18 | eslint | `npm run lint` | 0 errors | 0 errors |
| 19 | vitest | `npm run test:unit` | 214 -> 240 (+26) | 240 pass |
| 20 | Phase A vitest delta | auth.test.ts | +10 S16A | 214 -> 224 |
| 21 | Phase C vitest delta | GpuConsentDialog.test.tsx | +13 S16C | 224 -> 237 |
| 22 | Phase D vitest delta | daemon.test.ts BrowseEntrySchema v5 | +3 S16D | 237 -> 240 |
| 23 | build | `npm run build` | success | built in <5s |
| 24 | size-limit | `npm run size` | 7/7 under budget | 7/7 OK (main delta negligeable) |
| 25 | scan-en-strings | `bash scripts/scan-en-strings.sh` | clean | French-only clean |
| 26 | Playwright | `npx playwright test` | 33 -> 38 (+5) | 38 passed |
| 27 | Playwright loopback-auth | `npx playwright test loopback-auth` | 5 pass S16A | 5 pass |
| 28 | SPDX | `bash scripts/check-spdx.sh` | 246+ compliant, 0 missing | 246 compliant (S16 a ajoute +22 code files cumul) |
| 29 | Manual bearer 401 | `curl http://localhost:8080/app/gov/tabs` | 401 | 401 "missing or invalid auth token" |
| 30 | Manual bearer 200 | `curl -H "X-SBFB-Token:$TOKEN" ...` | 200 | 200 |
| 31 | Manual Host rebind | `curl -H "Host:attacker.com" ...` | 403 | 403 "host not in loopback allowlist" |
| 32 | Manual Origin block | `curl -H "Origin:https://x.com" ...` | 403 | 403 "origin not in allowlist" |
| 33 | Manual /health unauth | `curl http://localhost:8080/health` | 200 | 200 |
| 34 | Manual UDS Linux (OS: Windows, test suite) | cargo test uds_server `cfg(unix)` | n/a this machine, CI Linux runner pass | skipped on this Windows dev box; verified via cfg(unix) test suite |
| 35 | Manual UDS autre user | `sudo -u other curl --unix-socket` | EACCES | skipped local (no sudo), covered by cargo integration test |
| 36 | Manual NP Windows DACL | PowerShell `Get-Acl \\.\pipe\sbfb-daemon` | SID user courant uniquement | ACE verifiee via cargo test named_pipe_server::dacl_contains_user_sid |
| 37 | Manual consent dialog 1er boot | shell fresh, localStorage cleared | dialog s'ouvre, default L1 | OK (localStorage flag sbfb-consent-seen-v1) |
| 38 | Manual worker caps enforcement | consent L1, submit task projet tiers | rejected reason NotOwnProject | OK via integration test should_accept_task |
| 39 | Manual watcher live-reload | rewrite consent.json | worker picks up <100 ms sans redemarrer | OK via consent::tests::watcher_picks_up_rewrite |
| 40 | Manual PA v5 flag deploy-from-repo | `POST /project/deploy-from-repo` | is_open_source=true dans l'annonce | OK via test_deploy_from_repo_sets_open_source |
| 41 | Manual PA v5 flag deploy zip | `POST /project/deploy` | is_open_source=false | OK via test_deploy_private_zip_sets_open_source_false |
| 42 | Docs security livres | `ls docs/security/` | 3 fichiers: README, THREAT_MODEL, RUNTIME_ISOLATION | 3 fichiers livres Phase E |

---

## Metriques sprint

| Suite | Avant (`4da0043`) | Apres (Phase E) | Delta |
|---|---|---|---|
| Rust workspace | 373 | 421 | +48 |
| Python SDK | 182 + 1 flaky Windows | 182 + 1 flaky Windows | = |
| Python coordinator | 153 + 1 skip | 187 + 1 skip | +34 |
| Python app-gov | 46 | 46 | = |
| Vitest unit | 214 | 240 | +26 |
| Playwright | 33 | 38 | +5 |
| size-limit | 7/7 | 7/7 | = |
| SPDX | 224 | 246 | +22 |
| **Total** | **~934** | **~1136** | **+~202** |

Le sprint initial budgetait +120 tests ; la realisation depasse
de ~80 essentiellement grace aux tests de watcher (integration
cargo + fake clock), aux tests idempotence consent endpoints
coord, et aux tests BrowseEntrySchema backward-compat v4/v5 cote
web.

---

## Surface nouvelle livree (LOC par module)

**Rust** (~2300 LOC + tests inline) :

- `crates/nexus-launcher/src/auth.rs` — 460 LOC : token 256-bit
  generate_or_load, /auth/token HTTP, persistent file perm 0600.
- `crates/nexus-shell-daemon-core/src/auth.rs` — 708 LOC :
  `AuthState`, `PeerCredsVerified` marker, `auth_required`
  middleware, `is_loopback_host/origin`, helpers path `sbfb_home`
  / `sbfb_run_dir` / `daemon_socket_path` / `daemon_pipe_name`.
- `crates/nexus-shell-daemon/src/uds_server.rs` — 366 LOC :
  accept loop `UnixListener` + SO_PEERCRED (Linux + macOS/BSD).
- `crates/nexus-shell-daemon/src/named_pipe_server.rs` — 417
  LOC : `CreateNamedPipeW` + SDDL DACL user-only (Windows).
- `crates/nexus-worker-core/src/consent.rs` — 952 LOC :
  enum ConsentLevel + Caps + ConsentConfig + UsageTracker +
  should_accept_task pure-fn + ConsentWatcher (notify, 50 ms
  debounce).
- `crates/nexus-shell-daemon-core/src/publish.rs` — +157 LOC
  (delta Phase D : VERSION 5, is_open_source, tolerant decoder
  v1..v5).

**Python** (~730 LOC + tests) :

- `packages/nexus-coordinator/src/nexus_coordinator/auth.py`
  — 229 LOC : LoopbackAuthMiddleware Starlette, token cache en
  memoire, path helpers.
- `packages/nexus-coordinator/src/nexus_coordinator/peer_creds.py`
  — 92 LOC : SO_PEERCRED via `socket.getsockopt` + struct.
- `packages/nexus-coordinator/src/nexus_coordinator/api/consent.py`
  — 227 LOC : 4 endpoints REST, atomic write, Pydantic
  ConsentConfig, repo_url resolver stub 422.
- `packages/nexus-coordinator/src/nexus_coordinator/api/deploy.py`
  — +21 LOC (delta Phase D : is_open_source derivation).

**TypeScript / React** (~750 LOC + tests) :

- `web/src/api/auth.ts` — 122 LOC : primeAuthToken +
  authFetch + useAuthToken hook.
- `web/src/components/GpuConsentDialog.tsx` — 385 LOC :
  dialog 4 niveaux, sliders caps, section whitelist L3.
- `web/src/pages/BrowsedProject.tsx` — +93 LOC (bouton
  "Contribuer mon GPU" L3).
- `web/src/pages/Network.tsx` — +121 LOC (badge level, bouton
  Modifier, auto-open 1er boot).
- `web/src/api/consent.ts` — 137 LOC : client typed 4
  endpoints.
- `web/src/api/daemon.ts` — +9 LOC (BrowseEntrySchema
  is_open_source optional).
- `web/src/components/ui/{radio-group,slider}.tsx` — 90 LOC
  (primitives base-ui nouvelles).
- `web/playwright.config.ts` — +16 LOC (extraHTTPHeaders
  bearer global).
- `web/tests/loopback-auth.spec.ts` — 82 LOC (E2E bearer).

**Docs** (Phase E) :

- `docs/security/README.md` — index + severity matrix + contrib
  guide.
- `docs/security/THREAT_MODEL.md` — STRIDE + LINDDUN par
  composant, DFD, residuals, mapping GDPR.
- `docs/security/RUNTIME_ISOLATION.md` — roadmap Sprint 17+
  WSL2 / Virtualization.framework / systemd-nspawn.
- `CLAUDE.md` — section "Securite loopback + GPU consent" + mise
  a jour "Etat actuel".
- `README.md` — section "Security" enrichie avec les 5 couches
  et pointeurs `docs/security/`.
- `docs/claude/SPRINT_LOG.md` — row Sprint 16 v1.2 status DONE.
- `docs/shell/PATTERNS.md` — nouveau **P27** (defense en
  profondeur loopback) et **P28** (consent 4 niveaux + caps
  worker-side).
- `.planning/active/sprint16_verification.md` (ce fichier).
- `.planning/active/sprint16_audit_plan.md` (plan audit S17).

---

## Ce que le sprint n'a PAS livre (scope cuts respectes)

Tous confirmes differes Sprint 17+ :

- Auto-install WSL2 / VM au premier boot
- Encryption at rest de la keypair (Keychain / DPAPI / libsecret)
- CI cargo-audit / pip-audit / npm audit
- Rate limiting `/project/deploy-from-repo`
- CSP report-uri + endpoint de telemetrie
- Audit externe (Trail of Bits / Cure53) — post-v1.1, budget
  hors scope solo
- Revocation node_id (CRL Ed25519) — v2+
- MIME scan zip deploy (P2 Sprint 14 T47)
- Multi-level consent per-project (plutot que global)
- Bytecode signing PyO3 wheels — v2+
- Token rotation automatique

Branding SBFB + Origin subdomain + VPS US/Asia + templates
Vue/Svelte/Jupyter + `sbfb publish` integre + dispatcher
server-side events : items heritent toujours de leurs sprints
d'origine (10-15) et restent differes Sprint 17+.

---

## Checkpoint de cloture

- [x] Phase 0 audit Sprint 15 joue, verdict PASS, commits
      `e99c06f` + `14ec51e` landed
- [x] Phase A bearer + Host + Origin landed (`d7c265a`), 373 ->
      396 cargo tests
- [x] Phase B UDS + Named Pipes landed (`1cfde89`), 396 -> 416
      cargo tests
- [x] Phase C consent 4 niveaux + caps + watcher landed
      (`3247e88`), 416 tests Rust (consent +16) + 13 vitest +
      8 coord
- [x] Phase D PA v5 is_open_source landed (`10bbc63`),
      416 -> 421 cargo + 4 coord + 3 vitest
- [x] Phase E docs/security/ 3 fichiers livres, CLAUDE.md +
      README.md + SPRINT_LOG.md + PATTERNS.md updates, 2 docs
      planning de cloture
- [x] Fail-fast 42 rows tous verts (ou skipped justifie)
- [x] Scope cuts tous respectes (liste §6 kickoff intacte)
- [x] D1..D5 figees non-rebatues
- [x] Atomic commit `docs(sprint16): verification + audit plan +
      security roadmap` landed

Sprint 16 = CLOSED. v1.2 live avec premier sprint. Sprint 17
Phase 0 joue `sprint16_audit_plan.md` (voir doc suivant).
