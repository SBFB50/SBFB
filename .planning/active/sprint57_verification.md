# Sprint 57 — Verification

**Date** : 2026-05-10
**Tip Phase D** : `74fa29a` | **Tip fix** : `a3943ed` | **Tip research** : `97202b9`

---

## §1 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | 0 warnings |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1232, 0 fail | 1232 pass, 0 fail |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | ok |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | ok |
| 6 | npm lint | `npm run lint` (web/) | 0 error | 0 error (5 warnings pre-existants) |
| 7 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error | 0 error |
| 8 | Vitest | `npm run test:unit` (web/) | >= 256 | 256 pass |
| 9 | npm build | `npm run build` (web/) | ok | ok |
| 10 | size-limit | `npm run size` (web/) | 6/6 | 6/6 |
| 11 | scan-en-strings | `bash scripts/scan-en-strings.sh` | clean | clean |
| 12 | Phase A preflight G8 | verdict | EXECUTE | EXECUTE |
| 13 | Phase A review | verdict | PASS | PASS |
| 14 | Phase B preflight G8 | verdict | EXECUTE | EXECUTE |
| 15 | Phase B review | verdict | PASS | PASS |
| 16 | Phase C preflight G8 | verdict | EXECUTE | EXECUTE |
| 17 | Phase C review | verdict | PASS | PASS |
| 18 | Phase D preflight G8 | verdict | EXECUTE | EXECUTE |
| 19 | Phase D review | verdict | PASS | PASS |
| 20 | §P46 cross-platform doc | `grep "§P46" docs/rust/PATTERNS.md` | present | present |
| 21 | E2E gossip test exists | `grep "gossip_exchange" crates/nexus-test-harness/tests/multi_daemon.rs` | present | present |
| 22 | Storage persistence | `grep "app_storage" crates/nexus-coordinator-rs/src/db.rs` | present | present |
| 23 | Protocol Explorer exists | `ls examples/sbfb-explorer/index.html` | present | present |
| 24 | Ideas Hub exists | `ls examples/sbfb-ideas/index.html` | present | present |
| 25 | Scope cuts | 13/13 respectes | all checked | all checked |
| 26 | Delta tests | cumule documente | documented | +5 Rust (1227→1232), +0 Vitest (256→256) |
| 27 | Fix sandbox forms | Ideas Hub form→div + button click | committed | `a3943ed` |
| 28 | Research docs | 4 documents .planning/research/ | committed | `97202b9` |

---

## §2 Delta tests cumule

| Suite | Avant S57 | Apres S57 | Delta | Detail |
|---|---|---|---|---|
| Rust nextest | 1227 | 1232 | +5 | Phase A +1 (E2E gossip), Phase B +4 (storage persistence) |
| Vitest | 256 | 256 | +0 | Phases C+D = static HTML apps, no Vitest changes |
| Playwright | 42+2f | 42+2f | +0 | inchange |
| size-limit | 6/6 | 6/6 | +0 | inchange |

---

## §3 Scope cuts honoured (13/13)

1. LT-7 Tier 3 (N builders, auto-deploy) — S58+ : non touche
2. Verified deploy E2E from repos Git separes — S58 : non touche
3. Protocol Explorer F3 avance (gossip stats) — S58 : non touche
4. Protocol Explorer F4 (tutoriel interactif) — S58 : non touche
5. Ideas Hub F3 (lier repos Git) — S58 : non touche
6. Ideas Hub F4 (groupes de travail) — post-v1.0 : non touche
7. Ideas Hub F5 (integration reseau) — post-v1.0 : non touche
8. Kudos-weighted voting — S58 : non touche
9. AppStorage replication P2P — **replanifie pre-v1.0** (decision utilisateur 2026-05-10)
10. Rate-limit retain_recent — S58 : non touche
11. P2-JITTER-SCOPE test integration — S58 3/3 MANDATORY : non touche
12. P2-INVITE-U16-WIRE doc — S58 3/3 MANDATORY : non touche
13. LT-1 Kudos-v2 fairness reform — S58+ : non touche

---

## §4 MANDATORY items resolution

| Item | Compteur entree | Resolution | Commit |
|---|---|---|---|
| P2-S54-windows-test-cfg-unix | 3/3 MANDATORY | CLOSED Phase A | `f1f26d5` |
| P2-S54-test-E2E-multi-noeuds | 3/3 MANDATORY | CLOSED Phase A | `f1f26d5` |
| P2-STORAGE-SQLITE | 1/3 | CLOSED Phase B | `636c87c` |

---

## §5 Findings carry-over for memory

- **AppStorage replication P2P** replanifie de post-v1.0 a pre-v1.0 (S58). Decision utilisateur 2026-05-10. 4 docs research dans `.planning/research/`.
- **Ideas Hub sandbox fix** : `<form>` + `type="submit"` bloque par `sandbox="allow-scripts"` sans `allow-forms`. Fix : `<div>` + `type="button"` + click handler JS. Pattern a appliquer pour toute future app SBFB dans iframe.
- **sbfb-bridge.js copie manuelle** : divergence risk si SDK evolue. Carry S58 : script de sync ou build step.
- **Docker Linux flaky** : `sigint_triggers_graceful_shutdown_and_removes_running_json` echoue en Docker (timing signal). Pre-existant, non lie a S57.
