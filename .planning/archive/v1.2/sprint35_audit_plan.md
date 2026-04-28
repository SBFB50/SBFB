# Sprint 35 — Audit plan (audit gate S34→S35)

**Sprint audité** : S34 (UX launcher cross-platform + dette pair)
**Phases livrées** : A (dette MANDATORY rand + COEP E2E + frost 3.0), B (Windows launcher UX), C (macOS .app + Linux .desktop)
**Tip cloture** : `4776948`

## Track A — Phase A correctness (dette)

| # | Check | Fichiers | Quoi vérifier |
|---|---|---|---|
| A1 | COEP E2E test covers all 3 headers | `blob_serve_coep.rs` | Assert COOP + COEP + CSP all checked, not just one |
| A2 | frost-ed25519 3.0 signature byte-identical | `canary/mod.rs`, `canary/frost.rs` | CanarySigned v1 format preserved, standard Ed25519 verifier OK |
| A3 | frost 3.0 DKG ceremony functional | `canary/dkg.rs`, `ceremony.rs` | DKG round1/round2/aggregate tests still pass |
| A4 | rand triple documented not masked | `Cargo.lock` | `cargo tree -d \| grep rand` confirms 3 versions documented |
| A5 | cargo update aggressive revert clean | `Cargo.lock` | No RC or pre-release transitives in lock file |

## Track B — Phase B correctness (Windows UX)

| # | Check | Fichiers | Quoi vérifier |
|---|---|---|---|
| B1 | winresource build.rs compiles cross-platform | `build.rs` | `cfg(windows)` gate prevents Linux/macOS build failure |
| B2 | windows_subsystem conditional | `main.rs` | `cfg_attr(not(debug_assertions))` — debug mode has console |
| B3 | Log file setup before daemon spawn | `main.rs` | Log initialized before any spawn_daemon call |
| B4 | Panic hook writes to log file | `main.rs` | Custom panic hook registered, writes to same file |
| B5 | Icon asset valid | `assets/nexus-launcher.ico` | Multi-resolution ICO (16/32/48/256) |

## Track C — Phase C correctness (macOS/Linux)

| # | Check | Fichiers | Quoi vérifier |
|---|---|---|---|
| C1 | Info.plist CFBundleExecutable matches binary | `Info.plist` | `nexus-launcher` matches cargo target name |
| C2 | bundle-macos.sh creates valid structure | `bundle-macos.sh` | MacOS + Resources directories created, binary chmod +x |
| C3 | .desktop file freedesktop-compliant | `nexus-launcher.desktop` | Type, Name, Exec, Icon, Terminal=false all present |
| C4 | install-node.sh .desktop sed works | `install-node.sh` | Dynamic Exec path replacement produces valid path |
| C5 | .png icon 256x256 minimum | `assets/nexus-launcher.png` | Dimensions check (ImageMagick identify or file headers) |

## Track D — Cross-phase integration

| # | Check | Quoi vérifier |
|---|---|---|
| D1 | Launcher log path consistent across phases | B3 log path matches documented `~/.sbfb/launcher.log` |
| D2 | COEP test uses real daemon spawn | A1 test uses DaemonHandle not mock |
| D3 | frost 3.0 Cargo.lock consistent | Cargo.lock frost-ed25519 = 3.x, frost-core = 3.x |

## Track E — Security & hardening

| # | Check | Quoi vérifier |
|---|---|---|
| E1 | COEP headers on all blob-serve responses | blob_serve.rs constants BLOB_SERVE_COOP/COEP present |
| E2 | Launcher auth token read from correct SBFB_HOME | main.rs reads token after log setup, correct path |
| E3 | No secrets in launcher log | Log output doesn't write auth token or keypair material |

## Track F — Meta-process

| # | Check | Quoi vérifier |
|---|---|---|
| F1 | G8 preflight ran 3/3 phases | `sprint34_phase_{A,B,C}_preflight.md` exist, verdicts EXECUTE |
| F2 | Phase review ran 3/3 phases | `sprint34_phase_{A,B,C}_review.md` exist, verdicts PASS |
| F3 | Commit bodies have delta tests + scope cuts | All 3 feat commits have structured body |
| F4 | Carry counters incremented correctly | §6 carry-overs match review findings |
| F5 | MANDATORY items 3/3 resolved | P2-A-1 rand documented, P2-B-1 tor-rtcompat already FERME S33, P2-REVIEW-C-2 COEP E2E resolved |
