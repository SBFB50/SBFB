# Sprint 37 — Design Review (Day-0 Decisions D1-D5)

**Reviewer** : agent Explore independant (WebSearch + codebase audit).
**Date** : 2026-04-28.
**Scope** : Decisions D1-D5 from `sprint37_kickoff.md` §4.

---

## Summary

| Decision | Scoring | Risk |
|---|---|---|
| D1 — Log convergence (tracing-appender) | ✅ | LOW |
| D2 — .icns macOS (icns 0.3) | ⚠️ | MEDIUM |
| D3 — KudosLedger hash-chain (BLAKE3+JCS) | ✅ | LOW (MEDIUM future scale) |
| D4 — P2 batch fixes | ✅ | MINIMAL |
| D5 — Validator loop deferred S38 | ✅ | ACCEPTABLE |

---

## D1 — Log convergence ✅

- tracing-appender 0.2 : workspace dep, tokio-rs maintenu, standard Rust.
- Alternatives verifiees : log4rs (moins ergonomique), slog (moins maintenu),
  syslog (platform-dep, correctement rejete).
- Pas de risque identifie.

## D2 — .icns macOS ⚠️

- icns 0.3.1 : pure Rust (mdsteele/rust-icns), MIT, stable depuis 2017.
- **Gap** : "apple-icns alternative non-evaluee" mentionne dans la decision
  mais ce crate n'existe pas sur crates.io. `tauri-icns` (fork Tauri
  Foundation de mdsteele) existe et pourrait etre plus activement maintenu.
  La comparaison formelle est absente.
- **Gap 2** : couverture tailles icones macOS 11+ (1024x1024 requis) non
  verifiee explicitement.

## D3 — KudosLedger hash-chain ✅

- BLAKE3 1.5 + serde_jcs 0.2 : workspace deps, standards audites.
- `DOMAIN_KUDOS_V1` et `canonical_bytes()` deja reserves/existants dans
  nexus-core-rs (canonical.rs L86, L210).
- **Note architecture** : linear per-project chain choisi (O(n) verification)
  vs Merkle tree (O(log n), RFC 6962 Certificate Transparency). Linear est
  correct pour S37 (petits ledgers), mais si le ledger devient audit-grade
  (millions d'entrees), la migration vers Merkle tree sera necessaire.
- **Note** : append-only enforcement (pas d'UPDATE/DELETE sur kudos table)
  devrait etre documente dans le schema ou un commentaire SQL.

## D4 — P2 batch ✅

- Tous < 20 LOC, refactoring interne. Pas de risque.

## D5 — Validator loop deferred ✅

- Scope cut justifie (CuratorRuntimeHandle refactor requis).
- Carry S38 MANDATORY 3/3 correctement documente.

---

## Gaps signales

| # | Gap | Severite |
|---|---|---|
| 1 | D2: tauri-icns fork non compare ; "apple-icns" inexistant | MEDIUM |
| 2 | D3: Linear chain vs Merkle tree non analyse explicitement | MEDIUM (futur) |
| 3 | D3: Append-only enforcement non documente dans schema SQL | LOW |
| 4 | D1: Rotation daily synchronisee daemon/launcher a confirmer | LOW |

**Verdict** : PROCEED to Phase A. 1 ⚠️ a acknowledge.
