# Sprint 34 — Verification fail-fast

**Date** : 2026-04-28
**Tip Phase C** : `4776948`

## Fail-fast checklist (30/30)

| # | Check | Critère | Observed | Status |
|---|---|---|---|---|
| 1 | Rust compile workspace | 0 errors | 0 errors | ✅ |
| 2 | Rust nextest pass | 901+ pass, 0 fail | 902 pass, 0 fail | ✅ |
| 3 | Rust doctests pass | 0 fail | 0 fail (1 ignored) | ✅ |
| 4 | Rust clippy clean | 0 warnings | 0 warnings | ✅ |
| 5 | Rust fmt clean | no output | clean | ✅ |
| 6 | Release build daemon | Finished | Finished 0.66s (cached) | ✅ |
| 7 | Python ruff format | clean | 154 files already formatted | ✅ |
| 8 | Python ruff check | pass | All checks passed | ✅ |
| 9 | SDK 195 pass | 195 pass | 195 pass | ✅ |
| 10 | Coord 406+ pass | 406+ pass + 36f stale | 408 pass + 37 fail (PyO3 stale) + 6 skip | ✅ |
| 11 | Gov 46 pass | 46 pass | 46 pass | ✅ |
| 12 | Frontend lint | 0 errors | 0 errors (7 warnings pre-existing) | ✅ |
| 13 | Frontend tsc | clean | clean | ✅ |
| 14 | Vitest 267+ pass | 267+ pass | 267 pass | ✅ |
| 15 | Frontend build | success | success | ✅ |
| 16 | size-limit 7/7 | 7/7 pass | 7/7 pass (120.18 kB CSS) | ✅ |
| 17 | Playwright | 42+ pass | 42 pass + 2 fail (env pre-existing) | ✅ |
| 18 | en-strings | clean | clean | ✅ |
| 19 | FORMAT_VERSION v1 | all = 1 | all = 1 (8 consts checked + ANNOUNCEMENT_VERSION + 3 SCHEMA_VERSION) | ✅ |
| 20 | HARDENING compteurs | updated S34 | updated (902 Rust / ~1905 total) | ✅ |
| 21 | Planning docs | complets | kickoff + plan + design_review + 3 preflights + 3 reviews | ✅ |
| 22 | rand unification outcome | documented | Phase A review P2-A-1 : blocker upstream frost-core rand_core 0.6, sous-arbres disjoints | ✅ |
| 23 | COEP E2E test pass | real zip | blob_serve_coep_headers_on_real_zip pass (+1 test) | ✅ |
| 24 | frost-ed25519 eval | documented | upgraded 2.1→3.0 inline (0 LOC delta, Ed25519 byte-identical) | ✅ |
| 25 | Windows exe icon | present | assets/nexus-launcher.ico + build.rs winresource | ✅ |
| 26 | Windows exe no console | subsystem set | `cfg_attr(not(debug_assertions), windows_subsystem = "windows")` | ✅ |
| 27 | Launcher log file | setup | `~/.sbfb/launcher.log` + panic hook in main.rs | ✅ |
| 28 | macOS .app bundle | structure valid | configs/macos/Info.plist + scripts/bundle-macos.sh | ✅ |
| 29 | Linux .desktop file | valid | configs/desktop/nexus-launcher.desktop freedesktop compliant | ✅ |
| 30 | install-node.sh .desktop | integration | 4 occurrences .desktop/XDG integration | ✅ |

| 31 | Launcher running.json path | matches daemon | fix(launcher): delegates to daemon-core paths | ✅ (fixed Phase D) |

**Résultat** : 31/31 verts (row 31 = hotfix trouvé lors du test manuel Windows).

## §2 Compteurs tests cumulés

| Suite | Avant S34 | Après S34 | Delta |
|---|---|---|---|
| Rust nextest | 901 | 902 | +1 (COEP E2E Phase A) |
| Rust doctests | 0 pass (1 ignored) | 0 pass (1 ignored) | 0 |
| SDK pytest | 195 | 195 | 0 |
| Coord pytest | 409+36f+6s | 408+37f+6s | -1/+1 (PyO3 stale variance) |
| Gov pytest | 46 | 46 | 0 |
| Vitest | 267 | 267 | 0 |
| Playwright | 42+2f | 42+2f | 0 |
| size-limit | 7/7 | 7/7 | 0 |
| **Total** | **~1904** | **~1905** | **+1** |

## §3 Phase review verdicts

| Phase | Preflight G8 | Review | Findings |
|---|---|---|---|
| A | EXECUTE plan-as-is | PASS (2 P2 + 1 P3) | P2-A-1 rand blocker upstream, P2-A-2 aggressive update lesson, P3-A-1 frost DKG serialization |
| B | EXECUTE plan-as-is | PASS (1 P2 + 1 P3) | P2-B-1 log convergence carry S35, P3-B-1 flaky browse quorum |
| C | EXECUTE plan-as-is | PASS (1 P2 + 1 P3) | P2-C-1 .icns absent carry S35, P3-C-1 .desktop Exec path |

## §4 Scope cuts respectés (kickoff §7)

1. VPS deployment effectif — ✅ non touché
2. Code signing macOS — ✅ non touché (right-click bypass documenté)
3. MSI/NSIS installer Windows — ✅ non touché
4. .deb/.rpm packages Linux — ✅ non touché
5. Auto-update mechanism — ✅ non touché
6. Tray icon / notification area — ✅ non touché
7. frost-ed25519 3.0 upgrade — ✅ INCLUS Phase A (0 LOC delta, byte-identical)
8. CI pipeline Linux — ✅ non touché
9. stop/status CLI — ✅ non touché
10. Cross-node task Ollama réel — ✅ non touché (stub)
11. Docker daemon/worker — ✅ non touché
12. P3 grammar/watermark wiring — ✅ non touché

## §5 Findings carry-over for memory

- P2-A-1 rand triple : blocker upstream (frost-core rand_core 0.6 + iroh stack), sous-arbres disjoints, re-évaluer si convergence upstream.
- P2-A-2 aggressive update lesson : `cargo update --aggressive` peut tirer RC transitives. PATTERNS.md carry S35.
- P2-B-1 log convergence : launcher.log + daemon log séparés, convergence carry S35.
- P2-B-1-S33 shellcheck CI : 2/3, carry S35 (pas de CI Linux).
- P2-B-2-S33 REPO_URL : 2/3, carry S35 (blocker externe repo public).
- P2-C-1-S33 cross-daemon E2E : 2/3, carry S35 (harness blob publication).
- P2-C-1 .icns macOS : .png fallback, .icns nécessite macOS ou outil tiers. Carry S35.
