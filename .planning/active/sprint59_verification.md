# Sprint 59 — Verification

**Date** : 2026-05-11
**Tip d'entree** : `80ec664`
**Tip de sortie** : `5412d9b` (Phase D wrap-up inclus)
**Theme** : Launcher readiness + verified deploy E2E + LT-1 Kudos-v2
+ stabilisation (early adopter ready)

---

## §1 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | 0 diff ✅ |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | 0 warnings ✅ |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1254, 0 fail | 1257 pass, 0 fail ✅ |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | ok (1 ignored) ✅ |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | ok ✅ |
| 6 | npm lint | `npm run lint` (web/) | 0 error | 0 error (5 warnings pre-existant) ✅ |
| 7 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error | 0 error ✅ |
| 8 | Vitest | `npm run test:unit` (web/) | >= 258 | 258 pass ✅ |
| 9 | npm build | `npm run build` (web/) | ok | ok ✅ |
| 10 | size-limit | `npm run size` (web/) | 6/6 | 6/6 ✅ |
| 11 | scan-en-strings | `bash scripts/scan-en-strings.sh` | clean | clean ✅ |
| 12 | Phase A preflight G8 | sprint59_phase_A_preflight.md | EXECUTE | EXECUTE ✅ |
| 13 | Phase A review | sprint59_phase_A_review.md | PASS | PASS ✅ |
| 14 | Phase B preflight G8 | sprint59_phase_B_preflight.md | EXECUTE | EXECUTE ✅ |
| 15 | Phase B review | sprint59_phase_B_review.md | PASS | PASS ✅ |
| 16 | Phase C preflight G8 | sprint59_phase_C_preflight.md | EXECUTE | EXECUTE ✅ |
| 17 | Phase C review | sprint59_phase_C_review.md | PASS | PASS ✅ |
| 18 | Phase D preflight G8 | sprint59_phase_D_preflight.md | EXECUTE | EXECUTE ✅ |
| 19 | LT-1 credit() log-utility | `kudos_ledger.rs:51` credit() log2 | present | present ✅ |
| 20 | LT-1 effective_score EMA | `kudos_ledger.rs:18,124` alpha=0.97 | present | present ✅ |
| 21 | SBFB.json seed apps | `examples/sbfb-explorer/SBFB.json` + `examples/sbfb-ideas/SBFB.json` | present | present ✅ |
| 22 | Deploy E2E tests | `http.rs:4911,4934` deploy_from_repo validation | present | present ✅ |
| 23 | Deploy page | `web/src/pages/Deploy.tsx` | present | present ✅ |
| 24 | MessageBoxW | `launcher/main.rs` error_msgbox() cfg(windows) | present | present ✅ |
| 25 | STORAGE-JOIN-VALIDATE | `storage_api.rs` is_replicated check | present | present ✅ |
| 26 | STORAGE-ANTISPAM | `storage_limiter.rs` StorageWriteLimiter GCRA | present | present ✅ |
| 27 | Scope cuts | 14/14 respectes | all checked | 14/14 ✅ |
| 28 | Delta tests cumule | documented in commit bodies | documented | +17 Rust +2 Vitest ✅ |
| 29 | Sync bridge SDK | `bash scripts/sync-bridge-sdk.sh` | exit 0 | exit 0 (3 copies match) ✅ |

**Verdict : 29/29 rows verts.**

---

## §2 Compteurs tests

| Suite | Entree S59 | Sortie S59 | Delta |
|---|---|---|---|
| Rust nextest | 1240 | 1257 | +17 |
| Rust doctests | 6 (1 ignored) | 6 (1 ignored) | +0 |
| Vitest | 256 | 258 | +2 |
| Playwright | 42 + 2f (env) | 42 + 2f (env) | +0 |
| size-limit | 6/6 | 6/6 | = |
| **Total** | **~1502** | **~1521** | **+19** |

**Note env** : Vitest 258/258 requiert
`NODE_OPTIONS=--no-experimental-webstorage` sur Node 25 (CI pin
Node 20, pas regression S59).

---

## §3 Delta tests par phase

| Phase | Commit | Rust delta | Vitest delta | Total delta |
|---|---|---|---|---|
| A (Kudos-v2) | `e194329` | +7 (1240→1247) | +0 | +7 |
| A fixes | `2f2d513` + `14775f2` | +3 (1247→1250) | +0 | +3 |
| B (Deploy E2E) | `46ed2c2` | +1 (1250→1251) | +2 (256→258) | +3 |
| C (Launcher+storage) | `9f42ec4` | +4 (1251→1255) | +0 | +4 |
| C fixes | `f63ecef` + `4d0f7b2` | +2 (1255→1257) | +0 | +2 |
| **Total S59** | | **+17** | **+2** | **+19** |

---

## §4 Scope cuts verifies (14/14)

| # | Scope cut | Disposition | Respect |
|---|---|---|---|
| 1 | AppStorage Phase 2 (namespace per manifest) | Deferred S60+ | ✅ |
| 2 | AppStorage Phase 3 (optimisations, purge) | Deferred post-v1.0 | ✅ |
| 3 | Kudos-v2 DRF (Couche B multi-ressource) | Deferred post-v1.0 | ✅ |
| 4 | Kudos-weighted voting | Deferred post-v1.0 | ✅ |
| 5 | Keyoxide identity verification in deploy | Deferred S60/post-v1.0 | ✅ |
| 6 | NSIS/WiX installer | Deferred S60 | ✅ |
| 7 | Tray icon | Deferred S60 | ✅ |
| 8 | Frontend P2P distribution | Deferred S60 | ✅ |
| 9 | Protocol Explorer F3 (gossip stats avance) | Deferred S60+ | ✅ |
| 10 | Protocol Explorer F4 (tutoriel interactif) | Deferred post-v1.0 | ✅ |
| 11 | Ideas Hub F3 (lier repos Git) | Deferred S60 | ✅ |
| 12 | Ideas Hub F4-F5 (groupes, integration) | Deferred post-v1.0 | ✅ |
| 13 | Ticket Write rotation dynamique (Option B/C) | Deferred post-v1.0 | ✅ |
| 14 | LT-7 Tier 3 validation controlee | Deferred S60 pre-tag | ✅ |

---

## §5 Items CLOSED ce sprint

| Item | Phase | Detail |
|---|---|---|
| LT-1 Kudos-v2 fairness reform | Phase A | log-utility credit() + EMA effective_score() alpha=0.97. ROADMAP_COMMITMENTS pre-v1.0 satisfait. 9 sprints carry (S50→S59). |
| P2-STORAGE-JOIN-VALIDATE | Phase C | is_replicated_app() check dans storage_join handler. Compteur 1/3→CLOSED. |
| P2-STORAGE-ANTISPAM | Phase C | StorageWriteLimiter GCRA 10 writes/min per-author per-app. Compteur 1/3→CLOSED. |

---

## §6 Carries residuels S60

| Item | Compteur S60 | Justification |
|---|---|---|
| P2-A-1 rand blocker upstream | 20+/3 | Exemption externe — dep `rand` upstream bloque version compatible iroh 0.98. |
| P2-AUDIT-2 iroh transitives | herite | Pin iroh 0.98 (Day 0 #3) — transitives non controlables. |

---

## §7 Commits S59

| # | SHA | Type | Message |
|---|---|---|---|
| 1 | `9456572` | chore(planning) | Sprint 59 kickoff + plan + design review |
| 2 | `e194329` | feat(sprint59) | Sprint 59 Phase A — LT-1 Kudos-v2 log-utility + EMA fairness reform |
| 3 | `2f2d513` | fix(sprint59) | wire diagnostic fairness to EMA effective scores |
| 4 | `14775f2` | fix(sprint59) | fmt fix + diagnostic EMA non-empty test |
| 5 | `a734eb8` | chore(planning) | Sprint 59 Phase B preflight G8 EXECUTE |
| 6 | `fb43368` | chore(planning) | Sprint 59 Phase B review PASS |
| 7 | `46ed2c2` | feat(sprint59) | Sprint 59 Phase B — Verified deploy E2E + seed SBFB.json + Deploy page |
| 8 | `c882427` | chore(planning) | Sprint 59 Phase C preflight G8 EXECUTE |
| 9 | `8f17164` | chore(planning) | Sprint 59 Phase C review PASS |
| 10 | `9f42ec4` | feat(sprint59) | Sprint 59 Phase C — Launcher MessageBox + storage validation + rate-limit |
| 11 | `f63ecef` | fix(sprint59) | storage retain_recent housekeeping + 2 handler tests |
| 12 | `4d0f7b2` | fix(sprint59) | assert replicated-app guard in storage_join test |
| 13 | `6c76568` | chore(docs) | AFFiNE SBFB integration docs + babel protocol update + roadmap migration update |
| 14 | `5412d9b` | chore(sprint59) | Phase D — wrap-up + verification + audit plan S60 |

3 feat + 4 fix + 5 chore planning + 1 chore docs + 1 chore wrap-up = 14 commits.

---

## §8 G8 preflights resume

| Phase | Verdict | Document |
|---|---|---|
| A | EXECUTE plan-as-is | sprint59_phase_A_preflight.md |
| B | EXECUTE plan-as-is | sprint59_phase_B_preflight.md |
| C | EXECUTE plan-as-is | sprint59_phase_C_preflight.md |
| D | EXECUTE plan-as-is | sprint59_phase_D_preflight.md |

**Trente-neuvieme sprint G8 systematique 4/4 (0 DESIGN-CONFLICT,
4 EXECUTE).**

---

## §9 Findings carry-over for memory

- **Compteurs** : 1257 Rust / 258 Vitest / 42+2f PW / 6/6 size /
  ~1521 total. Entree → sortie : +17 Rust / +2 Vitest / +19 total.
- **LT-1 Kudos-v2 CLOSED** : 9 sprints carry (S50→S59), log-utility
  + EMA alpha=0.97. DRF (Couche B) reste post-v1.0.
- **P2-STORAGE-JOIN-VALIDATE CLOSED** + **P2-STORAGE-ANTISPAM CLOSED**.
- **Carries S60** : P2-A-1 rand (exemption) + P2-AUDIT-2 transitives
  (herite pin 0.98). Tres legers.
- **Roadmap** : S59 = early adopter ready. S60 = installer NSIS/WiX
  + tray + frontend P2P + LT-7 Tier 3 → tag v1.0 (end user ready).
- **Note env** : Node 25 require `--no-experimental-webstorage` pour
  Vitest (CI pin Node 20, pas regression).
