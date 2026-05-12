# Sprint 60 — Audit plan (pour Sprint 61 Phase 0)

**Ecrit** : 2026-05-12 (Phase E Sprint 60)
**Scope attendu S60** : installer Windows NSIS + tray icon + LT-7
Tier 3 validation + cross-platform installer validation → tag v1.0
(end user ready)

---

## Track A — Phase integrity

1. Verifier que chaque phase A-C a son preflight G8 + review dans
   `.planning/active/sprint60_phase_{A..C}_{preflight,review}.md`
2. Verifier coherence delta tests cumule (verification.md §2 vs
   git log) : +2 Rust (Phase A tray tests), +0 Vitest
3. Verifier que les 4 fixes post-phases sont documentes :
   - `dd55bf6` fix tray Win32 message pump
   - `3b0f227` fix build-release.sh --locked
   - `a045502` fix cross-platform installers Packager.toml
   - `b6a93a8` fix Linux .deb GTK features + publisher
4. Verifier que Phase D (validation) est documentee dans
   `sprint60_lt7_tier3_report.md` avec evidence concrete
5. Verifier Phase E preflight G8 EXECUTE dans
   `sprint60_phase_E_preflight.md`

## Track B — Tray icon correctness

1. Module `tray.rs` : `create_tray()` + `run_event_loop()` +
   fallback `ctrl_c` si tray init echoue
2. `main.rs` : refactor message loop, tokio runtime en background
   thread, shutdown via oneshot channel
3. `--web-root` wiring : launcher passe le chemin au daemon spawn
4. Cross-platform : compile sur Linux/macOS (GTK/AppKit), fallback
   vers ctrl_c sans tray
5. Unit tests : 2 tests tray module (web_root resolution +
   menu creation)

## Track C — Installer correctness

1. `Packager.toml` : product-name, version, binaries (launcher +
   daemon), resources (web/dist), NSIS config (currentUser, languages)
2. `scripts/build-installer.sh` : pipeline complet (npm build →
   cargo build → cargo packager)
3. Windows NSIS : install silencieux /S, $LOCALAPPDATA, shortcut
   Start Menu, uninstall registry HKCU, ~17 MB
4. Linux .deb : dpkg install/remove, binaires /usr/bin/, frontend
   /usr/lib/nexus-launcher/web/, 26 MB
5. macOS .dmg : .app bundle /Applications/, Info.plist, .icns, 23 MB
6. AppImage scope cut post-v1.0 documente (linuxdeploy FUSE)

## Track D — LT-7 Tier 3 validation quality

1. Gossip 3 machines WAN : Win (Paris) + VPS (Helsinki) + Mac (Paris)
   — logs gossip avec node_id prefixes et IPs
2. API mutual discovery : `x-sbfb-token` header (pas `Authorization:
   Bearer`), subscribed_curators vu sur chaque noeud
3. Task submit : Ed25519 signee, redundancy_factor, persistee
   coordinator.db
4. Worker quorum carry post-tag : documenter pourquoi (workers non
   deployes), Tier 2 gate satisfait
5. Cross-OS SHA256 limitation (D4) : documenter que consensus
   intra-OS seulement

## Track E — Build pipeline + dette pair

1. `build-release.sh` : 3 crates (worker + daemon + launcher) +
   frontend npm build + copie dist/web/
2. P2-G-1 exe lock : FERME non reproductible (5 builds consecutifs
   OK, Get-Process clean)
3. PATTERNS.md : §P48 StorageWriteLimiter + §P49 Kudos-v2 + §P50
   tray icon
4. `--locked` flag ajoute dans build-release.sh (fix `3b0f227`)

## Track F — Tag v1.0 integrity

1. Tag `v1.0` present sur master au commit wrap-up
2. Tag annote : `git tag -a v1.0 -m "v1.0 — end user ready"`
3. CLAUDE.md : S60 CLOSED, tag v1.0, compteurs mis a jour
4. SPRINT_LOG.md : row S60 presente
5. HARDENING_ROADMAP.md : last_validated S60
6. ROADMAP_COMMITMENTS.md : LT-7 status mis a jour, LT-2 trigger
   note tag pose

## Track G — Carries residuels

| Item | Compteur S61 | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 21+/3 | exemption externe |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |
| P2-NSIS-UNINSTALL multi-binary | NEW 1/3 | Phase D report |

## Track H — Pre-launch → post-launch policy transition

1. Verifier que CLAUDE.md §Pre-launch protocol policy mentionne
   la bascule post-tag v1.0
2. Verifier que les *_VERSION restent a 1 dans le code (pas de
   bump accidentel)
3. Verifier que le canonical.rs n'a pas ete modifie S60 (Phase E
   ne touche pas le wire format)
