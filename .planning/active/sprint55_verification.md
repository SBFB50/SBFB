# Sprint 55 — Verification

**Date** : 2026-05-08
**Tip cloture** : `d37c54f` (Phase D) → Phase E wrap-up
**Phases livrees** : A + A.1 + B + C + D (5 commits feat + 11 fix)

---

## §1 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | 0 warnings |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1216, 0 fail | 1216 passed, 0 fail |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | ok (1 ignored) |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | ok |
| 6 | npm lint | `npm run lint` (web/) | 0 error | 0 error |
| 7 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error | 0 error |
| 8 | Vitest | `npm run test:unit` (web/) | >= 250 | 250 |
| 9 | npm build | `npm run build` (web/) | ok | ok |
| 10 | size-limit | `npm run size` (web/) | 6/6 | 6/6 |
| 11 | scan-en-strings | `bash scripts/scan-en-strings.sh` | clean | clean |
| 12 | Phase A preflight G8 | verdict | EXECUTE | EXECUTE |
| 13 | Phase A review | verdict | PASS | PASS |
| 14 | Phase A.1 preflight G8 | verdict | EXECUTE | EXECUTE |
| 15 | Phase A.1 review | verdict | PASS | PASS |
| 16 | Phase B preflight G8 | verdict | EXECUTE | EXECUTE |
| 17 | Phase B review | verdict | PASS | PASS |
| 18 | Phase C preflight G8 | verdict | EXECUTE | EXECUTE |
| 19 | Phase C review | verdict | PASS | PASS |
| 20 | Phase D preflight G8 | verdict | EXECUTE | EXECUTE |
| 21 | Phase D review | verdict | PASS | PASS |
| 22 | build task dispatch | test_submit_build_task | pass | pass |
| 23 | quorum SHA256 | test_quorum_majority_sha256 | pass | pass |
| 24 | INVITE_FORMAT_VERSION | grep | present | present |
| 25 | Scope cuts | 15/15 respectes | all checked | all checked |
| 26 | Delta tests | cumule documente | documented | documented |

---

## §2 Delta tests cumule

| Suite | Entree (ee0e54c) | Sortie (d37c54f) | Delta | Phases |
|---|---|---|---|---|
| Rust nextest | 1207 | 1216 | +9 | B +4, C +5, A/A.1/D +0 |
| Rust doctests | 6p 1i | 6p 1i | +0 | — |
| Vitest | 250 | 250 | +0 | aucun changement frontend |
| size-limit | 6/6 | 6/6 | +0 | — |

**Total Rust nextest** : 1207 → 1216 (+9).

---

## §3 Phases livrees

### Phase A — Woodpecker server deploy + GHA validation
- Woodpecker CI deploye sur VPS sbfb-eu (ci.sbfb.world)
- Docker Compose server+agent, Caddy TLS Let's Encrypt, systemd
- Pipeline vert. 11 fix Linux-only (journald, /proc, SIGINT, timing)
- CLOSE P2-REVIEW-B-1-S52 (3/3 MANDATORY) + P2-REVIEW-B-2-S52 (3/3 MANDATORY)
- CLOSE P2-S54-AUDIT-1 (flaky browse test)

### Phase A.1 — CI test fiabilite
- 6 tests timing-dependent stabilises (sleep→drop+await, pause+advance)
- Process Docker pipeline local obligatoire avant push

### Phase B — LT-7 build executor + dispatcher task_type routing
- task_type "build" dans le protocole (prompt/model vides, metadata build.* requis)
- Build executor MVP : clone→checkout→cargo build→SHA256
- +4 tests Rust (1207→1211)
- Carries S56 : P2-BUILD-TIMEOUT + P2-REMAP-PATH

### Phase C — LT-7 quorum SHA256 validation
- TaskStatus::AwaitingQuorum + table task_results (DB-persistent)
- Validator quorum : majorite SHA256 → accepted, divergence → rejected
- BUILD_DEFAULT_REDUNDANCY=3, inference tasks bypass quorum
- +5 tests Rust (1211→1216)

### Phase D — P2 batch quick carries
- jitter ±15s republish timer (thundering-herd prevention)
- project_name "sbfb" → constante DEFAULT_PROJECT_NAME
- // SAFETY: comments sur tous les unsafe FFI
- INVITE_VERSION → INVITE_FORMAT_VERSION + u8→u16
- +0 tests (mecaniques)
- Carries S56 : P2-JITTER-SCOPE + P2-INVITE-U16-WIRE

---

## §4 Carries sortants S56

### 3/3 MANDATORY (passent de 2/3 a 3/3)

| Item | Source | Compteur S56 |
|---|---|---|
| P2-S53-outbox non-persistant | S53 Phase F | **3/3 MANDATORY** |
| P2-S53-browse_request rate-limit | S53 Phase G | **3/3 MANDATORY** |

### P2 (compteur incrémente)

| Item | Source | Compteur S56 |
|---|---|---|
| P2-S54-forbid-deny-doc | S54 Phase A | 2/3 |
| P2-S54-lightcheck-edition-faux-positif | S54 Phase A | 2/3 |
| P2-S54-windows-test-cfg-unix | S54 Phase B | 2/3 |
| P2-S54-test-E2E-multi-noeuds | S54 Phase C | 2/3 |
| P2-S54-rustfmt-drift-sessions | S54 Phase D | 2/3 |

### P2 nouveaux S55

| Item | Source |
|---|---|
| P2-BUILD-TIMEOUT | Phase B review |
| P2-REMAP-PATH | Phase B review |
| P2-JITTER-SCOPE | Phase D review |
| P2-INVITE-U16-WIRE | Phase D review |

### Exemptions / heritage

| Item | Statut |
|---|---|
| P2-A-1 rand blocker upstream | exemption externe (inchange) |
| P2-AUDIT-2 iroh transitives | herite pin 0.98 (inchange) |

### Long-term

| Item | Statut |
|---|---|
| LT-7 self-hosted build | Tier 1 DONE (Woodpecker CI). Tier 2 foundation DONE (executor+quorum). Tier 3 reste (N builders, auto-deploy). **PRE-V1.0 status : Tier 1+2 livres, Tier 3 S56+** |
| LT-1 Kudos-v2 fairness | trigger Gini > 0.70. Latent. |
| LT-2 Radicle | trigger tag v1.0. Latent. |
| LT-5 redundancy persistence | reclassifie S26. Latent. |

### CLOSED S55

| Item | Resolution |
|---|---|
| P2-REVIEW-B-1-S52 Woodpecker serveur | Phase A — 3/3 MANDATORY FERME |
| P2-REVIEW-B-2-S52 GHA validation | Phase A — 3/3 MANDATORY FERME |
| P2-S54-AUDIT-1 flaky browse test | Phase A — CLOSE |
| P2-S54-jitter-republish | Phase D — CLOSE |
| P2-S54-project-name-hardcode | Phase D — CLOSE |
| P2-S54-AUDIT-2 SAFETY convention FFI | Phase D — CLOSE |
| P2-S54-AUDIT-3 invite version naming | Phase D — CLOSE |

---

## §5 Findings carry-over for memory

- LT-7 Tier 1+2 livres S55, Tier 3 reste. Le build executor est
  MVP (tmpdir, pas d'isolation podman, pas de streaming logs).
  Le quorum est DB-persistent et survit aux restarts.
- CI operationnel : Woodpecker ci.sbfb.world + GHA. Docker pipeline
  local obligatoire avant push (memes images que VPS).
- 2 items passent 3/3 MANDATORY S56 (outbox + browse_request).
  S56 DOIT les resoudre.
- 4 P2 nouveaux S55 (build-timeout, remap-path, jitter-scope,
  invite-u16-wire).
- 5 P2 S54 passent 2/3 (forbid-deny-doc, lightcheck, windows-test,
  E2E multi-noeuds, rustfmt drift).
- Compteurs tests : 1216 Rust / 250 Vitest / 6/6 size.
